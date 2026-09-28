// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Publication through the operational catalog end to end (Plan 22 O8.5): durable
//! attempts publish composed artifacts into registered workspaces in isolated PostgreSQL
//! databases; readers hold catalog leases; retirement, collection and exports follow the
//! catalog. `concurrent_publishers_one_winner_no_lost_update` re-executes this test
//! binary as two publisher processes. Run with `just publication-test`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "integration assertions and the child-process protocol"
)]

use datafusion::{arrow::array::RecordBatch, common::ResolvedTableReference};
use pse_catalog::{
    artifact::{ArtifactPlan, RelationOutput},
    delta::publication::Publication,
};
use pse_columnar::CancellationToken;
use pse_operations::{
    OperationsError,
    attempts::{AttemptId, AttemptKind, NewAttempt, TransitionNote},
    catalog::PublicationId,
    lifecycle::AttemptState,
    testing::{Fault, FaultPoint, FaultProxy, TestDatabase},
};
use pse_relations::generated::{
    authored::{entities, packages},
    enums::{EntityKind, IdPolicy, PackageKind, PublicationKind},
};
use pse_runtime::{
    SharedRuntime,
    workflow::{
        Durability, LeasePolicy, Operations, PublicationAttempt, PublicationSettlement, Published,
        Runtime, WorkflowError, Workspace, open_export, prepare_artifact_publication,
    },
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc, time::Duration};

/// An ephemeral runtime over its own shared deployment.
fn ephemeral() -> Runtime {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(1 << 30),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: pse_runtime::DeltaCacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            worker_bytes: 128 << 20,
            workspace_bytes: 128 << 20,
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

async fn durable(url: &str, worker: &str) -> Runtime {
    let operations = Operations::connect(url, worker, LeasePolicy::default())
        .await
        .unwrap();
    ephemeral().with_durability(Durability::Durable(operations))
}

fn operations(runtime: &Runtime) -> &Operations {
    match runtime.durability() {
        Durability::Durable(operations) => operations,
        Durability::Ephemeral => panic!("an ephemeral runtime"),
    }
}

/// A durable attempt driven to `completed`, as a finished run's would be.
async fn finished_attempt(runtime: &Runtime) -> AttemptId {
    let attempts = operations(runtime).store().attempts();
    let attempt = NewAttempt {
        attempt_id: pse_operations::mint_id(),
        run_id: pse_operations::mint_id(),
        kind: AttemptKind::Simulation,
        request_identity: pse_ids::ContentHash::from_bytes([1; 32]),
        preparation_identity: None,
        parent_attempt: None,
    };
    attempts.create(&attempt, Some("test")).await.unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("test"),
        )
        .await
        .unwrap();
    attempts
        .start(attempt.attempt_id, "test", Duration::from_secs(30))
        .await
        .unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Completed,
            &TransitionNote::by("test"),
        )
        .await
        .unwrap();
    attempt.attempt_id
}

fn identity(byte: u8) -> pse_ids::SemanticId {
    pse_ids::SemanticId::from_bytes([byte; 16])
}

fn name(schema: &str, table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "artifact".into(),
        schema: schema.into(),
        table: table.into(),
    }
}

/// A package and one entity of it, published as `artifact.authored.*`.
fn artifact(runtime: &Runtime, package: &str) -> ArtifactPlan {
    let cancel = CancellationToken::new();
    let registry = runtime.registry().clone();
    let mut packages = packages::Builder::new().unwrap();
    packages
        .push(packages::Row {
            package_id: identity(10).into(),
            name: package.into(),
            version: "1.0.0".into(),
            kind: PackageKind::Model,
            id_policy: IdPolicy::Explicit,
            dependencies: vec![],
            content_hash: pse_ids::ContentHash::NIL,
            doc: String::new(),
        })
        .unwrap();
    let mut entities = entities::Builder::new().unwrap();
    entities
        .push(entities::Row {
            entity_id: identity(11),
            package_id: identity(10).into(),
            kind: EntityKind::Package,
            name: package.into(),
            qualified_name: package.into(),
            parent_entity_id: None,
            source_span: None,
        })
        .unwrap();
    let session = runtime
        .sessions()
        .candidate(
            BTreeMap::from([
                (
                    packages::RELATION_KEY,
                    packages.finish().unwrap().into_batch(),
                ),
                (
                    entities::RELATION_KEY,
                    entities.finish().unwrap().into_batch(),
                ),
            ]),
            Arc::clone(&registry),
            &cancel,
        )
        .unwrap();
    let mut outputs = BTreeMap::new();
    for spec in [
        registry.relation("authored.packages").unwrap(),
        registry.relation("authored.entities").unwrap(),
    ] {
        let source = session.table_reference(&spec.key).unwrap().resolve("", "");
        outputs.insert(
            name("authored", spec.key.name),
            RelationOutput {
                relation_id: spec.id,
                plan: session.relation_plan(&source).unwrap().plan().clone(),
            },
        );
    }
    ArtifactPlan::new(session, outputs, &cancel).unwrap()
}

