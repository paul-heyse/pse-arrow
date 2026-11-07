// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual bounded progression on isolated logical fixture databases.

use crate::{canonical::CanonicalError, native_fixture::canonical_fixture_store};

#[tokio::test]
async fn canonical_studies_progression_sparse_pages_continue_and_two_workers_share_claim_authority()
-> Result<(), CanonicalError> {
    let store = canonical_fixture_store()?;
    super::sparse_pages_and_competing_workers(&store).await;
    store.remove_isolated_fixture().await
}

#[tokio::test]
async fn canonical_studies_progression_cancellation_fences_native_preparation()
-> Result<(), CanonicalError> {
    let store = canonical_fixture_store()?;
    super::cancellation_fences_preparation(&store).await;
    store.remove_isolated_fixture().await
}

#[tokio::test]
async fn canonical_studies_progression_dependency_revision_and_issuer_are_exact()
-> Result<(), CanonicalError> {
    let store = canonical_fixture_store()?;
    super::dependency_revision_and_issuer(&store).await;
    store.remove_isolated_fixture().await
}

#[tokio::test]
async fn canonical_studies_progression_seed_refusal_retains_original_request()
-> Result<(), CanonicalError> {
    let store = canonical_fixture_store()?;
    super::seed_refusal_retains_original_request(&store).await;
    store.remove_isolated_fixture().await
}

#[tokio::test]
async fn canonical_studies_prepared_claim_lost_ack_and_failed_readback_keep_original_attempt()
-> Result<(), CanonicalError> {
    let store = canonical_fixture_store()?;
    super::prepared_claim_acknowledgement_is_original(&store).await;
    store.remove_isolated_fixture().await
}

#[tokio::test]
async fn canonical_studies_high_degree_scope_is_complete_and_stale_hint_cannot_bypass_dependencies()
-> Result<(), CanonicalError> {
    let store = canonical_fixture_store()?;
    super::high_degree_dependency_scope_is_complete(&store).await;
    store.remove_isolated_fixture().await
}
