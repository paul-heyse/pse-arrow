// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Resolve rule operators against their actual input schemas before identity is assigned.

use std::collections::BTreeSet;

use crate::checks::{invalid, key_type};
use crate::model::{
    Cell, DepthBound, FieldContract, RuleAggregateFn, RuleExpr, RuleHead, RulePlan,
};
use crate::{Registry, SchemaError};

#[derive(Clone)]
struct Column {
    name: std::borrow::Cow<'static, str>,
    qualifier: Option<&'static str>,
    ty: Option<FieldContract>,
    nullable: bool,
    literal: Option<Cell>,
}

type Shape = Vec<Column>;
type Binders = Vec<(&'static str, Option<Shape>)>;

pub(crate) fn validate(
    rule: &str,
    plan: &RulePlan,
    head: &RuleHead,
    registry: &Registry,
) -> Result<(), SchemaError> {
    assertion_scope(rule, plan, true)?;
    let output = shape(rule, plan, registry, &mut Vec::new())?;
    let target = registry
        .relation(head.relation())
        .ok_or_else(|| unknown(rule, head.relation()))?;
    let names: Vec<_> = match head {
        RuleHead::Relation(_) => target.columns.iter().map(FieldContract::name).collect(),
        RuleHead::Violations { key_columns, .. } => {
            if key_columns.is_empty()
                || key_columns.iter().copied().collect::<BTreeSet<_>>().len() != key_columns.len()
            {
                return Err(invalid(
                    rule,
                    "a violations head needs distinct, nonempty key columns",
                ));
            }
            key_columns.clone()
        }
    };
    if output.len() != names.len() {
        return Err(invalid(
            rule,
            "the rule output does not contain exactly the declared head columns",
        ));
    }
    for name in names {
        let declared = target.column(name).ok_or_else(|| unknown(rule, name))?;
        let actual = lookup(rule, &output, name)?;
        let admitted = actual.literal.as_ref().map_or_else(
            || {
                (actual.ty.as_ref() == Some(&declared.value_type())
                    || (!actual.nullable
                        && !declared.nullable()
                        && actual.ty.as_ref().is_some_and(integer_head_type)
                        && integer_head_type(&declared.value_type())))
                    && (!actual.nullable || declared.nullable())
            },
            |value| {
                crate::checks::cell_value(
                    value,
                    &declared.value_type(),
                    declared.nullable(),
                    registry,
                )
            },
        );
        if !admitted {
            return Err(invalid(
                rule,
                format!("head column {name} has an incompatible type or nullability"),
            ));
        }
        if matches!(head, RuleHead::Violations { .. }) {
            check_key(rule, actual)?;
        }
    }
    Ok(())
}

fn assertion_scope(rule: &str, plan: &RulePlan, root: bool) -> Result<(), SchemaError> {
    if matches!(plan, RulePlan::Assert { .. }) && !root {
        return Err(invalid(
            rule,
            "an assertion predicate is allowed only at the root beneath output projections",
        ));
    }
    let root = root && matches!(plan, RulePlan::Project { .. });
    for child in plan.children() {
        assertion_scope(rule, child, root)?;
    }
    Ok(())
}

fn integer_head_type(ty: &FieldContract) -> bool {
    ty.extension().is_none()
        && matches!(
            ty.data_type(),
            arrow_schema::DataType::UInt8
                | arrow_schema::DataType::UInt16
                | arrow_schema::DataType::UInt32
                | arrow_schema::DataType::UInt64
                | arrow_schema::DataType::Int32
                | arrow_schema::DataType::Int64
        )
}

fn unknown(rule: &str, reference: &str) -> SchemaError {
    SchemaError::UnknownReference {
        context: format!("rule {rule}"),
        reference: reference.to_owned(),
    }
}

fn lookup<'a>(rule: &str, columns: &'a [Column], name: &str) -> Result<&'a Column, SchemaError> {
    let mut matches = columns.iter().filter(|column| {
        column.name == name
            || column
                .qualifier
                .is_some_and(|qualifier| name == format!("{qualifier}.{}", column.name))
    });
    let column = matches.next().ok_or_else(|| unknown(rule, name))?;
    if matches.next().is_some() {
        return Err(invalid(
            rule,
            format!("column {name} is ambiguous; qualify its input"),
        ));
    }
    Ok(column)
}

