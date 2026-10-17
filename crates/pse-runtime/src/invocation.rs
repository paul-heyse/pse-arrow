// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Joined invocation evidence. Recorded clocks describe performing owners, never summed charges.
use crate::{math::PreparationCounts, workflow::AttemptSummary};
use pse_diagnostics::observation::OperationId;
use pse_model::generated::identities::{AttemptId, RunId};
use std::{sync::{Arc, Mutex}, time::Instant};

/// A boundary's observed outcome, independent of scientific product permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
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
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationProduct {
    /// Stable workflow-owned product role, not a solver status.
    pub role: String,
    /// The actual governing owner's retained conclusion.
    pub availability: ProductAvailability,
}
/// One directly observed inclusive owner interval, relative to supervisor entry.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationInterval {
    /// Performing responsibility; nested intervals must not be added together.
    pub stage: String,
    /// Wall seconds after this invocation's supervisor began.
    pub started_seconds: f64,
    /// Wall seconds after supervisor entry, absent if the owner never completed.
    pub ended_seconds: Option<f64>,
    /// Owner outcome, absent while active.
    pub outcome: Option<InvocationOutcome>,
}
/// Observed supervisor endpoints. Missing endpoints are unavailable, never zero.
#[derive(Clone, Debug, Default, serde::Serialize, schemars::JsonSchema)]
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
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvocationEvidenceDocument {
    /// Encoding contract version.
    pub version: u32,
    /// Original execution identity.
    pub run_id: RunId,
    /// Durable attempt when registered; ephemeral invocations have none.
    pub attempt_id: Option<AttemptId>,
    /// Diagnostic operation carrying this invocation, if observation was admitted.
    pub operation: Option<OperationId>,
    /// Actual workflow family.
    pub family: String,
    /// Exact canonical locators when registration/publication acknowledged them.
    pub canonical_run: Option<String>,
    /// Exact canonical native attempt locator.
    pub canonical_attempt: Option<String>,
    /// Actual joined invocation outcome; scientific permission remains separate.
    pub outcome: InvocationOutcome,
    /// Directly observed endpoints on this invocation's monotonic supervisor clock.
    pub endpoints: InvocationEndpoints,
    /// Bounded directly observed inclusive owner intervals.
    pub intervals: Vec<InvocationInterval>,
    /// Intervals omitted after the bounded owner capacity was exhausted.
    pub omitted_intervals: usize,
    /// Actual requested products and their independent availability.
    pub products: Vec<InvocationProduct>,
    /// Preparations performed in this task-local scope, if the owner established that scope.
    pub preparations: Option<PreparationCounts>,
    /// Existing retained attempt evidence: starts, reuse, configuration, work and coverage.
    /// Its native/strategy observations overlap and must not be added together.
    pub attempts: Arc<AttemptSummary>,
}
#[derive(Debug, Default)]
struct State {
    intervals: Vec<InvocationInterval>,
    omitted: usize,
    endpoints: InvocationEndpoints,
    preparations: Option<PreparationCounts>,
}
/// Supervisor-owned clock and bounded observations; no solver or cleanup authority.
#[derive(Debug)]
pub(crate) struct Recorder {
    started: Instant,
    operation: Option<OperationId>,
    state: Mutex<State>,
}
tokio::task_local! { static CURRENT: Arc<Recorder>; }
impl Recorder {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self { started: Instant::now(), operation: pse_diagnostics::observation::current().map(|v| v.id()), state: Mutex::new(State::default()) })
    }
    fn elapsed(&self) -> f64 { self.started.elapsed().as_secs_f64() }
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub(crate) fn qualified(&self, succeeded: bool) {
        if succeeded { self.state().endpoints.qualification_seconds = Some(self.elapsed()); }
    }
    pub(crate) fn terminal(&self) { self.state().endpoints.terminal_seconds = Some(self.elapsed()); }
    pub(crate) fn ready(&self) {
        let mut state = self.state();
        state.endpoints.response_ready_seconds = Some(self.elapsed());
        state.preparations = crate::math::current_preparations();
    }
    pub(crate) fn projection(&self) -> (Option<OperationId>, InvocationEndpoints, Vec<InvocationInterval>, usize, Option<PreparationCounts>) {
        let state = self.state();
        (self.operation, state.endpoints.clone(), state.intervals.clone(), state.omitted, state.preparations)
    }
}
pub(crate) fn current() -> Option<Arc<Recorder>> { CURRENT.try_with(Arc::clone).ok() }
pub(crate) async fn scope<T>(work: impl Future<Output = T>) -> T {
    CURRENT.scope(Recorder::new(), async { crate::math::counted(work).await.0 }).await
}
/// Count preparation only within the actual operation's task, including nested flights.
pub async fn counted<T>(work: impl Future<Output = T>) -> (T, PreparationCounts) {
    crate::math::counted(work).await
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
        if state.intervals.len() == 64 { state.omitted = state.omitted.saturating_add(1); return None; }
        let index = state.intervals.len();
        state.intervals.push(InvocationInterval { stage: name.into(), started_seconds: recorder.elapsed(), ended_seconds: None, outcome: None });
        Some(index)
    });
    Stage { recorder, index }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn invocation_endpoints_keep_failed_qualification_absent_and_clocks_separate() {
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
        }).await;
        scope(async {
            let recorder = current().unwrap();
            recorder.qualified(true);
            let (_, endpoints, _, _, _) = recorder.projection();
            assert!(endpoints.qualification_seconds.is_some());
            assert!(endpoints.terminal_seconds.is_none());
        }).await;
    }
    #[tokio::test]
    async fn invocation_stage_capacity_and_unfinished_intervals_are_explicit() {
        scope(async {
            for _ in 0..65 { let _unfinished = stage("boundary"); }
            let (_, _, intervals, omitted, _) = current().unwrap().projection();
            assert_eq!(intervals.len(), 64);
            assert_eq!(omitted, 1);
            assert!(intervals.iter().all(|v| v.ended_seconds.is_none() && v.outcome.is_none()));
        }).await;
    }
}
