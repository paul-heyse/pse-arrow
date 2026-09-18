// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite subset and Cartesian expansion; loop progress is an actual axis position.
#[cfg(test)]
mod tests;
use super::native::{
    Plans, c, concat, distinct, error, explode, filter, invalid, join, prefix, project,
};
use crate::CompilerError;
use datafusion::functions_nested::expr_fn::array_element;
use datafusion::{
    arrow::datatypes::DataType,
    catalog::cte_worktable::CteWorkTable,
    datasource::provider_as_source,
    functions::{core::expr_fn::coalesce, encoding::expr_fn::encode},
    functions_aggregate::string_agg::string_agg,
    functions_nested::expr_fn::{array_append, array_length, array_to_string, range},
    functions_nested::extract::array_slice,
    logical_expr::{
        Expr, ExprFunctionExt, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit, when,
    },
};
use pse_catalog::session::scalar;
use pse_ids::SemanticId;
use std::sync::Arc;

pub(super) async fn subsets(
    plans: &mut Plans<'_>,
    parents: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    plans
        .require(
            &parents,
            array_length(col("factors")).lt_eq(lit(65_536_u64)),
            "product position exceeds the declared UInt16 ordinal",
        )
        .await?;
    let seed = project(
        parents,
        [
            col("factors").alias("parent_factors"),
            scalar::id_list(vec![]).alias("factors"),
            range(lit(0_i64), lit(0_i64), lit(1_i64)).alias("positions"),
            lit(0_i64).alias("cursor"),
            col("supports"),
        ],
    )?;
    let seed = plans.session.derive_plan_fields(seed, plans.cancel)?;
    let name = "p3_ordered_product_subsets";
    let work = worktable(name, &seed)?;
    let work = filter(work, col("cursor").lt(array_length(col("parent_factors"))))?;
    let branch = LogicalPlanBuilder::values(vec![vec![lit(false)], vec![lit(true)]])
        .and_then(|plan| plan.project([col("column1").alias("include")]))
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let work = LogicalPlanBuilder::from(work)
        .cross_join(branch)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let appended = plans.ids(vec![
        col("factors"),
        array_slice(
            col("parent_factors"),
            col("cursor") + lit(1_i64),
            col("cursor") + lit(1_i64),
            None,
        ),
    ])?;
    let factors = pse_catalog::session::output::same_field_case(
        work.schema(),
        col("include"),
        appended,
        col("factors"),
    )
    .map_err(error)?;
    let step = project(
        work,
        [
            col("parent_factors"),
            factors.alias("factors"),
            when(
                col("include"),
                array_append(col("positions"), col("cursor")),
            )
            .otherwise(col("positions"))
            .map_err(error)?
            .alias("positions"),
            (col("cursor") + lit(1_i64)).alias("cursor"),
            col("supports"),
        ],
    )?;
    let closure = LogicalPlanBuilder::from(seed)
        .to_recursive_query(name.to_owned(), step, false)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let complete = filter(
        closure,
        col("cursor").eq(array_length(col("parent_factors"))),
    )?;
    plans
        .retain(project(
            complete,
            [
                col("parent_factors"),
                col("factors"),
                col("positions"),
                col("supports"),
            ],
        )?)
        .await
}

