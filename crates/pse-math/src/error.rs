// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Attributable mathematical admission and evaluation failures.
use pse_ids::SemanticId;
/// A failure at an authored expression occurrence, not a library-local node number.
#[derive(Debug, thiserror::Error)]
pub enum MathError {
    /// Local source failure attributed to the actual bound process instance.
    #[error("instance {instance}: {cause}")]
    Instance {
        /// Selected instance, distinct from the reusable definition occurrence.
        instance: SemanticId,
        /// Original typed domain/provider/library cause and source occurrence.
        #[source]
        cause: Box<MathError>,
    },
    /// Invalid input, binding or execution profile.
    #[error("math contract: {0}")]
    Contract(String),
    /// Library construction or evaluation failed.
    #[error("math library: {0}")]
    Library(String),
    /// Runtime library failure attributed to its admitted source and derivative product.
    #[error("expression {source_id}, {order:?}: {detail}")]
    Evaluation {
        /// Original authored occurrence.
        source_id: SemanticId,
        /// Requested derivative order.
        order: pse_kernels::DerivativeOrder,
        /// Library diagnostic.
        detail: String,
    },
    /// A required authored domain condition failed.
    #[error("expression {source_id}: {requirement}")]
    Domain {
        /// Original expression identity.
        source_id: SemanticId,
        /// The unmet condition.
        requirement: &'static str,
    },
    /// A configured finite admission bound was exceeded.
    #[error("math limit exceeded: {0}")]
    Limit(&'static str),
    /// Requested cancellation.
    #[error("math evaluation cancelled")]
    Cancelled,
    /// Physical inference failed before normalization.
    #[error(transparent)]
    Quantity(#[from] pse_quantity::QuantityError),
    /// Provider failure, retaining its typed recoverability.
    #[error("provider {provider}: {cause}")]
    Provider {
        /// Authored call occurrence.
        source_id: SemanticId,
        /// Actual provider identity.
        provider: SemanticId,
        /// Original typed failure.
        #[source]
        cause: pse_kernels::ProviderError,
    },
}
pse_diagnostics::impl_diagnostic! {
    MathError,
    code(this) { match this {
        Self::Domain {..} => Some(pse_diagnostics::DiagnosticCode::SolveEvaluationError),
        Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
        Self::Limit(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        Self::Quantity(_) | Self::Provider {..} | Self::Instance {..} => None,
        _ => Some(pse_diagnostics::DiagnosticCode::CompileMath),
    } },
    forward(this) { match this { Self::Quantity(e) => Some(e), Self::Instance {cause,..} => Some(cause.as_ref()), Self::Provider {cause,..} => Some(cause), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
