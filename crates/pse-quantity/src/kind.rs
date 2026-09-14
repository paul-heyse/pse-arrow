// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Quantity kind declarations (blueprint §6.2).
use crate::{DimensionVector, QuantityAdditionKind, QuantityKindId};
/// What is measured, independent of basis and datum.
#[derive(Clone, Debug)]
pub struct QuantityKind {
    /// Registry identity.
    pub id: QuantityKindId,
    /// SI exponents; this alone never resolves a physical type.
    pub dimension: DimensionVector,
    /// Whether values scale with system size.
    pub extensive: bool,
    /// Additive or origin-sensitive arithmetic.
    pub addition_kind: QuantityAdditionKind,
}
