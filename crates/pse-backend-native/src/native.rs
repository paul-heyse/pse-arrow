// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Composable native Ipopt plans over a one-row, derived numerical input layout.
mod execution;
mod output;

use crate::{NativeError, SolveOptions, Variable, error::invalid};
use datafusion::{
    common::{DFSchema, DFSchemaRef, DataFusionError, ExprSchema, Result},
    logical_expr::{Expr, Extension, LogicalPlan, UserDefinedLogicalNodeCore},
};
pub use execution::NativeSolverPlanner;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

/// One physical constraint expression with case-bound limits and scaling.
#[derive(Clone, Debug)]
pub struct Constraint {
    /// Native scalar mathematical expression in the child's decision/parameter columns.
    pub expression: Expr,
    /// Physical lower bound, including zero for an equality residual.
    pub lower: Option<f64>,
    /// Physical upper bound, including zero for an equality residual.
    pub upper: Option<f64>,
    /// Positive user scaling factor.
    pub scale: f64,
}
#[derive(Clone)]
pub(super) struct Specification {
    pub program_id: pse_ids::SemanticId,
    pub variables: Vec<Variable>,
    pub constraints: Vec<Constraint>,
    pub objective: Expr,
    pub options: SolveOptions,
}

/// Specialized solver node. It owns an immutable derived layout and actual input
/// plan; it retains no compiler, model store or mutable callback workspace.
#[derive(Clone)]
pub struct Solve {
    pub(super) specification: Arc<Specification>,
    pub(super) input: LogicalPlan,
    schema: DFSchemaRef,
}
impl Solve {
    /// Construct a native solver plan. All heavy preparation/solving occurs through
    /// the installed [`NativeSolverPlanner`], and solving occurs only on stream pull.
    /// The objective is minimized; feasibility uses a literal zero objective.
    /// # Errors
    /// Empty/duplicate decision columns or invalid finite bounds/scaling/options.
    pub fn plan(
        program_id: pse_ids::SemanticId,
        input: LogicalPlan,
        variables: Vec<Variable>,
        constraints: Vec<Constraint>,
        objective: Expr,
        options: SolveOptions,
    ) -> Result<LogicalPlan, NativeError> {
        options.validate()?;
        if variables.is_empty() {
            return Err(invalid("a solve requires decision columns"));
        }
        let mut columns = std::collections::BTreeSet::new();
        for variable in &variables {
            if !columns.insert(variable.column.clone()) {
                return Err(invalid("duplicate decision column"));
            }
            let field = input
                .schema()
                .field_from_column(&variable.column)
                .map_err(|e| invalid(e.to_string()))?;
            if field.data_type() != &datafusion::arrow::datatypes::DataType::Float64
                || field.is_nullable()
            {
                return Err(invalid("decision columns require nonnull Float64 fields"));
            }
            validate_bounds(variable.lower, variable.upper, variable.scale)?;
        }
        for constraint in &constraints {
            validate_bounds(constraint.lower, constraint.upper, constraint.scale)?;
        }
        let schema = Arc::new(
            DFSchema::try_from(output::schema()?.as_ref().clone())
                .map_err(|e| invalid(e.to_string()))?,
        );
        Ok(LogicalPlan::Extension(Extension {
            node: Arc::new(Self {
                specification: Arc::new(Specification {
                    program_id,
                    variables,
                    constraints,
                    objective,
                    options,
                }),
                input,
                schema,
            }),
        }))
    }
}
fn validate_bounds(lower: Option<f64>, upper: Option<f64>, scale: f64) -> Result<(), NativeError> {
    if lower
        .into_iter()
        .chain(upper)
        .any(|v| !v.is_finite() || v.abs() >= f64::MAX)
        || lower.zip(upper).is_some_and(|(l, u)| l > u)
        || !scale.is_finite()
        || scale <= 0.0
    {
        return Err(invalid("invalid numerical bounds or scaling"));
    }
    Ok(())
}
impl std::fmt::Debug for Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_for_explain(f)
    }
}
impl PartialEq for Solve {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.specification, &other.specification) && self.input == other.input
    }
}
impl Eq for Solve {}
impl Hash for Solve {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.specification).hash(state);
        self.input.hash(state);
    }
}
impl PartialOrd for Solve {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.specification).cmp(&Arc::as_ptr(&other.specification)) {
            std::cmp::Ordering::Equal => self.input.partial_cmp(&other.input),
            order => Some(order),
        }
    }
}
impl UserDefinedLogicalNodeCore for Solve {
    fn name(&self) -> &'static str {
        "PseSolve"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![&self.input]
    }
    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }
    fn expressions(&self) -> Vec<Expr> {
        std::iter::once(self.specification.objective.clone())
            .chain(
                self.specification
                    .constraints
                    .iter()
                    .map(|constraint| constraint.expression.clone()),
            )
            .collect()
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PseSolve: program={}, variables={}, constraints={}, hessian={:?}, max_iterations={}",
            self.specification.program_id.to_hex(),
            self.specification.variables.len(),
            self.specification.constraints.len(),
            self.specification.options.hessian,
            self.specification.options.max_iterations
        )
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        let [input] = inputs.as_slice() else {
            return Err(DataFusionError::Plan("solve requires one input".into()));
        };
        if expressions.len() != self.specification.constraints.len() + 1
            || input.schema() != self.input.schema()
        {
            return Err(DataFusionError::Plan(
                "solve numerical layout changed".into(),
            ));
        }
        let mut specification = self.specification.as_ref().clone();
        specification.objective = expressions[0].clone();
        for (constraint, expression) in specification.constraints.iter_mut().zip(&expressions[1..])
        {
            constraint.expression = expression.clone();
        }
        Ok(Self {
            specification: Arc::new(specification),
            input: input.clone(),
            schema: Arc::clone(&self.schema),
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|field| field.name().to_owned())
            .collect()
    }
}
