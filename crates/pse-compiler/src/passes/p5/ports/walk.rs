// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite native frontiers advance through parsed source positions and actual children.
use super::{
    CompilerError, JoinType, LogicalPlan, Plans, array_length, c, col, filter, join, lit, project,
    scalar, union,
};

const PORT: [&str; 8] = [
    "port_id",
    "owner_instance_id",
    "owner_template_id",
    "name",
    "kind",
    "direction",
    "guard_outcome",
    "length",
];
#[expect(
    clippy::too_many_lines,
    reason = "resolve keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn resolve(
    plans: &mut Plans<'_>,
    base: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let mut seed = PORT.into_iter().map(col).collect::<Vec<_>>();
    seed.extend([
        col("owner_instance_id").alias("state_instance_id"),
        col("owner_template_id").alias("state_template_id"),
        lit(0_i64).alias("position"),
        plans
            .session
            .scalar_function("pse_index_tuple")?
            .call(vec![scalar::id_list(vec![])])
            .alias("index"),
        scalar::id_list(vec![]).alias("domain_ids"),
        col("supports"),
    ]);
    let mut frontier = plans.retain(project(base, seed)?).await?;
    let mut finished = Vec::new();
    loop {
        plans.cancel.checkpoint()?;
        finished.push(filter(frontier.clone(), col("position").eq(col("length")))?);
        let (active, count) = plans
            .retain_with_count(filter(frontier, col("position").lt(col("length")))?)
            .await?;
        if count == 0 {
            break;
        }
        let steps = plans.scan("normalized.port_binding_steps", "step")?;
        let active = join(
            active,
            steps,
            JoinType::Left,
            [
                col("owner_template_id").eq(c("step", "template_id")),
                col("name").eq(c("step", "name")),
                col("position").eq(c("step", "position")),
            ],
        )?;
        plans
            .require(
                &active,
                c("step", "child_name").is_not_null(),
                "port traversal has no parsed next step",
            )
            .await?;
        let declarations = plans.scan("normalized.template_submodels", "child_declaration")?;
        let active = join(
            active,
            declarations,
            JoinType::Left,
            [
                col("state_template_id").eq(c("child_declaration", "template_id")),
                c("step", "child_name").eq(c("child_declaration", "name")),
            ],
        )?;
        plans
            .require(
                &active,
                c("child_declaration", "name").is_not_null(),
                "port traversal names an undeclared child",
            )
            .await?;
        let children = plans.scan("normalized.instance_bindings", "child")?;
        let active = join(
            active,
            children,
            JoinType::Left,
            [
                col("state_instance_id").eq(c("child", "parent_instance_id")),
                c("step", "child_name").eq(c("child", "submodel_name")),
                col("state_template_id").eq(c("child", "submodel_template_id")),
            ],
        )?;
        plans
            .require(
                &active,
                c("child", "instance_id").is_not_null(),
                "port child has no actual prospective target",
            )
            .await?;
        let bindings = plans.scan("normalized.instance_domain_bindings", "axis")?;
        let active = join(
            active,
            bindings,
            JoinType::Left,
            [
                col("state_instance_id").eq(c("axis", "instance_id")),
                c("child_declaration", "multiplicity_domain").eq(c("axis", "domain_name")),
            ],
        )?;
        let factors = scalar::id_list(vec![c("axis", "domain_id")]);
        plans
            .require(
                &active,
                c("child_declaration", "multiplicity_domain")
                    .is_null()
                    .or(c("axis", "domain_id").is_not_null())
                    .and(array_length(c("child", "index")).eq(array_length(factors.clone()))),
                "port child has incompatible declared index dimensions",
            )
            .await?;
        let mut fields = PORT.into_iter().map(col).collect::<Vec<_>>();
        fields.extend([
            plans
                .present(c("child", "instance_id"))?
                .alias("state_instance_id"),
            plans
                .present(c("child", "template_id"))?
                .alias("state_template_id"),
            (col("position") + lit(1_i64)).alias("position"),
            plans
                .ids(vec![col("index"), c("child", "index")])?
                .alias("index"),
            plans
                .ids(vec![col("domain_ids"), factors])?
                .alias("domain_ids"),
            plans
                .lists(vec![
                    col("supports"),
                    c("step", "support"),
                    c("child_declaration", "support"),
                    c("child", "support"),
                    c("axis", "support"),
                ])?
                .alias("supports"),
        ]);
        frontier = plans.retain(project(active, fields)?).await?;
    }
    let complete = plans.retain(union(finished)?).await?;
    plans
        .require(
            &complete,
            array_length(col("index")).eq(array_length(col("domain_ids"))),
            "port tuple differs from its ordered axes",
        )
        .await?;
    Ok(complete)
}
