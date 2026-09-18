// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native containment traversal terminates on actual visited identities and rejects cycles.
use super::native::{Plans, c, distinct, error, join, prefix, project, union};
use crate::CompilerError;
use datafusion::{
    catalog::cte_worktable::CteWorkTable,
    datasource::provider_as_source,
    functions_nested::expr_fn::{array_has, array_length},
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit, when},
};
use pse_catalog::session::scalar;
use std::{ops::Not, sync::Arc};

#[expect(
    clippy::too_many_lines,
    reason = "build keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn build(
    plans: &mut Plans<'_>,
    shapes: LogicalPlan,
) -> Result<(LogicalPlan, LogicalPlan), CompilerError> {
    let instances = plans.scan("normalized.instance_bindings", "instance")?;
    let parents = plans.scan("normalized.instance_bindings", "parent")?;
    let base = join(
        instances,
        parents,
        JoinType::Left,
        [c("instance", "parent_instance_id").eq(c("parent", "instance_id"))],
    )?;
    plans
        .require(
            &base,
            c("instance", "parent_instance_id")
                .is_null()
                .or(c("parent", "instance_id").is_not_null()),
            "prospective instance parent is absent",
        )
        .await?;
    let declarations = plans.scan("normalized.template_submodels", "declaration")?;
    let base = join(
        base,
        declarations,
        JoinType::Left,
        [
            c("parent", "template_id").eq(c("declaration", "template_id")),
            c("instance", "submodel_name").eq(c("declaration", "name")),
        ],
    )?;
    plans
        .require(
            &base,
            c("instance", "submodel_name")
                .is_null()
                .or(c("declaration", "template_id").is_not_null()),
            "prospective child has no exact declaration",
        )
        .await?;
    let bindings = plans.scan("normalized.instance_domain_bindings", "binding")?;
    let base = join(
        base,
        bindings,
        JoinType::Left,
        [
            c("parent", "instance_id").eq(c("binding", "instance_id")),
            c("declaration", "multiplicity_domain").eq(c("binding", "domain_name")),
        ],
    )?;
    plans
        .require(
            &base,
            c("declaration", "multiplicity_domain")
                .is_null()
                .or(c("binding", "domain_id").is_not_null()),
            "child multiplicity domain has no actual parent binding",
        )
        .await?;
    let factors = scalar::id_list(vec![c("binding", "domain_id")]);
    plans
        .require(
            &base,
            array_length(c("instance", "index")).eq(array_length(factors.clone())),
            "prospective child index and declared multiplicity arity differ",
        )
        .await?;
    let edges = project(
        base,
        [
            c("instance", "instance_id").alias("node"),
            c("instance", "parent_instance_id").alias("parent"),
            factors.alias("factors"),
            c("instance", "index").alias("index"),
            plans
                .lists(vec![
                    c("instance", "support"),
                    c("parent", "support"),
                    c("declaration", "support"),
                    c("binding", "support"),
                ])?
                .alias("supports"),
            c("instance", "support").alias("own_support"),
        ],
    )?;
    let edges = plans.retain(edges).await?;
    let bindings = project(
        edges.clone(),
        [
            col("node").alias("instance_id"),
            col("factors"),
            col("index"),
            col("supports"),
        ],
    )?;
    let seed = project(
        edges.clone(),
        [
            col("node").alias("owner"),
            col("parent"),
            col("factors"),
            scalar::id_list(vec![col("node")]).alias("visited"),
            col("supports"),
        ],
    )?;
    let seed = plans.session.derive_plan_fields(seed, plans.cancel)?;
    let name = "p3_product_ancestry";
    let work = LogicalPlanBuilder::scan(
        name,
        provider_as_source(Arc::new(CteWorkTable::new(
            name,
            Arc::new(seed.schema().as_arrow().clone()),
        ))),
        None,
    )
    .and_then(LogicalPlanBuilder::build)
    .map_err(error)?;
    let step = join(
        prefix(work, "known")?,
        prefix(edges.clone(), "edge")?,
        JoinType::Inner,
        [
            c("known", "parent").eq(c("edge", "node")),
            array_has(c("known", "visited"), c("edge", "node")).not(),
        ],
    )?;
    let step = project(
        step,
        [
            c("known", "owner").alias("owner"),
            c("edge", "parent").alias("parent"),
            plans
                .ids(vec![c("edge", "factors"), c("known", "factors")])?
                .alias("factors"),
            plans
                .ids(vec![
                    c("known", "visited"),
                    scalar::id_list(vec![c("edge", "node")]),
                ])?
                .alias("visited"),
            plans
                .lists(vec![c("known", "supports"), c("edge", "supports")])?
                .alias("supports"),
        ],
    )?;
    let closure = LogicalPlanBuilder::from(seed)
        .to_recursive_query(name.to_owned(), step, true)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let closure = plans.retain(closure).await?;
    plans
        .require(
            &closure,
            when(col("parent").is_null(), lit(true))
                .otherwise(array_has(col("visited"), col("parent")).not())
                .map_err(error)?,
            "prospective containment cycles",
        )
        .await?;
    let ancestry = union(vec![
        project(closure, [col("owner"), col("factors"), col("supports")])?,
        project(
            edges,
            [
                col("node").alias("owner"),
                scalar::id_list(vec![]).alias("factors"),
                col("own_support").alias("supports"),
            ],
        )?,
    ])?;
    let combined = join(
        prefix(shapes, "shape")?,
        prefix(ancestry, "ancestry")?,
        JoinType::Inner,
        [c("shape", "owner").eq(c("ancestry", "owner"))],
    )?;
    let full = project(
        combined,
        [
            plans
                .ids(vec![c("ancestry", "factors"), c("shape", "factors")])?
                .alias("factors"),
            plans
                .lists(vec![c("ancestry", "supports"), c("shape", "supports")])?
                .alias("supports"),
        ],
    )?;
    Ok((plans.retain(distinct(full)?).await?, bindings))
}
