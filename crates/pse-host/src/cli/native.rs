// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `native prepare --capabilities CAPS`: run explicitly through `scripts/pse-env`; native
//! launches never prepare missing artifacts.
use super::Context;
use crate::{HostError, native};
use clap::{Args, Subcommand};
use std::io::Write as _;

/// Native provider action.
#[derive(Debug, Args)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub(crate) struct NativeArgs {
    /// The action.
    #[command(subcommand)]
    pub(crate) action: NativeAction,
}

fn capabilities(value: &str) -> Result<String, HostError> {
    native::validate_capabilities(value)?;
    Ok(value.to_owned())
}

/// Native provider actions.
#[derive(Debug, Subcommand)]
pub(crate) enum NativeAction {
    /// Prepare immutable provider generations inside authenticated containment.
    Prepare {
        /// Comma-separated subset of compiler,solver,klu,isolation,uno,petsc.
        #[arg(long, required = true, value_name = "CAPABILITIES", value_parser = capabilities)]
        capabilities: String,
    },
}

pub(crate) fn run(ctx: &Context, args: NativeArgs) -> Result<i32, HostError> {
    let NativeAction::Prepare { capabilities } = args.action;
    let root = ctx.checkout_root()?;
    let prepared = native::prepare_explicit(&root, ctx.environment(), &capabilities)?;
    let mut stdout = std::io::stdout().lock();
    for (capability, prefix) in prepared {
        writeln!(stdout, "prepared {capability}: {}", prefix.display())?;
    }
    Ok(0)
}
