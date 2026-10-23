// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Raw project draft custody and its explicit bridge to ordinary source admission.
use super::{Runtime,WorkflowError,PhysicalContext,ModelingPackage,contract};
use pse_operations::{canonical_projects::{Project,Branch,ProjectAcknowledgement,ProjectPublication}, canonical_artifacts::{artifact_block,seal_artifact_request}};
use std::collections::BTreeMap;
/// Explicit raw draft document codec. Saving arbitrary bytes does not require this shape;
/// only scientific publication decodes it and invokes the ordinary package loader.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDraftDocuments {
    /// This explicit original-byte container's version.
    pub version:pse_model::document::Version<1>,
    /// Complete packages in original package order, with original relative paths/bytes.
    pub packages:Vec<BTreeMap<String,Vec<u8>>>,
}
/// Original, read-only issued large-draft custody intent. Preserve this value unchanged
/// until exact project and artifact receipts settle; it grants no scientific execution.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDraftSaveIntent {
    issuing_store:String,
    operation:String,
    project:Project,
    branch:Branch,
    key:String,
    base:String,
    expected:Option<u64>,
    digest:String,
    bytes:u64,
    artifact:Option<pse_operations::canonical_artifacts::Artifact>,
}
impl Runtime {
    /// Read-only issue of the original immutable raw-draft custody effect.
    pub async fn project_draft_save_intent(&self,operation:&str,project:&Project,branch:&Branch,key:&str,base:&str,expected:Option<u64>,content:&[u8])->Result<ProjectDraftSaveIntent,WorkflowError>{
        self.scope_activity(async {
            let store=self.canonical_store()?;
            if branch.project!=project.key || operation.is_empty() || key.is_empty(){return Err(contract("draft issuing context differs"));}
            let digest=pse_operations::canonical_execution::result_payload_digest(content);
            let artifact=if content.len()>128*1024 {
                let _lease=self.shared.math().reserve("project:draft-issue",content.len().checked_mul(2).ok_or_else(||contract("draft byte extent"))?)?;
                let revision=store.revision(base).await?.ok_or_else(||contract("draft source base unavailable"))?;
                if revision.problem!=branch.problem{return Err(contract("draft source belongs to another branch"));}
                let chunk=pse_operations::canonical_files::FILE_TARGET_BYTES;
                let mut header=store.new_artifact(&revision,"project-draft",Vec::new(),content.len().div_ceil(chunk) as u64,content.len() as u64).await?;
                let blocks=content.chunks(chunk).enumerate().map(|(ordinal,bytes)|artifact_block(&header.key,ordinal as u64,"project-draft",bytes.to_vec(),"",0)).collect::<Vec<_>>();
                seal_artifact_request(&mut header,std::slice::from_ref(&revision),&[],&blocks,"project",key);
                Some(header)
            }else{None};
            Ok(ProjectDraftSaveIntent{issuing_store:store.issuing_store()?.into(),operation:operation.into(),project:project.clone(),branch:branch.clone(),key:key.into(),base:base.into(),expected,digest,bytes:content.len() as u64,artifact})
        }).await
    }
    /// Save only the caller's original issued body/coordinates; no new artifact is minted on retry.
    pub async fn project_save_draft_intent(&self,intent:&ProjectDraftSaveIntent,content:&[u8])->Result<ProjectAcknowledgement,WorkflowError>{
        self.scope_activity(async {
            let store=self.canonical_store()?;
            if store.issuing_store()?!=intent.issuing_store || content.len() as u64!=intent.bytes || pse_operations::canonical_execution::result_payload_digest(content)!=intent.digest{return Err(pse_operations::canonical::CanonicalError::OperationReused.into());}
            if let Some(header)=&intent.artifact {
                let _lease=self.shared.math().reserve("project:draft-custody",content.len().checked_mul(3).ok_or_else(||contract("draft byte extent"))?)?;
                let revision=store.revision(&intent.base).await?.ok_or_else(||contract("draft source base unavailable"))?;
                let blocks=content.chunks(pse_operations::canonical_files::FILE_TARGET_BYTES).enumerate().map(|(ordinal,bytes)|artifact_block(&header.key,ordinal as u64,"project-draft",bytes.to_vec(),"",0)).collect::<Vec<_>>();
                let mut sealed=header.clone();
                seal_artifact_request(&mut sealed,std::slice::from_ref(&revision),&[],&blocks,"project",&intent.key);
                if sealed.creation_request_digest!=header.creation_request_digest||header.kind!="project-draft"||header.revision!=intent.base||header.primary_problem!=intent.branch.problem||header.payload_bytes!=content.len() as u64||header.block_count!=blocks.len() as u64||header.active||header.retiring{return Err(pse_operations::canonical::CanonicalError::OperationReused.into());}
                // A settled metadata receipt does not authenticate a caller-edited artifact
                // header. Compare the complete original immutable descriptor at its key.
                if let Some(ack)=store.project_acknowledgement(&intent.operation).await? {
                    let actual=store.artifact(&header.key).await?;
                    let mut expected=header.clone();expected.active=true;
                    if actual!=expected{return Err(pse_operations::canonical::CanonicalError::OperationReused.into());}
                    if ack.kind!="draft" || ack.project!=intent.project.key || ack.branch.as_deref()!=Some(intent.branch.key.as_str()){return Err(pse_operations::canonical::CanonicalError::OperationReused.into());}
                    return Ok(store.save_project_draft(&intent.operation,&intent.project,&intent.branch,&intent.key,&intent.base,intent.expected,&[],Some(&header.key)).await?);
                }
                store.persist_artifact(header,std::slice::from_ref(&revision),&[],&blocks,"project",&intent.key).await?;
                Ok(store.save_project_draft(&intent.operation,&intent.project,&intent.branch,&intent.key,&intent.base,intent.expected,&[],Some(&header.key)).await?)
            } else {
                Ok(store.save_project_draft(&intent.operation,&intent.project,&intent.branch,&intent.key,&intent.base,intent.expected,content,None).await?)
            }
        }).await
    }
    /// Single-effect bounded inline save; larger bodies require the retained original intent API.
    pub async fn project_save_draft(&self,operation:&str,project:&Project,branch:&Branch,key:&str,base:&str,expected:Option<u64>,content:&[u8])->Result<ProjectAcknowledgement,WorkflowError>{
        if content.len()>128*1024{return Err(contract("large draft save requires project_draft_save_intent and project_save_draft_intent"));}
        self.scope_activity(async {Ok(self.canonical_store()?.save_project_draft(operation,project,branch,key,base,expected,content,None).await?)}).await
    }
    /// Hydrate only this saved raw body through original artifact/file protection owners.
    pub async fn project_draft_bytes(&self,key:&str)->Result<pse_columnar::Leased<[u8]>,WorkflowError>{
        self.scope_activity(async {
            let draft=self.canonical_store()?.project_draft(key).await?.ok_or_else(||contract("project draft unavailable"))?;
            self.hydrate_selected_draft(&draft).await
        }).await
    }
    // Selection and hydration have deliberately separate lifetimes. Never reread a
    // mutable draft key after a caller selected the exact descriptor/generation.
    async fn hydrate_selected_draft(&self,draft:&pse_operations::canonical_projects::Draft)->Result<pse_columnar::Leased<[u8]>,WorkflowError>{
        let store=self.canonical_store()?;
        let Some(artifact)=draft.artifact.as_deref() else{
            let lease=self.shared.math().reserve("project:inline-draft-read",draft.content.len().checked_mul(2).ok_or_else(||contract("draft byte extent"))?)?;
            return Ok(pse_columnar::Leased::new(std::sync::Arc::<[u8]>::from(draft.content.as_slice()),lease));
        };
        let read=store.read_artifact(artifact).await?;
        if read.header().kind!="project-draft" || read.header().revision!=draft.base_revision {return Err(contract("project draft artifact context differs"));}
        let size=usize::try_from(read.header().payload_bytes).map_err(|_|contract("draft body extent"))?;
        let lease=self.shared.math().reserve("project:draft-read",size.checked_mul(2).ok_or_else(||contract("draft body extent"))?)?;
        let mut result=Vec::with_capacity(size);
        for ordinal in 0..read.header().block_count {let block=read.block(ordinal).await?;if block.relation!="project-draft"{return Err(contract("draft block codec differs"));} result.extend_from_slice(block.payload.as_slice());}
        read.release().await?;
        if result.len()!=size{return Err(contract("draft body coverage differs"));}Ok(pse_columnar::Leased::new(std::sync::Arc::<[u8]>::from(result),lease))
    }
    /// Decode and admit the exact saved generation; publish only after scientific admission succeeds.
    pub async fn project_publish_draft(&self,intent:&ProjectPublication,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        self.scope_activity(async {
            intent.validate()?;
            let (key,generation)=intent.draft().ok_or_else(||contract("draft publication requires its original saved generation"))?;
            let store=self.canonical_store()?;
            if let Some(revision)=store.settled_project_publication(intent).await? {
                return self.modeling_revision(revision,physical,BTreeMap::new()).await;
            }
            let saved=store.project_draft(key).await?.ok_or_else(||contract("project draft unavailable"))?;
            if saved.generation!=generation{
                // A publication may have committed between the first receipt observation
                // and the later draft edit. Never turn that accepted effect into rejection.
                if let Some(revision)=store.settled_project_publication(intent).await?{return self.modeling_revision(revision,physical,BTreeMap::new()).await;}
                return Err(contract("project draft generation changed"));
            }
            let bytes=self.hydrate_selected_draft(&saved).await?;
            let _lease=self.shared.math().reserve("project:draft-decode",bytes.len().checked_mul(4).ok_or_else(||contract("draft document extent"))?)?;
            let content:ProjectDraftDocuments=pse_model::document::decode_versioned::<ProjectDraftDocuments,1>(&bytes).map_err(|e|contract(format!("project draft document codec: {e}")))?;
            let pool=self.shared.pool();let token=cancel.token();let validation=self.admission_context();
            let bundles=content.packages.iter().map(|documents|pse_admission::authoring_driver::document::load_package_documents_owned(documents,self.registry(),pse_authoring::ParseBudget::default(),&pool,&token,&validation)).collect::<Result<Vec<_>,_>>()?;
            let documents=pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(bundles,&pool,&token)?;
            self.project_publish_documents(intent,&documents,physical,cancel).await
        }).await
    }
}

