// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One child-native session; public wire claims never establish native meaning.
use super::{PreparedSession, SessionHandle, WorkerError, protocol as p};
use pse_kernels::{
    DerivativeOrder, ProviderDemand, ProviderError, ProviderRequest, ProviderRuntime,
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
    failure_observation(&error)
}

/// Borrow the original performing owner's error without adopting or consuming
/// it. The joined native result remains owned independently of this observation.
pub(super) fn failure_observation(error: &WorkerError) -> p::Failure {
    observe_with_budget(
        error,
        FailureBudget {
            nodes: p::FAILURE_NODES,
            text: p::FAILURE_RETAINED_TEXT,
        },
    )
}
fn observe_with_budget(error: &WorkerError, mut budget: FailureBudget) -> p::Failure {
    match capture_failure(error, 0, &mut budget) {
        Ok(failure) if failure.within_limits() => failure,
        _ => p::Failure::UnknownCompletion("scientific worker could not transfer the complete failure within its declared failure allowance".into()),
    }
}

struct FailureBudget {
    nodes: usize,
    text: usize,
}
impl FailureBudget {
    fn node(&mut self, depth: usize) -> Result<(), ()> {
        if depth > p::FAILURE_DEPTH {
            return Err(());
        }
        self.nodes = self.nodes.checked_sub(1).ok_or(())?;
        Ok(())
    }
    fn text(&mut self, value: &str) -> Result<String, ()> {
        self.text = self.text.checked_sub(value.len()).ok_or(())?;
        let mut copy = String::new();
        copy.try_reserve_exact(value.len()).map_err(|_| ())?;
        copy.push_str(value);
        Ok(copy)
    }
    fn vector<T>(&self, length: usize) -> Result<Vec<T>, ()> {
        if length > self.nodes {
            return Err(());
        }
        let mut values = Vec::new();
        values.try_reserve_exact(length).map_err(|_| ())?;
        Ok(values)
    }
}

