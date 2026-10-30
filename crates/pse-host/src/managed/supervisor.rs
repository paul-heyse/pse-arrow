// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared generic managed service protocol. Native readiness never grants scientific acceptance.
use super::{
    payload::{self, PayloadCommand, PayloadPolicy},
    request::{self, PrivateRequest},
    result::{ManagedSummary, TerminalResult},
};
use crate::{
    HostError, HostManager, LaunchIntent, OwnedUnit, ServiceFinalizer, ServiceSpec,
    UnitObservation,
    admission::Selection,
    envelope::{CliInvocationId, RedactionPolicy},
    environment::Environment,
    generation::{HostTool, PreparedGeneration},
    invocation::{InvocationDirectory, InvocationEvidence, InvocationStore},
    paths::{CheckoutState, HostPaths, UserDirectory},
};
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc, time::Instant};
use tokio_util::sync::CancellationToken;

const STATE_BASE: &str = "PSE_MANAGED_STATE_BASE_BYTES";
pub(super) const CHECKOUT: &str = "PSE_MANAGED_CHECKOUT_BYTES";
mod leader;
pub(crate) use leader::{PayloadLeader, current_payload};

/// Already resolved generic command inputs. Test/compiler workflow owners use their own
/// reconciliation routes; a successful generic command cannot establish their acceptance.
pub struct ManagedRequest {
    /// Physical checkout, independent of payload cwd.
    pub root: PathBuf,
    /// Original protected immutable selection directory.
    pub directory: Arc<InvocationDirectory>,
    /// Byte-exact selected process and environment.
    pub command: PayloadCommand,
    /// Captured admission selection.
    pub selection: Selection,
    /// Explicit caller observation policies.
    pub policy: PayloadPolicy,
}
impl std::fmt::Debug for ManagedRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedRequest")
            .field("invocation", self.directory.id())
            .field("command", &self.command)
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LaunchRecord {
    pub(super) version: u32,
    invocation: CliInvocationId,
    pub(super) command: String,
    pub(super) controller: LaunchIntent,
    control: PreparedGeneration,
    request_digest: String,
    pub(super) policy: PayloadPolicy,
}
impl LaunchRecord {
    fn verified_executable(&self, root: &std::path::Path) -> Result<PathBuf, HostError> {
        self.control.executable(root, HostTool::Host)
    }
    fn validate(
        &self,
        id: &CliInvocationId,
        metadata: &crate::invocation::InvocationMetadata<Box<serde_json::value::RawValue>>,
    ) -> Result<(), HostError> {
        if self.version != 3
            || self.invocation != *id
            || self.command != metadata.selection.command
            || self.control.generation != metadata.selection.control.generation
            || self.control.manifest_digest != metadata.selection.control.manifest_digest
            || self.control.protocol != metadata.selection.control.protocol
        {
            return Err(invalid("recorded launch differs from original metadata"));
        }
        Ok(())
    }
}

/// Actual original controller incarnation, published by that authenticated live main before
/// any private input consumption or payload effect. A decoded record alone grants no mutation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ControllerRecord {
    pub(super) version: u32,
    pub(super) manager: String,
    pub(super) binding: crate::manager::UnitBinding,
    pub(super) process: ProcessIdentity,
}

/// Actual matching native ready acknowledgement, never a terminal command result.
#[derive(Debug)]
pub struct Launched {
    /// Original CLI identity.
    pub invocation_id: CliInvocationId,
    /// Actual original native service observation.
    pub observation: UnitObservation,
    unit: OwnedUnit,
}
impl Launched {
    /// Wait for the exact original service and recursive cgroup to finish. A bounded timeout
    /// is an observation only; it neither stops work nor promotes a missing result to success.
    pub async fn wait(
        &self,
        deadline: Option<Instant>,
    ) -> Result<Option<UnitObservation>, HostError> {
        self.unit.wait(deadline).await
    }
    /// Same-launch-client cancellation. Later-process control requires separate ledger authority.
    pub async fn cancel(&mut self, deadline: Instant) -> Result<UnitObservation, HostError> {
        self.unit.stop_and_drain(deadline).await
    }
}

/// Native service observation only; neither variant establishes payload/scientific settlement.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RecordedObservation {
    /// The original manager reports the exact recorded incarnation.
    Observed {
        /// Uncached observation fenced to the recorded original service incarnation.
        service: UnitObservation,
    },
    /// The original manager no longer loads the unit and its original group is proved empty.
    Gone,
}

