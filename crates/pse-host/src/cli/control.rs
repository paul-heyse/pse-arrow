// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Existing invocation control never re-admits work or rebuilds current sources.
use super::{HostError, OutputOptions, checkout_root, runtime};
use crate::{
    envelope::{self, CliInvocationId, Envelope},
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
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let store = super::invocation_store(selected)?;
    let environment = std::env::vars_os().collect();
    let session = crate::invocation::select_session(args.session, &environment)?;
    let id = store
        .resolve(&args.invocation, session.as_deref())?
        .ok_or_else(|| HostError::Ownership("no matching original invocation".into()))?;
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(args.timeout))
        .ok_or_else(|| {
            HostError::Ownership("control deadline exceeds supported monotonic horizon".into())
        })?;

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
    response.observation.limitations.push("Original child and capture evidence remains immutable; this view separately reconciles authenticated native service drain and capacity.".into());
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
    } else {
        project_terminal_owner(&mut response, cancel);
    }
    Ok(output.emit(response)?)
}

fn project_terminal_owner(response: &mut Envelope<ControlView>, cancel: bool) {
    if let Some(original) = &response.payload.result {
        // A native stop acknowledgement cannot promote missing original reconciliation.
        response.settlement = original.settlement;
        response.observation.completeness = original.observation.completeness;
        response.failures.extend(original.failures.iter().cloned());
        // This call follows successful recorded wait/cancel, including actual whole-unit
        // drain and finite-capacity reconciliation. It composes those facts with original
        // complete operation evidence without rewriting that earlier immutable report.
        let native_pending =
            serde_json::from_str::<crate::managed::result::ManagedSummary>(original.payload.get())
                .is_ok_and(|summary| {
                    summary.native_lifetime_pending
                        && summary.child_exit.is_some()
                        && summary
                            .stdout
                            .as_ref()
                            .is_some_and(|stream| stream.complete && !stream.io_unresolved)
                        && summary
                            .stderr
                            .as_ref()
                            .is_some_and(|stream| stream.complete && !stream.io_unresolved)
                });
        if native_pending && original.observation.completeness == envelope::Completeness::Complete {
            response.settlement = envelope::Settlement::Complete;
        }
        if !cancel {
            // Finished wait returns the original managed verdict, rather than success merely
            // because the observing command read a valid response. Exact evidence stays nested.
            response.outcome = original.outcome;
            response.subject_state = original.subject_state;
            response.child_exit = original.child_exit;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn complete_native_wait_composes_pending_child_capture_without_rewriting_original() {
        use super::*;
        use crate::managed::result::{ManagedSummary, StreamSummary};
        let stream = StreamSummary {
            bytes_read: 3,
            bytes_retained: 3,
            eof: true,
            complete: true,
            io_unresolved: false,
        };
        let summary = ManagedSummary {
            bundle: None,
            native_lifetime_pending: true,
            workflow: None,
            child_exit: Some(0),
            stdout: Some(stream.clone()),
            stderr: Some(stream),
            borrowed_reservation: false,
            service_result: None,
        };
        let mut original = Envelope::response(
            "run",
            "original child and capture",
            serde_json::value::to_raw_value(&summary).unwrap(),
        );
        original.child_exit = Some(0);
        original.subject_state = envelope::SubjectState::Succeeded;
        original.settlement = envelope::Settlement::Unknown;
        assert_eq!(
            original.exit_code(),
            1,
            "live wrapper cannot report whole-lifetime success"
        );
        let mut response = Envelope::response(
            "wait",
            "actual original native drain observed",
            ControlView {
                original_invocation: original.invocation_id.clone(),
                service: None,
                service_gone: true,
                result: Some(original),
            },
        );
        project_terminal_owner(&mut response, false);
        assert_eq!(response.exit_code(), 0);
        assert_eq!(response.settlement, envelope::Settlement::Complete);
        assert_eq!(
            response.payload.result.as_ref().unwrap().settlement,
            envelope::Settlement::Unknown
        );
        response
            .payload
            .result
            .as_mut()
            .unwrap()
            .observation
            .completeness = envelope::Completeness::Partial;
        project_terminal_owner(&mut response, false);
        assert_eq!(
            response.exit_code(),
            1,
            "native drain cannot replace missing required capture"
        );
    }
    #[test]
    fn finished_wait_preserves_managed_failure_while_cancel_requires_reconciliation() {
        use super::*;
        use serde_json::value::RawValue;
        let mut original = Envelope::response(
            "run",
            "original failure",
            RawValue::from_string("{}".into()).unwrap(),
        );
        original.outcome = envelope::CommandOutcome::Failure;
        original.subject_state = envelope::SubjectState::Failed;
        original.child_exit = Some(3);
        original.settlement = envelope::Settlement::Complete;
        let mut response = Envelope::response(
            "wait",
            "observed",
            ControlView {
                original_invocation: original.invocation_id.clone(),
                service: None,
                service_gone: true,
                result: Some(original),
            },
        );
        project_terminal_owner(&mut response, false);
        assert_eq!(response.exit_code(), 1);
        assert_eq!(response.child_exit, Some(3));
        assert_eq!(response.subject_state, envelope::SubjectState::Failed);
        response.outcome = envelope::CommandOutcome::Success;
        response.subject_state = envelope::SubjectState::NotApplicable;
        response.child_exit = None;
        project_terminal_owner(&mut response, true);
        assert_eq!(response.exit_code(), 0);
        response.payload.result.as_mut().unwrap().settlement = envelope::Settlement::Unknown;
        project_terminal_owner(&mut response, true);
        assert_eq!(response.exit_code(), 1);
    }
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
