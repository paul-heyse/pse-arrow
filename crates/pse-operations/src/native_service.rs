// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Application-owned disposable canonical service construction and physical settlement.
//!
//! This module owns only directories it creates. Ordinary host selection remains
//! database-free, and dropping a value never establishes process or test success.
use crate::{
    canonical::{CanonicalError, CanonicalOptions, CanonicalStore, MESSAGE_BYTES, migration::{SchemaReady, SchemaMaintenance}, study_module::StudyModuleReceipt},
    deployment::{CanonicalConnectionSettings, ServiceIdentity},
    transport::{client::{Client, RequestContext, Root}, complete_response},
};
use pse_host::{HostError, HostManager, LaunchIntent, OwnedUnit, ResourceControls, ServiceSpec, UnitObservation, server::{self, PreparedServer}};
use serde::{Deserialize, Serialize};
use futures_util::FutureExt;
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener},
    os::unix::{fs::{MetadataExt, OpenOptionsExt, PermissionsExt}, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const PRIVATE_BYTES: u64 = 256 * 1024;
const ROOT_USER: &str = "pse-fixture-owner";
const VIEWER_USER: &str = "pse-selection";

/// Host and canonical failures retain their existing infrastructure meanings.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(runtime::infrastructure))]
pub enum NativeServiceError {
    /// Original native process/unit failure.
    #[error(transparent)]
    Host(#[from] HostError),
    /// Original canonical request, admission or physical connection failure.
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    /// Private filesystem or native exec operation failed.
    #[error("native canonical service I/O: {0}")]
    Io(#[from] std::io::Error),
    /// This construction or observation cannot establish its premises.
    #[error("native canonical service: {0}")]
    Refused(&'static str),
    /// All independent teardown operations were attempted; retain the original owner.
    #[error("native canonical service retained at {} after {} settlement failures", .state.display(), .failures.len())]
    Settlement {
        /// Fresh fixture state retained for diagnosis and explicit retry.
        state: PathBuf,
        /// Every actual failure, rather than a successful-drop inference.
        #[related]
        failures: Vec<NativeServiceError>,
    },
}

/// Explicit already-prepared inputs; construction performs no source build or discovery.
#[derive(Clone, Debug)]
pub struct NativeServiceSpec {
    /// Exact immutable server capability, including admitted import provenance.
    pub server: PreparedServer,
    /// Explicit application helper executable built with native-service.
    pub helper: PathBuf,
    /// Existing private state supplies the selected original module receipt only.
    pub module_source_state: PathBuf,
    /// Parent for a newly created private disposable directory.
    pub state_parent: PathBuf,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Credentials {
    username: String,
    password: String,
    selection_username: String,
    selection_password: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StartupPayload {
    version: u32,
    state: PathBuf,
    generation: PathBuf,
    recipe: String,
    bind: SocketAddr,
    nonce: String,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StartupClaim { nonce: String, pid: u32, invocation: String }
#[derive(Serialize)]
struct Configuration {
    #[serde(flatten)]
    identity: ServiceIdentity,
    #[serde(flatten)]
    connection: CanonicalConnectionSettings,
    payload_incarnation: String,
    accepting_writes: bool,
    study_module: StudyModuleReceipt,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisposableIdentity { version: u32, nonce: String }
struct OwnedState { path: PathBuf, device: u64, inode: u64, nonce: String }
impl std::fmt::Debug for OwnedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_tuple("OwnedState").field(&self.path).finish() }
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, NativeServiceError> {
    serde_json::to_vec(value).map_err(|_| NativeServiceError::Refused("cannot encode private service record"))
}
fn private_path(path: &Path, directory: bool) -> Result<(), NativeServiceError> {
    if !path.is_absolute() { return Err(NativeServiceError::Refused("private service path must be absolute")); }
    for parent in path.ancestors() {
        if fs::symlink_metadata(parent)?.file_type().is_symlink() { return Err(NativeServiceError::Refused("private service path traverses a symlink")); }
    }
    let metadata = fs::metadata(path)?;
    if (directory && !metadata.is_dir()) || (!directory && !metadata.is_file())
        || metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
        return Err(NativeServiceError::Refused("service records must be private and owned by the current user"));
    }
    Ok(())
}
fn private_read<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, NativeServiceError> {
    private_path(path, false)?;
    let file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 || metadata.len() > PRIVATE_BYTES || metadata.nlink() != 1 { return Err(NativeServiceError::Refused("private service record extent or link count differs")); }
    let mut bytes = Vec::new();
    file.take(PRIVATE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > PRIVATE_BYTES { return Err(NativeServiceError::Refused("private service record exceeds its bound")); }
    serde_json::from_slice(&bytes).map_err(|_| NativeServiceError::Refused("invalid private service record"))
}
fn private_write<T: Serialize>(path: &Path, value: &T) -> Result<(), NativeServiceError> {
    let mut file = OpenOptions::new().create_new(true).write(true).mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
    file.write_all(&json(value)?)?; file.sync_all()?;
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), NativeServiceError> {
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    private_path(path, true)
}
fn secret() -> String { format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple()) }
fn deadline_left(deadline: Instant) -> Result<(), NativeServiceError> {
    if Instant::now() >= deadline { Err(NativeServiceError::Refused("original service deadline expired")) } else { Ok(()) }
}
impl OwnedState {
    fn verify(&self) -> Result<(), NativeServiceError> {
        private_path(&self.path, true)?;
        let metadata = fs::metadata(&self.path)?;
        let identity: DisposableIdentity = private_read(&self.path.join("disposable.json"))?;
        if metadata.dev() != self.device || metadata.ino() != self.inode || identity.version != 1 || identity.nonce != self.nonce {
            return Err(NativeServiceError::Refused("created disposable directory identity changed"));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct OwnedIngress { intent: LaunchIntent, unit: Option<OwnedUnit>, released: bool, ready: bool }
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerPayload { version: u32, state: PathBuf, nonce: String, index: usize, initial_schema: InitialSchema }
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InitialSchema { Current, PreEpoch }

/// Retained ownership of one newly constructed service. Not cloneable and no Drop cleanup claims.
#[derive(Debug)]
pub struct NativeCanonicalService {
    state: OwnedState,
    initial_schema: InitialSchema,
    initialized: bool,
    helper: PathBuf,
    bind: SocketAddr,
    intent: LaunchIntent,
    manager: Option<HostManager>,
    unit: Option<OwnedUnit>,
    launch_dispatched: bool,
    ready: Option<SchemaReady>,
    connections: Vec<CanonicalStore>,
    maintenances: Vec<SchemaMaintenance>,
    ingresses: Vec<OwnedIngress>,
    process_drained: Option<UnitObservation>,
    closing: bool,
    stopped: bool,
    panic_settlement: Option<NativeServiceError>,
}
impl NativeCanonicalService {
    /// Create a fresh private disposable state and copy a validated immutable policy artifact.
    /// This does not start a process, install dependencies or change existing state.
    pub fn new(spec: NativeServiceSpec) -> Result<Self, NativeServiceError> {
        Self::new_with_schema(spec, InitialSchema::Current)
    }
    /// Construct a newly owned predecessor fixture only, for explicit adjacent migration.
    /// The predecessor never grants ordinary application readiness.
    pub fn new_pre_epoch_fixture(spec: NativeServiceSpec) -> Result<Self, NativeServiceError> {
        Self::new_with_schema(spec, InitialSchema::PreEpoch)
    }
    fn new_with_schema(spec: NativeServiceSpec, initial_schema: InitialSchema) -> Result<Self, NativeServiceError> {
        if !spec.helper.is_absolute() || !spec.helper.is_file() { return Err(NativeServiceError::Refused("explicit native application helper is required")); }
        let root = spec.state_parent.canonicalize()?;
        let temporary = tempfile::Builder::new().prefix("pse-canonical-native-").tempdir_in(root)?;
        let path = temporary.path().canonicalize()?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        let module = StudyModuleReceipt::read(&spec.module_source_state)?;
        private_write(&path.join("original-module-receipt.json"), &module)?;
        let module = module.copy_into_private(&path, &path.join("module"))?;
        for directory in ["tmp", "cache", "home"] { private_directory(&path.join(directory))?; }
        let credentials = Credentials { username: ROOT_USER.into(), password: secret(), selection_username: VIEWER_USER.into(), selection_password: secret() };
        private_write(&path.join("credentials.json"), &credentials)?;
        // Pin a fresh loopback port. The release-to-exec race refuses on collision;
        // actual kernel socket ownership is established before authenticated RPC.
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let bind = listener.local_addr()?;
        let nonce = uuid::Uuid::new_v4().to_string();
        let payload = StartupPayload { version: 1, state: path.clone(), generation: spec.server.generation().to_path_buf(), recipe: spec.server.recipe().into(), bind, nonce: nonce.clone() };
        private_write(&path.join("startup.json"), &payload)?;
        let config = Configuration {
            identity: ServiceIdentity { profile_version: pse_model::document::Version, endpoint: format!("grpc://{bind}"), namespace: "native_fixture".into(), database: "canonical".into(), service_state: None, receiver_generation: None },
            connection: CanonicalConnectionSettings { schema_interpretation: crate::generated::surreal::INTERPRETATION.into(), max_message_bytes: MESSAGE_BYTES, credentials_file: path.join("credentials.json").to_string_lossy().into_owned() },
            payload_incarnation: uuid::Uuid::new_v4().to_string(), accepting_writes: true, study_module: module,
        };
        private_write(&path.join("config.json"), &config)?;
        private_write(&path.join("disposable.json"), &DisposableIdentity { version: 1, nonce: nonce.clone() })?;
        File::open(&path)?.sync_all()?;
        let metadata = fs::metadata(&path)?;
        let state = OwnedState { path: temporary.keep(), device: metadata.dev(), inode: metadata.ino(), nonce };
        drop(listener);
        Ok(Self { state, initial_schema, initialized: false, helper: spec.helper, bind, intent: LaunchIntent::fresh()?, manager: None, unit: None, launch_dispatched: false, ready: None, connections: Vec::new(), maintenances: Vec::new(), ingresses: Vec::new(), process_drained: None, closing: false, stopped: false, panic_settlement: None })
    }
    /// Cleanup and diagnostic-write failures retained if a body panic was resumed.
    pub fn panic_settlement_failure(&self) -> Option<&NativeServiceError> { self.panic_settlement.as_ref() }
    /// State created by this owner, retained across failures for diagnosis.
    pub fn state(&self) -> &Path { &self.state.path }
    /// Original manager intent; a unit name alone grants no stop authority.
    pub fn intent(&self) -> &LaunchIntent { &self.intent }
    /// Application-ready canonical operations, established by actual D construction.
    pub fn ready(&self) -> Result<&SchemaReady, NativeServiceError> { if self.closing { return Err(NativeServiceError::Refused("original native service admission is closed")); } self.ready.as_ref().ok_or(NativeServiceError::Refused("native canonical schema readiness has not been established")) }
    fn service_spec(&self) -> ServiceSpec {
        ServiceSpec { intent: self.intent.clone(), executable: self.helper.clone(), arguments: vec!["exec".into(), self.state.path.join("startup.json").to_string_lossy().into_owned()], environment: vec!["PATH=/usr/bin:/bin".into(), "LANG=C.UTF-8".into()] }
    }
    /// Launch the original owned native process once, then establish protocol and schema readiness.
    pub async fn launch(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        self.state.verify()?; deadline_left(deadline)?;
        if self.launch_dispatched { return Err(NativeServiceError::Refused("original service launch already dispatched; reconcile, do not replay")); }
        let manager = HostManager::connect_user(None).await?;
        let enclosing = manager.current_unit().await?;
        if !manager.native_workload(&enclosing).await? { return Err(NativeServiceError::Refused("native canonical fixture requires authenticated native workload containment")); }
        self.manager = Some(manager.clone()); self.launch_dispatched = true;
        let parent = manager.borrow(enclosing.unit);
        self.unit = Some(manager.start_service_bound(&self.service_spec(), &parent, &ResourceControls::default(), deadline).await?);
        self.wait_owned_listener(deadline).await?;
        crate::transport::operation_scope(deadline, self.provision_viewer(deadline)).await?;
        let options = CanonicalOptions::from_state(&self.state.path)?;
        crate::transport::operation_scope(deadline, async {
            match self.initial_schema {
                InitialSchema::Current => {
                    CanonicalStore::initialize_database(&options).await?;
                    let ready = CanonicalStore::connect_ready(&options).await?;
                    self.connections.push(ready.store().clone()); self.ready = Some(ready);
                }
                InitialSchema::PreEpoch => CanonicalStore::install_pre_epoch_fixture(&options).await?,
            }
            self.initialized = true;
            Ok::<(),CanonicalError>(())
        }).await?;
        Ok(())
    }
    async fn wait_owned_listener(&self, deadline: Instant) -> Result<(), NativeServiceError> {
        let unit = self.unit.as_ref().ok_or(NativeServiceError::Refused("original service capability is unavailable"))?;
        loop {
            deadline_left(deadline)?;
            let observed = unit.observe().await?;
            if observed.main_pid == 0 || matches!(observed.active_state.as_str(), "failed" | "inactive") { return Err(NativeServiceError::Refused("owned canonical service exited before listener readiness")); }
            if let Ok(claim) = private_read::<StartupClaim>(&self.state.path.join("started.json")) {
                if claim.nonce != self.state.nonce || claim.pid != observed.main_pid || claim.invocation != hex_bytes(&observed.invocation) { return Err(NativeServiceError::Refused("native startup claim differs from original unit invocation")); }
                if owns_listener(observed.main_pid, self.bind)? {
                    let payload: StartupPayload = private_read(&self.state.path.join("startup.json"))?;
                    let exact = server::reopen_generation(&payload.generation, &payload.recipe)?;
                    if fs::read_link(format!("/proc/{}/exe", observed.main_pid))? != exact.executable() { return Err(NativeServiceError::Refused("owned listener is not the exact prepared server executable")); }
                    let after = unit.observe().await?;
                    if after.main_pid == observed.main_pid && after.invocation == observed.invocation { return Ok(()); }
                    return Err(NativeServiceError::Refused("owned listener changed during observation"));
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    async fn provision_viewer(&self, deadline: Instant) -> Result<(), NativeServiceError> {
        deadline_left(deadline)?;
        let credentials: Credentials = private_read(&self.state.path.join("credentials.json"))?;
        let client = Client::connect(self.bind, MESSAGE_BYTES).await.map_err(CanonicalError::from)?;
        let context = || RequestContext::control(deadline);
        let provisioned = async {
            client.signin(Root { username: credentials.username, password: credentials.password }).request_context(context()).await?;
            let response = client.query("DEFINE USER OVERWRITE `pse-selection` ON ROOT PASSWORD $selection_password ROLES VIEWER;")
                .bind(("selection_password",credentials.selection_password)).request_context(context()).await?;
            complete_response(response)?;
            Ok::<(),CanonicalError>(())
        }.await;
        let drained = client.disconnect().await.map_err(CanonicalError::from);
        match (provisioned,drained) {
            (Ok(()),Ok(())) => Ok(()),
            (Err(error),Ok(())) | (Ok(()),Err(error)) => Err(error.into()),
            (Err(error),Err(cleanup)) => Err(NativeServiceError::Settlement { state: self.state.path.clone(), failures: vec![error.into(),cleanup.into()] }),
        }
    }
    /// Start a separate native control ingress with its own live canonical connection.
    /// Its exact original owned unit is retained before the first dispatch await.
    pub async fn start_ingress(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        if self.closing { return Err(NativeServiceError::Refused("original native service admission is closed")); }
        if !self.initialized { return Err(NativeServiceError::Refused("original native fixture schema installation did not complete")); }
        if matches!(self.initial_schema, InitialSchema::Current) { self.ready()?; }
        self.state.verify()?; deadline_left(deadline)?;
        let manager = self.manager.as_ref().ok_or(NativeServiceError::Refused("service manager is unavailable"))?.clone();
        let index = self.ingresses.len();
        let payload = WorkerPayload { version: 1, state: self.state.path.clone(), nonce: self.state.nonce.clone(), index, initial_schema: self.initial_schema };
        let path = self.state.path.join(format!("worker-{index}.json"));
        private_write(&path, &payload)?;
        let intent = LaunchIntent::fresh()?;
        let spec = ServiceSpec { intent: intent.clone(), executable: self.helper.clone(), arguments: vec!["worker-exec".into(), path.to_string_lossy().into_owned()], environment: vec!["PATH=/usr/bin:/bin".into(), "LANG=C.UTF-8".into()] };
        let parent = manager.borrow(manager.current_unit().await?.unit);
        self.ingresses.push(OwnedIngress { intent, unit: None, released: false, ready: false });
        self.ingresses[index].unit = Some(manager.start_service_bound(&spec, &parent, &ResourceControls::default(), deadline).await?);
        let unit = self.ingresses[index].unit.as_ref().ok_or(NativeServiceError::Refused("original ingress capability unavailable"))?;
        loop {
            deadline_left(deadline)?;
            let observed = unit.observe().await?;
            if observed.main_pid == 0 || matches!(observed.active_state.as_str(), "failed" | "inactive") { return Err(NativeServiceError::Refused("owned ingress exited before readiness")); }
            if let Ok(claim) = private_read::<StartupClaim>(&self.state.path.join(format!("worker-{index}.ready.json"))) {
                if claim.nonce != self.state.nonce || claim.pid != observed.main_pid || claim.invocation != hex_bytes(&observed.invocation) { return Err(NativeServiceError::Refused("original ingress invocation differs")); }
                if owns_unix_listener(observed.main_pid, &self.state.path.join(format!("worker-{index}.sock")))? {
                    let after = unit.observe().await?;
                    if after.main_pid == observed.main_pid && after.invocation == observed.invocation { self.ingresses[index].ready = true; return Ok(()); }
                    return Err(NativeServiceError::Refused("owned ingress changed during observation"));
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    /// Exercise the last original ingress through its private native control socket.
    /// A successful reply requires a canonical read over that worker's owned connection.
    pub async fn probe_ingress(&self, deadline: Instant) -> Result<String, NativeServiceError> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let index = self.ingresses.len().checked_sub(1).ok_or(NativeServiceError::Refused("no native ingress was started"))?;
        tokio::time::timeout_at(deadline.into(), async {
            let mut socket = tokio::net::UnixStream::connect(self.state.path.join(format!("worker-{index}.sock"))).await?;
            socket.write_all(&[0]).await?;
            let mut answer = Vec::new(); socket.take(256).read_to_end(&mut answer).await?;
            String::from_utf8(answer).map_err(|_| NativeServiceError::Refused("invalid native ingress reply"))
        }).await.map_err(|_| NativeServiceError::Refused("original ingress probe deadline expired"))?
    }
    async fn stop_ingresses(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        stop_ingress_owners(self.manager.as_ref(), &self.state, &mut self.ingresses, deadline).await
    }
    /// Close and physically join the actual native control ingress owners.
    /// This future is the B/C premise supplied to schema maintenance; the database
    /// remains live and D separately drains each supplied canonical connection.
    pub async fn stop_ingress_and_join(&mut self, deadline: Instant) -> Result<(), CanonicalError> {
        self.stop_ingresses(deadline).await.map_err(|error| CanonicalError::MigrationDrain { failures: vec![canonical_drain_failure(error)] })
    }
    /// Execute fixture work and settle the original physical owners even if the body panics.
    /// A failed settlement retains this object's private state for explicit diagnosis/retry.
    pub async fn run<T, F>(&mut self, deadline: Instant, body: F) -> Result<T, NativeServiceError>
    where F: for<'a> FnOnce(&'a mut Self) -> std::pin::Pin<Box<dyn Future<Output = Result<T, NativeServiceError>> + Send + 'a>> {
        let outcome = std::panic::AssertUnwindSafe(crate::transport::operation_scope(deadline, async { body(self).await })).catch_unwind().await;
        let settlement = self.shutdown(deadline).await;
        match (outcome, settlement) {
            (Ok(Ok(value)), Ok(())) => Ok(value),
            (Ok(Err(error)), Ok(())) => Err(error),
            (Ok(Ok(_)), Err(error)) => Err(error),
            (Ok(Err(error)), Err(cleanup)) => Err(NativeServiceError::Settlement { state: self.state.path.clone(), failures: vec![error, cleanup] }),
            (Err(payload), cleanup) => {
                if let Err(error) = cleanup {
                    let record = private_write(&self.state.path.join("settlement-failed.json"), &error.to_string());
                    let mut failures = vec![error];
                    if let Err(error) = record { failures.push(error); }
                    let details = failures.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
                    if let Err(error) = writeln!(std::io::stderr().lock(), "native canonical fixture retained at {} after panic cleanup failures: {details}", self.state.path.display()) { failures.push(error.into()); }
                    self.panic_settlement = Some(NativeServiceError::Settlement { state: self.state.path.clone(), failures });
                }
                std::panic::resume_unwind(payload)
            }
        }
    }
    /// Open explicit schema maintenance and retain its same original transport owner.
    pub async fn maintenance(&mut self) -> Result<SchemaMaintenance, NativeServiceError> {
        if self.closing { return Err(NativeServiceError::Refused("original native service admission is closed")); }
        if !self.initialized { return Err(NativeServiceError::Refused("original native fixture installation did not complete")); }
        let maintenance = SchemaMaintenance::connect(&CanonicalOptions::from_state(&self.state.path)?).await?;
        self.maintenances.push(maintenance.clone());
        Ok(maintenance)
    }
    /// Retain an actual successor connection from this same fixture for terminal drain.
    /// This grants no readiness or migration authority.
    pub fn retain_connection(&mut self, store: CanonicalStore) -> Result<(), NativeServiceError> {
        if store.deployment_state() != self.state.path { return Err(NativeServiceError::Refused("connection belongs to a different deployment")); }
        self.connections.push(store);
        Ok(())
    }
    /// Open an additional independently owned D connection and retain it for terminal drain.
    pub async fn connect(&mut self) -> Result<SchemaReady, NativeServiceError> {
        if self.closing { return Err(NativeServiceError::Refused("original native service admission is closed")); }
        if !self.initialized { return Err(NativeServiceError::Refused("original native fixture installation did not complete")); }
        let ready = CanonicalStore::connect_ready(&CanonicalOptions::from_state(&self.state.path)?).await?;
        self.connections.push(ready.store().clone());
        self.ready = Some(ready.clone());
        Ok(ready)
    }
    /// Stop and physically join the original service process without needing database connectivity.
    pub async fn stop_process(&mut self, deadline: Instant) -> Result<UnitObservation, NativeServiceError> {
        self.closing = true;
        stop_original_process(self.manager.as_ref(), &self.intent, &mut self.unit, self.launch_dispatched, &mut self.process_drained, deadline).await?
            .ok_or(NativeServiceError::Refused("service was never launched"))
    }
    /// Initiate all independent client, ingress and original database settlements
    /// concurrently under one deadline, retaining every failure.
    pub async fn shutdown(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        self.closing = true;
        let (drains, maintenance_drains, ingress_drain, process_drain) = tokio::join!(
            futures_util::future::join_all(self.connections.iter().map(|store| store.drain_owned_connection_until(deadline.into()))),
            futures_util::future::join_all(self.maintenances.iter().map(|maintenance| crate::transport::operation_scope(deadline, maintenance.disconnect()))),
            stop_ingress_owners(self.manager.as_ref(), &self.state, &mut self.ingresses, deadline),
            stop_original_process(self.manager.as_ref(), &self.intent, &mut self.unit, self.launch_dispatched, &mut self.process_drained, deadline)
        );
        let mut failures: Vec<_> = drains.into_iter().chain(maintenance_drains).filter_map(Result::err).map(NativeServiceError::from).collect();
        if let Err(error) = ingress_drain { failures.push(error); }
        if let Err(error) = process_drain { failures.push(error); }
        if !failures.is_empty() { return Err(NativeServiceError::Settlement { state: self.state.path.clone(), failures }); }
        self.stopped = true;
        Ok(())
    }
    /// Delete only this newly created disposable state after explicit successful settlement.
    pub fn dispose(self) -> Result<(), NativeServiceError> {
        if !self.stopped { return Err(NativeServiceError::Refused("explicit original physical settlement is required before disposal")); }
        self.state.verify()?;
        fs::remove_dir_all(&self.state.path)?;
        Ok(())
    }
}
fn hex_bytes(bytes: &[u8]) -> String { bytes.iter().map(|byte|format!("{byte:02x}")).collect() }
fn owns_listener(pid: u32, bind: SocketAddr) -> Result<bool, NativeServiceError> {
    let SocketAddr::V4(address) = bind else { return Err(NativeServiceError::Refused("only native IPv4 loopback listener fixtures are supported")); };
    let selected = format!("{:08X}:{:04X}", u32::from_le_bytes(address.ip().octets()), address.port());
    let table = fs::read_to_string(format!("/proc/{pid}/net/tcp"))?;
    let inodes: BTreeSet<_> = table.lines().skip(1).filter_map(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        (fields.get(1).copied()==Some(selected.as_str()) && fields.get(3).copied()==Some("0A")).then(|| fields.get(9).copied().map(str::to_owned)).flatten()
    }).collect();
    if inodes.is_empty() { return Ok(false); }
    for entry in fs::read_dir(format!("/proc/{pid}/fd"))? {
        let entry = entry?;
        if let Ok(target) = fs::read_link(entry.path()) {
            if let Some(inode) = target.to_str().and_then(|target|target.strip_prefix("socket:[")).and_then(|target|target.strip_suffix(']')) {
                if inodes.contains(inode) { return Ok(true); }
            }
        }
    }
    Ok(false)
}

/// Native exec helper entry. Only a bounded private file supplies credentials;
/// neither manager argv nor manager environment receives their bytes.
pub fn exec_startup(path: &Path) -> Result<(), NativeServiceError> {
    let payload: StartupPayload = private_read(path)?;
    private_path(&payload.state,true)?;
    if payload.version!=1 || path!=payload.state.join("startup.json") || payload.bind.ip()!=Ipv4Addr::LOCALHOST || payload.bind.port()==0 {
        return Err(NativeServiceError::Refused("invalid native startup coordinates"));
    }
    let disposable: DisposableIdentity = private_read(&payload.state.join("disposable.json"))?;
    if disposable.version!=1 || disposable.nonce!=payload.nonce { return Err(NativeServiceError::Refused("startup disposable identity differs")); }
    let server = server::reopen_generation(&payload.generation,&payload.recipe)?;
    let credentials: Credentials = private_read(&payload.state.join("credentials.json"))?;
    let invocation = std::env::var("INVOCATION_ID").map_err(|_|NativeServiceError::Refused("native startup requires its original systemd invocation"))?;
    if invocation.len()!=32 || !invocation.bytes().all(|byte|byte.is_ascii_hexdigit()) { return Err(NativeServiceError::Refused("systemd invocation identity is invalid")); }
    private_write(&payload.state.join("started.json"),&StartupClaim { nonce:payload.nonce,pid:std::process::id(),invocation })?;
    let log = OpenOptions::new().create_new(true).append(true).mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(payload.state.join("server.log"))?;
    let datastore = format!("mem://{}?aol=sync&sync=every&versioned=false&snapshot=60s",payload.state.join("database").display());
    let error = Command::new(server.executable()).args(["start","--no-banner","--no-defaults","--deny-guests","--deny-net","--log","warn","--query-timeout=90s","--transaction-timeout=90s","--bind"])
        .arg(payload.bind.to_string()).arg("--temporary-directory").arg(payload.state.join("tmp")).arg(datastore)
        .env_clear().env("PATH","/usr/bin:/bin").env("LANG","C.UTF-8").env("HOME",payload.state.join("home")).env("XDG_CACHE_HOME",payload.state.join("cache")).env("TMPDIR",payload.state.join("tmp"))
        .env("SURREAL_USER",credentials.username).env("SURREAL_PASS",credentials.password)
        .stdin(Stdio::null()).stdout(Stdio::from(log.try_clone()?)).stderr(Stdio::from(log)).exec();
    Err(error.into())
}

fn process_owns_inode(pid: u32, inode: &str) -> Result<bool, NativeServiceError> {
    let expected = format!("socket:[{inode}]");
    for entry in fs::read_dir(format!("/proc/{pid}/fd"))? {
        if fs::read_link(entry?.path()).ok().is_some_and(|path|path.as_os_str()==expected.as_str()) { return Ok(true); }
    }
    Ok(false)
}
fn owns_unix_listener(pid: u32, path: &Path) -> Result<bool, NativeServiceError> {
    let table = fs::read_to_string(format!("/proc/{pid}/net/unix"))?;
    for line in table.lines().skip(1) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.get(7).is_some_and(|field|Path::new(field)==path) && fields.get(3).copied()==Some("00010000") {
            if let Some(inode) = fields.get(6) { return process_owns_inode(pid, inode); }
        }
    }
    Ok(false)
}

/// Preserve the original systemd PID across a clean-environment worker exec.
/// User-manager environment inheritance never reaches the actual connection/control body.
pub fn exec_worker(path: &Path) -> Result<(), NativeServiceError> {
    let payload: WorkerPayload = private_read(path)?;
    private_path(&payload.state, true)?;
    let identity: DisposableIdentity = private_read(&payload.state.join("disposable.json"))?;
    if payload.version != 1 || identity.version != 1 || identity.nonce != payload.nonce || path != payload.state.join(format!("worker-{}.json", payload.index)) {
        return Err(NativeServiceError::Refused("invalid original worker coordinates"));
    }
    let invocation = std::env::var("INVOCATION_ID").map_err(|_| NativeServiceError::Refused("native worker requires its original systemd invocation"))?;
    if invocation.len()!=32 || !invocation.bytes().all(|byte|byte.is_ascii_hexdigit()) { return Err(NativeServiceError::Refused("invalid native worker invocation")); }
    let error = Command::new(std::env::current_exe()?).arg("worker").arg(path)
        .env_clear().env("PATH","/usr/bin:/bin").env("LANG","C.UTF-8")
        .env("HOME",payload.state.join("home")).env("XDG_CACHE_HOME",payload.state.join("cache")).env("TMPDIR",payload.state.join("tmp"))
        .env("INVOCATION_ID",invocation).exec();
    Err(error.into())
}

/// Application worker helper: one private control ingress and one real D connection.
/// TERM closes ingress, physically drains the original connection and only then exits.
pub async fn run_worker(path: &Path) -> Result<(), NativeServiceError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let payload: WorkerPayload = private_read(path)?;
    private_path(&payload.state, true)?;
    let identity: DisposableIdentity = private_read(&payload.state.join("disposable.json"))?;
    if payload.version != 1 || identity.version != 1 || identity.nonce != payload.nonce || path != payload.state.join(format!("worker-{}.json", payload.index)) {
        return Err(NativeServiceError::Refused("invalid original worker coordinates"));
    }
    let invocation = std::env::var("INVOCATION_ID").map_err(|_|NativeServiceError::Refused("native worker requires its original systemd invocation"))?;
    if invocation.len()!=32 || !invocation.bytes().all(|byte|byte.is_ascii_hexdigit()) { return Err(NativeServiceError::Refused("invalid native worker invocation")); }
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let options = CanonicalOptions::from_state(&payload.state)?;
    let connection = match payload.initial_schema {
        InitialSchema::Current => WorkerConnection::Current(CanonicalStore::connect_ready(&options).await?),
        InitialSchema::PreEpoch => {
            let connection = SchemaMaintenance::connect(&options).await?;
            let inspected = connection.inspect().await;
            if let Err(error) = inspected {
                let drained = connection.disconnect().await;
                return match drained { Ok(()) => Err(error.into()), Err(cleanup) => Err(NativeServiceError::Settlement { state:payload.state, failures:vec![error.into(),cleanup.into()] }) };
            }
            WorkerConnection::PreEpoch(connection)
        }
    };
    let socket = payload.state.join(format!("worker-{}.sock", payload.index));
    let listener = match tokio::net::UnixListener::bind(&socket) {
        Ok(listener) => listener,
        Err(error) => {
            let drained = connection.disconnect().await;
            return match drained { Ok(()) => Err(error.into()), Err(cleanup) => Err(NativeServiceError::Settlement { state: payload.state, failures: vec![error.into(),cleanup.into()] }) };
        }
    };
    let permissions = fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).map_err(NativeServiceError::from);
    let claim = StartupClaim { nonce: payload.nonce, pid: std::process::id(), invocation };
    let claimed = permissions.and_then(|()| private_write(&payload.state.join(format!("worker-{}.ready.json", payload.index)), &claim));
    let run = async {
        claimed?;
        loop {
            tokio::select! {
                biased;
                _ = term.recv() => return Ok::<(),NativeServiceError>(()),
                accepted = listener.accept() => {
                    let (mut stream, _) = accepted?;
                    let request = async {
                        if stream.read_u8().await? != 0 { return Err(NativeServiceError::Refused("unknown native fixture ingress request")); }
                        let epoch = connection.probe().await?;
                        stream.write_all(epoch.as_bytes()).await?;
                        stream.shutdown().await?;
                        Ok::<(),NativeServiceError>(())
                    };
                    tokio::select! {
                        biased;
                        _ = term.recv() => return Ok(()),
                        result = tokio::time::timeout(Duration::from_secs(30), request) => result.map_err(|_|NativeServiceError::Refused("native fixture control request deadline expired"))??,
                    }
                }
            }
        }
    }.await;
    drop(listener);
    let drain = connection.disconnect().await.map_err(NativeServiceError::from);
    if drain.is_ok() { private_write(&payload.state.join(format!("worker-{}.drained.json", payload.index)), &claim)?; }
    match (run,drain) {
        (Ok(()),Ok(()))=>Ok(()),
        (Err(error),Ok(()))|(Ok(()),Err(error))=>Err(error),
        (Err(error),Err(cleanup))=>Err(NativeServiceError::Settlement { state:payload.state,failures:vec![error,cleanup] }),
    }
}

fn canonical_drain_failure(error: NativeServiceError) -> CanonicalError {
    match error {
        NativeServiceError::Canonical(error) => error,
        NativeServiceError::Settlement { failures, .. } => CanonicalError::MigrationDrain { failures: failures.into_iter().map(canonical_drain_failure).collect() },
        error => CanonicalError::Configuration(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn private_state() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(),fs::Permissions::from_mode(0o700)).unwrap();
        directory
    }
    #[test]
    fn native_service_private_payload_refuses_symlinks_hardlinks_and_shared_modes() {
        let state = private_state();
        let path = state.path().join("private.json");
        private_write(&path,&DisposableIdentity {version:1,nonce:"identity".into()}).unwrap();
        assert_eq!(private_read::<DisposableIdentity>(&path).unwrap().nonce,"identity");
        assert!(private_write(&path,&"overwrite").is_err());
        let alias = state.path().join("alias.json");
        std::os::unix::fs::symlink(&path,&alias).unwrap();
        assert!(private_read::<DisposableIdentity>(&alias).is_err());
        fs::remove_file(alias).unwrap();
        fs::hard_link(&path,state.path().join("hardlink.json")).unwrap();
        assert!(private_read::<DisposableIdentity>(&path).is_err());
        fs::remove_file(state.path().join("hardlink.json")).unwrap();
        fs::set_permissions(&path,fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_read::<DisposableIdentity>(&path).is_err());
    }
    #[test]
    fn native_service_kernel_listener_proof_refuses_foreign_owner_and_unbound_endpoint() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST,0)).unwrap();
        let bind = listener.local_addr().unwrap();
        assert!(owns_listener(std::process::id(),bind).unwrap());
        assert!(!owns_listener(std::process::id(),SocketAddr::from((Ipv4Addr::LOCALHOST,0))).unwrap());
        let mut child = Command::new("/usr/bin/sleep").arg("5").spawn().unwrap();
        let observed = owns_listener(child.id(),bind);
        let killed = child.kill(); let waited = child.wait();
        killed.unwrap(); waited.unwrap();
        assert!(!observed.unwrap());
    }
    #[test]
    fn native_service_replaced_disposable_identity_cannot_authorize_deletion() {
        let state = private_state(); let path=state.path().to_path_buf();
        private_write(&path.join("disposable.json"),&DisposableIdentity{version:1,nonce:"original".into()}).unwrap();
        let metadata=fs::metadata(&path).unwrap();
        let owned=OwnedState{path:path.clone(),device:metadata.dev(),inode:metadata.ino(),nonce:"original".into()};
        owned.verify().unwrap();
        fs::remove_file(path.join("disposable.json")).unwrap();
        private_write(&path.join("disposable.json"),&DisposableIdentity{version:1,nonce:"replacement".into()}).unwrap();
        assert!(owned.verify().is_err());
    }
}

enum WorkerConnection { Current(SchemaReady), PreEpoch(SchemaMaintenance) }
impl WorkerConnection {
    async fn probe(&self) -> Result<String, CanonicalError> {
        match self {
            Self::Current(ready) => {
                ready.store().canonical_run("native-fixture-probe").await?;
                Ok(ready.fence().epoch().to_owned())
            }
            Self::PreEpoch(maintenance) => {
                maintenance.inspect().await?;
                Ok("maintenance".into())
            }
        }
    }
    async fn disconnect(&self) -> Result<(), CanonicalError> {
        match self { Self::Current(ready) => ready.store().drain_owned_connection_until(crate::transport::original_deadline(Duration::from_secs(30))).await, Self::PreEpoch(maintenance) => maintenance.disconnect().await }
    }
}

async fn stop_ingress_owners(manager: Option<&HostManager>, state: &OwnedState, ingresses: &mut [OwnedIngress], deadline: Instant) -> Result<(), NativeServiceError> {
        let results = futures_util::future::join_all(ingresses.iter_mut().enumerate().map(|(index, ingress)| {
            async move {
                if ingress.released { return Ok(()); }
                if ingress.unit.is_none() {
                    ingress.unit = manager.ok_or(NativeServiceError::Refused("original ingress manager unavailable"))?.reconcile_if_present(&ingress.intent, deadline).await?;
                }
                let unit = ingress.unit.as_mut().ok_or(NativeServiceError::Refused("original ingress launch remains uncertain"))?;
                let original = unit.observe().await?;
                unit.stop_and_drain(deadline).await?;
                if ingress.ready {
                    let drained: StartupClaim = private_read(&state.path.join(format!("worker-{index}.drained.json")))?;
                    let started: StartupClaim = private_read(&state.path.join(format!("worker-{index}.ready.json")))?;
                    if drained != started || drained.nonce != state.nonce || drained.invocation != hex_bytes(&original.invocation) { return Err(NativeServiceError::Refused("original ingress connection drain was not established")); }
                }
                unit.release_reference(deadline).await?;
                ingress.released = true;
                Ok::<(),NativeServiceError>(())
            }
        })).await;
        let failures: Vec<_> = results.into_iter().filter_map(Result::err).collect();
        if failures.is_empty() { Ok(()) } else { Err(NativeServiceError::Settlement { state: state.path.clone(), failures }) }
    }

async fn stop_original_process(manager: Option<&HostManager>, intent: &LaunchIntent, unit: &mut Option<OwnedUnit>, dispatched: bool, drained: &mut Option<UnitObservation>, deadline: Instant) -> Result<Option<UnitObservation>, NativeServiceError> {
        if !dispatched { return Ok(None); }
        if let Some(observed) = drained { return Ok(Some(observed.clone())); }
        if unit.is_none() && dispatched {
            let manager = manager.ok_or(NativeServiceError::Refused("original manager is unavailable"))?;
            *unit = manager.reconcile_if_present(intent, deadline).await?;
            if unit.is_none() { return Err(NativeServiceError::Refused("original launch remains uncertain; no physical release was established")); }
        }
        let original = unit.as_mut().ok_or(NativeServiceError::Refused("service was never launched"))?;
        let observed = original.stop_and_drain(deadline).await?;
        original.release_reference(deadline).await?;
        *drained = Some(observed.clone());
        Ok(Some(observed))
    }
