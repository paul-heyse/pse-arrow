// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Revealing acceptance cases for independently admitted finite detail collection.

use super::*;
use pse_diagnostics::observation::{
    ObservationHub, Policy, Producer, ProducerRole, RecordField, Value,
};

fn selection(output: PathBuf) -> DetailRequest {
    DetailRequest {
        version: pse_model::document::Version,
        families: vec!["iteration.v1".into()],
        requirement: DetailRequirement::Complete,
        sample_every: 1,
        boundary_thread_cpu: false,
        managed_budget: None,
        expiry_monotonic_ns: None,
        duration: Duration::from_secs(30),
        output,
        budget: CaptureBudget::default(),
    }
}

fn iteration(producer: &Producer, value: i64) -> CoherentRecord {
    producer.record(
        "iteration.v1",
        "ordinary",
        Some("submitted"),
        None,
        vec![RecordField {
            name: "iteration".into(),
            unit: "iteration".into(),
            value: Value::Integer(value),
            availability: None,
        }],
    )
}

fn iteration_values(page: &DetailRecordPage) -> Vec<i64> {
    page.records
        .iter()
        .map(|record| match record.values[0].value {
            Value::Integer(value) => value,
            ref other => panic!("expected an exact iteration, got {other:?}"),
        })
        .collect()
}

#[test]
fn finish_waits_for_admitted_offer_and_drains_before_terminal_counts() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("science");
    let session = start_detailed_collection(
        operation.context(),
        selection(directory.path().join("drain")),
    )
    .unwrap();
    let producer = session.context().producer(ProducerRole::NativeIteration, "native");
    let record = iteration(&producer, 7);
    let descriptor = ProducerDescriptor {
        id: producer.id(),
        role: ProducerRole::NativeIteration,
        name: "native".into(),
    };
    let tap = Arc::clone(&session.tap);
    let offered = tap.enter().expect("offer admitted before close");
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();
    let finishing = std::thread::spawn(move || {
        finished_tx.send(session.finish()).unwrap();
    });
    let watchdog = Instant::now() + Duration::from_secs(5);
    while !tap.closed.load(Ordering::Acquire) {
        assert!(Instant::now() < watchdog, "finish did not request close");
        std::thread::yield_now();
    }
    assert!(matches!(finished_rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));
    assert_eq!(tap.in_flight.load(Ordering::Acquire), 1);

    // This offer entered before close. Its descriptor and row must still drain;
    // rejecting new offers must not erase work already admitted by this guard.
    tap.describe_selected(&descriptor);
    tap.observe_selected(&record, 0);
    drop(offered);
    let document = finished_rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    finishing.join().unwrap();
    assert_eq!(document.outcome, DetailOutcome::Complete);
    assert_eq!(document.end, Some(DetailEnd::OwnerFinished));
    let coverage = &document.families[0];
    assert_eq!((coverage.observed, coverage.queued, coverage.acknowledged), (1, 1, 1));
    assert_eq!((coverage.unacknowledged, coverage.dropped), (0, 0));
    let page = read_detail_records(&document.records, 0, 8, 1 << 20).unwrap();
    assert_eq!(iteration_values(&page), [7]);
}

#[test]
fn unselected_producers_do_not_consume_selected_metadata_capacity() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("science");
    let session = start_detailed_collection(
        operation.context(),
        selection(directory.path().join("filtered-producers")),
    )
    .unwrap();
    let context = session.context();
    for index in 0..300 {
        let producer = context.producer(ProducerRole::Workflow, &format!("unselected-{index}"));
        producer.publish(&producer.record("unselected.v1", "work", None, None, vec![]));
    }
    let selected = context.producer(ProducerRole::NativeIteration, "selected");
    selected.publish(&iteration(&selected, 1));
    let document = session.finish().unwrap();
    assert_eq!(document.outcome, DetailOutcome::Complete);
    assert_eq!(document.metadata_dropped, 0);
    assert!(!document.source_partial);
    let page = read_detail_records(&document.records, 0, 8, 1 << 20).unwrap();
    assert_eq!(page.producers.len(), 1);
    assert_eq!(page.producers[0].id, selected.id());
    assert_eq!(iteration_values(&page), [1]);
}

