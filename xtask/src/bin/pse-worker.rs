// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-worker`: serve bounded canonical study actions on the supervised deployment.
//! Claims, cancellation and recovery use native fences. Scientific work stays outside
//! transactions, and completed native reports are retained as compact result selections.

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
    Durability, LeasePolicy, Operations, Runtime, WorkerSettings, WorkflowError,
};
use pse_runtime::{CancelSource, ResourceBudget, SharedRuntime};

/// Managed canonical study worker.
#[derive(Debug, Parser)]
#[command(name = "pse-worker", version)]
struct Cli {
    /// Canonical database on the supervised endpoint; defaults to the profile selection.
    #[arg(long)]
    canonical_database: Option<String>,
    /// Private bounded native-entry controller, only in a qualification build.
    #[cfg(feature = "canonical-tests")]
    #[arg(long)]
    qualification_native_entry: Option<std::path::PathBuf>,
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
    /// Stop after this many processed scheduling actions.
    #[arg(long)]
    maximum_actions: Option<usize>,
    /// Concurrent case lanes on this process's shared allocation.
    #[arg(long)]
    maximum_in_flight: Option<usize>,
    /// Stop as soon as a complete bounded discovery pass finds no work.
    #[arg(long)]
    until_idle: bool,
    /// Engine pool budget in MiB; a selected execution profile fixes it exactly.
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

fn resource_budget(
    cli: &Cli,
    allocation: pse_operations::canonical::NativeAllocation,
) -> Result<ResourceBudget, String> {
    allocation.validate().map_err(|error| error.to_string())?;
    let configured = allocation.native_worker_memory_bytes;
    if let Some(execution) = allocation.execution {
        if cli
            .memory_mib
            .is_some_and(|value| value.checked_mul(1 << 20) != Some(execution.pool_memory_bytes))
            || cli
                .threads
                .is_some_and(|value| value != execution.cpu_threads)
            || cli
                .maximum_in_flight
                .is_some_and(|value| value == 0 || value > execution.case_lanes)
        {
            return Err("worker overrides differ from the supervised execution profile".into());
        }
        let count = |value| NonZeroUsize::new(value).ok_or("zero execution allocation");
        return Ok(ResourceBudget {
            memory_limit_bytes: count(execution.pool_memory_bytes)?,
            spill_dir: std::env::temp_dir(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: count(5)?,
            threads: pse_engine::ThreadBudget {
                pool_threads: count(execution.cpu_threads)?,
                target_partitions: count(execution.cpu_threads)?,
            },
            execution: Default::default(),
            cache: pse_runtime::CacheBudget::disabled(1024),
            math: pse_runtime::math::MathPolicy {
                worker_bytes: execution.worker_bytes,
                jobs: execution.math_jobs,
                admission_wait: Duration::from_millis(execution.admission_wait_ms),
                ..Default::default()
            },
            hashing_may_use_pool: false,
        });
    }
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
        cache: pse_runtime::CacheBudget::disabled(1024),
        math,
        hashing_may_use_pool: false,
    })
}

fn verify_placement(
    allocation: &pse_operations::canonical::NativeAllocation,
) -> Result<(), String> {
    let Some(execution) = allocation.execution else {
        return Ok(());
    };
    let cgroups =
        std::fs::read_to_string("/proc/self/cgroup").map_err(|error| error.to_string())?;
    let relative = cgroups
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .ok_or("unified managed worker cgroup required")?;
    let root = std::path::Path::new("/sys/fs/cgroup");
    let own = root.join(relative.trim_start_matches('/'));
    let cap = std::fs::read_to_string(own.join("memory.max")).map_err(|error| error.to_string())?;
    if cap.trim().parse::<usize>().ok() != Some(allocation.native_worker_memory_bytes) {
        return Err("managed worker process cap differs from its profile".into());
    }
    let mut group = own.as_path();
    while group != root {
        let memory =
            std::fs::read_to_string(group.join("memory.max")).map_err(|error| error.to_string())?;
        if memory.trim() != "max"
            && memory
                .trim()
                .parse::<usize>()
                .map_err(|error| error.to_string())?
                < allocation.native_worker_memory_bytes
        {
            return Err("ancestor memory cap cannot admit the exact worker profile".into());
        }
        let cpu =
            std::fs::read_to_string(group.join("cpu.max")).map_err(|error| error.to_string())?;
        let fields = cpu.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 2 {
            return Err("invalid managed CPU quota".into());
        }
        if fields[0] != "max" {
            let quota = fields[0]
                .parse::<u64>()
                .map_err(|error| error.to_string())?;
            let period = fields[1]
                .parse::<u64>()
                .map_err(|error| error.to_string())?;
            if period == 0 || quota / period < execution.cpu_threads as u64 {
                return Err("ancestor CPU quota cannot admit sixteen case lanes".into());
            }
        }
        group = group
            .parent()
            .ok_or("worker placement has no cgroup ancestor")?;
    }
    if std::thread::available_parallelism()
        .map_err(|error| error.to_string())?
        .get()
        < execution.cpu_threads
    {
        return Err("worker affinity cannot admit sixteen case lanes".into());
    }
    Ok(())
}

