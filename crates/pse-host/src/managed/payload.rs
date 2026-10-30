// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original service-owned payload placement, capture and independent physical settlement.
use super::streams::{self, ChunkObserver, TeeReport};
use crate::{
    HostError, HostManager, LaunchIntent, OwnedUnit, ResourceControls, UnitObservation,
    admission::{Coordinator, Permit, Selection},
    environment::Environment,
    invocation::{InvocationDirectory, InvocationEvidence, InvocationLaunchGuard},
};
use command_fds::{CommandFdExt, FdMapping};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    io::Write,
    os::unix::{fs::OpenOptionsExt, process::ExitStatusExt},
    path::PathBuf,
    process::{ExitStatus, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
pub(super) mod nested;

/// Original Unix command. Debug deliberately excludes payload arguments, paths and environment.
pub struct PayloadCommand {
    /// Literal program, resolved by the command owner.
    pub program: PathBuf,
    /// Literal operating-system arguments.
    pub arguments: Vec<OsString>,
    /// Original working directory.
    pub cwd: PathBuf,
    /// Complete selected environment; execution clears inherited service values.
    pub environment: Environment,
}
impl std::fmt::Debug for PayloadCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadCommand")
            .field("argument_count", &self.arguments.len())
            .field("environment_count", &self.environment.len())
            .finish_non_exhaustive()
    }
}

/// Explicit operation observation/settlement policy, independent of host resource capacity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PayloadPolicy {
    /// Absolute startup budget is minted once at service initialization.
    pub startup: Duration,
    /// One common deadline for independent resource/capture settlement attempts.
    pub drain: Duration,
    /// Required per-stream chunk write observation policy.
    pub write: Duration,
}

/// Persisted before any child/allocation/scope effects. This snapshot grants no stop authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PayloadIntent {
    pub version: u32,
    pub controller: LaunchIntent,
    pub workload: LaunchIntent,
    pub operation: Option<LaunchIntent>,
    #[serde(default)]
    pub subgroup: Option<crate::SubgroupCreationIntent>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PayloadCustody {
    pub(super) version: u32,
    pub(super) manager: String,
    pub(super) workload: crate::manager::UnitBinding,
    pub(super) child: pse_diagnostics::process_facts::ProcessIdentity,
    pub(super) operation: Option<crate::manager::UnitBinding>,
    pub(super) reservation: Option<crate::admission::PermitRecovery>,
    #[serde(default)]
    pub(super) subgroup: Option<crate::DirectoryBinding>,
    #[serde(default)]
    pub(super) creation: Option<crate::SubgroupCreationIntent>,
}

/// Original physical resource results. Only the settling owner constructs this value.
pub struct PayloadReport<O: ChunkObserver> {
    pub(super) child: Result<ExitStatus, HostError>,
    pub(super) capture: Result<TeeReport<tokio::fs::File, tokio::fs::File, O>, HostError>,
    pub(super) operation_error: Option<HostError>,
    pub(super) closure: Result<(), HostError>,
    pub(super) workload: Result<Option<UnitObservation>, HostError>,
    pub(super) operation: Result<Option<UnitObservation>, HostError>,
    pub(super) kill: Result<(), HostError>,
    pub(super) permit: Result<bool, HostError>,
    pub(super) borrowed: bool,
    pub(super) cancelled: bool,
    pub(super) retained: RetainedResources,
    pub(super) subgroup: Option<nested::SubgroupReport>,
}
#[derive(Debug)]
pub(super) struct RetainedResources {
    _workload: Option<OwnedUnit>,
    _operation: Option<OwnedUnit>,
    _permit: Option<Permit>,
    _subgroup: Option<crate::OwnedSubgroup>,
}
impl<O: ChunkObserver> std::fmt::Debug for PayloadReport<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadReport")
            .field("child", &self.child)
            .field("operation_error", &self.operation_error)
            .field("closure", &self.closure)
            .field("capture", &self.capture)
            .field("workload", &self.workload)
            .field("operation", &self.operation)
            .field("permit", &self.permit)
            .finish_non_exhaustive()
    }
}
impl<O: ChunkObserver> PayloadReport<O> {
    /// Native child status only after actual wait/reap; never inferred from the service manager.
    pub fn child_exit(&self) -> Option<i32> {
        self.child.as_ref().ok().map(|status| {
            status
                .code()
                .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
        })
    }
    /// Positive original-resource settlement, independent of child success and raw log custody.
    pub(crate) fn resources_settled(&self) -> bool {
        self.child.is_ok()
            && self.closure.is_ok()
            && self.workload.is_ok()
            && self.operation.is_ok()
            && self.kill.is_ok()
            && matches!(self.permit, Ok(true))
            && self
                .subgroup
                .as_ref()
                .is_none_or(nested::SubgroupReport::settled)
    }
}

