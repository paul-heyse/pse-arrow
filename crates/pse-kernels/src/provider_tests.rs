// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::sync::atomic::{AtomicU8, AtomicUsize};

fn descriptor(n: usize, m: usize) -> ProviderSpec {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let ports = |count| {
        (0..count)
            .map(|i: usize| Port {
                id: pse_ids::named_id(SemanticId::NIL, &i.to_string()),
                quantity,
                unit,
            })
            .collect()
    };
    ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id: SemanticId::NIL,
        revision: ContentHash::from_bytes([1; 32]),
        data: ContentHash::from_bytes([2; 32]),
        inputs: ports(n),
        outputs: ports(m),
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    }
}

#[test]
fn provider_extent_projection_preserves_native_limits_and_refusal_categories() {
    use pse_contract::boundary_projection::ValidateProjection;

    let registry = pse_quantity::standard::standard_registry().unwrap();
    let mut spec = descriptor(MAXIMUM_SCALAR_PORTS, MINIMUM_PROVIDER_OUTPUTS);
    spec.id = pse_ids::named_id(SemanticId::NIL, "extent-provider");
    spec.validate(&registry).unwrap();
    spec.validate_projection().unwrap();

    spec.inputs.push(spec.inputs[0].clone());
    assert!(spec.validate_projection().is_err());
    assert!(
        matches!(spec.validate(&registry), Err(ProviderError::Contract(reason))
        if reason == "provider registration capacity")
    );
    spec.inputs.pop();
    spec.outputs.clear();
    assert!(spec.validate_projection().is_err());
    assert!(
        matches!(spec.validate(&registry), Err(ProviderError::Contract(reason))
        if reason == "empty outputs or duplicate identities")
    );
}

#[derive(Debug)]
struct Controlled {
    mode: Arc<AtomicU8>,
    calls: Arc<AtomicUsize>,
    capabilities: Arc<AtomicUsize>,
    cancel: Arc<AtomicBool>,
}
impl Provider for Controlled {
    fn supports(&self, demand: &ProviderDemand) -> bool {
        self.capabilities.fetch_add(1, Ordering::Relaxed);
        matches!(demand, ProviderDemand::CompleteJets)
    }
    fn fill(
        &mut self,
        _: &[f64],
        _: &PreparedProviderRequest,
        _: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        _: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        output.values[0] = 17.;
        match self.mode.load(Ordering::Relaxed) {
            1 => return Err(ProviderError::Trial("late partial fill refusal".into())),
            2 => return Ok(()),
            _ => {}
        }
        output.values.fill(3.);
        output.jacobian.fill(4.);
        output.hessians.fill(5.);
        if self.mode.load(Ordering::Relaxed) == 3 {
            self.cancel.store(true, Ordering::Release);
        }
        Ok(())
    }
}

#[test]
fn provider_fill_refusal_missing_cells_and_late_cancellation_publish_no_completion() {
    let cancel = Arc::new(AtomicBool::new(false));
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1 << 20,
    };
    let mode = Arc::new(AtomicU8::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let capabilities = Arc::new(AtomicUsize::new(0));
    let mut provider = ProviderBinding::new(
        descriptor(1, 2),
        Box::new(Controlled {
            mode: mode.clone(),
            calls: calls.clone(),
            capabilities: capabilities.clone(),
            cancel: cancel.clone(),
        }),
    );
    let request = provider
        .prepare(ProviderRequest::all(
            provider.spec(),
            DerivativeOrder::Second,
        ))
        .unwrap();
    let mut buffer = ProviderBuffer::new(&request);
    for code in [0, 1, 0, 2, 0, 3, 0] {
        mode.store(code, Ordering::Relaxed);
        cancel.store(false, Ordering::Release);
        let result = buffer.fill(
            &mut provider,
            &[2.],
            &request,
            ProviderRuntime::default(),
            &context,
        );
        match code {
            1 | 2 => assert!(matches!(result, Err(ProviderError::Trial(_)))),
            3 => assert!(matches!(result, Err(ProviderError::Cancelled))),
            _ => {
                let complete = result.unwrap();
                assert_eq!(complete.values, [3., 3.]);
                assert_eq!(complete.jacobian, [4., 4.]);
                assert_eq!(complete.hessians, [5., 5.]);
            }
        }
    }
    assert_eq!(calls.load(Ordering::Relaxed), 7);
    assert_eq!(capabilities.load(Ordering::Relaxed), 1);
}

