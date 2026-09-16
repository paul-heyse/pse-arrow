// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native per-scan input substitution for semi-naive positive fixed points.
use super::CompiledRule;
use crate::{RuleError, errmap::engine};
use datafusion_expr::LogicalPlanBuilder;

/// Substitute one recursive input occurrence at a time. In a self-join, the other
/// occurrence still reads accumulated facts; substituting all occurrences would lose
/// new/old matches. The union of these variants is the actual semi-naive input delta.
pub(crate) fn input_deltas(
    rule: &CompiledRule,
    replacements: &std::collections::BTreeMap<
        datafusion_common::TableReference,
        datafusion_expr::LogicalPlan,
    >,
) -> Result<Vec<CompiledRule>, RuleError> {
    substitute_inputs(&rule.plan, replacements)?
        .into_iter()
        .map(|plan| {
            let mut variant = rule.clone();
            variant.plan = plan;
            variant.undecided = None;
            Ok(variant)
        })
        .collect()
}

pub(crate) fn substitute_inputs(
    plan: &datafusion_expr::LogicalPlan,
    replacements: &std::collections::BTreeMap<
        datafusion_common::TableReference,
        datafusion_expr::LogicalPlan,
    >,
) -> Result<Vec<datafusion_expr::LogicalPlan>, RuleError> {
    use datafusion_common::tree_node::{Transformed, TreeNode, TreeNodeRecursion};
    use datafusion_expr::LogicalPlan;

    let mut count = 0;
    plan.apply(|node| {
        if matches!(node, LogicalPlan::TableScan(scan) if replacements.contains_key(&scan.table_name)) {
            count += 1;
        }
        Ok(TreeNodeRecursion::Continue)
    }).map_err(engine)?;
    let mut variants = Vec::with_capacity(count);
    for selected in 0..count {
        let mut occurrence = 0;
        let variant = plan
            .clone()
            .transform_down(|node| {
                let LogicalPlan::TableScan(scan) = &node else {
                    return Ok(Transformed::no(node));
                };
                let Some(delta) = replacements.get(&scan.table_name) else {
                    return Ok(Transformed::no(node));
                };
                let current = occurrence;
                occurrence += 1;
                if current != selected {
                    return Ok(Transformed::no(node));
                }
                if scan.projection.is_some() || !scan.filters.is_empty() || scan.fetch.is_some() {
                    return Err(datafusion_common::DataFusionError::Plan(
                        "input deltas bind the original unoptimized rule scans".to_owned(),
                    ));
                }
                let replacement = LogicalPlanBuilder::from(delta.clone())
                    .alias(scan.table_name.clone())?
                    .build()?;
                if replacement.schema().as_arrow() != scan.projected_schema.as_arrow() {
                    return Err(datafusion_common::DataFusionError::Plan(
                        "input delta differs from the actual recursive input schema".to_owned(),
                    ));
                }
                // The replacement is already a complete admitted scan. Do not revisit it
                // and count its own source as another occurrence in the original plan.
                Ok(Transformed::new(replacement, true, TreeNodeRecursion::Jump))
            })
            .map_err(engine)?
            .data;
        variants.push(variant);
    }
    Ok(variants)
}
