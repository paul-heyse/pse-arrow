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
    /// Retain the exact unissued creation intent outside the physical backup.
    #[arg(long)]
    creation_intent: Option<PathBuf>,
    /// Check stale refusal and fresh creation after explicit restored-store start.
    #[arg(long, requires = "creation_intent", conflicts_with = "seed")]
    check_creation: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    if cli.check_creation {
        if let Some(intent) = &cli.creation_intent {
            executor.block_on(canonical_recovery::creation_authority(
                &cli.state, intent, false,
            ))?;
        }
        println!("stale physical creation authority refused and fresh creation activated");
    } else {
        executor.block_on(canonical_recovery::run(&cli.state, cli.seed))?;
        if cli.seed
            && let Some(intent) = &cli.creation_intent
        {
            executor.block_on(canonical_recovery::creation_authority(
                &cli.state, intent, true,
            ))?;
        }
        println!(
            "native selected source, lineage, product, staged payload and admission gate verified"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_creation_requires_the_exact_saved_intent() -> Result<()> {
        assert!(Cli::try_parse_from(["recovery", "state", "--check-creation"]).is_err());
        assert!(
            Cli::try_parse_from([
                "recovery",
                "state",
                "--check-creation",
                "--creation-intent",
                "intent.json",
                "--seed"
            ])
            .is_err()
        );
        let cli = Cli::try_parse_from([
            "recovery",
            "state",
            "--check-creation",
            "--creation-intent",
            "intent.json",
        ])?;
        assert!(cli.check_creation);
        assert_eq!(cli.creation_intent, Some(PathBuf::from("intent.json")));
        Ok(())
    }
}
