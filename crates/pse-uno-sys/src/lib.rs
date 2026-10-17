// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Raw bindings to the project-owned, exception-shielded Uno bridge.
//!
//! The bridge pins the 32-bit Uno integer ABI and admits HiGHS-backed filter
//! trust-region SQP with LBFGS, or SLP with a zero Hessian. It never exposes an
//! unshielded Uno entrypoint. Callbacks must catch Rust unwinds and retain typed
//! original failures in their context; foreign status codes do not classify them.
//! The caller owns exclusive handle access, callback lifetime and scope admission.

#![allow(
    unsafe_code,
    reason = "raw declarations for the exception-shielded Uno C bridge"
)]
#![allow(
    missing_docs,
    non_camel_case_types,
    reason = "project-owned C ABI declarations"
)]
#![cfg_attr(feature = "link", feature(native_link_modifiers_as_needed))]

// The existing MKL provider requires all LP64/GNU components retained together.
// Record these as transitive native libraries, including in sys-only tests.
#[cfg(feature = "link")]
#[link(name = "mkl_intel_lp64", kind = "dylib", modifiers = "-as-needed")]
unsafe extern "C" {}

#[cfg(feature = "link")]
#[link(name = "mkl_gnu_thread", kind = "dylib", modifiers = "-as-needed")]
unsafe extern "C" {}

#[cfg(feature = "link")]
#[link(name = "mkl_core", kind = "dylib", modifiers = "-as-needed")]
unsafe extern "C" {}

#[cfg(feature = "link")]
#[link(name = "gomp", kind = "dylib", modifiers = "-as-needed")]
unsafe extern "C" {}

#[cfg(feature = "highs-provider")]
use highs_sys as _;

use std::ffi::{c_char, c_void};

#[cfg(feature = "link")]
pub const BUILD_ID: &str = env!("PSE_UNO_BUILD_ID");
#[cfg(feature = "link")]
pub const VERSION: &str = env!("PSE_UNO_VERSION");

pub const OK: i32 = 0;
pub const INVALID: i32 = 1;
pub const EXCEPTION: i32 = 2;
pub const MEMORY: i32 = 3;
pub const CALLBACK_TERMINAL: i32 = 4;
pub const CANCELLED: i32 = 5;
pub const DEADLINE: i32 = 6;
pub const ABANDONED: i32 = 7;
pub const NO_RESULT: i32 = 8;
pub const SQP_LBFGS: i32 = 0;
pub const SLP: i32 = 1;
pub const EVAL_OK: i32 = 0;
pub const EVAL_TRIAL: i32 = 1;
pub const EVAL_TERMINAL: i32 = 2;
pub const SCOPE_CONTINUE: i32 = 0;
pub const SCOPE_CANCEL: i32 = 1;
pub const SCOPE_DEADLINE: i32 = 2;
pub const SCOPE_ABANDON: i32 = 3;
pub const SCOPE_TERMINAL: i32 = 4;

