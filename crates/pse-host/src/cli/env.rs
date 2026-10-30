// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `env --print|--explain`: fixed, source-free environment reads.
use super::Context;
use crate::{
    HostError,
    environment::{self, CacheMode, EnvironmentRead},
};
use clap::{ArgGroup, Args};
use std::io::Write as _;

/// Exactly one environment read.
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("operation").required(true).args(["print", "explain", "activation_data"])))]
pub(crate) struct EnvArgs {
    /// Print the shell activation program.
    #[arg(long)]
    pub(crate) print: bool,
    /// Explain the selected environment without evaluating personal code.
    #[arg(long)]
    pub(crate) explain: bool,
    /// Explanation output is JSON.
    #[arg(long, requires = "explain", conflicts_with_all = ["print", "activation_data"])]
    pub(crate) json: bool,
    /// The private activation transfer the printed program reads from a pipe.
    #[arg(long, hide = true)]
    pub(crate) activation_data: bool,
    /// Select the documentation environment.
    #[arg(long)]
    pub(crate) docs: bool,
    /// Compiler-cache intent.
    #[arg(long, value_enum, default_value_t = CacheMode::Auto)]
    pub(crate) cache: CacheMode,
}

pub(crate) fn run(ctx: &Context, args: EnvArgs) -> Result<i32, HostError> {
    let operation = if args.print {
        EnvironmentRead::Print
    } else if args.explain {
        EnvironmentRead::Explain
    } else {
        EnvironmentRead::ActivationData
    };
    let output = environment::read(
        &ctx.checkout_root()?,
        ctx.environment(),
        operation,
        args.docs,
        args.cache,
    )?;
    std::io::stdout().lock().write_all(&output)?;
    Ok(0)
}
