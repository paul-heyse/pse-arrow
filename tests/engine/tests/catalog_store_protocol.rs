// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Publication, independent reopening and conditional-ref failure boundaries.
#![allow(clippy::unwrap_used, reason = "test fixtures fail directly")]

#[path = "../../support/native_catalog.rs"]
mod native_catalog;

use std::collections::BTreeMap;
use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use object_store::memory::InMemory;
use object_store::{ObjectStore, ObjectStoreExt};
use pse_catalog::snapshot::ManifestRef;
use pse_catalog::store::membership::{AdmissionContext, SemanticValidator};
use pse_catalog::store::open::Catalog;
use pse_catalog::store::publish::{BundleDraft, RelationDraft};
use pse_catalog::{
    CatalogError, CompilerRef, EncodingPolicy, FixedClock, Manifest, RefName, RelationContract,
    ToolchainRef, TrustLevel,
};
use pse_ids::{
    CANON_VERSION, CancellationToken, ContentHash, FixedBudget, MemoryReserver, SNAPSHOT_PROFILE,
    SnapshotId, SnapshotKind,
};
use pse_schema::Registry;
use pse_schema::builder::RegistryBuilder;
use pse_schema::model::RelationKey;
use pse_schema::model::{
    Authority, Cell, ExtensionUse, FieldContract, Namespace, RelationDecl, SnapshotClass,
};

fn registry() -> Arc<Registry> {
    registry_with_invariant(false)
}
fn registry_with_invariant(invariant: bool) -> Arc<Registry> {
    registry_of_class(invariant, SnapshotClass::Model)
}
fn registry_of_class(invariant: bool, class: SnapshotClass) -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "items",
            1,
            Authority::Authored,
            class,
            "fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(arrow::datatypes::DataType::UInt64),
                "identity",
            ),
            FieldContract::payload(
                "value",
                FieldContract::native(arrow::datatypes::DataType::Utf8),
                "value",
            ),
        ]),
    );
    if invariant {
        use pse_schema::model::{
            InvariantDecl, InvariantKind, RuleDecl, RuleExpr, RuleHead, RulePlan,
        };
        builder.declare_rule(RuleDecl::new(
            "reject_all",
            "1",
            0,
            RuleHead::Violations {
                of: "authored.items".to_owned(),
                key_columns: vec!["id"],
            },
            RulePlan::Project {
                input: Box::new(RulePlan::Scan {
                    relation: "authored.items".to_owned(),
                    port: "input",
                }),
                columns: vec![("id".into(), RuleExpr::col("input.id"))],
            },
        ));
        builder.declare_invariant(InvariantDecl::error(
            "authored.items",
            "reject_all",
            InvariantKind::Check,
            "reject_all@1",
            "fixture rejects rows",
        ));
    }
    Arc::new(builder.build().unwrap())
}

#[tokio::test]
async fn registered_invariants_require_an_executable_validator_even_with_valid_rows() {
    let reg = registry_with_invariant(true);
    let catalog = unvalidated_catalog(
        Arc::new(InMemory::new()),
        Arc::clone(&reg),
        FixedBudget::new(64 << 20),
    );
    let error = catalog
        .publish_bundle(
            draft(&reg, "structurally valid"),
            &CancellationToken::default(),
        )
        .await
        .unwrap_err();
    assert!(matches!(error,CatalogError::Admission {path,..} if path=="semantic validator"));
}

fn document_registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    for (name, columns) in [
        (
            "documents",
            vec![
                FieldContract::key("document_id", FieldContract::id(), "identity"),
                FieldContract::payload(
                    "source_text",
                    FieldContract::native(arrow::datatypes::DataType::Utf8),
                    "exact UTF-8 source",
                ),
            ],
        ),
        (
            "spans",
            vec![
                FieldContract::key("document_id", FieldContract::id(), "identity"),
                FieldContract::payload(
                    "span",
                    FieldContract::extended(ExtensionUse::SourceSpan),
                    "location",
                ),
            ],
        ),
    ] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "fixture",
            )
            .pk(&["document_id"])
            .columns(columns),
        );
    }
    Arc::new(builder.build().unwrap())
}
fn document_draft(reg: &Registry, source: &[u8], end: u64) -> BundleDraft {
    let document = pse_ids::SemanticId::from_bytes([1; 16]);
    let mut relations = BTreeMap::new();
    for (name, value) in [
        (
            "authored.documents",
            Cell::Text(std::str::from_utf8(source).unwrap().to_owned()),
        ),
        (
            "authored.spans",
            Cell::Struct(vec![Cell::Id(document), Cell::U64(0), Cell::U64(end)]),
        ),
    ] {
        let spec = reg.relation(name).unwrap();
        let batch =
            pse_relations::cells::batch_from_cells(reg, spec, &[vec![Cell::Id(document), value]])
                .unwrap();
        relations.insert(
            pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id),
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap(),
                ),
                batches: vec![batch],
            },
        );
    }
    BundleDraft {
        manifest: manifest(reg),
        relations,
        context: AdmissionContext::default(),
    }
}

