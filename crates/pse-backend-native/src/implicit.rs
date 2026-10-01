// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! KINSOL child execution on an already admitted outer worker. No runtime admission occurs here.
//! Native sessions are cached per worker thread and problem layout (Plan 22 Y6, L-D7): a
//! nested solve refreshes the retained SUNDIALS context, vectors and KLU analysis with the
//! call's parameters instead of allocating them again. The cache holds at most the bytes
//! the thread's job reserves for it (I14).
use crate::{
    NleOracle, OracleContract, ProblemError, Variable, kinsol, quality::Tolerances, solve::*,
};
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    implicit::{InnerSolver, Options, Problem},
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

/// Native root capability injected into generic implicit evaluation.
#[derive(Debug)]
pub struct Kinsol;
/// Budget this worker thread's KINSOL session cache at `bytes`, the amount its job or
/// session lease reserves for it (Plan 22 I14). Retained sessions beyond the budget are
/// released, least recently used first. A thread that never received a budget retains
/// no session, so no cache outlives an admitted job.
pub fn budget_sessions(bytes: usize) {
    sessions::budget(bytes);
}
/// Per-worker KINSOL sessions keyed by problem identity. A session is taken out of the
/// cache for the duration of its solve, so a nested solve on the same worker never
/// aliases it; it returns afterwards and is kept within the thread's byte budget, least
/// recently used first out. A session larger than the whole budget is never kept.
mod sessions {
    use super::{ProblemError, kinsol};
    use std::cell::{Cell, RefCell};
    /// Retained sessions, least recently used first, with their retained bytes.
    struct Cache {
        budget: usize,
        held: Vec<(pse_ids::ContentHash, kinsol::Session, usize)>,
    }
    impl Cache {
        fn bytes(&self) -> usize {
            self.held.iter().map(|(_, _, b)| *b).sum()
        }
        /// Release the least recently used sessions until the budget holds.
        fn evict(&mut self) -> Vec<kinsol::Session> {
            let mut released = Vec::new();
            while !self.held.is_empty() && self.bytes() > self.budget {
                released.push(self.held.remove(0).1);
            }
            released
        }
    }
    thread_local! {
        static SESSIONS: RefCell<Cache> = const {
            RefCell::new(Cache { budget: 0, held: Vec::new() })
        };
        static CREATED: Cell<u64> = const { Cell::new(0) };
    }
    /// Set the thread's budget. Released sessions are destroyed outside the cache borrow.
    pub(super) fn budget(bytes: usize) {
        let released = SESSIONS.with(|s| {
            let mut s = s.borrow_mut();
            s.budget = bytes;
            s.evict()
        });
        drop(released);
    }
    /// A compatible retained session refreshed with this call's function, or a new one.
    pub(super) fn take(
        key: pse_ids::ContentHash,
        function: kinsol::Function,
        settings: kinsol::Settings,
        execution: crate::solve::Execution,
        compatibility: crate::solve::Compatibility,
    ) -> Result<kinsol::Session, ProblemError> {
        let retained = SESSIONS.with(|s| {
            let mut s = s.borrow_mut();
            let held = &mut s.held;
            held.iter()
                .position(|(k, _, _)| *k == key)
                .map(|i| held.remove(i).1)
        });
        if let Some(mut session) = retained
            && session.matches_layout(&compatibility)
            && session.matches_settings(&settings)
        {
            session.replace(function, settings, compatibility)?;
            return Ok(session);
        }
        CREATED.with(|c| c.set(c.get() + 1));
        kinsol::Session::new(function, settings, execution, compatibility)
    }
    /// Return a session after its solve, counted at its current retained bytes.
    pub(super) fn give(key: pse_ids::ContentHash, session: kinsol::Session) {
        let bytes = session.retained_bytes();
        let released = SESSIONS.with(|s| {
            let mut s = s.borrow_mut();
            if bytes > s.budget {
                return vec![session];
            }
            s.held.push((key, session, bytes));
            s.evict()
        });
        drop(released);
    }
    /// Sessions this worker thread has allocated.
    #[cfg(test)]
    pub(super) fn created() -> u64 {
        CREATED.with(Cell::get)
    }
    /// Sessions this worker thread retains, and their bytes.
    #[cfg(test)]
    pub(super) fn retained() -> (usize, usize) {
        SESSIONS.with(|s| {
            let s = s.borrow();
            (s.held.len(), s.bytes())
        })
    }
}
/// The trial problem is lent to the native session for one solve only: the implicit
/// evaluator reconfigures trial problems in place and needs unique ownership back, so a
/// cached session never retains it.
type Lent = Rc<RefCell<Option<Arc<Problem>>>>;
/// Returns the lent problem on every exit path of a solve.
struct Loan(Lent);
impl Drop for Loan {
    fn drop(&mut self) {
        if let Ok(mut slot) = self.0.try_borrow_mut() {
            slot.take();
        }
    }
}
#[derive(Debug)]
struct Oracle {
    problem: Lent,
    /// The problem's structural Jacobian support, owned so the session keeps its layout
    /// after the problem is returned.
    pattern: faer::sparse::SymbolicSparseColMat<usize>,
    inputs: usize,
    parameters: Vec<f64>,
    contract: OracleContract,
    cancel: Arc<AtomicBool>,
}
impl Oracle {
    fn problem(&self) -> Result<Arc<Problem>, ProblemError> {
        self.problem
            .borrow()
            .clone()
            .ok_or_else(|| ProblemError::internal("implicit trial problem already returned"))
    }
}
impl NleOracle for Oracle {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.as_ref()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let jet =
            self.problem()?
                .evaluate(&self.parameters, x, DerivativeOrder::Value, &self.cancel)?;
        if jet.values.len() != out.len() {
            return Err(ProblemError::internal("implicit residual extent"));
        }
        out.copy_from_slice(&jet.values);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let n = self.contract.variables.len();
        let width = n + self.inputs;
        let jet =
            self.problem()?
                .evaluate(&self.parameters, x, DerivativeOrder::First, &self.cancel)?;
        let pattern = self.pattern.as_ref();
        if jet.jacobian.len() != n * width || out.len() != pattern.compute_nnz() {
            return Err(ProblemError::internal("implicit Jacobian extent"));
        }
        for (slot, (i, j)) in (0..n)
            .flat_map(|j| pattern.row_idx_of_col(j).map(move |i| (i, j)))
            .enumerate()
        {
            out[slot] = jet.jacobian[i * width + j];
        }
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let n = self.contract.variables.len();
        if direction.len() != n || out.len() != n {
            return Err(ProblemError::internal("implicit JVP extent"));
        }
        let mut values = vec![0.0; self.pattern.compute_nnz()];
        self.jacobian(x, &mut values)?;
        out.fill(0.);
        let pattern = self.pattern.as_ref();
        for (slot, (i, j)) in (0..n)
            .flat_map(|j| pattern.row_idx_of_col(j).map(move |i| (i, j)))
            .enumerate()
        {
            out[i] += values[slot] * direction[j];
        }
        Ok(())
    }
}
impl InnerSolver for Kinsol {
    fn minimum_order(&self) -> DerivativeOrder {
        crate::routing::derivative_demand(
            crate::execution::adapter(Backend::Kinsol).capability(),
            &Controls::default(),
        )
        .unwrap_or(DerivativeOrder::Value)
    }
    fn honors_operational(&self, settings: &str) -> bool {
        settings == "native.kinsol.v1"
    }
    fn identity(&self) -> pse_ids::ContentHash {
        pse_math::implicit::solver_identity("sundials.kinsol.v1")
    }
    fn solve(
        &self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<f64>, MathError> {
        problem.validate_options(options)?;
        // Typed native causes, including structural rows and columns, stay attributable.
        let map = |e: ProblemError| match e {
            ProblemError::Math(e) => e,
            ProblemError::Cancelled => MathError::Cancelled,
            other => MathError::Native {
                source_id: problem.id,
                retained: other.retained_bytes(),
                cause: Box::new(other),
            },
        };
        let controls = Controls {
            iterations: options.iterations,
            time_limit: options.time_limit,
            ..Controls::default()
        };
        // The normalized feasibility budget of the unknowns (tolerance over nominal), as
        // `ResolvedAccuracy::resolve` forms it; the function-norm test stays |r| <= tolerance.
        let accuracy = ResolvedAccuracy::from_policy(
            &Default::default(),
            options
                .variable_tolerance
                .iter()
                .zip(&options.variable_nominals)
                .map(|(t, n)| t / n)
                .fold(f64::INFINITY, f64::min),
        )
        .map_err(map)?;
        // One-sided bounds are exact (KINSOL shifts them to sign constraints); a two-sided
        // interval keeps only its sign information, and `Problem::verify` rechecks it.
        let contract = OracleContract {
            identity: problem.identity,
            variables: problem
                .unknowns
                .iter()
                .map(|u| {
                    let (lower, upper) = match (u.lower.is_finite(), u.upper.is_finite()) {
                        (true, false) => (u.lower, f64::INFINITY),
                        (false, true) => (f64::NEG_INFINITY, u.upper),
                        _ => (
                            if u.lower >= 0.0 {
                                0.0
                            } else {
                                f64::NEG_INFINITY
                            },
                            if u.upper <= 0.0 { 0.0 } else { f64::INFINITY },
                        ),
                    };
                    Variable {
                        id: u.id,
                        lower,
                        upper,
                    }
                })
                .collect(),
            rows: problem.rows.clone(),
            derivatives: problem.requirements.residual_compilation,
            smoothness: problem
                .requirements
                .output_smoothness
                .min(problem.requirements.residual_compilation),
        };
        let pattern = problem
            .pattern()
            .to_owned()
            .map_err(|_| MathError::Limit("implicit Jacobian support allocation"))?;
        let loan = Loan(Rc::new(RefCell::new(Some(problem.clone()))));
        let oracle = Oracle {
            problem: loan.0.clone(),
            pattern,
            inputs: problem.inputs,
            parameters: parameters.to_vec(),
            contract,
            cancel: cancel.clone(),
        };
        // The nested budgets come from the resolved numerical policy: residuals within
        // their tolerance and steps below the unknowns' tolerance, in original coordinates.
        let tolerances = Tolerances {
            variables: options.variable_tolerance.clone(),
            rows: options.residual_tolerance.clone(),
            integrality: f64::EPSILON,
        };
        let settings = kinsol::Settings::from_policy(
            kinsol::Method {
                setup_interval: 1,
                ..kinsol::Method::default()
            },
            &tolerances,
            &pse_math::normalization::Normalization::identity(
                problem.unknowns.len(),
                problem.rows.len(),
            ),
            accuracy.feasibility,
        );
        let compatibility = Compatibility {
            layout: problem.identity,
            profile: problem.identity,
            data: problem.identity,
            backend: Backend::Kinsol,
        };
        let execution = Execution::new(cancel.clone(), &controls);
        let mut session = sessions::take(
            problem.identity,
            kinsol::Function::Equations(Box::new(oracle)),
            settings,
            execution.clone(),
            compatibility,
        )
        .map_err(map)?;
        let report = session.solve(
            &options.start,
            &controls,
            &accuracy,
            execution,
            &tolerances,
            None,
        );
        sessions::give(problem.identity, session);
        drop(loan);
        let report = report.map_err(map)?;
        match report.termination.category {
            Termination::Cancelled => return Err(MathError::Cancelled),
            Termination::TimeLimit => return Err(MathError::Limit("implicit solve time")),
            Termination::Success | Termination::Acceptable => {}
            _ => {
                return Err(MathError::Domain {
                    source_id: problem.id,
                    requirement: "native inner solve did not converge",
                });
            }
        }
        let point = report
            .candidate
            .ok_or(MathError::Domain {
                source_id: problem.id,
                requirement: "inner solve returned no candidate",
            })?
            .primal;
        problem.verify(parameters, &point, options, cancel)?;
        Ok(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn problem(deficient: bool) -> Result<Problem, MathError> {
        problem_with(96, deficient)
    }
    fn problem_with(identity: u8, deficient: bool) -> Result<Problem, MathError> {
        problem_order(identity, deficient, DerivativeOrder::Second)
    }
    fn problem_order(
        identity: u8,
        deficient: bool,
        order: DerivativeOrder,
    ) -> Result<Problem, MathError> {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = pse_ids::SemanticId::from_bytes([96; 16]);
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut builder = pse_math::typed::BodyBuilder::new(
                pse_math::initialize().unwrap(),
                &registry,
                &pse_quantity::standard::StandardInvariantChecker,
                3,
                pse_math::typed::BodyLimits::default(),
            )
            .unwrap();
            let x = builder
                .input(0, q, pse_quantity::IndexSet::new(), id)
                .unwrap();
            let y = builder
                .input(1, q, pse_quantity::IndexSet::new(), id)
                .unwrap();
            let p = builder
                .input(2, q, pse_quantity::IndexSet::new(), id)
                .unwrap();
            let a = builder
                .binary(pse_math::typed::Binary::Sub, x.clone(), p.clone(), None, id)
                .unwrap();
            let b = builder
                .binary(
                    pse_math::typed::Binary::Sub,
                    if deficient { x } else { y },
                    p,
                    None,
                    id,
                )
                .unwrap();
            let body = Arc::new(
                builder
                    .finish(
                        &[a, b],
                        order,
                        pse_math::library::Optimization::default(),
                        &cancel,
                    )
                    .unwrap(),
            );
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([identity; 32]),
                vec![
                    pse_math::implicit::Unknown {
                        id: pse_ids::named_id(id, "x"),
                        lower: -10.,
                        upper: 10.,
                    },
                    pse_math::implicit::Unknown {
                        id: pse_ids::named_id(id, "y"),
                        lower: -10.,
                        upper: 10.,
                    },
                ],
                vec![pse_ids::named_id(id, "a"), pse_ids::named_id(id, "b")],
                1,
                body,
                100,
            )
        }
    }
    fn options() -> Options {
        Options {
            start: vec![1., 1.],
            variable_nominals: vec![1., 1.],
            variable_tolerance: vec![1e-8, 1e-8],
            residual_tolerance: vec![1e-8, 1e-8],
            iterations: 20,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        }
    }
    #[test]
    fn implicit_kinsol_first_order_contract_does_not_require_hessians() {
        let cancel = Arc::new(AtomicBool::new(false));
        let problem = Arc::new(problem_order(100, false, DerivativeOrder::First).unwrap());
        assert_eq!(Kinsol.minimum_order(), DerivativeOrder::First);
        assert_eq!(problem.compiled_order, DerivativeOrder::First);
        let root = Kinsol
            .solve(problem.clone(), &[3.], &options(), &cancel)
            .unwrap();
        assert!(root.iter().all(|x| (*x - 3.).abs() < 1e-8));
        assert_eq!(
            problem
                .derivatives(&[3.], &root, DerivativeOrder::First, &options(), &cancel)
                .unwrap()
                .jacobian,
            vec![1., 1.]
        );
        assert!(
            problem
                .derivatives(&[3.], &root, DerivativeOrder::Second, &options(), &cancel)
                .is_err()
        );
    }
    #[test]
    fn structural_failure_keeps_rows_and_columns() {
        let cancel = Arc::new(AtomicBool::new(false));
        let deficient = Arc::new(problem(true).unwrap());
        let error = Kinsol
            .solve(deficient.clone(), &[3.], &options(), &cancel)
            .unwrap_err();
        let MathError::Native {
            source_id, cause, ..
        } = &error
        else {
            panic!("untyped inner failure: {error:?}");
        };
        assert_eq!(*source_id, deficient.id);
        let Some(ProblemError::Structural { rows, columns, .. }) =
            cause.downcast_ref::<ProblemError>()
        else {
            panic!("structural identities were not retained: {cause:?}");
        };
        let y = deficient.unknowns[1].id;
        assert_eq!(columns, &vec![y]);
        assert!(!rows.is_empty() && rows.iter().all(|r| deficient.rows.contains(r)));
        assert!(error.retained_bytes() > size_of::<MathError>());
    }
    #[test]
    fn implicit_kinsol_preserves_sparse_support_and_refuses_structural_deficiency() {
        let cancel = Arc::new(AtomicBool::new(false));
        let build = problem;
        let options = Options {
            start: vec![1., 1.],
            variable_nominals: vec![1., 1.],
            variable_tolerance: vec![1e-8, 1e-8],
            residual_tolerance: vec![1e-8, 1e-8],
            iterations: 20,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        let problem = Arc::new(build(false).unwrap());
        assert_eq!(problem.pattern().compute_nnz(), 2);
        let root = Kinsol
            .solve(problem.clone(), &[3.], &options, &cancel)
            .unwrap();
        assert!(root.iter().all(|x| (*x - 3.).abs() < 1e-8));
        let jet = problem
            .derivatives(&[3.], &root, DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert_eq!(jet.jacobian, vec![1., 1.]);
        assert_eq!(jet.hessians, vec![0., 0.]);
        // Either faer's symbolic admission or the shared KINSOL structural check
        // must refuse a missing unknown column, even though row count is square.
        if let Ok(deficient) = build(true) {
            assert!(
                Kinsol
                    .solve(Arc::new(deficient), &[3.], &options, &cancel)
                    .is_err()
            );
        }
    }
    /// L-D7: repeated nested solves of one problem layout on a worker reuse its native
    /// session with each call's parameters; another layout allocates its own. The trial
    /// problem is returned to unique ownership after every solve.
    #[test]
    fn nested_inner_solver_reuses_session() {
        let cancel = Arc::new(AtomicBool::new(false));
        let mut problem = Arc::new(problem(false).unwrap());
        // The test thread starts with its own empty cache, budgeted as a job would.
        budget_sessions(16 << 20);
        let before = sessions::created();
        for p in [3.0, 5.0, -2.0, 3.0] {
            let root = Kinsol
                .solve(problem.clone(), &[p], &options(), &cancel)
                .unwrap();
            assert!(
                root.iter().all(|x| (*x - p).abs() < 1e-8),
                "{root:?} for {p}"
            );
            assert!(
                Arc::get_mut(&mut problem).is_some(),
                "a cached session kept the problem"
            );
        }
        assert_eq!(sessions::created() - before, 1);
        // A different layout identity allocates, and both are retained afterwards.
        let other = Arc::new(problem_with(97, false).unwrap());
        Kinsol
            .solve(other.clone(), &[4.0], &options(), &cancel)
            .unwrap();
        Kinsol
            .solve(problem.clone(), &[4.0], &options(), &cancel)
            .unwrap();
        Kinsol.solve(other, &[1.0], &options(), &cancel).unwrap();
        assert_eq!(sessions::created() - before, 2);
        // A cancelled call still returns its session: the next call reuses it.
        let stopped = Arc::new(AtomicBool::new(true));
        assert!(matches!(
            Kinsol.solve(problem.clone(), &[3.0], &options(), &stopped),
            Err(MathError::Cancelled)
        ));
        Kinsol.solve(problem, &[3.0], &options(), &cancel).unwrap();
        assert_eq!(sessions::created() - before, 2);
    }
    /// I14: the cache keeps sessions only within the thread's byte budget, releasing the
    /// least recently used first; a session larger than the budget is never kept, and a
    /// thread without a budget keeps none.
    #[test]
    fn inner_session_cache_bounded_by_bytes() {
        let cancel = Arc::new(AtomicBool::new(false));
        let first = Arc::new(problem_with(98, false).unwrap());
        let second = Arc::new(problem_with(99, false).unwrap());
        let solve = |p: &Arc<Problem>| Kinsol.solve(p.clone(), &[2.0], &options(), &cancel);
        // No budget: every solve allocates and nothing is retained.
        let before = sessions::created();
        solve(&first).unwrap();
        solve(&first).unwrap();
        assert_eq!(sessions::created() - before, 2);
        assert_eq!(sessions::retained(), (0, 0));
        // A generous budget keeps both layouts, counted in bytes.
        budget_sessions(16 << 20);
        solve(&first).unwrap();
        solve(&second).unwrap();
        let (count, both) = sessions::retained();
        assert_eq!(count, 2);
        // Vectors, workspaces, the sparse Jacobian and KLU's factors: well above a header.
        assert!(both > 2 * 1024, "{both}");
        // A budget for one session releases the least recently used one (the first).
        budget_sessions(both / 2 + both / 4);
        let (count, one) = sessions::retained();
        assert!(count == 1 && one <= both / 2 + both / 4, "{count} {one}");
        let before = sessions::created();
        solve(&second).unwrap();
        assert_eq!(
            sessions::created() - before,
            0,
            "the recent session was kept"
        );
        solve(&first).unwrap();
        assert_eq!(
            sessions::created() - before,
            1,
            "the released session is rebuilt"
        );
        assert_eq!(sessions::retained().0, 1);
        // A session larger than the whole budget is never kept.
        budget_sessions(one / 2);
        assert_eq!(sessions::retained(), (0, 0));
        solve(&second).unwrap();
        assert_eq!(sessions::retained(), (0, 0));
        budget_sessions(0);
    }
    #[test]
    fn implicit_kinsol_uses_physical_variable_nominals() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = pse_ids::SemanticId::from_bytes([95; 16]);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = pse_math::typed::BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            pse_math::typed::BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let residual = builder
            .binary(pse_math::typed::Binary::Sub, y, p, None, id)
            .unwrap();
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    pse_math::library::Optimization::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let problem = Arc::new(
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([95; 32]),
                vec![pse_math::implicit::Unknown {
                    id,
                    lower: 0.,
                    upper: f64::INFINITY,
                }],
                vec![pse_ids::named_id(id, "row")],
                1,
                body,
                100,
            )
            .unwrap(),
        );
        let mut options = Options {
            start: vec![1.],
            variable_nominals: vec![1e12],
            variable_tolerance: vec![1e-3],
            residual_tolerance: vec![1e-3],
            iterations: 20,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        let root = Kinsol
            .solve(problem.clone(), &[1e12], &options, &cancel)
            .unwrap();
        assert!((root[0] - 1e12).abs() < 1e-3);
        options.variable_nominals[0] = 1.;
        assert!(
            Kinsol
                .solve(problem.clone(), &[1e12], &options, &cancel)
                .is_err()
        );
        options.variable_nominals[0] = 0.;
        assert!(problem.validate_options(&options).is_err());
    }
    #[test]
    fn implicit_kinsol_rechecks_roots_and_arbitrary_interval_guards() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = pse_ids::SemanticId::from_bytes([94; 16]);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = pse_math::typed::BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            pse_math::typed::BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let square = builder
            .binary(pse_math::typed::Binary::Mul, y.clone(), y, None, id)
            .unwrap();
        let residual = builder
            .binary(pse_math::typed::Binary::Sub, square, p, None, id)
            .unwrap();
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    pse_math::library::Optimization::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let problem = Arc::new(
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([94; 32]),
                vec![pse_math::implicit::Unknown {
                    id,
                    lower: 0.5,
                    upper: 2.5,
                }],
                vec![pse_ids::named_id(id, "row")],
                1,
                body,
                100,
            )
            .unwrap(),
        );
        let options = Options {
            start: vec![1.0],
            variable_nominals: vec![2.0],
            variable_tolerance: vec![1e-9],
            residual_tolerance: vec![1e-9],
            iterations: 50,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        let root = Kinsol
            .solve(problem.clone(), &[4.0], &options, &cancel)
            .unwrap();
        assert!((root[0] - 2.0).abs() < 1e-8);
        let jet = problem
            .derivatives(&[4.0], &root, DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert!((jet.jacobian[0] - 0.25).abs() < 1e-8);
        assert!(Kinsol.solve(problem, &[9.0], &options, &cancel).is_err());
    }
}
