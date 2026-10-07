// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Python selection uses operation-owned fixtures and the configured native substrate.
#[cfg(feature = "package-fixtures")]
use anyhow::{Context, Result, ensure};
#[cfg(feature = "package-fixtures")]
use std::{path::Path, process::Command};

fn arguments(extra: &[String]) -> Vec<String> {
    // Pytest uses testpaths only when no positional file/directory/node is given.
    // Its own parser handles option values and any installed plugin's options.
    let mut args = [
        "-m",
        "pytest",
        "--maxfail=0",
        "--continue-on-collection-errors",
        "-m",
        "unit or component",
        "-n",
        "auto",
        "-o",
        "testpaths=python/pse/tests",
    ]
    .map(str::to_owned)
    .to_vec();
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
    fn python_selection_defaults_are_not_positional_targets() {
        let args = arguments(&[]);
        assert!(
            args.windows(2)
                .any(|pair| pair == ["-o", "testpaths=python/pse/tests"])
        );
        assert!(!args.iter().any(|arg| arg == "python/pse/tests"));
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