#[test]
fn prepared_wide_provider_bank_reuses_validated_binding_and_refuses_malformed_demand() {
    let cancel = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let capabilities = Arc::new(AtomicUsize::new(0));
    let mut source = descriptor(256, 128);
    let mut provider = ProviderBinding::new(
        source.clone(),
        Box::new(Controlled {
            mode: Arc::new(AtomicU8::new(0)),
            calls: calls.clone(),
            capabilities: capabilities.clone(),
            cancel: cancel.clone(),
        }),
    );
    source.outputs.clear();
    assert_eq!(
        provider.spec().outputs.len(),
        128,
        "the binding owns its immutable descriptor"
    );
    for request in [
        ProviderRequest {
            outputs: vec![0, 0],
            order: DerivativeOrder::First,
            demand: ProviderDemand::CompleteJets,
        },
        ProviderRequest {
            outputs: vec![128],
            order: DerivativeOrder::First,
            demand: ProviderDemand::CompleteJets,
        },
        ProviderRequest {
            outputs: vec![0],
            order: DerivativeOrder::First,
            demand: ProviderDemand::SelectedColumns {
                columns: vec![1, 1],
            },
        },
        ProviderRequest {
            outputs: vec![0],
            order: DerivativeOrder::First,
            demand: ProviderDemand::JacobianAction { directions: 1 },
        },
    ] {
        assert!(provider.prepare(request).is_err());
    }
    let before = capabilities.load(Ordering::Relaxed);
    let key = provider.key();
    let mut request = PreparedProviderRequest::new(
        Arc::new(provider.spec().clone()),
        ProviderRequest::all(provider.spec(), DerivativeOrder::Value),
    )
    .unwrap();
    let mut bank = ProviderBindings::try_from_iter([(key, provider)]).unwrap();
    let slot = bank.resolve(key).unwrap();
    bank.bind_request(slot, &mut request).unwrap();
    let mut buffer = ProviderBuffer::new(&request);
    let inputs = vec![1.; 256];
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1 << 20,
    };
    for _ in 0..8 {
        assert_eq!(
            bank.fill(
                slot,
                &mut buffer,
                &inputs,
                &request,
                ProviderRuntime::default(),
                &context
            )
            .unwrap()
            .values
            .len(),
            128
        );
    }
    assert_eq!(capabilities.load(Ordering::Relaxed), before + 1);
    assert_eq!(calls.load(Ordering::Relaxed), 8);
    let limited = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1,
    };
    assert!(matches!(
        bank.fill(
            slot,
            &mut buffer,
            &inputs,
            &request,
            ProviderRuntime::default(),
            &limited
        ),
        Err(ProviderError::Limit(_))
    ));
    assert_eq!(
        calls.load(Ordering::Relaxed),
        8,
        "current admission refuses before effects"
    );
}

