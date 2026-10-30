// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `diagnostic-tools install|verify TOOL`: prints the verified executable path.
use super::Context;
use crate::{
    HostError,
    diagnostic_tools::{self, Tool},
};
use clap::{Args, Subcommand};
use std::io::Write as _;

/// Diagnostic tool action.
#[derive(Debug, Args)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub(crate) struct DiagnosticToolsArgs {
    /// The action.
    #[command(subcommand)]
    pub(crate) action: ToolAction,
}

/// Install or verify one declared tool.
#[derive(Debug, Subcommand)]
pub(crate) enum ToolAction {
    /// Install the pinned release (idempotent), then verify it.
    Install {
        /// Declared diagnostic tool.
        #[arg(value_parser = Tool::parse)]
        tool: Tool,
    },
    /// Verify the installed release without installing.
    Verify {
        /// Declared diagnostic tool.
        #[arg(value_parser = Tool::parse)]
        tool: Tool,
    },
}

pub(crate) fn run(ctx: &Context, args: DiagnosticToolsArgs) -> Result<i32, HostError> {
    let root = ctx.checkout_root()?;
    let path = match args.action {
        ToolAction::Install { tool } => diagnostic_tools::install(&root, tool, ctx.environment())?,
        ToolAction::Verify { tool } => diagnostic_tools::verify(&root, tool, ctx.environment())?,
    };
    writeln!(std::io::stdout(), "{}", path.display())?;
    Ok(0)
}
