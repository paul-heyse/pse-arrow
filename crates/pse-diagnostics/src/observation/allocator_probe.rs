// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private I35 inquiry over the actual ordinary diagnostic owners.
use super::*;
use allocation_counter::AllocationInfo;

const WARM: u64 = 128;
const UPDATES: u64 = 10_000;

#[derive(Default)]
struct ProbeClock(AtomicU64);
impl Clock for ProbeClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Relaxed))
    }
}

#[derive(Debug)]
struct Sample {
    elapsed: Duration,
    allocations: AllocationInfo,
}
fn measured<T>(work: impl FnOnce() -> T) -> (T, Sample) {
    let start = Instant::now();
    let mut result = None;
    let allocations = allocation_counter::measure(|| result = Some(work()));
    (
        result.unwrap(),
        Sample {
            elapsed: start.elapsed(),
            allocations,
        },
    )
}

struct Ordinary {
    hub: Arc<ObservationHub>,
    clock: Arc<ProbeClock>,
    operation: Option<OperationGuard>,
    producer: Producer,
    count: PreparedFact,
    elapsed: PreparedFact,
    seconds: PreparedFact,
    events: u64,
}
impl Ordinary {
    fn new() -> Self {
        let clock = Arc::new(ProbeClock::default());
        let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
        let operation = hub.start("allocator-inquiry.ordinary");
        operation
            .context()
            .declare_summary("ordinary residual observation");
        let producer = operation
            .context()
            .producer(ProducerRole::NativeCallback, "residual");
        let count = producer.fact("completed", "count", FactKind::Counter);
        let elapsed = producer.fact("elapsed", "nanoseconds", FactKind::Duration);
        let seconds = producer.fact("seconds", "seconds", FactKind::Gauge);
        Self {
            hub,
            clock,
            operation: Some(operation),
            producer,
            count,
            elapsed,
            seconds,
            events: 0,
        }
    }
    fn publish(&mut self, count: u64, phases: bool) {
        for _ in 0..count {
            // Cross real bounded-history buckets deterministically. This clock is
            // fixture time; Sample.elapsed is independent measured process wall time.
            self.clock.0.fetch_add(1000, Ordering::Relaxed);
            let phase = phases.then(|| self.producer.context().phase("callback.residual"));
            self.events += 1;
            assert!(self.producer.publish_facts(&[
                FactUpdate::Count {
                    fact: &self.count,
                    delta: 1
                },
                FactUpdate::Duration {
                    fact: &self.elapsed,
                    elapsed: Duration::from_millis(1)
                },
                FactUpdate::Gauge {
                    fact: &self.seconds,
                    value: self.events as f64 * 0.001
                },
            ]));
            self.clock.0.fetch_add(1, Ordering::Relaxed);
            drop(phase);
        }
    }
    fn snapshot(&self) -> usize {
        let snapshot = self.producer.context().reader().snapshot(WINDOW);
        assert!(snapshot.coverage.available);
        assert_eq!(snapshot.coverage.refused_items, 0);
        assert_eq!(snapshot.coverage.dropped_updates, 0);
        assert!(snapshot.phases.is_empty());
        let metric = |name| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .unwrap()
        };
        assert_eq!(metric("completed").cumulative, Value::Count(self.events));
        assert_eq!(
            metric("elapsed").cumulative,
            Value::Count(self.events * 1_000_000)
        );
        assert_eq!(
            metric("seconds").latest,
            Value::Number(self.events as f64 * 0.001)
        );
        assert!(self.hub.retained_bytes() <= self.hub.policy().budget_bytes);
        self.hub.retained_bytes()
    }
    fn finish(&mut self) {
        self.operation.take().unwrap().finish(Outcome::Completed);
        assert_eq!(
            self.producer.context().reader().snapshot(WINDOW).outcome,
            Some(Outcome::Completed)
        );
    }
}

fn resident_bytes() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    line.split_whitespace()
        .nth(1)?
        .parse::<usize>()
        .ok()?
        .checked_mul(1024)
}

#[test]
#[allow(
    clippy::print_stderr,
    reason = "bounded I35 inquiry reports actual calling-thread allocation and operation wall observations"
)]
fn ordinary_diagnostic_allocation_and_complete_cleanup_probe() {
    // Exact allocation-counter 0.8.1 wraps Rust System and counts only this thread.
    // There is no native/FFI scientific work here. RSS is whole-process residency,
    // not Rust live bytes, a reservation, a child measurement or allocator-only cost.
    // Do not nest measurements: its AddAssign sums nested maxima, which would not
    // supply the complete operation's actual maximum live counted allocation.
    allocation_counter::measure(|| {});
    for index in 0..5 {
        let before = resident_bytes();
        let (retained, complete) = measured(|| {
            let mut ordinary = Ordinary::new();
            ordinary.publish(WARM + UPDATES, true);
            let retained = ordinary.snapshot();
            ordinary.finish();
            drop(ordinary);
            retained
        });
        assert_eq!(complete.allocations.count_current, 0);
        assert_eq!(complete.allocations.bytes_current, 0);
        let after_complete = resident_bytes();

        let (mut ordinary, setup) = measured(Ordinary::new);
        let (_, warm) = measured(|| ordinary.publish(WARM, true));
        let (_, publish) = measured(|| ordinary.publish(UPDATES, true));
        let (_, publication_only) = measured(|| ordinary.publish(UPDATES, false));
        let (phased_retained, snapshot) = measured(|| ordinary.snapshot());
        let (_, terminal) = measured(|| ordinary.finish());
        let weak = Arc::downgrade(&ordinary.hub);
        let (_, cleanup) = measured(|| drop(ordinary));
        assert!(
            weak.upgrade().is_none(),
            "all hub and scientific-independent diagnostic owners dropped"
        );
        drop(weak);
        let after_cleanup = resident_bytes();
        eprintln!(
            "I35 sample={index} updates={UPDATES} warm={WARM} accepted_complete={} accepted_phased={} complete={complete:?} setup={setup:?} warmup={warm:?} phase_and_publish={publish:?} publication_only={publication_only:?} snapshot={snapshot:?} terminal={terminal:?} cleanup={cleanup:?} retained_complete={retained} retained_phased={phased_retained} rss_before={before:?} rss_after_complete={after_complete:?} rss_after_cleanup={after_cleanup:?}",
            WARM + UPDATES,
            WARM + 2 * UPDATES
        );
        // Read each timing independently; summing phase clocks does not replace
        // the separately measured whole operation or isolate allocator cost.
        std::hint::black_box((
            complete.elapsed,
            setup.elapsed,
            warm.elapsed,
            publish.elapsed,
            publication_only.elapsed,
            snapshot.elapsed,
            terminal.elapsed,
            cleanup.elapsed,
        ));
    }
}
