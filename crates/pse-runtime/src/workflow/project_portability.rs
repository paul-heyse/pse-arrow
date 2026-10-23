// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical project exchange. Origin evidence never becomes receiving execution authority.
use super::{PhysicalContext, ProjectSelection, Runtime, WorkflowError, contract};
use crate::tooling::contained_copy;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

pub use pse_operations::canonical_project_imports::ProjectImportIntent;

/// Explicit finite selection; omitted payloads are never silently selected by a latest query.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectExportRequest {
    /// Selection codec.
    pub version: Version<1>,
    /// Exact selected source and branch/bookmark owner.
    pub selection: ProjectSelection,
    /// Frozen research selections, including their exact dataset/context ancestry.
    pub research_selections: Vec<String>,
    /// Exact parameter publication ancestry.
    pub publications: Vec<String>,
    /// Exact study definitions; historical terminals remain origin evidence.
    pub studies: Vec<String>,
    /// Requested original result rows/output predicates.
    pub results: Vec<super::ReviewScientificSelection>,
    /// Existing independently owned evidence artifacts to copy under their protected owner.
    pub artifacts: Vec<String>,
    /// Explicit owner-issued diagnostic reproducers; copied inspection never executes them.
    #[serde(default)]
    pub reproducers: Vec<super::DiagnosticReproducer>,
    /// Explicit descriptive-only publication, never executable admission.
    pub descriptive_only: bool,
    /// Intentional omissions with their execution consequences.
    pub omissions: Vec<ProjectOmission>,
    /// Total actual copied bytes; independent of analytical row selection.
    pub maximum_output_bytes: u64,
    /// Finite manifest/typed-document allowance.
    pub maximum_document_bytes: usize,
}
/// An intentional absent selection, distinguished from a failed required copy.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectOmission {
    /// Origin-qualified descriptive selection.
    pub selection: String,
    /// Why the operator omitted it.
    pub reason: String,
    /// The omitted member is needed for executable admission.
    pub required_for_execution: bool,
}
/// A contained original document, never an ambient source path.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDocumentMember {
    /// Exact package position in the original loader order.
    pub package: usize,
    /// Original safe relative loader path.
    pub path: String,
    /// Contained immutable bytes member.
    pub member: String,
}
/// One exact admitted source with all original source and package-data bytes.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSourceClosure {
    /// Original canonical source revision, meaningful only under manifest origin authority.
    pub revision: String,
    /// Original scientific identity when actually retained by its source/context owner; absence is unknown.
    pub source_identity: Option<String>,
    /// Physical identity from actual source/context metadata; absent when not authenticated independently.
    pub physical_context: Option<String>,
    /// Original source packages in loader order.
    pub documents: Vec<ProjectDocumentMember>,
    /// Complete authored/data source codec. None is permitted only for an explicitly omitted descriptive source.
    pub replay_source: Option<String>,
    /// Exact physical package document bytes.
    pub physical_documents: Vec<ProjectDocumentMember>,
}
/// Compact factual metadata from the original immutable revision owner; it grants no source access.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSourceRevisionMetadata {
    /// Actual original immutable revision.
    pub key: String,
    /// Actual original problem owner.
    pub problem: String,
    /// Original immutable source sequence.
    pub sequence: u64,
    /// Original parent, if present; never a floating head.
    pub parent: Option<String>,
    /// Original operation acknowledgement identity.
    pub operation: String,
    /// Original source interpretation; no receiving compatibility is asserted.
    pub interpretation: String,
}
/// Original run/attempt facts for a deliberately omitted result payload.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectResultMetadata {
    /// Original requested coordinates; no selected row bytes are claimed copied.
    pub selection: super::ReviewScientificSelection,
    /// Actual original run's source revision.
    pub source_revision: String,
    /// Actual original attempt generation.
    pub generation: u64,
    /// Actual original lifecycle phase.
    pub phase: String,
    /// Actual original outcome, if the attempt has one.
    pub terminal_class: Option<String>,
    /// Original closed-manifest locator, if present; the manifest payload is not read.
    pub closed_manifest: Option<String>,
}
/// Preserved original study definition. It is not a receiving durable study receipt.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectStudyClosure {
    /// Original canonical study key.
    pub key: String,
    /// Exact original definition codec, with historical source references unchanged.
    pub definition: super::StudyDefinition,
}
/// Required external provider description, carrying neither a factory nor credentials.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectProviderRequirement {
    /// Original provider binding.
    pub name: String,
    /// Exact provider implementation identity.
    pub identity: String,
    /// Current bundle admission supports this provider without ambient installation.
    pub supported: bool,
}
/// Original artifact block interpretation and scientific extent, preserved independently of copied integrity.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectArtifactBlockMetadata {
    /// Original relation or opaque capsule label.
    pub relation: String,
    /// Original producer-framed payload digest; distinct from the plain copied-byte checksum.
    pub payload_digest: String,
    /// Original producer encoding provenance, never replaced with a new qualification.
    pub original_digest: String,
    /// Original scientific row extent, independent of copied bytes.
    pub rows: u64,
    /// Original sealed-file encoding.
    pub encoding: String,
    /// Original registry schema identity; empty for opaque bytes/JSON.
    pub schema: String,
}
/// Exact contained member extent and codec, independently verified before publication.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectBundleMember {
    /// Unique immutable member name.
    pub id: String,
    /// Local relative path selected by this manifest.
    pub relative_path: String,
    /// Strict supported interpretation.
    pub codec: String,
    /// Exact full copied extent, not an inferred selected-row count.
    pub copied_bytes: u64,
    /// Original plain BLAKE3 encoding checksum of these copied bytes.
    pub digest: String,
    /// Original result qualifiers, never promoted by publication.
    pub result: Option<super::PackResultSelection>,
    /// Original evidence artifact identity and selected ordinal, if applicable.
    pub artifact: Option<(String, u64)>,
    /// Original block codec, digest and scientific qualifiers.
    pub artifact_metadata: Option<ProjectArtifactBlockMetadata>,
}
/// Standalone versioned logical exchange manifest, distinct from diagnostic PackIndex v2.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    /// Logical exchange codec.
    pub version: Version<1>,
    /// Exact original owner selection; original service addresses/grants are excluded.
    /// Descriptive identities are retained selector facts, not independent scientific re-admission.
    pub origin: ProjectSelection,
    /// Selected source closure plus required historical source ancestry.
    pub sources: Vec<ProjectSourceClosure>,
    /// Authenticated original source owner metadata, including descriptive-only source selection.
    pub source_revisions: Vec<ProjectSourceRevisionMetadata>,
    /// Authenticated original result metadata when payload copying was explicitly omitted.
    pub result_metadata: Vec<ProjectResultMetadata>,
    /// Origin metadata; never receiving dataset rows or execution grants.
    pub datasets: Vec<pse_operations::canonical_research::ResearchDataset>,
    /// Origin frozen calibration/fit observation selections.
    pub research_selections: Vec<pse_operations::canonical_research::ResearchSelection>,
    /// Origin parameter-publication ancestry; never a receiving publication receipt.
    pub publications: Vec<pse_operations::canonical_research::ResearchPublication>,
    /// Original immutable context records.
    pub contexts: Vec<pse_operations::canonical_contexts::ProjectContext>,
    /// Original study definition codecs.
    pub studies: Vec<ProjectStudyClosure>,
    /// Existing supported explicit new-work requests with original predicates.
    pub reproducers: Vec<super::DiagnosticReproducer>,
    /// External implementation requirements; inspection never installs them.
    pub providers: Vec<ProjectProviderRequirement>,
    /// Exact original selected artifact headers, preserved as origin evidence.
    pub artifact_origins: Vec<pse_operations::canonical_artifacts::Artifact>,
    /// Complete copied selected membership.
    pub members: Vec<ProjectBundleMember>,
    /// Original configured finite copied-payload allowance, separate from document decoding limits.
    pub maximum_copy_bytes: u64,
    /// Intentional omissions.
    pub omissions: Vec<ProjectOmission>,
    /// All selected required offline evidence is present and verified.
    pub offline_complete: bool,
    /// Selected editable source and required providers/data support ordinary admission.
    pub executable_complete: bool,
    /// Descriptive-only selection cannot be admitted as executable source.
    pub descriptive_only: bool,
}
/// Read-only owner-composed closure explanation; no protection grant escapes in this document.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectClosureExplanation {
    /// Exact current source and metadata closure.
    pub manifest: ProjectManifest,
    /// Byte estimate for known source/evidence files; result encoding size is unknown before export.
    pub known_source_bytes: u64,
    /// Result sizes are observations made only while copying the selected rows.
    pub result_bytes_known: bool,
}
/// Independently verified logical bundle. No runtime/service capability is retained.
#[derive(Debug)]
pub struct ProjectBundleReader {
    directory: File,
    manifest: ProjectManifest,
    digest: String,
    maximum_document_bytes: usize,
}
fn io(error: std::io::Error) -> WorkflowError {
    contract(format!("project bundle I/O: {error}"))
}
fn checksum(mut input: impl Read, maximum: u64) -> Result<(u64, String), WorkflowError> {
    let mut hasher = pse_ids::EncodingHasher::new();
    let copied = std::io::copy(
        &mut (&mut input).take(maximum.saturating_add(1)),
        &mut hasher,
    )
    .map_err(io)?;
    if copied > maximum {
        return Err(contract("project member exceeds its declared extent"));
    }
    Ok((copied, hasher.finish().content_hash().to_hex()))
}
fn decode<T: serde::de::DeserializeOwned>(
    input: impl Read,
    maximum: usize,
) -> Result<T, WorkflowError> {
    let mut bytes = Vec::new();
    input
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() > maximum {
        return Err(contract("project document allowance exhausted"));
    }
    serde_json::from_slice(&bytes).map_err(|e| contract(format!("project codec: {e}")))
}
pub(super) fn encode<T: Serialize>(value: &T, maximum: usize) -> Result<Vec<u8>, WorkflowError> {
    // serde's bounded writer avoids allocating an unbounded document before checking it.
    struct Output {
        bytes: Vec<u8>,
        remaining: usize,
    }
    impl Write for Output {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if b.len() > self.remaining {
                return Err(std::io::Error::other(
                    "project document allowance exhausted",
                ));
            }
            self.bytes.extend_from_slice(b);
            self.remaining -= b.len();
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = Output {
        bytes: Vec::new(),
        remaining: maximum,
    };
    serde_json::to_writer(&mut output, value).map_err(|e| contract(e.to_string()))?;
    Ok(output.bytes)
}
impl ProjectBundleReader {
    /// Open and validate only manifest-contained members, without contacting an original service.
    pub fn open(path: &Path, maximum_document_bytes: usize) -> Result<Self, WorkflowError> {
        use rustix::fs::{Mode, OFlags};
        if maximum_document_bytes == 0 || maximum_document_bytes > 64 << 20 {
            return Err(contract("invalid project document allowance"));
        }
        let directory = File::from(
            rustix::fs::open(
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(std::io::Error::from)
            .map_err(io)?,
        );
        let manifest_file =
            contained_copy::open_contained(&directory, "project.json").map_err(io)?;
        let (_, digest) = checksum(
            manifest_file.try_clone().map_err(io)?,
            maximum_document_bytes as u64,
        )?;
        // try_clone shares the open description offset; reopen the manifest for decoding.
        let manifest: ProjectManifest = decode(
            contained_copy::open_contained(&directory, "project.json").map_err(io)?,
            maximum_document_bytes,
        )?;
        validate_manifest(&manifest)?;
        for member in &manifest.members {
            let file =
                contained_copy::open_contained(&directory, &member.relative_path).map_err(io)?;
            if file.metadata().map_err(io)?.len() != member.copied_bytes {
                return Err(contract("project member extent differs"));
            }
            let (bytes, digest) = checksum(file, member.copied_bytes)?;
            if bytes != member.copied_bytes || digest != member.digest {
                return Err(contract("project member digest differs"));
            }
            if member.codec == "arrow-ipc-stream.v1" {
                let result = member
                    .result
                    .as_ref()
                    .ok_or_else(|| contract("result member lacks original scientific selection"))?;
                crate::tooling::review_pack::validate_scientific_ipc(
                    contained_copy::open_contained(&directory, &member.relative_path)
                        .map_err(io)?,
                    member.copied_bytes,
                    maximum_document_bytes,
                    result,
                )
                .map_err(|e| contract(e.to_string()))?;
            }
            if let Some(metadata) = &member.artifact_metadata {
                let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
                hash.str("pse.execution.result-payload.v1");
                hash.part_reader(
                    member.copied_bytes,
                    &mut contained_copy::open_contained(&directory, &member.relative_path)
                        .map_err(io)?,
                    &|| Ok(()),
                )
                .map_err(io)?;
                if hash.finish_hash().to_hex() != metadata.payload_digest {
                    return Err(contract("original artifact payload digest differs"));
                }
            }
        }
        let reader = Self {
            directory,
            manifest,
            digest,
            maximum_document_bytes,
        };
        reader.validate_sources()?;
        Ok(reader)
    }
    fn validate_sources(&self) -> Result<(), WorkflowError> {
        for source in &self.manifest.sources {
            let Some(replay_source) = source.replay_source.as_deref() else {
                continue;
            };
            let document: super::ReplaySourceDocument = decode(
                self.member_bytes(replay_source, self.maximum_document_bytes)?
                    .as_slice(),
                self.maximum_document_bytes,
            )?;
            if Some(document.identity.to_string()) != source.source_identity
                || Some(document.physical_identity.to_string()) != source.physical_context
            {
                return Err(contract(
                    "source codec identity differs from selected manifest",
                ));
            }
            if self
                .manifest
                .members
                .iter()
                .find(|m| Some(m.id.as_str()) == source.replay_source.as_deref())
                .is_none_or(|m| m.codec != "replay-source.v1")
            {
                return Err(contract("source codec differs from selected manifest"));
            }
            let names = document
                .objects
                .iter()
                .map(|o| (&o.scope, &o.name))
                .collect::<BTreeSet<_>>();
            let logicals = document
                .objects
                .iter()
                .map(|o| &o.logical)
                .collect::<BTreeSet<_>>();
            if names.len() != document.objects.len()
                || logicals.len() != document.objects.len()
                || !logicals.iter().any(|s| s.as_str() == "modeling:context")
                || !logicals
                    .iter()
                    .any(|s| s.as_str() == "modeling:physical-scope")
                || !logicals
                    .iter()
                    .any(|s| s.as_str() == "source:ingress-frame:v1")
                || document
                    .objects
                    .iter()
                    .flat_map(|o| &o.references)
                    .any(|(scope, name)| !names.contains(&(scope, name)))
            {
                return Err(contract(
                    "copied source object/reference closure is incomplete",
                ));
            }
            for provider in &document.providers {
                let identity =
                    pse_ids::encoding_checksum(&encode(provider, self.maximum_document_bytes)?)
                        .content_hash()
                        .to_hex();
                if !self
                    .manifest
                    .providers
                    .iter()
                    .any(|p| p.name == provider.name && p.identity == identity && !p.supported)
                {
                    return Err(contract(
                        "provider requirement is missing or unsupported capability is overstated",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Original exact membership and independent completeness/qualification claims.
    pub fn inspect(&self) -> &ProjectManifest {
        &self.manifest
    }
    /// Encoding checksum of the exact independently validated manifest bytes.
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Open only a contained verified member; no origin path or service address is followed.
    pub fn open_member(&self, id: &str) -> Result<std::io::Take<File>, WorkflowError> {
        let member = self
            .manifest
            .members
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| contract("unindexed project member"))?;
        let file =
            contained_copy::open_contained(&self.directory, &member.relative_path).map_err(io)?;
        let (bytes, digest) = checksum(file.try_clone().map_err(io)?, member.copied_bytes)?;
        if bytes != member.copied_bytes || digest != member.digest {
            return Err(contract("project member changed after inspection"));
        }
        use std::io::{Seek, SeekFrom};
        let mut file = file;
        file.seek(SeekFrom::Start(0)).map_err(io)?;
        Ok(file.take(member.copied_bytes))
    }
    /// Read a bounded member and verify the bytes actually consumed, including changes after open().
    pub fn member_bytes(&self, id: &str, maximum_bytes: usize) -> Result<Vec<u8>, WorkflowError> {
        let member = self
            .manifest
            .members
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| contract("unindexed project member"))?;
        if member.copied_bytes > maximum_bytes as u64 {
            return Err(contract("project member read allowance exhausted"));
        }
        let mut bytes = Vec::new();
        self.open_member(id)?
            .take(maximum_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() as u64 != member.copied_bytes
            || pse_ids::encoding_checksum(&bytes).content_hash().to_hex() != member.digest
        {
            return Err(contract("consumed project member digest differs"));
        }
        Ok(bytes)
    }
    /// Select a contained supported request for explicit new execution via Runtime::run_reproducer.
    pub fn reproducer(&self, index: usize) -> Result<super::DiagnosticReproducer, WorkflowError> {
        self.manifest
            .reproducers
            .get(index)
            .cloned()
            .ok_or_else(|| contract("selected project reproducer absent"))
    }
    /// Original selected source and physical bytes for ordinary receiving admission.
    pub fn package_sources(&self) -> Result<super::PackageSources, WorkflowError> {
        let source = self
            .manifest
            .sources
            .iter()
            .find(|s| s.revision == self.manifest.origin.revision)
            .ok_or_else(|| contract("selected project source absent"))?;
        self.source_documents(source)
    }
    pub(super) fn source_documents(
        &self,
        source: &ProjectSourceClosure,
    ) -> Result<super::PackageSources, WorkflowError> {
        fn documents(
            reader: &ProjectBundleReader,
            records: &[ProjectDocumentMember],
        ) -> Result<Vec<BTreeMap<String, Vec<u8>>>, WorkflowError> {
            let count = records.iter().map(|d| d.package).max().map_or(0, |n| n + 1);
            let mut bundles = vec![BTreeMap::new(); count];
            let mut total = 0usize;
            for record in records {
                let member = reader
                    .manifest
                    .members
                    .iter()
                    .find(|m| m.id == record.member)
                    .ok_or_else(|| contract("document member absent"))?;
                total = total
                    .checked_add(
                        usize::try_from(member.copied_bytes)
                            .map_err(|_| contract("document extent"))?,
                    )
                    .ok_or_else(|| contract("document extent"))?;
                if total > reader.maximum_document_bytes {
                    return Err(contract("source admission document allowance exhausted"));
                }
                let bytes = reader.member_bytes(&record.member, reader.maximum_document_bytes)?;
                if bundles[record.package]
                    .insert(record.path.clone(), bytes)
                    .is_some()
                {
                    return Err(contract("ambiguous project loader path"));
                }
            }
            Ok(bundles)
        }
        let physical = documents(self, &source.physical_documents)?;
        if physical.len() != 1 {
            return Err(contract("physical closure must contain one package"));
        }
        Ok(super::PackageSources {
            physical: physical
                .into_iter()
                .next()
                .ok_or_else(|| contract("physical package absent"))?,
            modeling: documents(self, &source.documents)?,
        })
    }
}
pub(super) fn validate_manifest(manifest: &ProjectManifest) -> Result<(), WorkflowError> {
    if manifest.members.len() > 65536
        || manifest.sources.len() > 1024
        || manifest.sources.is_empty()
        || !manifest.offline_complete
    {
        return Err(contract("incomplete or excessive logical project closure"));
    }
    let copied = manifest.members.iter().try_fold(0u64, |total, m| {
        total
            .checked_add(m.copied_bytes)
            .ok_or_else(|| contract("project payload extent"))
    })?;
    if copied > manifest.maximum_copy_bytes {
        return Err(contract(
            "project copied payload exceeds original allowance",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for member in &manifest.members {
        if member.id.is_empty()
            || !ids.insert(member.id.as_str())
            || !paths.insert(member.relative_path.as_str())
        {
            return Err(contract("duplicate project member"));
        }
        contained_copy::safe_relative(&member.relative_path).map_err(io)?;
        if !member.relative_path.starts_with("members/")
            || !matches!(
                member.codec.as_str(),
                "authored-document.v1"
                    | "replay-source.v1"
                    | "arrow-ipc-stream.v1"
                    | "artifact-block.v1"
                    | "diagnostic-review-pack.v2"
            )
        {
            return Err(contract("unknown project member codec"));
        }
        if (member.codec == "arrow-ipc-stream.v1") != member.result.is_some() {
            return Err(contract(
                "scientific result qualifiers require their exact IPC stream codec",
            ));
        }
        pse_ids::ContentHash::parse_hex(&member.digest).map_err(|e| contract(e.to_string()))?;
        match (&member.artifact, &member.artifact_metadata) {
            (Some((artifact, ordinal)), Some(metadata)) => {
                if member.codec != "artifact-block.v1"
                    || manifest
                        .artifact_origins
                        .iter()
                        .find(|a| &a.key == artifact)
                        .is_none_or(|a| *ordinal >= a.block_count)
                    || metadata.relation.is_empty()
                {
                    return Err(contract(
                        "original artifact membership or qualifiers missing",
                    ));
                }
                match metadata.encoding.as_str() {
                    pse_operations::canonical_files::ARROW_IPC_FILE => {
                        pse_ids::SemanticId::parse_hex(&metadata.schema)
                            .map_err(|e| contract(e.to_string()))?;
                    }
                    pse_operations::canonical_files::OPAQUE_BYTES
                    | pse_operations::canonical_files::OPAQUE_JSON
                        if metadata.schema.is_empty() => {}
                    _ => return Err(contract("unsupported original artifact encoding/schema")),
                }
                pse_ids::ContentHash::parse_hex(&metadata.payload_digest)
                    .map_err(|e| contract(e.to_string()))?;
            }
            (None, None) if member.codec != "artifact-block.v1" => {}
            _ => return Err(contract("artifact codec or original qualifiers incomplete")),
        }
    }
    let mut revisions = BTreeSet::new();
    for source in &manifest.sources {
        if !revisions.insert(&source.revision) {
            return Err(contract("duplicate source closure"));
        }
        match source.replay_source.as_deref() {
            Some(replay)
                if ids.contains(replay)
                    && !source.physical_documents.is_empty()
                    && source.source_identity.is_some()
                    && source.physical_context.is_some() => {}
            None if manifest.descriptive_only
                && source.documents.is_empty()
                && source.physical_documents.is_empty()
                && manifest.omissions.iter().any(|o| {
                    o.selection == format!("source:{}", source.revision) && o.required_for_execution
                }) => {}
            _ => {
                return Err(contract(
                    "incomplete source closure lacks required replay/physical members or explicit descriptive omission",
                ));
            }
        }
        for documents in [&source.documents, &source.physical_documents] {
            let mut names = BTreeSet::new();
            for document in documents {
                contained_copy::safe_relative(&document.path).map_err(io)?;
                if document.package >= 1024
                    || !names.insert((document.package, &document.path))
                    || !ids.contains(document.member.as_str())
                {
                    return Err(contract("missing or ambiguous source document closure"));
                }
                if manifest
                    .members
                    .iter()
                    .find(|m| m.id == document.member)
                    .is_none_or(|m| m.codec != "authored-document.v1")
                {
                    return Err(contract("source document codec differs"));
                }
            }
        }
    }
    if !revisions.contains(&manifest.origin.revision) {
        return Err(contract("selected source closure absent"));
    }
    if manifest
        .sources
        .iter()
        .find(|s| s.revision == manifest.origin.revision)
        .is_none_or(|s| {
            s.source_identity
                .as_ref()
                .is_some_and(|id| id != &manifest.origin.source_identity.to_string())
                || s.physical_context
                    .as_ref()
                    .is_some_and(|id| id != &manifest.origin.physical_context.to_string())
        })
    {
        return Err(contract("selected source identity differs from origin"));
    }
    let mut metadata_revisions = BTreeSet::new();
    for revision in &manifest.source_revisions {
        if !metadata_revisions.insert(revision.key.as_str())
            || !revisions.contains(&revision.key)
            || revision.key.is_empty()
            || revision.problem.is_empty()
            || revision.operation.is_empty()
            || revision.interpretation.is_empty()
        {
            return Err(contract(
                "source revision metadata differs from selected closure",
            ));
        }
    }
    if manifest
        .sources
        .iter()
        .any(|s| !metadata_revisions.contains(s.revision.as_str()))
    {
        return Err(contract("source closure lacks original revision metadata"));
    }
    if manifest
        .source_revisions
        .iter()
        .find(|r| r.key == manifest.origin.revision)
        .is_some_and(|r| r.problem != manifest.origin.problem)
    {
        return Err(contract(
            "selected revision metadata belongs to another source problem",
        ));
    }
    for result in &manifest.result_metadata {
        if !manifest.descriptive_only
            || !revisions.contains(&result.source_revision)
            || !manifest.omissions.iter().any(|o| {
                o.selection == format!("result:{}", result.selection.member)
                    && o.required_for_execution
            })
        {
            return Err(contract(
                "descriptive result metadata lacks explicit omitted payload",
            ));
        }
    }
    let mut artifact_headers = BTreeSet::new();
    for header in &manifest.artifact_origins {
        if !artifact_headers.insert(header.key.as_str())
            || !revisions.contains(&header.revision)
            || header.block_count == 0
        {
            return Err(contract(
                "duplicate artifact header or missing original source closure",
            ));
        }
        if manifest
            .source_revisions
            .iter()
            .find(|r| r.key == header.revision)
            .is_none_or(|r| r.problem != header.primary_problem)
        {
            return Err(contract("original artifact source ownership differs"));
        }
        let selected = manifest
            .members
            .iter()
            .filter(|m| {
                m.artifact
                    .as_ref()
                    .is_some_and(|(key, _)| key == &header.key)
            })
            .collect::<Vec<_>>();
        if manifest.descriptive_only
            && selected.is_empty()
            && manifest.omissions.iter().any(|o| {
                o.selection == format!("artifact:{}", header.key) && o.required_for_execution
            })
        {
            continue;
        }
        let mut ordinals = BTreeSet::new();
        let mut bytes = 0u64;
        for member in selected {
            let ordinal = member
                .artifact
                .as_ref()
                .ok_or_else(|| contract("artifact coordinate absent"))?
                .1;
            if !ordinals.insert(ordinal) {
                return Err(contract("duplicate original artifact block ordinal"));
            }
            bytes = bytes
                .checked_add(member.copied_bytes)
                .ok_or_else(|| contract("artifact copied extent"))?;
        }
        if ordinals.len() as u64 != header.block_count
            || bytes != header.payload_bytes
            || ordinals
                .into_iter()
                .enumerate()
                .any(|(i, ordinal)| i as u64 != ordinal)
        {
            return Err(contract(
                "original artifact block coverage or byte extent is incomplete",
            ));
        }
    }
    let executable = !manifest.descriptive_only
        && !manifest.omissions.iter().any(|o| o.required_for_execution)
        && manifest.providers.iter().all(|p| p.supported);
    if manifest.executable_complete != executable {
        return Err(contract(
            "project execution completeness contradicts selected closure",
        ));
    }
    for dataset in &manifest.datasets {
        if !revisions.contains(&dataset.revision)
            || dataset
                .parent
                .as_ref()
                .is_some_and(|p| !manifest.datasets.iter().any(|d| &d.key == p))
            || !manifest.contexts.iter().any(|c| c.key == dataset.context)
        {
            return Err(contract("research dataset ancestry missing"));
        }
    }
    for selection in &manifest.research_selections {
        if !revisions.contains(&selection.revision)
            || !manifest.datasets.iter().any(|d| d.key == selection.dataset)
            || !manifest.contexts.iter().any(|c| c.key == selection.context)
        {
            return Err(contract("research selection closure missing"));
        }
    }
    for publication in &manifest.publications {
        if !revisions.contains(&publication.revision)
            || !revisions.contains(&publication.parent)
            || !manifest
                .research_selections
                .iter()
                .any(|s| s.key == publication.selection)
        {
            return Err(contract("parameter publication ancestry missing"));
        }
    }
    for study in &manifest.studies {
        if !revisions.contains(&study.definition.modeling_revision) {
            return Err(contract("study source closure missing"));
        }
    }
    Ok(())
}

impl Runtime {
    /// Copy the selected original closure, verify destination bytes, and publish its sole manifest last.
    pub async fn project_export(
        &self,
        destination: &Path,
        request: &ProjectExportRequest,
        physical: PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectManifest, WorkflowError> {
        self.project_boxed(self.project_export_resolved(
            destination,
            request,
            Some(physical),
            cancel,
        ))?
        .await
    }
    /// Export only authenticated descriptive metadata without admitting unavailable physical/source bytes.
    pub async fn project_export_metadata(
        &self,
        destination: &Path,
        request: &ProjectExportRequest,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectManifest, WorkflowError> {
        if !request.descriptive_only {
            return Err(contract(
                "metadata export requires explicit descriptive-only selection",
            ));
        }
        self.project_boxed(self.project_export_resolved(destination, request, None, cancel))?
            .await
    }
    async fn project_export_resolved(
        &self,
        destination: &Path,
        request: &ProjectExportRequest,
        physical: Option<PhysicalContext>,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectManifest, WorkflowError> {
        let mut closure = self
            .project_boxed(self.resolve_project_closure(request, physical, cancel))?
            .await?;
        std::fs::create_dir(destination).map_err(io)?;
        let root = destination.canonicalize().map_err(io)?;
        std::fs::create_dir(root.join("members")).map_err(io)?;
        std::fs::create_dir(root.join(".staging")).map_err(io)?;
        let token = cancel.token();
        let mut total = 0u64;
        fn finish_member(
            root: &Path,
            manifest: &mut ProjectManifest,
            total: &mut u64,
            maximum: u64,
            token: &pse_columnar::CancellationToken,
            id: &str,
            bytes: u64,
        ) -> Result<(), WorkflowError> {
            *total = total
                .checked_add(bytes)
                .ok_or_else(|| contract("project copied extent"))?;
            if *total > maximum {
                return Err(contract("project output allowance exhausted"));
            }
            let m = manifest
                .members
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| contract("unselected project member"))?;
            let staged = root.join(".staging").join(id);
            let (actual, digest) = checksum(File::open(&staged).map_err(io)?, bytes)?;
            if actual != bytes {
                return Err(contract("short copied project member"));
            }
            File::open(&staged).map_err(io)?.sync_all().map_err(io)?;
            token.checkpoint().map_err(pse_engine::EngineError::from)?;
            std::fs::rename(staged, root.join(&m.relative_path)).map_err(io)?;
            m.copied_bytes = bytes;
            m.digest = digest;
            Ok(())
        }
        for (id, bytes) in &closure.documents {
            super::study::creation_checkpoint(cancel)?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(".staging").join(id.as_str()))
                .map_err(io)?;
            contained_copy::copy_exact(
                &mut bytes.as_slice(),
                &mut file,
                bytes.len() as u64,
                || {
                    if token.is_cancelled() {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Interrupted,
                            "project copy cancelled",
                        ))
                    } else {
                        Ok(())
                    }
                },
            )
            .map_err(io)?;
            file.sync_all().map_err(io)?;
            drop(file);
            finish_member(
                &root,
                &mut closure.manifest,
                &mut total,
                request.maximum_output_bytes,
                &token,
                id,
                bytes.len() as u64,
            )?;
        }
        for (id, reader) in &mut closure.results {
            let staging = root.join(".staging").join(id.as_str());
            let remaining = request.maximum_output_bytes.saturating_sub(total);
            let bytes = match reader {
                super::project_closure::ResultMember::Full(r) => {
                    r.export_ipc_bounded(&staging, remaining, &token).await?
                }
                super::project_closure::ResultMember::Selected(r) => {
                    r.export_ipc_bounded(&staging, remaining, &token).await?
                }
            };
            finish_member(
                &root,
                &mut closure.manifest,
                &mut total,
                request.maximum_output_bytes,
                &token,
                id,
                bytes,
            )?;
        }
        for (artifact_index, read) in closure.artifacts.iter().enumerate() {
            for ordinal in 0..read.header().block_count {
                super::study::creation_checkpoint(cancel)?;
                let block = read.block(ordinal).await?;
                let id = format!("artifact-{artifact_index}-{ordinal}");
                let selected = closure
                    .manifest
                    .members
                    .iter_mut()
                    .find(|m| m.id == id)
                    .ok_or_else(|| contract("unselected artifact member"))?;
                selected.artifact_metadata = Some(ProjectArtifactBlockMetadata {
                    relation: block.relation.clone(),
                    payload_digest: block.digest.clone(),
                    original_digest: block.original_digest.clone(),
                    rows: block.rows,
                    encoding: block.encoding().into(),
                    schema: block.schema().into(),
                });
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join(".staging").join(&id))
                    .map_err(io)?;
                contained_copy::copy_exact(
                    &mut block.payload.as_slice(),
                    &mut file,
                    block.payload.len() as u64,
                    || {
                        if token.is_cancelled() {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Interrupted,
                                "project copy cancelled",
                            ))
                        } else {
                            Ok(())
                        }
                    },
                )
                .map_err(io)?;
                file.sync_all().map_err(io)?;
                drop(file);
                finish_member(
                    &root,
                    &mut closure.manifest,
                    &mut total,
                    request.maximum_output_bytes,
                    &token,
                    &id,
                    block.payload.len() as u64,
                )?;
            }
        }
        closure.renew().await?;
        closure.manifest.offline_complete = true;
        validate_manifest(&closure.manifest)?;
        // Reopen every published contained member and verify the destination while all original grants remain owned.
        let directory = File::open(&root).map_err(io)?;
        for m in &closure.manifest.members {
            let (bytes, digest) = checksum(
                contained_copy::open_contained(&directory, &m.relative_path).map_err(io)?,
                m.copied_bytes,
            )?;
            if bytes != m.copied_bytes || digest != m.digest {
                return Err(contract("destination project member verification failed"));
            }
        }
        let bytes = encode(&closure.manifest, request.maximum_document_bytes)?;
        closure.renew().await?;
        super::study::creation_checkpoint(cancel)?;
        contained_copy::publish_document(
            &root,
            ".project-manifest",
            "project.json",
            &bytes,
            || {
                if token.is_cancelled() {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "project publication cancelled",
                    ))
                } else {
                    Ok(())
                }
            },
        )
        .map_err(io)?;
        Ok(closure.manifest.clone())
    }
}

/// Receiving placement and preserved origin, without imported historical execution receipts.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectImportReport {
    /// Fresh receiving project.
    pub project: String,
    /// Fresh writable receiving branch.
    pub branch: String,
    /// Fresh receiving source problem.
    pub problem: String,
    /// Fresh receiving immutable source revision.
    pub revision: String,
    /// Original source attribution, preserved separately from receiving ownership.
    pub origin: ProjectSelection,
    /// Exact imported manifest checksum.
    pub bundle_digest: String,
    /// Current project-owned copied evidence; independent of original pack lifetime.
    pub evidence_artifacts: Vec<String>,
    /// Original current creation intent, retained unchanged for exact recovery after uncertain completion.
    pub artifact_intent: Option<pse_operations::canonical_artifacts::Artifact>,
    /// Original collected occurrences; these receipts never become receiving run authority.
    pub retired_originals: Vec<pse_operations::canonical_artifacts::Artifact>,
    /// Source effect has acknowledged; this alone is not project availability.
    pub source_admitted: bool,
    /// Exact artifact membership finished and project is publicly available.
    pub project_complete: bool,
}
/// Explicit original-occurrence recovery outcome; pending cleanup never means a ready project.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectImportRecoveryState {
    /// The original active artifact settled and the project is available.
    ProjectComplete,
    /// No current artifact was issued; the caller can deliberately retry copying under the same import intent.
    ReadyForArtifact,
    /// Original allowance or bounded original-owner cleanup has not settled.
    OriginalCleanupPending,
    /// The exact original was collected and its immutable receipt is retained.
    OriginalCollected,
}
/// Exact recovery receipt, including current project availability and retired original headers.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectImportRecoveryReport {
    /// Current original import checkpoint and availability.
    pub import: ProjectImportReport,
    /// Exact state; no omitted header is interpreted as cleanup completion.
    pub state: ProjectImportRecoveryState,
}
impl ProjectBundleReader {
    fn manifest_bytes(&self) -> Result<Vec<u8>, WorkflowError> {
        let mut bytes = Vec::new();
        contained_copy::open_contained(&self.directory, "project.json")
            .map_err(io)?
            .take(self.maximum_document_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > self.maximum_document_bytes
            || pse_ids::encoding_checksum(&bytes).content_hash().to_hex() != self.digest
        {
            return Err(contract("project manifest changed after inspection"));
        }
        Ok(bytes)
    }
    fn archive_extent(&self) -> Result<u64, WorkflowError> {
        self.manifest.members.iter().try_fold(
            self.manifest_bytes()?.len() as u64 + 8,
            |total, m| {
                total
                    .checked_add(m.copied_bytes)
                    .ok_or_else(|| contract("import member extent"))
            },
        )
    }
    fn archive(&self, cancel: &crate::CancelSource) -> Result<File, WorkflowError> {
        struct Output<'a> {
            file: &'a mut File,
            hash: pse_ids::EncodingHasher,
        }
        impl Write for Output<'_> {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                let n = self.file.write(b)?;
                self.hash.update(&b[..n]);
                Ok(n)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.file.flush()
            }
        }
        let manifest = self.manifest_bytes()?;
        let mut file = tempfile::tempfile().map_err(io)?;
        file.write_all(&(manifest.len() as u64).to_le_bytes())
            .map_err(io)?;
        file.write_all(&manifest).map_err(io)?;
        for member in &self.manifest.members {
            super::study::creation_checkpoint(cancel)?;
            let mut source = self.open_member(&member.id)?;
            let mut output = Output {
                file: &mut file,
                hash: pse_ids::EncodingHasher::new(),
            };
            contained_copy::copy_exact(&mut source, &mut output, member.copied_bytes, || {
                if cancel.token().is_cancelled() {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "import copy cancelled",
                    ))
                } else {
                    Ok(())
                }
            })
            .map_err(io)?;
            if output.hash.finish().content_hash().to_hex() != member.digest {
                return Err(contract("consumed import member digest differs"));
            }
        }
        file.sync_all().map_err(io)?;
        file.seek(SeekFrom::Start(0)).map_err(io)?;
        Ok(file)
    }
    fn replay_source(&self) -> Result<super::ReplaySourceDocument, WorkflowError> {
        let selected = self
            .manifest
            .sources
            .iter()
            .find(|s| s.revision == self.manifest.origin.revision)
            .ok_or_else(|| contract("selected source absent"))?;
        decode(
            self.member_bytes(
                selected.replay_source.as_deref().ok_or_else(|| {
                    contract("descriptive source has no executable replay member")
                })?,
                self.maximum_document_bytes,
            )?
            .as_slice(),
            self.maximum_document_bytes,
        )
    }
    fn physical_documents(&self) -> Result<BTreeMap<String, Vec<u8>>, WorkflowError> {
        let selected = self
            .manifest
            .sources
            .iter()
            .find(|s| s.revision == self.manifest.origin.revision)
            .ok_or_else(|| contract("selected source absent"))?;
        let mut documents = BTreeMap::new();
        for document in &selected.physical_documents {
            if document.package != 0
                || documents
                    .insert(
                        document.path.clone(),
                        self.member_bytes(&document.member, self.maximum_document_bytes)?,
                    )
                    .is_some()
            {
                return Err(contract("ambiguous selected physical package"));
            }
        }
        Ok(documents)
    }
}
fn import_report(
    row: &pse_operations::canonical_project_imports::ProjectImport,
) -> Result<ProjectImportReport, WorkflowError> {
    let origin: ProjectSelection = serde_json::from_slice(row.origin.as_slice())
        .map_err(|e| contract(format!("import origin codec: {e}")))?;
    let artifact = row
        .artifact
        .as_ref()
        .map(|b| {
            serde_json::from_slice::<pse_operations::canonical_artifacts::Artifact>(b.as_slice())
                .map_err(|e| contract(e.to_string()))
        })
        .transpose()?;
    let retired_originals = row
        .retired_artifacts
        .as_ref()
        .map(|b| {
            serde_json::from_slice::<Vec<pse_operations::canonical_artifacts::Artifact>>(
                b.as_slice(),
            )
            .map_err(|e| contract(e.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    Ok(ProjectImportReport {
        project: row.project.clone(),
        branch: row.branch.clone(),
        problem: row.problem.clone(),
        revision: row.revision.clone(),
        origin,
        bundle_digest: row.bundle_digest.clone(),
        evidence_artifacts: artifact.iter().map(|a| a.key.clone()).collect(),
        artifact_intent: artifact,
        retired_originals,
        source_admitted: true,
        project_complete: row.complete,
    })
}
impl Runtime {
    /// Issue one immutable receiving intent with fresh project, branch, source and operation IDs.
    pub async fn project_import_issue(
        &self,
        bundle: &ProjectBundleReader,
        title: &str,
        branch_name: &str,
    ) -> Result<ProjectImportIntent, WorkflowError> {
        let _ = bundle.manifest_bytes()?;
        if !bundle.inspect().executable_complete {
            return Err(contract(
                "bundle is descriptive or has unsupported executable closure",
            ));
        }
        let store = self.canonical_store()?;
        let fresh = || pse_operations::mint_id::<pse_ids::SemanticId>().to_string();
        Ok(store.project_import_intent(
            &fresh(),
            &fresh(),
            &fresh(),
            &fresh(),
            bundle.digest(),
            encode(&bundle.inspect().origin, 131072)?,
            title,
            branch_name,
        )?)
    }
    /// Observe the original import checkpoint without making a new effect or consulting current heads.
    pub async fn project_import_status(
        &self,
        intent: &ProjectImportIntent,
    ) -> Result<Option<ProjectImportReport>, WorkflowError> {
        self.canonical_store()?
            .project_import_record(intent)
            .await?
            .as_ref()
            .map(import_report)
            .transpose()
    }
    /// Stage copied bytes and ordinary admission, then settle this exact guarded source/artifact activation.
    pub async fn project_import(
        &self,
        bundle: &ProjectBundleReader,
        intent: &ProjectImportIntent,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectImportReport, WorkflowError> {
        intent.validate()?;
        if bundle.digest() != intent.bundle_digest()
            || encode(&bundle.inspect().origin, 131072)?.as_slice() != intent.origin_bytes()
        {
            return Err(contract("import intent bundle digest differs"));
        }
        let store = self.canonical_store()?;
        // Settle the original source effect before any mutable preflight or renewed artifact intent.
        let prior = store.project_import_record(intent).await?;
        if let Some(row) = prior.as_ref().filter(|row| row.complete) {
            return import_report(row);
        }
        if let Some(bytes) = prior.as_ref().and_then(|row| row.artifact.as_ref()) {
            let original: pse_operations::canonical_artifacts::Artifact =
                serde_json::from_slice(bytes.as_slice()).map_err(|e| contract(e.to_string()))?;
            let status = store.artifact_creation_status(&original).await?;
            if status
                .header
                .as_ref()
                .is_some_and(|a| a.active && !a.retiring)
            {
                return import_report(&store.finish_project_import(intent).await?);
            }
            if !status.window_open {
                return Err(contract(
                    "original import artifact allowance closed; explicitly recover its unchanged original artifact intent before a fresh copy",
                ));
            }
        }
        let archive_extent = bundle.archive_extent()?;
        let chunk = pse_operations::canonical_files::FILE_TARGET_BYTES;
        let _archive_owner = self.shared.math().reserve(
            "project:import-copied-chunk",
            chunk
                .checked_mul(4)
                .ok_or_else(|| contract("import chunk extent"))?,
        )?;
        let mut archive = bundle.archive(cancel)?;
        let revision = if let Some(row) = &prior {
            store
                .revision(&row.revision)
                .await?
                .ok_or_else(|| contract("original import source unavailable"))?
        } else {
            if !bundle.inspect().executable_complete {
                return Err(contract(
                    "unsupported or incomplete executable project closure",
                ));
            }
            let selected_source = bundle
                .inspect()
                .sources
                .iter()
                .find(|s| s.revision == bundle.inspect().origin.revision)
                .ok_or_else(|| contract("selected import source absent"))?;
            let source_extent = bundle
                .inspect()
                .members
                .iter()
                .filter(|m| {
                    Some(m.id.as_str()) == selected_source.replay_source.as_deref()
                        || selected_source
                            .physical_documents
                            .iter()
                            .any(|d| d.member == m.id)
                })
                .try_fold(0usize, |total, m| {
                    total
                        .checked_add(
                            usize::try_from(m.copied_bytes)
                                .map_err(|_| contract("import source extent"))?,
                        )
                        .ok_or_else(|| contract("import source extent"))
                })?;
            let _source_owner = self.shared.math().reserve(
                "project:import-source-admission",
                source_extent
                    .checked_mul(4)
                    .ok_or_else(|| contract("import source extent"))?,
            )?;
            let local = self
                .clone()
                .with_durability(super::DurabilitySelection::Ephemeral)?;
            let physical = self
                .project_boxed(local.physical_from_sources(&bundle.physical_documents()?, cancel))?
                .await?;
            let replay = bundle.replay_source()?;
            let admitted = replay.reconstruct(&local, physical, BTreeMap::new(), cancel)?;
            let package = self
                .project_boxed(admitted.publish_portable_import(self, intent, cancel))?
                .await?;
            package
                .canonical_revision()
                .cloned()
                .ok_or_else(|| contract("import source was not recorded"))?
        };
        let row = store
            .project_import_record(intent)
            .await?
            .ok_or_else(|| contract("import source checkpoint unavailable"))?;
        let provenance = encode(&(1u8, intent.bundle_digest()), 131072)?;
        let mut header = if let Some(bytes) = &row.artifact {
            serde_json::from_slice::<pse_operations::canonical_artifacts::Artifact>(
                bytes.as_slice(),
            )
            .map_err(|e| contract(format!("import artifact intent codec: {e}")))?
        } else {
            store
                .new_artifact(
                    &revision,
                    "project-import-evidence",
                    provenance.clone(),
                    archive_extent.div_ceil(chunk as u64),
                    archive_extent,
                )
                .await?
        };
        let original = header.clone();
        let artifact_key = header.key.clone();
        let mut load_block = |ordinal: u64| -> Result<
            pse_operations::canonical_artifacts::ArtifactBlock,
            pse_operations::canonical::CanonicalError,
        > {
            if cancel.token().is_cancelled() {
                return Err(pse_operations::canonical::CanonicalError::Configuration(
                    "import chunk copy cancelled".into(),
                ));
            }
            let start = ordinal
                .checked_mul(chunk as u64)
                .ok_or(pse_operations::canonical::CanonicalError::PayloadLimit)?;
            if start >= archive_extent {
                return Err(pse_operations::canonical::CanonicalError::PayloadLimit);
            }
            let size = usize::try_from((archive_extent - start).min(chunk as u64))
                .map_err(|_| pse_operations::canonical::CanonicalError::PayloadLimit)?;
            let mut bytes = vec![0u8; size];
            archive
                .seek(SeekFrom::Start(start))
                .and_then(|_| archive.read_exact(&mut bytes))
                .map_err(|e| {
                    pse_operations::canonical::CanonicalError::Configuration(e.to_string())
                })?;
            Ok(pse_operations::canonical_artifacts::artifact_block(
                &artifact_key,
                ordinal,
                "project-import-evidence",
                bytes,
                intent.bundle_digest(),
                0,
            ))
        };
        pse_operations::canonical_artifacts::seal_artifact_stream_request(
            &mut header,
            std::slice::from_ref(&revision),
            &[],
            &mut load_block,
            "project",
            intent.operation(),
        )?;
        if row.artifact.is_some() && header != original {
            return Err(contract("original import artifact intent differs"));
        }
        if header.provenance.as_slice() != provenance
            || header.revision != revision.key
            || header.payload_bytes != archive_extent
            || header.block_count != archive_extent.div_ceil(chunk as u64)
        {
            return Err(contract("import artifact membership differs"));
        }
        store.project_import_bind_artifact(intent, &header).await?;
        // Original header is durable before copy. A lost copy/activation reply retries these same keys.
        store
            .persist_artifact_stream(
                &header,
                std::slice::from_ref(&revision),
                &[],
                &mut load_block,
                "project",
                intent.operation(),
            )
            .await?;
        let completed = store.finish_project_import(intent).await?;
        import_report(&completed)
    }
    /// Explicitly settle or collect this exact original artifact; never abort or renew an active occurrence.
    pub async fn project_import_recover(
        &self,
        intent: &ProjectImportIntent,
        original: &pse_operations::canonical_artifacts::Artifact,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectImportRecoveryReport, WorkflowError> {
        intent.validate()?;
        super::study::creation_checkpoint(cancel)?;
        let store = self.canonical_store()?;
        let row = store
            .project_import_record(intent)
            .await?
            .ok_or_else(|| contract("import source has not acknowledged"))?;
        let report = import_report(&row)?;
        if report.retired_originals.iter().any(|a| a == original) {
            return Ok(ProjectImportRecoveryReport {
                import: report,
                state: ProjectImportRecoveryState::OriginalCollected,
            });
        }
        if report.artifact_intent.as_ref() != Some(original) {
            return Err(contract(
                "recovery requires the unchanged original import artifact intent",
            ));
        }
        if row.complete {
            return Ok(ProjectImportRecoveryReport {
                import: report,
                state: ProjectImportRecoveryState::ProjectComplete,
            });
        }
        let status = store.artifact_creation_status(original).await?;
        if let Some(header) = status.header.as_ref().filter(|a| a.active) {
            if header.retiring {
                return Err(contract(
                    "active retiring artifact requires its explicit owner settlement",
                ));
            }
            let complete = store.finish_project_import(intent).await?;
            return Ok(ProjectImportRecoveryReport {
                import: import_report(&complete)?,
                state: ProjectImportRecoveryState::ProjectComplete,
            });
        }
        if status.window_open || !store.collect_artifact(original).await? {
            return Ok(ProjectImportRecoveryReport {
                import: report,
                state: ProjectImportRecoveryState::OriginalCleanupPending,
            });
        }
        super::study::creation_checkpoint(cancel)?;
        let settled = store
            .project_import_retire_artifact(intent, original)
            .await?;
        Ok(ProjectImportRecoveryReport {
            import: import_report(&settled)?,
            state: ProjectImportRecoveryState::OriginalCollected,
        })
    }
    /// Reopen current copied local source and physical closure after completed import, without the supplied pack.
    pub async fn project_import_open(
        &self,
        intent: &ProjectImportIntent,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingPackage, WorkflowError> {
        let store = self.canonical_store()?;
        let row = store
            .project_import_record(intent)
            .await?
            .filter(|r| r.complete)
            .ok_or_else(|| contract("project import membership is incomplete"))?;
        let header: pse_operations::canonical_artifacts::Artifact = serde_json::from_slice(
            row.artifact
                .as_ref()
                .ok_or_else(|| contract("import copied evidence absent"))?
                .as_slice(),
        )
        .map_err(|e| contract(e.to_string()))?;
        let read = store.read_artifact(&header.key).await?;
        let chunk = pse_operations::canonical_files::FILE_TARGET_BYTES;
        let _owner = self.shared.math().reserve(
            "project:import-reopen-chunk",
            chunk
                .checked_mul(4)
                .ok_or_else(|| contract("import reopen extent"))?,
        )?;
        let prefix = artifact_range(&read, 0, 8, 8, cancel).await?;
        let length = u64::from_le_bytes(
            prefix
                .as_slice()
                .try_into()
                .map_err(|_| contract("invalid imported archive prefix"))?,
        );
        if length > 64 << 20 {
            return Err(contract("imported manifest allowance exhausted"));
        }
        let _manifest_owner = self.shared.math().reserve(
            "project:import-manifest",
            (length as usize)
                .checked_mul(4)
                .ok_or_else(|| contract("import manifest extent"))?,
        )?;
        let manifest_bytes = artifact_range(&read, 8, 8 + length, 64 << 20, cancel).await?;
        if pse_ids::encoding_checksum(&manifest_bytes)
            .content_hash()
            .to_hex()
            != intent.bundle_digest()
        {
            return Err(contract("imported manifest digest differs"));
        }
        let manifest: ProjectManifest =
            serde_json::from_slice(&manifest_bytes).map_err(|e| contract(e.to_string()))?;
        validate_manifest(&manifest)?;
        let source = manifest
            .sources
            .iter()
            .find(|s| s.revision == manifest.origin.revision)
            .ok_or_else(|| contract("import source closure absent"))?;
        let mut ranges = BTreeMap::new();
        let mut position = 8 + length;
        for member in &manifest.members {
            let end = position
                .checked_add(member.copied_bytes)
                .ok_or_else(|| contract("import archive extent"))?;
            ranges.insert(member.id.as_str(), (position, end, member));
            position = end;
        }
        if position != read.header().payload_bytes {
            return Err(contract("imported archive extent differs"));
        }
        let physical_bytes = source
            .physical_documents
            .iter()
            .try_fold(0usize, |total, d| {
                let (start, end, _) = ranges
                    .get(d.member.as_str())
                    .ok_or_else(|| contract("physical member absent"))?;
                total
                    .checked_add(
                        usize::try_from(end - start).map_err(|_| contract("physical extent"))?,
                    )
                    .ok_or_else(|| contract("physical extent"))
            })?;
        let _physical_owner = self.shared.math().reserve(
            "project:import-physical-documents",
            physical_bytes
                .checked_mul(4)
                .ok_or_else(|| contract("physical extent"))?,
        )?;
        let mut physical_documents = BTreeMap::new();
        for document in &source.physical_documents {
            let (start, end, member) = ranges
                .get(document.member.as_str())
                .ok_or_else(|| contract("import physical member absent"))?;
            let bytes = artifact_range(&read, *start, *end, 64 << 20, cancel).await?;
            if pse_ids::encoding_checksum(&bytes).content_hash().to_hex() != member.digest {
                return Err(contract("import physical digest differs"));
            }
            physical_documents.insert(document.path.clone(), bytes);
        }
        let physical = self
            .project_boxed(self.physical_from_sources(&physical_documents, cancel))?
            .await?;
        let revision = store
            .project_branch_revision(&row.branch)
            .await?
            .ok_or_else(|| contract("imported current branch source unavailable"))?;
        self.project_boxed(self.modeling_revision(revision, physical, BTreeMap::new()))?
            .await
    }
}
async fn artifact_range(
    read: &pse_operations::canonical_artifacts::ArtifactRead,
    start: u64,
    end: u64,
    maximum: usize,
    cancel: &crate::CancelSource,
) -> Result<Vec<u8>, WorkflowError> {
    if end < start || end > read.header().payload_bytes || end - start > maximum as u64 {
        return Err(contract(
            "selected imported artifact range exceeds allowance",
        ));
    }
    let chunk = pse_operations::canonical_files::FILE_TARGET_BYTES as u64;
    let mut result = Vec::with_capacity((end - start) as usize);
    let mut position = start;
    while position < end {
        super::study::creation_checkpoint(cancel)?;
        let ordinal = position / chunk;
        let block = read.block(ordinal).await?;
        let offset = (position % chunk) as usize;
        let length =
            (end - position).min(block.payload.len().saturating_sub(offset) as u64) as usize;
        if length == 0 {
            return Err(contract("short imported artifact chunk"));
        }
        result.extend_from_slice(&block.payload[offset..offset + length]);
        position += length as u64;
    }
    Ok(result)
}

#[cfg(test)]
#[path = "project_portability_tests.rs"]
mod tests;
