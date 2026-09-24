// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A raw cold Delta writer enforces selected-arm shape using native CHECK expressions.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "independent contract assertions"
)]
use pse_testkit::execution as native_execution;

use datafusion::{
    arrow::{
        array::{ArrayRef, BooleanArray, Int64Array, RecordBatch, StringArray, StructArray},
        buffer::NullBuffer,
        datatypes::{DataType, SchemaRef},
    },
    execution::{context::SessionContext, session_state::SessionStateBuilder},
};
use deltalake::{
    DeltaTableBuilder,
    delta_datafusion::SessionFallbackPolicy,
    kernel::{engine::arrow_conversion::TryIntoArrow, transaction::CommitProperties},
    protocol::SaveMode,
};
use pse_catalog::delta::{contract::DeclaredCheck, write::DeltaWrite};
use pse_engine::session::planner::UnifiedPlanner;
use pse_schema::{RegistryBuilder, model::*};
use std::sync::Arc;

fn contract() -> (Arc<pse_schema::Registry>, DeclaredCheck) {
    let declaration = TaggedAlternative::new(
        "kind",
        [
            ("flag".into(), "flag".into()),
            ("flag_alias".into(), "flag".into()),
            ("count".into(), "count".into()),
        ],
    )
    .with_unit("absent");
    let value = FieldContract::structure(vec![
        FieldContract::native(DataType::Utf8).with_name("kind"),
        FieldContract::structure(vec![
            FieldContract::native(DataType::Boolean).with_name("value"),
        ])
        .with_name("flag")
        .optional(),
        FieldContract::structure(vec![FieldContract::nonnegative(9).with_name("value")])
            .with_name("count")
            .optional(),
    ])
    .with_alternative(&declaration);
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "choices",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "choices",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::nonnegative(i64::MAX), "identity"),
            FieldContract::payload("value", value, "selected value").optional(),
        ]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let check =
        DeclaredCheck::new(&registry, registry.relation("authored.choices").unwrap().id).unwrap();
    (registry, check)
}

fn batch(
    schema: SchemaRef,
    tag: &str,
    visible: bool,
    flag: bool,
    count: Option<i64>,
) -> RecordBatch {
    let DataType::Struct(fields) = schema.field(1).data_type() else {
        panic!("outer struct")
    };
    let DataType::Struct(flag_fields) = fields[1].data_type() else {
        panic!("flag arm")
    };
    let DataType::Struct(count_fields) = fields[2].data_type() else {
        panic!("count arm")
    };
    let flag: ArrayRef = Arc::new(StructArray::new(
        flag_fields.clone(),
        vec![Arc::new(BooleanArray::from(vec![true]))],
        Some(NullBuffer::from(vec![flag])),
    ));
    let count: ArrayRef = Arc::new(StructArray::new(
        count_fields.clone(),
        vec![Arc::new(Int64Array::from(vec![count.unwrap_or(-1)]))],
        Some(NullBuffer::from(vec![count.is_some()])),
    ));
    let values: ArrayRef = Arc::new(StructArray::new(
        fields.clone(),
        vec![Arc::new(StringArray::from(vec![tag])), flag, count],
        Some(NullBuffer::from(vec![visible])),
    ));
    RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1])), values]).unwrap()
}

