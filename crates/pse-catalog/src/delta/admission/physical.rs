// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Execute the exact selected physical closure before binding quantity obligations.
use datafusion::{
    arrow::array::RecordBatch, common::Result, execution::session_state::SessionState,
    logical_expr::LogicalPlan, physical_plan::collect,
};
use pse_columnar::CancellationToken;
use pse_relations::{
    columnar::FieldCheckedBatch,
    validate::{
        ValidationContext,
        obligations::{QuantityCompatibility, RelationInputs, quantity_requests},
    },
};
use pse_schema::Registry;
use std::collections::BTreeMap;

pub(super) async fn compatibility(
    registry: &Registry,
    inputs: &RelationInputs,
    state: &SessionState,
) -> Result<Option<QuantityCompatibility>> {
    let Some(request) = quantity_requests(inputs)? else {
        return Ok(None);
    };
    let requests = execute(request, state).await?;
    if requests.iter().all(|batch| batch.num_rows() == 0) {
        return Ok(None);
    }
    let cancel = CancellationToken::new();
    let validation = ValidationContext::new(
        registry,
        pse_engine::validation::NativeValidation(state.clone()),
    );
    let mut selected = BTreeMap::new();
    let reconciliation = pse_relations::physical::reconcile_units(inputs, registry)?;
    if let Some(plans) = &reconciliation {
        if execute(plans.conflicts.clone(), state)
            .await?
            .iter()
            .any(|batch| batch.num_rows() != 0)
        {
            return Err(super::invalid(
                "selected physical unit definitions conflict or repeat a source key",
            ));
        }
    }
    for name in pse_relations::physical::VALUE_INPUTS {
        if *name == "normalized.units" {
            continue;
        }
        let Some(spec) = registry.relation(name) else {
            continue;
        };
        let input = if *name == "reference.units" {
            reconciliation
                .as_ref()
                .map(|plans| &plans.merged)
                .or_else(|| inputs.get(&spec.id))
        } else {
            inputs.get(&spec.id)
        };
        let Some(input) = input else {
            continue;
        };
        let declared =
            pse_engine::session::output::declare_relation_output(input.clone(), registry, spec)?;
        let batches = execute(declared, state)
            .await?
            .into_iter()
            .map(|batch| {
                FieldCheckedBatch::admit_in(registry, spec, batch, &validation, &cancel)
                    .map_err(pse_columnar::external)
            })
            .collect::<Result<Vec<_>>>()?;
        selected.insert(
            spec.key,
            FieldCheckedBatch::concat(registry, spec, &batches).map_err(pse_columnar::external)?,
        );
    }
    QuantityCompatibility::admit(registry, inputs, &selected, &requests, &cancel).map(Some)
}
async fn execute(plan: LogicalPlan, state: &SessionState) -> Result<Vec<RecordBatch>> {
    let plan = state.optimize(&plan)?;
    collect(state.create_physical_plan(&plan).await?, state.task_ctx()).await
}