fn producer_receipt_observation(
    path: Option<&std::ffi::OsStr>,
) -> Result<(Option<String>, Option<String>), String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let Some(path) = path else {
        return Ok((None, None));
    };
    let path = std::path::Path::new(path);
    let display = path
        .to_str()
        .ok_or("worker receipt path is not UTF-8")?
        .to_owned();
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let bytes = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        hash.update(&buffer[..bytes]);
    }
    let digest = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok((Some(display), Some(digest)))
}

fn publish_ready(cli: &Cli) -> Result<(), String> {
    let Some(nonce) = std::env::var_os("PSE_PRIMARY_NONCE") else {
        return Ok(());
    };
    let state = std::env::var_os("PSE_SURREAL_STATE").ok_or("managed primary state missing")?;
    let state = std::path::Path::new(&state);
    let options = pse_operations::canonical::CanonicalOptions::from_state(state)
        .map_err(|error| error.to_string())?;
    let execution = options
        .native
        .execution
        .ok_or("managed primary execution profile missing")?;
    if cli.maximum_in_flight != Some(execution.case_lanes)
        || cli.until_idle
        || cli.maximum_actions.is_some()
    {
        return Err("managed primary requires its declared sustained case lanes".into());
    }
    // runtime() has completed the deployment owner's receipt/header/artifact admission.
    // This streamed observation associates that receiving input without another validator.
    let receipt = std::env::var_os("PSE_PRODUCER_RECEIPT");
    let (receipt_path, receipt_sha256) = producer_receipt_observation(receipt.as_deref())?;
    let value = serde_json::json!({
        "producer_receipt_path": receipt_path, "producer_receipt_sha256": receipt_sha256,
        "ready": true, "pid": std::process::id(), "nonce": nonce.to_string_lossy(),
        "canonical_database": cli.canonical_database.as_deref().unwrap_or(&options.database),
        "pool_memory_bytes": execution.pool_memory_bytes, "worker_bytes": execution.worker_bytes,
        "cpu_threads": execution.cpu_threads, "case_lanes": execution.case_lanes, "math_jobs": execution.math_jobs,
    });
    #[cfg(feature = "canonical-tests")]
    let value = {
        let mut value = value;
        value["qualification_native_entry"] = serde_json::json!(cli.qualification_native_entry);
        value
    };
    let temporary = state.join(format!(".primary-ready-{}.json", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    use std::io::Write;
    file.write_all(&serde_json::to_vec(&value).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, state.join("primary-receiver.json"))
        .map_err(|error| error.to_string())?;
    std::fs::File::open(state)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}

async fn runtime(cli: &Cli) -> Result<Runtime, String> {
    let state = std::env::var_os("PSE_SURREAL_STATE")
        .ok_or("PSE_SURREAL_STATE must select a supervised canonical deployment")?;
    let options =
        pse_operations::canonical::CanonicalOptions::from_state(std::path::Path::new(&state))
            .map_err(|e| e.to_string())?;
    verify_placement(&options.native)?;
    let shared = SharedRuntime::build(resource_budget(cli, options.native)?)
        .map_err(|e| format!("runtime budget: {e}"))?;
    let registry = pse_schema::shared_registry().map_err(|e| format!("registry: {e}"))?;
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .map_err(|e| format!("session factory: {e}"))?,
    );
    let deployment = deployment::open(cli.canonical_database.as_deref()).await?;
    let policy = LeasePolicy {
        lease: Duration::from_secs(cli.lease_seconds),
        heartbeat: Duration::from_millis(cli.heartbeat_ms),
        ..LeasePolicy::default()
    };
    if policy.lease.is_zero() || policy.heartbeat.is_zero() || policy.heartbeat >= policy.lease {
        return Err("worker heartbeat must be positive and shorter than its lease".into());
    }
    let operations = Operations::from_store(
        deployment.store().clone(),
        cli.name
            .clone()
            .unwrap_or_else(|| Operations::process_worker("pse-worker")),
        policy,
        shared.pool(),
    );
    Ok(Runtime::from_shared(shared, registry, sessions, deployment)
        .with_durability(Durability::Durable(operations)))
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

async fn serve(
    cli: &Cli,
    runtime: Runtime,
    #[cfg(feature = "canonical-tests")] qualification: Option<Arc<qualification::Controller>>,
) -> Result<usize, WorkflowError> {
    let settings = WorkerSettings {
        poll: Duration::from_millis(cli.poll_ms),
        maximum_actions: cli.maximum_actions,
        maximum_in_flight: cli.maximum_in_flight,
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
    #[cfg(feature = "canonical-tests")]
    if let Some(qualification) = qualification {
        let work = runtime.serve(settings, &stop);
        tokio::pin!(work);
        return tokio::select! {
            result = &mut work => result,
            result = qualification.wait_stop() => {
                stop.cancel();
                let drained = work.await;
                match result {
                    Ok(()) => drained,
                    Err(error) => Err(WorkflowError::Internal(format!("native qualification control: {error}"))),
                }
            }
        };
    }
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
    #[cfg(feature = "canonical-tests")]
    let qualification = match cli
        .qualification_native_entry
        .as_deref()
        .map(|directory| qualification::Controller::open(directory, &cli))
        .transpose()
    {
        Ok(qualification) => qualification,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    #[cfg(feature = "canonical-tests")]
    if let Some(controller) = &qualification {
        let observer = controller.clone();
        if let Err(error) =
            pse_runtime::math::install_qualified_native_entry(Arc::new(move |flag| {
                observer.enter(flag)
            }))
        {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    }
    if let Err(error) = publish_ready(&cli) {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    #[cfg(feature = "canonical-tests")]
    let result = executor.block_on(serve(&cli, runtime, qualification));
    #[cfg(not(feature = "canonical-tests"))]
    let result = executor.block_on(serve(&cli, runtime));
    match result {
        Ok(processed) => {
            println!("processed {processed} canonical scheduling actions");
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
    fn legacy_allocation(bytes: usize) -> pse_operations::canonical::NativeAllocation {
        pse_operations::canonical::NativeAllocation {
            native_workers: 2,
            native_worker_memory_bytes: bytes,
            execution: None,
        }
    }
    #[test]
    fn primary_receipt_observation_streams_actual_bytes_and_distinguishes_absence() {
        assert_eq!(producer_receipt_observation(None).unwrap(), (None, None));
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("worker.json");
        std::fs::write(&path, b"abc").unwrap();
        let observed = producer_receipt_observation(Some(path.as_os_str())).unwrap();
        assert_eq!(observed.0.as_deref(), path.to_str());
        assert_eq!(
            observed.1.as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
        std::fs::write(&path, vec![b'a'; 65537]).unwrap();
        assert_ne!(
            producer_receipt_observation(Some(path.as_os_str())).unwrap(),
            observed
        );
        std::fs::remove_file(&path).unwrap();
        assert!(producer_receipt_observation(Some(path.as_os_str())).is_err());
    }

    #[test]
    fn selected_reference_budget_consumes_exact_allocated_fields() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let execution = pse_operations::canonical::NativeExecutionAllocation {
            pool_memory_bytes: 128 << 30,
            worker_bytes: 16 << 30,
            cpu_threads: 16,
            case_lanes: 16,
            math_jobs: 32,
            compiler_cores: 1,
            admission_wait_ms: 30_000,
            process_headroom_bytes: 12 << 30,
            observer_memory_bytes: 4 << 30,
        };
        let allocation = pse_operations::canonical::NativeAllocation {
            native_workers: 1,
            native_worker_memory_bytes: 140 << 30,
            execution: Some(execution),
        };
        let budget = resource_budget(&cli, allocation).unwrap();
        assert_eq!(budget.memory_limit_bytes.get(), 128 << 30);
        assert_eq!(budget.math.worker_bytes, 16 << 30);
        assert_eq!(budget.math.jobs, 32);
        assert_eq!(budget.threads.pool_threads.get(), 16);
        assert_eq!(
            budget.math.workspace_bytes,
            pse_runtime::math::MathPolicy::default().workspace_bytes
        );
        let cli = Cli::try_parse_from(["pse-worker", "--threads", "8"]).unwrap();
        assert!(resource_budget(&cli, allocation).is_err());
    }

    #[test]
    fn parsed_default_budget_admits_default_compiler_scratch() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let budget = resource_budget(&cli, legacy_allocation(1 << 30)).unwrap();
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
        let budget = resource_budget(&cli, legacy_allocation(1 << 30)).unwrap();
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
        assert!(resource_budget(&cli, legacy_allocation(1 << 30)).is_err());
    }
}

#[cfg(feature = "canonical-tests")]
mod qualification {
    use std::{
        collections::BTreeSet,
        path::{Path, PathBuf},
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        time::{Duration, Instant},
    };

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Request {
        nonce: String,
        canonical_database: String,
        entries: usize,
        timeout_ms: u64,
    }
    #[derive(Default)]
    struct Entries {
        owners: BTreeSet<String>,
        observations: Vec<serde_json::Value>,
        active: usize,
        maximum: usize,
    }
    pub(super) struct Controller {
        directory: PathBuf,
        request: Request,
        deadline: Instant,
        entries: Mutex<Entries>,
    }
    fn private_metadata(path: &Path, directory: bool) -> Result<std::fs::Metadata, String> {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        let owner = std::fs::metadata("/proc/self")
            .map_err(|error| error.to_string())?
            .uid();
        if metadata.file_type().is_symlink()
            || metadata.uid() != owner
            || metadata.mode() & 0o077 != 0
            || if directory {
                !metadata.is_dir()
            } else {
                !metadata.is_file() || metadata.len() > 8192
            }
        {
            return Err(
                "qualification control must be private, bounded and owned by the receiver user"
                    .into(),
            );
        }
        Ok(metadata)
    }
    fn bounded_json(path: &Path) -> Result<Vec<u8>, String> {
        private_metadata(path, false)?;
        std::fs::read(path).map_err(|error| error.to_string())
    }
    fn write_private(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        if bytes.len() > 8192 {
            return Err("qualification observation extent".into());
        }
        let temporary = path.with_extension(format!(
            "{}-{:?}-temporary",
            std::process::id(),
            std::thread::current().id()
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| error.to_string())?;
        let result = (|| {
            file.write_all(&bytes).map_err(|error| error.to_string())?;
            file.sync_all().map_err(|error| error.to_string())?;
            std::fs::rename(&temporary, path).map_err(|error| error.to_string())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }
    impl Controller {
        pub(super) fn open(directory: &Path, cli: &super::Cli) -> Result<Arc<Self>, String> {
            let state =
                std::env::var_os("PSE_SURREAL_STATE").ok_or("qualification state missing")?;
            let state = std::fs::canonicalize(state).map_err(|error| error.to_string())?;
            private_metadata(directory, true)?;
            let directory = std::fs::canonicalize(directory).map_err(|error| error.to_string())?;
            if directory.parent() != Some(state.as_path()) {
                return Err(
                    "qualification control must be beneath the selected private state".into(),
                );
            }
            let options = pse_operations::canonical::CanonicalOptions::from_state(&state)
                .map_err(|error| error.to_string())?;
            let profile = options
                .native
                .execution
                .ok_or("qualification requires the selected reference profile")?;
            if profile.pool_memory_bytes != 128 << 30
                || profile.cpu_threads != 16
                || profile.case_lanes != 16
                || profile.math_jobs != 32
            {
                return Err(
                    "qualification requires the original 128 GiB sixteen-lane profile".into(),
                );
            }
            let request: Request =
                serde_json::from_slice(&bounded_json(&directory.join("request.json"))?)
                    .map_err(|error| error.to_string())?;
            if request.entries != 16
                || request.nonce.len() != 32
                || !request.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
                || request.canonical_database
                    != cli
                        .canonical_database
                        .as_deref()
                        .unwrap_or(&options.database)
                || request.timeout_ms == 0
                || request.timeout_ms > 120_000
            {
                return Err(
                    "qualification extent, original database association or deadline differs"
                        .into(),
                );
            }
            let deadline = Instant::now()
                .checked_add(Duration::from_millis(request.timeout_ms))
                .ok_or("qualification deadline overflow")?;
            let controller = Arc::new(Self {
                directory,
                request,
                deadline,
                entries: Mutex::new(Entries::default()),
            });
            controller.snapshot(&Entries::default())?;
            Ok(controller)
        }
        fn snapshot(&self, entries: &Entries) -> Result<(), String> {
            write_private(
                &self.directory.join("entries.json"),
                &serde_json::json!({
                    "nonce": self.request.nonce, "canonical_database": self.request.canonical_database,
                    "pid": std::process::id(), "entries": entries.observations,
                    "active": entries.active, "maximum": entries.maximum,
                }),
            )
        }
        fn signal(&self, name: &str) -> Result<bool, String> {
            let path = self.directory.join(name);
            match std::fs::symlink_metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(error) => Err(error.to_string()),
                Ok(_) => {
                    let value: String = serde_json::from_slice(&bounded_json(&path)?)
                        .map_err(|error| error.to_string())?;
                    if value != self.request.nonce {
                        return Err("qualification signal association differs".into());
                    }
                    Ok(true)
                }
            }
        }
        fn observe(&self, stop: &AtomicBool) -> Result<(), String> {
            let thread = format!("{:?}", std::thread::current().id());
            let ordinal = {
                let mut entries = self
                    .entries
                    .lock()
                    .map_err(|_| "qualification observation lock poisoned")?;
                if entries.owners.contains(&thread) {
                    return Ok(());
                }
                if entries.owners.len() >= self.request.entries {
                    return Err("qualified native owner entry extent exceeded".into());
                }
                entries.owners.insert(thread.clone());
                let ordinal = entries.observations.len();
                entries.observations.push(serde_json::json!({ "thread": thread, "released": false, "stop_observed": false }));
                entries.active += 1;
                entries.maximum = entries.maximum.max(entries.active);
                self.snapshot(&entries)?;
                ordinal
            };
            let waited = (|| {
                loop {
                    if stop.load(Ordering::Acquire) {
                        return Ok(true);
                    }
                    if self.signal("release.json")? {
                        return Ok(false);
                    }
                    if Instant::now() >= self.deadline {
                        return Err("qualification native entry release deadline expired".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            })();
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| "qualification observation lock poisoned")?;
            entries.active -= 1;
            entries.observations[ordinal]["released"] = serde_json::json!(true);
            entries.observations[ordinal]["stop_observed"] =
                serde_json::json!(waited.as_ref().is_ok_and(|value| *value));
            self.snapshot(&entries)?;
            waited.map(|_| ())
        }
        pub(super) fn enter(&self, stop: &AtomicBool) {
            if let Err(error) = self.observe(stop) {
                stop.store(true, Ordering::Release);
                let _ = write_private(&self.directory.join("failure.json"), &error);
            }
        }
        pub(super) async fn wait_stop(&self) -> Result<(), String> {
            loop {
                if self.signal("stop.json")? {
                    return Ok(());
                }
                if self.directory.join("failure.json").exists() {
                    return Err(String::from_utf8_lossy(&bounded_json(
                        &self.directory.join("failure.json"),
                    )?)
                    .into_owned());
                }
                if Instant::now() >= self.deadline {
                    return Err("qualification receiver stop deadline expired".into());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }
}
