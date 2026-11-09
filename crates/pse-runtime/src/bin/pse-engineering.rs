// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicitly prepared canonical engineering service over original managed payload custody.
// This consumer instantiates the same finite nested async/Send proofs as the
// runtime library and the other original scientific workflow binaries.
#![recursion_limit = "512"]
#![allow(
    clippy::print_stderr,
    reason = "standalone service reports actionable terminal refusal"
)]

use clap::Parser;
use pse_operations::canonical::{CanonicalOptions, CanonicalStore};
use pse_runtime::{
    CancelSource, SharedRuntime,
    host_operations::{
        engineering::{
            EngineeringFrontendReady, EngineeringProviderPreparation, EngineeringService,
            EngineeringServiceError, LocalAccessPolicy, serve,
        },
        project::PROJECT_OPERATOR_DEFAULTS,
    },
    workflow::{CanonicalDeployment, OuterAttestation, Runtime},
};
use std::{
    collections::BTreeMap,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

#[path = "support/native-stack.rs"]
mod native_stack;
#[path = "support/signals.rs"]
mod signals;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[cfg(feature = "property-coolprop")]
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum ProviderRoute {
    WaterLiquid,
    WaterVapor,
    WaterSaturation,
}

#[cfg(feature = "property-coolprop")]
impl ProviderRoute {
    fn registration(self) -> (&'static str, EngineeringProviderPreparation) {
        use pse_backend_native::coolprop::{self, WaterRegime};
        let (name, prepare): (_, fn(&pse_quantity::QuantityRegistry) -> _) = match self {
            Self::WaterLiquid => (WaterRegime::Liquid.implementation(), |registry| {
                coolprop::registration(registry, WaterRegime::Liquid)
            }),
            Self::WaterVapor => (WaterRegime::Vapor.implementation(), |registry| {
                coolprop::registration(registry, WaterRegime::Vapor)
            }),
            Self::WaterSaturation => (
                coolprop::WATER_SATURATION_IMPLEMENTATION,
                coolprop::saturation_registration,
            ),
        };
        (
            name,
            EngineeringProviderPreparation::new(prepare, coolprop::WATER_REGISTRATION_BYTES),
        )
    }
}

#[derive(Debug, Parser)]
#[command(
    about = "Start one explicit authenticated native engineering service; never starts a database"
)]
struct Args {
    /// Exact checkout whose original managed invocation owns this payload.
    #[arg(long)]
    root: PathBuf,
    /// Existing configured canonical deployment.
    #[arg(long)]
    state: PathBuf,
    /// Existing initialized canonical database; omission uses the configured owner.
    #[arg(long)]
    database: Option<String>,
    /// Fresh Unix socket; an existing destination always refuses.
    #[arg(long)]
    socket: PathBuf,
    /// Fresh atomic readiness document; this reports preparation, not scientific eligibility.
    #[arg(long)]
    ready: PathBuf,
    /// Explicit kernel user admitted before effectful dispatch.
    #[arg(long)]
    allow_uid: u32,
    /// Optional additional kernel group restriction.
    #[arg(long)]
    allow_gid: Option<u32>,
    /// Accounted runtime capacity, independent of host workload placement.
    #[arg(long, default_value_t = PROJECT_OPERATOR_DEFAULTS.memory_limit_bytes)]
    memory_limit_bytes: usize,
    /// Existing native scientific permits; independent of compiler/host CPU placement.
    #[arg(long, default_value_t = PROJECT_OPERATOR_DEFAULTS.threads)]
    threads: usize,
    #[arg(long)]
    spill_dir: PathBuf,
    #[arg(long, default_value_t = PROJECT_OPERATOR_DEFAULTS.max_spill_bytes)]
    max_spill_bytes: u64,
    #[arg(long, default_value_t = pse_runtime::host_operations::project::default_batch_size())]
    batch_size: usize,
    /// Finite original operation retention; full inventory refuses new identities.
    #[arg(long, default_value_t = 128)]
    maximum_operations: usize,
    /// Finite client-connection retention, independent of scientific CPU placement.
    #[arg(long, default_value_t = 64)]
    maximum_connections: usize,
    /// Permit typed source requests to select this explicitly prepared host capability.
    #[cfg(feature = "property-coolprop")]
    #[arg(long, value_enum)]
    allow_provider: Vec<ProviderRoute>,
    /// Transport and terminal canonical-connection drain allowance after native settlement.
    #[arg(long, default_value_t = 30_000)]
    shutdown_ms: u64,
}
macro_rules! field {
    (memory_limit_bytes,$args:ident) => {
        $args.memory_limit_bytes
    };
    (threads,$args:ident) => {
        $args.threads
    };
    (spill_dir,$args:ident) => {
        $args.spill_dir.clone()
    };
    (max_spill_bytes,$args:ident) => {
        $args.max_spill_bytes
    };
    (batch_size,$args:ident) => {
        $args.batch_size
    };
    ($name:ident,$args:ident) => {
        None
    };
}
fn budget(args: &Args) -> Result<pse_runtime::ResourceBudget, pse_runtime::RuntimeError> {
    macro_rules! input { ($budget:ident; $($name:ident:$ty:ty,$hint:literal $(,$default:expr)? =>$out:ty,$read:expr;)*)=>{
        pse_runtime::settings::EngineSettingsInput{$($name:field!($name,args),)*}
    }; }
    pse_runtime::engine_settings_fields!(input, unused).resolve(None)
}

