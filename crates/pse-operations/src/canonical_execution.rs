// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fenced durable execution and immutable closed result selections.

use crate::{
    canonical::{CanonicalError, CanonicalStore, REQUEST_TIMEOUT, bounded_query},
    canonical_codec,
    generated::surreal as wire,
};
use pse_ids::{Frame, FramedHasher};
pub use pse_model::generated::runtime::{
    canonical_attempts::Row as CanonicalAttempt, canonical_result_batches::Row as ResultBatch,
    canonical_result_manifests::Row as ResultManifest, canonical_runs::Row as CanonicalRun,
};
use pse_model::generated::runtime::{
    canonical_result_block_outputs::Row as ResultOutput,
    canonical_result_blocks::Row as ResultBlock, canonical_result_cells::Row as ResultCell,
    canonical_result_sets::Row as ResultSet,
};
use serde::{Deserialize, Serialize};
use std::{future::IntoFuture, time::Duration};
use surrealdb::types::{Bytes, Object, Value};

/// Independently submitted scientific block limit, below the protocol envelope.
pub const RESULT_BATCH_BYTES: usize = wire::RESULT_BLOCK_BYTES;
/// Completion and closed descriptors are bounded metadata, never trajectories.
pub const EXECUTION_METADATA_BYTES: usize = wire::RESULT_INDEX_BYTES;
const RESULT_SETS: usize = 256;

/// Conservative native CBOR extent for the canonical codec's metadata values.
/// Decimal uses a fixed bound covering its tag and the longest signed decimal text;
/// this is independent of protobuf framing and live allocation ownership.
pub fn result_metadata_extent(value: &Value) -> Result<usize, CanonicalError> {
    fn head(length: usize) -> usize {
        if length < 24 {
            1
        } else if length <= 255 {
            2
        } else if length <= 65535 {
            3
        } else if length <= u32::MAX as usize {
            5
        } else {
            9
        }
    }
    let add = |a: usize, b: usize| a.checked_add(b).ok_or(CanonicalError::PayloadLimit);
    match value {
        Value::None | Value::Null | Value::Bool(_) => Ok(4),
        Value::Number(surrealdb::types::Number::Decimal(_)) => Ok(40),
        Value::Number(_) => Ok(9),
        Value::String(v) => add(head(v.len()), v.len()),
        Value::Bytes(v) => add(head(v.len()), v.len()),
        Value::Array(v) => v.iter().try_fold(head(v.len()), |sum, value| {
            add(sum, result_metadata_extent(value)?)
        }),
        Value::Object(v) => v.iter().try_fold(head(v.len()), |sum, (key, value)| {
            add(
                add(sum, add(head(key.len()), key.len())?)?,
                result_metadata_extent(value)?,
            )
        }),
        _ => Err(CanonicalError::PayloadLimit),
    }
}

/// Derive encoded scalar metadata extent through the canonical codec owner.
pub fn result_cell_metadata_extent(cell: &ResultCell) -> Result<usize, CanonicalError> {
    result_metadata_extent(&Value::Object(wire::encode_canonical_result_cells(cell)?))
}

/// Exact selected scientific input and executable provenance, before native work.
#[derive(Clone, Debug)]
pub struct RunRequest {
    /// Stable semantic run and begin-effect identity.
    pub key: String,
    /// Immutable source receipt; server rechecks its actual meaning.
    pub revision: crate::canonical::Revision,
    /// Additional exact retained inputs, including selected physical definitions.
    pub sources: Vec<crate::canonical::Revision>,
    /// Scientific demand and resolved configuration, encoded by its owner.
    pub request: Vec<u8>,
    /// Selected compilation and provider interpretation, encoded by its owner.
    pub source_selection: Vec<u8>,
    /// Outer executable provenance, encoded by its owner.
    pub attestation: Vec<u8>,
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    unsafe_code,
    reason = "controlled fixture scientific admission asserts outcomes without pointer or ABI operations"
)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "isolated execution fixtures and assertions fail the test on unexpected results"
)]
mod canonical_execution_server_unit {
    use super::*;
    use crate::canonical::{CanonicalOptions, checked};
    use std::path::Path;

