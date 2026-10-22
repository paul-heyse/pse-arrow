// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed settings documents across the Python boundary (ADR-0116 Outcome 7; Plan 22 X13).
//! Python constructs the generated msgspec document types (`pse.contracts.documents`) and
//! passes their concrete document class; serde decodes its native transport into the pse-owned Rust type, which refuses
//! an unknown version, field or vocabulary member and every single-value domain violation.
//! The rules that relate fields stay native admission. There is no string-to-enum table and
//! no keyword projection here.
use super::invalid;
use pyo3::prelude::*;

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
