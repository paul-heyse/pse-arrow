// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Short periodic reaction/diffusion controls with exact two-mode trajectories.
use super::*;
use std::{cell::RefCell, rc::Rc, time::Instant};

#[derive(Debug)]
struct Periodic {
    contract: Contract,
    calls: Rc<RefCell<[usize; 4]>>,
}
impl Periodic {
    fn new(n: usize) -> Self {
        let id = |i: usize| SemanticId::from_bytes((i as u128).to_le_bytes());
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([135; 32]),
                states: (0..n).map(id).collect(), differential: vec![true; n],
                parameters: vec![id(n), id(n + 1)], outputs: vec![id(n + 2)],
                events: vec![vec![]], signs: vec![], quadratures: vec![], balances: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
            },
            calls: Rc::default(),
        }
    }
    fn exact(i: usize, n: usize, t: f64, rate: f64, diffusion: f64) -> f64 {
        let theta = std::f64::consts::TAU / n as f64;
        1. + 0.2 * (theta * i as f64).cos() * (-(rate + diffusion * (2. - 2. * theta.cos())) * t).exp()
            + 0.05 * if i % 2 == 0 { 1. } else { -1. } * (-(rate + 4. * diffusion) * t).exp()
    }
}
impl Oracle for Periodic {
    fn contract(&self) -> &Contract { &self.contract }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        let n = self.contract.states.len();
        entries(match function {
            Function::Rhs => (0..n).flat_map(|i| [(i, i), (i, (i + n - 1) % n), (i, (i + 1) % n), (i, n), (i, n + 1)]).collect(),
            Function::Output => vec![(0, 0)], _ => vec![],
        })
    }
    fn evaluate(&mut self, _: usize, function: Function, _: f64, x: &[f64], p: &[f64], derivative: bool) -> Result<Evaluation, ProblemError> {
        let n = self.contract.states.len();
        self.calls.borrow_mut()[usize::from(derivative) + 2 * usize::from(function == Function::Rhs)] += 1;
        let (values, partials) = match function {
            Function::Initial => ((0..n).map(|i| Self::exact(i, n, 0., p[0], p[1])).collect(), vec![]),
            Function::Rhs => {
                let mut values = vec![0.; n]; let mut partials = Vec::with_capacity(if derivative { 5 * n } else { 0 });
                for i in 0..n {
                    let left = (i + n - 1) % n; let right = (i + 1) % n;
                    let laplace = x[left] - 2. * x[i] + x[right];
                    values[i] = -p[0] * (x[i] - 1.) + p[1] * laplace;
                    if derivative { partials.extend([(i, i, -p[0] - 2. * p[1]), (i, left, p[1]), (i, right, p[1]), (i, n, -(x[i] - 1.)), (i, n + 1, laplace)]); }
                }
                (values, partials)
            }
            Function::Output => (vec![x[0]], vec![(0, 0, 1.)]), _ => (vec![], vec![]),
        };
        let jacobian = derivative.then(|| Arc::new(faer::sparse::SparseColMat::try_new_from_triplets(values.len(), n + 2,
            &partials.into_iter().map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v)).collect::<Vec<_>>()).unwrap()).into());
        Ok(Evaluation { values, jacobian })
    }
}
#[test]
fn nonstiff_stiff_sparse_profiles_and_output_density_preserve_two_mode_solution() {
    let n = 64;
    for diffusion in [0.1, 500.] {
        let mut profiles = Vec::new();
        #[cfg(feature = "diffsol")]
        for (method, linear) in [(DiffsolMethod::Tsit45, DiffsolLinear::FaerLu), (DiffsolMethod::Bdf, DiffsolLinear::FaerLu), (DiffsolMethod::Bdf, DiffsolLinear::Klu)] {
            let mut p = Profile { method: Method::Diffsol, ..Default::default() }; p.diffsol.method = method; p.diffsol.linear = linear; profiles.push(p);
        }
        #[cfg(feature = "idas")]
        for linear in [IdasLinear::Klu, IdasLinear::Spgmr { dimension: PositiveCount::try_new(20).unwrap(), preconditioner: crate::solve::Preconditioner::Jacobi }] {
            let mut p = Profile { method: Method::Idas, ..Default::default() }; p.idas.linear = linear; profiles.push(p);
        }
        for mut p in profiles {
            p.end = 1.; p.rtol = 0.001; p.atol = vec![0.001; n]; p.initial_step = 1e-5;
            p.parameter_scales = vec![1., 1.]; p.max_steps = 10000;
            let mut density_results = Vec::new();
            for samples in [5, 25] {
                p.samples = (0..samples).map(|i| i as f64 / (samples - 1) as f64).collect();
                let mut oracle = Periodic::new(n); let calls = oracle.calls.clone(); let started = Instant::now();
                let report = integrate(&mut oracle, &p, &[0.1, diffusion], Cancellation::default()).unwrap();
                assert_eq!(report.termination, Termination::Completed, "{:?} {:?} {:?} diffusion={diffusion}: {:?}", p.method, p.diffsol, p.idas.linear, report.error);
                for sample in &report.samples {
                    for (i, actual) in sample.state.iter().enumerate() {
                        let exact = Periodic::exact(i, n, sample.time, 0.1, diffusion);
                        assert!((actual - exact).abs() <= 0.01, "{:?} diffusion={diffusion} t={} state={i}: {actual} vs {exact}", p.method, sample.time);
                    }
                    assert!((sample.state.iter().sum::<f64>() / n as f64 - 1.).abs() <= 0.001, "mean closure");
                }
                let summaries: Vec<_> = report.statistics.iter().map(|value| {
                    value.as_object().unwrap().iter().filter(|(name, _)| name.starts_with("number_of_") || matches!(name.as_str(),
                        "steps" | "linear_setups" | "error_test_failures" | "nonlinear_iterations" | "nonlinear_convergence_failures" | "jacobian_products" | "preconditioner_evaluations" | "preconditioner_solves" | "linear_convergence_failures"))
                        .map(|(name, value)| (name, value)).collect::<std::collections::BTreeMap<_, _>>()
                }).collect();
                eprintln!("dynamics method={:?} diffsol={:?} idas={:?} diffusion={diffusion} samples={samples} wall_ns={} callbacks={:?} statistics={summaries:?}", p.method, p.diffsol, p.idas.linear, started.elapsed().as_nanos(), calls.borrow());
                density_results.push(report);
            }
            for key in ["number_of_steps", "number_of_linear_solver_setups", "steps", "linear_setups"] {
                assert_eq!(density_results[0].statistics[0].get(key), density_results[1].statistics[0].get(key), "matched density {key}");
            }
            for (a, b) in density_results[0].samples.iter().zip(density_results[1].samples.iter().step_by(6)) {
                assert_eq!(a.time, b.time);
                assert!(a.state.iter().zip(&b.state).all(|(a, b)| (a - b).abs() <= 0.01));
            }
        }
    }
}
