// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Refresh explicit fixtures when a reviewed registry invariant changes.
#[path = "../src/fixture.rs"]
mod fixture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let check = std::env::args().skip(1).any(|arg| arg == "--check");
    let registry = pse_schema::catalog::assemble()?;
    let cases = registry
        .invariants()
        .iter()
        .map(|invariant| {
            (
                fixture::directory(invariant),
                fixture::pair(&registry, invariant),
            )
        })
        .collect::<Vec<_>>();
    let paths = cases
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut mismatches = Vec::new();
    for (path, (valid, violating)) in cases {
        for (case, value) in [("valid", valid), ("violating", violating)] {
            let text = format!("{}\n", serde_json::to_string_pretty(&value)?);
            emit(
                &path.join(format!("{case}.yaml")),
                &text,
                check,
                &mut mismatches,
            )?;
        }
    }
    let obsolete =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/generated/invariant_cases.rs");
    if obsolete.exists() {
        if check {
            mismatches.push(obsolete);
        } else {
            std::fs::remove_file(obsolete)?;
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/invariants");
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && !paths.contains(&entry.path()) {
            if check {
                mismatches.push(entry.path());
            } else {
                std::fs::remove_dir_all(entry.path())?;
            }
        }
    }
    if !mismatches.is_empty() {
        return Err(format!("stale generated invariant fixtures: {mismatches:?}").into());
    }
    Ok(())
}

fn emit(
    path: &std::path::Path,
    text: &str,
    check: bool,
    mismatches: &mut Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    if std::fs::read_to_string(path).ok().as_deref() == Some(text) {
        return Ok(());
    }
    if check {
        mismatches.push(path.to_owned());
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
    }
    Ok(())
}
