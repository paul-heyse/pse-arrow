// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! C12 only: actual Delta residency and consumed-value publication reuse.
#[path = "support/cache_journey.rs"]
mod cache_journey;

#[tokio::test]
async fn selected_cache_and_projected_reuse_match_fresh_native_results() {
    if let Ok(payload) = std::env::var("PSE_NATIVE_CACHE_MAINTENANCE") {
        cache_journey::maintenance_child(&payload).await;
        return;
    }
    for enabled in [false, true] {
        let receipt = if enabled {
            Box::pin(cache_journey::run_with_fence(true, 16, true)).await
        } else {
            Box::pin(cache_journey::run(false, 16)).await
        };
        println!("{receipt}");
    }
}
