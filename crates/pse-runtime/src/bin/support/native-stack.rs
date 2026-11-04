// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Existing native application stack selection, shared by execution frontends.
use std::{ffi::OsStr, io};

pub(super) fn selected() -> io::Result<usize> {
    selected_value(std::env::var_os("RUST_MIN_STACK").as_deref())
}

pub(super) fn runtime(stack: usize) -> io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(stack)
        .enable_all()
        .build()
}

fn selected_value(value: Option<&OsStr>) -> io::Result<usize> {
    let native = pse_runtime::math::MathPolicy::default().stack_bytes;
    match value {
        None => Ok(native),
        Some(value) => value
            .to_str()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .map(|value| value.max(native))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "RUST_MIN_STACK must be a positive byte count",
                )
            }),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "native stack selection and actual Linux worker stack controls"
    )]
    use super::*;

    #[test]
    fn native_application_stack_selection_preserves_the_native_minimum_and_larger_explicit_values()
    {
        let native = pse_runtime::math::MathPolicy::default().stack_bytes;
        assert_eq!(selected_value(None).unwrap(), native);
        assert_eq!(selected_value(Some(OsStr::new("1"))).unwrap(), native);
        assert_eq!(
            selected_value(Some(OsStr::new(&(native * 2).to_string()))).unwrap(),
            native * 2
        );
        for invalid in ["0", "", "invalid"] {
            assert!(selected_value(Some(OsStr::new(invalid))).is_err());
        }
    }

    #[cfg(target_os = "linux")]
    #[inline(never)]
    fn actual_stack_extent() -> usize {
        let marker = 0u8;
        let address = std::ptr::from_ref(std::hint::black_box(&marker)).addr();
        std::fs::read_to_string("/proc/self/maps")
            .unwrap()
            .lines()
            .find_map(|line| {
                let (lower, upper) = line.split_whitespace().next()?.split_once('-')?;
                let lower = usize::from_str_radix(lower, 16).ok()?;
                let upper = usize::from_str_radix(upper, 16).ok()?;
                (lower <= address && address < upper).then_some(upper - lower)
            })
            .unwrap()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn native_application_worker_without_rust_min_stack_has_the_selected_native_stack() {
        // Supply absence directly; changing the shared process environment would
        // race other tests and would not reproduce an unconfigured application.
        let selected = selected_value(None).unwrap();
        let runtime = runtime(selected).unwrap();
        let observed = runtime
            .block_on(runtime.spawn(async { actual_stack_extent() }))
            .unwrap();
        assert!(
            observed >= selected,
            "observed worker stack {observed} is smaller than selected native stack {selected}"
        );
    }
}