fn stage_registry() -> Arc<Registry> {
    use pse_schema::model::{Determinism, InputPort, OutputPort, PassDecl, PortSource};
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    declare_change_fixture(&mut builder);
    for name in ["items", "other"] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "fixture",
            )
            .pk(&["id"])
            .columns(vec![FieldContract::key(
                "id",
                FieldContract::native(arrow::datatypes::DataType::UInt64),
                "identity",
            )]),
        );
    }
    for (namespace, name, key, columns) in [
        (
            Namespace::Authored,
            "model_revisions",
            "model_revision_id",
            vec![
                FieldContract::key("model_revision_id", FieldContract::id(), "revision"),
                FieldContract::payload("snapshot_id", FieldContract::hash(), "target"),
            ],
        ),
        (
            Namespace::Provenance,
            "pass_records",
            "pass_run_id",
            vec![
                FieldContract::key("pass_run_id", FieldContract::id(), "attempt"),
                FieldContract::payload("pass_id", FieldContract::id(), "producer"),
                FieldContract::payload(
                    "version",
                    FieldContract::native(arrow::datatypes::DataType::Utf8),
                    "version",
                ),
                FieldContract::payload("snapshot_out", FieldContract::hash(), "target"),
                FieldContract::payload(
                    "status",
                    FieldContract::native(arrow::datatypes::DataType::Utf8),
                    "outcome",
                ),
            ],
        ),
    ] {
        builder.declare_relation(
            RelationDecl::new(
                namespace,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Sidecar,
                "fixture",
            )
            .pk(&[key])
            .columns(columns),
        );
    }
    builder.declare_pass(
        PassDecl::new("P0", "1", Determinism::Deterministic)
            .inputs(vec![InputPort {
                port: "input",
                relation: "authored.items".to_owned(),
                source: PortSource::Pinned,
                required: true,
            }])
            .outputs(vec![OutputPort {
                port: "out",
                relation: "authored.items".to_owned(),
            }]),
    );
    Arc::new(builder.build().unwrap())
}

fn declare_change_fixture(builder: &mut RegistryBuilder) {
    pse_schema::catalog::enums_platform::declare(builder);
    pse_schema::catalog::s4_schema::declare_relations(builder);
    pse_schema::catalog::s22_change_sets::declare(builder);
}

