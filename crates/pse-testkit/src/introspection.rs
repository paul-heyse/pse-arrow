// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Inspect actual native plans and metrics without rendering or another graph model.
use datafusion::{
    common::{DataFusionError, Result},
    physical_plan::{ExecutionPlan, ExecutionPlanVisitor, accept},
};

/// Visit native operators with a hard work bound. The callback can inspect actual
/// properties, public downcasts, expressions and `MetricsSet`, including custom metrics
/// absent from the tracing macro's declared field vocabulary.
/// # Errors
/// The native traversal or callback fails, or the requested work bound is exceeded.
pub fn visit_physical(
    plan: &dyn ExecutionPlan,
    maximum_nodes: usize,
    inspect: impl FnMut(&dyn ExecutionPlan) -> Result<()>,
) -> Result<()> {
    struct Visitor<F> {
        remaining: usize,
        inspect: F,
    }
    impl<F: FnMut(&dyn ExecutionPlan) -> Result<()>> ExecutionPlanVisitor for Visitor<F> {
        type Error = DataFusionError;
        fn pre_visit(&mut self, plan: &dyn ExecutionPlan) -> Result<bool> {
            self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
                DataFusionError::ResourcesExhausted("native inspection node bound exceeded".into())
            })?;
            (self.inspect)(plan)?;
            Ok(true)
        }
        fn post_visit(&mut self, _: &dyn ExecutionPlan) -> Result<bool> {
            Ok(true)
        }
    }
    accept(
        plan,
        &mut Visitor {
            remaining: maximum_nodes,
            inspect,
        },
    )
}
