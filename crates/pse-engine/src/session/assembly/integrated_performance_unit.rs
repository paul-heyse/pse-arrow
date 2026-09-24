// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(clippy::unwrap_used, reason = "isolated assembly lifetime controls")]

use super::*;
use crate::{
    provider::binding::{BindingKey, TableBinding},
    session::{EngineFactory, EngineSession},
};
use datafusion::{
    arrow::datatypes::{DataType, Field, Schema},
    catalog::{CatalogProviderList, TableProvider},
    common::TableReference,
    datasource::MemTable,
    execution::{context::SessionContext, session_state::SessionStateBuilder},
    logical_expr::registry::FunctionRegistry,
};
use pse_columnar::CancellationToken;
use pse_schema::model::provider::{OperationPurpose, ProviderPolicy, ProviderScope};

fn fixture() -> (EngineFactory, Arc<pse_schema::Registry>, EngineSession) {
    let context = SessionContext::new();
    let factory = EngineFactory::from_builder(
        context.runtime_env(),
        context.runtime_env().memory_pool.clone(),
        "unit",
        SessionStateBuilder::from(context.state()).with_query_planner(Arc::new(
            crate::session::planner::UnifiedPlanner::new(vec![]),
        )),
    );
    let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
    let session = factory
        .candidate(BTreeMap::new(), registry.clone(), &CancellationToken::new())
        .unwrap();
    (factory, registry, session)
}

#[test]
fn compatible_revisions_share_model_context_and_retained_selection() {
    let (factory, registry, first) = fixture();
    let next = factory
        .candidate(BTreeMap::new(), registry, &CancellationToken::new())
        .unwrap();
    assert!(Arc::ptr_eq(&first.native, &next.native));
    assert!(Arc::ptr_eq(
        &first.native.context.state_ref(),
        &next.native.context.state_ref()
    ));
    let selected = first.selection().unwrap();
    assert!(Arc::ptr_eq(&selected, &first.selection().unwrap()));
    assert!(Arc::ptr_eq(
        first.bound_state().unwrap().catalog_list(),
        first.bound_state().unwrap().catalog_list()
    ));
    let other = factory
        .candidate(
            BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &CancellationToken::new(),
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&first.native, &other.native));
    let mut policy = ProviderPolicy::new(pse_ids::SemanticId::NIL, ProviderScope::Root);
    policy
        .required_settings
        .insert("datafusion.execution.batch_size".into(), "7".into());
    let changed = first.with_policy(policy).unwrap();
    assert!(Arc::ptr_eq(&first.native, &changed.native));
    assert!(!Arc::ptr_eq(&selected, &changed.selection().unwrap()));
    assert_eq!(
        changed
            .bound_state()
            .unwrap()
            .config_options()
            .execution
            .batch_size
            .get(),
        7
    );
    assert!(
        !crate::session::reuse::Witness::capture(&first)
            .unwrap()
            .matches(&changed)
    );
}

#[tokio::test]
async fn selected_catalogs_are_immutable_and_unaffected_tables_keep_their_owners() {
    let (_, _, mut first) = fixture();
    let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
    let table: Arc<dyn TableProvider> =
        Arc::new(MemTable::try_new(schema.clone(), vec![vec![]]).unwrap());
    let name = TableReference::full("model", "unit", "numbers");
    let role = BindingKey::Input("numbers".into());
    first
        .bindings
        .insert(
            role.clone(),
            TableBinding::new(name.clone(), table.clone(), None, None),
        )
        .unwrap();
    let before = first.selection().unwrap();
    let native_schema = before
        .catalogs
        .catalog("model")
        .unwrap()
        .schema("unit")
        .unwrap();
    assert!(native_schema.deregister_table("numbers").is_err());
    assert!(
        !crate::session::reuse::Witness::capture(&first)
            .unwrap()
            .matches(&first),
        "a mutable source address is not an immutable source witness"
    );
    let mut next = first.clone();
    let unrelated = TableReference::full("model", "unit", "unrelated");
    next.bindings
        .insert(
            BindingKey::Native(unrelated.clone()),
            TableBinding::new(unrelated, table.clone(), None, None),
        )
        .unwrap();
    assert!(Arc::ptr_eq(
        &first.bindings.get(&role).unwrap(),
        &next.bindings.get(&role).unwrap()
    ));
    let replacement: Arc<dyn TableProvider> =
        Arc::new(MemTable::try_new(schema, vec![vec![]]).unwrap());
    next.bindings
        .replace_table(
            &name,
            &TableBinding::new(name.clone(), replacement.clone(), None, None),
        )
        .unwrap();
    assert!(Arc::ptr_eq(
        &native_schema.table("numbers").await.unwrap().unwrap(),
        &table
    ));
    let after = next.selection().unwrap();
    assert!(Arc::ptr_eq(
        &after
            .catalogs
            .catalog("model")
            .unwrap()
            .schema("unit")
            .unwrap()
            .table("numbers")
            .await
            .unwrap()
            .unwrap(),
        &replacement
    ));
    let mutable = next
        .with_purpose(OperationPurpose::Mutate)
        .bound_state()
        .unwrap();
    assert!(
        mutable
            .catalog_list()
            .catalog("model")
            .unwrap()
            .schema("unit")
            .unwrap()
            .deregister_table("numbers")
            .unwrap()
            .is_some()
    );
    assert!(
        after
            .catalogs
            .catalog("model")
            .unwrap()
            .schema("unit")
            .unwrap()
            .table_exist("numbers")
    );
}

