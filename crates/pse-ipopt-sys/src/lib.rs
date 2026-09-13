// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    unsafe_code,
    reason = "FFI bindings to the Ipopt C API (blueprint §18.3)"
)]
#![allow(
    missing_docs,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::all,
    clippy::pedantic,
    reason = "generated bindings"
)]

//! Raw FFI bindings to the Ipopt C API (blueprint §3.1, §18.3).
//!
//! Nothing here is written by hand. `src/bindings.rs` is produced by
//! `cargo xtask codegen --only bindgen` from `$IPOPT_DIR/include/coin-or/`IpStdCInterface`.h`
//! inside the solver container, with an allowlist of the §18.3 functions, and committed.
//! The safe driver — `index_style = 0`, one `catch_unwind` per callback — is
//! `pse-backend-native`.

include!("bindings.rs");
