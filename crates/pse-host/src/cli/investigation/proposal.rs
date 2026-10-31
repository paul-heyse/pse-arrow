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
