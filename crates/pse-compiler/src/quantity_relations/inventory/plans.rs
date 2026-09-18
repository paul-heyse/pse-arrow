// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact unit inventory reconciliation is a native relational computation.

use super::{CompilerError, invalid};
use datafusion::{
    functions_aggregate::expr_fn::{count, count_distinct},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{SnapshotSession, output::declare_relation_output, scalar};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

#[expect(
    clippy::too_many_lines,
    reason = "units keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn units(
    session: &SnapshotSession,
    registry: &Registry,
    batches: &mut BTreeMap<RelationKey, FieldCheckedBatch>,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    let Some(normalized) = registry
        .relation("normalized.units")
        .filter(|spec| batches.contains_key(&spec.key))
    else {
        return Ok(());
    };
    let reference = registry
        .relation("reference.units")
        .ok_or_else(|| invalid("normalized units have no authoritative physical declaration"))?;
    let project = |key: &RelationKey| -> Result<LogicalPlan, CompilerError> {
        let scan = LogicalPlanBuilder::scan(
            session.table_reference(key)?,
            session.table_source(key)?,
            None,
        )
        .map_err(engine)?
        .project(reference.columns.iter().map(|column| col(column.name())))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
        declare_relation_output(scan, registry, reference).map_err(engine)
    };
    let mut inputs = vec![project(&normalized.key)?];
    if batches.contains_key(&reference.key) {
        inputs.push(project(&reference.key)?);
    }
    let mut failures = Vec::new();
    for input in &inputs {
        failures.push(
            LogicalPlanBuilder::from(input.clone())
                .aggregate([col("unit_id")], [count(lit(1_i64)).alias("__count")])
                .map_err(engine)?
                .filter(col("__count").gt(lit(1_i64)))
                .map_err(engine)?
                .project([col("unit_id")])
                .map_err(engine)?
                .build()
                .map_err(engine)?,
        );
    }
    let mut input = inputs.remove(0);
    for next in inputs {
        input = LogicalPlanBuilder::from(input)
            .union(next)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
    }
    let definitions = scalar::key(
        reference.id,
        reference
            .columns
            .iter()
            .map(|column| {
                (
                    column.name(),
                    Expr::Column(datafusion::common::Column::from_name(column.name())),
                )
            })
            .collect(),
    );
    let conflict = LogicalPlanBuilder::from(input.clone())
        .project([col("unit_id"), definitions.alias("__definition")])
        .map_err(engine)?
        .aggregate(
            [col("unit_id")],
            [count_distinct(col("__definition")).alias("__count")],
        )
        .map_err(engine)?
        .filter(col("__count").gt(lit(1_i64)))
        .map_err(engine)?
        .project([col("unit_id")])
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let mut conflict = conflict;
    for failure in failures {
        conflict = LogicalPlanBuilder::from(conflict)
            .union(failure)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
    }
    let conflicts = session
        .prepare_rule_plan(conflict, cancel)?
        .execute(cancel)
        .await?;
    if let Some(batch) = conflicts
        .batches()
        .iter()
        .find(|batch| batch.num_rows() != 0)
    {
        return Err(pse_quantity::QuantityError::Registry {
            rule: "unit_inventory.complete_definition",
            subject: identity(batch, 0, 0)?,
            detail: "native unit inventory found duplicate source keys or incompatible exact definitions".to_owned(),
        }.into());
    }
    let plan = LogicalPlanBuilder::from(input)
        .distinct()
        .map_err(engine)?
        .sort([col("unit_id").sort(true, false)])
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let plan = declare_relation_output(plan, registry, reference).map_err(engine)?;
    let complete = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?
        .into_checked_relation(registry, reference, cancel)?;
    batches.insert(reference.key, complete);
    Ok(())
}

pub(super) async fn context(
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Option<(SemanticId, SemanticId)>, CompilerError> {
    let Some(spec) = registry
        .relation("reference.math_context")
        .filter(|spec| session.table_provider(&spec.key).is_some())
    else {
        return Ok(None);
    };
    let plan = LogicalPlanBuilder::scan(
        session.table_reference(&spec.key)?,
        session.table_source(&spec.key)?,
        None,
    )
    .and_then(|plan| plan.project([col("neutral_quantity_type_id"), col("boolean_kind_id")]))
    .and_then(LogicalPlanBuilder::distinct)
    .and_then(LogicalPlanBuilder::build)
    .map_err(engine)?;
    let complete = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?;
    let mut selected = None;
    for batch in complete.batches() {
        for row in 0..batch.num_rows() {
            if selected.is_some() {
                return Err(invalid(
                    "selected packages have conflicting explicit mathematical contexts",
                ));
            }
            selected = Some((identity(batch, 0, row)?, identity(batch, 1, row)?));
        }
    }
    Ok(selected)
}

fn identity(
    batch: &pse_relations::RecordBatch,
    column: usize,
    row: usize,
) -> Result<SemanticId, CompilerError> {
    use datafusion::arrow::array::FixedSizeBinaryArray;
    let values = batch
        .column(column)
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .ok_or_else(|| invalid("native physical identity output has wrong storage"))?;
    Ok(SemanticId::from_bytes(
        values
            .value(row)
            .try_into()
            .map_err(|_| invalid("native physical identity output has wrong width"))?,
    ))
}

pub(super) fn engine(error: datafusion::common::DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}
