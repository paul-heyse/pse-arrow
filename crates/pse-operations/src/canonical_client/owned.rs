// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite native gRPC lifecycle. The owned HTTP/2 driver is the socket-drain
//! boundary; dropping a tonic request alone is not evidence of physical drain.

use super::EngineResult;
use super::outbound::{FlushFence, OutboundBody, OwnedIo};
use super::request::{RequestContext, ResponseLimits};
use bytes::{Bytes, BytesMut};
use http_body::{Body, Frame};
#[cfg(test)]
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use opentelemetry::{
    propagation::TextMapPropagator,
    trace::{FutureExt, TraceContextExt},
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};
use surrealdb_types::Error;
use tokio::sync::Notify;
use tower_service::Service;
use tracing::{Instrument, instrument::WithSubscriber};
use tracing_opentelemetry::OpenTelemetrySpanExt;

tokio::task_local! { pub(super) static REQUEST: RequestContext; }

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type HttpFuture =
    Pin<Box<dyn Future<Output = Result<http::Response<tonic::body::Body>, BoxError>> + Send>>;

/// Exact name of the canonical client's exported request span.
pub const REQUEST_SPAN_NAME: &str = "pse.canonical.request";
/// Instrumentation target that owns the canonical request span.
pub const REQUEST_SPAN_TARGET: &str = module_path!();

/// Admit only this owner's request spans, never events or similarly named spans.
pub fn is_request_span(metadata: &tracing::Metadata<'_>) -> bool {
    metadata.is_span()
        && metadata.target() == REQUEST_SPAN_TARGET
        && metadata.name() == REQUEST_SPAN_NAME
}

/// Observation follows the actual request owner, independently of REQUEST's
/// deadline, dispatch, prepaid payload and physical-drain authorities.
#[derive(Clone)]
struct RequestTrace {
    span: tracing::Span,
    context: opentelemetry::Context,
    dispatch: tracing::Dispatch,
}
impl RequestTrace {
    fn new(operation: &'static str) -> Self {
        let parent = tracing::Span::current().context();
        let parent = if parent.span().span_context().is_valid() {
            parent
        } else {
            opentelemetry::Context::current()
        };
        let span = tracing::info_span!(target: REQUEST_SPAN_TARGET, parent: None, REQUEST_SPAN_NAME, operation,
            otel.kind = "client", telemetry.parent = tracing::field::Empty,
            rpc.method = tracing::field::Empty, response = tracing::field::Empty);
        let valid = parent.span().span_context().is_valid();
        match span.set_parent(parent.clone()) {
            Ok(()) => {
                span.record(
                    "telemetry.parent",
                    if valid { "attached" } else { "missing" },
                );
            }
            Err(_) => {
                span.record("telemetry.parent", "unavailable");
            }
        }
        let context = span.context();
        // API-only callers can propagate even when no tracing layer is installed.
        let context = if context.span().span_context().is_valid() {
            context
        } else {
            parent
        };
        Self {
            span,
            context,
            dispatch: tracing::dispatcher::get_default(Clone::clone),
        }
    }
    fn future<F: Future>(self, future: F) -> impl Future<Output = F::Output> {
        future
            .with_context(self.context)
            .instrument(self.span)
            .with_subscriber(self.dispatch)
    }
    fn inject(&self, headers: &mut http::HeaderMap) {
        // Only the standard W3C carrier is generated from authentic API context.
        // Erase stale carrier fields even when this call has no valid parent.
        headers.remove("traceparent");
        headers.remove("tracestate");
        let mut fields = std::collections::HashMap::new();
        TraceContextPropagator::new().inject_context(&self.context, &mut fields);
        for (name, value) in fields {
            if value.is_empty() {
                continue;
            }
            if let (Ok(name), Ok(value)) = (
                http::header::HeaderName::try_from(name),
                http::HeaderValue::try_from(value),
            ) {
                headers.insert(name, value);
            }
        }
    }
}

