// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated canonical databases on the explicitly selected supervised server.

/// Shared by all managed store borrowers, including queued result-read releases.
pub(crate) struct FixtureLifetime {
    store: crate::canonical::CanonicalStore,
    executor: &'static tokio::runtime::Runtime,
    pub(crate) removed: tokio::sync::Mutex<bool>,
}
impl FixtureLifetime {
    pub(crate) fn new(
        store: crate::canonical::CanonicalStore,
        executor: &'static tokio::runtime::Runtime,
    ) -> Self {
        Self {
            store,
            executor,
            removed: tokio::sync::Mutex::new(false),
        }
    }
}
impl Drop for FixtureLifetime {
    #[expect(
        clippy::panic,
        reason = "fallback teardown reports failure on the thread owning the final fixture borrower"
    )]
    fn drop(&mut self) {
        if *self.removed.get_mut() {
            return;
        }
        // Nextest may exit immediately after the test: cleanup must complete,
        // rather than merely queue a task. The detached store has no owner.
        // A final borrower may itself live in a detached task, so qualification
        // consumers must await explicit removal to surface failure in their body.
        let result = std::thread::scope(|scope| {
            scope
                .spawn(|| self.executor.block_on(self.store.remove_isolated_fixture()))
                .join()
                .unwrap_or_else(|_| {
                    Err(crate::canonical::CanonicalError::Configuration(
                        "fixture cleanup executor panicked".into(),
                    ))
                })
        });
        if let Err(error) = result {
            if std::thread::panicking() {
                use std::io::Write;
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "isolated canonical fixture cleanup failed: {error}"
                );
            } else {
                panic!("isolated canonical fixture cleanup failed: {error}");
            }
        }
    }
}

/// Explicit isolated canonical fixture on the recipe-selected supervised server.
/// The retained fixture executor keeps the remote connection alive even when a
/// synchronous test helper is called from a different asynchronous executor.
/// Await `remove_isolated_fixture` after readers and workers finish when cleanup
/// failure must reach the caller; last-borrower Drop is a fallback.
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
    let store = std::thread::scope(|scope| {
        scope
            .spawn(|| executor.block_on(crate::canonical::CanonicalStore::connect(&options)))
            .join()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "canonical fixture executor panicked".into(),
                )
            })?
    })?;
    let store = store.own_fixture(executor);
    // Ownership exists before initialization can fail, but it is attached and
    // released outside the fixture executor. Failed schema creation also cleans up.
    std::thread::scope(|scope| {
        scope
            .spawn(|| executor.block_on(store.create()))
            .join()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "canonical fixture initialization executor panicked".into(),
                )
            })?
    })?;
    Ok(store)
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "fixture lifetime assertions require exact namespace responses and fail on unexpected results"
    )]
    use super::*;
    use crate::canonical::{CanonicalOptions, CanonicalStore, bounded_query};
    use surrealdb::types::Value;

    fn options() -> CanonicalOptions {
        let state = std::env::var_os("PSE_SURREAL_STATE").unwrap();
        CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap()
    }
    async fn database_exists(observer: &CanonicalStore, database: &str) -> bool {
        // Namespace inspection does not recreate the selected fixture database.
        let mut response = bounded_query(observer.db.query("INFO FOR NS;"))
            .await
            .unwrap();
        let Value::Object(info) = response.take::<Value>(0).unwrap() else {
            panic!("namespace inventory object required");
        };
        let Some(Value::Object(databases)) = info.get("databases") else {
            panic!("namespace database inventory required");
        };
        databases.contains_key(database)
    }

    #[tokio::test]
    async fn fixture_lifetime_waits_for_last_store_borrower() {
        let store = canonical_fixture_store().unwrap();
        assert!(store.owns_fixture());
        let database = store.database().to_owned();
        let observer = CanonicalStore::connect(&options()).await.unwrap();
        let borrower = store.clone();
        drop(store);
        assert!(database_exists(&observer, &database).await);
        borrower.open().await.unwrap();
        drop(borrower);
        assert!(!database_exists(&observer, &database).await);
    }

    #[tokio::test]
    async fn fixture_explicit_removal_disarms_last_owner_cleanup() {
        let store = canonical_fixture_store().unwrap();
        let database = store.database().to_owned();
        let observer = CanonicalStore::connect(&options()).await.unwrap();
        let borrower = store.clone();
        store.remove_isolated_fixture().await.unwrap();
        assert!(!database_exists(&observer, &database).await);
        let mut external = options();
        external.database = database.clone();
        let replacement = CanonicalStore::connect(&external).await.unwrap();
        replacement.create().await.unwrap();
        // A repeated explicit cleanup and the final managed drop cannot remove
        // a newly created, independently owned database under the same name.
        borrower.remove_isolated_fixture().await.unwrap();
        drop(store);
        drop(borrower);
        assert!(database_exists(&observer, &database).await);
        replacement.open().await.unwrap();
        replacement.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn fixture_connections_preserve_external_and_deployment_databases() {
        let mut external = options();
        let observer = CanonicalStore::connect(&external).await.unwrap();
        assert!(!observer.owns_fixture());
        let deployment_database = external.database.clone();
        let deployment_existed = database_exists(&observer, &deployment_database).await;
        let deployment_borrower = observer.clone();
        assert!(deployment_borrower.remove_isolated_fixture().await.is_err());
        drop(deployment_borrower);
        assert_eq!(
            database_exists(&observer, &deployment_database).await,
            deployment_existed
        );

        external.database = format!("canonical_test_external_{}", uuid::Uuid::new_v4().simple());
        let unmanaged = CanonicalStore::connect(&external).await.unwrap();
        assert!(!unmanaged.owns_fixture());
        unmanaged.create().await.unwrap();
        drop(unmanaged);
        assert!(database_exists(&observer, &external.database).await);
        let cleanup = CanonicalStore::connect(&external).await.unwrap();
        cleanup.remove_isolated_fixture().await.unwrap();
    }
}
