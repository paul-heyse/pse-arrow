// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Differential completeness over an actual admitted provider and an unpruned source.
#![allow(
    clippy::expect_used,
    reason = "fixture assertions identify exact failures"
)]
#[path = "../src/oracle.rs"]
mod oracle;
use datafusion::{
    arrow::{array::RecordBatch, compute::concat_batches, datatypes::SchemaRef},
    catalog::TableProvider,
    datasource::MemTable,
    logical_expr::{TableProviderFilterPushDown, col, lit},
};
use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, TrustLevel,
    provider::table::RelationTable,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SnapshotKind};
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, Cell, ColumnRole, ColumnSpec, EnumDecl, EnumMember, LogicalType, Namespace,
        RelationDecl, SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

async fn fixture() -> (Arc<RelationTable>, RecordBatch) {
    let mut builder = RegistryBuilder::new();
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
            ColumnSpec::key("id", LogicalType::I64, "key"),
            ColumnSpec::payload("choice", LogicalType::enumeration("Choice"), "enum").optional(),
            ColumnSpec::payload("label", LogicalType::Text, "payload"),
            ColumnSpec::new(
                "parent",
                LogicalType::I64,
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
                Cell::I64(id),
                if id % 3 == 0 {
                    Cell::Null
                } else {
                    Cell::Enum(if id % 2 == 0 { "one" } else { "two" })
                },
                Cell::text(format!("row-{id}")),
                if id % 2 == 0 {
                    Cell::I64(1)
                } else {
                    Cell::Null
                },
            ]
        })
        .collect::<Vec<_>>();
    let batch = pse_relations::cells::batch_from_cells(&registry, spec, &rows).expect("batch");
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(64 << 20);
    let catalog = Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        Arc::clone(&registry),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        Arc::clone(&reserver),
    );
    let context = AdmissionContext::default();
    let snapshot = catalog
        .publish_bundle(
            BundleDraft {
                manifest: catalog
                    .manifest_template(SnapshotKind::Model, &context)
                    .expect("manifest"),
                context,
                relations: BTreeMap::from([(
                    pse_ids::model_port_name("authored", spec.id),
                    RelationDraft {
                        contract: Arc::new(
                            RelationContract::from_spec(&registry, spec, EncodingPolicy::IpcFile)
                                .expect("contract"),
                        ),
                        batches: vec![batch],
                    },
                )]),
            },
            &CancellationToken::default(),
        )
        .await
        .expect("publish");
    let table = Arc::new(
        RelationTable::new(snapshot, spec.id, &registry, reserver).expect("admitted table"),
    );
    (Arc::clone(&table), table.relation().batch().clone())
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
    ) -> pse_catalog::BoxFut<
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
    let (table, batch) = fixture().await;
    let filters = [
        col("id").eq(lit(1_i64)),
        col("id").in_list(vec![lit(1_i64), lit(2_i64)], false),
        col("parent").is_null(),
        col("parent").is_not_null(),
        col("choice").eq(lit("one")),
    ];
    assert!(
        table
            .supports_filters_pushdown(&filters.iter().collect::<Vec<_>>())
            .expect("pure support")
            .iter()
            .all(|support| *support == TableProviderFilterPushDown::Exact)
    );
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
    let (_table, batch) = fixture().await;
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
