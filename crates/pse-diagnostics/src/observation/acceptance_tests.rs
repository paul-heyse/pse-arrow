// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Revealing integration controls for producer isolation and independent collection.
use super::*;

#[derive(Debug, Default)]
struct TestClock(AtomicU64);
impl Clock for TestClock {
    fn now(&self) -> Duration { Duration::from_millis(self.0.load(Ordering::Relaxed)) }
}
impl TestClock {
    fn set(&self, millis: u64) { self.0.store(millis, Ordering::Relaxed); }
}
fn setup(policy: Policy) -> (Arc<ObservationHub>, Arc<TestClock>) {
    let clock = Arc::new(TestClock::default());
    (ObservationHub::with_clock(policy, clock.clone()), clock)
}
fn record(producer: &Producer, family: &str, value: i64) -> CoherentRecord {
    producer.record(family, "native_event", Some("physical"), Some("infinity"), vec![
        RecordField { name: "iteration".into(), unit: "iteration".into(),
            value: Value::Integer(value), availability: None },
    ])
}
fn metric<'a>(snapshot: &'a OperationSnapshot, name: &str) -> &'a MetricSnapshot {
    snapshot.metrics.iter().find(|metric| metric.name == name).unwrap()
}

#[test]
fn first_retained_record_after_an_issued_prefix_gap_is_partial() {
    let (hub, clock) = setup(Policy::default());
    let operation = hub.start("source_prefix.control");
    let producer = operation.context().producer(ProducerRole::NativeIteration, "native iteration");
    let unobserved = record(&producer, "native_iteration.v1", 1);
    clock.set(750);
    let retained = record(&producer, "native_iteration.v1", 2);
    assert_eq!((unobserved.sequence, retained.sequence), (1, 2));
    assert_eq!((unobserved.family_sequence, retained.family_sequence), (Some(1), Some(2)));
    assert!(producer.publish(&retained));
    let snapshot = operation.reader().snapshot(WINDOW);
    let source = &snapshot.producers[0];
    assert_eq!(source.coverage.accepted, 1);
    assert!(source.coverage.partial);
    assert_eq!(source.current.as_ref().unwrap().observed_at, Duration::from_millis(750));
    assert_eq!(source.current.as_ref().unwrap().values[0].value, Value::Integer(2));
}

#[test]
fn prepared_and_legacy_updates_refuse_population_or_unit_reinterpretation() {
    let (hub, _) = setup(Policy::default());
    let operation = hub.start("population_contract.control");
    let producer = operation.context().producer(ProducerRole::OriginalOracle, "oracle");
    let context = producer.context();
    let counter = producer.fact("prepared_counter", "calls", FactKind::Counter);
    counter.count(2);
    context.count("prepared_counter", "seconds", 100);
    context.gauge("prepared_counter", "calls", 200.0);
    context.count_value("prepared_counter", "calls", 300, 1);
    producer.fact("prepared_counter", "calls", FactKind::Gauge).integer(400);
    producer.fact("prepared_counter", "seconds", FactKind::Counter).count(500);
    context.count("prepared_counter", "calls", 3);
    context.count("legacy_counter", "calls", 7);
    producer.fact("legacy_counter", "calls", FactKind::Gauge).integer(600);
    producer.fact("legacy_counter", "calls", FactKind::AbsoluteCounter).count_value(700, 1);
    producer.fact("legacy_counter", "calls", FactKind::Counter).count(1);
    context.gauge("legacy_gauge", "ratio", 1.5);
    producer.fact("legacy_gauge", "ratio", FactKind::Counter).count(800);
    let gauge = producer.fact("prepared_gauge", "ratio", FactKind::Gauge);
    gauge.gauge(2.5);
    context.count("prepared_gauge", "ratio", 900);
    let duration = producer.fact("prepared_duration", "nanoseconds", FactKind::Duration);
    duration.duration(Duration::from_nanos(11));
    context.count("prepared_duration", "nanoseconds", 1000);
    producer.fact("prepared_duration", "nanoseconds", FactKind::Counter).count(1100);
    producer.fact("invalid_duration", "seconds", FactKind::Duration).duration(Duration::from_secs(2));
    let snapshot = operation.reader().snapshot(WINDOW);
    assert_eq!(metric(&snapshot, "prepared_counter").unit, "calls");
    assert_eq!(metric(&snapshot, "prepared_counter").latest, Value::Count(5));
    assert_eq!(metric(&snapshot, "prepared_counter").cumulative, Value::Count(5));
    assert_eq!(metric(&snapshot, "legacy_counter").cumulative, Value::Count(8));
    assert_eq!(metric(&snapshot, "legacy_gauge").latest, Value::Number(1.5));
    assert_eq!(metric(&snapshot, "prepared_gauge").latest, Value::Number(2.5));
    assert_eq!(metric(&snapshot, "prepared_duration").cumulative, Value::Count(11));
    assert_eq!(metric(&snapshot, "prepared_duration").duration_summary.as_ref().unwrap().samples, 1);
    assert!(matches!(metric(&snapshot, "invalid_duration").latest, Value::Unavailable(_)));
    assert!(snapshot.coverage.refused_items >= 10);
    assert!(snapshot.metrics.iter().all(|metric| metric.producer == Some(producer.id())));
}

