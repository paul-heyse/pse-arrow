// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original protected scientific selection, independent of offline publication tooling.
use serde::{Deserialize, Serialize};

/// Exact original result lineage and range, including independent usability.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultSelection {
    /// Original canonical run identity; never a current-run alias.
    pub run: String,
    /// Exact original attempt identity; retries never substitute another attempt.
    pub attempt: String,
    /// Qualified original registry relation name.
    pub relation: String,
    /// Original registry relation schema version, independent of IPC transport encoding.
    pub schema_version: u32,
    /// Inclusive original row coordinate, before selected filtering.
    pub row_start: u64,
    /// Exclusive original row coordinate, before selected filtering.
    pub row_end: u64,
    /// Optional original field/output selection; schema remains the original typed relation.
    pub field: Option<String>,
    /// Original closed attempt classification, independent of publication.
    pub terminal_class: String,
    #[serde(default)]
    /// Exact original output predicate; absent for the whole selected relation.
    pub predicate: Option<PackResultPredicate>,
    /// Original scientific product usability, never upgraded by copying.
    pub usability: PackScientificState,
}
/// Exact protected output predicate. Floating endpoints retain IEEE bits, not JSON rounding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackResultPredicate {
    /// Original scientific output owner identity.
    pub output: String,
    /// Exact original selected output partition.
    pub partition: String,
    /// Optional lower predicate endpoint as exact IEEE-754 bits.
    pub minimum_bits: Option<u64>,
    /// Optional upper predicate endpoint as exact IEEE-754 bits.
    pub maximum_bits: Option<u64>,
    /// Original missing-value predicate, independent of empty output.
    pub missing: Option<bool>,
}
/// Scientific usability recorded by the producer; publication never upgrades it.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PackScientificState {
    /// Original scientific usability was not established.
    #[default]
    Unknown,
    /// Original authority admitted this selected product as usable.
    Usable,
    /// Original selected scientific product has partial coverage.
    Partial,
    /// Original scientific work failed; export may still complete.
    Failed,
    /// Original product is retained but lacks scientific use permission.
    Unusable,
}
