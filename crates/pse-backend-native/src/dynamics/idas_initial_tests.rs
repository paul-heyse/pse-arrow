// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Corrected algebraic states and sensitivities must seed both quadrature histories.
use super::*;

#[derive(Debug)]
struct AlgebraicFlux {
    contract: Contract,
    flux_calls: Vec<(f64, f64, f64, bool)>,
    reject_corrected: Option<bool>,
}
impl AlgebraicFlux {
    fn new() -> Self {
        let id = |v| SemanticId::from_bytes([v; 16]);
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([115; 32]),
                states: vec![id(1), id(2)],
                differential: vec![true, false],
                parameters: vec![id(3), id(4)],
                outputs: vec![id(5), id(6)],
                events: vec![vec![]],
                signs: vec![],
                quadratures: vec![id(7), id(8)],
                balances: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
            },
            flux_calls: vec![],
            reject_corrected: None,
        }
    }
}
impl Oracle for AlgebraicFlux {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        entries(match f {
            Function::Initial => vec![(0, 3)],
            Function::Rhs => vec![(1, 0), (1, 1), (1, 2)],
            Function::Output => vec![(0, 0), (1, 1)],
            Function::QuadratureFlux => vec![(0, 1), (1, 1)],
            _ => vec![],
        })
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        t: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        // x is constant; IDACalcIC must repair z and its sensitivities from zero
        // guesses to z = p[0] * x². The two fluxes depend on that algebraic state.
        let (values, partials) = match f {
            Function::Initial => (vec![p[1], 0.], vec![(0, 3, 1.)]),
            Function::Rhs => (
                vec![0., p[0] * x[0] * x[0] - x[1]],
                vec![(1, 0, 2. * p[0] * x[0]), (1, 1, -1.), (1, 2, x[0] * x[0])],
            ),
            Function::Output => (x.to_vec(), vec![(0, 0, 1.), (1, 1, 1.)]),
            Function::QuadratureFlux => {
                self.flux_calls.push((t, p[0], x[1], derivatives));
                if t == 0. && x[1] > 1. && self.reject_corrected == Some(derivatives) {
                    return Err(contract("intentional corrected quadrature history failure"));
                }
                (vec![x[1] * x[1], x[1]], vec![(0, 1, 2. * x[1]), (1, 1, 1.)])
            }
            _ => (vec![], vec![]),
        };
        let jacobian = derivatives
            .then(|| {
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    4,
                    &partials.into_iter().map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v)).collect::<Vec<_>>(),
                )
                .map_err(|error| ProblemError::internal(format!("fixture Jacobian: {error:?}")))
            })
            .transpose()?;
        Ok(Evaluation {
            values,
            jacobian: jacobian.map(|j| Arc::new(j).into()),
        })
    }
}