#[test]
fn legacy_and_prepared_absolute_resets_preserve_subtotals_and_actual_spans() {
    for prepared in [false, true] {
        let (hub, clock) = setup(Policy::default());
        let operation = hub.start("absolute_reset.control");
        let producer = operation.context().producer(ProducerRole::NativeCallback, "callback");
        let fact = producer.fact("completed", "calls", FactKind::AbsoluteCounter);
        let update = |value, generation| {
            if prepared { fact.count_value(value, generation); }
            else { producer.context().count_value("completed", "calls", value, generation); }
        };
        clock.set(1200); update(100, 1);
        clock.set(1400); update(104, 1);
        clock.set(1600); update(1, 2);
        let at_reset = operation.reader().snapshot(WINDOW);
        let reset = metric(&at_reset, "completed");
        assert_eq!((reset.cumulative.clone(), reset.latest.clone(), reset.resets),
            (Value::Count(4), Value::Count(1), 1));
        let interval = &reset.counter_window.as_ref().unwrap().intervals[0];
        assert_eq!((interval.first_ns, interval.last_ns, interval.delta), (1_200_000_000, 1_600_000_000, Some(4)));
        assert!(interval.reset);
        assert!(reset.history.iter().any(|point| point.reset));
        clock.set(1800); update(4, 2);
        let final_snapshot = operation.reader().snapshot(WINDOW);
        let final_metric = metric(&final_snapshot, "completed");
        assert_eq!(final_metric.cumulative, Value::Count(7));
        assert_eq!(final_metric.recent, Value::Count(7));
        assert_eq!(final_metric.resets, 1);
        assert_eq!(final_metric.generation, 2);
        assert!(final_metric.cumulative_partial);
        let interval = &final_metric.counter_window.as_ref().unwrap().intervals[0];
        assert_eq!((interval.first_ns, interval.last_ns, interval.delta), (1_200_000_000, 1_800_000_000, Some(7)));
        assert_eq!(final_metric.history.last().unwrap().last_observed_at, Duration::from_millis(1800));
    }
}

#[test]
fn prepared_duration_intersections_retain_real_earliest_and_latest_boundaries() {
    let (hub, clock) = setup(Policy::default());
    let operation = hub.start("duration_span.control");
    let producer = operation.context().producer(ProducerRole::OriginalOracle, "oracle");
    let fact = producer.fact("inclusive_wall", "nanoseconds", FactKind::Duration);
    clock.set(1900); fact.duration(Duration::from_millis(400));
    clock.set(1950); fact.duration(Duration::from_millis(100));
    let snapshot = operation.reader().snapshot(WINDOW);
    let metric = metric(&snapshot, "inclusive_wall");
    assert_eq!(metric.cumulative, Value::Count(500_000_000));
    let window = metric.counter_window.as_ref().unwrap();
    assert!(window.duration);
    assert_eq!(window.intervals.len(), 1);
    assert_eq!((window.intervals[0].first_ns, window.intervals[0].last_ns), (1_500_000_000, 1_950_000_000));
    assert_eq!(metric.duration_summary.as_ref().unwrap().samples, 2);
}

