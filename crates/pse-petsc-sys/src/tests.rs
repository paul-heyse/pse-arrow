// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::{
    ffi::CStr,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};

struct Object {
    handle: Handle,
    destroy: unsafe extern "C" fn(*mut Handle) -> i32,
}
impl Drop for Object {
    fn drop(&mut self) {
        // SAFETY: each test object owns one native handle; transferred handles are null.
        let code = unsafe { (self.destroy)(&mut self.handle) };
        // Preserve the original failing assertion during unwinding; on normal
        // completion, every native teardown failure independently fails the test.
        if !std::thread::panicking() {
            assert_eq!(code, 0);
        }
    }
}
fn object(
    create: unsafe extern "C" fn(*mut Handle) -> i32,
    destroy: unsafe extern "C" fn(*mut Handle) -> i32,
) -> Object {
    let mut handle = ptr::null_mut();
    // SAFETY: process initialization precedes creation, with an owned output slot.
    unsafe {
        assert_eq!(create(&mut handle), 0);
    }
    Object { handle, destroy }
}
fn vector(values: &[f64]) -> Object {
    let mut handle = ptr::null_mut();
    // SAFETY: nonempty slices provide exactly the admitted sequential dimension.
    {
        assert_eq!(
            // SAFETY: the positive declared sequential dimension and writable handle slot are valid under the initialized process.
            unsafe { pse_petsc_vec_create(values.len() as i32, &mut handle) },
            0
        );
        assert_eq!(
            // SAFETY: the vector is live and the readable values array matches its declared sequential dimension.
            unsafe { pse_petsc_vec_write(handle, values.len() as i32, values.as_ptr()) },
            0
        );
    }
    Object {
        handle,
        destroy: pse_petsc_vec_destroy,
    }
}
fn matrix(capacity: &[i32]) -> Object {
    let mut handle = ptr::null_mut();
    // SAFETY: one row-capacity entry per row, held through this creation call.
    unsafe {
        assert_eq!(
            pse_petsc_mat_create(
                capacity.len() as i32,
                capacity.len() as i32,
                capacity.as_ptr(),
                &mut handle
            ),
            0
        );
    }
    Object {
        handle,
        destroy: pse_petsc_mat_destroy,
    }
}
fn callback(work: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(work)).unwrap_or(EVAL_TERMINAL)
}
fn read<const N: usize>(x: Handle) -> [f64; N] {
    let mut values = [0.0; N];
    // SAFETY: callers supply a live sequential vector with N entries.
    unsafe {
        assert_eq!(
            pse_petsc_vec_read(x, N as i32, values.as_mut_ptr()),
            0,
            "native callback expected {N} coordinates"
        );
    }
    values
}
fn write<const N: usize>(x: Handle, values: &[f64; N]) -> i32 {
    // SAFETY: callers supply a live sequential vector with N entries.
    unsafe { pse_petsc_vec_write(x, N as i32, values.as_ptr()) }
}
fn diagonal(matrix: Handle, values: &[f64]) -> i32 {
    let indices: Vec<_> = (0..values.len() as i32).collect();
    // SAFETY: live sparse matrix, matching triplet arrays held through assembly.
    unsafe {
        pse_petsc_mat_write(
            matrix,
            values.len() as i32,
            indices.as_ptr(),
            indices.as_ptr(),
            values.as_ptr(),
        )
    }
}

