// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-ops`: inspect the operational store, or reset its schema.
//!
//! The `just db-status` and `just db-reset` recipes run it. The connection comes from
//! `--url`, else `PSE_DATABASE_URL`, else the local-socket development default.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line tool reports on its own terminal"
)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use pse_operations::{Opened, OperationsError, SchemaStatus, Store, database_url_from_env};

/// Operational store administration (ADR-0114).
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
    /// Report the server version, the connection target and the schema fingerprint; exits
    /// 1 when the server is unsupported or the schema records another fingerprint.
    Status,
    /// Drop the `pse_ops` schema with everything in it and create it from this build's
    /// generated DDL. Destructive: the store holds regenerable data only.
    Reset,
}

async fn status(store: &Store) -> Result<bool, OperationsError> {
    let server = store.server().await?;
    let schema = store.schema_status().await?;
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
    println!("expected:   {}", Store::expected_schema());
    let current = match &schema {
        SchemaStatus::Current => {
            println!("schema:     current");
            true
        }
        SchemaStatus::Absent => {
            println!("schema:     absent; the first durable open creates it (or run `just db-reset`)");
            true
        }
        SchemaStatus::Mismatch { recorded } => {
            println!(
                "schema:     MISMATCH, recorded {}",
                recorded.as_deref().unwrap_or("<no fingerprint>")
            );
            println!("run `just db-reset` to drop and recreate pse_ops (its data is regenerable)");
            false
        }
    };
    Ok(server.is_supported() && current)
}

async fn run(command: &Command, url: &str) -> Result<bool, OperationsError> {
    let store = Store::connect(url).await?;
    match command {
        Command::Status => status(&store).await,
        Command::Reset => {
            let opened = store.reset().await?;
            println!(
                "{}: pse_ops {} with schema {}",
                store.target(),
                match opened {
                    Opened::Created => "recreated",
                    Opened::Current => "already current",
                },
                Store::expected_schema()
            );
            Ok(true)
        }
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
