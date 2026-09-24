// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual SQL hooks over the pinned validating Delta builders.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration assertions"
)]
use pse_testkit::execution as native_execution;

use datafusion::{
    arrow::{
        array::{Array, RecordBatch, UInt64Array},
        datatypes::DataType,
    },
    common::Result,
    execution::{context::SessionContext, session_state::SessionStateBuilder},
    logical_expr::{Volatility, create_udf},
    prelude::SessionConfig,
};
use deltalake::{DeltaTable, DeltaTableBuilder, kernel::transaction::CommitProperties};
use pse_catalog::delta::dml::WritableTable;
use pse_engine::session::planner::UnifiedPlanner;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn context() -> (SessionContext, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let udf = create_udf(
        "sentinel",
        vec![DataType::Int64],
        DataType::Int64,
        Volatility::Volatile,
        Arc::new(move |args| {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(args[0].clone())
        }),
    );
    let state = SessionStateBuilder::new()
        .with_default_features()
        .with_config(SessionConfig::new().with_batch_size(13))
        .with_query_planner(Arc::new(UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )))
        .build();
    let context = SessionContext::new_with_state(state);
    context.register_udf(udf);
    (context, calls)
}
async fn load(location: &url::Url) -> DeltaTable {
    DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .load()
        .await
        .unwrap()
}
async fn seed(context: &SessionContext, location: &url::Url) {
    let input = context.sql("SELECT * FROM (VALUES (CAST(1 AS BIGINT), CAST(10 AS BIGINT)), (2, 20)) AS v(id, value)")
        .await.unwrap().into_unoptimized_plan();
    let table = DeltaTableBuilder::from_url(location.clone())
        .unwrap()
        .build()
        .unwrap()
        .write(Vec::<RecordBatch>::new())
        .with_input_plan(input)
        .with_session_state(Arc::new(context.state()))
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .await
        .unwrap();
    table
        .add_constraint()
        .with_constraint("positive", "id > 0 AND value >= 0")
        .with_session_state(Arc::new(context.state()))
        .await
        .unwrap();
}
async fn bind(context: &SessionContext, location: &url::Url) {
    let provider = WritableTable::new(
        load(location).await,
        Arc::new(context.state()),
        CommitProperties::default(),
    )
    .await
    .unwrap();
    context.deregister_table("edit").unwrap();
    context.register_table("edit", Arc::new(provider)).unwrap();
}
async fn sql_count(context: &SessionContext, location: &url::Url, sql: &str) -> Result<u64> {
    let state = context.state();
    let version = load(location).await.version();
    let logical = state.create_logical_plan(sql).await?;
    let plan = native_execution::prepare(&state, pse_schema::shared_registry().unwrap(), &logical)?;
    assert_eq!(
        load(location).await.version(),
        version,
        "planning must have no mutation"
    );
    let rows = plan
        .clone()
        .execute(&pse_columnar::CancellationToken::new())
        .await
        .map_err(datafusion::common::DataFusionError::from)?
        .into_batches();
    let result = rows[0]
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .unwrap();
    assert_eq!(result.len(), 1);
    assert!(
        plan.execute(&pse_columnar::CancellationToken::new())
            .await
            .is_err(),
        "prepared effect must execute once"
    );
    Ok(result.value(0))
}

#[tokio::test]
async fn sql_mutations_use_native_checks_children_and_caller_functions() {
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    let (context, calls) = context();
    seed(&context, &location).await;
    bind(&context, &location).await;
    let old_read = context.table("edit").await.unwrap();
    assert_eq!(
        sql_count(&context, &location, "INSERT INTO edit VALUES (3, 30)")
            .await
            .unwrap(),
        1
    );
    bind(&context, &location).await;
    assert_eq!(
        sql_count(
            &context,
            &location,
            "UPDATE edit SET value = sentinel(value + 1) WHERE id = 1"
        )
        .await
        .unwrap(),
        1
    );
    assert!(calls.load(Ordering::SeqCst) > 0);
    bind(&context, &location).await;
    assert_eq!(
        sql_count(&context, &location, "DELETE FROM edit WHERE id = 2")
            .await
            .unwrap(),
        1
    );
    bind(&context, &location).await;
    assert_eq!(sql_count(&context, &location,
        "MERGE INTO edit USING (SELECT * FROM (VALUES (1, 100), (4, 40)) AS v(id, value)) AS s ON edit.id = s.id \
         WHEN MATCHED THEN UPDATE SET value = s.value \
         WHEN NOT MATCHED BY TARGET THEN INSERT (id, value) VALUES (s.id, s.value)").await.unwrap(), 2);
    bind(&context, &location).await;
    assert_eq!(
        sql_count(&context, &location, "TRUNCATE TABLE edit")
            .await
            .unwrap(),
        3
    );
    let rows: usize = old_read
        .collect()
        .await
        .unwrap()
        .iter()
        .map(RecordBatch::num_rows)
        .sum();
    assert_eq!(
        rows, 2,
        "previous invocation remains bound to its actual Delta version"
    );
}

