// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Numerical preparation and evaluation failures (blueprint §23.2).

/// Typed numerical failure; unsupported derivatives never become finite differences.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum NumericsError {
    /// The caller's cancellation or allocation contract refused the operation.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Resource(#[from] pse_ids::CanonError),
    /// Invalid mathematical facts before numerical lowering.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Math(#[from] pse_mathir::MathIrError),
    /// A native expression or its preparation exceeded an execution resource limit.
    #[error("native numerical resource limit: {0}")]
    #[diagnostic(code(runtime::resource_limit))]
    NativeResource(#[source] datafusion::common::DataFusionError),
    /// No exact derivative implementation for the actual expression binding.
    #[error("unsupported numerical expression: {detail}")]
    #[diagnostic(code(capability::backend))]
    Unsupported {
        /// Expression or binding requiring an implementation.
        detail: String,
    },
    /// Input layout differs from the compiled problem.
    #[error("invalid numerical input: {detail}")]
    #[diagnostic(code(compile::math))]
    Input {
        /// Invalid shape, type, binding or nullability.
        detail: String,
    },
    /// Native expression compilation failed under the actual session state.
    #[error("native numerical preparation: {0}")]
    #[diagnostic(code(capability::backend))]
    Prepare(#[source] datafusion::common::DataFusionError),
    /// Native evaluation failed; the source retains domain/cancellation details.
    #[error("native numerical evaluation: {0}")]
    #[diagnostic(code(solve::evaluation_error))]
    Evaluate(#[source] datafusion::common::DataFusionError),
}

impl NumericsError {
    pub(crate) fn preparation(source: datafusion::common::DataFusionError) -> Self {
        if matches!(
            source.find_root(),
            datafusion::common::DataFusionError::ResourcesExhausted(_)
        ) {
            Self::NativeResource(source)
        } else {
            Self::Prepare(source)
        }
    }
    pub(crate) fn evaluation(source: datafusion::common::DataFusionError) -> Self {
        if matches!(
            source.find_root(),
            datafusion::common::DataFusionError::ResourcesExhausted(_)
        ) {
            Self::NativeResource(source)
        } else {
            Self::Evaluate(source)
        }
    }
}

pub(crate) fn unsupported(detail: impl Into<String>) -> NumericsError {
    NumericsError::Unsupported {
        detail: detail.into(),
    }
}
pub(crate) fn input(detail: impl Into<String>) -> NumericsError {
    NumericsError::Input {
        detail: detail.into(),
    }
}
