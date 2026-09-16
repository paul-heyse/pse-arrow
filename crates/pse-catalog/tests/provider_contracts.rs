// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Target hierarchy and general native-provider contracts (Plan 06 V01–V04/V07).

use std::{collections::BTreeMap, sync::Arc};

use datafusion::{
    arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema},
    },
    catalog::{CatalogProvider, CatalogProviderList, TableProvider},
    common::{Constraint, Constraints, TableReference},
    datasource::{MemTable, provider_as_source},
    execution::runtime_env::RuntimeEnv,
    logical_expr::{LogicalPlanBuilder, TableType},
};
use datafusion_catalog::{memory::MemoryCatalogProvider, view::ViewTable};
use pse_catalog::{
    provider::list::SnapshotCatalogList,
    session::{
        ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
    },
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_schema::RegistryBuilder;

#[expect(clippy::unwrap_used, reason = "fixed native test environment")]
fn session() -> SnapshotSession {
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
    .candidate(
        BTreeMap::new(),
        Arc::new(RegistryBuilder::new().build().unwrap()),
        &CancellationToken::new(),
    )
    .unwrap()
}

#[expect(clippy::unwrap_used, reason = "fixed Arrow test fixture")]
fn table() -> MemTable {
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::UInt64, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(UInt64Array::from(vec![7, 7, 11]))],
    )
    .unwrap();
    MemTable::try_new(schema, vec![vec![batch]]).unwrap()
}

#[test]
fn root_registration_inserts_and_returns_the_exact_replaced_owner() {
    let list = SnapshotCatalogList::new(BTreeMap::new());
    let first: Arc<dyn CatalogProvider> = Arc::new(MemoryCatalogProvider::new());
    let second: Arc<dyn CatalogProvider> = Arc::new(MemoryCatalogProvider::new());
    assert!(
        list.register_catalog("quoted.Name".into(), Arc::clone(&first))
            .is_none()
    );
    assert!(Arc::ptr_eq(&list.catalog("quoted.Name").unwrap(), &first));
    assert!(Arc::ptr_eq(
        &list
            .register_catalog("quoted.Name".into(), Arc::clone(&second))
            .unwrap(),
        &first
    ));
    assert!(Arc::ptr_eq(&list.catalog("quoted.Name").unwrap(), &second));
    assert_eq!(list.catalog_names(), ["quoted.Name"]);
    assert_eq!(list.generation(), 2);
}

#[tokio::test]
async fn native_created_tables_preserve_defaults_and_get_private_mutation_generations() {
    use pse_schema::model::provider::OperationPurpose;
    let cancel = CancellationToken::new();
    let original = session().with_purpose(OperationPurpose::Mutate);
    let created = original
        .prepare_sql(
            "CREATE TABLE authored.defaults (id BIGINT PRIMARY KEY, value BIGINT DEFAULT 17)",
            &cancel,
        )
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .resulting_session(&cancel)
        .await
        .unwrap();
    let inserted = created
        .prepare_sql("INSERT INTO authored.defaults (id) VALUES (1)", &cancel)
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .resulting_session(&cancel)
        .await
        .unwrap();
    let rows = inserted
        .sql("SELECT value FROM authored.defaults", &cancel)
        .await
        .unwrap();
    assert_eq!(
        rows[0]
            .column(0)
            .as_any()
            .downcast_ref::<datafusion::arrow::array::Int64Array>()
            .unwrap()
            .value(0),
        17
    );
    assert_eq!(
        created
            .sql("SELECT * FROM authored.defaults", &cancel)
            .await
            .unwrap()
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>(),
        0
    );
    assert!(
        inserted
            .prepare_sql("INSERT INTO authored.defaults (id) VALUES (1)", &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
}

#[expect(
    clippy::unwrap_used,
    reason = "assert the native affected-row count contract"
)]
fn affected(completed: &pse_catalog::session::CompletedComputation) -> u64 {
    assert_eq!(completed.batches().len(), 1);
    assert_eq!(completed.batches()[0].num_rows(), 1);
    completed.batches()[0]
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .unwrap()
        .value(0)
}