#[derive(Debug)]
struct PoisonedTap {
    state: Mutex<()>,
    fail_selection: bool,
    fail_description: bool,
}
impl PoisonedTap {
    fn new(fail_selection: bool, fail_description: bool) -> Self {
        let state = Mutex::new(());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = state.lock().unwrap();
            panic!("fixture collector owner failed");
        }));
        assert!(result.is_err());
        Self { state, fail_selection, fail_description }
    }
    fn check(&self) { drop(self.state.lock().unwrap()); }
}
impl RecordTap for PoisonedTap {
    fn selected(&self, _: &str) -> bool { if self.fail_selection { self.check(); } true }
    fn thread_cpu(&self) -> bool { self.check(); true }
    fn is_active(&self) -> bool { self.check(); true }
    fn reserve_owner_bytes(&self, _: usize) -> Option<OwnerReservation> { self.check(); None }
    fn describe(&self, _: &ProducerDescriptor) { if self.fail_description { self.check(); } }
    fn observe(&self, _: &CoherentRecord) { self.check(); }
}
#[test]
fn poisoned_optional_tap_cannot_unwind_into_scientific_work() {
    for (fail_selection, fail_description) in [(true, false), (false, true), (false, false)] {
        let (hub, _) = setup(Policy::default());
        let operation = hub.start("optional_tap_failure.control");
        let context = operation.context().with_record_tap(Arc::new(PoisonedTap::new(fail_selection, fail_description)));
        let producer = context.producer(ProducerRole::NativeIteration, "native iteration");
        let scientific_work = || -> Result<i64, &'static str> {
            assert!(!producer.thread_cpu_requested());
            assert!(producer.reserve_owner_bytes(usize::MAX).is_none());
            if fail_selection { assert!(!producer.detail_selected("native_iteration.v1")); }
            let event = record(&producer, "native_iteration.v1", 3);
            assert!(producer.publish(&event), "ordinary record retention remains independent");
            Ok(42)
        };
        assert_eq!(scientific_work(), Ok(42));
        let snapshot = operation.reader().snapshot(WINDOW);
        assert_eq!(snapshot.producers[0].current.as_ref().unwrap().values[0].value, Value::Integer(3));
        assert!(snapshot.coverage.dropped_updates >= 1);
        let off = ObservationHub::new(Policy { enabled: false, ..Policy::default() });
        let op = off.start("selected_admission_failure.control");
        let unavailable = op.context().with_record_tap(Arc::new(PoisonedTap::new(false, false)))
            .producer(ProducerRole::Evaluator, "evaluator");
        assert!(!unavailable.enabled(), "poisoned selected-owner admission is unavailable");
        assert!(!unavailable.publish(&record(&unavailable, "selected.v1", 1)));
    }
}

