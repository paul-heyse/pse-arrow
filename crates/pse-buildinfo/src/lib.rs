// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Captured implementation metadata and current deployment observations.
//!
//! Compiled implementation metadata and fresh outer deployment observations have
//! separate lifetimes. Actual selected artifacts qualify through reviewed external
//! receipts. `build_info()` leaves unembedded outer fields explicitly absent.
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

#[path = "../identity.rs"]
pub mod identity;
pub use identity::{
    DEPLOYMENT_RECEIPT_VERSION, DeploymentAssociation, FileObservation, OuterObservation,
    loaded_module_path, observe_deployment, observe_outer, verify_loaded_module,
    verify_receipt_artifact, workspace_root,
};

/// Build provenance of the compiled product, projected without recomputing its identity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildInfo {
    /// Workspace and wheel version.
    pub version: String,
    /// Captured compiler release.
    pub rustc_version: String,
    /// Captured Cargo build profile.
    pub profile: String,
    /// Captured Git commit; empty when the implementation does not embed outer context.
    pub git_sha: String,
    /// Historical public lockfile digest field; empty when no such evidence was produced.
    pub lockfile_hash: String,
    /// Captured Cargo lockfile checksum; empty when no lockfile is embedded.
    pub cargo_lock_sha256: String,
    /// Captured uv lockfile checksum; empty when no lockfile is embedded.
    pub uv_lock_sha256: String,
}
impl BuildInfo {
    /// Project only implementation metadata actually embedded during compilation.
    pub fn captured() -> Self {
        Self {
            version: VERSION.into(),
            rustc_version: RUSTC_VERSION.into(),
            profile: PROFILE.into(),
            git_sha: String::new(),
            lockfile_hash: String::new(),
            cargo_lock_sha256: String::new(),
            uv_lock_sha256: String::new(),
        }
    }
}
