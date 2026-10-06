// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native canonical recovery control, independent of other xtask feature builds.

#![allow(
    clippy::disallowed_types,
    clippy::print_stdout,
    reason = "xtask recovery driver reports its result and uses anyhow at the tool boundary"
)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[path = "../canonical_recovery.rs"]
mod canonical_recovery;

/// Seed or validate the supervisor's isolated canonical recovery fixture.
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// Private supervisor state for the fixture being checked.
    state: PathBuf,
    /// Initialize the fixture before exercising abrupt restart and offline restore.
    #[arg(long)]
    seed: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    executor.block_on(canonical_recovery::run(&cli.state, cli.seed))?;
    println!(
        "native selected source, lineage, product, staged payload and admission gate verified"
    );
    Ok(())
}
