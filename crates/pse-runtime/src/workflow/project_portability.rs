// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical project exchange. Origin evidence never becomes receiving execution authority.
use super::{Runtime, WorkflowError, PhysicalContext, ProjectSelection, contract};
use pse_model::document::Version;
use serde::{Serialize, Deserialize};
use std::{collections::{BTreeMap, BTreeSet}, fs::File, io::{Read, Write}, path::Path};
use crate::tooling::contained_copy;

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
    /// Original admitted scientific identity.
    pub source_identity: String,
    /// Complete physical meaning used for admission.
    pub physical_context: String,
    /// Original source packages in loader order.
    pub documents: Vec<ProjectDocumentMember>,
    /// Existing complete authored/data source codec, also supports typed source edits without invented text.
    pub replay_source: String,
    /// Exact physical package document bytes.
    pub physical_documents: Vec<ProjectDocumentMember>,
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
}
/// Standalone versioned logical exchange manifest, distinct from diagnostic PackIndex v2.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    /// Logical exchange codec.
    pub version: Version<1>,
    /// Exact original owner selection; original service addresses/grants are excluded.
    pub origin: ProjectSelection,
    /// Selected source closure plus required historical source ancestry.
    pub sources: Vec<ProjectSourceClosure>,
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
    /// Complete copied selected membership.
    pub members: Vec<ProjectBundleMember>,
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
fn io(error: std::io::Error) -> WorkflowError { contract(format!("project bundle I/O: {error}")) }
fn checksum(mut input: impl Read, maximum: u64) -> Result<(u64,String), WorkflowError> {
    let mut hasher = pse_ids::EncodingHasher::new();
    let copied = std::io::copy(&mut (&mut input).take(maximum.saturating_add(1)), &mut hasher).map_err(io)?;
    if copied > maximum { return Err(contract("project member exceeds its declared extent")); }
    Ok((copied, hasher.finish().content_hash().to_hex()))
}
fn decode<T: serde::de::DeserializeOwned>(input: impl Read, maximum: usize) -> Result<T,WorkflowError> {
    let mut bytes = Vec::new();
    input.take(maximum as u64 + 1).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len()>maximum { return Err(contract("project document allowance exhausted")); }
    serde_json::from_slice(&bytes).map_err(|e|contract(format!("project codec: {e}")))
}
pub(super) fn encode<T: Serialize>(value: &T, maximum: usize)->Result<Vec<u8>,WorkflowError> {
    // serde's bounded writer avoids allocating an unbounded document before checking it.
    struct Output { bytes:Vec<u8>, remaining:usize }
    impl Write for Output {
        fn write(&mut self,b:&[u8])->std::io::Result<usize> { if b.len()>self.remaining { return Err(std::io::Error::other("project document allowance exhausted")); } self.bytes.extend_from_slice(b); self.remaining-=b.len(); Ok(b.len()) }
        fn flush(&mut self)->std::io::Result<()> { Ok(()) }
    }
    let mut output=Output{bytes:Vec::new(),remaining:maximum};
    serde_json::to_writer(&mut output,value).map_err(|e|contract(e.to_string()))?;
    Ok(output.bytes)
}
impl ProjectBundleReader {
    /// Open and validate only manifest-contained members, without contacting an original service.
    pub fn open(path:&Path,maximum_document_bytes:usize)->Result<Self,WorkflowError> {
        use rustix::fs::{Mode,OFlags};
        if maximum_document_bytes==0 || maximum_document_bytes>64<<20 { return Err(contract("invalid project document allowance")); }
        let directory=File::from(rustix::fs::open(path,OFlags::RDONLY|OFlags::DIRECTORY|OFlags::NOFOLLOW|OFlags::CLOEXEC,Mode::empty()).map_err(std::io::Error::from).map_err(io)?);
        let manifest_file=contained_copy::open_contained(&directory,"project.json").map_err(io)?;
        let (_,digest)=checksum(manifest_file.try_clone().map_err(io)?,maximum_document_bytes as u64)?;
        // try_clone shares the open description offset; reopen the manifest for decoding.
        let manifest:ProjectManifest=decode(contained_copy::open_contained(&directory,"project.json").map_err(io)?,maximum_document_bytes)?;
        validate_manifest(&manifest)?;
        for member in &manifest.members {
            let file=contained_copy::open_contained(&directory,&member.relative_path).map_err(io)?;
            if file.metadata().map_err(io)?.len()!=member.copied_bytes { return Err(contract("project member extent differs")); }
            let (bytes,digest)=checksum(file,member.copied_bytes)?;
            if bytes!=member.copied_bytes || digest!=member.digest { return Err(contract("project member digest differs")); }
        }
        Ok(Self{directory,manifest,digest,maximum_document_bytes})
    }
    /// Original exact membership and independent completeness/qualification claims.
    pub fn inspect(&self)->&ProjectManifest { &self.manifest }
    /// Encoding checksum of the exact independently validated manifest bytes.
    pub fn digest(&self)->&str { &self.digest }
    /// Open only a contained verified member; no origin path or service address is followed.
    pub fn open_member(&self,id:&str)->Result<std::io::Take<File>,WorkflowError> {
        let member=self.manifest.members.iter().find(|m|m.id==id).ok_or_else(||contract("unindexed project member"))?;
        let file=contained_copy::open_contained(&self.directory,&member.relative_path).map_err(io)?;
        let (bytes,digest)=checksum(file.try_clone().map_err(io)?,member.copied_bytes)?;
        if bytes!=member.copied_bytes || digest!=member.digest { return Err(contract("project member changed after inspection")); }
        use std::io::{Seek,SeekFrom};
        let mut file=file; file.seek(SeekFrom::Start(0)).map_err(io)?;
        Ok(file.take(member.copied_bytes))
    }
    /// Select a contained supported request for explicit new execution via Runtime::run_reproducer.
    pub fn reproducer(&self,index:usize)->Result<super::DiagnosticReproducer,WorkflowError> {
        self.manifest.reproducers.get(index).cloned().ok_or_else(||contract("selected project reproducer absent"))
    }
    /// Original selected source and physical bytes for ordinary receiving admission.
    pub fn package_sources(&self)->Result<super::PackageSources,WorkflowError> {
        let source=self.manifest.sources.iter().find(|s|s.revision==self.manifest.origin.revision).ok_or_else(||contract("selected project source absent"))?;
        self.source_documents(source)
    }
    pub(super) fn source_documents(&self,source:&ProjectSourceClosure)->Result<super::PackageSources,WorkflowError> {
        fn documents(reader:&ProjectBundleReader,records:&[ProjectDocumentMember])->Result<Vec<BTreeMap<String,Vec<u8>>>,WorkflowError> {
            let count=records.iter().map(|d|d.package).max().map_or(0,|n|n+1);
            let mut bundles=vec![BTreeMap::new();count];
            let mut total=0usize;
            for record in records {
                let member=reader.manifest.members.iter().find(|m|m.id==record.member).ok_or_else(||contract("document member absent"))?;
                total=total.checked_add(usize::try_from(member.copied_bytes).map_err(|_|contract("document extent"))?).ok_or_else(||contract("document extent"))?;
                if total>reader.maximum_document_bytes { return Err(contract("source admission document allowance exhausted")); }
                let mut bytes=Vec::new(); reader.open_member(&record.member)?.read_to_end(&mut bytes).map_err(io)?;
                if bundles[record.package].insert(record.path.clone(),bytes).is_some() { return Err(contract("ambiguous project loader path")); }
            }
            Ok(bundles)
        }
        let physical=documents(self,&source.physical_documents)?;
        if physical.len()!=1 { return Err(contract("physical closure must contain one package")); }
        Ok(super::PackageSources{physical:physical.into_iter().next().ok_or_else(||contract("physical package absent"))?,modeling:documents(self,&source.documents)?})
    }
}
pub(super) fn validate_manifest(manifest:&ProjectManifest)->Result<(),WorkflowError> {
    if manifest.members.len()>65536 || manifest.sources.len()>1024 || manifest.sources.is_empty() || !manifest.offline_complete { return Err(contract("incomplete or excessive logical project closure")); }
    let mut ids=BTreeSet::new(); let mut paths=BTreeSet::new();
    for member in &manifest.members {
        if member.id.is_empty() || !ids.insert(member.id.as_str()) || !paths.insert(member.relative_path.as_str()) { return Err(contract("duplicate project member")); }
        contained_copy::safe_relative(&member.relative_path).map_err(io)?;
        if !member.relative_path.starts_with("members/") || !matches!(member.codec.as_str(),"authored-document.v1"|"replay-source.v1"|"arrow-ipc-stream.v1"|"artifact-block.v1"|"diagnostic-review-pack.v2") { return Err(contract("unknown project member codec")); }
        if member.codec=="arrow-ipc-stream.v1" && member.result.is_none() { return Err(contract("result member lacks original qualifiers")); }
        pse_ids::ContentHash::parse_hex(&member.digest).map_err(|e|contract(e.to_string()))?;
    }
    let mut revisions=BTreeSet::new();
    for source in &manifest.sources {
        if !revisions.insert(&source.revision) || !ids.contains(source.replay_source.as_str()) || source.physical_documents.is_empty() { return Err(contract("incomplete or duplicate source closure")); }
        for documents in [&source.documents,&source.physical_documents] {
            let mut names=BTreeSet::new();
            for document in documents {
                contained_copy::safe_relative(&document.path).map_err(io)?;
                if document.package>=1024 || !names.insert((document.package,&document.path)) || !ids.contains(document.member.as_str()) { return Err(contract("missing or ambiguous source document closure")); }
                if manifest.members.iter().find(|m|m.id==document.member).is_none_or(|m|m.codec!="authored-document.v1") { return Err(contract("source document codec differs")); }
            }
        }
    }
    if !revisions.contains(&manifest.origin.revision) { return Err(contract("selected source closure absent")); }
    let executable=!manifest.descriptive_only && !manifest.omissions.iter().any(|o|o.required_for_execution) && manifest.providers.iter().all(|p|p.supported);
    if manifest.executable_complete!=executable { return Err(contract("project execution completeness contradicts selected closure")); }
    for dataset in &manifest.datasets {
        if !revisions.contains(&dataset.revision) || dataset.parent.as_ref().is_some_and(|p|!manifest.datasets.iter().any(|d|&d.key==p)) || !manifest.contexts.iter().any(|c|c.key==dataset.context) { return Err(contract("research dataset ancestry missing")); }
    }
    for selection in &manifest.research_selections {
        if !revisions.contains(&selection.revision) || !manifest.datasets.iter().any(|d|d.key==selection.dataset) || !manifest.contexts.iter().any(|c|c.key==selection.context) { return Err(contract("research selection closure missing")); }
    }
    for publication in &manifest.publications {
        if !revisions.contains(&publication.revision) || !revisions.contains(&publication.parent) || !manifest.research_selections.iter().any(|s|s.key==publication.selection) { return Err(contract("parameter publication ancestry missing")); }
    }
    for study in &manifest.studies { if !revisions.contains(&study.definition.modeling_revision) { return Err(contract("study source closure missing")); } }
    Ok(())
}

