// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One typed meaning for requests, addressed results and streamed observations.
use super::Error;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};

/// Current investigation request and locator format.
pub const VERSION: u32 = 1;

/// The same request is constructed by flags, JSON and saved configuration.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    /// Explicit interpretation version.
    pub version: u32,
    /// Actual source universe.
    pub sources: SourceSelector,
    /// Optional named source scope, resolved by the shared configuration owner.
    pub scope: Option<String>,
    /// Scoped native question.
    pub predicate: Predicate,
    /// Work and completion requested from providers.
    pub answer: AnswerDemand,
    /// Independent detail choices.
    pub presentation: QueryPresentation,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            version: VERSION,
            sources: SourceSelector::default(),
            scope: None,
            predicate: Predicate::Files,
            answer: AnswerDemand::default(),
            presentation: QueryPresentation::default(),
        }
    }
}
impl Request {
    /// Validate shared semantics before acquisition or provider execution.
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION {
            return Err(Error::Invalid("unsupported request version".into()));
        }
        if self.answer.limit == Some(0) {
            return Err(Error::Invalid(
                "result limit must be positive; use exists for a witness".into(),
            ));
        }
        if self.predicate.has_related() && self.sources.kind != SourceKind::Working {
            return Err(Error::Invalid("native relationships require working answer sources; index/revision sources cannot be reinterpreted as current native semantics".into()));
        }
        if self.answer.cursor.is_some() && self.answer.order != ResultOrder::Stable {
            return Err(Error::Invalid(
                "continuation requires stable ordering".into(),
            ));
        }
        if super::composition::required(&self.predicate) {
            if self.answer.cursor.is_some() {
                return Err(Error::Invalid("a source-only cursor cannot continue session-bound composed native relationships".into()));
            }
            if self.answer.order == ResultOrder::Stable
                && !matches!(self.predicate, Predicate::Related { .. })
            {
                return Err(Error::Invalid("stable native composition is supported for bounded related populations; candidate refinement/union requires arrival order".into()));
            }
        }
        if matches!(&self.sources.kind, SourceKind::Revision { revision } if revision.is_empty()) {
            return Err(Error::Invalid("revision must not be empty".into()));
        }
        for field in &self.presentation.fields {
            if !matches!(field.as_str(), "name" | "context" | "captures" | "bytes") {
                return Err(Error::Invalid(format!(
                    "unsupported result detail field {field}; source/reference/range/kind are mandatory"
                )));
            }
        }
        if self
            .presentation
            .group_by
            .as_deref()
            .is_some_and(|group| !matches!(group, "source" | "kind" | "name"))
        {
            return Err(Error::Invalid(
                "group_by must be source, kind or name".into(),
            ));
        }
        if self.answer.kind == AnswerKind::Count && self.presentation.group_by.is_some() {
            return Err(Error::Invalid("group_by groups returned match witnesses, not count aggregates; request matches with a chosen limit or exhaustive examination".into()));
        }
        self.predicate.validate()
    }
    /// Derive the actual input-direction contract, rather than hand authoring JSON Schema.
    pub fn input_schema() -> schemars::Schema {
        SchemaSettings::draft2020_12()
            .for_deserialize()
            .into_generator()
            .into_root_schema_for::<Self>()
    }
}

