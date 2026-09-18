// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared and lexical axes become ordered factors through native owner/name joins.
use super::native::{Plans, append, c, error, join, prefix, project, union};
use crate::CompilerError;
use datafusion::functions_aggregate::expr_fn::array_agg;
use datafusion::{
    functions::core::expr_fn::{coalesce, get_field},
    functions_nested::expr_fn::flatten,
    logical_expr::{Expr, ExprFunctionExt, JoinType, LogicalPlan, LogicalPlanBuilder, col, when},
};
use pse_catalog::session::scalar;

pub(super) async fn build(plans: &mut Plans<'_>) -> Result<LogicalPlan, CompilerError> {
    let instances = plans.scan("normalized.instance_bindings", "instance")?;
    let mut shapes = vec![project(
        instances.clone(),
        [
            c("instance", "instance_id").alias("owner"),
            scalar::id_list(vec![]).alias("factors"),
            c("instance", "support").alias("supports"),
        ],
    )?];
    for relation in [
        "normalized.template_symbols",
        "normalized.template_equations",
        "normalized.instance_equations",
    ] {
        let declaration = plans.scan(relation, "declaration")?;
        let (left, right) = if relation == "normalized.instance_equations" {
            ("instance_id", "instance_id")
        } else {
            ("template_id", "template_id")
        };
        let base = join(
            instances.clone(),
            declaration,
            JoinType::Inner,
            [c("instance", left).eq(c("declaration", right))],
        )?;
        let base = project(
            base,
            [
                c("instance", "instance_id").alias("owner"),
                c("declaration", "source_token").alias("shape_key"),
                c("declaration", "indexed_by").alias("names"),
                plans
                    .lists(vec![c("instance", "support"), c("declaration", "support")])?
                    .alias("supports"),
            ],
        )?;
        let bound = plans
            .named_axes(base, &["owner", "shape_key"], "owner", "names", "factors")
            .await?;
        shapes.push(project(
            bound,
            [col("owner"), col("factors"), col("supports")],
        )?);
    }
    let bindings = plans.scan("normalized.instance_domain_bindings", "binding")?;
    let single = join(
        instances.clone(),
        bindings,
        JoinType::Inner,
        [c("instance", "instance_id").eq(c("binding", "instance_id"))],
    )?;
    shapes.push(project(
        single,
        [
            c("instance", "instance_id").alias("owner"),
            scalar::id_list(vec![c("binding", "domain_id")]).alias("factors"),
            plans
                .lists(vec![c("instance", "support"), c("binding", "support")])?
                .alias("supports"),
        ],
    )?);
    shapes.push(expression_axes(plans, instances).await?);
    plans.retain(union(shapes)?).await
}
#[expect(
    clippy::too_many_lines,
    reason = "expression_axes keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn expression_axes(
    plans: &mut Plans<'_>,
    instances: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let sources = plans.scan("normalized.expression_sources", "source")?;
    let base = join(
        instances,
        sources,
        JoinType::Inner,
        [c("instance", "instance_id")
            .eq(get_field(
                get_field(c("source", "owner"), "instance"),
                "instance_id",
            ))
            .or(c("instance", "template_id").eq(get_field(
                get_field(c("source", "owner"), "template"),
                "template_id",
            )))],
    )?;
    let base = project(
        base,
        [
            c("instance", "instance_id").alias("owner"),
            c("instance", "template_id").alias("template"),
            c("source", "source_id").alias("product_source_id"),
            plans
                .lists(vec![c("instance", "support"), c("source", "support")])?
                .alias("supports"),
        ],
    )?;
    let axes = plans.scan("normalized.expression_index_bindings", "axis")?;
    let actual_domain = get_field(get_field(c("axis", "domain"), "actual"), "domain_id");
    let template_domain = get_field(c("axis", "domain"), "template");
    let expanded = join(
        base.clone(),
        axes,
        JoinType::Inner,
        [col("product_source_id").eq(c("axis", "source_id"))],
    )?;
    plans
        .require(
            &expanded,
            actual_domain.clone().is_not_null().or(get_field(
                template_domain.clone(),
                "template_id",
            )
            .eq(col("template"))),
            "lexical product binder belongs to another template",
        )
        .await?;
    let binding = plans.scan("normalized.instance_domain_bindings", "binding")?;
    let expanded = join(
        expanded,
        binding,
        JoinType::Left,
        [
            actual_domain.clone().is_null(),
            col("owner").eq(c("binding", "instance_id")),
            get_field(template_domain, "name").eq(c("binding", "domain_name")),
        ],
    )?;
    let domain = coalesce(vec![actual_domain.clone(), c("binding", "domain_id")]);
    plans
        .require(
            &expanded,
            domain.clone().is_not_null(),
            "lexical product binder has no actual domain",
        )
        .await?;
    let supports = when(actual_domain.is_not_null(), c("axis", "support"))
        .otherwise(plans.lists(vec![c("axis", "support"), c("binding", "support")])?)
        .map_err(error)?;
    let expanded = append(
        expanded,
        [
            plans.present(domain)?.alias("axis_domain"),
            supports.alias("axis_support"),
        ],
    )?;
    let agg = LogicalPlanBuilder::from(expanded)
        .aggregate(
            [col("owner"), col("product_source_id")],
            [
                ordered(
                    array_agg(col("axis_domain")),
                    vec![
                        c("axis", "position").sort(true, false),
                        c("axis", "bound_index_id").sort(true, false),
                    ],
                )?
                .alias("axis_factors"),
                array_agg(col("axis_support")).alias("axis_supports"),
            ],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let joined = join(
        base,
        prefix(agg, "axes")?,
        JoinType::Left,
        [
            col("owner").eq(c("axes", "owner")),
            col("product_source_id").eq(c("axes", "product_source_id")),
        ],
    )?;
    project(
        joined,
        [
            col("owner"),
            coalesce(vec![c("axes", "axis_factors"), scalar::id_list(vec![])]).alias("factors"),
            plans
                .lists(vec![col("supports"), flatten(c("axes", "axis_supports"))])?
                .alias("supports"),
        ],
    )
}
fn ordered(
    expr: Expr,
    order: Vec<datafusion::logical_expr::SortExpr>,
) -> Result<Expr, CompilerError> {
    expr.order_by(order).build().map_err(error)
}
