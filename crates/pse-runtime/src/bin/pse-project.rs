// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed product project operator using the ordinary runtime and canonical owners.
#![recursion_limit = "512"]
#![allow(clippy::print_stdout, clippy::print_stderr, reason = "product operator reports an actionable terminal refusal")]
use clap::{Parser, Subcommand};
#[path = "project/runtime.rs"]
mod project_runtime;
#[path = "project/tools.rs"]
mod project_tools;
#[path = "project/error.rs"]
mod operator_error;
#[derive(Parser)]
struct Cli { #[command(subcommand)] command: Command }
#[derive(Subcommand)]
enum Command { Project(project_tools::ProjectArgs) }
fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command { Command::Project(args) => project_tools::run(args) };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => { eprintln!("pse-project: {error}"); std::process::ExitCode::FAILURE }
    }
}
