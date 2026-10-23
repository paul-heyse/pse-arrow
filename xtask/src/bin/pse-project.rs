// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dedicated project metadata operator artifact, using the same xtask operation owner.
use clap::{Parser, Subcommand};
#[path = "../project_tools.rs"]
mod project_tools;
#[derive(Parser)]
struct Cli { #[command(subcommand)] command: Command }
#[derive(Subcommand)]
enum Command { Project(project_tools::ProjectArgs) }
fn main() -> anyhow::Result<()> {
    match Cli::parse().command { Command::Project(args) => project_tools::run(args) }
}
