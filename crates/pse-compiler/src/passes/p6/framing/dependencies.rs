// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite tagged maps retain exact source axes and target declarations.
mod parameters;
use super::{
    CompilerError, Expr, JoinType, LogicalPlan, LogicalPlanBuilder, Not, Outputs, Plans,
    ScalarValue, append, array_element, array_length, c, coalesce, col, emit, error, filter,
    indices, join, lit, prefix, project, scalar, union, when,
};
use datafusion::logical_expr::ExprSchemable;
use datafusion::{functions::core::expr_fn::get_field, functions_nested::expr_fn::make_array};

pub(super) async fn build(
    plans: &mut Plans<'_>,
    indices: &indices::Indices,
    outputs: &mut Outputs,
) -> Result<(), CompilerError> {
    let provisions = plans.scan("reference.method_provisions", "provision")?;
    let base = join(
        indices.requirements.clone(),
        provisions,
        JoinType::Inner,
        [col("property_kind_id").eq(c("provision", "property_kind_id"))],
    )?;
    let base = project(
        base,
        [
            col("requirement_id"),
            col("state_instance_id"),
            col("index").alias("source_index"),
            col("domain_ids").alias("source_domains"),
            col("shape").alias("source_shape"),
            c("provision", "method_id").alias("method_id"),
            plans
                .lists(vec![col("supports"), c("provision", "support")])?
                .alias("supports"),
        ],
    )?;
    let base = plans.retain(base).await?;
    let dependencies = plans.scan("reference.method_dependencies", "dependency")?;
    let dependencies = join(
        base.clone(),
        dependencies,
        JoinType::Inner,
        [col("method_id").eq(c("dependency", "method_id"))],
    )?;
    let properties = plans.scan("reference.property_kinds", "target_property")?;
    let property = join(
        filter(
            dependencies.clone(),
            c("dependency", "target_kind").eq(lit("property")),
        )?,
        properties,
        JoinType::Left,
        [c("dependency", "target_id").eq(c("target_property", "property_kind_id"))],
    )?;
    plans
        .require(
            &property,
            c("target_property", "property_kind_id").is_not_null(),
            "dependency property target absent",
        )
        .await?;
    let property = project(
        property,
        columns(
            plans,
            c("target_property", "shape"),
            scalar::id_list(vec![]),
            c("target_property", "support"),
        )?,
    )?;
    let symbols = plans.scan("normalized.template_symbols", "target_symbol")?;
    let state = join(
        filter(
            dependencies,
            c("dependency", "target_kind").eq(lit("state_symbol")),
        )?,
        symbols,
        JoinType::Left,
        [c("dependency", "target_id").eq(c("target_symbol", "symbol_decl_id"))],
    )?;
    plans
        .require(
            &state,
            c("target_symbol", "symbol_decl_id").is_not_null(),
            "state dependency declaration absent",
        )
        .await?;
    let state = project(
        state,
        columns(
            plans,
            c("target_symbol", "indexed_by"),
            scalar::id_list(vec![]),
            c("target_symbol", "support"),
        )?,
    )?;
    // For symbols, target_shape temporarily names axes; shared binding supplies the
    // exact owner/domain vector. The per-axis join below reads its actual domain kind.
    let state = plans
        .named_axes(
            state,
            &["requirement_id", "method_id", "dependency_ordinal"],
            "state_instance_id",
            "target_shape",
            "symbol_domains",
        )
        .await?;
    let mut state_columns = state
        .schema()
        .columns()
        .into_iter()
        .filter(|field| field.name != "target_domains" && field.name != "symbol_domains")
        .map(Expr::Column)
        .collect::<Vec<_>>();
    state_columns.push(col("symbol_domains").alias("target_domains"));
    let state = project(state, state_columns)?;
    for (kind, target) in [("property", property), ("state_symbol", state)] {
        plans
            .require(
                &target,
                array_length(col("index_map")).eq(array_length(col("target_shape"))),
                "dependency map does not bind every target axis",
            )
            .await?;
        let target = expand(
            plans,
            target,
            &indices.axes,
            &indices.pool,
            kind == "state_symbol",
        )
        .await?;
        if kind == "property" {
            let target = join(
                target,
                prefix(indices.requirements.clone(), "target")?,
                JoinType::Inner,
                [
                    col("state_instance_id").eq(c("target", "state_instance_id")),
                    col("target_id").eq(c("target", "property_kind_id")),
                    col("mapped_index").eq(c("target", "index")),
                    col("mapped_domains").eq(c("target", "domain_ids")),
                ],
            )?;
            let target = project(
                target,
                [
                    col("requirement_id"),
                    col("method_id"),
                    col("dependency_ordinal"),
                    c("target", "requirement_id").alias("target_requirement_id"),
                    plans
                        .lists(vec![col("supports"), c("target", "supports")])?
                        .alias("supports"),
                ],
            )?;
            emit(plans, outputs, "dependency_key_maps", target).await?;
        } else {
            let products = plans.scan("normalized.domain_products", "target_product")?;
            let target = join(
                target,
                products,
                JoinType::Inner,
                [col("mapped_domains").eq(c("target_product", "domain_ids"))],
            )?;
            let target = project(
                target,
                [
                    col("requirement_id"),
                    col("method_id"),
                    col("dependency_ordinal"),
                    col("target_id").alias("symbol_decl_id"),
                    c("target_product", "product_id").alias("product_id"),
                    plans.index(col("mapped_index"))?.alias("index"),
                    plans
                        .lists(vec![col("supports"), c("target_product", "support")])?
                        .alias("supports"),
                ],
            )?;
            emit(plans, outputs, "state_dependency_keys", target).await?;
        }
    }
    parameters::build(plans, base, indices, outputs).await
}

