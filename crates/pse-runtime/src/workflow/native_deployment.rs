// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native execution selection at the runtime composition boundary.

use super::WorkflowError;
use pse_operations::canonical::{CanonicalError, CanonicalStore};
use std::path::{Path, PathBuf};

/// Materialized executable association for the managed study receiver.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
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
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if !(1..=32).contains(&self.native_workers) || self.native_worker_memory_bytes == 0 {
            return Err(super::contract("finite native-worker allocation required"));
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
            return Err(super::contract(
                "shared execution profile exceeds the managed allocation",
            ));
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct StorageIdentity {
    profile_version: u32,
    endpoint: String,
    namespace: String,
    database: String,
    #[serde(default)]
    service_state: Option<PathBuf>,
    #[serde(default)]
    receiver_generation: Option<String>,
}
#[derive(serde::Deserialize)]
struct NativePolicy {
    resources: NativeAllocation,
    #[serde(default)]
    primary_receiver: Option<ManagedPrimaryReceiver>,
}
#[derive(serde::Deserialize)]
struct RegisteredContext {
    version: u32,
    database: String,
    #[serde(flatten)]
    policy: NativePolicy,
    receiver_state: PathBuf,
    receiver_generation: String,
}
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
        let initial: StorageIdentity = decode(&initial_bytes)?;
        let service_state = initial.service_state.as_deref().unwrap_or(state);
        private_directory(service_state)?;
        let service_path = service_state.join("config.json");
        let service_bytes = if service_state == state {
            initial_bytes.clone()
        } else {
            private_file(&service_path)?
        };
        let service: StorageIdentity = decode(&service_bytes)?;
        if service.profile_version != 3
            || initial.profile_version != 3
            || service.endpoint != initial.endpoint
            || service.namespace != initial.namespace
        {
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
            let context: RegisteredContext = decode(&context_bytes)?;
            if context.version != 1
                || context.database != database
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
            let receiver_identity: StorageIdentity = decode(&receiver_bytes)?;
            let receiver_policy: NativePolicy = decode(&receiver_bytes)?;
            if receiver_identity.profile_version != 3
                || receiver_identity.database != database
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
            let policy: NativePolicy = decode(&service_bytes)?;
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
                || !receiver.supervisor_executable.is_absolute()
                || !receiver.supervisor_script.is_absolute()
                || !receiver.worker_executable.is_absolute()
                || !digest(&receiver.supervisor_sha256)
                || !digest(&receiver.worker_sha256)
            {
                return Err(super::contract(
                    "managed primary requires an execution profile and exact executable association",
                ));
            }
            // The frozen supervisor retains complete generation/placement/readiness admission.
            // Check the actually selected entry-point bytes before executing that supervisor.
            for (path, expected) in [
                (&receiver.supervisor_script, &receiver.supervisor_sha256),
                (&receiver.worker_executable, &receiver.worker_sha256),
            ] {
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
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
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

#[cfg(test)]
mod native_selection_unit {
    #![allow(clippy::unwrap_used, reason = "isolated native selection assertions")]
    use super::*;
    use serde_json::{Value, json};

    fn private_scope() -> tempfile::TempDir {
        use std::os::unix::fs::PermissionsExt;
        let scope = tempfile::tempdir().unwrap();
        std::fs::set_permissions(scope.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        scope
    }

    fn write(path: &Path, bytes: &[u8]) {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        file.write_all(bytes).unwrap();
    }
    fn document(path: &Path, value: &Value) {
        write(path, &serde_json::to_vec(value).unwrap());
    }
    fn directory(path: &Path) {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .unwrap();
    }
    fn policy() -> Value {
        json!({"native_workers": 1, "native_worker_memory_bytes": 1024,
            "execution": {"pool_memory_bytes": 512, "worker_bytes": 256,
                "cpu_threads": 2, "case_lanes": 2, "math_jobs": 4, "compiler_cores": 1,
                "admission_wait_ms": 1000, "process_headroom_bytes": 128,
                "observer_memory_bytes": 128}})
    }
    fn deployment(resources: Value, receiver: Value) -> Value {
        json!({"profile_version": 3, "endpoint": "grpc://127.0.0.1:8000",
            "namespace": "canonical", "database": "selected",
            "resources": resources, "primary_receiver": receiver})
    }
    fn receiver(root: &Path) -> Value {
        let script = root.join("supervisor.py");
        let worker = root.join("worker");
        write(&script, b"supervisor generation");
        write(&worker, b"worker generation");
        json!({"supervisor_executable": "/usr/bin/python3", "supervisor_script": script,
            "worker_executable": worker,
            "supervisor_sha256": pse_buildinfo::FileObservation::capture(&script).unwrap().sha256,
            "worker_sha256": pse_buildinfo::FileObservation::capture(&worker).unwrap().sha256})
    }
    fn context(root: &Path, database: &str, resources: &Value, receiver: &Value) -> PathBuf {
        let generation = "a".repeat(64);
        let state = root.join(".receivers").join(&generation).join(database);
        directory(&state);
        directory(&root.join(".contexts"));
        let mut config = deployment(resources.clone(), receiver.clone());
        config["database"] = database.into();
        config["receiver_generation"] = generation.clone().into();
        config["service_state"] = json!(root);
        document(&state.join("config.json"), &config);
        document(
            &root.join(".contexts").join(format!("{database}.json")),
            &json!({
                "version": 1, "database": database, "resources": resources,
                "primary_receiver": receiver, "receiver_state": state, "receiver_generation": generation,
            }),
        );
        state
    }
    #[test]
    fn execution_rejects_missing_malformed_or_nonfinite_policy_and_missing_primary() {
        let root = private_scope();
        let mut multiple_primaries = policy();
        multiple_primaries["native_workers"] = 2.into();
        for resources in [
            Value::Null,
            json!("malformed"),
            json!({"native_workers": 0, "native_worker_memory_bytes": 1024}),
            multiple_primaries,
        ] {
            document(
                &root.path().join("config.json"),
                &deployment(resources, Value::Null),
            );
            assert!(NativeExecutionSelection::from_state(root.path(), None).is_err());
        }
        let mut invalid = policy();
        invalid["execution"]["process_headroom_bytes"] = 1024.into();
        document(
            &root.path().join("config.json"),
            &deployment(invalid, Value::Null),
        );
        assert!(NativeExecutionSelection::from_state(root.path(), None).is_err());
        document(
            &root.path().join("config.json"),
            &deployment(policy(), Value::Null),
        );
        let selected = NativeExecutionSelection::from_state(root.path(), None).unwrap();
        assert!(selected.managed_primary().is_err());
        assert!(NativeExecutionSelection::from_state(root.path(), Some("unregistered")).is_err());
    }
    #[test]
    fn execution_checks_actual_receiver_bytes_and_refuses_replacement_after_selection() {
        let root = private_scope();
        let association = receiver(root.path());
        document(
            &root.path().join("config.json"),
            &deployment(policy(), association.clone()),
        );
        let selected = NativeExecutionSelection::from_state(root.path(), None).unwrap();
        assert_eq!(selected.managed_primary().unwrap().1.cpu_threads, 2);
        write(
            Path::new(association["worker_executable"].as_str().unwrap()),
            b"replacement",
        );
        assert!(selected.require_current().is_err());
        assert!(NativeExecutionSelection::from_state(root.path(), None).is_err());
    }
    #[test]
    fn registered_context_selects_one_coherent_profile_and_detects_stale_generations() {
        let root = private_scope();
        let association = receiver(root.path());
        // The unused base-native projection may be malformed; this database owns its policy.
        document(
            &root.path().join("config.json"),
            &deployment(json!("unused"), json!(17)),
        );
        let receiver_state = context(root.path(), "isolated", &policy(), &association);
        let selected = NativeExecutionSelection::from_state(root.path(), Some("isolated")).unwrap();
        assert_eq!(selected.database(), "isolated");
        assert_eq!(selected.receiver_state(), receiver_state);
        assert_eq!(selected.allocation().native_worker_memory_bytes, 1024);
        assert!(selected.managed_primary().is_ok());
        assert!(NativeExecutionSelection::from_state(&receiver_state, Some("isolated")).is_ok());
        let receiver_config = receiver_state.join("config.json");
        let mut config: Value =
            serde_json::from_slice(&std::fs::read(&receiver_config).unwrap()).unwrap();
        config["receiver_generation"] = "b".repeat(64).into();
        document(&receiver_config, &config);
        assert!(selected.require_current().is_err());
        assert!(NativeExecutionSelection::from_state(root.path(), Some("isolated")).is_err());
    }
    #[test]
    fn context_policy_or_database_mismatch_cannot_borrow_another_receivers_allocation() {
        let root = private_scope();
        let association = receiver(root.path());
        document(
            &root.path().join("config.json"),
            &deployment(policy(), association.clone()),
        );
        let state = context(root.path(), "isolated", &policy(), &association);
        let path = root.path().join(".contexts/isolated.json");
        let original: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        for (field, value) in [
            ("database", json!("other")),
            ("receiver_state", json!(root.path())),
            (
                "resources",
                json!({"native_workers": 1, "native_worker_memory_bytes": 4096}),
            ),
        ] {
            let mut invalid = original.clone();
            invalid[field] = value;
            document(&path, &invalid);
            assert!(NativeExecutionSelection::from_state(root.path(), Some("isolated")).is_err());
        }
        document(&path, &original);
        assert!(NativeExecutionSelection::from_state(&state, Some("other")).is_err());
    }
    #[test]
    fn native_selection_refuses_new_context_after_observing_base_policy() {
        let root = private_scope();
        let association = receiver(root.path());
        document(
            &root.path().join("config.json"),
            &deployment(policy(), association.clone()),
        );
        let selected = NativeExecutionSelection::from_state(root.path(), None).unwrap();
        context(root.path(), "selected", &policy(), &association);
        assert!(selected.require_current().is_err());
    }
    #[test]
    fn canonical_failures_retain_infrastructure_diagnostics_distinct_from_user_input() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
        for cause in [
            CanonicalError::Timeout,
            CanonicalError::Interpretation {
                expected: "required",
                observed: "installed".into(),
            },
            CanonicalError::Configuration(
                "disconnect refused while protected readers are live".into(),
            ),
        ] {
            let error = WorkflowError::from(cause);
            assert_eq!(
                error.diagnostic_code(),
                Some(DiagnosticCode::RuntimeInfrastructure)
            );
            assert!(matches!(error, WorkflowError::Canonical(_)));
        }
        assert_eq!(
            super::super::contract("reversed range").diagnostic_code(),
            Some(DiagnosticCode::WorkflowInput)
        );
    }
}
