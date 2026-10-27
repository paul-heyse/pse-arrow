// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "scoped SUNDIALS allocation/workspace ABI controls over live exclusively owned native objects"
)]
use sundials_sys as ffi;

#[test]
#[cfg(feature = "kinsol")]
fn sundials_790_added_statuses_preserve_native_names_and_causes() {
    for (code, name) in [
        (ffi::KIN_DAMPING_FN_ERR, "KIN_DAMPING_FN_ERR"),
        (ffi::KIN_DEPTH_FN_ERR, "KIN_DEPTH_FN_ERR"),
    ] {
        let value = crate::kinsol::termination(code);
        assert_eq!(value.name, name);
        assert_eq!(value.category, crate::solve::Termination::Evaluation);
    }
}

#[test]
fn sundials_790_krylov_reporters_and_opaque_headers_fit_admitted_layout() {
    let header = size_of::<ffi::_generic_N_Vector>()
        + size_of::<ffi::_generic_N_Vector_Ops>()
        + size_of::<ffi::_N_VectorContent_Serial>();
    assert_eq!(
        header, 504,
        "re-audit the consumed native vector ABI on a supplier move"
    );
    assert!(
        header <= 512,
        "per-vector header allowance must cover the selected ABI"
    );
    let n: i64 = 17;
    let m: i32 = 5;
    let mut context = std::ptr::null_mut();
    // SAFETY: valid out pointer, serial communicator, released after all dependent objects.
    assert_eq!(
        unsafe { ffi::SUNContext_Create(ffi::comm_no_mpi(), &mut context) },
        0
    );
    // SAFETY: the context is exclusively owned and live until cleanup below.
    unsafe { crate::sundials_errors::install(context) }.unwrap();
    // SAFETY: admitted small finite extent on the live context.
    let vector = unsafe { ffi::N_VNew_Serial(n, context) };
    assert!(!vector.is_null());
    for flexible in [false, true] {
        // SAFETY: live vector/context, supported no-preconditioner strategy and dimension.
        let solver = unsafe {
            if flexible {
                ffi::SUNLinSol_SPFGMR(vector, 0, m, context)
            } else {
                ffi::SUNLinSol_SPGMR(vector, 0, m, context)
            }
        };
        assert!(!solver.is_null());
        // SAFETY: this exclusively owned solver initializes its own work arrays.
        assert_eq!(unsafe { ffi::SUNLinSolInitialize(solver) }, 0);
        let (mut reals, mut integers) = (0, 0);
        // SAFETY: read-only query on the live initialized solver, valid out pointers.
        assert_eq!(
            unsafe { ffi::SUNLinSolSpace(solver, &mut reals, &mut integers) },
            0
        );
        let vectors = if flexible { 2 * m + 4 } else { m + 5 };
        assert_eq!(reals, n * i64::from(vectors) + i64::from(m * (m + 5) + 2));
        assert_eq!(integers, i64::from(vectors));
        let selected = if flexible {
            crate::dynamics::IdasLinear::Spfgmr {
                dimension: pse_model::scalars::PositiveCount::try_new(m as usize).unwrap(),
                preconditioner: crate::solve::Preconditioner::None,
            }
        } else {
            crate::dynamics::IdasLinear::Spgmr {
                dimension: pse_model::scalars::PositiveCount::try_new(m as usize).unwrap(),
                preconditioner: crate::solve::Preconditioner::None,
            }
        };
        let layout =
            crate::dynamics::PreparedLayout::new(n as usize, std::iter::empty(), usize::MAX)
                .unwrap();
        let observed = (reals + integers) as usize * 8 + vectors as usize * header;
        assert!(observed <= layout.linear_bytes(selected).unwrap());
        // SAFETY: each solver is freed before its template vector and context.
        assert_eq!(unsafe { ffi::SUNLinSolFree(solver) }, 0);
    }
    // SAFETY: no solver retains the template vector; no object retains the context.
    unsafe {
        ffi::N_VDestroy(vector);
        ffi::SUNContext_Free(&mut context);
    }
}

