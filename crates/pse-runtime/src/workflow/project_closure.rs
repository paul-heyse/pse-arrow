// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sole selected-project traversal, using the original source/result/artifact owners.
use super::{Runtime,PhysicalContext,WorkflowError,contract};
use super::project_portability::*;
use pse_operations::canonical::ProtectedSelection;
use std::collections::{BTreeMap,BTreeSet};

pub(super) enum ResultMember {
    Full(super::CanonicalResultReader),
    Selected(super::SelectedResultReader),
}
pub(super) struct ResolvedClosure {
    pub(super) manifest:ProjectManifest,
    pub(super) documents:Vec<(String,Vec<u8>)>,
    pub(super) results:Vec<(String,ResultMember)>,
    pub(super) artifacts:Vec<pse_operations::canonical_artifacts::ArtifactRead>,
    protections:Vec<ProtectedSelection>,
    _owners:Vec<std::sync::Arc<pse_columnar::AllocationLease>>,
}
impl ResolvedClosure {
    pub(super) async fn renew(&self)->Result<(),WorkflowError> {
        for protection in &self.protections {protection.renew(std::time::Duration::from_secs(3600)).await?;}
        // Result/artifact reads retain their original grant through final publication.
        for (_,reader) in &self.results {match reader {ResultMember::Full(r)=>r.selection().renew(std::time::Duration::from_secs(3600)).await?,ResultMember::Selected(r)=>r.selection().renew(std::time::Duration::from_secs(3600)).await?};}
        for read in &self.artifacts {read.renew(std::time::Duration::from_secs(3600)).await?;}
        Ok(())
    }
}
fn member(id:String,codec:&str)->ProjectBundleMember {
    ProjectBundleMember{id,relative_path:String::new(),codec:codec.into(),copied_bytes:0,digest:String::new(),result:None,artifact:None,artifact_metadata:None}
}
impl Runtime {
    /// Explain selected exact closure through the same traversal used by logical export.
    pub async fn project_closure(&self,request:&ProjectExportRequest,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ProjectClosureExplanation,WorkflowError> {
        let closure=self.project_boxed(self.resolve_project_closure(request,physical,cancel))?.await?;
        let known_source_bytes=closure.documents.iter().map(|(_,b)|b.len() as u64).sum();
        Ok(ProjectClosureExplanation{manifest:closure.manifest.clone(),known_source_bytes,result_bytes_known:false})
    }
    pub(super) async fn resolve_project_closure(&self,request:&ProjectExportRequest,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ResolvedClosure,WorkflowError> {
        super::study::creation_checkpoint(cancel)?;
        if request.maximum_document_bytes==0 || request.maximum_document_bytes>64<<20 || request.maximum_output_bytes==0
            || request.results.len()>1024 || request.artifacts.len()>1024 || request.studies.len()>1024 || request.research_selections.len()>1024 || request.publications.len()>1024 { return Err(contract("project selection exceeds finite allowance")); }
        let store=self.canonical_store()?;
        if request.selection.issuing_store!=store.issuing_store()? || request.selection.physical_context!=physical.identity() {return Err(contract("project closure authority or physical identity differs"));}
        let selected=self.project_boxed(self.project_open_selection(&request.selection,physical.clone(),cancel))?.await?;
        let mut closure=ResolvedClosure{
            manifest:ProjectManifest{version:pse_model::document::Version,origin:request.selection.clone(),sources:Vec::new(),datasets:Vec::new(),research_selections:Vec::new(),publications:Vec::new(),contexts:Vec::new(),studies:Vec::new(),reproducers:request.reproducers.clone(),providers:Vec::new(),artifact_origins:Vec::new(),members:Vec::new(),maximum_copy_bytes:request.maximum_output_bytes,omissions:request.omissions.clone(),offline_complete:false,executable_complete:false,descriptive_only:request.descriptive_only},
            documents:Vec::new(),results:Vec::new(),artifacts:Vec::new(),protections:Vec::new(),_owners:Vec::new(),
        };
        let mut packages=BTreeMap::new(); packages.insert(request.selection.revision.clone(),selected);
        let mut selections=request.research_selections.clone(); let mut revisions=BTreeSet::new(); let mut datasets=Vec::new(); let mut contexts=BTreeSet::new();
        if let Some(research)=packages.values().next().and_then(|p|p.research_source.as_ref()) {selections.push(research.selection.clone());}

        for key in &request.studies {
            let definition=self.project_boxed(self.portable_study_definition(key,cancel))?.await?;
            let physical=self.project_boxed(self.physical_source(&definition.physical,cancel))?.await?;
            if !packages.contains_key(&definition.modeling_revision) {
                let revision=store.revision(&definition.modeling_revision).await?.ok_or_else(||contract("study source unavailable"))?;
                let package=self.project_boxed(self.modeling_revision(revision,physical,BTreeMap::new()))?.await?;
                packages.insert(definition.modeling_revision.clone(),package);
            }
            closure.manifest.studies.push(ProjectStudyClosure{key:key.clone(),definition});
        }
        let mut publications=request.publications.clone();
        if store.research_publication(&request.selection.revision).await?.is_some(){publications.push(request.selection.revision.clone());}
        for study in &closure.manifest.studies {
            if let Some(research)=&study.definition.research {
                if research.issuing_store!=store.issuing_store()? {return Err(contract("study research origin authority differs"));}
                selections.push(research.selection.clone());
                if let Some(publication)=&research.publication {publications.push(publication.clone());}
            }
            if let Some(context)=&study.definition.context {contexts.insert(context.context.key.clone());}
        }
        publications.sort();publications.dedup();
        for key in &publications {
            let row=store.research_publication(key).await?.ok_or_else(||contract("selected parameter publication unavailable"))?;
            if row.project!=request.selection.project {return Err(contract("publication belongs to another project"));}
            selections.push(row.selection.clone()); revisions.insert(row.revision.clone());revisions.insert(row.parent.clone());
            closure.manifest.publications.push(row);
        }
        selections.sort();selections.dedup();
        for key in selections {
            let row=store.research_selection(&key).await?.ok_or_else(||contract("selected research observation selection unavailable"))?;
            if row.project!=request.selection.project {return Err(contract("research selection belongs to another project"));}
            datasets.push(row.dataset.clone());contexts.insert(row.context.clone());revisions.insert(row.revision.clone());
            closure.manifest.research_selections.push(row);
        }
        let mut seen=BTreeSet::new();
        while let Some(key)=datasets.pop() {
            if !seen.insert(key.clone()) {continue;} if seen.len()>1024 {return Err(contract("research dataset ancestry exceeds allowance"));}
            let row=store.research_dataset(&key).await?.ok_or_else(||contract("selected immutable dataset unavailable"))?;
            if row.project!=request.selection.project {return Err(contract("research dataset belongs to another project"));}
            if let Some(parent)=&row.parent {datasets.push(parent.clone());}
            contexts.insert(row.context.clone());revisions.insert(row.revision.clone());closure.manifest.datasets.push(row);
        }
        for key in contexts {
            let row=store.project_context_record(&key).await?.ok_or_else(||contract("selected research context unavailable"))?;
            let selection=pse_operations::canonical_contexts::ContextSelection{cut:pse_operations::canonical_contexts::ContextCut{issuing_store:store.issuing_store()?.into(),project:row.project.clone(),sequence:row.recorded_sequence},effective_at_us:row.effective_from_us,context:row.clone()};
            let package=self.project_boxed(self.project_context_reopen(&selection,cancel))?.await?;
            packages.insert(row.revision.clone(),package);closure.manifest.contexts.push(row);
        }
        let (result_descriptors,result_owner)=self.review_result_selections(&request.results,&cancel.token()).await?;
        for (request,descriptor) in request.results.iter().zip(result_descriptors) {
            let reader=if let Some(output)=&request.output {
                let (minimum,maximum)=output.bounds()?;
                ResultMember::Selected(self.output_results(&request.run,&request.attempt,&request.relation,output.owner,&output.field,&output.partition,request.row_start,request.row_end,minimum,maximum,output.missing,cancel.token()).await?)
            } else {ResultMember::Full(self.results(&request.run,&request.attempt,&request.relation,request.row_start,request.row_end,cancel.token()).await?)};
            let source=match &reader {ResultMember::Full(r)=>r.selection().source_revision(),ResultMember::Selected(r)=>r.selection().source_revision()};
            revisions.insert(source.key.clone());
            let mut selected_member=member(request.member.clone(),"arrow-ipc-stream.v1");selected_member.result=Some(descriptor);
            closure.manifest.members.push(selected_member);closure.results.push((request.member.clone(),reader));
        }
        drop(result_owner);
        for key in &request.artifacts {
            let read=store.read_artifact(key).await?;
            revisions.insert(read.header().revision.clone());
            closure.manifest.artifact_origins.push(read.header().clone());
            closure.artifacts.push(read);
        }
        for revision in revisions {
            if !packages.contains_key(&revision) {
                let row=store.revision(&revision).await?.ok_or_else(||contract("required ancestral source unavailable"))?;
                let package=self.project_boxed(self.modeling_revision(row,physical.clone(),BTreeMap::new()))?.await?;
                packages.insert(revision,package);
            }
        }
        for (ordinal,(revision,package)) in packages.into_iter().enumerate() {
            super::study::creation_checkpoint(cancel)?;
            let recorded=store.revision(&revision).await?.ok_or_else(||contract("selected original source unavailable"))?;
            let protection=store.protect(recorded,std::time::Duration::from_secs(3600)).await?;
            closure.protections.push(protection);
            let source_extent=package.replay_source_extent().checked_mul(4).ok_or_else(||contract("portable source extent"))?;
            let owner=self.shared.math().reserve("project:portable-source",source_extent)?;
            let replay=package.replay_source()?;
            for provider in &replay.providers {
                let name=provider.name.clone();
                let identity=pse_ids::encoding_checksum(&encode(provider,request.maximum_document_bytes)?).content_hash().to_hex();
                if !closure.manifest.providers.iter().any(|p|p.name==name && p.identity==identity) {closure.manifest.providers.push(ProjectProviderRequirement{name,identity,supported:false});}
            }
            let replay_id=format!("source-{ordinal}");
            closure.documents.push((replay_id.clone(),encode(&replay,request.maximum_document_bytes)?));
            closure.manifest.members.push(member(replay_id.clone(),"replay-source.v1"));
            let physical_documents=self.physical_source_documents(&package.physical)?;
            let mut physical_members=Vec::new();
            for (index,(path,bytes)) in physical_documents.iter().enumerate() {
                let id=format!("physical-{ordinal}-{index}");
                physical_members.push(ProjectDocumentMember{package:0,path:path.clone(),member:id.clone()});
                closure.documents.push((id.clone(),bytes.clone()));closure.manifest.members.push(member(id,"authored-document.v1"));
            }
            let mut documents=Vec::new();
            // Typed edits can deliberately invalidate loader text. The exact replay source remains the ordinary source owner.
            if let Some(sources)=package.portability_documents(cancel).await? {
                for (package_index,bundle) in sources.iter().enumerate() {for (index,(path,bytes)) in bundle.iter().enumerate() {
                    let id=format!("document-{ordinal}-{package_index}-{index}");
                    documents.push(ProjectDocumentMember{package:package_index,path:path.clone(),member:id.clone()});
                    closure.documents.push((id.clone(),bytes.clone()));closure.manifest.members.push(member(id,"authored-document.v1"));
                }}
            }
            closure.manifest.sources.push(ProjectSourceClosure{revision,source_identity:package.revision.identity().to_string(),physical_context:package.physical.identity().to_string(),documents,replay_source:replay_id,physical_documents:physical_members});
            closure._owners.push(owner);
        }
        // Artifacts use original protected reads and preserve their original encoding digests and ordinal extent.
        for (artifact_index,read) in closure.artifacts.iter().enumerate() {
            for ordinal in 0..read.header().block_count {
                let mut m=member(format!("artifact-{artifact_index}-{ordinal}"),"artifact-block.v1");m.artifact=Some((read.header().key.clone(),ordinal));closure.manifest.members.push(m);
            }
        }
        let mut ids=BTreeSet::new();
        for (ordinal,m) in closure.manifest.members.iter_mut().enumerate() {
            if !ids.insert(m.id.clone()) || m.id.is_empty() || !m.id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-') {return Err(contract("duplicate or unsafe selected project member"));}
            m.relative_path=format!("members/{ordinal:05}.bin");
        }
        closure.manifest.executable_complete=!request.descriptive_only && !request.omissions.iter().any(|o|o.required_for_execution) && closure.manifest.providers.iter().all(|p|p.supported);
        closure.renew().await?;
        Ok(closure)
    }
}
