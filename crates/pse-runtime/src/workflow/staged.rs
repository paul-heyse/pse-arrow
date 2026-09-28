// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One staged-sequence primitive (A6; Plan 22 architecture §4, scenarios S14 and S15).
//!
//! Staged and block initialization, homotopy advance, studies and authored solve sequences
//! compose here; rolling horizons (Y5) and study workers (O7) build on it. A sequence:
//!
//! - prepares structure once and rebinds values per step: equal structures share one
//!   prepared view, its programs and its assessment, and only the value-dependent products
//!   are rebuilt, when a value they consumed changed (DP-09, DP-10);
//! - runs every step on one worker-owned native session, whose retained native state serves
//!   the next step when coordinates and profile match and the step's reuse policy allows
//!   (§17.6); a step that does not end accepted drops that state;
//! - composes each step's [`Overlay`] (stage selections, parameter replacements, temporary
//!   fixes, relaxations and values) over the immutable original specification. An overlay
//!   exists only inside its step, so nothing it changes survives the step, whether the step
//!   succeeds, fails or is cancelled (PS-08, DP-05);
//! - records each step's typed outcome and candidate-use decision. A later step starts
//!   only from a step whose decision permits it (ADR-0106, [`Start`]); a failed step seeds
//!   nothing and does not stop independent steps.
use super::{
    ModelingAnalysis, ModelingPackage, ModelingResult, ModelingSolvePreparation, Runtime,
    WorkflowError,
    modeling::assessment::Obligations,
    numerics::{CandidateDecision, CandidateReason, refused},
};
use crate::math::{
    NativeSession,
    solves::{Outcome, Predecessor},
};
use pse_backend_native::solve::{Progress, WarmStart};
use pse_compiler::workspace::ModelingVariableState;
use pse_ids::SemanticId;
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic};
use pse_modeling::specialize::Value;
use std::{collections::BTreeMap, future::Future, sync::Arc, time::Instant};

