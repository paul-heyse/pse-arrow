// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::observation::Diagnostics;
use prost::Message;
use pse_diagnostics::observation::{Outcome, Policy, Relationship};
use std::{io::Write, time::Instant};
use tracing_perfetto_sdk_schema::{Trace, TracePacket, TrackEvent, trace_packet, track_event};

fn request(
    directory: &std::path::Path,
    diagnostics: &Diagnostics,
    operation: OperationId,
) -> CaptureRequest {
    CaptureRequest {
        version: Version,
        checkpoint: diagnostics.checkpoint_path(),
        operation,
        kind: CaptureKind::Timeline,
        output: directory.join("capture"),
        duration: Duration::from_millis(100),
        budget: CaptureBudget::default(),
    }
}
fn trace(document: &CaptureDocument) -> Trace {
    let output = document
        .outputs
        .iter()
        .find(|output| output.kind == CaptureOutputKind::Perfetto)
        .unwrap();
    let mut bytes = std::fs::read(&output.path).unwrap();
    bytes.truncate(output.readable_prefix_bytes as usize);
    Trace::decode(bytes.as_slice()).unwrap()
}
fn events(trace: &Trace) -> Vec<(&TracePacket, &TrackEvent)> {
    trace
        .packet
        .iter()
        .filter_map(|packet| match &packet.data {
            Some(trace_packet::Data::TrackEvent(event)) => Some((packet, event)),
            _ => None,
        })
        .collect()
}
fn state_for(request: &CaptureRequest) -> Shared {
    Arc::new(Mutex::new(CaptureDocument {
        version: Version,
        kind: request.kind,
        operation: Some(request.operation),
        related_operations: vec![request.operation],
        launch_process: None,
        status: CaptureStatus::Running,
        requested_duration: request.duration,
        observed_duration: Duration::ZERO,
        targets: Vec::new(),
        outputs: Vec::new(),
        loss: CaptureLoss::default(),
        symbols: CaptureSymbols::default(),
        open_intervals: 0,
        errors: Vec::new(),
    }))
}
#[test]
fn capture_open_interval_and_stop_are_independent_of_science() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let operation = owner.hub().start("occupied scientific owner");
    let phase = operation.context().phase("native call still running");
    owner.publish().unwrap();
    let mut request = request(directory.path(), &owner, operation.context().id());
    request.duration = Duration::from_secs(10);
    let session = start_capture(request).unwrap();
    std::thread::sleep(Duration::from_millis(75));
    session.request_stop();
    let captured = session.wait().unwrap();
    assert_eq!(captured.status, CaptureStatus::Stopped);
    assert!(captured.open_intervals >= 2);
    let trace = trace(&captured);
    assert!(
        events(&trace)
            .iter()
            .any(|(_, event)| event.r#type == Some(track_event::Type::SliceBegin as i32))
    );
    assert!(
        !events(&trace)
            .iter()
            .any(|(_, event)| event.r#type == Some(track_event::Type::SliceEnd as i32))
    );
    assert!(operation.context().is_active());
    drop(phase);
    operation.finish(Outcome::Completed);
}
#[test]
fn capture_actual_phase_end_and_shared_two_waiter_flows() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("study");
    let first = root.context().child("first waiter", Relationship::Nested);
    let second = root.context().child("second waiter", Relationship::Nested);
    let producer = first
        .context()
        .child("shared producer", Relationship::SharedProduction);
    second
        .context()
        .link(&producer.context(), Relationship::SharedProduction);
    {
        let _phase = producer.context().phase("factorization");
        std::thread::sleep(Duration::from_millis(1));
    }
    owner.publish().unwrap();
    let captured = start_capture(request(directory.path(), &owner, root.context().id()))
        .unwrap()
        .wait()
        .unwrap();
    let trace = trace(&captured);
    let events = events(&trace);
    let flows: Vec<u64> = events
        .iter()
        .flat_map(|(_, event)| event.flow_ids.iter().copied())
        .collect();
    assert_eq!(
        flows.len(),
        4,
        "two nested links plus two distinct shared-producer waiter links"
    );
    assert_eq!(
        flows.iter().collect::<std::collections::HashSet<_>>().len(),
        4
    );
    for flow in flows {
        assert!(
            events
                .iter()
                .any(|(_, event)| event.terminating_flow_ids.contains(&flow))
        );
    }
    assert!(
        events
            .iter()
            .any(|(_, event)| event.r#type == Some(track_event::Type::SliceEnd as i32)),
        "actual phase completion is exported"
    );
}
#[test]
fn capture_unknown_clock_keeps_unaligned_target_explicit() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("unknown alignment");
    owner.publish().unwrap();
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    image.clock = None;
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    let captured = start_capture(request(directory.path(), &owner, root.context().id()))
        .unwrap()
        .wait()
        .unwrap();
    assert!(captured.loss.unknown_clock);
    assert!(!captured.targets[0].clock_aligned);
    assert!(
        events(&trace(&captured))
            .iter()
            .all(|(_, event)| matches!(&event.name_field,
            Some(track_event::NameField::Name(name)) if name == "clock alignment")),
        "unknown source alignment must not invent timestamps for scientific owners"
    );
}
#[test]
fn capture_output_exhaustion_retains_decodable_prefix() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner
        .hub()
        .start("long recognizable operation name for output exhaustion");
    let _phase = root
        .context()
        .phase("an additional phase whose packet cannot fit");
    owner.publish().unwrap();
    let mut request = request(directory.path(), &owner, root.context().id());
    request.budget.output_bytes = 160;
    let captured = start_capture(request).unwrap().wait().unwrap();
    assert_eq!(captured.status, CaptureStatus::Partial);
    assert!(captured.loss.output_exhausted);
    assert!(captured.outputs[0].bytes <= 160);
    let _decoded = trace(&captured);
    assert!(root.context().is_active());
}
#[test]
fn capture_existing_output_is_refused_without_replacing_it() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("scientific work");
    owner.publish().unwrap();
    let request = request(directory.path(), &owner, root.context().id());
    std::fs::create_dir(&request.output).unwrap();
    std::fs::write(request.output.join("preserve"), "already owned").unwrap();
    assert!(matches!(start_capture(request), Err(CaptureError::Io(_))));
    assert_eq!(
        std::fs::read_to_string(directory.path().join("capture/preserve")).unwrap(),
        "already owned"
    );
}
#[test]
fn capture_terminal_cpu_is_refused_before_effects_but_timeline_retains_history() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("completed scientific work");
    let id = root.context().id();
    root.finish(Outcome::Completed);
    owner.publish().unwrap();
    let mut cpu = request(directory.path(), &owner, id);
    cpu.kind = CaptureKind::Cpu;
    let output = cpu.output.clone();
    assert!(
        matches!(start_capture(cpu), Err(CaptureError::Invalid(message)) if message.contains("active operation"))
    );
    assert!(
        !output.exists(),
        "refusal must not create files or start sampler tools"
    );
    let timeline = start_capture(request(directory.path(), &owner, id))
        .unwrap()
        .wait()
        .unwrap();
    assert!(timeline.outputs.iter().any(|output| output.readable));
    assert!(
        events(&trace(&timeline))
            .iter()
            .any(|(_, event)| event.r#type == Some(track_event::Type::SliceEnd as i32))
    );
}
struct FailingWriter {
    bytes: Vec<u8>,
    allowance: usize,
}
impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.allowance == 0 {
            return Err(std::io::Error::other("controlled writer failure"));
        }
        let count = self.allowance.min(bytes.len());
        self.bytes.extend_from_slice(&bytes[..count]);
        self.allowance -= count;
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[test]
fn capture_writer_failure_preserves_previous_complete_packet_offset() {
    let packet = TracePacket {
        timestamp: Some(500),
        ..Default::default()
    };
    let first = Trace {
        packet: vec![packet.clone()],
    }
    .encode_to_vec();
    let writer = FailingWriter {
        bytes: Vec::new(),
        allowance: first.len() + 2,
    };
    let mut output = timeline::PacketWriter::new(writer, 1024, 1024);
    output.append(packet.clone()).unwrap();
    output.flush().unwrap();
    let readable = output.flushed;
    assert!(output.append(packet).is_err());
    assert!(output.bytes > readable);
    output.flush().unwrap();
    assert_eq!(
        output.flushed, readable,
        "an incomplete packet tail never advances the prefix"
    );
}
#[test]
fn capture_publishes_open_prefix_before_waiting_for_operation_end() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("long active operation");
    owner.publish().unwrap();
    let mut request = request(directory.path(), &owner, root.context().id());
    request.duration = Duration::from_secs(10);
    let session = start_capture(request).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let snapshot = session.snapshot();
        if snapshot
            .outputs
            .first()
            .is_some_and(|output| output.readable_prefix_bytes > 0)
        {
            assert_eq!(snapshot.status, CaptureStatus::Running);
            assert_eq!(snapshot.open_intervals, 1);
            assert!(
                events(&trace(&snapshot))
                    .iter()
                    .any(|(_, event)| event.r#type == Some(track_event::Type::SliceBegin as i32))
            );
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(session);
    assert!(root.context().is_active());
}
#[test]
fn capture_caller_layer_exports_lifecycle_without_global_subscriber() {
    use tracing_subscriber::prelude::*;
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("caller composed trace");
    owner.publish().unwrap();
    let session = start_capture(request(directory.path(), &owner, root.context().id())).unwrap();
    let subscriber = tracing_subscriber::registry().with(session.layer().unwrap());
    tracing::subscriber::with_default(subscriber, || {
        let _span = tracing::info_span!("recognizable caller Rust lifecycle");
        tracing::info!("caller event");
    });
    let captured = session.wait().unwrap();
    let trace = trace(&captured);
    let events = events(&trace);
    assert!(events.iter().any(|(_, event)| matches!(&event.name_field, Some(track_event::NameField::Name(name)) if name == "recognizable caller Rust lifecycle")));
    assert!(
        events
            .iter()
            .any(|(_, event)| event.r#type == Some(track_event::Type::SliceEnd as i32))
    );
    assert!(root.context().is_active());
}
#[test]
fn capture_version_and_configuration_defaults_are_explicit() {
    let value = serde_json::json!({"version":1,"checkpoint":"root.json","operation":{"process_instance":[1,2],"sequence":3},"kind":"Timeline","output":"new-directory"});
    let request: CaptureRequest = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(request.duration, Duration::from_secs(10));
    assert_eq!(request.budget.memory_bytes, 16 << 20);
    assert_eq!(request.budget.output_bytes, 256 << 20);
    let mut wrong = value;
    wrong["version"] = 2.into();
    assert!(serde_json::from_value::<CaptureRequest>(wrong).is_err());
}
#[test]
fn capture_layer_queue_refusal_is_bounded_and_does_not_touch_science() {
    use tracing_subscriber::prelude::*;
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("independent scientific owner");
    let request = request(directory.path(), &owner, root.context().id());
    let stop = Arc::new(AtomicBool::new(false));
    let (layer, packets) = layer::channel(&request, &stop);
    let subscriber = tracing_subscriber::registry().with(layer);
    tracing::subscriber::with_default(subscriber, || {
        for _ in 0..10_000 {
            tracing::info!("bounded lifecycle queue control");
        }
    });
    assert!(packets.dropped.load(Ordering::Relaxed) > 0);
    assert!(packets.receiver.try_iter().count() <= 1024);
    assert!(root.context().is_active());
}
#[test]
fn capture_cpu_all_96_owned_files_are_reserved_and_inventoried() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("inventory control");
    let request = request(directory.path(), &owner, root.context().id());
    let state = state_for(&request);
    let outputs = directory.path().join("files");
    std::fs::create_dir(&outputs).unwrap();
    let limit = 96 * 4;
    let mut files = cpu::OwnedFiles::new(limit);
    for index in 0..48 {
        let first = outputs.join(format!("{index}-main"));
        let second = outputs.join(format!("{index}-log"));
        files.reserve_pair(&first, &second, 4).unwrap();
        std::fs::write(&first, [1; 4]).unwrap();
        std::fs::write(&second, [2; 4]).unwrap();
        cpu::output(&state, &first, CaptureOutputKind::PerfData, false);
        cpu::output(&state, &second, CaptureOutputKind::ToolLog, true);
        files.seal(&first);
        files.seal(&second);
    }
    assert_eq!(files.len(), 96);
    assert_eq!(files.available(), 0);
    assert!(
        files
            .reserve_pair(&outputs.join("excess-main"), &outputs.join("excess-log"), 1)
            .is_err()
    );
    let actual: std::collections::HashSet<PathBuf> = std::fs::read_dir(&outputs)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let inventory: std::collections::HashSet<PathBuf> = document(&state)
        .outputs
        .iter()
        .map(|output| output.path.clone())
        .collect();
    assert_eq!(actual, inventory);
    let directory_bytes: u64 = actual
        .iter()
        .map(|path| std::fs::metadata(path).unwrap().len())
        .sum();
    assert_eq!(directory_bytes, limit);
    assert_eq!(files.bytes(), directory_bytes);
}
#[test]
fn capture_cpu_skipped_decoder_never_marks_raw_readable() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("decoder refusal");
    let request = request(directory.path(), &owner, root.context().id());
    let state = state_for(&request);
    let raw = directory.path().join("retained.perf.data");
    std::fs::write(&raw, [1]).unwrap();
    cpu::output(&state, &raw, CaptureOutputKind::PerfData, false);
    let mut files = cpu::OwnedFiles::new(4096);
    files
        .reserve_pair(&raw, &directory.path().join("log"), 2048)
        .unwrap();
    assert_eq!(
        cpu::symbols(&raw, &state, Duration::from_secs(1), &mut files).unwrap(),
        cpu::Decode::Skipped
    );
    assert!(document(&state).loss.output_exhausted);
    assert!(!document(&state).symbols.checked);
    assert!(!document(&state).outputs[0].readable);
}
#[test]
fn capture_cpu_failed_tool_retains_actual_partial_inventory_and_log() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("failed exporter inventory");
    let request = request(directory.path(), &owner, root.context().id());
    let state = state_for(&request);
    let log = directory.path().join("tool.log");
    std::fs::write(&log, "controlled timeout detail").unwrap();
    for kind in [CaptureOutputKind::Symbols, CaptureOutputKind::Samply] {
        let path = directory.path().join(format!("{kind:?}.partial"));
        cpu::output(&state, &path, kind, false);
        std::fs::write(&path, [1; 3183]).unwrap();
        assert!(
            cpu::finish_tool_result(
                &state,
                &path,
                kind,
                &log,
                Err(CaptureError::Invalid("controlled tool timeout".into()))
            )
            .is_err()
        );
        let snapshot = document(&state);
        let partial = snapshot
            .outputs
            .iter()
            .find(|output| output.path == path)
            .unwrap();
        assert_eq!(partial.bytes, 3183);
        assert_eq!(partial.readable_prefix_bytes, 0);
        assert!(!partial.readable);
        let log = snapshot
            .outputs
            .iter()
            .find(|output| output.kind == CaptureOutputKind::ToolLog)
            .unwrap();
        assert!(log.readable && log.bytes > 0);
        assert!(
            snapshot
                .errors
                .iter()
                .any(|error| error.contains("controlled timeout detail"))
        );
    }
}
#[test]
fn capture_cpu_scope_loss_preserves_unavailable_registry_and_selected_updates() {
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("CPU scope coverage");
    owner.publish().unwrap();
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    let selected = [root.context().id()];
    image.registry_coverage.sampled = true;
    image.operations[0].coverage.sampled = true;
    assert!(
        !source_partial(&image, &selected),
        "point coalescing does not lose CPU attachment scope"
    );
    image.registry_coverage.curtailed = true;
    assert!(source_partial(&image, &selected));
    image.registry_coverage.curtailed = false;
    image.registry_coverage.available = false;
    assert!(source_partial(&image, &selected));
    image.registry_coverage.available = true;
    image.operations[0].coverage.curtailed = true;
    assert!(source_partial(&image, &selected));
    image.operations[0].coverage.curtailed = false;
    image.operations[0].coverage.available = false;
    assert!(source_partial(&image, &selected));
    image.operations[0].coverage.available = true;
    image.operations[0].coverage.dropped_updates = 1;
    assert!(source_partial(&image, &selected));
}
#[test]
fn capture_timeline_preserves_curtailed_registry_and_unavailable_operation_scope() {
    for output_bytes in [256 << 20, 1] {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let root = owner.hub().start("partial timeline scope");
        owner.publish().unwrap();
        let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
        image.registry_coverage.curtailed = true;
        image.operations[0].coverage.available = false;
        std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
        let mut request = request(directory.path(), &owner, root.context().id());
        request.budget.output_bytes = output_bytes;
        let captured = start_capture(request).unwrap().wait().unwrap();
        assert_eq!(captured.status, CaptureStatus::Partial);
        assert!(captured.loss.source_partial);
        assert_eq!(captured.loss.output_exhausted, output_bytes == 1);
        assert!(captured.errors.is_empty());
        if output_bytes == 1 {
            assert!(captured.outputs.iter().all(|output| !output.readable
                && output.bytes == 0
                && output.readable_prefix_bytes == 0));
        } else {
            assert!(captured.outputs.iter().any(|output| output.readable));
        }
        assert!(root.context().is_active());
    }
}
#[cfg(unix)]
#[test]
fn capture_cpu_exit_classification_requires_actual_owned_sigint() {
    use std::os::unix::process::ExitStatusExt;
    let success = std::process::ExitStatus::from_raw(0);
    let interrupted = std::process::ExitStatus::from_raw(2);
    let killed = std::process::ExitStatus::from_raw(9);
    let exit_error = std::process::ExitStatus::from_raw(2 << 8);
    assert!(cpu::expected_finish(success, false));
    assert!(cpu::expected_finish(interrupted, true));
    assert!(!cpu::expected_finish(interrupted, false));
    assert!(!cpu::expected_finish(killed, true));
    assert!(!cpu::expected_finish(exit_error, true));
}
#[cfg(unix)]
#[test]
fn capture_cpu_exit_log_preserves_actual_status_within_original_reservation() {
    use std::os::unix::process::ExitStatusExt;
    let directory = tempfile::tempdir().unwrap();
    let log = directory.path().join("perf.log");
    let prefix = b"recording tool output\n";
    std::fs::write(&log, prefix).unwrap();
    let status = std::process::ExitStatus::from_raw(2);
    assert!(cpu::append_finish(&log, 4096, status, true).unwrap());
    let receipt = std::fs::read_to_string(&log).unwrap();
    assert!(receipt.contains("code=None signal=Some(2) owner_sigint_sent=true expected=true"));
    let complete = receipt.len() as u64;
    std::fs::write(&log, prefix).unwrap();
    assert!(!cpu::append_finish(&log, complete - 1, status, true).unwrap());
    assert_eq!(std::fs::read(&log).unwrap(), prefix);
}
#[test]
fn capture_cpu_graph_and_process_bounds_report_partiality() {
    use pse_diagnostics::observation::ChildSnapshot;
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("bounded graph");
    owner.publish().unwrap();
    let request = request(directory.path(), &owner, root.context().id());
    let state = state_for(&request);
    let mut image = read_checkpoint(&request.checkpoint).unwrap();
    image.operations[0].children = vec![ChildSnapshot {
        id: OperationId {
            process_instance: [999, 888],
            sequence: 666,
        },
        relationship: Relationship::Nested,
        observed_at: Duration::from_millis(1),
        kind: "known remote beyond allowance".into(),
        outcome: None,
        started_at: Duration::ZERO,
        finished_at: None,
        available: false,
    }];
    let mut selected = vec![root.context().id()];
    selected.extend((0..511).map(|sequence| OperationId {
        process_instance: [1, 1],
        sequence,
    }));
    let mut paths: Vec<PathBuf> = (0..16)
        .map(|index| directory.path().join(format!("existing-{index}")))
        .collect();
    cpu::discover(&request, &image, &mut selected, &mut paths, &state);
    assert_eq!(selected.len(), 512);
    assert_eq!(paths.len(), 16);
    assert!(document(&state).loss.refused_items >= 2);
    assert!(document(&state).loss.source_partial);
}
#[test]
fn capture_numerical_history_loss_is_partial_without_metric_or_registry_loss() {
    use pse_diagnostics::observation::ProducerRole;
    for refused in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(
            Policy {
                record_capacity: 0,
                ..Policy::default()
            },
            directory.path().to_owned(),
        );
        let root = owner.hub().start("numerical history only");
        let producer = root
            .context()
            .producer(ProducerRole::NativeIteration, "numerical owner");
        for _ in 0..2 {
            let record = producer.record(
                "history_control.v1",
                "native progress",
                None,
                None,
                Vec::new(),
            );
            assert!(producer.publish(&record));
        }
        owner.publish().unwrap();
        let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
        let selected = [root.context().id()];
        assert!(
            !source_partial(&image, &selected),
            "CPU attachment coverage is independent from numerical history"
        );
        assert!(image.operations[0].metrics.is_empty());
        let coverage = &mut image.operations[0].producers[0].coverage;
        assert_eq!(coverage.history_dropped, 2);
        if refused {
            coverage.history_dropped = 0;
            coverage.refused = 1;
            coverage.partial = false;
        }
        std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
        let captured = start_capture(request(directory.path(), &owner, root.context().id()))
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(captured.status, CaptureStatus::Partial);
        assert!(captured.loss.source_partial);
        assert!(!captured.loss.output_exhausted);
        assert!(captured.errors.is_empty());
        assert!(root.context().is_active());
    }
}
#[test]
fn capture_metric_coalescing_and_refusal_survive_timeline_projection() {
    use tracing_perfetto_sdk_schema::debug_annotation;
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = owner.hub().start("dense same-second metric");
    for value in 0..1000 {
        root.context().count_value("evaluations", "calls", value, 0);
    }
    owner.publish().unwrap();
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    let metric = &mut image.operations[0].metrics[0];
    assert!(metric.history.iter().any(|point| point.samples > 2));
    metric.history_partial = true;
    metric.history_dropped = 7;
    image.operations[0].coverage.sampled = true;
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    let captured = start_capture(request(directory.path(), &owner, root.context().id()))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(captured.status, CaptureStatus::Partial);
    assert!(captured.loss.source_partial);
    let trace = trace(&captured);
    let event = events(&trace).into_iter().find(|(_, event)| matches!(&event.name_field, Some(track_event::NameField::Name(name)) if name == "metric history coverage")).unwrap().1;
    for (name, value) in [
        ("history_partial", debug_annotation::Value::BoolValue(true)),
        ("history_dropped", debug_annotation::Value::UintValue(7)),
        ("source_sampled", debug_annotation::Value::BoolValue(true)),
    ] {
        assert!(event.debug_annotations.iter().any(|annotation|
            matches!(&annotation.name_field, Some(debug_annotation::NameField::Name(found)) if found == name)
                && annotation.value.as_ref() == Some(&value)));
    }
}
#[test]
fn capture_remote_checkpoint_link_keeps_parent_and_child_clock_mapping() {
    let directory = tempfile::tempdir().unwrap();
    let parent = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let child = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let root = parent.hub().start("remote parent");
    let remote = child.hub().start("remote native child");
    root.context()
        .link_id(remote.context().id(), Relationship::Continuation);
    parent.publish().unwrap();
    child.publish().unwrap();
    let root_image = read_checkpoint(&parent.checkpoint_path()).unwrap();
    let child_image = read_checkpoint(&child.checkpoint_path()).unwrap();
    let parent_time = root_image
        .clock
        .as_ref()
        .unwrap()
        .monotonic_timestamp(root_image.operations[0].children[0].observed_at)
        .unwrap();
    let child_time = child_image
        .clock
        .as_ref()
        .unwrap()
        .monotonic_timestamp(child_image.operations[0].started_at)
        .unwrap();
    let captured = start_capture(request(directory.path(), &parent, root.context().id()))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(captured.targets.len(), 2);
    assert!(captured.targets.iter().all(|target| target.clock_aligned));
    let trace = trace(&captured);
    let events = events(&trace);
    let flow = events
        .iter()
        .find(|(_, event)| !event.flow_ids.is_empty())
        .unwrap();
    assert_eq!(flow.0.timestamp, Some(parent_time));
    assert!(events.iter().any(|(packet, event)| packet.timestamp == Some(child_time)
        && matches!(&event.name_field, Some(track_event::NameField::Name(name)) if name == "remote native child")));
    assert!(root.context().is_active() && remote.context().is_active());
}