#[test]
fn selected_child_has_real_clock_and_owner_budget_after_ordinary_refusal() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy { max_operations: 1, ..Policy::default() });
    let operation = hub.start("science");
    assert!(operation.context().is_active());
    let session = start_detailed_collection(
        operation.context(),
        selection(directory.path().join("refused-child")),
    )
    .unwrap();
    let child = session.context().child("native", pse_diagnostics::Relationship::Nested);
    assert!(!child.context().is_active(), "ordinary operation admission must refuse this child");
    let producer = child.context().producer(ProducerRole::Evaluator, "evaluator");
    assert!(producer.enabled(), "selected detail remains independently admitted");
    let owner_rows = producer.reserve_owner_bytes(4096).expect("selected owner storage remains available");
    let first = iteration(&producer, 1);
    assert_eq!(first.family_sequence, Some(1));
    assert!(!first.observed_at.is_zero(), "a refused ordinary child still has a real source clock");
    assert!(!producer.publish(&first), "ordinary publication must remain refused");
    let clock_interval = Instant::now();
    while clock_interval.elapsed() < Duration::from_millis(1) {
        std::thread::yield_now();
    }
    let second = iteration(&producer, 2);
    assert_eq!(second.family_sequence, Some(2));
    assert!(second.observed_at > first.observed_at);
    assert!(!producer.publish(&second));
    drop(owner_rows);
    let document = session.finish().unwrap();
    assert_eq!(document.outcome, DetailOutcome::Complete);
    assert_eq!(document.families[0].acknowledged, 2);
    let page = read_detail_records(&document.records, 0, 8, 1 << 20).unwrap();
    assert_eq!(iteration_values(&page), [1, 2]);
    assert!(page.records[1].observed_at > page.records[0].observed_at);
    assert!(operation.context().is_active());
}

#[test]
fn terminal_readback_and_finish_agree_on_complete_prefix_extent() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("science");
    let session = start_detailed_collection(
        operation.context(),
        selection(directory.path().join("prefix")),
    )
    .unwrap();
    let producer = session.context().producer(ProducerRole::NativeIteration, "native");
    producer.publish(&iteration(&producer, 1));
    let finished = session.finish().unwrap();
    let read = read_detailed_collection(&finished.records, 1 << 20).unwrap();
    assert_eq!(finished.outcome, DetailOutcome::Complete);
    assert_eq!(read.outcome, finished.outcome);
    assert_eq!(read.end, finished.end);
    assert_eq!(read.readable_prefix_bytes, finished.readable_prefix_bytes);
    assert_eq!(read.readable_prefix_bytes, std::fs::metadata(&finished.records).unwrap().len());
    assert_eq!(read.families[0].acknowledged, finished.families[0].acknowledged);
}

#[cfg(target_os = "linux")]
#[test]
fn delegated_absolute_expiry_refuses_delayed_owner_without_renewal_or_output() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("science");
    let mut request = selection(directory.path().join("expired"));
    request.expiry_monotonic_ns = Some(monotonic_ns().unwrap().saturating_sub(1));
    let output = request.output.clone();
    let error = match start_detailed_collection(operation.context(), request) {
        Ok(_) => panic!("an expired delegated interval must not restart its duration"),
        Err(error) => error,
    };
    assert!(matches!(error, CaptureError::Invalid(ref reason) if reason.contains("expired")));
    assert!(!output.exists(), "expiry refusal precedes output effects");
    assert!(operation.context().is_active());
}

