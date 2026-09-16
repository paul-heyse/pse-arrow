// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Guard source keys and settled scalar outcomes are matched relationally.
use super::native::{Sources, append, c, error, filter, join, require, sid};
use crate::CompilerError;
use datafusion::{
    common::UnnestOptions,
    functions::{core::expr_fn::get_field, string::expr_fn::ends_with},
    functions_nested::expr_fn::array_length,
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_ids::CancellationToken;

pub(super) async fn apply(
    base: LogicalPlan,
    sources: &mut Sources<'_>,
    cancel: &CancellationToken,
) -> Result<LogicalPlan, CompilerError> {
    let declarations = sources.scan("normalized.template_guards", "guard")?;
    let base = join(
        base,
        declarations,
        JoinType::Left,
        [
            c("law", "guard_id").eq(c("guard", "guard_id")),
            c("instance", "template_id").eq(c("guard", "template_id")),
        ],
    )?;
    let source = sources.scan("normalized.expression_sources", "guard_source")?;
    let source = filter(
        source,
        c("guard_source", "source_relation_id")
            .eq(sid(
                sources.session.registry(),
                pse_relations::generated::authored::template_guards::RELATION_ID,
            )?)
            .and(ends_with(
                c("guard_source", "field_path"),
                lit("/predicate"),
            )),
    )?;
    let source = append(source, [c("guard_source", "source_key").alias("guard_key")])?;
    let source = LogicalPlanBuilder::from(source)
        .unnest_column_with_options("guard_key", UnnestOptions::new().with_preserve_nulls(false))
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let source = filter(
        source,
        get_field(col("guard_key"), "column_name").eq(lit("guard_id")),
    )?;
    let base = join(
        base,
        source,
        JoinType::Left,
        [c("guard", "guard_id").eq(get_field(col("guard_key"), "semantic_id"))],
    )?;
    let outcomes = sources.scan("inferred.predicate_outcomes", "guard_outcome")?;
    let base = join(
        base,
        outcomes,
        JoinType::Left,
        [
            c("instance", "instance_id").eq(c("guard_outcome", "instance_id")),
            c("guard_source", "source_id").eq(c("guard_outcome", "source_id")),
            c("guard_source", "root_id").eq(c("guard_outcome", "predicate_id")),
            array_length(c("guard_outcome", "index")).eq(lit(0_u64)),
        ],
    )?;
    let decided = c("guard_outcome", "outcome")
        .eq(lit("true"))
        .or(c("guard_outcome", "outcome").eq(lit("false")));
    require(
        &base,
        c("law", "guard_id").is_null().or(decided),
        "law declaration guard lacks an exact settled scalar outcome",
        sources.session,
        cancel,
    )
    .await?;
    filter(
        base,
        c("law", "guard_id")
            .is_null()
            .or(c("guard_outcome", "outcome").eq(lit("true"))),
    )
}