#[tokio::test]
async fn complete_change_receipts_reopen_every_actual_staged_row_before_ref_publication() {
    let reg = stage_registry();
    let store = Arc::new(InMemory::new());
    let catalog = catalog(
        Arc::clone(&store),
        Arc::clone(&reg),
        FixedBudget::new(128 << 20),
    );
    let cancel = CancellationToken::default();
    let item = reg.relation("authored.items").unwrap();
    let staged = catalog
        .publish_staged_batch(
            sidecar_draft(&reg, "authored.items", vec![Cell::U64(1)]),
            &cancel,
        )
        .await
        .unwrap();
    assert!(
        catalog
            .publish_sidecar(
                sidecar_draft(&reg, "authored.items", vec![Cell::U64(1)]),
                &cancel
            )
            .await
            .is_err()
    );
    let mut draft = empty_model(&reg);
    draft.relations.insert(
        pse_ids::model_port_name(item.key.namespace.as_str(), item.id),
        sidecar_draft(&reg, "authored.items", vec![Cell::U64(1)]),
    );
    let model = catalog.publish_bundle(draft, &cancel).await.unwrap();
    let revision_id = pse_ids::SemanticId::from_bytes([31; 16]);
    let revision = catalog
        .publish_sidecar(
            sidecar_draft(
                &reg,
                "authored.model_revisions",
                vec![Cell::Id(revision_id), Cell::Hash(model.snapshot_id().0)],
            ),
            &cancel,
        )
        .await
        .unwrap();
    let revision = catalog
        .revision_receipt(&revision, revision_id, &model)
        .unwrap();
    let draft = change_receipt_draft(&catalog, &reg, staged, &cancel).await;
    assert_wrong_change_header_rejected(&catalog, &reg, &draft, &revision, &cancel).await;
    let mut missing = draft.clone();
    missing.staged.clear();
    assert!(
        catalog
            .publish_change_set(missing, None, &revision, &cancel)
            .await
            .is_err()
    );
    let changes = catalog
        .publish_change_set(draft, None, &revision, &cancel)
        .await
        .unwrap();
    assert_change_metadata_ownership(&store, &reg, changes.reference()).await;
    let restored = catalog
        .read_change_set(changes.reference(), &cancel)
        .await
        .unwrap();
    catalog
        .validate_change_context(&restored, None, &model, &cancel)
        .await
        .unwrap();
    assert_eq!(restored.artifacts().staged.len(), 1);
    let paired = catalog.with_change_set(&revision, &restored).unwrap();
    let head = RefName::parse("changes").unwrap();
    catalog
        .compare_and_swap_revision_ref(&head, None, &model, &paired, &cancel)
        .await
        .unwrap();
    let observed = catalog.read_ref(&head, &cancel).await.unwrap().unwrap();
    assert_eq!(
        observed.revision_ref().unwrap().change_set.as_ref(),
        Some(changes.reference())
    );
    let path = pse_catalog::store::layout::change_set_path(&pse_ids::EncodingChecksum(
        changes.reference().encoding_checksum,
    ));
    let bytes = store.get(&path).await.unwrap().bytes().await.unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    wire["staged"] = serde_json::json!({});
    let forged = serde_json::to_vec(&wire).unwrap();
    let checksum = pse_ids::encoding_checksum(&forged);
    store
        .put(
            &pse_catalog::store::layout::change_set_path(&checksum),
            forged.into(),
        )
        .await
        .unwrap();
    let mut reference = changes.reference().clone();
    reference.encoding_checksum = checksum.0;
    assert!(catalog.read_change_set(&reference, &cancel).await.is_err());
}
async fn assert_wrong_change_header_rejected(
    catalog: &Catalog,
    registry: &Registry,
    draft: &pse_catalog::store::changes::ChangeSetDraft,
    output: &pse_catalog::store::sidecar::RevisionReceipt,
    cancel: &CancellationToken,
) {
    let mut inconsistent = draft.clone();
    inconsistent.header = catalog
        .publish_sidecar(
            sidecar_draft(
                registry,
                "authored.change_sets",
                vec![
                    Cell::Id(pse_ids::SemanticId::from_bytes([33; 16])),
                    Cell::Id(pse_ids::SemanticId::NIL),
                    Cell::text("fixture"),
                    Cell::text("another valid header"),
                    Cell::I64(0),
                ],
            ),
            cancel,
        )
        .await
        .unwrap();
    let error = catalog
        .publish_change_set(inconsistent, None, output, cancel)
        .await
        .unwrap_err();
    assert!(
        matches!(&error, CatalogError::Admission { reason, .. } if
        reason.contains("operation identity or ordinal differs from its header")),
        "{error:?}"
    );
}
async fn change_receipt_draft(
    catalog: &Catalog,
    reg: &Registry,
    staged: pse_catalog::store::sidecar::StagedArtifact,
    cancel: &CancellationToken,
) -> pse_catalog::store::changes::ChangeSetDraft {
    let item = reg.relation("authored.items").unwrap();
    let change_id = pse_ids::SemanticId::from_bytes([32; 16]);
    let header = catalog
        .publish_sidecar(
            sidecar_draft(
                reg,
                "authored.change_sets",
                vec![
                    Cell::Id(change_id),
                    Cell::Id(pse_ids::SemanticId::NIL),
                    Cell::text("fixture"),
                    Cell::text("insert one"),
                    Cell::I64(0),
                ],
            ),
            cancel,
        )
        .await
        .unwrap();
    let reference = Cell::Struct(vec![Cell::text("operation/0/key"), Cell::U64(0)]);
    let operations = catalog
        .publish_sidecar(
            sidecar_draft(
                reg,
                "authored.change_ops",
                vec![
                    Cell::Id(change_id),
                    Cell::U64(0),
                    Cell::Enum("insert"),
                    Cell::Id(item.id),
                    reference.clone(),
                    reference,
                    Cell::Null,
                ],
            ),
            cancel,
        )
        .await
        .unwrap();
    pse_catalog::store::changes::ChangeSetDraft {
        header,
        operations,
        staged: BTreeMap::from([("operation/0/key".to_owned(), staged)]),
        supporting_revisions: vec![],
    }
}
fn empty_model(reg: &Registry) -> BundleDraft {
    BundleDraft {
        manifest: manifest(reg),
        relations: reg
            .relations()
            .iter()
            .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
            .map(|spec| {
                (
                    pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id),
                    RelationDraft {
                        contract: Arc::new(
                            RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile)
                                .unwrap(),
                        ),
                        batches: vec![],
                    },
                )
            })
            .collect(),
        context: AdmissionContext::default(),
    }
}

fn sidecar_draft(reg: &Registry, name: &str, row: Vec<Cell>) -> RelationDraft {
    let spec = reg.relation(name).unwrap();
    RelationDraft {
        contract: Arc::new(
            RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap(),
        ),
        batches: vec![pse_relations::cells::batch_from_cells(reg, spec, &[row]).unwrap()],
    }
}

