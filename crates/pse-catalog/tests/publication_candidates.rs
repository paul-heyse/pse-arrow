// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The member-only publication API (Plan 22 O8.4): candidate composition admits a complete
//! record without making anything visible; a catalog selection reopens exactly its
//! members; an export manifest round-trips; catalog-driven collection and removal act on
//! any object store.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration assertions"
)]

use datafusion::{
    arrow::array::RecordBatch, common::ResolvedTableReference, execution::runtime_env::RuntimeEnv,
};
use pse_catalog::{
    artifact::{ArtifactPlan, RelationOutput},
    delta::{
        collect::{CollectAction, CollectTarget},
        manifest,
        publication::{Publication, PublicationSelection},
        scope::ReadScope,
    },
};
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineFactory, planner::UnifiedPlanner};
use pse_ids::SemanticId;
use pse_model::generated::identities::{AttemptId, PublicationId, WorkspaceId};
use pse_relations::generated::{
    authored::{entities, packages},
    enums::{EntityKind, IdPolicy, PackageKind, PublicationKind, RetentionReason},
    runtime::{publication_manifests, retained_versions},
    structures::MemberDescriptor,
};
use std::{collections::BTreeMap, sync::Arc};

fn identity(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

/// A native factory over a fresh pool, with a memory store registered at `root`.
fn fixture(root: &url::Url) -> (EngineFactory, Arc<RuntimeEnv>) {
    let fixture = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap()).unwrap();
    let runtime = fixture.resources.runtime.clone();
    runtime.register_object_store(root, Arc::new(object_store::memory::InMemory::new()));
    let factory = fixture
        .into_factory()
        .with_query_planner(Arc::new(UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
    (factory, runtime)
}

/// The declared rows of a package and one entity of it (or of a missing package).
fn sources(dangling: bool) -> BTreeMap<pse_schema::model::RelationKey, RecordBatch> {
    let mut package = packages::Builder::new().unwrap();
    package
        .push(packages::Row {
            package_id: identity(10).into(),
            name: "model".into(),
            version: "1.0.0".into(),
            kind: PackageKind::Model,
            id_policy: IdPolicy::Explicit,
            dependencies: vec![],
            content_hash: pse_ids::ContentHash::NIL,
            doc: String::new(),
        })
        .unwrap();
    let mut entity = entities::Builder::new().unwrap();
    entity
        .push(entities::Row {
            entity_id: identity(11),
            package_id: identity(if dangling { 99 } else { 10 }).into(),
            kind: EntityKind::Package,
            name: "model".into(),
            qualified_name: "model".into(),
            parent_entity_id: None,
            source_span: None,
        })
        .unwrap();
    BTreeMap::from([
        (
            packages::RELATION_KEY,
            package.finish().unwrap().into_batch(),
        ),
        (
            entities::RELATION_KEY,
            entity.finish().unwrap().into_batch(),
        ),
    ])
}

fn name(table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "artifact".into(),
        schema: "authored".into(),
        table: table.into(),
    }
}

fn header() -> publication_manifests::Row {
    publication_manifests::Row {
        publication_id: identity(2).into(),
        workspace_id: identity(1).into(),
        parent_publication_id: None,
        attempt_id: identity(3).into(),
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
        windows: vec![],
        exported_at: None,
        export_lease_id: None,
        export_expires_at: None,
        maintenance_epoch: None,
        store_fingerprint: None,
    }
}

/// Compose and execute the candidate of both relations under `root`.
async fn candidate(
    factory: &EngineFactory,
    root: &url::Url,
    dangling: bool,
) -> Result<publication_manifests::Row, pse_engine::EngineError> {
    let cancel = CancellationToken::new();
    let registry = pse_schema::shared_registry().unwrap();
    let session = factory
        .candidate(sources(dangling), Arc::clone(&registry), &cancel)
        .unwrap();
    let mut outputs = BTreeMap::new();
    let mut destinations = BTreeMap::new();
    for spec in [
        registry.relation("authored.packages").unwrap(),
        registry.relation("authored.entities").unwrap(),
    ] {
        let source = session.table_reference(&spec.key).unwrap().resolve("", "");
        outputs.insert(
            name(spec.key.name),
            RelationOutput {
                relation_id: spec.id,
                plan: session.relation_plan(&source).unwrap().plan().clone(),
            },
        );
        destinations.insert(
            name(spec.key.name),
            root.join(&format!(
                "members/attempt/publication/authored/{}/",
                spec.key.name
            ))
            .unwrap(),
        );
    }
    let artifact = ArtifactPlan::new(session, outputs, &cancel).unwrap();
    let (command, ticket) = artifact
        .prepare_publication(header(), destinations, vec![], &cancel)
        .unwrap();
    // The ticket exists before any effect and plans the complete member inventory.
    assert_eq!(ticket.publication_id(), PublicationId::from(identity(2)));
    assert_eq!(ticket.candidate().members.len(), 2);
    assert!(
        ticket
            .candidate()
            .members
            .iter()
            .all(|m| m.delta_version == 0)
    );
    let completed = command.execute(&cancel).await?;
    let batches = completed.batches();
    assert_eq!(batches.len(), 1);
    Ok(
        publication_manifests::View::try_from_batch_with_registry(&registry, &batches[0])
            .unwrap()
            .row(0)
            .unwrap(),
    )
}

#[tokio::test]
async fn candidate_returns_admitted_record_with_actual_versions() {
    let root = url::Url::parse("memory://candidates/").unwrap();
    let (factory, runtime) = fixture(&root);
    let record = candidate(&factory, &root, false).await.unwrap();
    assert_eq!(record.publication_id.as_id(), identity(2));
    assert_eq!(record.workspace_id, WorkspaceId::from(identity(1)));
    assert_eq!(record.attempt_id, AttemptId::from(identity(3)));
    assert_eq!(record.parent_publication_id, None);
    // Every member carries its actual committed version (creation, then data).
    assert_eq!(
        record
            .members
            .iter()
            .map(|m| (m.table_name.as_str(), m.delta_version))
            .collect::<Vec<_>>(),
        [("entities", 1), ("packages", 1)]
    );
    assert!(
        record
            .members
            .iter()
            .all(|m| m.table_uri.starts_with("memory://candidates/members/"))
    );
    assert!(record.exported_at.is_none() && record.windows.is_empty());
    // Nothing but the member tables was written: no control table, no head.
    let store = runtime.object_store_registry.get_store(&root).unwrap();
    let paths: Vec<String> = futures_util::TryStreamExt::try_collect::<Vec<_>>(store.list(None))
        .await
        .unwrap()
        .into_iter()
        .map(|meta| meta.location.to_string())
        .collect();
    assert!(!paths.is_empty());
    assert!(
        paths.iter().all(|path| path.starts_with("members/")),
        "{paths:?}"
    );
}

#[tokio::test]
async fn candidate_rejects_dangling_references_before_visibility() {
    let root = url::Url::parse("memory://dangling/").unwrap();
    let (factory, _) = fixture(&root);
    let error = candidate(&factory, &root, true).await.unwrap_err();
    assert!(
        format!("{error:?}").contains("violates its relation contract"),
        "{error:?}"
    );
}

#[derive(Debug)]
struct Owner;

#[tokio::test]
async fn selected_publication_reopens_exact_members() {
    let root = url::Url::parse("memory://selected/").unwrap();
    let (factory, _) = fixture(&root);
    let record = candidate(&factory, &root, false).await.unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let cancel = CancellationToken::new();
    let owner = Arc::new(Owner);
    let publication = Publication::open(
        PublicationSelection {
            record: record.clone(),
            scope: Some(ReadScope {
                workspace: identity(1).into(),
                epoch: 0,
            }),
            owner: Some(owner.clone()),
        },
        Arc::clone(&registry),
        &factory,
        &cancel,
    )
    .await
    .unwrap();
    assert_eq!(publication.record(), &record);
    for member in &record.members {
        assert_eq!(
            &publication.member(&name(&member.table_name)).unwrap(),
            member
        );
    }
    let rows: usize = publication
        .session()
        .sql("SELECT * FROM artifact.authored.entities", &cancel)
        .await
        .unwrap()
        .iter()
        .map(RecordBatch::num_rows)
        .sum();
    assert_eq!(rows, 1);
    // The selection's owner lives as long as the publication's session.
    assert!(Arc::strong_count(&owner) > 1);
    drop(publication);
    assert_eq!(Arc::strong_count(&owner), 1);
    // A selection of a version that does not exist is refused.
    let mut missing = record;
    missing.members[0].delta_version = 7;
    assert!(
        Publication::open(
            PublicationSelection {
                record: missing,
                scope: None,
                owner: None,
            },
            registry,
            &factory,
            &cancel,
        )
        .await
        .is_err()
    );
}

fn exported(record: publication_manifests::Row) -> publication_manifests::Row {
    publication_manifests::Row {
        exported_at: Some(1_790_000_000_000_000),
        export_lease_id: Some(identity(40).into()),
        export_expires_at: Some(1_790_000_600_000_000),
        maintenance_epoch: Some(4),
        store_fingerprint: Some(pse_ids::ContentHash::from_bytes([5; 32])),
        ..record
    }
}

#[tokio::test]
async fn manifest_round_trip() {
    let root = url::Url::parse("memory://manifest/").unwrap();
    let (factory, _) = fixture(&root);
    let record = exported(candidate(&factory, &root, false).await.unwrap());
    let registry = pse_schema::shared_registry().unwrap();
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(BTreeMap::new(), Arc::clone(&registry), &cancel)
        .unwrap();
    let destination = root.join("exports/one/").unwrap();
    manifest::prepare_manifest(&session, destination.clone(), record.clone(), &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let state = Arc::new(factory.native_state().clone());
    assert_eq!(
        manifest::read_manifest(destination.clone(), &registry, Arc::clone(&state))
            .await
            .unwrap(),
        record
    );
    // The destination is written once.
    assert!(
        manifest::prepare_manifest(&session, destination.clone(), record.clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
    // A record without its export fields is not a manifest.
    let mut unexported = record.clone();
    unexported.exported_at = None;
    assert!(
        manifest::prepare_manifest(
            &session,
            root.join("exports/two/").unwrap(),
            unexported,
            &cancel
        )
        .is_err()
    );
    // A manifest with a later version is refused.
    let table = pse_catalog::delta::provider::table_builder(destination.clone(), &state)
        .unwrap()
        .load()
        .await
        .unwrap();
    table
        .set_tbl_properties()
        .with_properties(std::collections::HashMap::new())
        .await
        .unwrap();
    let refused = manifest::read_manifest(destination, &registry, state)
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("version 1"), "{refused}");
}

#[tokio::test]
async fn manifest_refuses_legacy_control_table() {
    let root = url::Url::parse("memory://legacy/").unwrap();
    let (factory, _) = fixture(&root);
    let state = Arc::new(factory.native_state().clone());
    // A table recording the former control relation's contract identity.
    let control = root.join("control/").unwrap();
    let legacy = pse_schema::builder::registry_id("relation:runtime.publications@2");
    pse_catalog::delta::provider::table_builder(control.clone(), &state)
        .unwrap()
        .build()
        .unwrap()
        .create()
        .with_column(
            "workspace_id",
            deltalake::kernel::DataType::Primitive(deltalake::kernel::PrimitiveType::Binary),
            false,
            None,
        )
        .with_configuration([("pse.contract.id", Some(legacy.to_hex()))])
        .with_raise_if_key_not_exists(false)
        .await
        .unwrap();
    let error = manifest::read_manifest(control, &pse_schema::shared_registry().unwrap(), state)
        .await
        .unwrap_err();
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
    let mut migration = false;
    while let Some(current) = cause {
        migration |= matches!(
            current.downcast_ref::<pse_schema::compatibility::CompatibilityError>(),
            Some(pse_schema::compatibility::CompatibilityError::MigrationRequired(_))
        );
        cause = current.source();
    }
    assert!(migration, "{error:?}");
}

/// A declared table of `authored.packages` under `location` with versions 0 (creation)
/// through `appends`.
async fn declared_table(factory: &EngineFactory, location: &url::Url, appends: usize) {
    let registry = pse_schema::shared_registry().unwrap();
    let cancel = CancellationToken::new();
    let spec = registry.relation("authored.packages").unwrap();
    for append in 0..appends {
        let session = factory
            .candidate(
                BTreeMap::from([(
                    packages::RELATION_KEY,
                    sources(false)[&packages::RELATION_KEY].clone(),
                )]),
                Arc::clone(&registry),
                &cancel,
            )
            .unwrap()
            .with_purpose(pse_schema::model::provider::OperationPurpose::Publish);
        let source = session.table_reference(&spec.key).unwrap().resolve("", "");
        let table = pse_catalog::delta::provider::table_builder(
            location.clone(),
            &session.bound_state().unwrap(),
        )
        .unwrap();
        let table = if append == 0 {
            table.build().unwrap()
        } else {
            table.load().await.unwrap()
        };
        let input = pse_engine::session::output::declare_relation_output(
            session.relation_plan(&source).unwrap().plan().clone(),
            &registry,
            spec,
        )
        .unwrap();
        let plan = pse_catalog::delta::write::DeltaWrite::declared(
            table,
            input,
            deltalake::protocol::SaveMode::Append,
            deltalake::kernel::transaction::CommitProperties::default(),
            pse_catalog::delta::contract::DeclaredCheck::new(&registry, spec.id).unwrap(),
        )
        .unwrap();
        let mut session = session;
        session.bind_target(pse_schema::model::provider::ProviderScope::Table(
            "artifact".into(),
            "authored".into(),
            "packages".into(),
        ));
        session
            .prepare(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
    }
}

fn range(
    table: &url::Url,
    from: i64,
    through: i64,
    reason: RetentionReason,
) -> retained_versions::Row {
    retained_versions::Row {
        table_uri: table.to_string(),
        from_version: from,
        through_version: through,
        reason,
    }
}

async fn rows_at(factory: &EngineFactory, table: &url::Url, version: i64) -> usize {
    let registry = pse_schema::shared_registry().unwrap();
    let spec = registry.relation("authored.packages").unwrap();
    let member = MemberDescriptor {
        catalog_name: "artifact".into(),
        schema_name: "authored".into(),
        table_name: "packages".into(),
        relation_id: spec.id,
        relation_version: i64::from(spec.key.version),
        contract_fingerprint: spec.fingerprint,
        table_uri: table.to_string(),
        delta_version: version,
        selection: pse_relations::generated::structures::MemberDescriptorSelection::from_full(),
    };
    let mut record = header();
    record.members = vec![member];
    let cancel = CancellationToken::new();
    let publication = Publication::open(
        PublicationSelection {
            record,
            scope: None,
            owner: None,
        },
        registry,
        factory,
        &cancel,
    )
    .await
    .unwrap();
    publication
        .session()
        .sql("SELECT * FROM artifact.authored.packages", &cancel)
        .await
        .unwrap()
        .iter()
        .map(RecordBatch::num_rows)
        .sum()
}

#[tokio::test]
async fn collect_keeps_protected_versions_on_memory_store() {
    let root = url::Url::parse("memory://collect/").unwrap();
    let (factory, runtime) = fixture(&root);
    let table = root.join("members/a/p/authored/packages/").unwrap();
    declared_table(&factory, &table, 3).await;
    let registry = pse_schema::shared_registry().unwrap();
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(BTreeMap::new(), Arc::clone(&registry), &cancel)
        .unwrap();
    let target = CollectTarget {
        reference: name("packages"),
        location: table.clone(),
        action: CollectAction::Collect,
        log_cutoff_ms: 0,
    };
    let other = root.join("members/b/q/authored/packages/").unwrap();
    let outcome = pse_catalog::delta::collect::prepare_collect(
        &session,
        target.clone(),
        vec![
            range(&table, 1, 1, RetentionReason::Publication),
            range(&table, 2, 3, RetentionReason::Changes),
            // Another table's range is ignored; an attempt range beyond the history
            // keeps what exists.
            range(&other, 0, 9, RetentionReason::Publication),
            range(
                &root.join("members/a/").unwrap(),
                0,
                i64::MAX,
                RetentionReason::Attempt,
            ),
        ],
        &cancel,
    )
    .unwrap()
    .execute(&cancel)
    .await
    .unwrap();
    let outcome = pse_relations::generated::runtime::maintenance_outcomes::View::try_from_batch_with_registry(
        &registry,
        &outcome.batches()[0],
    )
    .unwrap()
    .row(0)
    .unwrap();
    assert_eq!(outcome.table_uri, table.to_string());
    // The fence advanced the table; the protected versions still read exactly.
    assert!(outcome.delta_version > 3);
    assert_eq!(rows_at(&factory, &table, 1).await, 1);
    assert_eq!(rows_at(&factory, &table, 3).await, 3);
    // A protected version whose history is gone is refused before any effect.
    let store = runtime.object_store_registry.get_store(&root).unwrap();
    let before = pse_catalog::delta::provider::table_builder(table.clone(), factory.native_state())
        .unwrap()
        .load()
        .await
        .unwrap()
        .version();
    let log = object_store::path::Path::from_url_path(
        table
            .join("_delta_log/00000000000000000001.json")
            .unwrap()
            .path(),
    )
    .unwrap();
    object_store::ObjectStoreExt::delete(store.as_ref(), &log)
        .await
        .unwrap();
    assert!(
        pse_catalog::delta::collect::prepare_collect(
            &session,
            target,
            vec![range(&table, 1, 1, RetentionReason::Publication)],
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .is_err()
    );
    let after = pse_catalog::delta::provider::table_builder(table, factory.native_state())
        .unwrap()
        .load()
        .await
        .unwrap()
        .version();
    assert_eq!(before, after);
}

#[tokio::test]
async fn remove_tables_idempotent_on_fault_store() {
    let root = url::Url::parse("memory://remove/").unwrap();
    let fixture = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap()).unwrap();
    let runtime = fixture.resources.runtime.clone();
    let store =
        pse_testkit::fault_store::FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    runtime.register_object_store(&root, store.clone());
    let factory = fixture
        .into_factory()
        .with_query_planner(Arc::new(UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
    let table = root.join("members/a/p/authored/packages/").unwrap();
    let kept = root.join("members/b/q/authored/packages/").unwrap();
    declared_table(&factory, &table, 2).await;
    declared_table(&factory, &kept, 1).await;
    let state = factory.native_state();
    let removed = pse_catalog::delta::collect::remove_tables(std::slice::from_ref(&table), state)
        .await
        .unwrap();
    assert!(removed > 0);
    let listed = |prefix: &url::Url| {
        let store = store.clone();
        let prefix = object_store::path::Path::from_url_path(prefix.path()).unwrap();
        async move {
            futures_util::TryStreamExt::try_collect::<Vec<_>>(object_store::ObjectStore::list(
                store.as_ref(),
                Some(&prefix),
            ))
            .await
            .unwrap()
            .len()
        }
    };
    assert_eq!(listed(&table).await, 0);
    assert!(listed(&kept).await > 0);
    // Removing again, or a table that never existed, removes nothing and succeeds.
    assert_eq!(
        pse_catalog::delta::collect::remove_tables(std::slice::from_ref(&table), state)
            .await
            .unwrap(),
        0
    );
    let never = root.join("members/never/").unwrap();
    assert_eq!(
        pse_catalog::delta::collect::remove_prefix(&never, state)
            .await
            .unwrap(),
        0
    );
    assert_eq!(rows_at(&factory, &kept, 1).await, 1);
}