impl Runtime {
    /// Copy the selected original closure, verify destination bytes, and publish its sole manifest last.
    pub async fn project_export(&self,destination:&Path,request:&ProjectExportRequest,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ProjectManifest,WorkflowError> {
        let mut closure=self.project_boxed(self.resolve_project_closure(request,physical,cancel))?.await?;
        std::fs::create_dir(destination).map_err(io)?;
        let root=destination.canonicalize().map_err(io)?;
        std::fs::create_dir(root.join("members")).map_err(io)?;
        std::fs::create_dir(root.join(".staging")).map_err(io)?;
        let token=cancel.token();
        let mut total=0u64;
        fn finish_member(root:&Path,manifest:&mut ProjectManifest,total:&mut u64,maximum:u64,token:&pse_columnar::CancellationToken,id:&str,bytes:u64)->Result<(),WorkflowError> {
            *total=total.checked_add(bytes).ok_or_else(||contract("project copied extent"))?;
            if *total>maximum {return Err(contract("project output allowance exhausted"));}
            let m=manifest.members.iter_mut().find(|m|m.id==id).ok_or_else(||contract("unselected project member"))?;
            let staged=root.join(".staging").join(id);
            let (actual,digest)=checksum(File::open(&staged).map_err(io)?,bytes)?;
            if actual!=bytes {return Err(contract("short copied project member"));}
            File::open(&staged).map_err(io)?.sync_all().map_err(io)?;
            token.checkpoint().map_err(pse_engine::EngineError::from)?;
            std::fs::rename(staged,root.join(&m.relative_path)).map_err(io)?;
            m.copied_bytes=bytes;m.digest=digest;Ok(())
        }
        for (id,bytes) in &closure.documents {
            super::study::creation_checkpoint(cancel)?;
            let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(root.join(".staging").join(id)).map_err(io)?;
            contained_copy::copy_exact(&mut bytes.as_slice(),&mut file,bytes.len() as u64,||if token.is_cancelled(){Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"project copy cancelled"))}else{Ok(())}).map_err(io)?;
            file.sync_all().map_err(io)?;drop(file);finish_member(&root,&mut closure.manifest,&mut total,request.maximum_output_bytes,&token,id,bytes.len() as u64)?;
        }
        for (id,reader) in &mut closure.results {
            let staging=root.join(".staging").join(id);
            let remaining=request.maximum_output_bytes.saturating_sub(total);
            let bytes=match reader {
                super::project_closure::ResultMember::Full(r)=>r.export_ipc_bounded(&staging,remaining,&token).await?,
                super::project_closure::ResultMember::Selected(r)=>r.export_ipc_bounded(&staging,remaining,&token).await?,
            };
            finish_member(&root,&mut closure.manifest,&mut total,request.maximum_output_bytes,&token,id,bytes)?;
        }
        for (artifact_index,read) in closure.artifacts.iter().enumerate() {
            for ordinal in 0..read.header().block_count {
                super::study::creation_checkpoint(cancel)?;
                let block=read.block(ordinal).await?;
                let id=format!("artifact-{artifact_index}-{ordinal}");
                let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(root.join(".staging").join(&id)).map_err(io)?;
                contained_copy::copy_exact(&mut block.payload.as_slice(),&mut file,block.payload.len() as u64,||if token.is_cancelled(){Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"project copy cancelled"))}else{Ok(())}).map_err(io)?;
                file.sync_all().map_err(io)?;drop(file);finish_member(&root,&mut closure.manifest,&mut total,request.maximum_output_bytes,&token,&id,block.payload.len() as u64)?;
            }
        }
        closure.renew().await?;
        closure.manifest.offline_complete=true;
        validate_manifest(&closure.manifest)?;
        // Reopen every published contained member and verify the destination while all original grants remain owned.
        let directory=File::open(&root).map_err(io)?;
        for m in &closure.manifest.members {
            let (bytes,digest)=checksum(contained_copy::open_contained(&directory,&m.relative_path).map_err(io)?,m.copied_bytes)?;
            if bytes!=m.copied_bytes || digest!=m.digest {return Err(contract("destination project member verification failed"));}
        }
        let bytes=encode(&closure.manifest,request.maximum_document_bytes)?;
        closure.renew().await?;super::study::creation_checkpoint(cancel)?;
        contained_copy::publish_document(&root,".project-manifest","project.json",&bytes,||if token.is_cancelled(){Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"project publication cancelled"))}else{Ok(())}).map_err(io)?;
        Ok(closure.manifest.clone())
    }
}

