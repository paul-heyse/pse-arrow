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
            derivatives: pse_kernels::DerivativeOrder::First,
        })
    }
}
impl Oracle for Tank {
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
        Ok(Evaluation { values, jacobian })
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
        let plain = integrate(&mut Tank::new(), &horizon(method), &PARAMETERS, Arc::default()).unwrap();
        assert_eq!(plain.termination, Termination::Completed, "{method:?}: {:?}", plain.error);
        let middle = &plain.samples[1];
        let mut window = Anchored::new(Tank::new(), false).unwrap();
        assert_eq!(window.anchors(), &[0]);
        let mut profile = window.profile(&horizon(method).window(1.0, 2.0, vec![1.0, 2.0]));
        profile.sensitivity = DynamicSensitivity::Forward;
        let parameters = [PARAMETERS[0], PARAMETERS[1], middle.state[0]];
        let continued = integrate(&mut window, &profile, &parameters, Arc::default()).unwrap();
        assert_eq!(continued.termination, Termination::Completed, "{method:?}: {:?}", continued.error);
        let (end, reference) = (continued.samples.last().unwrap(), plain.samples.last().unwrap());
        for (a, b) in end.outputs.iter().zip(&reference.outputs) {
            assert!((a - b).abs() < 1e-7, "{method:?}: {a} vs {b}");
        }
        let window_integral = end.integrals[0];
        let difference = reference.integrals[0] - middle.integrals[0];
        assert!((window_integral - difference).abs() < 1e-7, "{method:?}: {window_integral} vs {difference}");
        // The anchor's column follows the unscheduled contract parameters.
        let step = 1e-5;
        let shifted = |delta: f64| {
            let mut q = parameters;
            q[2] += delta;
            let mut p = profile.clone();
            p.sensitivity = DynamicSensitivity::None;
            let r = integrate(&mut Anchored::new(Tank::new(), false).unwrap(), &p, &q, Arc::default()).unwrap();
            r.samples.last().unwrap().outputs.clone()
        };
        let (plus, minus) = (shifted(step), shifted(-step));
        for o in 0..2 {
            let d = (plus[o] - minus[o]) / (2.0 * step);
            let s = end.output_sensitivities[o * 3 + 2];
            assert!((s - d).abs() <= 1e-5 * (1.0 + d.abs()), "{method:?}: output {o}: {s} vs {d}");
        }
    }
}

/// Observed quadratures integrate as states: the window's quadrature output equals the
/// native quadrature, and its adjoint gradient equals central differences, over both
/// parameters and the anchor.
#[test]
fn observed_quadrature_adjoint_matches_differences() {
    for method in methods() {
        let native = integrate(&mut Tank::new(), &horizon(method), &PARAMETERS, Arc::default()).unwrap();
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
            Arc::default(),
            256 << 20,
        )
        .unwrap();
        assert_eq!(g.report.termination, Termination::Completed, "{method:?}: {:?}", g.report.error);
        let observed = g.report.samples.last().unwrap().outputs[2];
        let integral = native.samples.last().unwrap().integrals[0];
        assert!((observed - integral).abs() < 1e-7, "{method:?}: {observed} vs {integral}");
        let gradient = g.gradient.unwrap();
        profile.sensitivity = DynamicSensitivity::None;
        for (j, value) in gradient.iter().enumerate() {
            let step = 1e-5;
            let at = |delta: f64| {
                let mut q = parameters;
                q[j] += delta;
                let r = integrate(&mut Anchored::new(Tank::new(), true).unwrap(), &profile, &q, Arc::default()).unwrap();
                r.samples.last().unwrap().outputs[2]
            };
            let d = (at(step) - at(-step)) / (2.0 * step);
            assert!((value - d).abs() <= 1e-5 * (1.0 + d.abs()), "{method:?}: column {j}: {value} vs {d}");
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
        state: 0,
        scale: 1.0,
        tolerance: 1e-8,
        impulses: Default::default(),
    }];
    assert!(matches!(Anchored::new(balanced, true), Err(ProblemError::Contract(_))));
    let mut window = Anchored::new(Tank::new(), false).unwrap();
    assert!(matches!(
        window.evaluate(0, Function::Rhs, 0.0, &[1.0, 1.0], &[0.8, 1.2], false),
        Err(ProblemError::Contract(_))
    ));
}
