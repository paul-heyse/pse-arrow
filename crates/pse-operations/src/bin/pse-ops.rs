// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-ops`: apply and inspect the embedded operational-store migrations.
//!
//! The `just db-migrate` and `just db-status` recipes run it. The connection comes from
//! `--url`, else `PSE_DATABASE_URL`, else the local-socket development default.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line tool reports on its own terminal"
)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use pse_operations::{OperationsError, Store, database_url_from_env};

/// Operational store administration (ADR-0112).
#[derive(Debug, Parser)]
#[command(name = "pse-ops", version)]
struct Cli {
    /// Connection URL; defaults to `PSE_DATABASE_URL`, then the local Unix socket.
    #[arg(long, global = true)]
    url: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Apply every pending embedded migration.
    Migrate,
    /// Report the server version, the connection target and pending migrations; exits 1
    /// unless the server is supported and the schema is current.
    Status,
}

async fn status(store: &Store) -> Result<bool, OperationsError> {
    let server = store.server().await?;
    let migrations = store.migration_status().await?;
    println!("target:     {}", store.target());
    println!(
        "server:     PostgreSQL {} ({})",
        server.version,
        if server.is_supported() {
            "supported"
        } else {
            "unsupported; 18 or newer is required"
        }
    );
    println!(
        "migrations: {} applied, {} pending",
        migrations.applied.len(),
        migrations.pending.len()
    );
    for (version, description) in &migrations.pending {
        println!("  pending    {version} {description}");
    }
    for version in &migrations.mismatched {
        println!("  EDITED     {version}: the applied checksum differs from the embedded one");
    }
    for version in &migrations.unknown {
        println!("  UNKNOWN    {version}: applied, but not embedded in this build");
    }
    if let Some(version) = migrations.dirty {
        println!("  DIRTY      {version}: a migration failed part way");
    }
    if !migrations.pending.is_empty() {
        println!("run `just db-migrate` to apply pending migrations");
    }
    Ok(server.is_supported() && migrations.is_current())
}

async fn run(command: &Command, url: &str) -> Result<bool, OperationsError> {
    let store = Store::connect(url).await?;
    match command {
        Command::Migrate => {
            store.migrate().await?;
            let migrations = store.migration_status().await?;
            println!(
                "{}: {} migration(s) applied; schema current: {}",
                store.target(),
                migrations.applied.len(),
                migrations.is_current()
            );
            Ok(migrations.is_current())
        }
        Command::Status => status(&store).await,
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let url = cli.url.unwrap_or_else(database_url_from_env);
    match run(&cli.command, &url).await {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            let code = miette::Diagnostic::code(&error)
                .map(|code| format!(" [{code}]"))
                .unwrap_or_default();
            eprintln!("error{code}: {error}");
            if let Some(help) = miette::Diagnostic::help(&error) {
                eprintln!("help: {help}");
            }
            ExitCode::FAILURE
        }
    }
}
