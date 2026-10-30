// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native completed-report reconciliation; no test-event lifecycle is reconstructed.
use super::{ExtractionError, NextestExitClass, NextestOutcome, NextestSetup, NextestSummary};
use crate::{
    discovery::{Inventory, TestIdentity},
    selection::IgnoredPolicy,
};
use quick_junit::TestCaseStatus;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::BufReader,
    path::PathBuf,
};

/// Native retry facts, retaining exact selected identity.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct RetrySummary {
    /// Original selected test.
    pub identity: TestIdentity,
    /// Attempts recorded by the native writer, including the final attempt.
    pub attempts: usize,
    /// A final successful result after earlier unsuccessful attempts.
    pub flaky: bool,
}
/// Address of a selected failure in its original completed XML artifact.
#[derive(Clone, Debug)]
pub struct NativeFailure {
    /// Original selected test.
    pub identity: TestIdentity,
    /// Original invocation/group report; XML text is not raw process output.
    pub report: PathBuf,
}
/// Captured population and invocation-private report location.
#[derive(Debug)]
pub struct CompletedReport {
    path: PathBuf,
    selected: BTreeMap<(String, String), TestIdentity>,
    known: BTreeSet<(String, String)>,
    ignored_policy: IgnoredPolicy,
    fixture_outcome: Option<PathBuf>,
}
impl CompletedReport {
    /// Original invocation-owned report location, available even when publication fails.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
    /// Require the original borrowing fixture owner's result separately from the wrapper exit.
    pub fn with_fixture_outcome(mut self, path: PathBuf) -> Self {
        self.fixture_outcome = Some(path);
        self
    }
    /// Bind the observed native population before execution.
    pub fn new(
        inventory: &Inventory,
        group: usize,
        path: PathBuf,
    ) -> Result<Self, ExtractionError> {
        if !inventory.capture_complete()
            || !inventory
                .nextest_version
                .starts_with("cargo-nextest 0.9.146 ")
        {
            return Err(ExtractionError::Nextest(
                "completed reports require qualified complete nextest 0.9.146 inventory".into(),
            ));
        }
        let listing = &inventory
            .groups
            .get(group)
            .ok_or_else(|| ExtractionError::Nextest("captured group is absent".into()))?
            .listing;
        Self::from_listing(listing, group, path, inventory.selection.ignored())
    }
    fn from_listing(
        listing: &nextest_metadata::TestListSummary,
        group: usize,
        path: PathBuf,
        ignored_policy: IgnoredPolicy,
    ) -> Result<Self, ExtractionError> {
        let mut selected = BTreeMap::new();
        let mut known = BTreeSet::new();
        for (binary, suite) in &listing.rust_suites {
            if binary != &suite.binary.binary_id
                || suite.status != nextest_metadata::RustTestSuiteStatusSummary::LISTED
            {
                return Err(ExtractionError::Nextest(
                    "native suite is not completely listed".into(),
                ));
            }
            for (name, case) in &suite.test_cases {
                let key = (binary.to_string(), name.to_string());
                known.insert(key.clone());
                if case.filter_match.is_match() {
                    selected.insert(
                        key,
                        TestIdentity {
                            group,
                            package_id: suite.binary.package_id.clone(),
                            binary_id: binary.to_string(),
                            test_name: name.to_string(),
                        },
                    );
                }
            }
        }
        Ok(Self {
            path,
            selected,
            known,
            ignored_policy,
            fixture_outcome: None,
        })
    }
    /// Interpret completed evidence after raw capture and original wait, retaining native exit.
    pub fn finish(
        self,
        child_exit: i32,
        capture_complete: bool,
    ) -> (NextestSummary, Vec<NativeFailure>, Option<ExtractionError>) {
        let mut completed = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut failed = Vec::new();
        let mut retries = Vec::new();
        let mut setup_failed = false;
        let mut error = None;
        let decoded = File::open(&self.path)
            .map(BufReader::new)
            .map_err(|cause| cause.to_string())
            .and_then(|reader| {
                quick_junit::Report::deserialize(reader).map_err(|cause| cause.to_string())
            });
        let parsed = decoded.is_ok();
        match decoded {
            Err(cause) => {
                error = Some(ExtractionError::Nextest(format!(
                    "completed report {}: {cause}",
                    self.path.display()
                )))
            }
            Ok(report) => {
                for suite in report.test_suites {
                    if suite.name.as_str().starts_with("@setup-script:") {
                        setup_failed |= suite
                            .test_cases
                            .iter()
                            .any(|case| matches!(case.status, TestCaseStatus::NonSuccess { .. }));
                        continue;
                    }
                    for case in suite.test_cases {
                        let key = (
                            suite.name.as_str().to_owned(),
                            case.name.as_str().to_owned(),
                        );
                        if !seen.insert(key.clone()) {
                            error.get_or_insert_with(|| {
                                ExtractionError::Nextest(
                                    "completed report repeats a native identity".into(),
                                )
                            });
                            continue;
                        }
                        if case
                            .classname
                            .as_ref()
                            .is_some_and(|class| class.as_str() != suite.name.as_str())
                            || !self.known.contains(&key)
                        {
                            error.get_or_insert_with(|| {
                                ExtractionError::Nextest(
                                    "completed report contains a foreign native test identity"
                                        .into(),
                                )
                            });
                            continue;
                        }
                        if matches!(case.status, TestCaseStatus::Skipped { .. }) {
                            continue;
                        }
                        let Some(identity) = self.selected.get(&key) else {
                            error.get_or_insert_with(|| {
                                ExtractionError::Nextest(
                                    "completed report executed an unselected identity".into(),
                                )
                            });
                            continue;
                        };
                        if !completed.insert(key) {
                            error.get_or_insert_with(|| {
                                ExtractionError::Nextest(
                                    "completed report repeats a selected test".into(),
                                )
                            });
                            continue;
                        }
                        match case.status {
                            TestCaseStatus::Success { flaky_runs } if !flaky_runs.is_empty() => {
                                retries.push(RetrySummary {
                                    identity: identity.clone(),
                                    attempts: flaky_runs.len() + 1,
                                    flaky: true,
                                })
                            }
                            TestCaseStatus::NonSuccess { reruns, .. } => {
                                failed.push(identity.clone());
                                if !reruns.runs.is_empty() {
                                    retries.push(RetrySummary {
                                        identity: identity.clone(),
                                        attempts: reruns.runs.len() + 1,
                                        flaky: false,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        let (native_child_exit, fixture_settled) = match self.fixture_outcome {
            None => (Some(child_exit), None),
            Some(path) => match std::fs::read(&path)
                .map_err(|cause| cause.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<crate::fixtures::CanonicalRunReport>(&bytes)
                        .map_err(|cause| cause.to_string())
                }) {
                Ok(owner) if owner.version == 1 => (owner.runner_exit, Some(owner.fixture_settled)),
                _ => {
                    error.get_or_insert_with(|| {
                        ExtractionError::Nextest(
                            "original fixture-owner result is missing or invalid".into(),
                        )
                    });
                    (None, Some(false))
                }
            },
        };
        let selected = self.selected.len();
        let complete = parsed
            && error.is_none()
            && capture_complete
            && completed.len() == selected
            && native_child_exit.is_some()
            && fixture_settled != Some(false);
        let child_class =
            native_child_exit.map_or(NextestExitClass::Other, NextestExitClass::classify);
        let setup_failed = setup_failed || matches!(child_class, NextestExitClass::SetupFailed);
        let outcome = if setup_failed {
            NextestOutcome::SetupFailed
        } else if !failed.is_empty() || matches!(child_class, NextestExitClass::TestsFailed) {
            NextestOutcome::TestFailed
        } else if selected == 0 {
            NextestOutcome::ZeroMatch
        } else if child_exit != 0 || native_child_exit.is_some_and(|exit| exit != 0) {
            NextestOutcome::RunnerFailed
        } else if complete {
            NextestOutcome::Succeeded
        } else {
            NextestOutcome::Incomplete
        };
        let setup = if setup_failed {
            NextestSetup::FailedByRunner
        } else if parsed && native_child_exit == Some(0) {
            NextestSetup::SucceededByRunner
        } else {
            NextestSetup::Unknown
        };
        let failures = failed
            .iter()
            .map(|identity| NativeFailure {
                identity: identity.clone(),
                report: self.path.clone(),
            })
            .collect();
        (
            NextestSummary {
                child_exit,
                native_child_exit,
                fixture_settled,
                child_class,
                outcome,
                setup,
                complete,
                ignored_policy: self.ignored_policy,
                selected,
                completed: completed.len(),
                failed,
                incomplete_records: usize::from(!complete),
                retries,
            },
            failures,
            error,
        )
    }
}

#[cfg(test)]
mod tests;
