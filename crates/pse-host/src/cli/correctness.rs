// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `cargo pse-correctness`: Cargo's external subcommand. Failures print
//! `arrow-validation: …` and exit 2; Cargo and runner exits are forwarded.
//!
//! Adapter mode (Cargo through nextest, or cargo-hack) receives Cargo's own argv, which is
//! not this grammar, so it bypasses clap entirely.
use super::{VERSION, exit, parse_failure};
use crate::correctness::{self, CorrectnessError};
use clap::{Args, CommandFactory as _, FromArgMatches as _, Parser, Subcommand};
use std::{
    ffi::OsString,
    io::{self, Write as _},
    process::ExitCode,
};

/// Cargo's external-subcommand convention passes the subcommand name as argv[1].
const CARGO_SUBCOMMAND: &str = "pse-correctness";

/// Source-build correctness only. Nextest supports this executable's checkout; foreign
/// workspaces, manifest/config overrides and archived or reused binaries are refused before
/// nextest starts.
#[derive(Debug, Parser)]
#[command(
    name = "cargo-pse-correctness",
    bin_name = "cargo pse-correctness",
    version = VERSION,
    arg_required_else_help = true,
    args_conflicts_with_subcommands = true,
    disable_help_subcommand = true
)]
pub(crate) struct CorrectnessCli {
    /// Nextest setup action: restore native Cargo for actual test attempts.
    #[arg(long, hide = true, exclusive = true)]
    pub(crate) nextest_test_env: bool,
    /// The Cargo mode.
    #[command(subcommand)]
    pub(crate) mode: Option<Mode>,
}

/// Arguments forwarded to Cargo or nextest byte for byte; `-h` here belongs to them.
#[derive(Debug, Args)]
#[command(disable_help_flag = true)]
pub(crate) struct PassThrough {
    /// Cargo or nextest arguments.
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        num_args = 0..,
        value_name = "ARGS",
        value_parser = clap::value_parser!(OsString)
    )]
    pub(crate) arguments: Vec<OsString>,
}

/// Cargo compiler modes, each run with the validated feature closure.
#[derive(Debug, Subcommand)]
pub(crate) enum Mode {
    /// `cargo check`.
    Check(PassThrough),
    /// `cargo build`.
    Build(PassThrough),
    /// `cargo test`.
    Test(PassThrough),
    /// `cargo bench`.
    Bench(PassThrough),
    /// `cargo run`.
    Run(PassThrough),
    /// `cargo clippy`.
    Clippy(PassThrough),
    /// Native nextest, with its builds routed through this adapter.
    #[command(subcommand)]
    Nextest(NextestMode),
}

/// The nextest source-build modes.
#[derive(Debug, Subcommand)]
pub(crate) enum NextestMode {
    /// `cargo nextest run`.
    Run(PassThrough),
    /// `cargo nextest list`.
    List(PassThrough),
}

/// Parsed request: the validated mode, forwarded with the caller's original argv.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Request {
    /// Nextest's setup action.
    TestEnvironment,
    /// A Cargo compiler mode: the mode word and everything after it.
    Cargo(Vec<OsString>),
    /// `nextest run|list` and everything after it.
    Nextest(Vec<OsString>),
}

/// Validate `arguments` (argv without the program and Cargo's subcommand word) against the
/// grammar. Forwarded argv is the caller's own slice, not clap's values: clap consumes a
/// leading `--`, which Cargo must still receive.
pub(crate) fn parse(arguments: &[OsString]) -> Result<Request, clap::Error> {
    let program = OsString::from("cargo-pse-correctness");
    let matches = CorrectnessCli::command()
        .try_get_matches_from(std::iter::once(&program).chain(arguments))?;
    let cli = CorrectnessCli::from_arg_matches(&matches)?;
    Ok(match cli.mode {
        None => Request::TestEnvironment,
        Some(Mode::Nextest(_)) => Request::Nextest(arguments.to_vec()),
        Some(_) => Request::Cargo(arguments.to_vec()),
    })
}

fn failure(message: &dyn std::fmt::Display) -> ExitCode {
    let _ = writeln!(io::stderr(), "arrow-validation: {message}");
    ExitCode::from(2)
}

/// The `cargo-pse-correctness` entry point, after the trampoline check.
pub(crate) fn main(argv: &[OsString]) -> ExitCode {
    let mut arguments = argv.get(1..).unwrap_or_default();
    if arguments
        .first()
        .is_some_and(|word| word == CARGO_SUBCOMMAND)
    {
        arguments = &arguments[1..];
    }
    let result: Result<i32, CorrectnessError> = if correctness::adapter_mode() {
        correctness::adapter(arguments)
    } else {
        match parse(arguments) {
            Ok(Request::TestEnvironment) => correctness::restore_test_environment(),
            Ok(Request::Cargo(arguments)) => correctness::cargo(&arguments),
            Ok(Request::Nextest(arguments)) => correctness::nextest(&arguments),
            Err(error) => return parse_failure(&error, |line| failure(&line)),
        }
    };
    match result {
        Ok(code) => exit(code),
        Err(error) => failure(&error),
    }
}
