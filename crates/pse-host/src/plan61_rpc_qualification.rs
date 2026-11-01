// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualification only: released transport plus real CPU ownership, without a HIR workspace.
use crate::plan61_qualification::pb::plan61::qualification::v1 as pb;
use crate::plan61_qualification::services as bindings;
use connectrpc::{ConnectError, ServiceStream};
use futures_util::StreamExt as _;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::sync::{Notify, mpsc};
use tokio_util::sync::CancellationToken;

struct Implementation(Arc<Session>);
#[protovalidate_buffa::connect_impl]
impl bindings::Investigation for Implementation {
    async fn inspect<'a>(
        &'a self,
        context: connectrpc::RequestContext,
        request: connectrpc::ServiceRequest<'_, pb::Request>,
    ) -> connectrpc::ServiceResult<impl connectrpc::Encodable<pb::Observation> + Send + use<'a>>
    {
        assert_eq!(
            context.extensions().get::<u32>(),
            Some(&rustix::process::geteuid().as_raw())
        );
        Ok(connectrpc::Response::new(pb::Observation {
            value: request.view().query.to_string(),
            sequence: 1,
            ..Default::default()
        }))
    }
    async fn run(
        &self,
        _: connectrpc::RequestContext,
        requests: connectrpc::InboundStream<pb::Control>,
    ) -> connectrpc::ServiceResult<
        ServiceStream<impl connectrpc::Encodable<pb::Event> + Send + use<>>,
    > {
        Ok(connectrpc::Response::new(self.0.run(Box::pin(
            requests.map(|item| item.map(|value| value.to_owned_message())),
        ))))
    }
}

#[derive(Default)]
struct Gate {
    released: Mutex<bool>,
    condition: Condvar,
    started: Notify,
    cancelled: Notify,
    completed: AtomicBool,
}
impl Gate {
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.condition.notify_all();
    }
}

#[derive(Default)]
struct Session {
    stop: CancellationToken,
    workers: Mutex<Vec<tokio::task::JoinHandle<()>>>,
    gates: Mutex<Vec<Arc<Gate>>>,
}
impl Session {
    async fn shutdown(&self) {
        self.stop.cancel();
        let workers = std::mem::take(&mut *self.workers.lock().unwrap());
        for worker in workers {
            worker.await.unwrap();
        }
    }

