// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed scalar lowering retains declarations even when engine metadata is absent.
use super::{Column, Planned};
use crate::RuleError;
use crate::errmap::internal;
use datafusion::functions::core::expr_fn::get_field;
use datafusion::functions_nested::expr_fn::array_length;
use datafusion_common::ScalarValue;
use datafusion_expr::{Expr, lit};
use pse_schema::model::{Cell, CmpOp, ColumnSpec, LogicalType, RuleExpr};

pub(super) fn lookup<'a>(input: &'a Planned, name: &str) -> Result<&'a Column, RuleError> {
    let mut candidates = input.columns.iter().filter(|column| {
        column.spec.name == name
            || column
                .qualifier
                .as_ref()
                .is_some_and(|qualifier| name == format!("{qualifier}.{}", column.spec.name))
    });
    let result = candidates
        .next()
        .ok_or_else(|| internal(format!("unknown rule column {name}")))?;
    if candidates.next().is_some() {
        return Err(internal(format!("ambiguous rule column {name}")));
    }
    Ok(result)
}

pub(super) fn column_expression(column: &Column) -> Expr {
    Expr::Column(datafusion_common::Column::new(
        column.qualifier.clone(),
        column.spec.name,
    ))
}

type TypedExpr = (Expr, Column);

pub(super) fn lower(
    source: &RuleExpr,
    input: &Planned,
    registry: &pse_schema::Registry,
) -> Result<(Expr, Column), RuleError> {
    let result = match source {
        RuleExpr::Col(name) => {
            let column = lookup(input, name)?.clone();
            (column_expression(&column), column)
        }
        RuleExpr::Lit(value) => {
            let (value_expr, ty) = literal(value, registry)?;
            let mut column = synthetic(ty, matches!(value, Cell::Null));
            column.literal = Some(value.clone());
            (value_expr, column)
        }
        RuleExpr::And(values) | RuleExpr::Or(values) => {
            junction(values, matches!(source, RuleExpr::And(_)), input, registry)?
        }
        RuleExpr::Not(inner) => {
            let (value, column) = lower(inner, input, registry)?;
            boolean(&column)?;
            (!value, synthetic(LogicalType::Bool, column.spec.nullable))
        }
        RuleExpr::Cmp { op, l, r } => {
            let ((left, left_type), (right, right_type)) = operands(l, r, input, registry)?;
            let expr = match op {
                CmpOp::Eq => left.eq(right),
                CmpOp::NotEq => left.not_eq(right),
                CmpOp::Lt => left.lt(right),
                CmpOp::LtEq => left.lt_eq(right),
                CmpOp::Gt => left.gt(right),
                CmpOp::GtEq => left.gt_eq(right),
            };
            (
                expr,
                synthetic(
                    LogicalType::Bool,
                    left_type.spec.nullable || right_type.spec.nullable,
                ),
            )
        }
        RuleExpr::IsDistinctFrom(left, right) | RuleExpr::IsNotDistinctFrom(left, right) => {
            let ((left, _), (right, _)) = operands(left, right, input, registry)?;
            let op = if matches!(source, RuleExpr::IsDistinctFrom(..)) {
                datafusion_expr::Operator::IsDistinctFrom
            } else {
                datafusion_expr::Operator::IsNotDistinctFrom
            };
            (
                Expr::BinaryExpr(datafusion_expr::expr::BinaryExpr::new(
                    Box::new(left),
                    op,
                    Box::new(right),
                )),
                synthetic(LogicalType::Bool, false),
            )
        }
        RuleExpr::IsNull(inner)
        | RuleExpr::IsNotNull(inner)
        | RuleExpr::IsTrue(inner)
        | RuleExpr::IsFalse(inner)
        | RuleExpr::IsUnknown(inner) => {
            let (value, column) = lower(inner, input, registry)?;
            let expr = truth_test(source, value, &column)?;
            (expr, synthetic(LogicalType::Bool, false))
        }
        RuleExpr::InList { expr, list } => in_list(expr, list, input, registry)?,
        RuleExpr::Field { expr, name } => {
            let (value, column) = lower(expr, input, registry)?;
            let LogicalType::Struct(fields) = &column.spec.logical_type else {
                return Err(internal("field access requires a declared struct"));
            };
            let (_, ty, nullable) = fields
                .iter()
                .find(|(field, _, _)| field == name)
                .ok_or_else(|| internal(format!("unknown struct member {name}")))?;
            (
                get_field(value, *name),
                synthetic(ty.clone(), column.spec.nullable || *nullable),
            )
        }
        RuleExpr::ListLen(expr) => {
            let (value, column) = lower(expr, input, registry)?;
            if !matches!(
                column.spec.logical_type,
                LogicalType::List(_) | LogicalType::FixedList(..)
            ) {
                return Err(internal("list length requires a declared list"));
            }
            (
                array_length(value),
                synthetic(LogicalType::U64, column.spec.nullable),
            )
        }
    };
    Ok(result)
}

