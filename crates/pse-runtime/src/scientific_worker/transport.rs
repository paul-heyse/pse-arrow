// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticated local HTTP/2, independent I/O and a single mutable native lane.
use super::{
    PreparedSession, WorkerError,
    generated::{pb::pse::scientific_worker::v1 as pb, services},
    handler, infrastructure,
    process::ParentGuard,
    protocol as p,
};
use protovalidate_buffa::Validate as _;
use pse_ids::SemanticId;
use pse_kernels::{ExecutionScope, ProviderError};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub(super) const FRAME_LIMIT: usize = 1 << 20;
pub(super) const DECODE_LIMIT: usize = 4 << 20;
pub(super) const NATIVE_STACK: usize = 8 << 20;

#[derive(Clone)]
struct Connector {
    endpoint: PathBuf,
    pid: u32,
    connected: Arc<AtomicBool>,
}
impl tower_service::Service<http::Uri> for Connector {
    type Response = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Error = std::io::Error;
    type Future = std::pin::Pin<Box<dyn Future<Output = std::io::Result<Self::Response>> + Send>>;
    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: http::Uri) -> Self::Future {
        let connector = self.clone();
        Box::pin(async move {
            // Reconnect cannot renew a scientific session. The native client's
            // recovery machinery is deliberately refused at this connector.
            if connector.connected.swap(true, Ordering::AcqRel) {
                return Err(std::io::Error::other(
                    "scientific worker reconnect is unavailable",
                ));
            }
            let stream = tokio::net::UnixStream::connect(&connector.endpoint).await?;
            let peer = stream.peer_cred()?;
            if peer.uid() != rustix::process::geteuid().as_raw()
                || peer.pid().and_then(|pid| u32::try_from(pid).ok()) != Some(connector.pid)
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "scientific worker kernel peer differs from owned child",
                ));
            }
            Ok(hyper_util::rt::TokioIo::new(stream))
        })
    }
}

type Reply = std::sync::mpsc::Sender<Result<p::Response, WorkerError>>;
struct Outgoing {
    request: p::Request,
    timeout: Option<Duration>,
    reply: Reply,
}
fn rpc_timeout(
    scope: &ExecutionScope,
    evaluating: bool,
    now: std::time::Instant,
) -> Result<Option<Duration>, WorkerError> {
    scope.check()?;
    if !evaluating {
        return Ok(Some(scope.remaining(Duration::from_secs(30))?));
    }
    scope
        .deadline()
        .map(|deadline| {
            deadline
                .checked_duration_since(now)
                .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| WorkerError::from(ProviderError::Deadline))
        })
        .transpose()
}
fn call_options(timeout: Option<Duration>) -> connectrpc::client::CallOptions {
    let options = connectrpc::client::CallOptions::default()
        .with_max_message_size(FRAME_LIMIT)
        .with_element_memory_limit(DECODE_LIMIT);
    match timeout {
        Some(timeout) => options.with_timeout(timeout),
        None => options,
    }
}
#[cfg(test)]
type CleanupGate =
    Arc<std::sync::Mutex<Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>>>;
