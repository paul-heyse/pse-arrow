// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Required framing over the supervisor's independently retained original pipes.
use crate::{
    envelope::RedactionPolicy,
    extraction::{
        ExtractionError, NextestExtractor, NextestRecord, NextestSummary, NextestTestEvent,
        OutputStream, RecordOrigin,
    },
    managed::streams::ChunkObserver,
    stream_records::{Frame, RecordFramer},
};
use std::{collections::BTreeMap, num::NonZeroUsize};

/// One required protocol failure, retaining its exact source attribution and concrete cause.
#[derive(Debug, thiserror::Error)]
#[error("nextest observation at {origin:?}: {error}")]
pub struct ObservationError {
    /// Original raw stdout location, independently of normalized presentation.
    pub origin: RecordOrigin,
    /// Original decode or framing failure.
    #[source]
    pub error: ExtractionError,
}
pse_diagnostics::impl_diagnostic! {
    ObservationError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { Some(&this.error) }, help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        facts.observe("stdout_byte_offset", pse_diagnostics::DiagnosticObservation::Text(this.origin.byte_offset.to_string()));
        facts.observe("stdout_byte_length", pse_diagnostics::DiagnosticObservation::Text(this.origin.byte_length.to_string()));
        facts
    }
}
/// Selected failure address, suitable for a later bounded original-record read.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct FailureReference {
    /// Exact captured identity; same-named tests in other binaries remain distinct.
    pub identity: crate::discovery::TestIdentity,
    /// Exact byte range in the independently retained stdout stream.
    pub origin: RecordOrigin,
}
/// Reconciled workload facts, independent of the supervisor's physical settlement.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct Observation {
    /// Owner-constructed workload interpretation.
    pub summary: NextestSummary,
    /// Original records for exact selected failures; inline output is not accumulated in RAM.
    pub failures: Vec<FailureReference>,
}
/// A required interpreter, with bounded framing and state proportional to captured population.
#[derive(Debug)]
pub struct NextestObserver {
    framer: RecordFramer,
    extractor: NextestExtractor,
    policy: RedactionPolicy,
    next_offset: u64,
    frame_offset: u64,
    ordinal: u64,
    failures: BTreeMap<crate::discovery::TestIdentity, RecordOrigin>,
}
impl NextestObserver {
    /// The frame bound limits protocol decoding only; raw byte retention has its own owner.
    pub fn new(extractor: NextestExtractor, limit: NonZeroUsize, policy: RedactionPolicy) -> Self {
        Self {
            framer: RecordFramer::new(limit),
            extractor,
            policy,
            next_offset: 0,
            frame_offset: 0,
            ordinal: 0,
            failures: BTreeMap::new(),
        }
    }
    /// Consume every complete frame after any previous failure; return this chunk's first cause.
    fn frames(&mut self, bytes: &[u8], eof: bool) -> Option<ObservationError> {
        let Self {
            framer,
            extractor,
            policy,
            frame_offset,
            ordinal,
            failures,
            ..
        } = self;
        let mut first_error = None;
        let mut consume = |frame: Frame<'_>| {
            let (length, terminal) = match frame {
                Frame::Complete(bytes) | Frame::Truncated(bytes) => (bytes.len() as u64, true),
                Frame::Oversized { bytes, last, .. } => (bytes.len() as u64, last),
            };
            let origin = RecordOrigin {
                stream: OutputStream::Stdout,
                byte_offset: *frame_offset,
                byte_length: length,
                ordinal: *ordinal,
            };
            let result = match frame {
                Frame::Complete(bytes) => {
                    extractor
                        .consume(bytes, origin.clone(), policy)
                        .map(|record| {
                            if let NextestRecord::Test {
                                identity,
                                event: NextestTestEvent::Failed,
                                origin,
                                ..
                            } = record
                            {
                                failures.insert(identity, origin);
                            }
                        })
                }
                Frame::Oversized { first, .. } => {
                    if first {
                        extractor.mark_incomplete();
                    }
                    Err(ExtractionError::Nextest("required record exceeds the selected decoding bound; original bytes remain in raw custody".into()))
                }
                Frame::Truncated(_) => {
                    extractor.mark_incomplete();
                    Err(ExtractionError::Nextest(
                        "stdout ended before a required record delimiter".into(),
                    ))
                }
            };
            if let Err(error) = result
                && first_error.is_none()
            {
                first_error = Some(ObservationError { origin, error });
            }
            *frame_offset = frame_offset.saturating_add(length);
            if terminal {
                *ordinal = ordinal.saturating_add(1);
            }
        };
        if eof {
            framer.finish(&mut consume);
        } else {
            framer.push(bytes, &mut consume);
        }
        first_error
    }
    /// Finish only after capture has delivered every available byte. A false required-capture
    /// flag preserves loss even if the child and every decoded tail record reported success.
    pub fn finish(
        mut self,
        child_exit: i32,
        required_capture_complete: bool,
    ) -> (Observation, Option<ObservationError>) {
        let error = self.frames(&[], true);
        if !required_capture_complete {
            self.extractor.mark_incomplete();
        }
        let observation = Observation {
            summary: self.extractor.finish(child_exit),
            failures: self
                .failures
                .into_iter()
                .map(|(identity, origin)| FailureReference { identity, origin })
                .collect(),
        };
        (observation, error)
    }
}
impl ChunkObserver for NextestObserver {
    type Error = ObservationError;
    fn observe(
        &mut self,
        stream: OutputStream,
        byte_offset: u64,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        if stream == OutputStream::Stderr {
            return Ok(());
        }
        if byte_offset != self.next_offset {
            self.extractor.mark_incomplete();
            return Err(ObservationError {
                origin: RecordOrigin {
                    stream,
                    byte_offset,
                    byte_length: bytes.len() as u64,
                    ordinal: self.ordinal,
                },
                error: ExtractionError::Nextest(
                    "required stdout byte delivery is discontinuous".into(),
                ),
            });
        }
        self.next_offset = self.next_offset.saturating_add(bytes.len() as u64);
        self.frames(bytes, false).map_or(Ok(()), Err)
    }
}
