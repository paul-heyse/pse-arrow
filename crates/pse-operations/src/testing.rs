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
        let row = self
            .client
            .query_one(sql, &[])
            .await
            .classify(&self.target)?;
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
        admin
            .execute(&format!("CREATE DATABASE \"{name}\""))
            .await?;
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
        let drop = format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", self.name);
        // FORCE refuses to terminate a server background worker (autovacuum) attached to
        // the database, whose role this user may not signal. Such a worker leaves within
        // moments, so that refusal is retried.
        let mut attempts = 0;
        loop {
            match self.admin.client.simple_query(&drop).await {
                Err(error)
                    if attempts < 50
                        && error.code()
                            == Some(&tokio_postgres::error::SqlState::INSUFFICIENT_PRIVILEGE) =>
                {
                    attempts += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                result => {
                    result.classify(&self.admin.target)?;
                    return Ok(());
                }
            }
        }
    }
}

/// Where an armed [`FaultProxy`] cuts a connection, relative to its transaction's
/// `COMMIT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    /// Drop the connection instead of forwarding `COMMIT`: the server aborts the
    /// transaction and the client sees a lost connection.
    BeforeCommit,
    /// Forward `COMMIT`, wait until the server answered it, then drop the connection
    /// without relaying the answer: the transaction committed and the client sees a lost
    /// acknowledgement.
    AfterCommit,
}

/// A one-shot fault: the first connection that sends a statement containing `marker`
/// (a `Parse` or simple query) is cut at `point` of its next `COMMIT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault {
    /// Statement text that arms the connection, for example `INSERT INTO
    /// pse_ops.publications `.
    pub marker: &'static str,
    /// Where the connection is cut.
    pub point: FaultPoint,
}

#[derive(Debug, Default)]
struct FaultState {
    armed: std::sync::Mutex<Option<Fault>>,
    fired: std::sync::atomic::AtomicBool,
}

/// A TCP proxy in front of the test server that cuts one connection around its `COMMIT`
/// (Plan 22 O8.3), so a lost commit acknowledgement is reproducible. Clients connect to
/// [`FaultProxy::url`]; the proxy relays each connection to the server's socket and reads
/// the frontend messages only to find the arming statement and the `COMMIT`.
#[derive(Debug)]
pub struct FaultProxy {
    url: String,
    state: std::sync::Arc<FaultState>,
    listener: JoinHandle<()>,
}

impl Drop for FaultProxy {
    fn drop(&mut self) {
        self.listener.abort();
    }
}

/// Where the proxy reaches the server.
#[derive(Clone, Debug)]
enum Upstream {
    #[cfg(unix)]
    Socket(std::path::PathBuf),
    Tcp(String, u16),
}

/// One relayed stream, either kind of socket.
trait Relay: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static> Relay for T {}

impl Upstream {
    async fn connect(&self) -> std::io::Result<Box<dyn Relay>> {
        Ok(match self {
            #[cfg(unix)]
            Self::Socket(path) => Box::new(tokio::net::UnixStream::connect(path).await?),
            Self::Tcp(host, port) => {
                Box::new(tokio::net::TcpStream::connect((host.as_str(), *port)).await?)
            }
        })
    }
}

impl FaultProxy {
    /// Listen on a loopback port in front of the server `url` names.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Configuration`] for a URL without a host; a failure to listen.
    pub async fn start(url: &str) -> Result<Self, OperationsError> {
        let config = crate::store::configure(url, &StoreOptions::for_tests())?;
        let port = config.get_ports().first().copied().unwrap_or(5432);
        let upstream = match config.get_hosts().first() {
            #[cfg(unix)]
            Some(tokio_postgres::config::Host::Unix(directory)) => {
                Upstream::Socket(directory.join(format!(".s.PGSQL.{port}")))
            }
            Some(tokio_postgres::config::Host::Tcp(host)) => Upstream::Tcp(host.clone(), port),
            None => {
                return Err(OperationsError::Configuration {
                    reason: "the fault proxy needs a URL naming the server".to_owned(),
                });
            }
        };
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .map_err(|error| OperationsError::Configuration {
                reason: format!("fault proxy cannot listen: {error}"),
            })?;
        let address = listener
            .local_addr()
            .map_err(|error| OperationsError::Configuration {
                reason: format!("fault proxy has no address: {error}"),
            })?;
        let mut proxied = url::Url::parse("postgres://127.0.0.1/").map_err(|error| {
            OperationsError::Configuration {
                reason: error.to_string(),
            }
        })?;
        let _ = proxied.set_port(Some(address.port()));
        proxied.set_path(&format!("/{}", config.get_dbname().unwrap_or("postgres")));
        if let Some(user) = config.get_user() {
            let _ = proxied.set_username(user);
        }
        let state = std::sync::Arc::new(FaultState::default());
        let shared = std::sync::Arc::clone(&state);
        let listener = tokio::spawn(async move {
            while let Ok((client, _)) = listener.accept().await {
                let upstream = upstream.clone();
                let state = std::sync::Arc::clone(&shared);
                tokio::spawn(async move {
                    if let Ok(server) = upstream.connect().await {
                        relay(Box::new(client), server, state).await;
                    }
                });
            }
        });
        Ok(Self {
            url: proxied.to_string(),
            state,
            listener,
        })
    }

