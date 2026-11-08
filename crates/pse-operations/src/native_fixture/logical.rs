// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Disposable logical database borrowing and physical client settlement.

/// The original local fixture settlement allowance, never renewed by peers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SettlementDeadline {
    pub(crate) local: tokio::time::Instant,
}
impl SettlementDeadline {
    fn new(local: tokio::time::Instant) -> Self {
        Self { local }
    }
}

/// Collect every independent result. A failed or stalled sibling never cancels
/// the others; every child carries the original clock explicitly across spawn.
async fn join_settlement<F>(
    deadline: tokio::time::Instant,
    siblings: Vec<(String, F)>,
) -> Result<(), crate::canonical::CanonicalError>
where
    F: Future<Output = Result<(), crate::canonical::CanonicalError>> + Send + 'static,
{
    let mut tasks = Vec::new();
    for (name, future) in siblings {
        tasks.push(tokio::spawn(async move {
            (
                name,
                crate::canonical::transport::within_clock(deadline, future).await,
            )
        }));
    }
    let mut failures = Vec::new();
    for task in tasks {
        match task.await {
            Ok((name, Err(error))) => failures.push(format!("{name}: {error}")),
            Err(error) => failures.push(format!("fixture drain task: {error}")),
            Ok((_, Ok(()))) => {}
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(crate::canonical::CanonicalError::Configuration(
            failures.join("; "),
        ))
    }
}

/// Shared by logical store borrowers, including queued result-read releases.
pub(crate) struct FixtureLifetime {
    store: crate::canonical::CanonicalStore,
    executor: &'static tokio::runtime::Runtime,
    pub(crate) removed: tokio::sync::Mutex<bool>,
    pub(crate) drained: std::sync::Arc<std::sync::atomic::AtomicBool>,
    peers: std::sync::Mutex<Vec<crate::canonical::CanonicalStore>>,
    settlement: std::sync::Mutex<Option<SettlementDeadline>>,
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
            drained: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            peers: std::sync::Mutex::new(Vec::new()),
            settlement: std::sync::Mutex::new(None),
        }
    }
}
impl FixtureLifetime {
    #[cfg(test)]
    pub(crate) fn execute<F, T>(&self, future: F) -> Result<T, crate::canonical::CanonicalError>
    where
        F: Future<Output = Result<T, crate::canonical::CanonicalError>> + Send,
        T: Send,
    {
        std::thread::scope(|scope| {
            scope
                .spawn(|| self.executor.block_on(future))
                .join()
                .map_err(|_| {
                    crate::canonical::CanonicalError::Configuration(
                        "fixture peer executor panicked".into(),
                    )
                })?
        })
    }
    pub(crate) fn settlement_deadline(
        &self,
        offered: tokio::time::Instant,
        retain: bool,
    ) -> Result<SettlementDeadline, crate::canonical::CanonicalError> {
        let mut retained = self.settlement.lock().map_err(|_| {
            crate::canonical::CanonicalError::Configuration(
                "fixture settlement clock poisoned".into(),
            )
        })?;
        match *retained {
            Some(previous) if previous.local <= offered => Ok(previous),
            _ => {
                let deadline = SettlementDeadline::new(offered);
                if retain {
                    *retained = Some(deadline);
                }
                Ok(deadline)
            }
        }
    }
    pub(crate) async fn drain_connections(
        &self,
        deadline: SettlementDeadline,
    ) -> Result<(), crate::canonical::CanonicalError> {
        let peers = self
            .peers
            .lock()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "fixture peer owner poisoned".into(),
                )
            })
            .map(|peers| peers.clone());
        let mut stores = vec![self.store.clone()];
        let peer_error = match peers {
            Ok(peers) => {
                stores.extend(peers);
                None
            }
            Err(error) => Some(error),
        };
        let siblings = stores
            .into_iter()
            .enumerate()
            .map(|(index, store)| {
                (format!("fixture connection {index}"), async move {
                    store.drain_fixture_connection().await
                })
            })
            .collect();
        let drained = join_settlement(deadline.local, siblings).await;
        match (peer_error, drained) {
            (None, result) => result,
            (Some(error), Ok(())) => Err(error),
            (Some(error), Err(other)) => Err(crate::canonical::CanonicalError::Configuration(
                format!("{error}; {other}"),
            )),
        }
    }
    /// Join every retained physical client before the native command owner disposes state.
    pub(crate) async fn settle(
        &self,
        current: Option<&crate::canonical::CanonicalStore>,
        deadline: SettlementDeadline,
    ) -> Result<(), crate::canonical::CanonicalError> {
        let (current, peers) = tokio::join!(
            async {
                match current {
                    Some(store) => {
                        crate::canonical::transport::within_clock(
                            deadline.local,
                            store.drain_fixture_connection(),
                        )
                        .await
                    }
                    None => Ok(()),
                }
            },
            self.drain_connections(deadline),
        );
        let mut errors = Vec::new();
        if let Err(error) = current {
            errors.push(format!("current fixture: {error}"));
        }
        if let Err(error) = peers {
            errors.push(format!("retained fixture owners: {error}"));
        }
        if !errors.is_empty() {
            return Err(crate::canonical::CanonicalError::Configuration(
                errors.join("; "),
            ));
        }
        self.drained
            .store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn retain_peer(
        &self,
        peer: crate::canonical::CanonicalStore,
    ) -> Result<(), crate::canonical::CanonicalError> {
        self.peers
            .lock()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "fixture peer owner poisoned".into(),
                )
            })?
            .push(peer);
        Ok(())
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
        // A test outcome belongs to the runner, never to last-borrower Drop.
        // Join physical clients and local readers before reporting local drain.
        let result = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let offered = crate::canonical::transport::original_deadline(
                        crate::canonical::REQUEST_TIMEOUT,
                    );
                    let deadline = self.settlement_deadline(offered, true)?;
                    self.executor.block_on(self.settle(None, deadline))
                })
                .join()
                .unwrap_or_else(|_| {
                    Err(crate::canonical::CanonicalError::Configuration(
                        "fixture drain executor panicked".into(),
                    ))
                })
        });
        if let Err(error) = result {
            if std::thread::panicking() {
                use std::io::Write;
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "isolated canonical fixture drain failed: {error}"
                );
            } else {
                panic!("isolated canonical fixture drain failed: {error}");
            }
        }
    }
}

