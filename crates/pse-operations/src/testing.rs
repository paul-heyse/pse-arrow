// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated logical databases borrowing one explicitly owned native test service.

/// Lose the local cancellation ACK after an actual commit, optionally failing
/// its original receipt readback. The command itself still uses the real owner.
pub async fn lose_control_cancel_ack<F: Future>(fail_readback: bool, future: F) -> F::Output {
    crate::canonical_control::CONTROL_ACK_LOST
        .scope(
            true,
            crate::canonical_control::CONTROL_READBACK_FAILED.scope(fail_readback, future),
        )
        .await
}

/// Revoke one already issued fixture actor through the privileged native owner.
/// This external mutation challenges existing-token admission, not token parsing.
pub async fn disable_control_actor(
    store: &crate::canonical::CanonicalStore,
    key: &str,
) -> Result<(), crate::canonical::CanonicalError> {
    use crate::canonical_codec::{decode_boolean, decode_string, required};
    let mut response = crate::canonical::bounded_query(store.db.query(
        "UPDATE ONLY type::record('canonical_control_actors',$key) SET enabled=false RETURN AFTER;",
    ).bind(("key",key.to_owned()))).await?;
    let mut row = response
        .take::<Option<surrealdb_types::Object>>(0)?
        .ok_or(crate::canonical::CanonicalError::IncompleteResponse)?;
    if decode_string(required(&mut row, "key")?)? != key
        || decode_boolean(required(&mut row, "enabled")?)?
    {
        return Err(crate::canonical::CanonicalError::IncompleteResponse);
    }
    Ok(())
}

/// Install three fixed toy API routes to challenge the selected engine's actual
/// transaction boundary. This fixture supplies no arbitrary SQL or production grant.
pub async fn install_control_transaction_probe(
    store: &crate::canonical::CanonicalStore,
) -> Result<(), crate::canonical::CanonicalError> {
    store.ensure_writes()?;
    crate::canonical::bounded_query(store.db.query(
        r#"
        DEFINE TABLE pse_control_transaction_probe SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD marker ON pse_control_transaction_probe TYPE string;
        DEFINE API '/pse-fixture/control/success' FOR post PERMISSIONS FULL
            MIDDLEWARE api::res::body('json') THEN {
                CREATE pse_control_transaction_probe:success SET marker='committed-success';
                RETURN {status:200,body:{marker:'committed-success'}};
            };
        DEFINE API '/pse-fixture/control/refusal' FOR post PERMISSIONS FULL
            MIDDLEWARE api::res::body('json') THEN {
                CREATE pse_control_transaction_probe:refusal SET marker='committed-refusal';
                RETURN {status:409,body:{marker:'committed-refusal'}};
            };
        DEFINE API '/pse-fixture/control/failure' FOR post PERMISSIONS FULL
            MIDDLEWARE api::res::body('json') THEN {
                CREATE pse_control_transaction_probe:failure SET marker='uncommitted-failure';
                THROW 'private control after-write failure witness';
            };
    "#,
    ))
    .await?;
    Ok(())
}

/// Independent privileged readback of the fixed API probe, including after a
/// returned error. An empty table alone cannot establish successful rollback.
pub async fn control_transaction_probe_markers(
    store: &crate::canonical::CanonicalStore,
) -> Result<Vec<String>, crate::canonical::CanonicalError> {
    let mut response = crate::canonical::bounded_query(
        store
            .db
            .query("SELECT VALUE marker FROM pse_control_transaction_probe ORDER BY marker;"),
    )
    .await?;
    Ok(response.take(0)?)
}

/// Actual stored payload owners for this exact attempt, including reservations.
/// This fixture observation never creates a payload or grants read permission.
pub async fn result_payload_counts(
    store: &crate::canonical::CanonicalStore,
    fence: &crate::canonical_execution::AttemptFence,
) -> Result<(u64, u64), crate::canonical::CanonicalError> {
    use crate::canonical_codec::{decode_uint, required};
    let mut response = crate::canonical::bounded_query(store.db.query(
        "RETURN {inline:<decimal>array::len((SELECT key FROM canonical_inline_payloads WHERE attempt=$attempt)),sealed:<decimal>array::len((SELECT key FROM canonical_file_instances WHERE owner_kind='result' AND selection=$attempt))};"
    ).bind(("attempt",fence.attempt().to_owned()))).await?;
    let mut row = response
        .take::<Option<surrealdb_types::Object>>(0)?
        .ok_or(crate::canonical::CanonicalError::IncompleteResponse)?;
    Ok((
        decode_uint(required(&mut row, "inline")?)?,
        decode_uint(required(&mut row, "sealed")?)?,
    ))
}

