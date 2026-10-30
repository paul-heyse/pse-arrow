// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Agent front door over the shared native command library.
fn main() -> std::process::ExitCode {
    pse_host::cli::main(pse_host::cli::BinKind::Pse)
}
