// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical schema restart and old-epoch transaction controls.
#![allow(
    clippy::print_stderr,
    reason = "explicit original service identities support independent lifecycle readback"
)]

use crate::native_service::{NativeCanonicalService, NativeServiceError};
use std::time::{Duration, Instant};

fn current_fixture() -> Result<NativeCanonicalService, NativeServiceError> {
    let spec = super::owned_test_service_spec();
    let service = NativeCanonicalService::new(spec)?;
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
async fn native_service_final_apply_lost_ack_reads_original_commit_after_reopen() {
    let mut service = current_fixture().unwrap();
    let state = service.state().to_path_buf();
    let work_end = Instant::now() + Duration::from_secs(240);
    let settlement_end = work_end + Duration::from_secs(60);
    let (recovered, original_epoch) = service
        .run_until(work_end, settlement_end, |service| {
            Box::pin(async move {
                service.launch(work_end).await?;
                let old = service.ready()?;
                let original_epoch = old.fence().epoch().to_owned();
                let old = old.store().clone();
                let maintenance = service.maintenance().await?;
                let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                assert!(maintenance.reconcile_receipt(&request).await?.is_none());
                let pending = maintenance.begin(request.clone()).await?;
                let closed = pending
                    .quiesce(service.stop_ingress_and_join(work_end), &[old])
                    .await?;
                let before = maintenance.inspect().await?;
                assert_eq!(
                    before.pending_operation.as_deref(),
                    Some(request.operation())
                );
                assert_eq!(before.pending_phase.as_deref(), Some("schema-applying"));
                assert!(maintenance.reconcile_receipt(&request).await?.is_none());

                let mut committed = None;
                let recovered = closed
                    .apply_receipt_with_lost_ack_for_test(&mut committed)
                    .await?;
                assert_eq!(committed.as_ref(), Some(&recovered));
                assert_eq!(recovered.request(), &request);
                assert_eq!(
                    recovered.version(),
                    pse_schema::catalog::substrate::SCHEMA_VERSION
                );
                assert!(maintenance.inspect().await?.pending_operation.is_none());
                Ok((recovered, original_epoch))
            })
        })
        .await
        .unwrap();

    let mut service = service.try_into_stopped().unwrap().reopen().unwrap();
    service
        .run_until(work_end, settlement_end, |service| {
            Box::pin(async move {
                service.launch(work_end).await?;
                assert_ne!(service.ready()?.fence().epoch(), original_epoch);
                let maintenance = service.maintenance().await?;
                assert_eq!(
                    maintenance.reconcile_receipt(recovered.request()).await?,
                    Some(recovered)
                );
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
    let mut service = current_fixture().unwrap();
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