/// Explicit logical fixture borrowing the command owner's authenticated fresh state.
/// No service, worker, registry entry or process is implicitly created here.
pub fn canonical_fixture_store()
-> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    let selection = super::FixtureSelection::selected()?;
    let mut options = crate::canonical::CanonicalOptions::from_state(selection.state())?;
    options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
    canonical_fixture_with_options(&options, true)
}

/// Open an isolated logical database on the already owned selected native service.
/// `initialize=false` preserves tests whose subject is explicit schema installation.
/// Directory and disposable identity are admitted before any remote fixture effects.
pub fn canonical_fixture_with_options(
    options: &crate::canonical::CanonicalOptions,
    initialize: bool,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    use std::sync::OnceLock;
    let selection = super::FixtureSelection::selected()?;
    selection.validate_options(options)?;
    if std::env::var_os("PSE_TEST_EXECUTION_PROFILE").is_some() {
        return Err(crate::canonical::CanonicalError::Configuration(
            "ambient execution profiles are retired; construct an explicit native worker owner"
                .into(),
        ));
    }
    static EXECUTOR: OnceLock<Result<tokio::runtime::Runtime, std::io::Error>> = OnceLock::new();
    let executor = EXECUTOR
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
        })
        .as_ref()
        .map_err(|error| {
            crate::canonical::CanonicalError::Configuration(format!("fixture executor: {error}"))
        })?;
    let mut selected = crate::canonical::CanonicalOptions::from_state_for_database(
        options.state_path(),
        Some(&options.database),
    )?;
    selected.endpoint = options.endpoint.clone();
    let store = std::thread::scope(|scope| {
        scope
            .spawn(|| executor.block_on(crate::canonical::CanonicalStore::connect(&selected)))
            .join()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "canonical fixture executor panicked".into(),
                )
            })?
    })?
    .own_fixture(executor);
    if initialize {
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
    }
    Ok(store)
}

/// Open an independent physical client under the original logical fixture lifetime.
#[cfg(test)]
pub(crate) fn canonical_fixture_peer(
    owner: &crate::canonical::CanonicalStore,
    options: &crate::canonical::CanonicalOptions,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    let borrowed_owner = owner.independent_caller()?;
    let lifetime = borrowed_owner.fixture_lifetime.as_ref().ok_or_else(|| {
        crate::canonical::CanonicalError::Configuration(
            "fixture peer requires an owned logical fixture".into(),
        )
    })?;
    if owner.database() != options.database || owner.deployment_state() != options.state_path() {
        return Err(crate::canonical::CanonicalError::Configuration(
            "fixture peer identity differs from original logical owner".into(),
        ));
    }
    let peer = lifetime.execute(crate::canonical::CanonicalStore::connect(options))?;
    lifetime.execute(peer.open())?;
    lifetime.retain_peer(peer.clone())?;
    peer.borrow_fixture(&borrowed_owner)
}

