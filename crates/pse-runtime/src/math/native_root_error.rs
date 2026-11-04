// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original typed private-channel failures and their retained custody.
use super::MathRuntimeError;
use crate::projection_diagnostics::ProjectionDiagnosticError;
use pse_contract_proto::support::RetainedExtentError;

/// A private admitted-process frame failed; the original codec/rule cause remains available.
#[derive(Debug, thiserror::Error)]
pub enum NativeRootWireError {
    /// Original fallible, byte-bounded protobuf encoding failure.
    #[error("native root encoding: {0}")]
    Encode(#[from] buffa::EncodeError),
    /// Original byte/element/depth bounded protobuf decoding failure.
    #[error("native root decoding: {0}")]
    Decode(#[from] buffa::DecodeError),
    /// Original structural/rule/adoption refusal, including complete selectors.
    #[error("native root correspondence: {0}")]
    Projection(#[from] ProjectionDiagnosticError),
}
impl NativeRootWireError {
    /// Actual retained inline/owned extent; never normalize an unknown allowance to zero.
    pub fn retained_bytes(&self) -> Result<usize, RetainedExtentError> {
        let heap = match self {
            Self::Projection(error) => error.retained_bytes()?.checked_sub(size_of::<ProjectionDiagnosticError>())
                .ok_or(RetainedExtentError)?,
            // The pinned Buffa 0.9.2 variants retain scalar values and static explanations only.
            Self::Encode(_) | Self::Decode(_) => 0,
        };
        size_of::<Self>().checked_add(heap).ok_or(RetainedExtentError)
    }
    /// Retain this original typed failure under the existing runtime error and custody contract.
    pub fn into_runtime(self) -> MathRuntimeError {
        let retained = match self.retained_bytes() {
            Ok(bytes) => bytes,
            // Existing runtime checked retained-extent admission refuses this sentinel while
            // retaining the original typed error; an overflow never issues a false zero allowance.
            Err(_) => usize::MAX,
        };
        pse_math::MathError::Typed { retained, cause: pse_model::diagnostic::DiagnosticCause::new(self) }.into()
    }
}
pse_diagnostics::impl_diagnostic! {
    NativeRootWireError,
    code(this) {
        let resource = match this {
            Self::Encode(buffa::EncodeError::MessageTooLarge | buffa::EncodeError::ExceedsBudget { .. }) => true,
            Self::Decode(buffa::DecodeError::MessageTooLarge | buffa::DecodeError::ElementMemoryLimitExceeded
                | buffa::DecodeError::UnknownFieldLimitExceeded | buffa::DecodeError::RecursionLimitExceeded) => true,
            _ => false,
        };
        Some(if resource { pse_diagnostics::DiagnosticCode::RuntimeResourceLimit } else { pse_diagnostics::DiagnosticCode::InternalInvariant })
    },
    forward(this) { match this { Self::Projection(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        match this {
            Self::Projection(error) => error.diagnostic_facts(),
            Self::Encode(buffa::EncodeError::ExceedsBudget { len, max_bytes }) => {
                let mut facts = pse_diagnostics::DiagnosticFacts::default();
                facts.observe("native_root_encoded_bytes", pse_diagnostics::DiagnosticObservation::Integer(i64::from(*len)));
                facts.observe("native_root_frame_allowance", pse_diagnostics::DiagnosticObservation::Integer(i64::from(*max_bytes)));
                facts
            }
            _ => pse_diagnostics::DiagnosticFacts::default(),
        }
    }
}
impl pse_model::diagnostic::DiagnosticProjection for NativeRootWireError {}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection as _};
    #[test]
    fn private_protocol_retains_original_codec_and_rule_causes_with_owning_classes() {
        let error = NativeRootWireError::Encode(buffa::EncodeError::ExceedsBudget { len: 20, max_bytes: 8 }).into_runtime();
        assert_eq!(error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native).class, BoundaryClass::ResourceLimit);
        let original = pse_contract_proto::support::BoundaryProjectionError::WireValidation(
            protovalidate_buffa::ValidationError { compile_error: Some("original compiler failure".into()), ..Default::default() });
        let error = NativeRootWireError::Projection(original.into()).into_runtime();
        assert_eq!(error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native).class, BoundaryClass::Internal);
        let mut cause: Option<&dyn std::error::Error> = Some(&error);
        let mut found = false;
        while let Some(current) = cause {
            found |= current.downcast_ref::<protovalidate_buffa::ValidationError>()
                .is_some_and(|error| error.compile_error.as_deref() == Some("original compiler failure"));
            cause = current.source();
        }
        assert!(found);
        assert!(error.retained_bytes().unwrap() > size_of::<MathRuntimeError>());
    }
}