    fn run(self: &Arc<Self>, mut requests: ServiceStream<pb::Control>) -> ServiceStream<pb::Event> {
        let mut workers = self.workers.lock().unwrap();
        if self.stop.is_cancelled() {
            return Box::pin(futures_util::stream::once(async {
                Err(ConnectError::unavailable("session stopped"))
            }));
        }
        let (tx, rx) = mpsc::channel(1);
        let session = self.clone();
        let cancellation = self.stop.child_token();
        let guard = CancelOnDrop(cancellation.clone());
        let worker = tokio::spawn(async move {
            let first = tokio::select! {
                first = requests.next() => first,
                _ = cancellation.cancelled() => return,
            };
            let Some(Ok(first)) = first else {
                return;
            };
            let slow = match first.action {
                Some(pb::control::Action::Start(request)) => {
                    use protovalidate_buffa::Validate as _;
                    if let Err(error) = request.validate() {
                        let _ = tx.send(Err(error.into_connect_error())).await;
                        return;
                    }
                    request.query == "slow"
                }
                _ => {
                    let _ = tx
                        .send(Err(ConnectError::invalid_argument("Start must be first")))
                        .await;
                    return;
                }
            };
            if session.stop.is_cancelled() {
                let _ = tx
                    .send(Err(ConnectError::unavailable("session stopped")))
                    .await;
                return;
            }
            let gate = Arc::new(Gate::default());
            session.gates.lock().unwrap().push(gate.clone());
            let work_gate = gate.clone();
            let cancel_gate = gate.clone();
            let mut task =
                crate::source_command::batch::spawn_cpu(cancellation.clone(), move |token| {
                    work_gate.started.notify_one();
                    let mut released = work_gate.released.lock().unwrap();
                    while !*released {
                        released = work_gate.condition.wait(released).unwrap();
                    }
                    work_gate.completed.store(true, Ordering::SeqCst);
                    token.is_cancelled()
                })
                .with_cancel_hook(move || cancel_gate.cancelled.notify_one());
            if slow {
                let _ = tx
                    .send(Ok(pb::Event {
                        item: Some(pb::event::Item::Observation(Box::new(pb::Observation {
                            value: "admitted".into(),
                            sequence: 1,
                            ..Default::default()
                        }))),
                        ..Default::default()
                    }))
                    .await;
            }
            let mut input_open = true;
            let mut invalid = false;
            let mut cancelling = false;
            let cancelled = loop {
                tokio::select! {
                    result = task.settle() => break result.unwrap(),
                    _ = cancellation.cancelled(), if !cancelling => { task.cancel(); cancelling = true; }
                    next = requests.next(), if input_open => match next {
                        None => input_open = false,
                        Some(Ok(control)) if matches!(control.action, Some(pb::control::Action::Cancel(_))) => {
                            if !cancelling { task.cancel(); cancelling = true; }
                        }
                        Some(_) => { invalid = true; task.cancel(); cancelling = true; input_open = false; }
                    }
                }
            };
            let result = if invalid {
                Err(ConnectError::invalid_argument(
                    "only Cancel may follow Start",
                ))
            } else {
                Ok(pb::Event {
                    item: Some(pb::event::Item::Terminal(Box::new(pb::Terminal {
                        completed: true,
                        cancelled,
                        ..Default::default()
                    }))),
                    ..Default::default()
                })
            };
            if let Err(mpsc::error::TrySendError::Full(result)) = tx.try_send(result) {
                tokio::select! {
                    _ = tx.send(result) => {},
                    _ = session.stop.cancelled() => {},
                }
            }
        });
        workers.push(worker);
        Box::pin(futures_util::stream::unfold(
            (rx, guard),
            |(mut rx, guard)| async move { rx.recv().await.map(|item| (item, (rx, guard))) },
        ))
    }
}
struct CancelOnDrop(CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

fn start() -> pb::Control {
    pb::Control {
        action: Some(pb::control::Action::Start(Box::new(pb::Request {
            query: "source declaration".into(),
            ..Default::default()
        }))),
        ..Default::default()
    }
}
fn cancel() -> pb::Control {
    pb::Control {
        action: Some(pb::control::Action::Cancel(Default::default())),
        ..Default::default()
    }
}
struct GateRelease(Arc<Gate>);
impl std::ops::Deref for GateRelease {
    type Target = Gate;
    fn deref(&self) -> &Gate {
        &self.0
    }
}
impl Drop for GateRelease {
    fn drop(&mut self) {
        self.0.release();
    }
}
async fn gate(session: &Session, index: usize) -> GateRelease {
    // Cooperative scheduling only; timeout makes a failed admission observable.
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let found = session.gates.lock().unwrap().get(index).cloned();
            if let Some(gate) = found {
                let gate = GateRelease(gate);
                gate.started.notified().await;
                return gate;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}

#[derive(Clone)]
struct AuthenticatedConnector {
    socket: std::path::PathBuf,
    expected_pid: u32,
    authentications: Arc<AtomicUsize>,
}
impl tower_service::Service<http::Uri> for AuthenticatedConnector {
    type Response = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Error = std::io::Error;
    type Future =
        std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: http::Uri) -> Self::Future {
        let connector = self.clone();
        Box::pin(async move {
            let stream = tokio::net::UnixStream::connect(connector.socket).await?;
            let peer = stream.peer_cred()?;
            connector.authentications.fetch_add(1, Ordering::SeqCst);
            // The production connector must call authenticate_payload_peer here.
            // This fixture checks the real kernel credential, not a header or path.
            if peer.pid().and_then(|pid| u32::try_from(pid).ok()) != Some(connector.expected_pid) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "original helper custody differs",
                ));
            }
            Ok(hyper_util::rt::TokioIo::new(stream))
        })
    }
}

struct Server {
    _directory: tempfile::TempDir,
    connector: AuthenticatedConnector,
    stop: CancellationToken,
    task: tokio::task::JoinHandle<()>,
    session: Arc<Session>,
}
impl Server {
    async fn start() -> Self {
        use tower_service::Service as _;
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("analysis.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let session = Arc::new(Session::default());
        use bindings::InvestigationExt as _;
        let router = Arc::new(Implementation(session.clone())).register(connectrpc::Router::new());
        let service = connectrpc::ConnectRpcService::new(router);
        let stop = CancellationToken::new();
        let stopping = stop.clone();
        let task_session = session.clone();
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                let (stream, _) = tokio::select! {
                    result = listener.accept() => result.unwrap(),
                    _ = stopping.cancelled() => break,
                };
                let uid = stream.peer_cred().unwrap().uid();
                assert_eq!(uid, rustix::process::geteuid().as_raw());
                let service = service.clone();
                connections.spawn(async move {
                    let hyper_service = hyper::service::service_fn(
                        move |mut request: http::Request<hyper::body::Incoming>| {
                            let mut service = service.clone();
                            request.extensions_mut().insert(uid);
                            service.call(request)
                        },
                    );
                    let _ = hyper::server::conn::http2::Builder::new(
                        hyper_util::rt::TokioExecutor::new(),
                    )
                    .serve_connection(hyper_util::rt::TokioIo::new(stream), hyper_service)
                    .await;
                });
            }
            task_session.shutdown().await;
            connections.abort_all();
            while connections.join_next().await.is_some() {}
        });
        Self {
            _directory: directory,
            connector: AuthenticatedConnector {
                socket,
                expected_pid: std::process::id(),
                authentications: Arc::new(AtomicUsize::new(0)),
            },
            stop,
            task,
            session,
        }
    }
    async fn connection(&self) -> connectrpc::client::SharedHttp2Connection {
        connectrpc::client::Http2Connection::connect_with_connector(
            self.connector.clone(),
            "http://localhost".parse().unwrap(),
        )
        .await
        .unwrap()
        .shared(32)
    }
    async fn shutdown(self) {
        self.stop.cancel();
        self.task.await.unwrap();
    }
}

