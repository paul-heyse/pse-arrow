// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical projection of source-owned vocabulary into its generated Python enum.
use pyo3::prelude::*;
use pyo3::{sync::PyOnceLock, types::PyType};
pub(crate) trait RegistryEnum: std::str::FromStr {
    const NAME: &'static str;
    fn wire(&self) -> &'static str;
    fn python_type(py: Python<'_>) -> PyResult<&Bound<'_, PyType>>;
}
pub(crate) struct EnumValue<T>(pub(crate) T);
impl<T> From<T> for EnumValue<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
impl<T: RegistryEnum> FromPyObject<'_, '_> for EnumValue<T> {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.enums", T::NAME);
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        if !value.is_instance(T::python_type(value.py())?)? {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "expected pse.contracts.enums.{}", T::NAME
            )));
        }
        let wire: String = value.getattr("value")?.extract()?;
        wire.parse().map(Self).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(format!("unknown {} member", T::NAME))
        })
    }
}
impl<'py, T: RegistryEnum> IntoPyObject<'py> for EnumValue<T> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;
    const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
        pyo3::type_hint_identifier!("pse.contracts.enums", T::NAME);
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        T::python_type(py)?.call1((self.0.wire(),))
    }
}
macro_rules! vocabularies {
    ($($name:ident),* $(,)?) => {$(
        impl RegistryEnum for pse_model::generated::enums::$name {
            const NAME: &'static str = stringify!($name);
            fn wire(&self) -> &'static str { self.as_str() }
            fn python_type(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
                // A distinct concrete impl owns each cell. A generic-function static
                // would incorrectly share one Python class between vocabularies.
                static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
                TYPE.import(py, "pse.contracts.enums", Self::NAME)
            }
        }
    )*};
}
pse_model::for_each_registry_enum!(vocabularies);
