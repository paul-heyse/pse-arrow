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

#[derive(Debug)]
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
    fn solve(
        &self,
        _: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        _: &Arc<AtomicBool>,
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
    let cancel = Arc::new(AtomicBool::new(false));
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
                &cancel,
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
        body,
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
        cancel,
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
        Err(pse_kernels::ProviderError::Limit(_))
    ));
    assert!(!flag.load(Ordering::Acquire));
    flag.store(true, Ordering::Release);
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
        cancelled: &factory.cancel,
        max_result_bytes: 128,
    };
    let request = pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value);
    assert_eq!(
        worker.evaluate(&[1.], &request, &context).unwrap().values,
        vec![1.]
    );
    assert!(allowances.lock().unwrap()[0] <= Duration::from_millis(200));
    std::thread::sleep(
        deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(2),
    );
    assert!(matches!(
        worker.evaluate(&[1.], &request, &context),
        Err(pse_kernels::ProviderError::Limit(_))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(!factory.cancel.load(Ordering::Acquire));
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
        cancelled: &factory.cancel,
        max_result_bytes: 128,
    };
    let request = pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value);
    assert!(matches!(
        worker.evaluate(&[1.], &request, &context),
        Err(pse_kernels::ProviderError::Limit(_))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(allowances.lock().unwrap()[0] <= Duration::from_millis(40));
    assert!(!factory.cancel.load(Ordering::Acquire));
}
