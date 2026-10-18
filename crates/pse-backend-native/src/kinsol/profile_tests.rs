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
            Self::ExponentialSum => {
                (1..=16)
                    .map(|k| (f64::from(k) * x / 16.).exp())
                    .sum::<f64>()
                    / 16.
            }
            Self::Logarithmic => x.ln(),
        }
    }
    fn derivative(self, x: f64) -> f64 {
        match self {
            Self::Cubic => 3. * x * x,
            Self::ExponentialSum => {
                (1..=16)
                    .map(|k| {
                        let a = f64::from(k) / 16.;
                        a * (a * x).exp()
                    })
                    .sum::<f64>()
                    / 16.
            }
            Self::Logarithmic => x.recip(),
        }
    }
}
#[derive(Debug, Default)]
struct Calls {
    residuals: usize,
    jacobians: usize,
    products: usize,
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
    band: usize,
    target: f64,
    calls: Rc<RefCell<Calls>>,
}
impl Root {
    fn new(n: usize, coupled: bool, shape: Shape, target: f64) -> Self {
        Self::banded(n, usize::from(coupled), shape, target)
    }
    fn banded(n: usize, band: usize, shape: Shape, target: f64) -> Self {
        assert!(band < n);
        let id = |i: usize| pse_ids::SemanticId::from_bytes((i as u128).to_le_bytes());
        let mut contract = crate::solver_tests::contract();
        contract.variables = (0..n)
            .map(|i| crate::Variable {
                id: id(i),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            })
            .collect();
        contract.rows = (0..n).map(|i| id(n + i)).collect();
        let mut support = Vec::new();
        for i in 0..n {
            support.push(faer::sparse::Triplet::new(i, i, 1.));
            for distance in 1..=band {
                support.push(faer::sparse::Triplet::new(i, (i + n - distance) % n, 1.));
            }
        }
        Self {
            contract,
            matrix: faer::sparse::SparseColMat::try_new_from_triplets(n, n, &support).unwrap(),
            shape,
            coupling: if band > 0 { 0.1 } else { 0. },
            band,
            target,
            calls: Rc::default(),
        }
    }
}
impl NleOracle for Root {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let started = Instant::now();
        let mut calls = self.calls.borrow_mut();
        calls.residuals += 1;
        if matches!(self.shape, Shape::Logarithmic) && x.iter().any(|x| *x <= 0.) {
            calls.rejected += 1;
            return Err(pse_math::MathError::Domain {
                source_id: self.contract.rows[0],
                requirement: "positive logarithm argument",
            }
            .into());
        }
        for (i, value) in out.iter_mut().enumerate() {
            *value = self.shape.value(x[i]) - self.shape.value(self.target)
                + self.coupling
                    * (1..=self.band)
                        .map(|distance| x[i] - x[(i + x.len() - distance) % x.len()])
                        .sum::<f64>();
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
                    self.shape.derivative(*coordinate) + self.coupling * self.band as f64
                } else {
                    -self.coupling
                };
                offset += 1;
            }
        }
        let mut calls = self.calls.borrow_mut();
        calls.jacobians += 1;
        calls.jacobian_nanos += started.elapsed().as_nanos();
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.calls.borrow_mut().products += 1;
        for (i, value) in out.iter_mut().enumerate() {
            *value = self.shape.derivative(x[i]) * direction[i]
                + self.coupling
                    * (1..=self.band)
                        .map(|distance| {
                            direction[i] - direction[(i + x.len() - distance) % x.len()]
                        })
                        .sum::<f64>();
        }
        Ok(())
    }
}
fn settings(n: usize, interval: u32) -> Settings {
    Settings {
        method: Method {
            setup_interval: interval,
            ..Method::default()
        },
        variable_scales: vec![1.; n],
        residual_scales: vec![1.; n],
        step_tolerance: 0.001,
    }
}
fn execution() -> Execution {
    let mut execution = crate::solver_tests::execution();
    execution.memory = Some(64 << 20);
    execution
}
fn solve(session: &mut Session, start: &[f64]) -> SolveReport {
    session
        .solve(
            start,
            &Controls::default(),
            &ResolvedAccuracy::from_policy(&pse_model::numerics::NumericalPolicy::default(), 0.001)
                .unwrap(),
            execution(),
            &Tolerances {
                variables: vec![0.001; start.len()],
                rows: vec![0.001; start.len()],
                integrality: 0.001,
            },
            None,
        )
        .unwrap()
}
fn count(report: &SolveReport, name: &str) -> i64 {
    let Metric::Integer(value) = report.metrics[name] else {
        panic!("missing {name}")
    };
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
    let Some(index) = usize::try_from(id)
        .ok()
        .and_then(|i| i.checked_sub(1))
        .filter(|i| *i < 6)
    else {
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
    assert_eq!(
        // SAFETY: live exclusively owned KLU owner; phases remains fixed during solve.
        unsafe {
            ffi::SUNLinSol_KLUSetObserver(
                session.linear,
                Some(linear_phase),
                std::ptr::from_mut(&mut phases).cast(),
            )
        },
        0
    );
    let report = solve(session, start);
    assert_eq!(
        // SAFETY: clear exact live owner's hook before stack userdata may move or drop.
        unsafe { ffi::SUNLinSol_KLUSetObserver(session.linear, None, std::ptr::null_mut()) },
        0
    );
    assert!(phases.started.iter().all(Option::is_none));
    (report, phases)
}
/// Preserve the upstream generic owner as a complete-route control. This is test
/// setup only; production sessions select the private fixed-pattern lifecycle.
fn stock_session(function: Function, config: Settings, stamp: Compatibility) -> Session {
    let mut session = Session::new(function, config, execution(), stamp).unwrap();
    if session.settings.method.linear == Linear::Klu {
        // SAFETY: all owners are exclusive and idle. Attach the replacement before
        // freeing the old owner, then restore the analytic callback reset by attach.
        unsafe {
            let linear = ffi::SUNLinSol_KLU(session.x, session.matrix, session.ctx);
            assert!(!linear.is_null());
            assert_eq!(ffi::KINSetLinearSolver(session.mem, linear, session.matrix), 0);
            assert_eq!(ffi::KINSetJacFn(session.mem, Some(jacobian)), 0);
            assert_eq!(ffi::SUNLinSolFree(session.linear), 0);
            session.linear = linear;
        }
        session.stock_klu = true;
    }
    session
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
                    let mut session = Session::new(
                        Function::Equations(Box::new(root)),
                        settings(n, interval),
                        execution(),
                        crate::solver_tests::stamp(Backend::Kinsol),
                    )
                    .unwrap();
                    let start = if matches!(shape, Shape::Logarithmic) {
                        5.
                    } else {
                        0.25
                    };
                    let started = Instant::now();
                    let initial: Vec<_> = (0..n)
                        .map(|i| start + 0.02 * f64::from(u32::try_from(i).unwrap()))
                        .collect();
                    let (report, phases) = solve_observed(&mut session, &initial);
                    let elapsed = started.elapsed();
                    assert_eq!(
                        report.termination.category,
                        Termination::Success,
                        "n={n} coupled={coupled} {shape:?} interval={interval}"
                    );
                    assert!(report.quality.as_ref().unwrap().feasible());
                    assert!(
                        report
                            .candidate
                            .as_ref()
                            .unwrap()
                            .primal
                            .iter()
                            .all(|x| (*x - 1.).abs() < 0.001)
                    );
                    assert_eq!(count(&report, "klu.symbolic.completed"), 1);
                    assert_eq!(count(&report, "klu.numeric_factor.completed"), 1);
                    eprintln!(
                        "profile n={n} coupled={coupled} shape={shape:?} interval={interval} wall_ns={} calls={:?} linear_phase_nanos={:?} setup={} symbolic={} numeric={} refactor={} solve={} domain_rejections={}",
                        elapsed.as_nanos(),
                        calls.borrow(),
                        phases.nanos,
                        count(&report, "klu.setup.completed"),
                        count(&report, "klu.symbolic.completed"),
                        count(&report, "klu.numeric_factor.completed"),
                        count(&report, "klu.refactor.completed"),
                        count(&report, "klu.solve.completed"),
                        report.evidence.callback.trial_rejections
                    );
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
        let mut session = Session::new(
            Function::Equations(Box::new(Root::new(4, coupled, Shape::Cubic, 1.))),
            settings(4, 10),
            execution(),
            stamp.clone(),
        )
        .unwrap();
        for (generation, target) in [(1_u8, 1.), (2, 1.2)] {
            let mut current = stamp.clone();
            current.data = pse_ids::ContentHash::from_bytes([generation; 32]);
            session
                .replace(
                    Function::Equations(Box::new(Root::new(4, coupled, Shape::Cubic, target))),
                    settings(4, 10),
                    current,
                )
                .unwrap();
            let (report, phases) = solve_observed(&mut session, &[0.25; 4]);
            eprintln!(
                "retained coupled={coupled} generation={generation} linear_phase_nanos={:?}",
                phases.nanos
            );
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report, "klu.symbolic.completed"), i64::from(generation == 1));
            assert_eq!(count(&report, "klu.numeric_factor.completed"), 1);
            assert!(
                report
                    .candidate
                    .unwrap()
                    .primal
                    .iter()
                    .all(|x| (*x - target).abs() < 0.001)
            );
        }
        for ordering in [0, 2] {
            assert_ne!(
                // SAFETY: fixed owners refuse ordering mutation on this live solver.
                unsafe { ffi::SUNLinSol_KLUSetOrdering(session.linear, ordering) },
                0
            );
            let report = solve(&mut session, &[0.25; 4]);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report, "klu.symbolic.completed"), 0);
            assert_eq!(count(&report, "klu.numeric_factor.completed"), 1);
        }
        assert!(
            session
                .replace(
                    Function::Equations(Box::new(Root::new(4, !coupled, Shape::Cubic, 1.))),
                    settings(4, 10),
                    stamp.clone()
                )
                .is_err()
        );
        let mut reordered = Root::new(4, coupled, Shape::Cubic, 1.);
        reordered.contract.variables.swap(0, 1);
        assert!(
            session
                .replace(
                    Function::Equations(Box::new(reordered)),
                    settings(4, 10),
                    stamp.clone()
                )
                .is_err()
        );
        let mut changed_ordering = stamp;
        changed_ordering.profile = pse_ids::ContentHash::from_bytes([87; 32]);
        assert!(!session.matches_layout(&changed_ordering));
    }
}

