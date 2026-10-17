// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native flatbuffers peer controls for the pinned SDK lifecycle patch.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "mock-peer protocol failures must fail the lifecycle test"
)]

use futures_util::{SinkExt, StreamExt};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::{Config, DispatchState, RequestContext, WebsocketConfig},
    types::{Object, Value},
};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message,
        handshake::server::{ErrorResponse, Request, Response},
    },
};

#[expect(
    clippy::result_large_err,
    reason = "tungstenite fixes the handshake callback's HTTP error response type"
)]
fn native_protocol(_request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
    response
        .headers_mut()
        .insert("Sec-WebSocket-Protocol", "flatbuffers".parse().unwrap());
    Ok(response)
}

async fn peer(
    acknowledge_replay: bool,
) -> (
    Arc<Surreal<Client>>,
    mpsc::UnboundedReceiver<String>,
    tokio::task::JoinHandle<()>,
) {
    peer_with_limit(acknowledge_replay, 4 * 1024 * 1024).await
}

async fn peer_with_limit(
    acknowledge_replay: bool,
    message_bytes: usize,
) -> (
    Arc<Surreal<Client>>,
    mpsc::UnboundedReceiver<String>,
    tokio::task::JoinHandle<()>,
) {
    peer_with_replay_release(acknowledge_replay, message_bytes, None).await
}

async fn peer_with_replay_release(
    acknowledge_replay: bool,
    message_bytes: usize,
    mut release_replay: Option<tokio::sync::oneshot::Receiver<()>>,
) -> (
    Arc<Surreal<Client>>,
    mpsc::UnboundedReceiver<String>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent, mut received) = mpsc::unbounded_channel();
    let server = tokio::spawn(async move {
        let connections = if acknowledge_replay { 1 } else { 2 };
        for connection in 0..connections {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
            while let Some(Ok(message)) = socket.next().await {
                let Message::Binary(bytes) = message else {
                    continue;
                };
                let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                    continue;
                };
                let method = request
                    .get("method")
                    .and_then(Value::as_string)
                    .unwrap()
                    .to_owned();
                let query = request
                    .get("params")
                    .and_then(Value::as_array)
                    .and_then(|params| params.first())
                    .and_then(Value::as_string)
                    .map_or("", String::as_str);
                sent.send(if method == "query" {
                    query.to_owned()
                } else {
                    method.clone()
                })
                .ok();
                if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                    continue;
                }
                if method == "attach" && connection == 1 {
                    if let Some(release) = release_replay.take() {
                        release.await.unwrap();
                    } else {
                        continue;
                    }
                }
                if method == "query"
                    && query != "CONTROL"
                    && query != "STATISTICS"
                    && query != "REQUEST_FAILURE"
                {
                    continue;
                }
                let mut reply = Object::new();
                reply.insert("id", request.get("id").unwrap().clone());
                reply.insert("session", request.get("session").unwrap().clone());
                let result = if method == "query" {
                    let mut statement = Object::new();
                    statement.insert("status", "OK");
                    statement.insert("time", "0ns");
                    statement.insert("result", true);
                    if query == "STATISTICS" {
                        statement.insert("time", "37ns");
                        let mut failed = Object::new();
                        failed.insert("status", "ERR");
                        failed.insert("time", "91ns");
                        failed.insert("result", "controlled statement failure");
                        Value::Array(vec![Value::Object(statement), Value::Object(failed)].into())
                    } else {
                        Value::Array(vec![Value::Object(statement)].into())
                    }
                } else if method == "version" {
                    Value::String("3.3.0".into())
                } else {
                    Value::None
                };
                if query == "REQUEST_FAILURE" {
                    reply.insert(
                        "error",
                        surrealdb::types::SurrealValue::into_value(surrealdb::Error::internal(
                            "controlled request failure".into(),
                        )),
                    );
                } else {
                    reply.insert("result", result);
                }
                socket
                    .send(Message::Binary(
                        surrealdb::types::encode(&Value::Object(reply))
                            .unwrap()
                            .into(),
                    ))
                    .await
                    .unwrap();
                if !acknowledge_replay && connection == 0 && method == "version" {
                    socket.close(None).await.unwrap();
                    break;
                }
            }
        }
    });
    let client = Surreal::new::<Ws>((
        address,
        Config::new()
            .bounded_requests()
            .websocket(WebsocketConfig::new().max_message_size(message_bytes))
            .unwrap(),
    ))
    .await
    .unwrap();
    if !acknowledge_replay {
        let mut attachments = 0;
        while attachments < 2 {
            if tokio::time::timeout(Duration::from_secs(2), received.recv())
                .await
                .unwrap()
                .unwrap()
                == "attach"
            {
                attachments += 1;
            }
        }
    }
    (Arc::new(client), received, server)
}

async fn next_query(received: &mut mpsc::UnboundedReceiver<String>) -> String {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(2), received.recv())
            .await
            .unwrap()
            .unwrap();
        if !matches!(message.as_str(), "attach" | "ping" | "version") {
            return message;
        }
    }
}

