// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Application-owned scientific process, separate from canonical control ingress.

use super::{NativeAllocation, Runtime, WorkerSettings, WorkflowError, contract};
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};
use pse_host::{
    BorrowedService, HostManager, LaunchIntent, OwnedUnit, ResourceControls, ServiceSpec,
    UnitObservation,
};
use pse_operations::canonical::{
    CanonicalOptions, CanonicalStore,
    migration::{SchemaFence, SchemaReady},
};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const DOCUMENT_BYTES: u64 = 64 * 1024;

fn infrastructure(error: impl std::fmt::Display) -> WorkflowError {
    WorkflowError::Internal(format!("native study worker: {error}"))
}

/// Exact connection identity and explicit execution policy for one scientific process.
/// Construction reads no legacy receiver or frozen-controller configuration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeStudyWorkerInputs {
    version: u32,
    state: PathBuf,
    endpoint: String,
    namespace: String,
    database: String,
    fence: SchemaFence,
    unit: String,
    allocation: NativeAllocation,
    settings: WorkerSettings,
    nonce: String,
    ready: PathBuf,
    terminal: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Ready {
    nonce: String,
    pid: u32,
    invocation: String,
    endpoint: String,
    namespace: String,
    database: String,
    fence: SchemaFence,
    allocation: NativeAllocation,
    settings: WorkerSettings,
    diagnostics: serde_json::Value,
}

/// Original application completion, distinct from canonical scientific receipts
/// and from the host owner's proof of empty descendants.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeWorkerTerminal {
    nonce: String,
    pid: u32,
    invocation: String,
    /// Whether the original application completed its native-lane shutdown.
    pub native_joined: bool,
    /// Count of scheduling actions, never a count of successful scientific outcomes.
    pub processed: Option<usize>,
    /// Original serving/shutdown/connection failure, if any.
    pub failure: Option<String>,
}

fn private_read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, WorkflowError> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)
        .map_err(infrastructure)?;
    let metadata = file.metadata().map_err(infrastructure)?;
    let owner = fs::metadata("/proc/self").map_err(infrastructure)?.uid();
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.mode() & 0o077 != 0
        || metadata.len() > DOCUMENT_BYTES
    {
        return Err(contract(
            "native worker document must be private, bounded and owned by this user",
        ));
    }
    let mut bytes = Vec::new();
    file.take(DOCUMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(infrastructure)?;
    if bytes.len() as u64 > DOCUMENT_BYTES {
        return Err(contract("native worker document extent"));
    }
    serde_json::from_slice(&bytes).map_err(infrastructure)
}

fn private_write(path: &Path, value: &impl serde::Serialize) -> Result<(), WorkflowError> {
    use std::os::unix::fs::OpenOptionsExt;
    let bytes = serde_json::to_vec(value).map_err(infrastructure)?;
    if bytes.len() as u64 > DOCUMENT_BYTES {
        return Err(contract("native worker document extent"));
    }
    let temporary = path.with_extension("temporary");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(infrastructure)?;
    file.write_all(&bytes).map_err(infrastructure)?;
    file.sync_all().map_err(infrastructure)?;
    fs::rename(&temporary, path).map_err(infrastructure)?;
    fs::File::open(
        path.parent()
            .ok_or_else(|| contract("native worker document parent"))?,
    )
    .and_then(|directory| directory.sync_all())
    .map_err(infrastructure)
}

