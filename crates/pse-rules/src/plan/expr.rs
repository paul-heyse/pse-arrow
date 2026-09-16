// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed scalar lowering retains declarations even when engine metadata is absent.
use super::{Column, Planned};
use crate::RuleError;
use crate::errmap::internal;
use datafusion::arrow::datatypes::DataType;
use datafusion::functions::core::expr_fn::get_field;
use datafusion::functions_nested::expr_fn::array_length;
use datafusion_common::ScalarValue;
use datafusion_expr::{Expr, ExprSchemable, lit};
use pse_catalog::session::SnapshotSession;
use pse_schema::model::{Cell, CmpOp, FieldContract, RuleExpr};

pub(super) fn lookup<'a>(input: &'a Planned, name: &str) -> Result<&'a Column, RuleError> {
    let exact = |column: &&Column| {
        column.qualifier.as_ref().map_or_else(
            || column.name == name,
            |qualifier| name == format!("{qualifier}.{}", column.name),
        )
    };
    let has_exact = input.columns.iter().any(|column| exact(&column));
    let mut candidates = input.columns.iter().filter(|column| {
        if has_exact {
            exact(column)
        } else {
            column.name == name
        }
    });
    let result = candidates.next().ok_or_else(|| {
        internal(format!(
            "unknown rule column {name}; available: {}",
            input
                .columns
                .iter()
                .map(|column| column.qualifier.as_ref().map_or_else(
                    || column.name.to_string(),
                    |qualifier| format!("{qualifier}.{}", column.name)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })?;
    if candidates.next().is_some() {
        return Err(internal(format!("ambiguous rule column {name}")));
    }
    Ok(result)
}

pub(super) fn column_expression(column: &Column) -> Expr {
    Expr::Column(column.physical.clone().unwrap_or_else(|| {
        datafusion_common::Column::new(column.qualifier.clone(), column.name.as_ref())
    }))
}

type TypedExpr = (Expr, Column);

pub(super) fn lower(
    source: &RuleExpr,
    input: &Planned,
    registry: &pse_schema::Registry,
    session: &SnapshotSession,
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
        RuleExpr::Call {
            function,
            args,
            result,
            nullable,
        } => call(function, args, result, *nullable, input, registry, session)?,

        RuleExpr::And(values) | RuleExpr::Or(values) => junction(
            values,
            matches!(source, RuleExpr::And(_)),
            input,
            registry,
            session,
        )?,
        RuleExpr::Not(inner) => {
            let (value, column) = lower(inner, input, registry, session)?;
            boolean(&column)?;
            (
                !value,
                synthetic(
                    FieldContract::native(DataType::Boolean),
                    column.spec.nullable(),
                ),
            )
        }
        RuleExpr::Cmp { op, l, r } => {
            let ((left, left_type), (right, right_type)) =
                operands(l, r, input, registry, session)?;
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
                    FieldContract::native(DataType::Boolean),
                    left_type.spec.nullable() || right_type.spec.nullable(),
                ),
            )
        }
        RuleExpr::IsDistinctFrom(left, right) | RuleExpr::IsNotDistinctFrom(left, right) => {
            let ((left, _), (right, _)) = operands(left, right, input, registry, session)?;
            let op = if matches!(source, RuleExpr::IsDistinctFrom(..)) {
                datafusion_expr::Operator::IsDistinctFrom
            } else {
                datafusion_expr::Operator::IsNotDistinctFrom
            };
            (
                // Native DISTINCT comparisons are total Boolean predicates, but
                // DataFusion 55's binary field derivation still ORs operand
                // nullability. IS TRUE preserves that total result and exposes
                // its non-null field through native expression machinery.
                Expr::IsTrue(Box::new(Expr::BinaryExpr(
                    datafusion_expr::expr::BinaryExpr::new(Box::new(left), op, Box::new(right)),
                ))),
                synthetic(FieldContract::native(DataType::Boolean), false),
            )
        }
        RuleExpr::IsNull(inner)
        | RuleExpr::IsNotNull(inner)
        | RuleExpr::IsTrue(inner)
        | RuleExpr::IsFalse(inner)
        | RuleExpr::IsUnknown(inner) => {
            let (value, column) = lower(inner, input, registry, session)?;
            let expr = truth_test(source, value, &column)?;
            (
                expr,
                synthetic(FieldContract::native(DataType::Boolean), false),
            )
        }
        RuleExpr::InList { expr, list } => in_list(expr, list, input, registry, session)?,
        RuleExpr::Field { expr, name } => struct_field(expr, name, input, registry, session)?,
        RuleExpr::ListLen(expr) => {
            let (value, column) = lower(expr, input, registry, session)?;
            if !matches!(
                column.spec.data_type(),
                DataType::List(_) | DataType::FixedSizeList(..)
            ) {
                return Err(internal("list length requires a declared list"));
            }
            (
                datafusion::logical_expr::cast(array_length(value), DataType::Int64),
                synthetic(FieldContract::nonnegative(i64::MAX), column.spec.nullable()),
            )
        }
    };
    Ok(result)
}

