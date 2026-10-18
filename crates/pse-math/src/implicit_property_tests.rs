// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Production Peng–Robinson compressibility roots, with independent scalar IFT equations.
use super::*;
use crate::guarded::{PreparedBody, Stage};
use crate::jets::{EvaluationLimits, ProductKind, ProductValues};
use crate::library::Optimization;
use pse_kernels::{
    EvaluationContext, ExecutionScope, ProviderFactory, ProviderRequest, ProviderSpec,
};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use symbolica::atom::{Atom, AtomCore};

#[path = "implicit_case_effect_tests.rs"]
mod case_effects;

#[derive(Debug, Default)]
struct Work {
    solves: AtomicUsize,
    workers: AtomicUsize,
}
#[derive(Debug)]
struct ObservedCubic {
    actual: CubicRoots,
    work: Arc<Work>,
}
impl InnerSolver for ObservedCubic {
    fn minimum_order(&self) -> DerivativeOrder {
        self.actual.minimum_order()
    }
    fn identity(&self) -> ContentHash {
        self.actual.identity()
    }
    fn worker(&self) -> Box<dyn InnerWorker> {
        self.work.workers.fetch_add(1, Ordering::Relaxed);
        Box::new(ObservedCubicWorker {
            actual: self.actual.worker(),
            work: self.work.clone(),
        })
    }
    fn retained_bytes(&self) -> usize {
        self.actual.retained_bytes() + size_of::<Self>() + size_of::<Work>()
    }
    fn worker_bytes(&self) -> usize {
        self.actual.worker_bytes() + size_of::<ObservedCubicWorker>()
    }
}
#[derive(Debug)]
struct ObservedCubicWorker {
    actual: Box<dyn InnerWorker>,
    work: Arc<Work>,
}
impl Drop for ObservedCubicWorker {
    fn drop(&mut self) {
        self.work.workers.fetch_sub(1, Ordering::Relaxed);
    }
}
impl InnerWorker for ObservedCubicWorker {
    fn solve(
        &mut self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &pse_diagnostics::Cancellation,
    ) -> Result<Vec<f64>, MathError> {
        self.work.solves.fetch_add(1, Ordering::Relaxed);
        self.actual.solve(problem, parameters, options, cancel)
    }
}

