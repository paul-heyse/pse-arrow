// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive original-result observations; no execution or protected-reader authority.
use serde::{Deserialize, Serialize};

pse_contract::projection_struct! {
/// Exact original result lineage and range, including recorded independent usability.
/// The passive projection cannot create a protected reader or renew scientific permission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultSelection {
    projection: { name: "RecordedResultSelection", rust_path: crate::recorded_result::PackResultSelection,
        version: 1, direction: pse_contract::BoundaryDirection::Output, construction: ObservationOnly,
        reserved_tags: [], reserved_names: [] };
    /// Original canonical run identity; never a current-run alias.
    pub run: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exact original attempt identity; retries never substitute another attempt.
    pub attempt: String => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Qualified original registry relation name.
    pub relation: String => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original registry relation schema version, independent of IPC transport encoding.
    pub schema_version: u32 => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Inclusive original row coordinate, before selected filtering.
    pub row_start: u64 => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exclusive original row coordinate, before selected filtering.
    pub row_end: u64 => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Optional original field/output selection; schema remains the original typed relation.
    pub field: Option<String> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original closed attempt classification, independent of publication.
    pub terminal_class: String => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    #[serde(default)]
    /// Exact original output predicate; absent for the whole selected relation.
    pub predicate: Option<PackResultPredicate> => { tag: 9, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original scientific product usability, never upgraded by copying.
    pub usability: PackScientificState => { tag: 10, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original native interpretation; absent for historical observations that did not retain it.
    #[serde(default)]
    pub context: Option<PackResultContext> => { tag: 11, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Exact protected output predicate. Floating endpoints retain IEEE bits, not JSON rounding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultPredicate {
    projection: { name: "RecordedResultPredicate", rust_path: crate::recorded_result::PackResultPredicate,
        version: 1, direction: pse_contract::BoundaryDirection::Output, construction: ObservationOnly,
        reserved_tags: [], reserved_names: [] };
    /// Original scientific output owner identity.
    pub output: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exact original selected output partition.
    pub partition: String => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Optional lower predicate endpoint as exact IEEE-754 bits.
    pub minimum_bits: Option<u64> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Optional upper predicate endpoint as exact IEEE-754 bits.
    pub maximum_bits: Option<u64> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original missing-value predicate, independent of empty output.
    pub missing: Option<bool> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Original immutable native context needed to interpret a recorded selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultContext {
    projection: { name: "RecordedResultContext", rust_path: crate::recorded_result::PackResultContext,
        version: 1, direction: pse_contract::BoundaryDirection::Output, construction: ObservationOnly,
        reserved_tags: [], reserved_names: [] };
    /// Original canonical problem identity.
    pub problem: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original immutable problem revision.
    pub revision: String => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original closed manifest digest, independent of returned extent.
    pub manifest: String => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original native interpretation, never today's default mapping.
    pub interpretation: String => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_enum! {
/// Scientific usability recorded by the producer; publication never upgrades it.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PackScientificState {
    projection: { name: "RecordedScientificUsability", rust_path: crate::recorded_result::PackScientificState,
        version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Original scientific usability was not established.
    #[default]
    Unknown => 1;
    /// Original authority admitted this selected product as usable.
    Usable => 2;
    /// Original selected scientific product has partial coverage.
    Partial => 3;
    /// Original scientific work failed; export may still complete.
    Failed => 4;
    /// Original product is retained but lacks scientific use permission.
    Unusable => 5;
}
}
