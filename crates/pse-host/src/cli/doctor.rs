// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `doctor [--json]`: exit 0 when every blocking prerequisite holds, else 1.
use super::Context;
use crate::{HostError, doctor};
use clap::Args;
use std::io::Write as _;

/// Doctor options.
#[derive(Debug, Args)]
pub(crate) struct DoctorArgs {
    /// Print the versioned JSON report.
    #[arg(long)]
    pub(crate) json: bool,
}

pub(crate) fn run(ctx: &Context, args: DoctorArgs) -> Result<i32, HostError> {
    let root = ctx.checkout_root()?;
    let report = ctx
        .runtime()?
        .block_on(doctor::inspect(&root, ctx.environment()));
    let mut stdout = std::io::stdout().lock();
    if args.json {
        serde_json::to_writer_pretty(&mut stdout, &report).map_err(std::io::Error::other)?;
        writeln!(stdout)?;
    } else {
        for check in &report.checks {
            writeln!(
                stdout,
                "{}{}: {}: {}",
                if check.ok { "ok" } else { "unavailable" },
                if check.blocking { "" } else { " (optional)" },
                check.name,
                check.detail
            )?;
            if !check.fix.is_empty() {
                writeln!(stdout, "  {}", check.fix)?;
            }
        }
    }
    Ok(i32::from(!report.ready()))
}
