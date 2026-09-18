// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite assertion-to-source edges; source identities are never truth keys.

use super::{LocatedRuleInput, RuleBindings, RuleInputLocation, relational};
use crate::{
    RuleError,
    errmap::{engine, internal},
    plan::trace::SupportPlan,
};
use datafusion_common::{Column, JoinType, NullEquality, ScalarValue};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit};
use pse_catalog::session::{SnapshotSession, scalar};
use pse_schema::{Registry, model::RuleSpec};

pub(super) fn mapping_role(rule: pse_ids::SemanticId, port: &str) -> String {
    format!("__pse_native:{rule}:{port}")
}

pub(super) fn plan(
    trace: SupportPlan,
    rule: &RuleSpec,
    truth: &str,
    bindings: &RuleBindings,
    session: &SnapshotSession,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let head = registry
        .relation(rule.head.as_str())
        .ok_or_else(|| internal("support head absent"))?;
    let assertion = registry
        .relation(
            rule.assertion_relation
                .as_deref()
                .ok_or_else(|| internal("assertion absent"))?,
        )
        .ok_or_else(|| internal("assertion absent"))?;
    let source = registry
        .relation(&trace.source.relation.qualified_name())
        .ok_or_else(|| internal("support source absent"))?;
    let bound = bindings
        .ports
        .get(trace.source.port)
        .ok_or_else(|| internal("support port absent"))?;
    let source_key = if trace.source.absence {
        lit(ScalarValue::FixedSizeBinary(32, None))
    } else {
        scalar::key(
            source.id,
            source
                .primary_key
                .iter()
                .enumerate()
                .map(|(position, name)| (*name, col(format!("__pse_support_{position}"))))
                .collect(),
        )
    };
    let common = vec![
        relational::id(rule.id, registry)?,
        relational::id(assertion.id, registry)?,
        relational::assertion_id(rule, truth, head, registry)?,
        relational::id(head.id, registry)?,
        relational::key(head),
    ];
    let (input, location) = if matches!(bound.location, RuleInputLocation::Native(_)) {
        mapped(
            trace.plan,
            source_key,
            rule,
            trace.source.port,
            trace.source.absence,
            session,
        )?
    } else {
        (
            trace.plan,
            location(
                bound,
                trace.source.port,
                source_key,
                trace.source.absence,
                registry,
            )?,
        )
    };
    let target = registry
        .relation("provenance.rule_support_edges")
        .ok_or_else(|| internal("support declaration absent"))?;
    let mut values = common;
    values.extend(location);
    values.push(relational::declared_literal(
        registry,
        target
            .column("truth")
            .ok_or_else(|| internal("support truth absent"))?,
        ScalarValue::Utf8(Some(truth.to_owned())),
    )?);
    let edge_id = scalar::named_id(
        relational::id(rule.id, registry)?,
        relational::frame(values.clone()),
    );
    values.insert(0, edge_id);
    relational::declared_projection(input, target, values, registry)
}

fn location(
    bound: &LocatedRuleInput,
    port: &str,
    key: Expr,
    absence: bool,
    registry: &Registry,
) -> Result<Vec<Expr>, RuleError> {
    let source = registry
        .relation(&bound.relation.qualified_name())
        .ok_or_else(|| internal("located source absent"))?;
    let kind = kind(&bound.location);
    let target = registry
        .relation("provenance.rule_support_edges")
        .ok_or_else(|| internal("support declaration absent"))?;
    Ok(vec![
        lit(port),
        relational::id(source.id, registry)?,
        selection(&bound.location, target, registry)?,
        relational::declared_literal(
            registry,
            target
                .column("support_kind")
                .ok_or_else(|| internal("support kind absent"))?,
            ScalarValue::Utf8(Some(if absence { "absence" } else { kind }.to_owned())),
        )?,
        key,
    ])
}

fn mapped(
    input: LogicalPlan,
    source_key: Expr,
    rule: &RuleSpec,
    port: &str,
    absence: bool,
    session: &SnapshotSession,
) -> Result<(LogicalPlan, Vec<Expr>), RuleError> {
    let head_names = input
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().to_owned())
        .collect::<Vec<_>>();
    let mut expressions = input
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    expressions.push(source_key.alias("__pse_native_key"));
    let input = LogicalPlanBuilder::from(input)
        .project(expressions)
        .map_err(engine)?
        .alias("__pse_head")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let mapping = session.scan_role(&mapping_role(rule.id, port))?;
    let mapping = LogicalPlanBuilder::from(mapping)
        .filter(if absence {
            col("output_key").is_null()
        } else {
            col("output_key").is_not_null()
        })
        .map_err(engine)?
        .alias("__pse_mapping")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let input = LogicalPlanBuilder::from(input)
        .join_detailed(
            mapping,
            JoinType::Inner,
            (
                vec![Column::new(Some("__pse_head"), "__pse_native_key")],
                vec![Column::new(Some("__pse_mapping"), "output_key")],
            ),
            None,
            NullEquality::NullEqualsNull,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let mut projection = head_names
        .iter()
        .map(|name| Expr::Column(Column::new(Some("__pse_head"), name)).alias(name))
        .collect::<Vec<_>>();
    let mut location = vec![];
    for (position, name) in [
        "input_port",
        "input_relation_id",
        "input_selection",
        "support_kind",
        "input_key",
    ]
    .into_iter()
    .enumerate()
    {
        let alias = format!("__pse_location_{position}");
        if head_names.contains(&alias) {
            return Err(internal("head collides with native support bookkeeping"));
        }
        projection.push(Expr::Column(Column::new(Some("__pse_mapping"), name)).alias(&alias));
        location.push(col(alias));
    }
    let input = LogicalPlanBuilder::from(input)
        .project(projection)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    Ok((input, location))
}

/// Source location is descriptive data; it never admits facts by a label or hash.
pub(super) fn kind(location: &RuleInputLocation) -> &'static str {
    match location {
        RuleInputLocation::Facts(facts) => {
            if facts.selection().is_some() {
                "delta"
            } else {
                "facts"
            }
        }
        RuleInputLocation::Workspace => "workspace",
        RuleInputLocation::Completed(_) | RuleInputLocation::Native(_) => "completed",
    }
}

pub(super) fn selection(
    location: &RuleInputLocation,
    target: &pse_schema::model::RelationSpec,
    registry: &Registry,
) -> Result<Expr, RuleError> {
    use pse_relations::columnar::ArrowValue;
    let column = target
        .column("input_selection")
        .ok_or_else(|| internal("support selection absent"))?;
    let field = pse_schema::arrow::field_for(registry, column)
        .map_err(|error| internal(error.to_string()))?;
    let value = match location {
        RuleInputLocation::Facts(facts) => facts.selection().cloned(),
        _ => None,
    };
    let array = value.to_array(field.data_type())?;
    relational::declared_literal(
        registry,
        column,
        ScalarValue::try_from_array(&array, 0).map_err(engine)?,
    )
}
