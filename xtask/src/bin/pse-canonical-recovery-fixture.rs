// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native canonical recovery control, independent of other xtask feature builds.

#![allow(
    clippy::disallowed_types,
    clippy::print_stdout,
    reason = "xtask recovery driver reports its result and uses anyhow at the tool boundary"
)]

use std::{
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::PathBuf,
};

#[cfg(feature = "package-fixtures")]
use anyhow::Context;
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
    /// Explicit private mixed-placement storage control; 0 disables it, 1073741824 selects one GiB.
    #[arg(long, default_value_t = 0, conflicts_with_all = ["seed_signal_results", "check_creation"])]
    backup_scale_bytes: u64,
    /// Original storage observations retained outside the physical archive.
    #[arg(long, conflicts_with_all = ["seed_signal_results", "check_creation"])]
    backup_scale_receipt: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    canonical_recovery::check_backup_scale(
        cli.backup_scale_bytes,
        cli.backup_scale_receipt.as_deref(),
    )?;
    anyhow::ensure!(
        cli.backup_scale_bytes == 0 || cfg!(feature = "force-validate"),
        "backup scale fixture requires force-validate"
    );
    // Refuse replacement of the original private witness before any database effect.
    let scale_receipt = cli
        .backup_scale_receipt
        .as_ref()
        .map(|path| -> Result<std::fs::File> {
            let state = cli.state.canonicalize()?;
            let parent = path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("storage receipt parent absent"))?
                .canonicalize()?;
            anyhow::ensure!(
                !parent.starts_with(&state),
                "original storage receipt must remain outside the live state/archive"
            );
            let file = if cli.seed {
                std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .open(path)
            } else {
                anyhow::ensure!(
                    !std::fs::symlink_metadata(path)?.file_type().is_symlink(),
                    "original storage receipt must not be a symlink"
                );
                std::fs::OpenOptions::new().read(true).open(path)
            }?;
            let metadata = file.metadata()?;
            let named = std::fs::symlink_metadata(path)?;
            anyhow::ensure!(
                metadata.is_file()
                    && metadata.mode() & 0o7777 == 0o600
                    && metadata.nlink() == 1
                    && metadata.uid() == state.metadata()?.uid()
                    && metadata.ino() == named.ino()
                    && metadata.dev() == named.dev(),
                "original storage receipt lost its private owned identity"
            );
            Ok(file)
        })
        .transpose()?;
    #[cfg(not(feature = "package-fixtures"))]
    anyhow::ensure!(
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
        if let Some(mut file) = scale_receipt {
            if cli.seed {
                let receipt = executor.block_on(canonical_recovery::seed_backup_scale(state))?;
                serde_json::to_writer(&mut file, &receipt)?;
                file.write_all(b"\n")?;
                file.sync_all()?;
                if let Some(parent) = cli
                    .backup_scale_receipt
                    .as_ref()
                    .and_then(|path| path.parent())
                {
                    std::fs::File::open(parent)?.sync_all()?;
                }
            } else {
                let mut encoded = Vec::new();
                file.take(256 * 1024 + 1).read_to_end(&mut encoded)?;
                anyhow::ensure!(
                    encoded.len() <= 256 * 1024,
                    "original storage receipt exceeds its bounded envelope"
                );
                let receipt = serde_json::from_slice(&encoded)?;
                executor.block_on(canonical_recovery::verify_backup_scale(state, &receipt))?;
            }
            println!("private storage control verified one GiB sealed plus eight inline bytes");
        }
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
    fn backup_scale_is_disabled_by_default_and_requires_original_receipt() -> Result<()> {
        let ordinary = Cli::try_parse_from(["recovery", "state"])?;
        assert_eq!(ordinary.backup_scale_bytes, 0);
        assert!(ordinary.backup_scale_receipt.is_none());
        canonical_recovery::check_backup_scale(0, None)?;
        canonical_recovery::check_backup_scale(
            canonical_recovery::BACKUP_SCALE_BYTES,
            Some(std::path::Path::new("original.json")),
        )?;
        assert!(canonical_recovery::check_backup_scale(1, None).is_err());
        assert!(
            canonical_recovery::check_backup_scale(canonical_recovery::BACKUP_SCALE_BYTES, None)
                .is_err()
        );
        assert!(
            canonical_recovery::check_backup_scale(0, Some(std::path::Path::new("unused.json")))
                .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "recovery",
                "state",
                "--backup-scale-bytes",
                "1073741824",
                "--check-creation",
                "--creation-intent",
                "intent"
            ])
            .is_err()
        );
        Ok(())
    }

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
