// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native canonical storage meaning. Opaque lookup keys never become scientific identities.

/// Canonical row lifetime, independent of backup membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalRetention {
    /// Original application meaning and its retained closure.
    Durable,
    /// Rebuildable discovery or readiness data.
    Derived,
    /// Live process, lease or read authority.
    LiveAuthority,
    /// Immutable acknowledgement and retirement evidence.
    Receipt,
}
/// Application backup inventory membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalBackup {
    /// Enumerated original application owner, including an empty table.
    Required,
    /// Native restore/recovery provisions or rebuilds this table.
    Recovered,
}
/// Native storage policy declared at a relation owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalRelation {
    /// Canonical row lifetime.
    pub retention: CanonicalRetention,
    /// Application backup inventory membership.
    pub backup: CanonicalBackup,
    /// Bound native adjacency caches for reached vertices.
    pub inline_vertex: bool,
    /// Native relation endpoints must exist when inserted.
    pub enforced: bool,
    /// Native reverse references require records in the same generation.
    pub generation_references: bool,
}
impl CanonicalRelation {
    /// Original durable application rows without graph policy.
    pub const DURABLE: Self = Self { retention: CanonicalRetention::Durable, backup: CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false };
}
macro_rules! key_kinds {
    ($($kind:ident => $spelling:literal),* $(,)?) => {
        /// Nominal canonical lookup domains over opaque UTF-8, separate from SemanticId.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum CanonicalKeyKind { $(#[doc = concat!("Canonical ", $spelling, " lookup domain.")] $kind,)* }
        impl CanonicalKeyKind {
            /// Every declared nominal canonical key kind.
            pub const ALL: &'static [Self] = &[$(Self::$kind),*];
            /// Stable field facet spelling.
            pub const fn as_str(self) -> &'static str { match self { $(Self::$kind => $spelling,)* } }
            /// Decode only declared key kinds.
            pub fn parse(value: &str) -> Option<Self> { Self::ALL.iter().copied().find(|kind| kind.as_str() == value) }
        }
    };
}
key_kinds! {
    Problem => "problem",
    Revision => "revision",
    Run => "run",
    Attempt => "attempt",
    Study => "study",
    StudyPoint => "study_point",
    Operation => "operation",
    Artifact => "artifact",
    FileInstance => "file_instance",
    Project => "project",
    ProjectBranch => "project_branch",
    ProjectDraft => "project_draft",
    DraftVersion => "draft_version",
    ProjectSavepoint => "project_savepoint",
    ProjectBookmark => "project_bookmark",
    ProjectAncestry => "project_ancestry",
    DraftDiagnostic => "draft_diagnostic",
    ProjectContext => "project_context",
    ContextAncestry => "context_ancestry",
    ResearchReference => "research_reference",
    ResearchDataset => "research_dataset",
    ResearchSelection => "research_selection",
    ResearchPublication => "research_publication",
    ProjectImport => "project_import",
    Product => "product",
    BodyContent => "body_content",
    SelectionReceipt => "selection_receipt",
    ProductAssociation => "product_association",
    ProductDescriptor => "product_descriptor",
    Analysis => "analysis",
    AnalysisNode => "analysis_node",
    MaintenanceOperation => "maintenance_operation",
    MaintenanceActivation => "maintenance_activation",
    InlinePayload => "inline_payload",
    FileOwner => "file_owner",
    FileBorrower => "file_borrower",
    ManifestTicket => "manifest_ticket",
}

/// One exact key-construction association. Prefixes mint opaque spellings only;
/// consumers must use the stored relationship rather than recognize the prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalKeyConstructor {
    /// Generated constructor method.
    pub method: &'static str,
    /// Exact framing prefix.
    pub prefix: &'static str,
    /// Nominal source key.
    pub input: CanonicalKeyKind,
}
impl CanonicalKeyKind {
    /// Named key construction associations owned by this key domain.
    pub const fn constructors(self) -> &'static [CanonicalKeyConstructor] {
        match self {
            Self::Run => &[CanonicalKeyConstructor { method: "study_admission", prefix: "study-admission:", input: Self::StudyPoint }],
            Self::Problem => &[CanonicalKeyConstructor { method: "study_variant", prefix: "study-variant:", input: Self::StudyPoint }],
            Self::Operation => &[CanonicalKeyConstructor { method: "study_variant_source", prefix: "study-variant-source:", input: Self::StudyPoint }],
            _ => &[],
        }
    }
}
