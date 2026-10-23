// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dedicated typed project operator artifact, using existing runtime and storage owners.
#![recursion_limit = "512"]
#![allow(
    clippy::disallowed_types,
    clippy::print_stdout,
    reason = "xtask operator binaries publish structured command output and use the existing host-driver anyhow boundary (blueprint §23.2)"
)]
use clap::{Parser, Subcommand};
#[path = "../project_runtime.rs"]
mod project_runtime;
#[path = "../project_tools.rs"]
mod project_tools;
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Project(project_tools::ProjectArgs),
}
fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Project(args) => project_tools::run(args),
    }
}
