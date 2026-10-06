// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated canonical databases on the explicitly selected supervised server.

/// Explicit isolated canonical fixture on the recipe-selected supervised server.
/// The retained fixture executor keeps the remote connection alive even when a
/// synchronous test helper is called from a different asynchronous executor.
pub fn canonical_fixture_store()
-> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    use std::sync::OnceLock;
    static EXECUTOR: OnceLock<Result<tokio::runtime::Runtime, std::io::Error>> = OnceLock::new();
    let executor = EXECUTOR
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
        })
        .as_ref()
        .map_err(|error| {
            crate::canonical::CanonicalError::Configuration(format!("fixture executor: {error}"))
        })?;
    let state = std::env::var_os("PSE_SURREAL_STATE").ok_or_else(|| {
        crate::canonical::CanonicalError::Configuration(
            "canonical fixture requires recipe-selected PSE_SURREAL_STATE".into(),
        )
    })?;
    let mut options = crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state))?;
    options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                executor.block_on(async {
                    let store = crate::canonical::CanonicalStore::connect(&options).await?;
                    store.create().await?;
                    Ok(store)
                })
            })
            .join()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "canonical fixture executor panicked".into(),
                )
            })?
    })
}

