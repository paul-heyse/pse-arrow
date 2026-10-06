// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-publication`: catalog-owned publication maintenance for one registered workspace
//! (ADR-0114 Outcome 7; Plan 22 O8).
//!
//! - `export`: write a one-row manifest an offline reader opens, protected by an export
//!   lease for a stated time; `release` ends that lease early;
//! - `retire`: mark publications expiring, wait for their readers, remove the tables only
//!   they select and mark them deleted (the head is protected);
//! - `collect`: advance the workspace's maintenance epoch, then checkpoint and vacuum
//!   every selected table keeping every protected version;
//! - `reclaim`: fence and remove the members of unpublished intents that can never commit.
//!
//! Every command is idempotent: an interrupted run is completed by rerunning it.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line maintainer reports on its own terminal"
)]

use std::num::NonZeroUsize;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
#[path = "support/deployment.rs"]
mod deployment;
use pse_columnar::CancellationToken;
use pse_operations::catalog::PublicationId;
use pse_runtime::workflow::{
    DiscoveryBudget, Durability, ExportReceipt, LeasePolicy, Operations, Runtime, ScanId,
    WorkflowError,
};
use pse_runtime::{ResourceBudget, SharedRuntime};

/// Publication catalog maintenance (ADR-0114).
#[derive(Debug, Parser)]
#[command(name = "pse-publication", version)]
struct Cli {
    /// Connection URL; defaults to `PSE_DATABASE_URL`, then the local Unix socket.
    #[arg(long)]
    url: Option<String>,
    /// Canonical database on the supervised endpoint; defaults to the profile selection.
    #[arg(long)]
    canonical_database: Option<String>,
    /// Identity recorded on leases; defaults to `pse-publication:<host>:<pid>`.
    #[arg(long)]
    name: Option<String>,
    /// Process memory budget in MiB.
    #[arg(long, default_value_t = 2048)]
    memory_mib: usize,
    /// Engine pool threads; defaults to the available parallelism.
    #[arg(long)]
    threads: Option<usize>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Export a publication for offline readers; prints the receipt as JSON.
    Export {
        /// The publication (hexadecimal identity).
        #[arg(long)]
        publication: String,
        /// A new directory URL for the manifest.
        #[arg(long)]
        destination: url::Url,
        /// How long the export lease protects the members.
        #[arg(long, default_value_t = 86_400)]
        valid_for_seconds: u64,
    },
    /// Release an export's lease early, from the receipt `export` printed.
    Release {
        /// The receipt JSON.
        #[arg(long)]
        receipt: String,
    },
    /// Retire publications of a workspace.
    Retire {
        /// The workspace name.
        #[arg(long)]
        workspace: String,
        /// The publications (hexadecimal identities).
        #[arg(long, required = true, num_args = 1..)]
        publication: Vec<String>,
        /// How long to wait for active readers.
        #[arg(long, default_value_t = 60)]
        wait_seconds: u64,
    },
    /// Collect a workspace's tables, keeping every protected version.
    Collect {
        /// The workspace name.
        #[arg(long)]
        workspace: String,
    },
    /// Reclaim a workspace's unpublished intents that can never commit.
    Reclaim {
        /// The workspace name.
        #[arg(long)]
        workspace: String,
    },
    /// Enumerate one workspace root, recording candidates without deleting them.
    Discover {
        #[arg(long)]
        workspace: String,
        /// Existing inventory identity; a new process restarts its enumeration generation.
        #[arg(long)]
        scan: Option<String>,
        #[arg(long, default_value_t = 256)]
        entries_per_slice: usize,
        #[arg(long, default_value_t = 1_048_576)]
        bytes_per_slice: usize,
    },
    /// Read one bounded page of recorded candidates, including unresolved ownership.
    Orphans {
        #[arg(long)]
        scan: String,
        #[arg(long)]
        after_prefix: Option<String>,
        #[arg(long, default_value_t = 256)]
        limit: usize,
    },
    /// Recheck and remove exactly one explicitly selected attributable candidate.
    ReclaimOrphan {
        #[arg(long)]
        workspace: String,
        #[arg(long)]
        scan: String,
        #[arg(long)]
        prefix: String,
    },
}

