// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked mappings to generated binary messages; numerical cells retain bits.
use super::{WorkerError, generated::pb::pse::scientific_worker::v1 as pb, protocol as p};
use pse_kernels::ProviderError;

fn contract() -> WorkerError {
    ProviderError::Contract("malformed or unknown scientific worker wire meaning".into()).into()
}
fn unknown(fields: &buffa::UnknownFields) -> Result<(), WorkerError> {
    if fields.is_empty() {
        Ok(())
    } else {
        Err(contract())
    }
}
fn required<T: Default, P: buffa::ProtoBox<T>>(
    value: buffa::MessageField<T, P>,
) -> Result<T, WorkerError> {
    value.into_option().ok_or_else(contract)
}
fn id(value: &[u8]) -> Result<pse_ids::SemanticId, WorkerError> {
    pse_ids::SemanticId::try_from_slice(value).map_err(|_| contract())
}
fn hash(value: &[u8]) -> Result<pse_ids::ContentHash, WorkerError> {
    pse_ids::ContentHash::try_from_slice(value).map_err(|_| contract())
}
fn bits(value: Vec<f64>) -> Vec<u64> {
    value.into_iter().map(f64::to_bits).collect()
}
fn floats(value: Vec<u64>) -> Vec<f64> {
    value.into_iter().map(f64::from_bits).collect()
}

