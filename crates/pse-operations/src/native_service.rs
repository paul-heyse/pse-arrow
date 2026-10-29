// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Application-owned disposable canonical service construction and physical settlement.
//!
//! This module owns only directories it creates. Ordinary host selection remains
//! database-free, and dropping a value never establishes process or test success.
use crate::{
    canonical::{
        CanonicalError, CanonicalOptions, CanonicalStore, MESSAGE_BYTES,
        migration::{SchemaMaintenance, SchemaMigrationReceipt, SchemaReady},
        study_module::StudyModuleReceipt,
    },
    deployment::{CanonicalConnectionSettings, ServiceIdentity},
    transport::{
        client::{Client, RequestContext, Root},
        complete_response,
    },
};
use futures_util::FutureExt;
use pse_host::{
    HostError, HostManager, LaunchIntent, OwnedUnit, ResourceControls, ServiceSpec,
    UnitObservation,
    server::{self, PreparedServer},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use surrealdb_types::{Number, SurrealValue, ToSql, Value};

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
    /// Original privileged-bootstrap failure with SQL and credential-safe rendering.
    #[error("native canonical viewer bootstrap: {0}")]
    Bootstrap(#[source] ViewerBootstrapFailure),
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

/// Full original bootstrap failure remains private; automatic diagnostics expose
/// classification only, since upstream errors may reflect literal credential SQL.
pub struct ViewerBootstrapFailure {
    original: CanonicalError,
}
impl ViewerBootstrapFailure {
    /// Existing canonical infrastructure classification, without rendered query text.
    pub fn is_infrastructure(&self) -> bool {
        self.original.is_infrastructure()
    }
    /// Existing original-response effect classification; no replay is inferred.
    pub fn effect_is_unknown(&self) -> bool {
        self.original.effect_is_unknown()
    }
    /// Original native failure family, when the bootstrap failed in its driver.
    pub fn native_kind(&self) -> Option<&'static str> {
        match &self.original {
            CanonicalError::Driver(error) => Some(error.kind_str()),
            _ => None,
        }
    }
    /// Original native wire code, decoded through the pinned lossless value codec.
    pub fn native_code(&self) -> Option<i64> {
        let CanonicalError::Driver(error) = &self.original else {
            return None;
        };
        let Value::Object(value) = error.clone().into_value() else {
            return None;
        };
        match value.get("code") {
            Some(Value::Number(Number::Int(code))) => Some(*code),
            _ => None,
        }
    }
    /// Typed query finality classification, with no reflected server message.
    pub fn query_failure(&self) -> Option<surrealdb_types::QueryError> {
        match &self.original {
            CanonicalError::Driver(error) => error.query_details().cloned(),
            _ => None,
        }
    }
}
impl std::fmt::Debug for ViewerBootstrapFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerBootstrapFailure")
            .field("native_kind", &self.native_kind())
            .field("native_code", &self.native_code())
            .field("infrastructure", &self.is_infrastructure())
            .field("effect_unknown", &self.effect_is_unknown())
            .finish()
    }
}
impl std::fmt::Display for ViewerBootstrapFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "privileged account request failed (class {:?}, code {:?}, effect unknown {})",
            self.native_kind(),
            self.native_code(),
            self.effect_is_unknown()
        )
    }
}
// Deliberately no Error::source: the original SQL-bearing cause is private.
impl std::error::Error for ViewerBootstrapFailure {}

