// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact scalar guard binding reused for port declarations and state members.
use super::{
    CompilerError, Expr, JoinType, LogicalPlan, Plans, array_length, c, col, error, filter, join,
    lit, project,
};
use datafusion::functions::string::expr_fn::ends_with;

pub(super) async fn settle(
    plans: &mut Plans<'_>,
    base: LogicalPlan,
    owner: Expr,
    template: Expr,
    guard: Expr,
    outcome: &str,
) -> Result<LogicalPlan, CompilerError> {
    let declaration = plans.scan("normalized.template_guards", "guard_declaration")?;
    let joined = join(
        base.clone(),
        declaration,
        JoinType::Left,
        [
            guard.clone().eq(c("guard_declaration", "guard_id")),
            template.eq(c("guard_declaration", "template_id")),
        ],
    )?;
    let source = plans.scan("normalized.expression_sources", "guard_source")?;
    let source = filter(
        source,
        c("guard_source", "source_relation_id")
            .eq(plans.sid(pse_relations::generated::authored::template_guards::RELATION_ID)?)
            .and(ends_with(
                c("guard_source", "field_path"),
                lit("/predicate"),
            )),
    )?;
    let joined = join(
        joined,
        source,
        JoinType::Left,
        [
            c("guard_source", "source_key").eq(pse_catalog::session::scalar::key(
                pse_relations::generated::authored::template_guards::RELATION_ID,
                vec![("guard_id", c("guard_declaration", "guard_id"))],
            )),
        ],
    )?;
    let outcomes = plans.scan("inferred.predicate_outcomes", "guard_value")?;
    let joined = join(
        joined,
        outcomes,
        JoinType::Left,
        [
            owner.eq(c("guard_value", "instance_id")),
            c("guard_source", "source_id").eq(c("guard_value", "source_id")),
            c("guard_source", "root_id").eq(c("guard_value", "predicate_id")),
            array_length(c("guard_value", "index")).eq(lit(0_i64)),
        ],
    )?;
    let decided = c("guard_value", "outcome")
        .eq(lit("true"))
        .or(c("guard_value", "outcome").eq(lit("false")));
    plans
        .require(
            &joined,
            guard.clone().is_null().or(decided),
            "guard lacks one exact settled scalar instance outcome",
        )
        .await?;
    let mut columns = base
        .schema()
        .columns()
        .into_iter()
        .filter(|field| field.name != "supports")
        .map(Expr::Column)
        .collect::<Vec<_>>();
    let allowed = plans.enum_literal("TruthValue", "true")?;
    let outcome_value = pse_catalog::session::output::same_field_case(
        joined.schema(),
        guard.is_null(),
        allowed,
        c("guard_value", "outcome"),
    )
    .map_err(error)?;
    columns.push(plans.present(outcome_value)?.alias(outcome));
    columns.push(
        plans
            .lists(vec![
                col("supports"),
                c("guard_declaration", "support"),
                c("guard_source", "support"),
                c("guard_value", "support"),
            ])?
            .alias("supports"),
    );
    project(joined, columns)
}
