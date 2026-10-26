// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated canonical databases on the explicitly selected supervised server.

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
        .take::<Option<surrealdb::types::Object>>(0)?
        .ok_or(crate::canonical::CanonicalError::IncompleteResponse)?;
    Ok((
        decode_uint(required(&mut row, "inline")?)?,
        decode_uint(required(&mut row, "sealed")?)?,
    ))
}

/// Join already registered result-read cleanup without closing the fixture or
/// treating a facade drop as native/SDK drain. Future independent reads remain valid.
pub async fn drain_result_releases(
    store: &crate::canonical::CanonicalStore,
) -> Result<(), crate::canonical::CanonicalError> {
    store.result_read_drain.drain().await
}

/// A single fixture join clock, including helper startup and physical settlement.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SettlementDeadline {
    pub(crate) local: tokio::time::Instant,
    monotonic_ns: u64,
}
impl SettlementDeadline {
    pub(crate) fn new(
        local: tokio::time::Instant,
    ) -> Result<Self, crate::canonical::CanonicalError> {
        // Read CLOCK_MONOTONIC first: sampling skew can only shorten the offer.
        let now = crate::canonical::transport::monotonic_nanoseconds()?;
        let remaining = u64::try_from(
            local
                .saturating_duration_since(tokio::time::Instant::now())
                .as_nanos(),
        )
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?;
        Ok(Self {
            local,
            monotonic_ns: now.checked_add(remaining).ok_or_else(|| {
                crate::canonical::CanonicalError::Configuration("fixture deadline overflow".into())
            })?,
        })
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

/// Shared by all managed store borrowers, including queued result-read releases.
pub(crate) struct FixtureLifetime {
    store: crate::canonical::CanonicalStore,
    executor: &'static tokio::runtime::Runtime,
    pub(crate) removed: tokio::sync::Mutex<bool>,
    pub(crate) resource: String,
    peers: std::sync::Mutex<Vec<crate::canonical::CanonicalStore>>,
    settlement: std::sync::Mutex<Option<SettlementDeadline>>,
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
                let deadline = SettlementDeadline::new(offered)?;
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
    /// Join SDK owners and the separately supervised receiver. Native progress
    /// is independent; publication waits for all required acknowledgments.
    pub(crate) async fn settle(
        &self,
        current: Option<&crate::canonical::CanonicalStore>,
        deadline: SettlementDeadline,
    ) -> Result<(), crate::canonical::CanonicalError> {
        let resource = self.resource.clone();
        let native = tokio::task::spawn_blocking(move || {
            resource_bridge_until(
                "drain",
                &serde_json::json!({"resource": resource, "physical_only": true}),
                deadline,
            )
        });
        let (current, peers, native) = tokio::join!(
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
            native,
        );
        let mut errors = Vec::new();
        if let Err(error) = current {
            errors.push(format!("current fixture: {error}"));
        }
        if let Err(error) = peers {
            errors.push(format!("retained fixture owners: {error}"));
        }
        let units = match native {
            Ok(Ok(output)) => output
                .get("units")
                .filter(|value| value.is_array())
                .cloned()
                .ok_or_else(|| crate::canonical::CanonicalError::IncompleteResponse),
            Ok(Err(error)) => Err(error),
            Err(error) => Err(crate::canonical::CanonicalError::Configuration(format!(
                "fixture native drain task: {error}"
            ))),
        };
        if let Err(error) = &units {
            errors.push(format!("native receiver: {error}"));
        }
        if !errors.is_empty() {
            return Err(crate::canonical::CanonicalError::Configuration(
                errors.join("; "),
            ));
        }
        resource_bridge_until(
            "record-drain",
            &serde_json::json!({"resource": self.resource, "units": units?}),
            deadline,
        )
        .map(|_| ())
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

pub(crate) fn resource_bridge(
    action: &str,
    input: &serde_json::Value,
) -> Result<serde_json::Value, crate::canonical::CanonicalError> {
    let budget = std::time::Duration::from_secs(if matches!(action, "drain" | "record-drain") {
        50
    } else {
        60
    });
    let deadline = SettlementDeadline::new(crate::canonical::transport::original_deadline(budget))?;
    resource_bridge_until(action, input, deadline)
}

pub(crate) fn resource_bridge_until(
    action: &str,
    input: &serde_json::Value,
    deadline: SettlementDeadline,
) -> Result<serde_json::Value, crate::canonical::CanonicalError> {
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
    if tokio::time::Instant::now() >= deadline.local {
        return Err(crate::canonical::CanonicalError::Timeout);
    }
    let mut input = input.clone();
    if matches!(action, "register" | "drain" | "record-drain") {
        input["deadline_monotonic_ns"] = deadline.monotonic_ns.into();
    }
    let input = serde_json::to_vec(&input)
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))?;
    let output = execute_resource_bridge(command, input, deadline.local)?;
    if !output.status.success() {
        return Err(crate::canonical::CanonicalError::Configuration(format!(
            "fixture registry {action}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| crate::canonical::CanonicalError::Configuration(error.to_string()))
}

/// Bound pipe writes, both output streams and child exit by the original offer.
/// Killing an overdue helper is not evidence that its native receiver drained.
fn execute_resource_bridge(
    command: std::process::Command,
    input: Vec<u8>,
    deadline: tokio::time::Instant,
) -> Result<std::process::Output, crate::canonical::CanonicalError> {
    std::thread::scope(|scope| {
        scope
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| {
                        crate::canonical::CanonicalError::Configuration(error.to_string())
                    })?;
                runtime.block_on(crate::canonical::transport::within_clock(deadline, async {
                    use tokio::io::AsyncWriteExt;
                    if tokio::time::Instant::now() >= deadline {
                        return Err(crate::canonical::CanonicalError::Timeout);
                    }
                    let mut child = tokio::process::Command::from(command)
                        .kill_on_drop(true)
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|error| {
                            crate::canonical::CanonicalError::Configuration(error.to_string())
                        })?;
                    let mut stdin = child.stdin.take().ok_or_else(|| {
                        crate::canonical::CanonicalError::Configuration(
                            "fixture registry input unavailable".into(),
                        )
                    })?;
                    // Read stdout/stderr concurrently with input: a helper producing
                    // diagnostics must not deadlock before accepting its request.
                    let write = async move {
                        stdin.write_all(&input).await?;
                        drop(stdin);
                        Ok::<(), std::io::Error>(())
                    };
                    let (written, output) = tokio::join!(write, child.wait_with_output());
                    written.map_err(|error| {
                        crate::canonical::CanonicalError::Configuration(error.to_string())
                    })?;
                    output.map_err(|error| {
                        crate::canonical::CanonicalError::Configuration(error.to_string())
                    })
                }))
            })
            .join()
            .map_err(|_| {
                crate::canonical::CanonicalError::Configuration(
                    "fixture bridge executor panicked".into(),
                )
            })?
    })
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

fn fixture_setup_failure_payload(
    stage: &str,
    error: &crate::canonical::CanonicalError,
    test: &str,
    executable: &std::path::Path,
) -> Result<serde_json::Value, crate::canonical::CanonicalError> {
    let code = miette::Diagnostic::code(error).ok_or_else(|| {
        crate::canonical::CanonicalError::Configuration(
            "fixture setup error omitted its diagnostic code".into(),
        )
    })?;
    Ok(serde_json::json!({
        "test": test,
        "executable": executable,
        "stage": stage,
        "code": code.to_string(),
        "kind": "infrastructure",
        // A fixture helper may itself be a test subject after other assertions.
        "assertion_reach": "unknown",
    }))
}

fn fixture_setup_result<T>(
    stage: &str,
    result: Result<T, crate::canonical::CanonicalError>,
) -> Result<T, crate::canonical::CanonicalError> {
    result.inspect_err(|error| {
        // Bare test executions retain their narrower resource association; they
        // cannot publish an authenticated invocation observation.
        if std::env::var_os("PSE_TEST_INVOCATION").is_some() {
            let observation = selected_fixture_identity().and_then(|(test, executable)| {
                let payload = fixture_setup_failure_payload(stage, error, &test, &executable)?;
                resource_bridge("setup-failure", &payload).map(|_| ())
            });
            if let Err(observation_error) = observation {
                use std::io::Write;
                // Preserve the original typed error and make failed publication
                // visible. Missing authenticated evidence remains unknown.
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "canonical fixture setup observation failed: {observation_error}"
                );
            }
        }
    })
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
    if let Some(profile) = execution_profile {
        payload["execution_profile"] = profile.into();
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
    let options = fixture_setup_result("canonical-options", canonical_fixture_options())?;
    canonical_fixture_with_profile(&options, true, None)
}

/// Native workflow fixture: request its explicitly selected receiver context.
/// Database-only fixture callers do not inherit this demand from process placement.
pub fn canonical_native_fixture_store()
-> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    let profile = std::env::var("PSE_TEST_EXECUTION_PROFILE").ok();
    let options = fixture_setup_result("canonical-options", canonical_fixture_options())?;
    canonical_fixture_with_profile(&options, true, profile.as_deref())
}

/// Explicit managed fixture selection. The registry owns profile resources and
/// exact worker publication; codec-only fixtures never infer this allocation.
pub fn canonical_managed_fixture_store(
    execution_profile: &str,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    let options = fixture_setup_result("canonical-options", canonical_fixture_options())?;
    canonical_fixture_with_profile(&options, true, Some(execution_profile))
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
/// The supplied state's allocation is authoritative; ambient recipe profiles apply
/// only to `canonical_fixture_store`, not this explicit selection.
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
    let executor = fixture_setup_result(
        "canonical-executor",
        EXECUTOR
            .get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
            })
            .as_ref()
            .map_err(|error| {
                crate::canonical::CanonicalError::Configuration(format!(
                    "fixture executor: {error}"
                ))
            }),
    )?;
    let resource = fixture_setup_result(
        "canonical-register",
        register_database(options, execution_profile),
    )?;
    let mut selected = fixture_setup_result(
        "canonical-options",
        crate::canonical::CanonicalOptions::from_state_for_database(
            options.state_path(),
            Some(&options.database),
        ),
    )?;
    // Explicit test relay selection is separate from the registered DB allocation.
    selected.endpoint = options.endpoint.clone();
    let options = &selected;
    let store = fixture_setup_result(
        "canonical-connect",
        std::thread::scope(|scope| {
            scope
                .spawn(|| executor.block_on(crate::canonical::CanonicalStore::connect(options)))
                .join()
                .map_err(|_| {
                    crate::canonical::CanonicalError::Configuration(
                        "canonical fixture executor panicked".into(),
                    )
                })?
        }),
    )?
    .own_fixture(executor, resource);
    if initialize {
        fixture_setup_result(
            "canonical-create",
            std::thread::scope(|scope| {
                scope
                    .spawn(|| executor.block_on(store.create()))
                    .join()
                    .map_err(|_| {
                        crate::canonical::CanonicalError::Configuration(
                            "canonical fixture initialization executor panicked".into(),
                        )
                    })?
            }),
        )?;
    }
    Ok(store)
}

