// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Iteration, Request};
use crate::{NativeError, error::invalid};
use datafusion::arrow::array::{Array, Float64Array, ListArray, RecordBatch};
use pse_ids::{CancellationToken, ReservationLease};
use pse_numerics::EvaluationProgram;
use std::sync::Arc;

pub(super) struct Workspace {
    pub program: Arc<EvaluationProgram>,
    pub input: RecordBatch,
    pub positions: Vec<usize>,
    pub initial: Vec<f64>,
    pub jacobian: Vec<(usize, i32, i32)>,
    pub objective_gradient: Vec<(usize, usize)>,
    pub cancel: CancellationToken,
    pub failure: Option<NativeError>,
    pub iterations: Vec<Iteration>,
    pub iteration_capacity: usize,
    pub allocation: Arc<ReservationLease>,
    previous: Vec<u64>,
    residuals: Float64Array,
    derivatives: Float64Array,
}
impl Workspace {
    pub(super) fn new(request: &Request) -> Result<Self, NativeError> {
        let schema = request.program.input_schema();
        if request.variables.is_empty()
            || request.input.num_rows() != 1
            || request.input.schema() != *schema
            || request.program.residual_count() != request.constraints.len() + 1
            || !request
                .variables
                .iter()
                .map(|variable| variable.column.flat_name())
                .eq(request.program.contract().variable_columns.iter().cloned())
        {
            return Err(invalid(
                "solver needs one exact input row, decision columns, objective and constraint expressions",
            ));
        }
        let iteration_capacity = usize::try_from(request.options.max_iterations)
            .ok()
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| invalid("iteration capacity"))?;
        let extent = request
            .variables
            .len()
            .checked_add(request.constraints.len())
            .and_then(|n| n.checked_add(request.program.jacobian_coordinates().len()))
            .and_then(|n| n.checked_mul(256))
            .and_then(|n| n.checked_add(iteration_capacity.checked_mul(size_of::<Iteration>())?))
            .and_then(|n| n.checked_add(request.foreign_bytes))
            .and_then(|n| n.checked_add(<pse_relations::generated::runtime::solver_outcomes::Row as pse_relations::columnar::RelationRow>::builder_allocation_size()))
            .and_then(|n| n.checked_add(request.input.get_array_memory_size()))
            .ok_or_else(|| invalid("solver allocation extent overflows"))?;
        if request.foreign_bytes == 0 {
            return Err(invalid("foreign solver allocation allowance is required"));
        }
        let mut reservation = request
            .reserver
            .open("solve:ipopt-workspace-and-foreign-allowance");
        reservation
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        let mut positions = Vec::new();
        let mut initial = Vec::new();
        for variable in &request.variables {
            let index = schema
                .index_of(&variable.column.name())
                .map_err(|e| invalid(e.to_string()))?;
            if positions.contains(&index) {
                return Err(invalid("duplicate decision column"));
            }
            let values = request
                .input
                .column(index)
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| invalid("decision columns must be Float64"))?;
            if values.is_null(0) || !values.value(0).is_finite() {
                return Err(invalid("initial guesses must be finite and nonnull"));
            }
            positions.push(index);
            initial.push(values.value(0));
        }
        let mut jacobian = Vec::new();
        let mut objective_gradient = Vec::new();
        for (offset, coordinate) in request.program.jacobian_coordinates().iter().enumerate() {
            let row =
                usize::try_from(coordinate.residual).map_err(|_| invalid("Jacobian row range"))?;
            let column = usize::try_from(coordinate.variable)
                .map_err(|_| invalid("Jacobian column range"))?;
            if coordinate.program_id != request.program.contract().program_id
                || usize::try_from(coordinate.ordinal).ok() != Some(offset)
                || row >= request.program.residual_count()
            {
                return Err(invalid(
                    "coordinate differs from its exact program or vector order",
                ));
            }
            if column >= positions.len() {
                return Err(invalid("derivative variable order differs from problem"));
            }
            let result_column = offset;
            if row == 0 {
                objective_gradient.push((result_column, column));
            } else {
                jacobian.push((
                    result_column,
                    i32::try_from(row - 1).map_err(|_| invalid("Jacobian row range"))?,
                    i32::try_from(column).map_err(|_| invalid("Jacobian column range"))?,
                ));
            }
        }
        Ok(Self {
            program: Arc::clone(&request.program),
            input: request.input.clone(),
            positions,
            initial,
            jacobian,
            objective_gradient,
            cancel: request.cancel.clone(),
            failure: None,
            iterations: Vec::with_capacity(iteration_capacity),
            iteration_capacity,
            allocation: ReservationLease::new(reservation),
            previous: Vec::new(),
            residuals: Float64Array::from(Vec::<f64>::new()),
            derivatives: Float64Array::from(Vec::<f64>::new()),
        })
    }
    pub(super) fn evaluate(&mut self, values: &[f64]) -> Result<(), NativeError> {
        self.cancel.checkpoint()?;
        if values.len() != self.positions.len() || values.iter().any(|value| !value.is_finite()) {
            return Err(invalid("solver supplied invalid decision values"));
        }
        if self.previous.len() == values.len()
            && self
                .previous
                .iter()
                .zip(values)
                .all(|(bits, value)| *bits == value.to_bits())
        {
            return Ok(());
        }
        let mut columns = self.input.columns().to_vec();
        for (&position, &value) in self.positions.iter().zip(values) {
            columns[position] = Arc::new(Float64Array::from(vec![value]));
        }
        let input = RecordBatch::try_new(self.input.schema(), columns)
            .map_err(|e| invalid(e.to_string()))?;
        let result = self.program.evaluate(&input)?;
        let residuals = vector(&result, "residuals", self.program.residual_count())?;
        let derivatives = vector(
            &result,
            "jacobian",
            self.program.jacobian_coordinates().len(),
        )?;
        self.previous.clear();
        self.previous
            .extend(values.iter().map(|value| value.to_bits()));
        self.residuals = residuals;
        self.derivatives = derivatives;
        Ok(())
    }
    pub(super) fn residual(&self, index: usize) -> f64 {
        self.residuals.values()[index]
    }
    pub(super) fn derivative(&self, index: usize) -> f64 {
        self.derivatives.values()[index]
    }
}

fn vector(batch: &RecordBatch, name: &str, length: usize) -> Result<Float64Array, NativeError> {
    let list = batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<ListArray>())
        .ok_or_else(|| invalid("numerical result vector is not a declared List"))?;
    if list.len() != 1 || list.is_null(0) {
        return Err(invalid("callback requires one visible numerical vector"));
    }
    let values = list.value(0);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| invalid("numerical vector is not Float64"))?;
    if values.len() != length || values.null_count() != 0 {
        return Err(invalid(
            "numerical vector extent or visibility differs from program",
        ));
    }
    Ok(values.clone())
}
