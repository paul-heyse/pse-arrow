// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Python value-kind admission; numeric ranges remain native extraction/policy checks.
use pyo3::{prelude::*, types::PyBool};
pub(crate) fn extract<'a, 'py, T>(value: &'a Bound<'py, PyAny>) -> PyResult<T>
where
    T: FromPyObject<'a, 'py>,
{
    if value.is_instance_of::<PyBool>() && std::any::type_name::<T>() != "bool" {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "boolean is not an integer setting",
        ));
    }
    value.extract::<T>().map_err(Into::into)
}
