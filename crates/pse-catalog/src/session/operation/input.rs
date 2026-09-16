// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An operation's independently scoped native input, without a relational cross join.

use datafusion::{
    common::{DFSchema, DFSchemaRef, DataFusionError, Result},
    logical_expr::{Expr, Extension, LogicalPlan, UserDefinedLogicalNodeCore},
};
use std::sync::{Arc, LazyLock};

static NO_COLUMNS: LazyLock<DFSchemaRef> = LazyLock::new(|| Arc::new(DFSchema::empty()));

/// Full source fields stay on the actual child plan. They are consumed under that
/// input's own scope by the operation, not flattened into a common expression scope.
/// This also avoids quadratic schema merging for operations with many independent ports.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
struct OperationInput {
    input: LogicalPlan,
}
impl UserDefinedLogicalNodeCore for OperationInput {
    fn name(&self) -> &'static str {
        "OperationInput"
    }
    fn schema(&self) -> &DFSchemaRef {
        &NO_COLUMNS
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![&self.input]
    }
    fn expressions(&self) -> Vec<Expr> {
        Vec::new()
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OperationInput: independently scoped source")
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        mut inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if !expressions.is_empty() || inputs.len() != 1 {
            return Err(DataFusionError::Plan(
                "operation input requires its one captured source".into(),
            ));
        }
        let input = inputs
            .pop()
            .ok_or_else(|| DataFusionError::Plan("operation input absent".into()))?;
        Ok(Self { input })
    }
}

pub(super) fn scoped(input: LogicalPlan) -> LogicalPlan {
    LogicalPlan::Extension(Extension {
        node: Arc::new(OperationInput { input }),
    })
}
