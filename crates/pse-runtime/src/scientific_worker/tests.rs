// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{
    DerivativeOrder, DerivativeSource, EvaluationContext, ExecutionScope, Port,
    PreparedProviderRequest, Provider, ProviderBinding, ProviderDemand, ProviderFactory,
    ProviderOutput, ProviderRequest, ProviderRuntime, ProviderShapes, ProviderSpec,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
struct Factory {
    spec: ProviderSpec,
    calls: Arc<AtomicUsize>,
    linearization: bool,
    finishes: Arc<AtomicUsize>,
    fail_finish: bool,
    panic_finish: bool,
    prepare_trial: bool,
}
#[derive(Debug)]
struct Native {
    calls: Arc<AtomicUsize>,
    linearization: bool,
    finishes: Arc<AtomicUsize>,
    fail_finish: bool,
    panic_finish: bool,
    prepare_trial: bool,
}
impl Provider for Native {
    fn prepare_demand(&mut self, _: &PreparedProviderRequest) -> Result<(), ProviderError> {
        if self.prepare_trial {
            Err(ProviderError::Trial("controlled preparation trial".into()))
        } else {
            Ok(())
        }
    }
    fn finish(self: Box<Self>) -> Result<(), ProviderError> {
        self.finishes.fetch_add(1, Ordering::SeqCst);
        assert!(!self.panic_finish, "controlled native finish panic");
        if self.fail_finish {
            Err(ProviderError::Contract("controlled finish failure".into()))
        } else {
            Ok(())
        }
    }
    fn supports_linearization(&self) -> bool {
        self.linearization
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        _: &PreparedProviderRequest,
        _: ProviderRuntime<'_>,
        out: ProviderOutput<'_>,
        _: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if inputs[0] < 0.0 {
            out.values[0] = 999.0;
            return Err(ProviderError::Trial("controlled partial refusal".into()));
        }
        out.values.fill(inputs[0] * inputs[0]);
        out.jacobian.fill(2.0 * inputs[0]);
        out.hessians.fill(2.0);
        Ok(())
    }
}
impl ProviderFactory for Factory {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        Ok(ProviderBinding::new(
            self.spec.clone(),
            Box::new(Native {
                calls: self.calls.clone(),
                linearization: self.linearization,
                finishes: self.finishes.clone(),
                fail_finish: self.fail_finish,
                panic_finish: self.panic_finish,
                prepare_trial: self.prepare_trial,
            }),
        ))
    }
}
fn factory() -> Factory {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let port = Port {
        id: SemanticId::NIL,
        quantity,
        unit: registry.quantity_type(quantity).unwrap().canonical_unit,
    };
    Factory {
        spec: ProviderSpec {
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            id: SemanticId::NIL,
            revision: ContentHash::from_bytes([1; 32]),
            data: ContentHash::from_bytes([2; 32]),
            inputs: vec![port.clone()],
            outputs: vec![port],
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        },
        calls: Arc::new(AtomicUsize::new(0)),
        linearization: false,
        finishes: Arc::new(AtomicUsize::new(0)),
        fail_finish: false,
        panic_finish: false,
        prepare_trial: false,
    }
}

#[test]
fn scalar_support_never_advertises_or_synthesizes_a_native_group() {
    let factory = factory();
    let mut binding = factory.create().unwrap();
    let request = binding
        .prepare(ProviderRequest::all(binding.spec(), DerivativeOrder::Value))
        .unwrap();
    assert!(!binding.supports_batch(&request));
    let point = pse_kernels::ProviderBatchPoint::values(SemanticId::from_bytes([9; 16]), vec![2.0]);
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    assert!(
        binding
            .evaluate_batch(
                &[point],
                &request,
                &EvaluationContext {
                    cancelled: &cancelled,
                    max_result_bytes: 4096,
                }
            )
            .is_err()
    );
    assert_eq!(factory.calls.load(Ordering::SeqCst), 0);
}
fn handle() -> SessionHandle {
    SessionHandle {
        attempt: SemanticId::from_bytes([1; 16]),
        session: SemanticId::from_bytes([2; 16]),
        generation: SemanticId::from_bytes([3; 16]),
    }
}
fn scope() -> ExecutionScope {
    ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)
}

fn prepare_session(
    handle: SessionHandle,
    factory: &Factory,
    expected: &ProviderSpec,
    configuration: ContentHash,
    request: ProviderRequest,
    scope: ExecutionScope,
    result_limit: usize,
) -> Result<PreparedSession, WorkerError> {
    let checked = PreparedProviderRequest::new(Arc::new(expected.clone()), request)?;
    PreparedSession::prepare(handle, factory, configuration, checked, scope, result_limit)
}

