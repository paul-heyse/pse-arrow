// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One strict terminal transport; decoding it cannot construct native verdict capabilities.
use super::evidence::{self, EvidenceFile, EvidenceRole, EvidenceRoot, SourceFile};
use crate::{
    envelope::{CliInvocationId, ErrorRecord, RedactionPolicy},
    managed::payload::PayloadPolicy,
    selection,
    test_workflow::{CompilerObservation, Observation, WorkflowError},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::path::Path;
pub(super) const PROTOCOL: &str = "pse.test-workflow";
const REPORT: &str = "report.json";
const INVENTORY: &str = "inventory/inventory.json";

pse_contract::projection_enum! {
/// Exact operation; a list is never interpreted as a successful test execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    projection: { name: "WorkflowAction", rust_path: crate::test_workflow::execute::Action, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Source build/list only.
    List => 1;
    /// Reconcile actual selected test execution.
    Test => 2;
    /// Reconcile actual compiler check execution.
    Check => 3;
    /// Reconcile native rustdoc diagnostics over the selected authored library.
    Rustdoc => 4;
}
}
impl Action {
    /// One authoritative public command identity across transport and native dispatch.
    pub const fn command(self) -> &'static str {
        match self {
            Self::List => "tests",
            Self::Test => "test",
            Self::Check => "check",
            Self::Rustdoc => "api check-docs",
        }
    }
    /// Recognize precisely the fixed performing workflow commands.
    pub fn from_command(command: &str) -> Option<Self> {
        [Self::List, Self::Test, Self::Check, Self::Rustdoc]
            .into_iter()
            .find(|action| action.command() == command)
    }
    /// Compiler-only workflows share native diagnostic capture, never a test verdict.
    pub const fn is_compiler(self) -> bool {
        matches!(self, Self::Check | Self::Rustdoc)
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
    /// Request native Cargo timing diagnostics; emitted paths remain mutable Cargo outputs.
    #[serde(default)]
    pub timings: bool,
    /// Existing raw write/drain observation policy, not a workload allocation.
    pub policy: PayloadPolicy,
    /// Existing parser/view bounds.
    pub bounds: Bounds,
}
impl Input {
    /// Shared passive input-family validation. This does not admit or execute the request.
    pub(crate) fn validate_original(
        &self,
        id: &CliInvocationId,
        command: &str,
    ) -> Result<(), WorkflowError> {
        if self.version != 1
            || self.invocation != *id
            || self.action.command() != command
            || self.bounds.record_bytes == 0
            || self.bounds.grouping_capacity == 0
            || self.policy.write.is_zero()
            || self.policy.drain.is_zero()
        {
            return Err(WorkflowError::Invalid(
                "workflow input version, correlation, action or observation policy is invalid"
                    .into(),
            ));
        }
        Ok(())
    }
}
pse_contract::projection_enum! {
/// Helper reconciliation, independent of original managed physical settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    projection: { name: "WorkflowOutcome", rust_path: crate::test_workflow::execute::Outcome, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Complete actual source/list population, without a workload pass.
    Listed => 1;
    /// Complete successful Cargo check observations.
    CheckSucceeded => 2;
    /// Nonempty fully reconciled test execution.
    TestsSucceeded => 3;
    /// Known native compiler/test/setup/runner failure.
    Failed => 4;
    /// No selected tests; never a test pass.
    ZeroMatch => 5;
    /// Actual prerequisite owner did not grant readiness.
    Pending => 6;
    /// Required observation/capture/terminal information is incomplete.
    Incomplete => 7;
}
}
impl Outcome {
    /// Successful command semantics for this exact action, never transport or child0 alone.
    pub fn successful(self, action: Action) -> bool {
        matches!(
            (action, self),
            (Action::List, Self::Listed)
                | (Action::Check, Self::CheckSucceeded)
                | (Action::Rustdoc, Self::CheckSucceeded)
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
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Phase {
    /// Exact original native discovery subphase, such as version/build/list, when applicable.
    pub native_stage: Option<String>,
    /// Inclusive wall time from phase entry through child wait and required stream drain.
    /// Absent when the original producer did not capture a duration; never CPU time.
    pub elapsed_seconds: Option<f64>,
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
    /// Original inventory bytes committed by this report's interpretation.
    pub inventory_content: Option<ContentIdentity>,
    /// Once-sealed original dependencies; absent only in historical interpretations.
    pub evidence: Option<Vec<EvidenceFile>>,
    /// Count of actual matching captured tests.
    pub selected_tests: Option<usize>,
    /// Original declarations under admitted compiler contexts; passive, never readiness.
    pub fixture_requirements: Vec<crate::fixtures::Requirement>,
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
    /// Actual source promises before final original dependency sealing.
    #[serde(skip)]
    #[schemars(skip)]
    pub(super) evidence_sources: Vec<SourceFile>,
    /// Startup's actual accepted input binding is rechecked before the original cutoff.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) input_cutoff: Option<Result<ContentIdentity, crate::HostError>>,
    /// First serialized original facts stay fixed across failed publication retries.
    #[serde(skip)]
    #[schemars(skip)]
    pub(super) sealed_content: Option<ContentIdentity>,
}
impl Report {
    pub(super) fn new(input: &Input) -> Self {
        Self {
            version: 3,
            invocation: input.invocation.clone(),
            action: input.action,
            outcome: Outcome::Incomplete,
            selection: None,
            admission: Vec::new(),
            inventory: None,
            inventory_content: None,
            evidence: None,
            selected_tests: None,
            fixture_requirements: Vec::new(),
            pending: Vec::new(),
            phases: Vec::new(),
            observations: Vec::new(),
            required_capture_complete: true,
            failures: Vec::new(),
            causes: Vec::new(),
            evidence_sources: Vec::new(),
            input_cutoff: None,
            sealed_content: None,
        }
    }
    pub(super) fn failure(&mut self, error: WorkflowError) {
        self.failures.push(ErrorRecord::from_typed(
            &error,
            &RedactionPolicy::from_environment(),
        ));
        self.causes.push(error);
    }
    /// Record the native discovery owner's exact private inventory before using it.
    pub(super) fn record_inventory(&mut self, destination: &Path) -> Result<(), WorkflowError> {
        let bytes = crate::test_inventory::read_original_inventory(destination)?;
        self.inventory = Some(INVENTORY.into());
        self.inventory_content = Some(ContentIdentity::of(&bytes));
        Ok(())
    }
    /// Publish the complete report before emitting a terminal reference. Source failure leaves
    /// original evidence in place; this neither emits stdout nor claims physical settlement.
    pub fn publish(&mut self, destination: &Path) -> Result<TerminalRecord, WorkflowError> {
        if self.version != 3 {
            return Err(WorkflowError::Invalid(
                "current performer requires report interpretation 3".into(),
            ));
        }
        crate::test_inventory::checked_parent(destination)?;
        if self.inventory.is_some() {
            let bytes = crate::test_inventory::read_original_inventory(destination)?;
            self.inventory_content
                .as_ref()
                .ok_or_else(|| {
                    WorkflowError::Invalid("original report has no inventory commitment".into())
                })?
                .validate_content(&bytes, "original inventory")?;
        }
        if self.evidence.is_none() {
            let (mut files, mut causes) = evidence::seal(destination, self.original_sources());
            for file in &mut files {
                let retained = self.phases.iter().find_map(|phase| {
                    if phase.stdout == file.path && file.role == EvidenceRole::Stdout {
                        phase.stdout_capture.as_ref()
                    } else if phase.stderr == file.path && file.role == EvidenceRole::Stderr {
                        phase.stderr_capture.as_ref()
                    } else {
                        None
                    }
                });
                if file
                    .content
                    .as_ref()
                    .zip(retained)
                    .is_some_and(|(content, capture)| content.byte_length != capture.bytes_retained)
                {
                    let error = crate::HostError::Ownership(format!(
                        "original workflow stream {} differs from the actual retained capture extent",
                        file.path
                    ));
                    file.content = None;
                    file.omission = Some(super::EvidenceOmission::Refused);
                    file.failure = Some(ErrorRecord::from_typed(
                        &error,
                        &RedactionPolicy::from_environment(),
                    ));
                    causes.push(error.into());
                }
            }
            if let Some(input) = files
                .iter_mut()
                .find(|file| file.role == EvidenceRole::Input)
            {
                if let Some(Err(error)) = &self.input_cutoff {
                    input.omission = Some(super::EvidenceOmission::Refused);
                    input.failure = Some(ErrorRecord::from_typed(
                        error,
                        &RedactionPolicy::from_environment(),
                    ));
                } else if let Some(Ok(accepted)) = &self.input_cutoff {
                    // Bind the same snapshot which passed startup's original binding,
                    // not a second path read which could select replacement input.
                    input.content = Some(accepted.clone());
                    input.omission = None;
                    input.failure = None;
                }
            }
            self.causes.extend(causes);
            self.evidence = Some(files);
        }
        let observation = ReportObservation::of(self);
        observation.require_current()?;
        if self.inventory.is_some() {
            let bytes = crate::test_inventory::read_original_inventory(destination)?;
            observation.validate_inventory(&bytes)?;
        }
        let bytes = serde_json::to_vec(self).map_err(crate::discovery::DiscoveryError::from)?;
        let content = ContentIdentity::of(&bytes);
        if self
            .sealed_content
            .as_ref()
            .is_some_and(|original| original != &content)
        {
            return Err(WorkflowError::Invalid(
                "original report facts changed after dependency sealing".into(),
            ));
        }
        self.sealed_content = Some(content.clone());
        evidence::revalidate(
            destination,
            self.evidence.as_deref().ok_or_else(|| {
                WorkflowError::Invalid("original evidence sealing is absent".into())
            })?,
        )?;
        crate::invocation::publish_bytes(&destination.join(REPORT), &bytes, None, 0o400)?;
        Ok(TerminalRecord {
            protocol: PROTOCOL.into(),
            version: 1,
            invocation: self.invocation.clone(),
            action: self.action,
            outcome: self.outcome,
            report: ReportReference {
                path: REPORT.into(),
                sha256: content.sha256,
                byte_length: content.byte_length,
            },
            required_capture_complete: self.required_capture_complete,
        })
    }
    /// Register actual source promises, including work which may fail before an observation.
    pub(super) fn expect_file(
        &mut self,
        path: String,
        role: EvidenceRole,
        group: Option<usize>,
        settled: bool,
    ) {
        self.evidence_sources.push(SourceFile {
            root: EvidenceRoot::Workflow,
            path,
            role,
            group,
            settled,
        });
    }
    pub(super) fn expect_path(
        &mut self,
        destination: &Path,
        path: &Path,
        role: EvidenceRole,
        group: Option<usize>,
        settled: bool,
    ) -> Result<(), WorkflowError> {
        let relative = path
            .strip_prefix(destination)
            .ok()
            .and_then(Path::to_str)
            .filter(|path| evidence::relative_file(path))
            .ok_or_else(|| {
                WorkflowError::Invalid(
                    "original dependency is outside its native workflow root".into(),
                )
            })?;
        self.expect_file(relative.into(), role, group, settled);
        Ok(())
    }
    fn original_sources(&self) -> Vec<SourceFile> {
        use std::collections::BTreeMap;
        let mut sources = BTreeMap::new();
        let mut add = |source: SourceFile| {
            sources.insert((source.root, source.path.clone()), source);
        };
        // The original input is sealed from its authenticated snapshot separately.
        add(SourceFile {
            root: EvidenceRoot::Invocation,
            path: "workflow-input.json".into(),
            role: EvidenceRole::Input,
            group: None,
            settled: false,
        });
        for source in &self.evidence_sources {
            let settled = if matches!(
                source.role,
                EvidenceRole::RunnerReport
                    | EvidenceRole::CompilerContext
                    | EvidenceRole::FixtureOutcome
            ) {
                self.phases.iter().any(|phase| phase.group == source.group && matches!(phase.kind, PhaseKind::Test | PhaseKind::Check) && phase.child_exit.is_some() && phase.capture_complete)
                    && (source.role != EvidenceRole::RunnerReport || self.observations.iter().any(|observation| matches!(observation, GroupObservation::Test {group,observation,..} if Some(*group)==source.group && observation.summary.native_child_exit().is_some())))
            } else {
                source.settled
            };
            add(SourceFile {
                root: source.root,
                path: source.path.clone(),
                role: source.role,
                group: source.group,
                settled,
            });
        }
        if self.inventory.is_some() {
            add(SourceFile {
                root: EvidenceRoot::Workflow,
                path: INVENTORY.into(),
                role: EvidenceRole::Inventory,
                group: None,
                settled: true,
            });
        }
        for phase in &self.phases {
            for (path, role) in [
                (&phase.stdout, EvidenceRole::Stdout),
                (&phase.stderr, EvidenceRole::Stderr),
            ] {
                add(SourceFile {
                    root: EvidenceRoot::Workflow,
                    path: path.clone(),
                    role,
                    group: phase.group,
                    settled: phase.capture_complete,
                });
            }
            if matches!(phase.kind, PhaseKind::Discovery) {
                if let Some(group) = phase.group {
                    if phase.native_stage.as_deref() == Some("build") {
                        add(SourceFile {
                            root: EvidenceRoot::Workflow,
                            path: format!("inventory/group-{group}/build/build.json"),
                            role: EvidenceRole::CompilerContext,
                            group: Some(group),
                            settled: phase.child_exit.is_some() && phase.capture_complete,
                        });
                        if phase.child_exit == Some(0) {
                            add(SourceFile {
                                root: EvidenceRoot::Workflow,
                                path: format!("inventory/group-{group}/cargo-metadata.json"),
                                role: EvidenceRole::DiscoveryContext,
                                group: Some(group),
                                settled: phase.capture_complete,
                            });
                        }
                    }
                    if phase.native_stage.as_deref() == Some("list") {
                        add(SourceFile {
                            root: EvidenceRoot::Workflow,
                            path: format!("inventory/group-{group}/binaries.json"),
                            role: EvidenceRole::DiscoveryContext,
                            group: Some(group),
                            settled: true,
                        });
                    }
                    add(SourceFile {
                        root: EvidenceRoot::Workflow,
                        path: "inventory/nextest.toml".into(),
                        role: EvidenceRole::RunnerConfiguration,
                        group: None,
                        settled: true,
                    });
                }
            }
        }
        sources.into_values().collect()
    }
}
impl pse_contract::boundary_projection::ProjectionType for Report {
    fn projection(
        graph: &mut pse_contract::boundary_projection::ProjectionGraph,
    ) -> Result<
        pse_contract::boundary_projection::TypeRef,
        pse_contract::boundary_projection::ProjectionError,
    > {
        use pse_contract::boundary_projection::{Direction, Field, Shape};
        let root = graph.declare::<Self>(
            "NativeWorkflowReport",
            file!(),
            1,
            Direction::Output,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![Field::of::<ReportObservation>(
                    graph,
                    "observation",
                    1,
                    Direction::Output,
                    vec![],
                )?]))
            },
        )?;
        graph.public_path::<Self>(&root, "crate::test_workflow::execute::Report")?;
        graph.observation_record::<Self>(&root)?;
        Ok(root)
    }
}
impl pse_contract::boundary_projection::ProjectionObservationRecord for Report {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "observation")];
    type Fields = (ReportObservation,);
    fn observation_fields(
        &self,
    ) -> Result<Self::Fields, pse_contract::boundary_projection::AdapterError> {
        Ok((ReportObservation::of(self),))
    }
}
impl pse_contract::boundary_projection::ValidateProjection for Report {
    fn validate_at(
        &self,
        path: &str,
    ) -> Result<(), pse_contract::boundary_projection::ValidationError> {
        self.failures
            .validate_at(&pse_contract::boundary_projection::field_path(
                path,
                "observation.failures",
            ))?;
        self.inventory_content
            .validate_at(&pse_contract::boundary_projection::field_path(
                path,
                "observation.inventory_content",
            ))?;
        self.evidence
            .validate_at(&pse_contract::boundary_projection::field_path(
                path,
                "observation.evidence",
            ))
    }
}
pse_contract::projection_struct! {
/// Exact durable original report address, not arbitrary filesystem authority.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportReference {
    projection: { name: "WorkflowReportReference", rust_path: crate::test_workflow::execute::ReportReference, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Fixed relative report file.
    pub path: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original exact byte identity.
    pub sha256: String => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original complete byte count.
    #[serde(rename = "bytes")]
    pub byte_length: u64 => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ReportReference {
    /// Shared content commitment check for streamed live reads and complete retained bytes.
    pub(crate) fn validate_content(
        &self,
        byte_length: u64,
        sha256: &str,
    ) -> Result<(), WorkflowError> {
        if byte_length != self.byte_length || sha256 != self.sha256 {
            return Err(WorkflowError::Invalid(
                "original report content differs from its terminal extent or digest".into(),
            ));
        }
        Ok(())
    }
}
pse_contract::projection_struct! {
/// Exact original evidence content, distinct from its source-owned file address.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContentIdentity {
    projection: { name: "ContentIdentity", rust_path: crate::test_workflow::execute::ContentIdentity, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// SHA-256 of the complete original bytes, not their re-encoding.
    pub sha256: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [pse_contract::boundary_projection::LocalRule::Utf8Bytes { min:64, max:64 }] };
    /// Extent of those same original bytes.
    #[serde(rename = "bytes")]
    pub byte_length: u64 => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ContentIdentity {
    pub(crate) fn of(bytes: &[u8]) -> Self {
        Self {
            sha256: Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            byte_length: bytes.len() as u64,
        }
    }
    pub(crate) fn validate_content(
        &self,
        bytes: &[u8],
        subject: &str,
    ) -> Result<(), WorkflowError> {
        if &Self::of(bytes) != self {
            return Err(WorkflowError::Invalid(format!(
                "{subject} bytes differ from the original report commitment"
            )));
        }
        Ok(())
    }
}
pse_contract::projection_struct! {
/// Independently decodable facts recorded by an original performing report.
///
/// This is an observation, not `Report`, a test verdict, fixture readiness or a
/// physical-settlement capability. Additional report fields remain with their owners.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReportObservation {
    projection: { name: "WorkflowReportObservation", rust_path: crate::test_workflow::execute::ReportObservation, version: 2, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Original report interpretation, not this reader's policy version.
    pub version: u32 => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 3 }] };
    /// Original performing invocation.
    pub invocation: CliInvocationId => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original action; listing and compiler success remain distinct from tests.
    pub action: Action => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original owner's reconciliation, never a newly issued verdict.
    pub outcome: Outcome => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original immutable inventory association, absent for some failed operations.
    #[serde(default)]
    pub inventory: Option<String> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Actual captured test count when the performing owner established it.
    #[serde(default)]
    pub selected_tests: Option<usize> => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original required capture completeness, independent of physical drain.
    pub required_capture_complete: bool => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original typed diagnostic observations.
    pub failures: Vec<ErrorRecord> => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Absent only for legacy interpretation or a report without inventory.
    #[serde(default)]
    pub inventory_content: Option<ContentIdentity> => { tag: 9, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original dependency commitments and omissions, absent only for legacy interpretations.
    #[serde(default)]
    pub evidence: Option<Vec<EvidenceFile>> => { tag: 10, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ReportObservation {
    /// Borrow the performing owner's public recorded facts; custody stays in `Report`.
    pub fn of(report: &Report) -> Self {
        Self {
            version: report.version,
            invocation: report.invocation.clone(),
            action: report.action,
            outcome: report.outcome,
            inventory: report.inventory.clone(),
            selected_tests: report.selected_tests,
            required_capture_complete: report.required_capture_complete,
            failures: report.failures.clone(),
            inventory_content: report.inventory_content.clone(),
            evidence: report.evidence.clone(),
        }
    }
    /// Check recorded facts against the original terminal without granting authority.
    ///
    /// The caller separately authenticates the terminal, report bytes and original
    /// custody. Successful matching alone does not establish current execution eligibility.
    pub fn validate_against(&self, terminal: &TerminalRecord) -> Result<(), WorkflowError> {
        terminal.validate(&self.invocation, self.action)?;
        self.validate_interpretation()?;
        if self.invocation != terminal.invocation
            || self.action != terminal.action
            || self.outcome != terminal.outcome
            || self.required_capture_complete != terminal.required_capture_complete
        {
            return Err(WorkflowError::Invalid(
                "owner report facts differ from the original terminal".into(),
            ));
        }
        Ok(())
    }
    fn validate_interpretation(&self) -> Result<(), WorkflowError> {
        let valid = match self.version {
            1 => self.inventory_content.is_none() && self.evidence.is_none(),
            2 => {
                self.inventory.is_some() == self.inventory_content.is_some()
                    && self.evidence.is_none()
            }
            3 => {
                self.inventory.is_some() == self.inventory_content.is_some()
                    && self.evidence.is_some()
            }
            _ => false,
        };
        if !valid
            || self
                .inventory
                .as_deref()
                .is_some_and(|path| path != INVENTORY)
        {
            return Err(WorkflowError::Invalid(
                "unsupported report interpretation or missing original inventory commitment".into(),
            ));
        }
        if let Some(files) = &self.evidence {
            evidence::validate(files)?;
            if files
                .iter()
                .filter(|file| file.role == EvidenceRole::Input)
                .count()
                != 1
            {
                return Err(WorkflowError::Invalid("original evidence account requires exactly one authenticated-input commitment or original omission".into()));
            }
            if files
                .iter()
                .filter(|file| file.role == EvidenceRole::Inventory)
                .count()
                != usize::from(self.inventory.is_some())
            {
                return Err(WorkflowError::Invalid(
                    "original evidence account and inventory association disagree".into(),
                ));
            }
            if files
                .iter()
                .find(|file| file.role == EvidenceRole::Inventory)
                .and_then(|file| file.content.as_ref())
                .is_some_and(|content| Some(content) != self.inventory_content.as_ref())
            {
                return Err(WorkflowError::Invalid(
                    "original dependency and inventory commitments disagree".into(),
                ));
            }
        }
        Ok(())
    }
    /// Current producers cannot publish legacy or partially bound original evidence.
    pub fn require_current(&self) -> Result<(), WorkflowError> {
        self.validate_interpretation()?;
        if self.version != 3 {
            return Err(WorkflowError::Invalid(
                "current performer requires report interpretation 3".into(),
            ));
        }
        Ok(())
    }
    /// Reconcile original inventory bytes; legacy path-only reports cannot supply a commitment.
    pub fn validate_inventory(&self, bytes: &[u8]) -> Result<(), WorkflowError> {
        self.validate_interpretation()?;
        if self.version == 1 {
            return Err(WorkflowError::Invalid("legacy report interpretation 1 has no original inventory commitment; inventory adoption and exact rerun are unavailable".into()));
        }
        let expected = self.inventory_content.as_ref().ok_or_else(|| {
            WorkflowError::Invalid("original report has no inventory commitment".into())
        })?;
        expected.validate_content(bytes, "original inventory")
    }
    /// Reconcile the exact input snapshot with its original performing report. Legacy
    /// interpretations still require the native metadata binding at their owning caller.
    pub(crate) fn validate_input_bytes(&self, bytes: &[u8]) -> Result<Input, WorkflowError> {
        self.validate_interpretation()?;
        let input: Input =
            serde_json::from_slice(bytes).map_err(crate::discovery::DiscoveryError::from)?;
        input.validate_original(&self.invocation, self.action.command())?;
        if self.version == 3 {
            let file = self
                .evidence
                .as_ref()
                .and_then(|files| files.iter().find(|file| file.role == EvidenceRole::Input))
                .ok_or_else(|| {
                    WorkflowError::Invalid("original input dependency is absent".into())
                })?;
            file.content
                .as_ref()
                .ok_or_else(|| {
                    WorkflowError::Invalid(
                        "original input dependency was unavailable at the performing cutoff".into(),
                    )
                })?
                .validate_content(bytes, "original workflow input")?;
        }
        Ok(input)
    }
    /// Reconcile an observed count with the actual native listing population, when known.
    pub(crate) fn validate_selected_population(
        &self,
        inventory: &crate::discovery::Inventory,
    ) -> Result<(), WorkflowError> {
        if self
            .selected_tests
            .is_some_and(|count| count != inventory.selected_tests_iter().count())
        {
            return Err(WorkflowError::Invalid(
                "original report selected count differs from native inventory".into(),
            ));
        }
        Ok(())
    }
}
pse_contract::projection_struct! {
/// Exactly one final JSONL frame. Root validates this original helper transport separately
/// from raw capture, original native payload identity and physical descendant settlement.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalRecord {
    projection: { name: "WorkflowTerminalRecord", rust_path: crate::test_workflow::execute::TerminalRecord, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Exact performing protocol.
    pub protocol: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Supported wire version.
    pub version: u32 => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 1 }] };
    /// Original invoking owner.
    pub invocation: CliInvocationId => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exact expected operation.
    pub action: Action => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Helper reconciliation, never a manager capability.
    pub outcome: Outcome => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Already durable original report.
    pub report: ReportReference => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Inner required raw observation completeness.
    pub required_capture_complete: bool => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
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
            || self.report.byte_length == 0
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

#[cfg(test)]
mod observation_tests {
    use super::*;

    fn current_report() -> Report {
        Report {
            version: 3,
            invocation: CliInvocationId::new(),
            action: Action::Test,
            outcome: Outcome::Failed,
            selection: None,
            admission: vec![],
            inventory: None,
            inventory_content: None,
            evidence: None,
            selected_tests: Some(2),
            fixture_requirements: vec![],
            pending: vec![],
            phases: vec![],
            observations: vec![],
            required_capture_complete: false,
            failures: vec![],
            causes: vec![],
            evidence_sources: vec![],
            input_cutoff: None,
            sealed_content: None,
        }
    }

    #[test]
    fn native_report_projection_borrows_complete_diagnostics_without_verdict_construction() {
        use pse_contract::boundary_projection::{
            ConstructionRole, ProjectionGraph, ProjectionObservationRecord, Shape,
        };
        let graph = ProjectionGraph::of::<Report>().unwrap();
        assert_eq!(
            graph.definitions()["NativeWorkflowReport"].construction,
            ConstructionRole::ObservationOnly
        );
        let Shape::Record(fields) = &graph.definitions()["ErrorRecord"].shape else {
            panic!("record expected")
        };
        assert_eq!(fields.len(), 15);
        assert!(fields.iter().any(|field| field.name == "diagnostic_source"));
        assert!(fields.iter().any(|field| field.name == "recovery"));
        assert!(graph.definitions().values().any(|owner| owner.rust_type == std::any::type_name::<pse_diagnostics::NonFinite>()));
        let mut report = current_report();
        report.failure(WorkflowError::Invalid("original native failure".into()));
        let child = report.failures[0].clone();
        report.failures[0].children.push(child.clone());
        report.failures[0].diagnostic_source = Some(Box::new(child.clone()));
        report.failures[0].related.push(child);
        let expected = serde_json::to_value(&report.failures).unwrap();
        let (observation,) = report.observation_fields().unwrap();
        assert_eq!(
            serde_json::to_value(&observation.failures).unwrap(),
            expected
        );
        assert!(!observation.required_capture_complete);
        assert_eq!(
            report.causes.len(),
            1,
            "the performing owner retains original native causes"
        );
        assert_eq!(report.observations.len(), 0);
    }

    #[test]
    fn native_report_publication_commits_original_inventory_and_refuses_same_shape_replacement() {
        use std::fs;
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
        let scratch = tempfile::tempdir().unwrap();
        fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(scratch.path().join("inventory"))
            .unwrap();
        let path = scratch.path().join(INVENTORY);
        let original = br#"{"test":"first"}"#;
        let changed = br#"{"test":"other"}"#;
        assert_eq!(original.len(), changed.len());
        fs::write(&path, original).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let mut report = current_report();
        report.record_inventory(scratch.path()).unwrap();
        let terminal = report.publish(scratch.path()).unwrap();
        assert_eq!(
            terminal.version, 1,
            "terminal transport interpretation is independent"
        );
        let observation: ReportObservation =
            serde_json::from_slice(&fs::read(scratch.path().join(REPORT)).unwrap()).unwrap();
        assert_eq!(observation.version, 3);
        observation.validate_against(&terminal).unwrap();
        observation.validate_inventory(original).unwrap();
        assert!(observation.validate_inventory(changed).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, changed).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            report
                .publish(scratch.path())
                .unwrap_err()
                .to_string()
                .contains("inventory bytes")
        );
        fs::remove_file(&path).unwrap();
        assert!(report.publish(scratch.path()).is_err());
    }

    #[test]
    fn original_dependency_sealing_is_once_only_even_when_first_publication_refuses() {
        use std::{fs, os::unix::fs::PermissionsExt as _};
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = root.path().join("run-0.stdout");
        fs::write(&path, b"original").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let mut report = current_report();
        report.phases.push(Phase {
            native_stage: None,
            elapsed_seconds: None,
            kind: PhaseKind::Test,
            group: Some(0),
            stdout: "run-0.stdout".into(),
            stderr: "run-0.stderr".into(),
            child_exit: Some(100),
            capture_complete: true,
            stdout_capture: None,
            stderr_capture: None,
        });
        fs::write(root.path().join(REPORT), b"existing destination").unwrap();
        assert!(report.publish(root.path()).is_err());
        let sealed = serde_json::to_vec(&report.evidence).unwrap();
        fs::remove_file(root.path().join(REPORT)).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"replaced").unwrap();
        assert!(
            report
                .publish(root.path())
                .unwrap_err()
                .to_string()
                .contains("original commitment")
        );
        assert_eq!(serde_json::to_vec(&report.evidence).unwrap(), sealed);
        assert!(!root.path().join(REPORT).exists());
    }
    #[test]
    fn input_dependency_uses_authenticated_snapshot_instead_of_rehashing_replacement_path() {
        use std::{
            fs,
            os::unix::fs::{DirBuilderExt as _, PermissionsExt as _},
        };
        let invocation = tempfile::tempdir().unwrap();
        fs::set_permissions(invocation.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let workflow = invocation.path().join("workflow");
        fs::DirBuilder::new().mode(0o700).create(&workflow).unwrap();
        let original = b"{\"accepted\":\"original-input\"}\n";
        let replacement = b"{\"accepted\":\"original-input\"}\n \n\t";
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(original).unwrap(),
            serde_json::from_slice::<serde_json::Value>(replacement).unwrap()
        );
        let path = invocation.path().join("workflow-input.json");
        fs::write(&path, replacement).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let mut report = current_report();
        report.input_cutoff = Some(Ok(ContentIdentity::of(original)));
        assert!(report.publish(&workflow).is_err());
        let input = report
            .evidence
            .as_ref()
            .unwrap()
            .iter()
            .find(|file| file.role == EvidenceRole::Input)
            .unwrap();
        assert_eq!(
            input.content.as_ref().unwrap(),
            &ContentIdentity::of(original)
        );
        assert!(!workflow.join(REPORT).exists());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, original).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            report.publish(&workflow).is_ok(),
            "restoring exact original bytes preserves the original sealed promise"
        );
    }
    #[test]
    fn original_report_facts_cannot_change_after_a_failed_publication_attempt() {
        use std::{fs, os::unix::fs::PermissionsExt as _};
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(root.path().join(REPORT), b"occupied").unwrap();
        let mut report = current_report();
        assert!(report.publish(root.path()).is_err());
        fs::remove_file(root.path().join(REPORT)).unwrap();
        report.outcome = Outcome::Incomplete;
        assert!(
            report
                .publish(root.path())
                .unwrap_err()
                .to_string()
                .contains("facts changed after dependency sealing")
        );
        assert!(!root.path().join(REPORT).exists());
    }

    #[test]
    fn legacy_report_facts_remain_readable_without_fabricating_inventory_commitment() {
        let (legacy, terminal) = original();
        legacy.validate_against(&terminal).unwrap();
        assert!(legacy.require_current().is_err());
        assert!(
            legacy
                .validate_inventory(b"original")
                .unwrap_err()
                .to_string()
                .contains("legacy")
        );
        let mut current = legacy;
        current.version = 2;
        assert!(
            current.validate_against(&terminal).is_err(),
            "current inventory requires its original content commitment"
        );
    }

    fn original() -> (ReportObservation, TerminalRecord) {
        let invocation = CliInvocationId::new();
        let observation = serde_json::from_value(serde_json::json!({
            "version": 1, "invocation": invocation, "action": "test",
            "outcome": "failed", "inventory": "inventory/inventory.json",
            "selected_tests": 2, "required_capture_complete": true, "failures": [],
            "observations": [{"kind": "test", "private_native_verdict": "not reconstructed"}]
        }))
        .unwrap();
        let terminal = TerminalRecord {
            protocol: PROTOCOL.into(),
            version: 1,
            invocation,
            action: Action::Test,
            outcome: Outcome::Failed,
            report: ReportReference {
                path: REPORT.into(),
                sha256: "a".repeat(64),
                byte_length: 1,
            },
            required_capture_complete: true,
        };
        (observation, terminal)
    }

    #[test]
    fn recorded_report_facts_are_decodable_without_native_verdict_construction() {
        let (observation, terminal) = original();
        observation.validate_against(&terminal).unwrap();
        assert_eq!(observation.selected_tests, Some(2));
        assert_eq!(
            observation.inventory.as_deref(),
            Some("inventory/inventory.json")
        );
        assert_eq!(observation.outcome, Outcome::Failed);
        assert!(!observation.outcome.successful(observation.action));
        // Serialization exposes facts only, never the ignored native verdict population.
        assert!(
            serde_json::to_value(observation)
                .unwrap()
                .get("observations")
                .is_none()
        );
    }

    #[test]
    fn retained_report_facts_require_original_identity_interpretation_and_capture() {
        let (mut observation, terminal) = original();
        observation.version = 2;
        assert!(observation.validate_against(&terminal).is_err());
        observation.version = 1;
        observation.invocation = CliInvocationId::new();
        assert!(observation.validate_against(&terminal).is_err());
        observation.invocation = terminal.invocation.clone();
        observation.required_capture_complete = false;
        assert!(observation.validate_against(&terminal).is_err());
        observation.required_capture_complete = true;
        observation.action = Action::Check;
        assert!(observation.validate_against(&terminal).is_err());
        observation.action = Action::Test;
        observation.outcome = Outcome::TestsSucceeded;
        assert!(observation.validate_against(&terminal).is_err());
    }
}
