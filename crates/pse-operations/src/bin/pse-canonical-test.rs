// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Run host-admitted source captures inside one newly owned native canonical fixture.
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
#[command(about = "Run captured source tests inside their original native fixture owner")]
struct Cli {
    /// Explicit checkout whose native correctness adapter owns source admission.
    #[arg(long)]
    root: PathBuf,
    /// Host-owned same-invocation build/list inventory; execution does not build again.
    #[arg(long)]
    captured_inventory: PathBuf,
    /// Exact compiler context within the captured inventory.
    #[arg(long)]
    group: usize,
    /// Private create-once outcome path, retaining native exit across fixture cleanup failure.
    #[arg(long)]
    outcome: Option<PathBuf>,
    /// Already built canonical service helper executable.
    #[arg(long)]
    helper: PathBuf,
    /// Exact immutable production module receipt.
    #[arg(long)]
    module_receipt: PathBuf,
    /// Existing parent for the fresh disposable service directory.
    #[arg(long)]
    state_parent: PathBuf,
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
    let tests = NativeTestCommand::from_source_capture(
        &cli.captured_inventory,
        cli.group,
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
        cli.outcome,
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
