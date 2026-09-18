// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Volatile native resource observations; none are model or dependency authority.
use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "cache_statistics",
        S::Sidecar,
        &["name"],
        vec![
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("policy_limit_bytes", T::nonnegative(i64::MAX)),
            column("capacity_bytes", T::nonnegative(i64::MAX)),
            column("retained_bytes", T::nonnegative(i64::MAX)),
            column("live_bytes", T::nonnegative(i64::MAX)).optional(),
            column("pinned_bytes", T::nonnegative(i64::MAX)).optional(),
            column("inflight_bytes", T::nonnegative(i64::MAX)).optional(),
            column("active_loads", T::nonnegative(i64::MAX)).optional(),
            column("entries", T::nonnegative(i64::MAX)),
            column("hits", T::nonnegative(i64::MAX)),
            column("misses", T::nonnegative(i64::MAX)),
            column("bypasses", T::nonnegative(i64::MAX)),
            column("evictions", T::nonnegative(i64::MAX)).optional(),
        ],
        "Observed native cache counters; NULL means unavailable, not zero.",
    );
    relation(
        builder,
        N::Runtime,
        "cache_entry_statistics",
        S::Sidecar,
        &["cache", "key"],
        vec![
            column("cache", T::native(arrow_schema::DataType::Utf8)),
            column("key", T::native(arrow_schema::DataType::Utf8)),
            column("bytes", T::nonnegative(i64::MAX)),
            column("hits", T::nonnegative(i64::MAX)),
        ],
        "Explicit bounded cache entry diagnostics; not execution identity.",
    );
    relation(
        builder,
        N::Runtime,
        "execution_statistics",
        S::Sidecar,
        &["name"],
        vec![
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("count", T::nonnegative(i64::MAX)).optional(),
        ],
        "Actual native planning, load and post-commit acceleration observations.",
    );
}