/// Source selection is separate from active compiler membership.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SourceSelector {
    /// Working, index or a resolved revision universe.
    pub kind: SourceKind,
    /// Explicit files or directory roots.
    #[serde(with = "native_paths")]
    #[schemars(with = "Vec<NativePath>")]
    pub paths: Vec<PathBuf>,
    /// Package restrictions resolved by the native metadata owner.
    pub packages: Vec<String>,
    /// Native file-type names, such as rust or markdown.
    pub types: Vec<String>,
    /// None includes both tracked and untracked working sources.
    pub tracked: Option<bool>,
    /// Included path globs.
    pub globs: Vec<String>,
    /// Excluded path globs.
    pub exclude: Vec<String>,
    /// Widen ignore policy explicitly.
    pub include_ignored: bool,
    /// Include hidden entries explicitly.
    pub include_hidden: bool,
    /// Include protected generated sources for reading.
    pub include_generated: bool,
    /// Include dependency sources explicitly.
    pub include_dependencies: bool,
    /// Restrict to the selected native change population.
    pub changed: bool,
}
/// Acquisition source; a provider resolves revision selectors before publishing facts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceKind {
    /// Current independent working-file observations.
    #[default]
    Working,
    /// Stage-zero index objects.
    Index,
    /// Explicit committed revision.
    Revision {
        /// Revision selector, resolved by acquisition.
        revision: String,
    },
}
/// Unit that must contain the Boolean witnesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchScope {
    /// One line.
    Line,
    /// One file, potentially different nodes.
    File,
    /// One native syntax node.
    Node,
    /// One resolved entity in one context.
    Entity,
}
/// Native clauses and explicitly scoped combinations; no free-form application DSL.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Predicate {
    /// Discover selected source paths.
    #[default]
    Files,
    /// Native text matching.
    Text(TextQuery),
    /// Native structural matching.
    Syntax(SyntaxQuery),
    /// Name-first declaration discovery.
    Symbol(SymbolQuery),
    /// Acquired API search.
    Api(ApiQuery),
    /// Offline discovery in one immutable supplied descriptor world.
    Schema(crate::schema::search::Query),
    /// Structured adjacent content.
    Structured(StructuredQuery),
    /// Native history question.
    History(HistoryQuery),
    /// Refine native candidates through resolved context facts, never textual-name joins.
    Refine {
        /// Native candidate producer.
        candidates: Box<Predicate>,
        /// Context-bound refinement.
        filter: Refinement,
    },
    /// Traverse selected native relationships from guarded seeds in one prepared world.
    Related {
        /// Original source/entity/API addresses; their order defines seed attribution.
        seeds: Vec<ResultReference>,
        /// Nonempty selected native relationship families.
        relations: Vec<Relationship>,
        /// Domain admitting seeds and supporting intermediates, independent of answer sources.
        traversal: SourceSelector,
        /// Whether any seed or every seed must establish an ordinary answer.
        #[serde(default)]
        seed_match: super::graph::SeedMatch,
        /// Incoming uses/callers or outgoing declaration/call targets; unsupported inverse edges refuse.
        #[serde(default)]
        direction: Direction,
        /// Maximum edge distance; defaults to one hop.
        #[serde(default = "one")]
        depth: usize,
        /// Maximum distinct native locations admitted to traversal.
        #[serde(default = "relationship_nodes")]
        max_nodes: usize,
        /// Maximum native edge observations examined, independent of rendered rows.
        #[serde(default = "relationship_edges")]
        max_edges: usize,
    },
    /// All clauses have witnesses within the stated unit.
    All {
        /// Explicit conjunction unit.
        within: MatchScope,
        /// Native clauses.
        clauses: Vec<Predicate>,
    },
    /// At least one clause has a witness.
    Any {
        /// Explicit alternative unit.
        within: MatchScope,
        /// Native clauses.
        clauses: Vec<Predicate>,
    },
    /// Absence requires sufficiently complete examination.
    Not {
        /// Explicit negation unit.
        within: MatchScope,
        /// Native clause.
        clause: Box<Predicate>,
    },
}
impl Predicate {
    fn has_related(&self) -> bool {
        match self {
            Self::Related { .. } => true,
            Self::All { clauses, .. } | Self::Any { clauses, .. } => {
                clauses.iter().any(Self::has_related)
            }
            Self::Not { clause, .. } => clause.has_related(),
            Self::Refine { candidates, .. } => candidates.has_related(),
            _ => false,
        }
    }
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Schema(query) => query.validate(),
            Self::Text(q) if q.patterns.is_empty() => {
                Err(Error::Invalid("text search requires a pattern".into()))
            }
            Self::Syntax(q)
                if usize::from(q.pattern.is_some())
                    + usize::from(q.rule.is_some())
                    + usize::from(q.tree_sitter.is_some())
                    != 1 =>
            {
                Err(Error::Invalid(
                    "select exactly one syntax pattern, native rule or Tree-sitter query".into(),
                ))
            }
            Self::Symbol(q) if q.name.is_empty() => {
                Err(Error::Invalid("symbol name must not be empty".into()))
            }
            Self::All { clauses, .. } | Self::Any { clauses, .. } => {
                if clauses.is_empty() {
                    return Err(Error::Invalid("Boolean group must not be empty".into()));
                }
                for clause in clauses {
                    clause.validate()?;
                }
                Ok(())
            }
            Self::Not { clause, .. } => clause.validate(),
            Self::Refine { candidates, filter } => {
                candidates.validate()?;
                match filter {
                    Refinement::NativePredicate { text } if text.trim().is_empty() => Err(
                        Error::Invalid("native predicate text must not be empty".into()),
                    ),
                    Refinement::ResolvesTo { target }
                    | Refinement::UsedAt {
                        destination: target,
                    }
                    | Refinement::ImportableAt {
                        destination: target,
                    } => target.validate(),
                    _ => Ok(()),
                }
            }
            Self::Related {
                seeds,
                relations,
                traversal,
                depth,
                max_nodes,
                max_edges,
                ..
            } => {
                if seeds.is_empty() || relations.is_empty() {
                    return Err(Error::Invalid(
                        "relationship seeds and relations must not be empty".into(),
                    ));
                }
                if relations
                    .iter()
                    .enumerate()
                    .any(|(index, relation)| relations[..index].contains(relation))
                {
                    return Err(Error::Invalid(
                        "relationship relations must select each kind once".into(),
                    ));
                }
                if *depth == 0 || *max_nodes == 0 || *max_edges == 0 {
                    return Err(Error::Invalid("relationship depth, max_nodes and max_edges must be positive finite bounds".into()));
                }
                if seeds.len() > *max_nodes {
                    return Err(Error::Invalid(
                        "relationship max_nodes must admit every seed ordinal".into(),
                    ));
                }
                if traversal.kind != SourceKind::Working {
                    return Err(Error::Invalid(
                        "relationship traversal requires a working source domain".into(),
                    ));
                }
                for seed in seeds {
                    seed.validate()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
fn one() -> usize {
    1
}
fn relationship_nodes() -> usize {
    100
}
fn relationship_edges() -> usize {
    1000
}
/// Native context refinement; unavailable facts remain unknown rather than false.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Refinement {
    /// Resolve candidates to exactly this declaration in one native context.
    ResolvesTo {
        /// A guarded declaration or source position.
        target: ResultReference,
    },
    /// Ask the actual prepared semantic owner's structured predicate operation.
    NativePredicate {
        /// Native provider predicate syntax.
        text: String,
    },
    /// Keep contextual API candidates only where the native destination resolves that same entity.
    UsedAt {
        /// Exact destination source/entity position.
        destination: ResultReference,
    },
    /// Require a native positive importability fact at the selected destination.
    ImportableAt {
        /// Exact native destination context.
        destination: ResultReference,
    },
}
/// Selected native graph edge, not runtime reachability.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    /// Resolved declaration target.
    Definition,
    /// Native references including their actual declaration policy.
    References,
    /// Incoming or outgoing native calls.
    Calls,
    /// Native implementation targets.
    Implementations,
    /// Native declaration containment.
    Contains,
    /// Nominal type dependency of a native parameter position.
    ParameterType,
    /// Nominal declared return or async output type dependency.
    ReturnType,
    /// Nominal type dependency of a native field position.
    FieldType,
    /// Explicit native resolved use, distinct from calls and implicit language operations.
    Uses,
}
/// Explicit relationship direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Native outgoing targets.
    #[default]
    Outgoing,
    /// Native callers/references toward the selected entity.
    Incoming,
}
/// One resolved declaration belongs to an original prepared semantic world.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntityIdentity {
    /// Original physical semantic session owner.
    pub session_invocation: String,
    /// Exact selected compiler/model configuration.
    pub configuration: String,
    /// Native world revision.
    pub generation: u64,
    /// Native declaration source, preserving platform bytes.
    pub declaration: NativePath,
    /// Exact native declaration interval.
    pub byte_start: usize,
    /// Half-open native declaration endpoint.
    pub byte_end: usize,
}
/// Native regex case policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Case {
    /// Case sensitive.
    #[default]
    Sensitive,
    /// Native smart case.
    Smart,
    /// Case insensitive.
    Insensitive,
}
/// Binary-content policy remains visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BinaryPolicy {
    /// Native binary exclusion.
    #[default]
    Skip,
    /// Explicitly search bytes as text.
    Text,
}
/// Native text request; multiple patterns are alternatives.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TextQuery {
    /// Native patterns.
    pub patterns: Vec<String>,
    /// Treat patterns as literals.
    pub literal: bool,
    /// Case policy.
    pub case: Case,
    /// Match whole words.
    pub word: bool,
    /// Match whole lines.
    pub whole_line: bool,
    /// Permit multiline matches.
    pub multiline: bool,
    /// Native inverse-line selection.
    pub invert: bool,
    /// Explicit searched encoding.
    pub encoding: Option<String>,
    /// Binary input policy.
    pub binary: BinaryPolicy,
}
/// Native AST rules and Tree-sitter query subtrees are validated by their provider.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SyntaxQuery {
    /// Code-shaped native pattern.
    pub pattern: Option<String>,
    /// Native ast-grep rule subtree.
    pub rule: Option<Value>,
    /// Native Tree-sitter extraction query.
    pub tree_sitter: Option<String>,
    /// Requested capture names.
    pub capture: Vec<String>,
    /// Native strictness selection.
    pub strictness: Option<String>,
}
/// Explicit symbol discovery authority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryProvider {
    /// Written declarations without a session.
    #[default]
    Source,
    /// An explicitly prepared semantic world.
    Semantic,
    /// An already acquired API artifact.
    Api,
}
/// Native name-selection policy; fuzzy matching requires a provider with actual fuzzy search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NameMatch {
    /// Contiguous substring in native names.
    #[default]
    Contains,
    /// Exact native name.
    Exact,
    /// Native name prefix.
    Prefix,
    /// Provider-owned fuzzy/ranked candidate search.
    Fuzzy,
}
/// Name-first query retains provider context.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SymbolQuery {
    /// Simple or qualified name.
    pub name: String,
    /// Selected native kinds.
    pub kinds: Vec<String>,
    /// Module constraint.
    pub module: Option<String>,
    /// Visibility constraint.
    pub visibility: Option<String>,
    /// Requested authority.
    pub provider: DiscoveryProvider,
    /// Explicitly acquired artifact for API-backed symbol discovery.
    pub artifact: Option<PathBuf>,
    /// Exact dependency selector for API-backed symbol discovery.
    pub dependency: Option<String>,
    /// Matching policy; a provider must not synthesize fuzzy semantics.
    pub matching: NameMatch,
    /// Optional written declaration signature selector, separate from a resolved type predicate.
    pub signature: Option<String>,
}
/// Acquired API filters, retaining the artifact and package context.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ApiQuery {
    /// Name/docs query.
    pub query: String,
    /// Selected dependency.
    pub dependency: Option<String>,
    /// Explicit acquired artifact.
    pub artifact: Option<PathBuf>,
    /// Input-type shape.
    pub input: Option<String>,
    /// Output-type shape.
    pub output: Option<String>,
    /// Structurally selected trait/generic bound in the acquired API model.
    pub bound: Option<String>,
}
/// Bounded structured-value selection.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct StructuredQuery {
    /// JSON Pointer into the parsed native value.
    pub pointer: String,
    /// Optional typed equality.
    pub equals: Option<Value>,
    /// Explicit format, otherwise file extension selects the parser.
    pub format: Option<String>,
}
/// Selected native historical operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HistoryOperation {
    /// Native change history.
    #[default]
    Log,
    /// Selected blame ranges.
    Blame,
    /// String-count change history.
    Pickaxe,
    /// Native changed-line regular-expression pickaxe (`git log -G`).
    PickaxeLines,
}
/// Revision-aware history request.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct HistoryQuery {
    /// Native history mode.
    pub operation: HistoryOperation,
    /// Native string/path query when required.
    pub query: Option<String>,
    /// Explicit starting revision.
    pub revision: Option<String>,
}
/// Demand controls upstream work, not only rendering.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct AnswerDemand {
    /// Requested answer kind.
    pub kind: AnswerKind,
    /// None requests exhaustive matches.
    pub limit: Option<usize>,
    /// Explicit ordering/completion policy.
    pub order: ResultOrder,
    /// Count interpretation.
    pub count_unit: CountUnit,
    /// Guarded deterministic continuation.
    pub cursor: Option<String>,
}
impl Default for AnswerDemand {
    fn default() -> Self {
        Self {
            kind: AnswerKind::Matches,
            limit: Some(20),
            order: ResultOrder::Arrival,
            count_unit: CountUnit::Matches,
            cursor: None,
        }
    }
}
/// Native result shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnswerKind {
    /// Match witnesses.
    #[default]
    Matches,
    /// Paths with witnesses.
    Files,
    /// One sufficient witness.
    Exists,
    /// Explicit exact counting.
    Count,
}
/// Arrival order does not promise a repeatable cursor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultOrder {
    /// Useful available matches first.
    #[default]
    Arrival,
    /// Stable source/location order.
    Stable,
}
/// Counts state what they count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CountUnit {
    /// Individual native matches.
    #[default]
    Matches,
    /// Matching lines.
    Lines,
    /// Matching files.
    Files,
    /// Distinct provider entities.
    Entities,
}
/// Presentation choices do not assert exhaustive examination.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct QueryPresentation {
    /// Context lines or native witness detail.
    pub context: usize,
    /// Selected result detail fields.
    pub fields: Vec<String>,
    /// Optional grouping dimension.
    pub group_by: Option<String>,
    /// Whole result records published; zero publishes only summaries without changing demand.
    pub budget: Option<usize>,
}
impl Default for QueryPresentation {
    fn default() -> Self {
        Self {
            context: 1,
            fields: Vec::new(),
            group_by: None,
            budget: None,
        }
    }
}

