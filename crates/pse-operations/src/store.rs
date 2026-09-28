// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The store handle: a connection pool plus the embedded migrator.

use std::str::FromStr;
use std::time::Duration;

use sqlx::migrate::Migrate;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};

use crate::MIGRATOR;
use crate::error::{Classify, OperationsError, Target};

/// The environment variable naming the operational store (ADR-0112 Outcome 20).
pub const DATABASE_URL_ENV: &str = "PSE_DATABASE_URL";

/// The development default: database `pse` over the local Unix socket with peer
/// authentication, so no credential exists anywhere (`docs/dev/operational-store.md`).
pub const DEFAULT_DATABASE_URL: &str = "postgres:///pse?host=/var/run/postgresql";

/// The oldest supported server, as `server_version_num` (PostgreSQL 18: `uuidv7()`).
pub const MINIMUM_SERVER_VERSION: i32 = 180_000;

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
    /// How long to wait for a pooled connection.
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

/// Embedded migrations compared with what the database recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MigrationStatus {
    /// Embedded migrations the database has applied with the same checksum.
    pub applied: Vec<i64>,
    /// Embedded migrations not yet applied, with their descriptions.
    pub pending: Vec<(i64, String)>,
    /// Applied migrations whose embedded checksum differs: an edited migration.
    pub mismatched: Vec<i64>,
    /// Applied migrations this build does not embed: a newer schema than this build.
    pub unknown: Vec<i64>,
    /// A migration that failed part way, if any.
    pub dirty: Option<i64>,
}

impl MigrationStatus {
    /// Nothing pending, edited, unknown or dirty.
    pub const fn is_current(&self) -> bool {
        self.pending.is_empty()
            && self.mismatched.is_empty()
            && self.unknown.is_empty()
            && self.dirty.is_none()
    }
}

/// A handle on the operational store. Cheap to clone; clones share the pool.
#[derive(Clone, Debug)]
pub struct Store {
    pool: PgPool,
    target: Target,
}

impl Store {
    /// Connect with default [`StoreOptions`] and verify the server version.
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
        let connect = PgConnectOptions::from_str(url)
            .map_err(|error| OperationsError::Configuration {
                reason: format!("invalid operational store URL: {error}"),
            })?
            .application_name("pse-operations")
            .options([(
                "idle_in_transaction_session_timeout",
                format!("{}ms", options.idle_in_transaction_timeout.as_millis()),
            )]);
        let target = Target::describe(&connect);
        let pool = PgPoolOptions::new()
            .max_connections(options.max_connections)
            .acquire_timeout(options.acquire_timeout)
            .connect_with(connect)
            .await
            .classify(&target)?;
        let store = Self { pool, target };
        let server = store.server().await?;
        if !server.is_supported() {
            return Err(OperationsError::Configuration {
                reason: format!(
                    "{} runs PostgreSQL {}; the operational store requires 18 or newer",
                    store.target, server.version
                ),
            });
        }
        Ok(store)
    }

    /// Wrap an existing pool, e.g. the isolated database of a `#[sqlx::test]`.
    pub fn from_pool(pool: PgPool) -> Self {
        let target = Target::describe(&pool.connect_options());
        Self { pool, target }
    }

    /// The pool, for callers that compose their own transactions.
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// The connection target, without credentials.
    pub const fn target(&self) -> &Target {
        &self.target
    }

    /// The server's version.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn server(&self) -> Result<ServerInfo, OperationsError> {
        let (version_num, version): (String, String) = sqlx::query_as(
            "SELECT current_setting('server_version_num'), current_setting('server_version')",
        )
        .fetch_one(&self.pool)
        .await
        .classify(&self.target)?;
        let version_num = version_num
            .parse()
            .map_err(|_| OperationsError::CorruptValue {
                column: "server_version_num",
                detail: version_num.clone(),
            })?;
        Ok(ServerInfo {
            version_num,
            version,
        })
    }

    /// Apply every pending embedded migration. Concurrent callers serialize on the
    /// migrator's advisory lock.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Migration`] when a migration fails or an applied one was edited.
    pub async fn migrate(&self) -> Result<(), OperationsError> {
        MIGRATOR
            .run(&self.pool)
            .await
            .map_err(|source| OperationsError::Migration {
                target: self.target.clone(),
                source,
            })
    }

    /// Compare the embedded migrations with the database without changing it.
    ///
    /// # Errors
    ///
    /// Classified driver failures and [`OperationsError::Migration`].
    pub async fn migration_status(&self) -> Result<MigrationStatus, OperationsError> {
        let table = MIGRATOR.table_name.as_ref();
        let mut conn = self.pool.acquire().await.classify(&self.target)?;
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(&mut *conn)
            .await
            .classify(&self.target)?;
        let migration_error = |source| OperationsError::Migration {
            target: self.target.clone(),
            source,
        };
        let (recorded, dirty) = if exists {
            let dirty = conn.dirty_version(table).await.map_err(migration_error)?;
            let recorded = conn
                .list_applied_migrations(table)
                .await
                .map_err(migration_error)?;
            (recorded, dirty)
        } else {
            (Vec::new(), None)
        };
        let mut status = MigrationStatus {
            dirty,
            ..MigrationStatus::default()
        };
        for migration in MIGRATOR
            .iter()
            .filter(|m| !m.migration_type.is_down_migration())
        {
            match recorded.iter().find(|r| r.version == migration.version) {
                Some(applied) if applied.checksum == migration.checksum => {
                    status.applied.push(migration.version);
                }
                Some(_) => status.mismatched.push(migration.version),
                None => status
                    .pending
                    .push((migration.version, migration.description.to_string())),
            }
        }
        status.unknown = recorded
            .iter()
            .map(|r| r.version)
            .filter(|version| !MIGRATOR.version_exists(*version))
            .collect();
        Ok(status)
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
