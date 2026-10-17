// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Short mechanism contrasts. Durations are observations, never acceptance thresholds.
use super::*;
use std::{cell::RefCell, rc::Rc, time::Instant};

#[derive(Clone, Copy, Debug)]
enum Shape {
    Cubic,
    ExponentialSum,
    Logarithmic,
}
impl Shape {
    fn value(self, x: f64) -> f64 {
        match self {
            Self::Cubic => x * x * x,
            Self::ExponentialSum => (1..=16).map(|k| (f64::from(k) * x / 16.).exp()).sum::<f64>() / 16.,
            Self::Logarithmic => x.ln(),
        }
    }
    fn derivative(self, x: f64) -> f64 {
        match self {
            Self::Cubic => 3. * x * x,
            Self::ExponentialSum => (1..=16).map(|k| {
                let a = f64::from(k) / 16.;
                a * (a * x).exp()
            }).sum::<f64>() / 16.,
            Self::Logarithmic => x.recip(),
        }
    }
}
#[derive(Debug, Default)]
struct Calls {
    residuals: usize,
    jacobians: usize,
    residual_nanos: u128,
    jacobian_nanos: u128,
    rejected: usize,
}
#[derive(Debug)]
struct Root {
    contract: OracleContract,
    matrix: faer::sparse::SparseColMat<usize, f64>,
    shape: Shape,
    coupling: f64,
    target: f64,
    calls: Rc<RefCell<Calls>>,
}
impl Root {
    fn new(n: usize, coupled: bool, shape: Shape, target: f64) -> Self {
        let id = |i| pse_ids::SemanticId::from_bytes([u8::try_from(i).unwrap(); 16]);
        let mut contract = crate::solver_tests::contract();
        contract.variables = (0..n).map(|i| crate::Variable {
            id: id(i), lower: f64::NEG_INFINITY, upper: f64::INFINITY,
        }).collect();
        contract.rows = (0..n).map(|i| id(n + i)).collect();
        let mut support = Vec::new();
        for i in 0..n {
            support.push(faer::sparse::Triplet::new(i, i, 1.));
            if coupled {
                support.push(faer::sparse::Triplet::new(i, (i + n - 1) % n, 1.));
            }
        }
        Self {
            contract,
            matrix: faer::sparse::SparseColMat::try_new_from_triplets(n, n, &support).unwrap(),
            shape, coupling: if coupled { 0.1 } else { 0. }, target,
            calls: Rc::default(),
        }
    }
}
impl NleOracle for Root {
    fn contract(&self) -> &OracleContract { &self.contract }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let started = Instant::now();
        let mut calls = self.calls.borrow_mut();
        calls.residuals += 1;
        if matches!(self.shape, Shape::Logarithmic) && x.iter().any(|x| *x <= 0.) {
            calls.rejected += 1;
            return Err(pse_math::MathError::Domain {
                source_id: self.contract.rows[0], requirement: "positive logarithm argument",
            }.into());
        }
        for (i, value) in out.iter_mut().enumerate() {
            *value = self.shape.value(x[i]) - self.shape.value(self.target)
                + self.coupling * (x[i] - x[(i + x.len() - 1) % x.len()]);
        }
        calls.residual_nanos += started.elapsed().as_nanos();
        Ok(())
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let started = Instant::now();
        let mut offset = 0;
        for (col, coordinate) in x.iter().enumerate() {
            for row in self.matrix.symbolic().row_idx_of_col(col) {
                out[offset] = if row == col {
                    self.shape.derivative(*coordinate) + self.coupling
                } else { -self.coupling };
                offset += 1;
            }
        }
        let mut calls = self.calls.borrow_mut();
        calls.jacobians += 1;
        calls.jacobian_nanos += started.elapsed().as_nanos();
        Ok(())
    }
    fn jacobian_product(&mut self, x: &[f64], direction: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for (i, value) in out.iter_mut().enumerate() {
            *value = (self.shape.derivative(x[i]) + self.coupling) * direction[i]
                - self.coupling * direction[(i + x.len() - 1) % x.len()];
        }
        Ok(())
    }
}
fn settings(n: usize, interval: u32) -> Settings {
    Settings {
        method: Method { setup_interval: interval, ..Method::default() },
        variable_scales: vec![1.; n], residual_scales: vec![1.; n], step_tolerance: 0.001,
    }
}
fn execution() -> Execution {
    let mut execution = crate::solver_tests::execution();
    execution.memory = Some(64 << 20);
    execution
}
fn solve(session: &mut Session, start: &[f64]) -> SolveReport {
    session.solve(start, &Controls::default(),
        &ResolvedAccuracy::from_policy(&pse_model::numerics::NumericalPolicy::default(), 0.001).unwrap(),
        execution(), &Tolerances {
            variables: vec![0.001; start.len()], rows: vec![0.001; start.len()], integrality: 0.001,
        }, None).unwrap()
}
fn count(report: &SolveReport, name: &str) -> i64 {
    let Metric::Integer(value) = report.metrics[name] else { panic!("missing {name}") };
    value
}
#[derive(Debug, Default)]
struct LinearPhases {
    started: [Option<Instant>; 6],
    nanos: [u128; 6],
}
unsafe extern "C" fn linear_phase(id: i32, begin: i32, userdata: *mut c_void) {
    // SAFETY: these tests register this exact stack-owned record only during one
    // synchronous solve, then clear the hook before assertions or record release.
    let phases = unsafe { &mut *userdata.cast::<LinearPhases>() };
    let Some(index) = usize::try_from(id).ok().and_then(|i| i.checked_sub(1)).filter(|i| *i < 6) else {
        return;
    };
    if begin != 0 {
        phases.started[index] = Some(Instant::now());
    } else if let Some(started) = phases.started[index].take() {
        phases.nanos[index] += started.elapsed().as_nanos();
    }
}
fn solve_observed(session: &mut Session, start: &[f64]) -> (SolveReport, LinearPhases) {
    let mut phases = LinearPhases::default();
    // SAFETY: live exclusively owned KLU owner; phases remains fixed during solve.
    assert_eq!(unsafe { ffi::SUNLinSol_KLUSetObserver(session.linear, Some(linear_phase), std::ptr::from_mut(&mut phases).cast()) }, 0);
    let report = solve(session, start);
    // SAFETY: clear exact live owner's hook before stack userdata may move or drop.
    assert_eq!(unsafe { ffi::SUNLinSol_KLUSetObserver(session.linear, None, std::ptr::null_mut()) }, 0);
    assert!(phases.started.iter().all(Option::is_none));
    (report, phases)
}
#[test]
fn short_profile_contrasts_at_engineering_accuracy_preserve_original_roots() {
    assert_eq!(Method::default().setup_interval, 10);
    for n in [4, 8] {
        for coupled in [false, true] {
            for shape in [Shape::Cubic, Shape::ExponentialSum, Shape::Logarithmic] {
                for interval in [1, 10] {
                    let root = Root::new(n, coupled, shape, 1.);
                    let calls = root.calls.clone();
                    let mut session = Session::new(Function::Equations(Box::new(root)),
                        settings(n, interval), execution(), crate::solver_tests::stamp(Backend::Kinsol)).unwrap();
                    let start = if matches!(shape, Shape::Logarithmic) { 5. } else { 0.25 };
                    let started = Instant::now();
                    let initial: Vec<_> = (0..n).map(|i| start + 0.02 * f64::from(u32::try_from(i).unwrap())).collect();
                    let (report, phases) = solve_observed(&mut session, &initial);
                    let elapsed = started.elapsed();
                    assert_eq!(report.termination.category, Termination::Success, "n={n} coupled={coupled} {shape:?} interval={interval}");
                    assert!(report.quality.as_ref().unwrap().feasible());
                    assert!(report.candidate.as_ref().unwrap().primal.iter().all(|x| (*x - 1.).abs() < 0.001));
                    assert_eq!(count(&report, "klu.symbolic.completed"), 1);
                    assert_eq!(count(&report, "klu.numeric_factor.completed"), 1);
                    eprintln!("profile n={n} coupled={coupled} shape={shape:?} interval={interval} wall_ns={} calls={:?} linear_phase_nanos={:?} setup={} symbolic={} numeric={} refactor={} solve={} domain_rejections={}",
                        elapsed.as_nanos(), calls.borrow(), phases.nanos, count(&report,"klu.setup.completed"), count(&report,"klu.symbolic.completed"), count(&report,"klu.numeric_factor.completed"), count(&report,"klu.refactor.completed"), count(&report,"klu.solve.completed"), report.evidence.callback.trial_rejections);
                    let accepted = solve(&mut session, &vec![1.; n]);
                    assert_eq!(accepted.termination.name, "KIN_INITIAL_GUESS_OK");
                    assert_eq!(count(&accepted, "klu.setup.completed"), 0);
                }
            }
        }
    }
}
#[test]
fn retained_klu_exact_layout_and_fresh_coefficient_application_boundary() {
    for coupled in [false, true] {
        let stamp = crate::solver_tests::stamp(Backend::Kinsol);
        let mut session = Session::new(Function::Equations(Box::new(Root::new(4, coupled, Shape::Cubic, 1.))),
            settings(4,10), execution(), stamp.clone()).unwrap();
        for (generation, target) in [(1_u8, 1.), (2, 1.2)] {
            let mut current = stamp.clone();
            current.data = pse_ids::ContentHash::from_bytes([generation; 32]);
            session.replace(Function::Equations(Box::new(Root::new(4, coupled, Shape::Cubic, target))), settings(4,10), current).unwrap();
            let (report, phases) = solve_observed(&mut session, &[0.25;4]);
            eprintln!("retained coupled={coupled} generation={generation} linear_phase_nanos={:?}", phases.nanos);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report,"klu.symbolic.completed"),1);
            assert_eq!(count(&report,"klu.numeric_factor.completed"),1);
            assert!(report.candidate.unwrap().primal.iter().all(|x| (*x-target).abs() < 0.001));
        }
        for ordering in [0, 2] {
            // SAFETY: documented KLU ordering choices on the exclusively owned
            // live solver; next KINSol still performs its required initialization.
            assert_eq!(unsafe { ffi::SUNLinSol_KLUSetOrdering(session.linear, ordering) }, 0);
            let report = solve(&mut session, &[0.25; 4]);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report, "klu.symbolic.completed"), 1);
            assert_eq!(count(&report, "klu.numeric_factor.completed"), 1);
        }
        assert!(session.replace(Function::Equations(Box::new(Root::new(4, !coupled, Shape::Cubic, 1.))), settings(4,10), stamp.clone()).is_err());
        let mut reordered = Root::new(4, coupled, Shape::Cubic, 1.);
        reordered.contract.variables.swap(0,1);
        assert!(session.replace(Function::Equations(Box::new(reordered)), settings(4,10), stamp.clone()).is_err());
        let mut changed_ordering = stamp;
        changed_ordering.profile = pse_ids::ContentHash::from_bytes([87;32]);
        assert!(!session.matches_layout(&changed_ordering));
    }
}

