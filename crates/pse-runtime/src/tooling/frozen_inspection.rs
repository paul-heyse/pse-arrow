// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Recorded review. Supplied facts are the entire resolution authority.
use crate::tooling::inspection::{InspectionDocument, InspectionRequest, InspectionTransport};
use crate::{
    invocation::InvocationEvidenceDocument,
    observation::{ObservationError, ProcessCheckpoint},
};
use pse_diagnostics::observation::{OperationId, OperationSnapshot};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

/// Finite encoded/document admission. This is not a scientific workload budget.
pub const MAXIMUM_DOCUMENT_BYTES: usize = 64 << 20;
/// Recorded source-clock selection; no variant consults a reader clock.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecordedWindow {
    /// Anchor at the selected snapshot's recorded read_at with InspectionRequest.window.
    #[default]
    Recorded,
    /// Explicit endpoint on the selected operation's source clock.
    EndingAt {
        /// Recorded source-clock endpoint.
        #[schemars(with = "pse_model::document::ClosedDuration")]
        at: Duration,
    },
    /// Explicit closed source-clock range, retaining predecessor baselines.
    Interval {
        /// Requested source-clock start.
        #[schemars(with = "pse_model::document::ClosedDuration")]
        start: Duration,
        /// Requested source-clock end.
        #[schemars(with = "pse_model::document::ClosedDuration")]
        end: Duration,
    },
}
/// Explicit immutable read request, independent of a checkpoint's original filesystem location.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrozenInspectionRequest {
    /// Current request codec.
    pub version: Version<1>,
    /// Shared interpretation policy and exact operation identity.
    pub inspection: InspectionRequest,
    /// Recorded endpoint/range. Missing uses the source snapshot's read_at.
    #[serde(default)]
    pub window: RecordedWindow,
}
/// A supplied finite set. Paths and locators inside members are descriptive only.
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrozenEvidenceSet {
    /// Current supplied-set codec; older physical-observation shapes are not interpreted.
    pub version: Version<3>,
    /// Exact process publication images, at most 32; conflicting process identities refuse.
    pub checkpoints: Vec<ProcessCheckpoint>,
    /// Exact copied operation members supplied independently of a whole process image.
    #[serde(default)]
    pub operations: Vec<OperationSnapshot>,
    /// Saved invocation facts; no native or allocation owners can be reconstructed.
    #[serde(default)]
    pub invocations: Vec<InvocationEvidenceDocument>,
    /// Descriptive scientific context, independent of executable replay.
    #[serde(default)]
    pub contexts: Vec<crate::workflow::ReviewContextDocument>,
    /// Frozen exact membership/current-versus-retained attempt selections.
    #[serde(default)]
    pub inventories: Vec<crate::workflow::ReviewInventorySelection>,
    /// Capture metadata only. Embedded output paths never authorize file reads.
    #[serde(default)]
    pub captures: Vec<crate::tooling::capture::CaptureDocument>,
    /// Detail capture metadata only; raw pages are explicitly supplied separately.
    #[serde(default)]
    pub details: Vec<crate::observation_detail::DetailDocument>,
    /// Exact copied coherent source records, retaining their own clock and event identity.
    #[serde(default)]
    pub records: Vec<pse_diagnostics::observation::CoherentRecord>,
    /// Explicit immutable pages; embedded paths are descriptive, never resolution authority.
    #[serde(default)]
    pub detail_pages: Vec<crate::observation_detail::DetailRecordPage>,
    /// Copied engine observations, without live plan or session handles.
    #[serde(default)]
    pub engine_observations: Vec<pse_engine::session::CopiedExecutionObservations>,
    /// Contextual optional query outputs; raw members remain independent.
    #[serde(default)]
    pub query_results: Vec<crate::tooling::review_query::ReviewQueryDocument>,
}
/// Recorded interpretation plus explicit limits on temporal meaning.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrozenInspectionDocument {
    /// Current result codec.
    pub version: Version<1>,
    /// Actual recorded selection.
    pub request: FrozenInspectionRequest,
    /// Existing source-linked interpreter output; not live eligibility.
    pub inspection: InspectionDocument,
    /// Independent-clock/current-fact limitations, never guessed alignment or historical state.
    pub limitations: Vec<String>,
}
fn format(message: impl Into<String>) -> ObservationError {
    ObservationError::Format(message.into())
}
/// Decode supported version before interpreting nested fields, under a finite input bound.
pub fn decode_document<T: serde::de::DeserializeOwned, const N: u32>(
    bytes: &[u8],
) -> Result<T, ObservationError> {
    if bytes.len() > MAXIMUM_DOCUMENT_BYTES {
        return Err(format("recorded document exceeds 64 MiB"));
    }
    pse_model::document::decode_versioned::<T, N>(bytes).map_err(|error| format(error.to_string()))
}
/// Read exactly the supplied file; neither paths in its body nor neighbours are followed.
pub fn read_document<T: serde::de::DeserializeOwned, const N: u32>(
    path: &Path,
) -> Result<T, ObservationError> {
    read_document_with_cancellation::<T, N>(path, &pse_diagnostics::Cancellation::default())
}
/// Cooperative bounded file ingress; stopping the reader never stops its recorded producer.
pub fn read_document_with_cancellation<T: serde::de::DeserializeOwned, const N: u32>(
    path: &Path,
    cancel: &pse_diagnostics::Cancellation,
) -> Result<T, ObservationError> {
    let check = || {
        if cancel.is_cancelled() {
            Err(format("recorded reader cancelled"))
        } else {
            Ok(())
        }
    };
    check()?;
    let mut file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(format("recorded document requires a regular file"));
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 64 << 10];
    loop {
        check()?;
        let count = file.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        if bytes.len().saturating_add(count) > MAXIMUM_DOCUMENT_BYTES {
            return Err(format("recorded document exceeds 64 MiB"));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    check()?;
    let document = decode_document::<T, N>(&bytes)?;
    check()?;
    Ok(document)
}

struct Extent(usize);
impl Write for Extent {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= MAXIMUM_DOCUMENT_BYTES)
            .ok_or_else(|| std::io::Error::other("recorded document exceeds 64 MiB"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl FrozenEvidenceSet {
    /// Validate supplied identities and finite decoded/view extent, without resolving files.
    pub fn validate(&self) -> Result<(), ObservationError> {
        if self.query_results.len() > 1024
            || self.detail_pages.len() > 1024
            || self.engine_observations.len() > 1024
            || self.checkpoints.len() > 32
            || self.invocations.len() > 1024
            || self.contexts.len() > 1024
            || self.inventories.len() > 32
            || self.captures.len() > 32
            || self.details.len() > 32
            || self.records.len() > 65536
            || self.operations.len() > 1024
        {
            return Err(format("recorded set exceeds member bound"));
        }
        serde_json::to_writer(Extent(0), self).map_err(|error| format(error.to_string()))?;
        let mut processes = BTreeSet::new();
        let mut operations = BTreeSet::new();
        for image in &self.checkpoints {
            if !processes.insert(&image.process) {
                return Err(format("unsupported or duplicate checkpoint identity"));
            }
            for operation in &image.operations {
                let process = operation.id.process_instance;
                if format!("{:016x}-{:016x}", process[0], process[1]) != image.process
                    || !operations.insert((operation.id.process_instance, operation.id.sequence))
                {
                    return Err(format(
                        "checkpoint operation identity is inconsistent or duplicated",
                    ));
                }
            }
        }
        for operation in &self.operations {
            if !operations.insert((operation.id.process_instance, operation.id.sequence)) {
                return Err(format("duplicate supplied operation identity"));
            }
        }
        let mut invocations = BTreeSet::new();
        for invocation in &self.invocations {
            invocation.validate()?;
            if !invocations.insert((invocation.run_id, invocation.attempt_id)) {
                return Err(format(
                    "unsupported, inconsistent or duplicate invocation identity",
                ));
            }
        }
        for (index, context) in self.contexts.iter().enumerate() {
            context
                .validate()
                .map_err(|error| format(error.to_string()))?;
            if let Some(run) = context.run_id {
                for invocation in self.invocations.iter().filter(|invocation| {
                    invocation.run_id == run && invocation.attempt_id == context.attempt_id
                }) {
                    if context.operation.is_some()
                        && invocation.operation.is_some()
                        && context.operation != invocation.operation
                        && !self.context_operations_linked(context.operation, invocation.operation)
                    {
                        return Err(format(
                            "review context contradicts supplied invocation operation identity",
                        ));
                    }
                }
            }
            for previous in &self.contexts[..index] {
                // Missing lineage never establishes equality. Exact retries and
                // explicitly linked child operations retain their own scopes.
                let same_execution = context.run_id.is_some()
                    && context.run_id == previous.run_id
                    && context.attempt_id == previous.attempt_id;
                let contradictory_operation = context.operation.is_some()
                    && context.operation == previous.operation
                    && context.run_id.is_some()
                    && previous.run_id.is_some()
                    && context.run_id != previous.run_id;
                let same_prehandle = context.run_id.is_none()
                    && previous.run_id.is_none()
                    && context.operation.is_some()
                    && context.operation == previous.operation;
                let overlap = context.selection.steps.is_empty()
                    || previous.selection.steps.is_empty()
                    || context
                        .selection
                        .steps
                        .iter()
                        .any(|step| previous.selection.steps.contains(step));
                if (same_execution || same_prehandle || contradictory_operation)
                    && overlap
                    && !self.context_operations_linked(context.operation, previous.operation)
                {
                    return Err(format(
                        "duplicate or conflicting selected review context scope",
                    ));
                }
            }
        }
        for inventory in &self.inventories {
            inventory
                .validate()
                .map_err(|error| format(error.to_string()))?;
        }
        let mut records = BTreeSet::new();
        for record in &self.records {
            if !records.insert((
                record.producer.operation.process_instance,
                record.producer.operation.sequence,
                record.producer.sequence,
                record.generation,
                record.sequence,
            )) {
                return Err(format("duplicate supplied source event identity"));
            }
        }
        Ok(())
    }
    fn context_operations_linked(
        &self,
        first: Option<OperationId>,
        second: Option<OperationId>,
    ) -> bool {
        match (first, second) {
            (Some(first), Some(second)) if first != second => {
                self.operation(first).is_some_and(|operation| {
                    operation.children.iter().any(|child| child.id == second)
                }) || self.operation(second).is_some_and(|operation| {
                    operation.children.iter().any(|child| child.id == first)
                })
            }
            _ => false,
        }
    }
    /// Exact operation in the supplied set. No name, current pointer or path fallback.
    pub fn operation(&self, id: OperationId) -> Option<&OperationSnapshot> {
        self.checkpoints
            .iter()
            .flat_map(|image| &image.operations)
            .chain(&self.operations)
            .find(|op| op.id == id)
    }
    /// Project actual supplied relationship owners without discovering live children.
    pub fn operation_inventory(
        &self,
        scope: String,
        request: &crate::workflow::ReviewInventoryRequest,
    ) -> Result<crate::workflow::ReviewInventorySelection, ObservationError> {
        self.validate()?;
        let mut selected = crate::workflow::ReviewInventorySelection::from_operations(
            scope,
            request.attempts,
            self.checkpoints
                .iter()
                .flat_map(|image| &image.operations)
                .chain(&self.operations),
            request.maximum_members,
        )
        .map_err(|error| format(error.to_string()))?;
        for image in &self.checkpoints {
            if let Some(parent) = image.parent {
                selected.gaps.push(crate::workflow::ReviewInventoryGap {member:format!("process:{}",image.process),reason:format!("Recorded process-dispatch parent {parent} does not identify an exact child operation; no root/child binding or clock alignment is inferred.")});
            }
        }
        selected
            .validate()
            .map_err(|error| format(error.to_string()))?;
        Ok(selected)
    }
}
/// Interpret immutable copies at recorded source time using the shared live interpreter.
pub fn inspect_frozen(
    set: &FrozenEvidenceSet,
    request: FrozenInspectionRequest,
) -> Result<FrozenInspectionDocument, ObservationError> {
    set.validate()?;
    if let RecordedWindow::Interval { start, end } = &request.window
        && start > end
    {
        return Err(format("recorded interval start exceeds end"));
    }
    let source = set.operation(request.inspection.operation);
    let (endpoint, window) =
        source.map_or(
            (Duration::ZERO, request.inspection.window),
            |op| match request.window {
                RecordedWindow::Recorded => (op.read_at, request.inspection.window),
                RecordedWindow::EndingAt { at } => (at, request.inspection.window),
                RecordedWindow::Interval { start, end } => (end, end - start),
            },
        );
    let operation =
        source.map(|op| pse_diagnostics::inspection::select_window(op.clone(), endpoint, window));
    let mut related = Vec::new();
    let mut omitted = 0;
    let mut limitations = Vec::new();
    limitations.push("Current and cumulative fields describe the original publication; a selected earlier window does not reconstruct an earlier solver state. Freshness is recorded, not current live eligibility.".into());
    if request.inspection.include_related
        && let Some(op) = source
    {
        for child in &op.children {
            if related.len() >= request.inspection.related_limit.min(32) {
                omitted += 1;
                continue;
            }
            match set.operation(child.id) {
                Some(child) => {
                    let same_clock = child.id.process_instance == op.id.process_instance;
                    let at = if same_clock { endpoint } else { child.read_at };
                    if !same_clock {
                        limitations.push(format!("Related {} is selected on its independent recorded source clock; no cross-process ordering or alignment is established.",child.id));
                    }
                    related.push(pse_diagnostics::inspection::select_window(
                        child.clone(),
                        at,
                        window,
                    ));
                }
                None => {
                    omitted += 1;
                    limitations.push(format!(
                        "Related {} was not supplied; no live lookup was attempted.",
                        child.id
                    ));
                }
            }
        }
    }
    let image = set.checkpoints.iter().find(|image| {
        image
            .operations
            .iter()
            .any(|op| op.id == request.inspection.operation)
    });
    let transport = InspectionTransport {
        checkpoint: None,
        process: image.map(|image| image.locator.clone()),
        runtime_metrics: image.map(|image| image.runtime_metrics.clone()),
        published_unix_micros: image.map(|image| image.published_unix_micros),
        failure: None,
        partial: operation.is_none()
            || omitted > 0
            || image.is_some_and(|image| {
                image.omitted > 0
                    || image.registry_coverage.curtailed
                    || !image.registry_coverage.available
            }),
        clock_uncertainty_nanoseconds: image.and_then(|image| {
            image
                .clock
                .as_ref()
                .map(|clock| clock.uncertainty_nanoseconds)
        }),
    };
    let mut shared = request.inspection.clone();
    shared.window = window;
    let inspection =
        crate::tooling::inspection::document(shared, operation, related, omitted, transport);
    Ok(FrozenInspectionDocument {
        version: Version,
        request,
        inspection,
        limitations,
    })
}

/// Saved invocation facts admitted without recreating their former allocation/native owners.
pub fn read_invocation(path: &Path) -> Result<InvocationEvidenceDocument, ObservationError> {
    let doc: InvocationEvidenceDocument = read_document::<_, 2>(path)?;
    doc.validate()?;
    Ok(doc)
}

/// Read a validated descriptive context without source-store access or Replay admission.
pub fn read_context(
    path: &Path,
) -> Result<crate::workflow::ReviewContextDocument, ObservationError> {
    let doc: crate::workflow::ReviewContextDocument = read_document::<_, 2>(path)?;
    doc.validate().map_err(|error| format(error.to_string()))?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{Diagnostics, read_checkpoint};
    use pse_diagnostics::observation::{Policy, Relationship};
    #[test]
    fn context_scopes_reject_overlap_allow_exact_retries_disjoint_steps_and_linked_children() {
        use crate::workflow::{ReviewContextDocument, ReviewContextRequest};
        use pse_model::generated::identities::{AttemptId, RunId};
        fn context(
            run: RunId,
            attempt: Option<AttemptId>,
            steps: Vec<usize>,
            operation: Option<OperationId>,
        ) -> ReviewContextDocument {
            let mut context = ReviewContextDocument::pre_handle_failure(
                ReviewContextRequest {
                    steps,
                    ..Default::default()
                },
                operation,
                "retained".into(),
                vec![],
            )
            .unwrap();
            context.run_id = Some(run);
            context.attempt_id = attempt;
            context
        }
        let run = RunId::from_bytes([1; 16]);
        let attempt = Some(AttemptId::from_bytes([2; 16]));
        let (mut set, request) = fixture();
        let child = set
            .operation(request.inspection.operation)
            .unwrap()
            .children[0]
            .id;
        set.contexts.push(context(
            run,
            attempt,
            vec![0],
            Some(request.inspection.operation),
        ));
        set.contexts.push(context(
            run,
            attempt,
            vec![1],
            Some(request.inspection.operation),
        ));
        set.contexts.push(context(
            run,
            Some(AttemptId::from_bytes([3; 16])),
            vec![0],
            Some(request.inspection.operation),
        ));
        set.contexts
            .push(context(run, attempt, vec![0], Some(child)));
        set.contexts
            .push(context(RunId::from_bytes([4; 16]), None, vec![0], None));
        set.validate().unwrap();
        // A second run cannot claim the same known performing operation.
        set.contexts.push(context(
            RunId::from_bytes([5; 16]),
            attempt,
            vec![0],
            Some(child),
        ));
        assert!(set.validate().is_err());
        set.contexts.pop();
        set.contexts.push(context(
            run,
            attempt,
            vec![0],
            Some(request.inspection.operation),
        ));
        assert!(set.validate().is_err());
    }
    fn fixture() -> (FrozenEvidenceSet, FrozenInspectionRequest) {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let parent = owner.hub().start("recorded.parent");
        let child = parent
            .context()
            .child("recorded.child", Relationship::Nested);
        parent.context().count("actual.calls", "calls", u64::MAX);
        let request = FrozenInspectionRequest {
            version: Version,
            inspection: InspectionRequest::new(parent.context().id()),
            window: RecordedWindow::Recorded,
        };
        child.finish(pse_diagnostics::observation::Outcome::Completed);
        parent.finish(pse_diagnostics::observation::Outcome::Completed);
        owner.publish().unwrap();
        let image = read_checkpoint(&owner.checkpoint_path()).unwrap();
        owner.release().unwrap();
        (
            FrozenEvidenceSet {
                version: Version,
                checkpoints: vec![image],
                invocations: Vec::new(),
                ..Default::default()
            },
            request,
        )
    }
    #[test]
    fn frozen_inspection_retains_recorded_rows_after_relocation_and_unknown_reader_clock() {
        let (mut set, request) = fixture();
        let original = serde_json::to_vec(&set).unwrap();
        let expected = inspect_frozen(&set, request.clone()).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let moved = directory.path().join("moved.json");
        std::fs::write(&moved, &original).unwrap();
        std::fs::write(directory.path().join("unrelated.json"), b"not a checkpoint").unwrap();
        let restored: FrozenEvidenceSet = read_document::<_, 3>(&moved).unwrap();
        let actual = inspect_frozen(&restored, request.clone()).unwrap();
        assert_eq!(
            serde_json::to_vec(&expected).unwrap(),
            serde_json::to_vec(&actual).unwrap()
        );
        assert_eq!(std::fs::read(&moved).unwrap(), original);
        // Neither an absurd mapped reader endpoint nor missing mapping changes source selection.
        let recorded = actual.inspection.operation.as_ref().unwrap().read_at;
        set.checkpoints[0].clock = None;
        let no_clock = inspect_frozen(&set, request).unwrap();
        assert_eq!(
            no_clock.inspection.operation.as_ref().unwrap().read_at,
            recorded
        );
        assert_eq!(no_clock.inspection.related.len(), 1);
        assert_eq!(
            serde_json::to_vec(&no_clock.inspection.operation).unwrap(),
            serde_json::to_vec(&actual.inspection.operation).unwrap()
        );
    }
    #[test]
    fn cancelled_recorded_reader_does_not_open_a_file_or_control_a_producer() {
        let cancellation = pse_diagnostics::Cancellation::default();
        cancellation.cancel();
        let error = read_document_with_cancellation::<FrozenEvidenceSet, 3>(
            Path::new("nonexistent-recorded-member"),
            &cancellation,
        )
        .unwrap_err();
        assert!(error.to_string().contains("cancelled"));
    }
    #[test]
    fn plan47f_offline_readers_join_without_cancelling_independent_science() {
        let (set, request) = fixture();
        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("original.json");
        let relocated = directory.path().join("relocated.json");
        let bytes = serde_json::to_vec(&set).unwrap();
        std::fs::write(&original, &bytes).unwrap();
        std::fs::rename(&original, &relocated).unwrap();
        // A same-name live neighbour with conflicting identity cannot replace the pinned input.
        std::fs::write(
            &original,
            br#"{"version":999,"operations":"live conflict"}"#,
        )
        .unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(Policy::default());
        let science = hub.start("unrelated continuing science");
        let reader = science.reader();
        let barrier = std::sync::Barrier::new(7);
        std::thread::scope(|scope| {
            let mut readers = Vec::new();
            for index in 0..6 {
                let path = &relocated;
                let request = &request;
                let barrier = &barrier;
                readers.push(scope.spawn(move || {
                    let cancellation = pse_diagnostics::Cancellation::default();
                    if index % 2 == 0 {
                        cancellation.cancel();
                    }
                    barrier.wait();
                    let copied = read_document_with_cancellation::<FrozenEvidenceSet, 3>(
                        path,
                        &cancellation,
                    );
                    if index % 2 == 0 {
                        assert!(copied.unwrap_err().to_string().contains("cancelled"));
                    } else {
                        let inspected = inspect_frozen(&copied.unwrap(), request.clone()).unwrap();
                        assert_eq!(
                            inspected.inspection.operation.unwrap().id,
                            request.inspection.operation
                        );
                        assert_eq!(inspected.inspection.related.len(), 1);
                    }
                }));
            }
            barrier.wait();
            science
                .context()
                .count("actual.scientific.calls", "calls", 1);
            for reader in readers {
                reader.join().unwrap();
            }
        });
        assert_eq!(reader.snapshot(Duration::from_secs(60)).outcome, None);
        science.finish(pse_diagnostics::observation::Outcome::Completed);
        assert_eq!(
            reader.snapshot(Duration::from_secs(60)).outcome,
            Some(pse_diagnostics::observation::Outcome::Completed)
        );
        assert_eq!(std::fs::read(&relocated).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }
    #[test]
    fn frozen_ingress_refuses_versions_duplicates_conflicts_and_exposes_missing_members() {
        let (mut set, request) = fixture();
        set.checkpoints.push(set.checkpoints[0].clone());
        assert!(set.validate().is_err());
        set.checkpoints.pop();
        let root = request.inspection.operation;
        set.checkpoints[0].operations.retain(|op| op.id == root);
        let missing = inspect_frozen(&set, request.clone()).unwrap();
        assert_eq!(missing.inspection.omitted, 1);
        assert!(missing.inspection.transport.partial);
        let mut invalid = request.clone();
        invalid.window = RecordedWindow::Interval {
            start: Duration::from_secs(2),
            end: Duration::from_secs(1),
        };
        assert!(inspect_frozen(&set, invalid).is_err());
        set.checkpoints[0].operations[0].id.process_instance = [0, 0];
        assert!(set.validate().is_err());
        let unsupported = br#"{"checkpoints":"bad current body","version":9}"#;
        let error = decode_document::<FrozenEvidenceSet, 3>(unsupported).unwrap_err();
        assert!(error.to_string().contains("unsupported"));
    }

    #[test]
    fn structured_decoder_controls_keep_actual_operation_meaning_and_refuse_bad_boundaries() {
        use pse_diagnostics::observation::{ObservationHub, Outcome, Value};
        // Ordinary finite structured cases. These are not coverage-guided exploration.
        for count in [0, 1, 7, (1_u64 << 53) + 1, u64::MAX] {
            let hub = ObservationHub::new(Policy::default());
            let operation = hub.start("decoder.structured-control");
            let reader = operation.reader();
            operation.context().count("actual.calls", "count", count);
            operation.finish(Outcome::Completed);
            let set = FrozenEvidenceSet {
                operations: vec![reader.snapshot(Duration::from_secs(1))],
                ..Default::default()
            };
            let bytes = serde_json::to_vec(&set).unwrap();
            let decoded = decode_document::<FrozenEvidenceSet, 3>(&bytes).unwrap();
            decoded.validate().unwrap();
            assert_eq!(decoded.operations.len(), 1);
            let actual = &decoded.operations[0];
            assert_eq!(actual.kind, "decoder.structured-control");
            assert_eq!(actual.outcome, Some(Outcome::Completed));
            assert_eq!(
                actual
                    .metrics
                    .iter()
                    .find(|metric| metric.name == "actual.calls")
                    .unwrap()
                    .latest,
                Value::Count(count)
            );
            // Truncation cannot silently produce a complete accepted record.
            for end in [0, 1, bytes.len() / 2, bytes.len() - 1] {
                assert!(decode_document::<FrozenEvidenceSet, 3>(&bytes[..end]).is_err());
            }
        }
        let oversized = vec![b' '; MAXIMUM_DOCUMENT_BYTES + 1];
        assert!(
            decode_document::<FrozenEvidenceSet, 3>(&oversized)
                .unwrap_err()
                .to_string()
                .contains("exceeds 64 MiB")
        );
        assert!(
            decode_document::<FrozenEvidenceSet, 3>(
                br#"{"version":999,"checkpoints":"invalid body"}"#
            )
            .unwrap_err()
            .to_string()
            .contains("unsupported")
        );
        assert!(
            decode_document::<FrozenEvidenceSet, 3>(
                br#"{"version":3,"checkpoints":[{"process":null}]}"#
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod native_projection_tests {
    use super::*;
    use crate::observation::Diagnostics;
    use pse_diagnostics::observation::{Policy, ProducerRole, RecordField, Value};
    #[test]
    fn selected_native_families_keep_event_identity_in_saved_frozen_interpretation() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let operation = owner.hub().start("native.selected");
        let producer = operation
            .context()
            .producer(ProducerRole::NativeCallback, "selected.native");
        let mut records = Vec::new();
        for family in [
            "dynamic_local_error.v1",
            "native_factor.v2",
            "native_refinement.v2",
            "native_linear_call.v1",
        ] {
            let record = producer.record(
                family,
                "actual.owner.return",
                Some("native_contract"),
                Some("owner_defined"),
                vec![RecordField {
                    name: "call_ordinal".into(),
                    unit: "attempt_local_call_ordinal".into(),
                    value: Value::Count(u64::MAX),
                    availability: None,
                }],
            );
            assert!(producer.publish(&record));
            records.push(record);
        }
        let request = FrozenInspectionRequest {
            version: Version,
            inspection: InspectionRequest::new(operation.context().id()),
            window: RecordedWindow::Recorded,
        };
        operation.finish(pse_diagnostics::observation::Outcome::Completed);
        owner.publish().unwrap();
        let image = crate::observation::read_checkpoint(&owner.checkpoint_path()).unwrap();
        owner.release().unwrap();
        let set = FrozenEvidenceSet {
            checkpoints: vec![image],
            records,
            ..Default::default()
        };
        let bytes = serde_json::to_vec(&set).unwrap();
        drop(set);
        let copied: FrozenEvidenceSet = decode_document::<_, 3>(&bytes).unwrap();
        let result = inspect_frozen(&copied, request).unwrap();
        for code in [
            "native.idas.local_error_scope",
            "native.factor_scope",
            "native.refinement_scope",
            "native.linear_call_scope",
        ] {
            let statement = result
                .inspection
                .statements
                .iter()
                .find(|statement| statement.code == code)
                .unwrap();
            let evidence = &statement.evidence[0];
            assert!(evidence.record_sequence.is_some());
            assert!(evidence.producer.is_some());
            assert!(evidence.generation.is_some());
            assert!(
                copied
                    .records
                    .iter()
                    .any(|record| Some(record.sequence) == evidence.record_sequence
                        && Some(record.producer) == evidence.producer)
            );
        }
    }
}
