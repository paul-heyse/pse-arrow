// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `pse-host`: the host command tree as hooks, seeds and tests exec it.
fn main() -> std::process::ExitCode {
    pse_host::cli::main(pse_host::cli::BinKind::PseHost)
}
