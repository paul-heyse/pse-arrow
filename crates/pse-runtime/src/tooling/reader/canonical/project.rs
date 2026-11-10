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
    /// Observe the exact selected native incarnation; no maintenance effects.
    #[cfg(feature = "native-service")]
    Persistence,
    /// Observe current compact facts in one bounded native subscription window.
    Follow {
        /// Exact study or run identity; notification payloads never become facts.
        selection: pse_operations::canonical_study_workbench::FollowSelection,
        /// Finite observation window in 1..=20000ms.
        window_ms: u64,
        /// Maximum retained updates in 1..=64.
        maximum_updates: u16,
    },
    /// Read an occurrence-preserving canonical neighborhood.
    Graph {
        /// The original typed graph question.
        request: pse_operations::canonical_explanations::CanonicalGraphRequest,
    },
    /// Read bounded evidence-bearing paths in one native snapshot.
    GraphPaths {
        /// The original finite path question.
        request: pse_operations::canonical_explanations::CanonicalGraphPathsRequest,
    },
    /// Current compact lifecycle counts, separate from scientific results.
    StudySummary {
        /// Exact study identity.
        study: String,
    },
    /// Current compact run header, without acquiring result bytes.
    RunSummary {
        /// Exact run identity.
        run: String,
    },
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
    /// Authenticated original-server metrics, with unavailable capabilities explicit.
    #[cfg(feature = "native-service")]
    Persistence {
        /// Operational observations, independent of scientific time and protected results.
        observation: pse_operations::native_service::NativePersistenceObservation,
    },
    /// Whole current-state snapshots; this is not a resumable transition history.
    Follow {
        /// Original native snapshots or explicit unavailability.
        updates: Vec<pse_operations::canonical_study_workbench::FollowUpdate>,
        /// True when the caller's count, rather than its window, ended observation.
        update_limit_reached: bool,
    },
    /// Original bounded graph evidence.
    Graph {
        /// The product-owned neighborhood.
        graph: pse_operations::canonical_explanations::CanonicalGraphDocument,
    },
    /// Original bounded path evidence and explicit completeness.
    GraphPaths {
        /// Ordered pages from the same fully received bounded native snapshot.
        pages: Vec<pse_operations::canonical_explanations::CanonicalGraphPathsDocument>,
    },
    /// Current lifecycle counts as of the native read.
    StudySummary {
        /// Missing or unauthorized root remains unavailable.
        summary: Option<pse_operations::canonical_study_workbench::StudySummary>,
    },
    /// Current compact run facts as of the native read.
    RunSummary {
        /// Missing or unauthorized root remains unavailable.
        summary: Option<pse_operations::canonical_study_workbench::RunSummary>,
    },
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
    deadline: Instant,
) -> Result<ProjectReadView, ReaderError> {
    Ok(match query {
        #[cfg(feature = "native-service")]
        ProjectRead::Persistence => ProjectReadView::Persistence {
            observation: pse_operations::native_service::inspect_owned_persistence(
                store.deployment_state(),
                deadline,
            )
            .await
            .map_err(|error| ReaderError::NativeService(Box::new(error)))?,
        },
        ProjectRead::Follow {
            selection,
            window_ms,
            maximum_updates,
        } => {
            if !(1..=20_000).contains(&window_ms) || !(1..=64).contains(&maximum_updates) {
                return Err(super::super::invalid(
                    "project follow requires a 1..=20000ms window and 1..=64 updates",
                ));
            }
            // Leave the original connection owner time to kill registrations and drain.
            let stop = Instant::now() + Duration::from_millis(window_ms);
            if stop + Duration::from_secs(2) > deadline {
                return Err(super::super::invalid(
                    "project follow window exceeds the remaining observer allowance",
                ));
            }
            let mut follow = store
                .follow_current(selection, deadline)
                .await
                .map_err(canonical)?;
            let observed = async {
                let mut updates = Vec::with_capacity(usize::from(maximum_updates));
                while updates.len() < usize::from(maximum_updates) {
                    let next = tokio::select! {
                        _ = tokio::time::sleep_until(stop.into()) => break,
                        next = follow.next() => next.map_err(canonical)?,
                    };
                    let ended = matches!(
                        next,
                        pse_operations::canonical_study_workbench::FollowUpdate::Ended
                    );
                    updates.push(next);
                    if ended {
                        break;
                    }
                }
                Ok(ProjectReadView::Follow {
                    update_limit_reached: updates.len() == usize::from(maximum_updates),
                    updates,
                })
            }
            .await;
            let cleanup = follow.close().await.map_err(canonical);
            match (observed, cleanup) {
                (Ok(view), Ok(())) => view,
                (Err(error), Ok(())) | (Ok(_), Err(error)) => return Err(error),
                (Err(operation), Err(cleanup)) => {
                    return Err(ReaderError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(cleanup),
                    });
                }
            }
        }
        ProjectRead::Graph { request } => ProjectReadView::Graph {
            graph: store.graph_explanation(&request).await.map_err(canonical)?,
        },
        ProjectRead::GraphPaths { request } => {
            let mut read = store.begin_graph_paths(&request).await.map_err(canonical)?;
            let mut pages = Vec::new();
            while let Some(page) = read.next_page().map_err(canonical)? {
                pages.push(page);
            }
            ProjectReadView::GraphPaths { pages }
        }
        ProjectRead::StudySummary { study } => ProjectReadView::StudySummary {
            summary: store.study_summary(&study).await.map_err(canonical)?,
        },
        ProjectRead::RunSummary { run } => ProjectReadView::RunSummary {
            summary: store.run_summary(&run).await.map_err(canonical)?,
        },
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
            let view = cancel
                .until_cancelled(read(store, query, cancel, deadline))
                .await??;
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
