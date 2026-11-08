// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One child-native session; public wire claims never establish native meaning.
use super::{
    PreparedSession, SessionHandle, WorkerError,
    failure_budget::{FAILURE_DEPTH, FAILURE_NODES, FAILURE_RETAINED_TEXT, failure_within_limits},
};
use pse_diagnostics::{
    DiagnosticCode, DiagnosticFacts, DiagnosticObservation, DiagnosticStage, TypedDiagnostic,
};
use pse_kernels::{
    DerivativeOrder, ProviderError, ProviderRequest, ProviderRuntime, worker_protocol as p,
};
use pse_model::{
    HeapUsage as _,
    diagnostic::{BoundaryDiagnostic, DiagnosticProjection as _, Observation, project_facts},
};

const FAILURE_STAGE: DiagnosticStage = DiagnosticStage::Evaluation;

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
            nodes: FAILURE_NODES,
            text: FAILURE_RETAINED_TEXT,
        },
    )
}
fn observe_with_budget(error: &WorkerError, mut budget: FailureBudget) -> p::Failure {
    match capture_failure(error, 0, &mut budget) {
        Ok(failure) if failure_within_limits(&failure) => failure,
        _ => p::Failure::UnknownCompletion("scientific worker could not transfer the complete failure within its declared failure allowance".into()),
    }
}

struct FailureBudget {
    nodes: usize,
    text: usize,
}
impl FailureBudget {
    fn node(&mut self, depth: usize) -> Result<(), ()> {
        if depth > FAILURE_DEPTH {
            return Err(());
        }
        self.nodes = self.nodes.checked_sub(1).ok_or(())?;
        Ok(())
    }
    fn text(&mut self, value: &str) -> Result<String, ()> {
        self.charge(value.len())?;
        let mut copy = String::new();
        copy.try_reserve_exact(value.len()).map_err(|_| ())?;
        copy.push_str(value);
        Ok(copy)
    }
    fn charge(&mut self, bytes: usize) -> Result<(), ()> {
        self.text = self.text.checked_sub(bytes).ok_or(())?;
        Ok(())
    }
    fn vector<T>(&self, length: usize) -> Result<Vec<T>, ()> {
        if length > self.nodes {
            return Err(());
        }
        let mut values = Vec::new();
        values.try_reserve_exact(length).map_err(|_| ())?;
        Ok(values)
    }
    fn diagnostic_children(&mut self, length: usize) -> Result<Vec<BoundaryDiagnostic>, ()> {
        let bytes = length
            .checked_mul(size_of::<BoundaryDiagnostic>())
            .ok_or(())?;
        self.charge(bytes)?;
        self.vector(length)
    }
    fn diagnostic(&mut self, value: BoundaryDiagnostic) -> Result<BoundaryDiagnostic, ()> {
        self.charge(value.heap_bytes())?;
        Ok(value)
    }
}

