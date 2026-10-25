// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Package-owned entity kinds have semantic identities, with no closed scientific vocabulary.
use crate::EntityKindId;
/// A declared kind used as a physical axis or subject.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[schemars(deny_unknown_fields)]
#[schemars(rename = "PhysicalEntityKind")]
pub struct EntityKind {
    /// Stable authored identity.
    pub id: EntityKindId,
    /// Qualified inspection name; identity comparisons use `id`.
    pub name: String,
}
