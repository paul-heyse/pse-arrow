// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native typed set operations for finite facts, assertions and conflict scopes.

use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::functions_aggregate::expr_fn::{count, first_value};
use datafusion::functions_window::expr_fn::row_number;
use datafusion_common::{Column, JoinType, NullEquality, ScalarValue, metadata::FieldMetadata};
use datafusion_expr::{
    Expr, ExprFunctionExt, ExprSchemable, LogicalPlan, LogicalPlanBuilder, col, lit,
};
use pse_catalog::session::scalar;
use pse_ids::SemanticId;
use pse_schema::{
    Registry,
    model::{ColumnRole, FieldContract, RelationSpec, RuleSpec},
};
use std::sync::Arc;

pub(super) fn semantic_columns(spec: &RelationSpec) -> Vec<&str> {
    spec.columns
        .iter()
        .filter(|column| column.role() != ColumnRole::Provenance)
        .map(FieldContract::name)
        .collect()
}

pub(super) fn id(value: SemanticId, registry: &Registry) -> Result<Expr, RuleError> {
    let field = pse_schema::arrow::field_for(
        registry,
        &FieldContract::payload("_identity", FieldContract::id(), "Exact declared identity"),
    )
    .map_err(|error| internal(error.to_string()))?;
    Ok(Expr::Literal(
        ScalarValue::FixedSizeBinary(16, Some(value.as_bytes().to_vec())),
        Some(FieldMetadata::from(&field)),
    ))
}

pub(super) fn declared_literal(
    registry: &Registry,
    column: &FieldContract,
    value: ScalarValue,
) -> Result<Expr, RuleError> {
    let field = pse_schema::arrow::field_for(registry, column)
        .map_err(|error| internal(error.to_string()))?;
    let value = if value.data_type() == *field.data_type() {
        value
    } else {
        value.cast_to(field.data_type()).map_err(engine)?
    };
    pse_catalog::session::output::checked_literal(registry, column, value).map_err(engine)
}

pub(super) fn key(spec: &RelationSpec) -> Expr {
    scalar::key(
        spec.id,
        spec.primary_key
            .iter()
            .map(|name| (*name, col(*name)))
            .collect(),
    )
}

/// Label framing is a presentation/identity codec only. Native typed columns remain
/// the authority for matching, conflicting and detecting an empty input delta.
pub(super) fn frame(expressions: Vec<Expr>) -> Expr {
    let mut output = lit("[");
    for (position, value) in expressions.into_iter().enumerate() {
        if position != 0 {
            output = concatenate(output, lit(","));
        }
        output = concatenate(output, scalar::literal(value));
    }
    concatenate(output, lit("]"))
}

fn concatenate(left: Expr, right: Expr) -> Expr {
    Expr::BinaryExpr(datafusion_expr::expr::BinaryExpr::new(
        Box::new(left),
        datafusion_expr::Operator::StringConcat,
        Box::new(right),
    ))
}

pub(super) fn assertion_id(
    rule: &RuleSpec,
    truth: &str,
    head: &RelationSpec,
    registry: &Registry,
) -> Result<Expr, RuleError> {
    let payload = frame(semantic_columns(head).into_iter().map(col).collect());
    let identity = frame(vec![
        lit("pse.rule-assertion.v1"),
        id(rule.id, registry)?,
        lit(truth),
        key(head),
        payload,
    ]);
    Ok(scalar::named_id(id(rule.id, registry)?, identity))
}

