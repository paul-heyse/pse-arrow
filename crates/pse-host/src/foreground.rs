// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Foreground process ownership through private native scopes and an exec gate.
use crate::admission::{Coordinator, Permit, Selection};
use crate::{HostError, HostManager, LaunchIntent, OwnedUnit, environment::Environment};
use command_fds::{CommandFdExt, FdMapping};
use std::{
    ffi::OsString,
    io::{Read, Write},
    os::unix::process::{CommandExt, ExitStatusExt},
    path::{Path, PathBuf},
    process::{Child, Command},
    time::{Duration, Instant},
};

/// Exact Unix process request; no shell or D-Bus text conversion touches its payload.
pub struct ForegroundRequest {
    /// Executable followed by raw operating-system arguments.
    pub command: Vec<OsString>,
    /// Complete selected child environment.
    pub environment: Environment,
    /// Child working directory, including non-Unicode paths.
    pub cwd: PathBuf,
    /// Require authenticated enclosing native containment; never create a sibling.
    pub require_enclosing: bool,
    /// One absolute deadline covering all startup stages.
    pub startup_deadline: Instant,
    /// Maximum settlement duration, minted once when settlement begins.
    pub drain_timeout: Duration,
}

impl std::fmt::Debug for ForegroundRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ForegroundRequest")
            .field("argument_count", &self.command.len())
            .field("environment_count", &self.environment.len())
            .field("require_enclosing", &self.require_enclosing)
            .field("startup_deadline", &self.startup_deadline)
            .field("drain_timeout", &self.drain_timeout)
            .finish_non_exhaustive()
    }
}

