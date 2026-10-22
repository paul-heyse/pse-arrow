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

/// Native semantic role; Python NewType is an annotation, not a runtime witness.
pub(crate) trait IdentityRole: Copy {
    const NAME: &'static str;
    const MODULE: &'static str = "pse.contracts.identities";
    fn from_semantic(value: SemanticId) -> Self;
    fn as_semantic(self) -> SemanticId;
}
/// Strict existing SemanticId value crossing, with registry-owned static role hints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Identity<R>(pub(crate) R);
fn semantic_type(py: Python<'_>) -> PyResult<&Bound<'_, pyo3::types::PyType>> {
    static TYPE: pyo3::sync::PyOnceLock<Py<pyo3::types::PyType>> = pyo3::sync::PyOnceLock::new();
    TYPE.import(py, "pse.contracts.values", "SemanticId")
}
impl<R: IdentityRole> FromPyObject<'_, '_> for Identity<R> {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_identifier!(R::MODULE, R::NAME);
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if !value.is_instance(semantic_type(value.py())?)? {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "expected SemanticId for {}",
                R::NAME
            )));
        }
        let value = value.cast::<PyBytes>()?;
        let bytes = value.as_bytes();
        let raw =
            SemanticId::from_bytes(bytes.try_into().map_err(|_| {
                PyValueError::new_err("semantic identity requires exactly 16 bytes")
            })?);
        Ok(Self(R::from_semantic(raw)))
    }
}
macro_rules! roles {
    ($($name:ident),* $(,)?) => {$(
        impl IdentityRole for pse_model::generated::identities::$name {
            const NAME: &'static str = stringify!($name);
            fn from_semantic(value: SemanticId) -> Self { Self::from_id(value) }
            fn as_semantic(self) -> SemanticId { self.as_id() }
        }
    )*};
}
pse_model::for_each_semantic_identity!(roles);

impl IdentityRole for SemanticId {
    const NAME: &'static str = "SemanticId";
    const MODULE: &'static str = "pse.contracts.values";
    fn from_semantic(value: SemanticId) -> Self {
        value
    }
    fn as_semantic(self) -> SemanticId {
        self
    }
}
impl<'py, R: IdentityRole> IntoPyObject<'py> for Identity<R> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!(R::MODULE, R::NAME);
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        semantic_type(py)?.call1((PyBytes::new(py, self.0.as_semantic().as_bytes()),))
    }
}

/// Content identity uses the existing 32-byte value class at the native boundary.
pub(crate) struct HashValue(pub(crate) ContentHash);
fn hash_type(py: Python<'_>) -> PyResult<&Bound<'_, pyo3::types::PyType>> {
    static TYPE: pyo3::sync::PyOnceLock<Py<pyo3::types::PyType>> = pyo3::sync::PyOnceLock::new();
    TYPE.import(py, "pse.contracts.values", "ContentHash")
}
impl FromPyObject<'_, '_> for HashValue {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.values", "ContentHash");
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if !value.is_instance(hash_type(value.py())?)? {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "expected ContentHash",
            ));
        }
        Ok(Self(ContentHash::from_bytes(
            value
                .cast::<PyBytes>()?
                .as_bytes()
                .try_into()
                .map_err(|_| PyValueError::new_err("content identity requires exactly 32 bytes"))?,
        )))
    }
}
impl<'py> IntoPyObject<'py> for HashValue {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.values", "ContentHash");
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        hash_type(py)?.call1((PyBytes::new(py, self.0.as_bytes()),))
    }
}
