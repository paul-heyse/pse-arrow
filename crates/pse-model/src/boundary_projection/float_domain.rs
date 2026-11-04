// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Intrinsic binary64 setting domains shared by native and wire validation.

/// Locally decidable scalar domains declared by the native setting owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Float64Domain {
    /// Finite values of either sign, including both signed zeros.
    Finite,
    /// Finite strictly positive values, including positive subnormals.
    PositiveFinite,
    /// Finite values greater than zero and at most one.
    PositiveUnit,
}
impl Float64Domain {
    /// Evaluate the actual native domain independently of its fixed64 transport.
    pub const fn contains(self, value: f64) -> bool {
        match self {
            Self::Finite => value.is_finite(),
            Self::PositiveFinite => value.is_finite() && value > 0.0,
            Self::PositiveUnit => value.is_finite() && value > 0.0 && value <= 1.0,
        }
    }
    /// Exact increasing positive-bit interval for domains excluding both signed zeros.
    /// Signed finite domains require both IEEE sign intervals instead.
    pub const fn positive_maximum_bits(self) -> Option<u64> {
        match self {
            Self::Finite => None,
            Self::PositiveFinite => Some(f64::MAX.to_bits()),
            Self::PositiveUnit => Some(1.0_f64.to_bits()),
        }
    }
}
