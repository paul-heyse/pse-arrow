// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact-pin integration contracts: real native children, caller state and validation.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration assertions"
)]

use pse_testkit::fault_store;

use datafusion::{
    arrow::{
        array::{Array, Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    common::{Result, config::ConfigOptions},
    execution::{
        context::SessionContext,
        runtime_env::RuntimeEnv,
        session_state::{SessionState, SessionStateBuilder},
    },
    logical_expr::{LogicalPlan, LogicalPlanBuilder, Volatility, col, create_udf},
    physical_optimizer::PhysicalOptimizerRule,
    physical_plan::{ExecutionPlan, collect},
    prelude::SessionConfig,
};
use deltalake::{
    DeltaTable, DeltaTableBuilder, kernel::transaction::CommitProperties, protocol::SaveMode,
};
use pse_catalog::delta::write::DeltaWrite;
use pse_engine::session::planner::UnifiedPlanner;
use pse_relations::generated::{enums::PublicationKind, runtime::publications};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
struct SentinelRule(Arc<AtomicUsize>);
impl PhysicalOptimizerRule for SentinelRule {
    fn name(&self) -> &'static str {
        "sentinel_rule"
    }
    fn schema_check(&self) -> bool {
        true
    }
    fn optimize(
        &self,
        plan: Arc<dyn ExecutionPlan>,
        config: &ConfigOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        assert_eq!(config.execution.batch_size.get(), 13);
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(plan)
    }
}
fn context() -> (
    SessionContext,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    Arc<RuntimeEnv>,
) {
    let runtime = datafusion::execution::runtime_env::RuntimeEnvBuilder::new()
        .with_memory_pool(Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 30)))
        .build_arc()
        .unwrap();
    let rules = Arc::new(AtomicUsize::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let udf = create_udf(
        "sentinel",
        vec![DataType::Int64],
        DataType::Int64,
        Volatility::Volatile,
        Arc::new(move |args| {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(args[0].clone())
        }),
    );
    let state = SessionStateBuilder::new()
        .with_default_features()
        .with_runtime_env(Arc::clone(&runtime))
        .with_config(
            SessionConfig::new()
                .with_batch_size(13)
                .with_target_partitions(2),
        )
        .with_query_planner(Arc::new(UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )))
        .build();
    let mut optimizers = state.physical_optimizers().to_vec();
    optimizers.push(Arc::new(SentinelRule(Arc::clone(&rules))));
    let state = SessionStateBuilder::new_from_existing(state)
        .with_physical_optimizer_rules(optimizers)
        .build();
    let context = SessionContext::new_with_state(state);
    context.register_udf(udf);
    (context, calls, rules, runtime)
}
async fn input(context: &SessionContext, values: Vec<i64>) -> LogicalPlan {
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
    let batch = RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(values))]).unwrap();
    context.deregister_table("source").unwrap();
    context.register_batch("source", batch).unwrap();
    context
        .sql("SELECT sentinel(id) AS id FROM source")
        .await
        .unwrap()
        .into_unoptimized_plan()
}
async fn execute(
    state: &SessionState,
    table: DeltaTable,
    input: LogicalPlan,
) -> Result<Vec<RecordBatch>> {
    let plan = DeltaWrite::plan(table, input, SaveMode::Append, CommitProperties::default())?;
    run_native(state, &plan).await
}
fn location(path: &std::path::Path) -> url::Url {
    url::Url::from_directory_path(path).unwrap()
}

fn publication_row(id: u8, parent: Option<u8>) -> publications::Row {
    let identity = |id| pse_ids::SemanticId::from_bytes([id; 16]);
    publications::Row {
        workspace_id: identity(1),
        publication_id: identity(id),
        parent_publication_id: parent.map(identity),
        attempt_id: identity(id + 100),
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![],
    }
}

fn source_member_batches(dangling: bool) -> Vec<(&'static str, pse_ids::SemanticId, RecordBatch)> {
    use pse_relations::generated::{
        authored::{entities, packages},
        enums::{EntityKind, IdPolicy, PackageKind},
    };
    let identity = |id| pse_ids::SemanticId::from_bytes([id; 16]);
    let mut packages = packages::Builder::new().unwrap();
    packages
        .push(packages::Row {
            package_id: identity(10),
            name: "model".into(),
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
            package_id: identity(if dangling { 99 } else { 10 }),
            kind: EntityKind::Package,
            name: "model".into(),
            qualified_name: "model".into(),
            parent_entity_id: None,
            source_span: None,
        })
        .unwrap();
    vec![
        (
            "packages.v1",
            packages::RELATION_ID,
            packages.finish().unwrap().into_batch(),
        ),
        (
            "entities.v1",
            entities::RELATION_ID,
            entities.finish().unwrap().into_batch(),
        ),
    ]
}

