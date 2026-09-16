// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Attempt-local observations shared by native forks. These are diagnostics, not
//! output authority; completed computations retain their actual preparations.

use super::{PlanObservation, SnapshotSession};
use crate::CatalogError;
use pse_ids::{MemoryReserver, Reservation};
use std::sync::Mutex;

pub(super) struct ExecutionTrace(Mutex<State>);
impl std::fmt::Debug for ExecutionTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionTrace").finish_non_exhaustive()
    }
}
struct State {
    observations: Vec<PlanObservation>,
    requires_fresh: bool,
    reservation: Box<dyn Reservation>,
}
impl ExecutionTrace {
    pub(super) fn new(reserver: &dyn MemoryReserver) -> Self {
        Self(Mutex::new(State {
            observations: Vec::new(),
            requires_fresh: false,
            reservation: reserver.open("session:execution-observations"),
        }))
    }
    pub(super) fn record(
        &self,
        observation: PlanObservation,
        requires_fresh: bool,
    ) -> Result<(), CatalogError> {
        let mut state = self.0.lock().map_err(|_| invalid())?;
        state
            .reservation
            .try_grow(2 * size_of::<PlanObservation>())?;
        state.observations.reserve_exact(1);
        state.observations.push(observation);
        state.requires_fresh |= requires_fresh;
        Ok(())
    }
}
impl SnapshotSession {
    /// Observations from actual native executions under this attempt and its forks.
    /// # Errors
    /// The observation collector was interrupted by an internal panic.
    pub fn execution_observations(&self) -> Result<Vec<PlanObservation>, CatalogError> {
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
    /// # Errors
    /// The attempt's observation collector is unavailable.
    pub fn requires_fresh_execution(&self) -> Result<bool, CatalogError> {
        Ok(self.trace.0.lock().map_err(|_| invalid())?.requires_fresh)
    }
}
fn invalid() -> CatalogError {
    CatalogError::Internal {
        message: "native execution observation lock poisoned".to_owned(),
    }
}
