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
                + self.coupling * (1..=self.band)
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
                + self.coupling * (1..=self.band)
                    .map(|distance| direction[i] - direction[(i + x.len() - distance) % x.len()])
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
            assert_eq!(count(&report, "klu.symbolic.completed"), 1);
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
            assert_eq!(
                // SAFETY: documented KLU ordering choices on the exclusively owned
                // live solver; next KINSol still performs its required initialization.
                unsafe { ffi::SUNLinSol_KLUSetOrdering(session.linear, ordering) },
                0
            );
            let report = solve(&mut session, &[0.25; 4]);
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(count(&report, "klu.symbolic.completed"), 1);
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


fn retained_settings(n: usize) -> Settings {
    let mut settings = settings(n, 10);
    settings.method.linear = Linear::KluRetainedSymbolic;
    settings
}

/// Fill the exclusively owned native CSC with a nonsingular identity (stored
/// structural zeros remain part of the exact pattern).
fn native_identity(session: &Session, diagonal: f64) {
    let n = session.callback.n;
    // SAFETY: session owns this live n-column CSC and its allocated index/value arrays.
    unsafe {
        let pointers = std::slice::from_raw_parts(ffi::SUNSparseMatrix_IndexPointers(session.matrix), n + 1);
        let nnz = usize::try_from(pointers[n]).unwrap();
        let rows = std::slice::from_raw_parts(ffi::SUNSparseMatrix_IndexValues(session.matrix), nnz);
        let values = std::slice::from_raw_parts_mut(ffi::SUNSparseMatrix_Data(session.matrix), nnz);
        for col in 0..n {
            for k in usize::try_from(pointers[col]).unwrap()..usize::try_from(pointers[col + 1]).unwrap() {
                values[k] = if rows[k] == i64::try_from(col).unwrap() { diagonal } else { 0. };
            }
        }
    }
}
fn native_setup(session: &Session) -> i32 {
    // SAFETY: this session exclusively owns the live compatible CSC and KLU solver.
    unsafe { ffi::SUNLinSolSetup_KLU(session.linear, session.matrix) }
}
fn native_work(session: &Session) -> ffi::SUNLinearSolver_KLUWork {
    // SAFETY: read-only snapshot of this session's live KLU owner.
    unsafe { linear_work::snapshot(session.linear) }.unwrap()
}
fn native_invalidate(session: &Session) {
    // SAFETY: the live KLU owner is exclusively borrowed between native operations.
    assert_eq!(unsafe { ffi::SUNLinSol_KLUInvalidateNumeric(session.linear) }, 0);
}
fn native_solve(session: &Session) -> i32 {
    // SAFETY: owned live matrix and equal-length serial vectors; this call is synchronous.
    unsafe { ffi::SUNLinSolSolve_KLU(session.linear, session.matrix, session.x, session.fs, 0.) }
}

