// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One child-native session; public wire claims never establish native meaning.
use super::{PreparedSession, SessionHandle, WorkerError, protocol as p};
use pse_kernels::{
    DerivativeOrder, ProviderDemand, ProviderError, ProviderRequest, ProviderRuntime, ProviderSpec,
};

impl From<SessionHandle> for p::Handle {
    fn from(value: SessionHandle) -> Self {
        Self {
            attempt: value.attempt,
            session: value.session,
            generation: value.generation,
        }
    }
}
impl From<p::Handle> for SessionHandle {
    fn from(value: p::Handle) -> Self {
        Self {
            attempt: value.attempt,
            session: value.session,
            generation: value.generation,
        }
    }
}
impl p::Spec {
    pub(super) fn capture(spec: &ProviderSpec) -> Result<Self, WorkerError> {
        if spec.shapes != pse_kernels::ProviderShapes::default() {
            return Err(ProviderError::Contract(
                "scientific worker v1 requires scalar water ports".into(),
            )
            .into());
        }
        let ports = |ports: &[pse_kernels::Port]| {
            ports
                .iter()
                .map(|port| p::Port {
                    id: port.id,
                    quantity: port.quantity.as_id(),
                    unit: port.unit.as_id(),
                })
                .collect()
        };
        Ok(Self {
            id: spec.id,
            revision: spec.revision,
            data: spec.data,
            inputs: ports(&spec.inputs),
            outputs: ports(&spec.outputs),
            derivative_source: spec.derivative_source.as_str().into(),
            derivatives: spec.derivatives as u32,
            smoothness: spec.smoothness as u32,
        })
    }
    pub(super) fn restore(&self) -> Result<ProviderSpec, WorkerError> {
        let ports = |ports: &[p::Port]| {
            ports
                .iter()
                .map(|port| pse_kernels::Port {
                    id: port.id,
                    quantity: pse_quantity::QuantityTypeId::from_id(port.quantity),
                    unit: pse_quantity::UnitId::from_id(port.unit),
                })
                .collect()
        };
        Ok(ProviderSpec {
            id: self.id,
            revision: self.revision,
            data: self.data,
            inputs: ports(&self.inputs),
            outputs: ports(&self.outputs),
            shapes: pse_kernels::ProviderShapes::default(),
            derivative_source: self
                .derivative_source
                .parse()
                .map_err(|_| ProviderError::Contract("unknown worker derivative source".into()))?,
            derivatives: order(self.derivatives)?,
            smoothness: order(self.smoothness)?,
        })
    }
}
pub(super) fn order(value: u32) -> Result<DerivativeOrder, ProviderError> {
    match value {
        0 => Ok(DerivativeOrder::Value),
        1 => Ok(DerivativeOrder::First),
        2 => Ok(DerivativeOrder::Second),
        _ => Err(ProviderError::Contract(
            "unknown scientific worker derivative order".into(),
        )),
    }
}
impl p::Demand {
    pub(super) fn capture(value: &ProviderDemand) -> Self {
        match value {
            ProviderDemand::CompleteJets => Self::CompleteJets,
            ProviderDemand::SelectedColumns { columns } => {
                Self::SelectedColumns(columns.iter().map(|&n| n as u64).collect())
            }
            ProviderDemand::JacobianAction { directions } => {
                Self::JacobianAction(*directions as u64)
            }
            ProviderDemand::TransposeAction { cotangents } => {
                Self::TransposeAction(*cotangents as u64)
            }
            ProviderDemand::WeightedSecondAction { directions } => {
                Self::WeightedSecondAction(*directions as u64)
            }
        }
    }
    pub(super) fn restore(&self) -> Result<ProviderDemand, ProviderError> {
        let extent = |n: u64| {
            usize::try_from(n).map_err(|_| ProviderError::Limit("scientific worker demand extent"))
        };
        Ok(match self {
            Self::CompleteJets => ProviderDemand::CompleteJets,
            Self::SelectedColumns(columns) => ProviderDemand::SelectedColumns {
                columns: columns
                    .iter()
                    .map(|&n| extent(n))
                    .collect::<Result<_, _>>()?,
            },
            Self::JacobianAction(n) => ProviderDemand::JacobianAction {
                directions: extent(*n)?,
            },
            Self::TransposeAction(n) => ProviderDemand::TransposeAction {
                cotangents: extent(*n)?,
            },
            Self::WeightedSecondAction(n) => ProviderDemand::WeightedSecondAction {
                directions: extent(*n)?,
            },
        })
    }
}

