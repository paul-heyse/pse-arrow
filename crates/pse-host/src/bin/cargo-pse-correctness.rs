// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native Cargo correctness entry point; compiler and runner output remains original.
fn main() -> std::process::ExitCode {
    pse_host::cli::main(pse_host::cli::BinKind::CargoPseCorrectness)
}
