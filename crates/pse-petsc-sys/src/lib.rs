// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Narrow project ABI for PETSc 3.24.0, real double and serial MPIUNI.
//!
//! The native integration owner serializes all PETSc use and owns process initialization,
//! finalization, object lifetimes and panic containment at every Rust callback.
//!
//! Callbacks return [`EVAL_OK`], [`EVAL_DOMAIN`] to flag a native domain refusal,
//! whose recovery depends on the method and evaluation phase,
//! or [`EVAL_TERMINAL`] after latching their original cause. Initial points require
//! independent valid-domain admission. An options handle must outlive every object
//! to which it is attached; PETSc stores the pointer without retaining ownership.
//! Successful NASM registration consumes sub-SNES handles but only retains scatters.

#![allow(
    unsafe_code,
    reason = "raw declarations and tests of the pinned PETSc project C ABI"
)]
#![allow(
    missing_docs,
    reason = "raw project C ABI declarations documented in bridge/pse_petsc.h"
)]

use std::ffi::{c_char, c_void};

pub type Handle = *mut c_void;
#[cfg(feature = "link")]
pub const BUILD_ID: &str = env!("PSE_PETSC_BUILD_ID");
#[cfg(feature = "link")]
pub const VERSION: &str = env!("PSE_PETSC_VERSION");
pub const EVAL_OK: i32 = 0;
pub const EVAL_DOMAIN: i32 = 1;
pub const EVAL_TERMINAL: i32 = -1;
pub const SNES_FNORM_ABSOLUTE: i32 = 2;
pub const SNES_FNORM_RELATIVE: i32 = 3;
pub const SNES_STEP_RELATIVE: i32 = 4;
pub const SNES_FUNCTION_DOMAIN: i32 = -1;
pub const SNES_FUNCTION_COUNT: i32 = -2;
pub const SNES_MAX_ITERATIONS: i32 = -5;
pub const SNES_JACOBIAN_DOMAIN: i32 = -10;
pub const SNES_TR_DELTA: i32 = -11;
pub const SNES_USER_DIVERGED: i32 = -12;
pub const TS_USER_CONVERGED: i32 = 3;
pub const TS_STEP_LIMIT: i32 = 2;
pub const TS_NONLINEAR_FAILURES: i32 = -1;
pub const TS_REJECTIONS: i32 = -2;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorClass {
    Success = 0,
    Resource = 1,
    Contract = 2,
    Numerical = 3,
    Infrastructure = 4,
}

pub type Function = unsafe extern "C" fn(*mut c_void, Handle, Handle) -> i32;
pub type Jacobian = unsafe extern "C" fn(*mut c_void, Handle, Handle, Handle) -> i32;
pub type Convergence = unsafe extern "C" fn(*mut c_void, i32, f64, f64, f64, *mut i32) -> i32;
pub type IFunction = unsafe extern "C" fn(*mut c_void, f64, Handle, Handle, Handle) -> i32;
pub type IJacobian =
    unsafe extern "C" fn(*mut c_void, f64, Handle, Handle, f64, Handle, Handle) -> i32;
pub type Domain = unsafe extern "C" fn(*mut c_void, f64, Handle, *mut i32) -> i32;
pub type PostStep = unsafe extern "C" fn(*mut c_void, Handle) -> i32;
pub type PreStage = unsafe extern "C" fn(*mut c_void, Handle, f64) -> i32;
pub type BlockFunction = unsafe extern "C" fn(*mut c_void, Handle, Handle, Handle) -> i32;
pub type BlockJacobian = unsafe extern "C" fn(*mut c_void, Handle, Handle, Handle, Handle) -> i32;
#[repr(C)]
#[derive(Debug)]
pub struct Block {
    pub size: i32,
    pub overlap: *const i32,
    pub interior_size: i32,
    pub interior: *const i32,
    pub ghost_size: i32,
    pub ghost: *const i32,
    pub nonzeros: i32,
    pub rows: *const i32,
    pub columns: *const i32,
    pub function: BlockFunction,
    pub jacobian: BlockJacobian,
    pub context: *mut c_void,
    pub options: Handle,
}