fn config() -> connectrpc::client::ClientConfig {
    connectrpc::client::ClientConfig::new("http://localhost".parse().unwrap())
        .with_protocol(connectrpc::Protocol::Grpc)
}

#[tokio::test]
async fn plan61_uds_kernel_authentication_precedes_handshake_on_each_connection() {
    let server = Server::start().await;
    for _ in 0..2 {
        let connection = server.connection().await;
        let client = bindings::InvestigationClient::new(connection, config());
        let result = client
            .inspect(pb::Request {
                query: "source declaration".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.into_owned().value, "source declaration");
    }
    assert_eq!(server.connector.authentications.load(Ordering::SeqCst), 2);
    let mut wrong_custody = server.connector.clone();
    wrong_custody.expected_pid += 1;
    assert!(
        connectrpc::client::Http2Connection::connect_with_connector(
            wrong_custody,
            "http://localhost".parse().unwrap()
        )
        .await
        .is_err()
    );
    server.shutdown().await;
}

#[tokio::test]
async fn plan61_grpc_half_close_and_reset_preserve_real_cpu_owner() {
    let server = Server::start().await;
    let connection = server.connection().await;
    let client = bindings::InvestigationClient::new(connection, config());
    let mut conversation = client.run().await.unwrap();
    conversation.send(start()).await.unwrap();
    let first = gate(&server.session, 0).await;
    conversation.close_send();
    first.release();
    let terminal = conversation
        .message::<pb::Event>()
        .await
        .unwrap()
        .unwrap()
        .to_owned_message();
    assert!(
        matches!(terminal.item, Some(pb::event::Item::Terminal(t)) if t.completed && !t.cancelled)
    );
    assert!(conversation.message::<pb::Event>().await.unwrap().is_none());

    let mut lost = client.run().await.unwrap();
    lost.send(start()).await.unwrap();
    let second = gate(&server.session, 1).await;
    drop(lost);
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        second.cancelled.notified(),
    )
    .await
    .unwrap();
    assert!(!second.completed.load(Ordering::SeqCst));
    second.release();
    server.shutdown().await;
    assert!(second.completed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn plan61_generated_rpc_validates_unary_view_and_start_before_admission() {
    let server = Server::start().await;
    let client = bindings::InvestigationClient::new(server.connection().await, config());
    let failure = client.inspect(pb::Request::default()).await.unwrap_err();
    assert_eq!(failure.code, connectrpc::ErrorCode::InvalidArgument);
    let mut stream = client.run().await.unwrap();
    stream
        .send(pb::Control {
            action: Some(pb::control::Action::Start(Box::default())),
            ..Default::default()
        })
        .await
        .unwrap();
    let failure = stream.message::<pb::Event>().await.unwrap_err();
    assert_eq!(failure.code, connectrpc::ErrorCode::InvalidArgument);
    assert!(server.session.gates.lock().unwrap().is_empty());
    server.shutdown().await;
}

#[tokio::test]
async fn plan61_grpc_deadline_leaves_completion_unobserved_and_cpu_owned() {
    let server = Server::start().await;
    let client = bindings::InvestigationClient::new(server.connection().await, config());
    let mut stream = client
        .run_with_options(
            connectrpc::client::CallOptions::default()
                .with_timeout(std::time::Duration::from_millis(500)),
        )
        .await
        .unwrap();
    stream.send(start()).await.unwrap();
    let gate = gate(&server.session, 0).await;
    let failure = stream.message::<pb::Event>().await.unwrap_err();
    assert_eq!(failure.code, connectrpc::ErrorCode::DeadlineExceeded);
    drop(stream);
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        gate.cancelled.notified(),
    )
    .await
    .unwrap();
    assert!(!gate.completed.load(Ordering::SeqCst));
    gate.release();
    server.shutdown().await;
    assert!(gate.completed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn plan61_slow_output_cannot_hold_session_shutdown_after_cpu_return() {
    let session = Arc::new(Session::default());
    let (tx, requests) = controls();
    let _unpolled = session.run(requests);
    tx.send(Ok(pb::Control {
        action: Some(pb::control::Action::Start(Box::new(pb::Request {
            query: "slow".into(),
            ..Default::default()
        }))),
        ..Default::default()
    }))
    .await
    .unwrap();
    let gate = gate(&session, 0).await;
    gate.release();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !gate.completed.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
        session.shutdown().await;
    })
    .await
    .unwrap();
}
fn controls() -> (
    mpsc::Sender<Result<pb::Control, ConnectError>>,
    ServiceStream<pb::Control>,
) {
    let (tx, rx) = mpsc::channel(1);
    (
        tx,
        Box::pin(futures_util::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        })),
    )
}