fn admit_diagnostic_shape(
    error: &dyn pse_diagnostics::TypedDiagnostic,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<(), ()> {
    budget.node(depth)?;
    if let Some(children) = error.diagnostic_children() {
        for child in children {
            admit_diagnostic_shape(child, depth + 1, budget)?;
        }
    }
    Ok(())
}

fn capture_diagnostic<
    T: pse_model::diagnostic::DiagnosticProjection + pse_diagnostics::TypedDiagnostic,
>(
    error: &T,
    retained: impl FnOnce() -> Option<usize>,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<p::Failure, ()> {
    // Unknown library-owned retention is not an allocation allowance. Inspect
    // shape and admit the known original extent before copying source facts.
    admit_diagnostic_shape(error, depth, budget)?;
    budget.text = budget.text.checked_sub(retained().ok_or(())?).ok_or(())?;
    Ok(p::Failure::Diagnostic(Box::new(error.boundary_diagnostic(
        pse_diagnostics::DiagnosticStage::Evaluation,
    ))))
}

fn capture_failure(
    error: &WorkerError,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<p::Failure, ()> {
    budget.node(depth)?;
    Ok(match error {
        WorkerError::Provider(error) => return capture_provider_failure(error, depth, budget),
        WorkerError::Retirement { causes } => {
            if causes.is_empty() || causes.len() > 128 {
                return Err(());
            }
            let mut copied = budget.vector(causes.len())?;
            for cause in causes {
                copied.push(capture_failure(cause, depth + 1, budget)?);
            }
            p::Failure::Retirement { causes: copied }
        }
        WorkerError::NativePanic(detail) => p::Failure::NativePanic(budget.text(detail)?),
        WorkerError::RemoteDiagnostic(diagnostic) => {
            use pse_model::HeapUsage as _;
            // The already passive tree has no native type reconstruction.
            // Its finite extent is checked before cloning its owned buffers.
            fn shape(
                value: &pse_model::diagnostic::BoundaryDiagnostic,
                depth: usize,
                budget: &mut FailureBudget,
            ) -> Result<(), ()> {
                budget.node(depth)?;
                for child in &value.causes {
                    shape(child, depth + 1, budget)?;
                }
                Ok(())
            }
            shape(diagnostic, depth + 1, budget)?;
            budget.text = budget.text.checked_sub(diagnostic.heap_bytes()).ok_or(())?;
            p::Failure::Diagnostic(diagnostic.clone())
        }
        WorkerError::RemoteResource(reason) => p::Failure::Resource(budget.text(reason)?),
        WorkerError::UnknownCompletion(reason) => {
            p::Failure::UnknownCompletion(budget.text(reason)?)
        }
        error => return capture_diagnostic(error, || error.retained_bytes(), depth, budget),
    })
}

fn capture_provider_failure(
    error: &ProviderError,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<p::Failure, ()> {
    Ok(match error {
        ProviderError::Retirement { failures } => {
            if failures.is_empty() || failures.len() > 64 {
                return Err(());
            }
            let mut copied = budget.vector(failures.len())?;
            for failure in failures {
                budget.node(depth + 1)?;
                copied.push(p::KeyedFailure {
                    key: failure.key.0,
                    cause: Box::new(capture_provider_failure(&failure.cause, depth + 1, budget)?),
                });
            }
            p::Failure::ProviderRetirement { failures: copied }
        }
        ProviderError::Settlement { operation, cleanup } => {
            budget.node(depth + 1)?;
            let operation = capture_provider_failure(operation, depth + 1, budget)?;
            budget.node(depth + 1)?;
            p::Failure::ProviderSettlement {
                operation: Box::new(operation),
                cleanup: Box::new(capture_provider_failure(cleanup, depth + 1, budget)?),
            }
        }
        ProviderError::RetirementPanic(detail) => {
            p::Failure::ProviderRetirementPanic(budget.text(detail)?)
        }
        ProviderError::Singular(detail) => p::Failure::Singular(budget.text(detail)?),
        ProviderError::LinearizationChanged => p::Failure::LinearizationChanged,
        ProviderError::RegimeCrossing {
            selector,
            bound,
            selected,
        } => p::Failure::RegimeCrossing {
            selector: *selector,
            bound: *bound,
            selected: *selected,
        },
        ProviderError::Cancelled => p::Failure::Cancelled,
        ProviderError::Deadline => p::Failure::Deadline,
        ProviderError::Limit(reason) => p::Failure::Resource(budget.text(reason)?),
        ProviderError::Contract(reason) => p::Failure::Contract(budget.text(reason)?),
        ProviderError::Trial(reason) => p::Failure::Trial(budget.text(reason)?),
        ProviderError::OutsideEnvelope {
            axis,
            value: x,
            lower,
            upper,
        } => p::Failure::OutsideEnvelope {
            axis: budget.text(axis)?,
            value: *x,
            lower: *lower,
            upper: *upper,
        },
        ProviderError::Terminal(detail) => p::Failure::Terminal(budget.text(detail)?),
        error => return capture_diagnostic(error, || Some(error.retained_bytes()), depth, budget),
    })
}
pub(super) fn refusal(value: p::Failure) -> WorkerError {
    if !value.within_limits() {
        return WorkerError::UnknownCompletion(
            "scientific worker failure exceeds its declared restoration allowance".into(),
        );
    }
    restore_failure(value)
}
fn restore_failure(value: p::Failure) -> WorkerError {
    match value {
        p::Failure::ProviderRetirement { failures } => ProviderError::Retirement {
            failures: failures
                .into_iter()
                .map(|failure| pse_kernels::ProviderRetirementFailure {
                    key: pse_kernels::ProviderKey(failure.key),
                    cause: Box::new(restore_failure(*failure.cause).into_provider()),
                })
                .collect(),
        }
        .into(),
        p::Failure::ProviderSettlement { operation, cleanup } => ProviderError::Settlement {
            operation: Box::new(restore_failure(*operation).into_provider()),
            cleanup: Box::new(restore_failure(*cleanup).into_provider()),
        }
        .into(),
        p::Failure::Retirement { causes } => WorkerError::Retirement {
            causes: causes.into_iter().map(restore_failure).collect(),
        },
        p::Failure::ProviderRetirementPanic(detail) => {
            ProviderError::RetirementPanic(detail).into()
        }
        p::Failure::NativePanic(detail) => WorkerError::NativePanic(detail),
        p::Failure::Singular(detail) => ProviderError::Singular(detail).into(),
        p::Failure::LinearizationChanged => ProviderError::LinearizationChanged.into(),
        p::Failure::RegimeCrossing {
            selector,
            bound,
            selected,
        } => ProviderError::RegimeCrossing {
            selector,
            bound,
            selected,
        }
        .into(),
        p::Failure::Diagnostic(diagnostic) => WorkerError::RemoteDiagnostic(diagnostic),
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

/// Water consumes physical declarations directly, without authored-operation prerequisites.
pub(super) fn physical_identity(
    registry: &pse_quantity::QuantityRegistry,
) -> Result<pse_ids::ContentHash, WorkerError> {
    let prerequisites = pse_quantity::PhysicalPreconditions::new(Vec::new())
        .map_err(|error| ProviderError::Contract(error.to_string()))?;
    Ok(pse_quantity::physical_identity::identity(
        registry,
        &prerequisites,
    ))
}

#[cfg(feature = "property-coolprop")]
pub(super) fn prepare(
    request: p::Prepare,
    expected_session: pse_ids::SemanticId,
    cancellation: pse_diagnostics::Cancellation,
) -> Result<PreparedSession, WorkerError> {
    let received_at = std::time::Instant::now();
    if request.version != 2
        || request.handle.session != expected_session
        || [
            request.handle.attempt,
            request.handle.session,
            request.handle.generation,
        ]
        .contains(&pse_ids::SemanticId::NIL)
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
    let registry = request
        .registry
        .build()
        .map_err(|error| ProviderError::Contract(error.to_string()))?;
    if physical_identity(&registry)? != request.physical {
        return Err(ProviderError::Contract(
            "scientific worker physical declarations differ".into(),
        )
        .into());
    }
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
    if pse_kernels::portable::ProviderRecord::capture(&admitted) != request.spec {
        return Err(ProviderError::Contract(
            "scientific worker water descriptor or product unavailable".into(),
        )
        .into());
    }
    let checked = pse_kernels::PreparedProviderRequest::new(
        std::sync::Arc::new(admitted.spec().clone()),
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
    scope.check()?;
    let factory = pse_backend_native::coolprop::WaterFactory::prepare(&registry, regime)?;
    PreparedSession::prepare(
        request.handle.into(),
        &factory,
        admitted.spec(),
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
pub(super) fn evaluate_batch(
    session: &mut PreparedSession,
    value: p::EvaluateBatch,
) -> Result<p::Batch, WorkerError> {
    let floats = |bits: Vec<u64>| bits.into_iter().map(f64::from_bits).collect();
    let points: Vec<_> = value
        .points
        .into_iter()
        .map(|point| pse_kernels::ProviderBatchPoint {
            id: point.id,
            inputs: floats(point.inputs),
            directions: floats(point.directions),
            cotangents: floats(point.cotangents),
            weights: floats(point.weights),
        })
        .collect();
    let completed =
        session.evaluate_batch(value.handle.clone().into(), value.invocation, &points)?;
    Ok(p::Batch {
        handle: value.handle,
        invocation: value.invocation,
        points: completed
            .into_iter()
            .map(|point| p::BatchOutcome {
                id: point.id,
                result: match point.result {
                    Ok(values) => p::BatchResult::Product(p::BatchProduct {
                        values: values.values,
                        jacobian: values.jacobian,
                        hessians: values.hessians,
                    }),
                    Err(error) => p::BatchResult::Refused(failure(error.into())),
                },
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_kernels::ProviderFactory as _;

    #[test]
    fn borrowed_failure_refuses_oversize_or_copy_budget_without_consuming_original() {
        let error = WorkerError::Provider(ProviderError::Contract(
            "x".repeat(p::FAILURE_RETAINED_TEXT + 1),
        ));
        assert!(matches!(
            failure_observation(&error),
            p::Failure::UnknownCompletion(_)
        ));
        assert!(
            matches!(&error, WorkerError::Provider(ProviderError::Contract(detail)) if detail.len() == p::FAILURE_RETAINED_TEXT + 1)
        );
        let original = WorkerError::Provider(ProviderError::Contract(
            "original owned native cause".into(),
        ));
        let refused = observe_with_budget(
            &original,
            FailureBudget {
                nodes: p::FAILURE_NODES,
                text: 0,
            },
        );
        assert!(matches!(refused, p::Failure::UnknownCompletion(_)));
        assert!(
            matches!(failure_observation(&original), p::Failure::Contract(detail) if detail == "original owned native cause")
        );
        // Actual collection reservation refusal is recoverable too, without
        // trying to allocate an enormous fixture or reducing any host budget.
        let budget = FailureBudget {
            nodes: usize::MAX,
            text: 0,
        };
        assert!(budget.vector::<u64>(usize::MAX).is_err());
    }

    #[test]
    fn borrowed_failure_refuses_unknown_opaque_extent_and_deep_tree() {
        let original = WorkerError::Process {
            action: "retire",
            source: std::io::Error::other("original OS failure"),
        };
        assert!(matches!(
            failure_observation(&original),
            p::Failure::UnknownCompletion(_)
        ));
        assert!(matches!(
            original,
            WorkerError::Process {
                action: "retire",
                ..
            }
        ));
        let mut deep = WorkerError::Provider(ProviderError::Contract("original deep cause".into()));
        for _ in 0..p::FAILURE_DEPTH + 1 {
            deep = WorkerError::Retirement { causes: vec![deep] };
        }
        assert!(matches!(
            failure_observation(&deep),
            p::Failure::UnknownCompletion(_)
        ));
    }

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
            version: 2,
            handle: handle.clone(),
            spec: pse_kernels::portable::ProviderRecord::capture(
                &pse_backend_native::coolprop::descriptor(
                    &registry,
                    pse_backend_native::coolprop::WaterRegime::Vapor,
                )
                .unwrap(),
            ),
            configuration: factory.configuration_key(),
            registry: registry.to_builder(),
            physical: physical_identity(&registry).unwrap(),
            regime: "vapor".into(),
            outputs: vec![0, 1, 2],
            order: 2,
            demand: p::Demand::CompleteJets,
            remaining_ns: Some(60_000_000_000),
            result_limit: 4096,
        };
        let mut variants = Vec::new();
        let mut request = original.clone();
        let mut changed = serde_json::to_value(&request.spec).unwrap();
        changed["spec"]["data"] =
            serde_json::to_value(pse_ids::ContentHash::from_bytes([9; 32])).unwrap();
        request.spec = serde_json::from_value(changed).unwrap();
        variants.push(request);
        let mut request = original.clone();
        let mut changed = serde_json::to_value(&request.spec).unwrap();
        changed["spec"]["inputs"][0]["unit"] = changed["spec"]["inputs"][1]["unit"].clone();
        request.spec = serde_json::from_value(changed).unwrap();
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
        let mut request = original.clone();
        let first = registry.quantity_types().next().unwrap().clone();
        request.registry.quantity_type(first);
        variants.push(request);
        let mut request = original.clone();
        request.physical = pse_ids::ContentHash::from_bytes([8; 32]);
        variants.push(request);
        let mut request = original.clone();
        let mut declarations = serde_json::to_value(&request.registry).unwrap();
        let used_unit = factory.spec().inputs[0].unit.to_string();
        let units = declarations["units"].as_array_mut().unwrap();
        let mut changed = 0;
        for unit in units {
            if unit["id"] == used_unit {
                let scale = unit["scale_to_canonical"].as_f64().unwrap();
                unit["scale_to_canonical"] = serde_json::json!(scale * 2.0);
                changed += 1;
            }
        }
        assert_eq!(changed, 1);
        request.registry = serde_json::from_value(declarations).unwrap();
        // This inventory remains natively valid, but no longer means what the parent prepared.
        request.registry.clone().build().unwrap();
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
