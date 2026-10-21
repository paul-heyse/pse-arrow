//! Finite native gRPC lifecycle. The owned HTTP/2 driver is the socket-drain
//! boundary; dropping a tonic request alone is not evidence of physical drain.

use std::{future::Future, path::PathBuf, pin::Pin, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}}, task::{Context, Poll}};
use bytes::{Bytes, BytesMut};
use http_body::{Body, Frame};
use hyper_util::rt::{TokioExecutor, TokioIo};
use surrealdb_engine_api::{DbExportConfig, EngineContext, EngineFuture, MlExportConfig, RequestContext, ResponseLimits};
use surrealdb_rpc::{QueryResult, Token};
use crate::types::{Array, Notification, Object, Value, Variables};
use uuid::Uuid;
use tokio::sync::{Notify, Semaphore};
use tower_service::Service;
use super::grpc::{GrpcEngine, EngineResult};
use crate::Error;

tokio::task_local! { pub(super) static REQUEST: RequestContext; }

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type HttpFuture = Pin<Box<dyn Future<Output = Result<http::Response<tonic::body::Body>, BoxError>> + Send>>;

#[derive(Debug)]
pub(super) struct Driver {
    closed: AtomicBool,
    closing: AtomicBool,
    stopping: Notify,
    joined: Notify,
    drained: AtomicBool,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    bootstrap: Arc<Mutex<Option<Bootstrap>>>,
}
struct Bootstrap {
    _lifetime: Option<surrealdb_engine_api::request::RequestLifetimeRetention>,
    _payload: Option<surrealdb_engine_api::request::PayloadRetention>,
}
impl std::fmt::Debug for Bootstrap { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("GrpcBootstrapOwnership") } }
struct DriverCompletion(std::sync::Weak<Driver>, Arc<Mutex<Option<Bootstrap>>>);
impl Drop for DriverCompletion {
    fn drop(&mut self) {
        self.1.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        if let Some(driver) = self.0.upgrade() {
            driver.closed.store(true, Ordering::Release);
            driver.drained.store(true, Ordering::Release);
            driver.stopping.notify_waiters();
            driver.joined.notify_waiters();
        }
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        if let Some(task) = self.task.get_mut().unwrap_or_else(std::sync::PoisonError::into_inner).take() { task.abort(); }
    }
}
impl Driver {
    fn new() -> Arc<Self> {
        Arc::new(Self { closed: AtomicBool::new(false), closing: AtomicBool::new(false), stopping: Notify::new(), joined: Notify::new(), drained: AtomicBool::new(false), task: Mutex::new(None), bootstrap: Arc::new(Mutex::new(REQUEST.try_with(|ctx| Bootstrap { _payload: ctx.payload_owner().map(|o| o.retain_driver()), _lifetime: ctx.lifetime_owner().map(|o| o.retain()) }).ok())) })
    }
    fn finish_setup(&self) { self.bootstrap.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take(); }
    pub(super) fn begin_close(&self) -> bool { !self.closing.swap(true, Ordering::AcqRel) }
    pub(super) fn accepting(&self) -> bool { !self.closed() && !self.closing.load(Ordering::Acquire) }
    pub(super) fn closed(&self) -> bool { self.closed.load(Ordering::Acquire) }
    pub(super) async fn stopping(&self) {
        loop {
            let wake = self.stopping.notified();
            tokio::pin!(wake); wake.as_mut().enable();
            if self.closed() { return; }
            wake.await;
        }
    }
    pub(super) async fn shutdown(&self) {
        self.closed.store(true, Ordering::Release);
        self.stopping.notify_waiters();
        let task = self.task.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        if let Some(task) = task {
            task.abort();
            let _ = task.await;
            // DriverCompletion signals only after both socket halves drop.
        }
        loop {
            let wake = self.joined.notified();
            tokio::pin!(wake); wake.as_mut().enable();
            if self.drained.load(Ordering::Acquire) { return; }
            wake.await;
        }
    }
}

