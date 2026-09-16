// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Select the pinned engine's metadata-preserving Cartesian implementation.
//! Scalar field restoration is the visible `pse_preserve_field` logical-plan UDF.
use super::scalar::nested_metadata;
use datafusion::{
    common::{
        DataFusionError, Result,
        tree_node::{Transformed, TreeNode},
    },
    physical_plan::{
        ExecutionPlan,
        joins::{CrossJoinExec, NestedLoopJoinExec},
    },
};
use std::sync::Arc;

/// `CrossJoinExec` scalarizes build-side values before an output UDF can see them.
/// Select the engine's ordinary take-based Cartesian implementation for nested
/// fields. Scalar expression recovery lives in `pse_preserve_field`.
#[derive(Debug)]
pub(super) struct SemanticFields;
impl datafusion::physical_optimizer::PhysicalOptimizerRule for SemanticFields {
    fn name(&self) -> &'static str {
        "pse.semantic_fields.v1"
    }
    fn schema_check(&self) -> bool {
        true
    }
    fn optimize(
        &self,
        plan: Arc<dyn ExecutionPlan>,
        _config: &datafusion::common::config::ConfigOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        preserve(plan)
    }
}

fn preserve(plan: Arc<dyn ExecutionPlan>) -> Result<Arc<dyn ExecutionPlan>> {
    Ok(plan
        .transform_up(|plan| {
            if let Some(join) = plan.downcast_ref::<CrossJoinExec>()
                && join
                    .left()
                    .schema()
                    .fields()
                    .iter()
                    .any(|field| nested_metadata(field.data_type()))
            {
                // CrossJoinExec::build_batch scalarizes each build-side value and
                // loses nested child metadata. The same engine's unfiltered inner
                // nested-loop operator implements the identical Cartesian product
                // using Arrow take, preserving the already admitted child fields.
                let replacement: Arc<dyn ExecutionPlan> = Arc::new(NestedLoopJoinExec::try_new(
                    Arc::clone(join.left()),
                    Arc::clone(join.right()),
                    None,
                    &datafusion::common::JoinType::Inner,
                    None,
                )?);
                if replacement.schema() != plan.schema() {
                    return Err(DataFusionError::Internal(
                        "Cartesian replacement changed the admitted schema".to_owned(),
                    ));
                }
                return Ok(Transformed::yes(replacement));
            }
            Ok(Transformed::no(plan))
        })?
        .data)
}
