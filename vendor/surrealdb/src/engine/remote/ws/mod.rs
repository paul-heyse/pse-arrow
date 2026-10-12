//! WebSocket engine
//!
//! This module provides WebSocket connectivity to SurrealDB servers.
//! The core logic is shared between native and WASM platforms, with
//! platform-specific implementations in the `native` and `wasm` submodules.

#[cfg(not(target_family = "wasm"))]
pub(crate) mod native;
#[cfg(target_family = "wasm")]
pub(crate) mod wasm;

use std::marker::PhantomData;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_channel::Sender;
use futures::{Sink, SinkExt};
use surrealdb_rpc::{DbResponse, DbResult, QueryResult, QueryResultBuilder, Token};
use surrealdb_types::{
    AuthError, ConnectionError, Error as TypesError, NotAllowedError, SerializationError,
    ValidationError,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::conn::{Command, RequestData, Route};
use crate::engine::remote::{RemoteCommand, RouterRequest};
use crate::engine::{SessionError, session_error_to_error};
use crate::opt::IntoEndpoint;
use crate::types::{Action, Array, HashMap, Notification, Number, SurrealValue, Value};
use crate::{Connect, Error, Surreal};

pub(crate) const PATH: &str = "rpc";
const PING_INTERVAL: Duration = Duration::from_secs(5);
/// One allowance for the entire acknowledged bounded restoration.
const RESTORATION_TIMEOUT: Duration = Duration::from_secs(5);

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone)]
struct PendingRequest {
    request_context: Option<crate::opt::RequestContext>,
    _admission: Option<Arc<surrealdb_engine_api::request::AdmissionPermit>>,
    interrupted_at: Option<std::time::Instant>,
    /// The command to register for replay on success
    command: Option<Command>,
    /// The channel to send the result of the request into.
    response_channel: Sender<Result<Vec<QueryResult>, TypesError>>,
}

/// Own terminal admission and local activity independently of retired map snapshots.
/// Construct immediately after take, before any cancellable await. Payload-owned
/// malformed replies never enter this path: their physical socket must drain.
struct TerminalCompletion {
    admission: Option<Arc<surrealdb_engine_api::request::AdmissionPermit>>,
    context: Option<crate::opt::RequestContext>,
}
impl TerminalCompletion {
    fn non_payload(pending: &PendingRequest) -> Self {
        debug_assert!(
            pending
                .request_context
                .as_ref()
                .is_none_or(|context| context.payload_owner().is_none())
        );
        Self {
            admission: pending._admission.clone(),
            context: pending.request_context.clone(),
        }
    }
}
impl Drop for TerminalCompletion {
    fn drop(&mut self) {
        if let Some(admission) = &self.admission {
            admission.release();
        }
        if let Some(context) = &self.context {
            context.release_lifetime_after_drain();
        }
    }
}

/// Per-session state for WebSocket connections
struct SessionState {
    bounded: std::sync::atomic::AtomicBool,
    /// Send requests which are still awaiting an answer.
    pending_requests: HashMap<i64, PendingRequest>,
    /// Pending live queries
    live_queries: HashMap<Uuid, Sender<crate::Result<Notification>>>,
    /// Messages which ought to be replayed on a reconnect for this session
    replay: ReplayLog,
    /// The replay in progress for this session, if any.
    ///
    /// A session's state — namespace, database, authentication, variables — is
    /// established on the server by replaying [`Self::replay`] against it.
    /// While a replay is in progress the session is only half-built and
    /// requests must not run against it: they are parked in [`Self::deferred`].
    replay_cursor: Mutex<Option<ReplayCursor>>,
    /// Requests parked until the replay in progress finishes.
    ///
    /// Only ever locked for the push or the drain itself, never across an
    /// await, and only from the single router task.
    deferred: Mutex<Vec<Route>>,
    /// The last ID used for a request
    last_id: AtomicI64,
}

/// Serialized replay storage. A bounded selection checkpoint may replace
/// only its adjacent previous completed triple; the upstream recording policy
/// remains unchanged for clients that did not opt into bounded requests.
#[derive(Default)]
struct ReplayLog(Mutex<ReplayLogInner>);
#[derive(Default)]
struct ReplayLogInner {
    commands: boxcar::Vec<Command>,
    selection: Option<(Uuid, usize)>,
}
impl Clone for ReplayLog {
    fn clone(&self) -> Self {
        let log = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self(Mutex::new(ReplayLogInner {
            commands: log.commands.clone(),
            selection: log.selection,
        }))
    }
}
impl ReplayLog {
    fn count(&self) -> usize {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .commands
            .count()
    }
    fn get(&self, index: usize) -> Option<Command> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .commands
            .get(index)
            .cloned()
    }
    fn push(&self, command: Command) {
        self.record(command, true, None);
    }
    fn replace_attachment(&self, session_id: Uuid) {
        let mut log = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(command) = log.commands.get_mut(0) {
            *command = Command::Attach { session_id };
        }
    }
    fn record(&self, command: Command, bounded: bool, checkpoint: Option<Uuid>) {
        let mut log = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !bounded {
            super::record_replayable(&log.commands, command);
            return;
        }
        log.commands.push(command);
        let Some(identity) = checkpoint else {
            return;
        };
        let count = log.commands.count();
        if count < 3 {
            return;
        }
        let start = count - 3;
        let auth = |command: Option<&Command>| {
            matches!(
                command,
                Some(Command::Signin { .. } | Command::Authenticate { .. })
            )
        };
        if !auth(log.commands.get(start)) || !auth(log.commands.get(start + 2)) {
            return;
        }
        let Some(Command::Use {
            namespace: Some(namespace),
            database: Some(database),
        }) = log.commands.get(start + 1)
        else {
            return;
        };
        if let Some((previous_identity, previous_start)) = log.selection
            && previous_identity == identity
            && previous_start + 3 == start
            && let Some(Command::Use {
                namespace: Some(previous_namespace),
                database: Some(previous_database),
            }) = log.commands.get(previous_start + 1)
            && previous_namespace == namespace
            && previous_database == database
        {
            let replacement = boxcar::Vec::new();
            for (index, command) in log.commands.iter() {
                if !(previous_start..start).contains(&index) {
                    replacement.push(command.clone());
                }
            }
            log.commands = replacement;
            log.selection = Some((identity, previous_start));
        } else {
            log.selection = Some((identity, start));
        }
    }
}

/// A replay in progress: the request id currently awaiting acknowledgement, and
/// the index of the command to send once it lands.
///
/// The log is order-sensitive — two `set`s of one key, or a `set` followed by an
/// `unset`, leave different end states depending on the order they are applied —
/// and the server may apply requests that arrive together on one connection in
/// any order. Tracking a single outstanding command is what forces the replay to
/// go one at a time.
#[derive(Debug, Clone, Copy)]
struct ReplayCursor {
    /// Request id of the command in flight.
    awaiting: i64,
    /// Position in [`SessionState::replay`] of the command in flight.
    index: usize,
    /// Whether the command in flight has already been retried with a refreshed
    /// token. Bounds the retry to one attempt so an expiry the refresh token
    /// cannot fix fails the session instead of looping.
    refreshed: bool,
    /// Absolute socket/session restoration deadline, unchanged by ACK or refresh.
    deadline: std::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplaySend {
    Sent,
    Ready,
    Failed,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            bounded: std::sync::atomic::AtomicBool::new(false),
            pending_requests: HashMap::new(),
            live_queries: HashMap::new(),
            replay: ReplayLog::default(),
            replay_cursor: Mutex::new(None),
            deferred: Mutex::new(Vec::new()),
            last_id: AtomicI64::new(0),
        }
    }
}

impl Clone for SessionState {
    fn clone(&self) -> Self {
        Self {
            bounded: std::sync::atomic::AtomicBool::new(self.bounded.load(Ordering::Acquire)),
            replay: self.replay.clone(),
            pending_requests: HashMap::new(),
            live_queries: HashMap::new(),
            // The clone runs its own replay and parks its own requests; the
            // parent's in-flight setup and backlog belong to the parent.
            replay_cursor: Mutex::new(None),
            deferred: Mutex::new(Vec::new()),
            last_id: AtomicI64::new(0),
        }
    }
}

/// Handle result for WebSocket operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandleResult {
    /// Socket disconnected, should continue to reconnect
    Disconnected,
    /// Bounded restoration failed: close admission and require owner reopen.
    Failed,
    /// Nothing wrong, continue as normal.
    Ok,
}

// ============================================================================
// Platform Abstraction Traits
// ============================================================================

/// Trait for abstracting over different WebSocket message types (native vs WASM).
trait WsMessage: Sized + Clone + Unpin + Send {
    /// Create a binary message from bytes.
    fn binary(payload: Vec<u8>) -> Self;

    /// Check if this is a binary message and get the bytes.
    fn as_binary(&self) -> Option<&[u8]>;

    /// Check if this message should be processed (filters out ping/pong/etc).
    fn should_process(&self) -> bool {
        true
    }

    /// Get a description for logging purposes.
    fn log_description(&self) -> &'static str {
        "message"
    }
}

// ============================================================================
// Shared Helper Functions
// ============================================================================

/// Serialize a router request to a WebSocket message.
fn serialize_request<M: WsMessage>(request: RouterRequest) -> M {
    let request_value = request.into_value();
    let payload = surrealdb_types::encode(&request_value).expect("router request should serialize");
    M::binary(payload)
}

/// Create a ping message for keep-alive.
fn create_ping_message<M: WsMessage>() -> M {
    let request = Command::Health
        .into_router_request(None, None)
        .expect("HEALTH command should convert to router request");
    serialize_request(request)
}

/// Create a kill message for terminating a live query.
fn create_kill_message<M: WsMessage>(live_query_id: Uuid, session_id: Uuid) -> M {
    let request = Command::Kill {
        uuid: live_query_id,
    }
    .into_router_request(None, Some(session_id))
    .expect("KILL command should convert to router request");
    serialize_request(request)
}

/// Send a message through the sink.
async fn send_message<M, S, E>(sink: &RwLock<S>, message: M) -> Result<(), E>
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
{
    sink.write().await.send(message).await
}

#[derive(Debug)]
enum SendFailure<E> {
    Transport(E),
    InterruptedBeforeDispatch,
    InterruptedAfterDispatch,
}

async fn send_request_message<M, S, E>(
    sink: &RwLock<S>,
    message: M,
    context: Option<&crate::opt::RequestContext>,
) -> Result<(), SendFailure<E>>
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
{
    let Some(context) = context else {
        return send_message(sink, message)
            .await
            .map_err(SendFailure::Transport);
    };
    let mut sink = tokio::select! { biased;
        _ = context.interrupted() => return Err(SendFailure::InterruptedBeforeDispatch),
        sink = sink.write() => sink,
    };
    tokio::select! { biased;
        _ = context.interrupted() => return Err(SendFailure::InterruptedBeforeDispatch),
        ready = futures::future::poll_fn(|cx| std::pin::Pin::new(&mut *sink).poll_ready(cx)) => ready.map_err(SendFailure::Transport)?,
    }
    if context.is_interrupted() {
        return Err(SendFailure::InterruptedBeforeDispatch);
    }
    context.mark_dispatched();
    std::pin::Pin::new(&mut *sink)
        .start_send(message)
        .map_err(SendFailure::Transport)?;
    tokio::select! { biased;
        _ = context.interrupted() => Err(SendFailure::InterruptedAfterDispatch),
        result = sink.flush() => result.map_err(SendFailure::Transport),
    }
}

/// Release refused deferred work even when replay acknowledgments never arrive.
/// Retain dispatched correlations for a finite local drain, then force physical
/// invalidation. This is not evidence that the server operation stopped.
async fn maintain_request_lifetimes(
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
) -> HandleResult {
    let now = std::time::Instant::now();
    for (session_id, session) in sessions.to_vec() {
        let Ok(state) = session else {
            continue;
        };
        if state.bounded.load(Ordering::Acquire)
            && replay_cursor(&state).is_some_and(|cursor| now >= cursor.deadline)
        {
            return fail_replay(&state, sessions, session_id, restoration_error()).await;
        }
        let deferred = discard_interrupted_routes(take_deferred_routes(&state));
        match state.deferred.lock() {
            Ok(mut parked) => parked.extend(deferred),
            Err(poisoned) => poisoned.into_inner().extend(deferred),
        }
        for (id, mut pending) in state.pending_requests.to_vec() {
            if let Some(context) = &pending.request_context {
                if context.is_interrupted() {
                    let since = *pending.interrupted_at.get_or_insert(now);
                    if now.duration_since(since) >= Duration::from_secs(90) {
                        return HandleResult::Disconnected;
                    }
                    let _ = pending
                        .response_channel
                        .try_send(Err(context.interruption_error()));
                    state.pending_requests.insert(id, pending);
                }
            }
        }
    }
    HandleResult::Ok
}

// ============================================================================
// Request Handling
// ============================================================================

// Fields drop in declaration order even when an async owner is never polled.
struct UndispatchedRoute {
    route: Route,
    drain: surrealdb_engine_api::request::UndispatchedPayloadDrain,
}
impl UndispatchedRoute {
    fn new(route: Route) -> Self {
        let drain = surrealdb_engine_api::request::UndispatchedPayloadDrain::driver(
            route.request.request_context.clone(),
        );
        Self { route, drain }
    }
}