impl NativeStudyWorkerInputs {
    /// Read the original parent's private launch document; never provision or rebuild.
    /// # Errors
    /// Invalid private bytes, unsupported policy or changed handoff paths.
    pub fn read(path: &Path) -> Result<Self, WorkflowError> {
        let inputs: Self = private_read(path)?;
        inputs.validate()?;
        if path.parent() != inputs.ready.parent() {
            return Err(contract(
                "native worker input and reply paths must share the original private directory",
            ));
        }
        Ok(inputs)
    }
    fn validate(&self) -> Result<(), WorkflowError> {
        self.allocation.validate()?;
        if self.version != 1
            || !self.state.is_absolute()
            || !self.ready.is_absolute()
            || !self.terminal.is_absolute()
            || self.ready.parent() != self.terminal.parent()
            || self.nonce.len() != 32
            || !self.nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
            || self.settings.poll.is_zero()
            || self.settings.recovery.is_zero()
            || self.settings.until_idle
            || self.settings.maximum_actions.is_some()
            || self.settings.maximum_in_flight == Some(0)
            || !self.unit.ends_with(".service")
            || self.unit.len() > 255
        {
            return Err(contract(
                "native scientific worker requires bounded sustained execution inputs",
            ));
        }
        if let Some(execution) = self.allocation.execution
            && self
                .settings
                .maximum_in_flight
                .is_none_or(|width| width > execution.case_lanes)
        {
            return Err(contract(
                "native worker case width exceeds its explicit allocation",
            ));
        }
        Ok(())
    }
    /// Original materialized scientific allocation.
    pub const fn allocation(&self) -> NativeAllocation {
        self.allocation
    }
    /// Original canonical database, with no receiver-context reinterpretation.
    pub fn database(&self) -> &str {
        &self.database
    }
    /// Canonical connection state prepared by the application's service owner.
    pub fn state(&self) -> &Path {
        &self.state
    }
    /// Original sustained scientific scheduling policy.
    pub const fn settings(&self) -> WorkerSettings {
        self.settings
    }
    /// Open supported schema-ready canonical state without initialization or migration.
    /// # Errors
    /// Authentication/schema admission, connection identity or connection cleanup failed.
    pub async fn connect(&self) -> Result<CanonicalStore, WorkflowError> {
        self.validate()?;
        // A copied environment marker is not native worker admission. This must
        // be the scientific service's actual manager-owned main application.
        let manager = HostManager::connect_user(None)
            .await
            .map_err(infrastructure)?;
        let current = tokio::time::timeout(Duration::from_secs(5), manager.current_unit())
            .await
            .map_err(infrastructure)?
            .map_err(infrastructure)?;
        if current.unit != self.unit
            || current.main_pid != std::process::id()
            || !matches!(current.active_state.as_str(), "active" | "activating")
        {
            return Err(contract(
                "scientific bootstrap is not the original native application service",
            ));
        }
        let options = CanonicalOptions::from_state_for_database(&self.state, Some(&self.database))?;
        let ready = CanonicalStore::connect_ready(&options).await?;
        let same_fence = ready.fence() == &self.fence;
        let store = ready.into_store();
        let identity = if same_fence {
            self.require_store(&store)
        } else {
            Err(contract(
                "native worker schema opening premise changed before launch",
            ))
        };
        if let Err(error) = identity {
            let disconnected = store.disconnect().await;
            return match disconnected {
                Ok(()) => Err(error),
                Err(cleanup) => Err(WorkflowError::Settlement {
                    operation: Box::new(error),
                    cleanup: Box::new(cleanup.into()),
                }),
            };
        }
        Ok(store)
    }
    fn require_store(&self, store: &CanonicalStore) -> Result<(), WorkflowError> {
        if self.state != store.deployment_state()
            || self.endpoint != store.endpoint()
            || self.namespace != store.namespace()
            || self.database != store.database()
            || self.fence.issuer() != store.issuing_store()?
        {
            return Err(contract(
                "native worker canonical connection differs from its admitted service",
            ));
        }
        Ok(())
    }
    /// Called only after the actual scientific runtime has been constructed.
    /// # Errors
    /// Runtime/connection identity, manager invocation or private publication failed.
    pub fn publish_ready(&self, runtime: &Runtime) -> Result<(), WorkflowError> {
        let store = runtime.canonical_store()?;
        self.require_store(store)?;
        let invocation = std::env::var("INVOCATION_ID").map_err(infrastructure)?;
        if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(contract("native worker manager invocation is unavailable"));
        }
        private_write(
            &self.ready,
            &Ready {
                nonce: self.nonce.clone(),
                pid: std::process::id(),
                invocation,
                endpoint: self.endpoint.clone(),
                namespace: self.namespace.clone(),
                database: self.database.clone(),
                fence: self.fence.clone(),
                allocation: self.allocation,
                settings: self.settings,
                diagnostics: serde_json::to_value(crate::observation::ReceiverCheckpoint {
                    available: true,
                    receiver_state: self
                        .ready
                        .parent()
                        .ok_or_else(|| contract("native worker diagnostics directory missing"))?
                        .to_path_buf(),
                    canonical_database: self.database.clone(),
                    nonce: self.nonce.clone(),
                    identity: runtime.observations().checkpoint_identity(),
                    checkpoint: runtime.observations().checkpoint_path(),
                    enabled: runtime.observations().hub().policy().enabled,
                    writer_failure: runtime.observations().writer_failure(),
                })
                .map_err(infrastructure)?,
            },
        )
    }
    /// Publish the original application's join result after serving and native shutdown.
    /// # Errors
    /// Manager identity or private terminal publication failed.
    pub fn publish_terminal(
        &self,
        native_joined: bool,
        result: &Result<usize, WorkflowError>,
    ) -> Result<(), WorkflowError> {
        private_write(
            &self.terminal,
            &NativeWorkerTerminal {
                nonce: self.nonce.clone(),
                pid: std::process::id(),
                invocation: std::env::var("INVOCATION_ID").map_err(infrastructure)?,
                native_joined,
                processed: result.as_ref().ok().copied(),
                failure: result.as_ref().err().map(ToString::to_string),
            },
        )
    }
}

