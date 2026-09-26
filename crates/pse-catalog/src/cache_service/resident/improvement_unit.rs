// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fake native provider scans; no Delta commits or replay.
use super::*;
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    datasource::MemTable,
    execution::{
        memory_pool::{GreedyMemoryPool, MemoryPool},
        runtime_env::RuntimeEnvBuilder,
        session_state::SessionStateBuilder,
    },
    logical_expr::{TableProviderFilterPushDown, col, lit},
    physical_plan::collect,
};
use pse_relations::generated::runtime::publications::{
    RuntimePublicationsFieldMembersItem as Member,
    RuntimePublicationsFieldMembersItemSelection as Selection,
};

type Scan = (Option<Vec<usize>>, usize, Option<usize>);

#[tokio::test]
async fn selected_decoded_view_preserves_root_metadata_without_residency() {
    use datafusion::{
        common::DFSchema,
        datasource::ViewTable,
        logical_expr::{LogicalPlan, LogicalPlanBuilder, Projection},
    };
    let context = datafusion::prelude::SessionContext::new();
    let empty = Arc::new(LogicalPlanBuilder::empty(true).build().unwrap());
    let mut projection = Projection::try_new(vec![lit(17_i64).alias("value")], empty).unwrap();
    let schema = Arc::new(Schema::new_with_metadata(
        projection.schema.fields().clone(),
        std::collections::HashMap::from([("pse.contract.test".into(), "declared".into())]),
    ));
    projection.schema = Arc::new(DFSchema::try_from(schema.as_ref().clone()).unwrap());
    let provider = crate::delta::provider::selected_view(ViewTable::new(
        LogicalPlan::Projection(projection),
        None,
    ));
    let physical = provider
        .scan(&context.state(), None, &[], None)
        .await
        .unwrap();
    assert_eq!(physical.schema(), schema);
    context.register_table("decoded", provider).unwrap();
    let batches = context
        .sql("SELECT sum(value) FROM decoded")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(
        batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        17
    );
}
#[derive(Debug)]
struct Observed {
    table: MemTable,
    scans: Mutex<Vec<Scan>>,
}
#[async_trait::async_trait]
impl TableProvider for Observed {
    fn schema(&self) -> SchemaRef {
        self.table.schema()
    }
    fn table_type(&self) -> TableType {
        TableType::Base
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        Ok(vec![TableProviderFilterPushDown::Inexact; filters.len()])
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.scans
            .lock()
            .unwrap()
            .push((projection.cloned(), filters.len(), limit));
        self.table.scan(state, projection, &[], limit).await
    }
}

async fn fixture() -> (tempfile::TempDir, SelectedTable, Arc<Observed>) {
    let directory = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(directory.path()).unwrap();
    drop(
        crate::delta::lease::write(&location, &pse_columnar::CancellationToken::new())
            .await
            .unwrap(),
    );
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64 << 20));
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .unwrap();
    let state = Arc::new(
        SessionStateBuilder::new_with_default_features()
            .with_runtime_env(runtime)
            .build(),
    );
    let service =
        DeltaCacheService::new(super::super::DeltaCacheBudget::for_memory(64 << 20), &pool)
            .unwrap();
    let schema = Arc::new(Schema::new(vec![
        Field::new("key", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(Int64Array::from(vec![10, 20, 30])),
        ],
    )
    .unwrap();
    let inner = Arc::new(Observed {
        table: MemTable::try_new(schema, vec![vec![batch]]).unwrap(),
        scans: Mutex::default(),
    });
    let table = SelectedTable {
        inner: inner.clone(),
        service,
        state,
        selection: Arc::new(MemberSelection(Member {
            catalog_name: "unit".into(),
            schema_name: "unit".into(),
            table_name: "values".into(),
            relation_id: pse_ids::SemanticId::NIL,
            relation_version: 1,
            contract_fingerprint: pse_ids::ContentHash::NIL,
            table_uri: location.to_string(),
            delta_version: 0,
            selection: Selection {
                kind: pse_relations::generated::enums::MemberSelectionKind::Full,
                revision: None,
            },
        })),
        location,
        interpretation: Arc::new(Interpretation {
            settings: Arc::default(),
            implementation: pse_ids::SemanticId::NIL,
            policies: Arc::default(),
        }),
    };
    (directory, table, inner)
}

#[tokio::test]
async fn cold_narrow_reads_preserve_native_pruning_and_never_fill_complete_entries() {
    let (_directory, table, inner) = fixture().await;
    let table = Arc::new(table);
    let filters = vec![col("key").eq(lit(2_i64))];
    let plan = table
        .scan(table.state.as_ref(), Some(&vec![1]), &filters, Some(1))
        .await
        .unwrap();
    assert_eq!(
        inner.scans.lock().unwrap().as_slice(),
        &[(Some(vec![1]), 1, None)]
    );
    assert_eq!(table.service.resident.report().entries, 0);
    let output = collect(plan, table.state.task_ctx()).await.unwrap();
    // Inexact pruning leaves the predicate to the outer native FilterExec.
    assert_eq!(output[0].num_columns(), 1);
    assert_eq!(table.service.resident.report().entries, 0);
    let plan = table
        .scan(table.state.as_ref(), None, &[], Some(1))
        .await
        .unwrap();
    collect(plan, table.state.task_ctx()).await.unwrap();
    assert_eq!(table.service.resident.report().entries, 0);
    assert_eq!(table.service.resident.loads.load(Ordering::Relaxed), 0);
    assert_filtered_value(table).await;
}

async fn assert_filtered_value(table: Arc<SelectedTable>) {
    let context = datafusion::prelude::SessionContext::new_with_state(table.state.as_ref().clone());
    let batches = context
        .read_table(table)
        .unwrap()
        .filter(col("key").eq(lit(2_i64)))
        .unwrap()
        .select_columns(&["value"])
        .unwrap()
        .limit(0, Some(1))
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert_eq!(
        batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        20
    );
}

#[tokio::test]
async fn warm_complete_entries_project_and_limit_without_loading_partial_results() {
    let (_directory, table, _inner) = fixture().await;
    let table = Arc::new(table);
    let plan = table
        .scan(table.state.as_ref(), None, &[], None)
        .await
        .unwrap();
    collect(plan, table.state.task_ctx()).await.unwrap();
    assert_eq!(table.service.resident.report().entries, 1);
    let plan = table
        .scan(table.state.as_ref(), Some(&vec![1]), &[], Some(1))
        .await
        .unwrap();
    let batches = collect(plan, table.state.task_ctx()).await.unwrap();
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert_eq!(batches[0].schema().field(0).name(), "value");
    assert_eq!(table.service.resident.loads.load(Ordering::Relaxed), 1);
    assert_eq!(table.service.resident.report().hits, 1);
    assert_filtered_value(table.clone()).await;
    assert_eq!(table.service.resident.report().hits, 2);
    assert_eq!(table.service.resident.loads.load(Ordering::Relaxed), 1);
}
