// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Refresh explicit fixtures when a reviewed registry invariant changes.
#[path = "../src/fixture.rs"]
mod fixture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    for (path, (valid, violating)) in cases {
        valid.save(&path.join("valid.yaml"));
        violating.save(&path.join("violating.yaml"));
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/invariants");
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && !paths.contains(&entry.path()) {
            std::fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}