/// Typed selected-water failure, retaining envelope coordinates as exact bits.
pub(super) fn failure(error: WorkerError) -> p::Failure {
    match error {
        WorkerError::Provider(ProviderError::Cancelled) => p::Failure::Cancelled,
        WorkerError::Provider(ProviderError::Deadline) => p::Failure::Deadline,
        WorkerError::Provider(ProviderError::Limit(reason)) => p::Failure::Resource(reason.into()),
        WorkerError::RemoteResource(reason) => p::Failure::Resource(reason),
        WorkerError::Provider(ProviderError::Contract(reason)) => p::Failure::Contract(reason),
        WorkerError::Provider(ProviderError::Trial(reason)) => p::Failure::Trial(reason),
        WorkerError::Provider(ProviderError::OutsideEnvelope {
            axis,
            value: x,
            lower,
            upper,
        }) => p::Failure::OutsideEnvelope {
            axis,
            value: x,
            lower,
            upper,
        },
        WorkerError::UnknownCompletion(reason) => p::Failure::UnknownCompletion(reason),
        error => p::Failure::Terminal(error.to_string()),
    }
}
pub(super) fn refusal(value: p::Failure) -> WorkerError {
    match value {
        p::Failure::Cancelled => ProviderError::Cancelled.into(),
        p::Failure::Deadline => ProviderError::Deadline.into(),
        p::Failure::Resource(reason) => WorkerError::RemoteResource(reason),
        p::Failure::Contract(reason) => ProviderError::Contract(reason).into(),
        p::Failure::Trial(reason) => ProviderError::Trial(reason).into(),
        p::Failure::OutsideEnvelope {
            axis,
            value,
            lower,
            upper,
        } => ProviderError::OutsideEnvelope {
            axis,
            value,
            lower,
            upper,
        }
        .into(),
        p::Failure::Terminal(reason) => ProviderError::Terminal(reason).into(),
        p::Failure::UnknownCompletion(reason) => WorkerError::UnknownCompletion(reason),
    }
}

#[cfg(feature = "property-coolprop")]
pub(super) fn prepare(
    request: p::Prepare,
    expected_session: pse_ids::SemanticId,
    cancellation: pse_diagnostics::Cancellation,
) -> Result<PreparedSession, WorkerError> {
    let received_at = std::time::Instant::now();
    if request.version != 1
        || request.handle.session != expected_session
        || [
            request.handle.attempt,
            request.handle.session,
            request.handle.generation,
        ]
        .contains(&pse_ids::SemanticId::NIL)
        || request.physical_registry != "pse.standard.v1"
        || request.remaining_ns == Some(0)
        || request.result_limit == 0
        || request.result_limit > 1_048_576
    {
        return Err(ProviderError::Contract(
            "scientific worker version/session/physical interpretation or bounds differ".into(),
        )
        .into());
    }
    let regime = match request.regime.as_str() {
        "liquid" => pse_backend_native::coolprop::WaterRegime::Liquid,
        "vapor" => pse_backend_native::coolprop::WaterRegime::Vapor,
        "supercritical" => pse_backend_native::coolprop::WaterRegime::Supercritical,
        _ => {
            return Err(ProviderError::Contract(
                "unsupported scientific worker water regime".into(),
            )
            .into());
        }
    };
    let registry = pse_quantity::generated::standard_registry()
        .map_err(|error| ProviderError::Contract(error.to_string()))?;
    let expected = request.spec.restore()?;
    let demand = ProviderRequest {
        outputs: request
            .outputs
            .iter()
            .map(|&n| {
                usize::try_from(n)
                    .map_err(|_| ProviderError::Limit("scientific worker output ordinal"))
            })
            .collect::<Result<_, _>>()?,
        order: order(request.order)?,
        demand: request.demand.restore()?,
    };
    let admitted = pse_backend_native::coolprop::descriptor(&registry, regime)?;
    if admitted.spec() != &expected || !matches!(demand.demand, ProviderDemand::CompleteJets) {
        return Err(ProviderError::Contract(
            "scientific worker water descriptor or product unavailable".into(),
        )
        .into());
    }
    let checked = pse_kernels::PreparedProviderRequest::new(
        std::sync::Arc::new(expected.clone()),
        demand.clone(),
    )?;
    let flag = cancellation.flag();
    checked.check_context(&pse_kernels::EvaluationContext {
        cancelled: &flag,
        max_result_bytes: request.result_limit as usize,
    })?;
    // Parent retains its original absolute clock across transfer. A child clock
    // exists only when that original scope has one; reuse never renews it.
    let scope = scientific_scope(cancellation, request.remaining_ns, received_at)?;
    PreparedSession::prepare_water(
        request.handle.into(),
        &registry,
        regime,
        &expected,
        request.configuration,
        demand,
        scope,
        request.result_limit as usize,
    )
}

