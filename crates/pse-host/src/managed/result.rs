// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Terminal admission from original performing-owner results, distinct from rendering.
use super::{payload::PayloadReport, streams::StreamReport};
use crate::{
    HostError,
    envelope::{
        CliInvocationId, CommandOutcome, Completeness, Envelope, ErrorRecord, RedactionPolicy,
        Settlement, SubjectState,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Generic raw-stream observation; it deliberately makes no compiler/test/scientific claim.
#[derive(Debug)]
pub(crate) struct GenericObserver;
impl super::streams::ChunkObserver for GenericObserver {
    type Error = HostError;
    fn observe(
        &mut self,
        _: crate::extraction::OutputStream,
        _: u64,
        _: &[u8],
    ) -> Result<(), HostError> {
        Ok(())
    }
}

/// One raw stream's retained coverage. Native errors remain attributed in the terminal failures.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct StreamSummary {
    /// Bytes actually drained, including bytes not retained after a required sink failure.
    pub bytes_read: u64,
    /// Confirmed accepted prefix; synchronization and unresolved I/O remain separate.
    pub bytes_retained: u64,
    /// Actual EOF from the joined reader.
    pub eof: bool,
    /// Every required raw byte and durability operation completed.
    pub complete: bool,
    /// Underlying output I/O may outlive its cancelled observation.
    pub io_unresolved: bool,
}
impl<W: super::streams::LogSink> From<&StreamReport<W>> for StreamSummary {
    fn from(value: &StreamReport<W>) -> Self {
        Self {
            bytes_read: value.pipe.bytes_read,
            bytes_retained: value.bytes_retained,
            eof: value.pipe.eof,
            complete: value.log_complete(),
            io_unresolved: value.io_unresolved,
        }
    }
}

/// Generic managed process/capture result; no scientific acceptance or test population is inferred.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ManagedSummary {
    /// Original payload child status only after an actual wait.
    pub child_exit: Option<i32>,
    /// Original stdout facts; absent for an uncertain finalizer.
    pub stdout: Option<StreamSummary>,
    /// Original stderr facts; absent for an uncertain finalizer.
    pub stderr: Option<StreamSummary>,
    /// This owner borrowed an enclosing reservation and never released it.
    pub borrowed_reservation: bool,
    /// Actual systemd finalizer result, diagnostic only.
    pub service_result: Option<String>,
}