/// Generic SDK profiles retain upstream tonic; the canonical bounded profile
/// always selects the owned, drainable HTTP/2 connection.
#[derive(Clone)]
pub(super) enum Transport {
    Owned { sender: hyper::client::conn::http2::SendRequest<tonic::body::Body>, driver: Arc<Driver>, authority: http::uri::Authority, limit: usize },
    Upstream(tonic::transport::Channel),
}
impl Transport {
    pub(super) async fn connect(url: &url::Url, limit: usize) -> EngineResult<(Self, Arc<Driver>)> {
        if url.scheme() != "grpc" || !url.username().is_empty() || url.password().is_some() {
            return Err(Error::configuration("Bounded gRPC requires an explicit authenticated loopback grpc endpoint".to_string(), None));
        }
        let host = url.host_str().ok_or_else(|| Error::configuration("Missing gRPC host".to_string(), None))?;
        let port = url.port().ok_or_else(|| Error::configuration("Missing gRPC port".to_string(), None))?;
        let address = tokio::net::lookup_host((host, port)).await.map_err(connection_error)?.find(|a| a.ip().is_loopback()).ok_or_else(|| Error::configuration("Bounded gRPC requires loopback".to_string(), None))?;
        let socket = tokio::net::TcpStream::connect(address).await.map_err(connection_error)?;
        let (sender, connection) = hyper::client::conn::http2::Builder::new(TokioExecutor::new())
            .max_frame_size(16 << 10)
            .max_header_list_size(64 << 10)
            .initial_stream_window_size(64 << 10)
            .initial_connection_window_size(64 << 10)
            .max_concurrent_streams(0)
            .handshake(TokioIo::new(socket)).await.map_err(connection_error)?;
        let driver = Driver::new();
        let owned = (Box::pin(connection), DriverCompletion(Arc::downgrade(&driver), driver.bootstrap.clone()));
        let task = tokio::spawn(async move {
            let (connection, _complete) = owned;
            let _ = connection.await;
        });
        *driver.task.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(task);
        let authority = address.to_string().parse().map_err(connection_error)?;
        Ok((Self::Owned { sender, driver: driver.clone(), authority, limit }, driver))
    }
}
fn connection_error(e: impl std::fmt::Display) -> Error { Error::connection(e.to_string(), None) }
fn refused(reason: &'static str) -> BoxError { std::io::Error::other(reason).into() }

impl Service<http::Request<tonic::body::Body>> for Transport {
    type Response = http::Response<tonic::body::Body>;
    type Error = BoxError;
    type Future = HttpFuture;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self {
            Self::Owned { sender, driver, .. } => {
                if driver.closed() { return Poll::Ready(Err(refused("gRPC connection closed"))); }
                sender.poll_ready(cx).map_err(|e| Box::new(e) as BoxError)
            }
            Self::Upstream(channel) => channel.poll_ready(cx).map_err(|e| Box::new(e) as BoxError),
        }
    }
    fn call(&mut self, mut request: http::Request<tonic::body::Body>) -> Self::Future {
        match self {
            Self::Upstream(channel) => { let future = channel.call(request); Box::pin(async move { future.await.map_err(|e| Box::new(e) as BoxError) }) }
            Self::Owned { sender, driver, authority, limit } => {
                let context = REQUEST.try_with(Clone::clone).ok();
                let response_type = super::grpc_wire_schema::response(request.uri().path());
                let driver = driver.clone();
                let limit = *limit;
                let mut parts = request.uri().clone().into_parts();
                parts.scheme = Some(http::uri::Scheme::HTTP);
                parts.authority = Some(authority.clone());
                *request.uri_mut() = http::Uri::from_parts(parts).expect("valid gRPC URI parts");
                if let Some(context) = &context { context.mark_dispatched(); }
                let future = sender.send_request(request);
                Box::pin(async move {
                    if driver.closed() { return Err(refused("gRPC connection closed")); }
                    let response = future.await.map_err(|e| Box::new(e) as BoxError)?;
                    let (parts, body) = response.into_parts();
                    // Bounded profiles never advertise compressed responses: validation
                    // runs on the exact protobuf bytes before prost can allocate.
                    if parts.headers.get("grpc-encoding").is_some_and(|h| h != "identity") {
                        driver.shutdown().await;
                        return Err(refused("compressed bounded gRPC response refused"));
                    }
                    let body = WireBody::new(body, context, response_type, limit);
                    Ok(http::Response::from_parts(parts, tonic::body::Body::new(body)))
                })
            }
        }
    }
}