impl From<p::Handle> for pb::Handle {
    fn from(v: p::Handle) -> Self {
        Self {
            attempt: v.attempt.as_bytes().to_vec(),
            session: v.session.as_bytes().to_vec(),
            generation: v.generation.as_bytes().to_vec(),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Handle> for p::Handle {
    type Error = WorkerError;
    fn try_from(v: pb::Handle) -> Result<Self, Self::Error> {
        unknown(&v.__buffa_unknown_fields)?;
        Ok(Self {
            attempt: id(&v.attempt)?,
            session: id(&v.session)?,
            generation: id(&v.generation)?,
        })
    }
}
impl From<p::Port> for pb::Port {
    fn from(v: p::Port) -> Self {
        Self {
            id: v.id.as_bytes().to_vec(),
            quantity: v.quantity.as_bytes().to_vec(),
            unit: v.unit.as_bytes().to_vec(),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Port> for p::Port {
    type Error = WorkerError;
    fn try_from(v: pb::Port) -> Result<Self, Self::Error> {
        unknown(&v.__buffa_unknown_fields)?;
        Ok(Self {
            id: id(&v.id)?,
            quantity: id(&v.quantity)?,
            unit: id(&v.unit)?,
        })
    }
}
impl From<p::Spec> for pb::Spec {
    fn from(v: p::Spec) -> Self {
        Self {
            id: v.id.as_bytes().to_vec(),
            revision: v.revision.as_bytes().to_vec(),
            data: v.data.as_bytes().to_vec(),
            inputs: v.inputs.into_iter().map(Into::into).collect(),
            outputs: v.outputs.into_iter().map(Into::into).collect(),
            derivative_source: v.derivative_source,
            derivatives: v.derivatives,
            smoothness: v.smoothness,
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Spec> for p::Spec {
    type Error = WorkerError;
    fn try_from(v: pb::Spec) -> Result<Self, Self::Error> {
        unknown(&v.__buffa_unknown_fields)?;
        Ok(Self {
            id: id(&v.id)?,
            revision: hash(&v.revision)?,
            data: hash(&v.data)?,
            inputs: v
                .inputs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            outputs: v
                .outputs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            derivative_source: v.derivative_source,
            derivatives: v.derivatives,
            smoothness: v.smoothness,
        })
    }
}
impl From<p::Demand> for pb::Demand {
    fn from(v: p::Demand) -> Self {
        use pb::demand::{self, Kind};
        let kind = match v {
            p::Demand::CompleteJets => Kind::CompleteJets(demand::Alternative1::default().into()),
            p::Demand::SelectedColumns(value) => Kind::SelectedColumns(
                demand::Alternative2 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Demand::JacobianAction(value) => Kind::JacobianAction(
                demand::Alternative3 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Demand::TransposeAction(value) => Kind::TransposeAction(
                demand::Alternative4 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Demand::WeightedSecondAction(value) => Kind::WeightedSecondAction(
                demand::Alternative5 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
        };
        Self {
            kind: Some(kind),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Demand> for p::Demand {
    type Error = WorkerError;
    fn try_from(v: pb::Demand) -> Result<Self, Self::Error> {
        use pb::demand::Kind;
        unknown(&v.__buffa_unknown_fields)?;
        Ok(match v.kind.ok_or_else(contract)? {
            Kind::CompleteJets(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::CompleteJets
            }
            Kind::SelectedColumns(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::SelectedColumns(v.value)
            }
            Kind::JacobianAction(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::JacobianAction(v.value)
            }
            Kind::TransposeAction(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::TransposeAction(v.value)
            }
            Kind::WeightedSecondAction(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::WeightedSecondAction(v.value)
            }
        })
    }
}
impl From<p::Prepare> for pb::Prepare {
    fn from(v: p::Prepare) -> Self {
        Self {
            version: v.version,
            handle: pb::Handle::from(v.handle).into(),
            spec: pb::Spec::from(v.spec).into(),
            configuration: v.configuration.as_bytes().to_vec(),
            physical_registry: v.physical_registry,
            regime: v.regime,
            outputs: v.outputs,
            order: v.order,
            demand: pb::Demand::from(v.demand).into(),
            remaining_ns: v.remaining_ns,
            result_limit: v.result_limit,
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Prepare> for p::Prepare {
    type Error = WorkerError;
    fn try_from(v: pb::Prepare) -> Result<Self, Self::Error> {
        unknown(&v.__buffa_unknown_fields)?;
        Ok(Self {
            version: v.version,
            handle: required(v.handle)?.try_into()?,
            spec: required(v.spec)?.try_into()?,
            configuration: hash(&v.configuration)?,
            physical_registry: v.physical_registry,
            regime: v.regime,
            outputs: v.outputs,
            order: v.order,
            demand: required(v.demand)?.try_into()?,
            remaining_ns: v.remaining_ns,
            result_limit: v.result_limit,
        })
    }
}
impl From<p::Evaluate> for pb::Evaluate {
    fn from(v: p::Evaluate) -> Self {
        Self {
            handle: pb::Handle::from(v.handle).into(),
            invocation: v.invocation,
            inputs: bits(v.inputs),
            directions: bits(v.directions),
            cotangents: bits(v.cotangents),
            weights: bits(v.weights),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Evaluate> for p::Evaluate {
    type Error = WorkerError;
    fn try_from(v: pb::Evaluate) -> Result<Self, Self::Error> {
        unknown(&v.__buffa_unknown_fields)?;
        Ok(Self {
            handle: required(v.handle)?.try_into()?,
            invocation: v.invocation,
            inputs: floats(v.inputs),
            directions: floats(v.directions),
            cotangents: floats(v.cotangents),
            weights: floats(v.weights),
        })
    }
}
impl From<p::Request> for pb::Request {
    fn from(v: p::Request) -> Self {
        use pb::request::{self, Kind};
        let kind = match v {
            p::Request::Prepare(value) => Kind::Prepare(
                request::Alternative1 {
                    value: pb::Prepare::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Request::Evaluate(value) => Kind::Evaluate(
                request::Alternative2 {
                    value: pb::Evaluate::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Request::Stop(value) => Kind::Stop(
                request::Alternative3 {
                    value: pb::Handle::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Request::Close(value) => Kind::Close(
                request::Alternative4 {
                    value: pb::Handle::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
        };
        Self {
            kind: Some(kind),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Request> for p::Request {
    type Error = WorkerError;
    fn try_from(v: pb::Request) -> Result<Self, Self::Error> {
        use pb::request::Kind;
        unknown(&v.__buffa_unknown_fields)?;
        Ok(match v.kind.ok_or_else(contract)? {
            Kind::Prepare(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Prepare(required(v.value)?.try_into()?)
            }
            Kind::Evaluate(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Evaluate(required(v.value)?.try_into()?)
            }
            Kind::Stop(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Stop(required(v.value)?.try_into()?)
            }
            Kind::Close(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Close(required(v.value)?.try_into()?)
            }
        })
    }
}
impl From<p::Values> for pb::Values {
    fn from(v: p::Values) -> Self {
        Self {
            handle: pb::Handle::from(v.handle).into(),
            invocation: v.invocation,
            values: bits(v.values),
            jacobian: bits(v.jacobian),
            hessians: bits(v.hessians),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Values> for p::Values {
    type Error = WorkerError;
    fn try_from(v: pb::Values) -> Result<Self, Self::Error> {
        unknown(&v.__buffa_unknown_fields)?;
        Ok(Self {
            handle: required(v.handle)?.try_into()?,
            invocation: v.invocation,
            values: floats(v.values),
            jacobian: floats(v.jacobian),
            hessians: floats(v.hessians),
        })
    }
}
impl From<p::Failure> for pb::Failure {
    fn from(v: p::Failure) -> Self {
        use pb::failure::{self, Kind};
        let kind = match v {
            p::Failure::Cancelled => Kind::Cancelled(failure::Alternative1::default().into()),
            p::Failure::Deadline => Kind::Deadline(failure::Alternative2::default().into()),
            p::Failure::Resource(value) => Kind::Resource(
                failure::Alternative3 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Failure::Contract(value) => Kind::Contract(
                failure::Alternative4 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Failure::Trial(value) => Kind::Trial(
                failure::Alternative5 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Failure::OutsideEnvelope {
                axis,
                value,
                lower,
                upper,
            } => Kind::OutsideEnvelope(
                failure::Alternative6 {
                    axis,
                    value: value.to_bits(),
                    lower: lower.to_bits(),
                    upper: upper.to_bits(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Failure::Terminal(value) => Kind::Terminal(
                failure::Alternative7 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
            p::Failure::UnknownCompletion(value) => Kind::UnknownCompletion(
                failure::Alternative8 {
                    value,
                    ..Default::default()
                }
                .into(),
            ),
        };
        Self {
            kind: Some(kind),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Failure> for p::Failure {
    type Error = WorkerError;
    fn try_from(v: pb::Failure) -> Result<Self, Self::Error> {
        use pb::failure::Kind;
        unknown(&v.__buffa_unknown_fields)?;
        Ok(match v.kind.ok_or_else(contract)? {
            Kind::Cancelled(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Cancelled
            }
            Kind::Deadline(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Deadline
            }
            Kind::Resource(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Resource(v.value)
            }
            Kind::Contract(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Contract(v.value)
            }
            Kind::Trial(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Trial(v.value)
            }
            Kind::OutsideEnvelope(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::OutsideEnvelope {
                    axis: v.axis,
                    value: f64::from_bits(v.value),
                    lower: f64::from_bits(v.lower),
                    upper: f64::from_bits(v.upper),
                }
            }
            Kind::Terminal(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Terminal(v.value)
            }
            Kind::UnknownCompletion(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::UnknownCompletion(v.value)
            }
        })
    }
}
impl From<p::Response> for pb::Response {
    fn from(v: p::Response) -> Self {
        use pb::response::{self, Kind};
        let kind = match v {
            p::Response::Prepared(value) => Kind::Prepared(
                response::Alternative1 {
                    value: pb::Handle::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Response::Values(value) => Kind::Values(
                response::Alternative2 {
                    value: pb::Values::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Response::Failure(value) => Kind::Failure(
                response::Alternative3 {
                    value: pb::Failure::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
            p::Response::Closed(value) => Kind::Closed(
                response::Alternative4 {
                    value: pb::Handle::from(value).into(),
                    ..Default::default()
                }
                .into(),
            ),
        };
        Self {
            kind: Some(kind),
            ..Default::default()
        }
    }
}
impl TryFrom<pb::Response> for p::Response {
    type Error = WorkerError;
    fn try_from(v: pb::Response) -> Result<Self, Self::Error> {
        use pb::response::Kind;
        unknown(&v.__buffa_unknown_fields)?;
        Ok(match v.kind.ok_or_else(contract)? {
            Kind::Prepared(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Prepared(required(v.value)?.try_into()?)
            }
            Kind::Values(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Values(required(v.value)?.try_into()?)
            }
            Kind::Failure(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Failure(required(v.value)?.try_into()?)
            }
            Kind::Closed(v) => {
                unknown(&v.__buffa_unknown_fields)?;
                Self::Closed(required(v.value)?.try_into()?)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
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
        let encoded = pb::Request::from(request).encode_to_vec();
        let generated: pb::Request = decode(&encoded);
        generated.validate().unwrap();
        let p::Request::Evaluate(actual) = p::Request::try_from(generated).unwrap() else {
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

    #[test]
    fn generated_preparation_validator_preserves_absent_deadline_and_refuses_present_zero() {
        let registry = pse_quantity::generated::standard_registry().unwrap();
        let descriptor = pse_backend_native::coolprop::descriptor(
            &registry,
            pse_backend_native::coolprop::WaterRegime::Vapor,
        )
        .unwrap();
        let request = p::Prepare {
            version: 1,
            handle: handle(),
            spec: p::Spec::capture(descriptor.spec()).unwrap(),
            configuration: pse_ids::ContentHash::from_bytes([0x11; 32]),
            physical_registry: "pse.standard.v1".into(),
            regime: "vapor".into(),
            outputs: vec![0],
            order: 0,
            demand: p::Demand::CompleteJets,
            remaining_ns: None,
            result_limit: 4096,
        };
        for deadline in [None, Some(1)] {
            let mut request = request.clone();
            request.remaining_ns = deadline;
            let generated = decode::<pb::Prepare>(&pb::Prepare::from(request).encode_to_vec());
            generated.validate().unwrap();
            assert_eq!(
                p::Prepare::try_from(generated).unwrap().remaining_ns,
                deadline
            );
        }
        let mut invalid = pb::Prepare::from(request);
        invalid.remaining_ns = Some(0);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn absent_unknown_truncated_and_oversized_wire_refuse_without_a_native_owner() {
        assert!(pb::Request::default().validate().is_err());
        assert!(p::Request::try_from(pb::Request::default()).is_err());
        let mut encoded = pb::Request::from(p::Request::Close(handle())).encode_to_vec();
        assert!(pb::Request::decode_from_slice(&encoded[..encoded.len() - 1]).is_err());
        assert!(
            buffa::DecodeOptions::new()
                .with_max_message_size(encoded.len() - 1)
                .decode_from_slice::<pb::Request>(&encoded)
                .is_err()
        );
        // Unknown required meaning is rejected even when protobuf preserves it.
        encoded.extend([0xa0, 0x06, 0x01]);
        assert!(p::Request::try_from(decode::<pb::Request>(&encoded)).is_err());
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
            let encoded = pb::Demand::from(expected.clone()).encode_to_vec();
            let generated: pb::Demand = decode(&encoded);
            generated.validate().unwrap();
            assert_eq!(p::Demand::try_from(generated).unwrap(), expected);
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
        let encoded = pb::Failure::from(original).encode_to_vec();
        let generated: pb::Failure = decode(&encoded);
        generated.validate().unwrap();
        let WorkerError::Provider(ProviderError::OutsideEnvelope {
            value,
            lower,
            upper,
            ..
        }) = super::super::handler::refusal(p::Failure::try_from(generated).unwrap())
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
