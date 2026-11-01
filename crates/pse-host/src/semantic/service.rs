// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checkout-bound session discovery and authenticated local multiplexing, under B03 custody.
use super::{Backend, Configuration, Error, Request, Response, Result, engine::Engine};
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
use tokio_util::sync::CancellationToken;

pub(super) enum ServiceEngine {
    Lsp(Engine),
    #[cfg(feature = "native-hir")]
    Snapshot(super::native::Engine),
    #[cfg(test)]
    Fixture(Arc<crate::plan61_rpc_qualification::Fixture>),
}
impl ServiceEngine {
    pub(super) fn status(&self) -> Result<SessionStatus> {
        match self {
            Self::Lsp(engine) => engine.status(),
            #[cfg(feature = "native-hir")]
            Self::Snapshot(engine) => engine.status(),
            #[cfg(test)]
            Self::Fixture(engine) => Ok(engine.status()),
        }
    }
    pub(super) fn driver_finished(&self) -> bool {
        match self {
            Self::Lsp(engine) => engine.driver.is_finished(),
            #[cfg(feature = "native-hir")]
            Self::Snapshot(_) => false,
            #[cfg(test)]
            Self::Fixture(_) => false,
        }
    }
    async fn shutdown(self) -> Result<()> {
        match self {
            Self::Lsp(engine) => engine.shutdown().await,
            #[cfg(feature = "native-hir")]
            Self::Snapshot(_) => Ok(()),
            #[cfg(test)]
            Self::Fixture(_) => Ok(()),
        }
    }
}
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
    if !matches!(record.version, 1 | 2) {
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
/// Authenticate every physical connection before HTTP2 negotiation.
pub async fn client(
    root: &Path,
    environment: &Environment,
    record: &SessionRecord,
    request: &Request,
) -> Result<Response> {
    super::rpc::client(root, environment, record, request, None).await
}
/// Cancellation completion is observed only from the operation's typed terminal.
pub async fn client_cancellable(
    root: &Path,
    environment: &Environment,
    record: &SessionRecord,
    request: &Request,
    cancellation: &CancellationToken,
) -> Result<Response> {
    if cancellation.is_cancelled() {
        return Err(Error::Cancelled);
    }
    super::rpc::client(root, environment, record, request, Some(cancellation)).await
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
    let stopping = CancellationToken::new();
    let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    #[cfg(feature = "native-hir")]
    let mut terminate = terminate;
    let engine = match configuration.backend {
        Backend::Lsp => ServiceEngine::Lsp(Engine::start(&root, configuration, stderr).await?),
        Backend::NativeSnapshot => {
            #[cfg(feature = "native-hir")]
            {
                let root = root.clone();
                let mut task =
                    crate::source_command::batch::spawn_cpu(stopping.child_token(), move |token| {
                        let engine = super::native::Engine::start(&root, configuration)?;
                        if token.is_cancelled() {
                            return Err(Error::Cancelled);
                        }
                        Ok(engine)
                    });
                let engine = tokio::select! {
                    result = task.settle() => result.map_err(super::engine::protocol)??,
                    _ = terminate.recv() => {
                        task.cancel();
                        let _settled = task.settle().await.map_err(super::engine::protocol)?;
                        return Err(Error::Cancelled);
                    },
                    _ = tokio::signal::ctrl_c() => {
                        task.cancel();
                        let _settled = task.settle().await.map_err(super::engine::protocol)?;
                        return Err(Error::Cancelled);
                    },
                };
                ServiceEngine::Snapshot(engine)
            }
            #[cfg(not(feature = "native-hir"))]
            {
                return Err(Error::Unavailable("native snapshot requires the explicitly prepared native-analysis helper; the ordinary host does not load this optional dependency family".into()));
            }
        }
    };
    let engine = Arc::new(engine);
    let result = super::rpc::serve(listener, engine.clone(), stopping, terminate).await;
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
    if record.version == 1 {
        let mut original = host(
            crate::managed::supervisor::recorded(root, &record.invocation, environment).await,
        )?;
        return host(
            original
                .cancel(Instant::now() + Duration::from_secs(30))
                .await,
        );
    }
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
