// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Arrow transport extraction; runtime owns registry and physical admission.
use pyo3::prelude::*;

pub(super) struct PhysicalInput(pub(super) pyo3_arrow::input::AnyRecordBatch);
impl FromPyObject<'_, '_> for PhysicalInput {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_identifier!("pse._physical", "ArrowInput");
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        value.extract::<pyo3_arrow::input::AnyRecordBatch>().map(Self)
    }
}

/// Extract only the existing public deployment's actual native owner.
pub(super) struct RuntimeInput(pub(super) Py<super::NativeRuntime>);
impl FromPyObject<'_, '_> for RuntimeInput {
    type Error = PyErr;
    const INPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_identifier!("pse._workflow", "Runtime");
    fn extract(value: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let facade = value.py().import("pse._workflow")?.getattr("Runtime")?;
        if !value.is_instance(&facade)? {
            return Err(pyo3::exceptions::PyTypeError::new_err("runtime must be an existing pse.Runtime"));
        }
        Ok(Self(value.getattr("_handle")?.extract::<Py<super::NativeRuntime>>()?))
    }
}
