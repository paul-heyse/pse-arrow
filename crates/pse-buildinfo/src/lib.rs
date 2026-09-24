// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Build provenance: rustc version, profile, git sha and the embedded lockfiles
//! (blueprint §3.1, plan §5).
//!
//! `pse-catalog` uses these for the snapshot manifest's `toolchain.*` fields and `pse-py`
//! re-exports them as `pse._native.build_info()`, which `build_info_matches_checkout`
//! compares against the working tree's lockfiles: an extension built against a stale
//! `Cargo.lock` is a test failure, not a surprise at run time.
//!
//! This crate delegates stable build-input hashing to `pse-ids`, the workspace's only
//! hasher (blueprint §5.1).

/// The workspace version (`CARGO_PKG_VERSION`): one version authority for the crates, the
/// wheel and the git tag.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// `rustc` release the extension was compiled with, e.g. `1.98.1`.
pub const RUSTC_VERSION: &str = env!("PSE_RUSTC_VERSION");

/// Cargo profile the extension was compiled with (`debug`, `release`, `dist`, ...).
pub const PROFILE: &str = env!("PSE_PROFILE");

/// `git rev-parse HEAD` at build time, or `sdist` when the source is not a git checkout.
pub const GIT_SHA: &str = env!("PSE_GIT_SHA");

/// The workspace `Cargo.lock` as of the build. Empty only if the file was absent.
pub const CARGO_LOCK: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/cargo.lock"));

/// The workspace `uv.lock` as of the build. Empty until the Python skeleton lands
/// (plan §11 step 4).
pub const UV_LOCK: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/uv.lock"));

/// Actual product source bytes, including uncommitted generated and handwritten changes.
pub const SOURCE_IDENTITY: pse_ids::ContentHash = pse_ids::ContentHash::from_bytes(
    *include_bytes!(concat!(env!("OUT_DIR"), "/source.identity")),
);
/// Locks, toolchain, target and captured build configuration.
pub const BUILD_IDENTITY: pse_ids::ContentHash =
    pse_ids::ContentHash::from_bytes(*include_bytes!(concat!(env!("OUT_DIR"), "/build.identity")));
/// Artifact producers must also supply their actual consumer feature and plugin ABI contracts.
pub const PERSISTENT_IDENTITY_QUALIFIED: bool = !CARGO_LOCK.is_empty() && !UV_LOCK.is_empty();
#[cfg(test)]
#[path = "../identity.rs"]
mod identity;