/// Open an independent physical client under the existing fixture association.
/// Its shared lifetime delays registry drain until every peer borrower finishes.
pub fn canonical_fixture_peer(
    owner: &crate::canonical::CanonicalStore,
    options: &crate::canonical::CanonicalOptions,
) -> Result<crate::canonical::CanonicalStore, crate::canonical::CanonicalError> {
    // Pin the complete fixture membership while the owner's admission is open,
    // before connection setup can race the final resource-drain decision.
    let borrowed_owner = fixture_setup_result("canonical-peer-borrow", owner.independent_caller())?;
    let lifetime = fixture_setup_result(
        "canonical-peer-owner",
        borrowed_owner.fixture_lifetime.as_ref().ok_or_else(|| {
            crate::canonical::CanonicalError::Configuration(
                "fixture peer requires registered owner".into(),
            )
        }),
    )?;
    if owner.database() != options.database || owner.deployment_state() != options.state_path() {
        return fixture_setup_result(
            "canonical-peer-identity",
            Err(crate::canonical::CanonicalError::Configuration(
                "fixture peer identity differs from registered owner".into(),
            )),
        );
    }
    let peer = fixture_setup_result(
        "canonical-peer-connect",
        lifetime.execute(crate::canonical::CanonicalStore::connect(options)),
    )?;
    fixture_setup_result("canonical-peer-open", lifetime.execute(peer.open()))?;
    fixture_setup_result("canonical-peer-retain", lifetime.retain_peer(peer.clone()))?;
    fixture_setup_result(
        "canonical-peer-borrow",
        peer.borrow_fixture(&borrowed_owner),
    )
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
        .take::<Option<surrealdb::types::Object>>(0)?
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
    let Some(mut row) = response.take::<Option<surrealdb::types::Object>>(0)? else {
        return Ok(None);
    };
    Ok(Some(codec::decode_uint(codec::required(
        &mut row,
        "row_count",
    )?)?))
}