/// Keep query observation alive through protobuf and scientific-value decode,
/// rather than retiring it when the transport returns response headers.
pub(super) fn observe<F: Future>(
    operation: &'static str,
    future: F,
) -> impl Future<Output = F::Output> {
    let trace = RequestTrace::new(operation);
    trace.future(future)
}

/// Tonic owns this wrapper until the response stream completes or is dropped.
/// Each synchronous poll/drop installs only this request's context; no attached
/// thread-local guard crosses an await or leaks into an HTTP/2 sibling.
struct ObservedBody<B> {
    inner: Option<Pin<Box<B>>>,
    trace: RequestTrace,
    complete: bool,
}
impl<B: Body> ObservedBody<B> {
    fn new(inner: B, trace: RequestTrace) -> Self {
        let complete = inner.is_end_stream();
        if complete {
            trace.span.record("response", "complete");
        }
        Self {
            inner: Some(Box::pin(inner)),
            trace,
            complete,
        }
    }
}
impl<B: Body<Data = Bytes>> Body for ObservedBody<B> {
    type Data = Bytes;
    type Error = B::Error;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.get_mut();
        let trace = this.trace.clone();
        let _dispatch = tracing::dispatcher::set_default(&trace.dispatch);
        let _span = trace.span.enter();
        let _context = trace.context.attach();
        let Some(body) = this.inner.as_mut() else {
            return Poll::Ready(None);
        };
        let result = body.as_mut().poll_frame(cx);
        match &result {
            _ if this.complete => {}
            Poll::Ready(None) => {
                this.complete = true;
                trace.span.record("response", "complete");
            }
            Poll::Ready(Some(Err(_))) => {
                this.complete = true;
                trace.span.record("response", "error");
            }
            Poll::Ready(Some(Ok(frame))) if frame.is_trailers() => {
                this.complete = true;
                trace.span.record("response", "complete");
            }
            _ => {}
        }
        result
    }
    fn is_end_stream(&self) -> bool {
        self.inner.as_ref().is_none_or(|body| body.is_end_stream())
    }
    fn size_hint(&self) -> http_body::SizeHint {
        self.inner
            .as_ref()
            .map_or_else(http_body::SizeHint::default, |body| body.size_hint())
    }
}
impl<B> Drop for ObservedBody<B> {
    fn drop(&mut self) {
        let _dispatch = tracing::dispatcher::set_default(&self.trace.dispatch);
        let _span = self.trace.span.enter();
        let _context = self.trace.context.clone().attach();
        if !self.complete {
            self.trace.span.record("response", "dropped");
        }
        drop(self.inner.take());
    }
}

struct Bootstrap {
    _lifetime: Option<super::request::RequestLifetimeRetention>,
    _payload: Option<super::request::PayloadRetention>,
}
impl std::fmt::Debug for Bootstrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GrpcBootstrapOwnership")
    }
}