fn check_key(rule: &str, column: &Column) -> Result<(), SchemaError> {
    if column.ty == Some(FieldContract::native(arrow_schema::DataType::Float64)) {
        return Err(SchemaError::RuleFloatKey {
            rule: rule.to_owned(),
            column: column.name.to_string(),
        });
    }
    if !column.ty.as_ref().is_some_and(key_type) {
        return Err(invalid(
            rule,
            format!("column {} has no admitted key type", column.name),
        ));
    }
    Ok(())
}

fn compatible(rule: &str, left: &[Column], right: &[Column]) -> Result<(), SchemaError> {
    if left.len() != right.len()
        || left.iter().zip(right).any(|(left, right)| {
            left.name != right.name || left.ty != right.ty || left.nullable != right.nullable
        })
    {
        return Err(invalid(
            rule,
            "union/recursive branches have incompatible output schemas",
        ));
    }
    Ok(())
}

fn shape(
    rule: &str,
    plan: &RulePlan,
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    match plan {
        RulePlan::Scan { relation, port } => Ok(registry
            .relation(relation)
            .ok_or_else(|| unknown(rule, relation))?
            .columns
            .iter()
            .map(|column| Column {
                name: column.name().to_owned().into(),
                qualifier: Some(*port),
                ty: Some(column.value_type().clone()),
                nullable: column.nullable(),
                literal: None,
            })
            .collect()),
        RulePlan::RecursiveRef { name } => binders
            .iter()
            .rev()
            .find(|(candidate, _)| candidate == name)
            .and_then(|(_, schema)| schema.clone())
            .ok_or_else(|| {
                invalid(
                    rule,
                    format!(
                        "recursive reference {name} is unbound or appears in its binder's seed"
                    ),
                )
            }),
        RulePlan::Filter { input, predicate } | RulePlan::Assert { input, predicate } => {
            let mut input = shape(rule, input, registry, binders)?;
            let (ty, nullable) = expression(rule, predicate, &input, registry)?;
            require_bool(rule, ty.as_ref(), nullable)?;
            if matches!(plan, RulePlan::Filter { .. }) {
                refine_nonnull(predicate, &mut input);
            }
            Ok(input)
        }
        RulePlan::Project { input, columns } => {
            project_shape(rule, input, columns, registry, binders)
        }
        RulePlan::EquiJoin {
            left, right, keys, ..
        }
        | RulePlan::AntiJoin { left, right, keys } => join_shape(
            rule,
            (left, right),
            keys,
            matches!(plan, RulePlan::EquiJoin { .. }),
            registry,
            binders,
        ),
        RulePlan::Union(inputs) => union_shape(rule, inputs, registry, binders),
        RulePlan::Distinct(input) => {
            let output = shape(rule, input, registry, binders)?;
            for column in &output {
                check_key(rule, column)?;
            }
            Ok(output)
        }
        RulePlan::Aggregate {
            input,
            group,
            aggregates,
        } => aggregate_shape(rule, input, group, aggregates, registry, binders),
        RulePlan::Unnest {
            input,
            column,
            value_name,
            ..
        } => unnest_shape(rule, input, column, value_name.clone(), registry, binders),
        RulePlan::Recursive {
            name,
            seed,
            step,
            is_distinct,
            depth_bound,
        } => recursive_shape(
            rule,
            (name, *is_distinct, *depth_bound),
            (seed, step),
            registry,
            binders,
        ),
    }
}

