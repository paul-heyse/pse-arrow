// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Match the native engine's nested async ownership proof.
#![recursion_limit = "256"]

//! Native engine reads preserve actual providers, metadata and owned buffers.

#[path = "../../support/native_session.rs"]
mod native_session;

use std::collections::BTreeMap;
use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_columnar::CancellationToken;
use pse_engine::session::EngineSession;
use pse_ids::SemanticId;
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl, SnapshotClass,
    },
};

async fn fixture(
    value: &str,
) -> (
    EngineSession,
    Arc<dyn pse_columnar::MemoryPool>,
    tempfile::TempDir,
) {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new("one", "One")],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "items",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::id(), "Key"),
            FieldContract::payload("choice", FieldContract::enumeration("Choice"), "Choice"),
            FieldContract::payload(
                "label",
                FieldContract::native(arrow::datatypes::DataType::Utf8),
                "Value",
            )
            .optional(),
        ]),
    );
    let reg = Arc::new(builder.build().expect("registry"));
    let spec = reg.relation("authored.items").expect("relation");
    let validation =
        pse_relations::validate::ValidationContext::local(&reg).expect("local codec fixture");
    let batch = pse_relations::testing::batch_from_literals(
        &reg,
        spec,
        &[vec![
            serde_json::json!(["id", (SemanticId::from_bytes([1; 16])).to_hex()]),
            serde_json::json!(["enum", "one"]),
            serde_json::json!(["text", value]),
        ]],
        &validation,
    )
    .expect("batch");
    let key = spec.key;
    let (publication, directory, budget) =
        native_session::candidate(reg, BTreeMap::from([(key, batch)]));
    (publication, budget, directory)
}
fn scan(session: &EngineSession) -> LogicalPlan {
    let spec = session
        .registry()
        .relation("authored.items")
        .expect("relation");
    // Keep the exact selected-provider descriptor. LogicalPlanBuilder::scan
    // deliberately expands native ViewTables at this DataFusion pin.
    LogicalPlan::TableScan(
        datafusion::logical_expr::TableScanBuilder::new(
            session.table_reference(&spec.key).expect("name"),
            session.table_source(&spec.key).expect("source"),
        )
        .build()
        .expect("scan"),
    )
}
#[tokio::test]
async fn native_reads_preserve_actual_rows_extensions_and_owned_lifetimes() {
    let (session, budget, _directory) = fixture("λ").await;
    let cancel = CancellationToken::default();
    let plan = scan(&session);
    let expected = session.execute_plan(plan.clone(), &cancel).await.expect("original");
    let actual = session.execute_plan(plan, &cancel).await.expect("same actual selected owner");
    assert_eq!(actual, expected);
    assert_eq!(actual.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert_eq!(actual[0].column(2).as_any().downcast_ref::<arrow::array::StringArray>().unwrap().value(0), "λ");
    drop(expected);
    drop(session);
    assert!(budget.reserved() > 0, "returned Arrow buffers retain their real allocation owner");
    drop(actual);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn receiving_inventory_and_cancellation_refuse_an_unadmitted_plan() {
    let (session, _budget, _directory) = fixture("a").await;
    let cancel = CancellationToken::default();
    let plan=scan(&session);
    let (different, _other_budget, _other_directory) = fixture("different actual rows").await;
    assert!(different.execute_plan(plan.clone(), &cancel).await.is_err());
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(session.execute_plan(plan, &cancelled).await.is_err());
}

#[tokio::test]
async fn matching_builtin_names_cannot_substitute_foreign_function_implementations() {
    use datafusion::arrow::datatypes::DataType;
    use datafusion::logical_expr::{Volatility, create_udf, lit};
    let (session, _budget, _directory) = fixture("a").await;
    let cancel = CancellationToken::default();
    let foreign = create_udf(
        "abs",
        vec![DataType::Int64],
        DataType::Int64,
        Volatility::Immutable,
        Arc::new(|args| Ok(args[0].clone())),
    );
    let plan = LogicalPlanBuilder::empty(true)
        .project(vec![foreign.call(vec![lit(-3i64)])])
        .expect("foreign typed expression")
        .build()
        .expect("plan");
    let values = session
        .execute_plan(plan.clone(), &cancel)
        .await
        .expect("actual native function implementation is executable");
    assert_eq!(
        values[0]
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::Int64Array>()
            .unwrap()
            .value(0),
        -3,
        "the foreign function executes its own implementation, never the registered abs"
    );
    let result = session
        .sql(
            "SELECT abs(-3) AS positive, count(*) AS n FROM model.authored.items",
            &cancel,
        )
        .await
        .expect("actual registered scalar and aggregate implementations");
    assert_eq!(result.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert_eq!(result[0].column(0).as_any().downcast_ref::<arrow::array::Int64Array>().unwrap().value(0), 3);
    assert_eq!(result[0].column(1).as_any().downcast_ref::<arrow::array::Int64Array>().unwrap().value(0), 1);
}
