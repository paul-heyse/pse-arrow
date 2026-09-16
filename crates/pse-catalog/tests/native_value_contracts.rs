// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Delta contracts, with a native-expression adapter for collection constraints.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "independent native contract assertions"
)]

use datafusion::{
    arrow::{
        array::{ArrayRef, Int64Array, ListArray, RecordBatch, StringArray, StructArray},
        buffer::{NullBuffer, OffsetBuffer},
        datatypes::{DataType, SchemaRef},
    },
    common::{DFSchema, config::Dialect},
    execution::{
        context::{SessionConfig, SessionContext},
        session_state::SessionStateBuilder,
    },
    physical_plan::collect,
};
use deltalake::{
    DeltaTableBuilder,
    delta_datafusion::SessionFallbackPolicy,
    kernel::{engine::arrow_conversion::TryIntoArrow, transaction::CommitProperties},
    protocol::SaveMode,
};
use pse_catalog::{
    delta::{contract::DeclaredCheck, write::DeltaWrite},
    session::planner::UnifiedPlanner,
};
use pse_schema::{Registry, RegistryBuilder, model::*};
use std::sync::Arc;

#[test]
fn every_registry_contract_is_native_sql_against_its_durable_schema() {
    let registry = pse_schema::shared_registry().unwrap();
    let state = SessionContext::new().state();
    let mut config = SessionConfig::new();
    config.options_mut().sql_parser.dialect = Dialect::DuckDB;
    let expressions = SessionContext::new_with_config(config).state();
    for spec in registry.relations() {
        let contract = DeclaredCheck::new(&registry, spec.id).unwrap();
        let sql = &contract.properties()["delta.constraints.pse_contract"];
        assert!(!sql.contains("pse_check_"), "{}", spec.key);
        let schema =
            DFSchema::try_from(contract.layout().storage_schema().as_ref().clone()).unwrap();
        let bound = contract.bind(&state).unwrap();
        bound
            .create_logical_expr(sql, &schema)
            .unwrap_or_else(|error| panic!("{}: {error}; {sql}", spec.key));
        for (key, sql) in contract.properties() {
            if key.starts_with("pse.check.nested.expression.") {
                let predicate = expressions
                    .create_logical_expr(sql, &schema)
                    .unwrap_or_else(|error| panic!("{}: {error}; {sql}", spec.key));
                expressions
                    .create_physical_expr(predicate, &schema)
                    .unwrap_or_else(|error| panic!("{}: {error}; {sql}", spec.key));
            }
        }
        assert_eq!(
            contract.properties()[pse_schema::arrow::KEY_CONTRACT_FINGERPRINT],
            spec.fingerprint.to_hex()
        );
    }
}

fn fixture() -> (Arc<Registry>, DeclaredCheck) {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![
            EnumMember::new("one", "first"),
            EnumMember::new("two", "second"),
        ],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "native_values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Nested domains",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::nonnegative(i64::MAX), "identity"),
            FieldContract::payload(
                // A user field cannot collide with the stored parsing-dialect property.
                "dialect",
                FieldContract::list(FieldContract::structure(vec![
                    FieldContract::enumeration("Choice").with_name("kind"),
                    FieldContract::nonnegative(255).with_name("offset"),
                ])),
                "ordered members",
            )
            .optional(),
        ]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let contract = DeclaredCheck::new(
        &registry,
        registry.relation("authored.native_values").unwrap().id,
    )
    .unwrap();
    (registry, contract)
}

fn batch(schema: SchemaRef, kind: &str, offset: i64, visible: bool) -> RecordBatch {
    let DataType::List(child) = schema.field(1).data_type() else {
        panic!("list")
    };
    let DataType::Struct(fields) = child.data_type() else {
        panic!("struct")
    };
    let values: ArrayRef = Arc::new(StructArray::new(
        fields.clone(),
        vec![
            Arc::new(StringArray::from(vec![kind])),
            Arc::new(Int64Array::from(vec![offset])),
        ],
        None,
    ));
    let members: ArrayRef = Arc::new(ListArray::new(
        Arc::clone(child),
        OffsetBuffer::new(vec![0_i32, 1].into()),
        values,
        Some(NullBuffer::from(vec![visible])),
    ));
    RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1])), members]).unwrap()
}

async fn seed(contract: &DeclaredCheck, location: &url::Url) {
    let state = SessionStateBuilder::new()
        .with_default_features()
        .with_query_planner(Arc::new(UnifiedPlanner::default()))
        .build();
    let context = SessionContext::new_with_state(state);
    let input = context
        .read_batch(batch(
            Arc::clone(contract.layout().execution_schema()),
            "one",
            255,
            true,
        ))
        .unwrap()
        .into_unoptimized_plan();
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .build()
        .unwrap();
    let write = DeltaWrite::declared(
        table,
        input,
        SaveMode::ErrorIfExists,
        CommitProperties::default(),
        contract.clone(),
    )
    .unwrap();
    let state = context.state();
    collect(
        state.create_physical_plan(&write).await.unwrap(),
        state.task_ctx(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn nested_native_checks_guard_first_write_and_cold_raw_delta_mutations() {
    let (_registry, contract) = fixture();
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    seed(&contract, &location).await;
    // A fresh session needs only the declared native-expression adapter; neither
    // a PSE query planner nor the deleted Cell validator is involved.
    let cold = SessionContext::new_with_state(
        SessionStateBuilder::new()
            .with_default_features()
            .with_query_planner(deltalake::delta_datafusion::planner::DeltaPlanner::new())
            .build(),
    );
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    // Raw Delta input uses the schema persisted in the log, whose relation-level
    // metadata lives in table properties rather than Arrow schema metadata.
    let stored = Arc::new(
        table
            .snapshot()
            .unwrap()
            .schema()
            .as_ref()
            .try_into_arrow()
            .unwrap(),
    );
    let reopened = DeclaredCheck::open(&table, &cold.state()).unwrap();
    assert_eq!(reopened.properties(), contract.properties());
    assert_eq!(
        reopened.layout().execution_schema(),
        contract.layout().execution_schema()
    );
    let unbound = table
        .write(vec![batch(Arc::clone(&stored), "one", 0, true)])
        .with_save_mode(SaveMode::Append)
        .with_session_state(Arc::new(cold.state()))
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .await;
    assert!(
        unbound.is_err(),
        "missing collection-expression capability must fail closed"
    );
    for (kind, offset, visible, valid) in [
        ("missing", 2, true, false),
        ("one", -1, true, false),
        ("two", 256, true, false),
        ("missing", -1, false, true),
        ("two", 0, true, true),
    ] {
        let table = DeltaTableBuilder::from_url(location.clone())
            .unwrap()
            .load()
            .await
            .unwrap();
        contract.verify(&table).unwrap();
        let version = table.version();
        let candidate = batch(Arc::clone(&stored), kind, offset, visible);
        let result = table
            .write(vec![candidate])
            .with_save_mode(SaveMode::Append)
            .with_session_state(Arc::new(reopened.bind(&cold.state()).unwrap()))
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .await;
        assert_eq!(
            result.is_ok(),
            valid,
            "{kind}/{offset}/{visible}: {result:?}"
        );
        if !valid {
            assert_eq!(
                DeltaTableBuilder::from_url(location.clone())
                    .unwrap()
                    .load()
                    .await
                    .unwrap()
                    .version(),
                version
            );
        }
    }
}
