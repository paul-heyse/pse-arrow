// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete remote canonical substrate. Scientific work runs outside guarded transactions.

use crate::canonical_script::{Envelope, GuardedScript, Recheck, ResultSlot};
pub(crate) use crate::transport::lifetime;
use crate::{canonical_codec::CodecError, generated::surreal as wire};
#[cfg(feature = "canonical-maintenance")]
#[path = "canonical_maintenance.rs"]
pub mod maintenance;
#[path = "canonical_settlement.rs"]
pub(crate) mod settlement;
#[path = "canonical_transport.rs"]
pub(crate) mod transport;
pub use lifetime::{ConnectionActivity, inherit_activity};
pub use pse_model::generated::runtime::{
    canonical_memberships::Row as Membership, canonical_revisions::Row as Revision,
    canonical_versions::Row as ObjectVersion,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use surrealdb_types::{Object, Value};
#[path = "canonical_metadata.rs"]
mod metadata;
#[path = "canonical_study_module.rs"]
pub(crate) mod study_module;
pub(crate) use crate::transport::{
    bounded_query, checked, complete_response, operation_failed, protected_query, request,
};
use crate::transport::{original_deadline, within_clock};
pub use metadata::MetadataResponse;
use transport::CanonicalClient;
pub use transport::{
    CanonicalQueryEvent, CanonicalQueryMetrics, CanonicalQuerySample, query_metrics,
};

/// Initial negotiated protocol limit; batches must leave space for their envelope.
pub const MESSAGE_BYTES: usize = wire::RESULT_MESSAGE_BYTES;
/// Bounded submitted payload, leaving conservative room for query/record metadata.
pub const PAYLOAD_BYTES: usize = 3 * 1024 * 1024;
/// Client operation deadline. Mutations settle a lost acknowledgment by immutable identity.
pub use crate::transport::REQUEST_TIMEOUT;
pub(crate) const ACTIVATION_REQUEST_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(90);
// A finite contention window for sixteen clients sharing one retention guard.
// This does not change any RPC deadline or permit replay after uncertain completion.
pub(crate) use crate::transport::GUARDED_ATTEMPTS;

/// Registry-owned native refusal identities and their policy meaning.
pub use pse_schema::catalog::canonical_refusals::{RefusalClass, RefusalCode};
/// What a returned failure actually establishes about the submitted effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureEffect {
    /// This exact statement was synchronously refused; enclosing transaction policy
    /// still owns any broader absence/replay conclusion.
    StatementRefused,
    /// Dispatch/commit may have happened; settle the original immutable receipt.
    Unknown,
}
/// Native MX failure meaning, carried separately from domain THROW codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreFailure {
    /// Finite resident-engine capacity was exhausted.
    MemoryLimit,
    /// The transaction exceeded the configured maximum age.
    TransactionTooOld,
    /// The engine requires supervised recovery after an unrecoverable failure.
    DatabasePoisoned,
    /// A configured asynchronous persistence queue refused admission.
    AsyncQueueFull,
    /// Persistence contents failed integrity validation.
    Corrupted,
    /// Persistence failed without establishing the commit outcome.
    CommitUnknown,
}

/// Canonical protocol and guarded-operation failures.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(runtime::infrastructure))]
pub enum CanonicalError {
    /// A live exact capture or scan still owns its immutable selection.
    #[error("canonical study capture {capture} still has live readers")]
    CaptureInUse {
        /// Original immutable capture identity.
        capture: String,
    },
    /// Exact capture cleanup completed; its original grant cannot be reopened.
    #[error("canonical study capture {capture} is closed")]
    CaptureClosed {
        /// Original immutable capture identity.
        capture: String,
    },
    /// Exact immutable capture custody is draining and cannot admit another scan.
    #[error("canonical study capture {capture} is draining")]
    CaptureDraining {
        /// Original immutable capture identity, unchanged during cleanup.
        capture: String,
    },
    /// This logical caller has released admission; already admitted work is unaffected.
    #[error("canonical caller is closed")]
    CallerClosed,
    /// The actual connection pair is closing or terminal and cannot admit fresh work.
    #[error("canonical connection is unavailable")]
    ConnectionUnavailable,
    /// Physical shutdown refused without changing admission or cancelling activities.
    #[error("canonical connection still has {activities} admitted activities")]
    ConnectionBusy {
        /// Actual continuing dependencies, independent of idle aliases.
        activities: usize,
    },
    /// A physical drain attempt failed; the owner stays Closing for explicit recovery.
    #[error("canonical physical close did not finish: {source}")]
    PhysicalClose {
        /// Retained original failure from the owned teardown task.
        #[source]
        source: Arc<CanonicalError>,
    },
    #[error("canonical store configuration: {0}")]
    /// Invalid deployment or operation configuration.
    Configuration(String),
    #[error("canonical store driver: {0} (query domain: {domain:?})", domain = .0.query_details())]
    /// Protocol/database failure retaining its native typed cause.
    Driver(surrealdb_types::Error),
    /// A recognized synchronous native THROW; rendered prose never grants policy.
    #[error("canonical refusal {code} at {stage} ({class:?}, {effect:?})")]
    Refused {
        /// Exact registry identity.
        code: RefusalCode,
        /// Registry policy class, retained with the failure.
        class: RefusalClass,
        /// Actual operation stage when attributed, otherwise statement completion.
        stage: &'static str,
        /// The limited effect knowledge established by this failure.
        effect: FailureEffect,
    },
    /// Policy declined after its exact operation committed settlement and any
    /// earlier partial progress. This is not a transaction rollback or absence proof.
    #[error("canonical policy refusal {code} settled by operation {operation}")]
    SettledPolicyRefusal {
        /// Exact registry policy identity.
        code: RefusalCode,
        /// Original immutable operation whose receipt retains the outcome.
        operation: String,
    },
    /// A recognized native MX failure; never a transaction-conflict alias.
    #[error("canonical MX {failure:?} at {stage} ({effect:?}): {source}")]
    StoreFailure {
        /// Exact engine failure meaning.
        failure: StoreFailure,
        /// Operation stage retaining the engine refusal.
        stage: &'static str,
        /// Commit/dispatch uncertainty remains explicit.
        effect: FailureEffect,
        /// Original native typed failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Recovery fenced a lost worker; this is infrastructure, not a schema refusal.
    #[error("canonical worker lost for {run}/{attempt} generation {generation}")]
    WorkerLost {
        /// Original run identity.
        run: String,
        /// Original attempt identity.
        attempt: String,
        /// Original worker generation.
        generation: u64,
    },
    /// Definite native conflicts exhausted the finite fresh-decision budget.
    #[error("canonical {operation} exhausted {attempts} transaction-conflict attempts")]
    ConflictExhausted {
        /// Original guarded operation identity/stage.
        operation: &'static str,
        /// Total actual attempts, not replay permission.
        attempts: u32,
    },
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
    /// Scientific transfer may have reached the server; retain the exact sealed intent.
    #[error("artifact {key} publication did not complete: {source}", key = .intent.key)]
    ArtifactUnsettled {
        /// Original finite creation authority and exact selected closure.
        intent: Box<crate::canonical_artifacts::Artifact>,
        /// Original failure, never inferred to mean rollback.
        #[source]
        source: Box<CanonicalError>,
    },
    /// The actual source transaction refused this occurrence and an exact receipt
    /// read afterwards observed no acknowledgement. This is never minted from a
    /// post-commit acknowledgement-read failure.
    #[error("source edit {operation} for {problem} was refused: {source}")]
    SourceEditRefused {
        /// Original problem selected by the publication intent.
        problem: String,
        /// Original immutable source operation identity.
        operation: String,
        /// Actual native transaction refusal proof.
        #[source]
        source: Box<CanonicalError>,
    },
    /// Source publication may have reached the server; recover only this issue.
    #[error("source edit {operation} for {problem} did not settle: {source}")]
    SourceEditUnsettled {
        /// Exact source problem, never inferred from its current head.
        problem: String,
        /// Original immutable operation receipt identity.
        operation: String,
        /// Original typed failure; no success is inferred from it.
        #[source]
        source: Box<CanonicalError>,
    },
    /// An exact owner's effect remains unresolved; neither cause proves absence.
    #[error(
        "canonical {owner} operation {operation} did not settle: {source}; readback: {readback:?}"
    )]
    SettlementUnresolved {
        /// Actual effect owner, rather than the transport used for readback.
        owner: &'static str,
        /// Original immutable operation identity.
        operation: String,
        /// Complete original immutable request, retained for exact recovery.
        request: Vec<u8>,
        /// Original submission or acknowledgement-decoding failure.
        #[source]
        source: Box<CanonicalError>,
        /// Independent readback failure; absent for a completed but inconclusive read.
        readback: Option<Box<CanonicalError>>,
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
    /// Automatic protection cleanup failed; repeated drain retains its exact cause.
    #[error("canonical protection release did not settle: {source}")]
    ProtectionRelease {
        /// Original cleanup failure, never consumed by observing it.
        #[source]
        source: Arc<CanonicalError>,
    },
}
impl From<surrealdb_types::Error> for CanonicalError {
    fn from(error: surrealdb_types::Error) -> Self {
        // Only the native kind authenticates this wire grammar. An Internal error
        // whose message resembles THROW must remain an invariant/infrastructure error.
        if error.is_thrown() {
            let message = error
                .message()
                .strip_prefix("An error occurred: ")
                .unwrap_or(error.message());
            if let Some(code) = RefusalCode::parse(message) {
                return Self::Refused {
                    code,
                    class: code.class(),
                    stage: "statement",
                    effect: FailureEffect::StatementRefused,
                };
            }
        } else if error.is_internal() {
            let failure = [
                ("PSE_MX_MEMORY_LIMIT:", StoreFailure::MemoryLimit),
                ("PSE_MX_TX_TOO_OLD:", StoreFailure::TransactionTooOld),
                ("PSE_MX_DATABASE_POISONED:", StoreFailure::DatabasePoisoned),
                ("PSE_MX_ASYNC_QUEUE_FULL:", StoreFailure::AsyncQueueFull),
                ("PSE_MX_CORRUPTED:", StoreFailure::Corrupted),
                ("PSE_MX_COMMIT_UNKNOWN:", StoreFailure::CommitUnknown),
            ]
            .into_iter()
            .find_map(|(prefix, failure)| error.message().starts_with(prefix).then_some(failure));
            if let Some(failure) = failure {
                return Self::StoreFailure {
                    failure,
                    stage: "statement",
                    effect: FailureEffect::Unknown,
                    source: Box::new(error),
                };
            }
        }
        Self::Driver(error)
    }
}
impl CanonicalError {
    /// Infrastructure termination is distinct from a scientific/domain refusal.
    pub fn is_infrastructure(&self) -> bool {
        if let Self::SettlementUnresolved {
            source, readback, ..
        } = self
        {
            return source.is_infrastructure()
                || readback
                    .as_ref()
                    .is_some_and(|error| error.is_infrastructure());
        }
        if let Self::AnalysisUnsettled { source, .. }
        | Self::ArtifactUnsettled { source, .. }
        | Self::SourceEditUnsettled { source, .. }
        | Self::SourceEditRefused { source, .. } = self
        {
            return source.is_infrastructure();
        }
        matches!(
            self,
            Self::CaptureInUse { .. }
                | Self::CaptureDraining { .. }
                | Self::CaptureClosed { .. }
                | Self::Driver(_)
                | Self::StoreFailure { .. }
                | Self::ConflictExhausted { .. }
                | Self::WorkerLost { .. }
                | Self::ConnectionUnavailable
                | Self::PhysicalClose { .. }
                | Self::Timeout
                | Self::IncompleteResponse
                | Self::ProtectionRelease { .. }
        )
    }
    /// Whether this failure leaves submitted effects unresolved. The scientific
    /// owner must settle original receipts before granting replay.
    pub fn effect_is_unknown(&self) -> bool {
        match self {
            Self::StoreFailure { effect, .. } | Self::Refused { effect, .. } => {
                *effect == FailureEffect::Unknown
            }
            Self::Driver(error) => {
                !error.is_thrown()
                    && !matches!(
                        error.query_details(),
                        Some(
                            surrealdb_types::QueryError::NotExecuted
                                | surrealdb_types::QueryError::TransactionConflict
                        )
                    )
            }
            Self::Timeout | Self::IncompleteResponse | Self::WorkerLost { .. } => true,
            Self::AnalysisUnsettled { .. }
            | Self::ArtifactUnsettled { .. }
            | Self::SourceEditUnsettled { .. }
            | Self::SettlementUnresolved { .. } => true,
            Self::PhysicalClose { source } | Self::ProtectionRelease { source } => {
                source.effect_is_unknown()
            }
            _ => false,
        }
    }
    /// Only fresh-premise refusals can demand a new scientific policy decision.
    pub fn needs_fresh_premise(&self) -> bool {
        matches!(
            self,
            Self::Refused {
                class: RefusalClass::FreshPremise,
                ..
            }
        )
    }
    /// Capacity or definite contention may be retried by an owning policy; this
    /// observation never grants replay of an uncertain mutation.
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::ConflictExhausted { .. }
                | Self::WorkerLost { .. }
                | Self::StoreFailure {
                    failure: StoreFailure::MemoryLimit
                        | StoreFailure::TransactionTooOld
                        | StoreFailure::AsyncQueueFull,
                    ..
                }
        )
    }
    fn at_stage(mut self, operation: &'static str) -> Self {
        match &mut self {
            Self::Refused { stage, .. } | Self::StoreFailure { stage, .. } => *stage = operation,
            _ => {}
        }
        self
    }
    pub(crate) fn is_capture_busy(&self) -> bool {
        matches!(
            self,
            Self::Refused {
                code: RefusalCode::StudyCapturePublicationBusy,
                effect: FailureEffect::StatementRefused,
                ..
            }
        )
    }
    pub(crate) fn is_conflict(&self) -> bool {
        matches!(self, Self::Driver(error) if matches!(error.query_details(), Some(surrealdb_types::QueryError::TransactionConflict)))
    }
    pub(crate) fn exhaust_conflict(self, operation: &'static str, attempts: u32) -> Self {
        if self.is_conflict() {
            Self::ConflictExhausted {
                operation,
                attempts,
            }
        } else {
            self.at_stage(operation)
        }
    }
    /// Whether the source-effect owner established refusal in its actual
    /// transaction phase, rather than merely observing no receipt.
    pub fn proves_source_refusal(&self) -> bool {
        matches!(self, Self::SourceEditRefused { .. })
    }
    // Used only at the exact source transaction response, before interpreting a
    // later receipt read. The same error types at that later read prove nothing.
    pub(crate) fn source_transaction_refused(&self) -> bool {
        matches!(self, Self::Refused { .. } | Self::ConflictExhausted { .. })
            || matches!(self,Self::Driver(error) if error.is_thrown() || matches!(error.query_details(),Some(surrealdb_types::QueryError::NotExecuted|surrealdb_types::QueryError::TransactionConflict)))
    }
}
impl pse_diagnostics::TypedDiagnostic for CanonicalError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(match self {
            Self::Codec(_) => pse_diagnostics::DiagnosticCode::ValidationInvariant,
            _ => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        })
    }
}

/// Carry an enclosing operation's original deadline into all nested canonical
/// requests. The caller retains ownership of in-flight effects through settlement;
/// this scope does not spawn, detach or timeout-drop the enclosing future.
pub fn operation_scope<F: Future>(
    deadline: std::time::Instant,
    future: F,
) -> impl Future<Output = F::Output> {
    transport::operation_scope(deadline, future)
}

/// Retain an actual prepaid response allocation through native transport drain.
/// Nested scopes preserve their enclosing owners and original request clocks.
pub async fn response_ownership_scope<F: Future>(
    owner: impl Into<crate::transport::client::ResponseAllocation>,
    future: F,
) -> F::Output {
    transport::response_ownership_scope(owner, future).await
}

/// Retain issued read/release owners independently of prepaid allocation custody.
/// The scope supplies no new permission; native work retains these exact guards
/// until its original response processing and physical drain have completed.
pub async fn request_custody_scope<F: Future>(
    owner: impl Into<crate::transport::client::RetainedRequest>,
    future: F,
) -> F::Output {
    transport::request_custody_scope(owner, future).await
}

