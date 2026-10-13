// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private store-free native root worker. This binary has no public DTO admission mode.
#![allow(unsafe_code, reason = "ADR-0185 controlled native child composition root consumes only the private inherited socket role")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--private-native-root") {
        return Err("pse-native-lane requires its private deployment launcher".into());
    }
    let attempt = pse_ids::SemanticId::parse_hex(&args.next().ok_or("missing attempt")?)?;
    let parent = args.next().ok_or("missing parent")?.parse::<u32>()?;
    if args.next().is_some() { return Err("unexpected private worker arguments".into()); }
    std::thread::Builder::new().name("native-child-owner".into())
        .stack_size(pse_runtime::math::native_child::NATIVE_CHILD_STACK_BYTES)
        .spawn(move || {
            // SAFETY: this binary is a controlled deployment role. serve_private rejects
            // generic stdin pipes/files and checks the actual parent. Its launcher owns
            // the private socket, authenticates this executable/build/attempt, supplies
            // compiler-produced source, retains the original clock/charges, and joins us.
            unsafe { pse_runtime::math::native_child::serve_private(attempt, parent) }
        })?.join().map_err(|_| "native child owner panicked")??;
    Ok(())
}
