// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[cfg(feature = "solver-root-isolation")]
fn hash(n: u8) -> ContentHash {
    ContentHash::from_bytes([n; 32])
}
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn policy() -> PathPolicy {
    PathPolicy {
        steps: 4,
        subdivisions: 3,
        step: 0.2,
        minimum_step: 0.025,
        tangent: arclength::Limits {
            actions: 2,
            bytes: 1 << 20,
        },
        backward_limit: 1e-10,
        parameter_tolerance: 1e-8,
        hyperplane_tolerance: 1e-8,
        events: None,
    }
}
#[test]
fn declared_path_geometry_and_terminal_causes_never_create_retry_permission() {
    let mut p = policy();
    p.validate().unwrap();
    p.step = f64::INFINITY;
    assert!(p.validate().is_err());
    for cause in [
        ProblemError::Contract("source".into()),
        ProblemError::Unsupported("required sheet".into()),
        ProblemError::memory("storage"),
        ProblemError::internal("foreign exception"),
        ProblemError::Cancelled,
        ProblemError::stopped(Termination::TimeLimit, "original deadline"),
    ] {
        assert!(!retry_cause(&cause));
    }
    assert!(retry_cause(&ProblemError::numerical(
        "actual factor rank loss"
    )));
    let nested = ProblemError::Math(pse_math::MathError::Instance {
        instance: id(9),
        checked_members: Default::default(),
        cause: Box::new(pse_math::MathError::Contract("nested contract".into())),
    });
    assert!(!retry_cause(&nested));
}
#[cfg(feature = "solver-root-isolation")]
fn proof_payload() -> PathSelection {
    use pse_math::factorable::{Constant, Node, RootIsolationProgram};
    // Actual fixture graph p-x*x=0, unknown state first and parameter last.
    let program = Arc::new(RootIsolationProgram {
        inputs: 2,
        nodes: vec![
            Node::Var(1),
            Node::Var(0),
            Node::Product(vec![1, 1]),
            Node::Const(Constant::Float(-1.)),
            Node::Product(vec![2, 3]),
            Node::Sum(vec![0, 4]),
            Node::Const(Constant::Float(0.)),
            Node::Const(Constant::Float(1e-12)),
        ],
        residuals: vec![5],
        eligibility: vec![],
        criterion: [6, 7],
        obligations: vec![],
        derivative_obligations: vec![],
    });
    let verifier: Arc<dyn SelectionVerifier> = Arc::new(native::root_isolation::Ibex);
    let alternative = SelectionScope {
        id: id(1),
        program,
        residual_identity: hash(3),
        unknowns: vec![pse_math::implicit::Unknown {
            id: id(2),
            lower: 0.1,
            upper: 2.,
        }],
    };
    let alternatives = [SelectionAlternative {
        id: alternative.id,
        program: &alternative.program,
        residual_identity: alternative.residual_identity,
        unknowns: &alternative.unknowns,
    }];
    let cancel = Arc::default();
    let request = SelectionProofRequest {
        selection: id(4),
        alternatives: &alternatives,
        winner: 0,
        parameters: &[1.],
        candidate: &[1.],
        order: DerivativeOrder::First,
        time_limit: std::time::Duration::from_secs(10),
        cancel: &cancel,
    };
    let SelectionEvidence::Unique(previous) = verifier.certify(&request).unwrap() else {
        panic!("actual positive chart required")
    };
    PathSelection(Arc::new(SelectionData {
        source: hash(5),
        parameter: id(6),
        coordinates: vec![id(2)],
        verifier,
        previous,
        alternatives: vec![alternative],
        winner: 0,
        coverage: ChartChainCoverage::RootSheet,
        _owner: None,
    }))
}
#[cfg(feature = "solver-root-isolation")]
#[test]
fn actual_ibex_sheet_transport_consumes_original_endpoints_and_coverage() {
    let selection = proof_payload();
    let execution = Execution::new(Arc::default(), &Controls::default());
    let (chart, evidence) = selection
        .connect(
            &selection.previous,
            1.,
            &[0.99_f64.sqrt(), 0.99],
            &execution,
        )
        .unwrap();
    assert!(chart.is_some(), "{evidence:?}");
    assert!(
        matches!(evidence,ChartChainEvidence::Connected(proof) if proof.coverage==ChartChainCoverage::RootSheet && proof.proof_cells>0)
    );
    let mut required = proof_payload();
    Arc::get_mut(&mut required.0).unwrap().coverage = ChartChainCoverage::SelectedFunction;
    let (chart, evidence) = required
        .connect(&required.previous, 1., &[0.99_f64.sqrt(), 0.99], &execution)
        .unwrap();
    assert!(chart.is_none());
    assert!(!matches!(evidence, ChartChainEvidence::Connected(_)));
    execution.cancel.store(true, Ordering::Release);
    assert!(matches!(
        selection.connect(&selection.previous, 1., &[1., 1.], &execution),
        Err(ProblemError::Cancelled)
    ));
}

