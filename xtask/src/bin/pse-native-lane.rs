// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private store-free native root worker. This binary has no public DTO admission mode.
#![allow(
    unsafe_code,
    reason = "ADR-0185 controlled native child composition root consumes only the private inherited socket role"
)]
#[allow(
    clippy::print_stderr,
    reason = "private process host retains telemetry refusal on stderr"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--private-native-root") {
        return Err("pse-native-lane requires its private deployment launcher".into());
    }
    let attempt = pse_ids::SemanticId::parse_hex(&args.next().ok_or("missing attempt")?)?;
    let parent = args.next().ok_or("missing parent")?.parse::<u32>()?;
    if args.next().is_some() {
        return Err("unexpected private worker arguments".into());
    }
    // This private process exits as soon as its real native owner has joined.
    // Final SDK batches are best effort and may be lost; a slow Collector must not
    // extend process reaping beyond the original scientific execution deadline.
    if pse_runtime::tooling::telemetry::initialize_private_child_host().is_err() {
        eprintln!("pse-telemetry: native-host-initialization-refused");
    }
    let carrier = match std::env::var(pse_runtime::tooling::telemetry::CHILD_CARRIER_ENV) {
        Ok(value) => match pse_runtime::tooling::telemetry::Carrier::from_json(&value) {
            Ok(carrier) => carrier,
            Err(_) => {
                eprintln!("pse-telemetry: native-child-carrier-invalid");
                Default::default()
            }
        },
        Err(_) => Default::default(),
    };
    let telemetry = carrier.operation_span("native.child.owner");
    let dispatch = tracing::dispatcher::get_default(Clone::clone);
    std::thread::Builder::new()
        .name("native-child-owner".into())
        .stack_size(pse_runtime::math::native_child::NATIVE_CHILD_STACK_BYTES)
        .spawn(move || {
            let _dispatch = tracing::dispatcher::set_default(&dispatch);
            let _telemetry = telemetry.entered();
            // SAFETY: this binary is a controlled deployment role. serve_private rejects
            // generic stdin pipes/files and checks the actual parent. Its launcher owns
            // the private socket, authenticates this executable/build/attempt, supplies
            // compiler-produced source, retains the original clock/charges, and joins us.
            unsafe { pse_runtime::math::native_child::serve_private(attempt, parent) }
        })?
        .join()
        .map_err(|_| "native child owner panicked")??;
    Ok(())
}