#[expect(clippy::unwrap_used, reason = "successful native command fixture")]
async fn complete_sql(
    session: &SnapshotSession,
    sql: &str,
    cancel: &CancellationToken,
) -> pse_catalog::session::CompletedComputation {
    session
        .prepare_sql(sql, cancel)
        .await
        .unwrap()
        .execute(cancel)
        .await
        .unwrap()
}

#[tokio::test]
async fn private_native_dml_commits_a_new_generation_and_preserves_prepared_readers() {
    use pse_catalog::session::mutation::MemoryTableFactory;
    use pse_schema::model::provider::OperationPurpose;
    let cancel = CancellationToken::new();
    let base = session();
    let captured = base
        .prepare_provider_capture(
            TableReference::full("source", "public", "seed"),
            Arc::new(table()),
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let reference = TableReference::full("model", "authored", "editable");
    let original = base
        .with_mutable_capture(&reference, &captured, Arc::new(MemoryTableFactory), &cancel)
        .unwrap()
        .with_purpose(OperationPurpose::Mutate);
    let reader = original
        .prepare_sql("SELECT id FROM authored.editable ORDER BY id", &cancel)
        .await
        .unwrap();
    let delete = original
        .prepare_sql("DELETE FROM authored.editable WHERE id = 7", &cancel)
        .await
        .unwrap();
    let explained = complete_sql(
        &original.with_purpose(OperationPurpose::Inspect),
        "EXPLAIN DELETE FROM authored.editable WHERE id = 7",
        &cancel,
    )
    .await;
    assert!(explained.batches().iter().any(|batch| batch.num_rows() > 0));
    // Opening and dropping a physical stream must not run native eager DML hooks.
    let dropped = original
        .prepare_sql("DELETE FROM authored.editable", &cancel)
        .await
        .unwrap();
    drop(dropped.execute_stream(&cancel).await.unwrap());
    let completed = delete.clone().execute(&cancel).await.unwrap();
    assert_eq!(affected(&completed), 2);
    assert!(delete.execute(&cancel).await.is_err());
    let current = completed.resulting_session(&cancel).await.unwrap();
    assert_eq!(
        reader
            .execute(&cancel)
            .await
            .unwrap()
            .batches()
            .iter()
            .map(|b| b.num_rows())
            .sum::<usize>(),
        3
    );
    assert_eq!(
        current
            .sql("SELECT * FROM authored.editable", &cancel)
            .await
            .unwrap()
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>(),
        1
    );
    let inserted = complete_sql(
        &current,
        "INSERT INTO authored.editable VALUES (19), (23)",
        &cancel,
    )
    .await;
    assert_eq!(affected(&inserted), 2);
    let current = inserted.resulting_session(&cancel).await.unwrap();
    let updated = complete_sql(
        &current,
        "UPDATE authored.editable SET id = id + 1 WHERE id > 11",
        &cancel,
    )
    .await;
    assert_eq!(affected(&updated), 2);
    let result = updated
        .resulting_session(&cancel)
        .await
        .unwrap()
        .sql("SELECT id FROM authored.editable ORDER BY id", &cancel)
        .await
        .unwrap();
    let ids = result
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap()
                .values()
                .to_vec()
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, [11, 20, 24]);
}