#[derive(Debug)]
struct Recycle {
    root: Root,
    coefficient: f64,
}
impl FixedPointOracle for Recycle {
    fn contract(&self) -> &OracleContract {
        &self.root.contract
    }
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.root.matrix.symbolic()
    }
    fn physical_tear_coordinates(&self) -> bool {
        true
    }
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for (i, value) in out.iter_mut().enumerate() {
            *value = self.root.target
                + self.coefficient * (x[i] - self.root.target)
                + self.root.coupling * (x[(i + x.len() - 1) % x.len()] - x[i]);
        }
        Ok(())
    }
    fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.map(x, out)?;
        for (value, x) in out.iter_mut().zip(x) {
            *value -= x;
        }
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
            let mut session = Session::new(
                Function::FixedPoint(Box::new(Recycle {
                    root: Root::new(n, true, Shape::Cubic, 1.),
                    coefficient,
                })),
                config,
                execution(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let report = solve(&mut session, &[0.25, 0.3, 0.35, 0.4]);
            assert_eq!(report.termination.category, Termination::Success);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report, "setup.refreshes"), 0);
            eprintln!(
                "recycle coefficient={coefficient} anderson={anderson} iterations={} evaluations={} original_max={}",
                count(&report, "KINGetNumNonlinSolvIters"),
                count(&report, "KINGetNumFuncEvals"),
                report.quality.as_ref().unwrap().normalized_max
            );
        }
    }
}

