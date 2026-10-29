// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native execution selection at the runtime composition boundary.

use super::WorkflowError;
use pse_operations::canonical::{CanonicalError, CanonicalStore};
use std::path::{Path, PathBuf};

/// Materialized executable association for the managed study receiver.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
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
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct NativeAllocation {
    /// Finite service and execution envelope, separate from each process cap.
    pub total_memory_bytes: usize,
    /// Separately placed canonical storage process.
    pub server_memory_bytes: usize,
    /// Storage engine cache allowance materialized by the deployment owner.
    pub surrealmx_memory_limit_bytes: usize,
    /// Maximum transaction age selected by the deployment owner.
    pub surrealmx_max_transaction_age_seconds: u64,
    /// Storage snapshot interval selected by the deployment owner.
    pub surrealmx_snapshot_interval_seconds: u64,
    /// Maximum queued asynchronous storage jobs.
    pub surrealmx_max_async_queue_depth: usize,
    /// Maximum attached storage sessions.
    pub grpc_max_attached_sessions: usize,
    /// Accounted asynchronous storage queue byte capacity.
    pub surrealmx_max_async_queue_bytes: usize,
    /// Storage pressure threshold, distinct from the process hard cap.
    pub memory_threshold_bytes: usize,
    /// Maximum managed worker processes sharing this store.
    pub native_workers: usize,
    /// Hard process memory cap for each worker.
    pub native_worker_memory_bytes: usize,
    /// Optional materialized shared-runtime execution profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<NativeExecutionAllocation>,
}
/// Finite runtime allocation materialized by the existing deployment supervisor.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
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
impl NativeExecutionAllocation {
    /// Running and publication owners share the finite native population.
    pub(super) const POPULATION_PER_CASE: usize = 2;
}
impl NativeAllocation {
    /// Materialize the declared process envelope and the storage defaults once.
    /// # Errors
    /// The same relational validator refuses impossible allocations before effects.
    pub fn materialize(
        total: usize,
        server: usize,
        workers: usize,
        worker: usize,
        execution: Option<NativeExecutionAllocation>,
    ) -> Result<Self, WorkflowError> {
        let allocation = Self::declared(total, server, workers, worker, execution);
        allocation.validate()?;
        Ok(allocation)
    }
    fn declared(
        total: usize,
        server: usize,
        workers: usize,
        worker: usize,
        execution: Option<NativeExecutionAllocation>,
    ) -> Self {
        Self {
            total_memory_bytes: total,
            server_memory_bytes: server,
            native_workers: workers,
            native_worker_memory_bytes: worker,
            execution,
            surrealmx_memory_limit_bytes: server / 2,
            surrealmx_max_transaction_age_seconds: 90,
            surrealmx_snapshot_interval_seconds: 60,
            surrealmx_max_async_queue_depth: 32,
            grpc_max_attached_sessions: 256,
            surrealmx_max_async_queue_bytes: (64 * 1024 * 1024).min(server / 16),
            memory_threshold_bytes: server / 4 * 3 + server % 4 * 3 / 4,
        }
    }
    /// Validate the selected finite profile without inferring process RSS from pool capacity.
    /// # Errors
    /// Invalid populations or a runtime allocation that exceeds the process cap.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.server_memory_bytes < 512 * 1024 * 1024
            || self.total_memory_bytes >= (1_usize << 63)
            || self.server_memory_bytes >= self.total_memory_bytes
            || !(1..=32).contains(&self.native_workers)
            || self.native_worker_memory_bytes < 64 * 1024 * 1024
            || self.surrealmx_memory_limit_bytes == 0
            || self.surrealmx_memory_limit_bytes > self.server_memory_bytes
            || self.surrealmx_max_transaction_age_seconds == 0
            || self.surrealmx_snapshot_interval_seconds == 0
            || self.surrealmx_max_async_queue_depth == 0
            || self.grpc_max_attached_sessions == 0
            || self.surrealmx_max_async_queue_bytes == 0
            || self.surrealmx_max_async_queue_bytes > self.server_memory_bytes
            || self.memory_threshold_bytes == 0
            || self.memory_threshold_bytes >= self.server_memory_bytes
        {
            return Err(super::contract("finite native-worker allocation required"));
        }
        if let Some(profile) = self.execution
            && (profile.pool_memory_bytes == 0
                || profile.worker_bytes == 0
                || profile.worker_bytes > profile.pool_memory_bytes
                || profile.cpu_threads == 0
                || profile.case_lanes == 0
                || profile.case_lanes > profile.cpu_threads
                || profile
                    .case_lanes
                    .checked_mul(NativeExecutionAllocation::POPULATION_PER_CASE)
                    .is_none_or(|population| population > profile.math_jobs)
                || profile.compiler_cores == 0
                || profile.compiler_cores > profile.cpu_threads
                || profile.admission_wait_ms == 0
                || profile.process_headroom_bytes == 0
                || profile.observer_memory_bytes == 0
                || profile
                    .pool_memory_bytes
                    .checked_add(profile.process_headroom_bytes)
                    .is_none_or(|bytes| bytes != self.native_worker_memory_bytes))
        {
            return Err(super::contract(
                "shared execution profile exceeds the managed allocation",
            ));
        }
        let observer = self
            .execution
            .map_or(0, |profile| profile.observer_memory_bytes);
        let demand = self
            .native_workers
            .checked_mul(self.native_worker_memory_bytes)
            .and_then(|workers| workers.checked_add(self.server_memory_bytes))
            .and_then(|bytes| bytes.checked_add(observer));
        if demand.is_none_or(|bytes| bytes > self.total_memory_bytes) {
            return Err(super::contract(
                "storage, managed workers and observer exceed the declared envelope",
            ));
        }
        if *self
            != Self::declared(
                self.total_memory_bytes,
                self.server_memory_bytes,
                self.native_workers,
                self.native_worker_memory_bytes,
                self.execution,
            )
        {
            return Err(super::contract(
                "storage settings differ from the materialized allocation",
            ));
        }
        Ok(())
    }
}

