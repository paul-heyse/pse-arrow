// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native observation documents; table reference components remain literal.

/// A literal native table reference, without SQL normalization.
#[derive(
    Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct TableName {
    /// Literal catalog component.
    pub catalog: String,
    /// Literal schema component.
    pub schema: String,
    /// Literal table component.
    pub table: String,
}
impl From<(String, String, String)> for TableName {
    fn from((catalog, schema, table): (String, String, String)) -> Self {
        Self {
            catalog,
            schema,
            table,
        }
    }
}
/// A current native memory consumer and its accounted reservation.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ResourceConsumer {
    /// Accounted memory owner name.
    pub name: String,
    /// Current reservation in bytes.
    pub reserved_bytes: usize,
}
impl From<(String, usize)> for ResourceConsumer {
    fn from((name, reserved_bytes): (String, usize)) -> Self {
        Self {
            name,
            reserved_bytes,
        }
    }
}
