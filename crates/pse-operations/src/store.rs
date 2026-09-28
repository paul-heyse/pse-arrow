// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The store handle: a deadpool pool of tokio-postgres connections (ADR-0114 Outcome 26).
//! Schema creation is `Store::open` (`schema.rs`).

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use pse_operations_queries::queries::store as statements;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::error::{Classify, OperationsError, Target};

/// The environment variable naming the operational store (ADR-0114 Outcome 21).
pub const DATABASE_URL_ENV: &str = "PSE_DATABASE_URL";

/// The development default: database `pse` over the local Unix socket with peer
/// authentication, so no credential exists anywhere (`docs/dev/operational-store.md`).
pub const DEFAULT_DATABASE_URL: &str = "postgres:///pse?host=/var/run/postgresql";

/// The oldest supported server, as `server_version_num` (ADR-0114 Outcome 21).
pub const MINIMUM_SERVER_VERSION: i32 = 180_000;

/// The `application_name` of pooled store sessions.
pub(crate) const APPLICATION: &str = "pse-operations";

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
    /// The connection configuration, for connections outside the pool (sessions).
    #[cfg_attr(
        not(any(test, feature = "test-support")),
        expect(dead_code, reason = "read by the listener task in the next Plan 22 B2.4 step")
    )]
    config: tokio_postgres::Config,
    #[cfg_attr(
        not(any(test, feature = "test-support")),
        expect(dead_code, reason = "read by the listener task in the next Plan 22 B2.4 step")
    )]
    tls: MakeRustlsConnect,
    target: Target,
    /// The superseded sqlx pool of the repositories not yet on tokio-postgres (Plan 22
    /// B2.4); deleted with sqlx in B2.5.
    sqlx: sqlx::PgPool,
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
pub(crate) fn configure(url: &str, options: &StoreOptions) -> Result<tokio_postgres::Config, OperationsError> {
    let mut config: tokio_postgres::Config =
        url.parse()
            .map_err(|error: tokio_postgres::Error| OperationsError::Configuration {
                reason: format!("invalid operational store URL: {error}"),
            })?;
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
        let manager = deadpool_postgres::Manager::from_config(
            config.clone(),
            tls.clone(),
            deadpool_postgres::ManagerConfig {
                recycling_method: deadpool_postgres::RecyclingMethod::Fast,
            },
        );
        let size = usize::try_from(options.max_connections).unwrap_or(usize::MAX);
        let pool = deadpool_postgres::Pool::builder(manager)
            .max_size(size)
            .wait_timeout(Some(options.acquire_timeout))
            .create_timeout(Some(options.acquire_timeout))
            .runtime(deadpool_postgres::Runtime::Tokio1)
            .build()
            .map_err(|error| OperationsError::Configuration {
                reason: format!("operational store pool: {error}"),
            })?;
        let sqlx = {
            use std::str::FromStr as _;
            let connect = sqlx::postgres::PgConnectOptions::from_str(url)
                .map_err(|error| OperationsError::Configuration {
                    reason: format!("invalid operational store URL: {error}"),
                })?
                .application_name(APPLICATION)
                .options([(
                    "idle_in_transaction_session_timeout",
                    format!("{}ms", options.idle_in_transaction_timeout.as_millis()),
                )]);
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(options.max_connections)
                .acquire_timeout(options.acquire_timeout)
                .connect_lazy_with(connect)
        };
        let store = Self {
            inner: Arc::new(Inner {
                pool,
                config,
                tls,
                target,
                sqlx,
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

    /// Connect with explicit options, then create or confirm the schema ([`Store::open`]).
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
        self.inner.pool.get().await.classify(self.target())
    }

    /// The superseded sqlx pool (Plan 22 B2.4; deleted in B2.5).
    pub(crate) fn pool(&self) -> &sqlx::PgPool {
        &self.inner.sqlx
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

    /// Close the pool: waiting and later acquisitions fail as unavailable, and
    /// connections are closed as they are returned.
    pub async fn close(&self) {
        self.inner.pool.close();
        // The superseded pool waits for connections a listener still holds; bounded.
        let _ = tokio::time::timeout(Duration::from_millis(200), self.inner.sqlx.close()).await;
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
        let client = self.client().await?;
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

    /// The publication catalog and reader leases.
    pub const fn catalog(&self) -> crate::catalog::Catalog<'_> {
        crate::catalog::Catalog::new(self)
    }
}
