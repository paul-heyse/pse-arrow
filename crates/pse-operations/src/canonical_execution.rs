// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fenced durable execution and immutable closed result selections.

use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, REQUEST_TIMEOUT, bounded_query, protected_query},
    canonical_codec,
    canonical_script::{Envelope, GuardedScript, ResultSlot},
    generated::surreal as wire,
};
use pse_ids::{Frame, FramedHasher};
#[path = "canonical_payload.rs"]
mod payload;
pub use payload::{PayloadLocator, result_payload_locator};
pub use pse_model::generated::runtime::{
    canonical_attempts::Row as CanonicalAttempt, canonical_result_batches::Row as ResultBatch,
    canonical_result_manifests::Row as ResultManifest, canonical_runs::Row as CanonicalRun,
};
use pse_model::generated::runtime::{
    canonical_result_block_outputs::Row as ResultOutput,
    canonical_result_blocks::Row as ResultBlock, canonical_result_sets::Row as ResultSet,
};
use serde::{Deserialize, Serialize};
use std::{future::IntoFuture, time::Duration};
use surrealdb_types::{Bytes, Object, Value};

#[cfg(all(test, feature = "canonical-tests"))]
tokio::task_local! { static APPEND_GROUP_REPLY_LOSS: bool; }

#[cfg(all(test, feature = "canonical-tests"))]
#[derive(Clone, Copy, Debug)]
enum ExecutionAckFault {
    Missing,
    Malformed,
}
#[cfg(all(test, feature = "canonical-tests"))]
tokio::task_local! {
    static EXECUTION_ACK_FAULT: (&'static str, ExecutionAckFault);
    static EXECUTION_ACK_READBACK_FAILURE: bool;
}

// Decode only after native statement/commit completion. Controlled faults alter
// the returned ACK, while genuine server effects and recovery reads remain real.
fn take_execution_ack(
    owner: &'static str,
    response: &mut crate::transport::client::IndexedResults,
) -> Result<Object, CanonicalError> {
    let row = response.take::<Option<Object>>(0)?;
    #[cfg(all(test, feature = "canonical-tests"))]
    let row = match EXECUTION_ACK_FAULT
        .try_with(|(selected, fault)| (*selected == owner).then_some(*fault))
        .ok()
        .flatten()
    {
        Some(ExecutionAckFault::Missing) => None,
        Some(ExecutionAckFault::Malformed) => Some(Object::new()),
        None => row,
    };
    let _ = owner;
    row.ok_or(CanonicalError::IncompleteResponse)
}

/// Exact original run/attempt facts without execution requests, attestations or scientific completion.
#[derive(Clone, Debug)]
pub struct CanonicalResultOrigin {
    /// Exact attempt identity.
    pub key: String,
    /// Exact original run.
    pub run: String,
    /// Original source revision.
    pub source_revision: String,
    /// Original attempt generation.
    pub generation: u64,
    /// Original lifecycle phase.
    pub phase: String,
    /// Original terminal outcome when present.
    pub outcome: Option<String>,
    /// Original closed manifest locator; its payload is not read.
    pub closed_manifest: Option<String>,
}
const RESULT_ORIGIN_SQL: &str = "fn::pse_history_v1::check_identity($pse_rpc_expires_at,'attempt',$attempt); LET $a=SELECT key,run,generation,phase,outcome,closed_manifest FROM ONLY type::record('canonical_attempts',$attempt); IF $a!=NONE { fn::pse_history_v1::run($pse_rpc_expires_at,$a.run); }; LET $r=SELECT key,revision FROM ONLY type::record('canonical_runs',$run); IF $a=NONE OR $r=NONE { RETURN NONE; }; RETURN {key:$a.key,run:$a.run,source_revision:$r.revision,generation:$a.generation,phase:$a.phase,outcome:$a.outcome,closed_manifest:$a.closed_manifest};";
fn decode_result_origin(mut row: Object) -> Result<CanonicalResultOrigin, CanonicalError> {
    use canonical_codec::{decode_string, decode_uint, required};
    let result = CanonicalResultOrigin {
        key: decode_string(required(&mut row, "key")?)?,
        run: decode_string(required(&mut row, "run")?)?,
        source_revision: decode_string(required(&mut row, "source_revision")?)?,
        generation: decode_uint(required(&mut row, "generation")?)?,
        phase: decode_string(required(&mut row, "phase")?)?,
        outcome: row
            .remove("outcome")
            .filter(|v| !matches!(v, Value::None | Value::Null))
            .map(decode_string)
            .transpose()?,
        closed_manifest: row
            .remove("closed_manifest")
            .filter(|v| !matches!(v, Value::None | Value::Null))
            .map(decode_string)
            .transpose()?,
    };
    if !row.is_empty() {
        return Err(canonical_codec::CodecError::UnknownFields.into());
    }
    Ok(result)
}

/// Small descriptive projection of one retained attempt; excludes request and result payloads.
#[derive(Clone, Debug)]
pub struct CanonicalAttemptMetadata {
    /// Exact retained identity.
    pub key: String,
    /// Exact parent run.
    pub run: String,
    /// Original retry generation.
    pub generation: u64,
    /// Original terminal fact.
    pub terminal: bool,
    /// Original outcome when recorded.
    pub outcome: Option<String>,
    /// Exact immutable result manifest when closed.
    pub closed_manifest: Option<String>,
    /// Actual closed readable generation prefix when retained.
    pub close_generation: Option<u64>,
}
impl From<CanonicalAttempt> for CanonicalAttemptMetadata {
    fn from(row: CanonicalAttempt) -> Self {
        let terminal = row.terminal();
        Self {
            key: row.key,
            run: row.run,
            generation: row.generation,
            terminal,
            outcome: row.outcome.map(|outcome| outcome.as_str().to_owned()),
            closed_manifest: row.closed_manifest,
            close_generation: row.close_generation,
        }
    }
}
fn decode_attempt_metadata(mut row: Object) -> Result<CanonicalAttemptMetadata, CanonicalError> {
    use canonical_codec::{decode_boolean, decode_string, decode_uint, required};
    let key = decode_string(required(&mut row, "key")?)?;
    if let Some(id) = row.remove("id") {
        canonical_codec::check_record_identity(id, "canonical_attempts", &key)?;
    }
    Ok(CanonicalAttemptMetadata {
        key,
        run: decode_string(required(&mut row, "run")?)?,
        generation: decode_uint(required(&mut row, "generation")?)?,
        terminal: decode_boolean(required(&mut row, "terminal")?)?,
        outcome: row
            .remove("outcome")
            .filter(|value| !matches!(value, Value::None | Value::Null))
            .map(decode_string)
            .transpose()?,
        closed_manifest: row
            .remove("closed_manifest")
            .filter(|value| !matches!(value, Value::None | Value::Null))
            .map(decode_string)
            .transpose()?,
        close_generation: row
            .remove("close_generation")
            .filter(|value| !matches!(value, Value::None | Value::Null))
            .map(decode_uint)
            .transpose()?,
    })
}

/// Decode an acknowledged guarded refusal before the ordinary generated row codec.
/// No driver error or later cancellation observation grants this classification.
pub(crate) fn writer_response(mut row: Object) -> Result<Object, CanonicalError> {
    let Some(tag) = row.remove("pse_writer") else {
        return Ok(row);
    };
    if canonical_codec::decode_string(tag)? != "study-cancelled" {
        return Err(canonical_codec::CodecError::Domain("writer refusal tag").into());
    }
    let run = canonical_codec::decode_string(canonical_codec::required(&mut row, "run")?)?;
    let attempt = canonical_codec::decode_string(canonical_codec::required(&mut row, "attempt")?)?;
    let generation =
        canonical_codec::decode_uint(canonical_codec::required(&mut row, "generation")?)?;
    let operation =
        canonical_codec::decode_string(canonical_codec::required(&mut row, "operation")?)?;
    if !row.is_empty() {
        return Err(canonical_codec::CodecError::UnknownFields.into());
    }
    Err(CanonicalError::StudyCancellation {
        run,
        attempt,
        generation,
        operation,
    })
}

/// Independently submitted scientific block limit, below the protocol envelope.
pub const RESULT_BATCH_BYTES: usize = wire::RESULT_BLOCK_BYTES;
/// Completion and closed descriptors are bounded metadata, never trajectories.
pub const EXECUTION_METADATA_BYTES: usize = wire::RESULT_INDEX_BYTES;
pub(crate) const RESULT_SETS: usize = 256;

/// Conservative CBOR extent for the canonical codec's metadata budget.
/// Decimal uses a fixed bound covering its tag and the longest signed decimal text;
/// this is independent of native RPC framing and live allocation ownership.
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
        Value::Number(surrealdb_types::Number::Decimal(_)) => Ok(40),
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

/// Derive dense output metadata extent through the canonical codec owner.
pub fn result_output_metadata_extent(output: &ResultOutput) -> Result<usize, CanonicalError> {
    result_metadata_extent(&Value::Object(wire::encode_canonical_result_block_outputs(
        output,
    )?))
}

/// Exact selected scientific input and executable provenance, before native work.
#[derive(Clone, Debug)]
pub struct RunRequest {
    /// Stable semantic run and begin-effect identity.
    pub key: pse_model::generated::canonical::RunKey,
    /// Governing execution purpose, fixed before registration.
    pub purpose: pse_model::generated::enums::CanonicalRunPurpose,
    /// Explicit lazy-admission owner; both association coordinates occur together.
    pub admission_study: Option<pse_model::generated::canonical::StudyKey>,
    /// Exact immutable point selected by the admission owner.
    pub admission_point: Option<pse_model::generated::canonical::StudyPointKey>,
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
    use crate::canonical::CanonicalOptions;
    use std::path::Path;

    async fn fixture() -> (CanonicalStore, String, crate::canonical::Revision) {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_execution_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let revision = store
            .edit("problem", None, "initial-source", &[])
            .await
            .unwrap();
        (store, options.database, revision)
    }
    async fn remove(store: &CanonicalStore, database: &str) {
        assert_eq!(store.database(), database);
        store.remove_isolated_fixture().await.unwrap();
    }
    async fn run(
        store: &CanonicalStore,
        revision: &crate::canonical::Revision,
        key: &str,
    ) -> CanonicalRun {
        store
            .begin_run(&RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
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
    async fn canonical_empty_payload_refuses_before_all_client_and_server_effects() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "empty-refusal").await;
        let fence = store
            .claim_run(
                "empty-refusal",
                "claim-empty-refusal",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        assert!(ResultPayload::new(Vec::new()).is_err());
        assert!(ResultPayload::new_owned(Vec::new(), test_payload_owner(1024 * 1024)).is_err());
        assert!(
            ResultPayload::from_encoded_owned(
                Vec::new(),
                crate::canonical_files::OPAQUE_BYTES,
                "",
                0,
                0,
                "empty-body",
                test_payload_owner(1024 * 1024)
            )
            .is_err()
        );
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (0, 0)
        );
        // Deliberately bypass the Rust constructor in this isolated fixture to
        // prove the native function refuses zero bytes before publication too.
        let mut item = canonical_result_admission_unit::append_item(&fence, "empty", 0, vec![1]);
        let Value::Object(mut batch) = item.item.remove("batch").unwrap() else {
            panic!("batch object")
        };
        batch.insert("payload_bytes", canonical_codec::encode_uint(0).unwrap());
        let Value::Object(mut body) = item.item.remove("inline").unwrap() else {
            panic!("body object")
        };
        body.insert("payload_bytes", canonical_codec::encode_uint(0).unwrap());
        body.insert("body", Value::Bytes(Bytes::from(Vec::<u8>::new())));
        let operation = "raw-empty-append";
        assert!(store.protected_execution_query(fence.run(),"canonical_execution::empty_fixture",||Ok(store.db.query("RETURN fn::pse_execution_v1::append($pse_rpc_expires_at,$run,$attempt,$generation,$operation,$request,$set,$batch,$inline,$block,$outputs);")
            .bind(("run",fence.run().to_owned())).bind(("attempt",fence.attempt().to_owned())).bind(("generation",canonical_codec::encode_uint(fence.generation())?))
            .bind(("operation",operation.to_owned())).bind(("request",item.item.get("request").unwrap().clone()))
            .bind(("set",item.item.get("set").unwrap().clone())).bind(("batch",batch.clone())).bind(("inline",body.clone()))
            .bind(("block",item.item.get("block").unwrap().clone())).bind(("outputs",item.item.get("outputs").unwrap().clone())))).await.is_err());
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (0, 0)
        );
        let mut rows=bounded_query(store.db.query("RETURN {batches:(SELECT key FROM canonical_result_batches WHERE attempt=$attempt),sets:(SELECT key FROM canonical_result_sets WHERE attempt=$attempt),operation:(SELECT key FROM ONLY type::record('canonical_execution_operations',$operation))};").bind(("attempt",fence.attempt().to_owned())).bind(("operation",operation.to_owned()))).await.unwrap();
        let envelope = rows.take::<Option<Object>>(0).unwrap().unwrap();
        assert_eq!(
            envelope.get("batches"),
            Some(&Value::Array(Vec::<Value>::new().into()))
        );
        assert_eq!(
            envelope.get("sets"),
            Some(&Value::Array(Vec::<Value>::new().into()))
        );
        assert!(matches!(
            envelope.get("operation"),
            None | Some(Value::None) | Some(Value::Null)
        ));
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_missing_and_malformed_committed_acks_settle_exact_native_effects() {
        for fault in [ExecutionAckFault::Missing, ExecutionAckFault::Malformed] {
            let (store, database, revision) = fixture().await;
            let request = RunRequest {
                key: "ack-normalization".into(),
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            };
            EXECUTION_ACK_FAULT
                .scope(("execution-registration", fault), store.begin_run(&request))
                .await
                .unwrap();
            let fence = EXECUTION_ACK_FAULT
                .scope(
                    ("execution-claim", fault),
                    store.claim_run(
                        "ack-normalization",
                        "claim-ack-normalization",
                        "worker",
                        Duration::from_secs(60),
                    ),
                )
                .await
                .unwrap();
            EXECUTION_ACK_FAULT
                .scope(
                    ("execution-append", fault),
                    store.append_result_batch(
                        &fence,
                        "single-ack-normalization",
                        "single",
                        0,
                        ResultPayload::new(vec![7]).unwrap(),
                        1,
                    ),
                )
                .await
                .unwrap();
            EXECUTION_ACK_FAULT
                .scope(
                    ("execution-append-group", fault),
                    store.append_result_blocks(
                        &fence,
                        vec![
                            canonical_result_admission_unit::append_item(
                                &fence,
                                "group",
                                0,
                                vec![8],
                            ),
                            canonical_result_admission_unit::append_item(
                                &fence,
                                "group",
                                1,
                                vec![9],
                            ),
                        ],
                    ),
                )
                .await
                .unwrap();
            EXECUTION_ACK_FAULT
                .scope(
                    ("execution-empty", fault),
                    store.declare_empty_result_set(&fence, "empty-ack-normalization", "empty"),
                )
                .await
                .unwrap();
            let closed = EXECUTION_ACK_FAULT
                .scope(
                    ("execution-close", fault),
                    store.close_result_ingestion(&fence, "close-ack-normalization"),
                )
                .await
                .unwrap();
            let manifest = EXECUTION_ACK_FAULT
                .scope(
                    ("execution-reconcile", fault),
                    store.reconcile_closed_attempt(&closed),
                )
                .await
                .unwrap();
            // SAFETY: this native transport fixture asserts only a partial outcome.
            let terminal = EXECUTION_ACK_FAULT
                .scope(("execution-seal", fault), unsafe {
                    store.seal_attempt(
                        &manifest,
                        "seal-ack-normalization",
                        TerminalClass::Partial,
                        &[],
                        SeedEligibility::Unavailable,
                    )
                })
                .await
                .unwrap();
            assert_eq!(terminal.key, fence.attempt());
            assert_eq!(
                crate::testing::result_payload_counts(&store, &fence)
                    .await
                    .unwrap(),
                (3, 0)
            );
            let read = store
                .read_results(fence.run(), fence.attempt())
                .await
                .unwrap();
            for (name, ordinal, expected) in [("single", 0, 7), ("group", 0, 8), ("group", 1, 9)] {
                assert_eq!(
                    store
                        .result_payload(&read, &result_set_key(fence.attempt(), name), ordinal)
                        .await
                        .unwrap()
                        .batch
                        .payload
                        .as_slice(),
                    [expected]
                );
            }
            drop(read);
            crate::testing::drain_result_releases(&store).await.unwrap();
            run(&store, &request.revision, "cancel-ack-normalization").await;
            let cancelled = EXECUTION_ACK_FAULT
                .scope(
                    ("execution-cancel", fault),
                    store.cancel_run("cancel-ack-normalization", "cancel-ack-normalization"),
                )
                .await
                .unwrap();
            assert!(cancelled.cancelled);
            remove(&store, &database).await;
        }
    }

    #[tokio::test]
    async fn canonical_bad_committed_ack_and_failed_native_readback_keep_both_causes() {
        for fault in [ExecutionAckFault::Missing, ExecutionAckFault::Malformed] {
            let (store, database, revision) = fixture().await;
            run(&store, &revision, "dual-ack-failure").await;
            let fence = store
                .claim_run(
                    "dual-ack-failure",
                    "claim-dual-ack-failure",
                    "worker",
                    Duration::from_secs(60),
                )
                .await
                .unwrap();
            let error = EXECUTION_ACK_FAULT
                .scope(
                    ("execution-append", fault),
                    EXECUTION_ACK_READBACK_FAILURE.scope(
                        true,
                        store.append_result_batch(
                            &fence,
                            "dual-ack-append",
                            "values",
                            0,
                            ResultPayload::new(vec![7]).unwrap(),
                            1,
                        ),
                    ),
                )
                .await
                .unwrap_err();
            assert!(error.effect_is_unknown());
            assert!(
                matches!(error,CanonicalError::SettlementUnresolved {owner:"execution-append",operation,source,readback:Some(readback),request} if operation=="dual-ack-append" && !request.is_empty() && (match fault {ExecutionAckFault::Missing=>matches!(*source,CanonicalError::IncompleteResponse),ExecutionAckFault::Malformed=>matches!(*source,CanonicalError::OperationReused)}) && matches!(*readback,CanonicalError::Driver(ref driver) if driver.is_thrown()))
            );
            // The write really committed before either fault; exact retry observes
            // the original operation and never creates another body or reservation.
            assert_eq!(
                crate::testing::result_payload_counts(&store, &fence)
                    .await
                    .unwrap(),
                (1, 0)
            );
            store
                .append_result_batch(
                    &fence,
                    "dual-ack-append",
                    "values",
                    0,
                    ResultPayload::new(vec![7]).unwrap(),
                    1,
                )
                .await
                .unwrap();
            assert_eq!(
                crate::testing::result_payload_counts(&store, &fence)
                    .await
                    .unwrap(),
                (1, 0)
            );
            remove(&store, &database).await;
        }
    }

    #[tokio::test]
    async fn canonical_inline_shared_body_lost_ack_conflict_and_mixed_sealed_owners() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "placement").await;
        let fence = store
            .claim_run(
                "placement",
                "claim-placement",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let bytes = vec![7; pse_schema::catalog::substrate::RESULT_INLINE_BYTES];
        let group = || {
            (0..2)
                .map(|ordinal| {
                    ResultBlockAppend::prepare(
                        &fence,
                        &format!("inline-member-{ordinal}"),
                        "opaque",
                        ordinal,
                        ResultPayload::from_encoded_owned(
                            bytes.clone(),
                            crate::canonical_files::OPAQUE_BYTES,
                            "",
                            0,
                            0,
                            "original-complete-body",
                            test_payload_owner(1024 * 1024),
                        )
                        .unwrap(),
                        1,
                        None,
                        &[],
                        None,
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>()
        };
        let acks = APPEND_GROUP_REPLY_LOSS
            .scope(true, store.append_result_blocks(&fence, group()))
            .await
            .unwrap();
        let receipts = acks.into_iter().map(|ack| ack.receipt).collect::<Vec<_>>();
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (1, 0)
        );
        let replay = store.append_result_blocks(&fence, group()).await.unwrap();
        assert_eq!(
            replay
                .into_iter()
                .map(|ack| ack.receipt)
                .collect::<Vec<_>>(),
            receipts
        );
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (1, 0)
        );
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM canonical_inline_payloads WHERE attempt=$attempt;")
                .bind(("attempt", fence.attempt().to_owned())),
        )
        .await
        .unwrap();
        let bodies = response.take::<Vec<Object>>(0).unwrap();
        assert_eq!(bodies.len(), 1);
        assert_eq!(
            wire::decode_canonical_inline_payloads(bodies[0].clone())
                .unwrap()
                .body
                .as_slice(),
            bytes
        );
        assert!(matches!(
            store
                .append_result_batch(
                    &fence,
                    "inline-member-0",
                    "opaque",
                    0,
                    ResultPayload::new(vec![8; bytes.len()]).unwrap(),
                    1
                )
                .await,
            Err(CanonicalError::OperationReused)
        ));
        store
            .append_result_batch(
                &fence,
                "large-body",
                "large",
                0,
                ResultPayload::new(vec![9; bytes.len() + 1]).unwrap(),
                1,
            )
            .await
            .unwrap();
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (1, 1)
        );
        let closed = store
            .close_result_ingestion(&fence, "close-placement")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this transport fixture makes only the explicit partial claim.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "seal-placement",
                    TerminalClass::Partial,
                    &[],
                    SeedEligibility::Unavailable,
                )
                .await
                .unwrap();
        }
        let read = store
            .read_results(fence.run(), fence.attempt())
            .await
            .unwrap();
        let inline_set = result_set_key(fence.attempt(), "opaque");
        let sealed_set = result_set_key(fence.attempt(), "large");
        let inline = store.result_payload(&read, &inline_set, 0).await.unwrap();
        let sealed = store.result_payload(&read, &sealed_set, 0).await.unwrap();
        assert_eq!(inline.batch.payload.as_slice(), bytes);
        assert_eq!(sealed.batch.payload.as_slice(), vec![9; bytes.len() + 1]);
        assert!(store.forget_run_results(fence.run()).await.is_err());
        drop((read, sealed));
        assert!(
            store.forget_run_results(fence.run()).await.is_err(),
            "returned inline payload retains the actual read ticket"
        );
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (1, 1)
        );
        drop(inline);
        crate::testing::drain_result_releases(&store).await.unwrap();
        store.forget_run_results(fence.run()).await.unwrap();
        let mut complete = false;
        for _ in 0..32 {
            let mut response=bounded_query(store.db.query("SELECT key FROM canonical_result_batches WHERE attempt=$attempt AND placement='inline';").bind(("attempt",fence.attempt().to_owned()))).await.unwrap();
            let members = response.take::<Vec<Object>>(0).unwrap();
            assert_eq!(
                crate::testing::result_payload_counts(&store, &fence)
                    .await
                    .unwrap()
                    .0,
                u64::from(!members.is_empty())
            );
            if store
                .reclaim_result_page(fence.run())
                .await
                .unwrap()
                .complete
            {
                complete = true;
                break;
            }
        }
        assert!(complete);
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap()
                .0,
            0
        );
        remove(&store, &database).await;
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_inline_admitted_ipc_members_share_one_actual_body_and_no_files() {
        use pse_relations::generated::runtime::fit_parameters;
        use pse_relations::result_batch::AdmittedResultFile;
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "admitted-inline").await;
        let fence = store
            .claim_run(
                "admitted-inline",
                "claim-admitted-inline",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let registry = pse_schema::registry().unwrap();
        let validation = pse_relations::validate::ValidationContext::new(
            registry,
            pse_engine::validation::NativeValidation(
                datafusion::prelude::SessionContext::new().state(),
            ),
        );
        let mut builder = fit_parameters::Builder::new(&validation).unwrap();
        builder
            .push(fit_parameters::Row {
                run_id: pse_model::generated::identities::RunId::from_bytes([1; 16]),
                parameter_id: pse_ids::SemanticId::from_bytes([2; 16]),
                fixed: false,
                value: Some(-0.0),
                unit_id: pse_ids::SemanticId::NIL,
                scale: 1.0,
                at_bound: None,
            })
            .unwrap();
        let batch = builder.finish().unwrap();
        let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> =
            std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(16 * 1024 * 1024));
        let admitted = std::sync::Arc::new(
            AdmittedResultFile::encode(
                registry,
                &[batch.clone(), batch],
                pse_schema::catalog::substrate::RESULT_INLINE_BYTES,
                &pool,
                &pse_columnar::CancellationToken::new(),
            )
            .unwrap(),
        );
        let relation = admitted.relation().to_string();
        let set = result_set_key(fence.attempt(), &relation);
        let group = || {
            (0..2)
                .map(|ordinal| {
                    let payload = ResultPayload::from_admitted_encoded_owned(
                        admitted.clone(),
                        ordinal,
                        "original-ipc-body",
                        test_payload_owner(1024 * 1024),
                    )
                    .unwrap();
                    let key = result_batch_key(fence.attempt(), &set, ordinal);
                    let block = ResultBlock {
                        key: key.clone(),
                        batch: key,
                        result_set: set.clone(),
                        output: relation.clone(),
                        partition: "0".into(),
                        ordinal,
                        start: ordinal,
                        end: ordinal + 1,
                        rows: 1,
                        columns: 7,
                        coordinate_min: None,
                        coordinate_max: None,
                        payload_bytes: payload.byte_len(),
                        payload_digest: payload.digest().into(),
                        statistics: None,
                        interpretation: wire::INTERPRETATION.into(),
                    };
                    ResultBlockAppend::new(
                        &fence,
                        &format!("admitted-member-{ordinal}"),
                        &relation,
                        ordinal,
                        payload,
                        1,
                        &block,
                        &[],
                        test_payload_owner(1024 * 1024),
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>()
        };
        APPEND_GROUP_REPLY_LOSS
            .scope(true, store.append_result_blocks(&fence, group()))
            .await
            .unwrap();
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (1, 0)
        );
        store.append_result_blocks(&fence, group()).await.unwrap();
        assert_eq!(
            crate::testing::result_payload_counts(&store, &fence)
                .await
                .unwrap(),
            (1, 0)
        );
        let mut response=bounded_query(store.db.query("SELECT inline_key,payload_index FROM canonical_result_batches WHERE attempt=$attempt ORDER BY ordinal;").bind(("attempt",fence.attempt().to_owned()))).await.unwrap();
        let rows = response.take::<Vec<Object>>(0).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("inline_key"), rows[1].get("inline_key"));
        assert_ne!(rows[0].get("payload_index"), rows[1].get("payload_index"));
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_append_group_lost_reply_settles_exact_receipts_once_and_replays_after_cancel()
     {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "batch-replay").await;
        let fence = store
            .claim_run(
                "batch-replay",
                "claim-group",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let group = || {
            vec![
                canonical_result_admission_unit::append_item(&fence, "values", 0, vec![1]),
                canonical_result_admission_unit::append_item(&fence, "other", 0, vec![2]),
                canonical_result_admission_unit::append_item(&fence, "values", 1, vec![3]),
            ]
        };
        let acks = APPEND_GROUP_REPLY_LOSS
            .scope(true, store.append_result_blocks(&fence, group()))
            .await
            .unwrap();
        assert_eq!(acks.len(), 3);
        let receipts = acks
            .iter()
            .map(|ack| ack.receipt.clone())
            .collect::<Vec<_>>();
        let mut accumulated = ResultAcknowledgments::new(&fence);
        for ack in acks {
            accumulated.append(ack).unwrap();
        }
        assert_eq!(accumulated.rows("values"), 2);
        assert_eq!(accumulated.rows("other"), 1);
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM canonical_result_sets WHERE attempt=$attempt ORDER BY key;")
                .bind(("attempt", fence.attempt().to_owned())),
        )
        .await
        .unwrap();
        let sets = response
            .take::<Vec<Object>>(0)
            .unwrap()
            .into_iter()
            .map(|row| wire::decode_canonical_result_sets(row).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(sets.iter().map(|set| set.next_ordinal).sum::<u64>(), 3);
        store.cancel_run(fence.run(), "cancel-group").await.unwrap();
        let replay = store.append_result_blocks(&fence, group()).await.unwrap();
        assert_eq!(
            replay
                .iter()
                .map(|ack| ack.receipt.clone())
                .collect::<Vec<_>>(),
            receipts
        );
        // All-replay remains authenticated by original receipts, but mixing an
        // unacknowledged new append cannot borrow that earlier permission.
        let mut mixed = group();
        let new = canonical_result_admission_unit::append_item(&fence, "later", 0, vec![4]);
        let missing = new.acknowledgment.key.clone();
        mixed.push(new);
        assert!(store.append_result_blocks(&fence, mixed).await.is_err());
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY type::record('canonical_result_batches',$key);")
                .bind(("key", missing)),
        )
        .await
        .unwrap();
        assert!(response.take::<Option<Object>>(0).unwrap().is_none());
        remove(&store, &database).await;
    }
    #[tokio::test]
    async fn canonical_append_group_late_sequence_refusal_rolls_back_all_new_blocks() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "batch-rollback").await;
        let fence = store
            .claim_run(
                "batch-rollback",
                "claim-rollback",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let first = canonical_result_admission_unit::append_item(&fence, "values", 0, vec![1]);
        let key = first.acknowledgment.key.clone();
        // Different set: only authentic server state can establish that ordinal7
        // is a gap. The first append must be rolled back with that later refusal.
        let gap = canonical_result_admission_unit::append_item(&fence, "other", 7, vec![2]);
        assert!(
            store
                .append_result_blocks(&fence, vec![first, gap])
                .await
                .is_err()
        );
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY type::record('canonical_result_batches',$key);")
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert!(response.take::<Option<Object>>(0).unwrap().is_none());
        let valid = vec![
            canonical_result_admission_unit::append_item(&fence, "values", 0, vec![1]),
            canonical_result_admission_unit::append_item(&fence, "other", 0, vec![2]),
        ];
        assert_eq!(
            store
                .append_result_blocks(&fence, valid)
                .await
                .unwrap()
                .len(),
            2
        );
        remove(&store, &database).await;
    }
    async fn completion_ack(
        store: &CanonicalStore,
        fence: &AttemptFence,
        acks: &mut ResultAcknowledgments,
    ) -> Vec<u8> {
        let payload = b"original completion fixture";
        acks.append(
            store
                .append_result_batch(
                    fence,
                    &format!("completion:{}", fence.attempt()),
                    "__completion",
                    0,
                    ResultPayload::new((payload).to_vec()).unwrap(),
                    payload.len() as u64,
                )
                .await
                .unwrap(),
        )
        .unwrap();
        acks.complete("__completion").unwrap();
        serde_json::to_vec(&serde_json::json!({"version":1,"batch_count":1,"payload_bytes":payload.len(),"digest":result_payload_digest(payload),"inline":null})).unwrap()
    }
    #[tokio::test]
    async fn canonical_acknowledged_close_retains_empty_demand_and_actual_seal_headers() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "ack-empty").await;
        let fence = store
            .claim_run("ack-empty", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let mut acks = ResultAcknowledgments::new(&fence);
        acks.expect("demanded-empty").unwrap();
        acks.empty(
            store
                .declare_empty_result_set(&fence, "empty", "demanded-empty")
                .await
                .unwrap(),
        )
        .unwrap();
        acks.complete("demanded-empty").unwrap();
        let receipt = completion_ack(&store, &fence, &mut acks).await;
        let manifest = store
            .close_acknowledged(&fence, "close", acks)
            .await
            .unwrap();
        assert_eq!(manifest.closed.authority, fence.generation());
        let descriptors = decode_result_descriptors(manifest.row()).unwrap();
        let empty = descriptors
            .iter()
            .find(|set| set.name == "demanded-empty")
            .unwrap();
        assert_eq!((empty.batch_count, empty.row_count), (0, 0));
        // SAFETY: this fixture owns the exact finite bytes and asserts only partial availability.
        let (attempt, header) = unsafe {
            store
                .seal_acknowledged(
                    &manifest,
                    "seal",
                    TerminalClass::Partial,
                    &receipt,
                    SeedEligibility::Unavailable,
                )
                .await
                .unwrap()
        }
        .into_parts();
        assert_eq!(
            header.terminal_attempt.as_deref(),
            Some(attempt.key.as_str())
        );
        assert_eq!(attempt.completion.unwrap().as_slice(), receipt);
        let read = store
            .read_results(fence.run(), fence.attempt())
            .await
            .unwrap();
        assert_eq!(
            read.sets()
                .iter()
                .find(|set| set.name == "demanded-empty")
                .unwrap(),
            empty
        );
        assert!(store.result_payload(&read, &empty.key, 0).await.is_err());
        drop(read);
        remove(&store, &database).await;
    }
    #[tokio::test]
    async fn canonical_acknowledged_close_refuses_unobserved_sets_counts_and_missing_receipt() {
        for mutation in 0..4 {
            let (store, database, revision) = fixture().await;
            run(&store, &revision, "ack-coverage").await;
            let fence = store
                .claim_run("ack-coverage", "claim", "worker", Duration::from_secs(60))
                .await
                .unwrap();
            let mut acks = ResultAcknowledgments::new(&fence);
            completion_ack(&store, &fence, &mut acks).await;
            let first = store
                .append_result_batch(
                    &fence,
                    "value-0",
                    "values",
                    0,
                    ResultPayload::new([1].to_vec()).unwrap(),
                    1,
                )
                .await
                .unwrap();
            let set = first.result_set.clone();
            acks.append(first).unwrap();
            acks.complete("values").unwrap();
            match mutation {
                0 => {
                    store
                        .append_result_batch(
                            &fence,
                            "extra",
                            "unadvertised",
                            0,
                            ResultPayload::new([9].to_vec()).unwrap(),
                            1,
                        )
                        .await
                        .unwrap();
                }
                1 => {
                    store
                        .append_result_batch(
                            &fence,
                            "value-1",
                            "values",
                            1,
                            ResultPayload::new([2].to_vec()).unwrap(),
                            1,
                        )
                        .await
                        .unwrap();
                }
                2 => {
                    bounded_query(
                        store
                            .db
                            .query("DELETE ONLY type::record('canonical_result_sets',$key);")
                            .bind(("key", set)),
                    )
                    .await
                    .unwrap();
                }
                _ => {
                    bounded_query(store.db.query(
                        "DELETE ONLY type::record('canonical_execution_operations','value-0');",
                    ))
                    .await
                    .unwrap();
                }
            }
            assert!(
                store
                    .close_acknowledged(&fence, "close", acks)
                    .await
                    .is_err()
            );
            let current = store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap();
            assert!(current.ingestion_open() && !current.closed() && !current.terminal());
            assert!(current.closed_manifest.is_none());
            remove(&store, &database).await;
        }
    }
    #[tokio::test]
    async fn canonical_progress_uncertainty_readback_allows_only_bounded_optional_tail() {
        for mutation in 0..3 {
            let (store, database, revision) = fixture().await;
            run(&store, &revision, "ack-progress").await;
            let fence = store
                .claim_run("ack-progress", "claim", "worker", Duration::from_secs(60))
                .await
                .unwrap();
            let mut acks = ResultAcknowledgments::new(&fence);
            completion_ack(&store, &fence, &mut acks).await;
            acks.append(
                store
                    .append_result_batch(
                        &fence,
                        "progress-0",
                        "__progress",
                        0,
                        ResultPayload::new((b"[]").to_vec()).unwrap(),
                        1,
                    )
                    .await
                    .unwrap(),
            )
            .unwrap();
            acks.complete("__progress").unwrap();
            assert_eq!(acks.rows("__progress"), 1);
            // An append may commit without its original producer observing the ACK.
            store
                .append_result_batch(
                    &fence,
                    "progress-1",
                    "__progress",
                    1,
                    ResultPayload::new((b"[]").to_vec()).unwrap(),
                    2,
                )
                .await
                .unwrap();
            if mutation == 1 {
                store
                    .append_result_batch(
                        &fence,
                        "extra",
                        "unadvertised",
                        0,
                        ResultPayload::new([9].to_vec()).unwrap(),
                        1,
                    )
                    .await
                    .unwrap();
            }
            let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
            let result = store
                .reconcile_progress_uncertainty(&closed, acks, if mutation == 2 { 2 } else { 3 })
                .await;
            if mutation == 0 {
                let manifest = result.unwrap();
                let descriptors = decode_result_descriptors(manifest.row()).unwrap();
                let actual = descriptors
                    .iter()
                    .find(|set| set.name == "__progress")
                    .unwrap();
                assert_eq!((actual.batch_count, actual.row_count), (2, 3));
            } else {
                assert!(result.is_err());
            }
            assert!(
                !store
                    .canonical_attempt(fence.attempt())
                    .await
                    .unwrap()
                    .unwrap()
                    .terminal()
            );
            remove(&store, &database).await;
        }
    }
    #[tokio::test]
    async fn canonical_acknowledged_close_requires_complete_actual_seed_inventory() {
        for mode in 0..3 {
            let (store, database, revision) = fixture().await;
            let run = run(&store, &revision, "ack-seeds").await;
            let fence = store
                .claim_run("ack-seeds", "claim", "worker", Duration::from_secs(60))
                .await
                .unwrap();
            let mut acks = ResultAcknowledgments::new(&fence);
            completion_ack(&store, &fence, &mut acks).await;
            let payload = b"seed fixture";
            let batch = store
                .append_result_batch(
                    &fence,
                    "seed-chunk",
                    "__seeds",
                    0,
                    ResultPayload::new((payload).to_vec()).unwrap(),
                    payload.len() as u64,
                )
                .await
                .unwrap();
            let row = pse_model::generated::runtime::canonical_result_seeds::Row {
                key: "seed-fixture".into(),
                batch: batch.key.clone(),
                first_ordinal: 0,
                result_set: batch.result_set.clone(),
                attempt: fence.attempt().into(),
                run: fence.run().into(),
                layout: "layout".into(),
                preparation: "preparation".into(),
                profile: "profile".into(),
                data: "data".into(),
                backend: "constant".into(),
                step: 0,
                batch_count: 1,
                payload_bytes: payload.len() as u64,
                digest: result_payload_digest(payload),
                run_sequence: run.sequence,
                attempt_generation: fence.generation(),
            };
            acks.append(batch).unwrap();
            if mode != 0 {
                let registered = store
                    .register_result_seed(&fence, "seed-register", &row)
                    .await
                    .unwrap();
                if mode == 2 {
                    acks.seed(registered).unwrap();
                }
            }
            acks.complete("__seeds").unwrap();
            let closed = store.close_acknowledged(&fence, "close", acks).await;
            if mode == 2 {
                assert!(closed.is_ok());
            } else {
                assert!(closed.is_err());
            }
            assert!(
                !store
                    .canonical_attempt(fence.attempt())
                    .await
                    .unwrap()
                    .unwrap()
                    .terminal()
            );
            remove(&store, &database).await;
        }
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
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
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
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let points = (0..2)
            .map(|index| NewOccurrence {
                projection: None,
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
            .append_result_batch(
                first,
                "before-cancel",
                "observations",
                0,
                ResultPayload::new([7].to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(first, "before-cancel-close")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        store.cancel_study("cancel-study").await.unwrap();
        assert!(
            matches!(store.renew_attempt(first, Duration::from_secs(60)).await,
            Err(CanonicalError::StudyCancellation { run, attempt, generation, operation })
            if run == first.run() && attempt == first.attempt() && generation == first.generation() && operation == "renew")
        );
        // An acknowledged original append/close survives subsequent writer refusal.
        assert!(
            store
                .append_result_batch(
                    first,
                    "before-cancel",
                    "observations",
                    0,
                    ResultPayload::new([7].to_vec()).unwrap(),
                    1
                )
                .await
                .is_ok()
        );
        assert!(
            store
                .close_result_ingestion(first, "before-cancel-close")
                .await
                .is_ok()
        );
        assert!(
            // SAFETY: the isolated fixture owns these synthetic observations; this cancelled stale capability must refuse success.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        "stale-success",
                        TerminalClass::Succeeded,
                        &[9],
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .is_err()
        );
        assert!(
            // SAFETY: the fixture supplies its actual cancellation; this stale capability must still be refused.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        "stale-cancel",
                        TerminalClass::Cancelled,
                        &[9],
                        SeedEligibility::Unavailable,
                    )
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
                        ResultPayload::new([8].to_vec()).unwrap(),
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
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .unwrap();
            assert_eq!(
                attempt.outcome.as_ref().map(|value| value.as_str()),
                Some("cancelled")
            );
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
                .seal_attempt(
                    &manifest,
                    operation,
                    outcome,
                    &[9],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap()
    }

    async fn seed_headers(
        store: &CanonicalStore,
        revision: &crate::canonical::Revision,
        key: &str,
        count: u64,
    ) -> (
        AttemptFence,
        Vec<pse_model::generated::runtime::canonical_result_seeds::Row>,
    ) {
        let run = run(store, revision, key).await;
        let fence = store
            .claim_run(
                key,
                &format!("claim-{key}"),
                "worker",
                Duration::from_secs(600),
            )
            .await
            .unwrap();
        let payload = [7, 9];
        let batch = store
            .append_result_batch(
                &fence,
                &format!("batch-{key}"),
                "__seeds",
                0,
                ResultPayload::new(payload.to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        let mut seeds = Vec::new();
        for step in 0..count {
            let row = pse_model::generated::runtime::canonical_result_seeds::Row {
                key: format!("seed-{key}-{step:03}"),
                batch: batch.key.clone(),
                result_set: batch.result_set.clone(),
                attempt: fence.attempt().into(),
                run: key.into(),
                layout: "layout".into(),
                preparation: "preparation".into(),
                profile: "original-profile".into(),
                data: "original-data".into(),
                backend: "backend".into(),
                step,
                batch_count: 1,
                first_ordinal: 0,
                run_sequence: run.sequence,
                attempt_generation: fence.generation(),
                payload_bytes: payload.len() as u64,
                digest: result_payload_digest(&payload),
            };
            store
                .register_result_seed(&fence, &row.key, &row)
                .await
                .unwrap();
            seeds.push(row);
        }
        (fence, seeds)
    }

    async fn seal_seed_inventory(store: &CanonicalStore, fence: &AttemptFence) -> ClosedManifest {
        let closed = store
            .close_result_ingestion(fence, &format!("close-{}", fence.attempt()))
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: the fixture owns every registered synthetic seed and its complete
        // original observations in this exact frozen inventory, without native science.
        let terminal = unsafe {
            store
                .seal_attempt(
                    &manifest,
                    &format!("seal-{}", fence.attempt()),
                    TerminalClass::Partial,
                    &[9],
                    SeedEligibility::CompleteOriginalInventory,
                )
                .await
        }
        .unwrap();
        assert_eq!(
            terminal.seed_manifest.as_deref(),
            Some(manifest.row().key.as_str())
        );
        manifest
    }

    #[tokio::test]
    async fn canonical_execution_seed_eligibility_precedes_cutoff_and_retirement() {
        let (store, database, revision) = fixture().await;
        let (old, old_seeds) = seed_headers(&store, &revision, "old", 1).await;
        seal_seed_inventory(&store, &old).await;
        let (new, new_seeds) = seed_headers(&store, &revision, "new", 64).await;
        // Exactly 65 headers: LIMIT on unfiltered inventory would mask the old one.
        let candidates = store
            .result_seed_candidates("layout", "preparation", "backend", None)
            .await
            .unwrap();
        assert_eq!(candidates, old_seeds);
        assert!(
            store
                .result_seed_candidates("layout", "preparation", "backend", Some(new.attempt()))
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .result_seed_candidates("layout", "preparation", "backend", Some(old.attempt()))
                .await
                .unwrap(),
            old_seeds
        );
        let manifest = seal_seed_inventory(&store, &new).await;
        let candidates = store
            .result_seed_candidates("layout", "preparation", "backend", None)
            .await
            .unwrap();
        assert_eq!(candidates.len(), 64);
        assert_eq!(candidates.first(), new_seeds.last());
        // Exact replay after an unknown acknowledgement retains the same witness;
        // changing the eligibility under the same original intent is refused.
        // SAFETY: replay of the exact fixture-owned frozen inventory and completion.
        let replay = unsafe {
            store
                .seal_attempt(
                    &manifest,
                    &format!("seal-{}", new.attempt()),
                    TerminalClass::Partial,
                    &[9],
                    SeedEligibility::CompleteOriginalInventory,
                )
                .await
        }
        .unwrap();
        assert_eq!(
            replay.seed_manifest.as_deref(),
            Some(manifest.row().key.as_str())
        );
        assert!(
            // SAFETY: fixture-owned frozen observations and partial completion are
            // unchanged; only the replay's eligibility is deliberately inconsistent.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        &format!("seal-{}", new.attempt()),
                        TerminalClass::Partial,
                        &[9],
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .is_err()
        );
        for (layout, preparation, backend) in [
            ("other", "preparation", "backend"),
            ("layout", "other", "backend"),
            ("layout", "preparation", "other"),
        ] {
            assert!(
                store
                    .result_seed_candidates(layout, preparation, backend, None)
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
        store.forget_run_results(new.run()).await.unwrap();
        assert_eq!(
            store
                .result_seed_candidates("layout", "preparation", "backend", None)
                .await
                .unwrap(),
            old_seeds
        );
        assert!(store.read_results(new.run(), new.attempt()).await.is_err());
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_seed_witness_refuses_recovered_prefix_and_orders_ties() {
        let (store, database, revision) = fixture().await;
        let (old, seeds) = seed_headers(&store, &revision, "old", 1).await;
        let mut tied = seeds[0].clone();
        tied.key = "zz-tied-seed".into();
        // Data and solver profile are provenance, deliberately outside compatibility.
        tied.profile = "different-options".into();
        tied.data = "different-numeric-data".into();
        store
            .register_result_seed(&old, &tied.key, &tied)
            .await
            .unwrap();
        seal_seed_inventory(&store, &old).await;
        let (prefix, _) = seed_headers(&store, &revision, "prefix", 1).await;
        store
            .cancel_run(prefix.run(), "cancel-prefix")
            .await
            .unwrap();
        let closed = store
            .recover_closed_attempt(prefix.run(), "recover-prefix")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture supplies the actual cancelled incomplete prefix;
        // its registered header must not acquire original scientific permission.
        let terminal = unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "seal-prefix",
                    TerminalClass::Cancelled,
                    &[],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        assert!(terminal.terminal());
        assert!(terminal.seed_manifest.is_none());
        let candidates = store
            .result_seed_candidates("layout", "preparation", "backend", None)
            .await
            .unwrap();
        assert_eq!(candidates, vec![tied, seeds[0].clone()]);
        assert!(
            store
                .result_seed_candidates("layout", "preparation", "backend", Some(prefix.attempt()))
                .await
                .unwrap()
                .is_empty()
        );
        remove(&store, &database).await;
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
            .append_result_batch(
                &fence,
                "batch-0",
                "values",
                0,
                ResultPayload::new([1, 2].to_vec()).unwrap(),
                2,
            )
            .await
            .unwrap();
        assert_eq!(
            batch,
            store
                .append_result_batch(
                    &fence,
                    "batch-0",
                    "values",
                    0,
                    ResultPayload::new([1, 2].to_vec()).unwrap(),
                    2
                )
                .await
                .unwrap()
        );
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "batch-0",
                    "values",
                    0,
                    ResultPayload::new([1, 3].to_vec()).unwrap(),
                    2
                )
                .await
                .is_err()
        );
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "changed-batch",
                    "values",
                    0,
                    ResultPayload::new([1, 3].to_vec()).unwrap(),
                    2
                )
                .await
                .is_err()
        );
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "gap",
                    "values",
                    2,
                    ResultPayload::new([4].to_vec()).unwrap(),
                    1
                )
                .await
                .is_err()
        );
        assert!(store.read_results("run", fence.attempt()).await.is_err());
        let close = store.close_result_ingestion(&fence, "close").await.unwrap();
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "late",
                    "values",
                    1,
                    ResultPayload::new([5].to_vec()).unwrap(),
                    1
                )
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
                .seal_attempt(
                    &manifest,
                    "seal",
                    TerminalClass::Partial,
                    &[8],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        assert_eq!(
            terminal,
            // SAFETY: this is an exact replay of the fixture-owned partial observations and completion.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        "seal",
                        TerminalClass::Partial,
                        &[8],
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .unwrap()
        );
        assert!(
            // SAFETY: the fixture owns these synthetic observations; changing its admitted class must be refused.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        "seal",
                        TerminalClass::Succeeded,
                        &[8],
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .is_err()
        );
        assert!(
            // SAFETY: the fixture owns these synthetic observations; changing its admitted completion must be refused.
            unsafe {
                store
                    .seal_attempt(
                        &manifest,
                        "seal",
                        TerminalClass::Partial,
                        &[9],
                        SeedEligibility::Unavailable,
                    )
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
        let history = store.read_results("run", fence.attempt()).await.unwrap();
        assert_eq!(
            history
                .attempt()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("partial")
        );
        let fresh = store
            .claim_run("run", "new-claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        assert!(fresh.generation() > fence.generation());
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "stale",
                    "values",
                    1,
                    ResultPayload::new([5].to_vec()).unwrap(),
                    1
                )
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
        assert!(store.read_results("run", fence.attempt()).await.is_ok());
        drop(history);
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
                        ResultPayload::new(original.to_vec()).unwrap(),
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
                            SeedEligibility::Unavailable,
                        )
                        .await
                }
                .unwrap();
                assert_eq!(
                    terminal.outcome.as_ref().map(|value| value.as_str()),
                    Some("succeeded")
                );
                let read = store.read_results(&key, fence.attempt()).await.unwrap();
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
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .zip([0.0, 25.0, 60.0, 100.0])
                {
                    let actual = f64::from_le_bytes(*encoded);
                    assert!((actual - 273.15 - celsius).abs() < 1e-12);
                }
                assert!(
                    store
                        .append_result_batch(
                            &fence,
                            &format!("concurrent-late-{index}"),
                            "temperature_kelvin",
                            1,
                            ResultPayload::new([0].to_vec()).unwrap(),
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
                store.append_result_batch(
                    &fence,
                    &operation,
                    "values",
                    0,
                    ResultPayload::new(payload.to_vec()).unwrap(),
                    1
                ),
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
                    .append_result_batch(
                        &fence,
                        &format!("late-{index}"),
                        "values",
                        1,
                        ResultPayload::new([index].to_vec()).unwrap(),
                        1
                    )
                    .await
                    .is_err()
            );
        }
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_lookup_queue_consumes_original_clock() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "clock-queue").await;
        let mut pending = tokio::task::JoinSet::new();
        let mut contexts = Vec::new();
        for _ in 0..32 {
            let client = std::sync::Arc::clone(&store.db);
            let context = crate::transport::client::RequestContext::new(
                std::time::Instant::now() + Duration::from_secs(5),
            );
            contexts.push(context.clone());
            pending.spawn(async move {
                bounded_query(
                    client
                        .query("SLEEP 1s; RETURN true;")
                        .request_context(context),
                )
                .await
            });
        }
        tokio::time::timeout(Duration::from_secs(2), async {
            while !contexts
                .iter()
                .all(crate::transport::client::RequestContext::was_dispatched)
            {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        let offered = tokio::time::Instant::now() + REQUEST_TIMEOUT;
        let mut observed = None;
        let mut result = store
            .protected_execution_query("clock-queue", "execution-clock-control", || {
                observed = Some(original_deadline(REQUEST_TIMEOUT));
                Ok(store.db.query("RETURN true;"))
            })
            .await
            .unwrap();
        assert_eq!(result.take::<Value>(0).unwrap(), Value::Bool(true));
        // The blocked lookup consumes around a second, while this 50 ms
        // allowance covers only local scheduling before the first clock mint.
        assert!(observed.unwrap() <= offered + Duration::from_millis(50));
        while let Some(result) = pending.join_next().await {
            result.unwrap().unwrap();
        }
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_renewal_cannot_commit_after_original_lease_expires() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "late-renewal").await;
        let fence = store
            .claim_run(
                "late-renewal",
                "late-renewal-claim",
                "worker",
                Duration::from_secs(1),
            )
            .await
            .unwrap();
        let original = store
            .canonical_attempt(&fence.attempt)
            .await
            .unwrap()
            .unwrap();
        // Fault-inject a delay into this isolated database's installed production
        // renewal function after UPDATE and before its final lease admission.
        let start = wire::SCHEMA
            .find("DEFINE FUNCTION fn::pse_execution_v1::renew(")
            .unwrap();
        let length = wire::SCHEMA[start..].find("\n};").unwrap() + 3;
        let function = &wire::SCHEMA[start..start + length];
        let delayed = function.replacen("DEFINE FUNCTION ", "DEFINE FUNCTION OVERWRITE ", 1)
            .replacen("    fn::pse_execution_v1::lease($rpc_expiry, $original_expiry);", "    SLEEP 1500ms;\n    fn::pse_execution_v1::lease($rpc_expiry, $original_expiry);", 1);
        assert!(delayed.contains("SLEEP 1500ms;"));
        bounded_query(store.db.query(delayed)).await.unwrap();
        let error = store
            .renew_attempt(&fence, Duration::from_secs(60))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            CanonicalError::Refused {
                code: pse_schema::catalog::canonical_refusals::RefusalCode::ExecutionAttemptLeaseExpiredBeforeCommit,
                class: pse_schema::catalog::canonical_refusals::RefusalClass::Fence,
                effect: crate::canonical::FailureEffect::StatementRefused,
                ..
            }
        ), "{error}");
        let saved = store
            .canonical_attempt(&fence.attempt)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.expires_at, original.expires_at);
        assert!(!saved.terminal());
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_expiry_cancellation_and_truthful_recovery() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "expires").await;
        let fence = store
            .claim_run("expires", "expiry-claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        store
            .append_result_batch(
                &fence,
                "first",
                "diagnostics",
                0,
                ResultPayload::new([7].to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        // Establish the acknowledged prefix before starting the short expiry
        // phase. Initial claim/append scheduling is not the expiry behavior under
        // test; actual renewal preserves this exact attempt and generation.
        let expiring = store
            .renew_attempt(&fence, Duration::from_millis(250))
            .await
            .unwrap();
        assert_eq!(expiring.run, fence.run());
        assert_eq!(expiring.key, fence.attempt());
        assert_eq!(expiring.generation, fence.generation);
        tokio::time::sleep(Duration::from_millis(300)).await;
        let mut clock = bounded_query(store.db.query("RETURN time::micros();"))
            .await
            .unwrap();
        let now = canonical_codec::decode_int(clock.take::<Value>(0).unwrap()).unwrap();
        assert!(
            expiring.expires_at <= now,
            "the renewed server lease must expire"
        );
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "stale",
                    "diagnostics",
                    1,
                    ResultPayload::new([8].to_vec()).unwrap(),
                    1
                )
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
                    .seal_attempt(
                        &manifest,
                        "false-success",
                        TerminalClass::Succeeded,
                        &[],
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .is_err()
        );
        // SAFETY: the fixture owns the recovered diagnostic observation and admits its actual failed outcome.
        let failed = unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "failed",
                    TerminalClass::Failed,
                    &[7],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        assert_eq!(
            failed.outcome.as_ref().map(|value| value.as_str()),
            Some("failed")
        );
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
            .append_result_batch(
                &fence,
                "cancel-first",
                "values",
                0,
                ResultPayload::new([1].to_vec()).unwrap(),
                1,
            )
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
                    .seal_attempt(
                        &stale_manifest,
                        "stale-seal",
                        TerminalClass::Succeeded,
                        &[],
                        SeedEligibility::Unavailable,
                    )
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
                    .seal_attempt(
                        &manifest,
                        "ignored-cancel",
                        TerminalClass::Failed,
                        &[],
                        SeedEligibility::Unavailable,
                    )
                    .await
            }
            .is_err()
        );
        // SAFETY: the fixture owns the reconciled synthetic observation and admits its actual cancellation.
        let terminal = unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "cancel-seal",
                    TerminalClass::Cancelled,
                    &[1],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        assert_eq!(
            terminal.outcome.as_ref().map(|value| value.as_str()),
            Some("cancelled")
        );
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
    #[ignore = "explicit owned server restart needs its external lifecycle rendezvous"]
    async fn retained_canonical_store_recovers_a_new_generation_after_external_server_restart() {
        let directory = std::path::PathBuf::from(
            std::env::var_os("PSE_CANONICAL_RESTART_CONTROL")
                .expect("explicit owned restart control directory required"),
        );
        for name in [
            "ready.json",
            "stopped.json",
            "down-observed.json",
            "restarted.json",
            "complete.json",
        ] {
            assert!(
                !directory.join(name).exists(),
                "restart rendezvous must be fresh: {name}"
            );
        }
        let publish = |name: &str, value: serde_json::Value| {
            let temporary = directory.join(format!("{name}.tmp"));
            std::fs::write(&temporary, serde_json::to_vec(&value).unwrap()).unwrap();
            std::fs::rename(temporary, directory.join(name)).unwrap();
        };
        async fn wait(directory: &Path, name: &str) {
            let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
            while !directory.join(name).exists() {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "external lifecycle did not publish {name}"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
        let (store, database, revision) = fixture().await;
        let request = RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: "retained-restart-run".into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let original = store.begin_run(&request).await.unwrap();
        let fence = store
            .claim_run(
                &original.key,
                "retained-restart-claim",
                "worker",
                Duration::from_secs(600),
            )
            .await
            .unwrap();
        let acknowledged = store.canonical_run(&original.key).await.unwrap().unwrap();
        let issuing_store = store.issuing_store().unwrap().to_owned();
        publish(
            "ready.json",
            serde_json::json!({"database":database,"state":store.deployment_state(),"sequence":original.sequence,"attempt":fence.attempt(),"generation":fence.generation(),"issuing_store":issuing_store}),
        );
        wait(&directory, "stopped.json").await;
        // Force this retained physical generation to observe the real closed
        // server. The failed read is not an application replay authorization.
        let lost = crate::canonical::operation_scope(
            std::time::Instant::now() + Duration::from_secs(2),
            store.canonical_run(&original.key),
        )
        .await;
        let error = lost.unwrap_err();
        assert!(
            error.is_infrastructure(),
            "closed server must not fabricate a domain refusal: {error}"
        );
        publish(
            "down-observed.json",
            serde_json::json!({"database":database,"error":error.to_string()}),
        );
        wait(&directory, "restarted.json").await;
        // The same handle/session owner must restore ACK-only setup for a new
        // undispatched operation. Do not reconnect or reopen in this control.
        assert_eq!(store.begin_run(&request).await.unwrap(), acknowledged);
        assert_eq!(store.issuing_store().unwrap(), issuing_store);
        let resumed = store
            .claim_run(
                &original.key,
                "retained-restart-claim",
                "worker",
                Duration::from_secs(600),
            )
            .await
            .unwrap();
        assert_eq!(resumed.attempt(), fence.attempt());
        assert_eq!(resumed.generation(), fence.generation());
        store
            .renew_attempt(&fence, Duration::from_secs(600))
            .await
            .unwrap();
        store
            .append_result_batch(
                &fence,
                "retained-restart-batch",
                "values",
                0,
                ResultPayload::new(vec![7]).unwrap(),
                1,
            )
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&fence, "retained-restart-close")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        assert_eq!(decode_result_descriptors(manifest.row()).unwrap().len(), 1);
        let fresh = store
            .begin_run(&RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "retained-restart-next".into(),
                ..request
            })
            .await
            .unwrap();
        assert_eq!(
            fresh.sequence,
            original.sequence + 1,
            "ACK restoration must not replay a publication or consume another sequence"
        );
        assert_eq!(
            store
                .execution_run_page(&revision.problem, None, 64)
                .await
                .unwrap()
                .len(),
            2
        );
        remove(&store, &database).await;
        publish(
            "complete.json",
            serde_json::json!({"database":database,"same_store":true,"same_attempt":true,"next_sequence":fresh.sequence}),
        );
    }

    #[tokio::test]
    async fn canonical_physical_phase_assertions_backstop_direct_mutations() {
        let (store, database, revision) = fixture().await;
        run(&store, &revision, "phase-backstop").await;
        let fence = store
            .claim_run(
                "phase-backstop",
                "phase-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let original = store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        run(&store, &revision, "phase-legal-sibling").await;
        let sibling = store
            .claim_run(
                "phase-legal-sibling",
                "phase-sibling-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let sibling_closed = store
            .close_result_ingestion(&sibling, "phase-sibling-close")
            .await
            .unwrap();
        store
            .reconcile_closed_attempt(&sibling_closed)
            .await
            .unwrap();
        let sibling_before = store
            .canonical_attempt(sibling.attempt())
            .await
            .unwrap()
            .unwrap();
        let multirow=bounded_query(store.db.query("UPDATE canonical_attempts SET phase='terminal',outcome='failed',completion=$completion WHERE key IN [$attempt,$sibling];").bind(("attempt",fence.attempt().to_owned())).bind(("sibling",sibling.attempt().to_owned())).bind(("completion",Bytes::from(vec![1_u8])))).await.unwrap_err();
        assert!(matches!(multirow, CanonicalError::Driver(_)));
        assert_eq!(
            store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap(),
            original
        );
        assert_eq!(
            store
                .canonical_attempt(sibling.attempt())
                .await
                .unwrap()
                .unwrap(),
            sibling_before,
            "one invalid row must roll back the otherwise legal sibling mutation"
        );
        for (index, statement) in [
            "CREATE ONLY type::record('canonical_attempts',$key) CONTENT $row;",
            "UPSERT ONLY type::record('canonical_attempts',$key) CONTENT $row;",
            "INSERT INTO canonical_attempts $row;",
        ]
        .into_iter()
        .enumerate()
        {
            let key = format!("illegal-created-phase-{index}");
            let mut row = wire::encode_canonical_attempts(&original).unwrap();
            row.insert(
                "id",
                canonical_codec::record_link("canonical_attempts", &key),
            );
            row.insert("key", Value::String(key.clone()));
            row.insert("phase", Value::String("invented".into()));
            let error = bounded_query(
                store
                    .db
                    .query(statement)
                    .bind(("key", key.clone()))
                    .bind(("row", row)),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(error, CanonicalError::Driver(_)),
                "schema failure must stay internal: {error}"
            );
            assert!(store.canonical_attempt(&key).await.unwrap().is_none());
        }
        let mut duplicate = wire::encode_canonical_attempts(&original).unwrap();
        duplicate.insert(
            "id",
            canonical_codec::record_link("canonical_attempts", fence.attempt()),
        );
        let error = bounded_query(
            store
                .db
                .query(
                    "INSERT INTO canonical_attempts $row ON DUPLICATE KEY UPDATE phase='invented';",
                )
                .bind(("row", duplicate)),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, CanonicalError::Driver(_)));
        for (sql, transactional) in [
            (
                "UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='terminal',outcome='succeeded';",
                false,
            ),
            (
                "UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='terminal',outcome='failed',completion=NONE;",
                false,
            ),
            (
                "UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='closed_awaiting_manifest';",
                false,
            ),
            (
                "UPDATE ONLY type::record('canonical_attempts',$attempt) SET outcome='invented';",
                false,
            ),
            (
                "UPDATE ONLY type::record('canonical_attempts',$attempt) SET key='different';",
                false,
            ),
            (
                "UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='ingesting',completion=$completion;",
                false,
            ),
            (
                "CREATE canonical_guards:illegal_phase_rollback SET key='illegal_phase_rollback',generation=0dec; UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='invented';",
                true,
            ),
        ] {
            let query = if transactional {
                store.db.script(
                    GuardedScript::new(Envelope::DeadlineOnly)
                        .returning::<Value>(sql, "NONE")
                        .unwrap(),
                )
            } else {
                store.db.query(sql)
            };
            let error = bounded_query(
                query
                    .bind(("attempt", fence.attempt().to_owned()))
                    .bind(("completion", Bytes::from(vec![1_u8]))),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(error, CanonicalError::Driver(_)),
                "schema failures must not become domain refusals: {error}"
            );
            assert!(
                store
                    .canonical_attempt(fence.attempt())
                    .await
                    .unwrap()
                    .unwrap()
                    .ingestion_open()
            );
        }
        let mut rolled_back = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY canonical_guards:illegal_phase_rollback;"),
        )
        .await
        .unwrap();
        assert!(rolled_back.take::<Option<Object>>(0).unwrap().is_none());
        let closed = store
            .close_result_ingestion(&fence, "phase-close")
            .await
            .unwrap();
        let row = store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.phase,
            pse_model::generated::enums::CanonicalAttemptPhase::ClosedAwaitingManifest
        );
        assert!(row.closed_manifest.is_none());
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        let row = store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.phase,
            pse_model::generated::enums::CanonicalAttemptPhase::ClosedWithManifest
        );
        // SAFETY: this isolated fixture owns its empty synthetic result and exact completion.
        let terminal = unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "phase-seal",
                    TerminalClass::Succeeded,
                    b"original completion",
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        assert!(terminal.terminal());
        assert!(terminal.seed_manifest.is_none());
        let error=bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$attempt) SET seed_manifest=$attempt;").bind(("attempt",fence.attempt().to_owned()))).await.unwrap_err();
        assert!(
            matches!(error, CanonicalError::Driver(_)),
            "a terminal noneligible attempt cannot promote itself to a seed"
        );
        let error = bounded_query(
            store
                .db
                .query(
                    "UPDATE ONLY type::record('canonical_attempts',$attempt) SET completion=NONE;",
                )
                .bind(("attempt", fence.attempt().to_owned())),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(error, CanonicalError::Driver(_)),
            "payload removal requires the explicit physical compaction transition"
        );
        bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='terminal_compacted',completion=NONE;").bind(("attempt",fence.attempt().to_owned()))).await.unwrap();
        let compacted = store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            compacted.phase,
            pse_model::generated::enums::CanonicalAttemptPhase::TerminalCompacted
        );
        assert!(compacted.terminal() && compacted.closed() && !compacted.ingestion_open());
        assert_eq!(compacted.outcome, Some(TerminalClass::Succeeded));
        assert!(compacted.completion.is_none());
        for sql in [
            "UPDATE ONLY type::record('canonical_attempts',$attempt) SET completion=$completion;",
            "UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='terminal';",
        ] {
            let error = bounded_query(
                store
                    .db
                    .query(sql)
                    .bind(("attempt", fence.attempt().to_owned()))
                    .bind(("completion", Bytes::from(vec![9_u8]))),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(error, CanonicalError::Driver(_)),
                "compaction cannot be reversed or resurrect its payload: {error}"
            );
        }
        let metadata = store
            .canonical_attempt_page(fence.run(), None, fence.generation())
            .await
            .unwrap();
        assert_eq!(metadata.len(), 1);
        assert!(metadata[0].terminal);
        run(&store, &revision, "phase-seed-eligible").await;
        let eligible = store
            .claim_run(
                "phase-seed-eligible",
                "phase-seed-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let eligible_closed = store
            .close_result_ingestion(&eligible, "phase-seed-close")
            .await
            .unwrap();
        let eligible_manifest = store
            .reconcile_closed_attempt(&eligible_closed)
            .await
            .unwrap();
        // SAFETY: this owner declares the entire empty synthetic inventory eligible.
        let eligible_terminal = unsafe {
            store
                .seal_attempt(
                    &eligible_manifest,
                    "phase-seed-seal",
                    TerminalClass::Succeeded,
                    b"eligible completion",
                    SeedEligibility::CompleteOriginalInventory,
                )
                .await
        }
        .unwrap();
        assert_eq!(
            eligible_terminal.seed_manifest.as_deref(),
            Some(eligible.attempt())
        );
        for sql in [
            "UPDATE ONLY type::record('canonical_attempts',$attempt) SET seed_manifest='different';",
            "UPDATE ONLY type::record('canonical_attempts',$attempt) SET seed_manifest=NONE;",
        ] {
            let error = bounded_query(
                store
                    .db
                    .query(sql)
                    .bind(("attempt", eligible.attempt().to_owned())),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(error, CanonicalError::Driver(_)),
                "a sealed seed pointer cannot change or disappear"
            );
            assert_eq!(
                store
                    .canonical_attempt(eligible.attempt())
                    .await
                    .unwrap()
                    .unwrap(),
                eligible_terminal
            );
        }
        let multirow=bounded_query(store.db.query("UPDATE canonical_attempts SET expires_at+=1,seed_manifest=IF key=$eligible { NONE } ELSE { seed_manifest } WHERE key IN [$eligible,$other];")
            .bind(("eligible",eligible.attempt().to_owned())).bind(("other",fence.attempt().to_owned()))).await.unwrap_err();
        assert!(matches!(multirow, CanonicalError::Driver(_)));
        assert_eq!(
            store
                .canonical_attempt(eligible.attempt())
                .await
                .unwrap()
                .unwrap(),
            eligible_terminal
        );
        assert_eq!(
            store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap(),
            compacted,
            "the legal sibling mutation must roll back when the synchronous seed backstop throws"
        );
        bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='terminal_compacted',completion=NONE;").bind(("attempt",eligible.attempt().to_owned()))).await.unwrap();
        let eligible_compacted = store
            .canonical_attempt(eligible.attempt())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            eligible_compacted.seed_manifest, eligible_terminal.seed_manifest,
            "legal payload compaction retains the immutable seed pointer"
        );
        // The phase exception belongs to an actual declared admission run and
        // occurrence. Producer/key spelling is not an admission capability.
        let source_request = |key: &str| RunRequest {
            key: key.into(),
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let study = "phase-expired-admission-study";
        let occurrence = pse_model::study::OccurrenceKey(0);
        let point = crate::canonical_studies::NewOccurrence {
            projection: None,
            policy: pse_model::study::PointPolicy {
                key: occurrence,
                dependencies: vec![],
                start: pse_model::study::StartPolicy::Fresh,
                seed_need: pse_model::study::SeedNeed::Pending,
                attempt_limit: 1,
            },
            descriptor: vec![42],
            run: source_request("phase-expired-point-run"),
        };
        store
            .create_study(
                study,
                &source_request("phase-expired-parent-run"),
                b"phase assertion fixture",
                &[point],
                &|| false,
            )
            .await
            .unwrap();
        let point_key = crate::canonical_studies::point_key(study, occurrence);
        let scope = store.study_scope(&point_key).await.unwrap();
        let admission_run = scope.point().admission_run.as_deref().unwrap();
        assert!(
            matches!(
                store
                    .claim_run(
                        admission_run,
                        "phase-generic-cannot-admit",
                        "phase-admission-worker",
                        Duration::from_secs(60)
                    )
                    .await,
                Err(CanonicalError::Refused {
                    code: crate::canonical::RefusalCode::ExecutionRunNotClaimable,
                    ..
                })
            ),
            "an ordinary execution call cannot mint an input-admission fence"
        );
        for (selected_point, expected_code) in [
            (
                "phase-unrelated-point",
                crate::canonical::RefusalCode::ScientificAdmissionCapabilityBelongsElsewhere,
            ),
            (
                point_key.as_str(),
                crate::canonical::RefusalCode::StudyDecisionOmittedOrAddedAPremise,
            ),
        ] {
            let mut forged_owner = Object::new();
            forged_owner.insert("study", study);
            forged_owner.insert(
                "generation",
                canonical_codec::encode_uint(scope.generation()).unwrap(),
            );
            forged_owner.insert("point", selected_point.to_owned());
            forged_owner.insert("expected", Value::Array(Vec::<Value>::new().into()));
            let forged=bounded_query(store.db.query("RETURN fn::pse_execution_v1::claim($pse_rpc_expires_at,$run,'phase-forged-owner',$request,'phase-forged-attempt','worker',60000000,$interpretation,NONE,$owner);")
                .bind(("run",admission_run.to_owned())).bind(("request",Bytes::from(vec![9_u8])))
                .bind(("interpretation",wire::INTERPRETATION)).bind(("owner",forged_owner))).await;
            assert!(
                matches!(forged,Err(CanonicalError::Refused { code, .. }) if code==expected_code),
                "forged ownership cannot substitute another occurrence or omit its actual guarded premises"
            );
        }
        let admission = store
            .claim_study_admission(&scope, "phase-admission-worker", Duration::from_secs(60))
            .await
            .unwrap()
            .unwrap();
        let admission_attempt = store
            .canonical_attempt(admission.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(
                store
                    .claim_run(
                        admission.run(),
                        &admission_attempt.claim_operation,
                        "phase-admission-worker",
                        Duration::from_secs(60)
                    )
                    .await,
                Err(CanonicalError::OperationReused)
            ),
            "ordinary request settlement cannot recover the admission owner's tagged request"
        );
        assert_eq!(
            store
                .canonical_attempt(admission.attempt())
                .await
                .unwrap()
                .unwrap(),
            admission_attempt,
            "refused ordinary and forged-owner claims leave the admitted fence unchanged"
        );
        let owner = store.canonical_run(admission.run()).await.unwrap().unwrap();
        assert_eq!(
            owner.purpose,
            pse_model::generated::enums::CanonicalRunPurpose::StudyAdmission
        );
        assert_eq!(owner.admission_study.as_deref(), Some(study));
        assert_eq!(owner.admission_point.as_deref(), Some(point_key.as_str()));
        assert_eq!(
            scope.point().admission_run.as_deref(),
            Some(admission.run())
        );
        let admission_key = admission.attempt();
        bounded_query(
            store
                .db
                .query("UPDATE ONLY type::record('canonical_attempts',$key) SET expires_at=0;")
                .bind(("key", admission_key)),
        )
        .await
        .unwrap();
        bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$key) SET phase='terminal',outcome='failed';").bind(("key",admission_key))).await.unwrap();
        let error=bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$key) SET completion=$completion;").bind(("key",admission_key)).bind(("completion",Bytes::from(vec![9_u8])))).await.unwrap_err();
        assert!(
            matches!(error, CanonicalError::Driver(_)),
            "expired source admission has no completion payload to resurrect"
        );
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn canonical_execution_named_absence_guard_and_ordered_decimal_sequence() {
        let (store, database, revision) = fixture().await;
        let request = RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
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
            .append_result_batch(
                &fence,
                "missing-first",
                "values",
                0,
                ResultPayload::new([1].to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        let missing = store
            .append_result_batch(
                &fence,
                "missing-second",
                "values",
                1,
                ResultPayload::new([2].to_vec()).unwrap(),
                1,
            )
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
                .bind(("key", missing.key.clone())),
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
                .terminal()
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
                unsafe {
                    store.seal_attempt(
                        &manifest,
                        &seal,
                        TerminalClass::Succeeded,
                        &[],
                        SeedEligibility::Unavailable,
                    )
                },
                store.cancel_run(&key, &cancel)
            );
            assert_ne!(sealed.is_ok(), cancelled.is_ok());
            let actual = store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap();
            if sealed.is_ok() {
                assert_eq!(
                    actual.outcome.as_ref().map(|value| value.as_str()),
                    Some("succeeded")
                );
            } else {
                assert!(!actual.terminal());
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
                            SeedEligibility::Unavailable,
                        )
                        .await
                }
                .unwrap();
                assert_eq!(
                    final_outcome.outcome.as_ref().map(|value| value.as_str()),
                    Some("cancelled")
                );
            }
        }
        remove(&store, &database).await;
    }
}