/// An independent async driver; the native caller never drives Tokio itself.
pub(super) struct IoOwner {
    sender: Option<tokio::sync::mpsc::Sender<Outgoing>>,
    thread: Option<std::thread::JoinHandle<()>>,
    stop: pse_diagnostics::Cancellation,
    #[cfg(test)]
    cleanup_gate: CleanupGate,
}
impl std::fmt::Debug for IoOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScientificIoOwner").finish_non_exhaustive()
    }
}
impl IoOwner {
    pub(super) fn connect(
        endpoint: PathBuf,
        pid: u32,
        scope: &ExecutionScope,
    ) -> Result<Self, WorkerError> {
        let timeout = scope.remaining(Duration::from_secs(30))?;
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<Outgoing>(1);
        let (ready, startup) = std::sync::mpsc::channel();
        let stop = pse_diagnostics::Cancellation::default();
        let stopping = stop.clone();
        #[cfg(test)]
        let cleanup_gate = CleanupGate::default();
        #[cfg(test)]
        let thread_cleanup_gate = cleanup_gate.clone();
        let thread = std::thread::Builder::new().name("pse-science-io".into()).stack_size(2 << 20).spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(value) => value, Err(error) => { let _ = ready.send(Err(infrastructure(error))); return; }
            };
            runtime.block_on(async move {
                let config = match "http://localhost".parse::<http::Uri>() {
                    Ok(uri) => connectrpc::client::ClientConfig::new(uri).with_protocol(connectrpc::Protocol::Grpc),
                    Err(error) => { let _ = ready.send(Err(infrastructure(error))); return; }
                };
                let connector = Connector { endpoint, pid, connected: Arc::new(AtomicBool::new(false)) };
                let connection = tokio::select! {
                    result = tokio::time::timeout(timeout, connectrpc::client::Http2Connection::connect_with_connector(connector, config.base_uri().clone())) => result,
                    _ = stopping.cancelled() => return,
                };
                let connection = match connection {
                    Ok(Ok(value)) => value.shared(1),
                    Ok(Err(error)) => { let _ = ready.send(Err(infrastructure(error))); return; }
                    Err(error) => { let _ = ready.send(Err(infrastructure(error))); return; }
                };
                let client = services::ScientificWorkerClient::new(connection, config);
                if ready.send(Ok(())).is_err() { return; }
                loop {
                    let outgoing = tokio::select! { result = receiver.recv() => match result { Some(value) => value, None => break }, _ = stopping.cancelled() => break };
                    let options = call_options(outgoing.timeout);
                    let response = tokio::select! {
                        result = client.exchange_with_options(outgoing.request.into(), options) => result,
                        _ = stopping.cancelled() => break,
                    };
                    let result = response
                        .map_err(|error| WorkerError::UnknownCompletion(error.to_string()))
                        .and_then(|response| {
                            let response = response.into_owned();
                            response.validate().map_err(|error| WorkerError::UnknownCompletion(format!("scientific worker response constraint: {error}")))?;
                            p::Response::try_from(response)
                        });
                    let failed = result.is_err();
                    let _ = outgoing.reply.send(result);
                    if failed { break; }
                }
            });
            #[cfg(test)]
            if let Some((entered, release)) = thread_cleanup_gate.lock().unwrap().take() {
                let _ = entered.send(());
                let _ = release.recv();
            }
        }).map_err(infrastructure)?;
        let mut owner = Self {
            sender: Some(sender),
            thread: Some(thread),
            stop,
            #[cfg(test)]
            cleanup_gate,
        };
        loop {
            match startup.recv_timeout(Duration::from_millis(5)) {
                Ok(result) => {
                    result?;
                    scope.check()?;
                    return Ok(owner);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(infrastructure("scientific worker I/O startup departed"));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Err(error) = scope.check() {
                        owner.sender.take();
                        return Err(error.into());
                    }
                }
            }
        }
    }
    #[cfg(test)]
    pub(super) fn delay_cleanup(
        &mut self,
    ) -> (std::sync::mpsc::Receiver<()>, std::sync::mpsc::Sender<()>) {
        let (entered, observe) = std::sync::mpsc::channel();
        let (release, wait) = std::sync::mpsc::channel();
        *self.cleanup_gate.lock().unwrap() = Some((entered, wait));
        (observe, release)
    }
    pub(super) fn exchange(
        &self,
        request: p::Request,
        scope: &ExecutionScope,
        invocation_cancel: Option<&AtomicBool>,
    ) -> Result<p::Response, WorkerError> {
        let checkpoint = || -> Result<(), WorkerError> {
            scope.check()?;
            if invocation_cancel.is_some_and(|cancel| cancel.load(Ordering::Acquire)) {
                return Err(ProviderError::Cancelled.into());
            }
            Ok(())
        };
        checkpoint()?;
        let timeout = rpc_timeout(
            scope,
            matches!(&request, p::Request::Evaluate(_)),
            std::time::Instant::now(),
        )?;
        let (reply, received) = std::sync::mpsc::channel();
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| infrastructure("scientific worker I/O is closed"))?;
        sender
            .try_send(Outgoing {
                request,
                timeout,
                reply,
            })
            .map_err(infrastructure)?;
        loop {
            match received.recv_timeout(Duration::from_millis(5)) {
                Ok(value) => {
                    checkpoint()?;
                    return value;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(WorkerError::UnknownCompletion(
                        "scientific worker I/O departed after dispatch".into(),
                    ));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    checkpoint()?;
                }
            }
        }
    }
}
impl Drop for IoOwner {
    fn drop(&mut self) {
        self.sender.take();
        self.stop.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Job {
    request: p::Request,
    response: tokio::sync::oneshot::Sender<p::Response>,
}
struct Server {
    jobs: tokio::sync::mpsc::Sender<Job>,
}
impl services::ScientificWorker for Server {
    async fn exchange<'a>(
        &'a self,
        _: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::Request>,
    ) -> connectrpc::ServiceResult<impl connectrpc::Encodable<pb::Response> + Send + use<'a>> {
        if let Err(error) = request.validate() {
            return Ok(connectrpc::Response::new(pb::Response::from(
                p::Response::Failure(p::Failure::Contract(error.to_string())),
            )));
        }
        let request = match p::Request::try_from(request.to_owned_message()) {
            Ok(value) => value,
            Err(error) => {
                return Ok(connectrpc::Response::new(pb::Response::from(
                    p::Response::Failure(handler::failure(error)),
                )));
            }
        };
        let (response, received) = tokio::sync::oneshot::channel();
        self.jobs
            .send(Job { request, response })
            .await
            .map_err(|_| {
                connectrpc::ConnectError::unavailable("scientific worker native lane departed")
            })?;
        let response = received.await.map_err(|_| {
            connectrpc::ConnectError::unavailable("scientific worker native completion unavailable")
        })?;
        Ok(connectrpc::Response::new(pb::Response::from(response)))
    }
}

