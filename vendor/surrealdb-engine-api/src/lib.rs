//! The service-provider interface between the SurrealDB Rust SDK and the
//! engines it drives.
//!
//! An engine owns a datastore or a connection to one, and serves
//! [`SurrealEngine`] to the SDK. Some run as a task the SDK hands a [`Route`]
//! per request, answering on the route's response channel; session lifetime
//! travels alongside on a separate [`SessionId`] channel either way. This crate
//! holds exactly the types that cross that boundary, so an engine can live in
//! its own crate without the SDK depending on it, or on anything it in turn
//! depends on.
//!
//! # Stability
//!
//! This is an internal interface between crates released together. It carries
//! no stability guarantee and may change in any release, including a patch
//! release. Depend on it only if you implement an engine; application code
//! should use the [`surrealdb`](https://docs.rs/surrealdb) crate.

use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use async_channel::Sender;
pub use surrealdb_rpc::QUERY_STREAM_BUFFER;
pub use surrealdb_rpc::export::Config as DbExportConfig;
use surrealdb_rpc::{QueryResult, QueryStreamItem, Token, items_for_result};
use surrealdb_types::{
    Array, ConnectionError, Error, NotFoundError, Notification, Object, SurrealValue, Value,
    Variables,
};
use uuid::Uuid;

pub mod request;
pub mod session;
pub use request::{DispatchState, RequestContext};

pub use session::{Established, SessionEntry, SessionRegistry};

/// A future boxed for storage behind a trait object, as
/// [`SurrealEngine`]'s methods require.
///
/// `Send` but deliberately not `Sync`: an engine may await a future from a
/// client library that is not itself `Sync` (tonic's are not), and requiring
/// `Sync` here would rule those engines out entirely. Nothing polls one of
/// these from two threads at once, so `Sync` buys nothing.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A request travelling from the SDK to an engine, tagged with the session it
/// belongs to.
#[derive(Debug)]
pub struct RequestData {
    /// The operation the engine should perform.
    pub command: Command,
    /// The session the command runs under.
    pub session_id: Uuid,
    /// Original caller lifecycle, when explicitly supplied.
    pub request_context: Option<RequestContext>,
}

/// A request paired with the channel its response must be sent on.
#[derive(Debug)]
pub struct Route {
    /// Admission remains owned until response or physical driver drain.
    pub admission: Option<std::sync::Arc<request::AdmissionPermit>>,
    /// The request to execute.
    pub request: RequestData,
    /// Where the engine sends the outcome. Exactly one message is expected.
    pub response: Sender<Result<Vec<QueryResult>, Error>>,
}

/// A session-lifetime event. Engines keep per-session state (the authenticated
/// session, its variables, its live queries), and these events tell them when
/// to create, copy and discard it.
#[derive(Debug, Clone, Copy)]
pub enum SessionId {
    /// A new session was opened.
    Initial(Uuid),
    /// A session was cloned; `new` starts as a copy of `old`.
    Clone {
        /// The session being copied.
        old: Uuid,
        /// The session receiving the copy.
        new: Uuid,
    },
    /// A session was dropped and its state can be released.
    Drop(Uuid),
}

/// Why a route could not be matched to a session.
#[derive(Debug, Clone)]
pub enum SessionError {
    /// No state is registered for the session the route targets.
    NotFound(Uuid),
    /// The remote end reported a session failure.
    Remote(String),
    /// Bounded session failure retaining the exact remote typed error.
    RemoteTyped(Box<Error>),
}

impl From<SessionError> for Error {
    fn from(error: SessionError) -> Self {
        session_error_to_error(error)
    }
}

/// Convert a session error into the error type the SDK surfaces to callers.
pub fn session_error_to_error(e: SessionError) -> Error {
    match e {
        SessionError::NotFound(id) => Error::not_found(
            format!("Session not found: {id}"),
            NotFoundError::Session {
                id: Some(id.to_string()),
            },
        ),
        SessionError::Remote(msg) => Error::internal(msg),
        SessionError::RemoteTyped(error) => *error,
    }
}

/// Which machine learning model to export, by name and version.
#[derive(Debug, Clone)]
pub struct MlExportConfig {
    /// The model name.
    pub name: String,
    /// The model version.
    pub version: String,
}

