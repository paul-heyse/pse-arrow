// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated canonical databases on the explicitly selected supervised server.

/// Shared by all managed store borrowers, including queued result-read releases.
pub(crate) struct FixtureLifetime {
    store: crate::canonical::CanonicalStore,
    executor: &'static tokio::runtime::Runtime,
    pub(crate) removed: tokio::sync::Mutex<bool>,
    pub(crate) resource: String,
    peers: std::sync::Mutex<Vec<crate::canonical::CanonicalStore>>,
}
impl FixtureLifetime {
    pub(crate) fn new(
        store: crate::canonical::CanonicalStore,
        executor: &'static tokio::runtime::Runtime,
        resource: String,
    ) -> Self {
        Self {
            store,
            executor,
            removed: tokio::sync::Mutex::new(false),
            resource,
            peers: std::sync::Mutex::new(Vec::new()),
        }
    }
}
impl FixtureLifetime {
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
    pub(crate) async fn drain_connections(&self) -> Result<(), crate::canonical::CanonicalError> {
        self.store.disconnect().await?;
        let peers = self
            .peers
            .lock()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "fixture peer owner poisoned".into(),
                )
            })?
            .clone();
        for peer in peers {
            peer.disconnect().await?;
        }
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
        // Publish drain only after both physical clients and local readers finish.
        let result = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    self.executor.block_on(self.store.disconnect())?;
                    for peer in self
                        .peers
                        .get_mut()
                        .map_err(|_| {
                            crate::canonical::CanonicalError::Configuration(
                                "fixture peer owner poisoned".into(),
                            )
                        })?
                        .iter()
                    {
                        self.executor.block_on(peer.disconnect())?;
                    }
                    resource_bridge(
                        "drain",
                        &serde_json::json!({
                            "resource": self.resource,
                            "state": self.store.deployment_state(),
                            "database": self.store.database(),
                        }),
                    )
                    .map(|_| ())
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

