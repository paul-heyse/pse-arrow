// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Joined invocation evidence. Recorded clocks describe performing owners, never summed charges.
use crate::{math::PreparationCounts, workflow::AttemptSummary};
use pse_diagnostics::observation::OperationId;
use pse_model::generated::identities::{AttemptId, RunId};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

type InvocationVersion = pse_model::document::Version<2>;

/// A boundary's observed outcome, independent of scientific product permission.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum InvocationOutcome {
    /// The performing owner completed this responsibility.
    Completed,
    /// The performing owner failed or refused this responsibility.
    Failed,
    /// Cancellation was observed before completion.
    Cancelled,
}
/// Explicit scientific product availability; response readiness does not imply availability.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProductAvailability {
    /// Requested and established by its governing owner.
    Available,
    /// Requested but not established, including original refusal.
    Unavailable,
    /// This invocation did not request the product.
    NotRequested,
}
/// One requested product, independently of the invocation's terminal outcome.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationProduct {
    /// Stable workflow-owned product role, not a solver status.
    pub role: String,
    /// Original requested step, absent for a whole-workflow product.
    pub step: Option<usize>,
    /// The actual governing owner's retained conclusion.
    pub availability: ProductAvailability,
}
/// One directly observed inclusive owner interval, relative to invocation establishment.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationInterval {
    /// Performing responsibility; nested intervals must not be added together.
    pub stage: String,
    /// Wall seconds after this invocation was established before dispatch.
    pub started_seconds: f64,
    /// Wall seconds after invocation establishment, absent if the owner never completed.
    pub ended_seconds: Option<f64>,
    /// Owner outcome, absent while active.
    pub outcome: Option<InvocationOutcome>,
}
/// Observed invocation endpoints. Missing endpoints are unavailable, never zero.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationEndpoints {
    /// Governing original qualification succeeded; absent on failed qualification.
    pub qualification_seconds: Option<f64>,
    /// Scientific execution and native drain ended, including failed outcomes.
    pub terminal_seconds: Option<f64>,
    /// The typed joined response was finalized for publication to waiters.
    pub response_ready_seconds: Option<f64>,
}
/// One actual execution. Attempt evidence retains its existing charging and settings authority.
#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationEvidenceDocument {
    /// Encoding contract version.
    pub version: InvocationVersion,
    /// Original execution identity.
    pub run_id: RunId,
    /// Durable attempt when registered; ephemeral invocations have none.
    pub attempt_id: Option<AttemptId>,
    /// Diagnostic operation carrying this invocation, if observation was admitted.
    pub operation: Option<OperationId>,
    /// Actual workflow family.
    pub family: String,
    /// Performing clock boundary; no inferred preparation or native start is included.
    pub clock_scope: String,
    /// Why performing-clock collection is unavailable; absent means the owner was admitted.
    pub timing_unavailable: Option<pse_diagnostics::observation::RecordAvailability>,
    /// Exact canonical locators when registration/publication acknowledged them.
    pub canonical_run: Option<String>,
    /// Exact canonical native attempt locator.
    pub canonical_attempt: Option<String>,
    /// Actual joined invocation outcome; scientific permission remains separate.
    pub outcome: InvocationOutcome,
    /// Directly observed endpoints on this invocation's monotonic pre-dispatch clock.
    pub endpoints: InvocationEndpoints,
    /// Bounded directly observed inclusive owner intervals.
    pub intervals: Vec<InvocationInterval>,
    /// Intervals omitted after the bounded owner capacity was exhausted.
    pub omitted_intervals: usize,
    /// Actual requested products and their independent availability.
    pub products: Vec<InvocationProduct>,
    /// Requested per-step product sets beyond the same bounded attempt-summary prefix.
    pub omitted_product_steps: usize,
    /// Preparations performed in this task-local scope, if the owner established that scope.
    pub preparations: Option<PreparationCounts>,
    /// Existing retained attempt evidence: starts, reuse, configuration, work and coverage.
    /// Its native/strategy observations overlap and must not be added together.
    pub attempts: Arc<crate::workflow::AttemptSummaryDocument>,
}
impl InvocationEvidenceDocument {
    /// Actual fixed version of this original joined-evidence encoding.
    pub const fn version(&self) -> u32 { InvocationVersion::NUMBER }
    /// Validate factual coordinates before matching saved evidence; no native admission occurs.
    pub fn validate(&self) -> Result<(), crate::observation::ObservationError> {
        let invalid = |message: &str| crate::observation::ObservationError::Format(message.into());
        if self.run_id != self.attempts.run_id {
            return Err(invalid("unsupported or inconsistent invocation identity"));
        }
        let mut steps = std::collections::BTreeSet::new();
        for step in &self.attempts.steps {
            if step.step >= self.attempts.requested_steps || !steps.insert(step.step) {
                return Err(invalid(
                    "duplicate or out-of-range invocation step identity",
                ));
            }
        }
        let mut products = std::collections::BTreeSet::new();
        for product in &self.products {
            if product
                .step
                .is_some_and(|step| step >= self.attempts.requested_steps)
                || !products.insert((&product.role, product.step))
            {
                return Err(invalid(
                    "duplicate or out-of-range invocation product identity",
                ));
            }
        }
        let mut native = std::collections::BTreeSet::new();
        for attempt in &self.attempts.native_attempts {
            if attempt.step >= self.attempts.requested_steps
                || !native.insert((attempt.step, attempt.mechanism, attempt.component))
            {
                return Err(invalid(
                    "duplicate or out-of-range invocation native coordinate identity",
                ));
            }
        }
        Ok(())
    }
}
/// Live admitted invocation retains allocations separately from its readable payload.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct InvocationEvidence {
    #[serde(flatten)]
    pub(crate) document: InvocationEvidenceDocument,
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) _owner: Arc<AttemptSummary>,
}
impl std::ops::Deref for InvocationEvidence {
    type Target = InvocationEvidenceDocument;
    fn deref(&self) -> &Self::Target {
        &self.document
    }
}
#[derive(Debug, Default)]
struct State {
    intervals: Vec<InvocationInterval>,
    omitted: usize,
    endpoints: InvocationEndpoints,
    preparations: Option<PreparationCounts>,
}
/// Invocation-owned clock and bounded observations; no solver or cleanup authority.
#[derive(Debug)]
pub(crate) struct Recorder {
    started: Instant,
    operation: Option<OperationId>,
    state: Mutex<State>,
    _reservation: pse_diagnostics::observation::OwnerReservation,
}
tokio::task_local! { static CURRENT: (Option<Arc<Recorder>>, Option<pse_diagnostics::observation::RecordAvailability>); }
type ContextValue = (
    Option<Arc<Recorder>>,
    Option<pse_diagnostics::observation::RecordAvailability>,
);
thread_local! { static SYNCHRONOUS: std::cell::RefCell<Option<ContextValue>> = const { std::cell::RefCell::new(None) }; }
/// Explicitly transferred copied diagnostic context; never native/result ownership.
#[derive(Clone)]
pub(crate) struct TransferredContext(ContextValue);
pub(crate) struct EnteredContext(
    Option<ContextValue>,
    std::marker::PhantomData<std::rc::Rc<()>>,
);
impl TransferredContext {
    fn fresh() -> Self {
        let recorder = Recorder::new();
        let unavailable = if recorder.is_some() {
            None
        } else {
            Some(if pse_diagnostics::observation::current().is_some() {
                pse_diagnostics::observation::RecordAvailability::Refused
            } else {
                pse_diagnostics::observation::RecordAvailability::NotRequested
            })
        };
        Self((recorder, unavailable))
    }
    pub(crate) fn capture() -> Self {
        Self(CURRENT.try_with(Clone::clone).unwrap_or_else(|_| {
            SYNCHRONOUS.with(|scope| scope.borrow().clone()).unwrap_or((
                None,
                Some(pse_diagnostics::observation::RecordAvailability::Unavailable),
            ))
        }))
    }
    pub(crate) fn enter(&self) -> EnteredContext {
        EnteredContext(
            SYNCHRONOUS.with(|scope| scope.replace(Some(self.0.clone()))),
            std::marker::PhantomData,
        )
    }
    pub(crate) async fn scope<T>(self, work: impl Future<Output = T>) -> T {
        CURRENT
            .scope(self.0, async { crate::math::counted(work).await.0 })
            .await
    }
}
/// Establish the invocation before a synchronous start can submit native work.
/// Task-local sync_scope also overrides an enclosing async invocation while constructing
/// the child; a thread-local fallback alone would incorrectly inherit that parent.
pub(crate) fn start_scope<T>(work: impl FnOnce() -> T) -> impl FnOnce() -> T {
    move || {
        // The caller first enters its RuntimeActivity observation owner. Creating
        // the recorder while building this closure would capture the outer owner
        // (or no owner) before that synchronous activity scope is entered.
        let context = TransferredContext::fresh();
        CURRENT.sync_scope(context.0, work)
    }
}
impl Drop for EnteredContext {
    fn drop(&mut self) {
        SYNCHRONOUS.with(|scope| {
            scope.replace(self.0.take());
        });
    }
}
/// Capture at future construction, before the caller dispatches an independent task.
pub(crate) fn inherit<T>(work: impl Future<Output = T>) -> impl Future<Output = T> {
    let context = TransferredContext::capture();
    async move { CURRENT.scope(context.0, work).await }
}
impl Recorder {
    pub(crate) fn new() -> Option<Arc<Self>> {
        let context = pse_diagnostics::observation::current()?;
        let producer = context.producer(
            pse_diagnostics::observation::ProducerRole::Workflow,
            "workflow.invocation",
        );
        // Live state plus bounded document/serialization copies; literal stage names are bounded below.
        let reservation = producer.reserve_owner_bytes(
            size_of::<Self>() + 64 * (3 * size_of::<InvocationInterval>() + 384),
        )?;
        Some(Arc::new(Self {
            started: Instant::now(),
            operation: Some(context.id()),
            state: Mutex::new(State {
                intervals: Vec::with_capacity(64),
                ..State::default()
            }),
            _reservation: reservation,
        }))
    }
    pub(crate) fn operation(&self) -> Option<OperationId> {
        self.operation
    }
    fn elapsed(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub(crate) fn qualified(&self, succeeded: bool) {
        if succeeded {
            self.state().endpoints.qualification_seconds = Some(self.elapsed());
        }
    }
    pub(crate) fn terminal(&self) {
        self.state().endpoints.terminal_seconds = Some(self.elapsed());
    }
    pub(crate) fn ready(&self) {
        let mut state = self.state();
        state.endpoints.response_ready_seconds = Some(self.elapsed());
        state.preparations = crate::math::current_preparations();
    }
    pub(crate) fn projection(
        &self,
    ) -> (
        Option<OperationId>,
        InvocationEndpoints,
        Vec<InvocationInterval>,
        usize,
        Option<PreparationCounts>,
    ) {
        let state = self.state();
        (
            self.operation,
            state.endpoints.clone(),
            state.intervals.clone(),
            state.omitted,
            state.preparations,
        )
    }
}
pub(crate) fn current() -> Option<Arc<Recorder>> {
    CURRENT
        .try_with(|scope| scope.0.clone())
        .unwrap_or_else(|_| {
            SYNCHRONOUS.with(|scope| scope.borrow().as_ref().and_then(|scope| scope.0.clone()))
        })
}
pub(crate) fn current_unavailability() -> Option<pse_diagnostics::observation::RecordAvailability> {
    CURRENT.try_with(|scope| scope.1).unwrap_or_else(|_| {
        SYNCHRONOUS.with(|scope| {
            scope.borrow().as_ref().map_or(
                Some(pse_diagnostics::observation::RecordAvailability::Unavailable),
                |scope| scope.1,
            )
        })
    })
}
pub(crate) fn scope<T>(work: impl Future<Output = T>) -> impl Future<Output = T> {
    TransferredContext::fresh().scope(work)
}
/// Count preparation only within the actual operation's task, including nested flights.
pub async fn counted<T>(work: impl Future<Output = T>) -> (T, PreparationCounts) {
    crate::math::counted(work).await
}
/// Count a synchronous operation, including its explicitly polled async preparation.
pub fn counted_sync<T>(work: impl FnOnce() -> T) -> (T, PreparationCounts) {
    crate::math::counted_sync(work)
}
/// Boundary interval guard. An unfinished guard reports no completion endpoint.
pub(crate) struct Stage {
    recorder: Option<Arc<Recorder>>,
    index: Option<usize>,
}
impl Stage {
    pub(crate) fn finish(mut self, outcome: InvocationOutcome) {
        if let (Some(recorder), Some(index)) = (self.recorder.take(), self.index) {
            let end = recorder.elapsed();
            let mut state = recorder.state();
            state.intervals[index].ended_seconds = Some(end);
            state.intervals[index].outcome = Some(outcome);
        }
    }
}
pub(crate) fn stage(name: &'static str) -> Stage {
    let recorder = current();
    let index = recorder.as_ref().and_then(|recorder| {
        let mut state = recorder.state();
        if name.len() > 128 || state.intervals.len() == 64 {
            state.omitted = state.omitted.saturating_add(1);
            return None;
        }
        let index = state.intervals.len();
        state.intervals.push(InvocationInterval {
            stage: name.into(),
            started_seconds: recorder.elapsed(),
            ended_seconds: None,
            outcome: None,
        });
        Some(index)
    });
    Stage { recorder, index }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invocation_establishment_uses_the_entered_start_owner() {
        // This matches RuntimeActivity::sync_scope: its observation owner enters
        // after the argument closure is constructed, and before dispatch occurs.
        let start = start_scope(|| current().unwrap());
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("invocation.start_owner");
        let context = operation.context();
        let _entered = context.enter();
        let recorder = start();
        assert_eq!(recorder.operation(), Some(context.id()));
        assert!(current().is_none());
    }
    #[tokio::test]
    async fn synchronous_dispatch_and_supervisor_share_a_fresh_nested_invocation() {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("invocation.pre_dispatch");
        let context = operation.context();
        let _entered = context.enter();
        scope(async {
            let parent = current().unwrap();
            let (child, transferred) = start_scope(|| {
                let child = current().unwrap();
                assert!(!Arc::ptr_eq(&child, &parent));
                stage("native.before_supervisor").finish(InvocationOutcome::Completed);
                (child, TransferredContext::capture())
            })();
            assert!(Arc::ptr_eq(&current().unwrap(), &parent));
            transferred
                .scope(async {
                    assert!(Arc::ptr_eq(&current().unwrap(), &child));
                    stage("original.final_assessment").finish(InvocationOutcome::Completed);
                    child.terminal();
                    child.ready();
                })
                .await;
            assert!(Arc::ptr_eq(&current().unwrap(), &parent));
            let (_, endpoints, intervals, omitted, _) = child.projection();
            assert_eq!(omitted, 0);
            assert_eq!(intervals.len(), 2);
            assert_eq!(intervals[0].stage, "native.before_supervisor");
            assert!(intervals[1].started_seconds >= intervals[0].started_seconds);
            assert!(endpoints.response_ready_seconds >= endpoints.terminal_seconds);
            assert!(parent.projection().2.is_empty());
        })
        .await;
    }
    #[tokio::test]
    async fn invocation_endpoints_keep_failed_qualification_absent_and_clocks_separate() {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("invocation.control");
        let context = operation.context();
        let _entered = context.enter();
        scope(async {
            let recorder = current().unwrap();
            let failed = stage("original_qualification");
            recorder.qualified(false);
            failed.finish(InvocationOutcome::Failed);
            recorder.terminal();
            recorder.ready();
            let (_, endpoints, intervals, _, _) = recorder.projection();
            assert!(endpoints.qualification_seconds.is_none());
            assert!(endpoints.terminal_seconds.is_some());
            assert!(endpoints.response_ready_seconds >= endpoints.terminal_seconds);
            assert_eq!(intervals[0].outcome, Some(InvocationOutcome::Failed));
        })
        .await;
        scope(async {
            let recorder = current().unwrap();
            recorder.qualified(true);
            let (_, endpoints, _, _, _) = recorder.projection();
            assert!(endpoints.qualification_seconds.is_some());
            assert!(endpoints.terminal_seconds.is_none());
        })
        .await;
    }
    #[tokio::test]
    async fn unavailable_invocation_timing_does_not_fabricate_zero_endpoints() {
        scope(async {
            assert!(current().is_none());
            assert_eq!(
                current_unavailability(),
                Some(pse_diagnostics::observation::RecordAvailability::NotRequested)
            );
            stage("unobserved").finish(InvocationOutcome::Completed);
        })
        .await;
    }
    #[test]
    fn integration_reason_and_products_remain_coherent_across_failed_upgrade() {
        use pse_diagnostics::observation::{ObservationHub, ProducerRole, Value};
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("integration.control");
        let context = operation.context();
        let producer = context.producer(ProducerRole::Workflow, "window.products");
        IntegrationObservation::begin(
            Some(producer.clone()),
            IntegrationPurpose::PrimalTrial,
            false,
            false,
            false,
            0,
            None,
        )
        .finish(true);
        IntegrationObservation::begin(
            Some(producer),
            IntegrationPurpose::DerivativeUpgrade,
            true,
            false,
            false,
            0,
            Some(2),
        )
        .finish(false);
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let records = &snapshot.producers[0].recent;
        assert_eq!(records.len(), 4);
        assert_eq!(records[1].phase, "completed");
        assert_eq!(records[3].phase, "failed");
        assert_eq!(
            records[1].values[0].value,
            Value::Text("primal_trial".into())
        );
        assert_eq!(
            records[3].values[0].value,
            Value::Text("derivative_upgrade".into())
        );
        assert_eq!(records[3].values[2].value, Value::Boolean(true));
        assert_eq!(records[3].values[5].value, Value::Count(2));
        assert!(records[3].values[5].availability.is_none());
        assert_eq!(
            records[1].values[5].availability,
            Some(pse_diagnostics::observation::RecordAvailability::NotRequested)
        );
        assert!(
            records
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );
    }
    #[tokio::test]
    async fn invocation_transfers_exact_recorder_to_native_thread_and_clears_it() {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("invocation.transfer");
        let context = operation.context();
        let _entered = context.enter();
        scope(async {
            let recorder = current().unwrap();
            let transferred = TransferredContext::capture();
            let expected = recorder.clone();
            std::thread::spawn(move || {
                assert!(current().is_none());
                {
                    let _scope = transferred.enter();
                    assert!(Arc::ptr_eq(&current().unwrap(), &expected));
                    stage("original.qualification").finish(InvocationOutcome::Completed);
                }
                assert!(current().is_none());
            })
            .join()
            .unwrap();
            let (_, _, intervals, omitted, _) = recorder.projection();
            assert_eq!(omitted, 0);
            assert_eq!(intervals.len(), 1);
            assert_eq!(intervals[0].stage, "original.qualification");
            assert!(intervals[0].ended_seconds.is_some());
        })
        .await;
    }
    #[tokio::test]
    async fn invocation_stage_capacity_and_unfinished_intervals_are_explicit() {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("invocation.capacity");
        let context = operation.context();
        let _entered = context.enter();
        scope(async {
            for _ in 0..65 {
                let _unfinished = stage("boundary");
            }
            let (_, _, intervals, omitted, _) = current().unwrap().projection();
            assert_eq!(intervals.len(), 64);
            assert_eq!(omitted, 1);
            assert!(
                intervals
                    .iter()
                    .all(|v| v.ended_seconds.is_none() && v.outcome.is_none())
            );
        })
        .await;
    }
}

/// The workflow reason for an actual integration, independently of its derivative product.
#[derive(
    Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationPurpose {
    /// A new optimizer or requested primal point.
    #[default]
    PrimalTrial,
    /// Missing products at the same exact point.
    DerivativeUpgrade,
    /// Fresh governing original-model assessment.
    OriginalAssessment,
    /// Rank/response qualification after the native solve.
    Identifiability,
    /// Residual curvature needed by an actual Hessian request.
    Curvature,
}
impl IntegrationPurpose {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::PrimalTrial => "primal_trial",
            Self::DerivativeUpgrade => "derivative_upgrade",
            Self::OriginalAssessment => "original_assessment",
            Self::Identifiability => "identifiability",
            Self::Curvature => "curvature",
        }
    }
}
/// Bounded records from an actual performing owner. It creates no numerical demand.
pub(crate) struct IntegrationObservation {
    producer: Option<pse_diagnostics::observation::Producer>,
    fields: Vec<pse_diagnostics::observation::RecordField>,
    finished: bool,
    _reservation: Option<pse_diagnostics::observation::OwnerReservation>,
}
impl IntegrationObservation {
    pub(crate) fn begin(
        producer: Option<pse_diagnostics::observation::Producer>,
        purpose: IntegrationPurpose,
        forward: bool,
        adjoint: bool,
        curvature: bool,
        member: u64,
        forward_directions: Option<usize>,
    ) -> Self {
        use pse_diagnostics::observation::{RecordField, Value};
        let producer = producer.filter(pse_diagnostics::observation::Producer::enabled);
        let reservation = producer
            .as_ref()
            .and_then(|producer| producer.reserve_owner_bytes(4096));
        if reservation.is_none() {
            if let Some(producer) = &producer {
                producer
                    .context()
                    .record_source_unavailable("workflow.integration.v1");
            }
            return Self {
                producer: None,
                fields: Vec::new(),
                finished: false,
                _reservation: None,
            };
        }
        let fields = vec![
            RecordField {
                name: "purpose".into(),
                unit: "workflow reason".into(),
                value: Value::Text(purpose.as_str().into()),
                availability: None,
            },
            RecordField {
                name: "member".into(),
                unit: "experiment or window index".into(),
                value: Value::Count(member),
                availability: None,
            },
            RecordField {
                name: "forward_response".into(),
                unit: "requested product".into(),
                value: Value::Boolean(forward),
                availability: None,
            },
            RecordField {
                name: "adjoint".into(),
                unit: "requested product".into(),
                value: Value::Boolean(adjoint),
                availability: None,
            },
            RecordField {
                name: "curvature".into(),
                unit: "requested product".into(),
                value: Value::Boolean(curvature),
                availability: None,
            },
            RecordField {
                name: "forward_direction_columns".into(),
                unit: "selected physical coordinate request".into(),
                value: Value::Count(forward_directions.unwrap_or(0) as u64),
                availability: forward_directions
                    .is_none()
                    .then_some(pse_diagnostics::observation::RecordAvailability::NotRequested),
            },
        ];
        let mut observation = Self {
            producer,
            fields,
            finished: false,
            _reservation: reservation,
        };
        observation.publish("started");
        observation
    }
    fn publish(&mut self, phase: &'static str) {
        if let Some(producer) = &self.producer {
            let record = producer.record(
                "workflow.integration.v1",
                phase,
                None,
                None,
                self.fields.clone(),
            );
            producer.publish(&record);
        }
    }
    pub(crate) fn finish(mut self, completed: bool) {
        self.publish(if completed { "completed" } else { "failed" });
        self.finished = true;
    }
}
impl Drop for IntegrationObservation {
    fn drop(&mut self) {
        if !self.finished {
            self.publish("interrupted");
        }
    }
}