/// Private worker composition root; verifies actual parent and accepted kernel peer.
/// The executable owns this process and uses parent-loss termination to contain FFI.
pub fn serve(endpoint: PathBuf, session: SemanticId, parent: u32) -> Result<(), WorkerError> {
    serve_with_preparation(endpoint, parent, move |request, cancellation| {
        handler::prepare(request, session, cancellation)
    })
}

pub(super) fn serve_with_preparation(
    endpoint: PathBuf,
    parent: u32,
    prepare: impl Fn(p::Prepare, pse_diagnostics::Cancellation) -> Result<PreparedSession, WorkerError>
    + Send
    + 'static,
) -> Result<(), WorkerError> {
    use services::ScientificWorkerExt as _;
    use tower_service::Service as _;
    let _parent = ParentGuard::start(parent)?;
    let listener = std::os::unix::net::UnixListener::bind(&endpoint).map_err(infrastructure)?;
    std::fs::set_permissions(
        &endpoint,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o600),
    )
    .map_err(infrastructure)?;
    listener.set_nonblocking(true).map_err(infrastructure)?;
    let cancellation = pse_diagnostics::Cancellation::default();
    let cancel = cancellation.clone();
    let closed = Arc::new(AtomicBool::new(false));
    let drained = closed.clone();
    let (jobs, mut receiving) = tokio::sync::mpsc::channel::<Job>(1);
    let native = std::thread::Builder::new()
        .name("pse-science-native".into())
        .stack_size(NATIVE_STACK)
        .spawn(move || {
            let mut sessions = BTreeMap::<SemanticId, PreparedSession>::new();
            while let Some(job) = receiving.blocking_recv() {
                let stopping = matches!(&job.request, p::Request::Stop(_));
                let result: Result<p::Response, WorkerError> = (|| match job.request {
                    p::Request::Prepare(value) => {
                        if drained.load(Ordering::Acquire)
                            || sessions.len() >= 64
                            || sessions.contains_key(&value.handle.generation)
                        {
                            return Err(ProviderError::Contract(
                                "scientific worker duplicate/excess preparation".into(),
                            )
                            .into());
                        }
                        let prepared = prepare(value, cancel.clone())?;
                        let handle = p::Handle::from(prepared.handle());
                        sessions.insert(handle.generation, prepared);
                        Ok(p::Response::Prepared(handle))
                    }
                    p::Request::Evaluate(value) => {
                        let owner =
                            sessions.get_mut(&value.handle.generation).ok_or_else(|| {
                                ProviderError::Contract(
                                    "scientific worker unknown generation".into(),
                                )
                            })?;
                        Ok(p::Response::Values(handler::evaluate(owner, value)?))
                    }
                    p::Request::Stop(handle) | p::Request::Close(handle) => {
                        let actual = sessions.get(&handle.generation).ok_or_else(|| {
                            ProviderError::Contract(
                                "scientific worker unknown closing generation".into(),
                            )
                        })?;
                        if actual.handle() != handle.clone().into() {
                            return Err(ProviderError::Contract(
                                "scientific worker foreign closing context".into(),
                            )
                            .into());
                        }
                        if stopping {
                            cancel.cancel();
                        }
                        for owner in sessions.values_mut() {
                            owner.close();
                        }
                        sessions.clear();
                        drained.store(true, Ordering::Release);
                        Ok(p::Response::Closed(handle))
                    }
                })();
                let terminal = result.as_ref().err().is_some_and(|error| match error {
                    WorkerError::Provider(error) => !error.recoverable(),
                    _ => true,
                });
                if terminal {
                    cancel.cancel();
                    for owner in sessions.values_mut() {
                        owner.close();
                    }
                    sessions.clear();
                    drained.store(true, Ordering::Release);
                }
                let response =
                    result.unwrap_or_else(|error| p::Response::Failure(handler::failure(error)));
                #[cfg(test)]
                let response = perturb_response_for_test(response);
                if job.response.send(response).is_err() {
                    break;
                }
            }
        })
        .map_err(infrastructure)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(infrastructure)?;
    let result = runtime.block_on(async {
        let listener = tokio::net::UnixListener::from_std(listener).map_err(infrastructure)?;
        let (stream, _) = listener.accept().await.map_err(infrastructure)?;
        let peer = stream.peer_cred().map_err(infrastructure)?;
        if peer.uid() != rustix::process::geteuid().as_raw()
            || peer.pid().and_then(|pid| u32::try_from(pid).ok()) != Some(parent)
        {
            return Err(infrastructure(
                "scientific worker kernel parent peer differs",
            ));
        }
        let router = Arc::new(Server { jobs: jobs.clone() }).register(connectrpc::Router::new());
        let service = connectrpc::ConnectRpcService::new(router).with_limits(
            connectrpc::Limits::default()
                .with_max_request_body_size(FRAME_LIMIT)
                .with_max_message_size(FRAME_LIMIT)
                .with_element_memory_limit(DECODE_LIMIT),
        );
        let service = hyper::service::service_fn(move |request| {
            let mut service = service.clone();
            service.call(request)
        });
        let mut builder =
            hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new());
        builder.max_concurrent_streams(1);
        builder
            .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
            .await
            .map_err(infrastructure)
    });
    cancellation.cancel();
    if !closed.load(Ordering::Acquire) {
        // A disconnected active RPC must settle its actual native call, even if
        // cooperative cancellation cannot interrupt the foreign instruction.
        std::process::exit(125);
    }
    drop(jobs);
    native
        .join()
        .map_err(|_| infrastructure("scientific worker native lane panicked"))?;
    result
}

