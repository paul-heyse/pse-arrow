// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native completed-report reconciliation; no test-event lifecycle is reconstructed.
use super::{
    ExtractionError, NextestExitClass, NextestOutcome, NextestSetup, NextestSummary,
    TargetSetupRefusal,
};
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

/// Qualified Nextest writer for interpreting an original native report.
pub(crate) fn qualified_writer(version: &str) -> bool {
    let mut words = version.split_whitespace();
    words.next() == Some("cargo-nextest") && words.next() == Some("0.9.151")
}

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
    /// Native XML failure message, independent of raw stream bytes.
    pub message: Option<String>,
    /// Native XML assertion description, independent of raw stream bytes.
    pub description: Option<String>,
    /// Distinct target-owned setup refusal phases. These attempts never reached assertions.
    pub target_setup_refusals: Vec<TargetSetupRefusal>,
}
/// Captured population and invocation-private report location.
#[derive(Debug)]
pub struct CompletedReport {
    path: PathBuf,
    selected: BTreeMap<(String, String), TestIdentity>,
    known: BTreeSet<(String, String)>,
    ignored_policy: IgnoredPolicy,
    fixture_outcome: Option<PathBuf>,
    target_owned: BTreeSet<TestIdentity>,
    target_settlement: Option<PathBuf>,
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
    /// Accept setup evidence only for selected cases already matched to a performing owner.
    pub(crate) fn with_target_owned(mut self, cases: BTreeSet<TestIdentity>) -> Self {
        self.target_owned = cases;
        self
    }
    pub(crate) fn with_target_settlement(mut self, directory: PathBuf) -> Self {
        self.target_settlement = Some(directory);
        self
    }
    /// Bind the observed native population before execution.
    pub fn new(
        inventory: &Inventory,
        group: usize,
        path: PathBuf,
    ) -> Result<Self, ExtractionError> {
        if !inventory.capture_complete() || !qualified_writer(&inventory.nextest_version) {
            return Err(ExtractionError::Nextest(
                "completed reports require qualified complete nextest 0.9.151 inventory".into(),
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
                || !crate::discovery::suite_selection_complete(suite)
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
            target_owned: BTreeSet::new(),
            target_settlement: None,
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
        let mut failures = Vec::new();
        let mut retries = Vec::new();
        let mut runner_setup_failed = false;
        let mut target_setup_failed = false;
        let mut native_test_failed = false;
        let mut error = None;
        let mut run = None;
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
                run = report.uuid;
                for suite in report.test_suites {
                    if suite.name.as_str().starts_with("@setup-script:") {
                        runner_setup_failed |= suite
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
                            TestCaseStatus::NonSuccess {
                                reruns,
                                message,
                                description,
                                ..
                            } => {
                                failed.push(identity.clone());
                                let mut target_setup_refusals = Vec::new();
                                if self.target_owned.contains(identity) {
                                    let outputs = std::iter::once((
                                        case.system_out.as_ref(),
                                        case.system_err.as_ref(),
                                    ))
                                    .chain(
                                        reruns.runs.iter().map(|run| {
                                            (run.system_out.as_ref(), run.system_err.as_ref())
                                        }),
                                    );
                                    for (stdout, stderr) in outputs {
                                        if let Some(refusal) =
                                            TargetSetupRefusal::from_captured_output(stdout, stderr)
                                        {
                                            target_setup_failed = true;
                                            if !target_setup_refusals.contains(&refusal) {
                                                target_setup_refusals.push(refusal);
                                            }
                                        } else {
                                            native_test_failed = true;
                                        }
                                    }
                                } else {
                                    native_test_failed = true;
                                }
                                failures.push(NativeFailure {
                                    identity: identity.clone(),
                                    report: self.path.clone(),
                                    message: message.map(|value| value.as_str().to_owned()),
                                    description: description.map(|value| value.as_str().to_owned()),
                                    target_setup_refusals,
                                });
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
        let target_effect_account = self.target_settlement.as_ref().and_then(|directory| {
            if !capture_complete || error.is_some() {
                return None;
            }
            let run = run?.to_string().parse().ok()?;
            let mut attempts = BTreeSet::new();
            for identity in &self.target_owned {
                if !completed.contains(&(identity.binary_id.clone(), identity.test_name.clone())) {
                    return None;
                }
                let count = retries
                    .iter()
                    .find(|retry| &retry.identity == identity)
                    .map_or(1, |retry| retry.attempts);
                attempts.extend((1..=count).map(|attempt| {
                    (
                        identity.binary_id.clone(),
                        identity.test_name.clone(),
                        attempt,
                    )
                }));
            }
            crate::fixtures::target_settlement::observe(directory, Some((run, &attempts))).ok()
        });
        let target_fixture_settled = self.target_settlement.as_ref().map(|_| {
            target_effect_account.as_ref().is_some_and(|(settled, _)| *settled)
        });
        let target_fixture_effects = target_effect_account.map(|(_, effects)| effects);
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
        runner_setup_failed |= matches!(child_class, NextestExitClass::SetupFailed);
        let outcome = if runner_setup_failed || target_setup_failed && !native_test_failed {
            NextestOutcome::SetupFailed
        } else if native_test_failed || matches!(child_class, NextestExitClass::TestsFailed) {
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
        let setup = if runner_setup_failed {
            NextestSetup::FailedByRunner
        } else if target_setup_failed {
            NextestSetup::FailedByTargetOwner
        } else if parsed && native_child_exit == Some(0) {
            NextestSetup::SucceededByRunner
        } else {
            NextestSetup::Unknown
        };
        (
            NextestSummary {
                child_exit,
                native_child_exit,
                fixture_settled,
                target_fixture_settled,
                target_fixture_effects,
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