#[tokio::test]
async fn native_sdk_reserved_control_progress_and_physical_disconnect() {
    let (client, mut received, server) = peer(true).await;
    let mut tasks = tokio::task::JoinSet::new();
    let mut contexts = Vec::new();
    for _ in 0..32 {
        let context = RequestContext::new(Instant::now() + Duration::from_secs(10));
        contexts.push(context.clone());
        let client = Arc::clone(&client);
        tasks.spawn(async move { client.query("APPLICATION").request_context(context).await });
    }
    for _ in 0..32 {
        assert_eq!(next_query(&mut received).await, "APPLICATION");
    }
    let refused = RequestContext::new(Instant::now() + Duration::from_millis(40));
    assert!(
        client
            .query("OVER_CAPACITY")
            .request_context(refused.clone())
            .await
            .is_err()
    );
    assert_eq!(refused.dispatch_state(), DispatchState::NotDispatched);
    let mut result = client
        .query("CONTROL")
        .request_context(RequestContext::control(
            Instant::now() + Duration::from_secs(2),
        ))
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(result.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(next_query(&mut received).await, "CONTROL");
    tokio::time::timeout(Duration::from_secs(2), client.disconnect())
        .await
        .unwrap()
        .unwrap();
    while let Some(result) = tasks.join_next().await {
        assert!(result.unwrap().is_err());
    }
    assert!(contexts.iter().all(RequestContext::was_dispatched));
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn native_sdk_stalled_replay_terminally_drains_and_refuses_later_admission() {
    let (client, mut received, server) = peer(false).await;
    let expired = RequestContext::new(Instant::now() + Duration::from_millis(40));
    assert!(
        client
            .query("EXPIRED")
            .request_context(expired.clone())
            .await
            .is_err()
    );
    assert_eq!(expired.dispatch_state(), DispatchState::NotDispatched);
    let cancelled = RequestContext::new(Instant::now() + Duration::from_secs(5));
    let query = client.query("CANCELLED").request_context(cancelled.clone());
    cancelled.cancel();
    assert!(query.await.is_err());
    assert_eq!(cancelled.dispatch_state(), DispatchState::NotDispatched);
    // Keep the peer alive and consuming pings. A missing setup ACK itself must
    // close this client, independent of the short callers' original clocks.
    let waiting = RequestContext::new(Instant::now() + Duration::from_secs(20));
    assert!(
        tokio::time::timeout(
            Duration::from_secs(6),
            client.query("NEVER_READY").request_context(waiting.clone())
        )
        .await
        .expect("one five-second restoration budget")
        .is_err()
    );
    assert_eq!(waiting.dispatch_state(), DispatchState::NotDispatched);
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    let later = RequestContext::new(Instant::now() + Duration::from_secs(20));
    assert!(
        tokio::time::timeout(
            Duration::from_secs(1),
            client
                .query("AFTER_TERMINAL")
                .request_context(later.clone())
        )
        .await
        .expect("terminal admission refuses without owner reopen")
        .is_err()
    );
    assert_eq!(later.dispatch_state(), DispatchState::NotDispatched);
    tokio::time::timeout(Duration::from_secs(1), client.disconnect())
        .await
        .unwrap()
        .unwrap();
    let mut ping_seen = false;
    while let Ok(message) = received.try_recv() {
        ping_seen |= message == "ping";
        assert!(matches!(message.as_str(), "attach" | "ping" | "version"));
    }
    assert!(
        ping_seen,
        "the peer stayed open and consumed keepalive traffic"
    );
}

#[tokio::test]
async fn native_sdk_replay_ack_before_bound_readies_after_another_caller_expires() {
    use std::future::{Future, IntoFuture};
    let (release, released) = tokio::sync::oneshot::channel();
    let (client, mut received, server) =
        peer_with_replay_release(false, 4 * 1024 * 1024, Some(released)).await;
    let expired = RequestContext::new(Instant::now() + Duration::from_millis(40));
    assert!(
        client
            .query("EXPIRED")
            .request_context(expired.clone())
            .await
            .is_err()
    );
    assert_eq!(expired.dispatch_state(), DispatchState::NotDispatched);
    let context = RequestContext::control(Instant::now() + Duration::from_secs(3));
    let mut query = Box::pin(
        client
            .query("CONTROL")
            .request_context(context.clone())
            .into_future(),
    );
    std::future::poll_fn(|cx| {
        assert!(query.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.send(()).unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(2), query)
            .await
            .unwrap()
            .unwrap()
            .check()
            .is_ok()
    );
    assert_eq!(context.dispatch_state(), DispatchState::Completed);
    assert_eq!(next_query(&mut received).await, "CONTROL");
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn native_sdk_dropped_caller_cancels_queued_admission_before_dispatch() {
    use std::future::{Future, IntoFuture};
    let (client, mut received, server) = peer(false).await;
    let context = RequestContext::new(Instant::now() + Duration::from_secs(5));
    let mut pending = Box::pin(
        client
            .query("DROPPED_CALLER")
            .request_context(context.clone())
            .into_future(),
    );
    std::future::poll_fn(|cx| {
        assert!(pending.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(pending);
    assert!(context.is_interrupted());
    assert!(!context.was_dispatched());
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    while let Ok(message) = received.try_recv() {
        assert!(matches!(message.as_str(), "attach" | "ping" | "version"));
    }
}

#[tokio::test]
async fn native_sdk_final_owner_drop_physically_closes_abandoned_transport() {
    let (client, mut received, server) = peer(true).await;
    let context = RequestContext::new(Instant::now() + Duration::from_secs(5));
    let owned = Arc::clone(&client);
    let request_context = context.clone();
    let pending = tokio::spawn(async move {
        owned
            .query("APPLICATION")
            .request_context(request_context)
            .await
    });
    assert_eq!(next_query(&mut received).await, "APPLICATION");
    drop(client);
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    assert!(context.was_dispatched());
    assert!(context.is_interrupted());
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn native_sdk_replay_bound_counts_automatic_attachment_and_refuses_extra_session() {
    let (client, mut received, server) = peer(true).await;
    for index in 0..63 {
        client
            .use_ns(format!("namespace_{index}"))
            .request_context(RequestContext::control(
                Instant::now() + Duration::from_secs(2),
            ))
            .await
            .unwrap();
        assert_eq!(next_query(&mut received).await, "use");
    }
    let refused = RequestContext::control(Instant::now() + Duration::from_secs(2));
    assert!(
        client
            .use_ns("over_replay_limit")
            .request_context(refused.clone())
            .await
            .is_err()
    );
    assert!(!refused.was_dispatched());
    let clone = client.as_ref().clone();
    assert!(
        clone
            .query("SECOND_SESSION")
            .request_context(RequestContext::new(Instant::now() + Duration::from_secs(2)))
            .await
            .is_err()
    );
    drop(clone);
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn canonical_conflict_clock_does_not_renew_nested_deadlines() {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(60);
    let result = super::within_clock(deadline, async {
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert_eq!(super::original_deadline(Duration::from_secs(30)), deadline);
        super::within_clock(super::original_deadline(Duration::from_secs(30)), async {
            tokio::time::sleep(Duration::from_millis(40)).await;
            Ok(())
        })
        .await
    })
    .await;
    assert!(matches!(
        result,
        Err(crate::canonical::CanonicalError::Timeout)
    ));
}

#[tokio::test]
async fn canonical_file_metadata_conflicts_preserve_original_effect_and_unknown_seal_is_not_replayed()
 {
    use crate::canonical::CanonicalError;
    use std::{collections::BTreeMap, sync::Mutex};
    use surrealdb::types::{QueryError, SurrealValue};
    for unknown_seal in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let calls = Arc::new(Mutex::new(BTreeMap::<String, Vec<Object>>::new()));
        let observed = calls.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
            let mut file: Option<Object> = None;
            while let Some(Ok(Message::Binary(bytes))) = socket.next().await {
                let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                    continue;
                };
                if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                    continue;
                }
                let method = request.get("method").and_then(Value::as_string).unwrap();
                let result = if method == "query" {
                    let params = request.get("params").and_then(Value::as_array).unwrap();
                    let query = params[0].as_string().unwrap();
                    let bindings = params[1].as_object().unwrap();
                    let phase = ["register_owner", "reserve", "intent_open", "seal"]
                        .into_iter()
                        .find(|phase| query.contains(&format!("fn::pse_files_v1::{phase}(")))
                        .unwrap();
                    let first = {
                        let mut calls = observed.lock().unwrap();
                        let attempts = calls.entry(phase.to_owned()).or_default();
                        attempts.push(bindings.clone());
                        attempts.len() == 1
                    };
                    let output = if first {
                        Err(if phase == "seal" && unknown_seal {
                            surrealdb::Error::query(
                                "unknown original sealing acknowledgement".into(),
                                None,
                            )
                        } else {
                            surrealdb::Error::query(
                                "definite metadata commit conflict".into(),
                                QueryError::TransactionConflict,
                            )
                        })
                    } else {
                        let row = match phase {
                            "register_owner" => bindings
                                .get("row")
                                .and_then(Value::as_object)
                                .unwrap()
                                .clone(),
                            "reserve" => {
                                let row = bindings
                                    .get("row")
                                    .and_then(Value::as_object)
                                    .unwrap()
                                    .clone();
                                file = Some(row.clone());
                                row
                            }
                            "intent_open" => file.as_ref().unwrap().clone(),
                            "seal" => {
                                let row = file.as_mut().unwrap();
                                let descriptor = bindings
                                    .get("descriptor")
                                    .and_then(Value::as_object)
                                    .unwrap();
                                for (target, source) in [
                                    ("digest", "digest"),
                                    ("payload_bytes", "bytes"),
                                    ("encoding", "encoding"),
                                    ("batch_count", "batch_count"),
                                    ("schema", "schema"),
                                    ("interpretation", "interpretation"),
                                ] {
                                    row.insert(target, descriptor.get(source).unwrap().clone());
                                }
                                row.insert("state", "sealed");
                                row.clone()
                            }
                            _ => panic!("unexpected file-metadata phase: {phase}"),
                        };
                        Ok(Value::Object(row))
                    };
                    let ack = |result: Result<Value, surrealdb::Error>| {
                        let mut row = match result {
                            Ok(value) => {
                                let mut row = Object::new();
                                row.insert("status", "OK");
                                row.insert("result", value);
                                row
                            }
                            Err(error) => {
                                let mut row = error
                                    .details()
                                    .clone()
                                    .into_value()
                                    .as_object()
                                    .expect("query error details are an object")
                                    .clone();
                                row.insert("status", "ERR");
                                row.insert("result", error.message().to_owned());
                                row
                            }
                        };
                        row.insert("time", "0ns");
                        Value::Object(row)
                    };
                    // Refuse at COMMIT, the production failure seam, before any
                    // original catalog effect is acknowledged by the caller.
                    let statements = match output {
                        Ok(row) => vec![ack(Ok(Value::None)), ack(Ok(row)), ack(Ok(Value::None))],
                        Err(error) => {
                            vec![ack(Ok(Value::None)), ack(Ok(Value::None)), ack(Err(error))]
                        }
                    };
                    Value::Array(statements.into())
                } else if method == "version" {
                    Value::String("3.3.0".into())
                } else {
                    Value::None
                };
                let mut reply = Object::new();
                reply.insert("id", request.get("id").unwrap().clone());
                reply.insert("session", request.get("session").unwrap().clone());
                reply.insert("result", result);
                socket
                    .send(Message::Binary(
                        surrealdb::types::encode(&Value::Object(reply))
                            .unwrap()
                            .into(),
                    ))
                    .await
                    .unwrap();
            }
        });
        let state =
            std::env::temp_dir().join(format!("pse-file-metadata-retry-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&state).unwrap();
        let options = crate::canonical::CanonicalOptions {
            endpoint: format!("ws://{address}"),
            namespace: "namespace".into(),
            database: "database".into(),
            username: "owner".into(),
            password: "test".into(),
            selection_username: "viewer".into(),
            selection_password: "test".into(),
            state_path: state.clone(),
            initializer_nonce: None,
        };
        std::fs::write(state.join("config.json"), serde_json::to_vec(&serde_json::json!({
            "namespace": options.namespace, "database": options.database, "endpoint": options.endpoint,
            "payload_incarnation": uuid::Uuid::new_v4().to_string(), "accepting_writes": true,
        })).unwrap()).unwrap();
        let sdk = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
            .await
            .unwrap();
        let client = Arc::new(super::CanonicalClient::new(
            sdk,
            Duration::from_secs(5),
            &options,
        ));
        let issuing_store = Arc::new(std::sync::OnceLock::new());
        issuing_store.set("wire-file-store".to_owned()).unwrap();
        let store = crate::canonical::CanonicalStore {
            db: client.clone(),
            activation_db: client,
            problem_turns: Arc::default(),
            fixture_lifetime: None,
            issuing_store,
            files: Arc::new(
                crate::canonical_files::FileSession::from_state(&state, &options.database).unwrap(),
            ),
            result_read_drain: Arc::default(),
            connection_options: options.clone(),
            state_path: state.clone(),
            database: options.database,
            namespace: options.namespace,
        };
        let bytes = b"[1,2]";
        let digest = crate::canonical_execution::result_payload_digest(bytes);
        let intent = store
            .reserve_artifact_file("selected-artifact", "original-intent", &digest)
            .await
            .unwrap();
        let sealed = store
            .seal_file(&intent, bytes, crate::canonical_files::OPAQUE_JSON, 0, "")
            .await;
        if unknown_seal {
            assert!(
                matches!(sealed, Err(CanonicalError::Driver(error)) if error.is_query() && error.query_details().is_none())
            );
        } else {
            let sealed = sealed.unwrap();
            assert_eq!(
                serde_json::to_value(sealed.original_intent()).unwrap(),
                serde_json::to_value(&intent).unwrap()
            );
            assert_eq!(sealed.descriptor().digest, digest);
        }
        assert_eq!(
            std::fs::read(store.files.path(intent.instance()).unwrap()).unwrap(),
            bytes
        );
        {
            let calls = calls.lock().unwrap();
            for (phase, expected) in [
                ("register_owner", 2),
                ("reserve", 2),
                ("intent_open", 3),
                ("seal", if unknown_seal { 1 } else { 2 }),
            ] {
                let attempts = &calls[phase];
                assert_eq!(attempts.len(), expected, "{phase}");
                let meaning = |row: &Object| {
                    let mut row = row.clone();
                    row.remove("pse_rpc_expires_at");
                    row.remove("pse_rpc_timeout");
                    row
                };
                for row in &attempts[1..] {
                    assert_eq!(
                        meaning(&attempts[0]),
                        meaning(row),
                        "{phase} must retain exact original effect"
                    );
                }
            }
        }
        store.db.disconnect().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .unwrap()
            .unwrap();
        drop(store);
        std::fs::remove_dir_all(state).unwrap();
    }
}

#[tokio::test]
async fn canonical_operation_scope_keeps_nested_minimum_clock_and_drains_issued_owner() {
    let deadline = Instant::now() + Duration::from_millis(20);
    let expected = tokio::time::Instant::from_std(deadline);
    let mut drained = false;
    super::operation_scope(deadline, async {
        super::operation_scope(deadline + Duration::from_secs(30), async {
            assert_eq!(super::original_deadline(Duration::from_secs(60)), expected);
            let earlier = deadline - Duration::from_millis(1);
            super::operation_scope(earlier, async {
                assert_eq!(
                    super::original_deadline(Duration::from_secs(60)),
                    tokio::time::Instant::from_std(earlier)
                );
            })
            .await;
            assert_eq!(super::original_deadline(Duration::from_secs(60)), expected);
            // This represents an already issued effect whose completion owner must
            // remain polled after the caller's admission clock expires.
            tokio::time::sleep_until(expected + Duration::from_millis(10)).await;
            assert!(matches!(
                super::within_clock(super::original_deadline(Duration::from_secs(60)), async {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    Ok(())
                })
                .await,
                Err(crate::canonical::CanonicalError::Timeout)
            ));
            drained = true;
        })
        .await;
    })
    .await;
    assert!(
        drained,
        "expired admission must not drop the issued effect owner"
    );
}

async fn response_ownership_late_frame(spawned_acquisition: bool, second_metadata_rpc: bool) {
    use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (entered, mut requests) = mpsc::unbounded_channel();
    let (resume, released) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
        let mut released = Some(released);
        while let Some(Ok(message)) = socket.next().await {
            let Message::Binary(bytes) = message else {
                continue;
            };
            let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                continue;
            };
            if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                continue;
            }
            let method = request.get("method").and_then(Value::as_string).unwrap();
            let query = request
                .get("params")
                .and_then(Value::as_array)
                .and_then(|params| params.first())
                .and_then(Value::as_string);
            let result = if method == "query" {
                let query = query.unwrap();
                entered.send(query.clone()).unwrap();
                let mut statement = Object::new();
                statement.insert("status", "OK");
                statement.insert("time", "0ns");
                if query == "RETURN 'CHARGED_RESPONSE'" {
                    released.take().unwrap().await.unwrap();
                    statement.insert("result", surrealdb::types::Bytes::from(vec![7_u8; 8192]));
                } else {
                    statement.insert("result", true);
                }
                Value::Array(vec![Value::Object(statement)].into())
            } else if method == "version" {
                Value::String("3.3.0".into())
            } else {
                Value::None
            };
            let mut reply = Object::new();
            reply.insert("id", request.get("id").unwrap().clone());
            reply.insert("session", request.get("session").unwrap().clone());
            reply.insert("result", result);
            socket
                .send(Message::Binary(
                    surrealdb::types::encode(&Value::Object(reply))
                        .unwrap()
                        .into(),
                ))
                .await
                .unwrap();
        }
    });
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("ws://{address}"),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let sdk = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap();
    let client = Arc::new(super::CanonicalClient::new(
        sdk,
        Duration::from_secs(5),
        &options,
    ));
    // Three distinct actual grants exercise enclosing response ownership,
    // nested response ownership and an existing outbound payload owner.
    const GRANT: usize = 64 * 1024;
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(3 * GRANT));
    let grant = || {
        let owner = MemoryConsumer::new("unit:canonical-response").register(&pool);
        owner.try_grow(GRANT).unwrap();
        Arc::new(owner)
    };
    let (enclosing, nested, outbound) = (grant(), grant(), grant());
    // The response scope carries the original strong protection witness as well
    // as its grant. A copied identity or connection activity is insufficient.
    let ticket = Arc::new("original exact read ticket".to_owned());
    let original_ticket = Arc::downgrade(&ticket);
    let enclosing = Arc::new((ticket, enclosing));
    let active = client.clone();
    let caller = tokio::spawn(async move {
        let original = Instant::now() + Duration::from_secs(5);
        super::operation_scope(original, async {
            super::response_ownership_scope(enclosing, async {
                super::response_ownership_scope(nested, async {
                    assert_eq!(
                        super::original_deadline(Duration::from_secs(30)).into_std(),
                        original
                    );
                    let request = async move {
                        if second_metadata_rpc {
                            let mut first = active.query("RETURN 'FIRST_METADATA'").await?;
                            assert_eq!(first.take::<Value>(0)?, Value::Bool(true));
                        }
                        // The query's existing payload owner must compose with
                        // both response scopes rather than replace either grant.
                        let mut response = active
                            .query("RETURN 'CHARGED_RESPONSE'")
                            .payload_owner(outbound.clone())
                            .await?;
                        let Value::Bytes(bytes) = response.take::<Value>(0)? else {
                            panic!("charged response differs from original native bytes");
                        };
                        assert_eq!(bytes.len(), 8192);
                        drop(bytes);
                        Ok::<_, surrealdb::Error>(())
                    };
                    if spawned_acquisition {
                        tokio::spawn(super::inherit_response_ownership(request))
                            .await
                            .unwrap()
                    } else {
                        request.await
                    }
                })
                .await
            })
            .await
        })
        .await
    });
    if second_metadata_rpc {
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), requests.recv())
                .await
                .unwrap()
                .unwrap(),
            "RETURN 'FIRST_METADATA'"
        );
    }
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), requests.recv())
            .await
            .unwrap()
            .unwrap(),
        "RETURN 'CHARGED_RESPONSE'"
    );
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    assert_eq!(
        pool.reserved(),
        3 * GRANT,
        "original request grants must survive caller abandonment"
    );
    assert!(
        original_ticket.upgrade().is_some(),
        "the original strong read ticket must survive caller abandonment"
    );
    let next = MemoryConsumer::new("unit:canonical-response-reuse").register(&pool);
    assert!(
        next.try_grow(1).is_err(),
        "late native Value cannot use reused capacity"
    );
    resume.send(()).unwrap();
    // This later ACK crosses the same physical driver's ordered response loop.
    // SDK controls separately prove the inner raw-frame/failed-send drop order.
    let mut sentinel = client.control_query("CONTROL").await.unwrap();
    assert_eq!(sentinel.take::<Value>(0).unwrap(), Value::Bool(true));
    tokio::time::timeout(Duration::from_secs(2), async {
        while pool.reserved() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(
        original_ticket.upgrade().is_none(),
        "the exact read ticket must release after the actual late frame drains"
    );
    next.try_grow(3 * GRANT).unwrap();
    drop(next);
    assert_eq!(pool.reserved(), 0);
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn canonical_response_owner_survives_cancelled_caller_until_actual_late_frame_drains() {
    response_ownership_late_frame(false, false).await;
}

#[tokio::test]
async fn canonical_response_owner_crosses_spawned_acquisition_without_replacing_nested_grants() {
    response_ownership_late_frame(true, false).await;
}

#[tokio::test]
async fn canonical_response_owner_retains_exact_ticket_across_sequential_metadata_rpcs() {
    response_ownership_late_frame(false, true).await;
}

#[tokio::test]
async fn native_sdk_rejected_attachment_preserves_exact_method_error() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
        while let Some(Ok(Message::Binary(bytes))) = socket.next().await {
            let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                continue;
            };
            if request
                .get("method")
                .and_then(Value::as_string)
                .map(String::as_str)
                != Some("attach")
            {
                continue;
            }
            let mut reply = Object::new();
            reply.insert("id", request.get("id").unwrap().clone());
            reply.insert("session", request.get("session").unwrap().clone());
            use surrealdb::types::SurrealValue;
            reply.insert(
                "error",
                surrealdb::Error::not_allowed(
                    "Method not allowed".into(),
                    surrealdb::types::NotAllowedError::Method {
                        name: "attach".into(),
                    },
                )
                .into_value(),
            );
            socket
                .send(Message::Binary(
                    surrealdb::types::encode(&Value::Object(reply))
                        .unwrap()
                        .into(),
                ))
                .await
                .unwrap();
        }
    });
    let error = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap_err();
    assert!(
        matches!(error.not_allowed_details(), Some(surrealdb::types::NotAllowedError::Method { name }) if name == "attach")
    );
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn native_sdk_full_encoded_envelope_limit_refuses_before_dispatch() {
    let limit = 256 * 1024;
    let (client, mut received, server) = peer_with_limit(true, limit).await;
    let context = RequestContext::new(Instant::now() + Duration::from_secs(2));
    // The SQL text is tiny and the bound bytes alone fit exactly. The encoded
    // native RPC envelope includes binding keys, IDs and session, exceeding cap.
    let oversized = client
        .query("CONTROL")
        .bind(("payload", surrealdb::types::Bytes::from(vec![0_u8; limit])))
        .request_context(context.clone())
        .await
        .unwrap_err();
    assert!(oversized.to_string().contains("Message too long"));
    assert_eq!(context.dispatch_state(), DispatchState::NotDispatched);
    assert!(!context.was_dispatched());
    let mut result = client
        .query("CONTROL")
        .request_context(RequestContext::new(Instant::now() + Duration::from_secs(2)))
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(result.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(next_query(&mut received).await, "CONTROL");
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    while let Ok(message) = received.try_recv() {
        assert!(matches!(message.as_str(), "attach" | "ping" | "version"));
    }
}

/// Reconnect after repeated finalized selection and observe the actual retained
/// wire order. Optional setup between checkpoints must survive under its owner.
async fn selection_checkpoint_replay(intervening_variable: bool, refuse_final_owner: bool) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent, mut received) = mpsc::unbounded_channel();
    let server = tokio::spawn(async move {
        let mut owners = 0;
        for connection in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
            while let Some(Ok(Message::Binary(bytes))) = socket.next().await {
                let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                    continue;
                };
                let method = request.get("method").and_then(Value::as_string).unwrap();
                let first = request
                    .get("params")
                    .and_then(Value::as_array)
                    .and_then(|params| params.first());
                let label = if method == "signin" {
                    let username = first
                        .and_then(Value::as_object)
                        .and_then(|credentials| credentials.get("user"))
                        .and_then(Value::as_string)
                        .unwrap();
                    format!("signin:{username}")
                } else if method == "query" {
                    first.and_then(Value::as_string).unwrap().clone()
                } else {
                    method.clone()
                };
                sent.send(label.clone()).unwrap();
                if label == "RECONNECT" && connection == 0 {
                    socket.close(None).await.unwrap();
                    break;
                }
                if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                    continue;
                }
                let mut reply = Object::new();
                reply.insert("id", request.get("id").unwrap().clone());
                reply.insert("session", request.get("session").unwrap().clone());
                let result = if method == "signin" {
                    Value::String("synthetic-access-token".into())
                } else if method == "version" {
                    Value::String("3.3.0".into())
                } else if method == "query" {
                    let mut statement = Object::new();
                    statement.insert("status", "OK");
                    statement.insert("time", "0ns");
                    statement.insert("result", true);
                    Value::Array(vec![Value::Object(statement)].into())
                } else {
                    Value::None
                };
                if connection == 0 && label == "signin:owner" {
                    owners += 1;
                }
                if refuse_final_owner && connection == 0 && owners == 2 && label == "signin:owner" {
                    use surrealdb::types::SurrealValue;
                    reply.insert(
                        "error",
                        surrealdb::Error::internal("rejected owner checkpoint".into()).into_value(),
                    );
                } else {
                    reply.insert("result", result);
                }
                socket
                    .send(Message::Binary(
                        surrealdb::types::encode(&Value::Object(reply))
                            .unwrap()
                            .into(),
                    ))
                    .await
                    .unwrap();
            }
        }
    });
    let client = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap();
    let identity = uuid::Uuid::new_v4();
    for iteration in 0..80 {
        let clock = || Instant::now() + Duration::from_secs(2);
        client
            .signin(surrealdb::opt::auth::Root {
                username: "viewer".into(),
                password: "test".into(),
            })
            .request_context(RequestContext::control(clock()))
            .await
            .unwrap();
        assert_eq!(next_query(&mut received).await, "signin:viewer");
        client
            .use_ns("namespace")
            .use_db("database")
            .request_context(RequestContext::control(clock()))
            .await
            .unwrap();
        assert_eq!(next_query(&mut received).await, "use");
        let owner = client
            .signin(surrealdb::opt::auth::Root {
                username: "owner".into(),
                password: "test".into(),
            })
            .request_context(RequestContext::selection_checkpoint(clock(), identity))
            .await;
        assert_eq!(next_query(&mut received).await, "signin:owner");
        if refuse_final_owner && iteration == 1 {
            assert!(owner.is_err());
            break;
        }
        owner.unwrap();
        if iteration == 0 && intervening_variable {
            client
                .set("owner_variable", true)
                .request_context(RequestContext::control(clock()))
                .await
                .unwrap();
            assert_eq!(next_query(&mut received).await, "let");
        }
    }
    assert!(
        client
            .query("RECONNECT")
            .request_context(RequestContext::new(Instant::now() + Duration::from_secs(2)))
            .await
            .is_err()
    );
    assert_eq!(next_query(&mut received).await, "RECONNECT");
    // A later application request waits until every retained setup ACK arrives.
    let mut result = client
        .query("CONTROL")
        .request_context(RequestContext::new(Instant::now() + Duration::from_secs(5)))
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(result.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(next_query(&mut received).await, "signin:viewer");
    assert_eq!(next_query(&mut received).await, "use");
    assert_eq!(next_query(&mut received).await, "signin:owner");
    if refuse_final_owner {
        assert_eq!(next_query(&mut received).await, "signin:viewer");
        assert_eq!(next_query(&mut received).await, "use");
    }
    if intervening_variable {
        assert_eq!(next_query(&mut received).await, "let");
        assert_eq!(next_query(&mut received).await, "signin:viewer");
        assert_eq!(next_query(&mut received).await, "use");
        assert_eq!(next_query(&mut received).await, "signin:owner");
    }
    assert_eq!(next_query(&mut received).await, "CONTROL");
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn native_sdk_selection_checkpoint_bounds_repeated_setup_and_preserves_replay_order() {
    selection_checkpoint_replay(false, false).await;
}

#[tokio::test]
async fn native_sdk_selection_checkpoint_preserves_intervening_owner_variable() {
    selection_checkpoint_replay(true, false).await;
}

#[tokio::test]
async fn native_sdk_selection_checkpoint_rejected_owner_retains_prior_and_partial_setup() {
    selection_checkpoint_replay(false, true).await;
}

async fn canonical_selection_refuses_unacknowledged_owner(late_acknowledgment: bool) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent, mut received) = mpsc::unbounded_channel();
    let (owner_arrived, mut owner_arrival) = tokio::sync::oneshot::channel();
    let (release_owner, owner_release) = tokio::sync::oneshot::channel();
    let (owner_written, owner_write) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let mut owner_arrived = Some(owner_arrived);
        let mut owner_release = Some(owner_release);
        let mut owner_written = Some(owner_written);
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
        while let Some(Ok(Message::Binary(bytes))) = socket.next().await {
            let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                continue;
            };
            let method = request.get("method").and_then(Value::as_string).unwrap();
            if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                continue;
            }
            sent.send(method.clone()).unwrap();
            let first = request
                .get("params")
                .and_then(Value::as_array)
                .and_then(|params| params.first());
            let owner = method == "signin"
                && first
                    .and_then(Value::as_object)
                    .and_then(|credentials| credentials.get("user"))
                    .and_then(Value::as_string)
                    .is_some_and(|username| username == "owner");
            if owner {
                owner_arrived.take().unwrap().send(()).unwrap();
            }
            let mut reply = Object::new();
            reply.insert("id", request.get("id").unwrap().clone());
            reply.insert("session", request.get("session").unwrap().clone());
            if owner && !late_acknowledgment {
                use surrealdb::types::SurrealValue;
                reply.insert(
                    "error",
                    surrealdb::Error::internal("owner acknowledgment refused".into()).into_value(),
                );
            } else {
                if owner {
                    owner_release.take().unwrap().await.unwrap();
                }
                let result = if method == "signin" {
                    Value::String("synthetic-access-token".into())
                } else if method == "version" {
                    Value::String("3.3.0".into())
                } else if method == "query" {
                    let mut statement = Object::new();
                    statement.insert("status", "OK");
                    statement.insert("time", "0ns");
                    statement.insert("result", Value::None);
                    Value::Array(vec![Value::Object(statement)].into())
                } else {
                    Value::None
                };
                reply.insert("result", result);
            }
            socket
                .send(Message::Binary(
                    surrealdb::types::encode(&Value::Object(reply))
                        .unwrap()
                        .into(),
                ))
                .await
                .unwrap();
            if owner {
                owner_written.take().unwrap().send(()).unwrap();
            }
        }
    });
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("ws://{address}"),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let sdk = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap();
    // Ordinary peer completion and deliberate expiry have distinct fault models.
    // The late peer holds the final acknowledgment until the original clock expires.
    let client = super::CanonicalClient::new(sdk, Duration::from_secs(2), &options);
    let first = client.select_existing().await.unwrap_err();
    assert!(
        owner_arrival.try_recv().is_ok(),
        "selection failed before reaching the intended Owner acknowledgment: {first:?}"
    );
    if late_acknowledgment {
        assert!(matches!(
            first,
            crate::canonical::CanonicalError::Timeout | crate::canonical::CanonicalError::Driver(_)
        ));
    } else {
        assert!(
            matches!(&first, crate::canonical::CanonicalError::Driver(_))
                && first.to_string().contains("owner acknowledgment refused"),
            "expected the peer's acknowledged Owner refusal: {first:?}"
        );
    }
    assert_eq!(
        *client.selection.lock().await,
        (false, false),
        "observed presence is not cached before final Owner acknowledgment"
    );
    assert!(
        client
            .query("FORBIDDEN_BEFORE_OWNER_ACK")
            .await
            .unwrap_err()
            .to_string()
            .contains("not dispatched")
    );
    // Even an eventual successful transport acknowledgment cannot repair the
    // caller's abandoned authorization transition or admit later mutations.
    if late_acknowledgment {
        release_owner.send(()).unwrap();
    }
    tokio::time::timeout(Duration::from_secs(2), owner_write)
        .await
        .unwrap()
        .unwrap();
    assert!(
        client
            .select_existing()
            .await
            .unwrap_err()
            .to_string()
            .contains("authorization is uncertain")
    );
    assert!(
        client
            .query("FORBIDDEN_AFTER_OWNER_ACK")
            .await
            .unwrap_err()
            .to_string()
            .contains("not dispatched")
    );
    assert_eq!(*client.selection.lock().await, (false, false));
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    let methods: Vec<_> = std::iter::from_fn(|| received.try_recv().ok())
        .filter(|method| !matches!(method.as_str(), "attach" | "ping" | "version"))
        .collect();
    assert_eq!(methods, ["signin", "use", "query", "signin"]);
}