/// Holds the prepaid response/lifetime grants while the physical response body
/// exists. It yields only complete, checked messages to tonic's protobuf codec.
struct WireBody {
    inner: Pin<Box<hyper::body::Incoming>>,
    pending: BytesMut,
    context: Option<RequestContext>,
    _payload: Option<surrealdb_engine_api::request::PayloadRetention>,
    _lifetime: Option<surrealdb_engine_api::request::RequestLifetimeRetention>,
    schema: Option<&'static str>,
    limit: usize,
    counts: super::grpc_wire::Counts,
    done: bool,
}
impl WireBody {
    fn new(body: hyper::body::Incoming, context: Option<RequestContext>, schema: Option<&'static str>, limit: usize) -> Self {
        Self { inner: Box::pin(body), pending: BytesMut::new(), _payload: context.as_ref().and_then(RequestContext::payload_owner).map(|o| o.retain_driver()), _lifetime: context.as_ref().and_then(RequestContext::lifetime_owner).map(|o| o.retain()), context, schema, limit, counts: Default::default(), done: false }
    }
}
impl Body for WireBody {
    type Data = Bytes;
    type Error = BoxError;
    fn poll_frame(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        if self.done { return Poll::Ready(None); }
        loop {
            if self.pending.len() >= 5 {
                if self.pending[0] != 0 { self.done = true; return Poll::Ready(Some(Err(refused("compressed gRPC message refused")))); }
                let len = u32::from_be_bytes(self.pending[1..5].try_into().expect("four byte prefix")) as usize;
                let limits = self.context.as_ref().and_then(RequestContext::response_limits).unwrap_or_else(|| default_limits(self.limit));
                if len > self.limit || len > limits.wire_bytes { self.done = true; return Poll::Ready(Some(Err(refused("gRPC frame exceeds response envelope")))); }
                if self.pending.len() >= len + 5 {
                    let frame = self.pending.split_to(len + 5).freeze();
                    let schema = self.schema;
                    if let Err(reason) = self.counts.check(&frame[5..], schema, limits) { self.done = true; return Poll::Ready(Some(Err(refused(reason)))); }
                    return Poll::Ready(Some(Ok(Frame::data(frame))));
                }
            }
            match self.inner.as_mut().poll_frame(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(Ok(frame))) => match frame.into_data() {
                    Ok(data) => {
                        if self.pending.len().saturating_add(data.len()) > self.limit.saturating_add(5).saturating_add(64 << 10) { self.done = true; return Poll::Ready(Some(Err(refused("gRPC encoded buffer exceeds envelope")))); }
                        self.pending.extend_from_slice(&data);
                    }
                    Err(frame) => {
                        self.done = true;
                        if !self.pending.is_empty() { return Poll::Ready(Some(Err(refused("truncated gRPC message")))); }
                        return Poll::Ready(Some(Ok(frame)));
                    }
                },
                Poll::Ready(Some(Err(error))) => { self.done = true; return Poll::Ready(Some(Err(Box::new(error)))); }
                Poll::Ready(None) => { self.done = true; return if self.pending.is_empty() { Poll::Ready(None) } else { Poll::Ready(Some(Err(refused("truncated gRPC message")))) }; }
            }
        }
    }
}
fn default_limits(wire_bytes: usize) -> ResponseLimits {
    ResponseLimits { wire_bytes, nodes: 1 << 20, depth: 64, container_items: 1 << 20, string_bytes: wire_bytes, string_length: wire_bytes, blob_bytes: wire_bytes, blob_length: wire_bytes }
}