#[tokio::test]
async fn plan61_half_close_is_not_cancel_and_terminal_follows_cpu_completion() {
    let session = Arc::new(Session::default());
    let (tx, requests) = controls();
    let mut output = session.run(requests);
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&session, 0).await;
    drop(tx);
    assert!(!gate.completed.load(Ordering::SeqCst));
    gate.release();
    let terminal = output.next().await.unwrap().unwrap();
    assert!(
        matches!(terminal.item, Some(pb::event::Item::Terminal(t)) if t.completed && !t.cancelled)
    );
    assert!(gate.completed.load(Ordering::SeqCst));
    assert!(output.next().await.is_none());
    session.shutdown().await;
}

#[tokio::test]
async fn plan61_dropped_output_retains_cpu_owner_and_does_not_cancel_sibling() {
    let session = Arc::new(Session::default());
    let (tx, requests) = controls();
    let output = session.run(requests);
    tx.send(Ok(start())).await.unwrap();
    let first = gate(&session, 0).await;
    let (sibling_tx, requests) = controls();
    let mut sibling = session.run(requests);
    sibling_tx.send(Ok(start())).await.unwrap();
    let second = gate(&session, 1).await;
    drop(output);
    first.cancelled.notified().await;
    assert!(!first.completed.load(Ordering::SeqCst));
    first.release();
    second.release();
    let terminal = sibling.next().await.unwrap().unwrap();
    assert!(matches!(terminal.item, Some(pb::event::Item::Terminal(t)) if !t.cancelled));
    session.shutdown().await;
    assert!(first.completed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn plan61_cancel_and_shutdown_wait_for_actual_cpu_return() {
    for explicit in [true, false] {
        let session = Arc::new(Session::default());
        let (tx, requests) = controls();
        let mut output = session.run(requests);
        tx.send(Ok(start())).await.unwrap();
        let gate = gate(&session, 0).await;
        if explicit {
            tx.send(Ok(cancel())).await.unwrap();
        } else {
            session.stop.cancel();
        }
        gate.cancelled.notified().await;
        assert!(!gate.completed.load(Ordering::SeqCst));
        gate.release();
        let terminal = output.next().await.unwrap().unwrap();
        assert!(
            matches!(terminal.item, Some(pb::event::Item::Terminal(t)) if t.completed && t.cancelled)
        );
        session.shutdown().await;
    }
}

#[tokio::test]
async fn plan61_repeated_start_is_invalid_but_still_settles_cpu() {
    let session = Arc::new(Session::default());
    let (tx, requests) = controls();
    let mut output = session.run(requests);
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&session, 0).await;
    tx.send(Ok(start())).await.unwrap();
    gate.cancelled.notified().await;
    gate.release();
    assert!(output.next().await.unwrap().is_err());
    assert!(gate.completed.load(Ordering::SeqCst));
    session.shutdown().await;
}

#[tokio::test]
async fn plan61_repeated_cancel_is_idempotent_and_terminal_is_unique() {
    let session = Arc::new(Session::default());
    let (tx, requests) = controls();
    let mut output = session.run(requests);
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&session, 0).await;
    tx.send(Ok(cancel())).await.unwrap();
    gate.cancelled.notified().await;
    tx.send(Ok(cancel())).await.unwrap();
    drop(tx);
    gate.release();
    let terminal = output.next().await.unwrap().unwrap();
    assert!(
        matches!(terminal.item, Some(pb::event::Item::Terminal(t)) if t.completed && t.cancelled)
    );
    assert!(output.next().await.is_none());
    session.shutdown().await;
}