/// Original prepared module, or an explicitly selected historical receipt source.
#[derive(Clone, Debug)]
pub enum NativeModuleSource {
    /// Direct Rust construction/admission capability; no JSON selector is needed.
    Prepared(pse_host::study_module::StudyModuleReceipt),
    /// Historical state contributes its original immutable module receipt only.
    RecordedState(PathBuf),
}
impl NativeModuleSource {
    fn into_canonical(self) -> Result<StudyModuleReceipt, CanonicalError> {
        match self {
            Self::Prepared(receipt) => Ok(StudyModuleReceipt::from_prepared(receipt)),
            Self::RecordedState(state) => StudyModuleReceipt::read(&state),
        }
    }
}
/// Explicit already-prepared inputs; construction performs no source build or discovery.
#[derive(Clone, Debug)]
pub struct NativeServiceSpec {
    /// Exact immutable server capability, including admitted import provenance.
    pub server: PreparedServer,
    /// Explicit application helper executable built with native-service.
    pub helper: PathBuf,
    /// Exact prepared native capability or retained historical module source.
    pub module_source: NativeModuleSource,
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
    run: PathBuf,
    module_root: PathBuf,
    generation: PathBuf,
    recipe: String,
    bind: SocketAddr,
    nonce: String,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StartupClaim {
    nonce: String,
    pid: u32,
    invocation: String,
}
#[derive(Serialize)]
struct Configuration {
    #[serde(flatten)]
    identity: ServiceIdentity,
    #[serde(flatten)]
    connection: CanonicalConnectionSettings,
    payload_incarnation: String,
    accepting_writes: bool,
    study_module: StudyModuleReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_migration_predecessor: Option<StudyModuleReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_migration_target: Option<StudyModuleReceipt>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisposableIdentity {
    version: u32,
    nonce: String,
}
#[derive(Debug, PartialEq, Eq)]
struct RetainedRecords {
    hashes: Vec<String>,
}
impl RetainedRecords {
    fn read(state: &Path) -> Result<Self, NativeServiceError> {
        let config: serde_json::Value = private_read(&state.join("config.json"))?;
        let mut source = config.clone();
        let mut target = config.clone();
        if let Some(predecessor) = config.get("schema_migration_predecessor") {
            source["study_module"] = predecessor.clone();
        }
        if let Some(prepared) = config.get("schema_migration_target") {
            target["study_module"] = prepared.clone();
        }
        if config != source && config != target {
            return Err(NativeServiceError::Refused(
                "local module selection is neither prepared source nor prepared target",
            ));
        }
        let mut hashes = vec![
            hex::encode(Sha256::digest(json(&source)?)),
            hex::encode(Sha256::digest(json(&target)?)),
        ];
        for name in ["credentials.json", "original-module-receipt.json"] {
            let value: serde_json::Value = private_read(&state.join(name))?;
            hashes.push(hex::encode(Sha256::digest(json(&value)?)));
        }
        if config.get("schema_migration_target").is_some() {
            let value: serde_json::Value =
                private_read(&state.join("target-module-source-receipt.json"))?;
            hashes.push(hex::encode(Sha256::digest(json(&value)?)));
        }
        Ok(Self { hashes })
    }
    fn verify(&self, state: &Path) -> Result<(), NativeServiceError> {
        if *self != Self::read(state)? {
            return Err(NativeServiceError::Refused(
                "retained canonical configuration, credentials or module provenance changed",
            ));
        }
        Ok(())
    }
}
struct OwnedState {
    path: PathBuf,
    device: u64,
    inode: u64,
    nonce: String,
}
impl std::fmt::Debug for OwnedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OwnedState").field(&self.path).finish()
    }
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, NativeServiceError> {
    serde_json::to_vec(value)
        .map_err(|_| NativeServiceError::Refused("cannot encode private service record"))
}
fn private_path(path: &Path, directory: bool) -> Result<(), NativeServiceError> {
    if !path.is_absolute() {
        return Err(NativeServiceError::Refused(
            "private service path must be absolute",
        ));
    }
    for parent in path.ancestors() {
        if fs::symlink_metadata(parent)?.file_type().is_symlink() {
            return Err(NativeServiceError::Refused(
                "private service path traverses a symlink",
            ));
        }
    }
    let metadata = fs::metadata(path)?;
    if (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(NativeServiceError::Refused(
            "service records must be private and owned by the current user",
        ));
    }
    Ok(())
}
fn private_read<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, NativeServiceError> {
    private_path(path, false)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
        || metadata.len() > PRIVATE_BYTES
        || metadata.nlink() != 1
    {
        return Err(NativeServiceError::Refused(
            "private service record extent or link count differs",
        ));
    }
    let mut bytes = Vec::new();
    file.take(PRIVATE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > PRIVATE_BYTES {
        return Err(NativeServiceError::Refused(
            "private service record exceeds its bound",
        ));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| NativeServiceError::Refused("invalid private service record"))
}
fn private_write<T: Serialize>(path: &Path, value: &T) -> Result<(), NativeServiceError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    file.write_all(&json(value)?)?;
    file.sync_all()?;
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), NativeServiceError> {
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    private_path(path, true)
}
fn secret() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn deadline_left(deadline: Instant) -> Result<(), NativeServiceError> {
    if Instant::now() >= deadline {
        Err(NativeServiceError::Refused(
            "original service deadline expired",
        ))
    } else {
        Ok(())
    }
}
fn verify_run(state: &Path, run: &Path) -> Result<(), NativeServiceError> {
    private_path(run, true)?;
    if run.parent() != Some(state.join("runs").as_path()) {
        return Err(NativeServiceError::Refused(
            "private launch directory differs from owned state",
        ));
    }
    Ok(())
}
impl OwnedState {
    fn verify(&self) -> Result<(), NativeServiceError> {
        private_path(&self.path, true)?;
        let metadata = fs::metadata(&self.path)?;
        let identity: DisposableIdentity = private_read(&self.path.join("disposable.json"))?;
        if metadata.dev() != self.device
            || metadata.ino() != self.inode
            || identity.version != 1
            || identity.nonce != self.nonce
        {
            return Err(NativeServiceError::Refused(
                "created disposable directory identity changed",
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct OwnedIngress {
    intent: LaunchIntent,
    unit: Option<OwnedUnit>,
    settled: bool,
    ready: bool,
    drained: Option<UnitObservation>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerPayload {
    version: u32,
    state: PathBuf,
    run: PathBuf,
    nonce: String,
    index: usize,
    initial_schema: InitialSchema,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InitialSchema {
    Current,
    PreEpoch,
    SchemaV2,
}

/// Retained ownership of one newly constructed service. Not cloneable and no Drop cleanup claims.
#[derive(Debug)]
pub struct NativeCanonicalService {
    state: OwnedState,
    records: RetainedRecords,
    server_generation: PathBuf,
    server_recipe: String,
    module_root: PathBuf,
    run: PathBuf,
    reopening: bool,
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
    #[cfg(feature = "test-support")]
    schema_writes: Vec<crate::canonical::migration::SchemaFenceWrite>,
    ingresses: Vec<OwnedIngress>,
    process_drained: Option<UnitObservation>,
    closing: bool,
    stopped: bool,
    disposed: bool,
    panic_settlement: Option<NativeServiceError>,
}
/// State created by the private fixture factory, retained only after physical shutdown.
/// This capability cannot be constructed from an existing path and has no Drop cleanup.
#[derive(Debug)]
pub struct StoppedCanonicalState {
    service: NativeCanonicalService,
}
/// A refused transfer retains the original service and every partial physical owner.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("stopped-state transfer refused: {source}")]
#[diagnostic(code(runtime::infrastructure))]
pub struct NativeServiceTransferFailure {
    owner: Box<NativeCanonicalService>,
    #[source]
    source: Box<NativeServiceError>,
}
impl NativeServiceTransferFailure {
    /// Recover the same owner for explicit settlement or disposal retry.
    pub fn into_parts(self) -> (NativeCanonicalService, NativeServiceError) {
        (*self.owner, *self.source)
    }
}
/// A failed reopen retains the stopped state, including partially prepared private records.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("owned stopped-state reopen refused: {source}")]
#[diagnostic(code(runtime::infrastructure))]
pub struct StoppedStateReopenFailure {
    owner: Box<StoppedCanonicalState>,
    #[source]
    source: Box<NativeServiceError>,
}
impl StoppedStateReopenFailure {
    /// Recover the same stopped owner; no failed transition deletes its directory.
    pub fn into_parts(self) -> (StoppedCanonicalState, NativeServiceError) {
        (*self.owner, *self.source)
    }
}
impl StoppedCanonicalState {
    /// The same private nonempty directory retained from the original service.
    pub fn state(&self) -> &Path {
        self.service.state()
    }
    /// Delete only this retained disposable state, using the original settlement proof.
    /// # Errors
    /// Original directory identity changed or explicit deletion failed.
    pub fn dispose(&mut self) -> Result<(), NativeServiceError> {
        self.service.dispose()
    }
    /// Reopen this capability's same database under a fresh process owner.
    /// No schema installer runs on reopen; original credentials, payload and module remain.
    /// # Errors
    /// Original state/provenance differs or private launch preparation fails; the
    /// returned failure retains this stopped capability for retry or explicit disposal.
    pub fn reopen(self) -> Result<NativeCanonicalService, StoppedStateReopenFailure> {
        let prepared = (|| {
            let original = &self.service;
            original.state.verify()?;
            original.records.verify(&original.state.path)?;
            verify_run(&original.state.path, &original.run)?;
            if original.disposed {
                return Err(NativeServiceError::Refused(
                    "disposed state cannot be reopened",
                ));
            }
            let payload: StartupPayload = private_read(&original.run.join("startup.json"))?;
            if payload.state != original.state.path
                || payload.run != original.run
                || payload.bind != original.bind
                || payload.nonce != original.state.nonce
                || payload.generation != original.server_generation
                || payload.recipe != original.server_recipe
                || payload.module_root != original.module_root
            {
                return Err(NativeServiceError::Refused(
                    "retained original startup coordinates differ",
                ));
            }
            server::reopen_generation(&payload.generation, &payload.recipe)?;
            StudyModuleReceipt::read(&original.state.path)?;
            private_read::<Credentials>(&original.state.path.join("credentials.json"))?;
            CanonicalOptions::from_state(&original.state.path)?;
            let intent = LaunchIntent::fresh()?;
            let run = original
                .state
                .path
                .join("runs")
                .join(uuid::Uuid::new_v4().to_string());
            private_directory(&run)?;
            let next = StartupPayload {
                run: run.clone(),
                ..payload
            };
            private_write(&run.join("startup.json"), &next)?;
            File::open(&run)?.sync_all()?;
            Ok::<_, NativeServiceError>((intent, run))
        })();
        let (intent, run) = match prepared {
            Ok(prepared) => prepared,
            Err(source) => {
                return Err(StoppedStateReopenFailure {
                    owner: Box::new(self),
                    source: Box::new(source),
                });
            }
        };
        let mut service = self.service;
        service.run = run;
        service.reopening = true;
        service.initialized = false;
        service.intent = intent;
        service.manager = None;
        service.unit = None;
        service.launch_dispatched = false;
        service.ready = None;
        service.connections.clear();
        service.maintenances.clear();
        #[cfg(feature = "test-support")]
        service.schema_writes.clear();
        service.ingresses.clear();
        service.process_drained = None;
        service.closing = false;
        service.stopped = false;
        service.panic_settlement = None;
        Ok(service)
    }
}
impl NativeCanonicalService {
    /// Create a fresh private disposable state and copy a validated immutable policy artifact.
    /// This does not start a process, install dependencies or change existing state.
    /// # Errors
    /// Invalid prepared inputs, private-state creation failure, or module admission failure.
    pub fn new(spec: NativeServiceSpec) -> Result<Self, NativeServiceError> {
        Self::new_with_schema(spec, InitialSchema::Current)
    }
    /// Construct a newly owned predecessor fixture only, for explicit adjacent migration.
    /// The predecessor never grants ordinary application readiness.
    /// # Errors
    /// The same private construction and artifact admission failures as [`Self::new`].
    pub fn new_pre_epoch_fixture(spec: NativeServiceSpec) -> Result<Self, NativeServiceError> {
        Self::new_with_schema(spec, InitialSchema::PreEpoch)
    }
    /// Create an exact frozen schema-v2 fixture in a newly owned absent catalog.
    /// # Errors
    /// The same private construction and artifact admission failures as [`Self::new`].
    pub fn new_v2_fixture(spec: NativeServiceSpec) -> Result<Self, NativeServiceError> {
        Self::new_with_schema(spec, InitialSchema::SchemaV2)
    }
    /// Create a private v2 store with separate copied old and prepared target artifacts.
    /// The existing configured source stores supply immutable artifacts only.
    /// # Errors
    /// Private artifact admission, immutable copy or fixture construction fails.
    pub fn new_v2_upgrade_fixture(
        spec: NativeServiceSpec,
        target_source: NativeModuleSource,
    ) -> Result<Self, NativeServiceError> {
        Self::new_with_target(spec, InitialSchema::SchemaV2, Some(target_source))
    }
    fn new_with_schema(
        spec: NativeServiceSpec,
        initial_schema: InitialSchema,
    ) -> Result<Self, NativeServiceError> {
        Self::new_with_target(spec, initial_schema, None)
    }
    fn new_with_target(
        spec: NativeServiceSpec,
        initial_schema: InitialSchema,
        target_source: Option<NativeModuleSource>,
    ) -> Result<Self, NativeServiceError> {
        if !spec.helper.is_absolute() || !spec.helper.is_file() {
            return Err(NativeServiceError::Refused(
                "explicit native application helper is required",
            ));
        }
        let intent = LaunchIntent::fresh()?;
        let root = spec.state_parent.canonicalize()?;
        let temporary = tempfile::Builder::new()
            .prefix("pse-canonical-native-")
            .tempdir_in(root)?;
        let path = temporary.path().canonicalize()?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        let module = spec.module_source.into_canonical()?;
        private_write(&path.join("original-module-receipt.json"), &module)?;
        let module_root = path.join(if target_source.is_some() {
            "modules"
        } else {
            "module"
        });
        let (module, target) = if let Some(target_source) = target_source {
            private_directory(&module_root)?;
            let old = module.copy_into_private(&path, &module_root.join("predecessor"))?;
            let target = target_source.into_canonical()?;
            private_write(&path.join("target-module-source-receipt.json"), &target)?;
            let target = target.copy_into_private(&path, &module_root.join("target"))?;
            if target.sha256 == old.sha256 {
                return Err(NativeServiceError::Refused(
                    "module upgrade fixture requires distinct old and target artifacts",
                ));
            }
            (old, Some(target))
        } else {
            (module.copy_into_private(&path, &module_root)?, None)
        };
        for directory in ["tmp", "cache", "home", "runs"] {
            private_directory(&path.join(directory))?;
        }
        let credentials = Credentials {
            username: ROOT_USER.into(),
            password: secret(),
            selection_username: VIEWER_USER.into(),
            selection_password: secret(),
        };
        private_write(&path.join("credentials.json"), &credentials)?;
        // Pin a fresh loopback port. The release-to-exec race refuses on collision;
        // actual kernel socket ownership is established before authenticated RPC.
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let bind = listener.local_addr()?;
        let nonce = uuid::Uuid::new_v4().to_string();
        let run = path.join("runs").join(uuid::Uuid::new_v4().to_string());
        private_directory(&run)?;
        let payload = StartupPayload {
            version: 1,
            state: path.clone(),
            run: run.clone(),
            module_root: module_root.clone(),
            generation: spec.server.generation().to_path_buf(),
            recipe: spec.server.recipe().into(),
            bind,
            nonce: nonce.clone(),
        };
        private_write(&run.join("startup.json"), &payload)?;
        let config = Configuration {
            identity: ServiceIdentity {
                profile_version: pse_model::document::Version,
                endpoint: format!("grpc://{bind}"),
                namespace: "native_fixture".into(),
                database: "canonical".into(),
                service_state: None,
                receiver_generation: None,
            },
            connection: CanonicalConnectionSettings {
                schema_interpretation: crate::generated::surreal::INTERPRETATION.into(),
                max_message_bytes: MESSAGE_BYTES,
                credentials_file: path.join("credentials.json").to_string_lossy().into_owned(),
            },
            payload_incarnation: uuid::Uuid::new_v4().to_string(),
            accepting_writes: true,
            schema_migration_predecessor: target.as_ref().map(|_| module.clone()),
            schema_migration_target: target,
            study_module: module,
        };
        private_write(&path.join("config.json"), &config)?;
        private_write(
            &path.join("disposable.json"),
            &DisposableIdentity {
                version: 1,
                nonce: nonce.clone(),
            },
        )?;
        File::open(&path)?.sync_all()?;
        let metadata = fs::metadata(&path)?;
        let records = RetainedRecords::read(&path)?;
        let state = OwnedState {
            path: temporary.keep(),
            device: metadata.dev(),
            inode: metadata.ino(),
            nonce,
        };
        drop(listener);
        Ok(Self {
            state,
            records,
            server_generation: payload.generation,
            server_recipe: payload.recipe,
            module_root,
            run,
            reopening: false,
            initial_schema,
            initialized: false,
            helper: spec.helper,
            bind,
            intent,
            manager: None,
            unit: None,
            launch_dispatched: false,
            ready: None,
            connections: Vec::new(),
            maintenances: Vec::new(),
            #[cfg(feature = "test-support")]
            schema_writes: Vec::new(),
            ingresses: Vec::new(),
            process_drained: None,
            closing: false,
            stopped: false,
            disposed: false,
            panic_settlement: None,
        })
    }
    /// Cleanup and diagnostic-write failures retained if a body panic was resumed.
    pub fn panic_settlement_failure(&self) -> Option<&NativeServiceError> {
        self.panic_settlement.as_ref()
    }
    /// State created by this owner, retained across failures for diagnosis.
    pub fn state(&self) -> &Path {
        &self.state.path
    }
    /// Original manager intent; a unit name alone grants no stop authority.
    pub fn intent(&self) -> &LaunchIntent {
        &self.intent
    }
    /// Original ingress launch names for independent diagnostics; no control authority.
    pub fn ingress_intents(&self) -> impl Iterator<Item = &LaunchIntent> {
        self.ingresses.iter().map(|ingress| &ingress.intent)
    }
    /// Application-ready canonical operations, established by actual D construction.
    /// # Errors
    /// Original service admission is closed or current schema readiness is absent.
    pub fn ready(&self) -> Result<&SchemaReady, NativeServiceError> {
        if self.closing {
            return Err(NativeServiceError::Refused(
                "original native service admission is closed",
            ));
        }
        self.ready.as_ref().ok_or(NativeServiceError::Refused(
            "native canonical schema readiness has not been established",
        ))
    }
    fn service_spec(&self) -> ServiceSpec {
        ServiceSpec {
            intent: self.intent.clone(),
            executable: self.helper.clone(),
            arguments: vec![
                "exec".into(),
                self.run.join("startup.json").to_string_lossy().into_owned(),
            ],
            environment: vec!["PATH=/usr/bin:/bin".into(), "LANG=C.UTF-8".into()],
        }
    }
    /// Launch the original owned native process once, then establish protocol and schema readiness.
    /// # Errors
    /// Closed admission, repeated dispatch, original-deadline expiry, native ownership,
    /// startup, protocol, or schema construction failure. Original owners remain retained.
    pub async fn launch(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        if self.closing || self.disposed {
            return Err(NativeServiceError::Refused(
                "original native service admission is closed",
            ));
        }
        self.state.verify()?;
        self.records.verify(&self.state.path)?;
        deadline_left(deadline)?;
        if self.launch_dispatched {
            return Err(NativeServiceError::Refused(
                "original service launch already dispatched; reconcile, do not replay",
            ));
        }
        let manager = host_observation(deadline, HostManager::connect_user(None)).await?;
        let enclosing = host_observation(deadline, manager.current_unit()).await?;
        if !host_observation(deadline, manager.native_workload(&enclosing)).await? {
            return Err(NativeServiceError::Refused(
                "native canonical fixture requires authenticated native workload containment",
            ));
        }
        self.manager = Some(manager.clone());
        self.launch_dispatched = true;
        let parent = manager.borrow(enclosing.unit);
        self.unit = Some(
            manager
                .start_service_bound(
                    &self.service_spec(),
                    &parent,
                    &ResourceControls::default(),
                    deadline,
                )
                .await?,
        );
        self.wait_owned_listener(deadline).await?;
        if !self.reopening {
            crate::transport::operation_scope(deadline, self.provision_viewer(deadline)).await?;
        }
        let options = CanonicalOptions::from_state(&self.state.path)?;
        crate::transport::operation_scope(deadline, async {
            match self.initial_schema {
                InitialSchema::Current => {
                    if !self.reopening {
                        CanonicalStore::initialize_database(&options).await?;
                    }
                    let ready = CanonicalStore::connect_ready(&options).await?;
                    self.connections.push(ready.store().clone());
                    self.ready = Some(ready);
                }
                InitialSchema::PreEpoch | InitialSchema::SchemaV2 if self.reopening => {
                    let maintenance = SchemaMaintenance::connect(&options).await?;
                    self.maintenances.push(maintenance.clone());
                    maintenance.inspect().await?;
                }
                InitialSchema::PreEpoch => {
                    CanonicalStore::install_pre_epoch_fixture(&options).await?
                }
                InitialSchema::SchemaV2 => {
                    CanonicalStore::install_schema_v2_fixture(&options).await?
                }
            }
            self.initialized = true;
            Ok::<(), CanonicalError>(())
        })
        .await?;
        Ok(())
    }
    async fn wait_owned_listener(&self, deadline: Instant) -> Result<(), NativeServiceError> {
        let unit = self.unit.as_ref().ok_or(NativeServiceError::Refused(
            "original service capability is unavailable",
        ))?;
        loop {
            deadline_left(deadline)?;
            let observed = host_observation(deadline, unit.observe()).await?;
            if observed.main_pid == 0
                || matches!(observed.active_state.as_str(), "failed" | "inactive")
            {
                return Err(NativeServiceError::Refused(
                    "owned canonical service exited before listener readiness",
                ));
            }
            if let Ok(claim) = private_read::<StartupClaim>(&self.run.join("started.json")) {
                if claim.nonce != self.state.nonce
                    || claim.pid != observed.main_pid
                    || claim.invocation != hex_bytes(&observed.invocation)
                {
                    return Err(NativeServiceError::Refused(
                        "native startup claim differs from original unit invocation",
                    ));
                }
                if owns_listener(observed.main_pid, self.bind)? {
                    let payload: StartupPayload = private_read(&self.run.join("startup.json"))?;
                    let exact = server::reopen_generation(&payload.generation, &payload.recipe)?;
                    if fs::read_link(format!("/proc/{}/exe", observed.main_pid))?
                        != exact.executable()
                    {
                        return Err(NativeServiceError::Refused(
                            "owned listener is not the exact prepared server executable",
                        ));
                    }
                    let after = host_observation(deadline, unit.observe()).await?;
                    if after.main_pid == observed.main_pid
                        && after.invocation == observed.invocation
                    {
                        deadline_left(deadline)?;
                        return Ok(());
                    }
                    return Err(NativeServiceError::Refused(
                        "owned listener changed during observation",
                    ));
                }
            }
            tokio::time::sleep_until(
                deadline
                    .min(Instant::now() + Duration::from_millis(20))
                    .into(),
            )
            .await;
        }
    }
    async fn provision_viewer(&self, deadline: Instant) -> Result<(), NativeServiceError> {
        deadline_left(deadline)?;
        let credentials: Credentials = private_read(&self.state.path.join("credentials.json"))?;
        let client = Client::connect(self.bind, MESSAGE_BYTES)
            .await
            .map_err(CanonicalError::from)?;
        let context = || RequestContext::control(deadline);
        let provisioned = async {
            client.signin(Root { username: credentials.username, password: credentials.password }).request_context(context()).await?;
            // Exact 3.3.4 PASSWORD grammar requires a literal, not a parameter.
            // The SDK owns escaping; this SQL travels only over the native RPC.
            let password=Value::String(credentials.selection_password).to_sql();
            let response = client.query(format!("DEFINE USER OVERWRITE `pse-selection` ON ROOT PASSWORD {password} ROLES VIEWER;"))
                .request_context(context()).await?;
            complete_response(response)?;
            Ok::<(),CanonicalError>(())
        }.await;
        let drained = client.disconnect().await.map_err(CanonicalError::from);
        let recorded = if provisioned.is_err() || drained.is_err() {
            private_write(
                &self.run.join("viewer-bootstrap-failed.json"),
                &serde_json::json!({"request":format!("{provisioned:?}"),"drain":format!("{drained:?}")}),
            )
        } else {
            Ok(())
        };
        let private_failure =
            |original| NativeServiceError::Bootstrap(ViewerBootstrapFailure { original });
        let result = match (provisioned, drained) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(private_failure(error)),
            (Err(error), Err(cleanup)) => Err(NativeServiceError::Settlement {
                state: self.state.path.clone(),
                failures: vec![private_failure(error), private_failure(cleanup)],
            }),
        };
        match (result, recorded) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(error), Err(record)) => Err(NativeServiceError::Settlement {
                state: self.state.path.clone(),
                failures: vec![error, record],
            }),
        }
    }

    /// Start a separate native control ingress with its own live canonical connection.
    /// Its exact original owned unit is retained before the first dispatch await.
    /// # Errors
    /// Closed or uninitialized admission, native launch uncertainty, original-deadline
    /// expiry, or failed original listener and connection readiness.
    pub async fn start_ingress(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        if self.closing {
            return Err(NativeServiceError::Refused(
                "original native service admission is closed",
            ));
        }
        if !self.initialized {
            return Err(NativeServiceError::Refused(
                "original native fixture schema installation did not complete",
            ));
        }
        if matches!(self.initial_schema, InitialSchema::Current) {
            self.ready()?;
        }
        self.state.verify()?;
        deadline_left(deadline)?;
        let manager = self
            .manager
            .as_ref()
            .ok_or(NativeServiceError::Refused(
                "service manager is unavailable",
            ))?
            .clone();
        let index = self.ingresses.len();
        let payload = WorkerPayload {
            version: 1,
            state: self.state.path.clone(),
            run: self.run.clone(),
            nonce: self.state.nonce.clone(),
            index,
            initial_schema: self.initial_schema,
        };
        let path = self.run.join(format!("worker-{index}.json"));
        private_write(&path, &payload)?;
        let intent = LaunchIntent::fresh()?;
        let spec = ServiceSpec {
            intent: intent.clone(),
            executable: self.helper.clone(),
            arguments: vec!["worker-exec".into(), path.to_string_lossy().into_owned()],
            environment: vec!["PATH=/usr/bin:/bin".into(), "LANG=C.UTF-8".into()],
        };
        let parent = manager.borrow(
            host_observation(deadline, manager.current_unit())
                .await?
                .unit,
        );
        self.ingresses.push(OwnedIngress {
            intent,
            unit: None,
            settled: false,
            ready: false,
            drained: None,
        });
        self.ingresses[index].unit = Some(
            manager
                .start_service_bound(&spec, &parent, &ResourceControls::default(), deadline)
                .await?,
        );
        let unit = self.ingresses[index]
            .unit
            .as_ref()
            .ok_or(NativeServiceError::Refused(
                "original ingress capability unavailable",
            ))?;
        loop {
            deadline_left(deadline)?;
            let observed = host_observation(deadline, unit.observe()).await?;
            if observed.main_pid == 0
                || matches!(observed.active_state.as_str(), "failed" | "inactive")
            {
                return Err(NativeServiceError::Refused(
                    "owned ingress exited before readiness",
                ));
            }
            if let Ok(claim) =
                private_read::<StartupClaim>(&self.run.join(format!("worker-{index}.ready.json")))
            {
                if claim.nonce != self.state.nonce
                    || claim.pid != observed.main_pid
                    || claim.invocation != hex_bytes(&observed.invocation)
                {
                    return Err(NativeServiceError::Refused(
                        "original ingress invocation differs",
                    ));
                }
                if owns_unix_listener(
                    observed.main_pid,
                    &self.run.join(format!("worker-{index}.sock")),
                )? {
                    let after = host_observation(deadline, unit.observe()).await?;
                    if after.main_pid == observed.main_pid
                        && after.invocation == observed.invocation
                    {
                        deadline_left(deadline)?;
                        self.ingresses[index].ready = true;
                        return Ok(());
                    }
                    return Err(NativeServiceError::Refused(
                        "owned ingress changed during observation",
                    ));
                }
            }
            tokio::time::sleep_until(
                deadline
                    .min(Instant::now() + Duration::from_millis(20))
                    .into(),
            )
            .await;
        }
    }
    /// Exercise the last original ingress through its private native control socket.
    /// A successful reply requires a canonical read over that worker's owned connection.
    /// # Errors
    /// No owned ingress, failed private socket request, invalid reply, or deadline expiry.
    pub async fn probe_ingress(&self, deadline: Instant) -> Result<String, NativeServiceError> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let index = self
            .ingresses
            .len()
            .checked_sub(1)
            .ok_or(NativeServiceError::Refused("no native ingress was started"))?;
        tokio::time::timeout_at(deadline.into(), async {
            let mut socket =
                tokio::net::UnixStream::connect(self.run.join(format!("worker-{index}.sock")))
                    .await?;
            socket.write_all(&[0]).await?;
            let mut answer = Vec::new();
            socket.take(256).read_to_end(&mut answer).await?;
            String::from_utf8(answer)
                .map_err(|_| NativeServiceError::Refused("invalid native ingress reply"))
        })
        .await
        .map_err(|_| NativeServiceError::Refused("original ingress probe deadline expired"))?
    }
    async fn stop_ingresses(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        stop_ingress_owners(
            self.manager.as_ref(),
            &self.state,
            &self.run,
            &mut self.ingresses,
            deadline,
        )
        .await
    }
    /// Close and physically join the actual native control ingress owners.
    /// This future is the B/C premise supplied to schema maintenance; the database
    /// remains live and D separately drains each supplied canonical connection.
    /// # Errors
    /// Any original ingress fails physical settlement or its connection-drain proof.
    pub async fn stop_ingress_and_join(&mut self, deadline: Instant) -> Result<(), CanonicalError> {
        self.stop_ingresses(deadline)
            .await
            .map_err(|error| CanonicalError::MigrationDrain {
                failures: vec![canonical_drain_failure(error)],
            })
    }
    /// Execute fixture work and settle the original physical owners even if the body panics.
    /// A failed settlement retains this object's private state for explicit diagnosis/retry.
    /// # Errors
    /// The body or any independent settlement fails; all such failures are retained.
    /// # Panics
    /// Resumes the original body panic after attempting every independent settlement.
    pub async fn run<T, F>(&mut self, deadline: Instant, body: F) -> Result<T, NativeServiceError>
    where
        F: for<'a> FnOnce(
            &'a mut Self,
        ) -> std::pin::Pin<
            Box<dyn Future<Output = Result<T, NativeServiceError>> + Send + 'a>,
        >,
    {
        let outcome =
            std::panic::AssertUnwindSafe(crate::transport::operation_scope(deadline, async {
                body(self).await
            }))
            .catch_unwind()
            .await;
        let settlement = self.shutdown(deadline).await;
        match (outcome, settlement) {
            (Ok(Ok(value)), Ok(())) => Ok(value),
            (Ok(Err(error)), Ok(())) => Err(error),
            (Ok(Ok(_)), Err(error)) => Err(error),
            (Ok(Err(error)), Err(cleanup)) => Err(NativeServiceError::Settlement {
                state: self.state.path.clone(),
                failures: vec![error, cleanup],
            }),
            (Err(payload), cleanup) => {
                if let Err(error) = cleanup {
                    let record = private_write(
                        &self.state.path.join("settlement-failed.json"),
                        &error.to_string(),
                    );
                    let mut failures = vec![error];
                    if let Err(error) = record {
                        failures.push(error);
                    }
                    let details = failures
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; ");
                    if let Err(error) = writeln!(
                        std::io::stderr().lock(),
                        "native canonical fixture retained at {} after panic cleanup failures: {details}",
                        self.state.path.display()
                    ) {
                        failures.push(error.into());
                    }
                    self.panic_settlement = Some(NativeServiceError::Settlement {
                        state: self.state.path.clone(),
                        failures,
                    });
                }
                std::panic::resume_unwind(payload)
            }
        }
    }
    /// Open explicit schema maintenance and retain its same original transport owner.
    /// # Errors
    /// Original admission is closed, installation is incomplete, or connection setup fails.
    pub async fn maintenance(&mut self) -> Result<SchemaMaintenance, NativeServiceError> {
        if self.closing {
            return Err(NativeServiceError::Refused(
                "original native service admission is closed",
            ));
        }
        if !self.initialized {
            return Err(NativeServiceError::Refused(
                "original native fixture installation did not complete",
            ));
        }
        let maintenance =
            SchemaMaintenance::connect(&CanonicalOptions::from_state(&self.state.path)?).await?;
        self.maintenances.push(maintenance.clone());
        Ok(maintenance)
    }
    /// Atomically select the prepared target only after its exact verified activation.
    /// Repeating this with the original receipt settles a lost local publication ACK.
    /// No database transformation is replayed and no existing external state is edited.
    /// # Errors
    /// Original fixture admission, receipt target, private configuration or publication differs.
    pub fn promote_activated_module(
        &mut self,
        receipt: &SchemaMigrationReceipt,
    ) -> Result<(), NativeServiceError> {
        if self.closing || self.disposed {
            return Err(NativeServiceError::Refused(
                "original native service admission is closed",
            ));
        }
        self.state.verify()?;
        self.records.verify(&self.state.path)?;
        let target = receipt.target_module().ok_or(NativeServiceError::Refused(
            "activation has no explicit prepared target module",
        ))?;
        let path = self.state.path.join("config.json");
        let mut configuration: serde_json::Value = private_read(&path)?;
        let prepared = serde_json::to_value(&target.receipt)
            .map_err(|_| NativeServiceError::Refused("cannot encode prepared module selection"))?;
        if configuration.get("schema_migration_target") != Some(&prepared) {
            return Err(NativeServiceError::Refused(
                "verified activation belongs to a different prepared module selection",
            ));
        }
        if configuration.get("study_module") == Some(&prepared) {
            return Ok(());
        }
        configuration["study_module"] = prepared;
        let temporary = self
            .state
            .path
            .join(format!("module-selection-{}.json", uuid::Uuid::new_v4()));
        private_write(&temporary, &configuration)?;
        fs::rename(&temporary, &path)?;
        File::open(&self.state.path)?.sync_all()?;
        self.records.verify(&self.state.path)
    }
    /// Admit one fixed schema-fenced guard write and retain its original held driver.
    /// This fixture-only control never accepts SQL, arbitrary rows or another deployment.
    /// # Errors
    /// Original service/fence differs, or actual authentication/admission/transaction staging fails.
    #[cfg(feature = "test-support")]
    pub async fn stage_schema_fence_write(
        &mut self,
        ready: &SchemaReady,
        deadline: Instant,
    ) -> Result<crate::canonical::migration::SchemaFenceWrite, NativeServiceError> {
        if self.closing || !self.initialized || ready.store().deployment_state() != self.state.path
        {
            return Err(NativeServiceError::Refused(
                "held schema write requires this initialized private fixture",
            ));
        }
        let write = crate::canonical::migration::SchemaFenceWrite::new(ready, deadline);
        self.schema_writes.push(write.clone());
        write.stage().await?;
        Ok(write)
    }
    /// Retain an actual successor connection from this same fixture for terminal drain.
    /// This grants no readiness or migration authority.
    /// # Errors
    /// The supplied connection belongs to a different deployment.
    pub fn retain_connection(&mut self, store: CanonicalStore) -> Result<(), NativeServiceError> {
        if store.deployment_state() != self.state.path {
            return Err(NativeServiceError::Refused(
                "connection belongs to a different deployment",
            ));
        }
        self.connections.push(store);
        Ok(())
    }
    /// Open an additional independently owned D connection and retain it for terminal drain.
    /// # Errors
    /// Closed or uninitialized admission, connection setup, or actual schema readiness fails.
    pub async fn connect(&mut self) -> Result<SchemaReady, NativeServiceError> {
        if self.closing {
            return Err(NativeServiceError::Refused(
                "original native service admission is closed",
            ));
        }
        if !self.initialized {
            return Err(NativeServiceError::Refused(
                "original native fixture installation did not complete",
            ));
        }
        let ready =
            CanonicalStore::connect_ready(&CanonicalOptions::from_state(&self.state.path)?).await?;
        self.connections.push(ready.store().clone());
        self.ready = Some(ready.clone());
        Ok(ready)
    }
    /// Stop and physically join the original service process without needing database connectivity.
    /// # Errors
    /// The original service was never dispatched, remains uncertain, or cannot establish
    /// physical settlement before the original deadline.
    pub async fn stop_process(
        &mut self,
        deadline: Instant,
    ) -> Result<UnitObservation, NativeServiceError> {
        self.closing = true;
        stop_original_process(
            self.manager.as_ref(),
            &self.intent,
            &mut self.unit,
            self.launch_dispatched,
            &mut self.process_drained,
            deadline,
        )
        .await?
        .ok_or(NativeServiceError::Refused("service was never launched"))
    }
    /// Drain independent clients and ingresses concurrently, then stop the original
    /// database under the same deadline, retaining every failure.
    /// # Errors
    /// Any independent canonical connection, ingress, or database settlement fails.
    pub async fn shutdown(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        self.closing = true;
        #[cfg(feature = "test-support")]
        let schema_drains = futures_util::future::join_all(
            self.schema_writes
                .iter()
                .map(crate::canonical::migration::SchemaFenceWrite::cancel),
        );
        #[cfg(not(feature = "test-support"))]
        let schema_drains = async { Vec::<Result<(), CanonicalError>>::new() };
        // Connection close can still dispatch cancellation or protection-release RPCs.
        // Keep their original server alive until those owners have settled; process
        // stop is dependent teardown, even though sibling client drains are independent.
        let (drains, maintenance_drains, schema_drains, ingress_drain) = tokio::join!(
            futures_util::future::join_all(
                self.connections
                    .iter()
                    .map(|store| store.drain_owned_connection_until(deadline.into()))
            ),
            futures_util::future::join_all(self.maintenances.iter().map(|maintenance| {
                crate::transport::operation_scope(deadline, maintenance.disconnect())
            })),
            schema_drains,
            stop_ingress_owners(
                self.manager.as_ref(),
                &self.state,
                &self.run,
                &mut self.ingresses,
                deadline
            )
        );
        let mut failures: Vec<_> = drains
            .into_iter()
            .chain(maintenance_drains)
            .chain(schema_drains)
            .filter_map(Result::err)
            .map(NativeServiceError::from)
            .collect();
        if let Err(error) = ingress_drain {
            failures.push(error);
        }
        // Always attempt physical stop, including after any failed sibling drain.
        if let Err(error) = stop_original_process(
            self.manager.as_ref(),
            &self.intent,
            &mut self.unit,
            self.launch_dispatched,
            &mut self.process_drained,
            deadline,
        )
        .await
        {
            failures.push(error);
        }
        if !failures.is_empty() {
            return Err(NativeServiceError::Settlement {
                state: self.state.path.clone(),
                failures,
            });
        }
        self.stopped = true;
        Ok(())
    }
    /// Transfer only fully initialized, physically stopped disposable state.
    /// Closing this service remains terminal; the returned capability owns a future
    /// fresh service construction, rather than reopening this original process owner.
    /// # Errors
    /// Initialization, original physical shutdown or state identity is incomplete.
    /// The typed failure retains this exact service for explicit retry.
    pub fn try_into_stopped(self) -> Result<StoppedCanonicalState, NativeServiceTransferFailure> {
        let checked = (|| {
            if !self.stopped
                || !self.closing
                || !self.initialized
                || !self.launch_dispatched
                || self.process_drained.is_none()
                || self.disposed
            {
                return Err(NativeServiceError::Refused(
                    "initialized original physical shutdown is required before state transfer",
                ));
            }
            self.state.verify()?;
            verify_run(&self.state.path, &self.run)?;
            let receipt = serde_json::json!({
                "service": self.process_drained,
                "ingresses": self.ingresses.iter().map(|ingress| &ingress.drained).collect::<Vec<_>>(),
            });
            let path = self.run.join("stopped.json");
            if path.exists() {
                if private_read::<serde_json::Value>(&path)? != receipt {
                    return Err(NativeServiceError::Refused(
                        "original stopped-state receipt differs",
                    ));
                }
            } else {
                private_write(&path, &receipt)?;
            }
            Ok(())
        })();
        match checked {
            Ok(()) => Ok(StoppedCanonicalState { service: self }),
            Err(source) => Err(NativeServiceTransferFailure {
                owner: Box::new(self),
                source: Box::new(source),
            }),
        }
    }
    /// Delete only this newly created disposable state after explicit successful settlement.
    /// # Errors
    /// Physical settlement is incomplete, created-directory identity differs, or deletion fails.
    pub fn dispose(&mut self) -> Result<(), NativeServiceError> {
        if self.disposed {
            return Ok(());
        }
        if !self.stopped {
            return Err(NativeServiceError::Refused(
                "explicit original physical settlement is required before disposal",
            ));
        }
        self.state.verify()?;
        fs::remove_dir_all(&self.state.path)?;
        self.disposed = true;
        Ok(())
    }
}
fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn owns_listener(pid: u32, bind: SocketAddr) -> Result<bool, NativeServiceError> {
    let SocketAddr::V4(address) = bind else {
        return Err(NativeServiceError::Refused(
            "only native IPv4 loopback listener fixtures are supported",
        ));
    };
    let selected = format!(
        "{:08X}:{:04X}",
        u32::from_le_bytes(address.ip().octets()),
        address.port()
    );
    let table = fs::read_to_string(format!("/proc/{pid}/net/tcp"))?;
    let inodes: BTreeSet<_> = table
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            (fields.get(1).copied() == Some(selected.as_str())
                && fields.get(3).copied() == Some("0A"))
            .then(|| fields.get(9).copied().map(str::to_owned))
            .flatten()
        })
        .collect();
    if inodes.is_empty() {
        return Ok(false);
    }
    for entry in fs::read_dir(format!("/proc/{pid}/fd"))? {
        let entry = entry?;
        if let Ok(target) = fs::read_link(entry.path())
            && let Some(inode) = target
                .to_str()
                .and_then(|target| target.strip_prefix("socket:["))
                .and_then(|target| target.strip_suffix(']'))
            && inodes.contains(inode)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Native exec helper entry. Only a bounded private file supplies credentials;
/// neither manager argv nor manager environment receives their bytes.
/// # Errors
/// Private startup, credential, immutable generation, invocation, or native exec validation fails.
pub fn exec_startup(path: &Path) -> Result<(), NativeServiceError> {
    let payload: StartupPayload = private_read(path)?;
    private_path(&payload.state, true)?;
    if payload.version != 1
        || path != payload.run.join("startup.json")
        || payload.bind.ip() != Ipv4Addr::LOCALHOST
        || payload.bind.port() == 0
    {
        return Err(NativeServiceError::Refused(
            "invalid native startup coordinates",
        ));
    }
    verify_run(&payload.state, &payload.run)?;
    if payload.module_root != payload.state.join("module")
        && payload.module_root != payload.state.join("modules")
    {
        return Err(NativeServiceError::Refused(
            "startup module root differs from private owned artifact directories",
        ));
    }
    private_path(&payload.module_root, true)?;
    let disposable: DisposableIdentity = private_read(&payload.state.join("disposable.json"))?;
    if disposable.version != 1 || disposable.nonce != payload.nonce {
        return Err(NativeServiceError::Refused(
            "startup disposable identity differs",
        ));
    }
    let server = server::reopen_generation(&payload.generation, &payload.recipe)?;
    let module = StudyModuleReceipt::read(&payload.state)?;
    let credentials: Credentials = private_read(&payload.state.join("credentials.json"))?;
    let invocation = std::env::var("INVOCATION_ID").map_err(|_| {
        NativeServiceError::Refused("native startup requires its original systemd invocation")
    })?;
    if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(NativeServiceError::Refused(
            "systemd invocation identity is invalid",
        ));
    }
    private_write(
        &payload.run.join("started.json"),
        &StartupClaim {
            nonce: payload.nonce,
            pid: std::process::id(),
            invocation,
        },
    )?;
    let log = OpenOptions::new()
        .create_new(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(payload.run.join("server.log"))?;
    let datastore = format!(
        "mem://{}?aol=sync&sync=every&versioned=false&snapshot=60s",
        payload.state.join("database").display()
    );
    let error = Command::new(server.executable())
        .args([
            "start",
            "--no-banner",
            "--no-defaults",
            "--deny-guests",
            "--deny-net",
            "--log",
            "warn",
            "--query-timeout=90s",
            "--transaction-timeout=90s",
            "--bind",
        ])
        .arg(payload.bind.to_string())
        .arg("--temporary-directory")
        .arg(payload.state.join("tmp"))
        .arg(datastore)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("HOME", payload.state.join("home"))
        .env("XDG_CACHE_HOME", payload.state.join("cache"))
        .env("TMPDIR", payload.state.join("tmp"))
        .env("SURREAL_USER", credentials.username)
        .env("SURREAL_PASS", credentials.password)
        .env("SURREAL_GRPC_MAX_MESSAGE_SIZE", MESSAGE_BYTES.to_string())
        .env("SURREAL_CAPS_ALLOW_EXPERIMENTAL", "files,surrealism")
        .env("SURREAL_BUCKET_FOLDER_ALLOWLIST", &payload.module_root)
        .envs(module.runtime_environment())
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .exec();
    Err(error.into())
}

fn process_owns_inode(pid: u32, inode: &str) -> Result<bool, NativeServiceError> {
    let expected = format!("socket:[{inode}]");
    for entry in fs::read_dir(format!("/proc/{pid}/fd"))? {
        if fs::read_link(entry?.path())
            .ok()
            .is_some_and(|path| path.as_os_str() == expected.as_str())
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn owns_unix_listener(pid: u32, path: &Path) -> Result<bool, NativeServiceError> {
    let table = fs::read_to_string(format!("/proc/{pid}/net/unix"))?;
    for line in table.lines().skip(1) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.get(7).is_some_and(|field| Path::new(field) == path)
            && fields.get(3).copied() == Some("00010000")
            && let Some(inode) = fields.get(6)
        {
            return process_owns_inode(pid, inode);
        }
    }
    Ok(false)
}

/// Preserve the original systemd PID across a clean-environment worker exec.
/// User-manager environment inheritance never reaches the actual connection/control body.
/// # Errors
/// Private worker identity, invocation validation, or native exec fails.
pub fn exec_worker(path: &Path) -> Result<(), NativeServiceError> {
    let payload: WorkerPayload = private_read(path)?;
    private_path(&payload.state, true)?;
    verify_run(&payload.state, &payload.run)?;
    let identity: DisposableIdentity = private_read(&payload.state.join("disposable.json"))?;
    if payload.version != 1
        || identity.version != 1
        || identity.nonce != payload.nonce
        || path != payload.run.join(format!("worker-{}.json", payload.index))
    {
        return Err(NativeServiceError::Refused(
            "invalid original worker coordinates",
        ));
    }
    let invocation = std::env::var("INVOCATION_ID").map_err(|_| {
        NativeServiceError::Refused("native worker requires its original systemd invocation")
    })?;
    if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(NativeServiceError::Refused(
            "invalid native worker invocation",
        ));
    }
    let error = Command::new(std::env::current_exe()?)
        .arg("worker")
        .arg(path)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("HOME", payload.state.join("home"))
        .env("XDG_CACHE_HOME", payload.state.join("cache"))
        .env("TMPDIR", payload.state.join("tmp"))
        .env("INVOCATION_ID", invocation)
        .exec();
    Err(error.into())
}

/// Application worker helper: one private control ingress and one real D connection.
/// TERM closes ingress, physically drains the original connection and only then exits.
/// # Errors
/// Private startup, canonical readiness, control ingress, or owned connection settlement fails.
pub async fn run_worker(path: &Path) -> Result<(), NativeServiceError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let payload: WorkerPayload = private_read(path)?;
    private_path(&payload.state, true)?;
    verify_run(&payload.state, &payload.run)?;
    let identity: DisposableIdentity = private_read(&payload.state.join("disposable.json"))?;
    if payload.version != 1
        || identity.version != 1
        || identity.nonce != payload.nonce
        || path != payload.run.join(format!("worker-{}.json", payload.index))
    {
        return Err(NativeServiceError::Refused(
            "invalid original worker coordinates",
        ));
    }
    let invocation = std::env::var("INVOCATION_ID").map_err(|_| {
        NativeServiceError::Refused("native worker requires its original systemd invocation")
    })?;
    if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(NativeServiceError::Refused(
            "invalid native worker invocation",
        ));
    }
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let options = CanonicalOptions::from_state(&payload.state)?;
    let connection = match payload.initial_schema {
        InitialSchema::Current => {
            WorkerConnection::Current(CanonicalStore::connect_ready(&options).await?)
        }
        InitialSchema::PreEpoch | InitialSchema::SchemaV2 => {
            let connection = SchemaMaintenance::connect(&options).await?;
            let inspected = connection.inspect().await;
            if let Err(error) = inspected {
                let drained = connection.disconnect().await;
                return match drained {
                    Ok(()) => Err(error.into()),
                    Err(cleanup) => Err(NativeServiceError::Settlement {
                        state: payload.state,
                        failures: vec![error.into(), cleanup.into()],
                    }),
                };
            }
            WorkerConnection::PreEpoch(connection)
        }
    };
    let socket = payload.run.join(format!("worker-{}.sock", payload.index));
    let listener = match tokio::net::UnixListener::bind(&socket) {
        Ok(listener) => listener,
        Err(error) => {
            let drained = connection.disconnect().await;
            return match drained {
                Ok(()) => Err(error.into()),
                Err(cleanup) => Err(NativeServiceError::Settlement {
                    state: payload.state,
                    failures: vec![error.into(), cleanup.into()],
                }),
            };
        }
    };
    let permissions = fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))
        .map_err(NativeServiceError::from);
    let claim = StartupClaim {
        nonce: payload.nonce,
        pid: std::process::id(),
        invocation,
    };
    let claimed = permissions.and_then(|()| {
        private_write(
            &payload
                .run
                .join(format!("worker-{}.ready.json", payload.index)),
            &claim,
        )
    });
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
    let drain = connection
        .disconnect()
        .await
        .map_err(NativeServiceError::from);
    if drain.is_ok() {
        private_write(
            &payload
                .run
                .join(format!("worker-{}.drained.json", payload.index)),
            &claim,
        )?;
    }
    match (run, drain) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(NativeServiceError::Settlement {
            state: payload.state,
            failures: vec![error, cleanup],
        }),
    }
}

