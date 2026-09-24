// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fixtures bind their real sources and DML targets through product preparation.
use datafusion::{
    arrow::array::RecordBatch,
    common::{DataFusionError, Result, TableReference, tree_node::TreeNodeRecursion},
    datasource::source_as_provider,
    execution::session_state::{SessionState, SessionStateBuilder},
    logical_expr::LogicalPlan,
};
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineFactory, PreparedComputation};
use std::{collections::BTreeMap, sync::Arc};

/// Bind the actual native providers through production preparation.
/// # Errors
/// Conflicting sources, admission or preparation fails.
pub fn prepare(
    state: &SessionState,
    registry: Arc<pse_schema::Registry>,
    plan: &LogicalPlan,
) -> Result<PreparedComputation> {
    prepare_with_observation(
        state,
        registry,
        plan,
        pse_engine::session::assurance::ObservationPolicy::Contract,
    )
}

/// Bind native sources with explicitly requested observation.
/// # Errors
/// Conflicting sources, admission or preparation fails.
pub fn prepare_with_observation(
    state: &SessionState,
    registry: Arc<pse_schema::Registry>,
    plan: &LogicalPlan,
    observation: pse_engine::session::assurance::ObservationPolicy,
) -> Result<PreparedComputation> {
    let cancel = CancellationToken::new();
    let factory = EngineFactory::from_builder(
        Arc::clone(state.runtime_env()),
        Arc::clone(&state.runtime_env().memory_pool),
        "native-fixture",
        SessionStateBuilder::new_from_existing(state.clone()),
    )
    .with_observation(observation);
    let mut session = factory
        .candidate_checked(BTreeMap::new(), registry, &cancel)
        .map_err(external)?
        .with_purpose(pse_schema::model::provider::OperationPurpose::Publish);
    let mut sources = BTreeMap::new();
    plan.apply_with_subqueries(|node| {
        let source = match node {
            LogicalPlan::TableScan(scan) => Some((&scan.table_name, &scan.source)),
            LogicalPlan::Dml(command) => Some((&command.table_name, &command.target)),
            _ => None,
        };
        if let Some((name, source)) = source {
            let provider = source_as_provider(source)?;
            let defaults = &state.config_options().catalog;
            let name = name
                .clone()
                .resolve(&defaults.default_catalog, &defaults.default_schema);
            let name = TableReference::full(name.catalog, name.schema, name.table);
            if let Some(previous) = sources.insert(name, Arc::clone(&provider))
                && !Arc::ptr_eq(&previous, &provider)
            {
                return Err(DataFusionError::Plan(
                    "fixture has conflicting native providers".into(),
                ));
            }
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    for (name, provider) in sources {
        session = session
            .with_provider(name, provider, &cancel)
            .map_err(external)?;
    }
    session.prepare(plan.clone(), &cancel).map_err(external)
}

/// Prepare and fully consume a native fixture operation.
/// # Errors
/// Admission, preparation or execution fails.
pub async fn run(
    state: &SessionState,
    registry: Arc<pse_schema::Registry>,
    plan: &LogicalPlan,
) -> Result<Vec<RecordBatch>> {
    Ok(prepare(state, registry, plan)?
        .execute(&CancellationToken::new())
        .await
        .map_err(external)?
        .into_batches())
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}
