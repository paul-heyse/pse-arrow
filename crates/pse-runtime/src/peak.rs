// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pool accounting and process observations are separate measurements (ADR-0046).

use crate::RuntimeError;

/// Authoritative native resource observation fields for mechanical boundary projection.
#[macro_export]
macro_rules! resource_report_fields {
    ($emit:ident) => {
        $emit! {
            /// Configured finite accounted limit.
            limit_bytes: usize,
            /// Maximum accounted claims since this shared runtime was created.
            pool_peak_bytes: usize,
            /// Current accounted claims across every query and platform consumer.
            pool_reserved_now: usize,
            /// Whole-process peak resident bytes; None on unsupported platforms.
            process_peak_rss_bytes: Option<u64>;
            consumers: top_consumers;
            caches: caches;
        }
    };
}
macro_rules! report {
    ($( $(#[$meta:meta])* $field:ident: $ty:ty),*; consumers: $consumers:ident; caches: $caches:ident;) => {
        /// Pool counters do not claim global allocator coverage.
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct ResourceReport {
            $( $(#[$meta])* pub $field: $ty, )*
            /// Largest current consumers, decreasing by bytes then increasing by owner name.
            pub $consumers: Vec<(String, usize)>,
            /// Native retained capacities and hits, without cloning cached values.
            pub $caches: Vec<pse_engine::cache_service::CacheReport>,
        }
    };
}
resource_report_fields!(report);

#[cfg(target_os = "linux")]
pub(crate) fn process_peak_rss() -> Result<Option<u64>, RuntimeError> {
    let status = std::fs::read_to_string("/proc/self/status").map_err(|source| {
        RuntimeError::Infrastructure {
            op: "read process peak resident memory".to_owned(),
            source: Box::new(source),
        }
    })?;
    parse_peak_rss(&status).map(Some)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn process_peak_rss() -> Result<Option<u64>, RuntimeError> {
    Ok(None)
}

#[cfg(any(target_os = "linux", test))]
fn parse_peak_rss(status: &str) -> Result<u64, RuntimeError> {
    let mut fields = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .unwrap_or("")
        .split_whitespace();
    let value = fields.next().and_then(|value| value.parse::<u64>().ok());
    if fields.next() != Some("kB") || fields.next().is_some() {
        return Err(RuntimeError::Internal {
            message: "process VmHWM metric is missing or has an unknown unit".to_owned(),
        });
    }
    value
        .and_then(|value| value.checked_mul(1024))
        .ok_or_else(|| RuntimeError::Internal {
            message: "process VmHWM metric is invalid or overflows bytes".to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_peak_units_and_overflow_are_checked() {
        assert_eq!(
            parse_peak_rss("Name: test\nVmHWM: 123 kB\n").expect("valid metric"),
            125_952
        );
        for text in [
            "",
            "VmHWM: -1 kB",
            "VmHWM: 123 MB",
            "VmHWM: 18446744073709551615 kB",
        ] {
            assert!(parse_peak_rss(text).is_err(), "{text}");
        }
    }
}