#[tokio::test]
async fn private_dml_rejects_key_violations_and_truthfully_reports_unsupported_hooks() {
    use pse_catalog::session::mutation::MemoryTableFactory;
    use pse_schema::model::provider::OperationPurpose;
    let cancel = CancellationToken::new();
    let base = session();
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::UInt64, false)]));
    let source: Arc<dyn TableProvider> = Arc::new(
        MemTable::try_new(
            Arc::clone(&schema),
            vec![vec![
                RecordBatch::try_new(schema, vec![Arc::new(UInt64Array::from(vec![1, 2]))])
                    .unwrap(),
            ]],
        )
        .unwrap()
        .with_constraints(Constraints::new_unverified(vec![Constraint::PrimaryKey(
            vec![0],
        )])),
    );
    let captured = base
        .prepare_provider_capture(
            TableReference::full("source", "public", "seed"),
            source,
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let original = base
        .with_mutable_capture(
            &TableReference::full("model", "authored", "editable"),
            &captured,
            Arc::new(MemoryTableFactory),
            &cancel,
        )
        .unwrap()
        .with_purpose(OperationPurpose::Mutate);
    for sql in [
        "INSERT INTO authored.editable VALUES (1)",
        "UPDATE authored.editable SET id = 1",
    ] {
        let error = original
            .prepare_sql(sql, &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("key"), "{sql}: {error}");
    }
    let truncate = original
        .prepare_sql("TRUNCATE TABLE authored.editable", &cancel)
        .await
        .unwrap();
    assert!(
        truncate
            .execute(&cancel)
            .await
            .unwrap_err()
            .to_string()
            .contains("not supported")
    );
    assert_eq!(
        original
            .sql("SELECT * FROM authored.editable", &cancel)
            .await
            .unwrap()
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>(),
        2
    );
    let live = base
        .with_provider(
            TableReference::full("model", "authored", "live"),
            Arc::new(table()),
            &cancel,
        )
        .unwrap()
        .with_purpose(OperationPurpose::Mutate);
    assert!(
        live.prepare_sql("DELETE FROM authored.live", &cancel)
            .await
            .unwrap_err()
            .to_string()
            .contains("private-table factory")
    );
}

#[tokio::test]
async fn native_provider_sql_metadata_and_direct_plans_share_the_actual_source() {
    let cancel = CancellationToken::new();
    let provider: Arc<dyn TableProvider> = Arc::new(table());
    let reference = TableReference::full("External", "operations.inputs", "Before");
    let session = session()
        .with_provider(reference.clone(), Arc::clone(&provider), &cancel)
        .unwrap();
    let plan = LogicalPlanBuilder::scan(reference, provider_as_source(provider), None)
        .unwrap()
        .build()
        .unwrap();
    let direct = session.execute_plan(plan, &cancel).await.unwrap();
    let sql = session
        .sql(
            "SELECT * FROM \"External\".\"operations.inputs\".\"Before\"",
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(direct, sql);
    assert_eq!(direct.iter().map(RecordBatch::num_rows).sum::<usize>(), 3);
    let metadata = session.sql("SELECT table_name FROM information_schema.tables WHERE table_catalog = 'External' AND table_schema = 'operations.inputs'", &cancel).await.unwrap();
    assert_eq!(metadata.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert!(!session.read_back_settings().await.unwrap().is_empty());
}

#[tokio::test]
async fn native_view_uses_the_same_admission_without_a_concrete_type_branch() {
    let cancel = CancellationToken::new();
    let provider: Arc<dyn TableProvider> = Arc::new(table());
    let reference = TableReference::full("model", "authored", "input");
    let session = session()
        .with_provider(reference.clone(), Arc::clone(&provider), &cancel)
        .unwrap();
    let plan = LogicalPlanBuilder::scan(reference, provider_as_source(provider), None)
        .unwrap()
        .build()
        .unwrap();
    let view: Arc<dyn TableProvider> = Arc::new(ViewTable::new(plan, None));
    assert_eq!(view.table_type(), TableType::View);
    let session = session
        .with_provider(
            TableReference::full("model", "derived", "view"),
            view,
            &cancel,
        )
        .unwrap();
    let batches = session
        .sql("SELECT * FROM derived.view", &cancel)
        .await
        .unwrap();
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 3);
}

#[test]
fn a_native_provider_cannot_advertise_unestablished_keys() {
    let provider: Arc<dyn TableProvider> =
        Arc::new(table().with_constraints(Constraints::new_unverified(vec![
            Constraint::PrimaryKey(vec![0]),
        ])));
    assert!(
        session()
            .with_provider(
                TableReference::full("model", "authored", "input"),
                provider,
                &CancellationToken::new()
            )
            .is_err()
    );
}

#[tokio::test]
async fn general_capture_validates_keys_before_exposing_them_and_keeps_immutable_rows() {
    let cancel = CancellationToken::new();
    let session = session();
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::UInt64, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(UInt64Array::from(vec![7, 11, 13]))],
    )
    .unwrap();
    let constraints = Constraints::new_unverified(vec![Constraint::PrimaryKey(vec![0])]);
    let provider: Arc<dyn TableProvider> = Arc::new(
        MemTable::try_new(schema, vec![vec![batch]])
            .unwrap()
            .with_constraints(constraints.clone()),
    );
    let prepared = session
        .prepare_provider_capture(
            TableReference::full("external", "inputs", "source"),
            Arc::clone(&provider),
            &cancel,
        )
        .unwrap();
    assert!(
        prepared
            .computation()
            .original_plan()
            .display_indent()
            .to_string()
            .contains("pse_capture_values")
    );
    let captured = prepared.execute(&cancel).await.unwrap();
    assert_eq!(captured.provider().constraints(), Some(&constraints));
    let context = datafusion::execution::context::SessionContext::new();
    context.register_table("source", provider).unwrap();
    context
        .sql("DELETE FROM source WHERE id = 11")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let snapshot = session
        .with_captured_provider(
            TableReference::full("model", "derived", "captured"),
            &captured,
            &cancel,
        )
        .unwrap();
    let rows = snapshot
        .sql("SELECT * FROM derived.captured ORDER BY id", &cancel)
        .await
        .unwrap();
    let values = rows
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap()
                .values()
                .iter()
                .copied()
        })
        .collect::<Vec<_>>();
    assert_eq!(values, [7, 11, 13]);
}

