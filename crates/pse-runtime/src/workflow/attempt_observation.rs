// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive exact-attempt history over the existing protected terminal progress reader.
use super::{
    ProgressCoverage, ProgressStream, RetainedDataServices, StudyObservationContinuity,
    StudyObservationPosition, StudyObservedSource, WorkflowError, contract,
};
use pse_columnar::{CancellationToken, MemoryReservation};
use pse_operations::canonical::MetadataResponse;
use std::sync::Arc;

/// Exact original attempt and an explicitly bounded native progress page.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptObservationRequest {
    /// Exact original canonical run, never a latest pointer.
    pub run: String,
    /// Exact original canonical attempt; retries retain distinct identities.
    pub attempt: String,
    /// Maximum whole events in this page, from one through 512.
    pub maximum_events: usize,
    /// Original issuer/attempt/source-bound reconnect position.
    pub cursor: Option<AttemptObservationCursor>,
}
impl AttemptObservationRequest {
    pub(crate) fn validate(&self) -> Result<(), WorkflowError> {
        if !(1..=512).contains(&self.maximum_events) {
            return Err(contract("attempt observer event allowance outside bounds"));
        }
        Ok(())
    }
}
/// Original native metadata only; completion payloads and execution requests are not decoded.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AttemptObservationOrigin {
    /// Exact issuing canonical database.
    pub issuing_store: String,
    /// Original execution key.
    pub run: String,
    /// Original attempt key.
    pub attempt: String,
    /// Original scientific source revision.
    pub source_revision: String,
    /// Actual canonical retry generation.
    pub generation: u64,
    /// Original canonical phase, without interpreting success from the read.
    pub phase: pse_model::generated::enums::CanonicalAttemptPhase,
    /// Original canonical outcome, absent while unavailable or nonterminal.
    pub outcome: Option<String>,
    /// Exact native closed manifest locator when present; this is not a permission.
    pub closed_manifest: Option<String>,
}
/// Portable native progress coordinates. Decoding supplies no protected result capability.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptObservationCursor {
    /// Original exact canonical source identity and context.
    pub origin: AttemptObservationOrigin,
    /// The original whole-event allowance cannot silently change on reconnect.
    pub maximum_events: usize,
    /// Existing source and immutable-batch coordinates, independent of observer correlation.
    pub position: StudyObservationPosition,
}
#[derive(Debug)]
struct Custody {
    _header: Arc<MemoryReservation>,
    _history: Option<ProgressStream>,
}
/// One original attempt snapshot/history page, held under its actual native read custody.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct AttemptObservationPage {
    /// Original lineage and lifecycle outcome, never replaced by command success.
    pub origin: AttemptObservationOrigin,
    /// Whole native events, original coverage, and explicit live/history unavailability.
    pub source: StudyObservedSource,
    /// Exact reconnect coordinates; absent only after native terminal history exhaustion.
    pub next: Option<AttemptObservationCursor>,
    #[serde(skip)]
    #[schemars(skip)]
    _custody: Arc<Custody>,
}
fn origin(
    value: pse_operations::canonical_execution::CanonicalResultOrigin,
    issuer: String,
) -> Result<AttemptObservationOrigin, WorkflowError> {
    let phase = value
        .phase
        .parse()
        .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
    Ok(AttemptObservationOrigin {
        issuing_store: issuer,
        run: value.run,
        attempt: value.key,
        source_revision: value.source_revision,
        generation: value.generation,
        phase,
        outcome: value.outcome,
        closed_manifest: value.closed_manifest,
    })
}
fn position(
    request: &AttemptObservationRequest,
    origin: &AttemptObservationOrigin,
) -> Result<StudyObservationPosition, WorkflowError> {
    let Some(cursor) = &request.cursor else {
        return Ok(StudyObservationPosition {
            run: request.run.clone(),
            attempt: request.attempt.clone(),
            attempt_generation: origin.generation,
            metadata_revision: None,
            source_generation: None,
            source_sequence: 0,
            batch: 0,
            offset: 0,
            durable_sequence: None,
            history_complete: false,
        });
    };
    let old = &cursor.origin;
    if cursor.maximum_events != request.maximum_events
        || old.issuing_store != origin.issuing_store
        || old.run != origin.run
        || old.attempt != origin.attempt
        || old.generation != origin.generation
        || old.source_revision != origin.source_revision
        || old
            .closed_manifest
            .as_ref()
            .is_some_and(|manifest| origin.closed_manifest.as_ref() != Some(manifest))
        || cursor.position.run != origin.run
        || cursor.position.attempt != origin.attempt
        || cursor.position.attempt_generation != origin.generation
        || cursor.position.metadata_revision.is_some()
        || cursor.position.offset > 512
        || cursor.position.history_complete
    {
        return Err(contract(
            "attempt observer cursor issuer, original identity or selection differs",
        ));
    }
    Ok(cursor.position.clone())
}
impl RetainedDataServices {
    /// Read one explicitly selected original attempt without acquiring an execution lease.
    /// Retained output owns its actual metadata/progress protection until serialization ends.
    pub async fn observe_attempt(
        &self,
        request: AttemptObservationRequest,
    ) -> Result<AttemptObservationPage, WorkflowError> {
        request.validate()?;
        self.scope_activity(async {
            let store = self.canonical_store()?;
            let header = super::connected_results::pooled_response_owner(
                &self.pool(),
                "canonical:attempt-observation-origin",
                MetadataResponse::Header,
            )?;
            let original = pse_operations::canonical::response_ownership_scope(
                header.clone(),
                store.canonical_result_origin(&request.run, &request.attempt),
            )
            .await?
            .ok_or_else(|| contract("selected original attempt is absent"))?;
            if original.run != request.run || original.key != request.attempt {
                return Err(contract(
                    "selected attempt does not belong to the original run",
                ));
            }
            let origin = origin(original, store.issuing_store()?.to_owned())?;
            let mut position = position(&request, &origin)?;
            let mut source = StudyObservedSource {
                position: position.clone(),
                continuity: StudyObservationContinuity::CatchUp,
                events: Vec::new(),
                checkpoint: None,
                coverage: None,
                history_refusal: None,
            };
            let terminal = origin.phase.terminal();
            let mut retained = None;
            if !terminal {
                source.continuity = StudyObservationContinuity::LiveNumericsUnavailable;
            } else if !position.history_complete {
                match self
                    .progress(&request.run, &request.attempt, CancellationToken::new())
                    .await
                {
                    Ok(mut history) => {
                        source.coverage = Some(history.coverage().clone());
                        history.resume_batch(position.batch)?;
                        if let Some(page) = history.next_page().await? {
                            let length = page.len();
                            let begin = position.offset;
                            if begin > length {
                                return Err(contract(
                                    "attempt observer event offset exceeds original batch",
                                ));
                            }
                            let finish = length.min(begin + request.maximum_events);
                            for event in page.into_iter().skip(begin).take(finish - begin) {
                                position.durable_sequence = event.sequence;
                                if let Some(gap) =
                                    super::study_observation::consume(&mut position, &event)
                                {
                                    if gap {
                                        source.continuity =
                                            StudyObservationContinuity::GapCurrentResync;
                                    }
                                    source.events.push(event);
                                }
                            }
                            if finish == length {
                                position.batch = history.batch_position();
                                position.offset = 0;
                            } else {
                                position.offset = finish;
                            }
                        } else {
                            position.history_complete = true;
                            if matches!(history.coverage(), ProgressCoverage::Unknown {}) {
                                source.continuity = StudyObservationContinuity::HistoryUnavailable;
                            }
                        }
                        // Retain this exact read and its allocation through page encoding.
                        retained = Some(history);
                    }
                    Err(error) => match super::study_observation::unavailable_history(&error) {
                        Some(code) => {
                            source.continuity = StudyObservationContinuity::HistoryUnavailable;
                            source.history_refusal = Some(code);
                        }
                        None => return Err(error),
                    },
                }
            }
            source.position = position.clone();
            let next = (!position.history_complete).then(|| AttemptObservationCursor {
                origin: origin.clone(),
                maximum_events: request.maximum_events,
                position,
            });
            Ok(AttemptObservationPage {
                origin,
                source,
                next,
                _custody: Arc::new(Custody {
                    _header: header,
                    _history: retained,
                }),
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests;