#[test]
fn failed_native_preparation_preserves_original_and_finish_causes() {
    let mut factory = factory();
    factory.linearization = true;
    factory.fail_finish = true;
    let error = prepare_session(
        handle(),
        &factory,
        factory.spec(),
        factory.configuration_key(),
        ProviderRequest::all(factory.spec(), DerivativeOrder::Value),
        scope(),
        4096,
    )
    .unwrap_err();
    let WorkerError::Retirement { causes } = error else {
        panic!("preparation and finish failures must survive");
    };
    assert_eq!(causes.len(), 2);
    assert!(
        matches!(&causes[0], WorkerError::Provider(ProviderError::Contract(detail)) if detail == "scientific worker has no admitted transferable linearization context")
    );
    let WorkerError::Provider(ProviderError::Retirement { failures }) = &causes[1] else {
        panic!("original keyed finish failure required");
    };
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].key, factory.spec.key());
    assert!(
        matches!(failures[0].cause.as_ref(), ProviderError::Contract(detail) if detail == "controlled finish failure")
    );
    assert_eq!(factory.finishes.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_session_close_is_physically_finished_once() {
    let mut factory = factory();
    factory.fail_finish = true;
    let mut session = prepare_session(
        handle(),
        &factory,
        factory.spec(),
        factory.configuration_key(),
        ProviderRequest::all(factory.spec(), DerivativeOrder::Value),
        scope(),
        4096,
    )
    .unwrap();
    let WorkerError::Provider(ProviderError::Retirement { failures }) =
        session.close().unwrap_err()
    else {
        panic!("original keyed finish failure required");
    };
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].key, factory.spec.key());
    assert!(
        matches!(failures[0].cause.as_ref(), ProviderError::Contract(detail) if detail == "controlled finish failure")
    );
    assert_eq!(session.state(), SessionState::Closed);
    session.close().unwrap();
    drop(session);
    assert_eq!(factory.finishes.load(Ordering::SeqCst), 1);
}

#[test]
fn native_finish_panic_is_typed_and_does_not_escape_or_retry_the_finished_owner() {
    let mut factory = factory();
    factory.panic_finish = true;
    let mut session = prepare_session(
        handle(),
        &factory,
        factory.spec(),
        factory.configuration_key(),
        ProviderRequest::all(factory.spec(), DerivativeOrder::Value),
        scope(),
        4096,
    )
    .unwrap();
    let WorkerError::Provider(ProviderError::Retirement { failures }) = session.stop().unwrap_err()
    else {
        panic!("original keyed panic failure required");
    };
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].key, factory.spec.key());
    assert!(
        matches!(failures[0].cause.as_ref(), ProviderError::RetirementPanic(detail) if detail == "controlled native finish panic")
    );
    session.close().unwrap();
    drop(session);
    assert_eq!(factory.finishes.load(Ordering::SeqCst), 1);
}

#[test]
fn recoverable_preparation_trial_with_failed_finish_has_nonrecoverable_aggregate() {
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection as _};
    let mut factory = factory();
    factory.prepare_trial = true;
    factory.fail_finish = true;
    let error = prepare_session(
        handle(),
        &factory,
        factory.spec(),
        factory.configuration_key(),
        ProviderRequest::all(factory.spec(), DerivativeOrder::Value),
        scope(),
        4096,
    )
    .unwrap_err();
    let WorkerError::Retirement { causes } = &error else {
        panic!("original trial and independent cleanup required");
    };
    assert_eq!(causes.len(), 2);
    assert!(
        matches!(&causes[0], WorkerError::Provider(ProviderError::Trial(detail)) if detail == "controlled preparation trial")
    );
    let WorkerError::Provider(ProviderError::Retirement { failures }) = &causes[1] else {
        panic!("keyed cleanup required");
    };
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].key, factory.spec.key());
    let diagnostic = error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
    assert_eq!(diagnostic.class, BoundaryClass::Infrastructure);
    assert_eq!(diagnostic.causes.len(), 2);
    assert_eq!(diagnostic.causes[0].class, BoundaryClass::TrialRejected);
    assert!(matches!(
        &diagnostic.causes[1].causes[0].observations["provider_key"],
        pse_model::diagnostic::Observation::Text(key) if *key == factory.spec.key().0.to_string()
    ));
    #[cfg(feature = "property-coolprop")]
    assert!(!error.into_provider().recoverable());
}

#[test]
fn session_preserves_coherent_jets_and_refuses_stale_or_replayed_invocations() {
    let factory = factory();
    let mut session = prepare_session(
        handle(),
        &factory,
        &factory.spec,
        factory.configuration_key(),
        ProviderRequest::all(&factory.spec, DerivativeOrder::Second),
        scope(),
        4096,
    )
    .unwrap();
    for invocation in 1..=3 {
        let result = session
            .evaluate(handle(), invocation, &[3.0], ProviderRuntime::default())
            .unwrap();
        assert_eq!(result.values, [9.0]);
        assert_eq!(result.jacobian, [6.0]);
        assert_eq!(result.hessians, [2.0]);
    }
    assert!(
        session
            .evaluate(handle(), 3, &[3.0], ProviderRuntime::default())
            .is_err()
    );
    let mut stale = handle();
    stale.generation = SemanticId::NIL;
    assert!(
        session
            .evaluate(stale, 4, &[3.0], ProviderRuntime::default())
            .is_err()
    );
    assert_eq!(factory.calls.load(Ordering::SeqCst), 3);
    session.close().unwrap();
    assert_eq!(session.state(), SessionState::Closed);
    assert!(
        session
            .evaluate(handle(), 4, &[3.0], ProviderRuntime::default())
            .is_err()
    );
}

