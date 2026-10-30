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
#[cfg(feature = "product-operators")]
mod operator_error;
#[path = "project/runtime.rs"]
#[cfg(feature = "product-operators")]
mod project_runtime;
#[path = "project/tools.rs"]
#[cfg(feature = "product-operators")]
mod project_tools;
#[path = "support/signals.rs"]
#[cfg(feature = "product-operators")]
mod signals;
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    #[cfg(feature = "product-operators")]
    Project(project_tools::ProjectArgs),
    /// Read one bounded versioned passive observation request from stdin.
    Observe,
}
fn main() -> std::process::ExitCode {
    let result: Result<i32, Box<dyn std::error::Error + Send + Sync>> = match Cli::parse().command {
        #[cfg(feature = "product-operators")]
        Command::Project(args) => project_tools::run(args).map(|()| 0),
        Command::Observe => {
            pse_runtime::tooling::reader::serve(std::io::stdin().lock(), std::io::stdout().lock())
                .map_err(Into::into)
        }
    };
    match result {
        Ok(code) => std::process::ExitCode::from(u8::try_from(code).unwrap_or(125)),
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
