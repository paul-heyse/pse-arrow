// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native canonical recovery control, independent of other xtask feature builds.

#![allow(
    clippy::disallowed_types,
    clippy::print_stdout,
    reason = "xtask recovery driver reports its result and uses anyhow at the tool boundary"
)]

use std::{io::{BufRead, BufReader, Read}, path::{Path, PathBuf}};

use anyhow::{Context, Result, ensure};
use clap::Parser;

#[path = "../canonical_recovery.rs"]
mod canonical_recovery;

/// Seed or validate the supervisor's isolated canonical recovery fixture.
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// Private supervisor state for the fixture being checked.
    #[arg(required_unless_present = "verify_payload_inventory", conflicts_with = "verify_payload_inventory")]
    state: Option<PathBuf>,
    /// Verify exact canonical file digests offline, without opening a database.
    #[arg(long, conflicts_with_all = ["seed", "creation_intent", "check_creation"])]
    verify_payload_inventory: Option<PathBuf>,
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
    if let Some(inventory) = &cli.verify_payload_inventory {
        let count = verify_payload_inventory(inventory)?;
        println!("{}", serde_json::json!({"verified_payload_files": count}));
        return Ok(());
    }
    let state = cli.state.as_deref().context("missing recovery state")?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    if cli.check_creation {
        if let Some(intent) = &cli.creation_intent {
            executor.block_on(canonical_recovery::creation_authority(
                state, intent, false,
            ))?;
        }
        println!("stale physical creation authority refused and fresh creation activated");
    } else {
        executor.block_on(canonical_recovery::run(state, cli.seed))?;
        if cli.seed
            && let Some(intent) = &cli.creation_intent
        {
            executor.block_on(canonical_recovery::creation_authority(
                state, intent, true,
            ))?;
        }
        println!(
            "native selected source, lineage, product, staged payload and admission gate verified"
        );
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PayloadEntry {
    path: PathBuf,
    instance: String,
    digest: String,
    #[serde(deserialize_with = "catalog_uint")]
    bytes: u64,
    encoding: String,
    #[serde(deserialize_with = "catalog_uint")]
    batch_count: u64,
    schema: String,
    interpretation: String,
}

fn catalog_uint<'de, D: serde::Deserializer<'de>>(decoder: D) -> std::result::Result<u64, D::Error> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum UInt { Text(String), Number(u64) }
    match <UInt as serde::Deserialize>::deserialize(decoder)? {
        UInt::Number(value) => Ok(value),
        UInt::Text(text) if !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()) =>
            text.parse().map_err(serde::de::Error::custom),
        UInt::Text(_) => Err(serde::de::Error::custom("canonical extent is not an unsigned decimal")),
    }
}

const INVENTORY_LINE_BYTES: usize = 64 * 1024;

fn inventory_line(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let bytes = reader.take((INVENTORY_LINE_BYTES + 1) as u64).read_until(b'\n', &mut line)?;
    if bytes == 0 { return Ok(None); }
    ensure!(bytes <= INVENTORY_LINE_BYTES, "payload inventory entry exceeds its bounded envelope");
    Ok(Some(line))
}

fn verify_payload_inventory(path: &Path) -> Result<u64> {
    let mut input = BufReader::new(std::fs::File::open(path)?);
    let mut count = 0u64;
    while let Some(line) = inventory_line(&mut input)? {
        let entry: PayloadEntry = serde_json::from_slice(&line)?;
        ensure!(entry.path.is_absolute() && !entry.path.components().any(|part| matches!(part, std::path::Component::ParentDir)), "payload inventory requires a derived absolute path");
        let descriptor = pse_operations::canonical_files::FileDescriptor {
            instance: entry.instance, digest: entry.digest, bytes: entry.bytes,
            encoding: entry.encoding, batch_count: entry.batch_count,
            schema: entry.schema, interpretation: entry.interpretation,
        };
        pse_operations::canonical_files::verify_payload_file(&entry.path, &descriptor)?;
        count = count.checked_add(1).context("payload inventory count overflow")?;
    }
    Ok(count)
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
    fn payload_verification_is_offline_and_exclusive() -> Result<()> {
        let cli = Cli::try_parse_from(["recovery", "--verify-payload-inventory", "inventory.jsonl"])?;
        assert!(cli.state.is_none());
        assert!(Cli::try_parse_from(["recovery", "state", "--verify-payload-inventory", "inventory.jsonl"]).is_err());
        assert!(Cli::try_parse_from(["recovery", "--seed", "--verify-payload-inventory", "inventory.jsonl"]).is_err());
        Ok(())
    }

    #[test]
    fn payload_inventory_bounds_before_decoding() -> Result<()> {
        let mut large = std::io::Cursor::new(vec![b'x'; INVENTORY_LINE_BYTES + 8]);
        assert!(inventory_line(&mut large).is_err());
        assert_eq!(large.position(), (INVENTORY_LINE_BYTES + 1) as u64);
        let number: serde_json::Value = serde_json::json!({"bytes": "18446744073709551615"});
        #[derive(serde::Deserialize)]
        struct Extent { #[serde(deserialize_with="catalog_uint")] bytes: u64 }
        assert_eq!(serde_json::from_value::<Extent>(number)?.bytes, u64::MAX);
        assert!(serde_json::from_value::<Extent>(serde_json::json!({"bytes":"18446744073709551616"})).is_err());
        assert!(serde_json::from_value::<Extent>(serde_json::json!({"bytes":"1.0"})).is_err());
        Ok(())
    }
}
