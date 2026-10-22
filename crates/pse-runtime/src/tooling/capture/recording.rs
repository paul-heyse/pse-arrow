// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cooperating preconstruction selection. An environment variable alone records nothing.
use crate::observation_detail::{
    CaptureError, DetailDocument, DetailRequest, DetailSession, start_detailed_collection,
};
use pse_diagnostics::observation::{ObservationContext, OperationId};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Explicit request-file location inherited by a cooperating construction entrypoint.
pub const SOURCE_RECORDING_REQUEST_ENV: &str = "PSE_DIAGNOSTIC_RECORD_REQUEST";
/// Maximum request or acknowledgement bytes; independent from retained source output.
pub const SOURCE_RECORDING_PROTOCOL_BYTES: u64 = 64 << 10;

/// Finite launch selection. It must be consumed before constructing the selected work.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordingRequest {
    /// Launch protocol version, independent from the detail document version.
    pub version: Version<2>,
    /// Original explicit finite source selection.
    pub detail: DetailRequest,
    /// Exclusive new acknowledgement file, written before selected work construction.
    pub acknowledgement: PathBuf,
    /// Optional original-context handoff; absence retains no extra scientific context.
    #[serde(default)]
    pub review: Option<super::ReviewHandoffRequest>,
}
/// Capability admission at this cooperating entrypoint; actual source activation is separate.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordingFamilyAcknowledgement {
    /// Exact requested versioned family.
    pub family: String,
    /// The entrypoint supports this source; this is not a completeness declaration.
    pub accepted: bool,
}
/// Actual receiver and admitted selection. Never a terminal coverage or science outcome.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordingAcknowledgement {
    /// Launch acknowledgement version.
    pub version: Version<2>,
    /// Actual receiving OS process, separate from its instance locator.
    pub pid: u32,
    /// Actual operation including the receiving process instance.
    pub operation: OperationId,
    /// Admitted selection, including its actual same-host expiry; original on refusal.
    pub selection: DetailRequest,
    /// Collector admitted and acknowledgement published before returning its context.
    pub accepted: bool,
    /// Exact per-family capability admission, independent from event activation.
    pub families: Vec<SourceRecordingFamilyAcknowledgement>,
    /// Actual same-host monotonic admission time, absent on refusal or unavailable clock.
    pub started_monotonic_ns: Option<u64>,
    /// Bounded collector refusal, independent from the scientific command outcome.
    pub error: Option<String>,
    /// Independent optional context-handoff admission and receipt location.
    #[serde(default)]
    pub review: Option<super::ReviewHandoffAcknowledgement>,
}
/// Owns only the selected writer. Dropping/stopping this value never cancels science.
#[derive(Debug)]
pub struct SourceRecordingSession {
    detail: DetailSession,
    acknowledgement: SourceRecordingAcknowledgement,
    handoff: Option<super::handoff::ReviewHandoffSession>,
}
/// Exclusive publisher claim. The final name is absent until every receipt byte
/// has been written and flushed; readers never see a claimed but empty ACK.
struct AcknowledgementPublisher {
    pending: PathBuf,
    destination: PathBuf,
    output: Option<std::fs::File>,
}
impl AcknowledgementPublisher {
    fn new(destination: &Path) -> Result<Self, CaptureError> {
        fn absent(path: &Path) -> std::io::Result<()> {
            match std::fs::symlink_metadata(path) {
                Ok(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "recording acknowledgement already has an owner",
                )),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        }
        absent(destination)?;
        let mut filename = destination
            .file_name()
            .ok_or_else(|| {
                CaptureError::Invalid("recording acknowledgement requires a file name".into())
            })?
            .to_os_string();
        filename.push(".pending");
        let pending = destination.with_file_name(filename);
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let output = options.open(&pending)?;
        let owner = Self {
            pending,
            destination: destination.into(),
            output: Some(output),
        };
        // A previous publisher could have completed between the first check and
        // this claim. Refuse before constructing another selected collector.
        absent(destination)?;
        Ok(owner)
    }
    fn publish(&mut self, bytes: &[u8]) -> Result<(), CaptureError> {
        let output = self.output.as_mut().ok_or_else(|| {
            CaptureError::Invalid("recording acknowledgement publisher already closed".into())
        })?;
        output.write_all(bytes)?;
        output.write_all(b"\n")?;
        output.flush()?;
        drop(self.output.take());
        // Same-directory hard linking publishes atomically and refuses an existing
        // name; rename would silently replace another command's acknowledgement.
        std::fs::hard_link(&self.pending, &self.destination)?;
        Ok(())
    }
}
impl Drop for AcknowledgementPublisher {
    fn drop(&mut self) {
        drop(self.output.take());
        let _ = std::fs::remove_file(&self.pending);
    }
}
impl SourceRecordingSession {
    /// Retain original joined facts only when descriptive handoff was explicitly selected.
    /// Optional failure is returned separately; it never changes science or detail coverage.
    pub fn retain_result(
        &mut self,
        result: &std::sync::Arc<crate::workflow::RunResult>,
    ) -> Result<Option<super::ReviewHandoffReceipt>, CaptureError> {
        self.handoff
            .as_mut()
            .map(|handoff| {
                let result = handoff.retain_result(result);
                if let Err(error) = &result {
                    handoff.failure(error);
                }
                result
            })
            .transpose()
    }
    /// Retain an actual pre-handle failure/context without reconstructing a run.
    pub fn retain_context(
        &mut self,
        context: &crate::workflow::ReviewContextDocument,
    ) -> Result<Option<super::ReviewHandoffReceipt>, CaptureError> {
        self.handoff
            .as_mut()
            .map(|handoff| {
                let result = handoff.retain_context(context);
                if let Err(error) = &result {
                    handoff.failure(error);
                }
                result
            })
            .transpose()
    }
    /// Use this context to construct future work. Earlier constructed owners are unchanged.
    pub fn context(&self) -> ObservationContext {
        self.detail.context()
    }
    /// Published admission receipt, never a terminal coverage statement.
    pub fn acknowledgement(&self) -> &SourceRecordingAcknowledgement {
        &self.acknowledgement
    }
    /// Nonblocking copied current coverage.
    pub fn snapshot(&self) -> DetailDocument {
        self.detail.snapshot()
    }
    /// Stop the writer interval without canceling any scientific owner.
    pub fn request_stop(&self) {
        self.detail.request_stop();
    }
    /// Close an owner-completed interval and join only the writer.
    pub fn finish(mut self) -> Result<DetailDocument, CaptureError> {
        if let Some(handoff) = &mut self.handoff {
            handoff.finish(true);
        }
        self.detail.finish()
    }
    /// Interrupt collection and join only the writer.
    pub fn stop(mut self) -> Result<DetailDocument, CaptureError> {
        if let Some(handoff) = &mut self.handoff {
            handoff.finish(false);
        }
        self.detail.stop()
    }
}
/// Admit an explicit selection at a real cooperating construction boundary.
/// `supported_families` belongs to that entrypoint's capabilities; an admitted family
/// still requires actual performing-source activation to establish complete coverage.
pub fn start_source_recording(
    context: ObservationContext,
    request: SourceRecordingRequest,
    supported_families: &[&str],
) -> Result<SourceRecordingSession, CaptureError> {
    if request.acknowledgement.as_os_str().len() > 4096 {
        return Err(CaptureError::Invalid(
            "recording acknowledgement path exceeds finite protocol bound".into(),
        ));
    }
    let encoded =
        serde_json::to_vec(&request).map_err(|error| CaptureError::Invalid(error.to_string()))?;
    if encoded.len() as u64 > SOURCE_RECORDING_PROTOCOL_BYTES {
        return Err(CaptureError::Invalid(
            "recording request exceeds finite protocol bound".into(),
        ));
    }
    // Reserve an exclusive receipt before creating the collector. Never replace another
    // command's acknowledgement or allow its stale receipt to stand in for this owner.
    let mut output = AcknowledgementPublisher::new(&request.acknowledgement)?;
    let mut acknowledgement = SourceRecordingAcknowledgement {
        version: Version,
        pid: std::process::id(),
        operation: context.id(),
        families: request
            .detail
            .families
            .iter()
            .map(|family| SourceRecordingFamilyAcknowledgement {
                family: family.clone(),
                accepted: supported_families.contains(&family.as_str()),
            })
            .collect(),
        selection: request.detail.clone(),
        accepted: false,
        started_monotonic_ns: None,
        error: None,
        review: None,
    };
    let handoff_output = request.detail.output.clone();
    let detail = start_detailed_collection(context, request.detail);
    match &detail {
        Ok(detail) => {
            acknowledgement.selection = detail.snapshot().request;
            acknowledgement.accepted = true;
            acknowledgement.started_monotonic_ns = detail.started_monotonic_ns();
            for family in &acknowledgement.families {
                if !family.accepted {
                    detail.refuse_unsupported_family(&family.family);
                }
            }
        }
        Err(error) => acknowledgement.error = Some(error.to_string().chars().take(512).collect()),
    }
    let mut handoff = None;
    if let Some(review) = request.review {
        let receipt = handoff_output.join("review/index.json");
        match &detail {
            Ok(detail) => {
                let admitted = detail.snapshot().request;
                let expiry = admitted.expiry_monotonic_ns.or_else(|| {
                    detail.started_monotonic_ns().and_then(|start| {
                        start.checked_add(u64::try_from(admitted.duration.as_nanos()).ok()?)
                    })
                });
                match super::handoff::ReviewHandoffSession::create(
                    &handoff_output,
                    acknowledgement.operation,
                    review.clone(),
                    expiry,
                ) {
                    Ok(selected) => {
                        acknowledgement.review = Some(selected.acknowledgement());
                        handoff = Some(selected);
                    }
                    Err(error) => {
                        acknowledgement.review = Some(super::ReviewHandoffAcknowledgement {
                            request: review,
                            accepted: false,
                            receipt,
                            error: Some(error.to_string()),
                        })
                    }
                }
            }
            Err(error) => {
                acknowledgement.review = Some(super::ReviewHandoffAcknowledgement {
                    request: review,
                    accepted: false,
                    receipt,
                    error: Some(format!(
                        "cooperating collector admission unavailable: {error}"
                    )),
                })
            }
        }
    }
    let bytes = serde_json::to_vec(&acknowledgement)
        .map_err(|error| CaptureError::Invalid(error.to_string()))?;
    if bytes.len() as u64 > SOURCE_RECORDING_PROTOCOL_BYTES {
        return Err(CaptureError::Invalid(
            "recording acknowledgement exceeds finite protocol bound".into(),
        ));
    }
    output.publish(&bytes)?;
    Ok(SourceRecordingSession {
        detail: detail?,
        acknowledgement,
        handoff,
    })
}
/// Consume one bounded, versioned request file before returning a selected context.
pub fn consume_source_recording_request(
    context: ObservationContext,
    path: &Path,
    supported_families: &[&str],
) -> Result<SourceRecordingSession, CaptureError> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(SOURCE_RECORDING_PROTOCOL_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > SOURCE_RECORDING_PROTOCOL_BYTES {
        return Err(CaptureError::Invalid(
            "recording request file exceeds finite protocol bound".into(),
        ));
    }
    let request =
        serde_json::from_slice(&bytes).map_err(|error| CaptureError::Invalid(error.to_string()))?;
    start_source_recording(context, request, supported_families)
}
/// Optional environment-selected cooperating boundary. Arbitrary launched commands
/// which never call this function produce no acknowledgement and gain no source coverage.
pub fn source_recording_from_environment(
    context: ObservationContext,
    supported_families: &[&str],
) -> Result<Option<SourceRecordingSession>, CaptureError> {
    std::env::var_os(SOURCE_RECORDING_REQUEST_ENV)
        .map(|path| consume_source_recording_request(context, Path::new(&path), supported_families))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation_detail::{
        CaptureBudget, DetailFamilyActivation, DetailOutcome, DetailRequirement,
    };
    use pse_diagnostics::observation::{ObservationHub, ProducerRole};
    use std::time::Duration;
    fn request(directory: &Path) -> SourceRecordingRequest {
        SourceRecordingRequest {
            version: Version,
            acknowledgement: directory.join("ack.json"),
            review: None,
            detail: DetailRequest {
                version: Version,
                families: vec!["short.work.v1".into()],
                requirement: DetailRequirement::Complete,
                sample_every: 1,
                boundary_thread_cpu: false,
                managed_budget: None,
                expiry_monotonic_ns: None,
                duration: Duration::from_secs(10),
                output: directory.join("detail"),
                budget: CaptureBudget::default(),
            },
        }
    }
    #[test]
    fn cooperating_request_acknowledges_actual_receiver_before_work_and_empty_source() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("cooperator");
        let request = request(directory.path());
        let path = directory.path().join("request.json");
        std::fs::write(&path, serde_json::to_vec(&request).unwrap()).unwrap();
        let session =
            consume_source_recording_request(operation.context(), &path, &["short.work.v1"])
                .unwrap();
        let ack: SourceRecordingAcknowledgement =
            serde_json::from_slice(&std::fs::read(&request.acknowledgement).unwrap()).unwrap();
        assert!(ack.accepted);
        assert_eq!(ack.operation, operation.context().id());
        assert_eq!(ack.pid, std::process::id());
        assert_eq!(session.snapshot().outcome, DetailOutcome::Active);
        let source = session
            .context()
            .producer(ProducerRole::Workflow, "empty real owner");
        source.activate_record_family("short.work.v1");
        let document = session.finish().unwrap();
        assert_eq!(document.outcome, DetailOutcome::Complete);
        assert_eq!(document.families[0].issued, 0);
        assert_eq!(
            document.families[0].activation,
            DetailFamilyActivation::Applicable
        );
        assert!(operation.context().is_active());
    }
    #[test]
    fn old_construction_unsupported_and_refused_selection_remain_distinct() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("cooperator");
        let old = operation
            .context()
            .producer(ProducerRole::Workflow, "already constructed");
        let session = start_source_recording(
            operation.context(),
            request(directory.path()),
            &["short.work.v1"],
        )
        .unwrap();
        old.activate_record_family("short.work.v1");
        let document = session.finish().unwrap();
        assert_eq!(document.outcome, DetailOutcome::Incomplete);
        assert_eq!(
            document.families[0].activation,
            DetailFamilyActivation::Unobserved
        );
        let unsupported = tempfile::tempdir().unwrap();
        let session =
            start_source_recording(operation.context(), request(unsupported.path()), &[]).unwrap();
        assert!(!session.acknowledgement().families[0].accepted);
        assert_eq!(
            session.finish().unwrap().families[0].activation,
            DetailFamilyActivation::Unsupported
        );
        let refused = tempfile::tempdir().unwrap();
        let mut request = request(refused.path());
        request.detail.budget.memory_bytes = 1;
        assert!(
            start_source_recording(operation.context(), request.clone(), &["short.work.v1"])
                .is_err()
        );
        let ack: SourceRecordingAcknowledgement =
            serde_json::from_slice(&std::fs::read(request.acknowledgement).unwrap()).unwrap();
        assert!(!ack.accepted);
        assert!(ack.error.is_some());
    }
    #[test]
    fn request_protocol_refuses_unknown_version_and_oversized_file() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("cooperator");
        let path = directory.path().join("request.json");
        let mut value = serde_json::to_value(request(directory.path())).unwrap();
        value["version"] = serde_json::json!(99);
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(consume_source_recording_request(operation.context(), &path, &[]).is_err());
        assert!(!directory.path().join("ack.json").exists());
        std::fs::write(
            &path,
            vec![b' '; SOURCE_RECORDING_PROTOCOL_BYTES as usize + 1],
        )
        .unwrap();
        assert!(consume_source_recording_request(operation.context(), &path, &[]).is_err());
    }
    #[test]
    fn acknowledgement_is_published_whole_and_never_replaces_an_existing_owner() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("ack.json");
        let mut publisher = AcknowledgementPublisher::new(&path).unwrap();
        assert!(
            !path.exists(),
            "reserving a publisher must not expose a partial ACK"
        );
        assert!(AcknowledgementPublisher::new(&path).is_err());
        publisher
            .publish(br#"{"version":1,"accepted":false}"#)
            .unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"{\"version\":1,\"accepted\":false}\n"
        );
        drop(publisher);
        assert!(!directory.path().join("ack.json.pending").exists());
        assert!(AcknowledgementPublisher::new(&path).is_err());
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"{\"version\":1,\"accepted\":false}\n"
        );
    }
    #[test]
    fn short_cooperating_child() {
        if std::env::var_os("PSE_TEST_SOURCE_COOPERATOR").is_none() {
            return;
        }
        let operation = ObservationHub::new(Default::default()).start("cooperating.short_work");
        let session = source_recording_from_environment(operation.context(), &["short.work.v1"])
            .unwrap()
            .unwrap();
        // Construction receives the selected context only after the ACK exists.
        let source = session
            .context()
            .producer(ProducerRole::Workflow, "actual short accumulation");
        source.activate_record_family("short.work.v1");
        let mut sum = 0;
        for value in 0..32 {
            sum += value;
            let record = source.record(
                "short.work.v1",
                "accumulate",
                None,
                None,
                vec![pse_diagnostics::observation::RecordField {
                    name: "sum".into(),
                    unit: "integer".into(),
                    value: pse_diagnostics::observation::Value::Integer(sum),
                    availability: None,
                }],
            );
            source.publish(&record);
        }
        assert_eq!(sum, 496);
        assert_eq!(session.finish().unwrap().outcome, DetailOutcome::Complete);
        assert!(operation.context().is_active());
    }
    #[test]
    fn actual_child_consumes_request_and_retains_original_source_population() {
        let directory = tempfile::tempdir().unwrap();
        let request = request(directory.path());
        let path = directory.path().join("request.json");
        std::fs::write(&path, serde_json::to_vec(&request).unwrap()).unwrap();
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "capture::recording::tests::short_cooperating_child",
                "--nocapture",
            ])
            .env("PSE_TEST_SOURCE_COOPERATOR", "1")
            .env(SOURCE_RECORDING_REQUEST_ENV, &path)
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
        let ack: SourceRecordingAcknowledgement =
            serde_json::from_slice(&std::fs::read(request.acknowledgement).unwrap()).unwrap();
        assert!(ack.accepted);
        assert_ne!(ack.pid, std::process::id());
        let document = crate::observation_detail::read_detailed_collection(
            &ack.selection.output.join("records.jsonl"),
            1 << 20,
        )
        .unwrap();
        assert_eq!(document.outcome, DetailOutcome::Complete);
        assert_eq!(document.families[0].issued, 32);
        assert_eq!(document.families[0].acknowledged, 32);
        let page =
            crate::observation_detail::read_detail_records(&document.records, 0, 64, 1 << 20)
                .unwrap();
        assert_eq!(page.records.len(), 32);
        assert_eq!(page.activations.len(), 1);
        assert!(
            page.activations
                .iter()
                .all(|source| source.producer.id.operation == ack.operation)
        );
    }
}
