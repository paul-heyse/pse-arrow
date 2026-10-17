// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standard Perfetto protobuf packet streaming over immutable published owner facts.

use super::{
    CaptureError, CaptureOutput, CaptureOutputKind, CaptureRequest, Shared, document, failure,
    layer::{Lifecycle, LifecycleKind, ReceiverState},
    target,
};
use crate::observation::{ClockMapping, ProcessCheckpoint, read_checkpoint};
use prost::Message;
use pse_diagnostics::observation::{
    CoherentRecord, OperationId, OperationSnapshot, ProducerId, Value,
};
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tracing_perfetto_sdk_schema::{
    DebugAnnotation, Trace, TracePacket, TrackDescriptor, TrackEvent, debug_annotation,
    trace_packet, track_descriptor, track_event,
};

/// Writes a concatenation of ordinary one-packet Trace encodings: standard repeated field 1.
/// A length-delimited Trace message stream would be a different, invalid file format.
pub(super) struct PacketWriter<W> {
    writer: W,
    pub bytes: u64,
    pub complete: u64,
    pub flushed: u64,
    limit: u64,
    packet_limit: usize,
}
impl<W: Write> PacketWriter<W> {
    pub(super) fn new(writer: W, limit: u64, packet_limit: usize) -> Self {
        Self {
            writer,
            bytes: 0,
            complete: 0,
            flushed: 0,
            limit,
            packet_limit,
        }
    }
    pub(super) fn append(&mut self, packet: TracePacket) -> std::io::Result<()> {
        let trace = Trace {
            packet: vec![packet],
        };
        let size = trace.encoded_len();
        if size > self.packet_limit || self.bytes.saturating_add(size as u64) > self.limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::FileTooLarge,
                "capture output or packet allowance exhausted",
            ));
        }
        let encoded = trace.encode_to_vec();
        let mut remaining = encoded.as_slice();
        while !remaining.is_empty() {
            match self.writer.write(remaining) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "capture writer made no progress",
                    ));
                }
                Ok(count) => {
                    self.bytes += count as u64;
                    remaining = &remaining[count..];
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
        self.complete = self.bytes;
        Ok(())
    }
    pub(super) fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()?;
        self.flushed = self.complete;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Operation(OperationId),
    Phase(OperationId, u64),
    Metric(OperationId, Option<ProducerId>, String),
    Record(ProducerId),
    Thread(OperationId, u32, Duration),
    Span(u64),
    Events,
}
struct Track {
    uuid: u64,
    begun: bool,
    ended: bool,
    checkpoint: Option<(Duration, u64)>,
    metric_coverage: Option<(bool, u64, bool)>,
    record_checkpoint: Option<(u64, u64)>,
}
struct Projection<W> {
    output: PacketWriter<W>,
    tracks: HashMap<Key, Track>,
    links: Vec<(OperationId, OperationId, u8)>,
    next: u64,
    remaining_memory: usize,
    state: Shared,
    spans: HashMap<u64, Key>,
}
impl<W: Write> Projection<W> {
    fn new(writer: W, request: &CaptureRequest, state: &Shared) -> Self {
        Self {
            output: PacketWriter::new(
                writer,
                request.budget.output_bytes,
                request.budget.memory_bytes / 8,
            ),
            tracks: HashMap::new(),
            links: Vec::new(),
            next: 1,
            remaining_memory: request.budget.memory_bytes.saturating_sub(3 << 20),
            state: state.clone(),
            spans: HashMap::new(),
        }
    }
    fn packet(&mut self, packet: TracePacket) -> std::io::Result<()> {
        self.output.append(packet)
    }
    fn track(
        &mut self,
        key: Key,
        name: &str,
        parent: Option<u64>,
        counter: bool,
    ) -> std::io::Result<Option<u64>> {
        if let Some(track) = self.tracks.get(&key) {
            return Ok(Some(track.uuid));
        }
        let charge = 1024 + name.len().saturating_mul(2);
        if self.remaining_memory < charge || self.tracks.len() >= 2048 {
            let mut state = document(&self.state);
            state.loss.refused_items = state.loss.refused_items.saturating_add(1);
            return Ok(None);
        }
        self.remaining_memory -= charge;
        let uuid = self.next;
        self.next += 1;
        self.packet(TracePacket {
            data: Some(trace_packet::Data::TrackDescriptor(TrackDescriptor {
                uuid: Some(uuid),
                parent_uuid: parent,
                static_or_dynamic_name: Some(track_descriptor::StaticOrDynamicName::Name(
                    name.chars().take(512).collect(),
                )),
                counter: counter.then(Default::default),
                ..Default::default()
            })),
            ..Default::default()
        })?;
        self.tracks.insert(
            key,
            Track {
                uuid,
                begun: false,
                ended: false,
                checkpoint: None,
                metric_coverage: None,
                record_checkpoint: None,
            },
        );
        Ok(Some(uuid))
    }
    fn event(
        &mut self,
        uuid: u64,
        at: u64,
        kind: track_event::Type,
        name: &str,
        annotations: Vec<DebugAnnotation>,
    ) -> std::io::Result<()> {
        self.packet(event_packet(uuid, at, kind, name, annotations))
    }
    fn interval(
        &mut self,
        key: Key,
        uuid: u64,
        start: u64,
        end: Option<u64>,
        name: &str,
    ) -> std::io::Result<()> {
        if self.tracks.get(&key).is_some_and(|track| !track.begun) {
            self.event(uuid, start, track_event::Type::SliceBegin, name, Vec::new())?;
            if let Some(track) = self.tracks.get_mut(&key) {
                track.begun = true;
            }
        }
        if let Some(end) = end
            && end >= start
            && self.tracks.get(&key).is_some_and(|track| !track.ended)
        {
            self.event(uuid, end, track_event::Type::SliceEnd, "", Vec::new())?;
            if let Some(track) = self.tracks.get_mut(&key) {
                track.ended = true;
            }
        }
        Ok(())
    }
    fn operation(
        &mut self,
        operation: &OperationSnapshot,
        clock: Option<&ClockMapping>,
    ) -> std::io::Result<()> {
        let key = Key::Operation(operation.id);
        let label = format!(
            "{} [{:016x}-{:016x}/{}]",
            operation.kind,
            operation.id.process_instance[0],
            operation.id.process_instance[1],
            operation.id.sequence
        );
        let Some(uuid) = self.track(key.clone(), &label, None, false)? else {
            return Ok(());
        };
        let Some(clock) = clock else {
            document(&self.state).loss.unknown_clock = true;
            return Ok(());
        };
        let Some(start) = clock.monotonic_timestamp(operation.started_at) else {
            document(&self.state).loss.unknown_clock = true;
            return Ok(());
        };
        self.interval(
            key,
            uuid,
            start,
            operation
                .finished_at
                .and_then(|at| clock.monotonic_timestamp(at)),
            &operation.kind,
        )?;
        for phase in &operation.phases {
            self.phase(
                operation.id,
                uuid,
                phase.token,
                &phase.kind,
                phase.started_at,
                None,
                clock,
            )?;
        }
        for detail in &operation.detail {
            if let Some(phase) = &detail.phase {
                self.phase(
                    operation.id,
                    uuid,
                    phase.token,
                    &detail.name,
                    phase.started_at,
                    phase.finished_at,
                    clock,
                )?;
            }
        }
        for producer in &operation.producers {
            let record_key = Key::Record(producer.descriptor.id);
            let Some(track) = self.track(
                record_key.clone(),
                &format!(
                    "{} ({:?}) coherent records",
                    producer.descriptor.name, producer.descriptor.role
                ),
                Some(uuid),
                false,
            )?
            else {
                continue;
            };
            for record in producer.recent.iter().chain(producer.current.iter()) {
                self.record(
                    &record_key,
                    track,
                    record,
                    clock,
                    producer.coverage.partial || producer.coverage.history_dropped != 0,
                )?;
            }
        }
        for metric in &operation.metrics {
            let metric_key = Key::Metric(operation.id, metric.producer, metric.name.clone());
            let Some(track) = self.track(
                metric_key.clone(),
                &format!(
                    "{} [{}; producer {:?}]",
                    metric.name,
                    metric.unit,
                    metric.producer.map(|p| p.sequence)
                ),
                Some(uuid),
                true,
            )?
            else {
                continue;
            };
            let coverage = (
                metric.history_partial,
                metric.history_dropped,
                operation.coverage.sampled,
            );
            if self
                .tracks
                .get(&metric_key)
                .and_then(|track| track.metric_coverage)
                != Some(coverage)
            {
                if let Some(at) = clock.monotonic_timestamp(operation.read_at) {
                    self.event(
                        uuid,
                        at,
                        track_event::Type::Instant,
                        "metric history coverage",
                        vec![
                            annotation("metric", &metric.name),
                            count_annotation("metric_track", track),
                            bool_annotation("history_partial", metric.history_partial),
                            count_annotation("history_dropped", metric.history_dropped),
                            bool_annotation("source_sampled", operation.coverage.sampled),
                        ],
                    )?;
                    if let Some(track) = self.tracks.get_mut(&metric_key) {
                        track.metric_coverage = Some(coverage);
                    }
                } else {
                    document(&self.state).loss.unknown_clock = true;
                }
            }
            // Emit actual first/latest checkpoints; aggregates and bucket boundaries are not timestamps.
            for point in &metric.history {
                self.value(
                    &metric_key,
                    track,
                    point.first_observed_at,
                    point.first_generation,
                    &point.first_value,
                    clock,
                )?;
                self.value(
                    &metric_key,
                    track,
                    point.last_observed_at,
                    point.generation,
                    &point.value,
                    clock,
                )?;
            }
            self.value(
                &metric_key,
                track,
                metric.observed_at,
                metric.generation,
                &metric.latest,
                clock,
            )?;
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "projection keeps phase identity, parent track, label and source clock bounds explicit"
    )]
    fn phase(
        &mut self,
        operation: OperationId,
        parent: u64,
        token: u64,
        name: &str,
        started: Duration,
        finished: Option<Duration>,
        clock: &ClockMapping,
    ) -> std::io::Result<()> {
        let key = Key::Phase(operation, token);
        let Some(uuid) = self.track(key.clone(), name, Some(parent), false)? else {
            return Ok(());
        };
        if let Some(start) = clock.monotonic_timestamp(started) {
            self.interval(
                key,
                uuid,
                start,
                finished.and_then(|at| clock.monotonic_timestamp(at)),
                name,
            )?;
        } else {
            document(&self.state).loss.unknown_clock = true;
        }
        Ok(())
    }
    fn record(
        &mut self,
        key: &Key,
        uuid: u64,
        record: &CoherentRecord,
        clock: &ClockMapping,
        partial: bool,
    ) -> std::io::Result<()> {
        if self
            .tracks
            .get(key)
            .and_then(|t| t.record_checkpoint)
            .is_some_and(|(generation, sequence)| {
                generation == record.generation && sequence >= record.sequence
            })
        {
            return Ok(());
        }
        let Some(at) = clock.monotonic_timestamp(record.observed_at) else {
            document(&self.state).loss.unknown_clock = true;
            return Ok(());
        };
        let mut fields = vec![
            annotation("family", &record.family),
            count_annotation("source_sequence", record.sequence),
            count_annotation("generation", record.generation),
            bool_annotation("history_partial", partial),
        ];
        if let Some(coordinates) = &record.coordinates {
            fields.push(annotation("coordinates", coordinates));
        }
        if let Some(norm) = &record.norm {
            fields.push(annotation("norm", norm));
        }
        for field in &record.values {
            fields.push(annotation(
                &field.name,
                format!(
                    "{:?} [{}; {:?}]",
                    field.value, field.unit, field.availability
                ),
            ));
        }
        self.event(uuid, at, track_event::Type::Instant, &record.phase, fields)?;
        if let Some(track) = self.tracks.get_mut(key) {
            track.record_checkpoint = Some((record.generation, record.sequence));
        }
        Ok(())
    }
    fn value(
        &mut self,
        key: &Key,
        uuid: u64,
        at: Duration,
        generation: u64,
        value: &Value,
        clock: &ClockMapping,
    ) -> std::io::Result<()> {
        if self
            .tracks
            .get(key)
            .and_then(|track| track.checkpoint)
            .is_some_and(|last| last.0 >= at)
        {
            return Ok(());
        }
        let Some(timestamp) = clock.monotonic_timestamp(at) else {
            document(&self.state).loss.unknown_clock = true;
            return Ok(());
        };
        let counter = match value {
            Value::Count(value) => i64::try_from(*value)
                .ok()
                .map(track_event::CounterValueField::CounterValue),
            Value::Integer(value) => Some(track_event::CounterValueField::CounterValue(*value)),
            Value::Number(value) if value.is_finite() => {
                Some(track_event::CounterValueField::DoubleCounterValue(*value))
            }
            _ => None,
        };
        if let Some(counter) = counter {
            let mut packet = event_packet(
                uuid,
                timestamp,
                track_event::Type::Counter,
                "",
                vec![count_annotation("generation", generation)],
            );
            if let Some(trace_packet::Data::TrackEvent(event)) = packet.data.as_mut() {
                event.counter_value_field = Some(counter);
            }
            self.packet(packet)?;
        } else {
            // Unknown/nonfinite/text values remain explicit instant annotations, never counter zeroes.
            let parent = self.track(
                Key::Events,
                "Published nonnumeric observations",
                None,
                false,
            )?;
            if let Some(parent) = parent {
                self.event(
                    parent,
                    timestamp,
                    track_event::Type::Instant,
                    "unavailable/nonnumeric checkpoint",
                    vec![
                        annotation("value", format!("{value:?}")),
                        annotation("metric_track", uuid.to_string()),
                    ],
                )?;
            }
        }
        if let Some(track) = self.tracks.get_mut(key) {
            track.checkpoint = Some((at, generation));
        }
        Ok(())
    }
    fn link(
        &mut self,
        parent: &OperationSnapshot,
        child: &pse_diagnostics::observation::ChildSnapshot,
        clock: Option<&ClockMapping>,
    ) -> std::io::Result<()> {
        let relationship = match child.relationship {
            pse_diagnostics::observation::Relationship::Nested => 0,
            pse_diagnostics::observation::Relationship::DependencyWait => 1,
            pse_diagnostics::observation::Relationship::SharedProduction => 2,
            pse_diagnostics::observation::Relationship::Reuse => 3,
            pse_diagnostics::observation::Relationship::Continuation => 4,
        };
        let key = (parent.id, child.id, relationship);
        if self.links.contains(&key) {
            return Ok(());
        }
        if self.remaining_memory < 256 || self.links.len() >= 4096 {
            document(&self.state).loss.refused_items += 1;
            return Ok(());
        }
        let Some(at) = clock.and_then(|clock| clock.monotonic_timestamp(child.observed_at)) else {
            document(&self.state).loss.unknown_clock = true;
            return Ok(());
        };
        let Some(parent_track) =
            self.track(Key::Operation(parent.id), &parent.kind, None, false)?
        else {
            return Ok(());
        };
        let Some(child_track) = self.track(Key::Operation(child.id), &child.kind, None, false)?
        else {
            return Ok(());
        };
        let flow = self.next;
        self.next += 1;
        let mut source = event_packet(
            parent_track,
            at,
            track_event::Type::Instant,
            "observed relationship",
            vec![annotation(
                "relationship",
                format!("{:?}", child.relationship),
            )],
        );
        if let Some(trace_packet::Data::TrackEvent(event)) = source.data.as_mut() {
            event.flow_ids.push(flow);
        }
        self.packet(source)?;
        // Both endpoints timestamp the observed link in the parent's clock, not a reconstructed child start.
        let mut destination = event_packet(
            child_track,
            at,
            track_event::Type::Instant,
            "linked by parent",
            vec![annotation("source", "parent-observed relationship")],
        );
        if let Some(trace_packet::Data::TrackEvent(event)) = destination.data.as_mut() {
            event.terminating_flow_ids.push(flow);
        }
        self.packet(destination)?;
        self.links.push(key);
        self.remaining_memory -= 256;
        Ok(())
    }
    fn lifecycle(&mut self, packet: Lifecycle) -> std::io::Result<()> {
        let key = match packet.kind {
            LifecycleKind::Instant => Key::Events,
            LifecycleKind::Begin => Key::Span(self.next),
            LifecycleKind::End => match self.spans.remove(&packet.span) {
                Some(key) => key,
                None => return Ok(()),
            },
        };
        if matches!(packet.kind, LifecycleKind::End) {
            if let Some(track) = self.tracks.get(&key)
                && track.begun
                && !track.ended
            {
                let uuid = track.uuid;
                self.event(uuid, packet.at, track_event::Type::SliceEnd, "", Vec::new())?;
                if let Some(track) = self.tracks.get_mut(&key) {
                    track.ended = true;
                }
            }
            return Ok(());
        }
        let Some(uuid) = self.track(
            key.clone(),
            &format!("caller tracing lifecycle: {}", packet.name),
            None,
            false,
        )?
        else {
            return Ok(());
        };
        if matches!(packet.kind, LifecycleKind::Begin) {
            self.spans.insert(packet.span, key.clone());
        }
        let annotations = packet
            .operation
            .map(|operation| {
                vec![annotation(
                    "operation",
                    format!(
                        "{:016x}-{:016x}/{}",
                        operation.process_instance[0],
                        operation.process_instance[1],
                        operation.sequence
                    ),
                )]
            })
            .unwrap_or_default();
        self.event(
            uuid,
            packet.at,
            if matches!(packet.kind, LifecycleKind::Begin) {
                track_event::Type::SliceBegin
            } else {
                track_event::Type::Instant
            },
            &packet.name,
            annotations,
        )?;
        if matches!(packet.kind, LifecycleKind::Begin)
            && let Some(track) = self.tracks.get_mut(&key)
        {
            track.begun = true;
        }
        Ok(())
    }
    fn publish(&mut self, path: &Path, started: Instant, dropped: u64) -> std::io::Result<()> {
        let flushed = self.output.flush();
        let mut state = document(&self.state);
        state.observed_duration = started.elapsed();
        state.open_intervals = self
            .tracks
            .values()
            .filter(|track| track.begun && !track.ended)
            .count();
        state.loss.queued_packets_dropped = dropped;
        let output = CaptureOutput {
            path: path.to_path_buf(),
            kind: CaptureOutputKind::Perfetto,
            bytes: self.output.bytes,
            readable_prefix_bytes: self.output.flushed,
            readable: self.output.flushed > 0,
        };
        if let Some(existing) = state.outputs.first_mut() {
            *existing = output;
        } else {
            state.outputs.push(output);
        }
        flushed
    }
}
fn annotation(name: &str, value: impl ToString) -> DebugAnnotation {
    DebugAnnotation {
        name_field: Some(debug_annotation::NameField::Name(name.into())),
        value: Some(debug_annotation::Value::StringValue(
            value.to_string().chars().take(512).collect(),
        )),
        ..Default::default()
    }
}
fn bool_annotation(name: &str, value: bool) -> DebugAnnotation {
    DebugAnnotation {
        name_field: Some(debug_annotation::NameField::Name(name.into())),
        value: Some(debug_annotation::Value::BoolValue(value)),
        ..Default::default()
    }
}
fn count_annotation(name: &str, value: u64) -> DebugAnnotation {
    DebugAnnotation {
        name_field: Some(debug_annotation::NameField::Name(name.into())),
        value: Some(debug_annotation::Value::UintValue(value)),
        ..Default::default()
    }
}
fn event_packet(
    uuid: u64,
    at: u64,
    kind: track_event::Type,
    name: &str,
    annotations: Vec<DebugAnnotation>,
) -> TracePacket {
    TracePacket {
        timestamp: Some(at),
        timestamp_clock_id: Some(3),
        optional_trusted_packet_sequence_id: Some(
            trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(1),
        ),
        data: Some(trace_packet::Data::TrackEvent(TrackEvent {
            track_uuid: Some(uuid),
            r#type: Some(kind as i32),
            name_field: (!name.is_empty())
                .then(|| track_event::NameField::Name(name.chars().take(512).collect())),
            debug_annotations: annotations,
            ..Default::default()
        })),
        ..Default::default()
    }
}
fn path_for(request: &CaptureRequest, id: OperationId) -> Option<PathBuf> {
    Some(request.checkpoint.parent()?.join(format!(
        "{:016x}-{:016x}.json",
        id.process_instance[0], id.process_instance[1]
    )))
}
pub(super) fn run(
    request: &CaptureRequest,
    initial: ProcessCheckpoint,
    stop: &AtomicBool,
    state: &Shared,
    receiver: ReceiverState,
) -> Result<(), CaptureError> {
    let path = request.output.join("timeline.pftrace");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let mut projection = Projection::new(file, request, state);
    let started = Instant::now();
    let mut last_flush = started;
    let mut selected = vec![request.operation];
    let mut paths = vec![request.checkpoint.clone()];
    let mut first = Some(initial);
    let mut sequences = HashMap::<PathBuf, (u64, usize)>::new();
    let result = (|| -> Result<(), CaptureError> {
        loop {
            // Only explicit known links discover children; no unrelated checkpoint directory scan.
            let mut index = 0;
            while index < paths.len() {
                let image = if index == 0 && first.is_some() {
                    first.take()
                } else {
                    match read_checkpoint(&paths[index]) {
                        Ok(image) => Some(image),
                        Err(error) => {
                            failure(state, error);
                            None
                        }
                    }
                };
                if let Some(image) = image
                    && sequences.get(&paths[index]) != Some(&(image.sequence, selected.len()))
                {
                    let mut growing = true;
                    while growing {
                        growing = false;
                        for operation in &image.operations {
                            if !selected.contains(&operation.id) {
                                continue;
                            }
                            for child in &operation.children {
                                if !selected.contains(&child.id) {
                                    if selected.len() < 512 {
                                        selected.push(child.id);
                                        growing = true;
                                        if child.id.process_instance
                                            != operation.id.process_instance
                                            && let Some(path) = path_for(request, child.id)
                                        {
                                            if !paths.contains(&path) && paths.len() < 16 {
                                                paths.push(path);
                                            } else if !paths.contains(&path) {
                                                document(state).loss.refused_items += 1;
                                            }
                                        }
                                    } else {
                                        document(state).loss.refused_items += 1;
                                    }
                                }
                            }
                        }
                    }
                    // Preserve every known source gap before fallible packet output can stop
                    // projection. Source loss and writer/output loss are independent facts.
                    document(state).loss.source_partial |= super::source_partial(&image, &selected)
                        || image.operations.iter().any(|operation| {
                            selected.contains(&operation.id)
                                && (operation.interval_partial
                                    || operation.detail_partial
                                    || operation.coverage.sampled
                                    || operation.coverage.dropped_detail > 0
                                    || operation.metrics.iter().any(|metric| {
                                        metric.history_partial || metric.history_dropped > 0
                                    }))
                        });
                    for operation in &image.operations {
                        if !selected.contains(&operation.id) {
                            continue;
                        }
                        projection.operation(operation, image.clock.as_ref())?;
                        for child in &operation.children {
                            projection.link(operation, child, image.clock.as_ref())?;
                        }
                    }
                    for thread in &image.threads {
                        if !selected.contains(&thread.operation) {
                            continue;
                        }
                        let key = Key::Thread(thread.operation, thread.tid, thread.started_at);
                        let parent = projection
                            .tracks
                            .get(&Key::Operation(thread.operation))
                            .map(|track| track.uuid);
                        let Some(uuid) = projection.track(
                            key.clone(),
                            &format!(
                                "native thread {} start {:?}",
                                thread.tid, thread.start_ticks
                            ),
                            parent,
                            false,
                        )?
                        else {
                            continue;
                        };
                        if let Some(clock) = &image.clock {
                            if let Some(start) = clock.monotonic_timestamp(thread.started_at) {
                                projection.interval(
                                    key,
                                    uuid,
                                    start,
                                    thread
                                        .finished_at
                                        .and_then(|at| clock.monotonic_timestamp(at)),
                                    "actual native thread scope",
                                )?;
                            }
                        } else {
                            document(state).loss.unknown_clock = true;
                        }
                    }
                    let mut state = document(state);
                    let entry = target(&image, &selected);
                    if let Some(existing) = state
                        .targets
                        .iter_mut()
                        .find(|target| target.process == entry.process)
                    {
                        *existing = entry;
                    } else if state.targets.len() < 16 {
                        state.targets.push(entry);
                    }
                    sequences.insert(paths[index].clone(), (image.sequence, selected.len()));
                }
                index += 1;
            }
            // Drain a bounded batch so a caller cannot starve stop or prefix publication.
            for _ in 0..1024 {
                match receiver.receiver.try_recv() {
                    Ok(packet) => projection.lifecycle(packet)?,
                    Err(_) => break,
                }
            }
            if last_flush.elapsed() >= Duration::from_secs(1) {
                projection.publish(&path, started, receiver.dropped.load(Ordering::Relaxed))?;
                last_flush = Instant::now();
            }
            if stop.load(Ordering::Acquire) || started.elapsed() >= request.duration {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    })();
    if let Err(CaptureError::Io(error)) = &result
        && error.kind() == std::io::ErrorKind::FileTooLarge
    {
        document(state).loss.output_exhausted = true;
    }
    // Even a partial packet write preserves the previously complete flushed prefix.
    let flushed = projection.publish(&path, started, receiver.dropped.load(Ordering::Relaxed));
    if document(state).loss.output_exhausted {
        return flushed.map_err(Into::into);
    }
    result.and(flushed.map_err(Into::into))
}
