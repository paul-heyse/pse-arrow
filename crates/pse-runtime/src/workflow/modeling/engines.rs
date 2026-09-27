// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded initialization and case studies; each attempt has its own immutable specification.
use super::*;
use crate::math::solves::{NumericalInputs, SolverProfile};
use pse_compiler::workspace::{ModelingCaseBindings, Profile};
use pse_kernels::DerivativeOrder;
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic};
use pse_modeling::specialize::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Complete selected model, specifications and analysis policies.
#[derive(Clone, Debug)]
pub struct ModelingAnalysis {
    pub root: SemanticId,
    pub instance: SemanticId,
    pub bindings: Bindings,
    pub limits: Limits,
    pub case: ModelingCaseBindings,
    pub order: DerivativeOrder,
    pub compiler: Profile,
    pub solver: SolverProfile,
    pub numerical: NumericalInputs,
}
/// Stage names and optional declared homotopy, with finite work and recovery controls.
#[derive(Clone, Debug)]
pub struct ModelingInitialization {
    pub stages: Vec<String>,
    pub homotopy: bool,
    pub initial_step: f64,
    pub minimum_step: f64,
    pub growth: f64,
    pub maximum_attempts: usize,
    pub time_limit: Duration,
}
impl Default for ModelingInitialization {
    fn default() -> Self {
        Self {
            stages: Vec::new(),
            homotopy: false,
            initial_step: 0.25,
            minimum_step: 1e-6,
            growth: 1.5,
            maximum_attempts: 128,
            time_limit: Duration::from_secs(60),
        }
    }
}
/// The immutable specification selected for one initialization attempt.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelingInitializationStep {
    Stage(String),
    Homotopy(f64),
    Original,
}
/// Preparation failures and native results share one ordered history. An interrupted
/// attempt retains any native result produced while cancellation was joining.
#[derive(Clone, Debug)]
pub struct ModelingInitializationAttempt {
    pub step: ModelingInitializationStep,
    pub result: Result<ModelingResult, Arc<WorkflowError>>,
    pub interruption: Option<BoundaryDiagnostic>,
}
impl ModelingInitializationAttempt {
    /// Structured cause of an interrupted or rejected attempt.
    pub fn diagnostic(&self) -> Option<BoundaryDiagnostic> {
        if self.accepted() {
            return None;
        }
        Some(
            self.interruption
                .clone()
                .unwrap_or_else(|| match &self.result {
                    Err(error) => error.boundary_diagnostic(),
                    Ok(result) => result.diagnostic().unwrap_or_else(|| {
                        BoundaryDiagnostic::new(
                            BoundaryClass::Internal,
                            "initialization",
                            [],
                            "modeling.initialization.outcome",
                        )
                    }),
                }),
        )
    }
    pub fn accepted(&self) -> bool {
        self.interruption.is_none() && self.result.as_ref().is_ok_and(|r| r.accepted)
    }
    fn retryable(&self) -> bool {
        if self.interruption.is_some() {
            return false;
        }
        use crate::math::solves::Outcome;
        use pse_backend_native::solve::Termination as T;
        match &self.result {
            Err(error) => error.boundary_diagnostic().class == BoundaryClass::TrialRejected,
            Ok(result) => match &result.outcome {
                Outcome::Rejected(error) => {
                    WorkflowError::Math(crate::math::MathRuntimeError::Shared(error.clone()))
                        .boundary_diagnostic()
                        .class
                        == BoundaryClass::TrialRejected
                }
                Outcome::Constant(_) => true,
                Outcome::Native(native) => {
                    matches!(
                        native.termination.category,
                        T::Success
                            | T::Acceptable
                            | T::FeasibleOnly
                            | T::Infeasible
                            | T::Inconclusive
                            | T::IterationLimit
                            | T::Numerical
                    ) || pse_backend_native::callback::retryable_evaluation(native)
                }
            },
        }
    }
}
/// Attempts retain their own report/specification. A failed attempt never publishes a seed.
#[derive(Clone, Debug)]
pub struct ModelingInitializationReport {
    pub run_id: SemanticId,
    pub(super) runtime: Runtime,
    pub attempts: Vec<ModelingInitializationAttempt>,
    pub completed: bool,
    pub failure: Option<BoundaryDiagnostic>,
    /// Last fully accepted original specification, absent after a failed initialization.
    pub committed: Option<BTreeMap<SemanticId, f64>>,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}

