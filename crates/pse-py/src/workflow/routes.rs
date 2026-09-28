// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed routes and eligibility rows (F30, ADR-0113 §3): registry names and typed detail
//! values, never Rust `Debug` output or prose.
use pse_backend_native::routing::{Eligibility, Ineligible, Route};
use pse_kernels::DerivativeOrder;
use pyo3::prelude::*;

/// A selected execution route: a registry backend, or direct evaluation of a model
/// without free coordinates.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeRoute {
    inner: Route,
}
impl From<Route> for NativeRoute {
    fn from(inner: Route) -> Self {
        Self { inner }
    }
}
#[pymethods]
impl NativeRoute {
    /// Registry name of the native backend; `None` for direct evaluation.
    #[getter]
    fn backend(&self) -> Option<&'static str> {
        match self.inner {
            Route::Native(backend) => Some(backend.as_str()),
            Route::Constant => None,
        }
    }
    /// The model is evaluated directly, without a native attempt.
    #[getter]
    fn constant(&self) -> bool {
        self.inner == Route::Constant
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
    fn __repr__(&self) -> String {
        self.backend().map_or_else(
            || "NativeRoute(constant)".into(),
            |b| format!("NativeRoute({b})"),
        )
    }
}

/// One adapter's contextual assessment: eligible when it has no reason.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeEligibility {
    inner: Eligibility,
}
impl From<&Eligibility> for NativeEligibility {
    fn from(inner: &Eligibility) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}
#[pymethods]
impl NativeEligibility {
    /// Registry name of the assessed backend.
    #[getter]
    fn backend(&self) -> &'static str {
        self.inner.backend.as_str()
    }
    /// Whether the adapter can represent the request.
    #[getter]
    fn eligible(&self) -> bool {
        self.inner.reasons.is_empty()
    }
    /// Every applicable typed reason.
    #[getter]
    fn reasons(&self) -> Vec<NativeIneligible> {
        self.inner
            .reasons
            .iter()
            .cloned()
            .map(|inner| NativeIneligible { inner })
            .collect()
    }
}

/// Why one adapter cannot represent a request: a registry reason code with its typed
/// detail values.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeIneligible {
    inner: Ineligible,
}
#[pymethods]
impl NativeIneligible {
    /// Registry reason code.
    #[getter]
    fn code(&self) -> &'static str {
        self.inner.code().as_str()
    }
    /// Registry problem classes the facts and intent establish (`class`).
    #[getter]
    fn problem_classes(&self) -> Vec<&'static str> {
        match &self.inner {
            Ineligible::Class { problem } => problem.iter().map(|c| c.as_str()).collect(),
            _ => Vec::new(),
        }
    }
    /// Required prepared derivative order (`derivatives`).
    #[getter]
    fn derivative_order(&self) -> Option<u8> {
        match self.inner {
            Ineligible::Derivatives { required } => Some(match required {
                DerivativeOrder::Value => 0,
                DerivativeOrder::First => 1,
                DerivativeOrder::Second => 2,
            }),
            _ => None,
        }
    }
    /// Whether one-sided bounds stay representable as shifted sign constraints (`bounds`).
    #[getter]
    fn sign_bounds(&self) -> Option<bool> {
        match self.inner {
            Ineligible::Bounds { signs } => Some(signs),
            _ => None,
        }
    }
    /// Registry constraint forms outside the adapter's record (`native_forms`).
    #[getter]
    fn forms(&self) -> Vec<&'static str> {
        match &self.inner {
            Ineligible::NativeForms { missing } => missing.iter().map(|f| f.as_str()).collect(),
            _ => Vec::new(),
        }
    }
}