/// The operations an engine can be asked to perform.
#[derive(Debug, Clone)]
pub enum Command {
    /// Select the namespace and/or database for the session.
    Use {
        /// The namespace to use, or `None` to leave it unchanged.
        namespace: Option<String>,
        /// The database to use, or `None` to leave it unchanged.
        database: Option<String>,
    },
    /// Sign up as a record user and authenticate the session.
    Signup {
        /// The signup credentials.
        credentials: Object,
    },
    /// Sign in and authenticate the session.
    Signin {
        /// The signin credentials.
        credentials: Object,
    },
    /// Authenticate the session with an existing token.
    Authenticate {
        /// The token to authenticate with.
        token: Token,
    },
    /// Exchange a refresh token for a fresh token pair.
    Refresh {
        /// The token carrying the refresh token.
        token: Token,
    },
    /// Drop the session's authentication.
    Invalidate,
    /// Begin a manual transaction.
    Begin,
    /// Cancel a manual transaction.
    Rollback {
        /// The transaction to cancel.
        txn: Uuid,
    },
    /// Commit a manual transaction.
    Commit {
        /// The transaction to commit.
        txn: Uuid,
    },
    /// Invalidate a refresh token so it can no longer be redeemed.
    Revoke {
        /// The token carrying the refresh token.
        token: Token,
    },
    /// Execute a query, optionally inside a manual transaction.
    Query {
        /// The transaction to run in, or `None` for an implicit one.
        txn: Option<Uuid>,
        /// The query text.
        query: Cow<'static, str>,
        /// The variables bound for this query only.
        variables: Variables,
    },
    /// Export the database to a file.
    ExportFile {
        /// The file to write.
        path: PathBuf,
        /// What to include in the export.
        config: Option<DbExportConfig>,
    },
    /// Export a machine learning model to a file.
    ExportMl {
        /// The file to write.
        path: PathBuf,
        /// The model to export.
        config: MlExportConfig,
    },
    /// Export the database to a channel.
    ExportBytes {
        /// Where the export is streamed.
        bytes: Sender<Result<Vec<u8>, Error>>,
        /// What to include in the export.
        config: Option<DbExportConfig>,
    },
    /// Export a machine learning model to a channel.
    ExportBytesMl {
        /// Where the export is streamed.
        bytes: Sender<Result<Vec<u8>, Error>>,
        /// The model to export.
        config: MlExportConfig,
    },
    /// Import a database export from a file.
    ImportFile {
        /// The file to read.
        path: PathBuf,
    },
    /// Import a machine learning model from a file.
    ImportMl {
        /// The file to read.
        path: PathBuf,
    },
    /// Check that the engine is reachable.
    Health,
    /// Report the database version.
    Version,
    /// Set a session variable.
    Set {
        /// The variable name.
        key: String,
        /// The value to bind, or `Value::None` to remove it.
        value: Value,
    },
    /// Remove a session variable.
    Unset {
        /// The variable name.
        key: String,
    },
    /// Register the channel a live query's notifications are delivered on.
    SubscribeLive {
        /// The live query.
        uuid: Uuid,
        /// Where notifications are delivered.
        notification_sender: Sender<Result<Notification, Error>>,
    },
    /// Kill a live query.
    Kill {
        /// The live query.
        uuid: Uuid,
    },
    /// Adopt an existing remote session.
    Attach {
        /// The session to adopt.
        session_id: Uuid,
    },
    /// Release a remote session without ending it.
    Detach {
        /// The session to release.
        session_id: Uuid,
    },
    /// Call a function or a machine learning model.
    Run {
        /// The function or model name.
        name: String,
        /// The model version, if any.
        version: Option<String>,
        /// The arguments to pass.
        args: Array,
    },
}

/// Which session, and which explicit transaction, a request applies to.
///
/// Mirrors the `RequestContext` every request carries in the SurrealDB
/// network protocol, minus the fields the SDK does not populate: an engine
/// applies its own configured timeouts rather than being told them per call.
#[derive(Debug, Clone)]
pub struct EngineContext {
    /// Original caller lifecycle, shared through the request route.
    pub request_context: Option<RequestContext>,
    /// The session the request runs under.
    pub session: Uuid,
    /// The explicit transaction to run in, or `None` for an implicit one.
    ///
    /// Only meaningful on [`SurrealEngine::query`]; the transaction-lifecycle
    /// methods take the transaction they act on as an explicit argument.
    pub transaction: Option<Uuid>,
}

impl EngineContext {
    /// Attach the unchanged original request lifecycle.
    pub fn with_request_context(mut self, context: Option<RequestContext>) -> Self {
        self.request_context = context;
        self
    }

    /// A context for a request outside any explicit transaction.
    pub fn new(session: Uuid) -> Self {
        Self {
            session,
            transaction: None,
            request_context: None,
        }
    }

    /// A context for a request inside the given explicit transaction.
    pub fn with_transaction(session: Uuid, transaction: Option<Uuid>) -> Self {
        Self {
            session,
            transaction,
            request_context: None,
        }
    }
}

/// A boxed engine result.
pub type EngineFuture<'a, T> = BoxFuture<'a, Result<T, Error>>;

/// Answers a streaming caller from an engine's buffered `query`.
///
/// The caller sees the same items either way, just all at once. This is the
/// right answer for any engine whose transport cannot carry results before the
/// query ends -- which is most of them.
fn buffered_query_stream<'a, E>(
    engine: &'a E,
    ctx: EngineContext,
    query: Cow<'static, str>,
    variables: Variables,
    items: Sender<QueryStreamItem>,
) -> EngineFuture<'a, ()>
where
    E: SurrealEngine + ?Sized,
{
    Box::pin(async move {
        for (index, result) in engine
            .query(ctx, query, variables)
            .await?
            .into_iter()
            .enumerate()
        {
            for item in items_for_result(index, result) {
                // A receiver that has gone away wants no more items, and there
                // is nothing else to do with them.
                if items.send(item).await.is_err() {
                    return Ok(());
                }
            }
        }
        Ok(())
    })
}