fn prepare(
    runtime: &Runtime,
    attempt: AttemptId,
    workspace: &Workspace,
    parent: Option<PublicationId>,
    publication: Option<PublicationId>,
    package: &str,
) -> PublicationAttempt {
    prepare_artifact_publication(
        runtime,
        attempt,
        workspace,
        parent,
        publication,
        PublicationKind::Relations,
        artifact(runtime, package),
        vec![],
        &CancellationToken::new(),
    )
    .unwrap()
}

async fn publish(
    runtime: &Runtime,
    workspace: &Workspace,
    parent: Option<PublicationId>,
    package: &str,
) -> Published {
    let attempt = finished_attempt(runtime).await;
    prepare(runtime, attempt, workspace, parent, None, package)
        .commit(&CancellationToken::new())
        .await
        .unwrap()
}

/// The rows of one member of an opened publication.
async fn rows(publication: &Publication, schema: &str, table: &str) -> usize {
    publication
        .session()
        .sql(
            &format!("SELECT * FROM artifact.{schema}.{table}"),
            &CancellationToken::new(),
        )
        .await
        .unwrap()
        .iter()
        .map(RecordBatch::num_rows)
        .sum()
}

fn directory() -> (tempfile::TempDir, url::Url) {
    let directory = tempfile::tempdir().unwrap();
    let root = url::Url::from_directory_path(directory.path()).unwrap();
    (directory, root)
}

/// The latest committed Delta version of a table; `None` once it is removed.
async fn version(runtime: &Runtime, table: &str) -> Option<i64> {
    let table = pse_catalog::delta::provider::table_builder(
        url::Url::parse(table).unwrap(),
        runtime.sessions().native_state(),
    )
    .unwrap()
    .load()
    .await
    .ok()?;
    table
        .version()
        .map(|version| i64::try_from(version).unwrap())
}

