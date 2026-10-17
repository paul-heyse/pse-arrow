// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-host operational attachment identities and clock mappings, never attestations.

use pse_diagnostics::observation::{Clock, OperationId};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

/// Actual process locator. PID reuse must be checked before attaching an optional sampler.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProcessLocator {
    /// Actual operating-system process ID.
    pub pid: u32,
    /// Linux process start ticks; absence does not authorize attachment.
    pub start_ticks: Option<u64>,
    /// Current executable path for display/symbol discovery, not a content proof.
    pub executable: Option<PathBuf>,
}
impl ProcessLocator {
    pub(crate) fn current() -> Self {
        Self::for_pid(std::process::id())
    }
    /// Observe a caller-owned process without fabricating a diagnostic operation.
    pub(crate) fn for_pid(pid: u32) -> Self {
        Self {
            pid,
            start_ticks: process_start_ticks(pid),
            executable: std::fs::read_link(format!("/proc/{pid}/exe")).ok(),
        }
    }
    /// Observe whether the recorded process still has this actual start identity.
    pub fn is_current(&self) -> bool {
        self.start_ticks.is_some() && process_start_ticks(self.pid) == self.start_ticks
    }
}
/// Mapping of one observation clock checkpoint to the same-host Linux monotonic clock.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClockMapping {
    /// Observation-clock sample immediately preceding the OS clock read.
    #[schemars(with = "pse_diagnostics::ClosedDuration")]
    pub observation_at: Duration,
    /// CLOCK_MONOTONIC nanoseconds, not Unix time or process CPU time.
    pub monotonic_nanoseconds: u64,
    /// Elapsed observation time bracketing the clock read; alignment is not exact.
    pub uncertainty_nanoseconds: u64,
}
impl ClockMapping {
    /// Current read time projected into the producer clock, with uncertainty retained separately.
    pub fn observation_now(&self) -> Option<Duration> {
        #[cfg(target_os = "linux")]
        {
            let now = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
            let nanos = u64::try_from(now.tv_sec)
                .ok()?
                .checked_mul(1_000_000_000)?
                .checked_add(u64::try_from(now.tv_nsec).ok()?)?;
            if nanos >= self.monotonic_nanoseconds {
                self.observation_at
                    .checked_add(Duration::from_nanos(nanos - self.monotonic_nanoseconds))
            } else {
                self.observation_at
                    .checked_sub(Duration::from_nanos(self.monotonic_nanoseconds - nanos))
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }
    /// Project a timestamp without discarding overflow or the independent uncertainty.
    pub fn monotonic_timestamp(&self, timestamp: Duration) -> Option<u64> {
        let nanos = u64::try_from(timestamp.abs_diff(self.observation_at).as_nanos()).ok()?;
        if timestamp >= self.observation_at {
            self.monotonic_nanoseconds.checked_add(nanos)
        } else {
            self.monotonic_nanoseconds.checked_sub(nanos)
        }
    }
}
/// One actual entered native-thread scope. Async migration is not inferred from this record.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ThreadExecution {
    /// Actual diagnostic operation executing in this scope.
    pub operation: OperationId,
    /// Linux OS thread ID, distinct from Rust's opaque ThreadId.
    pub tid: u32,
    /// OS thread start identity; missing means thread-specific attachment is unavailable.
    pub start_ticks: Option<u64>,
    /// Actual scope entry on the process observation clock.
    #[schemars(with = "pse_diagnostics::ClosedDuration")]
    pub started_at: Duration,
    /// Actual return/drain from this scope, not scientific acceptance.
    #[schemars(with = "Option<pse_diagnostics::ClosedDuration>")]
    pub finished_at: Option<Duration>,
}
#[derive(Debug)]
pub(crate) struct ProcessClock(Instant);
impl ProcessClock {
    pub(crate) fn new() -> Self {
        Self(Instant::now())
    }
    pub(crate) fn mapping(&self) -> Option<ClockMapping> {
        #[cfg(target_os = "linux")]
        {
            let observation_at = self.now();
            let os = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
            let end = self.now();
            Some(ClockMapping {
                observation_at,
                monotonic_nanoseconds: u64::try_from(os.tv_sec)
                    .ok()?
                    .checked_mul(1_000_000_000)?
                    .checked_add(u64::try_from(os.tv_nsec).ok()?)?,
                uncertainty_nanoseconds: u64::try_from(
                    end.saturating_sub(observation_at).as_nanos(),
                )
                .ok()?,
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }
}
impl Clock for ProcessClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }
}
pub(crate) fn process_start_ticks(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}
pub(crate) fn thread_identity() -> Option<(u32, Option<u64>)> {
    #[cfg(target_os = "linux")]
    {
        let tid = u32::try_from(rustix::thread::gettid().as_raw_pid()).ok()?;
        Some((tid, process_start_ticks(tid)))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attachment_rejects_reused_or_unknown_process_identity() {
        let mut locator = ProcessLocator::current();
        assert!(locator.is_current());
        locator.start_ticks = locator.start_ticks.map(|ticks| ticks.wrapping_add(1));
        assert!(!locator.is_current());
        locator.start_ticks = None;
        assert!(!locator.is_current());
    }
    #[test]
    fn monotonic_mapping_preserves_offset_and_overflow() {
        let mapping = ClockMapping {
            observation_at: Duration::from_secs(5),
            monotonic_nanoseconds: 8_000_000_000,
            uncertainty_nanoseconds: 7,
        };
        assert_eq!(
            mapping.monotonic_timestamp(Duration::from_secs(3)),
            Some(6_000_000_000)
        );
        assert_eq!(
            mapping.monotonic_timestamp(Duration::from_secs(7)),
            Some(10_000_000_000)
        );
        assert_eq!(mapping.monotonic_timestamp(Duration::MAX), None);
        let clock = ProcessClock::new();
        let mapping = clock.mapping().unwrap();
        assert!(mapping.observation_now().unwrap() >= mapping.observation_at);
        assert!(
            mapping.observation_now().unwrap()
                <= clock.now() + Duration::from_nanos(mapping.uncertainty_nanoseconds)
        );
    }
}
