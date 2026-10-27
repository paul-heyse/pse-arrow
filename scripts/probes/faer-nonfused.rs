// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standalone optimized IR probe against the exact selected workspace faer artifact.
#[path = "../../crates/pse-backend-native/src/arithmetic_tests.rs"]
mod arithmetic;
fn main() {
    arithmetic::adversarial();
    arithmetic::faer_nonfused_decompositions();
    arithmetic::faer_nonfused_sparse_modes();
}
