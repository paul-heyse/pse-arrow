// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated structural adoption and bounded Buffa; no scientific authority is created here.
use super::{Hello, MathRuntimeError, Request, Response, RootInput};
use crate::math::native_root_generated as native_projection;
use crate::math::native_root_protocol::{RestoreGrant, RestorePreparation};
use crate::math::{
    native_root_error::NativeRootWireError, native_root_generated::pb::pse::native_root::v2 as pb,
};
use buffa::Message as _;
use pse_contract_proto::support::BoundaryProjectionError;
use pse_math::transfer::{ExecutionSnapshot, StageSnapshot};

const MESSAGE_DEPTH_LIMIT: u32 = 64;

pub(super) trait Frame: Sized {
    type Wire: buffa::Message;
    fn into_wire(self) -> Result<Self::Wire, BoundaryProjectionError>;
    fn from_wire(wire: Self::Wire) -> Result<Self, BoundaryProjectionError>;
    fn preflight(&self) -> Result<(), MathRuntimeError> {
        Ok(())
    }
}

impl Frame for ExecutionSnapshot {
    type Wire = pb::MathExecutionSnapshot;
    fn into_wire(self) -> Result<Self::Wire, BoundaryProjectionError> {
        native_projection::encode_math_execution_snapshot(self)
    }
    fn from_wire(wire: Self::Wire) -> Result<Self, BoundaryProjectionError> {
        native_projection::decode_math_execution_snapshot(wire)
    }
    fn preflight(&self) -> Result<(), MathRuntimeError> {
        snapshot_depth(self, 0)
    }
}

// Bound the native walk before generated validation, conversion and size traversal.
// Stage/Node recursion is distinct from the fixed leaf records. Reserve eight
// message levels for those records (layouts, tuples, lineage, claims, permissions),
// including optional/alternative carriers. The profile deliberately admits a
// subset of the native 512-stage envelope; its codec still has only 64 levels.
fn snapshot_depth(
    snapshot: &ExecutionSnapshot,
    enclosing_depth: u32,
) -> Result<(), MathRuntimeError> {
    fn leaf(depth: u32) -> Result<(), MathRuntimeError> {
        if depth + 8 > MESSAGE_DEPTH_LIMIT {
            Err(MathRuntimeError::Limit("native snapshot wire nesting"))
        } else {
            Ok(())
        }
    }
    fn node(plan: &pse_model::applicability::Node, depth: u32) -> Result<(), MathRuntimeError> {
        leaf(depth)?;
        for dependency in &plan.dependencies {
            node(dependency, depth + 1)?;
        }
        if let pse_model::applicability::Region::Union(alternatives) = &plan.region {
            for alternative in alternatives {
                node(alternative, depth + 3)?;
            }
        }
        Ok(())
    }
    fn stages(program: &[StageSnapshot], depth: u32) -> Result<(), MathRuntimeError> {
        leaf(depth)?;
        for stage in program {
            match stage {
                StageSnapshot::Block(_) | StageSnapshot::Require { .. } => {}
                StageSnapshot::Domain(domain) => stages(&domain.stages, depth + 3)?,
                StageSnapshot::Applicability(application) => {
                    node(&application.plan, depth + 3)?;
                    stages(&application.stages, depth + 3)?;
                }
                StageSnapshot::Branch {
                    then, otherwise, ..
                } => {
                    stages(then, depth + 2)?;
                    stages(otherwise, depth + 2)?;
                }
            }
        }
        Ok(())
    }
    stages(&snapshot.value, enclosing_depth + 1)?;
    stages(&snapshot.first, enclosing_depth + 1)
}
macro_rules! frame {
    ($native:ty, $wire:ty, $encode:ident, $decode:ident) => {
        impl Frame for $native {
            type Wire = $wire;
            fn into_wire(self) -> Result<Self::Wire, BoundaryProjectionError> {
                native_projection::$encode(self)
            }
            fn from_wire(wire: Self::Wire) -> Result<Self, BoundaryProjectionError> {
                native_projection::$decode(wire)
            }
        }
    };
}
impl Frame for RootInput {
    type Wire = pb::NativeRootInput;
    fn into_wire(self) -> Result<Self::Wire, BoundaryProjectionError> {
        native_projection::encode_native_root_input(self)
    }
    fn from_wire(wire: Self::Wire) -> Result<Self, BoundaryProjectionError> {
        native_projection::decode_native_root_input(wire)
    }
    fn preflight(&self) -> Result<(), MathRuntimeError> {
        // RootInput -> RootExecutionSnapshot -> ExecutionSnapshot.
        snapshot_depth(&self.execution.numerical, 2)
    }
}
frame!(
    RestorePreparation,
    pb::NativeRootRestorePreparation,
    encode_native_root_restore_preparation,
    decode_native_root_restore_preparation
);
frame!(
    RestoreGrant,
    pb::NativeRootRestoreGrant,
    encode_native_root_restore_grant,
    decode_native_root_restore_grant
);
frame!(
    Hello,
    pb::NativeRootHello,
    encode_native_root_hello,
    decode_native_root_hello
);
frame!(
    Request,
    pb::NativeRootRequest,
    encode_native_root_request,
    decode_native_root_request
);
frame!(
    Response,
    pb::NativeRootResponse,
    encode_native_root_response,
    decode_native_root_response
);