/// Ordinary paths are readable strings; exceptional Unix bytes remain exact.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum NativePath {
    /// Ordinary UTF-8 path.
    Utf8(String),
    /// Exact Unix-native path bytes.
    Raw {
        /// Raw path bytes, not lossy replacement text.
        unix: Vec<u8>,
    },
}
mod native_paths {
    use super::NativePath;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    pub(super) fn serialize<S: serde::Serializer>(
        paths: &[PathBuf],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        paths
            .iter()
            .cloned()
            .map(NativePath::from)
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<PathBuf>, D::Error> {
        Ok(Vec::<NativePath>::deserialize(deserializer)?
            .iter()
            .map(NativePath::to_path_buf)
            .collect())
    }
}
impl From<PathBuf> for NativePath {
    fn from(path: PathBuf) -> Self {
        match path.to_str() {
            Some(text) => Self::Utf8(text.into()),
            None => {
                use std::os::unix::ffi::OsStrExt as _;
                Self::Raw {
                    unix: path.as_os_str().as_bytes().to_vec(),
                }
            }
        }
    }
}
impl NativePath {
    /// Recover exact native bytes on the supported Unix host.
    pub fn to_path_buf(&self) -> PathBuf {
        use std::os::unix::ffi::OsStringExt as _;
        match self {
            Self::Utf8(path) => path.into(),
            Self::Raw { unix } => std::ffi::OsString::from_vec(unix.clone()).into(),
        }
    }
}
/// Half-open original byte locations with one-based line witnesses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MatchRange {
    /// Inclusive byte start.
    pub byte_start: usize,
    /// Exclusive byte end.
    pub byte_end: usize,
    /// One-based first line.
    pub line_start: usize,
    /// One-based last line.
    pub line_end: usize,
}
/// Source facts shared once per response or stream.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SourceContext {
    /// Response-local source identifier.
    pub id: String,
    /// Readable native operand.
    pub path: NativePath,
    /// Acquisition origin.
    pub origin: Option<SourceKind>,
    /// Actual provider.
    pub provider: String,
    /// Known source or provider generation.
    pub generation: Option<String>,
    /// Explicit searched encoding; None means original bytes.
    pub searched_encoding: Option<String>,
    /// Actual shared native world, including empty/fully filtered answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<NativeContext>,
}
/// Actual native reply context, reused instead of restating its backend/configuration fields.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeContext {
    /// Original authenticated semantic session.
    pub session_invocation: String,
    /// Original observed native world revision.
    pub generation: u64,
    /// Library-owned analysis context and preparation limits.
    pub analysis: crate::semantic::AnalysisContext,
}
/// Exact snapshot or lightweight-observation premises; absent fields assert nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceGuard {
    /// Original absolute source universe; relative locators cannot silently change checkouts.
    #[serde(default)]
    pub root: Option<NativePath>,
    /// Exact complete source hash only when actually acquired.
    pub sha256: Option<String>,
    /// Observed original byte length.
    pub length: Option<u64>,
    /// Observed filesystem modification time.
    pub modified_nanos: Option<u64>,
    /// Matched original byte witness, without claiming a whole-file snapshot.
    #[serde(default)]
    pub observed: Option<ObservedGuard>,
    /// Original native Git blob object; name-only enumeration need not acquire its bytes.
    #[serde(default)]
    pub git_blob: Option<String>,
}
/// A lexical witness binds only the byte interval actually observed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedGuard {
    /// Original observed byte range.
    pub range: MatchRange,
    /// SHA-256 of precisely the selected original bytes.
    pub sha256: String,
}
/// Explicit original evidence category.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// Original compiler diagnostic.
    Diagnostic,
    /// Retained immutable artifact.
    Artifact,
    /// Original log bytes.
    Log,
    /// Captured test population.
    TestPopulation,
}
/// Typed locations bypass an unnecessary search.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Locator {
    /// A declaration in one exact immutable descriptor artifact.
    Schema {
        /// Original descriptor artifact, preserving native path bytes.
        descriptor: NativePath,
        /// Digest of the original complete artifact bytes.
        descriptor_sha256: String,
        /// Descriptor declaration address, never a native Rust identity.
        declaration: String,
    },
    /// Explicit source/range, optionally guarded.
    Source {
        /// Native operand.
        path: NativePath,
        /// Original source universe.
        source: SourceKind,
        /// Selected native coordinates.
        range: Option<MatchRange>,
        /// Actual original premises.
        guard: Option<SourceGuard>,
    },
    /// One acquired API item.
    Api {
        /// Acquired original artifact.
        artifact: PathBuf,
        /// Artifact-local item identity.
        item: String,
        /// Exact original acquired artifact bytes, when known.
        #[serde(default)]
        artifact_sha256: Option<String>,
        /// Exact original extraction-context identity.
        #[serde(default)]
        context_key: Option<String>,
        /// Original rustdoc-qualified public route, preserving distinct reexports.
        #[serde(default)]
        public_path: Option<String>,
        /// Exact separately acquired definition used to qualify a public reexport.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        definition: Option<crate::api::query::DefinitionEvidence>,
    },
    /// Original retained evidence.
    Evidence {
        /// Original invocation.
        invocation: String,
        /// Evidence category, separate from locator discrimination.
        #[serde(rename = "evidence_kind")]
        kind: EvidenceKind,
        /// Native bounded selector.
        selector: Option<String>,
    },
    /// Context-bound semantic source position.
    Semantic {
        /// Prepared project.
        project: PathBuf,
        /// Original file.
        path: NativePath,
        /// One-based line.
        line: usize,
        /// One-based Unicode scalar column.
        column: usize,
        /// Known native generation.
        generation: Option<String>,
        /// Exact original source hash when the source was acquired.
        #[serde(default)]
        source_sha256: Option<String>,
        /// Exact lazy native candidate/hint/assist fingerprint.
        #[serde(default)]
        candidate_fingerprint: Option<String>,
        /// Original semantic session owner; generations are session-local.
        #[serde(default)]
        session_invocation: Option<String>,
    },
}
/// Portable source-bound address; no response database is required.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultReference {
    /// Explicit locator version.
    pub version: u32,
    /// Minimum addressing and freshness premises.
    pub locator: Locator,
}
impl ResultReference {
    /// Construct a current typed reference.
    pub fn new(locator: Locator) -> Self {
        Self {
            version: VERSION,
            locator,
        }
    }
    /// Encode URL-safe, shell-safe bytes without losing native paths.
    pub fn encode(&self) -> Result<String, Error> {
        Ok(URL_SAFE_NO_PAD.encode(serde_json::to_vec(self)?))
    }
    /// Refuse unknown interpretation before using a supplied address.
    pub fn decode(value: &str) -> Result<Self, Error> {
        let value = value.trim();
        let result: Self = if value.starts_with('{') {
            serde_json::from_str(value)?
        } else {
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(value)?)?
        };
        result.validate()?;
        Ok(result)
    }
    /// Validate the same premises for inline request references and portable CLI forms.
    pub fn validate(&self) -> Result<(), Error> {
        if let Locator::Schema {
            descriptor,
            descriptor_sha256,
            declaration,
        } = &self.locator
            && (!descriptor.to_path_buf().is_absolute()
                || descriptor_sha256.len() != 64
                || !descriptor_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
                || declaration.is_empty())
        {
            return Err(Error::Invalid("schema reference requires an absolute artifact, SHA256 identity and declaration address".into()));
        }
        if self.version != VERSION {
            return Err(Error::Invalid(
                "unsupported result reference version".into(),
            ));
        }
        if let Locator::Source {
            range: Some(range), ..
        } = &self.locator
            && (range.byte_end < range.byte_start
                || range.line_start == 0
                || range.line_end < range.line_start)
        {
            return Err(Error::Invalid("invalid result coordinates".into()));
        }
        if let Locator::Source {
            guard: Some(SourceGuard {
                root: Some(root), ..
            }),
            ..
        } = &self.locator
            && !root.to_path_buf().is_absolute()
        {
            return Err(Error::Invalid("source guard root must be absolute".into()));
        }
        Ok(())
    }
}
/// Compact row with original witnesses, not repeated provider metadata.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    /// Native written source containment/call context, preserving its original source identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_relationship: Option<crate::source_query::SourceRelationship>,
    /// Shared source identifier.
    pub source: String,
    /// Native result kind.
    pub kind: String,
    /// Qualified name where available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Original byte/line location where available.
    pub range: Option<MatchRange>,
    /// Short native witness.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub context: String,
    /// Exact exceptional witness bytes when UTF-8 display is insufficient.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bytes: Option<Vec<u8>>,
    /// Portable guarded address where available.
    pub reference: Option<ResultReference>,
    /// Exact pre-refinement source/API witness, distinct from its resolved native context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_reference: Option<ResultReference>,
    /// Native refinement state; unknown structural witnesses are not positive matches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refinement: Option<RefinementFact>,
    /// Resolved declaration/context when native refinement actually established one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityIdentity>,
    /// Named native capture locations.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub captures: BTreeMap<String, Vec<MatchRange>>,
}
/// Absence and unavailable facts compose differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Truth {
    /// Positive native witness.
    True,
    /// Complete negative observation.
    False,
    /// Insufficient/unsupported evidence.
    Unknown,
}
/// Original structural candidate and typed refinement outcome remain distinct facts.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RefinementFact {
    /// Proven match, contradiction or unproved/unsupported enrichment.
    pub truth: Truth,
    /// Original classification/reason where refinement is not established.
    pub diagnostic: Option<QueryDiagnostic>,
    /// Original typed native refusal, preserving backend code and opaque evidence.
    pub native: Option<crate::semantic::Failure>,
    /// Actual native predicate/importability subtree, without parsing rendered descriptions.
    pub evidence: Option<Value>,
}
impl Truth {
    /// Three-valued conjunction never invents a negative from unknown information.
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Unknown,
        }
    }
    /// Three-valued alternatives preserve sufficient positive witnesses.
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Unknown,
        }
    }
    /// Unknown absence is still unknown.
    pub fn negate(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}
