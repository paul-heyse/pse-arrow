// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `pse-worker`: serve bounded canonical study actions on the supervised deployment.
//! Claims, cancellation and recovery use native fences. Scientific work stays outside
//! transactions, and completed native reports are retained as compact result selections.

// Match the library's depth for the shared runtime's nested async `Send` proof.
#![recursion_limit = "512"]
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line worker reports on its own terminal"
)]

use std::num::NonZeroUsize;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

#[path = "support/signals.rs"]
mod signals;

use clap::Parser;
#[cfg(test)]
use pse_runtime::workflow::WorkerSettings;
use pse_runtime::workflow::{
    CanonicalDeployment, DurabilitySelection, LeasePolicy, NativeStudyWorkerInputs, Operations,
    OuterAttestation, Runtime, WorkflowError,
};
use pse_runtime::{CancelSource, ResourceBudget, SharedRuntime};

/// Managed canonical study worker.
#[derive(Debug, Parser)]
#[command(name = "pse-worker", version)]
struct Cli {
    /// Explicit native application handoff produced by NativeStudyWorker.
    #[arg(long)]
    native_input: Option<std::path::PathBuf>,
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
    #[arg(long)]
    poll_ms: Option<u64>,
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
    allocation: pse_runtime::workflow::NativeAllocation,
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
                ..pse_runtime::math::MathPolicy::for_pool(execution.pool_memory_bytes)
            },
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
    let threads = cli
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, NonZeroUsize::get));
    let math = pse_runtime::math::MathPolicy::for_pool(memory);
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
    })
}

fn verify_placement(allocation: &pse_runtime::workflow::NativeAllocation) -> Result<(), String> {
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
    verify_placement_hierarchy(
        root,
        &own,
        allocation.native_worker_memory_bytes,
        execution.cpu_threads,
    )?;
    if std::thread::available_parallelism()
        .map_err(|error| error.to_string())?
        .get()
        < execution.cpu_threads
    {
        return Err("worker affinity cannot admit sixteen case lanes".into());
    }
    Ok(())
}

fn verify_placement_hierarchy(
    root: &std::path::Path,
    own: &std::path::Path,
    worker_bytes: usize,
    cpu_threads: usize,
) -> Result<(), String> {
    let read = |group: &std::path::Path, name: &str| {
        let path = group.join(name);
        std::fs::read_to_string(&path)
            .map_err(|error| format!("read managed placement {}: {error}", path.display()))
    };
    let cap = read(own, "memory.max")?;
    if cap.trim().parse::<usize>().ok() != Some(worker_bytes) {
        return Err("managed worker process cap differs from its profile".into());
    }
    let mut group = own;
    let mut selected_cpu_quota = false;
    while group != root {
        let memory = read(group, "memory.max")?;
        if memory.trim() != "max"
            && memory
                .trim()
                .parse::<usize>()
                .map_err(|error| error.to_string())?
                < worker_bytes
        {
            return Err("ancestor memory cap cannot admit the exact worker profile".into());
        }
        // A service can inherit its parent's quota without enabling the CPU
        // controller locally. Missing controls are known only from kernel readback;
        // an active controller with an unreadable limit still refuses admission.
        let cpu = match std::fs::read_to_string(group.join("cpu.max")) {
            Ok(cpu) => Some(cpu),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let controllers = read(group, "cgroup.controllers")?;
                if controllers.split_whitespace().any(|name| name == "cpu") {
                    return Err(format!(
                        "active CPU controller lacks quota readback: {}",
                        group.display()
                    ));
                }
                None
            }
            Err(error) => {
                return Err(format!(
                    "read managed CPU quota {}: {error}",
                    group.display()
                ));
            }
        };
        if let Some(cpu) = cpu {
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
                if period == 0 || quota / period < cpu_threads as u64 {
                    return Err("ancestor CPU quota cannot admit sixteen case lanes".into());
                }
                // Host admission owns the exact logical-CPU envelope; execution
                // width counts scientific lanes and can be narrower than it.
                selected_cpu_quota = true;
            }
        }
        group = group
            .parent()
            .ok_or("worker placement has no cgroup ancestor")?;
    }
    if !selected_cpu_quota {
        return Err("selected finite ancestor CPU quota was not observed".into());
    }
    Ok(())
}

