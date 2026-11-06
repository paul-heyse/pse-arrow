// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Validate actual generated outgoing rules before any response bytes are published.

pub(super) struct Validated<M>(pub(super) M);

pub(super) fn validate<M: protovalidate_buffa::Validate>(
    message: &M,
) -> Result<(), connectrpc::ConnectError> {
    message.validate().map_err(|failure| {
        let mut error = super::study::projection_error(
            pse_contract_proto::support::BoundaryProjectionError::OutgoingWire(failure),
        );
        error.code = connectrpc::ErrorCode::Internal;
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
            assert_eq!(
                error
                    .details
                    .iter()
                    .filter(|detail| detail.type_url == "buf.validate.Violations")
                    .count(),
                1
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

    #[test]
    fn generated_service_status_preserves_false_full_counters_and_ieee_payloads() {
        use pb::engineering_progress_value::Kind;
        use pse_model::engineering::{
            EngineeringProgressValue as Value, EngineeringRunProgress, EngineeringRunStatus,
        };
        let nan = 0x7ff8_0000_0000_4321;
        let native = EngineeringRunStatus {
            operation: "original-operation".into(),
            run: None,
            settled: false,
            usable: Some(false),
            cancellation_requested: true,
            failure: None,
            canonical_run: None,
            canonical_attempt: None,
            progress: Some(EngineeringRunProgress {
                generation: u64::MAX,
                highest_issued_sequence: u64::MAX,
                dropped: 0,
                current_is_latest: false,
                phase: None,
                elapsed_seconds_bits: Some((-0.0_f64).to_bits()),
                sequence: Some(0),
                values: [
                    ("integer".into(), Value::Integer(9_007_199_254_740_993)),
                    ("nan".into(), Value::RealBits(nan)),
                    (
                        "negative_zero".into(),
                        Value::RealBits((-0.0_f64).to_bits()),
                    ),
                    ("false".into(), Value::Boolean(false)),
                ]
                .into_iter()
                .collect(),
                observations_available: Some(false),
                observation_updates_dropped: Some(0),
            }),
        };
        let wire = crate::engineering_projection::encode_engineering_run_status(native).unwrap();
        assert_eq!(wire.usable, Some(false));
        assert_eq!(wire.run, None);
        let progress = wire.progress.as_option().unwrap();
        assert_eq!(progress.generation, u64::MAX);
        assert_eq!(progress.highest_issued_sequence, u64::MAX);
        assert_eq!(progress.elapsed_seconds_bits, Some(1_u64 << 63));
        assert_eq!(progress.sequence, Some(0));
        assert_eq!(progress.observations_available, Some(false));
        assert_eq!(progress.observation_updates_dropped, Some(0));
        assert!(
            matches!(&progress.values["integer"].kind, Some(Kind::Integer(value)) if value.value == 9_007_199_254_740_993)
        );
        assert!(
            matches!(&progress.values["nan"].kind, Some(Kind::RealBits(value)) if value.value == nan)
        );
        assert!(
            matches!(&progress.values["negative_zero"].kind, Some(Kind::RealBits(value)) if value.value == 1_u64 << 63)
        );
        assert!(
            matches!(&progress.values["false"].kind, Some(Kind::Boolean(value)) if !value.value)
        );
    }
}
