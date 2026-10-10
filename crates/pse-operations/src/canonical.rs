// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete remote canonical substrate. Scientific work runs outside guarded transactions.

use crate::{canonical_codec::CodecError, generated::surreal as wire};
#[path = "canonical_transport.rs"]
pub(crate) mod transport;
pub use pse_model::generated::runtime::{
    canonical_memberships::Row as Membership, canonical_revisions::Row as Revision,
    canonical_versions::Row as ObjectVersion,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use surrealdb::{
    Surreal,
    engine::remote::ws::Ws,
    opt::{Config, WebsocketConfig},
    types::{Object, Value},
};
use transport::{CanonicalClient, original_deadline, within_clock};

/// Initial negotiated protocol limit; batches must leave space for their envelope.
pub const MESSAGE_BYTES: usize = wire::RESULT_MESSAGE_BYTES;
/// Bounded submitted payload, leaving conservative room for query/record metadata.
pub const PAYLOAD_BYTES: usize = 3 * 1024 * 1024;
/// Client operation deadline. Mutations settle a lost acknowledgment by immutable identity.
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub(crate) const ACTIVATION_REQUEST_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(90);
// A finite contention window for sixteen clients sharing one retention guard.
// This does not change any RPC deadline or permit replay after uncertain completion.
const GUARDED_ATTEMPTS: u32 = 32;

/// Canonical protocol and guarded-operation failures.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(runtime::infrastructure))]
pub enum CanonicalError {
    #[error("canonical store configuration: {0}")]
    /// Invalid deployment or operation configuration.
    Configuration(String),
    #[error("canonical store driver: {0} (query domain: {domain:?})", domain = .0.query_details())]
    /// Protocol/database failure retaining its native typed cause.
    Driver(#[from] surrealdb::Error),
    #[error(transparent)]
    /// Declared wire domain or shape refusal.
    Codec(#[from] CodecError),
    #[error("canonical interpretation mismatch: expected {expected}, observed {observed}")]
    /// Installed scientific interpretation differs from this reader.
    Interpretation {
        /// Required immutable interpretation.
        expected: &'static str,
        /// Recorded interpretation or absence.
        observed: String,
    },
    #[error("canonical immutable operation identity was reused for different inputs")]
    /// An idempotency identity was reused for different supplied bytes.
    OperationReused,
    /// The guarded writer acknowledged cancellation of this exact study attempt.
    #[error("study cancellation refused {operation} for {run}/{attempt} generation {generation}")]
    StudyCancellation {
        /// Exact selected run.
        run: String,
        /// Exact selected attempt.
        attempt: String,
        /// Original writer generation.
        generation: u64,
        /// Refused writer operation, never inferred from a later observation.
        operation: String,
    },
    /// Publication may have reached the server; retain this exact operation for recovery.
    #[error("analysis {key} publication did not complete: {source}", key = .intent.key)]
    AnalysisUnsettled {
        /// Original sealed occurrence intent; never replace it with a new issue.
        intent: Box<crate::canonical_analyses::Analysis>,
        /// Original typed transport, validation or database failure.
        #[source]
        source: Box<CanonicalError>,
    },
    #[error("canonical write payload exceeds the bounded protocol batch")]
    /// A bounded payload or selector limit was exceeded.
    PayloadLimit,
    #[error("canonical store is quiesced")]
    /// The supervisor has closed write admission.
    Quiesced,
    #[error("canonical store request deadline expired")]
    /// No success is inferred from a timed-out request.
    Timeout,
    #[error("canonical query returned no statement completion")]
    /// A cancelled or incomplete query cannot establish data absence or write success.
    IncompleteResponse,
}
impl pse_diagnostics::TypedDiagnostic for CanonicalError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(match self {
            Self::Codec(_) => pse_diagnostics::DiagnosticCode::ValidationInvariant,
            _ => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        })
    }
}

/// Authenticated deployment parameters. Credentials never appear in Debug or status output.
#[derive(Clone)]
pub struct CanonicalOptions {
    /// Authenticated loopback native WebSocket endpoint.
    pub endpoint: String,
    /// Application namespace.
    pub namespace: String,
    /// Application or isolated fixture database.
    pub database: String,
    /// Authentication role, omitted from Debug.
    pub username: String,
    /// Authentication secret, omitted from Debug.
    pub password: String,
    /// Read-only account used to select an existing context without implicit DDL.
    pub selection_username: String,
    /// Read-only authentication secret, omitted from Debug.
    pub selection_password: String,
    /// Finite managed native-worker allocation read from the owning supervisor profile.
    pub native: NativeAllocation,
    /// Explicit verified interpreter, supervisor and worker for managed primary startup.
    pub primary_receiver: Option<ManagedPrimaryReceiver>,
    state_path: PathBuf,
    initializer_nonce: Option<String>,
}

const INITIALIZER_AUTHORITY: &str = "PSE_CANONICAL_INITIALIZER";

#[derive(serde::Deserialize, PartialEq, Eq)]
struct InitializerOwner {
    nonce: String,
    pid: u32,
    start: String,
    boot: String,
}

#[derive(serde::Deserialize)]
struct PendingRebuild {
    #[serde(default)]
    initializer: Option<InitializerOwner>,
}

#[derive(serde::Deserialize)]
struct WriteAdmission {
    namespace: String,
    database: String,
    accepting_writes: bool,
    #[serde(default)]
    derived_rebuild_pending: Option<PendingRebuild>,
}

impl WriteAdmission {
    fn read(state: &Path) -> Result<Self, CanonicalError> {
        let config = std::fs::read(state.join("config.json"))
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        serde_json::from_slice(&config)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))
    }

    fn require_initializer(
        &self,
        state: &Path,
        namespace: &str,
        database: &str,
        nonce: Option<&str>,
    ) -> Result<(), CanonicalError> {
        let owner = self
            .derived_rebuild_pending
            .as_ref()
            .and_then(|pending| pending.initializer.as_ref())
            .ok_or(CanonicalError::Quiesced)?;
        if self.accepting_writes
            || self.namespace != namespace
            || self.database != database
            || nonce != Some(owner.nonce.as_str())
            || owner.nonce.len() != 32
            || !owner.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(CanonicalError::Quiesced);
        }
        let recorded = std::fs::read(state.join("lifecycle-owner.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<InitializerOwner>(&bytes).ok());
        if recorded.as_ref() != Some(owner) {
            return Err(CanonicalError::Quiesced);
        }
        let boot = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .map_err(|_| CanonicalError::Quiesced)?;
        let process = std::fs::read_to_string(format!("/proc/{}/stat", owner.pid))
            .map_err(|_| CanonicalError::Quiesced)?;
        let fields = process
            .rsplit_once(')')
            .ok_or(CanonicalError::Quiesced)?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        if boot.trim() != owner.boot
            || fields.get(19).copied() != Some(owner.start.as_str())
            || !matches!(fields.first(), Some(state) if !matches!(*state, "Z" | "X" | "x"))
        {
            return Err(CanonicalError::Quiesced);
        }
        Ok(())
    }

    fn require_writes(
        &self,
        state: &Path,
        namespace: &str,
        database: &str,
        initializer_nonce: Option<&str>,
    ) -> Result<(), CanonicalError> {
        if self.derived_rebuild_pending.is_some() {
            return self.require_initializer(state, namespace, database, initializer_nonce);
        }
        // An old handle cannot provision its previous namespace after cutover.
        // Explicit fixture databases within the current namespace remain valid.
        if !self.accepting_writes || self.namespace != namespace {
            return Err(CanonicalError::Quiesced);
        }
        Ok(())
    }
}
/// Materialized executable association for the managed study receiver.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedPrimaryReceiver {
    /// Interpreter selected by the owning supervisor environment.
    pub supervisor_executable: PathBuf,
    /// Supervisor entry point selected at profile construction.
    pub supervisor_script: PathBuf,
    /// Exact linked worker selected at profile construction.
    pub worker_executable: PathBuf,
    /// Supervisor file SHA-256 association, verified at launch.
    pub supervisor_sha256: String,
    /// Linked worker file SHA-256 association, verified at launch and readiness.
    pub worker_sha256: String,
}
/// The application profile's separately capped native-worker slots.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub struct NativeAllocation {
    /// Maximum managed worker processes sharing this store.
    pub native_workers: usize,
    /// Hard process memory cap for each worker.
    pub native_worker_memory_bytes: usize,
    /// Optional materialized shared-runtime execution profile.
    #[serde(default)]
    pub execution: Option<NativeExecutionAllocation>,
}
/// Finite runtime allocation materialized by the existing deployment supervisor.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExecutionAllocation {
    /// Shared application pool capacity, distinct from the process cap.
    pub pool_memory_bytes: usize,
    /// Maximum aggregate numeric capacity of one case worker.
    pub worker_bytes: usize,
    /// Shared CPU permits for all case and inner native work.
    pub cpu_threads: usize,
    /// Maximum ordinary concurrent case lanes in the primary group.
    pub case_lanes: usize,
    /// Total pending, executing and retained session population.
    pub math_jobs: usize,
    /// Ordinary optimizer team width.
    pub compiler_cores: usize,
    /// Admission-only clock where no enclosing deadline exists.
    pub admission_wait_ms: u64,
    /// Explicit process headroom policy; not a measured resident-memory bound.
    pub process_headroom_bytes: usize,
    /// Separately placed observer allocation without native assistance.
    pub observer_memory_bytes: usize,
}
impl NativeAllocation {
    /// Validate the selected finite profile without inferring process RSS from pool capacity.
    /// # Errors
    /// Invalid populations or a runtime allocation that exceeds the process cap.
    pub fn validate(&self) -> Result<(), CanonicalError> {
        if !(1..=32).contains(&self.native_workers) || self.native_worker_memory_bytes == 0 {
            return Err(CanonicalError::Configuration(
                "finite native-worker allocation required".into(),
            ));
        }
        if let Some(profile) = self.execution
            && (self.native_workers != 1
                || profile.pool_memory_bytes == 0
                || profile.worker_bytes == 0
                || profile.worker_bytes > profile.pool_memory_bytes
                || profile.cpu_threads == 0
                || profile.case_lanes == 0
                || profile.case_lanes > profile.cpu_threads
                || profile
                    .case_lanes
                    .checked_mul(2)
                    .is_none_or(|population| population > profile.math_jobs)
                || profile.compiler_cores == 0
                || profile.compiler_cores > profile.cpu_threads
                || profile.admission_wait_ms == 0
                || profile.process_headroom_bytes == 0
                || profile.observer_memory_bytes == 0
                || profile
                    .pool_memory_bytes
                    .checked_add(profile.process_headroom_bytes)
                    .is_none_or(|bytes| bytes > self.native_worker_memory_bytes))
        {
            return Err(CanonicalError::Configuration(
                "shared execution profile exceeds the managed allocation".into(),
            ));
        }
        Ok(())
    }
}
impl std::fmt::Debug for CanonicalOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalOptions")
            .field("endpoint", &self.endpoint)
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}
impl CanonicalOptions {
    /// Selected deployment state for fixture registration before connection.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn state_path(&self) -> &Path {
        &self.state_path
    }

    /// Read the supervisor's private state, including its admission gate.
    pub fn from_state(state: &Path) -> Result<Self, CanonicalError> {
        Self::from_state_for_database(state, None)
    }
    /// Select an explicit validated database without rewriting service configuration.
    pub fn from_state_for_database(
        state: &Path,
        database: Option<&str>,
    ) -> Result<Self, CanonicalError> {
        #[derive(serde::Deserialize)]
        struct Deployment {
            profile_version: u32,
            endpoint: String,
            namespace: String,
            database: String,
            schema_interpretation: String,
            max_message_bytes: usize,
            credentials_file: String,
            resources: NativeAllocation,
            #[serde(default)]
            primary_receiver: Option<ManagedPrimaryReceiver>,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Credentials {
            username: String,
            password: String,
            selection_username: String,
            selection_password: String,
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let directory = std::fs::symlink_metadata(state)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
            if !directory.is_dir()
                || directory.file_type().is_symlink()
                || directory.mode() & 0o077 != 0
            {
                return Err(CanonicalError::Configuration(
                    "canonical state must be a private real directory".into(),
                ));
            }
            #[cfg(target_os = "linux")]
            if directory.uid()
                != std::fs::metadata("/proc/self")
                    .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                    .uid()
            {
                return Err(CanonicalError::Configuration(
                    "canonical state must belong to the current application user".into(),
                ));
            }
        }
        let read = |path: &Path| -> Result<Vec<u8>, CanonicalError> {
            let metadata = std::fs::symlink_metadata(path)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > 64 * 1024
            {
                return Err(CanonicalError::Configuration(
                    "canonical configuration must be a bounded real file".into(),
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.mode() & 0o077 != 0
                    || metadata.uid()
                        != std::fs::metadata(state)
                            .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                            .uid()
                {
                    return Err(CanonicalError::Configuration("canonical configuration and credentials must be private and share the state owner".into()));
                }
            }
            std::fs::read(path).map_err(|error| CanonicalError::Configuration(error.to_string()))
        };
        let deployment: Deployment = serde_json::from_slice(&read(&state.join("config.json"))?)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        if deployment.profile_version != 2 {
            return Err(CanonicalError::Configuration("canonical service profile revision 2 required; explicitly upgrade/readmit an older profile".into()));
        }
        let mut deployment = deployment;
        if let Some(database) = database {
            if database.is_empty()
                || database.len() > 128
                || !database
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(CanonicalError::Configuration(
                    "explicit database must be a bounded ASCII identifier".into(),
                ));
            }
            deployment.database = database.to_owned();
            let context_path = state.join(".contexts").join(format!("{database}.json"));
            if context_path
                .try_exists()
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            {
                #[derive(serde::Deserialize)]
                struct Context {
                    version: u32,
                    database: String,
                    resources: NativeAllocation,
                    #[serde(default)]
                    primary_receiver: Option<ManagedPrimaryReceiver>,
                }
                let context: Context = serde_json::from_slice(&read(&context_path)?)
                    .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
                if context.version != 1 || context.database != database {
                    return Err(CanonicalError::Configuration(
                        "registered test context identity/version mismatch".into(),
                    ));
                }
                deployment.resources = context.resources;
                deployment.primary_receiver = context.primary_receiver;
            }
        }
        let initializer_nonce = std::env::var(INITIALIZER_AUTHORITY).ok();
        let admission = WriteAdmission::read(state)?;
        if admission.derived_rebuild_pending.is_some() {
            admission.require_initializer(
                state,
                &deployment.namespace,
                &deployment.database,
                initializer_nonce.as_deref(),
            )?;
        }
        if deployment.schema_interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Interpretation {
                expected: wire::INTERPRETATION,
                observed: deployment.schema_interpretation,
            });
        }
        if deployment.max_message_bytes != MESSAGE_BYTES {
            return Err(CanonicalError::Configuration(
                "unsupported protocol bound".into(),
            ));
        }
        deployment.resources.validate()?;
        if let Some(receiver) = &deployment.primary_receiver {
            let digest = |value: &str| {
                value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            };
            if deployment.resources.execution.is_none()
                || !receiver.supervisor_executable.is_absolute()
                || !receiver.supervisor_script.is_absolute()
                || !receiver.worker_executable.is_absolute()
                || !digest(&receiver.supervisor_sha256)
                || !digest(&receiver.worker_sha256)
            {
                return Err(CanonicalError::Configuration("managed primary requires an execution profile and exact executable association".into()));
            }
        }
        let credentials: Credentials =
            serde_json::from_slice(&read(Path::new(&deployment.credentials_file))?)
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        Ok(Self {
            endpoint: deployment.endpoint,
            namespace: deployment.namespace,
            database: deployment.database,
            username: credentials.username,
            password: credentials.password,
            selection_username: credentials.selection_username,
            selection_password: credentials.selection_password,
            native: deployment.resources,
            primary_receiver: deployment.primary_receiver,
            state_path: state.to_owned(),
            initializer_nonce,
        })
    }
}

