// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded operator access to project metadata; scientific admission remains a Runtime operation.
use super::operator_error::{Context, Result, ensure};
use clap::{Args, Subcommand};
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore},
    canonical_projects::{
        Project, ProjectCursor, ProjectMetadataRequest, ProjectNavigationRequest,
    },
};
use serde::Serialize;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, Args)]
pub(crate) struct ProjectArgs {
    /// Selected canonical deployment for online commands; offline inspect-pack needs none.
    #[arg(long)]
    state: Option<PathBuf>,
    /// Explicit already-initialized canonical database on that deployment.
    #[arg(long)]
    database: Option<String>,
    /// One original allowance for connection, selection and the requested effect.
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,
    #[command(subcommand)]
    command: ProjectCommand,
}
#[derive(Debug, Subcommand)]
enum ProjectCommand {
    /// Verify and inspect only a supplied logical project pack, without opening a service.
    InspectPack {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value_t = super::project_runtime::PROJECT_DOCUMENT_BYTES)]
        maximum_document_bytes: usize,
    },
    /// Apply an original typed navigation decision against its original project descriptor.
    Navigate {
        #[arg(long)]
        operation: String,
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        request: PathBuf,
    },
    /// Read a bounded typed project/draft/bookmark/savepoint/diagnostic metadata page.
    Metadata {
        #[arg(long)]
        request: PathBuf,
    },
    /// Execute a typed runtime action (admission, comparison, merge, history or discovery).
    Runtime(super::project_runtime::RuntimeArgs),
    /// Create project organization, with an original caller-selected operation identity.
    Create {
        #[arg(long)]
        operation: String,
        #[arg(long)]
        project: String,
        #[arg(long)]
        title: String,
    },
    /// Read project metadata; this does not hydrate model or result payloads.
    Show {
        #[arg(long)]
        project: String,
    },
    /// List one exact, generation-fenced branch page.
    Branches {
        #[arg(long)]
        project: String,
        #[arg(long)]
        cursor: Option<PathBuf>,
    },
    /// Read one branch. Its source head remains at the canonical problem owner.
    Branch {
        #[arg(long)]
        branch: String,
    },
    /// Inspect the saved draft descriptor. Artifact references grant no file lifetime.
    DraftShow {
        #[arg(long)]
        key: String,
    },
    /// Observe an original immutable project acknowledgement; absence remains unproved.
    Settlement {
        #[arg(long)]
        operation: String,
    },
    /// Retain exact original source/result lineage without copying fitted values.
    ResearchReference {
        #[arg(long)]
        operation: String,
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        key: String,
        #[arg(long)]
        revision: String,
        #[arg(long, requires = "attempt")]
        run: Option<String>,
        #[arg(long, requires = "run")]
        attempt: Option<String>,
        #[arg(long)]
        artifact: Option<String>,
    },
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    ensure!(file.metadata()?.is_file(), "regular request file required");
    let mut content = Vec::new();
    file.take(16 * 1024 * 1024 + 1).read_to_end(&mut content)?;
    ensure!(
        content.len() <= 16 * 1024 * 1024,
        "request exceeds 16MiB transport limit"
    );
    Ok(serde_json::from_slice(&content)?)
}
fn value<T: Serialize>(value: T) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(value)?)
}

fn output(result: serde_json::Value) -> Result<()> {
    #[derive(Serialize)]
    struct Output {
        version: pse_model::document::Version<1>,
        result: serde_json::Value,
    }
    println!(
        "{}",
        serde_json::to_string(&Output {
            version: pse_model::document::Version,
            result,
        })?
    );
    Ok(())
}