#[test]
fn repeated_banded_sparse_stock_and_krylov_complete_operation_contrast() {
    for (n, band) in [(128, 4), (512, 12)] {
        for linear in [
            Linear::Klu,
            Linear::Spgmr {
                dimension: pse_model::scalars::PositiveCount::try_new(20).unwrap(),
            },
        ] {
            let mut settings = settings(n, 10);
            settings.method.linear = linear;
            if linear.krylov().is_some() {
                settings.method.preconditioner = Preconditioner::Jacobi;
            }
            let stamp = crate::solver_tests::stamp(Backend::Kinsol);
            let mut session = stock_session(
                Function::Equations(Box::new(Root::banded(n, band, Shape::Cubic, 1.))),
                settings.clone(),
                stamp.clone(),
            );
            for (generation, target, coupling) in [(1_u8, 1., 0.1), (2, 1.2, 0.2), (3, 0.9, 0.05)] {
                let mut root = Root::banded(n, band, Shape::Cubic, target);
                root.coupling = coupling;
                let calls = root.calls.clone();
                let mut current = stamp.clone();
                current.data = pse_ids::ContentHash::from_bytes([generation; 32]);
                let started = Instant::now();
                session
                    .replace(
                        Function::Equations(Box::new(root)),
                        settings.clone(),
                        current,
                    )
                    .unwrap();
                let initial: Vec<_> = (0..n).map(|i| 0.5 + (i % 7) as f64 * 0.01).collect();
                let (report, phases) = if linear == Linear::Klu {
                    solve_observed(&mut session, &initial)
                } else {
                    (solve(&mut session, &initial), LinearPhases::default())
                };
                let elapsed = started.elapsed();
                assert_eq!(
                    report.termination.category,
                    Termination::Success,
                    "{linear:?} n={n} generation={generation}"
                );
                assert!(report.quality.as_ref().unwrap().feasible());
                let x = &report.candidate.as_ref().unwrap().primal;
                assert!(x.iter().all(|v| (*v - target).abs() < 0.001));
                // Independent original physical residual, including every band contribution.
                for i in 0..n {
                    let residual = x[i].powi(3) - target.powi(3)
                        + coupling
                            * (1..=band)
                                .map(|distance| x[i] - x[(i + n - distance) % n])
                                .sum::<f64>();
                    assert!(residual.abs() <= 0.001, "original row {i}: {residual}");
                }
                if linear == Linear::Klu {
                    assert_eq!(count(&report, "klu.symbolic.completed"), 1);
                    assert!(
                        count(&report, "klu.numeric_factor.completed") >= 1,
                        "fresh first factor every application"
                    );
                }
                eprintln!(
                    "sparse n={n} band={band} linear={linear:?} generation={generation} wall_ns={} retained_bytes={} calls={:?} native_phase_ns={:?} symbolic={:?} numeric={:?} refactor={:?} substitutions={:?}",
                    elapsed.as_nanos(),
                    session.retained_bytes(),
                    calls.borrow(),
                    phases.nanos,
                    report.metrics.get("klu.symbolic.completed"),
                    report.metrics.get("klu.numeric_factor.completed"),
                    report.metrics.get("klu.refactor.completed"),
                    report.metrics.get("klu.solve.completed")
                );
            }
        }
    }
}