/// The interface every SurrealDB engine implements, and the only thing the
/// Rust SDK calls to reach a database.
///
/// One method per operation, each taking and returning the types that
/// operation actually deals in, so no engine has to encode a result into a
/// generic [`Value`] purely for the SDK to take it apart again. An embedded
/// engine hands its own values straight back; a remote engine converts once,
/// from its wire format.
///
/// Methods for capabilities an engine may not have -- live queries, export
/// and import -- default to reporting that they are unsupported, so an engine
/// implements only what it serves. The SDK gates most of these on
/// `ExtraFeatures` before calling, so the default is a backstop rather than
/// the usual path.
///
/// # Stability
///
/// This is an internal interface between crates released together. It carries
/// no stability guarantee and may change in any release, including a patch
/// release.
pub trait SurrealEngine: Debug + Send + Sync + 'static {
    /// Close the physical connection and await driver shutdown.
    fn disconnect(&self) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Physical disconnect")) })
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    /// Executes SurrealQL, returning one result per statement, in order.
    fn query(
        &self,
        ctx: EngineContext,
        query: Cow<'static, str>,
        variables: Variables,
    ) -> EngineFuture<'_, Vec<QueryResult>>;

    /// Executes SurrealQL, sending results into `items` as they are produced.
    ///
    /// The returned future is the execution: drive it while draining `items`,
    /// and treat the channel closing as "no more results" rather than as
    /// success, since a failure that belongs to no single statement is reported
    /// by the future.
    ///
    /// The default answers from [`Self::query`] and replays the finished
    /// results, which is what an engine whose transport cannot carry
    /// incremental results should do — the caller sees the same items either
    /// way, just all at once. Overriding it is worthwhile only where results
    /// can actually reach the caller before the query ends.
    ///
    /// Rows are provisional until their statement's
    /// [`Finished`](QueryStreamItem::Finished) item arrives; see
    /// [`QueryStreamItem`].
    fn query_stream(
        &self,
        ctx: EngineContext,
        query: Cow<'static, str>,
        variables: Variables,
        items: Sender<QueryStreamItem>,
    ) -> EngineFuture<'_, ()> {
        buffered_query_stream(self, ctx, query, variables, items)
    }

    /// Calls a function, or a machine learning model when `version` is set.
    fn run(
        &self,
        ctx: EngineContext,
        name: String,
        version: Option<String>,
        args: Array,
    ) -> EngineFuture<'_, Value>;

    // ------------------------------------------------------------------
    // Session state
    // ------------------------------------------------------------------

    /// Selects the namespace and/or database, returning the resulting
    /// selection. `None` for either argument leaves that one unchanged.
    fn use_ns_db(
        &self,
        ctx: EngineContext,
        namespace: Option<String>,
        database: Option<String>,
    ) -> EngineFuture<'_, (Option<String>, Option<String>)>;

    /// Binds a session variable.
    fn set(&self, ctx: EngineContext, key: String, value: Value) -> EngineFuture<'_, ()>;

    /// Removes a session variable.
    fn unset(&self, ctx: EngineContext, key: String) -> EngineFuture<'_, ()>;

    // ------------------------------------------------------------------
    // Authentication
    // ------------------------------------------------------------------

    /// Registers a record user and authenticates the session as them.
    fn signup(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token>;

    /// Authenticates the session with credentials.
    fn signin(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token>;

    /// Authenticates the session with an existing token, returning the token
    /// now in effect.
    ///
    /// A server may hand back a token of its own rather than the one it was
    /// given, so the result is what the session is authenticated with -- not
    /// necessarily the argument.
    fn authenticate(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token>;

    /// Exchanges a refresh token for a fresh token pair.
    fn refresh(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token>;

    /// Invalidates a refresh token so it can no longer be redeemed.
    fn revoke(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, ()>;

    /// Drops the session's authentication.
    fn invalidate(&self, ctx: EngineContext) -> EngineFuture<'_, ()>;

    // ------------------------------------------------------------------
    // Transactions
    // ------------------------------------------------------------------

    /// Opens an explicit transaction, returning its id.
    fn begin(&self, ctx: EngineContext) -> EngineFuture<'_, Uuid>;

    /// Commits an explicit transaction.
    fn commit(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()>;

    /// Cancels an explicit transaction.
    fn rollback(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()>;

    // ------------------------------------------------------------------
    // Connection
    // ------------------------------------------------------------------

    /// Checks that the engine is reachable.
    fn health(&self, ctx: EngineContext) -> EngineFuture<'_, ()>;

    /// Reports the database version, as the server spells it (for example
    /// `surrealdb-3.0.0`).
    fn version(&self, ctx: EngineContext) -> EngineFuture<'_, String>;

    // ------------------------------------------------------------------
    // Live queries
    // ------------------------------------------------------------------

    /// Registers the channel a live query's notifications are delivered on.
    fn subscribe_live(
        &self,
        _ctx: EngineContext,
        _uuid: Uuid,
        _notifications: Sender<Result<Notification, Error>>,
    ) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Live queries")) })
    }

    /// Kills a live query.
    fn kill(&self, _ctx: EngineContext, _uuid: Uuid) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Live queries")) })
    }

    // ------------------------------------------------------------------
    // Export and import
    // ------------------------------------------------------------------

    /// Exports the database to a file.
    fn export_file(
        &self,
        _ctx: EngineContext,
        _path: PathBuf,
        _config: Option<DbExportConfig>,
    ) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Export")) })
    }

    /// Exports the database, streaming it to a channel.
    fn export_bytes(
        &self,
        _ctx: EngineContext,
        _bytes: Sender<Result<Vec<u8>, Error>>,
        _config: Option<DbExportConfig>,
    ) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Export")) })
    }

    /// Exports a machine learning model to a file.
    fn export_ml_file(
        &self,
        _ctx: EngineContext,
        _path: PathBuf,
        _config: MlExportConfig,
    ) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Machine learning model export")) })
    }

    /// Exports a machine learning model, streaming it to a channel.
    fn export_ml_bytes(
        &self,
        _ctx: EngineContext,
        _bytes: Sender<Result<Vec<u8>, Error>>,
        _config: MlExportConfig,
    ) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Machine learning model export")) })
    }

    /// Imports a database export from a file.
    fn import_file(&self, _ctx: EngineContext, _path: PathBuf) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Import")) })
    }

    /// Imports a machine learning model from a file.
    fn import_ml_file(&self, _ctx: EngineContext, _path: PathBuf) -> EngineFuture<'_, ()> {
        Box::pin(async { Err(unsupported("Machine learning model import")) })
    }
}

