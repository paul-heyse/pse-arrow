// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public Root request, physical response publication and independent withholding.
#![allow(
    clippy::unwrap_used,
    reason = "root response oracles require the original variable, primal response and numerical policy rows from their fixture"
)]
use super::results::{PortablePrediction, PredictionSample};
use super::*;
use crate::math::{settings::SensitivityRequest, solves::NumericalInputs};
use crate::workflow::tests as fixture;
use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
use pse_kernels::DerivativeOrder;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::NumericalTarget,
        runtime::{local_validity, parametric_sensitivities, solve_runs},
    },
};
type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn original_variable(prepared: &ModelingSolvePreparation, path: &str) -> SemanticId {
    let model = &prepared.model.model.compiled().model;
    let mut roots = model
        .instances
        .values()
        .filter(|instance| instance.parent.is_none());
    let root = roots.next().unwrap();
    assert!(roots.next().is_none());
    let lineage = format!("{}.{path}", root.path);
    let id = model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path == lineage)
        .unwrap()
        .id;
    assert!(prepared.model.case.compiled().plan.columns().contains(&id));
    id
}
fn variable_budget_from_prepared(prepared: &ModelingSolvePreparation, path: &str) -> f64 {
    let id = original_variable(prepared, path);
    fixture::engineering_target(prepared.solve.numerics(), NumericalTarget::Variable, id).budget
}
fn variable_budget(result: &ModelingResult, path: &str) -> f64 {
    variable_budget_from_prepared(&result.prepared, path)
}
fn assert_published_root_response(step: &ModelingResult, response: &parametric_sensitivities::Row) {
    let x = step.values.scalars[&response.target_id];
    let p = step.values.scalars[&response.parameter_id];
    let derivative = response.primal.unwrap();
    let target = fixture::engineering_target(
        step.prepared.solve.numerics(),
        NumericalTarget::Variable,
        response.target_id,
    );
    assert!(derivative.is_finite());
    // This checks the original square equation's linear action at the actual
    // returned point. Floating arithmetic identity does not demand a finer solve.
    let parameter = fixture::engineering_target(
        step.prepared.solve.numerics(),
        NumericalTarget::Variable,
        response.parameter_id,
    );
    let row = step
        .prepared
        .solve
        .numerics()
        .targets
        .iter()
        .find(|row| row.kind == NumericalTarget::Row)
        .unwrap();
    let sx = target.coordinate_scale;
    let sr = row.coordinate_scale;
    let sp = parameter.coordinate_scale;
    let normalized_action = derivative * sp / sx;
    let normalized_coefficient = 2. * x * sx / sr;
    let normalized_rhs = sp / sr;
    let backward_error = (normalized_coefficient * normalized_action - normalized_rhs).abs()
        / (normalized_coefficient.abs() * normalized_action.abs() + normalized_rhs.abs());
    assert!(backward_error <= step.prepared.solve.numerics().policy.linear_backward_error);
    // A meaningful parameter change tests the local prediction empirically in
    // the same output coordinates and frozen budget used for production.
    let increment = 0.1;
    let independent_change = (p + increment).sqrt() - p.sqrt();
    assert!(
        (derivative * increment - independent_change).abs() / target.coordinate_scale
            <= target.budget / target.coordinate_scale
    );
}

