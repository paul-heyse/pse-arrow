// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-worker`: claim durable jobs from the operational store and run them (ADR-0114
//! Outcome 18; Plan 22 O4).
//!
//! The worker is a composition root: one shared runtime and `MathService`, a durable
//! runtime class bound to the store, and a claim loop. Each claimed try runs under the
//! worker's lease; the durable cancellation flag stops it; its end goes through the job's
//! retry policy. On start-up the worker sweeps expired leases (their jobs requeue as new
//! attempts), and again periodically.
//!
//! The process-level OpenMP environment that SPRAL needs (ADR-0108) is set before any
//! thread starts: when it is missing, the worker re-executes itself with it.

// Match the library's depth for the shared runtime's nested async `Send` proof.
#![recursion_limit = "256"]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line worker reports on its own terminal"
)]

use std::num::NonZeroUsize;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

#[path = "support/deployment.rs"]
mod deployment;

use clap::Parser;
use pse_runtime::workflow::{
    Durability, LeasePolicy, Operations, Processed, Runtime, WorkerSettings, WorkflowError,
};
use pse_runtime::{CancelSource, ResourceBudget, SharedRuntime};

/// Durable job worker (ADR-0114).
#[derive(Debug, Parser)]
#[command(name = "pse-worker", version)]
struct Cli {
    /// Connection URL; defaults to `PSE_DATABASE_URL`, then the local Unix socket.
    #[arg(long)]
    url: Option<String>,
    /// Canonical database on the supervised endpoint; defaults to the profile selection.
    #[arg(long)]
    canonical_database: Option<String>,
    /// Worker identity recorded on leases; defaults to `pse-worker:<host>:<pid>`.
    #[arg(long)]
    name: Option<String>,
    /// Lease length in seconds.
    #[arg(long, default_value_t = 30)]
    lease_seconds: u64,
    /// Heartbeat period in milliseconds.
    #[arg(long, default_value_t = 10_000)]
    heartbeat_ms: u64,
    /// Idle poll period in milliseconds.
    #[arg(long, default_value_t = 500)]
    poll_ms: u64,
    /// Stop after this many jobs.
    #[arg(long)]
    jobs: Option<usize>,
    /// Stop as soon as no job is available.
    #[arg(long)]
    until_idle: bool,
    /// Process memory budget in MiB.
    #[arg(long)]
    memory_mib: Option<usize>,
    /// Engine pool threads; defaults to the available parallelism.
    #[arg(long)]
    threads: Option<usize>,
}

/// The OpenMP and hwloc settings SPRAL needs in the solver image (ADR-0108, execution
/// decision 9): set only when absent, so an operator's explicit choice stands.
const OPENMP: [(&str, &str); 4] = [
    ("OMP_CANCELLATION", "TRUE"),
    ("OMP_PROC_BIND", "TRUE"),
    ("OMP_PLACES", "sockets"),
    (
        "HWLOC_COMPONENTS",
        "-linuxio,-pci,-opencl,-cuda,-nvml,-rsmi,-levelzero,-gl",
    ),
];

/// Re-execute with the OpenMP environment when any of it is missing. Returns only when the
/// environment is already complete (or re-execution failed, which is reported).
fn ensure_openmp_environment() -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    let missing: Vec<_> = OPENMP
        .iter()
        .filter(|(name, _)| std::env::var_os(name).is_none())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| format!("locate pse-worker: {e}"))?;
    let error = std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .envs(missing.iter().map(|(name, value)| (*name, *value)))
        .exec();
    Err(format!("re-execute with the OpenMP environment: {error}"))
}

fn resource_budget(cli: &Cli, configured: usize) -> Result<ResourceBudget, String> {
    let memory = cli
        .memory_mib
        .map(|mib| {
            mib.checked_mul(1 << 20)
                .ok_or("worker memory extent overflow")
        })
        .transpose()?
        .unwrap_or(configured);
    if memory == 0 || memory > configured {
        return Err("worker memory exceeds the supervised native slot".into());
    }
    let count = |n: usize| NonZeroUsize::new(n.max(1)).unwrap_or(NonZeroUsize::MIN);
    let threads = cli.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map_or(1, NonZeroUsize::get)
            .min(4)
    });
    let math = pse_runtime::math::MathPolicy {
        workspace_bytes: memory / 8,
        worker_bytes: memory / 2,
        artifact_bytes: memory / 8,
        ..Default::default()
    };
    Ok(ResourceBudget {
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
        math,
        hashing_may_use_pool: false,
    })
}