async fn runtime(
    cli: &Cli,
    stop: &CancelSource,
) -> Result<(Runtime, NativeStudyWorkerInputs), String> {
    if stop.token().is_cancelled() {
        return Err("native worker startup cancelled".into());
    }
    let path = cli
        .native_input
        .as_deref()
        .ok_or("--native-input must select an explicit native application handoff")?;
    let selected = NativeStudyWorkerInputs::read(path).map_err(|error| error.to_string())?;
    if cli
        .canonical_database
        .as_deref()
        .is_some_and(|database| database != selected.database())
        || cli.until_idle
        || cli.maximum_actions.is_some()
        || cli
            .maximum_in_flight
            .is_some_and(|width| Some(width) != selected.settings().maximum_in_flight)
        || cli
            .poll_ms
            .is_some_and(|poll| Duration::from_millis(poll) != selected.settings().poll)
    {
        return Err(
            "worker overrides differ from the original native sustained application".into(),
        );
    }
    verify_placement(&selected.allocation())
        .map_err(|error| format!("managed placement: {error}"))?;
    let shared = SharedRuntime::build(resource_budget(cli, selected.allocation())?)
        .map_err(|error| format!("runtime budget: {error}"))?;
    let model = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .map_err(|error| error.to_string())?,
    )
    .bind_model(pse_schema::shared_registry().map_err(|error| error.to_string())?)
    .map_err(|error| error.to_string())?;
    let defaults = LeasePolicy::default();
    let policy = LeasePolicy::new(
        Duration::from_secs(cli.lease_seconds),
        Duration::from_millis(cli.heartbeat_ms),
        defaults.batch(),
        defaults.flush(),
    )
    .map_err(|error| error.to_string())?;
    let worker = cli
        .name
        .clone()
        .unwrap_or_else(|| Operations::process_worker("pse-worker"));
    // Same callable Rust composition as native_study: schema-ready connection,
    // explicit shared budget/model, then the real Runtime::serve scientific root.
    let actual = std::env::current_exe().map_err(|error| error.to_string())?;
    pse_buildinfo::verify_loaded_module(&actual).map_err(|error| error.to_string())?;
    let mut file = std::fs::File::open(&actual).map_err(|error| error.to_string())?;
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
    hash.part_reader(
        file.metadata().map_err(|error| error.to_string())?.len(),
        &mut file,
        &|| Ok(()),
    )
    .map_err(|error| error.to_string())?;
    let store = selected
        .connect()
        .await
        .map_err(|error| error.to_string())?;
    let built = Runtime::from_shared(
        shared,
        model,
        CanonicalDeployment::new(
            store.clone(),
            OuterAttestation {
                source: None,
                build: hash.finish_hash(),
            },
            None,
        ),
    )
    .and_then(|runtime| runtime.with_durability(DurabilitySelection::Durable { worker, policy }));
    match built {
        Ok(runtime) => Ok((runtime, selected)),
        Err(error) => {
            let cleanup = store.disconnect().await;
            Err(format!(
                "native runtime construction: {error}; connection cleanup: {cleanup:?}"
            ))
        }
    }
}

fn diagnostic_report(error: &WorkflowError) -> String {
    format!("{:#?}", error.boundary_diagnostic())
}

fn report(error: &WorkflowError) {
    let code = miette::Diagnostic::code(error)
        .map(|code| format!(" [{code}]"))
        .unwrap_or_default();
    eprintln!("error{code}: {error}");
    eprintln!("diagnostic: {}", diagnostic_report(error));
    if let Some(help) = miette::Diagnostic::help(error) {
        eprintln!("help: {help}");
    }
}

#[cfg(test)]
fn worker_settings(
    cli: &Cli,
    allocation: pse_runtime::workflow::NativeAllocation,
) -> WorkerSettings {
    WorkerSettings {
        poll: cli
            .poll_ms
            .map(Duration::from_millis)
            .unwrap_or(WorkerSettings::default().poll),
        maximum_actions: cli.maximum_actions,
        maximum_in_flight: cli
            .maximum_in_flight
            .or(allocation.execution.map(|profile| profile.case_lanes)),
        until_idle: cli.until_idle,
        ..WorkerSettings::default()
    }
}

