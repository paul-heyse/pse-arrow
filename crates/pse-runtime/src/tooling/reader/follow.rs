// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Product-owned continuity for one copied checkpoint, without a workload cancellation owner.
use super::*;

/// Original performing producer position; it is unrelated to CLI invocation ordinals.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerPosition {
    /// Performing source identity.
    pub producer: ProducerId,
    /// Original reset generation.
    pub generation: u64,
    /// Last observed whole record sequence.
    pub sequence: u64,
}
/// Reconnect coordinates are checked against the complete selected inspection context.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FollowCursor {
    /// Exact source/operation/window/association selection digest.
    pub query: String,
    /// Original diagnostic process instance.
    pub process: String,
    /// Original copied publication sequence.
    pub publication_sequence: u64,
    /// Original copied checkpoint bytes.
    pub source_identity: String,
    /// Last known performing producer positions, independent of publication sequence.
    pub producers: Vec<ProducerPosition>,
}
/// Relationship of this sample to the supplied original resume coordinates.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Continuity {
    /// No prior source coordinates were supplied.
    Initial,
    /// Identical source bytes; freshness is still assessed at this read.
    Unchanged,
    /// Same publication and producer generations advanced.
    Updated,
    /// A source incarnation, publication counter or producer generation changed.
    Resynchronized,
}
/// One bounded passive observation; terminal describes the observed operation only.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FollowSample {
    /// Original inspection, process and association sections.
    pub observation: Box<Payload>,
    /// Original source-bound reconnect coordinates, including terminal samples.
    pub cursor: FollowCursor,
    /// Source continuity, never inferred scientific convergence.
    pub continuity: Continuity,
    /// Publications skipped between samples; their numerical event membership is unknown.
    pub skipped_publications: u64,
    /// Native coverage or retained producer history leaves an explicit event gap.
    pub gap: bool,
    /// Producer generation or source incarnation changed; current sample resynchronizes.
    pub reset: bool,
    /// Owner published an original terminal outcome. Ending a poll allowance is separate.
    pub terminal: bool,
    #[serde(skip)]
    #[schemars(skip)]
    next: Option<Box<serde_json::value::RawValue>>,
}
impl FollowSample {
    pub(super) fn next_request(&self) -> Option<Box<serde_json::value::RawValue>> {
        self.next.clone()
    }
}
pub(super) fn sample(
    source: CheckpointSelection,
    request: InspectionRequest,
    invocation: Option<PathBuf>,
    resume: Option<FollowCursor>,
) -> Result<Payload, ReaderError> {
    let context = query(&(&source, &request, &invocation))?;
    if resume
        .as_ref()
        .is_some_and(|cursor| cursor.query != context || cursor.producers.len() > 4096)
    {
        return Err(invalid(
            "follow coordinates belong to another selection or exceed source extent",
        ));
    }
    let read = match read_checkpoint_source(source.path()) {
        Ok(read) => read,
        Err(error) => return Ok(unavailable(source.path(), "checkpoint.unavailable", &error)),
    };
    source.validate(&read.image)?;
    let Some(operation) = read
        .image
        .operations
        .iter()
        .find(|operation| operation.id == request.operation)
    else {
        return Ok(unavailable(
            source.path(),
            "operation.unavailable",
            &invalid("selected operation is absent from the copied checkpoint"),
        ));
    };
    let producers: Vec<_> = operation
        .producers
        .iter()
        .filter_map(|producer| {
            producer
                .current
                .as_ref()
                .or_else(|| producer.recent.last())
                .map(|record| ProducerPosition {
                    producer: record.producer,
                    generation: record.generation,
                    sequence: record.sequence,
                })
        })
        .collect();
    let mut reset = false;
    let mut gap = operation.coverage.curtailed
        || operation.coverage.dropped_updates > 0
        || operation.coverage.dropped_detail > 0
        || operation.coverage.refused_items > 0
        || operation.coverage.counter_overflow
        || operation.detail_partial
        || read.image.omitted > 0
        || operation.producers.iter().any(|producer| {
            producer.coverage.partial
                || producer.coverage.refused > 0
                || producer.coverage.history_dropped > 0
        });
    let mut skipped_publications = 0;
    let continuity = match &resume {
        None => Continuity::Initial,
        Some(previous) => {
            reset = previous.process != read.image.process
                || previous.publication_sequence > read.image.sequence
                || (previous.publication_sequence == read.image.sequence
                    && previous.source_identity != read.sha256);
            skipped_publications = read
                .image
                .sequence
                .saturating_sub(previous.publication_sequence)
                .saturating_sub(1);
            for old in &previous.producers {
                if let Some(current) = producers
                    .iter()
                    .find(|current| current.producer == old.producer)
                {
                    reset |=
                        old.generation != current.generation || current.sequence < old.sequence;
                    if old.generation == current.generation && current.sequence > old.sequence {
                        let retained = operation
                            .producers
                            .iter()
                            .find(|producer| producer.descriptor.id == old.producer);
                        let first_new = retained
                            .into_iter()
                            .flat_map(|producer| {
                                producer.recent.iter().chain(producer.current.iter())
                            })
                            .filter(|record| {
                                record.generation == old.generation
                                    && record.sequence > old.sequence
                            })
                            .map(|record| record.sequence)
                            .min();
                        gap |= first_new
                            .is_none_or(|sequence| sequence > old.sequence.saturating_add(1));
                    }
                } else {
                    gap = true;
                }
            }
            if reset {
                Continuity::Resynchronized
            } else if previous.source_identity == read.sha256 {
                Continuity::Unchanged
            } else {
                Continuity::Updated
            }
        }
    };
    let terminal = operation.outcome.is_some();
    let cursor = FollowCursor {
        query: context,
        process: read.image.process.clone(),
        publication_sequence: read.image.sequence,
        source_identity: read.sha256.clone(),
        producers,
    };
    let next = if terminal {
        None
    } else {
        Some(serde_json::value::to_raw_value(&Request::Follow {
            source: source.clone(),
            request: request.clone(),
            invocation: invocation.clone(),
            resume: Some(cursor.clone()),
        })?)
    };
    let association = invocation
        .as_ref()
        .map(|path| association(path, request.operation))
        .transpose()?;
    let process = ProcessFacts {
        managed_invocation: managed_candidate(&read.image),
        current_start_identity: read
            .image
            .locator
            .start_ticks
            .map(|_| read.image.locator.is_current()),
        process: read.image.process.clone(),
        locator: read.image.locator.clone(),
        publication_sequence: read.image.sequence,
        published_unix_micros: read.image.published_unix_micros,
        resources: read.image.resources.clone(),
    };
    let facts = source_facts(source.path(), read.sha256, Some(read.bytes), 1);
    let document = inspection::inspect_image(source.path(), request, read.image);
    Ok(Payload::Follow {
        sample: Box::new(FollowSample {
            observation: Box::new(Payload::Inspection {
                document: Box::new(document),
                process,
                source: facts,
                association,
            }),
            cursor,
            continuity,
            skipped_publications,
            gap,
            reset,
            terminal,
            next,
        }),
    })
}