#[tokio::test]
async fn query_boundaries_preserve_sql_definitions_and_refresh_attempt_services() {
    let (_, _, session) = fixture();
    session
        .native
        .context
        .sql("PREPARE retained (BIGINT) AS SELECT $1 AS value")
        .await
        .unwrap();
    // No lock is held across SQL planning. The sentinel proves state() refreshes
    // query time rather than retaining the model's earlier execution properties.
    session
        .native
        .context
        .state_ref()
        .write()
        .execution_props_mut()
        .query_execution_start_time = Some("1970-01-01T00:00:00Z".parse().unwrap());
    let state = session.bound_state().unwrap();
    assert_ne!(
        state.execution_props().query_execution_start_time,
        session
            .native
            .context
            .state_ref()
            .read()
            .execution_props()
            .query_execution_start_time
    );
    let copy = SessionContext::new_with_state(state.clone());
    assert!(copy.sql("EXECUTE retained(7)").await.is_ok());
    let bound = crate::session::execution::NativeExecutionContext::bind(
        &session,
        state,
        &CancellationToken::new(),
    )
    .unwrap();
    let first_cancel = CancellationToken::new();
    let second_cancel = CancellationToken::new();
    let first =
        crate::session::execution::NativeExecutionContext::execution_state(&bound, &first_cancel)
            .unwrap();
    let second =
        crate::session::execution::NativeExecutionContext::execution_state(&bound, &second_cancel)
            .unwrap();
    let a = crate::session::execution::NativeExecutionContext::from_session(&first).unwrap();
    let b = crate::session::execution::NativeExecutionContext::from_session(&second).unwrap();
    assert!(!Arc::ptr_eq(&a, &b));
    a.require_settlement();
    assert!(!b.must_settle());
    first_cancel.cancel();
    assert!(a.cancellation().checkpoint().is_err());
    assert!(b.cancellation().checkpoint().is_ok());
    assert!(
        SessionContext::new_with_state(second)
            .sql("EXECUTE retained(9)")
            .await
            .is_ok()
    );
    let nested = b.candidate_roles(&bound, BTreeMap::new());
    assert!(
        nested.is_err(),
        "a child cannot use another attempt's services"
    );
}