#[test]
fn public_fill_rechecks_foreign_binding_capability_and_current_runtime_seeds() {
    #[derive(Debug)]
    struct Action;
    impl Provider for Action {
        fn supports(&self, demand: &ProviderDemand) -> bool {
            matches!(demand, ProviderDemand::JacobianAction { directions: 1 })
        }
        fn fill(
            &mut self,
            inputs: &[f64],
            _: &PreparedProviderRequest,
            runtime: ProviderRuntime<'_>,
            output: ProviderOutput<'_>,
            _: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            output.values[0] = inputs[0];
            output.jacobian[0] = runtime.directions[0];
            Ok(())
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1024,
    };
    let spec = descriptor(1, 1);
    let mut action = ProviderBinding::new(spec.clone(), Box::new(Action));
    let prepared = action
        .prepare(ProviderRequest {
            outputs: vec![0],
            order: DerivativeOrder::First,
            demand: ProviderDemand::JacobianAction { directions: 1 },
        })
        .unwrap();
    let mut buffer = ProviderBuffer::new(&prepared);
    for direction in [3., -4.] {
        let runtime = ProviderRuntime {
            directions: &[direction],
            ..Default::default()
        };
        let completed = buffer
            .fill(&mut action, &[2.], &prepared, runtime, &context)
            .unwrap();
        assert_eq!(completed.values, [2.]);
        assert_eq!(completed.jacobian, [direction]);
    }
    assert!(matches!(
        buffer.fill(
            &mut action,
            &[2.],
            &prepared,
            ProviderRuntime::default(),
            &context
        ),
        Err(ProviderError::Contract(_))
    ));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut complete_only = ProviderBinding::new(
        spec,
        Box::new(Controlled {
            mode: Arc::new(AtomicU8::new(0)),
            calls: calls.clone(),
            capabilities: Arc::new(AtomicUsize::new(0)),
            cancel: cancel.clone(),
        }),
    );
    assert!(matches!(
        buffer.fill(
            &mut complete_only,
            &[2.],
            &prepared,
            ProviderRuntime {
                directions: &[3.],
                ..Default::default()
            },
            &context
        ),
        Err(ProviderError::Contract(_))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[test]
fn narrowed_worker_capability_binds_to_admitted_provider_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1024,
    };
    let admitted = descriptor(1, 1);
    let mut executable = admitted.clone();
    executable.derivatives = DerivativeOrder::First;
    executable.smoothness = DerivativeOrder::First;
    let binding = ProviderBinding::new(
        executable,
        Box::new(Controlled {
            mode: Arc::new(AtomicU8::new(0)),
            calls: Arc::new(AtomicUsize::new(0)),
            capabilities: Arc::new(AtomicUsize::new(0)),
            cancel: cancel.clone(),
        }),
    );
    assert_ne!(
        binding.key(),
        admitted.key(),
        "capability ceilings participate in descriptor identity"
    );
    let mut request = PreparedProviderRequest::new(
        Arc::new(admitted.clone()),
        ProviderRequest::all(&admitted, DerivativeOrder::First),
    )
    .unwrap();
    let mut bank = ProviderBindings::try_from_iter([(admitted.key(), binding)]).unwrap();
    let slot = bank.resolve(admitted.key()).unwrap();
    bank.bind_request(slot, &mut request).unwrap();
    let mut buffer = ProviderBuffer::new(&request);
    assert_eq!(
        bank.fill(
            slot,
            &mut buffer,
            &[2.],
            &request,
            ProviderRuntime::default(),
            &context
        )
        .unwrap()
        .jacobian,
        [4.]
    );
    let mut second = PreparedProviderRequest::new(
        Arc::new(admitted.clone()),
        ProviderRequest::all(&admitted, DerivativeOrder::Second),
    )
    .unwrap();
    assert!(matches!(
        bank.bind_request(slot, &mut second),
        Err(ProviderError::Contract(_))
    ));
}

#[test]
fn provider_linearization_rechecks_current_effects_and_refuses_changed_derivative_context() {
    #[derive(Debug)]
    struct Linearized {
        coefficient: Arc<AtomicUsize>,
        calls: Arc<AtomicUsize>,
        capability_checks: Arc<AtomicUsize>,
        completed: Option<ContentHash>,
    }
    impl Provider for Linearized {
        fn supports_linearization(&self) -> bool {
            self.capability_checks.fetch_add(1, Ordering::Relaxed);
            true
        }
        fn completed_linearization(&self) -> Option<ContentHash> {
            self.completed
        }
        fn fill(
            &mut self,
            _: &[f64],
            _: &PreparedProviderRequest,
            _: ProviderRuntime<'_>,
            output: ProviderOutput<'_>,
            _: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let coefficient = self.coefficient.load(Ordering::Relaxed);
            output.values.fill(0.);
            output.jacobian.fill(coefficient as f64);
            output.hessians.fill(0.);
            self.completed = Some(ContentHash::from_bytes([coefficient as u8; 32]));
            Ok(())
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1024,
    };
    let coefficient = Arc::new(AtomicUsize::new(2));
    let calls = Arc::new(AtomicUsize::new(0));
    let capability_checks = Arc::new(AtomicUsize::new(0));
    let fresh = || {
        ProviderBinding::new(
            descriptor(1, 1),
            Box::new(Linearized {
                coefficient: coefficient.clone(),
                calls: calls.clone(),
                capability_checks: capability_checks.clone(),
                completed: None,
            }),
        )
    };
    let mut provider = fresh();
    let request = provider
        .prepare(ProviderRequest::all(
            provider.spec(),
            DerivativeOrder::First,
        ))
        .unwrap();
    let mut buffer = ProviderBuffer::new(&request);
    let token = buffer
        .fill(
            &mut provider,
            &[0.],
            &request,
            ProviderRuntime::default(),
            &context,
        )
        .unwrap()
        .linearization
        .unwrap();
    let runtime = ProviderRuntime {
        expected_linearization: Some(token),
        ..Default::default()
    };
    assert_eq!(
        buffer
            .fill(&mut provider, &[0.], &request, runtime, &context)
            .unwrap()
            .linearization,
        Some(token)
    );
    assert_eq!(
        calls.load(Ordering::Relaxed),
        2,
        "same context still visits current effects"
    );
    coefficient.store(3, Ordering::Relaxed);
    assert!(matches!(
        buffer.fill(&mut provider, &[0.], &request, runtime, &context),
        Err(ProviderError::LinearizationChanged)
    ));
    assert_eq!(
        calls.load(Ordering::Relaxed),
        3,
        "same point and values do not certify derivatives"
    );
    coefficient.store(2, Ordering::Relaxed);
    assert!(matches!(
        buffer.fill(&mut provider, &[1.], &request, runtime, &context),
        Err(ProviderError::LinearizationChanged)
    ));
    let mut other = fresh();
    assert!(matches!(
        buffer.fill(&mut other, &[0.], &request, runtime, &context),
        Err(ProviderError::Contract(_))
    ));
    assert_eq!(
        calls.load(Ordering::Relaxed),
        4,
        "foreign owner refused before effects"
    );
    assert_eq!(
        capability_checks.load(Ordering::Relaxed),
        2,
        "immutable capability checked once per fresh binding, never per fill"
    );
    assert!(ProviderError::LinearizationChanged.recoverable());
}

#[test]
fn provider_demand_preparation_is_distinct_from_numerical_effects() {
    #[derive(Debug)]
    struct PreparedOnly {
        prepared: bool,
        preparations: Arc<AtomicUsize>,
    }
    impl Provider for PreparedOnly {
        fn prepare_demand(&mut self, _: &PreparedProviderRequest) -> Result<(), ProviderError> {
            self.preparations.fetch_add(1, Ordering::Relaxed);
            self.prepared = true;
            Ok(())
        }
        fn fill(
            &mut self,
            _: &[f64],
            _: &PreparedProviderRequest,
            _: ProviderRuntime<'_>,
            output: ProviderOutput<'_>,
            _: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            assert!(self.prepared);
            output.values.fill(1.);
            Ok(())
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let binding = ProviderBinding::new(
        descriptor(1, 1),
        Box::new(PreparedOnly {
            prepared: false,
            preparations: calls.clone(),
        }),
    );
    let key = binding.key();
    let mut request = PreparedProviderRequest::new(
        Arc::new(binding.spec().clone()),
        ProviderRequest::all(binding.spec(), DerivativeOrder::Value),
    )
    .unwrap();
    let mut bank = ProviderBindings::try_from_iter([(key, binding)]).unwrap();
    bank.prepare_request(&request).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let slot = bank.resolve(key).unwrap();
    bank.bind_prepared_request(slot, &mut request).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1024,
    };
    let mut buffer = ProviderBuffer::new(&request);
    for _ in 0..4 {
        assert_eq!(
            bank.fill(
                slot,
                &mut buffer,
                &[0.],
                &request,
                ProviderRuntime::default(),
                &context
            )
            .unwrap()
            .values,
            [1.]
        );
    }
    assert_eq!(
        calls.load(Ordering::Relaxed),
        1,
        "fills and prepared slot binds never optimize"
    );
}

#[derive(Debug)]
struct RetirementProbe {
    ordinal: usize,
    finished: Arc<std::sync::Mutex<Vec<usize>>>,
    dropped: Arc<AtomicUsize>,
    failure: Option<ProviderError>,
}
impl Drop for RetirementProbe {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }
}
impl Provider for RetirementProbe {
    fn finish(mut self: Box<Self>) -> Result<(), ProviderError> {
        self.finished.lock().unwrap().push(self.ordinal);
        self.failure.take().map_or(Ok(()), Err)
    }
    fn fill(
        &mut self,
        _: &[f64],
        _: &PreparedProviderRequest,
        _: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        _: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        output.values.fill(self.ordinal as f64);
        Ok(())
    }
}

fn retirement_cause(error: &ProviderError) -> &ProviderError {
    assert!(!error.recoverable());
    let ProviderError::Retirement { failures } = error else {
        panic!("retirement classification missing")
    };
    assert_eq!(
        failures.len(),
        1,
        "wrapper must preserve the original retirement aggregate"
    );
    failures[0].cause.as_ref()
}

#[test]
fn provider_bank_attributes_scope_wrappers_once_to_the_actual_key() {
    let spec = descriptor(1, 1);
    let key = spec.key();
    let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicUsize::new(0));
    let inner = ProviderBinding::new(
        spec.clone(),
        Box::new(RetirementProbe {
            ordinal: 1,
            finished: finished.clone(),
            dropped: dropped.clone(),
            failure: Some(ProviderError::Contract("original scoped retirement".into())),
        }),
    );
    let scoped = ProviderBinding::new(
        spec,
        Box::new(ScopedProvider {
            inner,
            scope: ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        }),
    );
    let error = ProviderBindings::try_from_iter([(key, scoped)])
        .unwrap()
        .finish()
        .unwrap_err();
    assert!(
        matches!(retirement_cause(&error), ProviderError::Contract(detail)
        if detail == "original scoped retirement")
    );
    let ProviderError::Retirement { failures } = error else {
        panic!("keyed retirement failures required")
    };
    assert_eq!(failures[0].key, key);
    assert_eq!(*finished.lock().unwrap(), [1]);
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
}

#[derive(Debug)]
struct PanickingRetirement(RetirementProbe, bool);
#[derive(Debug)]
struct PanickingPayload;
impl Drop for PanickingPayload {
    fn drop(&mut self) {
        panic!("retirement panic payload destructor");
    }
}
impl Provider for PanickingRetirement {
    fn finish(self: Box<Self>) -> Result<(), ProviderError> {
        self.0.finished.lock().unwrap().push(self.0.ordinal);
        if self.1 {
            std::panic::panic_any(PanickingPayload);
        }
        panic!("original consuming retirement panic");
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        self.0.fill(inputs, request, runtime, output, context)
    }
}

#[test]
fn provider_retirement_panic_preserves_typed_witness_and_never_skips_bank_siblings() {
    use pse_diagnostics::TypedDiagnostic;
    for (duplicate, payload_drop) in [(false, false), (true, false), (false, true), (true, true)] {
        let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicUsize::new(0));
        let bindings = (0..3).map(|ordinal| {
            let probe = RetirementProbe {
                ordinal,
                finished: finished.clone(),
                dropped: dropped.clone(),
                failure: (ordinal == 1)
                    .then(|| ProviderError::Terminal("independent teardown".into())),
            };
            let key = ProviderKey(ContentHash::from_bytes(
                [if duplicate {
                    0
                } else {
                    u8::try_from(ordinal).unwrap()
                }; 32],
            ));
            let provider: Box<dyn Provider> = if ordinal == 0 {
                Box::new(PanickingRetirement(probe, payload_drop))
            } else {
                Box::new(probe)
            };
            (key, ProviderBinding::new(descriptor(1, 1), provider))
        });
        let result = ProviderBindings::try_from_iter(bindings).and_then(ProviderBindings::finish);
        assert_eq!(*finished.lock().unwrap(), [0, 1, 2]);
        assert_eq!(dropped.load(Ordering::Relaxed), 3);
        let error = result.unwrap_err();
        assert!(!error.recoverable());
        let ProviderError::Retirement { failures } = error else {
            panic!("retirement failures missing")
        };
        assert_eq!(failures.len(), 2);
        assert!(
            matches!(failures[0].cause.as_ref(), ProviderError::RetirementPanic(message)
            if (!payload_drop && message == "original consuming retirement panic")
                || (payload_drop && message.contains("panic payload destruction also panicked")))
        );
        assert_eq!(
            failures[0].cause.diagnostic_code(),
            Some(pse_diagnostics::DiagnosticCode::WorkflowPanic)
        );
        assert!(failures[0].cause.retained_bytes() > size_of::<ProviderError>());
        assert!(
            matches!(failures[1].cause.as_ref(), ProviderError::Terminal(message) if message == "independent teardown")
        );
    }
}

#[test]
fn provider_retirement_attempts_all_slots_and_preserves_keyed_typed_failures() {
    use pse_diagnostics::TypedDiagnostic;
    use pse_model::diagnostic::DiagnosticProjection;

    let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicUsize::new(0));
    let keys = [11, 22, 33, 44].map(|n| ProviderKey(ContentHash::from_bytes([n; 32])));
    let original = ProviderError::Trial("original typed failure".into());
    let nested = ProviderError::Nested {
        retained: original.retained_bytes(),
        cause: pse_model::diagnostic::DiagnosticCause::new(original),
    };
    let bank = ProviderBindings::try_from_iter(
        [None, Some(ProviderError::Cancelled), None, Some(nested)]
            .into_iter()
            .enumerate()
            .map(|(ordinal, failure)| {
                (
                    keys[ordinal],
                    ProviderBinding::new(
                        descriptor(1, 1),
                        Box::new(RetirementProbe {
                            ordinal,
                            finished: finished.clone(),
                            dropped: dropped.clone(),
                            failure,
                        }),
                    ),
                )
            }),
    )
    .unwrap();
    let error = bank.finish().unwrap_err();
    assert_eq!(*finished.lock().unwrap(), [0, 1, 2, 3]);
    assert_eq!(dropped.load(Ordering::Relaxed), 4);
    assert!(!error.recoverable());
    assert_eq!(
        error.diagnostic_code(),
        Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure)
    );
    assert_eq!(error.diagnostic_children().unwrap().count(), 2);
    assert_eq!(miette::Diagnostic::related(&error).unwrap().count(), 2);
    let ProviderError::Retirement { failures } = &error else {
        panic!("retirement aggregate missing")
    };
    assert_eq!(
        failures
            .iter()
            .map(|failure| failure.key)
            .collect::<Vec<_>>(),
        [keys[1], keys[3]]
    );
    assert!(matches!(*failures[0].cause, ProviderError::Cancelled));
    let ProviderError::Nested { cause, .. } = failures[1].cause.as_ref() else {
        panic!("typed original missing")
    };
    assert!(
        matches!(cause.as_error().downcast_ref::<ProviderError>(), Some(ProviderError::Trial(message)) if message == "original typed failure")
    );
    assert_eq!(failures[1].diagnostic_children().unwrap().count(), 1);
    assert_eq!(
        error.retained_bytes(),
        size_of::<ProviderError>()
            + failures.capacity() * size_of::<ProviderRetirementFailure>()
            + failures
                .iter()
                .map(|failure| failure.cause.retained_bytes())
                .sum::<usize>()
    );
    assert!(
        failures
            .iter()
            .all(|failure| failure.retained_bytes() > size_of::<ProviderRetirementFailure>())
    );
    let diagnostic = error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
    assert_eq!(
        diagnostic.code,
        pse_diagnostics::DiagnosticCode::RuntimeInfrastructure
    );
}