#[cfg(feature = "solver-ipopt")]
async fn prepared(
    text: &str,
) -> (
    crate::workflow::Runtime,
    crate::workflow::ModelingPackage,
    crate::workflow::ModelingAnalysis,
    crate::workflow::ModelingSolvePreparation,
) {
    use crate::workflow::{ModelingAnalysis, tests as fixture};
    let rows = pse_authoring::language::parse(
        text,
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
    let runtime = fixture::runtime_with(6 << 30, 64 << 20, 16 << 30);
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let mut solver = SolverProfile {
        intent: SolveIntent::Root,
        ..Default::default()
    };
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    solver.controls.hessian = HessianMode::LimitedMemory;
    solver.controls.start = StartPolicy::PreviousAccepted;
    solver.presolve = native::presolve::Policy::Off;
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(),
        limits: Default::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let parameter = plain.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(crate::math::settings::SensitivityRequest {
        parameters: vec![parameter],
        reduced_hessian: false,
        propagation: None,
    });
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    (runtime, package, analysis, prepared)
}
#[cfg(feature = "solver-ipopt")]
fn request(original: &PreparedSolve, parameter: SemanticId) -> PathRequest {
    let mut corrector = original.profile.clone();
    corrector.sensitivity = None;
    corrector.controls.start = StartPolicy::Explicit;
    corrector.controls.reuse = ReusePolicy::Fresh;
    PathRequest {
        parameter,
        interval: (-2., 5.),
        scale: 1.,
        hyperplane: id(35),
        corrector,
        target: original.clone(),
        branch: BranchPolicy::any_qualified(),
        selection: None,
        curvature: None,
        policy: policy(),
    }
}
#[cfg(feature = "solver-ipopt")]
fn scope() -> ExecutionScope {
    ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(20)),
    )
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn prepared_first_source_lowers_actual_offset_and_partials_without_square_parameter_assumption()
 {
    let (runtime,_,_,prepared)=prepared("package p { def Root { param p:Scalar=3; var x:Scalar; annotation start x(2); eq root:x*x-p==1; } }").await;
    let original = prepared.solve;
    let Representation::Algebraic(case) = &original.representation else {
        panic!("algebraic")
    };
    let parameter = case
        .sensitivity
        .as_ref()
        .unwrap()
        .available()
        .unwrap()
        .parameters[0]
        .0;
    let path = runtime
        .native()
        .prepare_path(
            original.clone(),
            request(&original, parameter),
            scope(),
            &crate::CancelSource::new(),
        )
        .unwrap();
    assert_eq!(path.family.original().constraints()[0].lower, 0.);
    assert_eq!(
        path.case.prepared.compiled().plan.structure().rows()[0].lower,
        0.
    );
    let execution = Execution::within(
        path.scope.cancellation().clone(),
        &path.request.corrector.controls,
        path.scope.clone(),
    )
    .unwrap();
    let budget = WorkerBudget::new(path.workspace_bytes);
    let mut supplier = path
        .supplier(runtime.native(), &execution, &budget)
        .unwrap();
    let mut output = [100.];
    supplier.values(&[2.], 3., &mut output).unwrap();
    assert_eq!(output, [0.]);
    supplier
        .state_action(&[2.], 3., &[2.], &mut output)
        .unwrap();
    assert_eq!(output, [8.]);
    supplier
        .parameter_action(&[2.], 3., 2., &mut output)
        .unwrap();
    assert_eq!(output, [-2.]);
    let mut wrong = request(&original, parameter);
    wrong.branch = BranchPolicy {
        kind: pse_model::strategy::BranchKind::Connected,
        connected: Some(pse_model::strategy::ConnectedPath {
            path: ContentHash::from_bytes([1; 32]),
            sheet: ContentHash::from_bytes([2; 32]),
            transport: ContentHash::from_bytes([3; 32]),
            orientation: ContentHash::from_bytes([4; 32]),
        }),
    };
    assert!(
        runtime
            .native()
            .prepare_path(
                original.clone(),
                wrong,
                scope(),
                &crate::CancelSource::new()
            )
            .is_err()
    );
    #[cfg(feature = "solver-root-isolation")]
    {
        let mut wrong = request(&original, parameter);
        let mut selection = proof_payload();
        let data = Arc::get_mut(&mut selection.0).unwrap();
        data.coordinates = case.prepared.compiled().plan.columns().to_vec();
        data.parameter = parameter;
        data.source = hash(99);
        wrong.selection = Some(selection);
        assert!(
            runtime
                .native()
                .prepare_path(original, wrong, scope(), &crate::CancelSource::new())
                .is_err()
        );
    }
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn actual_native_path_passes_fold_and_only_returns_original_physical_start() {
    let (runtime,package,analysis,prepared)=prepared("package p { def Root { param p:Scalar=-0.9375; var x:Scalar; annotation start x(0.25); eq root:x*x-p==1; } }").await;
    let original = prepared.solve.clone();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    let cancel = crate::CancelSource::new();
    let base = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    let start = base.path_start(vec![-1., 0.]).unwrap();
    let path = runtime
        .native()
        .prepare_path(
            original.clone(),
            request(&original, parameter),
            scope(),
            &cancel,
        )
        .unwrap();
    let outcome = runtime
        .native()
        .path_task(path, start, &cancel)
        .await
        .unwrap();
    assert!(outcome.terminal.is_none(), "{:?}", outcome.terminal);
    assert_eq!(outcome.observations.len(), 4);
    let last = outcome.observations.last().unwrap();
    assert!(last.point[0] < 0., "{last:?}");
    assert!((last.point[0] * last.point[0] - last.point[1] - 1.).abs() < 1e-7);
    assert!(outcome.observations.first().unwrap().tangent.physical[1] < 0.);
    assert!(last.tangent.physical[1] > 0.);
    let proposal = outcome.proposal.unwrap();
    assert_eq!(proposal.target(), original.original_identity().unwrap());
    assert_eq!(proposal.values().count(), 1);
    assert!(proposal.source().derivation.is_some());
    let screened = runtime
        .native()
        .screen_start(
            original.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            scope(),
            &cancel,
        )
        .await
        .unwrap();
    assert!(screened.point()[0] < 0.);
}

#[cfg(all(feature = "solver-ipopt", feature = "solver-root-isolation"))]
#[tokio::test]
async fn genuine_compiled_sheet_binds_prior_origin_then_certifies_actual_target_and_rejects_root_jump()
 {
    let (runtime,package,analysis,prepared)=prepared("package p { def Root { param p:Scalar=3; var x:Scalar; annotation bounds x(0.1,3); annotation start x(2); eq root:x*x-p==1; } }").await;
    let original = prepared.solve.clone();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    let cancel = crate::CancelSource::new();
    let base = package
        .solve_case(prepared.clone(), analysis.compiler, &cancel)
        .await
        .unwrap();
    let start = base.path_start(vec![0., 1.]).unwrap();
    let scope = scope();
    let selection = runtime
        .native()
        .prepare_path_selection(
            original.clone(),
            start.clone(),
            parameter,
            (2., 4.),
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(selection.source(), original.preparation_identity().unwrap());
    assert_eq!(selection.coverage(), ChartChainCoverage::RootSheet);
    let mut required = request(&original, parameter);
    required.interval = (2., 4.);
    required.policy.steps = 1;
    required.policy.step = 0.025;
    required.selection = Some(selection.clone());
    let path = runtime
        .native()
        .prepare_path(original.clone(), required.clone(), scope.clone(), &cancel)
        .unwrap()
        .require_connected(&start)
        .unwrap();
    let prior = path.branch.connected.unwrap();
    assert_eq!(prior.sheet, selection.sheet());
    let outcome = runtime
        .native()
        .path_task(path.clone(), start.clone(), &cancel)
        .await
        .unwrap();
    assert!(outcome.terminal.is_none(), "{:?}", outcome.terminal);
    assert_eq!(outcome.native_calls, 2);
    let witness = outcome.completion_witness().unwrap();
    let proposal = outcome.proposal.as_ref().unwrap();
    assert_eq!(
        witness.recovery_branch(path.branch(), proposal).unwrap(),
        proposal.branch()
    );
    assert!(
        witness
            .recovery_branch(BranchPolicy::any_qualified(), proposal)
            .is_err()
    );
    assert_eq!(outcome.observations.len(), 2);
    assert_eq!(outcome.observations.last().unwrap().point[1], 3.);
    assert_ne!(outcome.connected.unwrap().transport, prior.transport);
    assert_eq!(
        outcome.proposal.as_ref().unwrap().branch().connected,
        outcome.connected
    );
    let execution = Execution::within(
        scope.cancellation().clone(),
        &path.profile().controls,
        scope.clone(),
    )
    .unwrap();
    outcome
        .validate_original_candidate(&original, &[2.], &execution)
        .unwrap();
    assert!(
        outcome
            .validate_original_candidate(&original, &[-2.], &execution)
            .is_err()
    );
    use pse_model::strategy::{
        Mechanism, MechanismKind, NumericalStrategy, Position, ProfileRef, StartOrigin, Transition,
        WorkLimits,
    };
    let limits = WorkLimits {
        attempts: 10,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let mut strategy = NumericalStrategy::direct(StartPolicy::PreviousAccepted, limits);
    strategy.branch = path.branch();
    strategy.start.recovery = vec![StartOrigin::Auxiliary];
    strategy.mechanisms = vec![
        Mechanism {
            operation: Default::default(),
            kind: MechanismKind::Continuation,
            position: Position::Preparation,
            required: true,
            profile: Some(ProfileRef {
                backend: Backend::Ipopt,
                key: profile_key(path.profile()).unwrap().as_id(),
            }),
            support: path.support().into_iter().collect(),
            limits,
            starts: vec![StartOrigin::Accepted],
            transitions: vec![Transition::Continue, Transition::Stop],
        },
        Mechanism {
            operation: Default::default(),
            kind: MechanismKind::Direct,
            position: Position::Execution,
            required: true,
            profile: Some(ProfileRef {
                backend: Backend::Ipopt,
                key: original.strategy_profile().unwrap(),
            }),
            support: vec![],
            limits,
            starts: vec![StartOrigin::Specification, StartOrigin::Auxiliary],
            transitions: vec![Transition::Finish, Transition::Stop],
        },
    ];
    let mut composed = prepared;
    composed.solve = original
        .clone()
        .within_task(scope.clone())
        .unwrap()
        .with_strategy(
            strategy,
            vec![
                PreparedRung::Path {
                    prepared: path.clone(),
                    start: Box::new(start.clone()),
                },
                original.clone().into(),
            ],
        )
        .unwrap();
    let final_original = package
        .solve_case(composed, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(final_original.accepted);
    assert!(final_original.validation_error.is_none());
    outcome
        .completion_witness()
        .unwrap()
        .validate_original_candidate(
            &original,
            final_original.path_start(vec![0., 1.]).unwrap().point(),
            &execution,
        )
        .unwrap();
    let mut changed = path.clone();
    changed.branch.connected.as_mut().unwrap().transport = hash(222);
    let refused = runtime
        .native()
        .path_task(changed, start, &cancel)
        .await
        .unwrap();
    assert_eq!(refused.native_calls, 0);
    assert!(matches!(
        refused.terminal.as_deref(),
        Some(ProblemError::Contract(_))
    ));
    let mut unsupported = required;
    let mut data =
        Arc::try_unwrap(unsupported.selection.take().unwrap().0).unwrap_or_else(|shared| {
            let selected = &*shared;
            SelectionData {
                source: selected.source,
                parameter: selected.parameter,
                coordinates: selected.coordinates.clone(),
                verifier: selected.verifier.clone(),
                previous: selected.previous.clone(),
                alternatives: selected.alternatives.clone(),
                winner: selected.winner,
                coverage: selected.coverage,
                _owner: None,
            }
        });
    data.coverage = ChartChainCoverage::SelectedFunction;
    unsupported.selection = Some(PathSelection(Arc::new(data)));
    let refused = runtime
        .native()
        .prepare_path(original, unsupported, scope, &cancel)
        .unwrap();
    let result = runtime
        .native()
        .path_task(refused, base.path_start(vec![0., 1.]).unwrap(), &cancel)
        .await
        .unwrap();
    assert!(result.proposal.is_none());
    assert!(result.terminal.is_some());
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn actual_path_rung_transfers_start_to_shared_original_correction_and_permission() {
    use pse_model::strategy::{
        Mechanism, MechanismKind, NumericalStrategy, Position, ProfileRef, StartOrigin, Transition,
        WorkLimits,
    };
    let (runtime,package,analysis,mut prepared)=prepared("package p { def Root { param p:Scalar=3; var x:Scalar; annotation start x(2); eq root:x*x-p==1; } }").await;
    let original = prepared.solve.clone();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    let cancel = crate::CancelSource::new();
    let base = package
        .solve_case(prepared.clone(), analysis.compiler, &cancel)
        .await
        .unwrap();
    let start = base.path_start(vec![0., 1.]).unwrap();
    let task_scope = scope();
    let path = runtime
        .native()
        .prepare_path(
            original.clone(),
            request(&original, parameter),
            task_scope.clone(),
            &cancel,
        )
        .unwrap();
    let limits = WorkLimits {
        attempts: 10,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let mut strategy = NumericalStrategy::direct(StartPolicy::PreviousAccepted, limits);
    strategy.start.recovery = vec![StartOrigin::Auxiliary];
    let original_profile = ProfileRef {
        backend: Backend::Ipopt,
        key: original.strategy_profile().unwrap(),
    };
    strategy.mechanisms = vec![
        Mechanism {
            operation: Default::default(),
            kind: MechanismKind::Continuation,
            position: Position::Preparation,
            required: true,
            profile: Some(ProfileRef {
                backend: Backend::Ipopt,
                key: profile_key(path.profile()).unwrap().as_id(),
            }),
            support: path.support().into_iter().collect(),
            limits,
            starts: vec![StartOrigin::Accepted],
            transitions: vec![Transition::Continue, Transition::Stop],
        },
        Mechanism {
            operation: Default::default(),
            kind: MechanismKind::Direct,
            position: Position::Execution,
            required: true,
            profile: Some(original_profile),
            support: vec![],
            limits,
            starts: vec![StartOrigin::Specification, StartOrigin::Auxiliary],
            transitions: vec![Transition::Finish, Transition::Stop],
        },
    ];
    prepared.solve = original
        .clone()
        .within_task(task_scope)
        .unwrap()
        .with_strategy(
            strategy,
            vec![
                PreparedRung::Path {
                    prepared: path,
                    start: Box::new(start),
                },
                original.into(),
            ],
        )
        .unwrap();
    let result = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(result.accepted);
    let point = result.path_start(vec![0., 1.]).unwrap();
    assert!((point.point()[0] - 2.).abs() < 1e-7);
}

#[cfg(feature = "solver-ipopt")]
fn event_policy() -> EventPolicy {
    EventPolicy {
        observations: 6,
        endpoints: true,
        localization_steps: 30,
        localization_tolerance: 1e-8,
        rank_threshold: 1e-6,
        nondegeneracy_threshold: 1e-7,
        probe: arclength::ProbeLimits {
            bytes: 1 << 20,
            svds: 2,
        },
    }
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn actual_compiled_second_curvature_localizes_fold_and_first_only_keeps_it_unresolved() {
    let (runtime,package,analysis,prepared)=prepared("package p { def Root { param p:Scalar=-0.9375; var x:Scalar; annotation start x(0.25); eq root:x*x-p==1; } }").await;
    let original = prepared.solve.clone();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    let cancel = crate::CancelSource::new();
    let task = scope();
    let base = package
        .solve_case(prepared.clone(), analysis.compiler, &cancel)
        .await
        .unwrap();
    let start = base.path_start(vec![-1., 0.]).unwrap();
    let curvature = package
        .prepare_path_curvature(&prepared, task.clone(), &cancel)
        .await
        .unwrap();
    assert_eq!(curvature.source(), original.preparation_identity().unwrap());
    let mut requested = request(&original, parameter);
    requested.policy.events = Some(event_policy());
    requested.curvature = Some(curvature.clone());
    let path = runtime
        .native()
        .prepare_path(original.clone(), requested.clone(), task.clone(), &cancel)
        .unwrap();
    let attempt_capacity = path.attempt_capacity().unwrap();
    let outcome = runtime
        .native()
        .path_task(path, start.clone(), &cancel)
        .await
        .unwrap();
    assert!(outcome.terminal.is_none(), "{:?}", outcome.terminal);
    let fold = outcome
        .events
        .iter()
        .find(|event| event.kind == arclength::EventKind::SimpleFold)
        .expect("actual localized native fold event");
    assert!(fold.point[0].abs() < 1e-6);
    assert!((fold.point[1] + 1.).abs() < 1e-7);
    assert_eq!(fold.state_rank, 0);
    assert_eq!(fold.augmented_rank, 1);
    assert_eq!(fold.curvature_source, Some(curvature.key()));
    assert!((fold.curvature.unwrap().abs() - 2.).abs() < 1e-7);
    assert!(fold.transversality.unwrap().abs() > 0.9);
    assert_eq!(fold.work.rank_probes, 2);
    assert_eq!(fold.work.curvature_actions, 1);
    let localized = fold.localization.unwrap();
    assert!(localized.interval.1 - localized.interval.0 <= event_policy().localization_tolerance);
    assert!(outcome.native_calls > 4);
    assert!(outcome.native_calls <= attempt_capacity as u64);
    assert!(outcome.auxiliary_evaluations > 0);
    let shared = outcome.events.clone();
    assert!(pse_math::SharedAllocation::ptr_eq(&shared, &outcome.events));
    drop(outcome);
    assert!(
        shared
            .iter()
            .any(|event| event.kind == arclength::EventKind::SimpleFold)
    );
    requested.curvature = None;
    let path = runtime
        .native()
        .prepare_path(original.clone(), requested, task.clone(), &cancel)
        .unwrap();
    let first = runtime
        .native()
        .path_task(path, start, &cancel)
        .await
        .unwrap();
    assert!(first.terminal.is_none(), "{:?}", first.terminal);
    let unresolved = first
        .events
        .iter()
        .find(|event| event.localization.is_some())
        .unwrap();
    assert_eq!(unresolved.kind, arclength::EventKind::Unresolved);
    assert_eq!(unresolved.curvature, None);
    assert_eq!(unresolved.curvature_source, None);
    assert_eq!(unresolved.work.curvature_actions, 0);
    let Representation::Algebraic(case) = &original.representation else {
        panic!("algebraic")
    };
    let actual_first = case
        .sensitivity
        .as_ref()
        .unwrap()
        .available()
        .unwrap()
        .program
        .clone();
    assert!(
        runtime
            .native()
            .prepare_path_curvature(original, actual_first, task, &cancel)
            .await
            .is_err()
    );
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn shared_driver_retains_actual_localized_fold_event_under_original_completion() {
    use pse_model::generated::enums::NumericalPathEventKind;
    use pse_model::strategy::{
        Mechanism, MechanismKind, NumericalStrategy, Position, ProfileRef, StartOrigin, Transition,
        WorkLimits,
    };
    let (runtime,package,analysis,mut prepared)=prepared("package p { def Root { param p:Scalar=-0.9375; var x:Scalar; annotation start x(0.25); eq root:x*x-p==1; } }").await;
    let original = prepared.solve.clone();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    let cancel = crate::CancelSource::new();
    let task = scope();
    let base = package
        .solve_case(prepared.clone(), analysis.compiler, &cancel)
        .await
        .unwrap();
    let start = base.path_start(vec![-1., 0.]).unwrap();
    let mut requested = request(&original, parameter);
    requested.policy.events = Some(event_policy());
    requested.curvature = Some(
        package
            .prepare_path_curvature(&prepared, task.clone(), &cancel)
            .await
            .unwrap(),
    );
    let path = runtime
        .native()
        .prepare_path(original.clone(), requested, task.clone(), &cancel)
        .unwrap();
    let limits = WorkLimits {
        attempts: 40,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let mut declaration = NumericalStrategy::direct(StartPolicy::PreviousAccepted, limits);
    declaration.start.recovery = vec![StartOrigin::Auxiliary];
    declaration.mechanisms = vec![
        Mechanism {
            operation: Default::default(),
            kind: MechanismKind::Continuation,
            position: Position::Preparation,
            required: true,
            profile: Some(ProfileRef {
                backend: Backend::Ipopt,
                key: profile_key(path.profile()).unwrap().as_id(),
            }),
            support: path.support().into_iter().collect(),
            limits,
            starts: vec![StartOrigin::Accepted],
            transitions: vec![Transition::Continue, Transition::Stop],
        },
        Mechanism {
            operation: Default::default(),
            kind: MechanismKind::Direct,
            position: Position::Execution,
            required: true,
            profile: Some(ProfileRef {
                backend: Backend::Ipopt,
                key: original.strategy_profile().unwrap(),
            }),
            support: vec![],
            limits,
            starts: vec![StartOrigin::Specification, StartOrigin::Auxiliary],
            transitions: vec![Transition::Finish, Transition::Stop],
        },
    ];
    prepared.solve = original
        .clone()
        .within_task(task)
        .unwrap()
        .with_strategy(
            declaration,
            vec![
                PreparedRung::Path {
                    prepared: path,
                    start: Box::new(start),
                },
                original.into(),
            ],
        )
        .unwrap();
    let result = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(result.accepted);
    let trace = result.strategy.as_ref().unwrap();
    let retained = trace.products[0].path_events.as_ref().unwrap().clone();
    let rows = trace.rows(result.run_id, 0).unwrap();
    let fold = rows
        .iter()
        .filter_map(|row| row.path_events.as_ref())
        .flatten()
        .find(|event| event.kind == NumericalPathEventKind::SimpleFold)
        .unwrap();
    assert_eq!(fold.state_rank, 0);
    assert_eq!(fold.augmented_rank, 1);
    assert_eq!(fold.rank_probes, 2);
    assert!(fold.localization.is_some());
    assert!(fold.curvature_source.is_some());
    drop(result);
    assert!(
        retained
            .iter()
            .any(|event| event.kind == arclength::EventKind::SimpleFold)
    );
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn actual_compiled_branch_singularity_is_unresolved_despite_second_curvature_and_source_mismatch_refuses()
 {
    let (runtime,package,analysis,preparation)=prepared("package p { def Root { param p:Scalar=0; var x:Scalar; annotation start x(0); eq root:x*x-p*p==0; } }").await;
    let original = preparation.solve.clone();
    let parameter = preparation.model.model.compiled().model.paths["p"];
    let cancel = crate::CancelSource::new();
    let task = scope();
    let curvature = package
        .prepare_path_curvature(&preparation, task.clone(), &cancel)
        .await
        .unwrap();
    let base = package
        .solve_case(preparation, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(base.accepted);
    let start = base
        .path_start(vec![std::f64::consts::FRAC_1_SQRT_2; 2])
        .unwrap();
    let mut requested = request(&original, parameter);
    requested.policy.steps = 1;
    requested.policy.events = Some(event_policy());
    requested.curvature = Some(curvature.clone());
    let path = runtime
        .native()
        .prepare_path(original.clone(), requested.clone(), task.clone(), &cancel)
        .unwrap();
    let outcome = runtime
        .native()
        .path_task(path, start, &cancel)
        .await
        .unwrap();
    assert!(outcome.terminal.is_some());
    let event = &outcome.events[0];
    assert_eq!(event.kind, arclength::EventKind::Unresolved);
    assert_eq!(event.state_rank, 0);
    assert_eq!(event.augmented_rank, 0);
    assert!(event.curvature.is_some());
    assert_eq!(event.curvature_source, Some(curvature.key()));
    assert!(
        !outcome
            .events
            .iter()
            .any(|event| event.kind == arclength::EventKind::SimpleFold)
    );
    let (_other_runtime,other_package,_other_analysis,other)=prepared("package p { def Root { param p:Scalar=0; var x:Scalar; annotation start x(0); eq root:x*x-p*p==1; } }").await;
    let other_curvature = other_package
        .prepare_path_curvature(&other, task.clone(), &cancel)
        .await
        .unwrap();
    assert!(
        runtime
            .native()
            .prepare_path_curvature(
                original.clone(),
                other_curvature.0.program.clone(),
                task.clone(),
                &cancel
            )
            .await
            .is_err()
    );
    requested.curvature = Some(other_curvature);
    assert!(
        runtime
            .native()
            .prepare_path(original, requested, task, &cancel)
            .is_err()
    );
}
