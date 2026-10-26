// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected offline publication. The index alone authorizes local member reads.
use crate::tooling::{frozen_inspection::FrozenEvidenceSet, review_analysis::AnalysisDocument};
use crate::workflow::PackResultSelection;
use pse_columnar::CancellationToken;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

mod presentation;
mod references;
#[cfg(test)]
mod tests;

/// Finite metadata ceiling, independent of potentially streamed scientific tables.
pub const MAXIMUM_PACK_DOCUMENT_BYTES: usize = 64 << 20;
const MAXIMUM_MEMBERS: usize = 65536;
const INDEX: &str = "index.json";
/// Explicit selected role; availability never implies a scientific outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PackMemberRole {
    /// Supplied frozen facts and their original recorded identities.
    FrozenEvidence,
    /// Shared typed explanations, hypotheses and limitations.
    Analysis,
    /// Original model, inputs, settings and source interpretation.
    Context,
    /// Acknowledged coherent diagnostic record prefixes.
    Detail,
    /// Selected raw traces, profiles and capture logs.
    Capture,
    /// Copied engine plans, policies and operator observations.
    Engine,
    /// Selected analysis query and policy inputs.
    Query,
    /// Protected typed scientific table selection.
    Result,
    /// Human-readable local projection of the authoritative facts.
    Report,
}
/// Actual bytes encoding. IPC stream and IPC file are deliberately different contracts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PackFormat {
    /// FrozenEvidenceSet version 3 JSON codec.
    FrozenEvidenceV3,
    /// AnalysisDocument version 2 JSON codec.
    AnalysisV2,
    /// Original owner JSON document; schema is not inferred.
    Json,
    /// Acknowledged newline-delimited original records.
    JsonLines,
    /// Finalized Arrow IPC streaming format, including terminal continuation.
    ArrowIpcStream,
    /// Ordinary UTF-8 descriptive text or capture logs.
    Text,
    /// Local human-readable Markdown projection.
    Markdown,
    /// Self-contained static HTML with no executable assets.
    Html,
    /// Original selected Perfetto trace bytes.
    Perfetto,
    /// Original profiler binary bytes, with separate capture interpretation.
    ProfileBinary,
}
impl PackFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::FrozenEvidenceV3 | Self::AnalysisV2 | Self::Json => "json",
            Self::JsonLines => "jsonl",
            Self::ArrowIpcStream => "arrow",
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Text => "txt",
            Self::Perfetto => "pftrace",
            Self::ProfileBinary => "data",
        }
    }
}
/// Namespaced source coordinate. Original paths are never resolution directives.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackSourceReference {
    /// Selected pack member identity.
    pub member: String,
    /// JSON pointer, event coordinate or named field within that selected member.
    pub pointer: String,
}
/// One explicitly requested member. Missing optional members differ from unrequested members.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackMemberSelection {
    /// Unique explicit member identity within this pack.
    pub id: String,
    /// Selected explanatory or scientific role.
    pub role: PackMemberRole,
    /// Whether a failed copy prevents complete selected publication.
    pub required: bool,
    /// Declared actual bytes encoding and interpretation version.
    pub format: PackFormat,
    /// Original JSON codec version, when declared. None means interpretation is unspecified.
    #[serde(default)]
    pub document_version: Option<u32>,
    /// Descriptive original source/version/clock selection, never followed by the reader.
    pub source: Option<String>,
    #[serde(default)]
    /// Explicit member-namespaced explanatory dependencies.
    pub references: Vec<PackSourceReference>,
    /// Exact protected scientific selection; present only for result members.
    pub result: Option<PackResultSelection>,
    /// Copied capture sampling/loss/termination scope. No seal can change this text.
    pub capture_scope: Option<String>,
    /// Actual registry diagnostic projection, independent of scientific lineage.
    #[serde(default)]
    pub diagnostic_table: Option<PackDiagnosticTable>,
}
/// Exact diagnostic projection and its declared registry interpretation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackDiagnosticTable {
    /// Selected immutable source and bounded allocation.
    pub request: crate::tooling::review_query::ReviewTableRequest,
    /// Qualified actual registry relation.
    pub relation: String,
    /// Actual registry relation schema version.
    pub schema_version: u32,
    /// Actual supplied source coordinate.
    pub source: PackSourceReference,
}
impl PackDiagnosticTable {
    /// Resolve the actual pinned registry interpretation without projecting or querying data.
    pub fn new(
        request: crate::tooling::review_query::ReviewTableRequest,
    ) -> Result<Self, PackError> {
        let name = crate::tooling::review_query::table_relation(&request);
        let registry = pse_schema::registry().map_err(|e| invalid(e.to_string()))?;
        let relation = registry
            .relation(name)
            .ok_or_else(|| invalid("unknown diagnostic registry relation"))?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, relation)
            .map_err(|e| invalid(e.to_string()))?;
        let schema_version = schema
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_VERSION)
            .and_then(|version| version.parse::<u32>().ok())
            .ok_or_else(|| invalid("registry schema version is absent"))?;
        let source = PackSourceReference {
            member: "frozen".into(),
            pointer: crate::tooling::review_query::table_source(&request),
        };
        Ok(Self {
            request,
            relation: name.into(),
            schema_version,
            source,
        })
    }
}
/// One requested diagnostic table from the supplied immutable set.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackTableInput {
    /// Exact selected pack member.
    pub member: String,
    /// Existing declared registry projection.
    pub request: crate::tooling::review_query::ReviewTableRequest,
}
/// Versioned complete explicit membership and finite publication allowances.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackSelection {
    /// Supported version of this document contract.
    pub version: Version<2>,
    /// Complete selected membership; no first-page completeness inference.
    pub members: Vec<PackMemberSelection>,
    /// Total selected payload bytes. Index metadata has the independent finite document allowance.
    pub maximum_output_bytes: u64,
    /// Finite encoded size allowance per index, typed document and report.
    pub maximum_document_bytes: usize,
    /// Missing explanatory facts known at selection time, independent of copy success.
    #[serde(default)]
    pub missing_context: Vec<String>,
}
impl Default for PackSelection {
    fn default() -> Self {
        Self {
            version: Version,
            members: Vec::new(),
            maximum_output_bytes: 256 << 20,
            maximum_document_bytes: 16 << 20,
            missing_context: Vec::new(),
        }
    }
}
/// Explicit live exporter input. Only the exporter opens this path; no offline reader does.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackFileInput {
    /// Exact previously declared selected member.
    pub member: String,
    /// Export supplier path, never stored as an offline resolution directive.
    pub path: PathBuf,
    /// Producer-acknowledged immutable byte boundary; unacknowledged tails are excluded.
    pub readable_prefix_bytes: u64,
}
/// Data-only requested publication with explicitly supplied live file prefixes.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackExportRequest {
    /// Current export input codec.
    pub version: Version<2>,
    /// Complete selected scope and independent finite allowances.
    pub selection: PackSelection,
    /// Exact explicitly authorized source prefixes, never discovered from embedded paths.
    #[serde(default)]
    pub files: Vec<PackFileInput>,
    /// Requests to actual protected scientific owners; metadata is derived, never asserted here.
    #[serde(default)]
    pub scientific_results: Vec<crate::workflow::ReviewScientificSelection>,
    /// Existing selected registry diagnostic tables.
    #[serde(default)]
    pub tables: Vec<PackTableInput>,
}
impl PackExportRequest {
    /// Declare selected diagnostic tables from the single registry interpretation owner.
    pub(crate) fn declare_tables(&mut self) -> Result<(), crate::workflow::WorkflowError> {
        if self.tables.len() > 1024 {
            return Err(crate::workflow::WorkflowError::Input(
                "review selects more than 1024 diagnostic tables".into(),
            ));
        }
        for table in &self.tables {
            if self
                .selection
                .members
                .iter()
                .any(|member| member.id == table.member)
            {
                return Err(crate::workflow::WorkflowError::Input(
                    "diagnostic table member is declared twice".into(),
                ));
            }
            let diagnostic = PackDiagnosticTable::new(table.request.clone())
                .map_err(|error| crate::workflow::WorkflowError::Input(error.to_string()))?;
            self.selection.members.push(PackMemberSelection {
                id: table.member.clone(),
                role: PackMemberRole::Query,
                required: true,
                format: PackFormat::ArrowIpcStream,
                document_version: None,
                source: None,
                references: vec![diagnostic.source.clone()],
                result: None,
                capture_scope: None,
                diagnostic_table: Some(diagnostic),
            });
        }
        Ok(())
    }
}
/// Export completion and member readability are separate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum PackAvailability {
    /// No selected exporter has resolved this member yet.
    Pending,
    /// The selected byte boundary finalized and was published.
    Available,
    /// Retention or optional capability is known unavailable without a copy attempt.
    Unavailable {
        /// Original observed absence or capability limitation.
        reason: String,
    },
    /// A selected copy/export failed; acknowledged local prefixes may remain readable.
    Failed {
        /// Actual export error, never substituted for a scientific outcome.
        reason: String,
    },
}
/// The indexed read boundary of one selected member.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackMember {
    /// Original explicit selection and its finite allowances.
    pub selection: PackMemberSelection,
    /// Generated contained path of the indexed local member.
    pub relative_path: String,
    /// Independent selected export availability.
    pub availability: PackAvailability,
    /// Acknowledged readable byte prefix. Never inferred from an original moving file.
    pub readable_bytes: u64,
    /// Whether the selected member writer completed and drained.
    pub finalized: bool,
}
/// Publication state, independent from scientific and capture state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PackPublication {
    /// Selected export has not acquired a complete seal.
    Incomplete,
    /// Selected publication finalized; evidence and science retain independent states.
    Sealed,
}
/// Sole authoritative manifest. The final sealed index is published last.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackIndex {
    /// Supported version of this document contract.
    pub version: Version<2>,
    /// Original explicit selection and its finite allowances.
    pub selection: PackSelection,
    /// Independent enclosing publication completion state.
    pub publication: PackPublication,
    /// Complete selected membership; no first-page completeness inference.
    pub members: Vec<PackMember>,
    /// Every declared selected member is available at its indexed boundary.
    pub membership_complete: bool,
    /// All required members and selected explanatory premises are available.
    pub self_contained: bool,
    /// Known missing explanatory premises; distinct from transport success.
    pub missing_context: Vec<String>,
    /// Actual publication errors retained independently of scientific outcomes.
    pub errors: Vec<String>,
}
/// A failed export leaves its owned directory and incomplete manifest for inspection.
#[derive(Debug, thiserror::Error)]
pub enum PackError {
    /// Selected local source or destination I/O failed.
    #[error("offline review pack I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Selection, byte extent, codec or contained-path contract was violated.
    #[error("offline review pack contract: {0}")]
    Contract(String),
}
pse_diagnostics::impl_diagnostic! {
    PackError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> PackError {
    PackError::Contract(message.into())
}
fn encoded<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>, PackError> {
    let mut writer = LimitedVec {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut writer, value).map_err(|error| invalid(error.to_string()))?;
    Ok(writer.bytes)
}
struct LimitedVec {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for LimitedVec {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("pack document allowance exhausted"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
fn validate_selection(selection: &PackSelection) -> Result<(), PackError> {
    if selection.maximum_output_bytes == 0
        || selection.maximum_output_bytes > (1_u64 << 40)
        || selection.maximum_document_bytes == 0
        || selection.maximum_document_bytes > MAXIMUM_PACK_DOCUMENT_BYTES
        || selection.members.len() > MAXIMUM_MEMBERS
    {
        return Err(invalid(
            "finite pack output/document/member allowances required",
        ));
    }
    let mut identities = BTreeSet::new();
    let mut scientific = BTreeSet::new();
    for member in &selection.members {
        if !valid_id(&member.id) || !identities.insert(&member.id) {
            return Err(invalid("unsafe or duplicate selected member identity"));
        }
        let reserved = match member.id.as_str() {
            "frozen" => Some((PackMemberRole::FrozenEvidence, PackFormat::FrozenEvidenceV3)),
            "analysis" => Some((PackMemberRole::Analysis, PackFormat::AnalysisV2)),
            "report_markdown" => Some((PackMemberRole::Report, PackFormat::Markdown)),
            "report_html" => Some((PackMemberRole::Report, PackFormat::Html)),
            _ => None,
        };
        if let Some((role, format)) = reserved {
            if member.role != role
                || member.format != format
                || !member.required
                || (member.id == "frozen" && member.document_version != Some(3))
                || (member.id == "analysis" && member.document_version != Some(2))
            {
                return Err(invalid("reserved typed member contract mismatch"));
            }
        } else if matches!(
            member.role,
            PackMemberRole::FrozenEvidence | PackMemberRole::Analysis | PackMemberRole::Report
        ) {
            return Err(invalid(
                "typed document/report requires its reserved identity",
            ));
        }

        if let Some(version) = member.document_version {
            let supported: Option<&[u32]> = match (member.role, member.format) {
                (PackMemberRole::FrozenEvidence, PackFormat::FrozenEvidenceV3) => Some(&[3]),
                (PackMemberRole::Analysis, PackFormat::AnalysisV2) => Some(&[2]),
                (PackMemberRole::Context, PackFormat::Json) => Some(&[1]),
                (PackMemberRole::Capture | PackMemberRole::Detail, PackFormat::Json) => Some(&[2]),
                (PackMemberRole::Detail, PackFormat::JsonLines) => Some(&[2]),
                // Retained query documents use v1; current investigation export
                // uses v2. The declared codec must still match the actual bytes.
                (PackMemberRole::Query, PackFormat::Json) => Some(&[1, 2]),
                _ => None,
            };
            if version == 0 || supported.is_some_and(|supported| !supported.contains(&version)) {
                return Err(invalid("unsupported declared selected document version"));
            }
        }
        if (member.role == PackMemberRole::Result) != member.result.is_some()
            || (member.role == PackMemberRole::Result
                && member.format != PackFormat::ArrowIpcStream)
        {
            return Err(invalid("result member requires typed IPC stream selection"));
        }
        if let Some(table) = &member.diagnostic_table
            && (member.result.is_some()
                || member.format != PackFormat::ArrowIpcStream
                || !matches!(
                    member.role,
                    PackMemberRole::Detail | PackMemberRole::Engine | PackMemberRole::Query
                )
                || table.source.member != "frozen"
                || table.source.pointer
                    != crate::tooling::review_query::table_source(&table.request)
                || table.relation != crate::tooling::review_query::table_relation(&table.request))
        {
            return Err(invalid(
                "diagnostic table selection differs from registry/source contract",
            ));
        }
        if let Some(result) = &member.result {
            let identity = encoded(
                &(
                    &result.run,
                    &result.attempt,
                    &result.relation,
                    result.schema_version,
                    result.row_start,
                    result.row_end,
                    &result.field,
                    &result.predicate,
                ),
                selection.maximum_document_bytes,
            )?;
            if !scientific.insert(identity) {
                return Err(invalid(
                    "duplicate or conflicting scientific selection identity",
                ));
            }
        }
        if let Some(result) = &member.result
            && (result.schema_version == 0
                || result.row_start > result.row_end
                || result.run.is_empty()
                || result.attempt.is_empty()
                || result.relation.is_empty())
        {
            return Err(invalid("invalid selected scientific coordinates"));
        }
        if matches!(member.format, PackFormat::FrozenEvidenceV3)
            && member.role != PackMemberRole::FrozenEvidence
            || matches!(member.format, PackFormat::AnalysisV2)
                && member.role != PackMemberRole::Analysis
        {
            return Err(invalid("role and document format mismatch"));
        }
    }
    let reserved_count = ["frozen", "analysis", "report_markdown", "report_html"]
        .iter()
        .filter(|id| identities.iter().any(|member| member.as_str() == **id))
        .count();
    if reserved_count != 0 && reserved_count != 4 {
        return Err(invalid(
            "pack lacks its reserved typed document/report membership",
        ));
    }
    if reserved_count == 0
        && selection.members.iter().any(|member| {
            member.result.is_some()
                || member.diagnostic_table.is_some()
                || matches!(
                    member.role,
                    PackMemberRole::Result
                        | PackMemberRole::FrozenEvidence
                        | PackMemberRole::Analysis
                        | PackMemberRole::Report
                )
        })
    {
        return Err(invalid(
            "supplied-member pack cannot assert scientific result or analysis authority",
        ));
    }
    for member in &selection.members {
        for reference in &member.references {
            if !identities.contains(&reference.member)
                || !(reference.pointer.is_empty() || reference.pointer.starts_with('/'))
            {
                return Err(invalid(
                    "source reference lacks selected member namespace or pointer",
                ));
            }
        }
    }
    Ok(())
}
fn built_in(id: &str, role: PackMemberRole, format: PackFormat) -> PackMemberSelection {
    PackMemberSelection {
        id: id.into(),
        role,
        required: true,
        format,
        document_version: match format {
            PackFormat::FrozenEvidenceV3 => Some(3),
            PackFormat::AnalysisV2 => Some(2),
            _ => None,
        },
        source: None,
        references: Vec::new(),
        result: None,
        capture_scope: None,
        diagnostic_table: None,
    }
}

// Select typed sources before encoding/comparison. The source's document allowance is
// independent of the Arrow projection pool; neither bounds all private decoder allocations.
fn same_table_source(
    retained: &FrozenEvidenceSet,
    supplied: &FrozenEvidenceSet,
    request: &crate::tooling::review_query::ReviewTableRequest,
    maximum_document_bytes: usize,
) -> Result<bool, PackError> {
    fn same<T: Serialize>(
        left: Option<&T>,
        right: Option<&T>,
        limit: usize,
    ) -> Result<bool, PackError> {
        let (Some(left), Some(right)) = (left, right) else {
            return Ok(false);
        };
        // Preserve JSON-value equality (including object ordering independence) while
        // bounding each selected encoding before it can grow a JSON value tree.
        let left: serde_json::Value = serde_json::from_slice(&encoded(left, limit)?)
            .map_err(|error| invalid(error.to_string()))?;
        let right: serde_json::Value = serde_json::from_slice(&encoded(right, limit)?)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(left == right)
    }
    use crate::tooling::review_query::ReviewTableKind;
    let index = request.index;
    match request.kind {
        ReviewTableKind::Detail => same(
            retained.detail_pages.get(index),
            supplied.detail_pages.get(index),
            maximum_document_bytes,
        ),
        ReviewTableKind::Invocation => same(
            retained.invocations.get(index),
            supplied.invocations.get(index),
            maximum_document_bytes,
        ),
        ReviewTableKind::Capture => same(
            retained.captures.get(index),
            supplied.captures.get(index),
            maximum_document_bytes,
        ),
        ReviewTableKind::Engine => same(
            retained.engine_observations.get(index),
            supplied.engine_observations.get(index),
            maximum_document_bytes,
        ),
        ReviewTableKind::Query => same(
            retained.query_results.get(index),
            supplied.query_results.get(index),
            maximum_document_bytes,
        ),
    }
}

/// Owns only private copied output, never scientific work or capture leases.
#[derive(Debug)]
pub struct PackSession {
    root: PathBuf,
    index: PackIndex,
    cancel: CancellationToken,
    written: u64,
    next_index: u64,
    stopped: bool,
}
impl PackSession {
    /// Exclusive directory creation refuses any existing destination, including incomplete output.
    pub fn create(
        destination: &Path,
        selection: PackSelection,
        cancel: CancellationToken,
    ) -> Result<Self, PackError> {
        Self::create_selected(destination, selection, cancel, true)
    }
    /// Copy explicitly supplied non-scientific members without inventing a scientific analysis.
    pub fn create_members(
        destination: &Path,
        selection: PackSelection,
        cancel: CancellationToken,
    ) -> Result<Self, PackError> {
        if selection.members.is_empty()
            || selection.members.iter().any(|member| {
                !matches!(
                    member.role,
                    PackMemberRole::Query
                        | PackMemberRole::Context
                        | PackMemberRole::Capture
                        | PackMemberRole::Detail
                        | PackMemberRole::Engine
                )
            })
        {
            return Err(invalid(
                "supplied-member pack requires explicit non-scientific membership",
            ));
        }
        Self::create_selected(destination, selection, cancel, false)
    }
    fn create_selected(
        destination: &Path,
        mut selection: PackSelection,
        cancel: CancellationToken,
        scientific: bool,
    ) -> Result<Self, PackError> {
        for reserved in [
            built_in(
                "frozen",
                PackMemberRole::FrozenEvidence,
                PackFormat::FrozenEvidenceV3,
            ),
            built_in("analysis", PackMemberRole::Analysis, PackFormat::AnalysisV2),
            built_in(
                "report_markdown",
                PackMemberRole::Report,
                PackFormat::Markdown,
            ),
            built_in("report_html", PackMemberRole::Report, PackFormat::Html),
        ] {
            if selection.members.iter().any(|m| m.id == reserved.id) {
                return Err(invalid("reserved pack member identity"));
            }
            if scientific {
                selection.members.push(reserved);
            }
        }
        validate_selection(&selection)?;
        encoded(&selection, selection.maximum_document_bytes)?;
        let members = selection
            .members
            .iter()
            .enumerate()
            .map(|(i, member)| PackMember {
                selection: member.clone(),
                relative_path: format!(
                    "members/{i:05}-{}.{}",
                    member.id,
                    member.format.extension()
                ),
                availability: PackAvailability::Pending,
                readable_bytes: 0,
                finalized: false,
            })
            .collect();
        let missing_context = selection.missing_context.clone();
        let index = PackIndex {
            version: Version,
            selection,
            publication: PackPublication::Incomplete,
            members,
            membership_complete: false,
            self_contained: false,
            missing_context,
            errors: Vec::new(),
        };
        encoded(&index, index.selection.maximum_document_bytes)?;
        // Directory creation itself is the local no-overwrite boundary.
        std::fs::create_dir(destination)?;
        let root = destination.canonicalize()?;
        std::fs::create_dir(root.join("members"))?;
        std::fs::create_dir(root.join(".staging"))?;
        let mut session = Self {
            root,
            index,
            cancel,
            written: 0,
            next_index: 0,
            stopped: false,
        };
        session.persist()?;
        Ok(session)
    }
    /// Current authoritative membership and independent completion states.
    pub fn index(&self) -> &PackIndex {
        &self.index
    }
    fn persist(&mut self) -> Result<(), PackError> {
        let result = self.persist_index();
        if result.is_err() {
            self.stopped = true;
        }
        result
    }
    fn persist_index(&mut self) -> Result<(), PackError> {
        let bytes = encoded(&self.index, self.index.selection.maximum_document_bytes)?;
        let temporary = self.root.join(format!(".index-{}", self.next_index));
        self.next_index += 1;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        if self.index.publication == PackPublication::Sealed {
            self.checkpoint()?;
        }
        std::fs::rename(temporary, self.root.join(INDEX))?;
        Ok(())
    }
    fn position(&self, id: &str) -> Result<usize, PackError> {
        let pos = self
            .index
            .members
            .iter()
            .position(|m| m.selection.id == id)
            .ok_or_else(|| invalid("member was not selected"))?;
        if self.index.members[pos].availability != PackAvailability::Pending {
            return Err(invalid("member already resolved"));
        }
        Ok(pos)
    }
    fn checkpoint(&self) -> Result<(), PackError> {
        if self.stopped {
            return Err(invalid("pack publication already stopped"));
        }
        self.cancel.checkpoint().map_err(|e| invalid(e.to_string()))
    }
    fn fail(&mut self, pos: usize, error: &PackError) {
        self.fail_at(pos, error, None);
    }
    fn fail_at(&mut self, pos: usize, error: &PackError, acknowledged: Option<u64>) {
        let member = &mut self.index.members[pos];
        let already_accounted = member.finalized;
        member.availability = PackAvailability::Failed {
            reason: error.to_string(),
        };
        member.finalized = false;
        // Acknowledged bytes already copied by our own writer can remain a readable
        // prefix, even though neither this member nor the enclosing pack completed.
        let destination = self.root.join(&member.relative_path);
        for suffix in ["", ".incomplete"] {
            let staging = self
                .root
                .join(".staging")
                .join(format!("{}{suffix}", member.selection.id));
            if staging.is_file() && !destination.exists() {
                let _ = std::fs::hard_link(&staging, &destination);
            }
        }
        let actual = std::fs::metadata(&destination).map_or(0, |m| m.len());
        member.readable_bytes = acknowledged.map_or(actual, |prefix| prefix.min(actual));
        if !already_accounted {
            self.written = self.written.saturating_add(actual);
        }
        self.index.errors.push(error.to_string());
        self.stopped |= member.selection.required
            || self.cancel.checkpoint().is_err()
            || error.to_string().contains("allowance");
        let _ = self.persist();
    }
    fn finalize_member(&mut self, pos: usize, staging: &Path, bytes: u64) -> Result<(), PackError> {
        self.checkpoint()?;
        let destination = self.root.join(&self.index.members[pos].relative_path);
        std::fs::hard_link(staging, &destination)?;
        std::fs::remove_file(staging)?;
        self.written = self
            .written
            .checked_add(bytes)
            .ok_or_else(|| invalid("output byte overflow"))?;
        let member = &mut self.index.members[pos];
        member.readable_bytes = bytes;
        member.finalized = true;
        member.availability = PackAvailability::Available;
        validate_encoding(
            member,
            File::open(destination)?,
            self.index.selection.maximum_document_bytes,
        )?;
        self.persist()
    }
    /// Retain an acquisition failure which occurred before the selected exporter opened.
    pub fn fail_member(&mut self, id: &str, reason: String) -> Result<(), PackError> {
        let pos = self.position(id)?;
        self.fail(pos, &invalid(reason));
        self.persist()
    }
    /// Known missing retention may be declared unavailable, including required context.
    /// It decreases completeness/self-containment without inventing a failed filesystem write.
    pub fn unavailable(&mut self, id: &str, reason: String) -> Result<(), PackError> {
        self.checkpoint()?;
        let pos = self.position(id)?;
        if self.index.members[pos].selection.required
            && self.index.members[pos].selection.role != PackMemberRole::Context
        {
            return Err(invalid(
                "required transport/report/result cannot be declared unavailable",
            ));
        }
        if self.index.members[pos].selection.role == PackMemberRole::Context
            && self.index.members[pos].selection.required
        {
            self.index.missing_context.push(reason.clone());
        }
        self.index.members[pos].availability = PackAvailability::Unavailable { reason };
        self.persist()
    }
    /// Export the declared files, permitting optional capability/copy failures while
    /// stopping immediately for cancellation, exhausted output or required writes.
    pub fn copy_files(&mut self, inputs: &[PackFileInput]) -> Result<(), PackError> {
        let validation = (|| {
            self.checkpoint()?;
            if inputs.len() > MAXIMUM_MEMBERS {
                return Err(invalid(
                    "supplied file count exceeds selected finite membership",
                ));
            }
            let mut identities = BTreeSet::new();
            for input in inputs {
                self.position(&input.member)?;
                if !identities.insert(&input.member) {
                    return Err(invalid("duplicate supplied file identity"));
                }
            }
            Ok(())
        })();
        if let Err(error) = validation {
            self.stopped = true;
            self.index.errors.push(error.to_string());
            let _ = self.persist();
            return Err(error);
        }
        for input in inputs {
            if let Err(error) = self.copy_file(input)
                && self.stopped
            {
                return Err(error);
            }
        }
        Ok(())
    }
    /// Export only explicitly supplied prefix inputs. Source opening failures are indexed.
    pub fn copy_file(&mut self, input: &PackFileInput) -> Result<(), PackError> {
        let pos = self.position(&input.member)?;
        let file = match File::open(&input.path) {
            Ok(file) => file,
            Err(source) => {
                let error = PackError::Io(source);
                self.fail(pos, &error);
                return Err(error);
            }
        };
        self.copy_prefix(&input.member, file, input.readable_prefix_bytes)
    }
    /// Copy exactly an acknowledged immutable prefix from an explicitly supplied open file.
    /// Opening/pinning the original is the live supplier's responsibility.
    pub fn copy_prefix(
        &mut self,
        id: &str,
        mut source: File,
        readable_bytes: u64,
    ) -> Result<(), PackError> {
        let pos = self.position(id)?;
        let result = (|| {
            self.checkpoint()?;
            if matches!(
                self.index.members[pos].selection.role,
                PackMemberRole::Result
                    | PackMemberRole::FrozenEvidence
                    | PackMemberRole::Analysis
                    | PackMemberRole::Report
            ) {
                return Err(invalid("typed selected member requires its owner exporter"));
            }
            if readable_bytes
                > self
                    .index
                    .selection
                    .maximum_output_bytes
                    .saturating_sub(self.written)
            {
                return Err(invalid("pack output allowance exhausted"));
            }
            let staging = self.root.join(".staging").join(id);
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging)?;
            source.seek(SeekFrom::Start(0))?;
            let mut remaining = readable_bytes;
            let mut buffer = [0_u8; 64 * 1024];
            while remaining > 0 {
                self.checkpoint()?;
                let requested = remaining.min(buffer.len() as u64) as usize;
                let read = source.read(&mut buffer[..requested])?;
                if read == 0 {
                    return Err(invalid("source shorter than acknowledged readable prefix"));
                }
                output.write_all(&buffer[..read])?;
                remaining -= read as u64;
            }
            output.flush()?;
            output.sync_all()?;
            drop(output);
            self.finalize_member(pos, &staging, readable_bytes)
        })();
        if let Err(error) = &result {
            self.fail(pos, error);
        }
        result
    }
    fn write_typed<T: Serialize>(&mut self, id: &str, value: &T) -> Result<(), PackError> {
        let bytes = match encoded(value, self.index.selection.maximum_document_bytes) {
            Ok(bytes) => bytes,
            Err(error) => {
                let pos = self.position(id)?;
                self.fail(pos, &error);
                return Err(error);
            }
        };
        self.write_bytes(id, &bytes)
    }
    fn write_bytes(&mut self, id: &str, bytes: &[u8]) -> Result<(), PackError> {
        let pos = self.position(id)?;
        let result = (|| {
            self.checkpoint()?;
            if bytes.len() as u64
                > self
                    .index
                    .selection
                    .maximum_output_bytes
                    .saturating_sub(self.written)
            {
                return Err(invalid("pack output allowance exhausted"));
            }
            let staging = self.root.join(".staging").join(id);
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging)?;
            file.write_all(bytes)?;
            file.flush()?;
            file.sync_all()?;
            drop(file);
            self.finalize_member(pos, &staging, bytes.len() as u64)
        })();
        if let Err(error) = &result {
            self.fail(pos, error);
        }
        result
    }
    /// Validate and export only the explicitly supplied frozen facts.
    pub fn add_frozen(&mut self, evidence: &FrozenEvidenceSet) -> Result<(), PackError> {
        self.position("frozen")?;
        evidence.validate().map_err(|e| invalid(e.to_string()))?;
        self.write_typed("frozen", evidence)
    }
    /// The same typed shared facts drive JSON, Markdown and local HTML.
    pub fn add_analysis(&mut self, analysis: &AnalysisDocument) -> Result<(), PackError> {
        let pos = self.position("analysis")?;
        let validation = (|| {
            analysis.validate().map_err(|e| invalid(e.to_string()))?;
            let frozen = self
                .index
                .members
                .iter()
                .find(|m| {
                    m.selection.id == "frozen" && m.availability == PackAvailability::Available
                })
                .ok_or_else(|| invalid("frozen facts must finalize before shared analysis"))?;
            let frozen: serde_json::Value = decode(
                File::open(self.root.join(&frozen.relative_path))?,
                self.index.selection.maximum_document_bytes,
            )?;
            validate_analysis_sources(analysis, &self.index, Some(&frozen))
        })();
        if let Err(error) = validation {
            self.fail(pos, &error);
            return Err(error);
        }
        self.write_typed("analysis", analysis)?;
        Ok(())
    }
    fn finalize_reports(&mut self) -> Result<(), PackError> {
        let directory = File::open(&self.root)?;
        let member = self
            .index
            .members
            .iter()
            .find(|m| m.selection.id == "analysis" && m.availability == PackAvailability::Available)
            .ok_or_else(|| invalid("typed analysis missing before report publication"))?;
        let analysis: AnalysisDocument = decode(
            open_contained(&directory, &member.relative_path)?.take(member.readable_bytes),
            self.index.selection.maximum_document_bytes,
        )?;
        let sources = self
            .index
            .members
            .iter()
            .map(|member| {
                let filename = Path::new(&member.relative_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| invalid("invalid generated member filename"))?;
                Ok((member.selection.id.clone(), filename.to_owned()))
            })
            .collect::<Result<std::collections::BTreeMap<_, _>, PackError>>()?;
        let orientation = presentation::orientation(&directory, &self.index)?;
        let (markdown, html) = crate::tooling::review_analysis::render::render_reports(
            &analysis,
            &sources,
            &orientation,
            self.index.selection.maximum_document_bytes,
        );
        if markdown.len() > self.index.selection.maximum_document_bytes
            || html.len() > self.index.selection.maximum_document_bytes
        {
            return Err(invalid("report document allowance exhausted"));
        }
        self.write_bytes("report_markdown", markdown.as_bytes())?;
        self.write_bytes("report_html", html.as_bytes())
    }
    /// Publish a bounded existing registry projection over the supplied immutable set.
    pub fn export_table(
        &mut self,
        id: &str,
        set: &FrozenEvidenceSet,
        request: &crate::tooling::review_query::ReviewTableRequest,
    ) -> Result<(), PackError> {
        let pos = self.position(id)?;
        let result = (|| {
            self.checkpoint()?;
            let descriptor = self.index.members[pos]
                .selection
                .diagnostic_table
                .as_ref()
                .ok_or_else(|| invalid("diagnostic projection lacks typed descriptor"))?;
            if encoded(&descriptor.request, 4096)? != encoded(request, 4096)? {
                return Err(invalid(
                    "diagnostic request differs from selected descriptor",
                ));
            }
            let supplied = self
                .index
                .members
                .iter()
                .find(|m| {
                    m.selection.id == "frozen" && m.availability == PackAvailability::Available
                })
                .ok_or_else(|| {
                    invalid("frozen facts must finalize before diagnostic projection")
                })?;
            // The finalized frozen document has its own finite ingress allowance. The
            // requested table pool covers its Arrow projection, not this typed decode.
            let retained: FrozenEvidenceSet = decode(
                File::open(self.root.join(&supplied.relative_path))?.take(supplied.readable_bytes),
                self.index.selection.maximum_document_bytes,
            )?;
            if !same_table_source(
                &retained,
                set,
                request,
                self.index.selection.maximum_document_bytes,
            )? {
                return Err(invalid(
                    "diagnostic table source differs from finalized frozen member",
                ));
            }
            let batch = crate::tooling::review_query::table_arrow(&retained, request, &self.cancel)
                .map_err(|e| invalid(e.to_string()))?;
            let staging = self.root.join(".staging").join(id);
            let maximum = self
                .index
                .selection
                .maximum_output_bytes
                .saturating_sub(self.written);
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging)?;
            {
                let mut bounded = BoundedOutput {
                    output: &mut file,
                    remaining: maximum,
                };
                let mut writer = datafusion::arrow::ipc::writer::StreamWriter::try_new(
                    &mut bounded,
                    batch.schema().as_ref(),
                )
                .map_err(|e| invalid(e.to_string()))?;
                writer.write(&batch).map_err(|e| invalid(e.to_string()))?;
                writer.finish().map_err(|e| invalid(e.to_string()))?;
            }
            file.flush()?;
            file.sync_all()?;
            let bytes = file.metadata()?.len();
            drop(file);
            self.finalize_member(pos, &staging, bytes)
        })();
        if let Err(error) = &result {
            self.fail(pos, error);
        }
        result
    }
    /// Consume the actual protected canonical reader; its borrowers drop before return.
    pub async fn export_result(
        &mut self,
        id: &str,
        mut reader: crate::workflow::CanonicalResultReader,
    ) -> Result<(), PackError> {
        let pos = self.position(id)?;
        let mut acknowledged = 0;
        let result = (async {
            self.checkpoint()?;
            reader
                .validate_pack_selection(
                    self.index.members[pos]
                        .selection
                        .result
                        .as_ref()
                        .ok_or_else(|| invalid("not a result selection"))?,
                )
                .map_err(|e| invalid(e.to_string()))?;
            let staging = self.root.join(".staging").join(id);
            let bytes = reader
                .export_ipc_acknowledged(
                    &staging,
                    self.index
                        .selection
                        .maximum_output_bytes
                        .saturating_sub(self.written),
                    &self.cancel,
                    &mut acknowledged,
                )
                .await
                .map_err(|e| invalid(e.to_string()))?;
            self.finalize_member(pos, &staging, bytes)
        })
        .await;
        drop(reader);
        if let Err(error) = &result {
            self.fail_at(pos, error, Some(acknowledged));
        }
        result
    }
    /// Output/field/range filtered protected reader; no separate unprotected materialization.
    pub async fn export_selected_result(
        &mut self,
        id: &str,
        mut reader: crate::workflow::SelectedResultReader,
    ) -> Result<(), PackError> {
        let pos = self.position(id)?;
        let mut acknowledged = 0;
        let result = (async {
            self.checkpoint()?;
            reader
                .validate_pack_selection(
                    self.index.members[pos]
                        .selection
                        .result
                        .as_ref()
                        .ok_or_else(|| invalid("not a result selection"))?,
                )
                .map_err(|e| invalid(e.to_string()))?;
            let staging = self.root.join(".staging").join(id);
            let bytes = reader
                .export_ipc_acknowledged(
                    &staging,
                    self.index
                        .selection
                        .maximum_output_bytes
                        .saturating_sub(self.written),
                    &self.cancel,
                    &mut acknowledged,
                )
                .await
                .map_err(|e| invalid(e.to_string()))?;
            self.finalize_member(pos, &staging, bytes)
        })
        .await;
        drop(reader);
        if let Err(error) = &result {
            self.fail_at(pos, error, Some(acknowledged));
        }
        result
    }
    /// Seal only after every selected member resolves and all required writes/reports drain.
    /// Known unavailable context remains explicitly non-self-contained in a valid seal.
    pub fn seal(mut self) -> Result<PackIndex, PackError> {
        if let Err(error) = self.checkpoint() {
            self.index.errors.push(error.to_string());
            let _ = self.persist();
            return Err(error);
        }
        let directory = File::open(&self.root)?;
        let missing = references::validate(&directory, &self.index)?;
        self.index.missing_context.extend(missing);
        self.index.missing_context.sort();
        self.index.missing_context.dedup();
        if self.index.members.iter().any(|m| {
            (m.availability == PackAvailability::Pending
                && m.selection.role != PackMemberRole::Report)
                || (m.selection.required
                    && matches!(m.availability, PackAvailability::Failed { .. }))
        }) {
            return Err(invalid(
                "unresolved or failed required member prevents sealing",
            ));
        }
        if self
            .index
            .members
            .iter()
            .any(|member| member.selection.id == "analysis")
            && let Err(error) = self.finalize_reports()
        {
            self.index.errors.push(error.to_string());
            let _ = self.persist();
            return Err(error);
        }
        self.checkpoint()?;
        self.index.membership_complete = self
            .index
            .members
            .iter()
            .all(|m| m.availability == PackAvailability::Available);
        self.index.self_contained = self.index.missing_context.is_empty()
            && self
                .index
                .members
                .iter()
                .filter(|m| m.selection.required)
                .all(|m| m.availability == PackAvailability::Available);
        self.index.publication = PackPublication::Sealed;
        self.persist()?;
        Ok(self.index)
    }
}
/// Offline-only validated indexed directory. Unindexed neighbors are never examined.
#[derive(Debug)]
pub struct PackReader {
    directory: File,
    index: PackIndex,
    maximum_document_bytes: usize,
}
impl PackReader {
    /// Validate the supported index and contained supplied files without any live fallback.
    pub fn open(path: &Path, maximum_document_bytes: usize) -> Result<Self, PackError> {
        if maximum_document_bytes == 0 || maximum_document_bytes > MAXIMUM_PACK_DOCUMENT_BYTES {
            return Err(invalid("finite reader document allowance required"));
        }
        let root = path.canonicalize()?;
        let directory = File::from(
            rustix::fs::open(
                &root,
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::DIRECTORY
                    | rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::empty(),
            )
            .map_err(std::io::Error::from)?,
        );
        let index: PackIndex = decode(open_contained(&directory, INDEX)?, maximum_document_bytes)?;
        validate_selection(&index.selection)?;
        if index.members.len() != index.selection.members.len() {
            return Err(invalid("index membership differs from selected identities"));
        }
        let selected_by_id: std::collections::BTreeMap<_, _> = index
            .selection
            .members
            .iter()
            .map(|m| (m.id.as_str(), m))
            .collect();
        let mut identities = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for member in &index.members {
            if !identities.insert(&member.selection.id) || !paths.insert(&member.relative_path) {
                return Err(invalid("duplicate indexed identity or path"));
            }
            let selected = selected_by_id
                .get(member.selection.id.as_str())
                .ok_or_else(|| invalid("unselected indexed identity"))?;
            if encoded(selected, maximum_document_bytes)?
                != encoded(&member.selection, maximum_document_bytes)?
            {
                return Err(invalid("indexed selection identity conflict"));
            }
            safe_relative(&member.relative_path)?;
            if !member.relative_path.starts_with("members/") {
                return Err(invalid("member path outside indexed member directory"));
            }
            if member.availability == PackAvailability::Available {
                if !member.finalized {
                    return Err(invalid("available member lacks finalization"));
                }
                let file = open_contained(&directory, &member.relative_path)?;
                let metadata = file.metadata()?;
                if !metadata.is_file()
                    || metadata.len() < member.readable_bytes
                    || (index.publication == PackPublication::Sealed
                        && metadata.len() != member.readable_bytes)
                {
                    return Err(invalid("indexed byte boundary differs from available file"));
                }
                validate_encoding(member, file, maximum_document_bytes)?;
            } else {
                if member.finalized {
                    return Err(invalid("unavailable member advertises finalization"));
                }
                if member.readable_bytes != 0 {
                    if !matches!(member.availability, PackAvailability::Failed { .. }) {
                        return Err(invalid("unavailable member advertises readable bytes"));
                    }
                    let metadata = open_contained(&directory, &member.relative_path)?.metadata()?;
                    if !metadata.is_file() || metadata.len() < member.readable_bytes {
                        return Err(invalid("failed member readable prefix is absent"));
                    }
                }
            }
        }
        if index.publication == PackPublication::Sealed
            && index.members.iter().any(|m| {
                m.availability == PackAvailability::Pending
                    || (m.selection.required
                        && matches!(m.availability, PackAvailability::Failed { .. }))
            })
        {
            return Err(invalid(
                "sealed index has unresolved or failed required members",
            ));
        }
        if index
            .members
            .iter()
            .try_fold(0_u64, |sum, member| sum.checked_add(member.readable_bytes))
            .is_none_or(|sum| sum > index.selection.maximum_output_bytes)
        {
            return Err(invalid("indexed payload exceeds declared output allowance"));
        }
        let complete = index
            .members
            .iter()
            .all(|m| m.availability == PackAvailability::Available);
        let self_contained = index.missing_context.is_empty()
            && index
                .members
                .iter()
                .filter(|m| m.selection.required)
                .all(|m| m.availability == PackAvailability::Available);
        if index.publication == PackPublication::Sealed
            && (index.membership_complete != complete || index.self_contained != self_contained)
        {
            return Err(invalid("seal completeness claims contradict indexed scope"));
        }
        let reader = Self {
            directory,
            index,
            maximum_document_bytes,
        };
        // Typed codecs reject unsupported nested versions without source-path fallback.
        let frozen = if reader.available("frozen") {
            Some(reader.read_frozen()?)
        } else {
            None
        };
        let missing = references::validate(&reader.directory, &reader.index)?;
        if reader.index.publication == PackPublication::Sealed
            && reader.index.self_contained
            && !missing.is_empty()
        {
            return Err(invalid(
                "self-contained claim lacks selected question prerequisites or references",
            ));
        }
        if reader.available("analysis") {
            let analysis = reader.read_analysis()?;
            let frozen_json = frozen
                .as_ref()
                .map(|f| serde_json::to_value(f).map_err(|e| invalid(e.to_string())))
                .transpose()?;
            validate_analysis_sources(&analysis, &reader.index, frozen_json.as_ref())?;
        }
        Ok(reader)
    }
    /// Current authoritative membership and independent completion states.
    pub fn index(&self) -> &PackIndex {
        &self.index
    }
    fn available(&self, id: &str) -> bool {
        self.index
            .members
            .iter()
            .any(|m| m.selection.id == id && m.availability == PackAvailability::Available)
    }
    /// Opens only the indexed acknowledged prefix. Descriptive paths are never followed.
    pub fn open_member(&self, id: &str) -> Result<std::io::Take<File>, PackError> {
        let member = self
            .index
            .members
            .iter()
            .find(|m| m.selection.id == id)
            .ok_or_else(|| invalid("unindexed member"))?;
        if member.availability != PackAvailability::Available
            && !(matches!(member.availability, PackAvailability::Failed { .. })
                && member.readable_bytes > 0)
        {
            return Err(invalid("selected member unavailable"));
        }
        let file = open_contained(&self.directory, &member.relative_path)?;
        if file.metadata()?.len() < member.readable_bytes {
            return Err(invalid("indexed file shrank below acknowledged prefix"));
        }
        Ok(file.take(member.readable_bytes))
    }
    /// Decode the indexed frozen set under the finite reader allowance.
    pub fn read_frozen(&self) -> Result<FrozenEvidenceSet, PackError> {
        let evidence: FrozenEvidenceSet =
            decode(self.open_member("frozen")?, self.maximum_document_bytes)?;
        evidence.validate().map_err(|e| invalid(e.to_string()))?;
        Ok(evidence)
    }
    /// Decode the shared typed analysis; never executes its next observations.
    pub fn read_analysis(&self) -> Result<AnalysisDocument, PackError> {
        let analysis: AnalysisDocument =
            decode(self.open_member("analysis")?, self.maximum_document_bytes)?;
        analysis.validate().map_err(|e| invalid(e.to_string()))?;
        Ok(analysis)
    }
}
fn missing_result_context(evidence: &FrozenEvidenceSet, index: &PackIndex) -> Vec<String> {
    // Build each retained association once. A large result membership never
    // repeats scans or decodes of every invocation/context per table member.
    let mut by_native = std::collections::BTreeMap::new();
    for invocation in &evidence.invocations {
        by_native
            .entry((invocation.run_id, invocation.attempt_id))
            .or_insert_with(Vec::new)
            .push(invocation);
    }
    let mut associated: std::collections::BTreeMap<String, BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for context in &evidence.contexts {
        let context_closed = [
            crate::tooling::review_analysis::requirements::RequiredFact::Model,
            crate::tooling::review_analysis::requirements::RequiredFact::Input,
            crate::tooling::review_analysis::requirements::RequiredFact::EffectiveSettings,
            crate::tooling::review_analysis::requirements::RequiredFact::PhysicalCoordinates,
        ]
        .into_iter()
        .all(|fact| crate::tooling::review_analysis::requirements::context_has(context, fact));
        let mut associate = |invocation: &crate::invocation::InvocationEvidenceDocument| {
            if context_closed
                && context.run_id == Some(invocation.run_id)
                && context.attempt_id == invocation.attempt_id
                && (context.operation.is_none() || context.operation == invocation.operation)
                && let (Some(run), Some(attempt)) =
                    (&invocation.canonical_run, &invocation.canonical_attempt)
            {
                associated
                    .entry(run.clone())
                    .or_default()
                    .insert(attempt.clone());
            }
        };
        if let Some(run) = context.run_id
            && let Some(invocations) = by_native.get(&(run, context.attempt_id))
        {
            for invocation in invocations {
                associate(invocation);
            }
        }
        for fact in context
            .facts
            .iter()
            .filter(|fact| fact.codec == "invocation-evidence.v2")
        {
            if let Ok(invocation) = serde_json::from_slice::<
                crate::invocation::InvocationEvidenceDocument,
            >(&fact.payload)
                && invocation.validate().is_ok()
            {
                associate(&invocation);
            }
        }
    }
    index.members.iter().filter_map(|member|member.selection.result.as_ref().filter(|result| !associated.get(&result.run).is_some_and(|attempts|attempts.contains(&result.attempt))).map(|result|format!("member {} lacks original model/input/settings context associated with exact canonical run {} and attempt {}",member.selection.id,result.run,result.attempt))).collect()
}
fn validate_analysis_sources(
    analysis: &AnalysisDocument,
    index: &PackIndex,
    frozen_json: Option<&serde_json::Value>,
) -> Result<(), PackError> {
    for question in &analysis.questions {
        for statement in &question.statements {
            for reference in &statement.evidence {
                if !index
                    .members
                    .iter()
                    .any(|m| m.selection.id == reference.member)
                    || !(reference.pointer.is_empty() || reference.pointer.starts_with('/'))
                {
                    return Err(invalid(
                        "analysis source reference lacks an indexed member/pointer",
                    ));
                }
                if reference.member == "frozen"
                    && frozen_json
                        .and_then(|f| f.pointer(&reference.pointer))
                        .is_none()
                {
                    return Err(invalid(
                        "analysis source pointer is absent from supplied frozen member",
                    ));
                }
                if let Some(event) = &reference.event {
                    if reference.member != "frozen" {
                        return Err(invalid(
                            "event reference must identify supplied frozen event",
                        ));
                    }
                    let frozen = frozen_json
                        .ok_or_else(|| invalid("event reference lacks supplied frozen facts"))?;
                    let mut pointer = reference.pointer.as_str();
                    let mut matched = false;
                    while !pointer.is_empty() {
                        if let Some(record) = frozen.pointer(pointer).filter(|value| {
                            value.get("producer").is_some()
                                && value.get("generation").is_some()
                                && value.get("sequence").is_some()
                        }) {
                            let producer = serde_json::to_value(event.producer)
                                .map_err(|e| invalid(e.to_string()))?;
                            matched = record.get("producer") == Some(&producer)
                                && record.get("generation").and_then(serde_json::Value::as_u64)
                                    == Some(event.generation)
                                && record.get("sequence").and_then(serde_json::Value::as_u64)
                                    == Some(event.sequence)
                                && record
                                    .get("family_sequence")
                                    .and_then(serde_json::Value::as_u64)
                                    == event.family_sequence
                                && event.field_ordinal.is_none_or(|ordinal| {
                                    reference.pointer == format!("{pointer}/values/{ordinal}/value")
                                });
                            break;
                        }
                        pointer = pointer.rsplit_once('/').map_or("", |(parent, _)| parent);
                    }
                    if !matched {
                        return Err(invalid(
                            "event identity differs from source-referenced original record",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
fn contained_error(error: std::io::Error) -> PackError {
    if error.kind() == std::io::ErrorKind::InvalidInput {
        invalid(error.to_string())
    } else {
        PackError::Io(error)
    }
}
fn safe_relative(path: &str) -> Result<(), PackError> {
    crate::workflow::portable_payload::safe_relative(path).map_err(contained_error)
}
fn open_contained(directory: &File, relative: &str) -> Result<File, PackError> {
    crate::workflow::portable_payload::open_contained(directory, relative).map_err(contained_error)
}
fn decode<T: serde::de::DeserializeOwned>(
    input: impl Read,
    maximum: usize,
) -> Result<T, PackError> {
    let mut bytes = Vec::new();
    input.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("reader document allowance exhausted"));
    }
    serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))
}
fn validate_encoding(
    member: &PackMember,
    mut file: File,
    maximum_document_bytes: usize,
) -> Result<(), PackError> {
    if Path::new(&member.relative_path)
        .extension()
        .and_then(|e| e.to_str())
        != Some(member.selection.format.extension())
    {
        return Err(invalid("indexed format/extension mismatch"));
    }
    if matches!(
        member.selection.format,
        PackFormat::Json | PackFormat::FrozenEvidenceV3 | PackFormat::AnalysisV2
    ) && let Some(version) = member.selection.document_version
    {
        let document: serde_json::Value = decode(&mut file, maximum_document_bytes)?;
        if document.get("version").and_then(serde_json::Value::as_u64) != Some(u64::from(version)) {
            return Err(invalid(
                "selected JSON document version differs from declared codec",
            ));
        }
    }
    if member.selection.format == PackFormat::JsonLines
        && member.selection.document_version.is_some()
    {
        let mut header_reader =
            std::io::BufReader::new((&mut file).take(maximum_document_bytes as u64 + 1));
        let mut line = Vec::new();
        std::io::BufRead::read_until(&mut header_reader, b'\n', &mut line)?;
        if line.len() > maximum_document_bytes || !line.ends_with(b"\n") {
            return Err(invalid(
                "selected JSONL header exceeds acknowledged bounded frame",
            ));
        }
        let header: serde_json::Value =
            serde_json::from_slice(&line).map_err(|e| invalid(e.to_string()))?;
        let version = header
            .get("version")
            .or_else(|| header.get("value").and_then(|value| value.get("version")))
            .and_then(serde_json::Value::as_u64);
        if version != member.selection.document_version.map(u64::from)
            || (member.selection.role == PackMemberRole::Detail
                && header.get("kind").and_then(serde_json::Value::as_str) != Some("header"))
        {
            return Err(invalid(
                "selected JSONL header differs from declared original codec",
            ));
        }
        file.seek(SeekFrom::Start(member.readable_bytes.saturating_sub(1)))?;
        let mut last = [0_u8; 1];
        file.read_exact(&mut last)?;
        if last != *b"\n" {
            return Err(invalid(
                "selected JSONL prefix lacks acknowledged final record boundary",
            ));
        }
    }
    if member.selection.format == PackFormat::ArrowIpcStream {
        let schema = crate::workflow::portable_payload::finalized_ipc_schema(
            file, member.readable_bytes, maximum_document_bytes,
        ).map_err(contained_error)?;
        if let Some(table) = &member.selection.diagnostic_table {
            let registry = pse_schema::registry().map_err(|e| invalid(e.to_string()))?;
            let relation = registry
                .relation(&table.relation)
                .ok_or_else(|| invalid("unknown diagnostic registry relation"))?;
            let expected = pse_schema::arrow::relation_schema_ref(registry, relation)
                .map_err(|e| invalid(e.to_string()))?;
            if schema.fields() != expected.fields()
                || schema.metadata() != expected.metadata()
                || expected
                    .metadata()
                    .get(pse_schema::arrow::KEY_CONTRACT_VERSION)
                    != Some(&table.schema_version.to_string())
            {
                return Err(invalid(
                    "diagnostic IPC differs from selected registry schema/version",
                ));
            }
            return Ok(());
        }
        let result = member
            .selection
            .result
            .as_ref()
            .ok_or_else(|| invalid("IPC stream lacks declared scientific lineage"))?;
        crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), result)
            .map_err(contained_error)?;
    }
    Ok(())
}


struct BoundedOutput<'a> {
    output: &'a mut File,
    remaining: u64,
}
impl Write for BoundedOutput<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() as u64 > self.remaining {
            return Err(std::io::Error::other("pack output allowance exhausted"));
        }
        let written = self.output.write(bytes)?;
        self.remaining -= written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}