#[tokio::test]
async fn native_member_writes_feed_actual_versions_into_coherent_publication() {
    use pse_catalog::delta::{
        publication::PublicationRoot,
        publication_plan::{self, Member, MemberWrite},
    };
    for dangling in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let (context, _, _, _) = context();
        let mut members = vec![];
        for (name, relation_id, batch) in source_member_batches(dangling) {
            let reference = datafusion::common::ResolvedTableReference {
                catalog: "model.a".into(),
                schema: "authored".into(),
                table: name.into(),
            };
            context
                .register_batch(datafusion::common::TableReference::bare(name), batch)
                .unwrap();
            members.push(Member::Write(MemberWrite {
                reference,
                relation_id,
                table: DeltaTableBuilder::from_url(location(&root.path().join(name)))
                    .unwrap()
                    .build()
                    .unwrap(),
                input: context
                    .table(datafusion::common::TableReference::bare(name))
                    .await
                    .unwrap()
                    .into_unoptimized_plan(),
            }));
        }
        let control = location(&root.path().join("control"));
        let registry = pse_schema::shared_registry().unwrap();
        let plan = publication_plan::plan(
            control.clone(),
            publication_row(2, None),
            members,
            Arc::clone(&registry),
        )
        .unwrap();
        let state = context.state();
        let prepared = prepare_native(&state, &plan).unwrap();
        assert_eq!(
            plan.display_indent()
                .to_string()
                .matches("DeltaWrite:")
                .count(),
            2
        );
        assert!(!root.path().join("control/_delta_log").exists());
        assert!(!root.path().join("packages.v1/_delta_log").exists());
        let result = prepared
            .execute(&pse_columnar::CancellationToken::new())
            .await;
        if dangling {
            assert!(result.is_err());
            assert!(
                !DeltaTableBuilder::from_url(control)
                    .unwrap()
                    .build()
                    .unwrap()
                    .verify_deltatable_existence()
                    .await
                    .unwrap()
            );
        } else {
            result.unwrap();
            let publication = open_publication(
                PublicationRoot {
                    location: control,
                    version: 1,
                },
                &registry,
                Arc::new(state),
            )
            .await
            .unwrap();
            assert_eq!(publication.record().members.len(), 2);
            assert_eq!(publication.record().members[0].table_name, "entities.v1");
            assert!(
                publication
                    .record()
                    .members
                    .iter()
                    .all(|m| m.delta_version == 1),
                "versions include the actual CHECK commits"
            );
            assert_eq!(
                publication_count(
                    &publication,
                    "SELECT * FROM \"model.a\".authored.\"entities.v1\""
                )
                .await,
                1
            );
        }
    }
}
fn publication_plan(location: url::Url, row: publications::Row) -> LogicalPlan {
    use datafusion::datasource::{MemTable, provider_as_source};
    let mut builder = publications::Builder::new().unwrap();
    builder.push(row).unwrap();
    let batch = builder.finish().unwrap().into_batch();
    let input = LogicalPlanBuilder::scan(
        "candidate_control",
        provider_as_source(Arc::new(
            MemTable::try_new(batch.schema(), vec![vec![batch]]).unwrap(),
        )),
        None,
    )
    .unwrap()
    .build()
    .unwrap();
    pse_catalog::delta::publish::DeltaPublish::plan(
        location,
        input,
        pse_schema::shared_registry().unwrap(),
    )
    .unwrap()
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "ordered native lifecycle qualification preserves the independent assertions beside each phase"
)]
async fn publication_composes_one_write_with_an_exact_unchanged_member() {
    use pse_catalog::delta::{
        publication::PublicationRoot,
        publication_plan::{self, Member, MemberWrite},
    };
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let state = context.state();
    let registry = pse_schema::shared_registry().unwrap();
    let mut original = publication_row(2, None);
    for (_, id, batch) in source_member_batches(false) {
        let name = registry.relation_by_id(id).unwrap().qualified_name();
        original
            .members
            .push(write_member(&context, temp.path(), &name, batch).await);
    }
    let control = location(&temp.path().join("control"));
    assert_eq!(
        publish(&state, control.clone(), original.clone())
            .await
            .unwrap(),
        1
    );
    let packages = original.members[0].clone();
    let entities = &original.members[1];
    let (_, relation_id, batch) = source_member_batches(false).pop().unwrap();
    let table = DeltaTableBuilder::from_url(location(&temp.path().join("child-entities")))
        .unwrap()
        .build()
        .unwrap();
    let members = vec![
        Member::Retained(packages.clone()),
        Member::Write(MemberWrite {
            reference: datafusion::common::ResolvedTableReference {
                catalog: entities.catalog_name.clone().into(),
                schema: entities.schema_name.clone().into(),
                table: entities.table_name.clone().into(),
            },
            relation_id,
            table,
            input: context.read_batch(batch).unwrap().into_unoptimized_plan(),
        }),
    ];
    let plan = publication_plan::plan(
        control.clone(),
        publication_row(3, Some(2)),
        members,
        registry.clone(),
    )
    .unwrap();
    assert_eq!(
        plan.display_indent()
            .to_string()
            .matches("DeltaWrite:")
            .count(),
        1
    );
    run_native(&state, &plan).await.unwrap();
    let current = open_publication(
        PublicationRoot {
            location: control.clone(),
            version: 2,
        },
        &registry,
        Arc::new(state.clone()),
    )
    .await
    .unwrap();
    let selected = &current.record().members;
    assert_eq!(
        selected
            .iter()
            .find(|m| m.table_name == "packages")
            .unwrap(),
        &packages
    );
    assert_eq!(
        selected
            .iter()
            .find(|m| m.table_name == "entities")
            .unwrap()
            .delta_version,
        1
    );
    assert_eq!(
        DeltaTableBuilder::from_url(packages.table_uri.parse().unwrap())
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(1)
    );
    assert_eq!(
        pse_catalog::selection::selected_member(
            current.session(),
            &datafusion::common::ResolvedTableReference {
                catalog: "model".into(),
                schema: "authored".into(),
                table: "entities".into()
            }
        )
        .unwrap()
        .delta_version,
        1
    );

    qualify_retained_only_composition(&state, control, selected).await;
}

async fn qualify_retained_only_composition(
    state: &SessionState,
    control: url::Url,
    selected: &[publications::RuntimePublicationsFieldMembersItem],
) {
    use pse_catalog::delta::{
        publication::PublicationRoot,
        publication_plan::{self, Member},
    };
    let registry = pse_schema::shared_registry().unwrap();
    // A publication can select only existing exact members without rewriting data.
    let retained = selected.iter().cloned().map(Member::Retained).collect();
    let plan = publication_plan::plan(
        control.clone(),
        publication_row(4, Some(3)),
        retained,
        registry.clone(),
    )
    .unwrap();
    assert!(!plan.display_indent().to_string().contains("DeltaWrite:"));
    run_native(state, &plan).await.unwrap();
    let reopened = open_publication(
        PublicationRoot {
            location: control.clone(),
            version: 3,
        },
        &registry,
        Arc::new(state.clone()),
    )
    .await
    .unwrap();
    assert_eq!(reopened.record().members, selected);

    let mut missing = selected.to_vec();
    missing[0].delta_version = 999;
    let plan = publication_plan::plan(
        control.clone(),
        publication_row(5, Some(4)),
        missing.into_iter().map(Member::Retained).collect(),
        registry,
    )
    .unwrap();
    assert!(run_native(state, &plan).await.is_err());
    assert_eq!(
        DeltaTableBuilder::from_url(control)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(3)
    );
}

async fn publish(state: &SessionState, location: url::Url, row: publications::Row) -> Result<i64> {
    let plan = publication_plan(location, row);
    let result = run_native(state, &plan).await?;
    Ok(result[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0))
}

#[tokio::test]
async fn publication_retry_checks_complete_request_after_head_advancement() {
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let location = location(temp.path());
    let state = context.state();
    let original = publication_row(2, None);
    let plan = publication_plan(location.clone(), original.clone());
    let prepared = prepare_native(&state, &plan).unwrap();
    assert!(!temp.path().join("_delta_log").exists());
    assert!(!prepared.optimized_plan().inputs().is_empty());
    prepared
        .execute(&pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        publish(&state, location.clone(), publication_row(3, Some(2)))
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        publish(&state, location.clone(), original.clone())
            .await
            .unwrap(),
        1
    );
    let mut changed = original;
    changed.kind = PublicationKind::Model;
    let error = publish(&state, location.clone(), changed)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("reused"), "{error}");
    assert!(
        publish(&state, location.clone(), publication_row(4, Some(2)))
            .await
            .unwrap_err()
            .to_string()
            .contains("parent changed")
    );
    assert_eq!(
        DeltaTableBuilder::from_url(location)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(2)
    );
}

