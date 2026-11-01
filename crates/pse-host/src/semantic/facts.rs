// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Consumed native observations, separate from original protocol documents.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A negotiated protocol position, retaining the provider's coordinate unit.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ProtocolPosition {
    /// Zero-based line.
    pub line: u32,
    /// Zero-based character in the selected protocol encoding.
    pub character: u32,
}
/// Half-open native protocol interval.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProtocolRange {
    /// Inclusive start.
    pub start: ProtocolPosition,
    /// Exclusive end.
    pub end: ProtocolPosition,
}
/// Coordinates supplied by the original native producer, never inferred from display.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "coordinates", rename_all = "snake_case")]
pub enum FactLocation {
    /// Exact original UTF-8 interval in a selected immutable native world.
    Bytes {
        /// Native source path.
        #[serde(with = "crate::investigation::native_path_serde")]
        #[schemars(with = "crate::investigation::NativePath")]
        path: PathBuf,
        /// Inclusive original byte offset.
        start: usize,
        /// Exclusive original byte offset.
        end: usize,
    },
    /// Original live protocol location with negotiated coordinate encoding.
    Protocol {
        /// Original canonical provider URI.
        uri: String,
        /// Native interval.
        range: ProtocolRange,
    },
}
/// Native indexed symbol; spelling and display containers confer no identity.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SymbolFact {
    /// Original name.
    pub name: String,
    /// Native kind, retained in its provider vocabulary.
    pub kind: Option<String>,
    /// Native display container, never a resolved path proof.
    pub container: Option<String>,
    /// Actual provider-mapped source.
    pub location: Option<FactLocation>,
    /// Digest bound by the original source owner, absent if unbound.
    pub source_sha256: Option<String>,
}
/// The meaning of a native relationship observation.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FactRelation {
    /// Resolved declaration target.
    Definition,
    /// Inferred type's declaration target.
    TypeDefinition,
    /// Static reference site.
    References,
    /// One-hop static call hierarchy.
    Calls,
    /// Actual implementation navigation.
    Implementations,
}
/// Native call direction, independent of the relationship type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FactDirection {
    /// Referenced/called target.
    Outgoing,
    /// Referring/calling site.
    Incoming,
}
/// A source-linked native relation, qualified by its enclosing answer context.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipFact {
    /// Actual producer operation.
    pub relation: FactRelation,
    /// Native edge direction.
    pub direction: FactDirection,
    /// Original mapped location.
    pub location: FactLocation,
    /// Native name if supplied.
    pub name: Option<String>,
    /// Library-owned read role, absent when unsupported.
    pub read: Option<bool>,
    /// Library-owned write role, absent when unsupported.
    pub write: Option<bool>,
}
/// Three-valued native proof; unsupported/not proven remains unknown under negation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FactTruth {
    /// No proof supplied in this selected model.
    Unknown,
    /// Actual native positive observation.
    True,
    /// Actual native negative observation.
    False,
}
/// Native predicate evaluation, never reconstructed from hover text.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PredicateFact {
    /// Original native truth state.
    pub truth: FactTruth,
}
/// Actual native visibility and use-path observations.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ImportabilityFact {
    /// Native HIR visibility from this destination, absent when unsupported.
    pub visibility: Option<bool>,
    /// Native use-path result, absent when applicability is not established.
    pub importable: Option<bool>,
    /// Actual native import path.
    pub path: Option<String>,
    /// Scope and mapping limitations.
    pub limitations: Vec<String>,
}
/// Lazy discovery operation retaining exact original record fingerprints.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    /// Native inlay hint.
    Hint,
    /// Native completion candidate.
    Completion,
    /// Native code action.
    Assist,
}
/// Original native candidate identity, source/generation scoped by its answer.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LazyCandidateFact {
    /// Actual resolution operation.
    pub kind: CandidateKind,
    /// Original full native ordering.
    pub index: usize,
    /// Exact native record hash checked on reacquisition.
    pub fingerprint: String,
    /// Optional original display label.
    pub label: Option<String>,
    /// Optional actual provider source mapping.
    pub location: Option<FactLocation>,
}
/// A native test candidate; arguments are original evidence and never executable authority.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TestCandidateFact {
    /// Native display label.
    pub label: Option<String>,
    /// Selected native source location.
    pub location: Option<FactLocation>,
    /// Original native runnable category; only Cargo candidates supply routing hints.
    pub runnable_kind: Option<String>,
    /// Original Cargo selection arguments, interpreted only as test-selection hints.
    pub cargo_args: Vec<String>,
    /// Original test names, never run as a native command.
    pub executable_args: Vec<String>,
    /// Original native record, for qualified inspection only.
    pub original: serde_json::Value,
}
/// Shared consumed observations; original evidence remains separately qualified.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ObservedFacts {
    /// Native name candidates.
    pub symbols: Vec<SymbolFact>,
    /// Actual source-linked native relationships.
    pub relationships: Vec<RelationshipFact>,
    /// Explicit predicate truth when requested.
    pub predicate: Option<PredicateFact>,
    /// Native visibility/import-path query when requested.
    pub importability: Option<ImportabilityFact>,
    /// Original guarded lazy candidates.
    pub candidates: Vec<LazyCandidateFact>,
    /// Native related test candidates, without execution.
    pub tests: Vec<TestCandidateFact>,
}