/// Worker admission capability; fields cannot be fabricated by an external caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptFence {
    run: pse_model::generated::canonical::RunKey,
    attempt: pse_model::generated::canonical::AttemptKey,
    generation: u64,
}
impl AttemptFence {
    pub(crate) fn from_row(row: CanonicalAttempt) -> Result<Self, CanonicalError> {
        if row.generation == 0 || row.terminal() || row.closed() {
            return Err(CanonicalError::Configuration(
                "native claim returned no live fence".into(),
            ));
        }
        Ok(Self {
            run: row.run.into(),
            attempt: row.key.into(),
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

/// Scientific authority for discovering seeds from this exact frozen inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedEligibility {
    /// Every registered seed belongs to the fully exported original completion.
    CompleteOriginalInventory,
    /// Recovery, summary or an incomplete export supplies no seed permission.
    Unavailable,
}

/// Terminal class supplied only after the scientific owner determines quality.
pub use pse_model::generated::enums::CanonicalTerminalOutcome as TerminalClass;

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

/// Conservative resident allowance for one bounded acknowledgment accumulator.
/// Callers reserve this before constructing the accumulator or retaining receipts.
pub const RESULT_ACKNOWLEDGMENT_BYTES: usize =
    RESULT_SETS * (64 * 1024) + 4 * EXECUTION_METADATA_BYTES;

/// Compact immutable metadata returned by an authenticated append.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppendReceipt {
    /// Exact immutable batch coordinate.
    pub key: String,
    /// Owning attempt.
    pub attempt: String,
    /// Owning result set.
    pub result_set: String,
    /// Actual admitted ordinal.
    pub ordinal: u64,
    /// Digest of the original admitted bytes.
    pub digest: String,
    /// Actual admitted row coverage.
    pub row_count: u64,
    /// Exact framed physical instance, native index, codec and schema binding.
    pub payload_binding: String,
}
/// Authenticated successful append or exact operation settlement. Not constructible
/// from a descriptor, serialized input or a copied public receipt.
#[derive(Debug, PartialEq, Eq)]
pub struct AppendAck {
    receipt: AppendReceipt,
    fence: AttemptFence,
    name: String,
    operation: String,
    request: Vec<u8>,
}
impl std::ops::Deref for AppendAck {
    type Target = AppendReceipt;
    fn deref(&self) -> &AppendReceipt {
        &self.receipt
    }
}
/// One already encoded immutable block, with the caller's prepaid staging owner.
/// Request framing and validation are shared with individual append operations.
/// This owns no successful acknowledgment or writer permission.
#[derive(Debug)]
pub struct ResultBlockAppend {
    acknowledgment: AppendAck,
    item: Object,
    extent: usize,
    _owner: Option<std::sync::Arc<pse_columnar::AllocationLease>>,
    payload: ResultPayload,
}
impl ResultBlockAppend {
    /// Prepare one block only after admitting the copied metadata/encoder extent.
    /// The payload keeps its existing immutable bytes and per-block request identity.
    #[allow(
        clippy::too_many_arguments,
        reason = "the original immutable block, its typed indexes and prepaid copy owner are separate append premises"
    )]
    pub fn new(
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: ResultPayload,
        row_count: u64,
        block: &ResultBlock,
        outputs: &[ResultOutput],
        owner: std::sync::Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, CanonicalError> {
        Self::prepare(
            fence,
            operation,
            name,
            ordinal,
            payload,
            row_count,
            Some(block),
            outputs,
            Some(owner),
        )
    }
    /// Retain both actual metadata/copy and encoded-payload reservations in the driver.
    /// They may be separate original allocations; selecting one cannot release the other.
    fn scientific_owners(&self) -> Vec<std::sync::Arc<pse_columnar::AllocationLease>> {
        let mut owners = self._owner.iter().cloned().collect::<Vec<_>>();
        if let Some(owner) = &self.payload.owner
            && !owners
                .iter()
                .any(|retained| std::sync::Arc::ptr_eq(retained, owner))
        {
            owners.push(owner.clone());
        }
        owners
    }
    /// Conservative encoded metadata extent, including file binding and request.
    pub fn encoded_extent(&self) -> usize {
        self.extent
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "one immutable append constructor preserves the original individual request framing"
    )]
    fn prepare(
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: ResultPayload,
        row_count: u64,
        block: Option<&ResultBlock>,
        outputs: &[ResultOutput],
        owner: Option<std::sync::Arc<pse_columnar::AllocationLease>>,
    ) -> Result<Self, CanonicalError> {
        identity(operation)?;
        if outputs.windows(2).any(|pair| pair[0].key >= pair[1].key) {
            return Err(CanonicalError::Configuration(
                "result index metadata must have strictly ordered identities".into(),
            ));
        }
        identity(name)?;
        if name.len() > 128 {
            return Err(CanonicalError::PayloadLimit);
        }
        let payload_bytes = payload.byte_len();
        let set_key = result_set_key(&fence.attempt, name);
        let key = result_batch_key(&fence.attempt, &set_key, ordinal);
        let digest = payload.digest.clone();
        let owner = owner.or(payload.owner.clone());
        let file = payload.file.as_ref().map(|file| file.descriptor().clone());
        let payload_index = payload.payload_index;
        let set = ResultSet {
            key: set_key.clone(),
            attempt: fence.attempt.as_str().to_owned(),
            name: name.into(),
            interpretation: wire::INTERPRETATION.into(),
            next_ordinal: 0,
            row_count: 0,
        };
        let batch = ResultBatch {
            key: key.clone(),
            attempt: fence.attempt.as_str().to_owned(),
            result_set: set_key.clone(),
            ordinal,
            digest: digest.clone(),
            row_count,
            payload_binding: String::new(),
            placement: if payload.inline() {
                pse_model::generated::enums::CanonicalPayloadPlacement::Inline
            } else {
                pse_model::generated::enums::CanonicalPayloadPlacement::Sealed
            },
            inline_key: payload
                .inline()
                .then(|| payload.inline_key(fence, operation)),
            file_instance: file.as_ref().map(|file| file.instance.clone()),
            payload_index,
            payload_bytes,
            schema: payload.schema().to_owned(),
            encoding: payload.encoding().to_owned(),
        };
        let mut admitted_block = block.cloned();
        if let Some(facts) = &payload.facts
            && (facts.rows() != row_count
                || facts.relation().to_string() != name
                || admitted_block.is_none())
        {
            return Err(CanonicalError::Configuration(
                "admitted result facts differ from append coordinates".into(),
            ));
        }
        let statistics = payload
            .facts
            .as_ref()
            .map(|facts| {
                facts
                    .encode_bound(&pse_relations::result_batch::StatisticsBinding {
                        payload: result_payload_locator(&batch)?
                            .payload_identity()
                            .to_owned(),
                        digest: batch.digest.clone(),
                        index: batch.payload_index,
                        interpretation: wire::INTERPRETATION.into(),
                    })
                    .map(pse_model::Bytes::from)
                    .map_err(|error| CanonicalError::Configuration(error.to_string()))
            })
            .transpose()?;
        if let Some(block) = &mut admitted_block {
            if block.statistics.is_some() && block.statistics != statistics {
                return Err(CanonicalError::Configuration(
                    "supplied statistics lack the actual admitted-file proof".into(),
                ));
            }
            block.statistics = statistics;
        }
        let block = admitted_block.as_ref();
        if let Some(block) = block
            && (block.key != key
                || block.batch != key
                || block.result_set != set_key
                || block.ordinal != ordinal
                || block.rows != row_count
                || block.payload_bytes != payload_bytes
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
            crate::canonical_results::validate_result_block_metadata(block, &batch)?;
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
            output_hash.finish_hash().to_hex(),
        ))?;
        let block = block
            .map(wire::encode_canonical_result_blocks)
            .transpose()?;
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

