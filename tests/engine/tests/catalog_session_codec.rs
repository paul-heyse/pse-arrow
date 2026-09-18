// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Diagnostic protobuf round trips resolve actual providers and preserve metadata.

#[path = "../../support/native_publication.rs"]
mod native_publication;

use std::collections::BTreeMap;
use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder};
use datafusion_proto::{
    logical_plan::AsLogicalPlan,
    protobuf::{LogicalPlanNode, logical_plan_node::LogicalPlanType},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId};
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, Cell, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl,
        SnapshotClass,
    },
};

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
async fn fixture(value: &str) -> (SnapshotSession, Arc<FixedBudget>, tempfile::TempDir) {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    pse_schema::catalog::declare_publications(&mut builder);
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
    let batch = pse_relations::cells::batch_from_cells(
        &reg,
        spec,
        &[vec![
            Cell::Id(SemanticId::from_bytes([1; 16])),
            Cell::Enum("one"),
            Cell::text(value),
        ]],
    )
    .expect("batch");
    let budget = FixedBudget::new(64 << 20);
    let reserver: Arc<dyn MemoryReserver> = budget.clone();
    let key = spec.key;
    let (publication, directory, _) =
        native_publication::publish(reg, BTreeMap::from([(key, batch)]), reserver).await;
    (publication.into_session(), budget, directory)
}
#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn scan(session: &SnapshotSession) -> LogicalPlan {
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
#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn encode_proto(proto: &LogicalPlanNode) -> Vec<u8> {
    let mut bytes = Vec::new();
    proto.try_encode(&mut bytes).expect("protobuf");
    bytes
}

#[tokio::test]
async fn diagnostic_roundtrip_preserves_actual_rows_extensions_and_owned_lifetimes() {
    let (session, budget, _directory) = fixture("λ").await;
    let cancel = CancellationToken::default();
    let plan = scan(&session);
    let bytes = session.encode_plan(&plan, &cancel).expect("encode");
    let decoded = session.decode_plan(&bytes, &cancel).expect("decode");
    assert_eq!(plan.schema(), decoded.plan().schema());
    let expected = session.execute_plan(plan, &cancel).await.expect("original");
    let actual = session
        .execute_decoded(&decoded, &cancel)
        .await
        .expect("decoded");
    assert_eq!(actual, expected);
    assert_eq!(actual.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    drop(actual);
    drop(expected);
    drop(decoded);
    drop(session);
    assert!(budget.reserved() > 0, "encoded evidence owns its bytes");
    drop(bytes);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn changed_member_schema_and_codec_are_refused_even_when_protobuf_is_well_formed() {
    let (session, _budget, _directory) = fixture("a").await;
    let cancel = CancellationToken::default();
    let bytes = session
        .encode_plan(&scan(&session), &cancel)
        .expect("encode");
    let mut proto = LogicalPlanNode::try_decode(&bytes).expect("protobuf");
    let Some(LogicalPlanType::CustomScan(scan)) = &mut proto.logical_plan_type else {
        panic!("custom provider");
    };
    scan.schema.as_mut().expect("schema").columns[0]
        .metadata
        .clear();
    assert!(session.decode_plan(&encode_proto(&proto), &cancel).is_err());

    let mut proto = LogicalPlanNode::try_decode(&bytes).expect("protobuf");
    let Some(LogicalPlanType::CustomScan(scan)) = &mut proto.logical_plan_type else {
        panic!("custom provider");
    };
    let mut binding: serde_json::Value =
        serde_json::from_slice(&scan.custom_table_data).expect("binding");
    binding["codec_version"] = "pse.plan.unknown".into();
    scan.custom_table_data = serde_json::to_vec(&binding).expect("binding bytes");
    assert!(session.decode_plan(&encode_proto(&proto), &cancel).is_err());

    let (different, _other_budget, _other_directory) = fixture("different actual rows").await;
    assert!(different.decode_plan(&bytes, &cancel).is_err());
    let decoded = session
        .decode_plan(&bytes, &cancel)
        .expect("retained provider");
    assert!(different.execute_decoded(&decoded, &cancel).await.is_err());
    assert!(session.decode_plan(&[0xff; 32], &cancel).is_err());
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(session.decode_plan(&bytes, &cancelled).is_err());
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
    assert!(session.encode_plan(&plan, &cancel).is_err());
    let result = session
        .sql(
            "SELECT abs(-3) AS positive, count(*) AS n FROM artifact.authored.items",
            &cancel,
        )
        .await
        .expect("actual registered scalar and aggregate implementations");
    assert_eq!(result.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
}
