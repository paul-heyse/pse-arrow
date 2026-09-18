// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered law axes come from declared names and their actual instance bindings.
use super::native::{Sources, append, c, error, join, project, require};
use crate::CompilerError;
use datafusion::functions_aggregate::expr_fn::array_agg;
use datafusion::functions_nested::expr_fn::array_element;
use datafusion::{
    common::UnnestOptions,
    functions::core::expr_fn::coalesce,
    functions_nested::expr_fn::{array_length, range},
    logical_expr::{ExprFunctionExt, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::scalar;
use pse_ids::CancellationToken;

#[expect(
    clippy::too_many_lines,
    reason = "apply keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn apply(
    base: LogicalPlan,
    sources: &mut Sources<'_>,
    cancel: &CancellationToken,
) -> Result<LogicalPlan, CompilerError> {
    // Source joins may retain several agreeing default-state witnesses. The ordered
    // factor vector is a function of the exact declaration and owner, not that count.
    let axis_seed = project(
        base.clone(),
        [
            c("instance", "instance_id").alias("axis_owner"),
            c("law", "law_instance_decl_id").alias("axis_declaration"),
            c("contract", "indexed_by").alias("axis_names"),
        ],
    )?;
    let axis_seed = LogicalPlanBuilder::from(axis_seed)
        .distinct()
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let axis_seed = append(
        axis_seed,
        [range(lit(0_i64), array_length(col("axis_names")), lit(1_i64)).alias("axis_position")],
    )?;
    let axis_seed = LogicalPlanBuilder::from(axis_seed)
        .unnest_column_with_options(
            "axis_position",
            UnnestOptions::new().with_preserve_nulls(false),
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let bindings = sources.scan("normalized.instance_domain_bindings", "axis_binding")?;
    let axes = join(
        axis_seed,
        bindings,
        JoinType::Left,
        [
            col("axis_owner").eq(c("axis_binding", "instance_id")),
            array_element(col("axis_names"), col("axis_position") + lit(1_i64))
                .eq(c("axis_binding", "domain_name")),
        ],
    )?;
    require(
        &axes,
        c("axis_binding", "domain_id").is_not_null(),
        "law axis has no actual instance domain binding",
        sources.session,
        cancel,
    )
    .await?;
    let axes = append(
        axes,
        [sources
            .session
            .scalar_function("pse_require_nonnull")?
            .call(vec![c("axis_binding", "domain_id")])
            .alias("axis_domain")],
    )?;
    let domains = LogicalPlanBuilder::from(axes.clone())
        .aggregate(
            [col("axis_owner"), col("axis_declaration")],
            [array_agg(col("axis_domain"))
                .order_by(vec![col("axis_position").sort(true, false)])
                .build()
                .map_err(error)?
                .alias("axis_domains")],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let domains = project(
        domains,
        [
            col("axis_owner").alias("product_owner"),
            col("axis_declaration").alias("product_declaration"),
            col("axis_domains"),
        ],
    )?;
    let base = join(
        base,
        domains,
        JoinType::Left,
        [
            c("instance", "instance_id").eq(col("product_owner")),
            c("law", "law_instance_decl_id").eq(col("product_declaration")),
        ],
    )?;
    let base = append(
        base,
        [coalesce(vec![col("axis_domains"), scalar::id_list(vec![])]).alias("ordered_domains")],
    )?;
    let products = sources.scan("normalized.domain_products", "product")?;
    let base = join(
        base,
        products,
        JoinType::Left,
        [col("ordered_domains").eq(c("product", "domain_ids"))],
    )?;
    require(
        &base,
        c("product", "product_id").is_not_null(),
        "law has no exact ordered domain product",
        sources.session,
        cancel,
    )
    .await?;
    // Restore each exact axis binding row beside the context. The output projection
    // coalesces repeated context values; its support retains every matched axis key.
    join(
        base,
        axes,
        JoinType::Left,
        [
            c("instance", "instance_id").eq(col("axis_owner")),
            c("law", "law_instance_decl_id").eq(col("axis_declaration")),
        ],
    )
}
