// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demand-order controls use one source revision and the public math service.
use super::*;
use pse_compiler::workspace::{CompilerContext, Profile};
use pse_kernels::DerivativeOrder;
use pse_model::diagnostic::DiagnosticProjection;
use pse_runtime::math::{ExecutableCase, MathService, Workspace, modeling::ModelingRevision};
use std::sync::Arc;

pub(super) const SOURCE: &str = "package k4 { def Root { var x:Scalar; port outlet:Scalar=x; let selected:Scalar=x*x; let unrelated:Scalar=x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x*x; eq balance:x*x==4; } }";

pub(super) fn definition(
    owner: &WorkflowRuntime,
) -> (
    Workspace,
    ModelingRevision,
    pse_modeling::DeclarationId,
    CompilerContext,
) {
    let context = CompilerContext {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(pse_quantity::PhysicalPreconditions::new(vec![]).unwrap()),
        providers: BTreeMap::new(),
    };
    let service = owner.runtime.math();
    let workspace = service
        .workspace(context.clone(), Default::default())
        .unwrap();
    let rows = pse_authoring::language::parse(
        SOURCE,
        pse_ids::SemanticId::from_bytes([81; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let physical =
        pse_compiler::workspace::physical_identity(&context.quantities, &context.preconditions);
    let revision = service
        .modeling_revision(
            &workspace,
            rows,
            Default::default(),
            Default::default(),
            &physical,
        )
        .unwrap();
    (workspace, revision, root, context)
}

/// Inventory only payloads simultaneously retained by the escaped executable owners.
/// Every shared component is deduplicated by its individual immutable allocation identity.
async fn inventory(service: &Arc<MathService>, executable: Arc<ExecutableCase>) -> Vec<Value> {
    service.with_worker(executable, BTreeMap::new(), &CancelSource::new(), |worker| {
        let assembly = worker.assembly();
        let mut entries = Vec::new();
        for body in assembly.bodies().values() {
            entries.push(json!({"kind":"body","identity":[body.allocation_identity()],"known_payload_estimate_bytes":body.retained_bytes()}));
            entries.push(json!({"kind":"body_wrapper","identity":[Arc::as_ptr(body) as usize],"known_payload_estimate_bytes":size_of_val(body.as_ref())}));
        }
        let (identity,bytes) = assembly.plan_wrapper_allocation();
        entries.push(json!({"kind":"plan_wrapper","identity":[identity],"known_payload_estimate_bytes":bytes}));
        for (component,identity,bytes) in assembly.allocation_components() {
            entries.push(json!({"kind":format!("plan_{component}"),"identity":[identity],"known_payload_estimate_bytes":bytes}));
        }
        for support in assembly.supports() {
            entries.push(json!({"kind":"support_wrapper","identity":[Arc::as_ptr(support) as usize],"known_payload_estimate_bytes":size_of_val(support.as_ref())}));
        }
        for support in assembly.supports().iter().map(|s| s.as_ref()).chain(assembly.programs().iter().map(|p| p.prepared_support())) {
            entries.push(json!({"kind":"support","identity":[support.allocation_identity()],"known_payload_estimate_bytes":support.retained_bytes(),"order":format!("{:?}",support.order())}));
        }
        for program in assembly.programs() {
            entries.push(json!({"kind":"compiled_artifact_wrapper","identity":[Arc::as_ptr(program) as usize],"known_payload_estimate_bytes":size_of_val(program.as_ref())}));
            for (component,identity,bytes) in program.allocation_components() {
                entries.push(json!({"kind":format!("compiled_{component}"),"identity":[identity],"known_payload_estimate_bytes":bytes,"order":format!("{:?}",program.compiled_order())}));
            }
        }
        Ok(entries)
    }).await.unwrap()
}

fn counters(phases: &phases::Phases, product: &str, order: &str, field: &str) -> u64 {
    phases.constructions()[format!("{product}.{order}")][field]
        .as_u64()
        .unwrap_or(0)
}
fn artifact_counts(service: &MathService) -> (usize, usize) {
    let report = service
        .report()
        .into_iter()
        .find(|r| r.name == "pse.cache.math_artifacts")
        .unwrap();
    (report.hits, report.misses)
}

pub(super) async fn run(owner: &WorkflowRuntime, phases: &phases::Phases) -> (Duration, Value) {
    let (workspace, revision, root, context) = definition(owner);
    let service = owner.runtime.math();
    let cancel = CancelSource::new();
    let instance = pse_modeling::specialize::root_instance(root);
    let source = pse_ids::encoding_checksum(SOURCE.as_bytes()).content_hash();
    let pool = owner.runtime.pool();
    let baseline = pool.reserved();
    let mut stages = Vec::new();
    let mut retained = Vec::new();
    let mut live = BTreeMap::<String, Value>::new();
    let started = Instant::now();
    phases.reset();
    let phase = Instant::now();
    let semantic = service
        .prepare_semantic_modeling_revision(
            workspace.clone(),
            revision.clone(),
            root,
            instance,
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let selection = pse_compiler::workspace::ModelingFlowSelection {
        nodes: semantic
            .model
            .ports
            .values()
            .map(|p| p.lineage.instance)
            .collect(),
        connections: BTreeMap::new(),
    };
    let flow = service
        .prepare_modeling_flow(
            semantic.clone(),
            context.quantities.clone(),
            selection,
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(flow.graph().declaration().nodes.len(), 1);
    assert!(
        phases.constructions().as_object().unwrap().is_empty(),
        "topology constructs no body, support or evaluator"
    );
    stages.push(json!({"stage":"topology","outcome":"prepared","seconds":{"total":phase.elapsed().as_secs_f64()},"constructions":phases.constructions(),"pool_reserved_bytes":pool.reserved()}));
    phases.reset();
    let phase = Instant::now();
    let model = service
        .prepare_modeling_revision(
            workspace.clone(),
            revision.clone(),
            root,
            instance,
            pse_modeling::Bindings {
                demand: vec!["x".into(), "selected".into()],
                ..Default::default()
            },
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert!(!model.compiled().model.paths.contains_key("unrelated"));
    assert!(
        model
            .compiled()
            .model
            .symbols
            .values()
            .all(|symbol| !symbol.lineage.path.ends_with(".unrelated"))
    );
    assert!(counters(phases, "body", "value", "successes") > 0);
    assert_eq!(
        counters(phases, "compiled_artifact", "value", "attempts"),
        0,
        "body admission constructs no evaluator"
    );
    let x = model.compiled().model.paths["x"];
    let selected = model.compiled().model.paths["selected"];
    let equation = model.compiled().model.equations.first().unwrap().id;
    assert!(
        model
            .compiled()
            .admitted
            .case()
            .rows()
            .iter()
            .any(|row| row.id == equation),
        "original equation remains mandatory"
    );
    let row = pse_compiler::workspace::ModelingOutput::Member(selected).row_id();
    let values = model
        .compiled()
        .case_values(&pse_compiler::workspace::ModelingCaseBindings {
            values: BTreeMap::from([("x".into(), 2.0)]),
            ..Default::default()
        })
        .unwrap();
    stages.push(json!({"stage":"selected-body-admission","outcome":"prepared","seconds":{"total":phase.elapsed().as_secs_f64()},"constructions":phases.constructions(),"unrelated_observation_admitted":false,"pool_reserved_bytes":pool.reserved()}));
    for (name, order) in [
        ("value", DerivativeOrder::Value),
        ("first", DerivativeOrder::First),
        ("second-refused", DerivativeOrder::Second),
        ("second", DerivativeOrder::Second),
        ("warm-first", DerivativeOrder::First),
    ] {
        phases.reset();
        let phase = Instant::now();
        let mut profile = Profile::default();
        if name == "second-refused" {
            profile.evaluation.derivative_components = 2;
        }
        let before = artifact_counts(service);
        let executable = service
            .prepare_modeling_functions(
                workspace.clone(),
                model.clone(),
                vec![row],
                vec![x],
                order,
                profile,
                &cancel,
            )
            .await;
        let after = artifact_counts(service);
        let seconds = phase.elapsed().as_secs_f64();
        assert_eq!(
            counters(phases, "body", "value", "attempts"),
            0,
            "orders share admitted arithmetic"
        );
        if name == "second-refused" {
            let error = executable.unwrap_err();
            assert_eq!(
                error
                    .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
                    .class,
                pse_model::diagnostic::BoundaryClass::ResourceLimit
            );
            assert!(counters(phases, "compiled_artifact", "second", "attempts") > 0);
            assert_eq!(
                counters(phases, "compiled_artifact", "second", "successes"),
                0
            );
            for (index, weaker) in retained.iter().enumerate() {
                let values = values.clone();
                service
                    .with_worker(
                        Arc::clone(weaker),
                        BTreeMap::new(),
                        &cancel,
                        move |worker| {
                            assert_eq!(worker.constraints(&values)?, vec![4.0]);
                            if index == 1 {
                                assert_eq!(*worker.jacobian(&values)?.get(0, 0).unwrap(), 4.0);
                            }
                            Ok(())
                        },
                    )
                    .await
                    .unwrap();
            }
            stages.push(json!({"stage":name,"outcome":"refused","seconds":{"total":seconds},"allowance":{"derivative_components":2},"refusal":error.to_string(),"constructions":phases.constructions(),"artifact_requests":{"reuses":after.0-before.0,"misses":after.1-before.1},"pool_reserved_bytes":pool.reserved()}));
            continue;
        }
        let executable = executable.unwrap();
        if order == DerivativeOrder::Value {
            assert_eq!(counters(phases, "support", "first", "attempts"), 0);
            assert_eq!(counters(phases, "support", "second", "attempts"), 0);
        }
        if order == DerivativeOrder::First {
            assert_eq!(
                counters(phases, "support", "second", "attempts"),
                0,
                "First demand constructs no stronger support"
            );
        }
        if name == "second" {
            assert!(counters(phases, "support", "second", "successes") > 0);
            assert!(counters(phases, "compiled_order", "second", "successes") > 0);
        }
        if name == "warm-first" {
            assert_eq!(
                counters(phases, "compiled_artifact", "first", "attempts"),
                0
            );
            assert!(
                after.0 > before.0,
                "warm demand reuses an actual retained artifact"
            );
        }
        let constructions = phases.constructions();
        let entries = inventory(service, executable.clone()).await;
        for entry in entries {
            let key = format!("{}:{}", entry["kind"], entry["identity"]);
            live.entry(key).or_insert(entry);
        }
        let values = values.clone();
        service
            .with_worker(
                executable.clone(),
                BTreeMap::new(),
                &cancel,
                move |worker| {
                    assert_eq!(worker.constraints(&values)?, vec![4.0]);
                    if order >= DerivativeOrder::First {
                        assert_eq!(*worker.jacobian(&values)?.get(0, 0).unwrap(), 4.0);
                    }
                    if order >= DerivativeOrder::Second {
                        assert_eq!(
                            *worker.hessian(&values, 0.0, &[1.0])?.get(0, 0).unwrap(),
                            2.0
                        );
                    }
                    Ok(())
                },
            )
            .await
            .unwrap();
        retained.push(executable);
        stages.push(json!({"stage":name,"outcome":"prepared","seconds":{"total":seconds},"constructions":constructions,"artifact_requests":{"reuses":after.0-before.0,"misses":after.1-before.1},"unique_live_allocation_count":live.len(),"known_unique_payload_estimate_bytes":live.values().map(|e| e["known_payload_estimate_bytes"].as_u64().unwrap()).sum::<u64>(),"pool_reserved_bytes":pool.reserved()}));
    }
    // Explicit selected observation uses exactly one row; the expensive unrelated let
    // remains authored but absent from the admitted specialization and evaluator closure.
    phases.reset();
    let phase = Instant::now();
    let selected_row = pse_compiler::workspace::ModelingOutput::Member(selected).row_id();
    let observation = service
        .prepare_modeling_observations(
            workspace.clone(),
            model.clone(),
            BTreeSet::from([selected_row]),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let seconds = phase.elapsed().as_secs_f64();
    let constructions = phases.constructions();
    assert_eq!(counters(phases, "support", "first", "attempts"), 0);
    assert_eq!(counters(phases, "support", "second", "attempts"), 0);
    for entry in inventory(service, observation.clone()).await {
        live.entry(format!("{}:{}", entry["kind"], entry["identity"]))
            .or_insert(entry);
    }
    let observation_values = values.clone();
    service
        .with_worker(
            observation.clone(),
            BTreeMap::new(),
            &cancel,
            move |worker| {
                assert_eq!(worker.assembly().structure().rows().len(), 1);
                assert_eq!(worker.constraints(&observation_values)?, vec![4.0]);
                Ok(())
            },
        )
        .await
        .unwrap();
    retained.push(observation);
    stages.push(json!({"stage":"selected-observation","outcome":"prepared","seconds":{"total":seconds},"constructions":constructions,"selected_rows":1,"unrelated_observation_admitted":false,"pool_reserved_bytes":pool.reserved()}));
    let retained_executable_owners = retained.len();
    let snapshot = live.values().cloned().collect::<Vec<_>>();
    service.clear_program_cache();
    let escaped_pool = pool.reserved();
    drop(retained);
    drop(model);
    drop(flow);
    drop(semantic);
    drop(revision);
    drop(workspace);
    let released_pool = pool.reserved();
    assert!(
        released_pool < baseline,
        "workspace and escaped products release their leases"
    );
    let elapsed = started.elapsed();
    (
        elapsed,
        json!({"source_content_hash":source,"physical_context_hash":pse_compiler::workspace::physical_identity(&context.quantities,&context.preconditions),"source_revision_scope":"one immutable parsed authored revision and admitted standard quantity/precondition context shared by every demand","selected_observation_orders":["value","first","second"],"mandatory_original_equation":equation,"stages":stages,"usable_weaker_products_checked_after_refusal":2,"retained_executable_owners":retained_executable_owners,"unique_live_allocations":snapshot,"unique_live_allocation_count":snapshot.len(),"known_unique_payload_estimate_bytes":snapshot.iter().map(|e|e["known_payload_estimate_bytes"].as_u64().unwrap()).sum::<u64>(),"allocation_scope":"individual simultaneously retained body/support/plan/evaluator component and wrapper identities; known payload estimates include map-node estimates and exclude allocator and opaque library overhead; pool separately reports authoritative owned leases","baseline_with_workspace_bytes":baseline,"escaped_after_retention_clear_bytes":escaped_pool,"after_product_release_bytes":released_pool,"cache_reports":service.report()}),
    )
}