#[tokio::test]
async fn sidecars_and_atomic_revision_refs_validate_actual_revision_rows() {
    let reg = stage_registry();
    let store = Arc::new(InMemory::new());
    let catalog = catalog(
        Arc::clone(&store),
        Arc::clone(&reg),
        FixedBudget::new(128 << 20),
    );
    let cancel = CancellationToken::default();
    let model = catalog
        .publish_bundle(empty_model(&reg), &cancel)
        .await
        .unwrap();
    let revision_id = pse_ids::SemanticId::from_bytes([42; 16]);
    let sidecar = catalog
        .publish_sidecar(
            sidecar_draft(
                &reg,
                "authored.model_revisions",
                vec![Cell::Id(revision_id), Cell::Hash(model.snapshot_id().0)],
            ),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(model.relations().len(), 3);
    let reread = catalog
        .read_sidecar(sidecar.reference(), &cancel)
        .await
        .unwrap();
    assert_eq!(reread.relation().batch(), sidecar.relation().batch());
    assert!(
        catalog
            .revision_receipt(&sidecar, pse_ids::SemanticId::from_bytes([43; 16]), &model)
            .is_err()
    );
    let receipt = catalog
        .revision_receipt(&sidecar, revision_id, &model)
        .unwrap();
    let head = RefName::parse("revision").unwrap();
    catalog
        .compare_and_swap_revision_ref(&head, None, &model, &receipt, &cancel)
        .await
        .unwrap();
    let observed = catalog.read_ref(&head, &cancel).await.unwrap().unwrap();
    assert_eq!(observed.revision_ref().unwrap().revision_id, revision_id);
    assert_eq!(observed.manifest_ref(), model.manifest_ref());
    catalog
        .read_snapshot(&head, &AdmissionContext::default(), &cancel)
        .await
        .unwrap()
        .unwrap();
    let bad = catalog
        .publish_sidecar(
            sidecar_draft(
                &reg,
                "authored.model_revisions",
                vec![Cell::Id(revision_id), Cell::Hash(ContentHash::NIL)],
            ),
            &cancel,
        )
        .await
        .unwrap();
    assert!(catalog.revision_receipt(&bad, revision_id, &model).is_err());
    let path =
        object_store::path::Path::from(sidecar.reference().member().encodings[0].path.as_str());
    store.put(&path, b"corrupt".to_vec().into()).await.unwrap();
    assert!(
        catalog
            .compare_and_swap_revision_ref(&head, Some(&observed), &model, &receipt, &cancel)
            .await
            .is_err()
    );
    assert!(catalog.read_ref(&head, &cancel).await.is_err());
}

fn stage_draft(reg: &Registry, parent: &Arc<pse_catalog::Snapshot>) -> BundleDraft {
    let pass = reg.pass("P0").unwrap();
    let spec = reg.relation("authored.items").unwrap();
    let mut manifest = manifest(reg);
    manifest.snapshot_kind = SnapshotKind::Stage;
    manifest.compiler.passes.push(pse_catalog::PassRef {
        pass_id: pass.id,
        version: pass.version.to_owned(),
    });
    manifest.semantic_parents.push(pse_ids::SnapshotParent {
        role: "input".to_owned(),
        snapshot_id: parent.snapshot_id(),
    });
    BundleDraft {
        manifest,
        relations: BTreeMap::from([(
            "out".to_owned(),
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap(),
                ),
                batches: vec![],
            },
        )]),
        context: AdmissionContext {
            traversal: Arc::default(),
            invocation: None,
            parents: BTreeMap::from([("input".to_owned(), Arc::clone(parent))]),
            stage_pass: Some(pass.id),
        },
    }
}