fn union_shape(
    rule: &str,
    inputs: &[RulePlan],
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    let mut inputs = inputs.iter();
    let first = inputs
        .next()
        .ok_or_else(|| invalid(rule, "union requires an input"))?;
    let mut output = shape(rule, first, registry, binders)?;
    for input in inputs {
        let mut branch = shape(rule, input, registry, binders)?;
        for (left, right) in output.iter_mut().zip(&mut branch) {
            left.nullable |= right.nullable;
            right.nullable = left.nullable;
            if left.literal != right.literal {
                left.literal = None;
            }
            left.qualifier = None;
        }
        compatible(rule, &output, &branch)?;
    }
    Ok(output)
}

fn require_bool(rule: &str, ty: Option<&FieldContract>, nullable: bool) -> Result<(), SchemaError> {
    if ty.is_some_and(|ty| *ty != FieldContract::native(arrow_schema::DataType::Boolean))
        || (ty.is_none() && !nullable)
    {
        return Err(invalid(
            rule,
            "boolean operator received a nonboolean operand",
        ));
    }
    Ok(())
}

fn literal_type(value: &Cell) -> Option<FieldContract> {
    match value {
        Cell::Null | Cell::Enum(_) | Cell::Struct(_) => None,
        Cell::Bool(_) => Some(FieldContract::native(arrow_schema::DataType::Boolean)),
        Cell::I64(_) => Some(FieldContract::native(arrow_schema::DataType::Int64)),
        Cell::U64(_) => Some(FieldContract::native(arrow_schema::DataType::UInt64)),
        Cell::F64(_) => Some(FieldContract::native(arrow_schema::DataType::Float64)),
        Cell::Text(_) => Some(FieldContract::native(arrow_schema::DataType::Utf8)),
        Cell::Id(_) => Some(FieldContract::id()),
        Cell::Hash(_) => Some(FieldContract::hash()),
        Cell::List(values) => values
            .first()
            .and_then(literal_type)
            .filter(|first| {
                values
                    .iter()
                    .all(|value| literal_type(value).as_ref() == Some(first))
            })
            .map(FieldContract::list),
    }
}

fn expression(
    rule: &str,
    expr: &RuleExpr,
    input: &[Column],
    registry: &Registry,
) -> Result<(Option<FieldContract>, bool), SchemaError> {
    use RuleExpr as E;
    match expr {
        E::Col(name) => {
            let column = lookup(rule, input, name)?;
            Ok((column.ty.clone(), column.nullable))
        }
        E::Lit(value) => Ok((literal_type(value), matches!(value, Cell::Null))),
        E::Call {
            function,
            args,
            result,
            nullable,
        } => {
            call_shape(rule, function, args, result, input, registry)?;
            Ok((Some(result.clone()), *nullable))
        }
        E::And(values) | E::Or(values) => {
            if values.is_empty() {
                return Err(invalid(
                    rule,
                    "boolean conjunction/disjunction requires an operand",
                ));
            }
            let mut nullable = false;
            for value in values {
                let (ty, missing) = expression(rule, value, input, registry)?;
                require_bool(rule, ty.as_ref(), missing)?;
                nullable |= missing;
            }
            Ok((
                Some(FieldContract::native(arrow_schema::DataType::Boolean)),
                nullable,
            ))
        }
        E::Not(value) | E::IsTrue(value) | E::IsFalse(value) | E::IsUnknown(value) => {
            let (ty, nullable) = expression(rule, value, input, registry)?;
            require_bool(rule, ty.as_ref(), nullable)?;
            Ok((
                Some(FieldContract::native(arrow_schema::DataType::Boolean)),
                matches!(expr, E::Not(_)) && nullable,
            ))
        }
        E::IsNull(value) | E::IsNotNull(value) => {
            expression(rule, value, input, registry)?;
            Ok((
                Some(FieldContract::native(arrow_schema::DataType::Boolean)),
                false,
            ))
        }
        E::Cmp { l, r, .. } | E::IsDistinctFrom(l, r) | E::IsNotDistinctFrom(l, r) => {
            comparison(rule, expr, (l, r), input, registry)
        }
        E::InList { expr, list } => {
            let (ty, mut nullable) = expression(rule, expr, input, registry)?;
            for value in list {
                if !ty
                    .as_ref()
                    .is_some_and(|ty| crate::checks::cell_value(value, ty, true, registry))
                {
                    return Err(invalid(rule, "membership literal type differs from probe"));
                }
                nullable |= matches!(value, Cell::Null);
            }
            Ok((
                Some(FieldContract::native(arrow_schema::DataType::Boolean)),
                nullable,
            ))
        }
        E::Field { expr, name } => {
            let (ty, nullable) = expression(rule, expr, input, registry)?;
            let Some(arrow_schema::DataType::Struct(fields)) =
                ty.as_ref().map(FieldContract::data_type)
            else {
                return Err(invalid(rule, "field access requires a declared struct"));
            };
            let field = fields
                .iter()
                .find(|field| field.name() == name.as_ref())
                .ok_or_else(|| unknown(rule, name))?;
            Ok((
                Some(FieldContract::from_field((**field).clone()).value_type()),
                nullable || field.is_nullable(),
            ))
        }
        E::ListLen(expr) => {
            let (ty, nullable) = expression(rule, expr, input, registry)?;
            if !ty.is_some_and(|ty| {
                matches!(
                    ty.data_type(),
                    arrow_schema::DataType::List(_) | arrow_schema::DataType::FixedSizeList(..)
                )
            }) {
                return Err(invalid(rule, "list length requires a declared list"));
            }
            Ok((Some(FieldContract::nonnegative(i64::MAX)), nullable))
        }
    }
}

