// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sole selected-project traversal, using the original source/result/artifact owners.
use super::project_portability::*;
use super::{PhysicalContext, Runtime, WorkflowError, contract};
use pse_operations::canonical::ProtectedSelection;
use std::collections::{BTreeMap, BTreeSet};

pub(super) enum ResultMember {
    Full(super::CanonicalResultReader),
    Selected(super::SelectedResultReader),
}
pub(super) struct ResolvedClosure {
    pub(super) manifest: ProjectManifest,
    pub(super) results: Vec<(String, ResultMember)>,
    pub(super) artifacts: Vec<pse_operations::canonical_artifacts::ArtifactRead>,
    protections: Vec<ProtectedSelection>,
}
impl ResolvedClosure {
    pub(super) async fn renew(&self) -> Result<(), WorkflowError> {
        for protection in &self.protections {
            protection
                .renew(std::time::Duration::from_secs(3600))
                .await?;
        }
        // Result/artifact reads retain their original grant through final publication.
        for (_, reader) in &self.results {
            match reader {
                ResultMember::Full(r) => {
                    r.selection()
                        .renew(std::time::Duration::from_secs(3600))
                        .await?
                }
                ResultMember::Selected(r) => {
                    r.selection()
                        .renew(std::time::Duration::from_secs(3600))
                        .await?
                }
            };
        }
        for read in &self.artifacts {
            read.renew(std::time::Duration::from_secs(3600)).await?;
        }
        Ok(())
    }
}
fn member(id: String, codec: &str) -> ProjectBundleMember {
    ProjectBundleMember {
        id,
        relative_path: String::new(),
        codec: codec.into(),
        copied_bytes: 0,
        digest: String::new(),
        result: None,
        artifact: None,
        artifact_metadata: None,
    }
}
impl Runtime {
    async fn project_selection_metadata(
        &self,
        selection: &super::ProjectSelection,
    ) -> Result<(), WorkflowError> {
        let store = self.canonical_store()?;
        if selection.issuing_store != store.issuing_store()?
            || store.project(&selection.project).await?.is_none()
        {
            return Err(contract("selected project metadata authority unavailable"));
        }
        let revision = store
            .revision(&selection.revision)
            .await?
            .ok_or_else(|| contract("selected source revision metadata unavailable"))?;
        if revision.problem != selection.problem {
            return Err(contract("selected original source problem differs"));
        }
        match &selection.reference {
            super::ProjectReference::Branch { key } => {
                let branch = store
                    .project_branch(key)
                    .await?
                    .ok_or_else(|| contract("selected original branch unavailable"))?;
                if branch.project != selection.project || branch.problem != selection.problem {
                    return Err(contract("selected branch owner differs"));
                }
            }
            super::ProjectReference::Bookmark { key } => {
                let bookmark = store
                    .project_bookmark(key)
                    .await?
                    .ok_or_else(|| contract("selected original bookmark unavailable"))?;
                if bookmark.project != selection.project || bookmark.revision != selection.revision
                {
                    return Err(contract("selected bookmark owner differs"));
                }
            }
        }
        Ok(())
    }
    /// Explain selected exact closure through the same traversal used by logical export.
    pub async fn project_closure(
        &self,
        request: &ProjectExportRequest,
        physical: PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectClosureExplanation, WorkflowError> {
        let mut sources = SourceSink::new(
            None,
            request.maximum_output_bytes,
            request.maximum_document_bytes,
        );
        let closure = self
            .project_boxed(self.resolve_project_closure(
                request,
                Some(physical),
                cancel,
                &mut sources,
            ))?
            .await?;
        let known_source_bytes = sources.bytes;
        Ok(ProjectClosureExplanation {
            manifest: closure.manifest.clone(),
            known_source_bytes,
            result_bytes_known: false,
        })
    }
    /// Explain authenticated descriptive metadata without constructing or admitting a physical context.
    pub async fn project_closure_metadata(
        &self,
        request: &ProjectExportRequest,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectClosureExplanation, WorkflowError> {
        if !request.descriptive_only {
            return Err(contract(
                "metadata closure requires explicit descriptive-only selection",
            ));
        }
        let mut sources = SourceSink::new(
            None,
            request.maximum_output_bytes,
            request.maximum_document_bytes,
        );
        let closure = self
            .project_boxed(self.resolve_project_closure(request, None, cancel, &mut sources))?
            .await?;
        Ok(ProjectClosureExplanation {
            manifest: closure.manifest.clone(),
            known_source_bytes: 0,
            result_bytes_known: false,
        })
    }
    pub(super) async fn resolve_project_closure(
        &self,
        request: &ProjectExportRequest,
        physical: Option<PhysicalContext>,
        cancel: &crate::CancelSource,
        sources: &mut SourceSink,
    ) -> Result<ResolvedClosure, WorkflowError> {
        super::study::creation_checkpoint(cancel)?;
        if request.maximum_document_bytes == 0
            || request.maximum_document_bytes > 64 << 20
            || (request.maximum_output_bytes == 0 && !request.descriptive_only)
            || request.results.len() > 1024
            || request.artifacts.len() > 1024
            || request.studies.len() > 1024
            || request.research_selections.len() > 1024
            || request.publications.len() > 1024
        {
            return Err(contract("project selection exceeds finite allowance"));
        }
        let store = self.canonical_store()?;
        if request.selection.issuing_store != store.issuing_store()?
            || (!request.descriptive_only
                && physical
                    .as_ref()
                    .is_none_or(|p| request.selection.physical_context != p.identity()))
        {
            return Err(contract(
                "project closure authority or physical identity differs",
            ));
        }
        // Metadata authentication is shared; descriptive inspection never opens source/result payload owners.
        self.project_selection_metadata(&request.selection).await?;
        let selected = if request.descriptive_only {
            None
        } else {
            Some(
                self.project_boxed(
                    self.project_open_selection(
                        &request.selection,
                        physical
                            .as_ref()
                            .ok_or_else(|| {
                                contract("executable closure requires admitted physical context")
                            })?
                            .clone(),
                        cancel,
                    ),
                )?
                .await?,
            )
        };
        let mut closure = ResolvedClosure {
            manifest: ProjectManifest {
                version: pse_model::document::Version,
                origin: request.selection.clone(),
                sources: Vec::new(),
                source_revisions: Vec::new(),
                result_metadata: Vec::new(),
                datasets: Vec::new(),
                research_selections: Vec::new(),
                publications: Vec::new(),
                contexts: Vec::new(),
                studies: Vec::new(),
                reproducers: request.reproducers.clone(),
                providers: Vec::new(),
                artifact_origins: Vec::new(),
                members: Vec::new(),
                maximum_copy_bytes: request.maximum_output_bytes,
                omissions: request.omissions.clone(),
                offline_complete: false,
                executable_complete: false,
                descriptive_only: request.descriptive_only,
            },
            results: Vec::new(),
            artifacts: Vec::new(),
            protections: Vec::new(),
        };
        let mut selected = selected;
        let mut selections = request.research_selections.clone();
        let mut revisions = BTreeSet::from([request.selection.revision.clone()]);
        let mut datasets = Vec::new();
        let mut contexts = BTreeSet::new();
        if let Some(research) = selected.as_ref().and_then(|p| p.research_source.as_ref()) {
            selections.push(research.selection.clone());
        }

        for key in &request.studies {
            let definition = self
                .project_boxed(self.portable_study_definition(key, cancel))?
                .await?;
            revisions.insert(definition.modeling_revision.clone());
            closure.manifest.studies.push(ProjectStudyClosure {
                key: key.clone(),
                definition,
            });
        }
        let mut publications = request.publications.clone();
        if store
            .research_publication(&request.selection.revision)
            .await?
            .is_some()
        {
            publications.push(request.selection.revision.clone());
        }
        for study in &closure.manifest.studies {
            if let Some(research) = &study.definition.research {
                if research.issuing_store != store.issuing_store()? {
                    return Err(contract("study research origin authority differs"));
                }
                selections.push(research.selection.clone());
                if let Some(publication) = &research.publication {
                    publications.push(publication.clone());
                }
            }
            if let Some(context) = &study.definition.context {
                contexts.insert(context.context.key.clone());
            }
        }
        publications.sort();
        publications.dedup();
        for key in &publications {
            let row = store
                .research_publication(key)
                .await?
                .ok_or_else(|| contract("selected parameter publication unavailable"))?;
            if row.project != request.selection.project {
                return Err(contract("publication belongs to another project"));
            }
            selections.push(row.selection.clone());
            revisions.insert(row.revision.clone());
            revisions.insert(row.parent.clone());
            closure.manifest.publications.push(row);
        }
        selections.sort();
        selections.dedup();
        for key in selections {
            let row = store
                .research_selection(&key)
                .await?
                .ok_or_else(|| contract("selected research observation selection unavailable"))?;
            if row.project != request.selection.project {
                return Err(contract("research selection belongs to another project"));
            }
            datasets.push(row.dataset.clone());
            contexts.insert(row.context.clone());
            revisions.insert(row.revision.clone());
            closure.manifest.research_selections.push(row);
        }
        let mut seen = BTreeSet::new();
        while let Some(key) = datasets.pop() {
            if !seen.insert(key.clone()) {
                continue;
            }
            if seen.len() > 1024 {
                return Err(contract("research dataset ancestry exceeds allowance"));
            }
            let row = store
                .research_dataset(&key)
                .await?
                .ok_or_else(|| contract("selected immutable dataset unavailable"))?;
            if row.project != request.selection.project {
                return Err(contract("research dataset belongs to another project"));
            }
            if let Some(parent) = &row.parent {
                datasets.push(parent.clone());
            }
            contexts.insert(row.context.clone());
            revisions.insert(row.revision.clone());
            closure.manifest.datasets.push(row);
        }
        for key in contexts {
            let row = store
                .project_context_record(&key)
                .await?
                .ok_or_else(|| contract("selected research context unavailable"))?;
            revisions.insert(row.revision.clone());
            closure.manifest.contexts.push(row);
        }
        if request.descriptive_only {
            for selected in &request.results {
                super::study::creation_checkpoint(cancel)?;
                if selected.row_start > selected.row_end
                    || self.registry().relation(&selected.relation).is_none()
                {
                    return Err(contract("invalid descriptive result relation or range"));
                }
                if let Some(output) = &selected.output {
                    output.bounds()?;
                }
                let origin = store
                    .canonical_result_origin(&selected.run, &selected.attempt)
                    .await?
                    .ok_or_else(|| contract("selected original result metadata unavailable"))?;
                if origin.run != selected.run || origin.key != selected.attempt {
                    return Err(contract("selected result attempt belongs to another run"));
                }
                revisions.insert(origin.source_revision.clone());
                closure
                    .manifest
                    .result_metadata
                    .push(ProjectResultMetadata {
                        selection: selected.clone(),
                        source_revision: origin.source_revision,
                        generation: origin.generation,
                        phase: origin.phase,
                        terminal_class: origin.outcome,
                        closed_manifest: origin.closed_manifest,
                    });
                closure.manifest.omissions.push(ProjectOmission{selection:format!("result:{}",selected.member),reason:"descriptive-only selection copies original run/attempt metadata; result rows, manifests and execution seed authority were not requested".into(),required_for_execution:true});
            }
            for key in &request.artifacts {
                let header = store.artifact(key).await?;
                revisions.insert(header.revision.clone());
                closure.manifest.artifact_origins.push(header);
                closure.manifest.omissions.push(ProjectOmission{selection:format!("artifact:{key}"),reason:"descriptive-only selection copies the original artifact header; protected block payloads were not requested".into(),required_for_execution:true});
            }
        } else {
            let (result_descriptors, result_owner) = self
                .review_result_selections(&request.results, &cancel.token())
                .await?;
            for (request, descriptor) in request.results.iter().zip(result_descriptors) {
                let reader = if let Some(output) = &request.output {
                    let (minimum, maximum) = output.bounds()?;
                    ResultMember::Selected(
                        self.output_results(
                            &request.run,
                            &request.attempt,
                            &request.relation,
                            output.owner,
                            &output.field,
                            &output.partition,
                            request.row_start,
                            request.row_end,
                            minimum,
                            maximum,
                            output.missing,
                            cancel.token(),
                        )
                        .await?,
                    )
                } else {
                    ResultMember::Full(
                        self.results(
                            &request.run,
                            &request.attempt,
                            &request.relation,
                            request.row_start,
                            request.row_end,
                            cancel.token(),
                        )
                        .await?,
                    )
                };
                let source = match &reader {
                    ResultMember::Full(r) => r.selection().source_revision(),
                    ResultMember::Selected(r) => r.selection().source_revision(),
                };
                revisions.insert(source.key.clone());
                let mut selected_member = member(request.member.clone(), "arrow-ipc-stream.v1");
                selected_member.result = Some(descriptor);
                closure.manifest.members.push(selected_member);
                closure.results.push((request.member.clone(), reader));
            }
            drop(result_owner);
            for key in &request.artifacts {
                let read = store.read_artifact(key).await?;
                revisions.insert(read.header().revision.clone());
                closure
                    .manifest
                    .artifact_origins
                    .push(read.header().clone());
                closure.artifacts.push(read);
            }
        }
        // Hold the complete compact revision selection before source encoding.
        for revision in &revisions {
            let row = store.revision(revision).await?.ok_or_else(|| {
                contract("required original source revision metadata unavailable")
            })?;
            closure
                .manifest
                .source_revisions
                .push(ProjectSourceRevisionMetadata {
                    key: row.key.clone(),
                    problem: row.problem.clone(),
                    sequence: row.sequence,
                    parent: row.parent.clone(),
                    operation: row.operation.clone(),
                    interpretation: row.interpretation.clone(),
                });
            if request.descriptive_only {
                let context = closure
                    .manifest
                    .contexts
                    .iter()
                    .find(|c| c.revision.as_str() == revision.as_str());
                closure.manifest.sources.push(ProjectSourceClosure {
                    revision: revision.clone(),
                    source_identity: context.map(|c| c.source_identity.clone()),
                    physical_context: context.map(|c| c.physical_context.clone()),
                    documents: Vec::new(),
                    replay_source: None,
                    physical_documents: Vec::new(),
                });
                closure.manifest.omissions.push(ProjectOmission{selection:format!("source:{revision}"),reason:"descriptive-only selection retains authenticated revision metadata; authored source objects, raw/curated data, physical documents and provider premises were not opened or copied".into(),required_for_execution:true});
            } else {
                closure.protections.push(
                    store
                        .protect(row, std::time::Duration::from_secs(3600))
                        .await?,
                );
            }
        }
        // Consume the already admitted origin first, preserving sorted revision ordinals for ids.
        let mut ordered = revisions.into_iter().enumerate().collect::<Vec<_>>();
        ordered.sort_by_key(|(_, revision)| revision != &request.selection.revision);
        for (ordinal, revision) in ordered {
            if request.descriptive_only {
                continue;
            }
            super::study::creation_checkpoint(cancel)?;
            let recorded = closure
                .protections
                .iter()
                .find(|protection| protection.revision().key == revision)
                .ok_or_else(|| contract("selected source lacks original protection"))?
                .revision()
                .clone();
            let package = if let Some(context) = closure
                .manifest
                .contexts
                .iter()
                .rev()
                .find(|context| context.revision == revision)
            {
                let selection = pse_operations::canonical_contexts::ContextSelection {
                    cut: pse_operations::canonical_contexts::ContextCut {
                        issuing_store: store.issuing_store()?.into(),
                        project: context.project.clone().into(),
                        sequence: context.recorded_sequence,
                    },
                    effective_at_us: context.effective_from_us,
                    context: context.clone(),
                };
                if revision == request.selection.revision {
                    drop(selected.take());
                }
                self.project_boxed(self.project_context_reopen(&selection, cancel))?
                    .await?
            } else if revision == request.selection.revision {
                selected
                    .take()
                    .ok_or_else(|| contract("selected origin was not admitted"))?
            } else {
                let physical = if let Some(study) = closure
                    .manifest
                    .studies
                    .iter()
                    .find(|study| study.definition.modeling_revision == revision)
                {
                    self.project_boxed(self.physical_source(&study.definition.physical, cancel))?
                        .await?
                } else {
                    physical
                        .as_ref()
                        .ok_or_else(|| contract("source requires admitted physical context"))?
                        .clone()
                };
                self.project_boxed(self.modeling_revision(recorded, physical, BTreeMap::new()))?
                    .await?
            };
            let source_extent = package
                .replay_source_extent()
                .checked_mul(4)
                .ok_or_else(|| contract("portable source extent"))?;
            let _owner = self
                .shared
                .math()
                .reserve("project:portable-source", source_extent)?;
            let replay = package.replay_source()?;
            for provider in &replay.providers {
                let name = provider.name.clone();
                let identity =
                    pse_ids::encoding_checksum(&encode(provider, request.maximum_document_bytes)?)
                        .content_hash()
                        .to_hex();
                if !closure
                    .manifest
                    .providers
                    .iter()
                    .any(|p| p.name == name && p.identity == identity)
                {
                    closure.manifest.providers.push(ProjectProviderRequirement {
                        name,
                        identity,
                        supported: false,
                    });
                }
            }
            let replay_id = format!("source-{ordinal}");
            let encoded = encode(&replay, sources.document_limit())?;
            sources.emit(&replay_id, &encoded, &cancel.token())?;
            drop(encoded);
            drop(replay);
            closure
                .manifest
                .members
                .push(member(replay_id.clone(), "replay-source.v1"));
            let physical_documents = self.physical_source_documents(&package.physical)?;
            let mut physical_members = Vec::new();
            for (index, (path, bytes)) in physical_documents.iter().enumerate() {
                let id = format!("physical-{ordinal}-{index}");
                physical_members.push(ProjectDocumentMember {
                    package: 0,
                    path: path.clone(),
                    member: id.clone(),
                });
                sources.emit(&id, bytes, &cancel.token())?;
                closure
                    .manifest
                    .members
                    .push(member(id, "authored-document.v1"));
            }
            let mut documents = Vec::new();
            // Typed edits can deliberately invalidate loader text. The exact replay source remains the ordinary source owner.
            if let Some(bundles) = package
                .portability_documents(&self.source_services(), cancel)
                .await?
            {
                for (package_index, bundle) in bundles.iter().enumerate() {
                    for (index, (path, bytes)) in bundle.iter().enumerate() {
                        let id = format!("document-{ordinal}-{package_index}-{index}");
                        documents.push(ProjectDocumentMember {
                            package: package_index,
                            path: path.clone(),
                            member: id.clone(),
                        });
                        sources.emit(&id, bytes, &cancel.token())?;
                        closure
                            .manifest
                            .members
                            .push(member(id, "authored-document.v1"));
                    }
                }
            }
            closure.manifest.sources.push(ProjectSourceClosure {
                revision,
                source_identity: Some(package.revision.identity().to_string()),
                physical_context: Some(package.physical.identity().to_string()),
                documents,
                replay_source: Some(replay_id),
                physical_documents: physical_members,
            });
        }
        closure
            .manifest
            .sources
            .sort_by(|left, right| left.revision.cmp(&right.revision));
        closure.manifest.providers.sort_by(|left, right| {
            (&left.name, &left.identity).cmp(&(&right.name, &right.identity))
        });
        // Artifacts use original protected reads and preserve their original encoding digests and ordinal extent.
        for (artifact_index, read) in closure.artifacts.iter().enumerate() {
            for ordinal in 0..read.header().block_count {
                let mut m = member(
                    format!("artifact-{artifact_index}-{ordinal}"),
                    "artifact-block.v1",
                );
                m.artifact = Some((read.header().key.clone(), ordinal));
                closure.manifest.members.push(m);
            }
        }
        let mut ids = BTreeSet::new();
        for (ordinal, m) in closure.manifest.members.iter_mut().enumerate() {
            if !ids.insert(m.id.clone())
                || m.id.is_empty()
                || !m
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            {
                return Err(contract("duplicate or unsafe selected project member"));
            }
            m.relative_path = format!("members/{ordinal:05}.bin");
        }
        closure.manifest.executable_complete = !request.descriptive_only
            && !request.omissions.iter().any(|o| o.required_for_execution)
            && closure.manifest.providers.iter().all(|p| p.supported);
        closure.renew().await?;
        Ok(closure)
    }
}