/// Explicit native profile journey: root selects its serialized native environment and output.
/// A useful-frame claim still requires examining both recognizable Rust and C++ frame names.
#[cfg(feature = "solver-highs")]
#[test]
#[ignore = "explicit owned perf attachment and linked HiGHS profile control"]
fn capture_owned_rust_and_linked_native_profile_control() {
    use faer::sparse::{SparseColMat, Triplet};
    use pse_backend_native::{
        CoefficientProblem, OracleContract, Variable, highs,
        quality::Tolerances,
        solve::{Backend, Compatibility, Controls, Execution, ResolvedAccuracy, Termination},
    };
    use pse_ids::{ContentHash, SemanticId};
    use pse_math::{binding::ObjectiveSense, normalization::Normalization};
    use pse_model::generated::enums::ModelingVariableDomain;
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let operation = owner
        .hub()
        .start("owned Rust and linked HiGHS CPU profile control");
    let _entered = operation.context().enter();
    let _thread = owner.execution(Some(operation.context().id()));
    owner.publish().unwrap();
    let mut request = request(directory.path(), &owner, operation.context().id());
    request.kind = CaptureKind::Cpu;
    request.duration = Duration::from_secs(6);
    request.output = std::env::var_os("PSE_CAPTURE_CONTROL_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| directory.path().join("owned-native-profile"));
    let output_directory = request.output.clone();
    let capture = start_capture(request).unwrap();
    let count = 128;
    let id = |index: usize| SemanticId::from_bytes((index as u128 + 1).to_le_bytes());
    let compatibility = Compatibility {
        layout: ContentHash::from_bytes([21; 32]),
        profile: ContentHash::from_bytes([22; 32]),
        data: ContentHash::from_bytes([23; 32]),
        backend: Backend::Highs,
    };
    let problem = CoefficientProblem {
        contract: OracleContract {
            identity: ContentHash::from_bytes([24; 32]),
            variables: (0..count)
                .map(|index| Variable {
                    id: id(index),
                    lower: 0.0,
                    upper: 10.0,
                })
                .collect(),
            rows: (count..2 * count).map(id).collect(),
            derivatives: pse_kernels::DerivativeOrder::Second,
            smoothness: pse_kernels::DerivativeOrder::Second,
        },
        objective: vec![1.0; count],
        objective_constant: 0.0,
        sense: ObjectiveSense::Minimize,
        domains: vec![ModelingVariableDomain::Continuous; count],
        assumptions: compatibility.data,
        constraints: SparseColMat::try_new_from_triplets(
            count,
            count,
            &(0..count)
                .map(|index| Triplet::new(index, index, 1.0))
                .collect::<Vec<_>>(),
        )
        .unwrap(),
        hessian: None,
        bounds: vec![(1.0, f64::INFINITY); count],
        objectives: Vec::new(),
    };
    let mut controls = Controls::default();
    controls.options.insert(
        "presolve".into(),
        pse_backend_native::solve::OptionValue::Text("off".into()),
    );
    let normalization = Normalization::identity(count, count);
    let tolerances = Tolerances {
        variables: vec![1e-8; count],
        rows: vec![1e-8; count],
        integrality: 1e-8,
    };
    let policy = pse_model::numerics::NumericalPolicy {
        kkt: pse_model::numerics::KktTolerances {
            stationarity: 1e-8,
            complementarity: 1e-8,
        },
        gap_absolute: 1e-8,
        gap_relative: 1e-8,
        ..Default::default()
    };
    let accuracy = ResolvedAccuracy::from_policy(&policy, 1e-8).unwrap();
    // Wait for recording output before creating a new scientific worker thread.
    // This control specifically challenges inheritance after initial attachment.
    let deadline = Instant::now() + Duration::from_secs(3);
    while !capture.snapshot().outputs.iter().any(|output| {
        output.kind == CaptureOutputKind::PerfData
            && std::fs::metadata(&output.path).is_ok_and(|metadata| metadata.len() > 0)
    }) {
        assert!(
            !capture.is_finished(),
            "CPU recording ended before late-thread control: {:?}",
            capture.snapshot().errors
        );
        assert!(
            Instant::now() < deadline,
            "CPU recording did not become active"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let context = operation.context();
    let solves = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _entered = context.enter();
                let _thread = owner.execution(Some(context.id()));
                owner.publish().unwrap();
                let started = Instant::now();
                let mut solves = 0;
                while started.elapsed() < Duration::from_secs(7) {
                    recognizable_rust_capture_control(solves);
                    let execution =
                        Execution::new(pse_diagnostics::Cancellation::new(false), &controls);
                    let mut session =
                        highs::Session::new(&problem, None, compatibility.clone(), &execution)
                            .unwrap();
                    let report = session
                        .solve(
                            &problem,
                            &normalization,
                            &controls,
                            &accuracy,
                            &highs::Settings::default(),
                            execution,
                            &tolerances,
                            None,
                        )
                        .unwrap();
                    assert_eq!(report.termination.category, Termination::Success);
                    solves += 1;
                }
                solves
            })
            .join()
            .unwrap()
    });
    let captured = capture.wait().unwrap();
    std::fs::write(
        output_directory.join("capture-document.json"),
        serde_json::to_vec_pretty(&captured).unwrap(),
    )
    .unwrap();
    assert!(solves > 0);
    assert!(
        operation.context().is_active(),
        "capture never finishes scientific work"
    );
    assert!(
        captured.symbols.checked && captured.symbols.resolved_frames > 0,
        "{:?}",
        captured.errors
    );
    assert!(
        captured
            .outputs
            .iter()
            .any(|output| output.kind == CaptureOutputKind::Samply && output.readable)
    );
    assert!(
        captured
            .targets
            .iter()
            .any(|target| target.thread_ids.len() >= 2),
        "late native-thread attribution missing"
    );
    let frames: String = captured
        .outputs
        .iter()
        .filter(|output| output.kind == CaptureOutputKind::Symbols)
        .map(|output| std::fs::read_to_string(&output.path).unwrap())
        .collect();
    assert!(
        frames.contains("recognizable_rust_capture_control"),
        "owned Rust control frame missing"
    );
    assert!(
        frames.contains("Highs_run") || frames.contains("Highs::") || frames.contains("HSimplex::"),
        "actual native HiGHS C/C++ control frame missing"
    );
}
#[cfg(feature = "solver-highs")]
#[inline(never)]
fn recognizable_rust_capture_control(seed: u64) {
    let mut value = seed;
    for index in 0..200_000 {
        value = std::hint::black_box(value.wrapping_mul(6364136223846793005).wrapping_add(index));
    }
    std::hint::black_box(value);
}