const APPLICATION_SLOTS: usize = 32;
const CONTROL_SLOTS: usize = 2;
const APPLICATION_WAITERS: usize = 32;
const CONTROL_WAITERS: usize = 32;

#[derive(Clone)]
pub(super) struct BoundedEngine { state: Arc<State> }
struct State {
    endpoint: crate::opt::Endpoint,
    generation: tokio::sync::Mutex<Arc<GrpcEngine>>,
    selected: Mutex<Option<(uuid::Uuid, Arc<super::grpc::GrpcSession>)>>,
    local: std::sync::OnceLock<uuid::Uuid>,
    announced: Notify,
    disconnected: AtomicBool,
    admission: Mutex<()>,
    active: AtomicUsize,
    work_drained: Notify,
    application: Arc<Semaphore>, control: Arc<Semaphore>,
    queued: Arc<Semaphore>, queued_control: Arc<Semaphore>,
}
impl std::fmt::Debug for BoundedEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("BoundedGrpcEngine").finish_non_exhaustive() }
}
impl State {
    async fn current(&self) -> Arc<GrpcEngine> { self.generation.lock().await.clone() }
    async fn shutdown(&self) {
        {
            let _admission = self.admission.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            self.disconnected.store(true, Ordering::Release);
        }
        self.current().await.disconnect_bounded().await;
        loop {
            let wake = self.work_drained.notified();
            tokio::pin!(wake); wake.as_mut().enable();
            if self.active.load(Ordering::Acquire) == 0 { break; }
            wake.await;
        }
    }
    async fn ready(&self, ctx: &EngineContext, request: &RequestContext) -> EngineResult<Arc<GrpcEngine>> {
        loop {
            let announced = self.announced.notified();
            tokio::pin!(announced); announced.as_mut().enable();
            if let Some(local) = self.local.get() {
                if *local != ctx.session { return Err(Error::configuration("Bounded gRPC permits one application session".to_string(), None)); }
                break;
            }
            tokio::select! { biased; _ = request.interrupted() => return Err(request.interruption_error()), _ = announced => {} }
        }
        let lock = self.generation.lock();
        let mut generation = tokio::select! { biased;
            _ = request.interrupted() => return Err(request.interruption_error()),
            lock = lock => lock,
        };
        if self.disconnected.load(Ordering::Acquire) { return Err(Error::connection("gRPC connection explicitly disconnected".to_string(), None)); }
        if generation.driver.as_ref().is_some_and(|driver| driver.accepting()) { return Ok(generation.clone()); }
        if request.was_dispatched() { return Err(request.interruption_error()); }
        // The failed generation's actual socket/codec owners retire before any
        // new connection can become visible. Application RPCs are never replayed.
        if let Some(driver) = &generation.driver { driver.shutdown().await; }
        let deadline = request.deadline().min(std::time::Instant::now() + std::time::Duration::from_secs(5));
        let selected = self.selected.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        let Some((local, acknowledged)) = selected else { return Err(Error::connection("No acknowledged gRPC session is available for restoration".to_string(), None)); };
        if local != ctx.session { return Err(Error::configuration("Bounded gRPC permits one application session".to_string(), None)); }
        let log = acknowledged.log();
        let checkpoint = *acknowledged.checkpoint.lock().expect("checkpoint registry poisoned");
        let setup = RequestContext::control(deadline).inherit_request_ownership(request);
        let connect = REQUEST.scope(setup.clone(), super::grpc::connect_generation(&self.endpoint));
        let fresh = tokio::select! { biased;
            _ = request.interrupted() => return Err(request.interruption_error()),
            result = tokio::time::timeout_at(deadline.into(), connect) => result.map_err(|_| Error::connection("gRPC restoration deadline expired".to_string(), None))??.0,
        };
        // Keep a failed new generation visible as failed, while selected retains
        // the complete old ACK snapshot rather than partially restored state.
        *generation = fresh.clone();
        let restore = REQUEST.scope(setup, async {
            let server = fresh.attach().await?;
            let session = super::grpc::GrpcSession::new(server, true);
            for op in log {
                fresh.apply(fresh.context(server, None), op.clone()).await?;
                session.record(op);
            }
            *session.checkpoint.lock().expect("checkpoint registry poisoned") = checkpoint;
            Ok::<_, Error>(session)
        });
        let result = tokio::select! { biased;
            _ = request.interrupted() => Err(request.interruption_error()),
            result = tokio::time::timeout_at(deadline.into(), restore) => result.unwrap_or_else(|_| Err(Error::connection("gRPC restoration deadline expired".to_string(), None))),
        };
        match result {
            Ok(session) => {
                if let Some(driver) = &fresh.driver { driver.finish_setup(); }
                if self.disconnected.load(Ordering::Acquire) {
                    fresh.disconnect_bounded().await;
                    return Err(Error::connection("gRPC connection explicitly disconnected".to_string(), None));
                }
                fresh.publish(local, Ok(session.clone()));
                *self.selected.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((local, session));
                Ok(fresh)
            }
            Err(error) => { if let Some(driver) = &fresh.driver { driver.shutdown().await; } Err(error) }
        }
    }
}
struct Active(Arc<State>);
impl Drop for Active {
    fn drop(&mut self) { self.0.active.fetch_sub(1, Ordering::AcqRel); self.0.work_drained.notify_waiters(); }
}
struct Cancel(RequestContext);
impl Drop for Cancel { fn drop(&mut self) { if self.0.dispatch_state() != surrealdb_engine_api::DispatchState::Completed { self.0.cancel(); } } }
struct Drain(RequestContext);
impl Drop for Drain { fn drop(&mut self) { self.0.release_lifetime_after_drain(); self.0.release_payload_after_drain(); } }
impl BoundedEngine {
    pub(super) fn new(inner: Arc<GrpcEngine>, endpoint: crate::opt::Endpoint) -> Self {
        Self { state: Arc::new(State { endpoint, generation: tokio::sync::Mutex::new(inner), selected: Mutex::new(None), local: std::sync::OnceLock::new(), announced: Notify::new(), disconnected: AtomicBool::new(false), admission: Mutex::new(()), active: AtomicUsize::new(0), work_drained: Notify::new(), application: Arc::new(Semaphore::new(APPLICATION_SLOTS)), control: Arc::new(Semaphore::new(CONTROL_SLOTS)), queued: Arc::new(Semaphore::new(APPLICATION_WAITERS)), queued_control: Arc::new(Semaphore::new(CONTROL_WAITERS)) }) }
    }
    pub(super) fn session_owner(&self, receiver: async_channel::Receiver<crate::SessionId>) {
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    crate::SessionId::Initial(local) => {
                        if state.local.set(local).is_err() { continue; }
                        state.announced.notify_waiters();
                        let engine = state.current().await;
                        let established = tokio::time::timeout(std::time::Duration::from_secs(5), engine.attach()).await
                            .map_err(|_| Error::connection("gRPC session attachment deadline expired".to_string(), None))
                            .and_then(|result| result).map(|server| super::grpc::GrpcSession::new(server, true));
                        if let Ok(session) = &established { *state.selected.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((local, session.clone())); }
                        else { engine.disconnect_bounded().await; }
                        engine.publish(local, established);
                    }
                    crate::SessionId::Clone { .. } => {
                        // ready() rejects every non-owner ID before lookup;
                        // remembering refused clones would itself be unbounded.
                    }
                    crate::SessionId::Drop(local) => {
                        let final_owner = state.local.get().is_some_and(|id| *id == local);
                        if final_owner { state.shutdown().await; break; }
                    }
                }
            }
            state.shutdown().await;
        });
    }
    fn execute<T: Send + 'static, F>(&self, ctx: EngineContext, operation: F) -> EngineFuture<'_, T>
    where F: FnOnce(Arc<GrpcEngine>, EngineContext) -> Pin<Box<dyn Future<Output = EngineResult<T>> + Send>> + Send + 'static {
        let state = self.state.clone();
        Box::pin(async move {
            let context = ctx.request_context.clone().ok_or_else(|| Error::configuration("Bounded gRPC requires an original RequestContext".to_string(), None))?;
            let _undispatched = surrealdb_engine_api::request::UndispatchedPayloadDrain::caller(Some(context.clone()));
            let _cancel = Cancel(context.clone());
            let _receiver_payload = context.payload_owner().map(|owner| owner.retain());
            let (waiting, immediate, active) = {
                let _admission = state.admission.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.disconnected.load(Ordering::Acquire) { return Err(Error::connection("gRPC connection explicitly disconnected".to_string(), None)); }
                let semaphore = if context.is_control() { state.control.clone() } else { state.application.clone() };
                let immediate = semaphore.try_acquire_owned().ok();
                let waiting = if immediate.is_none() {
                    let queued = if context.is_control() { state.queued_control.clone() } else { state.queued.clone() };
                    Some(queued.try_acquire_owned().map_err(|_| Error::connection("Bounded gRPC admission queue full".to_string(), None))?)
                } else { None };
                context.mark_enqueued();
                state.active.fetch_add(1, Ordering::AcqRel);
                (waiting, immediate, Active(state.clone()))
            };
            let task_context = context.clone();
            let (send, receive) = tokio::sync::oneshot::channel();
            tokio::spawn(async move {
                let _active = active;
                let _drain = Drain(task_context.clone());
                let _payload = task_context.payload_owner().map(|o| o.retain_driver());
                let _lifetime = task_context.lifetime_owner().map(|o| o.retain());
                let semaphore = if task_context.is_control() { state.control.clone() } else { state.application.clone() };
                let permit = match immediate {
                    Some(permit) => Ok(permit),
                    None => tokio::select! { biased;
                        _ = task_context.interrupted() => { let _ = send.send(Err(task_context.interruption_error())); return; },
                        permit = semaphore.acquire_owned() => permit,
                    },
                };
                let Ok(_permit) = permit else { return; };
                drop(waiting);
                let engine = match state.ready(&ctx, &task_context).await {
                    Ok(engine) => engine,
                    Err(error) => { let _ = send.send(Err(error)); return; },
                };
                let driver = engine.driver.as_ref().expect("bounded engine owns its driver").clone();
                let outcome = {
                    let future = REQUEST.scope(task_context.clone(), operation(engine, ctx));
                    tokio::pin!(future);
                    tokio::select! { biased;
                        _ = task_context.interrupted() => {
                            // Dropping a waiter does not cancel an already
                            // dispatched shared-connection operation. Retain its
                            // original owner until late completion or original
                            // deadline, while independent callers can progress.
                            if task_context.was_dispatched() && std::time::Instant::now() < task_context.deadline() {
                                tokio::select! { biased;
                                    _ = tokio::time::sleep_until(task_context.deadline().into()) => None,
                                    _ = driver.stopping() => None,
                                    result = &mut future => Some(result),
                                }
                            } else { None }
                        },
                        _ = driver.stopping() => None,
                        result = &mut future => Some(result),
                    }
                };
                let outcome = match outcome {
                    Some(result) => {
                        if result.as_ref().is_err_and(|error| error.is_connection() || error.is_serialization()) { driver.shutdown().await; }
                        else if task_context.was_dispatched() { task_context.mark_completed(); }
                        result
                    },
                    None => {
                        if task_context.was_dispatched() { driver.shutdown().await; }
                        Err(task_context.interruption_error())
                    }
                };
                let _ = send.send(outcome);
            });
            tokio::select! { biased;
                _ = context.interrupted() => Err(context.interruption_error()),
                result = receive => result.map_err(|_| Error::connection("gRPC driver task failed".to_string(), None))?,
            }
        })
    }
}

