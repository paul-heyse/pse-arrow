// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Python selection uses operation-owned fixtures and the configured native substrate.
#[cfg(feature = "package-fixtures")]
use anyhow::{Context, Result, ensure};
#[cfg(feature = "package-fixtures")]
use std::{path::Path, process::Command};

fn arguments(extra: &[String]) -> Vec<String> {
    // One Python owner selects categories, process allocation and worker count.
    // Keep option values and positional targets intact across this frontdoor.
    let mut args = ["-m", "scripts.python_tests"].map(str::to_owned).to_vec();
    args.extend_from_slice(extra);
    args
}

#[cfg(feature = "package-fixtures")]
pub(crate) fn run(root: &Path, args: &[String]) -> Result<()> {
    let status = Command::new(root.join(if cfg!(windows) {
        ".venv/Scripts/python.exe"
    } else {
        ".venv/bin/python"
    }))
    .current_dir(root)
    .args(arguments(args))
    .status()
    .context("running selected Python tests")?;
    ensure!(status.success(), "Python tests: {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_selection_uses_one_partition_and_placement_owner() {
        assert_eq!(arguments(&[]), ["-m", "scripts.python_tests"]);
        let extra = ["--managed-primary-route", "-m", "integration"].map(str::to_owned);
        let args = arguments(&extra);
        assert_eq!(&args[2..], extra.as_slice());
    }

    #[test]
    fn python_selection_user_arguments_follow_defaults_unchanged() {
        let extra = [
            "--log-file",
            "out.log",
            "--assert",
            "plain",
            "--junitprefix",
            "named",
            "python/pse/tests/test_studies.py::test_selected",
            "-o",
            "testpaths=selected_directory",
        ]
        .map(str::to_owned);
        let args = arguments(&extra);
        assert_eq!(&args[args.len() - extra.len()..], extra.as_slice());
        assert_eq!(args.len(), arguments(&[]).len() + extra.len());
    }
}