#[tokio::test]
async fn retained_root_action_screens_target_then_original_corrector_qualifies() {
    use crate::math::solves::Outcome;
    use pse_backend_native::solve::{Controls, Execution};
    use pse_model::strategy::BranchPolicy;
    let rows=pse_authoring::language::parse(
        "package p { def Root { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(2); } }",
        SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default(),
    ).unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let runtime = fixture::runtime_with(256 << 20, 16 << 20, 1 << 30);
    let physical = fixture::physical();
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    // This task explicitly admits fresh predictor proposals for its original corrector.
    solver.composition.recovery.extend([
        pse_model::strategy::StartOrigin::Predicted,
        pse_model::strategy::StartOrigin::Surrogate,
    ]);
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    analysis.case.variables.insert(
        "x".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(0.)),
            upper: None,
            fixed: None,
        },
    );
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let parameter = plain.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![parameter],
        reduced_hessian: false,
        propagation: None,
    });
    let mut prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    prepared.solve = prepared.solve.retaining_factor().unwrap();
    let base = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(base.completion.decision.permits_use());
    let predictor = base.root_predictor().unwrap();
    analysis.case.values.insert("p".into(), 4.4);
    let mut target = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    assert_eq!(
        target.solve.numerics().policy.engineering_relative_fraction,
        base.prepared
            .solve
            .numerics()
            .policy
            .engineering_relative_fraction
    );
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(10)),
    );
    // Portable scientific point evidence reconstructs the existing library factor;
    // no native factor bytes or independent permission decision are serialized.
    target.solve = runtime
        .native()
        .admit_proposal_task(target.solve.clone(), scope.clone())
        .unwrap();
    let portable = base.portable_prediction().unwrap().unwrap();
    let wire = serde_json::to_vec(&portable).unwrap();
    let portable: PortablePrediction = serde_json::from_slice(&wire).unwrap();
    let root = portable.root.as_ref().unwrap();
    let primal = portable
        .primal
        .iter()
        .copied()
        .map(f64::from_bits)
        .collect::<Vec<_>>();
    let original_values = root
        .values
        .iter()
        .copied()
        .map(f64::from_bits)
        .collect::<Vec<_>>();
    let rebuilt = runtime
        .native()
        .restore_root_predictor(
            base.prepared.solve.clone(),
            primal.clone(),
            original_values.clone(),
            root.key,
            scope.clone(),
            target.solve.task_admission(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(rebuilt.factor().key(), predictor.factor().key());
    assert_eq!(rebuilt.factor().point(), predictor.factor().point());
    let mut wrong_point = primal;
    wrong_point[0] += 0.1;
    assert!(
        runtime
            .native()
            .restore_root_predictor(
                base.prepared.solve.clone(),
                wrong_point,
                original_values,
                root.key,
                scope.clone(),
                target.solve.task_admission(),
                &cancel
            )
            .await
            .is_err()
    );
    let retained = PredictionSample {
        prepared: base.prepared.clone(),
        runtime: runtime.clone(),
        point: Arc::new(pse_columnar::Leased::new(
            Arc::new(portable),
            runtime
                .shared
                .math()
                .reserve("test:portable-root", wire.len())
                .unwrap(),
        )),
        root: Ok(rebuilt),
    };
    let execution = Execution::within(
        scope.cancellation().clone(),
        &Controls::default(),
        scope.clone(),
    )
    .unwrap();
    let mut portable_execution = Execution::within(
        scope.cancellation().clone(),
        &Controls::default(),
        scope.clone(),
    )
    .unwrap();
    portable_execution.work_admission = target
        .solve
        .task_admission()
        .map(|owner| -> Arc<dyn pse_backend_native::solve::WorkAdmission> { owner });
    let restored_proposal = retained
        .available_prediction(
            None,
            &target,
            BranchPolicy::any_qualified(),
            &portable_execution,
        )
        .unwrap();
    assert!((restored_proposal.values().next().unwrap().1 - 2.1).abs() <= 32. * f64::EPSILON * 2.1);
    let (proposal, work) = base
        .root_prediction(&target.solve, BranchPolicy::any_qualified(), &execution)
        .unwrap();
    assert!((proposal.values().next().unwrap().1 - 2.1).abs() <= 32. * f64::EPSILON * 2.1);
    assert_eq!(work.backsolves, 1);
    let screened = runtime
        .native()
        .screen_start(
            target.solve.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            scope,
            &cancel,
        )
        .await
        .unwrap();
    target.solve = target.solve.with_screened_start(&screened).unwrap();
    let corrected = package
        .solve_case(target, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(corrected.completion.decision.permits_use());
    let Outcome::Native(report) = &corrected.outcome else {
        panic!("native original corrector expected");
    };
    assert!(
        (report.candidate.as_ref().unwrap().primal[0] - 4.4_f64.sqrt()).abs()
            <= variable_budget(&corrected, "x"),
        "actual={:?}, original_values={:?}, termination={:?}, requested={:?}, quality={:?}",
        report.candidate.as_ref().unwrap().primal,
        report.observation.as_ref().map(|observed| &observed.values),
        report.termination,
        corrected.prepared.solve.accuracy(),
        report.quality
    );
    // Retained factor and source point survive independent native-session teardown.
    assert_eq!(predictor.factor().point(), [2.]);
    let mut changed = analysis.clone();
    changed.case.variables.get_mut("x").unwrap().lower = Some(Some(1.));
    let different = package.prepare_analysis(&changed, &cancel).await.unwrap();
    assert!(
        base.root_prediction(&different.solve, BranchPolicy::any_qualified(), &execution)
            .is_err()
    );
    // Two independently accepted original points produce a bounded secant proposal.
    analysis.case.values.insert("p".into(), 4.8);
    let mut secant_target = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let proposal = corrected
        .secant_prediction(
            &base,
            &secant_target,
            BranchPolicy::any_qualified(),
            &execution,
        )
        .unwrap();
    let predicted = 2. * corrected.values.scalars[&original_variable(&corrected.prepared, "x")]
        - base.values.scalars[&original_variable(&base.prepared, "x")];
    assert!(
        (proposal.values().next().unwrap().1 - predicted).abs()
            < 32. * f64::EPSILON * predicted.abs().max(1.)
    );
    let scope = execution.scope().unwrap();
    let screened = runtime
        .native()
        .screen_start(
            secant_target.solve.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            scope,
            &cancel,
        )
        .await
        .unwrap();
    secant_target.solve = secant_target.solve.with_screened_start(&screened).unwrap();
    let secant_corrected = package
        .solve_case(secant_target, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(secant_corrected.completion.decision.permits_use());
    let Outcome::Native(report) = &secant_corrected.outcome else {
        panic!("original correction expected");
    };
    assert!(
        (report.candidate.as_ref().unwrap().primal[0] - 4.8_f64.sqrt()).abs()
            <= variable_budget(&secant_corrected, "x")
    );
    analysis.case.values.insert("p".into(), 4.4);
    // A declared approximate fidelity follows the same original screening/correction path.
    use pse_math::surrogate::{FidelityCorrespondence, FidelityEvaluator, SurrogateOptions};
    #[derive(Debug)]
    struct ApproximateRootMerit(FidelityCorrespondence);
    impl FidelityEvaluator for ApproximateRootMerit {
        type Error = pse_backend_native::ProblemError;
        fn correspondence(&self) -> &FidelityCorrespondence {
            &self.0
        }
        fn evaluate(&mut self, x: &[f64], values: &mut [f64]) -> Result<(), Self::Error> {
            values[0] = (x[0] - 2.02).powi(2);
            Ok(())
        }
    }
    let mut target = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let identity = target.solve.original_identity().unwrap();
    assert_eq!(
        target.solve.numerics().policy.engineering_relative_fraction,
        base.prepared
            .solve
            .numerics()
            .policy
            .engineering_relative_fraction
    );
    let correspondence = FidelityCorrespondence {
        original_target: identity,
        model_target: identity,
        source: pse_ids::ContentHash::from_bytes([17; 32]),
        fidelity: pse_ids::ContentHash::from_bytes([18; 32]),
        coordinates: vec![original_variable(&target, "x")],
        outputs: vec![SemanticId::NIL],
    };
    let options = SurrogateOptions {
        bounds: vec![(1., 3.)],
        initial: vec![vec![1.], vec![2.02], vec![3.]],
        iterations: 2,
        evaluations: 20,
        infill_starts: 1,
        regression: pse_math::surrogate::RegressionSpec::CONSTANT,
        correlation: pse_math::surrogate::CorrelationSpec::MATERN52,
        theta: pse_math::surrogate::ThetaTuning::Fixed(vec![0.1].into()),
        gp_starts: 0,
        gp_evaluations: 5,
        seed: 74,
        phase_steps: (1, 1),
        radius: 0.2,
        contraction: 0.5,
        threads: 1,
        stack_bytes: runtime.shared.budget().math.stack_bytes,
        foreign_bytes: 1 << 20,
        time: std::time::Duration::from_secs(10),
    };
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(10)),
    );
    let handle = runtime
        .native()
        .prepare_surrogate(
            ApproximateRootMerit(correspondence.clone()),
            options,
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    let statistical = runtime
        .native()
        .advance_surrogate(&handle, &cancel)
        .await
        .unwrap()
        .unwrap();
    let proposal = target
        .solve
        .surrogate_start(
            &statistical,
            &correspondence,
            handle.key(),
            BranchPolicy::any_qualified(),
        )
        .unwrap();
    assert!(proposal.source().accuracy.is_none());
    let screened = runtime
        .native()
        .screen_start(
            target.solve.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            scope,
            &cancel,
        )
        .await
        .unwrap();
    target.solve = target.solve.with_screened_start(&screened).unwrap();
    drop(handle);
    let corrected = package
        .solve_case(target, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(corrected.completion.decision.permits_use());
    let Outcome::Native(report) = &corrected.outcome else {
        panic!("native original corrector expected");
    };
    assert!(
        (report.candidate.as_ref().unwrap().primal[0] - 4.4_f64.sqrt()).abs()
            <= variable_budget(&corrected, "x"),
        "actual={:?}, original_values={:?}, termination={:?}, requested={:?}, quality={:?}",
        report.candidate.as_ref().unwrap().primal,
        report.observation.as_ref().map(|observed| &observed.values),
        report.termination,
        corrected.prepared.solve.accuracy(),
        report.quality
    );
}

#[tokio::test]
async fn derived_preparation_corrects_original_with_declared_recovery_and_truthful_trace() {
    use crate::math::solves::{DerivedRequest, Outcome};
    use pse_backend_native::solve::StartPolicy;
    use pse_model::generated::enums::{
        CandidateUse, NumericalAttemptObservation, NumericalEventKind,
    };
    use pse_model::strategy::{
        MechanismKind, NumericalStrategy, Phase, Position, ProfileRef, StartOrigin, Transition,
        WorkLimits,
    };
    let rows = pse_authoring::language::parse(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
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
    let runtime = fixture::runtime_with(128 << 20, 1 << 20, 512 << 20);
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver: solver.clone(),
        numerical: NumericalInputs::default(),
    };
    let cancel = crate::CancelSource::new();
    let mut prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(20)),
    );
    let auxiliary = runtime
        .native()
        .prepare_derived(
            prepared.solve.clone(),
            DerivedRequest::AnchoredHomotopy {
                anchor: vec![0.],
                parameter: 0.5,
            },
            solver,
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    let family = auxiliary.family().key();
    let limits = WorkLimits {
        attempts: 2,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let mut declaration = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits);
    declaration.start.recovery.push(StartOrigin::Auxiliary);
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Kinsol,
        key: prepared.solve.strategy_profile().unwrap(),
    });
    declaration.mechanisms[0]
        .starts
        .push(StartOrigin::Auxiliary);
    let mut preparation = declaration.mechanisms[0].clone();
    preparation.kind = MechanismKind::Homotopy;
    preparation.position = Position::Preparation;
    preparation.profile = Some(auxiliary.profile_ref().unwrap());
    preparation.support = vec![family];
    preparation.transitions.push(Transition::Continue);
    declaration.mechanisms.insert(0, preparation);
    prepared.solve = prepared
        .solve
        .clone()
        .with_strategy(
            declaration,
            vec![auxiliary.into(), prepared.solve.clone().into()],
        )
        .unwrap();
    let result = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(
        result.completion.decision.permits_use(),
        "completion={:?}, outcome={:?}, events={:?}",
        result.completion,
        result.outcome,
        result.strategy.as_ref().map(|trace| &trace.events)
    );
    let Outcome::Native(report) = &result.outcome else {
        panic!("original corrector missing");
    };
    let x_id = result.prepared.model.case.compiled().plan.columns()[0];
    assert_eq!(
        result.prepared.model.model.compiled().model.symbols[&x_id]
            .lineage
            .path,
        "Root.x"
    );
    let coordinate_budget = fixture::engineering_target(
        result.prepared.solve.numerics(),
        NumericalTarget::Variable,
        x_id,
    )
    .budget;
    assert!((report.candidate.as_ref().unwrap().primal[0] - 3.).abs() <= coordinate_budget);
    assert!(
        (match &report
            .start_receipt
            .as_ref()
            .unwrap()
            .seed
            .as_ref()
            .unwrap()
            .payload
        {
            pse_backend_native::solve::WarmPayload::Root(x) => x[0],
            _ => panic!("root start required"),
        } - 1.5)
            .abs()
            <= coordinate_budget
    );
    let rows = result
        .strategy
        .as_ref()
        .unwrap()
        .rows(result.run_id, 0)
        .unwrap();
    let finished = rows
        .iter()
        .filter(|row| row.kind == NumericalEventKind::Finished && row.observation.is_some())
        .collect::<Vec<_>>();
    assert_eq!(finished.len(), 2);
    let charges = rows
        .iter()
        .filter(|row| {
            row.kind == NumericalEventKind::Finished
                && row.phase == Phase::Assessment
                && row.observation.is_none()
        })
        .collect::<Vec<_>>();
    assert_eq!(charges.len(), 1);
    // This case has no authored physical checks. Fresh original mathematical
    // evaluation belongs to the native corrector; assessment does not charge it twice.
    assert_eq!(charges[0].attempts, Some(0));
    assert_eq!(charges[0].evaluations, Some(0));
    assert!(charges[0].charging_owner.is_some());
    assert!(charges[0].decision_identity.is_some());
    assert_ne!(charges[0].charging_owner, finished[1].charging_owner);
    assert!(
        report
            .evidence
            .work
            .evaluations
            .is_some_and(|count| count > 0)
    );
    assert!(report.quality.as_ref().unwrap().feasible());
    assert_eq!(
        finished[0].observation,
        Some(NumericalAttemptObservation::Auxiliary)
    );
    assert_eq!(finished[0].permission, Some(CandidateUse::SeedOnly));
    assert_eq!(finished[0].derived_identity, Some(family));
    assert!(finished[0].start_identity.is_some());
    assert_eq!(finished[1].start_origin, Some(StartOrigin::Auxiliary));
    assert_eq!(finished[1].transition, Some(Transition::Finish));
    assert!(
        !scope
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
}

async fn solve_with_backend(
    boundary: bool,
    backend: Backend,
) -> TestResult<Arc<crate::workflow::RunResult>> {
    let text = "package p { def Root { param p: Scalar=4; var x: Scalar; eq root: x*x==p; annotation start x(2); } }";
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )?;
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .ok_or_else(|| std::io::Error::other("Root declaration absent"))?
        .declaration_id;
    let package = fixture::runtime_with(16 << 20, 16 << 20, 1 << 30)
        .modeling_package(rows, fixture::physical())
        .await?;
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(backend);
    let order = DerivativeOrder::First;
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    analysis.case.variables.insert(
        "x".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(if boundary { 2. } else { 0. })),
            upper: Some(if backend == Backend::Scip {
                Some(10.)
            } else {
                None
            }),
            fixed: None,
        },
    );
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&analysis, &cancel).await?;
    let p = plain.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![p],
        reduced_hessian: false,
        propagation: None,
    });
    Ok(package
        .prepare_analysis(&analysis, &cancel)
        .await?
        .start()?
        .wait()
        .await?)
}
#[tokio::test]
async fn root_response_publication_has_physical_primal_and_no_kkt_fields() {
    let result = solve_with_backend(false, Backend::Kinsol).await.unwrap();
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    let v = &validity[0].validity;
    assert!(v.certified, "{v:?}");
    assert_eq!(v.root_rank, Some(1));
    assert!(v.root_backward_error.is_some());
    assert!(v.root_neighborhood.is_some());
    assert!(v.licq.is_none() && v.strict_complementarity.is_none() && v.second_order.is_none());
    let response = parametric_sensitivities::Row::rows(
        &result.table("runtime.parametric_sensitivities").unwrap(),
    )
    .unwrap();
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].target_kind, NumericalTarget::Variable);
    assert!(response[0].dual.is_none());
    let crate::workflow::RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!("modeling report expected");
    };
    assert_published_root_response(&steps[0], &response[0]);
}
#[tokio::test]
async fn root_response_withheld_at_active_bound_keeps_base_solution() {
    let result = solve_with_backend(true, Backend::Kinsol).await.unwrap();
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert!(!validity[0].validity.certified);
    assert_eq!(
        validity[0].validity.reason,
        Some(pse_relations::generated::enums::WithheldReason::NeighborhoodUnavailable)
    );
    assert!(
        parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap()
        )
        .unwrap()
        .is_empty()
    );
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert!(!runs.is_empty());
    assert!(
        runs.iter()
            .any(|r| r.qualification
                == pse_relations::generated::enums::NativeQualification::Feasible)
    );
}

