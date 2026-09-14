// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalog publication and reopening execute the actual authoring source projection.
#![allow(clippy::unwrap_used, reason = "fixed source correspondence fixture")]

#[path = "../../support/workflow_budget.rs"]
mod workflow_budget;

use object_store::memory::InMemory;
use pse_authoring::{ParseBudget, document::DocumentBundle};
use pse_catalog::{
    Catalog, CatalogError, CompilerRef, EncodingPolicy, FixedClock, Manifest, RelationContract,
    ToolchainRef, TrustLevel,
    store::{
        membership::{AdmissionContext, SemanticValidator},
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_ids::{
    CANON_VERSION, CancellationToken, ContentHash, FixedBudget, SNAPSHOT_PROFILE, SnapshotId,
    SnapshotKind,
};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, RelationKey, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc};

/// Deliberately permissive fixture producer. Generic invariant execution is covered
/// separately; this test isolates the compiler's production source projection hook.
#[derive(Debug)]
struct FixtureProducer;
impl SemanticValidator for FixtureProducer {
    fn validate<'a>(
        &'a self,
        _: &'a Registry,
        _: &'a BTreeMap<RelationKey, RecordBatch>,
        _: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async { Ok(()) })
    }
    fn validate_snapshot_sources<'a>(
        &'a self,
        _: &'a Catalog,
        _: SnapshotKind,
        _: &'a AdmissionContext,
        _: &'a BTreeMap<RelationKey, RecordBatch>,
        _: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async { Ok(()) })
    }
}

fn catalog(store: Arc<InMemory>, registry: Arc<Registry>, checked: bool) -> Catalog {
    let validator: Arc<dyn SemanticValidator> = if checked {
        Arc::new(pse_compiler::validator::CompilerValidator::new(Arc::new(
            FixtureProducer,
        )))
    } else {
        Arc::new(FixtureProducer)
    };
    Catalog::open(
        store,
        registry,
        TrustLevel::Owned,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES),
    )
    .with_semantic_validator(validator)
}

fn draft(registry: &Registry, bundle: &DocumentBundle, changed_name: bool) -> BundleDraft {
    let relations = registry
        .relations()
        .iter()
        .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
        .map(|spec| {
            let mut rows = bundle.rows.get(&spec.id).cloned().unwrap_or_default();
            if changed_name && spec.key.qualified_name() == "authored.packages" {
                let name = spec
                    .columns
                    .iter()
                    .position(|column| column.name == "name")
                    .unwrap();
                rows[0][name] = Cell::text("renamed_without_source_edit");
            }
            (
                pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id),
                RelationDraft {
                    contract: Arc::new(
                        RelationContract::from_spec(registry, spec, EncodingPolicy::IpcFile)
                            .unwrap(),
                    ),
                    batches: vec![
                        pse_relations::cells::batch_from_cells(registry, spec, &rows).unwrap(),
                    ],
                },
            )
        })
        .collect();
    BundleDraft {
        manifest: Manifest {
            manifest_version: pse_catalog::MANIFEST_VERSION.to_owned(),
            snapshot_kind: SnapshotKind::Model,
            snapshot_id: SnapshotId(ContentHash::NIL),
            membership_profile: SNAPSHOT_PROFILE.to_owned(),
            created_at: String::new(),
            schema_registry_fingerprint: registry.fingerprint(),
            relations: vec![],
            packages: vec![],
            compiler: CompilerRef {
                version: "source-projection-fixture".to_owned(),
                passes: vec![],
            },
            engine_profile: None,
            numerical_policy: None,
            toolchain: ToolchainRef {
                lockfile_hash: ContentHash::NIL,
                canonicalization: CANON_VERSION.to_owned(),
            },
            kernels: vec![],
            semantic_parents: vec![],
            evidence: vec![],
        },
        relations,
        context: AdmissionContext::default(),
    }
}

#[tokio::test]
async fn valid_content_identity_cannot_replace_actual_source_correspondence() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let source = pse_authoring::document::load_package_texts(
        BTreeMap::from([(
            "package.toml".to_owned(),
            include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
        )]),
        &registry,
        ParseBudget::default(),
    )
    .unwrap();
    let store = Arc::new(InMemory::new());
    let checked = catalog(Arc::clone(&store), Arc::clone(&registry), true);
    let fixture = catalog(store, Arc::clone(&registry), false);
    let cancel = CancellationToken::new();
    for document in &source.documents {
        checked
            .put_document(document.id, document.text.as_bytes(), &cancel)
            .await
            .unwrap();
    }
    let original = checked
        .publish_bundle(draft(&registry, &source, false), &cancel)
        .await
        .unwrap();
    checked
        .read_manifest(
            original.manifest_ref(),
            &AdmissionContext::default(),
            &cancel,
        )
        .await
        .unwrap();
    let error = checked
        .publish_bundle(draft(&registry, &source, true), &cancel)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("differ from exact source projection"),
        "{error}"
    );

    // Construct finished encodings and a manifest with valid recomputed hashes but
    // inconsistent actual source/row values through the explicit permissive fixture.
    let forged = fixture
        .publish_bundle(draft(&registry, &source, true), &cancel)
        .await
        .unwrap();
    let error = checked
        .read_manifest(forged.manifest_ref(), &AdmissionContext::default(), &cancel)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("differ from exact source projection"),
        "{error}"
    );
}
