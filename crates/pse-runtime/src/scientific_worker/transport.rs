// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticated local HTTP/2, independent I/O and a single mutable native lane.
use super::{
    PreparedSession, WorkerError,
    generated::{pb::pse::scientific_worker::v1 as pb, services},
    handler, infrastructure,
    process::ParentGuard,
};
use pse_ids::SemanticId;
use pse_kernels::{ExecutionScope, ProviderError, worker_protocol as p};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tower_service::Service as _;

pub(super) const FRAME_LIMIT: usize = 1 << 20;
pub(super) const DECODE_LIMIT: usize = 4 << 20;
pub(super) const NATIVE_STACK: usize = 8 << 20;

async fn binary_grpc<D, B>(
    mut service: connectrpc::ConnectRpcService<D>,
    request: http::Request<B>,
) -> Result<http::Response<connectrpc::ConnectRpcBody>, std::convert::Infallible>
where
    D: connectrpc::Dispatcher,
    B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    // Kernel parent authentication precedes creation of this HTTP service. Refuse
    // unsupported protocols before body polling or library decode construction.
    if request.method() != http::Method::POST
        || request.version() != http::Version::HTTP_2
        || !connectrpc::Protocol::detect(request.headers()).is_some_and(|protocol| {
            protocol.protocol == connectrpc::Protocol::Grpc
                && protocol.codec_format == connectrpc::CodecFormat::Proto
        })
    {
        return Ok(connectrpc::ConnectError::unimplemented(
            "local scientific worker operations require binary gRPC POST over HTTP/2",
        )
        .into_http_response(request.headers()));
    }
    service.call(request).await
}

/// ConnectRPC owns framing/codecs; this adapter owns the one authenticated H2
/// sender, without Tower reconnect/buffer tasks or a detached connection driver.
#[derive(Clone)]
struct OwnedTransport(hyper::client::conn::http2::SendRequest<connectrpc::client::ClientBody>);
async fn authenticated_stream(
    endpoint: &std::path::Path,
    pid: u32,
) -> Result<tokio::net::UnixStream, WorkerError> {
    let stream = tokio::net::UnixStream::connect(endpoint)
        .await
        .map_err(|source| WorkerError::Process {
            action: "connect",
            source,
        })?;
    let peer = stream.peer_cred().map_err(|source| WorkerError::Process {
        action: "peer_cred",
        source,
    })?;
    if peer.uid() != rustix::process::geteuid().as_raw()
        || peer.pid().and_then(|pid| u32::try_from(pid).ok()) != Some(pid)
    {
        return Err(WorkerError::Process {
            action: "authenticate",
            source: std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "scientific worker kernel peer differs from owned child",
            ),
        });
    }
    Ok(stream)
}

fn retain_peer_reader(
    stream: tokio::net::UnixStream,
) -> std::io::Result<(tokio::net::UnixStream, std::os::unix::net::UnixStream)> {
    // Logical H2 completion can precede the peer's final shutdown frames. The
    // client's public driver and executor-owned socket task also finish separately.
    // Retain the same authenticated socket's read lifetime, without a competing
    // reader or another reactor registration while Hyper owns protocol I/O.
    let stream = stream.into_std()?;
    let reader = stream.try_clone()?;
    Ok((tokio::net::UnixStream::from_std(stream)?, reader))
}

async fn finish_peer_reader(reader: std::os::unix::net::UnixStream) -> std::io::Result<()> {
    use tokio::io::AsyncReadExt as _;
    // All Hyper tasks have joined. Explicitly half-close even if their shutdown
    // path only dropped its descriptor; the retained receive side permits the
    // peer's final GOAWAY/PING writes until its actual EOF.
    reader.shutdown(std::net::Shutdown::Write)?;
    let mut reader = tokio::net::UnixStream::from_std(reader)?;
    let mut buffer = [0; 8192];
    while reader.read(&mut buffer).await? != 0 {}
    Ok(())
}
impl connectrpc::client::ClientTransport for OwnedTransport {
    type ResponseBody = hyper::body::Incoming;
    type Error = hyper::Error;
    fn send(
        &self,
        request: http::Request<connectrpc::client::ClientBody>,
    ) -> connectrpc::client::BoxFuture<
        'static,
        Result<http::Response<Self::ResponseBody>, Self::Error>,
    > {
        let mut sender = self.0.clone();
        Box::pin(async move { sender.send_request(request).await })
    }
}

/// Hyper also starts H2 stream/keepalive work through its executor. Keep every
/// task join in the same I/O owner, including tasks spawned by an earlier task.
#[derive(Clone, Default)]
struct OwnedExecutor(Arc<std::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>>);
impl<F> hyper::rt::Executor<F> for OwnedExecutor
where
    F: Future<Output = ()> + Send + 'static,
{
    fn execute(&self, future: F) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(tokio::spawn(future));
    }
}
impl OwnedExecutor {
    async fn finish(&self, failures: &mut Vec<WorkerError>) {
        loop {
            let tasks = std::mem::take(
                &mut *self
                    .0
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
            if tasks.is_empty() {
                break;
            }
            for task in tasks {
                if let Err(error) = task.await {
                    failures.push(WorkerError::IoTask(error));
                }
            }
        }
    }
}
async fn finish_driver(
    driver: tokio::task::JoinHandle<Result<(), hyper::Error>>,
) -> Result<(), WorkerError> {
    match driver.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(WorkerError::Connection(error)),
        Err(error) => Err(WorkerError::IoTask(error)),
    }
}

