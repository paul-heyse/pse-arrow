// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared immutable module publication records; values confer no execution authority.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Declared guest limits shared by construction and canonical artifact admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleLimits {
    /// Guest linear-memory bound.
    pub max_memory_bytes: u64,
    /// Guest controller-pool bound.
    pub max_pool_size: u32,
    /// Guest execution deadline in milliseconds.
    pub max_execution_time_ms: u64,
    /// Guest-local KV entry bound.
    pub max_kv_entries: u64,
    /// Guest-local KV value bound.
    pub max_kv_value_bytes: u64,
    /// Enforce the declared guest deadline.
    pub strict_timeout: bool,
}

/// Existing module receipt wire shape, consumed by canonical artifact admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudyModuleRecord {
    /// Artifact interpretation and qualification variant.
    pub kind: String,
    /// Exact immutable package path.
    pub artifact: PathBuf,
    /// Package byte identity.
    pub sha256: String,
    /// Exact package extent.
    pub bytes: u64,
    /// Exact SDK release.
    pub sdk_version: String,
    /// Guest ABI interpretation.
    pub abi: u32,
    /// Retained construction provenance.
    pub inputs: serde_json::Map<String, Value>,
    /// Module-owned guest limits.
    pub limits: ModuleLimits,
}
