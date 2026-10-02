// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mechanical relation integrity; source identity/closure facts admit at their boundary.
use crate::RegistryBuilder;

/// Add registry-derived relation integrity projections.
pub fn declare(builder: &mut RegistryBuilder) {
    builder.derive_integrity();
}
