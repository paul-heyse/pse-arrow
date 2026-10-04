// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::print_stdout,
    reason = "build scripts emit Cargo link directives"
)]
//! Compile the typed MPIUNI bridge against a qualified pinned PETSc prefix.
use sha2::{Digest, Sha256};
use std::{env, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=PETSC_DIR");
    println!("cargo:rerun-if-changed=bridge/pse_petsc.c");
    println!("cargo:rerun-if-changed=bridge/pse_petsc.h");
    if env::var_os("CARGO_FEATURE_LINK").is_none() {
        return Ok(());
    }
    let prefix = env::var_os("PETSC_DIR")
        .map(PathBuf::from)
        .ok_or("PETSC_DIR must name the qualified PETSc prefix for the link feature")?;
    let receipt = std::fs::read(prefix.join(".complete.json"))?;
    let value: serde_json::Value = serde_json::from_slice(&receipt)?;
    let version = value["identity"]["pin"]["version"]
        .as_str()
        .ok_or("PETSc qualified source version missing")?;
    let mut identity = Sha256::new();
    identity.update(&receipt);
    for file in ["bridge/pse_petsc.c", "bridge/pse_petsc.h"] {
        identity.update(std::fs::read(file)?);
    }
    let build_id: String = identity
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("cargo:rustc-env=PSE_PETSC_BUILD_ID={build_id}");
    println!("cargo:rustc-env=PSE_PETSC_VERSION={version}");
    cc::Build::new()
        .std("c11")
        .file("bridge/pse_petsc.c")
        .include(prefix.join("include"))
        .try_compile("pse_petsc_bridge")?;
    println!(
        "cargo:rustc-link-search=native={}",
        prefix.join("lib").display()
    );
    println!("cargo:rustc-link-lib=petsc");
    Ok(())
}
