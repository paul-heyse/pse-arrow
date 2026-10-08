// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    unsafe_code,
    reason = "ADR-0164 bounded supported-glibc reconstruction scope with callback panic containment"
)]

//! Bounded glibc loader exclusion for immutable reconstruction.
//!
//! Reviewed glibc 2.39 source: elf/dl-iteratephdr.c holds dl_load_write_lock for
//! callbacks; elf/dl-object.c and elf/dl-close.c use the same lock for namespace
//! publication/removal. This implementation contract is version-bounded.

use std::{
    ffi::CStr,
    io,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

thread_local! { static MODULE_TLS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub(super) fn touch_current_module_tls() {
    MODULE_TLS.with(|marker| {
        marker.set(true);
        // Materialize this TLS address even under LTO. All statically linked
        // Rust mathematical/receipt TLS shares the executable or _native DSO.
        let address: *const std::cell::Cell<bool> = marker;
        std::hint::black_box(address);
    });
}

unsafe extern "C" fn counters(
    info: *mut libc::dl_phdr_info,
    size: usize,
    data: *mut libc::c_void,
) -> libc::c_int {
    catch_unwind(AssertUnwindSafe(|| {
        if info.is_null()
            || data.is_null()
            || size
                < std::mem::offset_of!(libc::dl_phdr_info, dlpi_subs)
                    + size_of::<libc::c_ulonglong>()
        {
            return 1;
        }
        // SAFETY: glibc supplied this extent-checked live struct.
        let info = unsafe { &*info };
        // SAFETY: data is our stack-owned Option for this synchronous callback.
        unsafe {
            *data.cast::<Option<(u64, u64)>>() = Some((info.dlpi_adds, info.dlpi_subs));
        }
        1
    }))
    .unwrap_or(1)
}
fn read_counters() -> io::Result<(u64, u64)> {
    let mut result = None::<(u64, u64)>;
    // SAFETY: callback receives the live stack-owned result only until this
    // synchronous call returns. The callback performs no Rust unwinding.
    unsafe {
        libc::dl_iterate_phdr(Some(counters), (&raw mut result).cast());
    }
    result.ok_or_else(|| io::Error::other("supported loader counter extent unavailable"))
}
struct Invocation<F, T> {
    operation: Option<F>,
    result: Option<std::thread::Result<io::Result<T>>>,
}
unsafe extern "C" fn invoke<F: FnOnce() -> io::Result<T>, T>(
    _: *mut libc::dl_phdr_info,
    _: usize,
    data: *mut libc::c_void,
) -> libc::c_int {
    // SAFETY: run passes a unique Invocation of these exact generic types and
    // dl_iterate_phdr invokes synchronously before its stack lifetime ends.
    let state = unsafe { &mut *data.cast::<Invocation<F, T>>() };
    state.result = Some(catch_unwind(AssertUnwindSafe(|| {
        let before = read_counters()?;
        let operation = state
            .operation
            .take()
            .ok_or_else(|| io::Error::other("loader operation already consumed"))?;
        let result = operation();
        if read_counters()? != before {
            return Err(io::Error::other(
                "reentrant loading changed the local runtime during reconstruction",
            ));
        }
        result
    })));
    1
}
pub(super) fn run<F: FnOnce() -> io::Result<T>, T>(operation: F) -> io::Result<T> {
    // glibc dlopen acquires TLS lock before loaded-object write lock. Allocate
    // this thread's owning-module TLS before taking the latter: first dynamic
    // access fixes its static/forced-dynamic offset. Later DTV generation updates
    // use _dl_update_slotinfo's atomic lookup, without taking dl_load_tls_lock;
    // they preserve already allocated TLS for this unchanged owning module.
    // Callback construction invokes no code/TLS from a newly loaded DSO.
    touch_current_module_tls();
    // SAFETY: this supported glibc function returns its static version pointer.
    let version = unsafe { libc::gnu_get_libc_version() };
    // SAFETY: glibc's returned pointer is process-lifetime and NUL-terminated.
    let version = unsafe { CStr::from_ptr(version) };
    if version.to_bytes() != b"2.39" {
        return Err(io::Error::other("unreviewed glibc loader interpretation"));
    }
    let mut state = Invocation {
        operation: Some(operation),
        result: None,
    };
    // SAFETY: unique stack state remains valid throughout this synchronous call;
    // invoke catches every Rust panic before returning across the C ABI.
    unsafe {
        libc::dl_iterate_phdr(Some(invoke::<F, T>), (&raw mut state).cast());
    }
    match state.result {
        Some(Ok(result)) => result,
        Some(Err(panic)) => resume_unwind(panic),
        None => Err(io::Error::other(
            "supported loader did not invoke the reconstruction scope",
        )),
    }
}