/// Control recovered only from checked invocation/launch/controller records and the fixed
/// prepared generation. It carries no launch-client reference or arbitrary-unit adoption API.
#[derive(Debug)]
pub struct RecordedInvocation {
    coordinator: crate::admission::Coordinator,
    manager: HostManager,
    controller: LaunchIntent,
    original: ControllerRecord,
    unit: Option<crate::manager::RecordedUnit>,
    completion: Option<RecordedObservation>,
}
impl RecordedInvocation {
    async fn resync(&mut self) -> Result<(), HostError> {
        self.unit = self
            .manager
            .recorded_original(
                &self.controller,
                &self.original.manager,
                &self.original.binding,
                Some(&self.original.process),
            )
            .await?;
        Ok(())
    }
    /// Wait on original unit/cgroup events. None means only this observation deadline expired.
    pub async fn wait(
        &mut self,
        deadline: Option<Instant>,
    ) -> Result<Option<RecordedObservation>, HostError> {
        let observed = self.wait_native(deadline).await?;
        if observed.is_some() {
            self.settle_capacity(
                deadline.unwrap_or_else(|| Instant::now() + std::time::Duration::from_secs(30)),
            )
            .await?;
        }
        Ok(observed)
    }
    async fn settle_capacity(&self, deadline: Instant) -> Result<(), HostError> {
        if !self
            .coordinator
            .settle_service(
                &self.manager,
                &self.controller,
                &self.original.binding,
                deadline,
            )
            .await?
        {
            return Err(invalid(
                "original native service drained but finite charge remains unreconciled",
            ));
        }
        Ok(())
    }
    async fn wait_native(
        &mut self,
        deadline: Option<Instant>,
    ) -> Result<Option<RecordedObservation>, HostError> {
        if let Some(completion) = &self.completion {
            return Ok(Some(completion.clone()));
        }
        let Some(unit) = &self.unit else {
            return Ok(Some(RecordedObservation::Gone));
        };
        match unit.wait(deadline).await {
            Ok(Some(service)) => {
                let completion = RecordedObservation::Observed { service };
                self.completion = Some(completion.clone());
                Ok(Some(completion))
            }
            Ok(None) => Ok(None),
            Err(error) => {
                if let Err(cleanup) = self.resync().await {
                    return Err(HostError::Settlement {
                        operation: Box::new(error),
                        cleanup: Box::new(cleanup),
                    });
                }
                if self.unit.is_none() {
                    Ok(Some(RecordedObservation::Gone))
                } else {
                    Err(error)
                }
            }
        }
    }
    /// Stop only the exact original service and prove its root empty; payload settlement
    /// remains with the original owner/finalizer evidence and is never inferred here.
    pub async fn cancel(&mut self, deadline: Instant) -> Result<RecordedObservation, HostError> {
        let observed = self.cancel_native(deadline).await?;
        self.settle_capacity(deadline).await?;
        Ok(observed)
    }
    async fn cancel_native(&mut self, deadline: Instant) -> Result<RecordedObservation, HostError> {
        if let Some(completion) = &self.completion {
            return Ok(completion.clone());
        }
        let Some(unit) = &mut self.unit else {
            return Ok(RecordedObservation::Gone);
        };
        match unit.stop_and_drain(deadline).await {
            Ok(Some(service)) => {
                let completion = RecordedObservation::Observed { service };
                self.completion = Some(completion.clone());
                Ok(completion)
            }
            Ok(None) => {
                self.completion = Some(RecordedObservation::Gone);
                Ok(RecordedObservation::Gone)
            }
            Err(error) => {
                if let Err(cleanup) = self.resync().await {
                    return Err(HostError::Settlement {
                        operation: Box::new(error),
                        cleanup: Box::new(cleanup),
                    });
                }
                if self.unit.is_none() {
                    Ok(RecordedObservation::Gone)
                } else {
                    Err(error)
                }
            }
        }
    }
}

