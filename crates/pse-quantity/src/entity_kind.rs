// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Package-owned entity kinds have semantic identities, with no closed scientific vocabulary.
use crate::EntityKindId;
crate::quantity_boundary_record! {
callback: entity_kind_boundary; name: "PhysicalEntityKind"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// A declared kind used as a physical axis or subject.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[schemars(deny_unknown_fields)]
#[schemars(rename = "PhysicalEntityKind")]
pub struct EntityKind {
    /// Stable authored identity.
    pub id: EntityKindId => 1;
    /// Qualified inspection name; identity comparisons use `id`.
    pub name: String => 2;
}
}