/// Run the same-image gated child, before constructing any async runtime.
/// This internal protocol grants no manager mutation authority.
#[allow(
    unsafe_code,
    reason = "command-fds requires early initialization before descriptors are owned"
)]
pub fn trampoline(arguments: &[OsString]) -> Result<i32, HostError> {
    if arguments.is_empty() {
        return Err(HostError::Ownership(
            "empty foreground trampoline command".into(),
        ));
    }
    // SAFETY: the binary dispatches here before creating a runtime or opening any
    // descriptors. Only standard descriptors are owned by Rust at this point.
    unsafe {
        command_fds::inherited::init_inherited_fds()?;
    }
    let descriptor = command_fds::inherited::take_fd_ownership(3)
        .map_err(|error| HostError::Ownership(format!("foreground gate: {error}")))?;
    let mut gate = std::fs::File::from(descriptor);
    let mut release = [0_u8; 1];
    match gate.read_exact(&mut release) {
        Ok(()) if release == [1] => {}
        Ok(()) => {
            return Err(HostError::Ownership(
                "invalid foreground gate release".into(),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(125),
        Err(error) => return Err(error.into()),
    }
    drop(gate);
    exec_result(Command::new(&arguments[0]).args(&arguments[1..]).exec())
}

fn exec_result(error: std::io::Error) -> Result<i32, HostError> {
    match error.kind() {
        std::io::ErrorKind::NotFound => Ok(127),
        std::io::ErrorKind::PermissionDenied => Ok(126),
        _ => Err(error.into()),
    }
}

fn pidfd(pid: u32) -> Result<std::os::fd::OwnedFd, HostError> {
    let raw = i32::try_from(pid)
        .map_err(|_| HostError::Ownership("process identity exceeds kernel PID range".into()))?;
    let pid = rustix::process::Pid::from_raw(raw)
        .ok_or_else(|| HostError::Ownership("invalid process identity".into()))?;
    Ok(
        rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty())
            .map_err(std::io::Error::from)?,
    )
}

fn preflight(group: &str, default_tasks_max: u64) -> Result<(), HostError> {
    let relative = group
        .strip_prefix('/')
        .ok_or_else(|| HostError::Ownership("nonabsolute placement".into()))?;
    if Path::new(relative)
        .components()
        .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(HostError::Ownership("uncertain current cgroup path".into()));
    }
    let current = Path::new("/sys/fs/cgroup").join(relative);
    // Shared slice ancestry is retained explicitly. Refuse leaf policies which
    // sibling placement would escape. Missing CPU/cpuset files mean that the
    // controller is not enabled in this leaf and ancestor limits are inherited.
    for file in [
        "memory.max",
        "memory.high",
        "memory.swap.max",
        "cpu.max",
        "cpuset.cpus",
        "cpuset.mems",
    ] {
        match std::fs::read_to_string(current.join(file)) {
            Ok(value) => {
                let supported = if file.starts_with("cpuset.") {
                    value.trim().is_empty()
                } else {
                    value.split_whitespace().next() == Some("max")
                };
                if !supported {
                    return Err(HostError::Ownership(format!(
                        "custom leaf {file} requires a separate native placement owner"
                    )));
                }
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound && !file.starts_with("memory.") => {
            }
            Err(error) => return Err(error.into()),
        }
    }
    let tasks = std::fs::read_to_string(current.join("pids.max"))?;
    if tasks.trim() != "max" && tasks.trim().parse::<u64>().ok() != Some(default_tasks_max) {
        return Err(HostError::Ownership(
            "custom leaf task limit requires a separate native placement owner".into(),
        ));
    }
    // Moving the caller must not itself empty the prior lifetime unit. This
    // supported entry uses the surviving application scope, never a dead binding.
    let own_pid = std::process::id().to_string();
    let population = std::fs::read_to_string(
        Path::new("/sys/fs/cgroup")
            .join(relative)
            .join("cgroup.procs"),
    )?;
    if !population.lines().any(|pid| pid != own_pid) {
        return Err(HostError::Ownership(
            "dedicated prior scope cannot anchor the foreground owner".into(),
        ));
    }
    Ok(())
}

fn status(status: std::process::ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}
fn settle_result(
    operation: Result<i32, HostError>,
    cleanup: Result<(), HostError>,
) -> Result<i32, HostError> {
    match (operation, cleanup) {
        (result, Ok(())) => result,
        (Err(operation), Err(cleanup)) => Err(HostError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        }),
        (Ok(code), Err(cleanup)) => Err(HostError::Ownership(format!(
            "original foreground status {code}; cleanup failed: {cleanup}"
        ))),
    }
}
async fn reap(child: &mut Child, deadline: Instant) -> Result<(), HostError> {
    loop {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(HostError::Deadline("foreground child reap".into()));
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

async fn settle_unit(
    manager: &HostManager,
    intent: Option<&LaunchIntent>,
    unit: &mut Option<OwnedUnit>,
    deadline: Instant,
) -> Result<(), HostError> {
    let Some(intent) = intent else {
        return Ok(());
    };
    if unit.is_none() {
        *unit = manager.reconcile_if_present(intent, deadline).await?;
    }
    if let Some(unit) = unit {
        unit.stop_and_drain(deadline).await?;
    }
    Ok(())
}

/// Execute in authenticated enclosing containment or establish caller-owned
/// scopes. The manager, rather than a resident helper, enforces caller death.
pub async fn run(request: ForegroundRequest) -> Result<i32, HostError> {
    let coordinator = Coordinator::ordinary(&request.environment)?;
    run_owned(request, coordinator, None).await
}

/// Run with an explicitly resolved host policy; construction does not grant admission.
pub async fn run_with_admission(
    request: ForegroundRequest,
    coordinator: Coordinator,
    selection: Selection,
) -> Result<i32, HostError> {
    run_owned(request, coordinator, Some(selection)).await
}

async fn run_owned(
    mut request: ForegroundRequest,
    coordinator: Coordinator,
    selection: Option<Selection>,
) -> Result<i32, HostError> {
    if request.command.is_empty() {
        return Err(HostError::Ownership("empty foreground command".into()));
    }
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let manager = tokio::time::timeout_at(
        request.startup_deadline.into(),
        HostManager::connect_user(None),
    )
    .await
    .map_err(|_| HostError::Deadline("foreground manager startup".into()))??;
    let current = tokio::time::timeout_at(request.startup_deadline.into(), manager.current_unit())
        .await
        .map_err(|_| HostError::Deadline("foreground placement authentication".into()))??;
    let native = tokio::time::timeout_at(
        request.startup_deadline.into(),
        manager.native_workload(&current),
    )
    .await
    .map_err(|_| HostError::Deadline("enclosing workload authentication".into()))??;
    if native {
        if let Some(requested) = selection
            .as_ref()
            .and_then(Selection::requested_ordinary_target)
        {
            let actual = tokio::time::timeout_at(
                request.startup_deadline.into(),
                manager.ordinary_target(&current),
            )
            .await
            .map_err(|_| HostError::Deadline("nested placement authentication".into()))??;
            if requested != actual {
                return Err(HostError::Ownership(
                    "nested ordinary placement cannot leave authenticated enclosing containment"
                        .into(),
                ));
            }
        }
        let inherited = coordinator
            .inherit(
                &request.environment,
                &manager,
                selection.as_ref(),
                &current,
                request.startup_deadline,
            )
            .await?;
        if inherited.is_none()
            && selection
                .as_ref()
                .is_some_and(|selection| !selection.profile().ordinary())
        {
            return Err(HostError::Ownership("nested finite selection requires an authenticated existing allocation; start it from an application scope".into()));
        }
        return exec_result(
            Command::new(&request.command[0])
                .args(&request.command[1..])
                .env_clear()
                .envs(request.environment)
                .current_dir(request.cwd)
                .exec(),
        );
    }
    if request.require_enclosing {
        return Err(HostError::Ownership(
            "--no-scope requires authenticated enclosing native workload containment".into(),
        ));
    }
    let (slice, slice_group, default_tasks_max) = tokio::time::timeout_at(
        request.startup_deadline.into(),
        manager.foreground_placement(&current),
    )
    .await
    .map_err(|_| HostError::Deadline("foreground slice authentication".into()))??;
    let ordinary_placement = match selection
        .as_ref()
        .and_then(Selection::requested_ordinary_target)
    {
        Some(requested) => Some(
            manager
                .ordinary_placement(&current, requested, request.startup_deadline)
                .await?,
        ),
        None => None,
    };
    preflight(&current.control_group, default_tasks_max)?;
    let owner_intent = LaunchIntent::fresh_scope()?;
    let workload_intent = LaunchIntent::fresh_scope()?;
    let operation_intent = if selection
        .as_ref()
        .is_none_or(|selection| selection.profile().ordinary())
    {
        Some(LaunchIntent::fresh_operation(
            ordinary_placement
                .as_ref()
                .map_or(slice.as_str(), |placement| placement.slice()),
            &owner_intent,
        )?)
    } else {
        None
    };
    if let Some(selection) = &selection {
        selection.apply_pending_environment(&mut request.environment, &workload_intent);
    }
    request.environment.insert(
        "PSE_HOST_WORKLOAD_UNIT".into(),
        workload_intent.unit().into(),
    );
    let (gate_read, mut gate_write) = std::io::pipe()?;
    let mut command = Command::new("/proc/self/exe");
    command
        .arg("__foreground-exec")
        .args(&request.command)
        .env_clear()
        .envs(&request.environment)
        .current_dir(&request.cwd);
    command
        .fd_mappings(vec![FdMapping {
            parent_fd: gate_read.into(),
            child_fd: 3,
        }])
        .map_err(|error| HostError::Ownership(error.to_string()))?;
    // Spawn while still in the old scope: the owner must contain only this controller.
    let mut child = command.spawn()?;
    drop(command);
    let child_pid = child.id();
    let mut owner: Option<OwnedUnit> = None;
    let mut workload: Option<OwnedUnit> = None;
    let mut operation_group: Option<OwnedUnit> = None;
    let mut permit: Option<Permit> = None;
    let mut cancellation = None;
    let operation = async {
        let caller_fd = pidfd(std::process::id())?;
        let workload_fd = pidfd(child_pid)?;
        let startup = async {
            if let Some(selection) = selection.as_ref().filter(|selection| !selection.profile().ordinary()) {
                permit = Some(coordinator.admit(&manager, selection, &owner_intent, &workload_intent, child_pid, request.startup_deadline).await?);
                permit.as_mut().ok_or_else(||HostError::Ownership("finite reservation missing".into()))?.prepare(&manager, request.startup_deadline).await?;
            }
            owner = Some(manager.start_scope(&owner_intent, caller_fd, &current.unit, &slice, request.startup_deadline, request.drain_timeout.saturating_sub(Duration::from_secs(1))).await?);
            owner.as_ref().ok_or_else(|| HostError::Ownership("owner missing after activation".into()))?.assert_population(&[std::process::id()]).await?;
            if let Some(permit) = &permit { permit.bind(owner.as_ref().ok_or_else(||HostError::Ownership("owner missing".into()))?, request.startup_deadline).await?; }
            if let Some(intent) = &operation_intent {
                operation_group = Some(manager.start_operation_slice(intent, owner.as_ref().ok_or_else(|| HostError::Ownership("operation controller missing".into()))?, request.startup_deadline).await?);
                let observed = operation_group.as_ref().ok_or_else(|| HostError::Ownership("operation group missing after activation".into()))?.observe().await?;
                let target_group = ordinary_placement.as_ref().map_or(slice_group.as_str(), |placement| placement.control_group());
                if Path::new(&observed.control_group).parent() != Some(Path::new(target_group)) {
                    return Err(HostError::Ownership("ordinary operation escaped original selected target".into()));
                }
            }
            let workload_slice = permit.as_ref().and_then(Permit::allocation_slice)
                .or_else(|| operation_intent.as_ref().map(LaunchIntent::unit)).unwrap_or(&slice);
            let workload_group = if workload_slice == slice { slice_group.clone() } else { manager.observe(workload_slice).await?.control_group };
            let controls = match &permit { Some(permit) => permit.profile().controls()?, None => crate::ResourceControls::default() };
            workload = Some(manager.start_scope_controlled(&workload_intent, workload_fd, owner_intent.unit(), workload_slice, request.startup_deadline, request.drain_timeout.saturating_sub(Duration::from_secs(1)), &controls).await?);
            workload.as_ref().ok_or_else(|| HostError::Ownership("workload missing after activation".into()))?.assert_population(&[child_pid]).await?;
            for (unit, group) in [(&owner, &slice_group), (&workload, &workload_group)] {
                let unit = unit.as_ref().ok_or_else(|| HostError::Ownership("scope missing".into()))?;
                let observed = unit.observe().await?;
                if Path::new(&observed.control_group).parent() != Some(Path::new(group)) {
                    return Err(HostError::Ownership("new scope escaped selected slice ancestry".into()));
                }
            }
            let unit = workload.as_ref().ok_or_else(||HostError::Ownership("workload missing".into()))?;
            if let Some(permit) = &permit { permit.bind(unit, request.startup_deadline).await?; }
            else { coordinator.admit_ordinary(&manager, &unit.observe().await?, request.startup_deadline).await?; }
            if let Some(placement) = &ordinary_placement { placement.validate(&manager, request.startup_deadline).await?; }
            Ok::<(), HostError>(())
        };
        let startup = tokio::time::timeout_at(request.startup_deadline.into(), startup);
        tokio::pin!(startup);
        // Do not drop an in-flight D-Bus effect on cancellation: finish observing
        // that same launch under its original deadline, keeping the gate closed.
        tokio::select! {
            biased;
            _ = interrupt.recv() => { cancellation = Some(130); startup.await.map_err(|_| HostError::Deadline("absolute foreground startup".into()))??; },
            _ = terminate.recv() => { cancellation = Some(143); startup.await.map_err(|_| HostError::Deadline("absolute foreground startup".into()))??; },
            result = &mut startup => result.map_err(|_| HostError::Deadline("absolute foreground startup".into()))??,
        }
        if let Some(code) = cancellation { return Ok(code); }
        if child.try_wait()?.is_some() { return Err(HostError::Ownership("gated workload exited before release".into())); }
        gate_write.write_all(&[1])?;
        loop {
            if let Some(exit) = child.try_wait()? { return Ok(status(exit)); }
            tokio::select! {
                _ = interrupt.recv() => return Ok(130),
                _ = terminate.recv() => return Ok(143),
                () = tokio::time::sleep(Duration::from_millis(5)) => {},
            }
        }
    }.await;
    drop(gate_write);
    let drain_deadline = Instant::now() + request.drain_timeout;
    // Reconcile every possibly submitted intent, including interrupted replies.
    // Authoritative absence is distinct from a failed/unknown observation.
    let (workload_drained, operation_drained, owner_recovery) = tokio::join!(
        settle_unit(
            &manager,
            Some(&workload_intent),
            &mut workload,
            drain_deadline
        ),
        settle_unit(
            &manager,
            operation_intent.as_ref(),
            &mut operation_group,
            drain_deadline
        ),
        async {
            if owner.is_none() {
                owner = manager
                    .reconcile_if_present(&owner_intent, drain_deadline)
                    .await?;
            }
            Ok::<(), HostError>(())
        },
    );
    // Settle independent resources even when a previous settlement failed.
    let killed = match child.try_wait() {
        Ok(None) => child.kill().map_err(HostError::from),
        Ok(Some(_)) => Ok(()),
        Err(error) => Err(error.into()),
    };
    let reaped = reap(&mut child, drain_deadline).await;
    let finite_release = if let Some(permit) = &mut permit {
        match permit.release(&manager, drain_deadline).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(HostError::Ownership(
                "finite charge retained until original physical drain".into(),
            )),
            Err(error) => Err(error),
        }
    } else {
        Ok(())
    };
    let released = if let Some(unit) = &mut owner {
        unit.release_reference(drain_deadline).await
    } else {
        Ok(())
    };
    let mut cleanup = Ok(());
    // Every resource was attempted above; retain all of its settlement failures.
    for next in [
        workload_drained,
        operation_drained,
        owner_recovery,
        killed,
        reaped,
        finite_release,
        released,
    ] {
        cleanup = match (cleanup, next) {
            (Ok(()), result) | (result, Ok(())) => result,
            (Err(operation), Err(cleanup)) => Err(HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            }),
        };
    }
    settle_result(operation, cleanup)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_debug_redacts_environment_arguments_and_cwd() {
        let request = ForegroundRequest {
            command: vec!["/secret/command".into(), "argument-secret".into()],
            environment: [("TOKEN".into(), "environment-secret".into())]
                .into_iter()
                .collect(),
            cwd: "/secret/directory".into(),
            require_enclosing: false,
            startup_deadline: Instant::now(),
            drain_timeout: Duration::from_secs(1),
        };
        let rendered = format!("{request:?}");
        assert!(!rendered.contains("secret"));
        assert!(
            rendered.contains("argument_count: 2") && rendered.contains("environment_count: 1")
        );
    }
}
