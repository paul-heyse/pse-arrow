// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::math::solves::{BackendSettings, SolverProfile};
use pse_backend_native::solve::*;
use pse_ids::SemanticId;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
pub(super) fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
pub(super) fn runtime() -> Runtime {
    runtime_with_workspace(16 << 20)
}
pub(super) fn runtime_with_workspace(workspace_bytes: usize) -> Runtime {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let shared = SharedRuntime::build(crate::ResourceBudget {
        memory_limit_bytes: n(512 << 20),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: crate::DeltaCacheBudget::disabled(1024),
        math: crate::math::MathPolicy {
            worker_bytes: 8 << 20,
            workspace_bytes,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    pse_engine::validation::bind_defaults(&registry).unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    Runtime::from_shared(shared, registry, sessions)
}
pub(super) fn physical() -> PhysicalContext {
    // Fixture only: production requires source-backed PhysicalInventory admission.
    let quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
    let preconditions = Arc::new(pse_quantity::PhysicalPreconditions::new(vec![]).unwrap());
    PhysicalContext {
        key: pse_compiler::workspace::physical_identity(&quantities, &preconditions),
        quantities,
        preconditions,
        sources: BTreeMap::new(),
        origin: "test_fixture",
        _inventory: None,
    }
}
pub(super) fn declaration() -> ModelDeclaration {
    let context = physical();
    let q = pse_quantity::standard::ids::quantity("neutral");
    let qid = q.as_id();
    let unit = context
        .quantities
        .quantity_type(q)
        .unwrap()
        .canonical_unit
        .as_id();
    serde_json::from_value(serde_json::json!({
        "model_id":id(20),"name":"constant","domains":[],"groups":[],
        "definitions":[{"definition_id":id(2),"sources":["x*x"],"formals":[{"path":"x","quantity_id":qid}],"domains":[],"groups":[],"providers":[],"units":[],"literals":[]}],
        "cases":[{"case_id":id(5),"name":"selected","variables":[{"port":{"symbol_id":id(1),"quantity_id":qid,"unit_id":unit},"fixed":true,"domain":"continuous","lower":null,"upper":null}],"parameters":[],
            "instances":[{"instance_id":id(3),"definition_id":id(2),"slots":[{"source_id":id(1),"formal_quantity_id":qid,"formal_unit_id":unit}],"contributions":[{"output":0,"row_id":id(4),"scale":1.0}]}],
            "rows":[{"row_id":id(4),"quantity_id":qid,"lower":4.0,"upper":4.0}],"objective":null,"values":[{"symbol_id":id(1),"value":2.0}]}]
    })).unwrap()
}
fn profile() -> SolverProfile {
    SolverProfile {
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
        intent: SolveIntent::Root,
        selection: SolverSelection::Auto,
        controls: Controls::default(),
        backend: BackendSettings::Default,
    }
}
pub(super) fn compiler_profile() -> pse_compiler::workspace::Profile {
    pse_compiler::workspace::Profile {
        evaluation: pse_math::jets::EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        },
        ..Default::default()
    }
}
#[tokio::test]
async fn immutable_revisions_atomic_edit_and_repeatable_owned_completion() {
    let rt = runtime();
    let revision = ModelBuilder::from_declaration(rt.clone(), declaration(), physical())
        .freeze()
        .unwrap();
    let old = revision.identity();
    let mut invalid = revision.edit();
    invalid
        .declaration_mut()
        .cases
        .push(declaration().cases.remove(0));
    assert!(invalid.freeze().is_err());
    assert_eq!(revision.identity(), old);
    let mut edit = revision.edit();
    edit.declaration_mut().cases[0].values[0].value = 3.0;
    let changed = edit.freeze().unwrap();
    assert_ne!(changed.identity(), old);
    let a = revision
        .prepare(
            id(5),
            profile(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let b = changed
        .prepare(
            id(5),
            profile(),
            compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_ne!(a.compiled().presolve.key, b.compiled().presolve.key);
    assert!(rt.start(vec![a.clone(), b], true).is_err());
    let job = a.start().unwrap();
    let result = job.wait().await.unwrap();
    let again = job.wait().await.unwrap();
    assert!(Arc::ptr_eq(&result, &again));
    let batch = result.table("runtime.solve_constraints").unwrap();
    use pse_relations::generated::runtime::solve_constraints as wire;
    let row = wire::View::from_checked(&batch).unwrap().row(0).unwrap();
    assert_eq!(row.value, Some(4.0));
    assert_eq!(row.equality_residual, Some(0.0));
    assert_eq!(row.lower_violation, Some(0.0));
    let attempt = result
        .prepare_publication(
            url::Url::parse("file:///tmp/pse-workflow-unit/").unwrap(),
            id(50),
            None,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
    assert_ne!(attempt.attempt_id, attempt.publication_id);
    drop(attempt);
    drop(result);
    drop(again);
    drop(job);
    drop(a);
    drop(changed);
    drop(revision);
    let array = batch.batch().column_by_name("value").unwrap().clone();
    drop(batch);
    assert!(rt.shared.pool().reserved() > 0);
    let values = array
        .as_any()
        .downcast_ref::<datafusion::arrow::array::Float64Array>()
        .unwrap();
    assert_eq!(values.value(0), 4.0);
    let retained = rt.shared.pool().reserved();
    drop(array);
    assert!(rt.shared.pool().reserved() < retained);
}
#[test]
fn typed_and_document_declarations_have_identical_admission() {
    let rt = runtime();
    let row = declaration();
    let physical = physical();
    let typed = ModelBuilder::from_declaration(rt.clone(), row.clone(), physical.clone())
        .freeze()
        .unwrap();
    let texts = BTreeMap::from([
        (
            "package.toml".into(),
            include_str!("../../../../tests/fixtures/packages/minimal_explicit/package.toml")
                .into(),
        ),
        (
            "computation_models/model.yaml".into(),
            serde_json::to_string(&serde_json::json!({"computation_models":[row]})).unwrap(),
        ),
    ]);
    let cancel = pse_columnar::CancellationToken::new();
    let pool = rt.shared.pool();
    let bundle = crate::authoring_driver::document::load_package_texts_owned(
        &texts,
        &rt.registry,
        pse_authoring::ParseBudget::default(),
        &pool,
        &cancel,
    )
    .unwrap();
    let documents = crate::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        vec![bundle],
        &pool,
        &cancel,
    )
    .unwrap();
    let document = rt
        .models_from_documents(&documents, physical)
        .unwrap()
        .remove(0)
        .freeze()
        .unwrap();
    assert_eq!(typed.identity(), document.identity());
    assert_eq!(typed.declaration(), document.declaration());
}

#[tokio::test]
async fn numerical_policy_and_closure_assessment_survive_public_result_encoding() {
    use pse_model::generated::enums::{CandidateUse, ClosurePolicy};
    let mut model = declaration();
    model.cases[0].rows.clear();
    model.cases[0].instances[0].contributions.clear();
    let mut builder = ModelBuilder::from_declaration(runtime(), model, physical());
    builder.balance(serde_json::from_value(serde_json::json!({"model_id":id(20),"case_id":id(5),"balance_id":id(4),"quantity_id":pse_quantity::standard::ids::quantity("neutral").as_id(),"accumulation":null,"tolerance":1e-5,"integral_tolerance":null,"provenance":"independent original physical closure","impulses":[],"terms":[{"source_id":id(30),"instance_id":id(3),"output":0,"role":"outlet","multiplier":1.0,"mode":null,"transfer_id":null}]})).unwrap());
    builder.numerical_requirement(serde_json::from_value(serde_json::json!({"requirement_id":id(31),"model_id":id(20),"case_id":id(5),"target_id":id(4),"target_kind":"row","nominal":null,"scaling_factor":null,"absolute_tolerance":10.0,"relative_tolerance":0.0,"unit_id":null,"coordinates":"physical","priority":0,"required":true,"provenance":"deliberately loose numerical feasibility"})).unwrap());
    let revision = builder.freeze().unwrap();
    for (policy, expected) in [
        (ClosurePolicy::RequireClosed, CandidateUse::Unusable),
        (
            ClosurePolicy::AllowUnclosed,
            CandidateUse::QualifiedUnclosed,
        ),
    ] {
        let mut settings = profile();
        settings.numerics.closure = policy;
        let result = revision
            .prepare(
                id(5),
                settings,
                compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert_eq!(result.assessments()[0].numerically_feasible, Some(true));
        assert_eq!(result.assessments()[0].usability, expected);
        assert_eq!(result.usable(), policy == ClosurePolicy::AllowUnclosed);
        let table = result.table("runtime.candidate_assessments").unwrap();
        let row =
            pse_relations::generated::runtime::candidate_assessments::View::from_checked(&table)
                .unwrap()
                .row(0)
                .unwrap();
        assert_eq!(row.usability, expected);
        let physical = result.table("runtime.physical_checks").unwrap();
        let row = pse_relations::generated::runtime::physical_checks::View::from_checked(&physical)
            .unwrap()
            .row(0)
            .unwrap();
        assert_eq!(row.closure, Some(-4.0));
        assert_eq!(row.accepted, Some(false));
        assert!(
            result
                .table("runtime.resolved_numerics")
                .unwrap()
                .batch()
                .num_rows()
                > 0
        );
        assert_eq!(result.assessments()[0].usability, expected);
    }
}
