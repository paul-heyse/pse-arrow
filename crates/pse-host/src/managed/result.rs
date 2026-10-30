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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_keeps_original_exit_and_unknown_evidence_through_bundle_and_workflow_augmentation()
     {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let mut environment: crate::environment::Environment = std::env::vars_os().collect();
        environment.insert(
            "XDG_STATE_HOME".into(),
            scratch.path().as_os_str().to_owned(),
        );
        let store = crate::invocation::InvocationStore::new(
            crate::paths::CheckoutState::resolve(root, &environment).unwrap(),
        );
        let id = CliInvocationId::new();
        let directory = store
            .begin(crate::invocation::InvocationSpec {
                invocation_id: id.clone(),
                command: "run-all".into(),
                session: None,
                parent: None,
                operation: (),
                control: crate::generation::prepared(root).unwrap().unwrap(),
                environment_digest: "test".into(),
                scientific: Vec::new(),
            })
            .unwrap();
        let policy = RedactionPolicy::from_inputs(&environment);
        let mut terminal =
            TerminalResult::finalizer_unknown(&id, "run-all", String::new(), Vec::new(), &policy);
        terminal.response.outcome = CommandOutcome::Interrupted;
        terminal.response.subject_state = SubjectState::Cancelled;
        let terminal = terminal.with_bundle(&directory, &policy);
        assert_eq!(terminal.response.exit_code(), 130);
        assert_eq!(terminal.response.subject_state, SubjectState::Cancelled);
        assert_eq!(
            terminal.response.observation.completeness,
            Completeness::Partial
        );
        assert_eq!(terminal.response.settlement, Settlement::Unknown);
        assert!(terminal.response.payload.bundle.is_none());
        assert_eq!(terminal.response.failures.len(), 1);
        let terminal = terminal.with_workflow(
            Err(HostError::Ownership(
                "cancelled helper did not publish a terminal".into(),
            )),
            &policy,
        );
        assert_eq!(terminal.response.exit_code(), 130);
        assert_eq!(terminal.response.subject_state, SubjectState::Cancelled);
        assert_eq!(terminal.response.failures.len(), 2);
    }
    #[test]
    fn wrapper_raw_exit_is_operation_status_while_native_lifetime_stays_pending() {
        let mut terminal = TerminalResult::finalizer_unknown(
            &CliInvocationId::new(),
            "run",
            String::new(),
            Vec::new(),
            &RedactionPolicy::from_environment(),
        );
        terminal.response.outcome = CommandOutcome::Success;
        terminal.response.subject_state = SubjectState::Succeeded;
        terminal.response.child_exit = Some(0);
        terminal.response.observation.completeness = Completeness::Complete;
        assert_eq!(
            terminal.response.exit_code(),
            1,
            "whole native lifetime is still pending"
        );
        assert_eq!(
            terminal.service_exit_code(),
            0,
            "successful wrapper must not fail its native service"
        );
        terminal.response.child_exit = Some(4);
        terminal.response.outcome = CommandOutcome::Failure;
        assert_eq!(terminal.service_exit_code(), 4);
        terminal.response.observation.completeness = Completeness::Partial;
        assert_eq!(
            terminal.service_exit_code(),
            125,
            "raw child status cannot conceal missing capture"
        );
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
    /// Original fixed bundle composition, absent for other operations or incomplete helpers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<crate::cli::bundle::Summary>,
    /// Child/capture evidence may be complete while its reporting service is still alive.
    #[serde(default)]
    pub native_lifetime_pending: bool,
    /// Authenticated fixed workflow transport, absent for generic commands or unknown recovery.
    pub workflow: Option<WorkflowSummary>,
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
/// The helper owns compiler/test meaning; the original complete report remains addressed data.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowSummary {
    /// Original validated helper terminal, not a deserialized native verdict capability.
    pub terminal: crate::test_workflow::execute::TerminalRecord,
    /// Exact native artifact path, including platform bytes unavailable in a text-only link.
    pub report: std::ffi::OsString,
}