async fn serve(
    _cli: &Cli,
    runtime: Runtime,
    selected: &NativeStudyWorkerInputs,
    stop: &CancelSource,
    #[cfg(feature = "canonical-tests")] qualification: Option<Arc<qualification::Controller>>,
) -> Result<usize, WorkflowError> {
    let settings = selected.settings();
    let result = async {
        #[cfg(feature = "canonical-tests")]
        if let Some(qualification) = qualification {
            let work = runtime.serve(settings, stop);
            tokio::pin!(work);
            return tokio::select! {
                result = &mut work => result,
                result = qualification.wait_stop() => {
                    stop.cancel();
                    let drained = work.await;
                    match (result, drained) {
                        (Ok(()), drained) => drained,
                        (Err(error), drained) => {
                            let operation = WorkflowError::Internal(format!("native qualification control: {error}"));
                            match drained {
                                Ok(_) => Err(operation),
                                Err(cleanup) => Err(WorkflowError::Settlement {
                                    operation: Box::new(operation), cleanup: Box::new(cleanup),
                                }),
                            }
                        },
                    }
                }
            };
        }
        runtime.serve(settings, stop).await
    }.await;
    finish(&runtime, selected, result).await
}

async fn finish(
    runtime: &Runtime,
    selected: &NativeStudyWorkerInputs,
    result: Result<usize, WorkflowError>,
) -> Result<usize, WorkflowError> {
    runtime.close();
    runtime.clear_program_cache();
    let native = runtime.native().shutdown_native_lanes().await;
    let joined = native.is_ok();
    let result = match (result, native) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
        (Err(operation), Err(cleanup)) => Err(WorkflowError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup.into()),
        }),
    };
    let cleanup = match runtime.canonical_store() {
        // This is the final application owner, after serve and native lanes
        // joined. A known response can precede its actual outbound transport
        // drain; seal admission and join that suffix instead of an early busy
        // refusal. Live escaped readers still refuse through the existing owner.
        Ok(store) => store
            .drain_owned_connection_until(tokio::time::Instant::now() + Duration::from_secs(30))
            .await
            .map_err(WorkflowError::from),
        Err(error) => Err(error),
    };
    let result = combine(result, cleanup);
    let published = selected.publish_terminal(joined, &result);
    combine(result, published)
}

fn combine<T>(
    result: Result<T, WorkflowError>,
    cleanup: Result<(), WorkflowError>,
) -> Result<T, WorkflowError> {
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(operation), Err(cleanup)) => Err(WorkflowError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        }),
    }
}

