// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cold Delta writers enforce correlated key presence without the original registry.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "independent cold-writer assertions"
)]

#[path = "../../../tests/support/catalog_context.rs"]
mod catalog_context;
use catalog_context::context;
use pse_testkit::execution as native_execution;

use datafusion::arrow::{
    array::{ArrayRef, Int64Array, RecordBatch, StructArray},
    datatypes::{DataType, SchemaRef},
};
use deltalake::{
    DeltaTableBuilder,
    delta_datafusion::SessionFallbackPolicy,
    kernel::{engine::arrow_conversion::TryIntoArrow, transaction::CommitProperties},
    protocol::SaveMode,
};
use pse_catalog::delta::{contract::DeclaredCheck, write::DeltaWrite};
use pse_schema::{Registry, RegistryBuilder, model::*};
use std::sync::Arc;

fn registry() -> Registry {
    let mut builder = RegistryBuilder::new();
    let relation = |name| {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "composite reference fixture",
        )
    };
    builder.declare_relation(
        relation("target").pk(&["tenant", "id"]).columns(
            ["tenant", "id"]
                .map(|name| FieldContract::native(DataType::Int64).with_name(name))
                .to_vec(),
        ),
    );
    let key = FieldContract::structure(
        ["tenant", "id"]
            .map(|name| {
                FieldContract::native(DataType::Int64)
                    .with_name(name)
                    .optional()
            })
            .to_vec(),
    )
    .with_name("target_key")
    .with_reference(&ReferenceContract {
        relation: "authored.target".into(),
        columns: ["tenant", "id"]
            .map(|name| ReferenceColumn {
                source: vec![name.into()],
                target: name.into(),
            })
            .to_vec(),
        null_policy: ReferenceNullPolicy::AllOrNone,
    })
    .unwrap();
    builder.declare_relation(relation("source").pk(&["id"]).columns(vec![
        FieldContract::native(DataType::Int64).with_name("id"),
        key,
    ]));
    builder.build().unwrap()
}

fn batch(schema: &SchemaRef, key: [Option<i64>; 2]) -> RecordBatch {
    let DataType::Struct(fields) = schema.field(1).data_type() else {
        panic!("native struct")
    };
    let arrays = key
        .map(|value| -> ArrayRef { Arc::new(Int64Array::from(vec![value])) })
        .to_vec();
    RecordBatch::try_new(
        Arc::clone(schema),
        vec![
            Arc::new(Int64Array::from(vec![1])),
            Arc::new(StructArray::new(fields.clone(), arrays, None)),
        ],
    )
    .unwrap()
}

#[tokio::test]
async fn cold_delta_check_rejects_partial_composite_keys() {
    let root = tempfile::tempdir().unwrap();
    let uri = url::Url::from_directory_path(root.path()).unwrap();
    {
        let registry = Arc::new(registry());
        let contract =
            DeclaredCheck::new(&registry, registry.relation("authored.source").unwrap().id)
                .unwrap();
        let context = context();
        let input = context
            .read_batch(batch(
                contract.layout().execution_schema(),
                [Some(1), Some(2)],
            ))
            .unwrap()
            .into_unoptimized_plan();
        let write = DeltaWrite::declared(
            DeltaTableBuilder::from_url(uri.clone())
                .unwrap()
                .build()
                .unwrap(),
            input,
            SaveMode::Append,
            CommitProperties::default(),
            contract,
        )
        .unwrap();
        let state = context.state();
        native_execution::run(&state, Arc::clone(&registry), &write)
            .await
            .unwrap();
    }
    let cold = context().state();
    for (key, valid) in [
        ([Some(1), None], false),
        ([None, Some(2)], false),
        ([None, None], true),
        ([Some(3), Some(4)], true),
    ] {
        let table = DeltaTableBuilder::from_url(uri.clone())
            .unwrap()
            .load()
            .await
            .unwrap();
        let version = table.version();
        let opened = DeclaredCheck::open(&table, &cold).unwrap();
        let state = Arc::new(opened.bind(&cold).unwrap());
        let schema = Arc::new(
            table
                .snapshot()
                .unwrap()
                .schema()
                .as_ref()
                .try_into_arrow()
                .unwrap(),
        );
        let result = table
            .write(vec![batch(&schema, key)])
            .with_save_mode(SaveMode::Append)
            .with_session_state(state)
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .await;
        assert_eq!(result.is_ok(), valid, "cold composite key: {result:?}");
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
