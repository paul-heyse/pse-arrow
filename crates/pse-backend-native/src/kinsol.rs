// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "checked SUNDIALS context/vector/matrix/solver RAII and panic-contained callbacks"
)]
//! SUNDIALS KINSOL owns nonlinear iteration, globalization and Anderson acceleration.
//! Krylov variants, forcing terms, preconditioning, Newton-step limits and Anderson
//! settings are typed [`Method`] fields (Plan 22 Y6); one-sided bounds are shifted to
//! native sign constraints.
#[cfg(test)]
mod grouping_inquiry;
mod linear_work;
#[cfg(test)]
mod policy_tests;
#[cfg(test)]
mod profile_tests;
mod restarts;
use crate::{
    NleOracle, OracleContract, ProblemError,
    callback::CallbackState,
    quality::{Quality, Tolerances, Violation, interval},
    solve::*,
};
pub use restarts::RestartSchedule;
use std::{
    ffi::{c_long, c_void},
    marker::PhantomData,
    rc::Rc,
};
use sundials_sys as ffi;
mod block;
// Retain native KLU/AMD/BTF linkage even though SUNDIALS owns all calls.
use suitesparse_sys as _;

/// Selected native source/configuration/provider fingerprint. The deployed executable's
/// separate file observation binds the actual static SUNDIALS bytes; this fingerprint
/// does not independently attest archive bytes. Missing qualified provider evidence
/// leaves direct native operation available but cannot authorize a child handshake.
pub fn build() -> Option<pse_ids::ContentHash> {
    let profile = NativeProfile {
        sundials_version: [
            ffi::SUNDIALS_VERSION_MAJOR,
            ffi::SUNDIALS_VERSION_MINOR,
            ffi::SUNDIALS_VERSION_PATCH,
        ],
        klu_version: [
            suitesparse_sys::KLU_MAIN_VERSION,
            suitesparse_sys::KLU_SUB_VERSION,
            suitesparse_sys::KLU_SUBSUB_VERSION,
        ],
        abi_widths: [
            u64::try_from(size_of::<ffi::sunrealtype>()).ok()?,
            u64::try_from(size_of::<ffi::sunindextype>()).ok()?,
        ],
        metadata: [
            ffi::SUNDIALS_VERSION,
            ffi::SUNDIALS_VERSION_LABEL,
            ffi::SUN_C_COMPILER,
            ffi::SUN_C_COMPILER_VERSION,
            ffi::SUN_C_COMPILER_FLAGS,
            ffi::SUN_BUILD_TYPE,
            ffi::SUN_TPL_LIST,
            ffi::SUN_KLU_VERSION,
        ],
    };
    profile_identity(
        include_bytes!(concat!(env!("OUT_DIR"), "/kinsol-klu-provider.json")),
        include_str!(concat!(env!("OUT_DIR"), "/kinsol-source.txt")),
        &profile,
    )
}
#[derive(Clone)]
struct NativeProfile<'a> {
    sundials_version: [u32; 3],
    klu_version: [u32; 3],
    abi_widths: [u64; 2],
    // Typed bindgen byte arrays, including their native terminators. In particular
    // SUN_KLU_VERSION may be empty; the linked KLU header version remains explicit.
    metadata: [&'a [u8]; 8],
}
fn profile_identity(
    provider: &[u8],
    source: &str,
    profile: &NativeProfile<'_>,
) -> Option<pse_ids::ContentHash> {
    if source.trim().is_empty() {
        return None;
    }
    // The build helper binds a coherent admitted prefix. Native admission owns
    // receipt/file validation; this hook does not independently inspect archives.
    // Refuse absent/malformed embedded evidence without inventing a fallback pin.
    let document: serde_json::Value = serde_json::from_slice(provider).ok()?;
    for field in ["identity", "files"] {
        if !document
            .get(field)?
            .as_object()
            .is_some_and(|values| !values.is_empty())
        {
            return None;
        }
    }
    let mut hasher = pse_ids::FramedHasher::new(pse_ids::Frame::NativeBuildV1);
    hasher.str("kinsol-profile.v1").part(provider).str(source);
    for value in profile
        .sundials_version
        .into_iter()
        .chain(profile.klu_version)
    {
        hasher.u64(u64::from(value));
    }
    for value in profile.abi_widths {
        hasher.u64(value);
    }
    for value in profile.metadata {
        hasher.part(value);
    }
    Some(hasher.finish_hash())
}

#[cfg(test)]
mod build_tests {
    use super::*;
    const PROVIDER: &[u8] = br#"{"identity":{"source":"qualified-provider"},"files":{"lib/libklu.a":"native-admission-digest"}}"#;
    fn profile() -> NativeProfile<'static> {
        NativeProfile {
            sundials_version: [7, 1, 1],
            klu_version: [2, 3, 3],
            abi_widths: [8, 8],
            metadata: [
                b"version\0",
                b"label\0",
                b"compiler\0",
                b"compiler-version\0",
                b"flags\0",
                b"build-type\0",
                b"TPL\0",
                b"\0",
            ],
        }
    }
    #[test]
    fn native_profile_identity_requires_provider_and_source_evidence() {
        let profile = profile();
        for provider in [
            b"".as_slice(),
            b"not json",
            b"null",
            b"[]",
            b"{}",
            br#"{"identity":{},"files":{"lib":1}}"#,
            br#"{"identity":{"source":1},"files":{}}"#,
        ] {
            assert!(profile_identity(provider, "declared source", &profile).is_none());
        }
        assert!(profile_identity(PROVIDER, " \n", &profile).is_none());
        assert!(profile_identity(PROVIDER, "declared source", &profile).is_some());
    }
    #[test]
    fn native_profile_identity_binds_all_native_inputs_without_host_source_provenance() {
        let original = profile();
        let identity = profile_identity(PROVIDER, "declared source", &original).unwrap();
        assert_eq!(
            profile_identity(PROVIDER, "declared source", &original),
            Some(identity)
        );
        assert_ne!(
            profile_identity(PROVIDER, "changed source", &original),
            Some(identity)
        );
        let changed_provider = br#"{"identity":{"source":"different-provider"},"files":{"lib/libklu.a":"native-admission-digest"}}"#;
        assert_ne!(
            profile_identity(changed_provider, "declared source", &original),
            Some(identity)
        );
        let changed_file = br#"{"identity":{"source":"qualified-provider"},"files":{"lib/libklu.a":"changed-native-file"}}"#;
        assert_ne!(
            profile_identity(changed_file, "declared source", &original),
            Some(identity)
        );
        for index in 0..3 {
            let mut changed = original.clone();
            changed.sundials_version[index] += 1;
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
            let mut changed = original.clone();
            changed.klu_version[index] += 1;
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
        }
        for index in 0..2 {
            let mut changed = original.clone();
            changed.abi_widths[index] += 1;
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
        }
        for index in 0..8 {
            let mut changed = original.clone();
            changed.metadata[index] = b"changed\0";
            assert_ne!(
                profile_identity(PROVIDER, "declared source", &changed),
                Some(identity)
            );
        }
    }
}

#[cfg(test)]
mod native_build_tests {
    use super::*;
    #[test]
    fn admitted_native_profile_has_concrete_adapter_identity() {
        let identity = build().expect("native controls require the managed qualified KLU prefix");
        assert_eq!(
            crate::execution::adapter(Backend::Kinsol).build(),
            Some(identity)
        );
    }
}