fn factory(control: &pse_diagnostics::Cancellation) -> (Factory, Arc<Work>) {
    crate::initialize().unwrap();
    let id = pse_ids::named_id(SemanticId::NIL, "Peng-Robinson compressibility");
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let q = registry.neutral_dimensionless().unwrap();
    let unit = registry.quantity_type(q).unwrap().canonical_unit;
    let z = crate::library::formal(0).unwrap();
    let a = crate::library::formal(1).unwrap();
    let b = crate::library::formal(2).unwrap();
    // Current authored compressibility_polynomial, specialized to PR u=2,w=-1:
    // z³-(1-B)z²+(A-2B-3B²)z-(AB-B²-B³).
    let expression = z.clone().pow(3) - (Atom::num(1) - &b) * z.clone().pow(2)
        + (&a - Atom::num(2) * &b - Atom::num(3) * b.clone().pow(2)) * &z
        - (&a * &b - b.clone().pow(2) - b.pow(3));
    let prepared = PreparedBody::new(
        3,
        4,
        vec![3],
        vec![Stage::Block {
            expressions: vec![expression],
            outputs: vec![3],
            source: id,
        }],
        DerivativeOrder::Second,
    )
    .unwrap();
    let limits = EvaluationLimits::default();
    let work = Arc::new(Work::default());
    let solver = Arc::new(ObservedCubic {
        actual: CubicRoots::new(&prepared, limits, &control.flag()).unwrap(),
        work: work.clone(),
    });
    let compiled = Arc::new(
        prepared
            .compile(
                &[0],
                &[0, 1, 2],
                DerivativeOrder::Second,
                Optimization::default(),
                limits,
                &ExecutionScope::new(control.clone(), None),
            )
            .unwrap(),
    );
    let unknowns = vec![Unknown {
        id,
        lower: 0.6,
        upper: 1.2,
    }];
    let options = Options {
        start: vec![0.9],
        variable_nominals: vec![1.],
        variable_tolerance: vec![1e-10],
        residual_tolerance: vec![1e-11],
        iterations: 20,
        time_limit: Duration::from_secs(10),
        derivative_tolerance: 1e-10,
    };
    let port = |name| pse_kernels::Port {
        id: pse_ids::named_id(id, name),
        quantity: q,
        unit,
    };
    (
        Factory {
            selection: Selection::default(),
            requirements: pse_kernels::DerivativeRequirements::new(
                DerivativeOrder::Second,
                DerivativeOrder::Second,
                DerivativeOrder::Second,
                DerivativeOrder::Value,
                DerivativeOrder::Second,
            )
            .unwrap(),
            spec: ProviderSpec {
                shapes: Default::default(),
                derivative_source: pse_kernels::DerivativeSource::Analytic,
                id,
                revision: ContentHash::from_bytes([111; 32]),
                data: ContentHash::from_bytes([112; 32]),
                inputs: vec![port("A"), port("B")],
                outputs: vec![port("Z")],
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            bodies: residual_bodies_for_test(compiled),
            unknowns: unknowns.clone(),
            rows: vec![id],
            configuration: Configuration::Fixed(unknowns, options),
            hints: None,
            terms: None,
            solver,
            cancel: control.clone(),
            max_entries: 1024,
            providers: BTreeMap::new(),
        },
        work,
    )
}

fn oracle(a: f64, b: f64, z: f64) -> ProviderValues {
    let fz = 3. * z * z - 2. * (1. - b) * z + a - 2. * b - 3. * b * b;
    let f =
        z * z * z - (1. - b) * z * z + (a - 2. * b - 3. * b * b) * z - a * b + b * b + b * b * b;
    assert!(f.abs() < 1e-14);
    let first = [
        -(z - b) / fz,
        -(z * z - (2. + 6. * b) * z - a + 2. * b + 3. * b * b) / fz,
    ];
    let fzi = [1., 2. * z - 2. - 6. * b];
    let fij = [[0., -1.], [-1., -6. * z + 2. + 6. * b]];
    let mut second = Vec::new();
    for i in 0..2 {
        for j in 0..2 {
            second.push(
                -(fij[i][j]
                    + fzi[i] * first[j]
                    + fzi[j] * first[i]
                    + (6. * z - 2. + 2. * b) * first[i] * first[j])
                    / fz,
            );
        }
    }
    ProviderValues {
        values: vec![z],
        jacobian: first.to_vec(),
        hessians: second,
    }
}
fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (&actual, &expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 2e-11 * expected.abs().max(1.),
            "{actual} != {expected}"
        );
    }
}
fn count(context: &pse_diagnostics::observation::ObservationContext, name: &str) -> u64 {
    context
        .reader()
        .snapshot(Duration::from_secs(60))
        .metrics
        .iter()
        .find_map(|metric| {
            if metric.name == name
                && let pse_diagnostics::observation::Value::Count(value) = metric.cumulative
            {
                return Some(value);
            }
            None
        })
        .unwrap_or(0)
}