fn admit_passive_shape(
    value: &BoundaryDiagnostic,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<(), ()> {
    budget.node(depth)?;
    if value.causes.len() > 128 {
        return Err(());
    }
    for child in &value.causes {
        admit_passive_shape(child, depth + 1, budget)?;
    }
    Ok(())
}

fn clone_passive(
    value: &BoundaryDiagnostic,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<BoundaryDiagnostic, ()> {
    admit_passive_shape(value, depth, budget)?;
    budget.charge(value.heap_bytes())?;
    Ok(value.clone())
}

fn capture_worker_diagnostic(
    error: &WorkerError,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<BoundaryDiagnostic, ()> {
    budget.node(depth)?;
    match error {
        WorkerError::Provider(error) => capture_provider_diagnostic(error, depth, budget),
        WorkerError::Lifecycle {
            action,
            handle,
            cause,
        } => {
            if action.len() > budget.text {
                return Err(());
            }
            // Lifecycle owns these facts. The cause owns its facts and rendering.
            let mut facts = DiagnosticFacts::default();
            facts.observe("action", DiagnosticObservation::Text((*action).into()));
            if let Some(handle) = handle {
                facts.observe(
                    "generation",
                    DiagnosticObservation::Text(handle.generation.to_string()),
                );
            }
            let mut value = budget.diagnostic(project_facts(
                Some(DiagnosticCode::RuntimeInfrastructure),
                facts,
                FAILURE_STAGE,
            ))?;
            value.causes = budget.diagnostic_children(1)?;
            value
                .causes
                .push(capture_worker_diagnostic(cause, depth + 1, budget)?);
            Ok(value)
        }
        WorkerError::Retirement { causes } => {
            if causes.is_empty() || causes.len() > 128 {
                return Err(());
            }
            let mut value = budget.diagnostic(project_facts(
                error.diagnostic_code(),
                error.diagnostic_facts(),
                FAILURE_STAGE,
            ))?;
            value.causes = budget.diagnostic_children(causes.len())?;
            for cause in causes {
                value
                    .causes
                    .push(capture_worker_diagnostic(cause, depth + 1, budget)?);
            }
            Ok(value)
        }
        WorkerError::RemoteDiagnostic(diagnostic) => {
            let mut value = budget.diagnostic(project_facts(
                error.diagnostic_code(),
                error.diagnostic_facts(),
                FAILURE_STAGE,
            ))?;
            value.causes = budget.diagnostic_children(1)?;
            value
                .causes
                .push(clone_passive(diagnostic, depth + 1, budget)?);
            Ok(value)
        }
        WorkerError::Acknowledgement { action, .. } if action.len() > budget.text => Err(()),
        WorkerError::Process { .. }
        | WorkerError::Connection(_)
        | WorkerError::Exchange(_)
        | WorkerError::IoTask(_) => Err(()),
        WorkerError::Acknowledgement { .. }
        | WorkerError::Exit(_)
        | WorkerError::Infrastructure(_)
        | WorkerError::UnknownCompletion(_)
        | WorkerError::RemoteResource(_)
        | WorkerError::IoThread(_)
        | WorkerError::NativePanic(_) => {
            if error.retained_bytes().ok_or(())? > budget.text {
                return Err(());
            }
            budget.diagnostic(error.boundary_diagnostic(FAILURE_STAGE))
        }
    }
}

// Only known leaf displays reach this writer. It rejects the complete observation
// before a rendered field can exceed the remaining allowance.
struct DiagnosticDetail {
    text: String,
    limit: usize,
}
impl std::fmt::Write for DiagnosticDetail {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        if value.len() > self.limit.saturating_sub(self.text.len()) {
            return Err(std::fmt::Error);
        }
        self.text
            .try_reserve_exact(value.len())
            .map_err(|_| std::fmt::Error)?;
        self.text.push_str(value);
        Ok(())
    }
}

fn capture_leaf_diagnostic(
    error: &dyn TypedDiagnostic,
    budget: &mut FailureBudget,
) -> Result<BoundaryDiagnostic, ()> {
    use std::fmt::Write as _;
    let mut value = project_facts(
        error.diagnostic_code(),
        error.diagnostic_facts(),
        FAILURE_STAGE,
    );
    let mut detail = DiagnosticDetail {
        text: String::new(),
        limit: budget.text.checked_sub(value.heap_bytes()).ok_or(())?,
    };
    write!(&mut detail, "{error}").map_err(|_| ())?;
    value
        .observations
        .insert("detail".into(), Observation::Text(detail.text));
    budget.diagnostic(value)
}

fn capture_provider_diagnostic(
    error: &ProviderError,
    depth: usize,
    budget: &mut FailureBudget,
) -> Result<BoundaryDiagnostic, ()> {
    budget.node(depth)?;
    match error {
        ProviderError::Nested { cause, retained } => {
            // A producer's allocation extent says nothing about its projection.
            // Admit only source types with a bounded projection at this boundary.
            if *retained > budget.text {
                return Err(());
            }
            let source = cause.as_error();
            if let Some(error) = source.downcast_ref::<ProviderError>() {
                return capture_provider_diagnostic(error, depth + 1, budget);
            }
            if let Some(error) = source.downcast_ref::<WorkerError>() {
                return capture_worker_diagnostic(error, depth + 1, budget);
            }
            if let Some(value) = source.downcast_ref::<BoundaryDiagnostic>() {
                return clone_passive(value, depth + 1, budget);
            }
            Err(())
        }
        ProviderError::Retirement { failures } => {
            if failures.is_empty() || failures.len() > 64 {
                return Err(());
            }
            // The aggregate's own count is bounded. Keyed wrappers carry their
            // keys and children without formatting the child's error again.
            let mut value = capture_leaf_diagnostic(error, budget)?;
            value.causes = budget.diagnostic_children(failures.len())?;
            for failure in failures {
                budget.node(depth + 1)?;
                let mut keyed = budget.diagnostic(project_facts(
                    failure.diagnostic_code(),
                    failure.diagnostic_facts(),
                    FAILURE_STAGE,
                ))?;
                keyed.causes = budget.diagnostic_children(1)?;
                keyed.causes.push(capture_provider_diagnostic(
                    &failure.cause,
                    depth + 2,
                    budget,
                )?);
                value.causes.push(keyed);
            }
            Ok(value)
        }
        ProviderError::Settlement { operation, cleanup } => {
            let mut value = budget.diagnostic(project_facts(
                error.diagnostic_code(),
                error.diagnostic_facts(),
                FAILURE_STAGE,
            ))?;
            value.causes = budget.diagnostic_children(2)?;
            for cause in [operation, cleanup] {
                value
                    .causes
                    .push(capture_provider_diagnostic(cause, depth + 1, budget)?);
            }
            Ok(value)
        }
        ProviderError::RecordMembership(cause) => {
            let mut value = budget.diagnostic(project_facts(
                error.diagnostic_code(),
                error.diagnostic_facts(),
                FAILURE_STAGE,
            ))?;
            value.causes = budget.diagnostic_children(1)?;
            budget.node(depth + 1)?;
            value.causes.push(capture_leaf_diagnostic(cause, budget)?);
            Ok(value)
        }
        ProviderError::OutsideEnvelope { .. } | ProviderError::DerivativeUnavailable { .. }
            if error.retained_bytes() > budget.text =>
        {
            Err(())
        }
        ProviderError::OutsideEnvelope { .. }
        | ProviderError::DerivativeUnavailable { .. }
        | ProviderError::RetirementPanic(_)
        | ProviderError::Cancelled
        | ProviderError::Deadline
        | ProviderError::Limit(_)
        | ProviderError::Contract(_)
        | ProviderError::LinearizationChanged
        | ProviderError::Trial(_)
        | ProviderError::Singular(_)
        | ProviderError::RegimeCrossing { .. }
        | ProviderError::Terminal(_) => capture_leaf_diagnostic(error, budget),
    }
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
            p::Failure::Diagnostic(Box::new(clone_passive(diagnostic, depth + 1, budget)?))
        }
        WorkerError::RemoteResource(reason) => p::Failure::Resource(budget.text(reason)?),
        WorkerError::UnknownCompletion(reason) => {
            p::Failure::UnknownCompletion(budget.text(reason)?)
        }
        error => p::Failure::Diagnostic(Box::new(capture_worker_diagnostic(
            error,
            depth + 1,
            budget,
        )?)),
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
        error => p::Failure::Diagnostic(Box::new(capture_provider_diagnostic(
            error,
            depth + 1,
            budget,
        )?)),
    })
}
pub(super) fn refusal(value: p::Failure) -> WorkerError {
    if !failure_within_limits(&value) {
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
        demand: (&request.demand).try_into()?,
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
            observe_selection: false,
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
    fn borrowed_lifecycle_40k_leaf_is_not_formatted_twice() {
        let handle = p::Handle {
            attempt: pse_ids::SemanticId::from_bytes([1; 16]),
            session: pse_ids::SemanticId::from_bytes([2; 16]),
            generation: pse_ids::SemanticId::from_bytes([3; 16]),
        };
        let key = pse_ids::ContentHash::from_bytes([4; 32]);
        let original = WorkerError::Lifecycle {
            action: "Stop",
            handle: Some(handle.clone()),
            cause: Box::new(WorkerError::Provider(ProviderError::Retirement {
                failures: vec![pse_kernels::ProviderRetirementFailure {
                    key: pse_kernels::ProviderKey(key),
                    cause: Box::new(ProviderError::Contract("x".repeat(40 << 10))),
                }],
            })),
        };
        assert!(original.retained_bytes().unwrap() < FAILURE_RETAINED_TEXT);
        let p::Failure::Diagnostic(observed) = failure_observation(&original) else {
            panic!("an eligible leaf must survive transfer with its lifecycle context");
        };
        assert_eq!(observed.code, DiagnosticCode::RuntimeInfrastructure);
        assert!(matches!(
            &observed.observations["action"],
            Observation::Text(action) if action == "Stop"
        ));
        assert!(matches!(
            &observed.observations["generation"],
            Observation::Text(generation) if generation == &handle.generation.to_string()
        ));
        let keyed = &observed.causes[0].causes[0];
        assert!(matches!(
            &keyed.observations["provider_key"],
            Observation::Text(provider_key) if provider_key == &key.to_string()
        ));
        assert!(!keyed.observations.contains_key("detail"));
        let leaf = &keyed.causes[0];
        assert_eq!(leaf.code, DiagnosticCode::KernelUnboundParameter);
        assert!(matches!(
            &leaf.observations["detail"],
            Observation::Text(detail) if detail.strip_prefix("provider contract: ")
                .is_some_and(|payload| payload.len() == 40 << 10 && payload.bytes().all(|b| b == b'x'))
        ));
    }

    #[test]
    fn borrowed_opaque_cause_is_refused_before_any_projection_or_render() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        #[derive(Debug)]
        struct Probe(Arc<AtomicUsize>);
        impl std::fmt::Display for Probe {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fetch_add(1, Ordering::SeqCst);
                f.write_str("an opaque renderer with no bounded output contract")
            }
        }
        impl std::error::Error for Probe {}
        impl miette::Diagnostic for Probe {}
        impl TypedDiagnostic for Probe {
            fn diagnostic_code(&self) -> Option<DiagnosticCode> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Some(DiagnosticCode::RuntimeInfrastructure)
            }
            fn diagnostic_facts(&self) -> DiagnosticFacts {
                self.0.fetch_add(1, Ordering::SeqCst);
                DiagnosticFacts::default()
            }
        }
        impl pse_model::diagnostic::DiagnosticProjection for Probe {}

        let calls = Arc::new(AtomicUsize::new(0));
        let original = || {
            WorkerError::Provider(ProviderError::Nested {
                retained: size_of::<Probe>(),
                cause: pse_model::diagnostic::DiagnosticCause::new(Probe(calls.clone())),
            })
        };
        assert!(matches!(
            failure_observation(&original()),
            p::Failure::UnknownCompletion(_)
        ));
        assert!(matches!(
            failure_observation(&WorkerError::Lifecycle {
                action: "Stop",
                handle: None,
                cause: Box::new(original()),
            }),
            p::Failure::UnknownCompletion(_)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn borrowed_failure_refuses_oversize_or_copy_budget_without_consuming_original() {
        let error = WorkerError::Provider(ProviderError::Contract(
            "x".repeat(FAILURE_RETAINED_TEXT + 1),
        ));
        assert!(matches!(
            failure_observation(&error),
            p::Failure::UnknownCompletion(_)
        ));
        assert!(
            matches!(&error, WorkerError::Provider(ProviderError::Contract(detail)) if detail.len() == FAILURE_RETAINED_TEXT + 1)
        );
        let original = WorkerError::Provider(ProviderError::Contract(
            "original owned native cause".into(),
        ));
        let refused = observe_with_budget(
            &original,
            FailureBudget {
                nodes: FAILURE_NODES,
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
        for _ in 0..FAILURE_DEPTH + 1 {
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
