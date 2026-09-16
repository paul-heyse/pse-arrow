// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native keyed change plans. Explicit side-presence is independent of payload nulls.

use super::{contract, exact};
use crate::AuthoringError;
use datafusion::{
    arrow::{
        array::{Array, BooleanArray, RecordBatch},
        compute::{concat_batches, filter_record_batch},
    },
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

pub(super) fn project(spec: &RelationSpec, alias: &str) -> Vec<Expr> {
    spec.columns
        .iter()
        .map(|column| qualified(alias, column.name()).alias(column.name()))
        .collect()
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

pub(super) async fn relation(
    session: &SnapshotSession,
    plan: LogicalPlan,
    spec: &RelationSpec,
    cancel: &CancellationToken,
    completed: &mut Completions,
) -> Result<FieldCheckedBatch, AuthoringError> {
    let plan =
        pse_catalog::session::output::declare_relation_output(plan, session.registry(), spec)
            .map_err(engine)?;
    let values = execute_recorded(session, plan, cancel, completed).await?;
    let mut work = session.reserver().open("authoring:relation-result");
    work.try_grow(values.iter().try_fold(0_usize, |sum, batch| {
        crate::work::add(sum, pse_ids::validation_extent(batch)?)
    })?)?;
    let schema = Arc::new(
        pse_schema::arrow::relation_schema(session.registry(), spec)
            .map_err(pse_relations::RelationError::from)?,
    );
    let batches = values
        .into_iter()
        .map(|batch| RecordBatch::try_new(Arc::clone(&schema), batch.columns().to_vec()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| engine(error.into()))?;
    let batch = concat_batches(&schema, &batches).map_err(|error| engine(error.into()))?;
    Ok(FieldCheckedBatch::admit(session.registry(), spec, batch)?
        .retained(session.reserver(), cancel)?)
}

pub(super) fn presence(batch: &RecordBatch, index: usize) -> Result<&BooleanArray, AuthoringError> {
    batch
        .column(index)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| contract("native change presence is not Boolean"))
}

pub(super) fn side(
    batch: &RecordBatch,
    present: &BooleanArray,
    offset: usize,
    spec: &RelationSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, AuthoringError> {
    let mut work = session.reserver().open("authoring:change-side");
    work.try_grow(pse_ids::validation_extent(batch)?)?;
    let mask = BooleanArray::from(
        ((0..present.len()).map(|row| present.is_valid(row) && present.value(row)))
            .collect::<Vec<_>>(),
    );
    let filtered = filter_record_batch(batch, &mask).map_err(|error| engine(error.into()))?;
    let fields = Arc::new(
        pse_schema::arrow::relation_schema(session.registry(), spec)
            .map_err(pse_relations::RelationError::from)?,
    );
    let columns = filtered.columns()[offset..offset + spec.columns.len()].to_vec();
    let batch = RecordBatch::try_new(fields, columns).map_err(|error| engine(error.into()))?;
    Ok(FieldCheckedBatch::admit(session.registry(), spec, batch)?
        .retained(session.reserver(), cancel)?)
}
