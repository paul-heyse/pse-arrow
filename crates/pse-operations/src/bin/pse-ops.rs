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
use pse_operations::{OperationsError, SchemaStatus, Store, database_url_from_env};

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
    /// Apply declared preservation-first transitions after draining workers and closing generations.
    Migrate,
    /// Inspect exact source/target, histories and pending checksums without mutation.
    MigrationPlan,
    /// Explicitly create an absent schema.
    Create,
    /// Explicitly retire the catalog generation through a completed external manifest,
    /// then reset execution state and import preserved retirement in one transaction.
    Reset {
        /// Completed retirement inventory destination outside every workspace/member root.
        #[arg(long)]
        inventory_destination: std::path::PathBuf,
        /// Complete inventory bound; larger inventories refuse rather than truncate.
        #[arg(long, default_value_t = 1000000)]
        max_rows: u64,
    },
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
            let plan = store.migration_plan().await?;
            println!("schema:     current; verified ready: {}", plan.ready);
            plan.ready
        }
        SchemaStatus::Absent => {
            println!("schema:     absent; explicitly run `just db-create`");
            false
        }
        SchemaStatus::Mismatch { recorded } => {
            println!(
                "schema:     MISMATCH, recorded {}",
                recorded.as_deref().unwrap_or("<no fingerprint>")
            );
            println!(
                "quiesce workers and close generations, then run `just db-migrate`; unknown sources refuse without reset"
            );
            false
        }
    };
    Ok(server.is_supported() && current)
}

async fn run(command: &Command, url: &str) -> Result<bool, OperationsError> {
    let store = Store::connect(url).await?;
    match command {
        Command::Status => status(&store).await,
        Command::MigrationPlan => {
            println!("{:#?}", store.migration_plan().await?);
            Ok(true)
        }
        Command::Create => {
            println!("{}: {:?}", store.target(), store.create().await?);
            Ok(true)
        }
        Command::Migrate => {
            let plan = store.migration_plan().await?;
            let report = store.migrate(&plan).await?;
            println!("applied: {:?}", report.applied);
            println!(
                "{}: schema transition verified at {}",
                store.target(),
                Store::expected_schema()
            );
            Ok(true)
        }
        Command::Reset {
            inventory_destination,
            max_rows,
        } => {
            let receipt = if inventory_destination.exists() {
                Store::inspect_reset_manifest(inventory_destination, *max_rows)?
            } else {
                store
                    .export_reset_inventory(
                        pse_operations::mint_id(),
                        inventory_destination,
                        *max_rows,
                    )
                    .await?
            };
            let report = store.reset(&receipt).await?;
            println!(
                "{}: reset {} manifest {} (already applied: {})",
                store.target(),
                report.reset_id,
                report.manifest_digest,
                report.already_applied
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
