// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated, migrated databases for tests in crates that must not use sqlx directly
//! (feature `test-support`). Like `#[sqlx::test]`, each database is created on the server
//! that `DATABASE_URL` names (`just` exports it), so a test never touches the development
//! store. A failed test leaves its database for inspection; [`TestDatabase::remove`] drops
//! it.

use crate::error::{Classify, OperationsError};
use crate::store::Store;

/// One isolated database with the embedded migrations applied.
#[derive(Debug)]
pub struct TestDatabase {
    store: Store,
    url: String,
    admin: Store,
    name: String,
}

/// One test connection outside the pool. Statements are test-authored literals.
#[derive(Debug)]
pub struct Session {
    connection: tokio::sync::Mutex<sqlx::PgConnection>,
    target: crate::error::Target,
}

impl Session {
    /// Run one statement; returns the rows it affected.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn execute(&self, sql: &str) -> Result<u64, OperationsError> {
        let mut connection = self.connection.lock().await;
        Ok(sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&mut *connection)
            .await
            .classify(&self.target)?
            .rows_affected())
    }

    /// Run a query returning one `bigint`.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn count(&self, sql: &str) -> Result<i64, OperationsError> {
        let mut connection = self.connection.lock().await;
        sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&mut *connection)
            .await
            .classify(&self.target)
    }
}

/// The server URL tests use: `DATABASE_URL`, as `#[sqlx::test]` requires.
fn server_url() -> Result<String, OperationsError> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .ok_or_else(|| OperationsError::Configuration {
            reason: "DATABASE_URL must name the test server (`just` exports it)".to_owned(),
        })
}

impl TestDatabase {
    /// Create and migrate a fresh database named `pse_test_<uuid>`.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Configuration`] without `DATABASE_URL`; connection, creation and
    /// migration failures.
    pub async fn create() -> Result<Self, OperationsError> {
        let server = server_url()?;
        let admin = Store::connect(&server).await?;
        let name = format!("pse_test_{}", uuid::Uuid::now_v7().simple());
        // The name is minted here from a UUID; it carries no caller text.
        sqlx::query(sqlx::AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
            .execute(admin.pool())
            .await
            .classify(admin.target())?;
        let mut url = url::Url::parse(&server).map_err(|error| OperationsError::Configuration {
            reason: format!("DATABASE_URL is not a URL: {error}"),
        })?;
        url.set_path(&format!("/{name}"));
        let url = url.to_string();
        let store = Store::connect(&url).await?;
        store.migrate().await?;
        Ok(Self {
            store,
            url,
            admin,
            name,
        })
    }

    /// The migrated store.
    pub const fn store(&self) -> &Store {
        &self.store
    }

    /// The connection URL, for child processes.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// A dedicated connection for raw SQL, for tests that hold locks in an open
    /// transaction or probe the server.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn session(&self) -> Result<Session, OperationsError> {
        let connection = self
            .store
            .pool()
            .acquire()
            .await
            .classify(self.store.target())?
            .detach();
        Ok(Session {
            connection: tokio::sync::Mutex::new(connection),
            target: self.store.target().clone(),
        })
    }

    /// Close every connection and drop the database.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn remove(self) -> Result<(), OperationsError> {
        self.store.pool().close().await;
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.name
        )))
        .execute(self.admin.pool())
        .await
        .classify(self.admin.target())?;
        Ok(())
    }
}
