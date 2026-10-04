// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "one serialized owner for the native HiGHS global scheduler reset"
)]
//! Shared scheduler ownership for direct HiGHS and Uno's HiGHS subproblems.
use crate::ProblemError;
#[cfg(feature = "uno")]
use crate::solve::Execution;
use std::sync::RwLock;
#[cfg(feature = "highs")]
use std::sync::RwLockReadGuard;
#[cfg(feature = "uno")]
use std::sync::{RwLockWriteGuard, TryLockError};
static LIFECYCLE: RwLock<()> = RwLock::new(());
thread_local! {static ACTIVE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}

#[cfg(feature = "highs")]
pub(crate) fn read() -> Result<RwLockReadGuard<'static, ()>, ProblemError> {
    if ACTIVE.with(|active| active.replace(true)) {
        return Err(ProblemError::Internal(
            "nested HiGHS scheduler ownership".into(),
        ));
    }
    Ok(LIFECYCLE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner))
}
/// Called after the read owner's last native model is destroyed. Another live owner
/// postpones reset until its destruction; an idle owner never makes teardown block.
#[cfg(feature = "highs")]
pub(crate) fn released() {
    ACTIVE.with(|active| active.set(false));
    if let Ok(_exclusive) = LIFECYCLE.try_write() {
        reset();
    }
}
fn reset() {
    // SAFETY: every caller holds the exclusive scheduler gate and all its native
    // HiGHS models have been destroyed before calling reset.
    unsafe { highs_sys::Highs_resetGlobalScheduler(1) };
}

/// Uno requires serial HiGHS. Hold this owner until Uno and every subproblem are gone.
#[cfg(feature = "uno")]
pub(crate) struct Exclusive {
    _gate: RwLockWriteGuard<'static, ()>,
}
#[cfg(feature = "uno")]
impl Drop for Exclusive {
    fn drop(&mut self) {
        reset();
        ACTIVE.with(|active| active.set(false));
    }
}
#[cfg(feature = "uno")]
pub(crate) fn exclusive(execution: &Execution) -> Result<Exclusive, ProblemError> {
    if ACTIVE.with(std::cell::Cell::get) {
        return Err(ProblemError::Internal(
            "Uno cannot enter while this worker retains a HiGHS model".into(),
        ));
    }
    loop {
        execution.check()?;
        let gate = match LIFECYCLE.try_write() {
            Ok(gate) => Some(gate),
            Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
            Err(TryLockError::WouldBlock) => None,
        };
        if let Some(gate) = gate {
            execution.check()?;
            reset();
            ACTIVE.with(|active| active.set(true));
            return Ok(Exclusive { _gate: gate });
        }
        std::thread::park_timeout(std::time::Duration::from_millis(10));
    }
}