#[test]
fn retained_symbolic_exact_common_inputs_and_csc_key_qualify_reuse() {
    let session = Session::new(
        Function::Equations(Box::new(Root::new(4, true, Shape::Cubic, 1.))),
        retained_settings(4), execution(), crate::solver_tests::stamp(Backend::Kinsol),
    ).unwrap();
    native_identity(&session, 1.);
    assert_ne!(native_solve(&session), 0, "no current numerical setup");
    assert_eq!(native_setup(&session), 0);
    let baseline = native_work(&session);
    assert_eq!(baseline.symbolic_completed, 1);
    assert_eq!(baseline.numeric_factor_completed, 1);
    // SAFETY: read-only extent and live Common access between exclusive native calls.
    let common = unsafe { ffi::SUNLinSol_KLUGetCommon(session.linear) };
    assert!(!common.is_null());
    // SAFETY: capacity is a pure checked query; live-key size does not mutate the owner.
    assert_eq!(unsafe { ffi::SUNLinSol_KLUSymbolicKeyBytes(session.linear) }, unsafe {
        ffi::SUNLinSol_KLUSymbolicKeyCapacity(4, 8)
    });
    for input in 0..9 {
        native_invalidate(&session);
        // SAFETY: these are documented KLU input settings on the exclusively owned Common.
        unsafe {
            match input {
                0 => (*common).tol = 0.2,
                1 => (*common).memgrow = 1.3,
                2 => (*common).initmem_amd = 1.4,
                3 => (*common).initmem = 1.5,
                4 => (*common).maxwork = 10.,
                5 => (*common).btf = 0,
                6 => (*common).ordering = 0,
                7 => (*common).scale = 1,
                8 => (*common).maxwork = -0.0,
                _ => unreachable!(),
            }
        }
        let before = native_work(&session);
        assert_eq!(native_setup(&session), 0, "input={input}");
        let after = native_work(&session);
        assert_eq!(after.symbolic_completed - before.symbolic_completed, 1, "input={input}");
        assert_eq!(after.numeric_factor_completed - before.numeric_factor_completed, 1);
        native_invalidate(&session);
        let before = native_work(&session);
        assert_eq!(native_setup(&session), 0);
        let after = native_work(&session);
        assert_eq!(after.symbolic_completed, before.symbolic_completed);
        assert_eq!(after.numeric_factor_completed - before.numeric_factor_completed, 1);
    }
    // Exact floating bits distinguish -0 from +0 even though KLU treats both identically.
    native_invalidate(&session);
    // SAFETY: as above, mutate only documented inputs/outputs between native calls.
    unsafe { (*common).maxwork = 0.0; }
    let before = native_work(&session);
    assert_eq!(native_setup(&session), 0);
    assert_eq!(native_work(&session).symbolic_completed - before.symbolic_completed, 1);
    native_invalidate(&session);
    // SAFETY: rcond is output-only and grants no compatibility premise.
    unsafe { (*common).rcond = 0.25; }
    let before = native_work(&session);
    assert_eq!(native_setup(&session), 0);
    assert_eq!(native_work(&session).symbolic_completed, before.symbolic_completed);

    // A different occupied, canonical CSC pattern must reanalyze even with unchanged nnz.
    native_invalidate(&session);
    // SAFETY: four columns and eight indices belong exclusively to this live test CSC.
    unsafe {
        std::slice::from_raw_parts_mut(ffi::SUNSparseMatrix_IndexPointers(session.matrix), 5)
            .copy_from_slice(&[0, 2, 4, 6, 8]);
        std::slice::from_raw_parts_mut(ffi::SUNSparseMatrix_IndexValues(session.matrix), 8)
            .copy_from_slice(&[0, 2, 1, 3, 0, 2, 1, 3]);
    }
    native_identity(&session, 2.);
    let before = native_work(&session);
    assert_eq!(native_setup(&session), 0);
    assert_eq!(native_work(&session).symbolic_completed - before.symbolic_completed, 1);
    assert_eq!(native_solve(&session), 0);
    // A changed key between setup and solve cannot authorize a stale substitution.
    // SAFETY: live Common input mutation between synchronous native operations.
    unsafe { (*common).scale = 2; }
    assert_ne!(native_solve(&session), 0);
    assert_eq!(native_setup(&session), 0);

    unsafe extern "C" fn unsupported_order(
        _: i64, _: *mut i64, _: *mut i64, _: *mut i64, _: *mut ffi::klu_l_common,
    ) -> i64 {
        -1
    }
    for refusal in 0..4 {
        native_invalidate(&session);
        // SAFETY: only eligibility fields change; userdata is never invoked or dereferenced.
        unsafe {
            match refusal {
                0 => (*common).halt_if_singular = 0,
                1 => (*common).ordering = 2,
                2 => (*common).user_data = std::ptr::from_ref(&session).cast_mut().cast(),
                _ => (*common).user_order = Some(unsupported_order),
            }
        }
        let before = native_work(&session);
        assert_ne!(native_setup(&session), 0);
        let refused = native_work(&session);
        assert_eq!(refused.symbolic_issued, before.symbolic_issued);
        assert_eq!(refused.numeric_factor_issued, before.numeric_factor_issued);
        assert_ne!(native_solve(&session), 0);
        // SAFETY: restore the candidate's complete supported eligibility premise.
        unsafe { (*common).halt_if_singular = 1; (*common).ordering = 1; (*common).user_data = std::ptr::null_mut(); (*common).user_order = None; }
        assert_eq!(native_setup(&session), 0);
    }
    // Initialization independently clears numerical validity while retaining the key.
    // SAFETY: exclusively owned live KLU linear solver.
    assert_eq!(unsafe { ffi::SUNLinSolInitialize_KLU(session.linear) }, 0);
    assert_ne!(native_solve(&session), 0);
    let before = native_work(&session);
    assert_eq!(native_setup(&session), 0);
    assert_eq!(native_work(&session).symbolic_completed, before.symbolic_completed);
    // ReInit explicitly relinquishes both factor objects and the key.
    // SAFETY: canonical live matrix; partial reinit reallocates no matrix storage.
    assert_eq!(unsafe { ffi::SUNLinSol_KLUReInit(session.linear, session.matrix, 8,
        i32::try_from(ffi::SUNKLU_REINIT_PARTIAL).unwrap()) }, 0);
    // SAFETY: read-only query of the same owner after reset.
    assert_eq!(unsafe { ffi::SUNLinSol_KLUSymbolicKeyBytes(session.linear) }, 0);
    // SAFETY: Common remains live after ReInit, while both KLU factor owners were freed.
    assert_eq!(unsafe { (*common).memusage }, 0, "all tracked KLU factor allocation released");
    let before = native_work(&session);
    assert_eq!(native_setup(&session), 0);
    assert_eq!(native_work(&session).symbolic_completed - before.symbolic_completed, 1);
    native_invalidate(&session);
    native_identity(&session, 0.);
    let before = native_work(&session);
    assert_ne!(native_setup(&session), 0);
    let failed = native_work(&session);
    assert_eq!(failed.numeric_factor_issued - before.numeric_factor_issued, 1);
    assert_eq!(failed.numeric_factor_completed, before.numeric_factor_completed);
    assert_ne!(native_solve(&session), 0);
    native_identity(&session, 1.);
    assert_eq!(native_setup(&session), 0);
    assert_eq!(native_work(&session).symbolic_completed, failed.symbolic_completed);
    assert_eq!(native_solve(&session), 0);
    // A singular refresh inside one application fails refactor, then recovery
    // must perform a fresh factor rather than trusting the damaged numeric object.
    native_identity(&session, 0.);
    let before = native_work(&session);
    assert_ne!(native_setup(&session), 0);
    let failed = native_work(&session);
    assert_eq!(failed.refactor_issued - before.refactor_issued, 1);
    assert_eq!(failed.refactor_completed, before.refactor_completed);
    assert_ne!(native_solve(&session), 0);
    native_identity(&session, 1.);
    assert_eq!(native_setup(&session), 0);
    let recovery = native_work(&session);
    assert_eq!(recovery.numeric_factor_completed - failed.numeric_factor_completed, 1);
    assert_eq!(recovery.symbolic_completed, failed.symbolic_completed);
    assert_eq!(native_solve(&session), 0);
}

