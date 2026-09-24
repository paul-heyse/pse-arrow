// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Differential completeness over an actual admitted provider and an unpruned source.
#![allow(
    clippy::expect_used,
    reason = "fixture assertions identify exact failures"
)]
#[path = "../../support/native_publication.rs"]
mod native_publication;
#[path = "../src/oracle.rs"]
mod oracle;
use datafusion::{
    arrow::{array::RecordBatch, compute::concat_batches, datatypes::SchemaRef},
    catalog::TableProvider,
    datasource::MemTable,
    logical_expr::TableProviderFilterPushDown,
};
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, ColumnRole, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl,
        SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

async fn fixture() -> (Arc<dyn TableProvider>, RecordBatch, tempfile::TempDir) {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    pse_schema::catalog::declare_publications(&mut builder);
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new("one", "One"), EnumMember::new("two", "Two")],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "items",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "oracle rows",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(arrow::datatypes::DataType::Int64),
                "key",
            ),
            FieldContract::payload("choice", FieldContract::enumeration("Choice"), "enum")
                .optional(),
            FieldContract::payload(
                "label",
                FieldContract::native(arrow::datatypes::DataType::Utf8),
                "payload",
            ),
            FieldContract::new(
                "parent",
                FieldContract::native(arrow::datatypes::DataType::Int64),
                true,
                ColumnRole::Reference,
                "reference",
            )
            .optional(),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.items").expect("relation");
    let rows = (1..=36)
        .map(|id| {
            vec![
                serde_json::json!(["i64", id]),
                if id % 3 == 0 {
                    serde_json::json!(["null", null])
                } else {
                    serde_json::json!(["enum", if id % 2 == 0 { "one" } else { "two" }])
                },
                serde_json::json!(["text", format!("row-{id}")]),
                if id % 2 == 0 {
                    serde_json::json!(["i64", 1])
                } else {
                    serde_json::json!(["null", null])
                },
            ]
        })
        .collect::<Vec<_>>();
    let batch = pse_relations::testing::batch_from_literals(&registry, spec, &rows).expect("batch");
    let key = spec.key;
    let (publication, directory, _) =
        native_publication::publish(registry, BTreeMap::from([(key, batch.clone())])).await;
    let table = datafusion::datasource::source_as_provider(
        &publication
            .session()
            .table_source(&key)
            .expect("native selected provider"),
    )
    .expect("provider");
    (table, batch, directory)
}
fn unpruned(batch: RecordBatch) -> Arc<dyn TableProvider> {
    Arc::new(MemTable::try_new(batch.schema(), vec![vec![batch]]).expect("unpruned source"))
}

/// Deliberately lies about exact filtering; only used to prove oracle completeness.
#[derive(Debug)]
struct DefectiveExact(MemTable);
impl TableProvider for DefectiveExact {
    fn schema(&self) -> SchemaRef {
        self.0.schema()
    }
    fn table_type(&self) -> datafusion::logical_expr::TableType {
        self.0.table_type()
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&datafusion::logical_expr::Expr],
    ) -> datafusion::common::Result<Vec<TableProviderFilterPushDown>> {
        Ok(vec![TableProviderFilterPushDown::Exact; filters.len()])
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        state: &'t dyn datafusion::catalog::Session,
        projection: Option<&'p Vec<usize>>,
        filters: &'f [datafusion::logical_expr::Expr],
        limit: Option<usize>,
    ) -> pse_engine::BoxFut<
        'future,
        datafusion::common::Result<Arc<dyn datafusion::physical_plan::ExecutionPlan>>,
    >
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        assert!(
            !filters.is_empty(),
            "negative control must exercise claimed Exact pushdown"
        );
        self.0.scan(state, projection, &[], limit)
    }
}
#[tokio::test]
async fn all_declared_filter_shapes_match_complete_unpruned_results() {
    let (table, batch, _directory) = fixture().await;
    let baseline = unpruned(batch);
    let cases = [
        "SELECT * FROM items WHERE id = 1",
        "SELECT * FROM items WHERE id IN (1,2)",
        "SELECT * FROM items WHERE id IN (1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20)",
        "SELECT * FROM items WHERE id=1 OR id=3",
        "SELECT * FROM items WHERE id IN (1,2,3,4) AND label LIKE '%2'",
        "SELECT * FROM items WHERE parent IS NULL",
        "SELECT * FROM items WHERE parent IS NOT NULL",
        "SELECT * FROM items WHERE choice='one'",
        "SELECT label FROM items WHERE id=2",
        "SELECT label FROM items WHERE id IN (1,2,3) LIMIT 2",
        "SELECT i.label AS value FROM items AS i WHERE i.id=4",
    ];
    for sql in cases {
        let actual = oracle::query(table.clone(), sql).await;
        let expected = oracle::query(Arc::clone(&baseline), sql).await;
        assert_eq!(
            oracle::multiset(&actual),
            oracle::multiset(&expected),
            "{sql}"
        );
    }
}
#[tokio::test]
async fn oracle_detects_over_pruning_empty_and_extra_rows() {
    let (_table, batch, _directory) = fixture().await;
    let sql = "SELECT * FROM items WHERE id IN (1,2)";
    let expected = oracle::multiset(&oracle::query(unpruned(batch.clone()), sql).await);
    let over_pruning = batch.slice(0, 1);
    let empty = batch.slice(0, 0);
    let extra = concat_batches(&batch.schema(), &[batch.slice(0, 2), batch.slice(0, 1)])
        .expect("duplicate a real match");
    for defective in [over_pruning, empty, extra] {
        let table = Arc::new(DefectiveExact(
            MemTable::try_new(defective.schema(), vec![vec![defective]]).expect("defective source"),
        ));
        let actual = oracle::multiset(&oracle::query(table, sql).await);
        assert_ne!(actual, expected, "complete oracle must catch every defect");
    }
}
