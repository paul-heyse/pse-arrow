// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Product-only native physical recovery. Application freeze belongs to the supervisor.

#![allow(clippy::print_stdout, clippy::print_stderr, reason = "product CLI reports bounded JSON results and actionable errors")]

use clap::Parser;
use pse_operations::canonical_files::offline::{self, Operation, Placement};
use std::{io::{self, BufReader, Write}, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long)]
    operation: String,
    #[arg(long)]
    root: PathBuf,
    #[arg(long)]
    inventory: Option<PathBuf>,
    #[arg(long)]
    destination: Option<PathBuf>,
    /// Original placement, including when the catalog is empty.
    #[arg(long)]
    placement: Option<String>,
    /// Fresh destination placement for restore.
    #[arg(long)]
    target: Option<String>,
}

fn run() -> Result<(), pse_operations::canonical::CanonicalError> {
    let cli = Cli::parse();
    let error = |error: serde_json::Error| pse_operations::canonical::CanonicalError::Configuration(error.to_string());
    let placement = cli.placement.as_deref().map(serde_json::from_str::<Placement>).transpose().map_err(error)?;
    let target = cli.target.as_deref().map(serde_json::from_str::<Placement>).transpose().map_err(error)?;
    let stdin = io::stdin();
    let mut input: Box<dyn io::BufRead> = if let Some(path) = &cli.inventory {
        Box::new(BufReader::new(std::fs::File::open(path).map_err(|e| pse_operations::canonical::CanonicalError::Configuration(e.to_string()))?))
    } else { Box::new(stdin.lock()) };
    let mut output = io::stdout().lock();
    match cli.operation.as_str() {
        "inventory" | "backup" | "verify-live" | "verify" | "restore" => {
            let operation = match cli.operation.as_str() {
                "inventory" => Operation::Inventory, "backup" => Operation::Backup,
                "verify-live" => Operation::VerifyLive, "verify" => Operation::Verify, _ => Operation::Restore,
            };
            offline::run(operation, &cli.root, placement.as_ref(), cli.destination.as_deref(), target.as_ref(), &mut input, &mut output)
        }
        "retired-check" | "retired-remove" => {
            let placement = placement.ok_or_else(|| pse_operations::canonical::CanonicalError::Configuration("retired operation requires placement".into()))?;
            offline::retired(&cli.root, &placement, &mut input, cli.operation == "retired-remove")
        }
        "layout" => {
            let placement = placement.ok_or_else(|| pse_operations::canonical::CanonicalError::Configuration("layout requires placement".into()))?;
            serde_json::to_writer(&mut output, &serde_json::json!({"root": placement.root(&cli.root)?})).map_err(error)?;
            output.write_all(b"\n").map_err(|e| pse_operations::canonical::CanonicalError::Configuration(e.to_string()))
        }
        "owner-observations" => {
            while let Some(bytes) = offline::read_entry(&mut input)? {
                let owner: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(&bytes).map_err(error)?;
                let result = match offline::owner_dead(&owner) {
                    Ok(dead) => serde_json::json!({"key": owner.get("key"), "dead": dead}),
                    Err(error) => serde_json::json!({"key": owner.get("key"), "refusal": error.to_string()}),
                };
                serde_json::to_writer(&mut output, &result).map_err(error)?;
                output.write_all(b"\n").map_err(|e| pse_operations::canonical::CanonicalError::Configuration(e.to_string()))?;
            }
            Ok(())
        }
        _ => Err(pse_operations::canonical::CanonicalError::Configuration("unknown offline physical operation".into())),
    }
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => { eprintln!("{error}"); std::process::ExitCode::FAILURE }
    }
}