#[test]
fn retained_symbolic_accepted_start_cancel_pressure_and_release_boundaries() {
    let n = 8;
    let mut fixed_settings = retained_settings(n);
    fixed_settings.method.strategy = Strategy::FixedPoint;
    assert!(matches!(Session::new(
        Function::FixedPoint(Box::new(Recycle { root: Root::new(n, false, Shape::Cubic, 1.), coefficient: 0.5 })),
        fixed_settings, execution(), crate::solver_tests::stamp(Backend::Kinsol),
    ), Err(ProblemError::Unsupported(_))), "fixed-point execution owns no native linear solver");
    let new_session = |execution| Session::new(
        Function::Equations(Box::new(Root::new(n, true, Shape::Cubic, 1.))),
        retained_settings(n), execution, crate::solver_tests::stamp(Backend::Kinsol),
    );
    let mut pressure = execution();
    pressure.memory = Some(1);
    assert!(new_session(pressure).is_err());
    // Repeated owner destruction exercises exact key/native factor release.
    for _ in 0..4 {
        let mut session = new_session(execution()).unwrap();
        let report = solve(&mut session, &[0.25; 8]);
        assert!(report.quality.as_ref().unwrap().feasible());
        assert_eq!(count(&report, "klu.symbolic.completed"), 1);
        let held = session.retained_bytes();
        let accepted = solve(&mut session, &[1.; 8]);
        assert_eq!(accepted.termination.name, "KIN_INITIAL_GUESS_OK");
        assert_eq!(count(&accepted, "klu.setup.completed"), 0);
        // SAFETY: read-only live owner getters between solves.
        assert!(unsafe { ffi::SUNLinSol_KLUGetNumeric(session.linear) }.is_null());
        // SAFETY: symbolic state remains exclusively owned by this session.
        assert!(!unsafe { ffi::SUNLinSol_KLUGetSymbolic(session.linear) }.is_null());
        assert!(session.retained_bytes() < held, "numeric allocation was released");
        let cancelled_execution = execution();
        cancelled_execution.cancel.store(true, std::sync::atomic::Ordering::Release);
        let report = session.solve(&[0.25; 8], &Controls::default(),
            &ResolvedAccuracy::from_policy(&pse_model::numerics::NumericalPolicy::default(), 0.001).unwrap(),
            cancelled_execution, &Tolerances { variables: vec![0.001; n], rows: vec![0.001; n], integrality: 0.001 }, None).unwrap();
        assert_eq!(report.termination.category, Termination::Cancelled);
        assert_ne!(native_solve(&session), 0);
        let recovery = solve(&mut session, &[0.25; 8]);
        assert!(recovery.quality.as_ref().unwrap().feasible());
        assert_eq!(count(&recovery, "klu.symbolic.completed"), 0);
        assert_eq!(count(&recovery, "klu.numeric_factor.completed"), 1);
    }
}

