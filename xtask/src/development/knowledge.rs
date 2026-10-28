// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disposable owner-native concepts above exact immutable source provenance.
use crate::development::catalog::knowledge as declaration;
use crate::development::evidence::{
    EvidenceAccess, EvidenceError, EvidenceStore, within_allowance,
};
pub use crate::generated::model::enums::{
    KnowledgeDirection as Direction, KnowledgeDocumentRole as DocumentRole,
    KnowledgeRelationKind as RelationKind, KnowledgeSourceRole as SourceRole,
    KnowledgeSubjectKind as SubjectKind,
};
use crate::generated::model::{
    enums::KnowledgeGenerationState,
    runtime::{
        knowledge_excerpts, knowledge_generations, knowledge_productions, knowledge_sources,
        knowledge_subject_edges, knowledge_subject_sources, knowledge_subject_topics,
        knowledge_subjects,
    },
};
use crate::generated::store::knowledge as wire;
use pse_operations::transport::client::{Query, ResponseLimits};
use pse_operations::{
    canonical::{self, CanonicalError},
    transport::{self, original_deadline},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use surrealdb_types::{Object, Value};
fn invalid(message: impl Into<String>) -> EvidenceError {
    EvidenceError::Configuration(message.into())
}
fn digest(bytes: impl AsRef<[u8]>) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn encoded<T: Serialize>(value: &T) -> Result<Vec<u8>, EvidenceError> {
    serde_json::to_vec(value).map_err(|cause| invalid(cause.to_string()))
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn bounded(value: &str, extent: usize) -> bool {
    !value.trim().is_empty() && value.len() <= extent && !value.contains('\0')
}
fn member(generation: &str, id: impl Serialize) -> Result<String, EvidenceError> {
    Ok(digest(encoded(&(generation, id))?))
}
fn basis(value: &str) -> bool {
    declaration::BASES.contains(&value)
}

/// Immutable source-version carrier, never a substitute for a domain concept.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeSource {
    /// Exact selected carrier identity.
    pub id: String,
    /// Original carrier category.
    pub kind: String,
    /// Original attributable locator, without lifetime or file-access grant.
    pub locator: String,
    /// SHA256 of exact selected source bytes.
    pub version_digest: String,
    /// Explicit evidence origin.
    pub basis: String,
    /// Compact original description.
    pub title: String,
    /// Source observation state, independent of subject lifecycle.
    pub status: String,
}
/// Original evidence-owner identity, not a replacement evidence codec.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceIdentity {
    /// Original issuing authority.
    pub authority: String,
    /// Original codec.
    pub codec: String,
    /// Original source.
    pub source_id: String,
    /// Original version or digest.
    pub version: String,
    /// Original member.
    pub member: String,
}
/// Role-specific immutable contributor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubjectSource {
    /// Exact source in this cut.
    pub source_id: String,
    /// Actual native contribution.
    pub role: SourceRole,
}
/// Topic copied from original metadata, with exact attribution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubjectTopic {
    /// Native topic, not an invented tag.
    pub topic: String,
    /// Subject metadata contributor.
    pub source_id: String,
}
/// Closed owner-native subject; publication and native lifecycle remain separate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeSubject {
    /// Stable adapter-qualified subject identity.
    pub id: String,
    /// Closed subject family.
    pub kind: SubjectKind,
    /// Native semantic owner.
    pub owner: String,
    /// Actual owner-native identity.
    pub native_id: String,
    /// Original compact label.
    pub label: String,
    /// Explicit evidence origin.
    pub basis: String,
    /// Native lifecycle/status only where applicable.
    pub native_state: Option<String>,
    /// Actual interpretation of native state.
    pub state_basis: Option<String>,
    /// Publication scope, independent of proposal/acceptance.
    pub publication_scope: Option<String>,
    /// Actual native document role.
    pub document_role: Option<DocumentRole>,
    /// Exact owner version when supplied.
    pub native_version: Option<String>,
    /// Original evidence identity for evidence_reference only.
    pub evidence: Option<EvidenceIdentity>,
    /// Complete explicit contributors, at most eight.
    pub sources: Vec<SubjectSource>,
    /// Native metadata topics, at most sixteen.
    pub topics: Vec<SubjectTopic>,
}
/// Selected excerpt; shared files do not imply shared subject attribution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeExcerpt {
    /// Exact excerpt identity.
    pub id: String,
    /// Exact source carrier.
    pub source_id: String,
    /// Subject only where correspondence is explicitly known.
    pub subject_id: Option<String>,
    /// Selected text, at most sixteen KiB.
    pub text: String,
    /// Original locator.
    pub locator: String,
    /// Explicit evidence origin.
    pub basis: String,
}
/// Typed native subject relation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeEdge {
    /// Original relationship identity.
    pub id: String,
    /// Exact source subject.
    pub from: String,
    /// Exact target subject.
    pub to: String,
    /// Closed relation meaning.
    pub relation: RelationKind,
    /// Actual relationship origin.
    pub basis: String,
}
/// Actual declaration-generator-specific-output correspondence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeProduction {
    /// Original producer record identity.
    pub id: String,
    /// Actual declaration subject.
    pub declaration_id: String,
    /// Actual generator subject.
    pub generator_id: String,
    /// Exact emitted artifact.
    pub artifact_id: String,
    /// Direction only where declared.
    pub direction: Option<Direction>,
    /// Actual producer basis.
    pub basis: String,
}
/// Complete explicitly selected corpus; no writable domain authority is introduced.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeCorpus {
    /// Codec version, exactly two.
    pub version: u32,
    /// CAS precondition; null requires an empty pointer.
    pub expected_active: Option<String>,
    /// Complete exact source membership.
    pub sources: Vec<KnowledgeSource>,
    /// Complete closed subject membership.
    pub subjects: Vec<KnowledgeSubject>,
    /// Complete selected excerpts.
    pub excerpts: Vec<KnowledgeExcerpt>,
    /// Complete typed relationships.
    pub edges: Vec<KnowledgeEdge>,
    /// Complete exact production triples.
    pub productions: Vec<KnowledgeProduction>,
}
fn subject_valid(
    subject: &KnowledgeSubject,
    sources: &BTreeSet<&String>,
) -> Result<(), EvidenceError> {
    let policy = declaration::SUBJECTS
        .iter()
        .find(|policy| policy.kind == subject.kind.as_str())
        .ok_or_else(|| invalid("unknown subject policy"))?;
    if [
        &subject.id,
        &subject.owner,
        &subject.native_id,
        &subject.label,
    ]
    .iter()
    .any(|value| !bounded(value, 4096))
        || !basis(&subject.basis)
        || (!policy.bases.is_empty() && !policy.bases.contains(&subject.basis.as_str()))
        || subject.sources.is_empty()
        || subject.sources.len() > 8
        || subject.topics.len() > 16
        || subject.native_state.is_some() != subject.state_basis.is_some()
        || (policy.requires_state && subject.native_state.is_none())
        || (!policy.permits_state
            && (subject.native_state.is_some() || subject.publication_scope.is_some()))
        || (policy.requires_version && subject.native_version.is_none())
        || subject.evidence.is_some() != policy.evidence
        || (if policy.roles.is_empty() {
            subject.document_role.is_some()
        } else {
            !subject
                .document_role
                .is_some_and(|role| policy.roles.contains(&role.as_str()))
        })
        || subject
            .native_state
            .iter()
            .chain(subject.state_basis.iter())
            .chain(subject.publication_scope.iter())
            .chain(subject.native_version.iter())
            .any(|value| !bounded(value, 4096))
    {
        return Err(invalid(
            "inapplicable subject role, lifecycle, version, evidence identity or origin",
        ));
    }
    if let Some(evidence) = &subject.evidence
        && [
            &evidence.authority,
            &evidence.codec,
            &evidence.source_id,
            &evidence.version,
            &evidence.member,
        ]
        .iter()
        .any(|value| !bounded(value, 4096))
    {
        return Err(invalid("original evidence identity required"));
    }
    let mut links = BTreeSet::new();
    for link in &subject.sources {
        if !sources.contains(&link.source_id)
            || !links.insert((&link.source_id, link.role.as_str()))
        {
            return Err(invalid("duplicate or absent contributor"));
        }
    }
    if policy.contributor.is_some_and(|role| {
        !subject
            .sources
            .iter()
            .any(|link| link.role.as_str() == role)
    }) {
        return Err(invalid("applicable native contributor required"));
    }
    if subject.kind.as_str() == "research_question" {
        let role = if subject.basis == "user-question" {
            "question_input"
        } else {
            "interpretation_input"
        };
        if !subject
            .sources
            .iter()
            .any(|link| link.role.as_str() == role)
        {
            return Err(invalid("explicit question origin contributor required"));
        }
    }
    let mut topics = BTreeSet::new();
    for topic in &subject.topics {
        if !bounded(&topic.topic, 4096)
            || !topics.insert((&topic.topic, &topic.source_id))
            || !["document", "contract", "decision", "plan_package"]
                .contains(&subject.kind.as_str())
            || !subject
                .sources
                .iter()
                .any(|link| link.source_id == topic.source_id && link.role.as_str() == "metadata")
        {
            return Err(invalid(
                "topic must come from an explicit native metadata contributor",
            ));
        }
    }
    Ok(())
}
impl KnowledgeCorpus {
    /// Validate closed domain meaning and exact membership before effects.
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.version != 2
            || self.sources.is_empty()
            || self.sources.len() > 256
            || self.subjects.is_empty()
            || self.subjects.len() > 256
            || self.excerpts.len() > 1024
            || self.edges.len() > 2048
            || self.productions.len() > 1024
            || self
                .expected_active
                .as_ref()
                .is_some_and(|value| !hash(value))
        {
            return Err(invalid(
                "bounded v2 corpus and exact pointer precondition required",
            ));
        }
        let mut sources = BTreeSet::new();
        for source in &self.sources {
            if !sources.insert(&source.id)
                || [&source.id, &source.kind, &source.locator, &source.title]
                    .iter()
                    .any(|value| !bounded(value, 4096))
                || !hash(&source.version_digest)
                || !basis(&source.basis)
                || !declaration::STATUSES.contains(&source.status.as_str())
            {
                return Err(invalid("invalid exact source identity or observation"));
            }
        }
        let mut subjects = BTreeMap::new();
        let mut native = BTreeSet::new();
        for subject in &self.subjects {
            subject_valid(subject, &sources)?;
            if subjects.insert(&subject.id, subject).is_some()
                || !native.insert((subject.kind.as_str(), &subject.owner, &subject.native_id))
            {
                return Err(invalid("duplicate native subject identity"));
            }
        }
        let mut excerpts = BTreeSet::new();
        for excerpt in &self.excerpts {
            if !excerpts.insert(&excerpt.id)
                || !sources.contains(&excerpt.source_id)
                || !bounded(&excerpt.id, 4096)
                || !bounded(&excerpt.locator, 4096)
                || !bounded(&excerpt.text, 16384)
                || !basis(&excerpt.basis)
                || excerpt.subject_id.as_ref().is_some_and(|id| {
                    !subjects.get(id).is_some_and(|subject| {
                        subject
                            .sources
                            .iter()
                            .any(|link| link.source_id == excerpt.source_id)
                    })
                })
            {
                return Err(invalid(
                    "excerpt subject/source correspondence is undeclared",
                ));
            }
        }
        let mut edges = BTreeSet::new();
        for edge in &self.edges {
            let from = subjects
                .get(&edge.from)
                .ok_or_else(|| invalid("edge source absent"))?;
            let to = subjects
                .get(&edge.to)
                .ok_or_else(|| invalid("edge target absent"))?;
            let policy = declaration::RELATIONS
                .iter()
                .find(|policy| policy.kind == edge.relation.as_str())
                .ok_or_else(|| invalid("relationship policy absent"))?;
            if !edges.insert(&edge.id)
                || !bounded(&edge.id, 4096)
                || !basis(&edge.basis)
                || edge.from == edge.to
                || !policy.from.contains(&from.kind.as_str())
                || !policy.to.contains(&to.kind.as_str())
                || (policy.same_kind && from.kind != to.kind)
            {
                return Err(invalid(
                    "relationship does not apply to these concept kinds",
                ));
            }
        }
        let mut productions = BTreeSet::new();
        let mut triples = BTreeSet::new();
        for production in &self.productions {
            let kind = |id: &String| subjects.get(id).map(|subject| subject.kind.as_str());
            if !productions.insert(&production.id)
                || !bounded(&production.id, 4096)
                || !basis(&production.basis)
                || kind(&production.declaration_id) != Some("declaration")
                || kind(&production.generator_id) != Some("generator")
                || kind(&production.artifact_id) != Some("generated_artifact")
                || !triples.insert((
                    &production.declaration_id,
                    &production.generator_id,
                    &production.artifact_id,
                    production.direction.map(|direction| direction.as_str()),
                ))
            {
                return Err(invalid("invalid exact production correspondence"));
            }
        }
        if encoded(self)?.len() > canonical::PAYLOAD_BYTES {
            return Err(invalid("corpus exceeds import envelope"));
        }
        Ok(())
    }
    /// Order-independent exact identity; the activation precondition is not content.
    pub fn generation(&self) -> Result<String, EvidenceError> {
        self.validate()?;
        let mut corpus = self.clone();
        corpus.expected_active = None;
        corpus.sources.sort_by(|a, b| a.id.cmp(&b.id));
        corpus.subjects.sort_by(|a, b| a.id.cmp(&b.id));
        for subject in &mut corpus.subjects {
            subject.sources.sort_by(|a, b| {
                (&a.source_id, a.role.as_str()).cmp(&(&b.source_id, b.role.as_str()))
            });
            subject
                .topics
                .sort_by(|a, b| (&a.topic, &a.source_id).cmp(&(&b.topic, &b.source_id)));
        }
        corpus.excerpts.sort_by(|a, b| a.id.cmp(&b.id));
        corpus.edges.sort_by(|a, b| a.id.cmp(&b.id));
        corpus.productions.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(digest(encoded(&corpus)?))
    }
}
/// Primary exact native query operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeMode {
    /// Exact subject conjunction, optionally narrowed by attributed text.
    Subjects,
    /// Exact outgoing typed graph relation.
    Graph,
    /// Exact declaration-generator-artifact assertion.
    Productions,
}
/// Exact issuing cut, predicate, and page fence.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeCursor {
    /// Issuing tooling identity.
    pub issuing_store: String,
    /// Selected activated generation.
    pub generation: String,
    /// Exact predicate and extent identity.
    pub query_digest: String,
    /// Last native member key.
    pub after: String,
}
/// Exact conjunction of native facets; no relevance substitution or arbitrary SQL.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = knowledge_query_example())]
pub struct KnowledgeQuery {
    /// Required native document encoding version.
    pub version: pse_model::document::Version<1>,
    /// Null selects the independent current pointer.
    pub generation: Option<String>,
    /// Subject, graph, or production operation.
    pub mode: KnowledgeMode,
    /// Exact concept family.
    pub kind: Option<SubjectKind>,
    /// Exact native owner.
    pub owner: Option<String>,
    /// Exact native identity.
    pub native_id: Option<String>,
    /// Exact document role.
    pub document_role: Option<DocumentRole>,
    /// Exact native lifecycle.
    pub native_state: Option<String>,
    /// Exact publication scope, independent of lifecycle.
    pub publication_scope: Option<String>,
    /// All specified native topics must match.
    #[serde(default)]
    pub topics: Vec<String>,
    /// Subject selection or exact graph root.
    pub subject_id: Option<String>,
    /// Optional typed graph relation.
    pub relation: Option<RelationKind>,
    /// Exact production declaration.
    pub declaration_id: Option<String>,
    /// Exact production generator.
    pub generator_id: Option<String>,
    /// Exact production artifact.
    pub artifact_id: Option<String>,
    /// Optional text predicate within a structurally selected subject population.
    pub text: Option<String>,
    /// Page extent, one through 64.
    pub limit: usize,
    /// Exact-cut continuation.
    pub cursor: Option<KnowledgeCursor>,
}
fn knowledge_query_example() -> serde_json::Value {
    serde_json::json!({"version":1,"mode":"productions","artifact_id":"generated_artifact:python/pse/_native.pyi","limit":8})
}
impl KnowledgeQuery {
    /// Decode the version header before the closed request body.
    pub fn decode_json(bytes: &[u8]) -> Result<Self, pse_model::document::JsonDecodeError> {
        pse_model::document::decode_versioned::<Self, 1>(bytes)
    }
    fn structural(&self) -> bool {
        self.kind.is_some()
            || self.owner.is_some()
            || self.native_id.is_some()
            || self.document_role.is_some()
            || self.native_state.is_some()
            || self.publication_scope.is_some()
            || !self.topics.is_empty()
            || self.subject_id.is_some()
    }
    /// Check finite mode-specific predicates without opening a store.
    /// # Errors
    /// The query mixes modes, exceeds bounds or supplies invalid identities.
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if !(1..=64).contains(&self.limit)
            || self.generation.as_ref().is_some_and(|value| !hash(value))
            || self.topics.len() > 16
            || self
                .owner
                .iter()
                .chain(self.native_id.iter())
                .chain(self.native_state.iter())
                .chain(self.publication_scope.iter())
                .chain(self.subject_id.iter())
                .chain(self.declaration_id.iter())
                .chain(self.generator_id.iter())
                .chain(self.artifact_id.iter())
                .chain(self.text.iter())
                .chain(self.topics.iter())
                .any(|value| !bounded(value, 4096))
            || self.cursor.as_ref().is_some_and(|cursor| {
                !hash(&cursor.generation)
                    || !hash(&cursor.query_digest)
                    || !hash(&cursor.after)
                    || !bounded(&cursor.issuing_store, 4096)
            })
        {
            return Err(invalid("invalid finite structured query"));
        }
        let production = self.declaration_id.is_some()
            || self.generator_id.is_some()
            || self.artifact_id.is_some();
        match self.mode {
            KnowledgeMode::Subjects
                if !production
                    && self.relation.is_none()
                    && (self.text.is_none() || self.structural()) =>
            {
                Ok(())
            }
            KnowledgeMode::Graph
                if !production && self.subject_id.is_some() && self.text.is_none() =>
            {
                Ok(())
            }
            KnowledgeMode::Productions
                if production
                    && !self.structural()
                    && self.relation.is_none()
                    && self.text.is_none() =>
            {
                Ok(())
            }
            _ => Err(invalid("mode and exact native predicates differ")),
        }
    }
    fn identity(&self) -> Result<String, EvidenceError> {
        let mut request = self.clone();
        request.cursor = None;
        request.topics.sort();
        request.topics.dedup();
        Ok(digest(encoded(&request)?))
    }
}
/// Conservative interpretation limits, independent of the delivered hit population.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeCoverage {
    /// Producer fingerprints are not a live freshness attestation.
    pub producer_freshness: &'static str,
    /// Registry declaration definition locations are not fabricated.
    pub declaration_locations: &'static str,
    /// Missing semantic links are not inferred from prose or topics.
    pub undeclared_relationships: &'static str,
    /// Text matches only subjects explicitly attributed by original owners.
    pub excerpt_attribution: &'static str,
}
const COVERAGE: KnowledgeCoverage = KnowledgeCoverage {
    producer_freshness: "unverified",
    declaration_locations: "not-established",
    undeclared_relationships: "not-inferred",
    excerpt_attribution: "explicit-subject-only",
};
/// Finite structural result, with the complete provenance source cut even for zero hits.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgePage {
    /// Native result encoding version.
    pub version: pse_model::document::Version<1>,
    /// Issuing tooling authority.
    pub issuing_store: String,
    /// Exact activated corpus.
    pub generation: String,
    /// Imported cut only; no automatic source reopening.
    pub freshness: &'static str,
    /// Exact producer and semantic coverage limits.
    pub coverage: KnowledgeCoverage,
    /// Complete source membership for independent freshness observation.
    pub sources: Vec<KnowledgeSource>,
    /// Selected subjects or exact edge/production endpoint closure.
    pub subjects: Vec<KnowledgeSubject>,
    /// Optional matching subject-attributed excerpts.
    pub excerpts: Vec<KnowledgeExcerpt>,
    /// Exact typed graph results.
    pub edges: Vec<KnowledgeEdge>,
    /// Exact production assertions.
    pub productions: Vec<KnowledgeProduction>,
    /// Matching excerpt delivery is separately bounded; this flag preserves omissions.
    pub excerpts_truncated: bool,
    /// Same-predicate, same-cut continuation.
    pub next: Option<KnowledgeCursor>,
}
/// Nonexecuting versioned plan envelope; optimizer plan contents are opaque version-specific data.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeExplain {
    /// Native result encoding version.
    pub version: pse_model::document::Version<1>,
    /// Original issuing tooling identity.
    pub issuing_store: String,
    /// Exact selected active generation.
    pub generation: String,
    /// Original query operation.
    pub mode: KnowledgeMode,
    /// Exact predicate and extent identity.
    pub query_digest: String,
    /// Raw informational optimizer output, not a stable query grammar.
    pub raw_plan_json: String,
    /// Optional population selection's informational plan.
    pub population_plan_json: Option<String>,
    /// Original coverage limitations.
    pub coverage: KnowledgeCoverage,
    /// This read does not reopen original source files.
    pub freshness: &'static str,
    /// False: explaining never runs ANALYZE.
    pub experimental_execution: bool,
}
/// Settled original import; replay never rewinds a newer active pointer.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeReceipt {
    /// Exact corpus identity.
    pub generation: String,
    /// All normalized memberships reconciled and activated.
    pub activated: bool,
}
struct Prepared {
    generation: String,
    header: Object,
    rows: BTreeMap<&'static str, Vec<Value>>,
}
fn ordered(mut rows: Vec<(String, Object)>) -> Vec<Value> {
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter()
        .map(|(_, row)| Value::Object(row))
        .collect()
}
fn prepare(corpus: &KnowledgeCorpus) -> Result<Prepared, EvidenceError> {
    let generation = corpus.generation()?;
    let mut rows: BTreeMap<&'static str, Vec<(String, Object)>> = declaration::FAMILIES
        .iter()
        .map(|(table, _)| (*table, Vec::new()))
        .collect();
    macro_rules! push {
        ($table:literal,$key:expr,$encode:ident,$row:expr) => {{
            rows.get_mut($table)
                .ok_or_else(|| invalid("family absent"))?
                .push(($key, wire::$encode(&$row)?));
        }};
    }
    for source in &corpus.sources {
        let key = member(&generation, &source.id)?;
        push!(
            "knowledge_sources",
            key.clone(),
            encode_knowledge_sources,
            knowledge_sources::Row {
                key,
                generation: generation.clone(),
                source_id: source.id.clone(),
                kind: source.kind.clone(),
                locator: source.locator.clone(),
                version_digest: source.version_digest.clone(),
                basis: source.basis.parse().map_err(|_| invalid("basis"))?,
                title: source.title.clone(),
                status: source.status.parse().map_err(|_| invalid("status"))?
            }
        );
    }
    for subject in &corpus.subjects {
        let key = member(&generation, &subject.id)?;
        let evidence = subject.evidence.as_ref();
        push!(
            "knowledge_subjects",
            key.clone(),
            encode_knowledge_subjects,
            knowledge_subjects::Row {
                key: key.clone(),
                generation: generation.clone(),
                subject_id: subject.id.clone(),
                kind: subject.kind,
                owner: subject.owner.clone(),
                native_id: subject.native_id.clone(),
                label: subject.label.clone(),
                basis: subject.basis.parse().map_err(|_| invalid("basis"))?,
                native_state: subject.native_state.clone(),
                state_basis: subject.state_basis.clone(),
                publication_scope: subject.publication_scope.clone(),
                document_role: subject.document_role,
                native_version: subject.native_version.clone(),
                evidence_authority: evidence.map(|e| e.authority.clone()),
                evidence_codec: evidence.map(|e| e.codec.clone()),
                evidence_source: evidence.map(|e| e.source_id.clone()),
                evidence_version: evidence.map(|e| e.version.clone()),
                evidence_member: evidence.map(|e| e.member.clone())
            }
        );
        for source in &subject.sources {
            let key_link = member(
                &generation,
                (&subject.id, &source.source_id, source.role.as_str()),
            )?;
            push!(
                "knowledge_subject_sources",
                key_link.clone(),
                encode_knowledge_subject_sources,
                knowledge_subject_sources::Row {
                    key: key_link,
                    generation: generation.clone(),
                    subject_key: key.clone(),
                    source_key: member(&generation, &source.source_id)?,
                    role: source.role
                }
            );
        }
        for topic in &subject.topics {
            let key_link = member(&generation, (&subject.id, &topic.topic, &topic.source_id))?;
            push!(
                "knowledge_subject_topics",
                key_link.clone(),
                encode_knowledge_subject_topics,
                knowledge_subject_topics::Row {
                    key: key_link,
                    generation: generation.clone(),
                    subject_key: key.clone(),
                    source_key: member(&generation, &topic.source_id)?,
                    topic: topic.topic.clone()
                }
            );
        }
    }
    for excerpt in &corpus.excerpts {
        let key = member(&generation, &excerpt.id)?;
        push!(
            "knowledge_excerpts",
            key.clone(),
            encode_knowledge_excerpts,
            knowledge_excerpts::Row {
                key,
                generation: generation.clone(),
                excerpt_id: excerpt.id.clone(),
                source_key: member(&generation, &excerpt.source_id)?,
                subject_key: excerpt
                    .subject_id
                    .as_ref()
                    .map(|id| member(&generation, id))
                    .transpose()?,
                text: excerpt.text.clone(),
                locator: excerpt.locator.clone(),
                basis: excerpt.basis.parse().map_err(|_| invalid("basis"))?
            }
        );
    }
    for edge in &corpus.edges {
        let key = member(&generation, &edge.id)?;
        push!(
            "knowledge_subject_edges",
            key.clone(),
            encode_knowledge_subject_edges,
            knowledge_subject_edges::Row {
                key,
                generation: generation.clone(),
                edge_id: edge.id.clone(),
                source_key: member(&generation, &edge.from)?,
                target_key: member(&generation, &edge.to)?,
                relation: edge.relation,
                basis: edge.basis.parse().map_err(|_| invalid("basis"))?
            }
        );
    }
    for production in &corpus.productions {
        let key = member(&generation, &production.id)?;
        push!(
            "knowledge_productions",
            key.clone(),
            encode_knowledge_productions,
            knowledge_productions::Row {
                key,
                generation: generation.clone(),
                production_id: production.id.clone(),
                declaration_key: member(&generation, &production.declaration_id)?,
                generator_key: member(&generation, &production.generator_id)?,
                artifact_key: member(&generation, &production.artifact_id)?,
                direction: production.direction,
                basis: production.basis.parse().map_err(|_| invalid("basis"))?
            }
        );
    }
    let count = |table: &str| {
        rows.get(table)
            .and_then(|values| u64::try_from(values.len()).ok())
            .ok_or_else(|| invalid("family count absent"))
    };
    let header = knowledge_generations::Row {
        key: generation.clone(),
        content_digest: generation.clone(),
        source_count: count("knowledge_sources")?,
        subject_count: count("knowledge_subjects")?,
        contributor_count: count("knowledge_subject_sources")?,
        topic_count: count("knowledge_subject_topics")?,
        excerpt_count: count("knowledge_excerpts")?,
        edge_count: count("knowledge_subject_edges")?,
        production_count: count("knowledge_productions")?,
        state: KnowledgeGenerationState::Staged,
    };
    Ok(Prepared {
        generation,
        header: wire::encode_knowledge_generations(&header)?,
        rows: rows
            .into_iter()
            .map(|(table, rows)| (table, ordered(rows)))
            .collect(),
    })
}
fn limits() -> ResponseLimits {
    ResponseLimits {
        wire_bytes: canonical::MESSAGE_BYTES,
        nodes: 131072,
        depth: 16,
        container_items: 4097,
        string_bytes: canonical::PAYLOAD_BYTES + 524288,
        string_length: 16384,
        blob_bytes: 0,
        blob_length: 0,
    }
}
const HEADER: &str = r#"BEGIN;
IF time::micros()>=$expiry { THROW 'knowledge original deadline expired'; };
LET $marker=SELECT * FROM ONLY evidence_interpretations:current;
IF $marker.issuing_store!=$issuer { THROW 'knowledge authority changed'; };
LET $row=SELECT * FROM ONLY type::record('knowledge_generations',$generation);
"#;
fn reconciliation() -> String {
    declaration::FAMILIES.iter().map(|(table,count)| {
        let parameter="$".to_owned()+*table;
        format!("LET $actual=SELECT * OMIT id FROM {table} WHERE generation=$generation ORDER BY key; IF $actual!={parameter} OR <decimal>array::len({parameter})!=$row.{count} {{ THROW 'knowledge exact {table} membership differs'; }};")
    }).collect()
}
fn bind_rows<'a>(mut query: Query<'a>, prepared: &Prepared) -> Query<'a> {
    for (table, rows) in &prepared.rows {
        query = query.bind(((*table).to_owned(), rows.clone()));
    }
    query
}
fn stage_sql() -> String {
    let mut sql = format!(
        "{HEADER} IF $row!=NONE AND $row.content_digest!=$generation {{ THROW 'knowledge identity reused'; }}; IF $row=NONE {{ CREATE type::record('knowledge_generations',$generation) CONTENT $header;"
    );
    for (table, _) in declaration::FAMILIES {
        let parameter = "$".to_owned() + *table;
        if declaration::ENFORCED_RELATIONS.contains(table) {
            sql.push_str(&format!("FOR $item IN {parameter} {{ INSERT RELATION INTO {table} object::extend($item,{{id:type::record('{table}',$item.key)}}) RETURN NONE; }};"));
        } else {
            sql.push_str(&format!("FOR $item IN {parameter} {{ CREATE type::record('{table}',$item.key) CONTENT $item RETURN NONE; }};"));
        }
    }
    sql.push_str(
        "}; IF time::micros()>=$expiry { THROW 'knowledge original deadline expired'; }; COMMIT;",
    );
    sql
}
fn activation_sql(active: Option<bool>) -> String {
    let mut sql = format!(
        "{HEADER} IF $row=NONE OR $row.content_digest!=$generation {{ THROW 'knowledge generation absent'; }}; {}",
        reconciliation()
    );
    match active {
        None=>sql.push_str("IF $row.state!='active' { LET $pointer=SELECT * FROM ONLY knowledge_state:current; IF $pointer=NONE OR $pointer.active_generation!=$expected_active { THROW 'knowledge activation pointer changed'; }; UPDATE type::record('knowledge_generations',$generation) SET state='active'; UPDATE knowledge_state:current SET active_generation=$generation; };"),
        Some(true)=>sql.push_str("IF $row.state!='active' { THROW 'knowledge generation is not activated'; };"),
        Some(false)=>{},
    }
    sql.push_str(
        "IF time::micros()>=$expiry { THROW 'knowledge original deadline expired'; }; COMMIT;",
    );
    sql
}
impl EvidenceStore {
    fn knowledge_importer(&self) -> Result<(), EvidenceError> {
        if matches!(self.role, EvidenceAccess::Importer) {
            Ok(())
        } else {
            Err(invalid("reader cannot mutate knowledge"))
        }
    }
    /// Stage, reconcile all normalized memberships and atomically activate this exact corpus.
    pub async fn knowledge_import(
        &self,
        corpus: &KnowledgeCorpus,
    ) -> Result<KnowledgeReceipt, EvidenceError> {
        within_allowance(
            original_deadline(canonical::REQUEST_TIMEOUT).into_std(),
            async {
                self.knowledge_importer()?;
                let prepared = prepare(corpus)?;
                self.knowledge_stage(&prepared).await?;
                self.knowledge_activate(corpus, &prepared).await
            },
        )
        .await
    }
    async fn knowledge_stage(&self, prepared: &Prepared) -> Result<(), EvidenceError> {
        self.knowledge_importer()?;
        self.marker().await?;
        let sql = stage_sql();
        let result = transport::protected_query("knowledge-stage", || {
            self.query(sql.clone())
                .map(|query| {
                    bind_rows(
                        query
                            .bind(("issuer", self.issuing_store.clone()))
                            .bind(("generation", prepared.generation.clone()))
                            .bind(("header", prepared.header.clone())),
                        prepared,
                    )
                })
                .map_err(|cause| CanonicalError::Configuration(cause.to_string()))
        })
        .await;
        if let Err(source) = result
            && self.knowledge_reconcile(prepared, false).await.is_err()
        {
            return Err(EvidenceError::Unsettled {
                generation: prepared.generation.clone(),
                source: Box::new(source),
            });
        }
        Ok(())
    }
    async fn knowledge_activate(
        &self,
        corpus: &KnowledgeCorpus,
        prepared: &Prepared,
    ) -> Result<KnowledgeReceipt, EvidenceError> {
        self.knowledge_importer()?;
        let sql = activation_sql(None);
        let result = transport::protected_query("knowledge-activate", || {
            self.query(sql.clone())
                .map(|query| {
                    bind_rows(
                        query
                            .bind(("issuer", self.issuing_store.clone()))
                            .bind(("generation", prepared.generation.clone()))
                            .bind(("expected_active", corpus.expected_active.clone())),
                        prepared,
                    )
                })
                .map_err(|cause| CanonicalError::Configuration(cause.to_string()))
        })
        .await;
        #[cfg(test)]
        let result = if self
            .ack_fault
            .compare_exchange(
                3,
                0,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
        {
            result.and_then(|_| Err(CanonicalError::IncompleteResponse))
        } else {
            result
        };
        if let Err(source) = result
            && self.knowledge_reconcile(prepared, true).await.is_err()
        {
            return Err(EvidenceError::Unsettled {
                generation: prepared.generation.clone(),
                source: Box::new(source),
            });
        }
        Ok(KnowledgeReceipt {
            generation: prepared.generation.clone(),
            activated: true,
        })
    }
    async fn knowledge_reconcile(
        &self,
        prepared: &Prepared,
        active: bool,
    ) -> Result<(), EvidenceError> {
        transport::bounded_query(bind_rows(
            self.query(activation_sql(Some(active)))?
                .bind(("issuer", self.issuing_store.clone()))
                .bind(("generation", prepared.generation.clone())),
            prepared,
        ))
        .await?;
        Ok(())
    }
    async fn knowledge_generation(
        &self,
        generation: &str,
    ) -> Result<knowledge_generations::Row, EvidenceError> {
        let mut response=transport::bounded_query(self.query("BEGIN; SELECT * FROM ONLY evidence_interpretations:current; SELECT * FROM ONLY type::record('knowledge_generations',$generation); COMMIT;")?.bind(("generation",generation.to_owned()))).await?;
        let marker: Option<Object> = response.take(1)?;
        self.decode_marker(
            marker.ok_or_else(|| EvidenceError::Unavailable("tooling marker absent".into()))?,
            true,
        )?;
        let row: Option<Object> = response.take(2)?;
        Ok(wire::decode_knowledge_generations(row.ok_or_else(
            || EvidenceError::Unavailable("knowledge generation absent".into()),
        )?)?)
    }
    /// Attest the independent knowledge pointer and schema. Opening never initializes it.
    pub async fn knowledge_status(&self) -> Result<serde_json::Value, EvidenceError> {
        let mut response=transport::bounded_query(self.query("BEGIN; SELECT * FROM ONLY evidence_interpretations:current; SELECT * FROM ONLY knowledge_state:current; COMMIT;")?).await?;
        let marker: Option<Object> = response.take(1)?;
        let marker = self.decode_marker(
            marker.ok_or_else(|| EvidenceError::Unavailable("tooling marker absent".into()))?,
            true,
        )?;
        let state: Option<Object> = response.take(2)?;
        let state = wire::decode_knowledge_state(state.ok_or_else(|| {
            EvidenceError::Unavailable(
                "knowledge state absent; explicit tooling rebuild required".into(),
            )
        })?)?;
        Ok(
            serde_json::json!({"issuing_store":marker.issuing_store,"interpretation":wire::INTERPRETATION,"schema_digest":wire::SCHEMA_DIGEST,"active_generation":state.active_generation,"freshness":"imported-cut-only"}),
        )
    }
    /// Remove only the derived knowledge family, leaving evidence and native sources intact.
    pub async fn knowledge_reset(&self) -> Result<(), EvidenceError> {
        self.knowledge_importer()?;
        self.marker().await?;
        let mut sql="BEGIN; IF time::micros()>=$expiry { THROW 'knowledge deadline expired'; }; LET $marker=SELECT * FROM ONLY evidence_interpretations:current; IF $marker.issuing_store!=$issuer { THROW 'knowledge authority changed'; };".to_owned();
        for (table, _) in declaration::FAMILIES.iter().rev() {
            sql.push_str(&format!("DELETE {table};"));
        }
        sql.push_str("DELETE knowledge_generations; UPDATE knowledge_state:current SET active_generation=NONE; IF time::micros()>=$expiry { THROW 'knowledge deadline expired'; }; COMMIT;");
        transport::protected_query("knowledge-reset", || {
            self.query(sql.clone())
                .map(|query| query.bind(("issuer", self.issuing_store.clone())))
                .map_err(|cause| CanonicalError::Configuration(cause.to_string()))
        })
        .await?;
        Ok(())
    }
    /// Query exact owner-native facets and relationships on one native read snapshot.
    pub async fn knowledge_select(
        &self,
        request: &KnowledgeQuery,
    ) -> Result<KnowledgePage, EvidenceError> {
        request.validate()?;
        let query_digest = request.identity()?;
        if request.cursor.as_ref().is_some_and(|cursor| {
            cursor.issuing_store != self.issuing_store || cursor.query_digest != query_digest
        }) {
            return Err(EvidenceError::Cursor);
        }
        let (prefix, item_query) = selection(request);
        let keys = match request.mode {
            KnowledgeMode::Subjects => "$items.key",
            KnowledgeMode::Graph => "array::concat($items.source_key,$items.target_key)",
            KnowledgeMode::Productions => {
                "array::concat($items.declaration_key,$items.generator_key,$items.artifact_key)"
            }
        };
        let excerpt_query = if request.text.is_some() {
            "SELECT * FROM knowledge_excerpts WHERE generation=$selected AND subject_key IN $keys AND text @0@ $text ORDER BY key LIMIT 65"
        } else {
            "[]"
        };
        let sql = format!(
            "BEGIN; LET $marker=SELECT * FROM ONLY evidence_interpretations:current; LET $pointer=SELECT * FROM ONLY knowledge_state:current; LET $selected=IF $generation=NONE {{ $pointer.active_generation }} ELSE {{ $generation }}; LET $header=SELECT * FROM ONLY type::record('knowledge_generations',$selected); {prefix} LET $items={item_query}; LET $keys={keys}; LET $sources=SELECT * FROM knowledge_sources WHERE generation=$selected ORDER BY key LIMIT 257; LET $subjects=SELECT * FROM knowledge_subjects WHERE generation=$selected AND key IN $keys LIMIT 257; LET $contributors=SELECT * FROM knowledge_subject_sources WHERE generation=$selected AND subject_key IN $keys LIMIT 2049; LET $topics=SELECT * FROM knowledge_subject_topics WHERE generation=$selected AND subject_key IN $keys LIMIT 4097; LET $excerpts={excerpt_query}; SELECT VALUE {{marker:$marker,header:$header,sources:$sources,subjects:$subjects,contributors:$contributors,topics:$topics,excerpts:$excerpts,items:$items}} FROM [0]; COMMIT;"
        );
        let mut response = transport::bounded_query(bind_request(
            self.query_with_limits(sql, limits())?,
            request,
        ))
        .await?;
        let index = response
            .num_statements()
            .checked_sub(2)
            .ok_or_else(|| invalid("native page response absent"))?;
        let mut records: Vec<Object> = response.take(index)?;
        if records.len() != 1 {
            return Err(invalid("native page envelope differs"));
        }
        let mut record = records.remove(0);
        self.decode_marker(
            as_object(pse_operations::canonical_codec::required(
                &mut record,
                "marker",
            )?)?,
            true,
        )?;
        let generation = wire::decode_knowledge_generations(as_object(
            pse_operations::canonical_codec::required(&mut record, "header")?,
        )?)?;
        if generation.state != KnowledgeGenerationState::Active
            || request
                .cursor
                .as_ref()
                .is_some_and(|cursor| cursor.generation != generation.key)
        {
            return Err(EvidenceError::Cursor);
        }
        let source_rows = objects(&mut record, "sources")?;
        if !(1..=256).contains(&generation.source_count)
            || u64::try_from(source_rows.len()).ok() != Some(generation.source_count)
        {
            return Err(EvidenceError::Unavailable(
                "complete source membership differs".into(),
            ));
        }
        let mut sources = BTreeMap::new();
        for object in source_rows {
            let source = wire::decode_knowledge_sources(object)?;
            if source.generation != generation.key
                || source.key != member(&generation.key, &source.source_id)?
            {
                return Err(EvidenceError::Cursor);
            }
            sources.insert(
                source.key,
                KnowledgeSource {
                    id: source.source_id,
                    kind: source.kind,
                    locator: source.locator,
                    version_digest: source.version_digest,
                    basis: source.basis.as_str().into(),
                    title: source.title,
                    status: source.status.as_str().into(),
                },
            );
        }
        let mut subjects = BTreeMap::new();
        for object in objects(&mut record, "subjects")? {
            let row = wire::decode_knowledge_subjects(object)?;
            if row.generation != generation.key
                || row.key != member(&generation.key, &row.subject_id)?
            {
                return Err(EvidenceError::Cursor);
            }
            let evidence = match row.evidence_authority {
                Some(authority) => Some(EvidenceIdentity {
                    authority,
                    codec: row
                        .evidence_codec
                        .ok_or_else(|| invalid("evidence codec absent"))?,
                    source_id: row
                        .evidence_source
                        .ok_or_else(|| invalid("evidence source absent"))?,
                    version: row
                        .evidence_version
                        .ok_or_else(|| invalid("evidence version absent"))?,
                    member: row
                        .evidence_member
                        .ok_or_else(|| invalid("evidence member absent"))?,
                }),
                None => None,
            };
            subjects.insert(
                row.key,
                KnowledgeSubject {
                    id: row.subject_id,
                    kind: row.kind,
                    owner: row.owner,
                    native_id: row.native_id,
                    label: row.label,
                    basis: row.basis.as_str().into(),
                    native_state: row.native_state,
                    state_basis: row.state_basis,
                    publication_scope: row.publication_scope,
                    document_role: row.document_role,
                    native_version: row.native_version,
                    evidence,
                    sources: Vec::new(),
                    topics: Vec::new(),
                },
            );
        }
        for object in objects(&mut record, "contributors")? {
            let row = wire::decode_knowledge_subject_sources(object)?;
            if row.generation != generation.key {
                return Err(EvidenceError::Cursor);
            }
            let source = sources
                .get(&row.source_key)
                .ok_or_else(|| invalid("contributor source absent"))?;
            subjects
                .get_mut(&row.subject_key)
                .ok_or_else(|| invalid("contributor subject absent"))?
                .sources
                .push(SubjectSource {
                    source_id: source.id.clone(),
                    role: row.role,
                });
        }
        for object in objects(&mut record, "topics")? {
            let row = wire::decode_knowledge_subject_topics(object)?;
            if row.generation != generation.key {
                return Err(EvidenceError::Cursor);
            }
            let source = sources
                .get(&row.source_key)
                .ok_or_else(|| invalid("topic metadata source absent"))?;
            subjects
                .get_mut(&row.subject_key)
                .ok_or_else(|| invalid("topic subject absent"))?
                .topics
                .push(SubjectTopic {
                    topic: row.topic,
                    source_id: source.id.clone(),
                });
        }
        let source_ids = sources.values().map(|source| &source.id).collect();
        for subject in subjects.values_mut() {
            subject.sources.sort_by(|a, b| {
                (&a.source_id, a.role.as_str()).cmp(&(&b.source_id, b.role.as_str()))
            });
            subject
                .topics
                .sort_by(|a, b| (&a.topic, &a.source_id).cmp(&(&b.topic, &b.source_id)));
            subject_valid(subject, &source_ids)?;
        }
        let mut items = objects(&mut record, "items")?;
        let more = items.len() > request.limit;
        items.truncate(request.limit);
        let mut required = BTreeSet::new();
        let mut edges = Vec::new();
        let mut productions = Vec::new();
        let mut after = None;
        for item in items {
            match request.mode {
                KnowledgeMode::Subjects => {
                    let row = wire::decode_knowledge_subjects(item)?;
                    if row.generation != generation.key {
                        return Err(EvidenceError::Cursor);
                    }
                    required.insert(row.key.clone());
                    after = Some(row.key);
                }
                KnowledgeMode::Graph => {
                    let row = wire::decode_knowledge_subject_edges(item)?;
                    if row.generation != generation.key {
                        return Err(EvidenceError::Cursor);
                    }
                    let from = subjects
                        .get(&row.source_key)
                        .ok_or_else(|| invalid("edge source absent"))?;
                    let to = subjects
                        .get(&row.target_key)
                        .ok_or_else(|| invalid("edge target absent"))?;
                    required.insert(row.source_key);
                    required.insert(row.target_key);
                    after = Some(row.key);
                    edges.push(KnowledgeEdge {
                        id: row.edge_id,
                        from: from.id.clone(),
                        to: to.id.clone(),
                        relation: row.relation,
                        basis: row.basis.as_str().into(),
                    });
                }
                KnowledgeMode::Productions => {
                    let row = wire::decode_knowledge_productions(item)?;
                    if row.generation != generation.key {
                        return Err(EvidenceError::Cursor);
                    }
                    let get = |key: &str| {
                        subjects
                            .get(key)
                            .map(|subject| subject.id.clone())
                            .ok_or_else(|| invalid("production subject absent"))
                    };
                    productions.push(KnowledgeProduction {
                        id: row.production_id,
                        declaration_id: get(&row.declaration_key)?,
                        generator_id: get(&row.generator_key)?,
                        artifact_id: get(&row.artifact_key)?,
                        direction: row.direction,
                        basis: row.basis.as_str().into(),
                    });
                    required.insert(row.declaration_key);
                    required.insert(row.generator_key);
                    required.insert(row.artifact_key);
                    after = Some(row.key);
                }
            }
        }
        let mut excerpt_rows = objects(&mut record, "excerpts")?;
        let excerpts_truncated = excerpt_rows.len() > 64;
        excerpt_rows.truncate(64);
        let mut excerpts = Vec::new();
        for object in excerpt_rows {
            let row = wire::decode_knowledge_excerpts(object)?;
            if row.generation != generation.key {
                return Err(EvidenceError::Cursor);
            }
            if let Some(key) = &row.subject_key
                && required.contains(key)
            {
                let subject = subjects
                    .get(key)
                    .ok_or_else(|| invalid("excerpt subject absent"))?;
                let source = sources
                    .get(&row.source_key)
                    .ok_or_else(|| invalid("excerpt source absent"))?;
                excerpts.push(KnowledgeExcerpt {
                    id: row.excerpt_id,
                    source_id: source.id.clone(),
                    subject_id: Some(subject.id.clone()),
                    text: row.text,
                    locator: row.locator,
                    basis: row.basis.as_str().into(),
                });
            }
        }
        let subjects = subjects
            .into_iter()
            .filter_map(|(key, subject)| required.contains(&key).then_some(subject))
            .collect();
        let next = if more {
            after.map(|after| KnowledgeCursor {
                issuing_store: self.issuing_store.clone(),
                generation: generation.key.clone(),
                query_digest,
                after,
            })
        } else {
            None
        };
        Ok(KnowledgePage {
            version: pse_model::document::Version,
            issuing_store: self.issuing_store.clone(),
            generation: generation.key,
            freshness: "imported-cut-only",
            coverage: COVERAGE,
            sources: sources.into_values().collect(),
            subjects,
            excerpts,
            edges,
            productions,
            excerpts_truncated,
            next,
        })
    }
    /// Explain the same native item query without ANALYZE or source reopening.
    pub async fn knowledge_explain(
        &self,
        request: &KnowledgeQuery,
    ) -> Result<KnowledgeExplain, EvidenceError> {
        request.validate()?;
        let status = self.knowledge_status().await?;
        let generation = request
            .generation
            .as_deref()
            .or_else(|| status["active_generation"].as_str())
            .ok_or_else(|| EvidenceError::Unavailable("no active knowledge cut".into()))?
            .to_owned();
        if self.knowledge_generation(&generation).await?.state != KnowledgeGenerationState::Active {
            return Err(EvidenceError::Cursor);
        }
        let identity = request.identity()?;
        if request.cursor.as_ref().is_some_and(|cursor| {
            cursor.issuing_store != self.issuing_store
                || cursor.generation != generation
                || cursor.query_digest != identity
        }) {
            return Err(EvidenceError::Cursor);
        }
        let (prefix, items) = selection(request);
        let sql = format!("LET $selected=$generation; {prefix} EXPLAIN FORMAT JSON {items};");
        let population_plan = if let Some(population) = prefix
            .strip_prefix("LET $population=")
            .and_then(|value| value.strip_suffix(';'))
        {
            let sql = format!("LET $selected=$generation; EXPLAIN FORMAT JSON {population};");
            let mut response = transport::bounded_query(
                bind_request(
                    self.query_with_limits(
                        sql,
                        canonical::MetadataResponse::DiagnosticPlan.limits(),
                    )?,
                    request,
                )
                .bind(("generation", generation.clone())),
            )
            .await?;
            let index = response
                .num_statements()
                .checked_sub(1)
                .ok_or_else(|| invalid("native population plan absent"))?;
            Some(response.take::<Value>(index)?)
        } else {
            None
        };
        let mut response = transport::bounded_query(
            bind_request(
                self.query_with_limits(sql, canonical::MetadataResponse::DiagnosticPlan.limits())?,
                request,
            )
            .bind(("generation", generation.clone())),
        )
        .await?;
        let index = response
            .num_statements()
            .checked_sub(1)
            .ok_or_else(|| invalid("native plan absent"))?;
        let plan: Value = response.take(index)?;
        let after = self.knowledge_status().await?;
        if request.generation.is_none() && status["active_generation"] != after["active_generation"]
        {
            return Err(EvidenceError::Cursor);
        }
        Ok(KnowledgeExplain {
            version: pse_model::document::Version,
            issuing_store: self.issuing_store.clone(),
            generation,
            mode: request.mode,
            query_digest: identity,
            raw_plan_json: serde_json::to_string(&plan)
                .map_err(|cause| invalid(cause.to_string()))?,
            population_plan_json: population_plan
                .map(|value| serde_json::to_string(&value))
                .transpose()
                .map_err(|cause| invalid(cause.to_string()))?,
            coverage: COVERAGE,
            freshness: "imported-cut-only",
            experimental_execution: false,
        })
    }
}
fn as_object(value: Value) -> Result<Object, EvidenceError> {
    match value {
        Value::Object(value) => Ok(value),
        _ => Err(invalid("native object absent")),
    }
}
fn objects(object: &mut Object, field: &'static str) -> Result<Vec<Object>, EvidenceError> {
    match pse_operations::canonical_codec::required(object, field)? {
        Value::Array(values) => values.into_vec().into_iter().map(as_object).collect(),
        _ => Err(invalid("native array absent")),
    }
}
fn subject_filter(request: &KnowledgeQuery, graph: bool) -> String {
    let field = if graph { "out." } else { "" };
    let key = if graph { "target_key" } else { "key" };
    let mut filter = String::new();
    for (present, name) in [
        (request.kind.is_some(), "kind"),
        (request.owner.is_some(), "owner"),
        (request.native_id.is_some(), "native_id"),
        (request.document_role.is_some(), "document_role"),
        (request.native_state.is_some(), "native_state"),
        (request.publication_scope.is_some(), "publication_scope"),
    ] {
        if present {
            let parameter = "$".to_owned() + name;
            filter.push_str(&format!(" AND {field}{name}={parameter}"));
        }
    }
    for (index, _) in request.topics.iter().enumerate() {
        filter.push_str(&format!(" AND array::len((SELECT VALUE key FROM knowledge_subject_topics WHERE generation=$selected AND subject_key=$parent.{key} AND topic=$topic_{index} LIMIT 1))=1"));
    }
    filter
}
fn selection(request: &KnowledgeQuery) -> (String, String) {
    match request.mode {
        KnowledgeMode::Subjects => {
            let mut filter = subject_filter(request, false);
            if request.subject_id.is_some() {
                filter.push_str(" AND subject_id=$subject_id");
            }
            let population = format!(
                "SELECT * FROM knowledge_subjects WHERE generation=$selected{filter} LIMIT 257"
            );
            let text = if request.text.is_some() {
                " AND key IN (SELECT VALUE subject_key FROM knowledge_excerpts WHERE generation=$selected AND subject_key IN $population.key AND text @0@ $text)"
            } else {
                ""
            };
            (
                format!("LET $population={population};"),
                format!(
                    "SELECT * FROM $population WHERE key>$after{text} ORDER BY key LIMIT $limit"
                ),
            )
        }
        KnowledgeMode::Graph => {
            let filter = subject_filter(request, true);
            let relation = if request.relation.is_some() {
                " AND relation=$relation"
            } else {
                ""
            };
            ("LET $root=(SELECT VALUE id FROM knowledge_subjects WHERE generation=$selected AND subject_id=$subject_id LIMIT 1)[0];".into(),format!("SELECT * FROM $root->(knowledge_subject_edges WHERE generation=$selected AND key>$after{relation}{filter}) ORDER BY key LIMIT $limit"))
        }
        KnowledgeMode::Productions => {
            let mut filter = String::new();
            for (present, field, name) in [
                (
                    request.declaration_id.is_some(),
                    "declaration_key_record",
                    "declaration_id",
                ),
                (
                    request.generator_id.is_some(),
                    "generator_key_record",
                    "generator_id",
                ),
                (
                    request.artifact_id.is_some(),
                    "artifact_key_record",
                    "artifact_id",
                ),
            ] {
                if present {
                    let parameter = "$".to_owned() + name;
                    filter.push_str(&format!(" AND {field}.subject_id={parameter}"));
                }
            }
            (
                String::new(),
                format!(
                    "SELECT * FROM knowledge_productions WHERE generation=$selected AND key>$after{filter} ORDER BY key LIMIT $limit"
                ),
            )
        }
    }
}
fn bind_request<'a>(mut query: Query<'a>, request: &KnowledgeQuery) -> Query<'a> {
    query = query
        .bind(("generation", request.generation.clone()))
        .bind(("kind", request.kind.map(|value| value.as_str().to_owned())))
        .bind(("owner", request.owner.clone()))
        .bind(("native_id", request.native_id.clone()))
        .bind((
            "document_role",
            request.document_role.map(|value| value.as_str().to_owned()),
        ))
        .bind(("native_state", request.native_state.clone()))
        .bind(("publication_scope", request.publication_scope.clone()))
        .bind(("subject_id", request.subject_id.clone()))
        .bind((
            "relation",
            request.relation.map(|value| value.as_str().to_owned()),
        ))
        .bind(("declaration_id", request.declaration_id.clone()))
        .bind(("generator_id", request.generator_id.clone()))
        .bind(("artifact_id", request.artifact_id.clone()))
        .bind(("text", request.text.clone()))
        .bind(("limit", (request.limit + 1) as i64))
        .bind((
            "after",
            request
                .cursor
                .as_ref()
                .map(|cursor| cursor.after.clone())
                .unwrap_or_default(),
        ));
    for (index, topic) in request.topics.iter().enumerate() {
        query = query.bind((format!("topic_{index}"), topic.clone()));
    }
    query
}
#[cfg(test)]
#[path = "knowledge_tests.rs"]
pub(crate) mod tests;