/// The error an engine reports for an operation it does not serve.
fn unsupported(what: &str) -> Error {
    Error::configuration(format!("{what} is not supported by this engine"), None)
}

/// Flattens the results of an operation that runs a single statement into the
/// one value it produced.
///
/// An empty reply reads as [`Value::None`]: an operation with no result may
/// answer with either, and both mean the same thing. Anything longer than one
/// result is a bug in the engine rather than something to report to the user.
pub fn single_result(mut results: Vec<QueryResult>) -> Result<Value, Error> {
    match results.len() {
        0 => Ok(Value::None),
        1 => results.remove(0).result,
        _ => Err(Error::internal(
            "expected the database to return one or no results".to_string(),
        )),
    }
}

/// A [`SurrealEngine`] that drives an engine which consumes [`Route`]s.
///
/// The WebSocket and HTTP engines each run a task that reads `Route`s off a
/// channel and answers on the response channel a `Route` carries -- the
/// channel is also their session replay log, so a reconnection can rebuild
/// what the connection had established. This adapter is the whole of what it
/// takes to expose one of them through [`SurrealEngine`]: it turns each typed
/// call back into the [`Command`] that task already understands, and unwraps
/// the single response into the type the method promises.
///
/// [`Command`] is therefore an implementation detail of those two engines, not
/// part of the interface: an engine with no route channel -- the embedded and
/// gRPC ones -- never constructs a `Command` at all.
#[derive(Debug, Clone)]
pub struct RouteChannelEngine {
    sender: Sender<Route>,
    shutdown: Option<async_channel::Receiver<()>>,
    application: Option<std::sync::Arc<tokio::sync::Semaphore>>,
    control: Option<std::sync::Arc<tokio::sync::Semaphore>>,
}

/// The response queue is actual byte ownership, unlike retired sender metadata.
/// Closing a channel alone leaves queued values alive while a sender exists.
/// Drain it before the immutable allocation holds are dropped, including when
/// cancellation abandons a successfully queued response.
struct ResponseReceiver<T> {
    receiver: async_channel::Receiver<T>,
    _retained: Option<request::PayloadRetention>,
}
impl<T> Drop for ResponseReceiver<T> {
    fn drop(&mut self) {
        self.receiver.close();
        while let Ok(value) = self.receiver.try_recv() {
            drop(value);
        }
    }
}

impl RouteChannelEngine {
    /// Construct a driver with an observable physical shutdown boundary.
    pub fn with_shutdown(sender: Sender<Route>, shutdown: async_channel::Receiver<()>) -> Self {
        Self {
            sender,
            shutdown: Some(shutdown),
            application: Some(std::sync::Arc::new(tokio::sync::Semaphore::new(32))),
            control: Some(std::sync::Arc::new(tokio::sync::Semaphore::new(2))),
        }
    }

    /// Wraps a route sender as a [`SurrealEngine`].
    pub fn new(sender: Sender<Route>) -> Self {
        Self {
            sender,
            shutdown: None,
            application: None,
            control: None,
        }
    }

