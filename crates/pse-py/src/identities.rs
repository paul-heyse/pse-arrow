// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delegate all identity text admission and normalization to pse-ids.
use pse_ids::{ContentHash, SemanticId};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};

#[pyfunction]
#[pyo3(signature = (text) -> "bytes")]
pub(crate) fn semantic_id_from_hex<'py>(
    py: Python<'py>,
    text: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let id =
        SemanticId::parse_hex(text).map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(PyBytes::new(py, id.as_bytes()))
}
#[pyfunction]
#[pyo3(signature = (value: "bytes") -> "str")]
pub(crate) fn semantic_id_to_hex(value: &[u8]) -> PyResult<String> {
    Ok(SemanticId::from_bytes(
        value
            .try_into()
            .map_err(|_| PyValueError::new_err("invalid semantic identity width"))?,
    )
    .to_hex())
}
#[pyfunction]
#[pyo3(signature = (text) -> "bytes")]
pub(crate) fn content_hash_from_prefixed<'py>(
    py: Python<'py>,
    text: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let hash = ContentHash::parse_prefixed(text)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(PyBytes::new(py, hash.as_bytes()))
}
#[pyfunction]
#[pyo3(signature = (value: "bytes") -> "str")]
pub(crate) fn content_hash_to_prefixed(value: &[u8]) -> PyResult<String> {
    Ok(ContentHash::from_bytes(
        value
            .try_into()
            .map_err(|_| PyValueError::new_err("invalid content digest width"))?,
    )
    .to_prefixed())
}
