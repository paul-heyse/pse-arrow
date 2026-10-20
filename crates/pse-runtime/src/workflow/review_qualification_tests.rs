// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compose original workflow controls through descriptive review and relocated publication.
use super::*;
use crate::{frozen_inspection::FrozenEvidenceSet, review_analysis, review_pack};
use pse_columnar::CancellationToken;
use pse_model::generated::identities::RunId;

/// The caller retains the original scientific assertions. This adds passive transport
/// qualification over that very result, without launching a replacement solve.
pub(in crate::workflow) async fn assert_review_transport(
    result: &Arc<RunResult>,
    required_facts: &[(&str, &str)],
) {
    let before = serde_json::to_value(&*result.invocation_evidence().unwrap()).unwrap();
    let ((), work) = crate::math::counted(async {
        let context = result.review_context(&ReviewContextRequest::default()).unwrap();
        let document = context.document();
        assert_eq!(document.run_id, Some(result.run_id));
        for (path, codec) in required_facts {
            assert!(document.facts.iter().any(|fact| fact.path == *path && fact.codec == *codec),
                "missing original {path} ({codec}); unavailable={:?}", document.unavailable);
        }
        // Challenge the derived context against the retained performing owners,
        // rather than treating the exporter's own member list as the oracle.
        let exact_fact = |path: &str, expected: Vec<u8>| {
            assert_eq!(document.facts.iter().find(|fact| fact.path == path).unwrap().payload, expected);
        };
        match result.replay_request() {
            RunRequest::Modeling(steps) => {
                for (index, step) in steps.iter().enumerate() {
                    exact_fact(&format!("/steps/{index}/runtime"), serde_json::to_vec(step.solve.execution_snapshot()).unwrap());
                    exact_fact(&format!("/steps/{index}/physical_values"), serde_json::to_vec(&step.model.values.scalars.iter().map(|(id, value)| (*id, value.to_bits())).collect::<std::collections::BTreeMap<_, _>>()).unwrap());
                }
            }
            RunRequest::Simulation(simulation) => {
                exact_fact("/simulation/runtime", serde_json::to_vec(simulation.snapshot()).unwrap());
                exact_fact("/simulation/context", serde_json::to_vec(&simulation.review_coordinates(result.run_id).unwrap()).unwrap());
                if let Ok(RunReport::Simulation(trajectory)) = result.report() {
                    let copied: serde_json::Value = serde_json::from_slice(&document.facts.iter().find(|fact| fact.codec == "dynamic-native-statistics.v1").unwrap().payload).unwrap();
                    assert_eq!(copied["run_id"], serde_json::to_value(result.run_id).unwrap());
                    assert_eq!(copied["segments"], serde_json::to_value(&trajectory.report().statistics).unwrap());
                    let mut wrong: ReviewContextDocument = serde_json::from_slice(&serde_json::to_vec(document).unwrap()).unwrap();
                    let native = wrong.facts.iter_mut().find(|fact| fact.codec == "dynamic-native-statistics.v1").unwrap();
                    let mut payload: serde_json::Value = serde_json::from_slice(&native.payload).unwrap();
                    payload["run_id"] = serde_json::to_value(RunId::from_bytes([254; 16])).unwrap();
                    native.payload = serde_json::to_vec(&payload).unwrap();
                    assert!(wrong.validate().is_err(), "native statistics must retain their original run identity");
                    let mut restricted: ReviewContextDocument = serde_json::from_slice(&serde_json::to_vec(document).unwrap()).unwrap();
                    restricted.selection.include_native_records = false;
                    assert!(restricted.admit_handoff(&restricted.selection).is_err(), "supplied native statistics cannot bypass a handoff that excludes native records");
                }
            }
            RunRequest::Fit(fit) => {
                exact_fact("/fit/runtime", serde_json::to_vec(fit.execution_snapshot()).unwrap());
                exact_fact("/fit/context", serde_json::to_vec(&fit.scientific_context(result.run_id).unwrap()).unwrap());
            }
            RunRequest::Horizon(horizon) => {
                exact_fact("/horizon/provenance", serde_json::to_vec(&horizon.provenance).unwrap());
                exact_fact("/horizon/plant/context", serde_json::to_vec(&horizon.original.plant.review_coordinates(result.run_id).unwrap()).unwrap());
            }
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem, .. } => {
                exact_fact("/shooting/runtime", serde_json::to_vec(problem.execution_snapshot()).unwrap());
                exact_fact("/shooting/simulation/context", serde_json::to_vec(&problem.simulation.review_coordinates(result.run_id).unwrap()).unwrap());
            }
        }
        for (step, mechanism, component, record) in result.native_attempt_records() {
            exact_fact(&format!("/steps/{step}/native/{mechanism}/{component:?}"), serde_json::to_vec(record.as_ref()).unwrap());
        }
        let invocation = result.invocation_evidence().unwrap();
        let operation = invocation.operation.expect("actual workflow operation");
        assert_eq!(document.operation, Some(operation));
        let original_invocation = serde_json::to_vec(&*invocation).unwrap();
        let original_context = serde_json::to_vec(document).unwrap();
        let mut set = FrozenEvidenceSet::default();
        set.invocations.push(serde_json::from_slice(&original_invocation).unwrap());
        set.contexts.push(serde_json::from_slice(&original_context).unwrap());
        let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
        let request = review_analysis::AnalysisRequest::new(operation);
        let cancel = CancellationToken::new();
        let analysis = review_analysis::analyze(&set, &request, &pool, &cancel).unwrap();
        let root = tempfile::tempdir().unwrap();
        let original = root.path().join("pack");
        let expected_analysis = serde_json::to_vec(analysis.document()).unwrap();
        export_supplied_review_pack(
            &original, &set, analysis.document(), review_pack::PackExportRequest::default(), cancel,
        ).unwrap();
        let moved = root.path().join("relocated");
        std::fs::rename(&original, &moved).unwrap();
        // Neither neighboring context nor an apparent executable may resolve missing evidence.
        std::fs::write(moved.join("context.json"), b"conflicting live sibling").unwrap();
        let reader = review_pack::PackReader::open(&moved, 16 << 20).unwrap();
        let restored = reader.read_frozen().unwrap();
        assert_eq!(serde_json::to_vec(&restored.contexts[0]).unwrap(), original_context);
        assert_eq!(serde_json::to_vec(&restored.invocations[0]).unwrap(), original_invocation);
        assert_eq!(serde_json::to_vec(&reader.read_analysis().unwrap()).unwrap(), expected_analysis);
        let replayed = review_analysis::analyze(&restored, &request, &pool, &CancellationToken::new()).unwrap();
        assert_eq!(serde_json::to_vec(replayed.document()).unwrap(), expected_analysis);
    }).await;
    assert_eq!(
        work,
        Default::default(),
        "passive review performed mathematical preparation"
    );
    assert_eq!(
        serde_json::to_value(&*result.invocation_evidence().unwrap()).unwrap(),
        before
    );
}