#[test]
fn peng_robinson_nested_property_reuses_qualified_jets_and_preserves_invalidations()
-> Result<(), &'static str> {
    let control = pse_diagnostics::Cancellation::new(false);
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let operation = hub.start("PR actual property reuse");
    let observation = operation.context();
    let _entered = observation.enter();
    let (factory, work) = factory(&control);
    let mut provider = factory.create().unwrap();
    let baseline = count(&observation, "math.arithmetic_region_preparations");
    assert!(baseline > 0);
    let cancel = control.flag();
    let context = EvaluationContext {
        cancelled: cancel.as_ref(),
        max_result_bytes: 1 << 20,
    };
    // Solving the declared polynomial for A at these prescribed Z/B values gives
    // exact nontrivial physical cubic controls, independent of the root algorithm.
    let points = [(287. / 1200., 0.05, 0.8), (12684. / 110000., 0.02, 0.9)];
    for (index, &(a, b, z)) in points.iter().enumerate() {
        let expected = oracle(a, b, z);
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
            DerivativeOrder::First,
        ] {
            let actual = provider_values(&mut provider, 
                    &[a, b],
                    &ProviderRequest::all(&factory.spec, order),
                    &context,
                )
                .unwrap();
            close(&actual.values, &expected.values);
            if order >= DerivativeOrder::First {
                close(&actual.jacobian, &expected.jacobian);
            }
            if order >= DerivativeOrder::Second {
                close(&actual.hessians, &expected.hessians);
            }
        }
        assert_eq!(work.solves.load(Ordering::Relaxed), index + 1);
    }
    assert_eq!(
        count(&observation, "math.arithmetic_region_preparations"),
        baseline
    );
    assert_eq!(count(&observation, "implicit.derivatives.jet.completed"), 4);
    assert_eq!(
        count(&observation, "implicit.derivatives.contraction.completed"),
        2
    );
    assert_eq!(count(&observation, "implicit.factor.numeric.completed"), 2);
    assert_eq!(
        count(&observation, "implicit.factor.solve.completed"),
        4,
        "First to Second retains the associated factor and completed first solves"
    );
    assert_eq!(count(&observation, "implicit.product.full_reuse"), 2);
    assert!(
        provider_values(&mut provider, 
                &[2., 0.02],
                &ProviderRequest::all(&factory.spec, DerivativeOrder::First),
                &context
            )
            .is_err(),
        "the gas branch has no admissible root"
    );
    let (a, b, z) = points[1];
    close(
        &provider_values(&mut provider, 
                &[a, b],
                &ProviderRequest::all(&factory.spec, DerivativeOrder::Second),
                &context,
            )
            .unwrap()
            .values,
        &[z],
    );
    assert_eq!(
        work.solves.load(Ordering::Relaxed),
        4,
        "failed branch work clears the former jet"
    );
    let mut changed = factory.clone();
    let Configuration::Fixed(_, options) = &mut changed.configuration else {
        return Err("the PR property fixture requires a fixed configuration");
    };
    options.derivative_tolerance *= 0.5;
    assert_ne!(changed.configuration_key(), factory.configuration_key());
    let mut independent = changed.create().unwrap();
    close(
        &provider_values(&mut independent, 
                &[a, b],
                &ProviderRequest::all(&changed.spec, DerivativeOrder::First),
                &context,
            )
            .unwrap()
            .jacobian,
        &oracle(a, b, z).jacobian,
    );
    assert_eq!(work.solves.load(Ordering::Relaxed), 5);
    let mut unsupported = factory.clone();
    unsupported.requirements.requested_output = DerivativeOrder::First;
    let mut restricted = unsupported.create().unwrap();
    assert!(
        provider_values(&mut restricted, 
                &[a, b],
                &ProviderRequest::all(&unsupported.spec, DerivativeOrder::Second),
                &context
            )
            .is_err()
    );
    assert_eq!(work.solves.load(Ordering::Relaxed), 5);
    control.cancel();
    assert!(matches!(
        provider_values(&mut provider, 
            &[a, b],
            &ProviderRequest::all(&factory.spec, DerivativeOrder::Value),
            &context
        ),
        Err(pse_kernels::ProviderError::Cancelled)
    ));
    drop((provider, independent, restricted));
    assert_eq!(work.workers.load(Ordering::Relaxed), 0);
    Ok(())
}

