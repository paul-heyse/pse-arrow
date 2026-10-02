// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The store handle: a deadpool pool of tokio-postgres connections (ADR-0114 Outcome 26).
//! Schema creation is `Store::open` (`schema.rs`).

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pse_operations_queries::queries::store as statements;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::error::{Classify, OperationsError, Target};
use crate::listener::{Listener, Subscription};

/// The environment variable naming the operational store (ADR-0114 Outcome 21).
pub const DATABASE_URL_ENV: &str = "PSE_DATABASE_URL";

/// The development default: database `pse` over the local Unix socket with peer
/// authentication, so no credential exists anywhere (`docs/dev/operational-store.md`).
pub const DEFAULT_DATABASE_URL: &str = "postgres:///pse?host=/var/run/postgresql";

/// The oldest supported server, as `server_version_num` (ADR-0114 Outcome 21).
pub const MINIMUM_SERVER_VERSION: i32 = 180_000;

/// The `application_name` of pooled store sessions.
pub(crate) const APPLICATION: &str = "pse-operations";

/// How long a pooled connection may take to prove it is idle before it is replaced.
const RECYCLE_TIMEOUT: Duration = Duration::from_secs(1);

/// `PSE_DATABASE_URL`, or the development default when it is unset or empty.
pub fn database_url_from_env() -> String {
    std::env::var(DATABASE_URL_ENV)
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| DEFAULT_DATABASE_URL.to_owned())
}

/// Pool and session settings (architecture §9.7).
#[derive(Clone, Debug)]
pub struct StoreOptions {
    /// Pool size; keep it well below the server's `max_connections`.
    pub max_connections: u32,
    /// How long to wait for a pooled connection, and for a new one to be established.
    pub acquire_timeout: Duration,
    /// Server-side `idle_in_transaction_session_timeout`: a stuck client cannot hold row
    /// locks indefinitely.
    pub idle_in_transaction_timeout: Duration,
}

impl Default for StoreOptions {
    fn default() -> Self {
        Self {
            max_connections: 10,
            acquire_timeout: Duration::from_secs(10),
            idle_in_transaction_timeout: Duration::from_secs(60),
        }
    }
}

impl StoreOptions {
    /// A small pool for one isolated test database: many tests run at once against one
    /// local server whose `max_connections` is 100.
    pub fn for_tests() -> Self {
        Self {
            max_connections: 4,
            ..Self::default()
        }
    }
}

/// The server a store is connected to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerInfo {
    /// `server_version_num`, e.g. 180006.
    pub version_num: i32,
    /// `server_version`, e.g. `18.6 (Ubuntu 18.6-1.pgdg24.04+2)`.
    pub version: String,
}

impl ServerInfo {
    /// Whether the server satisfies [`MINIMUM_SERVER_VERSION`].
    pub const fn is_supported(&self) -> bool {
        self.version_num >= MINIMUM_SERVER_VERSION
    }
}

/// A handle on the operational store. Cheap to clone; clones share the pool.
#[derive(Clone)]
pub struct Store {
    inner: Arc<Inner>,
}

struct Inner {
    pool: deadpool_postgres::Pool,
    runtime: tokio::runtime::Handle,
    /// The connection configuration, for connections outside the pool: the listener's
    /// and test sessions.
    config: tokio_postgres::Config,
    tls: MakeRustlsConnect,
    target: Target,
    /// How long to wait for a connection, and for `LISTEN` to take effect.
    connect_timeout: Duration,
    /// The one `LISTEN` connection of this store, started by the first watcher (X8).
    listener: tokio::sync::OnceCell<Listener>,
    /// Shared schema-admission lease for this opened generation.
    schema_lease: tokio::sync::Mutex<Option<SchemaSession>>,
    closed: AtomicBool,
    lease_abort: Mutex<Option<tokio::task::AbortHandle>>,
}

/// One owned connection: session-level locks end only after its connection is destroyed.
#[derive(Debug)]
pub(crate) struct SchemaSession {
    pub(crate) client: tokio_postgres::Client,
    connection: tokio::task::JoinHandle<()>,
}
impl Drop for SchemaSession {
    fn drop(&mut self) {
        self.connection.abort();
    }
}
impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("target", &self.inner.target)
            .field("pool", &self.inner.pool.status())
            .finish_non_exhaustive()
    }
}

/// rustls with the ring provider named explicitly: the process-level default provider is
/// never consulted, whichever providers other dependencies enable. Remote servers are
/// verified against the webpki roots; a local socket never negotiates TLS.
pub(crate) fn tls() -> Result<MakeRustlsConnect, OperationsError> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| OperationsError::Configuration {
            reason: format!("TLS configuration: {error}"),
        })?
        .with_root_certificates(rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth();
    Ok(MakeRustlsConnect::new(config))
}