#[test]
fn provider_retirement_retires_displaced_duplicates_and_preserves_last_lookup_winner() {
    let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicUsize::new(0));
    let key = descriptor(1, 1).key();
    let mut bank = ProviderBindings::try_from_iter((0..3).map(|ordinal| {
        (
            key,
            ProviderBinding::new(
                descriptor(1, 1),
                Box::new(RetirementProbe {
                    ordinal,
                    finished: finished.clone(),
                    dropped: dropped.clone(),
                    failure: None,
                }),
            ),
        )
    }))
    .unwrap();
    assert_eq!(bank.len(), 1);
    assert_eq!(*finished.lock().unwrap(), [0, 1]);
    let slot = bank.resolve(key).unwrap();
    let mut request = PreparedProviderRequest::new(
        Arc::new(descriptor(1, 1)),
        ProviderRequest::all(&descriptor(1, 1), DerivativeOrder::Value),
    )
    .unwrap();
    bank.bind_request(slot, &mut request).unwrap();
    let mut buffer = ProviderBuffer::new(&request);
    let cancel = AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancel,
        max_result_bytes: 1024,
    };
    assert_eq!(
        bank.fill(
            slot,
            &mut buffer,
            &[0.],
            &request,
            ProviderRuntime::default(),
            &context
        )
        .unwrap()
        .values,
        [2.]
    );
    assert_eq!(dropped.load(Ordering::Relaxed), 2);
    bank.finish().unwrap();
    assert_eq!(*finished.lock().unwrap(), [0, 1, 2]);
    assert_eq!(dropped.load(Ordering::Relaxed), 3);
    ProviderBindings::new().finish().unwrap();
}