fn call(
    function: &str,
    args: &[RuleExpr],
    result: &FieldContract,
    nullable: bool,
    input: &Planned,
    registry: &pse_schema::Registry,
    session: &SnapshotSession,
) -> Result<TypedExpr, RuleError> {
    let arguments = args
        .iter()
        .map(|argument| lower(argument, input, registry, session).map(|(value, _)| value))
        .collect::<Result<Vec<_>, _>>()?;
    let implementation = session.scalar_function(function)?;
    let fields = arguments
        .iter()
        .map(|argument| {
            argument
                .to_field(input.plan.schema())
                .map(|(_, field)| field)
        })
        .collect::<datafusion_common::Result<Vec<_>>>()
        .map_err(crate::errmap::engine)?;
    let coerced = datafusion_expr::type_coercion::functions::fields_with_udf(
        &fields,
        implementation.as_ref(),
    )
    .map_err(crate::errmap::engine)?;
    let arguments = arguments
        .into_iter()
        .zip(coerced)
        .map(|(argument, field)| argument.cast_to(field.data_type(), input.plan.schema()))
        .collect::<datafusion_common::Result<Vec<_>>>()
        .map_err(crate::errmap::engine)?;
    let mut value = implementation.call(arguments);
    let (_, actual) = value
        .to_field(input.plan.schema())
        .map_err(crate::errmap::engine)?;
    let expected = synthetic(result.clone(), nullable);
    let field = pse_schema::arrow::field_for(registry, &expected.spec)
        .map_err(|error| internal(error.to_string()))?;
    pse_catalog::session::output::check_field_output(&actual, &field).map_err(|error| {
        crate::errmap::engine(datafusion_common::DataFusionError::Context(
            format!("native call {function} expected output after actual argument coercion"),
            Box::new(error),
        ))
    })?;
    if actual.data_type() != field.data_type() {
        value = value
            .cast_to(field.data_type(), input.plan.schema())
            .map_err(crate::errmap::engine)?;
    }
    Ok((value, expected))
}

fn struct_field(
    expr: &RuleExpr,
    name: &str,
    input: &Planned,
    registry: &pse_schema::Registry,
    session: &SnapshotSession,
) -> Result<TypedExpr, RuleError> {
    let (value, column) = lower(expr, input, registry, session)?;
    let DataType::Struct(fields) = column.spec.data_type() else {
        return Err(internal("field access requires a declared struct"));
    };
    let field = fields
        .iter()
        .find(|field| field.name() == name)
        .ok_or_else(|| internal(format!("unknown struct member {name}")))?;
    Ok((
        get_field(value, name),
        synthetic(
            FieldContract::from_field((**field).clone()).value_type(),
            column.spec.nullable() || field.is_nullable(),
        ),
    ))
}

fn operands(
    left: &RuleExpr,
    right: &RuleExpr,
    input: &Planned,
    registry: &pse_schema::Registry,
    session: &SnapshotSession,
) -> Result<(TypedExpr, TypedExpr), RuleError> {
    let (mut left, mut left_type) = lower(left, input, registry, session)?;
    let (mut right, mut right_type) = lower(right, input, registry, session)?;
    if left_type.literal == Some(Cell::Null) {
        left = typed_null(&right_type.spec, registry)?;
        left_type.spec = right_type.spec.clone();
        left_type.spec = left_type.spec.clone().optional();
    }
    if right_type.literal == Some(Cell::Null) {
        right = typed_null(&left_type.spec, registry)?;
        right_type.spec = left_type.spec.clone();
        right_type.spec = right_type.spec.clone().optional();
    }
    if left_type.spec.quantity() != right_type.spec.quantity()
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
    target: &FieldContract,
    registry: &pse_schema::Registry,
) -> Result<Expr, RuleError> {
    let mut target = target.clone();
    target = target.clone().optional();
    let field = pse_schema::arrow::field_for(registry, &target)
        .map_err(|error| internal(error.to_string()))?;
    let scalar = ScalarValue::try_from(field.data_type()).map_err(crate::errmap::engine)?;
    Ok(Expr::Literal(
        scalar,
        Some(datafusion_common::metadata::FieldMetadata::from(&field)),
    ))
}