#[test]
fn peng_robinson_weighted_property_products_rebind_weights_and_isolate_workers() {
    let control = pse_diagnostics::Cancellation::new(false);
    let (factory, work) = factory(&control);
    let scope = ExecutionScope::new(control.clone(), None);
    let outer = PreparedBody::new(
        2,
        3,
        vec![2],
        vec![Stage::Provider {
            spec: factory.spec.clone(),
            partial: vec![],
            inputs: vec![0, 1],
            outputs: vec![2],
            source: factory.spec.id,
        }],
        DerivativeOrder::Second,
    )
    .unwrap();
    let support = outer
        .prepare_support(&[0], &[0, 1], DerivativeOrder::Second, &control)
        .unwrap();
    let compiled = support
        .compile_product(
            ProductKind::WeightedHessian,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope,
        )
        .unwrap();
    let mut worker = compiled.worker_scoped(scope);
    let mut providers = [(factory.spec.key(), factory.create().unwrap())].into_iter().collect();
    let a = 287. / 1200.;
    let b = 0.05;
    let expected = oracle(a, b, 0.8);
    for weight in [1., -2., -2.] {
        let mut output = [f64::NAN; 3];
        worker
            .evaluate_product_into(
                &[a, b],
                ProductValues::WeightedHessian { weights: &[weight] },
                &mut providers,
                &control.flag(),
                &mut output,
            )
            .unwrap();
        close(
            &output,
            &[
                weight * expected.hessians[0],
                weight * expected.hessians[1],
                weight * expected.hessians[3],
            ],
        );
        assert_eq!(
            work.solves.load(Ordering::Relaxed),
            1,
            "weights consume the already qualified property jet"
        );
    }
    let (next_a, next_b) = (12684. / 110000., 0.02);
    let expected = oracle(next_a, next_b, 0.9);
    let mut output = [f64::NAN; 3];
    worker
        .evaluate_product_into(
            &[next_a, next_b],
            ProductValues::WeightedHessian { weights: &[3.] },
            &mut providers,
            &control.flag(),
            &mut output,
        )
        .unwrap();
    close(
        &output,
        &[
            3. * expected.hessians[0],
            3. * expected.hessians[1],
            3. * expected.hessians[3],
        ],
    );
    assert_eq!(work.solves.load(Ordering::Relaxed), 2);
    let mut independent = factory.create().unwrap();
    let cancel = control.flag();
    let context = EvaluationContext {
        cancelled: cancel.as_ref(),
        max_result_bytes: 1 << 20,
    };
    let actual = provider_values(&mut independent, 
            &[next_a, next_b],
            &ProviderRequest::all(&factory.spec, DerivativeOrder::Second),
            &context,
        )
        .unwrap();
    close(&actual.hessians, &expected.hessians);
    assert_eq!(
        work.solves.load(Ordering::Relaxed),
        3,
        "independent attempts never borrow mutable property workers"
    );
    assert_eq!(work.workers.load(Ordering::Relaxed), 2);
    drop((providers, independent, worker));
    assert_eq!(work.workers.load(Ordering::Relaxed), 0);
}

#[derive(Debug, Clone)]
struct SelectorEffect {
    spec: ProviderSpec,
    value: Arc<AtomicU64>,
    calls: Arc<AtomicUsize>,
    refuse: Arc<AtomicBool>,
}
impl pse_kernels::Provider for SelectorEffect {
    fn fill(
        &mut self, _: &[f64], request: &pse_kernels::PreparedProviderRequest,
        _: pse_kernels::ProviderRuntime<'_>, output: pse_kernels::ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), pse_kernels::ProviderError> {
        request.check_context(context)?;
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.refuse.load(Ordering::Acquire) {
            return Err(pse_kernels::ProviderError::Trial("current operational selector refusal".into()));
        }
        output.values.fill(f64::from_bits(self.value.load(Ordering::Acquire)));
        output.jacobian.fill(0.); output.hessians.fill(0.);
        Ok(())
    }
}
impl ProviderFactory for SelectorEffect {
    fn spec(&self) -> &ProviderSpec { &self.spec }
    fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
        Ok(pse_kernels::ProviderBinding::new(self.spec.clone(), Box::new(self.clone())))
    }
}