    async fn fixture() -> (CanonicalStore, String, crate::canonical::Revision) {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_execution_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let revision = store
            .edit("problem", None, "initial-source", &[])
            .await
            .unwrap();
        (store, options.database, revision)
    }
    async fn remove(store: &CanonicalStore, database: &str) {
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }
    async fn run(
        store: &CanonicalStore,
        revision: &crate::canonical::Revision,
        key: &str,
    ) -> CanonicalRun {
        store
            .begin_run(&RunRequest {
                key: key.into(),
                revision: revision.clone(),
                sources: vec![],
                request: vec![1, 2, 3],
                source_selection: vec![4],
                attestation: vec![5],
            })
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn canonical_execution_native_metadata_bound_matches_conservative_client() {
        let (store, database, _) = fixture().await;
        let mut minimum = Object::new();
        minimum.insert("key", Value::String("a".into()));
        let mut values = vec![
            Value::Object(minimum),
            Value::None,
            canonical_codec::encode_uint(u64::MAX).unwrap(),
        ];
        for length in [23, 24, 255, 256, 65535, 65536] {
            values.push(Value::String("x".repeat(length)));
        }
        for (index, value) in values.into_iter().enumerate() {
            let mut result = bounded_query(
                store
                    .db
                    .query("RETURN bytes::len(encoding::cbor::encode($value));")
                    .bind(("value", value.clone())),
            )
            .await
            .unwrap();
            let native = canonical_codec::decode_int(result.take::<Value>(0).unwrap()).unwrap();
            assert!(native as usize <= result_metadata_extent(&value).unwrap());
            if index == 0 {
                assert_eq!(native, 7, "minimum valid canonical index key record");
            }
        }
        remove(&store, &database).await;
    }
    #[tokio::test]
    async fn canonical_execution_large_request_has_narrow_ordered_summary() {
        let (store, database, revision) = fixture().await;
        let request = RunRequest {
            key: "large-request".into(),
            revision,
            sources: vec![],
            request: vec![1; 256 * 1024],
            source_selection: vec![2; 256 * 1024],
            attestation: vec![3],
        };
        let row = store.begin_run(&request).await.unwrap();
        assert_eq!(row.request.as_slice(), request.request);
        let summary = store
            .execution_run_page(&row.problem, None, 64)
            .await
            .unwrap();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].key, row.key);
        assert_eq!(summary[0].sequence, row.sequence);
        assert!(summary[0].current_attempt.is_none() && summary[0].terminal_class.is_none());
        assert!(
            store
                .execution_run_page(&row.problem, Some(row.sequence), 64)
                .await
                .unwrap()
                .is_empty()
        );
        let mut changed = request;
        changed.source_selection.push(3);
        assert!(store.begin_run(&changed).await.is_err());
        remove(&store, &database).await;
    }
    #[tokio::test]
    async fn canonical_execution_study_cancellation_revokes_live_and_closed_workers() {
        use crate::canonical_studies::{NewOccurrence, point_key};
        use pse_model::study::{OccurrenceKey, PointPolicy, SeedNeed, StartPolicy};
        let (store, database, revision) = fixture().await;
        let request = |key: &str| RunRequest {
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let points = (0..2)
            .map(|index| NewOccurrence {
                policy: PointPolicy {
                    key: OccurrenceKey(index),
                    dependencies: vec![],
                    start: StartPolicy::Fresh,
                    seed_need: SeedNeed::NotNeeded,
                    attempt_limit: 1,
                },
                descriptor: vec![42],
                run: request(&format!("study-point-{index}")),
            })
            .collect::<Vec<_>>();
        store
            .create_study(
                "cancel-study",
                &request("study-summary"),
                &[9],
                &points,
                &|| false,
            )
            .await
            .unwrap();
        let mut claims = Vec::new();
        for index in 0..2 {
            let scope = store
                .study_scope(&point_key("cancel-study", OccurrenceKey(index)))
                .await
                .unwrap();
            claims.push(
                store
                    .claim_study_point(
                        &scope,
                        None,
                        &format!("claim-{index}"),
                        "worker",
                        Duration::from_secs(60),
                    )
                    .await
                    .unwrap(),
            );
        }
        let first = &claims[0].fence;
        store
            .append_result_batch(first, "before-cancel", "observations", 0, &[7], 1)
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(first, "before-cancel-close")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        store.cancel_study("cancel-study").await.unwrap();
        assert!(
            store
                .renew_attempt(first, Duration::from_secs(60))
                .await
                .is_err()
        );
        assert!(
            // SAFETY: the isolated fixture owns these synthetic observations; this cancelled stale capability must refuse success.
            unsafe {
                store
                    .seal_attempt(&manifest, "stale-success", TerminalClass::Succeeded, &[9])
                    .await
            }
            .is_err()
        );
        assert!(
            // SAFETY: the fixture supplies its actual cancellation; this stale capability must still be refused.
            unsafe {
                store
                    .seal_attempt(&manifest, "stale-cancel", TerminalClass::Cancelled, &[9])
                    .await
            }
            .is_err()
        );
        for (index, claim) in claims.iter().enumerate() {
            assert!(
                store
                    .append_result_batch(
                        &claim.fence,
                        &format!("late-{index}"),
                        "observations",
                        1,
                        &[8],
                        1
                    )
                    .await
                    .is_err()
            );
            let recovery = store
                .recover_closed_attempt(claim.fence.run(), &format!("recover-{index}"))
                .await
                .unwrap();
            assert!(
                store
                    .canonical_run(claim.fence.run())
                    .await
                    .unwrap()
                    .unwrap()
                    .cancelled
            );
            let manifest = store.reconcile_closed_attempt(&recovery).await.unwrap();
            // SAFETY: the fixture owns the reconciled synthetic observations and supplies their actual cancellation.
            let attempt = unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        &format!("cancel-terminal-{index}"),
                        TerminalClass::Cancelled,
                        &[9],
                    )
                    .await
            }
            .unwrap();
            assert_eq!(attempt.outcome.as_deref(), Some("cancelled"));
            assert!(
                store
                    .claim_run(
                        claim.fence.run(),
                        &format!("reclaim-{index}"),
                        "worker",
                        Duration::from_secs(60)
                    )
                    .await
                    .is_err()
            );
        }
        remove(&store, &database).await;
    }
    async fn finish(
        store: &CanonicalStore,
        fence: &AttemptFence,
        operation: &str,
        outcome: TerminalClass,
    ) -> CanonicalAttempt {
        let closed = store
            .close_result_ingestion(fence, &format!("close-{operation}"))
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture writer owns the exact reconciled observations and supplies the test-selected terminal class.
        unsafe {
            store
                .seal_attempt(&manifest, operation, outcome, &[9])
                .await
        }
        .unwrap()
    }

    #[tokio::test]
    async fn canonical_execution_exact_batches_private_closure_and_settlement() {
        let (store, database, revision) = fixture().await;
        let first = run(&store, &revision, "run").await;
        let repeat = run(&store, &revision, "run").await;
        assert_eq!(first, repeat);
        let fence = store
            .claim_run("run", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let replay = store
            .claim_run("run", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(fence.attempt(), replay.attempt());
        let batch = store
            .append_result_batch(&fence, "batch-0", "values", 0, &[1, 2], 2)
            .await
            .unwrap();
        assert_eq!(
            batch,
            store
                .append_result_batch(&fence, "batch-0", "values", 0, &[1, 2], 2)
                .await
                .unwrap()
        );
        assert!(
            store
                .append_result_batch(&fence, "batch-0", "values", 0, &[1, 3], 2)
                .await
                .is_err()
        );
        assert!(
            store
                .append_result_batch(&fence, "changed-batch", "values", 0, &[1, 3], 2)
                .await
                .is_err()
        );
        assert!(
            store
                .append_result_batch(&fence, "gap", "values", 2, &[4], 1)
                .await
                .is_err()
        );
        assert!(
            store
                .read_results("run", fence.attempt(), Duration::from_secs(30))
                .await
                .is_err()
        );
        let close = store.close_result_ingestion(&fence, "close").await.unwrap();
        assert!(
            store
                .append_result_batch(&fence, "late", "values", 1, &[5], 1)
                .await
                .is_err()
        );
        let manifest = store.reconcile_closed_attempt(&close).await.unwrap();
        let descriptors = decode_result_descriptors(manifest.row()).unwrap();
        assert_eq!(descriptors[0].batch_count, 1);
        assert_eq!(descriptors[0].row_count, 2);
        // SAFETY: the fixture admits exactly its two synthetic values as partial with its fixed completion.
        let terminal = unsafe {
            store
                .seal_attempt(&manifest, "seal", TerminalClass::Partial, &[8])
                .await
        }
        .unwrap();
        assert_eq!(
            terminal,
            // SAFETY: this is an exact replay of the fixture-owned partial observations and completion.
            unsafe {
                store
                    .seal_attempt(&manifest, "seal", TerminalClass::Partial, &[8])
                    .await
            }
            .unwrap()
        );
        assert!(
            // SAFETY: the fixture owns these synthetic observations; changing its admitted class must be refused.
            unsafe {
                store
                    .seal_attempt(&manifest, "seal", TerminalClass::Succeeded, &[8])
                    .await
            }
            .is_err()
        );
        assert!(
            // SAFETY: the fixture owns these synthetic observations; changing its admitted completion must be refused.
            unsafe {
                store
                    .seal_attempt(&manifest, "seal", TerminalClass::Partial, &[9])
                    .await
            }
            .is_err()
        );
        let reconnect = store
            .resume_closed_attempt(fence.attempt(), "close")
            .await
            .unwrap();
        assert_eq!(
            store
                .reconcile_closed_attempt(&reconnect)
                .await
                .unwrap()
                .row(),
            manifest.row()
        );
        let history = store
            .read_results("run", fence.attempt(), Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(history.attempt().outcome.as_deref(), Some("partial"));
        let fresh = store
            .claim_run("run", "new-claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        assert!(fresh.generation() > fence.generation());
        assert!(
            store
                .append_result_batch(&fence, "stale", "values", 1, &[5], 1)
                .await
                .is_err()
        );
        finish(&store, &fresh, "succeeded", TerminalClass::Succeeded).await;
        assert!(
            store
                .claim_run("run", "repeat-science", "worker", Duration::from_secs(60))
                .await
                .is_err()
        );
        assert!(
            store
                .read_results("run", fence.attempt(), Duration::from_secs(30))
                .await
                .is_ok()
        );
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_sixteen_retention_roundtrips_overlap_source_staging_and_private_pins()
     {
        use std::sync::Arc;
        let (store, database, revision) = fixture().await;
        let barrier = Arc::new(tokio::sync::Barrier::new(18));
        let mut jobs = tokio::task::JoinSet::new();
        let writer = store.clone();
        let writer_barrier = barrier.clone();
        jobs.spawn(async move {
            writer_barrier.wait().await;
            let payload = vec![37; crate::canonical_staging::SOURCE_BLOCK_BYTES + 17];
            writer
                .edit(
                    "problem",
                    Some("initial-source"),
                    "concurrent-source",
                    &[crate::canonical::ObjectEdit {
                        logical: "unconsumed".into(),
                        scope: "root".into(),
                        name: "Other".into(),
                        references: vec![],
                        version: Some(crate::canonical::ObjectVersion {
                            key: "other-version".into(),
                            logical: "unconsumed".into(),
                            kind: "test".into(),
                            payload: payload.into(),
                            interpretation: wire::INTERPRETATION.into(),
                        }),
                    }],
                )
                .await
                .unwrap();
        });
        let reader = store.clone();
        let reader_barrier = barrier.clone();
        let old_revision = revision.clone();
        jobs.spawn(async move {
            reader_barrier.wait().await;
            for _ in 0..16 {
                let protection = reader
                    .protect(old_revision.clone(), Duration::from_secs(60))
                    .await
                    .unwrap();
                let mut selected = crate::canonical_selection::SelectedRead::new(protection);
                assert!(
                    reader
                        .resolve_logicals(&mut selected, &["absent".into()])
                        .await
                        .unwrap()
                        .is_empty()
                );
                reader.release(selected.selection()).await.unwrap();
            }
        });
        for index in 0..16 {
            let store = store.clone();
            let revision = revision.clone();
            let barrier = barrier.clone();
            jobs.spawn(async move {
                barrier.wait().await;
                let key = format!("concurrent-run-{index}");
                let saved = run(&store, &revision, &key).await;
                assert_eq!(saved.revision, revision.key);
                let fence = store
                    .claim_run(
                        &key,
                        &format!("concurrent-claim-{index}"),
                        "worker",
                        Duration::from_secs(60),
                    )
                    .await
                    .unwrap();
                store
                    .renew_attempt(&fence, Duration::from_secs(60))
                    .await
                    .unwrap();
                // These original physical observations belong to the synthetic
                // scientific writer. The independently specified Celsius facts
                // below check their retained meaning, as well as exact bytes.
                let kelvin = [273.15_f64, 298.15, 333.15, 373.15];
                let original = kelvin
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect::<Vec<_>>();
                store
                    .append_result_batch(
                        &fence,
                        &format!("concurrent-batch-{index}"),
                        "temperature_kelvin",
                        0,
                        &original,
                        4,
                    )
                    .await
                    .unwrap();
                let closed = store
                    .close_result_ingestion(&fence, &format!("concurrent-close-{index}"))
                    .await
                    .unwrap();
                let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
                let descriptors = decode_result_descriptors(manifest.row()).unwrap();
                assert_eq!(descriptors.len(), 1);
                assert_eq!(descriptors[0].batch_count, 1);
                assert_eq!(descriptors[0].row_count, 4);
                // SAFETY: this synthetic writer owns these four original physical observations and their specified completion.
                let terminal = unsafe {
                    store
                        .seal_attempt(
                            &manifest,
                            &format!("concurrent-seal-{index}"),
                            TerminalClass::Succeeded,
                            b"four original temperatures",
                        )
                        .await
                }
                .unwrap();
                assert_eq!(terminal.outcome.as_deref(), Some("succeeded"));
                let read = store
                    .read_results(&key, fence.attempt(), Duration::from_secs(60))
                    .await
                    .unwrap();
                assert_eq!(read.run().revision, revision.key);
                let payload = store
                    .result_payload(&read, &descriptors[0].key, 0)
                    .await
                    .unwrap();
                assert_eq!(payload.batch.payload.as_slice(), original);
                assert_eq!(payload.batch.row_count, 4);
                for (encoded, celsius) in payload
                    .batch
                    .payload
                    .chunks_exact(8)
                    .zip([0.0, 25.0, 60.0, 100.0])
                {
                    let actual = f64::from_le_bytes(encoded.try_into().unwrap());
                    assert!((actual - 273.15 - celsius).abs() < 1e-12);
                }
                assert!(
                    store
                        .append_result_batch(
                            &fence,
                            &format!("concurrent-late-{index}"),
                            "temperature_kelvin",
                            1,
                            &[0],
                            1
                        )
                        .await
                        .is_err()
                );
                drop(payload);
                drop(read);
            });
        }
        while let Some(result) = jobs.join_next().await {
            result.unwrap();
        }
        store.result_read_drain.drain().await.unwrap();
        assert_eq!(
            store
                .execution_run_page("problem", None, 64)
                .await
                .unwrap()
                .len(),
            16
        );
        assert_eq!(
            store
                .revision("concurrent-source")
                .await
                .unwrap()
                .unwrap()
                .sequence,
            revision.sequence + 1
        );
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_claim_race_and_close_write_race() {
        let (store, database, revision) = fixture().await;
        for index in 0..8 {
            let key = format!("race-{index}");
            run(&store, &revision, &key).await;
            let left = format!("left-{index}");
            let right = format!("right-{index}");
            let (left, right) = tokio::join!(
                store.claim_run(&key, &left, "worker", Duration::from_secs(60)),
                store.claim_run(&key, &right, "worker", Duration::from_secs(60))
            );
            assert_ne!(left.is_ok(), right.is_ok());
            let fence = left.or(right).unwrap();
            let operation = format!("append-{index}");
            let close = format!("close-{index}");
            let payload = [index];
            let (batch, closed) = tokio::join!(
                store.append_result_batch(&fence, &operation, "values", 0, &payload, 1),
                store.close_result_ingestion(&fence, &close)
            );
            let closed = closed.unwrap();
            let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
            let descriptors = decode_result_descriptors(manifest.row()).unwrap();
            assert_eq!(descriptors.len(), usize::from(batch.is_ok()));
            if batch.is_ok() {
                assert_eq!(descriptors[0].batch_count, 1);
            }
            assert!(
                store
                    .append_result_batch(&fence, &format!("late-{index}"), "values", 1, &[index], 1)
                    .await
                    .is_err()
            );
        }
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_expiry_cancellation_and_truthful_recovery() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "expires").await;
        let fence = store
            .claim_run(
                "expires",
                "expiry-claim",
                "worker",
                Duration::from_millis(250),
            )
            .await
            .unwrap();
        store
            .append_result_batch(&fence, "first", "diagnostics", 0, &[7], 1)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(
            store
                .append_result_batch(&fence, "stale", "diagnostics", 1, &[8], 1)
                .await
                .is_err()
        );
        assert!(
            store
                .close_result_ingestion(&fence, "stale-close")
                .await
                .is_err()
        );
        let closed = store
            .recover_closed_attempt("expires", "recover")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        assert!(
            // SAFETY: the fixture owns the expired synthetic attempt; recovered failure observations must refuse success.
            unsafe {
                store
                    .seal_attempt(&manifest, "false-success", TerminalClass::Succeeded, &[])
                    .await
            }
            .is_err()
        );
        // SAFETY: the fixture owns the recovered diagnostic observation and admits its actual failed outcome.
        let failed = unsafe {
            store
                .seal_attempt(&manifest, "failed", TerminalClass::Failed, &[7])
                .await
        }
        .unwrap();
        assert_eq!(failed.outcome.as_deref(), Some("failed"));
        assert_eq!(
            decode_result_descriptors(manifest.row()).unwrap()[0].row_count,
            1
        );
        run(&store, &revision, "cancel").await;
        let fence = store
            .claim_run("cancel", "cancel-claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        store
            .append_result_batch(&fence, "cancel-first", "values", 0, &[1], 1)
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&fence, "before-cancel")
            .await
            .unwrap();
        let stale_manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        store
            .cancel_run("cancel", "cancel-operation")
            .await
            .unwrap();
        assert!(
            // SAFETY: the fixture owns these synthetic observations; cancellation must refuse this stale success candidate.
            unsafe {
                store
                    .seal_attempt(&stale_manifest, "stale-seal", TerminalClass::Succeeded, &[])
                    .await
            }
            .is_err()
        );
        let recovered = store
            .recover_closed_attempt("cancel", "cancel-recovery")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&recovered).await.unwrap();
        assert!(
            // SAFETY: the fixture owns the recovered attempt; its actual cancellation must refuse a failed candidate.
            unsafe {
                store
                    .seal_attempt(&manifest, "ignored-cancel", TerminalClass::Failed, &[])
                    .await
            }
            .is_err()
        );
        // SAFETY: the fixture owns the reconciled synthetic observation and admits its actual cancellation.
        let terminal = unsafe {
            store
                .seal_attempt(&manifest, "cancel-seal", TerminalClass::Cancelled, &[1])
                .await
        }
        .unwrap();
        assert_eq!(terminal.outcome.as_deref(), Some("cancelled"));
        assert!(
            store
                .claim_run(
                    "cancel",
                    "new-cancel-attempt",
                    "worker",
                    Duration::from_secs(60)
                )
                .await
                .is_err()
        );
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_named_absence_guard_and_ordered_decimal_sequence() {
        let (store, database, revision) = fixture().await;
        let request = RunRequest {
            key: "same".into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let different = RunRequest {
            request: vec![9],
            ..request.clone()
        };
        let (left, right) = tokio::join!(store.begin_run(&request), store.begin_run(&different));
        assert_ne!(left.is_ok(), right.is_ok());
        let mut result=bounded_query(store.db.query("UPDATE type::record('canonical_guards', 'execution-sequence:problem') SET generation=$sequence;").bind(("sequence",canonical_codec::encode_uint(1_u64<<63).unwrap()))).await.unwrap();
        let _: Vec<Object> = result.take(0).unwrap();
        let second = run(&store, &revision, "ordered").await;
        assert_eq!(second.sequence, (1_u64 << 63) + 1);
        let mut roots =
            bounded_query(store.db.query(
                "SELECT * FROM canonical_roots WHERE owner_kind='run' AND owner='ordered';",
            ))
            .await
            .unwrap();
        let roots: Vec<Object> = roots.take(0).unwrap();
        assert_eq!(roots.len(), 1);
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_missing_batch_refuses_descriptor_and_cancel_races_seal() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "missing").await;
        let fence = store
            .claim_run(
                "missing",
                "missing-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        store
            .append_result_batch(&fence, "missing-first", "values", 0, &[1], 1)
            .await
            .unwrap();
        let missing = store
            .append_result_batch(&fence, "missing-second", "values", 1, &[2], 1)
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&fence, "missing-close")
            .await
            .unwrap();
        bounded_query(
            store
                .db
                .query("DELETE ONLY type::record('canonical_result_batches',$key);")
                .bind(("key", missing.key)),
        )
        .await
        .unwrap();
        assert!(store.reconcile_closed_attempt(&closed).await.is_err());
        assert!(
            !store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap()
                .terminal
        );
        for index in 0..8 {
            let key = format!("seal-race-{index}");
            run(&store, &revision, &key).await;
            let fence = store
                .claim_run(
                    &key,
                    &format!("seal-claim-{index}"),
                    "worker",
                    Duration::from_secs(60),
                )
                .await
                .unwrap();
            let closed = store
                .close_result_ingestion(&fence, &format!("seal-close-{index}"))
                .await
                .unwrap();
            let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
            let seal = format!("seal-race-op-{index}");
            let cancel = format!("cancel-race-op-{index}");
            let (sealed, cancelled) = tokio::join!(
                // SAFETY: the fixture owns this empty synthetic success; cancellation races only its durable admission.
                unsafe { store.seal_attempt(&manifest, &seal, TerminalClass::Succeeded, &[]) },
                store.cancel_run(&key, &cancel)
            );
            assert_ne!(sealed.is_ok(), cancelled.is_ok());
            let actual = store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap();
            if sealed.is_ok() {
                assert_eq!(actual.outcome.as_deref(), Some("succeeded"));
            } else {
                assert!(!actual.terminal);
                let closed = store
                    .recover_closed_attempt(&key, &format!("race-recover-{index}"))
                    .await
                    .unwrap();
                let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
                // SAFETY: cancellation won the race and the fixture admits the exact recovered empty attempt as cancelled.
                let final_outcome = unsafe {
                    store
                        .seal_attempt(
                            &manifest,
                            &format!("race-cancelled-{index}"),
                            TerminalClass::Cancelled,
                            &[],
                        )
                        .await
                }
                .unwrap();
                assert_eq!(final_outcome.outcome.as_deref(), Some("cancelled"));
            }
        }
        remove(&store, &database).await;
    }
}

/// Worker admission capability; fields cannot be fabricated by an external caller.
#[derive(Clone, Debug)]
pub struct AttemptFence {
    run: String,
    attempt: String,
    generation: u64,
}
impl AttemptFence {
    pub(crate) fn from_row(row: CanonicalAttempt) -> Result<Self, CanonicalError> {
        if row.generation == 0 || row.terminal || row.closed {
            return Err(CanonicalError::Configuration(
                "native claim returned no live fence".into(),
            ));
        }
        Ok(Self {
            run: row.run,
            attempt: row.key,
            generation: row.generation,
        })
    }
    /// Semantic execution identity.
    pub fn run(&self) -> &str {
        &self.run
    }
    /// Actual native execution identity.
    pub fn attempt(&self) -> &str {
        &self.attempt
    }
    /// Server-issued generation, distinct from semantic identity.
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

/// Terminal class supplied only after the scientific owner determines quality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalClass {
    /// Qualified numerical completion under the scientific owner's admission.
    Succeeded,
    /// Failed execution with potentially useful retained diagnostics.
    Failed,
    /// Explicitly incomplete scientific coverage.
    Partial,
    /// Cancellation won admission, whether or not native work already stopped.
    Cancelled,
}
impl TerminalClass {
    /// Canonical terminal outcome spelling stored atomically with the manifest.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Partial => "partial",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Closed ingestion receipt. Recovery authority is separate from the old worker.
#[derive(Clone, Debug)]
pub struct ClosedAttempt {
    fence: AttemptFence,
    authority: u64,
}
impl ClosedAttempt {
    /// Original attempt; immutable coordinates keep its original generation.
    pub fn fence(&self) -> &AttemptFence {
        &self.fence
    }
}

/// Exact frozen coordinate extent of one scientific result set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultSetDescriptor {
    /// Immutable result-set coordinate identity.
    pub key: String,
    /// Scientific result-set name.
    pub name: String,
    /// All admitted ordinals are exactly 0..batch_count.
    pub batch_count: u64,
    /// Sum of admitted row coverage, independent of terminal quality.
    pub row_count: u64,
    /// Ordered framed digest of batch keys, payload digests and row coverage.
    pub batches_digest: String,
}

/// Only a reconciled immutable selection can authorize terminal admission.
#[derive(Clone, Debug)]
pub struct ClosedManifest {
    closed: ClosedAttempt,
    row: ResultManifest,
}
impl ClosedManifest {
    /// Exact immutable descriptor retained in the substrate.
    pub fn row(&self) -> &ResultManifest {
        &self.row
    }
}

fn hash(kind: &str, parts: &[&[u8]]) -> String {
    let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
    digest.str(kind);
    for part in parts {
        digest.part(part);
    }
    digest.finish_hash().to_hex()
}
/// Opaque native attempt identity known before claim and stable for its effect.
/// Scientific AttemptId lineage is derived independently and is never a native lookup key.
pub fn execution_attempt_key(run: &str, operation: &str) -> String {
    hash(
        "pse.execution.attempt.v1",
        &[run.as_bytes(), operation.as_bytes()],
    )
}
/// Exact result-set key derived without ambiguous textual concatenation.
pub fn result_set_key(attempt: &str, name: &str) -> String {
    hash(
        "pse.execution.result-set.v1",
        &[attempt.as_bytes(), name.as_bytes()],
    )
}
/// Exact immutable batch coordinate, shared by writer and admitted reader.
pub fn result_batch_key(attempt: &str, set: &str, ordinal: u64) -> String {
    hash(
        "pse.execution.result-batch.v1",
        &[attempt.as_bytes(), set.as_bytes(), &ordinal.to_le_bytes()],
    )
}
/// Scientific payload integrity digest; a digest alone does not establish admission.
pub fn result_payload_digest(payload: &[u8]) -> String {
    hash("pse.execution.result-payload.v1", &[payload])
}
/// Verify a bounded exact batch before decoding its scientific contents.
pub fn validate_result_batch(batch: &ResultBatch) -> Result<(), CanonicalError> {
    bounded(batch.payload.as_slice(), RESULT_BATCH_BYTES)?;
    if batch.key != result_batch_key(&batch.attempt, &batch.result_set, batch.ordinal)
        || batch.digest != result_payload_digest(batch.payload.as_slice())
    {
        return Err(CanonicalError::Configuration(
            "result batch integrity or coordinate mismatch".into(),
        ));
    }
    Ok(())
}
fn metadata_digest(payload: &[u8]) -> String {
    hash("pse.execution.result-manifest.v1", &[payload])
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json::to_vec(value).map_err(|error| CanonicalError::Configuration(error.to_string()))
}
fn identity(value: &str) -> Result<(), CanonicalError> {
    if value.is_empty() {
        return Err(CanonicalError::Configuration(
            "execution identity must be nonempty".into(),
        ));
    }
    if value.len() > 4096 {
        return Err(CanonicalError::PayloadLimit);
    }
    Ok(())
}
fn bounded(value: &[u8], limit: usize) -> Result<(), CanonicalError> {
    if value.len() > limit {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}
fn closed(row: CanonicalAttempt) -> Result<ClosedAttempt, CanonicalError> {
    if !row.closed || row.ingestion_open {
        return Err(CanonicalError::Configuration(
            "attempt ingestion is not closed".into(),
        ));
    }
    let authority = row
        .close_generation
        .ok_or_else(|| CanonicalError::Configuration("closed authority missing".into()))?;
    Ok(ClosedAttempt {
        fence: AttemptFence {
            run: row.run,
            attempt: row.key,
            generation: row.generation,
        },
        authority,
    })
}

/// Decode and verify the one canonical exact-selection descriptor format.
pub fn decode_result_descriptors(
    row: &ResultManifest,
) -> Result<Vec<ResultSetDescriptor>, CanonicalError> {
    bounded(row.descriptors.as_slice(), EXECUTION_METADATA_BYTES)?;
    if row.key != row.attempt || metadata_digest(row.descriptors.as_slice()) != row.digest {
        return Err(CanonicalError::Configuration(
            "result manifest integrity mismatch".into(),
        ));
    }
    let descriptors: Vec<ResultSetDescriptor> = serde_json::from_slice(row.descriptors.as_slice())
        .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
    if descriptors.len() > RESULT_SETS {
        return Err(CanonicalError::PayloadLimit);
    }
    let mut previous: Option<&str> = None;
    for item in &descriptors {
        if item.key != result_set_key(&row.attempt, &item.name)
            || previous.is_some_and(|key| key >= item.key.as_str())
        {
            return Err(CanonicalError::Configuration(
                "result descriptor coordinates mismatch".into(),
            ));
        }
        previous = Some(&item.key);
    }
    Ok(descriptors)
}

/// A narrow run list projection; scientific request and source-selection blobs
/// remain available only through explicit exact run reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    /// Exact opaque run key.
    pub key: String,
    /// Selected scientific problem.
    pub problem: String,
    /// Exact retained primary source revision.
    pub revision: String,
    /// Recorded primary source ordering.
    pub source_sequence: u64,
    /// Recorded per-problem execution ordering.
    pub sequence: u64,
    /// Admitted implementation interpretation.
    pub interpretation: String,
    /// Current native authority generation.
    pub current_generation: u64,
    /// Current actual native attempt, when claimed.
    pub current_attempt: Option<String>,
    /// Persistent native cancellation authority.
    pub cancelled: bool,
    /// Current recorded terminal attempt, when settled.
    pub terminal_attempt: Option<String>,
    /// Actual terminal lifecycle class, separate from scientific usability.
    pub terminal_class: Option<String>,
}
fn decode_run_summary(mut row: Object) -> Result<RunSummary, CanonicalError> {
    use canonical_codec::{decode_boolean, decode_string, decode_uint, required};
    let key = decode_string(required(&mut row, "key")?)?;
    let problem = decode_string(required(&mut row, "problem")?)?;
    let revision = decode_string(required(&mut row, "revision")?)?;
    let source_sequence = decode_uint(required(&mut row, "source_sequence")?)?;
    let sequence = decode_uint(required(&mut row, "sequence")?)?;
    let interpretation = decode_string(required(&mut row, "interpretation")?)?;
    let current_generation = decode_uint(required(&mut row, "current_generation")?)?;
    let mut optional = |field| match row.remove(field).unwrap_or(Value::None) {
        Value::None => Ok(None),
        value => decode_string(value).map(Some),
    };
    let current_attempt = optional("current_attempt")?;
    let terminal_attempt = optional("terminal_attempt")?;
    let terminal_class = optional("terminal_class")?;
    let cancelled = decode_boolean(required(&mut row, "cancelled")?)?;
    if !row.is_empty() {
        return Err(canonical_codec::CodecError::UnknownFields.into());
    }
    Ok(RunSummary {
        key,
        problem,
        revision,
        source_sequence,
        sequence,
        interpretation,
        current_generation,
        current_attempt,
        cancelled,
        terminal_attempt,
        terminal_class,
    })
}

// One owning encoding for immutable run meaning, shared by publication and
// effect-free retry checks. Mutable attempt/cancellation state is excluded.
fn run_request_receipt(
    request: &RunRequest,
) -> Result<(CanonicalRun, Vec<Object>), CanonicalError> {
    identity(&request.key)?;
    identity(&request.revision.key)?;
    identity(&request.revision.problem)?;
    bounded(&request.request, crate::canonical::PAYLOAD_BYTES)?;
    bounded(&request.source_selection, crate::canonical::PAYLOAD_BYTES)?;
    bounded(&request.attestation, EXECUTION_METADATA_BYTES)?;
    let row = CanonicalRun {
        key: request.key.clone(),
        problem: request.revision.problem.clone(),
        revision: request.revision.key.clone(),
        source_sequence: request.revision.sequence,
        sequence: 0,
        request: request.request.clone().into(),
        source_selection: request.source_selection.clone().into(),
        attestation: request.attestation.clone().into(),
        interpretation: wire::INTERPRETATION.into(),
        current_generation: 0,
        current_attempt: None,
        cancelled: false,
        terminal_attempt: None,
        terminal_class: None,
    };
    let mut selected = request
        .sources
        .iter()
        .filter(|source| source.key != request.revision.key)
        .collect::<Vec<_>>();
    if selected.len() > 64 {
        return Err(CanonicalError::PayloadLimit);
    }
    selected.sort_by(|left, right| left.key.cmp(&right.key));
    let mut sources = Vec::with_capacity(selected.len());
    let mut previous = None;
    for source in selected {
        identity(&source.key)?;
        identity(&source.problem)?;
        if previous == Some(source.key.as_str()) {
            return Err(CanonicalError::Configuration(
                "duplicate retained source".into(),
            ));
        }
        previous = Some(source.key.as_str());
        let key = hash(
            "canonical.execution.run-source.v1",
            &[
                request.key.as_bytes(),
                source.problem.as_bytes(),
                source.key.as_bytes(),
            ],
        );
        let mut header = Object::new();
        header.insert("key", Value::String(key));
        header.insert("revision", Value::String(source.key.clone()));
        header.insert("problem", Value::String(source.problem.clone()));
        header.insert("sequence", canonical_codec::encode_uint(source.sequence)?);
        header.insert(
            "interpretation",
            Value::String(source.interpretation.clone()),
        );
        sources.push(header);
    }
    // Retain the exact source list in the immutable receipt; no later caller can
    // replay a semantic run identity with a different physical/model selection.
    let selections = request
        .sources
        .iter()
        .map(|source| {
            (
                &source.problem,
                &source.key,
                source.sequence,
                &source.interpretation,
            )
        })
        .collect::<Vec<_>>();
    let mut row = row;
    row.source_selection = serde_json::to_vec(&(1_u8, &request.source_selection, selections))
        .map_err(|error| CanonicalError::Configuration(error.to_string()))?
        .into();
    bounded(
        row.source_selection.as_slice(),
        crate::canonical::PAYLOAD_BYTES,
    )?;
    Ok((row, sources))
}

fn same_run_receipt(expected: &CanonicalRun, saved: &CanonicalRun) -> bool {
    saved.key == expected.key
        && saved.problem == expected.problem
        && saved.revision == expected.revision
        && saved.source_sequence == expected.source_sequence
        && saved.request == expected.request
        && saved.source_selection == expected.source_selection
        && saved.attestation == expected.attestation
        && saved.interpretation == expected.interpretation
}

impl CanonicalStore {
    /// Resolve only immutable pacing ownership. Actual run/fence authority is still
    /// checked by every rebuilt server transaction; lookup and queuing consume the
    /// same original request clock, without retaining a turn through settlement.
    pub(crate) async fn protected_execution_query<F, Q>(
        &self,
        run: &str,
        operation: &'static str,
        build: F,
    ) -> Result<surrealdb::IndexedResults, CanonicalError>
    where
        F: FnMut() -> Result<Q, CanonicalError>,
        Q: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
    {
        tokio::time::timeout(REQUEST_TIMEOUT, async {
            let mut response = bounded_query(
                self.db
                    .query("SELECT problem FROM ONLY type::record('canonical_runs',$run);")
                    .bind(("run", run.to_owned())),
            )
            .await?;
            let mut row = response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?;
            let problem = canonical_codec::decode_string(
                row.remove("problem")
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?;
            self.protected_query(&problem, operation, build).await
        })
        .await
        .map_err(|_| CanonicalError::Timeout)?
    }
    /// Retain selected inputs and provenance atomically before native execution.
    pub async fn begin_run(&self, request: &RunRequest) -> Result<CanonicalRun, CanonicalError> {
        let (row, sources) = run_request_receipt(request)?;
        let encoded = wire::encode_canonical_runs(&row)?;
        self.ensure_writes()?;
        let response = self
            .protected_query(&row.problem, "canonical_execution::begin_run", || {
                Ok(self
                    .db
                    .query("RETURN fn::pse_execution_v1::begin_run($row,$sources);")
                    .bind(("row", encoded.clone()))
                    .bind(("sources", sources.clone())))
            })
            .await;
        match response {
            Ok(mut response) => Ok(wire::decode_canonical_runs(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?),
            Err(error) => match self.canonical_run(&row.key).await {
                Ok(Some(saved)) if same_run_receipt(&row, &saved) => {
                    self.ensure_execution_retained(&row.key).await?;
                    Ok(saved)
                }
                Ok(Some(_)) => Err(CanonicalError::OperationReused),
                _ => Err(error),
            },
        }
    }
    /// Check the complete immutable run receipt without publication or retention effects.
    pub(crate) async fn check_execution_run_identity(
        &self,
        request: &RunRequest,
    ) -> Result<(), CanonicalError> {
        let (expected, _) = run_request_receipt(request)?;
        let saved = self
            .canonical_run(&expected.key)
            .await?
            .ok_or(CanonicalError::OperationReused)?;
        if !same_run_receipt(&expected, &saved) {
            return Err(CanonicalError::OperationReused);
        }
        Ok(())
    }
    /// Bounded semantic ordering over one problem's recorded execution sequence.
    pub async fn execution_run_page(
        &self,
        problem: &str,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<RunSummary>, CanonicalError> {
        identity(problem)?;
        if !(1..=64).contains(&limit) {
            return Err(CanonicalError::PayloadLimit);
        }
        let after = after
            .map(canonical_codec::encode_uint)
            .transpose()?
            .unwrap_or(canonical_codec::encode_int(-1)?);
        let mut result=bounded_query(self.db.query("SELECT key,problem,revision,source_sequence,sequence,interpretation,current_generation,current_attempt,cancelled,terminal_attempt,terminal_class FROM canonical_runs WHERE problem=$problem AND sequence>$after ORDER BY sequence LIMIT $limit;").bind(("problem",problem.to_owned())).bind(("after",after)).bind(("limit",limit))).await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(decode_run_summary)
            .collect()
    }
    /// Exact scalar metadata for one retained seed; payload selection requires its manifest.
    pub async fn result_seed(
        &self,
        key: &str,
    ) -> Result<Option<pse_model::generated::runtime::canonical_result_seeds::Row>, CanonicalError>
    {
        identity(key)?;
        let mut result = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_result_seeds',$key);")
                .bind(("key", key.to_owned())),
        )
        .await?;
        result
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_result_seeds)
            .transpose()
            .map_err(Into::into)
    }
    /// Narrow eligibility selector; callers validate terminal scientific permission.
    pub async fn result_seed_candidates(
        &self,
        layout: &str,
        preparation: &str,
        backend: &str,
        attempt: Option<&str>,
    ) -> Result<Vec<pse_model::generated::runtime::canonical_result_seeds::Row>, CanonicalError>
    {
        let query = if attempt.is_some() {
            "SELECT * FROM canonical_result_seeds WHERE layout=$layout AND preparation=$preparation AND backend=$backend AND attempt=$attempt AND (SELECT VALUE key FROM canonical_result_retirements WHERE run=$parent.run LIMIT 1) = [] ORDER BY run_sequence DESC,attempt_generation DESC,step DESC LIMIT 64;"
        } else {
            "SELECT * FROM canonical_result_seeds WHERE layout=$layout AND preparation=$preparation AND backend=$backend AND (SELECT VALUE key FROM canonical_result_retirements WHERE run=$parent.run LIMIT 1) = [] ORDER BY run_sequence DESC,attempt_generation DESC,step DESC LIMIT 64;"
        };
        let mut result = bounded_query(
            self.db
                .query(query)
                .bind(("layout", layout.to_owned()))
                .bind(("preparation", preparation.to_owned()))
                .bind(("backend", backend.to_owned()))
                .bind(("attempt", attempt.map(str::to_owned))),
        )
        .await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(|row| wire::decode_canonical_result_seeds(row).map_err(Into::into))
            .collect()
    }
    /// Narrow immutable seed headers for one explicit attempt, ordered by scientific step.
    pub async fn result_seed_page(
        &self,
        attempt: &str,
        after: Option<u64>,
    ) -> Result<Vec<pse_model::generated::runtime::canonical_result_seeds::Row>, CanonicalError>
    {
        identity(attempt)?;
        let after = after
            .map(canonical_codec::encode_uint)
            .transpose()?
            .unwrap_or(canonical_codec::encode_int(-1)?);
        let mut result=bounded_query(self.db.query("SELECT * FROM canonical_result_seeds WHERE attempt=$attempt AND step>$after ORDER BY step LIMIT 64;").bind(("attempt",attempt.to_owned())).bind(("after",after))).await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(|row| wire::decode_canonical_result_seeds(row).map_err(Into::into))
            .collect()
    }
    /// Renew the current live worker fence; expired or cancelled workers cannot revive.
    pub async fn renew_attempt(
        &self,
        fence: &AttemptFence,
        lifetime: Duration,
    ) -> Result<CanonicalAttempt, CanonicalError> {
        let lifetime =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if lifetime <= 0 || lifetime > 86_400_000_000 {
            return Err(CanonicalError::Configuration(
                "attempt renewal lifetime out of bounds".into(),
            ));
        }
        let generation = canonical_codec::encode_uint(fence.generation)?;
        self.ensure_writes()?;
        let mut response = self
            .protected_execution_query(&fence.run, "canonical_execution::renew_attempt", || {
                Ok(self
                    .db
                    .query(
                        "RETURN fn::pse_execution_v1::renew($run,$attempt,$generation,$lifetime);",
                    )
                    .bind(("run", fence.run.clone()))
                    .bind(("attempt", fence.attempt.clone()))
                    .bind(("generation", generation.clone()))
                    .bind(("lifetime", lifetime)))
            })
            .await?;
        Ok(wire::decode_canonical_attempts(
            response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }
    /// Register only a structurally complete immutable chunked seed under its live gate.
    /// Eligibility still comes from the scientific completion owner and terminal descriptor.
    pub async fn register_result_seed(
        &self,
        fence: &AttemptFence,
        operation: &str,
        seed: &pse_model::generated::runtime::canonical_result_seeds::Row,
    ) -> Result<(), CanonicalError> {
        identity(operation)?;
        identity(&seed.key)?;
        if seed.run != fence.run
            || seed.attempt != fence.attempt
            || seed.batch_count == 0
            || seed.batch != result_batch_key(&fence.attempt, &seed.result_set, seed.first_ordinal)
        {
            return Err(CanonicalError::Configuration(
                "seed descriptor coordinate mismatch".into(),
            ));
        }
        let encoded = wire::encode_canonical_result_seeds(seed)?;
        let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
        digest.str("pse.execution.seed-descriptor.v1");
        pse_model::SemanticFrame::frame(seed, &mut digest);
        let request = json(&(
            &fence.run,
            &fence.attempt,
            fence.generation,
            digest.finish_hash().to_hex(),
        ))?;
        let generation = canonical_codec::encode_uint(fence.generation)?;
        self.ensure_writes()?;
        let result=self.protected_execution_query(&fence.run, "canonical_execution::register_result_seed", ||Ok(self.db.query("RETURN fn::pse_execution_v1::seed($run,$attempt,$generation,$operation,$request,$seed);").bind(("run",fence.run.clone())).bind(("attempt",fence.attempt.clone())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("seed",encoded.clone())))).await;
        match result {
            Ok(mut response) => {
                let saved = wire::decode_canonical_result_seeds(
                    response
                        .take::<Option<Object>>(0)?
                        .ok_or(CanonicalError::IncompleteResponse)?,
                )?;
                if saved != *seed {
                    return Err(CanonicalError::OperationReused);
                }
                Ok(())
            }
            Err(error) => match self.settle_operation(operation, "seed", &request).await? {
                Some(_) => Ok(()),
                None => Err(error),
            },
        }
    }
    /// Coherent bounded recovery observation. It grants no claim or fence;
    /// the recovery effect independently checks current expiration/cancellation.
    pub async fn recovery_snapshot(
        &self,
        key: &str,
    ) -> Result<
        (
            pse_model::generated::runtime::canonical_study_points::Row,
            CanonicalAttempt,
            pse_model::generated::runtime::canonical_studies::Row,
            CanonicalRun,
        ),
        CanonicalError,
    > {
        identity(key)?;
        let mut response = bounded_query(self.db.query("BEGIN; LET $point = SELECT * FROM ONLY type::record('canonical_study_points',$key); RETURN {point:$point,attempt:(SELECT * FROM ONLY type::record('canonical_attempts',$point.attempt)),study:(SELECT * FROM ONLY type::record('canonical_studies',$point.study)),run:(SELECT * FROM ONLY type::record('canonical_runs',$point.run))}; COMMIT;").bind(("key",key.to_owned()))).await?;
        let index = response.num_statements().saturating_sub(2);
        let mut row = response
            .take::<Option<Object>>(index)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        let object = |value| match value {
            Value::Object(row) => Ok(row),
            _ => Err(CanonicalError::IncompleteResponse),
        };
        let point = wire::decode_canonical_study_points(object(canonical_codec::required(
            &mut row, "point",
        )?)?)?;
        let attempt = wire::decode_canonical_attempts(object(canonical_codec::required(
            &mut row, "attempt",
        )?)?)?;
        let study =
            wire::decode_canonical_studies(object(canonical_codec::required(&mut row, "study")?)?)?;
        let run =
            wire::decode_canonical_runs(object(canonical_codec::required(&mut row, "run")?)?)?;
        if !row.is_empty()
            || point.key != key
            || point.attempt.as_deref() != Some(&attempt.key)
            || point.run != attempt.run
            || point.run != run.key
            || point.study != study.key
        {
            return Err(CanonicalError::Configuration(
                "recovery snapshot identity mismatch".into(),
            ));
        }
        Ok((point, attempt, study, run))
    }
    /// Read a retained semantic execution independently of its current generation.
    pub async fn canonical_run(&self, key: &str) -> Result<Option<CanonicalRun>, CanonicalError> {
        let mut response = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_runs', $key);")
                .bind(("key", key.to_owned())),
        )
        .await?;
        response
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_runs)
            .transpose()
            .map_err(Into::into)
    }
    /// Read an explicit attempt; terminal history is independent of the latest pointer.
    pub async fn canonical_attempt(
        &self,
        key: &str,
    ) -> Result<Option<CanonicalAttempt>, CanonicalError> {
        let mut response = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_attempts', $key);")
                .bind(("key", key.to_owned())),
        )
        .await?;
        response
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_attempts)
            .transpose()
            .map_err(Into::into)
    }
    /// Read a closed manifest; admission additionally requires a terminal attempt.
    pub async fn canonical_result_manifest(
        &self,
        key: &str,
    ) -> Result<Option<ResultManifest>, CanonicalError> {
        let mut response = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_result_manifests', $key);")
                .bind(("key", key.to_owned())),
        )
        .await?;
        let row = response
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_result_manifests)
            .transpose()?;
        if let Some(row) = &row {
            decode_result_descriptors(row)?;
        }
        Ok(row)
    }
    async fn ensure_execution_retained(&self, run: &str) -> Result<(), CanonicalError> {
        let mut retirement = bounded_query(
            self.db
                .query("SELECT key FROM ONLY type::record('canonical_result_retirements',$run);")
                .bind(("run", run.to_owned())),
        )
        .await?;
        if retirement.take::<Option<Object>>(0)?.is_some() {
            return Err(CanonicalError::Configuration(
                "execution results explicitly retired".into(),
            ));
        }
        Ok(())
    }
    pub(crate) async fn settle_operation(
        &self,
        operation: &str,
        kind: &str,
        request: &[u8],
    ) -> Result<Option<String>, CanonicalError> {
        let mut response = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_execution_operations', $key);")
                .bind(("key", operation.to_owned())),
        )
        .await?;
        let Some(object) = response.take::<Option<Object>>(0)? else {
            return Ok(None);
        };
        let row = wire::decode_canonical_execution_operations(object)?;
        self.ensure_execution_retained(&row.run).await?;
        if row.kind != kind || row.request.as_slice() != request {
            return Err(CanonicalError::OperationReused);
        }
        Ok(row.attempt)
    }
    /// Claim a fresh actual attempt, or settle the same claim after lost acknowledgment.
    pub async fn claim_run(
        &self,
        run: &str,
        operation: &str,
        worker: &str,
        lifetime: Duration,
    ) -> Result<AttemptFence, CanonicalError> {
        identity(run)?;
        identity(operation)?;
        identity(worker)?;
        let lifetime =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if lifetime <= 0 || lifetime > 86_400_000_000 {
            return Err(CanonicalError::Configuration(
                "attempt lifetime must be positive and at most one day".into(),
            ));
        }
        let attempt = execution_attempt_key(run, operation);
        let request = json(&(run, operation, worker, lifetime, wire::INTERPRETATION))?;
        self.ensure_writes()?;
        let result = self.protected_execution_query(run, "canonical_execution::claim_run", || Ok(self.db.query("RETURN fn::pse_execution_v1::claim($run,$operation,$request,$attempt,$worker,$lifetime,$interpretation);").bind(("run",run.to_owned())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("attempt",attempt.clone())).bind(("worker",worker.to_owned())).bind(("lifetime",lifetime)).bind(("interpretation",wire::INTERPRETATION)))).await;
        let row = match result {
            Ok(mut response) => wire::decode_canonical_attempts(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?,
            Err(error) => match self.settle_operation(operation, "claim", &request).await? {
                Some(key) => self.canonical_attempt(&key).await?.ok_or(error)?,
                None => return Err(error),
            },
        };
        Ok(AttemptFence {
            run: row.run,
            attempt: row.key,
            generation: row.generation,
        })
    }
    /// Append an exact bounded batch under the attempt's live ingestion gate.
    pub async fn append_result_batch(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: &[u8],
        row_count: u64,
    ) -> Result<ResultBatch, CanonicalError> {
        self.append_execution_batch(
            fence,
            operation,
            name,
            ordinal,
            payload,
            row_count,
            None,
            &[],
            &[],
        )
        .await
    }
    /// Append an independently decodable scientific block and its indexed metadata atomically.
    #[allow(
        clippy::too_many_arguments,
        reason = "the ingestion gate checks the attempt fence, operation identity, batch extent and scientific descriptor separately"
    )]
    pub async fn append_result_block(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: &[u8],
        row_count: u64,
        block: &ResultBlock,
    ) -> Result<ResultBatch, CanonicalError> {
        self.append_execution_batch(
            fence,
            operation,
            name,
            ordinal,
            payload,
            row_count,
            Some(block),
            &[],
            &[],
        )
        .await
    }
    /// Retain exact scalar values and their batch receipt under one ingestion gate.
    #[allow(
        clippy::too_many_arguments,
        reason = "scalar indexes and their immutable batch descriptor must share the explicit fenced ingestion operation"
    )]
    pub async fn append_result_block_cells(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: &[u8],
        row_count: u64,
        block: &ResultBlock,
        cells: &[ResultCell],
    ) -> Result<ResultBatch, CanonicalError> {
        if cells.windows(2).any(|pair| pair[0].key >= pair[1].key) {
            return Err(CanonicalError::Configuration(
                "scalar metadata must have strictly ordered identities".into(),
            ));
        }
        self.append_execution_batch(
            fence,
            operation,
            name,
            ordinal,
            payload,
            row_count,
            Some(block),
            cells,
            &[],
        )
        .await
    }
    /// Atomically index original IPC rows and scientific output groups with their one immutable payload.
    #[allow(
        clippy::too_many_arguments,
        reason = "the original payload, block descriptor and derived indexes are distinct premises of one fenced operation"
    )]
    pub async fn append_result_block_indexes(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: &[u8],
        row_count: u64,
        block: &ResultBlock,
        cells: &[ResultCell],
        outputs: &[ResultOutput],
    ) -> Result<ResultBatch, CanonicalError> {
        if cells.windows(2).any(|pair| pair[0].key >= pair[1].key)
            || outputs.windows(2).any(|pair| pair[0].key >= pair[1].key)
        {
            return Err(CanonicalError::Configuration(
                "result index metadata must have strictly ordered identities".into(),
            ));
        }
        self.append_execution_batch(
            fence,
            operation,
            name,
            ordinal,
            payload,
            row_count,
            Some(block),
            cells,
            outputs,
        )
        .await
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "one transaction validates separate fence, operation, batch and optional scientific index premises"
    )]
    async fn append_execution_batch(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: &[u8],
        row_count: u64,
        block: Option<&ResultBlock>,
        cells: &[ResultCell],
        outputs: &[ResultOutput],
    ) -> Result<ResultBatch, CanonicalError> {
        identity(operation)?;
        identity(name)?;
        if name.len() > 128 {
            return Err(CanonicalError::PayloadLimit);
        }
        bounded(payload, RESULT_BATCH_BYTES)?;
        let set_key = result_set_key(&fence.attempt, name);
        let key = result_batch_key(&fence.attempt, &set_key, ordinal);
        let digest = result_payload_digest(payload);
        let set = ResultSet {
            key: set_key.clone(),
            attempt: fence.attempt.clone(),
            name: name.into(),
            interpretation: wire::INTERPRETATION.into(),
            next_ordinal: 0,
            row_count: 0,
        };
        let batch = ResultBatch {
            key: key.clone(),
            attempt: fence.attempt.clone(),
            result_set: set_key.clone(),
            ordinal,
            digest: digest.clone(),
            payload: payload.to_vec().into(),
            row_count,
        };
        if let Some(block) = block
            && (block.key != key
                || block.batch != key
                || block.result_set != set_key
                || block.ordinal != ordinal
                || block.rows != row_count
                || block.payload_bytes != payload.len() as u64
                || block.payload_digest != digest
                || block.interpretation != wire::INTERPRETATION
                || block.end < block.start
                || block.end - block.start != row_count)
        {
            return Err(CanonicalError::Configuration(
                "scientific block descriptor does not match immutable batch".into(),
            ));
        }
        if let Some(block) = block {
            crate::canonical_results::validate_result_block(block, &batch)?;
        }
        if cells.len() > wire::RESULT_INDEX_RECORDS {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut cell_hash = FramedHasher::new(Frame::CanonicalPayloadV1);
        cell_hash.str("pse.execution.scalar-metadata.v1");
        for cell in cells {
            crate::canonical_results::validate_result_cell(cell)?;
            if cell.batch != key || cell.result_set != set_key {
                return Err(CanonicalError::Configuration(
                    "scalar cell batch coordinate mismatch".into(),
                ));
            }
            if block.is_some_and(|block| cell.row < block.start || cell.row >= block.end) {
                return Err(CanonicalError::Configuration(
                    "scalar cell row is outside original IPC block".into(),
                ));
            }
            pse_model::SemanticFrame::frame(cell, &mut cell_hash);
        }
        if outputs.len() > wire::RESULT_INDEX_RECORDS {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut output_hash = FramedHasher::new(Frame::CanonicalPayloadV1);
        output_hash.str("pse.execution.output-index-metadata.v1");
        for output in outputs {
            identity(&output.key)?;
            identity(&output.output)?;
            identity(&output.partition)?;
            if output.batch != key
                || output.result_set != set_key
                || output.interpretation != wire::INTERPRETATION
                || output.end < output.start
                || block.is_none_or(|block| output.start < block.start || output.end > block.end)
            {
                return Err(CanonicalError::Configuration(
                    "output index batch coordinate mismatch".into(),
                ));
            }
            pse_model::SemanticFrame::frame(output, &mut output_hash);
        }
        let mut block_hash = FramedHasher::new(Frame::CanonicalPayloadV1);
        block_hash.str("pse.execution.block-metadata.v1");
        if let Some(block) = block {
            pse_model::SemanticFrame::frame(block, &mut block_hash);
        }
        let request = json(&(
            &fence.run,
            &fence.attempt,
            fence.generation,
            name,
            ordinal,
            &digest,
            row_count,
            block_hash.finish_hash().to_hex(),
            cell_hash.finish_hash().to_hex(),
            output_hash.finish_hash().to_hex(),
        ))?;
        let block = block
            .map(wire::encode_canonical_result_blocks)
            .transpose()?;
        let cells = cells
            .iter()
            .map(wire::encode_canonical_result_cells)
            .collect::<Result<Vec<_>, _>>()?;
        let outputs = outputs
            .iter()
            .map(wire::encode_canonical_result_block_outputs)
            .collect::<Result<Vec<_>, _>>()?;
        let mut metadata = Object::new();
        metadata.insert(
            "block",
            block.clone().map(Value::Object).unwrap_or(Value::None),
        );
        metadata.insert(
            "cells",
            Value::Array(
                cells
                    .iter()
                    .cloned()
                    .map(Value::Object)
                    .collect::<Vec<_>>()
                    .into(),
            ),
        );
        metadata.insert(
            "outputs",
            Value::Array(
                outputs
                    .iter()
                    .cloned()
                    .map(Value::Object)
                    .collect::<Vec<_>>()
                    .into(),
            ),
        );
        if result_metadata_extent(&Value::Object(metadata))? > EXECUTION_METADATA_BYTES {
            return Err(CanonicalError::PayloadLimit);
        }
        let encoded_set = wire::encode_canonical_result_sets(&set)?;
        let encoded_batch = wire::encode_canonical_result_batches(&batch)?;
        let generation = canonical_codec::encode_uint(fence.generation)?;
        let variables = [
            Value::String(fence.run.clone()),
            Value::String(fence.attempt.clone()),
            generation.clone(),
            Value::String(operation.into()),
            Value::Bytes(Bytes::from(request.clone())),
            Value::Object(encoded_set.clone()),
            Value::Object(encoded_batch.clone()),
            block.clone().map(Value::Object).unwrap_or(Value::None),
            Value::Array(
                cells
                    .iter()
                    .cloned()
                    .map(Value::Object)
                    .collect::<Vec<_>>()
                    .into(),
            ),
            Value::Array(
                outputs
                    .iter()
                    .cloned()
                    .map(Value::Object)
                    .collect::<Vec<_>>()
                    .into(),
            ),
        ];
        let extent = variables.iter().try_fold(1024_usize, |sum, value| {
            let bytes =
                surrealdb::types::encode_proto(value).map_err(|_| CanonicalError::PayloadLimit)?;
            sum.checked_add(bytes.len())
                .ok_or(CanonicalError::PayloadLimit)
        })?;
        if extent > wire::RESULT_MESSAGE_BYTES {
            return Err(CanonicalError::PayloadLimit);
        }
        self.ensure_writes()?;
        let result = self.protected_execution_query(&fence.run, "canonical_execution::append_execution_batch", || Ok(self.db.query("RETURN fn::pse_execution_v1::append($run,$attempt,$generation,$operation,$request,$set,$batch,$block,$cells,$outputs);").bind(("run",fence.run.clone())).bind(("attempt",fence.attempt.clone())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("set",encoded_set.clone())).bind(("batch",encoded_batch.clone())).bind(("block",block.clone().map(Value::Object).unwrap_or(Value::None))).bind(("cells",cells.clone())).bind(("outputs",outputs.clone())))).await;
        match result {
            Ok(mut response) => {
                let saved = response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?;
                let mut expected = encoded_batch;
                expected.remove("payload");
                expected.insert("request", Value::Bytes(Bytes::from(request.clone())));
                if saved != expected {
                    return Err(CanonicalError::OperationReused);
                }
                Ok(batch)
            }
            Err(error) => {
                // The exact operation receipt exists only after all original
                // payload bytes and metadata were admitted atomically. Settlement
                // retains that guarantee without fetching the IPC success echo.
                if self
                    .settle_operation(operation, "append", &request)
                    .await?
                    .is_none()
                {
                    return Err(error);
                }
                Ok(batch)
            }
        }
    }
    /// Fence late batches before reconciling immutable metadata outside a transaction.
    pub async fn close_result_ingestion(
        &self,
        fence: &AttemptFence,
        operation: &str,
    ) -> Result<ClosedAttempt, CanonicalError> {
        identity(operation)?;
        let request = json(&(&fence.run, &fence.attempt, fence.generation))?;
        let generation = canonical_codec::encode_uint(fence.generation)?;
        self.ensure_writes()?;
        let result = self.protected_execution_query(&fence.run, "canonical_execution::close_result_ingestion", || Ok(self.db.query("RETURN fn::pse_execution_v1::close($run,$attempt,$generation,$operation,$request);").bind(("run",fence.run.clone())).bind(("attempt",fence.attempt.clone())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))))).await;
        self.closed_response(result, operation, "close", &request)
            .await
    }
    async fn closure_operation(
        &self,
        operation: &str,
        kind: &str,
        request: Option<&[u8]>,
    ) -> Result<Option<(String, u64)>, CanonicalError> {
        let mut response = bounded_query(
            self.db
                .query(
                    "SELECT * FROM ONLY type::record('canonical_execution_operations',$operation);",
                )
                .bind(("operation", operation.to_owned())),
        )
        .await?;
        let Some(row) = response.take::<Option<Object>>(0)? else {
            return Ok(None);
        };
        let row = wire::decode_canonical_execution_operations(row)?;
        if row.kind != kind || request.is_some_and(|request| row.request.as_slice() != request) {
            return Err(CanonicalError::OperationReused);
        }
        let authority = std::str::from_utf8(row.result.as_slice())
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .parse::<u64>()
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        Ok(Some((
            row.attempt
                .ok_or_else(|| CanonicalError::Configuration("closure attempt missing".into()))?,
            authority,
        )))
    }
    async fn closed_response(
        &self,
        result: Result<surrealdb::IndexedResults, CanonicalError>,
        operation: &str,
        kind: &str,
        request: &[u8],
    ) -> Result<ClosedAttempt, CanonicalError> {
        let receipt = self
            .closure_operation(operation, kind, Some(request))
            .await?;
        let Some((key, authority)) = receipt else {
            return Err(result.err().unwrap_or(CanonicalError::IncompleteResponse));
        };
        let mut value = match result {
            Ok(mut response) => closed(wire::decode_canonical_attempts(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?)?,
            Err(error) => closed(self.canonical_attempt(&key).await?.ok_or(error)?)?,
        };
        value.authority = authority;
        Ok(value)
    }
    /// Revoke worker authority and freeze available observations at cancellation.
    pub async fn cancel_run(
        &self,
        run: &str,
        operation: &str,
    ) -> Result<CanonicalRun, CanonicalError> {
        identity(run)?;
        identity(operation)?;
        let request = json(&(run, operation))?;
        self.ensure_writes()?;
        let result = self
            .protected_execution_query(run, "canonical_execution::cancel_run", || {
                Ok(self
                    .db
                    .query("RETURN fn::pse_execution_v1::cancel($run,$operation,$request);")
                    .bind(("run", run.to_owned()))
                    .bind(("operation", operation.to_owned()))
                    .bind(("request", Bytes::from(request.clone()))))
            })
            .await;
        match result {
            Ok(mut response) => Ok(wire::decode_canonical_runs(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?),
            Err(error) => {
                // Cancellation operations have no attempt; absence must be distinguished.
                let mut response = bounded_query(self.db.query("SELECT * FROM ONLY type::record('canonical_execution_operations',$operation);").bind(("operation",operation.to_owned()))).await?;
                let Some(row) = response.take::<Option<Object>>(0)? else {
                    return Err(error);
                };
                let row = wire::decode_canonical_execution_operations(row)?;
                if row.kind != "cancel" || row.request.as_slice() != request {
                    return Err(CanonicalError::OperationReused);
                }
                self.canonical_run(run).await?.ok_or(error)
            }
        }
    }
    /// Current recovery owner closes expired or cancelled staging; no stale worker can use this authority.
    pub async fn recover_closed_attempt(
        &self,
        run: &str,
        operation: &str,
    ) -> Result<ClosedAttempt, CanonicalError> {
        identity(run)?;
        identity(operation)?;
        let request = json(&(run, operation))?;
        self.ensure_writes()?;
        let result = self
            .protected_execution_query(run, "canonical_execution::recover_closed_attempt", || {
                Ok(self
                    .db
                    .query("RETURN fn::pse_execution_v1::recover($run,$operation,$request);")
                    .bind(("run", run.to_owned()))
                    .bind(("operation", operation.to_owned()))
                    .bind(("request", Bytes::from(request.clone()))))
            })
            .await;
        self.closed_response(result, operation, "recover", &request)
            .await
    }
    /// Resume an already closed live operation after restart without repeating native work.
    pub async fn resume_closed_attempt(
        &self,
        attempt: &str,
        operation: &str,
    ) -> Result<ClosedAttempt, CanonicalError> {
        let mut response=bounded_query(self.db.query("SELECT kind FROM ONLY type::record('canonical_execution_operations',$operation);").bind(("operation",operation.to_owned()))).await?;
        let mut row = response
            .take::<Option<Object>>(0)?
            .ok_or_else(|| CanonicalError::Configuration("closure operation unavailable".into()))?;
        let kind = canonical_codec::decode_string(canonical_codec::required(&mut row, "kind")?)?;
        if kind != "close" && kind != "recover" {
            return Err(CanonicalError::OperationReused);
        }
        let (key, authority) = self
            .closure_operation(operation, &kind, None)
            .await?
            .ok_or(CanonicalError::IncompleteResponse)?;
        if key != attempt {
            return Err(CanonicalError::OperationReused);
        }
        let mut value =
            closed(self.canonical_attempt(attempt).await?.ok_or_else(|| {
                CanonicalError::Configuration("closed attempt unavailable".into())
            })?)?;
        value.authority = authority;
        Ok(value)
    }
    /// Reconcile exact immutable batch membership in bounded metadata pages, then admit its descriptor.
    pub async fn reconcile_closed_attempt(
        &self,
        closed: &ClosedAttempt,
    ) -> Result<ClosedManifest, CanonicalError> {
        let attempt = self
            .canonical_attempt(&closed.fence.attempt)
            .await?
            .ok_or_else(|| CanonicalError::Configuration("closed attempt unavailable".into()))?;
        if !attempt.closed
            || attempt.ingestion_open
            || attempt.generation != closed.fence.generation
        {
            return Err(CanonicalError::Configuration(
                "reconciliation attempt fenced".into(),
            ));
        }
        if let Some(row) = self.canonical_result_manifest(&attempt.key).await? {
            return Ok(ClosedManifest {
                closed: closed.clone(),
                row,
            });
        }
        let mut response=bounded_query(self.db.query("SELECT * FROM canonical_result_sets WHERE attempt=$attempt ORDER BY key LIMIT 257;").bind(("attempt",attempt.key.clone()))).await?;
        let rows: Vec<Object> = response.take(0)?;
        if rows.len() > RESULT_SETS {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut descriptors = Vec::with_capacity(rows.len());
        for row in rows {
            let set = wire::decode_canonical_result_sets(row)?;
            if set.key != result_set_key(&attempt.key, &set.name)
                || set.interpretation != wire::INTERPRETATION
            {
                return Err(CanonicalError::Configuration(
                    "closed result-set coordinate mismatch".into(),
                ));
            }
            let mut next = 0_u64;
            let mut row_count = 0_u64;
            let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
            digest.str("pse.execution.batch-selection.v1");
            while next < set.next_ordinal {
                let mut page=bounded_query(self.db.query("SELECT key,attempt,result_set,ordinal,digest,row_count FROM canonical_result_batches WHERE result_set=$set AND ordinal >= $next ORDER BY ordinal LIMIT 64;").bind(("set",set.key.clone())).bind(("next",canonical_codec::encode_uint(next)?))).await?;
                let batches: Vec<Object> = page.take(0)?;
                if batches.is_empty() {
                    return Err(CanonicalError::Configuration(
                        "closed result batch missing".into(),
                    ));
                }
                for mut batch in batches {
                    let key = canonical_codec::decode_string(canonical_codec::required(
                        &mut batch, "key",
                    )?)?;
                    let selected_attempt = canonical_codec::decode_string(
                        canonical_codec::required(&mut batch, "attempt")?,
                    )?;
                    let selected_set = canonical_codec::decode_string(canonical_codec::required(
                        &mut batch,
                        "result_set",
                    )?)?;
                    let ordinal = canonical_codec::decode_uint(canonical_codec::required(
                        &mut batch, "ordinal",
                    )?)?;
                    let payload_digest = canonical_codec::decode_string(
                        canonical_codec::required(&mut batch, "digest")?,
                    )?;
                    let rows = canonical_codec::decode_uint(canonical_codec::required(
                        &mut batch,
                        "row_count",
                    )?)?;
                    if next >= set.next_ordinal
                        || ordinal != next
                        || key != result_batch_key(&attempt.key, &set.key, ordinal)
                        || selected_attempt != attempt.key
                        || selected_set != set.key
                    {
                        return Err(CanonicalError::Configuration(
                            "closed result batch coverage mismatch".into(),
                        ));
                    }
                    digest.str(&key).str(&payload_digest).u64(rows);
                    row_count = row_count
                        .checked_add(rows)
                        .ok_or(CanonicalError::PayloadLimit)?;
                    next = next.checked_add(1).ok_or(CanonicalError::PayloadLimit)?;
                }
            }
            if row_count != set.row_count {
                return Err(CanonicalError::Configuration(
                    "closed row coverage mismatch".into(),
                ));
            }
            descriptors.push(ResultSetDescriptor {
                key: set.key,
                name: set.name,
                batch_count: next,
                row_count,
                batches_digest: digest.finish_hash().to_hex(),
            });
        }
        let descriptors = json(&descriptors)?;
        bounded(&descriptors, EXECUTION_METADATA_BYTES)?;
        let row = ResultManifest {
            key: attempt.key.clone(),
            attempt: attempt.key,
            generation: attempt.generation,
            digest: metadata_digest(&descriptors),
            descriptors: descriptors.into(),
        };
        let encoded = wire::encode_canonical_result_manifests(&row)?;
        self.ensure_writes()?;
        let result = self
            .protected_execution_query(
                &closed.fence.run,
                "canonical_execution::reconcile_closed_attempt",
                || {
                    Ok(self
                        .db
                        .query("RETURN fn::pse_execution_v1::manifest($run,$manifest);")
                        .bind(("run", closed.fence.run.clone()))
                        .bind(("manifest", encoded.clone())))
                },
            )
            .await;
        match result {
            Ok(mut response) => {
                let saved = wire::decode_canonical_result_manifests(
                    response
                        .take::<Option<Object>>(0)?
                        .ok_or(CanonicalError::IncompleteResponse)?,
                )?;
                if saved != row {
                    return Err(CanonicalError::OperationReused);
                }
            }
            Err(error) => {
                self.ensure_execution_retained(&closed.fence.run).await?;
                if self.canonical_result_manifest(&row.key).await?.as_ref() != Some(&row) {
                    return Err(error);
                }
            }
        }
        Ok(ClosedManifest {
            closed: closed.clone(),
            row,
        })
    }
    /// Atomically expose the frozen selection with the actual scientific terminal class.
    ///
    /// # Safety
    /// The controlled scientific writer must have completed and admitted the exact
    /// observations described by this manifest, and must supply their actual terminal
    /// class and completion. In particular, success requires the scientific owner's
    /// numerical conclusion, physical eligibility and contextual accuracy admission.
    /// This is an intercrate authority assertion; it performs no pointer or ABI work.
    #[allow(
        unsafe_code,
        reason = "intercrate scientific-writer authority assertion; no pointer or ABI operations"
    )]
    pub async unsafe fn seal_attempt(
        &self,
        manifest: &ClosedManifest,
        operation: &str,
        outcome: TerminalClass,
        completion: &[u8],
    ) -> Result<CanonicalAttempt, CanonicalError> {
        self.seal_execution(manifest, operation, outcome, completion, None)
            .await
    }
    /// Atomically admit the owning study summary and conclude its exact study header.
    ///
    /// # Safety
    /// The study owner must have assembled this exact complete observation of every
    /// settled occurrence and supplied its actual conclusion. This assertion includes
    /// the scientific obligations of `seal_attempt`; it performs no pointer or ABI work.
    #[allow(
        unsafe_code,
        reason = "controlled study owner asserts complete admitted occurrence observations"
    )]
    pub async unsafe fn seal_study_summary(
        &self,
        study: &str,
        manifest: &ClosedManifest,
        operation: &str,
        outcome: TerminalClass,
        completion: &[u8],
    ) -> Result<CanonicalAttempt, CanonicalError> {
        identity(study)?;
        self.seal_execution(manifest, operation, outcome, completion, Some(study))
            .await
    }
    async fn seal_execution(
        &self,
        manifest: &ClosedManifest,
        operation: &str,
        outcome: TerminalClass,
        completion: &[u8],
        study: Option<&str>,
    ) -> Result<CanonicalAttempt, CanonicalError> {
        identity(operation)?;
        bounded(completion, EXECUTION_METADATA_BYTES)?;
        let fence = &manifest.closed.fence;
        let request = json(&(
            &fence.run,
            &fence.attempt,
            manifest.closed.authority,
            &manifest.row.key,
            &manifest.row.digest,
            outcome,
            result_payload_digest(completion),
            study,
        ))?;
        bounded(&request, crate::canonical::PAYLOAD_BYTES)?;
        let authority = canonical_codec::encode_uint(manifest.closed.authority)?;
        self.ensure_writes()?;
        let query = if study.is_some() {
            "RETURN fn::pse_study_v1::seal($study,$run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion);"
        } else {
            "RETURN fn::pse_execution_v1::seal($run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion);"
        };
        let result = self
            .protected_execution_query(&fence.run, "canonical_execution::seal_execution", || {
                Ok(self
                    .db
                    .query(query)
                    .bind(("study", study.map(str::to_owned)))
                    .bind(("run", fence.run.clone()))
                    .bind(("attempt", fence.attempt.clone()))
                    .bind(("authority", authority.clone()))
                    .bind(("operation", operation.to_owned()))
                    .bind(("request", Bytes::from(request.clone())))
                    .bind(("manifest", manifest.row.key.clone()))
                    .bind(("digest", manifest.row.digest.clone()))
                    .bind(("outcome", outcome.as_str()))
                    .bind(("completion", Bytes::from(completion.to_vec()))))
            })
            .await;
        match result {
            Ok(mut response) => Ok(wire::decode_canonical_attempts(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?),
            Err(error) => match self.settle_operation(operation, "seal", &request).await? {
                Some(key) => self.canonical_attempt(&key).await?.ok_or(error),
                None => Err(error),
            },
        }
    }
}

#[cfg(test)]
mod canonical_result_admission_unit {
    use super::*;
    #[test]
    fn index_metadata_accounts_strings_optional_values_and_combined_arrays() {
        let mut row = Object::new();
        row.insert("key", Value::String("a".into()));
        assert_eq!(
            result_metadata_extent(&Value::Object(row.clone())).unwrap(),
            7
        );
        assert_eq!(wire::RESULT_INDEX_RECORDS, wire::RESULT_INDEX_BYTES / 7);
        row.insert("coordinate", Value::String("x".repeat(65536)));
        row.insert("bits", Value::Bytes(Bytes::from(vec![0; 8])));
        row.insert(
            "projection",
            Value::Number(surrealdb::types::Number::Float(0.0)),
        );
        let mut envelope = Object::new();
        envelope.insert(
            "cells",
            Value::Array(vec![Value::Object(row.clone())].into()),
        );
        envelope.insert("outputs", Value::Array(vec![Value::Object(row)].into()));
        assert!(
            result_metadata_extent(&Value::Object(envelope)).unwrap() > EXECUTION_METADATA_BYTES
        );
    }
}
