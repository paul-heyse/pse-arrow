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

/// Decode an owned operation document exactly once, before native contextual admission.
pub(crate) fn decode<T: serde::de::DeserializeOwned>(
    py: Python<'_>,
    operation: &str,
    bytes: &[u8],
    allowance: usize,
) -> PyResult<T> {
    if bytes.len() > allowance {
        return Err(crate::workflow::invalid(
            py,
            format!("{operation} document extent"),
        ));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| crate::workflow::invalid(py, format!("{operation}: {error}")))
}
/// Project an owned result document; no adapter-owned JSON shape is introduced.
pub(crate) fn encode<T: serde::Serialize>(py: Python<'_>, value: &T) -> PyResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| crate::workflow::invalid(py, error.to_string()))
}

/// Omission is resolved by the Rust operation's default implementation.
pub(crate) fn controls<T: serde::de::DeserializeOwned + Default>(
    py: Python<'_>,
    operation: &str,
    bytes: Option<&[u8]>,
    allowance: usize,
) -> PyResult<T> {
    bytes.map_or_else(
        || Ok(T::default()),
        |bytes| decode(py, operation, bytes, allowance),
    )
}

/// A generated Python document projected from the actual Rust serde owner.
#[derive(Clone, Debug)]
pub(crate) struct DocumentValue<T>(pub(crate) T);
pub(crate) trait DocumentType {
    const NAME: &'static str;
}
impl DocumentType for pse_model::diagnostic::BoundaryDiagnostic {
    const NAME: &'static str = "BoundaryDiagnostic";
}
impl DocumentType for pse_runtime::workflow::DiagnosticCauseDocument {
    const NAME: &'static str = "DiagnosticCauseDocument";
}
impl DocumentType for pse_runtime::workflow::DiagnosticSpanDocument {
    const NAME: &'static str = "DiagnosticSpanDocument";
}
impl DocumentType for pse_runtime::workflow::DiagnosticNoteDocument {
    const NAME: &'static str = "DiagnosticNoteDocument";
}
impl DocumentType for pse_runtime::workflow::DiagnosticAnnotationDocument {
    const NAME: &'static str = "DiagnosticAnnotationDocument";
}
impl DocumentType for pse_runtime::workflow::DiagnosticContextDocument {
    const NAME: &'static str = "DiagnosticContextDocument";
}
impl DocumentType for pse_model::diagnostic::SourceLocation {
    const NAME: &'static str = "SourceLocation";
}
impl DocumentType for pse_runtime::ResourceReport {
    const NAME: &'static str = "ResourceReport";
}
impl DocumentType for pse_engine::cache_service::CacheReport {
    const NAME: &'static str = "CacheReport";
}
impl DocumentType for pse_runtime::TableName {
    const NAME: &'static str = "TableName";
}
impl DocumentType for pse_buildinfo::BuildInfo {
    const NAME: &'static str = "BuildInfo";
}
impl DocumentType for pse_runtime::workflow::ProgressEventDocument {
    const NAME: &'static str = "ProgressEventDocument";
}
impl DocumentType for pse_runtime::workflow::RouteDocument {
    const NAME: &'static str = "RouteDocument";
}
impl DocumentType for pse_runtime::workflow::EligibilityDocument {
    const NAME: &'static str = "EligibilityDocument";
}
impl DocumentType for pse_runtime::workflow::IneligibleDocument {
    const NAME: &'static str = "IneligibleDocument";
}
impl<'py, T: serde::Serialize + DocumentType> IntoPyObject<'py> for DocumentValue<T> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.documents", T::NAME);
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let bytes = encode(py, &self.0)?;
        let target = py.import("pse.contracts.documents")?.getattr(T::NAME)?;
        py.import("pse.codec")?
            .call_method1("decode_json", (PyBytes::new(py, &bytes), target))
    }
}

/// A generated Python input document decoded into its actual Rust owner.
pub(crate) struct DocumentInput<T>(pub(crate) T);
impl DocumentType for pse_model::numerics::NumericalPolicy {
    const NAME: &'static str = "NumericalPolicy";
}
impl<T: serde::de::DeserializeOwned + DocumentType> FromPyObject<'_, '_> for DocumentInput<T> {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.documents", T::NAME);
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let py = value.py();
        let target = py.import("pse.contracts.documents")?.getattr(T::NAME)?;
        if !value.is_instance(&target)? {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "expected {} document",
                T::NAME
            )));
        }
        let encoded = py
            .import("pse.codec")?
            .call_method1("encode_json", (value,))?;
        let bytes = encoded.cast::<PyBytes>()?;
        decode(py, T::NAME, bytes.as_bytes(), 1 << 20).map(Self)
    }
}