/// Remove only the very socket inode this process actually created.
struct SocketOwner {
    path: PathBuf,
    device: u64,
    inode: u64,
}
impl SocketOwner {
    fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_socket() {
            return Err(std::io::Error::other(
                "bound engineering socket was replaced before custody observation",
            ));
        }
        Ok(Self {
            path,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
}
impl Drop for SocketOwner {
    fn drop(&mut self) {
        if let Ok(metadata) = std::fs::symlink_metadata(&self.path)
            && metadata.file_type().is_socket()
            && metadata.dev() == self.device
            && metadata.ino() == self.inode
        {
            let _removed = std::fs::remove_file(&self.path);
        }
    }
}

fn publish_ready(path: &Path, ready: &EngineeringFrontendReady) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or("readiness output needs an existing parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(temporary.as_file_mut(), ready)?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(path)?;
    Ok(())
}

async fn run(args: Args) -> Result<(), Error> {
    let root = std::fs::canonicalize(&args.root)?;
    let environment: pse_host::environment::Environment = std::env::vars_os().collect();
    let invocation =
        pse_host::envelope::CliInvocationId::try_from(std::env::var("PSE_MANAGED_INVOCATION")?)?;
    pse_host::managed::supervisor::authenticate_payload_peer(
        &root,
        &invocation,
        std::process::id(),
        &environment,
    )
    .await?;
    let budget = budget(&args)?;
    let provider_preparations: BTreeMap<_, EngineeringProviderPreparation> = {
        #[cfg(feature = "property-coolprop")]
        {
            let mut selected = BTreeMap::new();
            for route in &args.allow_provider {
                let (name, prepare) = route.registration();
                if selected.insert(name, prepare).is_some() {
                    return Err("duplicate --allow-provider selection".into());
                }
            }
            selected
        }
        #[cfg(not(feature = "property-coolprop"))]
        BTreeMap::new()
    };
    let mut artifact = std::fs::File::open(std::env::current_exe()?)?;
    let mut digest = pse_ids::EncodingHasher::new();
    std::io::copy(&mut artifact, &mut digest)?;
    let build = digest.finish().content_hash();
    let image = pse_buildinfo::FileObservation::capture(&std::env::current_exe()?)?;
    let mut options = CanonicalOptions::from_state(&args.state)?;
    if let Some(database) = args.database {
        options.database = database;
    }
    let store = CanonicalStore::connect(&options).await?;
    let stop = CancelSource::new();
    let signal = signals::ProcessInterrupt::new(stop.clone())?;
    let action: Result<(), Error> = async {
        store.open().await?;
        let shared = SharedRuntime::build(budget)?;
        let sessions =
            Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
        let runtime = Runtime::from_shared(
            shared.clone(),
            sessions.bind_model(pse_schema::shared_registry()?)?,
            CanonicalDeployment::new(
                store.clone(),
                OuterAttestation {
                    source: None,
                    build,
                },
                None,
            ),
        )?;
        let engine = EngineeringService::new(
            runtime,
            shared,
            args.maximum_operations,
            provider_preparations,
        )?;
        let setup: Result<_, Error> = (|| {
            let listener = tokio::net::UnixListener::bind(&args.socket)?;
            let socket = SocketOwner::new(args.socket.clone())?;
            std::fs::set_permissions(&args.socket, std::fs::Permissions::from_mode(0o600))?;
            publish_ready(
                &args.ready,
                &EngineeringFrontendReady {
                    version: Default::default(),
                    root: root.clone(),
                    invocation: invocation.clone(),
                    socket: args.socket.clone(),
                    process: std::process::id(),
                    build,
                    artifact: image.clone(),
                },
            )?;
            Ok((listener, socket))
        })();
        match setup {
            Ok((listener, _socket)) => serve(
                listener,
                engine,
                LocalAccessPolicy {
                    uid: args.allow_uid,
                    gid: args.allow_gid,
                    maximum_connections: args.maximum_connections,
                    connection_drain: Duration::from_millis(args.shutdown_ms),
                },
                stop,
            )
            .await
            .map_err(Into::into),
            Err(action) => match engine.shutdown().await {
                Ok(()) => Err(action),
                Err(cleanup) => {
                    let failure: Error = Box::new(EngineeringServiceError::Shutdown(vec![
                        EngineeringServiceError::Owner(action),
                        cleanup,
                    ]));
                    Err(failure)
                }
            },
        }
    }
    .await;
    signal.finish().await;
    let deadline = tokio::time::Instant::now()
        .checked_add(Duration::from_millis(args.shutdown_ms))
        .ok_or("canonical terminal allowance exceeds clock range")?;
    let drain = store.drain_owned_connection_until(deadline).await;
    match (action, drain) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(action), Ok(())) => Err(action),
        (Ok(()), Err(drain)) => Err(Box::new(drain)),
        (Err(action), Err(drain)) => Err(Box::new(EngineeringServiceError::Shutdown(vec![
            EngineeringServiceError::Owner(action),
            EngineeringServiceError::Owner(Box::new(drain)),
        ]))),
    }
}
fn main() -> std::process::ExitCode {
    let result: Result<(), Error> = (|| {
        let args = Args::parse();
        let stack = native_stack::selected()?;
        // Like the existing scientific worker, construct and poll the actual
        // native application root on its qualified stack; block_on's caller is
        // not resized by RUST_MIN_STACK or Tokio's worker configuration.
        std::thread::Builder::new()
            .name("pse-engineering-root".into())
            .stack_size(stack)
            .spawn(move || -> Result<(), Error> {
                let executor = native_stack::runtime(stack)?;
                executor.block_on(run(args))
            })?
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pse-engineering: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