#[cfg(all(test,feature="canonical-tests"))]
mod tests {
    #![allow(clippy::unwrap_used,reason="owned canonical project fixture assertions require exact acknowledged effects")]
    use super::*;
    #[tokio::test]
    async fn project_large_draft_ack_rejects_coherently_forged_original_artifact_header() {
        let runtime=crate::workflow::tests::durable_runtime();let store=runtime.canonical_store().unwrap();
        let source=store.edit("original",None,"source",&[]).await.unwrap();store.create_project("create","project","Engineering").await.unwrap();
        let project=store.project("project").await.unwrap().unwrap();let fork=store.project_fork_intent(&project,"main","branch-problem",&source,"fork").unwrap();let revision=store.edit_project(&fork,&[]).await.unwrap();
        let project=store.project("project").await.unwrap().unwrap();let branch=store.project_branch("main").await.unwrap().unwrap();
        let body=vec![b'a';200_000];let original=runtime.project_draft_save_intent("save",&project,&branch,"draft",&revision.key,None,&body).await.unwrap();runtime.project_save_draft_intent(&original,&body).await.unwrap();
        let changed=vec![b'b';220_000];let mut forged=original.clone();forged.digest=pse_operations::canonical_execution::result_payload_digest(&changed);forged.bytes=changed.len() as u64;
        let header=forged.artifact.as_mut().unwrap();header.payload_bytes=changed.len() as u64;header.block_count=changed.len().div_ceil(pse_operations::canonical_files::FILE_TARGET_BYTES) as u64;
        let blocks=changed.chunks(pse_operations::canonical_files::FILE_TARGET_BYTES).enumerate().map(|(ordinal,bytes)|artifact_block(&header.key,ordinal as u64,"project-draft",bytes.to_vec(),"",0)).collect::<Vec<_>>();
        seal_artifact_request(header,std::slice::from_ref(&revision),&[],&blocks,"project","draft");
        assert!(matches!(runtime.project_save_draft_intent(&forged,&changed).await,Err(WorkflowError::Canonical(pse_operations::canonical::CanonicalError::OperationReused))));
        assert_eq!(&*runtime.project_draft_bytes("draft").await.unwrap(),body.as_slice());
        assert_eq!(runtime.project_save_draft_intent(&original,&body).await.unwrap().key,"save");
        let selected=store.project_draft("draft").await.unwrap().unwrap();
        runtime.project_save_draft("advance",&project,&branch,"draft",&revision.key,Some(1),b"later malformed [[").await.unwrap();
        assert_eq!(&*runtime.hydrate_selected_draft(&selected).await.unwrap(),body.as_slice(),"hydration must consume the selected immutable descriptor, never reread the mutable draft key");
        assert_eq!(&*runtime.project_draft_bytes("draft").await.unwrap(),b"later malformed [[");

    }
}