/// Where a step starts. Starts are typed; nothing is inferred from a label (F14, F25).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::workflow) enum Start {
    /// The specification's own values and start annotations.
    Specification,
    /// The solved values of an earlier step whose candidate is a result. Homotopy advance
    /// and stage chains start this way.
    Accepted(usize),
    /// An explicit dependency on an earlier step: a result, or a seed-only candidate that is
    /// never itself a result (studies).
    Seed(usize),
}
/// Temporary replacements composed over the original specification for one step only.
#[derive(Clone, Debug, Default)]
pub(in crate::workflow) struct Overlay {
    /// Specialization facts, such as a selected initialization stage.
    pub facts: BTreeMap<String, Value>,
    /// Replacements of declared parameters, such as continuation values.
    pub parameters: BTreeMap<SemanticId, f64>,
    /// Temporary fixes and relaxations, by case path; set fields replace the original's.
    pub variables: BTreeMap<String, ModelingVariableState>,
    /// Temporary case values, by path.
    pub values: BTreeMap<String, f64>,
}
impl Overlay {
    /// The step's specification. The original is only read (PS-08).
    fn compose(&self, original: &ModelingAnalysis) -> ModelingAnalysis {
        let mut step = original.clone();
        step.bindings
            .facts
            .extend(self.facts.iter().map(|(k, v)| (k.clone(), v.clone())));
        step.case
            .values
            .extend(self.values.iter().map(|(k, v)| (k.clone(), *v)));
        for (path, overlay) in &self.variables {
            let state = step.case.variables.entry(path.clone()).or_default();
            if overlay.fixed.is_some() {
                state.fixed = overlay.fixed;
            }
            if overlay.lower.is_some() {
                state.lower = overlay.lower;
            }
            if overlay.upper.is_some() {
                state.upper = overlay.upper;
            }
        }
        step
    }
}
/// What an executed step leaves for later steps.
#[derive(Clone, Debug)]
struct Record {
    decision: CandidateDecision,
    /// Complete values of the step with its solved coordinates.
    values: BTreeMap<SemanticId, f64>,
    /// Native output seed.
    warm: Option<WarmStart>,
}
/// One step's typed result and, when a deadline or cancellation stopped it, why.
#[derive(Clone, Debug)]
pub(in crate::workflow) struct StepRecord {
    pub result: Result<ModelingResult, Arc<WorkflowError>>,
    pub interruption: Option<BoundaryDiagnostic>,
}
/// One staged sequence over a native session.
#[derive(Debug)]
pub(in crate::workflow) struct Staged {
    runtime: Runtime,
    session: NativeSession,
    records: Vec<Record>,
    /// One event stream for every step (an authored sequence's run handle), or none: then
    /// every step reports its own.
    progress: Option<Arc<Progress>>,
}
impl Staged {
    /// Open a sequence on a new native session.
    ///
    /// # Errors
    /// No native job slot or pool capacity.
    pub(in crate::workflow) fn open(
        runtime: &Runtime,
        progress: Option<Arc<Progress>>,
    ) -> Result<Self, WorkflowError> {
        Ok(Self {
            runtime: runtime.clone(),
            session: runtime.native().open_session()?,
            records: Vec::new(),
            progress,
        })
    }
    /// The values a step starting at `start` is seeded with, by the candidate-use rule
    /// (ADR-0106): none for the specification; an accepted result's solved values; or, for
    /// an explicit dependency, a result's or a seed-only candidate's. `None` refuses the
    /// start.
    pub(in crate::workflow) fn seed(&self, start: Start) -> Option<BTreeMap<SemanticId, f64>> {
        let (index, permitted): (usize, fn(CandidateDecision) -> bool) = match start {
            Start::Specification => return Some(BTreeMap::new()),
            Start::Accepted(index) => (index, CandidateDecision::permits_use),
            Start::Seed(index) => (index, CandidateDecision::permits_seed),
        };
        self.records
            .get(index)
            .filter(|r| permitted(r.decision))
            .map(|r| r.values.clone())
    }
    /// The native output seed of the last step when its candidate is a result: what a
    /// `PreviousAccepted` start policy consumes (§17.6).
    pub(in crate::workflow) fn predecessor(&self) -> Option<Predecessor> {
        let attempt = self.records.len().checked_sub(1)?;
        let last = &self.records[attempt];
        last.decision
            .permits_use()
            .then(|| last.warm.clone().map(|seed| Predecessor { attempt, seed }))?
    }
    /// Record a step that ended before a native attempt. It seeds nothing.
    pub(in crate::workflow) fn refuse(&mut self) -> usize {
        self.records.push(Record {
            decision: refused(CandidateReason::NoCandidate),
            values: BTreeMap::new(),
            warm: None,
        });
        self.records.len() - 1
    }
    fn record(&mut self, result: &Result<ModelingResult, Arc<WorkflowError>>) -> usize {
        match result {
            Ok(result) => {
                self.records.push(Record {
                    decision: result.completion.decision,
                    values: result.values.scalars.clone(),
                    warm: match &result.outcome {
                        Outcome::Native(native) => native.warm_start.clone(),
                        Outcome::Constant(_) | Outcome::Rejected(_) => None,
                    },
                });
                self.records.len() - 1
            }
            Err(_) => self.refuse(),
        }
    }
    /// Execute one prepared step on the session, assess it against `obligations` on the
    /// same worker, and record it. `previous` is the native seed a `PreviousAccepted`
    /// start policy may consume.
    pub(in crate::workflow) async fn run(
        &mut self,
        prepared: ModelingSolvePreparation,
        obligations: Obligations,
        run_id: SemanticId,
        attempt: usize,
        previous: Option<Predecessor>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, Arc<WorkflowError>> {
        let result = self
            .execute(prepared, obligations, run_id, attempt, previous, cancel)
            .await
            .map_err(Arc::new);
        self.record(&result);
        result
    }
    async fn execute(
        &self,
        prepared: ModelingSolvePreparation,
        obligations: Obligations,
        run_id: SemanticId,
        attempt: usize,
        previous: Option<Predecessor>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        let assessment = prepared
            .source
            .assessment(&prepared, obligations, cancel)
            .await?;
        let service = self.runtime.native();
        let owner = service.reserve("math:solve-results", prepared.solve.result_bytes()?)?;
        let point_owner = service.reserve(
            "modeling:qualified-result",
            super::modeling::results::result_bytes(&prepared)?,
        )?;
        let progress = self
            .progress
            .clone()
            .unwrap_or_else(|| Arc::new(Progress::new(prepared.profile.controls.history)));
        let evaluated = prepared.clone();
        let (outcome, point) = self
            .session
            .step(
                prepared.solve.clone(),
                previous,
                attempt,
                progress,
                owner.clone(),
                cancel,
                move |outcome, flag, budget| {
                    let point = assessment.assess(
                        &evaluated,
                        run_id,
                        attempt,
                        outcome,
                        flag,
                        budget,
                        point_owner,
                    );
                    let accepted = point.accepted();
                    (point, accepted)
                },
            )
            .await?;
        Ok(ModelingResult::from_assessment(
            prepared, run_id, attempt, outcome, point, owner,
        ))
    }
    /// One step of `package`: `overlay` composed over `original`, seeded from `start`,
    /// bound to its prepared structure and executed within `deadline`. The deadline covers
    /// binding, native execution and assessment; an interruption cancels the step and waits
    /// for its native teardown. Every step is recorded, including one refused before it ran.
    #[expect(
        clippy::too_many_arguments,
        reason = "a step names its package, specification, overlay, start, obligations and bounds"
    )]
    pub(in crate::workflow) async fn step(
        &mut self,
        package: &ModelingPackage,
        original: &ModelingAnalysis,
        overlay: &Overlay,
        start: Start,
        obligations: Obligations,
        deadline: Option<Instant>,
        scope: &'static str,
        cancel: &crate::CancelSource,
    ) -> StepRecord {
        let Some(seed) = self.seed(start) else {
            self.refuse();
            let mut refusal = BoundaryDiagnostic::new(
                BoundaryClass::Conflict,
                scope,
                [original.root, original.instance],
                "modeling.staged.start",
            );
            if let Start::Accepted(index) | Start::Seed(index) = start {
                refusal.observations.insert(
                    "predecessor".into(),
                    pse_model::diagnostic::Observation::Integer(index as i64),
                );
            }
            return StepRecord {
                result: Err(Arc::new(refusal.into())),
                interruption: None,
            };
        };
        let mut step = overlay.compose(original);
        if let Some(deadline) = deadline {
            step.solver.controls.time_limit = step
                .solver
                .controls
                .time_limit
                .min(deadline.saturating_duration_since(Instant::now()));
        }
        let parameters = overlay.parameters.clone();
        let this = &*self;
        let (result, interruption) = bounded(scope, deadline, cancel, |child| async move {
            let prepared = package
                .prepare_analysis_attempt(&step, seed, parameters, &child)
                .await?;
            this.execute(
                prepared,
                obligations,
                pse_authoring::ids::uuid_v7(),
                0,
                None,
                &child,
            )
            .await
        })
        .await;
        let result = result.map_err(Arc::new);
        self.record(&result);
        StepRecord {
            result,
            interruption,
        }
    }
    /// Close the native session and wait for its thread to join.
    pub(in crate::workflow) async fn close(self) {
        self.session.close().await;
    }
}

