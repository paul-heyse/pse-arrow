// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Initialization and case studies as planners on the one staged-sequence primitive
//! (A6): each attempt composes its overlay over the immutable original specification, is
//! seeded only from a step whose candidate permits it, and shares prepared structure and the
//! native session with the other steps.
use super::assessment::Obligations;
use super::*;
use crate::math::solves::{NumericalInputs, SolverProfile};
use crate::workflow::staged::{Overlay, Staged, Start, bounded};
use pse_compiler::workspace::{ModelingCaseBindings, Profile};
use pse_kernels::DerivativeOrder;
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic};
use pse_modeling::specialize::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Complete selected model, specifications and analysis policies.
#[derive(Clone, Debug)]
pub struct ModelingAnalysis {
    /// Selected root declaration.
    pub root: DeclarationId,
    /// Identity of the instance the root becomes.
    pub instance: InstanceId,
    /// Specialization arguments, facts and demands.
    pub bindings: Bindings,
    /// Finite specialization limits.
    pub limits: Limits,
    /// Case values and variable states by path.
    pub case: ModelingCaseBindings,
    /// Prepared derivative order.
    pub order: DerivativeOrder,
    /// Evaluator profile.
    pub compiler: Profile,
    /// Solver selection, controls, settings and numerical policy.
    pub solver: SolverProfile,
    /// Authored numerical declarations and targets.
    pub numerical: NumericalInputs,
}
/// Stage names and optional declared homotopy, with finite work and recovery controls.
#[derive(Clone, Debug)]
pub struct ModelingInitialization {
    /// Declared stages, attempted in order before homotopy.
    pub stages: Vec<String>,
    /// Advance the declared continuation parameters from their start to their end.
    pub homotopy: bool,
    /// First homotopy step as a fraction of the path.
    pub initial_step: f64,
    /// Smallest step before a failed homotopy stops.
    pub minimum_step: f64,
    /// Step growth after an accepted homotopy step.
    pub growth: f64,
    /// Attempts across stages, homotopy and the original specification.
    pub maximum_attempts: usize,
    /// Wall-clock budget of the whole initialization.
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
    /// A named stage selected for this attempt only.
    Stage(String),
    /// Continuation parameters at this fraction of their path.
    Homotopy(f64),
    /// The unchanged requested specification.
    Original,
}
impl ModelingInitializationStep {
    /// Registry kind of this step.
    pub const fn kind(&self) -> pse_model::generated::enums::ModelingInitializationStep {
        use pse_model::generated::enums::ModelingInitializationStep as K;
        match self {
            Self::Stage(_) => K::Stage,
            Self::Homotopy(_) => K::Homotopy,
            Self::Original => K::Original,
        }
    }
}
/// Preparation failures and native results share one ordered history. An interrupted
/// attempt retains any native result produced while cancellation was joining.
#[derive(Clone, Debug)]
pub struct ModelingInitializationAttempt {
    /// The specification attempted.
    pub step: ModelingInitializationStep,
    /// The qualified result, or why none was produced.
    pub result: Result<ModelingResult, Arc<WorkflowError>>,
    /// Deadline or cancellation that stopped the attempt.
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
    /// The attempt ran to completion and its candidate is a result.
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
            Err(error) => matches!(
                error.boundary_diagnostic().class,
                BoundaryClass::TrialRejected | BoundaryClass::Numerical
            ),
            Ok(result) => match &result.outcome {
                Outcome::Rejected(error) => matches!(
                    WorkflowError::Math(crate::math::MathRuntimeError::Shared(error.clone()))
                        .boundary_diagnostic()
                        .class,
                    BoundaryClass::TrialRejected | BoundaryClass::Numerical
                ),
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
    /// Identity of this initialization run.
    pub run_id: SemanticId,
    pub(super) runtime: Runtime,
    /// Every attempt in order, including failed and interrupted ones.
    pub attempts: Vec<ModelingInitializationAttempt>,
    /// The original specification was solved and accepted.
    pub completed: bool,
    /// Why the initialization stopped before completing.
    pub failure: Option<BoundaryDiagnostic>,
    /// Last fully accepted original specification, absent after a failed initialization.
    pub committed: Option<BTreeMap<SemanticId, f64>>,
    pub(in crate::workflow::modeling) _owner: Arc<pse_columnar::AllocationLease>,
}

/// A predecessor is an explicit accepted-value dependency, never an implicit previous point.
#[derive(Clone, Debug)]
pub struct ModelingStudyPoint {
    /// The point's analysis, or why it could not be declared.
    pub analysis: Result<ModelingAnalysis, Arc<WorkflowError>>,
    /// Earlier point whose candidate seeds this one.
    pub predecessor: Option<usize>,
}
/// A point failure is represented independently of neighboring cases.
#[derive(Clone, Debug)]
pub struct ModelingStudyReport {
    /// Identity of this study run.
    pub run_id: SemanticId,
    pub(super) runtime: Runtime,
    pub(super) points: Vec<(Option<DeclarationId>, Option<InstanceId>, Option<usize>)>,
    /// Attempted points in order.
    pub outcomes: Vec<Result<ModelingResult, BoundaryDiagnostic>>,
    /// Points not attempted after cancellation.
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
impl ModelingInitialization {
    fn validate(&self) -> Result<(), WorkflowError> {
        if self.maximum_attempts == 0
            || self.maximum_attempts > 4096
            || self.stages.len() > 4096
            || self.time_limit.is_zero()
            || !self.initial_step.is_finite()
            || self.initial_step <= 0.
            || self.initial_step > 1.
            || !self.minimum_step.is_finite()
            || self.minimum_step <= 0.
            || self.minimum_step > self.initial_step
            || !self.growth.is_finite()
            || self.growth <= 1.
        {
            return Err(contract("invalid bounded initialization policy"));
        }
        Ok(())
    }
}
impl ModelingPackage {
    /// Bind an analysis to its prepared structure and values.
    pub async fn prepare_analysis(
        &self,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.prepare_analysis_attempt(analysis, BTreeMap::new(), BTreeMap::new(), cancel)
            .await
    }
    /// Bind one attempt: the analysis with a predecessor's seed and parameter replacements.
    /// Its structure is prepared once per package and rebound per values (A6).
    pub(in crate::workflow) async fn prepare_analysis_attempt(
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
    /// Independent points continue after failure; dependent points require their selected
    /// predecessor, whose candidate must permit seeding. All points run as one staged
    /// sequence: points of one structure prepare it once and rebind values (A6).
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
        let mut staged = Staged::open(&self.runtime, None)?;
        let mut outcomes: Vec<Result<ModelingResult, BoundaryDiagnostic>> = vec![];
        for p in points {
            if cancel.token().is_cancelled() {
                break;
            }
            outcomes.push(self.study_point(&mut staged, p, cancel).await);
        }
        staged.close().await;
        Ok(ModelingStudyReport {
            run_id: pse_authoring::ids::uuid_v7(),
            runtime: self.runtime.clone(),
            points: point_sources,
            unattempted: count - outcomes.len(),
            outcomes,
            _owner: owner,
        })
    }
    /// One study point. A failed point is recorded and isolated: it seeds nothing, and only
    /// points that name it as their predecessor are refused.
    async fn study_point(
        &self,
        staged: &mut Staged,
        point: ModelingStudyPoint,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, BoundaryDiagnostic> {
        let analysis = match point.analysis {
            Ok(analysis) => analysis,
            Err(error) => {
                staged.refuse();
                return Err(error.boundary_diagnostic());
            }
        };
        let start = point.predecessor.map_or(Start::Specification, Start::Seed);
        if let Some(previous) = point.predecessor
            && staged.seed(start).is_none()
        {
            staged.refuse();
            let mut error = BoundaryDiagnostic::new(
                BoundaryClass::Conflict,
                "modeling-study",
                [analysis.root.as_id(), analysis.instance.as_id()],
                "modeling.study.predecessor",
            );
            error.observations.insert(
                "predecessor".into(),
                pse_model::diagnostic::Observation::Integer(previous as i64),
            );
            return Err(error);
        }
        staged
            .step(
                self,
                &analysis,
                &Overlay::default(),
                start,
                Obligations::Final,
                None,
                "modeling-study",
                cancel,
            )
            .await
            .result
            .map_err(|error| error.boundary_diagnostic())
    }
    /// Accepted values advance through immutable stage overlays. Homotopy retries from
    /// the last accepted point, shrinks failures and qualifies the final original model.
    /// Every attempt is one step of a staged sequence (A6): homotopy steps change values
    /// only, so they share one prepared structure and the retained native session.
    pub async fn initialize_model(
        &self,
        analysis: &ModelingAnalysis,
        policy: ModelingInitialization,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingInitializationReport, WorkflowError> {
        policy.validate()?;
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
        let (base, interruption) = bounded(
            "initialization",
            Some(deadline),
            cancel,
            |child| async move {
                self.prepare(
                    analysis.root,
                    analysis.instance,
                    analysis.bindings.clone(),
                    analysis.limits,
                    &child,
                )
                .await
            },
        )
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
        let continuations = base.compiled().model.continuation.clone();
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
        let mut run = Initializer {
            package: self,
            analysis,
            policy: &policy,
            deadline,
            cancel,
            staged: Staged::open(&self.runtime, None)?,
            accepted: None,
            report: ModelingInitializationReport {
                run_id: pse_authoring::ids::uuid_v7(),
                runtime: self.runtime.clone(),
                attempts: Vec::with_capacity(policy.maximum_attempts),
                completed: false,
                failure: None,
                committed: None,
                _owner: owner,
            },
        };
        let _ = run.stages().await && run.homotopy(&continuations).await && run.original().await;
        let Initializer { staged, report, .. } = run;
        staged.close().await;
        Ok(report)
    }
}
/// The initialization planner over one staged sequence. Each method returns whether the
/// next phase may run; a stop records its cause in the report.
struct Initializer<'a> {
    package: &'a ModelingPackage,
    analysis: &'a ModelingAnalysis,
    policy: &'a ModelingInitialization,
    deadline: Instant,
    cancel: &'a crate::CancelSource,
    staged: Staged,
    /// The last accepted attempt, which seeds the next one.
    accepted: Option<usize>,
    report: ModelingInitializationReport,
}
impl Initializer<'_> {
    fn stop(&mut self, class: BoundaryClass, rule: &'static str) -> bool {
        self.report.failure = Some(BoundaryDiagnostic::new(
            class,
            "initialization",
            [self.analysis.root.as_id(), self.analysis.instance.as_id()],
            rule,
        ));
        false
    }
    /// One attempt: `overlay` over the original specification, seeded from the last
    /// accepted attempt. Returns whether it was accepted; the attempt is always reported.
    async fn attempt(
        &mut self,
        step: ModelingInitializationStep,
        overlay: Overlay,
    ) -> Option<(bool, bool)> {
        if self.report.attempts.len() == self.policy.maximum_attempts {
            self.stop(
                BoundaryClass::ResourceLimit,
                "modeling.initialization.attempt_limit",
            );
            return None;
        }
        let obligations = if step == ModelingInitializationStep::Original {
            Obligations::Final
        } else {
            Obligations::Intermediate
        };
        let record = self
            .staged
            .step(
                self.package,
                self.analysis,
                &overlay,
                self.accepted.map_or(Start::Specification, Start::Accepted),
                obligations,
                Some(self.deadline),
                "initialization",
                self.cancel,
            )
            .await;
        let attempt = ModelingInitializationAttempt {
            step,
            result: record.result,
            interruption: record.interruption,
        };
        let (accepted, retryable) = (attempt.accepted(), attempt.retryable());
        if accepted {
            self.accepted = Some(self.report.attempts.len());
        }
        self.report.attempts.push(attempt);
        Some((accepted, retryable))
    }
    fn failed_last(&mut self) -> bool {
        self.report.failure = self
            .report
            .attempts
            .last()
            .and_then(ModelingInitializationAttempt::diagnostic);
        false
    }
    /// Named stages in order; each stage's selection exists only inside its attempt.
    async fn stages(&mut self) -> bool {
        for stage in &self.policy.stages {
            let overlay = Overlay {
                facts: BTreeMap::from([(format!("stage.{stage}"), Value::Boolean(true))]),
                ..Overlay::default()
            };
            match self
                .attempt(ModelingInitializationStep::Stage(stage.clone()), overlay)
                .await
            {
                None => return false,
                Some((true, _)) => {}
                Some((false, _)) => return self.failed_last(),
            }
        }
        true
    }
    /// Bounded adaptive homotopy over the declared continuation endpoints: value-only
    /// steps from fraction zero to one, growing after acceptance and halving after a
    /// retryable failure down to the minimum step.
    async fn homotopy(
        &mut self,
        continuations: &BTreeMap<SemanticId, pse_modeling::specialize::Continuation>,
    ) -> bool {
        if !self.policy.homotopy {
            return true;
        }
        let mut progress = 0.;
        let mut step = self.policy.initial_step;
        let mut initial = true;
        loop {
            let fraction = if initial {
                0.
            } else {
                (progress + step).min(1.)
            };
            if !initial && fraction <= progress {
                return self.stop(
                    BoundaryClass::TrialRejected,
                    "modeling.initialization.step_precision",
                );
            }
            let parameters = match continuation_values(continuations, fraction) {
                Ok(parameters) => parameters,
                Err(error) => {
                    self.report.failure = Some(error.boundary_diagnostic());
                    return false;
                }
            };
            let overlay = Overlay {
                parameters,
                ..Overlay::default()
            };
            match self
                .attempt(ModelingInitializationStep::Homotopy(fraction), overlay)
                .await
            {
                None => return false,
                Some((true, _)) => {
                    progress = fraction;
                    if fraction == 1. {
                        return true;
                    }
                    if !initial {
                        step = (step * self.policy.growth).min(1. - progress);
                    }
                }
                Some((false, retryable)) => {
                    if initial || !retryable {
                        return self.failed_last();
                    }
                    step *= 0.5;
                    if step < self.policy.minimum_step {
                        return self.stop(
                            BoundaryClass::TrialRejected,
                            "modeling.initialization.minimum_step",
                        );
                    }
                }
            }
            initial = false;
        }
    }
    /// Only a separate solve of the unchanged requested specification may commit.
    async fn original(&mut self) -> bool {
        let Some((accepted, _)) = self
            .attempt(ModelingInitializationStep::Original, Overlay::default())
            .await
        else {
            return false;
        };
        self.report.completed = accepted;
        if accepted {
            self.report.committed = self
                .report
                .attempts
                .last()
                .and_then(|a| a.result.as_ref().ok())
                .map(|r| r.values.scalars.clone());
            true
        } else {
            self.failed_last()
        }
    }
}
/// Declared continuation parameters at `fraction` of the way to their endpoints.
fn continuation_values(
    continuations: &BTreeMap<SemanticId, pse_modeling::specialize::Continuation>,
    fraction: f64,
) -> Result<BTreeMap<SemanticId, f64>, WorkflowError> {
    continuations
        .iter()
        .map(|(id, c)| {
            let v = c.at(fraction).map_err(|e| contract(e.to_string()))?;
            let Value::Number { bits, .. } = v else {
                return Err(contract("physical continuation value"));
            };
            Ok((*id, f64::from_bits(bits)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_compiler::workspace::ModelingVariableState;
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
            instance: pse_modeling::specialize::root_instance(root),
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
        assert_eq!(diagnostic.class, BoundaryClass::TrialRejected);
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
            instance: pse_modeling::specialize::root_instance(root),
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
