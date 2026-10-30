// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Forward exact captured native parser inputs through the one managed invocation owner.
use super::{
    GenerationIntent, HostError, ManagedCommand, NativeToolSelection, OutputOptions, checkout_root,
};
use crate::{
    envelope::Envelope,
    generation::{EntryGuard, EntrySettlement},
    prepared_xtask::{self, PreparedXtask},
};
use clap::{Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    io::{self, Write},
    path::Path,
};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Build/capture the declared native tool; passive queries never invoke this operation.
    Prepare {
        #[arg(value_enum)]
        tool: Tool,
    },
    /// Read the selected native parser's retained catalogue, with captured-source limits.
    Catalog {
        #[arg(long)]
        selection: String,
    },
    /// Read the actual retained native document-owner response schema.
    OwnerSchema {
        #[arg(long)]
        selection: String,
    },
    /// Run exact arguments through the selected native parser under managed ownership.
    Run {
        #[arg(long)]
        selection: String,
        #[arg(long)]
        detach: bool,
        #[arg(long)]
        session: Option<String>,
        #[arg(last=true, required=true, num_args=1..)]
        arguments: Vec<OsString>,
    },
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum Tool {
    Xtask,
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Prepare { .. } | Self::Run { .. } => GenerationIntent::NewWork,
            _ => GenerationIntent::Passive,
        }
    }
    pub(super) fn raw_protocol(&self) -> bool {
        matches!(self, Self::Run { .. })
    }
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct PreparedTool {
    tool: &'static str,
    selection: String,
    source_identity: String,
    executable_sha256: String,
    features: BTreeSet<String>,
}
fn tool_error(error: prepared_xtask::PreparedXtaskError) -> HostError {
    HostError::PreparedXtask(Box::new(error))
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
    entry: &EntryGuard,
    settlement: &mut EntrySettlement,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    match action {
        Action::Prepare { tool: Tool::Xtask } => {
            *settlement = EntrySettlement::Unresolved;
            let tool = prepared_xtask::prepare(&root, &environment).map_err(tool_error)?;
            Ok(output.emit(Envelope::response(
                "tools prepare",
                "native development tool explicitly captured",
                PreparedTool {
                    tool: "xtask",
                    selection: tool.id().into(),
                    source_identity: tool.source_identity().map_err(tool_error)?,
                    executable_sha256: tool.executable().sha256.clone(),
                    features: tool.features().clone(),
                },
            ))?)
        }
        Action::Catalog { selection } => {
            let tool = PreparedXtask::open(&root, &environment, &selection).map_err(tool_error)?;
            Ok(output.emit(super::catalog_envelope(
                tool.catalog().map_err(tool_error)?,
                "tools catalog",
                format!("captured xtask selection {}", tool.id()),
            ))?)
        }
        Action::OwnerSchema { selection } => {
            let tool = PreparedXtask::open(&root, &environment, &selection).map_err(tool_error)?;
            serde_json::to_writer(
                io::stdout().lock(),
                &tool.owner_schema().map_err(tool_error)?,
            )
            .map_err(io::Error::other)?;
            writeln!(io::stdout().lock())?;
            Ok(0)
        }
        Action::Run {
            selection,
            detach,
            session,
            arguments,
        } => {
            let tool = PreparedXtask::open(&root, &environment, &selection).map_err(tool_error)?;
            let evidence = NativeToolSelection {
                kind: "xtask",
                selection: tool.id().into(),
                source_identity: tool.source_identity().map_err(tool_error)?,
                executable_sha256: tool.executable().sha256.clone(),
            };
            let payload = tool.launch(arguments, environment).map_err(tool_error)?;
            let command = std::iter::once(payload.program.into_os_string())
                .chain(payload.arguments)
                .collect();
            super::launch_managed(
                ManagedCommand {
                    root,
                    cwd: payload.cwd,
                    environment: payload.environment,
                    command,
                    detach,
                    session,
                    resource_class: None,
                    verb: "tools run",
                    tool: Some(evidence),
                    workflow: None,
                },
                entry,
                output,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn tool_parser_tail_stays_literal_and_discovery_remains_passive() {
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "tools",
            "run",
            "--selection",
            "selection",
            "--",
            "owner",
            "--format",
            "human",
        ])
        .unwrap();
        let Some(super::super::Command::Tools {
            action: Action::Run { arguments, .. },
        }) = cli.command
        else {
            panic!("native forwarding selection")
        };
        assert_eq!(arguments, ["owner", "--format", "human"]);
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "tools",
            "catalog",
            "--selection",
            "selection",
        ])
        .unwrap();
        assert_eq!(
            cli.command.unwrap().generation_intent(),
            GenerationIntent::Passive
        );
    }
}