#[tokio::test]
async fn root_response_rank_loss_withholds_sensitivity_and_keeps_feasible_base() {
    let rows = pse_authoring::language::parse(
        "package p { def Root { param p:Scalar=0; var x:Scalar; eq root:x*x*x==p; annotation start x(0); } }",
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
    let package = fixture::runtime_with(16 << 20, 16 << 20, 1 << 30)
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![parameter],
        reduced_hessian: false,
        propagation: None,
    });
    let result = package
        .prepare_analysis(&analysis, &cancel)
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    // The native initial point satisfies x³=p exactly; its actual original Jacobian
    // is zero, so a finite square response cannot be established there.
    assert!(result.usable());
    let crate::workflow::RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!("modeling report expected");
    };
    let crate::math::solves::Outcome::Native(native) = &steps[0].outcome else {
        panic!("native root report expected");
    };
    assert_eq!(native.backend, Backend::Kinsol);
    assert_eq!(native.candidate.as_ref().unwrap().primal, [0.]);
    assert!(native.quality.as_ref().unwrap().feasible());
    assert!(matches!(
        native.evidence.root_response.as_ref(),
        Some(Err(pse_backend_native::square_response::Withheld::Rank {
            rank: 0,
            dimension: 1,
            ..
        }))
    ));
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    let [validity] = validity.as_slice() else {
        panic!("one validity record expected")
    };
    assert!(!validity.validity.certified);
    assert_eq!(validity.validity.root_rank, Some(0));
    assert_eq!(
        validity.validity.reason,
        Some(pse_relations::generated::enums::WithheldReason::RankDeficient)
    );
    assert!(validity.validity.licq.is_none());
    assert!(validity.validity.second_order.is_none());
    assert!(
        parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap()
        )
        .unwrap()
        .is_empty()
    );
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert!(runs.iter().any(|r| r.feasible == Some(true)
        && r.qualification == pse_relations::generated::enums::NativeQualification::Feasible
        && r.objective.is_none()));
}

