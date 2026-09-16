// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native parameter tuples apply each declared source-coordinate mapping.
use super::{
    CompilerError, JoinType, LogicalPlan, LogicalPlanBuilder, Outputs, Plans, array_element,
    array_length, c, coalesce, col, emit, error, filter, indices, join, lit, prefix, project,
    scalar, union, when,
};
use datafusion::{functions_aggregate::expr_fn::count, functions_nested::expr_fn::flatten};
use pse_catalog::session::aggregate::array_agg;

pub(super) async fn build(
    plans: &mut Plans<'_>,
    base: LogicalPlan,
    indices: &indices::Indices,
    outputs: &mut Outputs,
) -> Result<(), CompilerError> {
    let parameters = plans.scan("reference.method_parameters", "parameter")?;
    let base = join(
        base,
        parameters,
        JoinType::Inner,
        [col("method_id").eq(c("parameter", "method_id"))],
    )?;
    let mappings = plans.scan("reference.method_parameter_axes", "mapping")?;
    let counts = LogicalPlanBuilder::from(mappings.clone())
        .aggregate(
            [c("mapping", "method_id"), c("mapping", "parameter_kind")],
            [count(lit(1_u64)).alias("axis_count")],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let base = join(
        base,
        prefix(counts, "count")?,
        JoinType::Left,
        [
            col("method_id").eq(c("count", "mapping:method_id")),
            c("parameter", "name").eq(c("count", "mapping:parameter_kind")),
        ],
    )?;
    plans
        .require(
            &base,
            coalesce(vec![c("count", "axis_count"), lit(0_i64)])
                .eq(array_length(c("parameter", "indexed_by"))),
            "parameter source-coordinate mapping must cover every axis exactly once",
        )
        .await?;
    let base = project(
        base,
        [
            col("requirement_id"),
            col("state_instance_id"),
            col("method_id"),
            c("parameter", "name").alias("name"),
            c("parameter", "indexed_by").alias("shape"),
            lit(0_i64).alias("position"),
            scalar::id_list(vec![]).alias("index"),
            plans
                .lists(vec![col("supports"), c("parameter", "support")])?
                .alias("supports"),
        ],
    )?;
    let source_axes = LogicalPlanBuilder::from(indices.axes.clone())
        .aggregate(
            [col("requirement_id"), col("kind")],
            [
                array_agg(col("member_id")).alias("members"),
                array_agg(col("domain_id")).alias("domains"),
                array_agg(col("supports")).alias("axis_supports"),
            ],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    // Equal source rows may have independent supports. Their complete axis key was
    // admitted by NativeInput, so scans use that completed relation below.
    let mut frontier = plans.retain(base).await?;
    let mut finished = Vec::new();
    loop {
        plans.cancel.checkpoint()?;
        finished.push(filter(
            frontier.clone(),
            col("position").eq(array_length(col("shape"))),
        )?);
        let (pending, count) = plans
            .retain_with_count(filter(
                frontier,
                col("position").lt(array_length(col("shape"))),
            )?)
            .await?;
        if count == 0 {
            break;
        }
        let step = join(
            pending,
            mappings.clone(),
            JoinType::Left,
            [
                col("method_id").eq(c("mapping", "method_id")),
                col("name").eq(c("mapping", "parameter_kind")),
                col("position").eq(c("mapping", "position")),
            ],
        )?;
        plans
            .require(
                &step,
                c("mapping", "source_coordinate").is_not_null(),
                "parameter mapping position is missing",
            )
            .await?;
        let kind = array_element(col("shape"), col("position") + lit(1_i64));
        let step = join(
            step,
            prefix(source_axes.clone(), "selected_axis")?,
            JoinType::Left,
            [
                col("requirement_id").eq(c("selected_axis", "requirement_id")),
                kind.clone().eq(c("selected_axis", "kind")),
            ],
        )?;
        plans
            .require(
                &step,
                coalesce(vec![
                    array_length(c("selected_axis", "members")),
                    lit(0_u64),
                ])
                .lt_eq(lit(1_u64)),
                "parameter signature ambiguously repeats a source domain kind",
            )
            .await?;
        let step = join(
            step,
            prefix(indices.pool.clone(), "axis")?,
            JoinType::Inner,
            [
                col("state_instance_id").eq(c("axis", "state_instance_id")),
                kind.clone().eq(c("axis", "kind")),
                c("selected_axis", "members").is_null().or(array_element(
                    c("selected_axis", "members"),
                    lit(1_i64),
                )
                .eq(c("axis", "member_id"))
                .and(
                    array_element(c("selected_axis", "domains"), lit(1_i64))
                        .eq(c("axis", "domain_id")),
                )),
            ],
        )?;
        let coordinate = c("mapping", "source_coordinate");
        let pair = coordinate.clone().eq(lit("phase_species_pair"));
        let material = plans.scan("normalized.material_domain_members", "material")?;
        let step = join(
            step,
            material,
            JoinType::Left,
            [
                pair.clone(),
                c("axis", "domain_id").eq(c("material", "domain_id")),
                c("axis", "member_id").eq(c("material", "member_id")),
            ],
        )?;
        plans
            .require(
                &step,
                coordinate
                    .clone()
                    .eq(lit("member"))
                    .or(coordinate
                        .clone()
                        .eq(lit("ref_entity"))
                        .and(c("axis", "subject_id").is_not_null()))
                    .or(pair
                        .clone()
                        .and(kind.eq(lit("phase_species")))
                        .and(c("material", "phase_id").is_not_null())
                        .and(c("material", "species_id").is_not_null())),
                "parameter source coordinate lacks its actual member/reference/material binding",
            )
            .await?;
        let mapped = when(
            coordinate.clone().eq(lit("member")),
            scalar::id_list(vec![c("axis", "member_id")]),
        )
        .when(
            coordinate.eq(lit("ref_entity")),
            scalar::id_list(vec![c("axis", "subject_id")]),
        )
        .otherwise(scalar::id_list(vec![
            c("material", "phase_id"),
            c("material", "species_id"),
        ]))
        .map_err(error)?;
        frontier = plans
            .retain(project(
                step,
                [
                    col("requirement_id"),
                    col("state_instance_id"),
                    col("method_id"),
                    col("name"),
                    col("shape"),
                    (col("position") + lit(1_i64)).alias("position"),
                    plans.ids(vec![col("index"), mapped])?.alias("index"),
                    plans
                        .lists(vec![
                            col("supports"),
                            c("mapping", "support"),
                            c("axis", "supports"),
                            flatten(c("selected_axis", "axis_supports")),
                            c("material", "support"),
                        ])?
                        .alias("supports"),
                ],
            )?)
            .await?;
    }
    let output = project(
        union(finished)?,
        [
            col("requirement_id"),
            col("method_id"),
            col("name"),
            plans.index(col("index"))?.alias("index"),
            col("supports"),
        ],
    )?;
    emit(plans, outputs, "method_parameter_keys", output).await
}
