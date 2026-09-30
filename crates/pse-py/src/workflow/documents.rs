// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The one Python document-content boundary; workers retain these owned bytes.
use pyo3::prelude::*;
use pyo3::type_hint_union;
use pyo3::types::{PyBytes, PyString};

pub(crate) struct DocumentContent(pub(crate) Vec<u8>);

impl FromPyObject<'_, '_> for DocumentContent {
    type Error = PyErr;

    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_union!(
        pyo3::type_hint_identifier!("builtins", "str"),
        pyo3::type_hint_identifier!("builtins", "bytes")
    );

    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if let Ok(text) = value.cast::<PyString>() {
            return Ok(Self(text.to_str()?.as_bytes().to_vec()));
        }
        if let Ok(bytes) = value.cast::<PyBytes>() {
            return Ok(Self(bytes.as_bytes().to_vec()));
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "document content must be str or bytes",
        ))
    }
}