#[tokio::test]
async fn general_capture_refuses_forged_primary_keys_without_optimizer_elimination() {
    let cancel = CancellationToken::new();
    let provider: Arc<dyn TableProvider> =
        Arc::new(table().with_constraints(Constraints::new_unverified(vec![
            Constraint::PrimaryKey(vec![0]),
        ])));
    let prepared = session()
        .prepare_provider_capture(
            TableReference::full("external", "inputs", "forged"),
            provider,
            &cancel,
        )
        .unwrap();
    let error = prepared.execute(&cancel).await.unwrap_err();
    assert!(
        matches!(error, pse_catalog::CatalogError::Admission { path, .. } if path == "provider.capture")
    );
}

#[tokio::test]
async fn captured_unique_keys_preserve_sql_null_semantics_and_primary_keys_refuse_nulls() {
    let cancel = CancellationToken::new();
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::UInt64, true)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(UInt64Array::from(vec![Some(1), None, None]))],
    )
    .unwrap();
    for primary in [false, true] {
        let constraint = if primary {
            Constraint::PrimaryKey(vec![0])
        } else {
            Constraint::Unique(vec![0])
        };
        let provider: Arc<dyn TableProvider> = Arc::new(
            MemTable::try_new(Arc::clone(&schema), vec![vec![batch.clone()]])
                .unwrap()
                .with_constraints(Constraints::new_unverified(vec![constraint])),
        );
        let result = session()
            .prepare_provider_capture(
                TableReference::full("external", "inputs", "nullable"),
                provider,
                &cancel,
            )
            .unwrap()
            .execute(&cancel)
            .await;
        assert_eq!(
            result.is_err(),
            primary,
            "only PRIMARY KEY forbids null key values"
        );
    }
}

#[tokio::test]
async fn native_capture_checks_keys_across_partitions_and_preserves_literal_column_names() {
    let cancel = CancellationToken::new();
    let schema = Arc::new(Schema::new(vec![Field::new(
        "id.with.dot",
        DataType::UInt64,
        false,
    )]));
    for duplicate in [false, true] {
        let partitions = [vec![1, 2], vec![3, if duplicate { 1 } else { 4 }]]
            .into_iter()
            .map(|values| {
                vec![
                    RecordBatch::try_new(
                        Arc::clone(&schema),
                        vec![Arc::new(UInt64Array::from(values))],
                    )
                    .unwrap(),
                ]
            })
            .collect();
        let source = MemTable::try_new(Arc::clone(&schema), partitions)
            .unwrap()
            .with_constraints(Constraints::new_unverified(vec![Constraint::PrimaryKey(
                vec![0],
            )]));
        let prepared = session()
            .prepare_provider_capture(
                TableReference::full("external", "inputs", "partitioned"),
                Arc::new(source),
                &cancel,
            )
            .unwrap();
        let plan = prepared
            .computation()
            .optimized_plan()
            .display_indent()
            .to_string();
        assert!(plan.contains("WindowAggr"), "{plan}");
        assert!(plan.contains("pse_capture_key"), "{plan}");
        assert_eq!(plan.matches("TableScan:").count(), 1, "{plan}");
        let captured = prepared.execute(&cancel).await;
        assert_eq!(captured.is_err(), duplicate);
        if let Ok(captured) = captured {
            assert_eq!(captured.provider().schema(), schema);
            assert_eq!(
                captured
                    .batches()
                    .iter()
                    .map(|b| b.num_rows())
                    .sum::<usize>(),
                4
            );
        }
    }
}

