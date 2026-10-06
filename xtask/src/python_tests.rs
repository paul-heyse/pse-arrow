// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Python selection uses operation-owned fixtures and the configured native substrate.
#[cfg(feature = "package-fixtures")]
use anyhow::{Context, Result, ensure};
use std::path::Path;
#[cfg(feature = "package-fixtures")]
use std::process::Command;
fn explicit_target(root: &Path, args: &[String]) -> bool {
    args.iter().filter(|arg| !arg.starts_with('-')).any(|arg| {
        let path = arg.split("::").next().unwrap_or(arg);
        arg.contains("::") || path.ends_with(".py") || root.join(path).exists()
    })
}
#[cfg(feature = "package-fixtures")]
pub(crate) fn run(root: &Path, args: &[String]) -> Result<()> {
    let mut command = Command::new(root.join(if cfg!(windows) {
        ".venv/Scripts/python.exe"
    } else {
        ".venv/bin/python"
    }));
    command.current_dir(root).args([
        "-m",
        "pytest",
        "--maxfail=0",
        "--continue-on-collection-errors",
        "-m",
        "unit or component",
        "-n",
        "auto",
    ]);
    if !explicit_target(root, args) {
        command.arg("python/pse/tests");
    }
    let status = command
        .args(args)
        .status()
        .context("running selected Python tests")?;
    ensure!(status.success(), "Python tests: {status}");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn python_selection_preserves_explicit_targets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for path in [
            "python/pse/tests/test_canonical_results.py",
            "python/pse/tests/test_canonical_results.py::test_reopen",
            "python/pse/tests",
            "absent_test.py",
        ] {
            assert!(explicit_target(
                root,
                &["-n".into(), "0".into(), path.into()]
            ));
        }
        assert!(!explicit_target(root, &[]));
        assert!(!explicit_target(
            root,
            &["-k".into(), "cache or binding".into()]
        ));
    }
}