/// The caller checks these inventory members against actual managed occurrence results.
#[cfg(feature = "canonical-tests")]
pub(in crate::workflow) fn assert_inventory_transport(
    inventory: ReviewInventorySelection,
    operation: pse_diagnostics::observation::OperationSnapshot,
) {
    let expected = serde_json::to_vec(&inventory).unwrap();
    let id = operation.id;
    let set = FrozenEvidenceSet {
        operations: vec![operation],
        inventories: vec![inventory],
        ..Default::default()
    };
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
    let mut request = review_analysis::AnalysisRequest::new(id);
    request.inventory_members = vec![0];
    let analysis =
        review_analysis::analyze(&set, &request, &pool, &CancellationToken::new()).unwrap();
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("pack");
    export_supplied_review_pack(
        &original,
        &set,
        analysis.document(),
        review_pack::PackExportRequest::default(),
        CancellationToken::new(),
    )
    .unwrap();
    let moved = root.path().join("relocated");
    std::fs::rename(&original, &moved).unwrap();
    let reader = review_pack::PackReader::open(&moved, 16 << 20).unwrap();
    assert_eq!(
        serde_json::to_vec(&reader.read_frozen().unwrap().inventories[0]).unwrap(),
        expected
    );
}

#[cfg(feature = "solver-kinsol")]
pub(in crate::workflow) fn assert_strategy_transport(context: &ReviewContext, runtime: &Runtime) {
    let original = serde_json::to_vec(context.document()).unwrap();
    let operation = context
        .document()
        .operation
        .expect("actual admitted strategy operation");
    let snapshot = runtime
        .observations()
        .hub()
        .reader(operation)
        .unwrap()
        .snapshot(std::time::Duration::from_secs(60));
    let set = FrozenEvidenceSet {
        operations: vec![snapshot],
        contexts: vec![serde_json::from_slice(&original).unwrap()],
        ..Default::default()
    };
    assert_supplied_transport(set, operation, &original);
}

