// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test the exact std-only polling implementation without linking the Python cdylib.
#[path = "../src/wait/policy.rs"]
mod policy;

use policy::{SignalAction, wait_with};
use std::{task::Poll, time::Duration};

mod tests {
    use super::*;
    #[test]
    fn final_poll_signal_retains_acknowledged_value() {
        let mut stopped = 0;
        let result = wait_with(
            Duration::from_millis(100),
            |_| true,
            SignalAction::Drain,
            |_| Poll::Ready("acknowledged-key"),
            || Err("first signal"),
            || stopped += 1,
        );
        assert_eq!(result, (Some("acknowledged-key"), Some("first signal")));
        assert_eq!(stopped, 1);
    }
    #[test]
    fn drain_keeps_polling_original_work_and_retains_first_signal() {
        let mut polls = 0;
        let mut checks = 0;
        let mut stops = 0;
        let result = wait_with(
            Duration::from_millis(20),
            |_| true,
            SignalAction::Drain,
            |interval| {
                assert_eq!(interval, Duration::from_millis(20));
                polls += 1;
                if polls == 3 {
                    Poll::Ready(7)
                } else {
                    Poll::Pending
                }
            },
            || {
                checks += 1;
                Err("first signal")
            },
            || stops += 1,
        );
        assert_eq!(result, (Some(7), Some("first signal")));
        assert_eq!((polls, checks, stops), (3, 1, 1));
    }
    #[test]
    fn observer_policy_returns_without_polling_again_after_signal() {
        let mut polls = 0;
        let result = wait_with(
            Duration::from_millis(100),
            |_| false,
            SignalAction::Return,
            |_| {
                polls += 1;
                Poll::<()>::Pending
            },
            || Err("interrupt"),
            || {},
        );
        assert_eq!(result, (None, Some("interrupt")));
        assert_eq!(polls, 1);
    }
    #[test]
    fn already_complete_capture_does_not_check_new_signal() {
        let result = wait_with(
            Duration::from_millis(20),
            |_| false,
            SignalAction::Drain,
            |_| Poll::Ready(3),
            || -> Result<(), ()> { panic!("completion precedes signal") },
            || panic!("no stop"),
        );
        assert_eq!(result, (Some(3), None));
    }
}
