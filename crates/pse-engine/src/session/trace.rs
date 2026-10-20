// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Attempt-local observations shared by native forks. These are diagnostics, not
//! output authority; completed computations retain their actual preparations.

use super::{EngineSession, PlanObservation};
use crate::EngineError;
use pse_columnar::MemoryPool;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

pub(super) struct ExecutionTrace(
    Mutex<State>,
    AtomicU64,
    super::CopiedObservationPolicy,
    Option<pse_diagnostics::observation::OperationId>,
);
impl std::fmt::Debug for ExecutionTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionTrace").finish_non_exhaustive()
    }
}
struct State {
    observations: Vec<PlanObservation>,
    reservation: pse_columnar::MemoryReservation,
}
impl ExecutionTrace {
    pub(super) fn new(
        _pool: &std::sync::Arc<dyn MemoryPool>,
        policy: super::assurance::ObservationPolicy,
    ) -> Self {
        Self(
            Mutex::new(State {
                observations: Vec::new(),
                reservation: pse_columnar::MemoryConsumer::new("session:execution-observations")
                    .register(&super::observation::diagnostic_pool()),
            }),
            AtomicU64::new(0),
            match policy {
                super::assurance::ObservationPolicy::Off => super::CopiedObservationPolicy::Off,
                super::assurance::ObservationPolicy::Contract => {
                    super::CopiedObservationPolicy::Contract
                }
                super::assurance::ObservationPolicy::Diagnostic => {
                    super::CopiedObservationPolicy::Diagnostic
                }
            },
            pse_diagnostics::observation::current().map(|context| context.id()),
        )
    }
    pub(super) fn record(&self, observation: PlanObservation) -> Result<(), EngineError> {
        if !observation.is_captured() {
            return Ok(());
        }
        let Ok(mut state) = self.0.try_lock() else {
            self.1.fetch_add(1, Ordering::Relaxed);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("query.capture.dropped", "observations", 1);
            }
            return Ok(());
        };
        if state.observations.len() >= 1024
            || state
                .reservation
                .try_grow(2 * size_of::<PlanObservation>())
                .is_err()
        {
            self.1.fetch_add(1, Ordering::Relaxed);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("query.capture.dropped", "observations", 1);
            }
            return Ok(());
        }
        state.observations.reserve_exact(1);
        state.observations.push(observation);
        Ok(())
    }
    fn copy_bounded(
        &self,
        maximum_bytes: usize,
    ) -> Result<(CopiedExecutionObservations, usize), EngineError> {
        let state = self.0.lock().map_err(|_| invalid())?;
        // Admit all temporary selection metadata before cloning even the source
        // handles. Payload strings and metrics remain borrowed until full sizing.
        let mut bytes = state
            .observations
            .len()
            .checked_mul(
                size_of::<PlanObservation>()
                    + size_of::<super::observation::SelectedPlanObservation<'_>>(),
            )
            .and_then(|bytes| bytes.checked_add(size_of::<CopiedExecutionObservations>() + 4096))
            .filter(|bytes| *bytes <= maximum_bytes)
            .ok_or_else(copy_limit)?;
        let mut observations = Vec::new();
        observations
            .try_reserve_exact(state.observations.len())
            .map_err(|_| copy_limit())?;
        observations.extend(state.observations.iter().cloned());
        let dropped = self.1.load(Ordering::Relaxed);
        drop(state);
        let mut selected = Vec::new();
        selected
            .try_reserve_exact(observations.len())
            .map_err(|_| copy_limit())?;
        selected.extend(observations.iter().map(PlanObservation::select_copy));
        for observation in &selected {
            bytes = observation
                .copied_extent()
                .and_then(|extent| bytes.checked_add(extent))
                .filter(|bytes| *bytes <= maximum_bytes)
                .ok_or_else(copy_limit)?;
        }
        // Every population below is the same immutable selection just sized.
        // Neither a newer checkpoint nor terminal publication is reread here.
        let mut copied = Vec::new();
        copied
            .try_reserve_exact(selected.len())
            .map_err(|_| copy_limit())?;
        copied.extend(
            selected
                .into_iter()
                .map(super::observation::SelectedPlanObservation::copied),
        );
        Ok((
            CopiedExecutionObservations {
                policy: Some(self.2),
                operation: self.3,
                observations: copied,
                dropped,
            },
            bytes,
        ))
    }
}
fn copy_limit() -> EngineError {
    EngineError::ConfigInvalid {
        key: "session.copied_observations".into(),
        reason: "copied execution observations exceed their allocation allowance".into(),
    }
}
impl EngineSession {
    /// Captured observations refused by the bounded diagnostic collector.
    pub fn dropped_execution_observations(&self) -> u64 {
        self.trace.1.load(Ordering::Relaxed)
    }