#[repr(C)]
#[derive(Debug)]
pub struct Handle {
    _private: [u8; 0],
}
pub type Objective = unsafe extern "C" fn(i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Constraints = unsafe extern "C" fn(i32, i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Gradient = unsafe extern "C" fn(i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Jacobian = unsafe extern "C" fn(i32, i32, *const f64, *mut f64, *mut c_void) -> i32;
pub type Poll = unsafe extern "C" fn(*mut c_void) -> i32;
pub type Observe =
    unsafe extern "C" fn(i32, u64, i32, i32, *const f64, f64, f64, f64, f64, *mut c_void);
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Callbacks {
    pub objective: Option<Objective>,
    pub constraints: Option<Constraints>,
    pub gradient: Option<Gradient>,
    pub jacobian: Option<Jacobian>,
    pub poll: Option<Poll>,
    pub observe: Option<Observe>,
    pub user: *mut c_void,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub profile: i32,
    pub max_iterations: i32,
    pub lbfgs_memory: i32,
    pub remaining_seconds: f64,
    pub primal_tolerance: f64,
    pub dual_tolerance: f64,
    pub trust_radius: f64,
    pub materialization_limit_bytes: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Result {
    pub optimization_status: i32,
    pub iterate_status: i32,
    pub iterations: i32,
    pub objective_evaluations: i32,
    pub constraint_evaluations: i32,
    pub gradient_evaluations: i32,
    pub jacobian_evaluations: i32,
    pub subproblems: i32,
    pub objective: f64,
    pub primal_feasibility: f64,
    pub stationarity: f64,
    pub complementarity: f64,
    pub cpu_seconds: f64,
    pub materialization_count: u64,
    pub materialization_columns: u64,
    pub materialization_numeric_bytes: u64,
    pub highs_primal_tolerance: f64,
    pub highs_dual_tolerance: f64,
    pub highs_optimality_tolerance: f64,
    pub highs_qp_regularization: f64,
    pub feasibility_calls: u64,
    pub feasibility_iterations: u64,
    pub feasibility_status: i32,
    pub feasibility_status_available: i32,
    pub feasibility_iterations_available: i32,
    pub qp_hot_start: i32,
    pub feasibility_primal_tolerance: f64,
}
unsafe extern "C" {
    pub fn pse_uno_notify_waiters();
    pub fn pse_uno_minimum_tolerance() -> f64;
    pub fn pse_uno_create(
        n: i32,
        m: i32,
        nnz: i32,
        lower: *const f64,
        upper: *const f64,
        row_lower: *const f64,
        row_upper: *const f64,
        rows: *const i32,
        columns: *const i32,
        config: *const Config,
        callbacks: *const Callbacks,
        out: *mut *mut Handle,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_solve(
        handle: *mut Handle,
        initial: *const f64,
        n: i32,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_result(
        handle: *mut Handle,
        result: *mut Result,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_vectors(
        handle: *mut Handle,
        primal: *mut f64,
        lower_dual: *mut f64,
        upper_dual: *mut f64,
        n: i32,
        constraints: *mut f64,
        row_dual: *mut f64,
        m: i32,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_string_option(
        handle: *mut Handle,
        key: *const c_char,
        value: *mut c_char,
        capacity: usize,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_double_option(
        handle: *mut Handle,
        key: *const c_char,
        value: *mut f64,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_bool_option(
        handle: *mut Handle,
        key: *const c_char,
        value: *mut i32,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_method(
        handle: *mut Handle,
        value: *mut c_char,
        capacity: usize,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_get_integer_option(
        handle: *mut Handle,
        key: *const c_char,
        value: *mut i32,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
    pub fn pse_uno_destroy(
        handle: *mut *mut Handle,
        error: *mut c_char,
        error_capacity: usize,
    ) -> i32;
}

#[cfg(all(test, feature = "link"))]
mod tests {
    use super::*;
    use std::ptr;
    struct Fixture {
        evaluations: usize,
        terminal: bool,
        scope: i32,
        trial_once: bool,
        rejected: usize,
    }
    unsafe extern "C" fn objective(
        _: i32,
        x: *const f64,
        value: *mut f64,
        user: *mut c_void,
    ) -> i32 {
        // SAFETY: registered Fixture is exclusively callback-owned and remains live until handle destruction.
        let fixture = unsafe { &mut *user.cast::<Fixture>() };
        fixture.evaluations += 1;
        if fixture.terminal {
            return EVAL_TERMINAL;
        }
        // SAFETY: the configured one-variable callback receives one readable coordinate.
        let coordinate = unsafe { *x };
        if fixture.trial_once && coordinate != 0.0 {
            fixture.trial_once = false;
            fixture.rejected += 1;
            return EVAL_TRIAL;
        }
        // SAFETY: the bridge supplies one writable objective output for this callback.
        unsafe {
            *value = (coordinate - 1.0).powi(2);
        }
        EVAL_OK
    }
    unsafe extern "C" fn gradient(_: i32, x: *const f64, value: *mut f64, _: *mut c_void) -> i32 {
        // SAFETY: the configured one-variable callback receives one readable coordinate.
        let coordinate = unsafe { *x };
        // SAFETY: the bridge supplies one writable gradient coordinate.
        unsafe {
            *value = 2.0 * (coordinate - 1.0);
        }
        EVAL_OK
    }
    unsafe extern "C" fn poll(user: *mut c_void) -> i32 {
        // SAFETY: registered Fixture is exclusively callback-owned and remains live until handle destruction.
        unsafe { (*user.cast::<Fixture>()).scope }
    }
    fn create(fixture: &mut Fixture, profile: i32) -> *mut Handle {
        create_with_limit(fixture, profile, 8 << 20)
    }
    fn create_with_limit(fixture: &mut Fixture, profile: i32, limit: u64) -> *mut Handle {
        let config = Config {
            profile,
            max_iterations: 30,
            lbfgs_memory: if profile == SQP_LBFGS { 4 } else { 0 },
            remaining_seconds: 5.0,
            primal_tolerance: 1e-8,
            dual_tolerance: 1e-8,
            trust_radius: 1.0,
            materialization_limit_bytes: limit,
        };
        let callbacks = Callbacks {
            objective: Some(objective),
            constraints: None,
            gradient: Some(gradient),
            jacobian: None,
            poll: Some(poll),
            observe: None,
            user: ptr::from_mut(fixture).cast(),
        };
        let mut handle = ptr::null_mut();
        // SAFETY: all declared one-variable bounds/pattern buffers are live; callbacks retain the Fixture until destruction.
        let status = unsafe {
            pse_uno_create(
                1,
                0,
                0,
                &-2.0,
                &2.0,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                &config,
                &callbacks,
                &mut handle,
                ptr::null_mut(),
                0,
            )
        };
        assert_eq!(status, OK);
        assert!(!handle.is_null());
        handle
    }
    fn destroy(handle: &mut *mut Handle) {
        // SAFETY: this output slot owns the live bridge handle and releases it exactly once.
        assert_eq!(unsafe { pse_uno_destroy(handle, ptr::null_mut(), 0) }, OK);
        assert!(handle.is_null());
    }
    struct GateFixture {
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
        evaluations: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        entered: Option<std::sync::mpsc::Sender<()>>,
        release: Option<std::sync::mpsc::Receiver<()>>,
        polled: Option<std::sync::mpsc::Sender<()>>,
    }
    unsafe extern "C" fn gate_objective(
        _: i32,
        x: *const f64,
        out: *mut f64,
        data: *mut c_void,
    ) -> i32 {
        // SAFETY: one live exclusively callback-owned fixture and scalar extents.
        let fixture = unsafe { &mut *data.cast::<GateFixture>() };
        fixture
            .evaluations
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        if let Some(entered) = fixture.entered.take() {
            if entered.send(()).is_err() {
                return EVAL_TERMINAL;
            }
            if fixture.release.as_ref().is_none_or(|release| {
                release
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .is_err()
            }) {
                return EVAL_TERMINAL;
            }
        }
        // SAFETY: bridge provides one readable/writable scalar for this configured model.
        unsafe {
            *out = (*x - 1.).powi(2);
        }
        EVAL_OK
    }
    unsafe extern "C" fn gate_poll(data: *mut c_void) -> i32 {
        // SAFETY: fixture remains live and exclusive throughout synchronous solve.
        let fixture = unsafe { &mut *data.cast::<GateFixture>() };
        if let Some(polled) = fixture.polled.take() {
            let _ = polled.send(());
        }
        if fixture.cancel.load(std::sync::atomic::Ordering::Acquire) {
            SCOPE_CANCEL
        } else {
            SCOPE_CONTINUE
        }
    }
    fn gate_solve(mut fixture: GateFixture, seconds: f64) -> i32 {
        let config = Config {
            profile: SQP_LBFGS,
            max_iterations: 30,
            lbfgs_memory: 4,
            remaining_seconds: seconds,
            primal_tolerance: 1e-8,
            dual_tolerance: 1e-8,
            trust_radius: 1.,
            materialization_limit_bytes: 8 << 20,
        };
        let callbacks = Callbacks {
            objective: Some(gate_objective),
            constraints: None,
            gradient: Some(gradient),
            jacobian: None,
            poll: Some(gate_poll),
            observe: None,
            user: ptr::from_mut(&mut fixture).cast(),
        };
        let mut handle = ptr::null_mut();
        // SAFETY: all exact one-variable extents and callback fixture remain live until destruction.
        let created = unsafe {
            pse_uno_create(
                1,
                0,
                0,
                &-2.,
                &2.,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                &config,
                &callbacks,
                &mut handle,
                ptr::null_mut(),
                0,
            )
        };
        assert_eq!(created, OK);
        // SAFETY: exclusive live handle, initialized start and callback fixture.
        let result = unsafe { pse_uno_solve(handle, &0., 1, ptr::null_mut(), 0) };
        destroy(&mut handle);
        result
    }
    #[test]
    fn global_permit_wait_observes_signal_cancellation_and_absolute_deadline() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc,
        };
        for stop in [SCOPE_CANCEL, SCOPE_DEADLINE, SCOPE_CONTINUE] {
            let (entered, owner_entered) = mpsc::channel();
            let (release, owner_release) = mpsc::channel();
            let owner = std::thread::spawn(move || {
                gate_solve(
                    GateFixture {
                        cancel: Arc::new(AtomicBool::new(false)),
                        evaluations: Arc::new(AtomicUsize::new(0)),
                        entered: Some(entered),
                        release: Some(owner_release),
                        polled: None,
                    },
                    5.,
                )
            });
            owner_entered
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            let evaluations = Arc::new(AtomicUsize::new(0));
            let (polled, waiter_polled) = mpsc::channel();
            let waiter = {
                let cancel = cancel.clone();
                let evaluations = evaluations.clone();
                std::thread::spawn(move || {
                    gate_solve(
                        GateFixture {
                            cancel,
                            evaluations,
                            entered: None,
                            release: None,
                            polled: Some(polled),
                        },
                        if stop == SCOPE_DEADLINE { 0.05 } else { 5. },
                    )
                })
            };
            waiter_polled
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            if stop == SCOPE_CANCEL {
                cancel.store(true, Ordering::Release);
                // SAFETY: process-global signal owns no handle or fixture pointer.
                unsafe { pse_uno_notify_waiters() };
            }
            if stop == SCOPE_CONTINUE {
                release.send(()).unwrap();
            }
            let result = waiter.join().unwrap();
            assert_eq!(
                result,
                match stop {
                    SCOPE_CANCEL => CANCELLED,
                    SCOPE_DEADLINE => DEADLINE,
                    _ => OK,
                }
            );
            if stop != SCOPE_CONTINUE {
                assert_eq!(evaluations.load(Ordering::Acquire), 0);
                release.send(()).unwrap();
            } else {
                assert!(evaluations.load(Ordering::Acquire) > 0);
            }
            assert_eq!(owner.join().unwrap(), OK);
        }
        // A late safe notification never borrows a destroyed native handle.
        unsafe { pse_uno_notify_waiters() };
    }
    #[test]
    fn unsupported_accuracy_refuses_before_native_start() {
        for (primal, dual) in [(1e-11, 1e-8), (1e-8, 1e-11)] {
            let mut fixture = Fixture {
                evaluations: 0,
                terminal: false,
                scope: SCOPE_CONTINUE,
                trial_once: false,
                rejected: 0,
            };
            let config = Config {
                profile: SQP_LBFGS,
                max_iterations: 30,
                lbfgs_memory: 4,
                remaining_seconds: 5.,
                primal_tolerance: primal,
                dual_tolerance: dual,
                trust_radius: 1.,
                materialization_limit_bytes: 8 << 20,
            };
            let callbacks = Callbacks {
                objective: Some(objective),
                constraints: None,
                gradient: Some(gradient),
                jacobian: None,
                poll: Some(poll),
                observe: None,
                user: ptr::from_mut(&mut fixture).cast(),
            };
            let mut handle = ptr::null_mut();
            assert_eq!(
                // SAFETY: all declared one-variable bounds/pattern buffers are live; callbacks retain the Fixture until destruction.
                unsafe {
                    pse_uno_create(
                        1,
                        0,
                        0,
                        &-2.,
                        &2.,
                        ptr::null(),
                        ptr::null(),
                        ptr::null(),
                        ptr::null(),
                        &config,
                        &callbacks,
                        &mut handle,
                        ptr::null_mut(),
                        0,
                    )
                },
                INVALID
            );
            assert!(handle.is_null());
            assert_eq!(fixture.evaluations, 0);
        }
    }
    #[test]
    fn terminal_callback_stops_without_evaluator_reentry() {
        let mut fixture = Fixture {
            evaluations: 0,
            terminal: true,
            scope: SCOPE_CONTINUE,
            trial_once: false,
            rejected: 0,
        };
        let mut handle = create(&mut fixture, SQP_LBFGS);
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_uno_solve(handle, &0.0, 1, ptr::null_mut(), 0) },
            CALLBACK_TERMINAL
        );
        assert_eq!(fixture.evaluations, 1);
        let mut result = Result::default();
        // SAFETY: the owned handle and separate one-variable output buffers are live; zero row counts permit null row buffers.
        let status = unsafe { pse_uno_get_result(handle, &mut result, ptr::null_mut(), 0) };
        assert!(status == NO_RESULT || status == OK);
        let mut value = 0.0;
        assert_eq!(
            // SAFETY: the owned handle and separate one-variable output buffers are live; zero row counts permit null row buffers.
            unsafe {
                pse_uno_get_vectors(
                    handle,
                    &mut value,
                    &mut 0.0,
                    &mut 0.0,
                    1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    0,
                )
            },
            NO_RESULT
        );
        destroy(&mut handle);
    }
    #[test]
    fn recoverable_trial_uses_native_trust_region_retry() {
        let mut fixture = Fixture {
            evaluations: 0,
            terminal: false,
            scope: SCOPE_CONTINUE,
            trial_once: true,
            rejected: 0,
        };
        let mut handle = create(&mut fixture, SQP_LBFGS);
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_uno_solve(handle, &0.0, 1, ptr::null_mut(), 0) },
            OK
        );
        let mut result = Result::default();
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_uno_get_result(handle, &mut result, ptr::null_mut(), 0) },
            OK
        );
        assert_eq!(result.optimization_status, 0);
        assert_eq!(result.iterate_status, 1);
        assert_eq!(fixture.rejected, 1);
        assert_eq!(result.highs_primal_tolerance, 1e-8);
        assert_eq!(result.highs_dual_tolerance, 1e-8);
        assert_eq!(result.highs_qp_regularization, 0.);
        assert!(fixture.evaluations > 2);
        destroy(&mut handle);
    }
    #[test]
    fn scope_stop_precedes_native_start() {
        for (scope, expected) in [
            (SCOPE_CANCEL, CANCELLED),
            (SCOPE_DEADLINE, DEADLINE),
            (SCOPE_ABANDON, ABANDONED),
        ] {
            let mut fixture = Fixture {
                evaluations: 0,
                terminal: false,
                scope,
                trial_once: false,
                rejected: 0,
            };
            let mut handle = create(&mut fixture, SLP);
            assert_eq!(
                // SAFETY: the owned handle retains its live Fixture; one primal coordinate and declared zero-row buffers match its shape.
                unsafe { pse_uno_solve(handle, &0.0, 1, ptr::null_mut(), 0) },
                expected
            );
            assert_eq!(fixture.evaluations, 0);
            destroy(&mut handle);
        }
    }
    fn bounded_profile(profile: i32, curvature: &std::ffi::CStr) {
        let mut fixture = Fixture {
            evaluations: 0,
            terminal: false,
            scope: SCOPE_CONTINUE,
            trial_once: false,
            rejected: 0,
        };
        let mut handle = create(&mut fixture, profile);
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_uno_solve(handle, &0.0, 1, ptr::null_mut(), 0) },
            OK
        );
        let mut result = Result::default();
        assert_eq!(
            // SAFETY: the owned bridge handle is live and result is a separate writable output slot.
            unsafe { pse_uno_get_result(handle, &mut result, ptr::null_mut(), 0) },
            OK
        );
        assert_eq!(result.optimization_status, 0);
        assert_eq!(result.iterate_status, 1);
        if profile == SQP_LBFGS {
            assert!(result.materialization_count > 0);
            assert!(result.materialization_columns > 0);
            assert_eq!(result.materialization_numeric_bytes, 8);
        } else {
            assert_eq!(result.materialization_count, 0);
            assert_eq!(result.materialization_numeric_bytes, 0);
        }
        let mut primal = 0.0;
        let mut lower = 0.0;
        let mut upper = 0.0;
        assert_eq!(
            // SAFETY: the owned handle and separate one-variable output buffers are live; zero row counts permit null row buffers.
            unsafe {
                pse_uno_get_vectors(
                    handle,
                    &mut primal,
                    &mut lower,
                    &mut upper,
                    1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert!((primal - 1.0).abs() < 1e-5, "{primal:?}");
        let mut value = [0_i8; 32];
        assert_eq!(
            // SAFETY: the owned handle, NUL-terminated option key and declared-capacity writable output array are live.
            unsafe {
                pse_uno_get_string_option(
                    handle,
                    c"hessian_model".as_ptr(),
                    value.as_mut_ptr(),
                    value.len(),
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert_eq!(
            // SAFETY: the preceding successful bridge getter wrote a NUL-terminated string into this live array.
            unsafe { std::ffi::CStr::from_ptr(value.as_ptr()) },
            curvature
        );
        assert_eq!(
            // SAFETY: the owned handle, NUL-terminated option key and declared-capacity writable output array are live.
            unsafe {
                pse_uno_get_string_option(
                    handle,
                    c"globalization_strategy".as_ptr(),
                    value.as_mut_ptr(),
                    value.len(),
                    ptr::null_mut(),
                    0,
                )
            },
            OK
        );
        assert_eq!(
            // SAFETY: the preceding successful bridge getter wrote a NUL-terminated string into this live array.
            unsafe { std::ffi::CStr::from_ptr(value.as_ptr()) },
            c"merit_function"
        );
        destroy(&mut handle);
    }
    #[test]
    fn bounded_sqp_solves_and_reports_effective_curvature() {
        bounded_profile(SQP_LBFGS, c"LBFGS");
    }
    #[test]
    fn bounded_slp_solves_and_reports_effective_curvature() {
        bounded_profile(SLP, c"zero");
    }
    #[test]
    fn operator_storage_limit_is_terminal_before_matrix_allocation() {
        let mut fixture = Fixture {
            evaluations: 0,
            terminal: false,
            scope: SCOPE_CONTINUE,
            trial_once: false,
            rejected: 0,
        };
        // History is admitted, but the actual triangular workspace exceeds the
        // remaining allowance. No arbitrary library-wide dimension cap is added.
        let mut handle = create_with_limit(&mut fixture, SQP_LBFGS, 1700);
        assert_eq!(
            // SAFETY: the owned handle and separate one-variable output buffers are live; zero row counts permit null row buffers.
            unsafe { pse_uno_solve(handle, &0., 1, ptr::null_mut(), 0) },
            MEMORY
        );
        assert_eq!(fixture.rejected, 0);
        let mut value = 0.;
        assert_eq!(
            // SAFETY: the owned handle and separate one-variable output buffers are live; zero row counts permit null row buffers.
            unsafe {
                pse_uno_get_vectors(
                    handle,
                    &mut value,
                    &mut 0.,
                    &mut 0.,
                    1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    0,
                )
            },
            NO_RESULT
        );
        destroy(&mut handle);
    }
}
