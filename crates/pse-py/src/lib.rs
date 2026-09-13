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
//! Phase 0 exposes only the build-provenance surface: `__version__` and `build_info()`,
//! which `pse._build.BuildInfo` structures and `build_info_matches_checkout` asserts
//! against the checkout's lockfiles (plan §5).

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

/// Build provenance of the compiled extension.
///
/// Keys: `version`, `rustc_version`, `profile`, `git_sha`, `lockfile_hash`,
/// `cargo_lock_bytes`, `uv_lock_bytes`. The lockfiles cross as the bytes that were
/// embedded at build time; `pse._build` hashes them on the Python side (hashlib) so that
/// no second hasher exists in Rust — `pse-ids` is the sole hasher (blueprint §5.1) and the
/// blake3 `lockfile_hash` is wired through `pse-catalog`'s manifest in phase 1.
#[pyfunction]
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

/// The `pse._native` module.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", pse_buildinfo::VERSION)?;
    m.add_function(wrap_pyfunction!(build_info, m)?)?;
    Ok(())
}
