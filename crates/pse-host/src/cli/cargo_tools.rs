// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `cargo-tools [NAME...]`: one declared install specification per line (read by bootstrap).
use super::Context;
use crate::{HostError, diagnostic_tools};
use clap::Args;
use std::io::Write as _;

/// Cargo tool selection.
#[derive(Debug, Args)]
pub(crate) struct CargoToolsArgs {
    /// Declared Cargo tool names (none: every declared tool).
    #[arg(value_name = "NAME")]
    pub(crate) names: Vec<String>,
}

pub(crate) fn run(ctx: &Context, args: CargoToolsArgs) -> Result<i32, HostError> {
    let mut stdout = std::io::stdout().lock();
    for spec in diagnostic_tools::cargo_tool_specs(&ctx.checkout_root()?, &args.names)? {
        writeln!(stdout, "{spec}")?;
    }
    Ok(0)
}