/// Immutable authored mutation. A removal has no new version or references.
#[derive(Clone, Debug)]
pub struct ObjectEdit {
    /// Stable authored logical object identity.
    pub logical: String,
    /// Lexical namespace of this object.
    pub scope: String,
    /// Authored name within the lexical namespace.
    pub name: String,
    /// New immutable version; absent for removal.
    pub version: Option<ObjectVersion>,
    /// Structural reference scope/name pairs.
    pub references: Vec<(String, String)>,
}

/// A store-issued immutable preparation/read protection; its expiry is checked server-side.
#[derive(Clone, Debug)]
pub struct ProtectedSelection {
    owner: Arc<ProtectionOwner>,
}
#[derive(Debug)]
struct ProtectionOwner {
    key: String,
    revision: Revision,
    store: Option<CanonicalStore>,
    executor: Option<tokio::runtime::Handle>,
    released: std::sync::atomic::AtomicBool,
}
impl ProtectedSelection {
    pub(crate) fn key(&self) -> &str {
        &self.owner.key
    }
    /// Exact canonical revision retained by this protection.
    pub fn revision(&self) -> &Revision {
        &self.owner.revision
    }
    /// Relinquish this reader. The final follower awaits exact protection cleanup.
    /// Other live followers retain their authority until they finish or drop.
    pub async fn finish(self) -> Result<(), CanonicalError> {
        if let Some(owner) = Arc::into_inner(self.owner) {
            let Some(store) = owner.store.as_ref() else {
                return Ok(());
            };
            let store = store.clone();
            let key = owner.key.clone();
            let revision = owner.revision.clone();
            let executor = owner.executor.clone().ok_or_else(|| {
                CanonicalError::Configuration("protection cleanup executor unavailable".into())
            })?;
            let task = executor.spawn(async move {
                let result = store.release_protection(&revision, &key).await;
                if result.is_ok() {
                    owner.acknowledge_release();
                }
                result
            });
            return task.await.map_err(|error| {
                CanonicalError::Configuration(format!("protection cleanup task failed: {error}"))
            })?;
        }
        Ok(())
    }
}
impl ProtectionOwner {
    fn acknowledge_release(&self) {
        if !self
            .released
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
            if let Some(store) = &self.store {
                store.result_read_drain.release().complete(Ok(()));
            }
        }
    }
}
impl Drop for ProtectionOwner {
    fn drop(&mut self) {
        if self.released.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        if let (Some(store), Some(executor)) = (&self.store, &self.executor) {
            #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
            let release = store.result_read_drain.release();
            let store = store.clone();
            let revision = self.revision.clone();
            let key = self.key.clone();
            executor.spawn(async move {
                let result = store.release_protection(&revision, &key).await;
                if let Err(error) = &result {
                    operation_failed("canonical::protection_drop", error);
                }
                #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
                release.complete(result);
            });
        }
    }
}

/// Local pacing of complete short decisions sharing one problem's server guards.
/// The server's guarded predicates and generations remain the distributed authority.
#[derive(Default)]
struct StagingTurns {
    problems: std::sync::Mutex<
        std::collections::BTreeMap<String, std::sync::Weak<tokio::sync::Mutex<()>>>,
    >,
}
impl StagingTurns {
    async fn run<F, T>(
        &self,
        problem: &str,
        deadline: tokio::time::Instant,
        operation: F,
    ) -> Result<T, CanonicalError>
    where
        F: Future<Output = Result<T, CanonicalError>>,
    {
        if tokio::time::Instant::now() >= deadline {
            return Err(CanonicalError::Timeout);
        }
        let gate = self.gate(problem)?;
        within_clock(deadline, async {
            let _turn = gate.lock_owned().await;
            if tokio::time::Instant::now() >= deadline {
                return Err(CanonicalError::Timeout);
            }
            operation.await
        })
        .await
    }
    fn gate(&self, problem: &str) -> Result<Arc<tokio::sync::Mutex<()>>, CanonicalError> {
        let mut problems = self.problems.lock().map_err(|_| {
            CanonicalError::Configuration("staging admission owner poisoned".into())
        })?;
        problems.retain(|_, gate| gate.strong_count() != 0);
        if let Some(gate) = problems.get(problem).and_then(std::sync::Weak::upgrade) {
            return Ok(gate);
        }
        let gate = Arc::new(tokio::sync::Mutex::new(()));
        problems.insert(problem.to_owned(), Arc::downgrade(&gate));
        Ok(gate)
    }
}

#[derive(Clone)]
/// Thin remote client; clones share its bounded transport.
pub struct CanonicalStore {
    pub(crate) db: Arc<CanonicalClient>,
    staging_turns: Arc<StagingTurns>,
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fixture_lifetime: Option<Arc<crate::testing::FixtureLifetime>>,
    #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
    pub(crate) result_read_drain: Arc<crate::canonical_results::ResultReadDrain>,
    activation_db: Arc<CanonicalClient>,
    connection_options: CanonicalOptions,
    state_path: PathBuf,
    database: String,
    namespace: String,
}
impl std::fmt::Debug for CanonicalStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalStore")
            .field("state", &self.state_path)
            .finish_non_exhaustive()
    }
}
impl CanonicalStore {
    /// Drain local result readers and close both physical SDK drivers.
    /// This establishes local transport drain, not server scientific completion.
    pub async fn disconnect(&self) -> Result<(), CanonicalError> {
        #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
        self.result_read_drain.drain().await?;
        self.db.disconnect().await?;
        self.activation_db.disconnect().await?;
        Ok(())
    }
    /// State selected by this authenticated deployment handle.
    pub fn deployment_state(&self) -> &Path {
        &self.state_path
    }
    /// Re-read the validated managed native allocation at its existing owner.
    /// # Errors
    /// Invalid or unavailable owning deployment configuration.
    pub fn native_allocation(&self) -> Result<NativeAllocation, CanonicalError> {
        Ok(
            CanonicalOptions::from_state_for_database(&self.state_path, Some(&self.database))?
                .native,
        )
    }
    /// Re-read the selected executable association for the managed primary receiver.
    /// # Errors
    /// Invalid or unavailable owning deployment configuration.
    pub fn managed_primary_receiver(
        &self,
    ) -> Result<Option<ManagedPrimaryReceiver>, CanonicalError> {
        Ok(
            CanonicalOptions::from_state_for_database(&self.state_path, Some(&self.database))?
                .primary_receiver,
        )
    }
    /// One owner-local turn covers only a bounded RPC, never hydration or
    /// scientific work. Its original request clock includes queueing; dropping a
    /// waiter or request releases local ownership without inferring remote completion.
    pub(crate) async fn staging_query<F>(
        &self,
        problem: &str,
        future: F,
    ) -> Result<surrealdb::IndexedResults, CanonicalError>
    where
        F: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
    {
        self.staging_turns
            .run(
                problem,
                original_deadline(REQUEST_TIMEOUT),
                bounded_query(future),
            )
            .await
    }

    /// Pace a complete protected RPC alongside staging under its original request
    /// clock. Definite conflicts rebuild the checked query; uncertain completion
    /// retains its typed failure. No turn escapes into hydration or scientific work.
    pub(crate) async fn protected_query<F, Q>(
        &self,
        problem: &str,
        operation: &'static str,
        build: F,
    ) -> Result<surrealdb::IndexedResults, CanonicalError>
    where
        F: FnMut() -> Result<Q, CanonicalError>,
        Q: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
    {
        self.staging_turns
            .run(
                problem,
                original_deadline(REQUEST_TIMEOUT),
                protected_query(operation, build),
            )
            .await
    }

