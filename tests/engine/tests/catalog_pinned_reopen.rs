// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact immutable contexts resolve Model/Case/Stage graphs without identity-only admission.

#[path = "../../support/native_catalog.rs"]
mod native_catalog;

use std::collections::BTreeMap;
use std::sync::Arc;

use object_store::{ObjectStoreExt, memory::InMemory};
use pse_catalog::snapshot::ManifestRef;
use pse_catalog::store::{
    layout::{context_path, manifest_path},
    membership::AdmissionContext,
    publish::{BundleDraft, RelationDraft},
};
use pse_catalog::{
    Catalog, CatalogError, EncodingPolicy, FixedClock, RefName, RelationContract, Snapshot,
    TrustLevel,
};
use pse_ids::{CancellationToken, FixedBudget, SnapshotKind};
use pse_schema::model::{
    Authority, Cell, DerivationGranularity, Determinism, FieldContract, InputPort, Namespace,
    OutputPort, PassDecl, PortSource, RelationDecl, SnapshotClass,
};
use pse_schema::{Registry, RegistryBuilder};

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    for (namespace, name, class, foreign) in [
        (Namespace::Authored, "items", SnapshotClass::Model, None),
        (
            Namespace::Authored,
            "cases",
            SnapshotClass::Case,
            Some("authored.items"),
        ),
        (
            Namespace::Normalized,
            "outputs",
            SnapshotClass::Derived,
            Some("authored.cases"),
        ),
    ] {
        let mut id = FieldContract::key(
            "id",
            FieldContract::native(arrow::datatypes::DataType::UInt64),
            "actual key",
        );
        if let Some(target) = foreign {
            id = id.with_fk(target, "id");
        }
        let mut declaration = RelationDecl::new(
            namespace,
            name,
            1,
            if class == SnapshotClass::Derived {
                Authority::Derived
            } else {
                Authority::Authored
            },
            class,
            "pinned graph fixture",
        )
        .pk(&["id"])
        .columns(vec![
            id,
            FieldContract::payload(
                "value",
                FieldContract::native(arrow::datatypes::DataType::UInt64),
                "actual value",
            ),
        ]);
        if class == SnapshotClass::Derived {
            declaration = declaration.granularity(DerivationGranularity::Rule);
        }
        builder.declare_relation(declaration);
    }
    builder.declare_pass(
        PassDecl::new("inspect", "1", Determinism::Deterministic)
            .inputs(vec![InputPort {
                port: "case",
                relation: "authored.cases".to_owned(),
                source: PortSource::Pinned,
                required: true,
            }])
            .outputs(vec![OutputPort {
                port: "result",
                relation: "normalized.outputs".to_owned(),
            }]),
    );
    Arc::new(builder.build().expect("registry"))
}

fn catalog(store: Arc<InMemory>, registry: Arc<Registry>, budget: Arc<FixedBudget>) -> Catalog {
    native_catalog::with_projection(
        Catalog::open(
            store,
            registry,
            TrustLevel::Untrusted,
            Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
            native_catalog::from_reserver(budget),
        ),
        "inspect@1",
        Some(10),
    )
}

#[tokio::test]
async fn native_resolution_prepares_without_backend_reads_and_runs_once() {
    let catalog = catalog(
        Arc::new(InMemory::new()),
        registry(),
        FixedBudget::new(32 << 20),
    );
    let cancel = CancellationToken::new();
    let missing = ManifestRef {
        snapshot_id: pse_ids::SnapshotId(pse_ids::ContentHash::NIL),
        manifest_checksum: pse_ids::EncodingChecksum(pse_ids::ContentHash::NIL),
    };
    // Missing immutable input is discovered during execution, never by preparation.
    let prepared = catalog.prepare_pinned_manifest(missing, &cancel).unwrap();
    let display = prepared
        .computation()
        .original_plan()
        .display_indent()
        .to_string();
    assert!(display.contains("store.admit_manifest"));
    assert!(display.contains(&missing.manifest_checksum.0.to_prefixed()));
    assert!(prepared.execute(&cancel).await.is_err());

    let prepared = catalog
        .prepare_pinned_ref(RefName::parse("absent").unwrap(), &cancel)
        .unwrap();
    let duplicate = prepared.computation().clone();
    let completed = prepared.execute(&cancel).await.unwrap();
    assert!(completed.value().is_none());
    assert_eq!(completed.computation().batches()[0].num_rows(), 1);
    assert!(duplicate.execute(&cancel).await.is_err());
}