    /// Sends one command and awaits its single response, flattening the
    /// engine's `Vec<QueryResult>` reply into the one value these
    /// non-`query` operations return.
    ///
    /// An empty reply reads as [`Value::None`]: the route protocol lets an
    /// engine answer a no-result operation with either an empty vector or a
    /// single `Value::None`, and both mean the same thing.
    fn value(
        &self,
        command: Command,
        ctx: EngineContext,
    ) -> impl std::future::Future<Output = Result<Value, Error>> + Send + '_ {
        let results = self.results(command, ctx);
        async move { single_result(results.await?) }
    }

    /// Sends one command and awaits its single response.
    fn results(
        &self,
        command: Command,
        ctx: EngineContext,
    ) -> impl std::future::Future<Output = Result<Vec<QueryResult>, Error>> + Send + '_ {
        // The tuple drops the actual command before its guard even if never polled.
        let owned = (
            command,
            request::UndispatchedPayloadDrain::caller(ctx.request_context.clone()),
        );
        async move {
            let (mut command, _drain) = owned;
            // Locals drop in reverse order: move bytes after the drain guard.
            let command = std::mem::replace(&mut command, Command::Health);
            let _lifetime = request::CancelOnDrop(ctx.request_context.clone());
            let admission = if let Some(application) = &self.application {
                let context = ctx.request_context.as_ref().ok_or_else(|| {
                    Error::configuration(
                        "Bounded requests require an original lifecycle".to_string(),
                        None,
                    )
                })?;
                let semaphore = if context.is_control() {
                    self.control.as_ref().unwrap_or(application)
                } else {
                    application
                };
                let acquire = std::sync::Arc::clone(semaphore).acquire_owned();
                let permit = tokio::select! { biased;
                    _ = context.interrupted() => return Err(context.interruption_error()),
                    result = acquire => result.map_err(|_| Error::internal("Connection admission closed".to_string()))?,
                };
                Some(std::sync::Arc::new(request::AdmissionPermit::new(
                    permit,
                    context.payload_owner(),
                )))
            } else {
                None
            };
            let retained = ctx
                .request_context
                .as_ref()
                .and_then(RequestContext::payload_owner)
                .map(|owner| owner.retain());
            let (response, receiver) = async_channel::bounded(1);
            let receiver = ResponseReceiver {
                receiver,
                _retained: retained,
            };
            let route = Route {
                admission,
                request: RequestData {
                    command,
                    session_id: ctx.session,
                    request_context: ctx.request_context.clone(),
                },
                response,
            };
            // Both failure modes mean the engine task is gone, which callers
            // distinguish from a database error with `Error::is_connection()` to
            // decide whether reconnecting is worth trying.
            let context = ctx.request_context;
            let send = self.sender.send(route);
            let admitted = if let Some(context) = &context {
                tokio::select! { biased;
                    _ = context.interrupted() => return Err(context.interruption_error()),
                    result = send => result,
                }
            } else {
                send.await
            };
            admitted.map_err(|e| {
                Error::connection(
                    format!("Failed to send command: {e}"),
                    ConnectionError::ConnectionFailed,
                )
            })?;
            if let Some(context) = &context {
                context.mark_enqueued();
            }
            let receive = receiver.receiver.recv();
            let outcome = if let Some(context) = &context {
                tokio::select! { biased;
                    _ = context.interrupted() => return Err(context.interruption_error()),
                    result = receive => result,
                }
            } else {
                receive.await
            };
            if let Some(context) = &context
                && context.payload_owner().is_some()
            {
                // A reply may wake this executor before the native response
                // handler releases its lexical snapshots. Keep receiver's
                // caller hold alive while joining only driver holds, under
                // the same cancellation and original absolute deadline.
                tokio::select! { biased;
                    _ = context.interrupted() => return Err(context.interruption_error()),
                    () = context.payload_driver_drained() => {},
                }
            }
            outcome.map_err(|_| {
                Error::connection(
                    "The engine dropped the request without answering".to_string(),
                    ConnectionError::ConnectionFailed,
                )
            })?
        }
    }

    /// Sends one command whose response carries nothing of interest.
    fn unit(
        &self,
        command: Command,
        ctx: EngineContext,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send + '_ {
        let value = self.value(command, ctx);
        async move {
            match value.await? {
                Value::None | Value::Null => Ok(()),
                Value::Array(array) if array.is_empty() => Ok(()),
                _ => Err(Error::internal(
                    "expected the database to return nothing".to_string(),
                )),
            }
        }
    }
}

/// Converts the value an engine returns for signin/signup/refresh into a
/// [`Token`].
///
/// These engines answer with the token's wire form (a bare string, or an
/// object carrying `token` and `refresh`), which is exactly what `Token`
/// deserialises from.
fn value_to_token(value: Value) -> Result<Token, Error> {
    // signin/signup answers historically arrive wrapped in a single-element
    // array from some engines; unwrap that before converting.
    let value = match value {
        Value::Array(array) if array.len() == 1 => array
            .into_iter()
            .next()
            .expect("array has exactly one element"),
        value => value,
    };
    Token::from_value(value)
}