/// Join already registered result-read cleanup without closing the fixture or
/// treating a facade drop as physical driver drain. Future independent reads remain valid.
pub async fn drain_result_releases(
    store: &crate::canonical::CanonicalStore,
) -> Result<(), crate::canonical::CanonicalError> {
    store.result_read_drain.drain().await
}

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
    pub(crate) fn cleanup_executor(&self) -> tokio::runtime::Handle {
        self.executor.handle().clone()
    }

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
    let selection = crate::native_fixture::FixtureSelection::selected()?;
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
    let selection = crate::native_fixture::FixtureSelection::selected()?;
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
pub fn canonical_fixture_peer(
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

/// Observe the exact current source head in an isolated fixture without changing it.
pub async fn source_problem_head(
    store: &crate::canonical::CanonicalStore,
    problem: &str,
) -> Result<
    Option<pse_model::generated::runtime::canonical_problems::Row>,
    crate::canonical::CanonicalError,
> {
    let mut response = crate::canonical::bounded_query(
        store
            .db
            .query("SELECT * FROM ONLY type::record('canonical_problems', $problem);")
            .bind(("problem", problem.to_owned())),
    )
    .await?;
    response
        .take::<Option<surrealdb_types::Object>>(0)?
        .map(crate::generated::surreal::decode_canonical_problems)
        .transpose()
        .map_err(Into::into)
}

/// Inspect one exact unsealed batch in a registered fixture without admitting its prefix.
pub async fn pending_result_batch_rows(
    store: &crate::canonical::CanonicalStore,
    fence: &crate::canonical_execution::AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<Option<u64>, crate::canonical::CanonicalError> {
    use crate::{
        canonical::bounded_query,
        canonical_codec as codec,
        canonical_execution::{result_batch_key, result_set_key},
    };
    let set = result_set_key(fence.attempt(), output);
    let key = result_batch_key(fence.attempt(), &set, ordinal);
    let mut response = bounded_query(store.db.query("RETURN SELECT * FROM ONLY type::record('canonical_result_batches',$key) WHERE attempt=$attempt AND result_set=$set;")
        .bind(("key",key)).bind(("attempt",fence.attempt().to_owned())).bind(("set",set))).await?;
    let Some(mut row) = response.take::<Option<surrealdb_types::Object>>(0)? else {
        return Ok(None);
    };
    Ok(Some(codec::decode_uint(codec::required(
        &mut row,
        "row_count",
    )?)?))
}

/// Inspect actual staged payload coordinates in an isolated schema-boundary control.
pub async fn pending_result_payload_coordinates(
    store: &crate::canonical::CanonicalStore,
    fence: &crate::canonical_execution::AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<
    Option<(crate::canonical_execution::PayloadLocator, String)>,
    crate::canonical::CanonicalError,
> {
    use crate::{
        canonical::bounded_query,
        canonical_execution::{result_batch_key, result_payload_locator, result_set_key},
        generated::surreal as wire,
    };
    let set = result_set_key(fence.attempt(), output);
    let key = result_batch_key(fence.attempt(), &set, ordinal);
    let mut response = bounded_query(store.db.query("RETURN SELECT * FROM ONLY type::record('canonical_result_batches',$key) WHERE attempt=$attempt AND result_set=$set;")
        .bind(("key",key)).bind(("attempt",fence.attempt().to_owned())).bind(("set",set))).await?;
    let Some(row) = response.take::<Option<surrealdb_types::Object>>(0)? else {
        return Ok(None);
    };
    let batch = wire::decode_canonical_result_batches(row)?;
    crate::canonical_execution::validate_result_batch(&batch)?;
    Ok(Some((result_payload_locator(&batch)?, batch.schema)))
}

/// Expire only an acknowledged current claim in a registered isolated fixture.
/// Closure does not end a lease; terminal and compacted attempts remain ineligible.
pub async fn expire_acknowledged_attempt(
    store: &crate::canonical::CanonicalStore,
    fence: &crate::canonical_execution::AttemptFence,
) -> Result<(), crate::canonical::CanonicalError> {
    use crate::canonical::{CanonicalError, bounded_query};
    if !store.database().starts_with("canonical_test_") || store.fixture_lifetime.is_none() {
        return Err(CanonicalError::Configuration(
            "claim expiry requires a registered isolated fixture".into(),
        ));
    }
    let generation = crate::canonical_codec::encode_uint(fence.generation())?;
    let script = crate::canonical_script::test_script::<Option<bool>>(
        "LET $run = SELECT * FROM ONLY type::record('canonical_runs',$run); LET $attempt = SELECT * FROM ONLY type::record('canonical_attempts',$attempt); IF $run = NONE OR $attempt = NONE OR $attempt.run != $run.key OR $run.current_attempt != $attempt.key OR $run.current_generation != $generation OR $attempt.generation != $generation OR $attempt.phase IN ['terminal','terminal_compacted'] { THROW 'PSE_REFUSAL:fixture_expiry_claim_changed'; }; UPDATE ONLY $attempt.id SET expires_at = time::micros() - 1;",
        "true",
    )?;
    let result = script.result();
    let mut response = bounded_query(
        store
            .db
            .script(script)
            .bind(("run", fence.run().to_owned()))
            .bind(("attempt", fence.attempt().to_owned()))
            .bind(("generation", generation)),
    )
    .await?;
    if result.take(&mut response)? != Some(true) {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(())
}

/// Corrupt one terminal occurrence only in its registered isolated fixture.
pub async fn invalidate_terminal_study_outcome(
    store: &crate::canonical::CanonicalStore,
    study: &str,
    occurrence: pse_model::study::OccurrenceKey,
) -> Result<(), crate::canonical::CanonicalError> {
    use crate::canonical::{CanonicalError, bounded_query};
    if !store.database().starts_with("canonical_test_") || store.fixture_lifetime.is_none() {
        return Err(CanonicalError::Configuration(
            "occurrence invalidation requires a registered isolated fixture".into(),
        ));
    }
    bounded_query(store.db.query("LET $study = SELECT * FROM ONLY type::record('canonical_studies',$study); IF $study = NONE OR !$study.terminal { THROW 'PSE_REFUSAL:fixture_study_is_not_terminal'; }; UPDATE ONLY type::record('canonical_study_points',$point) SET outcome=$outcome;")
        .bind(("study", study.to_owned()))
        .bind(("point", crate::canonical_studies::point_key(study, occurrence)))
        .bind(("outcome", surrealdb_types::Bytes::from(vec![0xff])))).await?;
    Ok(())
}

/// Remove one prepared source revision to exercise same-store registration refusal.
pub async fn invalidate_prepared_revision(
    store: &crate::canonical::CanonicalStore,
    revision: &crate::canonical::Revision,
) -> Result<(), crate::canonical::CanonicalError> {
    use crate::canonical::{CanonicalError, bounded_query};
    if !store.database().starts_with("canonical_test_") || store.fixture_lifetime.is_none() {
        return Err(CanonicalError::Configuration(
            "revision invalidation requires a registered isolated fixture".into(),
        ));
    }
    bounded_query(store.db.query("DELETE ONLY type::record('canonical_revisions',$key) WHERE problem=$problem AND sequence=$sequence;")
        .bind(("key",revision.key.clone())).bind(("problem",revision.problem.clone())).bind(("sequence",crate::canonical_codec::encode_uint(revision.sequence)?))).await?;
    Ok(())
}

/// Cancel the supplied operation after its first actual artifact append-group ACK.
/// The suspended transfer must be owned by a cancellation-aware caller.
#[cfg(any(test, feature = "test-support"))]
pub async fn cancel_after_artifact_append_ack<F: Future>(
    cancel: impl Fn() + Send + Sync + 'static,
    future: F,
) -> F::Output {
    crate::canonical_artifacts::CANCEL_AFTER_ARTIFACT_APPEND_ACK
        .scope(std::sync::Arc::new(cancel), future)
        .await
}

/// Suppress the real project's successful source-activation response so a composed
/// consumer exercises original-operation settlement. Available only in canonical fixtures.
#[cfg(any(test, feature = "test-support"))]
pub async fn lose_project_source_reply<F: Future>(future: F) -> F::Output {
    crate::canonical_projects::PROJECT_REPLY_LOSS
        .scope(true, future)
        .await
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

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "fixture lifetime assertions require exact namespace responses and fail on unexpected results"
    )]
    use super::*;
    use crate::canonical::{CanonicalOptions, CanonicalStore, bounded_query};
    use surrealdb_types::Value;

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
        assert!(database_exists(&observer, &database).await);
    }

    #[tokio::test]
    async fn fixture_explicit_drain_retains_database_for_runner_disposition() {
        let store = canonical_fixture_store().unwrap();
        let database = store.database().to_owned();
        let observer = CanonicalStore::connect(&options()).await.unwrap();
        let borrower = store.clone();
        store.remove_isolated_fixture().await.unwrap();
        assert!(database_exists(&observer, &database).await);
        // An outstanding borrower keeps its connection and owns final drain.
        borrower.open().await.unwrap();
        drop(store);
        drop(borrower);
        assert!(database_exists(&observer, &database).await);
    }

    #[tokio::test]
    async fn unowned_connections_preserve_selected_fixture_database() {
        // Register the protected database before opening its separate observer.
        let fixture = canonical_fixture_store().unwrap();
        let mut external = options();
        external.database = fixture.database().to_owned();
        let observer = CanonicalStore::connect(&external).await.unwrap();
        assert!(!observer.owns_fixture());
        assert!(database_exists(&fixture, &external.database).await);
        let unowned_borrower = observer.clone();
        assert!(unowned_borrower.remove_isolated_fixture().await.is_err());
        drop(unowned_borrower);
        observer.drain_fixture_connection().await.unwrap();
        drop(observer);
        assert!(database_exists(&fixture, &external.database).await);
        fixture.remove_isolated_fixture().await.unwrap();
    }
}