#[path = "native_deployment_protocol.rs"]
mod protocol;
pub use protocol::*;

#[derive(Clone, Debug)]
struct ConfigurationObservation {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
}

/// One database-matched allocation and receiver selected from the deployment owner.
/// Native execution consumes this snapshot together; storage never loads it.
#[derive(Clone, Debug)]
pub struct NativeExecutionSelection {
    allocation: NativeAllocation,
    receiver: Option<ManagedPrimaryReceiver>,
    database: String,
    namespace: String,
    endpoint: String,
    receiver_state: PathBuf,
    configuration: Vec<ConfigurationObservation>,
    artifacts: Vec<pse_buildinfo::FileObservation>,
}
impl NativeExecutionSelection {
    /// Select native policy for an already connected canonical storage identity.
    /// # Errors
    /// Missing/invalid policy, changed storage identity, or stale receiver association.
    pub fn from_store(store: &CanonicalStore) -> Result<Self, WorkflowError> {
        let selected = Self::from_state(store.deployment_state(), Some(store.database()))?;
        if selected.namespace != store.namespace() || selected.endpoint != store.endpoint() {
            return Err(super::contract(
                "native selection differs from connected storage identity",
            ));
        }
        Ok(selected)
    }

    /// Interpret the native projection only at an explicit execution boundary.
    /// Registered contexts override the base profile as one database-matched selection.
    /// # Errors
    /// Invalid finite policy, context/generation mismatch, or changed executable bytes.
    pub fn from_state(state: &Path, database: Option<&str>) -> Result<Self, WorkflowError> {
        private_directory(state)?;
        let canonical_state = state.canonicalize().map_err(infrastructure)?;
        let state = canonical_state.as_path();
        let initial_path = state.join("config.json");
        let initial_bytes = private_file(&initial_path)?;
        let initial: pse_operations::deployment::ServiceIdentity =
            pse_operations::deployment::projection(&initial_bytes)
                .map_err(|error| super::contract(error.to_string()))?;
        let service_state = initial.service_state.as_deref().unwrap_or(state);
        private_directory(service_state)?;
        let service_path = service_state.join("config.json");
        let service_bytes = if service_state == state {
            initial_bytes.clone()
        } else {
            private_file(&service_path)?
        };
        let service: pse_operations::deployment::ServiceIdentity =
            pse_operations::deployment::projection(&service_bytes)
                .map_err(|error| super::contract(error.to_string()))?;
        if service.endpoint != initial.endpoint || service.namespace != initial.namespace {
            return Err(super::contract(
                "native deployment storage identity/version mismatch",
            ));
        }
        let database = database.unwrap_or(&initial.database);
        if database.is_empty()
            || database.len() > 128
            || !database
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(super::contract(
                "native database must be a bounded ASCII identifier",
            ));
        }
        let context_path = service_state
            .join(".contexts")
            .join(format!("{database}.json"));
        let mut configuration = vec![ConfigurationObservation {
            path: service_path,
            bytes: Some(service_bytes.clone()),
        }];
        let (policy, receiver_state) = if exists(&context_path)? {
            private_directory(&service_state.join(".contexts"))?;
            let context_bytes = private_file(&context_path)?;
            let context: RegisteredNativeContext = decode(&context_bytes)?;
            if context.database != database
                || !digest(&context.receiver_generation)
                || context.receiver_state
                    != service_state
                        .join(".receivers")
                        .join(&context.receiver_generation)
                        .join(database)
            {
                return Err(super::contract(
                    "registered native context identity/generation mismatch",
                ));
            }
            private_directory(&service_state.join(".receivers"))?;
            private_directory(
                &service_state
                    .join(".receivers")
                    .join(&context.receiver_generation),
            )?;
            private_directory(&context.receiver_state)?;
            let receiver_path = context.receiver_state.join("config.json");
            let receiver_bytes = private_file(&receiver_path)?;
            let receiver_identity: pse_operations::deployment::ServiceIdentity =
                pse_operations::deployment::projection(&receiver_bytes)
                    .map_err(|error| super::contract(error.to_string()))?;
            let receiver_policy: ManagedExecutionPolicy =
                pse_operations::deployment::projection(&receiver_bytes)
                    .map_err(|error| super::contract(error.to_string()))?;
            if receiver_identity.database != database
                || receiver_identity.namespace != service.namespace
                || receiver_identity.endpoint != service.endpoint
                || receiver_identity.service_state.as_deref() != Some(service_state)
                || receiver_identity.receiver_generation.as_deref()
                    != Some(context.receiver_generation.as_str())
                || receiver_policy.resources != context.policy.resources
                || receiver_policy.primary_receiver != context.policy.primary_receiver
            {
                return Err(super::contract(
                    "receiver configuration does not match registered native generation",
                ));
            }
            configuration.push(ConfigurationObservation {
                path: context_path,
                bytes: Some(context_bytes),
            });
            configuration.push(ConfigurationObservation {
                path: receiver_path,
                bytes: Some(receiver_bytes),
            });
            (context.policy, context.receiver_state)
        } else {
            let policy: ManagedExecutionPolicy =
                pse_operations::deployment::projection(&service_bytes)
                    .map_err(|error| super::contract(error.to_string()))?;
            if database != service.database
                && (policy.resources.execution.is_some() || policy.primary_receiver.is_some())
            {
                return Err(super::contract(
                    "managed execution requires a database-matched registered context",
                ));
            }
            configuration.push(ConfigurationObservation {
                path: context_path,
                bytes: None,
            });
            (policy, service_state.to_owned())
        };
        if initial.service_state.is_some()
            && (state != receiver_state || initial.database != database)
        {
            return Err(super::contract(
                "selected receiver state is stale for this native context",
            ));
        }
        policy.resources.validate()?;
        let mut artifacts = Vec::new();
        if let Some(receiver) = &policy.primary_receiver {
            if policy.resources.execution.is_none()
                || !receiver.worker_executable.is_absolute()
                || !digest(&receiver.worker_sha256)
            {
                return Err(super::contract(
                    "managed primary requires an execution profile and exact executable association",
                ));
            }
            // Retained legacy profile readers still bind their scientific image.
            // No interpreter/controller artifact is authenticated or executed here.
            {
                let (path, expected) = (&receiver.worker_executable, &receiver.worker_sha256);
                let observed =
                    pse_buildinfo::FileObservation::capture(path).map_err(infrastructure)?;
                if &observed.sha256 != expected {
                    return Err(super::contract(
                        "managed receiver bytes changed; explicit readmission required",
                    ));
                }
                artifacts.push(observed);
            }
        }
        let selected = Self {
            allocation: policy.resources,
            receiver: policy.primary_receiver,
            database: database.to_owned(),
            namespace: service.namespace,
            endpoint: service.endpoint,
            receiver_state,
            configuration,
            artifacts,
        };
        selected.require_configuration_current()?;
        Ok(selected)
    }
    /// Finite process and shared execution policy from this single selection.
    pub const fn allocation(&self) -> NativeAllocation {
        self.allocation
    }
    /// Configured managed receiver; legacy explicitly managed workers may omit it.
    pub fn receiver(&self) -> Option<&ManagedPrimaryReceiver> {
        self.receiver.as_ref()
    }
    /// Database bound to this execution selection.
    pub fn database(&self) -> &str {
        &self.database
    }
    /// Exact registered receiver state, or the base state for its selected database.
    pub fn receiver_state(&self) -> &Path {
        &self.receiver_state
    }
    /// Require both the finite shared policy and executable association for primary serving.
    /// # Errors
    /// A storage-only or legacy worker profile cannot serve a managed primary.
    pub fn managed_primary(
        &self,
    ) -> Result<(&ManagedPrimaryReceiver, NativeExecutionAllocation), WorkflowError> {
        let profile = self
            .allocation
            .execution
            .ok_or_else(|| super::contract("durable study requires a managed execution profile"))?;
        let receiver = self.receiver.as_ref().ok_or_else(|| {
            super::contract("durable study requires a configured managed primary receiver")
        })?;
        Ok((receiver, profile))
    }
    /// Refuse stale configuration or replaced executable bytes before reporting readiness.
    /// This is separate from the supervisor's actual finite placement and receiver admission.
    /// # Errors
    /// Any selected configuration or executable changed since selection.
    pub fn require_current(&self) -> Result<(), WorkflowError> {
        self.require_configuration_current()?;
        for artifact in &self.artifacts {
            artifact.verify().map_err(infrastructure)?;
        }
        Ok(())
    }
    fn require_configuration_current(&self) -> Result<(), WorkflowError> {
        for observed in &self.configuration {
            let current = if exists(&observed.path)? {
                Some(private_file(&observed.path)?)
            } else {
                None
            };
            if current != observed.bytes {
                return Err(super::contract(
                    "native execution configuration changed during selection/admission",
                ));
            }
        }
        Ok(())
    }
}
fn digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn infrastructure(error: std::io::Error) -> WorkflowError {
    CanonicalError::Configuration(error.to_string()).into()
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, WorkflowError> {
    serde_json::from_slice(bytes)
        .map_err(|error| super::contract(format!("native execution configuration: {error}")))
}
fn private_directory(path: &Path) -> Result<(), WorkflowError> {
    let metadata = std::fs::symlink_metadata(path).map_err(infrastructure)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(super::contract(
            "native state must be a private real directory",
        ));
    }
    private_owner(&metadata)
}
fn private_owner(metadata: &std::fs::Metadata) -> Result<(), WorkflowError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.mode() & 0o077 != 0 {
            return Err(super::contract("native configuration must be private"));
        }
        #[cfg(target_os = "linux")]
        if metadata.uid()
            != std::fs::metadata("/proc/self")
                .map_err(infrastructure)?
                .uid()
        {
            return Err(super::contract(
                "native configuration must belong to the current application user",
            ));
        }
    }
    Ok(())
}
fn private_file(path: &Path) -> Result<Vec<u8>, WorkflowError> {
    let metadata = std::fs::symlink_metadata(path).map_err(infrastructure)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > pse_operations::deployment::CONFIGURATION_FILE_BYTES
    {
        return Err(super::contract(
            "native configuration must be a bounded real file",
        ));
    }
    private_owner(&metadata)?;
    std::fs::read(path).map_err(infrastructure)
}

fn exists(path: &Path) -> Result<bool, WorkflowError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(infrastructure(error)),
    }
}