/// Hyper's executor owns ConnTask (the actual I/O), Pipe and Send futures in
/// addition to the returned dispatcher. Admission and sealing share one lock.
#[derive(Debug)]
struct NativeTasks {
    tasks: Mutex<Option<tokio::task::JoinSet<()>>>,
    runtime: tokio::runtime::Handle,
    closed: AtomicBool,
    stopping: Notify,
    drained: AtomicBool,
    joined: Notify,
    bootstrap: Mutex<Option<Bootstrap>>,
    io: Arc<FlushFence>,
}
impl NativeTasks {
    fn spawn<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let rejected = {
            let mut admission = self
                .tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match admission.as_mut() {
                Some(tasks) => {
                    // Completed records do not accumulate across a long-lived connection.
                    while tasks.try_join_next().is_some() {}
                    tasks.spawn_on(
                        async move {
                            let _ = future.await;
                        },
                        &self.runtime,
                    );
                    None
                }
                None => Some(future),
            }
        };
        // Rejected child destructors run outside the admission lock. Its parent
        // remains in the sealed set until this synchronous execute call returns.
        drop(rejected);
    }
    fn stop(self: &Arc<Self>) {
        let tasks = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        self.closed.store(true, Ordering::Release);
        self.stopping.notify_waiters();
        if let Some(mut tasks) = tasks {
            tasks.abort_all();
            let owner = self.clone();
            // The joining owner is outside its own set and survives every waiter.
            self.runtime.spawn(async move {
                while tasks.join_next().await.is_some() {}
                owner.io.joined();
                owner
                    .bootstrap
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                owner.drained.store(true, Ordering::Release);
                owner.joined.notify_waiters();
            });
        }
    }
    async fn wait(&self) {
        loop {
            let wake = self.joined.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.drained.load(Ordering::Acquire) {
                return;
            }
            wake.await;
        }
    }
}
#[derive(Clone, Debug)]
struct OwnedExecutor(Arc<NativeTasks>);
impl<F> hyper::rt::Executor<F> for OwnedExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, future: F) {
        self.0.spawn(future);
    }
}
struct DispatcherCompletion(std::sync::Weak<NativeTasks>);
impl Drop for DispatcherCompletion {
    fn drop(&mut self) {
        if let Some(tasks) = self.0.upgrade() {
            tasks.stop();
        }
    }
}
#[derive(Debug)]
pub(super) struct Driver {
    tasks: Arc<NativeTasks>,
    closing: AtomicBool,
}
impl Driver {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            closing: AtomicBool::new(false),
            tasks: Arc::new(NativeTasks {
                tasks: Mutex::new(Some(tokio::task::JoinSet::new())),
                runtime: tokio::runtime::Handle::current(),
                closed: AtomicBool::new(false),
                stopping: Notify::new(),
                drained: AtomicBool::new(false),
                joined: Notify::new(),
                io: Arc::new(FlushFence::default()),
                bootstrap: Mutex::new(
                    REQUEST
                        .try_with(|context| Bootstrap {
                            _payload: context.payload_owner().map(|owner| owner.retain_driver()),
                            _lifetime: context.lifetime_owner().map(|owner| owner.retain()),
                        })
                        .ok(),
                ),
            }),
        })
    }
    pub(super) fn finish_setup(&self) {
        self.tasks
            .bootstrap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
    pub(super) fn begin_close(&self) -> bool {
        !self.closing.swap(true, Ordering::AcqRel)
    }
    pub(super) fn accepting(&self) -> bool {
        !self.closed() && !self.closing.load(Ordering::Acquire)
    }
    pub(super) fn closed(&self) -> bool {
        self.tasks.closed.load(Ordering::Acquire)
    }
    #[cfg(test)]
    fn drained(&self) -> bool {
        self.tasks.drained.load(Ordering::Acquire)
    }
    pub(super) async fn stopping(&self) {
        loop {
            let wake = self.tasks.stopping.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.closed() {
                return;
            }
            wake.await;
        }
    }
    pub(super) async fn shutdown(&self) {
        self.tasks.stop();
        self.tasks.wait().await;
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        self.tasks.stop();
    }
}

