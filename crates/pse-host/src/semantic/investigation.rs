// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualified native questions whose outcomes are not interchangeable booleans.
use super::NativeLocation;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The requested pairwise native operation, never inferred from displayed types.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityOperation {
    /// Exact native type equality in a common owner.
    Equal,
    /// Possible deep unification, not compiler acceptance.
    CouldUnify,
    /// Directional possible coercion from selected to against.
    CouldCoerce,
}
/// Selected unusual source-origin inquiry.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind {
    /// Captured format parts, including inline-assembly operands.
    Format,
    /// Derive-generated impl and helper observations.
    Derive,
}
/// One selected body's original source correspondence.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NativeBodyObservation {
    /// Unique native package identity when available.
    pub package_id: Option<String>,
    /// Native unit role when uniquely established.
    pub role: Option<String>,
    /// Original selected body interval when unique.
    pub location: Option<NativeLocation>,
    /// Native body category; generated bodies may lack unique source.
    pub kind: String,
    /// Original containing declaration.
    pub owner: Option<NativeLocation>,
    /// Missing/plural mapping remains explicit.
    pub limitations: Vec<String>,
}
/// Native comparison with source and owner premises retained.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CompatibilityObservation {
    /// Requested operation.
    pub operation: CompatibilityOperation,
    /// Original left type source.
    pub left: NativeLocation,
    /// Original right type source.
    pub right: NativeLocation,
    /// Display only, not identity.
    pub left_type: String,
    /// Display only, not identity.
    pub right_type: String,
    /// Absent when owner compatibility or inference is unresolved.
    pub result: Option<bool>,
    /// Whether fallible rebasing established a common generic owner.
    pub compatible_owner: bool,
    /// Original scope and unsupported facts.
    pub limitations: Vec<String>,
}
/// The target meaning of an original provenance observation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OriginTargetKind {
    /// Resolved declaration with separately qualified original mapping.
    Declaration,
    /// A local binding, potentially with plural pattern sources.
    Local,
    /// An operand scoped to one assembly invocation.
    AssemblyOperand,
    /// An authored ADT that originated derive evidence.
    OriginatingAdt,
    /// One native helper nomination, preserving multiplicity.
    DeriveHelper,
    /// An implementation generated from a derive; no fabricated identity.
    GeneratedImpl,
    /// No target was established by the native owner.
    Unresolved,
}
/// A format part or derive observation; response-local ordinal is not declaration identity.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OriginObservation {
    /// Ordinal local to this exact answer.
    pub ordinal: usize,
    /// Target category, separate from its optional source mapping.
    pub target_kind: OriginTargetKind,
    /// Original source occurrence.
    pub occurrence: Option<NativeLocation>,
    /// Precisely mapped original target, absent when unsupported.
    pub target: Option<NativeLocation>,
    /// Native label, never joined by spelling.
    pub name: Option<String>,
    /// The ADT that originated a generated impl, not its identity.
    pub originating_adt: Option<NativeLocation>,
    /// Original mapping/interpretation limits.
    pub limitations: Vec<String>,
}
/// Typed views consumed by lifetime, compatibility, visibility and provenance inspection.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct NativeInvestigationFacts {
    /// Body mapping for the selected original context.
    pub body: Option<NativeBodyObservation>,
    /// Requested pairwise comparison.
    pub compatibility: Option<CompatibilityObservation>,
    /// Contextual visibility when established.
    pub visible: Option<bool>,
    /// Source-origin observations, preserving multiplicity.
    pub origins: Vec<OriginObservation>,
    /// Question-level coverage limits.
    pub limitations: Vec<String>,
}