#[test]
fn provider_retirement_failed_duplicate_consumes_remaining_iterator_and_final_bank() {
    let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicUsize::new(0));
    let constructed = Arc::new(AtomicUsize::new(0));
    let key = ProviderKey(ContentHash::from_bytes([81; 32]));
    let sibling = ProviderKey(ContentHash::from_bytes([82; 32]));
    let error = ProviderBindings::try_from_iter((0..5).map(|ordinal| {
        constructed.fetch_add(1, Ordering::Relaxed);
        let selected = if ordinal == 3 { sibling } else { key };
        (
            selected,
            ProviderBinding::new(
                descriptor(1, 1),
                Box::new(RetirementProbe {
                    ordinal,
                    finished: finished.clone(),
                    dropped: dropped.clone(),
                    failure: matches!(ordinal, 0 | 2 | 3)
                        .then(|| ProviderError::Terminal(format!("retirement {ordinal}"))),
                }),
            ),
        )
    }))
    .unwrap_err();
    assert_eq!(constructed.load(Ordering::Relaxed), 5);
    assert_eq!(*finished.lock().unwrap(), [0, 1, 2, 4, 3]);
    assert_eq!(dropped.load(Ordering::Relaxed), 5);
    assert!(!error.recoverable());
    let ProviderError::Retirement { failures } = error else {
        panic!("retirement aggregate missing")
    };
    assert_eq!(
        failures
            .iter()
            .map(|failure| failure.key)
            .collect::<Vec<_>>(),
        [key, key, sibling]
    );
    for (failure, ordinal) in failures.iter().zip([0, 2, 3]) {
        assert!(
            matches!(failure.cause.as_ref(), ProviderError::Terminal(message) if message == &format!("retirement {ordinal}"))
        );
    }
}

