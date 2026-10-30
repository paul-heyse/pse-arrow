// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit persistent Collector/OpenObserve operator. Inspection never prepares or starts.
//!
//! Installed units have private immutable generations and a separately authenticated manager
//! capability. They are not transient workloads and acquire no `OwnedUnit` references.
use crate::{
    HostError, HostManager,
    diagnostic_tools::{self, Tool, digest, no_symlinks},
    environment::Environment,
    manager::{InstalledServiceSpec, InstalledServiceState},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
const OWNER: &str = "pse-telemetry-v1";
const TOOLS: [&str; 2] = ["collector", "openobserve"];
const ALERT_CIDR: &str = "127.0.0.1/32";
fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}
fn declaration(name: &str) -> Tool {
    if name == "collector" {
        Tool::Collector
    } else {
        Tool::OpenObserve
    }
}
fn nonce() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}
fn valid_nonce(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn text_path(path: &Path) -> Result<&str, HostError> {
    path.to_str()
        .filter(|value| !value.contains(['\0', '\r', '\n']))
        .ok_or_else(|| invalid("telemetry path must be UTF-8 without control characters"))
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphabetic() || byte == b'_' || index > 0 && byte.is_ascii_digit()
        })
}
fn default_ports() -> BTreeMap<String, u16> {
    [
        ("otlp_http", 4318),
        ("otlp_grpc", 4317),
        ("backend_http", 5080),
        ("backend_grpc", 5081),
        ("metrics", 8888),
        ("health", 13133),
    ]
    .into_iter()
    .map(|(name, port)| (name.into(), port))
    .collect()
}
fn default_streams() -> BTreeMap<String, String> {
    [
        ("traces", "pse_traces"),
        ("metrics", "pse_metrics"),
        ("logs", "pse_logs"),
    ]
    .into_iter()
    .map(|(name, value)| (name.into(), value.into()))
    .collect()
}
fn limits() -> Value {
    json!({"queue_requests_per_signal":1000,"retry_elapsed_seconds":300,"batch_items":512,"batch_max_items":1024,"ingress_body_bytes":4 << 20,"stop_seconds":30})
}
/// Setup selection. Missing entries in `ports` use the established local listener defaults.
#[derive(Clone, Debug)]
pub struct SetupOptions {
    /// Explicit listener overrides; names are the existing local listener keys.
    pub ports: BTreeMap<String, u16>,
    /// Local backend organization identifier.
    pub organization: String,
    /// Exactly the traces, metrics and logs stream names.
    pub streams: BTreeMap<String, String>,
    /// Backend retention, at least three days.
    pub retention_days: u32,
}
impl Default for SetupOptions {
    fn default() -> Self {
        Self {
            ports: BTreeMap::new(),
            organization: "default".into(),
            streams: default_streams(),
            retention_days: 7,
        }
    }
}
impl SetupOptions {
    fn validate(&self) -> Result<BTreeMap<String, u16>, HostError> {
        let mut ports = default_ports();
        for (name, value) in &self.ports {
            if !ports.contains_key(name) {
                return Err(invalid("unknown telemetry listener selection"));
            }
            ports.insert(name.clone(), *value);
        }
        if ports.values().any(|value| *value == 0)
            || ports.values().collect::<BTreeSet<_>>().len() != ports.len()
        {
            return Err(invalid(
                "telemetry ports must be distinct integers from 1 through 65535",
            ));
        }
        if !identifier(&self.organization)
            || self.streams.keys().collect::<BTreeSet<_>>()
                != default_streams().keys().collect::<BTreeSet<_>>()
            || self.streams.values().any(|value| !identifier(value))
        {
            return Err(invalid(
                "telemetry organization and three streams require safe identifiers",
            ));
        }
        if self.retention_days < 3 {
            return Err(invalid("OpenObserve retention must be at least three days"));
        }
        Ok(ports)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    size: u64,
    mtime_ns: i128,
    ctime_ns: i128,
    mode: u32,
    uid: u32,
}
impl FileIdentity {
    fn read(path: &Path) -> Result<Self, HostError> {
        no_symlinks(path)?;
        let info = fs::metadata(path)?;
        if !info.is_file() {
            return Err(invalid("telemetry executable must be a regular file"));
        }
        Ok(Self {
            device: info.dev(),
            inode: info.ino(),
            size: info.size(),
            mtime_ns: i128::from(info.mtime()) * 1_000_000_000 + i128::from(info.mtime_nsec()),
            ctime_ns: i128::from(info.ctime()) * 1_000_000_000 + i128::from(info.ctime_nsec()),
            mode: info.mode(),
            uid: info.uid(),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Binary {
    path: PathBuf,
    sha256: String,
    identity: FileIdentity,
    version: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct PublicConfiguration {
    schema_version: u32,
    configuration_id: String,
    otlp_http_endpoint: String,
    otlp_grpc_endpoint: String,
    backend_http_endpoint: String,
    organization: String,
    streams: BTreeMap<String, String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Configuration {
    schema_version: u32,
    owner: String,
    owner_id: String,
    state: PathBuf,
    configuration_id: String,
    generation: PathBuf,
    units: BTreeMap<String, String>,
    ports: BTreeMap<String, u16>,
    binaries: BTreeMap<String, Binary>,
    files: BTreeMap<String, String>,
    retention_days: u32,
    public_configuration: PublicConfiguration,
    limits: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    environment_file: Option<PathBuf>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
impl Configuration {
    fn env_file(&self) -> PathBuf {
        self.environment_file
            .clone()
            .unwrap_or_else(|| self.state.join("service.env"))
    }
    fn spec(&self, name: &str, directory: &Path) -> Result<InstalledServiceSpec, HostError> {
        let unit = self
            .units
            .get(name)
            .ok_or_else(|| invalid("telemetry unit missing"))?;
        let binary = self
            .binaries
            .get(name)
            .ok_or_else(|| invalid("telemetry binary missing"))?;
        let mut arguments = vec![text_path(&binary.path)?.to_owned()];
        if name == "collector" {
            arguments.push(format!(
                "--config={}",
                text_path(&self.generation.join("collector.yaml"))?
            ));
        }
        Ok(InstalledServiceSpec {
            name: unit.clone(),
            description: format!("PSE telemetry {name} {}", self.owner_id),
            fragment: directory.join(unit),
            fragment_sha256: digest(&self.generation.join(unit))?,
            executable: binary.path.clone(),
            executable_identity: (binary.identity.device, binary.identity.inode),
            arguments,
            environment_file: self.env_file(),
            working_directory: self.state.clone(),
        })
    }
}
/// Resolve state without creating it or following aliases.
pub fn state_directory(
    state: Option<&Path>,
    environment: &Environment,
) -> Result<PathBuf, HostError> {
    let path = match state {
        Some(path) => path.to_path_buf(),
        None => crate::paths::HostPaths::new(environment).telemetry()?,
    };
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };
    no_symlinks(&path)?;
    text_path(&path)?;
    Ok(path)
}
fn unit_directory(environment: &Environment) -> Result<PathBuf, HostError> {
    let path = crate::paths::HostPaths::new(environment).user_units()?;
    no_symlinks(&path)?;
    Ok(path)
}
fn private_directory(path: &Path, create: bool) -> Result<(), HostError> {
    no_symlinks(path)?;
    if create {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
    }
    let info = fs::metadata(path)?;
    if !info.is_dir()
        || info.uid() != rustix::process::geteuid().as_raw()
        || info.mode() & 0o077 != 0
    {
        return Err(invalid(
            "telemetry state must be an owned private directory (0700)",
        ));
    }
    Ok(())
}
fn private_file(path: &Path) -> Result<File, HostError> {
    no_symlinks(path)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let info = file.metadata()?;
    if !info.is_file()
        || info.uid() != rustix::process::geteuid().as_raw()
        || info.mode() & 0o077 != 0
    {
        return Err(invalid(
            "telemetry configuration permissions or ownership changed",
        ));
    }
    Ok(file)
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, HostError> {
    let mut bytes = Vec::new();
    private_file(path)?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(invalid("telemetry configuration exceeds 64 KiB"));
    }
    serde_json::from_slice(&bytes).map_err(|_| invalid("invalid telemetry configuration"))
}
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), HostError> {
    no_symlinks(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| invalid("telemetry publication parent missing"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary
        .as_file()
        .set_permissions(fs::Permissions::from_mode(0o600))?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| HostError::Io(error.error))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), HostError> {
    let mut bytes =
        serde_json::to_vec(value).map_err(|_| invalid("cannot encode telemetry configuration"))?;
    bytes.push(b'\n');
    write_atomic(path, &bytes)
}
struct Lock(File);
impl Lock {
    fn acquire(state: &Path) -> Result<Self, HostError> {
        private_directory(state, false)?;
        let path = state.join(".lifecycle.lock");
        no_symlinks(&path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(&path)?;
        let info = file.metadata()?;
        if !info.is_file()
            || info.uid() != rustix::process::geteuid().as_raw()
            || info.mode() & 0o077 != 0
        {
            return Err(invalid("telemetry lifecycle lock ownership differs"));
        }
        rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive)
            .map_err(|_| invalid("another telemetry lifecycle operation is in progress"))?;
        Ok(Self(file))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = rustix::fs::flock(&self.0, rustix::fs::FlockOperation::Unlock);
    }
}
/// Preserve Collector-owned queue admission, bounded retries and cumulative failure metrics.
pub fn collector_configuration(
    state: &Path,
    ports: &BTreeMap<String, u16>,
    organization: &str,
    streams: &BTreeMap<String, String>,
    configuration_id: &str,
) -> Result<Value, HostError> {
    let selected = SetupOptions {
        ports: ports.clone(),
        organization: organization.into(),
        streams: streams.clone(),
        retention_days: 7,
    }
    .validate()?;
    let mut exporters = serde_json::Map::new();
    let mut pipelines = serde_json::Map::new();
    for (signal, stream) in streams {
        exporters.insert(format!("otlp_http/{signal}"),json!({"endpoint":format!("http://127.0.0.1:{}/api/{organization}",selected["backend_http"]),"headers":{"Authorization":"${env:PSE_TELEMETRY_AUTHORIZATION}","stream-name":stream},"encoding":"json","compression":"gzip","timeout":"10s","retry_on_failure":{"enabled":true,"initial_interval":"1s","max_interval":"30s","max_elapsed_time":"300s"},"sending_queue":{"enabled":true,"sizer":"requests","queue_size":1000,"num_consumers":2,"block_on_overflow":false,"storage":"file_storage"}}));
        pipelines.insert(signal.clone(),json!({"receivers":["otlp"],"processors":["batch"],"exporters":[format!("otlp_http/{signal}")]}));
    }
    Ok(
        json!({"receivers":{"otlp":{"protocols":{"http":{"endpoint":format!("127.0.0.1:{}",selected["otlp_http"]),"max_request_body_size":4<<20},"grpc":{"endpoint":format!("127.0.0.1:{}",selected["otlp_grpc"]),"max_recv_msg_size_mib":4}}}},"processors":{"batch":{"timeout":"1s","send_batch_size":512,"send_batch_max_size":1024}},"exporters":exporters,"extensions":{"file_storage":{"directory":state.join("collector-queue"),"timeout":"10s","fsync":true},"health_check":{"endpoint":format!("127.0.0.1:{}",selected["health"]) }},"service":{"extensions":["file_storage","health_check"],"telemetry":{"resource":{"attributes":[{"name":"service.name","value":"pse-telemetry-collector"},{"name":"service.instance.id","value":"${env:INVOCATION_ID}"},{"name":"pse.configuration.id","value":configuration_id}]},"logs":{"level":"info"},"metrics":{"level":"normal","readers":[{"periodic":{"interval":10000,"exporter":{"otlp":{"protocol":"grpc","endpoint":format!("http://127.0.0.1:{}",selected["otlp_grpc"])}}}},{"pull":{"exporter":{"prometheus":{"host":"127.0.0.1","port":selected["metrics"]}}}}]}},"pipelines":pipelines}}),
    )
}
fn unit_quote(value: &str) -> Result<String, HostError> {
    if value.contains(['\0', '\r', '\n']) {
        return Err(invalid("invalid service path"));
    }
    Ok(format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('%', "%%")
            .replace('$', "$$")
    ))
}
fn unit_text(
    name: &str,
    binary: &Path,
    generation: &Path,
    state: &Path,
    environment_file: &Path,
    owner_id: &str,
) -> Result<String, HostError> {
    let mut command = unit_quote(text_path(binary)?)?;
    if name == "collector" {
        command.push_str(" --config=");
        command.push_str(&unit_quote(text_path(&generation.join("collector.yaml"))?)?);
    }
    Ok(format!(
        "[Unit]\nDescription=PSE telemetry {name} {owner_id}\n\n[Service]\nType=exec\nWorkingDirectory={}\nEnvironmentFile={}\nExecStart={command}\nRestart=on-failure\nRestartSec=2s\nTimeoutStopSec=30s\nKillMode=control-group\nUMask=0077\nNoNewPrivileges=yes\n\n[Install]\nWantedBy=default.target\n",
        text_path(state)?.replace('%', "%%"),
        text_path(environment_file)?.replace('%', "%%")
    ))
}
fn environment_values(path: &Path) -> Result<BTreeMap<String, String>, HostError> {
    let mut source = String::new();
    private_file(path)?
        .take(65537)
        .read_to_string(&mut source)?;
    if source.len() > 65536 {
        return Err(invalid("telemetry environment exceeds allowance"));
    }
    let mut values = BTreeMap::new();
    for line in source.lines() {
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| invalid("invalid telemetry environment"))?;
        let value: String = serde_json::from_str(value)
            .map_err(|_| invalid("invalid telemetry environment value"))?;
        if !identifier(name)
            || value.contains(['\0', '\r', '\n'])
            || values.insert(name.into(), value).is_some()
        {
            return Err(invalid("ambiguous telemetry environment"));
        }
    }
    if !values
        .get("PSE_TELEMETRY_AUTHORIZATION")
        .is_some_and(|value| value.starts_with("Basic "))
    {
        return Err(invalid("telemetry authorization is unavailable"));
    }
    Ok(values)
}
fn write_environment(path: &Path, values: &BTreeMap<String, String>) -> Result<(), HostError> {
    let mut source = String::new();
    for (name, value) in values {
        source.push_str(name);
        source.push('=');
        source.push_str(
            &serde_json::to_string(value)
                .map_err(|_| invalid("cannot encode telemetry environment"))?,
        );
        source.push('\n');
    }
    write_atomic(path, source.as_bytes())
}
fn checked_configuration(state: &Path) -> Result<Configuration, HostError> {
    let config: Configuration = read_json(&state.join("service.json"))?;
    let public: PublicConfiguration = read_json(&state.join("configuration.json"))?;
    validate_configuration(state, &config, &public)?;
    Ok(config)
}
fn validate_configuration(
    state: &Path,
    config: &Configuration,
    public: &PublicConfiguration,
) -> Result<(), HostError> {
    private_directory(state, false)?;
    if config.schema_version != 1
        || config.owner != OWNER
        || config.state != state
        || !valid_nonce(&config.owner_id)
        || !valid_nonce(&config.configuration_id)
        || config.generation != state.join("generations").join(&config.configuration_id)
    {
        return Err(invalid("selected telemetry state ownership differs"));
    }
    let options = SetupOptions {
        ports: config.ports.clone(),
        organization: public.organization.clone(),
        streams: public.streams.clone(),
        retention_days: config.retention_days,
    };
    let ports = options.validate()?;
    if config.ports != ports
        || public != &config.public_configuration
        || public.schema_version != 1
        || public.configuration_id != config.configuration_id
        || public.otlp_http_endpoint != format!("http://127.0.0.1:{}", ports["otlp_http"])
        || public.otlp_grpc_endpoint != format!("http://127.0.0.1:{}", ports["otlp_grpc"])
        || public.backend_http_endpoint != format!("http://127.0.0.1:{}", ports["backend_http"])
    {
        return Err(invalid("telemetry public listener identity differs"));
    }
    private_directory(&config.generation, false)?;
    let env_file = config.env_file();
    if env_file != state.join("service.env") && env_file != config.generation.join("service.env") {
        return Err(invalid("telemetry environment generation differs"));
    }
    let mut required = BTreeSet::new();
    required.insert(
        text_path(
            env_file
                .strip_prefix(state)
                .map_err(|_| invalid("telemetry environment escapes state"))?,
        )?
        .to_owned(),
    );
    required.insert(format!(
        "generations/{}/collector.yaml",
        config.configuration_id
    ));
    if config.units.len() != 2 || config.binaries.len() != 2 {
        return Err(invalid("telemetry unit/binary inventory differs"));
    }
    for name in TOOLS {
        let unit = format!(
            "pse-telemetry-{}-{}-{name}.service",
            config.owner_id, config.configuration_id
        );
        if config.units.get(name) != Some(&unit) {
            return Err(invalid("telemetry unit identity differs"));
        }
        required.insert(format!("generations/{}/{unit}", config.configuration_id));
        let binary = config
            .binaries
            .get(name)
            .ok_or_else(|| invalid("telemetry binary missing"))?;
        if FileIdentity::read(&binary.path)? != binary.identity || binary.identity.mode & 0o111 == 0
        {
            return Err(invalid("telemetry binary installation changed"));
        }
        let expected = unit_text(
            name,
            &binary.path,
            &config.generation,
            state,
            &env_file,
            &config.owner_id,
        )?;
        if fs::read(config.generation.join(&unit))? != expected.as_bytes() {
            return Err(invalid("telemetry unit declaration differs"));
        }
    }
    if config.files.keys().cloned().collect::<BTreeSet<_>>() != required {
        return Err(invalid("telemetry generation file inventory differs"));
    }
    for (name, expected) in &config.files {
        let path = state.join(name);
        private_file(&path)?;
        if digest(&path)? != *expected {
            return Err(invalid("telemetry generation files changed"));
        }
    }
    environment_values(&env_file)?;
    Ok(())
}
async fn validate_collector(
    binary: &Path,
    configuration: &Path,
    environment: &Environment,
    values: &BTreeMap<String, String>,
) -> Result<(), HostError> {
    let mut child = tokio::process::Command::new(binary)
        .args(["validate", "--config"])
        .arg(configuration)
        .env_clear()
        .envs(environment)
        .envs(values)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    match tokio::time::timeout(Duration::from_secs(30), child.wait()).await {
        Ok(Ok(status)) if status.success() => Ok(()),
        Ok(Ok(_)) => Err(invalid(
            "Collector rejected configuration; generation was not selected",
        )),
        Ok(Err(error)) => Err(error.into()),
        Err(_) => {
            let killed = child.kill().await;
            let drained = child.wait().await;
            if killed.is_err() || drained.is_err() {
                return Err(invalid(
                    "Collector validation timed out and settlement failed",
                ));
            }
            Err(HostError::Deadline(
                "Collector configuration validation".into(),
            ))
        }
    }
}
async fn generation(
    config: &mut Configuration,
    environment: &Environment,
    values: &BTreeMap<String, String>,
) -> Result<(), HostError> {
    private_directory(&config.generation, true)?;
    write_environment(&config.env_file(), values)?;
    write_json(
        &config.generation.join("collector.yaml"),
        &collector_configuration(
            &config.state,
            &config.ports,
            &config.public_configuration.organization,
            &config.public_configuration.streams,
            &config.configuration_id,
        )?,
    )?;
    validate_collector(
        &config.binaries["collector"].path,
        &config.generation.join("collector.yaml"),
        environment,
        values,
    )
    .await?;
    config.units.clear();
    config.files.clear();
    let mut files = vec![config.generation.join("collector.yaml"), config.env_file()];
    for name in TOOLS {
        let unit = format!(
            "pse-telemetry-{}-{}-{name}.service",
            config.owner_id, config.configuration_id
        );
        let path = config.generation.join(&unit);
        write_atomic(
            &path,
            unit_text(
                name,
                &config.binaries[name].path,
                &config.generation,
                &config.state,
                &config.env_file(),
                &config.owner_id,
            )?
            .as_bytes(),
        )?;
        files.push(path);
        config.units.insert(name.into(), unit);
    }
    for path in files {
        config.files.insert(
            text_path(
                path.strip_prefix(&config.state)
                    .map_err(|_| invalid("telemetry generation escapes state"))?,
            )?
            .to_owned(),
            digest(&path)?,
        );
    }
    Ok(())
}
/// Explicit setup validates a private generation and publishes producer selection; no service starts.
pub async fn setup(
    root: &Path,
    environment: &Environment,
    state: Option<&Path>,
    options: &SetupOptions,
) -> Result<Value, HostError> {
    let ports = options.validate()?;
    let state = state_directory(state, environment)?;
    private_directory(&state, true)?;
    let _lock = Lock::acquire(&state)?;
    if fs::read_dir(&state)?.any(|entry| {
        entry.map_or(true, |entry| {
            entry.file_name() != OsStr::new(".lifecycle.lock")
        })
    }) {
        return Err(invalid(
            "telemetry setup requires an empty owned state; inspect or select a new state",
        ));
    }
    let mut binaries = BTreeMap::new();
    for name in TOOLS {
        let tool = declaration(name);
        let path = diagnostic_tools::verify(root, tool, environment)?;
        binaries.insert(
            name.into(),
            Binary {
                identity: FileIdentity::read(&path)?,
                sha256: digest(&path)?,
                version: diagnostic_tools::tool_spec(root, tool)?.version,
                path,
            },
        );
    }
    let configuration_id = nonce();
    let public = PublicConfiguration {
        schema_version: 1,
        configuration_id: configuration_id.clone(),
        otlp_http_endpoint: format!("http://127.0.0.1:{}", ports["otlp_http"]),
        otlp_grpc_endpoint: format!("http://127.0.0.1:{}", ports["otlp_grpc"]),
        backend_http_endpoint: format!("http://127.0.0.1:{}", ports["backend_http"]),
        organization: options.organization.clone(),
        streams: options.streams.clone(),
        extra: BTreeMap::new(),
    };
    let mut config = Configuration {
        schema_version: 1,
        owner: OWNER.into(),
        owner_id: nonce(),
        generation: state.join("generations").join(&configuration_id),
        configuration_id,
        state: state.clone(),
        units: BTreeMap::new(),
        ports,
        binaries,
        files: BTreeMap::new(),
        retention_days: options.retention_days,
        public_configuration: public,
        limits: limits(),
        environment_file: None,
        extra: BTreeMap::new(),
    };
    for directory in [
        state.join("collector-queue"),
        state.join("openobserve-data"),
    ] {
        private_directory(&directory, true)?;
    }
    let email = "telemetry@localhost.local";
    let password = format!("pSe9!{}{}", nonce(), nonce());
    let authorization = format!("Basic {}", STANDARD.encode(format!("{email}:{password}")));
    let values: BTreeMap<String, String> = [
        ("PSE_TELEMETRY_AUTHORIZATION", authorization),
        ("ZO_ROOT_USER_EMAIL", email.into()),
        ("ZO_ROOT_USER_PASSWORD", password),
        ("ZO_LOCAL_MODE", "true".into()),
        ("ZO_LOCAL_MODE_STORAGE", "disk".into()),
        ("ZO_META_STORE", "sqlite".into()),
        ("ZO_QUEUE_STORE", "memory".into()),
        (
            "ZO_DATA_DIR",
            format!("{}/", text_path(&state.join("openobserve-data"))?),
        ),
        ("ZO_HTTP_ADDR", "127.0.0.1".into()),
        ("ZO_HTTP_PORT", config.ports["backend_http"].to_string()),
        ("ZO_GRPC_ADDR", "127.0.0.1".into()),
        ("ZO_GRPC_PORT", config.ports["backend_grpc"].to_string()),
        (
            "ZO_WEB_URL",
            config.public_configuration.backend_http_endpoint.clone(),
        ),
        ("ZO_TELEMETRY", "false".into()),
        ("ZO_SSRF_ALLOWED_CIDRS", ALERT_CIDR.into()),
        (
            "ZO_COMPACT_DATA_RETENTION_DAYS",
            options.retention_days.to_string(),
        ),
        (
            "ZO_COMPACT_EXTENDED_DATA_RETENTION_DAYS",
            options.retention_days.to_string(),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.into(), value))
    .collect();
    generation(&mut config, environment, &values).await?;
    write_json(&state.join("service.json"), &config)?;
    write_json(
        &state.join("configuration.json"),
        &config.public_configuration,
    )?;
    Ok(configuration_view(&config, json!({})))
}
fn configuration_view(config: &Configuration, services: Value) -> Value {
    json!({"state":config.state,"status":"configured","configuration":config.public_configuration,"owner_id":config.owner_id,"configuration_id":config.configuration_id,"generation":config.generation,"retention_days":config.retention_days,"limits":config.limits,"services":services,"endpoints":{"collector_health":format!("http://127.0.0.1:{}/",config.ports["health"]),"collector_metrics":format!("http://127.0.0.1:{}/metrics",config.ports["metrics"])},"versions":config.binaries.iter().map(|(name,binary)|(name.clone(),binary.version.clone())).collect::<BTreeMap<_,_>>(),"storage":{"collector_queue":config.state.join("collector-queue"),"backend":config.state.join("openobserve-data")},"declaration_sources":{"tools":".config/diagnostic-tools.toml","service":"pse-host::telemetry_service","collector":config.generation.join("collector.yaml")},"delivery":"not_observed"})
}
trait ServiceControl {
    async fn reload(&self) -> Result<(), HostError>;
    async fn inspect(
        &self,
        spec: &InstalledServiceSpec,
        load: bool,
    ) -> Result<Option<InstalledServiceState>, HostError>;
    async fn start(&self, spec: &InstalledServiceSpec) -> Result<(), HostError>;
    async fn stop(
        &self,
        spec: &InstalledServiceSpec,
        expected: &InstalledServiceState,
    ) -> Result<(), HostError>;
}
impl ServiceControl for HostManager {
    async fn reload(&self) -> Result<(), HostError> {
        self.reload_installed_units().await
    }
    async fn inspect(
        &self,
        spec: &InstalledServiceSpec,
        load: bool,
    ) -> Result<Option<InstalledServiceState>, HostError> {
        self.inspect_installed_service(spec, load).await
    }
    async fn start(&self, spec: &InstalledServiceSpec) -> Result<(), HostError> {
        self.start_installed_service(spec).await
    }
    async fn stop(
        &self,
        spec: &InstalledServiceSpec,
        expected: &InstalledServiceState,
    ) -> Result<(), HostError> {
        self.stop_installed_service(spec, expected).await
    }
}
async fn manager(environment: &Environment) -> Result<HostManager, HostError> {
    let address = environment
        .get(OsStr::new("DBUS_SESSION_BUS_ADDRESS"))
        .map(|value| {
            value
                .to_str()
                .ok_or_else(|| invalid("session bus address must be UTF-8"))
        })
        .transpose()?;
    // Explicitly choose this user's bus when a shell did not export its conventional address.
    let fallback = format!(
        "unix:path=/run/user/{}/bus",
        rustix::process::geteuid().as_raw()
    );
    HostManager::connect_user(Some(address.unwrap_or(&fallback))).await
}
async fn inspect_with(
    config: &Configuration,
    directory: &Path,
    control: &impl ServiceControl,
) -> Result<Value, HostError> {
    let mut services = serde_json::Map::new();
    for name in TOOLS {
        let spec = config.spec(name, directory)?;
        let view = match control.inspect(&spec, false).await {
            Ok(Some(state)) => {
                json!({"unit":spec.name,"load_state":"loaded","active_state":state.observation.active_state,"sub_state":state.observation.sub_state,"enabled":state.enabled,"ownership":"verified","process":state.process.map(|process|json!({"pid":process.pid,"start_ticks":process.start_ticks,"invocation_id":state.observation.invocation.iter().map(|byte|format!("{byte:02x}")).collect::<String>(),"control_group":state.observation.control_group}))})
            }
            Ok(None) => {
                json!({"unit":spec.name,"load_state":"not-found","active_state":"unknown","ownership":"absent","process":null})
            }
            Err(_) => {
                json!({"unit":spec.name,"load_state":"unavailable","active_state":"unknown","ownership":"unavailable","process":null,"problem":"service ownership or manager observation unavailable"})
            }
        };
        services.insert(name.into(), view);
    }
    Ok(configuration_view(config, Value::Object(services)))
}
/// Passive inspection reports unavailable manager facts and never repairs, loads or starts.
pub async fn inspect(environment: &Environment, state: Option<&Path>) -> Result<Value, HostError> {
    let state = state_directory(state, environment)?;
    if !state.try_exists()? {
        return Ok(json!({"state":state,"status":"unconfigured","services":{}}));
    }
    let config = checked_configuration(&state)?;
    let directory = unit_directory(environment)?;
    match manager(environment).await {
        Ok(control) => inspect_with(&config, &directory, &control).await,
        Err(_) => Ok(configuration_view(
            &config,
            json!({"collector":{"ownership":"unavailable","process":null},"openobserve":{"ownership":"unavailable","process":null}}),
        )),
    }
}
fn publish_units(config: &Configuration, directory: &Path) -> Result<(), HostError> {
    no_symlinks(directory)?;
    fs::create_dir_all(directory)?;
    no_symlinks(directory)?;
    for unit in config.units.values() {
        let source = config.generation.join(unit);
        let target = directory.join(unit);
        no_symlinks(&target)?;
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
        temporary.write_all(&fs::read(&source)?)?;
        temporary.as_file().sync_all()?;
        match temporary.persist_noclobber(&target) {
            Ok(_) => {}
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                private_file(&target)?;
                if digest(&source)? != digest(&target)? {
                    return Err(invalid(
                        "a different unit already occupies telemetry identity",
                    ));
                }
            }
            Err(error) => return Err(error.error.into()),
        }
    }
    Ok(())
}
async fn observations(
    config: &Configuration,
    directory: &Path,
    control: &impl ServiceControl,
    load: bool,
) -> Result<BTreeMap<String, Option<InstalledServiceState>>, HostError> {
    let mut result = BTreeMap::new();
    for name in TOOLS {
        result.insert(
            name.into(),
            control
                .inspect(&config.spec(name, directory)?, load)
                .await?,
        );
    }
    Ok(result)
}
async fn start_with(
    config: &Configuration,
    directory: &Path,
    control: &impl ServiceControl,
) -> Result<(), HostError> {
    let previous = observations(config, directory, control, true).await?;
    let mut reservations = Vec::new();
    for (key, port) in &config.ports {
        let name = if key.starts_with("backend_") {
            "openobserve"
        } else {
            "collector"
        };
        if previous[name]
            .as_ref()
            .is_none_or(|state| state.process.is_none())
        {
            reservations.push(
                TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, *port)).map_err(|_| {
                    invalid(format!("telemetry listener collision on 127.0.0.1:{port}"))
                })?,
            );
        }
    }
    drop(reservations);
    for name in ["openobserve", "collector"] {
        control.start(&config.spec(name, directory)?).await?;
    }
    Ok(())
}
async fn stop_with(
    config: &Configuration,
    directory: &Path,
    control: &impl ServiceControl,
) -> Result<(), HostError> {
    let mut previous = BTreeMap::new();
    let mut failure = None;
    // Authenticate every independently owned service. One foreign/unavailable
    // sibling cannot prevent settlement of another service whose owner is known.
    for name in TOOLS {
        let observed = match config.spec(name, directory) {
            Ok(spec) => control.inspect(&spec, false).await,
            Err(error) => Err(error),
        };
        match observed {
            Ok(state) => {
                previous.insert(name, state);
            }
            Err(error) => {
                previous.insert(name, None);
                failure = Some(match failure {
                    None => error,
                    Some(operation) => HostError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(error),
                    },
                });
            }
        }
    }
    for name in TOOLS {
        if let Some(expected) = &previous[name] {
            let stopped = match config.spec(name, directory) {
                Ok(spec) => control.stop(&spec, expected).await,
                Err(error) => Err(error),
            };
            if let Err(error) = stopped {
                failure = Some(match failure {
                    None => error,
                    Some(operation) => HostError::Settlement {
                        operation: Box::new(operation),
                        cleanup: Box::new(error),
                    },
                });
            }
        }
    }
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
/// Explicit start publishes only authenticated unit files, then activates backend before collector.
pub async fn start(environment: &Environment, state: Option<&Path>) -> Result<Value, HostError> {
    let state = state_directory(state, environment)?;
    let _lock = Lock::acquire(&state)?;
    let config = checked_configuration(&state)?;
    let directory = unit_directory(environment)?;
    let control = manager(environment).await?;
    publish_units(&config, &directory)?;
    control.reload().await?;
    start_with(&config, &directory, &control).await?;
    inspect_with(&config, &directory, &control).await
}
/// Explicit stop attempts both authentications and settles each independently verified owner.
pub async fn stop(environment: &Environment, state: Option<&Path>) -> Result<Value, HostError> {
    let state = state_directory(state, environment)?;
    let _lock = Lock::acquire(&state)?;
    let config = checked_configuration(&state)?;
    let directory = unit_directory(environment)?;
    let control = manager(environment).await?;
    stop_with(&config, &directory, &control).await?;
    inspect_with(&config, &directory, &control).await
}
#[derive(Serialize, Deserialize)]
struct Reconfiguration {
    previous: Configuration,
    candidate: Configuration,
    phase: String,
    running: bool,
    enabled: Vec<String>,
}
/// Explicitly replace configuration without rotating credentials or clearing queues/backend data.
/// Login activation is refused before effects; this operator does not silently retire that selection.
pub async fn reconfigure(
    environment: &Environment,
    state: Option<&Path>,
) -> Result<Value, HostError> {
    let state = state_directory(state, environment)?;
    let _lock = Lock::acquire(&state)?;
    let previous = checked_configuration(&state)?;
    let directory = unit_directory(environment)?;
    let control = manager(environment).await?;
    reconfigure_with(&state, &previous, environment, &directory, &control).await
}
async fn reconfigure_with(
    state: &Path,
    previous: &Configuration,
    environment: &Environment,
    directory: &Path,
    control: &impl ServiceControl,
) -> Result<Value, HostError> {
    let observed = observations(previous, directory, control, false).await?;
    if observed.values().flatten().any(|state| {
        matches!(
            state.enabled.as_str(),
            "enabled" | "enabled-runtime" | "linked-runtime"
        )
    }) {
        return Err(invalid(
            "telemetry login activation is selected; disable it explicitly before native reconfiguration",
        ));
    }
    let running = observed
        .values()
        .flatten()
        .any(|state| state.observation.active_state == "active");
    let mut values = environment_values(&previous.env_file())?;
    values.insert("ZO_SSRF_ALLOWED_CIDRS".into(), ALERT_CIDR.into());
    let mut candidate = previous.clone();
    candidate.configuration_id = nonce();
    candidate.generation = state.join("generations").join(&candidate.configuration_id);
    candidate.environment_file = Some(candidate.generation.join("service.env"));
    candidate.public_configuration.configuration_id = candidate.configuration_id.clone();
    candidate.extra.insert(
        "previous_configuration_id".into(),
        json!(previous.configuration_id),
    );
    generation(&mut candidate, environment, &values).await?;
    let mut receipt = Reconfiguration {
        previous: previous.clone(),
        candidate: candidate.clone(),
        phase: "validated".into(),
        running,
        enabled: vec![],
    };
    write_json(&state.join("reconfiguration.json"), &receipt)?;
    stop_with(previous, directory, control).await?;
    let activation = async {
        write_json(&state.join("service.json"), &candidate)?;
        write_json(
            &state.join("configuration.json"),
            &candidate.public_configuration,
        )?;
        validate_configuration(state, &candidate, &candidate.public_configuration)?;
        publish_units(&candidate, directory)?;
        control.reload().await?;
        if running {
            start_with(&candidate, directory, control).await?;
        }
        Ok::<(), HostError>(())
    }
    .await;
    if let Err(operation) = activation {
        let rollback = async {
            stop_with(&candidate, directory, control).await?;
            write_json(&state.join("service.json"), previous)?;
            write_json(
                &state.join("configuration.json"),
                &previous.public_configuration,
            )?;
            publish_units(previous, directory)?;
            control.reload().await?;
            if running {
                start_with(previous, directory, control).await?;
            }
            receipt.phase = "rolled_back".into();
            write_json(&state.join("reconfiguration.json"), &receipt)?;
            Ok::<(), HostError>(())
        }
        .await;
        return match rollback {
            Ok(()) => Err(operation),
            Err(cleanup) => Err(HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            }),
        };
    }
    receipt.phase = "activated".into();
    write_json(&state.join("reconfiguration.json"), &receipt)?;
    inspect_with(&candidate, directory, control).await
}
/// Explicit recovery of the exact authenticated predecessor retained by reconfiguration.
pub async fn recover_previous_configuration(
    environment: &Environment,
    state: Option<&Path>,
) -> Result<Value, HostError> {
    let state = state_directory(state, environment)?;
    let _lock = Lock::acquire(&state)?;
    let mut receipt: Reconfiguration = read_json(&state.join("reconfiguration.json"))?;
    if !receipt.enabled.is_empty() {
        return Err(invalid(
            "recovery contains login activation; reconcile that selection explicitly",
        ));
    }
    validate_configuration(
        &state,
        &receipt.previous,
        &receipt.previous.public_configuration,
    )?;
    validate_configuration(
        &state,
        &receipt.candidate,
        &receipt.candidate.public_configuration,
    )?;
    let selected: Configuration = read_json(&state.join("service.json"))?;
    if receipt.previous.owner_id != receipt.candidate.owner_id
        || selected.owner_id != receipt.previous.owner_id
        || ![
            receipt.previous.configuration_id.as_str(),
            receipt.candidate.configuration_id.as_str(),
        ]
        .contains(&selected.configuration_id.as_str())
    {
        return Err(invalid("reconfiguration recovery selection changed"));
    }
    let control = manager(environment).await?;
    let directory = unit_directory(environment)?;
    stop_with(&receipt.candidate, &directory, &control).await?;
    write_json(&state.join("service.json"), &receipt.previous)?;
    write_json(
        &state.join("configuration.json"),
        &receipt.previous.public_configuration,
    )?;
    publish_units(&receipt.previous, &directory)?;
    control.reload().await?;
    if receipt.running {
        start_with(&receipt.previous, &directory, &control).await?;
    }
    receipt.phase = "recovered_previous".into();
    write_json(&state.join("reconfiguration.json"), &receipt)?;
    inspect_with(&receipt.previous, &directory, &control).await
}
/// Native leaf command adapter. All stateful actions must be selected by name.
pub(crate) async fn execute(
    root: &Path,
    environment: &Environment,
    action: crate::cli::TelemetryAction,
) -> Result<Value, HostError> {
    use crate::cli::TelemetryAction;
    match action {
        TelemetryAction::Install => {
            let mut paths = BTreeMap::new();
            for name in TOOLS {
                paths.insert(
                    name,
                    diagnostic_tools::install(root, declaration(name), environment)?,
                );
            }
            Ok(json!({"installed":paths}))
        }
        TelemetryAction::Setup(args) => {
            let mut options = SetupOptions {
                organization: args.organization,
                retention_days: args.retention_days,
                ..SetupOptions::default()
            };
            for (name, port) in args.ports {
                if options.ports.insert(name, port).is_some() {
                    return Err(invalid("duplicate telemetry listener selection"));
                }
            }
            for (name, stream) in args.streams {
                options.streams.insert(name, stream);
            }
            setup(root, environment, args.state.as_deref(), &options).await
        }
        TelemetryAction::Inspect(args) => inspect(environment, args.state.as_deref()).await,
        TelemetryAction::Start(args) => start(environment, args.state.as_deref()).await,
        TelemetryAction::Stop(args) => stop(environment, args.state.as_deref()).await,
        TelemetryAction::Reconfigure(args) => reconfigure(environment, args.state.as_deref()).await,
        TelemetryAction::RecoverConfiguration(args) => {
            recover_previous_configuration(environment, args.state.as_deref()).await
        }
    }
}
#[cfg(test)]
#[path = "telemetry_service_tests.rs"]
mod tests;
