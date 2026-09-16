// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An explicit native checked kernel for the pinned Filter schema's missing refinement.
//! A successful result contains the original Arrow values and cannot contain NULL.
//! This is not a caller assertion: the operation itself owns the residual condition.

use datafusion::{
    arrow::datatypes::{DataType, FieldRef},
    common::{Column, DataFusionError, Result},
    logical_expr::{
        ColumnarValue, Expr, LogicalPlan, LogicalPlanBuilder, Operator, ReturnFieldArgs,
        ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature, Volatility, expr::Alias,
    },
};
use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
};

static FUNCTION: LazyLock<Arc<ScalarUDF>> = LazyLock::new(|| {
    Arc::new(ScalarUDF::from(RequireNonNull {
        signature: Signature::any(1, Volatility::Immutable),
        require: true,
    }))
});
static NULLABLE: LazyLock<Arc<ScalarUDF>> = LazyLock::new(|| {
    Arc::new(ScalarUDF::from(RequireNonNull {
        signature: Signature::any(1, Volatility::Immutable),
        require: false,
    }))
});
pub(super) fn nullable_function() -> Arc<ScalarUDF> {
    Arc::clone(&NULLABLE)
}
/// Widen the field's nullable promise as an actual native identity operation.
/// The input's values and semantic field meaning are retained without a value scan.
pub(crate) fn nullable(value: Expr) -> Expr {
    NULLABLE.call(vec![value])
}

pub(super) fn function() -> Arc<ScalarUDF> {
    Arc::clone(&FUNCTION)
}

/// Check the nonnull output obligation while retaining the input field and values.
pub fn require_nonnull(value: Expr) -> Expr {
    FUNCTION.call(vec![value])
}

/// Preserve supported filter refinements as actual native expressions. DataFusion 55
/// gives Filter its unchanged input schema, so an explicit checked identity supplies
/// the missing nonnull output field. The native filter still selects all rows; this
/// operation never changes values or supplies replacement defaults.
///
/// # Errors
/// Invalid field resolution or native projection construction.
pub fn refine_filtered_fields(plan: LogicalPlan) -> Result<LogicalPlan> {
    let LogicalPlan::Filter(filter) = &plan else {
        return Ok(plan);
    };
    let mut required = HashSet::new();
    nonnull_columns(&filter.predicate, &mut required);
    let mut changed = false;
    let expressions = plan
        .schema()
        .iter()
        .map(|(qualifier, field)| {
            let column = Column::new(qualifier.cloned(), field.name());
            if field.is_nullable()
                && required.iter().any(|required| {
                    plan.schema().index_of_column(required).ok()
                        == plan.schema().index_of_column(&column).ok()
                })
            {
                changed = true;
                Expr::Alias(Alias::new(
                    FUNCTION.call(vec![Expr::Column(column)]),
                    qualifier.cloned(),
                    field.name(),
                ))
            } else {
                Expr::Column(column)
            }
        })
        .collect::<Vec<_>>();
    if changed {
        LogicalPlanBuilder::from(plan).project(expressions)?.build()
    } else {
        Ok(plan)
    }
}

fn nonnull_columns(predicate: &Expr, columns: &mut HashSet<Column>) {
    match predicate {
        Expr::IsNotNull(value) => {
            if let Expr::Column(column) = value.as_ref() {
                columns.insert(column.clone());
            }
        }
        Expr::IsTrue(value) => nonnull_columns(value, columns),
        Expr::BinaryExpr(binary) if binary.op == Operator::And => {
            nonnull_columns(&binary.left, columns);
            nonnull_columns(&binary.right, columns);
        }
        _ => {}
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct RequireNonNull {
    signature: Signature,
    require: bool,
}
impl ScalarUDFImpl for RequireNonNull {
    fn name(&self) -> &str {
        if self.require {
            "pse_require_nonnull"
        } else {
            "pse_nullable"
        }
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        args.first()
            .cloned()
            .ok_or_else(|| invalid("expected one argument"))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [field] = args.arg_fields else {
            return Err(invalid("expected one argument"));
        };
        Ok(Arc::new(
            field.as_ref().clone().with_nullable(!self.require),
        ))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [value] = args.args.as_slice() else {
            return Err(invalid("expected one argument"));
        };
        let contains_null = self.require
            && match value {
                ColumnarValue::Array(array) => array.logical_null_count() != 0,
                ColumnarValue::Scalar(value) => value.is_null(),
            };
        if self.require && contains_null {
            return Err(invalid("nonnull output condition was not established"));
        }
        Ok(value.clone())
    }
}
fn invalid(message: &str) -> DataFusionError {
    DataFusionError::Execution(format!("pse_require_nonnull: {message}"))
}
