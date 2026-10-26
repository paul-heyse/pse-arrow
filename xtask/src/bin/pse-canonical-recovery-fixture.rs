// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native canonical recovery control, independent of other xtask feature builds.

#![allow(
    clippy::disallowed_types,
    clippy::print_stdout,
    reason = "xtask recovery driver reports its result and uses anyhow at the tool boundary"
)]

use std::{
    path::PathBuf,
};

use anyhow::{Context, Result, ensure};
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
    /// Seed two actual sealed files for the exclusively owned Python SIGINT control.
    #[arg(long, requires = "signal_results_receipt", conflicts_with_all = ["seed", "creation_intent", "check_creation"])]
    seed_signal_results: bool,
    /// Write the exact run/attempt/relation and verified first-page file inventory.
    #[arg(long, requires = "seed_signal_results")]
    signal_results_receipt: Option<PathBuf>,
    /// Retain the exact unissued creation intent outside the physical backup.
    #[arg(long)]
    creation_intent: Option<PathBuf>,
    /// Check stale refusal and fresh creation after explicit restored-store start.
    #[arg(long, requires = "creation_intent", conflicts_with = "seed")]
    check_creation: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    #[cfg(not(feature = "package-fixtures"))]
    ensure!(
        !cli.seed_signal_results,
        "signal fixture requires package-fixtures and force-validate"
    );
    let state = &cli.state;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    if cli.seed_signal_results {
        #[cfg(feature = "package-fixtures")]
        {
            let destination = cli
                .signal_results_receipt
                .as_ref()
                .context("missing signal receipt")?;
            // Refuse an existing receipt before creating any new store effects.
            let output = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(destination)?;
            let receipt = executor.block_on(canonical_recovery::seed_signal_results(state))?;
            serde_json::to_writer(output, &receipt)?;
            println!("{receipt}");
        }
    } else if cli.check_creation {
        if let Some(intent) = &cli.creation_intent {
            executor.block_on(canonical_recovery::creation_authority(state, intent, false))?;
        }
        println!("stale physical creation authority refused and fresh creation activated");
    } else {
        executor.block_on(canonical_recovery::run(state, cli.seed))?;
        if cli.seed
            && let Some(intent) = &cli.creation_intent
        {
            executor.block_on(canonical_recovery::creation_authority(state, intent, true))?;
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

    #[test]
    fn signal_results_requires_a_distinct_receipt_and_excludes_recovery_seed() -> Result<()> {
        assert!(Cli::try_parse_from(["recovery", "state", "--seed-signal-results"]).is_err());
        assert!(
            Cli::try_parse_from([
                "recovery",
                "state",
                "--seed",
                "--seed-signal-results",
                "--signal-results-receipt",
                "signal.json"
            ])
            .is_err()
        );
        let cli = Cli::try_parse_from([
            "recovery",
            "state",
            "--seed-signal-results",
            "--signal-results-receipt",
            "signal.json",
        ])?;
        assert!(cli.seed_signal_results);
        Ok(())
    }

}
