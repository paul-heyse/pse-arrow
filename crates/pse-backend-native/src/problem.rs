// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Derived solver layout. Authoritative model/case facts remain native child inputs.

/// One ordered decision column and its case-bound limits/scaling.
#[derive(Clone, Debug)]
pub struct Variable {
    /// Exact column in the native child's numerical layout.
    pub column: datafusion::common::Column,
    /// Physical lower bound; absent means unbounded.
    pub lower: Option<f64>,
    /// Physical upper bound; absent means unbounded.
    pub upper: Option<f64>,
    /// Positive scaling factor applied by Ipopt to this physical variable.
    pub scale: f64,
}

/// Hessian selection is mandatory; there is no silent finite-difference fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HessianPolicy {
    /// Ipopt's explicit limited-memory approximation; exact first derivatives remain required.
    LimitedMemory,
}

/// Explicit solver profile; no process-global option file is read.
#[derive(Clone, Debug)]
pub struct SolveOptions {
    /// The selected Hessian capability.
    pub hessian: HessianPolicy,
    /// Maximum iterations, positive and within the C API integer range.
    pub max_iterations: u32,
    /// Desired convergence tolerance.
    pub tolerance: f64,
    /// Wall-time limit, checked by Ipopt in addition to cooperative cancellation.
    pub max_wall_seconds: f64,
}
impl SolveOptions {
    pub(crate) fn validate(&self) -> Result<(), crate::NativeError> {
        if self.max_iterations == 0
            || i32::try_from(self.max_iterations).is_err()
            || !self.tolerance.is_finite()
            || self.tolerance <= 0.0
            || !self.max_wall_seconds.is_finite()
            || self.max_wall_seconds <= 0.0
        {
            return Err(crate::error::invalid("invalid solve limits or tolerance"));
        }
        Ok(())
    }
}
