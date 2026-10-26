// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owner-native development concepts above immutable source-version provenance.
use super::declarations::{column, enumeration};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;

fn relation(b: &mut RegistryBuilder, namespace: N, name: &'static str, class: S, keys: &[&'static str], columns: Vec<T>, doc: &'static str) {
    relation_version(b, namespace, name, 1, class, keys, columns, doc);
}
fn relation_version(b: &mut RegistryBuilder, namespace: N, name: &'static str, version: u32, class: S, keys: &[&'static str], columns: Vec<T>, doc: &'static str) {
    let columns = columns.into_iter().map(T::canonical_default_readonly).collect();
    b.declare_relation(super::declarations::canonical_declaration(namespace, name, version, class, keys, columns, doc)
        .with_canonical(crate::model::CanonicalRelation {
            retention: crate::model::CanonicalRetention::Derived,
            backup: crate::model::CanonicalBackup::Recovered,
            inline_vertex: false,
            enforced: true,
            generation_references: true,
        }));
}

/// Derived interpretation, independent of scientific research and source authority.
pub const INTERPRETATION: &str = "pse-development-knowledge-v2";
/// Evidence origin, independent of native lifecycle and publication state.
pub const BASES: &[&str] = &[
    "declared",
    "language-resolved",
    "text",
    "historical",
    "native",
    "agent-interpretation",
    "user-question",
];
/// Source observation state, not subject lifecycle.
pub const STATUSES: &[&str] = &[
    "current",
    "historical",
    "proposed",
    "missing",
    "stale",
    "unknown",
];
/// Role-specific immutable contributors, including policy interpretation inputs.
pub const SOURCE_ROLES: &[&str] = &[
    "definition",
    "catalogue_registration",
    "producer_declaration",
    "native_scope",
    "metadata",
    "owner_evidence",
    "question_input",
    "interpretation_input",
    "interpretation_policy",
];
/// Native document metadata vocabulary.
pub const DOCUMENT_ROLES: &[&str] = &[
    "contract",
    "instruction",
    "decision",
    "plan",
    "review",
    "reference",
    "evidence",
    "index",
];
/// Applicable fields for one closed subject kind; shared by admission and DDL.
#[derive(Debug)]
pub struct SubjectPolicy {
    /// Owner-native kind.
    pub kind: &'static str,
    /// Admissible document roles; empty means absent.
    pub roles: &'static [&'static str],
    /// Native state and its basis must be present.
    pub requires_state: bool,
    /// Publication/state metadata has an actual native owner for this kind.
    pub permits_state: bool,
    /// An original narrow producer version is required.
    pub requires_version: bool,
    /// Exact evidence identity is required for evidence references only.
    pub evidence: bool,
    /// Required owner contributor, when applicable.
    pub contributor: Option<&'static str>,
    /// Restricted origins; empty permits the declared origin vocabulary.
    pub bases: &'static [&'static str],
}
/// Finite actual concept families; no generic tag-defined entity model.
pub const SUBJECTS: &[SubjectPolicy] = &[
    SubjectPolicy {
        kind: "document",
        roles: &[
            "contract",
            "instruction",
            "plan",
            "review",
            "reference",
            "evidence",
            "index",
        ],
        requires_state: false,
        permits_state: true,
        requires_version: false,
        evidence: false,
        contributor: Some("metadata"),
        bases: &[],
    },
    SubjectPolicy {
        kind: "contract",
        roles: &["contract"],
        requires_state: false,
        permits_state: true,
        requires_version: false,
        evidence: false,
        contributor: Some("metadata"),
        bases: &[],
    },
    SubjectPolicy {
        kind: "decision",
        roles: &["decision"],
        requires_state: true,
        permits_state: true,
        requires_version: false,
        evidence: false,
        contributor: Some("metadata"),
        bases: &[],
    },
    SubjectPolicy {
        kind: "plan_package",
        roles: &["plan"],
        requires_state: true,
        permits_state: true,
        requires_version: false,
        evidence: false,
        contributor: Some("native_scope"),
        bases: &[],
    },
    SubjectPolicy {
        kind: "declaration",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: false,
        evidence: false,
        contributor: None,
        bases: &[],
    },
    SubjectPolicy {
        kind: "generator",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: false,
        evidence: false,
        contributor: None,
        bases: &[],
    },
    SubjectPolicy {
        kind: "generated_artifact",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: false,
        evidence: false,
        contributor: None,
        bases: &[],
    },
    SubjectPolicy {
        kind: "experiment",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: true,
        evidence: false,
        contributor: Some("owner_evidence"),
        bases: &["native"],
    },
    SubjectPolicy {
        kind: "diagnostic",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: true,
        evidence: false,
        contributor: Some("owner_evidence"),
        bases: &["native"],
    },
    SubjectPolicy {
        kind: "evidence_reference",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: false,
        evidence: true,
        contributor: Some("owner_evidence"),
        bases: &["native", "historical"],
    },
    SubjectPolicy {
        kind: "research_question",
        roles: &[],
        requires_state: false,
        permits_state: false,
        requires_version: false,
        evidence: false,
        contributor: None,
        bases: &["user-question", "agent-interpretation"],
    },
];
/// An explicitly supported owner-native relationship signature.
#[derive(Debug)]
pub struct RelationPolicy {
    /// Closed relation meaning.
    pub kind: &'static str,
    /// Applicable source concepts.
    pub from: &'static [&'static str],
    /// Applicable target concepts.
    pub to: &'static [&'static str],
    /// Supersession cannot silently change concept family.
    pub same_kind: bool,
}
/// Relation signatures, never inferred from prose proximity or shared topics.
pub const RELATIONS: &[RelationPolicy] = &[
    RelationPolicy {
        kind: "owns",
        from: &["document", "contract", "decision"],
        to: &[
            "document",
            "contract",
            "decision",
            "plan_package",
            "declaration",
            "generator",
            "generated_artifact",
        ],
        same_kind: false,
    },
    RelationPolicy {
        kind: "governs",
        from: &["contract", "decision"],
        to: &[
            "document",
            "contract",
            "decision",
            "plan_package",
            "declaration",
            "generator",
            "generated_artifact",
            "experiment",
            "diagnostic",
        ],
        same_kind: false,
    },
    RelationPolicy {
        kind: "contains",
        from: &["document"],
        to: &["contract", "decision", "plan_package"],
        same_kind: false,
    },
    RelationPolicy {
        kind: "depends_on",
        from: &["plan_package"],
        to: &["plan_package"],
        same_kind: false,
    },
    RelationPolicy {
        kind: "status_owned_by",
        from: &["plan_package"],
        to: &["plan_package"],
        same_kind: false,
    },
    RelationPolicy {
        kind: "superseded_by",
        from: &["decision", "plan_package", "contract"],
        to: &["decision", "plan_package", "contract"],
        same_kind: true,
    },
    RelationPolicy {
        kind: "supports",
        from: &["evidence_reference", "experiment", "diagnostic"],
        to: &[
            "document",
            "contract",
            "decision",
            "plan_package",
            "declaration",
            "research_question",
        ],
        same_kind: false,
    },
    RelationPolicy {
        kind: "answers",
        from: &["evidence_reference", "experiment", "diagnostic"],
        to: &["research_question"],
        same_kind: false,
    },
    RelationPolicy {
        kind: "diagnoses",
        from: &["diagnostic"],
        to: &["experiment"],
        same_kind: false,
    },
    RelationPolicy {
        kind: "references",
        from: &[
            "document",
            "contract",
            "decision",
            "plan_package",
            "research_question",
        ],
        to: &["evidence_reference"],
        same_kind: false,
    },
];
/// Knowledge-only graph enforcement; scientific lowering is unaffected.
pub const ENFORCED_RELATIONS: &[&str] = &["knowledge_subject_edges"];
/// Complete normalized membership and matching generation counters, in insertion order.
pub const FAMILIES: &[(&str, &str)] = &[
    ("knowledge_sources", "source_count"),
    ("knowledge_subjects", "subject_count"),
    ("knowledge_subject_sources", "contributor_count"),
    ("knowledge_subject_topics", "topic_count"),
    ("knowledge_excerpts", "excerpt_count"),
    ("knowledge_subject_edges", "edge_count"),
    ("knowledge_productions", "production_count"),
];
/// Full-text is optional within a structurally selected subject population.
#[derive(Debug)]
pub struct FullTextIndex {
    /// Native index name.
    pub name: &'static str,
    /// Declared relation table.
    pub table: &'static str,
    /// Declared text field.
    pub field: &'static str,
    /// Shared analyzer name.
    pub analyzer: &'static str,
}
/// Exact optional text index.
pub const FULL_TEXT: FullTextIndex = FullTextIndex {
    name: "knowledge_excerpt_text",
    table: "knowledge_excerpts",
    field: "text",
    analyzer: "knowledge_text_v2",
};
/// Selected tokenization.
pub const TOKENIZERS: &[&str] = &["blank", "class", "camel", "punct"];
/// Selected normalization.
pub const FILTERS: &[&str] = &["lowercase"];
/// Native exact access paths.
pub const INDEXES: &[super::substrate::NativeIndex] = &[
    super::substrate::NativeIndex {
        name: "knowledge_source_member",
        table: "knowledge_sources",
        fields: &["generation", "source_id"],
        unique: true,
    },
    super::substrate::NativeIndex {
        name: "knowledge_subject_member",
        table: "knowledge_subjects",
        fields: &["generation", "subject_id"],
        unique: true,
    },
    super::substrate::NativeIndex {
        name: "knowledge_subject_selection",
        table: "knowledge_subjects",
        fields: &["generation", "kind", "owner", "native_id"],
        unique: false,
    },
    super::substrate::NativeIndex {
        name: "knowledge_topic_selection",
        table: "knowledge_subject_topics",
        fields: &["generation", "topic", "subject_key"],
        unique: false,
    },
    super::substrate::NativeIndex {
        name: "knowledge_contributor_member",
        table: "knowledge_subject_sources",
        fields: &["generation", "subject_key", "source_key", "role"],
        unique: true,
    },
    super::substrate::NativeIndex {
        name: "knowledge_excerpt_member",
        table: "knowledge_excerpts",
        fields: &["generation", "excerpt_id"],
        unique: true,
    },
    super::substrate::NativeIndex {
        name: "knowledge_edge_member",
        table: "knowledge_subject_edges",
        fields: &["generation", "edge_id"],
        unique: true,
    },
    super::substrate::NativeIndex {
        name: "knowledge_production_member",
        table: "knowledge_productions",
        fields: &["generation", "production_id"],
        unique: true,
    },
    super::substrate::NativeIndex {
        name: "knowledge_production_selection",
        table: "knowledge_productions",
        fields: &[
            "generation",
            "declaration_key",
            "generator_key",
            "artifact_key",
        ],
        unique: false,
    },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    enumeration(b, "KnowledgeBasis", BASES.iter().copied());
    enumeration(b, "KnowledgeSourceStatus", STATUSES.iter().copied());
    enumeration(b, "KnowledgeGenerationState", ["staged", "active"]);
    enumeration(
        b,
        "KnowledgeSubjectKind",
        SUBJECTS.iter().map(|policy| policy.kind),
    );
    enumeration(b, "KnowledgeDocumentRole", DOCUMENT_ROLES.iter().copied());
    enumeration(b, "KnowledgeSourceRole", SOURCE_ROLES.iter().copied());
    enumeration(
        b,
        "KnowledgeRelationKind",
        RELATIONS.iter().map(|policy| policy.kind),
    );
    enumeration(b, "KnowledgeDirection", ["input", "output"]);
    let text = || T::native(D::Utf8);
    let link = |name, target| column(name, text()).with_fk(target, "key");
    relation(
        b,
        N::Runtime,
        "knowledge_state",
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("active_generation", text()).canonical_mutable().optional(),
        ],
        "Independent corpus activation pointer; no evidence pointer is changed.",
    );
    let mut generation = vec![column("key", text()), column("content_digest", text())];
    for (_, name) in FAMILIES {
        generation.push(column(name, T::native(D::UInt64)));
    }
    generation.push(column("state", T::enumeration("KnowledgeGenerationState")).canonical_mutable().with_canonical_assertion("($before=NONE AND $value='staged') OR ($before='staged' AND $value IN ['staged','active']) OR ($before='active' AND $value='active')"));
    relation_version(
        b,
        N::Runtime,
        "knowledge_generations",
        2,
        S::Derived,
        &["key"],
        generation,
        "Complete exact v2 corpus membership across every normalized family. Replay reconciles active and staged generations alike.",
    );
    relation(
        b,
        N::Runtime,
        "knowledge_sources",
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("generation", text()),
            column("source_id", text()),
            column("kind", text()),
            column("locator", text()),
            column("version_digest", text()),
            column("basis", T::enumeration("KnowledgeBasis")),
            column("title", text()),
            column("status", T::enumeration("KnowledgeSourceStatus")),
        ],
        "Immutable exact-version provenance carrier, not a substitute for owner-native concepts.",
    );
    let mut subjects = vec![
        column("key", text()),
        column("generation", text()),
        column("subject_id", text()),
        column("kind", T::enumeration("KnowledgeSubjectKind")),
        column("owner", text()),
        column("native_id", text()),
        column("label", text()),
        column("basis", T::enumeration("KnowledgeBasis")),
        column("native_state", text()).optional(),
        column("state_basis", text()).optional(),
        column("publication_scope", text()).optional(),
        column("document_role", T::enumeration("KnowledgeDocumentRole")).optional(),
        column("native_version", text()).optional(),
    ];
    for name in [
        "evidence_authority",
        "evidence_codec",
        "evidence_source",
        "evidence_version",
        "evidence_member",
    ] {
        subjects.push(column(name, text()).optional());
    }
    relation(
        b,
        N::Runtime,
        "knowledge_subjects",
        S::Derived,
        &["key"],
        subjects,
        "Closed owner-native concepts. Native lifecycle, document role, publication scope and evidence origin retain their independent meanings.",
    );
    relation(
        b,
        N::Runtime,
        "knowledge_subject_sources",
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("generation", text()),
            link("subject_key", "runtime.knowledge_subjects"),
            link("source_key", "runtime.knowledge_sources"),
            column("role", T::enumeration("KnowledgeSourceRole")),
        ],
        "Role-specific source contributors, including exact metadata interpretation policies.",
    );
    relation(
        b,
        N::Runtime,
        "knowledge_subject_topics",
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("generation", text()),
            link("subject_key", "runtime.knowledge_subjects"),
            link("source_key", "runtime.knowledge_sources"),
            column("topic", text()),
        ],
        "Topics supplied by native metadata only, with exact original metadata source attribution.",
    );
    relation_version(
        b,
        N::Runtime,
        "knowledge_excerpts",
        2,
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("generation", text()),
            column("excerpt_id", text()),
            link("source_key", "runtime.knowledge_sources"),
            link("subject_key", "runtime.knowledge_subjects").optional(),
            column("text", text()),
            column("locator", text()),
            column("basis", T::enumeration("KnowledgeBasis")),
        ],
        "Selected excerpt with explicit optional subject attribution; sharing a source file never makes other sections match.",
    );
    relation(
        b,
        N::Runtime,
        "knowledge_subject_edges",
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("generation", text()),
            column("edge_id", text()),
            link("source_key", "runtime.knowledge_subjects").with_graph_endpoint(true),
            link("target_key", "runtime.knowledge_subjects").with_graph_endpoint(false),
            column("relation", T::enumeration("KnowledgeRelationKind")),
            column("basis", T::enumeration("KnowledgeBasis")),
        ],
        "Enforced typed owner-native subject relations. Exact applicability is independent of prose or topical similarity.",
    );
    relation(
        b,
        N::Runtime,
        "knowledge_productions",
        S::Derived,
        &["key"],
        vec![
            column("key", text()),
            column("generation", text()),
            column("production_id", text()),
            link("declaration_key", "runtime.knowledge_subjects").with_canonical_reference_kind("declaration"),
            link("generator_key", "runtime.knowledge_subjects").with_canonical_reference_kind("generator"),
            link("artifact_key", "runtime.knowledge_subjects").with_canonical_reference_kind("generated_artifact"),
            column("direction", T::enumeration("KnowledgeDirection")).optional(),
            column("basis", T::enumeration("KnowledgeBasis")),
        ],
        "Exact declaration-generator-specific-output assertion. Pairwise generator adjacency cannot invent output cross-products.",
    );
}