fn columns(
    plans: &Plans<'_>,
    shape: Expr,
    domains: Expr,
    support: Expr,
) -> Result<Vec<Expr>, CompilerError> {
    Ok(vec![
        col("requirement_id"),
        col("state_instance_id"),
        col("method_id"),
        col("source_index"),
        col("source_domains"),
        col("source_shape"),
        c("dependency", "ordinal").alias("dependency_ordinal"),
        c("dependency", "target_id").alias("target_id"),
        c("dependency", "index_map").alias("index_map"),
        shape.alias("target_shape"),
        domains.alias("target_domains"),
        plans
            .lists(vec![col("supports"), c("dependency", "support"), support])?
            .alias("supports"),
    ])
}

async fn expand(
    plans: &mut Plans<'_>,
    base: LogicalPlan,
    axes: &LogicalPlan,
    pool: &LogicalPlan,
    symbol: bool,
) -> Result<LogicalPlan, CompilerError> {
    let initial = append(
        base,
        [
            lit(0_i64).alias("map_position"),
            scalar::id_list(vec![]).alias("mapped_index"),
            scalar::id_list(vec![]).alias("mapped_domains"),
            make_array(vec![lit(-1_i64)]).alias("used_source_positions"),
        ],
    )?;
    let mut frontier = plans.retain(initial).await?;
    let mut finished = Vec::new();
    loop {
        plans.cancel.checkpoint()?;
        finished.push(filter(
            frontier.clone(),
            col("map_position").eq(array_length(col("index_map"))),
        )?);
        let (pending, count) = plans
            .retain_with_count(filter(
                frontier,
                col("map_position").lt(array_length(col("index_map"))),
            )?)
            .await?;
        if count == 0 {
            break;
        }
        let step = append(
            pending,
            [array_element(col("index_map"), col("map_position") + lit(1_i64)).alias("axis_map")],
        )?;
        let step = plans.session.derive_plan_fields(step, plans.cancel)?;
        let tag = get_field(col("axis_map"), "kind");
        let position = get_field(col("axis_map"), "source_axis");
        let member = get_field(col("axis_map"), "member_id");
        let kind = get_field(col("axis_map"), "domain_kind");
        let valid = tag
            .clone()
            .eq(lit("source_axis"))
            .and(position.clone().is_not_null())
            .and(member.clone().is_null())
            .or(tag
                .clone()
                .eq(lit("fixed_member"))
                .and(position.clone().is_null())
                .and(member.clone().is_not_null()))
            .or(tag
                .clone()
                .eq(lit("bound_domain"))
                .and(position.clone().is_null())
                .and(member.clone().is_null()));
        plans
            .require(
                &step,
                valid,
                "dependency axis map has missing or extraneous payload",
            )
            .await?;
        if !symbol {
            plans
                .require(
                    &step,
                    kind.clone().eq(array_element(
                        col("target_shape"),
                        col("map_position") + lit(1_i64),
                    )),
                    "dependency map changes its target domain kind",
                )
                .await?;
        }
        let repeated = datafusion::functions_nested::expr_fn::array_has(
            col("used_source_positions"),
            position.clone(),
        );
        plans
            .require(
                &step,
                tag.clone().not_eq(lit("source_axis")).or(repeated.not()),
                "dependency map repeats a source axis",
            )
            .await?;
        let step = join(
            step,
            prefix(axes.clone(), "source_axis")?,
            JoinType::Left,
            [
                col("requirement_id").eq(c("source_axis", "requirement_id")),
                position.clone().eq(c("source_axis", "position")),
            ],
        )?;
        plans
            .require(
                &step,
                tag.clone()
                    .not_eq(lit("source_axis"))
                    .or(c("source_axis", "member_id")
                        .is_not_null()
                        .and(kind.clone().eq(c("source_axis", "kind")))),
                "dependency source axis is absent or has a different domain kind",
            )
            .await?;
        let mut on = vec![
            col("state_instance_id").eq(c("axis", "state_instance_id")),
            kind.eq(c("axis", "kind")),
            tag.clone()
                .not_eq(lit("source_axis"))
                .or(c("source_axis", "domain_id")
                    .eq(c("axis", "domain_id"))
                    .and(c("source_axis", "member_id").eq(c("axis", "member_id")))),
            tag.clone()
                .not_eq(lit("fixed_member"))
                .or(member.eq(c("axis", "member_id"))),
        ];
        if symbol {
            on.push(
                array_element(col("target_domains"), col("map_position") + lit(1_i64))
                    .eq(c("axis", "domain_id")),
            );
        }
        let step = join(step, prefix(pool.clone(), "axis")?, JoinType::Inner, on)?;
        let mut columns = [
            "requirement_id",
            "state_instance_id",
            "method_id",
            "source_index",
            "source_domains",
            "source_shape",
            "dependency_ordinal",
            "target_id",
            "index_map",
            "target_shape",
            "target_domains",
        ]
        .into_iter()
        .map(col)
        .collect::<Vec<_>>();
        columns.extend([
            plans
                .lists(vec![
                    col("supports"),
                    c("axis", "supports"),
                    c("source_axis", "supports"),
                ])?
                .alias("supports"),
            (col("map_position") + lit(1_i64)).alias("map_position"),
            plans
                .ids(vec![
                    col("mapped_index"),
                    scalar::id_list(vec![c("axis", "member_id")]),
                ])?
                .alias("mapped_index"),
            plans
                .ids(vec![
                    col("mapped_domains"),
                    scalar::id_list(vec![c("axis", "domain_id")]),
                ])?
                .alias("mapped_domains"),
            plans
                .lists(vec![
                    col("used_source_positions"),
                    when(
                        tag.eq(lit("source_axis")),
                        make_array(vec![
                            position
                                .cast_to(
                                    &datafusion::arrow::datatypes::DataType::Int64,
                                    step.schema(),
                                )
                                .map_err(error)?,
                        ]),
                    )
                    .otherwise(lit(ScalarValue::Null))
                    .map_err(error)?,
                ])?
                .alias("used_source_positions"),
        ]);
        frontier = plans.retain(project(step, columns)?).await?;
    }
    plans.retain(union(finished)?).await
}
