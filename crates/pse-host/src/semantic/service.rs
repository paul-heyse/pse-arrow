// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checkout-bound session discovery and authenticated local multiplexing, under B03 custody.
use super::{Configuration, Error, Reply, Request, Result, engine::Engine};
use crate::{
    envelope::CliInvocationId,
    environment::Environment,
    paths::{CheckoutState, HostPaths},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    io::Write as _,
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::sync::CancellationToken;

/// Explicit current session selector; immutable invocation records own actual native custody.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SessionRecord {
    /// Selector format, distinct from native protocol versions.
    pub version: u32,
    /// Original managed service invocation, never an arbitrary PID/unit name.
    pub invocation: CliInvocationId,
    /// Exact selected analyzer model.
    pub configuration: Configuration,
}
/// Actual initialization and selected context, separate from any successful source query.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SessionStatus {
    /// Selected Cargo analysis model.
    pub configuration: Configuration,
    /// Actual pinned analyzer --version response.
    pub backend: String,
    /// Actual negotiated LSP encoding.
    pub position_encoding: String,
    /// Selected-source synchronization generation; never a Git revision.
    pub generation: u64,
    /// Original actual initialization capabilities, retained without guessing.
    pub server_capabilities: serde_json::Value,
    /// Actual native loading/health notification when supplied.
    pub backend_status: Option<serde_json::Value>,
    /// Actual current semantic consumers; cancellation removes only that consumer.
    pub active_requests: usize,
    /// Captured Cargo/toolchain inputs still match; false never silently prepares another model.
    pub context_current: bool,
    /// Original reason the selected context could not be re-established.
    pub context_issue: Option<String>,
    /// Initial model activated and the native configuration exchange disabled automatic Cargo reload.
    pub automatic_reload_disabled: bool,
    /// Original failure of the one post-activation configuration exchange, if any.
    pub preparation_issue: Option<String>,
    /// Supported preparation boundary.
    pub preparation: String,
}
/// Passive session view; original native custody and analyzer readiness remain separate facts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SessionOverview {
    /// A selector exists; this does not establish a live/ready session.
    pub prepared: bool,
    /// Original immutable native invocation and selected analysis model.
    pub record: Option<SessionRecord>,
    /// Uncached original native observation, independent of analyzer query success.
    pub original_native: Option<crate::managed::supervisor::PassiveRecordedObservation>,
    /// Actual analyzer initialization/readiness, when reachable and authenticated.
    pub analyzer: Option<SessionStatus>,
    /// Original reason analyzer readiness could not be observed.
    pub unavailable: Option<String>,
}
fn host<T>(result: std::result::Result<T, crate::HostError>) -> Result<T> {
    result.map_err(|error| Error::Host(Box::new(error)))
}
fn state(root: &Path, environment: &Environment) -> Result<PathBuf> {
    Ok(CheckoutState::resolve(root, environment)
        .map_err(|error| Error::Host(Box::new(error.into())))?
        .root
        .join("semantic"))
}
fn private(path: &Path, create: bool) -> Result<()> {
    if create {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
    }
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(Error::Unavailable(
            "semantic state/socket parent must be an owned private directory".into(),
        ));
    }
    Ok(())
}
/// Hold preparation exclusion through selector inspection, launch and publication.
pub fn preparation_lock(root: &Path, environment: &Environment) -> Result<std::fs::File> {
    let state = state(root, environment)?;
    private(&state, true)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(state.join("preparation.lock"))?;
    file.lock()?;
    Ok(file)
}
/// Read only; absent preparation creates neither state nor service.
pub fn load(root: &Path, environment: &Environment) -> Result<Option<SessionRecord>> {
    let directory = state(root, environment)?;
    if !directory.exists() {
        return Ok(None);
    }
    private(&directory, false)?;
    let path = directory.join("current.json");
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let record: SessionRecord = serde_json::from_slice(&bytes)?;
    if record.version != 1 {
        return Err(Error::Unavailable(
            "unsupported semantic selector version".into(),
        ));
    }
    Ok(Some(record))
}
/// Atomically publish only the actual original managed invocation; caller holds preparation lock.
pub fn prepare_record(
    root: &Path,
    environment: &Environment,
    record: &SessionRecord,
) -> Result<()> {
    let directory = state(root, environment)?;
    private(&directory, true)?;
    let mut temporary = tempfile::NamedTempFile::new_in(&directory)?;
    temporary.write_all(&serde_json::to_vec(record)?)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(directory.join("current.json"))
        .map_err(|error| error.error)?;
    Ok(())
}
/// Short Unix address; the full checkout identity remains in authenticated invocation records.
pub fn socket_path(
    root: &Path,
    environment: &Environment,
    invocation: &CliInvocationId,
    create: bool,
) -> Result<PathBuf> {
    let checkout = CheckoutState::resolve(root, environment)
        .map_err(|error| Error::Host(Box::new(error.into())))?;
    let runtime = HostPaths::new(environment)
        .runtime()
        .map_err(|error| Error::Host(Box::new(error.into())))?;
    let directory = runtime.join(format!("pse-code-{}", &checkout.checkout_id.as_str()[..16]));
    private(&directory, create)?;
    let path = directory.join(format!("{}.sock", invocation.as_str().replace('-', "")));
    use std::os::unix::ffi::OsStrExt as _;
    if path.as_os_str().as_bytes().len() >= 108 {
        return Err(Error::Unavailable(
            "selected runtime directory exceeds the Linux Unix socket address limit".into(),
        ));
    }
    Ok(path)
}
/// Authenticate the actual kernel peer against original payload custody before sending any input.
pub async fn client(
    root: &Path,
    environment: &Environment,
    record: &SessionRecord,
    request: &Request,
) -> Result<serde_json::Value> {
    let socket = socket_path(root, environment, &record.invocation, false)?;
    let mut stream = tokio::net::UnixStream::connect(socket).await.map_err(|error|Error::Unavailable(format!("original semantic socket is unavailable ({error}); inspect code session status and explicitly prepare a session")))?;
    let peer = stream.peer_cred()?;
    let pid = peer
        .pid()
        .and_then(|pid| u32::try_from(pid).ok())
        .ok_or_else(|| Error::Unavailable("kernel socket peer PID is unavailable".into()))?;
    host(
        crate::managed::supervisor::authenticate_payload_peer(
            root,
            &record.invocation,
            pid,
            environment,
        )
        .await,
    )?;
    stream.write_all(&serde_json::to_vec(request)?).await?;
    stream.write_all(b"\n").await?;
    let mut response = String::new();
    let count = tokio::io::BufReader::new(stream)
        .read_line(&mut response)
        .await?;
    if count == 0 || !response.ends_with('\n') {
        return Err(Error::Protocol(
            "semantic peer closed without a complete response".into(),
        ));
    }
    match serde_json::from_str::<Reply>(&response)? {
        Reply::Answer { value } => Ok(value),
        Reply::Failed { failure } => Err(Error::Refusal(failure)),
    }
}
async fn connection(
    stream: tokio::net::UnixStream,
    engine: Arc<Engine>,
    stopping: CancellationToken,
) -> Result<()> {
    if stream.peer_cred()?.uid() != rustix::process::geteuid().as_raw() {
        return Err(Error::Unavailable(
            "semantic client UID differs from private session owner".into(),
        ));
    }
    let (read, mut write) = stream.into_split();
    let mut read = tokio::io::BufReader::new(read);
    let mut frame = String::new();
    // Fixed request grammar plus a wire extent; no arbitrary backend payload is accepted.
    let count = (&mut read).take(64 * 1024).read_line(&mut frame).await?;
    if count == 0 || !frame.ends_with('\n') {
        return Err(Error::Protocol(
            "incomplete semantic request or request frame exceeds 64 KiB".into(),
        ));
    }
    let request: Request = serde_json::from_str(&frame)?;
    let mut disconnected = [0u8; 1];
    let response = match request {
        Request::Status {} => serde_json::to_value(engine.status()?).map_err(Error::from),
        Request::Stop {} => {
            stopping.cancel();
            Ok(serde_json::json!({"stop_requested":true,"native_drain":"unobserved"}))
        }
        Request::Query(query) => tokio::select! {
            result = engine.query(query) => result.and_then(|answer| serde_json::to_value(answer).map_err(Error::from)),
            _ = read.read(&mut disconnected) => return Ok(()),
            _ = stopping.cancelled() => return Ok(()),
        },
    };
    let reply = match response {
        Ok(value) => Reply::Answer { value },
        Err(error) => Reply::Failed {
            failure: super::Failure::from_error(&error),
        },
    };
    write.write_all(&serde_json::to_vec(&reply)?).await?;
    write.write_all(b"\n").await?;
    write.shutdown().await?;
    Ok(())
}
/// Fixed native payload entry. Its analyzer/proc-macro descendants stay in B03's original group.
pub async fn serve(
    root: PathBuf,
    invocation: CliInvocationId,
    configuration: Configuration,
) -> Result<()> {
    let environment: Environment = std::env::vars_os().collect();
    // No manually invoked hidden entry may start a server outside authenticated native custody.
    let leader = host(crate::managed::supervisor::current_payload(&environment).await)?;
    if leader.root() != root || leader.invocation_id() != &invocation {
        return Err(Error::Unavailable(
            "semantic entry differs from original payload routing".into(),
        ));
    }
    let metadata: crate::invocation::InvocationMetadata<Configuration> = host(leader.metadata())?;
    if metadata.selection.command != "code session prepare"
        || metadata.selection.operation != configuration
    {
        return Err(Error::Unavailable(
            "semantic model differs from its original immutable native selection".into(),
        ));
    }
    let socket = socket_path(&root, &environment, &invocation, true)?;
    let listener = tokio::net::UnixListener::bind(&socket)?;
    let directory = state(&root, &environment)?;
    private(&directory, true)?;
    let stderr = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(directory.join(format!("{}.stderr", invocation.as_str())))?;
    let engine = Arc::new(Engine::start(&root, configuration, stderr).await?);
    let stopping = CancellationToken::new();
    let mut clients = tokio::task::JoinSet::new();
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut check = tokio::time::interval(Duration::from_millis(250));
    let result = loop {
        tokio::select! {
            connected = listener.accept() => match connected {
                Ok((stream, _)) => { let engine = engine.clone(); let token = stopping.clone(); clients.spawn(async move { connection(stream, engine, token).await }); },
                Err(error) => break Err(Error::Io(error)),
            },
            _ = stopping.cancelled() => break Ok(()),
            _ = terminate.recv() => break Ok(()),
            _ = tokio::signal::ctrl_c() => break Ok(()),
            Some(_) = clients.join_next(), if !clients.is_empty() => {},
            _ = check.tick() => { if engine.driver.is_finished() { break Err(Error::Protocol("analyzer connection terminated".into())); } },
        }
    };
    stopping.cancel();
    clients.abort_all();
    while clients.join_next().await.is_some() {}
    drop(listener);
    let cleanup = match Arc::try_unwrap(engine) {
        Ok(engine) => engine.shutdown().await,
        Err(_) => Err(Error::Protocol(
            "semantic engine still has a live request owner".into(),
        )),
    };
    let socket_cleanup = std::fs::remove_file(socket);
    result?;
    cleanup?;
    socket_cleanup?;
    Ok(())
}
/// Stop the original session and prove native drain. Acknowledgement alone is never settlement.
pub async fn stop(
    root: &Path,
    environment: &Environment,
    record: &SessionRecord,
) -> Result<crate::managed::supervisor::RecordedObservation> {
    let _graceful = tokio::time::timeout(
        Duration::from_secs(5),
        client(root, environment, record, &Request::Stop {}),
    )
    .await;
    let mut original =
        host(crate::managed::supervisor::recorded(root, &record.invocation, environment).await)?;
    if let Some(observed) = host(
        original
            .wait(Some(Instant::now() + Duration::from_secs(15)))
            .await,
    )? {
        return Ok(observed);
    }
    host(
        original
            .cancel(Instant::now() + Duration::from_secs(30))
            .await,
    )
}
