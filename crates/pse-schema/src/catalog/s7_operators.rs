// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Function capabilities are a projection of actual typed handlers.
use super::declarations::{column, relation};
use crate::{RegistryBuilder, model::{FieldContract as T, Namespace as N, SnapshotClass as S}};
/// Declare the actual typed-handler capability projection.
pub fn declare(builder: &mut RegistryBuilder) {
    relation(builder, N::Reference, "function_capabilities", S::Model, &["name"], vec![
        column("name", T::native(arrow_schema::DataType::Utf8)),
        column("implementation", T::native(arrow_schema::DataType::Utf8)),
    ], "Actual typed function handler; unsupported syntax is explicitly marked unavailable.");
}
