// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL declarations govern candidate admission and first/cold Delta writes.
#![allow(
    clippy::unwrap_used,
    reason = "independent native row-check assertions"
)]

#[path = "support/native_execution.rs"]
mod native_execution;

use datafusion::{
    arrow::{
        array::{BooleanArray, Int64Array, ListArray, RecordBatch},
        datatypes::{Int64Type, SchemaRef},
    },
    common::TableReference,
    execution::{context::SessionContext, session_state::SessionStateBuilder},
    physical_plan::collect,
};
use deltalake::{
    DeltaTableBuilder,
    delta_datafusion::SessionFallbackPolicy,
    kernel::{engine::arrow_conversion::TryIntoArrow, transaction::CommitProperties},
    protocol::SaveMode,
};
use pse_catalog::{
    delta::{admission::violation_plans, contract::DeclaredCheck, write::DeltaWrite},
    session::planner::UnifiedPlanner,
};
use pse_ids::SemanticId;
use pse_relations::generated::{enums::PublicationKind, runtime::publications};
use pse_schema::{Registry, RegistryBuilder, model::*};
use std::{collections::BTreeMap, sync::Arc};

#[path = "support/source_syntax.rs"]
mod source_syntax;

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "pairs",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Native collection relationships.",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::nonnegative(i64::MAX), "identity"),
            FieldContract::list(FieldContract::nonnegative(i64::MAX)).with_name("left_items"),
            FieldContract::list(FieldContract::nonnegative(i64::MAX)).with_name("right_items"),
            FieldContract::native(datafusion::arrow::datatypes::DataType::Boolean)
                .with_name("approved")
                .optional(),
        ])
        .checks(BTreeMap::from([
            (
                "same_length".into(),
                "array_length(left_items) = array_length(right_items)".into(),
            ),
            ("approval".into(), "approved".into()),
        ])),
    );
    Arc::new(builder.build().unwrap())
}
fn context() -> SessionContext {
    SessionContext::new_with_state(
        SessionStateBuilder::new()
            .with_default_features()
            .with_query_planner(Arc::new(UnifiedPlanner::default()))
            .build(),
    )
}
fn batch(schema: &SchemaRef, left: &[i64], right: &[i64], approved: Option<bool>) -> RecordBatch {
    let list = |index: usize, values: &[i64]| {
        let array = ListArray::from_iter_primitive::<Int64Type, _, _>([Some(
            values.iter().copied().map(Some).collect::<Vec<_>>(),
        )]);
        datafusion::arrow::compute::cast(&array, schema.field(index).data_type()).unwrap()
    };
    RecordBatch::try_new(
        Arc::clone(schema),
        vec![
            Arc::new(Int64Array::from(vec![1])),
            list(1, left),
            list(2, right),
            Arc::new(BooleanArray::from(vec![approved])),
        ],
    )
    .unwrap()
}
fn candidate(registry: &Registry) -> publications::Row {
    let spec = registry.relation("authored.pairs").unwrap();
    publications::Row {
        workspace_id: SemanticId::NIL,
        publication_id: SemanticId::NIL,
        parent_publication_id: None,
        attempt_id: SemanticId::NIL,
        kind: PublicationKind::Relations,
        inputs: vec![],
        members: vec![publications::RuntimePublicationsFieldMembersItem {
            catalog_name: "datafusion".into(),
            schema_name: "public".into(),
            table_name: "pairs".into(),
            relation_id: spec.id,
            relation_version: 1,
            contract_fingerprint: spec.fingerprint,
            table_uri: "memory://pairs".into(),
            delta_version: 1,
            selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
        }],
    }
}