/// The connection configuration of `url` with the store's session settings.
pub(crate) fn configure(
    url: &str,
    options: &StoreOptions,
) -> Result<tokio_postgres::Config, OperationsError> {
    let mut config: tokio_postgres::Config = url.parse().map_err(
        |error: tokio_postgres::Error| OperationsError::Configuration {
            reason: format!("invalid operational store URL: {error}"),
        },
    )?;
    let timeout = format!(
        "-c idle_in_transaction_session_timeout={}",
        options.idle_in_transaction_timeout.as_millis()
    );
    let session = match config.get_options() {
        Some(existing) if !existing.is_empty() => format!("{existing} {timeout}"),
        _ => timeout,
    };
    // As libpq does, `PGUSER` names the role when the URL does not: a process whose OS
    // user has no role of its own (the solver container) is peer-authenticated as it.
    if config.get_user().is_none()
        && let Some(user) = std::env::var("PGUSER").ok().filter(|user| !user.is_empty())
    {
        config.user(user);
    }
    config
        .application_name(APPLICATION)
        .options(session)
        .connect_timeout(options.acquire_timeout);
    Ok(config)
}

impl Store {
    /// Connect with default [`StoreOptions`] and verify the server version. No schema
    /// work happens here: [`Store::open`] creates or checks the schema.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Configuration`] for an invalid URL or a server older than 18;
    /// [`OperationsError::Unavailable`] when the server cannot be reached.
    pub async fn connect(url: &str) -> Result<Self, OperationsError> {
        Self::connect_with(url, &StoreOptions::default()).await
    }

    /// Connect with explicit options and verify the server version.
    ///
    /// # Errors
    ///
    /// As for [`Store::connect`].
    pub async fn connect_with(url: &str, options: &StoreOptions) -> Result<Self, OperationsError> {
        let config = configure(url, options)?;
        let target = Target::describe(&config);
        let tls = tls()?;
        // A cancelled operation returns its connection while the server may still run its
        // statement (a lock wait, say). A pooled connection is therefore verified before
        // it is handed out again; one that does not answer within the recycle timeout is
        // discarded and replaced, so nobody queues behind a dropped statement.
        let manager = deadpool_postgres::Manager::from_config(
            config.clone(),
            tls.clone(),
            deadpool_postgres::ManagerConfig {
                recycling_method: deadpool_postgres::RecyclingMethod::Verified,
            },
        );
        let size = usize::try_from(options.max_connections).unwrap_or(usize::MAX);
        let pool = deadpool_postgres::Pool::builder(manager)
            .max_size(size)
            .wait_timeout(Some(options.acquire_timeout))
            .create_timeout(Some(options.acquire_timeout))
            .recycle_timeout(Some(RECYCLE_TIMEOUT))
            .runtime(deadpool_postgres::Runtime::Tokio1)
            .build()
            .map_err(|error| OperationsError::Configuration {
                reason: format!("operational store pool: {error}"),
            })?;
        let store = Self {
            inner: Arc::new(Inner {
                pool,
                runtime: tokio::runtime::Handle::try_current().map_err(|error| {
                    OperationsError::Configuration {
                        reason: format!("store shutdown runtime: {error}"),
                    }
                })?,
                config,
                tls,
                target,
                connect_timeout: options.acquire_timeout,
                listener: tokio::sync::OnceCell::new(),
                schema_lease: tokio::sync::Mutex::new(None),
                closed: AtomicBool::new(false),
                lease_abort: Mutex::new(None),
            }),
        };
        let server = store.server().await?;
        if !server.is_supported() {
            return Err(OperationsError::Configuration {
                reason: format!(
                    "{} runs PostgreSQL {}; the operational store requires 18 or newer",
                    store.target(),
                    server.version
                ),
            });
        }
        Ok(store)
    }

    /// Connect with explicit options, then validate the existing schema ([`Store::open`]).
    ///
    /// # Errors
    ///
    /// As for [`Store::connect`] and [`Store::open`].
    pub async fn open_with(url: &str, options: &StoreOptions) -> Result<Self, OperationsError> {
        let store = Self::connect_with(url, options).await?;
        store.open().await?;
        Ok(store)
    }

