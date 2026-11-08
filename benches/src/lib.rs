// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Criterion benchmarks for the blueprint §24.3 performance groups.
//!
//! Ordinary smoke covers the local selection. Prepared canonical and solver cases use
//! named runs, and measurements retain Criterion's samples separately from smoke.

use std::ffi::OsStr;

/// The native Criterion invocation's actual execution mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CriterionMode {
    /// Exercise the selected callback once.
    Smoke,
    /// Sample the selected callback.
    Measure,
    /// List selected registrations without executing their callbacks.
    List,
    /// Criterion's separately selected profiling mode.
    Profile,
}

impl CriterionMode {
    /// Observe Criterion 0.8.2's mode switches. Its configured runner still owns
    /// argument validation, filtering and callback execution.
    pub fn from_args(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        let mut bench = false;
        let mut test = false;
        let mut profile = false;
        for argument in args {
            let argument = argument.as_ref();
            match argument.as_encoded_bytes() {
                b"--" => break,
                b"--list" => return Self::List,
                b"--bench" => bench = true,
                b"--test" => test = true,
                bytes if bytes == b"--profile-time" || bytes.starts_with(b"--profile-time=") => {
                    profile = true;
                }
                _ => {}
            }
        }
        match (bench && !test, profile) {
            (false, _) => Self::Smoke,
            (true, false) => Self::Measure,
            (true, true) => Self::Profile,
        }
    }

    /// Observe the current harness's arguments, skipping its executable name.
    pub fn current() -> Self {
        Self::from_args(std::env::args_os().skip(1))
    }

    /// Stable label used in the run's selection record.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Measure => "measure",
            Self::List => "list",
            Self::Profile => "profile",
        }
    }
}
