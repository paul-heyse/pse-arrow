// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::print_stdout,
    reason = "a build script talks to cargo on stdout"
)]

//! Captures build provenance as `rustc-env` values and stages the two lockfiles in
//! `OUT_DIR` so `lib.rs` can `include_bytes!` them unconditionally.
//!
//! `include_bytes!("../../../uv.lock")` cannot be used directly: `uv.lock` does not exist
//! until the Python skeleton lands (plan §11 step 4) and a missing path is a hard compile
//! error. Staging through `OUT_DIR` makes "absent" mean "zero bytes" instead, and
//! `Cargo.lock` goes through the same path for symmetry so both constants are produced by
//! one mechanism.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    // crates/pse-buildinfo -> crates -> workspace root
    let workspace_root = manifest_dir.join("../..");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap_or_default());

    stage(&workspace_root, &out_dir, "Cargo.lock", "cargo.lock");
    stage(&workspace_root, &out_dir, "uv.lock", "uv.lock");

    println!("cargo:rustc-env=PSE_RUSTC_VERSION={}", rustc_version());
    println!(
        "cargo:rustc-env=PSE_PROFILE={}",
        env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned())
    );
    println!("cargo:rustc-env=PSE_GIT_SHA={}", git_sha(&workspace_root));
}

/// Copies `<root>/<src>` to `<out_dir>/<dst>`, or writes an empty file when it is absent.
/// The `rerun-if-changed` is emitted either way so the file appearing triggers a rebuild.
fn stage(root: &Path, out_dir: &Path, src: &str, dst: &str) {
    let from = root.join(src);
    println!("cargo:rerun-if-changed={}", from.display());
    let bytes = fs::read(&from).unwrap_or_default();
    let to = out_dir.join(dst);
    if let Err(err) = fs::write(&to, bytes) {
        println!(
            "cargo:warning=pse-buildinfo: could not write {}: {err}",
            to.display()
        );
    }
}

/// The `release:` line of `rustc -vV`, e.g. `1.98.1`. Falls back to `RUSTC_VERSION_HINT`
/// only so a sandboxed build does not fail outright.
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

/// `git rev-parse HEAD`, or `sdist` when there is no git (a build from the sdist, or the
/// seeding commit that has no HEAD yet).
fn git_sha(root: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
            if sha.is_empty() {
                "sdist".to_owned()
            } else {
                sha
            }
        }
        _ => "sdist".to_owned(),
    }
}