#[tokio::test]
async fn stage_admission_requires_actual_parents_exact_producer_and_every_output() {
    let reg = stage_registry();
    let store = Arc::new(InMemory::new());
    let catalog = catalog(
        Arc::clone(&store),
        Arc::clone(&reg),
        FixedBudget::new(128 << 20),
    );
    let cancel = CancellationToken::default();
    let model = catalog
        .publish_bundle(empty_model(&reg), &cancel)
        .await
        .unwrap();
    for bad in [0, 1, 2, 3] {
        let mut draft = stage_draft(&reg, &model);
        match bad {
            0 => draft.context.parents.clear(),
            1 => draft.context.stage_pass = None,
            2 => draft.manifest.compiler.passes[0].version = "unregistered".to_owned(),
            _ => draft.relations.clear(),
        }
        assert!(catalog.publish_bundle(draft, &cancel).await.is_err());
    }
    let completed = catalog
        .prepare_production(stage_draft(&reg, &model).context, None, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let stage = catalog
        .publish_production(completed, &cancel)
        .await
        .unwrap()
        .snapshot;
    assert_eq!(stage.relations().len(), 1);
    assert!(
        catalog
            .read_manifest(stage.manifest_ref(), &AdmissionContext::default(), &cancel)
            .await
            .is_err()
    );
    let foreign = super_catalog(store, Arc::clone(&reg));
    assert!(
        foreign
            .read_manifest(
                stage.manifest_ref(),
                &stage_draft(&reg, &model).context,
                &cancel
            )
            .await
            .is_err()
    );
    let reopened = foreign
        .read_manifest(model.manifest_ref(), &AdmissionContext::default(), &cancel)
        .await
        .unwrap();
    assert_eq!(reopened.manifest_ref(), model.manifest_ref());
    let mut context = stage_draft(&reg, &reopened).context;
    context.invocation = stage.invocation().cloned();
    foreign
        .read_manifest(stage.manifest_ref(), &context, &cancel)
        .await
        .unwrap();
}
fn super_catalog(store: Arc<InMemory>, reg: Arc<Registry>) -> Catalog {
    catalog(store, reg, FixedBudget::new(128 << 20))
}

struct TemporaryStore(std::path::PathBuf);
impl TemporaryStore {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pse-catalog-fault-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TemporaryStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn real_create_failures_leave_the_previous_ref_readable() {
    let reg = registry();
    let cancel = CancellationToken::default();
    let reference = catalog(
        Arc::new(InMemory::new()),
        Arc::clone(&reg),
        FixedBudget::new(64 << 20),
    );
    let expected = reference
        .publish_bundle(draft(&reg, "new"), &cancel)
        .await
        .unwrap();
    let directory = TemporaryStore::new();
    let store: Arc<dyn ObjectStore> =
        Arc::new(object_store::local::LocalFileSystem::new_with_prefix(&directory.0).unwrap());
    let catalog = Catalog::open(
        store,
        Arc::clone(&reg),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-01-01T00:00:00Z".to_owned())),
        native_catalog::from_reserver(FixedBudget::new(64 << 20)),
    );
    let catalog = native_catalog::with_invariants(catalog);
    let old = catalog
        .publish_bundle(draft(&reg, "old"), &cancel)
        .await
        .unwrap();
    let head = RefName::parse("head").unwrap();
    catalog
        .compare_and_swap_ref(&head, None, &old, &cancel)
        .await
        .unwrap();
    for path in [
        expected.manifest().relations[0].encodings[0].path.clone(),
        pse_catalog::manifest_path(&expected.manifest_ref().manifest_checksum).to_string(),
    ] {
        let collision = directory.0.join(&path);
        std::fs::create_dir_all(&collision).unwrap();
        assert!(
            catalog
                .publish_bundle(draft(&reg, "new"), &cancel)
                .await
                .is_err()
        );
        assert_eq!(
            catalog
                .read_snapshot(&head, &AdmissionContext::default(), &cancel)
                .await
                .unwrap()
                .unwrap()
                .manifest_ref(),
            old.manifest_ref()
        );
        std::fs::remove_dir(&collision).unwrap();
    }
    let new = catalog
        .publish_bundle(draft(&reg, "new"), &cancel)
        .await
        .unwrap();
    let blocked = RefName::parse("blocked").unwrap();
    std::fs::create_dir_all(directory.0.join(pse_catalog::ref_path(&blocked).as_ref())).unwrap();
    assert!(
        catalog
            .compare_and_swap_ref(&blocked, None, &new, &cancel)
            .await
            .is_err()
    );
    assert_eq!(
        catalog
            .read_snapshot(&head, &AdmissionContext::default(), &cancel)
            .await
            .unwrap()
            .unwrap()
            .manifest_ref(),
        old.manifest_ref()
    );
}
fn catalog(store: Arc<InMemory>, reg: Arc<Registry>, budget: Arc<FixedBudget>) -> Catalog {
    let projection = reg.pass("P0@1").is_some();
    let catalog = unvalidated_catalog(store, reg, budget);
    if projection {
        native_catalog::with_projection(catalog, "P0@1", None)
    } else {
        native_catalog::with_invariants(catalog)
    }
}
fn unvalidated_catalog(
    store: Arc<InMemory>,
    reg: Arc<Registry>,
    budget: Arc<FixedBudget>,
) -> Catalog {
    let store: Arc<dyn ObjectStore> = store;
    let reserver: Arc<dyn MemoryReserver> = budget;
    Catalog::open(
        store,
        reg,
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-01-01T00:00:00Z".to_owned())),
        native_catalog::from_reserver(reserver),
    )
}
fn manifest(reg: &Registry) -> Manifest {
    Manifest {
        admission_binding: None,
        manifest_version: pse_catalog::MANIFEST_VERSION.to_owned(),
        snapshot_kind: SnapshotKind::Model,
        snapshot_id: SnapshotId(ContentHash::NIL),
        membership_profile: SNAPSHOT_PROFILE.to_owned(),
        created_at: String::new(),
        schema_registry_fingerprint: reg.fingerprint(),
        relations: Vec::new(),
        packages: Vec::new(),
        compiler: CompilerRef {
            version: "fixture".to_owned(),
            passes: Vec::new(),
        },
        engine_profile: None,
        numerical_policy: None,
        toolchain: ToolchainRef {
            lockfile_hash: ContentHash::NIL,
            canonicalization: CANON_VERSION.to_owned(),
        },
        kernels: Vec::new(),
        semantic_parents: Vec::new(),
        evidence: Vec::new(),
    }
}
fn draft(reg: &Registry, value: &str) -> BundleDraft {
    let spec = reg.relation("authored.items").unwrap();
    let batch =
        pse_relations::cells::batch_from_cells(reg, spec, &[vec![Cell::U64(1), Cell::text(value)]])
            .unwrap();
    BundleDraft {
        manifest: manifest(reg),
        relations: BTreeMap::from([(
            pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id),
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap(),
                ),
                batches: vec![batch],
            },
        )]),
        context: AdmissionContext::default(),
    }
}

