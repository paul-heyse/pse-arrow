// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `agent-hook guard|session-env`: reads the hook payload to EOF; help never reads stdin.
use super::Context;
use crate::{HostError, agent_hooks};
use clap::{Args, Subcommand};
use std::{
    ffi::OsStr,
    io::{Read as _, Write as _},
    path::Path,
};

/// Hook endpoint selection.
#[derive(Debug, Args)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub(crate) struct AgentHookArgs {
    /// The hook event.
    #[command(subcommand)]
    pub(crate) action: HookAction,
}

/// Hook events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub(crate) enum HookAction {
    /// `PreToolUse`: exit 0 to allow, or 2 with `BLOCKED: <reason>`.
    Guard,
    /// `SessionStart`: append the activation to `CLAUDE_ENV_FILE`; always exits 0.
    SessionEnv,
}

pub(crate) fn run(ctx: &Context, args: AgentHookArgs) -> Result<i32, HostError> {
    let mut input = String::new();
    let read = std::io::stdin().lock().read_to_string(&mut input);
    let environment = ctx.environment();
    match args.action {
        HookAction::SessionEnv => {
            let result = ctx.checkout_root().and_then(|root| {
                agent_hooks::session(
                    &root,
                    &input,
                    environment
                        .get(OsStr::new("CLAUDE_ENV_FILE"))
                        .map(Path::new),
                )
            });
            if let Err(error) = result {
                writeln!(
                    std::io::stderr(),
                    "pse-env: session environment unavailable: {error}"
                )?;
            }
            Ok(0)
        }
        HookAction::Guard => {
            let result = read.map_err(HostError::from).and_then(|_| {
                agent_hooks::guard(
                    &ctx.checkout_root()?,
                    &input,
                    environment
                        .get(OsStr::new("PSE_DESIGN_EDIT"))
                        .map(|value| value.as_os_str()),
                )
            });
            match result {
                Ok(()) => Ok(0),
                Err(error) => {
                    writeln!(std::io::stderr(), "BLOCKED: {error}")?;
                    Ok(2)
                }
            }
        }
    }
}
