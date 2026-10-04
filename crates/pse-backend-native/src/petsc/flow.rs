// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared artificial flow using actual TS velocity and the prepared mass derivative.
use super::derived as owner;
use super::*;
use pse_ids::ContentHash;
use pse_math::{
    derived::{DerivedFamily, FamilyClass, MassBinding, MassStructure},
    index::{Entry, GlobalCol, GlobalRow},
    sparse::AssemblyMatrix,
};
use std::sync::Arc;

/// Supplier explicitly refreshing a frozen mass at an accepted original anchor.
/// Rejected stages never call this owner.
pub trait FrozenMassRefresh: std::fmt::Debug {
    /// Immutable producer identity of this accepted-anchor refresh declaration.
    fn source(&self) -> ContentHash;
    /// Actual parameter/provider realization used by the next accepted-anchor refresh.
    fn realization(&self) -> ContentHash;
    /// Produce the exact declared sparse mass at an admitted accepted original point.
    fn at_anchor(&mut self, point: &[f64]) -> Result<AssemblyMatrix, ProblemError>;
}
/// Executable declared flow, independent of an ordinary PETSc root selection.
#[derive(Debug)]
pub struct ArtificialFlow<'a> {
    family: Arc<DerivedFamily>,
    original: &'a mut dyn NleOracle,
    mass: MassBinding<ProblemError>,
    domain: &'a mut dyn DomainOracle,
    refresh: Option<&'a mut dyn FrozenMassRefresh>,
    refresh_source: Option<ContentHash>,
    mass_realization: Option<ContentHash>,
}
impl<'a> ArtificialFlow<'a> {
    /// Bind the immutable original inventory, normalization, sign and mass authority.
    /// A frozen matrix without a refresh supplier is fixed for the entire trajectory.
    /// # Errors
    /// Original/guard identity, family or actual mass support differs from its declaration.
    pub fn new(
        family: Arc<DerivedFamily>,
        original: &'a mut dyn NleOracle,
        mass: MassBinding<ProblemError>,
        domain: &'a mut dyn DomainOracle,
    ) -> Result<Self, ProblemError> {
        if family.class() != FamilyClass::ShiftedPseudoTime
            || domain.source() != family.original().obligations().guards
        {
            return Err(ProblemError::Contract(
                "PETSc artificial flow family or original domain owner".into(),
            ));
        }
        owner::original(family.original(), original)?;
        validate_mass(&family, &mass)?;
        let mass_realization = match &mass {
            MassBinding::Frozen(_) => None,
            MassBinding::StateDependent(m) => Some(m.realization()),
        };
        Ok(Self {
            family,
            original,
            mass,
            domain,
            refresh: None,
            refresh_source: None,
            mass_realization,
        })
    }
    /// Declare accepted-anchor refresh; the supplied initial frozen matrix remains the
    /// initial-anchor binding. Refresh executes only after accepted-stage original admission.
    /// # Errors
    /// The mass is state dependent rather than frozen.
    pub fn with_frozen_refresh(
        mut self,
        refresh: &'a mut dyn FrozenMassRefresh,
    ) -> Result<Self, ProblemError> {
        if !matches!(self.mass, MassBinding::Frozen(_)) {
            return Err(ProblemError::Contract(
                "PETSc accepted-anchor refresh requires frozen mass".into(),
            ));
        }
        self.mass_realization = Some(refresh.realization());
        self.refresh_source = Some(refresh.source());
        self.refresh = Some(refresh);
        Ok(self)
    }
    /// Immutable mathematical authority consumed by this flow.
    pub fn family(&self) -> &Arc<DerivedFamily> {
        &self.family
    }
    /// Actual bound mass and declared refresh scope, distinct from immutable family support.
    pub fn binding_key(&self) -> ContentHash {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("petsc-artificial-trajectory")
            .hash(&self.family.key());
        match &self.mass {
            MassBinding::Frozen(matrix) => {
                h.str("frozen");
                for value in matrix.matrix().val() {
                    h.f64(*value);
                }
            }
            MassBinding::StateDependent(m) => {
                h.str("state-dependent").hash(&m.realization());
            }
        }
        h.bool(self.refresh_source.is_some());
        if let Some(source) = self.refresh_source {
            h.hash(&source);
        }
        if let Some(realization) = self.mass_realization {
            h.hash(&realization);
        }
        h.finish_hash()
    }
    fn mass_apply(
        &mut self,
        x: &[f64],
        velocity: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        match &mut self.mass {
            MassBinding::Frozen(matrix) => matrix.product(velocity, out)?,
            MassBinding::StateDependent(m) => {
                if Some(m.realization()) != self.mass_realization
                    || Some(m.structure()) != self.family.mass_structure()
                {
                    return Err(ProblemError::Contract(
                        "PETSc mass supplier changed its declared binding during a trajectory"
                            .into(),
                    ));
                }
                m.apply(x, velocity, out)?;
                if Some(m.realization()) != self.mass_realization
                    || Some(m.structure()) != self.family.mass_structure()
                {
                    return Err(ProblemError::Contract(
                        "PETSc mass binding changed during its actual action".into(),
                    ));
                }
            }
        }
        finite(out)
    }
    fn residual(&mut self, x: &[f64], velocity: &[f64]) -> Result<Vec<f64>, ProblemError> {
        self.domain.check(x)?;
        let mut values = vec![0.0; x.len()];
        self.original.residual(x, &mut values)?;
        let mut mass = vec![0.0; x.len()];
        self.mass_apply(x, velocity, &mut mass)?;
        let sign = self
            .family
            .pseudo_sign()
            .ok_or_else(|| ProblemError::Contract("PETSc flow sign".into()))?;
        for (v, m) in values.iter_mut().zip(mass) {
            *v = sign * *v + m;
        }
        self.domain.check(x)?;
        finite(&values)?;
        Ok(values)
    }
    fn jacobian(
        &mut self,
        x: &[f64],
        velocity: &[f64],
        shift: f64,
        execution: &Execution,
    ) -> Result<Vec<f64>, ProblemError> {
        if !shift.is_finite() || shift <= 0.0 {
            return Err(ProblemError::Contract("PETSc actual TS shift".into()));
        }
        self.domain.check(x)?;
        let family = self.family.clone();
        let entries = family.incidence();
        let n = x.len();
        let sign = family
            .pseudo_sign()
            .ok_or_else(|| ProblemError::Contract("PETSc flow sign".into()))?;
        let crate::nlp_pattern::Pattern { rows, columns } =
            crate::nlp_pattern::Pattern::new(self.original.jacobian_pattern(), false)?;
        let mut original = vec![0.0; rows.len()];
        self.original.jacobian(x, &mut original)?;
        finite(&original)?;
        let mut output = vec![0.0; entries.len()];
        for ((&row, &col), value) in rows.iter().zip(&columns).zip(original) {
            let slot = entries
                .binary_search(&Entry::new(
                    GlobalRow::new(row as usize),
                    GlobalCol::new(col as usize),
                ))
                .map_err(|_| {
                    ProblemError::Contract(
                        "PETSc original Jacobian outside prepared flow support".into(),
                    )
                })?;
            output[slot] += sign * value;
        }
        let (mass_edges, derivative_edges) = match family
            .mass_structure()
            .ok_or_else(|| ProblemError::Contract("PETSc mass authority".into()))?
        {
            MassStructure::Frozen { incidence } => (incidence.as_slice(), &[][..]),
            MassStructure::StateDependent {
                incidence,
                derivative_incidence,
                ..
            } => (incidence.as_slice(), derivative_incidence.as_slice()),
        };
        let mut basis = vec![0.0; n];
        let mut mass = vec![0.0; n];
        let mut derivative = vec![0.0; n];
        for col in 0..n {
            execution.check()?;
            basis[col] = 1.0;
            mass.fill(0.0);
            derivative.fill(0.0);
            if mass_edges.iter().any(|e| e.col.get() == col) {
                self.mass_apply(x, &basis, &mut mass)?;
            }
            if derivative_edges.iter().any(|e| e.col.get() == col) {
                let MassBinding::StateDependent(m) = &mut self.mass else {
                    return Err(ProblemError::Contract(
                        "PETSc declared DM has no actual supplier".into(),
                    ));
                };
                m.derivative_action(x, &basis, velocity, &mut derivative)?;
                finite(&derivative)?;
                if Some(m.realization()) != self.mass_realization
                    || Some(m.structure()) != family.mass_structure()
                {
                    return Err(ProblemError::Contract(
                        "PETSc mass binding changed during DM action".into(),
                    ));
                }
            }
            for row in 0..n {
                let edge = Entry::new(GlobalRow::new(row), GlobalCol::new(col));
                if mass[row] != 0.0 && mass_edges.binary_search(&edge).is_err()
                    || derivative[row] != 0.0 && derivative_edges.binary_search(&edge).is_err()
                {
                    return Err(ProblemError::Contract(
                        "PETSc mass/DM action outside its prepared support".into(),
                    ));
                }
                if let Ok(slot) = entries.binary_search(&edge) {
                    output[slot] += shift * mass[row] + derivative[row];
                }
            }
            basis[col] = 0.0;
        }
        self.domain.check(x)?;
        finite(&output)?;
        Ok(output)
    }
}
fn finite(values: &[f64]) -> Result<(), ProblemError> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(ProblemError::numerical(
            "nonfinite PETSc artificial-flow action",
        ))
    } else {
        Ok(())
    }
}
fn matrix_edges(matrix: &AssemblyMatrix) -> Vec<Entry<GlobalRow, GlobalCol>> {
    let mut edges = Vec::new();
    for col in 0..matrix.matrix().ncols() {
        for row in matrix.matrix().symbolic().row_idx_of_col(col) {
            edges.push(Entry::new(GlobalRow::new(row), GlobalCol::new(col)));
        }
    }
    edges.sort_unstable();
    edges
}
fn validate_mass(
    family: &DerivedFamily,
    mass: &MassBinding<ProblemError>,
) -> Result<(), ProblemError> {
    let n = family.original().coordinates().len();
    match (mass, family.mass_structure()) {
        (MassBinding::Frozen(matrix), Some(MassStructure::Frozen { incidence }))
            if matrix.matrix().nrows() == n
                && matrix.matrix().ncols() == n
                && matrix_edges(matrix) == *incidence =>
        {
            finite(matrix.matrix().val())
        }
        (MassBinding::StateDependent(m), Some(expected @ MassStructure::StateDependent { .. }))
            if m.structure() == expected =>
        {
            Ok(())
        }
        _ => Err(ProblemError::Contract(
            "PETSc actual mass differs from prepared family support/semantics".into(),
        )),
    }
}
/// Finite outer artificial trajectory caps, distinct from each inner SNES iteration cap.
#[derive(Clone, Copy, Debug)]
pub struct FlowLimits {
    /// Maximum accepted artificial steps (positive and PetscInt32 representable).
    pub steps: u32,
    /// Positive finite native outer stop time. The last accepted step may step over it.
    pub artificial_time: f64,
}
struct Context<'a, 'b> {
    flow: &'a mut ArtificialFlow<'b>,
    state: CallbackState,
    n: usize,
    rows: Vec<i32>,
    columns: Vec<i32>,
    x: ffi::Handle,
    minimum: f64,
    steady_tolerances: Vec<f64>,
    steady: bool,
    accepted: usize,
}
unsafe extern "C" fn function(
    ctx: *mut c_void,
    _time: f64,
    x: ffi::Handle,
    velocity: ffi::Handle,
    f: ffi::Handle,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Context<'_, '_>>() };
    let n = c.n;
    let output = c.state.evaluate("flow.residual", || {
        c.flow.residual(&read(x, n)?, &read(velocity, n)?)
    });
    let Some(output) = output else {
        return owner::refused(&c.state);
    };
    owner::publish(&mut c.state, || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_write(f, index(n)?, output.as_ptr()) },
            "flow residual publication",
        )
    })
}
unsafe extern "C" fn jacobian(
    ctx: *mut c_void,
    _time: f64,
    x: ffi::Handle,
    velocity: ffi::Handle,
    shift: f64,
    a: ffi::Handle,
    p: ffi::Handle,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Context<'_, '_>>() };
    let n = c.n;
    let execution = c.state.execution.clone();
    let output = c.state.evaluate("flow.jacobian", || {
        c.flow
            .jacobian(&read(x, n)?, &read(velocity, n)?, shift, &execution)
    });
    let Some(output) = output else {
        return owner::refused(&c.state);
    };
    let rows = c.rows.as_ptr();
    let columns = c.columns.as_ptr();
    owner::publish(&mut c.state, || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_mat_write(a, index(output.len())?, rows, columns, output.as_ptr())
            },
            "flow Jacobian publication",
        )?;
        if a != p {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe {
                    ffi::pse_petsc_mat_write(
                        p,
                        index(output.len())?,
                        rows,
                        columns,
                        output.as_ptr(),
                    )
                },
                "flow PC publication",
            )?;
        }
        Ok(())
    })
}
unsafe extern "C" fn domain(ctx: *mut c_void, _time: f64, x: ffi::Handle, accept: *mut i32) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Context<'_, '_>>() };
    let n = c.n;
    let accepted = c
        .state
        .evaluate("flow.candidate_domain", || {
            c.flow.domain.check(&read(x, n)?)
        })
        .is_some();
    // SAFETY: The C trampoline supplies a non-null aligned writable scalar for this callback result.
    unsafe { *accept = i32::from(accepted) };
    if c.state.terminal.is_some() {
        ffi::EVAL_TERMINAL
    } else {
        ffi::EVAL_OK
    }
}
unsafe extern "C" fn pre_stage(ctx: *mut c_void, ts: ffi::Handle, time: f64) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Context<'_, '_>>() };
    let minimum = c.minimum;
    let admitted=c.state.evaluate("flow.stage",||{
        let (_,_,_,_,_,_,dt)=statistics(ts)?;
        if !time.is_finite() || !dt.is_finite() || dt<minimum {return Err(ProblemError::Limit{kind:crate::LimitKind::Work,detail:"PETSc artificial stage time/radius exhausted its finite declaration before another SNES dispatch".into()});}
        Ok(())
    });
    if admitted.is_some() {
        ffi::EVAL_OK
    } else {
        if matches!(
            c.state.last_failure,
            Some(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                ..
            })
        ) {
            c.state.terminal = Some((
                Termination::Limit,
                "artificial step minimum exhausted".into(),
            ));
        }
        ffi::EVAL_TERMINAL
    }
}
unsafe extern "C" fn post_step(ctx: *mut c_void, ts: ffi::Handle) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Context<'_, '_>>() };
    let n = c.n;
    let accepted = c.state.evaluate("flow.accepted_original", || {
        let point = read(c.x, n)?;
        c.flow.domain.check(&point)?;
        let mut values = vec![0.0; n];
        c.flow.original.residual(&point, &mut values)?;
        finite(&values)?;
        c.flow.domain.check(&point)?;
        let steady = values
            .iter()
            .zip(&c.steady_tolerances)
            .all(|(v, t)| v.abs() <= *t);
        if let Some(refresh) = c.flow.refresh.as_mut() {
            if Some(refresh.source()) != c.flow.refresh_source {
                return Err(ProblemError::Contract(
                    "PETSc accepted-anchor mass refresh changed its declared producer".into(),
                ));
            }
            let matrix = refresh.at_anchor(&point)?;
            let mass = MassBinding::Frozen(matrix);
            validate_mass(&c.flow.family, &mass)?;
            c.flow.mass_realization = Some(refresh.realization());
            c.flow.mass = mass;
        }
        Ok(steady)
    });
    let Some(steady) = accepted else {
        owner::terminal(
            &mut c.state,
            "accepted artificial stage failed original admission or refresh",
        );
        return ffi::EVAL_TERMINAL;
    };
    c.accepted += 1;
    c.steady = steady;
    if steady {
        owner::publish(&mut c.state, || {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe { ffi::pse_petsc_ts_reason(ts, 3) },
                "original steady termination",
            )
        })
    } else {
        ffi::EVAL_OK
    }
}
type FlowStatistics = (i32, i32, i32, i32, i32, f64, f64);
fn statistics(ts: ffi::Handle) -> Result<FlowStatistics, ProblemError> {
    let (mut reason, mut steps, mut iterations, mut rejects, mut failures, mut time, mut dt) =
        (0, 0, 0, 0, 0, 0.0, 0.0);
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_ts_statistics(
                    ts,
                    &mut reason,
                    &mut steps,
                    &mut iterations,
                    &mut rejects,
                    &mut failures,
                    &mut time,
                    &mut dt,
                )
            },
            "TS statistics",
        )?;
    }
    Ok((reason, steps, iterations, rejects, failures, time, dt))
}
#[cfg(test)]
pub(super) fn assert_actual_mass_action(flow: &mut ArtificialFlow<'_>, execution: &Execution) {
    assert_eq!(flow.residual(&[2.0], &[3.0]).unwrap(), vec![16.0]);
    // DR=1, M=5, DM[1]*velocity=12, and the actual TS shift is four.
    assert_eq!(
        flow.jacobian(&[2.0], &[3.0], 4.0, execution).unwrap(),
        vec![33.0]
    );
}
/// Execute an explicitly declared artificial trajectory and assess the original root.
/// # Errors
/// Missing finite foreign reservation, inconsistent declarations, native setup or teardown.
pub fn solve_flow(
    flow: &mut ArtificialFlow<'_>,
    initial: &[f64],
    settings: &Settings,
    limits: FlowLimits,
    request: SolveRequest<'_>,
) -> Result<SolveReport, ProblemError> {
    let n = flow.family.original().coordinates().len();
    owner::original(flow.family.original(), flow.original)?;
    owner::controls(&request, settings, Method::PseudoTransient, n)?;
    let SolveRequest {
        controls,
        accuracy,
        execution,
        tolerances,
        warm,
        compatibility,
    } = request;
    if limits.steps == 0
        || limits.steps > i32::MAX as u32
        || !limits.artificial_time.is_finite()
        || limits.artificial_time <= 0.0
    {
        return Err(ProblemError::Contract(
            "PETSc finite artificial trajectory caps".into(),
        ));
    }
    let pseudo = settings
        .pseudo
        .ok_or_else(|| ProblemError::Contract("PETSc pseudo controls".into()))?;
    if u64::from(limits.steps)
        .checked_mul(u64::from(pseudo.rejections.max(1)))
        .and_then(|v| v.checked_mul(u64::from(controls.iterations)))
        .is_none_or(|v| v > i32::MAX as u64)
    {
        return Err(ProblemError::Contract(
            "PETSc aggregate TS iteration/rejection counters must fit PetscInt32".into(),
        ));
    }
    let start = owner::start(initial, warm, compatibility, n)?;
    let initial_binding = flow.binding_key();
    let incidence = flow.family.incidence();
    let mass_bytes = match &flow.mass {
        MassBinding::Frozen(m) => m.retained_bytes(),
        MassBinding::StateDependent(_) => 0,
    };
    let planned = owner::shape(
        &execution,
        n.checked_mul(12)
            .ok_or_else(|| ProblemError::memory("PETSc flow vector shape"))?,
        incidence.len(),
        n,
        settings,
        n,
    )?
    .checked_add(mass_bytes)
    .ok_or_else(|| ProblemError::memory("PETSc mass shape overflow"))?;
    if planned > execution.memory.unwrap_or(0) {
        return Err(ProblemError::memory(
            "PETSc mass and trajectory shape exceed foreign reservation",
        ));
    }
    let rows = incidence
        .iter()
        .map(|e| index(e.row.get()))
        .collect::<Result<Vec<_>, _>>()?;
    let columns = incidence
        .iter()
        .map(|e| index(e.col.get()))
        .collect::<Result<Vec<_>, _>>()?;
    let mut context = Box::new(Context {
        flow,
        state: CallbackState::new(execution.clone()),
        n,
        rows,
        columns,
        x: ptr::null_mut(),
        minimum: pseudo.minimum,
        steady_tolerances: tolerances.rows.clone(),
        steady: false,
        accepted: 0,
    });
    let admission = Admission::enter(&execution)?;
    let _threads = crate::mkl::Threads::enter(1)?;
    let admitted = context.state.evaluate("flow.initial_original", || {
        context.flow.domain.check(start)?;
        let mut values = vec![0.0; n];
        context.flow.original.residual(start, &mut values)?;
        finite(&values)?;
        context.flow.domain.check(start)?;
        Ok(())
    });
    if admitted.is_none() {
        owner::terminal(&mut context.state, "invalid initial artificial-flow domain");
        context.state.trial_rejections = 0;
        let mut report = SolveReport::new(
            Backend::Petsc,
            context.flow.original.contract(),
            NativeTermination {
                code: 0,
                name: "PETSC_FLOW_INITIAL_DOMAIN_REFUSED".into(),
                message: None,
                category: Termination::Evaluation,
                assurance: Assurance::None,
            },
            &execution,
        );
        context.state.finish(&mut report);
        return Ok(report);
    }
    let mut objects = Objects::new(&admission);
    let options = objects.create(
        ffi::pse_petsc_options_create,
        ffi::pse_petsc_options_destroy,
    )?;
    let x = objects.vector(start)?;
    context.x = x;
    let f = objects.vector(&vec![0.0; n])?;
    let matrix = owner::matrix(&mut objects, n, &context.rows)?;
    let ts = objects.create(ffi::pse_petsc_ts_create, ffi::pse_petsc_ts_destroy)?;
    let mut snes = ptr::null_mut();
    let ctx = ptr::from_mut(context.as_mut()).cast();
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_options_set(options, c"-ts_pseudo_fatol".as_ptr(), c"0".as_ptr())
            },
            "disable surrogate absolute stop",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_options_set(options, c"-ts_pseudo_frtol".as_ptr(), c"0".as_ptr())
            },
            "disable surrogate relative stop",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_pseudo(ts) },
            "declared pseudo method",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_options(ts, options) },
            "TS private options",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_from_options(ts) },
            "TS private controls",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_snes(ts, &mut snes) },
            "borrowed TS SNES",
        )?;
        owner::configure(snes, settings, accuracy, controls.iterations)?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_layout(snes, x) },
            "TS original layout",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_ifunction(ts, f, function, ctx) },
            "actual flow function",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_ijacobian(ts, matrix, matrix, jacobian, ctx) },
            "actual flow Jacobian",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_domain(ts, domain, ctx) },
            "original candidate domain",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_pre_stage(ts, pre_stage, ctx) },
            "bounded stage dispatch",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_post_step(ts, post_step, ctx) },
            "original accepted-stage stop",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_solution(ts, x) },
            "TS original state",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_ts_time(
                    ts,
                    0.0,
                    pseudo.initial,
                    limits.artificial_time,
                    index(limits.steps as usize)?,
                )
            },
            "finite artificial limits",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_ts_failures(
                    ts,
                    index(pseudo.nonlinear_failures as usize)?,
                    index(pseudo.rejections as usize)?,
                )
            },
            "finite TS failure caps",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_ts_adapt(ts, pseudo.minimum, pseudo.maximum, pseudo.failed_scale)
            },
            "TS reject scale",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_pseudo_growth(ts, pseudo.growth, pseudo.maximum) },
            "TS growth cap",
        )?;
    }
    let code = {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_ts_solve(ts, x) }
    };
    let (reason, steps, iterations, rejections, failures, time, dt) = statistics(ts)?;
    let mut name = ptr::null();
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_ts_reason_name(ts, &mut name) },
            "TS reason",
        )?;
    }
    let name = if name.is_null() {
        "TS_UNKNOWN_REASON".into()
    } else {
        // SAFETY: The pointer was checked non-null; PETSc supplies a live null-terminated library string for this query.
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    };
    let category = if reason == 3 && context.steady {
        Termination::Success
    } else if matches!(reason, 1 | 2 | -1 | -2) {
        Termination::Limit
    } else {
        Termination::Inconclusive
    };
    let mut report = SolveReport::new(
        Backend::Petsc,
        context.flow.original.contract(),
        NativeTermination {
            code: reason.into(),
            name,
            message: None,
            category,
            assurance: Assurance::None,
        },
        &execution,
    );
    let native_terminal =
        finish_native_error(&mut report, &mut context.state, code, "artificial solve");
    report.evidence.start_submitted = warm.is_some();
    report.evidence.work.iterations = u64::try_from(iterations).ok();
    report.metrics.extend(std::collections::BTreeMap::from([
        ("TSGetStepNumber".into(), Metric::Integer(steps.into())),
        (
            "TSGetStepRejections".into(),
            Metric::Integer(rejections.into()),
        ),
        ("TSGetSNESFailures".into(), Metric::Integer(failures.into())),
        ("TSGetTime".into(), Metric::Real(time)),
        ("TSGetTimeStep".into(), Metric::Real(dt)),
        (
            "planned.compact_bytes".into(),
            Metric::Integer(i64::try_from(planned).unwrap_or(i64::MAX)),
        ),
    ]));
    report
        .provenance
        .insert("native".into(), format!("PETSc {}", ffi::VERSION));
    report.provenance.insert("build".into(), build().to_hex());
    report
        .provenance
        .insert("family".into(), context.flow.family.key().to_hex());
    report.provenance.insert(
        "normalization".into(),
        context.flow.family.original().normalization().to_hex(),
    );
    report
        .provenance
        .insert("guards".into(), context.flow.domain.source().to_hex());
    report.provenance.insert(
        "mass.scope".into(),
        if context.flow.refresh.is_some() {
            "accepted-anchor"
        } else if matches!(context.flow.mass, MassBinding::Frozen(_)) {
            "fixed-trajectory"
        } else {
            "state-dependent"
        }
        .into(),
    );
    report
        .provenance
        .insert("mass.initial_binding".into(), initial_binding.to_hex());
    report.provenance.insert(
        "mass.final_binding".into(),
        context.flow.binding_key().to_hex(),
    );
    if let Some(refresh) = &context.flow.refresh {
        report
            .provenance
            .insert("mass.refresh_source".into(), refresh.source().to_hex());
    }
    if let Some(realization) = context.flow.mass_realization {
        report
            .provenance
            .insert("mass.realization".into(), realization.to_hex());
    }
    owner::effective(&mut report, settings, accuracy, controls.iterations, "")?;
    for (key, value) in [
        ("ts_type", OptionValue::Text("pseudo".into())),
        ("ts_pseudo_fatol", OptionValue::Real(0.0)),
        ("ts_pseudo_frtol", OptionValue::Real(0.0)),
        ("ts_dt", OptionValue::Real(pseudo.initial)),
        ("ts_max_time", OptionValue::Real(limits.artificial_time)),
        (
            "ts_max_steps",
            OptionValue::Integer(index(limits.steps as usize)?),
        ),
        ("ts_adapt_dt_min", OptionValue::Real(pseudo.minimum)),
        ("ts_adapt_dt_max", OptionValue::Real(pseudo.maximum)),
        (
            "ts_adapt_scale_solve_failed",
            OptionValue::Real(pseudo.failed_scale),
        ),
        ("ts_pseudo_increment", OptionValue::Real(pseudo.growth)),
        (
            "ts_max_snes_failures",
            OptionValue::Integer(index(pseudo.nonlinear_failures as usize)?),
        ),
        (
            "ts_max_reject",
            OptionValue::Integer(index(pseudo.rejections as usize)?),
        ),
    ] {
        report.options.insert(key.into(), value);
    }
    match read(x, n) {
        Ok(point) if native_terminal => retain_native_candidate(&mut report, point),
        Ok(point) => owner::assess(
            &mut report,
            context.flow.original,
            context.flow.domain,
            &mut context.state,
            point,
            tolerances,
            compatibility,
        ),
        Err(error) => {
            context.state.finish(&mut report);
            if !native_terminal {
                report.record_validation_failure(error);
            }
        }
    }
    objects.close()?;
    Ok(report)
}