async fn launch_lock(
    directory: &InvocationDirectory,
    deadline: Instant,
) -> Result<InvocationLaunchGuard<'_>, HostError> {
    loop {
        match directory.try_launch_lock() {
            Ok(guard) => return Ok(guard),
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(HostError::Deadline("managed launch exclusion".into()));
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(error) => return Err(error),
        }
    }
}
fn deadline(duration: Duration) -> Result<Instant, HostError> {
    Instant::now().checked_add(duration).ok_or_else(|| {
        HostError::Ownership("managed observation horizon exceeds clock range".into())
    })
}
fn pidfd(pid: u32) -> Result<std::os::fd::OwnedFd, HostError> {
    let pid = i32::try_from(pid)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .ok_or_else(|| HostError::Ownership("managed child has an invalid PID".into()))?;
    rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty())
        .map_err(std::io::Error::from)
        .map_err(HostError::from)
}
async fn settle_unit(
    manager: &HostManager,
    intent: Option<&LaunchIntent>,
    unit: &mut Option<OwnedUnit>,
    deadline: Instant,
) -> Result<Option<UnitObservation>, HostError> {
    let Some(intent) = intent else {
        return Ok(None);
    };
    if unit.is_none() {
        *unit = manager.reconcile_if_present(intent, deadline).await?;
    }
    match unit {
        Some(unit) => unit.stop_and_drain(deadline).await.map(Some),
        None => Ok(None),
    }
}
async fn captured_child<O: ChunkObserver>(
    directory: &InvocationDirectory,
    command: &PayloadCommand,
    trampoline: &std::path::Path,
    observer: O,
    write: Duration,
    startup_deadline: Instant,
) -> Result<
    (
        tokio::process::Child,
        std::io::PipeWriter,
        streams::TeeHandle<tokio::fs::File, tokio::fs::File, O>,
    ),
    HostError,
> {
    let file = |name| {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(directory.path().join(name))
            .map(tokio::fs::File::from_std)
    };
    let sinks = (file("stdout.raw")?, file("stderr.raw")?);
    let (gate_read, gate_write) = std::io::pipe()?;
    let mut process = tokio::process::Command::new(trampoline);
    process
        .arg("__foreground-exec")
        .arg(&command.program)
        .args(&command.arguments)
        .env_clear()
        .envs(&command.environment)
        .current_dir(&command.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    process
        .as_std_mut()
        .fd_mappings(vec![FdMapping {
            parent_fd: gate_read.into(),
            child_fd: 3,
        }])
        .map_err(|error| HostError::Ownership(error.to_string()))?;
    let mut child = process.spawn()?;
    drop(process);
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (Some(stdout), Some(stderr)) = (stdout, stderr) else {
        let operation = HostError::Ownership("managed child lacks required piped output".into());
        let _ = child.start_kill();
        let cleanup = tokio::time::timeout_at(startup_deadline.into(), child.wait()).await;
        return Err(match cleanup {
            Ok(Ok(_)) => operation,
            other => HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(HostError::Ownership(format!(
                    "unavailable-pipe child reap: {other:?}"
                ))),
            },
        });
    };
    let capture = streams::start(stdout, stderr, sinks, observer, None, write);
    Ok((child, gate_write, capture))
}