/// A declared causal map is distinct from an equation residual oracle.
pub trait FixedPointOracle: std::fmt::Debug {
    /// Exact original source variable/row order.
    fn contract(&self) -> &OracleContract;
    /// Conservative support of the original connection residual, including its identity term.
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize>;
    /// Compute the complete causal map, including admitted library-owned unit solves.
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError>;
    /// Scientific producer's actual frozen context; opaque maps may leave it unavailable.
    fn inner_context(&self) -> RawMapInnerContext {
        RawMapInnerContext::default()
    }
    /// Whether the producer supplies physical tear coordinates and row budgets.
    fn physical_tear_coordinates(&self) -> bool {
        false
    }
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
pub use crate::settings::kinsol::{Eta, Linear, Method, Orthogonalization, Strategy};
/// The native `KINSetOrthAA` code of an orthogonalization.
const fn orthogonalization_code(orthogonalization: Orthogonalization) -> i32 {
    match orthogonalization {
        Orthogonalization::ModifiedGramSchmidt => ffi::KIN_ORTH_MGS,
        Orthogonalization::InverseCompactWy => ffi::KIN_ORTH_ICWY,
        Orthogonalization::ClassicalGramSchmidt2 => ffi::KIN_ORTH_CGS2,
        Orthogonalization::DelayedClassicalGramSchmidt2 => ffi::KIN_ORTH_DCGS2,
    }
}
/// Typed SUNDIALS-specific controls in addition to common finite limits.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Caller-selected method controls.
    pub method: Method,
    /// Positive inverse characteristic variable scales.
    pub variable_scales: Vec<f64>,
    /// Positive inverse characteristic residual scales.
    pub residual_scales: Vec<f64>,
    /// Nonnegative native scaled-step stagnation tolerance; zero selects the
    /// SUNDIALS roundoff-scaled default, not a zero physical acceptance budget.
    pub step_tolerance: f64,
}
/// Native sign constraints and the coordinate shift that makes each one-sided bound a
/// sign bound: KINSOL iterates `u = x - offset`, so `x >= l` becomes `u >= 0` and
/// `x <= h` becomes `u <= 0`. Jacobians are unchanged; KINSOL's relative
/// stagnation test uses the shifted iterate magnitude.
#[derive(Clone, Debug, PartialEq)]
pub struct Constraints {
    /// Native codes 0, +-1, +-2 on the shifted coordinates.
    pub signs: Vec<f64>,
    /// Original-coordinate shifts; zero unless a nonzero one-sided bound is shifted.
    pub offsets: Vec<f64>,
}
impl Constraints {
    /// Whether any coordinate is constrained.
    pub fn any(&self) -> bool {
        self.signs.iter().any(|v| *v != 0.0)
    }
}
fn shifted(u: &[f64], offsets: &[f64], sign: f64) -> Vec<f64> {
    u.iter().zip(offsets).map(|(u, o)| u + sign * o).collect()
}
fn shifted_into(u: &[f64], offsets: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
    if u.len() != offsets.len() || out.len() != u.len() {
        return Err(ProblemError::internal("shifted root coordinates extent"));
    }
    for ((out, value), offset) in out.iter_mut().zip(u).zip(offsets) {
        *out = value + offset;
    }
    if out.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(ProblemError::numerical("nonfinite shifted root coordinate"))
    }
}
impl Settings {
    /// The one policy-derived KINSOL configuration (F22). `tolerances` are the original
    /// physical acceptance budgets of the solved coordinates and rows, `normalization` the
    /// coordinate transport of the function KINSOL evaluates (identity for an original-
    /// coordinate function) and `feasibility` the normalized budget used as KINSOL's
    /// function-norm tolerance. Each scale is `feasibility * s / t`, so the native
    /// residual test reduces to the original row budget `|r| <= t`. The scaled-step
    /// tolerance is a separate stagnation control: its denominator includes inverse
    /// variable scale plus the current shifted iterate magnitude. It is not an
    /// absolute physical step budget, nor does its exit establish convergence.
    /// The native roundoff default (zero sentinel) keeps this independent of the
    /// resolved physical feasibility scalar.
    #[must_use]
    pub fn from_policy(
        method: Method,
        tolerances: &Tolerances,
        normalization: &pse_math::normalization::Normalization,
        feasibility: f64,
    ) -> Self {
        let scales = |budgets: &[f64], coordinates: &[f64]| {
            budgets
                .iter()
                .zip(coordinates)
                .map(|(t, s)| feasibility * s / t)
                .collect()
        };
        Self {
            method,
            variable_scales: scales(&tolerances.variables, &normalization.variables),
            residual_scales: scales(&tolerances.rows, &normalization.rows),
            step_tolerance: 0.0,
        }
    }
    /// Caller-selected method controls of these settings.
    #[must_use]
    pub fn method(&self) -> Method {
        self.method
    }
    /// Admit exact dimensions, one-sided bound semantics and strategy representation.
    pub fn validate(&self, function: &Function) -> Result<Constraints, ProblemError> {
        let c = function.contract();
        if let Function::Equations(oracle) = function {
            oracle
                .operations()
                .admit(c, self.method.consumes_directional_only())?;
        }
        let representation = match function {
            Function::Equations(_) => Strategy::LineSearch,
            Function::Picard { .. } => Strategy::Picard,
            Function::FixedPoint(_) => Strategy::FixedPoint,
        };
        let guards = match function {
            Function::Equations(o) | Function::Picard { oracle: o, .. } => o.guard_signs(),
            Function::FixedPoint(_) => Default::default(),
        };
        let constraints = self.validate_contract(c, representation, &guards)?;
        if let Function::Picard { linear, .. } = function
            && (linear.nrows() != c.variables.len()
                || linear.ncols() != c.variables.len()
                || linear.val().iter().any(|v| !v.is_finite()))
        {
            return Err(ProblemError::Contract(
                "Picard splitting dimensions or nonfinite values".into(),
            ));
        }
        let original = match function {
            Function::Equations(o) => o.jacobian_pattern(),
            Function::Picard { oracle, .. } => oracle.jacobian_pattern(),
            Function::FixedPoint(o) => o.original_pattern(),
        };
        let analysis = match function {
            Function::Equations(o) | Function::Picard { oracle: o, .. } => o.structural_analysis(),
            Function::FixedPoint(_) => None,
        };
        crate::structural::check(
            c,
            original,
            &vec![(0.0, 0.0); c.variables.len()],
            crate::structural::Mode::Roots,
            analysis,
        )?;
        Ok(constraints)
    }
    /// Check strategy, scales, bounds and guards before an evaluator or native worker exists.
    pub fn validate_contract(
        &self,
        c: &OracleContract,
        representation: Strategy,
        guards: &std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign>,
    ) -> Result<Constraints, ProblemError> {
        let n = c.variables.len();
        let m = &self.method;
        if n == 0
            || c.rows.len() != n
            || self.variable_scales.len() != n
            || self.residual_scales.len() != n
            || self
                .variable_scales
                .iter()
                .chain(&self.residual_scales)
                .any(|v| !v.is_finite() || *v <= 0.0)
            || m.setup_interval == 0
            || !self.step_tolerance.is_finite()
            || self.step_tolerance < 0.0
            || m.anderson > n
        {
            return Err(ProblemError::Contract(
                "KINSOL square/scaling/settings contract".into(),
            ));
        }
        let fixed = representation == Strategy::FixedPoint;
        if fixed != (m.strategy == Strategy::FixedPoint) {
            return Err(ProblemError::Contract(
                "KINSOL strategy requires its declared residual/map representation".into(),
            ));
        }
        if (representation == Strategy::Picard) != (m.strategy == Strategy::Picard) {
            return Err(ProblemError::Contract(
                "Picard requires an explicit constant splitting".into(),
            ));
        }
        if !matches!(representation, Strategy::FixedPoint | Strategy::Picard)
            && !m.consumes_directional_only()
        {
            c.square()?;
        } else {
            c.validate(pse_kernels::DerivativeOrder::Value)?;
        }
        let constraints = guarded_sign_constraints(c, guards)?;
        if (fixed || m.strategy == Strategy::Picard) && constraints.any() {
            return Err(ProblemError::Unsupported(
                "KIN_FP/KIN_PICARD forbid constraints; choose an eligible declared map or NLP"
                    .into(),
            ));
        }
        m.validate()?;
        if let Linear::Dense { limit } = m.linear
            && n > limit.into_inner()
        {
            return Err(ProblemError::Unsupported(
                "KINSOL dense dimension limit".into(),
            ));
        }
        Ok(constraints)
    }
}
/// Represent one-sided bounds exactly as sign constraints on shifted coordinates
/// (`x >= l` becomes `u = x - l >= 0`, `x <= h` becomes `u = x - h <= 0`); two-sided boxes
/// are refused.
pub fn sign_constraints(c: &OracleContract) -> Result<Constraints, ProblemError> {
    let (signs, offsets) = c
        .variables
        .iter()
        .map(|v| match (v.lower, v.upper) {
            (l, u) if l == f64::NEG_INFINITY && u == f64::INFINITY => Ok((0.0, 0.0)),
            (l, u) if l.is_finite() && u == f64::INFINITY => Ok((1.0, l)),
            (l, u) if l == f64::NEG_INFINITY && u.is_finite() => Ok((-1.0, u)),
            _ => Err(ProblemError::Unsupported(
                "KINSOL sign constraints cannot encode two-sided box bounds".into(),
            )),
        })
        .collect::<Result<(Vec<_>, Vec<_>), _>>()?;
    Ok(Constraints { signs, offsets })
}
/// Check guard and bound intersection without constructing an oracle. A bound strictly
/// inside the guarded half-line keeps its shift; otherwise the guard is at least as
/// strong and is enforced unshifted.
pub fn guarded_sign_constraints(
    c: &OracleContract,
    guards: &std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign>,
) -> Result<Constraints, ProblemError> {
    let mut constraints = sign_constraints(c)?;
    for (i, v) in c.variables.iter().enumerate() {
        let Some(guard) = guards.get(&v.id) else {
            continue;
        };
        let direction = if guard.positive { 1.0 } else { -1.0 };
        let code = direction * if guard.strict { 2.0 } else { 1.0 };
        let (sign, offset) = (constraints.signs[i], constraints.offsets[i]);
        if sign != 0.0 && sign != direction {
            // The bound faces the guard: empty or a two-sided interval.
            return Err(if offset * direction <= 0.0 {
                ProblemError::Contract("bound/guard sign conflict".into())
            } else {
                ProblemError::Unsupported(
                    "KINSOL sign constraints cannot encode a guarded two-sided interval".into(),
                )
            });
        }
        if sign == 0.0 || offset * direction <= 0.0 {
            constraints.signs[i] = code;
            constraints.offsets[i] = 0.0;
        }
    }
    Ok(constraints)
}
#[derive(Clone, Debug)]
struct SetupStamp {
    point: Vec<f64>,
    data: pse_ids::ContentHash,
}
fn root_progress_layout(
    function: &Function,
    role: pse_diagnostics::observation::ProducerRole,
) -> Option<std::sync::Arc<RootEvaluationLayout>> {
    (!matches!(function, Function::FixedPoint(_))).then(|| {
        RootEvaluationLayout::new(
            function.contract(),
            matches!(function, Function::Picard { .. }),
            role,
        )
    })
}
struct Context {
    function: Function,
    state: CallbackState,
    progress_layout: Option<std::sync::Arc<RootEvaluationLayout>>,
    n: usize,
    /// Prepared private coordinates and complete numerical output. No caller
    /// buffer is touched until the entire callback succeeds.
    point: Vec<f64>,
    output: Vec<f64>,
    rows: Vec<ffi::sunindextype>,
    columns: Vec<ffi::sunindextype>,
    dense: bool,
    /// Original-coordinate shift of KINSOL's iterate (one-sided bounds).
    offsets: Vec<f64>,
    /// Inverse Jacobi diagonal from the last preconditioner setup.
    inverse_diagonal: Vec<f64>,
    /// Library BTF factors, never an accepted-point response factor.
    blocks: Option<block::Blocks>,
    /// Last actual matrix/preconditioner setup, not an accepted-point response factor.
    setup: Option<SetupStamp>,
    setup_calls: u64,
    data: pse_ids::ContentHash,
    raw_map: Option<RawMapCapture>,
    /// Read-only source-owned linear handle; never queried concurrently.
    klu: ffi::SUNLinearSolver,
    klu_baseline: Option<ffi::SUNLinearSolver_KLUWork>,
    klu_producer: Option<pse_diagnostics::observation::Producer>,
    klu_phases: [Option<pse_diagnostics::observation::PhaseGuard>; 6],
    klu_observer_installed: bool,
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
    retention_ready: bool,
    #[cfg(test)]
    stock_klu: bool,
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
        linear_work::clear(&mut self.callback);
        // Every object below was created by this session and is freed exactly once, the
        // solver memory first so nothing native refers to the rest afterwards.
        if !self.mem.is_null() {
            // SAFETY: the session's live KINSOL memory; `KINFree` nulls the pointer.
            unsafe { ffi::KINFree(&raw mut self.mem) }
        }
        if !self.linear.is_null() {
            // SAFETY: the session's linear solver, no longer attached to any memory.
            unsafe { ffi::SUNLinSolFree(self.linear) };
        }
        if !self.matrix.is_null() {
            // SAFETY: the session's matrix, freed after the solver that used it.
            unsafe { ffi::SUNMatDestroy(self.matrix) }
        }
        for v in [self.signs, self.fs, self.us, self.x] {
            if !v.is_null() {
                // SAFETY: each of the session's serial vectors, listed once.
                unsafe { ffi::N_VDestroy(v) }
            }
        }
        if !self.ctx.is_null() {
            // SAFETY: the session's context, freed last, after every object created in it.
            unsafe { ffi::SUNContext_Free(&raw mut self.ctx) };
        }
    }
}
fn check(code: i32, name: &str) -> Result<(), ProblemError> {
    if code < 0 {
        Err(ProblemError::Internal(format!("SUNDIALS {name}: {code}")))
    } else {
        Ok(())
    }
}
/// Call KINSOL or SUNDIALS and refuse a negative flag, as [`check`] does.
///
/// Every use passes live native objects of one session: its `SUNContext`, KINSOL memory,
/// matrix, linear solver and serial vectors, all freed only when the session drops. Calls
/// run on the session's owning thread, and out-parameters are caller storage that outlives
/// the call.
macro_rules! native {
    ($call:expr, $operation:expr $(,)?) => {{
        // SAFETY: the macro's contract: live session-owned native objects on the owning
        // thread, with caller storage that outlives the call.
        check(unsafe { $call }, $operation)
    }};
}
fn index(n: usize) -> Result<ffi::sunindextype, ProblemError> {
    ffi::sunindextype::try_from(n)
        .map_err(|_| ProblemError::Unsupported("SUNDIALS index overflow".into()))
}
#[allow(
    clippy::useless_conversion,
    reason = "C long width differs between native ABIs"
)]
fn long(value: c_long) -> i64 {
    i64::from(value)
}
/// The `n` values of a serial vector, checked against its native length.
///
/// # Safety
/// `v` is null or a live serial vector whose data outlives `'a` unaliased by writers.
unsafe fn values<'a>(v: ffi::N_Vector, n: usize) -> Result<&'a [f64], ProblemError> {
    // SAFETY: `v` is non-null here, so live (the caller's contract).
    if v.is_null() || unsafe { ffi::N_VGetLength(v) } != index(n)? {
        return Err(ProblemError::Internal("SUNDIALS vector dimensions".into()));
    }
    // SAFETY: as above.
    let p = unsafe { ffi::N_VGetArrayPointer(v) };
    if p.is_null() {
        return Err(ProblemError::Internal(
            "SUNDIALS null vector storage".into(),
        ));
    }
    // SAFETY: the non-null data array of a serial vector of length `n`, live for `'a`.
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
/// Overwrite a serial vector with finite values of its exact length.
///
/// # Safety
/// `v` is null or a live serial vector, and no reference into its data is live.
unsafe fn publish(v: ffi::N_Vector, from: &[f64]) -> Result<(), ProblemError> {
    // SAFETY: the caller's contract; the length check is its only use.
    unsafe { values(v, from.len()) }?;
    if from.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite root callback output"));
    }
    // SAFETY: `v` is a live serial vector (checked above).
    let data = unsafe { ffi::N_VGetArrayPointer(v) };
    // SAFETY: its non-null data array holds exactly `from.len()` values and cannot
    // overlap the Rust slice.
    unsafe { std::ptr::copy_nonoverlapping(from.as_ptr(), data, from.len()) };
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
fn raw_map_failure(error: &ProblemError) -> RawMapFailure {
    match crate::callback::classify(error) {
        crate::callback::Failure::Trial => RawMapFailure::Trial,
        crate::callback::Failure::Fatal => RawMapFailure::Fatal,
        crate::callback::Failure::Stopped(stop) => RawMapFailure::Stopped(stop),
    }
}
unsafe extern "C" fn residual(x: ffi::N_Vector, out: ffi::N_Vector, data: *mut c_void) -> i32 {
    // SAFETY: `data` is null or this session's registered boxed `Context`, live and not
    // otherwise borrowed while KINSOL calls back on the owning thread.
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let n = c.n;
    let execution = c.state.execution.clone();
    let producer = c.state.progress_producer().cloned();
    let ordinal = c
        .state
        .counts
        .get(&crate::callback::CallbackKey::Named("residual"))
        .copied()
        .unwrap_or(0)
        .saturating_add(1);
    let value = c.state.evaluate("residual", || {
        // SAFETY: KINSOL passes null or live serial vectors for the callback, and
        // `values` and `publish` check them against the `n` coordinates.
        shifted_into(unsafe { values(x, n) }?, &c.offsets, &mut c.point)?;
        let x = &c.point;
        let v = &mut c.output[..n];
        v.fill(f64::NAN);
        match &mut c.function {
            Function::Equations(o) => o.residual(x, v)?,
            Function::Picard { oracle, .. } => oracle.residual(x, v)?,
            Function::FixedPoint(o) => {
                let result = crate::quality::contained(|| o.map(x, v));
                let result = result.and_then(|()| {
                    if v.iter().all(|value| value.is_finite()) {
                        Ok(())
                    } else {
                        Err(ProblemError::numerical("nonfinite fixed-point map"))
                    }
                });
                if let Some(history) = &mut c.raw_map {
                    let failure = result.as_ref().err().map(raw_map_failure);
                    history.record(
                        o.contract(),
                        x,
                        result.is_ok().then_some(&*v),
                        RawMapRole::FixedPointEvaluation,
                        failure,
                    );
                }
                result?;
            }
        }
        // SAFETY: as above.
        unsafe { publish(out, v) }?;
        if let Some(layout) = &c.progress_layout {
            // Function evaluations are source-associated scalars, not accepted
            // iterations or a norm of unlike physical units.
            execution.progress.push_root_evaluation(
                layout.clone(),
                ordinal,
                v,
                execution.started.elapsed(),
                producer.as_ref(),
            );
        }
        Ok(())
    });
    linear_work::current(c);
    // KINFP and KINPicardAA only test for negative callback returns; a positive
    // trial refusal would let them consume stale output. They cannot recover the
    // failed map, so latch its original typed cause before returning terminally.
    if value.is_none()
        && c.state.terminal.is_none()
        && !matches!(c.function, Function::Equations(_))
    {
        c.state.terminal = Some((
            Termination::Evaluation,
            c.state.last_failure.as_ref().map_or_else(
                || "failed KINSOL map evaluation".into(),
                ToString::to_string,
            ),
        ));
    }
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
    // SAFETY: `data` is null or this session's registered boxed `Context`, live and not
    // otherwise borrowed while KINSOL calls back on the owning thread.
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let value = c.state.evaluate("jacobian", || {
        // SAFETY: the native callback's checked serial vector in original coordinates.
        shifted_into(unsafe { values(x, c.n) }?, &c.offsets, &mut c.point)?;
        let point = &c.point;
        let v = &mut c.output[..c.rows.len()];
        v.fill(f64::NAN);
        match &mut c.function {
            Function::Equations(o) => o.jacobian(
                // SAFETY: KINSOL passes null or a live serial vector for the callback, and `values`
                // checks it against the `n` coordinates.
                point, v,
            )?,
            Function::Picard { linear, .. } => v.copy_from_slice(linear.val()),
            Function::FixedPoint(_) => {
                return Err(ProblemError::Unsupported(
                    "map has no residual Jacobian".into(),
                ));
            }
        }
        if v.iter().any(|v| !v.is_finite()) || matrix.is_null() {
            return Err(ProblemError::numerical("invalid root Jacobian"));
        }
        // KINSOL passes the session's own matrix: dense `n × n`, or the KLU matrix `new`
        // created with `n + 1` index pointers and room for `c.rows.len()` nonzeros.
        if c.dense {
            native!(ffi::SUNMatZero(matrix), "dense zero")?;
            for col in 0..c.n {
                // SAFETY: a column `col < n` of the session's live dense matrix.
                let p = unsafe { ffi::SUNDenseMatrix_Column(matrix, index(col)?) };
                if p.is_null() {
                    return Err(ProblemError::Internal("null dense column".into()));
                }
                for k in c.columns[col] as usize..c.columns[col + 1] as usize {
                    // SAFETY: the canonical pattern's rows are below `n`, within the
                    // column's `n` entries.
                    let entry = unsafe { p.add(c.rows[k] as usize) };
                    // SAFETY: as above.
                    unsafe { *entry = v[k] };
                }
            }
        } else {
            // SAFETY: storage queries of the session's live KLU matrix.
            let p = unsafe { ffi::SUNSparseMatrix_Data(matrix) };
            // SAFETY: as above.
            let columns = unsafe { ffi::SUNSparseMatrix_IndexPointers(matrix) };
            // SAFETY: as above.
            let rows = unsafe { ffi::SUNSparseMatrix_IndexValues(matrix) };
            if columns.is_null() || (!v.is_empty() && (p.is_null() || rows.is_null())) {
                return Err(ProblemError::Internal("null sparse values".into()));
            }
            // KINSOL zeros the matrix before each Jacobian callback. SUNDIALS
            // sparse zero clears both indices and values, including CSC pointers.
            // SAFETY: the matrix holds the `n + 1 == c.columns.len()` index pointers, in
            // native storage that cannot overlap the Rust vector.
            unsafe { std::ptr::copy_nonoverlapping(c.columns.as_ptr(), columns, c.columns.len()) };
            if !v.is_empty() {
                // SAFETY: its nonzero capacity holds the `c.rows.len()` row indices.
                unsafe { std::ptr::copy_nonoverlapping(c.rows.as_ptr(), rows, c.rows.len()) };
                // SAFETY: and the `v.len() == c.rows.len()` values.
                unsafe { std::ptr::copy_nonoverlapping(v.as_ptr(), p, v.len()) };
            }
        }
        if let Some(setup) = &mut c.setup {
            setup.point.clone_from(&c.point);
            setup.data = c.data;
        } else {
            c.setup = Some(SetupStamp {
                point: c.point.clone(),
                data: c.data,
            });
        }
        c.setup_calls = c.setup_calls.saturating_add(1);
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
    // SAFETY: `data` is null or this session's registered boxed `Context`, live and not
    // otherwise borrowed while KINSOL calls back on the owning thread.
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let value = c.state.evaluate("jvp", || {
        let y = &mut c.output[..c.n];
        y.fill(f64::NAN);
        match &mut c.function {
            Function::Equations(o) => o.jacobian_product(
                {
                    // SAFETY: KINSOL supplies a live serial input (or null), and
                    // `values` checks null, length and storage before borrowing.
                    let point = unsafe { values(x, c.n) }?;
                    shifted_into(point, &c.offsets, &mut c.point)?;
                    &c.point
                },
                // SAFETY: as above.
                unsafe { values(v, c.n) }?,
                y,
            )?,
            Function::Picard { linear, .. } => {
                // SAFETY: as above.
                let direction = unsafe { values(v, c.n) }?;
                faer::sparse::linalg::matmul::sparse_dense_matmul(
                    faer::MatMut::from_column_major_slice_mut(y, c.n, 1),
                    faer::Accum::Replace,
                    linear.as_ref(),
                    faer::MatRef::from_column_major_slice(direction, c.n, 1),
                    1.,
                    faer::Par::Seq,
                );
            }
            Function::FixedPoint(_) => {
                return Err(ProblemError::Unsupported("map has no residual JVP".into()));
            }
        }
        // SAFETY: as above.
        unsafe { publish(out, y) }
    });
    result(value, &c.state)
}
/// Analytic right preconditioner setup under KINSOL's native refresh policy.
unsafe extern "C" fn precondition_setup(
    x: ffi::N_Vector,
    _us: ffi::N_Vector,
    _f: ffi::N_Vector,
    _fs: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is null or this session's registered boxed `Context`, live and not
    // otherwise borrowed while KINSOL calls back on the owning thread.
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let execution = c.state.execution.clone();
    let value = c.state.evaluate("preconditioner", || {
        // SAFETY: the native callback's checked serial vector in original coordinates.
        shifted_into(unsafe { values(x, c.n) }?, &c.offsets, &mut c.point)?;
        let point = &c.point;
        let v = &mut c.output[..c.rows.len()];
        v.fill(f64::NAN);
        match &mut c.function {
            Function::Equations(o) => o.jacobian(
                // SAFETY: KINSOL passes null or a live serial vector for the callback, and `values`
                // checks it against the `n` coordinates.
                point, v,
            )?,
            Function::Picard { linear, .. } => v.copy_from_slice(linear.val()),
            Function::FixedPoint(_) => {
                return Err(ProblemError::Unsupported(
                    "map has no preconditioner".into(),
                ));
            }
        }
        if v.iter().any(|value| !value.is_finite()) {
            return Err(ProblemError::numerical(
                "invalid root preconditioner Jacobian",
            ));
        }
        if let Some(blocks) = &mut c.blocks {
            blocks.setup(v, &execution)?;
        } else {
            for col in 0..c.n {
                let range = c.columns[col] as usize..c.columns[col + 1] as usize;
                let diagonal = range
                    .clone()
                    .find(|k| c.rows[*k] as usize == col)
                    .map_or(0.0, |k| v[k]);
                c.inverse_diagonal[col] =
                    if diagonal.is_finite() && diagonal.abs() > f64::MIN_POSITIVE {
                        diagonal.recip()
                    } else {
                        1.0
                    };
            }
        }
        if let Some(setup) = &mut c.setup {
            setup.point.clone_from(&c.point);
            setup.data = c.data;
        } else {
            c.setup = Some(SetupStamp {
                point: c.point.clone(),
                data: c.data,
            });
        }
        c.setup_calls = c.setup_calls.saturating_add(1);
        Ok(())
    });
    result(value, &c.state)
}
/// Selected library block or Jacobi preconditioner solve, in place.
unsafe extern "C" fn precondition_solve(
    _x: ffi::N_Vector,
    _us: ffi::N_Vector,
    _f: ffi::N_Vector,
    _fs: ffi::N_Vector,
    v: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is null or this session's registered boxed `Context`, live and not
    // otherwise borrowed while KINSOL calls back on the owning thread.
    let Some(c) = (unsafe { data.cast::<Context>().as_mut() }) else {
        return -1;
    };
    let execution = c.state.execution.clone();
    let value = c.state.evaluate("preconditioner.solve", || {
        // SAFETY: KINSOL passes null or a live serial vector for the callback, and
        // `values` checks it against the `n` coordinates.
        if let Some(blocks) = &mut c.blocks {
            // SAFETY: the callback's checked live serial right-hand side.
            let z = blocks.solve(unsafe { values(v, c.n) }?, &execution)?;
            // SAFETY: the native RHS is no longer borrowed; the output is session-owned.
            unsafe { publish(v, z) }
        } else {
            // SAFETY: the callback receives this session's live serial vector with n entries.
            let z: Vec<f64> = unsafe { values(v, c.n) }?
                .iter()
                .zip(&c.inverse_diagonal)
                .map(|(r, d)| r * d)
                .collect();
            // SAFETY: the RHS slice is no longer used.
            unsafe { publish(v, &z) }
        }
    });
    result(value, &c.state)
}
// The fixed allowance covers KINSOL's internal clones even without AA/Krylov.
// N_VSpace reports payload words and one integer, omitting each cloned vector's
// operations/content/object headers. SPFGMR owns two dimension-sized arrays;
// this conservative per-dimension allowance covers both plus pointer arrays.
fn native_header_bytes(linear: Linear) -> Result<usize, ProblemError> {
    match linear.krylov() {
        Some(dimension) => dimension
            .checked_mul(2048)
            .and_then(|bytes| bytes.checked_add(16384))
            .ok_or_else(|| ProblemError::memory("Krylov retained header extent")),
        None => Ok(16384),
    }
}
// SPFGMR dominates the four supported Krylov allocation shapes: two vector
// arrays, Hessenberg, rotations and header/pointer storage, plus callback scratch.
fn native_krylov_storage(n: usize, linear: Linear) -> Result<usize, ProblemError> {
    let headers = native_header_bytes(linear)?;
    let Some(dimension) = linear.krylov() else {
        return Ok(headers);
    };
    dimension
        .checked_mul(2)
        .and_then(|count| count.checked_add(64))
        .and_then(|count| count.checked_mul(n))
        .and_then(|count| count.checked_mul(128))
        .and_then(|vectors| {
            dimension
                .checked_add(10)
                .and_then(|width| dimension.checked_mul(width))
                .and_then(|entries| entries.checked_mul(64))
                .and_then(|matrix| vectors.checked_add(matrix))
        })
        .and_then(|bytes| bytes.checked_add(headers))
        .ok_or_else(|| ProblemError::memory("native Krylov workspace extent"))
}
// AA's R/T QR matrices and scalar/pointer arrays are not fully represented by
// KINGetWorkSpace. Vector payloads are queried there; their headers are not.
fn anderson_storage(n: usize, method: Method) -> Result<(usize, usize), ProblemError> {
    let m = method.anderson;
    if m == 0 {
        return Ok((0, 0));
    }
    let omitted = m
        .checked_mul(m)
        .and_then(|square| square.checked_mul(16))
        .and_then(|qr| {
            m.checked_mul(4096)
                .and_then(|headers| qr.checked_add(headers))
        })
        .ok_or_else(|| ProblemError::memory("Anderson QR/header extent"))?;
    // KINAllocVectors: three m-sized vector arrays, fold/gold and fixed work clones.
    // Two m*m matrices cover ICWY; every other QR route uses no more storage.
    let native_workspace = native_krylov_storage(n, method.linear)?;
    let peak = m
        .checked_mul(3)
        .and_then(|count| count.checked_add(16))
        .and_then(|count| count.checked_mul(n))
        .and_then(|count| count.checked_mul(8))
        .and_then(|payload| payload.checked_add(omitted))
        .and_then(|bytes| bytes.checked_add(native_workspace))
        .ok_or_else(|| ProblemError::memory("Anderson native allocation extent"))?;
    Ok((peak, omitted))
}
impl Session {
    /// Bytes the session keeps between solves, as the libraries report them: KINSOL's
    /// and its linear interface's workspace, the session's own vectors, the Jacobian
    /// storage and KLU's current factor memory, plus the owned callback buffers (Plan 22
    /// I14). Library-reported payload words and KLU memory are combined with
    /// source-backed conservative header, pointer-array and Anderson QR allowances.
    pub fn retained_bytes(&self) -> usize {
        const HEADER: usize = 256;
        // Real and integer word counts at 8 bytes each, plus the object header allowance.
        let words = |r: i64, i: i64| {
            usize::try_from(r.saturating_add(i).max(0))
                .unwrap_or(usize::MAX)
                .saturating_mul(8)
                .saturating_add(HEADER)
        };
        let callback = &self.callback;
        let mut bytes = (size_of::<Self>() + size_of::<Context>()).saturating_add(
            (callback.rows.capacity()
                + callback.columns.capacity()
                + callback.offsets.capacity()
                + callback.point.capacity()
                + callback.output.capacity()
                + callback.inverse_diagonal.capacity()
                + callback.setup.as_ref().map_or(0, |s| s.point.capacity()))
            .saturating_mul(8),
        );
        bytes = bytes.saturating_add(
            callback
                .progress_layout
                .as_ref()
                .map_or(0, |layout| layout.retained_bytes()),
        );
        bytes = bytes.saturating_add(
            callback
                .blocks
                .as_ref()
                .map_or(0, block::Blocks::retained_bytes),
        );
        bytes = bytes
            .saturating_add(native_header_bytes(self.settings.method.linear).unwrap_or(usize::MAX));
        bytes = bytes.saturating_add(
            anderson_storage(callback.n, self.settings.method)
                .map_or(usize::MAX, |(_, omitted)| omitted),
        );
        // Every queried object is live and owned by this session; the queries only write
        // the caller's counters.
        for v in [self.x, self.us, self.fs, self.signs] {
            if !v.is_null() {
                let (mut r, mut i) = (0, 0);
                // SAFETY: a size query of a live session vector into locals.
                unsafe { ffi::N_VSpace(v, &raw mut r, &raw mut i) };
                bytes = bytes.saturating_add(words(r, i));
            }
        }
        let (mut r, mut i) = (0, 0);
        if !self.mem.is_null()
            // SAFETY: a workspace query of the session's live KINSOL memory into locals.
            && unsafe { ffi::KINGetWorkSpace(self.mem, &raw mut r, &raw mut i) } >= 0
        {
            bytes = bytes.saturating_add(words(long(r), long(i)));
        }
        if !self.linear.is_null()
            // SAFETY: as above, for its attached linear solver.
            && unsafe { ffi::KINGetLinWorkSpace(self.mem, &raw mut r, &raw mut i) } >= 0
        {
            bytes = bytes.saturating_add(words(long(r), long(i)));
        }
        if !self.matrix.is_null()
            // SAFETY: a size query of the session's live matrix into locals.
            && unsafe { ffi::SUNMatSpace(self.matrix, &raw mut r, &raw mut i) } >= 0
        {
            bytes = bytes.saturating_add(words(long(r), long(i)));
        }
        if !self.linear.is_null() && self.settings.method.linear == Linear::Klu {
            // SAFETY: the session's live KLU solver, queried between solves.
            let mut native_bytes = 0;
            // SAFETY: read-only scalar query of the live solver into a local.
            let status =
                unsafe { ffi::SUNLinSol_KLUGetMemUsage(self.linear, &raw mut native_bytes) };
            bytes = bytes.saturating_add(if status == 0 {
                native_bytes
            } else {
                usize::MAX
            });
        }
        bytes
    }
    /// Controls fixed during native allocation must match; legal numeric controls refresh.
    pub fn matches_settings(&self, settings: &Settings) -> bool {
        let (held, next) = (&self.settings.method, &settings.method);
        held.linear == next.linear
            && held.anderson == next.anderson
            && held.orthogonalization == next.orthogonalization
            && held.preconditioner == next.preconditioner
            && (held.strategy == next.strategy
                || matches!(
                    (held.strategy, next.strategy),
                    (Strategy::Newton, Strategy::LineSearch)
                        | (Strategy::LineSearch, Strategy::Newton)
                ))
    }
    /// Check the immutable compiler/backend layout stamp before constructing an update.
    pub fn matches_layout(&self, stamp: &Compatibility) -> bool {
        self.compatibility.same_session(stamp) && stamp.backend == Backend::Kinsol
    }
    /// Replace compatible numeric equations while retaining native allocation and
    /// sparse layout. New equation, guard or splitting data never authorizes using
    /// a previous application's numeric setup or preconditioner policy.
    pub fn replace(
        &mut self,
        function: Function,
        settings: Settings,
        compatibility: Compatibility,
    ) -> Result<(), ProblemError> {
        self.clear_callback_setup();
        let constraints = settings.validate(&function)?;
        if !self.matches_settings(&settings)
            || !compatibility.same_session(&self.compatibility)
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
            return Err(ProblemError::Unsupported(
                "KINSOL replacement changes layout".into(),
            ));
        }
        if let Some(p) = function.pattern() {
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
                return Err(ProblemError::Unsupported(
                    "KINSOL replacement changes sparse layout".into(),
                ));
            }
        }
        if constraints.any() {
            if self.signs.is_null() {
                // SAFETY: the session's live context; `Drop` frees the vector.
                self.signs =
                    unsafe { ffi::N_VNew_Serial(index(constraints.signs.len())?, self.ctx) };
            }
            // SAFETY: null or the session's live sign vector, referenced nowhere else.
            unsafe { publish(self.signs, &constraints.signs)? };
            native!(
                ffi::KINSetConstraints(self.mem, self.signs),
                "updated sign constraints",
            )?;
        } else {
            native!(
                ffi::KINSetConstraints(self.mem, std::ptr::null_mut()),
                "clear sign constraints",
            )?;
        }
        self.callback.function = function;
        self.callback.offsets = constraints.offsets;
        self.callback.data = compatibility.data;
        self.settings = settings;
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
        let constraints = settings.validate(&function)?;
        let n = function.contract().variables.len();
        let (anderson_peak, _) = anderson_storage(n, settings.method)?;
        // AA's peak already includes the selected Krylov workspace. Without AA,
        // every Krylov route must still own its finite workspace before SUN allocation.
        let native_peak = if anderson_peak > 0 {
            anderson_peak
        } else if settings.method.linear.krylov().is_some() {
            native_krylov_storage(n, settings.method.linear)?
        } else {
            0
        };
        if native_peak > 0 {
            execution.check()?;
            let memory = execution.memory.filter(|bytes| *bytes > 0).ok_or_else(|| {
                ProblemError::memory("native Krylov/Anderson needs finite admitted storage")
            })?;
            if native_peak > memory {
                return Err(ProblemError::memory(
                    "native Krylov/Anderson plan exceeds admitted storage",
                ));
            }
        }
        let blocks = if settings.method.preconditioner == Preconditioner::BlockFactor {
            Some(block::Blocks::admit(
                &function,
                &execution,
                settings.method.linear,
            )?)
        } else {
            None
        };
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
        let progress_layout = root_progress_layout(&function, execution.callback_role);
        let callback = Box::new(Context {
            progress_layout,
            function,
            state: CallbackState::new(execution),
            n,
            point: vec![0.; n],
            output: vec![0.; n.max(rows.len())],
            rows,
            columns,
            dense: matches!(settings.method.linear, Linear::Dense { .. }),
            offsets: constraints.offsets.clone(),
            inverse_diagonal: vec![1.0; n],
            blocks,
            setup: None,
            setup_calls: 0,
            data: compatibility.data,
            raw_map: None,
            klu: std::ptr::null_mut(),
            klu_baseline: None,
            klu_producer: None,
            klu_phases: std::array::from_fn(|_| None),
            klu_observer_installed: false,
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
            retention_ready: true,
            #[cfg(test)]
            stock_klu: false,
            callback,
            settings,
            compatibility,
            _local: PhantomData,
        };
        native!(ffi::SUNContext_Create(0, &raw mut s.ctx), "context")?;
        // SAFETY: new context owned by s; Drop frees it after all native objects.
        unsafe { crate::sundials_errors::install(s.ctx) }?;
        // SAFETY: the session's live context; `Drop` frees the vector.
        s.x = unsafe { ffi::N_VNew_Serial(index(n)?, s.ctx) };
        // SAFETY: as above.
        s.us = unsafe { ffi::N_VNew_Serial(index(n)?, s.ctx) };
        // SAFETY: as above.
        s.fs = unsafe { ffi::N_VNew_Serial(index(n)?, s.ctx) };
        if s.x.is_null() || s.us.is_null() || s.fs.is_null() {
            return Err(ProblemError::memory("SUNDIALS vector allocation"));
        }
        // SAFETY: the new session's live scale vector, referenced nowhere else.
        unsafe { publish(s.us, &s.settings.variable_scales)? };
        // SAFETY: as above.
        unsafe { publish(s.fs, &s.settings.residual_scales)? };
        // SAFETY: the session's live context; `Drop` frees the memory.
        s.mem = unsafe { ffi::KINCreate(s.ctx) };
        if s.mem.is_null() {
            return Err(ProblemError::memory("KINSOL allocation"));
        }
        native!(
            ffi::KINSetMAA(s.mem, s.settings.method.anderson as _),
            "Anderson history",
        )?;
        native!(
            ffi::KINSetOrthAA(
                s.mem,
                orthogonalization_code(s.settings.method.orthogonalization),
            ),
            "Anderson orthogonalization",
        )?;
        native!(ffi::KINInit(s.mem, Some(residual), s.x), "initialization")?;
        // The boxed context has a stable address for the session's lifetime.
        native!(
            ffi::KINSetUserData(s.mem, (&raw mut *s.callback).cast()),
            "callback context",
        )?;
        let method = s.settings.method;
        if method.strategy != Strategy::FixedPoint {
            match method.linear {
                Linear::Klu => {
                    // SAFETY: the session's live context; `Drop` frees the matrix.
                    s.matrix = unsafe {
                        ffi::SUNSparseMatrix(
                            index(n)?,
                            index(n)?,
                            index(s.callback.rows.len())?,
                            0,
                            s.ctx,
                        )
                    };
                    if s.matrix.is_null() {
                        return Err(ProblemError::memory("sparse matrix allocation"));
                    }
                    // SAFETY: a storage query of the new live matrix.
                    let pointers = unsafe { ffi::SUNSparseMatrix_IndexPointers(s.matrix) };
                    // SAFETY: the matrix and the canonical pattern both hold `n + 1` index
                    // pointers, in storage that cannot overlap.
                    unsafe {
                        std::ptr::copy_nonoverlapping(s.callback.columns.as_ptr(), pointers, n + 1)
                    };
                    if !s.callback.rows.is_empty() {
                        // SAFETY: as above.
                        let indices = unsafe { ffi::SUNSparseMatrix_IndexValues(s.matrix) };
                        // SAFETY: the matrix was created with room for the pattern's rows.
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                s.callback.rows.as_ptr(),
                                indices,
                                s.callback.rows.len(),
                            )
                        };
                    }
                    // SAFETY: the session's live vector, matrix and context; `Drop` frees
                    // the solver.
                    s.linear = unsafe { ffi::SUNLinSol_KLUFixedPattern(s.x, s.matrix, s.ctx) };
                }
                Linear::Dense { .. } => {
                    // SAFETY: the session's live context; `Drop` frees the matrix.
                    s.matrix = unsafe { ffi::SUNDenseMatrix(index(n)?, index(n)?, s.ctx) };
                    if s.matrix.is_null() {
                        return Err(ProblemError::memory("dense matrix allocation"));
                    }
                    // SAFETY: the session's live vector, matrix and context; `Drop` frees
                    // the solver.
                    s.linear = unsafe { ffi::SUNLinSol_Dense(s.x, s.matrix, s.ctx) };
                }
                Linear::Spgmr { dimension }
                | Linear::Spfgmr { dimension }
                | Linear::Spbcgs { dimension }
                | Linear::Sptfqmr { dimension } => {
                    let maxl = i32::try_from(dimension.into_inner())
                        .map_err(|_| ProblemError::Contract("KINSOL Krylov dimension".into()))?;
                    let side = match method.preconditioner {
                        Preconditioner::None => ffi::SUN_PREC_NONE,
                        Preconditioner::Jacobi | Preconditioner::BlockFactor => ffi::SUN_PREC_RIGHT,
                    } as i32;
                    let krylov: unsafe extern "C" fn(
                        ffi::N_Vector,
                        i32,
                        i32,
                        ffi::SUNContext,
                    ) -> ffi::SUNLinearSolver = match method.linear {
                        Linear::Spgmr { .. } => ffi::SUNLinSol_SPGMR,
                        Linear::Spfgmr { .. } => ffi::SUNLinSol_SPFGMR,
                        Linear::Spbcgs { .. } => ffi::SUNLinSol_SPBCGS,
                        _ => ffi::SUNLinSol_SPTFQMR,
                    };
                    // SAFETY: a SUNDIALS Krylov constructor over the session's live vector
                    // and context; `Drop` frees the solver.
                    s.linear = unsafe { krylov(s.x, side, maxl, s.ctx) };
                }
            }
            if s.linear.is_null() {
                return Err(ProblemError::memory("native linear solver allocation"));
            }
            native!(
                ffi::KINSetLinearSolver(s.mem, s.linear, s.matrix),
                "linear solver",
            )?;
            if method.linear.krylov().is_some() {
                native!(ffi::KINSetJacTimesVecFn(s.mem, Some(jvp)), "analytic JVP")?;
                if method.preconditioner != Preconditioner::None {
                    native!(
                        ffi::KINSetPreconditioner(
                            s.mem,
                            Some(precondition_setup),
                            Some(precondition_solve),
                        ),
                        "selected analytic preconditioner",
                    )?;
                }
            } else {
                native!(ffi::KINSetJacFn(s.mem, Some(jacobian)), "analytic Jacobian")?;
            }
        }
        if constraints.any() {
            // SAFETY: the session's live context; `Drop` frees the vector.
            s.signs = unsafe { ffi::N_VNew_Serial(index(n)?, s.ctx) };
            // SAFETY: null or the new live sign vector, referenced nowhere else.
            unsafe { publish(s.signs, &constraints.signs)? };
            native!(ffi::KINSetConstraints(s.mem, s.signs), "sign constraints")?;
        }
        Ok(s)
    }
    fn clear_callback_setup(&mut self) {
        self.callback.setup = None;
        self.callback.inverse_diagonal.fill(1.0);
        // Block admission may depend on current values/guards and carries numeric
        // factors. Re-admit under the next application's actual scope/allowance.
        self.callback.blocks = None;
        self.callback.setup_calls = 0;
    }
    /// Run with an explicit start and a fresh native initial setup policy. Retained
    /// allocation does not authorize a previous application's numeric factor or
    /// preconditioner. KINSOL owns setup reuse and monitoring within this call and
    /// may return an already-converged initial guess before any setup is needed.
    pub fn solve(
        &mut self,
        initial: &[f64],
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        self.retention_ready = false;
        self.begin_application()?;
        let mut result =
            self.solve_application(initial, controls, accuracy, execution, tolerances, warm);
        // All work is synchronous on this owner. Failed attempts release numerical
        // factors even when they returned before KINSol or original validation.
        let successful = result.as_ref().is_ok_and(|report| {
            report.termination.category == Termination::Success
                && report.quality.as_ref().is_some_and(Quality::feasible)
                && report.validation_failure().is_none()
                && report.shared_effective_failure().is_none_or(|cause| {
                    crate::callback::classify(cause.as_ref()) == crate::callback::Failure::Trial
                })
        });
        if !successful && let Err(cleanup) = self.begin_application() {
            // The original scientific/callback cause stays authoritative.
            // This owner cannot be retained after uncertain cleanup.
            if let Ok(report) = &mut result {
                report.metrics.insert(
                    "klu.retention_reset_failure".into(),
                    Metric::Text(cleanup.to_string()),
                );
            }
            if let Some(context) = pse_diagnostics::observation::current() {
                context.unavailable("kinsol.retention_reset", &cleanup.to_string());
            }
            return result;
        }
        self.retention_ready = match &result {
            Ok(report) => {
                !matches!(report.termination.category, Termination::Panic)
                    && !matches!(
                        report.termination.code as i32,
                        ffi::KIN_MEM_NULL
                            | ffi::KIN_NO_MALLOC
                            | ffi::KIN_CONTEXT_ERR
                            | ffi::KIN_VECTOROP_ERR
                    )
                    && report.termination.name != "KIN_UNKNOWN"
            }
            Err(error) => matches!(
                error,
                ProblemError::Contract(_)
                    | ProblemError::Unsupported(_)
                    | ProblemError::Reuse { .. }
                    | ProblemError::Structural { .. }
                    | ProblemError::Cancelled
                    | ProblemError::Limit { .. }
                    | ProblemError::Math(_)
                    | ProblemError::Provider(_)
            ),
        };
        result
    }
    fn begin_application(&mut self) -> Result<(), ProblemError> {
        #[cfg(test)]
        if self.stock_klu {
            return Ok(());
        }
        if self.settings.method.linear == Linear::Klu && !self.linear.is_null() {
            native!(
                ffi::SUNLinSol_KLUBeginApplication(self.linear),
                "fresh KLU application"
            )?;
        }
        Ok(())
    }
    /// A completed synchronous application with known native integrity may return
    /// to the worker cache; terminal failures have already reset numerical state.
    pub(crate) fn ready_for_retention(&self) -> bool {
        self.retention_ready
    }
    #[cfg(test)]
    pub(crate) fn test_klu_state(&self) -> (usize, ffi::SUNLinearSolver_KLUWork) {
        // SAFETY: test-only read of the idle native owner retained by this session.
        (
            self.linear as usize,
            unsafe { linear_work::snapshot(self.linear) }.unwrap(),
        )
    }
    fn solve_application(
        &mut self,
        initial: &[f64],
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        self.clear_callback_setup();
        controls.validate()?;
        if controls.threads != 1 || !controls.options.is_empty() {
            return Err(ProblemError::Unsupported(
                "serial KINSOL uses typed Settings and one core".into(),
            ));
        }
        let n = self.callback.n;
        tolerances.validate(n, n)?;
        if initial.len() != n {
            return Err(ProblemError::Internal("root start dimensions".into()));
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
        if self.settings.method.preconditioner == Preconditioner::BlockFactor
            && self.callback.blocks.is_none()
        {
            // Replacements are admitted under this attempt's current scope/allowance,
            // never under the previous solve's expired callback clock.
            self.callback.blocks = Some(block::Blocks::admit(
                &self.callback.function,
                &execution,
                self.settings.method.linear,
            )?);
        }
        self.callback.progress_layout =
            root_progress_layout(&self.callback.function, execution.callback_role);
        self.callback.state = CallbackState::new(execution.clone());
        self.callback.klu = if self.settings.method.linear == Linear::Klu {
            self.linear
        } else {
            std::ptr::null_mut()
        };
        // SAFETY: this live linear solver remains exclusively Session-owned until drain.
        self.callback.klu_baseline = unsafe { linear_work::snapshot(self.callback.klu) };
        self.callback.klu_producer = pse_diagnostics::observation::current().map(|context| {
            context.producer(
                pse_diagnostics::observation::ProducerRole::NativeFactor,
                "kinsol.klu",
            )
        });
        self.callback.raw_map = match &self.callback.function {
            Function::FixedPoint(oracle) => Some(
                RawMapCapture::new(
                    oracle.contract(),
                    self.compatibility.profile,
                    controls.history,
                    oracle.physical_tear_coordinates(),
                    &tolerances.rows,
                )?
                .with_inner_context(oracle.inner_context())
                .with_progress(execution.progress.clone(), execution.started),
            ),
            _ => None,
        };
        self.callback.setup_calls = 0;
        if let Some(blocks) = &mut self.callback.blocks {
            blocks.reset_counts();
        }
        let method = self.settings.method;
        let (eta, eta_constant, eta_gamma, eta_alpha) = match method.eta {
            Eta::Choice1 => (ffi::KIN_ETACHOICE1, 0.0, 0.0, 0.0),
            Eta::Choice2 { gamma, alpha } => (ffi::KIN_ETACHOICE2, 0.0, gamma.into_inner(), alpha),
            Eta::Constant { value } => (ffi::KIN_ETACONSTANT, value.into_inner(), 0.0, 0.0),
        };
        let delay = c_long::try_from(method.anderson_delay)
            .map_err(|_| ProblemError::Contract("KINSOL Anderson delay".into()))?;
        // KINSOL iterates the shifted coordinates of one-sided bounds.
        // SAFETY: the session's live iterate vector, referenced nowhere else.
        unsafe { publish(self.x, &shifted(start, &self.callback.offsets, -1.0))? };
        // SAFETY: as above, for the variable scales.
        unsafe { publish(self.us, &self.settings.variable_scales)? };
        // SAFETY: as above, for the residual scales.
        unsafe { publish(self.fs, &self.settings.residual_scales)? };
        native!(
            ffi::KINSetNumMaxIters(self.mem, controls.iterations.into()),
            "iteration limit",
        )?;
        native!(
            ffi::KINSetFuncNormTol(self.mem, accuracy.feasibility),
            "residual tolerance",
        )?;
        native!(
            ffi::KINSetScaledStepTol(self.mem, self.settings.step_tolerance),
            "step tolerance",
        )?;
        native!(
            ffi::KINSetDamping(self.mem, method.damping.into_inner()),
            "damping",
        )?;
        native!(
            ffi::KINSetDampingAA(self.mem, method.damping.into_inner()),
            "Anderson damping",
        )?;
        native!(
            ffi::KINSetMaxSetupCalls(self.mem, method.setup_interval.into()),
            "setup interval",
        )?;
        native!(ffi::KINSetNoInitSetup(self.mem, 0), "initial setup policy")?;
        // Direct modified Newton uses KINSOL's progress monitor. KINSOL itself disables
        // this monitor for inexact Krylov and uses its forcing/refresh criteria instead.
        native!(
            ffi::KINSetNoResMon(self.mem, 0),
            "native residual monitoring"
        )?;
        // Every refreshable option is set on every solve (zero selects KINSOL's
        // default), so a reused session never inherits a previous request's value.
        native!(
            ffi::KINSetMaxNewtonStep(self.mem, method.max_newton_step.unwrap_or(0.0)),
            "maximum Newton step",
        )?;
        native!(ffi::KINSetEtaForm(self.mem, eta), "forcing-term form")?;
        native!(
            ffi::KINSetEtaConstValue(self.mem, eta_constant),
            "constant forcing term",
        )?;
        native!(
            ffi::KINSetEtaParams(self.mem, eta_gamma, eta_alpha),
            "forcing-term parameters",
        )?;
        native!(ffi::KINSetDelayAA(self.mem, delay), "Anderson delay")?;
        let strategy = match method.strategy {
            Strategy::Picard => 2,
            Strategy::Newton => 0,
            Strategy::LineSearch => 1,
            Strategy::FixedPoint => 3,
        };
        linear_work::install(&mut self.callback);
        // SAFETY: the session's live KINSOL memory and vectors on its owning thread; the
        // callbacks reach the context only through their user data.
        let code = crate::observation::scoped("kinsol.solve", || unsafe {
            ffi::KINSol(self.mem, self.x, strategy, self.us, self.fs)
        });
        linear_work::clear(&mut self.callback);
        let mut report = SolveReport::new(
            Backend::Kinsol,
            self.callback.function.contract(),
            termination(code),
            &execution,
        );
        // KINSolInit rejects illegal inputs before resetting primary counters.
        // Its initial residual and INITIAL_GUESS_OK returns precede linear reset.
        // These boundaries are source-owned for bundled SUNDIALS 7.1.1, not
        // inferred by subtracting preceding-session totals.
        let primary_current = !matches!(
            code,
            ffi::KIN_MEM_NULL | ffi::KIN_NO_MALLOC | ffi::KIN_ILL_INPUT
        );
        let native_count = |get: unsafe extern "C" fn(*mut c_void, *mut c_long) -> i32| {
            let mut count = 0;
            // SAFETY: read-only getter on this session's live KINSOL memory.
            (primary_current && unsafe { get(self.mem, &mut count) } == 0)
                .then(|| u64::try_from(count).ok())
                .flatten()
        };
        // KINFP increments before G, including a failed map attempt. Newton and
        // Picard enter their iteration only after successful linear initialization.
        let iterations = native_count(ffi::KINGetNumNonlinSolvIters);
        report.evidence.work.iterations = iterations;
        let linear_count = |get| {
            if method.strategy == Strategy::FixedPoint {
                None
            } else {
                match iterations {
                    // No linear work ran this attempt; the retained getter is stale.
                    Some(0) => Some(0),
                    Some(_) => native_count(get),
                    None => None,
                }
            }
        };
        macro_rules! count {
            ($read:ident; $($get:ident),*) => {$(
                report.metrics.insert(stringify!($get).into(), $read(ffi::$get)
                    .and_then(|value| i64::try_from(value).ok())
                    .map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
            )*};
        }
        count!(native_count;
            KINGetNumNonlinSolvIters, KINGetNumFuncEvals,
            KINGetNumBetaCondFails, KINGetNumBacktrackOps
        );
        linear_work::finish(&self.callback, &mut report);
        report.evidence.native_observations = Some(std::sync::Arc::new(NativeObservations {
            backend: Backend::Kinsol,
            problem: self.callback.function.contract().identity,
            profile: self.compatibility.profile,
            coordinates: ObservationCoordinates::SubmittedOracle,
            stop_fidelity: StopFidelity::EvaluationCheckpoints,
            counters: NativeCounters::Kinsol {
                fixed_point_iterations: method.strategy == Strategy::FixedPoint,
                iterations,
                function_evaluations: native_count(ffi::KINGetNumFuncEvals),
                jacobian_evaluations: linear_count(ffi::KINGetNumJacEvals),
                linear_iterations: linear_count(ffi::KINGetNumLinIters),
                preconditioner_evaluations: linear_count(ffi::KINGetNumPrecEvals),
                preconditioner_solves: linear_count(ffi::KINGetNumPrecSolves),
                jacobian_products: linear_count(ffi::KINGetNumJtimesEvals),
            },
            current: None,
        }));
        if method.strategy != Strategy::FixedPoint {
            count!(linear_count;
                KINGetNumJacEvals, KINGetNumLinFuncEvals, KINGetNumPrecEvals,
                KINGetNumPrecSolves, KINGetNumLinIters, KINGetNumLinConvFails,
                KINGetNumJtimesEvals
            );
            let mut flag = 0;
            // A status flag is not a work counter: no current initialization does
            // not establish a zero/success flag. Keep it unavailable in that case.
            // SAFETY: read-only getter on this session's live KINSOL memory.
            let flag = if iterations.is_some_and(|n| n > 0)
                && unsafe { ffi::KINGetLastLinFlag(self.mem, &mut flag) } == 0
            {
                Metric::Integer(long(flag))
            } else {
                Metric::Unavailable(UnavailableReason::Unknown)
            };
            report.metrics.insert("KINGetLastLinFlag".into(), flag);
        }
        let residual_calls = self
            .callback
            .state
            .counts
            .get(&crate::callback::CallbackKey::Named("residual"))
            .copied()
            .unwrap_or(0);
        let real_getters: [(
            &str,
            unsafe extern "C" fn(*mut c_void, *mut f64) -> i32,
            bool,
        ); 2] = [
            (
                "KINGetFuncNorm",
                ffi::KINGetFuncNorm,
                code == ffi::KIN_INITIAL_GUESS_OK
                    || iterations.is_some_and(|n| {
                        n > 0
                            && !(method.strategy == Strategy::FixedPoint
                                && n == 1
                                && code == ffi::KIN_SYSFUNC_FAIL)
                    }),
            ),
            (
                "KINGetStepLength",
                ffi::KINGetStepLength,
                matches!(method.strategy, Strategy::Newton | Strategy::LineSearch)
                    && residual_calls > 1,
            ),
        ];
        for (name, get, current) in real_getters {
            let mut value = 0.;
            // SAFETY: current-phase statistics query on this session's live memory.
            let observed = if current && unsafe { get(self.mem, &mut value) } == 0 {
                Metric::Real(value)
            } else {
                Metric::Unavailable(UnavailableReason::Unknown)
            };
            report.metrics.insert(name.into(), observed);
        }
        // KINSol received the live vector populated from either the explicit initial
        // coordinates or the admitted optional warm payload. Submission is distinct
        // from native convergence and from retaining a previous solver allocation.
        report.evidence.start_submitted = true;
        report
            .metrics
            .insert("start.submitted".into(), Metric::Bool(true));
        report
            .provenance
            .insert("native".into(), crate::sundials_version());
        // The typed method's serde encoding, never Rust `Debug` output (F30).
        if let Ok(method) = serde_json::to_string(&self.settings.method) {
            report.provenance.insert("settings".into(), method);
        }
        report.metrics.insert(
            "KINSetScaledStepTol.requested".into(),
            Metric::Real(self.settings.step_tolerance),
        );
        report.provenance.insert(
            "scaled_step_policy".into(),
            if self.settings.step_tolerance == 0.0 {
                "native_roundoff_default"
            } else {
                "explicit_native_relative_stagnation"
            }
            .into(),
        );
        // Native-exit failure and fresh original validation are distinct
        // observations. Preserve the former before validation replaces its
        // worker-local witness; finalize inclusive counters after both phases.
        self.callback.state.retain_failure(&mut report);
        // SAFETY: the session's live iterate vector of the `n` coordinates.
        let x = shifted(unsafe { values(self.x, n) }?, &self.callback.offsets, 1.0);
        if x.iter().all(|v| v.is_finite()) {
            if self.callback.state.terminal.is_none() {
                let mut f = vec![0.0; n];
                let evaluation = self
                    .callback
                    .state
                    .evaluate("native.validation.residual", || {
                        match &mut self.callback.function {
                            Function::Equations(o) => o.residual(&x, &mut f),
                            Function::Picard { oracle, .. } => oracle.residual(&x, &mut f),
                            Function::FixedPoint(o) => {
                                let result =
                                    crate::quality::contained(|| o.original_residual(&x, &mut f));
                                let produced = result.is_ok();
                                let result = result.and_then(|()| {
                                    if f.iter().all(|value| value.is_finite()) {
                                        Ok(())
                                    } else {
                                        Err(ProblemError::numerical(
                                            "nonfinite original map residual",
                                        ))
                                    }
                                });
                                if let Some(history) = &mut self.callback.raw_map {
                                    history.record(
                                        o.contract(),
                                        &x,
                                        produced.then_some(f.as_slice()),
                                        RawMapRole::OriginalAssessment,
                                        result.as_ref().err().map(raw_map_failure),
                                    );
                                }
                                result
                            }
                        }
                    });
                match evaluation {
                    Some(()) => {
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
                                        report.record_validation_failure(e);
                                        report.termination.assurance = Assurance::None;
                                    }
                                }
                            }
                            Err(e) => {
                                report.record_validation_failure(e);
                                report.termination.assurance = Assurance::None;
                            }
                        }
                    }
                    None => {
                        let e = self
                            .callback
                            .state
                            .terminal_error()
                            .or_else(|| self.callback.state.last_failure.take())
                            .unwrap_or_else(|| {
                                ProblemError::Internal("native residual validation failed".into())
                            });
                        report.record_validation_failure(e);
                        report.termination.assurance = Assurance::None
                    }
                }
            }
            report.candidate = Some(Candidate {
                kind: CandidateKind::FinalIterate,
                primal: x.clone(),
                objective: None,
                row_dual: None,
                bound_dual: None,
                reduced_costs: None,
                slacks: None,
                commitment: None,
            });
            report.warm_start = Some(WarmStart {
                origin: None,
                compatibility: self.compatibility.clone(),
                payload: WarmPayload::Root(x),
            });
        } else {
            report.termination.assurance = Assurance::None
        }
        // Final original residual validation belongs to this same inclusive owner.
        self.callback.state.finish(&mut report);
        report.evidence.raw_map = self.callback.raw_map.take().map(RawMapCapture::finish);
        // Independent applications never request or consume predecessor setup.
        // Actual setup callbacks below may still be zero on an early native exit.
        for name in [
            "setup.initial_reuse_requested",
            "setup.initial_reused",
            "setup.input_stale",
        ] {
            report.metrics.insert(name.into(), Metric::Bool(false));
        }
        report.metrics.insert(
            "setup.refreshes".into(),
            Metric::Integer(i64::try_from(self.callback.setup_calls).unwrap_or(i64::MAX)),
        );
        if let Some(blocks) = &self.callback.blocks {
            for (name, count) in [
                ("preconditioner.block.factor_calls", blocks.factor_calls),
                (
                    "preconditioner.block.factors_completed",
                    blocks.factors_completed,
                ),
                ("preconditioner.block.solve_calls", blocks.solve_calls),
            ] {
                report.metrics.insert(
                    name.into(),
                    i64::try_from(count).map_or(
                        Metric::Unavailable(UnavailableReason::Unknown),
                        Metric::Integer,
                    ),
                );
            }
            report.metrics.insert(
                "preconditioner.block.count".into(),
                i64::try_from(blocks.count()).map_or(
                    Metric::Unavailable(UnavailableReason::Unknown),
                    Metric::Integer,
                ),
            );
            // Native Krylov internal factorizations are not separately reported.
            // These subtotal metrics never replace unknown complete work evidence.
        }
        report.metrics.insert(
            "setup.monitoring".into(),
            Metric::Text(
                if method.strategy == Strategy::FixedPoint {
                    "not-applicable"
                } else if method.linear.krylov().is_some() {
                    "native-forcing-and-refresh"
                } else {
                    "native-residual"
                }
                .into(),
            ),
        );
        let output_stale = self.callback.setup.as_ref().is_some_and(|s| {
            s.data != self.compatibility.data
                || report
                    .candidate
                    .as_ref()
                    .is_none_or(|candidate| s.point != candidate.primal)
        });
        report
            .metrics
            .insert("setup.output_stale".into(), Metric::Bool(output_stale));
        report
            .metrics
            .insert("setup.fresh_response_factor".into(), Metric::Bool(false));
        Ok(report)
    }
}
/// Preserve KINSOL's distinct residual, step-size and initialization exits.
pub fn termination(code: i32) -> NativeTermination {
    let (name, category) = match code {
        ffi::KIN_SUCCESS => ("KIN_SUCCESS", Termination::Success),
        ffi::KIN_INITIAL_GUESS_OK => ("KIN_INITIAL_GUESS_OK", Termination::Success),
        ffi::KIN_STEP_LT_STPTOL => ("KIN_STEP_LT_STPTOL", Termination::Limit),
        ffi::KIN_MEM_NULL => ("KIN_MEM_NULL", Termination::Invalid),
        ffi::KIN_ILL_INPUT => ("KIN_ILL_INPUT", Termination::Invalid),
        ffi::KIN_NO_MALLOC => ("KIN_NO_MALLOC", Termination::Invalid),
        ffi::KIN_MEM_FAIL => ("KIN_MEM_FAIL", Termination::ResourceExhausted),
        ffi::KIN_LINESEARCH_NONCONV => ("KIN_LINESEARCH_NONCONV", Termination::Numerical),
        ffi::KIN_MAXITER_REACHED => ("KIN_MAXITER_REACHED", Termination::IterationLimit),
        ffi::KIN_MXNEWT_5X_EXCEEDED => ("KIN_MXNEWT_5X_EXCEEDED", Termination::Numerical),
        ffi::KIN_LINESEARCH_BCFAIL => ("KIN_LINESEARCH_BCFAIL", Termination::Numerical),
        ffi::KIN_LINSOLV_NO_RECOVERY => ("KIN_LINSOLV_NO_RECOVERY", Termination::Numerical),
        ffi::KIN_LINIT_FAIL => ("KIN_LINIT_FAIL", Termination::Numerical),
        ffi::KIN_LSETUP_FAIL => ("KIN_LSETUP_FAIL", Termination::Numerical),
        ffi::KIN_LSOLVE_FAIL => ("KIN_LSOLVE_FAIL", Termination::Numerical),
        ffi::KIN_SYSFUNC_FAIL => ("KIN_SYSFUNC_FAIL", Termination::Evaluation),
        ffi::KIN_FIRST_SYSFUNC_ERR => ("KIN_FIRST_SYSFUNC_ERR", Termination::Evaluation),
        ffi::KIN_REPTD_SYSFUNC_ERR => ("KIN_REPTD_SYSFUNC_ERR", Termination::Evaluation),
        ffi::KIN_VECTOROP_ERR => ("KIN_VECTOROP_ERR", Termination::Numerical),
        ffi::KIN_CONTEXT_ERR => ("KIN_CONTEXT_ERR", Termination::Invalid),
        ffi::KIN_WARNING => ("KIN_WARNING", Termination::Inconclusive),
        // Only an integer undeclared by the pinned SUNDIALS header reaches this arm.
        _ => ("KIN_UNKNOWN", Termination::Inconclusive),
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
    use pse_model::scalars::{Fraction, PositiveCount};
    fn positive(n: usize) -> PositiveCount {
        PositiveCount::try_new(n).unwrap()
    }
    fn fraction(v: f64) -> Fraction {
        Fraction::try_new(v).unwrap()
    }
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
        assert_eq!(sign_constraints(&c).unwrap().signs, vec![1.0]);
        c.variables[0].upper = 1.0;
        assert!(sign_constraints(&c).is_err());
    }
    #[test]
    fn strict_guards_are_native_constraints_without_bound_relaxation() {
        let mut c = crate::solver_tests::Polynomial::new().c;
        let id = c.variables[0].id;
        let guard = std::collections::BTreeMap::from([(
            id,
            pse_math::presolve::GuardSign {
                positive: true,
                strict: true,
            },
        )]);
        assert_eq!(
            guarded_sign_constraints(&c, &guard).unwrap().signs,
            vec![2.0]
        );
        c.variables[0].upper = 0.0;
        assert!(guarded_sign_constraints(&c, &guard).is_err());
    }
    fn execution() -> Execution {
        let mut execution = crate::solver_tests::execution();
        execution.memory = Some(64 << 20);
        execution
    }
    #[test]
    fn native_krylov_storage_refuses_unowned_peak_before_callbacks() {
        let dimension = positive(3);
        for linear in [
            Linear::Spgmr { dimension },
            Linear::Spfgmr { dimension },
            Linear::Spbcgs { dimension },
            Linear::Sptfqmr { dimension },
        ] {
            let peak = native_krylov_storage(1, linear).unwrap();
            for preconditioner in [Preconditioner::None, Preconditioner::Jacobi] {
                let method = Method {
                    linear,
                    preconditioner,
                    ..Method::default()
                };
                for memory in [None, Some(0), Some(peak - 1)] {
                    let oracle = FaultOracle::new(usize::MAX);
                    let calls = oracle.calls.clone();
                    let mut denied = execution();
                    denied.memory = memory;
                    assert!(matches!(
                        Session::new(
                            Function::Equations(Box::new(oracle)),
                            Settings {
                                method,
                                ..settings()
                            },
                            denied,
                            crate::solver_tests::stamp(Backend::Kinsol)
                        ),
                        Err(ProblemError::Limit {
                            kind: crate::LimitKind::Memory,
                            ..
                        })
                    ));
                    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
                }
                let mut admitted = execution();
                admitted.memory = Some(peak);
                let oracle = FaultOracle::new(usize::MAX);
                let calls = oracle.calls.clone();
                let session = Session::new(
                    Function::Equations(Box::new(oracle)),
                    Settings {
                        method,
                        ..settings()
                    },
                    admitted,
                    crate::solver_tests::stamp(Backend::Kinsol),
                )
                .unwrap();
                assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
                drop(session);
                let oversized = Method {
                    linear: Linear::Spfgmr {
                        dimension: positive(1_000_000),
                    },
                    ..method
                };
                let oracle = FaultOracle::new(usize::MAX);
                let calls = oracle.calls.clone();
                assert!(matches!(
                    Session::new(
                        Function::Equations(Box::new(oracle)),
                        Settings {
                            method: oversized,
                            ..settings()
                        },
                        execution(),
                        crate::solver_tests::stamp(Backend::Kinsol)
                    ),
                    Err(ProblemError::Limit {
                        kind: crate::LimitKind::Memory,
                        ..
                    })
                ));
                assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
            }
        }
        // Picard AA and Krylov coexist: owning either subtotal alone cannot admit
        // the combined native allocation, even though both controls are applicable.
        let linear = Linear::Spgmr { dimension };
        let method = Method {
            strategy: Strategy::Picard,
            linear,
            anderson: 1,
            ..Method::default()
        };
        let base = native_krylov_storage(1, linear).unwrap();
        assert!(anderson_storage(1, method).unwrap().0 > base);
        let oracle = FaultOracle::new(usize::MAX);
        let calls = oracle.calls.clone();
        let split = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 3.)],
        )
        .unwrap();
        let mut denied = execution();
        denied.memory = Some(base);
        assert!(matches!(
            Session::new(
                Function::Picard {
                    oracle: Box::new(oracle),
                    linear: split
                },
                Settings {
                    method,
                    ..settings()
                },
                denied,
                crate::solver_tests::stamp(Backend::Kinsol)
            ),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
    #[test]
    fn native_anderson_storage_refuses_unowned_peak_before_callbacks() {
        for orthogonalization in [
            Orthogonalization::ModifiedGramSchmidt,
            Orthogonalization::InverseCompactWy,
        ] {
            let method = Method {
                strategy: Strategy::FixedPoint,
                anderson: 1,
                orthogonalization,
                ..Method::default()
            };
            let (peak, omitted) = anderson_storage(1, method).unwrap();
            for memory in [None, Some(0), Some(peak - 1)] {
                let oracle = FaultOracle::new(usize::MAX);
                let calls = oracle.calls.clone();
                let mut denied = execution();
                denied.memory = memory;
                assert!(matches!(
                    Session::new(
                        Function::FixedPoint(Box::new(oracle)),
                        Settings {
                            method,
                            ..settings()
                        },
                        denied,
                        crate::solver_tests::stamp(Backend::Kinsol)
                    ),
                    Err(ProblemError::Limit {
                        kind: crate::LimitKind::Memory,
                        ..
                    })
                ));
                assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
            }
            let mut admitted = execution();
            admitted.memory = Some(peak);
            let mut session = Session::new(
                Function::FixedPoint(Box::new(FaultOracle::new(usize::MAX))),
                Settings {
                    method,
                    ..settings()
                },
                admitted,
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let report = solve_fault(&mut session, 0.0);
            solved(&report);
            assert!(
                session.retained_bytes() >= omitted + native_header_bytes(method.linear).unwrap()
            );
        }
        assert!(
            anderson_storage(
                usize::MAX,
                Method {
                    anderson: usize::MAX,
                    ..Method::default()
                }
            )
            .is_err()
        );
    }
    fn settings() -> Settings {
        Settings {
            method: Method::default(),
            variable_scales: vec![1.0],
            residual_scales: vec![1.0],
            step_tolerance: 1e-8,
        }
    }
    #[derive(Debug)]
    struct CountedAffineRoot {
        inner: crate::solver_tests::Polynomial,
        target: f64,
        setups: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    impl NleOracle for CountedAffineRoot {
        fn contract(&self) -> &OracleContract {
            &self.inner.c
        }
        fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x[0] - self.target;
            Ok(())
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.inner.matrix.symbolic()
        }
        fn jacobian(&mut self, _x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.setups
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            out[0] = 1.0;
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            _x: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            out[0] = v[0];
            Ok(())
        }
    }
    fn affine_function(
        target: f64,
        setups: &std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> Function {
        Function::Equations(Box::new(CountedAffineRoot {
            inner: crate::solver_tests::Polynomial::new(),
            target,
            setups: setups.clone(),
        }))
    }
    fn setup_solve(session: &mut Session, initial: f64, reuse: ReusePolicy) -> SolveReport {
        session
            .solve(
                &[initial],
                &Controls {
                    reuse,
                    ..Controls::default()
                },
                &ResolvedAccuracy::verification(),
                execution(),
                &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .unwrap()
    }
    #[test]
    fn retained_allocation_requires_fresh_numeric_setup_for_every_application() {
        for (linear, preconditioner) in [
            (Linear::Klu, Preconditioner::None),
            (
                Linear::Dense {
                    limit: positive(10),
                },
                Preconditioner::None,
            ),
            (
                Linear::Spgmr {
                    dimension: positive(4),
                },
                Preconditioner::Jacobi,
            ),
            (
                Linear::Spgmr {
                    dimension: positive(4),
                },
                Preconditioner::BlockFactor,
            ),
        ] {
            let setups = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut config = settings();
            config.method.linear = linear;
            config.method.preconditioner = preconditioner;
            let stamp = crate::solver_tests::stamp(Backend::Kinsol);
            let mut session = Session::new(
                affine_function(1.0, &setups),
                config.clone(),
                execution(),
                stamp.clone(),
            )
            .unwrap();
            let scratch_addresses = (
                session.callback.point.as_ptr(),
                session.callback.output.as_ptr(),
            );
            let cold = setup_solve(&mut session, 0.0, ReusePolicy::AllowRebuild);
            assert_eq!(cold.termination.category, Termination::Success);
            assert!(metric(&cold, "setup.refreshes") > 0);
            let native_address = session.linear;
            let before = setups.load(std::sync::atomic::Ordering::SeqCst);
            let mut next = stamp.clone();
            next.data = pse_ids::ContentHash::from_bytes([51; 32]);
            session
                .replace(affine_function(2.0, &setups), config.clone(), next)
                .unwrap();
            assert!(session.callback.setup.is_none());
            assert!(session.callback.blocks.is_none());
            assert_eq!(session.callback.inverse_diagonal, vec![1.0]);
            let reused = setup_solve(&mut session, 0.0, ReusePolicy::AllowRebuild);
            assert_eq!(reused.termination.category, Termination::Success);
            assert_eq!(reused.candidate.as_ref().unwrap().primal, [2.0]);
            assert_eq!(session.linear, native_address);
            assert_eq!(
                (
                    session.callback.point.as_ptr(),
                    session.callback.output.as_ptr()
                ),
                scratch_addresses,
                "changed values reuse complete private callback storage"
            );
            assert!(setups.load(std::sync::atomic::Ordering::SeqCst) > before);
            assert!(metric(&reused, "setup.refreshes") > 0);
            assert_eq!(
                reused.metrics["setup.initial_reuse_requested"],
                Metric::Bool(false)
            );
            assert_eq!(reused.metrics["setup.initial_reused"], Metric::Bool(false));
            assert_eq!(reused.metrics["setup.input_stale"], Metric::Bool(false));
            assert_eq!(
                reused.metrics["setup.fresh_response_factor"],
                Metric::Bool(false)
            );
            let fresh = setup_solve(&mut session, 0.0, ReusePolicy::Fresh);
            assert_eq!(fresh.termination.category, Termination::Success);
            assert!(metric(&fresh, "setup.refreshes") > 0);
            assert!(setups.load(std::sync::atomic::Ordering::SeqCst) > before);
            assert_eq!(fresh.metrics["setup.initial_reused"], Metric::Bool(false));
            let before_repeat = setups.load(std::sync::atomic::Ordering::SeqCst);
            let repeated = setup_solve(&mut session, 0.0, ReusePolicy::RequireReuse);
            assert_eq!(repeated.termination.category, Termination::Success);
            assert!(metric(&repeated, "setup.refreshes") > 0);
            assert!(setups.load(std::sync::atomic::Ordering::SeqCst) > before_repeat);
            assert_eq!(
                repeated.metrics["setup.initial_reuse_requested"],
                Metric::Bool(false)
            );
            if preconditioner == Preconditioner::BlockFactor {
                assert!(metric(&repeated, "preconditioner.block.factor_calls") > 0);
            }
            let before_early = setups.load(std::sync::atomic::Ordering::SeqCst);
            let at_root = setup_solve(&mut session, 2.0, ReusePolicy::AllowRebuild);
            assert_eq!(at_root.termination.name, "KIN_INITIAL_GUESS_OK");
            assert_eq!(
                setups.load(std::sync::atomic::Ordering::SeqCst),
                before_early
            );
            assert!(session.callback.setup.is_none());
            assert_eq!(metric(&at_root, "setup.refreshes"), 0);
            assert_eq!(metric(&at_root, "KINGetNumJacEvals"), 0);
            assert_eq!(at_root.metrics["setup.initial_reused"], Metric::Bool(false));
            if preconditioner == Preconditioner::BlockFactor {
                assert_eq!(metric(&at_root, "preconditioner.block.factor_calls"), 0);
            }
        }
    }
    #[test]
    fn klu_work_getter_reports_failed_guard_singular_factor_and_fresh_application() {
        let setups = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut session = Session::new(
            affine_function(1., &setups),
            settings(),
            execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        // SAFETY: live exclusively owned KLU solver; null matrix is a checked refusal.
        let refused = unsafe { ffi::SUNLinSolSetup_KLU(session.linear, std::ptr::null_mut()) };
        assert_ne!(refused, 0);
        // SAFETY: getter reads only this session's live KLU counters.
        let failed = unsafe { linear_work::snapshot(session.linear) }.unwrap();
        assert_eq!(failed.setup_issued, 1);
        assert_eq!(failed.setup_completed, 0);
        assert_eq!(failed.symbolic_issued, 0);
        for (application, start) in [0., 3.].into_iter().enumerate() {
            let report = setup_solve(&mut session, start, ReusePolicy::AllowRebuild);
            assert_eq!(report.termination.category, Termination::Success);
            let symbolic = i64::from(application == 0);
            assert_eq!(
                report.metrics["klu.symbolic.issued"],
                Metric::Integer(symbolic)
            );
            assert_eq!(
                report.metrics["klu.symbolic.completed"],
                Metric::Integer(symbolic)
            );
            assert_eq!(
                report.metrics["klu.numeric_factor.issued"],
                Metric::Integer(1)
            );
            assert_eq!(
                report.metrics["klu.numeric_factor.completed"],
                Metric::Integer(1)
            );
            assert!(metric(&report, "klu.solve.completed") > 0);
        }
        // SAFETY: this affine fixture owns a live one-entry CSC matrix.
        let column_pointer = unsafe { ffi::SUNSparseMatrix_IndexPointers(session.matrix) };
        // SAFETY: this one-column matrix owns two initialized column pointers.
        unsafe { std::slice::from_raw_parts_mut(column_pointer, 2) }.copy_from_slice(&[0, 1]);
        // SAFETY: the same live matrix owns its one row-index entry.
        let row_pointer = unsafe { ffi::SUNSparseMatrix_IndexValues(session.matrix) };
        // SAFETY: the one-entry row storage is exclusive throughout this test step.
        (unsafe { std::slice::from_raw_parts_mut(row_pointer, 1) })[0] = 0;
        // SAFETY: the same live matrix owns its one numeric entry.
        let data_pointer = unsafe { ffi::SUNSparseMatrix_Data(session.matrix) };
        // SAFETY: the numeric storage is exclusive and holds this one entry.
        (unsafe { std::slice::from_raw_parts_mut(data_pointer, 1) })[0] = 0.;
        // SAFETY: the exclusively owned solver is live; begin starts fresh numeric state.
        let initialized = unsafe { ffi::SUNLinSol_KLUBeginApplication(session.linear) };
        assert_eq!(initialized, 0);
        // SAFETY: safe-return getter on the same live owner.
        let before = unsafe { linear_work::snapshot(session.linear) }.unwrap();
        // SAFETY: admitted canonical singular CSC on this owned solver.
        let singular = unsafe { ffi::SUNLinSolSetup_KLU(session.linear, session.matrix) };
        assert_ne!(singular, 0);
        // SAFETY: getter reports the failed actual factor prefix without initiating work.
        let after = unsafe { linear_work::snapshot(session.linear) }.unwrap();
        assert_eq!(
            after.numeric_factor_issued - before.numeric_factor_issued,
            1
        );
        assert_eq!(
            after.numeric_factor_completed,
            before.numeric_factor_completed
        );
    }
    #[test]
    fn klu_optional_phase_hook_pairs_failed_and_successful_calls_and_clears() {
        unsafe extern "C" fn capture(phase: i32, begin: i32, userdata: *mut c_void) {
            // SAFETY: this test passes its exclusively borrowed twelve scalar slots
            // only for the synchronous native calls below, then clears the hook.
            let counts = unsafe { &mut *userdata.cast::<[u64; 12]>() };
            let index = usize::try_from(phase - 1).unwrap() * 2 + usize::from(begin == 0);
            counts[index] += 1;
        }
        let setups = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let session = Session::new(
            affine_function(1., &setups),
            settings(),
            execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        let mut counts = [0_u64; 12];
        // SAFETY: exact live owner; callback data remains in scope until cleared.
        let observed = unsafe {
            ffi::SUNLinSol_KLUSetObserver(
                session.linear,
                Some(capture),
                std::ptr::from_mut(&mut counts).cast(),
            )
        };
        assert_eq!(observed, 0);
        // SAFETY: this affine fixture owns a live one-entry CSC matrix.
        let column_pointer = unsafe { ffi::SUNSparseMatrix_IndexPointers(session.matrix) };
        // SAFETY: this one-column matrix owns two initialized column pointers.
        unsafe { std::slice::from_raw_parts_mut(column_pointer, 2) }.copy_from_slice(&[0, 1]);
        // SAFETY: the same live matrix owns its one row-index entry.
        let row_pointer = unsafe { ffi::SUNSparseMatrix_IndexValues(session.matrix) };
        // SAFETY: the one-entry row storage is exclusive throughout this test step.
        (unsafe { std::slice::from_raw_parts_mut(row_pointer, 1) })[0] = 0;
        // SAFETY: the same live matrix owns its one numeric entry.
        let data_pointer = unsafe { ffi::SUNSparseMatrix_Data(session.matrix) };
        // SAFETY: the numeric storage is exclusive and holds this one entry.
        (unsafe { std::slice::from_raw_parts_mut(data_pointer, 1) })[0] = 0.;
        // SAFETY: the exclusively owned solver is live; initialize starts fresh numeric state.
        let initialized = unsafe { ffi::SUNLinSolInitialize_KLU(session.linear) };
        assert_eq!(initialized, 0);
        // SAFETY: canonical one-entry singular CSC and live solver share this session.
        let singular = unsafe { ffi::SUNLinSolSetup_KLU(session.linear, session.matrix) };
        assert_ne!(singular, 0);
        assert_eq!(
            &counts[..4],
            &[1, 1, 1, 1],
            "failed factor still ends its active phase"
        );
        // SAFETY: the same one-entry live matrix owns this exclusive numeric storage.
        (unsafe { std::slice::from_raw_parts_mut(data_pointer, 1) })[0] = 1.;
        // SAFETY: same live solver and stable observer; reset only its numeric state.
        let initialized = unsafe { ffi::SUNLinSolInitialize_KLU(session.linear) };
        assert_eq!(initialized, 0);
        // SAFETY: the now-nonsingular canonical matrix belongs to this live solver.
        let factored = unsafe { ffi::SUNLinSolSetup_KLU(session.linear, session.matrix) };
        assert_eq!(factored, 0);
        // SAFETY: clear this exact live owner's hook before releasing its userdata.
        let cleared =
            unsafe { ffi::SUNLinSol_KLUSetObserver(session.linear, None, std::ptr::null_mut()) };
        assert_eq!(cleared, 0);
        assert_eq!(&counts[..4], &[1, 1, 2, 2]);
        // SAFETY: live solver after hook clear cannot invoke the former userdata.
        let initialized = unsafe { ffi::SUNLinSolInitialize_KLU(session.linear) };
        assert_eq!(initialized, 0);
        // SAFETY: the unchanged canonical matrix and solver are still exclusively live.
        let factored = unsafe { ffi::SUNLinSolSetup_KLU(session.linear, session.matrix) };
        assert_eq!(factored, 0);
        assert_eq!(&counts[..4], &[1, 1, 2, 2]);
    }
    #[test]
    fn setup_interval_profiles_preserve_original_acceptance_and_expose_actual_work() {
        assert_eq!(
            Method::default().setup_interval,
            10,
            "retained monitored global default"
        );
        for interval in [1, 2, 3, 10] {
            let mut config = settings();
            config.method.setup_interval = interval;
            let mut session = Session::new(
                Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
                config,
                execution(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let report = setup_solve(&mut session, 0.25, ReusePolicy::AllowRebuild);
            assert_eq!(
                report.termination.category,
                Termination::Success,
                "interval{interval}"
            );
            assert!(report.quality.as_ref().unwrap().feasible());
            assert_eq!(
                report.metrics["setup.initial_reuse_requested"],
                Metric::Bool(false)
            );
            assert_eq!(report.metrics["klu.symbolic.completed"], Metric::Integer(1));
            eprintln!(
                "setup_interval={interval} iterations={:?} evaluations={:?} setup={:?} fresh_factor={:?} refactor={:?} solve={:?}",
                report.metrics["KINGetNumNonlinSolvIters"],
                report.metrics["KINGetNumFuncEvals"],
                report.metrics["klu.setup.completed"],
                report.metrics["klu.numeric_factor.completed"],
                report.metrics["klu.refactor.completed"],
                report.metrics["klu.solve.completed"]
            );
        }
    }
    #[test]
    fn fresh_application_setup_preserves_within_call_native_monitoring() {
        let setups = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut config = settings();
        config.method.setup_interval = 2;
        let stamp = crate::solver_tests::stamp(Backend::Kinsol);
        let mut session = Session::new(
            affine_function(1.0, &setups),
            config.clone(),
            execution(),
            stamp.clone(),
        )
        .unwrap();
        let cold = setup_solve(&mut session, 0.0, ReusePolicy::AllowRebuild);
        assert_eq!(cold.termination.category, Termination::Success);
        let mut changed = stamp.clone();
        changed.data = pse_ids::ContentHash::from_bytes([52; 32]);
        session
            .replace(
                Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
                config.clone(),
                changed,
            )
            .unwrap();
        let monitored = setup_solve(&mut session, 0.25, ReusePolicy::AllowRebuild);
        assert_eq!(monitored.termination.category, Termination::Success);
        assert!(
            metric(&monitored, "setup.refreshes") > 1,
            "native within-call setup interval must refresh"
        );
        assert_eq!(
            monitored.metrics["setup.initial_reused"],
            Metric::Bool(false)
        );
        assert_eq!(
            monitored.metrics["setup.monitoring"],
            Metric::Text("native-residual".into())
        );
        assert_eq!(
            monitored.metrics["setup.fresh_response_factor"],
            Metric::Bool(false)
        );
        let mut profile = stamp.clone();
        profile.profile = pse_ids::ContentHash::from_bytes([53; 32]);
        assert!(!session.matches_layout(&profile));
        assert!(
            session
                .replace(affine_function(1.0, &setups), config, profile)
                .is_err()
        );
        assert!(
            session.callback.setup.is_none(),
            "refused replacement invalidates prior setup policy"
        );
        assert!(session.callback.blocks.is_none());
    }
    #[test]
    fn reused_native_counters_do_not_repeat_prior_attempt_before_reset() {
        for linear in [
            Linear::Klu,
            Linear::Dense {
                limit: positive(10),
            },
            Linear::Spgmr {
                dimension: positive(4),
            },
        ] {
            let mut config = settings();
            config.method.linear = linear;
            if linear.krylov().is_some() {
                config.method.preconditioner = Preconditioner::Jacobi;
            }
            let stamp = crate::solver_tests::stamp(Backend::Kinsol);
            let setups = std::sync::Arc::default();
            let mut session = Session::new(
                affine_function(1., &setups),
                config.clone(),
                execution(),
                stamp.clone(),
            )
            .unwrap();
            let cold = setup_solve(&mut session, 0., ReusePolicy::AllowRebuild);
            assert!(cold.evidence.work.iterations.is_some_and(|n| n > 0));
            assert!(
                metric(
                    &cold,
                    if linear.krylov().is_some() {
                        "KINGetNumJtimesEvals"
                    } else {
                        "KINGetNumJacEvals"
                    }
                ) > 0
            );
            session
                .replace(
                    Function::Equations(Box::new(FaultOracle::new(1))),
                    config.clone(),
                    stamp.clone(),
                )
                .unwrap();
            let failed = setup_solve(&mut session, 0., ReusePolicy::AllowRebuild);
            let NativeCounters::Kinsol {
                iterations,
                function_evaluations,
                jacobian_evaluations,
                linear_iterations,
                preconditioner_evaluations,
                preconditioner_solves,
                jacobian_products,
                ..
            } = &failed
                .evidence
                .native_observations
                .as_ref()
                .unwrap()
                .counters
            else {
                panic!("KINSOL facts");
            };
            assert_eq!(*iterations, Some(0));
            assert_eq!(*function_evaluations, Some(1));
            for value in [
                jacobian_evaluations,
                linear_iterations,
                preconditioner_evaluations,
                preconditioner_solves,
                jacobian_products,
            ] {
                assert_eq!(*value, Some(0));
            }
            assert_eq!(metric(&failed, "KINGetNumJacEvals"), 0);
            assert_eq!(
                failed.metrics["KINGetLastLinFlag"],
                Metric::Unavailable(UnavailableReason::Unknown)
            );
            assert_eq!(
                failed.metrics["KINGetFuncNorm"],
                Metric::Unavailable(UnavailableReason::Unknown)
            );
            // Populate a separately constrained session, then refuse a bad start
            // before even the primary native counters are reset.
            let mut inner = crate::solver_tests::Polynomial::new();
            inner.c.variables[0].lower = 0.;
            let function = Function::Equations(Box::new(CountedAffineRoot {
                inner,
                target: 1.,
                setups: setups.clone(),
            }));
            let mut constrained = Session::new(function, config, execution(), stamp).unwrap();
            let cold = setup_solve(&mut constrained, 0.25, ReusePolicy::AllowRebuild);
            assert!(cold.evidence.work.iterations.is_some_and(|n| n > 0));
            let illegal = setup_solve(&mut constrained, -1., ReusePolicy::AllowRebuild);
            assert_eq!(illegal.termination.code, i64::from(ffi::KIN_ILL_INPUT));
            assert_eq!(illegal.evidence.work.iterations, None);
            let NativeCounters::Kinsol {
                iterations,
                function_evaluations,
                jacobian_evaluations,
                linear_iterations,
                ..
            } = &illegal
                .evidence
                .native_observations
                .as_ref()
                .unwrap()
                .counters
            else {
                panic!("KINSOL facts");
            };
            for value in [
                iterations,
                function_evaluations,
                jacobian_evaluations,
                linear_iterations,
            ] {
                assert_eq!(*value, None);
            }
            for name in [
                "KINGetNumNonlinSolvIters",
                "KINGetNumFuncEvals",
                "KINGetNumJacEvals",
                "KINGetNumLinIters",
                "KINGetLastLinFlag",
                "KINGetFuncNorm",
                "KINGetStepLength",
            ] {
                assert_eq!(
                    illegal.metrics[name],
                    Metric::Unavailable(UnavailableReason::Unknown)
                );
            }
        }
    }
    #[derive(Debug)]
    struct FaultOracle {
        inner: crate::solver_tests::Polynomial,
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        fail_at: usize,
        reject_above: Option<f64>,
        distinguish_failures: bool,
        validation_bias: Option<(usize, f64)>,
    }
    impl FaultOracle {
        fn new(fail_at: usize) -> Self {
            Self {
                inner: crate::solver_tests::Polynomial::new(),
                calls: Default::default(),
                fail_at,
                reject_above: None,
                distinguish_failures: false,
                validation_bias: None,
            }
        }
        fn trial(&self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            let call = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if call == self.fail_at || self.reject_above.is_some_and(|limit| x[0] > limit) {
                out[0] = 99.0; // Partial work must never reach the native output.
                return Err(pse_math::MathError::Domain {
                    source_id: if self.distinguish_failures {
                        pse_ids::SemanticId::from_bytes([u8::try_from(call).unwrap(); 16])
                    } else {
                        self.inner.c.variables[0].id
                    },
                    requirement: "admitted trial",
                }
                .into());
            }
            Ok(())
        }
    }
    impl NleOracle for FaultOracle {
        fn contract(&self) -> &OracleContract {
            &self.inner.c
        }
        fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.trial(x, out)?;
            NleOracle::residual(&mut self.inner, x, out)?;
            if let Some((at, bias)) = self.validation_bias
                && self.calls.load(std::sync::atomic::Ordering::SeqCst) == at
            {
                out[0] += bias;
            }
            Ok(())
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.inner.matrix.symbolic()
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            NleOracle::jacobian(&mut self.inner, x, out)
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            NleOracle::jacobian_product(&mut self.inner, x, v, out)
        }
    }
    impl FixedPointOracle for FaultOracle {
        fn contract(&self) -> &OracleContract {
            &self.inner.c
        }
        fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.inner.matrix.symbolic()
        }
        fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.trial(x, out)?;
            out[0] = 1.0;
            Ok(())
        }
        fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            NleOracle::residual(self, x, out)
        }
    }
    fn fault_session(strategy: Strategy, oracle: FaultOracle) -> Session {
        let function = match strategy {
            Strategy::FixedPoint => Function::FixedPoint(Box::new(oracle)),
            Strategy::Picard => Function::Picard {
                oracle: Box::new(oracle),
                linear: faer::sparse::SparseColMat::try_new_from_triplets(
                    1,
                    1,
                    &[faer::sparse::Triplet::new(0, 0, 3.0)],
                )
                .unwrap(),
            },
            _ => Function::Equations(Box::new(oracle)),
        };
        Session::new(
            function,
            Settings {
                method: Method {
                    strategy,
                    ..Method::default()
                },
                ..settings()
            },
            execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap()
    }
    fn solve_fault(session: &mut Session, start: f64) -> SolveReport {
        session
            .solve(
                &[start],
                &Controls::default(),
                &ResolvedAccuracy::verification(),
                execution(),
                &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .unwrap()
    }
    #[test]
    fn compact_root_progress_reused_session_refreshes_attempt_role_and_source() {
        let mut session = fault_session(Strategy::Newton, FaultOracle::new(usize::MAX));
        let original = session.callback.progress_layout.as_ref().unwrap().clone();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let mut sources = Vec::new();
        for role in [
            pse_diagnostics::observation::ProducerRole::NativeCallback,
            pse_diagnostics::observation::ProducerRole::OriginalOracle,
        ] {
            let operation = hub.start("root progress attempt");
            let _entered = operation.context().enter();
            let mut scope = execution();
            scope.callback_role = role;
            scope.progress = std::sync::Arc::new(Progress::new(64));
            let report = session
                .solve(
                    &[1.0],
                    &Controls::default(),
                    &ResolvedAccuracy::verification(),
                    scope,
                    &Tolerances {
                        variables: vec![1e-8],
                        rows: vec![1e-8],
                        integrality: 1e-8,
                    },
                    None,
                )
                .unwrap();
            solved(&report);
            assert!(!std::sync::Arc::ptr_eq(
                &original,
                session.callback.progress_layout.as_ref().unwrap()
            ));
            let first = report
                .events
                .iter()
                .find(|event| {
                    event
                        .source
                        .as_ref()
                        .is_some_and(|source| source.family == "root_evaluation.v1")
                })
                .unwrap();
            assert_eq!(first.values["evaluation_ordinal"], Metric::Integer(1));
            assert_eq!(
                first.values["output_role"],
                Metric::Text("equation_residual".into())
            );
            let source = first.source.as_ref().unwrap();
            assert_eq!(source.role, role);
            assert!(source.producer.is_some());
            sources.push(source.clone());
        }
        assert_ne!(sources[0].generation, sources[1].generation);
        assert_ne!(sources[0].producer, sources[1].producer);
        let fixed_point = fault_session(Strategy::FixedPoint, FaultOracle::new(usize::MAX));
        assert!(fixed_point.callback.progress_layout.is_none());
    }
    #[test]
    fn raw_map_failed_evaluation_retains_previous_success_despite_dropped_telemetry() {
        let oracle = FaultOracle::new(2);
        let mut session = fault_session(Strategy::FixedPoint, oracle);
        let controls = Controls {
            history: 2,
            ..Controls::default()
        };
        let mut execution = execution();
        execution.progress = std::sync::Arc::new(Progress::new(0));
        let report = session
            .solve(
                &[0.],
                &controls,
                &ResolvedAccuracy::verification(),
                execution,
                &Tolerances {
                    variables: vec![0.25],
                    rows: vec![0.25],
                    integrality: 0.25,
                },
                None,
            )
            .unwrap();
        assert_eq!(report.termination.category, Termination::Evaluation);
        assert!(report.events.is_empty());
        let raw = report.evidence.raw_map.as_ref().unwrap();
        assert_eq!(raw.samples.len(), 2);
        assert!(matches!(raw.samples[1].failure, Some(RawMapFailure::Trial)));
        assert_eq!(raw.samples[1].values[0].mapped, None);
        assert_eq!(raw.last_successful.as_ref().unwrap().values[0].input, 0.);
        assert_eq!(
            raw.last_successful.as_ref().unwrap().values[0].mapped,
            Some(1.)
        );
        let NativeCounters::Kinsol {
            fixed_point_iterations,
            iterations,
            function_evaluations,
            linear_iterations,
            ..
        } = &report
            .evidence
            .native_observations
            .as_ref()
            .unwrap()
            .counters
        else {
            panic!("KINSOL facts");
        };
        assert!(*fixed_point_iterations);
        assert_eq!(*iterations, Some(2));
        assert_eq!(*function_evaluations, Some(2));
        assert_eq!(*linear_iterations, None);
        assert_eq!(report.evidence.work.iterations, Some(2));
    }
    #[derive(Clone, Copy, Debug)]
    enum RevealingMap {
        Affine(f64),
        NoisyBranch,
        DomainLeaving,
    }
    impl RevealingMap {
        fn value(self, x: f64, call: usize) -> Result<f64, ProblemError> {
            match self {
                Self::Affine(coefficient) => Ok(1. + coefficient * x),
                Self::NoisyBranch => Ok(0.5 * x
                    + if x < 1. { 1.2 } else { 0.8 }
                    + if call.is_multiple_of(2) { 0.05 } else { -0.05 }),
                Self::DomainLeaving if x > 1. => Err(pse_math::MathError::Domain {
                    source_id: pse_ids::SemanticId::NIL,
                    requirement: "map input <= 1",
                }
                .into()),
                Self::DomainLeaving => Ok(x + 2.),
            }
        }
    }
    #[derive(Debug)]
    struct ObservedMap {
        inner: crate::solver_tests::Polynomial,
        shape: RevealingMap,
        calls: usize,
        cancel_at: Option<usize>,
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
        original_nonfinite: Option<f64>,
    }
    impl FixedPointOracle for ObservedMap {
        fn contract(&self) -> &OracleContract {
            &self.inner.c
        }
        fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.inner.matrix.symbolic()
        }
        fn physical_tear_coordinates(&self) -> bool {
            true
        }
        fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.calls += 1;
            out[0] = self.shape.value(x[0], self.calls)?;
            if self.cancel_at == Some(self.calls) {
                self.cancel
                    .store(true, std::sync::atomic::Ordering::Release);
            }
            Ok(())
        }
        fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = self
                .original_nonfinite
                .unwrap_or(self.shape.value(x[0], self.calls)? - x[0]);
            Ok(())
        }
    }
    fn observed_map_report(
        shape: RevealingMap,
        anderson: usize,
        cancel_at: Option<usize>,
    ) -> SolveReport {
        let controls = Controls {
            iterations: 8,
            history: 2,
            ..Controls::default()
        };
        let mut execution = execution();
        execution.progress = std::sync::Arc::new(Progress::new(0));
        let method = Method {
            strategy: Strategy::FixedPoint,
            anderson,
            ..Method::default()
        };
        if anderson > 0 {
            execution.memory = Some(anderson_storage(1, method).unwrap().0);
        }
        let oracle = ObservedMap {
            inner: crate::solver_tests::Polynomial::new(),
            shape,
            calls: 0,
            cancel_at,
            cancel: execution.cancel.clone(),
            original_nonfinite: None,
        };
        let mut session = Session::new(
            Function::FixedPoint(Box::new(oracle)),
            Settings {
                method,
                ..settings()
            },
            execution.clone(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        session
            .solve(
                &[0.],
                &controls,
                &ResolvedAccuracy::verification(),
                execution,
                &Tolerances {
                    variables: vec![0.25],
                    rows: vec![0.25],
                    integrality: 0.25,
                },
                None,
            )
            .unwrap()
    }
    #[test]
    fn raw_map_scientific_shapes_retain_actual_physical_error_despite_acceleration_and_truncation()
    {
        for (shape, anderson) in [
            (RevealingMap::Affine(0.5), 0),
            (RevealingMap::Affine(-0.5), 0),
            (RevealingMap::Affine(-1.2), 1),
            (RevealingMap::NoisyBranch, 0),
        ] {
            let report = observed_map_report(shape, anderson, None);
            assert!(report.events.is_empty());
            let history = report.evidence.raw_map.as_ref().unwrap();
            assert_eq!(history.samples.len(), 2);
            assert!(history.omitted > 0);
            for sample in history.samples.iter().chain(history.last_successful.iter()) {
                assert_eq!(sample.role, RawMapRole::FixedPointEvaluation);
                assert_eq!(sample.coordinates, ObservationCoordinates::PhysicalTears);
                assert!(sample.failure.is_none());
                let coordinate = &sample.values[0];
                let mapped = shape
                    .value(coordinate.input, sample.ordinal as usize)
                    .unwrap();
                assert_eq!(coordinate.mapped, Some(mapped));
                assert_eq!(coordinate.residual, Some(mapped - coordinate.input));
                assert_eq!(coordinate.physical_tolerance, Some(0.25));
                assert_eq!(
                    coordinate.physical_error_ratio,
                    Some((mapped - coordinate.input).abs() / 0.25)
                );
                assert_eq!(sample.inner_start, None);
                assert_eq!(sample.inner_branch, None);
            }
            if anderson > 0 {
                // A larger second raw residual is retained even though the native
                // Anderson successor subsequently converges. No restart law is added.
                assert!(
                    history.samples[1].values[0].residual.unwrap().abs()
                        > history.samples[0].values[0].residual.unwrap().abs()
                );
            }
        }
    }
    #[test]
    fn authorized_recycle_restarts_screen_starts_reset_history_and_preserve_total_cap() {
        let run = |starts: Vec<Vec<f64>>, cap, cancel_at, shape| {
            let controls = Controls {
                iterations: cap,
                history: 3,
                ..Default::default()
            };
            let mut execution = execution();
            let method = Method {
                strategy: Strategy::FixedPoint,
                anderson: 1,
                anderson_delay: 1,
                damping: fraction(0.75),
                ..Default::default()
            };
            execution.memory = Some(anderson_storage(1, method).unwrap().0);
            let oracle = ObservedMap {
                inner: crate::solver_tests::Polynomial::new(),
                shape,
                calls: 0,
                cancel_at,
                cancel: execution.cancel.clone(),
                original_nonfinite: None,
            };
            let mut session = Session::new(
                Function::FixedPoint(Box::new(oracle)),
                Settings {
                    method,
                    ..settings()
                },
                execution.clone(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let schedule = RestartSchedule {
                starts,
                iterations: 1,
            };
            session.solve_with_restarts(
                &[0.0],
                &controls,
                &ResolvedAccuracy::verification(),
                execution,
                &Tolerances {
                    variables: vec![0.01],
                    rows: vec![0.01],
                    integrality: 0.01,
                },
                None,
                Some(&schedule),
            )
        };
        let succeeded = run(vec![vec![2.0]], 2, None, RevealingMap::Affine(0.5)).unwrap();
        assert!(succeeded.quality.as_ref().unwrap().feasible());
        assert_eq!(succeeded.evidence.work.iterations, Some(2));
        assert_eq!(
            succeeded.metrics["recycle.native_attempts"],
            Metric::Integer(2)
        );
        let history = succeeded.evidence.raw_map.as_ref().unwrap();
        assert!(
            history
                .samples
                .iter()
                .any(|sample| sample.role == RawMapRole::RestartScreening)
        );
        assert!(history.samples.len() <= 3);
        let capped = run(
            vec![vec![1.0], vec![2.0]],
            2,
            None,
            RevealingMap::Affine(0.5),
        )
        .unwrap();
        assert_eq!(capped.evidence.work.iterations, Some(2));
        assert_eq!(
            capped.metrics["recycle.native_attempts"],
            Metric::Integer(2)
        );
        assert!(!capped.quality.as_ref().unwrap().feasible());
        let failed_screen = run(vec![vec![2.0]], 3, None, RevealingMap::DomainLeaving).unwrap();
        assert_eq!(
            failed_screen.metrics["recycle.native_attempts"],
            Metric::Integer(1)
        );
        let history = failed_screen.evidence.raw_map.as_ref().unwrap();
        assert_eq!(
            history.last_evaluation.as_ref().unwrap().role,
            RawMapRole::RestartScreening
        );
        assert!(history.last_evaluation.as_ref().unwrap().failure.is_some());
        assert!(history.last_successful.is_some());
        let cancelled = run(vec![vec![2.0]], 3, Some(1), RevealingMap::Affine(0.5)).unwrap();
        assert_eq!(cancelled.termination.category, Termination::Cancelled);
        assert_eq!(
            cancelled.metrics["recycle.native_attempts"],
            Metric::Integer(1)
        );
        assert!(cancelled.evidence.raw_map.is_some());
    }
    #[test]
    fn raw_map_domain_failure_and_cancellation_preserve_last_success_without_successor_work() {
        let failed = observed_map_report(RevealingMap::DomainLeaving, 0, None);
        assert_eq!(failed.termination.category, Termination::Evaluation);
        let history = failed.evidence.raw_map.as_ref().unwrap();
        assert_eq!(failed.evidence.work.iterations, Some(2));
        assert_eq!(
            history.last_successful.as_ref().unwrap().values[0].input,
            0.
        );
        let last = history.last_evaluation.as_ref().unwrap();
        assert_eq!(last.values[0].input, 2.);
        assert!(matches!(last.failure, Some(RawMapFailure::Trial)));
        assert_eq!(last.values[0].mapped, None);
        let cancelled = observed_map_report(RevealingMap::Affine(0.5), 0, Some(2));
        assert_eq!(cancelled.termination.category, Termination::Cancelled);
        assert_eq!(cancelled.evidence.work.evaluations, Some(2));
        assert_eq!(cancelled.evidence.work.iterations, Some(2));
        assert_eq!(
            cancelled
                .evidence
                .raw_map
                .as_ref()
                .unwrap()
                .last_successful
                .as_ref()
                .unwrap()
                .ordinal,
            2
        );
    }
    #[test]
    fn raw_map_nonfinite_original_assessment_retains_value_and_fatal_validation_status() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let execution = execution();
            let oracle = ObservedMap {
                inner: crate::solver_tests::Polynomial::new(),
                shape: RevealingMap::Affine(0.),
                calls: 0,
                cancel_at: None,
                cancel: execution.cancel.clone(),
                original_nonfinite: Some(value),
            };
            let mut session = Session::new(
                Function::FixedPoint(Box::new(oracle)),
                Settings {
                    method: Method {
                        strategy: Strategy::FixedPoint,
                        ..Method::default()
                    },
                    ..settings()
                },
                execution,
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let report = solve_fault(&mut session, 0.);
            assert!(report.validation_failure().is_some());
            let raw = report.evidence.raw_map.as_ref().unwrap();
            let last = raw.last_evaluation.as_ref().unwrap();
            assert_eq!(last.role, RawMapRole::OriginalAssessment);
            assert!(matches!(last.failure, Some(RawMapFailure::Fatal)));
            assert!(!last.values[0].residual.unwrap().is_finite());
            assert_eq!(
                raw.last_successful.as_ref().unwrap().values[0].mapped,
                Some(1.)
            );
            let json = serde_json::to_value(last).unwrap();
            assert_eq!(json["values"][0]["residual"]["kind"], "nonfinite");
        }
    }
    #[derive(Debug)]
    struct EvaluationAdmission {
        cap: u64,
        observed: std::sync::atomic::AtomicU64,
    }
    impl WorkAdmission for EvaluationAdmission {
        fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            let evaluations = work.evaluations.ok_or_else(|| {
                ProblemError::Contract("KINSOL callback evaluation count is unknown".into())
            })?;
            if self
                .observed
                .load(std::sync::atomic::Ordering::SeqCst)
                .checked_add(evaluations)
                .is_none_or(|total| total > self.cap)
            {
                return Err(ProblemError::Limit {
                    kind: crate::LimitKind::Work,
                    detail: "KINSOL test evaluation allowance exhausted".into(),
                });
            }
            Ok(())
        }
        fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            let evaluations = work.evaluations.ok_or_else(|| {
                ProblemError::Contract("KINSOL observed evaluation count is unknown".into())
            })?;
            self.observed
                .try_update(
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                    |total| total.checked_add(evaluations),
                )
                .map_err(|_| ProblemError::Limit {
                    kind: crate::LimitKind::Work,
                    detail: "KINSOL test evaluation counter overflow".into(),
                })?;
            Ok(())
        }
    }
    #[test]
    fn evaluation_coverage_requires_the_actual_callback_owner_and_counts_final_validation() {
        use crate::solve::WorkCoverage;
        use std::sync::Arc;
        let adapter = crate::execution::adapter(Backend::Kinsol);
        assert_eq!(adapter.work_coverage(&execution()), WorkCoverage::default());
        for cap in [0_u64, 1, 2] {
            let admission = Arc::new(EvaluationAdmission {
                cap,
                observed: std::sync::atomic::AtomicU64::new(0),
            });
            let mut scope = execution();
            scope.work_admission = Some(admission.clone());
            assert_eq!(
                adapter.work_coverage(&scope),
                WorkCoverage {
                    evaluations: true,
                    ..WorkCoverage::default()
                }
            );
            scope.callback_work_owner = false;
            assert_eq!(adapter.work_coverage(&scope), WorkCoverage::default());
            scope.callback_work_owner = true;
            let oracle = FaultOracle::new(usize::MAX);
            let calls = oracle.calls.clone();
            let mut session = fault_session(Strategy::Newton, oracle);
            let report = session
                .solve(
                    &[1.0],
                    &Controls::default(),
                    &ResolvedAccuracy::verification(),
                    scope,
                    &Tolerances {
                        variables: vec![1e-8],
                        rows: vec![1e-8],
                        integrality: 1e-8,
                    },
                    None,
                )
                .unwrap();
            assert_eq!(
                calls.load(std::sync::atomic::Ordering::SeqCst),
                cap as usize
            );
            assert_eq!(
                admission.observed.load(std::sync::atomic::Ordering::SeqCst),
                cap
            );
            assert_eq!(report.evidence.work.evaluations, Some(cap));
            if cap < 2 {
                assert_eq!(report.termination.category, Termination::Limit);
                assert!(report.evidence.callback.terminal_failure);
                assert!(matches!(
                    report.callback_failure().or(report.validation_failure()),
                    Some(ProblemError::Limit {
                        kind: crate::LimitKind::Work,
                        ..
                    })
                ));
                assert_eq!(report.validation_failure().is_some(), cap == 1);
            } else {
                solved(&report);
                assert!(report.callback_failure().is_none());
                assert!(report.validation_failure().is_none());
            }
        }
    }
    #[test]
    fn final_residual_validation_is_inclusive_even_when_it_fails() {
        for fail in [false, true] {
            let oracle = FaultOracle::new(if fail { 2 } else { usize::MAX });
            let calls = oracle.calls.clone();
            let mut session = fault_session(Strategy::Newton, oracle);
            let report = solve_fault(&mut session, 1.0);
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
            assert_eq!(report.evidence.work.evaluations, Some(2));
            let checkpoint = report
                .events
                .iter()
                .find(|event| {
                    event
                        .source
                        .as_ref()
                        .is_some_and(|source| source.family == "root_evaluation.v1")
                })
                .unwrap();
            assert_eq!(
                checkpoint.values["output_role"],
                Metric::Text("equation_residual".into())
            );
            assert!(checkpoint.values.contains_key("scalar.0.residual"));
            assert!(!checkpoint.values.contains_key("iteration"));
            assert_eq!(
                report.metrics["callback.native_callback.native.validation.residual.calls"],
                Metric::Integer(1)
            );
            assert_eq!(report.validation_failure().is_some(), fail);
            if fail {
                assert!(matches!(
                    report.validation_failure(),
                    Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
                ));
            }
        }
    }
    #[test]
    fn native_exit_and_final_validation_retain_distinct_domain_witnesses() {
        let mut oracle = FaultOracle::new(usize::MAX);
        oracle.reject_above = Some(0.0);
        oracle.distinguish_failures = true;
        let calls = oracle.calls.clone();
        let mut session = fault_session(Strategy::Newton, oracle);
        let report = solve_fault(&mut session, 1.0);
        assert_eq!(report.termination.category, Termination::Evaluation);
        assert!(crate::callback::retryable_evaluation(&report));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(report.evidence.work.evaluations, Some(2));
        assert!(matches!(report.callback_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain {source_id, ..}))
                if *source_id == pse_ids::SemanticId::from_bytes([1; 16])));
        assert!(matches!(report.validation_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain {source_id, ..}))
                if *source_id == pse_ids::SemanticId::from_bytes([2; 16])));
    }
    #[test]
    fn map_callbacks_latch_failed_trials_but_newton_can_recover() {
        for strategy in [
            Strategy::FixedPoint,
            Strategy::Picard,
            Strategy::Newton,
            Strategy::LineSearch,
        ] {
            let oracle = FaultOracle::new(1);
            let calls = oracle.calls.clone();
            let mut session = fault_session(strategy, oracle);
            // SAFETY: session vectors/context are live, correctly sized and exclusively borrowed.
            {
                // SAFETY: exclusively owned live serial solution vector has one coordinate.
                unsafe { publish(session.x, &[1.0]) }.unwrap();
                // SAFETY: exclusively owned live serial output vector has one coordinate.
                unsafe { publish(session.fs, &[7.0]) }.unwrap();
                let data = (&raw mut *session.callback).cast();
                // SAFETY: live one-coordinate vectors and exclusive callback context belong to this session.
                let first = unsafe { residual(session.x, session.fs, data) };
                let map = matches!(strategy, Strategy::FixedPoint | Strategy::Picard);
                assert_eq!(first, if map { -1 } else { 1 });
                // SAFETY: this session's output is a live one-coordinate serial vector.
                assert_eq!(unsafe { values(session.fs, 1) }.unwrap(), &[7.0]);
                assert_eq!(session.callback.state.terminal.is_some(), map);
                assert!(matches!(
                    session.callback.state.last_failure,
                    Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
                ));
                assert_eq!(
                    // SAFETY: vectors/context remain exclusively session-owned through this second call.
                    unsafe { residual(session.x, session.fs, data) },
                    if map { -1 } else { 0 }
                );
                assert_eq!(
                    calls.load(std::sync::atomic::Ordering::SeqCst),
                    if map { 1 } else { 2 }
                );
                assert_eq!(
                    // SAFETY: live one-coordinate output vector is read between native callbacks.
                    unsafe { values(session.fs, 1) }.unwrap(),
                    if map { &[7.0] } else { &[0.0] }
                );
                assert_eq!(session.callback.state.terminal.is_some(), map);
                assert_eq!(session.callback.state.trial_rejections, 1);
                // A successful callback preserves the trial witness until native completion.
                assert!(matches!(
                    session.callback.state.last_failure,
                    Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
                ));
            }
        }
    }
    #[test]
    fn native_failed_fp_and_picard_do_not_consume_previous_outputs() {
        for strategy in [Strategy::FixedPoint, Strategy::Picard] {
            let oracle = FaultOracle::new(2);
            let calls = oracle.calls.clone();
            let mut session = fault_session(strategy, oracle);
            let report = solve_fault(
                &mut session,
                if strategy == Strategy::FixedPoint {
                    0.0
                } else {
                    1.2
                },
            );
            assert_eq!(report.termination.name, "KIN_SYSFUNC_FAIL", "{report:?}");
            assert_eq!(report.termination.category, Termination::Evaluation);
            assert_eq!(report.termination.assurance, Assurance::None);
            assert!(report.evidence.callback.terminal_failure);
            assert!(matches!(
                report.callback_failure(),
                Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
            ));
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
            assert!(report.quality.is_none());
        }
    }
    #[test]
    fn native_newton_line_search_recovers_domain_trial() {
        let mut oracle = FaultOracle::new(usize::MAX);
        oracle.reject_above = Some(1.5);
        let mut session = fault_session(Strategy::LineSearch, oracle);
        let report = solve_fault(&mut session, 0.5);
        solved(&report);
        assert!(report.evidence.callback.trial_rejections > 0);
        assert!(!report.evidence.callback.terminal_failure);
        assert!(report.callback_failure().is_none());
        // SAFETY: live canonical session objects after the successful application;
        // a recovered trial must leave its valid numerical factor available.
        let substituted = unsafe {
            ffi::SUNLinSolSolve_KLU(session.linear, session.matrix, session.x, session.us, 0.)
        };
        assert_eq!(substituted, 0);
    }
    #[test]
    fn original_validation_failure_resets_numeric_after_native_success() {
        let oracle = FaultOracle::new(usize::MAX);
        let calls = oracle.calls.clone();
        let mut reference = fault_session(Strategy::LineSearch, oracle);
        solved(&solve_fault(&mut reference, 0.5));
        // Deterministic native trajectory is unchanged; only the last independent
        // original validation differs, after native convergence and factorization.
        let last = calls.load(std::sync::atomic::Ordering::SeqCst);
        for domain_failure in [false, true] {
            let mut oracle = FaultOracle::new(if domain_failure { last } else { usize::MAX });
            if !domain_failure {
                oracle.validation_bias = Some((last, 1.));
            }
            let mut session = fault_session(Strategy::LineSearch, oracle);
            let report = solve_fault(&mut session, 0.5);
            assert_eq!(report.termination.category, Termination::Success);
            assert_eq!(report.validation_failure().is_some(), domain_failure);
            assert!(!report.quality.as_ref().is_some_and(Quality::feasible));
            assert_eq!(metric(&report, "klu.numeric_factor.completed"), 1);
            assert!(
                session.ready_for_retention(),
                "failed scientific validation drained and reset"
            );
            // SAFETY: live exclusively owned solver and vectors; missing numerical
            // validity must refuse substitution without invoking native KLU solve.
            let substituted = unsafe {
                ffi::SUNLinSolSolve_KLU(session.linear, session.matrix, session.x, session.us, 0.)
            };
            assert_ne!(substituted, 0);
            let config = session.settings.clone();
            let stamp = session.compatibility.clone();
            session
                .replace(
                    Function::Equations(Box::new(FaultOracle::new(usize::MAX))),
                    config,
                    stamp,
                )
                .unwrap();
            let recovered = solve_fault(&mut session, 0.5);
            solved(&recovered);
            assert_eq!(metric(&recovered, "klu.symbolic.completed"), 0);
            assert_eq!(metric(&recovered, "klu.numeric_factor.completed"), 1);
        }
    }
    #[test]
    fn native_step_tolerance_exit_is_a_limit_with_independent_quality() {
        let mut session = fault_session(Strategy::Newton, FaultOracle::new(usize::MAX));
        session.settings.step_tolerance = 1.0;
        let report = solve_fault(&mut session, 2.0);
        assert_eq!(report.termination.name, "KIN_STEP_LT_STPTOL");
        assert_eq!(report.termination.category, Termination::Limit);
        assert_eq!(report.termination.assurance, Assurance::None);
        assert!(!report.quality.as_ref().unwrap().feasible());
        assert_eq!(
            termination(ffi::KIN_STEP_LT_STPTOL).category,
            Termination::Limit
        );
    }
    #[test]
    fn method_controls_must_act_under_the_selected_strategy() {
        let c = crate::solver_tests::Polynomial::new().c;
        for strategy in [Strategy::Newton, Strategy::LineSearch] {
            for method in [
                Method {
                    strategy,
                    anderson: 1,
                    ..Method::default()
                },
                Method {
                    strategy,
                    damping: fraction(0.5),
                    ..Method::default()
                },
                Method {
                    strategy,
                    anderson: 1,
                    anderson_delay: 1,
                    ..Method::default()
                },
                Method {
                    strategy,
                    anderson: 1,
                    orthogonalization: Orthogonalization::InverseCompactWy,
                    ..Method::default()
                },
            ] {
                assert!(
                    matches!(
                        Settings {
                            method,
                            ..settings()
                        }
                        .validate_contract(
                            &c,
                            Strategy::LineSearch,
                            &Default::default()
                        ),
                        Err(ProblemError::Unsupported(_))
                    ),
                    "{method:?}"
                );
            }
        }
        for strategy in [Strategy::FixedPoint, Strategy::Picard] {
            let active = Method {
                strategy,
                anderson: 1,
                damping: fraction(0.5),
                anderson_delay: 1,
                orthogonalization: Orthogonalization::InverseCompactWy,
                ..Method::default()
            };
            assert!(
                Settings {
                    method: active,
                    ..settings()
                }
                .validate_contract(&c, strategy, &Default::default())
                .is_ok()
            );
            assert!(matches!(
                Settings {
                    method: Method {
                        max_newton_step: Some(1.0),
                        ..active
                    },
                    ..settings()
                }
                .validate_contract(&c, strategy, &Default::default()),
                Err(ProblemError::Unsupported(_))
            ));
        }
        for method in [
            Method {
                linear: Linear::Dense { limit: positive(4) },
                ..Method::default()
            },
            Method {
                setup_interval: 4,
                ..Method::default()
            },
            Method {
                linear: Linear::Spgmr {
                    dimension: positive(4),
                },
                eta: Eta::Constant {
                    value: fraction(0.1),
                },
                ..Method::default()
            },
            Method {
                linear: Linear::Spgmr {
                    dimension: positive(4),
                },
                preconditioner: Preconditioner::Jacobi,
                ..Method::default()
            },
        ] {
            let method = Method {
                strategy: Strategy::FixedPoint,
                ..method
            };
            assert!(matches!(
                Settings {
                    method,
                    ..settings()
                }
                .validate_contract(&c, Strategy::FixedPoint, &Default::default()),
                Err(ProblemError::Unsupported(_))
            ));
        }
        let active_picard = Method {
            strategy: Strategy::Picard,
            setup_interval: 4,
            linear: Linear::Spgmr {
                dimension: positive(4),
            },
            eta: Eta::Constant {
                value: fraction(0.1),
            },
            preconditioner: Preconditioner::Jacobi,
            ..Method::default()
        };
        assert!(
            Settings {
                method: active_picard,
                ..settings()
            }
            .validate_contract(&c, Strategy::Picard, &Default::default())
            .is_ok()
        );
    }
    #[test]
    fn native_allocation_reuses_checked_patterns_and_refuses_arbitrary_boxes() {
        let f = Function::Equations(Box::new(crate::solver_tests::Polynomial::new()));
        let mut s = Session::new(
            f,
            settings(),
            execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        let address = s.mem;
        s.replace(
            Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
            settings(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        assert_eq!(address, s.mem);
        let mut updated = settings();
        updated.method.strategy = Strategy::Newton;
        updated.variable_scales = vec![2.0];
        updated.residual_scales = vec![3.0];
        updated.method.setup_interval = 4;
        updated.method.max_newton_step = Some(5.0);
        updated.step_tolerance = 2e-7;
        assert!(s.matches_settings(&updated));
        s.replace(
            Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
            updated.clone(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        assert_eq!(address, s.mem);
        assert_eq!(s.settings.variable_scales, updated.variable_scales);
        assert_eq!(s.settings.residual_scales, updated.residual_scales);
        assert_eq!(s.settings.method.setup_interval, 4);
        updated.method.anderson += 1;
        assert!(!s.matches_settings(&updated));
        let mut o = crate::solver_tests::Polynomial::new();
        o.c.variables[0].lower = -3.0;
        o.c.variables[0].upper = 3.0;
        assert!(
            s.replace(
                Function::Equations(Box::new(o)),
                settings(),
                crate::solver_tests::stamp(Backend::Kinsol)
            )
            .is_err()
        );
    }
    #[test]
    fn settings_from_policy_reduce_residual_tests_to_original_budgets() {
        let tolerances = Tolerances {
            variables: vec![1e-3, 4.0],
            rows: vec![2e-6, 5e2],
            integrality: 1e-9,
        };
        let normalization = pse_math::normalization::Normalization {
            variables: vec![10.0, 1e3],
            rows: vec![1e-2, 1e4],
            objective: 1.0,
        };
        let feasibility = 1e-7;
        let method = Method {
            strategy: Strategy::Picard,
            linear: Linear::Dense { limit: positive(8) },
            anderson: 1,
            ..Method::default()
        };
        let s = Settings::from_policy(method, &tolerances, &normalization, feasibility);
        assert_eq!(s.method(), method);
        assert_eq!(s.step_tolerance, 0.0);
        for (scales, budgets, coordinates) in [
            (
                &s.variable_scales,
                &tolerances.variables,
                &normalization.variables,
            ),
            (&s.residual_scales, &tolerances.rows, &normalization.rows),
        ] {
            for ((scale, t), c) in scales.iter().zip(budgets).zip(coordinates) {
                assert_eq!(*scale, feasibility * c / t);
                // Scale arithmetic only: the residual test uses this weighted norm;
                // the variable stagnation norm additionally divides by iterate magnitude.
                assert!((scale * (t / c) - feasibility).abs() <= 4.0 * f64::EPSILON * feasibility);
            }
        }
        // Identity coordinates reduce to feasibility / t, bit for bit.
        let identity = pse_math::normalization::Normalization::identity(2, 2);
        let s = Settings::from_policy(Method::default(), &tolerances, &identity, 1.0);
        assert_eq!(s.residual_scales, vec![1.0 / 2e-6, 1.0 / 5e2]);
        assert_eq!(s.method(), Method::default());
        let c = crate::solver_tests::Polynomial::new().c;
        let one = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-8],
            integrality: 1e-9,
        };
        let s = Settings::from_policy(
            Method::default(),
            &one,
            &pse_math::normalization::Normalization::identity(1, 1),
            1e-8,
        );
        assert!(
            s.validate_contract(&c, Strategy::LineSearch, &Default::default())
                .is_ok()
        );
        // Policy dimensions that do not cover the contract are refused, never padded.
        let s = Settings::from_policy(Method::default(), &tolerances, &identity, 1.0);
        assert!(
            s.validate_contract(&c, Strategy::LineSearch, &Default::default())
                .is_err()
        );
    }
    /// Real native solves of `x^3 = 1` through each advertised path other than KLU.
    #[test]
    fn kinsol_dense_spgmr_picard_solve() {
        let solve = |function: Function, settings: Settings, start: f64| {
            let mut session = Session::new(
                function,
                settings,
                execution(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let report = session
                .solve(
                    &[start],
                    &Controls::default(),
                    &ResolvedAccuracy::verification(),
                    execution(),
                    &Tolerances {
                        variables: vec![1e-8],
                        rows: vec![1e-8],
                        integrality: 1e-8,
                    },
                    None,
                )
                .unwrap();
            assert_eq!(
                report.termination.category,
                Termination::Success,
                "{report:?}"
            );
            let x = report.candidate.as_ref().unwrap().primal[0];
            assert!((x - 1.0).abs() < 1e-7, "{x}");
            report
        };
        let count = |report: &SolveReport, key: &str| match report.metrics.get(key) {
            Some(Metric::Integer(v)) => *v,
            other => panic!("{key}: {other:?}"),
        };
        for strategy in [Strategy::Newton, Strategy::LineSearch] {
            // Dense: native dense factorization of the analytic Jacobian.
            let dense = solve(
                Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
                Settings {
                    method: Method {
                        strategy,
                        linear: Linear::Dense { limit: positive(4) },
                        ..Method::default()
                    },
                    ..settings()
                },
                2.0,
            );
            assert!(count(&dense, "KINGetNumJacEvals") > 0);
            assert_eq!(count(&dense, "KINGetNumJtimesEvals"), 0);
            // SPGMR: matrix-free Krylov iterations over the analytic JVP.
            let spgmr = solve(
                Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
                Settings {
                    method: Method {
                        strategy,
                        linear: Linear::Spgmr {
                            dimension: positive(4),
                        },
                        ..Method::default()
                    },
                    ..settings()
                },
                2.0,
            );
            assert!(count(&spgmr, "KINGetNumLinIters") > 0);
            assert!(count(&spgmr, "KINGetNumJtimesEvals") > 0);
        }
        // Picard: the declared constant splitting L = 3 (the derivative at the root) with
        // the original equations as the residual authority.
        let oracle = crate::solver_tests::Polynomial::new();
        let linear = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 3.0)],
        )
        .unwrap();
        let picard = solve(
            Function::Picard {
                oracle: Box::new(oracle),
                linear,
            },
            Settings {
                method: Method {
                    strategy: Strategy::Picard,
                    ..Method::default()
                },
                ..settings()
            },
            1.2,
        );
        assert!(count(&picard, "KINGetNumNonlinSolvIters") > 1);
    }
    #[test]
    fn picard_requires_explicit_splitting_and_forbids_sign_constraints() {
        let mut profile = settings();
        profile.method.strategy = Strategy::Picard;
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
            execution(),
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
    fn cube_root(method: Method, lower: f64, upper: f64, start: f64) -> (Session, SolveReport) {
        let mut oracle = crate::solver_tests::Polynomial::new();
        oracle.c.variables[0].lower = lower;
        oracle.c.variables[0].upper = upper;
        let mut session = Session::new(
            Function::Equations(Box::new(oracle)),
            Settings {
                method,
                ..settings()
            },
            execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        let report = session
            .solve(
                &[start],
                &Controls::default(),
                &ResolvedAccuracy::verification(),
                execution(),
                &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .unwrap();
        (session, report)
    }
    fn metric(report: &SolveReport, key: &str) -> i64 {
        match report.metrics.get(key) {
            Some(Metric::Integer(v)) => *v,
            other => panic!("{key}: {other:?}"),
        }
    }
    fn solved(report: &SolveReport) -> f64 {
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        let x = report.candidate.as_ref().unwrap().primal[0];
        assert!((x - 1.0).abs() < 1e-7, "{x}");
        x
    }
    #[test]
    fn directional_only_source_solves_without_claiming_an_assembled_jacobian() {
        #[derive(Debug)]
        struct Directional(crate::solver_tests::Polynomial);
        impl NleOracle for Directional {
            fn contract(&self) -> &OracleContract {
                &self.0.c
            }
            fn operations(&self) -> crate::RootOperations {
                crate::RootOperations {
                    jacobian: false,
                    jacobian_product: true,
                }
            }
            fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                NleOracle::residual(&mut self.0, x, out)
            }
            fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
                NleOracle::jacobian_pattern(&self.0)
            }
            fn jacobian(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
                panic!("directional-only source requested a full Jacobian")
            }
            fn jacobian_product(
                &mut self,
                x: &[f64],
                v: &[f64],
                out: &mut [f64],
            ) -> Result<(), ProblemError> {
                NleOracle::jacobian_product(&mut self.0, x, v, out)
            }
        }
        let source = || {
            let mut o = crate::solver_tests::Polynomial::new();
            o.c.derivatives = pse_kernels::DerivativeOrder::Value;
            Directional(o)
        };
        let settings = Settings {
            method: Method {
                linear: Linear::Spgmr {
                    dimension: positive(3),
                },
                ..Method::default()
            },
            ..settings()
        };
        assert!(
            settings
                .validate(&Function::Equations(Box::new(source())))
                .is_ok()
        );
        let assembled = Settings {
            method: Method::default(),
            ..settings.clone()
        };
        assert!(
            assembled
                .validate(&Function::Equations(Box::new(source())))
                .is_err()
        );
        for preconditioner in [Preconditioner::Jacobi, Preconditioner::BlockFactor] {
            let assembled = Settings {
                method: Method {
                    preconditioner,
                    ..settings.method
                },
                ..settings.clone()
            };
            assert!(
                assembled
                    .validate(&Function::Equations(Box::new(source())))
                    .is_err()
            );
        }
        let mut session = Session::new(
            Function::Equations(Box::new(source())),
            settings,
            execution(),
            crate::solver_tests::stamp(Backend::Kinsol),
        )
        .unwrap();
        let report = session
            .solve(
                &[2.0],
                &Controls::default(),
                &ResolvedAccuracy::verification(),
                execution(),
                &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .unwrap();
        solved(&report);
        assert!(metric(&report, "KINGetNumJtimesEvals") > 0);
    }
    /// L-D6: every matrix-free Krylov route solves `x^3 = 1` over the analytic JVP, with
    /// and without the right Jacobi preconditioner and under each forcing-term form; the
    /// Newton-step cap and Anderson settings are typed, validated and never leak on reuse.
    #[test]
    fn kinsol_krylov_variants_solve() {
        for linear in [
            Linear::Spgmr {
                dimension: positive(3),
            },
            Linear::Spfgmr {
                dimension: positive(3),
            },
            Linear::Spbcgs {
                dimension: positive(3),
            },
            Linear::Sptfqmr {
                dimension: positive(3),
            },
        ] {
            for preconditioner in [Preconditioner::None, Preconditioner::Jacobi] {
                for eta in [
                    Eta::Choice1,
                    Eta::Choice2 {
                        gamma: fraction(0.9),
                        alpha: 1.5,
                    },
                    Eta::Constant {
                        value: fraction(0.05),
                    },
                ] {
                    let method = Method {
                        linear,
                        preconditioner,
                        eta,
                        ..Method::default()
                    };
                    let (_, report) = cube_root(method, f64::NEG_INFINITY, f64::INFINITY, 2.0);
                    solved(&report);
                    assert!(metric(&report, "KINGetNumLinIters") > 0, "{linear:?}");
                    assert!(metric(&report, "KINGetNumJtimesEvals") > 0, "{linear:?}");
                    assert_eq!(
                        metric(&report, "KINGetNumPrecEvals") > 0,
                        preconditioner == Preconditioner::Jacobi,
                        "{linear:?} {preconditioner:?}"
                    );
                }
            }
        }
        // A Newton-step cap lengthens the solve; the reused session drops it again.
        let capped = Method {
            max_newton_step: Some(1.0),
            ..Method::default()
        };
        let (mut session, report) = cube_root(capped, f64::NEG_INFINITY, f64::INFINITY, 10.0);
        solved(&report);
        let slow = metric(&report, "KINGetNumNonlinSolvIters");
        let (_, free) = cube_root(Method::default(), f64::NEG_INFINITY, f64::INFINITY, 10.0);
        let fast = metric(&free, "KINGetNumNonlinSolvIters");

        assert!(slow > fast, "{slow} <= {fast}");
        session
            .replace(
                Function::Equations(Box::new(crate::solver_tests::Polynomial::new())),
                settings(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
        let again = session
            .solve(
                &[10.0],
                &Controls::default(),
                &ResolvedAccuracy::verification(),
                execution(),
                &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
            )
            .unwrap();
        solved(&again);
        assert_eq!(metric(&again, "KINGetNumNonlinSolvIters"), fast);
        // Anderson orthogonalization (fixed at allocation) and delay on the Picard route.
        for orthogonalization in [
            Orthogonalization::ModifiedGramSchmidt,
            Orthogonalization::InverseCompactWy,
            Orthogonalization::ClassicalGramSchmidt2,
            Orthogonalization::DelayedClassicalGramSchmidt2,
        ] {
            let oracle = crate::solver_tests::Polynomial::new();
            let linear = faer::sparse::SparseColMat::try_new_from_triplets(
                1,
                1,
                &[faer::sparse::Triplet::new(0, 0, 3.0)],
            )
            .unwrap();
            let method = Method {
                strategy: Strategy::Picard,
                anderson: 1,
                orthogonalization,
                anderson_delay: 1,
                ..Method::default()
            };
            let mut session = Session::new(
                Function::Picard {
                    oracle: Box::new(oracle),
                    linear,
                },
                Settings {
                    method,
                    ..settings()
                },
                execution(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let mut other = settings().method;
            other.strategy = Strategy::Picard;
            other.anderson = 1;
            assert_eq!(
                session.matches_settings(&Settings {
                    method: other,
                    ..settings()
                }),
                orthogonalization == Orthogonalization::ModifiedGramSchmidt
            );
            let report = session
                .solve(
                    &[1.2],
                    &Controls::default(),
                    &ResolvedAccuracy::verification(),
                    execution(),
                    &Tolerances {
                        variables: vec![1e-8],
                        rows: vec![1e-8],
                        integrality: 1e-8,
                    },
                    None,
                )
                .unwrap();
            solved(&report);
        }
        // Controls that would be ignored or are out of range are refused.
        let c = crate::solver_tests::Polynomial::new().c;
        for method in [
            Method {
                eta: Eta::Constant {
                    value: fraction(0.1),
                },
                ..Method::default()
            },
            Method {
                preconditioner: Preconditioner::Jacobi,
                linear: Linear::Dense { limit: positive(4) },
                ..Method::default()
            },
            Method {
                anderson_delay: 2,
                ..Method::default()
            },
            Method {
                orthogonalization: Orthogonalization::InverseCompactWy,
                ..Method::default()
            },
            Method {
                max_newton_step: Some(0.5),
                ..Method::default()
            },
            Method {
                linear: Linear::Sptfqmr {
                    dimension: positive(3),
                },
                eta: Eta::Choice2 {
                    gamma: fraction(0.9),
                    alpha: 2.5,
                },
                ..Method::default()
            },
        ] {
            let s = Settings {
                method,
                ..settings()
            };
            assert!(
                s.validate_contract(&c, Strategy::LineSearch, &Default::default())
                    .is_err(),
                "{method:?}"
            );
        }
        // A zero Krylov dimension or safeguard factor cannot be constructed or decoded.
        for method in [
            serde_json::json!({"linear": {"kind": "spbcgs", "dimension": 0}}),
            serde_json::json!({"eta": {"kind": "choice2", "gamma": 0.0, "alpha": 1.5}}),
        ] {
            assert!(serde_json::from_value::<Method>(method).is_err());
        }
    }
    /// Plan 22 Y6: a nonzero one-sided bound is a sign bound on shifted coordinates, so a
    /// root problem with one routes to KINSOL, which keeps every iterate inside it.
    #[test]
    fn one_sided_bounds_route_to_kinsol() {
        use crate::{
            execution::LINKED,
            routing::{Ineligible, Requirements, Route, oracle_facts},
        };
        let bounded = |lower: f64, upper: f64| {
            let mut c = crate::solver_tests::Polynomial::new().c;
            c.variables[0].lower = lower;
            c.variables[0].upper = upper;
            c
        };
        let route = |c: &OracleContract| {
            let facts = oracle_facts(c, false, true);
            let requirements = Requirements {
                context: crate::routing::test_context(&LINKED),
                table: &LINKED,
                facts: &facts,
                intent: SolveIntent::Root,
                numerical_psd: false,
                least_squares: false,
                controls: &Controls::default(),
                settings: &crate::execution::BackendSettings::Default,
                sensitivity: false,
            };
            let kinsol = requirements
                .eligibility()
                .into_iter()
                .find(|e| e.backend == Backend::Kinsol)
                .unwrap();
            (
                requirements.policy_select_for_test(SolverSelection::Auto),
                kinsol.reasons,
            )
        };
        for (lower, upper) in [(0.5, f64::INFINITY), (f64::NEG_INFINITY, 3.0)] {
            let (selected, reasons) = route(&bounded(lower, upper));
            assert!(reasons.is_empty(), "{reasons:?}");
            assert_eq!(selected.unwrap(), Route::Native(Backend::Kinsol));
        }
        let (_, reasons) = route(&bounded(0.5, 3.0));
        assert!(reasons.contains(&Ineligible::Bounds { signs: true }));
        // The shift is exact: the native iterate satisfies the original bound.
        let c = bounded(0.5, f64::INFINITY);
        assert_eq!(
            sign_constraints(&c).unwrap(),
            Constraints {
                signs: vec![1.0],
                offsets: vec![0.5]
            }
        );
        let (_, report) = cube_root(Method::default(), 0.5, f64::INFINITY, 2.0);
        solved(&report);
        let (_, report) = cube_root(Method::default(), f64::NEG_INFINITY, 3.0, 2.0);
        solved(&report);
        // A bound that excludes the root is never crossed.
        let (_, report) = cube_root(Method::default(), 1.5, f64::INFINITY, 2.0);
        assert_ne!(report.termination.category, Termination::Success);
        assert!(report.candidate.as_ref().unwrap().primal[0] >= 1.5);
        // Guards combine with shifted bounds: a bound inside the guarded half-line keeps
        // its shift, a weaker one yields to the guard, a facing bound is refused.
        let guard = |positive: bool| {
            std::collections::BTreeMap::from([(
                c.variables[0].id,
                pse_math::presolve::GuardSign {
                    positive,
                    strict: true,
                },
            )])
        };
        assert_eq!(
            guarded_sign_constraints(&c, &guard(true)).unwrap(),
            Constraints {
                signs: vec![1.0],
                offsets: vec![0.5]
            }
        );
        assert_eq!(
            guarded_sign_constraints(&bounded(-1.0, f64::INFINITY), &guard(true)).unwrap(),
            Constraints {
                signs: vec![2.0],
                offsets: vec![0.0]
            }
        );
        assert!(matches!(
            guarded_sign_constraints(&bounded(f64::NEG_INFINITY, 3.0), &guard(true)),
            Err(ProblemError::Unsupported(_))
        ));
        assert!(matches!(
            guarded_sign_constraints(&c, &guard(false)),
            Err(ProblemError::Contract(_))
        ));
    }
}
