// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original compiler evidence navigation; this boundary never starts Cargo or a probe.
use super::{HostError, OutputOptions, checkout_root};
use crate::{
    compiler_investigation::{self, DetailCursor, DiagnosticAddress, DiagnosticCursor},
    envelope::DetailView,
};
use clap::Subcommand;
use std::{num::NonZeroUsize, path::Path};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Page original retained check failures; related native detail is a separate observation.
    Diagnostics {
        invocation: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Read one addressed original rustc diagnostic, its suggestions and related spans.
    Diagnostic {
        /// Opaque original address returned by compiler diagnostics.
        #[arg(long)]
        address: String,
        #[arg(long)]
        cursor: Option<String>,
        /// Include exact original JSON as opaque evidence, preserving original numeric lexemes.
        #[arg(long)]
        raw: bool,
        /// Compare retained native source lines to independently acquired current candidates.
        #[arg(long)]
        source: bool,
        /// Whole-record decoding bound; oversized originals stay in their raw artifact.
        #[arg(long, default_value = "4194304")]
        max_record_bytes: NonZeroUsize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    #[test]
    fn original_evidence_is_passive_and_requires_explicit_record_selection() {
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "compiler",
            "diagnostic",
            "--address",
            "{}",
            "--source",
            "--raw",
            "--budget",
            "2",
        ])
        .unwrap();
        assert_eq!(
            cli.command.as_ref().unwrap().generation_intent(),
            super::super::GenerationIntent::Passive
        );
        let Some(super::super::Command::Compiler {
            action: Action::Diagnostic { source, raw, .. },
        }) = cli.command
        else {
            panic!("selected original reader")
        };
        assert!(source && raw);
        assert!(super::super::Cli::try_parse_from(["pse", "compiler", "diagnostic"]).is_err());
        assert!(
            super::super::Cli::try_parse_from([
                "pse",
                "compiler",
                "diagnostics",
                "latest",
                "--detach"
            ])
            .is_err()
        );
    }
}
fn decoded<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, HostError> {
    serde_json::from_str(text)
        .map_err(std::io::Error::other)
        .map_err(HostError::from)
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let store = super::invocation_store(selected)?;
    let limit = NonZeroUsize::new(output.presentation.budget.unwrap_or(20))
        .ok_or_else(|| HostError::Ownership("compiler detail allowance must be positive".into()))?;
    match action {
        Action::Diagnostics {
            invocation,
            session,
            cursor,
        } => {
            let environment = std::env::vars_os().collect();
            let session = crate::invocation::select_session(session, &environment)?;
            let cursor: Option<DiagnosticCursor> = cursor.as_deref().map(decoded).transpose()?;
            let response = compiler_investigation::diagnostics(
                &store,
                &invocation,
                session.as_deref(),
                cursor.as_ref(),
                limit,
                &root,
            )?;
            let omitted = response
                .payload
                .retained_references
                .saturating_sub(response.payload.records.len());
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: Some(limit.get()),
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..DetailView::default()
                },
            )?)
        }
        Action::Diagnostic {
            address,
            cursor,
            raw,
            source,
            max_record_bytes,
        } => {
            let address: DiagnosticAddress = decoded(&address)?;
            let cursor: Option<DetailCursor> = cursor.as_deref().map(decoded).transpose()?;
            let evidence = compiler_investigation::diagnostic(&store, &address, max_record_bytes)?;
            let response = evidence.page(&root, cursor.as_ref(), limit, raw, source)?;
            let omitted = response
                .payload
                .total_related_records
                .saturating_sub(response.payload.records.len());
            Ok(output.emit_page(
                response,
                DetailView {
                    budget: Some(limit.get()),
                    omitted_records: omitted,
                    truncated: omitted > 0,
                    ..DetailView::default()
                },
            )?)
        }
    }
}
