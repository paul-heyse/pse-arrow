// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dispose caught panic payload owners before reporting their settled failure.

/// Preserve the original string observation and destroy the original payload under
/// unwind containment. A `panic_any` payload may own a destructor that itself panics;
/// every resulting payload is consumed here before returning to independent cleanup.
/// Nonterminating destruction remains with this performing caller and its custody.
pub fn settle_payload(mut payload: Box<dyn std::any::Any + Send>, opaque: &'static str) -> String {
    let mut detail = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_owned())
        })
        .unwrap_or_else(|| opaque.into());
    loop {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(payload))) {
            Ok(()) => return detail,
            Err(next) => {
                detail.push_str("; panic payload destruction also panicked");
                if let Some(message) = next.downcast_ref::<String>() {
                    detail.push_str(": ");
                    detail.push_str(message);
                } else if let Some(message) = next.downcast_ref::<&str>() {
                    detail.push_str(": ");
                    detail.push_str(message);
                }
                payload = next;
            }
        }
    }
}