/// The deadline covers admission, compilation, native execution and qualification.
/// Cancel unique work and await its destruction before returning an interruption.
pub(super) async fn bounded_work<T, F, Fut>(
    scope: &'static str,
    deadline: Instant,
    cancel: &crate::CancelSource,
    work: F,
) -> (Result<T, WorkflowError>, Option<BoundaryDiagnostic>)
where
    F: FnOnce(crate::CancelSource) -> Fut,
    Fut: std::future::Future<Output = Result<T, WorkflowError>>,
{
    let interruption = |class, rule| BoundaryDiagnostic::new(class, scope, [], rule);
    if cancel.token().is_cancelled() {
        return (Err(crate::math::MathRuntimeError::Cancelled.into()), None);
    }
    if Instant::now() >= deadline {
        return (
            Err(interruption(BoundaryClass::ResourceLimit, "analysis time limit").into()),
            None,
        );
    }
    let child = crate::CancelSource::new();
    let operation = work(child.clone());
    tokio::pin!(operation);
    let stopped = tokio::select! {
        biased;
        ()=cancel.cancelled()=>interruption(BoundaryClass::Cancelled,"analysis cancelled"),
        ()=tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))=>interruption(BoundaryClass::ResourceLimit,"analysis time limit"),
        result=&mut operation=>return(result,None),
    };
    child.cancel();
    (operation.await, Some(stopped))
}
/// A predecessor is an explicit accepted-value dependency, never an implicit previous point.
#[derive(Clone, Debug)]
pub struct ModelingStudyPoint {
    pub analysis: Result<ModelingAnalysis, Arc<WorkflowError>>,
    pub predecessor: Option<usize>,
}
/// A point failure is represented independently of neighboring cases.
#[derive(Clone, Debug)]
pub struct ModelingStudyReport {
    pub run_id: SemanticId,
    pub(super) runtime: Runtime,
    pub(super) points: Vec<(Option<SemanticId>, Option<SemanticId>, Option<usize>)>,
    pub outcomes: Vec<Result<ModelingResult, BoundaryDiagnostic>>,
    pub unattempted: usize,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}