/// Already-prepared native image/environment and an explicit scientific allocation.
/// The caller supplies the real schema-ready store; no implicit service setup occurs.
pub struct NativeStudyWorkerSpec<'a> {
    /// Supported, connected canonical application store.
    pub ready: &'a SchemaReady,
    /// Already-built scientific `pse-worker`, not the canonical control helper.
    pub executable: PathBuf,
    /// Complete already-prepared systemd-compatible child environment; no global mutation or preparation.
    pub environment: BTreeMap<String, String>,
    /// Original finite scientific profile, distinct from ordinary host sharing.
    /// The application retains aggregate service/observer allocation ownership;
    /// this scientific root enforces its own worker cap and execution width.
    pub allocation: NativeAllocation,
    /// Sustained worker settings.
    pub settings: WorkerSettings,
}

impl std::fmt::Debug for NativeStudyWorkerSpec<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeStudyWorkerSpec")
            .field("executable", &self.executable)
            .field("allocation", &self.allocation)
            .field("settings", &self.settings)
            .field(
                "environment_keys",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

/// Original application root retains this performing owner through launch errors,
/// caller cancellation and explicit shutdown. Drop never establishes settlement.
#[must_use = "retain the scientific application root and explicitly shutdown/dispose it"]
pub struct NativeStudyWorker {
    inputs: NativeStudyWorkerInputs,
    directory: PathBuf,
    directory_identity: (u64, u64),
    executable: pse_buildinfo::FileObservation,
    spec: ServiceSpec,
    controls: ResourceControls,
    manager: Option<HostManager>,
    unit: Option<OwnedUnit>,
    dispatched: bool,
    closing: bool,
    observer: Option<NativeStudyWorkerObserver>,
    drained: Arc<AtomicBool>,
}

impl std::fmt::Debug for NativeStudyWorker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeStudyWorker")
            .field("intent", &self.spec.intent)
            .field("directory", &self.directory)
            .field("dispatched", &self.dispatched)
            .field("closing", &self.closing)
            .field("physically_drained", &self.drained.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

fn service_environment_entry(key: &str, value: &str) -> Result<String, WorkflowError> {
    let mut bytes = key.bytes();
    if !bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || value.contains('\0')
    {
        return Err(contract(
            "native worker environment names must be systemd-compatible ASCII identifiers and values must contain no NUL",
        ));
    }
    Ok(format!("{key}={value}"))
}

impl NativeStudyWorker {
    /// Construct one private launch handoff, with no host/service effects.
    /// # Errors
    /// Invalid scientific resources, unavailable prepared image or private handoff failure.
    pub fn new(spec: NativeStudyWorkerSpec<'_>) -> Result<Self, WorkflowError> {
        spec.allocation.validate()?;
        let store = spec.ready.store();
        let executable_path = spec.executable.canonicalize().map_err(infrastructure)?;
        if executable_path
            .file_name()
            .is_none_or(|name| name != "pse-worker")
        {
            return Err(contract(
                "native study requires the scientific pse-worker executable",
            ));
        }
        let executable =
            pse_buildinfo::FileObservation::capture(&executable_path).map_err(infrastructure)?;
        let intent = LaunchIntent::fresh().map_err(infrastructure)?;
        let temporary = tempfile::Builder::new()
            .prefix("pse-science-worker-")
            .tempdir()
            .map_err(infrastructure)?;
        let directory = temporary.path().to_path_buf();
        let mut settings = spec.settings;
        if let Some(execution) = spec.allocation.execution {
            settings
                .maximum_in_flight
                .get_or_insert(execution.case_lanes);
        }
        let inputs = NativeStudyWorkerInputs {
            version: 1,
            state: store.deployment_state().to_path_buf(),
            unit: intent.unit().to_owned(),
            endpoint: store.endpoint().into(),
            namespace: store.namespace().into(),
            database: store.database().into(),
            fence: spec.ready.fence().clone(),
            allocation: spec.allocation,
            settings,
            nonce: pse_operations::mint_id::<pse_ids::SemanticId>().to_string(),
            ready: directory.join("ready.json"),
            terminal: directory.join("terminal.json"),
        };
        inputs.validate()?;
        let path = directory.join("inputs.json");
        private_write(&path, &inputs)?;
        let cpu_quota = spec
            .allocation
            .execution
            .map(|execution| {
                u64::try_from(execution.cpu_threads)
                    .ok()
                    .and_then(|threads| threads.checked_mul(1_000_000))
                    .ok_or_else(|| contract("native worker CPU quota extent overflow"))
            })
            .transpose()?;
        let controls = ResourceControls {
            memory_max: Some(spec.allocation.native_worker_memory_bytes as u64),
            memory_swap_max: Some(0),
            cpu_quota_per_sec_usec: cpu_quota,
            ..Default::default()
        };
        let mut selected_environment = spec.environment;
        selected_environment.insert(
            "PSE_SURREAL_STATE".into(),
            inputs
                .state
                .to_str()
                .ok_or_else(|| contract("native canonical state path must be UTF-8"))?
                .into(),
        );
        let environment = selected_environment
            .into_iter()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    "INVOCATION_ID"
                        | "SYSTEMD_EXEC_PID"
                        | "NOTIFY_SOCKET"
                        | "LISTEN_PID"
                        | "LISTEN_FDS"
                        | "LISTEN_FDNAMES"
                )
            })
            .map(|(key, value)| service_environment_entry(&key, &value))
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        let path = path
            .to_str()
            .ok_or_else(|| contract("native worker input path must be UTF-8"))?
            .to_owned();
        let launch = ServiceSpec {
            intent,
            executable: executable_path,
            arguments: vec!["--native-input".into(), path],
            environment,
        };
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::symlink_metadata(&directory).map_err(infrastructure)?;
        let directory_identity = (metadata.dev(), metadata.ino());
        Ok(Self {
            inputs,
            directory: temporary.keep(),
            directory_identity,
            executable,
            spec: launch,
            controls,
            manager: None,
            unit: None,
            dispatched: false,
            closing: false,
            observer: None,
            drained: Arc::new(AtomicBool::new(false)),
        })
    }
    /// Original manager intent for diagnosis, never authority to adopt another worker.
    pub fn intent(&self) -> &LaunchIntent {
        &self.spec.intent
    }
    /// Retained startup/terminal diagnostics, including after uncertain launch.
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    /// Select the existing bounded native-entry qualification control before launch.
    /// Refusal returns the original owner, including any already-dispatched ownership.
    /// # Errors
    /// Launch is closed/dispatched or the control path is not absolute UTF-8.
    #[cfg(feature = "canonical-tests")]
    pub fn with_qualification_native_entry(
        mut self,
        path: PathBuf,
    ) -> Result<Self, Box<(Self, WorkflowError)>> {
        let argument = match path.to_str() {
            Some(value) if path.is_absolute() => value.to_owned(),
            _ => {
                return Err(Box::new((
                    self,
                    contract("native entry qualification requires an absolute UTF-8 path"),
                )));
            }
        };
        if self.closing || self.dispatched {
            return Err(Box::new((
                self,
                contract("native entry qualification must precede original launch"),
            )));
        }
        self.spec
            .arguments
            .extend(["--qualification-native-entry".into(), argument]);
        Ok(self)
    }
    /// Submit once and wait for application readiness under one original absolute clock.
    /// Errors/cancellation leave this original owner available for shutdown/reconciliation.
    /// # Errors
    /// Original launch, identity, application readiness or the absolute clock failed.
    pub async fn launch(&mut self, deadline: Instant) -> Result<(), WorkflowError> {
        if self.closing || self.dispatched {
            return Err(contract(
                "native worker launch is closed or already dispatched; reconcile original ownership",
            ));
        }
        self.executable.verify().map_err(infrastructure)?;
        let manager = tokio::time::timeout_at(deadline.into(), HostManager::connect_user(None))
            .await
            .map_err(infrastructure)?
            .map_err(infrastructure)?;
        let enclosing = tokio::time::timeout_at(deadline.into(), manager.current_unit())
            .await
            .map_err(infrastructure)?
            .map_err(infrastructure)?;
        self.manager = Some(manager.clone());
        self.dispatched = true;
        self.unit = Some(
            manager
                .start_service_bound(
                    &self.spec,
                    &manager.borrow(enclosing.unit),
                    &self.controls,
                    deadline,
                )
                .await
                .map_err(infrastructure)?,
        );
        let observer = tokio::time::timeout_at(deadline.into(), async {
            let unit = self.unit.as_ref().ok_or_else(|| contract("native worker owner absent"))?;
            loop {
                let observed = unit.observe().await.map_err(infrastructure)?;
                if observed.main_pid == 0 || matches!(observed.active_state.as_str(), "failed" | "inactive") {
                    return Err(contract("native scientific worker exited before application readiness"));
                }
                if self.inputs.ready.try_exists().map_err(infrastructure)? {
                    let ready: Ready = private_read(&self.inputs.ready)?;
                    require_ready(&self.inputs, &observed, &ready)?;
                    let process = match ProcessReader::default().process(observed.main_pid) {
                        ProcessRead::Present(facts) => facts.identity,
                        _ => return Err(contract("native worker original process identity unavailable")),
                    };
                    let borrowed = unit.borrow_service(deadline).await.map_err(infrastructure)?;
                    let check = borrowed.observe(deadline).await.map_err(infrastructure)?;
                    require_ready(&self.inputs, &check, &ready)?;
                    if fs::read_link(format!("/proc/{}/exe", ready.pid)).map_err(infrastructure)? != self.executable.canonical {
                        return Err(contract("native scientific application image differs from its selected executable"));
                    }
                    self.executable.verify().map_err(infrastructure)?;
                    return Ok(NativeStudyWorkerObserver { inputs: self.inputs.clone(), borrowed, process,
                        invocation: ready.invocation, diagnostics: ready.diagnostics, drained: self.drained.clone() });
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.map_err(infrastructure)??;
        self.observer = Some(observer);
        Ok(())
    }
    /// Lend observation only after application readiness; caller keeps this owner.
    /// # Errors
    /// Root admission is closed or application readiness has not been established.
    pub fn observer(&self) -> Result<NativeStudyWorkerObserver, WorkflowError> {
        if self.closing {
            return Err(contract("native worker root admission is closed"));
        }
        self.observer
            .clone()
            .ok_or_else(|| contract("native worker application readiness not established"))
    }
    /// Stop and drain the original host population without consulting the database.
    /// Failure retains the original manager, intent and OwnedUnit for another observation.
    /// # Errors
    /// Original launch reconciliation or physical drain is unknown or failed.
    pub async fn shutdown(
        &mut self,
        deadline: Instant,
    ) -> Result<Option<UnitObservation>, WorkflowError> {
        self.closing = true;
        if self.drained.load(Ordering::Acquire) {
            return Ok(None);
        }
        if self.dispatched && self.unit.is_none() {
            self.unit = self
                .manager
                .as_ref()
                .ok_or_else(|| contract("original worker manager unavailable"))?
                .reconcile_if_present(&self.spec.intent, deadline)
                .await
                .map_err(infrastructure)?;
        }
        let result = if let Some(unit) = &mut self.unit {
            if unit.released() {
                None
            } else {
                let result = unit.stop_and_drain(deadline).await;
                if unit.released() {
                    self.drained.store(true, Ordering::Release);
                }
                Some(result.map_err(infrastructure)?)
            }
        } else {
            None
        };
        self.drained.store(true, Ordering::Release);
        Ok(result)
    }
    /// Abruptly terminate only the original owned scientific population for a
    /// qualification control. Physical drain never fabricates an application join.
    /// # Errors
    /// The original performing owner is absent or its physical drain is unknown.
    #[cfg(feature = "canonical-tests")]
    pub async fn kill_and_drain_for_qualification(
        &mut self,
        deadline: Instant,
    ) -> Result<UnitObservation, WorkflowError> {
        self.closing = true;
        let unit = self
            .unit
            .as_mut()
            .ok_or_else(|| contract("original native worker launch owner absent"))?;
        let observed = unit.kill_and_drain(deadline).await;
        if unit.released() {
            self.drained.store(true, Ordering::Release);
        }
        observed.map_err(infrastructure)
    }
    /// Read the separate original application join receipt. Missing/partial bytes
    /// remain unavailable even after a successful host drain.
    /// # Errors
    /// Partial/private bytes or the original application identity could not be verified.
    pub fn terminal(&self) -> Result<Option<NativeWorkerTerminal>, WorkflowError> {
        if !self.inputs.terminal.try_exists().map_err(infrastructure)? {
            return Ok(None);
        }
        let terminal: NativeWorkerTerminal = private_read(&self.inputs.terminal)?;
        let observer = self
            .observer
            .as_ref()
            .ok_or_else(|| contract("original application readiness identity unavailable"))?;
        if terminal.nonce != self.inputs.nonce
            || terminal.pid != observer.process.pid
            || terminal.invocation != observer.invocation
        {
            return Err(contract("native worker terminal receipt identity differs"));
        }
        Ok(Some(terminal))
    }
    /// Remove only this owner's private handoff after proven physical settlement.
    /// # Errors
    /// Physical release is unknown, the directory changed or removal failed.
    pub fn dispose(&mut self) -> Result<(), WorkflowError> {
        if !self.drained.load(Ordering::Acquire) {
            return Err(contract("native worker physical drain is not established"));
        }
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::symlink_metadata(&self.directory).map_err(infrastructure)?;
        if !metadata.is_dir() || (metadata.dev(), metadata.ino()) != self.directory_identity {
            return Err(contract(
                "original native worker private directory identity changed",
            ));
        }
        fs::remove_dir_all(&self.directory).map_err(infrastructure)
    }
}

fn require_ready(
    inputs: &NativeStudyWorkerInputs,
    observed: &UnitObservation,
    ready: &Ready,
) -> Result<(), WorkflowError> {
    let invocation = observed
        .invocation
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if ready.nonce != inputs.nonce
        || ready.pid != observed.main_pid
        || ready.invocation != invocation
        || ready.endpoint != inputs.endpoint
        || ready.namespace != inputs.namespace
        || ready.database != inputs.database
        || ready.fence != inputs.fence
        || ready.allocation != inputs.allocation
        || ready.settings != inputs.settings
    {
        return Err(contract(
            "native scientific application readiness differs from original launch/connection",
        ));
    }
    Ok(())
}

/// Identity-pinned read-only borrow. It cannot stop, drain, release or keep a
/// process alive. Loss of its leader is not proof of empty descendants.
#[derive(Clone, Debug)]
pub struct NativeStudyWorkerObserver {
    inputs: NativeStudyWorkerInputs,
    borrowed: BorrowedService,
    process: ProcessIdentity,
    invocation: String,
    diagnostics: serde_json::Value,
    drained: Arc<AtomicBool>,
}
impl NativeStudyWorkerObserver {
    pub(super) fn require_store(
        &self,
        store: &CanonicalStore,
        observer_bytes: usize,
    ) -> Result<(), WorkflowError> {
        self.inputs.require_store(store)?;
        let execution = self.inputs.allocation.execution.ok_or_else(|| {
            contract("durable study requires an explicit native execution allocation")
        })?;
        if observer_bytes > execution.observer_memory_bytes {
            return Err(contract(
                "durable observer pool exceeds its explicit scientific observer allocation",
            ));
        }
        Ok(())
    }
    pub(super) async fn observe(&self) -> Result<super::worker::PrimaryObservation, WorkflowError> {
        use super::worker::PrimaryObservation;
        if self.drained.load(Ordering::Acquire) {
            return Ok(PrimaryObservation::Drained);
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        match self.borrowed.observe(deadline).await {
            Ok(_) => Ok(PrimaryObservation::Live),
            Err(error) => {
                if original_leader_ended(
                    &self.process,
                    ProcessReader::default().process(self.process.pid),
                ) {
                    Ok(PrimaryObservation::OwnerChanged)
                } else {
                    // Unknown manager/process evidence never becomes physical drain.
                    Err(infrastructure(error))
                }
            }
        }
    }
}

fn original_leader_ended(
    original: &ProcessIdentity,
    current: ProcessRead<pse_diagnostics::process_facts::ProcessFacts>,
) -> bool {
    match current {
        ProcessRead::Absent => true,
        ProcessRead::Present(facts) => facts.identity != *original,
        ProcessRead::Unavailable(_) | ProcessRead::Changed => false,
    }
}
impl Runtime {
    /// Bind the original application worker's read-only readiness capability.
    /// The application retains NativeStudyWorker separately until explicit shutdown.
    /// # Errors
    /// Canonical identity or the separately allocated observer pool does not match.
    pub fn with_managed_worker(
        mut self,
        observer: NativeStudyWorkerObserver,
    ) -> Result<Self, WorkflowError> {
        observer.require_store(
            self.canonical_store()?,
            self.shared.budget().memory_limit_bytes.get(),
        )?;
        self.observations().observe_receiver_ready(
            observer
                .inputs
                .ready
                .parent()
                .ok_or_else(|| contract("native worker diagnostic directory missing"))?,
            &observer.inputs.database,
            Some(observer.diagnostics.clone()),
        );
        self.managed_worker = Some(observer);
        Ok(self)
    }
}

#[cfg(test)]
#[path = "native_worker_tests.rs"]
mod tests;
