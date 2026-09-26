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
use pse_engine::session::{EngineSession, ExecutionSettings, ThreadBudget, native_engine_profile};

use pse_columnar::{CancellationToken, MemoryPool};
use pse_schema::{
    RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn fixture() -> (EngineSession, RecordBatch, Arc<dyn MemoryPool>) {
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
            FieldContract::key(
                "id",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Key.",
            ),
            FieldContract::payload(
                "label",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                "Value.",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.samples").expect("relation");
    let rows = vec![
        vec![
            serde_json::json!(["u64", 1]),
            serde_json::json!(["text", "a"]),
        ],
        vec![
            serde_json::json!(["u64", 1]),
            serde_json::json!(["text", "b"]),
        ],
        vec![
            serde_json::json!(["u64", 2]),
            serde_json::json!(["text", "c"]),
        ],
    ];
    let batch = pse_relations::testing::batch_from_literals(&registry, spec, &rows)
        .expect("schema admitted duplicates");
    let budget: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8 << 20));
    let pool: Arc<dyn MemoryPool> = budget.clone();
    let runtime: Arc<RuntimeEnv> = Arc::new(
        RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(GreedyMemoryPool::new(8 << 20)))
            .build()
            .expect("runtime"),
    );
    let thread = NonZeroUsize::new(1).expect("one");
    let session = pse_engine::EngineFactory::new(
        runtime,
        pool,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: thread,
            target_partitions: thread,
        },
        native_engine_profile(),
    )
    .and_then(|factory| {
        factory.candidate(
            BTreeMap::from([(spec.key, batch.clone())]),
            registry.clone(),
            &CancellationToken::new(),
        )
    })
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
    assert!(!all.contains_key("datafusion.pse.null_policy"));
    assert!(!all.contains_key("datafusion.pse.kernel_outcome_policies"));
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