/// Consume every actual parked command copy before releasing its drain cell.
fn refuse_undispatched_route(route: Route, error: TypesError) {
    let owned = UndispatchedRoute::new(route);
    let _drain = owned.drain;
    let Route {
        request,
        response,
        admission,
    } = owned.route;
    let RequestData { command, .. } = request;
    drop(command);
    if let Some(admission) = admission {
        admission.release();
    }
    let _ = response.try_send(Err(error));
    response.close();
}

fn discard_interrupted_routes(routes: Vec<Route>) -> Vec<Route> {
    let mut retained = Vec::new();
    for route in routes {
        let error = route
            .request
            .request_context
            .as_ref()
            .filter(|context| context.is_interrupted())
            .map(|context| context.interruption_error());
        if let Some(error) = error {
            refuse_undispatched_route(route, error);
        } else if route.response.is_closed() {
            refuse_undispatched_route(
                route,
                Error::connection(
                    "Request receiver closed".to_string(),
                    ConnectionError::ConnectionFailed,
                ),
            );
        } else {
            retained.push(route);
        }
    }
    retained
}

/// Handle an incoming route request.
///
/// This is the core logic for processing commands from the SDK client.
/// It's shared between native and WASM implementations.
fn handle_route<'a, M, S, E>(
    route: Route,
    max_message_size: Option<usize>,
    sessions: &'a HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &'a RwLock<S>,
) -> impl std::future::Future<Output = HandleResult> + 'a
where
    M: WsMessage + 'a,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug + 'a,
{
    let owned = UndispatchedRoute::new(route);
    async move {
        let owned = owned; // Capture the whole ordered owner even before first poll.
        let mut drain = owned.drain;
        let route = owned.route;
        let session_id = route.request.session_id;
        if route
            .request
            .request_context
            .as_ref()
            .is_some_and(|ctx| ctx.is_interrupted())
        {
            let error = route
                .request
                .request_context
                .as_ref()
                .unwrap()
                .interruption_error();
            let _ = route.response.send(Err(error)).await;
            return HandleResult::Ok;
        }

        // Get session state
        let session_state = match sessions.get(&session_id) {
            Some(Ok(state)) => state,
            Some(Err(error)) => {
                if route
                    .response
                    .send(Err(session_error_to_error(error)))
                    .await
                    .is_err()
                {
                    trace!("Receiver dropped");
                }
                return HandleResult::Ok;
            }
            None => {
                let error = session_error_to_error(SessionError::NotFound(session_id));
                if route.response.send(Err(error)).await.is_err() {
                    trace!("Receiver dropped");
                }
                return HandleResult::Ok;
            }
        };

        // The session's replayed setup is still in flight, so the server has not
        // applied all of it yet. Park the request until it has: dispatching now
        // would race the replay and could run against a session with no namespace,
        // no authentication, or none of its variables.
        if replay_cursor(&session_state).is_some() {
            drain.update_context(None); // Queue owns the still-undispatched bytes.
            park_route(&session_state, route);
            return HandleResult::Ok;
        }

        dispatch_route::<M, S, E>(route, max_message_size, &session_state, sink).await
    }
}

/// Send a request for a session whose setup the server has already applied.
fn dispatch_route<'a, M, S, E>(
    route: Route,
    max_message_size: Option<usize>,
    session_state: &'a Arc<SessionState>,
    sink: &'a RwLock<S>,
) -> impl std::future::Future<Output = HandleResult> + 'a
where
    M: WsMessage + 'a,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug + 'a,
{
    let owned = UndispatchedRoute::new(route);
    async move {
        let owned = owned; // Capture the whole ordered owner even before first poll.
        let _drain = owned.drain;
        let Route {
            request,
            response,
            admission,
        } = owned.route;
        let RequestData {
            command,
            session_id,
            request_context,
        } = request;

        if command.replayable()
            && request_context
                .as_ref()
                .is_some_and(|context| context.payload_owner().is_some())
        {
            // This new local ownership contract is qualified for nonreplayable
            // result Query bytes. Retained replay needs its own byte owner; never
            // release a grant while a successful command enters the replay log.
            drop(command);
            if let Some(admission) = &admission {
                admission.release();
            }
            let _ = response
                .send(Err(Error::configuration(
                    "Payload ownership is not qualified for retained replay commands".to_string(),
                    None,
                )))
                .await;
            return HandleResult::Ok;
        }

        if request_context
            .as_ref()
            .is_some_and(|ctx| ctx.is_interrupted())
        {
            let _ = response
                .send(Err(request_context.as_ref().unwrap().interruption_error()))
                .await;
            return HandleResult::Ok;
        }
        if session_state.bounded.load(Ordering::Acquire) {
            let prospective = session_state
                .pending_requests
                .values()
                .into_iter()
                .filter(|pending| pending.command.is_some())
                .count();
            if session_state.pending_requests.len() >= 34
                || (command.replayable() && session_state.replay.count() + prospective >= 64)
            {
                let _ = response
                    .send(Err(Error::configuration(
                        "Bounded connection capacity exhausted".to_string(),
                        None,
                    )))
                    .await;
                return HandleResult::Ok;
            }
        }

        // Generate a new request ID
        let id = session_state.last_id.fetch_add(1, Ordering::SeqCst);

        // Check for duplicate request IDs
        if session_state.pending_requests.contains_key(&id) {
            let error = Error::validation(
                format!("Duplicate request ID: {id}"),
                ValidationError::InvalidParams,
            );
            if response.send(Err(error)).await.is_err() {
                trace!("Receiver dropped");
            }
            return HandleResult::Ok;
        }

        // Handle special commands
        match command {
            Command::SubscribeLive {
                ref uuid,
                ref notification_sender,
            } => {
                session_state
                    .live_queries
                    .insert(*uuid, notification_sender.clone());
                if response
                    .send(Ok(vec![QueryResultBuilder::instant_none()]))
                    .await
                    .is_err()
                {
                    trace!("Receiver dropped");
                }
                return HandleResult::Ok;
            }
            Command::Kill { ref uuid } => {
                session_state.live_queries.remove(uuid);
            }
            _ => {}
        }

        // Serialize the request
        let Some(router_request) = command
            .clone()
            .into_router_request(Some(id), Some(session_id))
        else {
            response
                .send(Err(Error::internal(
                    "The protocol or storage engine does not support backups on this architecture"
                        .to_string(),
                )))
                .await
                .ok();
            return HandleResult::Ok;
        };

        let message: M = serialize_request(router_request);

        // Check message size
        if let Some(max_size) = max_message_size
            && let Some(binary) = message.as_binary()
            && binary.len() > max_size
        {
            if response
                .send(Err(Error::validation(
                    format!("Message too long: {}", binary.len()),
                    ValidationError::InvalidParams,
                )))
                .await
                .is_err()
            {
                trace!("Receiver dropped");
            }
            return HandleResult::Ok;
        }

        // A bounded route's admission is owned by the driver before polling the
        // sink. If a flush future is cancelled by the physical IO budget, epoch
        // storage keeps this ownership until both socket halves have been dropped.
        let staged_pending = session_state.bounded.load(Ordering::Acquire)
            || request_context
                .as_ref()
                .is_some_and(crate::opt::RequestContext::has_request_ownership);
        if staged_pending {
            session_state.pending_requests.insert(
                id,
                PendingRequest {
                    request_context: request_context.clone(),
                    _admission: admission.clone(),
                    interrupted_at: None,
                    command: if command.replayable() {
                        Some(command.clone())
                    } else {
                        None
                    },
                    response_channel: response.clone(),
                },
            );
        }
        // Send the message
        match send_request_message(sink, message, request_context.as_ref()).await {
            Ok(_) => {
                session_state.pending_requests.insert(
                    id,
                    PendingRequest {
                        request_context,
                        _admission: admission,
                        interrupted_at: None,
                        command: if command.replayable() {
                            Some(command)
                        } else {
                            None
                        },
                        response_channel: response,
                    },
                );
            }
            Err(SendFailure::InterruptedBeforeDispatch) => {
                if staged_pending {
                    session_state.pending_requests.take(&id);
                    if let Some(admission) = &admission {
                        admission.release();
                    }
                }
                let _ = response
                    .send(Err(request_context.as_ref().unwrap().interruption_error()))
                    .await;
                return HandleResult::Ok;
            }
            Err(error) => {
                let err = Error::connection(
                    format!("WebSocket error: {:?}", error),
                    ConnectionError::ConnectionFailed,
                );
                if response.send(Err(err)).await.is_err() {
                    trace!("Receiver dropped");
                }
                return HandleResult::Disconnected;
            }
        }

        HandleResult::Ok
    }
}

// ============================================================================
// Response Handling
// ============================================================================

/// Snapshot the finite pending grants before native response decoding allocates.
/// Owned decoded values keep these holds through failed sends and cancellation;
/// the original frame dies before delivery can wake a caller which partitions
/// its reservation into retained output buffers. No holds enter retired metadata.
#[cfg(not(target_family = "wasm"))]
fn handle_owned_response<'a, M, S, E>(
    message: M,
    max_message_size: Option<usize>,
    sessions: &'a HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &'a RwLock<S>,
) -> impl std::future::Future<Output = HandleResult> + 'a
where
    M: WsMessage + 'a,
    S: Sink<M, Error = E> + Unpin + 'a,
    E: std::fmt::Debug + 'a,
{
    let retained: Vec<_> = sessions
        .values()
        .into_iter()
        .filter_map(Result::ok)
        .flat_map(|session| session.pending_requests.values())
        .filter_map(|pending| {
            pending
                .request_context
                .and_then(|context| context.payload_owner())
        })
        .map(|owner| owner.retain_driver())
        .collect();
    let lifetime_retained: Vec<_> = sessions
        .values()
        .into_iter()
        .filter_map(Result::ok)
        .flat_map(|session| session.pending_requests.values())
        .filter_map(|pending| {
            pending
                .request_context
                .and_then(|context| context.lifetime_owner())
        })
        .map(|owner| owner.retain())
        .collect();
    // Struct/tuple fields drop in declaration order even if never polled.
    let owned = (message, retained, lifetime_retained);
    async move {
        let (message, retained, lifetime_retained) = owned;
        // Polled futures drop locals in reverse order. Move the actual frame
        // after the retained grants so its bytes always disappear first.
        let _retained = retained;
        let _lifetime_retained = lifetime_retained;
        let message = message;
        let Some(binary) = response_binary(&message) else {
            return HandleResult::Ok;
        };
        match surrealdb_rpc::db_response_from_bytes(binary) {
            Ok(response) => {
                // DbResponse owns its decoded Values (no frame borrow). Drain
                // the raw allocation before sending can wake the recipient.
                // That recipient may then split the same grant for its output.
                drop(message);
                handle_db_response::<M, S, E>(response, max_message_size, sessions, sink).await
            }
            Err(error) => {
                // Parse recovery still consumes raw bytes. Keep the frame and
                // grants through that work and drop bytes before the holds.
                handle_parse_error(
                    Error::serialization(error.to_string(), SerializationError::Deserialization),
                    binary,
                    sessions,
                )
                .await
            }
        }
    }
}

fn response_binary<M: WsMessage>(message: &M) -> Option<&[u8]> {
    if !message.should_process() {
        trace!("Received {}", message.log_description());
        return None;
    }
    let binary = message.as_binary();
    if binary.is_none() {
        trace!("Received non-binary message");
    }
    binary
}

/// Handle a response from the server.
///
/// This processes incoming messages and routes them to the appropriate
/// pending request or live query handler.
#[cfg(target_family = "wasm")]
async fn handle_response<M, S, E>(
    message: &M,
    max_message_size: Option<usize>,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    let Some(binary) = response_binary(message) else {
        return HandleResult::Ok;
    };

    match surrealdb_rpc::db_response_from_bytes(binary) {
        Ok(response) => {
            handle_db_response::<M, S, E>(response, max_message_size, sessions, sink).await
        }
        Err(error) => {
            handle_parse_error(
                Error::serialization(error.to_string(), SerializationError::Deserialization),
                binary,
                sessions,
            )
            .await
        }
    }
}

/// Handle a successfully parsed database response.
async fn handle_db_response<M, S, E>(
    response: DbResponse,
    max_message_size: Option<usize>,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    let Some(session_id) = response.session_id else {
        return HandleResult::Ok;
    };

    let session_state = match sessions.get(&session_id) {
        Some(Ok(state)) => state,
        _ => return HandleResult::Ok,
    };

    match response.id {
        // Normal response with ID
        Some(id) => {
            if let Value::Number(Number::Int(id_num)) = id {
                handle_response_with_id::<M, S, E>(
                    id_num,
                    response.result,
                    session_id,
                    &session_state,
                    max_message_size,
                    sessions,
                    sink,
                )
                .await
            } else {
                HandleResult::Ok
            }
        }
        // Live query notification (no ID)
        None => {
            handle_live_notification::<M, S, E>(response.result, session_id, &session_state, sink)
                .await
        }
    }
}