/// Identity framing exactly matches `templates::identity::domain_product_id`. The
/// ordered factor vector is retained and remains the equality authority.
pub(super) async fn labels(
    plans: &mut Plans<'_>,
    subsets: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let factors = distinct(project(subsets, [col("factors")])?)?;
    let rows = explode(factors.clone(), col("factors"), "factor_position")?;
    let encoding = string_agg(
        encode(
            array_element(col("factors"), col("factor_position") + lit(1_i64)),
            lit("hex"),
        ),
        lit(""),
    )
    .order_by(vec![col("factor_position").sort(true, false)])
    .build()
    .map_err(error)?;
    let names = LogicalPlanBuilder::from(rows)
        .aggregate([col("factors")], [encoding.alias("encoding")])
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let joined = join(
        factors,
        prefix(names, "label")?,
        JoinType::Left,
        [col("factors").eq(c("label", "factors"))],
    )?;
    let id = scalar::named_id(
        plans.sid(SemanticId::NIL)?,
        concat(
            lit("pse:domain-product:v1:"),
            coalesce(vec![c("label", "encoding"), lit("")]),
        ),
    );
    plans
        .retain(project(joined, [col("factors"), id.alias("product_id")])?)
        .await
}
pub(super) fn labelled(
    input: LogicalPlan,
    labels: LogicalPlan,
    factor: &str,
    name: &str,
) -> Result<LogicalPlan, CompilerError> {
    let labels = project(
        labels,
        [
            col("factors").alias(format!("{name}_factors")),
            col("product_id").alias(name),
        ],
    )?;
    join(
        input,
        labels,
        JoinType::Inner,
        [col(factor).eq(col(format!("{name}_factors")))],
    )
}
pub(super) fn projection(
    plans: &Plans<'_>,
    subsets: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let frame = concat(
        concat(
            lit("product-projection:["),
            coalesce(vec![array_to_string(col("positions"), lit(", ")), lit("")]),
        ),
        lit("]"),
    );
    let target = plans
        .session
        .registry()
        .relation("normalized.domain_product_projections")
        .ok_or_else(|| invalid("product projections undeclared"))?;
    let field = target
        .column("positions")
        .ok_or_else(|| invalid("product projection positions undeclared"))?;
    let arrow = pse_schema::arrow::relation_schema(plans.session.registry(), target)?;
    let data_type = arrow
        .field_with_name(field.name())
        .map_err(pse_relations::RelationError::from)?
        .data_type()
        .clone();
    let positions = Expr::Cast(datafusion::logical_expr::expr::Cast::new(
        Box::new(col("positions")),
        data_type,
    ));
    project(
        subsets,
        [
            scalar::named_id(col("parent_product_id"), frame).alias("projection_id"),
            col("product_id"),
            col("parent_product_id"),
            positions.alias("positions"),
            col("supports"),
        ],
    )
}
pub(super) async fn tuples(plans: &mut Plans<'_>) -> Result<LogicalPlan, CompilerError> {
    let products = plans.scan("normalized.domain_products", "product")?;
    let factors = explode(
        products.clone(),
        c("product", "domain_ids"),
        "factor_position",
    )?;
    let domains = plans.scan("normalized.domains", "domain")?;
    let factors = join(
        factors,
        domains.clone(),
        JoinType::Left,
        [array_element(
            c("product", "domain_ids"),
            col("factor_position") + lit(1_i64),
        )
        .eq(c("domain", "domain_id"))],
    )?;
    plans
        .require(
            &factors,
            c("domain", "domain_id").is_not_null(),
            "product factor domain absent",
        )
        .await?;
    let tuple = plans
        .session
        .scalar_function("pse_index_tuple")?
        .call(vec![scalar::id_list(vec![])]);
    let seed = project(
        products,
        [
            c("product", "product_id").alias("product_id"),
            c("product", "domain_ids").alias("factors"),
            lit(0_i64).alias("cursor"),
            tuple.alias("tuple"),
            c("product", "support").alias("supports"),
        ],
    )?;
    let seed = plans.session.derive_plan_fields(seed, plans.cancel)?;
    let name = "p3_cartesian_candidate_tuples";
    let work = filter(
        worktable(name, &seed)?,
        col("cursor").lt(array_length(col("factors"))),
    )?;
    let work = join(
        work,
        domains,
        JoinType::Inner,
        [
            array_element(col("factors"), col("cursor") + lit(1_i64)).eq(c("domain", "domain_id")),
            c("domain", "continuous").eq(lit(false)),
        ],
    )?;
    let members = plans.scan("normalized.domain_members", "member")?;
    let work = join(
        work,
        members,
        JoinType::Inner,
        [c("domain", "domain_id").eq(c("member", "domain_id"))],
    )?;
    let singleton = plans
        .session
        .scalar_function("pse_index_tuple")?
        .call(vec![scalar::id_list(vec![c("member", "member_id")])]);
    let step = project(
        work,
        [
            col("product_id"),
            col("factors"),
            (col("cursor") + lit(1_i64)).alias("cursor"),
            plans.ids(vec![col("tuple"), singleton])?.alias("tuple"),
            plans
                .lists(vec![
                    col("supports"),
                    c("domain", "support"),
                    c("member", "support"),
                ])?
                .alias("supports"),
        ],
    )?;
    let closure = LogicalPlanBuilder::from(seed)
        .to_recursive_query(name.to_owned(), step, false)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let complete = filter(closure, col("cursor").eq(array_length(col("factors"))))?;
    plans
        .retain(project(
            complete,
            [
                col("product_id"),
                col("factors"),
                col("tuple"),
                col("supports"),
            ],
        )?)
        .await
}
pub(super) fn members(input: LogicalPlan) -> Result<LogicalPlan, CompilerError> {
    let input = explode(input, col("factors"), "member_position")?;
    project(
        input,
        [
            col("product_id"),
            col("tuple"),
            Expr::Cast(datafusion::logical_expr::expr::Cast::new(
                Box::new(col("member_position")),
                DataType::Int64,
            ))
            .alias("position"),
            array_element(col("factors"), col("member_position") + lit(1_i64)).alias("domain_id"),
            array_element(col("tuple"), col("member_position") + lit(1_i64)).alias("member_id"),
            col("supports"),
        ],
    )
}
fn worktable(name: &str, seed: &LogicalPlan) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::scan(
        name,
        provider_as_source(Arc::new(CteWorkTable::new(
            name,
            Arc::new(seed.schema().as_arrow().clone()),
        ))),
        None,
    )
    .and_then(LogicalPlanBuilder::build)
    .map_err(error)
}
