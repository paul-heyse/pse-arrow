// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native worker placement, runtime construction and terminal settlement.

use std::{num::NonZeroUsize, sync::Arc, time::Duration};

use pse_runtime::workflow::{
    CanonicalDeployment, DurabilitySelection, LeasePolicy, NativeStudyWorkerInputs, Operations,
    OuterAttestation, Runtime, WorkflowError,
};
use pse_runtime::{CancelSource, ResourceBudget, SharedRuntime};

pub(super) const LEASE_SECONDS: u64 = 30;
pub(super) const HEARTBEAT_MS: u64 = 10_000;

/// Managed canonical study worker.
#[derive(Debug, Default)]
#[cfg_attr(feature = "product-operators", derive(clap::Parser))]
#[cfg_attr(feature = "product-operators", command(name = "pse-worker", version))]
pub(super) struct Cli {
    /// Explicit native application handoff produced by NativeStudyWorker.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) native_input: Option<std::path::PathBuf>,
    /// Canonical database on the supervised endpoint; defaults to the profile selection.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) canonical_database: Option<String>,
    /// Worker identity recorded on leases; defaults to `pse-worker:<host>:<pid>`.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) name: Option<String>,
    /// Lease length in seconds.
    #[cfg_attr(feature = "product-operators", arg(long, default_value_t = LEASE_SECONDS))]
    pub(super) lease_seconds: u64,
    /// Heartbeat period in milliseconds.
    #[cfg_attr(feature = "product-operators", arg(long, default_value_t = HEARTBEAT_MS))]
    pub(super) heartbeat_ms: u64,
    /// Idle poll period in milliseconds.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) poll_ms: Option<u64>,
    /// Stop after this many processed scheduling actions.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) maximum_actions: Option<usize>,
    /// Concurrent case lanes on this process's shared allocation.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) maximum_in_flight: Option<usize>,
    /// Stop as soon as a complete bounded discovery pass finds no work.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) until_idle: bool,
    /// Engine pool budget in MiB; a selected execution profile fixes it exactly.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) memory_mib: Option<usize>,
    /// Engine pool threads; defaults to the available parallelism.
    #[cfg_attr(feature = "product-operators", arg(long))]
    pub(super) threads: Option<usize>,
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
pub(super) fn ensure_openmp_environment() -> Result<(), String> {
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

pub(super) fn resource_budget(
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

pub(super) fn verify_placement_hierarchy(
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

pub(super) async fn runtime(
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

pub(super) async fn finish(
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

pub(super) fn combine<T>(
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
