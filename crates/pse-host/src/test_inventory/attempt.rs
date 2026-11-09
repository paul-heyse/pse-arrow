// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected, authenticated native JUnit observations. No execution authority.

use super::{
    CapturedReport, captured_inventory, captured_report_at_limit, digest, invalid, original_verdict,
};
use crate::{
    HostError,
    catalog::description::Information,
    discovery::{Inventory, TestIdentity},
    envelope::{CliInvocationId, Completeness, Envelope, SubjectState},
    investigation::{FrozenTestSelection, Locator, ResultReference},
    invocation::InvocationStore,
    test_workflow::{
        OriginalVerdict,
        execute::{EvidenceRole, EvidenceRoot, evidence},
    },
};
use quick_junit::{FlakyOrRerun, NonSuccessKind, TestCase, TestCaseStatus, TestRerun};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::{self, BufReader},
    num::NonZeroUsize,
};

// Independent read-time bounds, including for original records predating managed quotas.
const MAX_INPUT: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Passed,
    Failed,
    Error,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AttemptRow {
    /// Position in the report writer's chronological attempt sequence.
    pub ordinal: usize,
    pub outcome: AttemptOutcome,
    /// Native duration when retained; this is not exclusive phase time.
    pub duration_seconds: Option<f64>,
    /// The JUnit type, not a reconstructed status, signal, or failure phase.
    pub junit_type: Option<String>,
    /// The corresponding XML fields exist; they are not original process streams.
    pub junit_stdout_present: bool,
    pub junit_stderr_present: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    NotListed,
    NotSelected,
    DifferentCompilerContext,
    AmbiguousCompilerContext,
    /// Nextest emitted a skipped row, positively establishing no execution.
    Unattempted,
    /// No completed row was recorded. This does not distinguish no start from interruption.
    SelectedWithoutRecordedAttempt,
    Completed,
    /// The original report did not commit this group's XML bytes.
    NotCaptured,
    /// The original report commits bytes that are no longer available for reading.
    MissingEvidence,
    /// An earlier writer is retained as original evidence, but not interpreted here.
    UnsupportedWriter,
    /// The original workflow owner recorded a completed release.
    Expired,
    /// There is no original usable inventory/report under the selected invocation.
    Unavailable,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CaseObservation {
    pub state: CaseState,
    pub recorded_attempts: usize,
    /// Last attempt's execution outcome; not the native command verdict.
    pub last_attempt: Option<AttemptOutcome>,
    /// Native XML can record a passed final attempt as a policy failure.
    pub explicit_flaky_policy_failure: bool,
    /// Managed invocation interruption, never inferred as the cause of a case's absence.
    pub invocation_interrupted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// An original report and case-bound attempt continuation.
pub struct AttemptCursor {
    /// Original invocation being observed.
    pub invocation: CliInvocationId,
    /// Digest of the original workflow report.
    pub report_sha256: String,
    /// Exact original test identity or address.
    pub case: TestIdentity,
    /// Digest of the original runner artifact, where available.
    pub junit_sha256: String,
    /// Original next attempt position.
    pub offset: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
/// A bounded page of recorded attempts and separate original verdict.
pub struct AttemptPage {
    /// Original performing invocation.
    pub original_invocation: CliInvocationId,
    /// Original reconciled run verdict, separate from individual observations.
    pub original_verdict: Information<OriginalVerdict>,
    /// Exact original test identity or address.
    pub case: ResultReference,
    /// Coverage and result of the selected case.
    pub case_observation: CaseObservation,
    /// Digest of the original runner artifact, where available.
    pub junit_sha256: Option<String>,
    /// Original attempt records in execution order.
    pub records: Vec<AttemptRow>,
    /// Continuation for the same original case and report.
    pub next: Option<AttemptCursor>,
    /// Size of a whole record that could not fit the requested page.
    pub next_record_bytes: Option<usize>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ComparisonSide {
    /// Original invocation being observed.
    pub invocation: CliInvocationId,
    /// Original reconciled run verdict, separate from individual observations.
    pub original_verdict: Information<OriginalVerdict>,
    pub observation: CaseObservation,
    /// Digest of the original runner artifact, where available.
    pub junit_sha256: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
/// One exact case projected against two original invocations.
pub struct CaseComparison {
    /// Exact original test identity or address.
    pub case: ResultReference,
    /// Original case observation.
    pub original: ComparisonSide,
    /// Observation of the matching case in the second run.
    pub against: ComparisonSide,
}

struct Source {
    original: CapturedReport,
    inventory: Inventory,
    inventory_sha256: String,
    report_sha256: String,
}

impl Source {
    fn open(store: &InvocationStore, id: &CliInvocationId) -> Result<Self, HostError> {
        let original = captured_report_at_limit(store, &id.to_string(), None, Some(MAX_INPUT))?;
        let (inventory, bytes) = captured_inventory(&original, Some(MAX_INPUT))?;
        Ok(Self {
            inventory_sha256: digest(&bytes),
            report_sha256: digest(&original.report),
            original,
            inventory,
        })
    }

    fn case(&self, address: &str) -> Result<(ResultReference, TestIdentity), HostError> {
        let reference = ResultReference::decode(address)?;
        let Locator::FrozenTest {
            invocation, report_sha256, inventory_sha256,
            selection: FrozenTestSelection::Test { identity },
        } = &reference.locator else {
            return Err(invalid("expected an original selected-test reference"));
        };
        if invocation != &self.original.id.to_string()
            || report_sha256 != &self.report_sha256
            || inventory_sha256.as_ref() != Some(&self.inventory_sha256)
            || !self.inventory.selected_tests_iter().any(|test| &test == identity)
        {
            return Err(invalid("case reference differs from this original test and inventory"));
        }
        let identity = identity.clone();
        Ok((reference, identity))
    }

    fn observe(&self, identity: &TestIdentity) -> Result<ObservedCase, HostError> {
        if !crate::extraction::qualified_writer(&self.inventory.nextest_version) {
            return Ok(self.empty(CaseState::UnsupportedWriter));
        }
        let mut outcome = self.empty(CaseState::NotCaptured);
        let commitments = self.original.observation.evidence()
            .map_err(|error| HostError::Workflow(Box::new(error)))?;
        let mut group_reports = commitments.iter().filter(|file| {
            file.root == EvidenceRoot::Workflow
                && file.role == EvidenceRole::RunnerReport
                && file.group == Some(identity.group)
        });
        let Some(commitment) = group_reports.next() else {
            return Ok(outcome);
        };
        if group_reports.next().is_some() {
            return Err(invalid("original compiler group has multiple JUnit commitments"));
        }
        let Some(content) = &commitment.content else {
            return Ok(outcome);
        };
        if content.byte_length > MAX_INPUT as u64 {
            return Err(invalid("original JUnit exceeds the bounded read extent"));
        }
        outcome.junit_sha256 = Some(content.sha256.clone());
        let mut files = match evidence::acquire(
            &self.original.parent, std::slice::from_ref(commitment), |_, _| Ok(()),
        ) {
            Ok(files) => files,
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                outcome.observation.state = CaseState::MissingEvidence;
                return Ok(outcome);
            }
            Err(error) => return Err(error),
        };
        let member = files.first_mut().ok_or_else(|| invalid("original JUnit descriptor is absent"))?;
        let ((state, rows, policy_failure), _) = member.with_reader(|input| {
            let report = quick_junit::Report::deserialize(BufReader::new(input))
                .map_err(io::Error::other)?;
            let mut cases = report.test_suites.into_iter()
                .filter(|suite| suite.name.as_str() == identity.binary_id)
                .flat_map(|suite| suite.test_cases)
                .filter(|case| case.name.as_str() == identity.test_name);
            let Some(case) = cases.next() else {
                return Ok((CaseState::SelectedWithoutRecordedAttempt, Vec::new(), false));
            };
            if cases.next().is_some() || case.classname.as_ref()
                .is_some_and(|class| class.as_str() != identity.binary_id)
            {
                return Err(invalid("original JUnit repeats or misattributes the selected identity"));
            }
            Ok(observe_case(case))
        })?;
        outcome.observation.state = state;
        outcome.observation.recorded_attempts = rows.len();
        outcome.observation.last_attempt = rows.last().map(|row| row.outcome);
        outcome.observation.explicit_flaky_policy_failure = policy_failure;
        outcome.rows = rows;
        Ok(outcome)
    }

    fn empty(&self, state: CaseState) -> ObservedCase {
        ObservedCase::empty(state, self.original.original.subject_state == SubjectState::Cancelled)
    }

    fn comparison(&self, case: ObservedCase) -> ComparisonSide {
        ComparisonSide {
            invocation: self.original.id.clone(),
            original_verdict: original_verdict(&self.original.observation),
            observation: case.observation,
            junit_sha256: case.junit_sha256,
        }
    }
}

struct ObservedCase {
    observation: CaseObservation,
    junit_sha256: Option<String>,
    rows: Vec<AttemptRow>,
}

impl ObservedCase {
    fn empty(state: CaseState, interrupted: bool) -> Self {
        Self {
            observation: CaseObservation {
                state,
                recorded_attempts: 0,
                last_attempt: None,
                explicit_flaky_policy_failure: false,
                invocation_interrupted: interrupted,
            },
            junit_sha256: None,
            rows: Vec::new(),
        }
    }
}

impl CaseState {
    fn has_incomplete_evidence(self) -> bool {
        matches!(
            self,
            Self::SelectedWithoutRecordedAttempt
                | Self::NotCaptured
                | Self::MissingEvidence
                | Self::UnsupportedWriter
                | Self::Expired
                | Self::Unavailable
        )
    }
}

fn non_success(kind: NonSuccessKind) -> AttemptOutcome {
    match kind {
        NonSuccessKind::Failure => AttemptOutcome::Failed,
        NonSuccessKind::Error => AttemptOutcome::Error,
    }
}

fn observe_case(case: TestCase) -> (CaseState, Vec<AttemptRow>, bool) {
    let (outcome, ty, reruns, prior, policy_failure) = match case.status {
        TestCaseStatus::Skipped { .. } => return (CaseState::Unattempted, Vec::new(), false),
        TestCaseStatus::Success { flaky_runs } => (AttemptOutcome::Passed, None, flaky_runs, true, false),
        TestCaseStatus::NonSuccess { kind, ty, reruns, .. } => {
            let flaky = matches!(reruns.kind, FlakyOrRerun::Flaky)
                && ty.as_ref().is_some_and(|value| value.as_str() == "flaky failure");
            (if flaky { AttemptOutcome::Passed } else { non_success(kind) }, ty, reruns.runs, flaky, flaky)
        }
    };
    let primary = AttemptRow {
        ordinal: 0,
        outcome,
        duration_seconds: case.time.map(|time| time.as_secs_f64()),
        junit_type: ty.map(|ty| ty.as_str().to_owned()),
        junit_stdout_present: case.system_out.is_some(),
        junit_stderr_present: case.system_err.is_some(),
    };
    let retry = |run: TestRerun| AttemptRow {
        ordinal: 0,
        outcome: non_success(run.kind),
        duration_seconds: run.time.map(|time| time.as_secs_f64()),
        junit_type: run.ty.map(|ty| ty.as_str().to_owned()),
        junit_stdout_present: run.system_out.is_some(),
        junit_stderr_present: run.system_err.is_some(),
    };
    let count = reruns.len() + 1;
    let mut rows = Vec::with_capacity(count);
    if prior {
        rows.extend(reruns.into_iter().map(retry));
        rows.push(primary);
    } else {
        rows.push(primary);
        rows.extend(reruns.into_iter().map(retry));
    }
    for (index, row) in rows.iter_mut().enumerate() {
        row.ordinal = index + 1;
    }
    (CaseState::Completed, rows, policy_failure)
}

fn selected_invocation(
    store: &InvocationStore, selector: &str, session: Option<&str>,
) -> Result<CliInvocationId, HostError> {
    store.resolve(selector, session)?
        .ok_or_else(|| invalid("original invocation is absent"))
}

/// Read one exact case's recorded native attempts without using XML as a verdict.
pub fn attempts(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    address: &str,
    cursor: Option<&AttemptCursor>,
    limit: NonZeroUsize,
    byte_budget: NonZeroUsize,
) -> Result<Envelope<AttemptPage>, HostError> {
    let id = selected_invocation(store, selector, session)?;
    if let Some(digest) = store.workflow_retirement_digest(&id)? {
        let case = ResultReference::decode(address)?;
        if !matches!(&case.locator, Locator::FrozenTest {
            invocation, report_sha256, selection: FrozenTestSelection::Test { .. }, ..
        } if invocation == id.as_str() && report_sha256 == &digest)
        {
            return Err(invalid("case reference does not name the retired original report"));
        }
        if cursor.is_some() {
            return Err(invalid("original attempt continuation expired with its report"));
        }
        let retired = expired(store, &id, &digest)?;
        let mut response = Envelope::response(
            "inspect tests attempts",
            "the original workflow's attempt evidence expired",
            AttemptPage {
                original_invocation: id,
                original_verdict: retired.original_verdict,
                case,
                case_observation: retired.observation,
                junit_sha256: None,
                records: Vec::new(),
                next: None,
                next_record_bytes: None,
            },
        );
        response.subject_state = SubjectState::Unavailable;
        response.observation.completeness = Completeness::Partial;
        return Ok(response);
    }
    let source = Source::open(store, &id)?;
    let (case, identity) = source.case(address)?;
    let observed = source.observe(&identity)?;
    let offset = match cursor {
        None => 0,
        Some(cursor) if cursor.invocation == source.original.id
            && cursor.report_sha256 == source.report_sha256
            && cursor.case == identity
            && Some(&cursor.junit_sha256) == observed.junit_sha256.as_ref()
            && cursor.offset <= observed.rows.len() => cursor.offset,
        Some(_) => return Err(invalid("attempt continuation no longer names this original case and evidence")),
    };
    let total = observed.rows.len();
    let mut records = Vec::new();
    let mut next_record_bytes = None;
    let mut remaining = byte_budget.get();
    for row in observed.rows.into_iter().skip(offset).take(limit.get()) {
        let extent = serde_json::to_vec(&row).map_err(io::Error::other)?.len();
        let Some(rest) = remaining.checked_sub(extent) else {
            next_record_bytes = Some(extent);
            break;
        };
        remaining = rest;
        records.push(row);
    }
    let next_offset = offset + records.len();
    let next = observed.junit_sha256.as_ref().filter(|_| next_offset < total).map(|sha| AttemptCursor {
        invocation: source.original.id.clone(),
        report_sha256: source.report_sha256.clone(),
        case: identity,
        junit_sha256: sha.clone(),
        offset: next_offset,
    });
    let mut response = Envelope::response(
        "inspect tests attempts",
        format!("{} of {total} recorded native attempts", records.len()),
        AttemptPage {
            original_invocation: source.original.id.clone(),
            original_verdict: original_verdict(&source.original.observation),
            case,
            case_observation: observed.observation,
            junit_sha256: observed.junit_sha256,
            records,
            next,
            next_record_bytes,
        },
    );
    response.observation.scope = "retained_native_test_attempts".into();
    if response.payload.case_observation.state.has_incomplete_evidence() {
        response.observation.completeness = Completeness::Partial;
    }
    response.observation.limitations.push(
        "Native XML reports completed or explicitly skipped cases only; a missing case cannot establish whether it started. XML stream presence is not raw output coverage.".into(),
    );
    Ok(response)
}

fn expired(
    store: &InvocationStore,
    id: &CliInvocationId,
    digest: &str,
) -> Result<ComparisonSide, HostError> {
    let retained = store.result::<serde::de::IgnoredAny, crate::managed::result::ManagedSummary>(id)?;
    let original_verdict = retained.as_ref()
        .and_then(|result| result.payload.workflow.as_ref())
        .map(|workflow| &workflow.terminal)
        .filter(|terminal| terminal.action == crate::test_workflow::execute::Action::Test
            && terminal.invocation == *id
            && terminal.report.sha256 == digest)
        .map_or_else(
            || Information::Unavailable { reason: "the original test verdict is unavailable".into() },
            |terminal| Information::Available(OriginalVerdict {
                invocation: id.clone(),
                outcome: terminal.outcome,
                required_capture_complete: terminal.required_capture_complete,
            }),
        );
    Ok(ComparisonSide {
        invocation: id.clone(),
        original_verdict,
        observation: ObservedCase::empty(
            CaseState::Expired, retained.is_some_and(|result| result.subject_state == SubjectState::Cancelled),
        ).observation,
        junit_sha256: None,
    })
}

fn same_compiler_context(
    left: &nextest_metadata::RustTestSuiteSummary,
    left_group: &crate::discovery::ListedGroup,
    right: &nextest_metadata::RustTestSuiteSummary,
    right_group: &crate::discovery::ListedGroup,
) -> bool {
    fn artifact<'a>(
        suite: &nextest_metadata::RustTestSuiteSummary,
        group: &'a crate::discovery::ListedGroup,
    ) -> Option<(&'a cargo_metadata::Artifact, BTreeSet<&'a str>)> {
        let mut matching = group.build.artifacts.iter().filter(|artifact| {
            artifact.executable.captured == suite.binary.binary_path.as_std_path()
                && artifact.cargo.package_id.repr == suite.binary.package_id
        });
        let artifact = matching.next()?;
        if matching.next().is_some() {
            return None;
        }
        Some((&artifact.cargo, artifact.cargo.features.iter().map(String::as_str).collect()))
    }
    let Some((left_artifact, left_features)) = artifact(left, left_group) else {
        return false;
    };
    let Some((right_artifact, right_features)) = artifact(right, right_group) else {
        return false;
    };
    let (Some(left_platforms), Some(right_platforms)) = (
        &left_group.listing.rust_build_meta.platforms,
        &right_group.listing.rust_build_meta.platforms,
    ) else {
        return false;
    };
    left.binary.build_platform == right.binary.build_platform
        && left_artifact.target.name == right_artifact.target.name
        && left_artifact.target.kind == right_artifact.target.kind
        && left_artifact.profile == right_artifact.profile
        && left_features == right_features
        && left_group.build.resolution.native_capabilities == right_group.build.resolution.native_capabilities
        && left_platforms.host.platform == right_platforms.host.platform
        && left_platforms.targets.iter().map(|target| &target.platform)
            .eq(right_platforms.targets.iter().map(|target| &target.platform))
}

fn comparable_case(source: &Source, identity: &TestIdentity, against: &Source) -> Result<ObservedCase, HostError> {
    let binary = nextest_metadata::RustBinaryId::new(&identity.binary_id);
    let name = nextest_metadata::TestCaseName::new(&identity.test_name);
    let original_group = &source.inventory.groups[identity.group];
    let original_suite = original_group.listing.rust_suites.get(&binary)
        .ok_or_else(|| invalid("original binary disappeared"))?;
    let mut matching = None;
    let mut incompatible = false;
    for (group, entry) in against.inventory.groups.iter().enumerate() {
        let Some(suite) = entry.listing.rust_suites.get(&binary) else { continue; };
        if suite.binary.package_id != identity.package_id {
            continue;
        }
        let Some(case) = suite.test_cases.get(&name) else { continue; };
        if !same_compiler_context(original_suite, original_group, suite, entry) {
            incompatible = true;
            continue;
        }
        if matching.replace((group, case.filter_match.is_match())).is_some() {
            return Ok(against.empty(CaseState::AmbiguousCompilerContext));
        }
    }
    match matching {
        Some((group, true)) => against.observe(&TestIdentity { group, ..identity.clone() }),
        Some((_, false)) => Ok(against.empty(CaseState::NotSelected)),
        None if incompatible => Ok(against.empty(CaseState::DifferentCompilerContext)),
        None => Ok(against.empty(CaseState::NotListed)),
    }
}

/// Compare one exact original case with a second invocation without caching the comparison.
pub fn compare_case(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    address: &str,
    against: &str,
) -> Result<Envelope<CaseComparison>, HostError> {
    let source = Source::open(store, &selected_invocation(store, selector, session)?)?;
    let (case, identity) = source.case(address)?;
    let other_id = match store.resolve(against, session)? {
        Some(id) => id,
        None => CliInvocationId::try_from(against.to_owned())
            .map_err(|_| invalid("comparison requires a known selector or an exact invocation UUID"))?,
    };
    let other = if let Some(digest) = store.workflow_retirement_digest(&other_id)? {
        expired(store, &other_id, &digest)?
    } else {
        match Source::open(store, &other_id) {
        Ok(other) => other.comparison(comparable_case(&source, &identity, &other)?),
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => ComparisonSide {
            invocation: other_id,
            original_verdict: Information::Unavailable { reason: "original report or inventory is no longer available".into() },
            observation: ObservedCase::empty(CaseState::Unavailable, false).observation,
            junit_sha256: None,
        },
        Err(error) => return Err(error),
        }
    };
    let original = source.comparison(source.observe(&identity)?);
    let mut response = Envelope::response(
        "inspect tests compare", "comparison of two original case observations",
        CaseComparison { case, original, against: other },
    );
    response.observation.scope = "retained_native_test_case_comparison".into();
    if response.payload.original.observation.state.has_incomplete_evidence()
        || response.payload.against.observation.state.has_incomplete_evidence()
    {
        response.observation.completeness = Completeness::Partial;
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observe_junit_case(xml: &str) -> (CaseState, Vec<AttemptRow>, bool) {
        let input = format!(
            r#"<testsuites tests="1" failures="0" errors="0"><testsuite name="binary" tests="1" failures="0" errors="0"><testcase classname="binary" name="case" time="2">{xml}</testcase></testsuite></testsuites>"#
        );
        let mut report = quick_junit::Report::deserialize(input.as_bytes()).unwrap();
        let case = report.test_suites.remove(0).test_cases.remove(0);
        observe_case(case)
    }

    #[test]
    fn native_primary_failure_precedes_rerun_but_flaky_pass_follows_prior_failures() {
        let (_, failed, policy) = observe_junit_case(
            r#"<failure type="test failure"/><rerunError time="4" type="execution failure"/>"#,
        );
        assert!(!policy);
        assert_eq!(failed[0].outcome, AttemptOutcome::Failed);
        assert_eq!(failed[0].duration_seconds, Some(2.0));
        assert_eq!(failed[1].outcome, AttemptOutcome::Error);
        assert_eq!(failed[1].duration_seconds, Some(4.0));
        let (_, flaky, policy) = observe_junit_case(
            r#"<flakyFailure time="4" type="test failure"/><system-out>final</system-out>"#,
        );
        assert!(!policy);
        assert_eq!(flaky[0].ordinal, 1);
        assert_eq!(flaky[0].outcome, AttemptOutcome::Failed);
        assert_eq!(flaky[0].duration_seconds, Some(4.0));
        assert_eq!(flaky[1].ordinal, 2);
        assert_eq!(flaky[1].outcome, AttemptOutcome::Passed);
        assert!(flaky[1].junit_stdout_present);
    }

    #[test]
    fn native_flaky_policy_failure_preserves_final_execution_pass_and_skip_is_not_attempted() {
        let (state, rows, policy) = observe_junit_case(
            r#"<failure type="flaky failure"/><flakyFailure time="1" type="test failure"/>"#,
        );
        assert_eq!(state, CaseState::Completed);
        assert!(policy);
        assert_eq!(rows[0].outcome, AttemptOutcome::Failed);
        assert_eq!(rows[1].outcome, AttemptOutcome::Passed);
        let (state, rows, _) = observe_junit_case("<skipped/>");
        assert_eq!(state, CaseState::Unattempted);
        assert!(rows.is_empty());
    }
}
