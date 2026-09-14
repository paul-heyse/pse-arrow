// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Runtime checkout discovery shared by the driver and governance tests.

use std::path::{Path, PathBuf};

/// Locate the invocation's checkout, including when a cached binary was built elsewhere.
/// A recognizable but incomplete checkout is an error, never a fallback to another root.
pub(crate) fn find_workspace_root(start: &Path) -> Result<PathBuf, String> {
    let start = start.canonicalize().map_err(|error| {
        format!(
            "resolving invocation directory {}: {error}",
            start.display()
        )
    })?;
    for candidate in start.ancestors() {
        let path = candidate.join("Cargo.toml");
        if !path.is_file() {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("reading {}: {error}", path.display()))?;
        let manifest: toml::Value = toml::from_str(&source)
            .map_err(|error| format!("parsing {}: {error}", path.display()))?;
        if manifest
            .get("workspace")
            .and_then(|value| value.get("metadata"))
            .and_then(|value| value.get("pse"))
            .and_then(toml::Value::as_table)
            .is_none()
        {
            continue;
        }
        for (relative, expected_name) in [
            ("xtask/Cargo.toml", "xtask"),
            ("tests/governance/Cargo.toml", "pse-tests-governance"),
        ] {
            let member_path = candidate.join(relative);
            let source = std::fs::read_to_string(&member_path)
                .map_err(|error| format!("reading {}: {error}", member_path.display()))?;
            let member: toml::Value = toml::from_str(&source)
                .map_err(|error| format!("parsing {}: {error}", member_path.display()))?;
            if member
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str)
                != Some(expected_name)
            {
                return Err(format!(
                    "{} must declare package {expected_name}",
                    member_path.display()
                ));
            }
        }
        return Ok(candidate.to_path_buf());
    }
    Err(format!(
        "no pse-arrow workspace above invocation directory {}",
        start.display()
    ))
}