#[test]
fn provider_retirement_owning_wrappers_forward_success_and_failure_after_cancellation() {
    for failing in [false, true] {
        let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicUsize::new(0));
        let inner = ProviderBinding::new(
            descriptor(1, 1),
            Box::new(RetirementProbe {
                ordinal: 7,
                finished: finished.clone(),
                dropped: dropped.clone(),
                failure: failing
                    .then(|| ProviderError::Terminal("physical retirement failure".into())),
            }),
        );
        let cancellation = pse_diagnostics::Cancellation::default();
        cancellation.cancel();
        let scoped = ProviderBinding::new(
            descriptor(1, 1),
            Box::new(ScopedProvider {
                inner,
                scope: ExecutionScope::new(cancellation, Some(Instant::now())),
            }),
        );
        let enveloped = ProviderBinding::new(
            descriptor(1, 1),
            Box::new(Enveloped {
                inner: scoped,
                envelope: Arc::from([(0., 1.)]),
            }),
        );
        let observed = ProviderBinding::new(
            descriptor(1, 1),
            Box::new(ObservedProvider {
                inner: enveloped,
                observation: None,
                observation_operation: None,
            }),
        );
        let result = observed.finish();
        assert_eq!(result.is_err(), failing);
        if failing {
            let error = result.unwrap_err();
            assert!(
                matches!(retirement_cause(&error), ProviderError::Terminal(message) if message == "physical retirement failure")
            );
        }
        assert_eq!(*finished.lock().unwrap(), [7]);
        assert_eq!(dropped.load(Ordering::Relaxed), 1);
    }
}

