// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed settings documents across the Python boundary (ADR-0116 Outcome 7; Plan 22 X13).
//! Python constructs the generated msgspec document types (`pse.contracts.documents`) and
//! passes their JSON encoding; serde decodes it into the pse-owned Rust type, which refuses
//! an unknown version, field or vocabulary member and every single-value domain violation.
//! The rules that relate fields stay native admission. There is no string-to-enum table and
//! no keyword projection here.
use super::invalid;
use pse_backend_native::dynamics;
use pse_runtime::math::{settings::SolveSettings, solves::SolverProfile};
use pyo3::prelude::*;
use serde::de::DeserializeOwned;

/// Largest settings document accepted from Python.
const DOCUMENT_BYTES: usize = 1 << 20;

/// Decode one typed settings document; its serde refusal is the Python error.
fn document<T: DeserializeOwned>(py: Python<'_>, what: &str, bytes: &[u8]) -> PyResult<T> {
    super::documents::decode(py, what, bytes, DOCUMENT_BYTES)
}

/// The solver profile of an encoded `SolveSettings` document, after native admission of the
/// rules that relate its fields.
pub(super) fn solve_profile(py: Python<'_>, bytes: &[u8]) -> PyResult<SolverProfile> {
    super::documents::decode_versioned::<SolveSettings, 2>(
        py,
        "solve settings",
        bytes,
        DOCUMENT_BYTES,
    )?
    .profile()
    .map_err(|e| crate::inspection::errors::diagnostic(py, &e))
}

/// An encoded `DiffsolSettings` document.
pub(super) fn diffsol(py: Python<'_>, bytes: &[u8]) -> PyResult<dynamics::DiffsolSettings> {
    document(py, "Diffsol settings", bytes)
}

/// An encoded `IdasSettings` document.
pub(super) fn idas(py: Python<'_>, bytes: &[u8]) -> PyResult<dynamics::IdasSettings> {
    document(py, "IDAS settings", bytes)
}

/// An encoded `AdjointSettings` document.
pub(super) fn adjoint(py: Python<'_>, bytes: &[u8]) -> PyResult<dynamics::AdjointSettings> {
    document(py, "adjoint settings", bytes)
}

/// A boundary name, parsed by the owning type's serde spelling (a registry enum's is its
/// `as_str`); a refusal lists every accepted name.
pub(super) fn named<T: DeserializeOwned>(py: Python<'_>, what: &str, name: &str) -> PyResult<T> {
    serde_json::from_value(serde_json::Value::String(name.into()))
        .map_err(|e| invalid(py, format!("{what}: {e}")))
}

pub(super) fn numerical_policy(
    py: Python<'_>,
    value: Option<crate::documents::DocumentInput<pse_model::numerics::NumericalPolicy>>,
) -> PyResult<pse_model::numerics::NumericalPolicy> {
    let policy: pse_model::numerics::NumericalPolicy = match value {
        None => Default::default(),
        Some(value) => value.0,
    };
    policy.validate().map_err(|e| invalid(py, e.to_string()))?;
    Ok(policy)
}