/// Authenticated deployment parameters. Credentials never appear in Debug or status output.
#[derive(Clone)]
pub struct CanonicalOptions {
    /// Authenticated loopback native gRPC endpoint.
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
struct SelectedStudyModule {
    sha256: String,
}

#[derive(serde::Deserialize)]
struct WriteAdmission {
    #[serde(default)]
    study_module: Option<SelectedStudyModule>,
    namespace: String,
    database: String,
    accepting_writes: bool,
    #[serde(default)]
    derived_rebuild_pending: Option<PendingRebuild>,
    #[serde(default)]
    surrealkit_rollout_pending: Option<PendingRebuild>,
}

impl WriteAdmission {
    fn read(state: &Path) -> Result<Self, CanonicalError> {
        let config = std::fs::read(state.join("config.json"))
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        serde_json::from_slice(&config)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))
    }

    fn require_module(&self, expected: Option<&String>) -> Result<(), CanonicalError> {
        if let Some(expected) = expected
            && self.study_module.as_ref().map(|module| &module.sha256) != Some(expected)
        {
            return Err(CanonicalError::Quiesced);
        }
        Ok(())
    }

    fn require_initializer(
        &self,
        state: &Path,
        namespace: &str,
        database: &str,
        nonce: Option<&str>,
    ) -> Result<(), CanonicalError> {
        if self.derived_rebuild_pending.is_some() && self.surrealkit_rollout_pending.is_some() {
            return Err(CanonicalError::Quiesced);
        }
        let owner = self
            .derived_rebuild_pending
            .as_ref()
            .or(self.surrealkit_rollout_pending.as_ref())
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
        if self.derived_rebuild_pending.is_some() || self.surrealkit_rollout_pending.is_some() {
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
const CREDENTIALS_FILE_BYTES: u64 = 64 * 1024;

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
        let read = |path: &Path, max_bytes: u64| -> Result<Vec<u8>, CanonicalError> {
            let metadata = std::fs::symlink_metadata(path)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > max_bytes
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
        let configuration = read(
            &state.join("config.json"),
            crate::deployment::CONFIGURATION_FILE_BYTES,
        )?;
        let deployment: crate::deployment::ServiceIdentity =
            crate::deployment::projection(&configuration)
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        let connection: crate::deployment::CanonicalConnectionSettings =
            crate::deployment::projection(&configuration)
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
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
        }
        let initializer_nonce = std::env::var(INITIALIZER_AUTHORITY).ok();
        let admission = WriteAdmission::read(state)?;
        if admission.derived_rebuild_pending.is_some()
            || admission.surrealkit_rollout_pending.is_some()
        {
            admission.require_initializer(
                state,
                &deployment.namespace,
                &deployment.database,
                initializer_nonce.as_deref(),
            )?;
        }
        if connection.schema_interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Interpretation {
                expected: wire::INTERPRETATION,
                observed: connection.schema_interpretation,
            });
        }
        if connection.max_message_bytes != MESSAGE_BYTES {
            return Err(CanonicalError::Configuration(
                "unsupported protocol bound".into(),
            ));
        }
        study_module::StudyModuleReceipt::read(state)?;
        let credentials: Credentials = serde_json::from_slice(&read(
            Path::new(&connection.credentials_file),
            CREDENTIALS_FILE_BYTES,
        )?)
        .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        Ok(Self {
            endpoint: deployment.endpoint,
            namespace: deployment.namespace,
            database: deployment.database,
            username: credentials.username,
            password: credentials.password,
            selection_username: credentials.selection_username,
            selection_password: credentials.selection_password,
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
    activity: std::sync::Mutex<Option<ConnectionActivity>>,
    released: std::sync::atomic::AtomicBool,
    metadata_owner: std::sync::OnceLock<crate::transport::client::ResponseAllocation>,
}
impl ProtectedSelection {
    #[cfg(test)]
    pub(crate) fn test_only(revision: Revision, key: impl Into<String>) -> Self {
        Self {
            owner: Arc::new(ProtectionOwner {
                key: key.into(),
                revision,
                store: None,
                executor: None,
                activity: std::sync::Mutex::new(None),
                released: std::sync::atomic::AtomicBool::new(false),
                metadata_owner: std::sync::OnceLock::new(),
            }),
        }
    }
    /// Attach the one actual retained metadata allocation. Aliases may attach
    /// the same Arc; replacing it could release storage still borrowed by work.
    pub fn retain_metadata_owner(
        &self,
        owner: impl Into<crate::transport::client::ResponseAllocation>,
    ) -> Result<(), CanonicalError> {
        let owner: crate::transport::client::ResponseAllocation = owner.into();
        if let Some(existing) = self.owner.metadata_owner.get() {
            return if existing.same_owner(&owner) {
                Ok(())
            } else {
                Err(CanonicalError::Configuration(
                    "protected metadata allocation owner differs".into(),
                ))
            };
        }
        if self.owner.metadata_owner.set(owner.clone()).is_err()
            && self
                .owner
                .metadata_owner
                .get()
                .is_none_or(|existing| !existing.same_owner(&owner))
        {
            return Err(CanonicalError::Configuration(
                "protected metadata allocation owner differs".into(),
            ));
        }
        Ok(())
    }
    /// Native retained metadata extent, including the exact revision and owned
    /// store-handle values. Shared physical clients are charged by deployment.
    /// Returns `None` if the represented allocation extent overflows.
    pub fn retained_bytes(&self) -> Option<usize> {
        use pse_model::HeapUsage;
        let owner = &self.owner;
        let store = owner
            .store
            .as_ref()
            .map_or(Some(0), CanonicalStore::handle_heap_bytes)?;
        [
            size_of::<ProtectionOwner>(),
            2 * size_of::<usize>(),
            owner.key.capacity(),
            owner.revision.heap_bytes(),
            store,
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
    }
    pub(crate) fn key(&self) -> &str {
        &self.owner.key
    }
    /// Original grant identity, binding its server deadline, nonce and authority
    /// incarnation. An owner may save this value before finishing the selection
    /// and later pass it to [`CanonicalStore::settle_protection_grant`].
    /// Saving it does not extend the acquisition window or renew a released pin.
    pub fn grant_key(&self) -> &str {
        &self.owner.key
    }
    /// Exact canonical revision retained by this protection.
    pub fn revision(&self) -> &Revision {
        &self.owner.revision
    }
    /// Extend this exact live grant. Expired or explicitly released grants never revive.
    /// The original acquisition identity and already issued operation clocks do not change.
    pub async fn renew(&self, lifetime: std::time::Duration) -> Result<(), CanonicalError> {
        let store = self.owner.store.as_ref().ok_or_else(|| {
            CanonicalError::Configuration("source protection has no issuing store".into())
        })?;
        if self
            .owner
            .released
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(CanonicalError::Configuration(
                "source protection was released".into(),
            ));
        }
        transport::optional_response_ownership_scope(
            self.owner.metadata_owner.get().cloned(),
            store.renew_protection(self, lifetime),
        )
        .await
    }
    /// Relinquish this reader. The final follower awaits exact protection cleanup.
    /// Other live followers retain their authority until they finish or drop.
    pub async fn finish(self) -> Result<(), CanonicalError> {
        if let Some(owner) = Arc::into_inner(self.owner) {
            if owner.released.load(std::sync::atomic::Ordering::Acquire) {
                return Ok(());
            }
            let Some(store) = owner.store.as_ref() else {
                return Ok(());
            };
            let store = store.clone();
            let key = owner.key.clone();
            let revision = owner.revision.clone();
            let executor = owner.executor.clone().ok_or_else(|| {
                CanonicalError::Configuration("protection cleanup executor unavailable".into())
            })?;
            let activity = owner.activity()?;
            #[cfg(any(test, feature = "test-support"))]
            let fixture = store.fixture_lifetime.clone();
            let metadata_owner = owner.metadata_owner.get().cloned();
            let task = executor.spawn(async move {
                let result = transport::optional_response_ownership_scope(
                    metadata_owner,
                    activity.scope(async move {
                        let result = store.release_protection(&revision, &key).await;
                        if result.is_ok() {
                            owner.acknowledge_release();
                        }
                        result
                    }),
                )
                .await;
                drop(activity);
                // The last fixture borrower may physically disconnect here.
                // Its own cleanup continuation must already have ended.
                #[cfg(any(test, feature = "test-support"))]
                drop(fixture);
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
    fn activity(&self) -> Result<ConnectionActivity, CanonicalError> {
        self.activity
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or_else(|| CanonicalError::Configuration("source protection was released".into()))
    }
    fn acknowledge_release(&self) {
        if !self
            .released
            .swap(true, std::sync::atomic::Ordering::AcqRel)
            && let Some(store) = &self.store
        {
            let release = store.result_read_drain.release();
            self.activity
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            release.complete(Ok(()));
        }
    }
}
impl Drop for ProtectionOwner {
    fn drop(&mut self) {
        if self.released.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        if let (Some(store), Some(executor)) = (&self.store, &self.executor) {
            let store = store.clone();
            #[cfg(any(test, feature = "test-support"))]
            let store = {
                // The last lexical fixture borrower must join this tracked release
                // before its test process can exit. Retaining the fixture in this
                // detached task would postpone that join until after Drop returns.
                // Its physical clients remain owned by FixtureLifetime throughout
                // disconnect's ResultReadDrain join.
                let mut cleanup = store;
                cleanup.fixture_lifetime = None;
                cleanup
            };
            let revision = self.revision.clone();
            let key = self.key.clone();
            let activity = match self
                .activity
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                Some(activity) => activity,
                None => return,
            };
            let store = activity.sync_scope(|| store.activity_store());
            let store = match store {
                Ok(store) => store,
                Err(error) => {
                    operation_failed("canonical::protection_drop", &error);
                    return;
                }
            };
            let metadata_owner = self.metadata_owner.take();
            let release = store.result_read_drain.release_protection(
                store.clone(),
                revision.clone(),
                key.clone(),
                metadata_owner.clone(),
            );
            executor.spawn(async move {
                let result = transport::optional_response_ownership_scope(
                    metadata_owner,
                    store.release_protection(&revision, &key),
                )
                .await;
                if let Err(error) = &result {
                    operation_failed("canonical::protection_drop", error);
                }
                // Completion wakes physical drain. Relinquish this task's
                // continuation first; failure keeps its exact recovery owner
                // in the release guard instead of reporting a false drain.
                drop(store);
                release.complete(result);
            });
        }
    }
}

/// Local pacing of complete short decisions sharing one problem's server guards.
/// The server's guarded predicates and generations remain the distributed authority.
#[derive(Clone, Copy)]
pub(crate) enum QueryEffect {
    Read,
    Mutation,
}

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "canonical_pacing_observation.rs"]
pub(crate) mod pacing_observation;

#[derive(Default)]
struct ProblemTurns {
    problems: std::sync::Mutex<
        std::collections::BTreeMap<String, std::sync::Weak<tokio::sync::RwLock<()>>>,
    >,
}
impl ProblemTurns {
    async fn run<F, T>(
        &self,
        problem: &str,
        effect: QueryEffect,
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
        #[cfg(all(test, feature = "canonical-tests"))]
        let queued = {
            pacing_observation::submitted(problem);
            std::time::Instant::now()
        };
        within_clock(deadline, async {
            match effect {
                QueryEffect::Read => {
                    let _turn = gate.read_owned().await;
                    #[cfg(all(test, feature = "canonical-tests"))]
                    let _observation =
                        pacing_observation::entered(problem, effect, queued.elapsed());
                    if tokio::time::Instant::now() >= deadline {
                        return Err(CanonicalError::Timeout);
                    }
                    operation.await
                }
                QueryEffect::Mutation => {
                    let _turn = gate.write_owned().await;
                    #[cfg(all(test, feature = "canonical-tests"))]
                    let _observation =
                        pacing_observation::entered(problem, effect, queued.elapsed());
                    if tokio::time::Instant::now() >= deadline {
                        return Err(CanonicalError::Timeout);
                    }
                    operation.await
                }
            }
        })
        .await
    }
    fn gate(&self, problem: &str) -> Result<Arc<tokio::sync::RwLock<()>>, CanonicalError> {
        let mut problems = self.problems.lock().map_err(|_| {
            CanonicalError::Configuration("staging admission owner poisoned".into())
        })?;
        problems.retain(|_, gate| gate.strong_count() != 0);
        if let Some(gate) = problems.get(problem).and_then(std::sync::Weak::upgrade) {
            return Ok(gate);
        }
        let gate = Arc::new(tokio::sync::RwLock::new(()));
        problems.insert(problem.to_owned(), Arc::downgrade(&gate));
        Ok(gate)
    }
}

#[derive(Clone)]
/// Thin remote client; clones share its bounded transport.
pub struct CanonicalStore {
    pub(crate) db: Arc<CanonicalClient>,
    problem_turns: Arc<ProblemTurns>,
    pub(crate) result_read_drain: Arc<crate::canonical_results::ResultReadDrain>,
    activation_db: Arc<CanonicalClient>,
    connection_options: CanonicalOptions,
    state_path: PathBuf,
    pub(crate) files: Arc<crate::canonical_files::FileSession>,
    database: String,
    namespace: String,
    issuing_store: Arc<std::sync::OnceLock<String>>,
    study_module_sha256: Arc<std::sync::OnceLock<String>>,
    // Fields drop in declaration order. The fixture's final drain must run
    // after both client aliases have relinquished this handle's continuation.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fixture_lifetime: Option<Arc<crate::testing::FixtureLifetime>>,
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
    /// Release this logical caller. Clones and extracted aliases share its admission.
    /// This neither cancels admitted work nor disconnects the physical clients.
    pub fn close(&self) {
        self.db.access.close_caller();
    }

    /// Construct independent caller authority over this still-open connection pair.
    /// Ordinary cloning preserves the existing caller instead.
    pub fn independent_caller(&self) -> Result<Self, CanonicalError> {
        self.db
            .access
            .independent_with(|access| self.with_access(access))
    }

    /// Admit a real operation before its first native or canonical effect.
    pub fn admit_activity(&self) -> Result<ConnectionActivity, CanonicalError> {
        self.files.require_current()?;
        self.db.access.admit()
    }

    /// Retain admitted work through required children and cleanup. An idle alias
    /// should use ordinary `Clone`, which grants no continuation authority.
    /// # Errors
    /// The caller or physical connection no longer admits new work.
    pub fn activity_store(&self) -> Result<Self, CanonicalError> {
        Ok(self.with_access(self.db.access.with_activity()?))
    }

    fn with_access(&self, access: lifetime::ConnectionAccess) -> Self {
        Self {
            db: Arc::new(self.db.with_access(access.clone())),
            activation_db: Arc::new(self.activation_db.with_access(access)),
            ..self.clone()
        }
    }

    pub(crate) fn selection_activity(
        &self,
        selection: &ProtectedSelection,
    ) -> Result<ConnectionActivity, CanonicalError> {
        match &selection.owner.store {
            Some(store) if self.db.same_driver(&store.db) => selection.owner.activity(),
            Some(_) => self.admit_activity(),
            None => self.admit_activity(),
        }
    }

    /// Heap owned by this handle's cloned values; shared physical clients are
    /// charged by their deployment owner rather than by every selection alias.
    pub(crate) fn handle_heap_bytes(&self) -> Option<usize> {
        let options = &self.connection_options;
        [
            self.state_path.capacity(),
            self.database.capacity(),
            self.namespace.capacity(),
            options.endpoint.capacity(),
            options.namespace.capacity(),
            options.database.capacity(),
            options.username.capacity(),
            options.password.capacity(),
            options.selection_username.capacity(),
            options.selection_password.capacity(),
            options.state_path.capacity(),
            options
                .initializer_nonce
                .as_ref()
                .map_or(0, String::capacity),
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
    }
    /// Drain local result readers and close both physical SDK drivers.
    /// This establishes local transport drain, not server scientific completion.
    pub async fn disconnect(&self) -> Result<(), CanonicalError> {
        let readers = self.result_read_drain.clone();
        let ordinary = self.db.clone();
        let activation = self.activation_db.clone();
        let files = self.files.clone();
        self.db
            .access
            .physical
            .close(
                move || async move { readers.close_and_drain().await },
                move || async move {
                    if let Some(owner) = files.registered_owner().await {
                        ordinary.drain_file_owner(owner).await?;
                    }
                    ordinary.disconnect().await.map_err(Into::into)
                },
                move || async move { activation.disconnect().await.map_err(Into::into) },
            )
            .await
    }
    /// Retry only exact automatic releases whose original response failed.
    /// Their retained activity remains authoritative after logical caller close.
    /// # Errors
    /// An original grant still cannot be acknowledged released.
    pub async fn recover_protection_releases(&self) -> Result<(), CanonicalError> {
        self.result_read_drain.recover().await
    }
    /// State selected by this authenticated deployment handle.
    pub fn deployment_state(&self) -> &Path {
        &self.state_path
    }
    /// One owner-local turn covers only a bounded RPC, never hydration or
    /// scientific work. Its original request clock includes queueing; dropping a
    /// waiter or request releases local ownership without inferring remote completion.
    pub(crate) async fn staging_query<F>(
        &self,
        problem: &str,
        future: F,
    ) -> Result<crate::transport::client::IndexedResults, CanonicalError>
    where
        F: IntoFuture<
            Output = Result<crate::transport::client::IndexedResults, surrealdb_types::Error>,
        >,
    {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.problem_turns
                    .run(
                        problem,
                        QueryEffect::Mutation,
                        original_deadline(REQUEST_TIMEOUT),
                        bounded_query(future),
                    )
                    .await
            })
            .await
    }

    /// Pace a complete protected RPC alongside staging under its original request
    /// clock. Definite conflicts rebuild the checked query; uncertain completion
    /// retains its typed failure. No turn escapes into hydration or scientific work.
    pub(crate) async fn protected_query<F, Q>(
        &self,
        problem: &str,
        effect: QueryEffect,
        operation: &'static str,
        build: F,
    ) -> Result<crate::transport::client::IndexedResults, CanonicalError>
    where
        F: FnMut() -> Result<Q, CanonicalError>,
        Q: IntoFuture<
            Output = Result<crate::transport::client::IndexedResults, surrealdb_types::Error>,
        >,
    {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let deadline = original_deadline(REQUEST_TIMEOUT);
                within_clock(deadline, async {
                    let result = crate::transport::retry::retry_with_state(
                        crate::transport::retry::RetryPolicy::Guarded,
                        operation,
                        build,
                        |mut build, _round| async move {
                            let result = self
                                .problem_turns
                                .run(problem, effect, deadline, async {
                                    bounded_query(build()?).await
                                })
                                .await;
                            #[cfg(all(test, feature = "canonical-tests"))]
                            if result
                                .as_ref()
                                .err()
                                .is_some_and(CanonicalError::is_conflict)
                            {
                                pacing_observation::conflict(problem);
                            }
                            (build, crate::transport::retry::Attempt::settled(result))
                        },
                    )
                    .await;
                    if let Err(error) = &result {
                        operation_failed(operation, error);
                    }
                    result.map_err(|error| error.exhaust_conflict(operation, GUARDED_ATTEMPTS))
                })
                .await
            })
            .await
    }

    /// Endpoint actually selected by this authenticated storage handle.
    pub fn endpoint(&self) -> &str {
        &self.connection_options.endpoint
    }
    /// Namespace actually selected by this authenticated storage handle.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    /// Exact configured canonical database selected by this deployment handle.
    pub fn database(&self) -> &str {
        &self.database
    }
    /// Persistent identity of the issuing database, installed once with its schema.
    pub fn issuing_store(&self) -> Result<&str, CanonicalError> {
        self.issuing_store.get().map(String::as_str).ok_or_else(|| {
            CanonicalError::Configuration(
                "issuing store identity unavailable; open the activated schema first".into(),
            )
        })
    }
    /// Refuse new application writes and native claims while the supervisor is draining.
    pub fn check_write_admission(&self) -> Result<(), CanonicalError> {
        self.ensure_writes()
    }
    pub(crate) fn ensure_writes(&self) -> Result<(), CanonicalError> {
        let admission = WriteAdmission::read(&self.state_path)?;
        admission.require_module(self.study_module_sha256.get())?;
        admission.require_writes(
            &self.state_path,
            &self.namespace,
            &self.database,
            self.connection_options.initializer_nonce.as_deref(),
        )
    }
    pub(crate) fn study_module_hash(&self) -> Result<&str, CanonicalError> {
        self.study_module_sha256
            .get()
            .map(String::as_str)
            .ok_or_else(|| {
                CanonicalError::Configuration("study module identity has not been opened".into())
            })
    }
    /// Connect to the supported authenticated native gRPC deployment.
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
        if endpoint.scheme() != "grpc"
            || !matches!(endpoint.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
        {
            return Err(CanonicalError::Configuration(
                "supported profile requires loopback native gRPC".into(),
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
        let access = db.access.clone();
        let activation_db = Arc::new(activation_db.with_access(access));
        Ok(Self {
            db,
            problem_turns: Arc::default(),
            #[cfg(any(test, feature = "test-support"))]
            fixture_lifetime: None,
            result_read_drain: Arc::new(crate::canonical_results::ResultReadDrain::default()),
            activation_db,
            connection_options: options.clone(),
            state_path: options.state_path.clone(),
            files: Arc::new(crate::canonical_files::FileSession::from_state(
                &options.state_path,
                &options.database,
            )?),
            database: options.database.clone(),
            namespace: options.namespace.clone(),
            issuing_store: Arc::default(),
            study_module_sha256: Arc::default(),
        })
    }
    /// Explicitly initialize a database through an operation-owned connection
    /// pair, then join its actual terminal transport work before disconnecting.
    /// No caller or reader receives this temporary connection's authority.
    pub async fn initialize_database(options: &CanonicalOptions) -> Result<(), CanonicalError> {
        let store = Self::connect(options).await?;
        let initialized = store.create().await;
        let drained = store.drain_owned_connection().await;
        initialized?;
        drained
    }
    /// Explicit schema creation; ordinary opening never mutates an unknown schema.
    pub async fn create(&self) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async { self.initialize_with(|| self.submit_initialization()).await })
            .await
    }
    async fn initialize_with<F, Q>(&self, submit: F) -> Result<(), CanonicalError>
    where
        F: FnMut() -> Q,
        Q: Future<Output = Result<crate::transport::client::IndexedResults, CanonicalError>>,
    {
        within_clock(original_deadline(ACTIVATION_REQUEST_TIMEOUT), async {
            crate::transport::retry::retry_with_state(
                crate::transport::retry::RetryPolicy::Initialization,
                "canonical::initialize",
                submit,
                |mut submit, _round| async move {
                    use crate::transport::retry::Attempt;
                    let preflight = async {
                        self.ensure_writes()?;
                        if !self.initialization_required().await? {
                            return Ok(false);
                        }
                        self.provision_context().await?;
                        Ok(true)
                    }
                    .await;
                    let outcome = match preflight {
                        Ok(true) => match Attempt::settled(
                            submit().await.and_then(complete_response).map(|_| ()),
                        ) {
                            Attempt::Done(result) => Attempt::Done(result.map_err(|error| {
                                error.exhaust_conflict("canonical::initialize", 8)
                            })),
                            retry => retry,
                        },
                        Ok(false) => Attempt::Done(Ok(())),
                        Err(error) => Attempt::Done(Err(error)),
                    };
                    (submit, outcome)
                },
            )
            .await?;
            // An acknowledged installation is never retried because readback failed.
            self.open().await
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
                Some(surrealdb_types::NotFoundError::Namespace { name }) if name == &self.namespace)
                    || matches!(error.not_found_details(),
                Some(surrealdb_types::NotFoundError::Database { name }) if name == &self.database) =>
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

    async fn submit_initialization(
        &self,
    ) -> Result<crate::transport::client::IndexedResults, CanonicalError> {
        self.ensure_writes()?;
        // Installing the complete schema is an atomic structural transition.
        // Its client deadline must match that role, rather than an ordinary read.
        let module = study_module::StudyModuleReceipt::read(&self.state_path)?;
        let query = self
            .activation_db
            .script(initialization_statement(&module)?)
            .bind(("study_module_backend", module.backend()?))
            .bind(("study_module_sha256", module.sha256.clone()))
            .bind(("interpretation", wire::INTERPRETATION))
            .bind(("schema_digest", wire::SCHEMA_DIGEST))
            .bind(("payload_incarnation", self.files.incarnation.clone()));
        tokio::time::timeout(ACTIVATION_REQUEST_TIMEOUT, query.into_future())
            .await
            .map_err(|_| CanonicalError::Timeout)
            .and_then(|response| response.map_err(CanonicalError::from))
    }
    /// Read and verify the installed interpretation without DDL.
    pub async fn open(&self) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
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
            if !marker.contains_key("issuing_store") {
                return Err(CanonicalError::Configuration("canonical store has no issuing-store identity; regenerate the selected owned store".into()));
            }
            let marker = wire::decode_canonical_interpretations(marker)?;
            if marker.payload_incarnation != self.files.incarnation { return Err(CanonicalError::Configuration("selected payload incarnation differs from catalog; regenerate or restore explicitly".into())); }
            if uuid::Uuid::parse_str(&marker.issuing_store).is_err() {
                return Err(CanonicalError::Configuration("persistent issuing store identity is invalid".into()));
            }
            if marker.interpretation != wire::INTERPRETATION {
                return Err(CanonicalError::Interpretation {
                    expected: wire::INTERPRETATION,
                    observed: marker.interpretation,
                });
            }
            let module=study_module::StudyModuleReceipt::read(&self.state_path)?;
            if marker.study_module_sha256!=module.sha256 {
                return Err(CanonicalError::Configuration("installed study module identity differs; explicit coordinated activation is required".into()));
            }
            self.validate_study_module(&module).await?;
            if let Some(existing)=self.study_module_sha256.get() {
                if existing!=&module.sha256 {return Err(CanonicalError::Quiesced);}
            } else if self.study_module_sha256.set(module.sha256.clone()).is_err() && self.study_module_sha256.get()!=Some(&module.sha256) {
                return Err(CanonicalError::Quiesced);
            }
            if marker.schema_digest != wire::SCHEMA_DIGEST {
                return Err(CanonicalError::Interpretation {
                    expected: wire::SCHEMA_DIGEST,
                    observed: marker.schema_digest,
                });
            }
            if let Some(existing) = self.issuing_store.get() {
                if existing != &marker.issuing_store {
                    return Err(CanonicalError::Configuration("persistent issuing store identity changed".into()));
                }
            } else if self.issuing_store.set(marker.issuing_store.clone()).is_err()
                && self.issuing_store.get() != Some(&marker.issuing_store) {
                return Err(CanonicalError::Configuration("persistent issuing store identity changed".into()));
            }
            Ok(())
        })
        .await
            }).await
    }
    /// Exact canonical revision retained by this protection.
    pub async fn revision(&self, id: &str) -> Result<Option<Revision>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let row: Option<Object> = request(
                    self.db
                        .control_query(
                            "SELECT * FROM ONLY type::record('canonical_revisions', $key);",
                        )
                        .bind(("key", id.to_owned())),
                )
                .await?
                .take(0)?;
                row.map(wire::decode_canonical_revisions)
                    .transpose()
                    .map_err(Into::into)
            })
            .await
    }
    /// Atomic head-CAS edit. The operation ID settles an acknowledgment lost after commit.
    pub async fn edit(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        edits: &[ObjectEdit],
    ) -> Result<Revision, CanonicalError> {
        self.edit_authorized(problem, expected, operation, edits, None, None)
            .await
    }
    /// Publish a selected variant under its original source-admission capability.
    pub async fn edit_with_study_admission(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        edits: &[ObjectEdit],
        authority: &crate::canonical_studies::StudyAdmissionAuthority,
    ) -> Result<Revision, CanonicalError> {
        if expected.is_some()
            || problem != authority.variant_problem()?
            || operation != authority.variant_operation()?
        {
            return Err(CanonicalError::Configuration(
                "selected source effect must use its original independent occurrence identity"
                    .into(),
            ));
        }
        self.edit_authorized(problem, expected, operation, edits, Some(authority), None)
            .await
    }
    pub(crate) async fn edit_project_authorized(
        &self,
        intent: &crate::canonical_projects::ProjectPublication,
        edits: &[ObjectEdit],
    ) -> Result<Revision, CanonicalError> {
        self.edit_authorized(
            intent.problem(),
            intent.expected(),
            intent.operation(),
            edits,
            None,
            Some(intent),
        )
        .await
    }
    async fn edit_authorized(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        edits: &[ObjectEdit],
        authority: Option<&crate::canonical_studies::StudyAdmissionAuthority>,
        project: Option<&crate::canonical_projects::ProjectPublication>,
    ) -> Result<Revision, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let deadline = original_deadline(ACTIVATION_REQUEST_TIMEOUT);
                let revision = transport::operation_scope(deadline.into_std(), async {
                    self.ensure_writes()?;
                    let stage = match self
                        .stage_edits(problem, expected, operation, edits)
                        .await?
                    {
                        crate::canonical_staging::StageOutcome::Acknowledged(revision) => {
                            if let Some(intent) = project {
                                let ack = self
                                    .project_acknowledgement(operation)
                                    .await?
                                    .ok_or(CanonicalError::OperationReused)?;
                                if ack.request.as_slice() != intent.request() {
                                    return Err(CanonicalError::OperationReused);
                                }
                            }
                            return Ok(revision);
                        }
                        crate::canonical_staging::StageOutcome::Closed(stage) => stage,
                    };
                    let result = self
                        .activate_source_stage_mutation(&stage, authority, project, deadline)
                        .await;
                    if authority.is_some()
                        && result
                            .as_ref()
                            .is_err_and(CanonicalError::proves_source_refusal)
                    {
                        self.abandon_source_stage(&stage).await?;
                    }
                    result
                })
                .await?;
                self.reclaim_source_after_commit(&revision.problem, deadline)
                    .await;
                Ok(revision)
            })
            .await
    }
    /// Activate only the caller's closed source stage under its generation and head fences.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn activate_source_stage(
        &self,
        stage: &crate::canonical_staging::ClosedStage,
    ) -> Result<Revision, CanonicalError> {
        self.activate_source_stage_authorized(stage, None).await
    }
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn activate_source_stage_authorized(
        &self,
        stage: &crate::canonical_staging::ClosedStage,
        authority: Option<&crate::canonical_studies::StudyAdmissionAuthority>,
    ) -> Result<Revision, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let deadline = original_deadline(ACTIVATION_REQUEST_TIMEOUT);
                let revision = self
                    .activate_source_stage_mutation(stage, authority, None, deadline)
                    .await?;
                self.reclaim_source_after_commit(&revision.problem, deadline)
                    .await;
                Ok(revision)
            })
            .await
    }
    // The enclosing edit owns its one postcommit page outside the mutation clock.
    async fn activate_source_stage_mutation(
        &self,
        stage: &crate::canonical_staging::ClosedStage,
        authority: Option<&crate::canonical_studies::StudyAdmissionAuthority>,
        project: Option<&crate::canonical_projects::ProjectPublication>,
        deadline: tokio::time::Instant,
    ) -> Result<Revision, CanonicalError> {
        let admission = authority
            .map(|authority| authority.encode())
            .transpose()?
            .unwrap_or(Value::None);
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(deadline, async {
                    let admission = &admission;
                    crate::transport::retry::retry_with_state(
                        crate::transport::retry::RetryPolicy::Guarded,
                        "canonical::activate_source_stage",
                        (),
                        |(), round| async move {
                            let result = async {
                                self.ensure_writes()?;
                                let query = self
                                    .activation_db
                                    .script(
                                        GuardedScript::new(Envelope::DeadlineOnly)
                                            .returning::<Value>(
                                                &crate::canonical_staging::bounded_edit_sql(EDIT),
                                                "NONE",
                                            )?,
                                    )
                                    .bind(("admission", admission.clone()))
                                    .bind((
                                        "project_intent",
                                        project
                                            .map(|intent| intent.encode())
                                            .transpose()?
                                            .unwrap_or(Value::None),
                                    ))
                                    .bind(("problem", stage.problem.clone()))
                                    .bind(("expected", stage.expected.clone()))
                                    .bind(("operation", stage.operation.clone()))
                                    .bind(("interpretation", wire::INTERPRETATION))
                                    .bind((
                                        "request",
                                        surrealdb_types::Bytes::from(stage.request.clone()),
                                    ))
                                    .bind(("request_digest", stage.request_digest.clone()))
                                    .bind((
                                        "generation",
                                        crate::canonical_codec::encode_uint(stage.generation)?,
                                    ));
                                let result = tokio::time::timeout(
                                    ACTIVATION_REQUEST_TIMEOUT,
                                    query.into_future(),
                                )
                                .await
                                .map_err(|_| CanonicalError::Timeout)
                                .and_then(|response| response.map_err(CanonicalError::from))
                                .and_then(complete_response);
                                #[cfg(any(test, feature = "test-support"))]
                                let result = if project.is_some()
                                    && result.is_ok()
                                    && crate::canonical_projects::simulate_reply_loss()
                                {
                                    Err(CanonicalError::IncompleteResponse)
                                } else {
                                    result
                                };
                                if let Err(error) = result {
                                    if error.is_capture_busy() {
                                        return Err(error);
                                    }
                                    // A lost acknowledgment or concurrent identical activation is settled
                                    // exclusively by the immutable operation receipt, never by the head.
                                    let acknowledgement = self.revision(&stage.operation).await;
                                    if let Ok(Some(revision)) = &acknowledgement {
                                        if revision.request.as_slice() != stage.request.as_slice() {
                                            return Err(CanonicalError::OperationReused);
                                        }
                                        if let Some(intent) = project
                                            && let Err(readback) =
                                                self.verify_project_publication_ack(intent).await
                                        {
                                            return Err(settlement::unresolved(
                                                "source-activation",
                                                &stage.operation,
                                                &stage.request,
                                                error,
                                                Some(readback),
                                            ));
                                        }
                                        return Ok(revision.clone());
                                    }
                                    let refused = matches!(acknowledgement, Ok(None))
                                        && error.source_transaction_refused();
                                    if matches!(acknowledgement, Ok(None))
                                        && error.is_conflict()
                                        && round.can_retry_conflict()
                                    {
                                        return Err(error);
                                    }
                                    let error = error.exhaust_conflict(
                                        "canonical::activate_source_stage",
                                        GUARDED_ATTEMPTS,
                                    );
                                    operation_failed("canonical::activate_source_stage", &error);
                                    let result = settlement::settle_exact(
                                        "source-activation",
                                        &stage.operation,
                                        &stage.request,
                                        Err(error),
                                        async { acknowledgement },
                                        |revision| {
                                            revision.request.as_slice() == stage.request.as_slice()
                                        },
                                        CanonicalError::source_transaction_refused,
                                    )
                                    .await;
                                    return result.map_err(|error| {
                                        if refused {
                                            CanonicalError::SourceEditRefused {
                                                problem: stage.problem.clone(),
                                                operation: stage.operation.clone(),
                                                source: Box::new(error),
                                            }
                                        } else {
                                            error
                                        }
                                    });
                                }
                                // The write completed, but a missing or failed receipt read still
                                // cannot invent an immutable acknowledgement for this activation.
                                return settlement::settle_exact(
                                    "source-activation",
                                    &stage.operation,
                                    &stage.request,
                                    Err(CanonicalError::IncompleteResponse),
                                    async {
                                        let revision = self.revision(&stage.operation).await?;
                                        if revision.as_ref().is_some_and(|revision| {
                                            revision.request.as_slice() == stage.request.as_slice()
                                        }) && let Some(intent) = project
                                        {
                                            self.verify_project_publication_ack(intent).await?;
                                        }
                                        Ok(revision)
                                    },
                                    |revision| {
                                        revision.request.as_slice() == stage.request.as_slice()
                                    },
                                    |_| false,
                                )
                                .await;
                            }
                            .await;
                            ((), crate::transport::retry::Attempt::settled(result))
                        },
                    )
                    .await
                })
                .await
            })
            .await
    }
    /// Explicit historical inventory, indexed by problem and revision interval.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn inventory(
        &self,
        revision: &Revision,
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let mut response = bounded_query(self.db.query("SELECT * FROM canonical_memberships WHERE problem = $problem AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) ORDER BY scope, name;")
            .bind(("problem", revision.problem.clone())).bind(("sequence", crate::canonical_codec::encode_uint(revision.sequence)?))).await?;
        let rows: Vec<Object> = response.take(0)?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
            }).await
    }
    /// Read an immutable version by exact identity.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn object(
        &self,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async { self.assemble_object(version).await })
            .await
    }
    async fn protection_grant_key(&self, problem: &str) -> Result<String, CanonicalError> {
        let mut response = bounded_query(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$problem); IF $authority=NONE OR $authority.incarnation=NONE { THROW 'PSE_REFUSAL:source_protection_authority_unavailable'; };","{incarnation:$authority.incarnation,deadline:time::micros()+30000000}")?).bind(("problem",problem.to_owned()))).await?;
        let mut grant = ResultSlot::<Option<Object>>::returned()
            .take(&mut response)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        let incarnation = crate::canonical_codec::decode_string(crate::canonical_codec::required(
            &mut grant,
            "incarnation",
        )?)?;
        let deadline = crate::canonical_codec::decode_int(crate::canonical_codec::required(
            &mut grant, "deadline",
        )?)?;
        Ok(format!(
            "pse.protection.v2:{incarnation}:{}:{deadline}",
            uuid::Uuid::new_v4()
        ))
    }

    /// Collect one exact released or expired grant only after its original
    /// acquisition window settles. Save [`ProtectedSelection::grant_key`] and
    /// its revision's problem identity before releasing the selection. A false
    /// result preserves residue while the original window or a live lease remains;
    /// repeating the same identity never issues a new grant or resets its floor.
    pub async fn settle_protection_grant(
        &self,
        problem: &str,
        key: &str,
    ) -> Result<bool, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        self.ensure_writes()?;
        let mut response = self.protected_query(&format!("pin:{key}"), QueryEffect::Mutation,
            "canonical::settle_protection_grant", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("","fn::pse_retention_v1::settle_protection($pse_rpc_expires_at,$problem,$key)")?).bind(("problem",problem.to_owned())).bind(("key",key.to_owned())))).await?;
        let mut row = ResultSlot::<Option<Object>>::returned().take(&mut response)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        Ok(crate::canonical_codec::decode_boolean(
            crate::canonical_codec::required(&mut row, "complete")?,
        )?)
            }).await
    }

    /// Protect before off-transaction scientific work. Server time controls expiration.
    pub async fn protect(
        &self,
        revision: Revision,
        lifetime: std::time::Duration,
    ) -> Result<ProtectedSelection, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                // Read protections are lease bookkeeping, including gated restore
                // verification. Scientific mutation and reclamation remain quiesced.
                let micros = i64::try_from(lifetime.as_micros()).map_err(|_| {
                    CanonicalError::Configuration("protection lifetime overflow".into())
                })?;
                if micros <= 0 {
                    return Err(CanonicalError::Configuration(
                        "positive protection lifetime required".into(),
                    ));
                }
                let deadline = original_deadline(REQUEST_TIMEOUT);
                let grant_key =
                    within_clock(deadline, self.protection_grant_key(&revision.problem)).await?;
                self.result_read_drain.retain()?;
                let executor = tokio::runtime::Handle::current();
                // A fixture's physical clients outlive the caller's short-lived test
                // executor. Their automatic protection releases must have that same
                // retained lifetime, including when a test abandons its result reader.
                #[cfg(any(test, feature = "test-support"))]
                let executor = self
                    .fixture_lifetime
                    .as_ref()
                    .map_or(executor.clone(), |fixture| fixture.cleanup_executor());
                let selection = ProtectedSelection {
                    owner: Arc::new(ProtectionOwner {
                        key: grant_key,
                        revision,
                        store: Some(self.clone()),
                        activity: std::sync::Mutex::new(Some(self.admit_activity()?)),
                        executor: Some(executor),
                        released: std::sync::atomic::AtomicBool::new(false),
                        metadata_owner: std::sync::OnceLock::new(),
                    }),
                };
                let store = self.clone();
                // Submission owns the protection before any RPC. Abandoning the caller detaches
                // this task; its eventual result still owns cleanup through acknowledgement.
                tokio::spawn(inherit_activity(transport::inherit_response_ownership(
                    async move {
                        within_clock(deadline, store.acquire_protection(selection, micros)).await
                    },
                )))
                .await
                .map_err(|error| {
                    CanonicalError::Configuration(format!(
                        "protection acquisition task failed: {error}"
                    ))
                })?
            })
            .await
    }
    async fn acquire_protection(
        &self,
        selection: ProtectedSelection,
        micros: i64,
    ) -> Result<ProtectedSelection, CanonicalError> {
        let key = selection.key();
        let revision = selection.revision();
        let mut response = self
            .protected_query(
                &revision.problem,
                QueryEffect::Mutation,
                "canonical::protect",
                || {
                    Ok(self
                        .db
                        .script(
                            GuardedScript::new(Envelope::DeadlineOnly)
                                .returning::<Option<Object>>(ACQUIRE_PROTECTION, "$rpc_result")?,
                        )
                        .bind(("problem", revision.problem.clone()))
                        .bind(("revision", revision.key.clone()))
                        .bind(("key", key.to_owned()))
                        .bind(("lifetime", micros)))
                },
            )
            .await?;
        let actual = ResultSlot::<Option<Object>>::returned().take(&mut response)?;
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
    async fn renew_protection(
        &self,
        selection: &ProtectedSelection,
        lifetime: std::time::Duration,
    ) -> Result<(), CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity
            .scope(async {
                let micros = i64::try_from(lifetime.as_micros()).map_err(|_| {
                    CanonicalError::Configuration("protection lifetime overflow".into())
                })?;
                if micros <= 0 {
                    return Err(CanonicalError::Configuration(
                        "positive protection lifetime required".into(),
                    ));
                }
                let mut response = self
                    .protected_query(
                        &format!("pin:{}", selection.key()),
                        QueryEffect::Mutation,
                        "canonical::renew_protection",
                        || {
                            Ok(self
                                .db
                                .script(
                                    GuardedScript::new(Envelope::ProtectedAdmission { selection })
                                        .returning::<Option<bool>>(
                                            crate::canonical_results::RENEW_PROTECTION,
                                            "true",
                                        )?,
                                )
                                .bind(("lifetime", micros)))
                        },
                    )
                    .await?;
                let renewed: Option<bool> =
                    ResultSlot::<Option<bool>>::returned().take(&mut response)?;
                if renewed != Some(true) {
                    return Err(CanonicalError::IncompleteResponse);
                }
                Ok(())
            })
            .await
    }
    /// Resolve a bounded namespace/name premise in the exact protected revision.
    pub async fn select_names(
        &self,
        selection: &ProtectedSelection,
        scope: &str,
        names: &[String],
    ) -> Result<Vec<Membership>, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity.scope(async {
        if names.len() > 256 {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut response = self.protected_query(&selection.revision().problem, QueryEffect::Read, "canonical::select_names", || Ok(self.db.script(GuardedScript::new(Envelope::ProtectedRead{selection,recheck:Recheck::PinOrTicket})
            .returning::<Vec<Object>>("","SELECT * FROM canonical_memberships WHERE problem = $problem AND scope = $scope AND name IN $names AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) TIMEOUT $pse_rpc_timeout")?)
            .bind(("scope",scope.to_owned())).bind(("names",names.to_vec())))).await?;
        let rows: Vec<Object> = ResultSlot::<Vec<Object>>::returned().take(&mut response)?;
        rows.into_iter()
            .map(wire::decode_canonical_memberships)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
            }).await
    }
    fn product_descriptor_row(
        &self,
        product: &pse_model::generated::runtime::canonical_products::Row,
    ) -> Result<pse_model::generated::runtime::canonical_product_descriptors::Row, CanonicalError>
    {
        descriptor_for_product(product, self.issuing_store()?)
    }
    pub(crate) fn product_association_row(
        &self,
        product: &pse_model::generated::runtime::canonical_products::Row,
    ) -> Result<pse_model::generated::runtime::canonical_product_associations::Row, CanonicalError>
    {
        let descriptor = self.product_descriptor_row(product)?;
        Ok(
            pse_model::generated::runtime::canonical_product_associations::Row {
                key: product.key.clone(),
                issuing_store: descriptor.issuing_store,
                problem: product.problem.clone(),
                revision: product.revision.clone(),
                request: product.request.clone(),
                payload: product.payload.clone(),
                dependencies: product.dependencies.clone(),
                content: descriptor.content,
                receipt: descriptor.receipt,
                descriptor: descriptor.key,
                producer: product.producer.clone(),
                producer_class: product.producer_class,
                interpretation: product.interpretation.clone(),
            },
        )
    }
    /// Product and revision roots become durable before the caller explicitly ends
    /// the selection protection. Multiple body products may share the same selection.
    pub(crate) async fn admit_product(
        &self,
        selection: &ProtectedSelection,
        product: &pse_model::generated::runtime::canonical_products::Row,
        stage: &crate::canonical_staging::ClosedStage,
    ) -> Result<String, CanonicalError> {
        let activity = self.selection_activity(selection)?;
        activity.scope(async {
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
        let descriptor = self.product_descriptor_row(product)?;
        let association = self.product_association_row(product)?;
        let blob = crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?;
        let receipt = blob.receipt.as_ref().ok_or(CanonicalError::PayloadLimit)?;
        let content = pse_model::generated::runtime::canonical_body_contents::Row { key: blob.version.clone(), version: blob.version.clone(), digest: blob.digest.clone(), bytes: blob.payload_bytes as u64, producer: product.producer.clone(), interpretation: wire::INTERPRETATION.into(), format: crate::canonical_staging::BODY_CONTENT_KIND.into() };
        let selection_receipt = pse_model::generated::runtime::canonical_selection_receipts::Row { key: receipt.version.clone(), version: receipt.version.clone(), digest: receipt.digest.clone(), bytes: receipt.bytes as u64, interpretation: wire::INTERPRETATION.into(), format: crate::canonical_staging::SELECTION_RECEIPT_KIND.into() };
        let descriptor = &descriptor;
        let association = &association;
        let content = &content;
        let selection_receipt = &selection_receipt;
        crate::transport::retry::retry_with_state(
            crate::transport::retry::RetryPolicy::Guarded,
            "canonical::admit_product",
            (),
            |(), round| async move {
                let result = async {
            self.ensure_writes()?;
            let result = self
                .staging_query(
                    &selection.revision().problem,
                    self.db
                        .script(GuardedScript::new(Envelope::ProtectedAdmission{selection})
                            .returning::<Value>(ADMIT_PRODUCT,"$rpc_result")?)
                        .bind(("product", wire::encode_canonical_product_associations(&association)?))
                        .bind(("descriptor", wire::encode_canonical_product_descriptors(&descriptor)?))
                        .bind(("content", wire::encode_canonical_body_contents(&content)?))
                        .bind(("receipt", wire::encode_canonical_selection_receipts(&selection_receipt)?))
                        .bind(("operation", stage.operation.clone()))
                        .bind((
                            "generation",
                            crate::canonical_codec::encode_uint(stage.generation)?,
                        ))
                        .bind(("request_digest", stage.request_digest.clone()))
                )
                .await
                .and_then(checked);
            match result {
                Ok(mut response) => {
                    return Ok(crate::canonical_codec::decode_string(
                        ResultSlot::<Value>::returned().take(&mut response)?,
                    )?);
                }
                Err(error) => {
                    if error.is_capture_busy() { return Err(error); }
                    // This immutable acknowledgment remains meaningful even if the
                    // preparation pin expired after the successful commit.
                    if let Ok(Some(key)) = self.product_acknowledged(product).await {
                        return Ok(key);
                    }
                    if matches!(&error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(surrealdb_types::QueryError::TransactionConflict)))
                        && round.can_retry_conflict()
                    {
                        return Err(error);
                    }
                    let error = error.exhaust_conflict("canonical::admit_product", GUARDED_ATTEMPTS);
                    operation_failed("canonical::admit_product", &error);
                    return Err(error);
                }
            }
                }.await;
                ((), crate::transport::retry::Attempt::settled(result))
            },
        ).await
        }).await
            }).await
    }
    pub(crate) async fn product_acknowledged(
        &self,
        product: &pse_model::generated::runtime::canonical_products::Row,
    ) -> Result<Option<String>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let descriptor = self.product_descriptor_row(product)?;
                let mut response = self
                    .protected_query(
                        &product.problem,
                        QueryEffect::Read,
                        "canonical::product_acknowledged",
                        || {
                            Ok(self
                                .db
                                .script(
                                    GuardedScript::new(Envelope::DeadlineOnly)
                                        .returning::<Value>(PRODUCT_ACK, "$rpc_result")?,
                                )
                                .bind(("problem", product.problem.clone()))
                                .bind((
                                    "descriptor",
                                    wire::encode_canonical_product_descriptors(&descriptor)?,
                                )))
                        },
                    )
                    .await?;
                let value = ResultSlot::<Value>::returned().take(&mut response)?;
                if matches!(value, Value::None) {
                    return Ok(None);
                }
                Ok(Some(crate::canonical_codec::decode_string(value)?))
            })
            .await
    }
    /// End a selection protection through its issuing store and retention conflict guard.
    pub async fn release(&self, selection: &ProtectedSelection) -> Result<(), CanonicalError> {
        if selection
            .owner
            .released
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Ok(());
        }
        let activity = selection.owner.activity()?;
        transport::optional_response_ownership_scope(
            selection.owner.metadata_owner.get().cloned(),
            activity.scope(async {
                let store = selection.owner.store.as_ref().ok_or_else(|| {
                    CanonicalError::Configuration("protection issuing store unavailable".into())
                })?;
                store
                    .release_protection(selection.revision(), selection.key())
                    .await?;
                selection.owner.acknowledge_release();
                Ok(())
            }),
        )
        .await
    }
    pub(crate) async fn release_protection(
        &self,
        revision: &Revision,
        key: &str,
    ) -> Result<(), CanonicalError> {
        // Quiescing scientific writes must still let in-flight readers drain.
        // A submitted acquisition can outlive its failed local response. Record
        // release even before its row exists so that a later acquisition refuses it.
        let sequence = crate::canonical_codec::encode_uint(revision.sequence)?;
        self.protected_query(
            &format!("pin:{key}"),
            QueryEffect::Mutation,
            "canonical::release",
            || {
                Ok(self
                    .db
                    .script(
                        GuardedScript::new(Envelope::DeadlineOnly)
                            .returning::<Value>(RELEASE_PROTECTION, "NONE")?,
                    )
                    .bind(("problem", revision.problem.clone()))
                    .bind(("revision", revision.key.clone()))
                    .bind(("sequence", sequence.clone()))
                    .bind(("protection", key.to_owned())))
            },
        )
        .await?;
        Ok(())
    }
    /// Join physical owners of an isolated fixture under its original same-host
    /// monotonic expiry. Logical caller close alone does not establish this fact.
    #[doc(hidden)]
    pub async fn drain_fixture_connection_before(
        &self,
        expiry_monotonic_ns: u64,
    ) -> Result<(), CanonicalError> {
        if !self.database.starts_with("canonical_test_") {
            return Err(CanonicalError::Configuration(
                "physical fixture drain requires an isolated database".into(),
            ));
        }
        // Sampling the local instant first can only shorten the mapped offer.
        let sampled = tokio::time::Instant::now();
        let now = transport::monotonic_nanoseconds()?;
        if expiry_monotonic_ns <= now {
            return Err(CanonicalError::Timeout);
        }
        let deadline = sampled + std::time::Duration::from_nanos(expiry_monotonic_ns - now);
        self.drain_owned_connection_until(deadline).await
    }
    /// Final fixture ownership may join terminal SDK callbacks after their reply
    /// woke the caller. Seal new admission before waiting for actual activity;
    /// this path grants no cancellation or early close of escaped protected reads.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) async fn drain_fixture_connection(&self) -> Result<(), CanonicalError> {
        self.drain_owned_connection().await
    }
    /// Only private initialization or final fixture ownership establishes that
    /// no caller can escape. Ordinary disconnect keeps its live-activity refusal.
    async fn drain_owned_connection(&self) -> Result<(), CanonicalError> {
        self.drain_owned_connection_until(original_deadline(REQUEST_TIMEOUT))
            .await
    }
    /// Seal terminal-owner admission, join dropped-reader releases and SDK callbacks,
    /// then disconnect under the original shutdown allowance. Live protected readers
    /// still refuse; this grants no cancellation or early release of escaped work.
    /// Ordinary shared callers should retain `disconnect`'s immediate busy refusal.
    /// # Errors
    /// Live readers, original release failures, deadline expiry or SDK drain failure.
    pub async fn drain_owned_connection_until(
        &self,
        deadline: tokio::time::Instant,
    ) -> Result<(), CanonicalError> {
        within_clock(deadline, async {
            self.result_read_drain.close_and_drain().await?;
            self.db
                .access
                .physical
                .seal_and_wait_for_idle(deadline)
                .await?;
            self.disconnect().await
        })
        .await
    }
    #[cfg(any(test, feature = "test-support"))]
    fn seal_final_fixture_owner(&self) -> bool {
        self.db.access.physical.seal_if_final_fixture(|| {
            !self
                .fixture_lifetime
                .as_ref()
                .is_some_and(|lifetime| Arc::strong_count(lifetime) > 1)
        })
    }
    /// Drain a registered fixture locally; runner disposition owns disposal.
    #[cfg(any(test, feature = "test-support"))]
    pub async fn remove_isolated_fixture(&self) -> Result<(), CanonicalError> {
        let offered = original_deadline(REQUEST_TIMEOUT);
        let deadline = match &self.fixture_lifetime {
            Some(lifetime) => lifetime.settlement_deadline(offered, false)?,
            None => crate::testing::SettlementDeadline::new(offered)?,
        };
        if !self.database.starts_with("canonical_test_") {
            return Err(CanonicalError::Configuration(
                "fixture removal requires an isolated test database".into(),
            ));
        }
        let mut removed = match &self.fixture_lifetime {
            Some(lifetime) => Some(
                within_clock(deadline.local, async { Ok(lifetime.removed.lock().await) }).await?,
            ),
            None => None,
        };
        if removed.as_deref().is_some_and(|removed| *removed) {
            return Ok(());
        }
        if !self.seal_final_fixture_owner() {
            // Remaining registered borrowers own eventual drain. No new caller
            // can pass between the final-owner decision and admission seal.
            return within_clock(deadline.local, self.result_read_drain.drain()).await;
        }
        if let Some(lifetime) = &self.fixture_lifetime {
            let deadline = lifetime.settlement_deadline(deadline.local, true)?;
            lifetime.settle(Some(self), deadline).await?;
        } else {
            within_clock(deadline.local, self.drain_fixture_connection()).await?;
        }
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

fn initialization_statement(
    module: &study_module::StudyModuleReceipt,
) -> Result<crate::canonical_script::PreparedScript<Value>, CanonicalError> {
    let body = format!(
        "{}{}\nCREATE canonical_interpretations:current SET key = 'current', interpretation = $interpretation, schema_digest = $schema_digest, study_module_sha256 = $study_module_sha256, issuing_store = <string>rand::uuid::v4(), payload_incarnation = $payload_incarnation;\nCREATE canonical_file_admission:current SET key='current',storage_incarnation=$payload_incarnation,frozen=false,gc_frozen=false;",
        module.declarations(false),
        wire::SCHEMA,
    );
    GuardedScript::new(Envelope::Installation).returning::<Value>(&body, "NONE")
}

async fn connect_client(
    address: std::net::SocketAddr,
    options: &CanonicalOptions,
    query_timeout: std::time::Duration,
    select_context: bool,
) -> Result<Arc<CanonicalClient>, CanonicalError> {
    let deadline = original_deadline(REQUEST_TIMEOUT);
    within_clock(deadline, async {
        let db = within_clock(deadline, async {
            crate::transport::client::Client::connect(address, MESSAGE_BYTES)
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

pub(crate) fn descriptor_for_product(
    product: &pse_model::generated::runtime::canonical_products::Row,
    issuing_store: &str,
) -> Result<pse_model::generated::runtime::canonical_product_descriptors::Row, CanonicalError> {
    let blob = crate::canonical_staging::ProductBlob::decode(product.payload.as_slice())?;
    let receipt = blob.receipt.as_ref().ok_or_else(|| {
        CanonicalError::Configuration("current product requires a selection receipt".into())
    })?;
    if product.interpretation != wire::INTERPRETATION
        || blob.producer != product.producer
        || !product.dependencies.is_empty()
    {
        return Err(CanonicalError::Configuration(
            "product descriptor reconstruction meaning differs".into(),
        ));
    }
    let issuing_store = issuing_store.to_owned();
    let key = pse_ids::derive_hash(
        pse_ids::Frame::CanonicalProductDescriptorV1,
        &[
            issuing_store.as_bytes(),
            product.problem.as_bytes(),
            product.request.as_slice(),
            product.payload.as_slice(),
            product.dependencies.as_slice(),
            product.producer.as_bytes(),
            product.producer_class.as_str().as_bytes(),
            product.interpretation.as_bytes(),
        ],
    )
    .to_hex();
    Ok(
        pse_model::generated::runtime::canonical_product_descriptors::Row {
            key,
            issuing_store,
            problem: product.problem.clone(),
            request: product.request.clone(),
            payload: product.payload.clone(),
            dependencies: product.dependencies.clone(),
            content: blob.version,
            receipt: receipt.version.clone(),
            producer: product.producer.clone(),
            producer_class: product.producer_class,
            interpretation: product.interpretation.clone(),
            association: None,
        },
    )
}

pub(crate) fn decode_product_association(
    row: Object,
) -> Result<pse_model::generated::runtime::canonical_products::Row, CanonicalError> {
    let association = wire::decode_canonical_product_associations(row)?;
    let blob = crate::canonical_staging::ProductBlob::decode(association.payload.as_slice())?;
    let receipt = blob.receipt.as_ref().ok_or_else(|| {
        CanonicalError::Configuration("active association requires split descriptors".into())
    })?;
    let key = pse_ids::derive_hash(
        pse_ids::Frame::CanonicalProductDescriptorV1,
        &[
            association.issuing_store.as_bytes(),
            association.problem.as_bytes(),
            association.request.as_slice(),
            association.payload.as_slice(),
            association.dependencies.as_slice(),
            association.producer.as_bytes(),
            association.producer_class.as_str().as_bytes(),
            association.interpretation.as_bytes(),
        ],
    )
    .to_hex();
    if association.descriptor != key
        || association.content != blob.version
        || association.receipt != receipt.version
        || association.producer != blob.producer
        || association.interpretation != wire::INTERPRETATION
        || !association.dependencies.is_empty()
        || uuid::Uuid::parse_str(&association.issuing_store).is_err()
    {
        return Err(CanonicalError::Configuration(
            "immutable product association descriptor differs".into(),
        ));
    }
    Ok(pse_model::generated::runtime::canonical_products::Row {
        key: association.key,
        problem: association.problem,
        revision: association.revision,
        request: association.request,
        payload: association.payload,
        dependencies: association.dependencies,
        producer: association.producer,
        producer_class: association.producer_class,
        interpretation: association.interpretation,
    })
}

const PRODUCT_ACK: &str = r#"
SELECT * FROM type::record('canonical_guards','retention:'+$problem) FOR UPDATE;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $saved = SELECT * FROM ONLY type::record('canonical_product_descriptors',$descriptor.key);
IF $saved != NONE AND ($saved.issuing_store != $descriptor.issuing_store OR $saved.problem != $descriptor.problem OR $saved.request != $descriptor.request OR $saved.payload != $descriptor.payload OR $saved.dependencies != $descriptor.dependencies OR $saved.content != $descriptor.content OR $saved.receipt != $descriptor.receipt OR $saved.producer != $descriptor.producer OR $saved.producer_class != $descriptor.producer_class OR $saved.interpretation != $descriptor.interpretation) { THROW 'PSE_REFUSAL:immutable_descriptor_hash_collision'; };
LET $association = IF $saved = NONE OR $saved.association = NONE { NONE } ELSE { SELECT * FROM ONLY type::record('canonical_product_associations',$saved.association) };
LET $root = IF $association = NONE { NONE } ELSE { SELECT * FROM ONLY type::record('canonical_roots',$association.key) FOR UPDATE };
IF $association != NONE AND ($association.descriptor != $descriptor.key OR $association.issuing_store != $descriptor.issuing_store OR $association.problem != $descriptor.problem OR $association.request != $descriptor.request OR $association.payload != $descriptor.payload OR $association.dependencies != $descriptor.dependencies OR $association.content != $descriptor.content OR $association.receipt != $descriptor.receipt OR $association.producer != $descriptor.producer OR $association.producer_class != $descriptor.producer_class OR $association.interpretation != $descriptor.interpretation) { THROW 'PSE_REFUSAL:descriptor_association_equality_differs'; };
LET $rpc_result = IF $root != NONE AND $root.key = $association.key AND $root.owner_kind = 'product' AND $root.owner = $association.key AND $root.problem = $association.problem AND $root.revision = $association.revision { $association.key } ELSE { NONE };
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
"#;

const ADMIT_PRODUCT: &str = r#"
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $guards = array::sort([$content.version,$receipt.version]).map(|$v| type::record('canonical_guards','version:'+$v));
FOR $guard IN $guards { SELECT * FROM $guard FOR UPDATE; };
LET $stage = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $operation != $product.key OR $stage = NONE OR $stage.problem != $problem OR $stage.request_digest != $request_digest OR $stage.generation != $generation OR $stage.closed = false OR $stage.activated OR $stage.abandoned OR $stage.expires_at <= time::micros() OR $stage.edit_count != 2dec OR $stage.expected_head != NONE { THROW 'PSE_REFUSAL:closed_product_staging_manifest_fenced'; };
FOR $part IN [{ordinal:0dec,kind:'pse.body-content.v1',record:$content},{ordinal:1dec,kind:'pse.selection-receipt.v1',record:$receipt}] {
    LET $child = SELECT * FROM ONLY type::record('canonical_staged_edits',$operation+':'+<string>$part.ordinal);
    LET $manifest = SELECT * FROM ONLY type::record('canonical_version_manifests',$part.record.version);
    IF $child = NONE OR $child.stage != $operation OR $child.ordinal != $part.ordinal OR $child.version != $part.record.version OR $child.logical != $part.kind OR $child.scope != $part.kind OR $child.name != $part.kind OR $manifest = NONE OR $manifest.closed = false OR $manifest.logical != $part.kind OR $manifest.kind != $part.kind OR $manifest.interpretation != $product.interpretation OR $manifest.reference_count != 0dec OR $manifest.payload_digest != $part.record.digest OR $manifest.payload_len != $part.record.bytes { THROW 'PSE_REFUSAL:product_physical_part_association_invalid'; };
};
IF $product.issuing_store != (SELECT * FROM ONLY canonical_interpretations:current).issuing_store OR $descriptor.issuing_store != $product.issuing_store OR $descriptor.key != $product.descriptor OR $descriptor.producer_class != $product.producer_class OR $descriptor.content != $content.key OR $descriptor.receipt != $receipt.key OR $product.content != $content.key OR $product.receipt != $receipt.key OR $content.version != $content.key OR $receipt.version != $receipt.key OR $content.producer != $product.producer OR $content.interpretation != $product.interpretation OR $receipt.interpretation != $product.interpretation OR $content.format != 'pse.body-content.v1' OR $receipt.format != 'pse.selection-receipt.v1' { THROW 'PSE_REFUSAL:product_physical_meaning_differs'; };
LET $old_content = SELECT * FROM ONLY type::record('canonical_body_contents',$content.key);
IF $old_content = NONE { CREATE type::record('canonical_body_contents',$content.key) CONTENT $content; } ELSE {
    IF $old_content.version != $content.version OR $old_content.digest != $content.digest OR $old_content.bytes != $content.bytes OR $old_content.producer != $content.producer OR $old_content.interpretation != $content.interpretation OR $old_content.format != $content.format { THROW 'PSE_REFUSAL:immutable_body_content_collision'; };
};
LET $old_receipt = SELECT * FROM ONLY type::record('canonical_selection_receipts',$receipt.key);
IF $old_receipt = NONE { CREATE type::record('canonical_selection_receipts',$receipt.key) CONTENT $receipt; } ELSE {
    IF $old_receipt.version != $receipt.version OR $old_receipt.digest != $receipt.digest OR $old_receipt.bytes != $receipt.bytes OR $old_receipt.interpretation != $receipt.interpretation OR $old_receipt.format != $receipt.format { THROW 'PSE_REFUSAL:immutable_selection_receipt_collision'; };
};
LET $saved = SELECT * FROM ONLY type::record('canonical_product_descriptors',$descriptor.key) FOR UPDATE;
IF $saved != NONE AND ($saved.issuing_store != $descriptor.issuing_store OR $saved.problem != $descriptor.problem OR $saved.request != $descriptor.request OR $saved.payload != $descriptor.payload OR $saved.dependencies != $descriptor.dependencies OR $saved.content != $descriptor.content OR $saved.receipt != $descriptor.receipt OR $saved.producer != $descriptor.producer OR $saved.producer_class != $descriptor.producer_class OR $saved.interpretation != $descriptor.interpretation) { THROW 'PSE_REFUSAL:immutable_descriptor_hash_collision'; };
LET $equivalent = IF $saved = NONE OR $saved.association = NONE { NONE } ELSE { SELECT * FROM ONLY type::record('canonical_product_associations',$saved.association) };
LET $equivalent_root = IF $equivalent = NONE { NONE } ELSE { SELECT * FROM ONLY type::record('canonical_roots',$equivalent.key) FOR UPDATE };
IF $equivalent != NONE AND ($equivalent.descriptor != $descriptor.key OR $equivalent.issuing_store != $descriptor.issuing_store OR $equivalent.problem != $descriptor.problem OR $equivalent.request != $descriptor.request OR $equivalent.payload != $descriptor.payload OR $equivalent.dependencies != $descriptor.dependencies OR $equivalent.content != $descriptor.content OR $equivalent.receipt != $descriptor.receipt OR $equivalent.producer != $descriptor.producer OR $equivalent.producer_class != $descriptor.producer_class OR $equivalent.interpretation != $descriptor.interpretation) { THROW 'PSE_REFUSAL:descriptor_association_equality_differs'; };
LET $eligible = $equivalent_root != NONE AND $equivalent_root.key = $equivalent.key AND $equivalent_root.problem = $equivalent.problem AND $equivalent_root.revision = $equivalent.revision AND $equivalent_root.owner_kind = 'product' AND $equivalent_root.owner = $equivalent.key;
IF $eligible = false {
    LET $existing = SELECT * FROM ONLY type::record('canonical_product_associations',$product.key);
    IF $existing = NONE {
        CREATE type::record('canonical_product_associations',$product.key) CONTENT $product;
        fn::pse_execution_v1::source_creator($pse_rpc_expires_at,$problem,$product.key);
        CREATE type::record('canonical_roots',$product.key) SET key=$product.key,problem=$problem,revision=$product.revision,sequence=$sequence,owner_kind='product',owner=$product.key;
    } ELSE {
        IF $existing.issuing_store != $product.issuing_store OR $existing.problem != $product.problem OR $existing.revision != $product.revision OR $existing.descriptor != $product.descriptor OR $existing.request != $product.request OR $existing.payload != $product.payload OR $existing.dependencies != $product.dependencies OR $existing.content != $product.content OR $existing.receipt != $product.receipt OR $existing.producer != $product.producer OR $existing.producer_class != $product.producer_class OR $existing.interpretation != $product.interpretation { THROW 'PSE_REFUSAL:immutable_product_identity_reused'; };
        LET $root = SELECT * FROM ONLY type::record('canonical_roots',$product.key) FOR UPDATE;
        IF $root=NONE OR $root.problem!=$existing.problem OR $root.revision!=$existing.revision OR $root.owner_kind!='product' OR $root.owner!=$existing.key { THROW 'PSE_REFUSAL:product_retention_explicitly_released_or_changed'; };
    };
    IF $saved = NONE { CREATE type::record('canonical_product_descriptors',$descriptor.key) CONTENT $descriptor; };
    UPDATE type::record('canonical_product_descriptors',$descriptor.key) SET association=$product.key;
};
UPDATE type::record('canonical_stages',$operation) SET activated=true;
UPSERT $stage_guard SET key='stage:'+$operation,generation=(generation ?? 0dec)+1dec;
FOR $key IN [$content.version,$receipt.version] { UPSERT type::record('canonical_guards','version:'+$key) SET key='version:'+$key,generation=(generation ?? 0dec)+1dec; };
fn::pse_execution_v1::touch($pse_rpc_expires_at,'source-creators:'+$problem);
LET $rpc_result = IF $eligible { $equivalent.key } ELSE { $product.key };
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $pin.expires_at <= time::micros() OR $stage.expires_at <= time::micros() { THROW 'PSE_REFUSAL:product_publication_authority_expired'; };
"#;

#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) fn test_source_edit_sql() -> Result<String, CanonicalError> {
    crate::canonical_script::test_script_sql(
        &crate::canonical_staging::bounded_edit_sql(EDIT),
        "NONE",
    )
}

const EDIT: &str = r#"
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $head_guard = type::record('canonical_guards', 'head:' + $problem);
SELECT * FROM $head_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $ack = SELECT * FROM ONLY type::record('canonical_revisions', $operation);
IF $ack != NONE { THROW 'PSE_REFUSAL:operation_identity_already_committed_settle_immutable_revision'; };
LET $admission_owner=IF $admission=NONE {NONE} ELSE {fn::pse_study_v1::admission_authority($pse_rpc_expires_at,$admission)};
LET $stage = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $stage = NONE OR $stage.problem != $problem OR ($stage.expected_head ?? NONE) != $expected OR $stage.request_digest != $request_digest OR $stage.generation != $generation OR $stage.closed = false OR $stage.abandoned OR $stage.activated OR $stage.expires_at <= time::micros() { THROW 'PSE_REFUSAL:closed_staging_manifest_fenced_or_unavailable'; };
fn::pse_project_v1::source_guard($pse_rpc_expires_at,$problem,$operation,$project_intent);
LET $head = SELECT * FROM ONLY type::record('canonical_problems', $problem);
IF ($head.head ?? NONE) != $expected { THROW 'PSE_REFUSAL:head_compare_and_set_conflict'; };
LET $changes = SELECT * FROM canonical_staged_edits WHERE stage = $operation ORDER BY ordinal LIMIT /* EDIT_SCAN_LIMIT */;
IF array::len($changes) != $stage.edit_count OR array::len($changes) > /* EDIT_LIMIT */ { THROW 'PSE_REFUSAL:closed_staging_metadata_coverage_mismatch'; };
LET $sequence = ($head.sequence ?? 0dec) + 1dec;
UPSERT type::record('canonical_problems', $problem) SET key = $problem, head = $operation, sequence = $sequence;
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
IF array::len(array::distinct($names)) != array::len($names) { THROW 'PSE_REFUSAL:name_already_exists_within_incoming_revision'; };
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
    IF array::len($manifests) != array::len($creates) { THROW 'PSE_REFUSAL:staged_immutable_source_manifest_coverage_invalid'; };
    FOR $change IN $creates {
        LET $manifest = $manifests.find(|$header| $header.key = $change.version);
        IF $manifest = NONE OR $manifest.closed = false OR $manifest.logical != $change.logical OR $manifest.interpretation != $interpretation { THROW 'PSE_REFUSAL:staged_immutable_source_manifest_invalid'; };
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
            IF $replacement = NONE OR $survivor.logical != $replacement.logical { THROW 'PSE_REFUSAL:name_already_exists'; };
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
fn::pse_execution_v1::source_creator($pse_rpc_expires_at,$problem,'history:'+$operation);
CREATE type::record('canonical_roots', 'history:' + $operation) SET key = 'history:' + $operation, problem = $problem, revision = $operation, sequence = $sequence, owner_kind = 'history', owner = $operation;
fn::pse_project_v1::source_ack($pse_rpc_expires_at,$problem,$operation,$project_intent);
UPDATE type::record('canonical_stages', $operation) SET activated = true;
UPSERT $head_guard SET key = 'head:' + $problem, generation = (generation ?? 0dec) + 1dec;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
IF (SELECT * FROM ONLY $retention_guard)=NONE { CREATE ONLY $retention_guard SET key='retention:'+$problem,generation=0dec,incarnation=<string>rand::uuid::v4(); };
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $stage.expires_at <= time::micros() { THROW 'PSE_REFUSAL:activation_staging_lease_expired_before_commit'; };
IF $admission_owner!=NONE { fn::pse_execution_v1::lease($pse_rpc_expires_at,$admission_owner.expires_at); };
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
    fn private_document(path: &Path, value: &serde_json::Value) {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        file.write_all(&serde_json::to_vec(value).unwrap()).unwrap();
    }
    fn storage_scope() -> (Scope, serde_json::Value) {
        use std::os::unix::fs::PermissionsExt;
        let (scope, _) = scope();
        std::fs::set_permissions(&scope.0, std::fs::Permissions::from_mode(0o700)).unwrap();
        let credentials = scope.0.join("credentials.json");
        private_document(
            &credentials,
            &serde_json::json!({
                "username": "owner", "password": "secret",
                "selection_username": "viewer", "selection_password": "secret",
            }),
        );
        let config = serde_json::json!({
            "profile_version": 3, "endpoint": "grpc://127.0.0.1:8000",
            "namespace": "canonical", "database": "selected",
            "schema_interpretation": wire::INTERPRETATION,
            "max_message_bytes": MESSAGE_BYTES, "credentials_file": credentials,
            "accepting_writes": false,
            "study_module": serde_json::to_value(study_module::tests::receipt_in(&scope.0)).unwrap(),
        });
        (scope, config)
    }
    #[test]
    fn storage_projection_ignores_absent_malformed_and_invalid_native_policy() {
        let (scope, config) = storage_scope();
        std::fs::create_dir(scope.0.join(".contexts")).unwrap();
        std::fs::write(
            scope.0.join(".contexts/isolated.json"),
            b"invalid unused native context",
        )
        .unwrap();
        for native in [
            None,
            Some(serde_json::json!({"resources": "unused", "primary_receiver": 42})),
            Some(
                serde_json::json!({"resources": {"native_workers": 0}, "primary_receiver": {"worker_executable": "relative"}}),
            ),
        ] {
            let mut selected = config.clone();
            if let Some(native) = native {
                selected
                    .as_object_mut()
                    .unwrap()
                    .extend(native.as_object().unwrap().clone());
            }
            private_document(&scope.0.join("config.json"), &selected);
            let default = CanonicalOptions::from_state(&scope.0).unwrap();
            assert_eq!(default.database, "selected");
            let isolated =
                CanonicalOptions::from_state_for_database(&scope.0, Some("isolated")).unwrap();
            assert_eq!(isolated.database, "isolated");
        }
    }
    #[test]
    fn storage_projection_bounds_full_module_config_separately_from_credentials() {
        let (scope, mut config) = storage_scope();
        // Real source receipts can exceed the old 64 KiB configuration bound.
        // Provenance stays opaque to storage projection but the complete module
        // artifact, SDK, ABI and declared limits still undergo normal admission.
        let sources=(0..768).map(|index|serde_json::json!({
            "path":format!("crates/pse-model/src/source-{index}.rs"),"sha256":"a".repeat(64),
        })).collect::<Vec<_>>();
        config["study_module"]["inputs"]["source_files"] = sources.into();
        let mut qualification = config["study_module"].clone();
        qualification["kind"] = "pse-study-module-qualification-v1".into();
        qualification["inputs"]["features"] = serde_json::json!(["module-qualification"]);
        config["study_module_qualification"] = qualification;
        let path = scope.0.join("config.json");
        private_document(&path, &config);
        let mut bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > 64 * 1024 && bytes.len() < 256 * 1024);
        CanonicalOptions::from_state(&scope.0).unwrap();
        bytes.resize(256 * 1024, b' ');
        std::fs::write(&path, &bytes).unwrap();
        CanonicalOptions::from_state(&scope.0).unwrap();
        bytes.push(b' ');
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            CanonicalOptions::from_state(&scope.0),
            Err(CanonicalError::Configuration(_))
        ));
        private_document(&path, &config);
        let credentials = scope.0.join("credentials.json");
        let mut bytes = std::fs::read(&credentials).unwrap();
        bytes.resize(64 * 1024, b' ');
        std::fs::write(&credentials, &bytes).unwrap();
        CanonicalOptions::from_state(&scope.0).unwrap();
        bytes.push(b' ');
        std::fs::write(&credentials, &bytes).unwrap();
        assert!(matches!(
            CanonicalOptions::from_state(&scope.0),
            Err(CanonicalError::Configuration(_))
        ));
    }
    #[test]
    fn storage_projection_retains_interpretation_protocol_credentials_and_identity_validation() {
        let (scope, config) = storage_scope();
        let mut mismatched = config.clone();
        mismatched["schema_interpretation"] = "unsupported".into();
        private_document(&scope.0.join("config.json"), &mismatched);
        assert!(matches!(
            CanonicalOptions::from_state(&scope.0),
            Err(CanonicalError::Interpretation { .. })
        ));
        let mut invalid = config.clone();
        invalid["max_message_bytes"] = 0.into();
        private_document(&scope.0.join("config.json"), &invalid);
        assert!(matches!(
            CanonicalOptions::from_state(&scope.0),
            Err(CanonicalError::Configuration(_))
        ));
        private_document(&scope.0.join("config.json"), &config);
        assert!(CanonicalOptions::from_state_for_database(&scope.0, Some("../invalid")).is_err());
        private_document(
            &scope.0.join("credentials.json"),
            &serde_json::json!({"username": "owner"}),
        );
        assert!(matches!(
            CanonicalOptions::from_state(&scope.0),
            Err(CanonicalError::Configuration(_))
        ));
    }

    fn admission(owner: serde_json::Value, accepting_writes: bool) -> WriteAdmission {
        serde_json::from_value(serde_json::json!({
            "namespace": "fresh", "database": "current", "accepting_writes": accepting_writes,
            "derived_rebuild_pending": {"initializer": owner},
        }))
        .unwrap()
    }

    #[test]
    fn write_admission_refuses_changed_or_missing_opened_module() {
        let hash = "a".repeat(64);
        let selected:WriteAdmission=serde_json::from_value(serde_json::json!({"namespace":"fresh","database":"current","accepting_writes":true,"study_module":{"sha256":hash}})).unwrap();
        assert!(selected.require_module(Some(&hash)).is_ok());
        assert!(matches!(
            selected.require_module(Some(&"b".repeat(64))),
            Err(CanonicalError::Quiesced)
        ));
        let missing: WriteAdmission = serde_json::from_value(
            serde_json::json!({"namespace":"fresh","database":"current","accepting_writes":true}),
        )
        .unwrap();
        assert!(matches!(
            missing.require_module(Some(&hash)),
            Err(CanonicalError::Quiesced)
        ));
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
    fn rollout_initializer_requires_exact_live_owner_and_rejects_competing_pending_work() {
        let (scope, owner) = scope();
        let nonce = owner["nonce"].as_str().unwrap();
        let mut selected = admission(owner.clone(), false);
        selected.surrealkit_rollout_pending = selected.derived_rebuild_pending.take();
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "current", Some(nonce))
                .is_ok()
        );
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "other", Some(nonce))
                .is_err()
        );
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "current", None)
                .is_err()
        );
        selected.derived_rebuild_pending = admission(owner.clone(), false).derived_rebuild_pending;
        assert!(
            selected
                .require_writes(&scope.0, "fresh", "current", Some(nonce))
                .is_err()
        );
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
    async fn problem_turns_share_one_problem_and_release_cancelled_waiters() {
        let turns = ProblemTurns::default();
        let gate = turns.gate("problem").unwrap();
        assert!(Arc::ptr_eq(&gate, &turns.gate("problem").unwrap()));
        let held = gate.clone().write_owned().await;
        let mut cancelled = Box::pin(gate.clone().write_owned());
        std::future::poll_fn(|context| {
            assert!(cancelled.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        let mut following = Box::pin(gate.clone().write_owned());
        std::future::poll_fn(|context| {
            assert!(following.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        let independent = turns.gate("independent").unwrap();
        let independent_held = independent.clone().try_write_owned().unwrap();
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
        let turns = ProblemTurns::default();
        let gate = turns.gate("problem").unwrap();
        let held = gate.clone().write_owned().await;
        let started = std::sync::atomic::AtomicBool::new(false);
        let result = turns
            .run(
                "problem",
                QueryEffect::Mutation,
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
            .run(
                "problem",
                QueryEffect::Mutation,
                tokio::time::Instant::now(),
                async {
                    started.store(true, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                },
            )
            .await;
        assert!(matches!(result, Err(CanonicalError::Timeout)));
        assert!(!started.load(std::sync::atomic::Ordering::Relaxed));
        turns
            .run(
                "problem",
                QueryEffect::Mutation,
                tokio::time::Instant::now() + std::time::Duration::from_secs(1),
                async { Ok(()) },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn protected_turn_cancelled_waiter_releases_queue_without_blocking_other_problems() {
        let turns = ProblemTurns::default();
        let gate = turns.gate("problem").unwrap();
        let held = gate.clone().write_owned().await;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let cancelled_started = std::sync::atomic::AtomicBool::new(false);
        let mut cancelled =
            Box::pin(
                turns.run("problem", QueryEffect::Mutation, deadline, async {
                    cancelled_started.store(true, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }),
            );
        std::future::poll_fn(|context| {
            assert!(cancelled.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        let mut following =
            Box::pin(turns.run("problem", QueryEffect::Mutation, deadline, async { Ok(()) }));
        std::future::poll_fn(|context| {
            assert!(following.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        turns
            .run("independent", QueryEffect::Mutation, deadline, async {
                Ok(())
            })
            .await
            .unwrap();
        drop(cancelled);
        drop(held);
        following.await.unwrap();
        assert!(!cancelled_started.load(std::sync::atomic::Ordering::Relaxed));
        assert!(gate.try_write_owned().is_ok());
    }

    #[tokio::test]
    async fn overlapping_problem_reads_retain_queued_writer_preference() {
        use std::{future::Future, task::Poll};
        let turns = ProblemTurns::default();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        let (entered, mut observed) = tokio::sync::mpsc::unbounded_channel();
        let (first_release, first_held) = tokio::sync::oneshot::channel();
        let (second_release, second_held) = tokio::sync::oneshot::channel();
        let (writer_release, writer_held) = tokio::sync::oneshot::channel();
        let mut first = Box::pin(turns.run("problem", QueryEffect::Read, deadline, async {
            entered.send("first_read").unwrap();
            first_held.await.unwrap();
            Ok(())
        }));
        let mut second = Box::pin(turns.run("problem", QueryEffect::Read, deadline, async {
            entered.send("second_read").unwrap();
            second_held.await.unwrap();
            Ok(())
        }));
        std::future::poll_fn(|cx| {
            assert!(first.as_mut().poll(cx).is_pending());
            assert!(second.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert_eq!(observed.try_recv().unwrap(), "first_read");
        assert_eq!(observed.try_recv().unwrap(), "second_read");
        let mut writer = Box::pin(
            turns.run("problem", QueryEffect::Mutation, deadline, async {
                entered.send("writer").unwrap();
                writer_held.await.unwrap();
                Ok(())
            }),
        );
        let mut late = Box::pin(turns.run("problem", QueryEffect::Read, deadline, async {
            entered.send("late_read").unwrap();
            Ok(())
        }));
        std::future::poll_fn(|cx| {
            // Poll the writer first to establish its queue position exactly.
            assert!(writer.as_mut().poll(cx).is_pending());
            assert!(late.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert!(observed.try_recv().is_err());
        first_release.send(()).unwrap();
        first.await.unwrap();
        std::future::poll_fn(|cx| {
            assert!(writer.as_mut().poll(cx).is_pending());
            assert!(late.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert!(
            observed.try_recv().is_err(),
            "one remaining read still excludes the writer"
        );
        second_release.send(()).unwrap();
        second.await.unwrap();
        std::future::poll_fn(|cx| {
            // Even polling the late reader first cannot bypass the queued writer.
            assert!(late.as_mut().poll(cx).is_pending());
            assert!(writer.as_mut().poll(cx).is_pending());
            assert!(late.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert_eq!(observed.try_recv().unwrap(), "writer");
        assert!(observed.try_recv().is_err());
        writer_release.send(()).unwrap();
        writer.await.unwrap();
        late.await.unwrap();
        assert_eq!(observed.try_recv().unwrap(), "late_read");
    }

    #[tokio::test]
    async fn queued_problem_read_keeps_enclosing_original_clock_and_never_starts() {
        use std::{
            future::Future,
            sync::atomic::{AtomicBool, Ordering},
            task::Poll,
        };
        let turns = ProblemTurns::default();
        let held = turns.gate("problem").unwrap().write_owned().await;
        let original = tokio::time::Instant::now() + std::time::Duration::from_millis(20);
        let offered = original + std::time::Duration::from_secs(30);
        let started = AtomicBool::new(false);
        let mut queued = Box::pin(within_clock(
            original,
            turns.run("problem", QueryEffect::Read, offered, async {
                started.store(true, Ordering::SeqCst);
                Ok(())
            }),
        ));
        std::future::poll_fn(|cx| {
            assert!(queued.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        assert!(!started.load(Ordering::SeqCst));
        assert!(matches!(queued.await, Err(CanonicalError::Timeout)));
        assert!(!started.load(Ordering::SeqCst));
        drop(held);
        turns
            .run(
                "problem",
                QueryEffect::Read,
                tokio::time::Instant::now() + std::time::Duration::from_secs(1),
                async { Ok(()) },
            )
            .await
            .unwrap();
        assert!(
            !started.load(Ordering::SeqCst),
            "expired work never resumes after ownership is released"
        );
    }
}

#[cfg(test)]
mod description_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "pure description identity assertions fail on unexpected encoding errors"
    )]
    use super::*;
    use crate::canonical_selection::{ProductDescription, SelectedRead, SelectionMaterial};
    use pse_model::generated::runtime::canonical_products::Row as Product;

    fn read(revision: &str, protection: &str) -> SelectedRead {
        let mut read = SelectedRead::new(ProtectedSelection {
            owner: Arc::new(ProtectionOwner {
                store: None,
                activity: std::sync::Mutex::new(None),
                executor: None,
                released: std::sync::atomic::AtomicBool::new(false),
                metadata_owner: std::sync::OnceLock::new(),
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
            producer_class: pse_model::generated::enums::CanonicalProducerClass::General,
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
    #[test]
    fn split_descriptions_share_receipts_and_preserve_body_identity_when_premises_change() {
        let first = read("revision-one", "pin-one");
        let selected = SelectionMaterial::capture_unowned(&first).unwrap();
        let one = ProductDescription::prepare_with_selection(
            &first,
            product(&first, "one", b"body-one"),
            selected.clone(),
        )
        .unwrap();
        let two = ProductDescription::prepare_with_selection(
            &first,
            product(&first, "two", b"body-two"),
            selected.clone(),
        )
        .unwrap();
        let a = crate::canonical_staging::ProductBlob::decode(one.descriptor()).unwrap();
        let b = crate::canonical_staging::ProductBlob::decode(two.descriptor()).unwrap();
        assert_ne!(a.version, b.version);
        assert_eq!(a.receipt, b.receipt);
        assert!(Arc::ptr_eq(
            one.selection_material(),
            two.selection_material()
        ));
        let mut changed = read("revision-two", "pin-two");
        changed
            .interpretation("acquired".into(), "different".into())
            .unwrap();
        let other =
            ProductDescription::prepare(&changed, product(&changed, "one", b"body-one")).unwrap();
        let c = crate::canonical_staging::ProductBlob::decode(other.descriptor()).unwrap();
        assert_eq!(
            a.version, c.version,
            "receipt changes preserve exact body-content identity"
        );
        assert_ne!(a.receipt, c.receipt);
        let p = product(&first, "same", b"body-one");
        let raw = ProductDescription::prepare(&first, p).unwrap();
        let header = pse_model::generated::runtime::canonical_products::Row {
            payload: raw.descriptor().to_vec().into(),
            dependencies: Vec::new().into(),
            ..product(&first, "ignored", b"")
        };
        let store_one =
            descriptor_for_product(&header, "00000000-0000-4000-8000-000000000001").unwrap();
        let store_two =
            descriptor_for_product(&header, "00000000-0000-4000-8000-000000000002").unwrap();
        assert_ne!(
            store_one.key, store_two.key,
            "issuing stores cannot confer each other's ACKs"
        );
        let mut qualified = header.clone();
        qualified.producer_class =
            pse_model::generated::enums::CanonicalProducerClass::LibraryMathematics;
        let qualified_descriptor =
            descriptor_for_product(&qualified, "00000000-0000-4000-8000-000000000001").unwrap();
        assert_eq!(
            qualified.payload, header.payload,
            "authority does not rewrite scientific body bytes"
        );
        assert_ne!(
            store_one.key, qualified_descriptor.key,
            "a generic publication cannot borrow a qualified producer's acknowledgement"
        );
    }
    #[test]
    fn shared_body_description_retains_exact_payload_allocation_without_copy() {
        let selected = read("revision", "pin");
        let selection = SelectionMaterial::capture_unowned(&selected).unwrap();
        let payload = Arc::new(b"body".to_vec());
        let pointer = payload.as_ptr();
        use pse_columnar::{AllocationLease, GreedyMemoryPool, MemoryConsumer, MemoryPool};
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let reservation = MemoryConsumer::new("test:shared-payload").register(&pool);
        reservation.try_grow(4096).unwrap();
        let owner = AllocationLease::new(reservation);
        let lifetime = Arc::downgrade(&owner);
        let header = product(&selected, "offer", b"");
        let description = ProductDescription::prepare_with_shared_payload(
            &selected,
            header,
            selection,
            payload.clone(),
            owner.clone(),
        )
        .unwrap();
        // Arc counts distinguish sharing from an equally valued hidden copy.
        assert_eq!(Arc::strong_count(&payload), 2);
        assert_eq!(payload.as_ptr(), pointer);
        drop(owner);
        drop(payload);
        assert!(lifetime.upgrade().is_some());
        assert!(description.descriptor_allocation_bytes() < description.exclusive_retained_bytes());
        drop(description);
        assert!(lifetime.upgrade().is_none());
    }
    #[test]
    fn shared_selection_material_preserves_wire_bytes_and_allocation_owner() {
        let selected = read("revision", "pin");
        use pse_columnar::{AllocationLease, GreedyMemoryPool, MemoryConsumer, MemoryPool};
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let reservation = MemoryConsumer::new("test:selection-material").register(&pool);
        reservation.try_grow(4096).unwrap();
        let owner = AllocationLease::new(reservation);
        let lifetime = Arc::downgrade(&owner);
        let selection = SelectionMaterial::capture(&selected, owner.clone()).unwrap();
        drop(owner);
        let descriptions = [b"body-one".as_slice(), b"body-two".as_slice()].map(|payload| {
            let offered = product(&selected, "offer", payload);
            let shared = ProductDescription::prepare_with_selection(
                &selected,
                offered.clone(),
                selection.clone(),
            )
            .unwrap();
            let legacy = ProductDescription::prepare(&selected, offered).unwrap();
            assert_eq!(shared.descriptor(), legacy.descriptor());
            assert_eq!(
                shared.selected_dependencies(),
                legacy.selected_dependencies()
            );
            assert_eq!(
                shared.retained_bytes(),
                shared.exclusive_retained_bytes() + selection.retained_bytes()
            );
            assert!(Arc::ptr_eq(shared.selection_material(), &selection));
            shared
        });
        let uncertain_settlement = descriptions[1].clone();
        drop(descriptions);
        drop(selection);
        drop(selected);
        assert!(
            lifetime.upgrade().is_some(),
            "the escaped settlement description retains the charged owner"
        );
        assert_eq!(pool.reserved(), 4096);
        drop(uncertain_settlement);
        assert!(lifetime.upgrade().is_none());
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn shared_selection_material_refuses_acquired_changes_and_diverged_read_clones() {
        let selected = read("revision", "pin");
        let original = SelectionMaterial::capture_unowned(&selected).unwrap();
        let mut left = selected.clone();
        let mut right = selected.clone();
        left.interpretation("acquired".into(), "left".into())
            .unwrap();
        right
            .interpretation("acquired".into(), "right".into())
            .unwrap();
        let left_material = SelectionMaterial::capture_unowned(&left).unwrap();
        assert!(
            ProductDescription::prepare_with_selection(
                &left,
                product(&left, "offer", b"body"),
                original.clone()
            )
            .is_err()
        );
        assert!(
            ProductDescription::prepare_with_selection(
                &right,
                product(&right, "offer", b"body"),
                left_material.clone()
            )
            .is_err()
        );
        let another_pin = read("revision", "another-pin");
        assert!(
            ProductDescription::prepare_with_selection(
                &another_pin,
                product(&another_pin, "offer", b"body"),
                original
            )
            .is_err()
        );
        assert!(
            left_material
                .rebinding_from(
                    &SelectionMaterial::capture_unowned(&selected).unwrap(),
                    &left
                )
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn selection_rebinding_preserves_original_blob_and_requires_exact_receipt_origin() {
        let first = read("revision-one", "pin-one");
        let original =
            ProductDescription::prepare(&first, product(&first, "offer", b"body")).unwrap();
        let mut second = read("revision-two", "pin-two");
        let selection = SelectionMaterial::capture_unowned(&second).unwrap();
        let binding = selection
            .rebinding_from(original.selection_material(), &second)
            .unwrap()
            .unwrap();
        let rebound = original.rebind_selection(&second, &binding).unwrap();
        assert_eq!(
            original.descriptor().as_ptr(),
            rebound.descriptor().as_ptr()
        );
        assert!(Arc::ptr_eq(rebound.selection_material(), &selection));
        let separately_captured =
            ProductDescription::prepare(&first, product(&first, "offer", b"body")).unwrap();
        assert!(
            separately_captured
                .rebind_selection(&second, &binding)
                .is_err()
        );
        second
            .interpretation("acquired".into(), "changed".into())
            .unwrap();
        assert!(original.rebind_selection(&second, &binding).is_err());
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
    use surrealdb_types::Number;
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
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn canonical_sixteen_lane_pacing_attributes_actual_work_and_preserves_control_progress() {
        use crate::canonical_execution::{ResultPayload, RunRequest};
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::time::Duration;
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit("problem", None, "pacing-initial", &[])
            .await
            .unwrap();
        let request = |key: &str| RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        store.begin_run(&request("pacing-cancel")).await.unwrap();
        let fence = store
            .claim_run(
                "pacing-cancel",
                "pacing-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let probe = pacing_observation::Probe::default();
        let gate = store.problem_turns.gate("problem").unwrap();
        let held = gate.write_owned().await;
        let barrier = Arc::new(tokio::sync::Barrier::new(17));
        let independent = Arc::new(AtomicUsize::new(0));
        let cancelled = Arc::new(AtomicBool::new(false));
        let mut jobs = tokio::task::JoinSet::new();
        for lane in 0..16 {
            let store = store.clone();
            let revision = revision.clone();
            let barrier = barrier.clone();
            let probe = probe.clone();
            let independent = independent.clone();
            let cancelled = cancelled.clone();
            jobs.spawn(async move {
                probe
                    .scope(async {
                        barrier.wait().await;
                        match lane {
                            0..=3 => {
                                // Four exact callers publish one source receipt, without
                                // holding the problem turn across their whole operation.
                                store
                                    .edit(
                                        "problem",
                                        Some("pacing-initial"),
                                        "pacing-shared-source",
                                        &[ObjectEdit {
                                            logical: "paced-object".into(),
                                            scope: "root".into(),
                                            name: "Paced".into(),
                                            references: vec![],
                                            version: Some(version(
                                                "paced-version",
                                                "paced-object",
                                                b"retained",
                                            )),
                                        }],
                                    )
                                    .await
                                    .unwrap();
                            }
                            4..=6 => {
                                let key = format!("pacing-start-{lane}");
                                let request = RunRequest {
                                    purpose:
                                        pse_model::generated::enums::CanonicalRunPurpose::Execution,
                                    admission_study: None,
                                    admission_point: None,
                                    key: key.clone().into(),
                                    revision: revision.clone(),
                                    sources: vec![],
                                    request: vec![1],
                                    source_selection: vec![2],
                                    attestation: vec![3],
                                };
                                let run = store.begin_run(&request).await.unwrap();
                                assert_eq!(store.begin_run(&request).await.unwrap(), run);
                                let attempt = store
                                    .claim_run(
                                        &key,
                                        &format!("pacing-claim-{lane}"),
                                        "worker",
                                        Duration::from_secs(60),
                                    )
                                    .await
                                    .unwrap();
                                store
                                    .renew_attempt(&attempt, Duration::from_secs(60))
                                    .await
                                    .unwrap();
                                store
                                    .cancel_run(&key, &format!("pacing-cancel-{lane}"))
                                    .await
                                    .unwrap();
                            }
                            7..=10 => {
                                let protection = store
                                    .protect(revision.clone(), Duration::from_secs(60))
                                    .await
                                    .unwrap();
                                let mut selected =
                                    crate::canonical_selection::SelectedRead::new(protection);
                                assert!(
                                    store
                                        .resolve_logicals(&mut selected, &["absent".into()])
                                        .await
                                        .unwrap()
                                        .is_empty()
                                );
                                store.release(selected.selection()).await.unwrap();
                            }
                            11..=14 => {
                                store
                                    .edit(
                                        &format!("independent-{lane}"),
                                        None,
                                        &format!("independent-source-{lane}"),
                                        &[],
                                    )
                                    .await
                                    .unwrap();
                                independent.fetch_add(1, Ordering::SeqCst);
                            }
                            _ => {
                                let run = store
                                    .cancel_run("pacing-cancel", "pacing-control-cancel")
                                    .await
                                    .unwrap();
                                assert!(run.cancelled);
                                cancelled.store(true, Ordering::SeqCst);
                            }
                        }
                    })
                    .await
            });
        }
        barrier.wait().await;
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let queued = probe
                    .snapshot()
                    .get("problem")
                    .map_or(0, |observation| observation.submitted);
                if queued >= 4
                    && independent.load(Ordering::SeqCst) == 4
                    && cancelled.load(Ordering::SeqCst)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        // An expired original clock leaves the actual native mutation unissued.
        let expired=operation_scope(std::time::Instant::now()+Duration::from_millis(20),store.protected_query("problem",QueryEffect::Mutation,"qualification::expired_pacing",||Ok(store.db.query("CREATE canonical_guards:expired_pacing SET key='expired_pacing',generation=0dec;")))).await;
        assert!(matches!(expired, Err(CanonicalError::Timeout)));
        drop(held);
        tokio::time::timeout(Duration::from_secs(45), async {
            while let Some(result) = jobs.join_next().await {
                result.unwrap();
            }
        })
        .await
        .unwrap();
        let mut missing = bounded_query(
            store
                .db
                .query("SELECT * FROM ONLY canonical_guards:expired_pacing;"),
        )
        .await
        .unwrap();
        assert!(missing.take::<Option<Object>>(0).unwrap().is_none());
        assert!(
            store
                .append_result_batch(
                    &fence,
                    "pacing-revoked-write",
                    "values",
                    0,
                    ResultPayload::new(vec![1]).unwrap(),
                    1
                )
                .await
                .is_err()
        );
        assert_eq!(
            store
                .execution_run_page("problem", None, 64)
                .await
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            store
                .revision("pacing-shared-source")
                .await
                .unwrap()
                .unwrap()
                .sequence,
            revision.sequence + 1
        );
        let observations = probe.snapshot();
        assert!(observations["problem"].mutations >= 4);
        assert!(
            observations["problem"].reads >= 4,
            "actual protected selection reads must be observed"
        );
        assert!(observations["problem"].longest_queue > Duration::ZERO);
        assert!(
            observations
                .keys()
                .any(|key| key.starts_with("execution-run:pacing"))
        );
        assert!(
            observations
                .keys()
                .any(|key| key.starts_with("independent-"))
        );
        eprintln!(
            "actual sixteen-lane pacing (queue and hold exclude scientific work): {observations:#?}"
        );
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn operation_owned_initialization_repeats_and_preserves_readback_failure() {
        let (store, options) = initialization_fixture().await;
        CanonicalStore::initialize_database(&options).await.unwrap();
        store.open().await.unwrap();
        let identity = store.issuing_store().unwrap().to_owned();
        for _ in 0..4 {
            CanonicalStore::initialize_database(&options).await.unwrap();
            store.open().await.unwrap();
            assert_eq!(store.issuing_store().unwrap(), identity);
        }
        bounded_query(
            store.db.query(
                "UPDATE canonical_interpretations:current SET schema_digest='unknown-reader';",
            ),
        )
        .await
        .unwrap();
        assert!(matches!(
            CanonicalStore::initialize_database(&options).await,
            Err(CanonicalError::Interpretation { observed, .. }) if observed == "unknown-reader"
        ));
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn source_protection_last_fixture_drop_joins_automatic_release() {
        let (store, options) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit("last-fixture-release", None, "source", &[])
            .await
            .unwrap();
        let selection = store
            .protect(revision, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let key = selection.key().to_owned();
        let resource = store.fixture_lifetime.as_ref().unwrap().resource.clone();
        let drain = store.result_read_drain.clone();
        let held = store
            .problem_turns
            .gate(&format!("pin:{key}"))
            .unwrap()
            .write_owned()
            .await;
        let observer = CanonicalStore::connect(&options).await.unwrap();
        drop(store);
        // Only the final protected reader owns the fixture now. Its cleanup RPC
        // is deliberately paced before submission, on the real release path.
        let (done, completed) = std::sync::mpsc::channel();
        let dropping = std::thread::spawn(move || {
            drop(selection);
            done.send(()).unwrap();
        });
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while drain.live() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            matches!(
                completed.recv_timeout(std::time::Duration::from_millis(100)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            ),
            "last fixture Drop must join the outstanding automatic release"
        );
        let status =
            crate::testing::resource_bridge("status", &serde_json::json!({"resource": resource}))
                .unwrap();
        assert!(status.get("creation_borrower").is_some());
        assert_ne!(status["drained"], serde_json::json!(true));
        let mut response = bounded_query(
            observer
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
        completed
            .recv_timeout(REQUEST_TIMEOUT + std::time::Duration::from_secs(60))
            .unwrap();
        dropping.join().unwrap();
        let status =
            crate::testing::resource_bridge("status", &serde_json::json!({"resource": resource}))
                .unwrap();
        assert_eq!(status["drained"], serde_json::json!(true));
        assert!(status.get("creation_borrower").is_none());
        let mut response = bounded_query(
            observer
                .db
                .query(
                    "SELECT VALUE released FROM ONLY type::record('canonical_protections', $key);",
                )
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
        observer.disconnect().await.unwrap();
    }
    #[tokio::test]
    async fn source_protection_final_finish_drains_before_last_fixture_teardown() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit("last-finish", None, "source", &[])
            .await
            .unwrap();
        let selection = store
            .protect(revision, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let resource = store.fixture_lifetime.as_ref().unwrap().resource.clone();
        store.close();
        drop(store);
        selection.finish().await.unwrap();
        let status =
            crate::testing::resource_bridge("status", &serde_json::json!({"resource": resource}))
                .unwrap();
        assert_eq!(status["drained"], serde_json::json!(true));
        assert!(status.get("creation_borrower").is_none());
    }
    #[test]
    fn source_protection_cleanup_outlives_its_callers_executor() {
        let store = crate::testing::canonical_fixture_store().unwrap();
        let caller = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let protection = caller.block_on(async {
            let revision = store
                .edit("short-lived-executor", None, "source", &[])
                .await
                .unwrap();
            store
                .protect(revision, std::time::Duration::from_secs(30))
                .await
                .unwrap()
        });
        drop(caller);
        // Automatic cleanup must run on the still-live physical fixture owner,
        // even when no runtime is entered at the point the final reader drops.
        drop(protection);
        let observer = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        observer.block_on(store.result_read_drain.drain()).unwrap();
        observer.block_on(async {
            tokio::time::timeout(REQUEST_TIMEOUT, async {
                while store
                    .fixture_lifetime
                    .as_ref()
                    .is_some_and(|owner| Arc::strong_count(owner) != 1)
                {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
        });
        observer.block_on(store.remove_isolated_fixture()).unwrap();
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
        let gate = store.problem_turns.gate(&revision.problem).unwrap();
        let held = gate.write_owned().await;
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
            .problem_turns
            .gate(&format!("pin:{key}"))
            .unwrap()
            .write_owned()
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
    async fn source_protection_acquisition_and_renewal_continue_after_caller_close() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit("close-during-protection", None, "initial", &[])
            .await
            .unwrap();
        let held = store
            .problem_turns
            .gate(&revision.problem)
            .unwrap()
            .write_owned()
            .await;
        let acquiring = store.clone();
        let pending = tokio::spawn(async move {
            acquiring
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
        store.close();
        assert!(matches!(
            store.disconnect().await,
            Err(CanonicalError::ConnectionBusy { .. })
        ));
        drop(held);
        let selection = pending.await.unwrap().unwrap();
        selection
            .renew(std::time::Duration::from_secs(60))
            .await
            .unwrap();
        store.release(&selection).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn source_protection_renewal_preserves_original_grant_and_never_revives() {
        let (store, _) = initialization_fixture().await;
        store.create().await.unwrap();
        let revision = store
            .edit("renew-source", None, "initial", &[])
            .await
            .unwrap();
        let pin = store
            .protect(revision.clone(), std::time::Duration::from_secs(30))
            .await
            .unwrap();
        let key = pin.grant_key().to_owned();
        let before: Option<Object> =
            request(store.db.select(("canonical_protections", key.as_str())))
                .await
                .unwrap();
        let mut before = wire::decode_canonical_protections(before.unwrap()).unwrap();
        pin.renew(std::time::Duration::from_secs(60)).await.unwrap();
        assert_eq!(pin.grant_key(), key);
        let after: Option<Object> =
            request(store.db.select(("canonical_protections", key.as_str())))
                .await
                .unwrap();
        let mut after = wire::decode_canonical_protections(after.unwrap()).unwrap();
        assert!(after.expires_at > before.expires_at);
        before.expires_at = 0;
        after.expires_at = 0;
        assert_eq!(
            before, after,
            "renewal changes only expiry, never original grant/provenance"
        );
        bounded_query(
            store
                .db
                .query("UPDATE type::record('canonical_protections',$key) SET expires_at=0;")
                .bind(("key", key.clone())),
        )
        .await
        .unwrap();
        assert!(pin.renew(std::time::Duration::from_secs(60)).await.is_err());
        let expired: Option<Object> =
            request(store.db.select(("canonical_protections", key.as_str())))
                .await
                .unwrap();
        assert_eq!(
            wire::decode_canonical_protections(expired.unwrap())
                .unwrap()
                .expires_at,
            0
        );
        store.release(&pin).await.unwrap();
        assert!(pin.renew(std::time::Duration::from_secs(60)).await.is_err());
        let released = store
            .protect(revision, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        store.release(&released).await.unwrap();
        assert!(
            released
                .renew(std::time::Duration::from_secs(60))
                .await
                .is_err()
        );
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
        store.result_read_drain.retain().unwrap();
        let key = store.protection_grant_key(&revision.problem).await.unwrap();
        let selection = ProtectedSelection {
            owner: Arc::new(ProtectionOwner {
                key: key.clone(),
                revision: revision.clone(),
                store: Some(store.clone()),
                activity: std::sync::Mutex::new(Some(store.admit_activity().unwrap())),
                executor: Some(tokio::runtime::Handle::current()),
                released: std::sync::atomic::AtomicBool::new(false),
                metadata_owner: std::sync::OnceLock::new(),
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
            .problem_turns
            .gate(&format!("pin:{}", selection.key()))
            .unwrap()
            .write_owned()
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
        use surrealdb_types::Bytes;
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
            if endpoint != "grpc://127.0.0.1:18089" {
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
        surrealdb_types::Error::query(
            "definitely rejected installation".into(),
            surrealdb_types::QueryError::TransactionConflict,
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
                    Err(surrealdb_types::Error::query(
                        "definite protection rejection".into(),
                        surrealdb_types::QueryError::TransactionConflict,
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
                Err(surrealdb_types::Error::query(
                    "final definite rejection".into(),
                    surrealdb_types::QueryError::TransactionConflict,
                ))
            })
        })
        .await;
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            GUARDED_ATTEMPTS as usize
        );
        assert!(matches!(
            result,
            Err(CanonicalError::ConflictExhausted { .. })
        ));
        for kind in [
            surrealdb_types::QueryError::Cancelled,
            surrealdb_types::QueryError::NotExecuted,
        ] {
            attempts.store(0, std::sync::atomic::Ordering::SeqCst);
            let result = protected_query("canonical::protected_decision_retries_only_definite_conflicts_and_preserves_final_error", || {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let kind = kind.clone();
                Ok(async move { Err(surrealdb_types::Error::query("uncertain protection".into(), kind)) })
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
        assert!(matches!(
            result,
            Err(CanonicalError::ConflictExhausted { .. })
        ));
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
        let module = study_module::StudyModuleReceipt::read(&store.state_path).unwrap();
        let delayed = initialization_statement(&module)
            .unwrap()
            .into_parts()
            .0
            .replacen("BEGIN;", "BEGIN; SLEEP 1s;", 1);
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
                                .bind(("study_module_backend", module.backend().unwrap()))
                                .bind(("study_module_sha256", module.sha256.clone()))
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
            .with_canonical(pse_schema::model::CanonicalRelation::DURABLE)
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
        assert!(fixture.contains("DEFINE TABLE canonical_wire_fixture SCHEMAFULL;"));
        assert!(fixture.contains("DEFINE FIELD finite.bits ON canonical_wire_fixture"));
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
            object.insert("bits", surrealdb_types::Bytes::from(vec![0u8; 7]));
        }
        let malformed = store
            .db
            .query("CREATE canonical_wire_fixture:bad SET key = 'bad', finite = $cell, raw = $raw;")
            .bind(("cell", short))
            .bind(("raw", codec::encode_diagnostic_bits(0)))
            .await
            .unwrap()
            .check();
        assert!(
            malformed.is_err(),
            "seven-byte finite cell was not refused: {malformed:?}"
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
    async fn grpc_exact_cells_and_guarded_revisions() {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        store.open().await.unwrap();
        let result: Result<(), CanonicalError> = async {
            for (index, value) in [0, (1u64<<63)-1, 1u64<<63, (1u64<<63)+1, u64::MAX-1, u64::MAX].into_iter().enumerate() {
                let row = pse_model::generated::runtime::canonical_guards::Row { key: format!("boundary-{index}"), generation: value, incarnation: None, protection_closed_through: None, analysis_creation_closed_through: None, artifact_creation_closed_through: None, source_reclaim_after: None };
                let mut response = store.db.query("CREATE type::record('canonical_guards', $key) CONTENT $row;").bind(("key", row.key.clone())).bind(("row", wire::encode_canonical_guards(&row)?)).await.map_err(CanonicalError::from).and_then(checked)?;
                let saved: Option<Object> = response.take(0)?;
                assert_eq!(wire::decode_canonical_guards(saved.unwrap())?.generation, value);
            }
            let mut ordered = store.db.query("SELECT generation FROM canonical_guards WHERE generation >= $lower ORDER BY generation;").bind(("lower", crate::canonical_codec::encode_uint(1u64<<63)?)).await.map_err(CanonicalError::from).and_then(checked)?;
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

// The same transaction text is used by held-commit protocol controls.
pub(crate) const ACQUIRE_PROTECTION: &str = r#"
            SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            LET $grant=fn::pse_execution_v1::protection_grant($pse_rpc_expires_at,$problem,$key,true);
            LET $revision = SELECT * FROM ONLY type::record('canonical_revisions', $revision);
            IF $revision = NONE OR $revision.problem != $problem { THROW 'PSE_REFUSAL:revision_unavailable'; };
            LET $reclaimed = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $problem AND from_sequence <= $revision.sequence AND to_sequence > $revision.sequence LIMIT 1;
            IF array::len($reclaimed) != 0 { THROW 'PSE_REFUSAL:source_selection_reclaimed'; };
            IF (SELECT * FROM ONLY type::record('canonical_protections', $key) FOR UPDATE) = NONE { CREATE type::record('canonical_protections', $key) SET key = $key, problem = $problem, revision = $revision.key, sequence = $revision.sequence, expires_at = time::micros() + $lifetime, released = false, grant_deadline=$grant.grant_deadline, grant_nonce=$grant.grant_nonce, authority_incarnation=$grant.authority_incarnation; };
            fn::pse_execution_v1::touch($pse_rpc_expires_at,'source-creators:'+$problem);
            LET $rpc_result = SELECT * FROM ONLY type::record('canonical_revisions', $revision.key) TIMEOUT $pse_rpc_timeout;
            fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            LET $protection = SELECT * FROM ONLY type::record('canonical_protections', $key);
            IF $protection = NONE OR $protection.released OR $protection.expires_at <= time::micros() { THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; };
            IF $protection.problem!=$problem OR $protection.revision!=$revision.key OR $protection.sequence!=$revision.sequence OR $protection.grant_deadline!=$grant.grant_deadline OR $protection.grant_nonce!=$grant.grant_nonce OR $protection.authority_incarnation!=$grant.authority_incarnation { THROW 'PSE_REFUSAL:immutable_selection_grant_identity_reused'; };
"#;

// The same transaction text is used by held-commit protocol controls.
pub(crate) const RELEASE_PROTECTION: &str = r#"
            SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE;
            fn::pse_execution_v1::deadline($pse_rpc_expires_at);
            LET $grant=fn::pse_execution_v1::protection_grant($pse_rpc_expires_at,$problem,$protection,false);
            LET $pin = SELECT * FROM ONLY type::record('canonical_protections', $protection) FOR UPDATE;
            IF $pin != NONE AND ($pin.key != $protection OR $pin.problem != $problem OR $pin.revision != $revision OR $pin.sequence != $sequence) { THROW 'PSE_REFUSAL:immutable_selection_protection_identity_differs'; };
            IF !$grant.closed OR $pin!=NONE { UPSERT type::record('canonical_protections', $protection) SET key = $protection, problem = $problem, revision = $revision, sequence = $sequence, expires_at = 0, released = true, grant_deadline=$grant.grant_deadline, grant_nonce=$grant.grant_nonce, authority_incarnation=$grant.authority_incarnation; };
            fn::pse_files_v1::release_ticket($pse_rpc_expires_at,$protection);
            fn::pse_execution_v1::deadline($pse_rpc_expires_at);
"#;

#[cfg(test)]
mod source_refusal_tests {
    use super::*;
    #[test]
    fn publication_busy_retries_only_the_exact_native_rollback_tag() {
        let busy = CanonicalError::from(surrealdb_types::Error::thrown(
            "PSE_REFUSAL:study_capture_publication_busy".into(),
        ));
        assert!(busy.is_capture_busy());
        assert!(!busy.is_conflict());
        assert!(busy.source_transaction_refused());
        assert!(!busy.effect_is_unknown());
        for text in [
            "PSE_BUSY:other",
            "PSE_REFUSAL:study_capture_publication_busy changed",
            "PSE_REFUSAL:study_capture_publication_gate",
        ] {
            assert!(
                !CanonicalError::from(surrealdb_types::Error::thrown(text.into()))
                    .is_capture_busy()
            );
        }
        assert!(!CanonicalError::Timeout.is_capture_busy());
        assert!(
            !CanonicalError::from(surrealdb_types::Error::internal(
                RefusalCode::StudyCapturePublicationBusy.wire().into()
            ))
            .is_capture_busy()
        );
    }

    #[test]
    fn native_refusal_type_is_not_proof_without_the_source_effect_phase() {
        let raw = CanonicalError::Driver(surrealdb_types::Error::thrown(
            "explicit transaction refusal".into(),
        ));
        assert!(raw.source_transaction_refused());
        assert!(
            !raw.proves_source_refusal(),
            "an arbitrary receipt-read failure cannot prove source refusal"
        );
        let proved = CanonicalError::SourceEditRefused {
            problem: "original".into(),
            operation: "same-operation".into(),
            source: Box::new(raw),
        };
        assert!(proved.proves_source_refusal());
        for raw in [
            CanonicalError::Timeout,
            CanonicalError::IncompleteResponse,
            CanonicalError::Driver(surrealdb_types::Error::connection(
                "lost acknowledgement".into(),
                surrealdb_types::ConnectionError::ConnectionFailed,
            )),
        ] {
            assert!(!raw.source_transaction_refused());
            assert!(!raw.proves_source_refusal());
        }
    }
}

#[cfg(test)]
mod typed_refusal_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "exact native phase codec controls require valid generated wire values"
    )]
    use super::*;
    #[test]
    fn native_kind_authenticates_every_known_refusal_and_unknowns_fail_closed() {
        for &code in RefusalCode::ALL {
            let decoded = CanonicalError::from(surrealdb_types::Error::thrown(format!(
                "An error occurred: {}",
                code.wire()
            )));
            assert!(
                matches!(decoded, CanonicalError::Refused { code: actual, class, effect: FailureEffect::StatementRefused, .. } if actual == code && class == code.class())
            );
            assert!(matches!(
                CanonicalError::from(surrealdb_types::Error::internal(code.wire().into())),
                CanonicalError::Driver(_)
            ));
        }
        assert!(matches!(
            CanonicalError::from(surrealdb_types::Error::thrown(
                "An error occurred: PSE_REFUSAL:unknown".into()
            )),
            CanonicalError::Driver(_)
        ));
    }
    #[test]
    fn mx_capacity_and_poison_are_distinct_from_conflicts_and_keep_unknown_effect() {
        for (wire, expected) in [
            ("PSE_MX_MEMORY_LIMIT:", StoreFailure::MemoryLimit),
            ("PSE_MX_TX_TOO_OLD:", StoreFailure::TransactionTooOld),
            ("PSE_MX_DATABASE_POISONED:", StoreFailure::DatabasePoisoned),
            ("PSE_MX_ASYNC_QUEUE_FULL:", StoreFailure::AsyncQueueFull),
            ("PSE_MX_CORRUPTED:", StoreFailure::Corrupted),
            ("PSE_MX_COMMIT_UNKNOWN:", StoreFailure::CommitUnknown),
        ] {
            let decoded = CanonicalError::from(surrealdb_types::Error::internal(wire.into()));
            let original = std::error::Error::source(&decoded)
                .unwrap()
                .downcast_ref::<surrealdb_types::Error>()
                .unwrap();
            assert!(original.is_internal());
            assert_eq!(original.message(), wire);
            assert!(
                matches!(decoded, CanonicalError::StoreFailure { failure, effect: FailureEffect::Unknown, .. } if failure==expected)
            );
            assert!(matches!(
                CanonicalError::from(surrealdb_types::Error::thrown(wire.into())),
                CanonicalError::Driver(_)
            ));
        }
        let conflict = CanonicalError::from(surrealdb_types::Error::query(
            "conflict".into(),
            Some(surrealdb_types::QueryError::TransactionConflict),
        ));
        assert!(matches!(
            conflict.exhaust_conflict("test", 32),
            CanonicalError::ConflictExhausted { attempts: 32, .. }
        ));
    }
    #[test]
    fn every_physical_phase_roundtrips_without_unsigned_or_observational_coercion() {
        use crate::canonical_execution::{CanonicalAttempt, TerminalClass};
        use pse_model::generated::enums::CanonicalAttemptPhase as Phase;
        let mut row = CanonicalAttempt {
            key: "attempt".into(),
            run: "run".into(),
            generation: u64::MAX,
            claim_operation: "claim".into(),
            request: vec![1].into(),
            worker: "worker".into(),
            expires_at: 7,
            phase: Phase::Ingesting,
            close_generation: None,
            outcome: None,
            closed_manifest: None,
            completion: None,
            seed_manifest: None,
            finalization_study: None,
        };
        for (phase, ingesting, terminal, manifest) in [
            (Phase::Ingesting, true, false, false),
            (Phase::ClosedAwaitingManifest, false, false, false),
            (Phase::ClosedWithManifest, false, false, true),
            (Phase::Terminal, false, true, true),
            (Phase::TerminalCompacted, false, true, true),
        ] {
            row.phase = phase;
            row.close_generation = (!ingesting).then_some(u64::MAX);
            row.outcome = terminal.then_some(TerminalClass::Succeeded);
            row.closed_manifest = manifest.then(|| row.key.clone());
            row.completion = (phase == Phase::Terminal).then(|| vec![7].into());
            let decoded =
                wire::decode_canonical_attempts(wire::encode_canonical_attempts(&row).unwrap())
                    .unwrap();
            assert_eq!(decoded, row);
            assert_eq!(decoded.generation, u64::MAX);
            assert_eq!(decoded.ingestion_open(), ingesting);
            assert_eq!(decoded.closed(), !ingesting);
            assert_eq!(decoded.terminal(), terminal);
        }
    }
    #[test]
    fn physical_and_observed_lifecycle_projections_do_not_share_spellings() {
        use crate::canonical_execution::TerminalClass;
        use pse_model::generated::enums::{AttemptState, StudyPointState};
        assert_eq!(
            TerminalClass::Succeeded.attempt_state(),
            AttemptState::Completed
        );
        assert_eq!(
            TerminalClass::Partial.attempt_state(),
            AttemptState::Partial
        );
        assert_eq!(
            TerminalClass::Partial.study_point_state(true),
            StudyPointState::Completed
        );
        assert_eq!(
            TerminalClass::Partial.study_point_state(false),
            StudyPointState::Failed
        );
    }
}