#[cfg(test)]
mod setup_observation_unit {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "fixture settlement controls assert exact owner outcomes"
    )]
    #[tokio::test]
    async fn settlement_collects_sibling_errors_without_renewing_clock() {
        use crate::canonical::CanonicalError;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(40);
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let siblings = (0..3)
            .map(|index| {
                let completed = completed.clone();
                (format!("sibling {index}"), async move {
                    match index {
                        0 => Err(CanonicalError::Configuration("first refused".into())),
                        1 => std::future::pending::<Result<(), CanonicalError>>().await,
                        _ => {
                            completed.store(true, std::sync::atomic::Ordering::Release);
                            Ok(())
                        }
                    }
                })
            })
            .collect();
        let error = super::join_settlement(deadline, siblings)
            .await
            .unwrap_err();
        assert!(completed.load(std::sync::atomic::Ordering::Acquire));
        assert!(error.to_string().contains("first refused"));
        assert!(error.to_string().contains("sibling 1"));
    }

    #[tokio::test]
    async fn cancelled_join_keeps_independent_children_owned_until_original_expiry() {
        let finished = std::sync::Arc::new(tokio::sync::Notify::new());
        let completed = finished.clone();
        let (started, start) = tokio::sync::oneshot::channel();
        let (release, resume) = tokio::sync::oneshot::channel();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let joining = tokio::spawn(super::join_settlement(
            deadline,
            vec![("owned child".into(), async move {
                started.send(()).unwrap();
                resume.await.unwrap();
                completed.notify_one();
                Ok(())
            })],
        ));
        start.await.unwrap();
        joining.abort();
        let _ = joining.await;
        release.send(()).unwrap();
        tokio::time::timeout_at(deadline, finished.notified())
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod canonical_server_unit {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "fixture lifetime assertions require exact namespace responses and fail on unexpected results"
    )]
    use super::*;
    use crate::canonical::{CanonicalOptions, CanonicalStore};

    fn options() -> CanonicalOptions {
        let state = std::env::var_os("PSE_SURREAL_STATE").unwrap();
        CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap()
    }
    async fn database_exists(database: &str) -> bool {
        // Each probe checks the actual catalog on a fresh connection without
        // creating a missing database or reusing a cached selection.
        let mut selected = options();
        selected.database = database.to_owned();
        let observer = CanonicalStore::connect(&selected).await.unwrap();
        let exists = observer.db.select_existing().await.unwrap();
        observer.disconnect().await.unwrap();
        exists
    }

    #[tokio::test]
    async fn fixture_lifetime_waits_for_last_store_borrower() {
        let store = canonical_fixture_store().unwrap();
        assert!(store.owns_fixture());
        let database = store.database().to_owned();
        let borrower = store.clone();
        drop(store);
        assert!(database_exists(&database).await);
        borrower.open().await.unwrap();
        drop(borrower);
        assert!(database_exists(&database).await);
    }

    #[tokio::test]
    async fn fixture_explicit_drain_retains_database_for_runner_disposition() {
        let store = canonical_fixture_store().unwrap();
        let database = store.database().to_owned();
        let borrower = store.clone();
        store.remove_isolated_fixture().await.unwrap();
        assert!(database_exists(&database).await);
        // An outstanding borrower keeps its connection and owns final drain.
        borrower.open().await.unwrap();
        drop(store);
        drop(borrower);
        assert!(database_exists(&database).await);
    }

    #[tokio::test]
    async fn unowned_connections_preserve_selected_fixture_database() {
        // Register the protected database before opening its separate observer.
        let fixture = canonical_fixture_store().unwrap();
        let mut external = options();
        external.database = fixture.database().to_owned();
        let observer = CanonicalStore::connect(&external).await.unwrap();
        assert!(!observer.owns_fixture());
        assert!(database_exists(&external.database).await);
        let unowned_borrower = observer.clone();
        assert!(unowned_borrower.remove_isolated_fixture().await.is_err());
        drop(unowned_borrower);
        observer.drain_fixture_connection().await.unwrap();
        drop(observer);
        assert!(database_exists(&external.database).await);
        fixture.remove_isolated_fixture().await.unwrap();
    }
}