async fn publish(
    catalog: &Catalog,
    kind: SnapshotKind,
    parent: Option<Arc<Snapshot>>,
    value: u64,
) -> Arc<Snapshot> {
    let (name, role) = match kind {
        SnapshotKind::Model => ("authored.items", ""),
        SnapshotKind::Case => ("authored.cases", "model"),
        SnapshotKind::Stage => ("normalized.outputs", "case"),
        SnapshotKind::Run => panic!("fixture has no run producer"),
    };
    let mut context = AdmissionContext {
        traversal: Arc::default(),
        invocation: None,
        parents: parent.map_or_else(BTreeMap::new, |parent| {
            BTreeMap::from([(role.to_owned(), parent)])
        }),
        stage_pass: (kind == SnapshotKind::Stage)
            .then(|| catalog.registry().pass("inspect@1").expect("pass").id),
    };
    if kind == SnapshotKind::Stage {
        context = catalog
            .prepare_production(context, None, &CancellationToken::new())
            .expect("capture actual native stage binding")
            .inputs()
            .clone();
    }
    let spec = catalog.registry().relation(name).expect("relation");
    let cancel = CancellationToken::default();
    let batch = pse_relations::cells::batch_from_cells_owned(
        catalog.registry(),
        spec,
        &[vec![Cell::U64(1), Cell::U64(value)]],
        catalog.reserver().as_ref(),
        &cancel,
    )
    .expect("owned values");
    catalog
        .publish_bundle(
            BundleDraft {
                manifest: catalog.manifest_template(kind, &context).expect("template"),
                relations: BTreeMap::from([(
                    if kind == SnapshotKind::Stage {
                        "result".to_owned()
                    } else {
                        pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id)
                    },
                    RelationDraft {
                        contract: Arc::new(
                            RelationContract::from_spec(
                                catalog.registry(),
                                spec,
                                EncodingPolicy::IpcFile,
                            )
                            .expect("contract"),
                        ),
                        batches: vec![batch],
                    },
                )]),
                context,
            },
            &cancel,
        )
        .await
        .expect("complete publication")
}

async fn graph(catalog: &Catalog) -> Arc<Snapshot> {
    let model = publish(catalog, SnapshotKind::Model, None, 10).await;
    let case = publish(catalog, SnapshotKind::Case, Some(model), 20).await;
    publish(catalog, SnapshotKind::Stage, Some(case), 30).await
}

async fn selected_manifest(store: &InMemory, reference: ManifestRef) -> pse_catalog::Manifest {
    serde_json::from_slice(
        &store
            .get(&manifest_path(&reference.manifest_checksum))
            .await
            .expect("manifest")
            .bytes()
            .await
            .expect("manifest bytes"),
    )
    .expect("manifest JSON")
}

async fn receipt(store: &InMemory, reference: ManifestRef) -> serde_json::Value {
    let manifest = selected_manifest(store, reference).await;
    let checksum = manifest
        .admission_binding
        .expect("current binding")
        .encoding_checksum;
    serde_json::from_slice(
        &store
            .get(&context_path(&checksum))
            .await
            .expect("binding")
            .bytes()
            .await
            .expect("binding bytes"),
    )
    .expect("binding JSON")
}

async fn write_binding(
    store: &InMemory,
    reference: ManifestRef,
    wire: &serde_json::Value,
) -> ManifestRef {
    let bytes = serde_json::to_vec(wire).expect("JSON");
    let checksum = pse_ids::encoding_checksum(&bytes);
    store
        .put(&context_path(&checksum), bytes.into())
        .await
        .expect("binding fixture");
    let mut manifest = selected_manifest(store, reference).await;
    manifest
        .admission_binding
        .as_mut()
        .expect("binding")
        .encoding_checksum = checksum;
    let bytes = manifest.encode().expect("manifest encoding");
    let selected = ManifestRef {
        snapshot_id: manifest.snapshot_id,
        manifest_checksum: pse_ids::encoding_checksum(&bytes),
    };
    store
        .put(&manifest_path(&selected.manifest_checksum), bytes.into())
        .await
        .expect("manifest fixture");
    selected
}

