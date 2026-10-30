// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `server prepare [--from-release PATH] | select`: prints the selected executable. This
//! capability does not establish schema readiness or write admission.
use super::Context;
use crate::{HostError, server};
use clap::{Args, Subcommand};
use std::{io::Write as _, path::PathBuf};

/// Server action.
#[derive(Debug, Args)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub(crate) struct ServerArgs {
    /// The action.
    #[command(subcommand)]
    pub(crate) action: ServerAction,
}

/// Server actions.
#[derive(Debug, Subcommand)]
pub(crate) enum ServerAction {
    /// Build the pinned native server, or import a verified source release.
    Prepare {
        /// Verified source release to import instead of building.
        #[arg(long, value_name = "PATH")]
        from_release: Option<PathBuf>,
    },
    /// Select the prepared server; never builds.
    Select,
}

pub(crate) fn run(ctx: &Context, args: ServerArgs) -> Result<i32, HostError> {
    let environment = ctx.environment();
    let selected = match args.action {
        ServerAction::Select => server::select(environment)?,
        ServerAction::Prepare { from_release } => {
            let root = ctx.checkout_root()?;
            let runtime = ctx.runtime()?;
            match from_release {
                Some(release) => {
                    runtime.block_on(server::prepare_from_release(&root, environment, &release))?
                }
                None => runtime.block_on(server::prepare(&root, environment))?,
            }
        }
    };
    writeln!(std::io::stdout(), "{}", selected.executable().display())?;
    Ok(0)
}