/// Open passive/control custody for an existing invocation without current-source reads,
/// rebuilding, referencing the unit or accepting a caller-supplied executable path.
pub async fn recorded(
    root: &std::path::Path,
    id: &CliInvocationId,
    environment: &Environment,
) -> Result<RecordedInvocation, HostError> {
    let root = std::fs::canonicalize(root)?;
    let store = InvocationStore::new(CheckoutState::resolve(&root, environment)?);
    let directory = store.directory(id)?;
    let metadata: crate::invocation::InvocationMetadata<Box<serde_json::value::RawValue>> =
        store.metadata(id)?;
    let launch: LaunchRecord = directory
        .evidence(InvocationEvidence::Launch)?
        .ok_or_else(|| invalid("original launch record is absent"))?;
    launch.validate(id, &metadata)?;
    let original: ControllerRecord = directory
        .evidence(InvocationEvidence::Controller)?
        .ok_or_else(|| invalid("actual original controller record is absent"))?;
    if original.version != 1 || original.process.pid == 0 || original.binding.invocation.len() != 16
    {
        return Err(invalid("actual original controller record is invalid"));
    }
    launch.verified_executable(&root)?;
    let manager = HostManager::connect_user(None).await?;
    let unit = manager
        .recorded_original(
            &launch.controller,
            &original.manager,
            &original.binding,
            Some(&original.process),
        )
        .await?;
    if let Some(unit) = &unit {
        unit.observe().await?;
    }
    Ok(RecordedInvocation {
        coordinator: crate::admission::Coordinator::open(&root, environment)?,
        manager,
        controller: launch.controller,
        original,
        unit,
        completion: None,
    })
}

fn invalid(message: &str) -> HostError {
    HostError::Ownership(format!("managed service: {message}"))
}
fn current_identity() -> Result<ProcessIdentity, HostError> {
    match ProcessReader::default().process(std::process::id()) {
        ProcessRead::Present(facts) => Ok(facts.identity),
        _ => Err(invalid("current origin process identity is unavailable")),
    }
}

