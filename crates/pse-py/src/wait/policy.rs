// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Interpreter-independent polling and signal policy, shared with its standalone test target.
use std::{task::Poll, time::Duration};
#[derive(Clone, Copy)]
pub(crate) enum SignalAction {
    Drain,
    Return,
}
pub(super) fn wait_with<T, E>(
    interval: Duration,
    final_check: impl Fn(&T) -> bool,
    action: SignalAction,
    mut poll: impl FnMut(Duration) -> Poll<T>,
    mut check: impl FnMut() -> Result<(), E>,
    mut on_signal: impl FnMut(),
) -> (Option<T>, Option<E>) {
    let mut signal = None;
    loop {
        let value = poll(interval);
        if let Poll::Ready(value) = value {
            if final_check(&value)
                && signal.is_none()
                && let Err(error) = check()
            {
                on_signal();
                signal = Some(error);
            }
            return (Some(value), signal);
        }
        if signal.is_none()
            && let Err(error) = check()
        {
            on_signal();
            signal = Some(error);
            if matches!(action, SignalAction::Return) {
                return (None, signal);
            }
        }
    }
}
