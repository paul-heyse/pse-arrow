// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Interpret one finite native RPC failure family without inferring semantic authority.
use super::{Error, Failure, FailurePhase, FailureState};
use crate::envelope::{RedactionPolicy, ValidationCorrection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pse_contract::projection_enum! {
/// Disposition of native typed details, independent of transport status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RpcDisposition {
    projection: { name: "SemanticRpcDisposition", rust_path: crate::semantic::RpcDisposition, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Exactly one supported native failure was decoded and converted.
    Native => 1;
    /// The error carried no details.
    Absent => 2;
    /// Details exist, but none belongs to the native failure family.
    Unrecognized => 3;
    /// A recognized detail could not be decoded.
    Malformed => 4;
    /// Decoded native fields could not be interpreted by this client.
    ConversionRefused => 5;
    /// Multiple native failure details cannot select one authoritative failure.
    Ambiguous => 6;
    /// Details exceed this client's bounded interpretation allowance.
    Oversized => 7;
}
}
pse_contract::projection_struct! {
/// Passive public observation; complete original RPC evidence stays in the local cause.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RpcObservation {
    projection: { name: "SemanticRpcObservation", rust_path: crate::semantic::RpcObservation, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Original canonical gRPC status number, never a native refusal class.
    pub grpc_code: u32 => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// ConnectRPC's effective HTTP error mapping, including a local override when present.
    pub http_status: u32 => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Receiver's typed detail interpretation.
    pub disposition: RpcDisposition => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Actual total detail count, including unknowns.
    pub detail_count: u64 => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Actual number of recognized native failure details.
    pub native_detail_count: u64 => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original native state, available only from one valid authored failure.
    pub native_state: Option<FailureState> => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original native boundary, available only from one valid authored failure.
    pub native_phase: Option<FailurePhase> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Bounded canonical validation correction, independent of native refusal interpretation.
    pub validation: Option<Box<ValidationCorrection>> => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original authored diagnostic identity, including codes unknown to this client.
    pub native_diagnostic_code: Option<String> => { tag: 9, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}

/// Typed interpretation retains both conversion defects and the original ConnectRPC error.
#[derive(Debug, thiserror::Error)]
#[error("{detail}")]
pub struct RpcFailure {
    pub(super) detail: Detail,
    /// Full original status, message, details and local Rust cause; never public serialization.
    #[source]
    pub original: connectrpc::ConnectError,
}
#[derive(Debug, thiserror::Error)]
pub(super) enum Detail {
    #[error("{0}")]
    Native(Box<Failure>),
    #[error("semantic RPC failure has no typed details")]
    Absent,
    #[error("semantic RPC failure has no recognized native failure detail")]
    Unrecognized,
    #[error("malformed native semantic RPC failure detail: {0}")]
    Malformed(#[source] DecodeFailure),
    #[error("native semantic RPC failure conversion refused: {0}")]
    ConversionRefused(#[source] Box<Error>),
    #[error("semantic RPC failure has multiple native failure details")]
    Ambiguous,
    #[error("semantic RPC failure details exceed the interpretation allowance")]
    Oversized,
}
#[derive(Debug, thiserror::Error)]
pub(super) enum DecodeFailure {
    #[error("missing protobuf value")]
    MissingValue,
    #[error("invalid base64")]
    Base64(#[source] base64::DecodeError),
    #[error("invalid protobuf")]
    Protobuf(#[source] buffa::DecodeError),
}
impl RpcFailure {
    /// Borrow authored native meaning only after successful unambiguous interpretation.
    pub fn native_failure(&self) -> Option<&Failure> {
        match &self.detail {
            Detail::Native(failure) => Some(failure),
            _ => None,
        }
    }
    pub(crate) fn phase(&self) -> FailurePhase {
        match &self.detail {
            Detail::Native(failure) => failure.phase,
            Detail::Malformed(_) | Detail::ConversionRefused(_) | Detail::Ambiguous => {
                FailurePhase::Schema
            }
            _ => FailurePhase::Transport,
        }
    }
    /// Original decode/conversion defect, separately from the transport's local source chain.
    pub fn interpretation_error(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.detail {
            Detail::Malformed(error) => Some(error),
            Detail::ConversionRefused(error) => Some(error.as_ref()),
            _ => None,
        }
    }
    /// The independently retained native conversion refusal, when present.
    pub fn conversion_error(&self) -> Option<&Error> {
        match &self.detail {
            Detail::ConversionRefused(error) => Some(error),
            _ => None,
        }
    }
    pub(super) fn failure(&self, policy: &RedactionPolicy) -> Failure {
        if let Detail::Native(failure) = &self.detail {
            return failure.as_ref().clone().redacted(policy);
        }
        Failure {
            diagnostic_code: None,
            context: None,
            state: FailureState::Failed,
            phase: self.phase(),
            message: policy.redact(&self.to_string()),
            backend_code: None,
            backend_data: None,
            observed_cancellation: None,
            validation: self
                .validation()
                .map(|value| Box::new(value.redacted(policy))),
        }
    }
    fn validation(&self) -> Option<Box<ValidationCorrection>> {
        if let Detail::Native(failure) = &self.detail {
            return failure.validation.clone();
        }
        // The same allowance governs canonical detail expansion, including unknown details.
        if !super::rpc::details_within_allowance(&self.original.details) {
            return None;
        }
        ValidationCorrection::rpc(&self.original.details, None).map(Box::new)
    }
    /// Project selected facts without copying opaque original details or exposing local sources.
    pub fn observation(&self) -> RpcObservation {
        self.observation_with_policy(&RedactionPolicy::from_environment())
    }
    /// Use the performing caller's disclosure policy, which can differ from process variables.
    pub fn observation_with_policy(&self, policy: &RedactionPolicy) -> RpcObservation {
        let native = match &self.detail {
            Detail::Native(failure) => Some(failure),
            _ => None,
        };
        RpcObservation {
            grpc_code: self.original.code.grpc_code(),
            http_status: u32::from(self.original.http_status().as_u16()),
            disposition: match &self.detail {
                Detail::Native(_) => RpcDisposition::Native,
                Detail::Absent => RpcDisposition::Absent,
                Detail::Unrecognized => RpcDisposition::Unrecognized,
                Detail::Malformed(_) => RpcDisposition::Malformed,
                Detail::ConversionRefused(_) => RpcDisposition::ConversionRefused,
                Detail::Ambiguous => RpcDisposition::Ambiguous,
                Detail::Oversized => RpcDisposition::Oversized,
            },
            detail_count: self.original.details.len() as u64,
            native_detail_count: self
                .original
                .details
                .iter()
                .filter(|detail| super::rpc::is_native_detail(detail))
                .count() as u64,
            native_state: native.map(|failure| failure.state),
            native_phase: native.map(|failure| failure.phase),
            validation: self
                .validation()
                .map(|value| Box::new(value.redacted(policy))),
            native_diagnostic_code: native.and_then(|failure| {
                failure
                    .diagnostic_code
                    .as_ref()
                    .map(|code| policy.redact(code))
            }),
        }
    }
}