#[tokio::test]
async fn cold_native_checks_enforce_alternatives_and_parent_masks() {
    let (registry, contract) = contract();
    assert!(!contract.properties()["delta.constraints.pse_contract"].contains("pse_field_"));
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    let context = native_context();
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .build()
        .unwrap();
    let input = context
        .read_batch(batch(
            Arc::clone(contract.layout().execution_schema()),
            "flag",
            true,
            true,
            None,
        ))
        .unwrap()
        .into_unoptimized_plan();
    let write = DeltaWrite::declared(
        table,
        input,
        SaveMode::ErrorIfExists,
        CommitProperties::default(),
        contract.clone(),
    )
    .unwrap();
    let state = context.state();
    native_execution::run(&state, Arc::clone(&registry), &write)
        .await
        .unwrap();
    let cold = Arc::new(cold_state());
    for (tag, visible, flag, count, valid) in [
        ("unknown", true, true, None, false),
        ("flag", true, false, None, false),
        ("flag", true, true, Some(0), false),
        ("count", true, false, Some(-1), false),
        ("count", true, false, Some(10), false),
        ("count", true, false, Some(9), true),
        ("flag_alias", true, true, None, true),
        ("flag_alias", true, false, None, false),
        ("absent", true, false, None, true),
        ("absent", true, true, None, false),
        ("absent", true, false, Some(0), false),
        ("unknown", false, true, Some(-1), true),
    ] {
        let table = DeltaTableBuilder::from_url(location.clone())
            .unwrap()
            .load()
            .await
            .unwrap();
        let previous = table.version();
        let reopened = DeclaredCheck::open(&table, &cold).unwrap();
        assert_eq!(
            reopened.layout().execution_schema(),
            contract.layout().execution_schema()
        );
        let storage = Arc::new(
            table
                .snapshot()
                .unwrap()
                .schema()
                .as_ref()
                .try_into_arrow()
                .unwrap(),
        );
        let result = table
            .write(vec![batch(storage, tag, visible, flag, count)])
            .with_save_mode(SaveMode::Append)
            .with_session_state(cold.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .await;
        assert_eq!(
            result.is_ok(),
            valid,
            "{tag}/{visible}/{flag}/{count:?}: {result:?}"
        );
        if !valid {
            assert_eq!(
                DeltaTableBuilder::from_url(location.clone())
                    .unwrap()
                    .load()
                    .await
                    .unwrap()
                    .version(),
                previous
            );
        }
    }
}

#[tokio::test]
async fn cold_delta_collection_checks_reject_duplicates_without_losing_empty_or_null() {
    let (registry, contract) = collection_contract();
    let root = tempfile::tempdir().unwrap();
    let uri = url::Url::from_directory_path(root.path()).unwrap();
    let context = native_context();
    let input = context
        .read_batch(collection_batch(
            Arc::clone(contract.layout().execution_schema()),
            Some(&[1, 2]),
        ))
        .unwrap()
        .into_unoptimized_plan();
    let write = DeltaWrite::declared(
        DeltaTableBuilder::from_url(uri.clone())
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
    native_execution::run(&state, Arc::clone(&registry), &write)
        .await
        .unwrap();
    let cold = cold_state();
    for (values, valid) in [
        (Some(vec![]), true),
        (None, true),
        (Some(vec![1, 1]), false),
        (Some(vec![1, 2, 3]), false),
        (Some(vec![-1]), false),
        (Some(vec![2, 1]), true),
    ] {
        let table = DeltaTableBuilder::from_url(uri.clone())
            .unwrap()
            .load()
            .await
            .unwrap();
        let version = table.version();
        let contract = DeclaredCheck::open(&table, &cold).unwrap();
        let state = Arc::new(contract.bind(&cold).unwrap());
        let storage = Arc::new(
            table
                .snapshot()
                .unwrap()
                .schema()
                .as_ref()
                .try_into_arrow()
                .unwrap(),
        );
        let result = table
            .write(vec![collection_batch(storage, values.as_deref())])
            .with_save_mode(SaveMode::Append)
            .with_session_state(state)
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .await;
        assert_eq!(result.is_ok(), valid, "{values:?}: {result:?}");
        if !valid {
            assert_eq!(
                DeltaTableBuilder::from_url(uri.clone())
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

fn collection_contract() -> (Arc<pse_schema::Registry>, DeclaredCheck) {
    let mut registry = RegistryBuilder::new();
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "sets",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "bounded sets",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::nonnegative(i64::MAX), "identity"),
            FieldContract::payload(
                "values",
                FieldContract::list(FieldContract::nonnegative(9)).with_collection(
                    CollectionContract {
                        maximum: Some(2),
                        ..CollectionContract::SET
                    },
                ),
                "members",
            )
            .optional(),
        ]),
    );
    let registry = Arc::new(registry.build().unwrap());
    let check =
        DeclaredCheck::new(&registry, registry.relation("authored.sets").unwrap().id).unwrap();
    (registry, check)
}

fn collection_batch(schema: SchemaRef, values: Option<&[i64]>) -> RecordBatch {
    use datafusion::arrow::array::builder::{Int64Builder, ListBuilder};
    let DataType::List(child) = schema.field(1).data_type() else {
        panic!("list")
    };
    let mut list = ListBuilder::new(Int64Builder::new()).with_field(Arc::clone(child));
    if let Some(values) = values {
        list.values().append_slice(values);
    }
    list.append(values.is_some());
    RecordBatch::try_new(
        schema,
        vec![Arc::new(Int64Array::from(vec![1])), Arc::new(list.finish())],
    )
    .unwrap()
}

fn native_context() -> SessionContext {
    SessionContext::new_with_state(
        SessionStateBuilder::new_with_default_features()
            .with_query_planner(Arc::new(UnifiedPlanner::new(
                pse_catalog::assembly::planners(),
            )))
            .build(),
    )
}
fn cold_state() -> datafusion::execution::session_state::SessionState {
    SessionStateBuilder::new_with_default_features()
        .with_query_planner(deltalake::delta_datafusion::planner::DeltaPlanner::new())
        .build()
}