#[tokio::test]
async fn every_writing_sql_hook_enforces_delta_check_before_commit() {
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    let (context, _) = context();
    seed(&context, &location).await;
    let version = load(&location).await.version();
    for sql in [
        "INSERT INTO edit VALUES (-1, 1)",
        "INSERT OVERWRITE edit VALUES (3, -1)",
        "UPDATE edit SET value = -1 WHERE id = 1",
        "MERGE INTO edit USING (SELECT 1 AS id, -1 AS value) AS s ON edit.id = s.id WHEN MATCHED THEN UPDATE SET value = s.value",
        "MERGE INTO edit USING (SELECT -1 AS id, 1 AS value) AS s ON edit.id = s.id WHEN NOT MATCHED THEN INSERT (id, value) VALUES (s.id, s.value)",
    ] {
        bind(&context, &location).await;
        let result = sql_count(&context, &location, sql).await;
        assert!(result.is_err(), "{sql}: {result:?}");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("failed validation check"),
            "{sql}: {error}"
        );
        assert_eq!(
            load(&location).await.version(),
            version,
            "rejected mutation committed: {sql}"
        );
    }
}

#[tokio::test]
async fn merge_by_source_and_stale_mutation_are_native_delta_semantics() {
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    let (context, _) = context();
    seed(&context, &location).await;
    bind(&context, &location).await;
    let state = context.state();
    let prior_plan = native_execution::prepare(
        &state,
        pse_schema::shared_registry().unwrap(),
        &state
            .create_logical_plan("UPDATE edit SET value = 50 WHERE id = 1")
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        sql_count(
            &context,
            &location,
            "MERGE INTO edit USING (SELECT 1 AS id) AS s ON edit.id = s.id \
         WHEN MATCHED THEN UPDATE SET value = 11 \
         WHEN NOT MATCHED BY SOURCE THEN DELETE"
        )
        .await
        .unwrap(),
        2
    );
    assert!(
        prior_plan
            .execute(&pse_columnar::CancellationToken::new())
            .await
            .is_err()
    );
    bind(&context, &location).await;
    assert_eq!(
        sql_count(
            &context,
            &location,
            "INSERT OVERWRITE edit VALUES (8, 80), (9, 90)"
        )
        .await
        .unwrap(),
        2
    );
    bind(&context, &location).await;
    assert_eq!(
        sql_count(&context, &location, "DELETE FROM edit WHERE id = 100")
            .await
            .unwrap(),
        0
    );
}

async fn seed_declared(
    context: &SessionContext,
    location: &url::Url,
) -> pse_catalog::delta::contract::DeclaredCheck {
    use pse_catalog::delta::{contract::DeclaredCheck, write::DeltaWrite};
    use pse_relations::generated::{authored::entities, enums::EntityKind};
    let registry = pse_schema::shared_registry().unwrap();
    let check = DeclaredCheck::new(&registry, entities::RELATION_ID).unwrap();
    let mut builder = entities::Builder::new().unwrap();
    builder
        .push(entities::Row {
            entity_id: pse_ids::SemanticId::from_bytes([1; 16]),
            package_id: pse_ids::SemanticId::from_bytes([2; 16]),
            kind: EntityKind::Package,
            name: "sample".into(),
            qualified_name: "sample".into(),
            parent_entity_id: None,
            source_span: None,
        })
        .unwrap();
    let good = builder.finish().unwrap().into_batch();
    let mut columns = good.columns().to_vec();
    columns[2] = datafusion::arrow::compute::cast(
        &datafusion::arrow::array::StringArray::from(vec!["not_registered"]),
        good.schema().field(2).data_type(),
    )
    .unwrap();
    let bad = RecordBatch::try_new(good.schema(), columns).unwrap();
    for (name, batch) in [("bad", bad), ("good", good)] {
        let attempt_location = if name == "bad" {
            location.join("invalid/").unwrap()
        } else {
            location.clone()
        };
        std::fs::create_dir_all(attempt_location.to_file_path().unwrap()).unwrap();
        context.register_batch(name, batch).unwrap();
        let input = context.table(name).await.unwrap().into_unoptimized_plan();
        let table = DeltaTableBuilder::from_url(attempt_location.clone())
            .unwrap()
            .build()
            .unwrap();
        let plan = DeltaWrite::declared(
            table,
            input,
            deltalake::protocol::SaveMode::ErrorIfExists,
            CommitProperties::default(),
            check.clone(),
        )
        .unwrap();
        let state = context.state();
        let result = native_execution::run(&state, Arc::clone(&registry), &plan).await;
        if name == "bad" {
            assert!(result.is_err());
            let table = load(&attempt_location).await;
            assert_eq!(
                table.version(),
                Some(0),
                "only the empty declared table was committed"
            );
            check.verify(&table).unwrap();
            let scan = table
                .table_provider()
                .with_session(Arc::new(context.state()))
                .build()
                .await
                .unwrap();
            let batches = context
                .read_table(Arc::new(scan))
                .unwrap()
                .collect()
                .await
                .unwrap();
            assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 0);
        } else {
            result.unwrap();
        }
    }
    check
}

