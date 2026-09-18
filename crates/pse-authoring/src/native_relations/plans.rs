// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native keyed change plans. Explicit side-presence is independent of payload nulls.

use super::{contract, exact};
use crate::AuthoringError;
use datafusion::{
    arrow::array::RecordBatch,
    common::{Column, DataFusionError, NullEquality},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationSpec;
use std::{collections::BTreeMap, ops::Not, sync::Arc};

pub(crate) fn engine(error: DataFusionError) -> AuthoringError {
    AuthoringError::Catalog(pse_catalog::CatalogError::Multiple {
        errors: pse_catalog::classify(error, pse_catalog::PlanOrigin::RuleCompiler),
    })
}

pub(crate) fn session(session: &SnapshotSession) -> Result<SnapshotSession, AuthoringError> {
    Ok(session.with_scalar_functions(vec![exact::function()])?)
}

pub(crate) fn roles(
    session: &SnapshotSession,
    before: FieldCheckedBatch,
    after: FieldCheckedBatch,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, AuthoringError> {
    Ok(session.with_checked_role_inputs(
        BTreeMap::from([
            ("change_before".to_owned(), before),
            ("change_after".to_owned(), after),
        ]),
        cancel,
    )?)
}

fn qualified(alias: &str, name: &str) -> Expr {
    Expr::Column(Column::new(Some(alias), name))
}

pub(super) fn join(
    left: LogicalPlan,
    right: LogicalPlan,
    spec: &RelationSpec,
    kind: JoinType,
) -> Result<LogicalPlanBuilder, AuthoringError> {
    if spec.primary_key.is_empty() {
        return Err(contract("change relation has no declared primary key"));
    }
    let left = LogicalPlanBuilder::from(left)
        .alias("before")
        .map_err(engine)?;
    let right = LogicalPlanBuilder::from(right)
        .alias("after")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let keys = spec
        .primary_key
        .iter()
        .map(|name| {
            (
                Column::new(Some("before"), *name),
                Column::new(Some("after"), *name),
            )
        })
        .unzip();
    left.join_detailed(right, kind, keys, None, NullEquality::NullEqualsNull)
        .map_err(engine)
}

pub(super) fn row_equal(spec: &RelationSpec) -> Expr {
    spec.columns
        .iter()
        .map(|column| {
            exact::equal(
                qualified("before", column.name()),
                qualified("after", column.name()),
            )
        })
        .reduce(Expr::and)
        .unwrap_or_else(|| lit(true))
}

fn present(
    plan: LogicalPlan,
    spec: &RelationSpec,
    name: &str,
) -> Result<LogicalPlan, AuthoringError> {
    let mut columns = spec
        .columns
        .iter()
        .map(|column| col(column.name()))
        .collect::<Vec<_>>();
    columns.push(lit(true).alias(name));
    LogicalPlanBuilder::from(plan)
        .project(columns)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(crate) fn difference(
    session: &SnapshotSession,
    spec: &RelationSpec,
) -> Result<LogicalPlan, AuthoringError> {
    let left = present(
        session.scan_role("change_before")?,
        spec,
        "__before_present",
    )?;
    let right = present(session.scan_role("change_after")?, spec, "__after_present")?;
    let before_present = qualified("before", "__before_present");
    let after_present = qualified("after", "__after_present");
    let different = before_present
        .clone()
        .is_null()
        .or(after_present.clone().is_null())
        .or(row_equal(spec).not());
    let mut columns = vec![
        before_present.alias("before_present"),
        after_present.alias("after_present"),
    ];
    columns.extend(spec.columns.iter().enumerate().map(|(index, column)| {
        qualified("before", column.name()).alias(format!("before_{index}"))
    }));
    columns.extend(
        spec.columns.iter().enumerate().map(|(index, column)| {
            qualified("after", column.name()).alias(format!("after_{index}"))
        }),
    );
    let ordering = spec
        .primary_key
        .iter()
        .map(|name| {
            datafusion::functions::core::expr_fn::coalesce(vec![
                qualified("before", name),
                qualified("after", name),
            ])
            .sort(true, true)
        })
        .collect::<Vec<_>>();
    join(left, right, spec, JoinType::Full)?
        .filter(different)
        .and_then(|plan| plan.sort(ordering))
        .and_then(|plan| plan.project(columns))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(crate) type Completions = Vec<Arc<pse_catalog::session::CompletedComputation>>;

pub(crate) async fn execute_recorded(
    session: &SnapshotSession,
    plan: LogicalPlan,
    cancel: &CancellationToken,
    completed: &mut Completions,
) -> Result<Vec<RecordBatch>, AuthoringError> {
    let computation = Arc::new(
        session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?,
    );
    let batches = computation
        .batches()
        .iter()
        .map(|batch| batch.batch().clone())
        .collect();
    completed.push(computation);
    Ok(batches)
}

pub(crate) async fn execute(
    session: &SnapshotSession,
    plan: LogicalPlan,
    cancel: &CancellationToken,
) -> Result<Vec<RecordBatch>, AuthoringError> {
    Ok(session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?
        .into_batches())
}
