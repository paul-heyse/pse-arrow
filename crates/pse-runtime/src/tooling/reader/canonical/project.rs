// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Connection-only reads using the canonical project owner's native selections and pages.
use super::{MAXIMUM_READ_MILLIS, canonical, connected};
use crate::tooling::reader::{EncodedResponse, Payload, ReaderError, encode_response, response};
use pse_host::investigation::NativePath;
use pse_operations::{
    canonical::{CanonicalError, CanonicalOptions, CanonicalStore},
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

/// One canonical project read using the original metadata and branch cursor contracts.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectRead {
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
}

async fn read(store: &CanonicalStore, query: ProjectRead) -> Result<ProjectReadView, CanonicalError> {
    Ok(match query {
        ProjectRead::Metadata { request } => ProjectReadView::Metadata {
            page: store
                .project_metadata(request.kind, &request.owner, request.cursor.as_ref())
                .await?,
        },
        ProjectRead::Show { project } => ProjectReadView::Show {
            project: store.project(&project).await?,
        },
        ProjectRead::Branches { project, cursor } => ProjectReadView::Branches {
            page: store.project_branches(&project, cursor.as_ref()).await?,
        },
        ProjectRead::Branch { branch } => ProjectReadView::Branch {
            branch: store.project_branch(&branch).await?,
        },
        ProjectRead::DraftShow { key } => ProjectReadView::DraftShow {
            draft: store.project_draft(&key).await?,
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
    let result = (|| {
        cancel.checkpoint()?;
        let options = CanonicalOptions::from_state_for_database(
            &selection.state.to_path_buf(),
            selection.database.as_deref(),
        )
        .map_err(canonical)?;
        connected(&options, deadline, cancel, async |store| {
            let target = ProjectTarget {
                namespace: store.namespace().into(),
                database: store.database().into(),
                issuing_store: store.issuing_store().map_err(canonical)?.into(),
            };
            let view = cancel.until_cancelled(read(store, query)).await?.map_err(canonical)?;
            cancel.checkpoint()?;
            encode_response(response(
                correlation.clone(),
                Ok(Payload::Project {
                    target,
                    result: Box::new(view),
                }),
            ))
        })
    })();
    match result {
        Ok(output) => Ok(output),
        Err(error) => encode_response(response(correlation, Err(error))),
    }
}
