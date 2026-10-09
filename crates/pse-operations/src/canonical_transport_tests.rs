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
                if (method == "attach" && connection == 1)
                    || (method == "query" && query != "CONTROL")
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
                    Value::Array(vec![Value::Object(statement)].into())
                } else if method == "version" {
                    Value::String("3.3.0".into())
                } else {
                    Value::None
                };
                reply.insert("result", result);
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
async fn native_sdk_stalled_replay_refuses_cancelled_and_expired_routes() {
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
    tokio::time::timeout(Duration::from_secs(2), client.disconnect())
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    while let Ok(message) = received.try_recv() {
        assert!(matches!(message.as_str(), "attach" | "ping" | "version"));
    }
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
    let server = tokio::spawn(async move {
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
                    tokio::time::sleep(Duration::from_millis(150)).await;
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
        native: crate::canonical::NativeAllocation {
            native_workers: 1,
            native_worker_memory_bytes: 1024,
            execution: None,
        },
        primary_receiver: None,
        state_path: std::path::PathBuf::new(),
    };
    let sdk = Surreal::new::<Ws>((address, Config::new().bounded_requests()))
        .await
        .unwrap();
    let client = super::CanonicalClient::new(sdk, Duration::from_millis(60), &options);
    let first = client.select_existing().await.unwrap_err();
    if late_acknowledgment {
        assert!(matches!(
            first,
            crate::canonical::CanonicalError::Timeout | crate::canonical::CanonicalError::Driver(_)
        ));
    } else {
        assert!(first.to_string().contains("owner acknowledgment refused"));
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
    tokio::time::sleep(Duration::from_millis(200)).await;
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
