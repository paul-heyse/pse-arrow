// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! `pse`: the agent-facing front door over the host command tree.
fn main() -> std::process::ExitCode {
    pse_host::cli::main(pse_host::cli::BinKind::Pse)
}
