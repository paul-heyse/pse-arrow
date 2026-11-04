// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated structural adoption and bounded Buffa; no scientific authority is created here.
use super::{MathRuntimeError, RootInput, Hello, Request, Response};
use crate::math::{native_root_error::NativeRootWireError, native_root_generated::pb::pse::native_root::v2 as pb};
use pse_contract_proto::support::BoundaryProjectionError;
use buffa::Message as _;
use crate::math::native_root_generated as native_projection;

pub(super) trait Frame: Sized {
    type Wire: buffa::Message;
    fn into_wire(self) -> Result<Self::Wire, BoundaryProjectionError>;
    fn from_wire(wire: Self::Wire) -> Result<Self, BoundaryProjectionError>;
}
macro_rules! frame {
    ($native:ty, $wire:ty, $encode:ident, $decode:ident) => {
        impl Frame for $native {
            type Wire = $wire;
            fn into_wire(self) -> Result<Self::Wire, BoundaryProjectionError> { native_projection::$encode(self) }
            fn from_wire(wire: Self::Wire) -> Result<Self, BoundaryProjectionError> { native_projection::$decode(wire) }
        }
    };
}
frame!(RootInput, pb::NativeRootInput, encode_native_root_input, decode_native_root_input);
frame!(Hello, pb::NativeRootHello, encode_native_root_hello, decode_native_root_hello);
frame!(Request, pb::NativeRootRequest, encode_native_root_request, decode_native_root_request);
frame!(Response, pb::NativeRootResponse, encode_native_root_response, decode_native_root_response);

fn correspondence(error: BoundaryProjectionError) -> MathRuntimeError {
    NativeRootWireError::Projection(error.into()).into_runtime()
}
pub(super) fn encode<T: Frame>(value: T, limit: usize) -> Result<Vec<u8>, MathRuntimeError> {
    // The generated helper checks native rules before moving this owned data into the wire form.
    // Whole recipe/registry/numeric buffers are never cloned merely to frame them.
    let wire = value.into_wire().map_err(correspondence)?;
    let budget = u32::try_from(limit).map_err(|_| MathRuntimeError::Limit("native child outgoing frame extent"))?;
    let mut bytes = Vec::new();
    wire.try_encode_bounded(budget, &mut bytes).map_err(|error| NativeRootWireError::Encode(error).into_runtime())?;
    Ok(bytes)
}
pub(super) fn decode<T: Frame>(bytes: &[u8], limit: usize) -> Result<T, MathRuntimeError> {
    let elements = limit.checked_mul(4).ok_or(MathRuntimeError::Limit("native child decoder element allowance"))?;
    let wire = buffa::DecodeOptions::new().with_max_message_size(limit)
        .with_element_memory_limit(elements).with_recursion_limit(64).with_unknown_field_limit(256)
        .decode_from_slice::<T::Wire>(bytes).map_err(|error| NativeRootWireError::Decode(error).into_runtime())?;
    T::from_wire(wire).map_err(correspondence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::native_root_protocol::{RootOutcome, RESPONSE_LIMIT};
    use pse_ids::{ContentHash, SemanticId};
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection as _};

    #[test]
    fn private_root_generated_codec_keeps_proposal_bits_and_refuses_unknown_or_missing_meaning() {
        // A decoded candidate is raw data. The original parent still decides scientific use.
        let bits = [(-0.0_f64).to_bits(), 0x7ff8_0000_0000_0243, f64::INFINITY.to_bits()];
        let response = Response {
            attempt: SemanticId::NIL,
            execution: ContentHash::from_bytes([0; 32]),
            request: ContentHash::from_bytes([1; 32]),
            result: RootOutcome::Proposal(bits.into_iter().map(f64::from_bits).collect()),
        };
        let mut bytes = encode(response, RESPONSE_LIMIT).unwrap();
        let decoded: Response = decode(&bytes, RESPONSE_LIMIT).unwrap();
        let RootOutcome::Proposal(point) = decoded.result else { panic!("proposal became a refusal") };
        assert_eq!(point.iter().map(|value| value.to_bits()).collect::<Vec<_>>(), bits);

        // Unknown field 999 is retained by Buffa, then explicitly refused by source adoption.
        bytes.extend_from_slice(&[0xf8, 0x3e, 0x01]);
        let error = decode::<Response>(&bytes, RESPONSE_LIMIT).unwrap_err();
        assert_eq!(error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native).class, BoundaryClass::Internal);

        // Independent wire producer omits the required outcome; native construction cannot fill it.
        let mut bytes = Vec::new();
        pb::NativeRootResponse::default().try_encode_bounded(RESPONSE_LIMIT as u32, &mut bytes).unwrap();
        let error = decode::<Response>(&bytes, RESPONSE_LIMIT).unwrap_err();
        assert_eq!(error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native).class, BoundaryClass::Internal);
    }
}
