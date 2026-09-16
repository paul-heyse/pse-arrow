// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reusable native physical expressions for residuals and sparse Jacobians.
use crate::{NumericsError, differentiate, error::input, finite};
use datafusion::{
    arrow::{
        array::{Array, RecordBatch},
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    common::{
        Column, DFSchema, ExprSchema,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion},
    },
    execution::session_state::SessionState,
    logical_expr::{Expr, ExprSchemable, LogicalPlan, LogicalPlanBuilder, ScalarUDF},
    physical_expr::PhysicalExpr,
};
use pse_ids::{CancellationToken, MemoryReserver, ReservationLease};
use std::{collections::BTreeMap, sync::Arc};

/// Derived numerical layout over native expressions; contains no model store or graph.
/// It supports strict finite evaluation and an exact sparse first derivative. A
/// Hessian is not advertised; solver consumers must select that capability explicitly.
#[derive(Debug)]
pub struct EvaluationProgram {
    schema: SchemaRef,
    output: SchemaRef,
    logical: Vec<Expr>,
    physical: Vec<Arc<dyn PhysicalExpr>>,
    residuals: usize,
    coordinates: Vec<(usize, usize)>,
    nodes: usize,
    cancel: CancellationToken,
    reserver: Arc<dyn MemoryReserver>,
    _allocation: Arc<ReservationLease>,
}
impl EvaluationProgram {
    /// Compile residuals and derivatives once under the actual caller state.
    /// Variables are exact native columns in solver order. Other columns are fixed
    /// parameters; the same program evaluates successive Arrow batches/callback values.
    /// # Errors
    /// Invalid/null input contracts, unsupported derivative bindings, cancellation,
    /// resource exhaustion or native expression preparation failure.
    pub fn compile(
        residuals: &[Expr],
        variables: &[Column],
        schema: SchemaRef,
        state: &SessionState,
        reserver: Arc<dyn MemoryReserver>,
        cancel: CancellationToken,
    ) -> Result<Self, NumericsError> {
        cancel.checkpoint()?;
        let native =
            DFSchema::try_from(schema.as_ref().clone()).map_err(NumericsError::preparation)?;
        let variables = variable_map(variables, &native)?;
        if schema.fields().iter().any(|field| field.is_nullable()) {
            return Err(input("numerical input fields must be nonnullable"));
        }
        let count = expression_count(residuals)?;
        // Symbolic products duplicate operand trees. Charge a conservative quadratic
        // preparation bound before constructing them, against the caller's budget.
        let extent = count
            .checked_add(1)
            .and_then(|n| n.checked_mul(n))
            .and_then(|n| n.checked_mul(variables.len().max(1)))
            .and_then(|n| n.checked_mul(size_of::<Expr>() + 256))
            .ok_or_else(|| input("numerical preparation extent overflows"))?;
        let mut allocation = reserver.open("numerics:native-expression-preparation");
        allocation
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        let allocation = ReservationLease::new(allocation);
        let guard = finite::function(cancel.clone(), Arc::clone(&allocation));
        let mut values = vec![];
        let mut gradients = vec![];
        let mut coordinates = vec![];
        for (row, expression) in residuals.iter().enumerate() {
            cancel.checkpoint()?;
            if expression
                .get_type(&native)
                .map_err(NumericsError::preparation)?
                != DataType::Float64
            {
                return Err(input("residual expressions must produce Float64"));
            }
            let jet = differentiate::compile(expression, &variables, 0)?;
            values.push(instrument(jet.value, &native, &guard)?.alias(format!("residual_{row}")));
            for (column, derivative) in jet.gradient {
                coordinates.push((row, column));
                gradients.push(
                    instrument(derivative, &native, &guard)?
                        .alias(format!("jacobian_{row}_{column}")),
                );
            }
        }
        values.extend(gradients);
        let nodes = expression_count(&values)?;
        let physical = values
            .iter()
            .map(|expression| {
                state
                    .create_physical_expr(expression.clone(), &native)
                    .map_err(NumericsError::preparation)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let output = Arc::new(Schema::new(
            values
                .iter()
                .map(|expression| {
                    Field::new(
                        expression.schema_name().to_string(),
                        DataType::Float64,
                        false,
                    )
                })
                .collect::<Vec<_>>(),
        ));
        Ok(Self {
            schema,
            output,
            logical: values,
            physical,
            residuals: residuals.len(),
            coordinates,
            nodes,
            cancel,
            reserver,
            _allocation: allocation,
        })
    }
    /// Native expressions retain their actual functions, typing and strict assertions.
    pub fn expressions(&self) -> &[Expr] {
        &self.logical
    }
    /// Exact Arrow input layout retained by the prepared physical expressions.
    pub fn input_schema(&self) -> &SchemaRef {
        &self.schema
    }
    /// Actual prepared expressions for native execution-plan introspection.
    pub fn physical_expressions(&self) -> &[Arc<dyn PhysicalExpr>] {
        &self.physical
    }
    /// Ordinary native projection for inspection, batch evaluation or result sinks.
    /// Expressions and their reservation owners survive this program handle.
    /// # Errors
    /// The native child differs from the compiled input layout.
    pub fn logical_projection(&self, child: LogicalPlan) -> Result<LogicalPlan, NumericsError> {
        if child.schema().as_arrow() != self.schema.as_ref() {
            return Err(input(
                "numerical projection child differs from compiled input schema",
            ));
        }
        LogicalPlanBuilder::from(child)
            .project(self.logical.clone())
            .and_then(LogicalPlanBuilder::build)
            .map_err(NumericsError::preparation)
    }
    /// Number of leading output columns containing residuals in requested order.
    pub const fn residual_count(&self) -> usize {
        self.residuals
    }
    /// Sparse Jacobian columns following residuals, in `(residual, variable)` order.
    pub fn jacobian_coordinates(&self) -> &[(usize, usize)] {
        &self.coordinates
    }
    /// Evaluate new values without constructing/planning a query. Batches may contain
    /// multiple scenarios; output buffers retain their own allocation reservation.
    /// # Errors
    /// Input mismatch, null/nonfinite values, domain/derivative error, resource refusal
    /// or cancellation. No partial result is returned.
    pub fn evaluate(&self, values: &RecordBatch) -> Result<RecordBatch, NumericsError> {
        self.cancel.checkpoint()?;
        if values.schema() != self.schema
            || values
                .columns()
                .iter()
                .any(|column| column.null_count() != 0)
        {
            return Err(input(
                "evaluation batch differs from its exact nonnull input schema",
            ));
        }
        let extent = values
            .num_rows()
            .checked_mul(self.nodes.max(1))
            .and_then(|n| n.checked_mul(64))
            .and_then(|n| n.checked_add(values.get_array_memory_size()))
            .ok_or_else(|| input("evaluation allocation extent overflows"))?;
        let mut reservation = self.reserver.open("numerics:native-expression-evaluation");
        reservation
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        let mut columns = vec![];
        for expression in &self.physical {
            self.cancel.checkpoint()?;
            let result = expression
                .evaluate(values)
                .and_then(|value| value.into_array(values.num_rows()));
            self.cancel.checkpoint()?;
            columns.push(result.map_err(NumericsError::evaluation)?);
        }
        let output = RecordBatch::try_new_with_options(
            Arc::clone(&self.output),
            columns,
            &datafusion::arrow::array::RecordBatchOptions::new()
                .with_row_count(Some(values.num_rows())),
        )
        .map_err(|error| NumericsError::evaluation(error.into()))?;
        // Intermediate native arrays have been dropped. Keep the returned arrays'
        // actual extent charged until their last consumer releases them.
        reservation.shrink(extent.saturating_sub(output.get_array_memory_size()));
        Ok(pse_ids::owned_buffer::attach_reservation(
            output,
            ReservationLease::new(reservation),
        )?)
    }
}
fn variable_map(
    variables: &[Column],
    schema: &DFSchema,
) -> Result<BTreeMap<Column, usize>, NumericsError> {
    let mut result = BTreeMap::new();
    for (index, variable) in variables.iter().enumerate() {
        let field = schema
            .field_from_column(variable)
            .map_err(NumericsError::preparation)?;
        if field.data_type() != &DataType::Float64
            || result.insert(variable.clone(), index).is_some()
        {
            return Err(input("variables must be distinct Float64 columns"));
        }
    }
    Ok(result)
}
pub(crate) fn expression_count(expressions: &[Expr]) -> Result<usize, NumericsError> {
    let mut count = 0usize;
    for expression in expressions {
        count_expression(expression, 0, &mut count).map_err(NumericsError::preparation)?;
    }
    Ok(count)
}
fn count_expression(
    expression: &Expr,
    depth: usize,
    count: &mut usize,
) -> datafusion::common::Result<()> {
    if depth > 256 || *count == 1_000_000 {
        return Err(datafusion::common::DataFusionError::ResourcesExhausted(
            "numerical expression exceeds depth 256 or one million native nodes".into(),
        ));
    }
    *count += 1;
    expression.apply_children(|child| {
        count_expression(child, depth + 1, count)?;
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(())
}
fn instrument(
    expression: Expr,
    schema: &DFSchema,
    guard: &ScalarUDF,
) -> Result<Expr, NumericsError> {
    expression
        .transform_up(|node| {
            if node.get_type(schema)? == DataType::Float64 && !matches!(node, Expr::Alias(_)) {
                Ok(Transformed::yes(finite::checked(guard, node)))
            } else {
                Ok(Transformed::no(node))
            }
        })
        .map(|value| value.data)
        .map_err(NumericsError::preparation)
}
