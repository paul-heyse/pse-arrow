// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dedicated operator artifact; running codegen cannot replace this executable.
#![allow(
    clippy::disallowed_types,
    clippy::print_stdout,
    reason = "xtask operator binaries publish structured command output and use the existing host-driver anyhow boundary (blueprint §23.2)"
)]
use clap::{Parser, Subcommand};
#[path = "../evidence_tools.rs"]
mod evidence_tools;
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Evidence(evidence_tools::EvidenceArgs),
}
fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Evidence(args) => evidence_tools::run(args),
    }
}
