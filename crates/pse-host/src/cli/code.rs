// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Discoverable explicit syntax queries and immutable source-bound previews.
use super::{HostError, OutputOptions, SourceParsing};
use crate::{
    envelope::{DetailView, Envelope},
    source_command::batch::CpuTask,
    source_query::{self, Query, QueryError, Strictness},
    source_read::SourceRequest,
};
use clap::{Args, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;
mod semantic;

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explicit reusable semantic session preparation, passive status and original stop.
    Session {
        #[command(subcommand)]
        action: semantic::Session,
    },
    /// Explain selected code through native hover, definition and type resolution.
    Explain(semantic::Subject),
    /// Semantic document symbols, distinct from server-free outline.
    Symbols(semantic::Subject),
    /// Native semantic references including declarations.
    Refs(semantic::Subject),
    /// One-hop incoming and outgoing semantic calls.
    Calls(semantic::Subject),
    /// Native implementation targets; no universal trait-applicability claim.
    Implementations(semantic::Subject),
    /// Selected macro expansion text with invocation provenance limits.
    Macro(semantic::Subject),
    /// Related test candidates; does not run analyzer-supplied commands.
    Tests(semantic::Subject),
    /// Contextual completion candidates, preserving unresolved editing intent.
    Candidates(semantic::Subject),
    /// Selected native inlay annotations, without dumping the full file.
    Hints(semantic::Subject),
    /// Resolve one source/generation-bound native inlay hint lazily.
    Hint {
        #[command(flatten)]
        subject: semantic::Subject,
        #[arg(long)]
        select: usize,
    },
    /// Resolve one source-bound completion candidate, retaining imports and snippet intent.
    Candidate {
        #[command(flatten)]
        subject: semantic::Subject,
        #[arg(long)]
        select: usize,
    },
    /// Ask a native trait predicate question; not-proven remains distinct from contradiction.
    Predicate {
        #[command(flatten)]
        subject: semantic::Subject,
        #[arg(long)]
        text: String,
    },
    /// Native source-bound semantic edit previews, without filesystem application.
    Edit {
        #[command(subcommand)]
        action: semantic::Edit,
    },
    /// Follow an exact generated hit to existing declared contributors and regeneration routing.
    Origin {
        source: PathBuf,
        #[arg(long)]
        range_start: usize,
        #[arg(long)]
        range_end: usize,
        /// Refuse when the hit's original source bytes have changed.
        #[arg(long)]
        if_source: Option<String>,
        #[command(flatten)]
        parsing: SourceParsing,
    },
    /// Enumerate written Rust structural candidates and captures; nested matches remain visible.
    Query {
        #[command(flatten)]
        arguments: Arguments,
    },
    /// Inspect immutable capture-preserving replacements; selected overlaps refuse and no files change.
    Preview {
        #[command(flatten)]
        arguments: Arguments,
        /// Native ast-grep template; every named variable must be defined by the positive pattern.
        #[arg(long)]
        replacement: String,
        /// Candidate indices to select; absence retains candidates without selecting edits.
        #[arg(long = "select", value_delimiter = ',')]
        selected: Vec<usize>,
    },
}
impl Action {
    pub(super) fn intent(&self) -> super::GenerationIntent {
        match self {
            Self::Session { action } => action.intent(),
            _ => super::GenerationIntent::Passive,
        }
    }
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Mode {
    Syntax,
    Semantic,
}
#[derive(Debug, Args)]
pub(super) struct Arguments {
    /// Exact source operands, acquired independently without a semantic session.
    #[arg(required=true,num_args=1..)]
    files: Vec<PathBuf>,
    /// Select syntax explicitly. Semantic mode requires its actual provider and never falls back.
    #[arg(long, value_enum)]
    mode: Mode,
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
fn inputs(
    arguments: Arguments,
    selected: Option<&Path>,
    budget: Option<usize>,
) -> Result<(Vec<SourceRequest>, Query), HostError> {
    if matches!(arguments.mode, Mode::Semantic) {
        return Err(QueryError::Unavailable("semantic structural queries are unavailable without their selected synchronized provider; --mode syntax is a separate source-only alternative".into()).into());
    }
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
async fn settle<T>(
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
        Action::Explain(subject) => {
            semantic::query(crate::semantic::Kind::Explain, subject, selected, output)
        }
        Action::Symbols(subject) => {
            semantic::query(crate::semantic::Kind::Symbols, subject, selected, output)
        }
        Action::Refs(subject) => {
            semantic::query(crate::semantic::Kind::Refs, subject, selected, output)
        }
        Action::Calls(subject) => {
            semantic::query(crate::semantic::Kind::Calls, subject, selected, output)
        }
        Action::Implementations(subject) => semantic::query(
            crate::semantic::Kind::Implementations,
            subject,
            selected,
            output,
        ),
        Action::Macro(subject) => {
            semantic::query(crate::semantic::Kind::Macro, subject, selected, output)
        }
        Action::Tests(subject) => {
            semantic::query(crate::semantic::Kind::Tests, subject, selected, output)
        }
        Action::Candidates(subject) => {
            semantic::query(crate::semantic::Kind::Candidates, subject, selected, output)
        }
        Action::Hints(subject) => semantic::hint(subject, None, selected, output),
        Action::Hint { subject, select } => semantic::hint(subject, Some(select), selected, output),
        Action::Candidate { subject, select } => semantic::query(
            crate::semantic::Kind::Candidate { index: select },
            subject,
            selected,
            output,
        ),
        Action::Predicate { subject, text } => semantic::query(
            crate::semantic::Kind::Predicate { text },
            subject,
            selected,
            output,
        ),
        Action::Edit { action } => semantic::edit(action, selected, output),
        Action::Origin {
            source,
            range_start,
            range_end,
            if_source,
            parsing,
        } => {
            let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
            let file = if source.is_absolute() {
                source
            } else {
                root.join(source)
            };
            let response = crate::source_command::generated_origin(
                &root,
                &super::source_request(Some(&root), file, &parsing)?,
                range_start..range_end,
                if_source.as_deref(),
            )?;
            Ok(output.emit_page(response, DetailView::default())?)
        }
        Action::Query { arguments } => {
            let (requests, query) = inputs(arguments, selected, output.presentation.budget)?;
            let mut response = super::runtime()?.block_on(settle(source_query::start_query(
                requests,
                query,
                CancellationToken::new(),
            )))?;
            let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
            response
                .next
                .extend(origin_routes(&root, &response.payload));
            origin_limits(&root, &mut response);
            let omitted = response.payload.omitted_matches;
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: output.presentation.budget,
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..DetailView::default()
                },
            )?)
        }
        Action::Preview {
            arguments,
            replacement,
            selected: edits,
        } => {
            let (requests, query) = inputs(arguments, selected, output.presentation.budget)?;
            let mut response = super::runtime()?.block_on(settle(source_query::start_preview(
                requests,
                query,
                replacement,
                edits,
                CancellationToken::new(),
            )))?;
            let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
            response
                .next
                .extend(origin_routes(&root, &response.payload.query));
            origin_limits(&root, &mut response);
            let omitted = response.payload.query.omitted_matches;
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: output.presentation.budget,
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..DetailView::default()
                },
            )?)
        }
    }
}
fn origin_limits<T>(root: &Path, response: &mut Envelope<T>) {
    if root.to_str().is_none() {
        response.observation.limitations.push("generated-origin next actions cannot encode this non-UTF8 checkout in Unicode argv; exact source identities/locators remain byte-safe".into());
    }
}
fn origin_routes(
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
