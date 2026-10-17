// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit execution-only detail selection across the managed canonical worker boundary.
use super::{StudyHandle, WorkflowError, contract};
use crate::capture::{CaptureBudget, DetailDocument, DetailRequest, DetailSession};
use pse_diagnostics::observation::{ObservationContext, OperationId};
use pse_model::{document::Version, generated::identities::StudyId};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::{Path, PathBuf}};

const RECEIPT_BYTES: u64 = 2048;
const MAX_REFERENCES: usize = 128;

/// Why an explicitly selected managed scope was not collected; science is independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManagedDetailRefusal {
    /// No separately declared managed-group allowance was supplied.
    NoManagedBudget,
    /// This actual collector already granted its managed allowance to another group.
    BudgetAlreadyAllocated,
    /// The immutable policies did not provide a representable finite attempt bound.
    AttemptBoundUnavailable,
    /// The group's per-attempt share cannot admit the finite collector and owner receipt.
    PerAttemptBudgetInsufficient,
    /// The receiving owner could not admit the requested collector.
    CollectorRefused,
}
/// Immutable execution metadata beside the scientific descriptor and its hashes.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagedDetailSelection {
    /// Execution metadata interpretation.
    pub version: Version<1>,
    /// Original explicit local selection; its local budget is not a child allocation.
    pub original_selection: DetailRequest,
    /// Sum of declared native attempt limits, including retries.
    pub attempt_slots: u64,
    /// Additional group allowance actually granted once by the originating collector.
    pub group_budget: Option<CaptureBudget>,
    /// Each native attempt's share, including its bounded owner receipt.
    pub per_attempt_budget: Option<CaptureBudget>,
    /// Explicit uncollected scope, independent of scientific permission.
    pub refusal: Option<ManagedDetailRefusal>,
}
impl ManagedDetailSelection {
    pub(crate) fn selected(attempt_slots: Option<u64>) -> Option<Self> {
        let mut request = crate::capture::current_detail_request()?;
        let declared = request.managed_budget.take();
        let slots = attempt_slots.unwrap_or(0);
        let refusal = if slots == 0 { Some(ManagedDetailRefusal::AttemptBoundUnavailable) }
            else if declared.is_none() { Some(ManagedDetailRefusal::NoManagedBudget) }
            else if share(declared.as_ref()?, slots).is_none() { Some(ManagedDetailRefusal::PerAttemptBudgetInsufficient) }
            else { None };
        let grant = if refusal.is_none() { crate::capture::claim_managed_detail_budget() } else { None };
        let refusal = refusal.or_else(|| grant.is_none().then_some(ManagedDetailRefusal::BudgetAlreadyAllocated));
        if let Some(reason) = refusal { crate::capture::mark_detail_scope_unavailable(&format!("managed study selection refused: {reason:?}")); }
        let per_attempt_budget = grant.as_ref().and_then(|budget| share(budget, slots));
        Some(Self { version: Version, original_selection: request, attempt_slots: slots, group_budget: grant, per_attempt_budget, refusal })
    }
    fn output(&self, study: &str, run: &str, attempt: &str) -> Option<PathBuf> {
        fn part(value: &str) -> Option<String> {
            (!value.is_empty() && value.len() <= 128 && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b':'))
                .then(|| value.replace(':', "_"))
        }
        Some(self.original_selection.output.join("managed").join(part(study)?).join(part(run)?).join(part(attempt)?))
    }
    fn child_request(&self, study: &str, run: &str, attempt: &str) -> Option<DetailRequest> {
        if self.refusal.is_some() || self.original_selection.expiry_monotonic_ns.is_none() { return None; }
        let declared = self.group_budget.as_ref()?;
        let expected = share(declared, self.attempt_slots)?;
        let budget = self.per_attempt_budget.as_ref()?;
        if budget.memory_bytes != expected.memory_bytes || budget.output_bytes != expected.output_bytes { return None; }
        let mut request = self.original_selection.clone();
        request.output = self.output(study, run, attempt)?;
        request.budget = CaptureBudget { memory_bytes: budget.memory_bytes, output_bytes: budget.output_bytes.checked_sub(RECEIPT_BYTES)? };
        request.managed_budget = None;
        // The original same-host monotonic expiry is preserved; the capture owner clamps it.
        Some(request)
    }
}
fn share(budget: &CaptureBudget, slots: u64) -> Option<CaptureBudget> {
    let slots = usize::try_from(slots).ok().filter(|slots| *slots > 0)?;
    let memory_bytes = budget.memory_bytes.checked_div(slots)?;
    let output_bytes = budget.output_bytes.checked_div(slots as u64)?;
    (memory_bytes >= 4 << 20 && output_bytes > RECEIPT_BYTES).then_some(CaptureBudget { memory_bytes, output_bytes })
}
/// Actual receiving process and diagnostic owner; never a mathematical identity.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagedDetailOwner {
    /// Actual canonical native run.
    pub run: String,
    /// Actual acknowledged native attempt key.
    pub attempt: String,
    /// Actual receiving OS process.
    pub pid: u32,
    /// Actual admitted process/operation locator, including the process instance.
    pub operation: OperationId,
    /// Explicit child collector admission refusal; absent for an admitted collector.
    pub refusal: Option<ManagedDetailRefusal>,
    /// Bounded actual collector refusal cause.
    pub error: Option<String>,
}
/// Availability of a selected child artifact, independent of its scientific result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManagedDetailReferenceState {
    /// The scientific owner has not acknowledged native dispatch for this attempt.
    Pending,
    /// The selected group was explicitly refused.
    Refused,
    /// A source artifact was read; its own coverage can still be incomplete.
    Captured,
    /// Dispatch occurred, but no readable complete child header was established.
    Missing,
}
/// Bounded readback for an actual currently assigned native attempt.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagedDetailReference {
    /// Exact canonical occurrence run.
    pub run: String,
    /// Exact current assigned canonical attempt.
    pub attempt: String,
    /// Exact selected child output directory.
    pub output: Option<PathBuf>,
    /// Explicit reference availability, not scientific permission.
    pub state: ManagedDetailReferenceState,
    /// Actual owner receipt when readable and matching the canonical locators.
    pub owner: Option<ManagedDetailOwner>,
    /// Actual source prefix and family/writer coverage, when readable.
    pub detail: Option<DetailDocument>,
    /// Bounded read/refusal explanation; absent without an observed failure.
    pub error: Option<String>,
}
/// Explicit managed selection inspection. Local capture completeness never establishes graph completeness.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagedStudyDiagnosticsDocument {
    /// Document encoding version.
    pub version: Version<1>,
    /// Actual owning study.
    pub study_id: StudyId,
    /// The originating execution-only selection, absent if none was supplied.
    pub selection: Option<ManagedDetailSelection>,
    /// At most 128 actual current assigned attempt references.
    pub references: Vec<ManagedDetailReference>,
    /// Requested occurrence rows outside this bounded inspection prefix.
    pub omitted_points: u64,
    /// Exact limits; historical retry artifacts are not substituted for current attempts.
    pub scope: String,
}

