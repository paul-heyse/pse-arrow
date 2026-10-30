// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One strict terminal transport; decoding it cannot construct native verdict capabilities.
use crate::{
    envelope::{CliInvocationId, ErrorRecord, RedactionPolicy},
    managed::payload::PayloadPolicy,
    selection,
    test_workflow::{CompilerObservation, Observation, WorkflowError},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    os::unix::fs::OpenOptionsExt as _,
    path::Path,
};
pub(super) const PROTOCOL: &str = "pse.test-workflow";
const REPORT: &str = "report.json";

/// Exact operation; a list is never interpreted as a successful test execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Source build/list only.
    List,
    /// Reconcile actual selected test execution.
    Test,
    /// Reconcile actual compiler check execution.
    Check,
}
impl Action {
    /// One authoritative public command identity across transport and native dispatch.
    pub const fn command(self) -> &'static str {
        match self {
            Self::List => "tests",
            Self::Test => "test",
            Self::Check => "check",
        }
    }
    /// Recognize precisely the fixed performing workflow commands.
    pub fn from_command(command: &str) -> Option<Self> {
        [Self::List, Self::Test, Self::Check]
            .into_iter()
            .find(|action| action.command() == command)
    }
}
/// Protocol/view decoding bounds, independent of population, CPU and raw-log custody.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    /// One native protocol record decoding extent.
    pub record_bytes: usize,
    /// Optional duplicate-grouping cache entries.
    pub grouping_capacity: usize,
    /// Whole failure-reference records in the projected report.
    pub failure_records: usize,
}
/// Private operation input authenticated by the fixed helper's actual managed owner.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Helper wire contract.
    pub version: u32,
    /// Original performing invocation.
    pub invocation: CliInvocationId,
    /// Exact operation.
    pub action: Action,
    /// Same selection model as explain/admission.
    pub request: selection::Request,
    /// Original nextest policy profile.
    pub profile: String,
    /// Existing raw write/drain observation policy, not a workload allocation.
    pub policy: PayloadPolicy,
    /// Existing parser/view bounds.
    pub bounds: Bounds,
}
/// Helper reconciliation, independent of original managed physical settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Complete actual source/list population, without a workload pass.
    Listed,
    /// Complete successful Cargo check observations.
    CheckSucceeded,
    /// Nonempty fully reconciled test execution.
    TestsSucceeded,
    /// Known native compiler/test/setup/runner failure.
    Failed,
    /// No selected tests; never a test pass.
    ZeroMatch,
    /// Actual prerequisite owner did not grant readiness.
    Pending,
    /// Required observation/capture/terminal information is incomplete.
    Incomplete,
}
impl Outcome {
    /// Successful command semantics for this exact action, never transport or child0 alone.
    pub fn successful(self, action: Action) -> bool {
        matches!(
            (action, self),
            (Action::List, Self::Listed)
                | (Action::Check, Self::CheckSucceeded)
                | (Action::Test, Self::TestsSucceeded)
        )
    }
}
/// A typed unfulfilled native owner, never an implicit prepare/skip.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Pending {
    /// Actual reached native capabilities could not be selected.
    Native {
        /// Required capabilities from admitted compiler units.
        capabilities: std::collections::BTreeSet<String>,
    },
    /// Exact selected fixture still requires its original lifecycle owner.
    Fixture(Box<FixturePending>),
}
/// Original fixture declaration and its passive readiness form one pending-owner account.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FixturePending {
    /// Original applied declaration.
    pub requirement: crate::fixtures::Requirement,
    /// Passive original input-owner account.
    pub readiness: crate::fixtures::Readiness,
}
/// Native phase, with no second process/native-budget owner.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PhaseKind {
    /// Captured metadata input.
    Metadata,
    /// Source build/list phase.
    Discovery,
    /// Native named-group membership query.
    GroupPolicy,
    /// Actual Cargo check.
    Check,
    /// Actual reused-binary nextest execution.
    Test,
}
/// Independent raw-custody and original child facts for a phase.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Phase {
    /// Phase identity.
    pub kind: PhaseKind,
    /// Compiler group when applicable.
    pub group: Option<usize>,
    /// Fixed relative raw stdout path beneath helper evidence.
    pub stdout: String,
    /// Fixed relative raw stderr path beneath helper evidence.
    pub stderr: String,
    /// Original waited child exit, never guessed.
    pub child_exit: Option<i32>,
    /// Every raw byte/read/write/synchronization completed.
    pub capture_complete: bool,
    /// Original required stdout custody and byte counts.
    pub stdout_capture: Option<crate::managed::result::StreamSummary>,
    /// Original required stderr custody and byte counts.
    pub stderr_capture: Option<crate::managed::result::StreamSummary>,
}
/// Native private-summary projection; Deserialize deliberately remains absent.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GroupObservation {
    /// Original compiler interpreter result.
    Check {
        /// Exact admitted group.
        group: usize,
        /// Owner-constructed compiler verdict.
        observation: CompilerObservation,
    },
    /// Original selected-population nextest interpreter result.
    Test {
        /// Exact listed group.
        group: usize,
        /// Owner-constructed native test verdict.
        observation: Observation,
        /// Caller-selected whole failure-reference view extent.
        failure_budget: usize,
        /// Failure references omitted only from this view; native counts and raw output remain complete.
        omitted_failure_records: usize,
    },
}
/// Durable performing-helper report. Private native summaries are never decoded into passes.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Helper report version.
    pub version: u32,
    /// Original invocation association.
    pub invocation: CliInvocationId,
    /// Actual action.
    pub action: Action,
    /// Helper's reconciled operation result.
    pub outcome: Outcome,
    /// Normalized original selection if available.
    pub selection: Option<selection::Plan>,
    /// Original correctness-owned groups.
    pub admission: Vec<crate::correctness::Resolution>,
    /// Actual immutable inventory reference.
    pub inventory: Option<String>,
    /// Count of actual matching captured tests.
    pub selected_tests: Option<usize>,
    /// Required prerequisite refusals.
    pub pending: Vec<Pending>,
    /// Original child/raw phase facts.
    pub phases: Vec<Phase>,
    /// Owner-constructed native observations.
    pub observations: Vec<GroupObservation>,
    /// All required inner raw custody completed.
    pub required_capture_complete: bool,
    /// Original typed failure projections.
    pub failures: Vec<ErrorRecord>,
    /// Native causes retained through terminal publication.
    #[serde(skip)]
    #[schemars(skip)]
    pub causes: Vec<WorkflowError>,
}
impl Report {
    pub(super) fn new(input: &Input) -> Self {
        Self {
            version: 1,
            invocation: input.invocation.clone(),
            action: input.action,
            outcome: Outcome::Incomplete,
            selection: None,
            admission: Vec::new(),
            inventory: None,
            selected_tests: None,
            pending: Vec::new(),
            phases: Vec::new(),
            observations: Vec::new(),
            required_capture_complete: true,
            failures: Vec::new(),
            causes: Vec::new(),
        }
    }
    pub(super) fn failure(&mut self, error: WorkflowError) {
        self.failures.push(ErrorRecord::from_typed(
            &error,
            &RedactionPolicy::from_environment(),
        ));
        self.causes.push(error);
    }
    /// Publish the complete report before emitting a terminal reference. Source failure leaves
    /// original evidence in place; this neither emits stdout nor claims physical settlement.
    pub fn publish(&self, destination: &Path) -> Result<TerminalRecord, WorkflowError> {
        let bytes = serde_json::to_vec(self).map_err(crate::discovery::DiscoveryError::from)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o400)
            .open(destination.join(REPORT))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::File::open(destination)?.sync_all()?;
        Ok(TerminalRecord {
            protocol: PROTOCOL.into(),
            version: 1,
            invocation: self.invocation.clone(),
            action: self.action,
            outcome: self.outcome,
            report: ReportReference {
                path: REPORT.into(),
                sha256: Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
                bytes: bytes.len() as u64,
            },
            required_capture_complete: self.required_capture_complete,
        })
    }
}
/// Exact durable original report address, not arbitrary filesystem authority.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportReference {
    /// Fixed relative report file.
    pub path: String,
    /// Original exact byte identity.
    pub sha256: String,
    /// Original complete byte count.
    pub bytes: u64,
}
/// Exactly one final JSONL frame. Root validates this original helper transport separately
/// from raw capture, original native payload identity and physical descendant settlement.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalRecord {
    /// Exact performing protocol.
    pub protocol: String,
    /// Supported wire version.
    pub version: u32,
    /// Original invoking owner.
    pub invocation: CliInvocationId,
    /// Exact expected operation.
    pub action: Action,
    /// Helper reconciliation, never a manager capability.
    pub outcome: Outcome,
    /// Already durable original report.
    pub report: ReportReference,
    /// Inner required raw observation completeness.
    pub required_capture_complete: bool,
}
impl TerminalRecord {
    /// Check wire/correlation/fixed-address facts without constructing native verdict objects.
    pub fn validate(
        &self,
        invocation: &CliInvocationId,
        action: Action,
    ) -> Result<(), WorkflowError> {
        if self.protocol != PROTOCOL
            || self.version != 1
            || &self.invocation != invocation
            || self.action != action
            || self.report.path != REPORT
            || self.report.bytes == 0
            || self.report.sha256.len() != 64
            || !self
                .report
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(WorkflowError::Invalid(
                "workflow terminal identity/report association mismatch".into(),
            ));
        }
        if matches!(
            self.outcome,
            Outcome::Listed | Outcome::CheckSucceeded | Outcome::TestsSucceeded
        ) && (!self.outcome.successful(action) || !self.required_capture_complete)
        {
            return Err(WorkflowError::Invalid(
                "workflow success lacks action-matched complete raw capture".into(),
            ));
        }
        Ok(())
    }
    /// The helper's conventional command exit. Pending/zero/incomplete never pass.
    pub fn helper_exit(&self) -> i32 {
        if self.outcome.successful(self.action) && self.required_capture_complete {
            0
        } else {
            match self.outcome {
                Outcome::Failed | Outcome::ZeroMatch => 1,
                _ => 125,
            }
        }
    }
    /// Emit exactly one newline-terminated final record; all native output is retained by phase.
    pub fn write(&self, output: &mut impl std::io::Write) -> Result<(), WorkflowError> {
        serde_json::to_writer(&mut *output, self)
            .map_err(crate::discovery::DiscoveryError::from)?;
        output.write_all(b"\n")?;
        Ok(())
    }
}
