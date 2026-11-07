// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical schema restart and old-epoch transaction controls.
#![allow(
    clippy::print_stderr,
    reason = "explicit original service identities support independent lifecycle readback"
)]

use crate::{
    canonical::{CanonicalOptions, CanonicalStore},
    native_service::{NativeCanonicalService, NativeServiceError},
};
use std::time::{Duration, Instant};

fn fixture_with_schema(pre_epoch: bool) -> Result<NativeCanonicalService, NativeServiceError> {
    let spec = super::owned_test_service_spec();
    let service = if pre_epoch {
        NativeCanonicalService::new_pre_epoch_fixture(spec)?
    } else {
        NativeCanonicalService::new(spec)?
    };
    eprintln!(
        "native fixture state={} original_service={}",
        service.state().display(),
        service.intent().unit()
    );
    Ok(service)
}
fn report_ingresses(service: &NativeCanonicalService) {
    for intent in service.ingress_intents() {
        eprintln!(
            "native fixture state={} original_ingress={}",
            service.state().display(),
            intent.unit()
        );
    }
}

#[tokio::test]
async fn native_service_pre_epoch_checkpoint_lost_ack_restarts_and_resumes_original_request() {
    let mut service = fixture_with_schema(true).unwrap();
    let state = service.state().to_path_buf();
    let recorded = service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                report_ingresses(service);
                let maintenance = service.maintenance().await?;
                let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                let pending = maintenance.begin(request.clone()).await?;
                let closed = pending
                    .quiesce(
                        service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)),
                        &[],
                    )
                    .await?;
                // The server commits the real checkpoint; only its acknowledgement is discarded.
                closed.checkpoint_v2_lost_ack_for_test().await?;
                let checkpoint = maintenance.inspect().await?;
                assert_eq!(
                    checkpoint.pending_operation.as_deref(),
                    Some(request.operation())
                );
                assert_eq!(
                    checkpoint.pending_phase.as_deref(),
                    Some("schema-v2-checkpoint")
                );
                let intermediate = serde_json::to_value(checkpoint.predecessor).unwrap();
                assert_eq!(intermediate["schema_version"], 2);
                assert_eq!(
                    intermediate["schema_digest"],
                    pse_schema::catalog::substrate::SCHEMA_V2_DIGEST
                );
                assert!(
                    CanonicalStore::connect_ready(&CanonicalOptions::from_state(service.state())?)
                        .await
                        .is_err()
                );
                assert!(maintenance.reconcile_receipt(&request).await?.is_none());
                assert!(
                    maintenance
                        .request_current(uuid::Uuid::new_v4())
                        .await
                        .is_err()
                );
                let mut foreign = serde_json::to_value(&request).unwrap();
                foreign["operation"] = uuid::Uuid::new_v4().to_string().into();
                let foreign = serde_json::from_value(foreign).unwrap();
                assert!(maintenance.begin(foreign).await.is_err());
                maintenance.disconnect().await?;
                Ok(serde_json::to_vec(&request).unwrap())
            })
        })
        .await
        .unwrap();
    let stopped = service.try_into_stopped().unwrap();
    let mut service = stopped.reopen().unwrap();
    eprintln!(
        "native checkpoint fixture state={} resumed_service={}",
        state.display(),
        service.intent().unit()
    );
    service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                report_ingresses(service);
                let request = serde_json::from_slice(&recorded).unwrap();
                let maintenance = service.maintenance().await?;
                assert_eq!(
                    maintenance.inspect().await?.pending_phase.as_deref(),
                    Some("schema-v2-checkpoint")
                );
                assert!(maintenance.reconcile_receipt(&request).await?.is_none());
                let pending = maintenance.begin(request.clone()).await?;
                let closed = pending
                    .quiesce(
                        service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)),
                        &[],
                    )
                    .await?;
                let ready = closed.apply().await?;
                service.retain_connection(ready.store().clone())?;
                assert_eq!(
                    ready.fence().version(),
                    pse_schema::catalog::substrate::SCHEMA_VERSION
                );
                let receipt = maintenance.reconcile_receipt(&request).await?.unwrap();
                assert_eq!(serde_json::to_vec(receipt.request()).unwrap(), recorded);
                let original = serde_json::to_value(receipt.request()).unwrap();
                let installed =
                    serde_json::to_value(maintenance.inspect().await?.predecessor).unwrap();
                for field in [
                    "issuing_store",
                    "payload_incarnation",
                    "study_module_sha256",
                    "interpretation",
                ] {
                    assert_eq!(installed[field], original["predecessor"][field], "{field}");
                }
                ready.store().canonical_run("native-fixture-probe").await?;
                ready
                    .store()
                    .drain_owned_connection_until(crate::transport::original_deadline(
                        Duration::from_secs(30),
                    ))
                    .await?;
                maintenance.disconnect().await?;
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_inflight_old_epoch_writer_cannot_commit_across_activation() {
    let mut service = fixture_with_schema(false).unwrap();
    service.run(Instant::now() + Duration::from_secs(240), |service| {
        Box::pin(async move {
            service.launch(Instant::now() + Duration::from_secs(90)).await?;
            let old = service.ready()?.clone();
            let before = service.stage_schema_fence_write(&old, Instant::now() + Duration::from_secs(120)).await?;
            before.commit().await?;
            assert!(before.persisted(&old).await?);
            let inflight = service.stage_schema_fence_write(&old, Instant::now() + Duration::from_secs(120)).await?;
            let maintenance = service.maintenance().await?;
            let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
            let pending = maintenance.begin(request).await?;
            let closed = pending.quiesce(service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)), &[old.store().clone()]).await?;
            let successor = closed.apply().await?;
            service.retain_connection(successor.store().clone())?;
            assert_ne!(successor.fence().epoch(), old.fence().epoch());
            let refused = inflight.commit().await.unwrap_err();
            assert!(matches!(refused, crate::canonical::CanonicalError::Driver(ref error)
                if matches!(error.query_details(), Some(surrealdb_types::QueryError::TransactionConflict))),
                "actual old transaction must fail at the server commit boundary: {refused:?}");
            assert!(!inflight.persisted(&successor).await?);
            assert!(before.persisted(&successor).await?);
            successor.store().drain_owned_connection_until(crate::transport::original_deadline(Duration::from_secs(30))).await?;
            maintenance.disconnect().await?;
            Ok(())
        })
    }).await.unwrap();
    service.dispose().unwrap();
}
