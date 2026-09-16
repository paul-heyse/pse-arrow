// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native backend diagnostics retain numerical and resource causes.
/// Failures before a native run or during its actual numerical callbacks.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum NativeError {
    /// Invalid input layout, bound, scale or solver option.
    #[error("invalid native problem: {0}")]
    #[diagnostic(code(compile::math))]
    Input(String),
    /// A required backend capability is absent.
    #[error("native backend capability: {0}")]
    #[diagnostic(code(capability::backend))]
    Capability(String),
    /// Native expression preparation or evaluation.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Numerics(#[from] pse_numerics::NumericsError),
    /// Accounted allocation or cooperative cancellation.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Resource(#[from] pse_ids::CanonError),
    /// A callback panic was contained before crossing the C ABI.
    #[error("native numerical callback panicked")]
    #[diagnostic(code(solve::evaluation_error))]
    CallbackPanic,
    /// Actual C API rejected construction/options before solving.
    #[error("Ipopt rejected {0}")]
    #[diagnostic(code(capability::backend))]
    Ipopt(String),
}

pub(crate) fn invalid(detail: impl Into<String>) -> NativeError {
    NativeError::Input(detail.into())
}
