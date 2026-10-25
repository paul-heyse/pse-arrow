// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Execute vendored SDK controls in the workspace's supported unit-test harness.
//! The test-only SDK feature exposes controls over its actual private owners;
//! these wrappers neither copy that implementation nor create a second workspace.

use surrealdb::engine::remote::grpc_lifecycle_controls as sdk;

#[tokio::test]
async fn spawned_siblings_keep_independent_parent_and_wire_carrier() {
    sdk::spawned_siblings_keep_independent_parent_and_wire_carrier().await;
}

#[tokio::test]
async fn authentic_tracing_parent_precedes_unrelated_ambient_api_context() {
    sdk::authentic_tracing_parent_precedes_unrelated_ambient_api_context().await;
}

#[tokio::test]
async fn missing_context_removes_stale_carrier_without_inventing_parent() {
    sdk::missing_context_removes_stale_carrier_without_inventing_parent().await;
}

#[tokio::test]
async fn api_only_unsampled_context_preserves_flags_and_state() {
    sdk::api_only_unsampled_context_preserves_flags_and_state().await;
}

#[tokio::test]
async fn response_body_retains_span_and_installs_its_context_on_poll_error_and_drop() {
    sdk::response_body_retains_span_and_installs_its_context_on_poll_error_and_drop().await;
}

#[tokio::test]
async fn query_owner_outlives_transport_until_decode_finishes() {
    sdk::query_owner_outlives_transport_until_decode_finishes().await;
}

#[tokio::test]
async fn actual_owned_http2_headers_stream_error_cancel_and_siblings() {
    sdk::actual_owned_http2_headers_stream_error_cancel_and_siblings().await;
}

#[tokio::test]
async fn actual_upstream_http2_headers_stream_error_cancel_and_siblings() {
    sdk::actual_upstream_http2_headers_stream_error_cancel_and_siblings().await;
}
