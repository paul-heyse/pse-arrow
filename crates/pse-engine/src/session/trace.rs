// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Attempt-local observations shared by native forks. These are diagnostics, not
//! output authority; completed computations retain their actual preparations.

use super::{EngineSession, PlanObservation};
use crate::EngineError;
use pse_columnar::MemoryPool;
use std::sync::Mutex;

pub(super) struct ExecutionTrace(Mutex<State>);
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
    pub(super) fn new(pool: &std::sync::Arc<dyn MemoryPool>) -> Self {
        Self(Mutex::new(State {
            observations: Vec::new(),
            reservation: pse_columnar::MemoryConsumer::new("session:execution-observations")
                .register(pool),
        }))
    }
    pub(super) fn record(&self, observation: PlanObservation) -> Result<(), EngineError> {
        if !observation.is_captured() {
            return Ok(());
        }
        let mut state = self.0.lock().map_err(|_| invalid())?;
        state
            .reservation
            .try_grow(2 * size_of::<PlanObservation>())?;
        state.observations.reserve_exact(1);
        state.observations.push(observation);
        Ok(())
    }
}
impl EngineSession {
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
        self.requires_fresh
            .load(std::sync::atomic::Ordering::Acquire)
    }
}
fn invalid() -> EngineError {
    EngineError::Internal {
        message: "native execution observation lock poisoned".to_owned(),
    }
}
