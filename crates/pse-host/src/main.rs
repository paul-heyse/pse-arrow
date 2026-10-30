// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed native entry point over the shared command library.
fn main() -> std::process::ExitCode {
    pse_host::cli::main(pse_host::cli::BinKind::Host)
}