#[tokio::test]
async fn fresh_catalog_pins_complete_actual_graph_across_later_ref_movement() {
    let store = Arc::new(InMemory::new());
    let registry = registry();
    let writer = catalog(store.clone(), registry.clone(), FixedBudget::new(64 << 20));
    let stage = graph(&writer).await;
    let cancel = CancellationToken::default();
    let name = RefName::parse("main").expect("ref");
    writer
        .compare_and_swap_ref(&name, None, &stage, &cancel)
        .await
        .expect("head");
    let budget = FixedBudget::new(64 << 20);
    let reader = catalog(store.clone(), registry, budget.clone());
    let (pinned, snapshot) = reader
        .read_pinned_ref(&name, &cancel)
        .await
        .expect("active graph admission")
        .expect("head");
    assert_eq!(pinned.manifest_ref(), stage.manifest_ref());
    assert_eq!(
        snapshot.parents()["case"].parents()["model"].snapshot_id(),
        stage.parents()["case"].parents()["model"].snapshot_id()
    );
    let observed = writer
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("head");
    let replacement = publish(&writer, SnapshotKind::Model, None, 99).await;
    writer
        .compare_and_swap_ref(&name, Some(&observed), &replacement, &cancel)
        .await
        .expect("move");
    assert_eq!(snapshot.manifest_ref(), stage.manifest_ref());
    let (_, current) = reader
        .read_pinned_ref(&name, &cancel)
        .await
        .expect("read moved")
        .expect("head");
    assert_eq!(current.manifest_ref(), replacement.manifest_ref());
    drop((pinned, snapshot, current));
    assert_eq!(
        budget.reserved(),
        0,
        "all decoded metadata and graph/table owners release"
    );
}