#[tokio::test]
async fn publish_reopen_and_compare_and_swap_preserve_exact_manifest_and_old_head() {
    let reg = registry();
    let store = Arc::new(InMemory::new());
    let budget = FixedBudget::new(64 << 20);
    let catalog = catalog(Arc::clone(&store), Arc::clone(&reg), Arc::clone(&budget));
    let cancel = CancellationToken::default();
    let first = catalog
        .publish_bundle(draft(&reg, "first"), &cancel)
        .await
        .unwrap();
    let head = RefName::parse("head").unwrap();
    catalog
        .compare_and_swap_ref(&head, None, &first, &cancel)
        .await
        .unwrap();
    let observed = catalog.read_ref(&head, &cancel).await.unwrap().unwrap();
    let reopened = catalog
        .read_snapshot(&head, &AdmissionContext::default(), &cancel)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reopened.manifest_ref(), first.manifest_ref());
    assert_eq!(
        reopened.relation("authored", "items").unwrap().batch(),
        first.relation("authored", "items").unwrap().batch()
    );
    let second = catalog
        .publish_bundle(draft(&reg, "second"), &cancel)
        .await
        .unwrap();
    catalog
        .compare_and_swap_ref(&head, Some(&observed), &second, &cancel)
        .await
        .unwrap();
    assert!(matches!(
        catalog
            .compare_and_swap_ref(&head, Some(&observed), &first, &cancel)
            .await,
        Err(CatalogError::RefConflict { .. })
    ));
    assert_eq!(
        catalog
            .read_ref(&head, &cancel)
            .await
            .unwrap()
            .unwrap()
            .manifest_ref(),
        second.manifest_ref()
    );
    assert_eq!(catalog.list_refs(&cancel).await.unwrap(), vec![head]);
    drop(first);
    drop(second);
    drop(reopened);
    drop(observed);
    drop(catalog);
    drop(store);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn consistently_rewritten_manifest_cannot_hide_invalid_membership_or_row_claims() {
    let reg = registry();
    let store = Arc::new(InMemory::new());
    let budget = FixedBudget::new(64 << 20);
    let catalog = catalog(Arc::clone(&store), Arc::clone(&reg), budget);
    let cancel = CancellationToken::default();
    let snapshot = catalog
        .publish_bundle(draft(&reg, "value"), &cancel)
        .await
        .unwrap();
    let mut forged = (**snapshot.manifest()).clone();
    forged.relations[0].rows = 99;
    let bytes = forged.encode().unwrap();
    let checksum = pse_ids::encoding_checksum(&bytes);
    store
        .put(&pse_catalog::manifest_path(&checksum), bytes.into())
        .await
        .unwrap();
    assert!(
        catalog
            .read_manifest(
                ManifestRef {
                    snapshot_id: forged.snapshot_id,
                    manifest_checksum: checksum
                },
                &AdmissionContext::default(),
                &cancel
            )
            .await
            .is_err()
    );
    forged.relations.clear();
    forged.snapshot_id = pse_ids::snapshot_id(&forged.frame()).unwrap();
    let bytes = forged.encode().unwrap();
    let checksum = pse_ids::encoding_checksum(&bytes);
    store
        .put(&pse_catalog::manifest_path(&checksum), bytes.into())
        .await
        .unwrap();
    assert!(
        catalog
            .read_manifest(
                ManifestRef {
                    snapshot_id: forged.snapshot_id,
                    manifest_checksum: checksum
                },
                &AdmissionContext::default(),
                &cancel
            )
            .await
            .is_err()
    );
}

#[derive(Debug)]
struct GenericOnlyValidator;
impl SemanticValidator for GenericOnlyValidator {
    fn validate<'a>(
        &'a self,
        _reg: &'a Registry,
        _rows: &'a BTreeMap<RelationKey, RecordBatch>,
        _session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async { cancel.checkpoint().map_err(CatalogError::from) })
    }
}

#[derive(Debug)]
struct RejectNonempty;
impl SemanticValidator for RejectNonempty {
    fn validate<'a>(
        &'a self,
        _reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        _session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            cancel.checkpoint()?;
            if rows.values().any(|batch| batch.num_rows() != 0) {
                Err(CatalogError::UserModel {
                    message: "reject_all found actual rows".to_owned(),
                })
            } else {
                Ok(())
            }
        })
    }
}

#[tokio::test]
async fn sidecar_invariants_require_actual_evaluation_on_publish_and_restore() {
    let reg = registry_of_class(true, SnapshotClass::Sidecar);
    let store = Arc::new(InMemory::new());
    let budget = FixedBudget::new(64 << 20);
    let checked = catalog(store.clone(), reg.clone(), budget.clone())
        .with_semantic_validator(Arc::new(RejectNonempty));
    let unchecked = unvalidated_catalog(store, reg.clone(), budget.clone());
    let spec = reg.relation("authored.items").unwrap();
    let empty = RelationDraft {
        contract: Arc::new(
            RelationContract::from_spec(&reg, spec, EncodingPolicy::IpcFile).unwrap(),
        ),
        batches: vec![],
    };
    let cancel = CancellationToken::default();
    assert!(
        unchecked
            .publish_sidecar(empty.clone(), &cancel)
            .await
            .is_err()
    );
    assert!(
        matches!(checked.publish_sidecar(sidecar_draft(&reg, "authored.items",
        vec![Cell::U64(1), Cell::text("forbidden")]), &cancel).await,
        Err(CatalogError::UserModel { message }) if message.contains("actual rows"))
    );
    assert_eq!(budget.reserved(), 0);
    let admitted = checked.publish_sidecar(empty, &cancel).await.unwrap();
    assert!(
        unchecked
            .read_sidecar(admitted.reference(), &cancel)
            .await
            .is_err()
    );
    let restored = checked
        .read_sidecar(admitted.reference(), &cancel)
        .await
        .unwrap();
    assert_eq!(restored.relation().batch().num_rows(), 0);
    drop((admitted, restored, checked, unchecked));
    assert_eq!(budget.reserved(), 0);
}