#[tokio::test]
async fn declared_delta_check_enforces_enum_and_identity_contracts_through_sql() {
    use pse_catalog::delta::contract::DeclaredCheck;
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    let (context, _) = context();
    let check = seed_declared(&context, &location).await;
    for sql in [
        "UPDATE edit SET kind = 'not_registered'",
        "UPDATE edit SET entity_id = X'01'",
        "INSERT INTO edit SELECT entity_id, package_id, 'not_registered', name, qualified_name, parent_entity_id, source_span FROM edit",
        "MERGE INTO edit USING (SELECT entity_id FROM edit) AS s ON edit.entity_id = s.entity_id WHEN MATCHED THEN UPDATE SET kind = 'not_registered'",
    ] {
        let table = load(&location).await;
        let version = table.version();
        let provider = WritableTable::declared(
            table,
            Arc::new(context.state()),
            CommitProperties::default(),
            check.clone(),
        )
        .await
        .unwrap();
        context.deregister_table("edit").unwrap();
        context.register_table("edit", Arc::new(provider)).unwrap();
        let result = sql_count(&context, &location, sql).await;
        assert!(result.is_err(), "{sql}: {result:?}");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("validation")
                || error.to_string().contains("relation admission")
                || error.to_string().contains("cast")
                || error.to_string().contains("FixedSizeBinary"),
            "{sql}: {error}"
        );
        assert_eq!(load(&location).await.version(), version);
    }
    assert_eq!(
        sql_count(&context, &location, "UPDATE edit SET name = 'renamed'")
            .await
            .unwrap(),
        1
    );
    assert!(
        DeclaredCheck::new(
            &pse_schema::shared_registry().unwrap(),
            pse_relations::generated::authored::packages::RELATION_ID
        )
        .unwrap()
        .verify(&load(&location).await)
        .is_err()
    );
}

#[tokio::test]
async fn cold_mapped_checks_bind_from_the_stored_execution_descriptor() {
    use datafusion::{common::Column, logical_expr::lit};
    use deltalake::delta_datafusion::{SessionFallbackPolicy, planner::DeltaPlanner};
    let root = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(root.path()).unwrap();
    let (context, _) = context();
    let check = seed_declared(&context, &location).await;
    assert!(check.properties().contains_key("pse.check.field.encoding"));
    let cold = SessionStateBuilder::new()
        .with_default_features()
        .with_query_planner(DeltaPlanner::new())
        .build();
    let table = load(&location).await;
    let cold = pse_catalog::delta::contract::DeclaredCheck::open(&table, &cold)
        .unwrap()
        .bind(&cold)
        .unwrap();
    let version = table.version();
    let refused = load(&location)
        .await
        .update()
        .with_update(Column::from_name("kind"), lit("not_registered"))
        .with_session_state(Arc::new(cold.clone()))
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .await;
    assert!(refused.is_err());
    assert_eq!(load(&location).await.version(), version);
    let (_, metrics) = load(&location)
        .await
        .update()
        .with_update(Column::from_name("name"), lit("cold update"))
        .with_session_state(Arc::new(cold))
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .await
        .unwrap();
    assert_eq!(metrics.num_updated_rows, 1);
    check.verify(&load(&location).await).unwrap();
}

#[tokio::test]
async fn delete_counts_real_files_without_delta_row_statistics() {
    use deltalake::kernel::Action;
    let root = tempfile::tempdir().unwrap();
    let seed_path = root.path().join("seed");
    let edit_path = root.path().join("edit");
    std::fs::create_dir_all(&edit_path).unwrap();
    let seed_location = url::Url::from_directory_path(&seed_path).unwrap();
    let edit_location = url::Url::from_directory_path(&edit_path).unwrap();
    let (context, _) = context();
    seed(&context, &seed_location).await;
    let original = load(&seed_location).await;
    let bytes = original
        .log_store()
        .read_commit_entry(0)
        .await
        .unwrap()
        .unwrap();
    let mut actions = vec![];
    for line in bytes.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
        if let Action::Add(mut add) = serde_json::from_slice(line).unwrap() {
            // An imported Parquet file may legitimately lack optional Delta statistics.
            // Construct that external-table fixture through native CreateBuilder actions.
            std::fs::copy(seed_path.join(&add.path), edit_path.join(&add.path)).unwrap();
            add.stats = None;
            actions.push(Action::Add(add));
        }
    }
    assert!(!actions.is_empty());
    DeltaTableBuilder::from_url(edit_location.clone())
        .unwrap()
        .build()
        .unwrap()
        .create()
        .with_columns(original.snapshot().unwrap().schema().fields().cloned())
        .with_actions(actions)
        .await
        .unwrap();
    bind(&context, &edit_location).await;
    assert_eq!(
        sql_count(&context, &edit_location, "TRUNCATE TABLE edit")
            .await
            .unwrap(),
        2
    );
    bind(&context, &edit_location).await;
    assert_eq!(
        context.table("edit").await.unwrap().count().await.unwrap(),
        0
    );
}