pub(crate) fn resource_bridge(
    action: &str,
    input: &serde_json::Value,
) -> Result<serde_json::Value, crate::canonical::CanonicalError> {
    use std::io::Write;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or_else(|| {
            crate::canonical::CanonicalError::Configuration(
                "fixture checkout root unavailable".into(),
            )
        })?;
    let interpreter = std::env::var_os("PSE_TEST_RESOURCE_PYTHON")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join(".venv/bin/python"));
    let mut command = std::process::Command::new(interpreter);
    command
        .current_dir(root)
        .args(["-m", "scripts.test_resources", action]);
    if action == "status" {
        command.arg(
            input
                .get("resource")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    crate::canonical::CanonicalError::Configuration(
                        "fixture status requires exact resource".into(),
                    )
                })?,
        );
    }
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?;
    let mut stdin = child.stdin.take().ok_or_else(|| {
        crate::canonical::CanonicalError::Configuration("fixture registry input unavailable".into())
    })?;
    stdin
        .write_all(
            &serde_json::to_vec(input).map_err(|error| {
                crate::canonical::CanonicalError::Configuration(error.to_string())
            })?,
        )
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?;
    drop(stdin);
    let budget = if action == "drain" { 50 } else { 60 };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(budget);
    loop {
        if child
            .try_wait()
            .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?
            .is_some()
        {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(crate::canonical::CanonicalError::Configuration(format!(
                "fixture registry {action} exceeded its finite bridge budget"
            )));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?;
    if !output.status.success() {
        return Err(crate::canonical::CanonicalError::Configuration(format!(
            "fixture registry {action}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))
}

fn selected_fixture_identity()
-> Result<(String, std::path::PathBuf), crate::canonical::CanonicalError> {
    let args = std::env::args().collect::<Vec<_>>();
    let test = args
        .iter()
        .position(|arg| arg == "--exact")
        .and_then(|index| {
            index
                .checked_sub(1)
                .filter(|before| *before != 0)
                .and_then(|before| args.get(before))
                .filter(|value| !value.starts_with('-'))
                .or_else(|| args.get(index + 1).filter(|value| !value.starts_with('-')))
        })
        .cloned()
        .unwrap_or_default();
    let executable = std::env::current_exe()
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?;
    Ok((test, executable))
}

fn registered_resource(
    payload: &serde_json::Value,
) -> Result<String, crate::canonical::CanonicalError> {
    let result = resource_bridge("register", payload)?;
    result
        .get("resource")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            crate::canonical::CanonicalError::Configuration(
                "fixture registry omitted resource identity".into(),
            )
        })
}

/// Pin disposable controls before creating their directory. Only the runner's
/// exact selected test association can later authorize retained-data disposal.
pub fn register_fixture_controls(
    state: &std::path::Path,
    database: &str,
    directory: &std::path::Path,
) -> Result<String, crate::canonical::CanonicalError> {
    let (test, executable) = selected_fixture_identity()?;
    registered_resource(&serde_json::json!({
        "state": state, "database": database, "directory": directory,
        "kind": "controls", "test": test, "executable": executable, "units": [],
    }))
}

/// Record control drain after the caller has cooperatively drained its actual
/// native receiver. The registry independently verifies the associated context.
pub fn record_fixture_controls_drain(
    resource: &str,
    state: &std::path::Path,
    database: &str,
) -> Result<(), crate::canonical::CanonicalError> {
    resource_bridge(
        "drain",
        &serde_json::json!({
            "resource": resource, "state": state, "database": database,
        }),
    )
    .map(|_| ())
}

pub(crate) fn register_database(
    options: &crate::canonical::CanonicalOptions,
    execution_profile: Option<&str>,
) -> Result<String, crate::canonical::CanonicalError> {
    let (test, executable) = selected_fixture_identity()?;
    let mut payload = serde_json::json!({"state": options.state_path(), "database": options.database, "kind": "database", "test": test, "executable": executable, "units": []});
    if execution_profile.is_some()
        || options.native.execution.is_some()
        || options.primary_receiver.is_some()
    {
        if let Some(profile) = execution_profile {
            payload["execution_profile"] = profile.into();
        } else if let Ok(profile) = std::env::var("PSE_TEST_EXECUTION_PROFILE") {
            payload["execution_profile"] = profile.into();
        }
        if let Some(worker) = std::env::var_os("PSE_WORKER_BINARY") {
            payload["worker"] = std::path::PathBuf::from(worker)
                .to_string_lossy()
                .into_owned()
                .into();
        }
    }
    registered_resource(&payload)
}

/// Explicit isolated canonical fixture on the recipe-selected supervised server.
/// The retained fixture executor keeps the remote connection alive even when a
/// synchronous test helper is called from a different asynchronous executor.
/// Await `remove_isolated_fixture` to request local drain; final runner outcome
/// and reference eligibility alone authorize disposal. Drop never infers pass.
pub fn canonical_fixture_store()
-> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    canonical_fixture_with_options(&canonical_fixture_options()?, true)
}

/// Explicit managed fixture selection. The registry owns profile resources and
/// exact worker publication; codec-only fixtures never infer this allocation.
pub fn canonical_managed_fixture_store(
    execution_profile: &str,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    canonical_fixture_with_profile(&canonical_fixture_options()?, true, Some(execution_profile))
}

fn canonical_fixture_options()
-> Result<crate::canonical::CanonicalOptions, crate::canonical::CanonicalError> {
    let state = std::env::var_os("PSE_SURREAL_STATE").ok_or_else(|| {
        crate::canonical::CanonicalError::Configuration(
            "canonical fixture requires recipe-selected PSE_SURREAL_STATE".into(),
        )
    })?;
    let mut options = crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state))?;
    options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
    Ok(options)
}

/// Register an explicit test database before connecting, then retain its physical
/// clients on the fixture executor through the final managed borrower.
/// `initialize` is false for tests whose subject is explicit schema installation.
pub fn canonical_fixture_with_options(
    options: &crate::canonical::CanonicalOptions,
    initialize: bool,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    canonical_fixture_with_profile(options, initialize, None)
}

fn canonical_fixture_with_profile(
    options: &crate::canonical::CanonicalOptions,
    initialize: bool,
    execution_profile: Option<&str>,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
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
    let resource = register_database(options, execution_profile)?;
    let mut selected = crate::canonical::CanonicalOptions::from_state_for_database(
        options.state_path(),
        Some(&options.database),
    )?;
    // Explicit test relay selection is separate from the registered DB allocation.
    selected.endpoint = options.endpoint.clone();
    let options = &selected;
    let store = std::thread::scope(|scope| {
        scope
            .spawn(|| executor.block_on(crate::canonical::CanonicalStore::connect(options)))
            .join()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "canonical fixture executor panicked".into(),
                )
            })?
    })?
    .own_fixture(executor, resource);
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