/// This fixture has a deliberately tiny document language: exactly `model x`.
/// Both registered projections are reconstructed and compared from those actual bytes.
#[derive(Debug)]
struct DocumentFixtureValidator;
impl SemanticValidator for DocumentFixtureValidator {
    fn validate<'a>(
        &'a self,
        _reg: &'a Registry,
        _rows: &'a BTreeMap<RelationKey, RecordBatch>,
        _session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async { cancel.checkpoint().map_err(CatalogError::from) })
    }
    fn validate_snapshot_sources<'a>(
        &'a self,
        catalog: &'a Catalog,
        kind: SnapshotKind,
        context: &'a AdmissionContext,
        candidates: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            assert_eq!(kind, SnapshotKind::Model);
            assert!(context.parents.is_empty());
            let reg = catalog.registry();
            let documents = reg.relation("authored.documents").unwrap();
            let rows =
                pse_relations::cells::cells_from_batch(reg, documents, &candidates[&documents.key])
                    .unwrap();
            let [Cell::Id(_), Cell::Text(source)] = rows[0].as_slice() else {
                return Err(CatalogError::Membership {
                    reason: "fixture document row shape is invalid".to_owned(),
                });
            };
            cancel.checkpoint()?;
            if source != "model x" {
                return Err(CatalogError::Membership {
                    reason: "unrecognized fixture document".to_owned(),
                });
            }
            let expected = document_draft(reg, source.as_bytes(), 7);
            for spec in reg
                .relations()
                .iter()
                .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
            {
                let expected =
                    &expected.relations[&pse_ids::model_port_name("authored", spec.id)].batches[0];
                let actual =
                    pse_relations::cells::cells_from_batch(reg, spec, &candidates[&spec.key])
                        .unwrap();
                let expected = pse_relations::cells::cells_from_batch(reg, spec, expected).unwrap();
                if actual != expected {
                    return Err(CatalogError::Membership {
                        reason: "fixture document projection differs from actual rows".to_owned(),
                    });
                }
            }
            Ok(())
        })
    }
}

#[tokio::test]
async fn source_claims_require_correspondence_on_both_publish_and_read() {
    let reg = document_registry();
    let store = Arc::new(InMemory::new());
    let fixture = catalog(store.clone(), reg.clone(), FixedBudget::new(64 << 20))
        .with_semantic_validator(Arc::new(DocumentFixtureValidator));
    let cancel = CancellationToken::default();
    let admitted = fixture
        .publish_bundle(document_draft(&reg, b"model x", 7), &cancel)
        .await
        .unwrap();
    for generic in [false, true] {
        let mut guarded = catalog(store.clone(), reg.clone(), FixedBudget::new(64 << 20));
        if generic {
            guarded = guarded.with_semantic_validator(Arc::new(GenericOnlyValidator));
        }
        for result in [
            guarded
                .publish_bundle(document_draft(&reg, b"model x", 7), &cancel)
                .await,
            guarded
                .read_manifest(
                    admitted.manifest_ref(),
                    &AdmissionContext::default(),
                    &cancel,
                )
                .await,
        ] {
            assert!(matches!(result, Err(CatalogError::Membership { reason })
                if reason.contains("document-to-row correspondence")));
        }
    }
}

#[tokio::test]
async fn generic_invariant_success_cannot_certify_stage_producer_on_write_or_read() {
    let reg = stage_registry();
    let store = Arc::new(InMemory::new());
    let fixture = catalog(
        Arc::clone(&store),
        Arc::clone(&reg),
        FixedBudget::new(128 << 20),
    );
    let cancel = CancellationToken::default();
    let model = fixture
        .publish_bundle(empty_model(&reg), &cancel)
        .await
        .unwrap();
    let completed = fixture
        .prepare_production(stage_draft(&reg, &model).context, None, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let stage = fixture
        .publish_production(completed, &cancel)
        .await
        .unwrap()
        .snapshot;
    let guarded = catalog(store, Arc::clone(&reg), FixedBudget::new(128 << 20))
        .with_semantic_validator(Arc::new(GenericOnlyValidator));
    let parent = guarded
        .read_manifest(model.manifest_ref(), &AdmissionContext::default(), &cancel)
        .await
        .unwrap();
    let mut draft = stage_draft(&reg, &parent);
    draft.context.invocation = stage.invocation().cloned();
    let context = draft.context.clone();
    for result in [
        guarded.publish_bundle(draft, &cancel).await,
        guarded
            .read_manifest(stage.manifest_ref(), &context, &cancel)
            .await,
    ] {
        assert!(matches!(result, Err(CatalogError::Membership { reason })
            if reason.contains("executable validator") && reason.contains("producer")));
    }
}

#[derive(Debug)]
struct RejectingValidator;
impl SemanticValidator for RejectingValidator {
    fn validate<'a>(
        &'a self,
        _reg: &'a Registry,
        _rows: &'a BTreeMap<RelationKey, RecordBatch>,
        _session: &'a pse_catalog::session::SnapshotSession,
        _cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async {
            Err(CatalogError::UserModel {
                message: "actual invariant violation".to_owned(),
            })
        })
    }
}
#[tokio::test]
async fn semantic_refusal_cancellation_and_existing_corruption_cannot_move_head() {
    let reg = registry();
    let store = Arc::new(InMemory::new());
    let budget = FixedBudget::new(64 << 20);
    let catalog = catalog(Arc::clone(&store), Arc::clone(&reg), Arc::clone(&budget));
    let cancel = CancellationToken::default();
    let first = catalog
        .publish_bundle(draft(&reg, "value"), &cancel)
        .await
        .unwrap();
    let head = RefName::parse("head").unwrap();
    catalog
        .compare_and_swap_ref(&head, None, &first, &cancel)
        .await
        .unwrap();
    let path =
        object_store::path::Path::from(first.manifest().relations[0].encodings[0].path.as_str());
    store.put(&path, b"corrupt".to_vec().into()).await.unwrap();
    assert!(
        catalog
            .publish_bundle(draft(&reg, "value"), &cancel)
            .await
            .is_err()
    );
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(
        catalog
            .publish_bundle(draft(&reg, "next"), &cancelled)
            .await
            .is_err()
    );
    let catalog = catalog.with_semantic_validator(Arc::new(RejectingValidator));
    assert!(matches!(
        catalog.publish_bundle(draft(&reg, "next"), &cancel).await,
        Err(CatalogError::UserModel { .. })
    ));
    assert_eq!(
        catalog
            .read_ref(&head, &cancel)
            .await
            .unwrap()
            .unwrap()
            .manifest_ref(),
        first.manifest_ref()
    );
}