#[derive(Debug)]
struct SelectedTap {
    budget: OwnerBudget,
    records: Mutex<Vec<CoherentRecord>>,
    descriptors: Mutex<Vec<ProducerDescriptor>>,
}
impl Default for SelectedTap {
    fn default() -> Self { Self { budget: OwnerBudget::new(32 * 1024), records: Mutex::new(Vec::new()), descriptors: Mutex::new(Vec::new()) } }
}
impl RecordTap for SelectedTap {
    fn selected(&self, family: &str) -> bool { family == "selected.v1" }
    fn reserve_owner_bytes(&self, bytes: usize) -> Option<OwnerReservation> { self.budget.reserve(bytes) }
    fn describe(&self, producer: &ProducerDescriptor) { self.descriptors.lock().unwrap().push(producer.clone()); }
    fn observe(&self, record: &CoherentRecord) { self.records.lock().unwrap().push(record.clone()); }
}
#[test]
fn unselected_families_do_not_create_selected_family_sequence_gaps_or_descriptors() {
    let (hub, _) = setup(Policy::default());
    let operation = hub.start("family_selection.control");
    let tap = Arc::new(SelectedTap::default());
    let producer = operation.context().with_record_tap(tap.clone())
        .producer(ProducerRole::Evaluator, "evaluator");
    for (family, value) in [("unselected.v1", 1), ("selected.v1", 2), ("unselected.v1", 3), ("selected.v1", 4)] {
        assert!(producer.publish(&record(&producer, family, value)));
    }
    let records = tap.records.lock().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!((records[0].sequence, records[1].sequence), (2, 4));
    assert_eq!((records[0].family_sequence, records[1].family_sequence), (Some(1), Some(2)));
    assert!(records.iter().all(|record| record.family == "selected.v1"));
    let descriptors = tap.descriptors.lock().unwrap();
    assert_eq!(descriptors.len(), 2, "unselected events offer no producer descriptors");
    assert!(descriptors.iter().all(|descriptor| descriptor.id == producer.id()));
    let snapshot = operation.reader().snapshot(WINDOW);
    assert!(!snapshot.producers[0].coverage.partial);
}

#[test]
fn refused_or_detached_ordinary_child_keeps_real_clock_and_separate_selected_admission() {
    for detach_hub in [false, true] {
        let (hub, clock) = setup(Policy { max_operations: 1, ..Policy::default() });
        let operation = hub.start("ordinary_parent.control");
        let tap = Arc::new(SelectedTap::default());
        let context = operation.context().with_record_tap(tap.clone());
        let weak = Arc::downgrade(&hub);
        if detach_hub { drop(hub); }
        clock.set(750);
        let child = context.child("refused_child.control", Relationship::Nested);
        assert!(child.context().node.is_none());
        let retained_before_selected = weak.upgrade().map(|hub| hub.retained_bytes());
        let producer = child.context().producer(ProducerRole::Evaluator, "selected evaluator");
        assert!(producer.enabled(), "finite selected ownership survives ordinary admission refusal");
        let reservation = producer.reserve_owner_bytes(1024).unwrap();
        let event = record(&producer, "selected.v1", 7);
        assert_eq!(event.observed_at, Duration::from_millis(750));
        assert_eq!(event.family_sequence, Some(1));
        assert!(!producer.publish(&event), "ordinary retention is absent independently of selected delivery");
        assert_eq!(tap.records.lock().unwrap().len(), 1);
        assert_eq!(tap.descriptors.lock().unwrap()[0].id, producer.id());
        if let Some(hub) = weak.upgrade() { assert_eq!(Some(hub.retained_bytes()), retained_before_selected); }
        else { assert!(detach_hub); }
        drop(reservation);
    }
}

#[test]
fn temporary_gauge_unavailability_does_not_erase_its_population_kind() {
    let (hub, _) = setup(Policy::default());
    let operation = hub.start("temporary_absence.control");
    let producer = operation.context().producer(ProducerRole::OriginalOracle, "oracle");
    let context = producer.context();
    context.gauge("retained_gauge", "calls", 1.5);
    context.unavailable("retained_gauge", "trial did not supply this checkpoint");
    context.count("retained_gauge", "calls", 9);
    producer.fact("retained_gauge", "calls", FactKind::Counter).count(10);
    let absent = operation.reader().snapshot(WINDOW);
    let unchanged = metric(&absent, "retained_gauge");
    assert!(matches!(unchanged.latest, Value::Unavailable(_)));
    assert!(matches!(unchanged.cumulative, Value::Unavailable(_)));
    assert!(unchanged.counter_window.is_none());
    context.gauge("retained_gauge", "calls", 2.5);
    context.unavailable("initial_unknown", "not activated yet");
    producer.fact("initial_unknown", "calls", FactKind::Counter).count(1);
    let restored = operation.reader().snapshot(WINDOW);
    assert_eq!(metric(&restored, "retained_gauge").latest, Value::Number(2.5));
    assert_eq!(metric(&restored, "initial_unknown").cumulative, Value::Count(1),
        "initial absence never invents a population kind");
}
