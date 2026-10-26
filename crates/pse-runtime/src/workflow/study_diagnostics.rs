// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit execution-only detail selection across the managed canonical worker boundary.
use super::{StudyHandle, WorkflowError, contract};
use crate::observation_detail::{CaptureBudget, DetailDocument, DetailRequest, DetailSession};
use pse_diagnostics::observation::{ObservationContext, OperationId};
use pse_model::{document::Version, generated::identities::StudyId};
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

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
        let mut request = crate::observation_detail::current_detail_request()?;
        let declared = request.managed_budget.take();
        let slots = attempt_slots.unwrap_or(0);
        let refusal = if slots == 0 {
            Some(ManagedDetailRefusal::AttemptBoundUnavailable)
        } else if declared.is_none() {
            Some(ManagedDetailRefusal::NoManagedBudget)
        } else if share(declared.as_ref()?, slots).is_none() {
            Some(ManagedDetailRefusal::PerAttemptBudgetInsufficient)
        } else {
            None
        };
        let grant = if refusal.is_none() {
            crate::observation_detail::claim_managed_detail_budget()
        } else {
            None
        };
        let refusal = refusal.or_else(|| {
            grant
                .is_none()
                .then_some(ManagedDetailRefusal::BudgetAlreadyAllocated)
        });
        if let Some(reason) = refusal {
            crate::observation_detail::mark_detail_scope_unavailable(&format!(
                "managed study selection refused: {reason:?}"
            ));
        }
        let per_attempt_budget = grant.as_ref().and_then(|budget| share(budget, slots));
        Some(Self {
            version: Version,
            original_selection: request,
            attempt_slots: slots,
            group_budget: grant,
            per_attempt_budget,
            refusal,
        })
    }
    fn output(&self, study: &str, run: &str, attempt: &str) -> Option<PathBuf> {
        fn part(value: &str) -> Option<String> {
            (!value.is_empty()
                && value.len() <= 128
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b':'))
            .then(|| value.replace(':', "_"))
        }
        Some(
            self.original_selection
                .output
                .join("managed")
                .join(part(study)?)
                .join(part(run)?)
                .join(part(attempt)?),
        )
    }
    fn child_request(&self, study: &str, run: &str, attempt: &str) -> Option<DetailRequest> {
        if self.refusal.is_some() || self.original_selection.expiry_monotonic_ns.is_none() {
            return None;
        }
        let declared = self.group_budget.as_ref()?;
        let expected = share(declared, self.attempt_slots)?;
        let budget = self.per_attempt_budget.as_ref()?;
        if budget.memory_bytes != expected.memory_bytes
            || budget.output_bytes != expected.output_bytes
        {
            return None;
        }
        let mut request = self.original_selection.clone();
        request.output = self.output(study, run, attempt)?;
        request.budget = CaptureBudget {
            memory_bytes: budget.memory_bytes,
            output_bytes: budget.output_bytes.checked_sub(RECEIPT_BYTES)?,
        };
        request.managed_budget = None;
        // The original same-host monotonic expiry is preserved; the capture owner clamps it.
        Some(request)
    }
}
fn share(budget: &CaptureBudget, slots: u64) -> Option<CaptureBudget> {
    let slots = usize::try_from(slots).ok().filter(|slots| *slots > 0)?;
    let memory_bytes = budget.memory_bytes.checked_div(slots)?;
    let output_bytes = budget.output_bytes.checked_div(slots as u64)?;
    (memory_bytes >= 4 << 20 && output_bytes > RECEIPT_BYTES).then_some(CaptureBudget {
        memory_bytes,
        output_bytes,
    })
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

pub(crate) fn begin_child(
    selection: &ManagedDetailSelection,
    study: &str,
    run: &str,
    attempt: &str,
) -> Option<DetailSession> {
    let request = selection.child_request(study, run, attempt)?;
    let context = pse_diagnostics::observation::current()?;
    let owner = ManagedDetailOwner {
        run: run.into(),
        attempt: attempt.into(),
        pid: std::process::id(),
        operation: context.id(),
        refusal: None,
        error: None,
    };
    let result = (|| {
        let parent = request
            .output
            .parent()
            .ok_or_else(|| std::io::Error::other("child output parent absent"))?;
        std::fs::create_dir_all(parent)?;
        crate::observation_detail::start_detailed_collection(context.clone(), request.clone())
            .map_err(std::io::Error::other)
    })();
    match result {
        Ok(session) => {
            let written = write_owner(&request.output.join("owner.json"), &owner);
            if let Err(error) = written {
                session
                    .context()
                    .unavailable("managed.detail.owner_receipt", &error.to_string());
            }
            Some(session)
        }
        Err(error) => {
            crate::observation_detail::mark_detail_scope_unavailable(
                "managed child collector admission refused",
            );
            context.unavailable("managed.detail.collection", &error.to_string());
            // A bounded separate refusal receipt never replaces an existing child prefix.
            let mut owner = owner;
            owner.refusal = Some(ManagedDetailRefusal::CollectorRefused);
            owner.error = Some(bounded(error));
            if let Some(path) = refusal_path(&request.output) {
                let _ = write_owner(&path, &owner);
            }
            None
        }
    }
}
pub(crate) fn finish_child(session: Option<DetailSession>, context: Option<ObservationContext>) {
    if let Some(session) = session
        && let Err(error) = session.finish()
        && let Some(context) = context
    {
        context.unavailable("managed.detail.writer", &error.to_string());
    }
}
fn refusal_path(output: &Path) -> Option<PathBuf> {
    Some(
        output
            .parent()?
            .join(format!("{}-refusal.json", output.file_name()?.to_str()?)),
    )
}
fn write_owner(path: &Path, owner: &ManagedDetailOwner) -> Result<(), std::io::Error> {
    use std::io::Write;
    let bytes = serde_json::to_vec(owner).map_err(std::io::Error::other)?;
    if bytes.len() as u64 > RECEIPT_BYTES {
        return Err(std::io::Error::other(
            "managed detail receipt exceeds allowance",
        ));
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&bytes)
}
fn bounded(error: impl std::fmt::Display) -> String {
    error.to_string().chars().take(256).collect()
}
fn owner(path: &Path) -> Result<ManagedDetailOwner, std::io::Error> {
    let file = std::fs::File::open(path)?;
    serde_json::from_reader(file.take(RECEIPT_BYTES + 1)).map_err(std::io::Error::other)
}
impl StudyHandle {
    /// Read selected execution metadata and bounded current child artifacts; performs no science.
    pub async fn diagnostic_selection(
        &self,
    ) -> Result<ManagedStudyDiagnosticsDocument, WorkflowError> {
        let activity = self.services.preparation().admit_activity()?;
        activity.scope(async {
            let source_services = self.services.source_services();
            let store = source_services.canonical_store()?;
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
                            match selection.per_attempt_budget.as_ref().map(|budget| crate::observation_detail::read_detailed_collection(&output.join("records.jsonl"), budget.output_bytes)) {
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
        let budget = CaptureBudget {
            memory_bytes: 24 << 20,
            output_bytes: 60 << 20,
        };
        let child = share(&budget, 6).unwrap();
        assert_eq!(child.memory_bytes, 4 << 20);
        assert_eq!(child.output_bytes, 10 << 20);
        assert!(child.memory_bytes * 6 <= budget.memory_bytes);
        assert!(child.output_bytes * 6 <= budget.output_bytes);
        assert!(share(&budget, 7).is_none());
        assert!(share(&budget, 0).is_none());
        assert!(
            share(
                &CaptureBudget {
                    memory_bytes: 4 << 20,
                    output_bytes: RECEIPT_BYTES
                },
                1
            )
            .is_none()
        );
    }
}

#[cfg(test)]
mod selection_tests {
    use super::*;
    use crate::observation_detail::{DetailOutcome, DetailRequirement, start_detailed_collection};
    use pse_diagnostics::observation::{ObservationHub, ProducerRole};
    use std::time::Duration;
    fn request(output: PathBuf, managed_budget: Option<CaptureBudget>) -> DetailRequest {
        DetailRequest {
            version: Version,
            families: vec!["integration.v1".into()],
            requirement: DetailRequirement::Complete,
            sample_every: 1,
            boundary_thread_cpu: false,
            duration: Duration::from_secs(10),
            output,
            budget: CaptureBudget {
                memory_bytes: 4 << 20,
                output_bytes: 1 << 20,
            },
            managed_budget,
            expiry_monotonic_ns: None,
        }
    }
    #[test]
    fn managed_budget_is_additional_once_only_and_missing_selection_refuses_explicitly() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.root");
        let capture = start_detailed_collection(
            operation.context(),
            request(
                directory.path().join("local"),
                Some(CaptureBudget {
                    memory_bytes: 8 << 20,
                    output_bytes: 2 << 20,
                }),
            ),
        )
        .unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let selected = ManagedDetailSelection::selected(Some(2)).unwrap();
        assert!(selected.refusal.is_none());
        assert_eq!(
            selected.per_attempt_budget.as_ref().unwrap().memory_bytes,
            4 << 20
        );
        let child = selected.child_request("aa", "run:bb", "cc").unwrap();
        assert!(child.managed_budget.is_none());
        assert_eq!(
            child.expiry_monotonic_ns,
            selected.original_selection.expiry_monotonic_ns
        );
        assert_eq!(child.budget.output_bytes + RECEIPT_BYTES, 1 << 20);
        assert!(
            selected
                .child_request("../escape", "run:bb", "cc")
                .is_none()
        );
        assert_eq!(
            ManagedDetailSelection::selected(Some(2)).unwrap().refusal,
            Some(ManagedDetailRefusal::BudgetAlreadyAllocated)
        );
        assert!(capture.snapshot().source_partial);
        drop(_entered);
        capture.finish().unwrap();
    }
    #[test]
    fn child_collection_uses_actual_owner_and_survives_independent_parent_capture() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.owner");
        let capture = start_detailed_collection(
            operation.context(),
            request(
                directory.path().join("local"),
                Some(CaptureBudget {
                    memory_bytes: 4 << 20,
                    output_bytes: 1 << 20,
                }),
            ),
        )
        .unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let selected = ManagedDetailSelection::selected(Some(1)).unwrap();
        let child = begin_child(&selected, "aa", "run:bb", "cc").unwrap();
        let producer = child
            .context()
            .producer(ProducerRole::Workflow, "actual.child");
        let record = producer.record("integration.v1", "completed", None, None, vec![]);
        producer.publish(&record);
        let output = selected.output("aa", "run:bb", "cc").unwrap();
        finish_child(Some(child), Some(context));
        let receipt = owner(&output.join("owner.json")).unwrap();
        assert_eq!(receipt.pid, std::process::id());
        assert_eq!(receipt.run, "run:bb");
        assert_eq!(receipt.attempt, "cc");
        let detail = crate::observation_detail::read_detailed_collection(
            &output.join("records.jsonl"),
            1 << 20,
        )
        .unwrap();
        assert_eq!(detail.outcome, DetailOutcome::Complete);
        assert_eq!(detail.families[0].acknowledged, 1);
        drop(_entered);
        let local = capture.finish().unwrap();
        assert_ne!(
            local.outcome,
            DetailOutcome::Complete,
            "child coverage must not be substituted for an unobserved local source family"
        );
    }
    #[test]
    fn expired_child_keeps_an_explicit_bounded_refusal_without_starting_a_new_window() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.expired");
        let capture = start_detailed_collection(
            operation.context(),
            request(
                directory.path().join("local"),
                Some(CaptureBudget {
                    memory_bytes: 4 << 20,
                    output_bytes: 1 << 20,
                }),
            ),
        )
        .unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let mut selected = ManagedDetailSelection::selected(Some(1)).unwrap();
        selected.original_selection.expiry_monotonic_ns = Some(1);
        assert!(begin_child(&selected, "aa", "run:bb", "cc").is_none());
        let output = selected.output("aa", "run:bb", "cc").unwrap();
        let receipt = owner(&refusal_path(&output).unwrap()).unwrap();
        assert_eq!(
            receipt.refusal,
            Some(ManagedDetailRefusal::CollectorRefused)
        );
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
        let capture = start_detailed_collection(
            operation.context(),
            request(
                directory.path().join("local"),
                Some(CaptureBudget {
                    memory_bytes: 4 << 20,
                    output_bytes: 1 << 20,
                }),
            ),
        )
        .unwrap();
        let context = capture.context();
        let _entered = context.enter();
        assert_eq!(
            ManagedDetailSelection::selected(Some(2)).unwrap().refusal,
            Some(ManagedDetailRefusal::PerAttemptBudgetInsufficient)
        );
        assert!(
            ManagedDetailSelection::selected(Some(1))
                .unwrap()
                .refusal
                .is_none()
        );
        drop(_entered);
        capture.finish().unwrap();
    }
    #[test]
    fn no_separate_child_budget_does_not_copy_the_local_collector_allowance() {
        let directory = tempfile::tempdir().unwrap();
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("managed.no_budget");
        let capture = start_detailed_collection(
            operation.context(),
            request(directory.path().join("local"), None),
        )
        .unwrap();
        let context = capture.context();
        let _entered = context.enter();
        let selected = ManagedDetailSelection::selected(Some(1)).unwrap();
        assert_eq!(
            selected.refusal,
            Some(ManagedDetailRefusal::NoManagedBudget)
        );
        assert!(selected.per_attempt_budget.is_none());
        assert!(selected.child_request("aa", "run:bb", "cc").is_none());
        assert!(capture.snapshot().source_partial);
        drop(_entered);
        capture.finish().unwrap();
    }
}

/// Retry selection never substitutes a current attempt for an explicit historical identity.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAttemptSelection {
    /// Only the pointer recorded in each selected current occurrence.
    Current,
    /// All readable retained generations through each run's observed generation fence.
    Retained,
}
/// Finite selected review scope; public pages read only the resulting immutable selection.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewInventoryRequest {
    /// Current occurrence assignments or retained retry history.
    pub attempts: ReviewAttemptSelection,
    /// Maximum selected members; 1 through 4096.
    pub maximum_members: usize,
}
impl ReviewInventoryRequest {
    fn selects_attempt(&self, current: Option<&str>, candidate: &str) -> bool {
        self.attempts == ReviewAttemptSelection::Retained || current == Some(candidate)
    }
}
/// An actual recorded relationship. Its missing target remains a gap.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReviewInventoryLink {
    /// Existing producer role, e.g. parent, shared_producer, waiter or background.
    pub role: String,
    /// Exact member identity, never a display name.
    pub target: String,
}
/// Immutable copied member metadata, with actual readable result-prefix facts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewInventoryMember {
    /// Exact identity in this supplied selection.
    pub key: String,
    /// Existing producer role, not a fabricated global tree classification.
    pub role: String,
    /// Canonical semantic run, when established.
    pub run: Option<String>,
    /// Exact canonical attempt, when established.
    pub attempt: Option<String>,
    /// Exact diagnostic operation, including pre-handle failures.
    pub operation: Option<OperationId>,
    /// Recorded canonical generation fence, independent of current pointer drift.
    pub generation: Option<u64>,
    /// Original occurrence ordinal when this is study membership.
    pub occurrence: Option<u64>,
    /// Actual terminal fact at selection time.
    pub terminal: bool,
    /// Actual original outcome, including failure/cancellation when retained.
    pub outcome: Option<String>,
    /// Exact closed result manifest. None means not known, not an empty result.
    pub result_manifest: Option<String>,
    /// Actual recorded readable prefix/fence. Canonical attempts use close-generation here,
    /// which is never a row count or evidence of an empty result; None means unavailable.
    pub readable_prefix: Option<u64>,
    /// Actual links supplied by the relationship owner.
    pub links: Vec<ReviewInventoryLink>,
}
/// Explicit unavailable, omitted or unresolved selection premise.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewInventoryGap {
    /// Exact affected identity or scope.
    pub member: String,
    /// Recorded reason; no absence is promoted to a complete graph.
    pub reason: String,
}
/// Frozen metadata selected at owner boundaries; never an atomic cross-process graph.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewInventorySelection {
    /// Current immutable inventory codec version.
    pub version: Version<1>,
    /// Explicit scope selected by the caller.
    pub scope: String,
    /// Retry meaning used by this selection.
    pub attempts: ReviewAttemptSelection,
    /// Bounded immutable member prefix, in its recorded order.
    pub members: Vec<ReviewInventoryMember>,
    /// Missing links, unavailable retained history and explicit prefix omissions.
    pub gaps: Vec<ReviewInventoryGap>,
    /// True only when selection stopped at its finite member bound.
    pub truncated: bool,
}
/// One exact immutable selection page. Re-reading it performs no live lookup.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewInventoryPage {
    /// Current inventory page codec version.
    pub version: Version<1>,
    /// Original selected scope.
    pub scope: String,
    /// Original selection offset.
    pub offset: usize,
    /// Actual selected members, including exact empty pages.
    pub members: Vec<ReviewInventoryMember>,
    /// Next offset only when another retained member exists.
    pub next: Option<usize>,
    /// Selection-level gaps remain available on every page.
    pub gaps: Vec<ReviewInventoryGap>,
    /// Original bounded-prefix status.
    pub truncated: bool,
}
impl ReviewInventorySelection {
    /// Freeze supplied relationship-owner metadata. No runtime, service or filesystem is consulted.
    pub fn from_members(
        scope: String,
        attempts: ReviewAttemptSelection,
        members: Vec<ReviewInventoryMember>,
        mut gaps: Vec<ReviewInventoryGap>,
        truncated: bool,
    ) -> Result<Self, WorkflowError> {
        use std::collections::BTreeSet;
        if members.len() > 4096 || gaps.len() > 16384 || scope.len() > 4096 {
            return Err(contract(
                "review inventory exceeds finite metadata selection",
            ));
        }
        if members
            .iter()
            .map(|member| member.links.len())
            .sum::<usize>()
            > 16384
        {
            return Err(contract(
                "review inventory exceeds finite relationship selection",
            ));
        }
        let mut identities = BTreeSet::new();
        for member in &members {
            if member.key.is_empty() || !identities.insert(member.key.as_str()) {
                return Err(contract(
                    "review inventory contains empty or duplicate exact member identity",
                ));
            }
        }
        for member in &members {
            for link in &member.links {
                if !identities.contains(link.target.as_str()) {
                    gaps.push(ReviewInventoryGap {
                        member: link.target.clone(),
                        reason: format!(
                            "{} link from {} is outside supplied readable membership",
                            link.role, member.key
                        ),
                    });
                }
            }
        }
        let selected = Self {
            version: Version,
            scope,
            attempts,
            members,
            gaps,
            truncated,
        };
        selected.validate()?;
        Ok(selected)
    }
    /// Project copied relationship-owner facts, without a live reader or scientific callback.
    /// Operation kinds and performing roles are retained verbatim; labels never bind identities.
    pub fn from_operations<'a>(
        scope: String,
        attempts: ReviewAttemptSelection,
        operations: impl IntoIterator<Item = &'a pse_diagnostics::observation::OperationSnapshot>,
        maximum_members: usize,
    ) -> Result<Self, WorkflowError> {
        if maximum_members == 0 || maximum_members > 4096 {
            return Err(contract("review inventory requires 1 through 4096 members"));
        }
        let mut members = Vec::new();
        let mut gaps = Vec::new();
        let mut truncated = false;
        let mut seen = std::collections::HashSet::new();
        let mut links = 0_usize;
        for operation in operations {
            if members.len() >= maximum_members {
                truncated = true;
                break;
            }
            if !seen.insert(operation.id) {
                return Err(contract(
                    "review inventory contains duplicate copied operation identity",
                ));
            }
            links = links
                .saturating_add(operation.children.len())
                .saturating_add(operation.producers.len());
            if links > 16384 {
                return Err(contract(
                    "review inventory exceeds finite relationship selection",
                ));
            }
            let key = format!("operation:{}", operation.id);
            let relationships = operation
                .children
                .iter()
                .map(|child| ReviewInventoryLink {
                    role: child.relationship.to_string(),
                    target: format!("operation:{}", child.id),
                })
                .collect();
            members.push(ReviewInventoryMember {
                key: key.clone(),
                role: operation.kind.clone(),
                run: None,
                attempt: None,
                operation: Some(operation.id),
                generation: None,
                occurrence: None,
                terminal: operation.outcome.is_some(),
                outcome: operation.outcome.map(|outcome| outcome.to_string()),
                result_manifest: None,
                readable_prefix: None,
                links: relationships,
            });
            if operation.interval_partial
                || operation.detail_partial
                || operation.coverage.curtailed
                || operation.coverage.refused_items > 0
            {
                gaps.push(ReviewInventoryGap { member: key.clone(), reason: "relationship owner retained partial/refused diagnostic history; supplied membership does not establish complete history".into() });
            }
            for child in &operation.children {
                if !child.available {
                    gaps.push(ReviewInventoryGap {
                        member: format!("operation:{}", child.id),
                        reason: "relationship owner explicitly reports child summary unavailable"
                            .into(),
                    });
                }
            }
            for producer in &operation.producers {
                if members.len() >= maximum_members {
                    truncated = true;
                    break;
                }
                if producer.descriptor.id.operation != operation.id {
                    return Err(contract(
                        "review inventory producer/operation identity mismatch",
                    ));
                }
                let producer_key = format!(
                    "producer:{}:{}",
                    operation.id, producer.descriptor.id.sequence
                );
                members.push(ReviewInventoryMember {
                    key: producer_key.clone(),
                    role: format!(
                        "{:?}:{}",
                        producer.descriptor.role, producer.descriptor.name
                    ),
                    run: None,
                    attempt: None,
                    operation: Some(operation.id),
                    generation: None,
                    occurrence: None,
                    terminal: false,
                    outcome: None,
                    result_manifest: None,
                    readable_prefix: None,
                    links: vec![ReviewInventoryLink {
                        role: "performing_owner".into(),
                        target: key.clone(),
                    }],
                });
                gaps.push(ReviewInventoryGap { member: producer_key, reason: "performing identity and role retained; independent terminal outcome and scientific result prefix are not supplied by the producer catalogue".into() });
            }
            if truncated {
                break;
            }
        }
        if truncated {
            gaps.push(ReviewInventoryGap {
                member: scope.clone(),
                reason: "selected finite operation/member prefix omits later supplied membership"
                    .into(),
            });
        }
        gaps.push(ReviewInventoryGap { member: scope.clone(), reason: "copied diagnostic relationship-owner selection only; canonical run/attempt identities and payload readability require explicit corresponding owner metadata; clocks remain independent and absent relationships are not a complete atomic graph".into() });
        Self::from_members(scope, attempts, members, gaps, truncated)
    }
    /// Check immutable ingress without consulting any live relationship owner.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.members.len() > 4096 || self.gaps.len() > 32768 || self.scope.len() > 4096 {
            return Err(contract(
                "review inventory exceeds finite metadata selection",
            ));
        }
        let mut keys = std::collections::BTreeSet::new();
        for member in &self.members {
            if member.key.is_empty() || !keys.insert(member.key.as_str()) {
                return Err(contract(
                    "review inventory contains empty or duplicate exact member identity",
                ));
            }
        }
        if self
            .members
            .iter()
            .map(|member| member.links.len())
            .sum::<usize>()
            > 16384
        {
            return Err(contract(
                "review inventory exceeds finite relationship selection",
            ));
        }
        let gap_targets = self
            .gaps
            .iter()
            .filter(|gap| !gap.reason.is_empty())
            .map(|gap| gap.member.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for member in &self.members {
            for link in &member.links {
                if !keys.contains(link.target.as_str())
                    && !gap_targets.contains(link.target.as_str())
                {
                    return Err(contract(
                        "review inventory missing relationship target requires an explicit gap",
                    ));
                }
            }
        }
        super::replay::encoded_extent(self, 16 << 20)?;
        Ok(())
    }
    /// Page only this frozen prefix; offsets cannot trigger discovery of later live members.
    pub fn page(&self, offset: usize, limit: usize) -> Result<ReviewInventoryPage, WorkflowError> {
        self.validate()?;
        if limit == 0 || limit > 256 || offset > self.members.len() {
            return Err(contract(
                "review inventory page requires valid offset and 1 through 256 members",
            ));
        }
        let end = offset.saturating_add(limit).min(self.members.len());
        Ok(ReviewInventoryPage {
            version: Version,
            scope: self.scope.clone(),
            offset,
            members: self.members[offset..end].to_vec(),
            next: (end < self.members.len()).then_some(end),
            gaps: self.gaps.clone(),
            truncated: self.truncated,
        })
    }
}
fn retry_fence_gap(
    run: &str,
    through: u64,
    last_readable: Option<u64>,
    truncated: bool,
) -> Option<ReviewInventoryGap> {
    (!truncated && !(through == 0 && last_readable.is_none()) && last_readable != Some(through)).then(|| ReviewInventoryGap {
        member: run.into(),
        reason: format!("retained retry prefix ends at {last_readable:?}, before observed generation fence {through}; tail history is unavailable"),
    })
}
impl StudyHandle {
    /// Freeze a selected canonical occurrence/retry prefix. History-owner guards remain in force.
    pub async fn review_inventory(
        &self,
        request: &ReviewInventoryRequest,
    ) -> Result<ReviewInventorySelection, WorkflowError> {
        if request.maximum_members == 0 || request.maximum_members > 4096 {
            return Err(contract("review inventory requires 1 through 4096 members"));
        }
        let activity = self.services.preparation().admit_activity()?;
        activity.scope(async {
            let source_services = self.services.source_services();
            let store = source_services.canonical_store()?;
            let study = store.canonical_study(&self.study_id.to_string()).await?.ok_or_else(|| contract("canonical study absent"))?;
            let mut members = Vec::new(); let mut gaps = Vec::new(); let mut after = None; let mut scanned = 0_u64;
            let mut truncated = false;
            'points: loop {
                let points = store.study_point_page(&study.key, after).await?;
                if points.is_empty() { break; }
                for point in points {
                    if point.ordinal >= study.point_count { break 'points; }
                    after = Some(point.ordinal); scanned += 1;
                    let point_key = format!("occurrence:{}:{}", study.key, point.ordinal);
                    if members.len() >= request.maximum_members { truncated = true; break 'points; }
                    members.push(ReviewInventoryMember { key: point_key.clone(), role: "study_occurrence".into(), run: Some(point.run.clone()), attempt: point.attempt.clone(), operation: None, generation: None, occurrence: Some(point.ordinal), terminal: point.settled, outcome: None, result_manifest: None, readable_prefix: None, links: Vec::new() });
                    let current = point.attempt.as_deref();
                    if current.is_none() {
                        gaps.push(ReviewInventoryGap { member: point_key.clone(), reason: "occurrence has no established current attempt; preparation/pending state is retained".into() });
                        if request.attempts == ReviewAttemptSelection::Current { continue; }
                    }
                    let mut observed_fence = None;
                    let selected = match request.attempts {
                        ReviewAttemptSelection::Current => match current {
                            Some(current) => store.canonical_attempt(current).await?.into_iter().map(pse_operations::canonical_execution::CanonicalAttemptMetadata::from).collect::<Vec<_>>(),
                            None => Vec::new(),
                        },
                        ReviewAttemptSelection::Retained => {
                            let Some(run) = store.canonical_run(&point.run).await? else { gaps.push(ReviewInventoryGap { member: point.run.clone(), reason: "selected occurrence run metadata unavailable".into() }); continue; };
                            observed_fence = Some(run.current_generation);
                            if let Some(occurrence) = members.iter_mut().find(|member| member.key == point_key) {
                                occurrence.generation = observed_fence;
                            }
                            let mut attempts = Vec::new(); let mut generation = None;
                            loop {
                                let page = store.canonical_attempt_page(&point.run, generation, run.current_generation).await?;
                                if page.is_empty() { break; }
                                generation = page.last().map(|row| row.generation);
                                for attempt in page { if attempts.len().saturating_add(members.len()) >= request.maximum_members { truncated = true; break; } attempts.push(attempt); }
                                if truncated { break; }
                            }
                            if let Some(gap) = retry_fence_gap(&point.run, run.current_generation, attempts.last().map(|attempt| attempt.generation), truncated) {
                                gaps.push(gap);
                            }
                            if attempts.first().is_some_and(|attempt| attempt.generation > 1) || attempts.windows(2).any(|pair| pair[0].generation.checked_add(1) != Some(pair[1].generation)) {
                                gaps.push(ReviewInventoryGap { member: point.run.clone(), reason: "retained retry generations have unavailable earlier/intermediate history".into() });
                            }
                            attempts
                        }
                    };
                    if request.attempts == ReviewAttemptSelection::Current
                        && let Some(occurrence) = members.iter_mut().find(|member| member.key == point_key) {
                            occurrence.generation = selected.first().map(|attempt| attempt.generation);
                    }
                    if selected.is_empty() && observed_fence != Some(0) {
                        gaps.push(ReviewInventoryGap { member: current.unwrap_or(&point.run).into(), reason: "selected exact attempt metadata unavailable".into() });
                    }
                    for attempt in selected {
                        if !request.selects_attempt(current, &attempt.key) { continue; }
                        if attempt.run != point.run { return Err(contract("review inventory attempt/run identity mismatch")); }
                        let attempt_key = format!("attempt:{}", attempt.key);
                        if let Some(existing) = members.iter_mut().find(|member| member.key == attempt_key) {
                            existing.role = "shared_producer".into();
                            existing.links.push(ReviewInventoryLink { role: "waiter".into(), target: point_key.clone() });
                            if let Some(waiter) = members.iter_mut().find(|member| member.key == point_key) {
                                waiter.links.push(ReviewInventoryLink { role: "shared_producer".into(), target: attempt_key });
                            }
                            continue;
                        }
                        if members.len() >= request.maximum_members { truncated = true; break 'points; }
                        members.push(ReviewInventoryMember { key: attempt_key, role: "canonical_attempt".into(), run: Some(attempt.run), attempt: Some(attempt.key), operation: None, generation: Some(attempt.generation), occurrence: Some(point.ordinal), terminal: attempt.terminal, outcome: attempt.outcome, result_manifest: attempt.closed_manifest, readable_prefix: attempt.close_generation, links: vec![ReviewInventoryLink { role: "parent".into(), target: point_key.clone() }] });
                    }
                    if truncated { break 'points; }
                }
            }
            if scanned < study.point_count || truncated { gaps.push(ReviewInventoryGap { member: study.key.clone(), reason: "selected finite occurrence/member prefix omits later or growing membership".into() }); }
            gaps.push(ReviewInventoryGap { member: study.key.clone(), reason: "canonical occurrence/attempt membership only; result header references require their protected payload reader to establish current readability; cross-process operation, shared-flight and dynamic child links require their supplied relationship owners; page reads are not an atomic graph snapshot".into() });
            ReviewInventorySelection::from_members(study.key, request.attempts, members, gaps, truncated)
        }).await
    }
}

#[cfg(test)]
mod review_inventory_tests {
    use super::*;
    fn member(key: &str, attempt: &str, terminal: bool) -> ReviewInventoryMember {
        ReviewInventoryMember {
            key: key.into(),
            role: "native_child".into(),
            run: Some("same-name".into()),
            attempt: Some(attempt.into()),
            operation: None,
            generation: Some(u64::MAX),
            occurrence: None,
            terminal,
            outcome: terminal.then(|| "failed".into()),
            result_manifest: None,
            readable_prefix: Some(0),
            links: Vec::new(),
        }
    }
    #[test]
    fn review_inventory_pages_freeze_exact_retries_gaps_shared_waiters_and_empty_results() {
        let mut members = (0..300)
            .map(|n| {
                member(
                    &format!("child:{n}"),
                    &format!("attempt:{n}"),
                    n == 0 || n == 2,
                )
            })
            .collect::<Vec<_>>();
        members[0].result_manifest = Some("selected-empty-result".into());
        members[1].links = vec![
            ReviewInventoryLink {
                role: "shared_producer".into(),
                target: "child:0".into(),
            },
            ReviewInventoryLink {
                role: "parent".into(),
                target: "absent-parent".into(),
            },
        ];
        members[2].role = "horizon_background".into();
        let selected = ReviewInventorySelection::from_members(
            "frozen-prefix".into(),
            ReviewAttemptSelection::Retained,
            members.clone(),
            Vec::new(),
            true,
        )
        .unwrap();
        members[0].attempt = Some("convenient-current-root".into());
        let first = selected.page(0, 128).unwrap();
        let second = selected.page(first.next.unwrap(), 128).unwrap();
        let last = selected.page(second.next.unwrap(), 128).unwrap();
        assert_eq!(
            (
                first.members.len(),
                second.members.len(),
                last.members.len()
            ),
            (128, 128, 44)
        );
        assert_eq!(first.members[0].attempt.as_deref(), Some("attempt:0"));
        assert_eq!(first.members[0].readable_prefix, Some(0));
        assert!(first.members[0].terminal);
        assert!(!first.members[1].terminal);
        assert_eq!(first.members[2].role, "horizon_background");
        assert_eq!(first.gaps.len(), 1);
        assert_eq!(first.gaps[0].member, "absent-parent");
        assert!(last.next.is_none());
        assert!(last.truncated);
        let empty = selected.page(300, 12).unwrap();
        assert!(empty.members.is_empty());
        assert!(empty.next.is_none());
        let restored: ReviewInventorySelection =
            serde_json::from_slice(&serde_json::to_vec(&selected).unwrap()).unwrap();
        assert_eq!(restored.members[0].generation, Some(u64::MAX));
        assert!(selected.page(301, 12).is_err());
        assert!(selected.page(0, 0).is_err());
    }
    #[test]
    fn review_inventory_tail_gap_uses_observed_fence_and_respects_selected_prefix() {
        let gap = retry_fence_gap("exact-run", 3, Some(2), false).unwrap();
        assert_eq!(gap.member, "exact-run");
        assert!(gap.reason.contains("fence 3"));
        assert!(retry_fence_gap("exact-run", 3, Some(3), false).is_none());
        assert!(retry_fence_gap("exact-run", 3, Some(2), true).is_none());
        assert!(retry_fence_gap("exact-run", 3, None, false).is_some());
        assert!(retry_fence_gap("exact-run", 0, None, false).is_none());
    }
    #[test]
    fn review_inventory_current_and_retained_attempts_use_exact_identity() {
        let current = ReviewInventoryRequest {
            attempts: ReviewAttemptSelection::Current,
            maximum_members: 64,
        };
        let retained = ReviewInventoryRequest {
            attempts: ReviewAttemptSelection::Retained,
            maximum_members: 64,
        };
        assert!(current.selects_attempt(Some("run-a:attempt-2"), "run-a:attempt-2"));
        assert!(!current.selects_attempt(Some("run-a:attempt-2"), "run-a:attempt-1"));
        assert!(!current.selects_attempt(Some("run-a:attempt-2"), "run-b:attempt-2"));
        assert!(retained.selects_attempt(Some("run-a:attempt-2"), "run-a:attempt-1"));
        assert!(!current.selects_attempt(None, "run-a:attempt-1"));
        assert!(
            retained.selects_attempt(None, "run-a:attempt-1"),
            "pending occurrence can retain readable historical retries without acquiring a current assignment"
        );
    }
    #[test]
    fn review_inventory_rejects_duplicate_exact_identity_and_preserves_prehandle_selection() {
        assert!(
            ReviewInventorySelection::from_members(
                "scope".into(),
                ReviewAttemptSelection::Current,
                vec![member("one", "a", true), member("one", "b", false)],
                Vec::new(),
                false
            )
            .is_err()
        );
        let mut early = member("preparation-operation", "unused", true);
        early.run = None;
        early.attempt = None;
        early.role = "pre_handle_failure".into();
        early.outcome = Some("source error".into());
        let selected = ReviewInventorySelection::from_members(
            "earlier-failure".into(),
            ReviewAttemptSelection::Current,
            vec![early],
            Vec::new(),
            false,
        )
        .unwrap();
        assert!(selected.page(0, 1).unwrap().members[0].run.is_none());
    }
    #[test]
    fn review_inventory_ingress_requires_missing_target_gap_and_finite_links() {
        let mut first = member("one", "a", false);
        first.links.push(ReviewInventoryLink {
            role: "parent".into(),
            target: "missing".into(),
        });
        let selected = ReviewInventorySelection::from_members(
            "partial".into(),
            ReviewAttemptSelection::Retained,
            vec![first],
            Vec::new(),
            false,
        )
        .unwrap();
        let mut raw: ReviewInventorySelection =
            serde_json::from_slice(&serde_json::to_vec(&selected).unwrap()).unwrap();
        raw.gaps.clear();
        assert!(raw.validate().is_err());
        assert!(raw.page(0, 1).is_err());
        raw.gaps = selected.gaps.clone();
        raw.validate().unwrap();
        raw.members[0].links = vec![
            ReviewInventoryLink {
                role: "self".into(),
                target: "one".into()
            };
            16385
        ];
        assert!(raw.validate().is_err());
    }
    #[test]
    fn review_inventory_projects_actual_operation_relationship_and_performing_owners() {
        use pse_diagnostics::observation::{ObservationHub, Outcome, ProducerRole, Relationship};
        let hub = ObservationHub::new(Default::default());
        let root = hub.start("fit.execute");
        let child = root.context().child("fit.experiment", Relationship::Nested);
        let child_reader = child.reader();
        let child_id = child.context().id();
        let active = root
            .context()
            .child("horizon.background", Relationship::Continuation);
        active
            .context()
            .link(&child.context(), Relationship::SharedProduction);
        let missing = root
            .context()
            .child("dynamic.backward", Relationship::Nested);
        let native = child
            .context()
            .producer(ProducerRole::NativeIteration, "actual.native");
        let record = native.record("native.control.v1", "return", None, None, vec![]);
        native.publish(&record);
        child.finish(Outcome::Failed);
        let snapshots = [
            root.reader().snapshot(std::time::Duration::from_secs(5)),
            child_reader.snapshot(std::time::Duration::from_secs(5)),
            active.reader().snapshot(std::time::Duration::from_secs(5)),
        ];
        let selected = ReviewInventorySelection::from_operations(
            "actual-owner-copies".into(),
            ReviewAttemptSelection::Retained,
            snapshots.iter(),
            64,
        )
        .unwrap();
        assert!(
            selected
                .members
                .iter()
                .any(|member| member.operation == Some(child_id)
                    && member.terminal
                    && member.outcome.as_deref() == Some("Failed"))
        );
        assert!(selected.members.iter().any(|member| {
            member.role == "horizon.background"
                && !member.terminal
                && member.links.iter().any(|link| {
                    link.role == "SharedProduction"
                        && link.target == format!("operation:{child_id}")
                })
        }));
        assert!(
            selected
                .members
                .iter()
                .any(|member| member.key.starts_with("producer:")
                    && member.role.starts_with("NativeIteration:"))
        );
        assert!(
            selected
                .gaps
                .iter()
                .any(|gap| gap.member == format!("operation:{}", missing.context().id()))
        );
        let prefix = ReviewInventorySelection::from_operations(
            "bounded".into(),
            ReviewAttemptSelection::Retained,
            snapshots.iter(),
            1,
        )
        .unwrap();
        assert!(prefix.truncated);
        assert_eq!(prefix.members.len(), 1);
        prefix.validate().unwrap();
        assert!(
            ReviewInventorySelection::from_operations(
                "duplicate".into(),
                ReviewAttemptSelection::Retained,
                [&snapshots[0], &snapshots[0]],
                64
            )
            .is_err()
        );
    }
}
