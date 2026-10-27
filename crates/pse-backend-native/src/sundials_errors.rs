// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "SUNDIALS context handler installation and bounded copying of native error text"
)]
//! The only selected SUNDIALS context error sink. No borrowed C storage escapes.
use std::ffi::{c_char, c_void};
use sundials_sys as ffi;
const TEXT_LIMIT: usize = 1024;

/// Replace the context's default stderr handler before any solver/vector allocation.
/// No user-data pointer is retained, so a neutral IDAS context can outlive a session.
/// # Safety
/// `context` is a live SUNDIALS context used exclusively by its owning native thread.
pub(crate) unsafe fn install(context: ffi::SUNContext) -> Result<(), crate::ProblemError> {
    // Pinned SUNDIALS 7.9.0 Push adds to the handler stack. Clear must precede it;
    // otherwise the default SUNLogErrHandlerFn continues writing to stderr.
    // SAFETY: the caller provides a live, exclusively owned context.
    let clear = unsafe { ffi::SUNContext_ClearErrHandlers(context) };
    if clear != 0 {
        return Err(crate::ProblemError::internal(
            "SUNDIALS clear error handlers",
        ));
    }
    // SAFETY: the same live context accepts a static callback with no user data.
    let push =
        unsafe { ffi::SUNContext_PushErrHandler(context, Some(handler), std::ptr::null_mut()) };
    if push != 0 {
        return Err(crate::ProblemError::internal(
            "SUNDIALS bounded error handler",
        ));
    }
    Ok(())
}
unsafe extern "C" fn handler(
    _line: i32,
    _function: *const c_char,
    _file: *const c_char,
    message: *const c_char,
    code: ffi::SUNErrCode,
    _data: *mut c_void,
    _context: ffi::SUNContext,
) {
    // Diagnostics must never unwind through SUNDIALS, including observer failures.
    let _ = std::panic::catch_unwind(|| {
        let Some(context) = pse_diagnostics::observation::current() else {
            return;
        };
        let Some(policy_limit) = context.text_limit_bytes() else {
            return;
        };
        let limit = TEXT_LIMIT.min(policy_limit);
        let mut owned = [0_u8; TEXT_LIMIT];
        let mut length = 0;
        if !message.is_null() {
            while length < limit {
                // SAFETY: native handler contract supplies a terminated C string;
                // stop at its terminator or the selected finite diagnostic ceiling.
                let position = unsafe { message.add(length) };
                // SAFETY: position is within the live native string, including its terminator.
                let byte = unsafe { *position } as u8;
                if byte == 0 {
                    break;
                }
                owned[length] = byte;
                length += 1;
            }
        }
        let text = String::from_utf8_lossy(&owned[..length]);
        // Lossy replacement can grow the UTF-8 representation. Apply the same
        // selected byte ceiling after conversion, keeping only whole characters.
        let mut end = text.len().min(limit);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        context.text("sundials.error", &text[..end]);
        crate::observation::integer("sundials.error.code", "native_code", i64::from(code));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sundials_native_error_text_is_owned_and_bounded() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        for limit in [32, Policy::default().max_text_bytes] {
            let hub = ObservationHub::new(Policy {
                max_text_bytes: limit,
                ..Default::default()
            });
            let operation = hub.start("sundials-error-test");
            let _entered = operation.context().enter();
            let text = std::ffi::CString::new("x".repeat(TEXT_LIMIT * 2)).unwrap();
            // SAFETY: valid C string for the duration of the direct handler exercise.
            unsafe {
                handler(
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                    text.as_ptr(),
                    -1,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            }
            drop(text);
            let snapshot = operation
                .reader()
                .snapshot(std::time::Duration::from_secs(60));
            let metric = snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == "sundials.error")
                .unwrap();
            assert!(
                matches!(&metric.latest, Value::Text(value) if value.len() == limit && value.bytes().all(|byte| byte == b'x'))
            );
            // Invalid native UTF-8 must not grow beyond the selected observer limit.
            let text = std::ffi::CString::new(vec![0xff; TEXT_LIMIT * 2]).unwrap();
            // SAFETY: this live CString retains its bytes throughout the callback.
            unsafe {
                handler(
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                    text.as_ptr(),
                    -1,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            }
            drop(text);
            let snapshot = operation
                .reader()
                .snapshot(std::time::Duration::from_secs(60));
            let metric = snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == "sundials.error")
                .unwrap();
            assert!(
                matches!(&metric.latest, Value::Text(value) if !value.is_empty() && value.len() <= limit && value.chars().all(|character| character == '\u{fffd}'))
            );
        }
    }
    #[cfg(feature = "kinsol")]
    #[test]
    fn selected_sundials_handler_replaces_existing_native_handler_stack() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        unsafe extern "C" fn previous(
            _: i32,
            _: *const c_char,
            _: *const c_char,
            _: *const c_char,
            _: ffi::SUNErrCode,
            data: *mut c_void,
            _: ffi::SUNContext,
        ) {
            // SAFETY: the test retains this counter until the context is destroyed.
            unsafe { &*data.cast::<AtomicUsize>() }.fetch_add(1, Ordering::SeqCst);
        }
        let previous_calls = AtomicUsize::new(0);
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("native-error-stack-test");
        let _entered = operation.context().enter();
        let mut context = std::ptr::null_mut();
        // SAFETY: the out pointer is valid and the serial communicator is null.
        assert_eq!(unsafe { ffi::SUNContext_Create(0, &mut context) }, 0);
        // SAFETY: context is live and previous_calls outlives the handler stack.
        let pushed = unsafe {
            ffi::SUNContext_PushErrHandler(
                context,
                Some(previous),
                std::ptr::from_ref(&previous_calls).cast_mut().cast(),
            )
        };
        assert_eq!(pushed, 0);
        // SAFETY: this test exclusively owns the live context.
        unsafe { install(context) }.unwrap();
        // SAFETY: context remains live until after this KINSOL object is freed.
        let mut memory = unsafe { ffi::KINCreate(context) };
        assert!(!memory.is_null());
        // SAFETY: memory is a live KINSOL object; the invalid option exercises its error handler.
        assert!(unsafe { ffi::KINSetNumMaxIters(memory, -1) } < 0);
        // SAFETY: memory is exclusively owned and has not been freed.
        unsafe { ffi::KINFree(&mut memory) };
        // SAFETY: all users of this live, exclusively owned context were destroyed.
        assert_eq!(unsafe { ffi::SUNContext_Free(&mut context) }, 0);
        assert_eq!(
            previous_calls.load(Ordering::SeqCst),
            0,
            "pushing without clearing would retain the previous handler"
        );
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert!(
            snapshot
                .metrics
                .iter()
                .any(|metric| metric.name == "sundials.error")
        );
    }
}
