// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! C05 controls over an explicitly owned native service. Opaque transport bytes
//! establish only partial availability; scientific acceptance stays with runtime tests.
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreachable,
    reason = "private native fixture owns partial transport observations and exact failure assertions"
)]
use super::*;
use crate::native_service::{NativeCanonicalService, NativeServiceSpec};
use futures_util::{FutureExt, future::BoxFuture};
use std::{path::PathBuf, time::Instant};

fn service() -> NativeCanonicalService {
    let environment = std::env::vars_os().collect();
    let helper = PathBuf::from(
        std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
            .expect("explicit already-built native-service helper"),
    );
    assert!(helper.is_absolute() && helper.is_file());
    NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment).unwrap(),
        helper,
        module_source: if let Some(record) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT") {
            crate::native_service::NativeModuleSource::Prepared(pse_host::study_module::select(&PathBuf::from(record),pse_host::study_module::Variant::Production).unwrap())
        } else {
            crate::native_service::NativeModuleSource::RecordedState(PathBuf::from(std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE").expect("explicit private module receipt selector")))
        },
        state_parent: std::env::temp_dir(),
    })
    .unwrap()
}

fn claim<'a>(store: &'a CanonicalStore, key: &'a str) -> BoxFuture<'a, AttemptFence> {
    async move {
    let revision = store.edit(key, None, &format!("source:{key}"), &[]).await.unwrap();
    store.begin_run(&RunRequest {
        key: key.into(),
        purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
        admission_study: None,
        admission_point: None,
        revision,
        sources: vec![],
        request: vec![1],
        source_selection: vec![2],
        attestation: vec![3],
    }).await.unwrap();
    store.claim_run(key, &format!("claim:{key}"), "original-worker", Duration::from_secs(60))
        .await.unwrap()
    }.boxed()
}

fn prepared(fence: &AttemptFence, bytes: &[u8]) -> ResultBlockAppend {
    ResultBlockAppend::prepare(
        fence, "original-payload", "values", 0,
        ResultPayload::new(bytes.to_vec()).unwrap(), 1, None, &[], None,
    ).unwrap()
}

fn descriptor(item: &ResultBlockAppend) -> ResultBatch {
    let Some(Value::Object(batch)) = item.item.get("batch") else {
        unreachable!("prepared append owns its exact descriptor")
    };
    wire::decode_canonical_result_batches(batch.clone()).unwrap()
}

fn partial<'a>(store: &'a CanonicalStore, fence: &'a AttemptFence) -> BoxFuture<'a, CanonicalAttempt> {
    async move {
    let closed = store.close_result_ingestion(fence, "original-close").await.unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: the fixture owns only the explicit opaque partial observations.
    unsafe {
        store.seal_attempt(&manifest, "original-seal", TerminalClass::Partial,
            b"partial transport", SeedEligibility::Unavailable).await
    }.unwrap()
    }.boxed()
}