/// The canonical native client always uses one owned, drainable HTTP/2 connection.
#[derive(Clone)]
pub(super) enum OwnedHttp2 {
    Owned {
        sender: hyper::client::conn::http2::SendRequest<OutboundBody>,
        driver: Arc<Driver>,
        authority: http::uri::Authority,
        limit: usize,
    },
}
impl OwnedHttp2 {
    pub(super) async fn connect(
        url: &url::Url,
        limit: usize,
        driver: Arc<Driver>,
    ) -> EngineResult<Self> {
        if url.scheme() != "grpc" || !url.username().is_empty() || url.password().is_some() {
            return Err(Error::configuration(
                "Bounded gRPC requires an explicit authenticated loopback grpc endpoint"
                    .to_string(),
                None,
            ));
        }
        let host = url
            .host_str()
            .ok_or_else(|| Error::configuration("Missing gRPC host".to_string(), None))?;
        let port = url
            .port()
            .ok_or_else(|| Error::configuration("Missing gRPC port".to_string(), None))?;
        let address = tokio::net::lookup_host((host, port))
            .await
            .map_err(connection_error)?
            .find(|a| a.ip().is_loopback())
            .ok_or_else(|| {
                Error::configuration("Bounded gRPC requires loopback".to_string(), None)
            })?;
        let socket = tokio::net::TcpStream::connect(address)
            .await
            .map_err(connection_error)?;
        socket.set_nodelay(true).map_err(connection_error)?;
        Self::handshake(
            TokioIo::new(socket),
            address.to_string().parse().map_err(connection_error)?,
            limit,
            driver,
        )
        .await
    }
    async fn handshake<I>(
        io: I,
        authority: http::uri::Authority,
        limit: usize,
        driver: Arc<Driver>,
    ) -> EngineResult<Self>
    where
        I: hyper::rt::Read + hyper::rt::Write + Send + Unpin + 'static,
    {
        let (sender, connection) =
            hyper::client::conn::http2::Builder::new(OwnedExecutor(driver.tasks.clone()))
                .max_frame_size(16 << 10)
                .max_header_list_size(64 << 10)
                .initial_stream_window_size(64 << 10)
                .initial_connection_window_size(64 << 10)
                .max_concurrent_streams(0)
                .handshake(OwnedIo::new(io, driver.tasks.io.clone()))
                .await
                .map_err(connection_error)?;
        let owned = (
            Box::pin(connection),
            DispatcherCompletion(Arc::downgrade(&driver.tasks)),
        );
        driver.tasks.spawn(async move {
            let (connection, _completion) = owned;
            let _ = connection.await;
        });
        Ok(Self::Owned {
            sender,
            driver,
            authority,
            limit,
        })
    }
}
fn connection_error(e: impl std::fmt::Display) -> Error {
    Error::connection(e.to_string(), None)
}
fn refused(reason: &'static str) -> BoxError {
    std::io::Error::other(reason).into()
}

