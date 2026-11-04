// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-generated structural correspondence; native preparation retains scientific admission.
use super::{WorkerError, generated::pb::pse::scientific_worker::v1 as pb, protocol as p};
use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;
use pse_kernels::ProviderError;

#[path = "../generated/scientific-worker/native-adapters.rs"]
#[rustfmt::skip]
mod native;

fn contract(error: BoundaryProjectionError) -> WorkerError {
    ProviderError::Contract(format!("scientific worker projection: {error}")).into()
}
fn response_failure(error: BoundaryProjectionError) -> WorkerError {
    // This conversion runs after RPC dispatch. Failure to establish the complete
    // native outcome cannot become a definite scientific refusal.
    WorkerError::UnknownCompletion(format!("scientific worker response projection: {error}"))
}
macro_rules! crossing {
    ($native:ty, $wire:ty, $encode:ident, $decode:ident, $incoming:ident) => {
        impl TryFrom<$native> for $wire {
            type Error = WorkerError;
            fn try_from(value: $native) -> Result<Self, Self::Error> {
                native::$encode(value).map_err(contract)
            }
        }
        impl TryFrom<$wire> for $native {
            type Error = WorkerError;
            fn try_from(value: $wire) -> Result<Self, Self::Error> {
                native::$decode(value).map_err($incoming)
            }
        }
    };
}
crossing!(
    p::Request,
    pb::Request,
    encode_request,
    decode_request,
    contract
);
crossing!(
    p::Response,
    pb::Response,
    encode_response,
    decode_response,
    response_failure
);

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;

    #[test]
    fn shutdown_has_an_explicit_wire_operation_and_acknowledgement() {
        let request = native::encode_request(p::Request::Shutdown)
            .unwrap()
            .encode_to_vec();
        let response = native::encode_response(p::Response::Shutdown)
            .unwrap()
            .encode_to_vec();
        assert!(matches!(
            native::decode_request(decode::<pb::Request>(&request)).unwrap(),
            p::Request::Shutdown
        ));
        assert!(matches!(
            native::decode_response(decode::<pb::Response>(&response)).unwrap(),
            p::Response::Shutdown
        ));
    }
    #[test]
    fn batch_binary_preserves_raw_bits_before_independent_point_admission() {
        let exact = vec![0u64, (-0.0f64).to_bits(), 0x7ff8_0000_0000_1234];
        let request = p::Request::EvaluateBatch(p::EvaluateBatch {
            handle: handle(),
            invocation: 7,
            points: vec![p::BatchPoint {
                id: pse_ids::SemanticId::from_bytes([4; 16]),
                inputs: exact.clone(),
                directions: vec![],
                cotangents: vec![],
                weights: vec![],
            }],
        });
        let generated: pb::Request =
            decode(&native::encode_request(request).unwrap().encode_to_vec());
        generated.validate().unwrap();
        let p::Request::EvaluateBatch(actual) = native::decode_request(generated).unwrap() else {
            panic!("batch");
        };
        assert_eq!(actual.points[0].inputs, exact);
    }
    use protovalidate_buffa::Validate as _;

    fn handle() -> p::Handle {
        p::Handle {
            attempt: pse_ids::SemanticId::from_bytes([1; 16]),
            session: pse_ids::SemanticId::from_bytes([2; 16]),
            generation: pse_ids::SemanticId::from_bytes([3; 16]),
        }
    }
    fn decode<M: buffa::Message>(bytes: &[u8]) -> M {
        buffa::DecodeOptions::new()
            .with_max_message_size(1 << 20)
            .with_element_memory_limit(4 << 20)
            .with_recursion_limit(32)
            .with_unknown_field_limit(64)
            .decode_from_slice(bytes)
            .unwrap()
    }

    #[test]
    fn binary_fixed64_preserves_signed_zero_and_nan_payload_before_scientific_admission() {
        let values = [0.0, -0.0, f64::from_bits(0x7ff8_0000_0000_1234)];
        let request = p::Request::Evaluate(p::Evaluate {
            handle: handle(),
            invocation: 1,
            inputs: values.to_vec(),
            directions: vec![],
            cotangents: vec![],
            weights: vec![],
        });
        let encoded = native::encode_request(request).unwrap().encode_to_vec();
        let generated: pb::Request = decode(&encoded);
        generated.validate().unwrap();
        let p::Request::Evaluate(actual) = native::decode_request(generated).unwrap() else {
            panic!("evaluate")
        };
        assert_eq!(
            actual
                .inputs
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            values.map(f64::to_bits)
        );
    }

    fn preparation() -> p::Prepare {
        let registry = pse_quantity::generated::standard_registry().unwrap();
        let descriptor = pse_backend_native::coolprop::descriptor(
            &registry,
            pse_backend_native::coolprop::WaterRegime::Vapor,
        )
        .unwrap();
        p::Prepare {
            version: 2,
            handle: handle(),
            spec: pse_kernels::portable::ProviderRecord::capture(&descriptor),
            configuration: pse_ids::ContentHash::from_bytes([0x11; 32]),
            registry: registry.to_builder(),
            physical: super::super::handler::physical_identity(&registry).unwrap(),
            regime: "vapor".into(),
            outputs: vec![0],
            order: 0,
            demand: p::Demand::CompleteJets,
            remaining_ns: None,
            result_limit: 4096,
        }
    }

    #[test]
    fn generated_preparation_validator_preserves_absent_deadline_and_refuses_present_zero() {
        let request = preparation();
        for deadline in [None, Some(1)] {
            let mut request = request.clone();
            request.remaining_ns = deadline;
            let generated = decode::<pb::Prepare>(
                &encode_root::<_, pb::Prepare>(request, "Prepare")
                    .unwrap()
                    .encode_to_vec(),
            );
            generated.validate().unwrap();
            assert_eq!(
                decode_root::<p::Prepare, _>(generated, "Prepare")
                    .unwrap()
                    .remaining_ns,
                deadline
            );
        }
        let mut invalid = encode_root::<_, pb::Prepare>(request, "Prepare").unwrap();
        invalid.remaining_ns = Some(0);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn generated_registry_crossing_keeps_declarations_unadmitted_and_refuses_unknown_meaning() {
        let mut request = preparation();
        let registry = request.registry.clone().build().unwrap();
        request
            .registry
            .unit(registry.units().next().unwrap().clone());
        let session = request.handle.session;
        let wire = native::encode_request(p::Request::Prepare(request)).unwrap();
        let p::Request::Prepare(request) =
            native::decode_request(decode::<pb::Request>(&wire.encode_to_vec())).unwrap()
        else {
            panic!("Prepare")
        };
        assert!(matches!(
            super::super::handler::prepare(
                request,
                session,
                pse_diagnostics::Cancellation::default()
            ),
            Err(WorkerError::Provider(ProviderError::Contract(_)))
        ));

        let mut wire = encode_root::<_, pb::Prepare>(preparation(), "Prepare").unwrap();
        let mut encoded = wire.registry.into_option().unwrap().encode_to_vec();
        encoded.extend([0xa0, 0x06, 0x01]);
        wire.registry = decode::<pb::PhysicalRegistryDeclarations>(&encoded).into();
        assert!(decode_root::<p::Prepare, _>(wire, "Prepare").is_err());

        let mut wire = encode_root::<_, pb::Prepare>(preparation(), "Prepare").unwrap();
        wire.registry = Default::default();
        assert!(decode_root::<p::Prepare, _>(wire, "Prepare").is_err());
        let mut wire = encode_root::<_, pb::Prepare>(preparation(), "Prepare").unwrap();
        wire.version = 1;
        assert!(decode_root::<p::Prepare, _>(wire, "Prepare").is_err());
    }

    #[test]
    fn malformed_responses_are_unknown_completion_while_native_contract_refusal_stays_definite() {
        for wire in [
            pb::Response::default(),
            pb::Response {
                kind: Some(pb::response::Kind::Values(Default::default())),
                ..Default::default()
            },
        ] {
            assert!(matches!(
                p::Response::try_from(wire),
                Err(WorkerError::UnknownCompletion(_))
            ));
        }
        let refusal =
            p::Response::Failure(p::Failure::Contract("native unsupported demand".into()));
        let mut bytes = pb::Response::try_from(refusal).unwrap().encode_to_vec();
        let p::Response::Failure(refusal) =
            p::Response::try_from(decode::<pb::Response>(&bytes)).unwrap()
        else {
            panic!("native refusal")
        };
        assert!(matches!(
            super::super::handler::refusal(refusal),
            WorkerError::Provider(ProviderError::Contract(_))
        ));
        bytes.extend([0xa0, 0x06, 0x01]);
        assert!(matches!(
            p::Response::try_from(decode::<pb::Response>(&bytes)),
            Err(WorkerError::UnknownCompletion(_))
        ));
    }

    #[test]
    fn absent_unknown_truncated_and_oversized_wire_refuse_without_a_native_owner() {
        assert!(pb::Request::default().validate().is_err());
        assert!(native::decode_request(pb::Request::default()).is_err());
        let mut encoded = native::encode_request(p::Request::Close(handle()))
            .unwrap()
            .encode_to_vec();
        assert!(pb::Request::decode_from_slice(&encoded[..encoded.len() - 1]).is_err());
        assert!(
            buffa::DecodeOptions::new()
                .with_max_message_size(encoded.len() - 1)
                .decode_from_slice::<pb::Request>(&encoded)
                .is_err()
        );
        // Unknown required meaning is rejected even when protobuf preserves it.
        encoded.extend([0xa0, 0x06, 0x01]);
        assert!(native::decode_request(decode::<pb::Request>(&encoded)).is_err());
    }

    #[test]
    fn all_demand_alternatives_roundtrip_without_claiming_water_support() {
        for expected in [
            p::Demand::CompleteJets,
            p::Demand::SelectedColumns(vec![1, 0]),
            p::Demand::JacobianAction(3),
            p::Demand::TransposeAction(2),
            p::Demand::WeightedSecondAction(4),
        ] {
            let encoded = encode_root::<_, pb::Demand>(expected.clone(), "Demand")
                .unwrap()
                .encode_to_vec();
            let generated: pb::Demand = decode(&encoded);
            generated.validate().unwrap();
            assert_eq!(
                decode_root::<p::Demand, _>(generated, "Demand").unwrap(),
                expected
            );
        }
    }

    #[test]
    fn typed_envelope_refusal_preserves_exact_coordinates_and_resource_attribution() {
        let original = p::Failure::OutsideEnvelope {
            axis: "temperature".into(),
            value: -0.0,
            lower: 273.16,
            upper: 1200.0,
        };
        let encoded = encode_root::<_, pb::Failure>(original, "Failure")
            .unwrap()
            .encode_to_vec();
        let generated: pb::Failure = decode(&encoded);
        generated.validate().unwrap();
        let WorkerError::Provider(ProviderError::OutsideEnvelope {
            value,
            lower,
            upper,
            ..
        }) = super::super::handler::refusal(
            decode_root::<p::Failure, _>(generated, "Failure").unwrap(),
        )
        else {
            panic!("typed envelope")
        };
        assert_eq!(value.to_bits(), (-0.0f64).to_bits());
        assert_eq!(lower, 273.16);
        assert_eq!(upper, 1200.0);
        use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection as _};
        let failure =
            super::super::handler::refusal(p::Failure::Resource("original extent".into()))
                .into_provider();
        assert_eq!(
            failure
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
                .class,
            BoundaryClass::ResourceLimit
        );
    }
}
