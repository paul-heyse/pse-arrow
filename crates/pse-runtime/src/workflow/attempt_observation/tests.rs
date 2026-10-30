// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_model::generated::enums::CanonicalAttemptPhase;

fn request() -> AttemptObservationRequest {
    AttemptObservationRequest {
        run: "original-run".into(),
        attempt: "original-attempt".into(),
        maximum_events: 2,
        cursor: None,
    }
}
fn origin() -> AttemptObservationOrigin {
    AttemptObservationOrigin {
        issuing_store: "original-store".into(),
        run: "original-run".into(),
        attempt: "original-attempt".into(),
        source_revision: "original-revision".into(),
        generation: 3,
        phase: CanonicalAttemptPhase::Terminal,
        outcome: Some("failed".into()),
        closed_manifest: Some("original-manifest".into()),
    }
}
#[test]
fn exact_attempt_cursor_cannot_select_another_retry_source_or_allowance() {
    let mut request = request();
    let original = origin();
    let mut current = position(&request, &original).unwrap();
    current.batch = 1;
    current.offset = 1;
    current.source_generation = Some(19);
    current.source_sequence = 8;
    current.durable_sequence = Some(23);
    request.cursor = Some(AttemptObservationCursor {
        origin: original.clone(),
        maximum_events: request.maximum_events,
        position: current,
    });
    assert_eq!(position(&request, &original).unwrap().source_sequence, 8);
    for changed in [
        AttemptObservationOrigin {
            issuing_store: "another-store".into(),
            ..original.clone()
        },
        AttemptObservationOrigin {
            attempt: "retry-attempt".into(),
            ..original.clone()
        },
        AttemptObservationOrigin {
            generation: 4,
            ..original.clone()
        },
        AttemptObservationOrigin {
            source_revision: "another-source".into(),
            ..original.clone()
        },
        AttemptObservationOrigin {
            closed_manifest: Some("replacement-manifest".into()),
            ..original.clone()
        },
    ] {
        assert!(position(&request, &changed).is_err());
    }
    request.cursor.as_mut().unwrap().position.history_complete = true;
    assert!(
        position(&request, &original).is_err(),
        "the reader never issues a continuation after native exhaustion; decoded input cannot assert it"
    );
    request.cursor.as_mut().unwrap().position.history_complete = false;
    request.maximum_events += 1;
    assert!(position(&request, &original).is_err());
}
#[test]
fn original_active_attempt_can_close_without_inventing_live_numeric_samples() {
    let mut request = request();
    let mut active = origin();
    active.phase = CanonicalAttemptPhase::Ingesting;
    active.outcome = None;
    active.closed_manifest = None;
    let original = position(&request, &active).unwrap();
    assert!(original.source_generation.is_none());
    assert_eq!(original.source_sequence, 0);
    assert!(!original.history_complete);
    request.cursor = Some(AttemptObservationCursor {
        origin: active,
        maximum_events: request.maximum_events,
        position: original,
    });
    let closed = position(&request, &origin()).unwrap();
    assert_eq!(closed.attempt_generation, 3);
    assert_eq!(closed.batch, 0);
    assert_eq!(closed.offset, 0);
    assert!(
        AttemptObservationRequest {
            maximum_events: 0,
            ..request
        }
        .validate()
        .is_err()
    );
}
#[test]
fn source_origin_retains_native_terminal_failure_and_rejects_unknown_phase() {
    let native = |phase: &str| pse_operations::canonical_execution::CanonicalResultOrigin {
        key: "original-attempt".into(),
        run: "original-run".into(),
        source_revision: "original-revision".into(),
        generation: 3,
        phase: phase.into(),
        outcome: Some("failed".into()),
        closed_manifest: Some("original-manifest".into()),
    };
    let original = super::origin(native("terminal"), "original-store".into()).unwrap();
    assert!(original.phase.terminal());
    assert_eq!(original.outcome.as_deref(), Some("failed"));
    assert!(super::origin(native("not-a-native-phase"), "original-store".into()).is_err());
}
