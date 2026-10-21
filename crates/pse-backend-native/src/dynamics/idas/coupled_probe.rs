// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private full shifted-operator inquiry; no production BlockFactor route.
use super::*;
use pse_kernels::DerivativeOrder;
use std::time::Instant;

struct Permit {
    n: usize,
    bytes: usize,
}
impl Permit {
    fn admit(n: usize, execution: &Execution) -> Result<Self, ProblemError> {
        execution.check()?;
        // FERAL 0.18 Markowitz live fill plus lazy historical heap memberships;
        // same source-qualified extent as the existing sparse-LU owner, without BTF splitting.
        let bytes = n
            .checked_mul(n)
            .and_then(|square| {
                square.checked_mul(2048).and_then(|live| {
                    square
                        .checked_mul(n)
                        .and_then(|history| history.checked_mul(256))
                        .and_then(|history| live.checked_add(history))
                })
            })
            .and_then(|bytes| {
                n.checked_mul(4096)
                    .and_then(|linear| bytes.checked_add(linear))
            })
            .and_then(|bytes| {
                bytes.checked_add(size_of::<FactorState>() + size_of::<feral::SparseLu>())
            })
            .ok_or_else(|| ProblemError::memory("private coupled LU extent"))?;
        if n == 0 || execution.memory.is_none_or(|memory| bytes > memory) {
            return Err(ProblemError::memory(
                "private coupled LU exceeds admitted storage",
            ));
        }
        if let Some(owner) = &execution.work_admission {
            owner.admit_storage(
                crate::solve::NativeStorageScope::Linear,
                "idas.private-coupled-lu",
                bytes,
                false,
            )?;
        }
        Ok(Self { n, bytes })
    }
}
#[derive(Debug)]
pub(super) struct FactorState {
    matrix: feral::SparseColMatrix,
    factor: Option<feral::SparseLu>,
    rhs: Vec<f64>,
    original: Vec<f64>,
    residual: Vec<f64>,
    row_sums: Vec<f64>,
    allowance: usize,
    setups: u64,
    solves: u64,
    factor_nanos: u128,
    solve_nanos: u128,
    maximum_fill: usize,
    maximum_backward_error: f64,
}
impl FactorState {
    fn new(permit: Permit, columns: Vec<usize>, rows: Vec<usize>) -> Result<Self, ProblemError> {
        let n = permit.n;
        if columns.len() != n + 1 || rows.len() > n * n {
            return Err(ProblemError::Contract("private coupled CSC extent".into()));
        }
        let matrix = feral::SparseColMatrix {
            m: n,
            col_ptr: columns,
            values: vec![0.; rows.len()],
            row_idx: rows,
        };
        matrix.validate().map_err(crate::conditioning::native)?;
        Ok(Self {
            matrix,
            factor: None,
            rhs: vec![0.; n],
            original: vec![0.; n],
            residual: vec![0.; n],
            row_sums: vec![0.; n],
            allowance: permit.bytes,
            setups: 0,
            solves: 0,
            factor_nanos: 0,
            solve_nanos: 0,
            maximum_fill: 0,
            maximum_backward_error: 0.,
        })
    }
    fn clear(&mut self) {
        self.factor = None;
    }
    fn setup(&mut self, values: &[f64], execution: &Execution) -> Result<(), ProblemError> {
        self.clear();
        execution.check()?;
        if values.len() != self.matrix.values.len() || values.iter().any(|value| !value.is_finite())
        {
            return Err(ProblemError::numerical("private coupled coefficients"));
        }
        self.matrix.values.copy_from_slice(values);
        let started = Instant::now();
        self.setups += 1;
        let factor = execution.counted(
            crate::solve::WorkEvidence {
                factorizations: Some(1),
                ..Default::default()
            },
            || {
                feral::SparseLu::factor_markowitz(&self.matrix, feral::LuParams::default())
                    .map_err(crate::conditioning::native)
            },
        );
        self.factor_nanos += started.elapsed().as_nanos();
        let factor = factor?;
        self.maximum_fill = self.maximum_fill.max(factor.factor_nnz());
        self.factor = Some(factor);
        Ok(())
    }
    fn apply(&mut self, rhs: &[f64], execution: &Execution) -> Result<(), ProblemError> {
        let result = (|| {
            execution.check()?;
            if rhs.len() != self.rhs.len() || rhs.iter().any(|value| !value.is_finite()) {
                return Err(ProblemError::numerical("private coupled right-hand side"));
            }
            self.rhs.copy_from_slice(rhs);
            self.original.copy_from_slice(rhs);
            let factor = self
                .factor
                .as_mut()
                .ok_or_else(|| ProblemError::internal("private coupled setup absent"))?;
            self.solves += 1;
            let started = Instant::now();
            let solved = factor
                .ftran(&mut self.rhs)
                .map_err(crate::conditioning::native);
            self.solve_nanos += started.elapsed().as_nanos();
            solved?;
            execution.check()?;
            if self.rhs.iter().any(|value| !value.is_finite()) {
                return Err(ProblemError::numerical("private coupled solution"));
            }
            self.residual
                .iter_mut()
                .zip(&self.original)
                .for_each(|(r, b)| *r = -*b);
            self.row_sums.fill(0.);
            for col in 0..self.rhs.len() {
                for entry in self.matrix.col_ptr[col]..self.matrix.col_ptr[col + 1] {
                    let row = self.matrix.row_idx[entry];
                    self.residual[row] += self.matrix.values[entry] * self.rhs[col];
                    self.row_sums[row] += self.matrix.values[entry].abs();
                }
            }
            let norm = |values: &[f64]| values.iter().map(|value| value.abs()).fold(0., f64::max);
            let denominator = norm(&self.row_sums) * norm(&self.rhs) + norm(&self.original);
            let backward = norm(&self.residual) / denominator.max(f64::MIN_POSITIVE);
            if !backward.is_finite() || backward > 0.001 {
                return Err(ProblemError::numerical(
                    "private coupled original operator residual",
                ));
            }
            self.maximum_backward_error = self.maximum_backward_error.max(backward);
            Ok(())
        })();
        if result.is_err() {
            self.clear();
        }
        result
    }
}
fn failed(c: &mut Context<'_>, error: ProblemError) -> i32 {
    c.callback.last_failure = Some(error);
    if let Some(state) = &mut c.coupled_probe {
        state.clear();
    }
    -1
}
unsafe extern "C" fn setup(
    t: f64,
    y: ffi::N_Vector,
    _: ffi::N_Vector,
    _: ffi::N_Vector,
    cj: f64,
    data: *mut c_void,
) -> i32 {
    // SAFETY: the owning native slot resolves only this active Session's current Context.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    if let Some(state) = &mut c.coupled_probe {
        state.clear();
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let execution = c.callback.execution.clone();
        if let Err(error) = execution.check() {
            return failed(c, error);
        }
        let n = c.contract.states.len();
        // SAFETY: the serial input is live for this callback; publication never mutates y.
        let x = match unsafe { callback_input(y, n) } {
            Ok(x) => x,
            Err(error) => return failed(c, error),
        };
        let Some(jacobian) = c.partials(t, x) else {
            return c.failure();
        };
        c.newton_values.fill(0.);
        for (col, &slot) in c.layout.diagonal_slots.iter().enumerate() {
            if c.contract.differential[col] {
                c.newton_values[slot] += cj;
            }
        }
        if !scatter_partials(
            jacobian.as_ref().as_ref(),
            &c.layout,
            None,
            &mut c.newton_values,
        ) {
            return failed(
                c,
                ProblemError::Contract("private coupled original support".into()),
            );
        }
        let Some(state) = &mut c.coupled_probe else {
            return failed(c, ProblemError::internal("private coupled owner absent"));
        };
        match state.setup(&c.newton_values, &execution) {
            Ok(()) => 0,
            Err(error) => failed(c, error),
        }
    }));
    let status = finish_callback(c, result);
    if status != 0
        && let Some(state) = &mut c.coupled_probe
    {
        state.clear();
    }
    status
}
unsafe extern "C" fn solve(
    _: f64,
    _: ffi::N_Vector,
    _: ffi::N_Vector,
    _: ffi::N_Vector,
    rhs: ffi::N_Vector,
    output: ffi::N_Vector,
    _: f64,
    _: f64,
    data: *mut c_void,
) -> i32 {
    // SAFETY: the registered native slot resolves the exclusive active Context.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let execution = c.callback.execution.clone();
        let n = c.contract.states.len();
        {
            // SAFETY: the native input is live, and its borrow ends before output may alias it.
            let rhs = match unsafe { callback_input(rhs, n) } {
                Ok(rhs) => rhs,
                Err(error) => return failed(c, error),
            };
            let Some(state) = &mut c.coupled_probe else {
                return failed(c, ProblemError::internal("private coupled owner absent"));
            };
            if let Err(error) = state.apply(rhs, &execution) {
                return failed(c, error);
            }
        }
        // SAFETY: output is a live n-entry native vector; all foreign input borrows ended.
        unsafe {
            write(output, &c.coupled_probe.as_ref().unwrap().rhs);
        }
        0
    }));
    let status = finish_callback(c, result);
    if status != 0
        && let Some(state) = &mut c.coupled_probe
    {
        state.clear();
    }
    status
}
fn execution() -> Execution {
    let mut execution = Execution::new(Cancellation::default(), &crate::solve::Controls::default());
    execution.memory = Some(64 << 20);
    execution
}
#[test]
fn coupled_factor_current_shift_coefficients_singular_and_cancel_control() {
    let execution = execution();
    let permit = Permit::admit(2, &execution).unwrap();
    let mut state = FactorState::new(permit, vec![0, 2, 4], vec![0, 1, 0, 1]).unwrap();
    for alpha in [1., 0.01] {
        for coupling in [0., 200.] {
            for cj in [1e-6, 1., 1e6] {
                for parameter in [1., 3.] {
                    // Independent physical M=[1,0], f=[-(p+c)x+c*z/alpha,alpha*x-z].
                    state
                        .setup(
                            &[cj + parameter + coupling, -alpha, -coupling / alpha, 1.],
                            &execution,
                        )
                        .unwrap();
                    state.apply(&[2., 3.], &execution).unwrap();
                    let x = (2. + coupling / alpha * 3.) / (cj + parameter);
                    let z = 3. + alpha * x;
                    let scale = x.abs().max(z.abs()).max(1.);
                    assert!((state.rhs[0] - x).abs() <= 0.001 * scale);
                    assert!((state.rhs[1] - z).abs() <= 0.001 * scale);
                }
            }
        }
    }
    assert!(state.setup(&[0., -1., 0., 1.], &execution).is_err());
    assert!(state.factor.is_none());
    assert!(state.apply(&[1., 2.], &execution).is_err());
    state.setup(&[2., -1., 0., 1.], &execution).unwrap();
    execution.cancellation.cancel();
    assert!(state.apply(&[1., 2.], &execution).is_err());
    assert!(state.factor.is_none());
    execution
        .cancel
        .store(false, std::sync::atomic::Ordering::Release);
    state.setup(&[2., -1., 0., 1.], &execution).unwrap();
    assert!(state.setup(&[f64::NAN, -1., 0., 1.], &execution).is_err());
    assert!(state.factor.is_none());
    state.setup(&[2., -1., 0., 1.], &execution).unwrap();
    state.apply(&[2., 3.], &execution).unwrap();
    assert_eq!(state.rhs, [1., 4.]);
    let mut denied = execution.clone();
    denied.memory = Some(1);
    assert!(Permit::admit(2, &denied).is_err());
    assert!(Permit::admit(usize::MAX, &execution).is_err());
}

