// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Raw project draft custody and its explicit bridge to ordinary source admission.
use super::{ModelingPackage, PhysicalContext, Runtime, WorkflowError, contract};
use pse_operations::{
    canonical_artifacts::{artifact_block, seal_artifact_request},
    canonical_projects::{Branch, Project, ProjectAcknowledgement, ProjectPublication},
};
use std::collections::BTreeMap;
fn draft_deadline(cancel:Option<&crate::CancelSource>)->std::time::Instant{
    let ordinary=std::time::Instant::now()+pse_operations::canonical::REQUEST_TIMEOUT;
    cancel.and_then(crate::CancelSource::deadline).map_or(ordinary,|d|d.min(ordinary))
}
fn draft_checkpoint(cancel:&crate::CancelSource)->Result<(),WorkflowError>{
    cancel.token().checkpoint().map_err(pse_engine::EngineError::from)?;
    if cancel.deadline().is_some_and(|d|std::time::Instant::now()>=d){return Err(contract("draft original deadline expired"));}Ok(())
}
/// Explicit raw draft document codec. Saving arbitrary bytes does not require this shape;
/// only scientific publication decodes it and invokes the ordinary package loader.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDraftDocuments {
    /// This explicit original-byte container's version.
    pub version: pse_model::document::Version<1>,
    /// Complete packages in original package order, with original relative paths/bytes.
    pub packages: Vec<BTreeMap<String, Vec<u8>>>,
}
/// Exact-source typed draft edits. Reconstruction checks scientific and physical identity
/// before rebinding the original process-local parent token.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDraftDelta {
    /// Draft codec version.
    pub version:pse_model::document::Version<1>,
    /// Canonical authority that issued the exact original base.
    pub issuing_store:String,
    /// Exact durable parent; content equality cannot substitute another revision.
    pub base_revision:String,
    /// Exact immutable source and physical premises.
    pub source:super::OperationSource,
    /// Original local parent token captured with the edits.
    pub parent_local:String,
    /// Existing typed source edits; omissions inherit.
    pub delta:super::SourceDelta,
}
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum DraftContent { Documents(ProjectDraftDocuments), Delta(ProjectDraftDelta) }
/// One explicit navigation reference; a writable branch head and bookmark have different owners.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum ProjectReference {
    /// Select a branch's current exact source in a short snapshot.
    Branch { #[doc="Stable branch identity."] key:String },
    /// Select an immutable bookmark.
    Bookmark { #[doc="Stable bookmark identity."] key:String },
}
/// Explicit navigation command transport.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSelectRequest {
    /// Codec version.
    pub version:pse_model::document::Version<1>,
    /// Exact navigation owner.
    pub reference:ProjectReference,
}
/// Retain this selection to reopen the same science after a branch head or label changes.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSelection {
    /// Canonical installation incarnation.
    pub issuing_store:String,
    /// Stable project identity, independent of presentation labels.
    pub project:String,
    /// Original navigation owner; this does not mean its current head is still selected.
    pub reference:ProjectReference,
    /// Exact source problem owner.
    pub problem:String,
    /// Exact immutable accepted source revision.
    pub revision:String,
    /// Admitted scientific source identity; the exact revision additionally protects fits/provenance.
    pub source_identity:pse_ids::roles::SourceRevisionHash,
    /// Complete admitted physical closure identity, not a mutable package name.
    pub physical_context:pse_ids::ContentHash,
}
impl Runtime {
    /// Select one exact branch/bookmark source without preparing any mathematical products.
    pub async fn project_select(&self,request:&ProjectSelectRequest,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ProjectSelection,WorkflowError>{
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(Some(cancel)),async{
            draft_checkpoint(cancel)?;let store=self.canonical_store()?;
            let (project,revision)=match &request.reference {
                ProjectReference::Branch{key}=>{let branch=store.project_branch(key).await?.ok_or_else(||contract("project branch unavailable"))?;let revision=store.project_branch_revision(key).await?.ok_or_else(||contract("project branch source unavailable"))?;if revision.problem!=branch.problem{return Err(contract("branch source owner differs"));}(branch.project,revision)},
                ProjectReference::Bookmark{key}=>{let bookmark=store.project_bookmark(key).await?.ok_or_else(||contract("project bookmark unavailable"))?;let revision=store.revision(&bookmark.revision).await?.ok_or_else(||contract("bookmarked source unavailable"))?;(bookmark.project,revision)},
            };
            let package=self.modeling_revision(revision.clone(),physical,BTreeMap::new()).await?;let source=super::OperationSource::of(&package);
            Ok(ProjectSelection{issuing_store:store.issuing_store()?.into(),project,reference:request.reference.clone(),problem:revision.problem,revision:revision.key,source_identity:source.revision,physical_context:source.physical_context})
        })).await
    }
    /// Open the retained exact source; later branch heads and presentation labels never retarget it.
    pub async fn project_open_selection(&self,selection:&ProjectSelection,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(Some(cancel)),async{
            draft_checkpoint(cancel)?;let store=self.canonical_store()?;
            if selection.issuing_store!=store.issuing_store()? || physical.identity()!=selection.physical_context{return Err(contract("project selection authority or physical closure differs"));}
            match &selection.reference{
                ProjectReference::Branch{key}=>{let branch=store.project_branch(key).await?.ok_or_else(||contract("project branch unavailable"))?;if branch.project!=selection.project||branch.problem!=selection.problem{return Err(contract("project selection branch owner differs"));}},
                ProjectReference::Bookmark{key}=>{let bookmark=store.project_bookmark(key).await?.ok_or_else(||contract("project bookmark unavailable"))?;if bookmark.project!=selection.project||bookmark.revision!=selection.revision{return Err(contract("project selection bookmark differs"));}},
            }
            let revision=store.revision(&selection.revision).await?.ok_or_else(||contract("selected source unavailable"))?;if revision.problem!=selection.problem{return Err(contract("selected source problem differs"));}
            let package=self.modeling_revision(revision,physical,BTreeMap::new()).await?;if super::OperationSource::of(&package).revision!=selection.source_identity{return Err(contract("selected admitted source differs"));}Ok(package)
        })).await
    }
}
/// Original, read-only issued large-draft custody intent. Preserve this value unchanged
/// until exact project and artifact receipts settle; it grants no scientific execution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDraftSaveIntent {
    issuing_store: String,
    operation: String,
    project: Project,
    branch: Branch,
    key: String,
    base: String,
    expected: Option<u64>,
    digest: String,
    bytes: u64,
    artifact: Option<pse_operations::canonical_artifacts::Artifact>,
}
impl Runtime {
    /// Read-only issue of the original immutable raw-draft custody effect.
    pub async fn project_draft_save_intent(
        &self,
        operation: &str,
        project: &Project,
        branch: &Branch,
        key: &str,
        base: &str,
        expected: Option<u64>,
        content: &[u8],
    ) -> Result<ProjectDraftSaveIntent, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(None), async {
            let store = self.canonical_store()?;
            if branch.project != project.key || operation.is_empty() || key.is_empty() {
                return Err(contract("draft issuing context differs"));
            }
            let digest = pse_operations::canonical_execution::result_payload_digest(content);
            let artifact = if content.len() > 128 * 1024 {
                let _lease = self.shared.math().reserve(
                    "project:draft-issue",
                    content
                        .len()
                        .checked_mul(2)
                        .ok_or_else(|| contract("draft byte extent"))?,
                )?;
                let revision = store
                    .revision(base)
                    .await?
                    .ok_or_else(|| contract("draft source base unavailable"))?;
                if revision.problem != branch.problem {
                    return Err(contract("draft source belongs to another branch"));
                }
                let chunk = pse_operations::canonical_files::FILE_TARGET_BYTES;
                let mut header = store
                    .new_artifact(
                        &revision,
                        "project-draft",
                        Vec::new(),
                        content.len().div_ceil(chunk) as u64,
                        content.len() as u64,
                    )
                    .await?;
                let blocks = content
                    .chunks(chunk)
                    .enumerate()
                    .map(|(ordinal, bytes)| {
                        artifact_block(
                            &header.key,
                            ordinal as u64,
                            "project-draft",
                            bytes.to_vec(),
                            "",
                            0,
                        )
                    })
                    .collect::<Vec<_>>();
                seal_artifact_request(
                    &mut header,
                    std::slice::from_ref(&revision),
                    &[],
                    &blocks,
                    "project",
                    key,
                );
                Some(header)
            } else {
                None
            };
            Ok(ProjectDraftSaveIntent {
                issuing_store: store.issuing_store()?.into(),
                operation: operation.into(),
                project: project.clone(),
                branch: branch.clone(),
                key: key.into(),
                base: base.into(),
                expected,
                digest,
                bytes: content.len() as u64,
                artifact,
            })
        }))
        .await
    }
    /// Save only the caller's original issued body/coordinates; no new artifact is minted on retry.
    pub async fn project_save_draft_intent(
        &self,
        intent: &ProjectDraftSaveIntent,
        content: &[u8],
    ) -> Result<ProjectAcknowledgement, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(None), async {
            let store = self.canonical_store()?;
            if store.issuing_store()? != intent.issuing_store
                || content.len() as u64 != intent.bytes
                || pse_operations::canonical_execution::result_payload_digest(content)
                    != intent.digest
            {
                return Err(pse_operations::canonical::CanonicalError::OperationReused.into());
            }
            if let Some(header) = &intent.artifact {
                let _lease = self.shared.math().reserve(
                    "project:draft-custody",
                    content
                        .len()
                        .checked_mul(3)
                        .ok_or_else(|| contract("draft byte extent"))?,
                )?;
                let revision = store
                    .revision(&intent.base)
                    .await?
                    .ok_or_else(|| contract("draft source base unavailable"))?;
                let blocks = content
                    .chunks(pse_operations::canonical_files::FILE_TARGET_BYTES)
                    .enumerate()
                    .map(|(ordinal, bytes)| {
                        artifact_block(
                            &header.key,
                            ordinal as u64,
                            "project-draft",
                            bytes.to_vec(),
                            "",
                            0,
                        )
                    })
                    .collect::<Vec<_>>();
                let mut sealed = header.clone();
                seal_artifact_request(
                    &mut sealed,
                    std::slice::from_ref(&revision),
                    &[],
                    &blocks,
                    "project",
                    &intent.key,
                );
                if sealed.creation_request_digest != header.creation_request_digest
                    || header.kind != "project-draft"
                    || header.revision != intent.base
                    || header.primary_problem != intent.branch.problem
                    || header.payload_bytes != content.len() as u64
                    || header.block_count != blocks.len() as u64
                    || header.active
                    || header.retiring
                {
                    return Err(pse_operations::canonical::CanonicalError::OperationReused.into());
                }
                // A settled metadata receipt does not authenticate a caller-edited artifact
                // header. Compare the complete original immutable descriptor at its key.
                if let Some(ack) = store.project_acknowledgement(&intent.operation).await? {
                    let actual = store.artifact(&header.key).await?;
                    let mut expected = header.clone();
                    expected.active = true;
                    if actual != expected {
                        return Err(
                            pse_operations::canonical::CanonicalError::OperationReused.into()
                        );
                    }
                    if ack.kind != "draft"
                        || ack.project != intent.project.key
                        || ack.branch.as_deref() != Some(intent.branch.key.as_str())
                    {
                        return Err(
                            pse_operations::canonical::CanonicalError::OperationReused.into()
                        );
                    }
                    return Ok(store
                        .save_project_draft(
                            &intent.operation,
                            &intent.project,
                            &intent.branch,
                            &intent.key,
                            &intent.base,
                            intent.expected,
                            &[],
                            Some(&header.key),
                        )
                        .await?);
                }
                store
                    .persist_artifact(
                        header,
                        std::slice::from_ref(&revision),
                        &[],
                        &blocks,
                        "project",
                        &intent.key,
                    )
                    .await?;
                Ok(store
                    .save_project_draft(
                        &intent.operation,
                        &intent.project,
                        &intent.branch,
                        &intent.key,
                        &intent.base,
                        intent.expected,
                        &[],
                        Some(&header.key),
                    )
                    .await?)
            } else {
                Ok(store
                    .save_project_draft(
                        &intent.operation,
                        &intent.project,
                        &intent.branch,
                        &intent.key,
                        &intent.base,
                        intent.expected,
                        content,
                        None,
                    )
                    .await?)
            }
        }))
        .await
    }
    /// Single-effect bounded inline save; larger bodies require the retained original intent API.
    pub async fn project_save_draft(
        &self,
        operation: &str,
        project: &Project,
        branch: &Branch,
        key: &str,
        base: &str,
        expected: Option<u64>,
        content: &[u8],
    ) -> Result<ProjectAcknowledgement, WorkflowError> {
        if content.len() > 128 * 1024 {
            return Err(contract(
                "large draft save requires project_draft_save_intent and project_save_draft_intent",
            ));
        }
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(None), async {
            Ok(self
                .canonical_store()?
                .save_project_draft(
                    operation, project, branch, key, base, expected, content, None,
                )
                .await?)
        }))
        .await
    }
    /// Hydrate only this saved raw body through original artifact/file protection owners.
    pub async fn project_draft_bytes(
        &self,
        key: &str,
        cancel:&crate::CancelSource,
    ) -> Result<pse_columnar::Leased<[u8]>, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(Some(cancel)), async {
            draft_checkpoint(cancel)?;
            let draft = self
                .canonical_store()?
                .project_draft(key)
                .await?
                .ok_or_else(|| contract("project draft unavailable"))?;
            self.hydrate_selected_draft(&draft.version,cancel).await
        }))
        .await
    }
    // Selection and hydration have deliberately separate lifetimes. Never reread a
    // mutable draft key after a caller selected the exact descriptor/generation.
    async fn hydrate_selected_draft(
        &self,
        draft: &pse_operations::canonical_projects::DraftVersion,
        cancel:&crate::CancelSource,
    ) -> Result<pse_columnar::Leased<[u8]>, WorkflowError> {
        draft_checkpoint(cancel)?;
        let store = self.canonical_store()?;
        let Some(artifact) = draft.artifact.as_deref() else {
            let lease = self.shared.math().reserve(
                "project:inline-draft-read",
                draft
                    .content
                    .len()
                    .checked_mul(2)
                    .ok_or_else(|| contract("draft byte extent"))?,
            )?;
            return Ok(pse_columnar::Leased::new(
                std::sync::Arc::<[u8]>::from(draft.content.as_slice()),
                lease,
            ));
        };
        let read = store.read_artifact(artifact).await?;
        if read.header().kind != "project-draft" || read.header().revision != draft.base_revision {
            return Err(contract("project draft artifact context differs"));
        }
        let size = usize::try_from(read.header().payload_bytes)
            .map_err(|_| contract("draft body extent"))?;
        let lease = self.shared.math().reserve(
            "project:draft-read",
            size.checked_mul(2)
                .ok_or_else(|| contract("draft body extent"))?,
        )?;
        let mut result = Vec::with_capacity(size);
        let hydrated=async { for ordinal in 0..read.header().block_count {
            draft_checkpoint(cancel)?;
            let block = read.block(ordinal).await?;
            if block.relation != "project-draft" {
                return Err(contract("draft block codec differs"));
            }
            result.extend_from_slice(block.payload.as_slice());
        } Ok::<_,WorkflowError>(()) }.await;
        let released=read.release().await;
        hydrated?; released?;
        if result.len() != size {
            return Err(contract("draft body coverage differs"));
        }
        Ok(pse_columnar::Leased::new(
            std::sync::Arc::<[u8]>::from(result),
            lease,
        ))
    }
    /// Capture recoverable typed edits against one exact currently admitted parent.
    pub fn project_draft_delta(&self,base:&ModelingPackage,delta:super::SourceDelta)->Result<ProjectDraftDelta,WorkflowError>{
        if delta.parent!=base.revision_id(){return Err(contract("draft edits name a different local parent"));}
        let issuing=self.canonical_store()?.issuing_store()?;
        if base.runtime_store_issuer()?!=issuing{return Err(contract("draft parent issuing authority differs"));}
        let revision=base.canonical_revision().ok_or_else(||contract("typed project draft requires an exact durable parent"))?;
        Ok(ProjectDraftDelta{version:pse_model::document::Version,issuing_store:issuing.into(),base_revision:revision.key.clone(),source:super::OperationSource::of(base),parent_local:base.revision_id().into(),delta})
    }
    /// Authenticate and admit one immutable version locally, without publishing a branch head.
    pub async fn project_admit_draft_version(&self,version:&pse_operations::canonical_projects::DraftVersion,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(Some(cancel)), async{
            draft_checkpoint(cancel)?;
            let store=self.canonical_store()?;
            let actual=store.project_draft_version(&version.key).await?.ok_or_else(||contract("draft version unavailable"))?;
            if actual!=*version{return Err(contract("draft version descriptor differs"));}
            let base=store.revision(&version.base_revision).await?.ok_or_else(||contract("draft source base unavailable"))?;
            let bytes=self.hydrate_selected_draft(version,cancel).await?;
            let _lease=self.shared.math().reserve("project:draft-decode",bytes.len().checked_mul(4).ok_or_else(||contract("draft document extent"))?)?;
            let content:DraftContent=serde_json::from_slice(&bytes).map_err(|e|contract(format!("project draft document codec: {e}")))?;
            match content {
                DraftContent::Documents(content)=>{
                    // Authenticate the selected admitted base/physical closure before loading replacement documents.
                    let _base=self.modeling_revision(base,physical.clone(),BTreeMap::new()).await?;
                    let pool=self.shared.pool();let token=cancel.token();let validation=self.admission_context();
                    let bundles=content.packages.iter().map(|documents|pse_admission::authoring_driver::document::load_package_documents_owned(documents,self.registry(),pse_authoring::ParseBudget::default(),&pool,&token,&validation)).collect::<Result<Vec<_>,_>>()?;
                    let documents=pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(bundles,&pool,&token)?;
                    self.project_admit_documents_locally(&documents,physical,cancel).await
                },
                DraftContent::Delta(content)=>{
                    if content.issuing_store!=store.issuing_store()? || content.base_revision!=version.base_revision || content.delta.parent!=content.parent_local{return Err(contract("draft original parent token differs"));}
                    let mut base=self.modeling_revision(base,physical,BTreeMap::new()).await?;
                    self.restore_project_context(&mut base,content.source.context.as_ref()).await?;
                    base.construct_study_variant(&content.source,content.delta,cancel).await
                }
            }
        })).await
    }
    /// Explicit local admission, retaining version-linked diagnostics even for malformed text.
    pub async fn project_admit_draft(&self,operation:&str,key:&str,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(Some(cancel)), async{
            draft_checkpoint(cancel)?;let store=self.canonical_store()?;let draft=store.project_draft(key).await?.ok_or_else(||contract("draft unavailable"))?;
            if draft.deleted{return Err(contract("draft is soft deleted"));}
            let result=self.project_admit_draft_version(&draft.version,physical,cancel).await;
            let message=match &result {Ok(_)=>"admitted locally; branch is unchanged".into(),Err(error)=>error.to_string()};
            store.project_record_diagnostic(operation,&draft.version.key,&message,result.is_ok()).await?;
            result
        })).await
    }
    /// Decode and admit the exact saved generation; publish only after scientific admission succeeds.
    pub async fn project_publish_draft(
        &self,
        intent: &ProjectPublication,
        physical: PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingPackage, WorkflowError> {
        self.scope_activity(pse_operations::canonical::operation_scope(draft_deadline(Some(cancel)), async {
            intent.validate()?;
            draft_checkpoint(cancel)?;
            let (key, generation) = intent.draft().ok_or_else(|| {
                contract("draft publication requires its original saved generation")
            })?;
            let store = self.canonical_store()?;
            if let Some(revision) = store.settled_project_publication(intent).await? {
                return self
                    .modeling_revision(revision, physical, BTreeMap::new())
                    .await;
            }
            let saved = store
                .project_draft(key)
                .await?
                .ok_or_else(|| contract("project draft unavailable"))?;
            if saved.generation != generation {
                // A publication may have committed between the first receipt observation
                // and the later draft edit. Never turn that accepted effect into rejection.
                if let Some(revision) = store.settled_project_publication(intent).await? {
                    return self
                        .modeling_revision(revision, physical, BTreeMap::new())
                        .await;
                }
                return Err(contract("project draft generation changed"));
            }
            let admitted=self.project_admit_draft_version(&saved.version,physical,cancel).await?;
            self.project_publish_admitted(intent,&admitted,cancel).await
        }))
        .await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "owned canonical project fixture assertions require exact acknowledged effects"
    )]
    use super::*;
    #[tokio::test]
    async fn project_large_draft_ack_rejects_coherently_forged_original_artifact_header() {
        let runtime = crate::workflow::tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let source = store.edit("original", None, "source", &[]).await.unwrap();
        store
            .create_project("create", "project", "Engineering")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let fork = store
            .project_fork_intent(&project, "main", "branch-problem", &source, "fork")
            .unwrap();
        let revision = store.edit_project(&fork, &[]).await.unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch("main").await.unwrap().unwrap();
        let body = vec![b'a'; 200_000];
        let original = runtime
            .project_draft_save_intent(
                "save",
                &project,
                &branch,
                "draft",
                &revision.key,
                None,
                &body,
            )
            .await
            .unwrap();
        runtime
            .project_save_draft_intent(&original, &body)
            .await
            .unwrap();
        // Research artifact custody uses its own reference identity, not a draft field.
        store.retain_research_reference("retain-artifact",&project,"research-artifact",&revision.key,None,Some(&original.artifact.as_ref().unwrap().key)).await.unwrap();
        let stopped=crate::CancelSource::new();stopped.cancel();
        assert!(runtime.project_draft_bytes("draft",&stopped).await.is_err());
        assert!(runtime.project_admit_draft_version(&store.project_draft_version("save").await.unwrap().unwrap(),crate::workflow::tests::physical(),&stopped).await.is_err());
        assert_eq!(runtime.project_draft_bytes("draft",&crate::CancelSource::new()).await.unwrap().len(),body.len());
        let changed = vec![b'b'; 220_000];
        let mut forged = original.clone();
        forged.digest = pse_operations::canonical_execution::result_payload_digest(&changed);
        forged.bytes = changed.len() as u64;
        let header = forged.artifact.as_mut().unwrap();
        header.payload_bytes = changed.len() as u64;
        header.block_count = changed
            .len()
            .div_ceil(pse_operations::canonical_files::FILE_TARGET_BYTES)
            as u64;
        let blocks = changed
            .chunks(pse_operations::canonical_files::FILE_TARGET_BYTES)
            .enumerate()
            .map(|(ordinal, bytes)| {
                artifact_block(
                    &header.key,
                    ordinal as u64,
                    "project-draft",
                    bytes.to_vec(),
                    "",
                    0,
                )
            })
            .collect::<Vec<_>>();
        seal_artifact_request(
            header,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "project",
            "draft",
        );
        assert!(matches!(
            runtime.project_save_draft_intent(&forged, &changed).await,
            Err(WorkflowError::Canonical(
                pse_operations::canonical::CanonicalError::OperationReused
            ))
        ));
        assert_eq!(
            &*runtime.project_draft_bytes("draft",&crate::CancelSource::new()).await.unwrap(),
            body.as_slice()
        );
        assert_eq!(
            runtime
                .project_save_draft_intent(&original, &body)
                .await
                .unwrap()
                .key,
            "save"
        );
        let selected = store.project_draft("draft").await.unwrap().unwrap();
        runtime
            .project_save_draft(
                "advance",
                &project,
                &branch,
                "draft",
                &revision.key,
                Some(1),
                b"later malformed [[",
            )
            .await
            .unwrap();
        assert_eq!(
            &*runtime.hydrate_selected_draft(&selected.version,&crate::CancelSource::new()).await.unwrap(),
            body.as_slice(),
            "hydration must consume the selected immutable descriptor, never reread the mutable draft key"
        );
        assert_eq!(
            &*runtime.project_draft_bytes("draft",&crate::CancelSource::new()).await.unwrap(),
            b"later malformed [["
        );
    }
}