#[tokio::test]
async fn native_function_owners_execute_without_sql_registration_or_name_substitution() {
    use datafusion::logical_expr::{Volatility, col, create_udf};
    let cancel = CancellationToken::new();
    let source: Arc<dyn TableProvider> = Arc::new(table());
    let reference = TableReference::full("model", "authored", "input");
    let session = session()
        .with_provider(reference.clone(), Arc::clone(&source), &cancel)
        .unwrap();
    // Deliberately share a built-in name: a bound expression owns this implementation.
    let identity = create_udf(
        "sqrt",
        vec![DataType::UInt64],
        DataType::UInt64,
        Volatility::Immutable,
        Arc::new(|args| Ok(args[0].clone())),
    );
    let plan = LogicalPlanBuilder::scan(reference, provider_as_source(source), None)
        .unwrap()
        .project(vec![identity.call(vec![col("id")]).alias("value")])
        .unwrap()
        .build()
        .unwrap();
    let prepared = session.prepare(plan.clone(), &cancel).unwrap();
    let completed = prepared.execute(&cancel).await.unwrap();
    let values = completed
        .batches()
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap()
                .values()
                .iter()
                .copied()
        })
        .collect::<Vec<_>>();
    assert_eq!(values, [7, 7, 11]);
    // The old diagnostic codec cannot reconstruct this binding from a SQL name.
    assert!(session.encode_plan(&plan, &cancel).is_err());
}

#[tokio::test]
async fn materialized_native_roles_release_producing_providers_and_plans() {
    let cancel = CancellationToken::new();
    let base = session();
    let source = Arc::new(table());
    let source_owner = Arc::downgrade(&source);
    let producer = base
        .with_provider(
            TableReference::full("model", "authored", "input"),
            source.clone(),
            &cancel,
        )
        .unwrap();
    let completed = complete_sql(&producer, "SELECT id FROM authored.input", &cancel).await;
    let captured = base
        .prepare_provider_capture(
            TableReference::full("observed", "inputs", "source"),
            source.clone(),
            &cancel,
        )
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let consumer = base
        .with_computation_roles(
            BTreeMap::from([("materialized".into(), completed)]),
            &cancel,
        )
        .unwrap();
    drop((producer, source));
    assert!(source_owner.upgrade().is_none());
    assert_eq!(
        captured
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        3
    );
    let provider = datafusion::datasource::source_as_provider(
        &consumer.computation_source("materialized").unwrap(),
    )
    .unwrap();
    assert!(provider.get_logical_plan().is_none());
    assert!(provider.as_ref().downcast_ref::<MemTable>().is_none());
    assert!(
        provider
            .truncate(
                &datafusion::execution::session_state::SessionStateBuilder::new()
                    .with_default_features()
                    .build()
            )
            .await
            .is_err()
    );
    let plan = consumer.scan_computation_role("materialized").unwrap();
    let rows = consumer.execute_plan(plan, &cancel).await.unwrap();
    assert_eq!(rows.iter().map(RecordBatch::num_rows).sum::<usize>(), 3);
}

#[test]
fn provider_capture_checks_view_dependencies_before_native_inlining() {
    let provider: Arc<dyn TableProvider> =
        Arc::new(table().with_constraints(Constraints::new_unverified(vec![
            Constraint::PrimaryKey(vec![0]),
        ])));
    let plan = LogicalPlanBuilder::scan("unadmitted", provider_as_source(provider), None)
        .unwrap()
        .build()
        .unwrap();
    let view: Arc<dyn TableProvider> = Arc::new(ViewTable::new(plan, None));
    assert!(
        session()
            .prepare_provider_capture(
                TableReference::full("external", "views", "forged"),
                view,
                &CancellationToken::new()
            )
            .is_err()
    );
}

