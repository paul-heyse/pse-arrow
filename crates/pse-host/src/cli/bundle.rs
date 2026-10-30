// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Keep-going recipe composition inside one original native foreground lifetime.
use super::{EntryGuard, HostError, ManagedCommand, OutputOptions, checkout_root};
use crate::{
    envelope::{
        CliInvocationId, CommandOutcome, Completeness, ErrorRecord, OutputFormat, Presentation,
        RedactionPolicy, Settlement, SubjectState,
    },
    generation::HostTool,
    invocation::{InvocationEvidence, InvocationStore},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, clap::Args)]
pub(super) struct Options {
    /// Native Just file; relative paths are resolved from the selected checkout.
    #[arg(long)]
    justfile: Option<PathBuf>,
    /// Associate this bundle and its children with an explicit agent session.
    #[arg(long)]
    session: Option<String>,
    /// Independent zero-argument Just recipes; every recipe is attempted in order.
    #[arg(required = true, num_args = 1..)]
    recipes: Vec<String>,
}

#[derive(Debug, clap::Args)]
pub(super) struct Entry {
    #[arg(long)]
    justfile: PathBuf,
    #[arg(long, value_parser = super::parse_invocation_id)]
    invocation: CliInvocationId,
    #[arg(last = true, required = true, num_args = 1..)]
    recipes: Vec<String>,
}
#[derive(Deserialize)]
struct Operation {
    command: Vec<OsString>,
}

/// Each child retains its original result identity and verdict, independent of siblings.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Child {
    /// Exact recipe argument, never interpreted as shell source.
    pub recipe: String,
    /// Original performing invocation, including children whose dispatch failed.
    pub invocation: CliInvocationId,
    /// Original immutable result, absent when dispatch did not publish one.
    pub result: Option<OriginalResult>,
    /// Independently retained dispatch/publication failure.
    pub dispatch_failure: Option<ErrorRecord>,
}

/// Compact original result facts; the complete original response stays at `path`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OriginalResult {
    /// Exact native result path, retaining platform bytes.
    pub path: PathBuf,
    /// Original owner verdict.
    pub outcome: CommandOutcome,
    /// Original subject state.
    pub subject_state: SubjectState,
    /// Original child status, never synthesized from the aggregate exit.
    pub child_exit: Option<i32>,
    /// Original physical reconciliation.
    pub settlement: Settlement,
    /// Required capture and observation coverage.
    pub completeness: Completeness,
    /// Original owner diagnostics.
    pub failures: Vec<ErrorRecord>,
}
impl Child {
    pub(crate) fn complete(&self) -> bool {
        self.dispatch_failure.is_none()
            && self.result.as_ref().is_some_and(|result| {
                result.completeness == Completeness::Complete
                    && matches!(
                        result.settlement,
                        Settlement::Complete | Settlement::NotRequired
                    )
            })
    }
    pub(crate) fn succeeded(&self) -> bool {
        self.complete()
            && self
                .result
                .as_ref()
                .is_some_and(|result| result.outcome == CommandOutcome::Success)
    }
}

/// Completed composition, with one independently attributed row for every requested child.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    /// The original enclosing operation, not the helper's presentation identity.
    pub invocation: CliInvocationId,
    /// Original results in requested order, including failures and absent results.
    pub children: Vec<Child>,
}
impl Summary {
    pub(crate) fn succeeded(&self) -> bool {
        !self.children.is_empty() && self.children.iter().all(Child::succeeded)
    }
    pub(crate) fn complete(&self) -> bool {
        !self.children.is_empty() && self.children.iter().all(Child::complete)
    }
}

fn command(
    root: &Path,
    invocation: &CliInvocationId,
    justfile: &Path,
    recipes: &[String],
    executable: PathBuf,
) -> Vec<OsString> {
    [
        executable.into_os_string(),
        "-C".into(),
        root.as_os_str().to_owned(),
        "--format".into(),
        "json".into(),
        "__bundle".into(),
        "--invocation".into(),
        invocation.as_str().into(),
        "--justfile".into(),
        justfile.as_os_str().to_owned(),
        "--".into(),
    ]
    .into_iter()
    .chain(recipes.iter().map(OsString::from))
    .collect()
}

pub(super) fn start(
    options: Options,
    selected: Option<&Path>,
    entry: &EntryGuard,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let executable = entry
        .prepared()
        .ok_or_else(|| {
            HostError::Ownership(
                "bundle requires prepared host tools; run scripts/bootstrap-host".into(),
            )
        })?
        .executable(&root, HostTool::Host)?;
    let justfile = options
        .justfile
        .map_or_else(|| root.join("justfile"), |path| root.join(path));
    let command = command(
        &root,
        &output.invocation_id,
        &justfile,
        &options.recipes,
        executable,
    );
    super::launch_managed(
        ManagedCommand {
            cwd: root.clone(),
            root,
            command,
            environment: std::env::vars_os().collect(),
            detach: false,
            session: options.session,
            parent: None,
            resource_class: None,
            verb: "run-all",
            tool: None,
            workflow: None,
        },
        entry,
        output,
    )
}

