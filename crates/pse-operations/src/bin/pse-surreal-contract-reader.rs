// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Connection-only independent readback for the original SurrealKit suite owner.
#![allow(
    clippy::print_stderr,
    reason = "fixture helper reports sanitized failures"
)]
use clap::Parser;
use pse_operations::{
    generated::surreal as wire,
    native_service::NativeServiceError,
    transport::{
        bounded_query,
        client::{Client, RequestContext},
    },
};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use surrealdb_types::Object;

#[derive(Parser)]
struct Cli {
    /// Independent expected value; a deliberate mismatch exercises consumer failure.
    #[arg(long, default_value_t = u64::MAX)]
    expected_generation: u64,
    /// Hold an authenticated consumer until the original fixture interrupts it.
    #[arg(long)]
    wait_for_interruption: bool,
}
fn refused(message: &'static str) -> NativeServiceError {
    NativeServiceError::Refused(message)
}
fn input(name: &str) -> Result<String, NativeServiceError> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| refused("suite connection input is unavailable"))
}
async fn read(cli: Cli) -> Result<(), NativeServiceError> {
    let endpoint = input("SURREALKIT_EXEC_ENDPOINT")?;
    let address: SocketAddr = endpoint
        .strip_prefix("grpc://")
        .and_then(|value| value.parse().ok())
        .filter(|address: &SocketAddr| address.ip().is_loopback() && address.port() != 0)
        .ok_or_else(|| refused("suite endpoint must be an owned loopback gRPC service"))?;
    let namespace = input("SURREALKIT_EXEC_NAMESPACE")?;
    let database = input("SURREALKIT_EXEC_DATABASE")?;
    let token = input("SURREALKIT_EXEC_TOKEN")?;
    let schema = input("SURREALKIT_EXEC_SCHEMA_DIGEST")?;
    if schema.strip_prefix("sha256:").is_none_or(|digest| {
        digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        return Err(refused("suite semantic schema identity is malformed"));
    }
    let deadline = Instant::now() + Duration::from_secs(20);
    let client = Client::connect(address, wire::RESULT_MESSAGE_BYTES)
        .await
        .map_err(|_| refused("contract reader connection failed"))?;
    let outcome=async {
        client.authenticate(&token).request_context(RequestContext::control(deadline)).await
            .map_err(|_|refused("suite token authentication failed"))?;
        client.use_ns(&namespace).use_db(&database).request_context(RequestContext::control(deadline)).await
            .map_err(|_|refused("suite context selection failed"))?;
        let mut result=bounded_query(client.query("SELECT * FROM ONLY canonical_guards:exact; SELECT * FROM ONLY canonical_payload_blocks:exact;")
            .request_context(RequestContext::control(deadline))).await
            .map_err(|_|refused("independent contract read failed"))?;
        let guard:Option<Object>=result.take(0).map_err(|_|refused("guard row is absent or malformed"))?;
        let guard=guard.ok_or_else(||refused("guard row is absent"))?;
        let guard=wire::decode_canonical_guards(guard).map_err(|_|refused("native guard decoding refused"))?;
        let payload:Option<Object>=result.take(1).map_err(|_|refused("payload row is absent or malformed"))?;
        let payload=payload.ok_or_else(||refused("payload row is absent"))?;
        let payload=wire::decode_canonical_payload_blocks(payload).map_err(|_|refused("native payload decoding refused"))?;
        if guard.generation!=cli.expected_generation || payload.ordinal!=u64::MAX || payload.payload.as_slice()!=[0,1,0,255] {
            return Err(refused("independent unsigned or byte expectation differs"));
        }
        Ok(())
    }.await;
    if outcome.is_ok() && cli.wait_for_interruption {
        let _reader = std::fs::File::open("exec-wait")?;
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    let drained = client.disconnect().await;
    match (outcome, drained) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(_)) => Err(refused("contract reader connection did not drain")),
        (Ok(()), Ok(())) => Ok(()),
    }
}
fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = tokio::runtime::Runtime::new()
        .map_err(NativeServiceError::from)
        .and_then(|runtime| runtime.block_on(read(cli)));
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("contract reader: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