    /// Observations from actual native executions under this attempt and its forks.
    /// # Errors
    /// The observation collector was interrupted by an internal panic.
    pub fn execution_observations(&self) -> Result<Vec<PlanObservation>, EngineError> {
        Ok(self
            .trace
            .0
            .lock()
            .map_err(|_| invalid())?
            .observations
            .clone())
    }
    /// Stable/volatile or ambient-variable reads require a fresh preparation.
    /// A false value alone does not establish dependency equality or reuse.
    pub fn requires_fresh_execution(&self) -> bool {
        self.requires_fresh.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod bounded_copy_tests {
    use super::super::{assurance::ObservationPolicy, observation::Recorder};
    use super::*;
    use std::sync::Arc;

    #[test]
    fn bounded_copy_counts_repeated_shared_payloads_before_materializing() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let logical = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
            .build()
            .unwrap();
        let mut recorder = Recorder::new(&pool, ObservationPolicy::Diagnostic);
        recorder.rule(&"shared rule".repeat(4096));
        let observation = recorder
            .finish(&logical, &pse_columnar::CancellationToken::new())
            .unwrap();
        let trace = ExecutionTrace::new(&pool, ObservationPolicy::Diagnostic);
        for _ in 0..16 {
            trace.record(observation.clone()).unwrap();
        }
        assert!(
            trace.copy_bounded(16 << 10).is_err(),
            "shared source storage does not make sixteen owned copies free"
        );
        let (copied, bytes) = trace.copy_bounded(2 << 20).unwrap();
        assert_eq!(copied.observations.len(), 16);
        assert!(bytes > 16 * observation.rules_fired()[0].len());
        assert!(
            copied
                .observations
                .iter()
                .all(|copy| copy.rules.as_slice() == observation.rules_fired())
        );
        assert!(trace.copy_bounded(bytes - 1).is_err());
        let (exact, accounted) = trace.copy_bounded(bytes).unwrap();
        assert_eq!(accounted, bytes);
        assert_eq!(exact.observations.len(), 16);
    }

    #[test]
    fn bounded_copy_preserves_empty_off_policy_and_refuses_owner_overhead() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let trace = ExecutionTrace::new(&pool, ObservationPolicy::Off);
        assert!(trace.copy_bounded(4096).is_err());
        let (copied, bytes) = trace.copy_bounded(8192).unwrap();
        assert_eq!(
            copied.policy,
            Some(super::super::CopiedObservationPolicy::Off)
        );
        assert!(copied.observations.is_empty());
        assert_eq!(copied.dropped, 0);
        assert!(bytes <= 8192);
    }
}
fn invalid() -> EngineError {
    EngineError::Internal {
        message: "native execution observation lock poisoned".to_owned(),
    }
}

/// Passive data-only trace export; empty observations do not imply capture was enabled.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopiedExecutionObservations {
    /// Policy on the actual original session; absent in older copied documents.
    #[serde(default)]
    pub policy: Option<super::CopiedObservationPolicy>,
    /// Operation active when that session was established; absent is unknown lineage.
    #[serde(default)]
    pub operation: Option<pse_diagnostics::observation::OperationId>,
    /// Retained actual executions and preparations, in original collector order.
    pub observations: Vec<super::CopiedPlanObservation>,
    /// Original bounded collector's refused observations.
    pub dropped: u64,
}
impl EngineSession {
    /// Copy one frozen selection only after admitting all copied strings, vectors
    /// and metric populations under the supplied finite allocation allowance.
    /// Returns the accounted bytes, including temporary selection storage and
    /// fixed owner overhead, for the caller's retained allocation lease.
    /// # Errors
    /// The collector is poisoned or the complete selected copy exceeds the allowance.
    pub fn copy_execution_observations_bounded(
        &self,
        maximum_bytes: usize,
    ) -> Result<(CopiedExecutionObservations, usize), EngineError> {
        self.trace.copy_bounded(maximum_bytes)
    }
    /// Copy existing retained facts without enabling observation or reading a native operator.
    /// # Errors
    /// The retained observation collector was poisoned.
    pub fn copy_execution_observations(&self) -> Result<CopiedExecutionObservations, EngineError> {
        Ok(CopiedExecutionObservations {
            policy: Some(self.trace.2),
            operation: self.trace.3,
            observations: self
                .execution_observations()?
                .iter()
                .map(PlanObservation::copied)
                .collect(),
            dropped: self.dropped_execution_observations(),
        })
    }
}
