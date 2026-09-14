// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registered quantity conversions (blueprint §6.2, §8.4).
use crate::{ConversionId, ConversionKind, QuantityTypeId};
use pse_ids::SemanticId;
/// One explicit conversion; applying it is separate from declaring it.
#[derive(Clone, Debug)]
pub struct ConversionRule {
    /// Registry identity.
    pub id: ConversionId,
    /// Complete input contract.
    pub from: QuantityTypeId,
    /// Complete output contract.
    pub to: QuantityTypeId,
    /// Scale, affine, or a parameterized kernel.
    pub kind: ConversionKind,
    /// Required only for a kernel conversion.
    pub kernel: Option<SemanticId>,
    /// Named parameter dependencies, never implicit inputs.
    pub required_parameters: Vec<String>,
    /// Resolved multiplicative coefficient; absent for a kernel.
    pub scale: Option<f64>,
    /// Resolved additive coefficient; absent for a kernel.
    pub offset: Option<f64>,
}
