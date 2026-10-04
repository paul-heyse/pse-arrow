// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admission and original assessment shared by the explicitly declared native products.
use super::*;
use pse_ids::ContentHash;
use pse_math::derived::OriginalContract;

/// Executable original guard/domain owner. Derivative support grants no domain permission.
pub trait DomainOracle: std::fmt::Debug {
    /// Identity of the consumed original guard/applicability obligation.
    fn source(&self) -> ContentHash;
    /// Execute the original checks at this original-coordinate point.
    fn check(&mut self, point: &[f64]) -> Result<(), ProblemError>;
}
/// Declared admission provenance for a domain-safe block supplier.
/// It is checked at each trial and is not a proof that arbitrary trials are valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainWitness {
    /// Original guard obligation served by the declaration and executable supplier.
    pub source: ContentHash,
    /// Actual admitted declaration identity, retained in result provenance.
    pub declaration: ContentHash,
}
pub(super) fn original(
    contract: &OriginalContract,
    oracle: &dyn NleOracle,
) -> Result<(), ProblemError> {
    oracle.operations().admit(oracle.contract(), false)?;
    let native = oracle.contract();
    if native.identity != contract.identity()
        || native.variables.len() != contract.coordinates().len()
        || native.rows.len() != contract.constraints().len()
        || native
            .variables
            .iter()
            .zip(contract.coordinates())
            .any(|(a, b)| a.id != b.id || a.lower != b.lower || a.upper != b.upper)
        || native
            .rows
            .iter()
            .zip(contract.constraints())
            .any(|(a, b)| *a != b.id || b.lower != 0.0 || b.upper != 0.0)
        || contract.obligations().objective.is_some()
    {
        return Err(ProblemError::Contract("PETSc derived product must retain the complete original square root inventory and interpretation".into()));
    }
    if native
        .variables
        .iter()
        .any(|v| v.lower.is_finite() || v.upper.is_finite())
    {
        return Err(ProblemError::Unsupported(
            "PETSc derived profiles do not enforce arbitrary original bounds".into(),
        ));
    }
    let pattern = oracle.jacobian_pattern();
    if pattern.nrows() != native.rows.len() || pattern.ncols() != native.variables.len() {
        return Err(ProblemError::Contract(
            "PETSc original Jacobian shape".into(),
        ));
    }
    let mut edges = Vec::new();
    for col in 0..pattern.ncols() {
        for row in pattern.row_idx_of_col(col) {
            edges.push(pse_math::index::Entry::new(
                pse_math::index::GlobalRow::new(row),
                pse_math::index::GlobalCol::new(col),
            ));
        }
    }
    edges.sort_unstable();
    if edges != contract.incidence() {
        return Err(ProblemError::Contract(
            "PETSc original supplier differs from prepared all-branch incidence".into(),
        ));
    }
    Ok(())
}
pub(super) fn controls(
    request: &SolveRequest<'_>,
    settings: &Settings,
    expected: Method,
    n: usize,
) -> Result<(), ProblemError> {
    let SolveRequest {
        controls,
        accuracy,
        execution,
        tolerances,
        compatibility,
        ..
    } = request;
    controls.validate()?;
    accuracy.validate()?;
    settings.validate()?;
    tolerances.validate(n, n)?;
    if settings.method != expected || compatibility.backend != Backend::Petsc {
        return Err(ProblemError::Contract(
            "PETSc product/method compatibility".into(),
        ));
    }
    if controls.threads != 1
        || !controls.options.is_empty()
        || accuracy.native_scaling
        || accuracy.acceptable.is_some()
        || controls.reuse == ReusePolicy::RequireReuse
    {
        return Err(ProblemError::Unsupported("PETSc derived profiles require serial typed controls, strict original scaling and fresh ownership".into()));
    }
    if execution.memory.is_none() {
        return Err(ProblemError::Unsupported(
            "PETSc derived allocation requires a resolved finite foreign reservation".into(),
        ));
    }
    index(n)?;
    index(controls.iterations as usize)?;
    Ok(())
}
pub(super) fn start<'a>(
    initial: &'a [f64],
    warm: Option<&'a WarmStart>,
    compatibility: &Compatibility,
    n: usize,
) -> Result<&'a [f64], ProblemError> {
    let values = if let Some(warm) = warm {
        warm.validate(compatibility)?;
        let WarmPayload::Root(values) = &warm.payload else {
            return Err(ProblemError::Contract(
                "PETSc original primal warm payload".into(),
            ));
        };
        values.as_slice()
    } else {
        initial
    };
    if values.len() != n || values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract(
            "PETSc original finite initial coordinates".into(),
        ));
    }
    Ok(values)
}
/// Checked known payload/workspace shape, before foreign creation. PETSc has no hard
/// per-instance memory limiter; the complete foreign reservation remains owned upstream.
pub(super) fn shape(
    execution: &Execution,
    vector_elements: usize,
    entries: usize,
    indices: usize,
    settings: &Settings,
    n: usize,
) -> Result<usize, ProblemError> {
    let overflow = || ProblemError::memory("PETSc compact allocation shape overflow");
    // Default GMRES restart is 30 in the tagged source; flexible GMRES also stores
    // preconditioned directions. Direct LU can require dense fill even for sparse input.
    let krylov = match settings.linear {
        Linear::Gmres => 35,
        Linear::Fgmres => 66,
        Linear::Bicgstab => 12,
        Linear::Preonly => 2,
    };
    let work = n.checked_mul(krylov).ok_or_else(overflow)?;
    let fill = if settings.preconditioner == Preconditioner::Lu {
        n.checked_mul(n).ok_or_else(overflow)?
    } else {
        entries
    };
    let values = vector_elements
        .checked_add(work)
        .and_then(|v| v.checked_add(fill))
        .and_then(|v| v.checked_add(entries))
        .ok_or_else(overflow)?;
    let bytes = values
        .checked_mul(size_of::<f64>())
        .and_then(|v| {
            indices
                .checked_add(entries.checked_mul(2)?)
                .and_then(|i| i.checked_mul(size_of::<usize>()))
                .and_then(|i| v.checked_add(i))
        })
        .ok_or_else(overflow)?;
    let allowance = execution
        .memory
        .ok_or_else(|| ProblemError::Unsupported("PETSc finite foreign reservation".into()))?;
    if bytes > allowance {
        return Err(ProblemError::memory(
            "PETSc planned compact maps, sparse mass and native workspace exceed the admitted foreign reservation",
        ));
    }
    Ok(bytes)
}
pub(super) fn matrix(
    objects: &mut Objects<'_>,
    n: usize,
    rows: &[i32],
) -> Result<ffi::Handle, ProblemError> {
    let mut capacity = vec![0_i32; n];
    for &row in rows {
        let r =
            usize::try_from(row).map_err(|_| ProblemError::Contract("PETSc matrix row".into()))?;
        let value = capacity
            .get_mut(r)
            .ok_or_else(|| ProblemError::Contract("PETSc matrix row extent".into()))?;
        *value = value
            .checked_add(1)
            .ok_or_else(|| ProblemError::memory("PETSc sparse capacity overflow"))?;
    }
    let mut handle = ptr::null_mut();
    let code = {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_mat_create(index(n)?, index(n)?, capacity.as_ptr(), &mut handle) }
    };
    if !handle.is_null() {
        objects.entries.push((handle, ffi::pse_petsc_mat_destroy));
    }
    check(code, "derived matrix creation")?;
    if handle.is_null() {
        return Err(ProblemError::internal(
            "PETSc successful matrix creation returned null",
        ));
    }
    Ok(handle)
}
pub(super) fn configure(
    snes: ffi::Handle,
    settings: &Settings,
    accuracy: &ResolvedAccuracy,
    iterations: u32,
) -> Result<(), ProblemError> {
    let ksp = match settings.linear {
        Linear::Gmres => c"gmres",
        Linear::Fgmres => c"fgmres",
        Linear::Bicgstab => c"bcgs",
        Linear::Preonly => c"preonly",
    };
    let pc = match settings.preconditioner {
        Preconditioner::None => c"none",
        Preconditioner::Jacobi => c"jacobi",
        Preconditioner::Ilu => c"ilu",
        Preconditioner::Lu => c"lu",
    };
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_type(snes, c"newtontr".as_ptr()) },
            "derived inner method",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_tolerances(
                    snes,
                    accuracy.feasibility,
                    0.0,
                    0.0,
                    index(iterations as usize)?,
                    i32::MAX,
                )
            },
            "derived inner tolerances",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_tr_tolerances(
                    snes,
                    settings.trust.minimum,
                    settings.trust.maximum,
                    settings.trust.initial,
                )
            },
            "derived trust controls",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_linear(snes, ksp.as_ptr(), pc.as_ptr()) },
            "derived linear controls",
        )?;
    }
    Ok(())
}
pub(super) fn effective(
    report: &mut SolveReport,
    settings: &Settings,
    accuracy: &ResolvedAccuracy,
    iterations: u32,
    prefix: &str,
) -> Result<(), ProblemError> {
    let ksp = match settings.linear {
        Linear::Gmres => "gmres",
        Linear::Fgmres => "fgmres",
        Linear::Bicgstab => "bcgs",
        Linear::Preonly => "preonly",
    };
    let pc = match settings.preconditioner {
        Preconditioner::None => "none",
        Preconditioner::Jacobi => "jacobi",
        Preconditioner::Ilu => "ilu",
        Preconditioner::Lu => "lu",
    };
    for (key, value) in [
        ("snes_type", OptionValue::Text("newtontr".into())),
        ("snes_atol", OptionValue::Real(accuracy.feasibility)),
        ("snes_rtol", OptionValue::Real(0.0)),
        ("snes_stol", OptionValue::Real(0.0)),
        (
            "snes_max_it",
            OptionValue::Integer(index(iterations as usize)?),
        ),
        ("snes_max_funcs", OptionValue::Integer(i32::MAX)),
        (
            "snes_tr_deltamin",
            OptionValue::Real(settings.trust.minimum),
        ),
        (
            "snes_tr_deltamax",
            OptionValue::Real(settings.trust.maximum),
        ),
        ("snes_tr_delta0", OptionValue::Real(settings.trust.initial)),
        ("ksp_type", OptionValue::Text(ksp.into())),
        ("pc_type", OptionValue::Text(pc.into())),
    ] {
        report.options.insert(format!("{prefix}{key}"), value);
    }
    report.provenance.insert(
        "settings".into(),
        serde_json::to_string(settings).map_err(|e| ProblemError::internal(e.to_string()))?,
    );
    Ok(())
}
pub(super) fn publish(
    state: &mut CallbackState,
    operation: impl FnOnce() -> Result<(), ProblemError>,
) -> i32 {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => ffi::EVAL_OK,
        Ok(Err(error)) => {
            state.terminal = Some((Termination::Evaluation, error.to_string()));
            state.last_failure = Some(error);
            ffi::EVAL_TERMINAL
        }
        Err(_) => {
            state.terminal = Some((
                Termination::Panic,
                "panic publishing PETSc derived callback".into(),
            ));
            ffi::EVAL_TERMINAL
        }
    }
}
pub(super) fn terminal(state: &mut CallbackState, message: &str) {
    if state.terminal.is_none() {
        state.terminal = Some((Termination::Evaluation, message.into()));
    }
}
pub(super) fn refused(state: &CallbackState) -> i32 {
    if state.terminal.is_some() {
        ffi::EVAL_TERMINAL
    } else {
        ffi::EVAL_DOMAIN
    }
}
pub(super) fn assess(
    report: &mut SolveReport,
    oracle: &mut dyn NleOracle,
    domain: &mut dyn DomainOracle,
    state: &mut CallbackState,
    point: Vec<f64>,
    tolerances: &Tolerances,
    compatibility: &Compatibility,
) {
    report.candidate = Some(Candidate {
        kind: CandidateKind::FinalIterate,
        primal: point.clone(),
        objective: None,
        row_dual: None,
        bound_dual: None,
        reduced_costs: None,
        slacks: None,
        commitment: None,
    });
    state.finish(report);
    if state.terminal.is_some() {
        return;
    }
    let validated = crate::quality::contained(|| {
        state.execution.check()?;
        domain.check(&point)?;
        let mut values = vec![0.0; oracle.contract().rows.len()];
        oracle.residual(&point, &mut values)?;
        domain.check(&point)?;
        let rows = oracle
            .contract()
            .rows
            .iter()
            .zip(&values)
            .zip(&tolerances.rows)
            .map(|((id, value), tolerance)| Violation {
                id: *id,
                physical: value.abs(),
                tolerance: *tolerance,
            })
            .collect();
        let bounds = oracle
            .contract()
            .variables
            .iter()
            .zip(&tolerances.variables)
            .map(|(v, t)| Violation {
                id: v.id,
                physical: 0.0,
                tolerance: *t,
            })
            .collect();
        let quality = Quality::new(rows, bounds, vec![])?;
        let observation = oracle.observe(values)?;
        state.execution.check()?;
        Ok((quality, observation))
    });
    match validated {
        Ok((quality, observation)) => {
            if report.termination.category == Termination::Success && quality.feasible() {
                report.termination.assurance = Assurance::Feasible;
            }
            report.quality = Some(quality);
            report.observation = Some(observation);
            report.warm_start = Some(WarmStart {
                origin: None,
                compatibility: compatibility.clone(),
                payload: WarmPayload::Root(point),
            });
        }
        Err(error) => report.record_validation_failure(error),
    }
    state.finish(report);
}
