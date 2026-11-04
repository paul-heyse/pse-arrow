// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Validate actual generated outgoing rules before any response bytes are published.

pub(super) struct Validated<M>(pub(super) M);

pub(super) fn validate<M: protovalidate_buffa::Validate>(
    message: &M,
) -> Result<(), connectrpc::ConnectError> {
    message.validate().map_err(|failure| {
        // The pinned library emits bounded canonical buf.validate violation
        // details. Retain these together with the typed outgoing-owner failure.
        let details = failure.clone().into_connect_error().details;
        let mut error = super::study::projection_error(
            pse_contract_proto::support::BoundaryProjectionError::OutgoingWire(failure),
        );
        error.code = connectrpc::ErrorCode::Internal;
        error.details.extend(details);
        error
    })
}

impl<M: protovalidate_buffa::Validate + connectrpc::Encodable<M>> connectrpc::Encodable<M>
    for Validated<M>
{
    fn encode(
        &self,
        codec: connectrpc::CodecFormat,
    ) -> Result<bytes::Bytes, connectrpc::ConnectError> {
        validate(&self.0)?;
        self.0.encode(codec)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "literal generated outgoing local-rule failure is the test subject"
    )]
    use super::super::pb;
    use super::*;
    use connectrpc::Encodable as _;

    #[test]
    fn invalid_native_outgoing_envelope_refuses_before_binary_and_json_publication() {
        let invalid = Validated(pb::EngineeringContract {
            descriptor_sha256: vec![1; 31],
            ..Default::default()
        });
        for codec in [
            connectrpc::CodecFormat::Proto,
            connectrpc::CodecFormat::Json,
        ] {
            let error = invalid.encode(codec).unwrap_err();
            assert_eq!(error.code, connectrpc::ErrorCode::Internal);
            assert!(
                error
                    .details
                    .iter()
                    .any(|detail| detail.type_url == "buf.validate.Violations")
            );
            assert!(
                error
                    .details
                    .iter()
                    .any(|detail| detail.type_url == "pse.engineering.v1.EngineeringFailure")
            );
            let details = error
                .details
                .iter()
                .find_map(|detail| protovalidate_buffa::decode_violations(detail).unwrap())
                .unwrap();
            assert!(
                details
                    .violations
                    .iter()
                    .any(|violation| violation.rule_id.as_deref() == Some("bytes.min_len"))
            );
        }
        assert!(
            Validated(pb::EngineeringContract {
                descriptor_sha256: vec![1; 32],
                ..Default::default()
            })
            .encode(connectrpc::CodecFormat::Proto)
            .is_ok()
        );
    }
}