pub(crate) fn begin_child(selection: &ManagedDetailSelection, study: &str, run: &str, attempt: &str) -> Option<DetailSession> {
    let request = selection.child_request(study, run, attempt)?;
    let context = pse_diagnostics::observation::current()?;
    let owner = ManagedDetailOwner { run: run.into(), attempt: attempt.into(), pid: std::process::id(), operation: context.id(), refusal: None, error: None };
    let result = (|| {
        let parent = request.output.parent().ok_or_else(|| std::io::Error::other("child output parent absent"))?;
        std::fs::create_dir_all(parent)?;
        crate::capture::start_detailed_collection(context.clone(), request.clone()).map_err(std::io::Error::other)
    })();
    match result {
        Ok(session) => {
            let written = write_owner(&request.output.join("owner.json"), &owner);
            if let Err(error) = written { session.context().unavailable("managed.detail.owner_receipt", &error.to_string()); }
            Some(session)
        }
        Err(error) => {
            crate::capture::mark_detail_scope_unavailable("managed child collector admission refused");
            context.unavailable("managed.detail.collection", &error.to_string());
            // A bounded separate refusal receipt never replaces an existing child prefix.
            let mut owner = owner;
            owner.refusal = Some(ManagedDetailRefusal::CollectorRefused);
            owner.error = Some(bounded(error));
            if let Some(path) = refusal_path(&request.output) { let _ = write_owner(&path, &owner); }
            None
        }
    }
}
pub(crate) fn finish_child(session: Option<DetailSession>, context: Option<ObservationContext>) {
    if let Some(session) = session && let Err(error) = session.finish() && let Some(context) = context {
        context.unavailable("managed.detail.writer", &error.to_string());
    }
}
fn refusal_path(output: &Path) -> Option<PathBuf> {
    Some(output.parent()?.join(format!("{}-refusal.json", output.file_name()?.to_str()?)))
}
fn write_owner(path: &Path, owner: &ManagedDetailOwner) -> Result<(), std::io::Error> {
    use std::io::Write;
    let bytes = serde_json::to_vec(owner).map_err(std::io::Error::other)?;
    if bytes.len() as u64 > RECEIPT_BYTES { return Err(std::io::Error::other("managed detail receipt exceeds allowance")); }
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)
}
fn bounded(error: impl std::fmt::Display) -> String { error.to_string().chars().take(256).collect() }
fn owner(path: &Path) -> Result<ManagedDetailOwner, std::io::Error> {
    let file = std::fs::File::open(path)?;
    serde_json::from_reader(file.take(RECEIPT_BYTES + 1)).map_err(std::io::Error::other)
}
impl StudyHandle {
    /// Read selected execution metadata and bounded current child artifacts; performs no science.
    pub async fn diagnostic_selection(&self) -> Result<ManagedStudyDiagnosticsDocument, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity.scope(async {
            let store = self.runtime.canonical_store()?;
            let study = store.canonical_study(&self.study_id.to_string()).await?.ok_or_else(|| contract("canonical study absent"))?;
            let metadata: super::study::StudyMetadata = serde_json::from_slice(study.metadata.as_slice()).map_err(|error| contract(format!("study execution metadata: {error}")))?;
            let mut references = Vec::new();
            let mut scanned = 0_u64;
            let mut after = None;
            if let Some(selection) = &metadata.diagnostics {
                loop {
                    let page = store.study_point_page(&study.key, after).await?;
                    if page.is_empty() { break; }
                    for point in page {
                        if scanned as usize >= MAX_REFERENCES { break; }
                        scanned += 1; after = Some(point.ordinal);
                        let Some(attempt) = point.attempt.as_ref() else { continue; };
                        let output = selection.output(&study.key, &point.run, attempt);
                        let dispatched = point.facts()?.native_started;
                        let mut reference = ManagedDetailReference { run: point.run.clone(), attempt: attempt.clone(), output: output.clone(), state: if selection.refusal.is_some() { ManagedDetailReferenceState::Refused } else if !dispatched { ManagedDetailReferenceState::Pending } else { ManagedDetailReferenceState::Missing }, owner: None, detail: None, error: selection.refusal.map(|reason| format!("{reason:?}")) };
                        if selection.refusal.is_none() && let Some(output) = output {
                            let actual = owner(&output.join("owner.json")).or_else(|error| refusal_path(&output).map_or(Err(error), |path| owner(&path)));
                            if let Ok(actual) = actual && actual.run == point.run && actual.attempt == *attempt {
                                if actual.refusal.is_some() { reference.state = ManagedDetailReferenceState::Refused; reference.error = actual.error.clone(); }
                                reference.owner = Some(actual);
                            }
                            match selection.per_attempt_budget.as_ref().map(|budget| crate::capture::read_detailed_collection(&output.join("records.jsonl"), budget.output_bytes)) {
                                Some(Ok(detail)) => { reference.state = ManagedDetailReferenceState::Captured; reference.detail = Some(detail); }
                                Some(Err(error)) if dispatched && reference.state != ManagedDetailReferenceState::Refused => reference.error = Some(bounded(error)),
                                _ => {}
                            }
                        }
                        references.push(reference);
                    }
                    if scanned as usize >= MAX_REFERENCES { break; }
                }
            }
            Ok(ManagedStudyDiagnosticsDocument { version: Version, study_id: self.study_id, selection: metadata.diagnostics, references, omitted_points: study.point_count.saturating_sub(scanned), scope: "explicit managed child selection; current assigned attempts only, at most 128 occurrence rows; prior retry artifacts retain their exact paths but are not scanned; canonical references are observed per page, not an atomic graph snapshot; local capture completeness is not managed-graph completeness".into() })
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn managed_detail_partition_covers_retries_without_reusing_local_budget() {
        let budget = CaptureBudget { memory_bytes: 24 << 20, output_bytes: 60 << 20 };
        let child = share(&budget, 6).unwrap();
        assert_eq!(child.memory_bytes, 4 << 20);
        assert_eq!(child.output_bytes, 10 << 20);
        assert!(child.memory_bytes * 6 <= budget.memory_bytes);
        assert!(child.output_bytes * 6 <= budget.output_bytes);
        assert!(share(&budget, 7).is_none());
        assert!(share(&budget, 0).is_none());
        assert!(share(&CaptureBudget { memory_bytes: 4 << 20, output_bytes: RECEIPT_BYTES }, 1).is_none());
    }
}

#[cfg(test)]
mod selection_tests {
    use super::*;
    use crate::capture::{DetailOutcome, DetailRequirement, start_detailed_collection};
    use pse_diagnostics::observation::{ObservationHub, ProducerRole};
    use std::time::Duration;
    fn request(output: PathBuf, managed_budget: Option<CaptureBudget>) -> DetailRequest {
        DetailRequest { version: Version, families: vec!["integration.v1".into()], requirement: DetailRequirement::Complete,
            sample_every: 1, boundary_thread_cpu: false, duration: Duration::from_secs(10), output,
            budget: CaptureBudget { memory_bytes: 4 << 20, output_bytes: 1 << 20 }, managed_budget, expiry_monotonic_ns: None }
    }
    #[test]
    fn managed_budget_is_additional_once_only_and_missing_selection_refuses_explicitly() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.root");
        let capture = start_detailed_collection(operation.context(), request(directory.path().join("local"), Some(CaptureBudget { memory_bytes: 8 << 20, output_bytes: 2 << 20 }))).unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let selected = ManagedDetailSelection::selected(Some(2)).unwrap();
        assert!(selected.refusal.is_none());
        assert_eq!(selected.per_attempt_budget.as_ref().unwrap().memory_bytes, 4 << 20);
        let child = selected.child_request("aa", "run:bb", "cc").unwrap();
        assert!(child.managed_budget.is_none());
        assert_eq!(child.expiry_monotonic_ns, selected.original_selection.expiry_monotonic_ns);
        assert_eq!(child.budget.output_bytes + RECEIPT_BYTES, 1 << 20);
        assert!(selected.child_request("../escape", "run:bb", "cc").is_none());
        assert_eq!(ManagedDetailSelection::selected(Some(2)).unwrap().refusal, Some(ManagedDetailRefusal::BudgetAlreadyAllocated));
        assert!(capture.snapshot().source_partial);
        drop(_entered);
        capture.finish().unwrap();
    }
    #[test]
    fn child_collection_uses_actual_owner_and_survives_independent_parent_capture() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.owner");
        let capture = start_detailed_collection(operation.context(), request(directory.path().join("local"), Some(CaptureBudget { memory_bytes: 4 << 20, output_bytes: 1 << 20 }))).unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let selected = ManagedDetailSelection::selected(Some(1)).unwrap();
        let child = begin_child(&selected, "aa", "run:bb", "cc").unwrap();
        let producer = child.context().producer(ProducerRole::Workflow, "actual.child");
        let record = producer.record("integration.v1", "completed", None, None, vec![]);
        producer.publish(&record);
        let output = selected.output("aa", "run:bb", "cc").unwrap();
        finish_child(Some(child), Some(context));
        let receipt = owner(&output.join("owner.json")).unwrap();
        assert_eq!(receipt.pid, std::process::id());
        assert_eq!(receipt.run, "run:bb");
        assert_eq!(receipt.attempt, "cc");
        let detail = crate::capture::read_detailed_collection(&output.join("records.jsonl"), 1 << 20).unwrap();
        assert_eq!(detail.outcome, DetailOutcome::Complete);
        assert_eq!(detail.families[0].acknowledged, 1);
        drop(_entered);
        let local = capture.finish().unwrap();
        assert_ne!(local.outcome, DetailOutcome::Complete, "child coverage must not be substituted for an unobserved local source family");
    }
    #[test]
    fn expired_child_keeps_an_explicit_bounded_refusal_without_starting_a_new_window() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.expired");
        let capture = start_detailed_collection(operation.context(), request(directory.path().join("local"), Some(CaptureBudget { memory_bytes: 4 << 20, output_bytes: 1 << 20 }))).unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let mut selected = ManagedDetailSelection::selected(Some(1)).unwrap();
        selected.original_selection.expiry_monotonic_ns = Some(1);
        assert!(begin_child(&selected, "aa", "run:bb", "cc").is_none());
        let output = selected.output("aa", "run:bb", "cc").unwrap();
        let receipt = owner(&refusal_path(&output).unwrap()).unwrap();
        assert_eq!(receipt.refusal, Some(ManagedDetailRefusal::CollectorRefused));
        assert!(receipt.error.is_some());
        assert!(!output.join("records.jsonl").exists());
        drop(_entered);
        capture.finish().unwrap();
    }
    #[test]
    fn insufficient_group_share_does_not_consume_a_later_eligible_group_grant() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.insufficient");
        let capture = start_detailed_collection(operation.context(), request(directory.path().join("local"), Some(CaptureBudget { memory_bytes: 4 << 20, output_bytes: 1 << 20 }))).unwrap();
        let context = capture.context();
        let _entered = context.enter();
        assert_eq!(ManagedDetailSelection::selected(Some(2)).unwrap().refusal, Some(ManagedDetailRefusal::PerAttemptBudgetInsufficient));
        assert!(ManagedDetailSelection::selected(Some(1)).unwrap().refusal.is_none());
        drop(_entered);
        capture.finish().unwrap();
    }
    #[test]
    fn no_separate_child_budget_does_not_copy_the_local_collector_allowance() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.no_budget");
        let capture = start_detailed_collection(operation.context(), request(directory.path().join("local"), None)).unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let selected = ManagedDetailSelection::selected(Some(1)).unwrap();
        assert_eq!(selected.refusal, Some(ManagedDetailRefusal::NoManagedBudget));
        assert!(selected.per_attempt_budget.is_none());
        assert!(selected.child_request("aa", "run:bb", "cc").is_none());
        assert!(capture.snapshot().source_partial);
        drop(_entered);
        capture.finish().unwrap();
    }
}
