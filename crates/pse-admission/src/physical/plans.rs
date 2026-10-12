// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact unit inventory reconciliation is a native relational computation.

use super::{PhysicalError, invalid};
use datafusion::logical_expr::{LogicalPlanBuilder, col};
use pse_columnar::CancellationToken;
use pse_engine::session::{EngineSession, output::declare_relation_output};
use pse_ids::SemanticId;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

/// Execute shared reconciliation against the selected runtime sources.
pub(super) async fn units(
    session: &EngineSession,
    registry: &Registry,
    batches: &mut BTreeMap<RelationKey, FieldCheckedBatch>,
    cancel: &CancellationToken,
) -> Result<(), PhysicalError> {
    let mut inputs = pse_relations::validate::obligations::RelationInputs::new();
    for name in ["reference.units", "normalized.units"] {
        if let Some(spec) = registry
            .relation(name)
            .filter(|spec| batches.contains_key(&spec.key))
        {
            let plan = LogicalPlanBuilder::scan(
                session.table_reference(&spec.key)?,
                session.table_source(&spec.key)?,
                None,
            )
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
            inputs.insert(spec.id, plan);
        }
    }
    let Some(plans) =
        pse_relations::physical::reconcile_units(&inputs, registry).map_err(engine)?
    else {
        return Ok(());
    };
    let conflicts = session
        .prepare_rule_plan(plans.conflicts, cancel)?
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
            detail: "native unit inventory found duplicate source keys or incompatible exact definitions".into(),
        }.into());
    }
    let reference = registry
        .relation("reference.units")
        .ok_or_else(|| invalid("physical unit declaration disappeared"))?;
    let plan = declare_relation_output(plans.merged, registry, reference).map_err(engine)?;
    let complete = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?
        .into_checked_relation(registry, reference, cancel)?;
    batches.insert(reference.key, complete);
    Ok(())
}

pub(super) async fn context(
    session: &EngineSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Option<(SemanticId, SemanticId)>, PhysicalError> {
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
) -> Result<SemanticId, PhysicalError> {
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

pub(super) fn engine(error: datafusion::common::DataFusionError) -> PhysicalError {
    pse_engine::EngineError::from(error).into()
}
