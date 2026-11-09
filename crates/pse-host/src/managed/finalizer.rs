// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Stop-post attribution for the original service. It cannot reconstruct child/capture facts.
use super::{
    result::{ManagedSummary, TerminalResult},
    supervisor, workflow,
};
use crate::{
    HostError,
    envelope::{CliInvocationId, ErrorRecord, RedactionPolicy},
    invocation::{DeferredTestDrain, InvocationEvidence},
};
use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
use serde::Serialize;
use std::{
    io::Write as _,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Serialize)]
struct Finalization {
    version: u32,
    invocation: CliInvocationId,
    service: crate::UnitObservation,
    finalizer: pse_diagnostics::process_facts::ProcessIdentity,
    service_result: String,
    request_removed: Option<bool>,
    payload_result_observed: bool,
    native_lifetime_pending: bool,
    workflow_images_released: Option<bool>,
    workflow_release_failure: Option<ErrorRecord>,
    borrowed_release_failure: Option<ErrorRecord>,
    failures: Vec<ErrorRecord>,
}
/// Authenticate the actual stop-post process, preserve an existing immutable operation report,
/// and otherwise publish explicit missing original evidence. Capacity belongs to later native
/// inactive/empty reconciliation. This finalizer can retire private executable
/// images only after independently proving original worker and fixture drain.
pub async fn finalize(root: PathBuf, id: CliInvocationId) -> Result<(), HostError> {
    let context = supervisor::context(&root, &id).await?;
    let service = context
        .manager
        .authenticate_service_finalizer(&context.record.controller)
        .await?;
    let controller = context
        .directory
        .evidence::<supervisor::ControllerRecord>(InvocationEvidence::Controller)?;
    if let Some(controller) = &controller {
        if controller.version != 1
            || controller.manager != context.manager.identity_key()
            || controller.binding.invocation != service.invocation
            || controller.binding.group != service.control_group
        {
            return Err(HostError::Ownership(
                "finalizer differs from original recorded controller incarnation".into(),
            ));
        }
        context
            .manager
            .validate_binding(&context.record.controller, &controller.binding)
            .await?;
    }
    let process = match ProcessReader::default().process(std::process::id()) {
        ProcessRead::Present(facts) => facts.identity,
        _ => {
            return Err(HostError::Ownership(
                "original finalizer identity unavailable".into(),
            ));
        }
    };
    let service_result = std::env::var("SERVICE_RESULT")
        .map_err(|_| HostError::Ownership("original service result absent".into()))?;
    let policy = RedactionPolicy::from_environment();
    let mut errors = Vec::new();
    let prior = match context
        .store
        .result::<serde::de::IgnoredAny, ManagedSummary>(&id)
    {
        Ok(value) => value,
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let request_removed = match context.directory.discard_request() {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let mut workflow_images_released = None;
    let mut workflow_release_failure = None;
    let mut released_workflow = None;
    let original_workflow = prior
        .as_ref()
        .filter(|result| result.payload.workflow.is_some());
    let pending = pending_borrowed_release(controller.as_ref());
    let mut borrowed_release_failure = None;
    let borrowed_runtime = match pending {
        Ok(value) => value,
        Err(error) => {
            borrowed_release_failure = Some(ErrorRecord::from_typed(&error, &policy));
            None
        }
    };
    if original_workflow.is_some() || borrowed_runtime.is_some() {
        let checked = require_worker_drain(&context, controller.as_ref(), &process).await;
        if let Some(result) = original_workflow {
            match &checked {
                Ok(_) => match workflow::release_detached_images(
                    &context.store,
                    &context.directory,
                    result,
                ) {
                    Ok(released) => {
                        workflow_images_released = Some(released.is_some());
                        released_workflow = released;
                    }
                    Err(error) => {
                        workflow_images_released = Some(false);
                        workflow_release_failure = Some(ErrorRecord::from_typed(&error, &policy));
                    }
                },
                Err(error) => {
                    workflow_images_released = Some(false);
                    workflow_release_failure = Some(ErrorRecord::from_typed(error, &policy));
                }
            }
        }
        if let Some(runtime) = borrowed_runtime {
            let release =
                checked.and_then(|controller| release_borrowed(&context, controller, &runtime));
            if let Err(error) = release {
                borrowed_release_failure = Some(ErrorRecord::from_typed(&error, &policy));
            }
        }
    }
    if context.directory.is_conformance()
        && let Err(error) = async {
            require_worker_drain(&context, controller.as_ref(), &process).await?;
            context.directory.record_conformance_worker_drain()
        }.await
    {
        errors.push(error);
    }
    let finalized = context.directory.publish_evidence(
        InvocationEvidence::Finalization,
        &Finalization {
            version: 4,
            invocation: id.clone(),
            service,
            finalizer: process,
            service_result: service_result.clone(),
            request_removed,
            payload_result_observed: prior.is_some(),
            native_lifetime_pending: true,
            workflow_images_released,
            workflow_release_failure,
            borrowed_release_failure,
            failures: errors
                .iter()
                .map(|error| ErrorRecord::from_typed(error, &policy))
                .collect(),
        },
    );
    let finalization_failed = finalized.is_err();
    if let Err(error) = finalized {
        errors.push(error);
    } else if let Some(released) = released_workflow
        && let Err(error) = released.publish_compact(
            &context.store,
            &context.directory,
            DeferredTestDrain::Detached,
            &policy,
        )
    {
        let _ = writeln!(
            std::io::stderr().lock(),
            "pse-host: settled compact result remains pending: {}",
            policy.redact(&error.to_string())
        );
    }
    if prior.is_some() {
        return errors
            .into_iter()
            .reduce(|operation, cleanup| HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            })
            .map_or(Ok(()), Err);
    }
    errors.push(HostError::Ownership("original child reap or required capture report unavailable; native termination does not reconstruct it".into()));
    let terminal = TerminalResult::finalizer_unknown(
        &id,
        &context.record.command,
        service_result,
        errors,
        &policy,
    );
    if let Err(error) = context.directory.publish_result(&terminal) {
        return Err(terminal.publication_failed(error));
    }
    if finalization_failed {
        return Err(terminal.publication_failed(HostError::Ownership(
            "required finalization publication failed".into(),
        )));
    }
    Ok(())
}

fn pending_borrowed_release(
    controller: Option<&supervisor::ControllerRecord>,
) -> Result<Option<PathBuf>, HostError> {
    let Some(controller) = controller else {
        return Ok(None);
    };
    let Ok(value) = std::env::var(supervisor::RUNTIME_BASE) else {
        return Ok(None);
    };
    let runtime = super::request::decode_path(&value)?;
    Ok(
        workflow::borrowed::enclosing_pending(&runtime, &controller.manager, &controller.binding)?
            .then_some(runtime),
    )
}

fn release_borrowed(
    context: &supervisor::Context,
    controller: &supervisor::ControllerRecord,
    runtime: &Path,
) -> Result<(), HostError> {
    let deadline = Instant::now()
        .checked_add(context.record.policy.drain)
        .ok_or_else(|| HostError::Ownership("borrowed release horizon overflow".into()))?;
    workflow::borrowed::reconcile_enclosing(
        runtime,
        &controller.manager,
        context.record.controller.unit(),
        &controller.binding,
        deadline,
    )
}

async fn require_worker_drain<'a>(
    context: &supervisor::Context,
    controller: Option<&'a supervisor::ControllerRecord>,
    finalizer: &pse_diagnostics::process_facts::ProcessIdentity,
) -> Result<&'a supervisor::ControllerRecord, HostError> {
    let controller = controller.ok_or_else(|| {
        HostError::Ownership("original controller record unavailable for image release".into())
    })?;
    let child = context
        .directory
        .evidence::<super::service_payload::ServiceCustody>(InvocationEvidence::Custody)?
        .ok_or_else(|| HostError::Ownership("original worker custody is absent".into()))?;
    let reader = ProcessReader::default();
    let departed = |original: &pse_diagnostics::process_facts::ProcessIdentity| {
        if original.boot_id != finalizer.boot_id
            || original.pid_namespace != finalizer.pid_namespace
            || original.pid == 0
            || original.start_ticks == 0
        {
            return false;
        }
        match reader.process(original.pid) {
            ProcessRead::Present(current) => current.identity != *original,
            ProcessRead::Absent => true,
            ProcessRead::Changed | ProcessRead::Unavailable(_) => false,
        }
    };
    if child.version != 5
        || child.manager != controller.manager
        || child.service != controller.binding
        || !departed(&child.child)
        || !departed(&controller.process)
        || !context
            .manager
            .finalizer_alone(&context.record.controller, &controller.binding, finalizer)
            .await?
    {
        return Err(HostError::Ownership(
            "original finalizer cannot establish worker drain for image release".into(),
        ));
    }
    Ok(controller)
}
