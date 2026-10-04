// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared NASM products over compact external coordinates.
use super::derived as owner;
use super::*;
use pse_math::{
    derived::{DerivedFamily, FamilyClass},
    index::GlobalCol,
    sparse::AssemblyMatrix,
};
use std::{collections::BTreeSet, sync::Arc};

/// Read-only compact coordinates supplied by NASM's declared ghost scatter.
#[derive(Clone, Copy, Debug)]
pub struct GhostCoordinates<'a> {
    /// Original columns in the immutable compact scatter order.
    pub columns: &'a [GlobalCol],
    /// Current block unknowns overlay the frozen outer values in this same order.
    pub values: &'a [f64],
}
/// Actual local evaluator for a prepared block family. Local equation and derivative
/// order are the family's row/column maps and incidence, never a second equation authority.
pub trait BlockOracle: std::fmt::Debug {
    /// Prepared original contract and complete local/external incidence.
    fn family(&self) -> &Arc<DerivedFamily>;
    /// Actual frozen parameter/provider binding served throughout this attempt.
    fn realization(&self) -> pse_ids::ContentHash;
    /// Additional original external inputs selected by the actual compiler, including
    /// guard-only slots. These are execution inputs, independent of derivative incidence.
    /// The native scatter also includes every declared equation external automatically.
    fn external_coordinates(&self) -> Vec<GlobalCol> {
        Vec::new()
    }
    /// Evaluate the declared local rows using current local and compact external values.
    fn residual(
        &mut self,
        local: &[f64],
        ghosts: GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError>;
    /// Evaluate analytic Jacobian values in `family().incidence()` order.
    fn jacobian(
        &mut self,
        local: &[f64],
        ghosts: GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError>;
}
/// Admitted local evaluator with a declared correction interior and domain provenance.
#[derive(Debug)]
pub struct BlockSpec<'a> {
    oracle: &'a mut dyn BlockOracle,
    interior: Vec<GlobalCol>,
    witness: DomainWitness,
}
impl<'a> BlockSpec<'a> {
    /// Bind a block's correction interior and actual domain-safe admission declaration.
    /// # Errors
    /// Wrong family, nonsquare block, invalid interior or guard identity mismatch.
    pub fn new(
        oracle: &'a mut dyn BlockOracle,
        interior: Vec<GlobalCol>,
        witness: DomainWitness,
    ) -> Result<Self, ProblemError> {
        let family = oracle.family();
        let columns = family
            .coordinate_map()
            .ok_or_else(|| ProblemError::Contract("PETSc block coordinate map".into()))?;
        let rows = family
            .row_map()
            .ok_or_else(|| ProblemError::Contract("PETSc block row map".into()))?;
        if family.class() != FamilyClass::BlockSubsystem
            || columns.len() != rows.len()
            || columns.is_empty()
            || interior.is_empty()
            || interior.iter().any(|c| !columns.contains(c))
            || interior.iter().copied().collect::<BTreeSet<_>>().len() != interior.len()
            || witness.source != family.original().obligations().guards
        {
            return Err(ProblemError::Contract(
                "PETSc domain-safe square block declaration or correction interior".into(),
            ));
        }
        let external = oracle.external_coordinates();
        if external
            .iter()
            .any(|c| c.get() >= family.original().coordinates().len() || columns.contains(c))
            || external.iter().copied().collect::<BTreeSet<_>>().len() != external.len()
        {
            return Err(ProblemError::Contract("PETSc block actual external input map must be unique original columns outside the local unknowns".into()));
        }
        let original = family.original();
        // This mechanical inventory serves matching only. Native derivative/smoothness
        // admission is consumed from the complete original oracle in the composition.
        let contract = crate::OracleContract {
            identity: family.key(),
            variables: columns
                .iter()
                .map(|c| {
                    let v = &original.coordinates()[c.get()];
                    crate::Variable {
                        id: v.id,
                        lower: v.lower,
                        upper: v.upper,
                    }
                })
                .collect(),
            rows: rows
                .iter()
                .map(|r| original.constraints()[r.get()].id)
                .collect(),
            derivatives: pse_kernels::DerivativeOrder::Value,
            smoothness: pse_kernels::DerivativeOrder::Value,
        };
        let pattern = AssemblyMatrix::new(
            rows.len(),
            columns.len(),
            family.incidence(),
            i32::MAX as usize,
        )?;
        crate::structural::check(
            &contract,
            pattern.matrix().symbolic(),
            &vec![(0.0, 0.0); rows.len()],
            crate::structural::Mode::Roots,
            None,
        )?;
        Ok(Self {
            oracle,
            interior,
            witness,
        })
    }
}
/// Whole original reconstruction and explicit block products supplied to NASM.
#[derive(Debug)]
pub struct BlockComposition<'a> {
    original: &'a mut dyn NleOracle,
    domain: &'a mut dyn DomainOracle,
    blocks: Vec<BlockSpec<'a>>,
}
impl<'a> BlockComposition<'a> {
    /// Admit complete row coverage and exactly one correction interior owner per original
    /// coordinate. Overlap and external support remain explicit in each prepared family.
    /// # Errors
    /// Differing originals/guards, incomplete coverage or overlapping interiors.
    pub fn new(
        original: &'a mut dyn NleOracle,
        domain: &'a mut dyn DomainOracle,
        blocks: Vec<BlockSpec<'a>>,
    ) -> Result<Self, ProblemError> {
        let first = blocks.first().ok_or_else(|| {
            ProblemError::Contract("PETSc NASM requires explicit block products".into())
        })?;
        let full = first.oracle.family().original();
        owner::original(full, original)?;
        if domain.source() != full.obligations().guards {
            return Err(ProblemError::Contract(
                "PETSc block original domain owner".into(),
            ));
        }
        let n = full.coordinates().len();
        let mut rows = BTreeSet::new();
        let mut interiors = BTreeSet::new();
        for block in &blocks {
            let family = block.oracle.family();
            if family.original() != full || block.witness.source != domain.source() {
                return Err(ProblemError::Contract(
                    "PETSc block products do not share the admitted original and guard obligations"
                        .into(),
                ));
            }
            rows.extend(
                family
                    .row_map()
                    .ok_or_else(|| ProblemError::Contract("PETSc block rows".into()))?
                    .iter()
                    .map(|r| r.get()),
            );
            for col in &block.interior {
                if !interiors.insert(col.get()) {
                    return Err(ProblemError::Contract(
                        "PETSc correction interiors overlap".into(),
                    ));
                }
            }
        }
        if rows != (0..n).collect() || interiors != (0..n).collect() {
            return Err(ProblemError::Contract(
                "PETSc block products must cover every original row and correction coordinate"
                    .into(),
            ));
        }
        Ok(Self {
            original,
            domain,
            blocks,
        })
    }
    /// Complete immutable original contract consumed by every block.
    pub fn original(&self) -> &pse_math::derived::OriginalContract {
        self.blocks[0].oracle.family().original()
    }
}
struct Shared<'a> {
    state: CallbackState,
    domain: &'a mut dyn DomainOracle,
    anchor: Vec<f64>,
    scratch: Vec<f64>,
}
struct Parent<'a, 'b> {
    original: &'a mut dyn NleOracle,
    shared: *mut Shared<'b>,
    n: usize,
    rows: Vec<i32>,
    columns: Vec<i32>,
    snes: ffi::Handle,
}
struct Child<'a, 'b> {
    oracle: &'a mut dyn BlockOracle,
    shared: *mut Shared<'b>,
    family: Arc<DerivedFamily>,
    realization: pse_ids::ContentHash,
    witness: DomainWitness,
    columns: Vec<GlobalCol>,
    ghosts: Vec<GlobalCol>,
    rows: Vec<i32>,
    jac_columns: Vec<i32>,
    interior: Vec<i32>,
    overlap: Vec<i32>,
    ghost_indices: Vec<i32>,
    options: ffi::Handle,
    snes: ffi::Handle,
    label: String,
}
fn finite(values: &[f64]) -> Result<(), ProblemError> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(ProblemError::numerical("nonfinite PETSc block action"))
    } else {
        Ok(())
    }
}
unsafe extern "C" fn parent_function(ctx: *mut c_void, x: ffi::Handle, f: ffi::Handle) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Parent<'_, '_>>() };
    // The native serial owner dispatches one callback at a time. Shared is pinned and
    // outlives every parent/child registration; no callback lends its pointer to suppliers.
    // SAFETY: Shared is pinned through teardown and parent/child callbacks execute serially without lending its pointer to suppliers.
    let shared = unsafe { &mut *c.shared };
    let n = c.n;
    let output = shared.state.evaluate("blocks.original_residual", || {
        let point = read(x, n)?;
        shared.domain.check(&point)?;
        let mut values = vec![0.0; n];
        c.original.residual(&point, &mut values)?;
        finite(&values)?;
        shared.domain.check(&point)?;
        shared.anchor.copy_from_slice(&point);
        Ok(values)
    });
    let Some(output) = output else {
        owner::terminal(
            &mut shared.state,
            "domain-safe NASM original callback refused",
        );
        return ffi::EVAL_TERMINAL;
    };
    owner::publish(&mut shared.state, || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_write(f, index(n)?, output.as_ptr()) },
            "NASM original publication",
        )
    })
}
unsafe extern "C" fn parent_jacobian(
    ctx: *mut c_void,
    x: ffi::Handle,
    a: ffi::Handle,
    p: ffi::Handle,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Parent<'_, '_>>() };
    // SAFETY: Shared is pinned through teardown and parent/child callbacks execute serially without lending its pointer to suppliers.
    let shared = unsafe { &mut *c.shared };
    let n = c.n;
    let output = shared.state.evaluate("blocks.original_jacobian", || {
        let point = read(x, n)?;
        shared.domain.check(&point)?;
        let mut values = vec![0.0; c.rows.len()];
        c.original.jacobian(&point, &mut values)?;
        finite(&values)?;
        shared.domain.check(&point)?;
        Ok(values)
    });
    let Some(output) = output else {
        owner::terminal(
            &mut shared.state,
            "domain-safe NASM original Jacobian refused",
        );
        return ffi::EVAL_TERMINAL;
    };
    let rows = c.rows.as_ptr();
    let columns = c.columns.as_ptr();
    owner::publish(&mut shared.state, || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_mat_write(a, index(output.len())?, rows, columns, output.as_ptr())
            },
            "NASM original Jacobian publication",
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
                "NASM original PC publication",
            )?;
        }
        Ok(())
    })
}
unsafe extern "C" fn block_function(
    ctx: *mut c_void,
    x: ffi::Handle,
    ghost: ffi::Handle,
    f: ffi::Handle,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Child<'_, '_>>() };
    // SAFETY: Shared is pinned through teardown and parent/child callbacks execute serially without lending its pointer to suppliers.
    let shared = unsafe { &mut *c.shared };
    let n = c.columns.len();
    let output = shared.state.evaluate(&format!("{}.residual", c.label), || {
        if c.oracle.family().key() != c.family.key() || c.oracle.realization() != c.realization {
            return Err(ProblemError::Contract(
                "PETSc block supplier changed its admitted family/binding".into(),
            ));
        }
        let local = read(x, n)?;
        let ghost = read(ghost, c.ghosts.len())?;
        point_fields(
            shared.domain,
            &shared.anchor,
            &mut shared.scratch,
            &c.ghosts,
            &ghost,
        )?;
        let mut values = vec![0.0; n];
        c.oracle.residual(
            &local,
            GhostCoordinates {
                columns: &c.ghosts,
                values: &ghost,
            },
            &mut values,
        )?;
        finite(&values)?;
        shared.domain.check(&shared.scratch)?;
        if c.oracle.realization() != c.realization {
            return Err(ProblemError::Contract(
                "PETSc block binding changed during residual evaluation".into(),
            ));
        }
        Ok(values)
    });
    let Some(output) = output else {
        owner::terminal(&mut shared.state, "domain-safe NASM block residual refused");
        return ffi::EVAL_TERMINAL;
    };
    owner::publish(&mut shared.state, || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_write(f, index(n)?, output.as_ptr()) },
            "block residual publication",
        )
    })
}
// Split fields permit the callback-state owner and original-domain owner to be borrowed
// independently. One full original scratch vector is shared across all compact blocks.
fn point_fields(
    domain: &mut dyn DomainOracle,
    anchor: &[f64],
    scratch: &mut [f64],
    columns: &[GlobalCol],
    values: &[f64],
) -> Result<(), ProblemError> {
    if columns.len() != values.len() {
        return Err(ProblemError::Contract("PETSc compact ghost extent".into()));
    }
    scratch.copy_from_slice(anchor);
    for (col, value) in columns.iter().zip(values) {
        scratch[col.get()] = *value;
    }
    domain.check(scratch)
}
unsafe extern "C" fn block_jacobian(
    ctx: *mut c_void,
    x: ffi::Handle,
    ghost: ffi::Handle,
    a: ffi::Handle,
    p: ffi::Handle,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Child<'_, '_>>() };
    // SAFETY: Shared is pinned through teardown and parent/child callbacks execute serially without lending its pointer to suppliers.
    let shared = unsafe { &mut *c.shared };
    let n = c.columns.len();
    let output = shared.state.evaluate(&format!("{}.jacobian", c.label), || {
        if c.oracle.family().key() != c.family.key() || c.oracle.realization() != c.realization {
            return Err(ProblemError::Contract(
                "PETSc block derivative supplier changed its admitted family/binding".into(),
            ));
        }
        let local = read(x, n)?;
        let ghost = read(ghost, c.ghosts.len())?;
        point_fields(
            shared.domain,
            &shared.anchor,
            &mut shared.scratch,
            &c.ghosts,
            &ghost,
        )?;
        let mut values = vec![0.0; c.rows.len()];
        c.oracle.jacobian(
            &local,
            GhostCoordinates {
                columns: &c.ghosts,
                values: &ghost,
            },
            &mut values,
        )?;
        finite(&values)?;
        shared.domain.check(&shared.scratch)?;
        if c.oracle.realization() != c.realization {
            return Err(ProblemError::Contract(
                "PETSc block binding changed during derivative evaluation".into(),
            ));
        }
        Ok(values)
    });
    let Some(output) = output else {
        owner::terminal(&mut shared.state, "domain-safe NASM block Jacobian refused");
        return ffi::EVAL_TERMINAL;
    };
    let rows = c.rows.as_ptr();
    let columns = c.jac_columns.as_ptr();
    owner::publish(&mut shared.state, || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_mat_write(a, index(output.len())?, rows, columns, output.as_ptr())
            },
            "block Jacobian publication",
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
                "block PC publication",
            )?;
        }
        Ok(())
    })
}
unsafe extern "C" fn parent_convergence(
    ctx: *mut c_void,
    iteration: i32,
    xnorm: f64,
    ynorm: f64,
    fnorm: f64,
    reason: *mut i32,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Parent<'_, '_>>() };
    // SAFETY: Shared is pinned through teardown and parent/child callbacks execute serially without lending its pointer to suppliers.
    let shared = unsafe { &mut *c.shared };
    let mut observed = 0;
    let ok = shared
        .state
        .evaluate("blocks.outer_convergence", || {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe {
                    ffi::pse_petsc_snes_default_convergence(
                        c.snes,
                        iteration,
                        xnorm,
                        ynorm,
                        fnorm,
                        &mut observed,
                    )
                },
                "NASM native convergence",
            )
        })
        .is_some();
    // SAFETY: The C trampoline supplies a non-null aligned writable scalar for this callback result.
    unsafe {
        *reason = if ok {
            observed
        } else {
            ffi::SNES_USER_DIVERGED
        }
    };
    ffi::EVAL_OK
}
unsafe extern "C" fn child_convergence(
    ctx: *mut c_void,
    iteration: i32,
    xnorm: f64,
    ynorm: f64,
    fnorm: f64,
    reason: *mut i32,
) -> i32 {
    // SAFETY: The serial C trampoline supplies this pinned callback context; its exclusive borrow ends before the callback returns.
    let c = unsafe { &mut *ctx.cast::<Child<'_, '_>>() };
    // SAFETY: Shared is pinned through teardown and parent/child callbacks execute serially without lending its pointer to suppliers.
    let shared = unsafe { &mut *c.shared };
    let mut observed = 0;
    let ok = shared
        .state
        .evaluate(&format!("{}.convergence", c.label), || {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe {
                    ffi::pse_petsc_snes_default_convergence(
                        c.snes,
                        iteration,
                        xnorm,
                        ynorm,
                        fnorm,
                        &mut observed,
                    )
                },
                "block native convergence",
            )
        })
        .is_some();
    // SAFETY: The C trampoline supplies a non-null aligned writable scalar for this callback result.
    unsafe {
        *reason = if ok {
            observed
        } else {
            ffi::SNES_USER_DIVERGED
        }
    };
    ffi::EVAL_OK
}
/// Execute explicit unbounded block products through PETSc's automatic DMShell NASM
/// ownership route, then assess the independently reconstructed full original point.
/// # Errors
/// Declaration, finite foreign reservation, native setup or destruction fails.
pub fn solve_blocks(
    composition: &mut BlockComposition<'_>,
    initial: &[f64],
    settings: &Settings,
    request: SolveRequest<'_>,
) -> Result<SolveReport, ProblemError> {
    let n = composition.original.contract().variables.len();
    owner::original(
        composition.blocks[0].oracle.family().original(),
        composition.original,
    )?;
    owner::controls(&request, settings, Method::NonlinearAdditiveSchwarz, n)?;
    let SolveRequest {
        controls,
        accuracy,
        execution,
        tolerances,
        warm,
        compatibility,
    } = request;
    let schwarz = settings
        .schwarz
        .ok_or_else(|| ProblemError::Contract("PETSc Schwarz controls".into()))?;
    let start = owner::start(initial, warm, compatibility, n)?;
    let mut vectors = n
        .checked_mul(8)
        .ok_or_else(|| ProblemError::memory("PETSc parent vector shape"))?;
    let mut entries = composition.blocks[0]
        .oracle
        .family()
        .original()
        .incidence()
        .len();
    let mut indices = n;
    let mut planned = 0_usize;
    let mut ghost_maps = Vec::new();
    for block in &composition.blocks {
        let family = block.oracle.family();
        let columns = family
            .coordinate_map()
            .ok_or_else(|| ProblemError::Contract("PETSc block columns".into()))?;
        let external = block.oracle.external_coordinates();
        if external.iter().any(|c| c.get() >= n || columns.contains(c))
            || external.iter().copied().collect::<BTreeSet<_>>().len() != external.len()
        {
            return Err(ProblemError::Contract(
                "PETSc actual block input map changed before dispatch".into(),
            ));
        }
        let mut ghosts = columns.iter().copied().collect::<BTreeSet<_>>();
        ghosts.extend(family.external_incidence().iter().map(|e| e.col));
        ghosts.extend(external);
        let ghosts = ghosts.into_iter().collect::<Vec<_>>();
        vectors = vectors
            .checked_add(
                columns
                    .len()
                    .checked_mul(5)
                    .and_then(|v| v.checked_add(ghosts.len()))
                    .ok_or_else(|| ProblemError::memory("PETSc block vector shape"))?,
            )
            .ok_or_else(|| ProblemError::memory("PETSc aggregate vectors"))?;
        entries = entries
            .checked_add(family.incidence().len())
            .ok_or_else(|| ProblemError::memory("PETSc aggregate sparse entries"))?;
        indices = indices
            .checked_add(columns.len())
            .and_then(|v| v.checked_add(block.interior.len()))
            .and_then(|v| v.checked_add(ghosts.len()))
            .ok_or_else(|| ProblemError::memory("PETSc compact scatter shape"))?;
        planned = planned
            .checked_add(owner::shape(
                &execution,
                0,
                family.incidence().len(),
                0,
                settings,
                columns.len(),
            )?)
            .ok_or_else(|| ProblemError::memory("PETSc child workspace shape"))?;
        ghost_maps.push(ghosts);
    }
    planned = planned
        .checked_add(owner::shape(
            &execution, vectors, entries, indices, settings, n,
        )?)
        .ok_or_else(|| ProblemError::memory("PETSc composition shape"))?;
    if planned > execution.memory.unwrap_or(0) {
        return Err(ProblemError::memory(
            "PETSc complete compact block shape exceeds foreign reservation",
        ));
    }
    let mut shared = Box::new(Shared {
        state: CallbackState::new(execution.clone()),
        domain: composition.domain,
        anchor: start.to_vec(),
        scratch: vec![0.0; n],
    });
    let shared_ptr = ptr::from_mut(shared.as_mut());
    let crate::nlp_pattern::Pattern { rows, columns } =
        crate::nlp_pattern::Pattern::new(composition.original.jacobian_pattern(), false)?;
    let mut parent = Box::new(Parent {
        original: composition.original,
        shared: shared_ptr,
        n,
        rows,
        columns,
        snes: ptr::null_mut(),
    });
    let admission = Admission::enter(&execution)?;
    let _threads = crate::mkl::Threads::enter(1)?;
    let admitted = shared.state.evaluate("blocks.initial_original", || {
        shared.domain.check(start)?;
        let mut values = vec![0.0; n];
        parent.original.residual(start, &mut values)?;
        finite(&values)?;
        shared.domain.check(start)?;
        Ok(())
    });
    if admitted.is_none() {
        owner::terminal(&mut shared.state, "invalid initial block original domain");
        shared.state.trial_rejections = 0;
        let mut report = SolveReport::new(
            Backend::Petsc,
            parent.original.contract(),
            NativeTermination {
                code: 0,
                name: "PETSC_BLOCK_INITIAL_DOMAIN_REFUSED".into(),
                message: None,
                category: Termination::Evaluation,
                assurance: Assurance::None,
            },
            &execution,
        );
        shared.state.finish(&mut report);
        return Ok(report);
    }
    // Declared before the native RAII owner: contexts outlive native destruction on
    // setup errors too, not only the explicit successful close below.
    let mut children = Vec::new();
    let mut objects = Objects::new(&admission);
    let options = objects.create(
        ffi::pse_petsc_options_create,
        ffi::pse_petsc_options_destroy,
    )?;
    // Child private options must be registered before parent SNES, so reverse destruction
    // keeps every borrowed options database alive through the parent's child teardown.
    for (number, (block, ghosts)) in composition.blocks.iter_mut().zip(ghost_maps).enumerate() {
        let family = block.oracle.family().clone();
        let columns = family
            .coordinate_map()
            .ok_or_else(|| ProblemError::Contract("PETSc local map".into()))?
            .to_vec();
        let rows = family
            .incidence()
            .iter()
            .map(|e| index(e.row.get()))
            .collect::<Result<Vec<_>, _>>()?;
        let jac_columns = family
            .incidence()
            .iter()
            .map(|e| index(e.col.get()))
            .collect::<Result<Vec<_>, _>>()?;
        let overlap = columns
            .iter()
            .map(|c| index(c.get()))
            .collect::<Result<Vec<_>, _>>()?;
        let interior = block
            .interior
            .iter()
            .map(|c| index(c.get()))
            .collect::<Result<Vec<_>, _>>()?;
        let ghost_indices = ghosts
            .iter()
            .map(|c| index(c.get()))
            .collect::<Result<Vec<_>, _>>()?;
        let options = objects.create(
            ffi::pse_petsc_options_create,
            ffi::pse_petsc_options_destroy,
        )?;
        let realization = block.oracle.realization();
        children.push(Box::new(Child {
            oracle: block.oracle,
            shared: shared_ptr,
            family,
            realization,
            witness: block.witness,
            columns,
            ghosts,
            rows,
            jac_columns,
            interior,
            overlap,
            ghost_indices,
            options,
            snes: ptr::null_mut(),
            label: format!("block.{number}"),
        }));
    }
    let x = objects.vector(start)?;
    let f = objects.vector(&vec![0.0; n])?;
    let matrix = owner::matrix(&mut objects, n, &parent.rows)?;
    let snes = objects.create(ffi::pse_petsc_snes_create, ffi::pse_petsc_snes_destroy)?;
    parent.snes = snes;
    let parent_ctx = ptr::from_mut(parent.as_mut()).cast();
    let descriptors = children
        .iter_mut()
        .map(|child| {
            Ok(ffi::Block {
                size: index(child.columns.len())?,
                overlap: child.overlap.as_ptr(),
                interior_size: index(child.interior.len())?,
                interior: child.interior.as_ptr(),
                ghost_size: index(child.ghosts.len())?,
                ghost: child.ghost_indices.as_ptr(),
                nonzeros: index(child.rows.len())?,
                rows: child.rows.as_ptr(),
                columns: child.jac_columns.as_ptr(),
                function: block_function,
                jacobian: block_jacobian,
                context: ptr::from_mut(child.as_mut()).cast(),
                options: child.options,
            })
        })
        .collect::<Result<Vec<_>, ProblemError>>()?;
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_type(snes, c"nasm".as_ptr()) },
            "explicit block method",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_options(snes, options) },
            "NASM private options",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_function(snes, f, parent_function, parent_ctx) },
            "full original NASM residual",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_jacobian(snes, matrix, matrix, parent_jacobian, parent_ctx)
            },
            "full original NASM Jacobian",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_convergence(snes, parent_convergence, parent_ctx) },
            "NASM stop checkpoint",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_tolerances(
                    snes,
                    accuracy.feasibility,
                    0.0,
                    0.0,
                    index(controls.iterations as usize)?,
                    i32::MAX,
                )
            },
            "strict original NASM convergence",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_nasm_restrict(snes, i32::from(schwarz.restricted)) },
            "declared correction extension",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_nasm_damping(snes, schwarz.damping) },
            "declared NASM damping",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_nasm_dm(snes, x, index(descriptors.len())?, descriptors.as_ptr())
            },
            "automatic DMShell block ownership",
        )?;
        for (number, child) in children.iter_mut().enumerate() {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe { ffi::pse_petsc_nasm_subsolver(snes, index(number)?, &mut child.snes) },
                "borrowed NASM child",
            )?;
            owner::configure(child.snes, settings, accuracy, controls.iterations)?;
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe { ffi::pse_petsc_snes_error_if_not_converged(child.snes, 1) },
                "do not publish unsuccessful block corrections",
            )?;
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe {
                    ffi::pse_petsc_snes_convergence(
                        child.snes,
                        child_convergence,
                        ptr::from_mut(child.as_mut()).cast(),
                    )
                },
                "child checkpoint",
            )?;
        }
    }
    let code = {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_snes_solve(snes, x) }
    };
    let (mut reason, mut iterations, mut evaluations, mut linear, mut norm) = (0, 0, 0, 0, 0.0);
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_statistics(
                    snes,
                    &mut reason,
                    &mut iterations,
                    &mut evaluations,
                    &mut linear,
                    &mut norm,
                )
            },
            "NASM parent counters",
        )?;
    }
    let mut name = ptr::null();
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_reason_name(snes, &mut name) },
            "NASM native reason",
        )?;
    }
    let name = if name.is_null() {
        "SNES_UNKNOWN".into()
    } else {
        // SAFETY: The pointer was checked non-null; PETSc supplies a live null-terminated library string for this query.
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    };
    let mut report = SolveReport::new(
        Backend::Petsc,
        parent.original.contract(),
        native_termination(reason, name),
        &execution,
    );
    if reason == 0 && iterations >= index(controls.iterations as usize)? {
        report.termination.category = Termination::IterationLimit;
    }
    let native_terminal = finish_native_error(&mut report, &mut shared.state, code, "block solve");
    for (number, child) in children.iter().enumerate() {
        if !shared
            .state
            .counts
            .contains_key(&format!("{}.residual", child.label))
        {
            continue;
        }
        let (
            mut child_reason,
            mut child_iterations,
            mut child_evaluations,
            mut child_linear,
            mut child_norm,
        ) = (0, 0, 0, 0, 0.0);
        {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe {
                    ffi::pse_petsc_snes_statistics(
                        child.snes,
                        &mut child_reason,
                        &mut child_iterations,
                        &mut child_evaluations,
                        &mut child_linear,
                        &mut child_norm,
                    )
                },
                "last child solve counters",
            )?;
        }
        report.metrics.insert(
            format!("block.{number}.last_reason"),
            Metric::Integer(child_reason.into()),
        );
        report.metrics.insert(
            format!("block.{number}.last_iterations"),
            Metric::Integer(child_iterations.into()),
        );
        if code != 0 && child_reason < 0 && !native_terminal {
            let mut child_name = ptr::null();
            {
                check(
                    // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                    unsafe { ffi::pse_petsc_snes_reason_name(child.snes, &mut child_name) },
                    "child stop reason",
                )?;
            }
            let child_name = if child_name.is_null() {
                "SNES_UNKNOWN".into()
            } else {
                // SAFETY: The pointer was checked non-null; PETSc supplies a live null-terminated library string for this query.
                unsafe { CStr::from_ptr(child_name) }
                    .to_string_lossy()
                    .into_owned()
            };
            let stopped = native_termination(child_reason, child_name);
            report.termination.category = stopped.category;
            report.termination.message = Some(format!(
                "block {number}: {} ({child_reason}); {}",
                stopped.name,
                report
                    .termination
                    .message
                    .as_deref()
                    .unwrap_or("native parent error")
            ));
        }
    }
    report.evidence.start_submitted = warm.is_some();
    report.evidence.work.iterations = u64::try_from(iterations).ok();
    report.metrics.extend(std::collections::BTreeMap::from([
        (
            "SNESGetNumberFunctionEvals".into(),
            Metric::Integer(evaluations.into()),
        ),
        ("SNESGetFunctionNorm".into(), Metric::Real(norm)),
        (
            "planned.compact_bytes".into(),
            Metric::Integer(i64::try_from(planned).unwrap_or(i64::MAX)),
        ),
    ]));
    report
        .provenance
        .insert("native".into(), format!("PETSc {}", ffi::VERSION));
    report.provenance.insert("build".into(), build().to_hex());
    report.provenance.insert(
        "original".into(),
        children[0].family.original().identity().to_hex(),
    );
    report.provenance.insert(
        "normalization".into(),
        children[0].family.original().normalization().to_hex(),
    );
    report
        .provenance
        .insert("guards".into(), shared.domain.source().to_hex());
    for (number, child) in children.iter().enumerate() {
        report.provenance.insert(
            format!("block.{number}.family"),
            child.family.key().to_hex(),
        );
        report.provenance.insert(
            format!("block.{number}.realization"),
            child.realization.to_hex(),
        );
        report.provenance.insert(
            format!("block.{number}.domain_declaration"),
            child.witness.declaration.to_hex(),
        );
    }
    owner::effective(&mut report, settings, accuracy, controls.iterations, "sub_")?;
    for (key, value) in [
        ("snes_type", OptionValue::Text("nasm".into())),
        ("snes_atol", OptionValue::Real(accuracy.feasibility)),
        ("snes_rtol", OptionValue::Real(0.0)),
        ("snes_stol", OptionValue::Real(0.0)),
        (
            "snes_max_it",
            OptionValue::Integer(index(controls.iterations as usize)?),
        ),
        ("snes_max_funcs", OptionValue::Integer(i32::MAX)),
        ("snes_norm_schedule", OptionValue::Text("always".into())),
        (
            "sub_snes_error_if_not_converged",
            OptionValue::Text("true".into()),
        ),
        ("snes_nasm_damping", OptionValue::Real(schwarz.damping)),
        (
            "snes_nasm_type",
            OptionValue::Text(
                if schwarz.restricted {
                    "restrict"
                } else {
                    "basic"
                }
                .into(),
            ),
        ),
    ] {
        report.options.insert(key.into(), value);
    }
    match read(x, n) {
        Ok(point) if native_terminal => retain_native_candidate(&mut report, point),
        Ok(point) => owner::assess(
            &mut report,
            parent.original,
            shared.domain,
            &mut shared.state,
            point,
            tolerances,
            compatibility,
        ),
        Err(error) => {
            shared.state.finish(&mut report);
            if !native_terminal {
                report.record_validation_failure(error);
            }
        }
    }
    objects.close()?;
    Ok(report)
}
