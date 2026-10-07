// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::print_stdout,
    reason = "a build script talks to cargo on stdout"
)]
//! Compiled implementation metadata only. Outer observation belongs to deployment.
use std::{env, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=PSE_RUSTC_VERSION={}", rustc_version());
    println!(
        "cargo:rustc-env=PSE_PROFILE={}",
        env::var("PROFILE").unwrap_or_else(|_| "unknown".into())
    );
}
/// The selected compiler release line, or explicit unknown if its query fails.
fn rustc_version() -> String {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc).arg("-vV").output();
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("release: "))
            .map_or("unknown", str::trim)
            .to_owned(),
        _ => "unknown".to_owned(),
    }
}