#[tokio::test]
async fn native_sealed_payload_and_lost_append_and_terminal_ack_keep_original_identity() {
    for extent in [
        pse_schema::catalog::substrate::RESULT_INLINE_BYTES,
        pse_schema::catalog::substrate::RESULT_INLINE_BYTES + 1,
    ] {
        let mut service = service();
        service.run(Instant::now() + Duration::from_secs(240), |service| Box::pin(async move {
            service.launch(Instant::now() + Duration::from_secs(90)).await?;
            let store = service.ready()?.store();
            let fence = claim(store, "publication-loss").await;
            let bytes = vec![7; extent];
            let mut item = prepared(&fence, &bytes);
            store.prepare_result_payload(&fence, &mut item).await.unwrap();
            let original_descriptor = descriptor(&item);
            let original_request = item.acknowledgment.request.clone();
            let sealed = extent > pse_schema::catalog::substrate::RESULT_INLINE_BYTES;
            if let Some(file) = &item.payload.file {
                crate::canonical_files::verify_payload_file(
                    &store.files.path(&file.descriptor().instance).unwrap(), file.descriptor(),
                ).unwrap();
            }
            assert_eq!(crate::testing::result_payload_counts(store, &fence).await.unwrap(),
                (0, u64::from(sealed)));
            assert!(!store.canonical_attempt(fence.attempt()).await.unwrap().unwrap().terminal());
            assert!(store.read_results(fence.run(), fence.attempt()).await.is_err(),
                "completed file construction grants neither terminal nor reader authority");
            drop(item);

            // The actual append commits; both its response and its recovery read
            // fail locally. Retrying only publication must reuse the sealed file.
            let error = EXECUTION_ACK_FAULT.scope(("execution-append", ExecutionAckFault::Missing),
                EXECUTION_ACK_READBACK_FAILURE.scope(true, store.append_result_batch(
                    &fence, "original-payload", "values", 0,
                    ResultPayload::new(bytes.clone()).unwrap(), 1,
                ).boxed())).await.unwrap_err();
            assert!(error.effect_is_unknown());
            assert!(matches!(error, CanonicalError::SettlementUnresolved {
                owner: "execution-append", operation, request, readback: Some(_), ..
            } if operation == "original-payload" && request == original_request));
            let ack = store.append_result_batch(&fence, "original-payload", "values", 0,
                ResultPayload::new(bytes.clone()).unwrap(), 1).await.unwrap();
            assert_eq!(ack.key, original_descriptor.key);
            assert_eq!(ack.request, original_request);
            assert_eq!(ack.receipt.payload_binding, original_descriptor.payload_binding);
            let counts = (u64::from(!sealed), u64::from(sealed));
            assert_eq!(crate::testing::result_payload_counts(store, &fence).await.unwrap(), counts);
            let closed = store.close_result_ingestion(&fence, "original-close").await.unwrap();
            let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
            // SAFETY: these exact opaque bytes establish partial availability only.
            let error = EXECUTION_ACK_FAULT.scope(("execution-seal", ExecutionAckFault::Missing),
                EXECUTION_ACK_READBACK_FAILURE.scope(true, unsafe {
                    store.seal_attempt(&manifest, "original-seal", TerminalClass::Partial,
                        b"partial transport", SeedEligibility::Unavailable).boxed()
                })).await.unwrap_err();
            assert!(error.effect_is_unknown());
            let committed = store.canonical_attempt(fence.attempt()).await.unwrap().unwrap();
            assert!(committed.terminal(), "the lost ACK does not undo actual terminal publication");
            let replay = partial(store, &fence).await;
            assert_eq!(replay, committed);
            assert_eq!(replay.run, fence.run());
            assert_eq!(replay.generation, fence.generation());
            assert_eq!(store.canonical_run(fence.run()).await.unwrap().unwrap().current_generation,
                fence.generation(), "ACK recovery never claims a second scientific attempt");
            assert_eq!(crate::testing::result_payload_counts(store, &fence).await.unwrap(), counts);
            let read = store.read_results(fence.run(), fence.attempt()).await.unwrap();
            let payload = store.result_payload(&read, &ack.result_set, 0).await.unwrap();
            assert_eq!(payload.batch.descriptor, original_descriptor);
            assert_eq!(payload.batch.payload.as_slice(), bytes);
            drop((payload, read));
            crate::testing::drain_result_releases(store).await.unwrap();
            Ok(())
        })).await.unwrap();
        service.dispose().unwrap();
        assert!(!service.state().exists(), "original physical service/state settlement completed");
    }
}

