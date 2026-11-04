// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private explicitly deployed scientific provider helper.
use std::io::Write as _;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--scientific-worker")) {
        return Err("private scientific worker requires its owned launch contract".into());
    }
    let endpoint = args.next().ok_or("scientific worker endpoint absent")?;
    let session = args.next().ok_or("scientific worker session absent")?;
    let session =
        pse_ids::SemanticId::parse_hex(session.to_str().ok_or("invalid session encoding")?)?;
    let parent = args.next().ok_or("scientific worker parent absent")?;
    let parent: u32 = parent.to_str().ok_or("invalid parent encoding")?.parse()?;
    if args.next().is_some() {
        return Err("unexpected scientific worker launch argument".into());
    }
    pse_runtime::scientific_worker::serve(std::path::PathBuf::from(endpoint), session, parent)?;
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(std::io::stderr().lock(), "pse-scientific-worker: {error}");
            std::process::ExitCode::from(125)
        }
    }
}
