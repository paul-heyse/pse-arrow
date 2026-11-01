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
    /// Native import role; importing is not execution.
    pub import: Option<bool>,
    /// Native test-context role, without execution or reachability claims.
    pub test: Option<bool>,
}
/// Native predicate evaluation, never reconstructed from hover text.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PredicateFact {
    /// Native evaluation status, retaining a failed proof independently from false.
    pub outcome: PredicateOutcome,
    /// Original provider explanation where supplied.
    pub cause: Option<String>,
}
/// A predicate solver outcome is not a Boolean negation proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PredicateOutcome {
    /// Contextual proof succeeded.
    Holds,
    /// Contextual proof was not established.
    NotProven,
    /// Input could not be interpreted in this context.
    Invalid,
    /// The selected provider cannot evaluate this question.
    Unsupported,
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
    /// Original native test identifier, independent of presentation labels.
    pub test_id: Option<String>,
    /// Original native cfg expression rendering; not reconstructed eligibility.
    pub cfg: Option<String>,
    /// Why this runnable was returned for this selected source.
    pub relatedness: Option<String>,
    /// Eligibility only when the selected native world establishes it.
    pub eligible: Option<bool>,
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
    /// Native IMPORT category.
    pub import: bool,
    /// Native TEST category.
    pub test: bool,
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
    /// Actual selected crate premises; never inferred from rendered type names.
    pub context: Option<TypeContext>,
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
    /// Exact operation occurrence, distinct from its resolved declaration.
    pub callsite: Option<NativeLocation>,
    /// Ordered native coercion/autoref/autoderef steps.
    pub adjustments: Vec<AdjustmentFact>,
    /// Closure/coroutine capture observations when selected.
    pub captures: Vec<CaptureFact>,
    /// Actual enclosing closure/coroutine source, absent when original mapping is not unique.
    pub capture_owner: Option<NativeLocation>,
    /// Native callable category, including async callable traits.
    pub callable_kind: Option<String>,
    /// Fallible layout, computed only by an explicit layout request.
    pub layout: Option<LayoutFact>,
    /// Contextual callable discovery, computed only on explicit demand.
    pub callable_candidates: Vec<CallableCandidateFact>,
}
/// Owned context for a native type/layout observation.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TypeContext {
    /// Actual loaded crate root, preserving original path bytes.
    #[serde(with = "crate::investigation::native_path_serde")]
    #[schemars(with = "crate::investigation::NativePath")]
    pub crate_root: PathBuf,
    /// Captured root bytes.
    pub crate_source_sha256: Option<String>,
    /// Actual native feature atoms.
    pub features: std::collections::BTreeSet<String>,
    /// Actual native cfg atoms, including test context.
    pub cfg: Vec<String>,
    /// Actual loaded edition.
    pub edition: String,
    /// Unique captured target profile; host/proc-macro ambiguity remains absent.
    pub target: Option<String>,
    /// Actual compiler verbose identity captured at preparation.
    pub compiler_identity: String,
}
/// Native callable applicability and import observations retain separate proof boundaries.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CallableCandidateFact {
    /// Genuine source-qualified declaration; names are display only.
    pub target: TargetFact,
    /// Native declaration signature, with generic substitution limits explicit.
    pub signature: String,
    /// Actual receiver used for native candidate discovery.
    pub receiver: TypeFact,
    /// Contextual trait implementation observation; no false proof from failure.
    pub applicability: PredicateOutcome,
    /// Native use path of an extension trait absent from the current scope.
    pub required_import: Option<String>,
    /// Which native operation supplied the candidate and what remains unproved.
    pub reasons: Vec<String>,
}

/// A native adjustment's source and destination types in order.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdjustmentFact {
    /// Native adjustment classification and its library-owned detail.
    pub kind: String,
    /// Type before this step.
    pub source: TypeFact,
    /// Type after this step.
    pub target: TypeFact,
}
/// A captured place and its actual closure storage type.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaptureFact {
    /// Unique native local declaration; ambiguity stays absent rather than selecting a source.
    pub local: Option<NativeLocation>,
    /// Native place rendering, presentation rather than portable identity.
    pub place: String,
    /// True when HIR retains field projections.
    pub has_field_projections: bool,
    /// Move/shared/unique/mutable capture mode.
    pub mode: String,
    /// Captured place type.
    pub place_type: TypeFact,
    /// Actual stored type, including a capture reference.
    pub stored_type: TypeFact,
    /// Actual native capture uses; absent means this observation was not supplied.
    pub usages: Option<Vec<CaptureUseFact>>,
}
/// Native capture-use source, not runtime reach or borrow-checker evidence.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaptureUseFact {
    /// Exact original range when uniquely mapped; macro ambiguity remains absent.
    pub location: Option<NativeLocation>,
    /// Native use was a reference expression/pattern.
    pub is_ref: bool,
}
/// Fallible target-qualified layout, never a heap-memory measurement.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LayoutFact {
    /// Native type rendering.
    pub ty: String,
    /// Size only when native computation succeeded.
    pub size: Option<u64>,
    /// Alignment only when native computation succeeded.
    pub align: Option<u64>,
    /// Native Sized obligation; NotProven never means a negative proof.
    pub sized: PredicateOutcome,
    /// Actual static prefix size when full value size is not established.
    pub prefix_size: Option<u64>,
    /// Actual static prefix alignment when full value alignment is not established.
    pub prefix_align: Option<u64>,
    /// Native layout failure or unavailable full-size precondition, never a zero placeholder.
    pub error: Option<String>,
    /// Available niche count as decimal text, preserving u128.
    pub niches: Option<String>,
    /// Explicitly requested immediate children; each retains its own failure.
    pub fields: Vec<LayoutFieldFact>,
}
/// Immediate field layout; offset absence remains distinct from zero.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LayoutFieldFact {
    /// Native field name or tuple index.
    pub name: String,
    /// Actual parent-layout offset where supplied.
    pub offset: Option<u64>,
    /// Fallible child layout.
    pub layout: Box<LayoutFact>,
}
