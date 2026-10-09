// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused authored conditional, automatic recycle, and original assessment controls.
use super::*;
use crate::workflow::tests::{compiler_profile, id, physical, runtime};
use pse_structural::flowsheet::{Decision, Policy};

/// Actual arithmetic source with a growing local row/support/binding population.
async fn construction_source(runtime: Runtime, internal_rows: usize) -> PreparedRecycle {
    let mut source = String::from(
        "package demand {def Root {var x:Scalar; var y:Scalar; eq local:y==x/2+1; annotation start x(4); annotation start y(1);",
    );
    for row in 0..internal_rows {
        source.push_str(&format!(
            "var hidden{row}:Scalar; eq internal{row}:hidden{row}==y+{}; annotation start hidden{row}(1);",
            row + 3
        ));
    }
    source.push_str("state incoming supplied(true) {coordinate value=x; transport value=x tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=y; transport value=y tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet;}}");
    let declarations = pse_authoring::language::parse(
        &source,
        id(96),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &model.compiled().model;
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|port| port.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|edge| {
                (
                    *edge,
                    Decision {
                        id: *edge,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node: analysis.instance.as_id(),
            inputs: BTreeSet::from([port("inlet")]),
            outputs: BTreeSet::from([port("outlet")]),
            realization: CausalUnitRealization::Conditional {
                residuals: model
                    .equations
                    .iter()
                    .filter(|row| {
                        row.lineage.path.ends_with(".local")
                            || row
                                .lineage
                                .path
                                .rsplit('.')
                                .next()
                                .unwrap()
                                .starts_with("internal")
                    })
                    .map(|row| row.id)
                    .collect(),
                unknowns: model
                    .symbols
                    .values()
                    .filter(|symbol| {
                        symbol.lineage.path.ends_with(".y")
                            || symbol
                                .lineage
                                .path
                                .rsplit('.')
                                .next()
                                .unwrap()
                                .starts_with("hidden")
                    })
                    .map(|symbol| symbol.id)
                    .collect(),
                solver: Box::new(crate::math::settings::SolveSettings {
                    intent: SolveIntent::Root,
                    ..Default::default()
                }),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    package
        .prepare_recycle(&analysis, selection, request, &cancel)
        .await
        .unwrap()
}

fn construction_runtime() -> Runtime {
    crate::workflow::tests::runtime_on(
        1 << 30,
        crate::math::MathPolicy {
            worker_bytes: 512 << 20,
            workspace_bytes: 128 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    )
}

#[tokio::test]
async fn conditional_construction_bounds_follow_source_and_binding_populations() {
    let runtime = construction_runtime();
    let small = construction_source(runtime.clone(), 1).await;
    let larger = construction_source(runtime.clone(), 4).await;
    assert!(small.providers.is_empty());
    assert!(larger.providers.is_empty());
    let compiled = small._source.case.compiled();
    let grown = larger._source.case.compiled();
    assert!(grown.plan.structure().rows().len() > compiled.plan.structure().rows().len());
    assert!(grown.plan.columns().len() > compiled.plan.columns().len());
    let support = compiled
        .support_upgrade_allocation_bound(DerivativeOrder::Second)
        .unwrap()
        .unwrap();
    let schedule = compiled.initialization_allocation_bound().unwrap().unwrap();
    assert!(
        grown
            .support_upgrade_allocation_bound(DerivativeOrder::Second)
            .unwrap()
            .unwrap()
            > support
    );
    assert!(grown.initialization_allocation_bound().unwrap().unwrap() > schedule);
    // Matching has an actual stack allowance; it is included in source demand,
    // independently of the generous configured worker/workspace maxima.
    assert!(schedule >= pse_structural::incidence::MATCHING_STACK);
    assert!(schedule < runtime.shared.budget().math.workspace_bytes);
    let factory = small.factory_allocation_bound().unwrap().unwrap();
    assert!(larger.factory_allocation_bound().unwrap().unwrap() > factory);
    assert!(factory < runtime.shared.budget().math.worker_bytes);
    let program = &small.programs[0];
    let view = &program.conditional.as_ref().unwrap().view;
    let binding = view
        .binding_allocation_bound(&program.values)
        .unwrap()
        .unwrap();
    assert!(
        larger.programs[0]
            .conditional
            .as_ref()
            .unwrap()
            .view
            .binding_allocation_bound(&larger.programs[0].values)
            .unwrap()
            .unwrap()
            > binding
    );
    let mut changed_values = program.values.clone();
    *changed_values.scalars.values_mut().next().unwrap() += 1.0;
    assert_eq!(
        view.binding_allocation_bound(&changed_values).unwrap(),
        Some(binding)
    );
    changed_values.scalars.insert(id(97), 3.0);
    assert!(
        view.binding_allocation_bound(&changed_values)
            .unwrap()
            .unwrap()
            > binding
    );
}

#[tokio::test]
async fn conditional_schedule_and_first_binding_enter_with_source_demand() {
    let runtime = construction_runtime();
    let prepared = construction_source(runtime.clone(), 1).await;
    let compiled = prepared._source.case.compiled().clone();
    let demand = compiled.initialization_allocation_bound().unwrap().unwrap();
    let pool = runtime.shared.pool();
    let policy = &runtime.shared.budget().math;
    let fixed = policy.stack_bytes + policy.foreign_bytes + policy.inner_session_bytes;
    // Less free space than either maximum-sized entry: the real source schedule
    // must enter immediately with its demand and preserve matching's full stack.
    let free = fixed + demand + (8 << 20);
    assert!(free < fixed + policy.workspace_bytes);
    assert!(free < fixed + policy.worker_bytes);
    let pressure = runtime
        .native()
        .reserve(
            "test:conditional-source-pressure",
            runtime.shared.budget().memory_limit_bytes.get() - pool.reserved() - free,
        )
        .unwrap();
    let baseline = pool.reserved();
    let observed = pool.clone();
    let blocks = runtime
        .native()
        .submit(1, demand, move |flag, _| {
            assert_eq!(observed.reserved(), baseline + fixed + demand);
            let pse_compiler::workspace::Alternative::Available(blocks) =
                compiled.automatic_blocks(&flag)?
            else {
                panic!("authored arithmetic source must have a complete schedule");
            };
            let retained = blocks
                .iter()
                .map(|block| block.plan.retained_bytes() + block.structure.retained_bytes())
                .sum();
            Ok((blocks, retained))
        })
        .unwrap()
        .finish()
        .await
        .unwrap();
    assert!(blocks.0.len() >= 2);
    drop(blocks);
    assert_eq!(pool.reserved(), baseline);
    let program = &prepared.programs[0];
    let view = program.conditional.as_ref().unwrap().view.clone();
    let values = program.values.clone();
    let quantities = prepared._source.case.compiled().quantities.clone();
    let demand = view.binding_allocation_bound(&values).unwrap().unwrap();
    let expected = view.plan.columns().to_vec();
    let observed = pool.clone();
    runtime
        .native()
        .job(
            1,
            demand,
            pse_columnar::flight::FlightCancellation::default(),
            move |flag| {
                assert_eq!(observed.reserved(), baseline + fixed + demand);
                let bound = view.bind(quantities, &values, &flag)?;
                assert_eq!(bound.plan.columns(), expected);
                assert!(bound.values_match(&values));
                Ok(())
            },
        )
        .await
        .unwrap();
    assert_eq!(pool.reserved(), baseline);
    // The public recycle constructor consumes the same source-issued factory
    // bound, including its first conditional binding and live unit evaluators.
    let original = prepared._source.values.identity();
    let result = prepared.start().unwrap().finish().await.unwrap();
    let candidate = result.original_candidate.as_ref().unwrap();
    assert_eq!(
        candidate.scalars.len(),
        prepared._source.values.scalars.len()
    );
    assert_eq!(prepared._source.values.identity(), original);
    assert_eq!(
        result.candidate_use(),
        pse_model::generated::enums::CandidateUse::Usable
    );
    drop(result);
    assert_eq!(pool.reserved(), baseline);
    drop(pressure);
}

#[tokio::test]
async fn conditional_construction_oversized_and_overflow_refuse_before_dispatch() {
    let prepared = construction_source(construction_runtime(), 1).await;
    let demand = prepared.factory_allocation_bound().unwrap().unwrap();
    let limited = crate::workflow::tests::runtime_on(
        512 << 20,
        crate::math::MathPolicy {
            worker_bytes: demand - 1,
            workspace_bytes: 128 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let mut oversized = prepared.clone();
    oversized.runtime = limited.clone();
    let baseline = limited.shared.pool().reserved();
    assert!(matches!(
        oversized.start(),
        Err(WorkflowError::Math(MathRuntimeError::Limit(
            "declared root construction capacity"
        )))
    ));
    assert_eq!(limited.shared.pool().reserved(), baseline);
    assert!(
        CausalMap::construction_allocation_bound(&prepared.graph, usize::MAX, prepared.fixed.len())
            .is_err()
    );
    for (rows, columns, contributions) in [
        (pse_structural::incidence::MATCHING_ROWS + 1, 1, 1),
        (1, i32::MAX as usize + 1, 1),
        (1, 1, usize::MAX),
        (usize::MAX, 1, 1),
    ] {
        assert!(matches!(
            pse_structural::incidence::CaseIncidence::memory_extent(rows, columns, contributions),
            Err(pse_structural::projection::ProjectionError::Limit)
        ));
    }
    let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = called.clone();
    assert!(matches!(
        limited.native().submit(1, usize::MAX, move |_, _| {
            observed.store(true, std::sync::atomic::Ordering::Release);
            Ok(((), 0))
        }),
        Err(MathRuntimeError::Limit("native allowance overflow"))
    ));
    assert!(!called.load(std::sync::atomic::Ordering::Acquire));
    assert_eq!(limited.shared.pool().reserved(), baseline);
}

#[tokio::test]
async fn automatic_workflow_causal_acyclic_refusal_retains_original_alternatives() {
    use pse_model::strategy::MechanismKind;
    let declarations = pse_authoring::language::parse(
        "package causal {def Feed {param value:Scalar=2; port outlet:Scalar=value; annotation connectivity outlet(0,1);} def Sink {var x:Scalar; var y:Scalar; eq local:y==x+3; port inlet:Scalar=x; annotation connectivity inlet(1,0); annotation start x(0); annotation start y(0);} def Root {child feed:Feed=Feed(); child sink:Sink=Sink(); connect feed.outlet -> sink.inlet;}}",
        id(94), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime()
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let mut profile = super::super::tests::profile(SolveIntent::Root);
    profile.selection = SolverSelection::Explicit(Backend::Kinsol);
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            DerivativeOrder::First,
            compiler_profile(),
            profile,
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let y = prepared
        .model
        .model
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(".sink.y"))
        .unwrap()
        .id;
    let result = package
        .solve_case(prepared, compiler_profile(), &cancel)
        .await
        .unwrap();
    assert!(
        result.completion.decision.permits_use(),
        "{:?}",
        result.completion
    );
    assert!((result.values.scalars[&y] - 5.0).abs() < 1e-7);
    let trace = result.strategy.as_ref().unwrap();
    let map = trace
        .declaration
        .mechanisms
        .iter()
        .position(|mechanism| mechanism.kind == MechanismKind::MapsAnderson)
        .unwrap();
    assert!(!trace.events.iter().any(|event| event.mechanism == map
        && event.kind == pse_model::generated::enums::NumericalEventKind::Started));
    assert!(trace.events.iter().any(|event| event.mechanism == map
        && event.kind == pse_model::generated::enums::NumericalEventKind::Refused));
    assert!(matches!(
        trace.events.last().unwrap().original,
        Some(crate::math::strategy::OriginalConclusion::Satisfied)
    ));
}

#[tokio::test]
async fn automatic_workflow_causal_missing_locality_preserves_original_completion() {
    use pse_model::strategy::MechanismKind;
    let declarations = pse_authoring::language::parse(
        "package causal {def Feed {var x:Scalar; var hidden:Scalar; let value:Scalar=x/2+1; port inlet:Scalar=x; port outlet:Scalar=value; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); annotation start x(4); annotation start hidden(0);} def Sink {var x:Scalar; port inlet:Scalar=x; port outlet:Scalar=x; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); annotation start x(0);} def Root {child feed:Feed=Feed(); child sink:Sink=Sink(); connect feed.outlet -> sink.inlet; connect sink.outlet -> feed.inlet; eq cross:feed.hidden==sink.x+3;}}",
        id(95), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime()
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let mut profile = super::super::tests::profile(SolveIntent::Root);
    profile.selection = SolverSelection::Explicit(Backend::Kinsol);
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            DerivativeOrder::First,
            compiler_profile(),
            profile,
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let frozen = prepared.model.values.identity();
    let hidden = prepared
        .model
        .model
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(".feed.hidden"))
        .unwrap()
        .id;
    let result = package
        .solve_case(prepared.clone(), compiler_profile(), &cancel)
        .await
        .unwrap();
    assert!(
        result.completion.decision.permits_use(),
        "{:?}",
        result.completion
    );
    assert!((result.values.scalars[&hidden] - 5.0).abs() < 1e-6);
    assert_eq!(prepared.model.values.identity(), frozen);
    let trace = result.strategy.as_ref().unwrap();
    let map = trace
        .declaration
        .mechanisms
        .iter()
        .position(|mechanism| mechanism.kind == MechanismKind::MapsAnderson)
        .unwrap();
    let refused = trace
        .events
        .iter()
        .find(|event| {
            event.mechanism == map
                && event.kind == pse_model::generated::enums::NumericalEventKind::Refused
        })
        .unwrap();
    assert_eq!(
        refused.observation,
        Some(pse_model::generated::enums::NumericalAttemptObservation::CapabilityRefusal)
    );
    let diagnostic = pse_model::diagnostic::DiagnosticProjection::boundary_diagnostic(
        refused.cause.as_ref().unwrap().as_ref(),
        pse_diagnostics::DiagnosticStage::ModelingConditionalUnitAdmission,
    );
    assert_eq!(
        diagnostic.class,
        pse_model::diagnostic::BoundaryClass::Unsupported
    );
    assert!(!diagnostic.causes.is_empty());
    assert!(matches!(
        trace.events.last().unwrap().original,
        Some(crate::math::strategy::OriginalConclusion::Satisfied)
    ));
    assert_eq!(trace.start, pse_model::strategy::StartOrigin::Specification);
}

async fn causal_reconstruction_case(
    nonzero: bool,
) -> (
    ModelingPackage,
    crate::workflow::ModelingSolvePreparation,
    SolverProfile,
) {
    let allowance = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let declarations = pse_authoring::language::parse(
        "package causal {def Root {var x:Scalar; var y:Scalar; var hidden:Scalar; eq local:y==x/2+1; eq internal:hidden==y+3; state incoming supplied(true) {coordinate value=x; transport value=x tolerance ALLOWANCE{1};} state outgoing supplied(false) {coordinate value=y; transport value=y tolerance ALLOWANCE{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(4); annotation start y(LOCAL_START); annotation start hidden(LOCAL_START);}}".replace("LOCAL_START", if nonzero {"1"} else {"0"}).replace("ALLOWANCE", &allowance.to_string()).as_str(),
        id(93), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime()
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let mut profile = super::super::tests::profile(SolveIntent::Root);
    profile.selection = SolverSelection::Explicit(Backend::Kinsol);
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            DerivativeOrder::First,
            compiler_profile(),
            profile.clone(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    (package, prepared, profile)
}

#[tokio::test]
async fn automatic_workflow_causal_native_failure_does_not_commit_partial_state() {
    use pse_model::strategy::MechanismKind;
    let (package, prepared, _) = causal_reconstruction_case(false).await;
    let frozen = prepared.model.values.identity();
    let hidden = prepared
        .model
        .model
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(".hidden"))
        .unwrap()
        .id;
    let result = package
        .solve_case(
            prepared.clone(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(
        result.completion.decision.permits_use(),
        "{:?}",
        result.completion
    );
    assert_eq!(prepared.model.values.identity(), frozen);
    let target = crate::workflow::tests::engineering_target(
        result.prepared.solve.numerics(),
        pse_model::generated::enums::NumericalTarget::Variable,
        hidden,
    );
    assert!((result.values.scalars[&hidden] - 5.0).abs() <= target.budget);
    let trace = result.strategy.as_ref().unwrap();
    let map = trace
        .declaration
        .mechanisms
        .iter()
        .position(|mechanism| {
            mechanism.kind == MechanismKind::MapsAnderson
                && mechanism.position == pse_model::strategy::Position::Execution
        })
        .unwrap();
    let failed = trace
        .events
        .iter()
        .find(|event| {
            event.mechanism == map
                && event.observation
                    == Some(
                        pse_model::generated::enums::NumericalAttemptObservation::NumericalFailure,
                    )
        })
        .unwrap();
    assert!(!matches!(
        failed.original,
        Some(crate::math::strategy::OriginalConclusion::Satisfied)
    ));
    let native::ProblemError::Numerical {
        status: Some(status),
        ..
    } = failed.cause.as_ref().unwrap().as_ref()
    else {
        panic!("actual native failure status missing");
    };
    assert_eq!(status.code, -7);
    assert_eq!(status.name, "KIN_MXNEWT_5X_EXCEEDED");
    assert!(
        trace
            .starts
            .iter()
            .all(|origin| *origin == pse_model::strategy::StartOrigin::Specification)
    );
    assert!(matches!(
        trace.events.last().unwrap().original,
        Some(crate::math::strategy::OriginalConclusion::Satisfied)
    ));
}

#[tokio::test]
async fn automatic_workflow_causal_dispatch_reconstructs_nonport_original_state() {
    use crate::math::solves::Outcome;
    use pse_model::strategy::MechanismKind;
    let (package, prepared, profile) = causal_reconstruction_case(true).await;
    let cancel = crate::CancelSource::new();
    let original_start = prepared.model.values.identity();
    let hidden = prepared
        .model
        .model
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(".hidden"))
        .unwrap()
        .id;
    let result = package
        .solve_case(prepared.clone(), compiler_profile(), &cancel)
        .await
        .unwrap();
    assert!(
        result.completion.decision.permits_use(),
        "{:?}",
        result.completion
    );
    assert_eq!(prepared.model.values.identity(), original_start);
    let Outcome::Constant(original) = &result.outcome else {
        panic!("expected fresh full-original causal assessment");
    };
    assert_eq!(
        original.component_reports().count(),
        1,
        "mechanisms {:?}; events {:?}",
        result
            .strategy
            .as_ref()
            .unwrap()
            .declaration
            .mechanisms
            .iter()
            .map(|mechanism| (mechanism.kind, mechanism.position))
            .collect::<Vec<_>>(),
        result
            .strategy
            .as_ref()
            .unwrap()
            .events
            .iter()
            .map(|event| (
                event.mechanism,
                event.kind,
                event.observation,
                event.cause.as_ref().map(|cause| cause.to_string())
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        original.component_reports().next().unwrap().backend,
        Backend::Kinsol
    );
    assert_eq!(original.work.evaluations, None);
    assert_eq!(original.work.iterations, None);
    assert_eq!(original.work.factorizations, None);
    assert_eq!(original.work.proof_steps, None);
    assert!(
        (original
            .coordinates
            .iter()
            .find(|(id, _)| *id == hidden)
            .unwrap()
            .1
            - 5.0)
            .abs()
            <= crate::workflow::tests::engineering_target(
                result.prepared.solve.numerics(),
                pse_model::generated::enums::NumericalTarget::Variable,
                hidden
            )
            .budget
    );
    let trace = result.strategy.as_ref().unwrap();
    assert!(
        trace
            .declaration
            .mechanisms
            .iter()
            .any(|mechanism| mechanism.kind == MechanismKind::MapsAnderson)
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.kind == pse_model::generated::enums::NumericalEventKind::Started)
    );
    assert!(trace.events.iter().all(|event| event.decision.is_some()));
    assert!(matches!(
        trace.events.last().unwrap().original,
        Some(crate::math::strategy::OriginalConclusion::Satisfied)
    ));
    assert_eq!(trace.declaration.branch, profile.composition.branch);
    assert_eq!(trace.declaration.start.policy, StartPolicy::NoPriorStart);

    // A later explicit start removes the frozen declared-start map from eligibility.
    let mut explicit = prepared.clone();
    let seed = explicit
        .model
        .case
        .compiled()
        .plan
        .columns()
        .iter()
        .map(|id| (*id, 7.0))
        .collect();
    explicit.solve = explicit.solve.with_primal_start(seed).unwrap();
    let result = package
        .solve_case(explicit, compiler_profile(), &cancel)
        .await
        .unwrap();
    assert!(
        result.completion.decision.permits_use(),
        "{:?}",
        result.completion
    );
    assert!(matches!(result.outcome, Outcome::Native(_)));
    let trace = result.strategy.as_ref().unwrap();
    assert!(
        trace
            .declaration
            .mechanisms
            .iter()
            .all(|mechanism| mechanism.kind == MechanismKind::Direct)
    );
    assert_eq!(trace.start, pse_model::strategy::StartOrigin::Explicit);
    assert_eq!(prepared.model.values.identity(), original_start);
}

#[tokio::test]
async fn explicit_map_admission_requires_free_branch_controls_without_promoting_value() {
    for (index, declaration, allowed) in [
        (
            0,
            "var switching:Scalar; eq switch_value:switching==1; annotation start switching(1);",
            false,
        ),
        (1, "param switching:Scalar=1;", true),
    ] {
        let runtime = runtime();
        let source = "package p {def Root {CONTROL var x:Scalar; let result:Scalar=(if switching>0 then 1 else -1); state incoming supplied(true) {coordinate value=x; transport value=x tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=result; transport value=result tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(1);}}".replace("CONTROL", declaration);
        let declarations = pse_authoring::language::parse(
            &source,
            id(91 + index),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let analysis = package
            .declared_execution(
                root,
                compiler_profile(),
                super::super::tests::profile(SolveIntent::Root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap()
            .analysis;
        let prepared = package
            .prepare(
                root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                &cancel,
            )
            .await
            .unwrap();
        let model = &prepared.compiled().model;
        let port = |name: &str| {
            model
                .material_ports
                .values()
                .find(|port| port.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let selection = ModelingFlowSelection {
            nodes: BTreeSet::from([analysis.instance]),
            connections: model
                .connections
                .keys()
                .map(|&id| {
                    (
                        id,
                        Decision {
                            id,
                            cost: 1.0,
                            policy: Policy::Mandatory,
                        },
                    )
                })
                .collect(),
        };
        let request = RecycleRequest {
            tears: selection.connections.keys().copied().collect(),
            units: vec![CausalUnitRequest {
                node: analysis.instance.as_id(),
                inputs: BTreeSet::from([port("inlet")]),
                outputs: BTreeSet::from([port("outlet")]),
                realization: CausalUnitRealization::ExplicitMap,
            }],
            anderson: 0,
            damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
        };
        let result = package
            .prepare_recycle(&analysis, selection.clone(), request, &cancel)
            .await;
        if allowed {
            let admitted = result.unwrap();
            let automatic = RecycleRequest::from_model(
                admitted._source.model.compiled(),
                admitted._source.case.compiled(),
                &selection,
                &admitted._source.case.compiled().quantities,
                crate::math::settings::SolveSettings::default(),
                None,
            )
            .unwrap();
            assert_eq!(automatic.units.len(), 1);
            assert!(matches!(
                automatic.units[0].realization,
                CausalUnitRealization::ExplicitMap
            ));
            assert_eq!(
                automatic.tears,
                selection.connections.keys().copied().collect()
            );
            let assembly = &admitted.programs[0].program.assembly;
            assert_eq!(assembly.order(), DerivativeOrder::Value);
            assert!(
                assembly
                    .supports()
                    .iter()
                    .all(|support| support.support().first.is_empty())
            );
        } else {
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("causal function depends on an undeclared free input"),
                "{error}"
            );
        }
    }
}

#[tokio::test]
async fn conditional_unit_solves_original_rows_and_restores_each_boundary_overlay() {
    let runtime = runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Scalar; var z:Scalar; var y:Scalar; var w:Scalar; eq local:y==x/2+1; eq aux:w==z+3; let result:Scalar=y*2; state incoming supplied(true) {coordinate value=x; coordinate auxiliary=z; transport value=x tolerance 1e-7{1}; transport auxiliary=z tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=result; coordinate auxiliary=w; transport value=result tolerance 1e-7{1}; transport auxiliary=w tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(4); annotation start z(2); annotation start y(1); annotation start w(1);}}",
        id(85), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &model.compiled().model;
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
    };
    let symbol = |name: &str| {
        model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let row = |name: &str| {
        model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let unknowns = BTreeSet::from([symbol("y"), symbol("w")]);
    let rows = BTreeSet::from([row("local"), row("aux")]);
    let coordinate = |port: &pse_modeling::specialize::MaterialPort, name: &str| {
        *port
            .coordinates
            .iter()
            .find(|(key, _)| key.name == name)
            .unwrap()
            .1
    };
    let inlet = coordinate(port("inlet"), "value");
    let auxiliary = coordinate(port("inlet"), "auxiliary");
    let outlet = coordinate(port("outlet"), "value");
    let auxiliary_outlet = coordinate(port("outlet"), "auxiliary");
    assert!(expand_unit_ports(model, &BTreeSet::from([port("inlet").id, inlet])).is_err());
    let node = analysis.instance.as_id();
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let solver = crate::math::settings::SolveSettings {
        intent: SolveIntent::Root,
        ..Default::default()
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node,
            inputs: BTreeSet::from([port("inlet").id]),
            outputs: BTreeSet::from([port("outlet").id]),
            realization: CausalUnitRealization::Conditional {
                residuals: rows.clone(),
                unknowns: unknowns.clone(),
                solver: Box::new(solver),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let mut overlapping = request.clone();
    overlapping.units[0].inputs.insert(inlet);
    let refusal = package
        .prepare_recycle(&analysis, selection.clone(), overlapping, &cancel)
        .await
        .unwrap_err();
    let WorkflowError::ConditionalAdmission { diagnostic, .. } = refusal else {
        panic!("conditional aggregate admission lost its typed diagnostic")
    };
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionInvalidModel
    );
    assert!(diagnostic.sources.contains(&port("inlet").id));
    assert!(diagnostic.sources.contains(&inlet));
    assert!(unknowns.iter().all(|id| diagnostic.sources.contains(id)));
    assert!(rows.iter().all(|id| diagnostic.sources.contains(id)));
    let prepared = package
        .prepare_recycle(&analysis, selection.clone(), request.clone(), &cancel)
        .await
        .unwrap();
    let program = prepared.programs[0].clone();
    let original = program.values.identity();
    assert_eq!(prepared.request().units[0].inputs, request.units[0].inputs);
    assert_eq!(prepared.resolved_units().next().unwrap().inputs.len(), 2);
    assert_eq!(prepared.graph.declaration().connections.len(), 1);
    assert_eq!(
        prepared.graph.bindings()[&prepared.graph.declaration().connections[0].id].len(),
        2
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .rows,
        rows.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .columns,
        unknowns.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(program.program.assembly.order(), DerivativeOrder::Value);
    let controls = Controls::default();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &controls,
    );
    let budget = crate::math::WorkerBudget::new(16 << 20);
    let worker = runtime
        .native()
        .worker(
            program.program.clone(),
            &prepared.providers,
            execution.scope().unwrap(),
            &budget,
        )
        .unwrap();
    let mut unit = UnitWorker {
        last_values: None,
        input_ids: program.inputs.iter().map(|i| i.0).collect(),
        output_ids: program.outputs.iter().map(|o| o.0).collect(),
        program,
        worker,
        service: runtime.native().clone(),
        providers: prepared.providers,
        budget,
    };
    let charged = unit.budget.used();
    for (input, expected) in [(4.0, 6.0), (8.0, 10.0), (4.0, 6.0)] {
        let outputs = unit
            .evaluate(
                &BTreeMap::from([(inlet, input), (auxiliary, 2.0)]),
                &execution,
            )
            .unwrap();
        assert!((outputs[&outlet] - expected).abs() < 1e-7);
        assert!((outputs[&auxiliary_outlet] - 5.0).abs() < 1e-7);
        assert_eq!(unit.program.values.identity(), original);
        assert_eq!(unit.budget.used(), charged);
    }
    assert!(unit.evaluate(&BTreeMap::new(), &execution).is_err());
    assert!(
        unit.evaluate(
            &BTreeMap::from([(inlet, f64::NAN), (auxiliary, 2.0)]),
            &execution
        )
        .is_err()
    );
    let mut expired = execution.clone();
    expired.time_limit = std::time::Duration::ZERO;
    let error = unit
        .evaluate(&BTreeMap::from([(inlet, 4.0), (auxiliary, 2.0)]), &expired)
        .unwrap_err();
    assert_eq!(
        native::callback::classify(&error),
        native::callback::Failure::Stopped(Termination::TimeLimit)
    );
    assert!(!execution.cancel.load(std::sync::atomic::Ordering::Acquire));
    execution
        .cancel
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, 4.0)]), &execution)
            .is_err()
    );
    assert_eq!(unit.program.values.identity(), original);
    assert_eq!(unit.budget.used(), charged);

    let mut explicit = request.clone();
    explicit.units[0].realization = CausalUnitRealization::ExplicitMap;
    assert!(
        package
            .prepare_recycle(&analysis, selection.clone(), explicit, &cancel)
            .await
            .is_err()
    );
    let mut invalid = request;
    if let CausalUnitRealization::Conditional { solver, .. } = &mut invalid.units[0].realization {
        solver.backend = Some(Backend::Highs);
    }
    assert!(
        package
            .prepare_recycle(&analysis, selection, invalid, &cancel)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn conditional_unit_derived_boundary_solves_constituent_variables() {
    let runtime = runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Scalar; var z:Scalar; var y:Scalar; var w:Scalar; eq local:y==x/2+1; eq aux:w==z+3; let result:Scalar=y*2; let boundary:Scalar=x*2; state incoming supplied(true) {coordinate value=boundary; coordinate auxiliary=z; transport value=boundary tolerance 1e-7{1}; transport auxiliary=z tolerance 1e-7{1};} state outgoing supplied(false) {coordinate value=result; coordinate auxiliary=w; transport value=result tolerance 1e-7{1}; transport auxiliary=w tolerance 1e-7{1};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(4); annotation start z(2); annotation start y(1); annotation start w(1);}}",
        id(85), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &model.compiled().model;
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
    };
    let symbol = |name: &str| {
        model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let row = |name: &str| {
        model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let unknowns = BTreeSet::from([symbol("x"), symbol("y"), symbol("w")]);
    let rows = BTreeSet::from([row("local"), row("aux")]);
    let coordinate = |port: &pse_modeling::specialize::MaterialPort, name: &str| {
        *port
            .coordinates
            .iter()
            .find(|(key, _)| key.name == name)
            .unwrap()
            .1
    };
    let inlet = coordinate(port("inlet"), "value");
    let auxiliary = coordinate(port("inlet"), "auxiliary");
    let outlet = coordinate(port("outlet"), "value");
    let auxiliary_outlet = coordinate(port("outlet"), "auxiliary");
    assert!(expand_unit_ports(model, &BTreeSet::from([port("inlet").id, inlet])).is_err());
    let node = analysis.instance.as_id();
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let solver = crate::math::settings::SolveSettings {
        intent: SolveIntent::Root,
        ..Default::default()
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node,
            inputs: BTreeSet::from([port("inlet").id]),
            outputs: BTreeSet::from([port("outlet").id]),
            realization: CausalUnitRealization::Conditional {
                residuals: rows.clone(),
                unknowns: unknowns.clone(),
                solver: Box::new(solver),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let prepared = package
        .prepare_recycle(&analysis, selection.clone(), request.clone(), &cancel)
        .await
        .unwrap();
    let program = prepared.programs[0].clone();
    let original = program.values.identity();
    assert!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .plan
            .structure()
            .parameters()
            .iter()
            .any(|p| p.id == pse_ids::named_id(symbol("boundary"), "conditional-boundary-value"))
    );
    assert_eq!(prepared.request().units[0].inputs, request.units[0].inputs);
    assert_eq!(prepared.resolved_units().next().unwrap().inputs.len(), 2);
    assert_eq!(prepared.graph.declaration().connections.len(), 1);
    assert_eq!(
        prepared.graph.bindings()[&prepared.graph.declaration().connections[0].id].len(),
        2
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .rows,
        rows.into_iter()
            .chain(std::iter::once(
                ModelingOutput::ConditionalBoundary(symbol("boundary")).row_id()
            ))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        program
            .conditional
            .as_ref()
            .unwrap()
            .view
            .boundary
            .members
            .columns,
        unknowns.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(program.program.assembly.order(), DerivativeOrder::Value);
    let controls = Controls::default();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &controls,
    );
    let budget = crate::math::WorkerBudget::new(16 << 20);
    let worker = runtime
        .native()
        .worker(
            program.program.clone(),
            &prepared.providers,
            execution.scope().unwrap(),
            &budget,
        )
        .unwrap();
    let mut unit = UnitWorker {
        last_values: None,
        input_ids: program.inputs.iter().map(|i| i.0).collect(),
        output_ids: program.outputs.iter().map(|o| o.0).collect(),
        program,
        worker,
        service: runtime.native().clone(),
        providers: prepared.providers,
        budget,
    };
    let charged = unit.budget.used();
    for (input, expected) in [(4.0, 4.0), (8.0, 6.0), (4.0, 4.0)] {
        let outputs = unit
            .evaluate(
                &BTreeMap::from([(inlet, input), (auxiliary, 2.0)]),
                &execution,
            )
            .unwrap();
        assert!((outputs[&outlet] - expected).abs() < 1e-7);
        assert!((outputs[&auxiliary_outlet] - 5.0).abs() < 1e-7);
        assert_eq!(unit.program.values.identity(), original);
        assert_eq!(unit.budget.used(), charged);
    }
    assert!(unit.evaluate(&BTreeMap::new(), &execution).is_err());
    assert!(
        unit.evaluate(
            &BTreeMap::from([(inlet, f64::NAN), (auxiliary, 2.0)]),
            &execution
        )
        .is_err()
    );
    execution
        .cancel
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, 4.0)]), &execution)
            .is_err()
    );
    assert_eq!(unit.program.values.identity(), original);
    assert_eq!(unit.budget.used(), charged);

    let mut explicit = request.clone();
    explicit.units[0].realization = CausalUnitRealization::ExplicitMap;
    assert!(
        package
            .prepare_recycle(&analysis, selection.clone(), explicit, &cancel)
            .await
            .is_err()
    );
    let mut invalid = request;
    if let CausalUnitRealization::Conditional { solver, .. } = &mut invalid.units[0].realization {
        solver.backend = Some(Backend::Highs);
    }
    assert!(
        package
            .prepare_recycle(&analysis, selection, invalid, &cancel)
            .await
            .is_err()
    );
}

#[test]
fn conditional_unit_request_has_one_declared_realization_and_canonical_settings_defaults() {
    let request = CausalUnitRequest {
        node: id(1),
        inputs: BTreeSet::from([id(2)]),
        outputs: BTreeSet::from([id(3)]),
        realization: CausalUnitRealization::Conditional {
            residuals: BTreeSet::from([id(4)]),
            unknowns: BTreeSet::from([id(5)]),
            solver: Box::new(crate::math::settings::SolveSettings {
                intent: SolveIntent::Root,
                ..Default::default()
            }),
        },
    };
    let value = serde_json::to_value(&request).unwrap();
    let decoded: CausalUnitRequest = serde_json::from_value(value.clone()).unwrap();
    let CausalUnitRealization::Conditional { solver, .. } = decoded.realization else {
        panic!("changed realization")
    };
    let full = solver.as_ref().clone().profile().unwrap();
    let mut omitted = serde_json::to_value(solver.as_ref()).unwrap();
    omitted
        .as_object_mut()
        .unwrap()
        .retain(|field, _| matches!(field.as_str(), "version" | "intent"));
    let omitted: crate::math::settings::SolveSettings = serde_json::from_value(omitted).unwrap();
    assert_eq!(
        crate::math::solves::profile_key(&full).unwrap(),
        crate::math::solves::profile_key(&omitted.profile().unwrap()).unwrap()
    );
    let mut conflict = value;
    conflict["realization"]["fallback"] = serde_json::json!("simultaneous");
    assert!(serde_json::from_value::<CausalUnitRequest>(conflict).is_err());
}

#[test]
fn conditional_unit_admission_diagnostic_retains_sources_and_original_cause() {
    use std::error::Error;
    let unit = CausalUnitRequest {
        node: id(1),
        inputs: BTreeSet::from([id(2)]),
        outputs: BTreeSet::from([id(3)]),
        realization: CausalUnitRealization::Conditional {
            residuals: BTreeSet::from([id(4)]),
            unknowns: BTreeSet::from([id(5)]),
            solver: Default::default(),
        },
    };
    let request = RecycleRequest {
        tears: BTreeSet::new(),
        units: vec![unit.clone()],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let cause = MathRuntimeError::Compile(pse_compiler::workspace::CompileError::Missing(
        "external row couples selected unknown".into(),
    ));
    let error = conditional_admission(&unit, &request, cause);
    let diagnostic = error.boundary_diagnostic();
    assert_eq!(
        diagnostic.class,
        pse_model::diagnostic::BoundaryClass::InvalidModel
    );
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionInvalidModel
    );
    assert_eq!(diagnostic.sources, vec![id(1), id(2), id(3), id(4), id(5)]);
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("external row couples")
    );
    let encoded = serde_json::to_vec(&diagnostic).unwrap();
    let decoded: pse_model::diagnostic::BoundaryDiagnostic =
        serde_json::from_slice(&encoded).unwrap();
    let unavailable = conditional_admission(
        &unit,
        &request,
        MathRuntimeError::Compile(
            pse_compiler::workspace::CompileError::ConditionalUnavailable(
                "external row couples selected unknown".into(),
            ),
        ),
    );
    assert_eq!(
        unavailable.boundary_diagnostic().class,
        pse_model::diagnostic::BoundaryClass::Unsupported
    );
    assert_eq!(
        unavailable.boundary_diagnostic().rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionUnsupported
    );
    assert_eq!(
        unavailable.boundary_diagnostic().sources,
        diagnostic.sources
    );
    let numerical = conditional_admission(
        &unit,
        &request,
        native::ProblemError::numerical("native iteration stagnated").into(),
    );
    let numerical_diagnostic = numerical.boundary_diagnostic();
    assert_eq!(
        numerical_diagnostic.class,
        pse_model::diagnostic::BoundaryClass::Numerical
    );
    assert_eq!(
        numerical_diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionNumerical
    );
    assert!(
        numerical
            .source()
            .unwrap()
            .to_string()
            .contains("stagnated")
    );
    let nonfinite = conditional_admission(
        &unit,
        &request,
        pse_math::MathError::Evaluation {
            source_id: id(6),
            order: DerivativeOrder::Value,
            detail: "original boundary observation was nonfinite".into(),
        }
        .into(),
    );
    let observed = nonfinite.boundary_diagnostic();
    // The expression owner classifies this actual evaluation failure as a rejected
    // trial. Preserve that classification; an admission wrapper must not relabel it.
    assert_eq!(
        observed.class,
        pse_model::diagnostic::BoundaryClass::TrialRejected
    );
    assert_eq!(
        observed.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionTrialRejected
    );
    assert!(observed.sources.contains(&id(6)));
    assert!(
        nonfinite
            .source()
            .unwrap()
            .to_string()
            .contains("nonfinite")
    );
    assert_eq!(decoded.sources, diagnostic.sources);
}

#[tokio::test]
async fn conditional_unit_affine_boundary_retains_difference_magnitudes_and_point_outputs() {
    use pse_model::generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget};
    let runtime = runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Root {var x:Temperature; var y:Temperature; eq local:y==x; let boundary:Temperature=x; state incoming supplied(true) {coordinate value=boundary; transport value=boundary tolerance 1e-7{K};} state outgoing supplied(false) {coordinate value=y; transport value=y tolerance 1e-7{K};} state_port inlet=incoming; state_port outlet=outgoing; annotation connectivity inlet(1,0); annotation connectivity outlet(0,1); connect outlet -> inlet; annotation start x(300{K}); annotation start y(300{K});}}",
        id(86), pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let root = declarations
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, physical())
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let mut analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let prepared_model = package
        .prepare(
            root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let model = &prepared_model.compiled().model;
    let symbol = |name: &str| {
        model
            .symbols
            .values()
            .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let port = |name: &str| {
        model
            .material_ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
    };
    let incoming = port("inlet");
    let outgoing = port("outlet");
    let inlet = *incoming.coordinates.values().next().unwrap();
    let outlet = *outgoing.coordinates.values().next().unwrap();
    let boundary = symbol("boundary");
    let point = pse_quantity::standard::ids::quantity("temperature.point");
    let difference = pse_quantity::standard::ids::quantity("temperature.difference");
    analysis
        .numerical
        .targets
        .push(pse_math::numerics::TargetSpec {
            id: boundary,
            kind: NumericalTarget::Observable,
            quantity: point,
            unit: pse_quantity::standard::ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        });
    analysis
        .numerical
        .declarations
        .push(pse_math::numerics::SourcedRequirement {
            source: NumericalSource::Model,
            declaration: pse_model::numerics::NumericalRequirement {
                requirement_id: id(87),
                model_id: Some(root.as_id().into()),
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: boundary,
                target_kind: NumericalTarget::Observable,
                nominal: Some(8.0),
                scaling_factor: Some(0.125),
                absolute_tolerance: Some(0.08),
                shared_engineering_allowance: None,
                engineering_rule_id: None,
                relative_tolerance: Some(0.01),
                unit_id: Some(pse_quantity::standard::ids::unit("degF").as_id()),
                coordinates: NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "explicit point-observation magnitudes in Fahrenheit".into(),
            },
        });
    let selection = ModelingFlowSelection {
        nodes: BTreeSet::from([analysis.instance]),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let request = RecycleRequest {
        tears: selection.connections.keys().copied().collect(),
        units: vec![CausalUnitRequest {
            node: analysis.instance.as_id(),
            inputs: BTreeSet::from([incoming.id]),
            outputs: BTreeSet::from([outgoing.id]),
            realization: CausalUnitRealization::Conditional {
                residuals: model
                    .equations
                    .iter()
                    .filter(|r| r.lineage.path.ends_with(".local"))
                    .map(|r| r.id)
                    .collect(),
                unknowns: BTreeSet::from([symbol("x"), symbol("y")]),
                solver: Box::new(crate::math::settings::SolveSettings {
                    intent: SolveIntent::Root,
                    ..Default::default()
                }),
            },
        }],
        anderson: 0,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let prepared = package
        .prepare_recycle(&analysis, selection, request, &cancel)
        .await
        .unwrap();
    let program = prepared.programs[0].clone();
    let conditional = program.conditional.as_ref().unwrap();
    let boundary_row = ModelingOutput::ConditionalBoundary(boundary).row_id();
    let row = conditional
        .view
        .plan
        .structure()
        .rows()
        .iter()
        .find(|r| r.id == boundary_row)
        .unwrap();
    assert_eq!(row.quantity, difference);
    assert_eq!((row.lower, row.upper), (0.0, 0.0));
    assert_eq!(
        conditional
            .view
            .plan
            .structure()
            .parameters()
            .iter()
            .find(|p| p.id == pse_ids::named_id(boundary, "conditional-boundary-value"))
            .unwrap()
            .quantity,
        point
    );
    let (scale, tolerance) = conditional.row_magnitudes(boundary_row).unwrap();
    assert!((scale - 8.0 * 5.0 / 9.0).abs() < 1e-12);
    assert!((tolerance - (0.08 + 0.01 * 8.0) * 5.0 / 9.0).abs() < 1e-12);
    assert_eq!(
        program
            .program
            .assembly
            .structure()
            .rows()
            .iter()
            .find(|r| r.id == ModelingOutput::Member(symbol("y")).row_id())
            .unwrap()
            .quantity,
        point
    );
    let original = program.values.identity();
    let execution = Execution::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &Controls::default(),
    );
    let budget = crate::math::WorkerBudget::new(16 << 20);
    let worker = runtime
        .native()
        .worker(
            program.program.clone(),
            &prepared.providers,
            execution.scope().unwrap(),
            &budget,
        )
        .unwrap();
    let mut unit = UnitWorker {
        last_values: None,
        input_ids: program.inputs.iter().map(|i| i.0).collect(),
        output_ids: program.outputs.iter().map(|o| o.0).collect(),
        program,
        worker,
        service: runtime.native().clone(),
        providers: prepared.providers,
        budget,
    };
    let charged = unit.budget.used();
    for target in [310.0, 320.0, 310.0] {
        let outputs = unit
            .evaluate(&BTreeMap::from([(inlet, target)]), &execution)
            .unwrap();
        assert!((outputs[&outlet] - target).abs() < 1e-7);
        assert_eq!(unit.program.values.identity(), original);
        assert_eq!(unit.budget.used(), charged);
    }
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, f64::NAN)]), &execution)
            .is_err()
    );
    assert!(unit.evaluate(&BTreeMap::new(), &execution).is_err());
    execution
        .cancel
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(
        unit.evaluate(&BTreeMap::from([(inlet, 310.0)]), &execution)
            .is_err()
    );
    assert_eq!(unit.program.values.identity(), original);
    assert_eq!(unit.budget.used(), charged);
}