/// Terminal publication capability. Deserializing an envelope cannot construct this type.
#[derive(Debug)]
pub struct TerminalResult<T> {
    response: Envelope<T>,
    _resources: Option<super::payload::RetainedResources>,
    _streams: Vec<tokio::fs::File>,
    _failures: Vec<HostError>,
}
impl<T> TerminalResult<T> {
    /// Immutable presentation source; projections never replace retained evidence.
    pub fn response(&self) -> &Envelope<T> {
        &self.response
    }
    pub(crate) fn evidence(&self) -> &Envelope<T> {
        &self.response
    }
}
impl TerminalResult<ManagedSummary> {
    pub(crate) fn from_generic(
        id: &CliInvocationId,
        command: &str,
        report: PayloadReport<GenericObserver>,
        policy: &RedactionPolicy,
    ) -> Self {
        let resources_settled = report.resources_settled();
        let capture_complete = report
            .capture
            .as_ref()
            .is_ok_and(|capture| capture.log_complete() && capture.observer_complete());
        let io_unresolved = report
            .capture
            .as_ref()
            .map_or(true, |capture| capture.io_unresolved());
        let child_exit = report.child_exit();
        let summary = ManagedSummary {
            child_exit,
            stdout: report
                .capture
                .as_ref()
                .ok()
                .map(|capture| (&capture.stdout).into()),
            stderr: report
                .capture
                .as_ref()
                .ok()
                .map(|capture| (&capture.stderr).into()),
            borrowed_reservation: report.borrowed,
            service_result: None,
        };
        let mut response = Envelope::response(command, "managed payload completed", summary);
        response.invocation_id = id.clone();
        response.child_exit = child_exit;
        response.settlement = if resources_settled && !io_unresolved {
            Settlement::Complete
        } else {
            Settlement::Incomplete
        };
        response.observation.scope = "managed_payload_owner".into();
        response.observation.limitations.push(
            "Service termination, finalizer completion and generation-use release have separate owners and remain unobserved here.".into());
        response.observation.required = true;
        response.observation.completeness = if capture_complete {
            Completeness::Complete
        } else {
            Completeness::Partial
        };
        let mut errors = Vec::new();
        errors.extend(report.operation_error);
        errors.extend(report.child.err());
        errors.extend(report.closure.err());
        errors.extend(report.workload.err());
        errors.extend(report.operation.err());
        errors.extend(report.kill.err());
        let permit_released = matches!(report.permit, Ok(true));
        let permit_retained = matches!(report.permit, Ok(false));
        errors.extend(report.permit.err());
        if permit_retained {
            errors.push(HostError::Ownership(
                "finite reservation retained after settlement attempts".into(),
            ));
        }
        let mut streams = Vec::new();
        match report.capture {
            Ok(capture) => {
                for mut stream in [capture.stdout, capture.stderr] {
                    for error in [
                        stream.pipe.read_error.take(),
                        stream.write_error.take(),
                        stream.flush_error.take(),
                        stream.synchronization_error.take(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        errors.push(HostError::Io(error));
                    }
                    if let Some(error) = stream.pipe.task_error.take() {
                        errors.push(HostError::Task(error));
                    }
                    streams.push(stream.sink);
                }
                errors.extend(capture.observer_errors.into_iter().flatten());
            }
            Err(error) => errors.push(error),
        }
        response.failures.extend(
            errors
                .iter()
                .map(|error| ErrorRecord::from_typed(error, policy)),
        );
        let succeeded = child_exit == Some(0)
            && resources_settled
            && capture_complete
            && !report.cancelled
            && response.failures.is_empty()
            && permit_released;
        response.outcome = if succeeded {
            CommandOutcome::Success
        } else {
            CommandOutcome::Failure
        };
        response.subject_state = if report.cancelled {
            SubjectState::Cancelled
        } else if succeeded {
            SubjectState::Succeeded
        } else {
            SubjectState::Failed
        };
        Self {
            response,
            _resources: Some(report.retained),
            _streams: streams,
            _failures: errors,
        }
    }

    pub(crate) fn with_owner_failures(
        mut self,
        errors: Vec<HostError>,
        policy: &RedactionPolicy,
    ) -> Self {
        if !errors.is_empty() {
            self.response.outcome = CommandOutcome::Failure;
            self.response.observation.completeness = Completeness::Partial;
            self.response.failures.extend(
                errors
                    .iter()
                    .map(|error| ErrorRecord::from_typed(error, policy)),
            );
            self._failures.extend(errors);
        }
        self
    }
    pub(crate) fn supervisor_unknown(
        id: &CliInvocationId,
        command: &str,
        errors: Vec<HostError>,
        policy: &RedactionPolicy,
    ) -> Self {
        let mut terminal = Self::finalizer_unknown(id, command, String::new(), errors, policy);
        terminal.response.payload.service_result = None;
        terminal.response.observation.scope = "managed_supervisor".into();
        terminal.response.summary =
            "supervisor owner ended without complete payload evidence".into();
        terminal
    }

    /// An authenticated finalizer reports its actual service outcome and explicit unknowns.
    /// Manager process exit cannot manufacture original child reaping or capture evidence.
    pub(crate) fn finalizer_unknown(
        id: &CliInvocationId,
        command: &str,
        service_result: String,
        errors: Vec<HostError>,
        policy: &RedactionPolicy,
    ) -> Self {
        let mut response = Envelope::response(
            command,
            "supervisor ended without complete owner settlement evidence",
            ManagedSummary {
                child_exit: None,
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: Some(service_result),
            },
        );
        response.invocation_id = id.clone();
        response.outcome = CommandOutcome::Failure;
        response.subject_state = SubjectState::Failed;
        response.settlement = Settlement::Unknown;
        response.observation.scope = "managed_finalizer".into();
        response.observation.required = true;
        response.observation.completeness = Completeness::Partial;
        response.failures.extend(
            errors
                .iter()
                .map(|error| ErrorRecord::from_typed(error, policy)),
        );
        Self {
            response,
            _resources: None,
            _streams: Vec::new(),
            _failures: errors,
        }
    }
}
