// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Candidate constraints, active plan admission and exact engine setting behavior.

use datafusion::arrow::array::{Int64Array, RecordBatch};
use datafusion::catalog::TableProvider;
use datafusion::datasource::{MemTable, provider_as_source};
use datafusion::execution::{
    memory_pool::GreedyMemoryPool,
    runtime_env::{RuntimeEnv, RuntimeEnvBuilder},
};
use datafusion::logical_expr::LogicalPlanBuilder;
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    phase0_reference_profile,
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver};
use pse_schema::{
    RegistryBuilder,
    model::{Authority, Cell, ColumnSpec, LogicalType, Namespace, RelationDecl, SnapshotClass},
};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn fixture() -> (SnapshotSession, RecordBatch, Arc<FixedBudget>) {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "samples",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Candidate rows.",
        )
        .pk(&["id"])
        .columns(vec![
            ColumnSpec::key("id", LogicalType::U64, "Key."),
            ColumnSpec::payload("label", LogicalType::Text, "Value."),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.samples").expect("relation");
    let rows = vec![
        vec![Cell::U64(1), Cell::text("a")],
        vec![Cell::U64(1), Cell::text("b")],
        vec![Cell::U64(2), Cell::text("c")],
    ];
    let batch = pse_relations::cells::batch_from_cells(&registry, spec, &rows)
        .expect("schema admitted duplicates");
    let budget = FixedBudget::new(8 << 20);
    let reserver: Arc<dyn MemoryReserver> = budget.clone();
    let runtime: Arc<RuntimeEnv> = Arc::new(
        RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(GreedyMemoryPool::new(8 << 20)))
            .build()
            .expect("runtime"),
    );
    let thread = NonZeroUsize::new(1).expect("one");
    let session = build_candidate_session(
        BTreeMap::from([(spec.key, batch.clone())]),
        registry,
        runtime,
        reserver,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: thread,
            target_partitions: thread,
        },
        phase0_reference_profile(),
    )
    .expect("session");
    (session, batch, budget)
}
#[tokio::test]
async fn candidates_keep_duplicates_visible_and_reject_foreign_and_mutation_plans() {
    let (session, batch, _budget) = fixture();
    let cancel = CancellationToken::default();
    let out = session
        .sql("SELECT count(*) AS n FROM authored.samples", &cancel)
        .await
        .expect("count candidates");
    assert_eq!(
        out[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("count")
            .value(0),
        3
    );
    let table: Arc<dyn TableProvider> =
        Arc::new(MemTable::try_new(batch.schema(), vec![vec![batch]]).expect("foreign table"));
    let plan = LogicalPlanBuilder::scan("foreign", provider_as_source(table), None)
        .expect("scan")
        .build()
        .expect("plan");
    assert!(session.execute_plan(plan, &cancel).await.is_err());
    assert!(
        session
            .sql("DELETE FROM authored.samples", &cancel)
            .await
            .is_err()
    );
    assert!(
        session
            .sql("SET datafusion.execution.target_partitions = 4", &cancel)
            .await
            .is_err()
    );
    let key = session
        .registry()
        .relation("authored.samples")
        .expect("spec")
        .key;
    assert!(
        session
            .table_source(&key)
            .expect("source")
            .constraints()
            .is_none()
    );
}
#[tokio::test]
async fn actual_settings_and_owned_result_lifetimes_are_checked() {
    let (session, _batch, budget) = fixture();
    let all = session
        .read_back_settings()
        .await
        .expect("read actual settings");
    assert_eq!(
        all["datafusion.execution.enable_ansi_mode"].as_deref(),
        Some("false")
    );
    assert_eq!(
        all["datafusion.execution.skip_physical_aggregate_schema_check"].as_deref(),
        Some("false")
    );
    assert_eq!(
        all["datafusion.pse.null_policy"].as_deref(),
        Some("four_valued")
    );
    let result = session
        .sql(
            "SELECT id FROM authored.samples WHERE id = 1",
            &CancellationToken::default(),
        )
        .await
        .expect("query");
    assert_eq!(result.iter().map(RecordBatch::num_rows).sum::<usize>(), 2);
    let array = result[0].column(0).clone();
    drop(result);
    drop(session);
    assert!(budget.reserved() > 0, "detached array retains reservation");
    drop(array);
    assert_eq!(budget.reserved(), 0);
}