fn main() -> ExitCode {
    if let Err(error) = ensure_openmp_environment() {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    // block_on polls its root on the calling thread. RUST_MIN_STACK does not
    // resize the OS main stack: give the actual application root the same
    // qualified stack as native structural work, preserving a larger selection.
    let stack = match std::env::var("RUST_MIN_STACK") {
        Ok(value) => match value.parse::<usize>() {
            Ok(value) if value > 0 => {
                value.max(pse_runtime::math::MathPolicy::default().stack_bytes)
            }
            _ => {
                eprintln!("error: RUST_MIN_STACK must be a positive byte count");
                return ExitCode::FAILURE;
            }
        },
        Err(std::env::VarError::NotPresent) => pse_runtime::math::MathPolicy::default().stack_bytes,
        Err(error) => {
            eprintln!("error: RUST_MIN_STACK: {error}");
            return ExitCode::FAILURE;
        }
    };
    match std::thread::Builder::new()
        .name("pse-worker-root".into())
        .stack_size(stack)
        .spawn(move || run(stack))
    {
        Ok(root) => match root.join() {
            Ok(status) => status,
            Err(panic) => std::panic::resume_unwind(panic),
        },
        Err(error) => {
            eprintln!("error: worker root: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(stack: usize) -> ExitCode {
    if let Err(error) = ensure_openmp_environment() {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    let _telemetry_host =
        match pse_runtime::tooling::telemetry::Host::from_environment("pse-worker") {
            Ok(host) => host,
            Err(_) => {
                eprintln!("pse-telemetry: native-host-initialization-refused");
                None
            }
        };
    let cli = Cli::parse();
    let executor = match tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(stack)
        .enable_all()
        .build()
    {
        Ok(executor) => executor,
        Err(error) => {
            eprintln!("error: executor: {error}");
            return ExitCode::FAILURE;
        }
    };
    let _executor_metrics = pse_runtime::tooling::runtime_metrics::register_executor(
        executor.handle(),
        pse_runtime::tooling::runtime_metrics::ExecutorRole::Worker,
    );
    let stop = CancelSource::new();
    // One original source spans startup, readiness, serving and native drain.
    let interrupt = match executor.block_on(async { signals::ProcessInterrupt::new(stop.clone()) })
    {
        Ok(interrupt) => interrupt,
        Err(error) => {
            eprintln!("error: worker stop listener: {error}");
            return ExitCode::FAILURE;
        }
    };
    let (runtime, selected) = match executor.block_on(runtime(&cli, &stop)) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error: runtime initialization: {error}");
            return ExitCode::FAILURE;
        }
    };
    // Keep the actual application root through every startup refusal as well as
    // serving: no ready-publication or qualification error bypasses native drain.
    let diagnostics = runtime.observations().clone();
    #[cfg(feature = "canonical-tests")]
    let startup = (|| {
        let qualification = cli
            .qualification_native_entry
            .as_deref()
            .map(|directory| qualification::Controller::open(directory, &cli, &selected))
            .transpose()
            .map_err(|error| {
                WorkflowError::Internal(format!("native entry qualification: {error}"))
            })?;
        if let Some(controller) = &qualification {
            let observer = controller.clone();
            pse_runtime::math::install_qualified_native_entry(Arc::new(move |flag| {
                observer.enter(flag)
            }))
            .map_err(|error| WorkflowError::Internal(error.to_string()))?;
        }
        selected.publish_ready(&runtime)?;
        Ok::<_, WorkflowError>(qualification)
    })();
    #[cfg(not(feature = "canonical-tests"))]
    let startup = selected.publish_ready(&runtime);
    let result = match startup {
        #[cfg(feature = "canonical-tests")]
        Ok(qualification) => {
            executor.block_on(serve(&cli, runtime, &selected, &stop, qualification))
        }
        #[cfg(not(feature = "canonical-tests"))]
        Ok(()) => executor.block_on(serve(&cli, runtime, &selected, &stop)),
        Err(error) => {
            stop.cancel();
            executor.block_on(finish(&runtime, &selected, Err(error)))
        }
    };
    executor.block_on(interrupt.finish());
    // This process composition root joins its optional publisher after scientific
    // lanes have drained, outside RuntimeCaller admission and native ownership locks.
    diagnostics.stop_publisher();
    if let Some(error) = diagnostics.writer_failure() {
        eprintln!("diagnostic checkpoint unavailable: {error}");
    }
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

    #[test]
    fn workflow_error_diagnostic_report_retains_typed_detail_and_nested_cause() {
        use pse_model::diagnostic::{
            BoundaryClass, BoundaryDiagnostic, DiagnosticRule, DiagnosticStage,
        };

        let cause = pse_runtime::math::MathRuntimeError::Infrastructure(
            "specific native admission failure".into(),
        );
        let diagnostic = BoundaryDiagnostic::new(
            BoundaryClass::Infrastructure,
            DiagnosticStage::ModelingAdmission,
            [],
            DiagnosticRule::WorkflowUnclassified,
        )
        .with_code(pse_model::diagnostic::DiagnosticCode::RuntimeInfrastructure);
        let error = WorkflowError::ModelingAdmission {
            diagnostic: Box::new(diagnostic),
            cause: Box::new(cause),
        };

        let rendered = diagnostic_report(&error);
        assert!(rendered.contains("ModelingAdmission"), "{rendered}");
        assert!(rendered.contains("WorkflowUnclassified"), "{rendered}");
        assert!(
            rendered.contains("specific native admission failure"),
            "{rendered}"
        );
        assert!(rendered.contains("RuntimeInfrastructure"), "{rendered}");
    }
    fn placement_fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("reference.slice");
        let leaf = parent.join("primary.service");
        std::fs::create_dir_all(&leaf).unwrap();
        std::fs::write(leaf.join("memory.max"), (140_usize << 30).to_string()).unwrap();
        std::fs::write(leaf.join("cgroup.controllers"), "memory pids\n").unwrap();
        std::fs::write(parent.join("memory.max"), (160_usize << 30).to_string()).unwrap();
        std::fs::write(parent.join("cgroup.controllers"), "cpu memory pids\n").unwrap();
        std::fs::write(parent.join("cpu.max"), "1600000 100000\n").unwrap();
        (directory, parent, leaf)
    }

    #[test]
    fn managed_placement_inherits_exact_finite_cpu_quota_with_disabled_leaf_controller() {
        let (root, _, leaf) = placement_fixture();
        assert!(!leaf.join("cpu.max").exists());
        assert_eq!(
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16),
            Ok(())
        );
    }

    #[test]
    fn managed_placement_refuses_active_cpu_controller_without_quota_readback() {
        let (root, _, leaf) = placement_fixture();
        std::fs::write(leaf.join("cgroup.controllers"), "cpu memory pids\n").unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(
            error.contains("active CPU controller lacks quota readback"),
            "{error}"
        );
    }

    #[test]
    fn managed_placement_refuses_narrower_cpu_ancestor_even_with_exact_leaf_quota() {
        let (root, parent, leaf) = placement_fixture();
        std::fs::write(leaf.join("cgroup.controllers"), "cpu memory pids\n").unwrap();
        std::fs::write(leaf.join("cpu.max"), "1600000 100000\n").unwrap();
        std::fs::write(parent.join("cpu.max"), "800000 100000\n").unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(error.contains("ancestor CPU quota cannot admit"), "{error}");
    }

    #[test]
    fn managed_placement_refuses_narrower_memory_ancestor() {
        let (root, parent, leaf) = placement_fixture();
        std::fs::write(parent.join("memory.max"), (139_usize << 30).to_string()).unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(
            error.contains("ancestor memory cap cannot admit"),
            "{error}"
        );
    }

    #[test]
    fn managed_placement_admits_wider_logical_cpu_envelope_than_scientific_width() {
        let (root, parent, leaf) = placement_fixture();
        std::fs::write(parent.join("cpu.max"), "3200000 100000\n").unwrap();
        assert_eq!(
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16),
            Ok(())
        );
    }

    #[test]
    fn managed_placement_requires_an_observed_finite_ancestor_cpu_quota() {
        {
            let quota = "max 100000\n";
            let (root, parent, leaf) = placement_fixture();
            std::fs::write(parent.join("cpu.max"), quota).unwrap();
            let error =
                verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
            assert!(
                error.contains("selected finite ancestor CPU quota was not observed"),
                "quota={quota:?}, error={error}"
            );
        }
        let (root, parent, leaf) = placement_fixture();
        std::fs::remove_file(parent.join("cpu.max")).unwrap();
        std::fs::write(parent.join("cgroup.controllers"), "memory pids\n").unwrap();
        let error =
            verify_placement_hierarchy(root.path(), &leaf, 140_usize << 30, 16).unwrap_err();
        assert!(
            error.contains("selected finite ancestor CPU quota was not observed"),
            "{error}"
        );
    }

    fn legacy_allocation(bytes: usize) -> pse_runtime::workflow::NativeAllocation {
        pse_runtime::workflow::NativeAllocation::materialize(
            bytes.checked_mul(2).unwrap() + (512 << 20),
            512 << 20,
            2,
            bytes,
            None,
        )
        .unwrap()
    }
    #[test]
    fn selected_functional_worker_consumes_case_lanes_and_preserves_bounded_override() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profiles = pse_runtime::workflow::ProductionProfiles::read(
            &root.join(".config/agent-capacity.toml"),
        )
        .unwrap();
        let allocation = profiles
            .select(pse_runtime::workflow::ProductionProfileName::Functional)
            .unwrap();
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let selected = allocation.execution.unwrap();
        assert_eq!(selected.case_lanes, 2);
        assert_eq!(selected.math_jobs, 4);
        assert_eq!(
            worker_settings(&cli, allocation).maximum_in_flight,
            Some(selected.case_lanes)
        );
        let smaller = Cli::try_parse_from(["pse-worker", "--maximum-in-flight", "1"]).unwrap();
        assert!(resource_budget(&smaller, allocation).is_ok());
        assert_eq!(
            worker_settings(&smaller, allocation).maximum_in_flight,
            Some(1)
        );
        let larger = Cli::try_parse_from(["pse-worker", "--maximum-in-flight", "3"]).unwrap();
        assert!(resource_budget(&larger, allocation).is_err());
    }

    #[test]
    fn selected_reference_budget_consumes_exact_allocated_fields() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let execution = pse_runtime::workflow::NativeExecutionAllocation {
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
        let allocation = pse_runtime::workflow::NativeAllocation::materialize(
            160 << 30,
            16 << 30,
            1,
            140 << 30,
            Some(execution),
        )
        .unwrap();
        let budget = resource_budget(&cli, allocation).unwrap();
        assert_eq!(budget.memory_limit_bytes.get(), 128 << 30);
        assert_eq!(budget.math.worker_bytes, 16 << 30);
        assert_eq!(budget.math.jobs, 32);
        assert_eq!(budget.threads.pool_threads.get(), 16);
        assert_eq!(
            budget.math.workspace_bytes,
            pse_runtime::math::MathPolicy::for_pool(execution.pool_memory_bytes).workspace_bytes
        );
        let cli = Cli::try_parse_from(["pse-worker", "--threads", "8"]).unwrap();
        assert!(resource_budget(&cli, allocation).is_err());
    }

    #[test]
    fn parsed_default_budget_admits_default_compiler_scratch() {
        let cli = Cli::try_parse_from(["pse-worker"]).unwrap();
        let budget = resource_budget(&cli, legacy_allocation(1 << 30)).unwrap();
        let available = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        assert_eq!(budget.threads.pool_threads.get(), available);
        assert_eq!(budget.threads.target_partitions.get(), available);
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
        entry_timeout_ms: u64,
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
        fallback_deadline: Instant,
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
        pub(super) fn open(
            directory: &Path,
            cli: &super::Cli,
            selected: &pse_runtime::workflow::NativeStudyWorkerInputs,
        ) -> Result<Arc<Self>, String> {
            let state =
                std::fs::canonicalize(selected.state()).map_err(|error| error.to_string())?;
            private_metadata(directory, true)?;
            let directory = std::fs::canonicalize(directory).map_err(|error| error.to_string())?;
            if directory.parent() != Some(state.as_path()) {
                return Err(
                    "qualification control must be beneath the selected private state".into(),
                );
            }
            let profile = selected
                .allocation()
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
            if !(1..=profile.case_lanes).contains(&request.entries)
                || request.nonce.len() != 32
                || !request.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
                || request.canonical_database
                    != cli
                        .canonical_database
                        .as_deref()
                        .unwrap_or(selected.database())
                || request.entry_timeout_ms == 0
                || request.entry_timeout_ms > 90_000
            {
                return Err(
                    "qualification extent, original database association or deadline differs"
                        .into(),
                );
            }
            let startup = Duration::from_secs(45);
            let entry = Duration::from_secs(90);
            let operation = Duration::from_secs(90);
            let drain = Duration::from_secs(45);
            let fallback_deadline = Instant::now()
                // A crashed observer still has a finite receiver owner. Startup,
                // entry, public operation and drain keep their separate bounds;
                // startup cannot consume an actual native owner's entry clock.
                .checked_add(startup + entry + operation + drain)
                .ok_or("qualification deadline overflow")?;
            let controller = Arc::new(Self {
                directory,
                request,
                fallback_deadline,
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
            let entry_deadline = Instant::now()
                .checked_add(Duration::from_millis(self.request.entry_timeout_ms))
                .ok_or("qualification native entry deadline overflow")?;
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
                    if Instant::now() >= entry_deadline {
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
                if Instant::now() >= self.fallback_deadline {
                    return Err("qualification receiver stop deadline expired".into());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }
}