/// Cheap residuals separate native analysis from expensive oracle work. Every
/// application changes coefficients and the nonuniform manufactured physical root.
#[derive(Clone, Copy, Debug)]
enum SparseShape {
    Band,
    Blocks { recycle: bool },
    Grid,
}
#[derive(Debug)]
struct Manufactured {
    contract: OracleContract,
    matrix: faer::sparse::SparseColMat<usize, f64>,
    target: Vec<f64>,
    nonlinear: bool,
    calls: Rc<RefCell<Calls>>,
}
impl Manufactured {
    fn new(shape: SparseShape, generation: usize, nonlinear: bool) -> Self {
        let n = match shape {
            SparseShape::Band => 4096,
            SparseShape::Blocks { .. } => 64 * 8,
            SparseShape::Grid => 32 * 32,
        };
        let mut root = Root::new(n, false, Shape::Cubic, 1.);
        let mut support = Vec::new();
        let coefficient = 0.03 + generation as f64 * 0.015;
        for row in 0..n {
            let mut neighbours = Vec::new();
            match shape {
                SparseShape::Band => {
                    neighbours.extend((1..=4).map(|d| (row + n - d) % n));
                }
                SparseShape::Blocks { recycle } => {
                    let block = row / 8;
                    neighbours.extend((block * 8..block * 8 + 8).filter(|c| *c != row));
                    if block > 0 || recycle {
                        neighbours.push((row + n - 8) % n);
                    }
                }
                SparseShape::Grid => {
                    if row % 32 > 0 { neighbours.push(row - 1); }
                    if row % 32 < 31 { neighbours.push(row + 1); }
                    if row >= 32 { neighbours.push(row - 32); }
                    if row + 32 < n { neighbours.push(row + 32); }
                }
            }
            support.push(faer::sparse::Triplet::new(row, row,
                1. + coefficient * neighbours.len() as f64));
            for col in neighbours {
                support.push(faer::sparse::Triplet::new(row, col, -coefficient));
            }
        }
        root.matrix = faer::sparse::SparseColMat::try_new_from_triplets(n, n, &support).unwrap();
        Self {
            contract: root.contract,
            matrix: root.matrix,
            target: (0..n).map(|i| 0.9 + (i % 19) as f64 * 0.01 + generation as f64 * 0.1).collect(),
            nonlinear,
            calls: Rc::default(),
        }
    }
    fn physical_residual(&self, x: &[f64], out: &mut [f64]) {
        out.fill(0.);
        for col in 0..x.len() {
            let difference = x[col] - self.target[col];
            for (row, value) in self.matrix.symbolic().row_idx_of_col(col)
                .zip(self.matrix.val_of_col(col)) {
                out[row] += value * difference;
            }
            if self.nonlinear {
                out[col] += x[col].powi(3) - self.target[col].powi(3);
            }
        }
    }
}
impl NleOracle for Manufactured {
    fn contract(&self) -> &OracleContract { &self.contract }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let started = Instant::now();
        self.physical_residual(x, out);
        let mut calls = self.calls.borrow_mut();
        calls.residuals += 1;
        calls.residual_nanos += started.elapsed().as_nanos();
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let started = Instant::now();
        let mut offset = 0;
        for col in 0..x.len() {
            for (row, value) in self.matrix.symbolic().row_idx_of_col(col)
                .zip(self.matrix.val_of_col(col)) {
                out[offset] = value + if self.nonlinear && row == col { 3. * x[col] * x[col] } else { 0. };
                offset += 1;
            }
        }
        let mut calls = self.calls.borrow_mut();
        calls.jacobians += 1;
        calls.jacobian_nanos += started.elapsed().as_nanos();
        Ok(())
    }
    fn jacobian_product(&mut self, x: &[f64], direction: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.calls.borrow_mut().products += 1;
        out.fill(0.);
        for col in 0..x.len() {
            for (row, value) in self.matrix.symbolic().row_idx_of_col(col)
                .zip(self.matrix.val_of_col(col)) {
                out[row] += value * direction[col];
            }
            if self.nonlinear { out[col] += 3. * x[col] * x[col] * direction[col]; }
        }
        Ok(())
    }
}
#[test]
fn fixed_pattern_short_stock_screen() {
    sparse_screen(false);
}
#[test]
fn fixed_pattern_complete_operation_contrast() {
    sparse_screen(true);
}
fn sparse_screen(qualify: bool) {
    for shape in [SparseShape::Band, SparseShape::Blocks { recycle: false },
        SparseShape::Blocks { recycle: true }, SparseShape::Grid] {
        if qualify && matches!(shape, SparseShape::Blocks { .. }) { continue; }
        for nonlinear in [false, true] {
            // One nonlinear shape is enough to expose within-call setup behaviour.
            let nonlinear_shape = if qualify { matches!(shape, SparseShape::Grid) }
                else { matches!(shape, SparseShape::Blocks { recycle: true }) };
            if nonlinear && !nonlinear_shape { continue; }
            for route in 0..if qualify { 3 } else { 2 } {
                let fixed = route == 2;
                let linear = if route == 1 { Linear::Spgmr {
                    dimension: pse_model::scalars::PositiveCount::try_new(20).unwrap(),
                } } else { Linear::Klu };
                let n = Manufactured::new(shape, 0, nonlinear).target.len();
                let mut config = settings(n, 10);
                config.method.linear = linear;
                if linear.krylov().is_some() { config.method.preconditioner = Preconditioner::Jacobi; }
                let stamp = crate::solver_tests::stamp(Backend::Kinsol);
                let function = Function::Equations(Box::new(Manufactured::new(shape, 0, nonlinear)));
                let mut session = if fixed {
                    Session::new(function, config.clone(), execution(), stamp.clone()).unwrap()
                } else { stock_session(function, config.clone(), stamp.clone()) };
                for generation in 0..if qualify { 4 } else { 3 } {
                    let root = Manufactured::new(shape, generation, nonlinear);
                    let verification = Manufactured::new(shape, generation, nonlinear);
                    let calls = root.calls.clone();
                    let mut current = stamp.clone();
                    current.data = pse_ids::ContentHash::from_bytes([generation as u8; 32]);
                    let initial: Vec<_> = (0..n).map(|i| 0.3 + (i % 7) as f64 * 0.01).collect();
                    let started = Instant::now();
                    session.replace(Function::Equations(Box::new(root)), config.clone(), current).unwrap();
                    let (report, phases) = if linear == Linear::Klu {
                        solve_observed(&mut session, &initial)
                    } else { (solve(&mut session, &initial), LinearPhases::default()) };
                    let elapsed = started.elapsed();
                    assert_eq!(report.termination.category, Termination::Success,
                        "{shape:?} nonlinear={nonlinear} {linear:?} generation={generation}");
                    assert!(report.quality.as_ref().unwrap().feasible());
                    let x = &report.candidate.as_ref().unwrap().primal;
                    assert!(x.iter().zip(&verification.target).all(|(x, root)| (x - root).abs() <= 0.001));
                    let mut residual = vec![0.; n];
                    verification.physical_residual(x, &mut residual);
                    let original_max = residual.iter().map(|r| r.abs()).fold(0., f64::max);
                    assert!(original_max <= 0.001, "original residual {original_max}");
                    if linear == Linear::Klu {
                        assert_eq!(count(&report, "klu.symbolic.completed"), i64::from(!fixed || generation == 0));
                        assert!(count(&report, "klu.numeric_factor.completed") >= 1);
                    }
                    eprintln!("sparse_screen fixed={fixed} shape={shape:?} n={n} nonlinear={nonlinear} linear={linear:?} generation={generation} replacement_solve_ns={} retained_bytes={} calls={:?} native_phase_ns={:?} symbolic={:?} numeric={:?} refactor={:?} substitutions={:?} original_max={original_max}",
                        elapsed.as_nanos(), session.retained_bytes(), calls.borrow(), phases.nanos,
                        report.metrics.get("klu.symbolic.completed"), report.metrics.get("klu.numeric_factor.completed"),
                        report.metrics.get("klu.refactor.completed"), report.metrics.get("klu.solve.completed"));
                }
            }
        }
    }
}

