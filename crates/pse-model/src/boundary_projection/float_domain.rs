// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Intrinsic binary64 setting domains shared by native and wire validation.

use super::Float64Interval;

/// Locally decidable scalar domains declared by the native setting owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Float64Domain {
    /// Finite values of either sign, including both signed zeros.
    Finite,
    /// Finite values greater than or equal to zero, preserving both signed zeros.
    NonnegativeFinite,
    /// Finite strictly positive values, including positive subnormals.
    PositiveFinite,
    /// Finite values greater than zero and at most one.
    PositiveUnit,
    /// Exact finite source-owned endpoints, including their open/closed interpretation.
    Interval(Float64Interval),
}
impl Float64Domain {
    /// Evaluate the actual native domain independently of its fixed64 transport.
    pub const fn contains(self, value: f64) -> bool {
        self.interval().contains(value)
    }
    /// Resolve every preset and declared interval through the same exact finite mechanism.
    pub const fn interval(self) -> Float64Interval {
        match self {
            Self::Finite => Float64Interval::new(-f64::MAX, true, f64::MAX, true),
            Self::NonnegativeFinite => Float64Interval::new(0.0, true, f64::MAX, true),
            Self::PositiveFinite => Float64Interval::new(0.0, false, f64::MAX, true),
            Self::PositiveUnit => Float64Interval::new(0.0, false, 1.0, true),
            Self::Interval(interval) => interval,
        }
    }
}