#[tokio::test]
async fn canonical_selection_rejected_owner_ack_poisoned_handle_refuses_cached_presence_and_mutations()
 {
    canonical_selection_refuses_unacknowledged_owner(false).await;
}

#[tokio::test]
async fn canonical_selection_late_owner_ack_cannot_unpoison_cancelled_handle() {
    canonical_selection_refuses_unacknowledged_owner(true).await;
}

/// One real FlatBuffers connection whose first two read replies have a peer-side
/// barrier. A serialized canonical read path cannot complete either read.
pub(crate) async fn canonical_read_barrier_peer() -> (
    crate::canonical::CanonicalStore,
    mpsc::UnboundedReceiver<String>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent, received) = mpsc::unbounded_channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
        let mut withheld = Vec::new();
        while let Some(Ok(message)) = socket.next().await {
            let Message::Binary(bytes) = message else {
                continue;
            };
            let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                continue;
            };
            if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                continue;
            }
            let method = request.get("method").and_then(Value::as_string).unwrap();
            let query = request
                .get("params")
                .and_then(Value::as_array)
                .and_then(|params| params.first())
                .and_then(Value::as_string);
            let mut reply = Object::new();
            reply.insert("id", request.get("id").unwrap().clone());
            reply.insert("session", request.get("session").unwrap().clone());
            if method == "query" {
                let query = query.unwrap();
                sent.send(query.clone()).unwrap();
                let mut statement = Object::new();
                statement.insert("status", "OK");
                statement.insert("time", "0ns");
                statement.insert("result", true);
                reply.insert(
                    "result",
                    Value::Array(vec![Value::Object(statement)].into()),
                );
                if matches!(
                    query.as_str(),
                    "RETURN 'FIRST_READ'" | "RETURN 'SECOND_READ'"
                ) {
                    withheld.push(reply);
                    if withheld.len() != 2 {
                        continue;
                    }
                    // Neither reply is sent until two query envelopes arrived.
                    // This is a dispatch barrier, independent of elapsed duration.
                    for ready in withheld.drain(..) {
                        socket
                            .send(Message::Binary(
                                surrealdb::types::encode(&Value::Object(ready))
                                    .unwrap()
                                    .into(),
                            ))
                            .await
                            .unwrap();
                    }
                    continue;
                }
            } else if method == "version" {
                reply.insert("result", Value::String("3.3.0".into()));
            } else {
                reply.insert("result", Value::None);
            }
            socket
                .send(Message::Binary(
                    surrealdb::types::encode(&Value::Object(reply))
                        .unwrap()
                        .into(),
                ))
                .await
                .unwrap();
        }
        assert!(
            withheld.is_empty(),
            "both protected reads must cross the dispatch barrier"
        );
    });
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("ws://{address}"),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let sdk = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap();
    let client = Arc::new(super::CanonicalClient::new(
        sdk,
        Duration::from_secs(2),
        &options,
    ));
    // Test-only assembly leaves service/catalog installation outside this wire
    // control. Reads still traverse CanonicalStore::protected_query, ProblemTurns,
    // bounded response checking, CanonicalClient and the actual bounded SDK driver.
    let store = crate::canonical::CanonicalStore {
        db: client.clone(),
        activation_db: client,
        problem_turns: Arc::default(),
        fixture_lifetime: None,
        issuing_store: Arc::default(),
        files: Arc::new(crate::canonical_files::FileSession::wire_control().unwrap()),
        result_read_drain: Arc::default(),
        connection_options: options.clone(),
        state_path: options.state_path.clone(),
        database: options.database,
        namespace: options.namespace,
    };
    (store, received, server)
}