#[derive(Debug)]
struct Coupled {
    contract: Contract,
    count: usize,
    alpha: f64,
    coupling: f64,
    dense: bool,
    calls: Arc<std::sync::atomic::AtomicUsize>,
    actions: Arc<std::sync::atomic::AtomicUsize>,
    fail: Arc<std::sync::atomic::AtomicBool>,
}
impl Coupled {
    fn new(count: usize, alpha: f64, coupling: f64, dense: bool) -> Self {
        let id = |i: usize| SemanticId::from_bytes(((i + 1) as u128).to_le_bytes());
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([193; 32]),
                states: (0..2 * count).map(id).collect(),
                differential: (0..2 * count).map(|i| i < count).collect(),
                parameters: vec![id(2 * count)],
                outputs: vec![id(2 * count + 1)],
                events: vec![vec![]],
                quadratures: vec![],
                balances: vec![],
                signs: vec![],
                derivatives: DerivativeOrder::First,
            },
            count,
            alpha,
            coupling,
            dense,
            calls: Arc::default(),
            actions: Arc::default(),
            fail: Arc::default(),
        }
    }
    fn neighbor(&self, row: usize, col: usize) -> bool {
        row != col && (self.dense || col == (row + self.count - 1) % self.count)
    }
    fn rhs(&self, x: &[f64], parameter: f64) -> Vec<f64> {
        let mut out = vec![0.; 2 * self.count];
        for i in 0..self.count {
            out[i] = -(parameter + self.coupling) * x[i]
                + self.coupling / self.alpha * x[self.count + i]
                + (0..self.count)
                    .filter(|&j| self.neighbor(i, j))
                    .map(|j| 0.2 * (x[j] - x[i]))
                    .sum::<f64>();
            out[self.count + i] = self.alpha * x[i] - x[self.count + i];
        }
        out
    }
}
impl Oracle for Coupled {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        let pairs = match function {
            Function::Rhs => (0..self.count)
                .flat_map(|i| {
                    let mut entries = vec![
                        (i, i),
                        (i, self.count + i),
                        (i, 2 * self.count),
                        (self.count + i, i),
                        (self.count + i, self.count + i),
                    ];
                    entries.extend(
                        (0..self.count)
                            .filter(|&j| self.neighbor(i, j))
                            .map(|j| (i, j)),
                    );
                    entries
                })
                .collect(),
            Function::Output => vec![(0, 0)],
            _ => vec![],
        };
        entries(pairs)
    }
    fn evaluate(
        &mut self,
        _: usize,
        function: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        self.calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.fail.load(std::sync::atomic::Ordering::Acquire) {
            return Err(ProblemError::numerical(
                "private coupled current source refusal",
            ));
        }
        let values = match function {
            Function::Initial => (0..2 * self.count)
                .map(|i| if i < self.count { 1. } else { self.alpha })
                .collect(),
            Function::Rhs => self.rhs(x, p[0]),
            Function::Output => vec![x[0]],
            _ => vec![],
        };
        let mut triplets = Vec::new();
        if derivatives {
            match function {
                Function::Rhs => {
                    for (i, &state) in x[..self.count].iter().enumerate() {
                        let neighbors = (0..self.count).filter(|&j| self.neighbor(i, j)).count();
                        triplets.extend([
                            (i, i, -p[0] - self.coupling - 0.2 * neighbors as f64),
                            (i, self.count + i, self.coupling / self.alpha),
                            (i, 2 * self.count, -state),
                            (self.count + i, i, self.alpha),
                            (self.count + i, self.count + i, -1.),
                        ]);
                        triplets.extend(
                            (0..self.count)
                                .filter(|&j| self.neighbor(i, j))
                                .map(|j| (i, j, 0.2)),
                        );
                    }
                }
                Function::Output => triplets.push((0, 0, 1.)),
                _ => {}
            }
        }
        let jacobian = derivatives.then(|| {
            Arc::new(
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    2 * self.count + 1,
                    &triplets
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
    fn jacobian_action(
        &mut self,
        _: usize,
        function: Function,
        _: f64,
        state: &[f64],
        parameters: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.actions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let n = 2 * self.count;
        if self.fail.load(std::sync::atomic::Ordering::Acquire)
            || state.len() != n
            || parameters.len() != 1
            || direction.len() != n + 1
            || state
                .iter()
                .chain(parameters)
                .chain(direction)
                .any(|value| !value.is_finite())
        {
            return Err(ProblemError::numerical(
                "private coupled current action input",
            ));
        }
        let candidate = match function {
            Function::Rhs => {
                let mut values = self.rhs(&direction[..n], parameters[0]);
                for i in 0..self.count {
                    values[i] -= state[i] * direction[n];
                }
                values
            }
            Function::Initial => vec![0.; n],
            Function::Output => vec![direction[0]],
            _ => vec![],
        };
        if candidate.len() != output.len() || candidate.iter().any(|value| !value.is_finite()) {
            return Err(ProblemError::numerical("private coupled action output"));
        }
        output.copy_from_slice(&candidate);
        Ok(())
    }
}
fn install(session: &mut Session<'_>, permit: Permit) {
    let columns = session
        .callback
        .layout
        .columns
        .iter()
        .map(|&v| usize::try_from(v).unwrap())
        .collect();
    let rows = session
        .callback
        .layout
        .rows
        .iter()
        .map(|&v| usize::try_from(v).unwrap())
        .collect();
    session.callback.coupled_probe = Some(FactorState::new(permit, columns, rows).unwrap());
    // SAFETY: live exclusive IDAS memory retains this Context/factor through final native return.
    check(
        unsafe { ffi::IDASetPreconditioner(session.mem, Some(setup), Some(solve)) },
        "private coupled callbacks",
    )
    .unwrap();
}
#[test]
fn coupled_probe_direct_action_matches_original_directional_residual() {
    for (count, dense) in [(1, false), (4, false), (8, true)] {
        let mut oracle = Coupled::new(count, 0.01, 200., dense);
        let n = 2 * count;
        let state = (0..n).map(|i| 0.2 + i as f64 * 0.03).collect::<Vec<_>>();
        let direction = (0..=n).map(|i| -0.4 + i as f64 * 0.02).collect::<Vec<_>>();
        let mut action = vec![0.; n];
        oracle
            .jacobian_action(0, Function::Rhs, 0., &state, &[3.], &direction, &mut action)
            .unwrap();
        let step = 1e-5;
        let plus = state
            .iter()
            .zip(&direction)
            .map(|(x, d)| x + step * d)
            .collect::<Vec<_>>();
        let minus = state
            .iter()
            .zip(&direction)
            .map(|(x, d)| x - step * d)
            .collect::<Vec<_>>();
        let plus = oracle.rhs(&plus, 3. + step * direction[n]);
        let minus = oracle.rhs(&minus, 3. - step * direction[n]);
        for ((action, plus), minus) in action.iter().zip(plus).zip(minus) {
            let independent = (plus - minus) / (2. * step);
            assert!((action - independent).abs() <= 0.001 * independent.abs().max(1.));
        }
    }
}
#[test]
fn coupled_native_callbacks_clear_failed_current_context_and_preserve_output() {
    for source_refusal in [false, true] {
        let mut oracle = Coupled::new(1, 1., 1., false);
        let fail = oracle.fail.clone();
        let execution = execution();
        let permit = Permit::admit(2, &execution).unwrap();
        let linear = IdasLinear::Spgmr {
            dimension: PositiveCount::try_new(4).unwrap(),
            preconditioner: Preconditioner::Jacobi,
        };
        let (mut session, _) = Session::prepare(
            &mut oracle,
            &[1.],
            StatePreparation {
                layout: None,
                time: 0.,
                parameters: vec![1.],
                map: vec![0],
                trial_failures: TrialPolicy::Terminal,
                rtol: 0.001,
                atol: &[0.001; 2],
                initial_step: None,
                linear,
                forward: false,
            },
            execution,
        )
        .unwrap();
        install(&mut session, permit);
        assert_eq!(
            // SAFETY: the idle session owns live two-state y/dy and its stable userdata
            // slot. setup ignores the null residual argument and only borrows y.
            unsafe {
                setup(
                    0.,
                    session.y,
                    session.dy,
                    std::ptr::null_mut(),
                    3.,
                    session.user_data(),
                )
            },
            0
        );
        let vector = session.vector(&[2., 3.]).unwrap();
        assert_eq!(
            // SAFETY: vector is the session's live two-entry serial allocation. solve
            // ends its input borrow before writing the deliberately aliased output;
            // unused residual arguments are never dereferenced, and userdata is live.
            unsafe {
                solve(
                    0.,
                    session.y,
                    session.dy,
                    std::ptr::null_mut(),
                    vector,
                    vector,
                    3.,
                    0.,
                    session.user_data(),
                )
            },
            0
        );
        // SAFETY: vector is still owned by this session.
        assert_eq!(unsafe { read(vector, 2) }, [1.25, 4.25]);
        session.callback.parameters[0] = 3.;
        assert_eq!(
            // SAFETY: no native work overlaps this call; y and the current userdata
            // slot remain session-owned after changing the parameter and shift.
            // setup ignores the supplied null residual pointer.
            unsafe {
                setup(
                    0.,
                    session.y,
                    session.dy,
                    std::ptr::null_mut(),
                    7.,
                    session.user_data(),
                )
            },
            0
        );
        // SAFETY: restore the independently owned input before the aliased native solve.
        unsafe {
            write(vector, &[2., 3.]);
        }
        assert_eq!(
            // SAFETY: the two-entry vector and current userdata slot remain live;
            // solve copies input before publishing to its aliased output and ignores
            // the null residual pointer. The previous setup completed synchronously.
            unsafe {
                solve(
                    0.,
                    session.y,
                    session.dy,
                    std::ptr::null_mut(),
                    vector,
                    vector,
                    7.,
                    0.,
                    session.user_data(),
                )
            },
            0
        );
        // SAFETY: the vector remains live and the result must reflect p=3,cj=7, not the prior factor.
        assert_eq!(unsafe { read(vector, 2) }, [0.5, 3.5]);
        if source_refusal {
            fail.store(true, std::sync::atomic::Ordering::Release);
            assert_ne!(
                // SAFETY: the source refusal flag changes no native handle lifetime.
                // setup receives this idle session's live vectors/userdata slot and
                // ignores the null residual argument before reporting the refusal.
                unsafe {
                    setup(
                        0.,
                        session.y,
                        session.dy,
                        std::ptr::null_mut(),
                        7.,
                        session.user_data(),
                    )
                },
                0
            );
        } else {
            session.callback.callback.execution.cancellation.cancel();
        }
        // SAFETY: initialize a fresh output sentinel in the live two-entry vector.
        unsafe {
            write(vector, &[777., 888.]);
        }
        assert_ne!(
            // SAFETY: the session still owns the two-entry aliased vector and
            // userdata slot. solve checks the failed current context before writing,
            // never retaining an input borrow while publishing output.
            unsafe {
                solve(
                    0.,
                    session.y,
                    session.dy,
                    std::ptr::null_mut(),
                    vector,
                    vector,
                    7.,
                    0.,
                    session.user_data(),
                )
            },
            0
        );
        assert!(
            session
                .callback
                .coupled_probe
                .as_ref()
                .unwrap()
                .factor
                .is_none()
        );
        // SAFETY: output remains live after refusal.
        assert_eq!(unsafe { read(vector, 2) }, [777., 888.]);
        // SAFETY: detach before the same session releases its factor owner.
        check(
            unsafe { ffi::IDASetPreconditioner(session.mem, None, None) },
            "private coupled detach",
        )
        .unwrap();
    }
}
#[test]
fn full_coupled_preconditioner_complete_native_inquiry_keeps_original_trajectory() {
    use pse_model::scalars::PositiveCount;
    for (count, dense) in [(1, false), (4, false), (8, true)] {
        for alpha in [1., 0.01] {
            for coupling in [1., 200.] {
                let krylov = |preconditioner| IdasLinear::Spgmr {
                    dimension: PositiveCount::try_new(20).unwrap(),
                    preconditioner,
                };
                let bbd = |width| IdasLinear::SpgmrBbd {
                    dimension: PositiveCount::try_new(20).unwrap(),
                    difference: IdasBand {
                        upper: 2 * count - 1,
                        lower: 2 * count - 1,
                    },
                    retained: IdasBand {
                        upper: width,
                        lower: width,
                    },
                };
                for (name, linear, coupled) in [
                    ("klu", IdasLinear::Klu, false),
                    ("none", krylov(Preconditioner::None), false),
                    ("jacobi", krylov(Preconditioner::Jacobi), false),
                    ("coupled-private", krylov(Preconditioner::Jacobi), true),
                    ("bbd-narrow", bbd(1), false),
                    ("bbd-complete-support", bbd(count), false),
                ] {
                    if name == "bbd-complete-support" && count == 1 {
                        continue;
                    }
                    let complete = Instant::now();
                    let mut oracle = Coupled::new(count, alpha, coupling, dense);
                    let calls = oracle.calls.clone();
                    let actions = oracle.actions.clone();
                    let execution = execution();
                    // This permit is checked before any initial oracle/native effect.
                    let permit = coupled.then(|| Permit::admit(2 * count, &execution).unwrap());
                    let profile = Profile {
                        method: Method::Idas,
                        end: 0.4,
                        samples: vec![0.1, 0.2, 0.3, 0.4],
                        parameter_scales: vec![1.],
                        rtol: 0.001,
                        atol: (0..2 * count)
                            .map(|i| if i < count { 0.001 } else { 0.001 * alpha })
                            .collect(),
                        initial_step: 1e-5,
                        idas: IdasSettings {
                            linear,
                            native_initial_step: IdasInitialStep::Configured,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    profile.validate(oracle.contract(), &[1.]).unwrap();
                    let (mut session, _) = Session::prepare(
                        &mut oracle,
                        &[1.],
                        StatePreparation {
                            layout: None,
                            time: 0.,
                            parameters: vec![1.],
                            map: vec![0],
                            trial_failures: TrialPolicy::Terminal,
                            rtol: profile.rtol,
                            atol: &profile.atol,
                            initial_step: Some(profile.initial_step),
                            linear,
                            forward: false,
                        },
                        execution,
                    )
                    .unwrap();
                    if let Some(permit) = permit {
                        install(&mut session, permit);
                    }
                    session.initialize_roots().unwrap();
                    session
                        .consistent(
                            0.,
                            &profile,
                            initialization_option(profile.idas.initialization),
                        )
                        .unwrap();
                    let preparation_ns = complete.elapsed().as_nanos();
                    let started = Instant::now();
                    let mut maximum_error: f64 = 0.;
                    for target in &profile.samples {
                        let mut returned = 0.;
                        let flag = session
                            .solve(*target, &mut returned, "idas.private-coupled-inquiry")
                            .unwrap();
                        assert!(flag >= 0, "{name} {count}/{alpha}/{coupling}: {flag}");
                        session.callback.callback.execution.check().unwrap();
                        // SAFETY: the returned serial vector is still owned by this Session.
                        let state = unsafe { read(session.y, 2 * count) };
                        let expected = (-returned).exp();
                        for i in 0..count {
                            maximum_error = maximum_error.max((state[i] - expected).abs());
                            assert!((state[i] - expected).abs() <= 0.01);
                            assert!((state[count + i] / alpha - state[i]).abs() <= 0.001);
                        }
                    }
                    let solve_ns = started.elapsed().as_nanos();
                    let factor = session.callback.coupled_probe.as_ref().map(|f| {
                        (
                            f.setups,
                            f.solves,
                            f.factor_nanos,
                            f.solve_nanos,
                            f.maximum_fill,
                            f.maximum_backward_error,
                            f.allowance,
                        )
                    });
                    let statistics = session.statistics(0, &profile, Some(0.4));
                    let teardown = Instant::now();
                    if coupled {
                        // SAFETY: detach private callbacks before their exclusively session-owned factor is released.
                        check(
                            unsafe { ffi::IDASetPreconditioner(session.mem, None, None) },
                            "private coupled detach",
                        )
                        .unwrap();
                    }
                    drop(session);
                    drop(oracle);
                    let teardown_ns = teardown.elapsed().as_nanos();
                    let complete_ns = complete.elapsed().as_nanos();
                    eprintln!(
                        "coupled-inquiry n={} dense={dense} alpha={alpha} coupling={coupling} route={name} preparation_ns={preparation_ns} solve_ns={solve_ns} teardown_ns={teardown_ns} complete_ns={complete_ns} oracle_calls={} direct_actions={} maximum_analytic_error={maximum_error} factor={factor:?} statistics={statistics}",
                        2 * count,
                        calls.load(std::sync::atomic::Ordering::Relaxed),
                        actions.load(std::sync::atomic::Ordering::Relaxed)
                    );
                }
            }
        }
    }
}