#[tokio::test]
async fn missing_wrong_role_and_stale_producer_receipts_fail_closed() {
    let store = Arc::new(InMemory::new());
    let registry = registry();
    let writer = catalog(store.clone(), registry.clone(), FixedBudget::new(64 << 20));
    let stage = graph(&writer).await;
    let reference = stage.manifest_ref();
    let original = receipt(&store, reference).await;
    let budget = FixedBudget::new(64 << 20);
    let reader = catalog(store.clone(), registry, budget.clone());
    let cancel = CancellationToken::default();
    let checksum = stage
        .manifest()
        .admission_binding
        .as_ref()
        .unwrap()
        .encoding_checksum;
    let original_bytes = store
        .get(&context_path(&checksum))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    store.delete(&context_path(&checksum)).await.unwrap();
    assert!(
        reader
            .read_pinned_manifest(reference, &cancel)
            .await
            .is_err()
    );
    assert_eq!(budget.reserved(), 0);
    store
        .put(&context_path(&checksum), original_bytes.into())
        .await
        .unwrap();
    for mutation in ["wrong-role", "wrong-producer", "wrong-parent"] {
        let mut wire = original.clone();
        match mutation {
            "wrong-role" => {
                let parent = wire["parents"]["case"].take();
                wire["parents"] = serde_json::json!({"model": parent});
            }
            "wrong-producer" => wire["producer"] = serde_json::json!(pse_ids::SemanticId::NIL),
            _ => wire["parents"]["case"] = serde_json::to_value(reference).expect("reference"),
        }
        let selected = write_binding(&store, reference, &wire).await;
        assert!(
            matches!(reader.read_pinned_manifest(selected, &cancel).await,
            Err(CatalogError::Admission { path, .. }) if path == "admission binding"),
            "{mutation}"
        );
        assert_eq!(budget.reserved(), 0, "{mutation}");
    }
    reader
        .read_pinned_manifest(reference, &cancel)
        .await
        .expect("original exact binding");
    assert_eq!(budget.reserved(), 0);
    let mut retired = original.clone();
    retired["version"] = serde_json::json!(1);
    let retired = write_binding(&store, reference, &retired).await;
    assert!(
        matches!(reader.read_pinned_manifest(retired, &cancel).await,
        Err(CatalogError::UnknownVersion { field, .. }) if field == "admission binding")
    );
    assert_eq!(budget.reserved(), 0);
    cancel.cancel();
    assert!(
        reader
            .read_pinned_manifest(reference, &cancel)
            .await
            .is_err()
    );
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn validly_rehashed_actual_stage_rows_still_must_satisfy_parent_content() {
    let store = Arc::new(InMemory::new());
    let registry = registry();
    let writer = catalog(store.clone(), registry.clone(), FixedBudget::new(64 << 20));
    let stage = graph(&writer).await;
    let cancel = CancellationToken::default();
    let mut manifest = stage.manifest().as_ref().clone();
    let contract = stage.relation_port("result").expect("port").contract();
    let spec = registry.relation("normalized.outputs").expect("spec");
    let batch = pse_relations::cells::batch_from_cells_owned(
        &registry,
        spec,
        &[vec![Cell::U64(99), Cell::U64(30)]],
        writer.reserver().as_ref(),
        &cancel,
    )
    .expect("locally valid rows");
    let canonical = pse_ids::canonicalize(
        &contract.canonical,
        std::slice::from_ref(&batch),
        writer.reserver().as_ref(),
        pse_ids::CanonicalizeOptions::default(),
    )
    .expect("valid canonical values");
    let encoded = pse_catalog::store::encode::ipc_file(&batch, writer.reserver().as_ref(), &cancel)
        .expect("finished IPC");
    let member = &mut manifest.relations[0];
    member.logical_hash = canonical.logical_hash;
    let encoding = &mut member.encodings[0];
    encoding.encoding_checksum = encoded.checksum;
    encoding.bytes = u64::try_from(encoded.bytes.len()).expect("length");
    encoding.path = pse_catalog::relation_path(
        &member.namespace,
        &member.name,
        member.version,
        &encoded.checksum,
        encoded.format,
    )
    .to_string();
    store
        .put(
            &object_store::path::Path::from(encoding.path.as_str()),
            encoded.bytes.into(),
        )
        .await
        .expect("forged row object");
    manifest.snapshot_id = pse_ids::snapshot_id(&manifest.frame()).expect("consistent membership");
    manifest
        .validate(registry.fingerprint())
        .expect("consistent envelope");
    let bytes = manifest.encode().expect("encoded manifest");
    let reference = ManifestRef {
        snapshot_id: manifest.snapshot_id,
        manifest_checksum: pse_ids::encoding_checksum(&bytes),
    };
    store
        .put(&manifest_path(&reference.manifest_checksum), bytes.into())
        .await
        .expect("forged manifest");
    let budget = FixedBudget::new(64 << 20);
    let reader = catalog(store, registry, budget.clone());
    let error = reader
        .read_pinned_manifest(reference, &cancel)
        .await
        .unwrap_err();
    let CatalogError::Semantic(diagnostic) = &error else {
        panic!("{error:?}")
    };
    let diagnostic: &dyn std::error::Error = diagnostic.as_ref();
    let Some(pse_rules::RuleError::InvariantViolations { count, findings }) =
        diagnostic.downcast_ref::<pse_rules::RuleError>()
    else {
        panic!("{error:?}")
    };
    assert_eq!(*count, 1);
    let declaration = reader
        .registry()
        .relation("runtime.diagnostics_findings")
        .unwrap();
    let rows = pse_relations::cells::cells_from_batch(reader.registry(), declaration, &findings[0])
        .unwrap();
    let check = reader
        .registry()
        .invariants()
        .iter()
        .find(|check| {
            check.relation == "normalized.outputs"
                && check.kind == pse_schema::model::InvariantKind::ForeignKey
        })
        .unwrap();
    assert_eq!(rows[0][3], Cell::Id(check.id));
    let target = reader.registry().relation("normalized.outputs").unwrap();
    let expected = datafusion::execution::context::SessionContext::new()
        .read_batch(batch)
        .unwrap()
        .select(vec![pse_catalog::session::scalar::key(
            target.id,
            target
                .primary_key
                .iter()
                .map(|name| (*name, datafusion::logical_expr::col(*name)))
                .collect(),
        )])
        .unwrap()
        .collect()
        .await
        .unwrap();
    let token = expected[0]
        .column(0)
        .as_any()
        .downcast_ref::<arrow::array::FixedSizeBinaryArray>()
        .unwrap()
        .value(0);
    let finding =
        pse_relations::generated::runtime::diagnostics_findings::Row::from_cells(rows[0].clone())
            .unwrap();
    let evidence = finding.evidence.row.unwrap();
    assert_eq!(evidence.relation_id, target.id);
    assert_eq!(
        evidence.row_key,
        pse_ids::ContentHash::try_from_slice(token).unwrap()
    );
    drop(error);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn equal_membership_retains_distinct_exact_parent_bindings() {
    let store = Arc::new(InMemory::new());
    let registry = registry();
    let writer = catalog(store.clone(), registry.clone(), FixedBudget::new(64 << 20));
    let model = publish(&writer, SnapshotKind::Model, None, 10).await;
    let case = publish(&writer, SnapshotKind::Case, Some(model.clone()), 20).await;
    let original = receipt(&store, case.manifest_ref()).await;
    let alternate = Catalog::open(
        store.clone(),
        registry,
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:01Z".to_owned())),
        native_catalog::from_reserver(FixedBudget::new(64 << 20)),
    );
    let alternate = native_catalog::with_projection(alternate, "inspect@1", Some(10));
    let other = publish(&alternate, SnapshotKind::Model, None, 10).await;
    assert_eq!(other.snapshot_id(), model.snapshot_id());
    assert_ne!(other.manifest_ref(), model.manifest_ref());
    let cancel = CancellationToken::default();
    let other = writer
        .read_pinned_manifest(other.manifest_ref(), &cancel)
        .await
        .expect("same rows, exact other encoding");
    let context = AdmissionContext {
        traversal: Arc::default(),
        invocation: None,
        parents: BTreeMap::from([("model".to_owned(), other)]),
        stage_pass: None,
    };
    let draft = BundleDraft {
        manifest: writer
            .manifest_template(SnapshotKind::Case, &context)
            .expect("same semantic template"),
        relations: case
            .relations()
            .iter()
            .map(|(port, relation)| {
                (
                    port.clone(),
                    RelationDraft {
                        contract: Arc::clone(relation.contract()),
                        batches: vec![relation.batch().clone()],
                    },
                )
            })
            .collect(),
        context,
    };
    let second = writer
        .publish_bundle(draft, &cancel)
        .await
        .expect("distinct physical binding");
    assert_eq!(second.snapshot_id(), case.snapshot_id());
    assert_ne!(second.manifest_ref(), case.manifest_ref());
    let reopened = writer
        .read_pinned_manifest(second.manifest_ref(), &cancel)
        .await
        .unwrap();
    assert_eq!(
        reopened.parents()["model"].manifest_ref(),
        second.parents()["model"].manifest_ref()
    );
    assert_eq!(receipt(&store, case.manifest_ref()).await, original);
    let restored = writer
        .read_pinned_manifest(case.manifest_ref(), &cancel)
        .await
        .expect("original context remains complete");
    assert_eq!(
        restored.parents()["model"].manifest_ref(),
        model.manifest_ref()
    );
}

#[tokio::test]
async fn parent_graph_refuses_before_allocating_beyond_the_shared_budget() {
    let store = Arc::new(InMemory::new());
    let registry = registry();
    let writer = catalog(store.clone(), registry.clone(), FixedBudget::new(64 << 20));
    let stage = graph(&writer).await;
    let budget = FixedBudget::new(1);
    let reader = catalog(store, registry, budget.clone());
    let error = reader
        .read_pinned_manifest(stage.manifest_ref(), &CancellationToken::default())
        .await
        .expect_err("one byte cannot admit the parent graph");
    assert!(
        matches!(
            &error,
            CatalogError::Reserve(pse_ids::ReserveError::Exhausted { .. })
                | CatalogError::ResourceLimit { .. }
        ),
        "{error:?}"
    );
    assert_eq!(budget.reserved(), 0);
}