impl Service<http::Request<tonic::body::Body>> for OwnedHttp2 {
    type Response = http::Response<tonic::body::Body>;
    type Error = BoxError;
    type Future = HttpFuture;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self {
            Self::Owned { sender, driver, .. } => {
                if driver.closed() {
                    return Poll::Ready(Err(refused("gRPC connection closed")));
                }
                sender.poll_ready(cx).map_err(BoxError::from)
            }
        }
    }
    fn call(&mut self, mut request: http::Request<tonic::body::Body>) -> Self::Future {
        let trace = RequestTrace::new("grpc");
        trace.span.record("rpc.method", request.uri().path());
        trace.inject(request.headers_mut());
        match self {
            Self::Owned {
                sender,
                driver,
                authority,
                limit,
            } => {
                let context = REQUEST.try_with(Clone::clone).ok();
                let response_type = super::wire_schema::response(request.uri().path());
                let driver = driver.clone();
                let limit = *limit;
                let mut parts = request.uri().clone().into_parts();
                parts.scheme = Some(http::uri::Scheme::HTTP);
                parts.authority = Some(authority.clone());
                let uri = match http::Uri::from_parts(parts) {
                    Ok(uri) => uri,
                    Err(error) => return Box::pin(async move { Err(BoxError::from(error)) }),
                };
                *request.uri_mut() = uri;
                let (parts, body) = request.into_parts();
                let request = http::Request::from_parts(
                    parts,
                    OutboundBody::new(body, context.as_ref(), driver.tasks.io.clone()),
                );
                if let Some(context) = &context {
                    context.mark_dispatched();
                }
                let future = sender.send_request(request);
                let observation = trace.clone();
                Box::pin(trace.future(async move {
                    if driver.closed() {
                        return Err(refused("gRPC connection closed"));
                    }
                    let response = future.await.map_err(|e| {
                        observation.span.record("response", "error");
                        BoxError::from(e)
                    })?;
                    let (parts, body) = response.into_parts();
                    // Bounded profiles never advertise compressed responses: validation
                    // runs on the exact protobuf bytes before prost can allocate.
                    if parts
                        .headers
                        .get("grpc-encoding")
                        .is_some_and(|h| h != "identity")
                    {
                        observation.span.record("response", "error");
                        driver.shutdown().await;
                        return Err(refused("compressed bounded gRPC response refused"));
                    }
                    let body = WireBody::new(body, context, response_type, limit);
                    Ok(http::Response::from_parts(
                        parts,
                        tonic::body::Body::new(ObservedBody::new(body, observation)),
                    ))
                }))
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
    _payload: Option<super::request::PayloadRetention>,
    _lifetime: Option<super::request::RequestLifetimeRetention>,
    schema: Option<&'static str>,
    limit: usize,
    counts: super::wire::Counts,
    done: bool,
}
impl WireBody {
    fn new(
        body: hyper::body::Incoming,
        context: Option<RequestContext>,
        schema: Option<&'static str>,
        limit: usize,
    ) -> Self {
        Self {
            inner: Box::pin(body),
            pending: BytesMut::new(),
            _payload: context
                .as_ref()
                .and_then(RequestContext::payload_owner)
                .map(|o| o.retain_driver()),
            _lifetime: context
                .as_ref()
                .and_then(RequestContext::lifetime_owner)
                .map(|o| o.retain()),
            context,
            schema,
            limit,
            counts: Default::default(),
            done: false,
        }
    }
}
impl Body for WireBody {
    type Data = Bytes;
    type Error = BoxError;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        if self.done {
            return Poll::Ready(None);
        }
        loop {
            if self.pending.len() >= 5 {
                if self.pending[0] != 0 {
                    self.done = true;
                    return Poll::Ready(Some(Err(refused("compressed gRPC message refused"))));
                }
                let len = u32::from_be_bytes([
                    self.pending[1],
                    self.pending[2],
                    self.pending[3],
                    self.pending[4],
                ]) as usize;
                let limits = self
                    .context
                    .as_ref()
                    .and_then(RequestContext::response_limits)
                    .unwrap_or_else(|| default_limits(self.limit));
                if len > self.limit || len > limits.wire_bytes {
                    self.done = true;
                    return Poll::Ready(Some(Err(refused("gRPC frame exceeds response envelope"))));
                }
                if self.pending.len() >= len + 5 {
                    let frame = self.pending.split_to(len + 5).freeze();
                    let schema = self.schema;
                    if let Err(reason) = self.counts.check(&frame[5..], schema, limits) {
                        self.done = true;
                        return Poll::Ready(Some(Err(refused(reason))));
                    }
                    return Poll::Ready(Some(Ok(Frame::data(frame))));
                }
            }
            match self.inner.as_mut().poll_frame(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(Ok(frame))) => match frame.into_data() {
                    Ok(data) => {
                        if self.pending.len().saturating_add(data.len())
                            > self.limit.saturating_add(5).saturating_add(64 << 10)
                        {
                            self.done = true;
                            return Poll::Ready(Some(Err(refused(
                                "gRPC encoded buffer exceeds envelope",
                            ))));
                        }
                        self.pending.extend_from_slice(&data);
                    }
                    Err(frame) => {
                        self.done = true;
                        if !self.pending.is_empty() {
                            return Poll::Ready(Some(Err(refused("truncated gRPC message"))));
                        }
                        return Poll::Ready(Some(Ok(frame)));
                    }
                },
                Poll::Ready(Some(Err(error))) => {
                    self.done = true;
                    return Poll::Ready(Some(Err(Box::new(error))));
                }
                Poll::Ready(None) => {
                    self.done = true;
                    return if self.pending.is_empty() {
                        Poll::Ready(None)
                    } else {
                        Poll::Ready(Some(Err(refused("truncated gRPC message"))))
                    };
                }
            }
        }
    }
}
fn default_limits(wire_bytes: usize) -> ResponseLimits {
    ResponseLimits {
        wire_bytes,
        nodes: 1 << 20,
        depth: 64,
        container_items: 1 << 20,
        string_bytes: wire_bytes,
        string_length: wire_bytes,
        blob_bytes: wire_bytes,
        blob_length: wire_bytes,
    }
}

#[cfg(test)]
#[path = "owned_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "owned_physical_tests.rs"]
mod physical_tests;
