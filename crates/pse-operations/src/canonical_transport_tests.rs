// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual HTTP/2/protobuf controls for the pinned gRPC lifecycle.
use crate::transport::client::{Client, DispatchState, RequestContext};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use surrealdb_types::Value;
#[cfg(feature = "canonical-tests")]
use tokio::sync::mpsc;
#[path = "canonical_grpc_peer.rs"]
mod peer;
async fn connect(peer: &peer::Peer) -> Arc<Client> {
    Arc::new(Client::connect(peer.address, 4 << 20).await.unwrap())
}

fn context() -> RequestContext {
    RequestContext::new(Instant::now() + Duration::from_secs(5))
}

#[tokio::test]
async fn grpc_reserved_control_progress_and_awaited_physical_disconnect() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let mut requests = Vec::new();
    for _ in 0..32 {
        let db = db.clone();
        requests.push(tokio::spawn(async move {
            db.query("RETURN 'HOLD';").request_context(context()).await
        }));
    }
    for _ in 0..32 {
        peer.received.recv().await.unwrap();
    }
    let control = RequestContext::control(Instant::now() + Duration::from_secs(2));
    let response = db
        .query("RETURN 'CONTROL';")
        .request_context(control.clone())
        .await
        .unwrap();
    assert_eq!(response.num_statements(), 1);
    assert_eq!(control.dispatch_state(), DispatchState::Completed);
    assert!(peer.received.recv().await.unwrap().contains("CONTROL"));
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
    for request in requests {
        assert!(request.await.unwrap().is_err());
    }
    assert!(
        db.query("RETURN 'AFTER_CLOSE';")
            .request_context(context())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn grpc_control_waiters_are_finite_and_keep_original_clock_with_active_slots_reserved() {
    use std::future::IntoFuture;
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let mut held = Vec::new();
    for _ in 0..2 {
        let db = db.clone();
        held.push(tokio::spawn(async move {
            db.query("RETURN 'HOLD_CONTROL';")
                .request_context(RequestContext::control(
                    Instant::now() + Duration::from_secs(5),
                ))
                .await
        }));
    }
    for _ in 0..2 {
        peer.received.recv().await.unwrap();
    }
    let original = RequestContext::control(Instant::now() + Duration::from_millis(200));
    let mut queued = Vec::new();
    for index in 0..32 {
        let ctx = if index == 0 {
            original.clone()
        } else {
            RequestContext::control(Instant::now() + Duration::from_secs(5))
        };
        let mut future = Box::pin(
            db.query("RETURN 'WAIT_CONTROL';")
                .request_context(ctx)
                .into_future(),
        );
        assert!(futures_util::poll!(future.as_mut()).is_pending());
        queued.push(future);
    }
    let excess = RequestContext::control(Instant::now() + Duration::from_secs(5));
    let error = db
        .query("RETURN 'EXCESS_CONTROL';")
        .request_context(excess.clone())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("admission queue full"));
    assert_eq!(excess.dispatch_state(), DispatchState::NotDispatched);
    assert!(queued.remove(0).await.is_err());
    assert_eq!(original.dispatch_state(), DispatchState::NotDispatched);
    assert!(
        peer.received.try_recv().is_err(),
        "two active control slots stay saturated; expired waiter was never transmitted"
    );
    assert!(
        db.query("RETURN 'APPLICATION_PROGRESS';")
            .request_context(context())
            .await
            .is_ok()
    );
    assert!(
        peer.received
            .recv()
            .await
            .unwrap()
            .contains("APPLICATION_PROGRESS")
    );
    peer.release.add_permits(2);
    for hold in held {
        assert!(hold.await.unwrap().is_ok());
    }
    for future in queued {
        assert!(future.await.is_ok());
    }
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_native_uuid_envelopes_do_not_consume_application_blob_allowance() {
    for (query, accepted) in [
        ("RETURN 'BLOB_FOUR';", true),
        ("RETURN 'BLOB_FIVE';", false),
    ] {
        let peer = peer::start().await;
        let db = connect(&peer).await;
        let mut limits = crate::canonical::MetadataResponse::Protection.limits();
        limits.blob_bytes = 4;
        limits.blob_length = 4;
        let result = db
            .query(query)
            .request_context(context().with_response_limits(limits))
            .await;
        assert_eq!(
            result.is_ok(),
            accepted,
            "only actual application bytes consume the four-byte blob envelope"
        );
        db.disconnect().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), peer.server)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn grpc_unary_commit_preserves_exact_native_conflict_metadata() {
    let peer = peer::start().await;
    // Only the bounded test fixture exposes held transactions. This exercises
    // an actual unary status/trailer conflict without a production transaction API.
    let db = Client::connect(peer.address, 4 << 20).await.unwrap();
    let transaction = db
        .test_begin(Instant::now() + Duration::from_secs(5))
        .await
        .unwrap();
    let error = transaction.commit().await.unwrap_err();
    assert!(matches!(
        error.query_details(),
        Some(surrealdb_types::QueryError::TransactionConflict)
    ));
    peer.server.abort();
    assert!(peer.server.await.unwrap_err().is_cancelled());
}

#[tokio::test]
async fn grpc_generated_64_row_block_page_fits_original_owned_node_budget() {
    for accepted in [true, false] {
        let peer = peer::start().await;
        let db = connect(&peer).await;
        let mut limits = crate::canonical::MetadataResponse::JoinedBlockPage.limits();
        assert_eq!(
            limits.nodes, 8704,
            "the joined endpoint prepays block and batch metadata"
        );
        if !accepted {
            limits.nodes = 2048;
        }
        let result = db
            .query("RETURN 'GENERATED_BLOCK_PAGE';")
            .request_context(context().with_response_limits(limits))
            .await;
        if accepted {
            let mut result = result.unwrap();
            let rows: Vec<surrealdb_types::Object> = result.take(0).unwrap();
            assert_eq!(rows.len(), 64);
            for (ordinal, row) in rows.into_iter().enumerate() {
                let decoded =
                    crate::generated::surreal::decode_canonical_result_blocks(row).unwrap();
                assert_eq!(decoded.ordinal, ordinal as u64);
                assert_eq!(decoded.coordinate_min, Some(ordinal as f64));
                assert_eq!(decoded.coordinate_max, Some(ordinal as f64));
            }
        } else {
            let error = result.unwrap_err();
            assert!(
                error.to_string().contains("node envelope"),
                "a smaller prepaid node allowance still refuses the exact same encoded shape: {error}"
            );
        }
        db.disconnect().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), peer.server)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn grpc_cloned_handles_share_one_native_session_and_driver() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let clone = db.as_ref().clone();
    for _ in 0..100 {
        let original = context();
        clone
            .query("RETURN 'CLONED_HANDLE';")
            .request_context(original.clone())
            .await
            .unwrap();
        assert_eq!(original.dispatch_state(), DispatchState::Completed);
        assert_eq!(
            peer.received.recv().await.unwrap(),
            "RETURN 'CLONED_HANDLE'"
        );
    }
    drop(clone);
    db.query("RETURN 'ORIGINAL_OWNER';")
        .request_context(context())
        .await
        .unwrap();
    assert_eq!(
        peer.received.recv().await.unwrap(),
        "RETURN 'ORIGINAL_OWNER'"
    );
    let mut methods = Vec::new();
    while let Ok(method) = peer.methods.try_recv() {
        methods.push(method);
    }
    assert_eq!(
        methods
            .iter()
            .filter(|method| *method == "AttachSession")
            .count(),
        1,
        "cloning issued handles cannot attach another session"
    );
    assert_eq!(
        methods.iter().filter(|method| *method == "Query").count(),
        101
    );
    assert!(peer.received.try_recv().is_err());
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_expired_queued_request_never_dispatches_and_does_not_renew_clock() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let mut requests = Vec::new();
    for _ in 0..32 {
        let db = db.clone();
        requests.push(tokio::spawn(async move {
            db.query("RETURN 'HOLD';").request_context(context()).await
        }));
    }
    for _ in 0..32 {
        peer.received.recv().await.unwrap();
    }
    let expired = RequestContext::new(Instant::now() + Duration::from_millis(20));
    assert!(
        db.query("RETURN 'MUST_NOT_SEND';")
            .request_context(expired.clone())
            .await
            .is_err()
    );
    assert_eq!(expired.dispatch_state(), DispatchState::NotDispatched);
    peer.release.add_permits(32);
    for request in requests {
        assert!(request.await.unwrap().is_ok());
    }
    assert!(peer.received.try_recv().is_err());
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_cancelled_dispatched_owner_retains_grants_until_actual_response_drain() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    use datafusion_execution::memory_pool::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64 << 10));
    let reservation = MemoryConsumer::new("response-drain-fixture").register(&pool);
    reservation.try_grow(4096).unwrap();
    let owner = Arc::new(reservation);
    let access = crate::transport::lifetime::ConnectionAccess::default();
    let weak = Arc::downgrade(&owner);
    let ctx = context()
        .with_payload_owner(owner)
        .with_lifetime_owner(access.admit().unwrap());
    let retained = ctx.clone();
    let active = db.clone();
    let caller =
        tokio::spawn(async move { active.query("RETURN 'HOLD';").request_context(ctx).await });
    peer.received.recv().await.unwrap();
    assert!(retained.was_dispatched());
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    assert!(
        weak.upgrade().is_some(),
        "abandonment is not physical response drain"
    );
    peer.release.add_permits(1);
    tokio::time::timeout(Duration::from_secs(2), retained.payload_driver_drained())
        .await
        .unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(
        pool.reserved(),
        0,
        "actual prepaid capacity releases only after drain"
    );
    assert_eq!(retained.dispatch_state(), DispatchState::Completed);
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_predecode_response_limits_and_late_transaction_failures_refuse_results() {
    for query in [
        "RETURN 'BIG_STRING';",
        "RETURN 'MALFORMED';",
        "RETURN 'NO_END';",
        "RETURN 'LATE_FAILURE';",
    ] {
        let peer = peer::start().await;
        let db = connect(&peer).await;
        let ctx =
            context().with_response_limits(crate::canonical::MetadataResponse::Protection.limits());
        let outcome = db.query(query).request_context(ctx).await;
        assert!(
            outcome.is_err(),
            "{query} must not publish a successful partial result"
        );
        db.disconnect().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), peer.server)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn grpc_statement_scalars_cannot_grow_beyond_original_response_allowance() {
    for (query, message) in [
        ("RETURN 'COUNT_BOMB';", "statement allocation allowance"),
        ("RETURN 'SPARSE_INDEX';", "index exceeds"),
        ("RETURN 'REPEATED_BEGIN';", "repeated its Begin"),
    ] {
        let peer = peer::start().await;
        let db = connect(&peer).await;
        let limits = crate::canonical::MetadataResponse::Protection.limits();
        let error = db
            .query(query)
            .request_context(context().with_response_limits(limits))
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains(message),
            "tiny legal frame must fail its scalar check before growth: {error}"
        );
        db.disconnect().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), peer.server)
            .await
            .unwrap()
            .unwrap();
    }
    let peer = peer::start().await;
    let db = connect(&peer).await;
    let mut response = db
        .query("RETURN 'SKIPPED_VALID';")
        .request_context(
            context().with_response_limits(crate::canonical::MetadataResponse::Protection.limits()),
        )
        .await
        .unwrap();
    assert_eq!(
        response.num_statements(),
        1,
        "skipped indexes within the declared count do not invent results"
    );
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_individually_legal_frames_cannot_exceed_aggregate_response_allowance() {
    let peer = peer::start().await;
    let db = connect(&peer).await;
    let mut limits = crate::canonical::MetadataResponse::Header.limits();
    limits.string_length = 4096;
    limits.string_bytes = 8192;
    let outcome = db
        .query("RETURN 'AGGREGATE';")
        .request_context(context().with_response_limits(limits))
        .await;
    assert!(
        outcome.is_err(),
        "three legal 4096-byte frames exceed the one 8192-byte operation envelope"
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_legal_ordered_batches_publish_only_complete_result() {
    let peer = peer::start().await;
    let db = connect(&peer).await;
    let mut response = db
        .query("RETURN 'AGGREGATE';")
        .request_context(context())
        .await
        .unwrap();
    let values: Vec<String> = response.take(0).unwrap();
    assert_eq!(values.len(), 3);
    assert!(values.iter().all(|value| value.len() == 4096));
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_repeated_selection_checkpoint_retains_finite_acknowledged_setup() {
    let mut peer = peer::start().await;
    let raw = Arc::try_unwrap(connect(&peer).await).unwrap();
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("grpc://{}", peer.address),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let client = super::CanonicalClient::new(raw, Duration::from_secs(5), &options);
    for _ in 0..80 {
        assert!(client.select_existing().await.unwrap());
    }
    let mut methods = Vec::new();
    while let Ok(method) = peer.methods.try_recv() {
        methods.push(method);
    }
    assert_eq!(methods.iter().filter(|m| *m == "Signin").count(), 2);
    assert_eq!(methods.iter().filter(|m| *m == "Use").count(), 1);
    assert_eq!(methods.iter().filter(|m| *m == "Query").count(), 1);
    client.client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_ordered_setup_refuses_its_finite_cap_before_another_rpc() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    for _ in 0..63 {
        db.signin(crate::transport::client::Root {
            username: "owner".into(),
            password: "test".into(),
        })
        .request_context(context())
        .await
        .unwrap();
    }
    let refused = context();
    assert!(
        db.signin(crate::transport::client::Root {
            username: "owner".into(),
            password: "test".into()
        })
        .request_context(refused.clone())
        .await
        .is_err()
    );
    assert_eq!(refused.dispatch_state(), DispatchState::NotDispatched);
    let mut count = 0;
    while let Ok(method) = peer.methods.try_recv() {
        if method == "Signin" {
            count += 1;
        }
    }
    assert_eq!(count, 63, "a full setup log grants no further RPC");
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_retained_handle_restores_acknowledged_setup_for_new_rpc_without_app_replay() {
    let mut peer = peer::start_generations(2).await;
    let db = connect(&peer).await;
    db.signin(crate::transport::client::Root {
        username: "owner".into(),
        password: "test".into(),
    })
    .request_context(context())
    .await
    .unwrap();
    db.use_ns("namespace")
        .use_db("database")
        .request_context(context())
        .await
        .unwrap();
    db.signin(crate::transport::client::Root {
        username: "final-owner".into(),
        password: "test".into(),
    })
    .request_context(context())
    .await
    .unwrap();
    assert!(
        db.query("RETURN 'MALFORMED';")
            .request_context(context())
            .await
            .is_err()
    );
    let failed = peer.received.recv().await.unwrap();
    assert!(failed.contains("MALFORMED"));
    let mut result = db
        .query("RETURN 'FRESH';")
        .request_context(context())
        .await
        .unwrap();
    assert_eq!(result.take::<Value>(0).unwrap(), Value::Bool(true));
    assert!(peer.received.recv().await.unwrap().contains("FRESH"));
    assert!(
        peer.received.try_recv().is_err(),
        "failed application query must never be replayed"
    );
    let mut methods = Vec::new();
    while let Ok(method) = peer.methods.try_recv() {
        methods.push(method);
    }
    assert_eq!(
        methods
            .iter()
            .filter(|method| *method == "AttachSession")
            .count(),
        2
    );
    assert_eq!(
        methods.iter().filter(|method| *method == "Signin").count(),
        4
    );
    assert_eq!(methods.iter().filter(|method| *method == "Use").count(), 2);
    let setup: Vec<_> = methods
        .iter()
        .filter(|m| matches!(m.as_str(), "Signin" | "Use"))
        .map(String::as_str)
        .collect();
    let expected = ["Signin", "Use", "Signin", "Signin", "Use", "Signin"];
    assert_eq!(
        setup, expected,
        "only complete ACK setup replays in original order"
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
    assert!(
        db.query("RETURN 'CLOSED';")
            .request_context(context())
            .await
            .is_err(),
        "explicit disconnect remains permanent"
    );
}

#[tokio::test]
async fn grpc_restoration_uses_original_clock_and_retains_complete_ack_snapshot_after_failure() {
    let mut peer = peer::start_with_restore_delay(3, true).await;
    let db = connect(&peer).await;
    db.signin(crate::transport::client::Root {
        username: "owner".into(),
        password: "test".into(),
    })
    .request_context(context())
    .await
    .unwrap();
    db.use_ns("namespace")
        .use_db("database")
        .request_context(context())
        .await
        .unwrap();
    db.signin(crate::transport::client::Root {
        username: "final-owner".into(),
        password: "test".into(),
    })
    .request_context(context())
    .await
    .unwrap();
    assert!(
        db.query("RETURN 'MALFORMED';")
            .request_context(context())
            .await
            .is_err()
    );
    assert!(peer.received.recv().await.unwrap().contains("MALFORMED"));
    let original = RequestContext::new(Instant::now() + Duration::from_millis(50));
    assert!(
        db.query("RETURN 'MUST_NOT_SEND';")
            .request_context(original.clone())
            .await
            .is_err()
    );
    assert_eq!(
        original.dispatch_state(),
        DispatchState::NotDispatched,
        "setup dispatch is distinct from the application effect"
    );
    peer.release.add_permits(1);
    let mut result = db
        .query("RETURN 'FRESH';")
        .request_context(context())
        .await
        .unwrap();
    assert_eq!(result.take::<Value>(0).unwrap(), Value::Bool(true));
    assert!(peer.received.recv().await.unwrap().contains("FRESH"));
    assert!(peer.received.try_recv().is_err());
    let mut methods = Vec::new();
    while let Ok(method) = peer.methods.try_recv() {
        methods.push(method);
    }
    assert_eq!(
        methods
            .iter()
            .filter(|method| *method == "AttachSession")
            .count(),
        3
    );
    assert_eq!(
        methods.iter().filter(|method| *method == "Use").count(),
        2,
        "a partial failed restore must not erase the last complete ACK snapshot"
    );
    let setup: Vec<_> = methods
        .iter()
        .filter(|m| matches!(m.as_str(), "Signin" | "Use"))
        .map(String::as_str)
        .collect();
    let expected = [
        "Signin", "Use", "Signin", "Signin", "Signin", "Use", "Signin",
    ];
    assert_eq!(
        setup, expected,
        "only complete ACK setup replays in original order"
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[cfg(feature = "canonical-tests")]
pub(crate) async fn canonical_read_barrier_peer() -> (
    crate::canonical::CanonicalStore,
    mpsc::UnboundedReceiver<String>,
    tokio::task::JoinHandle<()>,
) {
    let peer = peer::start().await;
    let sdk = connect(&peer).await;
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("grpc://{}", peer.address),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let sdk = Arc::try_unwrap(sdk).unwrap();
    let client = Arc::new(super::CanonicalClient::new(
        sdk,
        Duration::from_secs(2),
        &options,
    ));
    let store = crate::canonical::CanonicalStore {
        db: client.clone(),
        activation_db: client,
        problem_turns: Arc::default(),
        fixture_lifetime: None,
        issuing_store: Arc::default(),
        study_module_sha256: Arc::default(),
        files: Arc::new(crate::canonical_files::FileSession::wire_control().unwrap()),
        result_read_drain: Arc::default(),
        connection_options: options.clone(),
        state_path: options.state_path.clone(),
        database: options.database,
        namespace: options.namespace,
    };
    (store, peer.received, peer.server)
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
async fn canonical_complete_read_retries_native_conflicts_before_exposing_response() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let mut clocks = Vec::new();
    let mut response = crate::canonical::protected_query("complete-read-control", || {
        let deadline = super::original_deadline(crate::canonical::REQUEST_TIMEOUT);
        clocks.push(deadline);
        Ok(db
            .query("RETURN 'READ_CONFLICT_TWICE';")
            .request_context(RequestContext::new(deadline.into_std())))
    })
    .await
    .unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(clocks.len(), 3);
    assert!(clocks.iter().all(|deadline| *deadline == clocks[0]));
    for _ in 0..3 {
        assert_eq!(
            peer.received.recv().await.unwrap(),
            "RETURN 'READ_CONFLICT_TWICE'"
        );
    }
    assert!(peer.received.try_recv().is_err());
    db.disconnect().await.unwrap();
    peer.server.await.unwrap();
}

#[tokio::test]
async fn canonical_complete_read_conflicts_keep_original_clock_and_typed_exhaustion() {
    let peer = peer::start().await;
    let db = connect(&peer).await;
    let deadline = tokio::time::Instant::now() + Duration::from_millis(30);
    let mut clocks = Vec::new();
    let result = super::within_clock(
        deadline,
        crate::canonical::protected_query("read-clock-control", || {
            let inherited = super::original_deadline(Duration::from_secs(30));
            clocks.push(inherited);
            // Observe the enclosing retry clock independently of a client deadline
            // racing to return its correctly unreplayed unknown-outcome error.
            Ok(db
                .query("RETURN 'READ_ALWAYS_CONFLICT';")
                .request_context(context()))
        }),
    )
    .await;
    assert!(
        matches!(result, Err(crate::canonical::CanonicalError::Timeout)),
        "{result:?}"
    );
    assert!(!clocks.is_empty());
    assert!(clocks.iter().all(|inherited| *inherited == deadline));
    assert!(clocks.len() < crate::canonical::GUARDED_ATTEMPTS as usize);
    db.disconnect().await.unwrap();
    peer.server.await.unwrap();

    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let result = crate::canonical::protected_query("read-exhaustion-control", || {
        Ok(db
            .query("RETURN 'READ_ALWAYS_CONFLICT';")
            .request_context(RequestContext::new(
                super::original_deadline(Duration::from_secs(30)).into_std(),
            )))
    })
    .await;
    assert!(matches!(
        result,
        Err(crate::canonical::CanonicalError::ConflictExhausted {
            operation: "read-exhaustion-control",
            attempts: crate::canonical::GUARDED_ATTEMPTS,
        })
    ));
    let mut sent = 0;
    while peer.received.try_recv().is_ok() {
        sent += 1;
    }
    assert_eq!(sent, crate::canonical::GUARDED_ATTEMPTS);
    db.disconnect().await.unwrap();
    peer.server.await.unwrap();
}

#[tokio::test]
async fn canonical_complete_read_never_retries_nonconflict_or_incomplete_protocol() {
    for query in [
        "RETURN 'READ_CANCELLED';",
        "RETURN 'LATE_FAILURE';",
        "RETURN 'MALFORMED';",
    ] {
        let mut peer = peer::start().await;
        let db = connect(&peer).await;
        let result = crate::canonical::protected_query("read-no-replay-control", || {
            Ok(db.query(query).request_context(RequestContext::new(
                super::original_deadline(crate::canonical::REQUEST_TIMEOUT).into_std(),
            )))
        })
        .await;
        assert!(
            matches!(result, Err(crate::canonical::CanonicalError::Driver(_))),
            "{query}: {result:?}"
        );
        assert_eq!(
            peer.received.recv().await.unwrap(),
            query.trim_end_matches(';')
        );
        assert!(
            peer.received.try_recv().is_err(),
            "failed read was not replayed"
        );
        db.disconnect().await.unwrap();
        peer.server.await.unwrap();
    }
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn source_stage_failed_acquisition_retains_both_causes_under_direct_and_project_clocks() {
    use crate::canonical::{CanonicalError, Revision};
    for project in [false, true] {
        for pending_readback in [false, true] {
            let mut peer = peer::start_source_stage_fault(pending_readback).await;
            let sdk = connect(&peer).await;
            let directory = tempfile::tempdir().unwrap();
            std::fs::write(
                directory.path().join("config.json"),
                br#"{"namespace":"namespace","database":"database","accepting_writes":true}"#,
            )
            .unwrap();
            let options = crate::canonical::CanonicalOptions {
                endpoint: format!("grpc://{}", peer.address),
                namespace: "namespace".into(),
                database: "database".into(),
                username: "owner".into(),
                password: "test".into(),
                selection_username: "viewer".into(),
                selection_password: "test".into(),
                state_path: directory.path().to_owned(),
                initializer_nonce: None,
            };
            let client = Arc::new(super::CanonicalClient::new(
                Arc::try_unwrap(sdk).unwrap(),
                Duration::from_secs(2),
                &options,
            ));
            let issuing_store = Arc::new(std::sync::OnceLock::new());
            issuing_store.set("issued-store".into()).unwrap();
            let store = crate::canonical::CanonicalStore {
                db: client.clone(),
                activation_db: client,
                problem_turns: Arc::default(),
                fixture_lifetime: None,
                issuing_store,
                study_module_sha256: Arc::default(),
                files: Arc::new(crate::canonical_files::FileSession::wire_control().unwrap()),
                result_read_drain: Arc::default(),
                connection_options: options.clone(),
                state_path: options.state_path.clone(),
                database: options.database,
                namespace: options.namespace,
            };
            let problem = "original-source-problem";
            let operation = "original-source-operation";
            let intent = store
                .project_fork_intent(
                    &crate::canonical_projects::Project {
                        key: "original-project".into(),
                        title: "Original".into(),
                        deleted: false,
                        generation: 7,
                    },
                    "original-branch",
                    problem,
                    &Revision {
                        key: "original-origin".into(),
                        problem: "origin-problem".into(),
                        sequence: 1,
                        parent: None,
                        operation: "original-origin".into(),
                        request: vec![1, 2, 3].into(),
                        interpretation: crate::generated::surreal::INTERPRETATION.into(),
                    },
                    operation,
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_millis(500);
            let original = tokio::time::Instant::from_std(deadline);
            let error = super::operation_scope(deadline, async {
                // Equal nested clocks must carry authority without racing a second dropping timer.
                super::operation_scope(deadline, async {
                    assert_eq!(super::original_deadline(Duration::from_secs(30)), original);
                    let result = if project {
                        store.edit_project(&intent, &[]).await
                    } else {
                        store.edit(problem, None, operation, &[]).await
                    };
                    assert_eq!(super::original_deadline(Duration::from_secs(30)), original);
                    result.unwrap_err()
                })
                .await
            })
            .await;
            assert!(error.effect_is_unknown());
            assert!(!error.retryable());
            let CanonicalError::SourceEditUnsettled {
                problem: retained_problem,
                operation: retained_operation,
                source,
            } = error
            else {
                panic!("lost source attribution: {error:?}")
            };
            assert_eq!(retained_problem, problem);
            assert_eq!(retained_operation, operation);
            let CanonicalError::SettlementUnresolved {
                owner,
                operation: retained,
                request,
                source,
                readback,
            } = *source
            else {
                panic!("lost dual settlement")
            };
            assert_eq!(owner, "source-stage");
            assert_eq!(retained, operation);
            let encoded =
                serde_json::to_vec(&(problem, None::<&str>, Vec::<serde_json::Value>::new()))
                    .unwrap();
            let expected_request = pse_ids::derive_hash(
                pse_ids::Frame::CanonicalEditV1,
                &[
                    crate::generated::surreal::INTERPRETATION.as_bytes(),
                    &encoded,
                ],
            );
            assert_eq!(request, expected_request.as_bytes());
            assert!(source.effect_is_unknown());
            assert!(!source.is_conflict());
            assert!(!source.retryable());
            let readback = readback.expect("independent receipt failure");
            assert!(readback.effect_is_unknown());
            if pending_readback {
                // At an equal clock, the native client may publish its original typed
                // interruption before Tokio's enclosing timeout observes expiry.
                assert!(
                    matches!(readback.as_ref(), CanonicalError::Timeout)
                        || matches!(readback.as_ref(), CanonicalError::Driver(error)
                        if error.connection_details() == Some(&surrealdb_types::ConnectionError::ConnectionFailed)),
                    "{readback:?}"
                );
                assert!(Instant::now() >= deadline);
                assert!(Instant::now() < deadline + Duration::from_secs(1));
            } else {
                assert!(
                    matches!(*readback, CanonicalError::Driver(_)),
                    "{readback:?}"
                );
            }
            let mut requests = Vec::new();
            while let Ok(query) = peer.received.try_recv() {
                requests.push(query);
            }
            assert_eq!(
                requests
                    .iter()
                    .filter(|q| q.contains("LET $stage_guard ="))
                    .count(),
                1
            );
            assert_eq!(
                requests
                    .iter()
                    .filter(|q| q
                        .contains("SELECT * FROM ONLY type::record('canonical_revisions', $key)"))
                    .count(),
                2
            );
            assert!(
                !requests
                    .iter()
                    .any(|q| q.contains("canonical_staged_edits")),
                "no staging or activation after unresolved acquisition"
            );
            let mut previous_timeout = Duration::from_millis(500);
            while let Ok((_received_at, request)) = peer.requests.try_recv() {
                let timeout = request.context.unwrap().timeout.unwrap();
                let timeout = Duration::new(u64::try_from(timeout.seconds).unwrap(), timeout.nanos);
                // The protobuf timeout is a relative client budget, so server receipt
                // time cannot be added to it without counting network transit twice.
                assert!(
                    timeout <= previous_timeout,
                    "wire request renewed original budget"
                );
                previous_timeout = timeout;
                let bindings = request
                    .variables
                    .unwrap()
                    .variables
                    .into_iter()
                    .map(|binding| {
                        (
                            binding.key,
                            Value::try_from(binding.value.unwrap()).unwrap(),
                        )
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                if request.query.contains("LET $stage_guard =") {
                    assert_eq!(bindings["problem"], Value::String(problem.into()));
                    assert_eq!(bindings["operation"], Value::String(operation.into()));
                    assert_eq!(
                        bindings["request_digest"],
                        Value::String(expected_request.to_hex())
                    );
                } else if request.query.contains("canonical_revisions") {
                    assert_eq!(bindings["key"], Value::String(operation.into()));
                }
            }
            if pending_readback {
                peer.release.add_permits(1);
            }
            store.db.disconnect().await.unwrap();
            tokio::time::timeout(Duration::from_secs(2), peer.server)
                .await
                .unwrap()
                .unwrap();
        }
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

#[tokio::test]
async fn canonical_statistics_keep_server_samples_out_of_local_active_intervals() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value as ObservedValue};
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("canonical-statistics");
    let observer = super::CanonicalStatistics {
        context: Some(operation.context()),
        metrics: &super::QUERY_METRICS,
        invocation: 42,
    };
    let mut stats = crate::transport::client::Stats::default();
    stats.execution_time = Some(Duration::from_nanos(91));
    crate::transport::client::QueryStatisticsObserver::observe(&observer, 3, stats, false);
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

#[test]
fn canonical_statistics_bound_bursts_without_displacing_attempt_identity() {
    use crate::transport::client::QueryStatisticsObserver;
    use pse_diagnostics::observation::{ObservationHub, Policy, Value as ObservedValue};
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("canonical-statistics-burst");
    for invocation in 0..1000 {
        let observer = super::CanonicalStatistics {
            context: Some(operation.context()),
            metrics: &super::QUERY_METRICS,
            invocation,
        };
        let mut stats = crate::transport::client::Stats::default();
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
        context: Some(operation.context()),
        metrics: &super::QUERY_METRICS,
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

#[test]
fn canonical_owner_metrics_without_capture_keep_exact_statement_population() {
    use crate::transport::client::QueryStatisticsObserver;
    static METRICS: super::CanonicalMetricState = super::CanonicalMetricState::new();
    let observer = super::CanonicalStatistics {
        context: None,
        metrics: &METRICS,
        invocation: 1,
    };
    let before = METRICS.snapshot();
    assert!(before.source_started_unix_micros.is_none());
    let mut stats = crate::transport::client::Stats::default();
    stats.execution_time = Some(Duration::from_nanos(91));
    observer.observe(0, stats, true);
    observer.observe(1, crate::transport::client::Stats::default(), false);
    observer.request_failed();
    let sample = METRICS.snapshot();
    assert_eq!(sample.builders, 0);
    assert_eq!(sample.statements, 2);
    assert_eq!(sample.statements_succeeded, 1);
    assert_eq!(sample.statements_failed, 1);
    assert_eq!(sample.statements_timed, 1);
    assert_eq!(sample.statements_untimed, 1);
    assert_eq!(sample.request_failures, 1);
    assert_eq!(sample.server_duration_nanos, 91);
    assert!(!sample.saturated);
    let repeat = METRICS.snapshot();
    assert_eq!(repeat.statements, 2);
    assert_eq!(
        repeat.source_started_unix_micros,
        sample.source_started_unix_micros
    );
}

#[test]
fn canonical_metric_keys_bind_producer_and_copied_populations_without_position() {
    use super::CanonicalQueryEvent as Event;
    let state = super::CanonicalMetricState::new();
    for (event, count) in [
        (Event::StatementUntimed, 19),
        (Event::StatementTimed, 17),
        (Event::StatementFailed, 13),
        (Event::StatementSucceeded, 11),
        (Event::Statement, 7),
        (Event::RequestFailed, 5),
        (Event::Builder, 3),
    ] {
        state.add(event, count);
    }
    state.add_duration(23);
    let sample = state.snapshot();
    assert_eq!(
        (sample.builders, sample.request_failures, sample.statements),
        (3, 5, 7)
    );
    assert_eq!(
        (sample.statements_succeeded, sample.statements_failed),
        (11, 13)
    );
    assert_eq!(
        (
            sample.statements_timed,
            sample.statements_untimed,
            sample.server_duration_nanos
        ),
        (17, 19, 23)
    );
    let events = Event::ALL
        .into_iter()
        .rev()
        .map(|key| (key.as_str(), key.count(&sample)))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        events,
        [
            ("builder", 3),
            ("request_failed", 5),
            ("statement", 7),
            ("statement_succeeded", 11),
            ("statement_failed", 13),
            ("statement_timed", 17),
            ("statement_untimed", 19)
        ]
        .into()
    );
    state.add(Event::Builder, u64::MAX);
    let saturated = state.snapshot();
    assert!(saturated.saturated);
    assert_eq!(saturated.builders, u64::MAX);
    assert_eq!(saturated.server_duration_nanos, 23);
}

#[tokio::test]
async fn complete_response_selects_lowest_substantive_statement_not_frame_or_hash_order() {
    let peer = peer::start().await;
    let db = connect(&peer).await;
    for _ in 0..16 {
        let response = db
            .query("RETURN 'MULTIPLE_STATEMENT_ERRORS';")
            .request_context(context())
            .await
            .unwrap();
        let error = crate::transport::complete_response(response).unwrap_err();
        let crate::canonical::CanonicalError::Driver(error) = error else {
            panic!("expected original typed statement error");
        };
        assert!(error.to_string().contains("first substantive"));
        assert!(matches!(
            error.query_details(),
            Some(surrealdb_types::QueryError::TransactionConflict)
        ));
    }
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_trimmed_query_compacts_unframed_indices_and_keeps_kill_gap() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let mut response = db
        .query("RETURN 'SPARSE_WITH_KILL'; \n;\t")
        .request_context(context())
        .await
        .unwrap();
    assert_eq!(
        peer.received.recv().await.unwrap(),
        "RETURN 'SPARSE_WITH_KILL'"
    );
    assert_eq!(
        response.num_statements(),
        2,
        "Kill is absent from published slots"
    );
    assert_eq!(
        response.take::<Option<String>>(0).unwrap().as_deref(),
        Some("first")
    );
    assert_eq!(
        response.take::<Value>(1).unwrap(),
        Value::None,
        "Kill keeps its compacted index"
    );
    assert_eq!(
        response.take::<Vec<String>>(2).unwrap(),
        vec!["last".to_owned()]
    );
    assert_eq!(response.num_statements(), 0);
    assert_eq!(
        response.take::<Value>(0).unwrap(),
        Value::None,
        "taken slots remain absent"
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_typed_authority_bindings_cannot_be_replaced_by_business_binds() {
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let mut authority = surrealdb_types::Object::new();
    authority.insert("problem", Value::String("issued-problem".into()));
    let original = context();
    let error = db
        .query("RETURN $problem")
        .bind_guarded(authority)
        .bind(("problem", "forged-problem"))
        .request_context(original.clone())
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("authority binding problem cannot be overwritten")
    );
    assert_eq!(original.dispatch_state(), DispatchState::NotDispatched);
    assert!(
        peer.received.try_recv().is_err(),
        "authority misuse never reaches a physical query"
    );
    db.query("RETURN 'AFTER_LOCAL_REFUSAL'")
        .request_context(context())
        .await
        .unwrap();
    assert!(
        peer.received
            .recv()
            .await
            .unwrap()
            .contains("AFTER_LOCAL_REFUSAL")
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_abandoned_request_retains_all_issued_read_guards_until_physical_drain() {
    use crate::transport::client::RetainedRequest;
    use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let mut peer = peer::start().await;
    let db = connect(&peer).await;
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64));
    let guards = ["first-issued-read", "second-issued-read"].map(|name| {
        let owner = MemoryConsumer::new(name).register(&pool);
        owner.try_grow(32).unwrap();
        let selection = crate::canonical::ProtectedSelection::test_only(
            crate::canonical::Revision {
                key: "revision".into(),
                problem: "problem".into(),
                sequence: 1,
                parent: None,
                operation: "operation".into(),
                request: Vec::new().into(),
                interpretation: "test".into(),
            },
            name,
        );
        selection.retain_metadata_owner(Arc::new(owner)).unwrap();
        selection
    });
    let [first, second] = guards;
    let ctx = context().with_retained_request(RetainedRequest::combine(first, second));
    let historical = ctx.clone();
    let active = db.clone();
    let caller =
        tokio::spawn(async move { active.query("RETURN 'HOLD';").request_context(ctx).await });
    peer.received.recv().await.unwrap();
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    assert_eq!(
        pool.reserved(),
        64,
        "caller abandonment cannot drop either issued release guard"
    );
    peer.release.add_permits(1);
    tokio::time::timeout(Duration::from_secs(2), async {
        while pool.reserved() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(historical.dispatch_state(), DispatchState::Completed);
    assert_eq!(
        pool.reserved(),
        0,
        "actual late response drain retires both guard cells before disconnect"
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_internal_owner_composition_keeps_each_original_drain_leaf_once() {
    use pse_columnar::{AllocationLease, GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let peer = peer::start().await;
    let db = connect(&peer).await;
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(3));
    let reservation = || {
        let grant = MemoryConsumer::new("test:distinct-drain-leaf").register(&pool);
        grant.try_grow(1).unwrap();
        grant
    };
    let query = db
        .query("RETURN 'MUST_NOT_DISPATCH'")
        .payload_owner(Arc::new(reservation()))
        .payload_owner(Arc::new(reservation()))
        .scientific_payload_owner(AllocationLease::new(reservation()));
    assert_eq!(
        query.payload_leaf_count(),
        3,
        "internal composition does not re-inherit an already retained owner tree"
    );
    assert_eq!(pool.reserved(), 3);
    drop(query);
    assert_eq!(
        pool.reserved(),
        0,
        "unpolled builder drops actual bytes before retiring custody cells"
    );
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_abandoned_close_joins_one_detach_round_under_its_original_clock() {
    let mut peer = peer::start_with_held_detach().await;
    let db = connect(&peer).await;
    let original = tokio::time::Instant::now() + Duration::from_millis(300);
    let first = db.clone();
    let closing = tokio::spawn(async move {
        crate::transport::operation_scope(original.into_std(), first.disconnect()).await
    });
    while peer.methods.recv().await.unwrap() != "DetachSession" {}
    closing.abort();
    assert!(closing.await.unwrap_err().is_cancelled());
    let request = context();
    assert!(
        db.query("RETURN 'AFTER_CLOSE';")
            .request_context(request.clone())
            .await
            .is_err()
    );
    assert_eq!(request.dispatch_state(), DispatchState::NotDispatched);
    tokio::time::timeout(Duration::from_secs(1), db.disconnect())
        .await
        .unwrap()
        .unwrap();
    while let Ok(method) = peer.methods.try_recv() {
        assert_ne!(
            method, "DetachSession",
            "successor waiter cannot issue a second detach or renew its clock"
        );
    }
    peer.release.add_permits(1);
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn canonical_envelopes_seal_original_clock_and_authority_before_dispatch() {
    let mut peer = peer::start().await;
    let raw = Arc::try_unwrap(connect(&peer).await).unwrap();
    let options = crate::canonical::CanonicalOptions {
        endpoint: format!("grpc://{}", peer.address),
        namespace: "namespace".into(),
        database: "database".into(),
        username: "owner".into(),
        password: "test".into(),
        selection_username: "viewer".into(),
        selection_password: "test".into(),
        state_path: std::path::PathBuf::new(),
        initializer_nonce: None,
    };
    let client = super::CanonicalClient::new(raw, Duration::from_secs(5), &options);
    let selection = crate::canonical::ProtectedSelection::test_only(
        crate::canonical::Revision {
            key: "revision".into(),
            problem: "problem".into(),
            sequence: 1,
            parent: None,
            operation: "operation".into(),
            request: Vec::new().into(),
            interpretation: "test".into(),
        },
        "issued-protection",
    );
    use crate::canonical_script::{Envelope, GuardedScript, Recheck};
    for envelope in [
        Envelope::DeadlineOnly,
        Envelope::ProtectedRead {
            selection: &selection,
            recheck: Recheck::Pin,
        },
    ] {
        for key in ["pse_rpc_expires_at", "pse_rpc_timeout", "problem"] {
            if key == "problem" && matches!(envelope, Envelope::DeadlineOnly) {
                continue;
            }
            let query = client.script(
                GuardedScript::new(envelope)
                    .returning::<Value>("", "true")
                    .unwrap(),
            );
            let original = query.context_snapshot().unwrap();
            let error = query.bind((key, Value::None)).await.unwrap_err();
            assert!(
                error.to_string().contains("cannot be overwritten"),
                "{error}"
            );
            assert_eq!(original.dispatch_state(), DispatchState::NotDispatched);
            assert!(
                peer.received.try_recv().is_err(),
                "clock/authority overwrite never reaches a physical query"
            );
        }
    }
    client.client.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_metadata_limits_preserve_original_lifecycle_and_exact_owner_population() {
    use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let peer = peer::start().await;
    let db = connect(&peer).await;
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1));
    let grant = MemoryConsumer::new("test:metadata-profile-context").register(&pool);
    grant.try_grow(1).unwrap();
    let original = context();
    let query = db
        .query("RETURN 'NOT_POLLED'")
        .request_context(original.clone())
        .payload_owner(Arc::new(grant));
    let count = query.payload_leaf_count();
    let query = query.response_limits(crate::canonical::MetadataResponse::Header.limits());
    assert!(
        query.shares_lifecycle(&original),
        "profile update retains the identical original deadline/cancellation/dispatch cell"
    );
    assert_eq!(
        query.payload_leaf_count(),
        count,
        "profile update adds no independent owner or activity cell"
    );
    assert_eq!(
        query.context_snapshot().unwrap().deadline(),
        original.deadline()
    );
    drop(query);
    assert_eq!(pool.reserved(), 0);
    db.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), peer.server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn grpc_malformed_batch_and_unsupported_live_close_generation_without_replaying_query() {
    for (marker, reason) in [
        (
            "MIXED_BATCH_TO_SINGLE",
            "Single query response cannot follow",
        ),
        ("UNSUPPORTED_LIVE_RESPONSE", "Unsupported LIVE response"),
    ] {
        let mut peer = peer::start_generations(2).await;
        let db = connect(&peer).await;
        let query = format!("RETURN '{marker}'");
        let error = db
            .query(query.clone())
            .request_context(context())
            .await
            .unwrap_err();
        assert!(
            error.is_connection(),
            "unsupported protocol must close the generation: {error}"
        );
        assert!(error.to_string().contains(reason), "{error}");
        assert_eq!(peer.received.recv().await.unwrap(), query);
        let mut valid = db
            .query("RETURN 'AFTER_PROTOCOL_REFUSAL'")
            .request_context(context())
            .await
            .unwrap();
        assert_eq!(valid.take::<Value>(0).unwrap(), Value::Bool(true));
        assert_eq!(
            peer.received.recv().await.unwrap(),
            "RETURN 'AFTER_PROTOCOL_REFUSAL'"
        );
        assert!(
            peer.received.try_recv().is_err(),
            "restoration never replays the refused application query"
        );
        db.disconnect().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), peer.server)
            .await
            .unwrap()
            .unwrap();
    }
}