#[derive(Default)]
struct RootContext {
    target: f64,
    log: bool,
    calls: usize,
    rejections: usize,
    panic: bool,
    terminal: bool,
}
unsafe extern "C" fn residual(ctx: *mut c_void, x: Handle, f: Handle) -> i32 {
    callback(|| {
        // SAFETY: callback context is a boxed RootContext alive through SNES destruction.
        let ctx = unsafe { &mut *ctx.cast::<RootContext>() };
        if ctx.terminal {
            return EVAL_TERMINAL;
        }
        ctx.calls += 1;
        if ctx.panic {
            ctx.terminal = true;
            std::panic::resume_unwind(Box::new("contained PETSc callback witness"));
        }
        let x = read::<1>(x)[0];
        if ctx.log && x <= 0.0 {
            ctx.rejections += 1;
            return EVAL_DOMAIN;
        }
        write(
            f,
            &[if ctx.log {
                x.ln() - ctx.target.ln()
            } else {
                x - ctx.target
            }],
        )
    })
}
unsafe extern "C" fn jacobian(ctx: *mut c_void, x: Handle, a: Handle, p: Handle) -> i32 {
    callback(|| {
        // SAFETY: registered RootContext remains alive for the complete solve.
        let ctx = unsafe { &mut *ctx.cast::<RootContext>() };
        let x = read::<1>(x)[0];
        let values = [if ctx.log { x.recip() } else { 1.0 }];
        let code = diagonal(a, &values);
        if code != 0 || a == p {
            code
        } else {
            diagonal(p, &values)
        }
    })
}
struct Root {
    solver: Object,
    residual: Object,
    _matrix: Object,
    primal: Object,
    _options: Object,
}
fn root(context: &mut RootContext, initial: f64) -> Root {
    let solver = object(pse_petsc_snes_create, pse_petsc_snes_destroy);
    let residual = vector(&[0.0]);
    let matrix = matrix(&[1]);
    let primal = vector(&[initial]);
    let options = object(pse_petsc_options_create, pse_petsc_options_destroy);
    let ctx = ptr::from_mut(context).cast();
    // SAFETY: exclusive live solver; context and all supplied objects outlive the solve.
    {
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_type(solver.handle, c"newtontr".as_ptr()) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_options(solver.handle, options.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_layout(solver.handle, residual.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe { pse_petsc_snes_function(solver.handle, residual.handle, self::residual, ctx) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe {
                pse_petsc_snes_jacobian(solver.handle, matrix.handle, matrix.handle, jacobian, ctx)
            },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_tolerances(solver.handle, 1e-11, 1e-11, 1e-12, 80, 300) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_tr_tolerances(solver.handle, 1e-14, 100.0, 2.0) },
            0
        );
    }
    Root {
        solver,
        residual,
        _matrix: matrix,
        primal,
        _options: options,
    }
}
fn stats(snes: Handle) -> (i32, i32, i32, i32, f64) {
    let (mut reason, mut iterations, mut evaluations, mut linear, mut norm) = (0, 0, 0, 0, 0.0);
    // SAFETY: live SNES and separate writable statistic outputs.
    unsafe {
        assert_eq!(
            pse_petsc_snes_statistics(
                snes,
                &mut reason,
                &mut iterations,
                &mut evaluations,
                &mut linear,
                &mut norm
            ),
            0
        );
    }
    (reason, iterations, evaluations, linear, norm)
}
fn trust_region_controls() {
    let mut context = RootContext {
        target: 0.1,
        log: true,
        ..Default::default()
    };
    let fixture = root(&mut context, 1.0);
    // SAFETY: fixture owns all referenced objects and context for the complete call.
    unsafe {
        assert_eq!(
            pse_petsc_snes_solve(fixture.solver.handle, fixture.primal.handle),
            0
        );
    }
    let (reason, iterations, evaluations, _, norm) = stats(fixture.solver.handle);
    assert!(matches!(reason, SNES_FNORM_ABSOLUTE | SNES_FNORM_RELATIVE));
    assert!((read::<1>(fixture.primal.handle)[0] - 0.1).abs() < 1e-9);
    assert!(iterations > 0 && norm < 1e-9);
    assert!(
        context.rejections > 0,
        "invalid trials must be recovered by the native trust region"
    );
    assert_eq!(evaluations as usize, context.calls);
    drop(fixture);
    let mut invalid = RootContext {
        target: 0.1,
        log: true,
        ..Default::default()
    };
    let fixture = root(&mut invalid, -1.0);
    // SAFETY: an invalid initial point is a negative control, never profile admission.
    unsafe {
        assert_eq!(
            pse_petsc_snes_solve(fixture.solver.handle, fixture.primal.handle),
            0
        );
    }
    assert_eq!(stats(fixture.solver.handle).0, SNES_FUNCTION_DOMAIN);
    assert_eq!(invalid.calls, 1);
    drop(fixture);
    let mut panicked = RootContext {
        target: 1.0,
        panic: true,
        ..Default::default()
    };
    let fixture = root(&mut panicked, 0.0);
    // SAFETY: the callback catches its unwind and returns a terminal error to PETSc.
    unsafe {
        assert_ne!(
            pse_petsc_snes_solve(fixture.solver.handle, fixture.primal.handle),
            0
        );
    }
    assert!(panicked.terminal);
    assert_eq!(panicked.calls, 1);
}
fn private_options_controls() {
    let mut first_ctx = RootContext::default();
    let mut second_ctx = RootContext::default();
    let first = root(&mut first_ctx, 0.0);
    let second = root(&mut second_ctx, 0.0);
    // SAFETY: each independent options database outlives its attached solver.
    {
        assert_eq!(
            // SAFETY: the private options object and NUL-terminated key/value strings remain live for this call.
            unsafe {
                pse_petsc_options_set(
                    first._options.handle,
                    c"-snes_tr_delta0".as_ptr(),
                    c"0.03".as_ptr(),
                )
            },
            0
        );
        assert_eq!(
            // SAFETY: the private options object and NUL-terminated key/value strings remain live for this call.
            unsafe {
                pse_petsc_options_set(
                    second._options.handle,
                    c"-snes_tr_delta0".as_ptr(),
                    c"0.8".as_ptr(),
                )
            },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_from_options(first.solver.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_from_options(second.solver.handle) },
            0
        );
        for (fixture, expected) in [(&first, 0.03), (&second, 0.8)] {
            let (mut minimum, mut maximum, mut initial) = (0.0, 0.0, 0.0);
            assert_eq!(
                // SAFETY: the solver remains live and every statistic/tolerance output is a separate writable local.
                unsafe {
                    pse_petsc_snes_tr_get_tolerances(
                        fixture.solver.handle,
                        &mut minimum,
                        &mut maximum,
                        &mut initial,
                    )
                },
                0
            );
            assert_eq!(initial, expected);
            let mut name = ptr::null();
            assert_eq!(
                // SAFETY: the live solver owns the returned type string and the separate pointer output remains writable.
                unsafe { pse_petsc_snes_get_type(fixture.solver.handle, &mut name) },
                0
            );
            assert_eq!(
                // SAFETY: PETSc returned a NUL-terminated type string borrowed from this still-live native object.
                unsafe { CStr::from_ptr(name) },
                c"newtontr"
            );
        }
        let mut empty = 0;
        assert_eq!(
            // SAFETY: the process is initialized and the query writes this separate live integer output.
            unsafe { pse_petsc_default_options_empty(&mut empty) },
            0
        );
        assert_eq!(
            empty, 1,
            "private method settings must not mutate global options"
        );
    }
}

struct PseudoContext {
    solution: Handle,
    domain_times: Vec<f64>,
    reject_all: bool,
    steady: bool,
}
unsafe extern "C" fn ifunction(_: *mut c_void, _: f64, x: Handle, xdot: Handle, f: Handle) -> i32 {
    callback(|| write(f, &[read::<1>(xdot)[0] + read::<1>(x)[0] - 1.0]))
}
unsafe extern "C" fn ijacobian(
    _: *mut c_void,
    _: f64,
    _: Handle,
    _: Handle,
    shift: f64,
    a: Handle,
    p: Handle,
) -> i32 {
    callback(|| {
        let code = diagonal(a, &[1.0 + shift]);
        if code != 0 || a == p {
            code
        } else {
            diagonal(p, &[1.0 + shift])
        }
    })
}
unsafe extern "C" fn domain(ctx: *mut c_void, time: f64, _: Handle, accepted: *mut i32) -> i32 {
    callback(|| {
        // SAFETY: pinned PseudoContext and project i32 output live through the callback.
        let ctx = unsafe { &mut *ctx.cast::<PseudoContext>() };
        ctx.domain_times.push(time);
        // SAFETY: the registered domain callback receives one live writable i32 output.
        unsafe {
            *accepted = i32::from(!ctx.reject_all && ctx.domain_times.len() > 1);
        }
        EVAL_OK
    })
}
unsafe extern "C" fn steady(ctx: *mut c_void, ts: Handle) -> i32 {
    callback(|| {
        // SAFETY: boxed PseudoContext lives until TS teardown; its solution is retained.
        let ctx = unsafe { &mut *ctx.cast::<PseudoContext>() };
        if (read::<1>(ctx.solution)[0] - 1.0).abs() < 1e-8 {
            ctx.steady = true;
            // SAFETY: this post-step callback is executing on the live TS owner.
            unsafe { pse_petsc_ts_reason(ts, TS_USER_CONVERGED) }
        } else {
            EVAL_OK
        }
    })
}
fn pseudo_controls(reject_all: bool) {
    let ts = object(pse_petsc_ts_create, pse_petsc_ts_destroy);
    let x = vector(&[0.0]);
    let f = vector(&[0.0]);
    let a = matrix(&[1]);
    let options = object(pse_petsc_options_create, pse_petsc_options_destroy);
    let mut ctx = Box::new(PseudoContext {
        solution: x.handle,
        domain_times: vec![],
        reject_all,
        steady: false,
    });
    let mut inner = ptr::null_mut();
    // SAFETY: TS owns its borrowed inner SNES; all buffers/options/context outlive execution.
    {
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_pseudo(ts.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the live parent owns the borrowed child; the separate handle output is writable and never acquires ownership.
            unsafe { pse_petsc_ts_snes(ts.handle, &mut inner) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_type(inner, c"newtontr".as_ptr()) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_options(ts.handle, options.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the private options object and NUL-terminated key/value strings remain live for this call.
            unsafe {
                pse_petsc_options_set(options.handle, c"-ts_pseudo_fatol".as_ptr(), c"0".as_ptr())
            },
            0
        );
        assert_eq!(
            // SAFETY: the private options object and NUL-terminated key/value strings remain live for this call.
            unsafe {
                pse_petsc_options_set(options.handle, c"-ts_pseudo_frtol".as_ptr(), c"0".as_ptr())
            },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_from_options(ts.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe { pse_petsc_ts_ifunction(ts.handle, f.handle, ifunction, ptr::null_mut()) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe {
                pse_petsc_ts_ijacobian(ts.handle, a.handle, a.handle, ijacobian, ptr::null_mut())
            },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe { pse_petsc_ts_domain(ts.handle, domain, ptr::from_mut(ctx.as_mut()).cast()) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe {
                pse_petsc_ts_post_step(ts.handle, steady, ptr::from_mut(ctx.as_mut()).cast())
            },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_tolerances(inner, 1e-12, 1e-12, 1e-14, 30, 100) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_solution(ts.handle, x.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_time(ts.handle, 0.0, 1.0, 1000.0, 50) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_failures(ts.handle, 3, 4) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_adapt(ts.handle, 1e-12, 10.0, 0.5) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_ts_pseudo_growth(ts.handle, 1.1, 10.0) },
            0
        );
        assert_eq!(
            // SAFETY: the live solver retains its callbacks/context and the supplied solution vector has the admitted layout.
            unsafe { pse_petsc_ts_solve(ts.handle, x.handle) },
            0
        );
        let (mut reason, mut steps, mut nonlinear, mut rejections, mut failures, mut time, mut dt) =
            (0, 0, 0, 0, 0, 0.0, 0.0);
        assert_eq!(
            // SAFETY: the solver remains live and every statistic/tolerance output is a separate writable local.
            unsafe {
                pse_petsc_ts_statistics(
                    ts.handle,
                    &mut reason,
                    &mut steps,
                    &mut nonlinear,
                    &mut rejections,
                    &mut failures,
                    &mut time,
                    &mut dt,
                )
            },
            0
        );
        assert!(ctx.domain_times.len() >= 2);
        assert!(
            ctx.domain_times[1] < ctx.domain_times[0],
            "PETSc must reduce the rejected step"
        );
        assert!(nonlinear > 0 && rejections > 0);
        if reject_all {
            assert_eq!(reason, TS_REJECTIONS);
            assert_eq!(steps, 0);
            assert_eq!(ctx.domain_times.len(), 4);
            assert_eq!(read::<1>(x.handle), [0.0]);
            assert!(!ctx.steady);
        } else {
            assert_eq!(reason, TS_USER_CONVERGED);
            assert!(ctx.steady && steps > 0 && steps < 50);
            assert!((read::<1>(x.handle)[0] - 1.0).abs() < 1e-8);
        }
        assert_ne!(
            // SAFETY: the bridge distinguishes borrowed child handles from owned handles and refuses their destruction.
            unsafe { pse_petsc_snes_destroy(&mut inner) },
            0,
            "inner solver is borrowed from TS"
        );
    }
    // Options and callback context remain alive until the owning TS is gone.
    drop(ts);
    drop(ctx);
    drop(options);
}

unsafe extern "C" fn block_residual(_: *mut c_void, x: Handle, f: Handle) -> i32 {
    callback(|| {
        let x = read::<2>(x);
        write(f, &[x[0] - 1.0, x[1] - 2.0])
    })
}
fn manual_nasm_negative_control() {
    let mut contexts = [
        RootContext {
            target: 1.0,
            ..Default::default()
        },
        RootContext {
            target: 2.0,
            ..Default::default()
        },
    ];
    let mut children: Vec<_> = contexts.iter_mut().map(|ctx| root(ctx, 0.0)).collect();
    let parent = object(pse_petsc_snes_create, pse_petsc_snes_destroy);
    let global = vector(&[0.0, 0.0]);
    let residual = vector(&[0.0, 0.0]);
    let options = object(pse_petsc_options_create, pse_petsc_options_destroy);
    let mut scatters = Vec::new();
    for (index, child) in children.iter().enumerate() {
        let mut scatter = ptr::null_mut();
        // SAFETY: index maps one global coordinate into the single-entry child layout.
        unsafe {
            assert_eq!(
                pse_petsc_scatter_create(
                    global.handle,
                    child.residual.handle,
                    1,
                    &(index as i32),
                    &0,
                    &mut scatter
                ),
                0
            );
        }
        scatters.push(Object {
            handle: scatter,
            destroy: pse_petsc_scatter_destroy,
        });
    }
    let scatter_handles: Vec<_> = scatters.iter().map(|object| object.handle).collect();
    let mut child_handles: Vec<_> = children.iter().map(|child| child.solver.handle).collect();
    // SAFETY: NASM consumes owned children; distinct retained scatter references remain valid.
    {
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_type(parent.handle, c"nasm".as_ptr()) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_options(parent.handle, options.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_layout(parent.handle, residual.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe {
                pse_petsc_snes_function(
                    parent.handle,
                    residual.handle,
                    block_residual,
                    ptr::null_mut(),
                )
            },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_tolerances(parent.handle, 1e-11, 1e-11, 1e-12, 10, 100) },
            0
        );
        assert_ne!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_tr_tolerances(parent.handle, 1e-12, 10.0, 1.0) },
            0,
            "inert trust-region setting must refuse NASM"
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_nasm_restrict(parent.handle, 1) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_nasm_damping(parent.handle, 1.0) },
            0
        );
        assert_eq!(
            // SAFETY: all child handles and scatter arrays match the declared count and remain live through parent teardown.
            unsafe {
                pse_petsc_nasm_subdomains(
                    parent.handle,
                    2,
                    child_handles.as_mut_ptr(),
                    scatter_handles.as_ptr(),
                    scatter_handles.as_ptr(),
                    scatter_handles.as_ptr(),
                )
            },
            0
        );
    }
    for (child, transferred) in children.iter_mut().zip(child_handles) {
        assert!(transferred.is_null());
        child.solver.handle = transferred;
    }
    drop(scatters); // Only NASM's retained references now keep the native scatters alive.
    // SAFETY: parent still owns subsolvers, callbacks and all native scatter references.
    {
        // Tagged 3.24.0 rejects supplied sub-SNES in SNESSetUp_NASM's else branch.
        assert_ne!(
            // SAFETY: the live solver retains its callbacks/context and the supplied solution vector has the admitted layout.
            unsafe { pse_petsc_snes_solve(parent.handle, global.handle) },
            0
        );
        let mut borrowed = ptr::null_mut();
        assert_eq!(
            // SAFETY: the live parent owns the borrowed child; the separate handle output is writable and never acquires ownership.
            unsafe { pse_petsc_nasm_subsolver(parent.handle, 0, &mut borrowed) },
            0
        );
        assert_ne!(
            // SAFETY: the bridge distinguishes borrowed child handles from owned handles and refuses their destruction.
            unsafe { pse_petsc_snes_destroy(&mut borrowed) },
            0
        );
    }
    assert_eq!(read::<2>(global.handle), [0.0, 0.0]);
    drop(parent); // Native subsolvers are destroyed before their callback contexts/buffers.
    drop(children);
    drop(options);
}

struct BlockContext {
    index: usize,
    calls: usize,
    frozen_witness: bool,
    terminal: bool,
}
unsafe extern "C" fn coupled_block(ctx: *mut c_void, x: Handle, ghost: Handle, f: Handle) -> i32 {
    callback(|| {
        // SAFETY: each block context remains pinned until the owning NASM is destroyed.
        let ctx = unsafe { &mut *ctx.cast::<BlockContext>() };
        let x = read::<1>(x)[0];
        let ghost = read::<2>(ghost);
        if (ghost[ctx.index] - x).abs() > 1e-12 {
            return EVAL_TERMINAL;
        }
        ctx.calls += 1;
        if ctx.terminal {
            return EVAL_TERMINAL;
        }
        ctx.frozen_witness |= ghost[1 - ctx.index] != 0.0;
        let target = if ctx.index == 0 { 1.4 } else { 2.2 };
        write(f, &[x + 0.2 * ghost[1 - ctx.index] - target])
    })
}
unsafe extern "C" fn coupled_jacobian(
    _: *mut c_void,
    _: Handle,
    _: Handle,
    a: Handle,
    p: Handle,
) -> i32 {
    callback(|| {
        let code = diagonal(a, &[1.0]);
        if code != 0 || a == p {
            code
        } else {
            diagonal(p, &[1.0])
        }
    })
}
unsafe extern "C" fn coupled_outer(_: *mut c_void, x: Handle, f: Handle) -> i32 {
    callback(|| {
        let x = read::<2>(x);
        write(f, &[x[0] + 0.2 * x[1] - 1.4, 0.2 * x[0] + x[1] - 2.2])
    })
}
fn automatic_nasm_control(failure: Option<usize>) {
    let mut contexts = [
        Box::new(BlockContext {
            index: 0,
            calls: 0,
            frozen_witness: false,
            terminal: failure == Some(0),
        }),
        Box::new(BlockContext {
            index: 1,
            calls: 0,
            frozen_witness: false,
            terminal: failure == Some(1),
        }),
    ];
    let options = [
        object(pse_petsc_options_create, pse_petsc_options_destroy),
        object(pse_petsc_options_create, pse_petsc_options_destroy),
    ];
    let parent_options = object(pse_petsc_options_create, pse_petsc_options_destroy);
    let global = vector(&[0.0, 0.0]);
    let residual = vector(&[0.0, 0.0]);
    let parent = object(pse_petsc_snes_create, pse_petsc_snes_destroy);
    let mut indices = [[0], [1]];
    let rows = [0];
    let ghost = [0, 1];
    let descriptors: Vec<_> = contexts
        .iter_mut()
        .enumerate()
        .map(|(i, ctx)| Block {
            size: 1,
            overlap: indices[i].as_ptr(),
            interior_size: 1,
            interior: indices[i].as_ptr(),
            ghost_size: 2,
            ghost: ghost.as_ptr(),
            nonzeros: 1,
            rows: rows.as_ptr(),
            columns: rows.as_ptr(),
            function: coupled_block,
            jacobian: coupled_jacobian,
            context: ptr::from_mut(ctx.as_mut()).cast(),
            options: options[i].handle,
        })
        .collect();
    // SAFETY: maps/pattern are valid; callbacks and private options live through teardown.
    {
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_type(parent.handle, c"nasm".as_ptr()) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_options(parent.handle, parent_options.handle) },
            0
        );
        assert_eq!(
            // SAFETY: the solver, layout objects and registered callback context remain live through solve and native teardown.
            unsafe {
                pse_petsc_snes_function(
                    parent.handle,
                    residual.handle,
                    coupled_outer,
                    ptr::null_mut(),
                )
            },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_snes_tolerances(parent.handle, 1e-11, 1e-11, 1e-14, 30, 300) },
            0
        );
        assert_eq!(
            // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
            unsafe { pse_petsc_nasm_restrict(parent.handle, 1) },
            0
        );
        assert_eq!(
            // SAFETY: the full vector, descriptors, copied index maps and callback contexts remain live through parent teardown.
            unsafe { pse_petsc_nasm_dm(parent.handle, global.handle, 2, descriptors.as_ptr()) },
            0
        );
        for i in 0..2 {
            let mut child = ptr::null_mut();
            assert_eq!(
                // SAFETY: the live parent owns the borrowed child; the separate handle output is writable and never acquires ownership.
                unsafe { pse_petsc_nasm_subsolver(parent.handle, i, &mut child) },
                0
            );
            assert_eq!(
                // SAFETY: the initialized process owns this live solver/options/vector; supplied configuration and any borrowed handles remain valid.
                unsafe { pse_petsc_snes_tolerances(child, 1e-12, 1e-12, 1e-14, 30, 100) },
                0
            );
            let mut name = ptr::null();
            assert_eq!(
                // SAFETY: the live solver owns the returned type string and the separate pointer output remains writable.
                unsafe { pse_petsc_snes_get_type(child, &mut name) },
                0
            );
            assert_eq!(
                // SAFETY: PETSc returned a NUL-terminated type string borrowed from this still-live native object.
                unsafe { CStr::from_ptr(name) },
                c"newtontr"
            );
            assert_ne!(
                // SAFETY: the bridge distinguishes borrowed child handles from owned handles and refuses their destruction.
                unsafe { pse_petsc_snes_destroy(&mut child) },
                0
            );
        }
    }
    // Input coordinate arrays were copied into the DM owner, never borrowed afterward.
    indices = [[-1], [-1]];
    assert_eq!(indices, [[-1], [-1]]);
    drop(descriptors);
    // SAFETY: composition retains its copied maps and library-owned child DMs/scatters.
    unsafe {
        assert_eq!(
            pse_petsc_snes_solve(parent.handle, global.handle),
            if failure.is_some() { 83 } else { 0 }
        );
    }
    if let Some(index) = failure {
        assert_eq!(contexts[index].calls, 1);
        if index == 0 {
            assert_eq!(contexts[1].calls, 0);
        }
        drop(parent); // Pending later scatters complete; ended current scatters are never ended twice.
        return;
    }
    let point = read::<2>(global.handle);
    assert!((point[0] - 1.0).abs() < 1e-9 && (point[1] - 2.0).abs() < 1e-9);
    assert!(stats(parent.handle).0 > 0);
    assert!(
        contexts
            .iter()
            .all(|ctx| ctx.calls > 0 && ctx.frozen_witness)
    );
    drop(parent); // Library children and DM callback owners die before Rust contexts/options.
}