#[derive(Debug)]
struct RetirementFactory {
    spec: ProviderSpec,
    cancellation: pse_diagnostics::Cancellation,
    cancel_after_create: bool,
    mismatch: bool,
    fail_cleanup: bool,
    finished: Arc<std::sync::Mutex<Vec<usize>>>,
    dropped: Arc<AtomicUsize>,
}
impl ProviderFactory for RetirementFactory {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        let mut spec = self.spec.clone();
        if self.mismatch {
            spec.revision = ContentHash::from_bytes([77; 32]);
        }
        let worker = ProviderBinding::new(
            spec,
            Box::new(RetirementProbe {
                ordinal: 8,
                finished: self.finished.clone(),
                dropped: self.dropped.clone(),
                failure: self
                    .fail_cleanup
                    .then(|| ProviderError::Trial("cleanup original".into())),
            }),
        );
        if self.cancel_after_create {
            self.cancellation.cancel();
        }
        Ok(worker)
    }
}

#[derive(Debug)]
struct CustomScopedRetirementFactory(RetirementFactory);
impl ProviderFactory for CustomScopedRetirementFactory {
    fn spec(&self) -> &ProviderSpec {
        self.0.spec()
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        self.0.create()
    }
    fn create_scoped(&self, _: ExecutionScope) -> Result<ProviderBinding, ProviderError> {
        self.create()
    }
}

#[derive(Debug)]
struct PanickingRetirementFactory(RetirementFactory);
impl ProviderFactory for PanickingRetirementFactory {
    fn spec(&self) -> &ProviderSpec {
        self.0.spec()
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        let probe = RetirementProbe {
            ordinal: 9,
            finished: self.0.finished.clone(),
            dropped: self.0.dropped.clone(),
            failure: None,
        };
        self.0.cancellation.cancel();
        Ok(ProviderBinding::new(
            self.0.spec.clone(),
            Box::new(PanickingRetirement(probe, false)),
        ))
    }
}

#[test]
fn provider_retirement_factory_partial_cleanup_preserves_operation_and_panic() {
    let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicUsize::new(0));
    let cancellation = pse_diagnostics::Cancellation::default();
    let factory = PanickingRetirementFactory(RetirementFactory {
        spec: descriptor(1, 1),
        cancellation: cancellation.clone(),
        cancel_after_create: true,
        mismatch: false,
        fail_cleanup: false,
        finished: finished.clone(),
        dropped: dropped.clone(),
    });
    let error = factory
        .create_scoped(ExecutionScope::new(cancellation, None))
        .unwrap_err();
    let ProviderError::Settlement { operation, cleanup } = error else {
        panic!("operation or panic missing")
    };
    assert!(matches!(*operation, ProviderError::Cancelled));
    assert!(
        matches!(retirement_cause(&cleanup), ProviderError::RetirementPanic(message) if message == "original consuming retirement panic")
    );
    assert_eq!(*finished.lock().unwrap(), [9]);
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
}