/// Launch one top-level detached native lifetime. Nested commands inherit foreground ownership.
pub async fn launch_generic(request: ManagedRequest) -> Result<Launched, HostError> {
    let ManagedRequest {
        root,
        directory,
        command,
        selection,
        policy,
    } = request;
    let root = std::fs::canonicalize(root)?;
    let state = CheckoutState::resolve(&root, &command.environment)?;
    if directory.path() != state.invocations().join(directory.id().as_str()) {
        return Err(invalid(
            "selected state environment differs from original invocation directory",
        ));
    }
    let store = InvocationStore::new(state);
    let metadata: crate::invocation::InvocationMetadata<Box<serde_json::value::RawValue>> =
        store.metadata(directory.id())?;
    let control = metadata.selection.control;
    let startup = Instant::now()
        .checked_add(policy.startup)
        .ok_or_else(|| invalid("startup horizon exceeds clock range"))?;
    let manager = HostManager::connect_user(None).await?;
    let membership = manager.current_membership().await?;
    if manager.native_workload(membership.root()).await? {
        return Err(invalid(
            "independent detachment from an enclosing native workload is unsupported; use inherited foreground ownership",
        ));
    }
    let (source_slice, _, tasks) = manager.foreground_placement(membership.root()).await?;
    crate::foreground::preflight_service(&membership.root().control_group, tasks)?;
    let placement = match selection.requested_ordinary_target() {
        Some(target) => Some(
            manager
                .ordinary_placement(membership.root(), target, startup)
                .await?,
        ),
        None => None,
    };
    let base = HostPaths::new(&command.environment).base(UserDirectory::State)?;
    if !base.is_absolute() {
        return Err(invalid("managed state base must be absolute"));
    }
    let controller = LaunchIntent::fresh()?;
    let mut command = command;
    selection.apply_pending_environment(&mut command.environment, &controller);
    let coordinator = crate::admission::Coordinator::open(&root, &command.environment)?;
    let controls = selection.profile().controls()?;
    let stop = policy
        .drain
        .checked_add(std::time::Duration::from_secs(5))
        .ok_or_else(|| invalid("managed stop horizon overflow"))?;
    let private = PrivateRequest::new(command, selection.clone(), policy)?;
    let executable = control.executable(&root, HostTool::Host)?;
    let record = LaunchRecord {
        version: 3,
        invocation: directory.id().clone(),
        command: metadata.selection.command,
        controller: controller.clone(),
        control,
        request_digest: private.digest()?,
        policy,
    };
    let checkout = request::encode_path(&root);
    let common = vec![
        "--checkout-bytes".into(),
        checkout,
        "--invocation".into(),
        directory.id().to_string(),
    ];
    let mut arguments = vec!["__supervise".into()];
    arguments.extend(common.clone());
    let mut finalizer_arguments = vec!["__finalize".into()];
    finalizer_arguments.extend(common);
    let mut environment = vec![format!("{STATE_BASE}={}", request::encode_path(&base))];
    if let Ok(address) = std::env::var("DBUS_SESSION_BUS_ADDRESS") {
        environment.push(format!("DBUS_SESSION_BUS_ADDRESS={address}"));
    }
    let spec = ServiceSpec {
        intent: controller,
        executable: executable.clone(),
        arguments,
        environment,
    };
    let finalizer = ServiceFinalizer {
        executable,
        arguments: finalizer_arguments,
    };
    let mut permit = if selection.profile().ordinary() {
        None
    } else {
        Some(
            coordinator
                .admit_service(&manager, &selection, &spec.intent, startup)
                .await?,
        )
    };
    let mut unit = None;
    let launched = async {
        if let Some(permit) = &mut permit { permit.prepare(&manager, startup).await?; }
        let service_policy = crate::manager::ManagedServicePolicy {
            slice: permit.as_ref().and_then(crate::admission::Permit::allocation_slice)
                .map(str::to_owned).unwrap_or_else(|| placement.as_ref().map_or(source_slice.as_str(), |p| p.slice()).to_owned()),
            controls, startup: policy.startup, stop,
        };
        directory.publish_evidence(InvocationEvidence::Launch, &record)?;
        directory.publish_request(&private)?;
        if let Some(placement) = &placement { placement.validate(&manager, startup).await?; }
        unit = Some(manager.start_managed_service(&spec, &finalizer, &service_policy, startup).await?);
        let unit = unit.as_ref().ok_or_else(|| invalid("original service custody is missing"))?;
        if unit.launch_job_result() != Some("done") {
            return Err(invalid("original notify job did not acknowledge readiness; inspect retained invocation evidence"));
        }
        unit.observe().await
    }.await;
    match launched {
        Ok(observation) => Ok(Launched {
            invocation_id: directory.id().clone(),
            observation,
            unit: unit
                .ok_or_else(|| invalid("original acknowledged service custody is missing"))?,
        }),
        Err(operation) => {
            let deadline = Instant::now()
                .checked_add(policy.drain)
                .ok_or_else(|| invalid("startup cleanup observation horizon overflow"))?;
            let resources = async {
                if unit.is_none() {
                    unit = manager.reconcile_if_present(&spec.intent, deadline).await?;
                }
                if let Some(unit) = &mut unit {
                    unit.stop_and_drain(deadline).await?;
                }
                Ok::<_, HostError>(())
            }
            .await;
            let allocation = match &mut permit {
                Some(permit) => permit.release(&manager, deadline).await.and_then(|released| if released { Ok(()) } else { Err(invalid("finite service startup charge retained after original cleanup attempts")) }),
                None => Ok(()),
            };
            let request = directory.discard_request().map(|_| ());
            let mut failure = operation;
            for cleanup in [resources, allocation, request] {
                if let Err(cleanup) = cleanup {
                    failure = HostError::Settlement {
                        operation: Box::new(failure),
                        cleanup: Box::new(cleanup),
                    };
                }
            }
            Err(failure)
        }
    }
}

pub(super) struct Context {
    pub(super) directory: Arc<InvocationDirectory>,
    pub(super) record: LaunchRecord,
    pub(super) manager: HostManager,
    pub(super) store: InvocationStore,
}
pub(super) async fn context(
    root: &std::path::Path,
    id: &CliInvocationId,
) -> Result<Context, HostError> {
    let mut environment: Environment = std::env::vars_os().collect();
    let base =
        std::env::var(STATE_BASE).map_err(|_| invalid("protected state selector is absent"))?;
    environment.insert(
        "XDG_STATE_HOME".into(),
        request::decode_path(&base)?.into_os_string(),
    );
    let store = InvocationStore::new(CheckoutState::resolve(root, &environment)?);
    let directory = Arc::new(store.directory(id)?);
    let record: LaunchRecord = directory
        .evidence(InvocationEvidence::Launch)?
        .ok_or_else(|| invalid("original launch evidence is absent"))?;
    let metadata: crate::invocation::InvocationMetadata<Box<serde_json::value::RawValue>> =
        store.metadata(id)?;
    record.validate(id, &metadata)?;
    if std::env::current_exe()? != record.control.executable(root, HostTool::Host)? {
        return Err(invalid(
            "internal entry is not the recorded prepared control executable",
        ));
    }
    Ok(Context {
        directory,
        record,
        store,
        manager: HostManager::connect_user(None).await?,
    })
}

