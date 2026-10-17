// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Anchored shooting windows (ADR-0110 Outcome 5) against the plain horizon, central
//! differences and native quadratures.
use super::*;
use crate::dynamics::{DynamicSensitivity, Method};
use std::sync::Arc;

/// A tank DAE: `x' = u − z` with the algebraic `0 = k·x² − z` from `x(0) = k`, the
/// quadrature `∫ x·u dt`, and the outputs `x` and `z`.
#[derive(Debug)]
struct Tank(Contract);
impl Tank {
    fn new() -> Self {
        let id = |n: u8| SemanticId::from_bytes([n; 16]);
        Self(Contract {
            quadratures: vec![id(90)],
            balances: vec![],
            identity: ContentHash::from_bytes([91; 32]),
            states: vec![id(92), id(93)],
            differential: vec![true, false],
            parameters: vec![id(94), id(95)],
            outputs: vec![id(92), id(93)],
            events: vec![vec![]],
            signs: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
        })
    }
}
impl Oracle for Tank {
    fn snapshot_owner(
        &self,
        bytes: usize,
    ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
        #[derive(Debug)]
        struct FixtureSnapshot {
            _bytes: usize,
        }
        Ok(Arc::new(FixtureSnapshot { _bytes: bytes }))
    }
    fn contract(&self) -> &Contract {
        &self.0
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        entries(match f {
            Function::Initial => vec![(0, 2), (1, 2)],
            Function::Rhs => vec![(0, 1), (0, 3), (1, 0), (1, 1), (1, 2)],
            Function::Output => vec![(0, 0), (1, 1)],
            Function::QuadratureFlux => vec![(0, 0), (0, 3)],
            _ => vec![],
        })
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        let (k, u) = (p[0], p[1]);
        let (values, entries) = match f {
            Function::Initial => (vec![k, k * k * k], vec![(0, 2, 1.0), (1, 2, 3.0 * k * k)]),
            Function::Rhs => (
                vec![u - x[1], k * x[0] * x[0] - x[1]],
                vec![
                    (0, 1, -1.0),
                    (0, 3, 1.0),
                    (1, 0, 2.0 * k * x[0]),
                    (1, 1, -1.0),
                    (1, 2, x[0] * x[0]),
                ],
            ),
            Function::Output => (x.to_vec(), vec![(0, 0, 1.0), (1, 1, 1.0)]),
            Function::QuadratureFlux => (vec![x[0] * u], vec![(0, 0, u), (0, 3, x[0])]),
            _ => (vec![], vec![]),
        };
        let jacobian = derivatives.then(|| {
            faer::sparse::SparseColMat::try_new_from_triplets(
                values.len(),
                4,
                &entries
                    .into_iter()
                    .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        });
        Ok(Evaluation {
            values,
            jacobian: jacobian.map(|matrix| Arc::new(matrix).into()),
        })
    }
}
fn methods() -> Vec<Method> {
    let mut methods = vec![Method::Diffsol];
    if cfg!(feature = "idas") {
        methods.push(Method::Idas);
    }
    methods
}
fn horizon(method: Method) -> Profile {
    Profile {
        method,
        end: 2.0,
        samples: vec![0.0, 1.0, 2.0],
        rtol: 1e-10,
        atol: vec![1e-12; 2],
        out_rtol: Some(1e-10),
        out_atol: vec![1e-12],
        initial_step: 1e-6,
        parameter_scales: vec![1.0; 2],
        ..Default::default()
    }
}
const PARAMETERS: [f64; 2] = [0.8, 1.2];

/// A window anchored at the horizon's state at t = 1 continues the horizon: its end outputs
/// and quadrature equal the plain integration's, and its anchor sensitivities equal
/// central differences.
#[test]
fn anchored_window_continues_the_horizon() {
    for method in methods() {
        let plain = integrate(
            &mut Tank::new(),
            &horizon(method),
            &PARAMETERS,
            Cancellation::default(),
        )
        .unwrap();
        assert_eq!(
            plain.termination,
            Termination::Completed,
            "{method:?}: {:?}",
            plain.error
        );
        let middle = &plain.samples[1];
        let mut window = Anchored::new(Tank::new(), false).unwrap();
        assert_eq!(window.anchors(), &[0]);
        let mut profile = window.profile(&horizon(method).window(1.0, 2.0, vec![1.0, 2.0]));
        profile.sensitivity = DynamicSensitivity::Forward;
        let parameters = [PARAMETERS[0], PARAMETERS[1], middle.state[0]];
        let continued =
            integrate(&mut window, &profile, &parameters, Cancellation::default()).unwrap();
        assert_eq!(
            continued.termination,
            Termination::Completed,
            "{method:?}: {:?}",
            continued.error
        );
        let (end, reference) = (
            continued.samples.last().unwrap(),
            plain.samples.last().unwrap(),
        );
        for (a, b) in end.outputs.iter().zip(&reference.outputs) {
            assert!((a - b).abs() < 1e-7, "{method:?}: {a} vs {b}");
        }
        let window_integral = end.integrals[0];
        let difference = reference.integrals[0] - middle.integrals[0];
        assert!(
            (window_integral - difference).abs() < 1e-7,
            "{method:?}: {window_integral} vs {difference}"
        );
        // The anchor's column follows the unscheduled contract parameters.
        let step = 1e-5;
        let shifted = |delta: f64| {
            let mut q = parameters;
            q[2] += delta;
            let mut p = profile.clone();
            p.sensitivity = DynamicSensitivity::None;
            let r = integrate(
                &mut Anchored::new(Tank::new(), false).unwrap(),
                &p,
                &q,
                Cancellation::default(),
            )
            .unwrap();
            r.samples.last().unwrap().outputs.clone()
        };
        let (plus, minus) = (shifted(step), shifted(-step));
        for o in 0..2 {
            let d = (plus[o] - minus[o]) / (2.0 * step);
            let s = end.output_sensitivities[o * 3 + 2];
            assert!(
                (s - d).abs() <= 1e-5 * (1.0 + d.abs()),
                "{method:?}: output {o}: {s} vs {d}"
            );
        }
    }
}

/// Observed quadratures integrate as states: the window's quadrature output equals the
/// native quadrature, and its adjoint gradient equals central differences, over both
/// parameters and the anchor.
#[test]
fn observed_quadrature_adjoint_matches_differences() {
    for method in methods() {
        let native = integrate(
            &mut Tank::new(),
            &horizon(method),
            &PARAMETERS,
            Cancellation::default(),
        )
        .unwrap();
        let mut window = Anchored::new(Tank::new(), true).unwrap();
        assert!(window.contract().quadratures.is_empty());
        assert_eq!(window.contract().states.len(), 3);
        let mut profile = window.profile(&horizon(method));
        assert!(profile.out_rtol.is_none() && profile.atol.len() == 3);
        let parameters = [PARAMETERS[0], PARAMETERS[1], PARAMETERS[0]];
        profile.sensitivity = DynamicSensitivity::Adjoint;
        let g = gradient(
            &mut window,
            &profile,
            &parameters,
            &mut |r: &Report| {
                let mut w = vec![0.0; r.samples.len() * 3];
                w[(r.samples.len() - 1) * 3 + 2] = 1.0;
                Ok(w)
            },
            Cancellation::default(),
            256 << 20,
        )
        .unwrap();
        assert_eq!(
            g.report.termination,
            Termination::Completed,
            "{method:?}: {:?}",
            g.report.error
        );
        let observed = g.report.samples.last().unwrap().outputs[2];
        let integral = native.samples.last().unwrap().integrals[0];
        assert!(
            (observed - integral).abs() < 1e-7,
            "{method:?}: {observed} vs {integral}"
        );
        let gradient = g.gradient.unwrap();
        profile.sensitivity = DynamicSensitivity::None;
        for (j, value) in gradient.iter().enumerate() {
            let step = 1e-5;
            let at = |delta: f64| {
                let mut q = parameters;
                q[j] += delta;
                let r = integrate(
                    &mut Anchored::new(Tank::new(), true).unwrap(),
                    &profile,
                    &q,
                    Cancellation::default(),
                )
                .unwrap();
                r.samples.last().unwrap().outputs[2]
            };
            let d = (at(step) - at(-step)) / (2.0 * step);
            assert!(
                (value - d).abs() <= 1e-5 * (1.0 + d.abs()),
                "{method:?}: column {j}: {value} vs {d}"
            );
        }
    }
}

/// Observed quadratures cannot carry conservation balances, and a mismatched binding is
/// refused.
#[test]
fn anchored_contract_limits() {
    let mut balanced = Tank::new();
    balanced.0.balances = vec![Balance {
        id: SemanticId::from_bytes([90; 16]),
        inventory: SemanticId::from_bytes([91; 16]),
        flux: SemanticId::from_bytes([90; 16]),
        tolerance: 1e-8,
        transfers: Default::default(),
    }];
    assert!(matches!(
        Anchored::new(balanced, true),
        Err(ProblemError::Contract(_))
    ));
    let mut window = Anchored::new(Tank::new(), false).unwrap();
    assert!(matches!(
        window.evaluate(0, Function::Rhs, 0.0, &[1.0, 1.0], &[0.8, 1.2], false),
        Err(ProblemError::Contract(_))
    ));
}

#[test]
fn anchored_snapshots_retain_distinct_owners_and_refuse_missing_admission() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Owner(Arc<AtomicUsize>);
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::AcqRel);
        }
    }
    #[derive(Debug)]
    struct Accounted {
        inner: Tank,
        live: Arc<AtomicUsize>,
        admit: bool,
    }
    impl Oracle for Accounted {
        fn contract(&self) -> &Contract {
            self.inner.contract()
        }
        fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry> {
            self.inner.support(mode, function)
        }
        fn snapshot_owner(
            &self,
            _bytes: usize,
        ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
            if !self.admit {
                return Err(ProblemError::memory("fixture snapshot admission refused"));
            }
            self.live.fetch_add(1, Ordering::AcqRel);
            Ok(Arc::new(Owner(self.live.clone())))
        }
        fn evaluate(
            &mut self,
            mode: usize,
            function: Function,
            time: f64,
            state: &[f64],
            parameters: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            self.inner
                .evaluate(mode, function, time, state, parameters, derivatives)
        }
    }
    let live = Arc::new(AtomicUsize::new(0));
    let mut worker = Anchored::new(
        Accounted {
            inner: Tank::new(),
            live: live.clone(),
            admit: true,
        },
        false,
    )
    .unwrap();
    let first = worker
        .evaluate(0, Function::Rhs, 0., &[1., 1.], &[0.8, 1.2, 1.], true)
        .unwrap();
    let alias = first.clone();
    let second = worker
        .evaluate(0, Function::Rhs, 0., &[2., 1.], &[0.8, 1.2, 1.], true)
        .unwrap();
    assert_eq!(live.load(Ordering::Acquire), 2);
    assert!(pse_math::SharedAllocation::ptr_eq(
        first.jacobian.as_ref().unwrap(),
        alias.jacobian.as_ref().unwrap()
    ));
    assert!(!pse_math::SharedAllocation::ptr_eq(
        first.jacobian.as_ref().unwrap(),
        second.jacobian.as_ref().unwrap()
    ));
    drop(worker);
    drop(first);
    assert_eq!(live.load(Ordering::Acquire), 2);
    drop(alias);
    assert_eq!(live.load(Ordering::Acquire), 1);
    drop(second);
    assert_eq!(live.load(Ordering::Acquire), 0);
    let mut refused = Anchored::new(
        Accounted {
            inner: Tank::new(),
            live: live.clone(),
            admit: false,
        },
        false,
    )
    .unwrap();
    assert!(
        refused
            .evaluate(0, Function::Rhs, 0., &[1., 1.], &[0.8, 1.2, 1.], true)
            .is_err()
    );
    assert!(
        refused
            .evaluate(0, Function::Rhs, 0., &[1., 1.], &[0.8, 1.2, 1.], false)
            .is_ok()
    );
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn anchored_layout_is_source_bound_and_retains_its_allocation_without_support_replay() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Charge(Arc<AtomicUsize>);
    impl Drop for Charge {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::AcqRel);
        }
    }
    #[derive(Debug)]
    struct Counted {
        inner: Tank,
        base: Arc<PreparedLayout>,
        queries: Arc<AtomicUsize>,
        live: Arc<AtomicUsize>,
    }
    impl Oracle for Counted {
        fn contract(&self) -> &Contract {
            self.inner.contract()
        }
        fn prepared_layout(&self, allowance: usize) -> Result<Arc<PreparedLayout>, ProblemError> {
            if self.base.retained_bytes() > allowance {
                return Err(ProblemError::memory("fixture layout allowance"));
            }
            Ok(self.base.clone())
        }
        fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry> {
            self.queries.fetch_add(1, Ordering::AcqRel);
            self.inner.support(mode, function)
        }
        fn snapshot_owner(
            &self,
            _: usize,
        ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
            self.live.fetch_add(1, Ordering::AcqRel);
            Ok(Arc::new(Charge(self.live.clone())))
        }
        fn evaluate(
            &mut self,
            mode: usize,
            function: Function,
            time: f64,
            state: &[f64],
            parameters: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            self.inner
                .evaluate(mode, function, time, state, parameters, derivatives)
        }
    }
    let tank = Tank::new();
    let base = Arc::new(
        PreparedLayout::new(
            tank.contract().states.len(),
            tank.support(0, Function::Rhs).into_iter(),
            4096,
        )
        .unwrap(),
    );
    let queries = Arc::new(AtomicUsize::new(0));
    let live = Arc::new(AtomicUsize::new(0));
    let worker = |tank| Counted {
        inner: tank,
        base: base.clone(),
        queries: queries.clone(),
        live: live.clone(),
    };
    let plain = Anchored::new(worker(Tank::new()), false).unwrap();
    assert!(Arc::ptr_eq(&plain.prepared_layout(4096).unwrap(), &base));
    assert_eq!(queries.load(Ordering::Acquire), 0);
    let observed = Anchored::new(worker(Tank::new()), true).unwrap();
    let prepared = observed.prepare_layout(4096).unwrap();
    assert_eq!(queries.load(Ordering::Acquire), 2);
    assert_eq!(live.load(Ordering::Acquire), 1);
    let mut reused = Anchored::new(worker(Tank::new()), true).unwrap();
    reused.use_prepared_layout(prepared.clone()).unwrap();
    let first = reused.prepared_layout(4096).unwrap();
    let second = reused.prepared_layout(4096).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.states(), 3);
    assert_eq!(queries.load(Ordering::Acquire), 2);
    assert!(reused.prepared_layout(0).is_err());
    let mut different = Tank::new();
    different.0.identity = ContentHash::from_bytes([7; 32]);
    assert!(
        Anchored::new(worker(different), true)
            .unwrap()
            .use_prepared_layout(prepared.clone())
            .is_err()
    );
    assert!(
        Anchored::new(worker(Tank::new()), false)
            .unwrap()
            .use_prepared_layout(prepared.clone())
            .is_err()
    );
    drop(first);
    drop(second);
    drop(prepared);
    assert_eq!(live.load(Ordering::Acquire), 1);
    drop(reused);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn anchored_actions_match_exact_partials_without_materializing_wrapper_matrices() {
    for observed in [false, true] {
        let mut oracle = Anchored::new(Tank::new(), observed).unwrap();
        let state = if observed {
            vec![1.2, 0.8, 0.3]
        } else {
            vec![1.2, 0.8]
        };
        let parameters = [0.7, 1.1, 0.9];
        let direction = (0..state.len() + parameters.len())
            .map(|i| 0.2 * (i + 1) as f64)
            .collect::<Vec<_>>();
        for function in [Function::Rhs, Function::Initial, Function::Output] {
            let reference = oracle
                .evaluate(0, function, 0.4, &state, &parameters, true)
                .unwrap();
            let matrix = reference.jacobian.unwrap();
            let mut expected = vec![0.; matrix.nrows()];
            for col in 0..matrix.ncols() {
                for entry in matrix.col_range(col) {
                    expected[matrix.row_idx()[entry]] += matrix.val()[entry] * direction[col];
                }
            }
            let mut actual = vec![f64::NAN; expected.len()];
            oracle
                .jacobian_action(
                    0,
                    function,
                    0.4,
                    &state,
                    &parameters,
                    &direction,
                    &mut actual,
                )
                .unwrap();
            for (actual, expected) in actual.iter().zip(&expected) {
                assert!((actual - expected).abs() < 1e-12);
            }
            let old = actual.clone();
            assert!(
                oracle
                    .jacobian_action(
                        0,
                        function,
                        0.4,
                        &state,
                        &parameters,
                        &direction[..1],
                        &mut actual
                    )
                    .is_err()
            );
            assert_eq!(actual, old, "failed action publishes no partial product");
        }
    }
}
