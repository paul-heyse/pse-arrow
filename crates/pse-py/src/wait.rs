// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronous interpreter waits retain each performing owner's stop and completion policy.
use pyo3::prelude::*;
use std::{task::Poll, time::Duration};
mod policy;
pub(crate) use policy::SignalAction;
use policy::wait_with;
pub(crate) struct WaitOutcome<T> {
    pub(crate) value: Option<T>,
    pub(crate) signal: Option<PyErr>,
}
pub(crate) fn precheck(py: Python<'_>) -> PyResult<()> { py.check_signals() }
pub(crate) fn wait<T: Send>(
    py: Python<'_>,
    interval: Duration,
    final_check: impl Fn(&T) -> bool,
    action: SignalAction,
    mut poll: impl FnMut(Duration) -> Poll<T> + Send,
    mut on_signal: impl FnMut(),
) -> WaitOutcome<T> {
    let (value, signal) = wait_with(interval, final_check, action,
        |interval| py.detach(|| poll(interval)), || py.check_signals(), &mut on_signal);
    WaitOutcome { value, signal }
}
