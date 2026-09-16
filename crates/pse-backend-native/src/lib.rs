// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    unsafe_code,
    reason = "FFI driver over the Ipopt C API (blueprint §18.3)"
)]

//! Ipopt C-API FFI driver over the evaluation program (blueprint §3.2, §18.3).
//!
//! `index_style = 0` is asserted once here and every `extern "C"` callback wraps its
//! body in `catch_unwind`: an unwind through Ipopt's C frames is undefined behaviour
//! (blueprint §18.3).
//!
//! Native problem execution prepares expressions once and owns the mutable callback
//! workspace until Ipopt returns. No callback plans or rebuilds a query.

#[cfg(feature = "ipopt")]
mod driver;
mod error;
pub mod native;
mod problem;

pub use error::NativeError;
pub use problem::{HessianPolicy, SolveOptions, Variable};
