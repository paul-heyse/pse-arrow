// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual finite tuples are selected by native ordered domain/member joins.
use super::{
    CompilerError, Expr, JoinType, LogicalPlan, LogicalPlanBuilder, Outputs, Plans, array_element,
    array_length, c, coalesce, col, concat, distinct, emit, error, explode, filter, join, lit,
    prefix, project, scalar, union,
};
use datafusion::functions_aggregate::expr_fn::array_agg;
use datafusion::functions_aggregate::string_agg::string_agg;
use datafusion::{
    functions::encoding::expr_fn::encode, functions_aggregate::expr_fn::count,
    functions_nested::expr_fn::flatten, logical_expr::ExprFunctionExt,
};

pub(super) struct Indices {
    pub requirements: LogicalPlan,
    pub axes: LogicalPlan,
    pub pool: LogicalPlan,
}

#[expect(
    clippy::too_many_lines,
    reason = "build keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn build(
    plans: &mut Plans<'_>,
    outputs: &mut Outputs,
) -> Result<Indices, CompilerError> {
    let instances = plans.scan("normalized.instance_bindings", "owner")?;
    let bindings = plans.scan("normalized.instance_domain_bindings", "binding")?;
    let domains = plans.scan("normalized.domains", "domain")?;
    let members = plans.scan("normalized.domain_members", "member")?;
    let pool = join(
        instances.clone(),
        bindings,
        JoinType::Inner,
        [c("owner", "instance_id").eq(c("binding", "instance_id"))],
    )?;
    let pool = join(
        pool,
        domains,
        JoinType::Inner,
        [c("binding", "domain_id").eq(c("domain", "domain_id"))],
    )?;
    let pool = filter(pool, c("domain", "continuous").eq(lit(false)))?;
    let pool = join(
        pool,
        members,
        JoinType::Inner,
        [c("domain", "domain_id").eq(c("member", "domain_id"))],
    )?;
    let pool = project(
        pool,
        [
            c("owner", "instance_id").alias("state_instance_id"),
            c("domain", "domain_id").alias("domain_id"),
            c("domain", "kind").alias("kind"),
            c("member", "member_id").alias("member_id"),
            c("member", "ref_entity_id").alias("subject_id"),
            plans
                .lists(vec![
                    c("owner", "support"),
                    c("binding", "support"),
                    c("domain", "support"),
                    c("member", "support"),
                ])?
                .alias("supports"),
        ],
    )?;
    let pool = plans.retain(pool).await?;
    let properties = plans.scan("reference.property_kinds", "property")?;
    let base = join(instances, properties, JoinType::Inner, [lit(true)])?;
    let products = plans.scan("normalized.domain_products", "product")?;
    let base = join(
        base,
        products,
        JoinType::Inner,
        [array_length(c("property", "shape")).eq(array_length(c("product", "domain_ids")))],
    )?;
    let tuples = plans.scan("inferred.valid_index_tuples", "tuple")?;
    let base = join(
        base,
        tuples,
        JoinType::Inner,
        [c("product", "product_id").eq(c("tuple", "product_id"))],
    )?;
    let base = project(
        base,
        [
            c("owner", "instance_id").alias("state_instance_id"),
            c("property", "property_kind_id").alias("property_kind_id"),
            c("property", "shape").alias("shape"),
            c("product", "product_id").alias("product_id"),
            c("product", "domain_ids").alias("domain_ids"),
            c("tuple", "tuple").alias("index"),
            scalar::named_id(c("owner", "instance_id"), lit("pse:state-scope:v1"))
                .alias("state_scope_id"),
            plans
                .lists(vec![
                    c("owner", "support"),
                    c("property", "support"),
                    c("product", "support"),
                    c("tuple", "support"),
                ])?
                .alias("supports"),
        ],
    )?;
    let base = plans.retain(base).await?;
    let expanded = explode(base.clone(), col("index"), "axis_position")?;
    let expanded = join(
        expanded,
        prefix(pool.clone(), "axis")?,
        JoinType::Inner,
        [
            col("state_instance_id").eq(c("axis", "state_instance_id")),
            array_element(col("domain_ids"), col("axis_position") + lit(1_i64))
                .eq(c("axis", "domain_id")),
            array_element(col("index"), col("axis_position") + lit(1_i64))
                .eq(c("axis", "member_id")),
            array_element(col("shape"), col("axis_position") + lit(1_i64)).eq(c("axis", "kind")),
        ],
    )?;
    let expanded = plans.retain(expanded).await?;
    let keys = [
        "state_instance_id",
        "property_kind_id",
        "product_id",
        "index",
    ];
    let count = count(col("axis_position"))
        .distinct()
        .build()
        .map_err(error)?;
    let summary = LogicalPlanBuilder::from(expanded.clone())
        .aggregate(
            keys.into_iter().map(col),
            [
                count.alias("matched_axes"),
                array_agg(c("axis", "supports")).alias("axis_supports"),
            ],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let summary = prefix(summary, "matched")?;
    let joined = join(
        base,
        summary,
        JoinType::Left,
        keys.into_iter().map(|key| col(key).eq(c("matched", key))),
    )?;
    let joined = filter(
        joined,
        coalesce(vec![c("matched", "matched_axes"), lit(0_i64)]).eq(array_length(col("index"))),
    )?;
    let label_rows = distinct(project(
        expanded.clone(),
        [
            col("index"),
            col("axis_position"),
            c("axis", "member_id").alias("member_id"),
        ],
    )?)?;
    let encoding = string_agg(encode(col("member_id"), lit("hex")), lit(":"))
        .order_by(vec![col("axis_position").sort(true, false)])
        .build()
        .map_err(error)?;
    let labels = LogicalPlanBuilder::from(label_rows)
        .aggregate([col("index")], [encoding.alias("encoding")])
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let joined = join(
        joined,
        prefix(labels, "label")?,
        JoinType::Left,
        [col("index").eq(c("label", "index"))],
    )?;
    let requirements = project(
        joined,
        [
            scalar::named_id(
                col("state_scope_id"),
                concat(
                    concat(
                        lit("pse:property-requirement:v1:"),
                        encode(col("property_kind_id"), lit("hex")),
                    ),
                    concat(lit(":"), coalesce(vec![c("label", "encoding"), lit("")])),
                ),
            )
            .alias("requirement_id"),
            col("state_scope_id"),
            col("state_instance_id"),
            col("property_kind_id"),
            col("index"),
            col("product_id"),
            col("shape"),
            col("domain_ids"),
            plans
                .lists(vec![
                    col("supports"),
                    flatten(c("matched", "axis_supports")),
                ])?
                .alias("supports"),
        ],
    )?;
    let requirements = plans.retain(requirements).await?;
    emit(plans, outputs, "requirement_keys", requirements.clone()).await?;
    let axes = join(
        prefix(requirements.clone(), "requirement")?,
        expanded,
        JoinType::Inner,
        keys.into_iter()
            .map(|key| c("requirement", key).eq(col(key))),
    )?;
    plans
        .require(
            &axes,
            col("axis_position").lt_eq(lit(u16::MAX)),
            "requirement axis exceeds the declared 16-bit bound",
        )
        .await?;
    let axes = project(
        axes,
        [
            c("requirement", "requirement_id").alias("requirement_id"),
            Expr::Cast(datafusion::logical_expr::expr::Cast::new(
                Box::new(col("axis_position")),
                datafusion::arrow::datatypes::DataType::Int64,
            ))
            .alias("position"),
            c("axis", "domain_id").alias("domain_id"),
            c("axis", "member_id").alias("member_id"),
            c("axis", "kind").alias("kind"),
            c("axis", "subject_id").alias("subject_id"),
            plans
                .lists(vec![c("requirement", "supports"), c("axis", "supports")])?
                .alias("supports"),
        ],
    )?;
    let axes = plans.retain(axes).await?;
    emit(plans, outputs, "requirement_key_axes", axes.clone()).await?;
    scopes(plans, outputs, &requirements, &axes, &pool).await?;
    // Subsequent algorithms consume the admitted unique axis and requirement
    // values. Independent supports stay attached to their actual native producers.
    let axes = plans.scan("inferred.requirement_key_axes", "completed_axis")?;
    let axes = project(
        axes,
        [
            c("completed_axis", "requirement_id").alias("requirement_id"),
            c("completed_axis", "position").alias("position"),
            c("completed_axis", "domain_id").alias("domain_id"),
            c("completed_axis", "member_id").alias("member_id"),
            c("completed_axis", "kind").alias("kind"),
            c("completed_axis", "subject_id").alias("subject_id"),
            c("completed_axis", "support").alias("supports"),
        ],
    )?;
    let requirements = plans.scan("inferred.requirement_keys", "completed_requirement")?;
    let products = plans.scan("normalized.domain_products", "completed_product")?;
    let requirements = join(
        requirements,
        products,
        JoinType::Inner,
        [c("completed_requirement", "product_id").eq(c("completed_product", "product_id"))],
    )?;
    let mut columns = [
        "requirement_id",
        "state_scope_id",
        "state_instance_id",
        "property_kind_id",
        "index",
        "product_id",
        "shape",
    ]
    .into_iter()
    .map(|name| c("completed_requirement", name).alias(name))
    .collect::<Vec<_>>();
    columns.push(c("completed_product", "domain_ids").alias("domain_ids"));
    columns.push(
        plans
            .lists(vec![
                c("completed_requirement", "support"),
                c("completed_product", "support"),
            ])?
            .alias("supports"),
    );
    let requirements = project(requirements, columns)?;
    Ok(Indices {
        requirements,
        axes,
        pool,
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "scopes keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn scopes(
    plans: &mut Plans<'_>,
    outputs: &mut Outputs,
    requirements: &LogicalPlan,
    axes: &LogicalPlan,
    pool: &LogicalPlan,
) -> Result<(), CompilerError> {
    let package = project(
        requirements.clone(),
        [
            col("requirement_id"),
            plans
                .enum_literal("ScopeKind", "package")?
                .alias("scope_kind"),
            plans.index(scalar::id_list(vec![]))?.alias("scope_ids"),
            col("supports"),
        ],
    )?;
    let subjects = filter(
        axes.clone(),
        col("subject_id").is_not_null().and(
            col("kind")
                .eq(lit("phase"))
                .or(col("kind").eq(lit("species")))
                .or(col("kind").eq(lit("reaction"))),
        ),
    )?;
    let scope_kind = pse_catalog::session::output::same_field_case(
        subjects.schema(),
        col("kind").eq(lit("species")),
        plans.enum_literal("ScopeKind", "species")?,
        plans.enum_literal("ScopeKind", "reaction")?,
    )
    .map_err(error)?;
    let scope_kind = pse_catalog::session::output::same_field_case(
        subjects.schema(),
        col("kind").eq(lit("phase")),
        plans.enum_literal("ScopeKind", "phase")?,
        scope_kind,
    )
    .map_err(error)?;
    let subjects = project(
        subjects,
        [
            col("requirement_id"),
            scope_kind.alias("scope_kind"),
            plans
                .index(scalar::id_list(vec![col("subject_id")]))?
                .alias("scope_ids"),
            col("supports"),
        ],
    )?;
    let phase_requirements = distinct(project(
        filter(axes.clone(), col("kind").eq(lit("phase")))?,
        [col("requirement_id")],
    )?)?;
    let unbound = join(
        requirements.clone(),
        prefix(phase_requirements, "phase_bound")?,
        JoinType::LeftAnti,
        [col("requirement_id").eq(c("phase_bound", "requirement_id"))],
    )?;
    let phases = filter(
        pool.clone(),
        col("kind")
            .eq(lit("phase"))
            .and(col("subject_id").is_not_null()),
    )?;
    let phases = join(
        unbound,
        prefix(phases, "phase")?,
        JoinType::Inner,
        [col("state_instance_id").eq(c("phase", "state_instance_id"))],
    )?;
    let phases = project(
        phases,
        [
            col("requirement_id"),
            plans
                .enum_literal("ScopeKind", "phase")?
                .alias("scope_kind"),
            plans
                .index(scalar::id_list(vec![c("phase", "subject_id")]))?
                .alias("scope_ids"),
            plans
                .lists(vec![col("supports"), c("phase", "supports")])?
                .alias("supports"),
        ],
    )?;
    let scopes = plans
        .retain(union(vec![package, subjects, phases])?)
        .await?;
    let phase = filter(scopes.clone(), col("scope_kind").eq(lit("phase")))?;
    let species = prefix(
        filter(scopes.clone(), col("scope_kind").eq(lit("species")))?,
        "species",
    )?;
    let pairs = join(
        phase,
        species,
        JoinType::Inner,
        [col("requirement_id").eq(c("species", "requirement_id"))],
    )?;
    let pairs = project(
        pairs,
        [
            col("requirement_id"),
            plans
                .enum_literal("ScopeKind", "phase_species")?
                .alias("scope_kind"),
            plans
                .ids(vec![col("scope_ids"), c("species", "scope_ids")])?
                .alias("scope_ids"),
            plans
                .lists(vec![col("supports"), c("species", "supports")])?
                .alias("supports"),
        ],
    )?;
    emit(
        plans,
        outputs,
        "requirement_scope_keys",
        union(vec![scopes, pairs])?,
    )
    .await
}