#[test]
fn fixed_pattern_owner_refuses_mutation_and_recovers_with_fresh_numeric() {
    let stamp = crate::solver_tests::stamp(Backend::Kinsol);
    let mut session = Session::new(
        Function::Equations(Box::new(Root::new(4, true, Shape::Cubic, 1.))),
        settings(4, 10), execution(), stamp,
    ).unwrap();
    assert_eq!(solve(&mut session, &[0.3; 4]).termination.category, Termination::Success);
    let memory = |session: &Session| {
        let mut bytes = 0;
        // SAFETY: scalar read from this exclusively owned, idle native solver.
        assert_eq!(unsafe { ffi::SUNLinSol_KLUGetMemUsage(session.linear, &raw mut bytes) }, 0);
        bytes
    };
    let full_memory = memory(&session);
    // SAFETY: these API refusals occur on live, exclusively owned native objects.
    unsafe {
        assert!(ffi::SUNLinSol_KLUGetCommon(session.linear).is_null());
        assert!(ffi::SUNLinSol_KLUGetSymbolic(session.linear).is_null());
        assert!(ffi::SUNLinSol_KLUGetNumeric(session.linear).is_null());
        for ordering in [0, 1, 2] {
            assert_ne!(ffi::SUNLinSol_KLUSetOrdering(session.linear, ordering), 0);
        }
        let data = ffi::SUNSparseMatrix_Data(session.matrix);
        let nnz = ffi::SUNSparseMatrix_NNZ(session.matrix);
        for reinit in [1, 2] {
            assert_ne!(ffi::SUNLinSol_KLUReInit(session.linear, session.matrix, nnz + 1, reinit), 0);
            assert_eq!(ffi::SUNSparseMatrix_NNZ(session.matrix), nnz);
            assert_eq!(ffi::SUNSparseMatrix_Data(session.matrix), data);
        }
        // Initialize repeats the idempotent fresh-application transition.
        assert_eq!(ffi::SUNLinSolInitialize_KLU(session.linear), 0);
        assert_eq!(ffi::SUNLinSolInitialize_KLU(session.linear), 0);
        assert_eq!(ffi::SUNLinSol_KLUBeginApplication(session.linear), 0);
    }
    let symbolic_memory = memory(&session);
    assert!(symbolic_memory > 0 && symbolic_memory < full_memory);
    let retained_symbolic = session.retained_bytes();
    let rejected_solve = |session: &Session| {
        // SAFETY: all objects are live; this is a checked numerical-validity refusal.
        assert_ne!(unsafe { ffi::SUNLinSolSolve_KLU(session.linear, session.matrix, session.x, session.us, 0.) }, 0);
    };
    rejected_solve(&session);
    let accepted = solve(&mut session, &[1.; 4]);
    assert_eq!(accepted.termination.name, "KIN_INITIAL_GUESS_OK");
    assert_eq!(count(&accepted, "klu.numeric_factor.issued"), 0);
    assert_eq!(memory(&session), symbolic_memory);
    assert_eq!(session.retained_bytes(), retained_symbolic);
    rejected_solve(&session);
    let error = session.solve(&[0.3; 4], &Controls { threads: 2, ..Controls::default() },
        &ResolvedAccuracy::verification(), execution(), &Tolerances {
            variables: vec![0.001; 4], rows: vec![0.001; 4], integrality: 0.001,
        }, None);
    assert!(error.is_err());
    assert!(!session.ready_for_retention());
    rejected_solve(&session);
    let recovered = solve(&mut session, &[0.3; 4]);
    assert_eq!(count(&recovered, "klu.symbolic.completed"), 0);
    assert_eq!(count(&recovered, "klu.numeric_factor.completed"), 1);
    assert!(session.ready_for_retention());

    // Exercise both a singular fresh factor and a failed in-application refactor.
    for refactor in [false, true] {
        // SAFETY: owned live matrix values; canonical indices are unchanged.
        unsafe {
            if !refactor { assert_eq!(ffi::SUNLinSol_KLUBeginApplication(session.linear), 0); }
            std::slice::from_raw_parts_mut(ffi::SUNSparseMatrix_Data(session.matrix), session.callback.rows.len()).fill(0.);
        }
        // SAFETY: getter and setup on these exclusively owned live objects.
        let before = unsafe { linear_work::snapshot(session.linear) }.unwrap();
        assert_ne!(unsafe { ffi::SUNLinSolSetup_KLU(session.linear, session.matrix) }, 0);
        assert_eq!(memory(&session), symbolic_memory, "failed partial factors released");
        rejected_solve(&session);
        // SAFETY: the canonical CSC and values remain exclusively session owned.
        unsafe {
            let values = std::slice::from_raw_parts_mut(ffi::SUNSparseMatrix_Data(session.matrix), session.callback.rows.len());
            for col in 0..4 {
                for offset in session.callback.columns[col]..session.callback.columns[col + 1] {
                    let offset = usize::try_from(offset).unwrap();
                    values[offset] = if session.callback.rows[offset] == col as i64 { 1. } else { 0. };
                }
            }
            assert_eq!(ffi::SUNLinSolSetup_KLU(session.linear, session.matrix), 0);
        }
        // SAFETY: scalar getter reads completed work on the idle owner.
        let after = unsafe { linear_work::snapshot(session.linear) }.unwrap();
        assert_eq!(after.symbolic_completed, before.symbolic_completed);
        assert_eq!(after.numeric_factor_completed - before.numeric_factor_completed, 1);
        assert_eq!(after.refactor_issued - before.refactor_issued, u64::from(refactor));
    }
    // A wrong native matrix is refused before use, invalidating uncertain analysis.
    // SAFETY: temporary independent native matrix, freed before this scope exits.
    unsafe {
        let foreign = ffi::SUNSparseMatrix(4, 4, session.callback.rows.len() as i64, 0, session.ctx);
        assert!(!foreign.is_null());
        assert_ne!(ffi::SUNLinSolSetup_KLU(session.linear, foreign), 0);
        ffi::SUNMatDestroy(foreign);
    }
    assert_eq!(memory(&session), 0);
    let fresh = solve(&mut session, &[0.3; 4]);
    assert_eq!(count(&fresh, "klu.symbolic.completed"), 1);
    assert_eq!(count(&fresh, "klu.numeric_factor.completed"), 1);
    let stopped_execution = execution();
    stopped_execution.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    let stopped = session.solve(&[0.3; 4], &Controls::default(),
        &ResolvedAccuracy::verification(), stopped_execution, &Tolerances {
            variables: vec![0.001; 4], rows: vec![0.001; 4], integrality: 0.001,
        }, None).unwrap();
    assert_eq!(stopped.termination.category, Termination::Cancelled);
    assert!(session.ready_for_retention(), "synchronous canceled application drained and reset");
    assert_eq!(memory(&session), symbolic_memory);
    rejected_solve(&session);
    let recovered = solve(&mut session, &[0.3; 4]);
    assert_eq!(count(&recovered, "klu.symbolic.completed"), 0);
    assert_eq!(count(&recovered, "klu.numeric_factor.completed"), 1);
}