#[tokio::test]
async fn preparing_a_ddl_command_cannot_execute_the_eager_native_handler() {
    let session = session();
    let cancel = CancellationToken::new();
    assert!(
        session
            .prepare_sql("CREATE TABLE authored.created (id BIGINT)", &cancel)
            .await
            .is_err()
    );
    assert!(
        session
            .sql("SELECT * FROM authored.created", &cancel)
            .await
            .is_err()
    );
    let command_session =
        session.with_purpose(pse_schema::model::provider::OperationPurpose::Mutate);
    let prepared = command_session
        .prepare_sql("CREATE TABLE authored.created (id BIGINT)", &cancel)
        .await
        .unwrap();
    assert!(
        command_session
            .sql("SELECT * FROM authored.created", &cancel)
            .await
            .is_err()
    );
    let completed = prepared.clone().execute(&cancel).await.unwrap();
    assert!(prepared.execute(&cancel).await.is_err());
    let resulting = completed.resulting_session(&cancel).await.unwrap();
    assert!(
        resulting
            .sql("SELECT * FROM authored.created", &cancel)
            .await
            .unwrap()
            .iter()
            .all(|batch| batch.num_rows() == 0)
    );
    assert!(
        session
            .sql("SELECT * FROM authored.created", &cancel)
            .await
            .is_err()
    );
}

fn policy(
    n: u8,
    scope: pse_schema::model::provider::ProviderScope,
) -> pse_schema::model::provider::ProviderPolicy {
    pse_schema::model::provider::ProviderPolicy::new(
        pse_ids::SemanticId::from_bytes(u128::from(n).to_be_bytes()),
        scope,
    )
}