#[cfg(test)]
fn perturb_response_for_test(mut response: p::Response) -> p::Response {
    // Only the explicitly launched library test child includes these faults.
    // The deployed executable has neither the branch nor its environment input.
    if let p::Response::Values(values) = &mut response {
        match std::env::var("PSE_SCIENCE_TEST_MODE").as_deref() {
            Ok("wrong-shape") => values.values.clear(),
            Ok("wrong-context") => values.handle.generation = SemanticId::NIL,
            Ok("nonfinite") => values.values.fill(f64::NAN),
            Ok("oversized") => values
                .values
                .resize(FRAME_LIMIT / size_of::<f64>() + 128, 1.0),
            _ => {}
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_service::Service as _;

    #[test]
    fn evaluation_options_preserve_original_optional_deadline_without_renewal() {
        let control = pse_diagnostics::Cancellation::default();
        let now = std::time::Instant::now();
        let scope = ExecutionScope::new(control.clone(), None);
        assert_eq!(
            call_options(rpc_timeout(&scope, true, now).unwrap()).timeout(),
            None
        );
        assert_eq!(
            call_options(rpc_timeout(&scope, false, now).unwrap()).timeout(),
            Some(Duration::from_secs(30))
        );

        let original = ExecutionScope::new(control.clone(), Some(now + Duration::from_secs(1800)));
        assert_eq!(
            call_options(rpc_timeout(&original, true, now).unwrap()).timeout(),
            Some(Duration::from_secs(1800))
        );
        assert_eq!(
            call_options(rpc_timeout(&original, true, now + Duration::from_secs(601)).unwrap())
                .timeout(),
            Some(Duration::from_secs(1199))
        );
        assert!(matches!(
            rpc_timeout(&original, true, now + Duration::from_secs(1801)),
            Err(WorkerError::Provider(ProviderError::Deadline))
        ));
        assert!(!control.flag().load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn connector_refuses_wrong_kernel_peer_and_any_reconnect() {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = directory.path().join("peer.sock");
        let _listener = tokio::net::UnixListener::bind(&endpoint).unwrap();
        let mut wrong = Connector {
            endpoint: endpoint.clone(),
            pid: u32::MAX,
            connected: Arc::new(AtomicBool::new(false)),
        };
        assert_eq!(
            wrong
                .call("http://localhost".parse().unwrap())
                .await
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        let mut actual = Connector {
            endpoint,
            pid: std::process::id(),
            connected: Arc::new(AtomicBool::new(false)),
        };
        let stream = actual
            .call("http://localhost".parse().unwrap())
            .await
            .unwrap();
        drop(stream);
        assert!(
            actual
                .call("http://localhost".parse().unwrap())
                .await
                .is_err()
        );
    }
}
