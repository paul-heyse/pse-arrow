// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit semantic sessions and shared native syntax-proposal construction.
use super::{HostError, OutputOptions, SourceParsing};
use crate::{
    envelope::Envelope,
    source_command::batch::CpuTask,
    source_query::{self, Query, QueryError, Strictness},
    source_read::SourceRequest,
};
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
pub(in crate::cli) mod semantic;

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explicit reusable semantic session preparation, passive status and original stop.
    Session {
        #[command(subcommand)]
        action: semantic::Session,
    },
}
impl Action {
    pub(super) fn intent(&self) -> super::GenerationIntent {
        match self {
            Self::Session { action } => action.intent(),
        }
    }
}
#[derive(Debug, Args)]
pub(super) struct Arguments {
    /// Exact source operands, acquired independently without a semantic session.
    #[arg(required=true,num_args=1..)]
    files: Vec<PathBuf>,
    /// Native code-shaped Rust pattern ($NAME or $$$SEQUENCE captures).
    #[arg(long)]
    pattern: String,
    #[arg(long, value_enum, default_value = "smart")]
    strictness: Strictness,
    /// Require an ancestor matching each native pattern.
    #[arg(long)]
    inside: Vec<String>,
    /// Require a proper descendant matching each native pattern.
    #[arg(long)]
    has: Vec<String>,
    /// Exclude a candidate matching any of these native patterns.
    #[arg(long)]
    exclude: Vec<String>,
    /// Extract only this named capture (without sigils); absence retains all captures.
    #[arg(long)]
    capture: Vec<String>,
    #[command(flatten)]
    parsing: SourceParsing,
}
pub(in crate::cli) fn inputs(
    arguments: Arguments,
    selected: Option<&Path>,
    budget: Option<usize>,
) -> Result<(Vec<SourceRequest>, Query), HostError> {
    if arguments
        .parsing
        .language
        .is_some_and(|language| !matches!(language, crate::source_command::Language::Rust))
    {
        return Err(QueryError::Unavailable(
            "structural syntax queries currently support Rust sources only".into(),
        )
        .into());
    }
    let requests = arguments
        .files
        .into_iter()
        .map(|file| {
            let file = selected.map_or_else(
                || Ok(file.clone()),
                |root| std::path::absolute(root.join(&file)),
            )?;
            super::source_request(selected, file, &arguments.parsing)
        })
        .collect::<Result<Vec<_>, HostError>>()?;
    Ok((
        requests,
        Query {
            pattern: arguments.pattern,
            strictness: arguments.strictness,
            inside: arguments.inside,
            has: arguments.has,
            excluded: arguments.exclude,
            captures: arguments.capture,
            edition: arguments.parsing.edition,
            max_records: budget,
        },
    ))
}
pub(in crate::cli) async fn settle<T>(
    mut task: CpuTask<Result<Envelope<T>, QueryError>>,
) -> Result<Envelope<T>, HostError> {
    tokio::select! {
        response=task.settle()=>Ok(response??),
        signal=tokio::signal::ctrl_c()=>{
            task.cancel();
            let response=task.settle().await??;
            signal?;
            Ok(response)
        }
    }
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &super::EntryGuard,
) -> Result<i32, HostError> {
    match action {
        Action::Session { action } => semantic::session(action, selected, entry, output),
    }
}
pub(in crate::cli) fn origin_limits<T>(root: &Path, response: &mut Envelope<T>) {
    if root.to_str().is_none() {
        response.observation.limitations.push("generated-origin next actions cannot encode this non-UTF8 checkout in Unicode argv; exact source identities/locators remain byte-safe".into());
    }
}
pub(in crate::cli) fn origin_routes(
    root: &Path,
    report: &source_query::QueryReport,
) -> Vec<crate::envelope::NextAction> {
    let Ok(map) = crate::ownership::OwnershipMap::read(root) else {
        return vec![];
    };
    report
        .matches
        .iter()
        .filter_map(|matched| {
            let path = matched.locator.source.path();
            let relative = if path.is_absolute() {
                path.strip_prefix(root).ok()?
            } else {
                path
            };
            map.owner(relative).ok().flatten()?;
            crate::source_command::origin_action(root, &matched.locator)
        })
        .collect()
}