#[tokio::test]
async fn unavailable_declared_input_prevents_publication() {
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let mut record = publication_row(2, None);
    let relation = pse_engine::validation::registry()
        .unwrap()
        .relation("authored.packages")
        .unwrap();
    record
        .inputs
        .push(publications::RuntimePublicationsFieldInputsItem {
            catalog_name: "source".into(),
            schema_name: "authored".into(),
            table_name: "packages".into(),
            relation_id: relation.id,
            relation_version: i64::from(relation.key.version),
            contract_fingerprint: relation.fingerprint,
            table_uri: location(&temp.path().join("absent")).to_string(),
            delta_version: 13,
            selection: publications::RuntimePublicationsFieldInputsItemSelection::from_full(),
        });
    assert!(
        publish(
            &context.state(),
            location(&temp.path().join("control")),
            record
        )
        .await
        .is_err()
    );
    assert!(!temp.path().join("control/_delta_log").exists());
}

#[tokio::test]
async fn a_nested_delta_write_without_scans_cannot_bypass_inspection_policy() {
    use pse_engine::session::{ExecutionSettings, ThreadBudget};
    use pse_schema::model::provider::OperationPurpose;
    let temp = tempfile::tempdir().unwrap();
    let cancel = pse_columnar::CancellationToken::new();
    let session = pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
    )
    .unwrap()
    .candidate(
        std::collections::BTreeMap::new(),
        pse_schema::shared_registry().unwrap(),
        &cancel,
    )
    .unwrap()
    .with_purpose(OperationPurpose::Inspect);
    let input = LogicalPlanBuilder::empty(true)
        .project(vec![datafusion::logical_expr::lit(1i64).alias("id")])
        .unwrap()
        .build()
        .unwrap();
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .build()
        .unwrap();
    let write =
        DeltaWrite::plan(table, input, SaveMode::Append, CommitProperties::default()).unwrap();
    let plan = LogicalPlanBuilder::from(write)
        .project(vec![col("version")])
        .unwrap()
        .build()
        .unwrap();
    let error = session.prepare(plan, &cancel).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("outside the effective purpose/policy"),
        "{error}"
    );
    assert!(!temp.path().join("_delta_log").exists());
}

#[tokio::test]
async fn concurrent_publication_creation_and_parent_updates_have_one_winner() {
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let location = location(temp.path());
    let state = context.state();
    let (left, right) = tokio::join!(
        publish(&state, location.clone(), publication_row(2, None)),
        publish(&state, location.clone(), publication_row(3, None))
    );
    assert_ne!(left.is_ok(), right.is_ok(), "{left:?} {right:?}");
    let parent = if left.is_ok() { 2 } else { 3 };
    let (left, right) = tokio::join!(
        publish(&state, location.clone(), publication_row(4, Some(parent))),
        publish(&state, location.clone(), publication_row(5, Some(parent)))
    );
    assert_ne!(left.is_ok(), right.is_ok(), "{left:?} {right:?}");
    assert_eq!(
        DeltaTableBuilder::from_url(location)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(2)
    );
}

#[tokio::test]
async fn publication_reconciles_actual_lost_commit_acknowledgments() {
    use fault_store::{Fault, FaultPlan, FaultStore};
    let (context, _, _, runtime) = context();
    let location = url::Url::parse("memory://publication/control/").unwrap();
    let counts = pse_testkit::counting_store::CountingStore::new(Arc::new(
        object_store::memory::InMemory::new(),
    ));
    let store = FaultStore::new(counts.clone());
    runtime.register_object_store(&location, store.clone());
    let state = context.state();
    for (id, parent, version) in [(2, None, 1), (3, Some(2), 2)] {
        store.arm(FaultPlan {
            operation: "put",
            prefix: format!("control/_delta_log/{version:020}.json"),
            call: 1,
            fault: Fault::LostResponse,
        });
        let record = publication_row(id, parent);
        counts.reset();
        let started = std::time::Instant::now();
        assert_eq!(
            publish(&state, location.clone(), record.clone())
                .await
                .unwrap(),
            version
        );
        recovery_measurement("lost-commit-response", version, started, &counts);
        assert_eq!(store.fired(), 1, "the real commit fault must fire");
        counts.reset();
        let started = std::time::Instant::now();
        assert_eq!(
            publish(&state, location.clone(), record).await.unwrap(),
            version
        );
        recovery_measurement(
            "confirmed-idempotent-publication",
            version,
            started,
            &counts,
        );
    }
    let table = pse_catalog::delta::provider::table_builder(location, &state)
        .unwrap()
        .load()
        .await
        .unwrap();
    assert_eq!(
        table.version(),
        Some(2),
        "retries must not duplicate a commit"
    );
}

#[expect(
    clippy::print_stdout,
    reason = "optional phase timings and actual IO counts for the performance recipe"
)]
fn recovery_measurement(
    phase: &str,
    version: i64,
    started: std::time::Instant,
    counts: &pse_testkit::counting_store::CountingStore,
) {
    if std::env::var("PSE_RECOVERY_MEASURE").as_deref() == Ok("1") {
        println!(
            "PSE_RECOVERY_MEASUREMENT {}",
            serde_json::json!({
                "phase": phase, "delta_version": version,
                "seconds": started.elapsed().as_secs_f64(), "io": counts.report(),
                "backend": "in-memory; actual conditional Delta commit with lost response",
                "pool_budget_bytes": 1u64 << 30, "observation": "Contract"
            })
        );
    }
}