#[allow(clippy::unwrap_used, reason = "the deterministic fixture must complete every declared route")]
fn corrected_algebraic_flux(scheduled: bool) {
    for corrector in [SensitivityCorrector::Simultaneous, SensitivityCorrector::Staggered] {
        for selected in [None, Some(vec![if scheduled { 2 } else { 1 }, 0]), Some(vec![])] {
            let mut oracle = AlgebraicFlux::new();
            let mut p = Profile {
                method: Method::Idas,
                sensitivity: DynamicSensitivity::Forward,
                forward_axes: selected.clone(),
                samples: vec![0., 0.25, 0.5, 0.75, 1.],
                rtol: 0.001,
                atol: vec![0.001; 2],
                out_rtol: Some(0.001),
                out_atol: vec![0.001; 2],
                parameter_scales: vec![0.25, 8.],
                ..Default::default()
            };
            p.idas.sensitivity = corrector;
            let parameters = if scheduled {
                p.schedule = vec![ScheduledInput { parameter: 0, times: vec![0.5] }];
                vec![2., 3., 5.]
            } else {
                vec![3., 2.]
            };
            let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
            let product = forward_quadratures_observed(
                &mut oracle,
                &p,
                &parameters,
                Cancellation::default(),
                Arc::new(crate::solve::Progress::new(256)),
                &snapshot,
                32 << 20,
            ).unwrap();
            let report = product.report;
            assert_eq!(report.termination, Termination::Completed, "{corrector:?} {selected:?}: {:?}", report.error);
            assert_eq!(report.requested_initial, [2., 0.]);
            assert!((report.consistent_initial[1] - 12.).abs() < 1e-8);
            let axes = selected.unwrap_or_else(|| (0..parameters.len()).collect());
            assert_eq!(report.forward_axes, axes);
            // This is a history-initialization assertion, independent of whether an
            // adaptive step can eventually recover from an incorrect initial slope.
            // Q and QS must both evaluate at the segment start with corrected z.
            for (time, coefficient) in if scheduled { vec![(0., 3.), (0.5, 5.)] } else { vec![(0., 3.)] } {
                for derivative in [false, true].into_iter().filter(|d| !*d || !axes.is_empty()) {
                    assert!(oracle.flux_calls.iter().any(|&(t, c, z, d)|
                        t == time && c == coefficient && d == derivative && (z - 4. * coefficient).abs() < 1e-8
                    ), "{corrector:?} axes {axes:?}: missing corrected Q/QS history at {time}, derivative={derivative}: {:?}", oracle.flux_calls);
                }
            }
            for sample in &report.samples {
                let expected_z = if scheduled && sample.time >= 0.5 { 20. } else { 12. };
                assert!((sample.state[0] - 2.).abs() < 1e-8);
                assert!((sample.state[1] - expected_z).abs() < 1e-8);
                let first = sample.time.min(0.5);
                let later = (sample.time - 0.5).max(0.);
                let expected = if scheduled {
                    [144. * first + 400. * later, 12. * first + 20. * later]
                } else {
                    [144. * sample.time, 12. * sample.time]
                };
                for (actual, exact) in sample.integrals.iter().zip(expected) {
                    assert!((actual - exact).abs() <= 0.001 * (1. + exact.abs()), "{actual} vs {exact}");
                }
            }
            let gradients = if scheduled {
                vec![vec![544., 48., 80.], vec![16., 2., 2.]]
            } else {
                vec![vec![96., 288.], vec![4., 12.]]
            };
            let derivatives = product.derivatives.unwrap();
            assert_eq!(derivatives.len(), 2 * axes.len());
            for (q, expected) in gradients.iter().enumerate() {
                for (column, axis) in axes.iter().enumerate() {
                    let actual = derivatives[q * axes.len() + column];
                    let exact = expected[*axis];
                    assert!((actual - exact).abs() <= 0.001 * (1. + exact.abs()), "{corrector:?} axis {axis} q{q}: {actual} vs {exact}");
                }
            }
            // The retained native summary still reports the actual IC/step work;
            // the production repair changes no counter reset or segment boundary.
            assert!(report.statistics.iter().all(|stats| stats["nonlinear_iterations"].as_u64().unwrap() > 0));
        }
    }
}

#[test]
fn quadrature_histories_use_corrected_algebraic_initial_state_and_sensitivities() {
    corrected_algebraic_flux(false);
}
#[test]
fn quadrature_histories_use_corrected_algebraic_scheduled_state_and_sensitivities() {
    corrected_algebraic_flux(true);
}

#[test]
#[allow(clippy::unwrap_used, reason = "the fixture must return an attributable callback failure")]
fn corrected_quadrature_history_callback_failures_stop_before_physical_steps() {
    for derivatives in [false, true] {
        let mut oracle = AlgebraicFlux::new();
        oracle.reject_corrected = Some(derivatives);
        let p = Profile {
            method: Method::Idas,
            sensitivity: DynamicSensitivity::Forward,
            rtol: 0.001,
            atol: vec![0.001; 2],
            out_rtol: Some(0.001),
            out_atol: vec![0.001; 2],
            parameter_scales: vec![0.25, 8.],
            ..Default::default()
        };
        let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
        let product = forward_quadratures_observed(
            &mut oracle, &p, &[3., 2.], Cancellation::default(),
            Arc::new(crate::solve::Progress::new(256)), &snapshot, 32 << 20,
        ).unwrap();
        assert_ne!(product.report.termination, Termination::Completed);
        assert_eq!(product.report.completed_time, p.start);
        assert!(product.derivatives.is_none());
        assert!(product.report.error.unwrap().to_string().contains("intentional corrected quadrature history failure"));
        assert!(oracle.flux_calls.iter().all(|&(t, _, _, _)| t == p.start));
    }
}