type Reply = std::sync::mpsc::Sender<Result<p::Response, WorkerError>>;
fn deliver_reply(
    reply: Reply,
    result: Result<p::Response, WorkerError>,
) -> (bool, Option<WorkerError>) {
    let failed = result.is_err();
    let departed = reply.send(result).err().and_then(|error| error.0.err());
    (failed, departed)
}
fn received_outcome(
    value: Result<p::Response, WorkerError>,
    checkpoint: Result<(), WorkerError>,
) -> Result<p::Response, WorkerError> {
    match (value, checkpoint) {
        (value, Ok(())) => value,
        (Ok(_), Err(error)) => Err(error),
        (Err(original), Err(checkpoint)) => Err(WorkerError::Retirement {
            causes: vec![original, checkpoint],
        }),
    }
}
struct Outgoing {
    request: pb::Request,
    shutdown: bool,
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
    if scope.deadline().is_some_and(|deadline| deadline <= now) {
        return Err(ProviderError::Deadline.into());
    }
    // The original scope owns scientific expiry. A derived RPC timer rounds
    // its wire budget and can expire first, replacing that typed outcome with
    // a protocol deadline. Parent checkpoints and the native remaining_ns scope
    // enforce the original allowance without that competing timer.
    Ok(None)
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
#[cfg(test)]
type TestClient = services::ScientificWorkerClient<OwnedTransport>;
#[cfg(test)]
type TestClientObserver = Arc<std::sync::Mutex<std::sync::Weak<TestClient>>>;
/// An independent async driver; the native caller never drives Tokio itself.
pub(super) struct IoOwner {
    sender: Option<tokio::sync::mpsc::Sender<Outgoing>>,
    thread: Option<std::thread::JoinHandle<Result<(), WorkerError>>>,
    stop: pse_diagnostics::Cancellation,
    #[cfg(test)]
    cleanup_gate: CleanupGate,
    #[cfg(test)]
    client_observer: TestClientObserver,
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
        #[cfg(test)]
        let client_observer = TestClientObserver::default();
        #[cfg(test)]
        let thread_client_observer = client_observer.clone();
        let thread = std::thread::Builder::new().name("pse-science-io".into()).stack_size(2 << 20).spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(value) => value, Err(error) => { let _ = ready.send(Err(infrastructure(error))); return Ok(()); }
            };
            let result = runtime.block_on(async move {
                let executor = OwnedExecutor::default();
                let mut failures = Vec::new();
                let mut peer_reader = None;
                let shutdown_acknowledged = AtomicBool::new(false);
                let result = async {
                let config = match "http://localhost".parse::<http::Uri>() {
                    Ok(uri) => connectrpc::client::ClientConfig::new(uri).with_protocol(connectrpc::Protocol::Grpc),
                    Err(error) => { let _ = ready.send(Err(infrastructure(error))); return Ok(()); }
                };
                let connection = tokio::select! {
                    result = tokio::time::timeout(timeout, async {
                        let stream = authenticated_stream(&endpoint, pid).await?;
                        let (stream, reader) = retain_peer_reader(stream).map_err(|source| WorkerError::Process {
                            action: "retain peer socket",
                            source,
                        })?;
                        peer_reader = Some(reader);
                        hyper::client::conn::http2::Builder::new(executor.clone())
                            .handshake::<_, connectrpc::client::ClientBody>(hyper_util::rt::TokioIo::new(stream))
                            .await.map_err(WorkerError::Connection)
                    }) => result,
                    _ = stopping.cancelled() => return Ok(()),
                };
                let (sender, connection) = match connection {
                    Ok(Ok(value)) => value,
                    Ok(Err(error)) => { let _ = ready.send(Err(error)); return Ok(()); }
                    Err(error) => { let _ = ready.send(Err(infrastructure(error))); return Ok(()); }
                };
                let driver = tokio::spawn(connection);
                let client = services::ScientificWorkerClient::new(OwnedTransport(sender), config);
                #[cfg(test)]
                let client = {
                    let client = Arc::new(client);
                    *thread_client_observer.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(&client);
                    client
                };
                let active = ready.send(Ok(())).is_ok();
                use futures_util::{stream::FuturesUnordered, StreamExt as _};
                let mut pending = FuturesUnordered::new();
                if active {
                    loop {
                        tokio::select! {
                            completed = pending.next(), if !pending.is_empty() => {
                                match completed {
                                    Some((failed, departed)) => {
                                        if let Some(error) = departed { failures.push(error); }
                                        if failed { break; }
                                    },
                                    None => break,
                                }
                            },
                            result = receiver.recv(), if pending.len() < 2 => {
                                let Some(outgoing) = result else { break; };
                                let client = &client;
                                let shutdown_acknowledged = &shutdown_acknowledged;
                                pending.push(async move {
                                    let response = client.exchange_with_options(
                                        outgoing.request, call_options(outgoing.timeout),
                                    ).await;
                                    let result = response
                                        .map_err(WorkerError::Exchange)
                                        .and_then(|response| {
                                            let response = response.into_owned();
                                            p::Response::try_from(response)
                                        });
                                    if outgoing.shutdown && matches!(&result, Ok(p::Response::Shutdown)) {
                                        // Publish before waking the synchronous caller: its
                                        // stop may win select before pending.next observes us.
                                        shutdown_acknowledged.store(true, Ordering::Release);
                                    }
                                    deliver_reply(outgoing.reply, result)
                                });
                            },
                            _ = stopping.cancelled() => break,
                        }
                    }
                }
                drop(pending);
                if shutdown_acknowledged.load(Ordering::Acquire) {
                    // The peer owns graceful GOAWAY after actual native Shutdown.
                    // Retain the sender through the public driver; the separate
                    // socket reader also outlives every executor-owned H2 task.
                    let result = finish_driver(driver).await;
                    drop(client);
                    result
                } else {
                    // Startup, cancellation, failure and missing acknowledgements
                    // cannot wait for a peer that never agreed to close.
                    drop(client);
                    finish_driver(driver).await
                }
                }.await;
                if let Err(error) = result { failures.push(error); }
                executor.finish(&mut failures).await;
                if let Some(reader) = peer_reader
                    && shutdown_acknowledged.load(Ordering::Acquire)
                    && let Err(source) = finish_peer_reader(reader).await
                {
                    failures.push(WorkerError::Process { action: "finish peer socket", source });
                }
                super::retirement_result(failures)
            });
            // Runtime teardown precedes OS-thread completion and admission release.
            drop(runtime);
            #[cfg(test)]
            if let Some((entered, release)) = thread_cleanup_gate.lock().unwrap().take() {
                let _ = entered.send(());
                let _ = release.recv();
            }
            result
        }).map_err(infrastructure)?;
        let owner = Self {
            sender: Some(sender),
            thread: Some(thread),
            stop,
            #[cfg(test)]
            cleanup_gate,
            #[cfg(test)]
            client_observer,
        };
        loop {
            match startup.recv_timeout(Duration::from_millis(5)) {
                Ok(result) => {
                    if let Err(error) = result {
                        return Err(owner.fail(error));
                    }
                    if let Err(error) = scope.check() {
                        return Err(owner.fail(error.into()));
                    }
                    return Ok(owner);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(
                        owner.fail(infrastructure("scientific worker I/O startup departed"))
                    );
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Err(error) = scope.check() {
                        return Err(owner.fail(error.into()));
                    }
                }
            }
        }
    }
    /// Test only: abandon one generated unary future on the original authenticated
    /// connection, without changing the production queue's fatal RPC-error policy.
    /// The caller must drop this future before normal finish; the stored weak
    /// observer itself cannot extend the client's or connection's lifetime.
    #[cfg(test)]
    pub(super) fn departing_exchange(
        &self,
        request: p::Request,
    ) -> Result<impl Future<Output = Result<p::Response, WorkerError>> + Send + 'static, WorkerError>
    {
        let request = pb::Request::try_from(request)?;
        let client = self
            .client_observer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade()
            .ok_or_else(|| infrastructure("scientific worker test client already departed"))?;
        Ok(async move {
            client
                .exchange_with_options(request, call_options(None))
                .await
                .map_err(WorkerError::Exchange)
                .and_then(|response| p::Response::try_from(response.into_owned()))
        })
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
            matches!(
                &request,
                p::Request::Evaluate(_) | p::Request::EvaluateBatch(_)
            ),
            std::time::Instant::now(),
        )?;
        let shutdown = matches!(&request, p::Request::Shutdown);
        let request = pb::Request::try_from(request)?;
        let (reply, received) = std::sync::mpsc::channel();
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| infrastructure("scientific worker I/O is closed"))?;
        sender
            .try_send(Outgoing {
                request,
                shutdown,
                timeout,
                reply,
            })
            .map_err(infrastructure)?;
        loop {
            match received.recv_timeout(Duration::from_millis(5)) {
                Ok(value) => {
                    return received_outcome(value, checkpoint());
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
    pub(super) fn request_stop(&mut self) {
        self.sender.take();
        self.stop.cancel();
    }
    pub(super) fn finish(mut self) -> Result<(), WorkerError> {
        self.join()
    }
    fn fail(mut self, cause: WorkerError) -> WorkerError {
        match self.join() {
            Ok(()) => cause,
            Err(cleanup) => WorkerError::Retirement {
                causes: vec![cause, cleanup],
            },
        }
    }
    fn join(&mut self) -> Result<(), WorkerError> {
        self.request_stop();
        match self.thread.take().map(std::thread::JoinHandle::join) {
            None => Ok(()),
            Some(Ok(result)) => result,
            Some(Err(payload)) => Err(WorkerError::IoThread(
                pse_diagnostics::panic::settle_payload(payload, "non-text panic payload"),
            )),
        }
    }
}
impl Drop for IoOwner {
    fn drop(&mut self) {
        if let Err(error) = self.join() {
            tracing::error!(error = %error, "scientific worker I/O abandonment settlement failed");
        }
    }
}

struct Job {
    request: p::Request,
    completion: NativeCompletion,
}
struct NativeCompletion {
    response: tokio::sync::oneshot::Sender<NativeReply>,
    // The native job, rather than the observing RPC, owns numerical-lane admission.
    _work: Option<tokio::sync::OwnedSemaphorePermit>,
}
impl NativeCompletion {
    fn deliver(self, response: p::Response) -> std::sync::mpsc::Receiver<DeliveryDisposition> {
        let (observed, observation) = std::sync::mpsc::channel();
        // If the method has departed, dropping the undelivered payload transfers
        // its admission back to the native owner just like a later guard drop.
        let _ = self.response.send(NativeReply {
            response,
            delivery: Delivery {
                observed: Some(observed),
                work: self._work,
            },
        });
        observation
    }
    fn finish(
        self,
        response: p::Response,
        preparation: Option<p::Handle>,
        retire: impl FnOnce(p::Handle) -> Result<(), WorkerError>,
    ) -> Result<(), WorkerError> {
        settle_delivery(self.deliver(response), preparation, retire)
    }
}
#[derive(Debug)]
struct NativeReply {
    response: p::Response,
    delivery: Delivery,
}
#[derive(Debug)]
struct Delivery {
    observed: Option<std::sync::mpsc::Sender<DeliveryDisposition>>,
    work: Option<tokio::sync::OwnedSemaphorePermit>,
}
#[derive(Debug)]
enum DeliveryDisposition {
    Consumed,
    Abandoned(Option<tokio::sync::OwnedSemaphorePermit>),
}
impl Delivery {
    fn consume(mut self) {
        if let Some(observed) = self.observed.take() {
            let _ = observed.send(DeliveryDisposition::Consumed);
        }
        // No await follows consumption in the observing method. Its native work
        // is settled, so release admission before returning the encoded response.
        self.work.take();
    }
}
impl Drop for Delivery {
    fn drop(&mut self) {
        if let Some(observed) = self.observed.take() {
            let _ = observed.send(DeliveryDisposition::Abandoned(self.work.take()));
        }
    }
}
fn settle_delivery(
    observation: std::sync::mpsc::Receiver<DeliveryDisposition>,
    preparation: Option<p::Handle>,
    retire: impl FnOnce(p::Handle) -> Result<(), WorkerError>,
) -> Result<(), WorkerError> {
    match observation.recv().map_err(infrastructure)? {
        DeliveryDisposition::Consumed => Ok(()),
        DeliveryDisposition::Abandoned(_work) => {
            // Local method departure is known, unlike a lost wire reply after
            // consumption. Hold returned admission through native retirement.
            if let Some(handle) = preparation {
                retire(handle)?;
            }
            Ok(())
        }
    }
}
#[derive(Debug)]
struct Submitted {
    response: tokio::sync::oneshot::Receiver<NativeReply>,
}
#[derive(Debug)]
enum SubmissionError {
    Admission(WorkerError),
    Departed,
}
#[derive(Default)]
struct HandleControls {
    closed: bool,
    handles: BTreeMap<SemanticId, (p::Handle, pse_diagnostics::Cancellation)>,
}
impl HandleControls {
    fn signal(&self, request: &p::Request) -> Result<(), WorkerError> {
        if self.closed {
            return Err(
                ProviderError::Contract("scientific worker admission is closed".into()).into(),
            );
        }
        match request {
            p::Request::Stop(handle) => self.control(handle)?.cancel(),
            p::Request::Shutdown => {
                for (_, cancellation) in self.handles.values() {
                    cancellation.cancel();
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn admit(&mut self, request: &p::Request) -> Result<(), WorkerError> {
        if self.closed {
            return Err(
                ProviderError::Contract("scientific worker admission is closed".into()).into(),
            );
        }
        match request {
            p::Request::Prepare(value) => {
                if self.handles.len() >= 64 || self.handles.contains_key(&value.handle.generation) {
                    return Err(ProviderError::Contract(
                        "scientific worker duplicate/excess preparation".into(),
                    )
                    .into());
                }
                self.handles.insert(
                    value.handle.generation,
                    (value.handle.clone(), Default::default()),
                );
            }
            p::Request::Evaluate(value) => {
                self.control(&value.handle)?;
            }
            p::Request::EvaluateBatch(value) => {
                self.control(&value.handle)?;
            }
            p::Request::Close(handle) => {
                self.control(handle)?;
            }
            p::Request::Stop(handle) => self.control(handle)?.cancel(),
            p::Request::Shutdown => {
                self.closed = true;
                for (_, cancellation) in self.handles.values() {
                    cancellation.cancel();
                }
            }
        }
        Ok(())
    }
    fn control(&self, handle: &p::Handle) -> Result<&pse_diagnostics::Cancellation, WorkerError> {
        self.handles
            .get(&handle.generation)
            .filter(|(actual, _)| actual == handle)
            .map(|(_, cancellation)| cancellation)
            .ok_or_else(|| {
                ProviderError::Contract("scientific worker unknown or foreign handle".into()).into()
            })
    }
}
type SharedControls = Arc<std::sync::Mutex<HandleControls>>;
fn controls_lock(
    controls: &SharedControls,
) -> Result<std::sync::MutexGuard<'_, HandleControls>, WorkerError> {
    controls
        .lock()
        .map_err(|_| infrastructure("scientific worker controls poisoned"))
}
fn close_native_admission(controls: &SharedControls, joined: bool) -> Result<(), WorkerError> {
    let mut controls = controls_lock(controls)?;
    controls.closed = true;
    if joined {
        controls.handles.clear();
    } else {
        for (_, cancellation) in controls.handles.values() {
            cancellation.cancel();
        }
    }
    Ok(())
}
type EncodedResponse = connectrpc::PreEncoded<pb::Response>;
fn encode_response(value: p::Response) -> Result<EncodedResponse, connectrpc::ConnectError> {
    use buffa::Message as _;
    let message = pb::Response::try_from(value)
        .map_err(|error| connectrpc::ConnectError::internal(error.to_string()))?;
    let length = usize::try_from(
        message
            .try_encoded_len()
            .map_err(|error| connectrpc::ConnectError::resource_exhausted(error.to_string()))?,
    )
    .map_err(|error| connectrpc::ConnectError::resource_exhausted(error.to_string()))?;
    if length > FRAME_LIMIT {
        return Err(connectrpc::ConnectError::resource_exhausted(
            "scientific worker response frame allowance",
        ));
    }
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(length)
        .map_err(|error| connectrpc::ConnectError::resource_exhausted(error.to_string()))?;
    message
        .try_encode_bounded(
            u32::try_from(FRAME_LIMIT)
                .map_err(|error| connectrpc::ConnectError::resource_exhausted(error.to_string()))?,
            &mut encoded,
        )
        .map_err(|error| connectrpc::ConnectError::resource_exhausted(error.to_string()))?;
    // Only locally generated, validated, bounded bytes of this exact type enter
    // PreEncoded. Binary ingress excludes its JSON fallback. Bytes clones retain
    // this one encoding through the last concurrent response/body owner.
    Ok(EncodedResponse::from_bytes_unchecked(encoded))
}
fn join_native<T>(
    runtime: tokio::runtime::Runtime,
    native: std::thread::JoinHandle<T>,
) -> Result<T, WorkerError> {
    // Hyper's connection future can depart before its spawned method tasks.
    // Destroy their delivery guards before joining the lane that observes them.
    drop(runtime);
    join_native_handle(native)
}
fn join_native_handle<T>(native: std::thread::JoinHandle<T>) -> Result<T, WorkerError> {
    native.join().map_err(|payload| {
        WorkerError::NativePanic(pse_diagnostics::panic::settle_payload(
            payload,
            "non-text native lane panic payload",
        ))
    })
}
fn retire_sessions(
    sessions: &mut BTreeMap<SemanticId, PreparedSession>,
) -> Result<(), WorkerError> {
    let mut failures = Vec::new();
    for (_, mut owner) in std::mem::take(sessions) {
        let handle = p::Handle::from(owner.handle());
        if let Err(cause) = owner.stop() {
            failures.push(WorkerError::Lifecycle {
                action: "Stop",
                handle: Some(handle),
                cause: Box::new(cause),
            });
        }
    }
    super::retirement_result(failures)
}
struct Server {
    jobs: tokio::sync::mpsc::Sender<Job>,
    controls: SharedControls,
    work: Arc<tokio::sync::Semaphore>,
    terminal: tokio::sync::watch::Receiver<Option<EncodedResponse>>,
}
impl Server {
    async fn terminal_response(&self) -> Result<EncodedResponse, connectrpc::ConnectError> {
        let mut terminal = self.terminal.clone();
        loop {
            if let Some(response) = terminal.borrow_and_update().clone() {
                return Ok(response);
            }
            terminal.changed().await.map_err(|_| {
                connectrpc::ConnectError::unavailable(
                    "scientific worker joined outcome unavailable",
                )
            })?;
        }
    }
    async fn submit(
        &self,
        request: p::Request,
        work: Option<tokio::sync::OwnedSemaphorePermit>,
    ) -> Result<Submitted, SubmissionError> {
        let immediate = matches!(&request, p::Request::Stop(_) | p::Request::Shutdown);
        if immediate {
            // Cancellation must reach active native work even while its queue is
            // full. The later queued operation owns physical retirement.
            controls_lock(&self.controls)
                .and_then(|controls| controls.signal(&request))
                .map_err(SubmissionError::Admission)?;
        }
        let permit = self
            .jobs
            .clone()
            .reserve_owned()
            .await
            .map_err(|_| SubmissionError::Departed)?;
        controls_lock(&self.controls)
            .and_then(|mut controls| controls.admit(&request))
            .map_err(SubmissionError::Admission)?;
        // Admission and handoff have no suspension point. A canceled reservation
        // cannot leave a phantom prepared handle in the control map.
        let (response, received) = tokio::sync::oneshot::channel();
        #[cfg(test)]
        let closing = match &request {
            p::Request::Close(handle) => Some(handle.clone()),
            _ => None,
        };
        permit.send(Job {
            request,
            completion: NativeCompletion {
                response,
                _work: work,
            },
        });
        #[cfg(test)]
        observe_close_handoff_for_test(closing);
        Ok(Submitted { response: received })
    }
}
impl services::ScientificWorker for Server {
    async fn exchange<'a>(
        &'a self,
        _: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::Request>,
    ) -> connectrpc::ServiceResult<impl connectrpc::Encodable<pb::Response> + Send + use<'a>> {
        let request = match p::Request::try_from(request.to_owned_message()) {
            Ok(value) => value,
            Err(error) => {
                return Ok(connectrpc::Response::new(encode_response(
                    p::Response::Failure(handler::failure(error)),
                )?));
            }
        };
        if let Some(response) = self.terminal.borrow().clone() {
            return Ok(connectrpc::Response::new(response));
        }
        if self.jobs.is_closed() {
            return Ok(connectrpc::Response::new(self.terminal_response().await?));
        }
        let work = if matches!(
            &request,
            p::Request::Prepare(_) | p::Request::Evaluate(_) | p::Request::EvaluateBatch(_)
        ) {
            match self.work.clone().try_acquire_owned() {
                Ok(permit) => Some(permit),
                Err(_) => {
                    return Ok(connectrpc::Response::new(encode_response(
                        p::Response::Failure(p::Failure::Resource(
                            "scientific worker numerical lane is busy".into(),
                        )),
                    )?));
                }
            }
        } else {
            None
        };
        let submitted = match self.submit(request, work).await {
            Ok(submitted) => submitted,
            Err(SubmissionError::Admission(error)) => {
                if self.jobs.is_closed() {
                    return Ok(connectrpc::Response::new(self.terminal_response().await?));
                }
                return Ok(connectrpc::Response::new(encode_response(
                    p::Response::Failure(handler::failure(error)),
                )?));
            }
            Err(SubmissionError::Departed) => {
                return Ok(connectrpc::Response::new(self.terminal_response().await?));
            }
        };
        let reply = match submitted.response.await {
            Ok(reply) => reply,
            Err(_) => return Ok(connectrpc::Response::new(self.terminal_response().await?)),
        };
        let response = connectrpc::Response::new(encode_response(reply.response)?);
        // This final synchronous section acknowledges only local method
        // consumption. The transport can still lose the later network reply.
        reply.delivery.consume();
        Ok(response)
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

    let parent_owner = ParentGuard::start(parent)?;
    let listener = std::os::unix::net::UnixListener::bind(&endpoint).map_err(infrastructure)?;
    std::fs::set_permissions(
        &endpoint,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o600),
    )
    .map_err(infrastructure)?;
    listener.set_nonblocking(true).map_err(infrastructure)?;
    let controls = SharedControls::default();
    let native_controls = controls.clone();
    let closed = pse_diagnostics::Cancellation::default();
    let drained = closed.clone();
    let (jobs, mut receiving) = tokio::sync::mpsc::channel::<Job>(1);
    let (native_finished, mut native_completion) = tokio::sync::oneshot::channel::<()>();
    let (terminal, terminal_reader) = tokio::sync::watch::channel(None);
    // Prepare the small omission response before publication. Even if deriving
    // or encoding a larger observation refuses allocation, readers can retain
    // this existing Bytes owner without another allocation or a false success.
    let omitted = encode_response(p::Response::Failure(p::Failure::UnknownCompletion(
        "scientific worker could not transfer the complete joined failure within its declared failure allowance".into(),
    ))).map_err(WorkerError::Exchange)?;
    let native = std::thread::Builder::new()
        .name("pse-science-native".into())
        .stack_size(NATIVE_STACK)
        .spawn(move || {
            // Sender departure wakes the composition root on return or panic.
            // It carries no result/settlement claim: only the original join
            // handle below can establish and return the native outcome.
            let _finished = native_finished;
            let mut sessions = BTreeMap::<SemanticId, PreparedSession>::new();
            let mut failures = Vec::new();
            while let Some(job) = receiving.blocking_recv() {
                let shutdown = matches!(&job.request, p::Request::Shutdown);
                let preparation = match &job.request {
                    p::Request::Prepare(value) => Some(value.handle.clone()),
                    _ => None,
                };
                let handle = match &job.request {
                    p::Request::Prepare(value) => Some(value.handle.clone()),
                    p::Request::Evaluate(value) => Some(value.handle.clone()),
                    p::Request::EvaluateBatch(value) => Some(value.handle.clone()),
                    p::Request::Stop(handle) | p::Request::Close(handle) => Some(handle.clone()),
                    p::Request::Shutdown => None,
                };
                let mut result: Result<p::Response, WorkerError> = (|| match job.request {
                    p::Request::Prepare(value) => {
                        if drained.is_cancelled()
                            || sessions.len() >= 64
                            || sessions.contains_key(&value.handle.generation)
                        {
                            return Err(ProviderError::Contract(
                                "scientific worker duplicate/excess preparation".into(),
                            )
                            .into());
                        }
                        let cancellation = controls_lock(&native_controls)?
                            .control(&value.handle)?
                            .clone();
                        let prepared = prepare(value, cancellation)?;
                        let handle = p::Handle::from(prepared.handle());
                        let batch_supported = prepared.supports_batch();
                        sessions.insert(handle.generation, prepared);
                        Ok(p::Response::Prepared(p::Prepared {
                            handle,
                            batch_supported,
                        }))
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
                    p::Request::EvaluateBatch(value) => {
                        let owner =
                            sessions.get_mut(&value.handle.generation).ok_or_else(|| {
                                ProviderError::Contract(
                                    "scientific worker unknown generation".into(),
                                )
                            })?;
                        Ok(p::Response::Batch(handler::evaluate_batch(owner, value)?))
                    }
                    p::Request::Stop(handle) => {
                        let actual = sessions.get_mut(&handle.generation).ok_or_else(|| {
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
                        let retired = actual.stop();
                        sessions.remove(&handle.generation);
                        let mut failures: Vec<_> = retired.err().into_iter().collect();
                        match controls_lock(&native_controls) {
                            Ok(mut controls) => {
                                controls.handles.remove(&handle.generation);
                            }
                            Err(error) => failures.push(error),
                        }
                        super::retirement_result(failures)?;
                        Ok(p::Response::Closed(handle))
                    }
                    p::Request::Close(handle) => {
                        let actual = sessions.get_mut(&handle.generation).ok_or_else(|| {
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
                        let retired = actual.close();
                        sessions.remove(&handle.generation);
                        let mut failures: Vec<_> = retired.err().into_iter().collect();
                        match controls_lock(&native_controls) {
                            Ok(mut controls) => {
                                controls.handles.remove(&handle.generation);
                            }
                            Err(error) => failures.push(error),
                        }
                        super::retirement_result(failures)?;
                        Ok(p::Response::Closed(handle))
                    }
                    p::Request::Shutdown => {
                        let mut failures: Vec<_> =
                            retire_sessions(&mut sessions).err().into_iter().collect();
                        match controls_lock(&native_controls) {
                            Ok(mut controls) => controls.handles.clear(),
                            Err(error) => failures.push(error),
                        }
                        super::retirement_result(failures)?;
                        Ok(p::Response::Shutdown)
                    }
                })();
                let terminal = result.as_ref().err().is_some_and(|error| match error {
                    WorkerError::Provider(error) => !error.recoverable(),
                    _ => true,
                });
                if terminal && let Some(handle) = handle {
                    // A failed handle cannot erase unrelated prepared science. The parent
                    // still settles the actual process if this connection is abandoned.
                    if let Some(owner) = sessions.get_mut(&handle.generation) {
                        // Native refusal before mutation (for example a replayed ordinal)
                        // leaves the admitted preparation ready. Preserve that owner's decision.
                        if owner.state() != super::SessionState::Ready
                            && let Err(cleanup) = owner.stop()
                        {
                            result = Err(match result {
                                Err(cause) => WorkerError::Retirement {
                                    causes: vec![cause, cleanup],
                                },
                                Ok(_) => cleanup,
                            });
                        }
                    } else if let Ok(mut controls) = controls_lock(&native_controls) {
                        // Failed preparation never issued a live native handle.
                        controls.handles.remove(&handle.generation);
                    }
                }
                let response = match result {
                    Ok(response) => response,
                    Err(error) if shutdown => {
                        // The RPC is a passive observation. The original
                        // process-wide retirement failure belongs to join.
                        let observation = handler::failure_observation(&error);
                        failures.push(error);
                        p::Response::Failure(observation)
                    }
                    Err(error) => p::Response::Failure(handler::failure(error)),
                };
                #[cfg(test)]
                if shutdown
                    && !failures.is_empty()
                    && let Err(error) = observe_shutdown_departure_for_test(&job.completion)
                {
                    failures.push(error);
                    break;
                }
                if shutdown {
                    // Every native owner has settled; begin peer-led H2 drain
                    // while the method still owns its response stream.
                    drained.cancel();
                }
                #[cfg(test)]
                let response = perturb_response_for_test(response);
                #[cfg(test)]
                if let Err(error) = observe_prepare_departure_for_test(
                    &job.completion,
                    &response,
                    &sessions,
                    &native_controls,
                ) {
                    failures.push(error);
                    break;
                }
                if let Err(error) = job.completion.finish(response, preparation, |handle| {
                    let mut failures = Vec::new();
                    if let Some(mut owner) = sessions.remove(&handle.generation)
                        && let Err(error) = owner.stop()
                    {
                        failures.push(error);
                    }
                    match controls_lock(&native_controls) {
                        Ok(mut controls) => {
                            controls.handles.remove(&handle.generation);
                        }
                        Err(error) => failures.push(error),
                    }
                    let result = super::retirement_result(failures);
                    #[cfg(test)]
                    observe_orphan_retirement_for_test(
                        &handle,
                        &sessions,
                        &native_controls,
                        result.is_ok(),
                    )?;
                    result
                }) {
                    failures.push(error);
                }
                if !failures.is_empty() {
                    break;
                }
            }
            if let Err(error) = retire_sessions(&mut sessions) {
                failures.push(error);
            }
            #[cfg(test)]
            observe_native_retirement_failures_for_test(&failures)?;
            super::retirement_result(failures)
        })
        .map_err(infrastructure)?;
    let mut native = Some(native);
    let mut joined_native = None;
    let mut admission_closed = false;
    let mut admission_error = None;
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
        let (stream, peer_reader) =
            retain_peer_reader(stream).map_err(|source| WorkerError::Process {
                action: "retain parent socket",
                source,
            })?;
        let router = Arc::new(Server {
            jobs: jobs.clone(),
            controls: controls.clone(),
            work: Arc::new(tokio::sync::Semaphore::new(1)),
            terminal: terminal_reader,
        })
        .register(connectrpc::Router::new());
        let service = connectrpc::ConnectRpcService::new(router).with_limits(
            connectrpc::Limits::default()
                .with_max_request_body_size(FRAME_LIMIT)
                .with_max_message_size(FRAME_LIMIT)
                .with_element_memory_limit(DECODE_LIMIT),
        );
        let service =
            hyper::service::service_fn(move |request| binary_grpc(service.clone(), request));
        let mut builder =
            hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new());
        // One numerical request and one bounded control request may coexist. Native
        // mutation stays on the single owned lane; cancellation is independently signaled.
        builder.max_concurrent_streams(2);
        let connection = builder.serve_connection(hyper_util::rt::TokioIo::new(stream), service);
        tokio::pin!(connection);
        let result = loop {
            tokio::select! {
                biased;
                _ = &mut native_completion, if native.is_some() => {
                    let handle = native.take().ok_or_else(|| infrastructure("scientific worker native join already consumed"))?;
                    let result = join_native_handle(handle).and_then(std::convert::identity);
                    // Joining, not the wakeup, establishes completed native
                    // ownership. No future method may enter numerical work.
                    admission_error = close_native_admission(&controls, true).err();
                    admission_closed = true;
                    let response = match &result {
                        Err(error) => encode_response(p::Response::Failure(handler::failure_observation(error)))
                            .unwrap_or_else(|_| omitted.clone()),
                        Ok(()) => omitted.clone(),
                    };
                    joined_native = Some(result);
                    terminal.send_replace(Some(response));
                    #[cfg(test)]
                    observe_native_join_for_test()?;
                },
                _ = closed.cancelled() => {
                    connection.as_mut().graceful_shutdown();
                    break connection.await.map_err(WorkerError::Connection);
                },
                result = connection.as_mut() => break result.map_err(WorkerError::Connection),
            }
        };
        let mut failures = result.err().into_iter().collect::<Vec<_>>();
        // Server H2 completion can also precede unread client shutdown frames.
        // Keep both receive halves alive until each writer has half-closed;
        // closing the last socket with unread data would reset its peer.
        if closed.is_cancelled()
            && let Err(source) = finish_peer_reader(peer_reader).await
        {
            failures.push(WorkerError::Process {
                action: "finish parent socket",
                source,
            });
        }
        super::retirement_result(failures)
    });
    // Connection departure can race the native completion wake. Consume its
    // original join when available before deciding native work is unsettled.
    if joined_native.is_none()
        && !matches!(
            native_completion.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        )
    {
        let handle = native
            .take()
            .ok_or_else(|| infrastructure("scientific worker native join already consumed"))?;
        joined_native = Some(join_native_handle(handle).and_then(std::convert::identity));
    }
    if !admission_closed {
        admission_error = close_native_admission(&controls, joined_native.is_some()).err();
    }
    if !closed.is_cancelled() && joined_native.is_none() {
        // A disconnected active RPC must settle its actual native call, even if
        // cooperative cancellation cannot interrupt the foreign instruction.
        parent_owner.terminate_unsettled_native();
    }
    drop(jobs);
    let native = match joined_native {
        Some(result) => {
            drop(runtime);
            result
        }
        None => join_native(
            runtime,
            native.ok_or_else(|| infrastructure("scientific worker native join absent"))?,
        )
        .and_then(std::convert::identity),
    };
    let failures = result
        .err()
        .into_iter()
        .chain(native.err())
        .chain(admission_error)
        .collect();
    super::retirement_result(failures)
}

#[cfg(test)]
fn observe_native_join_for_test() -> Result<(), WorkerError> {
    if std::env::var("PSE_SCIENCE_TEST_MODE").as_deref() == Ok("prepare-departure-retire-faults") {
        let marker = PathBuf::from(
            std::env::var_os("PSE_SCIENCE_TEST_MARKER")
                .ok_or_else(|| infrastructure("missing native join test marker"))?,
        );
        std::fs::write(marker.with_extension("native-joined"), b"joined")
            .map_err(infrastructure)?;
    }
    Ok(())
}

#[cfg(test)]
fn departing_preparation_marker_for_test(handle: &p::Handle) -> Option<PathBuf> {
    if !std::env::var("PSE_SCIENCE_TEST_MODE")
        .is_ok_and(|mode| mode.starts_with("prepare-departure"))
    {
        return None;
    }
    let marker = PathBuf::from(std::env::var_os("PSE_SCIENCE_TEST_MARKER")?);
    (std::fs::read_to_string(marker.with_extension("prepare-built"))
        .ok()
        .as_deref()
        == Some(handle.generation.to_string().as_str()))
    .then_some(marker)
}

#[cfg(test)]
fn observe_prepare_departure_for_test(
    completion: &NativeCompletion,
    response: &p::Response,
    sessions: &BTreeMap<SemanticId, PreparedSession>,
    controls: &SharedControls,
) -> Result<(), WorkerError> {
    let p::Response::Prepared(prepared) = response else {
        return Ok(());
    };
    let Some(marker) = departing_preparation_marker_for_test(&prepared.handle) else {
        return Ok(());
    };
    if !sessions.contains_key(&prepared.handle.generation) {
        return Err(infrastructure(
            "departing preparation has no actual native owner",
        ));
    }
    controls_lock(controls)?.control(&prepared.handle)?;
    std::fs::write(
        marker.with_extension("prepare-claimed"),
        prepared.handle.generation.to_string(),
    )
    .map_err(infrastructure)?;
    wait_for_method_departure_for_test(completion)?;
    std::fs::write(
        marker.with_extension("prepare-departed"),
        prepared.handle.generation.to_string(),
    )
    .map_err(infrastructure)
}

#[cfg(test)]
fn observe_shutdown_departure_for_test(completion: &NativeCompletion) -> Result<(), WorkerError> {
    if std::env::var("PSE_SCIENCE_TEST_MODE").as_deref() != Ok("shutdown-departure-retire-faults") {
        return Ok(());
    }
    let marker = PathBuf::from(
        std::env::var_os("PSE_SCIENCE_TEST_MARKER")
            .ok_or_else(|| infrastructure("missing shutdown departure test marker"))?,
    );
    std::fs::write(marker.with_extension("shutdown-claimed"), b"retired")
        .map_err(infrastructure)?;
    wait_for_method_departure_for_test(completion)
}

#[cfg(test)]
fn wait_for_method_departure_for_test(completion: &NativeCompletion) -> Result<(), WorkerError> {
    // Tokio 1.53.3 oneshot::Sender::is_closed guarantees send failure after
    // receiver departure. Wait for that actual server-method fact, never infer
    // it from dropping the client future or from a delay. This test-only wait
    // does not introduce a protocol or scientific-operation deadline.
    let end = std::time::Instant::now() + Duration::from_secs(10);
    while !completion.response.is_closed() {
        if std::time::Instant::now() >= end {
            return Err(infrastructure(
                "actual generated method did not depart before late response delivery",
            ));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

#[cfg(test)]
fn observe_orphan_retirement_for_test(
    handle: &p::Handle,
    sessions: &BTreeMap<SemanticId, PreparedSession>,
    controls: &SharedControls,
    successful: bool,
) -> Result<(), WorkerError> {
    let Some(marker) = departing_preparation_marker_for_test(handle) else {
        return Ok(());
    };
    if sessions.contains_key(&handle.generation)
        || controls_lock(controls)?
            .handles
            .contains_key(&handle.generation)
    {
        return Err(infrastructure(
            "abandoned preparation retained native or control ownership",
        ));
    }
    std::fs::write(
        marker.with_extension(if successful {
            "prepare-retired"
        } else {
            "prepare-retirement-error"
        }),
        handle.generation.to_string(),
    )
    .map_err(infrastructure)
}

#[cfg(test)]
fn observe_native_retirement_failures_for_test(
    failures: &[WorkerError],
) -> Result<(), WorkerError> {
    let mode = std::env::var("PSE_SCIENCE_TEST_MODE");
    let shutdown = mode.as_deref() == Ok("shutdown-departure-retire-faults");
    if !shutdown && mode.as_deref() != Ok("prepare-departure-retire-faults") {
        return Ok(());
    }
    let marker = PathBuf::from(
        std::env::var_os("PSE_SCIENCE_TEST_MARKER")
            .ok_or_else(|| infrastructure("missing retirement-failure test marker"))?,
    );
    fn original_finish(error: &WorkerError, generation: &str) -> Result<(), WorkerError> {
        let WorkerError::Provider(ProviderError::Retirement { failures }) = error else {
            return Err(infrastructure(
                "native finish lost original keyed provider retirement failure",
            ));
        };
        let [failure] = failures.as_slice() else {
            return Err(infrastructure(
                "native finish changed original failure population",
            ));
        };
        if !matches!(failure.cause.as_ref(), ProviderError::Contract(detail)
            if detail == &format!("controlled native retirement failure {generation}"))
        {
            return Err(infrastructure(
                "native finish lost original typed Contract cause",
            ));
        }
        Ok(())
    }
    let mut generations = Vec::new();
    let remaining = if shutdown {
        let [WorkerError::Retirement { causes }] = failures else {
            return Err(infrastructure(
                "native Shutdown lost its original retirement failures",
            ));
        };
        causes
    } else {
        let orphan = std::fs::read_to_string(marker.with_extension("prepare-built"))
            .map_err(infrastructure)?;
        let [original, WorkerError::Retirement { causes }] = failures else {
            return Err(infrastructure(
                "native failure did not retain original orphan and remaining-owner failures",
            ));
        };
        original_finish(original, &orphan)?;
        generations.push(orphan);
        causes
    };
    if remaining.len() != 2 {
        return Err(infrastructure(
            "native failure skipped a remaining prepared owner",
        ));
    }
    for error in remaining {
        let WorkerError::Lifecycle {
            action: "Stop",
            handle: Some(handle),
            cause,
        } = error
        else {
            return Err(infrastructure(
                "remaining native finish lost exact Stop handle attribution",
            ));
        };
        let generation = handle.generation.to_string();
        original_finish(cause, &generation)?;
        generations.push(generation);
    }
    // This witness checks the child-owned typed errors. The parent separately
    // checks its passive wire observation and actual process settlement.
    generations.sort();
    std::fs::write(
        marker.with_extension("native-errors"),
        generations.join("\n"),
    )
    .map_err(infrastructure)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "assertions in the explicitly launched controlled test child"
)]
fn observe_close_handoff_for_test(closing: Option<p::Handle>) {
    if let Some(handle) = closing
        && std::env::var("PSE_SCIENCE_TEST_MODE").as_deref() == Ok("close-pending")
    {
        let marker = PathBuf::from(std::env::var_os("PSE_SCIENCE_TEST_MARKER").unwrap());
        // Test-child observation of the actual committed queue handoff,
        // independent of thread scheduling or later acknowledgement.
        std::os::unix::net::UnixDatagram::unbound()
            .unwrap()
            .send_to(
                format!("queued:{}", handle.generation).as_bytes(),
                marker.with_extension("close-events"),
            )
            .unwrap();
    }
}

#[cfg(test)]
fn perturb_response_for_test(mut response: p::Response) -> p::Response {
    // Only the explicitly launched library test child includes these faults.
    // The deployed executable has neither the branch nor its environment input.
    if std::env::var("PSE_SCIENCE_TEST_MODE").as_deref() == Ok("finish-ack-faults") {
        let action = match response {
            p::Response::Closed(_) => Some("Close"),
            p::Response::Shutdown => Some("Shutdown"),
            _ => None,
        };
        if let Some(action) = action {
            use std::io::Write as _;
            let marker = PathBuf::from(std::env::var_os("PSE_SCIENCE_TEST_MARKER").unwrap());
            writeln!(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(marker.with_extension("lifecycle"))
                    .unwrap(),
                "{action}"
            )
            .unwrap();
            return p::Response::Failure(p::Failure::Contract(format!(
                "controlled {action} acknowledgement refusal"
            )));
        }
    }
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
    if let p::Response::Batch(batch) = &mut response {
        match std::env::var("PSE_SCIENCE_TEST_MODE").as_deref() {
            Ok("water-batch-wrong-shape") => {
                if let Some(p::BatchOutcome {
                    result: p::BatchResult::Product(product),
                    ..
                }) = batch.points.last_mut()
                {
                    product.values.clear();
                }
            }
            Ok("water-batch-wrong-point") => {
                if let Some(point) = batch.points.last_mut() {
                    point.id = SemanticId::NIL;
                }
            }
            Ok("water-batch-missing-point") => {
                batch.points.pop();
            }
            Ok("water-batch-nonfinite") => {
                if let Some(p::BatchOutcome {
                    result: p::BatchResult::Product(product),
                    ..
                }) = batch.points.last_mut()
                {
                    product.values.fill(f64::NAN);
                }
            }
            _ => {}
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ObservedIngress {
        reads: Arc<std::sync::atomic::AtomicUsize>,
        frame: Option<bytes::Bytes>,
    }
    impl http_body::Body for ObservedIngress {
        type Data = bytes::Bytes;
        type Error = std::convert::Infallible;
        fn poll_frame(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            let body = self.get_mut();
            body.reads.fetch_add(1, Ordering::SeqCst);
            std::task::Poll::Ready(
                body.frame
                    .take()
                    .map(|frame| Ok(http_body::Frame::data(frame))),
            )
        }
    }

    #[tokio::test]
    async fn scientific_worker_ingress_rejects_nonbinary_protocols_before_body_or_dispatch() {
        use http_body_util::BodyExt as _;
        use std::sync::atomic::AtomicUsize;
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let router = connectrpc::Router::new().route::<_, pb::Request, pb::Response>(
            "pse.scientific_worker.v1.ScientificWorker",
            "Exchange",
            connectrpc::handler::handler_fn(move |_, _: pb::Request| {
                observed.fetch_add(1, Ordering::SeqCst);
                async { Ok(connectrpc::Response::new(pb::Response::default())) }
            }),
        );
        let service = connectrpc::ConnectRpcService::new(router).with_limits(
            connectrpc::Limits::default()
                .with_max_request_body_size(FRAME_LIMIT)
                .with_max_message_size(FRAME_LIMIT)
                .with_element_memory_limit(DECODE_LIMIT),
        );
        for (method, version, content_type) in [
            (
                http::Method::POST,
                http::Version::HTTP_2,
                Some("application/json"),
            ),
            (
                http::Method::POST,
                http::Version::HTTP_2,
                Some("application/proto"),
            ),
            (
                http::Method::POST,
                http::Version::HTTP_2,
                Some("application/connect+proto"),
            ),
            (
                http::Method::POST,
                http::Version::HTTP_2,
                Some("application/grpc+json"),
            ),
            (
                http::Method::POST,
                http::Version::HTTP_2,
                Some("application/grpc-web+proto"),
            ),
            (
                http::Method::POST,
                http::Version::HTTP_2,
                Some("application/grpc-web-text+proto"),
            ),
            (http::Method::GET, http::Version::HTTP_2, None),
            (
                http::Method::PUT,
                http::Version::HTTP_2,
                Some("application/grpc+proto"),
            ),
            (
                http::Method::POST,
                http::Version::HTTP_11,
                Some("application/grpc+proto"),
            ),
            (http::Method::POST, http::Version::HTTP_2, None),
        ] {
            let reads = Arc::new(AtomicUsize::new(0));
            let mut request = http::Request::builder()
                .method(method)
                .version(version)
                .uri("/pse.scientific_worker.v1.ScientificWorker/Exchange");
            if let Some(content_type) = content_type {
                request = request.header(http::header::CONTENT_TYPE, content_type);
            }
            let request = request
                .body(ObservedIngress {
                    reads: reads.clone(),
                    frame: Some(bytes::Bytes::from_static(&[0, 0, 0, 0, 0])),
                })
                .unwrap();
            let response = binary_grpc(service.clone(), request).await.unwrap();
            assert_eq!(reads.load(Ordering::SeqCst), 0);
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            if content_type == Some("application/grpc+json") {
                assert_eq!(response.headers()["grpc-status"], "12");
            }
        }
        for content_type in [
            "application/grpc",
            "application/grpc+proto",
            "application/grpc+proto; charset=utf-8",
        ] {
            let reads = Arc::new(AtomicUsize::new(0));
            let request = http::Request::builder()
                .method("POST")
                .version(http::Version::HTTP_2)
                .uri("/pse.scientific_worker.v1.ScientificWorker/Exchange")
                .header(http::header::CONTENT_TYPE, content_type)
                .body(ObservedIngress {
                    reads: reads.clone(),
                    frame: Some(bytes::Bytes::from_static(&[0, 0, 0, 0, 0])),
                })
                .unwrap();
            let response = binary_grpc(service.clone(), request).await.unwrap();
            let collected = response.into_body().collect().await.unwrap();
            assert_eq!(collected.trailers().unwrap()["grpc-status"], "0");
            assert!(reads.load(Ordering::SeqCst) > 0);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    struct GatedShutdownWrites {
        stream: tokio::net::UnixStream,
        armed: Arc<AtomicBool>,
        first_write: bool,
        blocked: Option<tokio::sync::oneshot::Sender<()>>,
        release: Option<tokio::sync::oneshot::Receiver<()>>,
    }
    impl tokio::io::AsyncRead for GatedShutdownWrites {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buffer: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::pin::Pin::new(&mut self.stream).poll_read(cx, buffer)
        }
    }
    impl tokio::io::AsyncWrite for GatedShutdownWrites {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buffer: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            if self.armed.load(Ordering::Acquire) && self.first_write {
                if let Some(blocked) = self.blocked.take() {
                    blocked.send(()).unwrap();
                }
                if let Some(release) = &mut self.release {
                    match std::pin::Pin::new(release).poll(cx) {
                        std::task::Poll::Pending => return std::task::Poll::Pending,
                        std::task::Poll::Ready(result) => result.unwrap(),
                    }
                    self.release = None;
                }
            }
            let result = std::pin::Pin::new(&mut self.stream).poll_write(cx, buffer);
            if self.armed.load(Ordering::Acquire)
                && matches!(&result, std::task::Poll::Ready(Ok(written)) if *written > 0)
            {
                self.first_write = true;
            }
            result
        }
        fn poll_flush(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::pin::Pin::new(&mut self.stream).poll_flush(cx)
        }
        fn poll_shutdown(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
        }
    }

    #[tokio::test]
    async fn peer_final_h2_write_survives_public_driver_and_executor_completion() {
        use http_body_util::BodyExt as _;
        tokio::time::timeout(Duration::from_secs(5), async {
            let (client, peer) = tokio::net::UnixStream::pair().unwrap();
            let (client, retained) = retain_peer_reader(client).unwrap();
            let executor = OwnedExecutor::default();
            let (mut sender, connection) = hyper::client::conn::http2::Builder::new(executor.clone())
                .handshake::<_, http_body_util::Empty<bytes::Bytes>>(hyper_util::rt::TokioIo::new(client))
                .await.unwrap();
            let driver = tokio::spawn(connection);
            let armed = Arc::new(AtomicBool::new(false));
            let arm = armed.clone();
            let (blocked, blocked_write) = tokio::sync::oneshot::channel();
            let (release, released) = tokio::sync::oneshot::channel();
            let (close, closing) = tokio::sync::oneshot::channel::<()>();
            let server = tokio::spawn(async move {
                let service = hyper::service::service_fn(|_| async {
                    Ok::<_, std::convert::Infallible>(http::Response::new(
                        http_body_util::Full::new(bytes::Bytes::from_static(b"original response")),
                    ))
                });
                let (peer, retained) = retain_peer_reader(peer).unwrap();
                let socket = GatedShutdownWrites {
                    stream: peer, armed, first_write: false,
                    blocked: Some(blocked), release: Some(released),
                };
                let builder = hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new());
                let connection = builder.serve_connection(hyper_util::rt::TokioIo::new(socket), service);
                tokio::pin!(connection);
                tokio::select! {
                    result = connection.as_mut() => panic!("peer departed before shutdown: {result:?}"),
                    result = closing => result.unwrap(),
                }
                arm.store(true, Ordering::Release);
                connection.as_mut().graceful_shutdown();
                connection.await.map_err(WorkerError::Connection)?;
                finish_peer_reader(retained).await.map_err(infrastructure)
            });
            let response = sender.send_request(http::Request::builder()
                .uri("http://localhost/original")
                .body(http_body_util::Empty::new()).unwrap()).await.unwrap();
            assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), "original response");
            close.send(()).unwrap();
            finish_driver(driver).await.unwrap();
            drop(sender);
            let mut failures = Vec::new();
            executor.finish(&mut failures).await;
            assert!(failures.is_empty());
            blocked_write.await.unwrap();
            assert!(!server.is_finished(), "peer final write must cross completed Hyper ownership");
            let mut drain = Box::pin(finish_peer_reader(retained));
            assert!(futures_util::poll!(drain.as_mut()).is_pending(), "peer EOF has not occurred");
            release.send(()).unwrap();
            server.await.unwrap().unwrap();
            drain.await.unwrap();
        }).await.expect("real H2 shutdown must settle every owner");
    }

    #[test]
    fn departed_reply_observer_leaves_actual_typed_failure_with_io_owner() {
        let (reply, observer) = std::sync::mpsc::channel();
        drop(observer);
        let (failed, departed) = deliver_reply(
            reply,
            Err(WorkerError::Process {
                action: "controlled dispatch",
                source: std::io::Error::other("original I/O error"),
            }),
        );
        assert!(failed);
        assert!(
            matches!(departed, Some(WorkerError::Process { action: "controlled dispatch", source }) if source.to_string() == "original I/O error")
        );
    }

    #[test]
    fn checkpoint_after_receipt_preserves_original_failure_and_independent_cancellation() {
        let error = received_outcome(
            Err(WorkerError::Provider(ProviderError::Trial(
                "original science".into(),
            ))),
            Err(ProviderError::Cancelled.into()),
        )
        .unwrap_err();
        let WorkerError::Retirement { causes } = error else {
            panic!("both originals required");
        };
        assert!(
            matches!(&causes[..], [WorkerError::Provider(ProviderError::Trial(detail)), WorkerError::Provider(ProviderError::Cancelled)] if detail == "original science")
        );
    }

    #[tokio::test]
    async fn owned_driver_retains_inner_connection_failure_and_outer_task_failure() {
        use tokio::io::AsyncWriteExt as _;
        let executor = OwnedExecutor::default();
        let (client, mut peer) = tokio::io::duplex(4096);
        let (sender, connection) = hyper::client::conn::http2::Builder::new(executor.clone())
            .handshake::<_, connectrpc::client::ClientBody>(hyper_util::rt::TokioIo::new(client))
            .await
            .unwrap();
        let driver = tokio::spawn(connection);
        peer.write_all(b"not an HTTP2 frame").await.unwrap();
        drop(peer);
        assert!(matches!(
            finish_driver(driver).await,
            Err(WorkerError::Connection(_))
        ));
        drop(sender);
        let mut failures = Vec::new();
        executor.finish(&mut failures).await;
        assert!(failures.is_empty());

        let driver = tokio::spawn(async {
            assert!(std::hint::black_box(false), "controlled outer driver panic");
            Ok::<(), hyper::Error>(())
        });
        assert!(
            matches!(finish_driver(driver).await, Err(WorkerError::IoTask(error)) if error.is_panic())
        );
    }

    #[tokio::test]
    async fn every_owned_hyper_task_joins_after_an_independent_task_panics() {
        let executor = OwnedExecutor::default();
        let completed = Arc::new(AtomicBool::new(false));
        hyper::rt::Executor::execute(&executor, async {
            panic!("controlled H2 executor panic");
        });
        let signal = completed.clone();
        hyper::rt::Executor::execute(&executor, async move {
            signal.store(true, Ordering::Release);
        });
        let mut failures = Vec::new();
        executor.finish(&mut failures).await;
        assert!(completed.load(Ordering::Acquire));
        assert!(matches!(&failures[..], [WorkerError::IoTask(error)] if error.is_panic()));
    }

    fn handle(generation: u8) -> p::Handle {
        p::Handle {
            attempt: SemanticId::from_bytes([1; 16]),
            session: SemanticId::from_bytes([2; 16]),
            generation: SemanticId::from_bytes([generation; 16]),
        }
    }

    fn preparation(handle: p::Handle) -> p::Request {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let descriptor = pse_backend_native::coolprop::descriptor(
            &registry,
            pse_backend_native::coolprop::WaterRegime::Liquid,
        )
        .unwrap();
        p::Request::Prepare(p::Prepare {
            version: 2,
            handle,
            spec: pse_kernels::portable::ProviderRecord::capture(&descriptor),
            configuration: pse_ids::ContentHash::from_bytes([1; 32]),
            regime: "liquid".into(),
            outputs: vec![0],
            order: 0,
            demand: p::Demand::CompleteJets,
            remaining_ns: None,
            result_limit: 4096,
            physical: handler::physical_identity(&registry).unwrap(),
            registry: registry.to_builder(),
        })
    }

    fn completion(
        work: Option<tokio::sync::OwnedSemaphorePermit>,
    ) -> (NativeCompletion, Submitted) {
        let (response, received) = tokio::sync::oneshot::channel();
        (
            NativeCompletion {
                response,
                _work: work,
            },
            Submitted { response: received },
        )
    }

    fn server() -> (Server, tokio::sync::mpsc::Receiver<Job>) {
        let (jobs, receiving) = tokio::sync::mpsc::channel(1);
        (
            Server {
                jobs,
                controls: SharedControls::default(),
                work: Arc::new(tokio::sync::Semaphore::new(1)),
                terminal: tokio::sync::watch::channel(None).1,
            },
            receiving,
        )
    }

    fn fill_queue(server: &Server) {
        let (completion, _observer) = completion(None);
        assert!(
            server
                .jobs
                .try_send(Job {
                    request: p::Request::Shutdown,
                    completion
                })
                .is_ok()
        );
    }

    #[tokio::test]
    async fn terminal_readers_wait_for_join_and_retain_shared_bytes_after_departure() {
        use buffa::Message as _;
        use connectrpc::Encodable as _;
        let (mut server, receiving) = server();
        drop(receiving);
        let (publisher, reader) = tokio::sync::watch::channel(None);
        server.terminal = reader;
        assert!(matches!(
            server.submit(p::Request::Shutdown, None).await,
            Err(SubmissionError::Departed)
        ));
        let mut departed = Box::pin(server.terminal_response());
        assert!(futures_util::poll!(&mut departed).is_pending());
        drop(departed);
        let mut first = Box::pin(server.terminal_response());
        let mut second = Box::pin(server.terminal_response());
        assert!(futures_util::poll!(&mut first).is_pending());
        assert!(futures_util::poll!(&mut second).is_pending());
        publisher.send_replace(Some(
            encode_response(p::Response::Failure(p::Failure::Contract(
                "joined immutable facts".into(),
            )))
            .unwrap(),
        ));
        let first = first
            .await
            .unwrap()
            .encode(connectrpc::CodecFormat::Proto)
            .unwrap();
        let second = second
            .await
            .unwrap()
            .encode(connectrpc::CodecFormat::Proto)
            .unwrap();
        assert_eq!(
            first.as_ptr(),
            second.as_ptr(),
            "terminal readers share one immutable byte owner"
        );
        drop(publisher);
        drop(server);
        drop(first);
        let response =
            p::Response::try_from(pb::Response::decode_from_slice(&second).unwrap()).unwrap();
        assert!(
            matches!(response, p::Response::Failure(p::Failure::Contract(detail)) if detail == "joined immutable facts")
        );
    }

    #[test]
    fn abandoned_observer_does_not_release_a_native_job_admission() {
        let lane = Arc::new(tokio::sync::Semaphore::new(1));
        let permit = lane.clone().try_acquire_owned().unwrap();
        let (completion, observer) = completion(Some(permit));
        let job = Job {
            request: p::Request::Shutdown,
            completion,
        };
        drop(observer);
        assert!(lane.clone().try_acquire_owned().is_err());
        drop(job);
        assert!(lane.try_acquire_owned().is_ok());
    }

    #[tokio::test]
    async fn canceled_queue_reservation_never_commits_preparation_or_leaks_admission() {
        let (server, mut receiving) = server();
        fill_queue(&server);
        let work = server.work.clone().try_acquire_owned().unwrap();
        let mut waiting = Box::pin(server.submit(preparation(handle(3)), Some(work)));
        assert!(futures_util::poll!(waiting.as_mut()).is_pending());
        assert!(controls_lock(&server.controls).unwrap().handles.is_empty());
        assert_eq!(server.work.available_permits(), 0);
        drop(waiting);
        assert_eq!(server.work.available_permits(), 1);
        drop(receiving.recv().await.unwrap());
        assert!(matches!(
            receiving.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
        ));
        assert!(controls_lock(&server.controls).unwrap().handles.is_empty());
    }

    #[tokio::test]
    async fn departed_native_queue_refuses_before_preparation_commit() {
        let (server, receiving) = server();
        drop(receiving);
        let work = server.work.clone().try_acquire_owned().unwrap();
        assert!(matches!(
            server.submit(preparation(handle(3)), Some(work)).await,
            Err(SubmissionError::Departed)
        ));
        assert!(controls_lock(&server.controls).unwrap().handles.is_empty());
        assert_eq!(server.work.available_permits(), 1);
    }

    #[tokio::test]
    async fn lost_preparation_observer_retires_only_its_handle_before_releasing_admission() {
        let (server, mut receiving) = server();
        let survivor = handle(4);
        let cancellation = pse_diagnostics::Cancellation::default();
        controls_lock(&server.controls).unwrap().handles.insert(
            survivor.generation,
            (survivor.clone(), cancellation.clone()),
        );
        let orphan = handle(3);
        let work = server.work.clone().try_acquire_owned().unwrap();
        let observer = server
            .submit(preparation(orphan.clone()), Some(work))
            .await
            .unwrap();
        let job = receiving.recv().await.unwrap();
        assert!(
            controls_lock(&server.controls)
                .unwrap()
                .control(&orphan)
                .is_ok()
        );
        drop(observer);
        let mut retired = false;
        job.completion
            .finish(
                p::Response::Prepared(p::Prepared {
                    handle: orphan.clone(),
                    batch_supported: false,
                }),
                Some(orphan.clone()),
                |actual| {
                    assert_eq!(actual, orphan);
                    assert_eq!(server.work.available_permits(), 0);
                    controls_lock(&server.controls)?
                        .handles
                        .remove(&actual.generation);
                    retired = true;
                    Ok(())
                },
            )
            .unwrap();
        assert!(retired);
        assert_eq!(server.work.available_permits(), 1);
        let controls = controls_lock(&server.controls).unwrap();
        assert!(controls.control(&orphan).is_err());
        assert!(controls.control(&survivor).is_ok());
        assert!(!cancellation.is_cancelled());
    }

    #[tokio::test]
    async fn prepared_reply_departure_before_or_after_poll_returns_admission_for_retirement() {
        for polled in [false, true] {
            let (server, mut receiving) = server();
            let orphan = handle(3);
            let survivor = handle(4);
            let cancellation = pse_diagnostics::Cancellation::default();
            controls_lock(&server.controls).unwrap().handles.insert(
                survivor.generation,
                (survivor.clone(), cancellation.clone()),
            );
            let work = server.work.clone().try_acquire_owned().unwrap();
            let observer = server
                .submit(preparation(orphan.clone()), Some(work))
                .await
                .unwrap();
            let job = receiving.recv().await.unwrap();
            // The response send succeeds while the observing method still exists.
            // Neither the unpolled payload nor a polled-but-unconsumed payload
            // establishes local method consumption.
            let observation = job.completion.deliver(p::Response::Prepared(p::Prepared {
                handle: orphan.clone(),
                batch_supported: false,
            }));
            assert!(matches!(
                observation.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ));
            if polled {
                let reply = observer.response.await.unwrap();
                assert!(
                    matches!(&reply.response, p::Response::Prepared(value) if value.handle == orphan)
                );
                drop(reply);
            } else {
                drop(observer);
            }
            assert_eq!(server.work.available_permits(), 0);
            let mut retired = false;
            settle_delivery(observation, Some(orphan.clone()), |actual| {
                assert_eq!(actual, orphan);
                assert_eq!(server.work.available_permits(), 0);
                controls_lock(&server.controls)?
                    .handles
                    .remove(&actual.generation);
                retired = true;
                Ok(())
            })
            .unwrap();
            assert!(retired);
            assert_eq!(server.work.available_permits(), 1);
            let controls = controls_lock(&server.controls).unwrap();
            assert!(controls.control(&orphan).is_err());
            assert!(controls.control(&survivor).is_ok());
            assert!(!cancellation.is_cancelled());
        }
    }

    #[tokio::test]
    async fn consumed_preparation_does_not_treat_later_wire_loss_as_method_departure() {
        let (server, mut receiving) = server();
        let prepared = handle(3);
        let work = server.work.clone().try_acquire_owned().unwrap();
        let observer = server
            .submit(preparation(prepared.clone()), Some(work))
            .await
            .unwrap();
        let job = receiving.recv().await.unwrap();
        let observation = job.completion.deliver(p::Response::Prepared(p::Prepared {
            handle: prepared.clone(),
            batch_supported: false,
        }));
        let reply = observer.response.await.unwrap();
        let encoded = encode_response(reply.response).unwrap();
        reply.delivery.consume();
        assert_eq!(server.work.available_permits(), 1);
        // No network delivery is claimed. Losing these bytes cannot manufacture
        // a second native outcome or retire an already consumed preparation.
        drop(encoded);
        settle_delivery(observation, Some(prepared.clone()), |_| {
            panic!("wire delivery uncertainty is not local method abandonment")
        })
        .unwrap();
        assert!(
            controls_lock(&server.controls)
                .unwrap()
                .control(&prepared)
                .is_ok()
        );
    }

    #[test]
    fn transport_runtime_drops_pending_shutdown_method_before_native_join() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (completion, observer) = completion(None);
        let drained = Arc::new(AtomicBool::new(false));
        let native_drained = drained.clone();
        let native = std::thread::spawn(move || {
            native_drained.store(true, Ordering::Release);
            completion
                .finish(p::Response::Shutdown, None, |_| {
                    panic!("shutdown has no orphan preparation")
                })
                .unwrap();
        });
        runtime.block_on(async {
            let reply = observer.response.await.unwrap();
            assert!(matches!(&reply.response, p::Response::Shutdown));
            tokio::spawn(async move {
                let _unconsumed = reply;
                std::future::pending::<()>().await;
            });
            tokio::task::yield_now().await;
        });
        assert!(drained.load(Ordering::Acquire));
        assert!(!native.is_finished());
        join_native(runtime, native).unwrap();
    }

    #[tokio::test]
    async fn stop_and_shutdown_signal_while_queue_capacity_is_unavailable() {
        for shutdown in [false, true] {
            let (server, mut receiving) = server();
            let first = handle(3);
            let second = handle(4);
            let a = pse_diagnostics::Cancellation::default();
            let b = pse_diagnostics::Cancellation::default();
            {
                let mut controls = controls_lock(&server.controls).unwrap();
                controls
                    .handles
                    .insert(first.generation, (first.clone(), a.clone()));
                controls
                    .handles
                    .insert(second.generation, (second, b.clone()));
            }
            fill_queue(&server);
            let _native_work = server.work.clone().try_acquire_owned().unwrap();
            let request = if shutdown {
                p::Request::Shutdown
            } else {
                p::Request::Stop(first)
            };
            let mut waiting = Box::pin(server.submit(request, None));
            assert!(futures_util::poll!(waiting.as_mut()).is_pending());
            assert!(a.is_cancelled());
            assert_eq!(b.is_cancelled(), shutdown);
            assert!(!controls_lock(&server.controls).unwrap().closed);
            drop(receiving.recv().await.unwrap());
            let _observer = waiting.await.unwrap();
            assert_eq!(controls_lock(&server.controls).unwrap().closed, shutdown);
            assert!(receiving.recv().await.is_some());
        }
    }

    #[tokio::test]
    async fn lost_evaluation_observer_does_not_retire_handles_or_close_native_queue() {
        let (server, mut receiving) = server();
        let first = handle(3);
        let survivor = handle(4);
        for handle in [&first, &survivor] {
            controls_lock(&server.controls)
                .unwrap()
                .handles
                .insert(handle.generation, (handle.clone(), Default::default()));
        }
        let work = server.work.clone().try_acquire_owned().unwrap();
        let observer = server
            .submit(
                p::Request::Evaluate(p::Evaluate {
                    handle: first.clone(),
                    invocation: 1,
                    inputs: vec![1.],
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }),
                Some(work),
            )
            .await
            .unwrap();
        let job = receiving.recv().await.unwrap();
        drop(observer);
        job.completion
            .finish(
                p::Response::Failure(p::Failure::Trial("original refusal".into())),
                None,
                |_| panic!("lost evaluation cannot retire a preparation"),
            )
            .unwrap();
        assert_eq!(server.work.available_permits(), 1);
        let next = server
            .submit(p::Request::Close(survivor.clone()), None)
            .await
            .unwrap();
        let job = receiving.recv().await.unwrap();
        let observed = job
            .completion
            .deliver(p::Response::Closed(survivor.clone()));
        let reply = next.response.await.unwrap();
        assert!(matches!(reply.response, p::Response::Closed(actual) if actual == survivor));
        reply.delivery.consume();
        settle_delivery(observed, None, |_| {
            panic!("delivered close cannot retire another preparation")
        })
        .unwrap();
        let controls = controls_lock(&server.controls).unwrap();
        assert!(!controls.closed);
        assert!(controls.control(&first).is_ok());
        assert!(controls.control(&survivor).is_ok());
    }

    #[test]
    fn handle_controls_cancel_only_exact_context_and_shutdown_closes_admission() {
        let first = p::Handle {
            attempt: SemanticId::from_bytes([1; 16]),
            session: SemanticId::from_bytes([2; 16]),
            generation: SemanticId::from_bytes([3; 16]),
        };
        let mut second = first.clone();
        second.generation = SemanticId::from_bytes([4; 16]);
        let a = pse_diagnostics::Cancellation::default();
        let b = pse_diagnostics::Cancellation::default();
        let mut controls = HandleControls::default();
        controls
            .handles
            .insert(first.generation, (first.clone(), a.clone()));
        controls
            .handles
            .insert(second.generation, (second.clone(), b.clone()));
        let mut foreign = first.clone();
        foreign.attempt = SemanticId::NIL;
        assert!(controls.admit(&p::Request::Stop(foreign)).is_err());
        assert!(!a.is_cancelled());
        assert!(!b.is_cancelled());
        controls.admit(&p::Request::Stop(first)).unwrap();
        assert!(a.is_cancelled());
        assert!(!b.is_cancelled());
        assert!(!controls.closed);
        controls.admit(&p::Request::Close(second.clone())).unwrap();
        assert!(!b.is_cancelled());
        controls.admit(&p::Request::Shutdown).unwrap();
        assert!(b.is_cancelled());
        assert!(controls.closed);
        assert!(controls.admit(&p::Request::Close(second)).is_err());
    }

    #[test]
    fn evaluation_options_leave_original_deadline_with_scientific_scope() {
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
            None
        );
        assert_eq!(
            call_options(rpc_timeout(&original, true, now + Duration::from_secs(601)).unwrap())
                .timeout(),
            None
        );
        assert_eq!(original.deadline(), Some(now + Duration::from_secs(1800)));
        assert!(matches!(
            rpc_timeout(&original, true, now + Duration::from_secs(1801)),
            Err(WorkerError::Provider(ProviderError::Deadline))
        ));
        assert!(!control.flag().load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn owned_connection_refuses_wrong_kernel_peer() {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = directory.path().join("peer.sock");
        let _listener = tokio::net::UnixListener::bind(&endpoint).unwrap();
        assert!(
            matches!(authenticated_stream(&endpoint, u32::MAX).await, Err(WorkerError::Process { action: "authenticate", source }) if source.kind() == std::io::ErrorKind::PermissionDenied)
        );
        drop(
            authenticated_stream(&endpoint, std::process::id())
                .await
                .unwrap(),
        );
    }
}