pub(crate) fn run(args: ProjectArgs) -> Result<()> {
    ensure!(args.timeout_ms > 0, "positive original allowance required");
    // Offline source inspection must precede all service, environment and executor selection.
    if let ProjectCommand::InspectPack {
        input,
        maximum_document_bytes,
    } = &args.command
    {
        let bundle =
            pse_runtime::workflow::ProjectBundleReader::open(input, *maximum_document_bytes)?;
        return output(value(bundle.inspect())?);
    }
    let state = args
        .state
        .context("online project commands require explicit --state")?;
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(args.timeout_ms))
        .context("original allowance overflow")?;
    let mut options = CanonicalOptions::from_state(&state)?;
    if let Some(database) = args.database {
        options.database = database;
    }
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result = executor.block_on(pse_operations::canonical::operation_scope(
        deadline,
        async {
            let store = CanonicalStore::connect(&options).await?;
            let result: Result<serde_json::Value> = async {
                store.open().await?;
                match args.command {
                    ProjectCommand::InspectPack { .. } => {
                        Err("offline project inspection was not dispatched".into())
                    }
                    ProjectCommand::Navigate {
                        operation,
                        project,
                        request,
                    } => {
                        let project: Project = read_json(&project)?;
                        let request: ProjectNavigationRequest = read_json(&request)?;
                        value(
                            store
                                .project_navigate(&operation, &project, &request.action)
                                .await?,
                        )
                    }
                    ProjectCommand::Metadata { request } => {
                        let request: ProjectMetadataRequest = read_json(&request)?;
                        value(
                            store
                                .project_metadata(
                                    request.kind,
                                    &request.owner,
                                    request.cursor.as_ref(),
                                )
                                .await?,
                        )
                    }
                    ProjectCommand::Runtime(input) => {
                        super::project_runtime::run(&store, input, deadline).await
                    }
                    ProjectCommand::Create {
                        operation,
                        project,
                        title,
                    } => value(store.create_project(&operation, &project, &title).await?),
                    ProjectCommand::Show { project } => value(store.project(&project).await?),
                    ProjectCommand::Branches { project, cursor } => {
                        let cursor = cursor
                            .map(|path| -> Result<ProjectCursor> {
                                let metadata = std::fs::metadata(&path)?;
                                ensure!(
                                    metadata.is_file() && metadata.len() <= 16 * 1024,
                                    "bounded cursor file required"
                                );
                                Ok(serde_json::from_slice(&std::fs::read(path)?)?)
                            })
                            .transpose()?;
                        value(store.project_branches(&project, cursor.as_ref()).await?)
                    }
                    ProjectCommand::Branch { branch } => {
                        value(store.project_branch(&branch).await?)
                    }
                    ProjectCommand::DraftShow { key } => value(store.project_draft(&key).await?),
                    ProjectCommand::Settlement { operation } => {
                        value(store.project_acknowledgement(&operation).await?)
                    }
                    ProjectCommand::ResearchReference {
                        operation,
                        project,
                        key,
                        revision,
                        run,
                        attempt,
                        artifact,
                    } => {
                        let project: Project = read_json(&project)?;
                        let selected = run.as_deref().zip(attempt.as_deref());
                        value(
                            store
                                .retain_research_reference(
                                    &operation,
                                    &project,
                                    &key,
                                    &revision,
                                    selected,
                                    artifact.as_deref(),
                                )
                                .await?,
                        )
                    }
                }
            }
            .await;
            let drained = store
                .drain_owned_connection_until(tokio::time::Instant::from_std(deadline))
                .await;
            super::operator_error::settle_action(result, drained.map_err(Into::into))
        },
    ))?;
    output(result)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "finite CLI parsing controls require exact successful decoding"
    )]
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        arguments: ProjectArgs,
    }

    #[test]
    fn offline_inspection_parses_without_service_selection() {
        let parsed =
            Cli::try_parse_from(["project", "inspect-pack", "--input", "selected-pack"]).unwrap();
        assert!(parsed.arguments.state.is_none());
        assert!(matches!(
            parsed.arguments.command,
            ProjectCommand::InspectPack { .. }
        ));
    }

    #[test]
    fn online_command_refuses_missing_state_before_service_selection() {
        let parsed =
            Cli::try_parse_from(["project", "show", "--project", "original-project"]).unwrap();
        let error = run(parsed.arguments).unwrap_err();
        assert!(error.to_string().contains("require explicit --state"));
    }

    #[test]
    fn supplied_inspection_reaches_original_bundle_reader_without_a_service() {
        let directory = tempfile::tempdir().unwrap();
        let error = run(ProjectArgs {
            state: None,
            database: None,
            timeout_ms: 30_000,
            command: ProjectCommand::InspectPack {
                input: directory.path().into(),
                maximum_document_bytes: super::super::project_runtime::PROJECT_DOCUMENT_BYTES,
            },
        })
        .unwrap_err();
        assert!(!error.to_string().contains("require explicit --state"));
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    }
}