#[cfg(any(
    feature = "solver-ipopt",
    feature = "solver-pounce",
    feature = "solver-scip"
))]
async fn assert_root_response(backend: Backend, boundary: bool) -> TestResult<()> {
    let result = solve_with_backend(boundary, backend).await?;
    let validity = local_validity::Row::rows(&result.table("runtime.local_validity")?)?;
    assert_eq!(validity.len(), 1);
    let v = &validity[0].validity;
    let response =
        parametric_sensitivities::Row::rows(&result.table("runtime.parametric_sensitivities")?)?;
    assert!(v.licq.is_none() && v.strict_complementarity.is_none() && v.second_order.is_none());
    if boundary {
        assert!(!v.certified, "{backend:?}: {v:?}");
        assert_eq!(
            v.reason,
            Some(pse_relations::generated::enums::WithheldReason::NeighborhoodUnavailable),
            "{backend:?}: {v:?}"
        );
        assert!(response.is_empty());
    } else {
        assert!(v.certified, "{backend:?}: {v:?}");
        assert_eq!(v.root_rank, Some(1));
        assert_eq!(response.len(), 1);
        assert!(response[0].dual.is_none());
        let crate::workflow::RunReport::Modeling(steps) = result
            .report()
            .map_err(|error| std::io::Error::other(error.to_string()))?
        else {
            return Err(std::io::Error::other("modeling report expected").into());
        };
        assert_published_root_response(&steps[0], &response[0]);
    }
    let expected = if matches!(backend, Backend::Ipopt | Backend::Pounce) {
        pse_relations::generated::enums::NativeQualification::Stationary
    } else {
        pse_relations::generated::enums::NativeQualification::Feasible
    };
    let crate::workflow::RunReport::Modeling(steps) = result
        .report()
        .map_err(|error| std::io::Error::other(error.to_string()))?
    else {
        return Err(std::io::Error::other("modeling report expected").into());
    };
    let step = steps
        .first()
        .ok_or_else(|| std::io::Error::other("root solve step expected"))?;
    let crate::math::solves::Outcome::Native(native) = &step.outcome else {
        return Err(std::io::Error::other("native root report expected").into());
    };
    assert!(
        native
            .quality
            .as_ref()
            .is_some_and(|quality| quality.feasible())
    );
    if matches!(backend, Backend::Ipopt | Backend::Pounce) {
        let kkt = native
            .evidence
            .kkt
            .as_ref()
            .ok_or_else(|| std::io::Error::other("stationary solver KKT evidence expected"))?;
        assert_eq!(kkt.stationarity, Some(true));
        assert_eq!(kkt.complementarity, Some(true));
        assert_eq!(
            native.termination.assurance,
            pse_backend_native::solve::Assurance::LocalStationary
        );
    }
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs")?)?;
    assert!(
        runs.iter().any(|r| r.backend == Some(backend)
            && r.feasible == Some(true)
            && r.qualification == expected),
        "{backend:?}: {runs:?}"
    );
    Ok(())
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Ipopt, false).await.unwrap();
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Ipopt, true).await.unwrap();
}
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn explicit_pounce_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Pounce, false).await.unwrap();
}
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn explicit_pounce_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Pounce, true).await.unwrap();
}