        let acknowledgment = AppendAck {
            receipt: AppendReceipt {
                key: batch.key,
                attempt: batch.attempt,
                result_set: batch.result_set,
                ordinal: batch.ordinal,
                digest: batch.digest,
                row_count: batch.row_count,
                payload_binding: batch.payload_binding,
            },
            fence: fence.clone(),
            name: name.into(),
            operation: operation.into(),
            request: request.clone(),
        };
        let mut item = Object::new();
        item.insert("operation", Value::String(operation.into()));
        item.insert("request", Value::Bytes(Bytes::from(request)));
        item.insert("set", Value::Object(encoded_set));
        item.insert("batch", Value::Object(encoded_batch));
        let inline = if payload.inline() {
            Some(wire::encode_canonical_inline_payloads(
                &pse_model::generated::runtime::canonical_inline_payloads::Row {
                    key: payload.inline_key(fence, operation),
                    attempt: fence.attempt.as_str().to_owned(),
                    digest: payload.digest.clone(),
                    encoding: payload.encoding().into(),
                    schema: payload.schema().into(),
                    interpretation: wire::INTERPRETATION.into(),
                    payload_bytes,
                    batch_count: payload.batch_count(),
                    body: payload.as_slice().to_vec().into(),
                },
            )?)
        } else {
            None
        };
        item.insert("inline", inline.map(Value::Object).unwrap_or(Value::None));
        item.insert("block", block.map(Value::Object).unwrap_or(Value::None));
        item.insert(
            "outputs",
            Value::Array(
                outputs
                    .into_iter()
                    .map(Value::Object)
                    .collect::<Vec<_>>()
                    .into(),
            ),
        );
        let extent = result_metadata_extent(&Value::Object(item.clone()))?;
        Ok(Self {
            acknowledgment,
            item,
            extent,
            _owner: owner,
            payload,
        })
    }
    fn expected_response(&self) -> Result<Object, CanonicalError> {
        let ack = &self.acknowledgment;
        let mut row = Object::new();
        row.insert("key", Value::String(ack.key.clone()));
        row.insert("attempt", Value::String(ack.attempt.clone()));
        row.insert("result_set", Value::String(ack.result_set.clone()));
        row.insert("ordinal", canonical_codec::encode_uint(ack.ordinal)?);
        row.insert("digest", Value::String(ack.digest.clone()));
        row.insert("row_count", canonical_codec::encode_uint(ack.row_count)?);
        row.insert(
            "payload_binding",
            Value::String(ack.payload_binding.clone()),
        );
        row.insert("request", Value::Bytes(Bytes::from(ack.request.clone())));
        Ok(row)
    }
}