#[test]
fn repeated_banded_sparse_stock_retained_and_krylov_complete_operation_contrast() {
    for (n, band) in [(128, 4), (512, 12)] {
        for linear in [Linear::Klu, Linear::KluRetainedSymbolic, Linear::Spgmr { dimension: pse_model::scalars::PositiveCount::try_new(20).unwrap() }] {
            let mut settings = settings(n, 10);
            settings.method.linear = linear;
            if linear.krylov().is_some() { settings.method.preconditioner = Preconditioner::Jacobi; }
            let stamp = crate::solver_tests::stamp(Backend::Kinsol);
            let mut session = Session::new(Function::Equations(Box::new(Root::banded(n, band, Shape::Cubic, 1.))),
                settings.clone(), execution(), stamp.clone()).unwrap();
            for (generation, target, coupling) in [(1_u8, 1., 0.1), (2, 1.2, 0.2), (3, 0.9, 0.05)] {
                let mut root = Root::banded(n, band, Shape::Cubic, target);
                root.coupling = coupling;
                let calls = root.calls.clone();
                let mut current = stamp.clone(); current.data = pse_ids::ContentHash::from_bytes([generation; 32]);
                let started = Instant::now();
                session.replace(Function::Equations(Box::new(root)), settings.clone(), current).unwrap();
                let initial: Vec<_> = (0..n).map(|i| 0.5 + (i % 7) as f64 * 0.01).collect();
                let (report, phases) = if linear.is_klu() { solve_observed(&mut session, &initial) }
                    else { (solve(&mut session, &initial), LinearPhases::default()) };
                let elapsed = started.elapsed();
                assert_eq!(report.termination.category, Termination::Success, "{linear:?} n={n} generation={generation}");
                assert!(report.quality.as_ref().unwrap().feasible());
                let x = &report.candidate.as_ref().unwrap().primal;
                assert!(x.iter().all(|v| (*v - target).abs() < 0.001));
                // Independent original physical residual, including every band contribution.
                for i in 0..n {
                    let residual = x[i].powi(3) - target.powi(3) + coupling * (1..=band)
                        .map(|distance| x[i] - x[(i + n - distance) % n]).sum::<f64>();
                    assert!(residual.abs() <= 0.001, "original row {i}: {residual}");
                }
                if linear.is_klu() {
                    let expected = i64::from(linear == Linear::Klu || generation == 1);
                    assert_eq!(count(&report, "klu.symbolic.completed"), expected);
                    assert!(count(&report, "klu.numeric_factor.completed") >= 1, "fresh first factor every application");
                }
                eprintln!("sparse n={n} band={band} linear={linear:?} generation={generation} wall_ns={} retained_bytes={} calls={:?} native_phase_ns={:?} symbolic={:?} numeric={:?} refactor={:?} substitutions={:?}", elapsed.as_nanos(), session.retained_bytes(), calls.borrow(), phases.nanos,
                    report.metrics.get("klu.symbolic.completed"), report.metrics.get("klu.numeric_factor.completed"), report.metrics.get("klu.refactor.completed"), report.metrics.get("klu.solve.completed"));
            }
        }
    }
}
