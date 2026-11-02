// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual native reference workers exercised through the production gRPC conversation.
use super::*;
use crate::semantic::{Kind, native};

#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "test-only admission barriers must fail the fixture if its native engine or synchronization custody is lost"
)]
fn observe(
    engine: &ServiceEngine,
) -> (
    oneshot::Receiver<CancellationToken>,
    std::sync::mpsc::Sender<()>,
) {
    let (entered, observed) = oneshot::channel();
    let (release, blocked) = std::sync::mpsc::channel();
    let ServiceEngine::Snapshot(engine) = engine else {
        panic!("native fixture required");
    };
    engine.observe_next_query(move |token| {
        entered.send(token.clone()).unwrap();
        blocked.recv().unwrap();
    });
    (observed, release)
}

#[tokio::test]
async fn native_grpc_cancel_half_close_reset_and_reuse_retain_actual_drain() {
    let (directory, configuration, file) = native::tests::fixture();
    let engine = Arc::new(ServiceEngine::Snapshot(Box::new(
        native::Engine::start(directory.path(), configuration).unwrap(),
    )));
    // Creating service I/O beneath the prepared package would invalidate its
    // Cargo input watch before any native query was admitted.
    let socket_directory = tempfile::tempdir().unwrap();
    let socket = socket_directory.path().join("native.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let stopping = CancellationToken::new();
    let terminate =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    let server = tokio::spawn(serve(listener, engine.clone(), stopping.clone(), terminate));
    let connection = connectrpc::client::Http2Connection::connect_unix(
        &socket,
        "http://localhost".parse().unwrap(),
    )
    .await
    .unwrap()
    .shared(32);
    let client = bindings::NativeSemanticClient::new(
        connection,
        connectrpc::client::ClientConfig::new("http://localhost".parse().unwrap())
            .with_protocol(connectrpc::Protocol::Grpc),
    );
    let start = || pb::NativeControl {
        action: Some(pb::native_control::Action::Start(Box::new(
            wire::request_to_pb(&Request::Query(native::tests::query(
                &file,
                "original()->",
                Kind::Refs,
            )))
            .unwrap(),
        ))),
        ..Default::default()
    };

    // Half-closing the control side permits an actual native result and terminal.
    let (entered, release) = observe(&engine);
    let mut half = client.run().await.unwrap();
    half.send(start()).await.unwrap();
    let _token = entered.await.unwrap();
    half.close_send();
    release.send(()).unwrap();
    assert!(matches!(
        half.message::<pb::NativeEvent>()
            .await
            .unwrap()
            .unwrap()
            .to_owned_message()
            .item,
        Some(pb::native_event::Item::Query(_))
    ));
    let terminal = half
        .message::<pb::NativeEvent>()
        .await
        .unwrap()
        .unwrap()
        .to_owned_message();
    assert!(
        matches!(terminal.item, Some(pb::native_event::Item::Terminal(terminal)) if terminal.completed && !terminal.cancellation_requested)
    );
    assert!(half.message::<pb::NativeEvent>().await.unwrap().is_none());

    // Explicit Cancel reaches the token of a blocked, admitted native reference
    // worker. Terminal is emitted only after releasing and joining that worker.
    let (entered, release) = observe(&engine);
    let mut cancelled = client.run().await.unwrap();
    cancelled.send(start()).await.unwrap();
    let token = entered.await.unwrap();
    cancelled
        .send(pb::NativeControl {
            action: Some(pb::native_control::Action::Cancel(Default::default())),
            ..Default::default()
        })
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(30), token.cancelled())
        .await
        .unwrap();
    assert_eq!(engine.status().unwrap().active_requests, 1);
    release.send(()).unwrap();
    assert!(matches!(
        cancelled
            .message::<pb::NativeEvent>()
            .await
            .unwrap()
            .unwrap()
            .to_owned_message()
            .item,
        Some(pb::native_event::Item::Failure(_))
    ));
    let terminal = cancelled
        .message::<pb::NativeEvent>()
        .await
        .unwrap()
        .unwrap()
        .to_owned_message();
    assert!(
        matches!(terminal.item, Some(pb::native_event::Item::Terminal(terminal)) if terminal.completed && terminal.cancelled && terminal.cancellation_requested && terminal.completion == pb::terminal::Completion::CpuSettled)
    );
    assert!(
        cancelled
            .message::<pb::NativeEvent>()
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(engine.status().unwrap().active_requests, 0);

    // Losing a consumer requests cancellation without losing the native owner.
    let (entered, release) = observe(&engine);
    let mut lost = client.run().await.unwrap();
    lost.send(start()).await.unwrap();
    let token = entered.await.unwrap();
    drop(lost);
    tokio::time::timeout(Duration::from_secs(30), token.cancelled())
        .await
        .unwrap();
    assert_eq!(engine.status().unwrap().active_requests, 1);
    release.send(()).unwrap();
    // A subsequent native request remains usable on the same world.
    let mut fresh = client.run().await.unwrap();
    fresh.send(start()).await.unwrap();
    fresh.close_send();
    assert!(matches!(
        fresh
            .message::<pb::NativeEvent>()
            .await
            .unwrap()
            .unwrap()
            .to_owned_message()
            .item,
        Some(pb::native_event::Item::Query(_))
    ));
    assert!(
        matches!(fresh.message::<pb::NativeEvent>().await.unwrap().unwrap().to_owned_message().item, Some(pb::native_event::Item::Terminal(terminal)) if terminal.completed && !terminal.cancelled)
    );
    stopping.cancel();
    tokio::time::timeout(Duration::from_secs(30), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(engine.status().unwrap().active_requests, 0);
}

#[tokio::test]
async fn native_conversation_deadline_cancels_and_joins_admitted_reference_worker() {
    let (directory, configuration, file) = native::tests::fixture();
    let engine = Arc::new(ServiceEngine::Snapshot(Box::new(
        native::Engine::start(directory.path(), configuration).unwrap(),
    )));
    let (entered, release) = observe(&engine);
    let session = Arc::new(Session::new(engine.clone(), CancellationToken::new()));
    let (tx, rx) = mpsc::channel(1);
    let requests = Box::pin(futures_util::stream::unfold(rx, |mut rx| async {
        rx.recv().await.map(|value| (value, rx))
    }));
    let mut output = session
        .run_with_deadline(
            requests,
            Some(std::time::Instant::now() + Duration::from_secs(1)),
        )
        .unwrap();
    tx.send(Ok(pb::NativeControl {
        action: Some(pb::native_control::Action::Start(Box::new(
            wire::request_to_pb(&Request::Query(native::tests::query(
                &file,
                "original()->",
                Kind::Refs,
            )))
            .unwrap(),
        ))),
        ..Default::default()
    }))
    .await
    .unwrap();
    let token = entered.await.unwrap();
    tokio::time::timeout(Duration::from_secs(30), token.cancelled())
        .await
        .unwrap();
    assert_eq!(engine.status().unwrap().active_requests, 1);
    release.send(()).unwrap();
    let _failure = output.next().await.unwrap().unwrap();
    assert!(
        matches!(output.next().await.unwrap().unwrap().item, Some(pb::native_event::Item::Terminal(terminal)) if terminal.completed && terminal.cancellation_requested && terminal.completion == pb::terminal::Completion::CpuSettled)
    );
    session.shutdown().await.unwrap();
    assert_eq!(engine.status().unwrap().active_requests, 0);
}