    /// Exact configured canonical database selected by this deployment handle.
    pub fn database(&self) -> &str {
        &self.database
    }
    /// Refuse new application writes and native claims while the supervisor is draining.
    pub fn check_write_admission(&self) -> Result<(), CanonicalError> {
        self.ensure_writes()
    }
    pub(crate) fn ensure_writes(&self) -> Result<(), CanonicalError> {
        WriteAdmission::read(&self.state_path)?.require_writes(
            &self.state_path,
            &self.namespace,
            &self.database,
            self.connection_options.initializer_nonce.as_deref(),
        )
    }
    /// Connect to the supported authenticated native WebSocket deployment.
    pub async fn connect(options: &CanonicalOptions) -> Result<Self, CanonicalError> {
        for identifier in [&options.namespace, &options.database] {
            if identifier.is_empty()
                || identifier.len() > 128
                || !identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(CanonicalError::Configuration(
                    "namespace/database must be bounded ASCII identifiers".into(),
                ));
            }
        }
        let endpoint = url::Url::parse(&options.endpoint)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        if endpoint.scheme() != "ws"
            || !matches!(endpoint.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
        {
            return Err(CanonicalError::Configuration(
                "supported profile requires loopback native WebSocket".into(),
            ));
        }
        let address = endpoint
            .socket_addrs(|| None)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| CanonicalError::Configuration("endpoint has no port".into()))?;
        let deadline = original_deadline(REQUEST_TIMEOUT);
        let (db, activation_db) = within_clock(deadline, async {
            let db = connect_client(address, options, REQUEST_TIMEOUT, true).await?;
            let activation_db =
                connect_client(address, options, ACTIVATION_REQUEST_TIMEOUT, true).await?;
            Ok((db, activation_db))
        })
        .await?;
        Ok(Self {
            db,
            staging_turns: Arc::default(),
            #[cfg(any(test, feature = "test-support"))]
            fixture_lifetime: None,
            #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
            result_read_drain: Arc::new(crate::canonical_results::ResultReadDrain::default()),
            activation_db,
            connection_options: options.clone(),
            state_path: options.state_path.clone(),
            database: options.database.clone(),
            namespace: options.namespace.clone(),
        })
    }
    /// Explicit schema creation; ordinary opening never mutates an unknown schema.
    pub async fn create(&self) -> Result<(), CanonicalError> {
        self.initialize_with(|| self.submit_initialization()).await
    }
    async fn initialize_with<F, Q>(&self, mut submit: F) -> Result<(), CanonicalError>
    where
        F: FnMut() -> Q,
        Q: Future<Output = Result<surrealdb::IndexedResults, CanonicalError>>,
    {
        within_clock(original_deadline(ACTIVATION_REQUEST_TIMEOUT), async {
            for attempt in 0..8 {
                self.ensure_writes()?;
                if !self.initialization_required().await? {
                    return self.open().await;
                }
                // Only an acknowledged transaction rejection allows a fresh complete
                // installation decision. Lost responses and cancellation stay uncertain.
                self.provision_context().await?;
                match submit().await.and_then(complete_response) {
                    Err(CanonicalError::Driver(error))
                        if matches!(
                            error.query_details(),
                            Some(surrealdb::types::QueryError::TransactionConflict)
                        ) && attempt < 7 =>
                    {
                        // Desynchronize independent databases sharing physical history.
                        let jitter = u64::from(uuid::Uuid::new_v4().as_bytes()[0] % 16);
                        tokio::time::sleep(std::time::Duration::from_millis(
                            (1 << attempt) + jitter,
                        ))
                        .await;
                    }
                    Err(error) => return Err(error),
                    // Readback failure after an acknowledged commit never replays DDL.
                    Ok(_) => return self.open().await,
                }
            }
            Err(CanonicalError::Configuration(
                "initialization retry extent".into(),
            ))
        })
        .await
    }
    /// INFO FOR DB uses the server's existing catalog lookup, never implicit
    /// namespace/database provisioning. Only exact typed absence is accepted.
    async fn database_inventory(&self) -> Result<Option<Object>, CanonicalError> {
        if !self.db.select_existing().await? {
            return Ok(None);
        }
        let mut response = match bounded_query(self.db.control_query("INFO FOR DB;")).await {
            Ok(response) => response,
            Err(CanonicalError::Driver(error))
                if matches!(error.not_found_details(),
                Some(surrealdb::types::NotFoundError::Namespace { name }) if name == &self.namespace)
                    || matches!(error.not_found_details(),
                Some(surrealdb::types::NotFoundError::Database { name }) if name == &self.database) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let Value::Object(info) = response.take::<Value>(0)? else {
            return Err(CanonicalError::IncompleteResponse);
        };
        Ok(Some(info))
    }
    async fn database_exists(&self) -> Result<bool, CanonicalError> {
        self.db.select_existing().await
    }
    async fn initialization_required(&self) -> Result<bool, CanonicalError> {
        self.ensure_writes()?;
        // Refuse an unmarked partial database rather than declaring imported records ready.
        let Some(info) = self.database_inventory().await? else {
            return Ok(true);
        };
        if let Some(Value::Object(tables)) = info.get("tables") {
            if tables.contains_key("canonical_interpretations") {
                return Ok(false);
            }
            if tables.keys().any(|name| name.starts_with("canonical_")) {
                return Err(CanonicalError::Configuration(
                    "unmarked canonical tables require explicit rebuild".into(),
                ));
            }
        }
        Ok(true)
    }
    /// Creation explicitly provisions catalog entries before selecting them;
    /// the complete canonical schema remains a separate atomic transaction.
    async fn provision_context(&self) -> Result<(), CanonicalError> {
        // Failed ordinary selection retains the missing database name. The
        // administrative DDL session starts unselected, using the same immutable
        // connection premises and bounded native transport as this store.
        let address = url::Url::parse(&self.connection_options.endpoint)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .socket_addrs(|| None)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| CanonicalError::Configuration("endpoint has no port".into()))?;
        let administration = connect_client(
            address,
            &self.connection_options,
            ACTIVATION_REQUEST_TIMEOUT,
            false,
        )
        .await?;
        let provisioned = administration.provision().await;
        let disconnected = administration.disconnect().await;
        provisioned?;
        disconnected?;
        if !self.activation_db.select_existing().await? || !self.db.select_existing().await? {
            return Err(CanonicalError::IncompleteResponse);
        }
        Ok(())
    }

    async fn submit_initialization(&self) -> Result<surrealdb::IndexedResults, CanonicalError> {
        self.ensure_writes()?;
        // Installing the complete schema is an atomic structural transition.
        // Its client deadline must match that role, rather than an ordinary read.
        let query = self
            .activation_db
            .query(initialization_statement())
            .bind(("interpretation", wire::INTERPRETATION))
            .bind(("schema_digest", wire::SCHEMA_DIGEST));
        tokio::time::timeout(ACTIVATION_REQUEST_TIMEOUT, query.into_future())
            .await
            .map_err(|_| CanonicalError::Timeout)
            .and_then(|response| response.map_err(CanonicalError::from))
    }
    /// Read and verify the installed interpretation without DDL.
    pub async fn open(&self) -> Result<(), CanonicalError> {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            if !self.database_exists().await? {
                return Err(CanonicalError::Interpretation {
                    expected: wire::INTERPRETATION,
                    observed: "absent".into(),
                });
            }
            if !self.activation_db.select_existing().await? {
                return Err(CanonicalError::IncompleteResponse);
            }
            let marker: Option<Object> =
                request(self.db.select(("canonical_interpretations", "current"))).await?;
            let Some(marker) = marker else {
                return Err(CanonicalError::Interpretation {
                    expected: wire::INTERPRETATION,
                    observed: "absent".into(),
                });
            };
            let marker = wire::decode_canonical_interpretations(marker)?;
            if marker.interpretation != wire::INTERPRETATION {
                return Err(CanonicalError::Interpretation {
                    expected: wire::INTERPRETATION,
                    observed: marker.interpretation,
                });
            }
            if marker.schema_digest != wire::SCHEMA_DIGEST {
                return Err(CanonicalError::Interpretation {
                    expected: wire::SCHEMA_DIGEST,
                    observed: marker.schema_digest,
                });
            }
            Ok(())
        })
        .await
    }
    /// Exact canonical revision retained by this protection.
    pub async fn revision(&self, id: &str) -> Result<Option<Revision>, CanonicalError> {
        let row: Option<Object> = request(
            self.db
                .control_query("SELECT * FROM ONLY type::record('canonical_revisions', $key);")
                .bind(("key", id.to_owned())),
        )
        .await?
        .take(0)?;
        row.map(wire::decode_canonical_revisions)
            .transpose()
            .map_err(Into::into)
    }
    /// Atomic head-CAS edit. The operation ID settles an acknowledgment lost after commit.
    pub async fn edit(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        edits: &[ObjectEdit],
    ) -> Result<Revision, CanonicalError> {
        within_clock(original_deadline(ACTIVATION_REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let stage = match self
                .stage_edits(problem, expected, operation, edits)
                .await?
            {
                crate::canonical_staging::StageOutcome::Acknowledged(revision) => {
                    return Ok(revision);
                }
                crate::canonical_staging::StageOutcome::Closed(stage) => stage,
            };
            self.activate_source_stage(&stage).await
        })
        .await
    }
    /// Activate only the caller's closed source stage under its generation and head fences.
    pub(crate) async fn activate_source_stage(
        &self,
        stage: &crate::canonical_staging::ClosedStage,
    ) -> Result<Revision, CanonicalError> {
        let deadline = original_deadline(ACTIVATION_REQUEST_TIMEOUT);
        within_clock(deadline, async {
        for attempt in 0..GUARDED_ATTEMPTS {
            self.ensure_writes()?;
            let query = self
                .activation_db
                .query(crate::canonical_staging::bounded_edit_sql(EDIT))
                .bind(("problem", stage.problem.clone()))
                .bind(("expected", stage.expected.clone()))
                .bind(("operation", stage.operation.clone()))
                .bind(("interpretation", wire::INTERPRETATION))
                .bind((
                    "request",
                    surrealdb::types::Bytes::from(stage.request.clone()),
                ))
                .bind(("request_digest", stage.request_digest.clone()))
                .bind((
                    "generation",
                    crate::canonical_codec::encode_uint(stage.generation)?,
                ));
            let result = tokio::time::timeout(ACTIVATION_REQUEST_TIMEOUT, query.into_future())
                .await
                .map_err(|_| CanonicalError::Timeout)
                .and_then(|response| response.map_err(CanonicalError::from))
                .and_then(complete_response);
            if let Err(error) = result {
                // A lost acknowledgment or concurrent identical activation is settled
                // exclusively by the immutable operation receipt, never by the head.
                if let Ok(Some(revision)) = self.revision(&stage.operation).await {
                    return if revision.request.as_slice() == stage.request.as_slice() {
                        Ok(revision)
                    } else {
                        Err(CanonicalError::OperationReused)
                    };
                }
                if matches!(&error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
                    && attempt + 1 < GUARDED_ATTEMPTS
                {
                    conflict_backoff(attempt).await;
                    continue;
                }
                operation_failed("canonical::activate_source_stage", &error);
                return Err(error);
            }
            let revision = self.revision(&stage.operation).await?;
            return revision
                .ok_or_else(|| CanonicalError::Configuration("committed revision missing".into()));
        }
        Err(CanonicalError::Configuration(
            "staged activation retries exhausted".into(),
        ))
        }).await
    }
    /// Explicit historical inventory, indexed by problem and revision interval.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn inventory(
        &self,
        revision: &Revision,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let mut response = bounded_query(self.db.query("SELECT * FROM canonical_memberships WHERE problem = $problem AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) ORDER BY scope, name;")
            .bind(("problem", revision.problem.clone())).bind(("sequence", crate::canonical_codec::encode_uint(revision.sequence)?))).await?;
        let rows: Vec<Object> = response.take(0)?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
    /// Read an immutable version by exact identity.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn object(
        &self,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        self.assemble_object(version).await
    }
    /// Protect before off-transaction scientific work. Server time controls expiration.
    pub async fn protect(
        &self,
        revision: Revision,
        lifetime: std::time::Duration,
    ) -> Result<ProtectedSelection, CanonicalError> {
        // Read protections are lease bookkeeping, including gated restore
        // verification. Scientific mutation and reclamation remain quiesced.
        let micros = i64::try_from(lifetime.as_micros())
            .map_err(|_| CanonicalError::Configuration("protection lifetime overflow".into()))?;
        if micros <= 0 {
            return Err(CanonicalError::Configuration(
                "positive protection lifetime required".into(),
            ));
        }
        #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
        self.result_read_drain.retain();
        let selection = ProtectedSelection {
            owner: Arc::new(ProtectionOwner {
                key: uuid::Uuid::new_v4().to_string(),
                revision,
                store: Some(self.clone()),
                executor: Some(tokio::runtime::Handle::current()),
                released: std::sync::atomic::AtomicBool::new(false),
            }),
        };
        let store = self.clone();
        let deadline = original_deadline(REQUEST_TIMEOUT);
        // Submission owns the protection before any RPC. Abandoning the caller detaches
        // this task; its eventual result still owns cleanup through acknowledgement.
        tokio::spawn(async move {
            within_clock(deadline, store.acquire_protection(selection, micros)).await
        })
        .await
        .map_err(|error| {
            CanonicalError::Configuration(format!("protection acquisition task failed: {error}"))
        })?
    }
    async fn acquire_protection(
        &self,
        selection: ProtectedSelection,
        micros: i64,
    ) -> Result<ProtectedSelection, CanonicalError> {
        let key = selection.key();
        let revision = selection.revision();
        let mut response = self.protected_query(&revision.problem, "canonical::protect", || Ok(self.db.query(r#"BEGIN;
            SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            LET $revision = SELECT * FROM ONLY type::record('canonical_revisions', $revision);
            IF $revision = NONE OR $revision.problem != $problem { THROW 'revision unavailable'; };
            LET $reclaimed = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $problem AND from_sequence <= $revision.sequence AND to_sequence > $revision.sequence LIMIT 1;
            IF array::len($reclaimed) != 0 { THROW 'source selection reclaimed'; };
            IF (SELECT * FROM ONLY type::record('canonical_protections', $key)) = NONE { CREATE type::record('canonical_protections', $key) SET key = $key, problem = $problem, revision = $revision.key, sequence = $revision.sequence, expires_at = time::micros() + $lifetime, released = false; };
            UPSERT type::record('canonical_guards', 'retention:' + $problem) SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
            LET $rpc_result = SELECT * FROM ONLY type::record('canonical_revisions', $revision.key) TIMEOUT $pse_rpc_timeout;
            fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            LET $protection = SELECT * FROM ONLY type::record('canonical_protections', $key);
            IF $protection = NONE OR $protection.released OR $protection.expires_at <= time::micros() { THROW 'immutable selection protection expired'; };
            RETURN $rpc_result;
            COMMIT;"#).bind(("problem", revision.problem.clone())).bind(("revision", revision.key.clone())).bind(("key", key.to_owned())).bind(("lifetime", micros)))).await?;
        let actual: Option<Object> = response.take(response.num_statements().saturating_sub(2))?;
        let revision = wire::decode_canonical_revisions(actual.ok_or_else(|| {
            CanonicalError::Configuration("protected revision unavailable".into())
        })?)?;
        if revision != *selection.revision() {
            return Err(CanonicalError::Configuration(
                "protected revision differs from requested revision".into(),
            ));
        }
        Ok(selection)
    }
    /// Resolve a bounded namespace/name premise in the exact protected revision.
    pub async fn select_names(
        &self,
        selection: &ProtectedSelection,
        scope: &str,
        names: &[String],
    ) -> Result<Vec<Membership>, CanonicalError> {
        if names.len() > 256 {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = self.protected_query(&selection.revision().problem, "canonical::select_names", || Ok(self.db.query(format!("{}\nLET $rpc_result = SELECT * FROM canonical_memberships WHERE problem = $problem AND scope = $scope AND name IN $names AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'immutable selection protection expired'; }};\nRETURN $rpc_result;\nCOMMIT;", PROTECTED_BEGIN))
            .bind(("problem", selection.revision().problem.clone())).bind(("revision", selection.revision().key.clone())).bind(("protection", selection.key().to_owned())).bind(("scope", scope.to_owned())).bind(("names", names.to_vec())).bind(("sequence", crate::canonical_codec::encode_uint(selection.revision().sequence)?)))).await?;
        let rows: Vec<Object> = response.take(response.num_statements().saturating_sub(2))?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
    /// Product and revision roots become durable before the caller explicitly ends
    /// the selection protection. Multiple body products may share the same selection.
    pub(crate) async fn admit_product(
        &self,
        selection: &ProtectedSelection,
        product: &pse_model::generated::runtime::canonical_products::Row,
        stage: &crate::canonical_staging::ClosedStage,
    ) -> Result<String, CanonicalError> {
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
        self.ensure_writes()?;
        if product.problem != selection.revision().problem
            || product.revision != selection.revision().key
            || product.interpretation != wire::INTERPRETATION
        {
            return Err(CanonicalError::Configuration(
                "product selection/interpretation mismatch".into(),
            ));
        }
        if product
            .payload
            .len()
            .saturating_add(product.dependencies.len())
            .saturating_add(product.request.len())
            > PAYLOAD_BYTES
        {
            return Err(CanonicalError::PayloadLimit);
        }
        for attempt in 0..GUARDED_ATTEMPTS {
            self.ensure_writes()?;
            let result = self
                .staging_query(
                    &selection.revision().problem,
                    self.db
                        .query(format!("{PROTECTED_BEGIN}\n{ADMIT_PRODUCT}\nCOMMIT;"))
                        .bind(("problem", selection.revision().problem.clone()))
                        .bind(("revision", selection.revision().key.clone()))
                        .bind(("protection", selection.key().to_owned()))
                        .bind((
                            "sequence",
                            crate::canonical_codec::encode_uint(selection.revision().sequence)?,
                        ))
                        .bind(("product", wire::encode_canonical_products(product)?))
                        .bind(("operation", stage.operation.clone()))
                        .bind((
                            "generation",
                            crate::canonical_codec::encode_uint(stage.generation)?,
                        ))
                        .bind(("request_digest", stage.request_digest.clone()))
                        .bind((
                            "blob",
                            crate::canonical_staging::ProductBlob::decode(
                                product.payload.as_slice(),
                            )?
                            .version,
                        ))
                        .bind((
                            "blob_digest",
                            crate::canonical_staging::ProductBlob::decode(
                                product.payload.as_slice(),
                            )?
                            .digest,
                        ))
                        .bind((
                            "blob_bytes",
                            crate::canonical_codec::encode_uint(
                                crate::canonical_staging::ProductBlob::decode(
                                    product.payload.as_slice(),
                                )?
                                .bytes as u64,
                            )?,
                        )),
                )
                .await
                .and_then(|response| checked(response).map_err(Into::into));
            match result {
                Ok(mut response) => {
                    return Ok(crate::canonical_codec::decode_string(
                        response.take::<Value>(response.num_statements().saturating_sub(2))?,
                    )?);
                }
                Err(error) => {
                    // This immutable acknowledgment remains meaningful even if the
                    // preparation pin expired after the successful commit.
                    if let Ok(Some(key)) = self.product_acknowledged(product).await {
                        return Ok(key);
                    }
                    if matches!(&error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
                        && attempt + 1 < GUARDED_ATTEMPTS
                    {
                        conflict_backoff(attempt).await;
                        continue;
                    }
                    operation_failed("canonical::admit_product", &error);
                    return Err(error);
                }
            }
        }
        Err(CanonicalError::Configuration(
            "product admission retries exhausted".into(),
        ))
        }).await
    }
    pub(crate) async fn product_acknowledged(
        &self,
        product: &pse_model::generated::runtime::canonical_products::Row,
    ) -> Result<Option<String>, CanonicalError> {
        // Admission can reuse a previously rooted exact product rather than create
        // the proposed publication key. Settle both branches after a lost response,
        // without requiring a still-live preparation pin or admitting anything new.
        let mut response = self.protected_query(&product.problem, "canonical::product_acknowledged", || Ok(self.db.query("BEGIN; SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at); LET $saved = SELECT key FROM canonical_products WHERE problem = $problem AND request = $product.request AND payload = $product.payload AND dependencies = $product.dependencies AND producer = $product.producer AND interpretation = $product.interpretation AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).revision = revision AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).owner = key ORDER BY key LIMIT 1; LET $rpc_result = IF array::len($saved) = 0 { NONE } ELSE { $saved[0].key }; fn::pse_execution_v1::deadline($pse_rpc_expires_at); RETURN $rpc_result; COMMIT;")
            .bind(("problem", product.problem.clone())).bind(("product", wire::encode_canonical_products(product)?)))).await?;
        let value = response.take::<Value>(response.num_statements().saturating_sub(2))?;
        if matches!(value, Value::None) {
            return Ok(None);
        }
        Ok(Some(crate::canonical_codec::decode_string(value)?))
    }
    /// End a selection protection through its issuing store and retention conflict guard.
    pub async fn release(&self, selection: &ProtectedSelection) -> Result<(), CanonicalError> {
        let store = selection.owner.store.as_ref().ok_or_else(|| {
            CanonicalError::Configuration("protection issuing store unavailable".into())
        })?;
        store
            .release_protection(selection.revision(), selection.key())
            .await?;
        selection.owner.acknowledge_release();
        Ok(())
    }
    async fn release_protection(
        &self,
        revision: &Revision,
        key: &str,
    ) -> Result<(), CanonicalError> {
        // Quiescing scientific writes must still let in-flight readers drain.
        // A submitted acquisition can outlive its failed local response. Record
        // release even before its row exists so that a later acquisition refuses it.
        let sequence = crate::canonical_codec::encode_uint(revision.sequence)?;
        self.protected_query(&revision.problem, "canonical::release", || Ok(self.db.query(r#"BEGIN;
            SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE;
            fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            LET $pin = SELECT * FROM ONLY type::record('canonical_protections', $protection);
            IF $pin != NONE AND ($pin.key != $protection OR $pin.problem != $problem OR $pin.revision != $revision OR $pin.sequence != $sequence) { THROW 'immutable selection protection identity differs'; };
            UPSERT type::record('canonical_protections', $protection) SET key = $protection, problem = $problem, revision = $revision, sequence = $sequence, expires_at = 0, released = true;
            UPSERT type::record('canonical_guards', 'retention:' + $problem) SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
            fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            COMMIT;"#)
            .bind(("problem", revision.problem.clone())).bind(("revision", revision.key.clone()))
            .bind(("sequence", sequence.clone())).bind(("protection", key.to_owned())))).await?;
        Ok(())
    }
    /// Drain a registered fixture locally; runner disposition owns disposal.
    #[cfg(any(test, feature = "test-support", feature = "canonical-tests"))]
    pub async fn remove_isolated_fixture(&self) -> Result<(), CanonicalError> {
        if !self.database.starts_with("canonical_test_") {
            return Err(CanonicalError::Configuration(
                "fixture removal requires an isolated test database".into(),
            ));
        }
        #[cfg(any(test, feature = "test-support"))]
        let mut removed = match &self.fixture_lifetime {
            Some(lifetime) => Some(lifetime.removed.lock().await),
            None => None,
        };
        #[cfg(any(test, feature = "test-support"))]
        if removed.as_deref().is_some_and(|removed| *removed) {
            return Ok(());
        }
        #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
        self.result_read_drain.drain().await?;
        #[cfg(any(test, feature = "test-support"))]
        if self
            .fixture_lifetime
            .as_ref()
            .is_some_and(|lifetime| Arc::strong_count(lifetime) > 1)
        {
            // Remaining borrowers own the eventual Drop/drain publication.
            return Ok(());
        }
        self.disconnect().await?;
        #[cfg(any(test, feature = "test-support"))]
        if let Some(lifetime) = &self.fixture_lifetime {
            lifetime.drain_connections().await?;
            crate::testing::resource_bridge(
                "drain",
                &serde_json::json!({
                    "resource": lifetime.resource, "state": self.state_path, "database": self.database,
                }),
            )?;
        }
        #[cfg(any(test, feature = "test-support"))]
        if let Some(removed) = removed.as_mut() {
            **removed = true;
        }
        Ok(())
    }
    /// Retain the fixture's runner-owned resource association for all borrowers.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn own_fixture(
        mut self,
        executor: &'static tokio::runtime::Runtime,
        resource: String,
    ) -> Self {
        // The cleanup handle has no lifetime owner, so there is no reference cycle.
        let detached = self.clone();
        self.fixture_lifetime = Some(Arc::new(crate::testing::FixtureLifetime::new(
            detached, executor, resource,
        )));
        self
    }
    #[cfg(all(test, feature = "canonical-tests"))]
    fn replace_fixture_transport(
        &mut self,
        options: &CanonicalOptions,
        budget: std::time::Duration,
    ) -> Result<(), CanonicalError> {
        let lifetime = self.fixture_lifetime.as_ref().ok_or_else(|| {
            CanonicalError::Configuration("fixture transport requires registered owner".into())
        })?;
        let address = url::Url::parse(&options.endpoint)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?
            .socket_addrs(|| None)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?[0];
        let replacement = lifetime.execute(connect_client(address, options, budget, true))?;
        let mut detached = self.clone();
        detached.fixture_lifetime = None;
        detached.db = replacement.clone();
        lifetime.retain_peer(detached)?;
        self.db = replacement;
        Ok(())
    }
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn borrow_fixture(mut self, owner: &Self) -> Result<Self, CanonicalError> {
        self.fixture_lifetime = Some(
            owner
                .fixture_lifetime
                .as_ref()
                .ok_or_else(|| {
                    CanonicalError::Configuration(
                        "fixture borrower requires registered owner".into(),
                    )
                })?
                .clone(),
        );
        Ok(self)
    }
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) fn owns_fixture(&self) -> bool {
        self.fixture_lifetime.is_some()
    }
}

fn initialization_statement() -> String {
    format!(
        "BEGIN;\nIF time::micros() >= $pse_rpc_expires_at {{ THROW 'original schema installation deadline expired'; }};\n{}\nCREATE canonical_interpretations:current SET key = 'current', interpretation = $interpretation, schema_digest = $schema_digest;\nIF time::micros() >= $pse_rpc_expires_at {{ THROW 'original schema installation deadline expired'; }};\nCOMMIT;",
        wire::SCHEMA
    )
}

/// Bound one native client operation, preserving its typed failure.
pub(crate) async fn request<F, T>(future: F) -> Result<T, CanonicalError>
where
    F: IntoFuture<Output = Result<T, surrealdb::Error>>,
{
    within_clock(original_deadline(REQUEST_TIMEOUT), async {
        future.into_future().await.map_err(Into::into)
    })
    .await
}

// Transaction failures also mark preceding statements NotExecuted. Preserve the substantive failure.
/// A single checked RPC with a transport deadline; callers retry complete decisions.
pub(crate) async fn bounded_query<F>(future: F) -> Result<surrealdb::IndexedResults, CanonicalError>
where
    F: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
{
    complete_response(request(future).await?)
}

fn complete_response(
    response: surrealdb::IndexedResults,
) -> Result<surrealdb::IndexedResults, CanonicalError> {
    let response = checked(response)?;
    if response.num_statements() == 0 {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(response)
}

async fn connect_client(
    address: std::net::SocketAddr,
    options: &CanonicalOptions,
    query_timeout: std::time::Duration,
    select_context: bool,
) -> Result<Arc<CanonicalClient>, CanonicalError> {
    let deadline = original_deadline(REQUEST_TIMEOUT);
    within_clock(deadline, async {
        let config = Config::new().bounded_requests().websocket(
            WebsocketConfig::new()
                .max_message_size(MESSAGE_BYTES)
                .read_buffer_size(128 * 1024)
                .write_buffer_size(128 * 1024)
                .max_write_buffer_size(8 * 1024 * 1024),
        )?;
        let db = within_clock(deadline, async {
            Surreal::new::<Ws>((address, config))
                .await
                .map_err(Into::into)
        })
        .await?;
        let client = Arc::new(CanonicalClient::new(db, query_timeout, options));
        if select_context {
            client.select_existing().await?;
        }
        Ok(client)
    })
    .await
}

/// Rebuild a protected immutable read or idempotent lease transaction after a
/// conflict. Every attempt rechecks the live pin and reclamation guard; no
/// scientific decision or partial response is retained across attempts.
pub(crate) async fn protected_query<F, Q>(
    operation: &'static str,
    mut build: F,
) -> Result<surrealdb::IndexedResults, CanonicalError>
where
    F: FnMut() -> Result<Q, CanonicalError>,
    Q: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
{
    within_clock(original_deadline(REQUEST_TIMEOUT), async {
        for attempt in 0..GUARDED_ATTEMPTS {
            match bounded_query(build()?).await {
                Err(CanonicalError::Driver(error))
                    if matches!(
                        error.query_details(),
                        Some(surrealdb::types::QueryError::TransactionConflict)
                    ) && attempt + 1 < GUARDED_ATTEMPTS =>
                {
                    conflict_backoff(attempt).await;
                }
                result => {
                    if let Err(error) = &result {
                        operation_failed(operation, error);
                    }
                    return result;
                }
            }
        }
        Err(CanonicalError::Configuration(
            "guarded operation retries exhausted".into(),
        ))
    })
    .await
}

/// Attribute an escaping failure without altering its typed cause or retry authority.
/// Labels are supplied by callers; bindings and query text are never logged.
pub(crate) fn operation_failed(operation: &'static str, error: &CanonicalError) {
    tracing::warn!(operation, error = %error, "canonical operation failed");
    #[cfg(feature = "canonical-tests")]
    {
        #[allow(
            clippy::print_stderr,
            reason = "explicit canonical qualification captures final operation attribution without requiring a tracing subscriber"
        )]
        {
            eprintln!("canonical operation {operation} failed: {error}");
        }
    }
}

// Short guarded decisions contend on one problem's retention guard. Pacing and
// jitter keep sixteen independent readers from repeatedly colliding; each owner
// still retries only a definite typed rejection and rebuilds its complete decision.
async fn conflict_backoff(attempt: u32) {
    let jitter = u64::from(uuid::Uuid::new_v4().as_bytes()[0] % 64);
    tokio::time::sleep(std::time::Duration::from_millis(
        (10_u64 << attempt.min(4)) + jitter,
    ))
    .await;
}

pub(crate) fn checked(
    mut response: surrealdb::IndexedResults,
) -> Result<surrealdb::IndexedResults, surrealdb::Error> {
    let errors = response.take_errors().into_values().collect::<Vec<_>>();
    if !errors.is_empty() {
        let substantive = errors.iter().position(|error| {
            !matches!(
                error.query_details(),
                Some(
                    surrealdb::types::QueryError::NotExecuted
                        | surrealdb::types::QueryError::Cancelled
                )
            )
        });
        if let Some(error) = errors.into_iter().nth(substantive.unwrap_or(0)) {
            return Err(error);
        }
    }
    Ok(response)
}

// The native indexed route includes BEGIN and COMMIT result slots. Consumer SELECT
// is immediately before COMMIT; do not decode either control's NONE as an object.
pub(crate) const PROTECTED_BEGIN: &str = r#"BEGIN;
SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $pin = SELECT * FROM ONLY type::record('canonical_protections', $protection);
IF $pin = NONE OR $pin.problem != $problem OR $pin.revision != $revision OR $pin.sequence != $sequence OR $pin.released OR $pin.expires_at <= time::micros() { THROW 'immutable selection protection expired'; };"#;

const ADMIT_PRODUCT: &str = r#"
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $version_guard = type::record('canonical_guards', 'version:' + $blob);
SELECT * FROM $version_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $stage = SELECT * FROM ONLY type::record('canonical_stages', $operation);
LET $child = SELECT * FROM ONLY type::record('canonical_staged_edits', $operation + ':0');
LET $manifest = SELECT * FROM ONLY type::record('canonical_version_manifests', $blob);
IF $operation != $product.key OR $stage = NONE OR $stage.problem != $problem OR $stage.request_digest != $request_digest OR $stage.generation != $generation OR $stage.closed = false OR $stage.activated OR $stage.abandoned OR $stage.expires_at <= time::micros() OR $stage.edit_count != 1dec OR $stage.expected_head != NONE { THROW 'closed product staging manifest fenced'; };
IF $child = NONE OR $child.stage != $operation OR $child.ordinal != 0dec OR $child.version != $blob OR $child.logical != 'pse.product-blob.v1' OR $child.scope != 'pse.product-blob.v1' OR $child.name != 'pse.product-blob.v1' OR $manifest = NONE OR $manifest.closed = false OR $manifest.logical != 'pse.product-blob.v1' OR $manifest.kind != 'pse.product-blob.v1' OR $manifest.interpretation != $product.interpretation OR $manifest.reference_count != 0dec OR $manifest.payload_digest != $blob_digest OR $manifest.payload_len != $blob_bytes { THROW 'product blob association invalid'; };
// Repeated preparations retain one already rooted exact description. Publication
// identity is distinct from its request, and a released publication is never revived.
LET $equivalent = SELECT key FROM canonical_products WHERE problem = $problem AND producer = $product.producer AND request = $product.request AND payload = $product.payload AND dependencies = $product.dependencies AND interpretation = $product.interpretation AND type::record('canonical_roots', key).owner_kind = 'product' AND type::record('canonical_roots', key).revision = revision AND type::record('canonical_roots', key).problem = problem AND type::record('canonical_roots', key).owner = key ORDER BY key LIMIT 1;
IF array::len($equivalent) = 0 {
    LET $existing = SELECT * FROM ONLY type::record('canonical_products', $product.key);
    IF $existing = NONE {
        CREATE type::record('canonical_products', $product.key) CONTENT $product;
        CREATE type::record('canonical_roots', $product.key) SET key = $product.key, problem = $problem, revision = $product.revision, sequence = $sequence, owner_kind = 'product', owner = $product.key;
    } ELSE {
        IF $existing.problem != $product.problem OR $existing.request != $product.request OR $existing.payload != $product.payload OR $existing.dependencies != $product.dependencies OR $existing.producer != $product.producer OR $existing.interpretation != $product.interpretation { THROW 'immutable product identity reused'; };
        LET $root = SELECT * FROM ONLY type::record('canonical_roots', $product.key);
        IF $root = NONE { THROW 'product retention was explicitly released'; };
        IF $root.problem != $existing.problem OR $root.revision != $existing.revision OR $root.owner_kind != 'product' OR $root.owner != $existing.key { THROW 'product root disagrees with immutable origin'; };
    };
};
UPDATE type::record('canonical_stages', $operation) SET activated = true;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $version_guard SET key = 'version:' + $blob, generation = (generation ?? 0dec) + 1dec;
UPSERT type::record('canonical_guards', 'retention:' + $problem) SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
LET $rpc_result = IF array::len($equivalent) != 0 { $equivalent[0].key } ELSE { $product.key };
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $pin.expires_at <= time::micros() OR $stage.expires_at <= time::micros() { THROW 'product publication authority expired'; };
RETURN $rpc_result;
"#;

const EDIT: &str = r#"
BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $head_guard = type::record('canonical_guards', 'head:' + $problem);
SELECT * FROM $head_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $ack = SELECT * FROM ONLY type::record('canonical_revisions', $operation);
IF $ack != NONE { THROW 'operation identity already committed; settle immutable revision'; };
LET $stage = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $stage = NONE OR $stage.problem != $problem OR ($stage.expected_head ?? NONE) != $expected OR $stage.request_digest != $request_digest OR $stage.generation != $generation OR $stage.closed = false OR $stage.abandoned OR $stage.activated OR $stage.expires_at <= time::micros() { THROW 'closed staging manifest fenced or unavailable'; };
LET $head = SELECT * FROM ONLY type::record('canonical_problems', $problem);
IF ($head.head ?? NONE) != $expected { THROW 'head compare-and-set conflict'; };
LET $changes = SELECT * FROM canonical_staged_edits WHERE stage = $operation ORDER BY ordinal LIMIT /* EDIT_SCAN_LIMIT */;
IF array::len($changes) != $stage.edit_count OR array::len($changes) > /* EDIT_LIMIT */ { THROW 'closed staging metadata coverage mismatch'; };
LET $sequence = ($head.sequence ?? 0dec) + 1dec;
// Close all displaced intervals before checking the final namespace. Every
// physical page uses exact changed logical IDs; no unchanged source is rewritten.
FOR $batch IN array::clump($changes, 64) {
    LET $logicals = $batch.logical;
    LET $old = SELECT * FROM canonical_memberships WITH INDEX membership_logical WHERE problem = $problem AND logical IN $logicals AND to_sequence = NONE;
    LET $old_guards = array::distinct(array::concat(
        $old.map(|$member| type::string(['pse.scope-guard.v1', $problem, $member.scope])),
        $old.map(|$member| type::string(['pse.name-guard.v1', $problem, $member.scope, $member.name]))
    )).map(|$key| { id: type::record('canonical_guards', $key), key: $key, generation: 1dec });
    LET $existing = SELECT id FROM $old_guards.id;
    UPDATE $existing.id SET generation = generation + 1dec RETURN NONE;
    LET $existing_ids = $existing.id;
    LET $missing = $old_guards.filter(|$guard| $guard.id NOT IN $existing_ids);
    INSERT INTO canonical_guards $missing RETURN NONE;
    UPDATE $old.id SET to_sequence = $sequence RETURN NONE;
};
LET $scope_guards = array::distinct($changes.scope).map(|$scope| {
    LET $key = type::string(['pse.scope-guard.v1', $problem, $scope]);
    { id: type::record('canonical_guards', $key), key: $key, generation: 1dec }
});
FOR $guards IN array::clump($scope_guards, 64) {
    LET $existing = SELECT id FROM $guards.id;
    UPDATE $existing.id SET generation = generation + 1dec RETURN NONE;
    LET $existing_ids = $existing.id;
    LET $missing = $guards.filter(|$guard| $guard.id NOT IN $existing_ids);
    INSERT INTO canonical_guards $missing RETURN NONE;
};
LET $incoming = $changes.filter(|$change| $change.version != NONE);
LET $names = $incoming.map(|$change| type::string([$change.scope, $change.name]));
IF array::len(array::distinct($names)) != array::len($names) { THROW 'name already exists within incoming revision'; };
FOR $batch IN array::clump($changes, 64) {
    LET $creates = $batch.filter(|$change| $change.version != NONE);
    // Read exact guard IDs, update existing records, and insert absent records.
    // Released 3.3 UPSERT is create-first even for existing targets; recovering
    // that duplicate rebuilds the growing RocksDB transaction write index.
    // Every guard is actually written, so a concurrent premise writer conflicts
    // even for a previously absent record. Head/stage/retention decision locks
    // above still fence the complete transaction and its fresh retries.
    LET $guard_keys = array::distinct(array::concat(
        $batch.map(|$change| type::string(['pse.name-guard.v1', $problem, $change.scope, $change.name])),
        $creates.map(|$change| 'version:' + $change.version)
    ));
    LET $guard_rows = $guard_keys.map(|$key| { id: type::record('canonical_guards', $key), key: $key, generation: 1dec });
    LET $existing = SELECT id FROM $guard_rows.id;
    UPDATE $existing.id SET generation = generation + 1dec RETURN NONE;
    LET $existing_ids = $existing.id;
    LET $missing = $guard_rows.filter(|$guard| $guard.id NOT IN $existing_ids);
    INSERT INTO canonical_guards $missing RETURN NONE;
    LET $manifest_records = $creates.map(|$change| type::record('canonical_version_manifests', $change.version));
    LET $manifests = SELECT * FROM $manifest_records;
    IF array::len($manifests) != array::len($creates) { THROW 'staged immutable source manifest coverage invalid'; };
    FOR $change IN $creates {
        LET $manifest = $manifests.find(|$header| $header.key = $change.version);
        IF $manifest = NONE OR $manifest.closed = false OR $manifest.logical != $change.logical OR $manifest.interpretation != $interpretation { THROW 'staged immutable source manifest invalid'; };
    };
    // One native name selection per demanded namespace/page, with scalar
    // bindings so released 3.3 selects the compound namespace index. Scientific
    // logical identity is checked against the exact incoming name afterwards.
    FOR $scope IN array::distinct($creates.scope) {
        LET $scope_changes = $creates.filter(|$change| $change.scope = $scope);
        LET $names = $scope_changes.name;
        LET $survivors = SELECT name, logical FROM canonical_memberships WITH INDEX membership_selection WHERE problem = $problem AND scope = $scope AND name IN $names AND to_sequence = NONE;
        FOR $survivor IN $survivors {
            LET $replacement = $scope_changes.find(|$change| $change.name = $survivor.name);
            IF $replacement = NONE OR $survivor.logical != $replacement.logical { THROW 'name already exists'; };
        };
    };
    LET $memberships = $creates.map(|$change| {
        LET $key = type::string(['pse.membership.v1', $operation, $change.logical]);
        { id: type::record('canonical_memberships', $key), key: $key,
          in: type::record('canonical_problems', $problem), out: type::record('canonical_version_manifests', $change.version),
          problem: $problem, scope: $change.scope, name: $change.name, logical: $change.logical,
          version: $change.version, from_sequence: $sequence }
    });
    INSERT RELATION INTO canonical_memberships $memberships RETURN NONE;
};
CREATE type::record('canonical_revisions', $operation) SET key = $operation, problem = $problem, sequence = $sequence, parent = $expected, operation = $operation, request = $request, interpretation = $interpretation;
CREATE type::record('canonical_roots', 'history:' + $operation) SET key = 'history:' + $operation, problem = $problem, revision = $operation, sequence = $sequence, owner_kind = 'history', owner = $operation;
UPDATE type::record('canonical_stages', $operation) SET activated = true;
UPSERT type::record('canonical_problems', $problem) SET key = $problem, head = $operation, sequence = $sequence;
UPSERT $head_guard SET key = 'head:' + $problem, generation = (generation ?? 0dec) + 1dec;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $retention_guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec, incarnation = incarnation ?? <string>rand::uuid::v4();
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $stage.expires_at <= time::micros() { THROW 'activation staging lease expired before commit'; };
COMMIT;
"#;

#[cfg(test)]
mod canonical_admission_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "isolated admission controls require exact filesystem outcomes"
    )]
    use super::*;

    struct Scope(PathBuf);
    impl Drop for Scope {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn scope() -> (Scope, serde_json::Value) {
        let scope =
            Scope(std::env::temp_dir().join(format!("pse-admission-{}", uuid::Uuid::new_v4())));
        std::fs::create_dir(&scope.0).unwrap();
        let process = std::fs::read_to_string("/proc/self/stat").unwrap();
        let start = process
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap();
        let owner = serde_json::json!({
            "nonce": "0123456789abcdef0123456789abcdef", "pid": std::process::id(),
            "start": start, "boot": std::fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap().trim(),
        });
        std::fs::write(
            scope.0.join("lifecycle-owner.json"),
            serde_json::to_vec(&owner).unwrap(),
        )
        .unwrap();
        (scope, owner)
    }
    fn admission(owner: serde_json::Value, accepting_writes: bool) -> WriteAdmission {
        serde_json::from_value(serde_json::json!({
            "namespace": "fresh", "database": "current", "accepting_writes": accepting_writes,
            "derived_rebuild_pending": {"initializer": owner},
        }))
        .unwrap()
    }

    #[test]
    fn rebuild_initializer_borrows_only_exact_closed_live_owner_and_target() {
        let (scope, owner) = scope();
        let nonce = owner["nonce"].as_str().unwrap();
        let selected = admission(owner.clone(), false);
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "current", Some(nonce))
                .is_ok()
        );
        for (namespace, database, nonce) in [
            ("old", "current", Some(nonce)),
            ("fresh", "old", Some(nonce)),
            ("fresh", "current", None),
            ("fresh", "current", Some("wrong")),
        ] {
            assert!(matches!(
                selected.require_writes(&scope.0, namespace, database, nonce),
                Err(CanonicalError::Quiesced)
            ));
        }
        // A persisted true override cannot make an ordinary handle eligible.
        let override_true = admission(owner.clone(), true);
        assert!(matches!(
            override_true.require_writes(&scope.0, "fresh", "current", None),
            Err(CanonicalError::Quiesced)
        ));
        assert!(matches!(
            override_true.require_writes(&scope.0, "fresh", "current", Some(nonce)),
            Err(CanonicalError::Quiesced)
        ));
    }

    #[test]
    fn initializer_authority_expires_on_owner_removal_replacement_or_process_change() {
        let (scope, owner) = scope();
        let nonce = owner["nonce"].as_str().unwrap();
        let selected = admission(owner.clone(), false);
        std::fs::remove_file(scope.0.join("lifecycle-owner.json")).unwrap();
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "current", Some(nonce))
                .is_err()
        );
        let mut replaced = owner.clone();
        replaced["nonce"] = "fedcba9876543210fedcba9876543210".into();
        std::fs::write(
            scope.0.join("lifecycle-owner.json"),
            serde_json::to_vec(&replaced).unwrap(),
        )
        .unwrap();
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "current", Some(nonce))
                .is_err()
        );
        for (field, changed) in [
            ("pid", serde_json::json!(u32::MAX)),
            ("start", serde_json::json!("reused-pid")),
            ("boot", serde_json::json!("previous-boot")),
        ] {
            let mut stale = owner.clone();
            stale[field] = changed;
            std::fs::write(
                scope.0.join("lifecycle-owner.json"),
                serde_json::to_vec(&stale).unwrap(),
            )
            .unwrap();
            assert!(
                admission(stale, false)
                    .require_writes(&scope.0, "fresh", "current", Some(nonce))
                    .is_err()
            );
        }
    }

    #[test]
    fn initializer_authority_expires_for_an_unreaped_dead_owner() {
        let (scope, mut owner) = scope();
        let mut child = std::process::Command::new("/bin/sh")
            .args(["-c", "exit 0"])
            .spawn()
            .unwrap();
        let observed = (|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                let process = std::fs::read_to_string(format!("/proc/{}/stat", child.id()))?;
                let fields = process
                    .rsplit_once(')')
                    .unwrap()
                    .1
                    .split_whitespace()
                    .collect::<Vec<_>>();
                if fields.first() == Some(&"Z") {
                    return Ok::<_, std::io::Error>(fields[19].to_owned());
                }
                if std::time::Instant::now() >= deadline {
                    return Err(std::io::Error::other(
                        "child did not reach an unreaped zombie state",
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })();
        let refused = observed.map(|start| {
            owner["pid"] = child.id().into();
            owner["start"] = start.into();
            std::fs::write(
                scope.0.join("lifecycle-owner.json"),
                serde_json::to_vec(&owner).unwrap(),
            )
            .unwrap();
            matches!(
                admission(owner.clone(), false).require_writes(
                    &scope.0,
                    "fresh",
                    "current",
                    owner["nonce"].as_str()
                ),
                Err(CanonicalError::Quiesced)
            )
        });
        let status = child.wait().unwrap();
        assert!(status.success());
        assert!(refused.unwrap());
    }

    #[test]
    fn restore_pending_has_no_initializer_authority_and_old_namespace_stays_closed() {
        let (scope, owner) = scope();
        let nonce = owner["nonce"].as_str().unwrap();
        let restore: WriteAdmission = serde_json::from_value(serde_json::json!({
            "namespace": "fresh", "database": "current", "accepting_writes": false,
            "derived_rebuild_pending": {"kind": "current-restore"},
        }))
        .unwrap();
        assert!(
            restore
                .require_writes(&scope.0, "fresh", "current", Some(nonce))
                .is_err()
        );
        for accepting in [false, true] {
            let ordinary: WriteAdmission = serde_json::from_value(serde_json::json!({
                "namespace": "fresh", "database": "current", "accepting_writes": accepting,
            }))
            .unwrap();
            assert_eq!(
                ordinary
                    .require_writes(&scope.0, "fresh", "isolated-fixture", None)
                    .is_ok(),
                accepting
            );
            assert!(
                ordinary
                    .require_writes(&scope.0, "old", "current", Some(nonce))
                    .is_err()
            );
        }
    }
}

#[cfg(test)]
mod staging_turn_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "local ownership controls fail on unexpected admission errors"
    )]
    use super::*;

    #[tokio::test]
    async fn staging_turns_share_one_problem_and_release_cancelled_waiters() {
        let turns = StagingTurns::default();
        let gate = turns.gate("problem").unwrap();
        assert!(Arc::ptr_eq(&gate, &turns.gate("problem").unwrap()));
        let held = gate.clone().lock_owned().await;
        let mut cancelled = Box::pin(gate.clone().lock_owned());
        std::future::poll_fn(|context| {
            assert!(cancelled.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        let mut following = Box::pin(gate.clone().lock_owned());
        std::future::poll_fn(|context| {
            assert!(following.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        let independent = turns.gate("independent").unwrap();
        let independent_held = independent.clone().try_lock_owned().unwrap();
        drop(cancelled);
        drop(held);
        let following = tokio::time::timeout(std::time::Duration::from_secs(1), following)
            .await
            .unwrap();
        drop(following);
        drop(gate);
        drop(independent_held);
        drop(independent);
        let _next = turns.gate("next").unwrap();
        assert_eq!(
            turns.problems.lock().unwrap().len(),
            1,
            "dead problem keys must not accumulate in the local owner"
        );
    }

    #[tokio::test]
    async fn protected_turn_queue_preserves_deadline_and_does_not_start_expired_work() {
        let turns = StagingTurns::default();
        let gate = turns.gate("problem").unwrap();
        let held = gate.clone().lock_owned().await;
        let started = std::sync::atomic::AtomicBool::new(false);
        let result = turns
            .run(
                "problem",
                tokio::time::Instant::now() + std::time::Duration::from_millis(1),
                async {
                    started.store(true, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                },
            )
            .await;
        assert!(matches!(result, Err(CanonicalError::Timeout)));
        assert!(!started.load(std::sync::atomic::Ordering::Relaxed));
        drop(held);
        let result = turns
            .run("problem", tokio::time::Instant::now(), async {
                started.store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            })
            .await;
        assert!(matches!(result, Err(CanonicalError::Timeout)));
        assert!(!started.load(std::sync::atomic::Ordering::Relaxed));
        turns
            .run(
                "problem",
                tokio::time::Instant::now() + std::time::Duration::from_secs(1),
                async { Ok(()) },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn protected_turn_cancelled_waiter_releases_queue_without_blocking_other_problems() {
        let turns = StagingTurns::default();
        let gate = turns.gate("problem").unwrap();
        let held = gate.clone().lock_owned().await;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let cancelled_started = std::sync::atomic::AtomicBool::new(false);
        let mut cancelled = Box::pin(turns.run("problem", deadline, async {
            cancelled_started.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        }));
        std::future::poll_fn(|context| {
            assert!(cancelled.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        let mut following = Box::pin(turns.run("problem", deadline, async { Ok(()) }));
        std::future::poll_fn(|context| {
            assert!(following.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        turns
            .run("independent", deadline, async { Ok(()) })
            .await
            .unwrap();
        drop(cancelled);
        drop(held);
        following.await.unwrap();
        assert!(!cancelled_started.load(std::sync::atomic::Ordering::Relaxed));
        assert!(gate.try_lock_owned().is_ok());
    }
}

#[cfg(test)]
mod description_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "pure description identity assertions fail on unexpected encoding errors"
    )]
    use super::*;
    use crate::canonical_selection::{ProductDescription, SelectedRead};
    use pse_model::generated::runtime::canonical_products::Row as Product;

    fn read(revision: &str, protection: &str) -> SelectedRead {
        let mut read = SelectedRead::new(ProtectedSelection {
            owner: Arc::new(ProtectionOwner {
                store: None,
                executor: None,
                released: std::sync::atomic::AtomicBool::new(false),
                key: protection.into(),
                revision: Revision {
                    key: revision.into(),
                    problem: "problem".into(),
                    sequence: 1,
                    parent: None,
                    operation: revision.into(),
                    request: Vec::new().into(),
                    interpretation: wire::INTERPRETATION.into(),
                },
            }),
        });
        read.interpretation("physical".into(), "provider-v1".into())
            .unwrap();
        read
    }
    fn product(read: &SelectedRead, key: &str, payload: &[u8]) -> Product {
        Product {
            key: key.into(),
            problem: "problem".into(),
            revision: read.selection().revision().key.clone(),
            request: b"request".to_vec().into(),
            payload: payload.to_vec().into(),
            // Preparation always takes complete actual premises rather than this caller field.
            dependencies: b"untrusted caller dependencies".to_vec().into(),
            producer: "producer".into(),
            interpretation: wire::INTERPRETATION.into(),
        }
    }
    #[test]
    fn description_encoding_reuses_exact_material_across_selection_attribution() {
        let first = read("revision-one", "pin-one");
        let second = read("revision-two", "pin-two");
        let description =
            ProductDescription::prepare(&first, product(&first, "offer-one", b"body")).unwrap();
        let separately_prepared =
            ProductDescription::prepare(&second, product(&second, "offer-two", b"body")).unwrap();
        assert_eq!(description.descriptor(), separately_prepared.descriptor());
        assert_eq!(
            description.selected_dependencies(),
            separately_prepared.selected_dependencies()
        );
        assert!(description.matches_read(&second).unwrap());
        let shared = description.clone();
        assert_eq!(
            description.descriptor().as_ptr(),
            shared.descriptor().as_ptr(),
            "retained descriptions share owned encoded material"
        );
        let changed_body =
            ProductDescription::prepare(&first, product(&first, "offer-one", b"other body"))
                .unwrap();
        assert_ne!(description.descriptor(), changed_body.descriptor());
        let mut changed_dependencies = read("revision-three", "pin-three");
        changed_dependencies
            .interpretation("demand".into(), "derivatives".into())
            .unwrap();
        assert!(!description.matches_read(&changed_dependencies).unwrap());
        let changed = ProductDescription::prepare(
            &changed_dependencies,
            product(&changed_dependencies, "offer-one", b"body"),
        )
        .unwrap();
        assert_ne!(description.descriptor(), changed.descriptor());
        assert!(
            description.retained_bytes()
                >= description.descriptor().len()
                    + description.selected_dependencies().retained_bytes()
        );
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        reason = "isolated server fixtures and assertions fail the test on unexpected results"
    )]
    use super::*;
    use surrealdb::types::Number;
    fn version(key: &str, logical: &str, payload: &[u8]) -> ObjectVersion {
        ObjectVersion {
            key: key.into(),
            logical: logical.into(),
            kind: "test".into(),
            payload: payload.to_vec().into(),
            interpretation: wire::INTERPRETATION.into(),
        }
    }
    async fn initialization_fixture() -> (CanonicalStore, CanonicalOptions) {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_init_{}", uuid::Uuid::new_v4().simple());
        (
            crate::testing::canonical_fixture_with_options(&options, false).unwrap(),
            options,
        )
    }
    #[tokio::test]
    async fn source_protection_follows_last_reader_and_abandoned_acquisition() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit(
                "protected-problem",
                None,
                "initial",
                &[ObjectEdit {
                    logical: "x".into(),
                    scope: "root".into(),
                    name: "x".into(),
                    version: Some(version("x-version", "x", &[1, 2, 3])),
                    references: vec![],
                }],
            )
            .await
            .unwrap();
        let first = store
            .protect(revision.clone(), std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let follower = first.clone();
        first.finish().await.unwrap();
        assert_eq!(
            store
                .select_names(&follower, "root", &["x".into()])
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(store.result_read_drain.live(), 1);
        drop(follower);
        store.result_read_drain.drain().await.unwrap();

        // Hold local pacing so submission is known to own its token before the
        // caller disappears. The completion task must still acquire and release it.
        let gate = store.staging_turns.gate(&revision.problem).unwrap();
        let held = gate.lock_owned().await;
        let pending_store = store.clone();
        let pending = tokio::spawn(async move {
            pending_store
                .protect(revision, std::time::Duration::from_secs(30))
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while store.result_read_drain.live() == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        pending.abort();
        assert!(pending.await.unwrap_err().is_cancelled());
        drop(held);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while store.result_read_drain.live() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        store.result_read_drain.drain().await.unwrap();
        let mut response = bounded_query(store.db.query(
            "SELECT released FROM canonical_protections WHERE problem = 'protected-problem';",
        ))
        .await
        .unwrap();
        let rows: Vec<Object> = response.take(0).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(
            rows.into_iter()
                .all(|mut row| row.remove("released") == Some(Value::Bool(true)))
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn source_protection_concurrent_finish_awaits_final_release() {
        const FOLLOWERS: usize = 16;
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit(
                "concurrent-protection",
                None,
                "initial",
                &[ObjectEdit {
                    logical: "x".into(),
                    scope: "root".into(),
                    name: "x".into(),
                    version: Some(version("x-version", "x", &[1, 2, 3])),
                    references: vec![],
                }],
            )
            .await
            .unwrap();
        let selection = store
            .protect(revision.clone(), std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let key = selection.key().to_owned();
        let mut followers = (1..FOLLOWERS)
            .map(|_| selection.clone())
            .collect::<Vec<_>>();
        followers.push(selection);
        let held = store
            .staging_turns
            .gate(&revision.problem)
            .unwrap()
            .lock_owned()
            .await;
        let barrier = Arc::new(tokio::sync::Barrier::new(FOLLOWERS));
        let mut finishes = tokio::task::JoinSet::new();
        for follower in followers {
            let barrier = barrier.clone();
            finishes.spawn(async move {
                barrier.wait().await;
                follower.finish().await
            });
        }
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            for _ in 1..FOLLOWERS {
                finishes.join_next().await.unwrap().unwrap().unwrap();
            }
        })
        .await
        .unwrap();
        assert_eq!(finishes.len(), 1);
        assert!(
            finishes.try_join_next().is_none(),
            "the final finish must await release acknowledgment"
        );
        assert_eq!(
            store.result_read_drain.live(),
            1,
            "cleanup retains its owner until acknowledgment"
        );
        let mut response = bounded_query(
            store
                .db
                .query(
                    "SELECT VALUE released FROM ONLY type::record('canonical_protections', $key);",
                )
                .bind(("key", key.clone())),
        )
        .await
        .unwrap();
        assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(false));
        drop(held);
        tokio::time::timeout(std::time::Duration::from_secs(5), finishes.join_next())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .unwrap();
        store.result_read_drain.drain().await.unwrap();
        let mut response = bounded_query(
            store
                .db
                .query(
                    "SELECT VALUE released FROM ONLY type::record('canonical_protections', $key);",
                )
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn source_protection_release_uses_issuing_store_through_foreign_handle() {
        let (store, options) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit(
                "issuing-protection",
                None,
                "initial",
                &[ObjectEdit {
                    logical: "x".into(),
                    scope: "root".into(),
                    name: "x".into(),
                    version: Some(version("x-version", "x", &[1, 2, 3])),
                    references: vec![],
                }],
            )
            .await
            .unwrap();
        let (foreign, _) = initialization_fixture().await;
        foreign.create().await.unwrap();
        let selection = store
            .protect(revision.clone(), std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let key = selection.key().to_owned();
        foreign.release(&selection).await.unwrap();
        let mut response = bounded_query(
            store
                .db
                .query(
                    "SELECT VALUE released FROM ONLY type::record('canonical_protections', $key);",
                )
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
        assert!(
            store
                .select_names(&selection, "root", &["x".into()])
                .await
                .is_err()
        );
        assert_eq!(store.result_read_drain.live(), 0);
        drop(selection);
        store.result_read_drain.drain().await.unwrap();
        // A separately connected handle for the same database remains supported.
        let peer = crate::testing::canonical_fixture_peer(&store, &options).unwrap();
        let selection = store
            .protect(revision, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        peer.release(&selection).await.unwrap();
        assert!(
            store
                .select_names(&selection, "root", &["x".into()])
                .await
                .is_err()
        );
        drop(selection);
        store.result_read_drain.drain().await.unwrap();
        foreign.remove_isolated_fixture().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn source_protection_release_before_acquire_refuses_late_admission() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit(
                "late-protection",
                None,
                "initial",
                &[ObjectEdit {
                    logical: "x".into(),
                    scope: "root".into(),
                    name: "x".into(),
                    version: Some(version("x-version", "x", &[1, 2, 3])),
                    references: vec![],
                }],
            )
            .await
            .unwrap();
        // Control the server ordering directly: cleanup acknowledges this exact
        // owner before its delayed acquisition reaches the guarded transaction.
        store.result_read_drain.retain();
        let key = uuid::Uuid::new_v4().to_string();
        let selection = ProtectedSelection {
            owner: Arc::new(ProtectionOwner {
                key: key.clone(),
                revision: revision.clone(),
                store: Some(store.clone()),
                executor: Some(tokio::runtime::Handle::current()),
                released: std::sync::atomic::AtomicBool::new(false),
            }),
        };
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY type::record('canonical_protections', $key);")
                .bind(("key", key.clone())),
        )
        .await
        .unwrap();
        assert!(response.take::<Option<Object>>(0).unwrap().is_none());
        store.release(&selection).await.unwrap();
        assert_eq!(store.result_read_drain.live(), 0);
        store.result_read_drain.drain().await.unwrap();
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY type::record('canonical_protections', $key);")
                .bind(("key", key.clone())),
        )
        .await
        .unwrap();
        let released = wire::decode_canonical_protections(
            response.take::<Option<Object>>(0).unwrap().unwrap(),
        )
        .unwrap();
        assert_eq!(released.key, key);
        assert_eq!(released.problem, revision.problem);
        assert_eq!(released.revision, revision.key);
        assert_eq!(released.sequence, revision.sequence);
        assert!(released.released);
        assert_eq!(released.expires_at, 0);
        // This is the actual acquisition path with a fresh, unexpired RPC clock,
        // so refusal depends on release bookkeeping rather than deadline expiry.
        assert!(
            store
                .acquire_protection(selection, 30_000_000)
                .await
                .is_err()
        );
        let mut response = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY type::record('canonical_protections', $key);")
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert_eq!(
            wire::decode_canonical_protections(
                response.take::<Option<Object>>(0).unwrap().unwrap()
            )
            .unwrap(),
            released
        );
        store.result_read_drain.drain().await.unwrap();

        // The opposite ordering remains live until release, and mismatched
        // captured metadata cannot revoke that valid protection.
        let selection = store
            .protect(revision.clone(), std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let mut wrong = revision;
        wrong.sequence = wrong.sequence.checked_add(1).unwrap();
        assert!(
            store
                .release_protection(&wrong, selection.key())
                .await
                .is_err()
        );
        assert_eq!(
            store
                .select_names(&selection, "root", &["x".into()])
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(store.result_read_drain.live(), 1);
        selection.finish().await.unwrap();
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn source_protection_release_timeout_retains_cleanup_authority() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit(
                "release-timeout",
                None,
                "initial",
                &[ObjectEdit {
                    logical: "x".into(),
                    scope: "root".into(),
                    name: "x".into(),
                    version: Some(version("x-version", "x", &[1, 2, 3])),
                    references: vec![],
                }],
            )
            .await
            .unwrap();
        let selection = store
            .protect(revision.clone(), std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let held = store
            .staging_turns
            .gate(&revision.problem)
            .unwrap()
            .lock_owned()
            .await;
        let release = within_clock(
            tokio::time::Instant::now() + std::time::Duration::from_millis(25),
            store.release(&selection),
        )
        .await;
        assert!(matches!(release, Err(CanonicalError::Timeout)));
        assert!(
            !selection
                .owner
                .released
                .load(std::sync::atomic::Ordering::Acquire)
        );
        assert_eq!(store.result_read_drain.live(), 1);
        drop(held);
        assert_eq!(
            store
                .select_names(&selection, "root", &["x".into()])
                .await
                .unwrap()
                .len(),
            1
        );
        // Failed explicit release must leave the final-drop backstop intact.
        drop(selection);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn connection_and_open_never_provision_unknown_database() {
        let (store, _) = initialization_fixture().await;
        assert!(!store.database_exists().await.unwrap());
        assert!(store.open().await.is_err());
        assert!(!store.database_exists().await.unwrap());
        store.create().await.unwrap();
        assert!(store.database_exists().await.unwrap());
        store.open().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    #[allow(
        clippy::print_stderr,
        reason = "bounded explicit transport qualification emits only finite case counts and typed failures, never bindings or payload"
    )]
    async fn actual_sdk_stream_query_serial_and_sixteen_concurrent_readonly_control()
    -> Result<(), CanonicalError> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use surrealdb::types::Bytes;
        use tokio::time::Instant;

        #[allow(
            clippy::result_large_err,
            reason = "test-only SDK control retains the exact native error and failing operation without altering transport allocations"
        )]
        async fn echo(
            store: &CanonicalStore,
            payload: &[u8],
            deadline: Instant,
        ) -> Result<(), (&'static str, CanonicalError)> {
            let deadline = deadline.min(Instant::now() + REQUEST_TIMEOUT);
            tokio::time::timeout_at(deadline, async {
                // Exercise the actual SDK indexed-query StreamQuery/End path.
                // There is no repository retry helper or replacement transport.
                let mut response = store
                    .db
                    .query("RETURN $payload;")
                    .bind(("payload", Bytes::from(payload.to_vec())))
                    .await
                    .map_err(|error| ("query.await", CanonicalError::from(error)))?
                    .check()
                    .map_err(|error| ("response.check", CanonicalError::from(error)))?;
                if response.num_statements() != 1 {
                    return Err(("statement_count", CanonicalError::IncompleteResponse));
                }
                let returned = response
                    .take::<Value>(0)
                    .map_err(|error| ("response.take", CanonicalError::from(error)))?;
                match returned {
                    Value::Bytes(bytes) if &bytes[..] == payload => Ok(()),
                    _ => Err((
                        "exact_payload",
                        CanonicalError::Configuration("readonly SDK echo differs".into()),
                    )),
                }
            })
            .await
            .map_err(|_| ("original_request_deadline", CanonicalError::Timeout))?
        }

        let started = Instant::now();
        let deadline = started + std::time::Duration::from_secs(120);
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("explicit native transport fixture required");
        let mut options = CanonicalOptions::from_state(Path::new(&state))?;
        options.database = format!("canonical_test_transport_{}", uuid::Uuid::new_v4().simple());
        // TEST ONLY: an explicit bounded opaque loopback relay may forward the
        // configured actual server. Ordinary native controls keep their fixture route.
        if let Ok(endpoint) = std::env::var("PSE_CANONICAL_TEST_ENDPOINT") {
            if endpoint != "ws://127.0.0.1:18089" {
                return Err(CanonicalError::Configuration(
                    "transport fixture relay must be the assigned loopback endpoint".into(),
                ));
            }
            options.endpoint = endpoint;
        }
        let store = crate::testing::canonical_fixture_with_options(&options, true)?;
        let mut cases = Vec::new();
        let mut original_failure = None;
        for (case, bytes, lanes) in [
            ("serial-small", 64, 1),
            ("serial-inventory", 256 * 1024, 1),
            ("sixteen-small", 64, 16),
            ("sixteen-inventory", 256 * 1024, 16),
        ] {
            let case_started = Instant::now();
            // Existing UUID randomness avoids making the inventory control a tiny
            // compressed message; only bounded lengths/counts are reported.
            let mut material = Vec::with_capacity(bytes);
            while material.len() < bytes {
                let block = uuid::Uuid::new_v4();
                let count = (bytes - material.len()).min(block.as_bytes().len());
                material.extend_from_slice(&block.as_bytes()[..count]);
            }
            let payload = Arc::new(material);
            let attempted = Arc::new(AtomicUsize::new(0));
            let completed = Arc::new(AtomicUsize::new(0));
            let barrier = Arc::new(tokio::sync::Barrier::new(lanes));
            let mut jobs = tokio::task::JoinSet::new();
            for lane in 0..lanes {
                let store = store.clone();
                let payload = payload.clone();
                let attempted = attempted.clone();
                let completed = completed.clone();
                let barrier = barrier.clone();
                jobs.spawn(async move {
                    barrier.wait().await;
                    for iteration in 0..1024 / lanes {
                        attempted.fetch_add(1, Ordering::Relaxed);
                        if let Err((operation, error)) = echo(&store, &payload, deadline).await {
                            return Err((lane, iteration, operation, error));
                        }
                        completed.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok(())
                });
            }
            // Drain every already-issued bounded read before cleanup. A failed lane
            // is never retried, and the first fully typed native cause is returned.
            while let Some(result) = jobs.join_next().await {
                if let Err((lane, iteration, operation, error)) = result.unwrap() {
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "probe": "actual-sdk-stream-query", "case": case,
                            "lane": lane, "iteration": iteration, "operation": operation,
                            "error_type": format!("{error:?}"), "error": error.to_string(),
                        })
                    );
                    if original_failure.is_none() {
                        original_failure = Some(error);
                    }
                }
            }
            let report = serde_json::json!({
                "probe": "actual-sdk-stream-query", "case": case, "lanes": lanes,
                "expected_rpc": 1024, "attempted_rpc": attempted.load(Ordering::Relaxed),
                "completed_rpc": completed.load(Ordering::Relaxed), "payload_bytes_per_rpc": bytes,
                "completed_payload_bytes_each_direction": completed.load(Ordering::Relaxed) * bytes,
                "elapsed_ms": case_started.elapsed().as_millis(), "passed": original_failure.is_none(),
            });
            eprintln!("{report}");
            cases.push(report);
            if original_failure.is_some() {
                break;
            }
            assert_eq!(completed.load(Ordering::Relaxed), 1024);
        }
        eprintln!(
            "{}",
            serde_json::json!({
                "probe": "actual-sdk-stream-query", "fixture_database": options.database,
                "expected_rpc": 4096, "deadline_seconds": 120, "elapsed_ms": started.elapsed().as_millis(),
                "cases": cases, "passed": original_failure.is_none(),
            })
        );
        let cleanup = store.remove_isolated_fixture().await;
        if let Some(error) = original_failure {
            return Err(error);
        }
        cleanup?;
        assert_eq!(cases.len(), 4);
        Ok(())
    }

    fn rejected_initialization() -> CanonicalError {
        surrealdb::Error::query(
            "definitely rejected installation".into(),
            surrealdb::types::QueryError::TransactionConflict,
        )
        .into()
    }
    #[tokio::test]
    async fn protected_decision_retries_only_definite_conflicts_and_preserves_final_error() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        let mut response = protected_query("canonical::protected_decision_retries_only_definite_conflicts_and_preserves_final_error", || {
            let attempt = attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let store = &store;
            Ok(async move {
                if attempt < 2 {
                    Err(surrealdb::Error::query(
                        "definite protection rejection".into(),
                        surrealdb::types::QueryError::TransactionConflict,
                    ))
                } else {
                    store.db.query("RETURN 'complete decision';").await
                }
            })
        })
        .await
        .unwrap();
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3);
        assert_eq!(
            crate::canonical_codec::decode_string(response.take::<Value>(0).unwrap()).unwrap(),
            "complete decision"
        );
        attempts.store(0, std::sync::atomic::Ordering::SeqCst);
        let result = protected_query("canonical::protected_decision_retries_only_definite_conflicts_and_preserves_final_error", || {
            attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(async {
                Err(surrealdb::Error::query(
                    "final definite rejection".into(),
                    surrealdb::types::QueryError::TransactionConflict,
                ))
            })
        })
        .await;
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            GUARDED_ATTEMPTS as usize
        );
        assert!(
            matches!(result, Err(CanonicalError::Driver(error)) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
        );
        for kind in [
            surrealdb::types::QueryError::Cancelled,
            surrealdb::types::QueryError::NotExecuted,
        ] {
            attempts.store(0, std::sync::atomic::Ordering::SeqCst);
            let result = protected_query("canonical::protected_decision_retries_only_definite_conflicts_and_preserves_final_error", || {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let kind = kind.clone();
                Ok(async move { Err(surrealdb::Error::query("uncertain protection".into(), kind)) })
            })
            .await;
            assert!(result.is_err());
            assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn initialization_rejected_transaction_rebuilds_complete_schema() {
        let (store, _) = initialization_fixture().await;
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        store
            .initialize_with(|| async {
                if attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst) < 2 {
                    Err(rejected_initialization())
                } else {
                    store.submit_initialization().await
                }
            })
            .await
            .unwrap();
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3);
        store.open().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn initialization_retries_rejected_atomic_installation_and_preserves_inventory() {
        let (store, _) = initialization_fixture().await;
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        store
            .initialize_with(|| async {
                let attempt = attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt == 0 {
                    assert!(store.initialization_required().await?);
                    // A separate initializer commits while this transaction is
                    // definitively rejected. The next inventory must recognize it.
                    complete_response(store.submit_initialization().await?)?;
                    Err(rejected_initialization())
                } else {
                    panic!("fresh inventory must recognize another initializer")
                }
            })
            .await
            .unwrap();
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 1);
        store.open().await.unwrap();
        // A different creator's acknowledged installation is recognized by the
        // next inventory; this creator must not submit another schema transaction.
        store
            .initialize_with(|| async { panic!("installed schema must not be replayed") })
            .await
            .unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn initialization_exhaustion_preserves_typed_rejection_and_uncertainty_is_not_retried() {
        let (store, _) = initialization_fixture().await;
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        let result = store
            .initialize_with(|| async {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(rejected_initialization())
            })
            .await;
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 8);
        assert!(
            matches!(result, Err(CanonicalError::Driver(error)) if matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
        );
        attempts.store(0, std::sync::atomic::Ordering::SeqCst);
        let result = store
            .initialize_with(|| async {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(CanonicalError::Timeout)
            })
            .await;
        assert!(matches!(result, Err(CanonicalError::Timeout)));
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(store.initialization_required().await.unwrap());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn initialization_sixteen_independent_databases_complete_in_parallel() {
        let barrier = Arc::new(tokio::sync::Barrier::new(16));
        let mut jobs = tokio::task::JoinSet::new();
        for _ in 0..16 {
            let barrier = barrier.clone();
            jobs.spawn(async move {
                let (store, _) = initialization_fixture().await;
                barrier.wait().await;
                let result = store.create().await;
                (store, result)
            });
        }
        let mut outcomes = Vec::new();
        while let Some(outcome) = jobs.join_next().await {
            outcomes.push(outcome.unwrap());
        }
        assert_eq!(outcomes.len(), 16);
        // Drain every issued initializer before cleanup or surfacing a failure.
        let mut failures = Vec::new();
        for (store, result) in outcomes {
            match result {
                Ok(()) => {
                    if let Err(error) = store.open().await {
                        failures.push(error.to_string());
                    }
                }
                Err(error) => failures.push(error.to_string()),
            }
            store.remove_isolated_fixture().await.unwrap();
        }
        assert!(
            failures.is_empty(),
            "parallel initialization failures: {failures:?}"
        );
    }
    #[tokio::test]
    async fn initialization_uses_atomic_transition_client_for_complete_schema() {
        let (mut store, options) = initialization_fixture().await;
        store
            .replace_fixture_transport(&options, std::time::Duration::from_millis(250))
            .unwrap();
        let cancelled = request(store.db.query("SLEEP 750ms; RETURN true;")).await;
        assert!(
            cancelled.and_then(complete_response).is_err(),
            "the ordinary client retains its shorter query deadline"
        );
        // The dispatched probe has an uncertain outcome while its session drains.
        // Schema setup starts on a fresh ordinary client with the same short budget.
        store
            .replace_fixture_transport(&options, std::time::Duration::from_millis(250))
            .unwrap();
        store.create().await.unwrap();
        store.open().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    #[expect(
        clippy::panic,
        reason = "intentional unwind verifies replacement client teardown on a current-thread test runtime"
    )]
    async fn initialization_replacement_transport_drains_on_current_thread_unwind() {
        let (mut store, options) = initialization_fixture().await;
        store
            .replace_fixture_transport(&options, std::time::Duration::from_millis(250))
            .unwrap();
        let resource = store.fixture_lifetime.as_ref().unwrap().resource.clone();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _owned_replacement = store;
            panic!("intentional failure after retaining the replacement transport");
        }));
        assert!(outcome.is_err());
        let status =
            crate::testing::resource_bridge("status", &serde_json::json!({"resource": resource}))
                .unwrap();
        assert_eq!(status["drained"], serde_json::json!(true));
    }
    #[tokio::test]
    async fn initialization_requires_complete_response_and_verified_marker() {
        let (store, _) = initialization_fixture().await;
        // Use an actual SDK zero-statement response, not a fabricated response value.
        assert!(matches!(
            store.initialize_with(|| request(store.db.query(""))).await,
            Err(CanonicalError::IncompleteResponse)
        ));
        assert!(store.open().await.is_err());
        store.create().await.unwrap();
        store.open().await.unwrap();
        store.create().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
        let (store, _) = initialization_fixture().await;
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        assert!(matches!(
            store.initialize_with(|| async {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let complete = store.submit_initialization().await?;
                bounded_query(store.db.query("UPDATE canonical_interpretations:current SET schema_digest='unknown-reader';")).await?;
                Ok(complete)
            }).await,
            Err(CanonicalError::Interpretation { .. })
        ));
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "readback failure after acknowledged installation must not replay DDL"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn initialization_lost_ack_is_uncertain_without_ddl_replay() {
        let (store, _) = initialization_fixture().await;
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        // Commit occurred, but the caller lost its acknowledgment at this boundary.
        assert!(matches!(
            store
                .initialize_with(|| async {
                    attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    complete_response(store.submit_initialization().await?)?;
                    Err(CanonicalError::Timeout)
                })
                .await,
            Err(CanonicalError::Timeout)
        ));
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 1);
        store.open().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn initialization_cancellation_cannot_establish_success() {
        let (store, options) = initialization_fixture().await;
        let address = url::Url::parse(&options.endpoint)
            .unwrap()
            .socket_addrs(|| None)
            .unwrap()[0];
        let selected = connect_client(address, &options, REQUEST_TIMEOUT, true)
            .await
            .unwrap();
        let delayed = initialization_statement().replacen("BEGIN;", "BEGIN; SLEEP 1s;", 1);
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        let response = store
            .initialize_with(|| {
                // Authentication and initialization inventory are fixture setup;
                // only the delayed transaction consumes the original short clock.
                within_clock(
                    original_deadline(std::time::Duration::from_millis(50)),
                    async {
                        attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        request(
                            selected
                                .query(delayed.clone())
                                .bind(("interpretation", wire::INTERPRETATION))
                                .bind(("schema_digest", wire::SCHEMA_DIGEST)),
                        )
                        .await
                    },
                )
            })
            .await;
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "cancellation must attempt the delayed transaction without replaying DDL"
        );
        assert!(response.is_err());
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        selected.disconnect().await.unwrap();
        assert!(store.open().await.is_err());
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn query_cancellation_cannot_establish_empty_success() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let address = url::Url::parse(&options.endpoint)
            .unwrap()
            .socket_addrs(|| None)
            .unwrap()[0];
        let selected = connect_client(address, &options, REQUEST_TIMEOUT, true)
            .await
            .unwrap();
        let result = within_clock(
            original_deadline(std::time::Duration::from_millis(50)),
            async {
                bounded_query(selected.query("BEGIN; SLEEP 1s; CREATE canonical_guards:late SET key = 'late', generation = 1dec; fn::pse_execution_v1::deadline($pse_rpc_expires_at); COMMIT;")).await
            },
        ).await;
        assert!(
            result.is_err(),
            "cancellation cannot become a completed empty transaction"
        );
        // Cancellation classifies an unknown transport outcome. The transaction
        // expiry fence establishes rollback after server work actually settles.
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        selected.disconnect().await.unwrap();
        let absent: Option<Object> = request(store.db.select(("canonical_guards", "late")))
            .await
            .unwrap();
        assert!(absent.is_none());
        let mut empty = bounded_query(
            store
                .db
                .query("SELECT * FROM canonical_guards WHERE key = 'late';"),
        )
        .await
        .unwrap();
        assert!(
            empty.take::<Vec<Object>>(0).unwrap().is_empty(),
            "a completed empty selector remains valid"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn large_activation_commits_3500_tiny_objects() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let edits = (0..3500)
            .map(|ordinal| {
                let logical = format!("object-{ordinal}");
                ObjectEdit {
                    logical: logical.clone(),
                    scope: "root".into(),
                    name: logical.clone(),
                    version: Some(version(&format!("version-{ordinal}"), &logical, &[1])),
                    references: Vec::new(),
                }
            })
            .collect::<Vec<_>>();
        let result = store
            .edit(
                "large-activation",
                None,
                "large-activation-operation",
                &edits,
            )
            .await;
        let revision = result.expect("3500 separately staged objects must activate atomically");
        assert_eq!(revision.sequence, 1);
        // Duplicate incoming names straddling a physical activation page still
        // refuse the complete revision, with no partially visible memberships.
        let duplicate = (0..65)
            .map(|ordinal| {
                let logical = format!("duplicate-{ordinal}");
                ObjectEdit {
                    logical: logical.clone(),
                    scope: "root".into(),
                    name: format!("duplicate-name-{}", ordinal % 64),
                    version: Some(version(
                        &format!("duplicate-version-{ordinal}"),
                        &logical,
                        &[2],
                    )),
                    references: Vec::new(),
                }
            })
            .collect::<Vec<_>>();
        assert!(
            store
                .edit(
                    "large-activation",
                    Some(&revision.key),
                    "duplicate-names",
                    &duplicate
                )
                .await
                .is_err()
        );
        let head: Option<Object> =
            request(store.db.select(("canonical_problems", "large-activation")))
                .await
                .unwrap();
        let head = wire::decode_canonical_problems(head.unwrap()).unwrap();
        assert_eq!(head.head, revision.key);
        assert_eq!(head.sequence, revision.sequence);
        assert!(store.revision("duplicate-names").await.unwrap().is_none());
        assert_eq!(store.inventory(&revision).await.unwrap().len(), 3500);
        // All old intervals close before any replacements open: a swap of names
        // is legal, while the retained historical revision keeps its old meaning.
        let swap = [0, 3499].map(|ordinal| {
            let logical = format!("object-{ordinal}");
            ObjectEdit {
                logical: logical.clone(),
                scope: "root".into(),
                name: format!("object-{}", 3499 - ordinal),
                version: Some(version(&format!("version-{ordinal}"), &logical, &[1])),
                references: Vec::new(),
            }
        });
        let swapped = store
            .edit(
                "large-activation",
                Some(&revision.key),
                "swapped-names",
                &swap,
            )
            .await
            .unwrap();
        let old = store
            .protect(revision, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let new = store
            .protect(swapped, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(
            store
                .select_names(&old, "root", &["object-0".into()])
                .await
                .unwrap()[0]
                .logical,
            "object-0"
        );
        assert_eq!(
            store
                .select_names(&new, "root", &["object-0".into()])
                .await
                .unwrap()[0]
                .logical,
            "object-3499"
        );
        store.release(&old).await.unwrap();
        store.release(&new).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn generated_native_scientific_cells_and_schema_identity() {
        use crate::canonical_codec as codec;
        use pse_schema::model::{
            Authority, ColumnRole, DerivationGranularity, FieldContract, Namespace, RelationDecl,
            SnapshotClass,
        };
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, false).unwrap();
        assert!(
            store.open().await.is_err(),
            "normal open must not install missing schema"
        );
        store.create().await.unwrap();
        let mut builder = pse_schema::RegistryBuilder::new();
        pse_schema::catalog::declare(&mut builder);
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Runtime,
                "canonical_wire_fixture",
                1,
                Authority::Derived,
                SnapshotClass::Sidecar,
                "Native exact codec fixture.",
            )
            .pk(&["key"])
            .granularity(DerivationGranularity::Row)
            .columns(vec![
                FieldContract::native(arrow_schema::DataType::Utf8)
                    .with_name("key")
                    .with_role(ColumnRole::Key),
                FieldContract::native(arrow_schema::DataType::Float64).with_name("finite"),
                FieldContract::native(arrow_schema::DataType::Float64)
                    .with_name("optional")
                    .optional(),
                FieldContract::native(arrow_schema::DataType::Binary).with_name("raw"),
            ]),
        );
        let registry = builder.build().unwrap();
        let schema = pse_codegen::codegen::canonical_schema(&registry).unwrap();
        let fixture = schema
            .lines()
            .filter(|line| {
                line.starts_with("DEFINE TABLE canonical_wire_fixture ")
                    || line.contains(" ON canonical_wire_fixture ")
            })
            .collect::<Vec<_>>()
            .join("\n");
        store.db.query(fixture).await.unwrap().check().unwrap();
        for (index, value) in [
            0.0f64,
            -0.0,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
            f64::MAX,
            -17.25,
        ]
        .into_iter()
        .enumerate()
        {
            let mut object = Object::new();
            object.insert("key", format!("cell-{index}"));
            object.insert("finite", codec::encode_finite(value).unwrap());
            object.insert("raw", codec::encode_diagnostic_bits(0x7ff8_0000_0000_0042));
            let mut response = bounded_query(
                store
                    .db
                    .query("CREATE type::record('canonical_wire_fixture', $key) CONTENT $row;")
                    .bind(("key", format!("cell-{index}")))
                    .bind(("row", object)),
            )
            .await
            .unwrap();
            let saved: Option<Object> = response.take(0).unwrap();
            let mut saved = saved.unwrap();
            assert_eq!(
                codec::decode_finite(saved.remove("finite").unwrap())
                    .unwrap()
                    .to_bits(),
                value.to_bits()
            );
            assert!(
                saved
                    .get("optional")
                    .is_none_or(|value| matches!(value, Value::None))
            );
            assert_eq!(
                codec::decode_diagnostic_bits(saved.remove("raw").unwrap()).unwrap(),
                0x7ff8_0000_0000_0042
            );
        }
        // Deliberately malformed native input cannot become a valid scientific cell.
        let mut short = codec::encode_finite(1.0).unwrap();
        if let Value::Object(ref mut object) = short {
            object.insert("bits", surrealdb::types::Bytes::from(vec![0u8; 7]));
        }
        assert!(
            store
                .db
                .query(
                    "CREATE canonical_wire_fixture:bad SET key = 'bad', finite = $cell, raw = $raw;"
                )
                .bind(("cell", short))
                .bind(("raw", codec::encode_diagnostic_bits(0)))
                .await
                .unwrap()
                .check()
                .is_err()
        );
        for absent in [Value::None, Value::Null] {
            assert!(store.db.query("CREATE canonical_wire_fixture:absent SET key = 'absent', finite = $cell, raw = $raw;").bind(("cell", absent)).bind(("raw", codec::encode_diagnostic_bits(0))).await.unwrap().check().is_err());
        }
        let mut corrupt = codec::encode_finite(2.0).unwrap();
        if let Value::Object(ref mut object) = corrupt {
            object.insert("projection", 3.0);
        }
        let mut result = store
            .db
            .query("RETURN $cell;")
            .bind(("cell", corrupt))
            .await
            .unwrap();
        assert!(codec::decode_finite(result.take::<Value>(0).unwrap()).is_err());
        assert!(
            codec::check_record_identity(
                codec::record_link("other_table", "x"),
                "canonical_wire_fixture",
                "x"
            )
            .is_err()
        );
        store
            .db
            .query("UPDATE canonical_interpretations:current SET schema_digest = 'unknown-reader';")
            .await
            .unwrap()
            .check()
            .unwrap();
        assert!(matches!(
            store.open().await,
            Err(CanonicalError::Interpretation { .. })
        ));
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn websocket_exact_cells_and_guarded_revisions() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        store.open().await.unwrap();
        let result: Result<(), CanonicalError> = async {
            for (index, value) in [0, (1u64<<63)-1, 1u64<<63, (1u64<<63)+1, u64::MAX-1, u64::MAX].into_iter().enumerate() {
                let row = pse_model::generated::runtime::canonical_guards::Row { key: format!("boundary-{index}"), generation: value, incarnation: None, analysis_creation_closed_through: None };
                let mut response = store.db.query("CREATE type::record('canonical_guards', $key) CONTENT $row;").bind(("key", row.key.clone())).bind(("row", wire::encode_canonical_guards(&row)?)).await.and_then(checked)?;
                let saved: Option<Object> = response.take(0)?;
                assert_eq!(wire::decode_canonical_guards(saved.unwrap())?.generation, value);
            }
            let mut ordered = store.db.query("SELECT generation FROM canonical_guards WHERE generation >= $lower ORDER BY generation;").bind(("lower", crate::canonical_codec::encode_uint(1u64<<63)?)).await.and_then(checked)?;
            let rows: Vec<Object> = ordered.take(0)?;
            let ordered = rows.into_iter().map(|mut row| crate::canonical_codec::decode_uint(row.remove("generation").unwrap())).collect::<Result<Vec<_>,_>>()?;
            assert_eq!(ordered, [1u64<<63, (1u64<<63)+1, u64::MAX-1, u64::MAX]);
            for value in ["-1", "0.5", "18446744073709551616"] {
                let response = store.db.query("CREATE canonical_guards:invalid SET key = 'invalid', generation = $generation;").bind(("generation", Value::Number(Number::Decimal(value.parse().unwrap())))).await?;
                assert!(response.check().is_err());
            }
            let initial = vec![ObjectEdit { logical: "x".into(), scope: "root".into(), name: "x".into(), version: Some(version("x-v1", "x", &[1,2,3])), references: vec![] }, ObjectEdit { logical: "y".into(), scope: "root".into(), name: "y".into(), version: Some(version("y-v1", "y", &[4,5])), references: vec![] }];
            let first = store.edit("problem", None, "op-1", &initial).await.map_err(|error| CanonicalError::Configuration(format!("initial edit: {error}")))?;
            assert_eq!(first.sequence, 1);
            assert_eq!(store.edit("problem", None, "op-1", &initial).await?, first);
            assert!(store.edit("problem", None, "op-1", &[]).await.is_err());
            let changed = [ObjectEdit { logical: "x".into(), scope: "root".into(), name: "x".into(), version: Some(version("x-v2", "x", &[9])), references: vec![] }];
            let second = store.edit("problem", Some("op-1"), "op-2", &changed).await.map_err(|error| CanonicalError::Configuration(format!("second edit: {error}")))?;
            assert_eq!(second.sequence, 2);
            assert!(store.edit("problem", Some("op-1"), "op-stale", &[]).await.is_err());
            let old = store.inventory(&first).await.map_err(|error| CanonicalError::Configuration(format!("old inventory: {error}")))?;
            let new = store.inventory(&second).await.map_err(|error| CanonicalError::Configuration(format!("new inventory: {error}")))?;
            assert_eq!(old.iter().find(|member| member.logical == "x").unwrap().version, "x-v1");
            assert_eq!(new.iter().find(|member| member.logical == "x").unwrap().version, "x-v2");
            assert_eq!(new.iter().find(|member| member.logical == "y").unwrap().version, "y-v1");
            assert_eq!(store.object("x-v1").await?.unwrap().payload.as_slice(), [1,2,3]);
            let selection = store.protect(first, std::time::Duration::from_secs(30)).await.map_err(|error| CanonicalError::Configuration(format!("protection: {error}")))?;
            let selected = store.select_names(&selection, "root", &["x".into()]).await.map_err(|error| CanonicalError::Configuration(format!("selected names: {error}")))?;
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].version, "x-v1");
            store.release(&selection).await?;
            assert!(store.select_names(&selection, "root", &["x".into()]).await.is_err());
            Ok(())
        }.await;
        assert_eq!(store.database(), options.database);
        store.remove_isolated_fixture().await.unwrap();
        result.unwrap();
    }
}