fn validate_append_group(
    fence: &AttemptFence,
    items: &[ResultBlockAppend],
) -> Result<(), CanonicalError> {
    if items.is_empty() || items.len() > wire::RESULT_INDEX_RECORDS {
        return Err(CanonicalError::PayloadLimit);
    }
    let mut extent = 4096usize;
    let mut operations = std::collections::BTreeSet::new();
    let mut next = std::collections::BTreeMap::<&str, u64>::new();
    for item in items {
        if &item.acknowledgment.fence != fence
            || !operations.insert(item.acknowledgment.operation.as_str())
        {
            return Err(CanonicalError::OperationReused);
        }
        let ack = &item.acknowledgment;
        if next
            .get(ack.result_set.as_str())
            .is_some_and(|ordinal| *ordinal != ack.ordinal)
        {
            return Err(CanonicalError::OperationReused);
        }
        next.insert(
            &ack.result_set,
            ack.ordinal
                .checked_add(1)
                .ok_or(CanonicalError::PayloadLimit)?,
        );
        extent = extent
            .checked_add(item.extent)
            .ok_or(CanonicalError::PayloadLimit)?;
        if extent > pse_schema::catalog::substrate::RESULT_APPEND_TARGET_BYTES {
            return Err(CanonicalError::PayloadLimit);
        }
    }
    Ok(())
}