#[cfg(target_os = "linux")]
#[test]
fn capture_builtin_clock_relationship_precedes_typed_source_events() {
    use pse_diagnostics::observation::{ProducerRole, RecordAvailability, RecordField, Value};
    use tracing_perfetto_sdk_schema::debug_annotation;
    let nanos = |clock| {
        let value = rustix::time::clock_gettime(clock);
        u64::try_from(value.tv_sec).unwrap() * 1_000_000_000 + u64::try_from(value.tv_nsec).unwrap()
    };
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let operation = owner.hub().start("actual source clock control");
    let producer = operation
        .context()
        .producer(ProducerRole::NativeIteration, "actual numerical owner");
    let field = |name: &str, value| RecordField {
        name: name.into(),
        unit: "source-unit".into(),
        value,
        availability: None,
    };
    let record = producer.record(
        "clock_control.v1",
        "native checkpoint",
        Some("physical-original"),
        Some("original-infinity"),
        vec![
            field("iteration", Value::Count(u64::MAX)),
            field("signed", Value::Integer(i64::MIN)),
            field("zero", Value::Number(0.)),
            field("predicate", Value::Boolean(false)),
            field("bad", Value::number(f64::NAN)),
            RecordField {
                availability: Some(RecordAvailability::NotSupplied),
                ..field(
                    "absent",
                    Value::Unavailable("not supplied by native owner".into()),
                )
            },
        ],
    );
    assert!(producer.publish(&record));
    owner.publish().unwrap();
    let before_mono = nanos(rustix::time::ClockId::Monotonic);
    let before_boot = nanos(rustix::time::ClockId::Boottime);
    let captured = start_capture(request(directory.path(), &owner, operation.context().id()))
        .unwrap()
        .wait()
        .unwrap();
    let after_mono = nanos(rustix::time::ClockId::Monotonic);
    let after_boot = nanos(rustix::time::ClockId::Boottime);
    let trace = trace(&captured);
    let first_event = trace
        .packet
        .iter()
        .position(|packet| matches!(&packet.data, Some(trace_packet::Data::TrackEvent(_))))
        .unwrap();
    let snapshot_index = trace
        .packet
        .iter()
        .position(|packet| matches!(&packet.data, Some(trace_packet::Data::ClockSnapshot(_))))
        .unwrap();
    assert!(
        snapshot_index < first_event,
        "a standalone consumer sees clock relationships before timestamps"
    );
    let Some(trace_packet::Data::ClockSnapshot(snapshot)) = &trace.packet[snapshot_index].data
    else {
        panic!("the located clock packet must contain a snapshot")
    };
    assert_eq!(snapshot.primary_trace_clock, Some(6));
    let clock = |id| {
        snapshot
            .clocks
            .iter()
            .find(|clock| clock.clock_id == Some(id))
            .unwrap()
            .timestamp
            .unwrap()
    };
    assert!(
        (before_mono..=after_mono).contains(&clock(3)),
        "MONOTONIC snapshot is an actual sampled reading"
    );
    assert!(
        (before_boot..=after_boot).contains(&clock(6)),
        "BOOTTIME snapshot is an actual sampled reading"
    );
    let source = events(&trace)
        .into_iter()
        .find(|(_, event)| {
            matches!(&event.name_field,
        Some(track_event::NameField::Name(name)) if name == "native checkpoint")
        })
        .unwrap();
    assert_eq!(source.0.timestamp_clock_id, Some(3));
    let annotations = &source.1.debug_annotations;
    let annotation = |name: &str| {
        annotations
            .iter()
            .find(|annotation| {
                matches!(&annotation.name_field,
        Some(debug_annotation::NameField::Name(found)) if found == name)
            })
            .unwrap()
            .value
            .clone()
            .unwrap()
    };
    assert_eq!(
        annotation("operation_id"),
        debug_annotation::Value::StringValue(record.producer.operation.to_string())
    );
    for (name, expected) in [
        ("producer_sequence", record.producer.sequence),
        ("source_sequence", record.sequence),
        ("family_sequence", record.family_sequence.unwrap()),
        ("generation", record.generation),
        ("iteration", u64::MAX),
    ] {
        assert_eq!(
            annotation(name),
            debug_annotation::Value::UintValue(expected)
        );
    }
    assert_eq!(
        annotation("family"),
        debug_annotation::Value::StringValue("clock_control.v1".into())
    );
    assert_eq!(
        annotation("signed"),
        debug_annotation::Value::IntValue(i64::MIN)
    );
    assert_eq!(annotation("zero"), debug_annotation::Value::DoubleValue(0.));
    assert_eq!(
        annotation("predicate"),
        debug_annotation::Value::BoolValue(false)
    );
    assert_eq!(
        annotation("bad.nonfinite"),
        debug_annotation::Value::StringValue("Nan".into())
    );
    assert_eq!(
        annotation("absent.availability"),
        debug_annotation::Value::StringValue("not_supplied".into())
    );
    assert!(
        operation.context().is_active(),
        "capture has no scientific completion authority"
    );
}

