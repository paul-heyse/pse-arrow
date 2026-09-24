// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed immutable sequence conversion, including native stub introspection.
use pyo3::{
    inspect::{PyStaticConstant, PyStaticExpr},
    prelude::*,
    types::PyTuple,
};

pub(super) struct Tuple<T>(pub Vec<T>);
impl<'py, T: IntoPyObject<'py>> IntoPyObject<'py> for Tuple<T> {
    type Target = PyTuple;
    type Output = Bound<'py, PyTuple>;
    type Error = PyErr;
    const OUTPUT_TYPE: PyStaticExpr = pyo3::type_hint_subscript!(
        pyo3::type_hint_identifier!("builtins", "tuple"),
        T::OUTPUT_TYPE,
        PyStaticExpr::Constant {
            value: PyStaticConstant::Ellipsis
        }
    );
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        PyTuple::new(py, self.0)
    }
}