/// Open an independent physical client under the existing fixture association.
/// Its shared lifetime delays registry drain until every peer borrower finishes.
pub fn canonical_fixture_peer(
    owner: &crate::canonical::CanonicalStore,
    options: &crate::canonical::CanonicalOptions,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    let lifetime = owner.fixture_lifetime.as_ref().ok_or_else(|| {
        crate::canonical::CanonicalError::Configuration(
            "fixture peer requires registered owner".into(),
        )
    })?;
    if owner.database() != options.database || owner.deployment_state() != options.state_path() {
        return Err(crate::canonical::CanonicalError::Configuration(
            "fixture peer identity differs from registered owner".into(),
        ));
    }
    let peer = lifetime.execute(crate::canonical::CanonicalStore::connect(options))?;
    lifetime.retain_peer(peer.clone())?;
    peer.borrow_fixture(owner)
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
    async fn fixture_connections_preserve_deployment_database() {
        let external = options();
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
    }

    #[tokio::test]
    #[ignore = "explicit Rust/Python overlap control needs its owned rendezvous directory"]
    async fn plan30_rust_python_fixture_context_overlap() {
        let directory =
            std::path::PathBuf::from(std::env::var_os("PSE_PLAN30_C1_CONTROL").unwrap());
        let publish = |name: &str, value: serde_json::Value| {
            let temporary = directory.join(format!("{name}.tmp"));
            std::fs::write(&temporary, serde_json::to_vec(&value).unwrap()).unwrap();
            std::fs::rename(temporary, directory.join(name)).unwrap();
        };
        let store = canonical_fixture_store().unwrap();
        let marker = format!("rust:{}", store.database());
        bounded_query(store.db.query(
            "DEFINE TABLE c1_overlap_problem SCHEMALESS; CREATE c1_overlap_problem:same_authored_name SET marker = $marker;",
        ).bind(("marker", marker.clone()))).await.unwrap();
        let resource = &store.fixture_lifetime.as_ref().unwrap().resource;
        publish(
            "rust-ready.json",
            serde_json::json!({
                "database": store.database(),
                "resource": resource, "marker": marker,
            }),
        );
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
        while !directory.join("python-ready.json").exists() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "Python overlap peer did not arrive within the control deadline"
            );
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        let peer: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("python-ready.json")).unwrap())
                .unwrap();
        assert_ne!(peer["database"].as_str().unwrap(), store.database());
        let mut response = bounded_query(
            store
                .db
                .query("SELECT VALUE marker FROM c1_overlap_problem:same_authored_name;"),
        )
        .await
        .unwrap();
        let observed: Vec<String> = response.take(0).unwrap();
        assert_eq!(observed, vec![marker]);
        publish("rust-observed.json", serde_json::json!(observed));
        // Both proofs complete while both live fixture handles still own their
        // contexts; fixture Drop supplies drain, and the runner owns disposition.
        while !directory.join("python-observed.json").exists() {
            assert!(
                tokio::time::Instant::now() < deadline,
                "Python overlap proof did not complete within the control deadline"
            );
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }
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
    let Some(mut row) = response.take::<Option<surrealdb::types::Object>>(0)? else {
        return Ok(None);
    };
    Ok(Some(codec::decode_uint(codec::required(
        &mut row,
        "row_count",
    )?)?))
}

/// Expire only an acknowledged current claim in a registered isolated fixture.
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
    let mut response = bounded_query(store.db.query("BEGIN; LET $run = SELECT * FROM ONLY type::record('canonical_runs',$run); LET $attempt = SELECT * FROM ONLY type::record('canonical_attempts',$attempt); IF $run = NONE OR $attempt = NONE OR $attempt.run != $run.key OR $run.current_attempt != $attempt.key OR $run.current_generation != $generation OR $attempt.generation != $generation OR $attempt.terminal OR $attempt.closed { THROW 'fixture expiry claim changed'; }; UPDATE ONLY $attempt.id SET expires_at = time::micros() - 1; RETURN true; COMMIT;")
        .bind(("run", fence.run().to_owned())).bind(("attempt", fence.attempt().to_owned())).bind(("generation",generation))).await?;
    let index = response.num_statements().saturating_sub(2);
    if response.take::<Option<bool>>(index)? != Some(true) {
        return Err(CanonicalError::IncompleteResponse);
    }
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