/// Explicitly selected consumer qualification; ordinary tests never install or require this tool.
#[test]
#[ignore = "requires explicit PSE_TRACE_PROCESSOR binary and pinned checkout Python"]
fn capture_standalone_trace_processor_reads_numerical_events_with_clock_alignment() {
    use pse_diagnostics::observation::{ProducerRole, RecordField, Value};
    let binary = std::env::var_os("PSE_TRACE_PROCESSOR")
        .expect("select an existing Trace Processor binary explicitly");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let operation = owner.hub().start("Trace Processor consumer control");
    let producer = operation
        .context()
        .producer(ProducerRole::NativeIteration, "consumer numerical owner");
    let record = producer.record(
        "trace_consumer_control.v1",
        "recognizable native checkpoint",
        None,
        None,
        vec![RecordField {
            name: "iteration".into(),
            unit: "count".into(),
            value: Value::Count(7),
            availability: None,
        }],
    );
    assert!(producer.publish(&record));
    owner.publish().unwrap();
    let captured = start_capture(request(directory.path(), &owner, operation.context().id()))
        .unwrap()
        .wait()
        .unwrap();
    let output = captured
        .outputs
        .iter()
        .find(|output| output.kind == CaptureOutputKind::Perfetto && output.readable)
        .unwrap();
    let results = directory.path().join("actual-consumer");
    // Existing adapter applies a finite child clock/file allowance and joins its exact child.
    let status = std::process::Command::new(root.join(".venv/bin/python"))
        .arg(root.join("scripts/diagnostic_trace.py"))
        .arg("query")
        .arg(&output.path)
        .arg("numerical")
        .arg("--output")
        .arg(&results)
        .arg("--timeout")
        .arg("10")
        .arg("--output-bytes")
        .arg("65536")
        .arg("--binary")
        .arg(binary)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "{}",
        std::fs::read_to_string(results.join("stderr.txt")).unwrap_or_default()
    );
    let rows = std::fs::read_to_string(results.join("rows.csv")).unwrap();
    assert!(
        rows.contains("trace_consumer_control.v1")
            && rows.contains("recognizable native checkpoint"),
        "actual consumer lost source event: {rows}"
    );
    assert!(
        std::fs::read_to_string(results.join("stderr.txt"))
            .unwrap()
            .to_ascii_lowercase()
            .find("unknown clock")
            .is_none()
    );
    assert!(
        output.path.is_file(),
        "query does not consume or replace raw source"
    );
    assert!(operation.context().is_active());
}
