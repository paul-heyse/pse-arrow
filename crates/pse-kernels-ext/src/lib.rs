// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    unsafe_code,
    reason = "FFI to external property libraries (blueprint §9.8)"
)]

//! Optional external property providers: CoolProp FFI, Helmholtz libraries and FeOs (blueprint §3.2, §9.8).
//!
//! Every `extern "C"` callback in this crate catches unwinds at the boundary
//! (governance `ffi_callbacks_catch_unwind`, blueprint §9.8).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