#[test]
fn record_pages_are_exact_and_bad_cursors_and_partial_tails_are_explicit() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("science");
    let session = start_detailed_collection(
        operation.context(),
        selection(directory.path().join("pages")),
    )
    .unwrap();
    let producer = session.context().producer(ProducerRole::NativeIteration, "native");
    for value in 1..=3 { producer.publish(&iteration(&producer, value)); }
    let document = session.finish().unwrap();
    assert_eq!(document.outcome, DetailOutcome::Complete);
    let first = read_detail_records(&document.records, 0, 2, 1 << 20).unwrap();
    assert_eq!(iteration_values(&first), [1, 2]);
    assert!(!first.partial);
    let cursor = first.next_offset.expect("first page is bounded at its second record");
    let second = read_detail_records(&document.records, cursor, 2, 1 << 20).unwrap();
    assert_eq!(iteration_values(&second), [3]);
    assert!(!second.partial);
    assert!(second.next_offset.is_none());
    assert!(read_detail_records(&document.records, cursor + 1, 2, 1 << 20).is_err());
    assert!(read_detail_records(&document.records, 1, 2, 1 << 20).is_err());

    // Retain exact complete frames, remove only the terminal receipt, and append
    // an interrupted frame. Readers may return that prefix but never completion.
    let retained = std::fs::read(&document.records).unwrap();
    let mut prefix = Vec::new();
    for line in retained.split_inclusive(|byte| *byte == b'\n') {
        if matches!(serde_json::from_slice::<Frame>(line).unwrap(), Frame::End(_)) { break; }
        prefix.extend_from_slice(line);
    }
    let complete_extent = prefix.len() as u64;
    prefix.extend_from_slice(b"{\"kind\":\"record\"");
    let partial_path = directory.path().join("partial.jsonl");
    std::fs::write(&partial_path, prefix).unwrap();
    let partial_page = read_detail_records(&partial_path, 0, 8, 1 << 20).unwrap();
    assert_eq!(iteration_values(&partial_page), [1, 2, 3]);
    assert!(partial_page.partial);
    assert!(partial_page.next_offset.is_none());
    let partial_document = read_detailed_collection(&partial_path, 1 << 20).unwrap();
    assert_eq!(partial_document.outcome, DetailOutcome::Incomplete);
    assert!(partial_document.end.is_none());
    assert_eq!(partial_document.readable_prefix_bytes, complete_extent);
}

#[test]
fn a_lost_final_issued_event_prevents_complete_even_without_a_later_gap() {
    let directory = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("science");
    let session = start_detailed_collection(
        operation.context(),
        selection(directory.path().join("lost-final")),
    )
    .unwrap();
    let producer = session.context().producer(ProducerRole::NativeIteration, "native");
    producer.publish(&iteration(&producer, 1));
    let unpublished_final = iteration(&producer, 2);
    assert_eq!(unpublished_final.family_sequence, Some(2));
    let document = session.finish().unwrap();
    assert_eq!(document.outcome, DetailOutcome::Incomplete);
    assert_eq!(document.end, Some(DetailEnd::OwnerFinished));
    assert!(document.source_partial, "issued final head differs from retained observation head");
    assert_eq!(document.families[0].acknowledged, 1);
    let page = read_detail_records(&document.records, 0, 8, 1 << 20).unwrap();
    assert_eq!(iteration_values(&page), [1]);
    let readback = read_detailed_collection(&document.records, 1 << 20).unwrap();
    assert_eq!(readback.outcome, DetailOutcome::Incomplete);
    assert!(readback.source_partial);
}

#[test]
fn selected_mapping_refusal_and_tiny_header_allowance_stay_explicit() {
    let directory=tempfile::tempdir().unwrap();
    let hub=ObservationHub::new(Policy::default());
    let operation=hub.start("science");
    let session=start_detailed_collection(operation.context(),selection(directory.path().join("mapping"))).unwrap();
    let producer=session.context().producer(ProducerRole::NativeIteration,"native");
    producer.publish(&iteration(&producer,1));
    session.context().record_source_unavailable("iteration.v1");
    let document=session.finish().unwrap();
    assert_eq!(document.outcome,DetailOutcome::Incomplete);
    assert!(document.source_partial);
    let header_only=directory.path().join("header.jsonl");
    let mut header=serde_json::to_vec(&Frame::Header(document.request)).unwrap();
    header.push(b'\n');std::fs::write(&header_only,header).unwrap();
    assert!(read_detail_records(&header_only,0,1,1).is_err());
    assert!(read_detail_records(&header_only,0,1,1<<20).unwrap().records.is_empty());
}