fn canonical_drain_failure(error: NativeServiceError) -> CanonicalError {
    match error {
        NativeServiceError::Canonical(error) => error,
        NativeServiceError::Settlement { failures, .. } => CanonicalError::MigrationDrain {
            failures: failures.into_iter().map(canonical_drain_failure).collect(),
        },
        error => CanonicalError::Configuration(error.to_string()),
    }
}

enum WorkerConnection {
    Current(SchemaReady),
    PreEpoch(SchemaMaintenance),
}
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
        match self {
            Self::Current(ready) => {
                ready
                    .store()
                    .drain_owned_connection_until(crate::transport::original_deadline(
                        Duration::from_secs(30),
                    ))
                    .await
            }
            Self::PreEpoch(maintenance) => maintenance.disconnect().await,
        }
    }
}

async fn stop_ingress_owners(
    manager: Option<&HostManager>,
    state: &OwnedState,
    run: &Path,
    ingresses: &mut [OwnedIngress],
    deadline: Instant,
) -> Result<(), NativeServiceError> {
    let results =
        futures_util::future::join_all(ingresses.iter_mut().enumerate().map(|(index, ingress)| {
            async move {
                tokio::time::timeout_at(deadline.into(), async {
                    if ingress.settled {
                        return Ok(());
                    }
                    if ingress.drained.is_none() {
                        if ingress.unit.is_none() {
                            ingress.unit = manager
                                .ok_or(NativeServiceError::Refused(
                                    "original ingress manager unavailable",
                                ))?
                                .reconcile_if_present(&ingress.intent, deadline)
                                .await?;
                        }
                        let unit = ingress.unit.as_mut().ok_or(NativeServiceError::Refused(
                            "original ingress launch remains uncertain",
                        ))?;
                        // stop_and_drain includes relinquishing the original client reference.
                        // Retain its terminal proof before any fallible receipt read.
                        ingress.drained = Some(unit.stop_and_drain(deadline).await?);
                    }
                    let original = ingress.drained.as_ref().ok_or(NativeServiceError::Refused(
                        "original ingress terminal observation unavailable",
                    ))?;
                    if ingress.ready {
                        let drained: StartupClaim =
                            private_read(&run.join(format!("worker-{index}.drained.json")))?;
                        let started: StartupClaim =
                            private_read(&run.join(format!("worker-{index}.ready.json")))?;
                        if drained != started
                            || drained.nonce != state.nonce
                            || drained.invocation != hex_bytes(&original.invocation)
                        {
                            return Err(NativeServiceError::Refused(
                                "original ingress connection drain was not established",
                            ));
                        }
                    }
                    ingress.settled = true;
                    Ok::<(), NativeServiceError>(())
                })
                .await
                .map_err(|_| {
                    NativeServiceError::Refused("original ingress settlement deadline expired")
                })?
            }
        }))
        .await;
    let failures: Vec<_> = results.into_iter().filter_map(Result::err).collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(NativeServiceError::Settlement {
            state: state.path.clone(),
            failures,
        })
    }
}