/// Handle a response that has an ID (normal request/response).
async fn handle_response_with_id<M, S, E>(
    id: i64,
    result: Result<DbResult, TypesError>,
    session_id: Uuid,
    session_state: &Arc<SessionState>,
    max_message_size: Option<usize>,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    // An acknowledgement of a replayed setup command rather than a reply to a
    // caller's request.
    if let Some(cursor) = replay_cursor(session_state)
        && cursor.awaiting == id
    {
        if session_state.bounded.load(Ordering::Acquire)
            && std::time::Instant::now() >= cursor.deadline
        {
            return fail_replay(session_state, sessions, session_id, restoration_error()).await;
        }
        if let Err(error) = result {
            // An expired access token on a replayed `authenticate` is routine:
            // the handle has been idle past the token's lifetime, or is being
            // cloned after it elapsed. The token carries its own refresh half,
            // so retry the command with it rather than abandoning the session —
            // poisoning here would permanently disable a handle that holds
            // perfectly good credentials, since a poisoned session is never
            // replayed again and cannot route even an explicit refresh.
            if !cursor.refreshed
                && let Some(Command::Authenticate { token }) =
                    session_state.replay.get(cursor.index)
                && let Token::WithRefresh { .. } = token
                && error
                    .not_allowed_details()
                    .is_some_and(|a| matches!(a, NotAllowedError::Auth(AuthError::TokenExpired)))
            {
                let refresh_request = RouterRequest {
                    id: Some(id),
                    method: "authenticate",
                    params: Some(Value::Array(Array::from(vec![token.clone().into_value()]))),
                    txn: None,
                    session_id: Some(session_id),
                };
                let message: M = serialize_request(refresh_request);

                if !send_restoration_message(sink, message, session_state, cursor.deadline).await {
                    trace!("Refresh setup transmission failed");
                    return fail_replay(session_state, sessions, session_id, restoration_error())
                        .await;
                } else {
                    // Same command, same id: stay parked on it until the
                    // refreshed attempt is answered.
                    set_replay_cursor(
                        session_state,
                        Some(ReplayCursor {
                            refreshed: true,
                            ..cursor
                        }),
                    );
                }
                return HandleResult::Ok;
            }

            return fail_replay(session_state, sessions, session_id, error).await;
        }

        // This command has been applied, so the next one can go out. Sending
        // them one at a time is what keeps an order-sensitive log — a `set`
        // followed by an `unset` of the same key — from being applied in the
        // wrong order by a server that dispatches concurrently.
        return match send_replay_command::<M, S, E>(
            session_id,
            session_state,
            cursor.index + 1,
            cursor.deadline,
            sink,
        )
        .await
        {
            ReplaySend::Sent => HandleResult::Ok,
            ReplaySend::Ready => {
                flush_deferred_routes::<M, S, E>(session_state, max_message_size, sink).await
            }
            ReplaySend::Failed => {
                fail_replay(session_state, sessions, session_id, restoration_error()).await
            }
        };
    }

    let Some(mut pending) = session_state.pending_requests.take(&id) else {
        warn!("got response for request with id '{id}', which was not in pending requests");
        return HandleResult::Ok;
    };

    let mut terminal_lifetime = TerminalCompletion {
        admission: None,
        context: pending.request_context.clone(),
    };
    if let Some(context) = &pending.request_context {
        context.mark_completed();
        // The serial router has joined dispatch and its send/flush before this
        // response branch. Owner-bearing replay commands were refused before
        // dispatch, so correlation metadata retains no actual request bytes.
        if pending.command.is_none() {
            context.release_payload_after_drain();
        }
    }
    if let Some(admission) = &pending._admission {
        admission.release();
    }
    match result {
        Ok(DbResult::Query(results)) => {
            if let Some(command) = pending.command {
                session_state.replay.record(
                    command,
                    session_state
                        .bounded
                        .load(std::sync::atomic::Ordering::Acquire),
                    pending
                        .request_context
                        .as_ref()
                        .and_then(crate::opt::RequestContext::selection_checkpoint_identity),
                );
            }
            if pending.response_channel.send(Ok(results)).await.is_err() {
                tracing::error!("Query response receiver closed");
            }
        }
        Ok(DbResult::Live(_)) => {
            tracing::error!("Unexpected live query result in response");
        }
        Ok(DbResult::Other(mut value)) => {
            if let Some(command) = pending.command {
                if let Command::Authenticate { token, .. } = &command {
                    value = token.clone().into_value();
                }
                session_state.replay.record(
                    command,
                    session_state
                        .bounded
                        .load(std::sync::atomic::Ordering::Acquire),
                    pending
                        .request_context
                        .as_ref()
                        .and_then(crate::opt::RequestContext::selection_checkpoint_identity),
                );
            }
            let result = QueryResultBuilder::started_now().finish_with_result(Ok(value));
            if pending
                .response_channel
                .send(Ok(vec![result]))
                .await
                .is_err()
            {
                tracing::error!("Setup response receiver closed");
            }
        }
        Err(error) => {
            // Handle automatic token refresh
            if !session_state.bounded.load(Ordering::Acquire)
                && let Some(Command::Authenticate { token, .. }) = pending.command
                && let Token::WithRefresh { .. } = &token
                && error
                    .not_allowed_details()
                    .is_some_and(|a| matches!(a, NotAllowedError::Auth(AuthError::TokenExpired)))
            {
                // Attempt automatic refresh
                let refresh_request = RouterRequest {
                    id: Some(id),
                    method: "authenticate",
                    params: Some(Value::Array(Array::from(vec![token.into_value()]))),
                    txn: None,
                    session_id: Some(session_id),
                };
                let message: M = serialize_request(refresh_request);

                match send_message(sink, message).await {
                    Err(_) => {
                        trace!("Refresh setup transmission failed");
                        pending.response_channel.send(Err(error)).await.ok();
                    }
                    Ok(..) => {
                        // Keep request pending for retry after refresh
                        pending.command = None;
                        // Refresh remains the same local request, with its owner live.
                        terminal_lifetime.context = None;
                        session_state.pending_requests.insert(id, pending);
                    }
                }
                return HandleResult::Ok;
            }

            // Return error to caller
            pending.response_channel.send(Err(error)).await.ok();
        }
    }

    HandleResult::Ok
}

/// Park a request behind a session's in-flight setup.
fn park_route(session_state: &SessionState, route: Route) {
    match session_state.deferred.lock() {
        Ok(mut deferred) => {
            if session_state.bounded.load(Ordering::Acquire) && deferred.len() >= 34 {
                refuse_undispatched_route(
                    route,
                    Error::configuration("Deferred request capacity exhausted".to_string(), None),
                );
            } else {
                deferred.push(route);
            }
        }
        Err(poisoned) => poisoned.into_inner().push(route),
    }
}

/// Take everything parked behind a session's setup.
///
/// The lock is released before the caller awaits on any of the routes, and only
/// the router task ever touches the queue.
fn take_deferred_routes(session_state: &SessionState) -> Vec<Route> {
    match session_state.deferred.lock() {
        Ok(mut deferred) => std::mem::take(&mut *deferred),
        Err(poisoned) => std::mem::take(&mut *poisoned.into_inner()),
    }
}

/// Dispatch the requests parked while a session's setup was in flight.
async fn flush_deferred_routes<M, S, E>(
    session_state: &Arc<SessionState>,
    max_message_size: Option<usize>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    // Guard the entire drained queue before the first await. A timeout can
    // otherwise drop untouched, already-enqueued routes outside both the
    // caller guard and the pending correlation/reset registry.
    let drained: Vec<UndispatchedRoute> = take_deferred_routes(session_state)
        .into_iter()
        .map(UndispatchedRoute::new)
        .collect();
    let mut outcome = HandleResult::Ok;
    for owned in drained {
        let _drain = owned.drain;
        let route = owned.route;
        let result = dispatch_route::<M, S, E>(route, max_message_size, session_state, sink).await;
        if result != HandleResult::Ok {
            outcome = result;
        }
    }
    outcome
}

/// Fail the requests parked behind a session setup that did not complete.
async fn fail_deferred_routes(session_state: &SessionState, error: TypesError) {
    for route in take_deferred_routes(session_state) {
        refuse_undispatched_route(route, error.clone());
    }
}

/// Abandon the replay in progress: poison the session and fail everything
/// parked behind it.
///
/// Used when a replayed setup command is rejected, or when its reply cannot be
/// read. Either way the session never reached the state it was cloned or
/// reconnected into, so it is not the session the caller asked for, and the
/// acknowledgement the replay is waiting on is never going to arrive.
async fn fail_replay(
    session_state: &SessionState,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    session_id: Uuid,
    error: TypesError,
) -> HandleResult {
    set_replay_cursor(session_state, None);
    let bounded = session_state.bounded.load(Ordering::Acquire);
    let session_error = if bounded {
        SessionError::RemoteTyped(Box::new(error.clone()))
    } else {
        SessionError::Remote(error.to_string())
    };
    sessions.insert(session_id, Err(session_error));
    fail_deferred_routes(session_state, error).await;
    if bounded {
        HandleResult::Failed
    } else {
        HandleResult::Ok
    }
}

/// Handle a live query notification.
async fn handle_live_notification<M, S, E>(
    result: Result<DbResult, TypesError>,
    session_id: Uuid,
    session_state: &Arc<SessionState>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    if let Ok(DbResult::Live(notification)) = result {
        let live_query_id = notification.id.into_inner();
        // `Killed` is the subscription's own end: the server has removed it,
        // and this notification is what that removal sends. The registration
        // leaves with the notification rather than outliving the subscription
        // it names, holding the subscriber's channel until the connection
        // closes.
        let ended = matches!(notification.action, Action::Killed);
        let registration = match ended {
            true => session_state.live_queries.take(&live_query_id),
            false => session_state.live_queries.get(&live_query_id),
        };

        if let Some(sender) = registration
            && sender.send(Ok(notification)).await.is_err()
            && !ended
        {
            // Receiver dropped, kill the live query
            session_state.live_queries.remove(&live_query_id);
            let kill: M = create_kill_message(live_query_id, session_id);

            if send_message(sink, kill).await.is_err() {
                trace!("Live-query cancellation transmission failed");
                return HandleResult::Disconnected;
            }
        }
    }

    HandleResult::Ok
}

/// Handle a parse error by trying to extract the ID and return the error.
async fn handle_parse_error(
    error: crate::Error,
    binary: &[u8],
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
) -> HandleResult {
    // An unreadable response cannot prove authentic completion for an owned
    // payload. Keep its correlation reachable until the router drops both
    // physical halves; never strand it in epoch-retired metadata.
    let mut owned_pending = false;
    for state in sessions.values().into_iter().flatten() {
        for pending in state.pending_requests.values() {
            if pending
                .request_context
                .as_ref()
                .is_some_and(|context| context.payload_owner().is_some())
            {
                let _ = pending.response_channel.try_send(Err(error.clone()));
                owned_pending = true;
            }
        }
    }
    if owned_pending {
        return HandleResult::Disconnected;
    }

    #[derive(SurrealValue)]
    #[surreal(crate = "crate::types")]
    struct ErrorResponse {
        id: Option<Value>,
        #[surreal(rename = "session")]
        session_id: Option<Uuid>,
    }

    match surrealdb_types::decode::<ErrorResponse>(binary) {
        Ok(ErrorResponse { id, session_id }) => {
            let Some(session_id) = session_id else {
                return HandleResult::Ok;
            };

            let session_state = match sessions.get(&session_id) {
                Some(Ok(state)) => state,
                _ => return HandleResult::Ok,
            };

            match id {
                Some(Value::Number(Number::Int(id_num))) => {
                    if let Some(pending) = session_state.pending_requests.take(&id_num) {
                        let _completion = TerminalCompletion::non_payload(&pending);
                        let _ = pending.response_channel.send(Err(error)).await;
                    } else if replay_cursor(&session_state).is_some_and(|c| c.awaiting == id_num) {
                        // A replayed setup command whose reply we cannot read. The
                        // session's state on the server is now unknown, so poison it
                        // and fail the requests parked behind the replay instead of
                        // waiting for an acknowledgement that will never come.
                        return fail_replay(&session_state, sessions, session_id, error).await;
                    } else {
                        warn!(
                            "got response for request with id '{id_num}', which was not in pending requests"
                        );
                    }
                }
                // The envelope named a session but carries no usable request id,
                // so we cannot tell which request it answers. Treat a replay in
                // flight the same as the fully undecodable case: its
                // acknowledgement may be exactly what this frame was, and leaving
                // the cursor set would park every later request for the session
                // forever.
                _ => {
                    if replay_cursor(&session_state).is_some() {
                        return fail_replay(&session_state, sessions, session_id, error).await;
                    }
                }
            }
        }
        _ => {
            // The payload could not be decoded far enough to recover the request
            // id, so we cannot route the error to a single waiting request. Rather
            // than silently dropping it — which leaves every awaiting query hanging
            // forever (https://github.com/surrealdb/surrealdb/issues/7037) — fail
            // all currently-pending requests so callers surface the deserialization
            // error instead of blocking indefinitely.
            error!("Response decoding failed; refusing pending requests");
            return fail_all_pending_requests(sessions, error).await;
        }
    }

    HandleResult::Ok
}

/// Fail every pending request across all sessions with the given error.
///
/// Used when an incoming message cannot be parsed well enough to identify which
/// request it belongs to; failing the requests prevents them from hanging.
async fn fail_all_pending_requests(
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    error: crate::Error,
) -> HandleResult {
    let mut outcome = HandleResult::Ok;
    for (session_id, session) in sessions.to_vec() {
        let Ok(session_state) = session else {
            continue;
        };
        for (id, _) in session_state.pending_requests.to_vec() {
            if let Some(pending) = session_state.pending_requests.take(&id) {
                let _completion = TerminalCompletion::non_payload(&pending);
                pending.response_channel.send(Err(error.clone())).await.ok();
            }
        }
        // The unreadable frame may have been the acknowledgement a replay was
        // waiting on. A session mid-replay would otherwise park every later
        // request behind an acknowledgement that never arrives.
        if replay_cursor(&session_state).is_some() {
            outcome = fail_replay(&session_state, sessions, session_id, error.clone()).await;
        }
    }
    outcome
}