#[tokio::test]
async fn both_finished_encodings_are_admitted_and_compared_on_reopen() {
    let reg = registry();
    let store = Arc::new(InMemory::new());
    let budget = FixedBudget::new(64 << 20);
    let catalog = catalog(Arc::clone(&store), Arc::clone(&reg), budget);
    let cancel = CancellationToken::default();
    let mut offered = draft(&reg, "both");
    for relation in offered.relations.values_mut() {
        Arc::make_mut(&mut relation.contract).encodings = EncodingPolicy::IpcFileAndParquet;
    }
    let published = catalog.publish_bundle(offered, &cancel).await.unwrap();
    assert_eq!(published.manifest().relations[0].encodings.len(), 2);
    let reopened = catalog
        .read_manifest(
            published.manifest_ref(),
            &AdmissionContext::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(reopened.snapshot_id(), published.snapshot_id());
    assert_eq!(
        reopened.relation("authored", "items").unwrap().batch(),
        published.relation("authored", "items").unwrap().batch()
    );
}

async fn assert_change_metadata_ownership(
    store: &Arc<InMemory>,
    reg: &Arc<Registry>,
    reference: &pse_catalog::store::changes::ChangeSetRef,
) {
    let budget = FixedBudget::new(64 << 20);
    let reader = catalog(Arc::clone(store), Arc::clone(reg), budget.clone());
    let receipt = reader
        .read_change_set(reference, &CancellationToken::default())
        .await
        .unwrap();
    let used = budget.reserved();
    assert!(used > 0);
    let retained = receipt.clone();
    assert_eq!(budget.reserved(), used);
    drop(receipt);
    drop(reader);
    assert_eq!(
        budget.reserved(),
        used,
        "detached clone retains decoded metadata and artifact buffers"
    );
    assert_eq!(retained.artifacts().staged.len(), 1);
    drop(retained);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn manifest_metadata_has_a_finite_decode_claim_and_survives_detached_clones() {
    let reg = registry();
    let store = Arc::new(InMemory::new());
    let writer = catalog(
        Arc::clone(&store),
        Arc::clone(&reg),
        FixedBudget::new(64 << 20),
    );
    let cancel = CancellationToken::default();
    let snapshot = writer
        .publish_bundle(draft(&reg, "actual"), &cancel)
        .await
        .unwrap();
    let budget = FixedBudget::new(64 << 20);
    let reader = catalog(Arc::clone(&store), Arc::clone(&reg), budget.clone());
    let restored = reader
        .read_manifest(
            snapshot.manifest_ref(),
            &AdmissionContext::default(),
            &cancel,
        )
        .await
        .unwrap();
    let retained = restored.manifest().clone();
    let before_clone = budget.reserved();
    let another = retained.clone();
    assert_eq!(budget.reserved(), before_clone);
    drop((restored, reader, retained));
    assert!(
        budget.reserved() > 0,
        "manifest can outlive every snapshot and catalog"
    );
    assert_eq!(another.relations.len(), 1);
    drop(another);
    assert_eq!(budget.reserved(), 0);

    let mut hostile = snapshot.manifest().as_ref().clone();
    hostile.packages = (0..4096_u32)
        .map(|index| pse_catalog::PackageRef {
            package_id: pse_ids::SemanticId::from_bytes(u128::from(index).to_le_bytes()),
            version: "arbitrary input-derived package metadata".repeat(4),
            logical_hash: pse_ids::LogicalHash(ContentHash::NIL),
        })
        .collect();
    let bytes = hostile.encode().unwrap();
    let tiny = FixedBudget::new(bytes.len() * 2 + (128 << 10));
    let checksum = pse_ids::encoding_checksum(&bytes);
    store
        .put(&pse_catalog::manifest_path(&checksum), bytes.into())
        .await
        .unwrap();
    let reader = catalog(store, reg, tiny.clone());
    let result = reader
        .read_manifest(
            ManifestRef {
                snapshot_id: hostile.snapshot_id,
                manifest_checksum: checksum,
            },
            &AdmissionContext::default(),
            &cancel,
        )
        .await;
    assert!(
        matches!(result, Err(CatalogError::Reserve(pse_ids::ReserveError::Exhausted { owner, .. }))
        if owner == "store:manifest-metadata"),
        "metadata must reserve before serde's proportional container allocation"
    );
    assert_eq!(tiny.reserved(), 0);
}
