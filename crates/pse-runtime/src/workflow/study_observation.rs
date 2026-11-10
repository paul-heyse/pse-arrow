// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive bounded study telemetry: exact per-attempt streams, never a study execution owner.
use super::{
    ProgressCheckpointDocument, ProgressEventDocument, RetainedDataServices, Runtime,
    StudyCohortHeaderRequest, StudyCohortMemberPage, StudyCohortPageRequest, StudyCohortReceipt,
    StudyCohortRequest, StudyWorkbenchFilter, StudyWorkbenchHeader, WorkflowError, contract,
};
use pse_columnar::CancellationToken;
use pse_contract::boundary_projection::{Direction, LocalRule, ValidateProjection as _};
#[cfg(test)]
use std::time::Duration;
use std::{collections::BTreeMap, sync::Arc};
const OBSERVED_MEMBERS: LocalRule = LocalRule::UnsignedRange { min: 1, max: 64 };
const OBSERVED_EVENTS: LocalRule = LocalRule::UnsignedRange { min: 1, max: 512 };
const MAXIMUM_OBSERVED_SOURCES: usize = 64;
const OBSERVED_SOURCES: LocalRule = LocalRule::Length {
    min: 0,
    max: MAXIMUM_OBSERVED_SOURCES as u64,
};
const OBSERVED_EVENT_OFFSET: LocalRule = LocalRule::UnsignedRange { min: 0, max: 512 };
fn observed_members_schema(schema: &mut schemars::Schema) {
    use schemars::transform::Transform as _;
    let mut rule = OBSERVED_MEMBERS;
    rule.transform(schema);
}
fn observed_events_schema(schema: &mut schemars::Schema) {
    use schemars::transform::Transform as _;
    let mut rule = OBSERVED_EVENTS;
    rule.transform(schema);
}
fn observed_sources_schema(schema: &mut schemars::Schema) {
    use schemars::transform::Transform as _;
    let mut rule = OBSERVED_SOURCES;
    rule.transform(schema);
}
fn observed_offset_schema(schema: &mut schemars::Schema) {
    use schemars::transform::Transform as _;
    let mut rule = OBSERVED_EVENT_OFFSET;
    rule.transform(schema);
}
pse_contract::projection_struct! {
/// Selected study and bounded observation allowance. These controls never govern science.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationRequest {
    projection: { name: "StudyObservationRequest", rust_path: crate::workflow::StudyObservationRequest,
        version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact existing study key.
    pub study: String => { tag: 1, direction: Direction::Both, rules: [] };
    /// Maximum complete selected point population, from one through 64.
    #[schemars(transform = observed_members_schema)]
    pub maximum_members: usize => { tag: 2, direction: Direction::Both, rules: [OBSERVED_MEMBERS] };
    /// Maximum events returned per poll, from one through 512.
    #[schemars(transform = observed_events_schema)]
    pub maximum_events: usize => { tag: 3, direction: Direction::Both, rules: [OBSERVED_EVENTS] };
}
}
impl StudyObservationRequest {
    pub(crate) fn validate(&self) -> Result<(), WorkflowError> {
        self.validate_projection().map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })
    }
}
pse_contract::projection_struct! {
/// Original immutable selection binds a cursor independently from mutable metadata revisions.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationContext {
    projection: { name: "StudyObservationContext", rust_path: crate::workflow::StudyObservationContext,
        version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact parent execution.
    pub run: String => { tag: 1, direction: Direction::Both, rules: [] };
    /// Exact original source revision.
    pub source: String => { tag: 2, direction: Direction::Both, rules: [] };
    /// Exact original physical source.
    pub physical_revision: Option<String> => { tag: 3, direction: Direction::Both, rules: [] };
    /// Original physical interpretation.
    pub physical_identity: Option<String> => { tag: 4, direction: Direction::Both, rules: [] };
    /// Original frozen engineering context.
    pub context: Option<String> => { tag: 5, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Position in one exact attempt, with separate native source and durable batch clocks.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationPosition {
    projection: { name: "StudyObservationPosition", rust_path: crate::workflow::StudyObservationPosition,
        version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact original execution.
    pub run: String => { tag: 1, direction: Direction::Both, rules: [] };
    /// Exact original attempt; retries create another position.
    pub attempt: String => { tag: 2, direction: Direction::Both, rules: [] };
    /// Actual canonical attempt generation.
    pub attempt_generation: u64 => { tag: 3, direction: Direction::Both, rules: [] };
    /// Actual point metadata revision, absent for parent summary.
    pub metadata_revision: Option<u64> => { tag: 4, direction: Direction::Both, rules: [] };
    /// Actual numerical source generation, unavailable for remote live observations.
    pub source_generation: Option<u64> => { tag: 5, direction: Direction::Both, rules: [] };
    /// Last actually observed source sequence; never a timestamp.
    pub source_sequence: u64 => { tag: 6, direction: Direction::Both, rules: [] };
    /// Next immutable terminal progress batch.
    pub batch: u64 => { tag: 7, direction: Direction::Both, rules: [] };
    /// Next event inside that batch.
    #[schemars(transform = observed_offset_schema)]
    pub offset: usize => { tag: 8, direction: Direction::Both, rules: [OBSERVED_EVENT_OFFSET] };
    /// Highest consumed durable append sequence, distinct from source sequence.
    pub durable_sequence: Option<i64> => { tag: 9, direction: Direction::Both, rules: [] };
    /// Terminal history was exhausted under its original reader.
    pub history_complete: bool => { tag: 10, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Portable reconnect coordinates; decoding grants no result or cancellation authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationCursor {
    projection: { name: "StudyObservationCursor", rust_path: crate::workflow::StudyObservationCursor,
        version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact canonical issuer.
    pub issuing_store: String => { tag: 1, direction: Direction::Both, rules: [] };
    /// Exact original observation selection and limits.
    pub request: StudyObservationRequest => { tag: 2, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Exact scientific and engineering context.
    pub context: StudyObservationContext => { tag: 3, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Original parent and retry sources; older attempts remain present.
    #[schemars(transform = observed_sources_schema)]
    pub sources: Vec<StudyObservationPosition> => { tag: 4, direction: Direction::Both, rules: [OBSERVED_SOURCES] };
}
}
pse_contract::projection_enum! {
/// Explicit continuity statement for one selected source in this poll.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum StudyObservationContinuity {
    projection: { name: "StudyObservationContinuity", rust_path: crate::workflow::StudyObservationContinuity,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Actual contiguous available source history after the captured position.
    CatchUp => 1;
    /// Prefix or intervening samples are unavailable; current state is a resynchronization.
    GapCurrentResync => 2;
    /// No local numeric telemetry owner exists for this ongoing remote attempt.
    LiveNumericsUnavailable => 3;
    /// Selected attempt has no retained numerical history.
    HistoryUnavailable => 4;
}
}
pse_contract::projection_struct! {
/// A source-local page; ordering across sources has no global causal interpretation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservedSource {
    projection: { name: "StudyObservedSource", rust_path: crate::workflow::StudyObservedSource,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact original run/attempt position after this page.
    pub position: StudyObservationPosition => { tag: 1, direction: Direction::Output, rules: [] };
    /// Actual source continuity, independent from scientific outcome.
    pub continuity: StudyObservationContinuity => { tag: 2, direction: Direction::Output, rules: [] };
    /// Original whole numerical observations in their original source order.
    pub events: Vec<ProgressEventDocument> => { tag: 3, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 512 }] };
    /// Current live checkpoint when the local original owner is attached.
    pub checkpoint: Option<ProgressCheckpointDocument> => { tag: 4, direction: Direction::Output, rules: [] };
    /// Original terminal writer coverage when available.
    pub coverage: Option<super::ProgressCoverage> => { tag: 5, direction: Direction::Output, rules: [] };
    /// Original canonical refusal when retained terminal history is unavailable.
    pub history_refusal: Option<String> => { tag: 6, direction: Direction::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// One explicitly bounded observer status page; never a complete analytical document.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationStatus {
    projection: { name: "StudyObservationStatus", rust_path: crate::workflow::StudyObservationStatus,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact historical native cut and counts.
    pub receipt: StudyCohortReceipt => { tag: 1, direction: Direction::Output, rules: [] };
    /// The one explicitly selected study header.
    pub header: StudyWorkbenchHeader => { tag: 2, direction: Direction::Output, rules: [] };
    /// One bounded original member page (observer admission remains at64).
    pub members: StudyCohortMemberPage => { tag: 3, direction: Direction::Output, rules: [] };
    @local {
    #[serde(skip)]
    #[schemars(skip)]
    _custody: Option<Arc<dyn std::fmt::Debug + Send + Sync>> = None
        => check: |_, _| Ok::<(), pse_contract::boundary_projection::ValidationError>(());
    }
}
}
async fn status_page(
    cohort: super::FrozenStudyCohort,
) -> Result<StudyObservationStatus, WorkflowError> {
    let headers = cohort
        .header_page(StudyCohortHeaderRequest {
            maximum: 1,
            cursor: None,
        })
        .await?;
    let members = cohort
        .member_page(StudyCohortPageRequest {
            maximum: 64,
            cursor: None,
        })
        .await?;
    if cohort.receipt().header_count != 1 || headers.next.is_some() || members.next.is_some() {
        return Err(contract(
            "selected observation bounded population incomplete",
        ));
    }
    let header = headers
        .headers
        .into_iter()
        .next()
        .ok_or_else(|| contract("selected observation study absent"))?
        .value;
    Ok(StudyObservationStatus {
        receipt: cohort.receipt().clone(),
        header,
        members,
        _custody: Some(Arc::new((headers._owner, cohort))),
    })
}
pse_contract::projection_struct! {
/// One bounded read-only poll and its exact reconnect position.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationPage {
    projection: { name: "StudyObservationPage", rust_path: crate::workflow::StudyObservationPage,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Canonical compact status with actual point metadata revisions.
    pub status: StudyObservationStatus => { tag: 1, direction: Direction::Output, rules: [] };
    /// Bounded selected owner observations.
    pub sources: Vec<StudyObservedSource> => { tag: 2, direction: Direction::Output, rules: [OBSERVED_SOURCES] };
    /// Exact next reconnect position.
    pub cursor: StudyObservationCursor => { tag: 3, direction: Direction::Output, rules: [] };
}
}
impl StudyObservationPage {
    /// Retain the actual native frozen cut and selected metadata while encoding this page.
    /// A decoded portable page has no original custody and gains no result capability here.
    #[must_use]
    pub fn payload_owner(&self) -> Option<Arc<dyn pse_columnar::PayloadOwner>> {
        if self.status._custody.is_none() && self.status.members.payload_owner().is_none() {
            return None;
        }
        Some(Arc::new((
            self.status._custody.clone(),
            self.status.members.payload_owner(),
        )))
    }
}
/// Observer resources contain no study handle, worker, cancellation lease or dispatch route.
#[derive(Debug)]
pub struct StudyObserver {
    services: RetainedDataServices,
    cursor: StudyObservationCursor,
    live: BTreeMap<(String, String), Arc<pse_backend_native::solve::Progress>>,
    cancel: CancellationToken,
    closed: bool,
}
#[cfg(all(test, feature = "engineering-service", feature = "solver-kinsol"))]
mod capture_probe {
    use super::StudyCohortReceipt;
    use datafusion::execution::memory_pool::MemoryReservation;
    use std::{
        collections::BTreeMap,
        sync::{Mutex, Weak},
    };
    use tokio::sync::oneshot;

    pub(crate) type Ingress = Weak<MemoryReservation>;
    tokio::task_local! {
        pub(super) static INGRESS: Ingress;
    }
    pub(crate) struct CaptureProbe {
        pub(crate) entered: oneshot::Receiver<(StudyCohortReceipt, Ingress)>,
        pub(crate) resume: oneshot::Sender<()>,
        pub(crate) completed: oneshot::Receiver<String>,
    }
    struct Gate {
        entered: oneshot::Sender<(StudyCohortReceipt, Ingress)>,
        resume: oneshot::Receiver<()>,
        completed: oneshot::Sender<String>,
    }
    static GATES: Mutex<BTreeMap<String, Gate>> = Mutex::new(BTreeMap::new());

    pub(super) fn install(study: String) -> CaptureProbe {
        let (entered, observe_entry) = oneshot::channel();
        let (allow_resume, resume) = oneshot::channel();
        let (completed, observe_completion) = oneshot::channel();
        let previous = GATES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                study,
                Gate {
                    entered,
                    resume,
                    completed,
                },
            );
        assert!(previous.is_none(), "one gate for the original Study");
        CaptureProbe {
            entered: observe_entry,
            resume: allow_resume,
            completed: observe_completion,
        }
    }

    pub(super) async fn pause(
        receipt: &StudyCohortReceipt,
        study: &str,
    ) -> Option<oneshot::Sender<String>> {
        let gate = GATES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(study)?;
        let ingress = INGRESS.try_with(Weak::clone).ok()?;
        let _observed = gate.entered.send((receipt.clone(), ingress));
        // A failed test drops the sender and lets the original owner settle.
        let _resumed = gate.resume.await;
        Some(gate.completed)
    }
}

fn context(document: &StudyObservationStatus) -> Result<StudyObservationContext, WorkflowError> {
    let h = &document.header;
    Ok(StudyObservationContext {
        run: h.run.clone(),
        source: h.source.clone(),
        physical_revision: h.physical_revision.clone(),
        physical_identity: h.physical_identity.clone(),
        context: h.context.clone(),
    })
}
impl Runtime {
    /// Observe existing study state using this caller's retained-data capabilities.
    pub async fn observe_study(
        &self,
        request: StudyObservationRequest,
        cursor: Option<StudyObservationCursor>,
    ) -> Result<StudyObserver, WorkflowError> {
        self.retained_data_services()
            .observe_study(request, cursor)
            .await
    }
}
impl RetainedDataServices {
    /// Open or reconnect selected existing owners. This performs no coordination or science.
    pub async fn observe_study(
        &self,
        request: StudyObservationRequest,
        cursor: Option<StudyObservationCursor>,
    ) -> Result<StudyObserver, WorkflowError> {
        request.validate()?;
        let status = self
            .freeze_study_cohort(StudyCohortRequest {
                filter: StudyWorkbenchFilter {
                    study: Some(request.study.clone()),
                    ..Default::default()
                },
                maximum_studies: 1,
                maximum_members: request.maximum_members,
            })
            .await?;
        let status = status_page(status).await?;
        let selected = context(&status)?;
        let issuer = self.canonical_store()?.issuing_store()?.to_owned();
        let cursor = match cursor {
            Some(cursor) => {
                cursor.validate_projection().map_err(|error| {
                    WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
                })?;
                if cursor.issuing_store != issuer
                    || cursor.request != request
                    || cursor.context != selected
                {
                    return Err(contract(
                        "observer cursor issuer or exact selection differs",
                    ));
                }
                let mut keys = std::collections::BTreeSet::new();
                for source in &cursor.sources {
                    if !keys.insert((&source.run, &source.attempt))
                        || source.run != selected.run
                            && !status
                                .members
                                .members
                                .iter()
                                .any(|p| p.value.run == source.run)
                    {
                        return Err(contract(
                            "observer cursor includes foreign or malformed source",
                        ));
                    }
                    let attempts = self
                        .canonical_store()?
                        .canonical_attempt_page(
                            &source.run,
                            source.attempt_generation.checked_sub(1),
                            source.attempt_generation,
                        )
                        .await?;
                    if attempts.len() != 1
                        || attempts[0].key != source.attempt
                        || attempts[0].run != source.run
                        || attempts[0].generation != source.attempt_generation
                    {
                        return Err(contract("observer cursor attempt attribution differs"));
                    }
                }
                cursor
            }
            None => StudyObservationCursor {
                issuing_store: issuer,
                request,
                context: selected,
                sources: Vec::new(),
            },
        };
        Ok(StudyObserver {
            services: self.clone(),
            cursor,
            live: BTreeMap::new(),
            cancel: CancellationToken::new(),
            closed: false,
        })
    }
}
impl StudyObserver {
    #[cfg(all(test, feature = "engineering-service", feature = "solver-kinsol"))]
    pub(crate) fn probe_next_capture(study: String) -> capture_probe::CaptureProbe {
        capture_probe::install(study)
    }

    #[cfg(all(test, feature = "engineering-service", feature = "solver-kinsol"))]
    pub(crate) async fn with_ingress_probe<T>(
        ingress: std::sync::Weak<datafusion::execution::memory_pool::MemoryReservation>,
        work: impl Future<Output = T>,
    ) -> T {
        capture_probe::INGRESS.scope(ingress, work).await
    }

    /// Cancellation scope belongs only to observation, never to scientific execution.
    pub fn observation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }
    /// Capture portable coordinates without affecting execution or observation resources.
    pub fn cursor(&self) -> StudyObservationCursor {
        self.cursor.clone()
    }
    /// Attach the original local telemetry owner without retaining its cancellation lease.
    pub fn attach_run(&mut self, handle: &super::RunHandle) -> Result<(), WorkflowError> {
        if self.closed {
            return Err(contract("study observer closed"));
        }
        let (run, attempt, progress) = handle
            .study_progress_source()
            .ok_or_else(|| contract("study observer requires a canonical local source"))?;
        if !self
            .cursor
            .sources
            .iter()
            .any(|s| s.run == run && s.attempt == attempt)
        {
            return Err(contract(
                "local telemetry source is outside current selected study; poll to discover its attempt",
            ));
        }
        self.live.insert((run, attempt), progress);
        Ok(())
    }
    /// Release observation only. No study cancellation, draining, dispatch or coordination occurs.
    pub fn close(&mut self) {
        self.closed = true;
        self.cancel.cancel();
        self.live.clear();
    }
    /// Poll only this complete selected bounded study and its existing original attempt owners.
    pub async fn poll(&mut self) -> Result<StudyObservationPage, WorkflowError> {
        let activity = self.services.admit_activity()?;
        activity.scope(self.poll_checked()).await
    }
    async fn poll_checked(&mut self) -> Result<StudyObservationPage, WorkflowError> {
        if self.closed {
            return Err(contract("study observer closed"));
        }
        let status = self
            .services
            .freeze_study_cohort(StudyCohortRequest {
                filter: StudyWorkbenchFilter {
                    study: Some(self.cursor.request.study.clone()),
                    ..Default::default()
                },
                maximum_studies: 1,
                maximum_members: self.cursor.request.maximum_members,
            })
            .await?;
        #[cfg(all(test, feature = "engineering-service", feature = "solver-kinsol"))]
        let capture_probe =
            capture_probe::pause(status.receipt(), &self.cursor.request.study).await;
        let status = status_page(status).await?;
        if context(&status)? != self.cursor.context {
            return Err(contract("observer selected context changed"));
        }
        let mut runs = vec![(self.cursor.context.run.clone(), None)];
        runs.extend(
            status
                .members
                .members
                .iter()
                .map(|p| (p.value.run.clone(), Some(p.value.metadata_revision))),
        );
        let mut attempts = Vec::new();
        for (run, revision) in runs {
            let Some(generation) = self
                .services
                .canonical_store()?
                .workbench_run_generation(&run)
                .await?
            else {
                continue;
            };
            let page = self
                .services
                .canonical_store()?
                .canonical_attempt_page(&run, None, generation)
                .await?;
            // Refuse an incomplete retry population rather than silently dropping earlier attempts.
            if page.len() == MAXIMUM_OBSERVED_SOURCES
                || attempts.len() + page.len() > MAXIMUM_OBSERVED_SOURCES
            {
                return Err(contract(
                    "observer attempt population exceeds bound; narrower observation required",
                ));
            }
            for attempt in page {
                attempts.push((attempt, revision));
            }
        }
        for (attempt, revision) in &attempts {
            if let Some(position) = self
                .cursor
                .sources
                .iter_mut()
                .find(|p| p.run == attempt.run && p.attempt == attempt.key)
            {
                position.metadata_revision = *revision;
            } else {
                self.cursor.sources.push(StudyObservationPosition {
                    run: attempt.run.clone(),
                    attempt: attempt.key.clone(),
                    attempt_generation: attempt.generation,
                    metadata_revision: *revision,
                    source_generation: None,
                    source_sequence: 0,
                    batch: 0,
                    offset: 0,
                    durable_sequence: None,
                    history_complete: false,
                });
            }
        }
        let mut output = Vec::new();
        let mut remaining = self.cursor.request.maximum_events;
        for (attempt, _) in attempts {
            let parent = attempt.run == self.cursor.context.run;
            let position = self
                .cursor
                .sources
                .iter_mut()
                .find(|p| p.run == attempt.run && p.attempt == attempt.key)
                .ok_or_else(|| contract("observed attempt position absent"))?;
            let mut events = Vec::new();
            let mut checkpoint = None;
            let mut coverage = None;
            let mut history_refusal = None;
            let mut continuity = StudyObservationContinuity::CatchUp;
            let summary = if parent && attempt.terminal {
                let original = self
                    .services
                    .canonical_store()?
                    .canonical_attempt(&attempt.key)
                    .await?
                    .ok_or_else(|| contract("observed parent attempt absent"))?;
                if original.run != attempt.run
                    || original.key != attempt.key
                    || !original.terminal()
                {
                    return Err(contract("observed parent attempt identity differs"));
                }
                original
                    .completion
                    .as_ref()
                    .map(|bytes| super::study::summary_conclusion(bytes.as_slice()))
                    .transpose()?
                    .flatten()
            } else {
                None
            };
            if summary.is_some() {
                // A parent joins point outcomes; it never ran numerical callbacks.
                // The selected study status retains its original parent outcome,
                // while absent progress counters must not become sampled coverage.
                coverage = Some(super::ProgressCoverage::Unknown {});
                continuity = StudyObservationContinuity::HistoryUnavailable;
                position.history_complete = true;
            } else if attempt.terminal && !position.history_complete && remaining > 0 {
                let history = match self
                    .services
                    .progress(&position.run, &position.attempt, self.cancel.child_token())
                    .await
                {
                    Ok(history) => Some(history),
                    Err(error) => match unavailable_history(&error) {
                        Some(code) => {
                            history_refusal = Some(code);
                            continuity = StudyObservationContinuity::HistoryUnavailable;
                            None
                        }
                        None => return Err(error),
                    },
                };
                if let Some(mut history) = history {
                    coverage = Some(history.coverage().clone());
                    history.resume_batch(position.batch)?;
                    if let Some(page) = history.next_page().await? {
                        let length = page.len();
                        let begin = position.offset;
                        if begin > length {
                            return Err(contract("observer event offset exceeds original batch"));
                        }
                        let finish = length.min(begin + remaining);
                        for event in page.into_iter().skip(begin).take(finish - begin) {
                            position.durable_sequence = event.sequence;
                            if let Some(gap) = consume(position, &event) {
                                if gap {
                                    continuity = StudyObservationContinuity::GapCurrentResync;
                                }
                                events.push(event);
                            }
                        }
                        remaining -= finish - begin;
                        if finish == length {
                            position.batch = history.batch_position();
                            position.offset = 0;
                        } else {
                            position.offset = finish;
                        }
                    } else {
                        position.history_complete = true;
                        if matches!(history.coverage(), super::ProgressCoverage::Unknown {}) {
                            continuity = StudyObservationContinuity::HistoryUnavailable;
                        }
                    }
                    history.close();
                }
            } else if !attempt.terminal {
                if let Some(progress) = self
                    .live
                    .get(&(position.run.clone(), position.attempt.clone()))
                {
                    let (page, dropped) = progress.snapshot_after(
                        position.source_generation,
                        position.source_sequence,
                        remaining,
                    );
                    let current = progress.current().map(ProgressEventDocument::from);
                    let (generation, head) = progress.source_identity();
                    let latest = current
                        .as_ref()
                        .and_then(|e| e.source.as_ref())
                        .is_some_and(|s| s.generation == generation && s.sequence == head);
                    checkpoint = Some(ProgressCheckpointDocument {
                        version: pse_model::document::Version,
                        current,
                        generation,
                        highest_issued_sequence: head,
                        dropped,
                        current_is_latest: latest,
                    });
                    for event in page.into_iter().map(ProgressEventDocument::from) {
                        if events.len() == remaining {
                            break;
                        }
                        if let Some(gap) = consume(position, &event) {
                            if gap {
                                continuity = StudyObservationContinuity::GapCurrentResync;
                            }
                            events.push(event);
                        }
                    }
                    remaining = remaining.saturating_sub(events.len());
                    if events.is_empty() && head > position.source_sequence {
                        continuity = StudyObservationContinuity::GapCurrentResync;
                    }
                } else {
                    continuity = StudyObservationContinuity::LiveNumericsUnavailable;
                }
            }
            output.push(StudyObservedSource {
                position: position.clone(),
                continuity,
                events,
                checkpoint,
                coverage,
                history_refusal,
            });
        }
        #[cfg(all(test, feature = "engineering-service", feature = "solver-kinsol"))]
        if let Some(completed) = capture_probe {
            let _observed = completed.send(status.receipt.capture.clone());
        }
        Ok(StudyObservationPage {
            status,
            sources: output,
            cursor: self.cursor.clone(),
        })
    }
}
pub(super) fn unavailable_history(error: &WorkflowError) -> Option<String> {
    use pse_operations::canonical::CanonicalError;
    use pse_schema::catalog::canonical_refusals::RefusalCode as C;
    match error {
        WorkflowError::Canonical(CanonicalError::Refused { code, .. })
            if matches!(
                code,
                C::ExecutionResultsExplicitlyRetired
                    | C::ResultProtectionRetiredOrManifestUnavailable
                    | C::TerminalResultSelectionUnavailable
                    | C::HistoryExplicitlyRetiredDetailedResponseUnavailable
            ) =>
        {
            Some(code.wire().to_owned())
        }
        _ => None,
    }
}
impl Drop for StudyObserver {
    fn drop(&mut self) {
        self.close();
    }
}
/// Source sequence alone owns continuity; append sequence and wall time never match events.
pub(super) fn consume(
    position: &mut StudyObservationPosition,
    event: &ProgressEventDocument,
) -> Option<bool> {
    let Some(source) = &event.source else {
        return Some(true);
    };
    if position.source_generation == Some(source.generation)
        && source.sequence <= position.source_sequence
    {
        return None;
    }
    let gap = if position.source_generation == Some(source.generation) {
        source.sequence != position.source_sequence + 1
    } else {
        position.source_generation.is_some() || source.sequence != 1
    };
    position.source_generation = Some(source.generation);
    position.source_sequence = source.sequence;
    Some(gap)
}
#[cfg(test)]
mod tests {
    use super::*;
    use pse_backend_native::solve::{Event, EventSource, Progress};
    fn position() -> StudyObservationPosition {
        StudyObservationPosition {
            run: "run".into(),
            attempt: "attempt".into(),
            attempt_generation: 1,
            metadata_revision: Some(9),
            source_generation: None,
            source_sequence: 0,
            batch: 0,
            offset: 0,
            durable_sequence: None,
            history_complete: false,
        }
    }
    fn event(progress: &Progress) -> ProgressEventDocument {
        let mut e = Event {
            source: Some(EventSource::new("test", None, None)),
            phase: "step".into(),
            elapsed: Duration::ZERO,
            values: Default::default(),
            incumbent: None,
        };
        progress.push(e.clone());
        e = progress.current().unwrap();
        ProgressEventDocument::from(e)
    }
    #[test]
    fn study_observer_reconnect_uses_exact_source_sequences_and_reports_missing_prefix() {
        let progress = Progress::new(8);
        let first = event(&progress);
        let second = event(&progress);
        let third = event(&progress);
        let mut original = position();
        assert_eq!(consume(&mut original, &first), Some(false));
        let mut reconnected = original.clone();
        assert_eq!(consume(&mut reconnected, &first), None);
        assert_eq!(consume(&mut reconnected, &second), Some(false));
        assert_eq!(consume(&mut reconnected, &third), Some(false));
        let mut gap = original;
        assert_eq!(consume(&mut gap, &third), Some(true));
        assert_eq!(gap.source_sequence, 3);
        let mut absent_prefix = position();
        assert_eq!(consume(&mut absent_prefix, &second), Some(true));
        // Equal wall times cannot turn a different generation into continuity.
        let other = Progress::new(2);
        let replacement = event(&other);
        assert_eq!(consume(&mut gap, &replacement), Some(true));
    }
    #[test]
    fn study_observer_retains_telemetry_without_acquiring_scientific_cancellation() {
        let progress = Arc::new(Progress::new(1));
        let mut live = BTreeMap::<(String, String), Arc<Progress>>::new();
        live.insert(("run".into(), "attempt".into()), progress.clone());
        drop(live);
        assert_eq!(event(&progress).source.unwrap().sequence, 1);
        assert_eq!(event(&progress).source.unwrap().sequence, 2);
        let (retained, dropped) = progress.snapshot();
        assert_eq!(retained.len(), 1);
        assert_eq!(dropped, 1);
    }
}
#[cfg(test)]
#[path = "reader-history-tests.rs"]
mod reader_history_tests;

#[cfg(test)]
#[allow(
    unsafe_code,
    reason = "controlled original sealed progress fixture admits partial outcome only"
)]
mod canonical_tests {
    use super::*;
    use pse_backend_native::solve::{Event, EventSource, Progress};
    use pse_model::{generated::enums::AttemptState, study::OccurrenceKey};
    use pse_operations::{
        canonical_execution::{ResultPayload, SeedEligibility, TerminalClass},
        canonical_studies::point_key,
    };
    #[tokio::test]
    async fn study_observer_terminal_parent_conclusion_has_no_numeric_history() {
        let runtime = crate::workflow::durable_tests::durable_runtime();
        let physical = crate::workflow::study_workbench::canonical_tests::physical(&runtime).await;
        let store = runtime.canonical_store().unwrap();
        let source = store
            .edit("observed-summary-source", None, "source", &[])
            .await
            .unwrap();
        let study = pse_operations::mint_id::<pse_ids::SemanticId>().to_string();
        crate::workflow::study_workbench::canonical_tests::create(
            &runtime, &study, &source, &physical, 1,
        )
        .await;
        // Effect-free cancellation settles the point without constructing a
        // scientific completion or a numerical progress stream for either run.
        store.cancel_study(&study).await.unwrap();
        let scope = store
            .study_scope(&point_key(&study, OccurrenceKey(0)))
            .await
            .unwrap();
        store
            .refuse_study_point(&scope, &scope.action(None).unwrap(), None)
            .await
            .unwrap();
        assert!(runtime.finalize_canonical_study(&study).await.unwrap());
        let mut observer = runtime
            .observe_study(
                StudyObservationRequest {
                    study: study.clone(),
                    maximum_members: 1,
                    maximum_events: 1,
                },
                None,
            )
            .await
            .unwrap();
        for _ in 0..2 {
            let page = observer.poll().await.unwrap();
            assert_eq!(page.status.header.study, study);
            assert!(page.status.header.cancelled);
            assert!(page.status.header.terminal);
            assert_eq!(page.status.receipt.member_count, 1);
            let parent = page
                .sources
                .iter()
                .find(|source| source.position.run == page.status.header.run)
                .unwrap();
            assert_eq!(
                parent.continuity,
                StudyObservationContinuity::HistoryUnavailable
            );
            assert!(parent.position.history_complete);
            assert!(parent.events.is_empty());
            assert!(parent.checkpoint.is_none());
            assert!(matches!(
                parent.coverage,
                Some(crate::workflow::ProgressCoverage::Unknown {})
            ));
            assert!(parent.history_refusal.is_none());
        }
        drop(observer);
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn study_observer_public_disconnect_reconnect_reads_full_history_or_reports_missing_prefix()
     {
        for missing_prefix in [false, true] {
            let runtime = crate::workflow::durable_tests::durable_runtime();
            let physical =
                crate::workflow::study_workbench::canonical_tests::physical(&runtime).await;
            let store = runtime.canonical_store().unwrap();
            let source = store
                .edit("observed-source", None, "source", &[])
                .await
                .unwrap();
            crate::workflow::study_workbench::canonical_tests::create(
                &runtime, "observed", &source, &physical, 1,
            )
            .await;
            let scope = store
                .study_scope(&point_key("observed", OccurrenceKey(0)))
                .await
                .unwrap();
            let claim = store
                .claim_study_point(&scope, None, "claim", "worker", Duration::from_secs(60))
                .await
                .unwrap();
            let request = StudyObservationRequest {
                study: "observed".into(),
                maximum_members: 2,
                maximum_events: 1,
            };
            let progress = Arc::new(Progress::new(8));
            let mut original_events = Vec::new();
            for _ in 0..3 {
                progress.push(Event {
                    source: Some(EventSource::new("fixture", None, None)),
                    phase: "step".into(),
                    elapsed: Duration::ZERO,
                    values: Default::default(),
                    incumbent: None,
                });
                original_events.push(ProgressEventDocument::from(progress.current().unwrap()));
            }
            let mut observer = runtime.observe_study(request.clone(), None).await.unwrap();
            observer.poll().await.unwrap();
            // Inject only the real passive Progress owner in this storage control;
            // production attach_run obtains this same Arc without the run's lease.
            observer.live.insert(
                (claim.point.run.clone(), claim.fence.attempt().to_owned()),
                progress,
            );
            let first = observer.poll().await.unwrap();
            assert_eq!(
                first.sources[0].events[0].source.as_ref().unwrap().sequence,
                1
            );
            let cursor = observer.cursor();
            observer.close();
            drop(observer);
            let selected = if missing_prefix {
                original_events[2..].to_vec()
            } else {
                original_events
            };
            let selected = selected
                .into_iter()
                .enumerate()
                .map(|(i, mut event)| {
                    event.sequence = Some(i as i64 + 1);
                    event
                })
                .collect::<Vec<_>>();
            store
                .append_result_batch(
                    &claim.fence,
                    "progress",
                    "__progress",
                    0,
                    ResultPayload::new(serde_json::to_vec(&selected).unwrap()).unwrap(),
                    selected.len() as u64,
                )
                .await
                .unwrap();
            let stored = crate::workflow::StoredCompletion {
                version: 4,
                attempt_id: pse_operations::mint_id(),
                completion: Some(Default::default()),
                termination: crate::workflow::TerminationDetail {
                    version: pse_model::document::Version,
                    cause: crate::workflow::TerminationCause::Assessment {
                        usable: false,
                        candidate_use: vec![],
                        diagnostics: vec![],
                    },
                    point: None,
                    retry_failure: None,
                    effect: pse_model::study::EffectState::Absent,
                },
                state: AttemptState::Partial,
                progress: crate::workflow::ProgressCoverage::Sampled {
                    observed: 3,
                    queued: selected.len() as u64,
                    dropped_before_queue: if missing_prefix { 2 } else { 0 },
                    acknowledged: selected.len() as u64,
                    unacknowledged: 0,
                    writer: crate::workflow::ProgressWriterOutcome::Complete,
                },
            };
            let receipt=serde_json::to_vec(&serde_json::json!({"version":1,"batch_count":0,"payload_bytes":0,"digest":"","inline":stored})).unwrap();
            let closed = store
                .close_result_ingestion(&claim.fence, "close")
                .await
                .unwrap();
            let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
            // SAFETY: original fixture progress is preserved exactly, independent of
            // the explicitly partial scientific completion and its unavailable result.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        "seal",
                        TerminalClass::Partial,
                        &receipt,
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .unwrap();
            let mut reconnected = runtime.observe_study(request, Some(cursor)).await.unwrap();
            let mut sequences = Vec::new();
            let mut gap = false;
            for _ in 0..5 {
                let page = reconnected.poll().await.unwrap();
                for source in page.sources {
                    gap |= source.continuity == StudyObservationContinuity::GapCurrentResync;
                    sequences.extend(
                        source
                            .events
                            .into_iter()
                            .map(|e| e.source.unwrap().sequence),
                    );
                }
            }
            assert_eq!(sequences, if missing_prefix { vec![3] } else { vec![2, 3] });
            assert_eq!(gap, missing_prefix);
            assert!(
                !store
                    .canonical_study("observed")
                    .await
                    .unwrap()
                    .unwrap()
                    .cancelled
            );
            reconnected.close();
            drop(reconnected);
            let capture = first.status.receipt.capture.clone();
            drop(first);
            store.recover_study_capture(&capture).await.unwrap();
            store.remove_isolated_fixture().await.unwrap();
        }
    }
}