/// Execute one top-level managed payload while keeping the authenticated controller in its
/// original notify service. A caller must establish origin and parent-launch lifetime first.
/// Readiness is sent only after original placement, admission and required capture exist.
pub(crate) async fn run<O: ChunkObserver>(
    directory: Arc<InvocationDirectory>,
    controller: LaunchIntent,
    root: PathBuf,
    mut command: PayloadCommand,
    selection: Selection,
    trampoline: PathBuf,
    policy: PayloadPolicy,
    cancellation: CancellationToken,
    observer: O,
    parent: Option<super::supervisor::ManagedParent>,
) -> Result<PayloadReport<O>, HostError> {
    if let Some(parent) = parent {
        return nested::run(
            directory,
            controller,
            root,
            command,
            trampoline,
            policy,
            cancellation,
            observer,
            parent,
        )
        .await;
    }
    let startup_deadline = deadline(policy.startup)?;
    let manager = HostManager::connect_user(None).await?;
    let (original, slice, slice_group) = manager.managed_service_placement(&controller).await?;
    let placement = match selection.requested_ordinary_target() {
        Some(target) if selection.profile().ordinary() => Some(
            manager
                .ordinary_placement(&manager.current_unit().await?, target, startup_deadline)
                .await?,
        ),
        _ => None,
    };
    let workload_intent = LaunchIntent::fresh_scope()?;
    let operation_intent = if selection.profile().ordinary() {
        Some(LaunchIntent::fresh_operation(
            placement.as_ref().map_or(slice.as_str(), |p| p.slice()),
            &controller,
        )?)
    } else {
        None
    };
    let intent = PayloadIntent {
        version: 1,
        controller: controller.clone(),
        workload: workload_intent.clone(),
        operation: operation_intent.clone(),
        subgroup: None,
    };
    directory.publish_evidence(InvocationEvidence::PayloadIntent, &intent)?;
    let guard = launch_lock(&directory, startup_deadline).await?;
    if guard.is_closed()? {
        return Err(HostError::Ownership(
            "managed invocation already closed launches".into(),
        ));
    }
    selection.apply_pending_environment(&mut command.environment, &workload_intent);
    command.environment.insert(
        "PSE_HOST_WORKLOAD_UNIT".into(),
        workload_intent.unit().into(),
    );
    command.environment.insert(
        "PSE_MANAGED_INVOCATION".into(),
        directory.id().as_str().into(),
    );
    command.environment.insert(
        super::supervisor::CHECKOUT.into(),
        super::request::encode_path(&root).into(),
    );
    command.environment.insert(
        "PSE_MANAGED_STATE_BASE_BYTES".into(),
        super::request::encode_path(
            &crate::paths::HostPaths::new(&command.environment)
                .base(crate::paths::UserDirectory::State)?,
        )
        .into(),
    );
    let coordinator = Coordinator::open(&root, &command.environment)?;
    let (mut child, mut gate_write, capture) = captured_child(
        &directory,
        &command,
        &trampoline,
        observer,
        policy.write,
        startup_deadline,
    )
    .await?;
    let mut workload = None;
    let mut operation = None;
    let mut permit: Option<Permit> = None;
    let mut cancelled = false;
    let startup = async {
        let child_pid = child
            .id()
            .ok_or_else(|| HostError::Ownership("gated child already reaped".into()))?;
        if !selection.profile().ordinary() {
            permit = Some(
                coordinator
                    .admit(
                        &manager,
                        &selection,
                        &controller,
                        &workload_intent,
                        child_pid,
                        startup_deadline,
                    )
                    .await?,
            );
            permit
                .as_mut()
                .ok_or_else(|| HostError::Ownership("finite payload reservation missing".into()))?
                .prepare(&manager, startup_deadline)
                .await?;
            permit
                .as_ref()
                .ok_or_else(|| HostError::Ownership("finite payload reservation missing".into()))?
                .bind_service_controller(&manager, &controller, startup_deadline)
                .await?;
        }
        if let Some(intent) = &operation_intent {
            operation = Some(
                manager
                    .start_service_operation_slice(intent, &controller, startup_deadline)
                    .await?,
            );
        }
        let selected_slice = permit
            .as_ref()
            .and_then(Permit::allocation_slice)
            .or_else(|| operation_intent.as_ref().map(LaunchIntent::unit))
            .unwrap_or(&slice);
        let controls = match &permit {
            Some(permit) => permit.profile().controls()?,
            None => ResourceControls::default(),
        };
        workload = Some(
            manager
                .start_scope_delegated(
                    &workload_intent,
                    pidfd(child_pid)?,
                    controller.unit(),
                    selected_slice,
                    startup_deadline,
                    policy.drain.saturating_sub(Duration::from_secs(1)),
                    &controls,
                )
                .await?,
        );
        let unit = workload
            .as_ref()
            .ok_or_else(|| HostError::Ownership("managed workload missing after launch".into()))?;
        unit.assert_population(&[child_pid]).await?;
        if let Some(permit) = &permit {
            permit.bind(unit, startup_deadline).await?;
        } else {
            coordinator
                .admit_ordinary(&manager, &unit.observe().await?, startup_deadline)
                .await?;
        }
        if let Some(placement) = &placement {
            placement.validate(&manager, startup_deadline).await?;
        }
        let group = unit.observe().await?;
        let parent_group = if selected_slice == slice {
            slice_group.clone()
        } else {
            manager.observe(selected_slice).await?.control_group
        };
        if PathBuf::from(format!("/sys/fs/cgroup{}", group.control_group)).parent()
            != Some(PathBuf::from(format!("/sys/fs/cgroup{parent_group}")).as_path())
        {
            return Err(HostError::Ownership(
                "managed workload escaped selected allocation".into(),
            ));
        }
        let child =
            match pse_diagnostics::process_facts::ProcessReader::default().process(child_pid) {
                pse_diagnostics::process_facts::ProcessRead::Present(facts) => facts.identity,
                _ => {
                    return Err(HostError::Ownership(
                        "gated original child identity unavailable before handoff".into(),
                    ));
                }
            };
        let operation_binding = match &operation {
            Some(operation) => Some(operation.binding().await?),
            None => None,
        };
        let reservation = match &permit {
            Some(permit) => permit.recovery().await?,
            None => None,
        };
        directory.publish_evidence(
            InvocationEvidence::Custody,
            &PayloadCustody {
                version: 2,
                manager: manager.identity_key(),
                workload: unit.binding().await?,
                child,
                operation: operation_binding,
                reservation,
                subgroup: None,
                creation: None,
            },
        )?;
        let notifier =
            super::readiness::SupervisorNotifier::authenticate(&manager, &controller).await?;
        let current = manager.managed_service_placement(&controller).await?.0;
        if current.invocation != original.invocation
            || current.control_group != original.control_group
            || current.main_pid != original.main_pid
        {
            return Err(HostError::Ownership(
                "original notify controller changed before payload release".into(),
            ));
        }
        if cancellation.is_cancelled() {
            cancelled = true;
            return Ok(());
        }
        notifier
            .send_ready("payload custody and capture initialized")
            .await?;
        gate_write.write_all(&[1])?;
        drop(guard);
        Ok(())
    };
    let execution = match tokio::time::timeout_at(startup_deadline.into(), startup).await {
        Ok(Ok(())) if cancelled => Ok(None),
        Ok(Ok(())) => tokio::select! {
            result = child.wait() => result.map(Some).map_err(HostError::from),
            () = cancellation.cancelled() => { cancelled = true; Ok(None) },
        },
        Ok(Err(error)) => Err(error),
        Err(_) => Err(HostError::Deadline(
            "absolute managed payload startup; original intents retained".into(),
        )),
    };
    drop(gate_write);
    let drain_deadline = deadline(policy.drain)?;
    let closure = match launch_lock(&directory, drain_deadline).await {
        Ok(guard) => guard.close(),
        Err(error) => Err(error),
    };
    let (workload_result, operation_result) = tokio::join!(
        settle_unit(
            &manager,
            Some(&workload_intent),
            &mut workload,
            drain_deadline
        ),
        settle_unit(
            &manager,
            operation_intent.as_ref(),
            &mut operation,
            drain_deadline
        )
    );
    let kill = match &execution {
        Ok(Some(_)) => Ok(()),
        _ => match child.try_wait() {
            Ok(Some(_)) => Ok(()),
            Ok(None) => child.start_kill().map_err(HostError::from),
            Err(error) => Err(error.into()),
        },
    };
    let waited = match &execution {
        Ok(Some(status)) => Ok(*status),
        _ => tokio::time::timeout_at(drain_deadline.into(), child.wait())
            .await
            .map_err(|_| HostError::Deadline("managed original child reap".into()))
            .and_then(|result| result.map_err(HostError::from)),
    };
    let borrowed = permit.as_ref().is_some_and(Permit::borrowed);
    let permit_result = match &mut permit {
        Some(permit) => permit.release(&manager, drain_deadline).await,
        None => Ok(true),
    };
    let capture = capture.finish(drain_deadline).await;
    let (child, operation_error) = match execution {
        Ok(Some(status)) => (Ok(status), None),
        Ok(None) => (waited, None),
        Err(error) => (waited, Some(error)),
    };
    Ok(PayloadReport {
        child,
        capture,
        operation_error,
        closure,
        workload: workload_result,
        operation: operation_result,
        kill,
        permit: permit_result,
        borrowed,
        cancelled,
        retained: RetainedResources {
            _workload: workload,
            _operation: operation,
            _permit: permit,
            _subgroup: None,
        },
        subgroup: None,
    })
}

#[cfg(test)]
mod tests;
