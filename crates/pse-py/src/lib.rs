// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    unsafe_code,
    reason = "pyo3 expansions carry the CPython ABI glue (blueprint §21)"
)]

//! pyo3 + pyo3-arrow extension module (cdylib) that is the whole Python boundary
//! (blueprint §3.2, §21).
//!
//! The module is imported as `pse._native`; nothing else in the Python package is
//! allowed to import it directly (ast-grep rule `no-direct-native-import`).
//!
//! Exposes build provenance and read-only exact Delta publication inspection. Named table
//! streams preserve the catalog's final-buffer reservations through Arrow C Stream
//! consumers; opening checks exact declarations under one explicit process budget.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

mod identities;
mod inspection;
mod workflow;

/// Build provenance of the compiled extension.
///
/// Keys: `version`, `rustc_version`, `profile`, `git_sha`, `lockfile_hash`,
/// `cargo_lock_bytes`, `uv_lock_bytes`. The lockfiles cross as the bytes that were
/// embedded at build time; `pse._build` hashes them on the Python side (hashlib) so that
/// no second hasher exists in Rust — `pse-ids` is the sole hasher (blueprint §5.1) and the
/// blake3 `lockfile_hash` is wired through `pse-catalog`'s manifest in phase 1.
#[pyfunction]
#[pyo3(signature = () -> "dict[str, str | bytes]")]
fn build_info(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("version", pse_buildinfo::VERSION)?;
    dict.set_item("rustc_version", pse_buildinfo::RUSTC_VERSION)?;
    dict.set_item("profile", pse_buildinfo::PROFILE)?;
    dict.set_item("git_sha", pse_buildinfo::GIT_SHA)?;
    dict.set_item("lockfile_hash", "")?;
    dict.set_item(
        "cargo_lock_bytes",
        PyBytes::new(py, pse_buildinfo::CARGO_LOCK),
    )?;
    dict.set_item("uv_lock_bytes", PyBytes::new(py, pse_buildinfo::UV_LOCK))?;
    Ok(dict)
}

/// Compiled registry identity; does not assemble or inspect the runtime registry.
#[pyfunction]
fn registry_fingerprint() -> String {
    pse_catalog::inspection::REGISTRY_FINGERPRINT.to_prefixed()
}

/// Immutable admitted data inspection and build provenance.
#[pymodule]
mod _native {
    #[pymodule_export]
    use super::{
        build_info,
        identities::{
            content_hash_from_prefixed, content_hash_to_prefixed, semantic_id_from_hex,
            semantic_id_to_hex,
        },
        inspection::{
            CacheReport, CacheSettings, DiagnosticAnnotation, DiagnosticCause, DiagnosticContext,
            DiagnosticNote, DiagnosticReport, DiagnosticSpan, EngineSettings, InspectionError,
            Publication, ResourceConsumer, ResourceReport, TableName, TableStream,
            open_publication,
        },
        registry_fingerprint,
        workflow::{
            NativeModelRevision, NativePhysicalContext, NativePreparedCase,
            NativePreparedOperation, NativePublicationAttempt, NativeRunHandle, NativeRunResult,
            NativeRuntime, ProgressEvent, SimulationSettings, SolveSettings, SolverCapability,
        },
    };

    #[pymodule_export]
    #[expect(non_upper_case_globals, reason = "Python module version convention")]
    const __version__: &str = pse_buildinfo::VERSION;
}
