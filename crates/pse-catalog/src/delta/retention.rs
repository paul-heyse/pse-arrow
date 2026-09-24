// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reachability is a native query over exact publication and retention relations.
use datafusion::{
    common::{NullHandling, UnnestOptions},
    functions::core::expr_fn::get_field,
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use pse_columnar::CancellationToken;
use pse_engine::{
    EngineError,
    session::{EngineSession, RelationPlan},
};
use pse_relations::generated::runtime::{publications, retained_versions};

/// Feed whole-release replay windows directly into the existing maintenance input.
/// Endpoint versions are included in each interval; an empty CDF cannot erase them.
/// # Errors
/// Wrong declared checkpoint source or native projection/contract refusal.
pub fn checkpoint_retention(
    session: &EngineSession,
    source: &RelationPlan,
    cancel: &CancellationToken,
) -> Result<RelationPlan, EngineError> {
    if source.relation_id() != pse_relations::generated::runtime::release_checkpoints::RELATION_ID {
        return Err(invalid(
            "retention source must be runtime.release_checkpoints",
        ));
    }
    let checked = session.scalar_function("pse_checked_value")?;
    let value = |expression, name: &str| {
        checked
            .call(vec![
                expression,
                lit("runtime.retained_versions"),
                lit(name),
            ])
            .alias(name)
    };
    let plan = LogicalPlanBuilder::from(source.plan().clone())
        .project([col("intervals")])?
        .unnest_column_with_options(
            "intervals",
            UnnestOptions::new().with_null_handling(NullHandling::Drop),
        )?
        .project([
            value(get_field(col("intervals"), "table_uri"), "table_uri"),
            value(get_field(col("intervals"), "from_version"), "from_version"),
            value(
                get_field(col("intervals"), "through_version"),
                "through_version",
            ),
            value(lit("changes"), "reason"),
        ])?
        .distinct()?
        .build()?;
    RelationPlan::derived(
        retained_versions::RELATION_ID,
        session.derive_plan_fields(plan, cancel)?,
        session.registry(),
    )
}

/// Derive exact member and input reachability from retained control rows. Empty
/// member lists protect nothing; unmatched input selections still protect their
/// source version. Include control-root versions and active attempt/CDF ranges
/// through `combine_retention` using declared retained-version inputs.
/// # Errors
/// Foreign source, wrong relation declaration or invalid native output.
pub fn publication_retention(
    source_session: &EngineSession,
    source: &RelationPlan,
    cancel: &CancellationToken,
) -> Result<RelationPlan, EngineError> {
    if source.relation_id() != publications::RELATION_ID {
        return Err(invalid("retention source must be runtime.publications"));
    }
    let checked = source_session.scalar_function("pse_checked_value")?;
    let value = |expression, name: &str| {
        checked
            .call(vec![
                expression,
                lit("runtime.retained_versions"),
                lit(name),
            ])
            .alias(name)
    };
    let members = |name: &str| {
        LogicalPlanBuilder::from(source.plan().clone())
            .project([col(name).alias("selected")])?
            .unnest_column_with_options(
                "selected",
                UnnestOptions::new().with_null_handling(NullHandling::Drop),
            )?
            .project([
                value(get_field(col("selected"), "table_uri"), "table_uri"),
                value(get_field(col("selected"), "delta_version"), "from_version"),
                value(
                    get_field(col("selected"), "delta_version"),
                    "through_version",
                ),
                value(lit("publication"), "reason"),
            ])?
            .build()
    };
    let plan = LogicalPlanBuilder::from(members("members").map_err(pse_engine::session::engine)?)
        .union(members("inputs").map_err(pse_engine::session::engine)?)
        .and_then(LogicalPlanBuilder::distinct)
        .and_then(LogicalPlanBuilder::build)
        .map_err(pse_engine::session::engine)?;
    let plan = source_session.derive_plan_fields(plan, cancel)?;
    RelationPlan::derived(
        retained_versions::RELATION_ID,
        plan,
        source_session.registry(),
    )
}

/// Union publication reachability, retained outputs, unresolved attempts and CDF
/// cursors without host-language row reconstruction. Every inclusive interval
/// remains visible as a real child of the maintenance command.
/// # Errors
/// No inputs, undeclared retention source, invalid plan or cancellation.
pub fn combine_retention(
    source_session: &EngineSession,
    inputs: &[RelationPlan],
    cancel: &CancellationToken,
) -> Result<RelationPlan, EngineError> {
    let mut inputs = inputs.iter();
    let first = inputs
        .next()
        .ok_or_else(|| invalid("retention requires an explicit declared input, possibly empty"))?;
    let checked = |input: &RelationPlan| {
        if input.relation_id() != retained_versions::RELATION_ID {
            return Err(invalid("retention input must be runtime.retained_versions"));
        }
        Ok(input.plan().clone())
    };
    let mut plan = checked(first)?;
    for input in inputs {
        cancel.checkpoint()?;
        plan = LogicalPlanBuilder::from(plan)
            .union(checked(input)?)
            .and_then(LogicalPlanBuilder::build)
            .map_err(pse_engine::session::engine)?;
    }
    let plan = LogicalPlanBuilder::from(plan)
        .distinct()
        .and_then(LogicalPlanBuilder::build)
        .map_err(pse_engine::session::engine)?;
    RelationPlan::derived(
        retained_versions::RELATION_ID,
        source_session.derive_plan_fields(plan, cancel)?,
        source_session.registry(),
    )
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "delta.retention".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
