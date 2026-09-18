// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Current native cache/reuse journey, shared by final integration and measurements.
#![allow(clippy::unwrap_used, reason = "qualification assertions")]
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    },
    common::ResolvedTableReference,
    execution::{
        memory_pool::{GreedyMemoryPool, MemoryPool, PeakRecordingPool},
        runtime_env::RuntimeEnvBuilder,
        session_state::{SessionState, SessionStateBuilder},
    },
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    cache_service::{CacheBudget, NativeCacheService},
    delta::{
        contract::DeclaredCheck,
        publication::{Publication, PublicationRoot},
        publication_plan::{self, Member},
    },
    session::{SessionFactory, planner::UnifiedPlanner},
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::generated::{enums::PublicationKind, runtime::publications};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc, time::Instant};

fn name(table: &str) -> ResolvedTableReference {
    ResolvedTableReference {
        catalog: "cache_fixture".into(),
        schema: "authored".into(),
        table: table.to_owned().into(),
    }
}
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn header(publication: u8, parent: Option<u8>) -> publications::Row {
    publications::Row {
        workspace_id: id(1),
        publication_id: id(publication),
        parent_publication_id: parent.map(id),
        attempt_id: id(publication + 64),
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
    }
}
fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut builder);
    for (table, payload) in [("cache_values", true), ("cache_keys", false)] {
        let mut fields = vec![FieldContract::key(
            "id",
            FieldContract::native(DataType::Int64),
            "Stable identity",
        )];
        if payload {
            fields.push(FieldContract::payload(
                "value",
                FieldContract::native(DataType::Int64),
                "Changing parameter",
            ));
        }
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                table,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Native cache qualification",
            )
            .pk(&["id"])
            .columns(fields),
        );
    }
    Arc::new(builder.build().unwrap())
}
async fn publish(
    artifact: &ArtifactPlan,
    base: &url::Url,
    label: &str,
    mut header: publications::Row,
    cancel: &CancellationToken,
) -> PublicationRoot {
    let location = base.join(&format!("{label}_control/")).unwrap();
    let mut inputs = BTreeMap::new();
    for output in artifact.outputs().values() {
        for selected in artifact
            .session()
            .selected_dependencies(&output.plan)
            .unwrap()
        {
            inputs.insert(
                (
                    selected.catalog_name.clone(),
                    selected.schema_name.clone(),
                    selected.table_name.clone(),
                ),
                selected,
            );
        }
    }
    header.inputs = inputs
        .into_values()
        .map(|member| {
            use publications::{
                RuntimePublicationsFieldInputsItem as Input,
                RuntimePublicationsFieldInputsItemSelection as Selection,
                RuntimePublicationsFieldMembersItemSelectionSelected as Selected,
            };
            let selection = match member.selection.selected().unwrap() {
                Selected::Full => Selection::from_full(),
                Selected::Revision(value) => Selection::from_revision(
                    publications::RuntimePublicationsFieldInputsItemSelectionRevision {
                        column: value.column.clone(),
                        revision_id: value.revision_id,
                    },
                ),
            };
            Input {
                catalog_name: member.catalog_name,
                schema_name: member.schema_name,
                table_name: member.table_name,
                relation_id: member.relation_id,
                relation_version: member.relation_version,
                contract_fingerprint: member.contract_fingerprint,
                table_uri: member.table_uri,
                delta_version: member.delta_version,
                selection,
            }
        })
        .collect();
    let destinations = artifact
        .outputs()
        .keys()
        .map(|name| {
            (
                name.clone(),
                base.join(&format!("{label}_{}/", name.table)).unwrap(),
            )
        })
        .collect();
    let result = artifact
        .prepare_publication(
            PublicationTarget {
                reference: name("publications"),
                location: location.clone(),
            },
            header,
            destinations,
            vec![],
            cancel,
        )
        .unwrap()
        .execute(cancel)
        .await
        .unwrap();
    let version = result.batches()[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    PublicationRoot { location, version }
}
#[expect(
    clippy::too_many_arguments,
    reason = "explicit native fixture inputs and publication provenance"
)]
async fn selected_after_update(
    previous: &Publication,
    registry: Arc<Registry>,
    factory: &SessionFactory,
    state: &SessionState,
    field: &str,
    increment: i64,
    publication_id: u8,
    parent: u8,
    cancel: &CancellationToken,
) -> Publication {
    let mut member = previous.member(&name("cache_values")).unwrap();
    // Native table setup is independent of the PSE reader under test. Mutations
    // retain the declared CHECK and actual supplied session; no private replay.
    let table = deltalake::DeltaTableBuilder::from_url(url::Url::parse(&member.table_uri).unwrap())
        .unwrap()
        .load()
        .await
        .unwrap();
    let contract = DeclaredCheck::new(&registry, member.relation_id).unwrap();
    let state = Arc::new(contract.bind(state).unwrap());
    let (updated, _) = table
        .update()
        .with_update(field, col(field) + lit(increment))
        .with_session_state(state)
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .with_commit_properties(
            deltalake::kernel::transaction::CommitProperties::default()
                .with_create_checkpoint(true)
                .with_cleanup_expired_logs(Some(false)),
        )
        .await
        .unwrap();
    member.delta_version = i64::try_from(updated.version().unwrap()).unwrap();
    let plan = publication_plan::plan(
        previous.root().location.clone(),
        header(publication_id, Some(parent)),
        vec![Member::Retained(member)],
        registry.clone(),
    )
    .unwrap();
    let session = factory
        .candidate(BTreeMap::new(), registry.clone(), cancel)
        .unwrap()
        .with_purpose(pse_schema::model::provider::OperationPurpose::Publish);
    let result = session
        .prepare(plan, cancel)
        .unwrap()
        .execute(cancel)
        .await
        .unwrap();
    let root = PublicationRoot {
        location: previous.root().location.clone(),
        version: result.batches()[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
    };
    Publication::open(root, registry, factory, cancel)
        .await
        .unwrap()
}

/// Execute the same target code with retention on/off. Durations exclude compilation.
pub(crate) async fn run(enabled: bool, rows: usize) -> serde_json::Value {
    Box::pin(run_with_fence(enabled, rows, false)).await
}

pub(crate) async fn run_with_fence(
    enabled: bool,
    rows: usize,
    cross_process: bool,
) -> serde_json::Value {
    Box::pin(run_policy(enabled, rows, cross_process, None)).await
}

#[expect(
    clippy::too_many_lines,
    reason = "ordered cache and publication journey with independent fresh output assertions"
)]
pub(crate) async fn run_policy(
    enabled: bool,
    rows: usize,
    cross_process: bool,
    override_policy: Option<CacheBudget>,
) -> serde_json::Value {
    let custom_policy = override_policy.is_some();
    let cancel = CancellationToken::new();
    let registry = registry();
    let peak = Arc::new(PeakRecordingPool::new(Arc::new(GreedyMemoryPool::new(
        256 << 20,
    ))));
    let pool: Arc<dyn MemoryPool> = peak.clone();
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .unwrap();
    let policy = if enabled {
        CacheBudget::for_memory(256 << 20)
    } else {
        CacheBudget::disabled(128 << 20)
    };
    let caches = NativeCacheService::new(override_policy.unwrap_or(policy), &pool).unwrap();
    let state = SessionStateBuilder::new()
        .with_default_features()
        .with_runtime_env(runtime.clone())
        .with_query_planner(Arc::new(UnifiedPlanner::default()))
        .build();
    let factory = SessionFactory::from_builder(
        runtime,
        FixedBudget::new(256 << 20),
        "cache-qualification",
        SessionStateBuilder::new_from_existing(state.clone()),
    )
    .with_cache_service(caches.clone());
    let spec = registry.relation("authored.cache_values").unwrap();
    let keys = (0..i64::try_from(rows).unwrap()).collect::<Vec<_>>();
    let batch = RecordBatch::try_new(
        Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
        vec![
            Arc::new(Int64Array::from(keys.clone())),
            Arc::new(Int64Array::from(vec![10; rows])),
        ],
    )
    .unwrap();
    let session = factory
        .candidate(
            BTreeMap::from([(spec.key, batch)]),
            registry.clone(),
            &cancel,
        )
        .unwrap();
    let source = session
        .relation_plan(
            &session
                .table_reference(&spec.key)
                .unwrap()
                .resolve("workspace", "authored"),
        )
        .unwrap();
    let artifact = ArtifactPlan::new(
        session,
        BTreeMap::from([(
            name("cache_values"),
            RelationOutput {
                relation_id: spec.id,
                plan: source.plan().clone(),
            },
        )]),
        &cancel,
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let start = Instant::now();
    let root = publish(&artifact, &base, "source", header(2, None), &cancel).await;
    let publish_seconds = start.elapsed().as_secs_f64();
    let start = Instant::now();
    let first = Publication::open(root.clone(), registry.clone(), &factory, &cancel)
        .await
        .unwrap();
    let open_seconds = start.elapsed().as_secs_f64();
    let mut reads = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let result = first
            .session()
            .relation_plan(&name("cache_values"))
            .unwrap();
        let completed = first
            .session()
            .prepare(result.plan().clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        assert_eq!(
            completed
                .batches()
                .iter()
                .map(|batch| batch.num_rows())
                .sum::<usize>(),
            rows
        );
        reads.push(start.elapsed().as_secs_f64());
    }
    let first = if cross_process {
        let member = first.member(&name("cache_values")).unwrap();
        let request = serde_json::json!({"root":root,"member":member});
        drop(first);
        // The parent retains only idle caches. The child executes actual native
        // maintenance under the same OS lease and commits fences in both logs.
        let before = caches.report().iter().map(|row| row.misses).sum::<usize>();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "selected_cache_and_projected_reuse_match_fresh_native_results",
                "--nocapture",
            ])
            .env("PSE_NATIVE_CACHE_MAINTENANCE", request.to_string())
            .status()
            .unwrap();
        assert!(status.success());
        let reopened = Publication::open(root.clone(), registry.clone(), &factory, &cancel)
            .await
            .unwrap();
        assert!(caches.report().iter().map(|row| row.misses).sum::<usize>() > before || !enabled);
        reopened
    } else {
        first
    };
    let output_id = registry.relation("authored.cache_keys").unwrap().id;
    let source = first
        .session()
        .relation_plan(&name("cache_values"))
        .unwrap();
    let plan = LogicalPlanBuilder::from(source.plan().clone())
        .project([col("id")])
        .unwrap()
        .build()
        .unwrap();
    let structural = ArtifactPlan::new(
        first.session().clone(),
        BTreeMap::from([(
            name("cache_keys"),
            RelationOutput {
                relation_id: output_id,
                plan,
            },
        )]),
        &cancel,
    )
    .unwrap();
    assert!(
        structural
            .dependencies()
            .unwrap()
            .iter()
            .any(|row| row.evidence.projection.is_some()),
        "native projection must be real dependency evidence: {:?}; policy: {:?}; plan: {}",
        structural.dependencies().unwrap(),
        structural.session().effective_policy().unwrap(),
        structural
            .outputs()
            .values()
            .next()
            .unwrap()
            .plan
            .display_indent()
    );
    let output_root = publish(&structural, &base, "derived", header(10, None), &cancel).await;
    let output = Publication::open(output_root, registry.clone(), &factory, &cancel)
        .await
        .unwrap();
    let next = selected_after_update(
        &first,
        registry.clone(),
        &factory,
        &state,
        "value",
        1,
        3,
        2,
        &cancel,
    )
    .await;
    let rebound = structural
        .rebind_inputs(next.session().clone(), &cancel)
        .unwrap();
    let start = Instant::now();
    let reused = rebound
        .prepare_reuse(&output, &name("cache_keys"), &cancel)
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let reuse_seconds = start.elapsed().as_secs_f64();
    let fresh = rebound
        .prepare(&name("cache_keys"), &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let values = |batches: &[pse_ids::owned_buffer::OwnedRecordBatch]| {
        let mut values = batches
            .iter()
            .flat_map(|batch| {
                batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .values()
                    .to_vec()
            })
            .collect::<Vec<_>>();
        values.sort_unstable();
        values
    };
    assert_eq!(values(reused.batches()), keys);
    assert_eq!(values(reused.batches()), values(fresh.batches()));
    let changed = selected_after_update(
        &next,
        registry.clone(),
        &factory,
        &state,
        "id",
        i64::try_from(rows).unwrap(),
        4,
        3,
        &cancel,
    )
    .await;
    let changed = structural
        .rebind_inputs(changed.into_session(), &cancel)
        .unwrap();
    assert!(
        changed
            .prepare_reuse(&output, &name("cache_keys"), &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .is_err(),
        "structural changes must refuse reuse"
    );
    let report = caches.report();
    if enabled && !custom_policy {
        assert!(
            report
                .iter()
                .any(|row| row.name == "pse.cache.resident" && row.hits >= 2)
        );
    }
    serde_json::json!({ "cache_enabled": enabled, "rows": rows, "publish_seconds": publish_seconds, "open_seconds": open_seconds,
        "read_seconds": reads, "reuse_seconds": reuse_seconds, "pool_reserved_bytes": pool.reserved(), "pool_peak_bytes": peak.max_reserved(), "process_peak_rss_bytes":process_peak_rss(),
        "cache": report.iter().map(|row| serde_json::json!({"name":row.name,"hits":row.hits,"misses":row.misses,"retained_bytes":row.retained_bytes,"live_bytes":row.live_bytes,"pinned_bytes":row.pinned_bytes})).collect::<Vec<_>>(),
        "execution_counts": caches.execution_report().into_iter().collect::<BTreeMap<_,_>>() })
}

/// Child entry for actual two-process maintenance; no simulated invalidation call.
pub(crate) async fn maintenance_child(payload: &str) {
    use pse_catalog::delta::maintenance::{MaintenanceAction, MaintenanceTarget};
    use pse_relations::generated::runtime::retained_versions;
    let request: serde_json::Value = serde_json::from_str(payload).unwrap();
    let root: PublicationRoot = serde_json::from_value(request["root"].clone()).unwrap();
    let member: publications::RuntimePublicationsFieldMembersItem =
        serde_json::from_value(request["member"].clone()).unwrap();
    let registry = registry();
    let runtime = RuntimeEnvBuilder::new().build_arc().unwrap();
    let factory = SessionFactory::from_builder(
        runtime.clone(),
        FixedBudget::new(256 << 20),
        "maintenance-child",
        SessionStateBuilder::new()
            .with_default_features()
            .with_runtime_env(runtime)
            .with_query_planner(Arc::new(UnifiedPlanner::default())),
    );
    let cancel = CancellationToken::new();
    let empty = retained_versions::Builder::with_registry(&registry, 0)
        .unwrap()
        .finish()
        .unwrap()
        .into_batch();
    let key = retained_versions::spec(&registry).unwrap().key;
    let session = factory
        .candidate(BTreeMap::from([(key, empty)]), registry, &cancel)
        .unwrap();
    let reference = session
        .table_reference(&key)
        .unwrap()
        .resolve("workspace", "runtime");
    let retention = session.relation_plan(&reference).unwrap();
    session
        .prepare_maintenance(
            MaintenanceTarget {
                head: root,
                reference: name("cache_values"),
                location: url::Url::parse(&member.table_uri).unwrap(),
                relation_id: member.relation_id,
                action: MaintenanceAction::Optimize,
                log_cutoff_ms: 0,
            },
            &retention,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
}

/// Process high water, separate from this fixture's native reservation peak.
pub(crate) fn process_peak_rss() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find(|line| line.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()?
        .checked_mul(1024)
}