/// Receiving placement and preserved origin, without imported historical execution receipts.
#[derive(Clone,Debug,Serialize,Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectImportReport {
    /// Fresh receiving project.
    pub project:String,
    /// Fresh writable receiving branch.
    pub branch:String,
    /// Fresh receiving source problem.
    pub problem:String,
    /// Fresh receiving immutable source revision.
    pub revision:String,
    /// Original source attribution, preserved separately from receiving ownership.
    pub origin:ProjectSelection,
    /// Exact imported manifest checksum.
    pub bundle_digest:String,
    /// Current project-owned copied evidence; independent of original pack lifetime.
    pub evidence_artifacts:Vec<String>,
    /// Source effect has acknowledged; this alone is not project availability.
    pub source_admitted:bool,
    /// Exact artifact membership finished and project is publicly available.
    pub project_complete:bool,
}
const MAXIMUM_IMPORT_BYTES:usize=256<<20;
impl ProjectBundleReader {
    fn manifest_bytes(&self)->Result<Vec<u8>,WorkflowError> {
        let mut bytes=Vec::new();contained_copy::open_contained(&self.directory,"project.json").map_err(io)?.take(self.maximum_document_bytes as u64+1).read_to_end(&mut bytes).map_err(io)?;
        if bytes.len()>self.maximum_document_bytes || pse_ids::encoding_checksum(&bytes).content_hash().to_hex()!=self.digest {return Err(contract("project manifest changed after inspection"));}
        Ok(bytes)
    }
    fn archive_extent(&self)->Result<usize,WorkflowError> {
        self.manifest.members.iter().try_fold(self.manifest_bytes()?.len().saturating_add(8),|total,m|total.checked_add(usize::try_from(m.copied_bytes).map_err(|_|contract("import member extent"))?).filter(|n|*n<=MAXIMUM_IMPORT_BYTES).ok_or_else(||contract("import closure exceeds finite owned artifact allowance")))
    }
    fn archive(&self,cancel:&crate::CancelSource)->Result<Vec<u8>,WorkflowError> {
        let manifest=self.manifest_bytes()?;
        let mut bytes=Vec::with_capacity(self.archive_extent()?);
        bytes.extend_from_slice(&(manifest.len() as u64).to_le_bytes());bytes.extend_from_slice(&manifest);
        for member in &self.manifest.members {
            super::study::creation_checkpoint(cancel)?;
            let start=bytes.len();
            let mut source=self.open_member(&member.id)?;
            contained_copy::copy_exact(&mut source,&mut bytes,member.copied_bytes,||if cancel.token().is_cancelled(){Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"import copy cancelled"))}else{Ok(())}).map_err(io)?;
            if pse_ids::encoding_checksum(&bytes[start..]).content_hash().to_hex()!=member.digest {return Err(contract("consumed import member digest differs"));}
        }
        Ok(bytes)
    }
    fn replay_source(&self)->Result<super::ReplaySourceDocument,WorkflowError> {
        let selected=self.manifest.sources.iter().find(|s|s.revision==self.manifest.origin.revision).ok_or_else(||contract("selected source absent"))?;
        decode(self.open_member(&selected.replay_source)?,self.maximum_document_bytes)
    }
    fn physical_documents(&self)->Result<BTreeMap<String,Vec<u8>>,WorkflowError> {
        let selected=self.manifest.sources.iter().find(|s|s.revision==self.manifest.origin.revision).ok_or_else(||contract("selected source absent"))?;
        let sources=self.source_documents(selected)?;Ok(sources.physical)
    }
}
fn import_report(row:&pse_operations::canonical_project_imports::ProjectImport)->Result<ProjectImportReport,WorkflowError> {
    let origin:ProjectSelection=serde_json::from_slice(row.origin.as_slice()).map_err(|e|contract(format!("import origin codec: {e}")))?;
    let artifact=row.artifact.as_ref().map(|b|serde_json::from_slice::<pse_operations::canonical_artifacts::Artifact>(b.as_slice()).map_err(|e|contract(e.to_string()))).transpose()?;
    Ok(ProjectImportReport{project:row.project.clone(),branch:row.branch.clone(),problem:row.problem.clone(),revision:row.revision.clone(),origin,bundle_digest:row.bundle_digest.clone(),evidence_artifacts:artifact.into_iter().map(|a|a.key).collect(),source_admitted:true,project_complete:row.complete})
}
impl Runtime {
    /// Issue one immutable receiving intent with fresh project, branch, source and operation IDs.
    pub async fn project_import_issue(&self,bundle:&ProjectBundleReader,title:&str,branch_name:&str)->Result<ProjectImportIntent,WorkflowError> {
        let _=bundle.manifest_bytes()?;
        if !bundle.inspect().executable_complete {return Err(contract("bundle is descriptive or has unsupported executable closure"));}
        let store=self.canonical_store()?;
        let fresh=||pse_operations::mint_id::<pse_ids::SemanticId>().to_string();
        Ok(store.project_import_intent(&fresh(),&fresh(),&fresh(),&fresh(),bundle.digest(),encode(&bundle.inspect().origin,131072)?,title,branch_name)?)
    }
    /// Observe the original import checkpoint without making a new effect or consulting current heads.
    pub async fn project_import_status(&self,intent:&ProjectImportIntent)->Result<Option<ProjectImportReport>,WorkflowError> {
        self.canonical_store()?.project_import_record(intent).await?.as_ref().map(import_report).transpose()
    }
    /// Stage copied bytes and ordinary admission, then settle this exact guarded source/artifact activation.
    pub async fn project_import(&self,bundle:&ProjectBundleReader,intent:&ProjectImportIntent,cancel:&crate::CancelSource)->Result<ProjectImportReport,WorkflowError> {
        intent.validate()?;
        if bundle.digest()!=intent.bundle_digest() {return Err(contract("import intent bundle digest differs"));}
        let store=self.canonical_store()?;
        // Settle the original source effect before any mutable preflight or renewed artifact intent.
        let prior=store.project_import_record(intent).await?;
        if let Some(row)=prior.as_ref().filter(|row|row.complete) {return import_report(row);}
        let archive_extent=bundle.archive_extent()?;
        let _archive_owner=self.shared.math().reserve("project:import-copied-closure",archive_extent.checked_mul(3).ok_or_else(||contract("import archive extent"))?)?;
        let archive=bundle.archive(cancel)?;
        let revision=if let Some(row)=&prior {
            store.revision(&row.revision).await?.ok_or_else(||contract("original import source unavailable"))?
        } else {
            if !bundle.inspect().executable_complete {return Err(contract("unsupported or incomplete executable project closure"));}
            let local=self.with_durability(super::DurabilitySelection::Ephemeral)?;
            let physical=self.project_boxed(local.physical_from_sources(&bundle.physical_documents()?,cancel))?.await?;
            let replay=bundle.replay_source()?;
            let admitted=replay.reconstruct(&local,physical,BTreeMap::new(),cancel)?;
            let package=self.project_boxed(admitted.publish_portable_import(self,intent,cancel))?.await?;
            package.canonical_revision().cloned().ok_or_else(||contract("import source was not recorded"))?
        };
        let row=store.project_import_record(intent).await?.ok_or_else(||contract("import source checkpoint unavailable"))?;
        let chunk=pse_operations::canonical_files::FILE_TARGET_BYTES;
        let provenance=encode(&(1u8,intent.bundle_digest()),131072)?;
        let mut header=if let Some(bytes)=&row.artifact {
            serde_json::from_slice::<pse_operations::canonical_artifacts::Artifact>(bytes.as_slice()).map_err(|e|contract(format!("import artifact intent codec: {e}")))?
        } else {
            store.new_artifact(&revision,"project-import-evidence",provenance.clone(),archive.len().div_ceil(chunk) as u64,archive.len() as u64).await?
        };
        let blocks=archive.chunks(chunk).enumerate().map(|(ordinal,bytes)|pse_operations::canonical_artifacts::artifact_block(&header.key,ordinal as u64,"project-import-evidence",bytes.to_vec(),intent.bundle_digest(),0)).collect::<Vec<_>>();
        let original=header.clone();
        pse_operations::canonical_artifacts::seal_artifact_request(&mut header,std::slice::from_ref(&revision),&[],&blocks,"project",intent.operation());
        if row.artifact.is_some() && header!=original {return Err(contract("original import artifact intent differs"));}
        if header.provenance.as_slice()!=provenance || header.revision!=revision.key || header.payload_bytes!=archive.len() as u64 || header.block_count!=blocks.len() as u64 {return Err(contract("import artifact membership differs"));}
        store.project_import_bind_artifact(intent,&header).await?;
        // Original header is durable before copy. A lost copy/activation reply retries these same keys.
        store.persist_artifact(&header,std::slice::from_ref(&revision),&[],&blocks,"project",intent.operation()).await?;
        let completed=store.finish_project_import(intent).await?;
        import_report(&completed)
    }
    /// Reopen current copied local source and physical closure after completed import, without the supplied pack.
    pub async fn project_import_open(&self,intent:&ProjectImportIntent,cancel:&crate::CancelSource)->Result<super::ModelingPackage,WorkflowError> {
        let store=self.canonical_store()?;
        let row=store.project_import_record(intent).await?.filter(|r|r.complete).ok_or_else(||contract("project import membership is incomplete"))?;
        let header:pse_operations::canonical_artifacts::Artifact=serde_json::from_slice(row.artifact.as_ref().ok_or_else(||contract("import copied evidence absent"))?.as_slice()).map_err(|e|contract(e.to_string()))?;
        let read=store.read_artifact(&header.key).await?;
        let extent=usize::try_from(read.header().payload_bytes).map_err(|_|contract("import artifact extent"))?;
        if extent>MAXIMUM_IMPORT_BYTES {return Err(contract("import artifact exceeds finite reopen allowance"));}
        let _owner=self.shared.math().reserve("project:import-reopen",extent.checked_mul(3).ok_or_else(||contract("import reopen extent"))?)?;
        let mut archive=Vec::with_capacity(extent);
        for ordinal in 0..read.header().block_count {super::study::creation_checkpoint(cancel)?;archive.extend_from_slice(read.block(ordinal).await?.payload.as_slice());}
        let (manifest,members)=decode_archive(&archive,intent.bundle_digest())?;
        let source=manifest.sources.iter().find(|s|s.revision==manifest.origin.revision).ok_or_else(||contract("import source closure absent"))?;
        let mut physical_documents=BTreeMap::new();
        for document in &source.physical_documents {physical_documents.insert(document.path.clone(),members.get(document.member.as_str()).ok_or_else(||contract("import physical member absent"))?.to_vec());}
        let physical=self.project_boxed(self.physical_from_sources(&physical_documents,cancel))?.await?;
        let revision=store.project_branch_revision(&row.branch).await?.ok_or_else(||contract("imported current branch source unavailable"))?;
        self.project_boxed(self.modeling_revision(revision,physical,BTreeMap::new()))?.await
    }
}
fn decode_archive<'a>(archive:&'a [u8],digest:&str)->Result<(ProjectManifest,BTreeMap<String,&'a [u8]>),WorkflowError> {
    let length=u64::from_le_bytes(archive.get(..8).ok_or_else(||contract("short imported archive"))?.try_into().map_err(|_|contract("invalid archive frame"))?);
    let length=usize::try_from(length).map_err(|_|contract("archive manifest extent"))?;
    if length>64<<20 {return Err(contract("archive manifest allowance exhausted"));}
    let mut position=8usize.checked_add(length).ok_or_else(||contract("archive manifest extent"))?;
    let manifest_bytes=archive.get(8..position).ok_or_else(||contract("incomplete archive manifest"))?;
    if pse_ids::encoding_checksum(manifest_bytes).content_hash().to_hex()!=digest {return Err(contract("imported archive manifest digest differs"));}
    let manifest:ProjectManifest=serde_json::from_slice(manifest_bytes).map_err(|e|contract(e.to_string()))?;
    validate_manifest(&manifest)?;
    let mut members=BTreeMap::new();
    for member in &manifest.members {
        let end=position.checked_add(usize::try_from(member.copied_bytes).map_err(|_|contract("archive member extent"))?).ok_or_else(||contract("archive member extent"))?;
        let bytes=archive.get(position..end).ok_or_else(||contract("incomplete imported archive member"))?;
        if pse_ids::encoding_checksum(bytes).content_hash().to_hex()!=member.digest {return Err(contract("imported archive member digest differs"));}
        members.insert(member.id.clone(),bytes);position=end;
    }
    if position!=archive.len() {return Err(contract("unselected trailing imported archive bytes"));}
    Ok((manifest,members))
}