/// Inspect actual staged file coordinates in an isolated schema-boundary control.
pub async fn pending_result_file_coordinates(
    store: &crate::canonical::CanonicalStore,
    fence: &crate::canonical_execution::AttemptFence,
    output: &str,
    ordinal: u64,
) -> Result<Option<(String, String)>, crate::canonical::CanonicalError> {
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
    Ok(Some((
        codec::decode_string(codec::required(&mut row, "file_instance")?)?,
        codec::decode_string(codec::required(&mut row, "schema")?)?,
    )))
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
    let mut response = bounded_query(store.db.query("BEGIN; LET $run = SELECT * FROM ONLY type::record('canonical_runs',$run); LET $attempt = SELECT * FROM ONLY type::record('canonical_attempts',$attempt); IF $run = NONE OR $attempt = NONE OR $attempt.run != $run.key OR $run.current_attempt != $attempt.key OR $run.current_generation != $generation OR $attempt.generation != $generation OR $attempt.phase IN ['terminal','terminal_compacted'] { THROW 'PSE_REFUSAL:fixture_expiry_claim_changed'; }; UPDATE ONLY $attempt.id SET expires_at = time::micros() - 1; RETURN true; COMMIT;")
        .bind(("run", fence.run().to_owned())).bind(("attempt", fence.attempt().to_owned())).bind(("generation",generation))).await?;
    let index = response.num_statements().saturating_sub(2);
    if response.take::<Option<bool>>(index)? != Some(true) {
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
    bounded_query(store.db.query("BEGIN; LET $study = SELECT * FROM ONLY type::record('canonical_studies',$study); IF $study = NONE OR !$study.terminal { THROW 'PSE_REFUSAL:fixture_study_is_not_terminal'; }; UPDATE ONLY type::record('canonical_study_points',$point) SET outcome=$outcome; COMMIT;")
        .bind(("study", study.to_owned()))
        .bind(("point", crate::canonical_studies::point_key(study, occurrence)))
        .bind(("outcome", surrealdb::types::Bytes::from(vec![0xff])))).await?;
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

    #[test]
    fn bridge_bounds_blocked_input_and_output_by_original_deadline() {
        let mut command = std::process::Command::new("/bin/sh");
        // The child fills stdout before reading stdin. Neither pipe may block
        // the outer expiry, and helper death never constitutes native drain.
        command.args([
            "-c",
            "while :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done",
        ]);
        let started = std::time::Instant::now();
        let result = super::execute_resource_bridge(
            command,
            vec![0; 1024 * 1024],
            tokio::time::Instant::now() + std::time::Duration::from_millis(50),
        );
        assert!(matches!(
            result,
            Err(crate::canonical::CanonicalError::Timeout)
        ));
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }

    #[test]
    fn bridge_rejects_expiry_before_spawning_child() {
        let directory = tempfile::tempdir().unwrap();
        let witness = directory.path().join("started");
        let mut command = std::process::Command::new("/bin/sh");
        command
            .args(["-c", "touch -- \"$1\"", "fixture-bridge"])
            .arg(&witness);
        let result = super::execute_resource_bridge(
            command,
            Vec::new(),
            tokio::time::Instant::now() - std::time::Duration::from_millis(1),
        );
        assert!(matches!(
            result,
            Err(crate::canonical::CanonicalError::Timeout)
        ));
        assert!(!witness.exists());
    }

    #[test]
    fn setup_failure_payload_uses_declared_diagnostic_and_exact_identity() {
        let error = crate::canonical::CanonicalError::Configuration(
            "assertion failed: validation::invariant".into(),
        );
        let executable = std::path::Path::new("/selected/test-executable");
        let payload = super::fixture_setup_failure_payload(
            "canonical-register",
            &error,
            "module::selected_test",
            executable,
        );
        assert_eq!(
            payload.ok(),
            Some(serde_json::json!({
                "test": "module::selected_test",
                "executable": executable,
                "stage": "canonical-register",
                "code": "runtime::infrastructure",
                "kind": "infrastructure",
                "assertion_reach": "unknown",
            }))
        );
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