impl surrealdb_engine_api::SurrealEngine for BoundedEngine {
    fn disconnect(&self) -> EngineFuture<'_, ()> { Box::pin(async move { self.state.shutdown().await; Ok(()) }) }
    fn query(
		&self,
		ctx: EngineContext,
		query: std::borrow::Cow<'static, str>,
		variables: Variables,
	) -> EngineFuture<'_, Vec<QueryResult>> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.query(ctx, query, variables).await }))
    }
    fn run(
		&self,
		ctx: EngineContext,
		name: String,
		version: Option<String>,
		args: Array,
	) -> EngineFuture<'_, Value> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.run(ctx, name, version, args).await }))
    }
    fn use_ns_db(
		&self,
		ctx: EngineContext,
		namespace: Option<String>,
		database: Option<String>,
	) -> EngineFuture<'_, (Option<String>, Option<String>)> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.use_ns_db(ctx, namespace, database).await }))
    }
    fn set(&self, ctx: EngineContext, key: String, value: Value) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.set(ctx, key, value).await }))
    }
    fn unset(&self, ctx: EngineContext, key: String) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.unset(ctx, key).await }))
    }
    fn signup(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.signup(ctx, credentials).await }))
    }
    fn signin(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.signin(ctx, credentials).await }))
    }
    fn authenticate(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.authenticate(ctx, token).await }))
    }
    fn refresh(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.refresh(ctx, token).await }))
    }
    fn revoke(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.revoke(ctx, token).await }))
    }
    fn invalidate(&self, ctx: EngineContext) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.invalidate(ctx).await }))
    }
    fn begin(&self, ctx: EngineContext) -> EngineFuture<'_, Uuid> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.begin(ctx).await }))
    }
    fn commit(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.commit(ctx, txn).await }))
    }
    fn rollback(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.rollback(ctx, txn).await }))
    }
    fn health(&self, ctx: EngineContext) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.health(ctx).await }))
    }
    fn version(&self, ctx: EngineContext) -> EngineFuture<'_, String> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.sessions.resolve(ctx.session).await?; engine.version(ctx).await }))
    }
    fn kill(&self, ctx: EngineContext, uuid: Uuid) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.kill(ctx, uuid).await }))
    }
    fn subscribe_live(
		&self,
		ctx: EngineContext,
		uuid: Uuid,
		notifications: async_channel::Sender<Result<Notification, Error>>,
	) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.subscribe_live(ctx, uuid, notifications).await }))
    }
    fn export_file(
		&self,
		ctx: EngineContext,
		path: PathBuf,
		config: Option<DbExportConfig>,
	) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.export_file(ctx, path, config).await }))
    }
    fn export_bytes(
		&self,
		ctx: EngineContext,
		bytes: async_channel::Sender<Result<Vec<u8>, Error>>,
		config: Option<DbExportConfig>,
	) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.export_bytes(ctx, bytes, config).await }))
    }
    fn export_ml_file(
		&self,
		ctx: EngineContext,
		path: PathBuf,
		config: MlExportConfig,
	) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.export_ml_file(ctx, path, config).await }))
    }
    fn export_ml_bytes(
		&self,
		ctx: EngineContext,
		bytes: async_channel::Sender<Result<Vec<u8>, Error>>,
		config: MlExportConfig,
	) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.export_ml_bytes(ctx, bytes, config).await }))
    }
    fn import_file(&self, ctx: EngineContext, path: PathBuf) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.import_file(ctx, path).await }))
    }
    fn import_ml_file(&self, ctx: EngineContext, path: PathBuf) -> EngineFuture<'_, ()> {
        self.execute(ctx, move |engine, ctx| Box::pin(async move { engine.import_ml_file(ctx, path).await }))
    }
}
