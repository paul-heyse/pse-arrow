// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::print_stdout,
    reason = "build scripts emit Cargo link directives"
)]
//! Compile the exception shield against a qualified pinned Uno prefix.
use sha2::{Digest, Sha256};
use std::{env, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=UNO_DIR");
    println!("cargo:rerun-if-env-changed=IPOPT_DIR");
    println!("cargo:rerun-if-changed=bridge/pse_uno.cpp");
    println!("cargo:rerun-if-changed=bridge/pse_uno.h");
    if env::var_os("CARGO_FEATURE_LINK").is_none() {
        return Ok(());
    }
    let prefix = env::var_os("UNO_DIR")
        .map(PathBuf::from)
        .ok_or("UNO_DIR must name the qualified Uno prefix for the link feature")?;
    let receipt = std::fs::read(prefix.join(".complete.json"))?;
    let value: serde_json::Value = serde_json::from_slice(&receipt)?;
    let version = value["identity"]["pin"]["revision"]
        .as_str()
        .ok_or("Uno qualified source revision missing")?;
    let mut identity = Sha256::new();
    identity.update(&receipt);
    for file in ["bridge/pse_uno.cpp", "bridge/pse_uno.h"] {
        identity.update(std::fs::read(file)?);
    }
    let build_id: String = identity
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("cargo:rustc-env=PSE_UNO_BUILD_ID={build_id}");
    println!("cargo:rustc-env=PSE_UNO_VERSION={version}");
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("bridge/pse_uno.cpp")
        .include(prefix.join("include/uno"))
        .try_compile("pse_uno_bridge")?;
    println!(
        "cargo:rustc-link-search=native={}",
        prefix.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=uno");
    println!("cargo:rustc-link-lib=stdc++");
    let blas = env::var_os("IPOPT_DIR")
        .map(PathBuf::from)
        .ok_or("Uno links the existing LP64 BLAS provider: set IPOPT_DIR")?;
    println!(
        "cargo:rustc-link-search=native={}",
        blas.join("lib").display()
    );
    // lib.rs carries the provider's required no-as-needed native-link modifiers.
    // Keeping them on Rust native libraries preserves retention in downstream links.
    Ok(())
}
