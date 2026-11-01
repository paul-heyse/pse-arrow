// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Production retention and HTTP2 hosting with controlled actual Rayon work.
use crate::{
    code_intelligence_contracts::{pb::pse::code_intelligence::v1 as pb, services as bindings},
    semantic::{self, rpc},
};
use connectrpc::{ConnectError, ServiceStream};
use futures_util::StreamExt as _;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Notify, mpsc};
use tokio_util::sync::CancellationToken;

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
struct Release(Arc<Gate>);
impl std::ops::Deref for Release {
    type Target = Gate;
    fn deref(&self) -> &Gate {
        &self.0
    }
}
impl Drop for Release {
    fn drop(&mut self) {
        self.0.release();
    }
}
#[derive(Default)]
pub(crate) struct Fixture {
    gates: Mutex<Vec<Arc<Gate>>>,
}
impl Fixture {
    pub(crate) fn status(&self) -> semantic::SessionStatus {
        semantic::SessionStatus {
            configuration: semantic::Configuration {
                backend: semantic::Backend::NativeSnapshot,
                project: "/fixture".into(),
                features: vec![],
                no_default_features: false,
                target: None,
                build_macros: false,
            },
            backend: "controlled real CPU operation".into(),
            position_encoding: "utf-8".into(),
            generation: 1,
            server_capabilities: serde_json::json!({}),
            backend_status: None,
            active_requests: self
                .gates
                .lock()
                .unwrap()
                .iter()
                .filter(|gate| !gate.completed.load(Ordering::SeqCst))
                .count(),
            context_current: true,
            context_issue: None,
            automatic_reload_disabled: true,
            preparation_issue: None,
            preparation: "test-controlled execution".into(),
        }
    }
    pub(crate) fn start(
        &self,
        _: semantic::Request,
        token: CancellationToken,
    ) -> crate::source_command::batch::CpuTask<semantic::Result<semantic::Response>> {
        let gate = Arc::new(Gate::default());
        self.gates.lock().unwrap().push(gate.clone());
        let work = gate.clone();
        let configuration = self.status().configuration;
        crate::source_command::batch::spawn_cpu(token, move |token| {
            work.started.notify_one();
            let mut released = work.released.lock().unwrap();
            while !*released {
                released = work.condition.wait(released).unwrap();
            }
            work.completed.store(true, Ordering::SeqCst);
            if token.is_cancelled() {
                return Err(semantic::Error::Cancelled);
            }
            Ok(semantic::Response::WorkspaceSymbols(
                semantic::WorkspaceAnswer {
                    session: semantic::AnalysisContext {
                        backend: "controlled CPU".into(),
                        configuration,
                        position_encoding: "utf-8".into(),
                        backend_status: None,
                        preparation: "test controlled".into(),
                    },
                    generation: 1,
                    symbols: semantic::Section {
                        category: "workspace_symbols".into(),
                        evidence: serde_json::json!([]),
                        omitted: 0,
                        handles: vec![],
                    },
                    observed: vec![],
                    limitations: vec![],
                },
            ))
        })
        .with_cancel_hook(move || gate.cancelled.notify_one())
    }
}
async fn gate(fixture: &Fixture, index: usize) -> Release {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let found = fixture.gates.lock().unwrap().get(index).cloned();
            if let Some(gate) = found {
                let gate = Release(gate);
                gate.started.notified().await;
                return gate;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}
fn session() -> (Arc<rpc::Session>, Arc<Fixture>, CancellationToken) {
    let fixture = Arc::new(Fixture::default());
    let stop = CancellationToken::new();
    let session = Arc::new(rpc::Session::fixture(fixture.clone(), stop.clone()));
    (session, fixture, stop)
}
fn start() -> pb::NativeControl {
    pb::NativeControl {
        action: Some(pb::native_control::Action::Start(Box::new(
            pb::NativeRequest {
                operation: Some(pb::native_request::Operation::Workspace(Box::new(
                    pb::NativeWorkspaceQuery {
                        query: "target".into(),
                        limit: 1,
                        ..Default::default()
                    },
                ))),
                ..Default::default()
            },
        ))),
        ..Default::default()
    }
}
fn cancel() -> pb::NativeControl {
    pb::NativeControl {
        action: Some(pb::native_control::Action::Cancel(Default::default())),
        ..Default::default()
    }
}
fn controls() -> (
    mpsc::Sender<Result<pb::NativeControl, ConnectError>>,
    ServiceStream<pb::NativeControl>,
) {
    let (tx, rx) = mpsc::channel(2);
    (
        tx,
        Box::pin(futures_util::stream::unfold(rx, |mut rx| async {
            rx.recv().await.map(|value| (value, rx))
        })),
    )
}
async fn terminal(output: &mut ServiceStream<pb::NativeEvent>) -> pb::Terminal {
    let first = output.next().await.unwrap().unwrap();
    assert!(matches!(
        first.item,
        Some(pb::native_event::Item::Workspace(_) | pb::native_event::Item::Failure(_))
    ));
    let finality = output.next().await.unwrap().unwrap();
    let Some(pb::native_event::Item::Terminal(terminal)) = finality.item else {
        panic!("missing Terminal")
    };
    assert_eq!(terminal.completion, pb::terminal::Completion::CpuSettled);
    assert!(terminal.completed);
    assert!(output.next().await.is_none());
    *terminal
}
#[tokio::test]
async fn plan61_half_close_is_not_cancel_and_terminal_follows_cpu_completion() {
    let (session, fixture, _) = session();
    let (tx, requests) = controls();
    let mut output = session.run(requests).unwrap();
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&fixture, 0).await;
    drop(tx);
    assert!(!gate.completed.load(Ordering::SeqCst));
    gate.release();
    let terminal = terminal(&mut output).await;
    assert!(!terminal.cancelled && !terminal.cancellation_requested);
    assert!(matches!(
        terminal.original_outcome,
        Some(pb::terminal::OriginalOutcome::Succeeded(_))
    ));
    session.reap().unwrap();
    session.shutdown().await.unwrap();
}
#[tokio::test]
async fn plan61_dropped_output_retains_cpu_owner_and_does_not_cancel_sibling() {
    let (session, fixture, _) = session();
    let (tx, requests) = controls();
    let output = session.run(requests).unwrap();
    tx.send(Ok(start())).await.unwrap();
    let first = gate(&fixture, 0).await;
    let (other_tx, requests) = controls();
    let mut sibling = session.run(requests).unwrap();
    other_tx.send(Ok(start())).await.unwrap();
    let second = gate(&fixture, 1).await;
    drop(output);
    first.cancelled.notified().await;
    assert!(!first.completed.load(Ordering::SeqCst));
    first.release();
    second.release();
    assert!(!terminal(&mut sibling).await.cancelled);
    session.shutdown().await.unwrap();
    assert!(first.completed.load(Ordering::SeqCst));
}
#[tokio::test]
async fn plan61_cancel_and_shutdown_wait_for_actual_cpu_return() {
    for explicit in [true, false] {
        let (session, fixture, stop) = session();
        let (tx, requests) = controls();
        let mut output = session.run(requests).unwrap();
        tx.send(Ok(start())).await.unwrap();
        let gate = gate(&fixture, 0).await;
        if explicit {
            tx.send(Ok(cancel())).await.unwrap();
        } else {
            stop.cancel();
        }
        gate.cancelled.notified().await;
        assert!(!gate.completed.load(Ordering::SeqCst));
        let draining = {
            let session = session.clone();
            tokio::spawn(async move { session.shutdown().await })
        };
        tokio::task::yield_now().await;
        assert!(!draining.is_finished());
        gate.release();
        while output.next().await.is_some() {}
        draining.await.unwrap().unwrap();
        assert!(gate.completed.load(Ordering::SeqCst));
        assert!(session.run(controls().1).is_err());
    }
}
#[tokio::test]
async fn plan61_repeated_start_is_invalid_but_still_settles_cpu() {
    let (session, fixture, _) = session();
    let (tx, requests) = controls();
    let mut output = session.run(requests).unwrap();
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&fixture, 0).await;
    tx.send(Ok(start())).await.unwrap();
    gate.cancelled.notified().await;
    gate.release();
    let finality = terminal(&mut output).await;
    assert!(finality.cancelled && finality.original_outcome.is_some());
    assert!(gate.completed.load(Ordering::SeqCst));
    session.shutdown().await.unwrap();
}
#[tokio::test]
async fn plan61_repeated_cancel_is_idempotent_and_terminal_is_unique() {
    let (session, fixture, _) = session();
    let (tx, requests) = controls();
    let mut output = session.run(requests).unwrap();
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&fixture, 0).await;
    tx.send(Ok(cancel())).await.unwrap();
    gate.cancelled.notified().await;
    tx.send(Ok(cancel())).await.unwrap();
    drop(tx);
    gate.release();
    let finality = terminal(&mut output).await;
    assert!(finality.cancelled && finality.cancellation_requested);
    session.shutdown().await.unwrap();
}
#[tokio::test]
async fn plan61_slow_output_cannot_hold_session_shutdown_after_cpu_return() {
    let (session, fixture, _) = session();
    let (tx, requests) = controls();
    let _unpolled = session.run(requests).unwrap();
    tx.send(Ok(start())).await.unwrap();
    let gate = gate(&fixture, 0).await;
    gate.release();
    tokio::time::timeout(std::time::Duration::from_secs(10), session.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert!(gate.completed.load(Ordering::SeqCst));
}
struct Server {
    _directory: tempfile::TempDir,
    socket: std::path::PathBuf,
    stop: CancellationToken,
    task: tokio::task::JoinHandle<semantic::Result<()>>,
    fixture: Arc<Fixture>,
}
impl Server {
    async fn start() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("analysis.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let fixture = Arc::new(Fixture::default());
        let stop = CancellationToken::new();
        let task = tokio::spawn(rpc::serve_fixture(listener, fixture.clone(), stop.clone()));
        Self {
            _directory: directory,
            socket,
            stop,
            task,
            fixture,
        }
    }
    async fn client(
        &self,
    ) -> bindings::NativeSemanticClient<connectrpc::client::SharedHttp2Connection> {
        let configuration =
            connectrpc::client::ClientConfig::new("http://localhost".parse().unwrap())
                .with_protocol(connectrpc::Protocol::Grpc);
        let connection = connectrpc::client::Http2Connection::connect_unix(
            &self.socket,
            "http://localhost".parse().unwrap(),
        )
        .await
        .unwrap()
        .shared(32);
        bindings::NativeSemanticClient::new(connection, configuration)
    }
    async fn shutdown(self) {
        self.stop.cancel();
        self.task.await.unwrap().unwrap();
    }
}
#[tokio::test]
async fn plan61_production_grpc_half_close_and_reset_preserve_real_cpu_owner() {
    let server = Server::start().await;
    let client = server.client().await;
    assert_eq!(
        client
            .status(buffa_types::google::protobuf::Empty::default())
            .await
            .unwrap()
            .into_owned()
            .backend,
        "controlled real CPU operation"
    );
    let mut stream = client.run().await.unwrap();
    stream.send(start()).await.unwrap();
    let first = gate(&server.fixture, 0).await;
    stream.close_send();
    first.release();
    assert!(stream.message::<pb::NativeEvent>().await.unwrap().is_some());
    let finality = stream
        .message::<pb::NativeEvent>()
        .await
        .unwrap()
        .unwrap()
        .to_owned_message();
    assert!(
        matches!(finality.item, Some(pb::native_event::Item::Terminal(t)) if t.completed && !t.cancelled)
    );
    assert!(stream.message::<pb::NativeEvent>().await.unwrap().is_none());
    let mut lost = client.run().await.unwrap();
    lost.send(start()).await.unwrap();
    let second = gate(&server.fixture, 1).await;
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
async fn plan61_production_grpc_validates_start_before_admission() {
    let server = Server::start().await;
    let client = server.client().await;
    let mut stream = client.run().await.unwrap();
    stream
        .send(pb::NativeControl {
            action: Some(pb::native_control::Action::Start(Box::default())),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        stream.message::<pb::NativeEvent>().await.unwrap_err().code,
        connectrpc::ErrorCode::InvalidArgument
    );
    assert!(server.fixture.gates.lock().unwrap().is_empty());
    server.shutdown().await;
}
#[tokio::test]
async fn plan61_production_grpc_deadline_leaves_cpu_owned_until_return() {
    let server = Server::start().await;
    let client = server.client().await;
    let mut stream = client
        .run_with_options(
            connectrpc::client::CallOptions::default()
                .with_timeout(std::time::Duration::from_millis(500)),
        )
        .await
        .unwrap();
    stream.send(start()).await.unwrap();
    let gate = gate(&server.fixture, 0).await;
    assert_eq!(
        stream.message::<pb::NativeEvent>().await.unwrap_err().code,
        connectrpc::ErrorCode::DeadlineExceeded
    );
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
async fn plan61_unary_reset_keeps_cpu_work_in_the_production_session() {
    let server = Server::start().await;
    let client = server.client().await;
    let request = tokio::spawn(async move {
        client
            .package_context(pb::NativePackageRequest {
                package_id: "actual-package".into(),
                ..Default::default()
            })
            .await
    });
    let gate = gate(&server.fixture, 0).await;
    request.abort();
    let _aborted = request.await;
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
async fn plan61_reconnecting_client_refuses_real_wrong_custody_before_http2_bytes() {
    use tokio::io::AsyncReadExt as _;
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("untrusted.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let connector = rpc::AuthenticatedConnector {
        socket,
        root,
        invocation: crate::envelope::CliInvocationId::new(),
        environment: std::env::vars_os().collect(),
    };
    let peer = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            assert_eq!(
                stream.peer_cred().unwrap().uid(),
                rustix::process::geteuid().as_raw()
            );
            let mut bytes = [0u8; 1];
            assert_eq!(
                stream.read(&mut bytes).await.unwrap(),
                0,
                "authentication must precede the h2 handshake"
            );
        }
    });
    let config = connectrpc::client::ClientConfig::new("http://localhost".parse().unwrap())
        .with_protocol(connectrpc::Protocol::Grpc);
    let connection = connectrpc::client::Http2Connection::lazy_with_connector(
        connector,
        config.base_uri().clone(),
    )
    .shared(32);
    let client = bindings::NativeSemanticClient::new(connection, config);
    for _ in 0..2 {
        assert!(
            client
                .status(buffa_types::google::protobuf::Empty::default())
                .await
                .is_err()
        );
    }
    tokio::time::timeout(std::time::Duration::from_secs(10), peer)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn plan61_typed_caller_preserves_validation_field_locations_and_rule_ids() {
    let server = Server::start().await;
    let client = server.client().await;
    let mut stream = client.run().await.unwrap();
    stream
        .send(pb::NativeControl {
            action: Some(pb::native_control::Action::Start(Box::new(
                pb::NativeRequest {
                    operation: Some(pb::native_request::Operation::Query(Box::new(
                        pb::NativeQuery {
                            file: b"/fixture.rs".to_vec(),
                            line: 0,
                            column: 1,
                            limit: 1,
                            ..Default::default()
                        },
                    ))),
                    ..Default::default()
                },
            ))),
            ..Default::default()
        })
        .await
        .unwrap();
    let wire_error = stream.message::<pb::NativeEvent>().await.unwrap_err();
    let semantic::Error::Refusal(failure) = rpc::transport_error(wire_error) else {
        panic!("missing typed request failure")
    };
    assert!(matches!(failure.phase, semantic::FailurePhase::Constraint));
    let details: Vec<connectrpc::ErrorDetail> =
        serde_json::from_value(failure.backend_data.unwrap()["original_rpc_details"].clone())
            .unwrap();
    let violations = details
        .iter()
        .find_map(|detail| protovalidate_buffa::decode_violations(detail).unwrap())
        .unwrap();
    assert!(violations.violations.iter().any(|violation| {
        violation.rule_id.as_deref() == Some("uint64.gte")
            && violation
                .field
                .elements
                .iter()
                .any(|element| element.field_name.as_deref() == Some("line"))
    }));
    assert!(violations.violations.iter().any(|violation| {
        violation.rule_id.as_deref() == Some("required")
            && violation
                .field
                .elements
                .iter()
                .any(|element| element.field_name.as_deref() == Some("kind"))
    }));
    assert!(server.fixture.gates.lock().unwrap().is_empty());
    server.shutdown().await;
}

#[tokio::test]
async fn plan61_server_deadline_cancels_retained_cpu_with_open_client_transport() {
    let server = Server::start().await;
    let client = server.client().await;
    // Assert only wire metadata: no local client timeout or reset drives cancellation.
    let mut stream = client
        .run_with_options(
            connectrpc::client::CallOptions::default().with_header("grpc-timeout", "500m"),
        )
        .await
        .unwrap();
    stream.send(start()).await.unwrap();
    let gate = gate(&server.fixture, 0).await;
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        gate.cancelled.notified(),
    )
    .await
    .unwrap();
    assert!(!gate.completed.load(Ordering::SeqCst));
    gate.release();
    let failure = stream
        .message::<pb::NativeEvent>()
        .await
        .unwrap()
        .unwrap()
        .to_owned_message();
    assert!(
        matches!(failure.item, Some(pb::native_event::Item::Failure(f)) if f.phase == pb::failure::Phase::Transport)
    );
    let finality = stream
        .message::<pb::NativeEvent>()
        .await
        .unwrap()
        .unwrap()
        .to_owned_message();
    assert!(
        matches!(finality.item, Some(pb::native_event::Item::Terminal(t)) if t.completed && t.cancelled && t.completion == pb::terminal::Completion::CpuSettled)
    );
    assert!(stream.message::<pb::NativeEvent>().await.unwrap().is_none());
    assert!(gate.completed.load(Ordering::SeqCst));
    server.shutdown().await;
}

#[test]
fn plan61_client_validates_every_terminal_without_a_cancel_request() {
    fn success() -> semantic::Result<semantic::Response> {
        let configuration = Fixture::default().status().configuration;
        Ok(semantic::Response::WorkspaceSymbols(
            semantic::WorkspaceAnswer {
                session: semantic::AnalysisContext {
                    backend: "controlled native".into(),
                    configuration,
                    position_encoding: "utf-8".into(),
                    backend_status: None,
                    preparation: "test controlled".into(),
                },
                generation: 1,
                symbols: semantic::Section {
                    category: "workspace_symbols".into(),
                    evidence: serde_json::json!([]),
                    omitted: 0,
                    handles: vec![],
                },
                observed: vec![],
                limitations: vec![],
            },
        ))
    }
    fn finality() -> pb::Terminal {
        pb::Terminal {
            completed: true,
            completion: pb::terminal::Completion::CpuSettled.into(),
            original_outcome: Some(pb::terminal::OriginalOutcome::Succeeded(Default::default())),
            ..Default::default()
        }
    }
    let mut absent_settlement = finality();
    absent_settlement.completion = pb::terminal::Completion::Unobserved.into();
    let mut unknown_settlement = finality();
    unknown_settlement.completion = buffa::EnumValue::Unknown(99);
    let mut absent_original = finality();
    absent_original.original_outcome = None;
    let mut invalid_original = finality();
    invalid_original.original_outcome = Some(pb::terminal::OriginalOutcome::Failed(Box::new(
        pb::Failure {
            state: buffa::EnumValue::Unknown(99),
            ..Default::default()
        },
    )));
    let mut contradicted_success = finality();
    contradicted_success.original_outcome = Some(pb::terminal::OriginalOutcome::Failed(Box::new(
        pb::Failure {
            state: pb::failure::State::Failed.into(),
            phase: pb::failure::Phase::Provider.into(),
            message: "actual provider failed".into(),
            ..Default::default()
        },
    )));
    for terminal in [
        absent_settlement,
        unknown_settlement,
        absent_original,
        invalid_original,
        contradicted_success,
    ] {
        assert!(matches!(
            rpc::finish_run(success(), terminal, false),
            Err(semantic::Error::Schema(_))
        ));
    }
    assert!(rpc::finish_run(success(), finality(), false).is_ok());
    assert!(
        matches!(
            rpc::finish_run(
                Err(semantic::Error::InvalidRequest(
                    "invalid post-Start control".into()
                )),
                finality(),
                false
            ),
            Err(semantic::Error::InvalidRequest(_))
        ),
        "control failure must remain distinct from the original successful provider result"
    );
    let mut original_cancelled = finality();
    original_cancelled.original_outcome = Some(pb::terminal::OriginalOutcome::Failed(Box::new(
        pb::Failure {
            state: pb::failure::State::Cancelled.into(),
            phase: pb::failure::Phase::Settlement.into(),
            ..Default::default()
        },
    )));
    assert!(
        matches!(
            rpc::finish_run(
                Err(semantic::Error::InvalidRequest(
                    "invalid post-Start control".into()
                )),
                original_cancelled,
                false
            ),
            Err(semantic::Error::InvalidRequest(_))
        ),
        "control failure must not be equated with a settled native cancellation"
    );
}