fn operands(
    left: &RuleExpr,
    right: &RuleExpr,
    input: &Planned,
    registry: &pse_schema::Registry,
) -> Result<(TypedExpr, TypedExpr), RuleError> {
    let (mut left, mut left_type) = lower(left, input, registry)?;
    let (mut right, mut right_type) = lower(right, input, registry)?;
    if left_type.literal == Some(Cell::Null) {
        left = typed_null(&right_type.spec, registry)?;
        left_type.spec = right_type.spec.clone();
        left_type.spec.nullable = true;
    }
    if right_type.literal == Some(Cell::Null) {
        right = typed_null(&left_type.spec, registry)?;
        right_type.spec = left_type.spec.clone();
        right_type.spec.nullable = true;
    }
    if left_type.spec.quantity != right_type.spec.quantity
        && left_type.literal != Some(Cell::Null)
        && right_type.literal != Some(Cell::Null)
    {
        return Err(internal(
            "comparison operands have different complete quantity contracts",
        ));
    }
    Ok(((left, left_type), (right, right_type)))
}
pub(super) fn typed_null(
    target: &ColumnSpec,
    registry: &pse_schema::Registry,
) -> Result<Expr, RuleError> {
    let mut target = target.clone();
    target.nullable = true;
    let field = pse_schema::arrow::field_for(registry, &target)
        .map_err(|error| internal(error.to_string()))?;
    let scalar = ScalarValue::try_from(field.data_type()).map_err(crate::errmap::engine)?;
    Ok(Expr::Literal(
        scalar,
        Some(datafusion_common::metadata::FieldMetadata::from(&field)),
    ))
}

pub(super) fn boolean(column: &Column) -> Result<(), RuleError> {
    if column.spec.logical_type == LogicalType::Bool || column.literal == Some(Cell::Null) {
        Ok(())
    } else {
        Err(internal("predicate is not Boolean"))
    }
}

pub(super) fn literal(
    value: &Cell,
    registry: &pse_schema::Registry,
) -> Result<(Expr, LogicalType), RuleError> {
    let (scalar, ty) = match value {
        Cell::Null => (ScalarValue::Boolean(None), LogicalType::Bool),
        Cell::Bool(v) => (ScalarValue::Boolean(Some(*v)), LogicalType::Bool),
        Cell::I64(v) => (ScalarValue::Int64(Some(*v)), LogicalType::I64),
        Cell::U64(v) => (ScalarValue::UInt64(Some(*v)), LogicalType::U64),
        Cell::F64(v) if v.is_finite() => (ScalarValue::Float64(Some(*v)), LogicalType::F64),
        Cell::Text(v) => (ScalarValue::Utf8(Some(v.clone())), LogicalType::Text),
        Cell::Id(v) => (
            ScalarValue::FixedSizeBinary(16, Some(v.as_bytes().to_vec())),
            LogicalType::Ext(pse_schema::model::ExtensionUse::SemanticId),
        ),
        Cell::Hash(v) => (
            ScalarValue::FixedSizeBinary(32, Some(v.as_bytes().to_vec())),
            LogicalType::Ext(pse_schema::model::ExtensionUse::ContentHash),
        ),
        Cell::Enum(v) => (ScalarValue::Utf8(Some((*v).to_owned())), LogicalType::Text),
        _ => {
            return Err(internal(
                "rule literal requires a finite scalar or an explicitly typed destination",
            ));
        }
    };
    let field = pse_schema::arrow::field_for(
        registry,
        &ColumnSpec::payload("_literal", ty.clone(), "Explicit typed literal"),
    )
    .map_err(|error| internal(error.to_string()))?;
    Ok((
        Expr::Literal(
            scalar,
            Some(datafusion_common::metadata::FieldMetadata::from(&field)),
        ),
        ty,
    ))
}

/// Arrow nullability is part of a declared column, not inferred from storage values.
trait Nullable {
    fn with_nullable(self, nullable: bool) -> Self;
}
impl Nullable for ColumnSpec {
    fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
}

fn synthetic(ty: LogicalType, nullable: bool) -> Column {
    Column {
        spec: ColumnSpec::payload("_expression", ty, "Typed rule expression")
            .with_nullable(nullable),
        qualifier: None,
        literal: None,
    }
}

fn truth_test(source: &RuleExpr, value: Expr, column: &Column) -> Result<Expr, RuleError> {
    let expr = match source {
        RuleExpr::IsNull(_) => value.is_null(),
        RuleExpr::IsNotNull(_) => value.is_not_null(),
        RuleExpr::IsTrue(_) => {
            boolean(column)?;
            Expr::IsTrue(Box::new(value))
        }
        RuleExpr::IsFalse(_) => {
            boolean(column)?;
            Expr::IsFalse(Box::new(value))
        }
        _ => {
            boolean(column)?;
            Expr::IsUnknown(Box::new(value))
        }
    };
    Ok(expr)
}

fn in_list(
    expr: &RuleExpr,
    list: &[Cell],
    input: &Planned,
    registry: &pse_schema::Registry,
) -> Result<TypedExpr, RuleError> {
    let (mut value, mut column) = lower(expr, input, registry)?;
    if column.literal == Some(Cell::Null)
        && let Some(member) = list.iter().find(|cell| !matches!(cell, Cell::Null))
    {
        let (_, ty) = literal(member, registry)?;
        column.spec.logical_type = ty;
        value = typed_null(&column.spec, registry)?;
    }
    let expressions = list
        .iter()
        .map(|value| {
            if matches!(value, Cell::Null) {
                typed_null(&column.spec, registry)
            } else {
                literal(value, registry).map(|(expr, _)| expr)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        value.in_list(expressions, false),
        synthetic(
            LogicalType::Bool,
            column.spec.nullable || list.contains(&Cell::Null),
        ),
    ))
}

fn junction(
    values: &[RuleExpr],
    and: bool,
    input: &Planned,
    registry: &pse_schema::Registry,
) -> Result<TypedExpr, RuleError> {
    let mut result = lit(and);
    let mut nullable = false;
    for value in values {
        let (value, column) = lower(value, input, registry)?;
        boolean(&column)?;
        nullable |= column.spec.nullable;
        result = if and {
            result.and(value)
        } else {
            result.or(value)
        };
    }
    Ok((result, synthetic(LogicalType::Bool, nullable)))
}
