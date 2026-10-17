// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit synchronous, actual-thread CPU intervals; never wall-minus-CPU wait.
use std::{
    marker::PhantomData,
    rc::Rc,
    thread::ThreadId,
    time::Duration,
};

/// An explicitly selected synchronous interval on the calling thread.
///
/// This sample cannot be transferred between threads or held by a `Send` task.
/// It measures CPU consumed by this thread, including synchronous nested work;
/// it does not measure children, exclusive owner CPU, asynchronous work or wait.
/// Constructing an ordinary diagnostic producer does not read this clock.
///
/// ```compile_fail
/// use pse_diagnostics::observation::ThreadCpuSample;
/// let sample = ThreadCpuSample::start().unwrap();
/// std::thread::spawn(move || sample.elapsed());
/// ```
#[derive(Debug)]
pub struct ThreadCpuSample {
    owner: ThreadId,
    started: Duration,
    _synchronous: PhantomData<Rc<()>>,
}
impl ThreadCpuSample {
    /// Read this thread's CPU clock only when a synchronous caller selects it.
    /// Unsupported platforms, failed clocks and malformed timestamps are unavailable.
    #[must_use]
    pub fn start() -> Option<Self> {
        Self::start_with(std::thread::current().id(), read_thread_cpu)
    }
    /// Read elapsed CPU on the exact thread that opened this synchronous interval.
    /// A different thread, failed clock or backwards reading is unavailable, not zero.
    #[must_use]
    pub fn elapsed(&self) -> Option<Duration> {
        self.elapsed_with(std::thread::current().id(), read_thread_cpu)
    }
    fn start_with(owner: ThreadId, read: impl FnOnce() -> Option<Duration>) -> Option<Self> {
        Some(Self { owner, started: read()?, _synchronous: PhantomData })
    }
    fn elapsed_with(&self, current: ThreadId, read: impl FnOnce() -> Option<Duration>) -> Option<Duration> {
        if self.owner != current { return None; }
        read()?.checked_sub(self.started)
    }
}
#[cfg(target_os = "linux")]
fn read_thread_cpu() -> Option<Duration> {
    use rustix::time::{ClockId, DynamicClockId, clock_gettime_dynamic};
    let timestamp = clock_gettime_dynamic(DynamicClockId::Known(ClockId::ThreadCPUTime)).ok()?;
    duration_from_parts(timestamp.tv_sec, timestamp.tv_nsec)
}
#[cfg(not(target_os = "linux"))]
fn read_thread_cpu() -> Option<Duration> { None }
#[cfg(any(target_os = "linux", test))]
fn duration_from_parts(seconds: impl TryInto<u64>, nanoseconds: impl TryInto<u32>) -> Option<Duration> {
    let seconds = seconds.try_into().ok()?;
    let nanoseconds = nanoseconds.try_into().ok()?;
    if nanoseconds >= 1_000_000_000 { return None; }
    Some(Duration::new(seconds, nanoseconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    #[test]
    fn thread_cpu_failure_backwards_and_real_zero_remain_distinct() {
        let owner = std::thread::current().id();
        assert!(ThreadCpuSample::start_with(owner, || None).is_none());
        let sample = ThreadCpuSample::start_with(owner, || Some(Duration::from_nanos(20))).unwrap();
        assert_eq!(sample.elapsed_with(owner, || Some(Duration::from_nanos(27))), Some(Duration::from_nanos(7)));
        assert_eq!(sample.elapsed_with(owner, || Some(Duration::from_nanos(20))), Some(Duration::ZERO));
        assert_eq!(sample.elapsed_with(owner, || Some(Duration::from_nanos(19))), None);
        assert_eq!(sample.elapsed_with(owner, || None), None);
    }
    #[test]
    fn thread_cpu_refuses_other_thread_before_reading_its_clock() {
        let owner = std::thread::current().id();
        let other = std::thread::spawn(|| std::thread::current().id()).join().unwrap();
        let reads = Cell::new(0);
        let sample = ThreadCpuSample::start_with(owner, || {
            reads.set(reads.get() + 1); Some(Duration::from_nanos(1))
        }).unwrap();
        assert_eq!(reads.get(), 1);
        assert_eq!(sample.elapsed_with(other, || {
            reads.set(reads.get() + 1); Some(Duration::from_nanos(99))
        }), None);
        assert_eq!(reads.get(), 1);
        assert_eq!(sample.elapsed_with(owner, || {
            reads.set(reads.get() + 1); Some(Duration::from_nanos(2))
        }), Some(Duration::from_nanos(1)));
        assert_eq!(reads.get(), 2);
    }
    #[test]
    fn thread_cpu_rejects_malformed_native_timestamp_parts() {
        assert_eq!(duration_from_parts(-1, 0), None);
        assert_eq!(duration_from_parts(0, -1), None);
        assert_eq!(duration_from_parts(0, 1_000_000_000), None);
        assert_eq!(duration_from_parts(2, 999_999_999), Some(Duration::new(2, 999_999_999)));
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn thread_cpu_actual_owned_work_has_available_monotonic_elapsed() {
        let sample = ThreadCpuSample::start().unwrap();
        let first = sample.elapsed().unwrap();
        let mut value = std::hint::black_box(1u64);
        for index in 0..10_000u64 {
            value = std::hint::black_box(value.wrapping_mul(31).wrapping_add(index));
        }
        std::hint::black_box(value);
        let second = sample.elapsed().unwrap();
        assert!(second >= first);
    }
}