/// Authenticated declaration of a demanded relation with no scientific rows.
#[derive(Debug)]
pub struct EmptySetAck {
    fence: AttemptFence,
    name: String,
    key: String,
    operation: String,
    request: Vec<u8>,
}
/// Authenticated immutable seed registration; scientific eligibility is separate.
#[derive(Debug)]
pub struct SeedAck {
    fence: AttemptFence,
    key: String,
    operation: String,
    request: Vec<u8>,
    first: u64,
    count: u64,
    bytes: u64,
}
#[derive(Debug, PartialEq, Eq)]
struct ReceiptProof {
    operation: String,
    request: Vec<u8>,
    kind: &'static str,
    batch: Option<AppendReceipt>,
}
impl ReceiptProof {
    fn encode(&self) -> Result<Object, CanonicalError> {
        let mut row = Object::new();
        row.insert("operation", Value::String(self.operation.clone()));
        row.insert("request", Value::Bytes(Bytes::from(self.request.clone())));
        row.insert("kind", Value::String(self.kind.into()));
        if let Some(batch) = &self.batch {
            row.insert("key", Value::String(batch.key.clone()));
            row.insert("ordinal", canonical_codec::encode_uint(batch.ordinal)?);
            row.insert("digest", Value::String(batch.digest.clone()));
            row.insert("rows", canonical_codec::encode_uint(batch.row_count)?);
            row.insert(
                "payload_binding",
                Value::String(batch.payload_binding.clone()),
            );
        }
        Ok(row)
    }
}
#[derive(Debug)]
struct AcknowledgedSet {
    key: String,
    name: String,
    next: u64,
    rows: u64,
    digest: FramedHasher,
    last: Option<ReceiptProof>,
    complete: bool,
}
/// Attempt-local, bounded actual receipt accumulation. This contains no payload,
/// scientific permission, database connection or replay authority. It must live
/// under the caller's prepaid RESULT_ACKNOWLEDGMENT_BYTES allowance.
#[derive(Debug)]
pub struct ResultAcknowledgments {
    fence: AttemptFence,
    sets: std::collections::BTreeMap<String, AcknowledgedSet>,
    seed_count: u64,
    seed_batches: u64,
    seed_bytes: u64,
    last_seed: Option<(String, ReceiptProof)>,
}
impl ResultAcknowledgments {
    /// Bind this precharged owner to one actual live attempt.
    pub fn new(fence: &AttemptFence) -> Self {
        Self {
            fence: fence.clone(),
            sets: Default::default(),
            seed_count: 0,
            seed_batches: 0,
            seed_bytes: 0,
            last_seed: None,
        }
    }
    /// Original binding; this does not mint another writer capability.
    pub fn fence(&self) -> &AttemptFence {
        &self.fence
    }
    fn same_fence(&self, fence: &AttemptFence) -> Result<(), CanonicalError> {
        if self.fence.run != fence.run
            || self.fence.attempt != fence.attempt
            || self.fence.generation != fence.generation
        {
            return Err(CanonicalError::OperationReused);
        }
        Ok(())
    }
    /// Advertise real producer demand before reading or exporting its rows.
    pub fn expect(&mut self, name: &str) -> Result<(), CanonicalError> {
        identity(name)?;
        if name.len() > 128 {
            return Err(CanonicalError::PayloadLimit);
        }
        if self.sets.contains_key(name) {
            return Ok(());
        }
        if self.sets.len() == RESULT_SETS {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
        digest.str("pse.execution.batch-selection.v1");
        self.sets.insert(
            name.into(),
            AcknowledgedSet {
                key: result_set_key(&self.fence.attempt, name),
                name: name.into(),
                next: 0,
                rows: 0,
                digest,
                last: None,
                complete: false,
            },
        );
        Ok(())
    }
    /// Consume an actual ACK once. Only an exact immediate replay is idempotent;
    /// older ordinals, alternate operations and gaps never extend coverage.
    pub fn append(&mut self, ack: AppendAck) -> Result<(), CanonicalError> {
        self.same_fence(&ack.fence)?;
        self.expect(&ack.name)?;
        let set = self
            .sets
            .get_mut(&ack.name)
            .ok_or(CanonicalError::IncompleteResponse)?;
        let proof = ReceiptProof {
            operation: ack.operation,
            request: ack.request,
            kind: "append",
            batch: Some(ack.receipt),
        };
        let batch = proof
            .batch
            .as_ref()
            .ok_or(CanonicalError::IncompleteResponse)?;
        if set.last.as_ref() == Some(&proof) {
            return Ok(());
        }
        if set.complete
            || batch.result_set != set.key
            || batch.attempt != self.fence.attempt.as_str()
            || batch.key != result_batch_key(&self.fence.attempt, &set.key, batch.ordinal)
            || batch.ordinal != set.next
        {
            return Err(CanonicalError::OperationReused);
        }
        let next = set
            .next
            .checked_add(1)
            .ok_or(CanonicalError::PayloadLimit)?;
        let rows = set
            .rows
            .checked_add(batch.row_count)
            .ok_or(CanonicalError::PayloadLimit)?;
        set.digest
            .str(&batch.key)
            .str(&batch.digest)
            .u64(batch.row_count)
            .str(&batch.payload_binding);
        set.next = next;
        set.rows = rows;
        set.last = Some(proof);
        Ok(())
    }
    /// Consume the actual fenced declaration, preserving demanded empty membership.
    pub fn empty(&mut self, ack: EmptySetAck) -> Result<(), CanonicalError> {
        self.same_fence(&ack.fence)?;
        self.expect(&ack.name)?;
        let set = self
            .sets
            .get_mut(&ack.name)
            .ok_or(CanonicalError::IncompleteResponse)?;
        let proof = ReceiptProof {
            operation: ack.operation,
            request: ack.request,
            kind: "empty",
            batch: None,
        };
        if set.last.as_ref() == Some(&proof) {
            return Ok(());
        }
        if set.next != 0 || set.last.is_some() || set.key != ack.key || set.complete {
            return Err(CanonicalError::OperationReused);
        }
        set.last = Some(proof);
        Ok(())
    }
    /// Mark authentic producer exhaustion. A missing declaration or ACK refuses close.
    pub fn complete(&mut self, name: &str) -> Result<(), CanonicalError> {
        let set = self
            .sets
            .get_mut(name)
            .ok_or(CanonicalError::IncompleteResponse)?;
        if set.last.is_none() {
            return Err(CanonicalError::IncompleteResponse);
        }
        set.complete = true;
        Ok(())
    }
    /// Finish an optional producer which may have acknowledged no batches.
    pub fn complete_present(&mut self, name: &str) -> Result<(), CanonicalError> {
        if self.sets.contains_key(name) {
            self.complete(name)
        } else {
            Ok(())
        }
    }
    /// Actual acknowledged rows, useful for checking the joined progress writer.
    pub fn rows(&self, name: &str) -> u64 {
        self.sets.get(name).map_or(0, |set| set.rows)
    }
    /// Consume one actual seed registration after its contiguous payload ACKs.
    pub fn seed(&mut self, ack: SeedAck) -> Result<(), CanonicalError> {
        self.same_fence(&ack.fence)?;
        let proof = ReceiptProof {
            operation: ack.operation,
            request: ack.request,
            kind: "seed",
            batch: None,
        };
        if self
            .last_seed
            .as_ref()
            .is_some_and(|(key, last)| key == &ack.key && last == &proof)
        {
            return Ok(());
        }
        let end = ack
            .first
            .checked_add(ack.count)
            .ok_or(CanonicalError::PayloadLimit)?;
        if ack.first != self.seed_batches
            || ack.count == 0
            || self.sets.get("__seeds").is_none_or(|set| set.next != end)
        {
            return Err(CanonicalError::OperationReused);
        }
        let count = self
            .seed_count
            .checked_add(1)
            .ok_or(CanonicalError::PayloadLimit)?;
        let bytes = self
            .seed_bytes
            .checked_add(ack.bytes)
            .ok_or(CanonicalError::PayloadLimit)?;
        self.seed_batches = end;
        self.seed_count = count;
        self.seed_bytes = bytes;
        self.last_seed = Some((ack.key, proof));
        Ok(())
    }
    fn finish(self) -> Result<(ResultManifest, Vec<Object>, Object), CanonicalError> {
        let mut descriptors = Vec::with_capacity(self.sets.len());
        let mut proofs = Vec::with_capacity(self.sets.len());
        for (_, set) in self.sets {
            if !set.complete {
                return Err(CanonicalError::IncompleteResponse);
            }
            let mut proof = set
                .last
                .ok_or(CanonicalError::IncompleteResponse)?
                .encode()?;
            proof.insert("set", Value::String(set.key.clone()));
            proof.insert("name", Value::String(set.name.clone()));
            proof.insert("count", canonical_codec::encode_uint(set.next)?);
            proof.insert("row_count", canonical_codec::encode_uint(set.rows)?);
            proofs.push(proof);
            descriptors.push(ResultSetDescriptor {
                key: set.key,
                name: set.name,
                batch_count: set.next,
                row_count: set.rows,
                batches_digest: set.digest.finish_hash().to_hex(),
            });
        }
        descriptors.sort_by(|a, b| a.key.cmp(&b.key));
        let bytes = json(&descriptors)?;
        bounded(&bytes, EXECUTION_METADATA_BYTES)?;
        let row = ResultManifest {
            key: self.fence.attempt.as_str().to_owned(),
            attempt: self.fence.attempt.into_string(),
            generation: self.fence.generation,
            digest: metadata_digest(&bytes),
            descriptors: bytes.into(),
        };
        let mut seed = match self.last_seed {
            Some((key, proof)) => {
                let mut row = proof.encode()?;
                row.insert("key", Value::String(key));
                row
            }
            None => Object::new(),
        };
        seed.insert("count", canonical_codec::encode_uint(self.seed_count)?);
        seed.insert("batches", canonical_codec::encode_uint(self.seed_batches)?);
        seed.insert("bytes", canonical_codec::encode_uint(self.seed_bytes)?);
        Ok((row, proofs, seed))
    }
}
/// Actual terminal response together with the run header from the same transaction.
#[derive(Debug)]
pub struct SealAck {
    attempt: CanonicalAttempt,
    run: CanonicalRun,
}
impl SealAck {
    /// Consume the authenticated terminal response; no read permission is retained.
    pub fn into_parts(self) -> (CanonicalAttempt, CanonicalRun) {
        (self.attempt, self.run)
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
    /// Actual closed authority which admitted this immutable selection.
    pub fn closed(&self) -> &ClosedAttempt {
        &self.closed
    }
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
/// Immutable bounded result bytes and the digest of those exact bytes. This is
/// transport integrity, not Arrow, registry or scientific admission. A fresh
/// store read still independently verifies its returned payload.
#[derive(Debug)]
pub struct ResultPayload {
    bytes: pse_model::Bytes,
    digest: String,
    owner: Option<std::sync::Arc<pse_columnar::AllocationLease>>,
    file: Option<crate::canonical_files::SealedFile>,
    payload_index: u64,
    facts: Option<pse_relations::result_batch::BatchFacts>,
    admitted: Option<std::sync::Arc<pse_relations::result_batch::AdmittedResultFile>>,
    encoding: String,
    schema: String,
    batch_count: u64,
    body_operation: Option<String>,
}

/// An actual sealed file matched once against the controlled admitted encoder.
/// Copied descriptors or serialized statistics cannot construct this capability.
#[derive(Clone, Debug)]
pub struct AdmittedSealedResultFile {
    file: crate::canonical_files::SealedFile,
    facts: std::sync::Arc<[pse_relations::result_batch::BatchFacts]>,
}
impl AdmittedSealedResultFile {
    /// Bind the complete original bytes, schema relation and native member count.
    /// # Errors
    /// A physical file differs from the actual admitted encoder product.
    pub fn bind(
        file: crate::canonical_files::SealedFile,
        admitted: pse_relations::result_batch::AdmittedResultFile,
    ) -> Result<Self, CanonicalError> {
        let descriptor = file.descriptor();
        if descriptor.encoding != crate::canonical_files::ARROW_IPC_FILE
            || descriptor.schema != admitted.relation().to_string()
            || descriptor.batch_count != admitted.facts().len() as u64
            || descriptor.bytes != admitted.bytes().len() as u64
            || descriptor.digest != result_payload_digest(admitted.bytes())
        {
            return Err(CanonicalError::Configuration(
                "sealed result file differs from admitted bytes".into(),
            ));
        }
        Ok(Self {
            file,
            facts: admitted.facts().to_vec().into(),
        })
    }
}
impl ResultPayload {
    /// Consume native storage and calculate its integrity digest once.
    /// # Errors
    /// The immutable batch exceeds its existing transport bound.
    pub fn new(bytes: Vec<u8>) -> Result<Self, CanonicalError> {
        if bytes.is_empty() {
            return Err(CanonicalError::Configuration(
                "empty encoded payload; declare an empty demanded result set instead".into(),
            ));
        }
        bounded(&bytes, RESULT_BATCH_BYTES)?;
        let digest = result_payload_digest(&bytes);
        let encoding = if serde_json::from_slice::<serde_json::Value>(&bytes).is_ok() {
            crate::canonical_files::OPAQUE_JSON.into()
        } else {
            crate::canonical_files::OPAQUE_BYTES.into()
        };
        Ok(Self {
            bytes: bytes.into(),
            digest,
            owner: None,
            file: None,
            payload_index: 0,
            facts: None,
            admitted: None,
            encoding,
            schema: String::new(),
            batch_count: 0,
            body_operation: None,
        })
    }
    /// Consume bytes only after their actual encoder/transport allowance is charged.
    /// The owner follows every local route copy until response or physical drain.
    pub fn new_owned(
        bytes: Vec<u8>,
        owner: std::sync::Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, CanonicalError> {
        let mut payload = Self::new(bytes)?;
        payload.owner = Some(owner);
        Ok(payload)
    }
    /// Bind an already complete file member without copying its physical bytes.
    pub fn from_sealed(
        file: crate::canonical_files::SealedFile,
        index: u64,
    ) -> Result<Self, CanonicalError> {
        file.batch(index)?;
        Ok(Self {
            bytes: Vec::new().into(),
            digest: file.descriptor().digest.clone(),
            owner: None,
            file: Some(file),
            payload_index: index,
            facts: None,
            admitted: None,
            encoding: String::new(),
            schema: String::new(),
            batch_count: 0,
            body_operation: None,
        })
    }
    /// Carry the caller's actual prepaid encoder and metadata allowance.
    pub fn from_sealed_owned(
        file: crate::canonical_files::SealedFile,
        index: u64,
        owner: std::sync::Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, CanonicalError> {
        let mut payload = Self::from_sealed(file, index)?;
        payload.owner = Some(owner);
        Ok(payload)
    }
    /// Reference one member with facts from the verified admitted file owner.
    /// # Errors
    /// The requested native member is absent.
    pub fn from_admitted_sealed_owned(
        admitted: &AdmittedSealedResultFile,
        index: u64,
        owner: std::sync::Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, CanonicalError> {
        let facts = usize::try_from(index)
            .ok()
            .and_then(|index| admitted.facts.get(index))
            .cloned()
            .ok_or_else(|| CanonicalError::Configuration("admitted file member absent".into()))?;
        let mut payload = Self::from_sealed_owned(admitted.file.clone(), index, owner)?;
        payload.facts = Some(facts);
        Ok(payload)
    }
    /// Complete physical extent, independent of whether bytes remain resident.
    pub fn byte_len(&self) -> u64 {
        self.file
            .as_ref()
            .map(|f| f.descriptor().bytes)
            .unwrap_or_else(|| self.as_slice().len() as u64)
    }
    fn encoding(&self) -> &str {
        self.file
            .as_ref()
            .map(|file| file.descriptor().encoding.as_str())
            .unwrap_or(&self.encoding)
    }
    fn schema(&self) -> &str {
        self.file
            .as_ref()
            .map(|file| file.descriptor().schema.as_str())
            .unwrap_or(&self.schema)
    }
    fn batch_count(&self) -> u64 {
        self.file
            .as_ref()
            .map(|file| file.descriptor().batch_count)
            .unwrap_or(self.batch_count)
    }
    fn inline(&self) -> bool {
        self.file.is_none()
            && self.byte_len() <= pse_schema::catalog::substrate::RESULT_INLINE_BYTES as u64
    }
    fn inline_key(&self, fence: &AttemptFence, operation: &str) -> String {
        hash(
            "pse.execution.inline-body.v1",
            &[
                fence.attempt.as_bytes(),
                self.body_operation
                    .as_deref()
                    .unwrap_or(operation)
                    .as_bytes(),
                self.digest.as_bytes(),
                self.encoding().as_bytes(),
                self.schema().as_bytes(),
                &self.batch_count().to_le_bytes(),
            ],
        )
    }
    /// Own complete actual admitted bytes and facts before any physical file work.
    pub fn from_admitted_encoded_owned(
        admitted: std::sync::Arc<pse_relations::result_batch::AdmittedResultFile>,
        index: u64,
        body_operation: &str,
        owner: std::sync::Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, CanonicalError> {
        identity(body_operation)?;
        bounded(
            admitted.bytes(),
            pse_schema::catalog::substrate::RESULT_INLINE_BYTES,
        )?;
        let facts = usize::try_from(index)
            .ok()
            .and_then(|index| admitted.facts().get(index))
            .cloned()
            .ok_or_else(|| {
                CanonicalError::Configuration("admitted encoded member absent".into())
            })?;
        let digest = result_payload_digest(admitted.bytes());
        let schema = admitted.relation().to_string();
        let batch_count = admitted.facts().len() as u64;
        Ok(Self {
            bytes: Vec::new().into(),
            digest,
            owner: Some(owner),
            file: None,
            payload_index: index,
            facts: Some(facts),
            admitted: Some(admitted),
            encoding: crate::canonical_files::ARROW_IPC_FILE.into(),
            schema,
            batch_count,
            body_operation: Some(body_operation.into()),
        })
    }
    /// Carry an explicitly encoded native/opaque body. This proves integrity,
    /// never the scientific facts supplied by the admitted encoder constructor.
    #[allow(
        clippy::too_many_arguments,
        reason = "the original complete encoding and its member coordinates are independent premises"
    )]
    pub fn from_encoded_owned(
        bytes: Vec<u8>,
        encoding: &str,
        schema: &str,
        batch_count: u64,
        index: u64,
        body_operation: &str,
        owner: std::sync::Arc<pse_columnar::AllocationLease>,
    ) -> Result<Self, CanonicalError> {
        crate::canonical_files::validate_codec_schema(encoding, schema)?;
        identity(body_operation)?;
        if (encoding == crate::canonical_files::ARROW_IPC_FILE
            && (batch_count == 0 || index >= batch_count))
            || (encoding != crate::canonical_files::ARROW_IPC_FILE
                && (batch_count != 0 || index != 0))
        {
            return Err(CanonicalError::Configuration(
                "encoded payload member coordinate differs".into(),
            ));
        }
        let mut payload = Self::new_owned(bytes, owner)?;
        payload.encoding = encoding.into();
        payload.schema = schema.into();
        payload.batch_count = batch_count;
        payload.payload_index = index;
        payload.body_operation = Some(body_operation.into());
        Ok(payload)
    }
    /// Exact immutable bytes; no mutable alias or digest setter is exposed.
    pub fn as_slice(&self) -> &[u8] {
        self.admitted
            .as_ref()
            .map(|admitted| admitted.bytes())
            .unwrap_or_else(|| self.bytes.as_slice())
    }
    /// Existing framed payload digest, bound to this immutable allocation.
    pub fn digest(&self) -> &str {
        &self.digest
    }
}
fn payload_binding(batch: &ResultBatch) -> String {
    let mut payload = FramedHasher::new(Frame::CanonicalPayloadV1);
    payload
        .str("pse.execution.payload-binding.v2")
        .str(batch.placement.as_str())
        .str(
            batch
                .inline_key
                .as_deref()
                .or(batch.file_instance.as_deref())
                .unwrap_or_default(),
        )
        .str(&batch.digest)
        .u64(batch.payload_index)
        .u64(batch.payload_bytes)
        .str(&batch.encoding)
        .str(&batch.schema)
        .str(wire::INTERPRETATION);
    payload.finish_hash().to_hex()
}
/// Verify exact placement and bounded descriptor facts before reading contents.
pub fn validate_result_batch(batch: &ResultBatch) -> Result<(), CanonicalError> {
    crate::canonical_files::validate_codec_schema(&batch.encoding, &batch.schema)?;
    let locator = result_payload_locator(batch)?;
    if batch.key != result_batch_key(&batch.attempt, &batch.result_set, batch.ordinal)
        || batch.payload_bytes == 0
        || batch.payload_bytes > RESULT_BATCH_BYTES as u64
        || batch.digest.len() != 64
        || !batch.digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        || batch.payload_binding != payload_binding(batch)
        || (matches!(locator, PayloadLocator::Inline { .. })
            && batch.payload_bytes > pse_schema::catalog::substrate::RESULT_INLINE_BYTES as u64)
        || (batch.encoding != crate::canonical_files::ARROW_IPC_FILE && batch.payload_index != 0)
    {
        return Err(CanonicalError::Configuration(
            "result payload identity or coordinate mismatch".into(),
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
    if value.len() > pse_schema::catalog::substrate::METADATA_ID_BYTES {
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
    if !row.closed() || row.ingestion_open() {
        return Err(CanonicalError::Configuration(
            "attempt ingestion is not closed".into(),
        ));
    }
    let authority = row
        .close_generation
        .ok_or_else(|| CanonicalError::Configuration("closed authority missing".into()))?;
    Ok(ClosedAttempt {
        fence: AttemptFence {
            run: row.run.into(),
            attempt: row.key.into(),
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

// Compact discoverability is projected only from the current exact original source
// receipt. It never grants payload lifetime or infers scientific qualification.
#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ResearchInputSource {
    issuing_store: String,
    selection: String,
    dataset: String,
    revision: String,
    physical_context: String,
    publication: Option<String>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ResearchOperationSource {
    research: Option<ResearchInputSource>,
    context: Option<crate::canonical_contexts::ContextSelection>,
    revision: pse_ids::roles::SourceRevisionHash,
    physical_context: pse_ids::ContentHash,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResearchPhysicalSource {
    revision: String,
    identity: pse_ids::ContentHash,
}
fn projected_research_input(
    request: &RunRequest,
) -> Result<(Option<String>, Option<String>), CanonicalError> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&request.source_selection) else {
        return Ok((None, None));
    };
    let Some(array) = value.as_array() else {
        return Ok((None, None));
    };
    if array.first().and_then(serde_json::Value::as_u64) != Some(3) {
        return Ok((None, None));
    }
    if array.get(2).is_some_and(serde_json::Value::is_object) {
        let (_version, revision, physical, context, research): (
            u8,
            String,
            ResearchPhysicalSource,
            Option<crate::canonical_contexts::ContextSelection>,
            Option<ResearchInputSource>,
        ) = serde_json::from_value(value).map_err(|e| {
            CanonicalError::Configuration(format!("original study source selection: {e}"))
        })?;
        let Some(research) = research else {
            return Ok((None, None));
        };
        for key in [
            &research.issuing_store,
            &research.selection,
            &research.dataset,
            &research.revision,
            &research.physical_context,
            &physical.revision,
        ] {
            identity(key)?;
        }
        if revision != request.revision.key
            || research.revision != revision
            || !request
                .sources
                .iter()
                .any(|source| source.key == physical.revision)
        {
            return Err(CanonicalError::Configuration(
                "research study source or physical reference differs".into(),
            ));
        }
        pse_ids::ContentHash::parse_hex(&research.physical_context)
            .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
        if let Some(publication) = &research.publication {
            if publication != &research.revision {
                return Err(CanonicalError::Configuration(
                    "research study publication source differs".into(),
                ));
            }
        } else if context.as_ref().is_none_or(|c| {
            c.context.revision != revision
                || c.context.physical_context != research.physical_context
                || c.context.physical_revision != physical.revision
                || c.context.physical_source_identity != physical.identity.to_string()
        }) {
            return Err(CanonicalError::Configuration(
                "research study historical physical context differs".into(),
            ));
        }
        return Ok((Some(research.selection), Some(research.dataset)));
    }
    if !array.get(2).is_some_and(serde_json::Value::is_string) {
        return Ok((None, None));
    }
    let (_version, _identity, physical, _provenance, sources): (
        u8,
        serde_json::Value,
        pse_ids::ContentHash,
        serde_json::Value,
        Vec<ResearchOperationSource>,
    ) = serde_json::from_value(value).map_err(|e| {
        CanonicalError::Configuration(format!("original scientific source selection: {e}"))
    })?;
    let mut unique = Vec::new();
    for source in &sources {
        if !unique.contains(&source) {
            unique.push(source);
        }
    }
    // One compact column cannot represent a mixed input population. Preserve the
    // actual execution and complete original receipt with explicit absent metadata.
    if unique.len() != 1 {
        return Ok((None, None));
    }
    let source = unique[0];
    let Some(research) = source.research.as_ref() else {
        return Ok((None, None));
    };
    for key in [
        &research.issuing_store,
        &research.selection,
        &research.dataset,
        &research.revision,
        &research.physical_context,
    ] {
        identity(key)?;
    }
    if research.revision != request.revision.key
        || source.physical_context != physical
        || research.physical_context != physical.to_string()
    {
        return Err(CanonicalError::Configuration(
            "research source receipt revision or physical meaning differs".into(),
        ));
    }
    if let Some(publication) = &research.publication {
        if publication != &research.revision {
            return Err(CanonicalError::Configuration(
                "research publication source differs".into(),
            ));
        }
    } else if source.context.as_ref().is_none_or(|c| {
        c.context.revision != research.revision
            || c.context.physical_context != research.physical_context
            || c.context.source_identity != source.revision.to_string()
    }) {
        return Err(CanonicalError::Configuration(
            "research observation context differs".into(),
        ));
    }
    Ok((
        Some(research.selection.clone()),
        Some(research.dataset.clone()),
    ))
}

// One owning encoding for immutable run meaning, shared by publication and
// effect-free retry checks. Mutable attempt/cancellation state is excluded.
pub(crate) fn run_request_receipt(
    request: &RunRequest,
) -> Result<(CanonicalRun, Vec<Object>), CanonicalError> {
    identity(&request.key)?;
    identity(&request.revision.key)?;
    identity(&request.revision.problem)?;
    bounded(&request.request, crate::canonical::PAYLOAD_BYTES)?;
    bounded(&request.source_selection, crate::canonical::PAYLOAD_BYTES)?;
    bounded(&request.attestation, EXECUTION_METADATA_BYTES)?;
    match (
        request.purpose,
        &request.admission_study,
        &request.admission_point,
    ) {
        (pse_model::generated::enums::CanonicalRunPurpose::Execution, None, None) => {}
        (
            pse_model::generated::enums::CanonicalRunPurpose::StudyAdmission,
            Some(study),
            Some(point),
        ) => {
            identity(study)?;
            identity(point)?;
        }
        _ => {
            return Err(CanonicalError::Configuration(
                "run purpose and admission association differ".into(),
            ));
        }
    }
    let (research_selection, research_dataset) = projected_research_input(request)?;
    let row = CanonicalRun {
        research_selection,
        research_dataset,
        purpose: request.purpose,
        admission_study: request
            .admission_study
            .as_ref()
            .map(|key| key.as_str().to_owned()),
        admission_point: request
            .admission_point
            .as_ref()
            .map(|key| key.as_str().to_owned()),
        key: request.key.as_str().to_owned(),
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
        && saved.purpose == expected.purpose
        && saved.admission_study == expected.admission_study
        && saved.admission_point == expected.admission_point
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
    ) -> Result<crate::transport::client::IndexedResults, CanonicalError>
    where
        F: FnMut() -> Result<Q, CanonicalError>,
        Q: IntoFuture<
            Output = Result<crate::transport::client::IndexedResults, surrealdb_types::Error>,
        >,
    {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.protected_query(
                    &format!("execution-run:{run}"),
                    crate::canonical::QueryEffect::Mutation,
                    operation,
                    build,
                )
                .await
            })
            .await
    }
    /// Retain selected inputs and provenance atomically before native execution.
    pub async fn begin_run(&self, request: &RunRequest) -> Result<CanonicalRun, CanonicalError> {
        self.begin_run_authorized(request, None).await
    }
    /// Publish actual selected scientific inputs under the original lazy admission capability.
    pub async fn begin_run_with_study_admission(
        &self,
        request: &RunRequest,
        authority: &crate::canonical_studies::StudyAdmissionAuthority,
    ) -> Result<CanonicalRun, CanonicalError> {
        self.begin_run_authorized(request, Some(authority)).await
    }
    async fn begin_run_authorized(
        &self,
        request: &RunRequest,
        authority: Option<&crate::canonical_studies::StudyAdmissionAuthority>,
    ) -> Result<CanonicalRun, CanonicalError> {
        let admission = authority
            .map(|authority| authority.encode())
            .transpose()?
            .unwrap_or(Value::None);
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let (row, sources) = run_request_receipt(request)?;
            let encoded = wire::encode_canonical_runs(&row)?;
            self.ensure_writes()?;
            let response = self
                .protected_query(&row.problem, crate::canonical::QueryEffect::Mutation, "canonical_execution::begin_run", || {
                    Ok(self
                        .db
                        .query("RETURN fn::pse_execution_v1::begin_run($pse_rpc_expires_at, $row,$sources,$admission);")
                        .bind(("admission", admission.clone()))
                        .bind(("row", encoded.clone()))
                        .bind(("sources", sources.clone())))
                })
                .await;
            let write = response.and_then(|mut response| {
                let saved=wire::decode_canonical_runs(take_execution_ack("execution-registration",&mut response)?)?;
                if !same_run_receipt(&row,&saved) {return Err(CanonicalError::OperationReused);}
                Ok(saved)
            });
            let intent = json(&(&row.key, &row.purpose, &row.admission_study, &row.admission_point, &row.problem, &row.revision, row.source_sequence, row.request.as_slice(), row.source_selection.as_slice(), row.attestation.as_slice(), &row.interpretation))?;
            crate::canonical::settlement::settle_exact(
                "execution-registration", &row.key, &intent, write,
                async {
                    let saved = self.canonical_run(&row.key).await?;
                    if saved.as_ref().is_some_and(|saved| same_run_receipt(&row,saved)) { self.ensure_execution_retained(&row.key).await?; }
                    Ok(saved)
                },
                |saved| same_run_receipt(&row,saved), CanonicalError::source_transaction_refused,
            ).await
        }).await
            }).await
    }
    /// Bounded semantic ordering over one problem's recorded execution sequence.
    pub async fn execution_run_page(
        &self,
        problem: &str,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<RunSummary>, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Page, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(problem)?;
            if !(1..=64).contains(&limit) {
                return Err(CanonicalError::PayloadLimit);
            }
            let after = after
                .map(canonical_codec::encode_uint)
                .transpose()?
                .unwrap_or(canonical_codec::encode_int(-1)?);
            let mut result=bounded_query(self.db.query("SELECT key,problem,revision,source_sequence,sequence,interpretation,current_generation,current_attempt,cancelled,terminal_attempt,terminal_class FROM canonical_runs WHERE problem=$problem AND sequence>$after AND fn::pse_history_v1::visible_run($pse_rpc_expires_at,key) ORDER BY sequence LIMIT $limit;").bind(("problem",problem.to_owned())).bind(("after",after)).bind(("limit",limit))).await?;
            result
                .take::<Vec<Object>>(0)?
                .into_iter()
                .map(decode_run_summary)
                .collect()
        }).await
            }).await
        }).await
    }
    /// Exact scalar metadata for one retained seed; payload selection requires its manifest.
    pub async fn result_seed(
        &self,
        key: &str,
    ) -> Result<Option<pse_model::generated::runtime::canonical_result_seeds::Row>, CanonicalError>
    {
        crate::canonical::transport::metadata_scope(
            crate::canonical::MetadataResponse::Header,
            async {
                let activity = self.admit_activity()?;
                activity
                    .scope(async {
                        within_clock(original_deadline(REQUEST_TIMEOUT), async {
                            identity(key)?;
                            let mut result = bounded_query(
                        self.db
                            .query(
                                "RETURN fn::pse_history_v1::read_seed($pse_rpc_expires_at,$key);",
                            )
                            .bind(("key", key.to_owned())),
                    )
                    .await?;
                            result
                                .take::<Option<Object>>(0)?
                                .map(wire::decode_canonical_result_seeds)
                                .transpose()
                                .map_err(Into::into)
                        })
                        .await
                    })
                    .await
            },
        )
        .await
    }
    /// Filter sealed complete-original inventory before cutoff; admission remains independent.
    pub async fn result_seed_candidates(
        &self,
        layout: &str,
        preparation: &str,
        backend: &str,
        attempt: Option<&str>,
    ) -> Result<Vec<pse_model::generated::runtime::canonical_result_seeds::Row>, CanonicalError>
    {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Page, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let query = if attempt.is_some() {
                "RETURN { fn::pse_history_v1::check_identity($pse_rpc_expires_at,'attempt',$attempt); SELECT * FROM canonical_result_seeds WHERE layout=$layout AND preparation=$preparation AND backend=$backend AND attempt=$attempt AND fn::pse_execution_v1::seed_eligible($this) AND (SELECT VALUE key FROM canonical_result_retirements WHERE run=$parent.run LIMIT 1) = [] ORDER BY run_sequence DESC,attempt_generation DESC,step DESC,key DESC LIMIT 64; };"
            } else {
                "SELECT * FROM canonical_result_seeds WHERE layout=$layout AND preparation=$preparation AND backend=$backend AND fn::pse_execution_v1::seed_eligible($this) AND (SELECT VALUE key FROM canonical_result_retirements WHERE run=$parent.run LIMIT 1) = [] ORDER BY run_sequence DESC,attempt_generation DESC,step DESC,key DESC LIMIT 64;"
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
        }).await
            }).await
        }).await
    }
    /// Narrow immutable seed headers for one explicit attempt, ordered by scientific step.
    pub async fn result_seed_page(
        &self,
        attempt: &str,
        after: Option<u64>,
    ) -> Result<Vec<pse_model::generated::runtime::canonical_result_seeds::Row>, CanonicalError>
    {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Page, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(attempt)?;
            let after = after
                .map(canonical_codec::encode_uint)
                .transpose()?
                .unwrap_or(canonical_codec::encode_int(-1)?);
            let mut result=bounded_query(self.db.query("RETURN { fn::pse_history_v1::check_identity($pse_rpc_expires_at,'attempt',$attempt); SELECT * FROM canonical_result_seeds WHERE attempt=$attempt AND step>$after ORDER BY step LIMIT 64; };").bind(("attempt",attempt.to_owned())).bind(("after",after))).await?;
            result
                .take::<Vec<Object>>(0)?
                .into_iter()
                .map(|row| wire::decode_canonical_result_seeds(row).map_err(Into::into))
                .collect()
        }).await
            }).await
        }).await
    }
    /// Renew the current live worker fence; expired or cancelled workers cannot revive.
    pub async fn renew_attempt(
        &self,
        fence: &AttemptFence,
        lifetime: Duration,
    ) -> Result<CanonicalAttempt, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
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
                            "RETURN fn::pse_execution_v1::renew($pse_rpc_expires_at, $run,$attempt,$generation,$lifetime);",
                        )
                        .bind(("run", fence.run.as_str().to_owned()))
                        .bind(("attempt", fence.attempt.as_str().to_owned()))
                        .bind(("generation", generation.clone()))
                        .bind(("lifetime", lifetime)))
                })
                .await?;
            Ok(wire::decode_canonical_attempts(writer_response(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?)?)
        }).await
            }).await
    }
    /// Register only a structurally complete immutable chunked seed under its live gate.
    /// Eligibility still comes from the scientific completion owner and terminal descriptor.
    pub async fn register_result_seed(
        &self,
        fence: &AttemptFence,
        operation: &str,
        seed: &pse_model::generated::runtime::canonical_result_seeds::Row,
    ) -> Result<SeedAck, CanonicalError> {
        crate::canonical::migration::settlement_scope(fence, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(operation)?;
            identity(&seed.key)?;
            if seed.run != fence.run.as_str()
                || seed.attempt != fence.attempt.as_str()
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
            let result=self.protected_execution_query(&fence.run, "canonical_execution::register_result_seed", ||Ok(self.db.query("RETURN fn::pse_execution_v1::seed($pse_rpc_expires_at, $run,$attempt,$generation,$operation,$request,$seed);").bind(("run",fence.run.as_str().to_owned())).bind(("attempt",fence.attempt.as_str().to_owned())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("seed",encoded.clone())))).await;
            let acknowledged = || SeedAck { fence: fence.clone(), key: seed.key.clone(), operation: operation.into(), request: request.clone(), first: seed.first_ordinal, count: seed.batch_count, bytes: seed.payload_bytes };
            let write=result.and_then(|mut response| {
                let saved=wire::decode_canonical_result_seeds(writer_response(take_execution_ack("execution-seed",&mut response)?)?)?;
                if saved != *seed {return Err(CanonicalError::OperationReused);}
                Ok(fence.attempt().to_owned())
            });
            if matches!(&write,Err(CanonicalError::StudyCancellation { .. })) {return write.map(|_|acknowledged());}
            crate::canonical::settlement::settle_exact("execution-seed",operation,&request,write,
                self.settle_operation_for_run(operation,"seed",&request,Some(fence.run())),
                |attempt|attempt==fence.attempt(),CanonicalError::source_transaction_refused).await?;
            Ok(acknowledged())
        }).await
            }).await
        }).await
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
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(key)?;
            let mut response = protected_query("canonical_execution::recovery_snapshot", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("fn::pse_history_v1::point($pse_rpc_expires_at,$key); LET $point = SELECT * FROM ONLY type::record('canonical_study_points',$key);", "{point:$point,attempt:(SELECT * FROM ONLY type::record('canonical_attempts',$point.attempt)),study:(SELECT * FROM ONLY type::record('canonical_studies',$point.study)),run:(SELECT * FROM ONLY type::record('canonical_runs',$point.run))}")?).bind(("key",key.to_owned())))).await?;
            let mut row = ResultSlot::<Option<Object>>::returned()
                .take(&mut response)?
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
        }).await
            }).await
    }
    /// Project only the selected run/attempt metadata under the same original history guards.
    pub async fn canonical_result_origin(
        &self,
        run: &str,
        attempt: &str,
    ) -> Result<Option<CanonicalResultOrigin>, CanonicalError> {
        crate::canonical::transport::metadata_scope(
            crate::canonical::MetadataResponse::Header,
            async {
                let activity = self.admit_activity()?;
                activity
                    .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                        identity(run)?;
                        identity(attempt)?;
                        let mut response =
                            protected_query("canonical_execution::canonical_result_origin", || {
                                Ok(self
                                    .db
                                    .metadata_read_query(
                                        crate::canonical::MetadataResponse::Header,
                                        crate::canonical_history::run_query(
                                            RESULT_ORIGIN_SQL,
                                            "run",
                                        ),
                                    )
                                    .bind(("run", run.to_owned()))
                                    .bind(("attempt", attempt.to_owned())))
                            })
                            .await?;
                        response
                            .take::<Option<Object>>(0)?
                            .map(decode_result_origin)
                            .transpose()
                    }))
                    .await
            },
        )
        .await
    }
    /// Read a retained semantic execution independently of its current generation.
    pub async fn canonical_run(&self, key: &str) -> Result<Option<CanonicalRun>, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Header, async {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    let mut response = protected_query("canonical_execution::canonical_run", || Ok(
                        self.db
                            .metadata_read_query(crate::canonical::MetadataResponse::Header, "RETURN fn::pse_history_v1::read_run($pse_rpc_expires_at,$key);")
                            .bind(("key", key.to_owned()))))
                    .await?;
                    response
                        .take::<Option<Object>>(0)?
                        .map(wire::decode_canonical_runs)
                        .transpose()
                        .map_err(Into::into)
                })
                .await
            })
            .await
        }).await
    }
    /// Exact retained retries in a bounded generation prefix, under the history-owner guard.
    /// The endpoint is supplied by the caller's recorded run selection; this never follows
    /// a newer current pointer while reading subsequent pages.
    pub async fn canonical_attempt_page(
        &self,
        run: &str,
        after: Option<u64>,
        through: u64,
    ) -> Result<Vec<CanonicalAttemptMetadata>, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Page, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
            within_clock(original_deadline(REQUEST_TIMEOUT), async {
                identity(run)?;
                let mut response = bounded_query(self.db.query(crate::canonical_history::run_query(
                    "SELECT key,run,generation,phase IN ['terminal','terminal_compacted'] AS terminal,outcome,closed_manifest,close_generation FROM canonical_attempts WHERE run=$run AND generation>$after AND generation<=$through ORDER BY generation LIMIT 64;", "run"))
                    .bind(("run", run.to_owned()))
                    .bind(("after", after.map(canonical_codec::encode_uint).transpose()?.unwrap_or(canonical_codec::encode_int(-1)?)))
                    .bind(("through", canonical_codec::encode_uint(through)?))).await?;
                response.take::<Vec<Object>>(0)?.into_iter().map(decode_attempt_metadata).collect()
            }).await
        }).await
        }).await
    }
    /// Read an explicit attempt; terminal history is independent of the latest pointer.
    pub async fn canonical_attempt(
        &self,
        key: &str,
    ) -> Result<Option<CanonicalAttempt>, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Header, async {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    let mut response = protected_query("canonical_execution::canonical_attempt", || Ok(
                        self.db
                            .metadata_read_query(crate::canonical::MetadataResponse::Header, "RETURN fn::pse_history_v1::read_attempt($pse_rpc_expires_at,$key);")
                            .bind(("key", key.to_owned()))))
                    .await?;
                    response
                        .take::<Option<Object>>(0)?
                        .map(wire::decode_canonical_attempts)
                        .transpose()
                        .map_err(Into::into)
                })
                .await
            })
            .await
        }).await
    }
    /// Read a closed manifest; admission additionally requires a terminal attempt.
    pub async fn canonical_result_manifest(
        &self,
        key: &str,
    ) -> Result<Option<ResultManifest>, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Header, async {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    let mut response = protected_query("canonical_execution::canonical_result_manifest", || Ok(
                self.db
                    .metadata_read_query(crate::canonical::MetadataResponse::Header, "RETURN fn::pse_history_v1::read_manifest($pse_rpc_expires_at,$key);")
                    .bind(("key", key.to_owned()))))
            .await?;
                    let row = response
                        .take::<Option<Object>>(0)?
                        .map(wire::decode_canonical_result_manifests)
                        .transpose()?;
                    if let Some(row) = &row {
                        decode_result_descriptors(row)?;
                    }
                    Ok(row)
                })
                .await
            })
            .await
        }).await
    }
    async fn ensure_execution_retained(&self, run: &str) -> Result<(), CanonicalError> {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut retirement = bounded_query(
                self.db
                    .metadata_read_query(
                        crate::canonical::MetadataResponse::Header,
                        "SELECT key FROM ONLY type::record('canonical_result_retirements',$run);",
                    )
                    .bind(("run", run.to_owned())),
            )
            .await?;
            if retirement.take::<Option<Object>>(0)?.is_some() {
                return Err(CanonicalError::Configuration(
                    "execution results explicitly retired".into(),
                ));
            }
            Ok(())
        })
        .await
    }
    pub(crate) async fn settle_operation_for_run(
        &self,
        operation: &str,
        kind: &str,
        request: &[u8],
        expected_run: Option<&str>,
    ) -> Result<Option<String>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    #[cfg(all(test, feature = "canonical-tests"))]
                    if EXECUTION_ACK_READBACK_FAILURE.try_with(|fail| *fail).unwrap_or(false) {
                        bounded_query(self.db.control_query("THROW 'controlled_ack_readback_failure';")).await?;
                        return Err(CanonicalError::IncompleteResponse);
                    }
                    let mut response = protected_query("canonical_execution::settle_operation_for_run", || Ok(
                self.db
                    .control_query(
                        "RETURN fn::pse_history_v1::operation($pse_rpc_expires_at,$key,$expected_run);",
                    )
                    .bind(("key", operation.to_owned())).bind(("expected_run",expected_run.map(|run|Value::String(run.to_owned())).unwrap_or(Value::None)))))
            .await?;
                    let Some(object) = response.take::<Option<Object>>(0)? else {
                        return Ok(None);
                    };
                    let row = wire::decode_canonical_execution_operations(object)?;
                    if row.kind != kind || row.request.as_slice() != request {
                        return Err(CanonicalError::OperationReused);
                    }
                    Ok(row.attempt)
                })
                .await
            })
            .await
    }
    /// Claim a fresh actual attempt, or settle the same claim after lost acknowledgment.
    pub async fn claim_run(
        &self,
        run: &str,
        operation: &str,
        worker: &str,
        lifetime: Duration,
    ) -> Result<AttemptFence, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
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
            let result = self.protected_execution_query(run, "canonical_execution::claim_run", || Ok(self.db.query("RETURN fn::pse_execution_v1::claim($pse_rpc_expires_at, $run,$operation,$request,$attempt,$worker,$lifetime,$interpretation,NONE,NONE);").bind(("run",run.to_owned())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("attempt",attempt.clone())).bind(("worker",worker.to_owned())).bind(("lifetime",lifetime)).bind(("interpretation",wire::INTERPRETATION)))).await;
            let matches = |row: &CanonicalAttempt| row.run == run && row.key == attempt && row.claim_operation == operation && row.request.as_slice() == request;
            let write = result.and_then(|mut response| {
                let row=wire::decode_canonical_attempts(take_execution_ack("execution-claim",&mut response)?)?;
                if !matches(&row) { return Err(CanonicalError::OperationReused); }
                Ok(row)
            });
            let row = crate::canonical::settlement::settle_exact(
                "execution-claim", operation, &request, write,
                async { match self.settle_operation_for_run(operation,"claim",&request,Some(run)).await? {
                    Some(key) => Ok(Some(self.canonical_attempt(&key).await?.ok_or(CanonicalError::IncompleteResponse)?)),
                    None => Ok(None),
                } }, matches, CanonicalError::source_transaction_refused,
            ).await?;
            Ok(AttemptFence {
                run: row.run.into(),
                attempt: row.key.into(),
                generation: row.generation,
            })
        }).await
            }).await
    }
    /// Append an exact bounded batch under the attempt's live ingestion gate.
    pub async fn append_result_batch(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: ResultPayload,
        row_count: u64,
    ) -> Result<AppendAck, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    self.append_execution_batch(
                        fence,
                        operation,
                        name,
                        ordinal,
                        payload,
                        row_count,
                        None,
                        &[],
                    )
                    .await
                })
                .await
            })
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
        payload: ResultPayload,
        row_count: u64,
        block: &ResultBlock,
    ) -> Result<AppendAck, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    self.append_execution_batch(
                        fence,
                        operation,
                        name,
                        ordinal,
                        payload,
                        row_count,
                        Some(block),
                        &[],
                    )
                    .await
                })
                .await
            })
            .await
    }
    /// Atomically retain dense scientific output ranges with their one immutable payload.
    #[allow(
        clippy::too_many_arguments,
        reason = "the original payload, block descriptor and derived indexes are distinct premises of one fenced operation"
    )]
    pub async fn append_result_block_outputs(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
        ordinal: u64,
        payload: ResultPayload,
        row_count: u64,
        block: &ResultBlock,
        outputs: &[ResultOutput],
    ) -> Result<AppendAck, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    if outputs.windows(2).any(|pair| pair[0].key >= pair[1].key) {
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
                        outputs,
                    )
                    .await
                })
                .await
            })
            .await
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "one transaction validates separate fence, operation, batch and optional scientific index premises"
    )]
    fn append_execution_batch<'a>(
        &'a self,
        fence: &'a AttemptFence,
        operation: &'a str,
        name: &'a str,
        ordinal: u64,
        payload: ResultPayload,
        row_count: u64,
        block: Option<&'a ResultBlock>,
        outputs: &'a [ResultOutput],
    ) -> futures_util::future::BoxFuture<'a, Result<AppendAck, CanonicalError>> {
        // This owned submission boundary carries payload custody through all
        // settlement and clock scopes. Erase its internal future type once so
        // callers retain an explicit Send contract without propagating that
        // deeply nested implementation type through every public append path.
        Box::pin(async move {
        crate::canonical::migration::settlement_scope(fence, async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut prepared = ResultBlockAppend::prepare(fence, operation, name, ordinal, payload,
                row_count, block, outputs, None)?;
            self.prepare_result_payload(fence,&mut prepared).await?;
            let generation = canonical_codec::encode_uint(fence.generation)?;
            self.ensure_writes()?;
            let result = self.protected_execution_query(&fence.run, "canonical_execution::append_execution_batch", || {
                let item = &prepared.item;
                let query = self.db.query("RETURN fn::pse_execution_v1::append($pse_rpc_expires_at,$run,$attempt,$generation,$operation,$request,$set,$batch,$inline,$block,$outputs);")
                    .bind(("run",fence.run.as_str().to_owned())).bind(("attempt",fence.attempt.as_str().to_owned())).bind(("generation",generation.clone()))
                    .bind(("operation",item.get("operation").cloned().unwrap_or(Value::None)))
                    .bind(("request",item.get("request").cloned().unwrap_or(Value::None)))
                    .bind(("set",item.get("set").cloned().unwrap_or(Value::None)))
                    .bind(("batch",item.get("batch").cloned().unwrap_or(Value::None)))
                    .bind(("inline",item.get("inline").cloned().unwrap_or(Value::None)))
                    .bind(("block",item.get("block").cloned().unwrap_or(Value::None)))
                    .bind(("outputs",item.get("outputs").cloned().unwrap_or(Value::None)))
                    ;
                Ok(query.scientific_payload_owners(prepared.scientific_owners()))
            }).await;
            let write = result.and_then(|mut response| {
                let saved=writer_response(take_execution_ack("execution-append",&mut response)?)?;
                if saved != prepared.expected_response()? { return Err(CanonicalError::OperationReused); }
                Ok(fence.attempt().to_owned())
            });
            if matches!(&write,Err(CanonicalError::StudyCancellation { .. })) { return write.map(|_|prepared.acknowledgment); }
            let ack=&prepared.acknowledgment;
            crate::canonical::settlement::settle_exact(
                "execution-append",&ack.operation,&ack.request,write,
                self.settle_operation_for_run(&ack.operation,"append",&ack.request,Some(ack.fence.run())),
                |attempt| attempt==fence.attempt(),CanonicalError::source_transaction_refused,
            ).await?;
            Ok(prepared.acknowledgment)
        }).await
        }).await
        })
    }
    async fn prepare_result_payload(
        &self,
        fence: &AttemptFence,
        item: &mut ResultBlockAppend,
    ) -> Result<(), CanonicalError> {
        let sealed = if item.payload.inline() {
            None
        } else {
            Some(match item.payload.file.as_ref() {
                Some(file) => file.clone(),
                None => {
                    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
                    hash.str("pse.file.original-append.v1")
                        .part(&item.acknowledgment.request);
                    let intent = self
                        .reserve_result_file(
                            fence,
                            &format!("file:{}", item.acknowledgment.operation),
                            &hash.finish_hash().to_hex(),
                        )
                        .await?;
                    self.seal_file(
                        &intent,
                        item.payload.as_slice(),
                        item.payload.encoding(),
                        item.payload.batch_count(),
                        item.payload.schema(),
                    )
                    .await?
                }
            })
        };
        if sealed.as_ref().is_some_and(|sealed| {
            sealed.original_intent().instance() != sealed.descriptor().instance
        }) {
            return Err(CanonicalError::OperationReused);
        }
        let mut batch = wire::decode_canonical_result_batches(match item.item.get("batch") {
            Some(Value::Object(batch)) => batch.clone(),
            _ => return Err(CanonicalError::IncompleteResponse),
        })?;
        if let Some(sealed) = &sealed {
            batch.file_instance = Some(sealed.descriptor().instance.clone());
            batch.payload_bytes = sealed.descriptor().bytes;
            batch.schema = sealed.descriptor().schema.clone();
            batch.encoding = sealed.descriptor().encoding.clone();
        }
        batch.payload_index = item.payload.payload_index;
        // The original semantic request is stable before file work; its final
        // append receipt additionally seals the complete physical coordinates.
        batch.payload_binding = payload_binding(&batch);
        item.acknowledgment.receipt.payload_binding = batch.payload_binding.clone();
        let request = json(&(&item.acknowledgment.request, &batch.payload_binding))?;
        item.acknowledgment.request = request.clone();
        item.item
            .insert("request", Value::Bytes(Bytes::from(request)));
        item.item.insert(
            "batch",
            Value::Object(wire::encode_canonical_result_batches(&batch)?),
        );
        item.extent = result_metadata_extent(&Value::Object(item.item.clone()))?;
        item.payload.file = sealed;
        Ok(())
    }
    /// Submit multiple original immutable blocks in one server transaction.
    /// All actual per-block receipts are validated before any opaque ACK escapes.
    /// Uncertain transport settles every exact operation under this original clock.
    pub async fn append_result_blocks(
        &self,
        fence: &AttemptFence,
        mut items: Vec<ResultBlockAppend>,
    ) -> Result<Vec<AppendAck>, CanonicalError> {
        crate::canonical::migration::settlement_scope(fence, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            validate_append_group(fence, &items)?;
            for item in &mut items { self.prepare_result_payload(fence,item).await?; }
            validate_append_group(fence, &items)?;
            let generation = canonical_codec::encode_uint(fence.generation)?;
            let owners = items.iter().flat_map(ResultBlockAppend::scientific_owners).collect::<Vec<_>>();
            let mut shared = std::collections::BTreeSet::new();
            let encoded = items.iter().map(|item| {
                let mut encoded = item.item.clone();
                if let Some(Value::Object(body)) = encoded.get("inline")
                    && let Some(Value::String(key)) = body.get("key")
                    && !shared.insert(key.clone()) {
                    encoded.insert("inline", Value::None);
                }
                Value::Object(encoded)
            }).collect::<Vec<_>>();
            self.ensure_writes()?;
            // The bounded native driver encodes/checks the FULL query+bindings
            // against MESSAGE_BYTES before dispatch, including its live deadline.
            let result = self.protected_execution_query(&fence.run, "canonical_execution::append_result_blocks", ||
                Ok(self.db.query("RETURN fn::pse_execution_v1::append_many($pse_rpc_expires_at,$run,$attempt,$generation,$items);")
                    .bind(("run",fence.run.as_str().to_owned())).bind(("attempt",fence.attempt.as_str().to_owned()))
                    .bind(("generation",generation.clone())).bind(("items",encoded.clone())).scientific_payload_owners(owners.clone()))).await;
            #[cfg(all(test, feature = "canonical-tests"))]
            let result = if result.is_ok() && APPEND_GROUP_REPLY_LOSS.try_with(|lost| *lost).unwrap_or(false) {
                // Controlled actual-server post-commit reply loss. Settlement
                // still reads every genuine original operation receipt.
                Err(CanonicalError::IncompleteResponse)
            } else { result };
            let write = result.and_then(|mut response| {
                let mut saved=writer_response(take_execution_ack("execution-append-group",&mut response)?)?;
                let Value::Array(rows)=canonical_codec::required(&mut saved,"acknowledgments")? else {return Err(CanonicalError::IncompleteResponse);};
                if !saved.is_empty() || rows.len()!=items.len() {return Err(CanonicalError::IncompleteResponse);}
                for (row,item) in rows.iter().zip(&items) {
                    if row != Value::Object(item.expected_response()?) {return Err(CanonicalError::OperationReused);}
                }
                Ok(())
            });
            if matches!(&write,Err(CanonicalError::StudyCancellation { .. })) {return write.map(|_|Vec::new());}
            let request=json(&items.iter().map(|item|(&item.acknowledgment.operation,&item.acknowledgment.request)).collect::<Vec<_>>())?;
            crate::canonical::settlement::settle_exact(
                "execution-append-group",&items[0].acknowledgment.operation,&request,write,
                async {
                    for item in &items {
                        let ack=&item.acknowledgment;
                        if self.settle_operation_for_run(&ack.operation,"append",&ack.request,Some(ack.fence.run())).await?.is_none() {return Ok(None);}
                    }
                    Ok(Some(()))
                },|_|true,CanonicalError::source_transaction_refused,
            ).await?;
            Ok(items.into_iter().map(|item| item.acknowledgment).collect())
        }).await
            }).await
        }).await
    }
    /// Authenticate demanded zero-row membership without writing a schema-only IPC batch.
    pub async fn declare_empty_result_set(
        &self,
        fence: &AttemptFence,
        operation: &str,
        name: &str,
    ) -> Result<EmptySetAck, CanonicalError> {
        crate::canonical::migration::settlement_scope(fence, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(operation)?;
            identity(name)?;
            if name.len() > 128 { return Err(CanonicalError::PayloadLimit); }
            let key = result_set_key(fence.attempt(), name);
            let set = ResultSet { key: key.clone(), attempt: fence.attempt.as_str().to_owned(), name: name.into(), interpretation: wire::INTERPRETATION.into(), next_ordinal: 0, row_count: 0 };
            let encoded = wire::encode_canonical_result_sets(&set)?;
            let request = json(&(&fence.run, &fence.attempt, fence.generation, name, &key, wire::INTERPRETATION))?;
            let generation = canonical_codec::encode_uint(fence.generation)?;
            self.ensure_writes()?;
            let result = self.protected_execution_query(&fence.run, "canonical_execution::declare_empty_result_set", || Ok(self.db.query("RETURN fn::pse_execution_v1::empty($pse_rpc_expires_at,$run,$attempt,$generation,$operation,$request,$set);")
                .bind(("run",fence.run.as_str().to_owned())).bind(("attempt",fence.attempt.as_str().to_owned())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("set",encoded.clone())))).await;
            let write=result.and_then(|mut response| {
                let saved=wire::decode_canonical_result_sets(writer_response(take_execution_ack("execution-empty",&mut response)?)?)?;
                if saved != set {return Err(CanonicalError::OperationReused);}
                Ok(fence.attempt().to_owned())
            });
            if matches!(&write,Err(CanonicalError::StudyCancellation { .. })) {return write.map(|_|EmptySetAck {fence:fence.clone(),name:name.into(),key:key.clone(),operation:operation.into(),request:request.clone()});}
            crate::canonical::settlement::settle_exact("execution-empty",operation,&request,write,
                self.settle_operation_for_run(operation,"empty",&request,Some(fence.run())),
                |attempt|attempt==fence.attempt(),CanonicalError::source_transaction_refused).await?;
            Ok(EmptySetAck { fence: fence.clone(), name: name.into(), key, operation: operation.into(), request })
        }).await
            }).await
        }).await
    }
    /// Close the complete actual acknowledged inventory and admit its unchanged
    /// framed descriptor in the same fenced transaction. Server coverage checks
    /// do not reimplement the trusted writer's framed BLAKE3 algorithm.
    pub async fn close_acknowledged(
        &self,
        fence: &AttemptFence,
        operation: &str,
        acknowledgments: ResultAcknowledgments,
    ) -> Result<ClosedManifest, CanonicalError> {
        crate::canonical::migration::settlement_scope(fence, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(operation)?;
            acknowledgments.same_fence(fence)?;
            let (manifest, proofs, seeds) = acknowledgments.finish()?;
            let encoded = wire::encode_canonical_result_manifests(&manifest)?;
            let request = json(&(&fence.run,&fence.attempt,fence.generation,&manifest.digest))?;
            let generation = canonical_codec::encode_uint(fence.generation)?;
            self.ensure_writes()?;
            let result = self.protected_execution_query(&fence.run,"canonical_execution::close_acknowledged", || Ok(self.db.query("RETURN fn::pse_execution_v1::close_acknowledged($pse_rpc_expires_at,$run,$attempt,$generation,$operation,$request,$manifest,$proofs,$seeds);")
                .bind(("run",fence.run.as_str().to_owned())).bind(("attempt",fence.attempt.as_str().to_owned())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("manifest",encoded.clone())).bind(("proofs",proofs.clone())).bind(("seeds",seeds.clone())))).await;
            let uncertain=result.is_err();
            let closed = self.closed_response(result,operation,"close",&request,Some(fence)).await?;
            if uncertain {
                let saved=self.canonical_result_manifest(&manifest.key).await?.ok_or(CanonicalError::IncompleteResponse)?;
                if saved != manifest { return Err(CanonicalError::OperationReused); }
            }
            // A lost close reply retains exact settlement/readback comparison;
            // an acknowledged reply itself carries the actual closed authority.
            Ok(ClosedManifest { closed, row: manifest })
        }).await
            }).await
        }).await
    }
    /// Fence late batches before reconciling immutable metadata outside a transaction.
    pub async fn close_result_ingestion(
        &self,
        fence: &AttemptFence,
        operation: &str,
    ) -> Result<ClosedAttempt, CanonicalError> {
        crate::canonical::migration::settlement_scope(fence, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(operation)?;
            let request = json(&(&fence.run, &fence.attempt, fence.generation))?;
            let generation = canonical_codec::encode_uint(fence.generation)?;
            self.ensure_writes()?;
            let result = self.protected_execution_query(&fence.run, "canonical_execution::close_result_ingestion", || Ok(self.db.query("RETURN fn::pse_execution_v1::close($pse_rpc_expires_at, $run,$attempt,$generation,$operation,$request);").bind(("run",fence.run.as_str().to_owned())).bind(("attempt",fence.attempt.as_str().to_owned())).bind(("generation",generation.clone())).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))))).await;
            self.closed_response(result, operation, "close", &request, Some(fence))
                .await
        }).await
            }).await
        }).await
    }
    async fn closure_operation(
        &self,
        operation: &str,
        kind: &str,
        request: Option<&[u8]>,
    ) -> Result<Option<(String, u64)>, CanonicalError> {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response = bounded_query(
                self.db
                    .control_query(
                        "RETURN fn::pse_history_v1::operation($pse_rpc_expires_at,$operation,NONE);",
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
        }).await
    }
    async fn closed_response(
        &self,
        result: Result<crate::transport::client::IndexedResults, CanonicalError>,
        operation: &str,
        kind: &str,
        request: &[u8],
        expected: Option<&AttemptFence>,
    ) -> Result<ClosedAttempt, CanonicalError> {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let write = result.and_then(|mut response| {
                let row = writer_response(take_execution_ack("execution-close", &mut response)?)?;
                let mut value = closed(wire::decode_canonical_attempts(row)?)?;
                if let Some(fence) = expected {
                    if value.fence != *fence {
                        return Err(CanonicalError::OperationReused);
                    }
                    // The original close ACK retains its own generation even
                    // when the same attempt was subsequently cancelled.
                    value.authority = fence.generation;
                }
                Ok(value)
            });
            if matches!(&write, Err(CanonicalError::StudyCancellation { .. })) {
                return write;
            }
            let (write, acknowledged) = if expected.is_some() {
                (write, None)
            } else {
                // Recovery still observes its exact operation authority; a later
                // generation on an acknowledged row cannot replace that fact.
                match write {
                    Ok(value) => (Err(CanonicalError::IncompleteResponse), Some(value)),
                    Err(source) => (Err(source), None),
                }
            };
            crate::canonical::settlement::settle_exact(
                "execution-close",
                operation,
                request,
                write,
                async {
                    let Some((key, authority)) = self
                        .closure_operation(operation, kind, Some(request))
                        .await?
                    else {
                        return Ok(None);
                    };
                    let mut value = match acknowledged {
                        Some(value) => value,
                        None => closed(
                            self.canonical_attempt(&key)
                                .await?
                                .ok_or(CanonicalError::IncompleteResponse)?,
                        )?,
                    };
                    if value.fence.attempt.as_str() != key
                        || expected.is_some_and(|fence| value.fence != *fence)
                    {
                        return Err(CanonicalError::OperationReused);
                    }
                    value.authority = authority;
                    Ok(Some(value))
                },
                |value| expected.is_none_or(|fence| value.fence == *fence),
                CanonicalError::source_transaction_refused,
            )
            .await
        })
        .await
    }
    /// Revoke worker authority as a fresh run command. Migration closes this route.
    pub async fn cancel_run(&self, run: &str, operation: &str) -> Result<CanonicalRun, CanonicalError> {
        self.cancel_execution(run, operation, json(&(run, operation))?, None).await
    }
    /// Cancel only this original current attempt while retaining immutable request identity.
    /// This settlement route cannot revoke a replacement worker's authority.
    pub async fn cancel_attempt(&self, fence: &AttemptFence, operation: &str) -> Result<CanonicalRun, CanonicalError> {
        let request = json(&(fence.run(), fence.attempt(), fence.generation(), operation))?;
        crate::canonical::migration::settlement_scope(fence, self.cancel_execution(fence.run(), operation, request, Some(fence))).await
    }
    async fn cancel_execution(&self, run: &str, operation: &str, request: Vec<u8>, fence: Option<&AttemptFence>) -> Result<CanonicalRun, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(run)?;
            identity(operation)?;
            self.ensure_writes()?;
            let result = self
                .protected_execution_query(run, "canonical_execution::cancel_run", || {
                    Ok(self
                        .db
                        .query(if fence.is_some() {
                            "RETURN fn::pse_execution_v1::cancel_attempt($pse_rpc_expires_at,$run,$cancel_attempt,$cancel_generation,$operation,$request);"
                        } else {
                            "RETURN fn::pse_execution_v1::cancel($pse_rpc_expires_at,$run,$operation,$request);"
                        })
                        .bind(("cancel_attempt", fence.map(|fence| fence.attempt().to_owned())))
                        .bind(("cancel_generation", fence.map(|fence| canonical_codec::encode_uint(fence.generation())).transpose()?))
                        .bind(("run", run.to_owned()))
                        .bind(("operation", operation.to_owned()))
                        .bind(("request", Bytes::from(request.clone()))))
                })
                .await;
            let write=result.and_then(|mut response| {
                let saved=wire::decode_canonical_runs(take_execution_ack("execution-cancel",&mut response)?)?;
                if saved.key!=run {return Err(CanonicalError::OperationReused);}
                Ok(saved)
            });
            crate::canonical::settlement::settle_exact(
                "execution-cancel",operation,&request,write,
                async {
                    // Cancellation operations deliberately have no attempt.
                    let mut response=bounded_query(self.db.control_query("RETURN fn::pse_history_v1::operation($pse_rpc_expires_at,$operation,NONE);").bind(("operation",operation.to_owned()))).await?;
                    let Some(row)=response.take::<Option<Object>>(0)? else {return Ok(None);};
                    let row=wire::decode_canonical_execution_operations(row)?;
                    if row.kind!="cancel" || row.request.as_slice()!=request {return Err(CanonicalError::OperationReused);}
                    Ok(Some(self.canonical_run(run).await?.ok_or(CanonicalError::IncompleteResponse)?))
                },|saved|saved.key==run,CanonicalError::source_transaction_refused,
            ).await
        }).await
            }).await
    }
    /// Current recovery owner closes expired or cancelled staging; no stale worker can use this authority.
    pub async fn recover_closed_attempt(
        &self,
        run: &str,
        operation: &str,
    ) -> Result<ClosedAttempt, CanonicalError> {
        let observed = self.canonical_run(run).await?.ok_or(CanonicalError::IncompleteResponse)?;
        let current = observed.current_attempt.ok_or(CanonicalError::IncompleteResponse)?;
        let original = self.canonical_attempt(&current).await?.ok_or(CanonicalError::IncompleteResponse)?;
        let settlement = AttemptFence { run: original.run.into(), attempt: original.key.into(), generation: original.generation };
        crate::canonical::migration::settlement_scope(&settlement, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            identity(run)?;
            identity(operation)?;
            let request = json(&(run, operation))?;
            self.ensure_writes()?;
            let result = self
                .protected_execution_query(run, "canonical_execution::recover_closed_attempt", || {
                    Ok(self
                        .db
                        .query("RETURN fn::pse_execution_v1::recover($pse_rpc_expires_at, $run,$operation,$request);")
                        .bind(("run", run.to_owned()))
                        .bind(("operation", operation.to_owned()))
                        .bind(("request", Bytes::from(request.clone()))))
                })
                .await;
            self.closed_response(result, operation, "recover", &request, None)
                .await
        }).await
            }).await
        }).await
    }
    /// Resume an already closed live operation after restart without repeating native work.
    pub async fn resume_closed_attempt(
        &self,
        attempt: &str,
        operation: &str,
    ) -> Result<ClosedAttempt, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response=bounded_query(self.db.query("RETURN fn::pse_history_v1::operation($pse_rpc_expires_at,$operation,NONE);").bind(("operation",operation.to_owned()))).await?;
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
        }).await
            }).await
    }
    /// Reconcile a joined optional progress writer's uncertain tail. Actual ACKs
    /// still describe only the observed prefix; this readback never promotes the
    /// original sampled counters or grants permission from guessed totals.
    pub async fn reconcile_progress_uncertainty(
        &self,
        closed: &ClosedAttempt,
        acknowledgments: ResultAcknowledgments,
        queued: u64,
    ) -> Result<ClosedManifest, CanonicalError> {
        crate::canonical::migration::settlement_scope(closed.fence(), async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            acknowledgments.same_fence(closed.fence())?;
            if closed.authority != closed.fence.generation || acknowledgments.rows("__progress") > queued { return Err(CanonicalError::OperationReused); }
            let (expected,proofs,seeds)=acknowledgments.finish()?;
            let generation=canonical_codec::encode_uint(closed.fence.generation)?;
            let queued_value=canonical_codec::encode_uint(queued)?;
            let mut response=self.protected_execution_query(closed.fence.run(),"canonical_execution::reconcile_progress_uncertainty", || Ok(self.db.query("RETURN fn::pse_execution_v1::check_closed_acknowledgments($pse_rpc_expires_at,$run,$attempt,$generation,$proofs,$seeds,$queued);")
                .bind(("run",closed.fence.run.as_str().to_owned())).bind(("attempt",closed.fence.attempt.as_str().to_owned())).bind(("generation",generation.clone())).bind(("proofs",proofs.clone())).bind(("seeds",seeds.clone())).bind(("queued",queued_value.clone())))).await?;
            let mut saved=writer_response(response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse)?)?;
            if saved.remove("valid")!=Some(Value::Bool(true)) || !saved.is_empty() { return Err(CanonicalError::IncompleteResponse); }
            // Independently read every frozen metadata page and frame the actual
            // inventory, including any unobserved progress tail, under this clock.
            let actual=self.reconcile_closed_attempt(closed).await?;
            let expected=decode_result_descriptors(&expected)?;
            let actual_sets=decode_result_descriptors(actual.row())?;
            for set in &actual_sets {
                let prefix=expected.iter().find(|item|item.key==set.key);
                if set.name=="__progress" {
                    if set.row_count>queued || prefix.is_some_and(|prefix|set.batch_count<prefix.batch_count || set.row_count<prefix.row_count) { return Err(CanonicalError::OperationReused); }
                } else if prefix!=Some(set) { return Err(CanonicalError::OperationReused); }
            }
            if expected.iter().any(|set|actual_sets.iter().all(|actual|actual.key!=set.key)) { return Err(CanonicalError::OperationReused); }
            Ok(actual)
        }).await
            }).await
        }).await
    }
    /// Reconcile exact immutable batch membership in bounded metadata pages, then admit its descriptor.
    pub async fn reconcile_closed_attempt(
        &self,
        closed: &ClosedAttempt,
    ) -> Result<ClosedManifest, CanonicalError> {
        crate::canonical::migration::settlement_scope(closed.fence(), async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let attempt = self
                .canonical_attempt(&closed.fence.attempt)
                .await?
                .ok_or_else(|| CanonicalError::Configuration("closed attempt unavailable".into()))?;
            if !attempt.closed()
                || attempt.ingestion_open()
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
                    let mut page=bounded_query(self.db.query("SELECT key,attempt,result_set,ordinal,digest,row_count,payload_binding FROM canonical_result_batches WHERE result_set=$set AND ordinal >= $next ORDER BY ordinal LIMIT 64;").bind(("set",set.key.clone())).bind(("next",canonical_codec::encode_uint(next)?))).await?;
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
                        let binding=canonical_codec::decode_string(canonical_codec::required(&mut batch,"payload_binding")?)?;
                        digest.str(&key).str(&payload_digest).u64(rows).str(&binding);
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
                            .query("RETURN fn::pse_execution_v1::manifest($pse_rpc_expires_at, $run,$manifest);")
                            .bind(("run", closed.fence.run.as_str().to_owned()))
                            .bind(("manifest", encoded.clone())))
                    },
                )
                .await;
            let write=result.and_then(|mut response| {
                let saved=wire::decode_canonical_result_manifests(take_execution_ack("execution-reconcile",&mut response)?)?;
                if saved!=row {return Err(CanonicalError::OperationReused);}
                Ok(saved)
            });
            crate::canonical::settlement::settle_exact(
                "execution-reconcile", &row.key, row.descriptors.as_slice(), write,
                async { self.ensure_execution_retained(closed.fence.run()).await?; self.canonical_result_manifest(&row.key).await },
                |saved| saved == &row, CanonicalError::source_transaction_refused,
            ).await?;
            Ok(ClosedManifest {
                closed: closed.clone(),
                row,
            })
        }).await
            }).await
        }).await
    }
    /// Atomically expose the frozen selection with the actual scientific terminal class.
    ///
    /// # Safety
    /// The controlled scientific writer must have completed and admitted the exact
    /// observations described by this manifest, and must supply their actual terminal
    /// class and completion. In particular, success requires the scientific owner's
    /// numerical conclusion, physical eligibility and contextual accuracy admission.
    /// CompleteOriginalInventory additionally asserts that every registered seed belongs
    /// to the fully exported original completion and its per-step seed permissions.
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
        seeds: SeedEligibility,
    ) -> Result<CanonicalAttempt, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    self.seal_execution(manifest, operation, outcome, completion, None, seeds, None)
                        .await
                        .map(|ack| ack.attempt)
                })
                .await
            })
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
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(REQUEST_TIMEOUT), async {
                    identity(study)?;
                    self.seal_execution(
                        manifest,
                        operation,
                        outcome,
                        completion,
                        Some(study),
                        SeedEligibility::Unavailable,
                        None,
                    )
                    .await
                    .map(|ack| ack.attempt)
                })
                .await
            })
            .await
    }
    /// Seal joined scientific observations and return both actual terminal headers.
    /// # Safety
    /// The caller must satisfy the same original scientific and seed admission
    /// obligations as `seal_attempt` for this exact frozen manifest.
    #[allow(
        unsafe_code,
        reason = "controlled scientific owner supplies original admission"
    )]
    pub async unsafe fn seal_acknowledged(
        &self,
        manifest: &ClosedManifest,
        operation: &str,
        outcome: TerminalClass,
        completion: &[u8],
        seeds: SeedEligibility,
    ) -> Result<SealAck, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let header: serde_json::Value = serde_json::from_slice(completion)
                    .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
                if header.get("version").and_then(serde_json::Value::as_u64) != Some(1)
                    || !header.get("inline").is_some_and(serde_json::Value::is_null)
                {
                    return Err(CanonicalError::IncompleteResponse);
                }
                let count = header
                    .get("batch_count")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(CanonicalError::IncompleteResponse)?;
                let bytes = header
                    .get("payload_bytes")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(CanonicalError::IncompleteResponse)?;
                let descriptors = decode_result_descriptors(&manifest.row)?;
                let chunk = descriptors
                    .iter()
                    .find(|set| set.name == "__completion")
                    .ok_or(CanonicalError::IncompleteResponse)?;
                if chunk.batch_count != count || chunk.row_count != bytes {
                    return Err(CanonicalError::OperationReused);
                }
                let mut coverage = Object::new();
                coverage.insert("count", canonical_codec::encode_uint(count)?);
                coverage.insert("bytes", canonical_codec::encode_uint(bytes)?);
                coverage.insert(
                    "progress",
                    canonical_codec::encode_uint(
                        descriptors
                            .iter()
                            .find(|set| set.name == "__progress")
                            .map_or(0, |set| set.row_count),
                    )?,
                );
                self.seal_execution(
                    manifest,
                    operation,
                    outcome,
                    completion,
                    None,
                    seeds,
                    Some(coverage),
                )
                .await
            })
            .await
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "exact seal protocol carries distinct fence, outcome, study and seed authorities"
    )]
    async fn seal_execution(
        &self,
        manifest: &ClosedManifest,
        operation: &str,
        outcome: TerminalClass,
        completion: &[u8],
        study: Option<&str>,
        seeds: SeedEligibility,
        coverage: Option<Object>,
    ) -> Result<SealAck, CanonicalError> {
        crate::canonical::migration::settlement_scope(&manifest.closed.fence, async {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
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
                seeds,
            ))?;
            bounded(&request, crate::canonical::PAYLOAD_BYTES)?;
            let authority = canonical_codec::encode_uint(manifest.closed.authority)?;
            self.ensure_writes()?;
            let query = if study.is_some() {
                "RETURN fn::pse_execution_v1::study_seal_ack($pse_rpc_expires_at, $study,$run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion);"
            } else {
                "RETURN fn::pse_execution_v1::seal_ack($pse_rpc_expires_at, $run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion,$seed_eligible,$coverage);"
            };
            let result = self
                .protected_execution_query(&fence.run, "canonical_execution::seal_execution", || {
                    Ok(self
                        .db
                        .query(query)
                        .bind(("study", study.map(str::to_owned)))
                        .bind(("run", fence.run.as_str().to_owned()))
                        .bind(("attempt", fence.attempt.as_str().to_owned()))
                        .bind(("authority", authority.clone()))
                        .bind(("operation", operation.to_owned()))
                        .bind(("request", Bytes::from(request.clone())))
                        .bind(("manifest", manifest.row.key.clone()))
                        .bind(("digest", manifest.row.digest.clone()))
                        .bind(("outcome", outcome.as_str()))
                        .bind(("completion", Bytes::from(completion.to_vec())))
                        .bind(("seed_eligible", seeds == SeedEligibility::CompleteOriginalInventory))
                        .bind(("coverage", coverage.clone().map(Value::Object).unwrap_or(Value::None))))
                })
                .await;
            let matches=|saved:&SealAck| saved.attempt.run==saved.run.key
                && saved.attempt.key==fence.attempt() && saved.run.key==fence.run()
                && saved.run.terminal_attempt.as_deref()==Some(saved.attempt.key.as_str())
                && saved.attempt.terminal() && saved.attempt.closed_manifest.as_deref()==Some(manifest.row.key.as_str())
                && saved.attempt.completion.as_ref().map(|bytes|bytes.as_slice())==Some(completion);
            let write=result.and_then(|mut response| {
                let mut row=writer_response(take_execution_ack("execution-seal",&mut response)?)?;
                let attempt=match canonical_codec::required(&mut row,"attempt")? {Value::Object(row)=>wire::decode_canonical_attempts(row)?,_=>return Err(CanonicalError::IncompleteResponse)};
                let run=match canonical_codec::required(&mut row,"run")? {Value::Object(row)=>wire::decode_canonical_runs(row)?,_=>return Err(CanonicalError::IncompleteResponse)};
                let saved=SealAck {attempt,run};
                if !row.is_empty() || !matches(&saved) {return Err(CanonicalError::OperationReused);}
                Ok(saved)
            });
            if matches!(&write,Err(CanonicalError::StudyCancellation { .. })) {return write;}
            crate::canonical::settlement::settle_exact(
                "execution-seal",operation,&request,write,
                async {match self.settle_operation_for_run(operation,"seal",&request,Some(fence.run())).await? {
                    Some(key)=>{
                        let attempt=self.canonical_attempt(&key).await?.ok_or(CanonicalError::IncompleteResponse)?;
                        let run=self.canonical_run(fence.run()).await?.ok_or(CanonicalError::IncompleteResponse)?;
                        Ok(Some(SealAck {attempt,run}))
                    },None=>Ok(None),
                }},matches,CanonicalError::source_transaction_refused,
            ).await
        }).await
        }).await
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "isolated protocol fixtures reserve their actual synthetic payload allowance"
)]
fn test_payload_owner(bytes: usize) -> std::sync::Arc<pse_columnar::AllocationLease> {
    let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(bytes));
    let reservation =
        pse_columnar::MemoryConsumer::new("canonical:payload-fixture").register(&pool);
    reservation.try_grow(bytes).unwrap();
    pse_columnar::AllocationLease::new(reservation)
}

