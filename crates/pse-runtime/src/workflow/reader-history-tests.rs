// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One owned canonical fixture challenges the product reader's actual retained-data boundary.
#![allow(
    unsafe_code,
    reason = "isolated native progress fixture asserts an original failed effect-free completion and transient retry under the original seal/observation contracts"
)]
use super::*;
use crate::tooling::reader::{self, CanonicalReadSelection, Protocol, ReaderRequest, Request};
use pse_backend_native::solve::{Event, EventSource, Progress};
use pse_model::{
    generated::enums::{AttemptState, StudyPointState},
    study::{EffectState, OccurrenceKey, PointOutcome, RetryFailure},
};
use pse_operations::{
    canonical_execution::{ResultPayload, SeedEligibility, TerminalClass},
    canonical_studies::{point_facts, point_key},
};
use std::num::NonZeroUsize;

type FixtureResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn query(
    correlation: &str,
    request: Request,
    expected: i32,
) -> FixtureResult<serde_json::Value> {
    let request = serde_json::to_vec(&ReaderRequest {
        protocol: Protocol::ProductReader,
        version: pse_model::document::Version,
        correlation: correlation.into(),
        request,
    })?;
    let (status, frame) = tokio::task::spawn_blocking(move || {
        let mut frame = Vec::new();
        let status = reader::serve(request.as_slice(), &mut frame)?;
        Ok::<_, reader::ReaderError>((status, frame))
    })
    .await??;
    let response: serde_json::Value = serde_json::from_slice(&frame)?;
    assert_eq!(status, expected, "{response}");
    assert_eq!(response["correlation"], correlation);
    Ok(response)
}
fn selection(
    store: &pse_operations::canonical::CanonicalStore,
) -> FixtureResult<CanonicalReadSelection> {
    Ok(CanonicalReadSelection {
        state: store.deployment_state().to_owned(),
        database: Some(store.database().into()),
        memory_bytes: NonZeroUsize::new(256 << 20).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "reader fixture allocation must be nonzero",
            )
        })?,
        timeout_ms: 25_000,
    })
}
#[tokio::test]
async fn product_retained_study_and_attempt_pages_preserve_original_history_and_join_read_custody()
{
    // Only this producer fixture constructs a Runtime. The actual product read path owns
    // its independent math-free EngineFactory, configured clients and joined drain.
    let runtime = crate::workflow::durable_tests::durable_runtime();
    let physical = crate::workflow::study_workbench::canonical_tests::physical(&runtime).await;
    let store = runtime.canonical_store().unwrap();
    let original = store
        .edit("reader-history-source", None, "source", &[])
        .await
        .unwrap();
    crate::workflow::study_workbench::canonical_tests::create_with_attempt_limit(
        &runtime,
        "reader-history",
        &original,
        &physical,
        1,
        2,
    )
    .await;
    let scope = store
        .study_scope(&point_key("reader-history", OccurrenceKey(0)))
        .await
        .unwrap();
    let mut claim = store
        .claim_study_point(
            &scope,
            None,
            "reader-history-claim",
            "fixture-worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    let canonical = selection(store).unwrap();
    let selected_study = StudyObservationRequest {
        study: "reader-history".into(),
        maximum_members: 1,
        maximum_events: 1,
    };
    let active = query(
        "study-active-one",
        Request::Study {
            canonical: canonical.clone(),
            request: selected_study.clone(),
            cursor: None,
        },
        0,
    )
    .await
    .unwrap();
    let active_page = &active["payload"]["page"];
    assert_eq!(active_page["status"]["header"]["study"], "reader-history");
    assert!(active_page["sources"].as_array().unwrap().iter().any(
        |source| source["position"]["attempt"] == claim.fence.attempt()
            && source["continuity"] == "live_numerics_unavailable"
            && source["checkpoint"].is_null()
            && source["events"].as_array().unwrap().is_empty()
    ));
    let active_cursor = serde_json::from_value(active_page["cursor"].clone()).unwrap();
    let attempt = crate::workflow::AttemptObservationRequest {
        run: claim.point.run.clone(),
        attempt: claim.fence.attempt().into(),
        maximum_events: 1,
        cursor: None,
    };
    let live = query(
        "attempt-active",
        Request::Attempt {
            canonical: canonical.clone(),
            request: attempt.clone(),
        },
        0,
    )
    .await
    .unwrap();
    assert_eq!(
        live["payload"]["page"]["source"]["continuity"],
        "live_numerics_unavailable"
    );
    assert!(live["payload"]["page"]["origin"]["outcome"].is_null());
    claim.point = store.mark_study_started(&claim).await.unwrap();
    let progress = Progress::new(8);
    let mut events = Vec::new();
    for ordinal in 1..=3 {
        progress.push(Event {
            source: Some(EventSource::new("reader-fixture", None, None)),
            phase: "original retained phase".into(),
            elapsed: Duration::from_millis(ordinal),
            values: BTreeMap::from([(
                "iteration".into(),
                pse_backend_native::solve::Metric::Integer(ordinal as i64),
            )]),
            incumbent: None,
        });
        let mut event = ProgressEventDocument::from(progress.current().unwrap());
        event.sequence = Some(ordinal as i64);
        events.push(event);
    }
    store
        .append_result_batch(
            &claim.fence,
            "reader-history-progress",
            "__progress",
            0,
            ResultPayload::new(serde_json::to_vec(&events).unwrap()).unwrap(),
            events.len() as u64,
        )
        .await
        .unwrap();
    let stored = crate::workflow::StoredCompletion {
        version: 3,
        attempt_id: pse_operations::mint_id(),
        completion: Some(Default::default()),
        termination: crate::workflow::TerminationDetail {
            version: pse_model::document::Version,
            cause: crate::workflow::TerminationCause::Assessment {
                usable: false,
                candidate_use: vec![],
                diagnostics: vec![],
            },
            point: None,
            retry_failure: None,
            effect: EffectState::Absent,
        },
        state: AttemptState::Failed,
        progress: crate::workflow::ProgressCoverage::Sampled {
            observed: 3,
            queued: 3,
            dropped_before_queue: 0,
            acknowledged: 3,
            unacknowledged: 0,
            writer: crate::workflow::ProgressWriterOutcome::Complete,
        },
    };
    let receipt=serde_json::to_vec(&serde_json::json!({"version":1,"batch_count":0,"payload_bytes":0,"digest":"","inline":stored})).unwrap();
    let closed = store
        .close_result_ingestion(&claim.fence, "reader-history-close")
        .await
        .unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: this isolated fixture owns actual retained progress and explicitly asserts
    // a failed effect-free completion. A successful read never promotes that outcome.
    unsafe {
        store
            .seal_attempt(
                &manifest,
                "reader-history-seal",
                TerminalClass::Failed,
                &receipt,
                SeedEligibility::Unavailable,
            )
            .await
    }
    .unwrap();
    let first = query(
        "attempt-reader-one",
        Request::Attempt {
            canonical: canonical.clone(),
            request: attempt.clone(),
        },
        0,
    )
    .await
    .unwrap();
    let first_page = &first["payload"]["page"];
    assert_eq!(first_page["origin"]["run"], claim.point.run);
    assert_eq!(first_page["origin"]["attempt"], claim.fence.attempt());
    assert_eq!(first_page["origin"]["outcome"], "failed");
    assert_eq!(
        first_page["source"]["events"][0],
        serde_json::to_value(&events[0]).unwrap()
    );
    let mut facts = point_facts(&claim.point).unwrap();
    facts.lifecycle = StudyPointState::Failed;
    facts.retry_failure = Some(RetryFailure::Transient);
    facts.effect = EffectState::Absent;
    let outcome = PointOutcome {
        key: facts.key,
        lifecycle: facts.lifecycle,
        scientific: facts.scientific.clone(),
        diagnostic: None,
        start: claim
            .point
            .start
            .as_ref()
            .map(|bytes| serde_json::from_slice(bytes.as_slice()).unwrap()),
        effect: facts.effect,
        attempts: vec![],
    };
    // SAFETY: the original failed terminal receipt above backs the exact effect-free
    // transient observation; the admitted fixture policy allows a second attempt.
    unsafe {
        store
            .observe_study_point(&claim.point, &facts, &outcome, false)
            .await
    }
    .unwrap();
    let retry_scope = store.study_scope(&claim.point.key).await.unwrap();
    let retry = store
        .claim_study_point(
            &retry_scope,
            None,
            "reader-history-retry",
            "fixture-worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    assert_ne!(retry.fence.attempt(), claim.fence.attempt());
    assert!(retry.fence.generation() > claim.fence.generation());
    let mut next: Option<crate::workflow::AttemptObservationCursor> =
        serde_json::from_value(first_page["next"].clone()).unwrap();
    let mut observed = vec![1_u64];
    for correlation in [
        "attempt-reader-two",
        "attempt-reader-three",
        "attempt-reader-end",
    ] {
        let response = query(
            correlation,
            Request::Attempt {
                canonical: canonical.clone(),
                request: crate::workflow::AttemptObservationRequest {
                    cursor: next.take(),
                    ..attempt.clone()
                },
            },
            0,
        )
        .await
        .unwrap();
        let page = &response["payload"]["page"];
        observed.extend(
            page["source"]["events"]
                .as_array()
                .unwrap()
                .iter()
                .map(|event| event["source"]["sequence"].as_u64().unwrap()),
        );
        next = serde_json::from_value(page["next"].clone()).unwrap();
        assert_eq!(page["origin"]["outcome"], "failed");
    }
    assert_eq!(observed, vec![1, 2, 3]);
    assert!(next.is_none());
    let reconnected = query(
        "study-reconnected",
        Request::Study {
            canonical: canonical.clone(),
            request: selected_study,
            cursor: Some(active_cursor),
        },
        0,
    )
    .await
    .unwrap();
    let sources = reconnected["payload"]["page"]["sources"]
        .as_array()
        .unwrap();
    let old = sources
        .iter()
        .find(|source| source["position"]["attempt"] == claim.fence.attempt())
        .unwrap();
    let new = sources
        .iter()
        .find(|source| source["position"]["attempt"] == retry.fence.attempt())
        .unwrap();
    assert_eq!(old["events"][0], serde_json::to_value(&events[0]).unwrap());
    assert_eq!(new["continuity"], "live_numerics_unavailable");
    assert_ne!(
        old["position"]["attempt_generation"],
        new["position"]["attempt_generation"]
    );
    let mut changed: crate::workflow::AttemptObservationCursor =
        serde_json::from_value(first_page["next"].clone()).unwrap();
    changed.origin.attempt = retry.fence.attempt().into();
    let refused = query(
        "wrong-retry",
        Request::Attempt {
            canonical,
            request: crate::workflow::AttemptObservationRequest {
                cursor: Some(changed),
                ..attempt
            },
        },
        2,
    )
    .await
    .unwrap();
    assert_eq!(refused["payload"]["kind"], "refusal");
    // Successful fixture removal follows independently drained reader connections,
    // not a dropped page or a command status taken as custody settlement evidence.
    store.remove_isolated_fixture().await.unwrap();
}
