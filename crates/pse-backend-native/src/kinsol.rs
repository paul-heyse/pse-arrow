// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "checked SUNDIALS context/vector/matrix/solver RAII and panic-contained callbacks"
)]
//! SUNDIALS KINSOL owns nonlinear iteration, globalization and Anderson acceleration.
use crate::{
    NleOracle, OracleContract, ProblemError,
    callback::CallbackState,
    quality::{Quality, Tolerances, Violation, interval},
    solve::*,
};
use std::{ffi::c_void, marker::PhantomData, rc::Rc};
use sundials_sys as ffi;
// Retain native KLU/AMD/BTF linkage even though SUNDIALS owns all calls.
use suitesparse_sys as _;
/// A declared causal map is distinct from an equation residual oracle.
pub trait FixedPointOracle: std::fmt::Debug {
    /// Exact original source variable/row order.
    fn contract(&self) -> &OracleContract;
    /// Conservative support of the original connection residual, including its identity term.
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize>;
    /// Compute the complete causal map, including admitted library-owned unit solves.
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Original physical process residuals, independent of the fixed-point iteration.
    fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
}
/// Distinct function representations prevent accidental root-to-fixed-point reformulation.
#[derive(Debug)]
pub enum Function {
    /// Residual equations with exact local derivatives.
    Equations(Box<dyn NleOracle>),
    /// Explicit Picard splitting F(x)=L*x-N(x), with the declared constant L.
    /// The original equation oracle remains the residual authority; L is never
    /// advertised as its derivative. Native KINSOL solves each linear correction.
    Picard {
        /// Original equation residual and source contract.
        oracle: Box<dyn NleOracle>,
        /// Authored constant splitting operator; not a residual derivative claim.
        linear: faer::sparse::SparseColMat<usize, f64>,
    },
    /// Explicit causal map and original residual validator.
    FixedPoint(Box<dyn FixedPointOracle>),
}
impl Function {
    fn pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        match self {
            Self::Equations(o) => Some(o.jacobian_pattern()),
            Self::Picard { linear, .. } => Some(linear.symbolic()),
            Self::FixedPoint(_) => None,
        }
    }
    /// Original source contract of this explicitly selected representation.
    pub fn contract(&self) -> &OracleContract {
        match self {
            Self::Equations(o) => o.contract(),
            Self::Picard { oracle, .. } => oracle.contract(),
            Self::FixedPoint(o) => o.contract(),
        }
    }
}
/// KINSOL nonlinear strategy, without a project-owned Newton method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    /// Declared constant linear splitting with native Anderson acceleration.
    Picard,
    /// Full Newton step.
    Newton,
    /// Globalized Newton line search.
    LineSearch,
    /// Declared fixed-point map with native Anderson acceleration.
    FixedPoint,
}
/// Selected native linear algebra. Dense allocation has an explicit dimension ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Linear {
    /// Vendored SuiteSparse KLU with analytic CSC Jacobian.
    Klu,
    /// Serial dense factorization for explicitly small systems.
    Dense {
        /// Maximum admitted dimension.
        limit: usize,
    },
    /// Matrix-free native GMRES with analytic JVP.
    Spgmr {
        /// Maximum Krylov subspace dimension.
        dimension: usize,
    },
}
/// Typed SUNDIALS-specific controls in addition to common finite limits.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Nonlinear strategy.
    pub strategy: Strategy,
    /// Native linear solver for equation profiles.
    pub linear: Linear,
    /// Positive inverse characteristic variable scales.
    pub variable_scales: Vec<f64>,
    /// Positive inverse characteristic residual scales.
    pub residual_scales: Vec<f64>,
    /// Native Anderson history; zero disables acceleration.
    pub anderson: usize,
    /// Damping in (0,1].
    pub damping: f64,
    /// Maximum nonlinear iterations between linear setups.
    pub setup_interval: u32,
    /// Positive scaled-step stopping tolerance.
    pub step_tolerance: f64,
}
impl Settings {
    /// Admit exact dimensions, sign-only bound semantics and strategy representation.
    pub fn validate(&self, function: &Function) -> Result<Vec<f64>, ProblemError> {
        let c = function.contract();
        let n = c.variables.len();
        if n == 0
            || c.rows.len() != n
            || self.variable_scales.len() != n
            || self.residual_scales.len() != n
            || self
                .variable_scales
                .iter()
                .chain(&self.residual_scales)
                .any(|v| !v.is_finite() || *v <= 0.0)
            || !self.damping.is_finite()
            || self.damping <= 0.0
            || self.damping > 1.0
            || self.setup_interval == 0
            || !self.step_tolerance.is_finite()
            || self.step_tolerance <= 0.0
            || self.anderson > n
        {
            return Err(ProblemError::Contract(
                "KINSOL square/scaling/settings contract".into(),
            ));
        }
        let fixed = matches!(function, Function::FixedPoint(_));
        if fixed != (self.strategy == Strategy::FixedPoint) {
            return Err(ProblemError::Contract(
                "KINSOL strategy requires its declared residual/map representation".into(),
            ));
        }
        if matches!(function, Function::Picard { .. }) != (self.strategy == Strategy::Picard) {
            return Err(ProblemError::Contract(
                "Picard requires an explicit constant splitting".into(),
            ));
        }
        if !fixed {
            c.square()?;
        } else {
            c.validate(pse_kernels::DerivativeOrder::Value)?;
        }
        if let Function::Picard { linear, .. } = function {
            if linear.val().iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::Contract("nonfinite Picard splitting".into()));
            }
        }
        let original = match function {
            Function::Equations(o) => o.jacobian_pattern(),
            Function::Picard { oracle, .. } => oracle.jacobian_pattern(),
            Function::FixedPoint(o) => o.original_pattern(),
        };
        crate::structural::oracle(
            c,
            original,
            &vec![(0.0, 0.0); n],
            crate::structural::Mode::Roots,
        )?;
        let signs = sign_constraints(c)?;
        if (fixed || self.strategy == Strategy::Picard) && signs.iter().any(|v| *v != 0.0) {
            return Err(ProblemError::Contract(
                "KIN_FP/KIN_PICARD forbid constraints; choose an eligible declared map or NLP"
                    .into(),
            ));
        }
        match self.linear {
            Linear::Dense { limit } if limit == 0 || n > limit => {
                return Err(ProblemError::Contract(
                    "KINSOL dense dimension limit".into(),
                ));
            }
            Linear::Spgmr { dimension } if dimension == 0 || dimension > i32::MAX as usize => {
                return Err(ProblemError::Contract("KINSOL Krylov dimension".into()));
            }
            _ => {}
        }
        Ok(signs)
    }
}
/// Convert only exactly representable sign bounds; arbitrary boxes are refused.
pub fn sign_constraints(c: &OracleContract) -> Result<Vec<f64>, ProblemError> {
    c.variables
        .iter()
        .map(|v| match (v.lower, v.upper) {
            (l, u) if l == f64::NEG_INFINITY && u == f64::INFINITY => Ok(0.0),
            (0.0, u) if u == f64::INFINITY => Ok(1.0),
            (l, 0.0) if l == f64::NEG_INFINITY => Ok(-1.0),
            _ => Err(ProblemError::Contract(
                "KINSOL sign constraints cannot encode general box bounds".into(),
            )),
        })
        .collect()
}
struct Context {
    function: Function,
    state: CallbackState,
    n: usize,
    rows: Vec<ffi::sunindextype>,
    columns: Vec<ffi::sunindextype>,
    dense: bool,
}
/// Worker-local allocation retains native layouts across compatible equation values.
pub struct Session {
    ctx: ffi::SUNContext,
    mem: *mut c_void,
    x: ffi::N_Vector,
    us: ffi::N_Vector,
    fs: ffi::N_Vector,
    signs: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    callback: Box<Context>,
    settings: Settings,
    compatibility: Compatibility,
    _local: PhantomData<Rc<()>>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KinsolSession")
            .field("compatibility", &self.compatibility)
            .field("settings", &self.settings)
            .finish_non_exhaustive()
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            if !self.mem.is_null() {
                ffi::KINFree(&raw mut self.mem)
            }
            if !self.linear.is_null() {
                ffi::SUNLinSolFree(self.linear);
            }
            if !self.matrix.is_null() {
                ffi::SUNMatDestroy(self.matrix)
            }
            for v in [self.signs, self.fs, self.us, self.x] {
                if !v.is_null() {
                    ffi::N_VDestroy(v)
                }
            }
            if !self.ctx.is_null() {
                ffi::SUNContext_Free(&raw mut self.ctx);
            }
        }
    }
}
fn check(code: i32, name: &str) -> Result<(), ProblemError> {
    if code < 0 {
        Err(ProblemError::Contract(format!("SUNDIALS {name}: {code}")))
    } else {
        Ok(())
    }
}
fn index(n: usize) -> Result<ffi::sunindextype, ProblemError> {
    ffi::sunindextype::try_from(n)
        .map_err(|_| ProblemError::Contract("SUNDIALS index overflow".into()))
}
unsafe fn values<'a>(v: ffi::N_Vector, n: usize) -> Result<&'a [f64], ProblemError> {
    if v.is_null() || unsafe { ffi::N_VGetLength(v) } != index(n)? {
        return Err(ProblemError::Contract("SUNDIALS vector dimensions".into()));
    }
    let p = unsafe { ffi::N_VGetArrayPointer(v) };
    if p.is_null() {
        return Err(ProblemError::Contract(
            "SUNDIALS null vector storage".into(),
        ));
    }
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
unsafe fn publish(v: ffi::N_Vector, from: &[f64]) -> Result<(), ProblemError> {
    unsafe { values(v, from.len()) }?;
    if from.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract(
            "nonfinite root callback output".into(),
        ));
    }
    unsafe { std::ptr::copy_nonoverlapping(from.as_ptr(), ffi::N_VGetArrayPointer(v), from.len()) };
    Ok(())
}
fn result<T>(value: Option<T>, state: &CallbackState) -> i32 {
    if value.is_some() {
        0
    } else if state.terminal.is_some() {
        -1
    } else {
        1
    }
}
unsafe extern "C" fn residual(x: ffi::N_Vector, out: ffi::N_Vector, data: *mut c_void) -> i32 {
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let n = c.n;
    let value = c.state.evaluate("residual", || {
        let x = unsafe { values(x, n) }?.to_vec();
        let mut v = vec![0.0; n];
        match &mut c.function {
            Function::Equations(o) => o.residual(&x, &mut v)?,
            Function::Picard { oracle, .. } => oracle.residual(&x, &mut v)?,
            Function::FixedPoint(o) => o.map(&x, &mut v)?,
        }
        unsafe { publish(out, &v) }
    });
    result(value, &c.state)
}
unsafe extern "C" fn jacobian(
    x: ffi::N_Vector,
    _f: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _t1: ffi::N_Vector,
    _t2: ffi::N_Vector,
) -> i32 {
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let value = c.state.evaluate("jacobian", || {
        let mut v = vec![0.0; c.rows.len()];
        match &mut c.function {
            Function::Equations(o) => o.jacobian(unsafe { values(x, c.n) }?, &mut v)?,
            Function::Picard { linear, .. } => v.copy_from_slice(linear.val()),
            Function::FixedPoint(_) => {
                return Err(ProblemError::Contract(
                    "map has no residual Jacobian".into(),
                ));
            }
        }
        if v.iter().any(|v| !v.is_finite()) || matrix.is_null() {
            return Err(ProblemError::Contract("invalid root Jacobian".into()));
        }
        if c.dense {
            check(unsafe { ffi::SUNMatZero(matrix) }, "dense zero")?;
            for col in 0..c.n {
                let p = unsafe { ffi::SUNDenseMatrix_Column(matrix, index(col)?) };
                if p.is_null() {
                    return Err(ProblemError::Contract("null dense column".into()));
                }
                for k in c.columns[col] as usize..c.columns[col + 1] as usize {
                    unsafe { *p.add(c.rows[k] as usize) = v[k] };
                }
            }
        } else {
            let p = unsafe { ffi::SUNSparseMatrix_Data(matrix) };
            if p.is_null() && !v.is_empty() {
                return Err(ProblemError::Contract("null sparse values".into()));
            }
            if !v.is_empty() {
                unsafe { std::ptr::copy_nonoverlapping(v.as_ptr(), p, v.len()) };
            }
        }
        Ok(())
    });
    result(value, &c.state)
}
unsafe extern "C" fn jvp(
    v: ffi::N_Vector,
    out: ffi::N_Vector,
    x: ffi::N_Vector,
    _new: *mut i32,
    data: *mut c_void,
) -> i32 {
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let value = c.state.evaluate("jvp", || {
        let mut y = vec![0.0; c.n];
        match &mut c.function {
            Function::Equations(o) => o.jacobian_product(
                unsafe { values(x, c.n) }?,
                unsafe { values(v, c.n) }?,
                &mut y,
            )?,
            Function::Picard { linear, .. } => {
                let result = linear.as_ref() * faer::ColRef::from_slice(unsafe { values(v, c.n) }?);
                for (i, v) in y.iter_mut().enumerate() {
                    *v = result[i];
                }
            }
            Function::FixedPoint(_) => {
                return Err(ProblemError::Contract("map has no residual JVP".into()));
            }
        }
        unsafe { publish(out, &y) }
    });
    result(value, &c.state)
}
impl Session {
    /// Check the immutable compiler/backend layout stamp before constructing an update.
    pub fn matches_layout(&self, stamp: &Compatibility) -> bool {
        self.compatibility.layout == stamp.layout && stamp.backend == Backend::Kinsol
    }
    /// Replace compatible numeric equations while retaining native allocation and
    /// symbolic layout. The next solve explicitly refreshes numeric setup.
    pub fn replace(
        &mut self,
        function: Function,
        compatibility: Compatibility,
    ) -> Result<(), ProblemError> {
        let signs = self.settings.validate(&function)?;
        if compatibility.layout != self.compatibility.layout
            || compatibility.backend != Backend::Kinsol
            || function.contract().rows != self.callback.function.contract().rows
            || function.contract().variables.iter().map(|v| v.id).ne(self
                .callback
                .function
                .contract()
                .variables
                .iter()
                .map(|v| v.id))
        {
            return Err(ProblemError::Contract(
                "KINSOL replacement changes layout".into(),
            ));
        }
        match function.pattern() {
            Some(p) => {
                let rows: Vec<_> = (0..p.ncols())
                    .flat_map(|c| p.row_idx_of_col(c))
                    .map(index)
                    .collect::<Result<_, _>>()?;
                let mut columns = vec![0];
                let mut count = 0;
                for c in 0..p.ncols() {
                    count += p.row_idx_of_col(c).len();
                    columns.push(index(count)?);
                }
                if rows != self.callback.rows || columns != self.callback.columns {
                    return Err(ProblemError::Contract(
                        "KINSOL replacement changes sparse layout".into(),
                    ));
                }
            }
            None => {}
        }
        unsafe {
            if signs.iter().any(|v| *v != 0.0) {
                if self.signs.is_null() {
                    self.signs = ffi::N_VNew_Serial(index(signs.len())?, self.ctx);
                }
                publish(self.signs, &signs)?;
                check(
                    ffi::KINSetConstraints(self.mem, self.signs),
                    "updated sign constraints",
                )?;
            } else {
                check(
                    ffi::KINSetConstraints(self.mem, std::ptr::null_mut()),
                    "clear sign constraints",
                )?;
            }
        }
        self.callback.function = function;
        self.compatibility = compatibility;
        Ok(())
    }
    /// Construct native context, serial vectors and the selected analytic linear route.
    pub fn new(
        function: Function,
        settings: Settings,
        execution: Execution,
        compatibility: Compatibility,
    ) -> Result<Self, ProblemError> {
        let signs = settings.validate(&function)?;
        let n = function.contract().variables.len();
        let (rows, columns) = match function.pattern() {
            Some(p) => {
                if p.nrows() != n || p.ncols() != n {
                    return Err(ProblemError::Contract("root Jacobian dimensions".into()));
                }
                let mut rows = Vec::new();
                let mut cols = vec![0];
                for c in 0..n {
                    let mut last = None;
                    for r in p.row_idx_of_col(c) {
                        if r >= n || last.is_some_and(|p| p >= r) {
                            return Err(ProblemError::Contract(
                                "root Jacobian canonical structure".into(),
                            ));
                        }
                        last = Some(r);
                        rows.push(index(r)?);
                    }
                    cols.push(index(rows.len())?);
                }
                (rows, cols)
            }
            None => (vec![], vec![]),
        };
        let callback = Box::new(Context {
            function,
            state: CallbackState::new(execution),
            n,
            rows,
            columns,
            dense: matches!(settings.linear, Linear::Dense { .. }),
        });
        let mut s = Self {
            ctx: std::ptr::null_mut(),
            mem: std::ptr::null_mut(),
            x: std::ptr::null_mut(),
            us: std::ptr::null_mut(),
            fs: std::ptr::null_mut(),
            signs: std::ptr::null_mut(),
            matrix: std::ptr::null_mut(),
            linear: std::ptr::null_mut(),
            callback,
            settings,
            compatibility,
            _local: PhantomData,
        };
        unsafe {
            check(ffi::SUNContext_Create(0, &raw mut s.ctx), "context")?;
            s.x = ffi::N_VNew_Serial(index(n)?, s.ctx);
            s.us = ffi::N_VNew_Serial(index(n)?, s.ctx);
            s.fs = ffi::N_VNew_Serial(index(n)?, s.ctx);
            if s.x.is_null() || s.us.is_null() || s.fs.is_null() {
                return Err(ProblemError::Contract("SUNDIALS vector allocation".into()));
            }
            publish(s.us, &s.settings.variable_scales)?;
            publish(s.fs, &s.settings.residual_scales)?;
            s.mem = ffi::KINCreate(s.ctx);
            if s.mem.is_null() {
                return Err(ProblemError::Contract("KINSOL allocation".into()));
            }
            check(
                ffi::KINSetMAA(s.mem, s.settings.anderson as _),
                "Anderson history",
            )?;
            check(ffi::KINInit(s.mem, Some(residual), s.x), "initialization")?;
            check(
                ffi::KINSetUserData(s.mem, (&raw mut *s.callback).cast()),
                "callback context",
            )?;
            if s.settings.strategy != Strategy::FixedPoint {
                match s.settings.linear {
                    Linear::Klu => {
                        s.matrix = ffi::SUNSparseMatrix(
                            index(n)?,
                            index(n)?,
                            index(s.callback.rows.len())?,
                            0,
                            s.ctx,
                        );
                        if s.matrix.is_null() {
                            return Err(ProblemError::Contract("sparse matrix allocation".into()));
                        }
                        std::ptr::copy_nonoverlapping(
                            s.callback.columns.as_ptr(),
                            ffi::SUNSparseMatrix_IndexPointers(s.matrix),
                            n + 1,
                        );
                        if !s.callback.rows.is_empty() {
                            std::ptr::copy_nonoverlapping(
                                s.callback.rows.as_ptr(),
                                ffi::SUNSparseMatrix_IndexValues(s.matrix),
                                s.callback.rows.len(),
                            );
                        }
                        s.linear = ffi::SUNLinSol_KLU(s.x, s.matrix, s.ctx);
                    }
                    Linear::Dense { .. } => {
                        s.matrix = ffi::SUNDenseMatrix(index(n)?, index(n)?, s.ctx);
                        if s.matrix.is_null() {
                            return Err(ProblemError::Contract("dense matrix allocation".into()));
                        }
                        s.linear = ffi::SUNLinSol_Dense(s.x, s.matrix, s.ctx);
                    }
                    Linear::Spgmr { dimension } => {
                        s.linear = ffi::SUNLinSol_SPGMR(s.x, 0, dimension as i32, s.ctx);
                    }
                }
                if s.linear.is_null() {
                    return Err(ProblemError::Contract(
                        "native linear solver allocation".into(),
                    ));
                }
                check(
                    ffi::KINSetLinearSolver(s.mem, s.linear, s.matrix),
                    "linear solver",
                )?;
                if matches!(s.settings.linear, Linear::Spgmr { .. }) {
                    check(ffi::KINSetJacTimesVecFn(s.mem, Some(jvp)), "analytic JVP")?;
                } else {
                    check(ffi::KINSetJacFn(s.mem, Some(jacobian)), "analytic Jacobian")?;
                }
            }
            if signs.iter().any(|v| *v != 0.0) {
                s.signs = ffi::N_VNew_Serial(index(n)?, s.ctx);
                publish(s.signs, &signs)?;
                check(ffi::KINSetConstraints(s.mem, s.signs), "sign constraints")?;
            }
        }
        Ok(s)
    }
    /// Run with an explicit start. A reused session refreshes its numeric setup;
    /// KLU owns symbolic reuse and numeric factorization decisions.
    pub fn solve(
        &mut self,
        initial: &[f64],
        controls: &Controls,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        controls.validate()?;
        if controls.threads != 1 || !controls.options.is_empty() {
            return Err(ProblemError::Contract(
                "serial KINSOL uses typed Settings and one core".into(),
            ));
        }
        let n = self.callback.n;
        tolerances.validate(n, n)?;
        if initial.len() != n {
            return Err(ProblemError::Contract("root start dimensions".into()));
        }
        let start = if let Some(w) = warm {
            w.validate(&self.compatibility)?;
            let WarmPayload::Root(x) = &w.payload else {
                return Err(ProblemError::Contract("root warm payload".into()));
            };
            if x.len() != n {
                return Err(ProblemError::Contract("root seed dimensions".into()));
            }
            x.as_slice()
        } else {
            initial
        };
        self.callback.state = CallbackState::new(execution.clone());
        unsafe {
            publish(self.x, start)?;
            check(
                ffi::KINSetNumMaxIters(self.mem, controls.iterations.into()),
                "iteration limit",
            )?;
            check(
                ffi::KINSetFuncNormTol(self.mem, controls.tolerance),
                "residual tolerance",
            )?;
            check(
                ffi::KINSetScaledStepTol(self.mem, self.settings.step_tolerance),
                "step tolerance",
            )?;
            check(
                ffi::KINSetDamping(self.mem, self.settings.damping),
                "damping",
            )?;
            check(
                ffi::KINSetDampingAA(self.mem, self.settings.damping),
                "Anderson damping",
            )?;
            check(
                ffi::KINSetMaxSetupCalls(self.mem, self.settings.setup_interval.into()),
                "setup interval",
            )?;
            check(ffi::KINSetNoInitSetup(self.mem, 0), "refresh numeric setup")?;
        }
        let strategy = match self.settings.strategy {
            Strategy::Picard => 2,
            Strategy::Newton => 0,
            Strategy::LineSearch => 1,
            Strategy::FixedPoint => 3,
        };
        let code = unsafe { ffi::KINSol(self.mem, self.x, strategy, self.us, self.fs) };
        let mut report = SolveReport::new(
            Backend::Kinsol,
            self.callback.function.contract(),
            termination(code),
            &execution,
        );
        macro_rules! count{($($get:ident),*)=>{$(let mut v=0;if unsafe{ffi::$get(self.mem,&mut v)}==0{report.metrics.insert(stringify!($get).into(),Metric::Integer(v.into()));})*}}
        count!(
            KINGetNumNonlinSolvIters,
            KINGetNumFuncEvals,
            KINGetNumBetaCondFails,
            KINGetNumBacktrackOps
        );
        if self.settings.strategy != Strategy::FixedPoint {
            count!(
                KINGetNumJacEvals,
                KINGetNumLinFuncEvals,
                KINGetNumPrecEvals,
                KINGetNumPrecSolves,
                KINGetNumLinIters,
                KINGetNumLinConvFails,
                KINGetNumJtimesEvals,
                KINGetLastLinFlag
            );
        }
        macro_rules! real{($($get:ident),*)=>{$(let mut v=0.0;if unsafe{ffi::$get(self.mem,&mut v)}==0{report.metrics.insert(stringify!($get).into(),Metric::Real(v));})*}}
        real!(KINGetFuncNorm, KINGetStepLength);
        report.provenance.insert(
            "native".into(),
            "SUNDIALS 7.1.1; serial vectors; SuiteSparse 7.7.0 KLU".into(),
        );
        report
            .provenance
            .insert("settings".into(), format!("{:?}", self.settings));
        self.callback.state.finish(&mut report);
        let x = unsafe { values(self.x, n) }?.to_vec();
        if x.iter().all(|v| v.is_finite()) {
            let mut f = vec![0.0; n];
            let evaluation = crate::quality::contained(|| match &mut self.callback.function {
                Function::Equations(o) => o.residual(&x, &mut f),
                Function::Picard { oracle, .. } => oracle.residual(&x, &mut f),
                Function::FixedPoint(o) => o.original_residual(&x, &mut f),
            });
            match evaluation {
                Ok(()) => {
                    let c = self.callback.function.contract();
                    let rows = c
                        .rows
                        .iter()
                        .zip(&f)
                        .zip(&tolerances.rows)
                        .map(|((id, v), t)| Violation {
                            id: *id,
                            physical: v.abs(),
                            tolerance: *t,
                        })
                        .collect();
                    let bounds = c
                        .variables
                        .iter()
                        .zip(&x)
                        .zip(&tolerances.variables)
                        .map(|((v, x), t)| Violation {
                            id: v.id,
                            physical: interval(*x, v.lower, v.upper),
                            tolerance: *t,
                        })
                        .collect();
                    match Quality::new(rows, bounds, vec![]) {
                        Ok(quality) => {
                            report.termination.assurance =
                                if quality.feasible() && matches!(code, 0..=2) {
                                    Assurance::Feasible
                                } else {
                                    Assurance::None
                                };
                            report.quality = Some(quality);
                            let observation = match &self.callback.function {
                                Function::Equations(o) => o.observe(f.clone()),
                                Function::Picard { oracle, .. } => oracle.observe(f.clone()),
                                Function::FixedPoint(_) => {
                                    crate::quality::Observation::from_values(
                                        None,
                                        f.clone(),
                                        vec![(0.0, 0.0); f.len()],
                                    )
                                }
                            };
                            match observation {
                                Ok(o) => report.observation = Some(o),
                                Err(e) => {
                                    report.validation_error = Some(e.to_string());
                                    report.termination.assurance = Assurance::None;
                                }
                            }
                        }
                        Err(e) => {
                            report.validation_error = Some(e.to_string());
                            report.termination.assurance = Assurance::None;
                        }
                    }
                }
                Err(e) => {
                    report.validation_error = Some(e.to_string());
                    report.termination.assurance = Assurance::None
                }
            }
            report.candidate = Some(Candidate {
                primal: x.clone(),
                objective: None,
                row_dual: None,
                bound_dual: None,
                reduced_costs: None,
                slacks: None,
            });
            report.warm_start = Some(WarmStart {
                compatibility: self.compatibility.clone(),
                payload: WarmPayload::Root(x),
            });
        } else {
            report.termination.assurance = Assurance::None
        }
        Ok(report)
    }
}
/// Preserve KINSOL's distinct residual, step-size and initialization exits.
pub fn termination(code: i32) -> NativeTermination {
    let (name, category) = match code {
        0 => ("KIN_SUCCESS", Termination::Success),
        1 => ("KIN_INITIAL_GUESS_OK", Termination::Success),
        2 => ("KIN_STEP_LT_STPTOL", Termination::Acceptable),
        -1 => ("KIN_MEM_NULL", Termination::Invalid),
        -2 => ("KIN_ILL_INPUT", Termination::Invalid),
        -3 => ("KIN_NO_MALLOC", Termination::Invalid),
        -4 => ("KIN_MEM_FAIL", Termination::Limit),
        -5 => ("KIN_LINESEARCH_NONCONV", Termination::Numerical),
        -6 => ("KIN_MAXITER_REACHED", Termination::Limit),
        -7 => ("KIN_MXNEWT_5X_EXCEEDED", Termination::Numerical),
        -8 => ("KIN_LINESEARCH_BCFAIL", Termination::Numerical),
        -9 => ("KIN_LINSOLV_NO_RECOVERY", Termination::Numerical),
        -10 => ("KIN_LINIT_FAIL", Termination::Numerical),
        -11 => ("KIN_LSETUP_FAIL", Termination::Numerical),
        -12 => ("KIN_LSOLVE_FAIL", Termination::Numerical),
        -13 => ("KIN_SYSFUNC_FAIL", Termination::Evaluation),
        -14 => ("KIN_FIRST_SYSFUNC_ERR", Termination::Evaluation),
        -15 => ("KIN_REPTD_SYSFUNC_ERR", Termination::Evaluation),
        -16 => ("KIN_VECTOROP_ERR", Termination::Numerical),
        -17 => ("KIN_CONTEXT_ERR", Termination::Invalid),
        _ => ("KIN_UNKNOWN", Termination::Invalid),
    };
    NativeTermination {
        code: i64::from(code),
        name: name.into(),
        message: None,
        category,
        assurance: Assurance::None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn abi_and_native_entrypoints() {
        assert_eq!(size_of::<ffi::sunrealtype>(), 8);
        assert_eq!(size_of::<ffi::sunindextype>(), 8);
        std::hint::black_box(ffi::SUNLinSol_KLU);
        std::hint::black_box(ffi::KINSetJacTimesVecFn);
        for i in -17..=2 {
            assert_ne!(termination(i).name, "KIN_UNKNOWN");
        }
    }
    #[test]
    fn general_boxes_are_not_sign_constraints() {
        let mut c = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([0; 32]),
            variables: vec![crate::Variable {
                id: pse_ids::SemanticId::from_bytes([0; 16]),
                lower: 0.0,
                upper: f64::INFINITY,
            }],
            rows: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        assert_eq!(sign_constraints(&c).unwrap(), vec![1.0]);
        c.variables[0].upper = 1.0;
        assert!(sign_constraints(&c).is_err());
    }
    fn settings() -> Settings {
        Settings {
            strategy: Strategy::LineSearch,
            linear: Linear::Klu,
            variable_scales: vec![1.0],
            residual_scales: vec![1.0],
            anderson: 0,
            damping: 1.0,
            setup_interval: 10,
            step_tolerance: 1e-8,
        }
    }
    #[test]
    fn native_allocation_reuses_checked_patterns_and_refuses_arbitrary_boxes() {
        let f = Function::Equations(Box::new(crate::solver_tests::Polynomial::new()));
        let mut s = Session::new(
            f,
            settings(),
            crate::solver_tests::execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        let address = s.mem;
        s.replace(
            Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        assert_eq!(address, s.mem);
        let mut o = crate::solver_tests::Polynomial::new();
        o.c.variables[0].upper = 3.0;
        assert!(
            s.replace(
                Function::Equations(Box::new(o)),
                crate::solver_tests::stamp(Backend::Kinsol)
            )
            .is_err()
        );
    }
    #[test]
    fn picard_requires_explicit_splitting_and_forbids_sign_constraints() {
        let mut profile = settings();
        profile.strategy = Strategy::Picard;
        let o = crate::solver_tests::Polynomial::new();
        assert!(profile.validate(&Function::Equations(Box::new(o))).is_err());
        let o = crate::solver_tests::Polynomial::new();
        let linear = o.matrix.clone();
        let f = Function::Picard {
            oracle: Box::new(o),
            linear,
        };
        assert!(profile.validate(&f).is_ok());
        let _s = Session::new(
            f,
            profile.clone(),
            crate::solver_tests::execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        let mut o = crate::solver_tests::Polynomial::new();
        o.c.variables[0].lower = 0.0;
        let linear = o.matrix.clone();
        assert!(
            profile
                .validate(&Function::Picard {
                    oracle: Box::new(o),
                    linear
                })
                .is_err()
        );
    }
}