#[cfg(test)]
mod canonical_result_admission_unit {
    use super::*;
    fn ack(
        fence: &AttemptFence,
        name: &str,
        ordinal: u64,
        operation: &str,
        bytes: &[u8],
        rows: u64,
    ) -> AppendAck {
        let result_set = result_set_key(fence.attempt(), name);
        AppendAck {
            receipt: AppendReceipt {
                key: result_batch_key(fence.attempt(), &result_set, ordinal),
                attempt: fence.attempt().into(),
                result_set,
                ordinal,
                digest: result_payload_digest(bytes),
                row_count: rows,
                payload_binding: "synthetic-file-binding".into(),
            },
            fence: fence.clone(),
            name: name.into(),
            operation: operation.into(),
            request: operation.as_bytes().to_vec(),
        }
    }
    pub(super) fn append_item(
        fence: &AttemptFence,
        name: &str,
        ordinal: u64,
        bytes: Vec<u8>,
    ) -> ResultBlockAppend {
        let set = result_set_key(fence.attempt(), name);
        let key = result_batch_key(fence.attempt(), &set, ordinal);
        let payload = ResultPayload::new(bytes).unwrap();
        let block = ResultBlock {
            key: key.clone(),
            batch: key.clone(),
            result_set: set,
            output: name.into(),
            partition: "0".into(),
            ordinal,
            start: ordinal,
            end: ordinal + 1,
            rows: 1,
            columns: 1,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: payload.as_slice().len() as u64,
            payload_digest: payload.digest().into(),
            statistics: None,
            interpretation: wire::INTERPRETATION.into(),
        };
        // Protocol fixtures own synthetic tiny bytes; production callers pass
        // the actual reserved immutable encoded-item/transport owner.
        ResultBlockAppend::new(
            fence,
            &format!("batch:{key}"),
            name,
            ordinal,
            payload,
            1,
            &block,
            &[],
            test_payload_owner(1024 * 1024),
        )
        .unwrap()
    }
    #[test]
    fn append_driver_keeps_distinct_metadata_and_payload_reservations() {
        let mut item = append_item(&ack_fence(), "values", 0, vec![1]);
        let metadata = test_payload_owner(4096);
        let payload = test_payload_owner(128);
        let metadata_weak = std::sync::Arc::downgrade(&metadata);
        let payload_weak = std::sync::Arc::downgrade(&payload);
        item._owner = Some(metadata.clone());
        item.payload.owner = Some(payload.clone());
        let retained = item.scientific_owners();
        assert_eq!(retained.len(), 2);
        assert!(
            retained
                .iter()
                .any(|owner| std::sync::Arc::ptr_eq(owner, &metadata))
        );
        assert!(
            retained
                .iter()
                .any(|owner| std::sync::Arc::ptr_eq(owner, &payload))
        );
        drop((item, metadata, payload));
        assert!(metadata_weak.upgrade().is_some());
        assert!(payload_weak.upgrade().is_some());
        drop(retained);
        assert!(metadata_weak.upgrade().is_none());
        assert!(payload_weak.upgrade().is_none());

        let mut item = append_item(&ack_fence(), "shared", 0, vec![1]);
        item.payload.owner = item._owner.clone();
        assert_eq!(item.scientific_owners().len(), 1);
    }
    #[test]
    fn copied_statistics_and_equal_replay_cannot_mint_exact_first_append_facts() {
        let fence = ack_fence();
        for _ in 0..2 {
            let seed = append_item(&fence, "values", 0, vec![1]);
            let mut block =
                wire::decode_canonical_result_blocks(match seed.item.get("block").unwrap() {
                    Value::Object(block) => block.clone(),
                    _ => panic!("fixture block"),
                })
                .unwrap();
            block.statistics = Some(br#"{"rows":1,"minimum":-999,"maximum":-999}"#.to_vec().into());
            let result = ResultBlockAppend::new(
                &fence,
                "forged-equal-replay",
                "values",
                0,
                seed.payload,
                1,
                &block,
                &[],
                test_payload_owner(1024 * 1024),
            );
            assert!(
                matches!(result, Err(CanonicalError::Configuration(message)) if message.contains("actual admitted-file proof"))
            );
        }
    }
    #[test]
    fn bounded_append_group_preserves_individual_request_and_exact_fence_order() {
        let fence = ack_fence();
        let first = append_item(&fence, "values", 0, vec![1]);
        let expected = first.acknowledgment.request.clone();
        let mut group = vec![
            first,
            append_item(&fence, "other", 0, vec![3]),
            append_item(&fence, "values", 1, vec![2]),
        ];
        validate_append_group(&fence, &group).unwrap();
        assert_eq!(group[0].acknowledgment.request, expected);
        assert!(
            group
                .iter()
                .all(|item| item.encoded_extent() > item.acknowledgment.request.len())
        );
        group.swap(0, 2);
        assert!(validate_append_group(&fence, &group).is_err());
        let foreign = AttemptFence {
            generation: 8,
            ..fence.clone()
        };
        assert!(validate_append_group(&foreign, &group).is_err());
        assert!(validate_append_group(&fence, &[]).is_err());
        let duplicate = vec![
            append_item(&fence, "values", 0, vec![1]),
            append_item(&fence, "values", 0, vec![1]),
        ];
        assert!(validate_append_group(&fence, &duplicate).is_err());
    }
    #[test]
    fn bounded_append_group_budgets_metadata_separately_from_file_bytes() {
        let fence = ack_fence();
        let files = (0..6)
            .map(|ordinal| append_item(&fence, "values", ordinal, vec![7; RESULT_BATCH_BYTES]))
            .collect::<Vec<_>>();
        // File contents travel separately; their summed extent is not the
        // metadata transaction's wire budget.
        assert!(validate_append_group(&fence, &files).is_ok());
        let group = (0..8192)
            .map(|ordinal| append_item(&fence, "values", ordinal, vec![7]))
            .collect::<Vec<_>>();
        assert!(group.len() < wire::RESULT_INDEX_RECORDS);
        assert!(
            group
                .iter()
                .map(ResultBlockAppend::encoded_extent)
                .sum::<usize>()
                > crate::canonical::PAYLOAD_BYTES
        );
        assert!(validate_append_group(&fence, &group[..16]).is_ok());
        assert!(matches!(
            validate_append_group(&fence, &group),
            Err(CanonicalError::PayloadLimit)
        ));
    }
    fn ack_fence() -> AttemptFence {
        AttemptFence {
            run: "ack-run".into(),
            attempt: "ack-attempt".into(),
            generation: 7,
        }
    }
    #[test]
    fn actual_ack_accumulator_preserves_framing_and_only_last_exact_replay() {
        let fence = ack_fence();
        let mut acks = ResultAcknowledgments::new(&fence);
        let first = ack(&fence, "values", 0, "first", &[0, 1], 2);
        let mut expected = FramedHasher::new(Frame::CanonicalPayloadV1);
        expected
            .str("pse.execution.batch-selection.v1")
            .str(&first.key)
            .str(&first.digest)
            .u64(first.row_count)
            .str(&first.payload_binding);
        acks.append(first).unwrap();
        acks.append(ack(&fence, "values", 0, "first", &[0, 1], 2))
            .unwrap();
        assert!(
            acks.append(ack(&fence, "values", 0, "other-operation", &[0, 1], 2))
                .is_err()
        );
        assert!(
            acks.append(ack(&fence, "values", 2, "gap", &[2], 1))
                .is_err()
        );
        let second = ack(&fence, "values", 1, "second", &[2], 1);
        expected
            .str(&second.key)
            .str(&second.digest)
            .u64(second.row_count)
            .str(&second.payload_binding);
        acks.append(second).unwrap();
        assert!(
            acks.append(ack(&fence, "values", 0, "first", &[0, 1], 2))
                .is_err()
        );
        assert_eq!(acks.sets["values"].next, 2);
        assert_eq!(acks.rows("values"), 3);
        acks.complete("values").unwrap();
        let (manifest, proofs, _) = acks.finish().unwrap();
        let sets = decode_result_descriptors(&manifest).unwrap();
        assert_eq!(sets[0].batches_digest, expected.finish_hash().to_hex());
        assert_eq!(sets[0].batch_count, 2);
        assert_eq!(proofs.len(), 1);
    }
    #[test]
    fn demanded_empty_requires_actual_declaration_and_complete_producer() {
        let fence = ack_fence();
        let mut absent = ResultAcknowledgments::new(&fence);
        absent.expect("empty").unwrap();
        assert!(absent.complete("empty").is_err());
        assert!(absent.finish().is_err());
        let mut declared = ResultAcknowledgments::new(&fence);
        declared.expect("empty").unwrap();
        declared
            .empty(EmptySetAck {
                fence: fence.clone(),
                name: "empty".into(),
                key: result_set_key(fence.attempt(), "empty"),
                operation: "declare".into(),
                request: b"declare".to_vec(),
            })
            .unwrap();
        assert!(!declared.sets["empty"].complete);
        declared.complete("empty").unwrap();
        let (manifest, proofs, _) = declared.finish().unwrap();
        let descriptors = decode_result_descriptors(&manifest).unwrap();
        assert_eq!(
            (descriptors[0].batch_count, descriptors[0].row_count),
            (0, 0)
        );
        assert_eq!(proofs[0].get("kind"), Some(&Value::String("empty".into())));
    }
    #[test]
    fn actual_ack_accumulator_refuses_foreign_attempt_and_bounds_resident_sets() {
        let fence = ack_fence();
        let mut acks = ResultAcknowledgments::new(&fence);
        let mut foreign = fence.clone();
        foreign.generation += 1;
        assert!(
            acks.append(ack(&foreign, "values", 0, "wrong-fence", &[1], 1))
                .is_err()
        );
        assert!(acks.sets.is_empty());
        for set in 0..RESULT_SETS {
            acks.expect(&format!("set-{set}")).unwrap();
        }
        assert!(matches!(
            acks.expect("one-too-many"),
            Err(CanonicalError::PayloadLimit)
        ));
        assert_eq!(acks.sets.len(), RESULT_SETS);
        assert!(acks.finish().is_err());
    }
    #[test]
    fn actual_seed_ack_requires_contiguous_acknowledged_chunks() {
        let fence = ack_fence();
        let mut acks = ResultAcknowledgments::new(&fence);
        acks.append(ack(&fence, "__seeds", 0, "chunk-0", &[1, 2], 2))
            .unwrap();
        let seed = |first, count| SeedAck {
            fence: fence.clone(),
            key: "seed".into(),
            operation: "seed-op".into(),
            request: b"seed-op".to_vec(),
            first,
            count,
            bytes: 2,
        };
        assert!(acks.seed(seed(1, 1)).is_err());
        assert!(acks.seed(seed(0, 2)).is_err());
        acks.seed(seed(0, 1)).unwrap();
        acks.seed(seed(0, 1)).unwrap();
        assert_eq!(
            (acks.seed_count, acks.seed_batches, acks.seed_bytes),
            (1, 1, 2)
        );
        acks.complete("__seeds").unwrap();
        assert!(
            acks.append(ack(&fence, "__seeds", 1, "late", &[3], 1))
                .is_err()
        );
    }
    #[test]
    fn immutable_payload_transfers_storage_but_fresh_read_rechecks_integrity() {
        let mut bytes = Vec::with_capacity(256);
        bytes.extend_from_slice(&(-0.0f64).to_bits().to_be_bytes());
        let pointer = bytes.as_ptr();
        let payload = ResultPayload::new(bytes).unwrap();
        assert_eq!(payload.as_slice().as_ptr(), pointer);
        assert_ne!(
            payload.digest(),
            result_payload_digest(&0.0f64.to_bits().to_be_bytes())
        );
        let digest = payload.digest().to_owned();
        let mut batch = ResultBatch {
            key: result_batch_key("attempt", "set", 0),
            attempt: "attempt".into(),
            result_set: "set".into(),
            ordinal: 0,
            digest: digest.clone(),
            row_count: 1,
            payload_binding: "synthetic".into(),
            placement: pse_model::generated::enums::CanonicalPayloadPlacement::Sealed,
            inline_key: None,
            file_instance: Some(uuid::Uuid::new_v4().to_string()),
            payload_index: 0,
            payload_bytes: payload.byte_len(),
            schema: String::new(),
            encoding: crate::canonical_files::OPAQUE_BYTES.into(),
        };
        batch.payload_binding = payload_binding(&batch);
        validate_result_batch(&batch).unwrap();
        batch.payload_index = 1;
        assert!(validate_result_batch(&batch).is_err());
        assert!(matches!(
            ResultPayload::new(vec![0; RESULT_BATCH_BYTES + 1]),
            Err(CanonicalError::PayloadLimit)
        ));
    }
    #[test]
    fn writer_refusal_retains_exact_identity_and_rejects_malformed_tags() {
        let mut row = Object::new();
        row.insert("pse_writer", Value::String("study-cancelled".into()));
        row.insert("run", Value::String("run".into()));
        row.insert("attempt", Value::String("attempt".into()));
        row.insert("generation", canonical_codec::encode_uint(7).unwrap());
        row.insert("operation", Value::String("append-17".into()));
        assert!(
            matches!(writer_response(row.clone()), Err(CanonicalError::StudyCancellation { run, attempt, generation: 7, operation })
            if run == "run" && attempt == "attempt" && operation == "append-17")
        );
        row.insert("generation", Value::Number(surrealdb_types::Number::Int(7)));
        assert!(matches!(
            writer_response(row.clone()),
            Err(CanonicalError::Codec(_))
        ));
        row.insert("generation", canonical_codec::encode_uint(7).unwrap());
        row.insert("extra", Value::Bool(true));
        assert!(matches!(
            writer_response(row.clone()),
            Err(CanonicalError::Codec(_))
        ));
        row.remove("extra");
        row.insert("pse_writer", Value::String("unknown".into()));
        assert!(matches!(
            writer_response(row),
            Err(CanonicalError::Codec(_))
        ));
        let ordinary = Object::new();
        assert_eq!(writer_response(ordinary.clone()).unwrap(), ordinary);
    }
    #[test]
    fn index_metadata_accounts_strings_optional_values_and_combined_arrays() {
        let mut row = Object::new();
        row.insert("key", Value::String("a".into()));
        assert_eq!(
            result_metadata_extent(&Value::Object(row.clone())).unwrap(),
            7
        );
        assert_eq!(wire::RESULT_INDEX_RECORDS, wire::RESULT_INDEX_BYTES / 7);
        row.insert("partition", Value::String("x".repeat(65536)));
        let mut envelope = Object::new();
        envelope.insert(
            "block",
            Value::Array(vec![Value::Object(row.clone())].into()),
        );
        envelope.insert("outputs", Value::Array(vec![Value::Object(row)].into()));
        assert!(
            result_metadata_extent(&Value::Object(envelope)).unwrap() > EXECUTION_METADATA_BYTES
        );
    }
}

#[cfg(test)]
mod review_attempt_metadata_tests {
    use super::*;
    #[test]
    fn project_result_origin_projection_excludes_scientific_payloads_and_preserves_history_guards()
    {
        let sql = crate::canonical_history::run_query(RESULT_ORIGIN_SQL, "run");
        for guard in [
            "fn::pse_history_v1::run($pse_rpc_expires_at,$run)",
            "fn::pse_history_v1::check_identity($pse_rpc_expires_at,'attempt',$attempt)",
            "fn::pse_history_v1::run($pse_rpc_expires_at,$a.run)",
        ] {
            assert!(sql.contains(guard));
        }
        assert!(sql.contains("SELECT key,run,generation,phase,outcome,closed_manifest FROM ONLY"));
        assert!(sql.contains("SELECT key,revision FROM ONLY"));
        for forbidden in [
            "SELECT *",
            "completion",
            "request",
            "source_selection",
            "attestation",
        ] {
            assert!(
                !sql.contains(forbidden),
                "descriptive result projection includes {forbidden}"
            );
        }
        let mut row = Object::new();
        for (name, value) in [
            ("key", "attempt"),
            ("run", "run"),
            ("source_revision", "revision"),
            ("phase", "terminal"),
        ] {
            row.insert(name, canonical_codec::encode_string(value.into()).unwrap());
        }
        row.insert("generation", canonical_codec::encode_uint(2).unwrap());
        assert_eq!(decode_result_origin(row.clone()).unwrap().generation, 2);
        row.insert(
            "completion",
            canonical_codec::encode_bytes(vec![42].into()).unwrap(),
        );
        assert!(
            decode_result_origin(row).is_err(),
            "payload accidentally added to this projection must fail the typed decoder"
        );
    }
    #[test]
    fn review_attempt_metadata_preserves_uint_fences_and_absent_result() {
        let mut row = Object::default();
        row.insert(
            "key",
            canonical_codec::encode_string("attempt:old".into()).unwrap(),
        );
        row.insert(
            "run",
            canonical_codec::encode_string("run:exact".into()).unwrap(),
        );
        row.insert(
            "generation",
            canonical_codec::encode_uint(u64::MAX).unwrap(),
        );
        row.insert("terminal", canonical_codec::encode_boolean(true).unwrap());
        row.insert("close_generation", canonical_codec::encode_uint(0).unwrap());
        row.insert("closed_manifest", Value::None);
        let actual = decode_attempt_metadata(row.clone()).unwrap();
        assert_eq!(actual.generation, u64::MAX);
        assert_eq!(actual.close_generation, Some(0));
        assert!(actual.closed_manifest.is_none());
        row.remove("generation");
        assert!(decode_attempt_metadata(row).is_err());
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "exact current source receipt projection assertions"
)]
mod research_projection_tests {
    use super::*;
    fn request() -> RunRequest {
        let physical = pse_ids::ContentHash::from_bytes([1; 32]);
        let identity = pse_ids::ContentHash::from_bytes([2; 32]);
        RunRequest{purpose:pse_model::generated::enums::CanonicalRunPurpose::Execution,admission_study:None,admission_point:None,key:"run".into(),revision:crate::canonical::Revision{key:"published".into(),problem:"problem".into(),sequence:1,parent:None,operation:"published".into(),request:vec![].into(),interpretation:wire::INTERPRETATION.into()},sources:vec![],request:vec![],attestation:vec![],source_selection:serde_json::to_vec(&(3u8,identity,physical,serde_json::Value::Null,vec![serde_json::json!({"revision":identity,"physical_context":physical,"context":null,"research":{"issuing_store":"issuer","selection":"selected","dataset":"data-v1","revision":"published","physical_context":physical.to_string(),"publication":"published"}})])).unwrap()}
    }
    #[test]
    fn research_run_projection_retains_exact_original_research_and_refuses_context_differences() {
        let original = request();
        assert_eq!(
            projected_research_input(&original).unwrap(),
            (Some("selected".into()), Some("data-v1".into()))
        );
        let mut wrong = request();
        wrong.revision.key = "other-source".into();
        assert!(projected_research_input(&wrong).is_err());
        let mut wrong = request();
        let mut value: serde_json::Value = serde_json::from_slice(&wrong.source_selection).unwrap();
        value[4][0]["physical_context"] =
            serde_json::json!(pse_ids::ContentHash::from_bytes([3; 32]));
        wrong.source_selection = serde_json::to_vec(&value).unwrap();
        assert!(projected_research_input(&wrong).is_err());
        let mut old = request();
        old.source_selection = vec![2];
        assert_eq!(projected_research_input(&old).unwrap(), (None, None));
    }
    #[test]
    fn research_study_parent_projection_uses_original_exact_physical_source_and_research() {
        let mut request = request();
        let physical = pse_ids::ContentHash::from_bytes([4; 32]);
        let mut physical_revision = request.revision.clone();
        physical_revision.key = "physical".into();
        request.sources.push(physical_revision);
        request.source_selection=serde_json::to_vec(&(3u8,"published",serde_json::json!({"revision":"physical","identity":physical}),serde_json::Value::Null,serde_json::json!({"issuing_store":"issuer","selection":"selected","dataset":"data-v1","revision":"published","physical_context":physical.to_string(),"publication":"published"}))).unwrap();
        assert_eq!(
            projected_research_input(&request).unwrap(),
            (Some("selected".into()), Some("data-v1".into()))
        );
        request.sources.clear();
        assert!(projected_research_input(&request).is_err());
    }

    #[test]
    fn research_projection_deduplicates_repeated_inputs_and_preserves_mixed_executions() {
        let mut request = request();
        let mut value: serde_json::Value =
            serde_json::from_slice(&request.source_selection).unwrap();
        let repeated = value[4][0].clone();
        value[4].as_array_mut().unwrap().push(repeated);
        request.source_selection = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            projected_research_input(&request).unwrap(),
            (Some("selected".into()), Some("data-v1".into()))
        );
        value[4][1]["research"] = serde_json::Value::Null;
        request.source_selection = serde_json::to_vec(&value).unwrap();
        assert_eq!(projected_research_input(&request).unwrap(), (None, None));
        value[4][1] = value[4][0].clone();
        value[4][1]["research"]["selection"] = serde_json::json!("different-selection");
        request.source_selection = serde_json::to_vec(&value).unwrap();
        assert_eq!(projected_research_input(&request).unwrap(), (None, None));
    }
}
