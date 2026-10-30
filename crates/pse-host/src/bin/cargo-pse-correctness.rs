// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native Cargo correctness entry point; compiler and runner output remains original.
use std::{
    io::{self, Write},
    process::ExitCode,
};
fn main() -> ExitCode {
    match pse_host::correctness::run(std::env::args_os().skip(1).collect()) {
        Ok(status) => ExitCode::from(u8::try_from(status).unwrap_or(125)),
        Err(error) => {
            let _ = writeln!(io::stderr(), "arrow-validation: {error}");
            ExitCode::from(2)
        }
    }
}
