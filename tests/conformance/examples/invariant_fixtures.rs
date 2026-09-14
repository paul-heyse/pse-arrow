// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Refresh explicit fixtures when a reviewed registry invariant changes.
#[path = "../src/fixture.rs"]
mod fixture;

fn main() -> Result<(), pse_schema::SchemaError> {
    let registry = pse_schema::catalog::assemble()?;
    for invariant in registry.invariants() {
        let (valid, violating) = fixture::pair(&registry, invariant);
        let path = fixture::directory(invariant);
        valid.save(&path.join("valid.yaml"));
        violating.save(&path.join("violating.yaml"));
    }
    Ok(())
}
