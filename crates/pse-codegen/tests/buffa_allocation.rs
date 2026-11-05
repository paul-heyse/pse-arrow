// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exercise maintained Buffa's generated decoders and schema-name policy.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "supplier decoder controls preserve original failures and compile generated fixtures"
)]

#[path = "../../../vendor/buffa-codegen/tests/allocation_compile.rs"]
mod allocation_compile;

#[path = "../../../vendor/buffa-codegen/tests/schema_names.rs"]
mod schema_names;
