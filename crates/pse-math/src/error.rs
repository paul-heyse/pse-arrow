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
    /// Optional symbolic coefficient projection is outside finite f64 representation.
    #[error("symbolic coefficient is not representable as a finite f64")]
    CoefficientRange,
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
    /// An observed canonical value lies outside an explicitly declared range.
    #[error("range {source_id}, target {target}: value {value} outside [{lower:?}, {upper:?}]")]
    OutsideRange {
        /// Authored bound or validity annotation.
        source_id: SemanticId,
        /// Actual specialized member, distinct from a reusable annotation.
        target: SemanticId,
        /// Observed value in the target's canonical unit.
        value: f64,
        /// Inclusive lower bound, when declared.
        lower: Option<f64>,
        /// Inclusive upper bound, when declared.
        upper: Option<f64>,
    },
    /// A configured finite admission bound was exceeded.
    #[error("math limit exceeded: {0}")]
    Limit(&'static str),
    /// A construction demand exceeds its explicit work allowance.
    #[error(
        "expression {source_id}: {resource} requires {required} operations, allowance {available}, Taylor width {components}"
    )]
    WorkLimit {
        /// Authored occurrence of the mathematical block.
        source_id: SemanticId,
        /// The admitted operation category.
        resource: &'static str,
        /// Conservative required operation count.
        required: usize,
        /// Remaining operation allowance.
        available: usize,
        /// Number of Taylor coefficients per scalar.
        components: usize,
    },
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
impl MathError {
    /// Conservative owned extent of the complete source/provider error chain.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(match self {
            Self::Instance { cause, .. } => cause.retained_bytes(),
            Self::Contract(s) | Self::Library(s) | Self::Evaluation { detail: s, .. } => {
                s.capacity()
            }
            Self::Quantity(e) => e.retained_bytes(),
            Self::Provider { cause, .. } => cause.retained_bytes(),
            Self::Domain { .. }
            | Self::OutsideRange { .. }
            | Self::Limit(_)
            | Self::WorkLimit { .. }
            | Self::Cancelled
            | Self::CoefficientRange => 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_failure_counts_owned_capacity_through_instance_and_provider() {
        let mut message = String::with_capacity(1 << 20);
        message.push('x');
        let capacity = message.capacity();
        let error = MathError::Instance {
            instance: SemanticId::NIL,
            cause: Box::new(MathError::Provider {
                source_id: SemanticId::NIL,
                provider: SemanticId::NIL,
                cause: pse_kernels::ProviderError::Terminal(message),
            }),
        };
        assert!(error.retained_bytes() >= capacity + 2 * size_of::<MathError>());
    }
}
pse_diagnostics::impl_diagnostic! {
    MathError,
    code(this) { match this {
        Self::Domain {..} | Self::OutsideRange {..} => Some(pse_diagnostics::DiagnosticCode::SolveEvaluationError),
        Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
        Self::Limit(_) | Self::WorkLimit {..} => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        Self::Quantity(_) | Self::Provider {..} | Self::Instance {..} => None,
        _ => Some(pse_diagnostics::DiagnosticCode::CompileMath),
    } },
    forward(this) { match this { Self::Quantity(e) => Some(e), Self::Instance {cause,..} => Some(cause.as_ref()), Self::Provider {cause,..} => Some(cause), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
