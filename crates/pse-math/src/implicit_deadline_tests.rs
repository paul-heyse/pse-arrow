// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Outer clocks bound independently finite inner numerical allowances.
#![allow(
    clippy::unwrap_used,
    reason = "test fixtures panic when an asserted setup contract fails"
)]
use super::*;
use pse_kernels::{ExecutionScope, ProviderFactory};
use std::{sync::atomic::AtomicUsize, time::Instant};

#[derive(Debug, Clone)]
struct Root {
    calls: Arc<AtomicUsize>,
    allowances: Arc<Mutex<Vec<Duration>>>,
    delay: Duration,
}
impl InnerSolver for Root {
    fn identity(&self) -> ContentHash {
        solver_identity("test.deadline-root.v1")
    }
    fn minimum_order(&self) -> DerivativeOrder {
        DerivativeOrder::First
    }

    fn worker(&self) -> Box<dyn InnerWorker> {
        Box::new(self.clone())
    }
}
impl InnerWorker for Root {
    fn solve(
        &mut self,
        _: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        _: &pse_diagnostics::Cancellation,
    ) -> Result<Vec<f64>, MathError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.allowances.lock().unwrap().push(options.time_limit);
        std::thread::sleep(self.delay);
        Ok(vec![parameters[0]])
    }
}
fn factory(delay: Duration) -> (Factory, Arc<AtomicUsize>, Arc<Mutex<Vec<Duration>>>) {
    use crate::typed::{Binary, BodyBuilder, BodyLimits};
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let id = pse_ids::named_id(SemanticId::NIL, "deadline-root");
    let unknown = pse_ids::named_id(id, "unknown");
    let control = pse_diagnostics::Cancellation::new(false);
    let mut builder = BodyBuilder::new(
        crate::initialize().unwrap(),
        &registry,
        &pse_quantity::standard::StandardInvariantChecker,
        2,
        BodyLimits::default(),
    )
    .unwrap();
    let y = builder
        .input(0, quantity, pse_quantity::IndexSet::new(), id)
        .unwrap();
    let p = builder
        .input(1, quantity, pse_quantity::IndexSet::new(), id)
        .unwrap();
    let residual = builder.binary(Binary::Sub, y, p, None, id).unwrap();
    let body = Arc::new(
        builder
            .finish(
                &[residual],
                DerivativeOrder::First,
                crate::library::Optimization::default(),
                &ExecutionScope::new(control.clone(), None),
            )
            .unwrap(),
    );
    let bounds = vec![Unknown {
        id: unknown,
        lower: -10.,
        upper: 10.,
    }];
    let port = |id| pse_kernels::Port { id, quantity, unit };
    let calls = Arc::new(AtomicUsize::new(0));
    let allowances = Arc::new(Mutex::new(Vec::new()));
    let factory = Factory {
        selection: Selection::default(),
        requirements: pse_kernels::DerivativeRequirements::new(
            DerivativeOrder::First,
            DerivativeOrder::First,
            DerivativeOrder::First,
            DerivativeOrder::First,
            DerivativeOrder::Value,
        )
        .unwrap(),
        spec: pse_kernels::ProviderSpec {
            shapes: Default::default(),
            derivative_source: pse_kernels::DerivativeSource::Analytic,
            id,
            revision: ContentHash::from_bytes([1; 32]),
            data: ContentHash::from_bytes([2; 32]),
            inputs: vec![port(pse_ids::named_id(id, "parameter"))],
            outputs: vec![port(unknown)],
            derivatives: DerivativeOrder::Value,
            smoothness: DerivativeOrder::Value,
        },
        bodies: residual_bodies_for_test(body),
        unknowns: bounds.clone(),
        rows: vec![pse_ids::named_id(id, "row")],
        configuration: Configuration::Fixed(
            bounds,
            Options {
                start: vec![1.],
                variable_nominals: vec![1.],
                variable_tolerance: vec![1e-9],
                residual_tolerance: vec![1e-9],
                iterations: 10,
                time_limit: Duration::from_secs(10),
                derivative_tolerance: 1e-10,
            },
        ),
        hints: None,
        terms: None,
        solver: Arc::new(Root {
            calls: calls.clone(),
            allowances: allowances.clone(),
            delay,
        }),
        cancel: control.clone(),
        max_entries: 100,
        providers: BTreeMap::new(),
    };
    (factory, calls, allowances)
}
#[test]
fn implicit_outer_deadline_expiry_avoids_native_and_remains_distinct_from_cancel() {
    let (factory, calls, _) = factory(Duration::ZERO);
    let flag = factory.cancel.clone();
    let scope = ExecutionScope::new(flag.clone(), Some(Instant::now()));
    assert!(matches!(
        factory.create_scoped(scope.clone()),
        Err(pse_kernels::ProviderError::Deadline)
    ));
    assert!(!flag.is_cancelled());
    flag.cancel();
    assert!(matches!(
        factory.create_scoped(scope),
        Err(pse_kernels::ProviderError::Cancelled)
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
#[test]
fn implicit_outer_deadline_caps_local_allowance_and_cannot_be_renewed_by_evaluation() {
    let (factory, calls, allowances) = factory(Duration::ZERO);
    let deadline = Instant::now() + Duration::from_millis(200);
    let scope = ExecutionScope::new(factory.cancel.clone(), Some(deadline));
    let mut worker = factory.create_scoped(scope).unwrap();
    let context = pse_kernels::EvaluationContext {
        cancelled: &factory.cancel.flag(),
        max_result_bytes: 128,
    };
    let request = pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value);
    assert_eq!(
        provider_values(&mut worker, &[1.], &request, &context)
            .unwrap()
            .values,
        vec![1.]
    );
    assert!(allowances.lock().unwrap()[0] <= Duration::from_millis(200));
    std::thread::sleep(
        deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(2),
    );
    assert!(matches!(
        provider_values(&mut worker, &[1.], &request, &context),
        Err(pse_kernels::ProviderError::Deadline)
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(!factory.cancel.is_cancelled());
}
#[test]
fn implicit_outer_deadline_rejects_late_positive_native_result() {
    let (factory, calls, allowances) = factory(Duration::from_millis(60));
    let scope = ExecutionScope::new(
        factory.cancel.clone(),
        Some(Instant::now() + Duration::from_millis(40)),
    );
    let mut worker = factory.create_scoped(scope).unwrap();
    let context = pse_kernels::EvaluationContext {
        cancelled: &factory.cancel.flag(),
        max_result_bytes: 128,
    };
    let request = pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value);
    assert!(matches!(
        provider_values(&mut worker, &[1.], &request, &context),
        Err(pse_kernels::ProviderError::Deadline)
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(allowances.lock().unwrap()[0] <= Duration::from_millis(40));
    assert!(!factory.cancel.is_cancelled());
}

#[test]
fn observation_distinguishes_actual_inner_attempt_from_reused_product_and_original_assessment() {
    use pse_diagnostics::observation::{ObservationHub, Outcome, Policy, Value};
    for enabled in [false, true] {
        let hub = ObservationHub::new(Policy {
            enabled,
            ..Policy::default()
        });
        let operation = hub.start("implicit.observation.control");
        let observation = operation.context();
        let _entered = observation.enter();
        let (factory, calls, _) = factory(Duration::ZERO);
        let mut worker = factory
            .create_scoped(ExecutionScope::new(factory.cancel.clone(), None))
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        let context = pse_kernels::EvaluationContext {
            cancelled: &factory.cancel.flag(),
            max_result_bytes: 128,
        };
        let request = pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value);
        for _ in 0..2 {
            assert_eq!(
                provider_values(&mut worker, &[1.], &request, &context)
                    .unwrap()
                    .values,
                vec![1.]
            );
        }
        let snapshot = observation.reader().snapshot(Duration::from_secs(60));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        if enabled {
            let metric = |name| {
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .unwrap()
            };
            assert_eq!(
                metric("implicit.provider.calls").cumulative,
                Value::Count(2)
            );
            assert_eq!(
                metric("implicit.product.full_reuse").cumulative,
                Value::Count(1)
            );
            assert_eq!(
                metric("implicit.original_assessment.calls").cumulative,
                Value::Count(1)
            );
            let child = snapshot
                .children
                .iter()
                .find(|child| child.kind == "implicit.inner_attempt")
                .unwrap();
            assert_eq!(child.outcome, Some(Outcome::Completed));
            let inner = hub
                .reader(child.id)
                .unwrap()
                .snapshot(Duration::from_secs(60));
            assert_eq!(
                inner
                    .metrics
                    .iter()
                    .find(|metric| metric.name == "implicit.inner_solve.calls")
                    .unwrap()
                    .cumulative,
                Value::Count(1)
            );
            assert!(!snapshot.coverage.curtailed);
        } else {
            assert!(!snapshot.coverage.enabled);
            assert!(snapshot.metrics.is_empty());
        }
        assert_eq!(
            calls.load(Ordering::Relaxed),
            1,
            "reading never asks the inner solver for another point"
        );
    }
}