    /// The URL clients connect to.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Arm a one-shot fault; it replaces any fault not yet fired.
    pub fn arm(&self, fault: Fault) {
        if let Ok(mut armed) = self.state.armed.lock() {
            *armed = Some(fault);
        }
        self.state
            .fired
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Whether the armed fault cut a connection.
    pub fn fired(&self) -> bool {
        self.state.fired.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// The frontend message framing: untyped messages (SSL/GSS negotiation, startup) until
/// the startup message, then a type byte, a length and the payload.
struct Frames {
    buffer: Vec<u8>,
    typed: bool,
}

impl Frames {
    /// The next complete message as (type, whole message bytes), if buffered.
    fn next(&mut self) -> Option<(Option<u8>, Vec<u8>)> {
        let offset = usize::from(self.typed);
        if self.buffer.len() < offset + 4 {
            return None;
        }
        let length = u32::from_be_bytes([
            self.buffer[offset],
            self.buffer[offset + 1],
            self.buffer[offset + 2],
            self.buffer[offset + 3],
        ]);
        let total = offset + usize::try_from(length).ok()?;
        if self.buffer.len() < total {
            return None;
        }
        let message: Vec<u8> = self.buffer.drain(..total).collect();
        if self.typed {
            Some((message.first().copied(), message))
        } else {
            // SSLRequest (80877103) and GSSENCRequest (80877104) precede the startup
            // message; everything after the startup message is typed.
            let code = message
                .get(4..8)
                .map(|code| u32::from_be_bytes([code[0], code[1], code[2], code[3]]));
            if !matches!(code, Some(80_877_103 | 80_877_104)) {
                self.typed = true;
            }
            Some((None, message))
        }
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Relay one connection, cutting it once if the armed fault matches.
async fn relay(client: Box<dyn Relay>, server: Box<dyn Relay>, state: std::sync::Arc<FaultState>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (mut client_read, mut client_write) = tokio::io::split(client);
    let (mut server_read, mut server_write) = tokio::io::split(server);
    // Set when the backend's answers must no longer reach the client.
    let swallow = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let answered = std::sync::Arc::new(tokio::sync::Notify::new());
    let backend = {
        let swallow = std::sync::Arc::clone(&swallow);
        let answered = std::sync::Arc::clone(&answered);
        tokio::spawn(async move {
            let mut chunk = vec![0; 16 * 1024];
            while let Ok(read) = server_read.read(&mut chunk).await {
                if read == 0 {
                    break;
                }
                if swallow.load(std::sync::atomic::Ordering::SeqCst) {
                    // The server answered the forwarded COMMIT; the client never learns it.
                    answered.notify_one();
                    break;
                }
                if client_write.write_all(&chunk[..read]).await.is_err() {
                    break;
                }
            }
            let _ = client_write.shutdown().await;
        })
    };
    let mut frames = Frames {
        buffer: Vec::new(),
        typed: false,
    };
    let mut chunk = vec![0; 16 * 1024];
    let mut marked = false;
    'relay: while let Ok(read) = client_read.read(&mut chunk).await {
        if read == 0 {
            break;
        }
        frames.buffer.extend_from_slice(&chunk[..read]);
        while let Some((kind, message)) = frames.next() {
            let fault = state.armed.lock().ok().and_then(|armed| *armed);
            if let Some(fault) = fault {
                if matches!(kind, Some(b'P' | b'Q')) && contains(&message, fault.marker.as_bytes())
                {
                    marked = true;
                }
                let commit = kind == Some(b'Q') && contains(&message, b"COMMIT");
                if marked && commit {
                    let taken = state
                        .armed
                        .lock()
                        .ok()
                        .and_then(|mut armed| armed.take())
                        .is_some();
                    if taken {
                        state.fired.store(true, std::sync::atomic::Ordering::SeqCst);
                        if fault.point == FaultPoint::AfterCommit {
                            swallow.store(true, std::sync::atomic::Ordering::SeqCst);
                            let _ = server_write.write_all(&message).await;
                            let _ = tokio::time::timeout(
                                std::time::Duration::from_secs(10),
                                answered.notified(),
                            )
                            .await;
                        }
                        break 'relay;
                    }
                }
            }
            if server_write.write_all(&message).await.is_err() {
                break 'relay;
            }
        }
    }
    let _ = server_write.shutdown().await;
    backend.abort();
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