fn assert_supplied_transport(
    set: FrozenEvidenceSet,
    operation: pse_diagnostics::observation::OperationId,
    original_context: &[u8],
) {
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
    let request = review_analysis::AnalysisRequest::new(operation);
    let analysis =
        review_analysis::analyze(&set, &request, &pool, &CancellationToken::new()).unwrap();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    export_supplied_review_pack(
        &path,
        &set,
        analysis.document(),
        review_pack::PackExportRequest::default(),
        CancellationToken::new(),
    )
    .unwrap();
    let relocated = root.path().join("relocated");
    std::fs::rename(&path, &relocated).unwrap();
    let reader = review_pack::PackReader::open(&relocated, 16 << 20).unwrap();
    assert_eq!(
        serde_json::to_vec(&reader.read_frozen().unwrap().contexts[0]).unwrap(),
        original_context
    );
}

#[tokio::test]
async fn review_qualification_actual_prehandle_failure_retains_original_request_and_source() {
    let runtime = tests::runtime();
    let rows = pse_authoring::language::parse(
        "package review { def Root { var x:Scalar; eq original:x==3; } }",
        pse_ids::SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let package = runtime
        .modeling_package(rows, tests::physical())
        .await
        .unwrap();
    let missing = tests::id(254).into();
    let analysis = ModelingAnalysis {
        root: missing,
        instance: pse_modeling::specialize::root_instance(missing),
        bindings: Default::default(),
        limits: Default::default(),
        case: Default::default(),
        order: pse_kernels::DerivativeOrder::First,
        compiler: tests::compiler_profile(),
        solver: tests::profile(),
        numerical: Default::default(),
    };
    let original_request =
        serde_json::to_vec(&serde_json::json!({"root":missing, "kind":"original_missing_case"}))
            .unwrap();
    let observed = runtime
        .observations()
        .hub()
        .start("qualification.preparation-failure");
    let operation = observed.context().id();
    let reader = observed.reader();
    let error = pse_diagnostics::observation::scope(
        Some(observed.context()),
        package.prepare_analysis(&analysis, &crate::CancelSource::new()),
    )
    .await
    .unwrap_err();
    observed.finish(pse_diagnostics::observation::Outcome::Failed);
    let ((), counts) = crate::math::counted(async {
        let context = package
            .review_context_failure(&ReviewContextRequest::default(), Some(operation), &error)
            .unwrap();
        let mut document: ReviewContextDocument =
            serde_json::from_slice(&serde_json::to_vec(context.document()).unwrap()).unwrap();
        document.facts.push(ReviewContextFact {
            path: "/submitted/request".into(),
            codec: "submitted-request.v1".into(),
            payload: original_request.clone(),
        });
        assert!(document.run_id.is_none());
        assert_eq!(
            document.failure.as_deref(),
            Some(error.to_string().as_str())
        );
        assert!(!document.sources[0].objects.is_empty());
        assert!(
            !document
                .facts
                .iter()
                .any(|fact| fact.codec == "native-attempt-record.v1")
        );
        let original = serde_json::to_vec(&document).unwrap();
        let set = FrozenEvidenceSet {
            operations: vec![reader.snapshot(std::time::Duration::from_secs(60))],
            contexts: vec![document],
            ..Default::default()
        };
        assert_supplied_transport(set, operation, &original);
    })
    .await;
    assert_eq!(counts, Default::default());
}
