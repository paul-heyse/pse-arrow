// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The one host command tree behind the `pse`, `pse-host` and `cargo-pse-correctness` bins.
//!
//! Each verb is one file here holding its `#[derive(Args)]` struct and
//! `fn run(ctx: &Context, args) -> Result<i32, HostError>`, delegating to the owning module.
//! Adding a verb is one `Command` variant and one `dispatch` arm.
use crate::{HostError, environment::Environment, foreground};
use clap::{CommandFactory as _, FromArgMatches as _, Parser, Subcommand, error::ErrorKind};
use std::{
    ffi::OsString,
    io::{self, Write as _},
    path::PathBuf,
    process::ExitCode,
};

mod activity;
mod agent_hook;
pub mod args;
mod boundary;
mod cargo_tools;
mod correctness;
mod diagnostic_tools;
mod doctor;
mod env;
mod native;
mod run;
mod server;
mod study_module;
mod telemetry_service;
#[cfg(test)]
mod tests;

pub use args::{Format, GlobalArgs, OutputArgs};

/// Which thin binary entered the tree. All three share one build, generation and contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinKind {
    /// The agent-facing front door.
    Pse,
    /// The same tree, exec'd by hooks, seeds and tests.
    PseHost,
    /// Cargo's external subcommand and invocation-local Cargo adapter.
    CargoPseCorrectness,
}
impl BinKind {
    /// The executable name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pse => "pse",
            Self::PseHost => "pse-host",
            Self::CargoPseCorrectness => "cargo-pse-correctness",
        }
    }
}

/// Every probe line names the binary, its version and the host protocol.
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " protocol 1");

/// Native host capabilities and checkout workflows.
#[derive(Debug, Parser)]
#[command(
    name = "pse",
    version = VERSION,
    subcommand_required = true,
    arg_required_else_help = true,
    disable_help_subcommand = true,
    after_help = "Construction/update is explicit: scripts/bootstrap-host. Reads never build."
)]
pub(crate) struct Cli {
    /// Options accepted by every verb.
    #[command(flatten)]
    pub(crate) globals: GlobalArgs,
    /// The selected verb.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// One variant per verb; the argument struct lives in the verb's own file.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Run a command in the checkout environment under an owned foreground scope (`pse-env`).
    #[command(name = "boundary")]
    Boundary(boundary::BoundaryArgs),
    /// Agent harness hook endpoint: exits 0, or 2 with `BLOCKED: …`.
    #[command(name = "agent-hook")]
    AgentHook(agent_hook::AgentHookArgs),
    /// Read-only native prerequisites; no build, repair, database or scientific startup.
    Doctor(doctor::DoctorArgs),
    /// Print the declared `cargo install` specifications for the named Cargo tools.
    #[command(name = "cargo-tools")]
    CargoTools(cargo_tools::CargoToolsArgs),
    /// Install or verify one pinned diagnostic tool.
    #[command(name = "diagnostic-tools")]
    DiagnosticTools(diagnostic_tools::DiagnosticToolsArgs),
    /// Explicit persistent local telemetry service operator.
    #[command(name = "telemetry-service")]
    TelemetryService(telemetry_service::TelemetryServiceArgs),
    /// Explicit native provider preparation.
    Native(native::NativeArgs),
    /// Prepare or select the pinned native database server executable.
    Server(server::ServerArgs),
    /// Prepare or select the study database module.
    #[command(name = "study-module")]
    StudyModule(study_module::StudyModuleArgs),
    /// Fixed, source-free checkout environment reads.
    Env(env::EnvArgs),
    /// Observe a user-manager unit and this process's kernel facts, without activation.
    Activity(activity::ActivityArgs),
    /// Run an absolute executable in an owned foreground scope with the caller environment.
    Run(run::RunArgs),
}

/// The verbs `pse -h` hides; they stay available in every binary.
const HIDDEN_FROM_PSE: [&str; 2] = ["boundary", "agent-hook"];

/// Per-invocation facts every handler reads. Later packages add output settings (A03) and
/// state paths (A04) here; handlers take `&Context` so those additions reach every verb.
#[derive(Debug)]
pub struct Context {
    bin: BinKind,
    globals: GlobalArgs,
    environment: Environment,
}
impl Context {
    /// Compose one invocation's context.
    pub fn new(bin: BinKind, globals: GlobalArgs, environment: Environment) -> Self {
        Self {
            bin,
            globals,
            environment,
        }
    }
    /// The binary that was invoked.
    pub fn bin(&self) -> BinKind {
        self.bin
    }
    /// Parsed global options.
    pub fn globals(&self) -> &GlobalArgs {
        &self.globals
    }
    /// The caller's environment, captured once; handlers never read `std::env` directly.
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
    /// The selected checkout: `-C`, else the published generation's checkout, else the
    /// current directory walked up. Resolved on demand so checkout-free verbs never need one.
    pub fn checkout_root(&self) -> Result<PathBuf, HostError> {
        Ok(crate::environment::checkout_root(
            self.globals.checkout.as_deref(),
        )?)
    }
    /// A fresh multi-thread runtime for an async owner.
    pub fn runtime(&self) -> Result<tokio::runtime::Runtime, HostError> {
        Ok(tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?)
    }
}