#[test]
fn linked_profiles_preserve_phase_controls_and_native_ownership() {
    // One process lifecycle exercises initialization/finalization exactly once, avoiding
    // any test-only alternative to the native integration's central process owner.
    {
        let (mut major, mut minor, mut patch, mut integer, mut scalar) = (0, 0, 0, 0, 0);
        assert_eq!(
            // SAFETY: the version and ABI outputs are separate live writable integer slots; no native initialization is required.
            unsafe {
                pse_petsc_abi(
                    &mut major,
                    &mut minor,
                    &mut patch,
                    &mut integer,
                    &mut scalar,
                )
            },
            0
        );
        assert_eq!((major, minor, patch, integer, scalar), (3, 24, 0, 4, 8));
        assert_eq!(
            // SAFETY: this single serialized test owns the process lifecycle; repeated initialization is an explicit refused control.
            unsafe { pse_petsc_initialize() },
            0
        );
        assert_ne!(
            // SAFETY: this single serialized test owns the process lifecycle; repeated initialization is an explicit refused control.
            unsafe { pse_petsc_initialize() },
            0,
            "process initialization is not per-attempt"
        );
    }
    trust_region_controls();
    private_options_controls();
    pseudo_controls(false);
    pseudo_controls(true);
    manual_nasm_negative_control();
    automatic_nasm_control(None);
    automatic_nasm_control(Some(0));
    automatic_nasm_control(Some(1));
    // SAFETY: all helper-owned native objects were destroyed before process finalization.
    {
        assert_eq!(
            // SAFETY: all helper-owned native handles have been destroyed before terminal process finalization.
            unsafe { pse_petsc_finalize() },
            0
        );
        let (mut initialized, mut finalized) = (0, 0);
        assert_eq!(
            // SAFETY: the lifecycle query writes two separate live integer output slots.
            unsafe { pse_petsc_initialized(&mut initialized, &mut finalized) },
            0
        );
        assert_eq!((initialized, finalized), (0, 1));
        assert_ne!(
            // SAFETY: this single serialized test owns the process lifecycle; repeated initialization is an explicit refused control.
            unsafe { pse_petsc_initialize() },
            0,
            "a finalized process owner cannot be renewed"
        );
    }
}