#[test]
fn stopping_one_prepared_handle_preserves_an_independent_session() {
    let factory = factory();
    let first_scope = scope();
    let second_scope = scope();
    let mut second_handle = handle();
    second_handle.generation = SemanticId::from_bytes([4; 16]);
    let prepare = |handle, scope| {
        prepare_session(
            handle,
            &factory,
            &factory.spec,
            factory.configuration_key(),
            ProviderRequest::all(&factory.spec, DerivativeOrder::Second),
            scope,
            4096,
        )
        .unwrap()
    };
    let mut first = prepare(handle(), first_scope.clone());
    let mut second = prepare(second_handle, second_scope.clone());
    first.stop().unwrap();
    assert_eq!(first.state(), SessionState::Closed);
    assert!(first_scope.check().is_err());
    assert!(second_scope.check().is_ok());
    assert_eq!(second.state(), SessionState::Ready);
    assert_eq!(
        second
            .evaluate(second_handle, 1, &[3.0], ProviderRuntime::default())
            .unwrap()
            .values,
        [9.0]
    );
    // Ordinary retirement destroys this handle without manufacturing a cancellation cause.
    second.close().unwrap();
    assert!(second_scope.check().is_ok());
    assert_eq!(second.state(), SessionState::Closed);
}

#[test]
fn session_partial_trial_refusal_does_not_publish_old_or_partial_completion() {
    let factory = factory();
    let mut session = prepare_session(
        handle(),
        &factory,
        &factory.spec,
        factory.configuration_key(),
        ProviderRequest::all(&factory.spec, DerivativeOrder::Second),
        scope(),
        4096,
    )
    .unwrap();
    assert_eq!(
        session
            .evaluate(handle(), 1, &[3.0], ProviderRuntime::default())
            .unwrap()
            .values,
        [9.0]
    );
    assert!(matches!(
        session.evaluate(handle(), 2, &[-1.0], ProviderRuntime::default()),
        Err(WorkerError::Provider(ProviderError::Trial(_)))
    ));
    assert_eq!(session.state(), SessionState::Ready);
    assert_eq!(
        session
            .evaluate(handle(), 3, &[4.0], ProviderRuntime::default())
            .unwrap()
            .values,
        [16.0]
    );
}

#[test]
fn session_admission_checks_actual_configuration_products_and_original_deadline() {
    let factory = factory();
    let request = ProviderRequest::all(&factory.spec, DerivativeOrder::Second);
    assert!(
        prepare_session(
            handle(),
            &factory,
            &factory.spec,
            ContentHash::from_bytes([7; 32]),
            request.clone(),
            scope(),
            4096
        )
        .is_err()
    );
    let mut action = request.clone();
    action.order = DerivativeOrder::First;
    action.demand = ProviderDemand::JacobianAction { directions: 1 };
    assert!(
        prepare_session(
            handle(),
            &factory,
            &factory.spec,
            factory.configuration_key(),
            action,
            scope(),
            4096
        )
        .is_err()
    );
    assert!(matches!(
        prepare_session(
            handle(),
            &factory,
            &factory.spec,
            factory.configuration_key(),
            request.clone(),
            ExecutionScope::new(
                pse_diagnostics::Cancellation::default(),
                Some(std::time::Instant::now())
            ),
            4096
        ),
        Err(WorkerError::Provider(ProviderError::Deadline))
    ));
    assert!(
        prepare_session(
            handle(),
            &factory,
            &factory.spec,
            factory.configuration_key(),
            request,
            scope(),
            8
        )
        .is_err()
    );
    assert_eq!(factory.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn session_refuses_changed_data_physical_ports_and_unadmitted_linearization() {
    let mut factory = factory();
    let request = ProviderRequest::all(&factory.spec, DerivativeOrder::Second);
    let mut changed = factory.spec.clone();
    changed.data = ContentHash::from_bytes([8; 32]);
    assert!(matches!(
        prepare_session(
            handle(),
            &factory,
            &changed,
            factory.configuration_key(),
            request.clone(),
            scope(),
            4096
        ),
        Err(WorkerError::Provider(ProviderError::Contract(_)))
    ));
    let mut changed = factory.spec.clone();
    changed.inputs[0].unit = pse_quantity::UnitId::from_id(SemanticId::NIL);
    assert!(matches!(
        prepare_session(
            handle(),
            &factory,
            &changed,
            factory.configuration_key(),
            request.clone(),
            scope(),
            4096
        ),
        Err(WorkerError::Provider(ProviderError::Contract(_)))
    ));
    factory.linearization = true;
    assert!(matches!(
        prepare_session(
            handle(),
            &factory,
            &factory.spec,
            factory.configuration_key(),
            request,
            scope(),
            4096
        ),
        Err(WorkerError::Provider(ProviderError::Contract(_)))
    ));
    assert_eq!(factory.calls.load(Ordering::SeqCst), 0);
}