/// A genuine native type observation; display is presentation, not serialized type identity.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TypeFact {
    /// Native HIR rendering, never parsed back into a type.
    pub display: String,
    /// Native reference classification.
    pub is_reference: bool,
    /// Native mutable-reference classification.
    pub is_mutable_reference: bool,
    /// Native unknown component, preventing an invented proof.
    pub contains_unknown: bool,
}

/// Source/configuration-qualified target, not a spelling-based symbol identifier.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TargetFact {
    /// Native relationship, keeping a selected declaration distinct from a resolved call.
    pub relationship: String,
    /// Native selected declaration name.
    pub name: String,
    /// Authored exact location when the native mapping establishes one.
    pub location: Option<NativeLocation>,
    /// Mapping limitations remain explicit for macro/generated origins.
    pub precision: String,
}
/// One native original byte range.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NativeLocation {
    /// Exact source file in the retained native world.
    #[serde(with = "crate::investigation::native_path_serde")]
    #[schemars(with = "crate::investigation::NativePath")]
    pub path: PathBuf,
    /// Half-open original byte interval.
    pub start: usize,
    /// Half-open original byte interval.
    pub end: usize,
}
/// One reference and its library-owned roles; absence of a role is not runtime reachability.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReferenceFact {
    /// Native result group; ambiguous entities are not silently merged.
    pub entity: usize,
    /// Native original range.
    pub location: NativeLocation,
    /// Native READ category.
    pub read: bool,
    /// Native WRITE category.
    pub write: bool,
}
/// A declared generic bound, distinct from arbitrary trait obligation proof.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BoundFact {
    /// The actual generic parameter name.
    pub parameter: String,
    /// Actual native trait declaration names; identities remain in this snapshot context.
    pub declared_traits: Vec<String>,
}
/// Compact typed facts from one selected native expression and context.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TypedFacts {
    /// Actual expression whose type was queried, mapped to original source when unique.
    pub expression: Option<NativeLocation>,
    /// Original inferred type where the native provider supplies one.
    pub original: Option<TypeFact>,
    /// Adjusted type where the native provider performed an adjustment.
    pub adjusted: Option<TypeFact>,
    /// Actual native type equality, never display-string equality.
    pub original_equals_adjusted: Option<bool>,
    /// Actual native coercion query for the observed original/adjusted pair.
    pub original_coerces_to_adjusted: Option<bool>,
    /// Resolved explicit or implicit method target where supported.
    pub target: Option<TargetFact>,
    /// Native declared generic context.
    pub bounds: Vec<BoundFact>,
}