pub(super) fn boolean(column: &Column) -> Result<(), RuleError> {
    if column.spec.value_type() == FieldContract::native(DataType::Boolean)
        || column.literal == Some(Cell::Null)
    {
        Ok(())
    } else {
        Err(internal("predicate is not Boolean"))
    }
}

pub(super) fn literal(
    value: &Cell,
    registry: &pse_schema::Registry,
) -> Result<(Expr, FieldContract), RuleError> {
    let (scalar, ty) = match value {
        Cell::Null => (
            ScalarValue::Boolean(None),
            FieldContract::native(DataType::Boolean),
        ),
        Cell::Bool(v) => (
            ScalarValue::Boolean(Some(*v)),
            FieldContract::native(DataType::Boolean),
        ),
        Cell::I64(v) => (
            ScalarValue::Int64(Some(*v)),
            FieldContract::native(DataType::Int64),
        ),
        Cell::U64(v) => (
            ScalarValue::UInt64(Some(*v)),
            FieldContract::native(DataType::UInt64),
        ),
        Cell::F64(v) if v.is_finite() => (
            ScalarValue::Float64(Some(*v)),
            FieldContract::native(DataType::Float64),
        ),
        Cell::Text(v) => (
            ScalarValue::Utf8(Some(v.clone())),
            FieldContract::native(DataType::Utf8),
        ),
        Cell::Id(v) => (
            ScalarValue::FixedSizeBinary(16, Some(v.as_bytes().to_vec())),
            FieldContract::extended(pse_schema::model::ExtensionUse::SemanticId),
        ),
        Cell::Hash(v) => (
            ScalarValue::FixedSizeBinary(32, Some(v.as_bytes().to_vec())),
            FieldContract::extended(pse_schema::model::ExtensionUse::ContentHash),
        ),
        Cell::Enum(v) => (
            ScalarValue::Utf8(Some((*v).to_owned())),
            FieldContract::native(DataType::Utf8),
        ),
        _ => {
            return Err(internal(
                "rule literal requires a finite scalar or an explicitly typed destination",
            ));
        }
    };
    let field = pse_schema::arrow::field_for(
        registry,
        &FieldContract::payload("_literal", ty.clone(), "Explicit typed literal"),
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
fn synthetic(ty: FieldContract, nullable: bool) -> Column {
    Column {
        name: "_expression".into(),
        spec: FieldContract::payload("_expression", ty, "Typed rule expression")
            .with_nullable(nullable),
        qualifier: None,
        physical: None,
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
    session: &SnapshotSession,
) -> Result<TypedExpr, RuleError> {
    let (mut value, mut column) = lower(expr, input, registry, session)?;
    if column.literal == Some(Cell::Null)
        && let Some(member) = list.iter().find(|cell| !matches!(cell, Cell::Null))
    {
        let (_, ty) = literal(member, registry)?;
        column.spec = FieldContract::new(
            column.spec.name(),
            ty,
            column.spec.nullable(),
            column.spec.role(),
            column.spec.doc(),
        );
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
            FieldContract::native(DataType::Boolean),
            column.spec.nullable() || list.contains(&Cell::Null),
        ),
    ))
}

fn junction(
    values: &[RuleExpr],
    and: bool,
    input: &Planned,
    registry: &pse_schema::Registry,
    session: &SnapshotSession,
) -> Result<TypedExpr, RuleError> {
    let mut result = lit(and);
    let mut nullable = false;
    for value in values {
        let (value, column) = lower(value, input, registry, session)?;
        boolean(&column)?;
        nullable |= column.spec.nullable();
        result = if and {
            result.and(value)
        } else {
            result.or(value)
        };
    }
    Ok((
        result,
        synthetic(FieldContract::native(DataType::Boolean), nullable),
    ))
}
