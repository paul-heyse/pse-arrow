// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native syntax templates and semantic proposals keep their existing checked producers.
use crate::{
    HostError,
    envelope::{DetailView, OutputOptions},
};
use clap::Subcommand;
use std::path::Path;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Subcommand)]
pub(in crate::cli) enum Action {
    /// Grammar-aware Rust edits at an exact native node, without semantic preparation.
    Grammar {
        /// Working Rust source operand.
        file: std::path::PathBuf,
        /// First byte of the exact syntax node.
        #[arg(long)]
        byte_start: usize,
        /// First byte after the exact syntax node.
        #[arg(long)]
        byte_end: usize,
        /// Require the original source digest before constructing a preview.
        #[arg(long)]
        if_source: Option<String>,
        #[command(subcommand)]
        operation: crate::source_proposal::GrammarEdit,
    },
    /// Native semantic transformations at a known guarded source position.
    #[command(flatten)]
    Semantic(crate::cli::code::semantic::Edit),
    /// Native capture-preserving syntax template, not literal replacement text.
    Syntax {
        #[command(flatten)]
        arguments: crate::cli::code::Arguments,
        #[arg(long)]
        replacement: String,
        /// Select candidate indices; absence retains candidates without choosing edits.
        #[arg(long = "select", value_delimiter = ',')]
        selected: Vec<usize>,
    },
}
pub(in crate::cli) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    match action {
        Action::Grammar {
            file,
            byte_start,
            byte_end,
            if_source,
            operation,
        } => {
            let path = match selected {
                Some(root) => std::path::absolute(root.join(file))?,
                None => std::path::absolute(file)?,
            };
            let source = crate::source_read::SourceRequest::Working { path }
                .acquire()
                .map_err(crate::source_query::QueryError::from)?;
            if if_source
                .as_ref()
                .is_some_and(|digest| digest != source.identity().source_sha256())
            {
                return Err(crate::source_query::QueryError::from(
                    crate::source_proposal::ProposalError::StaleSource(
                        source.identity().path().into(),
                    ),
                )
                .into());
            }
            let proposal =
                crate::source_proposal::preview_grammar(source, byte_start..byte_end, operation)
                    .map_err(crate::source_query::QueryError::from)?;
            let response = crate::envelope::Envelope::response(
                "propose grammar",
                "source-guarded grammar preview; no source application or compiler acceptance",
                proposal,
            );
            Ok(output.emit(response)?)
        }
        Action::Semantic(action) => crate::cli::code::semantic::edit(action, selected, output),
        Action::Syntax {
            arguments,
            replacement,
            selected: edits,
        } => {
            let (requests, query) =
                crate::cli::code::inputs(arguments, selected, output.presentation.budget)?;
            let mut response = crate::cli::runtime()?.block_on(crate::cli::code::settle(
                crate::source_query::start_preview(
                    requests,
                    query,
                    replacement,
                    edits,
                    CancellationToken::new(),
                ),
            ))?;
            response.command = "propose syntax".into();
            let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
            response.next.extend(crate::cli::code::origin_routes(
                &root,
                &response.payload.query,
            ));
            crate::cli::code::origin_limits(&root, &mut response);
            let omitted = response.payload.query.omitted_matches;
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: output.presentation.budget,
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..Default::default()
                },
            )?)
        }
    }
}
