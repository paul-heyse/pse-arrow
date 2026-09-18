// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Small native-memory write units, with no compiler or durable storage journey.
use super::*;
use crate::session::{SessionFactory, mutation::MemoryTableFactory};
use datafusion::{
    arrow::{
        array::Int64Array,
        datatypes::{DataType, Field, Schema},
    },
    common::ScalarValue,
    datasource::MemTable,
    execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
    logical_expr::{LogicalPlanBuilder, lit},
};
use pse_ids::FixedBudget;
use pse_schema::model::provider::OperationPurpose;
use std::collections::BTreeMap;

fn session(cancel: &CancellationToken) -> (SnapshotSession, TableReference) {
    let factory = SessionFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(16 << 20),
        "insert-unit",
        SessionStateBuilder::new_with_default_features(),
    );
    let mut session = factory
        .candidate_checked(
            BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            cancel,
        )
        .unwrap()
        .with_purpose(OperationPurpose::Mutate);
    let schema = Arc::new(Schema::new(vec![
        Field::new("id \"quoted\"", DataType::Int64, false),
        Field::new("computed default", DataType::Int64, true),
    ]));
    let provider = Arc::new(
        MemTable::try_new(schema, vec![vec![]])
            .unwrap()
            .with_column_defaults(std::collections::HashMap::from([(
                "computed default".into(),
                lit(40_i64) + lit(2_i64),
            )])),
    );
    let name = TableReference::full("native.catalog", "private schema", "target \"quoted\"");
    let mut binding = TableBinding::new(name.clone(), provider, None, None);
    binding.mutation = Some(Arc::new(MemoryTableFactory));
    session
        .bindings
        .insert(BindingKey::Native(name.clone()), binding)
        .unwrap();
    (session, name)
}

fn input(explicit_null: bool) -> LogicalPlan {
    let mut fields = vec![lit(7_i32).alias("id \"quoted\"")];
    if explicit_null {
        fields.push(lit(ScalarValue::Int64(None)).alias("computed default"));
    }
    LogicalPlanBuilder::empty(true)
        .project(fields)
        .unwrap()
        .build()
        .unwrap()
}

#[tokio::test]
async fn rust_insert_uses_native_defaults_and_preserves_explicit_null() {
    let cancel = CancellationToken::new();
    let (session, target) = session(&cancel);
    for explicit_null in [false, true] {
        let prepared = session
            .prepare_insert(
                target.clone(),
                input(explicit_null),
                InsertOp::Append,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(prepared.computation_roles().count(), 0);
        let LogicalPlan::Dml(command) = prepared.original_plan() else {
            panic!("expected native DML");
        };
        let values = session
            .prepare(command.input.as_ref().clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap()
            .into_batches();
        let expected = if explicit_null {
            Int64Array::from(vec![None])
        } else {
            Int64Array::from(vec![42])
        };
        assert_eq!(values[0].column(1).as_ref(), &expected);
        let completed = prepared.execute(&cancel).await.unwrap();
        let updated = completed.resulting_session(&cancel).await.unwrap();
        assert_eq!(
            updated.bindings.iter().count(),
            session.bindings.iter().count()
        );
        let rows = updated
            .prepare_sql(
                &format!("SELECT * FROM {}", target.to_quoted_string()),
                &cancel,
            )
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap()
            .into_batches();
        assert_eq!(
            rows.iter()
                .map(pse_relations::RecordBatch::num_rows)
                .sum::<usize>(),
            1
        );
        assert_eq!(rows[0].column(1).as_ref(), &expected);
    }
    let source = session
        .prepare_sql(
            &format!("SELECT * FROM {}", target.to_quoted_string()),
            &cancel,
        )
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .into_batches();
    assert_eq!(
        source
            .iter()
            .map(pse_relations::RecordBatch::num_rows)
            .sum::<usize>(),
        0
    );
}

#[tokio::test]
async fn insert_keeps_policy_and_native_column_errors() {
    let cancel = CancellationToken::new();
    let (session, target) = session(&cancel);
    let unknown = LogicalPlanBuilder::empty(true)
        .project(vec![lit(1_i64).alias("unknown")])
        .unwrap()
        .build()
        .unwrap();
    assert!(
        session
            .prepare_insert(target.clone(), unknown, InsertOp::Append, &cancel)
            .await
            .is_err()
    );
    assert!(
        session
            .with_purpose(OperationPurpose::Query)
            .prepare_insert(target, input(false), InsertOp::Append, &cancel)
            .await
            .is_err()
    );
}