// ============================================================================
// Session Management
// ============================================================================

/// Establish a session's state on the server by replaying its command log.
///
/// Used both to build a newly registered or cloned session and to rebuild every
/// session after a reconnect.
///
/// Each command is sent with a request id recorded in
/// [`SessionState::replay_cursor`], so the router can tell when the server has
/// applied the whole batch. Requests for the session are parked until it has:
/// the server may apply requests that arrive together on one connection in any
/// order, so a request dispatched alongside the replay can observe a session
/// whose namespace, authentication or variables are not in place yet.
async fn replay_session<M, S, E>(
    session_id: Uuid,
    session_state: &SessionState,
    sink: &RwLock<S>,
) -> crate::Result<()>
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    // This replay supersedes any earlier one: acknowledgements for a command
    // sent on a connection that has since been replaced will never arrive, and
    // leaving the cursor on it would park this session's requests forever.
    let deadline = std::time::Instant::now() + RESTORATION_TIMEOUT;
    match send_replay_command::<M, S, E>(session_id, session_state, 0, deadline, sink).await {
        ReplaySend::Sent | ReplaySend::Ready => Ok(()),
        ReplaySend::Failed => Err(restoration_error()),
    }
}

fn restoration_error() -> TypesError {
    Error::connection(
        "Session restoration failed or exceeded its acknowledged deadline".to_string(),
        ConnectionError::ConnectionFailed,
    )
}

/// Every setup send, including token refresh, spends the same operation budget.
async fn send_restoration_message<M, S, E>(
    sink: &RwLock<S>,
    message: M,
    state: &SessionState,
    deadline: std::time::Instant,
) -> bool
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
{
    #[cfg(not(target_family = "wasm"))]
    if state.bounded.load(Ordering::Acquire) {
        if std::time::Instant::now() >= deadline {
            return false;
        }
        return tokio::time::timeout_at(deadline.into(), send_message(sink, message))
            .await
            .is_ok_and(|result| result.is_ok());
    }
    #[cfg(target_family = "wasm")]
    let _ = (state, deadline); // Bounded requests are a native profile.
    send_message(sink, message).await.is_ok()
}

/// Only acknowledged exhaustion is Ready; a failed send never flushes callers.
async fn send_replay_command<M, S, E>(
    session_id: Uuid,
    session_state: &SessionState,
    index: usize,
    deadline: std::time::Instant,
    sink: &RwLock<S>,
) -> ReplaySend
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    if session_state.bounded.load(Ordering::Acquire) && std::time::Instant::now() >= deadline {
        return ReplaySend::Failed;
    }
    let Some(command) = session_state.replay.get(index) else {
        set_replay_cursor(session_state, None);
        return ReplaySend::Ready;
    };
    let id = session_state.last_id.fetch_add(1, Ordering::SeqCst);
    set_replay_cursor(
        session_state,
        Some(ReplayCursor {
            awaiting: id,
            index,
            refreshed: false,
            deadline,
        }),
    );
    let request = command
        .into_router_request(Some(id), Some(session_id))
        .expect("replay commands should always convert to route requests");
    let message: M = serialize_request(request);
    if !send_restoration_message(sink, message, session_state, deadline).await {
        // Keep the failed cursor until its owner poisons or replaces this
        // incarnation. Its absence must never masquerade as readiness.
        debug!("Session setup transmission failed");
        return ReplaySend::Failed;
    }
    ReplaySend::Sent
}

