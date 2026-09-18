// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated solver outcome contract; values follow its exact program ordering.
use datafusion::arrow::datatypes::SchemaRef;

pub(super) fn schema() -> Result<SchemaRef, crate::NativeError> {
    Ok(pse_relations::generated::runtime::solver_outcomes::schema()?)
}

#[cfg(feature = "ipopt")]
pub(super) fn pack(
    outcome: crate::driver::Outcome,
) -> datafusion::common::Result<datafusion::arrow::array::RecordBatch> {
    use pse_relations::generated::{enums::SolverTermination, runtime::solver_outcomes as rows};
    let external = |error| datafusion::common::DataFusionError::External(Box::new(error));
    let termination = if outcome.cancelled {
        SolverTermination::Cancelled
    } else if outcome.failure.is_some() {
        SolverTermination::EvaluationFailure
    } else if matches!(outcome.status, 0 | 1 | 6) {
        SolverTermination::Success
    } else {
        SolverTermination::Stopped
    };
    if termination == SolverTermination::Success
        && (!outcome.objective.is_finite()
            || outcome
                .values
                .iter()
                .chain(&outcome.constraints)
                .chain(&outcome.constraint_duals)
                .chain(&outcome.lower_duals)
                .chain(&outcome.upper_duals)
                .any(|v| !v.is_finite()))
    {
        return Err(datafusion::common::DataFusionError::Execution(
            "Ipopt reported success with unavailable numeric values".into(),
        ));
    }
    let optional = |value: f64| value.is_finite().then_some(value);
    let diagnostic =
        outcome
            .failure
            .as_ref()
            .map(|failure| rows::RuntimeSolverOutcomesFieldDiagnostic {
                code: miette::Diagnostic::code(failure).map(|code| code.to_string()),
                message: failure.to_string(),
            });
    let mut builder = rows::Builder::new().map_err(external)?;
    builder
        .push(rows::Row {
            program_id: outcome.program.program_id,
            scenario_id: outcome.scenario_id,
            variable_columns: outcome.program.variable_columns,
            constraint_dimension: outcome.program.residual_dimension - 1,
            ipopt_status: i64::from(outcome.status),
            termination,
            objective: optional(outcome.objective),
            values: outcome.values.into_iter().map(optional).collect(),
            constraints: outcome.constraints.into_iter().map(optional).collect(),
            constraint_duals: outcome.constraint_duals.into_iter().map(optional).collect(),
            lower_duals: outcome.lower_duals.into_iter().map(optional).collect(),
            upper_duals: outcome.upper_duals.into_iter().map(optional).collect(),
            diagnostic,
            iterations: outcome
                .iterations
                .into_iter()
                .map(|iteration| rows::RuntimeSolverOutcomesFieldIterationsItem {
                    iteration: i64::from(iteration.index),
                    restoration: iteration.restoration,
                    objective: optional(iteration.objective),
                    primal_infeasibility: optional(iteration.primal_infeasibility),
                    dual_infeasibility: optional(iteration.dual_infeasibility),
                    barrier: optional(iteration.barrier),
                    step: optional(iteration.step),
                })
                .collect(),
        })
        .map_err(external)?;
    pse_ids::owned_buffer::attach_reservation(
        builder.finish().map_err(external)?.into_batch(),
        outcome.allocation,
    )
    .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))
}