impl SurrealEngine for RouteChannelEngine {
    fn disconnect(&self) -> EngineFuture<'_, ()> {
        Box::pin(async move {
            self.sender.close();
            if let Some(application) = &self.application {
                application.close();
            }
            if let Some(control) = &self.control {
                control.close();
            }
            if let Some(shutdown) = &self.shutdown {
                let _ = shutdown.recv().await;
            }
            Ok(())
        })
    }

    fn query(
        &self,
        ctx: EngineContext,
        query: Cow<'static, str>,
        variables: Variables,
    ) -> EngineFuture<'_, Vec<QueryResult>> {
        Box::pin(self.results(
            Command::Query {
                txn: ctx.transaction,
                query,
                variables,
            },
            ctx,
        ))
    }

    fn run(
        &self,
        ctx: EngineContext,
        name: String,
        version: Option<String>,
        args: Array,
    ) -> EngineFuture<'_, Value> {
        Box::pin(self.value(
            Command::Run {
                name,
                version,
                args,
            },
            ctx,
        ))
    }

    fn use_ns_db(
        &self,
        ctx: EngineContext,
        namespace: Option<String>,
        database: Option<String>,
    ) -> EngineFuture<'_, (Option<String>, Option<String>)> {
        Box::pin(async move {
            let value = self
                .value(
                    Command::Use {
                        namespace,
                        database,
                    },
                    ctx,
                )
                .await?;
            // Engines that predate reporting the resulting selection answer
            // with something other than an object; report "unknown" rather
            // than failing, as the SDK has always done.
            let Value::Object(object) = value else {
                return Ok((None, None));
            };
            let read = |key: &str| {
                object
                    .get(key)
                    .and_then(|v| v.as_string())
                    .map(String::from)
            };
            Ok((read("namespace"), read("database")))
        })
    }

    fn set(&self, ctx: EngineContext, key: String, value: Value) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::Set { key, value }, ctx))
    }

    fn unset(&self, ctx: EngineContext, key: String) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::Unset { key }, ctx))
    }

    fn signup(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token> {
        Box::pin(async move {
            let value = self.value(Command::Signup { credentials }, ctx).await?;
            value_to_token(value)
        })
    }

    fn signin(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token> {
        Box::pin(async move {
            let value = self.value(Command::Signin { credentials }, ctx).await?;
            value_to_token(value)
        })
    }

    fn authenticate(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token> {
        Box::pin(async move {
            let value = self.value(Command::Authenticate { token }, ctx).await?;
            value_to_token(value)
        })
    }

    fn refresh(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token> {
        Box::pin(async move {
            let value = self.value(Command::Refresh { token }, ctx).await?;
            value_to_token(value)
        })
    }

    fn revoke(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::Revoke { token }, ctx))
    }

    fn invalidate(&self, ctx: EngineContext) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::Invalidate, ctx))
    }

    fn begin(&self, ctx: EngineContext) -> EngineFuture<'_, Uuid> {
        Box::pin(async move {
            let value = self.value(Command::Begin, ctx).await?;
            let uuid = value
                .into_uuid()
                .map_err(|e| Error::internal(e.to_string()))?;
            Ok(uuid.into_inner())
        })
    }

    fn commit(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()> {
        Box::pin(async move {
            self.value(Command::Commit { txn }, ctx).await?;
            Ok(())
        })
    }

    fn rollback(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()> {
        Box::pin(async move {
            self.value(Command::Rollback { txn }, ctx).await?;
            Ok(())
        })
    }

    fn health(&self, ctx: EngineContext) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::Health, ctx))
    }

    fn version(&self, ctx: EngineContext) -> EngineFuture<'_, String> {
        Box::pin(async move {
            let value = self.value(Command::Version, ctx).await?;
            value
                .into_string()
                .map_err(|e| Error::internal(e.to_string()))
        })
    }

    fn subscribe_live(
        &self,
        ctx: EngineContext,
        uuid: Uuid,
        notifications: Sender<Result<Notification, Error>>,
    ) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(
            Command::SubscribeLive {
                uuid,
                notification_sender: notifications,
            },
            ctx,
        ))
    }

    fn kill(&self, ctx: EngineContext, uuid: Uuid) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::Kill { uuid }, ctx))
    }

    fn export_file(
        &self,
        ctx: EngineContext,
        path: PathBuf,
        config: Option<DbExportConfig>,
    ) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::ExportFile { path, config }, ctx))
    }

    fn export_bytes(
        &self,
        ctx: EngineContext,
        bytes: Sender<Result<Vec<u8>, Error>>,
        config: Option<DbExportConfig>,
    ) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::ExportBytes { bytes, config }, ctx))
    }

    fn export_ml_file(
        &self,
        ctx: EngineContext,
        path: PathBuf,
        config: MlExportConfig,
    ) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::ExportMl { path, config }, ctx))
    }

    fn export_ml_bytes(
        &self,
        ctx: EngineContext,
        bytes: Sender<Result<Vec<u8>, Error>>,
        config: MlExportConfig,
    ) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::ExportBytesMl { bytes, config }, ctx))
    }

    fn import_file(&self, ctx: EngineContext, path: PathBuf) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::ImportFile { path }, ctx))
    }

    fn import_ml_file(&self, ctx: EngineContext, path: PathBuf) -> EngineFuture<'_, ()> {
        Box::pin(self.unit(Command::ImportMl { path }, ctx))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use surrealdb_types::Value;

    use super::*;

    #[derive(Debug)]
    struct ChargedBytes {
        bytes: Vec<u8>,
        reserved: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl Drop for ChargedBytes {
        fn drop(&mut self) {
            self.reserved
                .fetch_sub(self.bytes.len(), std::sync::atomic::Ordering::AcqRel);
        }
    }
    impl ChargedBytes {
        fn reserve(
            reserved: &Arc<std::sync::atomic::AtomicUsize>,
            bytes: usize,
            limit: usize,
        ) -> Option<Arc<Self>> {
            reserved
                .try_update(
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Acquire,
                    |current| current.checked_add(bytes).filter(|next| *next <= limit),
                )
                .ok()?;
            Some(Arc::new(Self {
                bytes: vec![0; bytes],
                reserved: reserved.clone(),
            }))
        }
    }

    #[tokio::test]
    async fn queued_response_abandonment_drains_bytes_with_retired_sender_alive() {
        let (sender, routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(30))
                .with_payload_owner(ChargedBytes::reserve(&reserved, 4096, 4096).unwrap());
        let mut request = Box::pin(engine.results(
            Command::Health,
            EngineContext::new(Uuid::nil()).with_request_context(Some(context.clone())),
        ));
        let route = tokio::select! {
            route = routes.recv() => route.unwrap(),
            result = &mut request => panic!("request ended before enqueue: {result:?}"),
        };
        context.mark_dispatched();
        let retired_sender = route.response.clone();
        drop(route.request);
        route
            .response
            .send(Ok(vec![QueryResult {
                time: std::time::Duration::ZERO,
                result: Ok(Value::String("queued bytes".repeat(128))),
                query_type: surrealdb_rpc::QueryType::Other,
            }]))
            .await
            .unwrap();
        context.mark_completed();
        context.release_payload_after_drain();
        assert_eq!(retired_sender.len(), 1);
        assert!(ChargedBytes::reserve(&reserved, 4096, 4096).is_none());
        drop(request);
        assert_eq!(
            retired_sender.len(),
            0,
            "receiver cleanup destroys values, not just channel access"
        );
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = ChargedBytes::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        assert_eq!(context.dispatch_state(), request::DispatchState::Completed);
    }

    #[tokio::test]
    async fn successful_response_transfers_to_callers_actual_decode_owner() {
        let (sender, routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let caller_owner = ChargedBytes::reserve(&reserved, 4096, 4096).unwrap();
        let context =
            RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(30))
                .with_payload_owner(caller_owner.clone());
        let request = engine.results(
            Command::Health,
            EngineContext::new(Uuid::nil()).with_request_context(Some(context.clone())),
        );
        let driver = async {
            let route = routes.recv().await.unwrap();
            context.mark_dispatched();
            drop(route.request);
            route
                .response
                .send(Ok(vec![QueryResult {
                    time: std::time::Duration::ZERO,
                    result: Ok(Value::String("actual output".repeat(128))),
                    query_type: surrealdb_rpc::QueryType::Other,
                }]))
                .await
                .unwrap();
            context.mark_completed();
            context.release_payload_after_drain();
        };
        let (output, ()) = futures::future::join(request, driver).await;
        let output = output.unwrap();
        assert!(ChargedBytes::reserve(&reserved, 4096, 4096).is_none());
        drop(output);
        drop(caller_owner);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = ChargedBytes::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
    }

    #[tokio::test]
    async fn response_waits_for_gated_native_drain_and_preserves_original_clock() {
        enum Completion {
            Success,
            Cancel,
            Expire,
        }
        for completion in [Completion::Success, Completion::Cancel, Completion::Expire] {
            let (sender, routes) = async_channel::bounded(1);
            let engine = RouteChannelEngine::new(sender);
            let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let caller_owner = ChargedBytes::reserve(&reserved, 4096, 4096).unwrap();
            let budget = if matches!(completion, Completion::Expire) {
                std::time::Duration::from_millis(100)
            } else {
                std::time::Duration::from_secs(30)
            };
            let context = RequestContext::new(std::time::Instant::now() + budget)
                .with_payload_owner(caller_owner.clone());
            let original_deadline = context.deadline();
            let mut request = Box::pin(engine.results(
                Command::Health,
                EngineContext::new(Uuid::nil()).with_request_context(Some(context.clone())),
            ));
            let route = tokio::select! {
                route = routes.recv() => route.unwrap(),
                result = &mut request => panic!("request ended before enqueue: {result:?}"),
            };
            context.mark_dispatched();
            // Model the native response handler's lexical allocation snapshot.
            // The gate is this explicit guard, never a scheduler delay.
            let driver = context.payload_owner().unwrap().retain_driver();
            drop(route.request);
            context.mark_completed();
            context.release_payload_after_drain();
            route
                .response
                .try_send(Ok(vec![QueryResult {
                    time: std::time::Duration::ZERO,
                    result: Ok(Value::String("original decoded output".into())),
                    query_type: surrealdb_rpc::QueryType::Other,
                }]))
                .unwrap();
            assert!(
                futures::poll!(&mut request).is_pending(),
                "reply delivery must not expose success before native drain"
            );
            assert_eq!(context.deadline(), original_deadline);
            if !matches!(completion, Completion::Success) {
                if matches!(completion, Completion::Cancel) {
                    context.cancel();
                }
                // Expiry awaits the original request timer while the driver
                // gate stays held; no sleep, reset or second timeout is used.
                let error = request.await.unwrap_err();
                assert!(error.to_string().contains("Request interrupted"));
                assert_eq!(context.deadline(), original_deadline);
                if matches!(completion, Completion::Expire) {
                    assert!(std::time::Instant::now() >= original_deadline);
                }
                assert!(ChargedBytes::reserve(&reserved, 4096, 4096).is_none());
                drop(caller_owner);
                assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
                drop(driver);
            } else {
                drop(driver);
                let output = request.await.unwrap();
                assert_eq!(output.len(), 1);
                assert_eq!(
                    output[0].result,
                    Ok(Value::String("original decoded output".into()))
                );
                assert!(ChargedBytes::reserve(&reserved, 4096, 4096).is_none());
                drop(output);
                drop(caller_owner);
            }
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
            let restored = ChargedBytes::reserve(&reserved, 4096, 4096).unwrap();
            drop(restored);
        }
    }

    #[test]
    fn queued_values_drop_before_response_receiver_grant() {
        struct CheckedValue(Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for CheckedValue {
            fn drop(&mut self) {
                assert_eq!(self.0.load(std::sync::atomic::Ordering::Acquire), 4096);
            }
        }
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(30))
                .with_payload_owner(ChargedBytes::reserve(&reserved, 4096, 4096).unwrap());
        let retained = context.payload_owner().unwrap().retain();
        let (sender, receiver) = async_channel::bounded(1);
        let receiver = ResponseReceiver {
            receiver,
            _retained: Some(retained),
        };
        sender
            .try_send(CheckedValue(reserved.clone()))
            .ok()
            .unwrap();
        context.release_payload_after_drain();
        drop(receiver);
        assert_eq!(sender.len(), 0);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn timed_out_route_retains_actual_payload_budget_until_driver_drain() {
        let (sender, routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(4096));
        let owner = Arc::new(ChargedBytes {
            bytes: vec![0; 4096],
            reserved: reserved.clone(),
        });
        let context = request::RequestContext::new(
            std::time::Instant::now() + std::time::Duration::from_millis(20),
        )
        .with_payload_owner(owner.clone());
        drop(owner);
        let mut request = Box::pin(engine.results(
            Command::Health,
            EngineContext::new(Uuid::nil()).with_request_context(Some(context)),
        ));
        let mut route = tokio::select! {
            route = routes.recv() => route.unwrap(),
            result = &mut request => panic!("request ended before driver received it: {result:?}"),
        };
        route
            .request
            .request_context
            .as_ref()
            .unwrap()
            .mark_dispatched();
        assert!((&mut request).await.is_err());
        drop(request);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
        // Holding a routed copy is actual pending driver ownership even though
        // the caller has already observed its original deadline.
        route.request.request_context.take();
        drop(route);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn never_enqueued_payload_restores_capacity_with_live_context() {
        for mode in 0..3 {
            let blocked_admission = mode == 1;
            let (sender, routes) = async_channel::bounded(1);
            let mut engine = RouteChannelEngine::new(sender.clone());
            if blocked_admission {
                engine.application = Some(Arc::new(tokio::sync::Semaphore::new(0)));
            } else if mode == 0 {
                let (response, _receiver) = async_channel::bounded(1);
                sender
                    .send(Route {
                        admission: None,
                        request: RequestData {
                            command: Command::Health,
                            session_id: Uuid::nil(),
                            request_context: None,
                        },
                        response,
                    })
                    .await
                    .unwrap();
            } else {
                routes.close();
            }
            let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(4096));
            let owner = Arc::new(ChargedBytes {
                bytes: vec![0; 4096],
                reserved: reserved.clone(),
            });
            let context =
                RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(30))
                    .with_payload_owner(owner);
            let request = engine.results(
                Command::Query {
                    txn: None,
                    query: Cow::Borrowed("RETURN $payload"),
                    variables: Variables::default(),
                },
                EngineContext::new(Uuid::nil()).with_request_context(Some(context.clone())),
            );
            let interrupt = async {
                tokio::task::yield_now().await;
                if mode != 2 {
                    assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
                }
                context.cancel();
            };
            let (result, ()) = tokio::join!(request, interrupt);
            assert!(result.is_err());
            assert_eq!(
                context.dispatch_state(),
                request::DispatchState::NotDispatched
            );
            assert_eq!(
                reserved.load(std::sync::atomic::Ordering::Acquire),
                0,
                "actual canceled command/send future drained despite live caller metadata"
            );
            assert_eq!(routes.len(), usize::from(mode == 0));
        }
    }

    #[test]
    fn unpolled_request_drops_bytes_before_never_enqueued_guard() {
        let (sender, _routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(4096));
        let context =
            RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(30))
                .with_payload_owner(Arc::new(ChargedBytes {
                    bytes: vec![0; 4096],
                    reserved: reserved.clone(),
                }));
        let request = engine.results(
            Command::Health,
            EngineContext::new(Uuid::nil()).with_request_context(Some(context.clone())),
        );
        drop(request);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        assert_eq!(
            context.dispatch_state(),
            request::DispatchState::NotDispatched
        );
    }

    /// The buffered adaptation produces the same items a streaming engine
    /// would, so a caller cannot tell which one answered.
    #[tokio::test]
    async fn the_buffered_adaptation_produces_the_same_items() {
        let (sender, routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let (items, received) = async_channel::bounded(8);
        let stream = engine.query_stream(
            EngineContext::new(Uuid::nil()),
            Cow::Borrowed("SELECT * FROM thing"),
            Variables::default(),
            items,
        );
        let serve = async {
            let route = routes.recv().await.expect("a route");
            let _ = route
                .response
                .send(Ok(vec![QueryResult {
                    time: std::time::Duration::ZERO,
                    result: Ok(Value::Array(vec![Value::Bool(true)].into())),
                    query_type: surrealdb_rpc::QueryType::Other,
                }]))
                .await;
        };
        let (outcome, ()) = futures::future::join(stream, serve).await;
        outcome.expect("the engine answered");

        let mut items = Vec::new();
        while let Ok(item) = received.try_recv() {
            items.push(item);
        }
        assert!(
            matches!(items[0], QueryStreamItem::Rows { .. }),
            "a list becomes rows"
        );
        assert!(
            matches!(items[1], QueryStreamItem::Finished { error: None, .. }),
            "and the statement is terminated"
        );
        assert_eq!(items.len(), 2);
    }
}