pub(super) fn scientific_scope(
    cancellation: pse_diagnostics::Cancellation,
    remaining_ns: Option<u64>,
    received_at: std::time::Instant,
) -> Result<pse_kernels::ExecutionScope, WorkerError> {
    let deadline = remaining_ns
        .map(|remaining| {
            if remaining == 0 {
                return Err(ProviderError::Deadline);
            }
            received_at
                .checked_add(std::time::Duration::from_nanos(remaining))
                .ok_or(ProviderError::Limit("scientific worker deadline extent"))
        })
        .transpose()?;
    Ok(pse_kernels::ExecutionScope::new(cancellation, deadline))
}

pub(super) fn evaluate(
    session: &mut PreparedSession,
    value: p::Evaluate,
) -> Result<p::Values, WorkerError> {
    let completed = session.evaluate(
        value.handle.clone().into(),
        value.invocation,
        &value.inputs,
        ProviderRuntime {
            directions: &value.directions,
            cotangents: &value.cotangents,
            weights: &value.weights,
            expected_linearization: None,
        },
    )?;
    Ok(p::Values {
        handle: value.handle,
        invocation: value.invocation,
        values: completed.values,
        jacobian: completed.jacobian,
        hessians: completed.hessians,
    })
}
pub(super) fn evaluate_batch(session: &mut PreparedSession, value: p::EvaluateBatch) -> Result<p::Batch, WorkerError> {
    let floats = |bits: Vec<u64>| bits.into_iter().map(f64::from_bits).collect();
    let points: Vec<_> = value.points.into_iter().map(|point| pse_kernels::ProviderBatchPoint {
        id: point.id, inputs: floats(point.inputs), directions: floats(point.directions),
        cotangents: floats(point.cotangents), weights: floats(point.weights),
    }).collect();
    let completed = session.evaluate_batch(value.handle.clone().into(), value.invocation, &points)?;
    Ok(p::Batch { handle: value.handle, invocation: value.invocation,
        points: completed.into_iter().map(|point| p::BatchOutcome { id: point.id,
            result: match point.result {
                Ok(values) => p::BatchResult::Product(p::BatchProduct { values: values.values,
                    jacobian: values.jacobian, hessians: values.hessians }),
                Err(error) => p::BatchResult::Refused(failure(error.into())),
            },
        }).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_kernels::ProviderFactory as _;

    #[test]
    fn water_preparation_refuses_changed_data_units_regime_and_datum_configuration() {
        let registry = pse_quantity::generated::standard_registry().unwrap();
        let factory = pse_backend_native::coolprop::WaterFactory::prepare(
            &registry,
            pse_backend_native::coolprop::WaterRegime::Vapor,
        )
        .unwrap();
        let handle = p::Handle {
            attempt: pse_ids::SemanticId::from_bytes([1; 16]),
            session: pse_ids::SemanticId::from_bytes([2; 16]),
            generation: pse_ids::SemanticId::from_bytes([3; 16]),
        };
        let original = p::Prepare {
            version: 1,
            handle: handle.clone(),
            spec: p::Spec::capture(factory.spec()).unwrap(),
            configuration: factory.configuration_key(),
            physical_registry: "pse.standard.v1".into(),
            regime: "vapor".into(),
            outputs: vec![0, 1, 2],
            order: 2,
            demand: p::Demand::CompleteJets,
            remaining_ns: Some(60_000_000_000),
            result_limit: 4096,
        };
        let mut variants = Vec::new();
        let mut request = original.clone();
        request.spec.data = pse_ids::ContentHash::from_bytes([9; 32]);
        variants.push(request);
        let mut request = original.clone();
        request.spec.inputs[0].unit = request.spec.inputs[1].unit;
        variants.push(request);
        let mut request = original.clone();
        request.regime = "liquid".into();
        variants.push(request);
        let mut request = original.clone();
        request.configuration = pse_ids::ContentHash::from_bytes([7; 32]);
        variants.push(request);
        let mut request = original.clone();
        request.outputs = vec![0, 0];
        variants.push(request);
        let mut request = original;
        request.demand = p::Demand::WeightedSecondAction(1);
        variants.push(request);
        for request in variants {
            assert!(matches!(
                prepare(
                    request,
                    handle.session,
                    pse_diagnostics::Cancellation::default()
                ),
                Err(WorkerError::Provider(ProviderError::Contract(_)))
            ));
        }
    }
}
