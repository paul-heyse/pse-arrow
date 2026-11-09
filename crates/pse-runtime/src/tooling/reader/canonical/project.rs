// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Connection-only reads using the canonical project owner's native selections and pages.
use super::{MAXIMUM_READ_MILLIS, canonical, canonical_redaction_policy, connected, workflow};
use crate::tooling::reader::{
    EncodedResponse, Payload, ReaderError, encode_response, response_with_policy,
};
use crate::workflow::{
    ProjectClosureExplanation, ProjectExportRequest, ProjectImportIntent, ProjectImportReport,
    ResearchReadRequest, ResearchReadResult, project_closure_metadata, project_import_status,
    research_read,
};
use pse_host::envelope::RedactionPolicy;
use pse_host::investigation::NativePath;
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore},
    canonical_control::{StudyWorkbenchPage, StudyWorkbenchRequest},
    canonical_projects::{
        Branch, BranchPage, Draft, Project, ProjectCursor, ProjectMetadataPage,
        ProjectMetadataRequest,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// One explicitly selected private deployment and its configured or selected database.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectReadSelection {
    /// Existing private deployment state directory, as exact native path bytes.
    pub state: NativePath,
    /// Existing database override; absence selects the database named by the deployment.
    pub database: Option<String>,
    /// Require this installation identity before reading any selected records.
    pub expected_issuer: Option<String>,
    /// Optional observer allowance in 1..=25000ms, including connection and drain.
    pub timeout_ms: Option<u64>,
}
impl ProjectReadSelection {
    fn timeout_ms(&self) -> u64 {
        self.timeout_ms.unwrap_or(MAXIMUM_READ_MILLIS)
    }

    pub(in crate::tooling::reader) fn validate(&self) -> Result<(), ReaderError> {
        super::super::absolute(&self.state.to_path_buf())?;
        if !(1..=MAXIMUM_READ_MILLIS).contains(&self.timeout_ms()) {
            return Err(super::super::invalid(
                "project read requires a 1..=25000ms observer allowance",
            ));
        }
        Ok(())
    }
}

/// One canonical project read using the original owner request and cursor contracts.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectRead {
    /// Authenticate the opened store without reading project records.
    Target,
    /// Read bounded metadata for the selected collection and owner.
    Metadata {
        /// Existing canonical metadata request.
        request: ProjectMetadataRequest,
    },
    /// Read an exact project header.
    Show {
        /// Canonical project key.
        project: String,
    },
    /// Read one generation-fenced branch page.
    Branches {
        /// Canonical project key.
        project: String,
        /// Original branch continuation.
        cursor: Option<ProjectCursor>,
    },
    /// Read an exact branch header.
    Branch {
        /// Canonical branch key.
        branch: String,
    },
    /// Read a draft header and its exact version descriptor.
    DraftShow {
        /// Canonical draft key.
        key: String,
    },
    /// Browse compact live study metadata under indexed native predicates.
    StudyWorkbench {
        /// Original predicates, row limit and issuer-bound continuation.
        request: Box<StudyWorkbenchRequest>,
    },
    /// Browse or read exact canonical research metadata.
    ResearchRead {
        /// Original dataset seek bounds or exact immutable key.
        request: ResearchReadRequest,
    },
    /// Explain an already-issued project selection using descriptive metadata only.
    Closure {
        /// The original finite closure selection. Executable requests refuse.
        request: Box<ProjectExportRequest>,
    },
    /// Observe the receiving checkpoint for this exact original import intent.
    ImportStatus {
        /// The immutable issued intent, unchanged from preparation.
        intent: Box<ProjectImportIntent>,
    },
}

/// Authenticated coordinates observed by the original opened canonical owner.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProjectTarget {
    namespace: String,
    database: String,
    issuing_store: String,
}

