// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded operator access to project metadata; scientific admission remains a Runtime operation.
use anyhow::{Context, Result, ensure};
use clap::{Args, Subcommand};
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore},
    canonical_projects::{Project, ProjectCursor},
};
use serde::Serialize;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, Args)]
pub(crate) struct ProjectArgs {
    /// Selected canonical deployment; opening never installs or repairs schema.
    #[arg(long)]
    state: PathBuf,
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
    /// Save bounded inline raw draft bytes, including malformed or empty text.
    /// Large opaque draft custody uses Runtime's explicit retained-intent API.
    DraftSave {
        #[arg(long)]
        operation: String,
        #[arg(long)]
        project: String,
        #[arg(long)]
        project_generation: u64,
        #[arg(long)]
        branch: String,
        #[arg(long)]
        key: String,
        #[arg(long)]
        base: String,
        #[arg(long)]
        expected_generation: Option<u64>,
        #[arg(long)]
        input: PathBuf,
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
        project: String,
        #[arg(long)]
        project_generation: u64,
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

async fn expected_project(store: &CanonicalStore, key: &str, generation: u64) -> Result<Project> {
    let mut project = store
        .project(key)
        .await?
        .context("selected project is absent")?;
    // Keep the caller's original decision for exact receipt settlement. The
    // native operation checks it before a new effect, after checking its receipt.
    project.generation = generation;
    Ok(project)
}
fn value<T: Serialize>(value: T) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(value)?)
}

pub(crate) fn run(args: ProjectArgs) -> Result<()> {
    ensure!(args.timeout_ms > 0, "positive original allowance required");
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(args.timeout_ms))
        .context("original allowance overflow")?;
    let mut options = CanonicalOptions::from_state(&args.state)?;
    if let Some(database) = args.database {
        options.database = database;
    }
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result=executor.block_on(pse_operations::canonical::operation_scope(deadline,async {
        let store=CanonicalStore::connect(&options).await?;
        let result:Result<serde_json::Value>=async {
            store.open().await?;
            match args.command {
                ProjectCommand::Create{operation,project,title}=>value(store.create_project(&operation,&project,&title).await?),
                ProjectCommand::Show{project}=>value(store.project(&project).await?),
                ProjectCommand::Branches{project,cursor}=>{
                    let cursor=cursor.map(|path|->Result<ProjectCursor>{let metadata=std::fs::metadata(&path)?;ensure!(metadata.is_file()&&metadata.len()<=16*1024,"bounded cursor file required");Ok(serde_json::from_slice(&std::fs::read(path)?)?)}).transpose()?;
                    value(store.project_branches(&project,cursor.as_ref()).await?)
                }
                ProjectCommand::Branch{branch}=>value(store.project_branch(&branch).await?),
                ProjectCommand::DraftShow{key}=>value(store.project_draft(&key).await?),
                ProjectCommand::Settlement{operation}=>value(store.project_acknowledgement(&operation).await?),
                ProjectCommand::DraftSave{operation,project,project_generation,branch,key,base,expected_generation,input}=>{
                    let project=expected_project(&store,&project,project_generation).await?;
                    let branch=store.project_branch(&branch).await?.context("selected branch is absent")?;
                    let metadata=std::fs::metadata(&input)?;
                    ensure!(metadata.is_file()&&metadata.len()<=128*1024,"inline draft input exceeds supported envelope; use the retained Runtime draft intent for larger bodies");
                    let content=std::fs::read(input)?;
                    value(store.save_project_draft(&operation,&project,&branch,&key,&base,expected_generation,&content,None).await?)
                }
                ProjectCommand::ResearchReference{operation,project,project_generation,key,revision,run,attempt,artifact}=>{
                    let project=expected_project(&store,&project,project_generation).await?;
                    let selected=run.as_deref().zip(attempt.as_deref());
                    value(store.retain_research_reference(&operation,&project,&key,&revision,selected,artifact.as_deref()).await?)
                }
            }
        }.await;
        let drained=store.disconnect().await;
        let result=result?;drained?;Ok::<_,anyhow::Error>(result)
    }))?;
    #[derive(Serialize)]
    struct Output {
        version: pse_model::document::Version<1>,
        result: serde_json::Value,
    }
    println!(
        "{}",
        serde_json::to_string(&Output {
            version: pse_model::document::Version,
            result
        })?
    );
    Ok(())
}