async fn write_member(
    context: &SessionContext,
    root: &std::path::Path,
    name: &str,
    batch: RecordBatch,
) -> publications::RuntimePublicationsFieldMembersItem {
    use datafusion::datasource::{MemTable, provider_as_source};
    use pse_catalog::delta::contract::DeclaredCheck;
    let registry = pse_engine::validation::registry().unwrap();
    let spec = registry.relation(name).unwrap();
    let contract = DeclaredCheck::new(registry, spec.id).unwrap();
    let input = LogicalPlanBuilder::scan(
        "member_input",
        provider_as_source(Arc::new(
            MemTable::try_new(batch.schema(), vec![vec![batch]]).unwrap(),
        )),
        None,
    )
    .unwrap()
    .build()
    .unwrap();
    let location = location(&root.join(spec.key.name));
    let plan = DeltaWrite::declared(
        DeltaTableBuilder::from_url(location.clone())
            .unwrap()
            .build()
            .unwrap(),
        input,
        SaveMode::ErrorIfExists,
        CommitProperties::default(),
        contract,
    )
    .unwrap();
    let state = context.state();
    let result = run_native(&state, &plan).await.unwrap();
    let delta_version = result[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    publications::RuntimePublicationsFieldMembersItem {
        catalog_name: "model".into(),
        schema_name: spec.key.namespace.as_str().into(),
        table_name: spec.key.name.into(),
        relation_id: spec.id,
        relation_version: i64::from(spec.key.version),
        contract_fingerprint: spec.fingerprint,
        table_uri: location.to_string(),
        delta_version,
        selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
    }
}

async fn assert_isolated_publication(publication: &pse_catalog::delta::publication::Publication) {
    let cancel = pse_columnar::CancellationToken::new();
    let first = publication.session().clone();
    let second = publication.session().clone();
    let prepared = first
        .prepare_sql("SELECT * FROM model.authored.entities", &cancel)
        .await
        .unwrap();
    drop(first);
    assert_eq!(
        prepared
            .execute(&cancel)
            .await
            .unwrap()
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        second
            .sql("SELECT * FROM model.authored.entities", &cancel)
            .await
            .unwrap()
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>(),
        1
    );
    assert_eq!(
        publication_count(publication, "SELECT * FROM model.authored.entities").await,
        1
    );
}

#[tokio::test]
async fn publication_admits_real_members_and_rejects_duplicates_and_dangling_references() {
    use pse_catalog::delta::publication::PublicationRoot;
    use pse_relations::generated::{
        authored::{entities, packages},
        enums::{EntityKind, IdPolicy, PackageKind},
    };
    let identity = |id| pse_ids::SemanticId::from_bytes([id; 16]);
    for case in [
        "valid",
        "duplicate",
        "dangling",
        "missing-check",
        "wrong-contract",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let (context, _, _, _) = context();
        let mut packages = packages::Builder::new().unwrap();
        packages
            .push(packages::Row {
                package_id: identity(10),
                name: "test".into(),
                version: "1.0.0".into(),
                kind: PackageKind::Model,
                id_policy: IdPolicy::Explicit,
                dependencies: vec![],
                content_hash: pse_ids::ContentHash::NIL,
                doc: String::new(),
            })
            .unwrap();
        let mut entities = entities::Builder::new().unwrap();
        let entity = entities::Row {
            entity_id: identity(11),
            package_id: identity(10),
            kind: EntityKind::Package,
            name: "test".into(),
            qualified_name: "test".into(),
            parent_entity_id: (case == "dangling").then(|| identity(99)),
            source_span: None,
        };
        entities.push(entity.clone()).unwrap();
        if case == "duplicate" {
            entities.push(entity).unwrap();
        }
        let mut row = publication_row(2, None);
        row.members.push(
            write_member(
                &context,
                temp.path(),
                "authored.packages",
                packages.finish().unwrap().into_batch(),
            )
            .await,
        );
        let entities_batch = entities.finish().unwrap().into_batch();
        row.members
            .push(write_member(&context, temp.path(), "authored.entities", entities_batch).await);
        if case == "missing-check" {
            remove_native_check(&mut row.members[1]).await;
        }
        if case == "wrong-contract" {
            row.members[1].contract_fingerprint = pse_ids::ContentHash::NIL;
        }
        let root = location(&temp.path().join("control"));
        let result = publish(&context.state(), root.clone(), row).await;
        if case == "valid" {
            assert_eq!(result.unwrap(), 1);
            let publication = open_publication(
                PublicationRoot {
                    location: root,
                    version: 1,
                },
                pse_engine::validation::registry().unwrap(),
                Arc::new(context.state()),
            )
            .await
            .unwrap();
            assert_isolated_publication(&publication).await;
        } else {
            let error = result.unwrap_err();
            if case == "missing-check" {
                assert!(error.to_string().contains("property"), "{error}");
            } else if case == "wrong-contract" {
                assert!(error.to_string().contains("fingerprint"), "{error}");
            } else {
                assert!(
                    error.to_string().contains("candidate violates"),
                    "{case}: {error:?}"
                );
            }
            assert!(
                !temp.path().join("control/_delta_log").exists(),
                "bad candidate advanced publication"
            );
        }
    }
}

#[tokio::test]
async fn nested_write_uses_real_child_caller_rules_functions_and_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let (context, calls, rules, runtime) = context();
    let input = input(&context, vec![3, 5, 8]).await;
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .build()
        .unwrap();
    let commit = CommitProperties::default()
        .with_metadata([("pse.attempt".into(), serde_json::json!("native-child"))]);
    let write = DeltaWrite::plan(table, input, SaveMode::Append, commit).unwrap();
    // A projection above the command must use ordinary recursive extension planning.
    let nested = LogicalPlanBuilder::from(write)
        .project(vec![col("version")])
        .unwrap()
        .build()
        .unwrap();
    let state = context.state();
    let prepared = prepare_native_observed(
        &state,
        &nested,
        pse_engine::session::assurance::ObservationPolicy::Diagnostic,
    )
    .unwrap();
    assert!(Arc::ptr_eq(&runtime, state.runtime_env()));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(!temp.path().join("_delta_log").exists());
    let before = rules.load(Ordering::SeqCst);
    let result = prepared
        .clone()
        .execute(&pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    let display = result.observation().physical_plan().unwrap();
    assert!(display.contains("DeltaWriteExec"), "{display}");
    assert!(
        display.contains("sentinel"),
        "native child disappeared: {display}"
    );
    assert_eq!(
        result
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        1
    );
    assert!(calls.load(Ordering::SeqCst) > 0);
    assert!(
        rules.load(Ordering::SeqCst) > before,
        "Delta builder discarded caller optimizers"
    );
    assert!(
        prepared
            .execute(&pse_columnar::CancellationToken::new())
            .await
            .is_err(),
        "re-execution must not duplicate a commit"
    );
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .load()
        .await
        .unwrap();
    assert_eq!(table.version(), Some(0));
    assert_eq!(
        table.history(Some(1)).await.unwrap().next().unwrap().info["pse.attempt"],
        "native-child"
    );
    let provider = table
        .table_provider()
        .with_session(Arc::new(state))
        .build()
        .await
        .unwrap();
    context
        .register_table("written", Arc::new(provider))
        .unwrap();
    let rows = context
        .sql("SELECT id FROM written ORDER BY id")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let ids: Vec<_> = rows
        .iter()
        .flat_map(|b| {
            b.column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .values()
                .to_vec()
        })
        .collect();
    assert_eq!(ids, vec![3, 5, 8]);
}

#[tokio::test]
async fn validating_route_rejects_bad_rows_and_constraint_addition_scans_existing_rows() {
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .build()
        .unwrap();
    execute(&context.state(), table, input(&context, vec![1]).await)
        .await
        .unwrap();
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .load()
        .await
        .unwrap();
    let table = table
        .add_constraint()
        .with_constraint("positive", "id > 0")
        .with_session_state(Arc::new(context.state()))
        .await
        .unwrap();
    let version = table.version();
    let result = execute(
        &context.state(),
        table.clone(),
        input(&context, vec![-1]).await,
    )
    .await;
    assert!(result.is_err(), "CHECK predicate must reject a write");
    let reopened = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .load()
        .await
        .unwrap();
    assert_eq!(reopened.version(), version);
    assert!(
        table
            .add_constraint()
            .with_constraint("large", "id > 10")
            .with_session_state(Arc::new(context.state()))
            .await
            .is_err(),
        "constraint creation must scan old rows"
    );
}

#[test]
fn every_registry_relation_has_a_declared_delta_layout() {
    use deltalake::kernel::{StructType, engine::arrow_conversion::TryIntoKernel};
    let registry = pse_engine::validation::registry().unwrap();
    for relation in registry.relations() {
        let schema = pse_schema::delta::relation_schema(registry, relation).unwrap();
        let converted: std::result::Result<StructType, _> = (&schema).try_into_kernel();
        assert!(converted.is_ok(), "{}: {converted:?}", relation.key);
    }
}

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "keep the complete independent nested fixture and its assertions together"
)]
async fn unsigned_ids_nested_values_and_nanoseconds_roundtrip_through_delta() {
    use datafusion::arrow::{
        array::{
            FixedSizeBinaryArray, StructArray, TimestampMillisecondArray, TimestampNanosecondArray,
            TimestampSecondArray, UInt64Array,
        },
        buffer::NullBuffer,
    };
    use pse_catalog::delta::layout::DurableLayout;
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let ids: Arc<dyn Array> = Arc::new(
        FixedSizeBinaryArray::try_from_iter([[1u8; 16], [2; 16], [3; 16]].into_iter()).unwrap(),
    );
    let ordinals: Arc<dyn Array> = Arc::new(UInt64Array::from(vec![Some(0), None, Some(u64::MAX)]));
    let nested_fields = vec![Arc::new(Field::new("ordinal", DataType::UInt64, false))];
    let nested: Arc<dyn Array> = Arc::new(StructArray::new(
        nested_fields.clone().into(),
        vec![ordinals],
        Some(NullBuffer::from(vec![true, false, true])),
    ));
    let schema = Arc::new(Schema::new(vec![
        Field::new("entity_id", ids.data_type().clone(), false).with_metadata(
            std::collections::HashMap::from([
                ("pse.semantic.logical_type".into(), "semantic_id".into()),
                ("ARROW:extension:name".into(), "pse.semantic_id".into()),
                ("ARROW:extension:metadata".into(), "{\"v\":1}".into()),
            ]),
        ),
        Field::new("nested", DataType::Struct(nested_fields.into()), true),
        Field::new("wide", DataType::UInt64, false),
        Field::new(
            "time",
            DataType::Timestamp(
                datafusion::arrow::datatypes::TimeUnit::Nanosecond,
                Some("UTC".into()),
            ),
            false,
        ),
        Field::new(
            "millisecond_ticks",
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Millisecond, None),
            false,
        ),
        Field::new(
            "zoned_second_ticks",
            DataType::Timestamp(
                datafusion::arrow::datatypes::TimeUnit::Second,
                Some("America/New_York".into()),
            ),
            false,
        ),
    ]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            ids,
            nested,
            Arc::new(UInt64Array::from(vec![0, 1u64 << 63, u64::MAX])),
            Arc::new(
                TimestampNanosecondArray::from(vec![1, 1_000_000_001, -1]).with_timezone("UTC"),
            ),
            Arc::new(TimestampMillisecondArray::from(vec![i64::MIN, 0, i64::MAX])),
            Arc::new(
                TimestampSecondArray::from(vec![i64::MIN, 0, i64::MAX])
                    .with_timezone("America/New_York"),
            ),
        ],
    )
    .unwrap();
    let layout = DurableLayout::new(schema).unwrap();
    context.register_batch("typed", batch.clone()).unwrap();
    let input = layout
        .encode(
            context
                .table("typed")
                .await
                .unwrap()
                .into_unoptimized_plan(),
        )
        .unwrap();
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .build()
        .unwrap();
    execute(&context.state(), table, input).await.unwrap();
    let table = DeltaTableBuilder::from_url(location(temp.path()))
        .unwrap()
        .load()
        .await
        .unwrap();
    context
        .register_table(
            "stored",
            Arc::new(
                table
                    .table_provider()
                    .with_session(Arc::new(context.state()))
                    .build()
                    .await
                    .unwrap(),
            ),
        )
        .unwrap();
    let stored = context
        .table("stored")
        .await
        .unwrap()
        .into_unoptimized_plan();
    let field = stored.schema().field(0);
    assert!(!field.metadata().contains_key("ARROW:extension:name"));
    assert_eq!(
        field.metadata()[pse_schema::delta::KEY_LAYOUT_VERSION],
        pse_schema::delta::LAYOUT_VERSION
    );
    let declared: Field =
        serde_json::from_str(&field.metadata()[pse_schema::delta::KEY_EXECUTION_FIELD]).unwrap();
    assert_eq!(&declared, layout.execution_schema().field(0));
    assert_eq!(
        &pse_schema::delta::execution_schema(stored.schema().as_arrow()).unwrap(),
        layout.execution_schema().as_ref(),
    );
    let mut wrong_fields = layout
        .execution_schema()
        .fields()
        .iter()
        .map(|field| field.as_ref().clone())
        .collect::<Vec<_>>();
    wrong_fields[0]
        .metadata_mut()
        .insert("domain.meaning".into(), "different".into());
    let wrong = DurableLayout::new(Arc::new(Schema::new(wrong_fields))).unwrap();
    assert!(wrong.decode(stored.clone()).is_err());
    let decoded = layout.decode(stored).unwrap();
    let state = context.state();
    let results = run_native(&state, &decoded).await.unwrap();
    let actual =
        datafusion::arrow::compute::concat_batches(layout.execution_schema(), &results).unwrap();
    assert_eq!(actual, batch);
}