#[test]
fn provider_retirement_failed_construction_preserves_operation_and_cleanup() {
    use pse_diagnostics::TypedDiagnostic;
    for cancelled in [false, true] {
        for fail_cleanup in [false, true] {
            let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
            let dropped = Arc::new(AtomicUsize::new(0));
            let cancellation = pse_diagnostics::Cancellation::default();
            let mut spec = descriptor(1, 1);
            spec.id = pse_ids::named_id(SemanticId::NIL, "retirement-construction");
            let registry = pse_quantity::standard::standard_registry().unwrap();
            let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
            let registration = Registration::bind(
                admitted,
                Arc::new(CustomScopedRetirementFactory(RetirementFactory {
                    spec,
                    cancellation: cancellation.clone(),
                    cancel_after_create: cancelled,
                    mismatch: !cancelled,
                    fail_cleanup,
                    finished: finished.clone(),
                    dropped: dropped.clone(),
                })),
            )
            .unwrap();
            let error = registration
                .worker_scoped(ExecutionScope::new(cancellation, None))
                .unwrap_err();
            let operation = if fail_cleanup {
                assert!(!error.recoverable());
                assert_eq!(error.diagnostic_children().unwrap().count(), 2);
                let ProviderError::Settlement { operation, cleanup } = &error else {
                    panic!("cleanup failure lost")
                };
                assert!(
                    matches!(retirement_cause(cleanup), ProviderError::Trial(message) if message == "cleanup original")
                );
                assert_eq!(
                    error.retained_bytes(),
                    size_of::<ProviderError>()
                        + operation.retained_bytes()
                        + cleanup.retained_bytes()
                );
                operation.as_ref()
            } else {
                &error
            };
            if cancelled {
                assert!(matches!(operation, ProviderError::Cancelled));
            } else {
                assert!(
                    matches!(operation, ProviderError::Contract(message) if message == "factory returned a different provider contract")
                );
            }
            assert_eq!(*finished.lock().unwrap(), [8]);
            assert_eq!(dropped.load(Ordering::Relaxed), 1);
        }
    }
}

#[test]
fn provider_retirement_default_scoped_factory_settles_post_construction_cancellation() {
    for fail_cleanup in [false, true] {
        let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicUsize::new(0));
        let cancellation = pse_diagnostics::Cancellation::default();
        let factory = RetirementFactory {
            spec: descriptor(1, 1),
            cancellation: cancellation.clone(),
            cancel_after_create: true,
            mismatch: false,
            fail_cleanup,
            finished: finished.clone(),
            dropped: dropped.clone(),
        };
        let error = factory
            .create_scoped(ExecutionScope::new(cancellation, None))
            .unwrap_err();
        if fail_cleanup {
            let ProviderError::Settlement { operation, cleanup } = error else {
                panic!("cleanup failure missing")
            };
            assert!(matches!(*operation, ProviderError::Cancelled));
            assert!(matches!(
                retirement_cause(&cleanup),
                ProviderError::Trial(_)
            ));
        } else {
            assert!(matches!(error, ProviderError::Cancelled));
        }
        assert_eq!(*finished.lock().unwrap(), [8]);
        assert_eq!(dropped.load(Ordering::Relaxed), 1);
    }
}

#[test]
fn provider_retirement_default_finishes_synchronous_in_process_drop() {
    #[derive(Debug)]
    struct InProcess(Arc<AtomicUsize>);
    impl Drop for InProcess {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    impl Provider for InProcess {
        fn fill(
            &mut self,
            _: &[f64],
            _: &PreparedProviderRequest,
            _: ProviderRuntime<'_>,
            _: ProviderOutput<'_>,
            _: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            Ok(())
        }
    }
    let dropped = Arc::new(AtomicUsize::new(0));
    let provider: Box<dyn Provider> = Box::new(InProcess(dropped.clone()));
    provider.finish().unwrap();
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
}

#[test]
fn provider_retirement_registration_retires_validation_worker_and_reports_cleanup() {
    for fail_cleanup in [false, true] {
        let finished = Arc::new(std::sync::Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicUsize::new(0));
        let mut spec = descriptor(1, 1);
        spec.id = pse_ids::named_id(SemanticId::NIL, "retirement-validation");
        let result = Registration::new(
            Arc::new(RetirementFactory {
                spec,
                cancellation: pse_diagnostics::Cancellation::default(),
                cancel_after_create: false,
                mismatch: false,
                fail_cleanup,
                finished: finished.clone(),
                dropped: dropped.clone(),
            }),
            &pse_quantity::standard::standard_registry().unwrap(),
        );
        assert_eq!(result.is_err(), fail_cleanup);
        if fail_cleanup {
            let error = result.unwrap_err();
            assert!(
                matches!(retirement_cause(&error), ProviderError::Trial(message) if message == "cleanup original")
            );
        }
        assert_eq!(*finished.lock().unwrap(), [8]);
        assert_eq!(dropped.load(Ordering::Relaxed), 1);
    }
}