/// Bound one step by an optional deadline and the driver's cancellation. The deadline
/// covers admission, compilation, native execution and qualification. Interrupted work is
/// cancelled and awaited, so its native teardown completes before the step reports.
pub(in crate::workflow) async fn bounded<T, F, Fut>(
    scope: &'static str,
    deadline: Option<Instant>,
    cancel: &crate::CancelSource,
    work: F,
) -> (Result<T, WorkflowError>, Option<BoundaryDiagnostic>)
where
    F: FnOnce(crate::CancelSource) -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
{
    let interruption = |class, rule| BoundaryDiagnostic::new(class, scope, [], rule);
    if cancel.token().is_cancelled() {
        return (Err(crate::math::MathRuntimeError::Cancelled.into()), None);
    }
    if deadline.is_some_and(|d| Instant::now() >= d) {
        return (
            Err(interruption(BoundaryClass::ResourceLimit, "analysis time limit").into()),
            None,
        );
    }
    let child = crate::CancelSource::new();
    let operation = work(child.clone());
    tokio::pin!(operation);
    let expiry = async {
        match deadline {
            Some(deadline) => {
                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
            }
            None => std::future::pending::<()>().await,
        }
    };
    let stopped = tokio::select! {
        biased;
        () = cancel.cancelled() => interruption(BoundaryClass::Cancelled, "analysis cancelled"),
        () = expiry => interruption(BoundaryClass::ResourceLimit, "analysis time limit"),
        result = &mut operation => return (result, None),
    };
    child.cancel();
    (operation.await, Some(stopped))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::tests::runtime;
    use pse_model::generated::enums::CandidateUse;
    use std::time::Duration;

    #[tokio::test]
    async fn step_deadline_and_cancellation_join_work() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let joined = &AtomicBool::new(false);
        let cancel = crate::CancelSource::new();
        let (result, interruption) = bounded(
            "initialization",
            Some(Instant::now() + Duration::from_millis(20)),
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
            bounded("initialization", None, &cancel, |child| async move {
                began.notify_one();
                child.cancelled().await;
                joined.store(true, Ordering::SeqCst);
                Ok(7)
            }),
            async {
                began.notified().await;
                cancel.cancel();
            }
        );
        assert_eq!(observed.0.unwrap(), 7);
        assert!(joined.load(Ordering::SeqCst));
        assert_eq!(observed.1.unwrap().class, BoundaryClass::Cancelled);
    }

    /// The A1 candidate-use rule for staged starts: a seed-only candidate seeds an explicit
    /// dependent (a study point) but never advances an accepted chain (homotopy), and a
    /// refused step seeds nothing.
    #[tokio::test]
    async fn staged_sequence_seeds_from_seed_only_predecessor() {
        let runtime = runtime();
        let mut staged = Staged::open(&runtime, None).unwrap();
        let x = SemanticId::from_bytes([3; 16]);
        let decision = |usability, reason| CandidateDecision { usability, reason };
        for (usability, reason, value) in [
            (
                CandidateUse::SeedOnly,
                CandidateReason::StoppedFeasible,
                1.5,
            ),
            (CandidateUse::Usable, CandidateReason::Accepted, 2.0),
            (
                CandidateUse::DiagnosticOnly,
                CandidateReason::LeastInfeasible,
                9.0,
            ),
        ] {
            staged.records.push(Record {
                decision: decision(usability, reason),
                values: BTreeMap::from([(x, value)]),
                warm: None,
            });
        }
        let refused = staged.refuse();
        assert_eq!(staged.seed(Start::Specification), Some(BTreeMap::new()));
        // Seed-only: an explicit dependent starts from it; an accepted chain does not.
        assert_eq!(
            staged.seed(Start::Seed(0)),
            Some(BTreeMap::from([(x, 1.5)]))
        );
        assert_eq!(staged.seed(Start::Accepted(0)), None);
        // A result serves both.
        assert_eq!(
            staged.seed(Start::Seed(1)),
            Some(BTreeMap::from([(x, 2.0)]))
        );
        assert_eq!(
            staged.seed(Start::Accepted(1)),
            Some(BTreeMap::from([(x, 2.0)]))
        );
        // Diagnostic-only, refused and unknown steps seed nothing.
        for start in [
            Start::Seed(2),
            Start::Accepted(2),
            Start::Seed(refused),
            Start::Seed(refused + 1),
        ] {
            assert_eq!(staged.seed(start), None, "{start:?}");
        }
        // `PreviousAccepted` reads only the last step, and only when it is a result.
        assert!(staged.predecessor().is_none());
        staged.close().await;
    }
}