#[tokio::test]
async fn durable_projection_refuses_overflow_and_bad_identity_width() {
    use datafusion::arrow::array::{BinaryArray, Decimal128Array, Float32Array};
    use pse_catalog::delta::layout::DurableLayout;
    let (context, _, _, _) = context();
    let cases: Vec<(DataType, Arc<dyn Array>)> = vec![
        (
            DataType::Float16,
            Arc::new(Float32Array::from(vec![0.1f32])),
        ),
        (
            DataType::FixedSizeBinary(16),
            Arc::new(BinaryArray::from_vec(vec![&[0u8; 15]])),
        ),
        (
            DataType::UInt64,
            Arc::new(
                Decimal128Array::from(vec![-1i128])
                    .with_precision_and_scale(20, 0)
                    .unwrap(),
            ),
        ),
    ];
    for (execution_type, array) in cases {
        let layout = DurableLayout::new(Arc::new(Schema::new(vec![Field::new(
            "value",
            execution_type,
            false,
        )])))
        .unwrap();
        context.deregister_table("bad").unwrap();
        context
            .register_batch(
                "bad",
                RecordBatch::try_new(Arc::clone(layout.storage_schema()), vec![array]).unwrap(),
            )
            .unwrap();
        let decoded = layout
            .decode(context.table("bad").await.unwrap().into_unoptimized_plan())
            .unwrap();
        let state = context.state();
        let physical = state.create_physical_plan(&decoded).await.unwrap();
        assert!(collect(physical, state.task_ctx()).await.is_err());
    }
}

