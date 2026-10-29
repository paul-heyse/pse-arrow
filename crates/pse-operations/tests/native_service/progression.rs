// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual bounded progression in owned private service lifetimes.
use super::*;
use pse_operations::canonical_studies::progression_controls;

async fn progression_control(control: u8) {
    let mut service = fixture_with_schema(false).unwrap();
    service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let ready = service.ready()?;
                match control {
                    0 => {
                        progression_controls::sparse_pages_and_competing_workers(ready.store())
                            .await
                    }
                    1 => progression_controls::cancellation_fences_preparation(ready.store()).await,
                    2 => progression_controls::dependency_revision_and_issuer(ready.store()).await,
                    3 => {
                        progression_controls::seed_refusal_retains_original_request(ready.store())
                            .await
                    }
                    4 => {
                        progression_controls::prepared_claim_acknowledgement_is_original(
                            ready.store(),
                        )
                        .await
                    }
                    _ => {
                        progression_controls::high_degree_dependency_scope_is_complete(
                            ready.store(),
                        )
                        .await
                    }
                }
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
}

#[tokio::test]
async fn canonical_studies_progression_sparse_pages_continue_and_two_workers_share_claim_authority()
{
    progression_control(0).await;
}
#[tokio::test]
async fn canonical_studies_progression_cancellation_fences_native_preparation() {
    progression_control(1).await;
}
#[tokio::test]
async fn canonical_studies_progression_dependency_revision_and_issuer_are_exact() {
    progression_control(2).await;
}
#[tokio::test]
async fn canonical_studies_progression_seed_refusal_retains_original_request() {
    progression_control(3).await;
}
#[tokio::test]
async fn canonical_studies_prepared_claim_lost_ack_and_failed_readback_keep_original_attempt() {
    progression_control(4).await;
}
#[tokio::test]
async fn canonical_studies_high_degree_scope_is_complete_and_stale_hint_cannot_bypass_dependencies()
{
    progression_control(5).await;
}