#[derive(Debug)]
struct Recycle {
    root: Root,
    coefficient: f64,
}
impl FixedPointOracle for Recycle {
    fn contract(&self) -> &OracleContract { &self.root.contract }
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.root.matrix.symbolic()
    }
    fn physical_tear_coordinates(&self) -> bool { true }
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for (i, value) in out.iter_mut().enumerate() {
            *value = self.root.target + self.coefficient * (x[i] - self.root.target)
                + self.root.coupling * (x[(i + x.len() - 1) % x.len()] - x[i]);
        }
        Ok(())
    }
    fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.map(x, out)?;
        for (value, x) in out.iter_mut().zip(x) { *value -= x; }
        Ok(())
    }
}
#[test]
fn short_recycle_profiles_cover_slow_and_oscillatory_maps_without_new_default() {
    for coefficient in [0.5, 0.98, -0.5] {
        for anderson in [0, 2] {
            let n = 4;
            let mut config = settings(n, 10);
            config.method.strategy = Strategy::FixedPoint;
            config.method.anderson = anderson;
            let mut session = Session::new(Function::FixedPoint(Box::new(Recycle {
                root: Root::new(n, true, Shape::Cubic, 1.), coefficient,
            })), config, execution(), crate::solver_tests::stamp(Backend::Kinsol)).unwrap();
            let report = solve(&mut session, &[0.25, 0.3, 0.35, 0.4]);
            assert_eq!(report.termination.category, Termination::Success);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report, "setup.refreshes"), 0);
            eprintln!("recycle coefficient={coefficient} anderson={anderson} iterations={} evaluations={} original_max={}",
                count(&report,"KINGetNumNonlinSolvIters"), count(&report,"KINGetNumFuncEvals"), report.quality.as_ref().unwrap().normalized_max);
        }
    }
}
