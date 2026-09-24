// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact leaf-crate imports for columnar contracts; no SQL planner in the normal closure.
#[allow(
    clippy::single_component_path_imports,
    reason = "this module provides the leaf-crate route used by shared native codecs"
)]
pub(crate) use arrow;
#[cfg(test)]
pub(crate) use datafusion::{execution, physical_plan};
pub(crate) use datafusion_common as common;
pub(crate) use datafusion_expr as logical_expr;
pub(crate) use datafusion_functions as functions;
pub(crate) use datafusion_functions_aggregate as functions_aggregate;
pub(crate) use datafusion_functions_nested as functions_nested;
pub(crate) use datafusion_physical_expr as physical_expr;
