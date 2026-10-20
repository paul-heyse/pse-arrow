// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admission covers validation, borrowed selection/indexes, decoded context and bounded output.
use super::*;
use std::{
    io::{self, Write},
    mem::size_of,
};

struct Extent<'a> {
    bytes: usize,
    cancel: &'a pse_columnar::CancellationToken,
}
impl Write for Extent<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.cancel.is_cancelled() {
            return Err(io::Error::other("analysis cancelled during preparation"));
        }
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|size| *size <= crate::frozen_inspection::MAXIMUM_DOCUMENT_BYTES)
            .ok_or_else(|| {
                io::Error::other("analysis supplied set exceeds finite document admission")
            })?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
// The pinned Rust BTree has capacity 11 and at least five keys per nonroot
// node (alloc/collections/btree/node.rs). Include internal edges, alignment and
// split/root slack instead of charging a complete node for every borrowed item.
fn tree(entries: usize, key: usize, value: usize) -> usize {
    if entries == 0 {
        return 0;
    }
    entries.div_ceil(5).saturating_add(2).saturating_mul(
        160usize
            .saturating_add(11usize.saturating_mul(key.saturating_add(value).saturating_add(16))),
    )
}
fn checkpoint(cancel: &pse_columnar::CancellationToken) -> Result<(), ObservationError> {
    cancel
        .checkpoint()
        .map_err(|error| invalid(error.to_string()))
}
/// Lexical storage counting needs no JSON value, string scratch buffer or index.
/// Syntax and identity remain the existing validators' responsibility after admission.
fn decoded_extent(
    bytes: &[u8],
    cancel: &pse_columnar::CancellationToken,
) -> Result<usize, ObservationError> {
    let (mut nodes, mut strings, mut quoted, mut escaped, mut atom) =
        (0usize, 0usize, false, false, false);
    for (index, byte) in bytes.iter().copied().enumerate() {
        if index % 4096 == 0 {
            checkpoint(cancel)?;
        }
        if quoted {
            strings = strings.saturating_add(1);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else {
            match byte {
                b'"' => {
                    nodes = nodes.saturating_add(1);
                    quoted = true;
                    atom = false;
                }
                b'{' | b'[' => {
                    nodes = nodes.saturating_add(1);
                    atom = false;
                }
                b'}' | b']' | b',' | b':' | b' ' | b'\n' | b'\r' | b'\t' => {
                    atom = false;
                }
                _ if !atom => {
                    nodes = nodes.saturating_add(1);
                    atom = true;
                }
                _ => {}
            }
        }
    }
    // Typed vectors, map nodes, serde scratch and escaped strings. Counts include
    // object keys as well as values, conservatively covering ignored struct fields.
    Ok(nodes
        .saturating_mul(256)
        .saturating_add(strings.saturating_mul(2))
        .saturating_add(bytes.len()))
}
#[derive(Default)]
struct Working {
    records: usize,
    producers: usize,
    operations: usize,
    sources: usize,
    record_names: usize,
    record_bytes: usize,
    largest_tuple: usize,
    largest_value: usize,
    decoded: usize,
    scratch: usize,
}
impl Working {
    fn record(&mut self, record: &CoherentRecord) {
        self.records = self.records.saturating_add(1);
        self.sources = self
            .sources
            .saturating_add(1)
            .saturating_add(record.values.len());
        self.record_names = self
            .record_names
            .saturating_add(record.family.len())
            .saturating_add(record.phase.len());
        self.largest_tuple = self.largest_tuple.max(record.retained_bytes());
        self.record_bytes = self.record_bytes.saturating_add(record.retained_bytes());
        for field in &record.values {
            self.value(&field.value);
            if field.name == "demand"
                && let Value::Text(text) = &field.value
            {
                self.record_names = self.record_names.saturating_add(text.len());
            }
        }
    }
    fn value(&mut self, value: &Value) {
        let bytes = match value {
            Value::Text(text) if text.len() <= 2048 => text.len(),
            Value::Unavailable(text) => text.chars().take(1024).map(char::len_utf8).sum(),
            _ => 256,
        };
        self.largest_value = self.largest_value.max(bytes);
    }
}
/// Admission follows actual tree cardinalities, maximum concurrently decoded payloads,
/// transient tuple copies and bounded output. Supplied bytes themselves stay borrowed.
pub(super) fn admit(
    set: &FrozenEvidenceSet,
    request: &AnalysisRequest,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<Arc<pse_columnar::AllocationLease>, ObservationError> {
    checkpoint(cancel)?;
    let mut encoded = Extent { bytes: 0, cancel };
    serde_json::to_writer(&mut encoded, set).map_err(|error| invalid(error.to_string()))?;
    let mut state = Working::default();
    let mut numerical_copy = 0usize;
    for operation in set
        .operations
        .iter()
        .chain(set.checkpoints.iter().flat_map(|image| &image.operations))
    {
        checkpoint(cancel)?;
        state.operations = state.operations.saturating_add(1);
        state.producers = state.producers.saturating_add(operation.producers.len());
        state.sources = state
            .sources
            .saturating_add(operation.metrics.len())
            .saturating_add(operation.phases.len())
            .saturating_add(operation.children.len());
        for metric in &operation.metrics {
            state.value(&metric.latest);
            state.value(&metric.cumulative);
            let text = |value: &Value| match value {
                Value::Text(text) | Value::Unavailable(text) => text.len(),
                _ => 0,
            };
            state.largest_tuple = state.largest_tuple.max(
                metric
                    .name
                    .len()
                    .saturating_add(metric.unit.len())
                    .saturating_add(text(&metric.latest))
                    .saturating_add(text(&metric.cumulative)),
            );
        }
        for producer in &operation.producers {
            for record in &producer.recent {
                checkpoint(cancel)?;
                state.record(record);
            }
            if let Some(record) = &producer.current {
                state.record(record);
            }
        }
        if request.numerical_history_policy.is_some() && operation.id == request.operation {
            let mut extent = Extent { bytes: 0, cancel };
            serde_json::to_writer(&mut extent, operation)
                .map_err(|error| invalid(error.to_string()))?;
            numerical_copy = extent.bytes.saturating_mul(4);
        }
    }
    for record in &set.records {
        checkpoint(cancel)?;
        state.record(record);
    }
    for page in &set.detail_pages {
        state.producers = state
            .producers
            .saturating_add(page.producers.len())
            .saturating_add(page.activations.len());
        for record in &page.records {
            checkpoint(cancel)?;
            state.record(record);
        }
    }
    for context in &set.contexts {
        checkpoint(cancel)?;
        state.sources = state
            .sources
            .saturating_add(context.facts.len())
            .saturating_add(context.sources.len())
            .saturating_add(context.unavailable.len());
        for fact in &context.facts {
            checkpoint(cancel)?;
            state.scratch = state.scratch.max(fact.payload.len());
            state.largest_tuple = state
                .largest_tuple
                .max(fact.path.len().saturating_add(fact.codec.len()));
            if matches!(
                fact.codec.as_str(),
                "modeling-source-map.v1"
                    | "native-attempt-record.v1"
                    | "engine-execution-observations.v1"
                    | "replay-simulation-constructor.v2"
                    | "replay-horizon-constructor.v2"
                    | "replay-shooting-constructor.v2"
                    | "scientific-context.v1"
                    | "scientific-coordinates.v1"
            ) {
                state.decoded = state.decoded.max(decoded_extent(&fact.payload, cancel)?);
            }
        }
        // Validation uses borrowed path/source identity sets and cloned selected ordinals.
        state.sources = state
            .sources
            .saturating_add(context.source_positions.len())
            .saturating_add(context.selection.steps.len());
        for source in &context.sources {
            state.sources = state.sources.saturating_add(source.objects.len());
        }
        for unavailable in &context.unavailable {
            state.largest_tuple = state.largest_tuple.max(
                unavailable
                    .path
                    .len()
                    .saturating_add(unavailable.reason.len()),
            );
        }
    }
    for inventory in &set.inventories {
        checkpoint(cancel)?;
        state.sources = state
            .sources
            .saturating_add(inventory.members.len())
            .saturating_add(inventory.gaps.len());
        for member in &inventory.members {
            state.sources = state.sources.saturating_add(member.links.len());
            state.largest_tuple = state.largest_tuple.max(
                member
                    .key
                    .len()
                    .saturating_add(member.role.len())
                    .saturating_add(member.run.as_ref().map_or(0, String::len))
                    .saturating_add(member.attempt.as_ref().map_or(0, String::len))
                    .saturating_add(member.outcome.as_ref().map_or(0, String::len))
                    .saturating_add(member.result_manifest.as_ref().map_or(0, String::len)),
            );
            for link in &member.links {
                state.largest_tuple = state
                    .largest_tuple
                    .max(link.role.len().saturating_add(link.target.len()));
            }
        }
        for gap in &inventory.gaps {
            state.largest_tuple = state
                .largest_tuple
                .max(gap.member.len().saturating_add(gap.reason.len()));
        }
    }
    for invocation in &set.invocations {
        checkpoint(cancel)?;
        // The invocation owner bounds scalar text to 512 bytes, including option values.
        state.largest_value = state.largest_value.max(512);
        state.sources = state
            .sources
            .saturating_add(invocation.attempts.steps.len())
            .saturating_add(invocation.products.len())
            .saturating_add(invocation.attempts.native_attempts.len());
        for step in &invocation.attempts.steps {
            state.sources = state
                .sources
                .saturating_add(step.findings.len())
                .saturating_add(step.decisions.len())
                .saturating_add(step.effective_options.len())
                .saturating_add(step.native_defaults.len());
        }
    }
    for trace in &set.engine_observations {
        checkpoint(cancel)?;
        state.sources = state.sources.saturating_add(trace.observations.len());
        for observation in &trace.observations {
            let mut extent = Extent { bytes: 0, cancel };
            serde_json::to_writer(&mut extent, observation)
                .map_err(|error| invalid(error.to_string()))?;
            state.largest_tuple = state.largest_tuple.max(extent.bytes);
            if let Some(execution) = &observation.execution {
                state.sources = state.sources.saturating_add(execution.metrics.len());
            }
        }
    }
    let statements = request.statements.min(256);
    let source_limit = request.source_items.min(4096);
    let retained = state.records.min(source_limit);
    let pointer = if set.invocations.is_empty() {
        160usize
    } else {
        2304usize
    };
    let reference = size_of::<AnalysisSourceReference>()
        .saturating_mul(2)
        .saturating_add(pointer)
        .saturating_add(state.largest_value.max(256));
    // Event tuple statements retain at most 17 fields/references. The invocation
    // owner uses smaller fixed tuples (including bounded strategy references).
    let references_per_statement = 17usize;
    let subjects = state.operations.saturating_add(2).min(64);
    let requirements = subjects
        .saturating_mul(if request.question.is_some() { 4 } else { 16 })
        .min(384);
    let validation = tree(set.records.len(), size_of::<EventKey>(), 0).saturating_add(tree(
        state.operations.saturating_add(set.invocations.len()),
        48,
        0,
    ));
    let event_indexes = tree(state.records, size_of::<EventKey>(), size_of::<Event<'_>>())
        .saturating_add(tree(state.records, 64, size_of::<Event<'_>>()))
        .saturating_add(state.records.saturating_mul(192))
        .saturating_add(tree(state.producers, 32, 32));
    let source_keys = tree(state.sources, size_of::<String>(), 0)
        .saturating_mul(2)
        .saturating_add(
            state
                .sources
                .saturating_mul(pointer.saturating_add(48))
                .saturating_mul(2),
        );
    let groups = tree(retained, 128, 128)
        .saturating_mul(4)
        .saturating_add(retained.saturating_mul(reference).saturating_mul(4))
        .saturating_add(state.record_names.saturating_mul(4));
    let output = statements
        .saturating_mul(
            8192usize.saturating_add(references_per_statement.saturating_mul(reference)),
        )
        .saturating_add(requirements.saturating_mul(8192));
    // Two decoded payloads can coexist in native attribution; a third footprint covers
    // projection fields/paths. Map term replication is admitted separately below.
    if request.numerical_history_policy.is_some() {
        numerical_copy = numerical_copy.saturating_add(state.record_bytes.saturating_mul(2));
    }
    let capacity = 65536usize
        .saturating_add(validation)
        .saturating_add(event_indexes)
        .saturating_add(source_keys)
        .saturating_add(groups)
        .saturating_add(output)
        .saturating_add(state.largest_tuple.saturating_mul(8))
        .saturating_add(numerical_copy)
        .saturating_add(state.scratch)
        .saturating_add(state.decoded.saturating_mul(3));
    if capacity == usize::MAX {
        return Err(invalid("analysis allocation extent overflow"));
    }
    let reservation = pse_columnar::MemoryConsumer::new("review:analysis").register(pool);
    reservation
        .try_grow(capacity)
        .map_err(|error| invalid(format!("analysis allocation admission refused: {error}")))?;
    checkpoint(cancel)?;
    // This parse happens under its already reserved decoded footprint. An occurrence
    // duplicates its checked-member terms for every matched slot/addend in the helper;
    // count that actual amplification instead of hiding it in a source-byte multiplier.
    let mut projection = 0usize;
    for (context, fact) in set
        .contexts
        .iter()
        .flat_map(|context| context.facts.iter().map(move |fact| (context, fact)))
        .filter(|(_, fact)| fact.codec == "modeling-source-map.v1")
    {
        checkpoint(cancel)?;
        let physical = request
            .question
            .is_none_or(|question| question == AnalysisQuestion::PhysicalAttribution);
        let scope = fact.path.strip_suffix("/source_map").unwrap_or("");
        let used = physical
            && if scope.contains("/native/") {
                context
                    .facts
                    .iter()
                    .any(|fact| fact.codec == "native-attempt-record.v1" && fact.path == scope)
            } else {
                set.invocations.iter().any(|invocation| {
                    context.run_id == Some(invocation.run_id)
                        && context.operation == invocation.operation
                        && invocation.attempts.steps.iter().any(|step| {
                            !step.findings.is_empty() && scope == format!("/steps/{}", step.step)
                        })
                })
            };
        if !used {
            continue;
        }
        let map: crate::workflow::ModelingSourceMapDocument =
            serde_json::from_slice(&fact.payload).map_err(|error| invalid(error.to_string()))?;
        let mut terms = 0usize;
        for occurrence in &map.instances {
            checkpoint(cancel)?;
            terms = terms.saturating_add(
                occurrence.checked_members.len().saturating_mul(
                    occurrence
                        .slots
                        .len()
                        .saturating_add(occurrence.contributions.len()),
                ),
            );
        }
        projection = projection.max(terms.saturating_mul(512));
    }
    if projection == usize::MAX {
        return Err(invalid("analysis source-map projection extent overflow"));
    }
    reservation.try_grow(projection).map_err(|error| {
        invalid(format!(
            "analysis source-map allocation admission refused: {error}"
        ))
    })?;
    checkpoint(cancel)?;
    Ok(pse_columnar::AllocationLease::new(reservation))
}
pub(super) fn same_record(left: &CoherentRecord, right: &CoherentRecord) -> bool {
    left.producer == right.producer
        && left.generation == right.generation
        && left.sequence == right.sequence
        && left.family_sequence == right.family_sequence
        && left.observed_at == right.observed_at
        && left.family == right.family
        && left.phase == right.phase
        && left.coordinates == right.coordinates
        && left.norm == right.norm
        && left.values.len() == right.values.len()
        && left.values.iter().zip(&right.values).all(|(left, right)| {
            left.name == right.name
                && left.unit == right.unit
                && left.value == right.value
                && left.availability == right.availability
        })
}
pub(super) fn selections(
    set: &FrozenEvidenceSet,
    request: &AnalysisRequest,
    ids: &[OperationId],
) -> Vec<AnalysisSelection> {
    ids.iter()
        .map(|id| {
            let aligned = id.process_instance == request.operation.process_instance;
            let snapshot = set.operation(*id);
            let explicit = aligned.then_some(&request.window);
            let interval = if snapshot.is_none() {
                None
            } else {
                match explicit {
                    Some(RecordedWindow::Interval { start, end }) => Some((*start, *end)),
                    Some(RecordedWindow::EndingAt { at }) => Some((Duration::ZERO, *at)),
                    _ => snapshot.map(|snapshot| (snapshot.interval_start, snapshot.interval_end)),
                }
            };
            AnalysisSelection {
                operation: *id,
                process_clock: id.process_instance,
                interval_start: interval.map(|interval| interval.0),
                interval_end: interval.map(|interval| interval.1),
                scope: if snapshot.is_none() {
                    AnalysisSelectionScope::Unknown
                } else if aligned {
                    AnalysisSelectionScope::Direct
                } else {
                    AnalysisSelectionScope::Independent
                },
                aligned_to_request: aligned,
            }
        })
        .collect()
}
