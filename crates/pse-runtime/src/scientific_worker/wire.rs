// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-generated structural correspondence; native preparation retains scientific admission.
use super::{
    WorkerError, failure_budget::failure_within_limits,
    generated::pb::pse::scientific_worker::v2 as pb,
};
use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;
use pse_kernels::{ProviderError, worker_protocol as p};

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
impl TryFrom<p::Response> for pb::Response {
    type Error = WorkerError;
    fn try_from(value: p::Response) -> Result<Self, Self::Error> {
        validate_failure_response(&value)?;
        native::encode_response(value).map_err(contract)
    }
}
impl TryFrom<pb::Response> for p::Response {
    type Error = WorkerError;
    fn try_from(value: pb::Response) -> Result<Self, Self::Error> {
        let value = native::decode_response(value).map_err(response_failure)?;
        validate_failure_response(&value)?;
        Ok(value)
    }
}

fn validate_failure_response(value: &p::Response) -> Result<(), WorkerError> {
    let valid = match value {
        p::Response::Failure(failure) => failure_within_limits(failure),
        p::Response::Batch(batch) => batch.points.iter().all(|point| match &point.result {
            p::BatchResult::Refused(failure) => failure_within_limits(failure),
            p::BatchResult::Product(_) => true,
        }),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(WorkerError::UnknownCompletion(
            "scientific worker response failure exceeds its declared restoration allowance".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use pse_kernels::{DerivativeOrder, ProviderDemand, ProviderRequest};

    fn round_trip_failure(error: WorkerError) -> WorkerError {
        let response = p::Response::Failure(super::super::handler::failure(error));
        let encoded = pb::Response::try_from(response).unwrap().encode_to_vec();
        let restored = p::Response::try_from(decode::<pb::Response>(&encoded)).unwrap();
        let p::Response::Failure(failure) = restored else {
            panic!("original failure required");
        };
        super::super::handler::refusal(failure)
    }

    #[test]
    fn generated_binary_preserves_keyed_provider_settlement_and_panic_leaves() {
        let first = pse_kernels::ProviderKey(pse_ids::ContentHash::from_bytes([3; 32]));
        let second = pse_kernels::ProviderKey(pse_ids::ContentHash::from_bytes([4; 32]));
        let original = ProviderError::Settlement {
            operation: Box::new(ProviderError::Trial("original trial".into())),
            cleanup: Box::new(ProviderError::Retirement {
                failures: vec![
                    pse_kernels::ProviderRetirementFailure {
                        key: first,
                        cause: Box::new(ProviderError::RetirementPanic("original panic".into())),
                    },
                    pse_kernels::ProviderRetirementFailure {
                        key: second,
                        cause: Box::new(ProviderError::Contract("independent contract".into())),
                    },
                ],
            }),
        };
        let WorkerError::Provider(ProviderError::Settlement { operation, cleanup }) =
            round_trip_failure(original.into())
        else {
            panic!("typed settlement required");
        };
        assert!(
            matches!(operation.as_ref(), ProviderError::Trial(detail) if detail == "original trial")
        );
        let ProviderError::Retirement { failures } = *cleanup else {
            panic!("keyed retirements required");
        };
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].key, first);
        assert_eq!(failures[1].key, second);
        assert!(
            matches!(failures[0].cause.as_ref(), ProviderError::RetirementPanic(detail) if detail == "original panic")
        );
        assert!(
            matches!(failures[1].cause.as_ref(), ProviderError::Contract(detail) if detail == "independent contract")
        );
    }

    #[test]
    fn generated_worker_aggregate_is_terminal_and_opaque_diagnostic_stays_an_observation() {
        use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection as _};
        let opaque = ProviderError::Trial("opaque observed trial".into())
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
        let opaque_original = ProviderError::Trial("opaque observed trial".into());
        let opaque_original = ProviderError::Nested {
            retained: opaque_original.retained_bytes(),
            cause: pse_model::diagnostic::DiagnosticCause::new(opaque_original),
        };
        let original = WorkerError::Retirement {
            causes: vec![
                ProviderError::Trial("original known trial".into()).into(),
                opaque_original.into(),
            ],
        };
        let restored = round_trip_failure(original);
        let diagnostic = restored.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
        assert_eq!(diagnostic.class, BoundaryClass::Infrastructure);
        assert_eq!(diagnostic.causes.len(), 2);
        assert_eq!(diagnostic.causes[0].class, BoundaryClass::TrialRejected);
        let WorkerError::Retirement { causes } = restored else {
            panic!("worker aggregate required");
        };
        let WorkerError::RemoteDiagnostic(observed) = &causes[1] else {
            panic!("opaque observation must not become a native error");
        };
        assert_eq!(observed.code, opaque.code);
        assert_eq!(observed.class, opaque.class);
        assert!(
            !WorkerError::RemoteDiagnostic(observed.clone())
                .into_provider()
                .recoverable()
        );
        let restored = WorkerError::Retirement { causes };
        assert!(!restored.into_provider().recoverable());
    }

    #[test]
    fn complete_generated_failure_restoration_refuses_depth_or_node_excess() {
        let mut deep = p::Failure::Cancelled;
        for _ in 0..=super::super::failure_budget::FAILURE_DEPTH {
            deep = p::Failure::ProviderSettlement {
                operation: Box::new(deep),
                cleanup: Box::new(p::Failure::Cancelled),
            };
        }
        assert!(!failure_within_limits(&deep));
        assert!(matches!(
            super::super::handler::refusal(deep),
            WorkerError::UnknownCompletion(_)
        ));
        let wide = p::Failure::Retirement {
            causes: (0..128)
                .map(|_| p::Failure::Retirement {
                    causes: (0..8).map(|_| p::Failure::Cancelled).collect(),
                })
                .collect(),
        };
        assert!(!failure_within_limits(&wide));
        // Generated schema validation permits each local list; the source's
        // whole-tree allowance independently rejects complete restoration.
        let encoded = native::encode_response(p::Response::Failure(wide))
            .unwrap()
            .encode_to_vec();
        assert!(matches!(
            p::Response::try_from(decode::<pb::Response>(&encoded)),
            Err(WorkerError::UnknownCompletion(_))
        ));
    }

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

    fn handle() -> p::SessionHandle {
        p::SessionHandle {
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
            version: p::PREPARE_VERSION,
            handle: handle(),
            spec: pse_kernels::portable::ProviderRecord::capture(&descriptor),
            configuration: pse_ids::ContentHash::from_bytes([0x11; 32]),
            registry: registry.to_builder(),
            physical: super::super::handler::physical_identity(&registry).unwrap(),
            regime: "vapor".into(),
            request: ProviderRequest {
                outputs: vec![0],
                order: DerivativeOrder::Value,
                demand: ProviderDemand::CompleteJets,
            },
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
        wire.version = 2;
        assert!(wire.validate().is_err());
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
            ProviderDemand::CompleteJets,
            ProviderDemand::SelectedColumns {
                columns: vec![1, 0],
            },
            ProviderDemand::JacobianAction { directions: 3 },
            ProviderDemand::TransposeAction { cotangents: 2 },
            ProviderDemand::WeightedSecondAction { directions: 4 },
        ] {
            let encoded = encode_root::<_, pb::ProviderDemand>(expected.clone(), "ProviderDemand")
                .unwrap()
                .encode_to_vec();
            let generated: pb::ProviderDemand = decode(&encoded);
            generated.validate().unwrap();
            assert_eq!(
                decode_root::<ProviderDemand, _>(generated, "ProviderDemand").unwrap(),
                expected
            );
        }
    }

    #[test]
    fn preparation_uses_the_native_derivative_tags_and_keeps_selected_output_order() {
        for (order, tag) in [
            (DerivativeOrder::Value, 0x0a),
            (DerivativeOrder::First, 0x12),
            (DerivativeOrder::Second, 0x1a),
        ] {
            let mut native = preparation();
            native.request.order = order;
            native.request.outputs = vec![2, 0];
            let generated = encode_root::<_, pb::Prepare>(native, "Prepare").unwrap();
            generated.validate().unwrap();
            let request = generated.request.as_option().unwrap();
            assert_eq!(request.outputs, [2, 0]);
            let order_bytes = request.order.as_option().unwrap().encode_to_vec();
            assert_eq!(order_bytes, [tag, 0]);
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