#[tokio::test]
async fn native_candidate_checks_enforce_relationships_and_null_refusal() {
    let registry = registry();
    let schema = Arc::new(
        pse_schema::arrow::relation_schema(&registry, registry.relation("authored.pairs").unwrap())
            .unwrap(),
    );
    let context = context();
    for (left, right, approved, valid) in [
        (vec![1, 2], vec![3, 4], Some(true), true),
        (vec![], vec![], Some(true), true),
        (vec![1], vec![], Some(true), false),
        (vec![], vec![], Some(false), false),
        (vec![], vec![], None, false),
    ] {
        let reference = TableReference::full("datafusion", "public", "pairs");
        context.deregister_table(reference.clone()).unwrap();
        context
            .register_batch(reference, batch(&schema, &left, &right, approved))
            .unwrap();
        let state = context.state();
        let checks = violation_plans(&candidate(&registry), Arc::clone(&registry), &state)
            .await
            .unwrap();
        let mut violations = 0;
        for check in checks {
            violations += collect(
                state.create_physical_plan(&check).await.unwrap(),
                state.task_ctx(),
            )
            .await
            .unwrap()
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>();
        }
        assert_eq!(violations == 0, valid);
    }
}

#[tokio::test]
async fn first_and_cold_delta_writes_execute_declared_native_sql() {
    let registry = registry();
    let contract =
        DeclaredCheck::new(&registry, registry.relation("authored.pairs").unwrap().id).unwrap();
    let root = tempfile::tempdir().unwrap();
    let uri = url::Url::from_directory_path(root.path()).unwrap();
    let context = context();
    for (right, valid) in [vec![], vec![2]].into_iter().zip([false, true]) {
        let input = context
            .read_batch(batch(
                contract.layout().execution_schema(),
                &[1],
                &right,
                Some(true),
            ))
            .unwrap()
            .into_unoptimized_plan();
        let table = DeltaTableBuilder::from_url(uri.clone()).unwrap();
        let table = if valid {
            table.load().await.unwrap()
        } else {
            table.build().unwrap()
        };
        let write = DeltaWrite::declared(
            table,
            input,
            SaveMode::Append,
            CommitProperties::default(),
            contract.clone(),
        )
        .unwrap();
        let state = context.state();
        let result = native_execution::run(&state, Arc::clone(&registry), &write).await;
        assert_eq!(result.is_ok(), valid, "first write: {result:?}");
        if let Err(error) = result {
            assert!(
                error.to_string().contains("failed validation check"),
                "{error}"
            );
        }
    }
    let cold = self::context().state();
    drop(context);
    drop(contract);
    drop(registry);
    for (right, approval, valid) in [
        (vec![], Some(true), false),
        (vec![2], None, false),
        (vec![2], Some(true), true),
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
            .write(vec![batch(&schema, &[1], &right, approval)])
            .with_save_mode(SaveMode::Append)
            .with_session_state(state)
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .await;
        assert_eq!(result.is_ok(), valid, "cold write: {result:?}");
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
    assert_complete_constraint_set(uri, &cold).await;
}

async fn assert_complete_constraint_set(
    uri: url::Url,
    cold: &datafusion::execution::session_state::SessionState,
) {
    let changed = DeltaTableBuilder::from_url(uri.clone())
        .unwrap()
        .load()
        .await
        .unwrap()
        .add_constraint()
        .with_constraint("undeclared", "approved IS NOT NULL")
        .with_session_state(Arc::new(cold.clone()))
        .await
        .unwrap();
    assert!(
        DeclaredCheck::open(&changed, cold).is_err(),
        "extra constraints change the admitted contract"
    );
    changed
        .drop_constraints()
        .with_constraint("undeclared")
        .await
        .unwrap();
    let changed = DeltaTableBuilder::from_url(uri)
        .unwrap()
        .load()
        .await
        .unwrap()
        .drop_constraints()
        .with_constraint("pse_declared_same_length")
        .await
        .unwrap();
    assert!(
        DeclaredCheck::open(&changed, cold).is_err(),
        "cold reconstruction requires the recorded CHECK"
    );
}

#[test]
fn implicit_system_cardinality_is_a_declared_native_check() {
    let registry = pse_schema::registry().unwrap();
    for name in [
        "compiled.math_implicit_systems",
        "inferred.math_implicit_systems",
    ] {
        let spec = registry.relation(name).unwrap();
        let contract = DeclaredCheck::new(registry, spec.id).unwrap();
        assert_eq!(
            contract.properties()["delta.constraints.pse_declared_implicit_cardinality"],
            "(array_length(unknown_symbol_ids) = array_length(equation_ids)) IS TRUE"
        );
        contract.bind(&context().state()).unwrap();
    }
}