unsafe extern "C" {
    pub fn pse_petsc_abi(
        major: *mut i32,
        minor: *mut i32,
        patch: *mut i32,
        integer_bytes: *mut i32,
        scalar_bytes: *mut i32,
    ) -> i32;
    pub fn pse_petsc_initialized(initialized: *mut i32, finalized: *mut i32) -> i32;
    pub fn pse_petsc_default_options_empty(empty: *mut i32) -> i32;
    pub fn pse_petsc_initialize() -> i32;
    pub fn pse_petsc_finalize() -> i32;
    pub fn pse_petsc_error_message(code: i32, message: *mut *const c_char) -> i32;
    pub fn pse_petsc_error_name(code: i32, name: *mut *const c_char) -> i32;
    pub fn pse_petsc_error_class(code: i32) -> ErrorClass;
    pub static PSE_PETSC_ERR_MEM: i32;
    pub static PSE_PETSC_ERR_ARG_SIZ: i32;
    pub static PSE_PETSC_ERR_ORDER: i32;
    pub static PSE_PETSC_ERR_SYS: i32;
    pub static PSE_PETSC_ERR_LIB: i32;
    pub static PSE_PETSC_ERR_PLIB: i32;
    pub static PSE_PETSC_ERR_MEMC: i32;
    pub static PSE_PETSC_ERR_FP: i32;
    pub static PSE_PETSC_ERR_NOT_CONVERGED: i32;
    pub static PSE_PETSC_ERR_USER: i32;
    pub fn pse_petsc_options_create(out: *mut Handle) -> i32;
    pub fn pse_petsc_options_set(options: Handle, key: *const c_char, value: *const c_char) -> i32;
    pub fn pse_petsc_options_destroy(options: *mut Handle) -> i32;
    pub fn pse_petsc_vec_create(n: i32, out: *mut Handle) -> i32;
    pub fn pse_petsc_vec_destroy(vector: *mut Handle) -> i32;
    pub fn pse_petsc_vec_read(vector: Handle, n: i32, values: *mut f64) -> i32;
    pub fn pse_petsc_vec_write(vector: Handle, n: i32, values: *const f64) -> i32;
    pub fn pse_petsc_mat_create(
        rows: i32,
        columns: i32,
        row_capacity: *const i32,
        out: *mut Handle,
    ) -> i32;
    pub fn pse_petsc_mat_destroy(matrix: *mut Handle) -> i32;
    pub fn pse_petsc_mat_write(
        matrix: Handle,
        n: i32,
        rows: *const i32,
        columns: *const i32,
        values: *const f64,
    ) -> i32;
    pub fn pse_petsc_scatter_create(
        global: Handle,
        local: Handle,
        n: i32,
        global_indices: *const i32,
        local_indices: *const i32,
        out: *mut Handle,
    ) -> i32;
    pub fn pse_petsc_scatter_destroy(scatter: *mut Handle) -> i32;
    pub fn pse_petsc_snes_create(out: *mut Handle) -> i32;
    pub fn pse_petsc_snes_destroy(snes: *mut Handle) -> i32;
    pub fn pse_petsc_snes_type(snes: Handle, name: *const c_char) -> i32;
    pub fn pse_petsc_snes_get_type(snes: Handle, name: *mut *const c_char) -> i32;
    pub fn pse_petsc_snes_options(snes: Handle, options: Handle) -> i32;
    pub fn pse_petsc_snes_from_options(snes: Handle) -> i32;
    pub fn pse_petsc_snes_function(
        snes: Handle,
        residual: Handle,
        callback: Function,
        ctx: *mut c_void,
    ) -> i32;
    pub fn pse_petsc_snes_jacobian(
        snes: Handle,
        jacobian: Handle,
        preconditioner: Handle,
        callback: Jacobian,
        ctx: *mut c_void,
    ) -> i32;
    pub fn pse_petsc_snes_convergence(snes: Handle, callback: Convergence, ctx: *mut c_void)
    -> i32;
    pub fn pse_petsc_snes_default_convergence(
        snes: Handle,
        iteration: i32,
        xnorm: f64,
        ynorm: f64,
        fnorm: f64,
        reason: *mut i32,
    ) -> i32;
    pub fn pse_petsc_snes_tolerances(
        snes: Handle,
        absolute: f64,
        relative: f64,
        step: f64,
        iterations: i32,
        evaluations: i32,
    ) -> i32;
    pub fn pse_petsc_snes_error_if_not_converged(snes: Handle, enabled: i32) -> i32;
    pub fn pse_petsc_snes_tr_tolerances(
        snes: Handle,
        minimum: f64,
        maximum: f64,
        initial: f64,
    ) -> i32;
    pub fn pse_petsc_snes_tr_get_tolerances(
        snes: Handle,
        minimum: *mut f64,
        maximum: *mut f64,
        initial: *mut f64,
    ) -> i32;
    pub fn pse_petsc_snes_tr_update(
        snes: Handle,
        eta1: f64,
        eta2: f64,
        eta3: f64,
        shrink: f64,
        grow: f64,
    ) -> i32;
    pub fn pse_petsc_snes_linear(snes: Handle, ksp: *const c_char, pc: *const c_char) -> i32;
    pub fn pse_petsc_snes_layout(snes: Handle, layout: Handle) -> i32;
    pub fn pse_petsc_snes_solve(snes: Handle, x: Handle) -> i32;
    pub fn pse_petsc_snes_statistics(
        snes: Handle,
        reason: *mut i32,
        iterations: *mut i32,
        evaluations: *mut i32,
        linear_iterations: *mut i32,
        norm: *mut f64,
    ) -> i32;
    pub fn pse_petsc_snes_reason_name(snes: Handle, name: *mut *const c_char) -> i32;
    pub fn pse_petsc_nasm_subdomains(
        snes: Handle,
        n: i32,
        children: *mut Handle,
        interior: *const Handle,
        overlap: *const Handle,
        ghost: *const Handle,
    ) -> i32;
    pub fn pse_petsc_nasm_subsolver(snes: Handle, index: i32, out: *mut Handle) -> i32;
    pub fn pse_petsc_nasm_restrict(snes: Handle, restricted: i32) -> i32;
    pub fn pse_petsc_nasm_damping(snes: Handle, damping: f64) -> i32;
    pub fn pse_petsc_nasm_dm(snes: Handle, global: Handle, n: i32, blocks: *const Block) -> i32;
    pub fn pse_petsc_ts_create(out: *mut Handle) -> i32;
    pub fn pse_petsc_ts_destroy(ts: *mut Handle) -> i32;
    pub fn pse_petsc_ts_pseudo(ts: Handle) -> i32;
    pub fn pse_petsc_ts_get_type(ts: Handle, name: *mut *const c_char) -> i32;
    pub fn pse_petsc_ts_snes(ts: Handle, out: *mut Handle) -> i32;
    pub fn pse_petsc_ts_options(ts: Handle, options: Handle) -> i32;
    pub fn pse_petsc_ts_from_options(ts: Handle) -> i32;
    pub fn pse_petsc_ts_ifunction(
        ts: Handle,
        residual: Handle,
        callback: IFunction,
        ctx: *mut c_void,
    ) -> i32;
    pub fn pse_petsc_ts_ijacobian(
        ts: Handle,
        jacobian: Handle,
        preconditioner: Handle,
        callback: IJacobian,
        ctx: *mut c_void,
    ) -> i32;
    pub fn pse_petsc_ts_domain(ts: Handle, callback: Domain, ctx: *mut c_void) -> i32;
    pub fn pse_petsc_ts_post_step(ts: Handle, callback: PostStep, ctx: *mut c_void) -> i32;
    pub fn pse_petsc_ts_pre_stage(ts: Handle, callback: PreStage, ctx: *mut c_void) -> i32;
    pub fn pse_petsc_ts_solution(ts: Handle, x: Handle) -> i32;
    pub fn pse_petsc_ts_time(ts: Handle, start: f64, dt: f64, end: f64, steps: i32) -> i32;
    pub fn pse_petsc_ts_failures(ts: Handle, nonlinear_failures: i32, rejections: i32) -> i32;
    pub fn pse_petsc_ts_adapt(ts: Handle, minimum: f64, maximum: f64, failed_scale: f64) -> i32;
    pub fn pse_petsc_ts_pseudo_growth(ts: Handle, increment: f64, maximum: f64) -> i32;
    pub fn pse_petsc_ts_reason(ts: Handle, reason: i32) -> i32;
    pub fn pse_petsc_ts_solve(ts: Handle, x: Handle) -> i32;
    pub fn pse_petsc_ts_statistics(
        ts: Handle,
        reason: *mut i32,
        steps: *mut i32,
        nonlinear_iterations: *mut i32,
        rejections: *mut i32,
        failures: *mut i32,
        time: *mut f64,
        dt: *mut f64,
    ) -> i32;
    pub fn pse_petsc_ts_reason_name(ts: Handle, name: *mut *const c_char) -> i32;
}

#[cfg(all(test, feature = "link"))]
mod tests;