async fn runtime(cli: &Cli) -> Result<Runtime, String> {
    let count = |n: usize| NonZeroUsize::new(n.max(1)).unwrap_or(NonZeroUsize::MIN);
    let threads = cli
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, NonZeroUsize::get));
    let memory = cli.memory_mib.saturating_mul(1 << 20);
    let shared = SharedRuntime::build(ResourceBudget {
        memory_limit_bytes: count(memory),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: count(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: count(threads),
            target_partitions: count(threads),
        },
        execution: Default::default(),
        cache: pse_runtime::DeltaCacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            workspace_bytes: memory / 16,
            worker_bytes: memory / 32,
            artifact_bytes: memory / 16,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })
    .map_err(|e| format!("runtime budget: {e}"))?;
    let registry = pse_schema::shared_registry().map_err(|e| format!("registry: {e}"))?;
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .map_err(|e| format!("session factory: {e}"))?,
    );
    Ok(Runtime::from_shared(
        shared,
        registry,
        sessions,
        deployment::open(cli.canonical_database.as_deref()).await?,
    ))
}

fn identity(text: &str) -> Result<PublicationId, WorkflowError> {
    pse_ids::SemanticId::parse_hex(text)
        .map(PublicationId::from)
        .map_err(|error| WorkflowError::Input(format!("publication {text}: {error}")))
}

fn scan_identity(text: &str) -> Result<ScanId, WorkflowError> {
    pse_ids::SemanticId::parse_hex(text)
        .map(ScanId::from)
        .map_err(|error| WorkflowError::Input(format!("scan {text}: {error}")))
}

fn report(error: &WorkflowError) {
    let code = miette::Diagnostic::code(error)
        .map(|code| format!(" [{code}]"))
        .unwrap_or_default();
    eprintln!("error{code}: {error}");
    if let Some(help) = miette::Diagnostic::help(error) {
        eprintln!("help: {help}");
    }
}

