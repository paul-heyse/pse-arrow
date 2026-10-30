// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticated stop-post attribution. Service exit alone never settles payload resources.
use super::{
    result::{ManagedSummary, TerminalResult},
    supervisor,
};
use crate::{
    HostError,
    envelope::{CliInvocationId, ErrorRecord, RedactionPolicy},
    invocation::{InvocationDirectory, InvocationEvidence},
};
use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
use serde::Serialize;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Serialize)]
struct Finalization {
    version: u32,
    invocation: CliInvocationId,
    service: crate::UnitObservation,
    finalizer: pse_diagnostics::process_facts::ProcessIdentity,
    service_result: String,
    request_removed: Option<bool>,
    launch_closed: Option<bool>,
    payload_result_observed: bool,
    failures: Vec<ErrorRecord>,
}
async fn close(directory: &InvocationDirectory, deadline: Instant) -> Result<(), HostError> {
    loop {
        match directory.try_launch_lock() {
            Ok(guard) => return guard.close(),
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(HostError::Deadline(
                        "original finalizer launch exclusion".into(),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(error) => return Err(error),
        }
    }
}

/// Hidden stop-post entry. An existing payload result is immutable; this invocation supplies
/// a separately attributed service observation and an explicit unknown fallback when absent.
/// Original external-payload/permit recovery requires their protected original custody.
pub async fn finalize(root: PathBuf, id: CliInvocationId) -> Result<(), HostError> {
    let context = supervisor::context(&root, &id).await?;
    let service = context
        .manager
        .authenticate_service_finalizer(&context.record.controller)
        .await?;
    if let Some(controller) = context
        .directory
        .evidence::<supervisor::ControllerRecord>(InvocationEvidence::Controller)?
    {
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
                "original finalizer process identity is unavailable".into(),
            ));
        }
    };
    let service_result = std::env::var("SERVICE_RESULT")
        .map_err(|_| HostError::Ownership("original finalizer service result is absent".into()))?;
    let deadline = Instant::now()
        .checked_add(context.record.policy.drain)
        .ok_or_else(|| HostError::Ownership("finalizer observation horizon overflow".into()))?;
    let (cutoff, request) = tokio::join!(close(&context.directory, deadline), async {
        context.directory.discard_request()
    });
    let mut failures = Vec::new();
    if let Err(error) = cutoff {
        failures.push(error);
    }
    let request_removed = match request {
        Ok(value) => Some(value),
        Err(error) => {
            failures.push(error);
            None
        }
    };
    let launch_closed = match context.directory.launch_closed() {
        Ok(value) => Some(value),
        Err(error) => {
            failures.push(error);
            None
        }
    };
    let prior = match context
        .store
        .result::<Box<serde_json::value::RawValue>, ManagedSummary>(&id)
    {
        Ok(value) => value.is_some(),
        Err(error) => {
            failures.push(error);
            false
        }
    };
    let policy = RedactionPolicy::from_environment();
    context.directory.publish_evidence(
        InvocationEvidence::Finalization,
        &Finalization {
            version: 1,
            invocation: id.clone(),
            service,
            finalizer: process,
            service_result: service_result.clone(),
            request_removed,
            launch_closed,
            payload_result_observed: prior,
            failures: failures
                .iter()
                .map(|error| ErrorRecord::from_typed(error, &policy))
                .collect(),
        },
    )?;
    if !prior {
        failures.push(HostError::Ownership(
            "original supervisor payload settlement/capture evidence is unavailable; service exit does not establish it".into()));
        let terminal = TerminalResult::finalizer_unknown(
            &id,
            &context.record.command,
            service_result,
            failures,
            &policy,
        );
        match context.directory.publish_result(&terminal) {
            Ok(()) => {}
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Only a complete binding-checked original response establishes a competing winner.
                context
                    .store
                    .result::<Box<serde_json::value::RawValue>, ManagedSummary>(&id)?
                    .ok_or_else(|| {
                        HostError::Ownership(
                            "competing terminal publication is not readable original evidence"
                                .into(),
                        )
                    })?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