    /// A pooled connection.
    pub(crate) async fn client(&self) -> Result<deadpool_postgres::Object, OperationsError> {
        let lease = self.inner.schema_lease.lock().await;
        if self.inner.closed.load(Ordering::Acquire)
            || !lease.as_ref().is_some_and(|s| !s.client.is_closed())
        {
            return Err(OperationsError::MigrationRefused {
                reason: crate::schema::MigrationRefusal::NotReady,
                detail: "open a verified store generation before acquiring repository statements"
                    .into(),
            });
        }
        // Acquire while holding admission ownership; migration waits for the whole generation.
        self.inner.pool.get().await.classify(self.target())
    }
    pub(crate) async fn admin_client(&self) -> Result<deadpool_postgres::Object, OperationsError> {
        self.inner.pool.get().await.classify(self.target())
    }
    pub(crate) async fn schema_session(&self) -> Result<SchemaSession, OperationsError> {
        let (client, connection) = self
            .inner
            .config
            .connect(self.inner.tls.clone())
            .await
            .classify(self.target())?;
        let connection = tokio::spawn(async move {
            let _ = connection.await;
        });
        client
            .batch_execute("SET search_path=pg_catalog,public")
            .await
            .classify(self.target())?;
        Ok(SchemaSession { client, connection })
    }
    pub(crate) async fn retain_schema_lease(&self) -> Result<(), OperationsError> {
        let mut lease = self.inner.schema_lease.lock().await;
        if let Some(session) = lease.as_ref().filter(|session| !session.client.is_closed()) {
            crate::schema::verify_ready(&session.client, self.target()).await?;
            return Ok(());
        }
        let session = self.schema_session().await?;
        session
            .client
            .query_one(
                "SELECT pg_advisory_lock_shared($1)",
                &[&crate::schema::SCHEMA_LOCK],
            )
            .await
            .classify(self.target())?;
        crate::schema::verify_ready(&session.client, self.target()).await?;
        let mut abort = self
            .inner
            .lease_abort
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(OperationsError::MigrationRefused {
                reason: crate::schema::MigrationRefusal::NotReady,
                detail: "store generation closed during admission".into(),
            });
        }
        *abort = Some(session.connection.abort_handle());
        *lease = Some(session);
        Ok(())
    }
    pub(crate) async fn has_schema_lease(&self) -> bool {
        self.inner.schema_lease.lock().await.is_some()
    }

    /// Every notification from now on, once the store's listener has `LISTEN` in effect
    /// (Plan 22 X8). The first subscription starts the listener.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Unavailable`] when `LISTEN` does not take effect in time.
    pub(crate) async fn subscribe(&self) -> Result<Subscription, OperationsError> {
        let listener = self
            .inner
            .listener
            .get_or_init(|| async { Listener::spawn(&self.inner.config, &self.inner.tls) })
            .await;
        listener
            .subscribe(&self.inner.target, self.inner.connect_timeout)
            .await
    }

    /// The store's listener, once a watcher started it.
    #[cfg(test)]
    pub(crate) fn listener(&self) -> Option<&Listener> {
        self.inner.listener.get()
    }

    /// The connection configuration, for connections outside the pool.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn config(&self) -> &tokio_postgres::Config {
        &self.inner.config
    }

    /// The TLS connector every connection of this store uses.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn tls(&self) -> &MakeRustlsConnect {
        &self.inner.tls
    }

    /// Forget every cached prepared statement: after a reset the objects they name are
    /// gone.
    pub(crate) fn forget_statements(&self) {
        self.inner.pool.manager().statement_caches.clear();
    }

    /// Close the store: its listener stops, waiting and later acquisitions fail as
    /// unavailable, and connections are closed as they are returned.
    pub fn close(&self) {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Some(listener) = self.inner.listener.get() {
            listener.stop();
        }
        self.inner.pool.close();
        // The closed pool still counts checked-out objects. Keep the generation's
        // namespace lease until their queries/statements have returned and detached.
        let inner = self.inner.clone();
        self.inner.runtime.spawn(async move {
            let lease = inner.schema_lease.lock().await.take();
            while inner.pool.status().size > 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            drop(lease);
            if let Some(connection) = inner
                .lease_abort
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                connection.abort();
            }
        });
    }

    pub(crate) fn acquire_timeout(&self) -> Duration {
        self.inner.connect_timeout
    }

    /// The connection target, without credentials.
    pub fn target(&self) -> &Target {
        &self.inner.target
    }

    /// The server's version.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn server(&self) -> Result<ServerInfo, OperationsError> {
        let client = self.admin_client().await?;
        let server = statements::server_version()
            .bind(&client)
            .one()
            .await
            .classify(self.target())?;
        Ok(ServerInfo {
            version_num: server.version_num,
            version: server.version,
        })
    }

    /// Attempts and their lifecycle.
    pub const fn attempts(&self) -> crate::attempts::Attempts<'_> {
        crate::attempts::Attempts::new(self)
    }

    /// The durable job queue.
    pub const fn jobs(&self) -> crate::jobs::Jobs<'_> {
        crate::jobs::Jobs::new(self)
    }

    /// Progress and incumbent streams.
    pub const fn streams(&self) -> crate::streams::Streams<'_> {
        crate::streams::Streams::new(self)
    }

    /// The solution and warm-start store.
    pub const fn solutions(&self) -> crate::solutions::Solutions<'_> {
        crate::solutions::Solutions::new(self)
    }

    /// Content-addressed authored source bundles for job execution.
    pub const fn sources(&self) -> crate::sources::Sources<'_> {
        crate::sources::Sources::new(self)
    }

    /// The study repository (Plan 22 O7).
    pub const fn studies(&self) -> crate::studies::Studies<'_> {
        crate::studies::Studies::new(self)
    }

    /// The publication catalog and reader leases.
    pub const fn catalog(&self) -> crate::catalog::Catalog<'_> {
        crate::catalog::Catalog::new(self)
    }
}