async fn runtime(cli: &Cli) -> Result<Runtime, String> {
    let state = std::env::var_os("PSE_SURREAL_STATE")
        .ok_or("PSE_SURREAL_STATE must select a supervised canonical deployment")?;
    let options =
        pse_operations::canonical::CanonicalOptions::from_state(std::path::Path::new(&state))
            .map_err(|e| e.to_string())?;
    let shared = SharedRuntime::build(resource_budget(
        cli,
        options.native.native_worker_memory_bytes,
    )?)
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

fn report(error: &WorkflowError) {
    let code = miette::Diagnostic::code(error)
        .map(|code| format!(" [{code}]"))
        .unwrap_or_default();
    eprintln!("error{code}: {error}");
    if let Some(help) = miette::Diagnostic::help(error) {
        eprintln!("help: {help}");
    }
}

async fn serve(cli: &Cli, runtime: Runtime) -> Result<Vec<Processed>, WorkflowError> {
    let url = cli
        .url
        .clone()
        .unwrap_or_else(pse_operations::database_url_from_env);
    let name = cli
        .name
        .clone()
        .unwrap_or_else(|| Operations::process_worker("pse-worker"));
    let policy = LeasePolicy {
        lease: Duration::from_secs(cli.lease_seconds),
        heartbeat: Duration::from_millis(cli.heartbeat_ms),
        ..LeasePolicy::default()
    };
    let operations = Operations::connect(&url, name, policy).await?;
    let runtime = runtime.with_durability(Durability::Durable(operations));
    let settings = WorkerSettings {
        poll: Duration::from_millis(cli.poll_ms),
        jobs: cli.jobs,
        until_idle: cli.until_idle,
        ..WorkerSettings::default()
    };
    let stop = CancelSource::new();
    let interrupt = stop.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            interrupt.cancel();
        }
    });
    runtime.serve(settings, &stop).await
}

fn main() -> ExitCode {
    if let Err(error) = ensure_openmp_environment() {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    let cli = Cli::parse();
    let executor = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
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
    match executor.block_on(serve(&cli, runtime)) {
        Ok(processed) => {
            for ran in &processed {
                if let Processed::Ran { job, record } = ran {
                    let state = record
                        .attempt
                        .as_ref()
                        .map_or("unrecorded", |a| a.state.as_str());
                    println!("job {job}: attempt {} {state}", record.attempt_id);
                }
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            report(&error);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "assert the CLI parser and its production budget contract"
)]
mod tests {
    use super::*;

    #[test]
    fn parsed_default_budget_admits_default_compiler_scratch() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let budget = resource_budget(&cli, 1 << 30).unwrap();
        let compiler = pse_runtime::workflow::PreparationSettings::default().compiler;
        assert!(budget.math.worker_bytes >= compiler.evaluation.scratch_bytes);
        let compilation = compiler.evaluation.scratch_bytes
            + budget.math.stack_bytes
            + budget.math.foreign_bytes
            + budget.math.inner_session_bytes;
        assert!(compilation + budget.math.workspace_bytes <= budget.memory_limit_bytes.get());
    }

    #[test]
    fn explicit_smaller_budget_retains_its_compilation_admission_limit() {
        let cli = Cli::try_parse_from(["pse-worker", "--memory-mib", "512"]).unwrap();
        let budget = resource_budget(&cli, 1 << 30).unwrap();
        assert!(
            budget.math.worker_bytes
                < pse_runtime::workflow::PreparationSettings::default()
                    .compiler
                    .evaluation
                    .scratch_bytes
        );
    }

    #[test]
    fn explicit_worker_budget_cannot_exceed_the_supervised_slot() {
        let cli = Cli::try_parse_from(["pse-worker", "--memory-mib", "2048"]).unwrap();
        assert!(resource_budget(&cli, 1 << 30).is_err());
    }
}