#[tokio::test]
async fn canonical_protected_reads_cross_actual_dispatch_barrier_and_expired_queue_never_dispatches()
 {
    use crate::canonical::{CanonicalError, QueryEffect};
    use std::{
        future::Future,
        sync::atomic::{AtomicBool, Ordering},
        task::Poll,
    };
    let (store, mut received, server) = canonical_read_barrier_peer().await;
    let first = store.protected_query(
        "problem",
        QueryEffect::Read,
        "canonical::barrier_first",
        || Ok(store.db.query("RETURN 'FIRST_READ';")),
    );
    let second = store.protected_query(
        "problem",
        QueryEffect::Read,
        "canonical::barrier_second",
        || Ok(store.db.query("RETURN 'SECOND_READ';")),
    );
    let (mut first, mut second) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::try_join!(first, second)
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(first.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(second.take::<Value>(0).unwrap(), Value::Bool(true));
    let mut dispatched = [
        next_query(&mut received).await,
        next_query(&mut received).await,
    ];
    dispatched.sort();
    assert_eq!(dispatched, ["RETURN 'FIRST_READ'", "RETURN 'SECOND_READ'"]);

    let held = store
        .problem_turns
        .gate("problem")
        .unwrap()
        .write_owned()
        .await;
    let original = tokio::time::Instant::now() + Duration::from_millis(20);
    let context = RequestContext::new(original.into_std());
    let built = AtomicBool::new(false);
    let mut expired = Box::pin(super::within_clock(
        original,
        store.protected_query(
            "problem",
            QueryEffect::Read,
            "canonical::expired_queued_read",
            || {
                built.store(true, Ordering::SeqCst);
                Ok(store
                    .db
                    .query("RETURN 'EXPIRED_READ';")
                    .request_context(context.clone()))
            },
        ),
    ));
    // Establish that this read really joined the writer's local queue before
    // its original clock expires; do not substitute an already-expired call.
    std::future::poll_fn(|cx| {
        assert!(expired.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert!(!built.load(Ordering::SeqCst));
    assert!(matches!(expired.await, Err(CanonicalError::Timeout)));
    assert!(!built.load(Ordering::SeqCst));
    assert_eq!(context.dispatch_state(), DispatchState::NotDispatched);
    drop(held);
    let mut after = store
        .protected_query(
            "problem",
            QueryEffect::Read,
            "canonical::after_expiry",
            || Ok(store.db.query("RETURN 'AFTER_EXPIRY';")),
        )
        .await
        .unwrap();
    assert_eq!(after.take::<Value>(0).unwrap(), Value::Bool(true));
    // This acknowledged later query is an ordered wire sentinel: an accidentally
    // dispatched expired query would precede it in the peer's observed messages.
    assert_eq!(next_query(&mut received).await, "RETURN 'AFTER_EXPIRY'");
    store.db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert!(received.try_recv().is_err());
}

#[tokio::test]
async fn canonical_conflict_rebuild_releases_local_turn_and_keeps_original_clock() {
    use crate::canonical::{CanonicalError, QueryEffect};
    use std::{
        future::{Future, Ready},
        sync::atomic::{AtomicUsize, Ordering},
        task::Poll,
    };
    let (store, mut received, server) = canonical_read_barrier_peer().await;
    let calls = AtomicUsize::new(0);
    let original = tokio::time::Instant::now() + Duration::from_millis(20);
    let mut pending=Box::pin(super::within_clock(original,store.protected_query("pin:exact",QueryEffect::Mutation,"canonical::conflict_clock",|| -> Result<Ready<Result<surrealdb::IndexedResults,surrealdb::Error>>,CanonicalError> {
        calls.fetch_add(1,Ordering::SeqCst);
        Err(CanonicalError::Driver(surrealdb::Error::query("definite rejection".into(),surrealdb::types::QueryError::TransactionConflict)))
    })));
    std::future::poll_fn(|cx| {
        assert!(pending.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let held = store
        .problem_turns
        .gate("pin:exact")
        .unwrap()
        .try_write_owned()
        .expect("conflict backoff releases the completed RPC turn");
    drop(held);
    assert!(matches!(pending.await, Err(CanonicalError::Timeout)));
    let uncertain = AtomicUsize::new(0);
    let result=store.protected_query("pin:exact",QueryEffect::Mutation,"canonical::uncertain_once",|| -> Result<Ready<Result<surrealdb::IndexedResults,surrealdb::Error>>,CanonicalError> {
        uncertain.fetch_add(1,Ordering::SeqCst); Err(CanonicalError::Timeout)
    }).await;
    assert!(matches!(result, Err(CanonicalError::Timeout)));
    assert_eq!(
        uncertain.load(Ordering::SeqCst),
        1,
        "uncertain mutation completion never becomes an application replay"
    );
    assert!(
        received.try_recv().is_err(),
        "expired or unbuilt work never dispatches"
    );
    server.abort();
}

#[tokio::test]
async fn canonical_closed_file_caller_cannot_start_publication_or_collection() {
    let (store, mut received, server) = canonical_read_barrier_peer().await;
    let instance = uuid::Uuid::new_v4().to_string();
    let intent:crate::canonical_files::FilePublicationIntent=serde_json::from_value(serde_json::json!({"instance":instance,"intent":"original-file","request_digest":"0".repeat(64),"owner":"original-owner","storage_incarnation":store.files.incarnation})).unwrap();
    let path = store.files.path(intent.instance()).unwrap();
    store.close();
    assert!(matches!(
        store
            .seal_file(&intent, b"[1]", crate::canonical_files::OPAQUE_JSON, 0, "")
            .await,
        Err(crate::canonical::CanonicalError::CallerClosed)
    ));
    assert!(matches!(
        store.collect_file(intent.instance()).await,
        Err(crate::canonical::CanonicalError::CallerClosed)
    ));
    assert!(!path.exists());
    assert!(received.try_recv().is_err());
    store.db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert!(received.try_recv().is_err());
}

// The peer commits registration but withholds its first acknowledgement. Public
// collection cannot proceed past owner registration until an exact replay ACK.
async fn file_owner_registration_peer() -> (
    crate::canonical::CanonicalStore,
    String,
    mpsc::UnboundedReceiver<Object>,
    tokio::sync::oneshot::Sender<()>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let files = Arc::new(crate::canonical_files::FileSession::wire_control().unwrap());
    let incarnation = files.incarnation.clone();
    let instance = uuid::Uuid::new_v4().to_string();
    let selected = instance.clone();
    let (sent, received) = mpsc::unbounded_channel();
    let (release, released) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
        let mut released = Some(released);
        let mut original: Option<Object> = None;
        let mut drained = false;
        while let Some(Ok(Message::Binary(bytes))) = socket.next().await {
            let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                continue;
            };
            if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                continue;
            }
            let method = request.get("method").and_then(Value::as_string).unwrap();
            let result = if method == "query" {
                let params = request.get("params").and_then(Value::as_array).unwrap();
                let query = params[0].as_string().unwrap();
                let variables = params[1].as_object().unwrap();
                let (value, transaction) = if query.contains("register_owner(") {
                    let row = variables.get("row").unwrap().as_object().unwrap().clone();
                    assert!(!drained);
                    let first = original.is_none();
                    if let Some(expected) = &original {
                        assert_eq!(
                            &row, expected,
                            "retry must settle the exact original registration"
                        );
                    } else {
                        original = Some(row.clone());
                    }
                    sent.send(row.clone()).unwrap();
                    if first {
                        released.take().unwrap().await.unwrap();
                    }
                    (Value::Object(row), true)
                } else if query.contains("drain_owner(") {
                    let expected = variables.get("expected").unwrap().as_object().unwrap();
                    assert_eq!(Some(expected), original.as_ref());
                    drained = true;
                    sent.send(expected.clone()).unwrap();
                    (Value::Bool(true), true)
                } else if query.contains("SELECT key,state,storage_incarnation") {
                    let mut row = Object::new();
                    row.insert("key", selected.clone());
                    row.insert("state", "attached");
                    row.insert("storage_incarnation", incarnation.clone());
                    (Value::Object(row), false)
                } else if query.contains("fn::pse_files_v1::collect(") {
                    assert!(original.is_some() && !drained);
                    let mut row = Object::new();
                    row.insert("state", "collected");
                    (Value::Object(row), true)
                } else {
                    panic!("unexpected file-owner query: {query}");
                };
                let statement = |value| {
                    let mut row = Object::new();
                    row.insert("status", "OK");
                    row.insert("time", "0ns");
                    row.insert("result", value);
                    Value::Object(row)
                };
                let statements = if transaction {
                    vec![
                        statement(Value::None),
                        statement(value),
                        statement(Value::None),
                    ]
                } else {
                    vec![statement(value)]
                };
                Value::Array(statements.into())
            } else if method == "version" {
                Value::String("3.3.0".into())
            } else {
                Value::None
            };
            let mut reply = Object::new();
            reply.insert("id", request.get("id").unwrap().clone());
            reply.insert("session", request.get("session").unwrap().clone());
            reply.insert("result", result);
            socket
                .send(Message::Binary(
                    surrealdb::types::encode(&Value::Object(reply))
                        .unwrap()
                        .into(),
                ))
                .await
                .unwrap();
        }
        assert!(
            drained,
            "physical disconnect must drain the retained original registration"
        );
    });
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("ws://{address}"),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let sdk = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap();
    let client = Arc::new(super::CanonicalClient::new(
        sdk,
        Duration::from_secs(2),
        &options,
    ));
    let issuing_store = Arc::new(std::sync::OnceLock::new());
    issuing_store.set(uuid::Uuid::new_v4().to_string()).unwrap();
    let store = crate::canonical::CanonicalStore {
        db: client.clone(),
        activation_db: client,
        problem_turns: Arc::default(),
        fixture_lifetime: None,
        issuing_store,
        files,
        result_read_drain: Arc::default(),
        connection_options: options.clone(),
        state_path: options.state_path.clone(),
        database: options.database,
        namespace: options.namespace,
    };
    (store, instance, received, release, server)
}

async fn original_file_owner_survives_abandoned_ack(cancelled: bool, retry: bool) {
    let (store, instance, mut requests, release, server) = file_owner_registration_peer().await;
    let caller = store.clone();
    let selected = instance.clone();
    let submitted = tokio::spawn(async move {
        super::within_clock(
            tokio::time::Instant::now() + Duration::from_millis(300),
            caller.collect_file(&selected),
        )
        .await
    });
    let original = tokio::time::timeout(Duration::from_secs(2), requests.recv())
        .await
        .unwrap()
        .unwrap();
    if cancelled {
        submitted.abort();
        assert!(submitted.await.unwrap_err().is_cancelled());
    } else {
        assert!(submitted.await.unwrap().is_err());
    }
    assert_eq!(
        store.files.registered_owner().await,
        Some(original.clone()),
        "unknown ACK retains the original row for retry and teardown"
    );
    release.send(()).unwrap();
    if retry {
        store.collect_file(&instance).await.unwrap();
        assert_eq!(requests.recv().await.unwrap(), original);
    }
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match store.disconnect().await {
                Ok(()) => break,
                Err(crate::canonical::CanonicalError::ConnectionBusy { .. }) => {
                    tokio::task::yield_now().await
                }
                Err(error) => panic!("pending original drain failed: {error}"),
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(requests.recv().await.unwrap(), original);
    assert!(requests.try_recv().is_err());
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    let path = store.files.path(&instance).unwrap();
    let root = path.parent().unwrap().parent().unwrap();
    if root.exists() {
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[tokio::test]
async fn canonical_file_owner_lost_ack_retries_exact_original_then_disconnects() {
    original_file_owner_survives_abandoned_ack(false, true).await;
}
#[tokio::test]
async fn canonical_file_owner_cancelled_ack_disconnects_pending_original() {
    original_file_owner_survives_abandoned_ack(true, false).await;
}

#[derive(Debug, Default)]
struct StatisticsProbe {
    samples: std::sync::Mutex<Vec<(usize, Option<Duration>, bool)>>,
    panic_first: bool,
    failed_requests: std::sync::atomic::AtomicUsize,
}
impl surrealdb::method::QueryStatisticsObserver for StatisticsProbe {
    fn request_failed(&self) {
        self.failed_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(
            !self.panic_first,
            "controlled optional failure-observer panic"
        );
    }
    fn observe(&self, statement: usize, stats: surrealdb::method::Stats, succeeded: bool) {
        self.samples
            .lock()
            .unwrap()
            .push((statement, stats.execution_time, succeeded));
        assert!(
            !self.panic_first || statement != 0,
            "controlled optional observer panic"
        );
    }
}

#[tokio::test]
async fn native_sdk_statistics_observe_actual_response_before_take_and_contain_panics() {
    for panic_first in [false, true] {
        let (client, mut received, server) = peer(true).await;
        let probe = Arc::new(StatisticsProbe {
            panic_first,
            ..StatisticsProbe::default()
        });
        let mut result = client
            .query("STATISTICS")
            .request_context(RequestContext::new(Instant::now() + Duration::from_secs(5)))
            .observe_statistics(probe.clone())
            .bind(("unused", true))
            .await
            .unwrap();
        assert_eq!(
            *probe.samples.lock().unwrap(),
            vec![
                (0, Some(Duration::from_nanos(37)), true),
                (1, Some(Duration::from_nanos(91)), false)
            ]
        );
        let value: Option<bool> = result.take(0).unwrap();
        assert_eq!(value, Some(true));
        assert!(
            result.check().is_err(),
            "statement failure remains scientific result failure"
        );
        assert!(
            client
                .query("REQUEST_FAILURE")
                .request_context(RequestContext::new(Instant::now() + Duration::from_secs(5)))
                .observe_statistics(probe.clone())
                .await
                .is_err()
        );
        assert_eq!(
            probe
                .failed_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            probe.samples.lock().unwrap().len(),
            2,
            "request failure fabricates no statements"
        );
        client
            .query("CONTROL")
            .request_context(RequestContext::control(
                Instant::now() + Duration::from_secs(5),
            ))
            .await
            .unwrap()
            .check()
            .unwrap();
        assert_eq!(next_query(&mut received).await, "STATISTICS");
        assert_eq!(next_query(&mut received).await, "REQUEST_FAILURE");
        assert_eq!(
            next_query(&mut received).await,
            "CONTROL",
            "statistics never replay the query"
        );
        assert!(received.try_recv().is_err());
        client.disconnect().await.unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn canonical_statistics_keep_server_samples_out_of_local_active_intervals() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value as ObservedValue};
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("canonical-statistics");
    let observer = super::CanonicalStatistics {
        context: operation.context(),
        invocation: 42,
    };
    let mut stats = surrealdb::method::Stats::default();
    stats.execution_time = Some(Duration::from_nanos(91));
    surrealdb::method::QueryStatisticsObserver::observe(&observer, 3, stats, false);
    let snapshot = operation.reader().snapshot(Duration::from_secs(60));
    let sample = snapshot
        .metrics
        .iter()
        .find(|m| m.name == "canonical.query.latest_sample")
        .unwrap();
    let ObservedValue::Text(value) = &sample.latest else {
        panic!("typed sample expected");
    };
    assert_eq!(
        serde_json::from_str::<super::CanonicalQuerySample>(value).unwrap(),
        super::CanonicalQuerySample::Statement {
            invocation: 42,
            statement: 3,
            server_nanoseconds: Some(91),
            succeeded: false,
        }
    );
    assert!(matches!(sample.cumulative, ObservedValue::Unavailable(_)));
    assert!(matches!(sample.recent, ObservedValue::Unavailable(_)));
    assert_eq!(snapshot.sources.len(), 1);
}

#[tokio::test]
async fn canonical_client_attaches_statistics_to_the_original_query() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value as ObservedValue, scope};
    let (sdk, mut received, server) = peer(true).await;
    let options = crate::canonical::CanonicalOptions {
        endpoint: String::new(),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let client = super::CanonicalClient::new(
        Arc::try_unwrap(sdk).unwrap(),
        Duration::from_secs(5),
        &options,
    );
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("actual-canonical-query");
    let result = scope(Some(operation.context()), async {
        client.query("STATISTICS").await
    })
    .await
    .unwrap();
    assert!(result.check().is_err());
    let snapshot = operation.reader().snapshot(Duration::from_secs(60));
    let sample = snapshot
        .metrics
        .iter()
        .find(|m| m.name == "canonical.query.latest_sample")
        .unwrap();
    let read_sample = |value: &ObservedValue| {
        let ObservedValue::Text(value) = value else {
            panic!("typed sample expected");
        };
        serde_json::from_str::<super::CanonicalQuerySample>(value).unwrap()
    };
    let first = read_sample(&sample.history[0].first_value);
    let latest = read_sample(&sample.latest);
    let super::CanonicalQuerySample::Statement {
        invocation,
        statement: 0,
        server_nanoseconds: Some(37),
        succeeded: true,
    } = first
    else {
        panic!("actual first statement expected: {first:?}");
    };
    assert_eq!(
        latest,
        super::CanonicalQuerySample::Statement {
            invocation,
            statement: 1,
            server_nanoseconds: Some(91),
            succeeded: false
        }
    );
    client.query("CONTROL").await.unwrap().check().unwrap();
    assert_eq!(next_query(&mut received).await, "STATISTICS");
    assert_eq!(next_query(&mut received).await, "CONTROL");
    client.client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[test]
fn canonical_statistics_bound_bursts_without_displacing_attempt_identity() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value as ObservedValue};
    use surrealdb::method::QueryStatisticsObserver;
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("canonical-statistics-burst");
    for invocation in 0..1000 {
        let observer = super::CanonicalStatistics {
            context: operation.context(),
            invocation,
        };
        let mut stats = surrealdb::method::Stats::default();
        stats.execution_time = Some(Duration::from_nanos(invocation));
        observer.observe(3, stats, true);
    }
    operation
        .context()
        .source_ref("canonical.attempt", "exact-attempt");
    let snapshot = operation.reader().snapshot(Duration::from_secs(60));
    assert_eq!(
        snapshot.metrics.len(),
        4,
        "field catalogue does not grow with query count"
    );
    assert_eq!(
        snapshot.sources.len(),
        2,
        "one origin leaves room for attempt identity"
    );
    assert!(
        snapshot
            .sources
            .iter()
            .any(|source| source.name == "canonical.attempt")
    );
    let sample = snapshot
        .metrics
        .iter()
        .find(|m| m.name == "canonical.query.latest_sample")
        .unwrap();
    let ObservedValue::Text(value) = &sample.latest else {
        panic!("typed sample expected");
    };
    assert_eq!(
        serde_json::from_str::<super::CanonicalQuerySample>(value).unwrap(),
        super::CanonicalQuerySample::Statement {
            invocation: 999,
            statement: 3,
            server_nanoseconds: Some(999),
            succeeded: true
        }
    );
    assert_eq!(
        sample
            .history
            .iter()
            .map(|sample| sample.samples)
            .sum::<u64>(),
        1000
    );
    let observer = super::CanonicalStatistics {
        context: operation.context(),
        invocation: 1000,
    };
    observer.request_failed();
    let snapshot = operation.reader().snapshot(Duration::from_secs(60));
    let sample = snapshot
        .metrics
        .iter()
        .find(|m| m.name == "canonical.query.latest_sample")
        .unwrap();
    let ObservedValue::Text(value) = &sample.latest else {
        panic!("typed sample expected");
    };
    assert_eq!(
        serde_json::from_str::<super::CanonicalQuerySample>(value).unwrap(),
        super::CanonicalQuerySample::RequestFailure { invocation: 1000 }
    );
}

#[test]
fn canonical_statistics_exact_sample_fits_default_text_bound() {
    let sample = super::CanonicalQuerySample::Statement {
        invocation: u64::MAX,
        statement: usize::MAX,
        server_nanoseconds: Some(u128::MAX),
        succeeded: false,
    };
    let wire = serde_json::to_string(&sample).unwrap();
    assert!(
        wire.len() <= pse_diagnostics::observation::Policy::default().max_text_bytes,
        "{wire}"
    );
    assert_eq!(
        serde_json::from_str::<super::CanonicalQuerySample>(&wire).unwrap(),
        sample
    );
}

#[derive(Clone, Copy, Debug)]
enum MetadataFault {
    Valid,
    BlockBits,
    SingleBlockBits,
    ExcessBlockBits,
    LongBlockBits,
    LongString,
    SharedCopies,
    UnknownId,
    UnknownSession,
    DuplicateId,
    MalformedId,
}

fn metadata_block_bits(rows: usize, width: usize) -> Value {
    Value::Array(
        (0..rows)
            .map(|_| {
                let coordinate = || {
                    let mut value = Object::new();
                    value.insert("bits", Value::Bytes(vec![0; width].into()));
                    value.insert(
                        "projection",
                        Value::Number(surrealdb::types::Number::Float(0.0)),
                    );
                    Value::Object(value)
                };
                let mut row = Object::new();
                row.insert("coordinate_min", coordinate());
                row.insert("coordinate_max", coordinate());
                Value::Object(row)
            })
            .collect::<Vec<_>>()
            .into(),
    )
}

fn metadata_peer_frame(request: &Object, fault: MetadataFault) -> Vec<u8> {
    use surrealdb::types::ToFlatbuffers;
    use surrealdb_protocol::fb::v1 as fb;
    fn object<'a>(
        builder: &mut flatbuffers::FlatBufferBuilder<'a>,
        fields: &[(&str, flatbuffers::WIPOffset<fb::Value<'a>>)],
    ) -> flatbuffers::WIPOffset<fb::Value<'a>> {
        let entries = fields
            .iter()
            .map(|(key, value)| {
                let key = builder.create_string(key);
                fb::KeyValue::create(
                    builder,
                    &fb::KeyValueArgs {
                        key: Some(key),
                        value: Some(*value),
                    },
                )
            })
            .collect::<Vec<_>>();
        let items = builder.create_vector(&entries);
        let object = fb::Object::create(builder, &fb::ObjectArgs { items: Some(items) });
        fb::Value::create(
            builder,
            &fb::ValueArgs {
                value_type: fb::ValueType::Object,
                value: Some(object.as_union_value()),
            },
        )
    }
    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let Value::Number(surrealdb::types::Number::Int(actual_id)) = request.get("id").unwrap() else {
        panic!("native request id must be an integer")
    };
    let id = match fault {
        MetadataFault::UnknownId => Value::Number(surrealdb::types::Number::Int(*actual_id + 100)),
        MetadataFault::MalformedId => Value::String("not an integer".into()),
        _ => request.get("id").unwrap().clone(),
    }
    .to_fb(&mut builder)
    .unwrap();
    let session = match fault {
        MetadataFault::UnknownSession => Value::Uuid(uuid::Uuid::new_v4().into()),
        _ => request.get("session").unwrap().clone(),
    }
    .to_fb(&mut builder)
    .unwrap();
    let result = match fault {
        MetadataFault::Valid
        | MetadataFault::LongString
        | MetadataFault::SharedCopies
        | MetadataFault::BlockBits
        | MetadataFault::SingleBlockBits
        | MetadataFault::ExcessBlockBits
        | MetadataFault::LongBlockBits => {
            let value = match fault {
                MetadataFault::BlockBits => metadata_block_bits(64, 8).to_fb(&mut builder).unwrap(),
                MetadataFault::SingleBlockBits => {
                    metadata_block_bits(1, 8).to_fb(&mut builder).unwrap()
                }
                MetadataFault::ExcessBlockBits => {
                    metadata_block_bits(65, 8).to_fb(&mut builder).unwrap()
                }
                MetadataFault::LongBlockBits => {
                    metadata_block_bits(1, 9).to_fb(&mut builder).unwrap()
                }
                MetadataFault::SharedCopies => {
                    let text = builder.create_string(&"s".repeat(4096));
                    let string = fb::StringValue::create(
                        &mut builder,
                        &fb::StringValueArgs { value: Some(text) },
                    );
                    let value = fb::Value::create(
                        &mut builder,
                        &fb::ValueArgs {
                            value_type: fb::ValueType::String,
                            value: Some(string.as_union_value()),
                        },
                    );
                    let values = builder.create_vector(&[value, value, value]);
                    let array = fb::Array::create(
                        &mut builder,
                        &fb::ArrayArgs {
                            values: Some(values),
                        },
                    );
                    fb::Value::create(
                        &mut builder,
                        &fb::ValueArgs {
                            value_type: fb::ValueType::Array,
                            value: Some(array.as_union_value()),
                        },
                    )
                }
                MetadataFault::LongString => {
                    Value::String("s".repeat(4097)).to_fb(&mut builder).unwrap()
                }
                _ => Value::Bool(true).to_fb(&mut builder).unwrap(),
            };
            let status = Value::String("OK".into()).to_fb(&mut builder).unwrap();
            let time = Value::String("0ns".into()).to_fb(&mut builder).unwrap();
            let statement = object(
                &mut builder,
                &[("status", status), ("time", time), ("result", value)],
            );
            let statements = builder.create_vector(&[statement]);
            let array = fb::Array::create(
                &mut builder,
                &fb::ArrayArgs {
                    values: Some(statements),
                },
            );
            fb::Value::create(
                &mut builder,
                &fb::ValueArgs {
                    value_type: fb::ValueType::Array,
                    value: Some(array.as_union_value()),
                },
            )
        }
        _ => {
            // Generated verifier permits an absent optional StringValue field;
            // types3.3 owned conversion expects it. An accidental owned fallback
            // would panic before routing, so this is a revealing negative control.
            let string =
                fb::StringValue::create(&mut builder, &fb::StringValueArgs { value: None });
            fb::Value::create(
                &mut builder,
                &fb::ValueArgs {
                    value_type: fb::ValueType::String,
                    value: Some(string.as_union_value()),
                },
            )
        }
    };
    let mut fields = vec![("id", id), ("session", session), ("result", result)];
    if matches!(fault, MetadataFault::DuplicateId) {
        fields.push(("id", id));
    }
    let root = object(&mut builder, &fields);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

async fn canonical_metadata_preflight_peer(
    fault: MetadataFault,
    cancelled: bool,
    profile: crate::canonical::MetadataResponse,
) {
    use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (entered, arrived) = tokio::sync::oneshot::channel();
    let (release, released) = tokio::sync::oneshot::channel();
    let (frame_written, frame_arrival) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_hdr_async(stream, native_protocol).await.unwrap();
        let mut entered = Some(entered);
        let mut released = Some(released);
        let mut frame_written = Some(frame_written);
        while let Some(Ok(Message::Binary(bytes))) = socket.next().await {
            let Value::Object(request) = surrealdb::types::decode(&bytes).unwrap() else {
                continue;
            };
            if request.get("id").is_none_or(|id| matches!(id, Value::None)) {
                continue;
            }
            let method = request.get("method").and_then(Value::as_string).unwrap();
            let frame = if method == "query" {
                entered.take().unwrap().send(()).unwrap();
                released.take().unwrap().await.unwrap();
                metadata_peer_frame(&request, fault)
            } else {
                let mut reply = Object::new();
                reply.insert("id", request.get("id").unwrap().clone());
                reply.insert("session", request.get("session").unwrap().clone());
                reply.insert(
                    "result",
                    if method == "version" {
                        Value::String("3.3.0".into())
                    } else {
                        Value::None
                    },
                );
                surrealdb::types::encode(&Value::Object(reply)).unwrap()
            };
            if method == "query" && matches!(fault, MetadataFault::Valid) {
                use surrealdb_protocol::fb::v1 as fb;
                let mut builder = flatbuffers::FlatBufferBuilder::new();
                let string =
                    fb::StringValue::create(&mut builder, &fb::StringValueArgs { value: None });
                let result = fb::Value::create(
                    &mut builder,
                    &fb::ValueArgs {
                        value_type: fb::ValueType::String,
                        value: Some(string.as_union_value()),
                    },
                );
                let key = builder.create_string("result");
                let field = fb::KeyValue::create(
                    &mut builder,
                    &fb::KeyValueArgs {
                        key: Some(key),
                        value: Some(result),
                    },
                );
                let items = builder.create_vector(&[field]);
                let object =
                    fb::Object::create(&mut builder, &fb::ObjectArgs { items: Some(items) });
                let global = fb::Value::create(
                    &mut builder,
                    &fb::ValueArgs {
                        value_type: fb::ValueType::Object,
                        value: Some(object.as_union_value()),
                    },
                );
                builder.finish_minimal(global);
                socket
                    .send(Message::Binary(builder.finished_data().to_vec().into()))
                    .await
                    .unwrap();
            }
            socket.send(Message::Binary(frame.into())).await.unwrap();
            if method == "query" {
                frame_written.take().unwrap().send(()).unwrap();
            }
        }
    });
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("ws://{address}"),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let sdk = Surreal::new::<Ws>((
        address,
        Config::new()
            .bounded_requests()
            .websocket(WebsocketConfig::new().max_message_size(4 << 20))
            .unwrap(),
    ))
    .await
    .unwrap();
    let client = Arc::new(super::CanonicalClient::new(
        sdk,
        Duration::from_secs(5),
        &options,
    ));
    let bytes = profile.allocation_bytes();
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(bytes));
    let owner = MemoryConsumer::new("test:actual-metadata-preflight").register(&pool);
    owner.try_grow(bytes).unwrap();
    let witness = Arc::new("exact original read permission".to_owned());
    let weak = Arc::downgrade(&witness);
    let owner = Arc::new((witness, owner));
    let active = client.clone();
    let caller = tokio::spawn(super::response_ownership_scope(owner, async move {
        let mut query = active.metadata_query(profile, "RETURN 'PREFLIGHT'");
        if matches!(fault, MetadataFault::SharedCopies) {
            let mut limits = profile.limits();
            limits.string_bytes = 8192; // Three references need12288 plus envelope keys.
            query = query.request_context(
                RequestContext::new(Instant::now() + Duration::from_secs(5))
                    .with_response_limits(limits),
            );
        }
        query.await
    }));
    tokio::time::timeout(Duration::from_secs(2), arrived)
        .await
        .unwrap()
        .unwrap();
    if cancelled {
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        assert_eq!(
            pool.reserved(),
            bytes,
            "caller cancellation is not physical response drain"
        );
        assert!(weak.upgrade().is_some());
        let next = MemoryConsumer::new("test:metadata-reuse-refusal").register(&pool);
        assert!(next.try_grow(1).is_err());
        release.send(()).unwrap();
    } else {
        release.send(()).unwrap();
        let response = tokio::time::timeout(Duration::from_secs(2), caller)
            .await
            .unwrap()
            .unwrap();
        if matches!(fault, MetadataFault::Valid) {
            assert_eq!(
                response.unwrap().take::<Value>(0).unwrap(),
                Value::Bool(true)
            );
        } else if matches!(fault, MetadataFault::BlockBits)
            && matches!(profile, crate::canonical::MetadataResponse::BlockPage)
        {
            assert_eq!(
                response.unwrap().take::<Value>(0).unwrap(),
                metadata_block_bits(64, 8)
            );
        } else {
            assert!(
                response.is_err(),
                "{fault:?} must refuse before owned response conversion"
            );
        }
    }
    tokio::time::timeout(Duration::from_secs(2), frame_arrival)
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while pool.reserved() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        pool.reserved(),
        0,
        "physical driver drain releases the exact grant"
    );
    assert!(weak.upgrade().is_none());
}

#[tokio::test]
async fn canonical_metadata_preflight_bounds_exact_block_coordinate_bits() {
    for fault in [
        MetadataFault::BlockBits,
        MetadataFault::ExcessBlockBits,
        MetadataFault::LongBlockBits,
    ] {
        canonical_metadata_preflight_peer(
            fault,
            false,
            crate::canonical::MetadataResponse::BlockPage,
        )
        .await;
    }
    for profile in [
        crate::canonical::MetadataResponse::Page,
        crate::canonical::MetadataResponse::Protection,
    ] {
        canonical_metadata_preflight_peer(MetadataFault::SingleBlockBits, false, profile).await;
    }
}

#[tokio::test]
async fn canonical_metadata_preflight_accepts_valid_and_refuses_field_and_shared_copy_extents() {
    for fault in [
        MetadataFault::Valid,
        MetadataFault::LongString,
        MetadataFault::SharedCopies,
    ] {
        canonical_metadata_preflight_peer(fault, false, crate::canonical::MetadataResponse::Page)
            .await;
    }
}
#[tokio::test]
async fn canonical_metadata_preflight_refuses_unmatched_duplicate_and_malformed_correlations() {
    for fault in [
        MetadataFault::UnknownId,
        MetadataFault::UnknownSession,
        MetadataFault::DuplicateId,
        MetadataFault::MalformedId,
    ] {
        canonical_metadata_preflight_peer(fault, false, crate::canonical::MetadataResponse::Page)
            .await;
    }
}
#[tokio::test]
async fn canonical_metadata_preflight_keeps_cancelled_suffix_until_actual_physical_drain() {
    canonical_metadata_preflight_peer(
        MetadataFault::SharedCopies,
        true,
        crate::canonical::MetadataResponse::Page,
    )
    .await;
}

#[tokio::test]
async fn canonical_metadata_preflight_protection_ack_and_cancelled_cleanup_suffix() {
    canonical_metadata_preflight_peer(
        MetadataFault::Valid,
        false,
        crate::canonical::MetadataResponse::Protection,
    )
    .await;
    canonical_metadata_preflight_peer(
        MetadataFault::SharedCopies,
        true,
        crate::canonical::MetadataResponse::Protection,
    )
    .await;
}
