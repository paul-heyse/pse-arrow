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
    let mut bank = ProviderBindings::from([(key, provider)]);
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
    let mut bank = ProviderBindings::from([(admitted.key(), binding)]);
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
    let mut bank = ProviderBindings::from([(key, binding)]);
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