fn member_uri(record: &pse_operations::catalog::PublicationRecord, table: &str) -> String {
    record
        .members
        .iter()
        .find(|m| m.table_name == table)
        .unwrap()
        .table_uri
        .clone()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ChildReport {
    publication: PublicationId,
    first: String,
    head_at_conflict: Option<PublicationId>,
    member_versions: Vec<Option<i64>>,
}

/// One publisher process: prepare against the base and commit; on a conflict re-prepare
/// the same intent against the head and commit again.
async fn publisher_child(request: &serde_json::Value) {
    let runtime = durable(request["url"].as_str().unwrap(), "publisher").await;
    let workspace: Workspace = serde_json::from_value(request["workspace"].clone()).unwrap();
    let base: PublicationId = serde_json::from_value(request["base"].clone()).unwrap();
    let package = request["package"].as_str().unwrap();
    let attempt = finished_attempt(&runtime).await;
    let prepared = prepare(&runtime, attempt, &workspace, Some(base), None, package);
    let publication = prepared.publication_id;
    let prefix = prepared.member_prefix.clone();
    let cancel = CancellationToken::new();
    let (first, head_at_conflict) = match prepared.commit(&cancel).await {
        Ok(_) => ("committed".to_owned(), None),
        Err(WorkflowError::Operations(OperationsError::PublicationConflict {
            current, ..
        })) => {
            prepare(
                &runtime,
                attempt,
                &workspace,
                current,
                Some(publication),
                package,
            )
            .commit(&cancel)
            .await
            .unwrap();
            ("conflict".to_owned(), current)
        }
        Err(error) => panic!("{error:?}"),
    };
    let mut member_versions = Vec::new();
    for table in ["entities", "packages"] {
        let location = prefix.join(&format!("authored/{table}/")).unwrap();
        member_versions.push(version(&runtime, location.as_str()).await);
    }
    println!(
        "PUBLISHER {}",
        serde_json::to_string(&ChildReport {
            publication,
            first,
            head_at_conflict,
            member_versions,
        })
        .unwrap()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_publishers_one_winner_no_lost_update() {
    if let Ok(request) = std::env::var("PSE_PUBLISHER_CHILD") {
        return publisher_child(&serde_json::from_str(&request).unwrap()).await;
    }
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "parent").await;
    let (_directory, root) = directory();
    let workspace = runtime
        .register_workspace("concurrent", root)
        .await
        .unwrap();
    let base = publish(&runtime, &workspace, None, "base").await;
    let children = ["left", "right"]
        .into_iter()
        .map(|package| {
            let request = serde_json::json!({
                "url": database.url(),
                "workspace": workspace,
                "base": base.publication_id,
                "package": package,
            });
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "concurrent_publishers_one_winner_no_lost_update",
                    "--nocapture",
                ])
                .env("PSE_PUBLISHER_CHILD", request.to_string())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect::<Vec<_>>();
    let reports = tokio::task::spawn_blocking(move || {
        children
            .into_iter()
            .map(|child| {
                let output = child.wait_with_output().unwrap();
                let stdout = String::from_utf8(output.stdout).unwrap();
                assert!(output.status.success(), "{stdout}");
                let line = stdout
                    .lines()
                    .find_map(|line| line.strip_prefix("PUBLISHER "))
                    .unwrap_or_else(|| panic!("no report in {stdout}"));
                serde_json::from_str::<ChildReport>(line).unwrap()
            })
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();
    assert_eq!(
        reports.iter().filter(|r| r.first == "committed").count(),
        1,
        "exactly one winner: {reports:?}"
    );
    let winner = reports.iter().find(|r| r.first == "committed").unwrap();
    let loser = reports.iter().find(|r| r.first == "conflict").unwrap();
    // The loser met the winner as the head and re-prepared with the same intent without
    // rewriting its members (creation, then data: version 1).
    assert_eq!(loser.head_at_conflict, Some(winner.publication));
    assert_eq!(loser.member_versions, [Some(1), Some(1)]);
    assert_eq!(winner.member_versions, [Some(1), Some(1)]);
    // No lost update: the head chain holds both, the loser on the winner on the base.
    assert_eq!(
        runtime.head(workspace.workspace_id).await.unwrap(),
        Some(loser.publication)
    );
    let catalog = operations(&runtime).store().catalog();
    let parent_of = |record: Option<pse_operations::catalog::PublicationRecord>| {
        record.unwrap().publication.parent_publication
    };
    assert_eq!(
        parent_of(catalog.publication(loser.publication).await.unwrap()),
        Some(winner.publication)
    );
    assert_eq!(
        parent_of(catalog.publication(winner.publication).await.unwrap()),
        Some(base.publication_id)
    );
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lost_ack_settles_via_catalog() {
    let database = TestDatabase::create().await.unwrap();
    let proxy = FaultProxy::start(database.url()).await.unwrap();
    let runtime = durable(proxy.url(), "lossy").await;
    let (_directory, root) = directory();
    let workspace = runtime.register_workspace("lossy", root).await.unwrap();
    let cancel = CancellationToken::new();
    let fault = |point| Fault {
        marker: "INSERT INTO pse_ops.publications ",
        point,
    };

    // The server commits but the acknowledgement is lost: unresolved, then settled as
    // committed by the catalog.
    let attempt = finished_attempt(&runtime).await;
    let prepared = prepare(&runtime, attempt, &workspace, None, None, "first");
    let (ticket, first) = (prepared.ticket.clone(), prepared.publication_id);
    proxy.arm(fault(FaultPoint::AfterCommit));
    let lost = prepared.commit(&cancel).await.unwrap_err();
    assert!(proxy.fired());
    assert!(
        matches!(lost, WorkflowError::PublicationUnresolved { publication, .. } if publication == first),
        "{lost:?}"
    );
    assert_eq!(
        runtime.settle_publication(&ticket).await,
        PublicationSettlement::Committed {
            publication_id: first
        }
    );
    assert_eq!(
        runtime.head(workspace.workspace_id).await.unwrap(),
        Some(first)
    );

    // The connection drops before COMMIT: unresolved, then proved not committed; the same
    // intent then commits, recovering the members it wrote.
    let attempt = finished_attempt(&runtime).await;
    let prepared = prepare(&runtime, attempt, &workspace, Some(first), None, "second");
    let (ticket, second) = (prepared.ticket.clone(), prepared.publication_id);
    proxy.arm(fault(FaultPoint::BeforeCommit));
    let lost = prepared.commit(&cancel).await.unwrap_err();
    assert!(proxy.fired());
    assert!(
        matches!(lost, WorkflowError::PublicationUnresolved { .. }),
        "{lost:?}"
    );
    assert_eq!(
        runtime.settle_publication(&ticket).await,
        PublicationSettlement::ProvedNoncommit
    );
    let retried = prepare(
        &runtime,
        attempt,
        &workspace,
        Some(first),
        Some(second),
        "second",
    );
    assert_eq!(retried.ticket, ticket);
    retried.commit(&cancel).await.unwrap();
    assert_eq!(
        runtime.settle_publication(&ticket).await,
        PublicationSettlement::Committed {
            publication_id: second
        }
    );
    // A ticket never registered proves nothing was committed; once its intent is
    // registered against a parent that is no longer the head, it can never commit as
    // prepared.
    let attempt = finished_attempt(&runtime).await;
    let stale = prepare(&runtime, attempt, &workspace, Some(first), None, "stale");
    let stale_ticket = stale.ticket.clone();
    assert_eq!(
        runtime.settle_publication(&stale_ticket).await,
        PublicationSettlement::ProvedNoncommit
    );
    let conflict = stale.commit(&cancel).await.unwrap_err();
    assert!(
        matches!(
            conflict,
            WorkflowError::Operations(OperationsError::PublicationConflict { current: Some(head), .. })
                if head == second
        ),
        "{conflict:?}"
    );
    let settled = runtime.settle_publication(&stale_ticket).await;
    assert!(
        matches!(&settled, PublicationSettlement::Conflict { head: Some(head), .. } if *head == second),
        "{settled:?}"
    );
    // Without the catalog nothing can be concluded.
    database.remove().await.unwrap();
    assert!(matches!(
        runtime.settle_publication(&ticket).await,
        PublicationSettlement::Unresolved { .. }
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn maintenance_waits_for_reader_leases() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "maintainer").await;
    let root = url::Url::parse("memory://maintenance/workspace/").unwrap();
    let store = Arc::new(object_store::memory::InMemory::new());
    runtime
        .sessions()
        .native_state()
        .runtime_env()
        .register_object_store(&root, store.clone());
    let workspace = runtime.register_workspace("memory", root).await.unwrap();
    let old = publish(&runtime, &workspace, None, "old").await;
    let head = publish(&runtime, &workspace, Some(old.publication_id), "head").await;
    let cancel = CancellationToken::new();

    let reader = runtime.open(old.publication_id, &cancel).await.unwrap();
    assert_eq!(rows(reader.publication(), "authored", "entities").await, 1);
    // The reader holds its lease: retirement marks the publication expiring and waits.
    let busy = runtime
        .retire_publications(
            workspace.workspace_id,
            &[old.publication_id],
            Duration::from_millis(200),
            &cancel,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(
            busy,
            WorkflowError::Operations(OperationsError::ReadersActive { .. })
        ),
        "{busy:?}"
    );
    // No new reader is admitted once it is expiring; the existing one still reads.
    let refused = runtime.open(old.publication_id, &cancel).await.unwrap_err();
    assert!(
        matches!(
            refused,
            WorkflowError::Operations(OperationsError::PublicationRetiring { .. })
        ),
        "{refused:?}"
    );
    assert_eq!(rows(reader.publication(), "authored", "packages").await, 1);
    // The reader finishes; the retirement completes and removes only its tables.
    drop(reader);
    let retired = runtime
        .retire_publications(
            workspace.workspace_id,
            &[old.publication_id],
            Duration::from_secs(10),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(retired.retired, [old.publication_id]);
    assert_eq!(retired.removed_tables.len(), 2);
    assert!(retired.removed_objects > 0);
    let remaining: Vec<String> = futures_util::TryStreamExt::try_collect::<Vec<_>>(
        object_store::ObjectStore::list(store.as_ref(), None),
    )
    .await
    .unwrap()
    .into_iter()
    .map(|meta| meta.location.to_string())
    .collect();
    let (old_id, head_id) = (
        old.publication_id.to_string(),
        head.publication_id.to_string(),
    );
    assert!(remaining.iter().any(|path| path.contains(&head_id)));
    assert!(
        remaining.iter().all(|path| !path.contains(&old_id)),
        "{remaining:?}"
    );
    // The head is protected, and still reads.
    let protected = runtime
        .retire_publications(
            workspace.workspace_id,
            &[head.publication_id],
            Duration::from_millis(10),
            &cancel,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(
            protected,
            WorkflowError::Operations(OperationsError::ProtectedPublication { .. })
        ),
        "{protected:?}"
    );
    let current = runtime
        .open_head(workspace.workspace_id, &cancel)
        .await
        .unwrap();
    assert_eq!(current.publication_id(), head.publication_id);
    assert_eq!(rows(current.publication(), "authored", "entities").await, 1);
    drop((current, runtime));
    database.remove().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn catalog_protects_published_versions() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "protector").await;
    let (_directory, root) = directory();
    let workspace = runtime.register_workspace("protected", root).await.unwrap();
    let cancel = CancellationToken::new();
    let first = publish(&runtime, &workspace, None, "first").await;

    // The second publication derives from the first's packages: its exact input.
    let reader = runtime.open(first.publication_id, &cancel).await.unwrap();
    let session = reader.publication().session().clone();
    let spec = runtime.registry().relation("authored.packages").unwrap();
    let plan = session
        .relation_plan(&name("authored", "packages"))
        .unwrap()
        .plan()
        .clone();
    let derived = ArtifactPlan::new(
        session,
        BTreeMap::from([(
            name("derived", "packages"),
            RelationOutput {
                relation_id: spec.id,
                plan,
            },
        )]),
        &cancel,
    )
    .unwrap();
    let attempt = finished_attempt(&runtime).await;
    let second = prepare_artifact_publication(
        &runtime,
        attempt,
        &workspace,
        Some(first.publication_id),
        None,
        PublicationKind::Relations,
        derived,
        vec![],
        &cancel,
    )
    .unwrap();
    assert_eq!(second.ticket.candidate().inputs.len(), 1);
    let second = second.commit(&cancel).await.unwrap();
    drop(reader);
    let third = publish(&runtime, &workspace, Some(second.publication_id), "third").await;
    let catalog = operations(&runtime).store().catalog();
    let first_record = catalog
        .publication(first.publication_id)
        .await
        .unwrap()
        .unwrap();
    let second_record = catalog
        .publication(second.publication_id)
        .await
        .unwrap()
        .unwrap();
    let packages = member_uri(&first_record, "packages");
    let entities = member_uri(&first_record, "entities");
    let derived = member_uri(&second_record, "packages");
    assert_eq!(second_record.inputs.len(), 1);
    assert_eq!(second_record.inputs[0].table_uri, packages);

    // Collection maintains every selected table and keeps each protected version.
    let collected = runtime
        .collect(workspace.workspace_id, &cancel)
        .await
        .unwrap();
    assert!(collected.maintenance_epoch > 0);
    assert!(collected.tables.iter().any(|t| t.table_uri == packages));
    assert!(collected.tables.iter().any(|t| t.table_uri == derived));
    let reread = runtime.open(first.publication_id, &cancel).await.unwrap();
    assert_eq!(rows(reread.publication(), "authored", "packages").await, 1);
    assert_eq!(rows(reread.publication(), "authored", "entities").await, 1);
    drop(reread);

    // Retiring the first keeps the table the second reads.
    let retired = runtime
        .retire_publications(
            workspace.workspace_id,
            &[first.publication_id],
            Duration::from_secs(10),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(retired.removed_tables, [entities.clone()]);
    assert_eq!(version(&runtime, &entities).await, None);
    assert!(version(&runtime, &packages).await.is_some());
    let reader = runtime.open(second.publication_id, &cancel).await.unwrap();
    assert_eq!(rows(reader.publication(), "derived", "packages").await, 1);
    drop(reader);
    // The input stays maintained while the second selects it.
    let collected = runtime
        .collect(workspace.workspace_id, &cancel)
        .await
        .unwrap();
    assert!(collected.tables.iter().any(|t| t.table_uri == packages));
    assert!(!collected.tables.iter().any(|t| t.table_uri == entities));

    // An export is a lease: the exported publication is not retired until it is released.
    let (_exports, exports) = directory();
    let receipt = runtime
        .export_publication(
            second.publication_id,
            exports.join("second/").unwrap(),
            Duration::from_secs(600),
            &cancel,
        )
        .await
        .unwrap();
    let busy = runtime
        .retire_publications(
            workspace.workspace_id,
            &[second.publication_id],
            Duration::from_millis(100),
            &cancel,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(
            busy,
            WorkflowError::Operations(OperationsError::ReadersActive { .. })
        ),
        "{busy:?}"
    );
    assert!(runtime.release_export(&receipt).await.unwrap());
    let retired = runtime
        .retire_publications(
            workspace.workspace_id,
            &[second.publication_id],
            Duration::from_secs(10),
            &cancel,
        )
        .await
        .unwrap();
    // Nothing undeleted selects the first's packages any more: it goes with the second.
    let mut expected = vec![derived.clone(), packages.clone()];
    expected.sort();
    assert_eq!(retired.removed_tables, expected);
    assert_eq!(version(&runtime, &packages).await, None);
    assert_eq!(
        runtime.head(workspace.workspace_id).await.unwrap(),
        Some(third.publication_id)
    );
    drop(runtime);
    database.remove().await.unwrap();
}

/// Whether a failure's chain holds a migration-required compatibility refusal.
fn migration_required(error: &WorkflowError) -> bool {
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(current) = cause {
        if matches!(
            current.downcast_ref::<pse_schema::compatibility::CompatibilityError>(),
            Some(pse_schema::compatibility::CompatibilityError::MigrationRequired(_))
        ) {
            return true;
        }
        cause = current.source();
    }
    format!("{error:?}").contains("MigrationRequired")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exported_publication_opens_offline() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "exporter").await;
    let (_directory, root) = directory();
    let workspace = runtime
        .register_workspace("exports", root.clone())
        .await
        .unwrap();
    let published = publish(&runtime, &workspace, None, "exported").await;
    let cancel = CancellationToken::new();
    let lasting = root.join("exports/lasting/").unwrap();
    let receipt = runtime
        .export_publication(
            published.publication_id,
            lasting.clone(),
            Duration::from_secs(3600),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(receipt.publication_id, published.publication_id);
    let brief = root.join("exports/brief/").unwrap();
    runtime
        .export_publication(
            published.publication_id,
            brief.clone(),
            Duration::from_millis(1),
            &cancel,
        )
        .await
        .unwrap();
    // Offline: no operational store at all.
    drop(runtime);
    database.remove().await.unwrap();
    let offline = ephemeral();
    let publication = open_export(
        lasting,
        offline.registry().clone(),
        offline.sessions(),
        &cancel,
    )
    .await
    .unwrap();
    let record = publication.record();
    assert_eq!(record.publication_id, published.publication_id);
    assert_eq!(record.export_lease_id, Some(receipt.lease_id));
    assert_eq!(record.export_expires_at, Some(receipt.expires_at));
    assert_eq!(record.members.len(), 2);
    assert_eq!(rows(&publication, "authored", "entities").await, 1);
    // An expired export is refused.
    tokio::time::sleep(Duration::from_millis(20)).await;
    let expired = open_export(
        brief,
        offline.registry().clone(),
        offline.sessions(),
        &cancel,
    )
    .await
    .unwrap_err();
    assert!(
        matches!(expired, WorkflowError::ExportLeaseExpired { .. }),
        "{expired:?}"
    );
    // A former Delta control table is refused as a historical format.
    let legacy = root.join("legacy/control/").unwrap();
    pse_catalog::delta::provider::table_builder(legacy.clone(), offline.sessions().native_state())
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
        .with_configuration([(
            "pse.contract.id",
            Some(pse_schema::builder::registry_id("relation:runtime.publications@2").to_hex()),
        )])
        .with_raise_if_key_not_exists(false)
        .await
        .unwrap();
    let refused = open_export(
        legacy,
        offline.registry().clone(),
        offline.sessions(),
        &cancel,
    )
    .await
    .unwrap_err();
    assert!(migration_required(&refused), "{refused:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn publication_uses_durable_attempt_identity() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "identities").await;
    let (_directory, root) = directory();
    let workspace = runtime
        .register_workspace("identities", root)
        .await
        .unwrap();
    let attempt = finished_attempt(&runtime).await;
    let prepared = prepare(&runtime, attempt, &workspace, None, None, "identity");
    // The publication attempt is the durable attempt, from the ticket on.
    assert_eq!(prepared.attempt_id, attempt);
    assert_eq!(AttemptId::from(prepared.ticket.attempt_id()), attempt);
    let publication = prepared.publication_id;
    let prefix = workspace.member_prefix(attempt, publication).unwrap();
    assert_eq!(prepared.member_prefix, prefix);
    assert!(
        prepared
            .ticket
            .candidate()
            .members
            .iter()
            .all(|m| m.table_uri.starts_with(prefix.as_str()))
    );
    let published = prepared.commit(&CancellationToken::new()).await.unwrap();
    assert_eq!(published.attempt_id, attempt);
    let catalog = operations(&runtime).store().catalog();
    let record = catalog.publication(publication).await.unwrap().unwrap();
    assert_eq!(record.publication.attempt_id, attempt);
    assert!(
        record
            .members
            .iter()
            .all(|m| m.table_uri.starts_with(prefix.as_str()))
    );
    let intent = catalog.intent(publication).await.unwrap().unwrap();
    assert_eq!(intent.attempt_id, attempt);
    assert_eq!(intent.member_prefix, prefix.as_str());
    // Preparing the same publication again recovers it: nothing is rewritten.
    let again = prepare(
        &runtime,
        attempt,
        &workspace,
        None,
        Some(publication),
        "identity",
    )
    .commit(&CancellationToken::new())
    .await
    .unwrap();
    assert_eq!(again, published);
    for member in &record.members {
        assert_eq!(version(&runtime, &member.table_uri).await, Some(1));
    }
    // The attempt is published once: another publication identity is refused.
    let other = prepare(
        &runtime,
        attempt,
        &workspace,
        Some(publication),
        None,
        "identity",
    )
    .commit(&CancellationToken::new())
    .await
    .unwrap_err();
    assert!(
        matches!(
            other,
            WorkflowError::Operations(OperationsError::PublicationIdentityReused { .. })
        ),
        "{other:?}"
    );
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_workspace_root_refused() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "legacy").await;
    let (legacy_directory, root) = directory();
    let log = legacy_directory.path().join("control/_delta_log");
    std::fs::create_dir_all(&log).unwrap();
    std::fs::write(log.join("00000000000000000000.json"), b"{}\n").unwrap();
    let refused = runtime
        .register_workspace("legacy", root.clone())
        .await
        .unwrap_err();
    assert!(
        matches!(&refused, WorkflowError::LegacyWorkspace { root: at } if *at == root),
        "{refused:?}"
    );
    assert!(refused.to_string().contains("migration required"));
    assert!(runtime.workspace("legacy").await.is_err());
    // A clean root registers; the same name and root are idempotent, another root is not.
    let (_clean_directory, clean) = directory();
    let registered = runtime
        .register_workspace("clean", clean.clone())
        .await
        .unwrap();
    assert_eq!(
        runtime.register_workspace("clean", clean).await.unwrap(),
        registered
    );
    assert!(runtime.register_workspace("clean", root).await.is_err());
    assert_eq!(runtime.workspace("clean").await.unwrap(), registered);
    // An ephemeral runtime has no catalog.
    assert!(
        ephemeral()
            .register_workspace("ephemeral", registered.root)
            .await
            .is_err()
    );
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn maintainer_binary_retires_and_collects() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "binary").await;
    let (_directory, root) = directory();
    let workspace = runtime.register_workspace("binary", root).await.unwrap();
    let old = publish(&runtime, &workspace, None, "old").await;
    let head = publish(&runtime, &workspace, Some(old.publication_id), "head").await;
    let maintain = |args: Vec<String>| {
        let url = database.url().to_owned();
        tokio::task::spawn_blocking(move || {
            std::process::Command::new(env!("CARGO_BIN_EXE_pse-publication"))
                .args(["--url", &url, "--threads", "1"])
                .args(args)
                .output()
                .unwrap()
        })
    };
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    let collected = maintain(vec![
        "collect".into(),
        "--workspace".into(),
        "binary".into(),
    ])
    .await
    .unwrap();
    let (stdout, stderr) = (text(&collected.stdout), text(&collected.stderr));
    assert!(collected.status.success(), "{stdout}{stderr}");
    assert!(stdout.contains("maintenance epoch"), "{stdout}");
    let retired = maintain(vec![
        "retire".into(),
        "--workspace".into(),
        "binary".into(),
        "--publication".into(),
        old.publication_id.to_string(),
        "--wait-seconds".into(),
        "5".into(),
    ])
    .await
    .unwrap();
    let (stdout, stderr) = (text(&retired.stdout), text(&retired.stderr));
    assert!(retired.status.success(), "{stdout}{stderr}");
    assert!(
        stdout.contains(&format!("retired {}", old.publication_id)),
        "{stdout}"
    );
    // The head is protected: the maintainer reports the refusal and fails.
    let refused = maintain(vec![
        "retire".into(),
        "--workspace".into(),
        "binary".into(),
        "--publication".into(),
        head.publication_id.to_string(),
    ])
    .await
    .unwrap();
    assert!(!refused.status.success());
    assert!(text(&refused.stderr).contains("protected"));
    assert!(
        runtime
            .open(old.publication_id, &CancellationToken::new())
            .await
            .is_err()
    );
    drop(runtime);
    database.remove().await.unwrap();
}

/// The object paths under a memory store that contain `needle`.
async fn objects_with(store: &Arc<pse_testkit::fault_store::FaultStore>, needle: &str) -> usize {
    futures_util::TryStreamExt::try_collect::<Vec<_>>(object_store::ObjectStore::list(
        store.as_ref(),
        None,
    ))
    .await
    .unwrap()
    .iter()
    .filter(|meta| meta.location.as_ref().contains(needle))
    .count()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn interrupted_deletion_resumes() {
    use pse_testkit::fault_store::{Fault as StoreFault, FaultPlan, FaultStore};
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable(database.url(), "interrupted").await;
    let root = url::Url::parse("memory://interrupted/workspace/").unwrap();
    let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    runtime
        .sessions()
        .native_state()
        .runtime_env()
        .register_object_store(&root, store.clone());
    let workspace = runtime
        .register_workspace("interrupted", root)
        .await
        .unwrap();
    let old = publish(&runtime, &workspace, None, "old").await;
    let middle = publish(&runtime, &workspace, Some(old.publication_id), "middle").await;
    let head = publish(&runtime, &workspace, Some(middle.publication_id), "head").await;
    let cancel = CancellationToken::new();
    let old_id = old.publication_id.to_string();
    let before = objects_with(&store, &old_id).await;
    assert!(before > 1);

    // The second object deletion fails: the retirement stops with the publication
    // expiring and part of its objects gone.
    store.arm(FaultPlan {
        operation: "delete",
        prefix: String::new(),
        call: 2,
        fault: StoreFault::FailBefore,
    });
    assert!(
        runtime
            .retire_publications(
                workspace.workspace_id,
                &[old.publication_id],
                Duration::from_secs(10),
                &cancel,
            )
            .await
            .is_err()
    );
    assert_eq!(store.fired(), 1);
    let remaining = objects_with(&store, &old_id).await;
    assert!(
        remaining > 0 && remaining < before,
        "{remaining} of {before}"
    );
    let refused = runtime.open(old.publication_id, &cancel).await.unwrap_err();
    assert!(
        matches!(
            refused,
            WorkflowError::Operations(OperationsError::PublicationRetiring { .. })
        ),
        "{refused:?}"
    );
    // Rerunning completes it: exactly the remaining objects go.
    let retired = runtime
        .retire_publications(
            workspace.workspace_id,
            &[old.publication_id],
            Duration::from_secs(10),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(retired.retired, [old.publication_id]);
    assert_eq!(retired.removed_objects, u64::try_from(remaining).unwrap());
    assert_eq!(objects_with(&store, &old_id).await, 0);

    // A failed listing removes nothing; the rerun removes everything.
    let middle_id = middle.publication_id.to_string();
    let listed = objects_with(&store, &middle_id).await;
    store.arm(FaultPlan {
        operation: "list",
        prefix: String::new(),
        call: 1,
        fault: StoreFault::FailBefore,
    });
    assert!(
        runtime
            .retire_publications(
                workspace.workspace_id,
                &[middle.publication_id],
                Duration::from_secs(10),
                &cancel,
            )
            .await
            .is_err()
    );
    assert_eq!(store.fired(), 1);
    assert_eq!(objects_with(&store, &middle_id).await, listed);
    runtime
        .retire_publications(
            workspace.workspace_id,
            &[middle.publication_id],
            Duration::from_secs(10),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(objects_with(&store, &middle_id).await, 0);
    // The head is untouched and reads.
    let current = runtime
        .open_head(workspace.workspace_id, &cancel)
        .await
        .unwrap();
    assert_eq!(current.publication_id(), head.publication_id);
    assert_eq!(rows(current.publication(), "authored", "entities").await, 1);
    drop((current, runtime));
    database.remove().await.unwrap();
}