pub(super) fn declared_projection(
    plan: LogicalPlan,
    spec: &RelationSpec,
    expressions: Vec<Expr>,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    if expressions.len() != spec.columns.len() {
        return Err(internal(
            "native relation projection has the wrong declared width",
        ));
    }
    let expressions = expressions
        .into_iter()
        .zip(&spec.columns)
        .map(|(value, column)| {
            let field = pse_schema::arrow::field_for(registry, column)
                .map_err(|error| internal(error.to_string()))?;
            let value = if let Expr::Literal(scalar, _) = value {
                declared_literal(registry, column, scalar)?
            } else if value.get_type(plan.schema()).map_err(engine)? == *field.data_type() {
                value
            } else {
                Expr::Cast(datafusion_expr::expr::Cast::new_from_field(
                    Box::new(value),
                    Arc::new(field.clone()),
                ))
            };
            Ok(value.alias_with_metadata(column.name(), Some(FieldMetadata::from(&field))))
        })
        .collect::<Result<Vec<_>, RuleError>>()?;
    LogicalPlanBuilder::from(plan)
        .project(expressions)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn assertions(
    candidate: LogicalPlan,
    rule: &RuleSpec,
    truth: &str,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let head = registry
        .relation(rule.head.relation())
        .ok_or_else(|| internal("assertion head absent"))?;
    let spec = registry
        .relation(
            rule.assertion_relation
                .as_deref()
                .ok_or_else(|| internal("assertion declaration absent"))?,
        )
        .ok_or_else(|| internal("assertion declaration absent"))?;
    let mut expressions = vec![
        assertion_id(rule, truth, head, registry)?,
        id(rule.id, registry)?,
        lit(truth),
    ];
    expressions.extend(semantic_columns(head).into_iter().map(col));
    declared_projection(candidate, spec, expressions, registry)
}

pub(crate) fn union(
    plans: impl IntoIterator<Item = LogicalPlan>,
) -> Result<LogicalPlan, RuleError> {
    union_all(plans)?
        .distinct()
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

/// Assertion identities include the producer, truth, key and actual payload.
/// Deduplicate that identity and retain its original row. Grouping every payload
/// field would normalize floating signed zero, losing exact assertion evidence.
pub(super) fn union_assertions(
    plans: impl IntoIterator<Item = LogicalPlan>,
    spec: &RelationSpec,
) -> Result<LogicalPlan, RuleError> {
    // A window keeps the original arrays. DISTINCT ON lowers to FIRST_VALUE,
    // whose nested ScalarValue conversion discards child metadata at this pin.
    let ordinal = row_number()
        .partition_by(vec![col("assertion_id")])
        .order_by(vec![col("assertion_id").sort(true, true)])
        .build()
        .map_err(engine)?
        .alias("__pse_assertion_ordinal");
    union_all(plans)?
        .window(vec![ordinal])
        .and_then(|builder| builder.filter(col("__pse_assertion_ordinal").eq(lit(1u64))))
        .and_then(|builder| builder.project(spec.columns.iter().map(|column| col(column.name()))))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(crate) fn union_all(
    plans: impl IntoIterator<Item = LogicalPlan>,
) -> Result<LogicalPlanBuilder, RuleError> {
    let mut plans = plans.into_iter();
    let first = plans
        .next()
        .ok_or_else(|| internal("native union needs a declared empty input"))?;
    plans
        .try_fold(LogicalPlanBuilder::from(first), |builder, plan| {
            builder.union(plan)
        })
        .map_err(engine)
}

pub(crate) fn difference(
    left: LogicalPlan,
    right: LogicalPlan,
    columns: &[&str],
) -> Result<LogicalPlan, RuleError> {
    keyed_join(left, right, columns, JoinType::LeftAnti)
}

pub(super) fn keyed_join(
    left: LogicalPlan,
    right: LogicalPlan,
    columns: &[&str],
    kind: JoinType,
) -> Result<LogicalPlan, RuleError> {
    let left = LogicalPlanBuilder::from(left)
        .alias("__pse_left")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let right = LogicalPlanBuilder::from(right)
        .alias("__pse_right")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    LogicalPlanBuilder::from(left)
        .join_detailed(
            right,
            kind,
            (
                columns
                    .iter()
                    .map(|name| Column::new(Some("__pse_left"), *name))
                    .collect::<Vec<_>>(),
                columns
                    .iter()
                    .map(|name| Column::new(Some("__pse_right"), *name))
                    .collect::<Vec<_>>(),
            ),
            None,
            NullEquality::NullEqualsNull,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn facts(
    assertions: LogicalPlan,
    head: &RelationSpec,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let columns = semantic_columns(head);
    let builder = LogicalPlanBuilder::from(assertions.clone())
        .filter(col("truth").eq(lit("true")))
        .map_err(engine)?;
    let has_derivation = head
        .column("derivation_id")
        .is_some_and(|column| column.role() == ColumnRole::Provenance);
    // Grouping defines native value equality (including equal signed zeros), while
    // a window's ordering distinguishes their encodings. Group only to select an
    // actual assertion identity, then join back to its original complete payload.
    let selected = builder
        .aggregate(
            columns.iter().map(|name| col(*name)).collect::<Vec<_>>(),
            vec![
                first_value(
                    col("assertion_id"),
                    vec![col("assertion_id").sort(true, true)],
                )
                .alias("__pse_selected_assertion"),
            ],
        )
        .and_then(|builder| builder.project(vec![col("__pse_selected_assertion")]))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let actual = LogicalPlanBuilder::from(assertions)
        .alias("__pse_actual_fact")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let actual_column = |name| Expr::Column(Column::new(Some("__pse_actual_fact"), name));
    let mut projected = columns
        .iter()
        .map(|name| actual_column(*name))
        .collect::<Vec<_>>();
    if has_derivation {
        projected.push(actual_column("assertion_id").alias("derivation_id"));
    }
    let plan = LogicalPlanBuilder::from(selected)
        .join_detailed(
            actual,
            JoinType::Inner,
            (
                vec![Column::new_unqualified("__pse_selected_assertion")],
                vec![Column::new(Some("__pse_actual_fact"), "assertion_id")],
            ),
            None,
            NullEquality::NullEqualsNothing,
        )
        .and_then(|builder| builder.project(projected))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    declared_projection(
        plan,
        head,
        head.columns
            .iter()
            .map(|column| col(column.name()))
            .collect(),
        registry,
    )
}

pub(super) fn conflicts(
    assertions: LogicalPlan,
    head: &RelationSpec,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let positive = facts(assertions.clone(), head, registry)?;
    let incompatible = LogicalPlanBuilder::from(positive.clone())
        .aggregate(
            head.primary_key
                .iter()
                .map(|name| col(*name))
                .collect::<Vec<_>>(),
            vec![count(lit(1_u64)).alias("__pse_distinct_payloads")],
        )
        .map_err(engine)?
        .filter(col("__pse_distinct_payloads").gt(lit(1_i64)))
        .map_err(engine)?
        .project(head.primary_key.iter().map(|name| col(*name)))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let negative = LogicalPlanBuilder::from(assertions)
        .filter(col("truth").eq(lit("false")))
        .map_err(engine)?
        .project(head.primary_key.iter().map(|name| col(*name)))
        .and_then(LogicalPlanBuilder::distinct)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let contradictory = keyed_join(positive, negative, &head.primary_key, JoinType::LeftSemi)?;
    let contradictory = LogicalPlanBuilder::from(contradictory)
        .project(head.primary_key.iter().map(|name| col(*name)))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    union([incompatible, contradictory])
}

pub(super) fn identity_collisions(plan: LogicalPlan, name: &str) -> Result<LogicalPlan, RuleError> {
    LogicalPlanBuilder::from(plan)
        .distinct()
        .map_err(engine)?
        .aggregate(
            vec![col(name)],
            vec![count(lit(1_u64)).alias("__pse_distinct_identity_values")],
        )
        .map_err(engine)?
        .filter(col("__pse_distinct_identity_values").gt(lit(1_i64)))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn ordered(plan: LogicalPlan, spec: &RelationSpec) -> Result<LogicalPlan, RuleError> {
    LogicalPlanBuilder::from(plan)
        .sort(
            spec.primary_key
                .iter()
                .map(|name| col(*name).sort(true, true)),
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}
