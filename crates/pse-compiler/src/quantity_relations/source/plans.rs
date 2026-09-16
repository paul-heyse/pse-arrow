// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native relational selection of the admitted P7–P9 physical carriers.
use super::{CompilerError, invalid};
use datafusion::{
    catalog::cte_worktable::CteWorkTable,
    datasource::provider_as_source,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use std::sync::Arc;

pub(super) fn engine(error: datafusion::common::DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}
pub(super) fn scan(
    session: &SnapshotSession,
    name: &str,
    alias: &str,
) -> Result<LogicalPlan, CompilerError> {
    let spec = session
        .registry()
        .relation(name)
        .ok_or_else(|| invalid(format!("quantity source {name} is undeclared")))?;
    LogicalPlanBuilder::scan(
        session.table_reference(&spec.key)?,
        session.table_source(&spec.key)?,
        None,
    )
    .and_then(|plan| plan.alias(alias))
    .and_then(LogicalPlanBuilder::build)
    .map_err(engine)
}
pub(super) fn project(
    input: LogicalPlan,
    fields: impl IntoIterator<Item = Expr>,
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .project(fields)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}
fn union(mut plans: Vec<LogicalPlan>) -> Result<LogicalPlan, CompilerError> {
    let mut result = plans
        .pop()
        .ok_or_else(|| invalid("native quantity union has no declared sources"))?;
    for plan in plans {
        result = LogicalPlanBuilder::from(result)
            .union(plan)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
    }
    Ok(result)
}
fn join(
    left: LogicalPlan,
    right: LogicalPlan,
    kind: JoinType,
    on: Vec<Expr>,
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(left)
        .join_on(right, kind, on)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}
fn alias(input: LogicalPlan, name: &str) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .alias(name)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

/// Explicit group quantities seed a native finite closure. Projection and reindexing
/// preserve all physical axes except shape; the exact destination factors supply shape.
pub(super) fn groups(
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<LogicalPlan, CompilerError> {
    let mut seeds = Vec::new();
    for name in [
        "compiled.predicate_masks",
        "compiled.group_collections",
        "compiled.port_member_groups",
        "compiled.element_projection_groups",
    ] {
        seeds.push(project(
            scan(session, name, "carrier")?,
            [
                col("carrier.group_id").alias("group_id"),
                col("carrier.quantity_type_id").alias("quantity_type_id"),
            ],
        )?);
    }
    let mut edges = Vec::new();
    for name in ["compiled.group_projections", "compiled.group_reindexings"] {
        edges.push(project(
            scan(session, name, "edge")?,
            [
                col("edge.group_id").alias("group_id"),
                col("edge.source_group_id").alias("source_group_id"),
            ],
        )?);
    }
    let edges = union(edges)?;
    let special = union(vec![
        project(union(seeds.clone())?, [col("group_id")])?,
        project(edges.clone(), [col("group_id")])?,
    ])?;
    let ordinary = join(
        scan(session, "compiled.symbol_groups", "g")?,
        alias(special, "special")?,
        JoinType::LeftAnti,
        vec![col("g.group_id").eq(col("special.group_id"))],
    )?;
    let ordinary = join(
        ordinary,
        scan(session, "inferred.instances", "i")?,
        JoinType::Inner,
        vec![col("g.owner_instance_id").eq(col("i.instance_id"))],
    )?;
    let ordinary = join(
        ordinary,
        scan(session, "normalized.template_symbols", "declaration")?,
        JoinType::Inner,
        vec![
            col("i.template_id").eq(col("declaration.template_id")),
            col("g.name").eq(col("declaration.name")),
        ],
    )?;
    seeds.push(project(
        ordinary,
        [
            col("g.group_id").alias("group_id"),
            col("declaration.quantity_type_id").alias("quantity_type_id"),
        ],
    )?);
    let seed = project(
        union(seeds)?,
        [
            col("group_id"),
            col("quantity_type_id"),
            col("group_id").alias("origin_group_id"),
        ],
    )?;
    let seed = pse_catalog::session::output::forget_relation_annotations(seed).map_err(engine)?;
    let seed = session.derive_plan_fields(seed, cancel)?;
    let name = "physical_group_quantity_ancestry";
    let work = LogicalPlanBuilder::scan(
        name,
        provider_as_source(Arc::new(CteWorkTable::new(
            name,
            Arc::new(seed.schema().as_arrow().clone()),
        ))),
        None,
    )
    .and_then(|plan| plan.alias("known"))
    .and_then(LogicalPlanBuilder::build)
    .map_err(engine)?;
    let step = join(
        alias(edges, "edge")?,
        work,
        JoinType::Inner,
        vec![col("edge.source_group_id").eq(col("known.group_id"))],
    )?;
    let step = project(
        step,
        [
            col("edge.group_id").alias("group_id"),
            col("known.quantity_type_id").alias("quantity_type_id"),
            col("known.origin_group_id").alias("origin_group_id"),
        ],
    )?;
    let step = pse_catalog::session::output::forget_relation_annotations(step).map_err(engine)?;
    let quantities = LogicalPlanBuilder::from(seed)
        .to_recursive_query(name.to_owned(), step, true)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let products = union(vec![
        project(
            scan(session, "normalized.domain_products", "product")?,
            [
                col("product.product_id").alias("product_id"),
                col("product.domain_ids").alias("domain_ids"),
            ],
        )?,
        project(
            scan(session, "compiled.element_projection_groups", "element")?,
            [
                col("element.product_id").alias("product_id"),
                col("element.domain_ids").alias("domain_ids"),
            ],
        )?,
    ])?;
    let products = LogicalPlanBuilder::from(products)
        .distinct()
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let groups = join(
        scan(session, "compiled.symbol_groups", "g")?,
        alias(quantities, "quantity")?,
        JoinType::Left,
        vec![col("g.group_id").eq(col("quantity.group_id"))],
    )?;
    let groups = join(
        groups,
        alias(products, "product")?,
        JoinType::Left,
        vec![col("g.product_id").eq(col("product.product_id"))],
    )?;
    let groups = project(
        groups,
        [
            col("g.group_id").alias("group_id"),
            col("quantity.quantity_type_id").alias("quantity_type_id"),
            col("quantity.origin_group_id").alias("origin_group_id"),
            col("product.domain_ids").alias("domain_ids"),
        ],
    )?;
    LogicalPlanBuilder::from(groups)
        .sort([col("group_id").sort(true, false)])
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn domain_members(session: &SnapshotSession) -> Result<LogicalPlan, CompilerError> {
    let joined = join(
        scan(session, "normalized.domains", "domain")?,
        scan(session, "normalized.domain_members", "member")?,
        JoinType::Inner,
        vec![col("domain.domain_id").eq(col("member.domain_id"))],
    )?;
    LogicalPlanBuilder::from(joined)
        .sort([
            col("domain.domain_id").sort(true, false),
            col("member.ordinal").sort(true, false),
        ])
        .and_then(|plan| {
            plan.project([
                col("domain.domain_id").alias("domain_id"),
                col("member.member_id").alias("member_id"),
            ])
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn group_members(session: &SnapshotSession) -> Result<LogicalPlan, CompilerError> {
    let joined = join(
        scan(session, "compiled.symbol_group_members", "member")?,
        scan(session, "compiled.symbols", "symbol")?,
        JoinType::Left,
        vec![col("member.symbol_id").eq(col("symbol.symbol_id"))],
    )?;
    LogicalPlanBuilder::from(joined)
        .sort([
            col("member.group_id").sort(true, false),
            col("member.tuple").sort(true, false),
        ])
        .and_then(|plan| {
            plan.project([
                col("member.group_id").alias("group_id"),
                col("member.tuple").alias("tuple"),
                col("symbol.symbol_id").alias("symbol_id"),
                col("symbol.quantity_type_id").alias("quantity_type_id"),
            ])
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn unknowns(session: &SnapshotSession) -> Result<LogicalPlan, CompilerError> {
    use datafusion::common::UnnestOptions;
    use datafusion::functions_nested::expr_fn::{array_element, array_length, range};
    let list = col("unknown_symbol_ids");
    let source = project(
        scan(session, "inferred.math_implicit_systems", "system")?,
        [
            col("system.implicit_system_id").alias("implicit_system_id"),
            col("system.unknown_symbol_ids").alias("unknown_symbol_ids"),
        ],
    )?;
    let source = LogicalPlanBuilder::from(source)
        .project([
            col("implicit_system_id"),
            list.clone(),
            range(lit(0_i64), array_length(list), lit(1_i64)).alias("ordinal"),
        ])
        .and_then(|plan| {
            plan.unnest_column_with_options(
                "ordinal",
                UnnestOptions::new().with_preserve_nulls(false),
            )
        })
        .and_then(|plan| {
            plan.project([
                col("implicit_system_id"),
                col("ordinal"),
                array_element(col("unknown_symbol_ids"), col("ordinal") + lit(1_i64))
                    .alias("symbol_id"),
            ])
        })
        .and_then(|plan| plan.alias("unknown"))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let joined = join(
        source,
        scan(session, "compiled.symbols", "symbol")?,
        JoinType::Left,
        vec![col("unknown.symbol_id").eq(col("symbol.symbol_id"))],
    )?;
    project(
        joined,
        [
            col("unknown.implicit_system_id").alias("implicit_system_id"),
            col("unknown.ordinal").alias("ordinal"),
            col("symbol.quantity_type_id").alias("quantity_type_id"),
        ],
    )
}
