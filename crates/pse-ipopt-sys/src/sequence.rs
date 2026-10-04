// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Project-owned C++ TNLP sequence ABI. A handle retains the same TNLP and
//! application; reuse calls `ReOptimizeTNLP` rather than reconstructing a solve.
//! The safe native owner admits full compatibility and owns typed callback causes.
//! Every callback must catch Rust panics before returning. Raw status codes here
//! distinguish foreign failure from Ipopt termination; they do not qualify a solve.

use std::ffi::{c_char, c_void};
/// Identity of the compiled persistent TNLP bridge and linked library version.
#[cfg(feature = "link")]
pub const BUILD_ID: &str = env!("PSE_IPOPT_SEQUENCE_BUILD_ID");
pub const OK: i32 = 0;
pub const INVALID: i32 = 1;
pub const EXCEPTION: i32 = 2;
pub const MEMORY: i32 = 3;
pub const TERMINAL: i32 = 4;
pub const CANCEL: i32 = 5;
pub const DEADLINE: i32 = 6;
pub const ABANDON: i32 = 7;
pub const NO_RESULT: i32 = 8;
#[repr(C)]
#[derive(Debug)]
pub struct Handle {
    _private: [u8; 0],
}
pub type Objective = unsafe extern "C" fn(i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Gradient = unsafe extern "C" fn(i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Constraints = unsafe extern "C" fn(i32, i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Jacobian = unsafe extern "C" fn(i32, i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Hessian =
    unsafe extern "C" fn(i32, i32, i32, *const f64, f64, *const f64, *mut f64, *mut c_void) -> i32;
pub type Iteration = unsafe extern "C" fn(
    i32,
    i32,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    i32,
    i32,
    *const f64,
    *const f64,
    *mut c_void,
) -> i32;
pub type Poll = unsafe extern "C" fn(*mut c_void) -> i32;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Callbacks {
    pub objective: Option<Objective>,
    pub gradient: Option<Gradient>,
    pub constraints: Option<Constraints>,
    pub jacobian: Option<Jacobian>,
    pub hessian: Option<Hessian>,
    pub poll: Option<Poll>,
    pub iteration: Option<Iteration>,
    pub user: *mut c_void,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Result {
    pub application_status: i32,
    pub solver_status: i32,
    pub iterations: i32,
    pub reoptimized: i32,
    pub iterations_available: i32,
    pub barrier_available: i32,
    pub candidate_available: i32,
    pub objective: f64,
    pub barrier: f64,
}
unsafe extern "C" {
    pub fn pse_ipopt_sequence_create(
        n: i32,
        m: i32,
        jac_nnz: i32,
        hess_nnz: i32,
        lower: *const f64,
        upper: *const f64,
        row_lower: *const f64,
        row_upper: *const f64,
        jac_rows: *const i32,
        jac_columns: *const i32,
        hess_rows: *const i32,
        hess_columns: *const i32,
        callbacks: *const Callbacks,
        out: *mut *mut Handle,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_set_string(
        handle: *mut Handle,
        key: *const c_char,
        value: *const c_char,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_set_double(
        handle: *mut Handle,
        key: *const c_char,
        value: f64,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_set_integer(
        handle: *mut Handle,
        key: *const c_char,
        value: i32,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_reset_restart(
        handle: *mut Handle,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_solve(
        handle: *mut Handle,
        reoptimize: i32,
        remaining_seconds: f64,
        n: i32,
        primal: *const f64,
        lower_dual: *const f64,
        upper_dual: *const f64,
        m: i32,
        row_dual: *const f64,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_get_result(
        handle: *mut Handle,
        result: *mut Result,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_get_vectors(
        handle: *mut Handle,
        n: i32,
        primal: *mut f64,
        lower_dual: *mut f64,
        upper_dual: *mut f64,
        m: i32,
        constraints: *mut f64,
        row_dual: *mut f64,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_ipopt_sequence_destroy(
        handle: *mut *mut Handle,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
}

#[cfg(all(test, feature = "link"))]
mod tests {
    use super::*;
    use std::{cell::Cell, ptr};
    struct Fixture {
        target: Cell<f64>,
        terminal: bool,
        calls: usize,
    }
    unsafe extern "C" fn objective(_: i32, x: *const f64, out: *mut f64, user: *mut c_void) -> i32 {
        // SAFETY: registered Fixture is exclusively callback-owned and remains live until handle destruction.
        let fixture = unsafe { &mut *user.cast::<Fixture>() };
        fixture.calls += 1;
        if fixture.terminal {
            return 2;
        }
        // SAFETY: the configured one-variable callback receives one readable coordinate.
        let coordinate = unsafe { *x };
        // SAFETY: the bridge supplies one writable objective output.
        unsafe {
            *out = (coordinate - fixture.target.get()).powi(2);
        }
        0
    }
    unsafe extern "C" fn gradient(_: i32, x: *const f64, out: *mut f64, user: *mut c_void) -> i32 {
        // SAFETY: registered Fixture is exclusively callback-owned and remains live until handle destruction.
        let fixture = unsafe { &*user.cast::<Fixture>() };
        // SAFETY: the configured one-variable callback receives one readable coordinate.
        let coordinate = unsafe { *x };
        // SAFETY: the bridge supplies one writable gradient coordinate.
        unsafe {
            *out = 2.0 * (coordinate - fixture.target.get());
        }
        0
    }
    unsafe extern "C" fn hessian(
        _: i32,
        _: i32,
        _: i32,
        _: *const f64,
        factor: f64,
        _: *const f64,
        out: *mut f64,
        _: *mut c_void,
    ) -> i32 {
        // SAFETY: the bridge supplies the configured single writable Hessian output.
        unsafe {
            *out = 2.0 * factor;
        }
        0
    }
    unsafe extern "C" fn poll(_: *mut c_void) -> i32 {
        0
    }
    fn create(fixture: &mut Fixture) -> *mut Handle {
        let callbacks = Callbacks {
            objective: Some(objective),
            gradient: Some(gradient),
            constraints: None,
            jacobian: None,
            hessian: Some(hessian),
            poll: Some(poll),
            iteration: None,
            user: ptr::from_mut(fixture).cast(),
        };
        let mut handle = ptr::null_mut();
        assert_eq!(
            // SAFETY: all declared one-variable bounds/pattern buffers are live; callbacks retain the Fixture until destruction.
            unsafe {
                pse_ipopt_sequence_create(
                    1,
                    0,
                    0,
                    1,
                    &-2.0,
                    &2.0,
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    &0,
                    &0,
                    &callbacks,
                    &mut handle,
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert_eq!(
            // SAFETY: the owned bridge handle and NUL-terminated option strings are live through this call.
            unsafe {
                pse_ipopt_sequence_set_integer(
                    handle,
                    c"print_level".as_ptr(),
                    0,
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert_eq!(
            // SAFETY: the owned bridge handle and NUL-terminated option strings are live through this call.
            unsafe {
                pse_ipopt_sequence_set_integer(handle, c"max_iter".as_ptr(), 30, ptr::null_mut(), 0)
            },
            OK
        );
        assert_eq!(
            // SAFETY: the owned bridge handle and NUL-terminated option strings are live through this call.
            unsafe {
                pse_ipopt_sequence_set_string(
                    handle,
                    c"nlp_scaling_method".as_ptr(),
                    c"none".as_ptr(),
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        handle
    }
    #[test]
    fn same_tnlp_sequence_uses_reoptimize_and_refuses_profile_mutation() {
        let mut fixture = Fixture {
            target: Cell::new(0.5),
            terminal: false,
            calls: 0,
        };
        let mut handle = create(&mut fixture);
        assert_eq!(
            // SAFETY: the owned handle retains its live Fixture; one primal coordinate and declared zero-row buffers match its shape.
            unsafe {
                pse_ipopt_sequence_solve(
                    handle,
                    0,
                    5.0,
                    1,
                    &0.0,
                    ptr::null(),
                    ptr::null(),
                    0,
                    ptr::null(),
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        let mut result = Result::default();
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_ipopt_sequence_get_result(handle, &mut result, ptr::null_mut(), 0) },
            OK
        );
        assert_eq!(result.application_status, 0);
        assert_eq!(result.reoptimized, 0);
        assert_eq!(
            // SAFETY: the owned bridge handle and NUL-terminated option strings are live through this call.
            unsafe {
                pse_ipopt_sequence_set_integer(handle, c"max_iter".as_ptr(), 40, ptr::null_mut(), 0)
            },
            INVALID
        );
        fixture.target.set(1.0);
        assert_eq!(
            // SAFETY: the owned handle retains its live Fixture; one primal coordinate and declared zero-row buffers match its shape.
            unsafe {
                pse_ipopt_sequence_solve(
                    handle,
                    1,
                    5.0,
                    1,
                    &0.5,
                    ptr::null(),
                    ptr::null(),
                    0,
                    ptr::null(),
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_ipopt_sequence_get_result(handle, &mut result, ptr::null_mut(), 0) },
            OK
        );
        assert_eq!(result.application_status, 0, "{result:?}");
        assert_eq!(result.reoptimized, 1);
        assert!(result.objective < 1e-12);
        let mut primal = 0.;
        let mut lower = 0.;
        let mut upper = 0.;
        assert_eq!(
            // SAFETY: the owned handle and separate one-variable output buffers are live; zero row counts permit null row buffers.
            unsafe {
                pse_ipopt_sequence_get_vectors(
                    handle,
                    1,
                    &mut primal,
                    &mut lower,
                    &mut upper,
                    0,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert!(
            (primal - 1.).abs() < 1e-6,
            "changed borrowed numerical target was not consumed: {primal}"
        );
        assert_eq!(
            // SAFETY: this output slot owns the live bridge handle and releases it exactly once.
            unsafe { pse_ipopt_sequence_destroy(&mut handle, ptr::null_mut(), 0) },
            OK
        );
        assert!(handle.is_null());
    }
    #[test]
    fn terminal_sequence_callback_is_latched_and_not_reentered() {
        let mut fixture = Fixture {
            target: Cell::new(0.5),
            terminal: true,
            calls: 0,
        };
        let mut handle = create(&mut fixture);
        assert_eq!(
            // SAFETY: the owned handle retains its live Fixture; one primal coordinate and declared zero-row buffers match its shape.
            unsafe {
                pse_ipopt_sequence_solve(
                    handle,
                    0,
                    5.0,
                    1,
                    &0.0,
                    ptr::null(),
                    ptr::null(),
                    0,
                    ptr::null(),
                    ptr::null_mut(),
                    0,
                )
            },
            TERMINAL
        );
        assert_eq!(fixture.calls, 1);
        let mut result = Result::default();
        let status =
            // SAFETY: this output slot owns the live bridge handle and releases it exactly once.
            unsafe { pse_ipopt_sequence_get_result(handle, &mut result, ptr::null_mut(), 0) };
        assert!(status == NO_RESULT || status == OK);
        if status == OK {
            assert_eq!(result.candidate_available, 0);
        }
        assert_eq!(
            // SAFETY: this output slot owns the live bridge handle and releases it exactly once.
            unsafe { pse_ipopt_sequence_destroy(&mut handle, ptr::null_mut(), 0) },
            OK
        );
    }
}