#[tokio::test]
async fn scoped_command_capture_preserves_definitions_and_actual_functions() {
    let (_, _, session) = fixture();
    session
        .native
        .context
        .sql("PREPARE retained (BIGINT) AS SELECT $1 AS value")
        .await
        .unwrap();
    let mut policy = ProviderPolicy::new(pse_ids::SemanticId::NIL, ProviderScope::Root);
    policy.max_bytes = Some(8 << 20);
    let scoped = session
        .with_policy(policy)
        .unwrap()
        .execution_scope()
        .unwrap();
    assert!(
        Arc::ptr_eq(&session.native, &scoped.native),
        "attempt quota is not another model context"
    );
    assert!(
        SessionContext::new_with_state(scoped.bound_state().unwrap())
            .sql("EXECUTE retained(9)")
            .await
            .is_ok()
    );
    let scoped_state = crate::session::execution::NativeExecutionContext::bind(
        &scoped,
        scoped.bound_state().unwrap(),
        &CancellationToken::new(),
    )
    .unwrap();
    let services =
        crate::session::execution::NativeExecutionContext::from_session(&scoped_state).unwrap();
    assert!(
        SessionContext::new_with_state(services.definition_state(&scoped_state))
            .sql("EXECUTE retained(11)")
            .await
            .is_ok()
    );
    assert!(Arc::ptr_eq(
        scoped_state.runtime_env(),
        scoped.execution_runtime.as_ref().unwrap()
    ));
    let mut completed_state = scoped_state.clone();
    *completed_state.config_mut() = completed_state.config().clone().with_batch_size(11);
    let replacement = Arc::new(
        completed_state.scalar_functions()["abs"]
            .as_ref()
            .clone()
            .with_aliases(["captured_abs"]),
    );
    completed_state.register_udf(replacement.clone()).unwrap();
    let mut completed = scoped.clone();
    completed.capture_configuration(&completed_state).unwrap();
    assert_ne!(
        completed.implementation_generation(),
        scoped.implementation_generation()
    );
    assert!(Arc::ptr_eq(
        &completed.scalar_function("captured_abs").unwrap(),
        &replacement
    ));
    assert!(
        completed
            .native
            .context
            .state()
            .config()
            .get_extension::<crate::session::execution::NativeExecutionContext>()
            .is_none()
    );
    let resulting = SessionContext::new_with_state(completed.bound_state().unwrap());
    assert_eq!(
        resulting
            .state()
            .config_options()
            .execution
            .batch_size
            .get(),
        11
    );
    assert!(resulting.sql("EXECUTE retained(13)").await.is_ok());
}

#[tokio::test]
async fn terminal_count_sample_and_existence_have_distinct_completion_contracts() {
    use crate::session::{SampleRetention, TerminalDemand};
    let (_, _, session) = fixture();
    let cancel = CancellationToken::new();
    let plan = datafusion::logical_expr::LogicalPlanBuilder::values(vec![
        vec![datafusion::logical_expr::lit(1i64)],
        vec![datafusion::logical_expr::lit(2i64)],
        vec![datafusion::logical_expr::lit(3i64)],
    ])
    .unwrap()
    .build()
    .unwrap();
    assert!(
        session
            .prepare_terminal(plan.clone(), TerminalDemand::Exists, &cancel)
            .unwrap()
            .exists(&cancel)
            .await
            .unwrap()
    );
    assert_eq!(
        session
            .prepare_terminal(plan.clone(), TerminalDemand::Count, &cancel)
            .unwrap()
            .count_rows(&cancel)
            .await
            .unwrap(),
        3
    );
    let sample = session
        .prepare_terminal(plan.clone(), TerminalDemand::Sample(2), &cancel)
        .unwrap()
        .sample_with_retention(2, SampleRetention::Compact, &cancel)
        .await
        .unwrap();
    assert_eq!(
        sample
            .batches
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        2
    );
    assert!(sample.truncated);
    let sample = session
        .prepare_terminal(plan.clone(), TerminalDemand::Sample(3), &cancel)
        .unwrap()
        .sample(3, &cancel)
        .await
        .unwrap();
    assert!(!sample.truncated);
    let sample = session
        .prepare_terminal(plan.clone(), TerminalDemand::Sample(0), &cancel)
        .unwrap()
        .sample(0, &cancel)
        .await
        .unwrap();
    assert!(sample.truncated && sample.batches.is_empty());
    assert!(
        session
            .prepare_terminal(plan.clone(), TerminalDemand::Count, &cancel)
            .unwrap()
            .exists(&cancel)
            .await
            .is_err()
    );
    assert!(
        session
            .prepare_terminal(plan, TerminalDemand::Exists, &cancel)
            .unwrap()
            .sample(3, &cancel)
            .await
            .is_err()
    );
    assert!(
        !session
            .prepare(
                datafusion::logical_expr::LogicalPlanBuilder::empty(false)
                    .build()
                    .unwrap(),
                &cancel
            )
            .unwrap()
            .exists(&cancel)
            .await
            .unwrap()
    );
}