#[test]
#[cfg(feature = "kinsol")]
fn sundials_790_anderson_allocations_follow_allocated_depth_through_resize_failure_and_drop() {
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
    static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
    static CLONES: AtomicUsize = AtomicUsize::new(0);
    static DESTROYS: AtomicUsize = AtomicUsize::new(0);
    static FAIL_AT: AtomicUsize = AtomicUsize::new(usize::MAX);
    unsafe extern "C" fn clone(vector: ffi::N_Vector) -> ffi::N_Vector {
        if ATTEMPTS.fetch_add(1, SeqCst) + 1 == FAIL_AT.load(SeqCst) {
            // SAFETY: test-only supplier seam models the serial allocator's
            // malloc failure signal on this vector's exclusively owned context.
            unsafe {
                (*(*vector).sunctx).last_err = ffi::SUNErrCode__SUN_ERR_MALLOC_FAIL;
            }
            return std::ptr::null_mut();
        }
        // SAFETY: native cloning supplies a live serial template. The serial clone
        // copies its operation table, retaining these test-only ownership callbacks.
        let copied = unsafe { ffi::N_VClone_Serial(vector) };
        if !copied.is_null() {
            CLONES.fetch_add(1, SeqCst);
        }
        copied
    }
    unsafe extern "C" fn clone_empty(vector: ffi::N_Vector) -> ffi::N_Vector {
        if ATTEMPTS.fetch_add(1, SeqCst) + 1 == FAIL_AT.load(SeqCst) {
            // SAFETY: same deterministic private allocation-error seam as above.
            unsafe {
                (*(*vector).sunctx).last_err = ffi::SUNErrCode__SUN_ERR_MALLOC_FAIL;
            }
            return std::ptr::null_mut();
        }
        // SAFETY: serial header clone of a live template, preserving test ops.
        let copied = unsafe { ffi::N_VCloneEmpty_Serial(vector) };
        if !copied.is_null() {
            CLONES.fetch_add(1, SeqCst);
        }
        copied
    }
    unsafe extern "C" fn destroy(vector: ffi::N_Vector) {
        DESTROYS.fetch_add(1, SeqCst);
        // SAFETY: each callback receives one live exclusively owned serial vector.
        unsafe {
            ffi::N_VDestroy_Serial(vector);
        }
    }
    unsafe extern "C" fn map(
        input: ffi::N_Vector,
        output: ffi::N_Vector,
        _: *mut std::ffi::c_void,
    ) -> i32 {
        // SAFETY: this control supplies equal 17-entry serial input/output vectors.
        unsafe {
            let input = std::slice::from_raw_parts(ffi::N_VGetArrayPointer_Serial(input), 17);
            let output = std::slice::from_raw_parts_mut(ffi::N_VGetArrayPointer_Serial(output), 17);
            for (source, target) in input.iter().zip(output) {
                *target = 1.0 + 0.5 * source;
            }
        }
        0
    }
    // Fail clone indices 0, 1 and 2 in each of Q, df, dg, then the orthogonalization work vector.
    // N_VCloneVectorArray owns cleanup of a partially cloned array; KINFreeAA
    // owns every earlier completed array using its published allocation count.
    for fail_offset in [
        None,
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        Some(12),
    ] {
        ATTEMPTS.store(0, SeqCst);
        CLONES.store(0, SeqCst);
        DESTROYS.store(0, SeqCst);
        FAIL_AT.store(usize::MAX, SeqCst);
        let mut context = std::ptr::null_mut();
        // SAFETY: all native objects below have one test owner and are destroyed
        // before the context. No failing allocation is requested from the host.
        unsafe {
            assert_eq!(ffi::SUNContext_Create(ffi::comm_no_mpi(), &mut context), 0);
            crate::sundials_errors::install(context).unwrap();
            let vector = ffi::N_VNew_Serial(17, context);
            assert!(!vector.is_null());
            (*(*vector).ops).nvclone = Some(clone);
            (*(*vector).ops).nvcloneempty = Some(clone_empty);
            (*(*vector).ops).nvdestroy = Some(destroy);
            let scale = ffi::N_VClone(vector);
            assert!(!scale.is_null());
            ffi::N_VConst(1.0, scale);
            let mut memory = ffi::KINCreate(context);
            assert!(!memory.is_null());
            assert_eq!(ffi::KINInit(memory, Some(map), vector), 0);
            assert_eq!(ffi::KINSetOrthAA(memory, ffi::KIN_ORTH_ICWY), 0);
            assert_eq!(ffi::KINSetFuncNormTol(memory, 1e-8), 0);
            if let Some(offset) = fail_offset {
                FAIL_AT.store(ATTEMPTS.load(SeqCst) + offset, SeqCst);
            }
            for (iterations, depth) in [(4, 3), (1, 0), (9, 5), (1, 0)] {
                ffi::N_VConst(0.0, vector);
                assert_eq!(ffi::KINSetNumMaxIters(memory, iterations), 0);
                assert_eq!(ffi::KINSetMAA(memory, depth), 0);
                let flag = ffi::KINSol(memory, vector, ffi::KIN_FP, scale, scale);
                if fail_offset.is_some() {
                    assert_eq!(flag, ffi::KIN_MEM_FAIL);
                    assert_eq!(ATTEMPTS.load(SeqCst), FAIL_AT.load(SeqCst));
                    break;
                }
                assert!(
                    flag >= 0 || flag == ffi::KIN_MAXITER_REACHED,
                    "unexpected native status {flag}"
                );
            }
            ffi::KINFree(&mut memory);
            ffi::SUNContext_GetLastError(context);
            // Exercise the identical generic empty-array cleanup at j=0/1/2.
            if fail_offset.is_none() {
                for offset in 1..=3 {
                    FAIL_AT.store(ATTEMPTS.load(SeqCst) + offset, SeqCst);
                    assert!(ffi::N_VCloneEmptyVectorArray(3, vector).is_null());
                    assert_eq!(ATTEMPTS.load(SeqCst), FAIL_AT.load(SeqCst));
                    ffi::SUNContext_GetLastError(context);
                }
            }
            ffi::N_VDestroy(scale);
            ffi::N_VDestroy(vector);
            ffi::SUNContext_Free(&mut context);
        }
        assert_eq!(
            DESTROYS.load(SeqCst),
            CLONES.load(SeqCst) + 1,
            "every allocated clone and original vector must be destroyed exactly once ({fail_offset:?})"
        );
    }
}