/// Actual hidden service main. Parsing/entry policy belongs to the CLI; this path never builds.
pub async fn supervise(root: PathBuf, id: CliInvocationId) -> Result<i32, HostError> {
    let context = context(&root, &id).await?;
    context
        .manager
        .authenticate_service_controller(&context.record.controller)
        .await?;
    let controller = ControllerRecord {
        version: 1,
        manager: context.manager.identity_key(),
        binding: context
            .manager
            .service_controller_binding(&context.record.controller)
            .await?,
        process: current_identity()?,
    };
    context
        .directory
        .publish_evidence(InvocationEvidence::Controller, &controller)?;
    let terminal = match supervise_owned(&context, root, &id).await {
        Ok(terminal) => terminal,
        Err(error) => {
            // This is the authenticated original main, so preserve its actual initialization
            // failure before native exit. Stop-post cannot reconstruct the original cause.
            TerminalResult::supervisor_unknown(
                &id,
                &context.record.command,
                vec![error],
                &RedactionPolicy::from_environment(),
            )
        }
    };
    if let Err(error) = context
        .directory
        .publish_evidence(InvocationEvidence::Settlement, terminal.response())
    {
        return Err(terminal.publication_failed(error));
    }
    if let Err(error) = context.directory.publish_result(&terminal) {
        return Err(terminal.publication_failed(error));
    }
    Ok(terminal.service_exit_code())
}
async fn supervise_owned(
    context: &Context,
    root: PathBuf,
    id: &CliInvocationId,
) -> Result<TerminalResult<ManagedSummary>, HostError> {
    let private: PrivateRequest = context.directory.take_request()?;
    if private.digest()? != context.record.request_digest {
        return Err(invalid("private input differs from captured launch"));
    }
    let (command, selection, policy) = private.into_parts()?;
    let observer = super::workflow::Observer::for_payload(context, &root, &command)?;
    let redaction = RedactionPolicy::from_inputs(&command.environment);
    if policy.startup != context.record.policy.startup
        || policy.drain != context.record.policy.drain
        || policy.write != context.record.policy.write
    {
        return Err(invalid("private input policy differs from original launch"));
    }
    let cancellation = CancellationToken::new();
    let finished = CancellationToken::new();
    let signal_cancel = cancellation.clone();
    let signal_finished = finished.clone();
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let signals = tokio::spawn(async move {
        tokio::select! {
            _ = interrupt.recv() => signal_cancel.cancel(),
            _ = terminate.recv() => signal_cancel.cancel(),
            () = signal_finished.cancelled() => {},
        }
    });
    let trampoline = context.record.control.executable(&root, HostTool::Host)?;
    let report = super::service_payload::run(payload::PayloadRequest {
        directory: context.directory.clone(),
        controller: context.record.controller.clone(),
        root,
        command,
        selection,
        trampoline,
        policy,
        cancellation,
        observer,
    })
    .await;
    finished.cancel();
    let mut failures = Vec::new();
    if let Err(error) = signals.await {
        failures.push(HostError::Task(error));
    }
    let policy = redaction;
    let terminal: TerminalResult<ManagedSummary> = match report {
        Ok(mut report) => {
            let child_exit = report.child_exit();
            let validation = match &mut report.capture {
                Ok(capture) => match capture.observer.take() {
                    Some(observer) => match observer.finish() {
                        Ok(Some(terminal)) => super::workflow::validate_report(
                            context.directory.clone(),
                            terminal,
                            child_exit,
                        )
                        .await
                        .map(Some),
                        Ok(None) => Ok(None),
                        Err(error) => Err(error),
                    },
                    None => Err(invalid("original required interpreter is unavailable")),
                },
                Err(_)
                    if crate::test_workflow::execute::Action::from_command(
                        &context.record.command,
                    )
                    .is_some() =>
                {
                    Err(invalid("original required workflow capture is unavailable"))
                }
                Err(_) => Ok(None),
            };
            TerminalResult::from_payload(id, &context.record.command, report, &policy)
                .with_workflow(validation, &policy)
                .with_owner_failures(failures, &policy)
        }
        Err(error) => {
            failures.push(error);
            TerminalResult::supervisor_unknown(id, &context.record.command, failures, &policy)
        }
    }
    .with_bundle(&context.directory, &policy);
    Ok(terminal)
}

#[cfg(test)]
pub(super) mod tests;