#[test]
fn peng_robinson_operational_selector_effects_run_before_same_point_reuse() {
    // Both source owners permit registered providers. Their effect barriers must
    // survive independently: anchors precede configuration and restrictions follow
    // a qualified jet, including when a previous numerical product can be reused.
    for restriction in [false, true] {
        for effectful in [false, true] {
            let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
            let operation = hub.start("PR operational selector effects");
            let observation = operation.context();
            let _entered = observation.enter();
            let control = pse_diagnostics::Cancellation::new(false);
            let (mut factory, work) = factory(&control);
            let calls = Arc::new(AtomicUsize::new(0));
            let refuse = Arc::new(AtomicBool::new(false));
            let value: f64 = if restriction { 1. } else { 0.9 };
            let current_value = Arc::new(AtomicU64::new(value.to_bits()));
            let mut spec = factory.spec.clone();
            spec.id = pse_ids::named_id(spec.id, "operational selector provider");
            let stage = if effectful {
                let registration = pse_kernels::Registration::new(
                    Arc::new(SelectorEffect {
                        spec: spec.clone(),
                        value: current_value.clone(),
                        calls: calls.clone(),
                        refuse: refuse.clone(),
                    }),
                    &pse_quantity::standard::standard_registry().unwrap(),
                )
                .unwrap();
                factory.providers.insert(spec.key(), registration);
                Stage::Provider {
                    spec,
                    partial: vec![],
                    inputs: vec![1, 2],
                    outputs: vec![3],
                    source: factory.spec.id,
                }
            } else {
                Stage::Block {
                    expressions: vec![Atom::num(value)],
                    outputs: vec![3],
                    source: factory.spec.id,
                }
            };
            let order = if restriction {
                DerivativeOrder::First
            } else {
                DerivativeOrder::Value
            };
            let source = PreparedBody::new(3, 4, vec![3], vec![stage], order).unwrap();
            let program = Arc::new(
                source
                    .compile(
                        &[0],
                        &[0, 1, 2],
                        order,
                        Optimization::default(),
                        EvaluationLimits::default(),
                        &ExecutionScope::new(control.clone(), None),
                    )
                    .unwrap(),
            );
            if restriction {
                factory.selection.restriction = Some(program);
            } else {
                factory.selection.anchor = Some(program);
            }
            let mut provider = factory.create().unwrap();
            let cancel = control.flag();
            let context = EvaluationContext {
                cancelled: cancel.as_ref(),
                max_result_bytes: 1 << 20,
            };
            let request = ProviderRequest::all(&factory.spec, DerivativeOrder::Second);
            for _ in 0..2 {
                let result = provider_values(&mut provider, &[287. / 1200., 0.05], &request, &context)
                    .unwrap();
                close(&result.hessians, &oracle(287. / 1200., 0.05, 0.8).hessians);
            }
            assert_eq!(
                work.solves.load(Ordering::Relaxed),
                1,
                "unchanged resolved selector premises retain the qualified numerical product"
            );
            assert_eq!(
                count(&observation, "implicit.product.full_reuse"),
                u64::from(!effectful),
                "pure programs retain the original complete fast reuse route"
            );
            if effectful {
                assert_eq!(
                    calls.load(Ordering::Relaxed),
                    2,
                    "same-point reuse must execute the actual operational provider barrier"
                );
                if !restriction {
                    current_value.store(0.8_f64.to_bits(), Ordering::Release);
                    provider_values(&mut provider, &[287. / 1200., 0.05], &request, &context)
                        .unwrap();
                    assert_eq!(
                        work.solves.load(Ordering::Relaxed),
                        2,
                        "changed operational anchor invalidates the numerical realization key"
                    );
                }
                let before_calls = calls.load(Ordering::Relaxed);
                let before_solves = work.solves.load(Ordering::Relaxed);
                refuse.store(true, Ordering::Release);
                assert!(
                    provider_values(&mut provider, &[287. / 1200., 0.05], &request, &context)
                        .is_err()
                );
                assert_eq!(calls.load(Ordering::Relaxed), before_calls + 1);
                refuse.store(false, Ordering::Release);
                provider_values(&mut provider, &[287. / 1200., 0.05], &request, &context)
                    .unwrap();
                assert_eq!(
                    work.solves.load(Ordering::Relaxed),
                    before_solves + 1,
                    "a refused selector clears previous numerical validity"
                );
            } else {
                assert_eq!(calls.load(Ordering::Relaxed), 0);
                assert!(provider.spec() == &factory.spec);
            }
            drop(provider);
            assert_eq!(work.workers.load(Ordering::Relaxed), 0);
        }
    }
}