pub(super) fn run(
    args: Entry,
    selected: Option<&Path>,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    let store = InvocationStore::new(crate::paths::CheckoutState::resolve(&root, &environment)?);
    let directory = store.directory(&args.invocation)?;
    let metadata = store.metadata::<Operation>(&args.invocation)?;
    let custody: crate::foreground::ForegroundCustody = directory
        .evidence(InvocationEvidence::Custody)?
        .ok_or_else(|| HostError::Ownership("bundle foreground custody is absent".into()))?;
    let executable = metadata
        .selection
        .control
        .executable(&root, HostTool::Host)?;
    let actual = match pse_diagnostics::process_facts::ProcessReader::default()
        .process(std::process::id())
    {
        pse_diagnostics::process_facts::ProcessRead::Present(facts) => facts.identity,
        _ => {
            return Err(HostError::Ownership(
                "bundle helper process identity unavailable".into(),
            ));
        }
    };
    let native = super::runtime()?.block_on(async {
        let manager = crate::HostManager::connect_user(None).await?;
        let membership = manager.current_membership().await?;
        Ok::<_, HostError>(
            custody.version == 4
                && custody.child == actual
                && custody.manager == manager.identity_key()
                && custody.unit == membership.root().unit
                && membership.matches_root(&custody.root)
                && manager.native_workload(membership.root()).await?,
        )
    })?;
    if !native
        || metadata.selection.command != "run-all"
        || std::env::current_exe()? != executable
        || metadata.selection.operation.command
            != command(
                &root,
                &args.invocation,
                &args.justfile,
                &args.recipes,
                executable,
            )
    {
        return Err(HostError::Ownership(
            "bundle is not the original fixed native foreground child".into(),
        ));
    }
    let policy = RedactionPolicy::from_inputs(&environment);
    let mut children = Vec::with_capacity(args.recipes.len());
    for recipe in args.recipes {
        let invocation = CliInvocationId::new();
        let output = OutputOptions {
            invocation_id: invocation.clone(),
            format: OutputFormat::Json,
            presentation: Presentation::default(),
            artifact_directory: None,
        };
        let performed = super::launch_managed(
            ManagedCommand {
                root: root.clone(),
                cwd: root.clone(),
                environment: environment.clone(),
                command: vec![
                    "just".into(),
                    "--justfile".into(),
                    args.justfile.as_os_str().to_owned(),
                    "--working-directory".into(),
                    root.as_os_str().to_owned(),
                    "--".into(),
                    recipe.clone().into(),
                ],
                detach: false,
                session: metadata.selection.session.clone(),
                parent: Some(args.invocation.clone()),
                resource_class: None,
                verb: "run",
                tool: None,
                workflow: None,
            },
            entry,
            &output,
        );
        // A dispatch/publication failure never suppresses the next independent recipe.
        let mut failures: Vec<HostError> = performed.err().into_iter().collect();
        let result = match store
            .result::<serde::de::IgnoredAny, crate::managed::result::ManagedSummary>(&invocation)
        {
            Ok(Some(result)) => match store.directory(&invocation) {
                Ok(directory) => Some(OriginalResult {
                    path: directory.path().join("result.json"),
                    outcome: result.outcome,
                    subject_state: result.subject_state,
                    child_exit: result.child_exit,
                    settlement: result.settlement,
                    completeness: result.observation.completeness,
                    failures: result.failures,
                }),
                Err(error) => {
                    failures.push(error);
                    None
                }
            },
            Ok(None) => None,
            Err(error) => {
                failures.push(error);
                None
            }
        };
        let failure = failures
            .into_iter()
            .reduce(|operation, cleanup| HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            });
        children.push(Child {
            recipe,
            invocation,
            result,
            dispatch_failure: failure
                .as_ref()
                .map(|error| ErrorRecord::from_typed(error, &policy)),
        });
    }
    let summary = Summary {
        invocation: args.invocation,
        children,
    };
    let exit = if summary.succeeded() { 0 } else { 1 };
    directory.publish_evidence(InvocationEvidence::Bundle, &summary)?;
    Ok(exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_failure_incomplete_and_successful_siblings_remain_distinct() {
        let row = |recipe: &str, outcome, completeness| Child {
            recipe: recipe.into(),
            invocation: CliInvocationId::new(),
            dispatch_failure: None,
            result: Some(OriginalResult {
                path: PathBuf::from(recipe),
                outcome,
                completeness,
                subject_state: if outcome == CommandOutcome::Success {
                    SubjectState::Succeeded
                } else {
                    SubjectState::Failed
                },
                child_exit: Some(if outcome == CommandOutcome::Success {
                    0
                } else {
                    3
                }),
                settlement: Settlement::Complete,
                failures: Vec::new(),
            }),
        };
        let summary = Summary {
            invocation: CliInvocationId::new(),
            children: vec![
                row("failed", CommandOutcome::Failure, Completeness::Complete),
                row("incomplete", CommandOutcome::Success, Completeness::Partial),
                row("passed", CommandOutcome::Success, Completeness::Complete),
            ],
        };
        assert!(!summary.succeeded());
        assert!(!summary.complete());
        assert!(summary.children[0].complete());
        assert!(!summary.children[0].succeeded());
        assert!(!summary.children[1].complete());
        assert!(summary.children[2].succeeded());
        assert_eq!(
            summary.children[0].result.as_ref().unwrap().child_exit,
            Some(3)
        );
    }
    #[test]
    fn bundle_is_new_work_and_hidden_entry_cannot_refresh_itself() {
        use clap::Parser as _;
        let parsed = super::super::Cli::try_parse_from([
            "pse",
            "run-all",
            "--session",
            "test",
            "fmt-check",
            "lint-toml",
        ])
        .unwrap();
        assert!(matches!(
            parsed.command.unwrap().generation_intent(),
            super::super::GenerationIntent::NewWork
        ));
        assert!(super::super::response_schema("run-all", &Presentation::default()).is_ok());
    }
}
