// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reachability is a native query over exact publication and retention relations.
use crate::{
    CatalogError,
    session::{RelationPlan, SnapshotSession},
};
use datafusion::{
    common::{NullHandling, UnnestOptions},
    functions::core::expr_fn::get_field,
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use pse_ids::CancellationToken;
use pse_relations::generated::runtime::{publications, retained_versions};

impl SnapshotSession {
    /// Derive exact member and input reachability from retained control rows. Empty
    /// member lists protect nothing; unmatched input selections still protect their
    /// source version. Include control-root versions and active attempt/CDF ranges
    /// through `combine_retention` using declared retained-version inputs.
    /// # Errors
    /// Foreign source, wrong relation declaration or invalid native output.
    pub fn publication_retention(
        &self,
        source: &RelationPlan,
        cancel: &CancellationToken,
    ) -> Result<RelationPlan, CatalogError> {
        if source.relation_id() != publications::RELATION_ID {
            return Err(invalid("retention source must be runtime.publications"));
        }
        let checked = self.scalar_function("pse_checked_value")?;
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
        let plan = LogicalPlanBuilder::from(members("members").map_err(crate::session::engine)?)
            .union(members("inputs").map_err(crate::session::engine)?)
            .and_then(LogicalPlanBuilder::distinct)
            .and_then(LogicalPlanBuilder::build)
            .map_err(crate::session::engine)?;
        let plan = self.derive_plan_fields(plan, cancel)?;
        RelationPlan::derived(retained_versions::RELATION_ID, plan, self.registry())
    }

    /// Union publication reachability, retained outputs, unresolved attempts and CDF
    /// cursors without host-language row reconstruction. Every inclusive interval
    /// remains visible as a real child of the maintenance command.
    /// # Errors
    /// No inputs, undeclared retention source, invalid plan or cancellation.
    pub fn combine_retention(
        &self,
        inputs: &[RelationPlan],
        cancel: &CancellationToken,
    ) -> Result<RelationPlan, CatalogError> {
        let mut inputs = inputs.iter();
        let first = inputs.next().ok_or_else(|| {
            invalid("retention requires an explicit declared input, possibly empty")
        })?;
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
                .map_err(crate::session::engine)?;
        }
        let plan = LogicalPlanBuilder::from(plan)
            .distinct()
            .and_then(LogicalPlanBuilder::build)
            .map_err(crate::session::engine)?;
        RelationPlan::derived(
            retained_versions::RELATION_ID,
            self.derive_plan_fields(plan, cancel)?,
            self.registry(),
        )
    }
}

fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "delta.retention".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