fn call_shape(
    rule: &str,
    function: &str,
    args: &[RuleExpr],
    result: &FieldContract,
    input: &[Column],
    registry: &Registry,
) -> Result<(), SchemaError> {
    if function.trim().is_empty() {
        return Err(invalid(
            rule,
            "native call requires an actual function name",
        ));
    }
    for argument in args {
        expression(rule, argument, input, registry)?;
    }
    // Registry assembly checks the declared shape. Only the retained engine
    // function can establish its actual result type and nullability.
    crate::arrow::field_for(
        registry,
        &FieldContract::payload("_call", result.clone(), "Expected native result obligation"),
    )?;
    Ok(())
}

fn comparison(
    rule: &str,
    expr: &RuleExpr,
    operands: (&RuleExpr, &RuleExpr),
    input: &[Column],
    registry: &Registry,
) -> Result<(Option<FieldContract>, bool), SchemaError> {
    use RuleExpr as E;
    let (l, r) = operands;
    let (left, left_null) = expression(rule, l, input, registry)?;
    let (right, right_null) = expression(rule, r, input, registry)?;
    let literal_fits = |expr: &RuleExpr, expected: Option<&FieldContract>| match (expr, expected) {
        (RuleExpr::Lit(value), Some(expected)) => {
            crate::checks::cell_value(value, expected, true, registry)
        }
        (RuleExpr::Lit(Cell::Null), None) => true,
        _ => false,
    };
    let unsigned = |ty: Option<&FieldContract>| {
        ty.is_some_and(|ty| {
            ty.extension().is_none()
                && matches!(
                    ty.data_type(),
                    arrow_schema::DataType::UInt8
                        | arrow_schema::DataType::UInt16
                        | arrow_schema::DataType::UInt32
                        | arrow_schema::DataType::UInt64
                )
        })
    };
    // Ordinal bounds compare the declared nonnegative Int64 index with
    // an actual target COUNT. This does not equate different target domains.
    let ordinal_bound = matches!(
        expr,
        E::Cmp {
            op: crate::model::CmpOp::Lt
                | crate::model::CmpOp::LtEq
                | crate::model::CmpOp::Gt
                | crate::model::CmpOp::GtEq,
            ..
        }
    ) && ((left.as_ref().is_some_and(|ty| {
        matches!(
            ty.extension(),
            Some(crate::model::ExtensionUse::OrdinalRef { .. })
        )
    }) && right
        == Some(FieldContract::native(arrow_schema::DataType::UInt64)))
        || (right.as_ref().is_some_and(|ty| {
            matches!(
                ty.extension(),
                Some(crate::model::ExtensionUse::OrdinalRef { .. })
            )
        }) && left == Some(FieldContract::native(arrow_schema::DataType::UInt64))));
    // Every unsigned integer width embeds exactly in UInt64. The engine's
    // integer coercion is lossless here; signed/float coercions remain refused.
    if !(left.is_some() && left == right
        || unsigned(left.as_ref()) && unsigned(right.as_ref())
        || ordinal_bound
        || literal_fits(l, right.as_ref())
        || literal_fits(r, left.as_ref()))
    {
        return Err(invalid(rule, "comparison operand types differ"));
    }
    Ok((
        Some(FieldContract::native(arrow_schema::DataType::Boolean)),
        matches!(expr, E::Cmp { .. }) && (left_null || right_null),
    ))
}

