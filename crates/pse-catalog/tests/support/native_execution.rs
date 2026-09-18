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
use pse_catalog::session::{PreparedComputation, SessionFactory};
use pse_ids::{CancellationToken, FixedBudget};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn prepare(
    state: &SessionState,
    registry: Arc<pse_schema::Registry>,
    plan: &LogicalPlan,
) -> Result<PreparedComputation> {
    let cancel = CancellationToken::new();
    let factory = SessionFactory::from_builder(
        Arc::clone(state.runtime_env()),
        FixedBudget::new(256 << 20),
        "native-fixture",
        SessionStateBuilder::new_from_existing(state.clone()),
    );
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

pub(crate) async fn run(
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
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