#[cfg(all(test, feature = "solver-kinsol"))]
mod native_tests {
    use super::*;
    use crate::math::solves::NumericalInputs;
    use crate::workflow::{
        ModelingInitialization, ModelingStudyPoint, StartSource,
        modeling::assessment::Obligations,
        tests::{compiler_profile, physical, profile, runtime},
    };
    use pse_backend_native::solve::{Backend, ReusePolicy, SolverSelection};
    use pse_compiler::workspace::ModelingCaseBindings;
    use std::time::Duration;

    fn package_on(runtime: &Runtime, source: &str) -> (ModelingPackage, ModelingAnalysis) {
        let physical = physical();
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
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let mut solver = profile();
        solver.selection = SolverSelection::Explicit(Backend::Kinsol);
        solver.controls.reuse = ReusePolicy::AllowRebuild;
        let analysis = ModelingAnalysis {
            root,
            instance: root,
            bindings: Default::default(),
            limits: Default::default(),
            case: ModelingCaseBindings::default(),
            order: pse_kernels::DerivativeOrder::Second,
            compiler: compiler_profile(),
            solver,
            numerical: NumericalInputs::default(),
        };
        (package, analysis)
    }
    fn point(
        analysis: &ModelingAnalysis,
        t: f64,
        predecessor: Option<usize>,
    ) -> ModelingStudyPoint {
        let mut analysis = analysis.clone();
        analysis.case.values.insert("t".into(), t);
        ModelingStudyPoint {
            analysis: Ok(analysis),
            predecessor,
        }
    }
    fn native(result: &ModelingResult) -> &pse_backend_native::solve::SolveReport {
        match &result.outcome {
            Outcome::Native(native) => native,
            other => panic!("{other:?}"),
        }
    }
    fn symbol(result: &ModelingResult, name: &str) -> SemanticId {
        result
            .prepared
            .model
            .model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.rsplit('.').next() == Some(name))
            .unwrap()
            .id
    }
    fn x(result: &ModelingResult) -> f64 {
        result.values.scalars[&symbol(result, "x")]
    }
    const LINEAR: &str = "package p { def Root { param t: Scalar = 1; var x: Scalar; eq e: x == 2+t; annotation start x(2+t); annotation check x(x > 0); } }";

    /// An N-point study whose points differ only in values prepares its structure once;
    /// every later point rebinds values onto the prepared view and its programs (A6).
    #[tokio::test]
    async fn value_only_study_prepares_once() {
        let runtime = runtime();
        let cancel = crate::CancelSource::new();
        let (single, analysis) = package_on(&runtime, LINEAR);
        let before = runtime.native().preparations();
        let one = single
            .study(vec![point(&analysis, 1., None)], 8, &cancel)
            .await
            .unwrap();
        assert!(one.outcomes[0].as_ref().unwrap().accepted);
        let after_one = runtime.native().preparations();
        assert_eq!(after_one.views - before.views, 1);
        // A second package over the same runtime starts without prepared views.
        let (package, analysis) = package_on(&runtime, LINEAR);
        let start = runtime.native().preparations();
        let points = (1..=5)
            .map(|t| point(&analysis, f64::from(t), None))
            .collect();
        let report = package.study(points, 8, &cancel).await.unwrap();
        let end = runtime.native().preparations();
        for (t, outcome) in (1..=5).zip(&report.outcomes) {
            let result = outcome.as_ref().unwrap();
            assert!(result.accepted, "{:?}", result.diagnostic());
            assert!((x(result) - (2. + f64::from(t))).abs() < 1e-8);
        }
        // Exactly one structural preparation of the solver view, and no observation program
        // beyond what a single point needs.
        assert_eq!(end.views - start.views, 1);
        assert_eq!(
            end.observations - start.observations,
            after_one.observations - before.observations
        );
        // Each changed parameter rebuilt only the value-dependent products.
        assert_eq!(end.rebuilt - start.rebuilt, 4);
    }

    /// Homotopy steps change values only: they share one prepared structure and the
    /// retained native session, which every step after an accepted one reuses.
    #[tokio::test]
    async fn homotopy_steps_reuse_session() {
        let runtime = runtime();
        let (package, analysis) = package_on(
            &runtime,
            "package p { def Root { param t: Scalar = 100; var x: Scalar; eq residual: x*x == t; annotation start x(1); continue ramp on t from 1 to 100; } }",
        );
        let before = runtime.native().preparations();
        let report = package
            .initialize_model(
                &analysis,
                ModelingInitialization {
                    stages: vec![],
                    homotopy: true,
                    initial_step: 0.25,
                    minimum_step: 1e-6,
                    growth: 1.5,
                    maximum_attempts: 64,
                    time_limit: Duration::from_secs(60),
                },
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(report.completed, "{:?}", report.failure);
        let after = runtime.native().preparations();
        assert_eq!(after.views - before.views, 1);
        assert!(report.attempts.len() >= 3);
        let reused = report
            .attempts
            .iter()
            .map(|a| {
                native(a.result.as_ref().unwrap())
                    .evidence
                    .reused_native_state
            })
            .collect::<Vec<_>>();
        assert!(!reused[0], "the first step builds the session");
        for (index, attempt) in report.attempts.iter().enumerate().skip(1) {
            if report.attempts[index - 1].accepted() {
                assert!(reused[index], "{index}: {:?}", attempt.step);
            }
        }
        assert!(
            report
                .committed
                .as_ref()
                .unwrap()
                .values()
                .any(|v| (v - 10.).abs() < 1e-6)
        );
    }

    /// PS-08: a step's temporary fixes, relaxations and values exist only inside it. A
    /// failed or refused overlay step leaves the original specification in force for the
    /// next step and for the package, and seeds nothing.
    #[tokio::test]
    async fn initialization_restores_overlays_on_failure() {
        let runtime = runtime();
        let (package, mut analysis) = package_on(
            &runtime,
            "package p { def Root { var t: Scalar; var x: Scalar; var y: Scalar; eq e: x == 2+t; eq f: y == x; annotation start x(1); annotation start y(1); annotation check y(y > 0); } }",
        );
        analysis.case.values.insert("t".into(), 1.);
        analysis.case.variables.insert(
            "t".into(),
            ModelingVariableState {
                fixed: Some(true),
                ..Default::default()
            },
        );
        let cancel = crate::CancelSource::new();
        let before = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let mut staged = Staged::open(&runtime, None).unwrap();
        // Swap the specification: free `t`, fix `x` far below zero. The step solves, but the
        // model check fails, so it is not accepted.
        let swap = Overlay {
            variables: BTreeMap::from([
                (
                    "t".into(),
                    ModelingVariableState {
                        fixed: Some(false),
                        ..Default::default()
                    },
                ),
                (
                    "x".into(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                ),
            ]),
            values: BTreeMap::from([("x".into(), -9.)]),
            ..Overlay::default()
        };
        let failed = staged
            .step(
                &package,
                &analysis,
                &swap,
                Start::Specification,
                Obligations::Intermediate,
                None,
                "initialization",
                &cancel,
            )
            .await;
        let failed = failed.result.unwrap();
        assert!(!failed.accepted);
        assert!((x(&failed) + 9.).abs() < 1e-12);
        // A refused overlay (unknown path) fails before any native attempt.
        let refused = staged
            .step(
                &package,
                &analysis,
                &Overlay {
                    values: BTreeMap::from([("absent".into(), 1.)]),
                    ..Overlay::default()
                },
                Start::Specification,
                Obligations::Intermediate,
                None,
                "initialization",
                &cancel,
            )
            .await;
        assert!(refused.result.is_err());
        // Neither step seeds a successor.
        assert!(staged.seed(Start::Accepted(0)).is_none());
        assert!(staged.seed(Start::Accepted(1)).is_none());
        // The original specification is in force for the next step: `t` fixed, `x` free.
        let original = staged
            .step(
                &package,
                &analysis,
                &Overlay::default(),
                Start::Specification,
                Obligations::Final,
                None,
                "initialization",
                &cancel,
            )
            .await
            .result
            .unwrap();
        staged.close().await;
        assert!(original.accepted, "{:?}", original.diagnostic());
        assert!((x(&original) - 3.).abs() < 1e-8);
        let columns = original.prepared.model.case.compiled().plan.columns();
        assert!(
            columns.contains(&symbol(&original, "x")) && !columns.contains(&symbol(&original, "t"))
        );
        // And for the package.
        let after = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        assert_eq!(
            before.model.case.compiled().plan.structure().key(),
            after.model.case.compiled().plan.structure().key()
        );
        assert_eq!(before.model.values, after.model.values);
    }

    /// A failed study point is isolated: it drops the retained session and seeds nothing,
    /// independent points continue, and only its dependents are refused.
    #[tokio::test]
    async fn failed_point_isolated_in_study() {
        let runtime = runtime();
        let (package, analysis) = package_on(&runtime, LINEAR);
        let points = vec![
            point(&analysis, 1., None),
            point(&analysis, -5., None),
            point(&analysis, 2., None),
            point(&analysis, 3., None),
            point(&analysis, 4., Some(1)),
            point(&analysis, 5., Some(0)),
        ];
        let report = package
            .study(points, 8, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.unattempted, 0);
        let result = |i: usize| report.outcomes[i].as_ref().unwrap();
        assert!(result(0).accepted);
        // The failed point solved, but its model check refused the candidate.
        assert!(!result(1).accepted);
        assert!((x(result(1)) + 3.).abs() < 1e-8);
        // The next point rebuilt the dropped session; the one after reused it.
        assert!(result(2).accepted && !native(result(2)).evidence.reused_native_state);
        assert!(result(3).accepted && native(result(3)).evidence.reused_native_state);
        let refused = report.outcomes[4].as_ref().unwrap_err();
        assert_eq!(refused.rule, "modeling.study.predecessor");
        // A dependent of an accepted point starts from its solved values (typed start).
        assert!(result(5).accepted);
        let id = symbol(result(5), "x");
        assert_eq!(result(5).prepared.starts[&id], StartSource::Predecessor);
        assert!((x(result(5)) - 7.).abs() < 1e-8);
    }
}