#[tokio::test]
async fn native_late_sealed_publication_cannot_borrow_replacement_fence() {
    let mut service = service();
    service.run(Instant::now() + Duration::from_secs(240), |service| Box::pin(async move {
        service.launch(Instant::now() + Duration::from_secs(90)).await?;
        let store = service.ready()?.store();
        let old = claim(store, "late-publication").await;
        let bytes = vec![9; pse_schema::catalog::substrate::RESULT_INLINE_BYTES + 1];
        let mut late = prepared(&old, &bytes);
        store.prepare_result_payload(&old, &mut late).await.unwrap();
        let sealed = late.payload.file.as_ref().unwrap().clone();
        let original_file = sealed.descriptor().clone();
        drop(late);
        // Deterministic expiry of only this fixture's exact original attempt.
        bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$attempt) SET expires_at=0;")
            .bind(("attempt", old.attempt().to_owned()))).await.unwrap();
        let recovered = store.recover_closed_attempt(old.run(), "recover-old").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&recovered).await.unwrap();
        // SAFETY: this expired transport attempt never published its constructed
        // file and is recorded only as failed, with no scientific success claim.
        unsafe {
            store.seal_attempt(&manifest, "failed-old", TerminalClass::Failed,
                b"expired before publication", SeedEligibility::Unavailable).await
        }.unwrap();
        let replacement = store.claim_run(old.run(), "replacement", "replacement-worker",
            Duration::from_secs(60)).await.unwrap();
        assert_ne!(replacement.attempt(), old.attempt());
        assert!(replacement.generation() > old.generation());
        // Carry the actual constructed file through the ordinary public append
        // preparation, without inventing a second file or physical identity.
        let late = ResultBlockAppend::prepare(&old, "original-payload", "values", 0,
            ResultPayload::from_sealed(sealed, 0).unwrap(), 1, None, &[], None).unwrap();
        let late_error = store.append_result_blocks(&old, vec![late]).await.unwrap_err();
        assert!(matches!(late_error, CanonicalError::Refused {
            code: crate::canonical::RefusalCode::ExecutionAttemptFenceRevoked, ..
        }));
        assert!(store.append_result_batch(&old, "original-payload", "values", 0,
            ResultPayload::new(bytes).unwrap(), 1).await.is_err());
        assert!(store.close_result_ingestion(&old, "late-close").await.is_err());
        let current = store.canonical_attempt(replacement.attempt()).await.unwrap().unwrap();
        assert!(current.ingestion_open() && !current.terminal());
        assert_eq!(crate::testing::result_payload_counts(store, &old).await.unwrap(), (0, 1));
        assert_eq!(crate::testing::result_payload_counts(store, &replacement).await.unwrap(), (0, 0));
        assert_eq!(store.canonical_run(old.run()).await.unwrap().unwrap().current_attempt.as_deref(),
            Some(replacement.attempt()));
        let mut response = bounded_query(store.db.query(
            "SELECT file_instance FROM canonical_result_batches WHERE attempt IN [$old,$replacement];")
            .bind(("old", old.attempt().to_owned()))
            .bind(("replacement", replacement.attempt().to_owned()))).await.unwrap();
        assert!(response.take::<Vec<Object>>(0).unwrap().is_empty(),
            "the original sealed file {} never becomes accepted late output", original_file.instance);
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
    assert!(!service.state().exists());
}

#[tokio::test]
async fn native_abandoned_reader_and_publisher_close_keep_independent_output_custody() {
    let mut service = service();
    service.run(Instant::now() + Duration::from_secs(240), |service| Box::pin(async move {
        service.launch(Instant::now() + Duration::from_secs(90)).await?;
        let publisher = service.ready()?.store().clone();
        let reader_store = service.connect().await?.store().clone();
        let fence = claim(&publisher, "output-custody").await;
        let bytes = vec![11; pse_schema::catalog::substrate::RESULT_INLINE_BYTES + 1];
        let ack = publisher.append_result_batch(&fence, "original-payload", "values", 0,
            ResultPayload::new(bytes.clone()).unwrap(), 1).await.unwrap();
        partial(&publisher, &fence).await;
        let abandoned = publisher.read_results(fence.run(), fence.attempt()).await.unwrap();
        let reader = reader_store.read_results(fence.run(), fence.attempt()).await.unwrap();
        drop(abandoned);
        publisher.close();
        crate::testing::drain_result_releases(&publisher).await.unwrap();
        assert!(publisher.canonical_run(fence.run()).await.is_err());
        assert!(reader_store.forget_run_results(fence.run()).await.is_err(),
            "abandoning one reader never releases another independent protected selection");
        let payload = reader_store.result_payload(&reader, &ack.result_set, 0).await.unwrap();
        let instance = payload.batch.file_instance.clone().unwrap();
        let path = reader_store.files.path(&instance).unwrap();
        assert!(path.is_file());
        let retained = payload.clone();
        drop((payload, reader));
        assert!(reader_store.forget_run_results(fence.run()).await.is_err(),
            "escaped delivery retains its original read custody after reader close");
        assert!(reader_store.disconnect().await.is_err(),
            "physical disconnect cannot substitute for retained-output release");
        assert!(reader_store.collect_file(&instance).await.is_err(),
            "physical output is still protected by the escaped delivery");
        assert!(path.is_file());
        assert_eq!(retained.batch.payload.as_slice(), bytes);
        assert_eq!(retained.batch.attempt, fence.attempt());
        assert_eq!(crate::testing::result_payload_counts(&reader_store, &fence).await.unwrap(), (0, 1));
        drop(retained);
        crate::testing::drain_result_releases(&reader_store).await.unwrap();
        reader_store.forget_run_results(fence.run()).await.unwrap();
        let mut reclaimed = false;
        for _ in 0..32 {
            if reader_store.reclaim_result_page(fence.run()).await.unwrap().complete {
                reclaimed = true;
                break;
            }
        }
        assert!(reclaimed, "last retained output release permits bounded physical reclamation");
        reader_store.collect_file(&instance).await.unwrap();
        assert!(!path.exists(), "physical release follows the final retained consumer");
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
    assert!(!service.state().exists());
}
