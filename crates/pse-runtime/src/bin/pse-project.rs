// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed product project operator using the ordinary runtime and canonical owners.
#![recursion_limit = "512"]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "product operator reports an actionable terminal refusal"
)]
use clap::{Parser, Subcommand};
#[path = "project/error.rs"]
mod operator_error;
#[path = "project/runtime.rs"]
mod project_runtime;
#[path = "project/tools.rs"]
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
fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command {
        Command::Project(args) => project_tools::run(args),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pse-project: {error}");
            let mut cause = error.source();
            // Keep the original operation cause visible through typed wrappers.
            // A malformed cyclic external source chain cannot loop the operator.
            for _ in 0..16 {
                let Some(source) = cause else {
                    break;
                };
                eprintln!("  caused by: {source}");
                cause = source.source();
            }
            std::process::ExitCode::FAILURE
        }
    }
}
