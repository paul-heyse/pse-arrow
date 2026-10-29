// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-build first, then run nextest inside one newly owned native canonical fixture.
#![allow(
    clippy::print_stderr,
    reason = "native fixture executable reports sanitized terminal outcomes"
)]
use clap::Parser;
use pse_operations::{
    native_fixture::NativeTestCommand,
    native_service::{NativeModuleSource, NativeServiceError, NativeServiceSpec},
};
use std::{ffi::OsString, path::PathBuf, time::Duration};
#[derive(Parser)]
#[command(
    about = "Build/list validated source tests before starting a fresh disposable canonical service"
)]
struct Cli {
    /// Explicit checkout whose native correctness adapter owns source admission.
    #[arg(long)]
    root: PathBuf,
    /// Already prepared native correctness adapter in this checkout.
    #[arg(long)]
    correctness: PathBuf,
    /// Already installed native cargo-nextest executable.
    #[arg(long)]
    nextest: PathBuf,
    /// Already built canonical service helper executable.
    #[arg(long)]
    helper: PathBuf,
    /// Exact immutable production module receipt.
    #[arg(long)]
    module_receipt: PathBuf,
    /// Existing parent for the fresh disposable service directory.
    #[arg(long)]
    state_parent: PathBuf,
    /// One nextest source-build selection argument; repeat as needed.
    #[arg(long, allow_hyphen_values = true)]
    build_arg: Vec<OsString>,
    /// Execution deadline; settlement has its separate original reserved allowance.
    #[arg(long, default_value_t = 14400)]
    timeout_seconds: u64,
    /// Original reserved physical settlement allowance.
    #[arg(long, default_value_t = 90)]
    settlement_seconds: u64,
    /// Native nextest run-only filter/reporting arguments.
    #[arg(last = true)]
    run_args: Vec<OsString>,
}
fn run(cli: Cli) -> Result<std::process::ExitStatus, NativeServiceError> {
    let tests = NativeTestCommand::prepare(
        cli.correctness,
        cli.nextest,
        cli.build_arg,
        cli.run_args,
        cli.root,
    )?;
    let server = pse_host::server::select(&std::env::vars_os().collect())?;
    let module = pse_host::study_module::select(
        &cli.module_receipt,
        pse_host::study_module::Variant::Production,
    )?;
    let spec = NativeServiceSpec {
        server,
        helper: cli.helper,
        module_source: NativeModuleSource::Prepared(module),
        state_parent: cli.state_parent,
    };
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(tests.run(
        spec,
        Duration::from_secs(cli.timeout_seconds),
        Duration::from_secs(cli.settlement_seconds),
    ))
}
fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(status) => std::process::ExitCode::from(
            status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .unwrap_or(1),
        ),
        Err(error) => {
            eprintln!("pse-canonical-test: {error}");
            match error {
                NativeServiceError::Command { status, .. } => std::process::ExitCode::from(
                    status
                        .code()
                        .and_then(|code| u8::try_from(code).ok())
                        .unwrap_or(1),
                ),
                _ => std::process::ExitCode::FAILURE,
            }
        }
    }
}
