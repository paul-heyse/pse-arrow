// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated databases with the generated schema, for this crate's store tests and for
//! tests in other crates (feature `test-support`).
//!
//! Each [`TestDatabase`] is created on the server `PSE_DATABASE_URL` names (else the
//! development default), under a name minted here, so a test never touches the
//! development store. A failed test leaves its database for inspection;
//! [`TestDatabase::remove`] drops it.

use tokio::task::JoinHandle;

use crate::error::{Classify, OperationsError, Target};
use crate::store::{Store, StoreOptions, database_url_from_env};

/// One isolated database whose schema `Store::open` created from the generated DDL.
#[derive(Debug)]
pub struct TestDatabase {
    store: Store,
    url: String,
    admin: Session,
    name: String,
}

/// One test connection outside the pool. Statements are test-authored literals.
#[derive(Debug)]
pub struct Session {
    client: tokio_postgres::Client,
    connection: JoinHandle<()>,
    target: Target,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.connection.abort();
    }
}

impl Session {
    async fn connect(
        config: &tokio_postgres::Config,
        tls: &tokio_postgres_rustls::MakeRustlsConnect,
        target: &Target,
    ) -> Result<Self, OperationsError> {
        let (client, connection) = config.connect(tls.clone()).await.classify(target)?;
        let connection = tokio::spawn(async move {
            // The session ends when its client is dropped or the server closes it.
            let _ = connection.await;
        });
        Ok(Self {
            client,
            connection,
            target: target.clone(),
        })
    }

    /// Run test-authored statements with the simple query protocol; returns the rows the
    /// last one affected.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn execute(&self, sql: &str) -> Result<u64, OperationsError> {
        let messages = self.client.simple_query(sql).await.classify(&self.target)?;
        Ok(messages
            .iter()
            .rev()
            .find_map(|message| match message {
                tokio_postgres::SimpleQueryMessage::CommandComplete(rows) => Some(*rows),
                _ => None,
            })
            .unwrap_or(0))
    }

    /// Run a test-authored query returning one `bigint`.
    ///
    /// # Errors
    ///
    /// Classified driver failures, including a query that does not return one row.
    pub async fn count(&self, sql: &str) -> Result<i64, OperationsError> {
        let row = self.client.query_one(sql, &[]).await.classify(&self.target)?;
        row.try_get(0).classify(&self.target)
    }

    /// Run a test-authored query returning text columns, one `Vec` per row; NULL is `None`.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn texts(&self, sql: &str) -> Result<Vec<Vec<Option<String>>>, OperationsError> {
        let messages = self.client.simple_query(sql).await.classify(&self.target)?;
        Ok(messages
            .iter()
            .filter_map(|message| match message {
                tokio_postgres::SimpleQueryMessage::Row(row) => Some(
                    (0..row.len())
                        .map(|index| row.get(index).map(str::to_owned))
                        .collect(),
                ),
                _ => None,
            })
            .collect())
    }
}

impl TestDatabase {
    /// Create a fresh database named `pse_test_<uuid>` and open its schema.
    ///
    /// # Errors
    ///
    /// Connection, creation and schema-creation failures.
    pub async fn create() -> Result<Self, OperationsError> {
        Self::new(true).await
    }

    /// Create a fresh database without a schema; the store is connected, not opened.
    ///
    /// # Errors
    ///
    /// Connection and creation failures.
    pub async fn empty() -> Result<Self, OperationsError> {
        Self::new(false).await
    }

    async fn new(open: bool) -> Result<Self, OperationsError> {
        let server = database_url_from_env();
        let config = crate::store::configure(&server, &StoreOptions::for_tests())?;
        let target = Target::describe(&config);
        let admin = Session::connect(&config, &crate::store::tls()?, &target).await?;
        let name = format!("pse_test_{}", uuid::Uuid::now_v7().simple());
        // The name is minted here from a UUID; it carries no caller text.
        admin.execute(&format!("CREATE DATABASE \"{name}\"")).await?;
        let mut url = url::Url::parse(&server).map_err(|error| OperationsError::Configuration {
            reason: format!("{} is not a URL: {error}", crate::DATABASE_URL_ENV),
        })?;
        url.set_path(&format!("/{name}"));
        let url = url.to_string();
        let options = StoreOptions::for_tests();
        let store = if open {
            Store::open_with(&url, &options).await?
        } else {
            Store::connect_with(&url, &options).await?
        };
        Ok(Self {
            store,
            url,
            admin,
            name,
        })
    }

    /// The opened store.
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
        self.store.session().await
    }

    /// Close every connection of the store and drop the database.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn remove(self) -> Result<(), OperationsError> {
        self.store.close();
        self.admin
            .execute(&format!(
                "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
                self.name
            ))
            .await?;
        Ok(())
    }
}

impl Store {
    /// A dedicated connection outside the pool, with the store's session settings.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn session(&self) -> Result<Session, OperationsError> {
        Session::connect(self.config(), self.tls(), self.target()).await
    }
}