/// Typed original metadata and header results. Missing headers remain explicit.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectReadView {
    /// The store identity is in the enclosing authenticated target.
    Target,
    /// Canonical metadata page.
    Metadata {
        /// Up to 64 rows and their original continuation.
        page: ProjectMetadataPage,
    },
    /// Compact exact project header.
    Show {
        /// Original project metadata if available.
        project: Option<Project>,
    },
    /// Canonical branch page.
    Branches {
        /// Up to 64 rows, the observed project, and their original continuation.
        page: BranchPage,
    },
    /// Compact exact branch header.
    Branch {
        /// Original branch metadata if available.
        branch: Option<Branch>,
    },
    /// Exact draft head and version descriptor.
    DraftShow {
        /// Original draft metadata if available.
        draft: Option<Draft>,
    },
    /// One bounded live page of compact study headers.
    StudyWorkbench {
        /// Original headers and continuation, with no scientific payloads opened.
        page: StudyWorkbenchPage,
    },
    /// Original research metadata. Measurement payloads are never opened.
    ResearchRead {
        /// The original dataset page or exact research descriptor.
        result: ResearchReadResult,
    },
    /// Authenticated descriptive closure with no source or result payload authority.
    Closure {
        /// The original closure explanation.
        explanation: Box<ProjectClosureExplanation>,
    },
    /// The original receiving checkpoint. Absence alone does not prove rollback.
    ImportStatus {
        /// Original source and project membership settlement facts, when present.
        report: Option<Box<ProjectImportReport>>,
    },
}

async fn read(
    store: &CanonicalStore,
    query: ProjectRead,
    cancel: &pse_columnar::CancellationToken,
) -> Result<ProjectReadView, ReaderError> {
    Ok(match query {
        ProjectRead::Target => ProjectReadView::Target,
        ProjectRead::Metadata { request } => ProjectReadView::Metadata {
            page: store
                .project_metadata(request.kind, &request.owner, request.cursor.as_ref())
                .await
                .map_err(canonical)?,
        },
        ProjectRead::Show { project } => ProjectReadView::Show {
            project: store.project(&project).await.map_err(canonical)?,
        },
        ProjectRead::Branches { project, cursor } => ProjectReadView::Branches {
            page: store
                .project_branches(&project, cursor.as_ref())
                .await
                .map_err(canonical)?,
        },
        ProjectRead::Branch { branch } => ProjectReadView::Branch {
            branch: store.project_branch(&branch).await.map_err(canonical)?,
        },
        ProjectRead::DraftShow { key } => ProjectReadView::DraftShow {
            draft: store.project_draft(&key).await.map_err(canonical)?,
        },
        ProjectRead::StudyWorkbench { request } => ProjectReadView::StudyWorkbench {
            page: store
                .control_study_workbench_page(*request)
                .await
                .map_err(canonical)?,
        },
        ProjectRead::ResearchRead { request } => ProjectReadView::ResearchRead {
            result: research_read(store, &request).await.map_err(workflow)?,
        },
        ProjectRead::Closure { request } => {
            let registry = pse_schema::registry().map_err(|error| {
                super::engine(pse_engine::EngineError::Semantic(std::sync::Arc::new(
                    error,
                )))
            })?;
            ProjectReadView::Closure {
                explanation: Box::new(
                    project_closure_metadata(store, registry, &request, cancel)
                        .await
                        .map_err(workflow)?,
                ),
            }
        }
        ProjectRead::ImportStatus { intent } => ProjectReadView::ImportStatus {
            report: project_import_status(store, &intent)
                .await
                .map_err(workflow)?
                .map(Box::new),
        },
    })
}

pub(in crate::tooling::reader) fn encode(
    correlation: String,
    selection: ProjectReadSelection,
    query: ProjectRead,
    cancel: &pse_columnar::CancellationToken,
) -> Result<EncodedResponse, ReaderError> {
    let deadline = Instant::now() + Duration::from_millis(selection.timeout_ms());
    let mut policy = RedactionPolicy::from_environment();
    let result = (|| {
        cancel.checkpoint()?;
        let options = CanonicalOptions::from_state_for_database(
            &selection.state.to_path_buf(),
            selection.database.as_deref(),
        )
        .map_err(canonical)?;
        policy = canonical_redaction_policy(&options);
        connected(&options, deadline, cancel, async |store| {
            let issuing_store = store.issuing_store().map_err(canonical)?;
            if selection
                .expected_issuer
                .as_deref()
                .is_some_and(|expected| expected != issuing_store)
            {
                return Err(super::super::invalid(
                    "opened store differs from the expected project issuer",
                ));
            }
            let target = ProjectTarget {
                namespace: store.namespace().into(),
                database: store.database().into(),
                issuing_store: issuing_store.into(),
            };
            let view = cancel.until_cancelled(read(store, query, cancel)).await??;
            cancel.checkpoint()?;
            encode_response(response_with_policy(
                correlation.clone(),
                Ok(Payload::Project {
                    target,
                    result: Box::new(view),
                }),
                &policy,
            ))
        })
    })();
    match result {
        Ok(output) => Ok(output),
        Err(error) => encode_response(response_with_policy(correlation, Err(error), &policy)),
    }
}