fn correspondence(error: BoundaryProjectionError) -> MathRuntimeError {
    NativeRootWireError::Projection(error.into()).into_runtime()
}
pub(super) fn encode<T: Frame>(value: T, limit: usize) -> Result<Vec<u8>, MathRuntimeError> {
    value.preflight()?;
    // The generated helper checks native rules before moving this owned data into the wire form.
    // Whole recipe/registry/numeric buffers are never cloned merely to frame them.
    let wire = value.into_wire().map_err(correspondence)?;
    let budget = u32::try_from(limit)
        .map_err(|_| MathRuntimeError::Limit("native child outgoing frame extent"))?;
    let mut bytes = Vec::new();
    wire.try_encode_bounded(budget, &mut bytes)
        .map_err(|error| NativeRootWireError::Encode(error).into_runtime())?;
    Ok(bytes)
}
pub(super) fn decode<T: Frame>(bytes: &[u8], limit: usize) -> Result<T, MathRuntimeError> {
    let elements = limit.checked_mul(4).ok_or(MathRuntimeError::Limit(
        "native child decoder element allowance",
    ))?;
    let wire = buffa::DecodeOptions::new()
        .with_max_message_size(limit)
        .with_element_memory_limit(elements)
        .with_recursion_limit(MESSAGE_DEPTH_LIMIT)
        .with_unknown_field_limit(256)
        .decode_from_slice::<T::Wire>(bytes)
        .map_err(|error| NativeRootWireError::Decode(error).into_runtime())?;
    T::from_wire(wire).map_err(correspondence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::native_root_protocol::{RESPONSE_LIMIT, RootOutcome};
    use pse_ids::{ContentHash, SemanticId};
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection as _};

    #[test]
    fn private_root_generated_codec_keeps_proposal_bits_and_refuses_unknown_or_missing_meaning() {
        // A decoded candidate is raw data. The original parent still decides scientific use.
        let bits = [
            (-0.0_f64).to_bits(),
            0x7ff8_0000_0000_0243,
            f64::INFINITY.to_bits(),
        ];
        let response = Response {
            attempt: SemanticId::NIL,
            execution: ContentHash::from_bytes([0; 32]),
            request: ContentHash::from_bytes([1; 32]),
            result: RootOutcome::Proposal(bits.into_iter().map(f64::from_bits).collect()),
        };
        let mut bytes = encode(response, RESPONSE_LIMIT).unwrap();
        let decoded: Response = decode(&bytes, RESPONSE_LIMIT).unwrap();
        let RootOutcome::Proposal(point) = decoded.result else {
            panic!("proposal became a refusal")
        };
        assert_eq!(
            point
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            bits
        );

        // Unknown field 999 is retained by Buffa, then explicitly refused by source adoption.
        bytes.extend_from_slice(&[0xf8, 0x3e, 0x01]);
        let error = decode::<Response>(&bytes, RESPONSE_LIMIT).unwrap_err();
        assert_eq!(
            error
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                .class,
            BoundaryClass::Internal
        );

        // Independent wire producer omits the required outcome; native construction cannot fill it.
        let mut bytes = Vec::new();
        pb::NativeRootResponse::default()
            .try_encode_bounded(RESPONSE_LIMIT as u32, &mut bytes)
            .unwrap();
        let error = decode::<Response>(&bytes, RESPONSE_LIMIT).unwrap_err();
        assert_eq!(
            error
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                .class,
            BoundaryClass::Internal
        );
    }
}