async fn stop_original_process(
    manager: Option<&HostManager>,
    intent: &LaunchIntent,
    unit: &mut Option<OwnedUnit>,
    dispatched: bool,
    drained: &mut Option<UnitObservation>,
    deadline: Instant,
) -> Result<Option<UnitObservation>, NativeServiceError> {
    if !dispatched {
        return Ok(None);
    }
    if let Some(observed) = drained {
        return Ok(Some(observed.clone()));
    }
    if unit.is_none() && dispatched {
        let manager = manager.ok_or(NativeServiceError::Refused(
            "original manager is unavailable",
        ))?;
        *unit = manager.reconcile_if_present(intent, deadline).await?;
        if unit.is_none() {
            return Err(NativeServiceError::Refused(
                "original launch remains uncertain; no physical release was established",
            ));
        }
    }
    let original = unit
        .as_mut()
        .ok_or(NativeServiceError::Refused("service was never launched"))?;
    let observed = original.stop_and_drain(deadline).await?;
    *drained = Some(observed.clone());
    Ok(Some(observed))
}

async fn host_observation<T>(
    deadline: Instant,
    future: impl Future<Output = Result<T, HostError>>,
) -> Result<T, NativeServiceError> {
    tokio::time::timeout_at(deadline.into(), future)
        .await
        .map_err(|_| NativeServiceError::Refused("original native observation deadline expired"))?
        .map_err(NativeServiceError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn private_state() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        directory
    }
    #[test]
    fn native_service_bootstrap_failure_hides_sql_and_preserves_typed_finality() {
        let first = "abcdef0123456789".repeat(4);
        let second = "9876543210fedcba".repeat(4);
        for detail in [Some(surrealdb_types::QueryError::NotExecuted), None] {
            let original = CanonicalError::from(surrealdb_types::Error::query(
                format!("PASSWORD '{first}' account '{second}'"),
                detail.clone(),
            ));
            let unknown = original.effect_is_unknown();
            let infrastructure = original.is_infrastructure();
            let failure = ViewerBootstrapFailure { original };
            assert_eq!(failure.effect_is_unknown(), unknown);
            assert_eq!(failure.is_infrastructure(), infrastructure);
            assert_eq!(failure.query_failure(), detail);
            assert!(failure.native_code().is_some());
            let outer = NativeServiceError::Bootstrap(failure);
            for rendered in [
                format!("{outer:?}"),
                outer.to_string(),
                std::error::Error::source(&outer).unwrap().to_string(),
            ] {
                assert!(!rendered.contains(&first));
                assert!(!rendered.contains(&second));
                assert!(!rendered.contains("PASSWORD"));
            }
            assert!(
                std::error::Error::source(std::error::Error::source(&outer).unwrap()).is_none()
            );
        }
    }
    fn unlaunched_service() -> (tempfile::TempDir, NativeCanonicalService) {
        let temporary = private_state();
        let path = temporary.path().to_path_buf();
        private_write(
            &path.join("disposable.json"),
            &DisposableIdentity {
                version: 1,
                nonce: "original".into(),
            },
        )
        .unwrap();
        let metadata = fs::metadata(&path).unwrap();
        let state = OwnedState {
            path,
            device: metadata.dev(),
            inode: metadata.ino(),
            nonce: "original".into(),
        };
        for name in [
            "config.json",
            "credentials.json",
            "original-module-receipt.json",
        ] {
            private_write(&state.path.join(name), &serde_json::json!({})).unwrap();
        }
        let records = RetainedRecords::read(&state.path).unwrap();
        let run = state.path.join("runs").join("original");
        private_directory(&state.path.join("runs")).unwrap();
        private_directory(&run).unwrap();
        let service = NativeCanonicalService {
            state,
            records,
            server_generation: PathBuf::from("/unprepared-unit-test-generation"),
            server_recipe: "unit-test".into(),
            module_root: PathBuf::from("/unprepared-unit-test-modules"),
            run,
            reopening: false,
            initial_schema: InitialSchema::Current,
            initialized: false,
            helper: PathBuf::from("/usr/bin/false"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            intent: LaunchIntent::fresh().unwrap(),
            manager: None,
            unit: None,
            launch_dispatched: false,
            ready: None,
            connections: Vec::new(),
            maintenances: Vec::new(),
            #[cfg(feature = "test-support")]
            schema_writes: Vec::new(),
            ingresses: Vec::new(),
            process_drained: None,
            closing: false,
            stopped: false,
            disposed: false,
            panic_settlement: None,
        };
        (temporary, service)
    }
    #[tokio::test]
    async fn native_service_shutdown_before_launch_is_terminal_and_disposal_is_idempotent() {
        let (_temporary, mut service) = unlaunched_service();
        service
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        assert!(matches!(
            service
                .launch(Instant::now() + Duration::from_secs(1))
                .await,
            Err(NativeServiceError::Refused(
                "original native service admission is closed"
            ))
        ));
        assert!(service.unit.is_none());
        assert!(!service.launch_dispatched);
        service.dispose().unwrap();
        service.dispose().unwrap();
        assert!(!service.state().exists());
        assert!(matches!(
            service
                .launch(Instant::now() + Duration::from_secs(1))
                .await,
            Err(NativeServiceError::Refused(
                "original native service admission is closed"
            ))
        ));
    }
    #[tokio::test]
    async fn native_service_failed_dispose_retains_owner_for_explicit_retry() {
        let (_temporary, mut service) = unlaunched_service();
        let original = service.intent.clone();
        assert!(service.dispose().is_err());
        service
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        fs::set_permissions(service.state(), fs::Permissions::from_mode(0o000)).unwrap();
        let failed = service.dispose();
        fs::set_permissions(service.state(), fs::Permissions::from_mode(0o700)).unwrap();
        assert!(failed.is_err());
        assert!(!service.disposed);
        assert_eq!(service.intent.unit(), original.unit());
        service.state.verify().unwrap();
        service.dispose().unwrap();
    }
    #[tokio::test]
    async fn native_service_failed_stopped_transfer_returns_original_owner() {
        let (_temporary, service) = unlaunched_service();
        let original = service.intent().unit().to_owned();
        let path = service.state().to_path_buf();
        let failed = service.try_into_stopped().unwrap_err();
        let (mut service, _) = failed.into_parts();
        assert_eq!(service.intent().unit(), original);
        assert_eq!(service.state(), path);
        service
            .shutdown(Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        // A shutdown of an unlaunched fixture cannot authorize retaining a database.
        let (mut service, _) = service.try_into_stopped().unwrap_err().into_parts();
        service.dispose().unwrap();
        assert!(!path.exists());
    }
    #[test]
    fn native_service_retained_records_refuse_changed_endpoint_and_credentials() {
        let (_temporary, service) = unlaunched_service();
        service.records.verify(service.state()).unwrap();
        for name in [
            "config.json",
            "credentials.json",
            "original-module-receipt.json",
        ] {
            let path = service.state().join(name);
            fs::remove_file(&path).unwrap();
            private_write(&path, &serde_json::json!({"changed": true})).unwrap();
            assert!(service.records.verify(service.state()).is_err(), "{name}");
            fs::remove_file(&path).unwrap();
            private_write(&path, &serde_json::json!({})).unwrap();
        }
    }
    #[test]
    fn native_service_prepared_config_variants_preserve_endpoint_and_credentials() {
        let (_temporary, service) = unlaunched_service();
        let path = service.state().join("config.json");
        fs::remove_file(&path).unwrap();
        let mut configuration = serde_json::json!({
            "endpoint":"grpc://original", "credentials_file":"original-private-credentials",
            "study_module":{"sha256":"old"}, "schema_migration_predecessor":{"sha256":"old"},
            "schema_migration_target":{"sha256":"new"},
        });
        private_write(&path, &configuration).unwrap();
        private_write(
            &service.state().join("target-module-source-receipt.json"),
            &serde_json::json!({}),
        )
        .unwrap();
        let records = RetainedRecords::read(service.state()).unwrap();
        configuration["study_module"] = configuration["schema_migration_target"].clone();
        fs::remove_file(&path).unwrap();
        private_write(&path, &configuration).unwrap();
        records.verify(service.state()).unwrap();
        for (field, value) in [
            ("endpoint", serde_json::json!("grpc://foreign")),
            ("credentials_file", serde_json::json!("foreign-credentials")),
            ("study_module", serde_json::json!({"sha256":"third"})),
        ] {
            let mut changed = configuration.clone();
            changed[field] = value;
            fs::remove_file(&path).unwrap();
            private_write(&path, &changed).unwrap();
            assert!(records.verify(service.state()).is_err(), "{field}");
        }
    }
    #[test]
    fn native_service_failed_reopen_retains_stopped_directory_for_disposal() {
        let (_temporary, mut service) = unlaunched_service();
        let path = service.state().to_path_buf();
        service.closing = true;
        service.stopped = true;
        // Only this internal negative control can create a malformed stopped capability.
        // No startup provenance exists, so reopening must return that same owner.
        let stopped = StoppedCanonicalState { service };
        let failed = stopped.reopen().unwrap_err();
        let (mut stopped, _) = failed.into_parts();
        assert_eq!(stopped.state(), path);
        assert!(path.exists());
        stopped.dispose().unwrap();
        assert!(!path.exists());
    }
    #[tokio::test]
    async fn native_service_host_observation_preserves_expired_original_deadline() {
        let original = Instant::now();
        for _ in 0..2 {
            let result = host_observation::<()>(original, std::future::pending()).await;
            assert!(matches!(
                result,
                Err(NativeServiceError::Refused(
                    "original native observation deadline expired"
                ))
            ));
        }
    }
    #[test]
    fn native_service_private_payload_refuses_symlinks_hardlinks_and_shared_modes() {
        let state = private_state();
        let path = state.path().join("private.json");
        private_write(
            &path,
            &DisposableIdentity {
                version: 1,
                nonce: "identity".into(),
            },
        )
        .unwrap();
        assert_eq!(
            private_read::<DisposableIdentity>(&path).unwrap().nonce,
            "identity"
        );
        assert!(private_write(&path, &"overwrite").is_err());
        let alias = state.path().join("alias.json");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(private_read::<DisposableIdentity>(&alias).is_err());
        fs::remove_file(alias).unwrap();
        fs::hard_link(&path, state.path().join("hardlink.json")).unwrap();
        assert!(private_read::<DisposableIdentity>(&path).is_err());
        fs::remove_file(state.path().join("hardlink.json")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_read::<DisposableIdentity>(&path).is_err());
    }
    #[test]
    fn native_service_kernel_listener_proof_refuses_foreign_owner_and_unbound_endpoint() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let bind = listener.local_addr().unwrap();
        assert!(owns_listener(std::process::id(), bind).unwrap());
        assert!(
            !owns_listener(
                std::process::id(),
                SocketAddr::from((Ipv4Addr::LOCALHOST, 0))
            )
            .unwrap()
        );
        let mut child = Command::new("/usr/bin/sleep").arg("5").spawn().unwrap();
        let observed = owns_listener(child.id(), bind);
        let killed = child.kill();
        let waited = child.wait();
        killed.unwrap();
        waited.unwrap();
        assert!(!observed.unwrap());
    }
    #[test]
    fn native_service_replaced_disposable_identity_cannot_authorize_deletion() {
        let state = private_state();
        let path = state.path().to_path_buf();
        private_write(
            &path.join("disposable.json"),
            &DisposableIdentity {
                version: 1,
                nonce: "original".into(),
            },
        )
        .unwrap();
        let metadata = fs::metadata(&path).unwrap();
        let owned = OwnedState {
            path: path.clone(),
            device: metadata.dev(),
            inode: metadata.ino(),
            nonce: "original".into(),
        };
        owned.verify().unwrap();
        fs::remove_file(path.join("disposable.json")).unwrap();
        private_write(
            &path.join("disposable.json"),
            &DisposableIdentity {
                version: 1,
                nonce: "replacement".into(),
            },
        )
        .unwrap();
        assert!(owned.verify().is_err());
    }
}