/// Terminal publication capability. Deserializing an envelope cannot construct this type.
#[derive(Debug)]
pub struct TerminalResult<T> {
    response: Envelope<T>,
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
    /// Preserve original native owner failures if their terminal evidence cannot be published.
    pub(crate) fn publication_failed(self, publication: HostError) -> HostError {
        self._failures
            .into_iter()
            .fold(publication, |cleanup, operation| HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            })
    }
}
impl TerminalResult<ManagedSummary> {
    pub(crate) fn with_bundle(
        mut self,
        directory: &crate::invocation::InvocationDirectory,
        policy: &RedactionPolicy,
    ) -> Self {
        if self.response.command != "run-all" {
            return self;
        }
        let summary = directory
            .evidence::<crate::cli::bundle::Summary>(crate::invocation::InvocationEvidence::Bundle)
            .and_then(|summary| {
                summary
                    .filter(|summary| &summary.invocation == directory.id())
                    .ok_or_else(|| {
                        HostError::Ownership(
                            "original bundle child results are absent or mismatched".into(),
                        )
                    })
            });
        match summary {
            Ok(summary) => {
                if !summary.complete() {
                    self.response.observation.completeness = Completeness::Partial;
                }
                if !summary.succeeded() && self.response.outcome == CommandOutcome::Success {
                    self.response.outcome = CommandOutcome::Failure;
                    self.response.subject_state = SubjectState::Failed;
                }
                for child in &summary.children {
                    self.response
                        .failures
                        .extend(child.dispatch_failure.iter().cloned());
                    if let Some(result) = &child.result {
                        self.response
                            .failures
                            .extend(result.failures.iter().cloned());
                    }
                }
                self.response.payload.bundle = Some(summary);
                self
            }
            Err(error) => self.with_owner_failures(vec![error], policy),
        }
    }
    /// The wrapper's raw OS exit describes its operation and required capture. It cannot
    /// describe its own future native drain, which remains pending in the immutable report.
    pub(crate) fn service_exit_code(&self) -> i32 {
        if self.response.subject_state == SubjectState::Cancelled {
            return 130;
        }
        if self.response.observation.completeness != Completeness::Complete
            || self.response.outcome == CommandOutcome::BoundaryFailure
        {
            return 125;
        }
        match self.response.child_exit {
            Some(0) if self.response.outcome == CommandOutcome::Success => 0,
            Some(code) if code != 0 => code,
            _ => 125,
        }
    }
    pub(crate) fn from_foreground<O: super::streams::ChunkObserver<Error = HostError>>(
        id: &CliInvocationId,
        command: &str,
        report: crate::foreground::CapturedForeground<O>,
        policy: &RedactionPolicy,
    ) -> Self {
        let cancelled = report.cancelled;
        let mut terminal =
            Self::from_payload(id, command, PayloadReport::from_foreground(report), policy);
        terminal.response.summary =
            "foreground child, capture and native ownership attempts completed".into();
        terminal.response.observation.scope = "native_foreground_owner".into();
        terminal.response.observation.limitations.clear();
        if cancelled {
            terminal.response.outcome = CommandOutcome::Interrupted;
        }
        terminal
    }
    pub(crate) fn from_payload<O: super::streams::ChunkObserver<Error = HostError>>(
        id: &CliInvocationId,
        command: &str,
        report: PayloadReport<O>,
        policy: &RedactionPolicy,
    ) -> Self {
        let resources_settled = report.resources_settled();
        let native_pending = report.native_pending;
        let operation_settled = if native_pending {
            report.child.is_ok() && report.kill.is_ok() && report.operation_error.is_none()
        } else {
            resources_settled
        };
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
            native_lifetime_pending: native_pending,
            bundle: None,
            workflow: None,
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
        response.settlement = if native_pending {
            Settlement::Unknown
        } else if resources_settled && !io_unresolved {
            Settlement::Complete
        } else {
            Settlement::Incomplete
        };
        response.observation.scope = "managed_payload_owner".into();
        if native_pending {
            response.observation.scope = "detached_child_and_capture_owner".into();
        }
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
        errors.extend(report.kill.err());
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
            && operation_settled
            && capture_complete
            && !report.cancelled
            && response.failures.is_empty();
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
            if !matches!(
                self.response.outcome,
                CommandOutcome::BoundaryFailure | CommandOutcome::Interrupted
            ) {
                self.response.outcome = CommandOutcome::Failure;
            }
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

    pub(crate) fn with_workflow(
        mut self,
        validation: Result<Option<super::workflow::Validated>, HostError>,
        policy: &RedactionPolicy,
    ) -> Self {
        use crate::test_workflow::execute::Outcome;
        match validation {
            Ok(None) => self,
            Err(error) => {
                self = self.with_owner_failures(vec![error], policy);
                if self.response.outcome != CommandOutcome::Interrupted {
                    self.response.outcome = CommandOutcome::BoundaryFailure;
                    self.response.subject_state = SubjectState::Unknown;
                }
                self
            }
            Ok(Some(validated)) => {
                let (terminal, path, failures) = validated.into_parts();
                self.response.failures.extend(failures);
                self.response.summary = match terminal.outcome {
                    Outcome::Listed => "actual selected native test population listed",
                    Outcome::CheckSucceeded => "native compiler check reconciled",
                    Outcome::TestsSucceeded => "selected native test execution reconciled",
                    Outcome::Failed => "native workflow reported failure",
                    Outcome::ZeroMatch => "native selection matched no tests",
                    Outcome::Pending => "native workflow requires prerequisite-owner readiness",
                    Outcome::Incomplete => "native workflow required observation is incomplete",
                }
                .into();
                if terminal.outcome == Outcome::Listed
                    && self.response.outcome == CommandOutcome::Success
                {
                    self.response.subject_state = SubjectState::Available;
                }
                if let Some(location) = path.to_str() {
                    self.response
                        .artifacts
                        .push(crate::envelope::ArtifactReference {
                            kind: "workflow_report".into(),
                            location: location.into(),
                            digest: Some(terminal.report.sha256.clone()),
                        });
                }
                if !terminal.outcome.successful(terminal.action)
                    && self.response.outcome != CommandOutcome::Interrupted
                {
                    (self.response.outcome, self.response.subject_state) = match terminal.outcome {
                        Outcome::Pending => {
                            (CommandOutcome::BoundaryFailure, SubjectState::Unprepared)
                        }
                        Outcome::Incomplete => {
                            (CommandOutcome::BoundaryFailure, SubjectState::Unknown)
                        }
                        Outcome::ZeroMatch => (CommandOutcome::Failure, SubjectState::NoMatch),
                        _ => (CommandOutcome::Failure, SubjectState::Failed),
                    };
                }
                if !terminal.required_capture_complete {
                    self.response.observation.completeness = Completeness::Partial;
                }
                self.response.observation.scope =
                    "fixed_workflow_protocol_and_managed_payload_owner".into();
                self.response.payload.workflow = Some(WorkflowSummary {
                    terminal,
                    report: path.into_os_string(),
                });
                self
            }
        }
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
                native_lifetime_pending: true,
                bundle: None,
                workflow: None,
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
            _streams: Vec::new(),
            _failures: errors,
        }
    }
}