/// Precision is part of the count, rather than an invented omitted total.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "precision", content = "value", rename_all = "snake_case")]
pub enum Count {
    /// Complete native count.
    Exact(u64),
    /// Known lower bound.
    AtLeast(u64),
    /// Not computed.
    #[default]
    Unknown,
}
/// Why new work stopped; completion of submitted work is separately acknowledged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Requested universe examined.
    #[default]
    Complete,
    /// Requested answer demand satisfied early.
    Limit,
    /// User cancellation observed.
    Cancelled,
    /// Result consumer closed.
    ConsumerClosed,
}
/// Terminal work facts emitted only after actual owned settlement.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    /// Files actually examined.
    pub examined_files: u64,
    /// Bytes actually examined.
    pub examined_bytes: u64,
    /// Native match witnesses observed.
    pub matches: u64,
    /// Complete rows delivered.
    pub emitted: u64,
    /// Independent native failures.
    pub failures: u64,
    /// Unavailable/unknown predicate observations.
    pub unknown: u64,
    /// Complete examination, not merely requested-demand satisfaction.
    pub complete: bool,
    /// Cooperative cancellation was observed.
    pub cancellation_requested: bool,
    /// Actual admission stopping reason.
    pub stop: StopReason,
    /// Count precision.
    pub count: Count,
    /// Guarded stable continuation, never arrival-order offset arithmetic.
    pub next_cursor: Option<String>,
    /// Requested grouping over zero-based ordinals of actually delivered result rows.
    #[serde(default)]
    pub groups: BTreeMap<String, Vec<u64>>,
    /// Intentionally omitted detail fields; mandatory addressing meaning is retained.
    #[serde(default)]
    pub omitted_fields: Vec<String>,
    /// Shared witnessed relationship evidence, retained independently of row/text projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipEvidence>,
}
impl Default for Summary {
    fn default() -> Self {
        Self {
            examined_files: 0,
            examined_bytes: 0,
            matches: 0,
            emitted: 0,
            failures: 0,
            unknown: 0,
            complete: true,
            cancellation_requested: false,
            stop: StopReason::Complete,
            count: Count::Unknown,
            next_cursor: None,
            groups: BTreeMap::new(),
            omitted_fields: Vec::new(),
            relationships: Vec::new(),
        }
    }
}
/// One graph node retains native declaration identity and guarded expansion.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipNode {
    /// Index used by this answer's canonical edges and attribution.
    pub id: usize,
    /// Original resolved declaration identity, absent for a reference-use occurrence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityIdentity>,
    /// Guarded original declaration action.
    pub reference: ResultReference,
    /// Presentation only; never used to join identities.
    pub name: Option<String>,
}
/// One original occurrence preserves role, type context and a guarded source action.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipWitness {
    /// Index used by the canonical edge.
    pub id: usize,
    /// Guarded original occurrence action, absent when mapping is unavailable.
    pub reference: Option<ResultReference>,
    /// Producer-supplied structural/native role.
    pub role: crate::semantic::GraphRole,
    /// Native declaration-owned rendering, never parsed to discover endpoints.
    pub declared_type: Option<String>,
    /// Native contextual rendering, separate from declaration-owned type.
    pub instantiated_type: Option<String>,
    /// Original mapping and evidence limits.
    pub limitations: Vec<String>,
}
/// Compact final graph answer; coverage is independent of a rendered match allowance.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipEvidence {
    /// Original seed ordinals as normalized request-local node indexes.
    pub seeds: Vec<usize>,
    /// Requested relation families, including unsupported or empty observations.
    pub relations: Vec<Relationship>,
    /// Requested walk direction; edge endpoints remain canonical.
    pub direction: Direction,
    /// Required seed-attribution rule for selected answer nodes.
    pub seed_match: super::graph::SeedMatch,
    /// Declared traversal domain, separate from ordinary answer selection.
    pub traversal: SourceSelector,
    /// Intentional edge-distance boundary.
    pub depth: usize,
    /// Shared selected and supporting identities.
    pub nodes: Vec<RelationshipNode>,
    /// Original edge-occurrence evidence.
    pub witnesses: Vec<RelationshipWitness>,
    /// Attribution, canonical edges, explicit frontier and actual stopping reason.
    pub graph: super::graph::TraversalResult,
    /// Native provider limits, not fabricated negative facts.
    pub limitations: Vec<String>,
}
/// Original ordered query diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticPhase {
    /// Input could not be decoded as the declared request schema.
    Schema,
    /// A decoded request contradicts the selected operation's constraints.
    Constraint,
    /// The request validator failed to establish validity.
    Validator,
    /// Source, artifact or prepared-world premises are missing or stale.
    Context,
    /// Original work settlement could not be observed or has a qualified completion state.
    Settlement,
    /// The selected provider could not establish the requested observation.
    Provider,
    /// Complete records could not reach the selected consumer.
    Delivery,
}
/// Original ordered query diagnostic.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryDiagnostic {
    /// The failing boundary, independent of native diagnostic wording.
    pub phase: DiagnosticPhase,
    /// Native typed category.
    pub code: String,
    /// Original explanation.
    pub message: String,
    /// Selected source when known.
    pub source: Option<String>,
    /// Structured native cause and query coordinates, when the producer supplies them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Box<crate::envelope::ErrorRecord>>,
}
/// Complete typed stream records, with one terminal coverage summary.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// Original typed native refusal; no local error is relabelled as remote evidence.
    NativeFailure {
        /// Exactly the owner's structured reply.
        failure: Box<crate::semantic::Failure>,
    },
    /// Actual internal progress facts; the CLI renders these on stderr, never machine stdout.
    #[serde(skip)]
    Progress {
        /// Files actually examined.
        examined_files: u64,
        /// Bytes actually examined.
        examined_bytes: u64,
        /// Native matches observed.
        matches: u64,
    },
    /// Declare one invocation and its effective query.
    Begin {
        /// Original selected source/control checkout, including empty/count answers.
        root: NativePath,
        /// CLI invocation, not a scientific run.
        invocation_id: String,
        /// Validated effective query.
        request: Box<Request>,
    },
    /// Declare shared source/provider context before its rows.
    Source(Box<SourceContext>),
    /// One complete match row.
    Result(Box<SearchResult>),
    /// An original source/provider refusal.
    Diagnostic {
        /// The failing boundary; original provider causes retain their own codes.
        phase: DiagnosticPhase,
        /// Typed category.
        code: String,
        /// Original native explanation.
        message: String,
        /// Source when known.
        source: Option<String>,
        /// Structured native query cause, retained independently of its display message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<Box<crate::envelope::ErrorRecord>>,
    },
    /// Actual final coverage and settlement.
    Terminal(Summary),
}
impl Event {
    /// Output-direction schema for actual serialized stream records.
    pub fn output_schema() -> schemars::Schema {
        SchemaSettings::draft2020_12()
            .for_serialize()
            .into_generator()
            .into_root_schema_for::<Self>()
    }
}
/// A bounded collected view uses the same rows as the incremental stream.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    /// Performed source root and normalized question, retained even with no result rows.
    pub query: Option<PerformedQuery>,
    /// Shared source declarations.
    pub sources: Vec<SourceContext>,
    /// Requested complete rows only.
    pub results: Vec<SearchResult>,
    /// Original source/provider refusals.
    pub diagnostics: Vec<QueryDiagnostic>,
    /// Original typed remote refusals, including cancellation acknowledgement data.
    pub native_failures: Vec<crate::semantic::Failure>,
    /// Actual terminal work facts.
    pub summary: Summary,
}
/// One performed question; no answer or process inventory is duplicated here.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PerformedQuery {
    /// Original selected source/control checkout.
    pub root: NativePath,
    /// Effective typed request after source configuration and explicit overrides.
    pub request: Request,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_defaults_and_unknown_semantics_are_explicit() {
        let request: Request = serde_json::from_str("{}").unwrap();
        request.validate().unwrap();
        assert_eq!(request.answer.limit, Some(20));
        assert_eq!(request.answer.order, ResultOrder::Arrival);
        assert_eq!(Truth::Unknown.negate(), Truth::Unknown);
        assert_eq!(Truth::Unknown.and(Truth::True), Truth::Unknown);
        assert_eq!(Truth::Unknown.or(Truth::True), Truth::True);
    }
    #[test]
    fn arrival_cursor_and_empty_groups_refuse_before_acquisition() {
        let mut request = Request::default();
        request.answer.cursor = Some("old".into());
        assert!(request.validate().is_err());
        request.answer.order = ResultOrder::Stable;
        request.predicate = Predicate::All {
            within: MatchScope::File,
            clauses: Vec::new(),
        };
        assert!(request.validate().is_err());
    }
    #[test]
    fn related_contract_requires_independent_domain_and_nonempty_finite_intent() {
        let seed = ResultReference::new(Locator::Source {
            path: PathBuf::from("src/lib.rs").into(),
            source: SourceKind::Working,
            range: None,
            guard: None,
        });
        let base = serde_json::json!({"predicate": {
            "kind":"related", "seeds":[seed], "relations":["parameter_type"],
            "traversal":{"packages":["native"]}
        }});
        let request: Request = serde_json::from_value(base.clone()).unwrap();
        request.validate().unwrap();
        assert!(matches!(
            request.predicate,
            Predicate::Related {
                seed_match: super::super::graph::SeedMatch::Any,
                max_edges: 1000,
                ..
            }
        ));
        for (field, value) in [
            ("seeds", serde_json::json!([])),
            ("relations", serde_json::json!([])),
            ("relations", serde_json::json!(["uses", "uses"])),
            ("depth", serde_json::json!(0)),
            ("max_nodes", serde_json::json!(0)),
            ("max_edges", serde_json::json!(0)),
            ("traversal", serde_json::json!({"kind":{"kind":"index"}})),
        ] {
            let mut invalid = base.clone();
            invalid["predicate"][field] = value;
            let request: Request = serde_json::from_value(invalid).unwrap();
            assert!(request.validate().is_err(), "accepted invalid {field}");
        }
        let mut legacy = base;
        let predicate = legacy["predicate"].as_object_mut().unwrap();
        let seed = predicate.remove("seeds").unwrap()[0].clone();
        predicate.insert("seed".into(), seed);
        assert!(serde_json::from_value::<Request>(legacy).is_err());
    }
    #[test]
    fn references_preserve_raw_paths_and_reject_bad_versions() {
        use std::os::unix::ffi::OsStringExt as _;
        let path: PathBuf = std::ffi::OsString::from_vec(vec![b'a', 255]).into();
        let reference = ResultReference::new(Locator::Source {
            path: path.clone().into(),
            source: SourceKind::Working,
            range: None,
            guard: None,
        });
        let decoded = ResultReference::decode(&reference.encode().unwrap()).unwrap();
        let Locator::Source {
            path: decoded_path, ..
        } = decoded.locator
        else {
            panic!("wrong source kind");
        };
        assert_eq!(decoded_path.to_path_buf(), path);
        let bad = ResultReference {
            version: VERSION + 1,
            ..reference
        };
        assert!(ResultReference::decode(&bad.encode().unwrap()).is_err());
    }
    #[test]
    fn request_native_paths_round_trip_without_lossy_unicode_or_hashing_failure() {
        use std::os::unix::ffi::OsStringExt as _;
        let path: PathBuf = std::ffi::OsString::from_vec(vec![b'f', 255, b'.', b'r', b's']).into();
        let mut request = Request::default();
        request.sources.paths.push(path.clone());
        let wire = serde_json::to_value(&request).unwrap();
        assert_eq!(
            wire["sources"]["paths"][0]["unix"],
            serde_json::json!([102, 255, 46, 114, 115])
        );
        let decoded: Request = serde_json::from_value(wire).unwrap();
        assert_eq!(decoded.sources.paths, [path]);
        assert!(
            serde_json::from_value::<NativePath>(
                serde_json::json!({"unix":[102,255],"lossy":"f�"})
            )
            .is_err()
        );
    }
    #[test]
    fn guarded_references_decode_from_values_and_portable_encoded_addresses() {
        let reference = ResultReference::new(Locator::Source {
            path: PathBuf::from("source.rs").into(),
            source: SourceKind::Working,
            range: Some(MatchRange {
                byte_start: 1,
                byte_end: 3,
                line_start: 1,
                line_end: 1,
            }),
            guard: Some(SourceGuard {
                root: Some(PathBuf::from("/original-root").into()),
                sha256: None,
                length: Some(5),
                modified_nanos: Some(1_798_756_800_000_000_000),
                observed: Some(ObservedGuard {
                    range: MatchRange {
                        byte_start: 1,
                        byte_end: 3,
                        line_start: 1,
                        line_end: 1,
                    },
                    sha256: "original-observed-span".into(),
                }),
                git_blob: None,
            }),
        });
        let value = serde_json::to_value(&reference).unwrap();
        let from_value: ResultReference = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(from_value).unwrap(), value);
        assert_eq!(
            serde_json::to_value(ResultReference::decode(&reference.encode().unwrap()).unwrap())
                .unwrap(),
            value
        );
        assert_eq!(
            serde_json::to_value(
                ResultReference::decode(&serde_json::to_string(&reference).unwrap()).unwrap()
            )
            .unwrap(),
            value
        );
        for field in ["version", "range", "root"] {
            let mut invalid = reference.clone();
            match field {
                "version" => invalid.version = VERSION + 1,
                "range" => {
                    if let Locator::Source {
                        range: Some(range), ..
                    } = &mut invalid.locator
                    {
                        range.byte_end = 0;
                    }
                }
                "root" => {
                    if let Locator::Source {
                        guard: Some(guard), ..
                    } = &mut invalid.locator
                    {
                        guard.root = Some(PathBuf::from("relative").into());
                    }
                }
                _ => panic!("unexpected invalid reference field"),
            }
            for form in [
                invalid.encode().unwrap(),
                serde_json::to_string(&invalid).unwrap(),
            ] {
                assert!(ResultReference::decode(&form).is_err(), "{field}: {form}");
            }
            let request = Request {
                predicate: Predicate::Related {
                    seeds: vec![invalid],
                    relations: vec![Relationship::References],
                    traversal: SourceSelector::default(),
                    seed_match: super::super::graph::SeedMatch::Any,
                    direction: Direction::Outgoing,
                    depth: 1,
                    max_nodes: 10,
                    max_edges: 100,
                },
                ..Default::default()
            };
            assert!(
                request.validate().is_err(),
                "inline request bypassed {field} validation"
            );
        }
    }
    #[test]
    fn directional_contracts_are_generated_from_actual_types() {
        assert!(
            Request::input_schema()
                .to_value()
                .get("properties")
                .is_some()
        );
        let event = Event::Terminal(Summary::default());
        let schema = Event::output_schema();
        let validator = jsonschema::validator_for(schema.as_value()).unwrap();
        assert!(validator.is_valid(&serde_json::to_value(event).unwrap()));
    }
    #[test]
    fn indirect_event_payloads_preserve_wire_shape_and_original_addresses() {
        use std::os::unix::ffi::OsStringExt as _;
        let path: PathBuf = std::ffi::OsString::from_vec(vec![b'f', 255]).into();
        let source = SourceContext {
            id: "source".into(),
            path: path.clone().into(),
            origin: Some(SourceKind::Working),
            provider: "original-provider".into(),
            generation: Some("original-generation".into()),
            searched_encoding: None,
            native: None,
        };
        let reference = ResultReference::new(Locator::Source {
            path: path.into(),
            source: SourceKind::Working,
            range: None,
            guard: Some(SourceGuard {
                root: Some(PathBuf::from("/original-root").into()),
                sha256: Some("original-digest".into()),
                ..Default::default()
            }),
        });
        let row = SearchResult {
            source_relationship: None,
            source: source.id.clone(),
            kind: "text".into(),
            name: None,
            range: None,
            context: "original witness".into(),
            bytes: Some(vec![255]),
            reference: Some(reference.clone()),
            candidate_reference: Some(reference),
            refinement: None,
            entity: None,
            captures: BTreeMap::new(),
        };
        let request = Request::default();
        let root: NativePath = PathBuf::from("/original-root").into();
        let begin_wire = serde_json::json!({
            "event": "begin",
            "root": root,
            "invocation_id": "original-invocation",
            "request": request,
        });
        let mut source_wire = serde_json::to_value(&source).unwrap();
        source_wire["event"] = serde_json::json!("source");
        let mut result_wire = serde_json::to_value(&row).unwrap();
        result_wire["event"] = serde_json::json!("result");
        let schema = Event::output_schema();
        let validator = jsonschema::validator_for(schema.as_value()).unwrap();
        for (event, expected) in [
            (
                Event::Begin {
                    root,
                    invocation_id: "original-invocation".into(),
                    request: Box::new(request),
                },
                begin_wire,
            ),
            (Event::Source(Box::new(source)), source_wire),
            (Event::Result(Box::new(row)), result_wire),
        ] {
            let actual = serde_json::to_value(&event).unwrap();
            assert_eq!(actual, expected);
            assert!(validator.is_valid(&actual), "{actual}");
            let decoded: Event = serde_json::from_value(actual.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), actual);
        }
    }
}
