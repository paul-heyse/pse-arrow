// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Stop-post attribution for the original service. It cannot reconstruct child/capture facts.
use super::{
    result::{ManagedSummary, TerminalResult},
    supervisor,
};
use crate::{
    HostError,
    envelope::{CliInvocationId, ErrorRecord, RedactionPolicy},
    invocation::InvocationEvidence,
};
use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
use serde::Serialize;
use std::path::PathBuf;

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
    failures: Vec<ErrorRecord>,
}
/// Authenticate the actual stop-post process, preserve an existing immutable operation report,
/// and otherwise publish explicit missing original evidence. Capacity belongs to later native
/// inactive/empty reconciliation; a still-running finalizer never releases it.
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
        Ok(value) => value.is_some(),
        Err(error) => {
            errors.push(error);
            false
        }
    };
    let request_removed = match context.directory.discard_request() {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error);
            None
        }
    };
    let finalized = context.directory.publish_evidence(
        InvocationEvidence::Finalization,
        &Finalization {
            version: 3,
            invocation: id.clone(),
            service,
            finalizer: process,
            service_result: service_result.clone(),
            request_removed,
            payload_result_observed: prior,
            native_lifetime_pending: true,
            failures: errors
                .iter()
                .map(|error| ErrorRecord::from_typed(error, &policy))
                .collect(),
        },
    );
    let finalization_failed = finalized.is_err();
    if let Err(error) = finalized {
        errors.push(error);
    }
    if prior {
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
