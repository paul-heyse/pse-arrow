// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Short coupled index-1 inquiry for the actual cj*M-J operator.
use super::*;
use std::time::Instant;
#[derive(Debug)]
struct Coupled {
    contract: Contract,
    alpha: f64,
    coupling: f64,
    calls: [usize; 2],
}
impl Coupled {
    fn new(alpha: f64, coupling: f64) -> Self {
        let id = |i| SemanticId::from_bytes([i; 16]);
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([171; 32]),
                states: vec![id(1), id(2)],
                differential: vec![true, false],
                parameters: vec![id(3)],
                outputs: vec![id(4)],
                events: vec![vec![]],
                quadratures: vec![],
                balances: vec![],
                signs: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
            },
            alpha,
            coupling,
            calls: [0; 2],
        }
    }
}
impl Oracle for Coupled {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        entries(match f {
            Function::Rhs => vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1)],
            Function::Output => vec![(0, 0)],
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
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        self.calls[usize::from(d)] += 1;
        let (values, partials) = match f {
            Function::Initial => (vec![1., self.alpha], vec![]),
            Function::Rhs => (
                vec![
                    -(p[0] + self.coupling) * x[0] + self.coupling / self.alpha * x[1],
                    self.alpha * x[0] - x[1],
                ],
                vec![
                    (0, 0, -p[0] - self.coupling),
                    (0, 1, self.coupling / self.alpha),
                    (0, 2, -x[0]),
                    (1, 0, self.alpha),
                    (1, 1, -1.),
                ],
            ),
            Function::Output => (vec![x[0]], vec![(0, 0, 1.)]),
            _ => (vec![], vec![]),
        };
        let jacobian = d.then(|| {
            Arc::new(
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    3,
                    &partials
                        .into_iter()
                        .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            )
            .into()
        });
        Ok(Evaluation { values, jacobian })
    }
}
#[test]
fn coupled_index1_shift_preconditioner_inquiry_preserves_scheduled_and_nonnormal_trajectories() {
    for alpha in [1., 0.01] {
        for coupling in [1., 200.] {
            for linear in [
                IdasLinear::Klu,
                IdasLinear::Spgmr {
                    dimension: pse_model::scalar!(PositiveCount(4)),
                    preconditioner: crate::solve::Preconditioner::None,
                },
                IdasLinear::Spgmr {
                    dimension: pse_model::scalar!(PositiveCount(4)),
                    preconditioner: crate::solve::Preconditioner::Jacobi,
                },
            ] {
                let mut oracle = Coupled::new(alpha, coupling);
                let profile = Profile {
                    method: Method::Idas,
                    end: 0.4,
                    samples: vec![0., 0.1, 0.2, 0.3, 0.4],
                    rtol: 0.001,
                    atol: vec![0.001, 0.001 * alpha],
                    initial_step: 1e-5,
                    parameter_scales: vec![1.],
                    schedule: vec![ScheduledInput {
                        parameter: 0,
                        times: vec![0.2],
                    }],
                    idas: IdasSettings {
                        linear,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let started = Instant::now();
                let report =
                    integrate(&mut oracle, &profile, &[1., 3.], Cancellation::default()).unwrap();
                assert_eq!(
                    report.termination,
                    Termination::Completed,
                    "{alpha}/{coupling}/{linear:?}: {:?}",
                    report.error
                );
                for sample in &report.samples {
                    let expected =
                        (-(sample.time.min(0.2) + 3. * (sample.time - 0.2).max(0.))).exp();
                    assert!((sample.state[0] - expected).abs() < 0.01);
                    assert!((sample.state[1] / alpha - sample.state[0]).abs() < 0.01);
                }
                eprintln!(
                    "dae-inquiry {}",
                    serde_json::json!({"alpha":alpha,"coupling":coupling,"linear":linear,
                    "elapsed_seconds":started.elapsed().as_secs_f64(),"oracle_calls":oracle.calls,"statistics":report.statistics})
                );
            }
        }
    }
}
