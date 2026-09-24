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
//! `just codegen --only bindgen` from the actual `IpStdCInterface.h` headers.
//! `IPOPT_DIR` selects an installed prefix; otherwise the generator extracts headers
//! from the digest-pinned solver image. The C interface and dependent types are
//! generated and committed, and `just codegen-check` compares real regeneration.
//! Plan 14 M11 supplies the safe native driver; the former driver has been removed.

include!("bindings.rs");

#[cfg(all(test, feature = "link"))]
mod abi_tests {
    #[test]
    fn pinned_c_interface_links_with_expected_scalar_and_index_width() {
        assert_eq!(size_of::<super::ipindex>(), 4);
        assert_eq!(size_of::<super::ipnumber>(), 8);
        assert_eq!(super::IPOPT_VERSION, b"3.14.20\0");
        // Retain relocations to the real C entry points without running a solve.
        std::hint::black_box(super::CreateIpoptProblem);
        std::hint::black_box(super::FreeIpoptProblem);
        std::hint::black_box(super::IpoptSolve);
        std::hint::black_box(super::SetIntermediateCallback);
    }
}
