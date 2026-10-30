// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Existing invocation control never re-admits work or rebuilds current sources.
use super::{HostError, OutputOptions, checkout_root, runtime};
use crate::{
    envelope::{self, CliInvocationId, Envelope},
    generation::EntrySettlement,
    managed::supervisor::{self, RecordedObservation},
};
use clap::Args;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[derive(Debug, Args)]
pub(super) struct Selection {
    /// Exact invocation ID, a unique prefix, or latest within this checkout/session.
    invocation: String,
    #[arg(long)]
    session: Option<String>,
    /// Observation/stop deadline in seconds; expiration never implies subject completion.
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct ControlView {
    /// Original selected CLI identity, independent of this observing invocation.
    original_invocation: CliInvocationId,
    /// Exact uncached original native service observation, when still loaded.
    service: Option<crate::UnitObservation>,
    /// Original manager no longer loads the service and its original group is empty/absent.
    service_gone: bool,
    /// Original immutable owner result, retaining scientific and resource outcome separation.
    #[schemars(with = "Option<Envelope<serde_json::Value>>")]
    result: Option<Envelope<Box<serde_json::value::RawValue>>>,
}
pub(super) fn dispatch(
    args: Selection,
    cancel: bool,
    selected: Option<&Path>,
    output: &OutputOptions,
    settlement: &mut EntrySettlement,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let store = super::invocation_store(selected)?;
    let id = store
        .resolve(&args.invocation, args.session.as_deref())?
        .ok_or_else(|| HostError::Ownership("no matching original invocation".into()))?;
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(args.timeout))
        .ok_or_else(|| {
            HostError::Ownership("control deadline exceeds supported monotonic horizon".into())
        })?;
    *settlement = EntrySettlement::Unresolved;
    let environment = std::env::vars_os().collect();
    let (observed, interrupted) = runtime()?.block_on(async {
        let mut original = supervisor::recorded(&root, &id, &environment).await?;
        if cancel {
            return original
                .cancel(deadline)
                .await
                .map(|state| (Some(state), false));
        }
        let mut interrupt =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = original.wait(Some(deadline)) => result.map(|state|(state, false)),
            _ = interrupt.recv() => Ok((None, true)),
            _ = terminate.recv() => Ok((None, true)),
        }
    })?;
    // This observer executed no protected image and delegated no descendants. The original
    // performing invocation retains its own independent resources and generation claims.
    *settlement = EntrySettlement::ResourcesSettled;
    let (service, service_gone) = match observed.as_ref() {
        Some(RecordedObservation::Observed { service }) => (Some(service.clone()), false),
        Some(RecordedObservation::Gone) => (None, true),
        None => (None, false),
    };
    let result = store.result::<serde::de::IgnoredAny, Box<serde_json::value::RawValue>>(&id)?;
    let command = if cancel { "cancel" } else { "wait" };
    let mut response = Envelope::response(
        command,
        if interrupted {
            "observation interrupted; subject was not cancelled"
        } else if observed.is_none() {
            "observation deadline ended; completion remains unobserved"
        } else {
            "original native service settled; owner evidence inspected separately"
        },
        ControlView {
            original_invocation: id,
            service,
            service_gone,
            result,
        },
    );
    response.observation.scope = "original_native_service_and_retained_owner_result".into();
    response.observation.limitations.push("Native service drain does not establish payload, reservation or generation settlement; the original owner result retains those facts.".into());
    if interrupted {
        response.outcome = envelope::CommandOutcome::Interrupted;
        response.subject_state = envelope::SubjectState::Unknown;
        output.emit(response)?;
        return Ok(130);
    }
    if observed.is_none() {
        response.subject_state = envelope::SubjectState::Unknown;
        response.observation.timed_out = true;
        response.observation.required = false;
    } else if response.payload.result.is_none() {
        response.subject_state = envelope::SubjectState::Unknown;
        response.outcome = envelope::CommandOutcome::BoundaryFailure;
    }
    // A wait/cancel response concerns the observer/control operation. Original child status
    // remains nested and is never reinterpreted as a new scientific verdict.
    Ok(output.emit(response)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn control_selects_existing_work_and_never_new_work() {
        use clap::Parser;
        let cli = super::super::Cli::try_parse_from([
            "pse",
            "wait",
            "latest",
            "--session",
            "analysis",
            "--timeout",
            "0",
        ])
        .unwrap();
        assert!(matches!(
            cli.command.unwrap().generation_intent(),
            super::super::GenerationIntent::ExistingControl
        ));
        let cli = super::super::Cli::try_parse_from(["pse", "cancel", "01a1", "--timeout", "60"])
            .unwrap();
        assert!(matches!(
            cli.command.unwrap().generation_intent(),
            super::super::GenerationIntent::ExistingControl
        ));
    }
}