/// The command tree as the given binary presents it.
pub fn command(bin: BinKind) -> clap::Command {
    let mut command = Cli::command().name(bin.name()).bin_name(bin.name());
    if bin == BinKind::Pse {
        for name in HIDDEN_FROM_PSE {
            command = command.mut_subcommand(name, |verb| verb.hide(true));
        }
    }
    command
}

fn dispatch(ctx: &Context, command: Command) -> Result<i32, HostError> {
    match command {
        Command::Boundary(args) => boundary::run(ctx, args),
        Command::AgentHook(args) => agent_hook::run(ctx, args),
        Command::Doctor(args) => doctor::run(ctx, args),
        Command::CargoTools(args) => cargo_tools::run(ctx, args),
        Command::DiagnosticTools(args) => diagnostic_tools::run(ctx, args),
        Command::TelemetryService(args) => telemetry_service::run(ctx, args),
        Command::Native(args) => native::run(ctx, args),
        Command::Server(args) => server::run(ctx, args),
        Command::StudyModule(args) => study_module::run(ctx, args),
        Command::Env(args) => env::run(ctx, args),
        Command::Activity(args) => activity::run(ctx, args),
        Command::Run(args) => run::run(ctx, args),
    }
}

/// How a failure is reported, by the verb that failed (A03 replaces this with exit classes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reporting {
    /// `pse-host: …`, exit 125.
    Host,
    /// `pse-env: …`, exit 125.
    Boundary,
    /// Hook protocol: `BLOCKED: …`, exit 2.
    Hook,
}
impl Reporting {
    fn prefix(self) -> &'static str {
        match self {
            Self::Host => "pse-host",
            Self::Boundary => "pse-env",
            Self::Hook => "BLOCKED",
        }
    }
}

/// The verb word in `arguments`, skipping global options (and the values they take) as the
/// tree declares them, without parsing.
fn verb_word(command: &clap::Command, arguments: &[OsString]) -> Option<String> {
    let valued = |matches: &dyn Fn(&clap::Arg) -> bool| {
        command
            .get_arguments()
            .filter(|arg| arg.is_global_set() && arg.get_action().takes_values())
            .any(|arg| matches(arg))
    };
    let mut words = arguments.iter().skip(1);
    while let Some(word) = words.next() {
        let text = word.to_str()?;
        if text == "--" {
            return None;
        }
        let takes_value = if let Some(long) = text.strip_prefix("--") {
            !long.contains('=') && valued(&|arg| arg.get_long() == Some(long))
        } else if let Some(short) = text.strip_prefix('-') {
            let mut flags = short.chars();
            flags.next().is_some_and(|first| {
                flags.as_str().is_empty() && valued(&|arg| arg.get_short() == Some(first))
            })
        } else {
            return Some(text.to_owned());
        };
        if takes_value {
            words.next();
        }
    }
    None
}

/// One line on stderr, then the reporting's exit code.
fn report(reporting: Reporting, message: &dyn std::fmt::Display) -> ExitCode {
    let _ = writeln!(io::stderr(), "{}: {message}", reporting.prefix());
    ExitCode::from(if reporting == Reporting::Hook { 2 } else { 125 })
}

/// Print help/version as clap renders it (exit 0); any other parse error goes to `fail`
/// as its one-line headline.
fn parse_failure(error: &clap::Error, fail: impl FnOnce(&str) -> ExitCode) -> ExitCode {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            let _ = error.print();
            ExitCode::SUCCESS
        }
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let _ = write!(io::stdout(), "{}", error.render());
            ExitCode::SUCCESS
        }
        _ => fail(&first_line(error)),
    }
}

/// The headline of a rendered clap error, without its `error: ` label.
fn first_line(error: &clap::Error) -> String {
    let rendered = error.render().to_string();
    let line = rendered.lines().next().unwrap_or_default();
    line.strip_prefix("error: ").unwrap_or(line).to_owned()
}

fn exit(code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(code).unwrap_or(125))
}

/// The entry point every bin calls.
pub fn main(bin: BinKind) -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().collect();
    // The gated child starts before parsing, runtimes or any descriptor setup.
    if arguments
        .get(1)
        .is_some_and(|word| word == foreground::TRAMPOLINE)
    {
        return match foreground::trampoline(&arguments[2..]) {
            Ok(code) => exit(code),
            Err(error) => report(Reporting::Host, &error),
        };
    }
    if bin == BinKind::CargoPseCorrectness {
        return correctness::main(&arguments);
    }
    let tree = command(bin);
    let reporting = match verb_word(&tree, &arguments).as_deref() {
        Some("boundary") => Reporting::Boundary,
        Some("agent-hook") => Reporting::Hook,
        _ => Reporting::Host,
    };
    let parsed = tree
        .try_get_matches_from(&arguments)
        .and_then(|matches| Cli::from_arg_matches(&matches));
    let cli = match parsed {
        Ok(cli) => cli,
        Err(error) => return parse_failure(&error, |line| report(reporting, &line)),
    };
    let ctx = Context::new(bin, cli.globals, std::env::vars_os().collect());
    match dispatch(&ctx, cli.command) {
        Ok(code) => exit(code),
        // A hook's own failures are already protocol output; only infrastructure fails here.
        Err(error) if reporting == Reporting::Hook => report(Reporting::Host, &error),
        Err(error) => report(reporting, &error),
    }
}