#[cfg(feature = "solver-scip")]
#[tokio::test]
async fn explicit_scip_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Scip, false).await.unwrap();
}
#[cfg(feature = "solver-scip")]
#[tokio::test]
async fn explicit_scip_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Scip, true).await.unwrap();
}

#[cfg(all(feature = "solver-pounce", feature = "solver-ipopt"))]
#[tokio::test]
async fn demanded_qp_prediction_screens_and_original_corrector_qualifies() {
    use crate::math::solves::Outcome;
    use pse_backend_native::kkt::path::qp;
    use pse_backend_native::solve::{Controls, Execution};
    use pse_model::strategy::BranchPolicy;
    let rows = pse_authoring::language::parse("package p { def Qp { param p:Scalar=1; var x:Scalar; eq floor:x>=0; let cost:Scalar=0.5*x*x-p*x; annotation objective cost(minimize); annotation start x(1); } }",
        SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap();
    let root = rows.iter().find(|r| r.name == "Qp").unwrap().declaration_id;
    let runtime = fixture::runtime_with(256 << 20, 16 << 20, 1 << 30);
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Optimize;
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::Second,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    let prepared = package
        .prepare_analysis(&analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    let base = package
        .solve_case(prepared, analysis.compiler, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(base.completion.decision.permits_use());
    let Outcome::Native(report) = &base.outcome else {
        panic!("original native result expected");
    };
    let candidate = report.candidate.as_ref().unwrap();
    let mut h = qp::SymTMatrix::new(qp::SymTMatrixSpace::new(1, vec![1], vec![1]));
    h.set_values(&[1.]);
    let mut a = qp::GenTMatrix::new(qp::GenTMatrixSpace::new(1, 1, vec![1], vec![1]));
    a.set_values(&[1.]);
    let previous = qp::QpProblem {
        n: 1,
        m: 1,
        h: &h,
        g: &[-1.],
        a: &a,
        bl: &[0.],
        bu: &[1e20],
        xl: &[-1e20],
        xu: &[1e20],
        hessian_inertia: qp::HessianInertia::Psd,
    };
    let target_qp = qp::QpProblem {
        g: &[-2.],
        ..previous
    };
    let source = qp::QpSolution {
        x: candidate.primal.clone(),
        lambda_g: vec![0.],
        lambda_x: vec![0.],
        working: qp::WorkingSet::cold(1, 1),
        obj: 0.5 * candidate.primal[0].powi(2) - candidate.primal[0],
        status: qp::QpStatus::Optimal,
        stats: Default::default(),
        unbounded_ray: None,
    };
    let linear = qp::FeralConfig {
        parallel: Some(false),
        fma: false,
        ..Default::default()
    };
    let x_id = base.prepared.model.case.compiled().plan.columns()[0];
    assert_eq!(
        base.prepared.model.model.compiled().model.symbols[&x_id]
            .lineage
            .path,
        "Qp.x"
    );
    assert_eq!(
        fixture::engineering_target(
            base.prepared.solve.numerics(),
            NumericalTarget::Variable,
            x_id
        )
        .coordinate_scale,
        1.
    );
    assert_eq!(
        base.prepared
            .solve
            .numerics()
            .targets
            .iter()
            .find(|row| row.kind == NumericalTarget::Row)
            .unwrap()
            .coordinate_scale,
        1.
    );
    assert_eq!(
        fixture::engineering_target(
            base.prepared.solve.numerics(),
            NumericalTarget::Objective,
            SemanticId::NIL
        )
        .coordinate_scale,
        1.
    );
    let options = qp::QpOptions {
        max_iter: 50,
        feas_tol: base.prepared.solve.accuracy().feasibility,
        opt_tol: base.prepared.solve.accuracy().stationarity,
        ..Default::default()
    };
    analysis.case.values.insert("p".into(), 2.);
    analysis
        .solver
        .composition
        .recovery
        .push(pse_model::strategy::StartOrigin::Predicted);
    let cancel = crate::CancelSource::new();
    let mut target = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let mut execution = Execution::new(Arc::default(), &Controls::default());
    let request = qp::Request {
        previous: &previous,
        source: &source,
        target: &target_qp,
        options: &options,
        linear: &linear,
        limits: pse_backend_native::kkt::activity::Limits {
            backsolves: 1000,
            refactorizations: 200,
            bytes: 1 << 20,
        },
    };
    let limits = pse_model::strategy::WorkLimits {
        attempts: 1,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let uncapped = crate::math::strategy::admission::TaskAdmission::new(
        limits,
        execution.scope().unwrap(),
        None,
        false,
    );
    execution.work_admission = Some(uncapped.clone());
    let capped = crate::math::strategy::admission::TaskAdmission::new(
        pse_model::strategy::WorkLimits {
            proof_steps: Some(0),
            ..limits
        },
        execution.scope().unwrap(),
        None,
        false,
    );
    let mut capped_execution = execution.clone();
    capped_execution.work_admission = Some(capped.clone());
    let refusal = base
        .qp_prediction(
            &target.solve,
            request,
            BranchPolicy::any_qualified(),
            &capped_execution,
        )
        .unwrap_err();
    assert!(
        refusal.to_string().contains("known operation count"),
        "{refusal}"
    );
    let no_work = capped.observation().unwrap();
    assert_eq!(no_work.factorizations, Some(0));
    assert_eq!(no_work.proof_steps, Some(0));
    // The unchanged library family alone cannot establish original-model provenance.
    let mut wrong_h = qp::SymTMatrix::new(qp::SymTMatrixSpace::new(1, vec![1], vec![1]));
    wrong_h.set_values(&[2.]);
    let wrong_previous = qp::QpProblem {
        h: &wrong_h,
        ..previous
    };
    let wrong_target = qp::QpProblem {
        h: &wrong_h,
        ..target_qp
    };
    let refusal = base
        .qp_prediction(
            &target.solve,
            qp::Request {
                previous: &wrong_previous,
                target: &wrong_target,
                ..request
            },
            BranchPolicy::any_qualified(),
            &execution,
        )
        .unwrap_err();
    assert!(
        refusal.to_string().contains("compiler's actual"),
        "{refusal}"
    );
    let mut wrong_a = qp::GenTMatrix::new(qp::GenTMatrixSpace::new(1, 1, vec![1], vec![1]));
    wrong_a.set_values(&[2.]);
    let wrong_previous = qp::QpProblem {
        a: &wrong_a,
        ..previous
    };
    let wrong_target = qp::QpProblem {
        a: &wrong_a,
        ..target_qp
    };
    assert!(
        base.qp_prediction(
            &target.solve,
            qp::Request {
                previous: &wrong_previous,
                target: &wrong_target,
                ..request
            },
            BranchPolicy::any_qualified(),
            &execution
        )
        .unwrap_err()
        .to_string()
        .contains("compiler's actual")
    );
    let wrong_target = qp::QpProblem {
        g: &[-3.],
        ..target_qp
    };
    assert!(
        base.qp_prediction(
            &target.solve,
            qp::Request {
                target: &wrong_target,
                ..request
            },
            BranchPolicy::any_qualified(),
            &execution
        )
        .unwrap_err()
        .to_string()
        .contains("compiler's actual")
    );
    let wrong_previous = qp::QpProblem {
        g: &[-2.],
        ..previous
    };
    assert!(
        base.qp_prediction(
            &target.solve,
            qp::Request {
                previous: &wrong_previous,
                ..request
            },
            BranchPolicy::any_qualified(),
            &execution
        )
        .unwrap_err()
        .to_string()
        .contains("compiler's actual")
    );
    let mut wrong_source = source.clone();
    wrong_source.x[0] += 1.;
    assert!(
        base.qp_prediction(
            &target.solve,
            qp::Request {
                source: &wrong_source,
                ..request
            },
            BranchPolicy::any_qualified(),
            &execution
        )
        .unwrap_err()
        .to_string()
        .contains("actual original candidate")
    );
    let unrelated_rows = pse_authoring::language::parse("package other { def Qp { param p:Scalar=2; var x:Scalar; eq floor:x>=0; let cost:Scalar=0.5*x*x-p*x; annotation objective cost(minimize); annotation start x(1); } }",
        SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap();
    let unrelated_root = unrelated_rows
        .iter()
        .find(|r| r.name == "Qp")
        .unwrap()
        .declaration_id;
    let unrelated_package = runtime
        .modeling_package(unrelated_rows, fixture::physical())
        .await
        .unwrap();
    let mut unrelated_analysis = analysis.clone();
    unrelated_analysis.root = unrelated_root;
    unrelated_analysis.instance = pse_modeling::specialize::root_instance(unrelated_root);
    let unrelated = unrelated_package
        .prepare_analysis(&unrelated_analysis, &cancel)
        .await
        .unwrap();
    assert!(
        base.qp_prediction(
            &unrelated.solve,
            request,
            BranchPolicy::any_qualified(),
            &execution
        )
        .unwrap_err()
        .to_string()
        .contains("same named original coordinates")
    );
    let (proposal, outcome) = base
        .qp_prediction(
            &target.solve,
            request,
            BranchPolicy::any_qualified(),
            &execution,
        )
        .unwrap();
    assert_eq!(outcome.solution.status, qp::QpStatus::Optimal);
    assert!(outcome.linear_calls > 0);
    assert!(outcome.solution.stats.parametric_source.is_some());
    assert_eq!(
        uncapped.observation().unwrap().proof_steps,
        None,
        "actual class production cannot be relabeled as zero proof work"
    );
    assert!(
        (proposal.values().next().unwrap().1 - 2.).abs()
            <= variable_budget_from_prepared(&target, "x")
    );
    assert_eq!(
        proposal.origin(),
        pse_model::strategy::StartOrigin::Predicted
    );
    let screened = runtime
        .native()
        .screen_start(
            target.solve.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            execution.scope().unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    target.solve = target.solve.with_screened_start(&screened).unwrap();
    let corrected = package
        .solve_case(target, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(corrected.completion.decision.permits_use());
    let Outcome::Native(report) = &corrected.outcome else {
        panic!("original corrector expected");
    };
    let candidate = report.candidate.as_ref().unwrap();
    let actual_cost = 0.5 * candidate.primal[0].powi(2) - 2. * candidate.primal[0];
    assert!(
        (candidate.objective.unwrap() - actual_cost).abs()
            <= 32. * f64::EPSILON * actual_cost.abs().max(1.)
    );
    let objective = fixture::engineering_target(
        corrected.prepared.solve.numerics(),
        NumericalTarget::Objective,
        SemanticId::NIL,
    );
    // Independent objective comparison at the returned production candidate; no
    // forward-coordinate guarantee is inferred from its KKT termination.
    assert!((actual_cost + 2.).abs() <= objective.budget);
}