fn project_shape(
    rule: &str,
    input: &RulePlan,
    columns: &[(std::borrow::Cow<'static, str>, RuleExpr)],
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    let input = shape(rule, input, registry, binders)?;
    let mut names = BTreeSet::new();
    columns
        .iter()
        .map(|(name, expr)| {
            if !names.insert(name) {
                return Err(invalid(rule, format!("duplicate projected column {name}")));
            }
            let (ty, nullable) = expression(rule, expr, &input, registry)?;
            Ok(Column {
                name: name.clone(),
                qualifier: None,
                ty,
                nullable,
                literal: if let RuleExpr::Lit(value) = expr {
                    Some(value.clone())
                } else {
                    None
                },
            })
        })
        .collect()
}

fn join_shape(
    rule: &str,
    inputs: (&RulePlan, &RulePlan),
    keys: &[(
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    )],
    include_right: bool,
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    let mut left = shape(rule, inputs.0, registry, binders)?;
    let right = shape(rule, inputs.1, registry, binders)?;
    if keys.is_empty() {
        return Err(invalid(rule, "a keyed join requires at least one key pair"));
    }
    for (left_name, right_name) in keys {
        let left_key = lookup(rule, &left, left_name)?;
        let right_key = lookup(rule, &right, right_name)?;
        check_key(rule, left_key)?;
        check_key(rule, right_key)?;
        if left_key.ty != right_key.ty {
            return Err(invalid(rule, "join key types differ"));
        }
    }
    if include_right {
        left.extend(right);
    }
    Ok(left)
}

fn aggregate_shape(
    rule: &str,
    input: &RulePlan,
    group: &[std::borrow::Cow<'static, str>],
    aggregates: &[crate::model::RuleAggregate],
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    let input = shape(rule, input, registry, binders)?;
    let mut output = Vec::new();
    let mut names = BTreeSet::new();
    for name in group {
        let column = lookup(rule, &input, name)?;
        check_key(rule, column)?;
        if !names.insert(column.name.clone()) {
            return Err(invalid(rule, "duplicate aggregate group key"));
        }
        output.push(Column {
            qualifier: None,
            ..column.clone()
        });
    }
    for aggregate in aggregates {
        if !names.insert(aggregate.output_name.clone()) {
            return Err(invalid(rule, "duplicate aggregate output column"));
        }
        for (name, _) in &aggregate.order_by {
            check_key(rule, lookup(rule, &input, name)?)?;
        }
        let typed = aggregate
            .input
            .as_ref()
            .map(|expr| expression(rule, expr, &input, registry))
            .transpose()?;
        let ty = match aggregate.function {
            RuleAggregateFn::Count => Some(FieldContract::native(arrow_schema::DataType::UInt64)),
            RuleAggregateFn::CollectOrdered => {
                if aggregate.order_by.is_empty() {
                    return Err(invalid(
                        rule,
                        "ordered collection requires explicit ordering keys",
                    ));
                }
                Some(FieldContract::list(
                    typed
                        .clone()
                        .and_then(|(ty, _)| ty)
                        .ok_or_else(|| invalid(rule, "collection input has no declared type"))?,
                ))
            }
            RuleAggregateFn::Sum => match typed
                .clone()
                .and_then(|(ty, _)| ty)
                .as_ref()
                .map(FieldContract::data_type)
            {
                Some(arrow_schema::DataType::Int32 | arrow_schema::DataType::Int64) => {
                    Some(FieldContract::native(arrow_schema::DataType::Int64))
                }
                Some(
                    arrow_schema::DataType::UInt8
                    | arrow_schema::DataType::UInt16
                    | arrow_schema::DataType::UInt32
                    | arrow_schema::DataType::UInt64,
                ) => Some(FieldContract::native(arrow_schema::DataType::UInt64)),
                _ => {
                    return Err(invalid(rule, "rule sum requires a declared integer input"));
                }
            },
            RuleAggregateFn::Min | RuleAggregateFn::Max => typed.and_then(|(ty, _)| ty),
        };
        if ty.is_none() {
            return Err(invalid(rule, "aggregate requires a typed input"));
        }
        output.push(Column {
            name: aggregate.output_name.clone(),
            qualifier: None,
            ty,
            nullable: false,
            literal: None,
        });
    }
    Ok(output)
}

fn unnest_shape(
    rule: &str,
    input: &RulePlan,
    column: &str,
    value_name: std::borrow::Cow<'static, str>,
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    let mut input = shape(rule, input, registry, binders)?;
    let source = lookup(rule, &input, column)?;
    let Some(
        arrow_schema::DataType::List(element) | arrow_schema::DataType::FixedSizeList(element, _),
    ) = source.ty.as_ref().map(FieldContract::data_type)
    else {
        return Err(invalid(rule, "unnest requires a declared list column"));
    };
    let ty = Some(FieldContract::from_field((*element).clone()).value_type());
    let nullable = element.is_nullable();
    if input.iter().any(|column| column.name == value_name) {
        return Err(invalid(rule, "unnest output name already exists"));
    }
    input.push(Column {
        name: value_name,
        qualifier: None,
        ty,
        nullable,
        literal: None,
    });
    Ok(input)
}

fn recursive_shape(
    rule: &str,
    binding: (&'static str, bool, DepthBound),
    inputs: (&RulePlan, &RulePlan),
    registry: &Registry,
    binders: &mut Binders,
) -> Result<Shape, SchemaError> {
    let (name, _is_distinct, depth_bound) = binding;
    if matches!(depth_bound, DepthBound::Bounded(0)) {
        return Err(invalid(
            rule,
            "an explicit recursion bound must be positive",
        ));
    }
    binders.push((name, None));
    let seed = shape(rule, inputs.0, registry, binders)?;
    binders.pop();
    binders.push((name, Some(seed.clone())));
    let step = shape(rule, inputs.1, registry, binders)?;
    binders.pop();
    compatible(rule, &seed, &step)?;
    Ok(seed)
}

/// Whether a plan is positive and monotone in one workspace relation.
pub(crate) fn monotone_over(plan: &RulePlan, relation: &str) -> bool {
    match plan {
        RulePlan::AntiJoin { left, right, .. } => {
            !right
                .dependencies()
                .iter()
                .any(|(name, _, _)| *name == relation)
                && monotone_over(left, relation)
        }
        RulePlan::Aggregate { input, .. } => !input
            .dependencies()
            .iter()
            .any(|(name, _, _)| *name == relation),
        RulePlan::Recursive { .. } => false,
        other => other
            .children()
            .iter()
            .all(|child| monotone_over(child, relation)),
    }
}

fn refine_nonnull(predicate: &RuleExpr, columns: &mut [Column]) {
    match predicate {
        RuleExpr::IsTrue(inner) => refine_nonnull(inner, columns),
        RuleExpr::And(parts) => {
            for part in parts {
                refine_nonnull(part, columns);
            }
        }
        RuleExpr::IsNotNull(inner) => {
            if let RuleExpr::Col(name) = inner.as_ref() {
                for column in columns {
                    if column.name == *name
                        || column.qualifier.is_some_and(|qualifier| {
                            *name == format!("{qualifier}.{}", column.name)
                        })
                    {
                        column.nullable = false;
                    }
                }
            }
        }
        _ => {}
    }
}