#[tokio::test]
async fn destination_policies_apply_before_a_table_or_namespace_exists() {
    use pse_schema::model::provider::{
        OperationEffect as E, OperationPurpose as P, ProviderScope as S,
    };
    let cancel = CancellationToken::new();
    let mut table = policy(
        21,
        S::Table("model".into(), "authored".into(), "refused".into()),
    );
    table.effects.remove(&E::Namespace);
    let mut schema = policy(22, S::Schema("model".into(), "future.scope".into()));
    schema.effects.remove(&E::Namespace);
    let mut catalog = policy(23, S::Catalog("future".into()));
    catalog.effects.remove(&E::Namespace);
    let session = session()
        .with_policies([table, schema, catalog])
        .unwrap()
        .with_purpose(P::Mutate);
    for sql in [
        "CREATE TABLE authored.refused (id BIGINT)",
        "CREATE SCHEMA \"future.scope\"",
        "CREATE DATABASE future",
    ] {
        let error = session.prepare_sql(sql, &cancel).await.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("outside the effective purpose/policy"),
            "{sql}: {error}"
        );
    }
    assert!(
        session
            .prepare_sql("CREATE TABLE authored.allowed (id BIGINT)", &cancel)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn private_quoted_namespaces_survive_capture_and_obey_native_cascade() {
    use pse_schema::model::provider::OperationPurpose;
    let cancel = CancellationToken::new();
    let original = session().with_purpose(OperationPurpose::Mutate);
    let mut current = original.clone();
    for sql in [
        "CREATE DATABASE \"Next.Catalog\"",
        "CREATE SCHEMA \"Next.Catalog\".\"New.Scope\"",
        "CREATE TABLE \"Next.Catalog\".\"New.Scope\".\"A.Table\" (id BIGINT)",
    ] {
        current = current
            .prepare_sql(sql, &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap()
            .resulting_session(&cancel)
            .await
            .unwrap();
    }
    let sql = "SELECT * FROM \"Next.Catalog\".\"New.Scope\".\"A.Table\"";
    assert!(current.sql(sql, &cancel).await.is_ok());
    assert!(original.sql(sql, &cancel).await.is_err());
    assert!(
        current
            .prepare_sql("DROP SCHEMA \"Next.Catalog\".\"New.Scope\"", &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
    current = current
        .prepare_sql(
            "DROP SCHEMA \"Next.Catalog\".\"New.Scope\" CASCADE",
            &cancel,
        )
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .resulting_session(&cancel)
        .await
        .unwrap();
    assert!(current.sql(sql, &cancel).await.is_err());
    // An explicitly dropped configured default schema must not be recreated by lookup.
    current = current
        .prepare_sql("DROP SCHEMA authored", &cancel)
        .await
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .resulting_session(&cancel)
        .await
        .unwrap();
    assert!(
        current
            .prepare_sql("CREATE TABLE authored.unavailable (id BIGINT)", &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
}

#[test]
fn policies_conjoin_requirements_restrict_effects_and_resolve_defaults_with_origins() {
    use pse_catalog::session::policy::EffectivePolicy;
    use pse_schema::model::provider::{
        OperationEffect as E, OperationPurpose as P, ProviderScope as S,
    };
    let mut root = policy(1, S::Root);
    root.defaults.insert("setting".into(), "root".into());
    root.max_bytes = Some(1024);
    root.effects.remove(&E::Publish);
    root.requirements.insert(root.id);
    let mut a = policy(2, S::Catalog("a".into()));
    a.defaults.insert("setting".into(), "a".into());
    a.max_bytes = Some(512);
    a.requirements.insert(a.id);
    let mut b = policy(3, S::Catalog("b".into()));
    b.defaults.insert("setting".into(), "b".into());
    assert!(EffectivePolicy::compose(P::Publish, [&root, &a, &b]).is_err());
    let mut invocation = policy(4, S::Invocation);
    invocation
        .defaults
        .insert("setting".into(), "chosen".into());
    let effective = EffectivePolicy::compose(P::Publish, [&root, &a, &b, &invocation]).unwrap();
    assert_eq!(effective.settings["setting"], "chosen");
    assert_eq!(
        effective.setting_origins["setting"],
        [invocation.id].into_iter().collect()
    );
    assert_eq!(effective.requirements.len(), 2);
    assert!(!effective.effects.contains(&E::Publish));
    assert_eq!(effective.max_bytes, Some(512));
    a.required_settings
        .insert("setting".into(), "required_a".into());
    b.required_settings
        .insert("setting".into(), "required_b".into());
    assert!(EffectivePolicy::compose(P::Publish, [&a, &b, &invocation]).is_err());
}

#[tokio::test]
async fn scalar_only_plans_capture_policy_configuration_and_purpose() {
    use pse_schema::model::provider::{
        OperationEffect as E, OperationPurpose as P, ProviderScope as S,
    };
    let base = session();
    let mut declaration = policy(5, S::Root);
    declaration
        .defaults
        .insert("datafusion.execution.batch_size".into(), "2".into());
    declaration.effects.remove(&E::Namespace);
    let selected = base
        .with_policy(declaration)
        .unwrap()
        .with_purpose(P::Mutate);
    assert_eq!(
        selected.read_back_settings().await.unwrap()["datafusion.execution.batch_size"],
        Some("2".into())
    );
    assert!(!base.matches_semantic_inputs(&selected.semantic_inputs()));
    assert!(
        selected
            .prepare_sql(
                "CREATE TABLE authored.refused (n BIGINT)",
                &CancellationToken::new()
            )
            .await
            .is_err()
    );
    let original = selected
        .prepare_sql("SELECT 42", &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        original
            .execute(&CancellationToken::new())
            .await
            .unwrap()
            .batches()[0]
            .num_rows(),
        1
    );
}

#[tokio::test]
async fn stream_keeps_batch_owners_and_refuses_partial_materialization() {
    let cancel = CancellationToken::new();
    let source = session()
        .with_provider(
            TableReference::full("model", "authored", "input"),
            Arc::new(table()),
            &cancel,
        )
        .unwrap();
    let prepared = source
        .prepare_sql("SELECT * FROM authored.input", &cancel)
        .await
        .unwrap();
    let mut stream = prepared.execute_stream(&cancel).await.unwrap();
    let batch = stream.next_batch(&cancel).await.unwrap().unwrap();
    drop(source);
    assert_eq!(batch.num_rows(), 3);
    assert!(stream.collect(&cancel).await.is_err());
    assert_eq!(batch.num_rows(), 3);
}