#[tokio::test]
async fn cold_native_hierarchy_pins_versions_and_views_refuse_write_bypass() {
    use datafusion::catalog::{
        CatalogProvider, MemoryCatalogProvider, MemorySchemaProvider, SchemaProvider,
    };
    use pse_catalog::delta::{layout::DurableLayout, provider::open_view};
    let temp = tempfile::tempdir().unwrap();
    let (writer, _, _, _) = context();
    let layout = DurableLayout::new(Arc::new(Schema::new(vec![Field::new(
        "id",
        DataType::Int64,
        true,
    )])))
    .unwrap();
    let location = location(temp.path());
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .build()
        .unwrap();
    execute(
        &writer.state(),
        table,
        layout.encode(input(&writer, vec![7]).await).unwrap(),
    )
    .await
    .unwrap();
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    execute(
        &writer.state(),
        table,
        layout.encode(input(&writer, vec![9]).await).unwrap(),
    )
    .await
    .unwrap();
    drop(writer);
    let (reader, _, _, _) = context();
    // The sentinel UDF declares nullable output. The stored contract therefore
    // cannot be reopened as a non-null field merely because these rows are valid.
    let wrong = DurableLayout::new(Arc::new(Schema::new(vec![Field::new(
        "id",
        DataType::Int64,
        false,
    )])))
    .unwrap();
    assert!(
        open_view(location.clone(), 0, &wrong, Arc::new(reader.state()))
            .await
            .is_err()
    );
    let layout = DurableLayout::new(Arc::new(Schema::new(vec![Field::new(
        "id",
        DataType::Int64,
        true,
    )])))
    .unwrap();
    let view = open_view(location.clone(), 0, &layout, Arc::new(reader.state()))
        .await
        .unwrap();
    let schema = Arc::new(MemorySchemaProvider::new());
    schema
        .register_table("source.table".into(), Arc::new(view))
        .unwrap();
    let catalog = Arc::new(MemoryCatalogProvider::new());
    catalog.register_schema("typed schema", schema).unwrap();
    reader.register_catalog("cold model", catalog);
    let rows = reader
        .sql("SELECT id FROM \"cold model\".\"typed schema\".\"source.table\"")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(rows.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert_eq!(
        rows[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        7
    );
    let command = reader
        .sql("INSERT INTO \"cold model\".\"typed schema\".\"source.table\" VALUES (42)")
        .await;
    if let Ok(frame) = command {
        assert!(
            frame.collect().await.is_err(),
            "view must not expose Delta's raw insert hook"
        );
    }
    assert_eq!(
        DeltaTableBuilder::from_url(location)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(1)
    );
}

#[tokio::test]
async fn float_precision_admission_checks_visible_children_and_preserves_signed_zero() {
    use datafusion::arrow::{
        array::{Float32Array, StructArray},
        buffer::NullBuffer,
    };
    use pse_catalog::delta::layout::DurableLayout;
    let (context, _, _, _) = context();
    let layout = DurableLayout::new(Arc::new(Schema::new(vec![Field::new(
        "value",
        DataType::Struct(vec![Field::new("number", DataType::Float16, false)].into()),
        true,
    )])))
    .unwrap();
    for visible in [false, true] {
        let DataType::Struct(fields) = layout.storage_schema().field(0).data_type() else {
            panic!("declared struct shape")
        };
        let values = Arc::new(StructArray::new(
            fields.clone(),
            vec![Arc::new(Float32Array::from(vec![0.125, 0.1, -0.0]))],
            Some(NullBuffer::from(vec![true, visible, true])),
        ));
        context.deregister_table("precision").unwrap();
        context
            .register_batch(
                "precision",
                RecordBatch::try_new(Arc::clone(layout.storage_schema()), vec![values]).unwrap(),
            )
            .unwrap();
        let decoded = layout
            .decode(
                context
                    .table("precision")
                    .await
                    .unwrap()
                    .into_unoptimized_plan(),
            )
            .unwrap();
        let state = context.state();
        let result = run_native(&state, &decoded).await;
        assert_eq!(
            result.is_err(),
            visible,
            "only visible precision loss is invalid"
        );
        if !visible {
            let batches = result.unwrap();
            let value = batches[0]
                .column(0)
                .as_any()
                .downcast_ref::<StructArray>()
                .unwrap();
            let numbers =
                datafusion::arrow::compute::cast(value.column(0), &DataType::Float32).unwrap();
            let numbers = numbers.as_any().downcast_ref::<Float32Array>().unwrap();
            assert_eq!(numbers.value(0).to_bits(), 0.125_f32.to_bits());
            assert_eq!(numbers.value(2).to_bits(), (-0.0f32).to_bits());
            assert!(value.is_null(1));
        }
    }
}

#[tokio::test]
async fn durable_admission_rejects_nested_relabeling_before_physical_planning() {
    use pse_catalog::delta::layout::DurableLayout;
    let (context, _, _, _) = context();
    let schema = |child: Field| {
        Arc::new(Schema::new(vec![Field::new(
            "record",
            DataType::Struct(vec![child].into()),
            true,
        )]))
    };
    let child = Field::new("source_id", DataType::Int64, false);
    let layout = DurableLayout::new(schema(child.clone())).unwrap();
    for changed in [
        child.clone().with_name("target_id"),
        child.with_nullable(true),
    ] {
        context.deregister_table("mismatched").unwrap();
        context
            .register_batch("mismatched", RecordBatch::new_empty(schema(changed)))
            .unwrap();
        let input = context
            .table("mismatched")
            .await
            .unwrap()
            .into_unoptimized_plan();
        assert!(layout.encode(input.clone()).is_err());
        assert!(layout.decode(input).is_err());
    }
}

#[tokio::test]
async fn conditional_control_update_conflicts_with_a_stale_writer() {
    use datafusion::logical_expr::lit;
    use deltalake::delta_datafusion::SessionFallbackPolicy;
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let location = location(temp.path());
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .build()
        .unwrap();
    execute(&context.state(), table, input(&context, vec![0]).await)
        .await
        .unwrap();
    let base = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    let update = |table: DeltaTable, value: i64| {
        table
            .update()
            .with_session_state(Arc::new(context.state()))
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_predicate(col("id").eq(lit(0i64)))
            .with_update("id", lit(value))
    };
    let (winner, metrics) = update(base.clone(), 1).await.unwrap();
    assert_eq!(metrics.num_updated_rows, 1);
    assert!(
        update(base, 2).await.is_err(),
        "a stale publication parent must conflict"
    );
    let version = winner.version();
    let (unchanged, metrics) = update(winner, 3).await.unwrap();
    assert_eq!(
        metrics.num_updated_rows, 0,
        "fresh wrong-parent match must be empty"
    );
    assert_eq!(
        unchanged.version(),
        version,
        "an unmatched expected parent must not commit"
    );
    let final_table = DeltaTableBuilder::from_url(location)
        .unwrap()
        .load()
        .await
        .unwrap();
    assert_eq!(final_table.version(), version);
}

#[tokio::test]
async fn application_transaction_records_do_not_implement_replay_deduplication() {
    use deltalake::kernel::Transaction;
    let temp = tempfile::tempdir().unwrap();
    let (context, _, _, _) = context();
    let location = location(temp.path());
    let mut table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .build()
        .unwrap();
    // Characterize the library: PSE publication must reconcile attempt/input identity
    // before a retry. Merely attaching SetTransaction does not enforce that contract.
    for _ in 0..2 {
        let request = DeltaWrite::plan(
            table,
            input(&context, vec![1]).await,
            SaveMode::Append,
            CommitProperties::default().with_application_transaction(Transaction::new("probe", 4)),
        )
        .unwrap();
        let state = context.state();
        run_native(&state, &request).await.unwrap();
        table = DeltaTableBuilder::from_url(location.clone())
            .unwrap()
            .load()
            .await
            .unwrap();
    }
    assert_eq!(table.version(), Some(1));
    assert_eq!(
        table
            .snapshot()
            .unwrap()
            .transaction_version(table.log_store().as_ref(), "probe")
            .await
            .unwrap(),
        Some(4)
    );
}

#[tokio::test]
async fn generated_publication_control_reopens_its_exact_member_catalog() {
    use pse_catalog::delta::publication::PublicationRoot;
    let temp = tempfile::tempdir().unwrap();
    let (writer, _, _, _) = context();
    let registry = pse_engine::validation::registry().unwrap();
    let relation = registry.relation("authored.entities").unwrap();
    let schema = Arc::new(pse_schema::arrow::relation_schema(registry, relation).unwrap());
    let member = write_member(
        &writer,
        temp.path(),
        "authored.entities",
        RecordBatch::new_empty(schema),
    )
    .await;
    let mut row = publication_row(2, None);
    row.members.push(member);
    let control_uri = location(&temp.path().join("control"));
    write_control(&writer, control_uri.clone(), row).await;
    drop(writer);
    let (reader, _, _, _) = context();
    let publication = open_publication(
        PublicationRoot {
            location: control_uri,
            version: 1,
        },
        registry,
        Arc::new(reader.state()),
    )
    .await
    .unwrap();
    assert_eq!(
        publication.record().publication_id,
        pse_ids::SemanticId::from_bytes([2; 16])
    );
    assert_eq!(
        publication_count(&publication, "SELECT * FROM model.authored.entities").await,
        0
    );
}

#[tokio::test]
async fn publication_selection_preserves_full_tables_and_exact_identity_slices() {
    use pse_catalog::delta::publication::PublicationRoot;
    use pse_relations::generated::{authored::entities, enums::EntityKind};
    use publications::{
        RuntimePublicationsFieldMembersItemSelection as Selection,
        RuntimePublicationsFieldMembersItemSelectionRevision as Revision,
    };
    let root = tempfile::tempdir().unwrap();
    let (writer, _, _, _) = context();
    let identity = |value| pse_ids::SemanticId::from_bytes([value; 16]);
    let mut builder = entities::Builder::new().unwrap();
    for id in [10, 11] {
        builder
            .push(entities::Row {
                entity_id: identity(id),
                package_id: identity(1),
                kind: EntityKind::Package,
                name: format!("entity-{id}"),
                qualified_name: format!("entity-{id}"),
                parent_entity_id: None,
                source_span: None,
            })
            .unwrap();
    }
    let member = write_member(
        &writer,
        root.path(),
        "authored.entities",
        builder.finish().unwrap().into_batch(),
    )
    .await;
    for (case, selection, expected) in [
        ("full", Selection::from_full(), Some(2)),
        (
            "slice",
            Selection::from_revision(Revision {
                column: "entity_id".into(),
                revision_id: identity(10),
            }),
            Some(1),
        ),
        (
            "empty",
            Selection::from_revision(Revision {
                column: "entity_id".into(),
                revision_id: identity(99),
            }),
            Some(0),
        ),
        (
            "unknown",
            Selection::from_revision(Revision {
                column: "not_declared".into(),
                revision_id: identity(10),
            }),
            None,
        ),
        (
            "wrong-type",
            Selection::from_revision(Revision {
                column: "name".into(),
                revision_id: identity(10),
            }),
            None,
        ),
    ] {
        let mut selected = member.clone();
        selected.selection = selection;
        let mut row = publication_row(2, None);
        row.members.push(selected);
        let uri = location(&root.path().join(case));
        write_control(&writer, uri.clone(), row).await;
        let result = open_publication(
            PublicationRoot {
                location: uri,
                version: 1,
            },
            pse_engine::validation::registry().unwrap(),
            Arc::new(writer.state()),
        )
        .await;
        if let Some(expected) = expected {
            let publication = result.unwrap();
            assert_eq!(
                publication_count(&publication, "SELECT * FROM model.authored.entities").await,
                expected,
                "{case}"
            );
        } else {
            assert!(result.is_err(), "{case}");
        }
    }
}

async fn write_control(context: &SessionContext, uri: url::Url, row: publications::Row) {
    let registry = pse_engine::validation::registry().unwrap();
    let contract = pse_catalog::delta::contract::DeclaredCheck::new(
        registry,
        publications::spec(registry).unwrap().id,
    )
    .unwrap();
    let mut builder = publications::Builder::new().unwrap();
    builder.push(row).unwrap();
    let batch = builder.finish().unwrap().into_batch();
    let plan = DeltaWrite::declared(
        DeltaTableBuilder::from_url(uri).unwrap().build().unwrap(),
        context.read_batch(batch).unwrap().into_unoptimized_plan(),
        SaveMode::ErrorIfExists,
        CommitProperties::default(),
        contract,
    )
    .unwrap();
    let state = context.state();
    run_native(&state, &plan).await.unwrap();
}

#[tokio::test]
async fn publication_root_records_native_checks_and_refuses_empty_or_invalid_heads() {
    use deltalake::delta_datafusion::SessionFallbackPolicy;
    use pse_catalog::delta::{contract::DeclaredCheck, publication::PublicationRoot};
    let root = tempfile::tempdir().unwrap();
    let uri = location(root.path());
    let (writer, _, _, _) = context();
    assert_eq!(
        publish(&writer.state(), uri.clone(), publication_row(2, None))
            .await
            .unwrap(),
        1
    );
    let registry = pse_engine::validation::registry().unwrap();
    assert!(
        open_publication(
            PublicationRoot {
                location: uri.clone(),
                version: 0
            },
            registry,
            Arc::new(writer.state())
        )
        .await
        .is_err()
    );
    let table = DeltaTableBuilder::from_url(uri.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    let cold = SessionStateBuilder::new_with_default_features()
        .with_query_planner(deltalake::delta_datafusion::planner::DeltaPlanner::new())
        .build();
    let contract = DeclaredCheck::open(&table, &cold).unwrap();
    assert!(
        contract
            .properties()
            .contains_key("delta.constraints.pse_contract")
    );
    assert!(
        contract
            .properties()
            .contains_key(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
    );
    let cold = Arc::new(contract.bind(&cold).unwrap());
    let result = table
        .update()
        .with_session_state(cold)
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .with_update("kind", datafusion::logical_expr::lit("invalid-kind"))
        .await;
    assert!(result.is_err());
    assert_eq!(
        DeltaTableBuilder::from_url(uri)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(1)
    );
}

#[tokio::test]
async fn a_lost_control_schema_creation_response_is_reconciled_before_publishing() {
    use fault_store::{Fault, FaultPlan, FaultStore};
    let (context, _, _, runtime) = context();
    let location = url::Url::parse("memory://creation/control/").unwrap();
    let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    runtime.register_object_store(&location, store.clone());
    store.arm(FaultPlan {
        operation: "put",
        prefix: "control/_delta_log/00000000000000000000.json".into(),
        call: 1,
        fault: Fault::LostResponse,
    });
    let row = publication_row(2, None);
    assert_eq!(
        publish(&context.state(), location.clone(), row.clone())
            .await
            .unwrap(),
        1
    );
    assert_eq!(store.fired(), 1);
    assert_eq!(publish(&context.state(), location, row).await.unwrap(), 1);
}

async fn remove_native_check(member: &mut publications::RuntimePublicationsFieldMembersItem) {
    let table = DeltaTableBuilder::from_url(member.table_uri.parse().unwrap())
        .unwrap()
        .load()
        .await
        .unwrap()
        .drop_constraints()
        .with_constraint("pse_contract")
        .await
        .unwrap();
    member.delta_version = i64::try_from(table.version().unwrap()).unwrap();
}

fn native_factory(state: &SessionState) -> pse_engine::session::EngineFactory {
    pse_engine::session::EngineFactory::from_builder(
        Arc::clone(state.runtime_env()),
        Arc::clone(&state.runtime_env().memory_pool),
        "delta-fixture",
        SessionStateBuilder::new_from_existing(state.clone()),
    )
}
async fn open_publication(
    root: pse_catalog::delta::publication::PublicationRoot,
    registry: &pse_schema::Registry,
    state: Arc<SessionState>,
) -> std::result::Result<pse_catalog::delta::publication::Publication, pse_engine::EngineError> {
    assert_eq!(
        registry.fingerprint(),
        pse_engine::validation::registry().unwrap().fingerprint()
    );
    pse_catalog::delta::publication::Publication::open(
        root,
        pse_schema::shared_registry().unwrap(),
        &native_factory(&state),
        &pse_columnar::CancellationToken::new(),
    )
    .await
}
async fn publication_count(
    publication: &pse_catalog::delta::publication::Publication,
    sql: &str,
) -> usize {
    publication
        .session()
        .sql(sql, &pse_columnar::CancellationToken::new())
        .await
        .unwrap()
        .iter()
        .map(RecordBatch::num_rows)
        .sum()
}

fn prepare_native(
    state: &SessionState,
    plan: &LogicalPlan,
) -> Result<pse_engine::session::PreparedComputation> {
    prepare_native_observed(
        state,
        plan,
        pse_engine::session::assurance::ObservationPolicy::Contract,
    )
}
fn prepare_native_observed(
    state: &SessionState,
    plan: &LogicalPlan,
    policy: pse_engine::session::assurance::ObservationPolicy,
) -> Result<pse_engine::session::PreparedComputation> {
    use datafusion::common::tree_node::TreeNodeRecursion;
    let cancel = pse_columnar::CancellationToken::new();
    let mut session = native_factory(state)
        .with_observation(policy)
        .candidate(
            std::collections::BTreeMap::default(),
            pse_schema::shared_registry().unwrap(),
            &cancel,
        )
        .map_err(datafusion::common::DataFusionError::from)?
        .with_purpose(pse_schema::model::provider::OperationPurpose::Publish);
    plan.apply_with_subqueries(|node| {
        if let LogicalPlan::TableScan(scan) = node {
            let source = datafusion::datasource::source_as_provider(&scan.source)?;
            session = session
                .with_provider(scan.table_name.clone(), source, &cancel)
                .map_err(datafusion::common::DataFusionError::from)?;
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    session
        .prepare(plan.clone(), &cancel)
        .map_err(datafusion::common::DataFusionError::from)
}
async fn run_native(state: &SessionState, plan: &LogicalPlan) -> Result<Vec<RecordBatch>> {
    Ok(prepare_native(state, plan)?
        .execute(&pse_columnar::CancellationToken::new())
        .await
        .map_err(datafusion::common::DataFusionError::from)?
        .into_batches())
}