async fn run(cli: Cli, runtime: Runtime) -> Result<(), WorkflowError> {
    let url = cli
        .url
        .clone()
        .unwrap_or_else(pse_operations::database_url_from_env);
    let name = cli
        .name
        .clone()
        .unwrap_or_else(|| Operations::process_worker("pse-publication"));
    let operations = Operations::connect(&url, name, LeasePolicy::default()).await?;
    let runtime = runtime.with_durability(Durability::Durable(operations));
    let cancel = CancellationToken::new();
    let interrupt = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            interrupt.cancel();
        }
    });
    match cli.command {
        Command::Export {
            publication,
            destination,
            valid_for_seconds,
        } => {
            let receipt = runtime
                .export_publication(
                    identity(&publication)?,
                    destination,
                    Duration::from_secs(valid_for_seconds),
                    &cancel,
                )
                .await?;
            let json = serde_json::to_string(&receipt)
                .map_err(|error| WorkflowError::Input(error.to_string()))?;
            println!("{json}");
        }
        Command::Release { receipt } => {
            let receipt: ExportReceipt = serde_json::from_str(&receipt)
                .map_err(|error| WorkflowError::Input(format!("receipt: {error}")))?;
            let held = runtime.release_export(&receipt).await?;
            println!(
                "export lease {} {}",
                receipt.lease_id,
                if held {
                    "released"
                } else {
                    "was no longer held"
                }
            );
        }
        Command::Retire {
            workspace,
            publication,
            wait_seconds,
        } => {
            let workspace = runtime.workspace(&workspace).await?;
            let publications = publication
                .iter()
                .map(|text| identity(text))
                .collect::<Result<Vec<_>, _>>()?;
            let retired = runtime
                .retire_publications(
                    workspace.workspace_id,
                    &publications,
                    Duration::from_secs(wait_seconds),
                    &cancel,
                )
                .await?;
            for publication in &retired.retired {
                println!("retired {publication}");
            }
            for table in &retired.removed_tables {
                println!("removed {table}");
            }
            println!("removed {} object(s)", retired.removed_objects);
        }
        Command::Collect { workspace } => {
            let workspace = runtime.workspace(&workspace).await?;
            let collected = runtime.collect(workspace.workspace_id, &cancel).await?;
            println!("maintenance epoch {}", collected.maintenance_epoch);
            for table in &collected.tables {
                println!(
                    "{} at version {}: {} file(s), {} log(s) removed",
                    table.table_uri,
                    table.delta_version,
                    table.deleted_files.len(),
                    table.deleted_logs
                );
            }
        }
        Command::Reclaim { workspace } => {
            let workspace = runtime.workspace(&workspace).await?;
            let reclaimed = runtime
                .reclaim_unpublished(workspace.workspace_id, &cancel)
                .await?;
            for publication in &reclaimed.reclaimed {
                println!("reclaimed {publication}");
            }
            println!("removed {} object(s)", reclaimed.removed_objects);
        }
        Command::Discover {
            workspace,
            scan,
            entries_per_slice,
            bytes_per_slice,
        } => {
            let workspace = runtime.workspace(&workspace).await?;
            let scan = scan
                .as_deref()
                .map(scan_identity)
                .transpose()?
                .unwrap_or_else(pse_operations::mint_id);
            println!("scan {scan}");
            loop {
                let checkpoint = runtime
                    .discover_orphans(
                        workspace.workspace_id,
                        scan,
                        DiscoveryBudget {
                            maximum_entries: entries_per_slice,
                            maximum_bytes: bytes_per_slice,
                        },
                        &cancel,
                    )
                    .await?;
                println!(
                    "{}",
                    serde_json::json!({"scan_id": checkpoint.scan_id.to_string(),
                    "generation": checkpoint.generation, "maintenance_epoch": checkpoint.maintenance_epoch,
                    "listed_count": checkpoint.listed_count, "complete": checkpoint.complete})
                );
                if checkpoint.complete {
                    break;
                }
            }
        }
        Command::Orphans {
            scan,
            after_prefix,
            limit,
        } => {
            let page = runtime
                .orphan_candidates(scan_identity(&scan)?, after_prefix.as_deref(), limit)
                .await?;
            let candidates = page.candidates.into_iter().map(|candidate| serde_json::json!({
                "prefix": candidate.prefix, "generation": candidate.generation,
                "discovery_epoch": candidate.discovery_epoch, "ownership": candidate.ownership.as_str(),
                "evidence": candidate.evidence, "protections": candidate.protections,
                "disposition": candidate.disposition.as_str(),
            })).collect::<Vec<_>>();
            println!(
                "{}",
                serde_json::json!({"candidates": candidates, "next_prefix": page.next_prefix})
            );
        }
        Command::ReclaimOrphan {
            workspace,
            scan,
            prefix,
        } => {
            let workspace = runtime.workspace(&workspace).await?;
            let report = runtime
                .reclaim_orphan(
                    workspace.workspace_id,
                    scan_identity(&scan)?,
                    &prefix,
                    &cancel,
                )
                .await?;
            println!(
                "{}",
                serde_json::json!({"scan_id": report.scan_id.to_string(), "prefix": report.prefix,
                "maintenance_epoch": report.maintenance_epoch, "removed_objects": report.removed_objects})
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let executor = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(executor) => executor,
        Err(error) => {
            eprintln!("error: executor: {error}");
            return ExitCode::FAILURE;
        }
    };
    let runtime = match executor.block_on(runtime(&cli)) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match executor.block_on(run(cli, runtime)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            ExitCode::FAILURE
        }
    }
}