/// The replay command this session is currently waiting to have acknowledged.
fn replay_cursor(session_state: &SessionState) -> Option<ReplayCursor> {
    match session_state.replay_cursor.lock() {
        Ok(cursor) => *cursor,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

fn set_replay_cursor(session_state: &SessionState, cursor: Option<ReplayCursor>) {
    match session_state.replay_cursor.lock() {
        Ok(mut slot) => *slot = cursor,
        Err(poisoned) => *poisoned.into_inner() = cursor,
    }
}

/// Handle new session registration.
async fn handle_session_initial<M, S, E>(
    session_id: Uuid,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
    bounded: bool,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    let session_state = Arc::new(SessionState::default());
    session_state.bounded.store(bounded, Ordering::Release);
    session_state.replay.push(Command::Attach { session_id });
    sessions.insert(session_id, Ok(Arc::clone(&session_state)));

    if let Err(error) = replay_session::<M, S, E>(session_id, &session_state, sink).await {
        return fail_replay(&session_state, sessions, session_id, error).await;
    }
    HandleResult::Ok
}

/// Handle session cloning.
async fn handle_session_clone<M, S, E>(
    old: Uuid,
    new: Uuid,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    match sessions.get(&old) {
        Some(Ok(session_state)) => {
            let session_state = session_state.as_ref().clone();
            // Replace the attach command with the new session id
            session_state.replay.replace_attachment(new);
            let session_state = Arc::new(session_state);
            sessions.insert(new, Ok(Arc::clone(&session_state)));

            if let Err(error) = replay_session::<M, S, E>(new, &session_state, sink).await {
                return fail_replay(&session_state, sessions, new, error).await;
            }
        }
        Some(Err(error)) => {
            sessions.insert(new, Err(error));
        }
        None => {
            sessions.insert(new, Err(SessionError::NotFound(old)));
        }
    }
    HandleResult::Ok
}

/// Handle session drop.
async fn handle_session_drop<M, S, E>(
    session_id: Uuid,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    if let Some(Ok(state)) = sessions.get(&session_id) {
        let pending_owner = state.pending_requests.values().into_iter().any(|pending| {
            pending
                .request_context
                .as_ref()
                .is_some_and(crate::opt::RequestContext::has_request_ownership)
        });
        let deferred_owner = match state.deferred.lock() {
            Ok(routes) => routes.iter().any(|route| {
                route
                    .request
                    .request_context
                    .as_ref()
                    .is_some_and(crate::opt::RequestContext::has_request_ownership)
            }),
            Err(routes) => routes.into_inner().iter().any(|route| {
                route
                    .request
                    .request_context
                    .as_ref()
                    .is_some_and(crate::opt::RequestContext::has_request_ownership)
            }),
        };
        if pending_owner || deferred_owner {
            // Detach is not drain. Leave all correlation reachable until the
            // serial router drops both physical halves and resets this state.
            return HandleResult::Disconnected;
        }
    }
    if sessions.get(&session_id).is_some() {
        // Fire-and-forget: the session is going away, so there is nothing left
        // to order the detach against and no one to hand an acknowledgement to.
        let request = Command::Detach { session_id }
            .into_router_request(None, Some(session_id))
            .expect("detach should always convert to a route request");

        let message: M = serialize_request(request);

        if send_message(sink, message).await.is_err() {
            debug!("Session setup transmission failed");
        }
    }
    sessions.remove(&session_id);
    HandleResult::Ok
}

/// Dispatch a session-lifecycle event to the appropriate handler.
async fn handle_session<M, S, E>(
    session_id: crate::SessionId,
    sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>,
    sink: &RwLock<S>,
) -> HandleResult
where
    M: WsMessage,
    S: Sink<M, Error = E> + Unpin,
    E: std::fmt::Debug,
{
    match session_id {
        crate::SessionId::Initial(id) => {
            handle_session_initial::<M, S, E>(id, sessions, sink, false).await
        }
        crate::SessionId::Clone { old, new } => {
            handle_session_clone::<M, S, E>(old, new, sessions, sink).await
        }
        crate::SessionId::Drop(id) => handle_session_drop::<M, S, E>(id, sessions, sink).await,
    }
}

/// Clear all pending requests on connection reset.
///
/// Requests still parked behind a session's setup are failed the same way as
/// ones already on the wire: the connection they were waiting on is gone.
async fn clear_pending_requests(sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>) {
    for state in sessions.values().into_iter().flatten() {
        for request in state.pending_requests.values() {
            let err = crate::Error::connection(
                "Connection reset".to_string(),
                surrealdb_types::ConnectionError::ConnectionFailed,
            );
            request.response_channel.try_send(Err(err)).ok();
            request.response_channel.close();
            // Only qualified nonreplayable payloads have no retained command
            // bytes in epoch metadata. Unsupported future retention stays charged.
            if request.command.is_none()
                && let Some(context) = &request.request_context
            {
                context.release_payload_after_drain();
            }
            if let Some(context) = &request.request_context {
                context.release_lifetime_after_drain();
            }
            if let Some(admission) = &request._admission {
                admission.release();
            }
        }
        state.pending_requests.clear();

        let err = crate::Error::connection(
            "Connection reset".to_string(),
            surrealdb_types::ConnectionError::ConnectionFailed,
        );
        fail_deferred_routes(&state, err).await;
    }
}

/// Clear all live queries on connection reset.
async fn clear_live_queries(sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>) {
    for state in sessions.values().into_iter().flatten() {
        for sender in state.live_queries.values() {
            let err = crate::Error::connection(
                "Connection reset".to_string(),
                surrealdb_types::ConnectionError::ConnectionFailed,
            );
            sender.send(Err(err)).await.ok();
            sender.close();
        }
        state.live_queries.clear();
    }
}

/// Reset all sessions on disconnect.
async fn reset_sessions(sessions: &HashMap<Uuid, Result<Arc<SessionState>, SessionError>>) {
    tokio::join!(
        clear_pending_requests(sessions),
        clear_live_queries(sessions)
    );
}

// ============================================================================
// Public Types
// ============================================================================

/// The WS scheme used to connect to `ws://` endpoints
#[derive(Debug)]
pub struct Ws;

/// The WSS scheme used to connect to `wss://` endpoints
#[derive(Debug)]
pub struct Wss;

/// A WebSocket client for communicating with the server via WebSockets
#[derive(Debug, Clone)]
pub struct Client(());

impl Surreal<Client> {
    /// Connects to a specific database endpoint, saving the connection on the
    /// static client
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::LazyLock;
    /// use surrealdb::Surreal;
    /// use surrealdb::engine::remote::ws::Client;
    /// use surrealdb::engine::remote::ws::Ws;
    ///
    /// static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);
    ///
    /// # #[tokio::main]
    /// # async fn main() -> surrealdb::Result<()> {
    /// DB.connect::<Ws>("localhost:8000").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect<P>(
        &self,
        address: impl IntoEndpoint<P, Client = Client>,
    ) -> Connect<Client, ()> {
        Connect {
            surreal: Arc::clone(&self.inner).into(),
            address: address.into_endpoint(),
            capacity: 0,
            response_type: PhantomData,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use surrealdb_rpc::{DbResult, QueryResult, Token};
    use surrealdb_types::{AuthError, Error as TypesError, NotAllowedError};
    use tokio::sync::RwLock;
    use uuid::Uuid;

    use super::{
        HandleResult, PendingRequest, SessionState, WsMessage, fail_all_pending_requests,
        handle_live_notification, handle_parse_error, handle_response_with_id, handle_route,
        replay_cursor, replay_session,
    };
    use crate::conn::{Command, RequestData, Route};
    use crate::engine::SessionError;
    use crate::types::{Action, HashMap, Notification, Number, Value};

    type Sessions = HashMap<Uuid, Result<Arc<SessionState>, SessionError>>;

    /// An empty session registry: these tests drive the pending-request path,
    /// which never consults it.
    fn no_sessions() -> Sessions {
        HashMap::new()
    }

    /// Mock WebSocket message for testing.
    #[derive(Clone)]
    struct MockMessage;

    impl WsMessage for MockMessage {
        fn binary(_payload: Vec<u8>) -> Self {
            MockMessage
        }

        fn as_binary(&self) -> Option<&[u8]> {
            None
        }
    }

    #[tokio::test]
    async fn handle_response_removes_pending_request() {
        let session_state = Arc::new(SessionState::default());
        let sessions = no_sessions();
        let session_id = Uuid::new_v4();
        let request_id: i64 = 1;

        // Insert a pending request
        let (sender, receiver) = async_channel::bounded(1);
        session_state.pending_requests.insert(
            request_id,
            PendingRequest {
                request_context: None,
                _admission: None,
                interrupted_at: None,
                command: None,
                response_channel: sender,
            },
        );
        assert_eq!(session_state.pending_requests.len(), 1);

        // Handle a successful response
        let sink = RwLock::new(futures::sink::drain::<MockMessage>());
        let result = handle_response_with_id::<MockMessage, _, _>(
            request_id,
            Ok(DbResult::Other(Value::None)),
            session_id,
            &session_state,
            None,
            &sessions,
            &sink,
        )
        .await;

        // Entry should be removed from pending_requests
        assert_eq!(result, HandleResult::Ok);
        assert!(
            session_state.pending_requests.is_empty(),
            "pending request should be removed after handling response"
        );

        // Response should have been delivered to the receiver
        let response = receiver.recv().await.unwrap();
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn handle_response_error_removes_pending_request() {
        let session_state = Arc::new(SessionState::default());
        let sessions = no_sessions();
        let session_id = Uuid::new_v4();
        let request_id: i64 = 1;

        // Insert a pending request (no replayable command, so no token refresh path)
        let (sender, receiver) = async_channel::bounded(1);
        session_state.pending_requests.insert(
            request_id,
            PendingRequest {
                request_context: None,
                _admission: None,
                interrupted_at: None,
                command: None,
                response_channel: sender,
            },
        );
        assert_eq!(session_state.pending_requests.len(), 1);

        // Handle an error response
        let sink = RwLock::new(futures::sink::drain::<MockMessage>());
        let error = TypesError::internal("test error".to_string());
        let result = handle_response_with_id::<MockMessage, _, _>(
            request_id,
            Err(error),
            session_id,
            &session_state,
            None,
            &sessions,
            &sink,
        )
        .await;

        // Entry should be removed from pending_requests
        assert_eq!(result, HandleResult::Ok);
        assert!(
            session_state.pending_requests.is_empty(),
            "pending request should be removed after handling error response"
        );

        // Error should have been delivered to the receiver
        let response = receiver.recv().await.unwrap();
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn handle_multiple_responses_cleans_up_all_entries() {
        let session_state = Arc::new(SessionState::default());
        let sessions = no_sessions();
        let session_id = Uuid::new_v4();
        let sink = RwLock::new(futures::sink::drain::<MockMessage>());

        // Insert many pending requests
        let mut receivers = Vec::new();
        for id in 0..100i64 {
            let (sender, receiver) = async_channel::bounded(1);
            session_state.pending_requests.insert(
                id,
                PendingRequest {
                    request_context: None,
                    _admission: None,
                    interrupted_at: None,
                    command: None,
                    response_channel: sender,
                },
            );
            receivers.push(receiver);
        }
        assert_eq!(session_state.pending_requests.len(), 100);

        // Handle all responses
        for id in 0..100i64 {
            handle_response_with_id::<MockMessage, _, _>(
                id,
                Ok(DbResult::Other(Value::None)),
                session_id,
                &session_state,
                None,
                &sessions,
                &sink,
            )
            .await;
        }

        // All entries should have been removed
        assert!(
            session_state.pending_requests.is_empty(),
            "all pending requests should be removed, but {} remain",
            session_state.pending_requests.len()
        );

        // All responses should have been delivered
        for receiver in &receivers {
            let response = receiver.recv().await.unwrap();
            assert!(response.is_ok());
        }
    }

    /// A registered session with `commands` queued for replay, none sent yet.
    fn session_with_replay(commands: Vec<Command>) -> (Uuid, Arc<SessionState>, Sessions) {
        let session_id = Uuid::new_v4();
        let session_state = Arc::new(SessionState::default());
        for command in commands {
            session_state.replay.push(command);
        }

        let sessions = HashMap::new();
        sessions.insert(session_id, Ok(Arc::clone(&session_state)));

        (session_id, session_state, sessions)
    }

    fn set_cmd(key: &str, value: i64) -> Command {
        Command::Set {
            key: key.to_string(),
            value: Value::Number(Number::Int(value)),
        }
    }

    /// Acknowledge the replay command currently in flight.
    async fn ack_replay<S>(
        session_id: Uuid,
        session_state: &Arc<SessionState>,
        sessions: &Sessions,
        sink: &RwLock<S>,
        result: Result<DbResult, TypesError>,
    ) where
        S: futures::Sink<MockMessage, Error = std::convert::Infallible> + Unpin,
    {
        let cursor = replay_cursor(session_state).expect("a replay command should be in flight");
        handle_response_with_id::<MockMessage, _, _>(
            cursor.awaiting,
            result,
            session_id,
            session_state,
            None,
            sessions,
            sink,
        )
        .await;
    }

    fn route_for(
        session_id: Uuid,
    ) -> (
        Route,
        async_channel::Receiver<Result<Vec<QueryResult>, TypesError>>,
    ) {
        let (response, receiver) = async_channel::bounded(1);
        let route = Route {
            admission: None,
            request: RequestData {
                request_context: None,
                command: Command::Health,
                session_id,
            },
            response,
        };
        (route, receiver)
    }

    #[tokio::test]
    async fn bounded_replay_keeps_one_deadline_across_acks_refresh_and_late_incarnation() {
        let (id, state, sessions) =
            session_with_replay(vec![refreshable_auth_cmd(), set_cmd("x", 1)]);
        state
            .bounded
            .store(true, std::sync::atomic::Ordering::Release);
        let sink = RwLock::new(Vec::<MockMessage>::new());
        replay_session::<MockMessage, _, _>(id, &state, &sink)
            .await
            .unwrap();
        let old = replay_cursor(&state).unwrap();
        // A replacement socket begins its own incarnation; old ACKs cannot finish it.
        replay_session::<MockMessage, _, _>(id, &state, &sink)
            .await
            .unwrap();
        let first = replay_cursor(&state).unwrap();
        assert_ne!(old.awaiting, first.awaiting);
        assert_eq!(
            handle_response_with_id::<MockMessage, _, _>(
                old.awaiting,
                Ok(DbResult::Other(Value::None)),
                id,
                &state,
                None,
                &sessions,
                &sink
            )
            .await,
            HandleResult::Ok
        );
        assert_eq!(replay_cursor(&state).unwrap().awaiting, first.awaiting);
        ack_replay(id, &state, &sessions, &sink, Err(token_expired())).await;
        let refreshed = replay_cursor(&state).unwrap();
        assert!(refreshed.refreshed);
        assert_eq!(refreshed.deadline, first.deadline);
        ack_replay(
            id,
            &state,
            &sessions,
            &sink,
            Ok(DbResult::Other(Value::None)),
        )
        .await;
        let second = replay_cursor(&state).unwrap();
        assert_eq!(second.deadline, first.deadline);
        assert_eq!(second.index, 1);
        // Duplicate previous ACK does not advance a later command.
        handle_response_with_id::<MockMessage, _, _>(
            first.awaiting,
            Ok(DbResult::Other(Value::None)),
            id,
            &state,
            None,
            &sessions,
            &sink,
        )
        .await;
        assert_eq!(replay_cursor(&state).unwrap().awaiting, second.awaiting);
        let (route, response) = route_for(id);
        handle_route::<MockMessage, _, _>(route, None, &sessions, &sink).await;
        super::set_replay_cursor(
            &state,
            Some(super::ReplayCursor {
                deadline: std::time::Instant::now() - Duration::from_millis(1),
                ..second
            }),
        );
        let sent = sink.read().await.len();
        assert_eq!(
            super::maintain_request_lifetimes(&sessions).await,
            HandleResult::Failed
        );
        assert!(response.recv().await.unwrap().is_err());
        assert_eq!(
            sink.read().await.len(),
            sent,
            "no deferred application dispatch"
        );
        assert!(matches!(sessions.get(&id), Some(Err(_))));
    }

    /// This sink fails exactly the second setup send, then would accept callers.
    /// A second transport error cannot accidentally enforce the ready invariant.
    #[derive(Default)]
    struct SingleSetupFailureSink {
        attempts: usize,
    }
    impl futures::Sink<MockMessage> for SingleSetupFailureSink {
        type Error = &'static str;
        fn poll_ready(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
        fn start_send(
            mut self: std::pin::Pin<&mut Self>,
            _: MockMessage,
        ) -> Result<(), Self::Error> {
            self.attempts += 1;
            if self.attempts == 2 {
                Err("single setup failure")
            } else {
                Ok(())
            }
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn bounded_replay_single_send_failure_never_dispatches_deferred_routes() {
        let (id, state, sessions) = session_with_replay(vec![set_cmd("x", 1), set_cmd("x", 2)]);
        state
            .bounded
            .store(true, std::sync::atomic::Ordering::Release);
        let sink = RwLock::new(SingleSetupFailureSink::default());
        replay_session::<MockMessage, _, _>(id, &state, &sink)
            .await
            .unwrap();
        let (route, response) = route_for(id);
        handle_route::<MockMessage, _, _>(route, None, &sessions, &sink).await;
        let first = replay_cursor(&state).unwrap();
        assert_eq!(
            handle_response_with_id::<MockMessage, _, _>(
                first.awaiting,
                Ok(DbResult::Other(Value::None)),
                id,
                &state,
                None,
                &sessions,
                &sink
            )
            .await,
            HandleResult::Failed
        );
        assert_eq!(sink.read().await.attempts, 2, "no third, application send");
        assert!(response.recv().await.unwrap().is_err());
        assert!(state.pending_requests.is_empty());
        assert!(matches!(sessions.get(&id), Some(Err(_))));
    }

    #[tokio::test]
    async fn bounded_refresh_send_stall_spends_remaining_restoration_deadline() {
        let (id, state, sessions) = session_with_replay(vec![refreshable_auth_cmd()]);
        state
            .bounded
            .store(true, std::sync::atomic::Ordering::Release);
        let initial = RwLock::new(Vec::<MockMessage>::new());
        replay_session::<MockMessage, _, _>(id, &state, &initial)
            .await
            .unwrap();
        let mut cursor = replay_cursor(&state).unwrap();
        cursor.deadline = std::time::Instant::now() + Duration::from_millis(20);
        super::set_replay_cursor(&state, Some(cursor));
        let (route, response) = route_for(id);
        handle_route::<MockMessage, _, _>(route, None, &sessions, &initial).await;
        let sink = RwLock::new(ReadyStallSink {
            polled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        });
        assert_eq!(
            tokio::time::timeout(
                Duration::from_secs(1),
                handle_response_with_id::<MockMessage, _, _>(
                    cursor.awaiting,
                    Err(token_expired()),
                    id,
                    &state,
                    None,
                    &sessions,
                    &sink
                )
            )
            .await
            .unwrap(),
            HandleResult::Failed
        );
        assert!(response.recv().await.unwrap().is_err());
        assert!(matches!(sessions.get(&id), Some(Err(_))));
    }

    async fn malformed_nonpayload_admission(undecodable: bool, cancel_handler: bool) {
        use std::future::Future;
        use surrealdb_engine_api::{EngineContext, RouteChannelEngine, SurrealEngine};
        for control in [false, true] {
            let (sender, routes) = async_channel::bounded(34);
            let (_shutdown, shutdown) = async_channel::bounded(1);
            let engine = RouteChannelEngine::with_shutdown(sender, shutdown);
            let (id, state, sessions) = session_with_replay(Vec::new());
            let sink = RwLock::new(Vec::<MockMessage>::new());
            let mut retired = Vec::new();
            // Retain more metadata than the exact finite pool permits.
            let iterations = if control { 6 } else { 40 };
            for _ in 0..iterations {
                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                let context = if control {
                    crate::opt::RequestContext::control(deadline)
                } else {
                    crate::opt::RequestContext::new(deadline)
                };
                let activity = Arc::new(());
                let activity_weak = Arc::downgrade(&activity);
                let context = context.with_lifetime_owner(activity);
                assert!(
                    context.payload_owner().is_none(),
                    "activity is never a payload classification"
                );
                let mut request = engine
                    .health(EngineContext::new(id).with_request_context(Some(context.clone())));
                let route = tokio::time::timeout(Duration::from_secs(1), async {
                    tokio::select! {
                        route = routes.recv() => route.unwrap(),
                        result = &mut request => panic!("admission ended before enqueue: {result:?}"),
                    }
                }).await.expect("exact application/control capacity must be reusable");
                assert_eq!(
                    super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink).await,
                    HandleResult::Ok
                );
                let (request_id, snapshot) = state.pending_requests.to_vec().pop().unwrap();
                retired.push(snapshot.clone());
                let mut envelope = crate::types::Object::new();
                envelope.insert("id", Value::Number(Number::Int(request_id)));
                envelope.insert("session", Value::Uuid(id.into()));
                let binary = if undecodable {
                    vec![0xff]
                } else {
                    surrealdb_types::encode(&Value::Object(envelope)).unwrap()
                };
                if cancel_handler {
                    snapshot
                        .response_channel
                        .try_send(Err(TypesError::internal("full response queue".into())))
                        .unwrap();
                }
                let mut handler = Box::pin(handle_parse_error(
                    TypesError::internal("malformed".into()),
                    &binary,
                    &sessions,
                ));
                if cancel_handler {
                    std::future::poll_fn(|cx| {
                        assert!(
                            handler.as_mut().poll(cx).is_pending(),
                            "error delivery is blocked"
                        );
                        std::task::Poll::Ready(())
                    })
                    .await;
                    assert!(
                        state.pending_requests.is_empty(),
                        "take happened before cancellation"
                    );
                    drop(handler);
                } else {
                    assert_eq!(handler.await, HandleResult::Ok);
                }
                assert!(request.await.is_err());
                assert!(
                    activity_weak.upgrade().is_none(),
                    "terminal take clears activity with all retired metadata held"
                );
                assert_ne!(
                    context.dispatch_state(),
                    crate::opt::DispatchState::Completed,
                    "malformed response never establishes DB completion"
                );
                // Explicit completion/reset are idempotent with all snapshots held.
                super::reset_sessions(&sessions).await;
                super::reset_sessions(&sessions).await;
            }
            assert_eq!(retired.len(), iterations);
            assert!(retired.iter().all(|pending| pending._admission.is_some()));
            for pending in &retired {
                pending._admission.as_ref().unwrap().release();
                pending._admission.as_ref().unwrap().release();
            }
        }
    }

    #[tokio::test]
    async fn named_malformed_nonpayload_releases_admission_with_held_metadata() {
        malformed_nonpayload_admission(false, false).await;
    }
    #[tokio::test]
    async fn undecodable_nonpayload_releases_admission_with_held_metadata() {
        malformed_nonpayload_admission(true, false).await;
    }
    #[tokio::test]
    async fn cancelled_named_malformed_handler_releases_admission_after_take() {
        malformed_nonpayload_admission(false, true).await;
    }
    #[tokio::test]
    async fn cancelled_undecodable_handler_releases_admission_after_take() {
        malformed_nonpayload_admission(true, true).await;
    }

    /// Regression for the clone-setup race. A session inherits its namespace,
    /// authentication and variables by replaying commands at the server, which is
    /// free to apply requests that arrive together in any order. So a request must
    /// not go out while that replay is unacknowledged, and must be released once
    /// it lands — otherwise it can execute against a half-built session.
    #[tokio::test]
    async fn route_waits_for_replay_acknowledgement() {
        let (session_id, session_state, sessions) = session_with_replay(vec![set_cmd("x", 1)]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();
        assert_eq!(
            sink.read().await.len(),
            1,
            "the replay command should be on the wire"
        );

        let (route, _receiver) = route_for(session_id);
        assert_eq!(
            handle_route::<MockMessage, _, _>(route, None, &sessions, &sink).await,
            HandleResult::Ok
        );
        assert_eq!(
            sink.read().await.len(),
            1,
            "request must not reach the wire before the session's setup is acknowledged"
        );
        assert_eq!(
            session_state.deferred.lock().unwrap().len(),
            1,
            "request should be parked"
        );

        // The acknowledgement lands, completing the session's setup.
        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Ok(DbResult::Other(Value::None)),
        )
        .await;

        assert!(
            replay_cursor(&session_state).is_none(),
            "replay should be finished"
        );
        assert!(
            session_state.deferred.lock().unwrap().is_empty(),
            "parked request should be released once setup is acknowledged"
        );
        assert_eq!(
            sink.read().await.len(),
            2,
            "released request should be sent"
        );
        assert_eq!(
            session_state.pending_requests.len(),
            1,
            "released request should be pending"
        );
    }

    /// The replay log is order-sensitive: two `set`s of one key leave different
    /// end states depending on which lands last. The server may apply requests
    /// that arrive together in any order, so the commands must go one at a time,
    /// each only after the previous has been acknowledged.
    #[tokio::test]
    async fn replay_commands_are_sent_one_at_a_time() {
        let (session_id, session_state, sessions) =
            session_with_replay(vec![set_cmd("x", 1), set_cmd("x", 2)]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();
        assert_eq!(
            sink.read().await.len(),
            1,
            "only the first command may be in flight; the second would be free to overtake it"
        );
        let first = replay_cursor(&session_state).expect("first command in flight");

        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Ok(DbResult::Other(Value::None)),
        )
        .await;

        assert_eq!(
            sink.read().await.len(),
            2,
            "the second command should follow its predecessor's acknowledgement"
        );
        let second = replay_cursor(&session_state).expect("second command in flight");
        assert_ne!(
            second.awaiting, first.awaiting,
            "each command needs its own request id"
        );

        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Ok(DbResult::Other(Value::None)),
        )
        .await;

        assert!(
            replay_cursor(&session_state).is_none(),
            "the session should be ready once the log is exhausted"
        );
    }

    fn refreshable_auth_cmd() -> Command {
        Command::Authenticate {
            token: Token::WithRefresh {
                access: "expired-access".to_string(),
                refresh: "valid-refresh".to_string(),
            },
        }
    }

    fn token_expired() -> TypesError {
        TypesError::not_allowed(
            "token expired".to_string(),
            NotAllowedError::Auth(AuthError::TokenExpired),
        )
    }

    /// An access token that expired while the handle sat idle is routine, and the
    /// token carries its own refresh half. Poisoning the session would strand a
    /// handle that holds perfectly good credentials — a poisoned session is never
    /// replayed again and cannot route even an explicit refresh — so the command
    /// is retried once with the refresh token instead.
    #[tokio::test]
    async fn expired_token_during_replay_is_refreshed_not_poisoned() {
        let (session_id, session_state, sessions) =
            session_with_replay(vec![refreshable_auth_cmd(), set_cmd("x", 1)]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();
        let first = replay_cursor(&session_state).expect("authenticate in flight");

        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Err(token_expired()),
        )
        .await;

        assert!(
            matches!(sessions.get(&session_id), Some(Ok(_))),
            "a refreshable expiry must not poison the session"
        );
        let retry = replay_cursor(&session_state).expect("replay should still be in flight");
        assert_eq!(
            retry.index, first.index,
            "the retry re-sends the same command"
        );
        assert!(
            retry.refreshed,
            "the retry should be recorded so it happens only once"
        );
        assert_eq!(
            sink.read().await.len(),
            2,
            "the refreshed authenticate should be sent"
        );

        // The refreshed attempt succeeds, so the replay carries on to the `set`.
        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Ok(DbResult::Other(Value::None)),
        )
        .await;

        let next = replay_cursor(&session_state).expect("set should now be in flight");
        assert_eq!(
            next.index,
            first.index + 1,
            "the replay should advance past authenticate"
        );
        assert!(
            !next.refreshed,
            "a fresh command starts with no retry spent"
        );
    }

    /// The retry is bounded: an expiry the refresh token cannot fix must fail the
    /// session rather than loop.
    #[tokio::test]
    async fn expired_token_is_refreshed_only_once() {
        let (session_id, session_state, sessions) =
            session_with_replay(vec![refreshable_auth_cmd()]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();
        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Err(token_expired()),
        )
        .await;
        assert!(replay_cursor(&session_state).is_some_and(|c| c.refreshed));

        // The refreshed attempt is rejected too.
        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Err(token_expired()),
        )
        .await;

        assert!(
            replay_cursor(&session_state).is_none(),
            "the replay should be abandoned"
        );
        assert!(
            matches!(sessions.get(&session_id), Some(Err(_))),
            "a refresh that cannot recover should poison the session"
        );
        assert_eq!(sink.read().await.len(), 2, "no further refresh attempts");
    }

    /// A reply whose envelope decodes but carries no usable request id could be
    /// the acknowledgement a replay is waiting on. Leaving the cursor set would
    /// park every later request for that session forever.
    #[tokio::test]
    async fn response_without_usable_id_releases_a_parked_replay() {
        let (session_id, session_state, sessions) = session_with_replay(vec![set_cmd("x", 1)]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();
        let (route, receiver) = route_for(session_id);
        handle_route::<MockMessage, _, _>(route, None, &sessions, &sink).await;
        assert_eq!(session_state.deferred.lock().unwrap().len(), 1);

        // An error envelope naming the session but with no id at all.
        let mut envelope = crate::types::Object::new();
        envelope.insert("id".to_string(), Value::None);
        envelope.insert("session".to_string(), Value::Uuid(session_id.into()));
        let binary = surrealdb_types::encode(&Value::Object(envelope)).unwrap();

        handle_parse_error(
            TypesError::internal("unreadable".to_string()),
            &binary,
            &sessions,
        )
        .await;

        assert!(
            replay_cursor(&session_state).is_none(),
            "the stuck replay should be abandoned"
        );
        assert!(
            receiver.recv().await.unwrap().is_err(),
            "the parked request should be failed, not left hanging"
        );
    }

    /// A reply that cannot be decoded is the acknowledgement a replay was waiting
    /// on. Without releasing it the session parks every later request behind an
    /// acknowledgement that will never arrive.
    #[tokio::test]
    async fn undecodable_response_releases_a_parked_replay() {
        let (session_id, session_state, sessions) = session_with_replay(vec![set_cmd("x", 1)]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();

        let (route, receiver) = route_for(session_id);
        handle_route::<MockMessage, _, _>(route, None, &sessions, &sink).await;
        assert_eq!(session_state.deferred.lock().unwrap().len(), 1);

        fail_all_pending_requests(&sessions, TypesError::internal("undecodable".to_string())).await;

        assert!(
            replay_cursor(&session_state).is_none(),
            "the stuck replay should be abandoned"
        );
        assert!(
            receiver.recv().await.unwrap().is_err(),
            "the parked request should be failed, not left hanging"
        );
        assert!(
            matches!(sessions.get(&session_id), Some(Err(_))),
            "a session whose setup cannot be confirmed should be poisoned"
        );
    }

    /// A session whose setup the server rejected never reaches the state the
    /// caller asked for, so it is poisoned and the requests parked behind it are
    /// failed rather than run against a half-built session.
    #[tokio::test]
    async fn failed_replay_acknowledgement_fails_parked_requests() {
        let (session_id, session_state, sessions) = session_with_replay(vec![set_cmd("x", 1)]);
        let sink = RwLock::new(Vec::<MockMessage>::new());

        replay_session::<MockMessage, _, _>(session_id, &session_state, &sink)
            .await
            .unwrap();
        let sent_during_replay = sink.read().await.len();

        let (route, receiver) = route_for(session_id);
        handle_route::<MockMessage, _, _>(route, None, &sessions, &sink).await;
        assert_eq!(session_state.deferred.lock().unwrap().len(), 1);

        ack_replay(
            session_id,
            &session_state,
            &sessions,
            &sink,
            Err(TypesError::internal("setup rejected".to_string())),
        )
        .await;

        assert_eq!(
            sink.read().await.len(),
            sent_during_replay,
            "a rejected setup must not release the request"
        );
        assert!(
            receiver.recv().await.unwrap().is_err(),
            "the parked request should be failed, not left hanging"
        );
        assert!(
            matches!(sessions.get(&session_id), Some(Err(_))),
            "the session should be poisoned once its setup is rejected"
        );
    }

    /// Delivers one notification for `live` to a session that has it
    /// registered, and reports whether the registration survived.
    async fn deliver(action: Action) -> (bool, bool) {
        let session_state = Arc::new(SessionState::default());
        let live = Uuid::new_v4();
        let (sender, subscriber) = async_channel::unbounded();
        session_state.live_queries.insert(live, sender);

        let sink = RwLock::new(futures::sink::drain::<MockMessage>());
        let result = handle_live_notification::<MockMessage, _, _>(
            Ok(DbResult::Live(Notification::new(
                live.into(),
                None,
                action,
                Value::None,
                Value::None,
            ))),
            Uuid::new_v4(),
            &session_state,
            &sink,
        )
        .await;

        assert_eq!(result, HandleResult::Ok);
        (
            subscriber.try_recv().is_ok(),
            session_state.live_queries.contains_key(&live),
        )
    }

    /// A `Killed` notification is the subscription's end, so the registration
    /// goes with it -- otherwise it holds the subscriber's channel until the
    /// connection closes, and nothing later arrives to clear it.
    #[tokio::test]
    async fn a_killed_notification_takes_the_registration_with_it() {
        let (delivered, registered) = deliver(Action::Killed).await;

        assert!(
            delivered,
            "the subscriber is still owed the end of its subscription"
        );
        assert!(
            !registered,
            "the registration outlived the subscription it names"
        );
    }

    /// Every other notification leaves the registration in place: the
    /// subscription is still running, and more are coming.
    #[tokio::test]
    async fn a_change_notification_leaves_the_registration_in_place() {
        let (delivered, registered) = deliver(Action::Create).await;

        assert!(delivered, "the change reaches the subscriber");
        assert!(
            registered,
            "the subscription is still running and still the session's"
        );
    }
    struct FaultSink {
        attempted: Arc<std::sync::atomic::AtomicBool>,
        refuse_start: bool,
    }
    impl futures::Sink<MockMessage> for FaultSink {
        type Error = &'static str;
        fn poll_ready(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
        fn start_send(self: std::pin::Pin<&mut Self>, _: MockMessage) -> Result<(), Self::Error> {
            self.attempted
                .store(true, std::sync::atomic::Ordering::Release);
            if self.refuse_start {
                Err("uncertain partial transmission")
            } else {
                Ok(())
            }
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Pending
        }
        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn bounded_start_send_failure_preserves_unknown_dispatch() {
        let attempted = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sink = RwLock::new(FaultSink {
            attempted: Arc::clone(&attempted),
            refuse_start: true,
        });
        let context = crate::opt::RequestContext::new(
            std::time::Instant::now() + std::time::Duration::from_secs(1),
        );
        let result = super::send_request_message(&sink, MockMessage, Some(&context)).await;
        assert!(matches!(
            result,
            Err(super::SendFailure::Transport(
                "uncertain partial transmission"
            ))
        ));
        assert!(attempted.load(std::sync::atomic::Ordering::Acquire));
        assert!(context.was_dispatched());
        assert_eq!(
            context.dispatch_state(),
            crate::opt::DispatchState::Dispatched
        );
        assert!(
            context
                .interruption_error()
                .to_string()
                .contains("unknown server outcome")
        );
    }

    #[tokio::test]
    async fn bounded_stalled_flush_cancellation_preserves_unknown_dispatch() {
        let attempted = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sink = RwLock::new(FaultSink {
            attempted: Arc::clone(&attempted),
            refuse_start: false,
        });
        let context = crate::opt::RequestContext::new(
            std::time::Instant::now() + std::time::Duration::from_secs(1),
        );
        let cancel = async {
            while !attempted.load(std::sync::atomic::Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
            context.cancel();
        };
        let (result, ()) = tokio::join!(
            super::send_request_message(&sink, MockMessage, Some(&context)),
            cancel
        );
        assert!(matches!(
            result,
            Err(super::SendFailure::InterruptedAfterDispatch)
        ));
        assert!(context.was_dispatched());
        assert_eq!(
            context.dispatch_state(),
            crate::opt::DispatchState::Dispatched
        );
        assert!(
            context
                .interruption_error()
                .to_string()
                .contains("unknown server outcome")
        );
    }

    #[derive(Debug)]
    struct PayloadBudget {
        bytes: Vec<u8>,
        reserved: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl Drop for PayloadBudget {
        fn drop(&mut self) {
            self.reserved
                .fetch_sub(self.bytes.len(), std::sync::atomic::Ordering::AcqRel);
        }
    }
    impl PayloadBudget {
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

    #[derive(Clone)]
    struct OwnedResponseFrame {
        bytes: Vec<u8>,
        monitor: Option<(
            Arc<std::sync::atomic::AtomicUsize>,
            Arc<std::sync::atomic::AtomicUsize>,
            crate::opt::RequestContext,
        )>,
    }
    impl WsMessage for OwnedResponseFrame {
        fn binary(bytes: Vec<u8>) -> Self {
            Self {
                bytes,
                monitor: None,
            }
        }
        fn as_binary(&self) -> Option<&[u8]> {
            Some(&self.bytes)
        }
    }
    impl Drop for OwnedResponseFrame {
        fn drop(&mut self) {
            if let Some((reserved, dropped, context)) = &self.monitor {
                assert_eq!(
                    context.dispatch_state(),
                    crate::opt::DispatchState::Dispatched,
                    "raw frame must drain before response delivery/completion wakes the caller"
                );
                assert_eq!(
                    reserved.load(std::sync::atomic::Ordering::Acquire),
                    4096,
                    "actual binary frame must die while its original grant remains held"
                );
                dropped.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
            }
        }
    }
    fn owned_response_frame(
        session: Uuid,
        context: &crate::opt::RequestContext,
        reserved: &Arc<std::sync::atomic::AtomicUsize>,
        dropped: &Arc<std::sync::atomic::AtomicUsize>,
    ) -> OwnedResponseFrame {
        use crate::types::SurrealValue;
        let value = surrealdb_rpc::DbResponse::success(
            Some(Value::Number(Number::Int(0))),
            Some(session),
            DbResult::Other(Value::String("real decoded response".repeat(64))),
        )
        .into_value();
        OwnedResponseFrame {
            bytes: surrealdb_types::encode(&value).unwrap(),
            monitor: Some((reserved.clone(), dropped.clone(), context.clone())),
        }
    }

    #[tokio::test]
    async fn inbound_late_canceled_response_holds_grant_through_failed_send_and_frame_drop() {
        use surrealdb_engine_api::{EngineContext, RouteChannelEngine, SurrealEngine};
        let (sender, routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let sessions = no_sessions();
        let state = Arc::new(SessionState::default());
        let session = Uuid::new_v4();
        sessions.insert(session, Ok(state.clone()));
        let sink = RwLock::new(futures::sink::drain::<OwnedResponseFrame>());
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            crate::opt::RequestContext::new(std::time::Instant::now() + Duration::from_secs(30))
                .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
        let mut request = engine.query(
            EngineContext::new(session).with_request_context(Some(context.clone())),
            std::borrow::Cow::Borrowed("RETURN 1"),
            Default::default(),
        );
        let route = tokio::select! {
            route=routes.recv()=>route.unwrap(),
            result=&mut request=>panic!("ended before dispatch: {result:?}"),
        };
        assert_eq!(
            super::dispatch_route::<OwnedResponseFrame, _, _>(route, None, &state, &sink).await,
            HandleResult::Ok
        );
        let retired = state.pending_requests.values().into_iter().next().unwrap();
        context.cancel();
        assert!(request.await.is_err());
        assert!(PayloadBudget::reserve(&reserved, 4096, 4096).is_none());
        let frame = owned_response_frame(session, &context, &reserved, &dropped);
        assert_eq!(
            super::handle_owned_response(frame, None, &sessions, &sink).await,
            HandleResult::Ok
        );
        assert_eq!(dropped.load(std::sync::atomic::Ordering::Acquire), 1);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
        assert!(retired.request_context.is_some());
        assert!(state.pending_requests.is_empty());
    }

    #[tokio::test]
    async fn inbound_successful_queued_response_abandonment_drains_actual_values() {
        use surrealdb_engine_api::{EngineContext, RouteChannelEngine, SurrealEngine};
        let (sender, routes) = async_channel::bounded(1);
        let engine = RouteChannelEngine::new(sender);
        let sessions = no_sessions();
        let state = Arc::new(SessionState::default());
        let session = Uuid::new_v4();
        sessions.insert(session, Ok(state.clone()));
        let sink = RwLock::new(futures::sink::drain::<OwnedResponseFrame>());
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            crate::opt::RequestContext::new(std::time::Instant::now() + Duration::from_secs(30))
                .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
        let mut request = engine.query(
            EngineContext::new(session).with_request_context(Some(context.clone())),
            std::borrow::Cow::Borrowed("RETURN 1"),
            Default::default(),
        );
        let route = tokio::select! {
            route=routes.recv()=>route.unwrap(),
            result=&mut request=>panic!("ended before dispatch: {result:?}"),
        };
        assert_eq!(
            super::dispatch_route::<OwnedResponseFrame, _, _>(route, None, &state, &sink).await,
            HandleResult::Ok
        );
        let retired = state.pending_requests.values().into_iter().next().unwrap();
        let frame = owned_response_frame(session, &context, &reserved, &dropped);
        assert_eq!(
            super::handle_owned_response(frame, None, &sessions, &sink).await,
            HandleResult::Ok
        );
        assert_eq!(dropped.load(std::sync::atomic::Ordering::Acquire), 1);
        assert!(state.pending_requests.is_empty());
        assert_eq!(retired.response_channel.len(), 1);
        assert!(
            PayloadBudget::reserve(&reserved, 4096, 4096).is_none(),
            "successful enqueue still owns decoded response bytes after physical frame drain"
        );
        drop(request);
        assert_eq!(
            retired.response_channel.len(),
            0,
            "abandoned caller drains actual queued Values"
        );
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
        assert!(retired.request_context.is_some());
    }

    #[tokio::test]
    async fn inbound_response_handler_timeout_drops_decoded_send_and_frame_before_grant() {
        let sessions = no_sessions();
        let state = Arc::new(SessionState::default());
        let session = Uuid::new_v4();
        sessions.insert(session, Ok(state.clone()));
        let sink = RwLock::new(futures::sink::drain::<OwnedResponseFrame>());
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            crate::opt::RequestContext::new(std::time::Instant::now() + Duration::from_secs(30))
                .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
        let activity = Arc::new(());
        let weak = Arc::downgrade(&activity);
        let context = context.with_lifetime_owner(activity);
        context.mark_dispatched();
        let (sender, receiver) = async_channel::bounded(1);
        sender.send(Ok(Vec::new())).await.unwrap(); // Stall the actual decoded response send.
        let retired = PendingRequest {
            request_context: Some(context.clone()),
            _admission: None,
            interrupted_at: None,
            command: None,
            response_channel: sender,
        };
        state.pending_requests.insert(0, retired.clone());
        let frame = owned_response_frame(session, &context, &reserved, &dropped);
        let mut handler = Box::pin(super::handle_owned_response(frame, None, &sessions, &sink));
        std::future::poll_fn(|cx| {
            use std::future::Future;
            assert!(handler.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        assert!(
            state.pending_requests.is_empty(),
            "terminal response has taken correlation"
        );
        assert!(
            weak.upgrade().is_some(),
            "actual blocked decoded send still owns activity"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(20), handler)
                .await
                .is_err()
        );
        assert!(
            weak.upgrade().is_none(),
            "cancelled decoded send clears non-byte activity despite retired metadata"
        );
        assert_eq!(dropped.load(std::sync::atomic::Ordering::Acquire), 1);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
        assert!(state.pending_requests.is_empty());
        assert_eq!(
            receiver.len(),
            1,
            "blocked decoded response was dropped, not queued"
        );
        assert!(retired.request_context.is_some());
    }

    #[tokio::test]
    async fn authentic_response_restores_capacity_with_retired_pending_metadata_alive() {
        let sink = RwLock::new(futures::sink::drain::<MockMessage>());
        let sessions = no_sessions();
        let state = Arc::new(SessionState::default());
        let session = Uuid::new_v4();
        sessions.insert(session, Ok(state.clone()));
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let owner = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        let context = crate::opt::RequestContext::new(
            std::time::Instant::now() + std::time::Duration::from_secs(30),
        )
        .with_payload_owner(owner.clone());
        drop(owner);
        let (mut route, response) = route_for(session);
        route.request.request_context = Some(context.clone());
        route.request.command = Command::Query {
            txn: None,
            query: std::borrow::Cow::Borrowed("RETURN 1"),
            variables: Default::default(),
        };
        assert_eq!(
            super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink).await,
            HandleResult::Ok
        );
        let retired = state.pending_requests.values().into_iter().next().unwrap();
        assert!(
            retired.command.is_none(),
            "owner-bearing result Query never enters replay storage"
        );
        assert!(PayloadBudget::reserve(&reserved, 4096, 4096).is_none());
        assert_eq!(
            handle_response_with_id::<MockMessage, _, _>(
                0,
                Ok(DbResult::Other(Value::None)),
                session,
                &state,
                None,
                &sessions,
                &sink
            )
            .await,
            HandleResult::Ok
        );
        assert!(response.recv().await.unwrap().is_ok());
        assert_eq!(
            context.dispatch_state(),
            crate::opt::DispatchState::Completed
        );
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        assert!(
            retired.request_context.is_some(),
            "live and epoch-retired metadata do not own drained allocation bytes"
        );
        drop(restored);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn owner_bearing_replay_refuses_before_effects_and_restores_capacity() {
        let sink = RwLock::new(Vec::<MockMessage>::new());
        let state = Arc::new(SessionState::default());
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let owner = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        let context = crate::opt::RequestContext::new(
            std::time::Instant::now() + std::time::Duration::from_secs(30),
        )
        .with_payload_owner(owner.clone());
        drop(owner);
        let (mut route, response) = route_for(Uuid::new_v4());
        route.request.request_context = Some(context.clone());
        route.request.command = set_cmd("payload", 1);
        assert_eq!(
            super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink).await,
            HandleResult::Ok
        );
        assert!(
            response
                .recv()
                .await
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("not qualified")
        );
        assert!(sink.read().await.is_empty());
        assert!(state.pending_requests.is_empty());
        assert_eq!(state.replay.count(), 0);
        assert_eq!(
            context.dispatch_state(),
            crate::opt::DispatchState::NotDispatched
        );
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
    }

    #[tokio::test]
    async fn malformed_payload_keeps_activity_and_correlation_until_physical_drain() {
        for undecodable in [false, true] {
            let (session, state, sessions) = session_with_replay(Vec::new());
            let sink = RwLock::new(Vec::<MockMessage>::new());
            let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let activity = Arc::new(());
            let weak = Arc::downgrade(&activity);
            let context = crate::opt::RequestContext::new(
                std::time::Instant::now() + Duration::from_secs(30),
            )
            .with_lifetime_owner(activity)
            .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
            let (mut route, response) = route_for(session);
            route.request.request_context = Some(context.clone());
            route.request.command = Command::Query {
                txn: None,
                query: std::borrow::Cow::Borrowed("RETURN 1"),
                variables: Default::default(),
            };
            assert_eq!(
                super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink).await,
                HandleResult::Ok
            );
            let retired = state.pending_requests.values().into_iter().next().unwrap();
            let mut envelope = crate::types::Object::new();
            envelope.insert("id", Value::Number(Number::Int(0)));
            envelope.insert("session", Value::Uuid(session.into()));
            let binary = if undecodable {
                vec![0xff]
            } else {
                surrealdb_types::encode(&Value::Object(envelope)).unwrap()
            };
            assert_eq!(
                handle_parse_error(TypesError::internal("malformed".into()), &binary, &sessions)
                    .await,
                HandleResult::Disconnected
            );
            assert!(response.recv().await.unwrap().is_err());
            assert_eq!(
                state.pending_requests.len(),
                1,
                "owned malformed correlation remains reachable"
            );
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
            assert!(
                weak.upgrade().is_some(),
                "malformed payload cannot terminally release local activity"
            );
            assert_eq!(
                context.dispatch_state(),
                crate::opt::DispatchState::Dispatched
            );
            drop(sink); // Native reset is ordered after both physical socket halves.
            super::reset_sessions(&sessions).await;
            assert!(weak.upgrade().is_none());
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
            assert!(retired.request_context.is_some());
            super::reset_sessions(&sessions).await;
        }
    }

    #[tokio::test]
    async fn nonpayload_activity_survives_cancelled_dispatch_until_physical_drain() {
        for bounded in [false, true] {
            let attempted = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let sink = RwLock::new(FaultSink {
                attempted: attempted.clone(),
                refuse_start: false,
            });
            let (session, state, sessions) = session_with_replay(Vec::new());
            state
                .bounded
                .store(bounded, std::sync::atomic::Ordering::Release);
            let activity = Arc::new(());
            let weak = Arc::downgrade(&activity);
            let context = crate::opt::RequestContext::new(
                std::time::Instant::now() + Duration::from_secs(30),
            )
            .with_lifetime_owner(activity);
            let (mut route, _response) = route_for(session);
            route.request.request_context = Some(context.clone());
            let send = super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink);
            let cancel = async {
                while !attempted.load(std::sync::atomic::Ordering::Acquire) {
                    tokio::task::yield_now().await;
                }
                context.cancel();
            };
            let (result, ()) = tokio::join!(send, cancel);
            assert_eq!(result, HandleResult::Disconnected);
            let retired = state.pending_requests.values().into_iter().next().unwrap();
            assert!(context.payload_owner().is_none());
            assert!(
                weak.upgrade().is_some(),
                "caller cancellation cannot drain a dispatched activity"
            );
            assert_eq!(
                context.dispatch_state(),
                crate::opt::DispatchState::Dispatched
            );
            drop(sink); // Both native socket halves precede reset in the actual router.
            super::reset_sessions(&sessions).await;
            assert!(
                weak.upgrade().is_none(),
                "physical drain explicitly clears held metadata"
            );
            assert!(retired.request_context.is_some());
            assert_eq!(
                context.dispatch_state(),
                crate::opt::DispatchState::Dispatched
            );
            super::reset_sessions(&sessions).await;
        }
    }

    #[tokio::test]
    async fn nonpayload_replayable_activity_clears_on_ack_with_replay_and_metadata_alive() {
        let (session, state, sessions) = session_with_replay(Vec::new());
        let sink = RwLock::new(Vec::<MockMessage>::new());
        let activity = Arc::new(());
        let weak = Arc::downgrade(&activity);
        let context = crate::opt::RequestContext::control(
            std::time::Instant::now() + Duration::from_secs(30),
        )
        .with_lifetime_owner(activity);
        let (mut route, response) = route_for(session);
        route.request.request_context = Some(context.clone());
        route.request.command = set_cmd("activity", 1);
        assert_eq!(
            super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink).await,
            HandleResult::Ok
        );
        let retired = state.pending_requests.values().into_iter().next().unwrap();
        assert!(weak.upgrade().is_some());
        assert_eq!(
            handle_response_with_id::<MockMessage, _, _>(
                0,
                Ok(DbResult::Other(Value::None)),
                session,
                &state,
                None,
                &sessions,
                &sink
            )
            .await,
            HandleResult::Ok
        );
        assert!(response.recv().await.unwrap().is_ok());
        assert!(weak.upgrade().is_none());
        assert_eq!(
            state.replay.count(),
            1,
            "activity does not refuse or erase retained setup"
        );
        assert!(retired.command.is_some());
        assert!(context.payload_owner().is_none());
    }

    #[tokio::test]
    async fn stalled_payload_route_keeps_charge_until_physical_socket_invalidation() {
        for bounded in [false, true] {
            let attempted = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let sink = RwLock::new(FaultSink {
                attempted: attempted.clone(),
                refuse_start: false,
            });
            let sessions = no_sessions();
            let state = Arc::new(SessionState::default());
            state
                .bounded
                .store(bounded, std::sync::atomic::Ordering::Release);
            let session = Uuid::new_v4();
            sessions.insert(session, Ok(state.clone()));
            let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let owner = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
            let context = crate::opt::RequestContext::new(
                std::time::Instant::now() + std::time::Duration::from_secs(30),
            )
            .with_payload_owner(owner.clone());
            let (mut route, _response) = route_for(session);
            route.request.request_context = Some(context.clone());
            drop(owner);
            let send = super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink);
            let interrupt = async {
                while !attempted.load(std::sync::atomic::Ordering::Acquire) {
                    tokio::task::yield_now().await;
                }
                context.cancel();
            };
            let _ = tokio::join!(send, interrupt);
            let retired = state.pending_requests.values().into_iter().next().unwrap();
            assert_eq!(
                state.pending_requests.len(),
                1,
                "owner-bearing unbounded routes also stage before polling the sink"
            );
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
            assert!(PayloadBudget::reserve(&reserved, 4096, 4096).is_none());
            drop(sink); // Exactly the transport owner's physical invalidation ordering.
            super::reset_sessions(&sessions).await;
            assert_eq!(
                reserved.load(std::sync::atomic::Ordering::Acquire),
                0,
                "physical drain restores exact finite capacity without epoch collection"
            );
            let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
            assert!(retired.request_context.is_some());
            assert_eq!(
                context.dispatch_state(),
                crate::opt::DispatchState::Dispatched,
                "socket drain does not turn unknown server outcome into completed permission"
            );
            drop(restored);
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        }
    }

    struct ReadyStallSink {
        polled: Arc<std::sync::atomic::AtomicBool>,
    }
    impl futures::Sink<MockMessage> for ReadyStallSink {
        type Error = &'static str;
        fn poll_ready(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.polled
                .store(true, std::sync::atomic::Ordering::Release);
            std::task::Poll::Pending
        }
        fn start_send(self: std::pin::Pin<&mut Self>, _: MockMessage) -> Result<(), Self::Error> {
            panic!("canceled before start_send")
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn poll_ready_cancellation_restores_capacity_with_retired_pending_alive() {
        let polled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sink = RwLock::new(ReadyStallSink {
            polled: polled.clone(),
        });
        let state = Arc::new(SessionState::default());
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            crate::opt::RequestContext::new(std::time::Instant::now() + Duration::from_secs(30))
                .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
        let (mut route, response) = route_for(Uuid::new_v4());
        route.request.request_context = Some(context.clone());
        route.request.command = Command::Query {
            txn: None,
            query: std::borrow::Cow::Borrowed("RETURN 1"),
            variables: Default::default(),
        };
        let send = super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink);
        let interrupt = async {
            while !polled.load(std::sync::atomic::Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
            let retired = state.pending_requests.values().into_iter().next().unwrap();
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
            context.cancel();
            retired
        };
        let (result, retired) = tokio::join!(send, interrupt);
        assert_eq!(result, HandleResult::Ok);
        assert!(response.recv().await.unwrap().is_err());
        assert!(state.pending_requests.is_empty());
        assert!(retired.request_context.is_some());
        assert_eq!(
            context.dispatch_state(),
            crate::opt::DispatchState::NotDispatched
        );
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        let restored = PayloadBudget::reserve(&reserved, 4096, 4096).unwrap();
        drop(restored);
    }

    #[tokio::test]
    async fn deferred_flush_timeout_drains_all_enqueued_payloads_with_live_metadata() {
        let polled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sink = RwLock::new(ReadyStallSink {
            polled: polled.clone(),
        });
        let state = Arc::new(SessionState::default());
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut contexts = Vec::new();
        let mut responses = Vec::new();
        for _ in 0..3 {
            let context = crate::opt::RequestContext::new(
                std::time::Instant::now() + Duration::from_secs(30),
            )
            .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 12288).unwrap());
            context.mark_enqueued(); // Caller cleanup cannot reclaim a driver queue.
            let (mut route, response) = route_for(Uuid::new_v4());
            route.request.request_context = Some(context.clone());
            route.request.command = Command::Query {
                txn: None,
                query: std::borrow::Cow::Borrowed("RETURN 1"),
                variables: Default::default(),
            };
            super::park_route(&state, route);
            contexts.push(context);
            responses.push(response);
        }
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 12288);
        let flush = super::flush_deferred_routes::<MockMessage, _, _>(&state, None, &sink);
        assert!(
            tokio::time::timeout(Duration::from_millis(20), flush)
                .await
                .is_err()
        );
        assert!(polled.load(std::sync::atomic::Ordering::Acquire));
        let retired = state.pending_requests.values().into_iter().next().unwrap();
        assert!(retired.request_context.is_some());
        assert!(super::take_deferred_routes(&state).is_empty());
        assert!(contexts.iter().all(|context| context.dispatch_state()==crate::opt::DispatchState::NotDispatched));
        assert_eq!(
            reserved.load(std::sync::atomic::Ordering::Acquire),
            0,
            "active and untouched iterator routes drop actual bytes before clearing shared cells"
        );
        let restored = PayloadBudget::reserve(&reserved, 12288, 12288).unwrap();
        drop(restored);
        assert_eq!(responses.len(), 3);
    }

    #[tokio::test]
    async fn queued_cancellation_and_unpolled_driver_restore_capacity() {
        for parked in [false, true] {
            let sessions = no_sessions();
            let session = Uuid::new_v4();
            let state = Arc::new(SessionState::default());
            sessions.insert(session, Ok(state.clone()));
            let sink = RwLock::new(Vec::<MockMessage>::new());
            let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let context = crate::opt::RequestContext::new(
                std::time::Instant::now() + Duration::from_secs(30),
            )
            .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
            let (mut route, response) = route_for(session);
            route.request.request_context = Some(context.clone());
            if parked {
                super::park_route(&state, route);
                context.cancel();
                assert_eq!(
                    super::maintain_request_lifetimes(&sessions).await,
                    HandleResult::Ok
                );
                assert!(response.recv().await.unwrap().is_err());
                assert!(super::take_deferred_routes(&state).is_empty());
            } else {
                drop(super::handle_route::<MockMessage, _, _>(
                    route, None, &sessions, &sink,
                ));
            }
            assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
            assert!(sink.read().await.is_empty());
            assert_eq!(
                context.dispatch_state(),
                crate::opt::DispatchState::NotDispatched
            );
        }
    }

    #[tokio::test]
    async fn ordinary_session_drop_preserves_dispatched_owner_until_socket_drain() {
        let attempted = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sink = RwLock::new(FaultSink {
            attempted: attempted.clone(),
            refuse_start: false,
        });
        let sessions = no_sessions();
        let session = Uuid::new_v4();
        let state = Arc::new(SessionState::default());
        sessions.insert(session, Ok(state.clone()));
        let reserved = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context =
            crate::opt::RequestContext::new(std::time::Instant::now() + Duration::from_secs(30))
                .with_payload_owner(PayloadBudget::reserve(&reserved, 4096, 4096).unwrap());
        let (mut route, _response) = route_for(session);
        route.request.request_context = Some(context.clone());
        let send = super::dispatch_route::<MockMessage, _, _>(route, None, &state, &sink);
        let interrupt = async {
            while !attempted.load(std::sync::atomic::Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
            context.cancel();
        };
        let _ = tokio::join!(send, interrupt);
        let retired = state.pending_requests.values().into_iter().next().unwrap();
        assert_eq!(
            super::handle_session_drop::<MockMessage, _, _>(session, &sessions, &sink).await,
            HandleResult::Disconnected
        );
        assert!(sessions.contains_key(&session));
        assert_eq!(state.pending_requests.len(), 1);
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 4096);
        drop(sink);
        super::reset_sessions(&sessions).await;
        assert_eq!(reserved.load(std::sync::atomic::Ordering::Acquire), 0);
        assert!(retired.request_context.is_some());
        assert_eq!(
            context.dispatch_state(),
            crate::opt::DispatchState::Dispatched
        );
    }

    #[tokio::test]
    async fn bounded_submitted_drain_retains_correlation_until_ninety_seconds() {
        let sessions = no_sessions();
        let session = Arc::new(SessionState::default());
        let id = Uuid::new_v4();
        sessions.insert(id, Ok(Arc::clone(&session)));
        let context = crate::opt::RequestContext::new(
            std::time::Instant::now() + std::time::Duration::from_secs(1),
        );
        context.mark_dispatched();
        context.cancel();
        let (sender, receiver) = async_channel::bounded(1);
        let mut pending = PendingRequest {
            request_context: Some(context.clone()),
            _admission: None,
            interrupted_at: Some(std::time::Instant::now() - std::time::Duration::from_secs(89)),
            command: None,
            response_channel: sender,
        };
        session.pending_requests.insert(1, pending.clone());
        assert_eq!(
            super::maintain_request_lifetimes(&sessions).await,
            HandleResult::Ok
        );
        assert_eq!(session.pending_requests.len(), 1);
        assert!(
            receiver
                .recv()
                .await
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("unknown server outcome")
        );
        pending.interrupted_at =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(91));
        session.pending_requests.insert(1, pending);
        assert_eq!(
            super::maintain_request_lifetimes(&sessions).await,
            HandleResult::Disconnected
        );
        assert_eq!(
            session.pending_requests.len(),
            1,
            "correlation survives until the socket owner invalidates both halves"
        );
        // The native socket owner invokes this only after both halves drop;
        // actual socket teardown is covered by the separate mock-peer control.
        super::reset_sessions(&sessions).await;
        assert!(session.pending_requests.is_empty());
        assert!(context.was_dispatched());
    }

    #[tokio::test]
    async fn closed_response_channels_never_log_tokens_or_bindings() {
        use tracing::instrument::WithSubscriber;
        #[derive(Clone)]
        struct Captured(Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for Captured {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let bytes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let writer = Arc::clone(&bytes);
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || Captured(Arc::clone(&writer)))
            .finish();
        async {
            let session = Arc::new(SessionState::default());
            let sink = RwLock::new(futures::sink::drain::<MockMessage>());
            let sessions = no_sessions();
            let id = Uuid::new_v4();
            let mut credentials = crate::types::Object::new();
            credentials.insert("user", "credential-user-sentinel");
            credentials.insert("pass", "credential-password-sentinel");
            let query = super::QueryResultBuilder::started_now()
                .finish_with_result(Ok(Value::String("binding-sentinel".into())));
            for (request, result) in [
                (1, DbResult::Other(Value::String("token-sentinel".into()))),
                (2, DbResult::Query(vec![query])),
            ] {
                let (sender, receiver) = async_channel::bounded(1);
                drop(receiver);
                session.pending_requests.insert(
                    request,
                    PendingRequest {
                        request_context: None,
                        _admission: None,
                        interrupted_at: None,
                        command: Some(Command::Signin {
                            credentials: credentials.clone(),
                        }),
                        response_channel: sender,
                    },
                );
                assert_eq!(
                    handle_response_with_id::<MockMessage, _, _>(
                        request,
                        Ok(result),
                        id,
                        &session,
                        None,
                        &sessions,
                        &sink
                    )
                    .await,
                    HandleResult::Ok
                );
            }
        }
        .with_subscriber(subscriber)
        .await;
        let output = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
        assert!(output.contains("Setup response receiver closed"));
        assert!(output.contains("Query response receiver closed"));
        for secret in [
            "token-sentinel",
            "binding-sentinel",
            "credential-user-sentinel",
            "credential-password-sentinel",
        ] {
            assert!(!output.contains(secret));
        }
    }
}
