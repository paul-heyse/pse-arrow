// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private native canonical process exec; credentials are never command-line values.
#![allow(
    clippy::print_stderr,
    reason = "native helper reports only sanitized terminal boundary errors"
)]
use clap::{Parser, Subcommand};
use pse_operations::native_service::{NativeServiceError, exec_startup, exec_worker, run_worker};
use std::path::PathBuf;
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Exec { payload: PathBuf },
    WorkerExec { payload: PathBuf },
    Worker { payload: PathBuf },
}
fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command {
        Command::Exec { payload } => exec_startup(&payload),
        Command::WorkerExec { payload } => exec_worker(&payload),
        Command::Worker { payload } => match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(run_worker(&payload)),
            Err(error) => Err(error.into()),
        },
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            std::process::ExitCode::FAILURE
        }
    }
}
fn report(error: &NativeServiceError) {
    eprintln!("pse-canonical-service: {error}");
}