pub(super) fn bounded_error(error: impl std::fmt::Display) -> String {
    use std::fmt::Write;
    struct Message {
        text: String,
        truncated: bool,
    }
    impl Write for Message {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            let keep = value.floor_char_boundary(4093usize.saturating_sub(self.text.len()));
            self.text.push_str(&value[..keep]);
            self.truncated |= keep < value.len();
            Ok(())
        }
    }
    let mut message = Message {
        text: String::with_capacity(4096),
        truncated: false,
    };
    let _ = write!(message, "{error}");
    if message.truncated {
        message.text.push_str("...");
    }
    message.text
}
impl ModelingPackage {
    pub async fn prepare_analysis(
        &self,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.prepare_analysis_seed(analysis, BTreeMap::new(), cancel)
            .await
    }
    async fn prepare_analysis_seed(
        &self,
        a: &ModelingAnalysis,
        seed: BTreeMap<SemanticId, f64>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.prepare_analysis_attempt(a, seed, BTreeMap::new(), cancel)
            .await
    }
    pub(super) async fn prepare_analysis_attempt(
        &self,
        a: &ModelingAnalysis,
        seed: BTreeMap<SemanticId, f64>,
        parameters: BTreeMap<SemanticId, f64>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.prepare_solve_seed(
            a.root,
            a.instance,
            a.bindings.clone(),
            a.limits,
            a.case.clone(),
            a.order,
            a.compiler,
            a.solver.clone(),
            a.numerical.clone(),
            seed,
            parameters,
            cancel,
        )
        .await
    }
    /// Independent points continue after failure; dependent points require their selected predecessor.
    pub async fn study(
        &self,
        points: Vec<ModelingStudyPoint>,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingStudyReport, WorkflowError> {
        if maximum_points == 0 || maximum_points > 4096 || points.len() > maximum_points {
            return Err(contract("bounded study extent"));
        }
        for (i, p) in points.iter().enumerate() {
            if p.predecessor.is_some_and(|j| j >= i) {
                return Err(contract("study predecessor must identify an earlier point"));
            }
        }
        let bytes = points
            .len()
            .checked_mul(size_of::<Result<ModelingResult, BoundaryDiagnostic>>() + 4096)
            .ok_or_else(|| contract("study outcome extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:study-outcomes", bytes)?;
        let mut outcomes: Vec<Result<ModelingResult, BoundaryDiagnostic>> = vec![];
        let count = points.len();
        let point_sources = points
            .iter()
            .map(|p| {
                (
                    p.analysis.as_ref().ok().map(|a| a.root),
                    p.analysis.as_ref().ok().map(|a| a.instance),
                    p.predecessor,
                )
            })
            .collect();
        for p in points {
            if cancel.token().is_cancelled() {
                break;
            }
            let analysis = match p.analysis {
                Ok(analysis) => analysis,
                Err(error) => {
                    outcomes.push(Err(error.boundary_diagnostic()));
                    continue;
                }
            };
            let seed = if let Some(previous) = p.predecessor {
                match &outcomes[previous] {
                    Ok(result) if result.accepted => result.values.scalars.clone(),
                    _ => {
                        let mut error = BoundaryDiagnostic::new(
                            BoundaryClass::Conflict,
                            "modeling-study",
                            [analysis.root, analysis.instance],
                            "modeling.study.predecessor",
                        );
                        error.observations.insert(
                            "predecessor".into(),
                            pse_model::diagnostic::Observation::Integer(previous as i64),
                        );
                        outcomes.push(Err(error));
                        continue;
                    }
                }
            } else {
                BTreeMap::new()
            };
            let result = match self.prepare_analysis_seed(&analysis, seed, cancel).await {
                Ok(prepared) => self
                    .solve_case(prepared, analysis.compiler, cancel)
                    .await
                    .map_err(|error| error.boundary_diagnostic()),
                Err(error) => Err(error.boundary_diagnostic()),
            };
            outcomes.push(result);
        }
        Ok(ModelingStudyReport {
            run_id: pse_authoring::ids::uuid_v7(),
            runtime: self.runtime.clone(),
            points: point_sources,
            unattempted: count - outcomes.len(),
            outcomes,
            _owner: owner,
        })
    }
    /// Accepted values advance through immutable stage overlays. Homotopy retries from
    /// the last accepted point, shrinks failures and qualifies the final original model.
    pub async fn initialize_model(
        &self,
        analysis: &ModelingAnalysis,
        policy: ModelingInitialization,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingInitializationReport, WorkflowError> {
        if policy.maximum_attempts == 0
            || policy.maximum_attempts > 4096
            || policy.stages.len() > 4096
            || policy.time_limit.is_zero()
            || !policy.initial_step.is_finite()
            || policy.initial_step <= 0.
            || policy.initial_step > 1.
            || !policy.minimum_step.is_finite()
            || policy.minimum_step <= 0.
            || policy.minimum_step > policy.initial_step
            || !policy.growth.is_finite()
            || policy.growth <= 1.
        {
            return Err(contract("invalid bounded initialization policy"));
        }
        if analysis
            .bindings
            .facts
            .keys()
            .any(|k| k.starts_with("stage."))
        {
            return Err(contract(
                "initialization owns stage selection; the final analysis must name the original specification",
            ));
        }
        let deadline = Instant::now()
            .checked_add(policy.time_limit)
            .ok_or_else(|| contract("initialization deadline overflow"))?;
        let (base, interruption) =
            bounded_work("initialization", deadline, cancel, |child| async move {
                self.prepare(
                    analysis.root,
                    analysis.instance,
                    analysis.bindings.clone(),
                    analysis.limits,
                    &child,
                )
                .await
            })
            .await;
        if let Some(error) = interruption {
            return Err(error.into());
        }
        let base = base?;
        let stages = base
            .compiled()
            .model
            .instances
            .values()
            .flat_map(|i| i.stages.iter().map(String::as_str))
            .collect::<std::collections::BTreeSet<_>>();
        if policy.stages.iter().any(|s| !stages.contains(s.as_str())) {
            return Err(contract("unknown initialization stage"));
        }
        let continuations = &base.compiled().model.continuation;
        if policy.homotopy && continuations.is_empty() {
            return Err(contract(
                "homotopy requires declared continuation endpoints",
            ));
        }
        let names = policy
            .stages
            .iter()
            .try_fold(0usize, |n, s| n.checked_add(s.len()))
            .ok_or_else(|| contract("initialization name extent"))?;
        let bytes = policy
            .maximum_attempts
            .checked_mul(size_of::<ModelingInitializationAttempt>() + 4096)
            .and_then(|n| n.checked_add(names))
            .and_then(|n| n.checked_add(base.compiled().admitted.inputs.len().checked_mul(128)?))
            .ok_or_else(|| contract("initialization report extent"))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("modeling:initialization", bytes)?;
        let mut report = ModelingInitializationReport {
            run_id: pse_authoring::ids::uuid_v7(),
            runtime: self.runtime.clone(),
            attempts: Vec::with_capacity(policy.maximum_attempts),
            completed: false,
            failure: None,
            committed: None,
            _owner: owner,
        };
        let mut seed = BTreeMap::new();
        for stage in &policy.stages {
            if report.attempts.len() == policy.maximum_attempts {
                report.failure = Some(BoundaryDiagnostic::new(
                    BoundaryClass::ResourceLimit,
                    "initialization",
                    [analysis.root, analysis.instance],
                    "modeling.initialization.attempt_limit",
                ));
                return Ok(report);
            }
            let mut trial = analysis.clone();
            trial
                .bindings
                .facts
                .insert(format!("stage.{stage}"), Value::Boolean(true));
            let attempt = self
                .initialization_attempt(
                    &trial,
                    ModelingInitializationStep::Stage(stage.clone()),
                    seed.clone(),
                    BTreeMap::new(),
                    deadline,
                    cancel,
                )
                .await;
            let accepted = attempt.accepted();
            if accepted {
                seed = attempt
                    .result
                    .as_ref()
                    .map(|r| r.values.scalars.clone())
                    .unwrap_or_default();
            } else {
                report.failure = attempt.diagnostic();
            }
            report.attempts.push(attempt);
            if !accepted {
                return Ok(report);
            }
        }
        if policy.homotopy {
            let mut progress = 0.;
            let mut step = policy.initial_step;
            let mut initial = true;
            loop {
                if report.attempts.len() == policy.maximum_attempts {
                    report.failure = Some(BoundaryDiagnostic::new(
                        BoundaryClass::ResourceLimit,
                        "initialization",
                        [analysis.root, analysis.instance],
                        "modeling.initialization.attempt_limit",
                    ));
                    return Ok(report);
                }
                let fraction = if initial {
                    0.
                } else {
                    (progress + step).min(1.)
                };
                if !initial && fraction <= progress {
                    report.failure = Some(BoundaryDiagnostic::new(
                        BoundaryClass::TrialRejected,
                        "initialization",
                        [analysis.root, analysis.instance],
                        "modeling.initialization.step_precision",
                    ));
                    return Ok(report);
                }
                let parameters = continuations
                    .iter()
                    .map(|(id, c)| {
                        let v = c.at(fraction).map_err(|e| contract(e.to_string()))?;
                        let Value::Number { bits, .. } = v else {
                            return Err(contract("physical continuation value"));
                        };
                        Ok((*id, f64::from_bits(bits)))
                    })
                    .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
                let attempt = self
                    .initialization_attempt(
                        analysis,
                        ModelingInitializationStep::Homotopy(fraction),
                        seed.clone(),
                        parameters,
                        deadline,
                        cancel,
                    )
                    .await;
                let accepted = attempt.accepted();
                let retryable = attempt.retryable();
                if accepted {
                    seed = attempt
                        .result
                        .as_ref()
                        .map(|r| r.values.scalars.clone())
                        .unwrap_or_default();
                }
                report.attempts.push(attempt);
                if accepted {
                    progress = fraction;
                    if fraction == 1. {
                        break;
                    }
                    if !initial {
                        step = (step * policy.growth).min(1. - progress);
                    }
                } else {
                    if initial || !retryable {
                        report.failure = report
                            .attempts
                            .last()
                            .and_then(ModelingInitializationAttempt::diagnostic);
                        return Ok(report);
                    }
                    step *= 0.5;
                    if step < policy.minimum_step {
                        report.failure = Some(BoundaryDiagnostic::new(
                            BoundaryClass::TrialRejected,
                            "initialization",
                            [analysis.root, analysis.instance],
                            "modeling.initialization.minimum_step",
                        ));
                        return Ok(report);
                    }
                }
                initial = false;
            }
        }
        // Only a separate solve of the unchanged requested specification may commit.
        if report.attempts.len() == policy.maximum_attempts {
            report.failure = Some(BoundaryDiagnostic::new(
                BoundaryClass::ResourceLimit,
                "initialization",
                [analysis.root, analysis.instance],
                "modeling.initialization.attempt_limit",
            ));
            return Ok(report);
        }
        let attempt = self
            .initialization_attempt(
                analysis,
                ModelingInitializationStep::Original,
                seed,
                BTreeMap::new(),
                deadline,
                cancel,
            )
            .await;
        report.completed = attempt.accepted();
        if report.completed {
            report.committed = attempt
                .result
                .as_ref()
                .ok()
                .map(|r| r.values.scalars.clone());
        } else {
            report.failure = attempt.diagnostic();
        }
        report.attempts.push(attempt);
        Ok(report)
    }
    async fn initialization_attempt(
        &self,
        analysis: &ModelingAnalysis,
        step: ModelingInitializationStep,
        seed: BTreeMap<SemanticId, f64>,
        parameters: BTreeMap<SemanticId, f64>,
        deadline: Instant,
        cancel: &crate::CancelSource,
    ) -> ModelingInitializationAttempt {
        let original = matches!(step, ModelingInitializationStep::Original);
        let (result, interruption) =
            bounded_work("initialization", deadline, cancel, |child| async move {
                let mut trial = analysis.clone();
                trial.solver.controls.time_limit = trial
                    .solver
                    .controls
                    .time_limit
                    .min(deadline.saturating_duration_since(Instant::now()));
                let prepared = self
                    .prepare_analysis_attempt(&trial, seed, parameters, &child)
                    .await?;
                if original {
                    self.solve_case(prepared, trial.compiler, &child).await
                } else {
                    self.solve_initialization_trial(prepared, trial.compiler, &child)
                        .await
                }
            })
            .await;
        ModelingInitializationAttempt {
            step,
            result: result.map_err(Arc::new),
            interruption,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_compiler::workspace::ModelingVariableState;
    #[tokio::test]
    async fn kernel_initialization_deadline_and_cancellation_join_work() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let joined = &AtomicBool::new(false);
        let cancel = crate::CancelSource::new();
        let (result, interruption) = bounded_work(
            "initialization",
            Instant::now() + Duration::from_millis(20),
            &cancel,
            |child| async move {
                child.cancelled().await;
                joined.store(true, Ordering::SeqCst);
                Ok(42)
            },
        )
        .await;
        assert_eq!(result.unwrap(), 42);
        assert!(joined.load(Ordering::SeqCst));
        assert_eq!(interruption.unwrap().class, BoundaryClass::ResourceLimit);
        joined.store(false, Ordering::SeqCst);
        let began = &tokio::sync::Notify::new();
        let (observed, ()) = tokio::join!(
            bounded_work(
                "initialization",
                Instant::now() + Duration::from_secs(10),
                &cancel,
                |child| async move {
                    began.notify_one();
                    child.cancelled().await;
                    joined.store(true, Ordering::SeqCst);
                    Ok(7)
                }
            ),
            async {
                began.notified().await;
                cancel.cancel();
            }
        );
        assert_eq!(observed.0.unwrap(), 7);
        assert!(joined.load(Ordering::SeqCst));
        assert_eq!(observed.1.unwrap().class, BoundaryClass::Cancelled);
    }
    #[cfg(feature = "solver-kinsol")]
    fn initialization_summary(report: &ModelingInitializationReport) -> String {
        format!(
            "failure={:?}; attempts={:?}",
            report.failure,
            report
                .attempts
                .iter()
                .map(|a| (
                    &a.step,
                    a.accepted(),
                    match &a.result {
                        Err(e) => e.to_string(),
                        Ok(r) => match &r.outcome {
                            crate::math::solves::Outcome::Native(n) =>
                                format!("{:?}; {}", n.termination.category, n.termination.name),
                            crate::math::solves::Outcome::Rejected(e) => e.to_string(),
                            crate::math::solves::Outcome::Constant(_) => "constant".into(),
                        },
                    }
                ))
                .collect::<Vec<_>>()
        )
    }
    #[cfg(feature = "solver-kinsol")]
    fn continuation_fixture(
        source: &str,
        iterations: u32,
    ) -> (ModelingPackage, ModelingAnalysis, ModelingInitialization) {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical, names).unwrap();
        let mut solver = super::super::super::tests::profile();
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        solver.controls.iterations = iterations;
        let analysis = ModelingAnalysis {
            root,
            instance: root,
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: ModelingCaseBindings::default(),
            order: DerivativeOrder::Second,
            compiler: super::super::super::tests::compiler_profile(),
            solver,
            numerical: NumericalInputs::default(),
        };
        let policy = ModelingInitialization {
            stages: vec![],
            homotopy: true,
            initial_step: 1.,
            minimum_step: 1e-6,
            growth: 1.5,
            maximum_attempts: 256,
            time_limit: Duration::from_secs(60),
        };
        (package, analysis, policy)
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn kernel_initialization_retries_typed_preparation_rejections() {
        let (package, analysis, policy) = continuation_fixture(
            "package p { def Root { param t: Scalar = 1; var x: Scalar; eq residual: log(x-t) == 0; annotation start x(1); annotation nominal x(1+log(x-t)*log(x-t)); continue ramp on t from 0 to 1; } }",
            100,
        );
        let report = package
            .initialize_model(&analysis, policy.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.completed, "{}", initialization_summary(&report));
        assert!(
            report.attempts.iter().any(|a| a
                .result
                .as_ref()
                .is_err_and(|e| e.boundary_diagnostic().class == BoundaryClass::TrialRejected)),
            "{}",
            initialization_summary(&report)
        );
        assert!(
            report
                .committed
                .as_ref()
                .unwrap()
                .values()
                .any(|v| (v - 2.).abs() < 1e-6)
        );
        assert_eq!(
            report.attempts.last().unwrap().step,
            ModelingInitializationStep::Original
        );
        // Failed preparations count toward the attempt cap and never commit a partial result.
        let capped = package
            .initialize_model(
                &analysis,
                ModelingInitialization {
                    maximum_attempts: 2,
                    ..policy
                },
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(capped.attempts.len(), 2);
        assert!(!capped.completed);
        assert!(capped.committed.is_none());
        assert!(capped.attempts[1].result.is_err());
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn kernel_initialization_shrinks_failed_native_nonlinear_steps() {
        let (package, analysis, policy) = continuation_fixture(
            "package p { def Root { param t: Scalar = 100; var x: Scalar; eq residual: x*x == t; annotation start x(1); continue ramp on t from 1 to 100; } }",
            12,
        );
        let report = package
            .initialize_model(&analysis, policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.completed, "{}", initialization_summary(&report));
        assert!(report.attempts.iter().any(|a|!a.accepted() && matches!(&a.result,Ok(r) if matches!(r.outcome,crate::math::solves::Outcome::Native(_)))),"{}", initialization_summary(&report));
        assert!(
            report
                .committed
                .as_ref()
                .unwrap()
                .values()
                .any(|v| (v - 10.).abs() < 1e-6)
        );
        let original = report.attempts.last().unwrap().result.as_ref().unwrap();
        assert!(original.accepted);
        assert_eq!(
            original
                .prepared
                .model
                .model
                .compiled()
                .model
                .continuation
                .len(),
            1
        );
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn kernel_initialization_retries_callback_trials_without_event_history() {
        let (package, mut analysis, policy) = continuation_fixture(
            "package p { def Root { param t: Scalar = 1; var x: Scalar; eq residual: log(x-t) == 0; annotation start x(1); continue ramp on t from 0 to 1; } }",
            100,
        );
        analysis.solver.controls.history = 0;
        let report = package
            .initialize_model(&analysis, policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.completed, "{}", initialization_summary(&report));
        assert!(report.attempts.iter().any(|a|matches!(&a.result,Ok(r) if matches!(&r.outcome,crate::math::solves::Outcome::Native(n) if pse_backend_native::callback::retryable_evaluation(n) && n.events.is_empty()))));
        let failed=report.attempts.iter().filter_map(|a|a.result.as_ref().ok()).find(|r|
            matches!(&r.outcome,crate::math::solves::Outcome::Native(n) if n.callback_failure().is_some())).unwrap();
        let diagnostic = failed.diagnostic().unwrap();
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::TrialRejected
        );
        assert!(!diagnostic.sources.is_empty());
        assert!(matches!(
            diagnostic.rule.as_str(),
            "math.domain" | "math.provider"
        ));
        assert!(
            report
                .committed
                .as_ref()
                .unwrap()
                .values()
                .any(|v| (v - 2.).abs() < 1e-6)
        );
    }
    #[tokio::test]
    async fn kernel_study_isolates_failures_and_initialization_restores_original_specification() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let rows=pse_authoring::language::parse(
            "package p { def Root { param t: Scalar = 1; var x: Scalar; eq e: x == 2+t; annotation start x(2+t); annotation check x(x >= 2); continue ramp on t from 0 to 1; stage easy { override eq e: x == 2+t; } stage wrong { override eq e: x == -1; } } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical, names).unwrap();
        let analysis = ModelingAnalysis {
            root,
            instance: root,
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: ModelingCaseBindings {
                values: BTreeMap::new(),
                variables: BTreeMap::from([(
                    "x".into(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                )]),
            },
            order: DerivativeOrder::Second,
            compiler: super::super::super::tests::compiler_profile(),
            solver: super::super::super::tests::profile(),
            numerical: NumericalInputs::default(),
        };
        let cancel = crate::CancelSource::new();
        let before = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let mut failing = analysis.clone();
        failing.case.values.insert("x".into(), 0.);
        let study = package
            .study(
                vec![
                    ModelingStudyPoint {
                        analysis: Ok(analysis.clone()),
                        predecessor: None,
                    },
                    ModelingStudyPoint {
                        analysis: Ok(failing),
                        predecessor: None,
                    },
                    ModelingStudyPoint {
                        analysis: Ok(analysis.clone()),
                        predecessor: None,
                    },
                    ModelingStudyPoint {
                        analysis: Ok(analysis.clone()),
                        predecessor: Some(1),
                    },
                ],
                4,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(study.unattempted, 0);
        assert!(study.outcomes[0].as_ref().unwrap().accepted);
        assert!(!study.outcomes[1].as_ref().unwrap().accepted);
        assert!(study.outcomes[2].as_ref().unwrap().accepted);
        assert!(study.outcomes[3].is_err());
        let policy = ModelingInitialization {
            stages: vec!["easy".into()],
            homotopy: true,
            initial_step: 0.25,
            minimum_step: 0.01,
            growth: 2.,
            maximum_attempts: 10,
            time_limit: Duration::from_secs(20),
        };
        let initialized = package
            .initialize_model(&analysis, policy.clone(), &cancel)
            .await
            .unwrap();
        assert!(initialized.completed, "{:?}", initialized.failure);
        assert!(initialized.committed.is_some());
        assert_eq!(
            initialized
                .attempts
                .iter()
                .map(|a| a.step.clone())
                .collect::<Vec<_>>(),
            vec![
                ModelingInitializationStep::Stage("easy".into()),
                ModelingInitializationStep::Homotopy(0.),
                ModelingInitializationStep::Homotopy(0.25),
                ModelingInitializationStep::Homotopy(0.75),
                ModelingInitializationStep::Homotopy(1.),
                ModelingInitializationStep::Original
            ]
        );
        let mut bad = policy;
        bad.stages = vec!["wrong".into()];
        let rejected = package
            .initialize_model(&analysis, bad, &cancel)
            .await
            .unwrap();
        assert!(!rejected.completed);
        assert!(rejected.committed.is_none());
        let after = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        assert_eq!(
            before.model.case.compiled().plan.structure().key(),
            after.model.case.compiled().plan.structure().key()
        );
        assert_eq!(before.model.values, after.model.values);
    }
}
