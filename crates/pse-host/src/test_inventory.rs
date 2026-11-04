// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive pages of an original captured population; never execution or fixture authority.
use crate::{
    HostError,
    discovery::Inventory,
    envelope::{CliInvocationId, Envelope},
    invocation::InvocationStore,
    managed::result::ManagedSummary,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{self, Read as _},
    num::NonZeroUsize,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::Path,
};

/// One indivisible actual selected test, preserving its original compiler context.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TestRow {
    /// Exact row in the authenticated original inventory and report.
    pub reference: crate::investigation::ResultReference,
    /// Original group, package, binary and exact harness ID.
    pub identity: crate::discovery::TestIdentity,
    /// Same-build target name, independent of the test's spelling.
    pub target: String,
    /// Original Cargo target kinds.
    pub target_kinds: Vec<String>,
    /// Actual compiler-artifact features, not guessed from declarations.
    pub features: BTreeSet<String>,
    /// Original listed ignored status.
    pub ignored: bool,
    /// Actual admitted compiler-group capabilities, not a guessed per-test dependency graph.
    pub group_capabilities: BTreeSet<String>,
    /// Original matching declarations; these do not establish readiness or create fixtures.
    pub fixtures: Option<Vec<crate::fixtures::Requirement>>,
}

/// A source-bound whole-record window, independent of original workload acceptance.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TestPage {
    /// Selected original operation, not this observer.
    pub original_invocation: CliInvocationId,
    /// Exact bytes read from the selected retained inventory.
    pub inventory_sha256: String,
    /// Actual selected population size in this inventory.
    pub selected_tests: usize,
    /// Complete rows only; a byte budget never truncates a test identity.
    pub records: Vec<TestRow>,
    /// Explicit continuation, bound to these exact inventory bytes.
    pub next: Option<Cursor>,
    /// A next record exceeding the byte budget stays unread rather than being split.
    pub next_record_bytes: Option<usize>,
}

/// Read position only. Decoding this value grants no process or execution authority.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    /// Original operation identity.
    pub invocation: CliInvocationId,
    /// Original inventory byte identity.
    pub inventory_sha256: String,
    /// Zero-based selected-test index, never a name-only continuation.
    pub offset: usize,
}

/// Read-only continuation bound to the original frozen performing report.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FailureCursor {
    /// Original performing operation.
    pub invocation: CliInvocationId,
    /// Exact frozen report bytes.
    pub report_sha256: String,
    /// Whole failure row position.
    pub offset: usize,
}
/// Original diagnostics and native failure artifact addresses; no reconstructed verdict.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FailureRow {
    /// Original typed infrastructure, compiler, setup or runner diagnostic.
    Diagnostic {
        /// Exact row in the authenticated original report.
        reference: Option<crate::investigation::ResultReference>,
        /// Original typed diagnostic with source/help records.
        diagnostic: Box<crate::envelope::ErrorRecord>,
    },
    /// Exact failed test in its native completed artifact. XML text is not raw process output.
    Test {
        /// Exact row in the authenticated original report.
        reference: Option<crate::investigation::ResultReference>,
        /// Original group, package, binary and exact test ID.
        identity: crate::discovery::TestIdentity,
        /// Original completed native artifact containing assertion output.
        report: std::path::PathBuf,
        /// Redacted native XML message retained by the performing owner, when available.
        message: Option<String>,
        /// Redacted native XML assertion text retained by the performing owner, when available.
        description: Option<String>,
    },
    /// Original setup or runner context, retaining wrapper/native distinctions.
    Runner {
        /// Exact row in the authenticated original report.
        reference: Option<crate::investigation::ResultReference>,
        /// Original compiler context ordinal.
        group: usize,
        /// Original completed native artifact, possibly absent on startup failure.
        report: std::path::PathBuf,
        /// Original directly waited child status, which may be a fixture wrapper.
        child_exit: i32,
        /// Original native Nextest status, absent when unobserved.
        native_child_exit: Option<i32>,
        /// Original borrowed fixture owner's actual settlement observation.
        fixture_settled: Option<bool>,
        /// Original native setup interpretation, without reconstruction.
        setup: String,
    },
}
/// Bounded whole-record view of one original performing report.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FailurePage {
    /// Original operation, separate from this observer.
    pub original_invocation: CliInvocationId,
    /// Original frozen report byte identity.
    pub report_sha256: String,
    /// All original diagnostic and native test/runner rows.
    pub total: usize,
    /// Whole rows only; source fields are never cut to fit.
    pub records: Vec<FailureRow>,
    /// Continuation within these frozen report bytes.
    pub next: Option<FailureCursor>,
    /// A row larger than this page budget stays unread.
    pub next_record_bytes: Option<usize>,
}

#[derive(Deserialize)]
struct FailureSource {
    failures: Vec<crate::envelope::ErrorRecord>,
    observations: Vec<FailureGroup>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FailureGroup {
    Test {
        group: usize,
        observation: FailureObservation,
    },
    #[serde(other)]
    Other,
}
#[derive(Deserialize)]
struct FailureObservation {
    report: std::path::PathBuf,
    summary: FailureSummary,
    #[serde(default)]
    failures: Vec<FailureDetail>,
}
#[derive(Deserialize)]
struct FailureDetail {
    identity: crate::discovery::TestIdentity,
    message: Option<String>,
    description: Option<String>,
}
#[derive(Deserialize)]
struct FailureSummary {
    failed: Vec<crate::discovery::TestIdentity>,
    child_exit: i32,
    native_child_exit: Option<i32>,
    fixture_settled: Option<bool>,
    setup: String,
}

fn failure_rows(report: &[u8]) -> Result<Vec<FailureRow>, HostError> {
    let source: FailureSource = serde_json::from_slice(report).map_err(io::Error::other)?;
    let mut rows: Vec<_> = source
        .failures
        .into_iter()
        .map(|diagnostic| FailureRow::Diagnostic {
            reference: None,
            diagnostic: Box::new(diagnostic),
        })
        .collect();
    for group in source.observations {
        let FailureGroup::Test { group, observation } = group else {
            continue;
        };
        let summary = observation.summary;
        rows.extend(summary.failed.into_iter().map(|identity| {
            let detail = observation
                .failures
                .iter()
                .find(|detail| detail.identity == identity);
            FailureRow::Test {
                reference: None,
                message: detail.and_then(|detail| detail.message.clone()),
                description: detail.and_then(|detail| detail.description.clone()),
                identity,
                report: observation.report.clone(),
            }
        }));
        if summary.child_exit != 0
            || summary.native_child_exit.is_some_and(|exit| exit != 0)
            || summary.fixture_settled == Some(false)
            || summary.setup == "failed_by_runner"
        {
            rows.push(FailureRow::Runner {
                reference: None,
                group,
                report: observation.report,
                child_exit: summary.child_exit,
                native_child_exit: summary.native_child_exit,
                fixture_settled: summary.fixture_settled,
                setup: summary.setup,
            });
        }
    }
    Ok(rows)
}

/// Read original failures without rerunning, consuming live logs, or starting fixtures.
pub fn failures(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    cursor: Option<&FailureCursor>,
    limit: NonZeroUsize,
    byte_budget: NonZeroUsize,
) -> Result<Envelope<FailurePage>, HostError> {
    let CapturedReport { id, report, .. } = captured_report(store, selector, session)?;
    let sha = digest(&report);
    let mut rows = failure_rows(&report)?;
    address_failures(&mut rows, &id, &sha);
    let total = rows.len();
    let offset = match cursor {
        None => 0,
        Some(cursor)
            if cursor.invocation == id && cursor.report_sha256 == sha && cursor.offset <= total =>
        {
            cursor.offset
        }
        Some(_) => {
            return Err(invalid(
                "failure continuation identity, report bytes or extent changed",
            ));
        }
    };
    let mut records = Vec::new();
    let mut used = 0usize;
    let mut next_record_bytes = None;
    for row in rows.into_iter().skip(offset).take(limit.get()) {
        let extent = serde_json::to_vec(&row).map_err(io::Error::other)?.len();
        if used.saturating_add(extent) > byte_budget.get() {
            next_record_bytes = Some(extent);
            break;
        }
        used += extent;
        records.push(row);
    }
    let next_offset = offset + records.len();
    let next = (next_offset < total).then(|| FailureCursor {
        invocation: id.clone(),
        report_sha256: sha.clone(),
        offset: next_offset,
    });
    let mut response = Envelope::response(
        "inspect tests failures",
        format!("{} of {total} original failure rows", records.len()),
        FailurePage {
            original_invocation: id,
            report_sha256: sha,
            total,
            records,
            next,
            next_record_bytes,
        },
    );
    response.observation.scope = "retained_native_failure_page".into();
    response.observation.limitations=vec!["Rows project the frozen original performing report. Artifact paths are original references; full evidence scope separately reconciles retained bytes with original commitments.".into(),"The full retained report is decoded; page limits bound whole-record output rather than scan cost.".into()];
    Ok(response)
}
fn invalid(message: &str) -> HostError {
    HostError::Ownership(format!("test inventory: {message}"))
}
pub(crate) fn read_private(path: &Path) -> Result<Vec<u8>, HostError> {
    read_private_bounded(path, None)
}
fn read_private_bounded(path: &Path, maximum: Option<usize>) -> Result<Vec<u8>, HostError> {
    let mut file = open_private(path)?;
    let mut bytes = Vec::new();
    if let Some(maximum) = maximum {
        if file.metadata()?.len() > maximum as u64 {
            return Err(invalid(
                "original evidence exceeds the selected portable extent",
            ));
        }
        file.take(maximum.saturating_add(1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > maximum {
            return Err(invalid(
                "original evidence grew beyond the selected portable extent",
            ));
        }
    } else {
        file.read_to_end(&mut bytes)?;
    }
    Ok(bytes)
}
/// One custody rule for publication, current live validation and retained adoption.
pub(crate) fn read_original_inventory(workflow: &Path) -> Result<Vec<u8>, HostError> {
    read_original_inventory_bounded(workflow, None)
}
fn read_original_inventory_bounded(
    workflow: &Path,
    maximum: Option<usize>,
) -> Result<Vec<u8>, HostError> {
    checked_parent(workflow)?;
    let inventory = workflow.join("inventory");
    checked_parent(&inventory)?;
    read_private_bounded(&inventory.join("inventory.json"), maximum)
}
/// Open original private evidence once so ranged consumers retain the checked descriptor.
pub(crate) fn open_private(path: &Path) -> Result<fs::File, HostError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(invalid(
            "retained input is not an owned regular evidence file",
        ));
    }
    Ok(file)
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn address_failures(rows: &mut [FailureRow], id: &CliInvocationId, sha: &str) {
    for (ordinal, row) in rows.iter_mut().enumerate() {
        let address = Some(crate::investigation::ResultReference::new(
            crate::investigation::Locator::FrozenTest {
                invocation: id.to_string(),
                report_sha256: sha.into(),
                inventory_sha256: None,
                selection: crate::investigation::FrozenTestSelection::Failure { ordinal },
            },
        ));
        match row {
            FailureRow::Diagnostic { reference, .. }
            | FailureRow::Test { reference, .. }
            | FailureRow::Runner { reference, .. } => *reference = address,
        }
    }
}

/// An exact frozen row and the original report context; no execution verdict is reconstructed.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FrozenRecord {
    /// Guarded original row address.
    pub reference: crate::investigation::ResultReference,
    /// Original selected test, when a population row was selected.
    pub test: Option<TestRow>,
    /// Original diagnostic/test/runner failure, when selected.
    pub failure: Option<FailureRow>,
    /// Original frozen action/outcome/phase/setup inputs, retaining their original vocabulary.
    pub context: serde_json::Value,
}

/// Resolve exact original bytes, never treating the reference as execution or rerun authority.
pub fn frozen(
    store: &InvocationStore,
    reference: &crate::investigation::ResultReference,
) -> Result<FrozenRecord, HostError> {
    use crate::investigation::{FrozenTestSelection, Locator};
    reference.validate()?;
    let Locator::FrozenTest {
        invocation,
        report_sha256,
        inventory_sha256,
        selection,
    } = &reference.locator
    else {
        return Err(invalid("expected a frozen test/failure row"));
    };
    let CapturedReport { id, report, .. } = captured_report(store, invocation, None)?;
    if digest(&report) != *report_sha256 {
        return Err(invalid("frozen report digest changed"));
    }
    let mut original: serde_json::Value =
        serde_json::from_slice(&report).map_err(io::Error::other)?;
    let (test, failure) = match selection {
        FrozenTestSelection::Failure { ordinal } => {
            let mut rows = failure_rows(&report)?;
            address_failures(&mut rows, &id, report_sha256);
            let row = rows
                .into_iter()
                .nth(*ordinal)
                .ok_or_else(|| invalid("frozen failure ordinal is outside original report"))?;
            (None, Some(row))
        }
        FrozenTestSelection::Test { identity } => {
            let captured = captured(store, invocation, None)?;
            if inventory_sha256.as_deref() != Some(&captured.inventory_sha256) {
                return Err(invalid("frozen inventory digest changed"));
            }
            let offset = captured
                .inventory
                .selected_tests_iter()
                .position(|candidate| candidate == *identity)
                .ok_or_else(|| invalid("exact original test identity is absent"))?;
            let cursor = Cursor {
                invocation: id,
                inventory_sha256: captured.inventory_sha256,
                offset,
            };
            let one = NonZeroUsize::new(1).ok_or_else(|| invalid("invalid row extent"))?;
            let bytes =
                NonZeroUsize::new(usize::MAX).ok_or_else(|| invalid("invalid row budget"))?;
            let row = read(store, invocation, None, Some(&cursor), one, bytes)?
                .payload
                .records
                .into_iter()
                .next()
                .ok_or_else(|| invalid("original selected row disappeared"))?;
            (Some(row), None)
        }
    };
    // These are exact original values, not a second status/setup interpreter.
    let mut context = serde_json::Map::new();
    if let Some(object) = original.as_object_mut() {
        for key in [
            "invocation",
            "action",
            "outcome",
            "selection",
            "admission",
            "pending",
            "phases",
            "fixture_requirements",
            "required_capture_complete",
        ] {
            if let Some(value) = object.remove(key) {
                context.insert(key.into(), value);
            }
        }
    }
    Ok(FrozenRecord {
        reference: reference.clone(),
        test,
        failure,
        context: context.into(),
    })
}
pub(crate) fn checked_parent(path: &Path) -> Result<(), HostError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(invalid(
            "evidence directory is not private original custody",
        ));
    }
    Ok(())
}

/// Original report and inventory, authenticated by their frozen terminal association.
#[derive(Debug)]
pub(crate) struct CapturedEvidence {
    pub id: CliInvocationId,
    pub report: Vec<u8>,
    pub inventory: Inventory,
    /// Complete original bytes, retained independently of the native parsed inventory.
    pub inventory_bytes: Vec<u8>,
    pub inventory_sha256: String,
    pub observation: crate::test_workflow::execute::ReportObservation,
    /// Original report commitment and terminal facts; never a new performing result.
    pub terminal: crate::test_workflow::execute::TerminalRecord,
    /// Manager's recorded read-time settlement, independent of the helper commitment.
    pub manager_settlement: Option<crate::envelope::Settlement>,
    /// Missing legacy observations stay unavailable rather than becoming false.
    pub native_lifetime_pending: Option<bool>,
}
impl pse_contract::boundary_projection::ProjectionType for CapturedEvidence {
    fn projection(
        graph: &mut pse_contract::boundary_projection::ProjectionGraph,
    ) -> Result<
        pse_contract::boundary_projection::TypeRef,
        pse_contract::boundary_projection::ProjectionError,
    > {
        use pse_contract::boundary_projection::{Direction, Field, Shape};
        let root = graph.declare::<Self>(
            "CapturedWorkflowEvidence",
            file!(),
            1,
            Direction::Output,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<CliInvocationId>(
                        graph,
                        "invocation",
                        1,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Vec<u8>>(graph, "report_original", 2, Direction::Output, vec![])?,
                    Field::of::<Vec<u8>>(
                        graph,
                        "inventory_original",
                        3,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<String>(graph, "inventory_sha256", 4, Direction::Output, vec![])?,
                    Field::of::<crate::test_workflow::execute::ReportObservation>(
                        graph,
                        "report_observation",
                        5,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<crate::test_workflow::execute::TerminalRecord>(
                        graph,
                        "terminal",
                        6,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Vec<crate::discovery::TestIdentity>>(
                        graph,
                        "selected_tests",
                        7,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Option<bool>>(
                        graph,
                        "selected_population_complete",
                        8,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Option<crate::envelope::Settlement>>(
                        graph,
                        "recorded_manager_settlement",
                        9,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Option<bool>>(
                        graph,
                        "recorded_native_lifetime_pending",
                        10,
                        Direction::Output,
                        vec![],
                    )?,
                ]))
            },
        )?;
        graph.public_path::<Self>(&root, "crate::test_inventory::CapturedEvidence")?;
        graph.observation_record::<Self>(&root)?;
        Ok(root)
    }
}
impl pse_contract::boundary_projection::ProjectionObservationRecord for CapturedEvidence {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[
        (1, "invocation"),
        (2, "report_original"),
        (3, "inventory_original"),
        (4, "inventory_sha256"),
        (5, "report_observation"),
        (6, "terminal"),
        (7, "selected_tests"),
        (8, "selected_population_complete"),
        (9, "recorded_manager_settlement"),
        (10, "recorded_native_lifetime_pending"),
    ];
    type Fields = (
        CliInvocationId,
        Vec<u8>,
        Vec<u8>,
        String,
        crate::test_workflow::execute::ReportObservation,
        crate::test_workflow::execute::TerminalRecord,
        Vec<crate::discovery::TestIdentity>,
        Option<bool>,
        Option<crate::envelope::Settlement>,
        Option<bool>,
    );
    fn observation_fields(
        &self,
    ) -> Result<Self::Fields, pse_contract::boundary_projection::AdapterError> {
        Ok((
            self.id.clone(),
            self.report.clone(),
            self.inventory_bytes.clone(),
            self.inventory_sha256.clone(),
            self.observation.clone(),
            self.terminal.clone(),
            self.inventory.selected_tests_iter().collect(),
            (!self.observation.action.is_compiler())
                .then(|| self.inventory.selected_population_complete()),
            self.manager_settlement,
            self.native_lifetime_pending,
        ))
    }
}
impl pse_contract::boundary_projection::ValidateProjection for CapturedEvidence {
    fn validate_at(
        &self,
        path: &str,
    ) -> Result<(), pse_contract::boundary_projection::ValidationError> {
        self.observation
            .validate_at(&pse_contract::boundary_projection::field_path(
                path,
                "report_observation",
            ))?;
        self.terminal
            .validate_at(&pse_contract::boundary_projection::field_path(
                path, "terminal",
            ))
    }
}

/// Source-built developer contract world; it does not acquire evidence or run a workflow.
pub fn developer_evidence_projection() -> Result<
    pse_contract::boundary_projection::ProjectionGraph,
    pse_contract::boundary_projection::ProjectionError,
> {
    let mut graph = pse_contract::boundary_projection::ProjectionGraph::of::<
        crate::test_workflow::execute::Report,
    >()?;
    graph.register::<CapturedReport>()?;
    graph.register::<CapturedEvidence>()?;
    Ok(graph)
}
/// Read evidence as data, without treating decoded fields as native pass capabilities.
pub(crate) fn captured(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
) -> Result<CapturedEvidence, HostError> {
    captured_at_limit(store, selector, session, None)
}
pub(crate) fn captured_at_limit(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    maximum: Option<usize>,
) -> Result<CapturedEvidence, HostError> {
    let CapturedReport {
        id,
        report,
        parent,
        observation,
        original,
        ..
    } = captured_report_at_limit(store, selector, session, maximum)?;
    if observation.inventory.as_deref() != Some("inventory/inventory.json") {
        return Err(invalid(
            "original report has no supported inventory association",
        ));
    }
    let remaining = maximum
        .map(|maximum| {
            maximum
                .checked_sub(report.len())
                .ok_or_else(|| invalid("original report exceeds the selected portable extent"))
        })
        .transpose()?;
    let bytes = read_original_inventory_bounded(&parent, remaining)?;
    observation
        .validate_inventory(&bytes)
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let sha = digest(&bytes);
    let inventory: Inventory = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if !inventory.capture_complete() {
        return Err(invalid(
            "original inventory lacks complete supported discovery capture",
        ));
    }
    observation
        .validate_selected_population(&inventory)
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    Ok(CapturedEvidence {
        id,
        report,
        inventory,
        inventory_bytes: bytes,
        inventory_sha256: sha,
        observation,
        manager_settlement: Some(original.settlement),
        native_lifetime_pending: original.payload.native_lifetime_pending,
        terminal: original
            .payload
            .workflow
            .ok_or_else(|| invalid("original workflow terminal disappeared"))?
            .terminal,
    })
}

/// One canonical read keeps the authenticated terminal beside its exact report bytes.
pub(crate) struct CapturedReport {
    pub id: CliInvocationId,
    pub report: Vec<u8>,
    pub parent: std::path::PathBuf,
    pub original: Envelope<ManagedSummary>,
    pub observation: crate::test_workflow::execute::ReportObservation,
    /// Explicit report-only or full dependency account, without selection authority.
    pub scope: crate::test_workflow::execute::EvidenceScope,
    /// Exact bytes for the original report's committed dependencies only.
    pub files: Vec<crate::test_workflow::execute::CapturedFile>,
}
impl pse_contract::boundary_projection::ProjectionType for CapturedReport {
    fn projection(
        graph: &mut pse_contract::boundary_projection::ProjectionGraph,
    ) -> Result<
        pse_contract::boundary_projection::TypeRef,
        pse_contract::boundary_projection::ProjectionError,
    > {
        use pse_contract::boundary_projection::{Direction, Field, Shape};
        let root = graph.declare::<Self>(
            "CapturedWorkflowReport",
            file!(),
            2,
            Direction::Output,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<CliInvocationId>(
                        graph,
                        "invocation",
                        1,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Vec<u8>>(graph, "report_original", 2, Direction::Output, vec![])?,
                    Field::of::<crate::test_workflow::execute::ReportObservation>(
                        graph,
                        "report_observation",
                        3,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<crate::test_workflow::execute::TerminalRecord>(
                        graph,
                        "terminal",
                        4,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Option<crate::envelope::Settlement>>(
                        graph,
                        "recorded_manager_settlement",
                        5,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Option<bool>>(
                        graph,
                        "recorded_native_lifetime_pending",
                        6,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<crate::test_workflow::execute::EvidenceScope>(
                        graph,
                        "scope",
                        7,
                        Direction::Output,
                        vec![],
                    )?,
                    Field::of::<Vec<crate::test_workflow::execute::CapturedFile>>(
                        graph,
                        "files",
                        8,
                        Direction::Output,
                        vec![],
                    )?,
                ]))
            },
        )?;
        graph.public_path::<Self>(&root, "crate::test_inventory::CapturedReport")?;
        graph.observation_record::<Self>(&root)?;
        Ok(root)
    }
}
impl pse_contract::boundary_projection::ProjectionObservationRecord for CapturedReport {
    const VERSION: u32 = 2;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[
        (1, "invocation"),
        (2, "report_original"),
        (3, "report_observation"),
        (4, "terminal"),
        (5, "recorded_manager_settlement"),
        (6, "recorded_native_lifetime_pending"),
        (7, "scope"),
        (8, "files"),
    ];
    type Fields = (
        CliInvocationId,
        Vec<u8>,
        crate::test_workflow::execute::ReportObservation,
        crate::test_workflow::execute::TerminalRecord,
        Option<crate::envelope::Settlement>,
        Option<bool>,
        crate::test_workflow::execute::EvidenceScope,
        Vec<crate::test_workflow::execute::CapturedFile>,
    );
    fn observation_fields(
        &self,
    ) -> Result<Self::Fields, pse_contract::boundary_projection::AdapterError> {
        let terminal = self.original.payload.workflow.as_ref().ok_or_else(|| {
            pse_contract::boundary_projection::AdapterError::refused(
                "terminal",
                pse_contract::boundary_projection::AdapterFailure::InvalidValue,
                "original workflow terminal is unavailable",
            )
        })?;
        Ok((
            self.id.clone(),
            self.report.clone(),
            self.observation.clone(),
            terminal.terminal.clone(),
            Some(self.original.settlement),
            self.original.payload.native_lifetime_pending,
            self.scope,
            self.files.clone(),
        ))
    }
}
impl pse_contract::boundary_projection::ValidateProjection for CapturedReport {
    fn validate_at(
        &self,
        path: &str,
    ) -> Result<(), pse_contract::boundary_projection::ValidationError> {
        self.observation
            .validate_at(&pse_contract::boundary_projection::field_path(
                path,
                "report_observation",
            ))?;
        self.files
            .validate_at(&pse_contract::boundary_projection::field_path(
                path, "files",
            ))
    }
}
pub(crate) fn captured_report(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
) -> Result<CapturedReport, HostError> {
    captured_report_at_limit(store, selector, session, None)
}
pub(crate) fn captured_report_at_limit(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    maximum: Option<usize>,
) -> Result<CapturedReport, HostError> {
    let id = store
        .resolve(selector, session)?
        .ok_or_else(|| invalid("original invocation is absent"))?;
    let result = store
        .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
        .ok_or_else(|| invalid("original workflow has no published terminal evidence"))?;
    let workflow = result
        .payload
        .workflow
        .as_ref()
        .ok_or_else(|| invalid("selected operation is not a captured test workflow"))?;
    let action = crate::test_workflow::execute::Action::from_command(&result.command)
        .ok_or_else(|| invalid("original workflow command is unsupported"))?;
    workflow
        .terminal
        .validate(&id, action)
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let directory = store.directory(&id)?;
    let parent = directory.path().join("workflow");
    checked_parent(&parent)?;
    let report = read_private_bounded(&parent.join("report.json"), maximum)?;
    workflow
        .terminal
        .report
        .validate_content(report.len() as u64, &digest(&report))
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let observation: crate::test_workflow::execute::ReportObservation =
        serde_json::from_slice(&report).map_err(io::Error::other)?;
    observation
        .validate_against(&workflow.terminal)
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    Ok(CapturedReport {
        id,
        report,
        parent,
        original: result,
        observation,
        scope: crate::test_workflow::execute::EvidenceScope::Report,
        files: Vec::new(),
    })
}

pub(crate) fn captured_full_report(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    maximum: usize,
) -> Result<CapturedReport, HostError> {
    let mut original = captured_report_at_limit(store, selector, session, Some(maximum))?;
    let files = original.observation.evidence.as_deref().ok_or_else(|| invalid("legacy report has no original full-dependency commitment; report or selection scope remains available"))?;
    let budget = maximum
        .checked_sub(original.report.len())
        .ok_or_else(|| invalid("original report exceeds the selected portable extent"))?;
    original.files =
        crate::test_workflow::execute::evidence::capture(&original.parent, files, budget)?;
    original.scope = crate::test_workflow::execute::EvidenceScope::Full;
    Ok(original)
}

/// Read the selected protected original inventory without preparing or verifying executable
/// eligibility. This scans/decodes the retained inventory; the window bounds output, not scan cost.
pub fn read(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    cursor: Option<&Cursor>,
    limit: NonZeroUsize,
    byte_budget: NonZeroUsize,
) -> Result<Envelope<TestPage>, HostError> {
    let CapturedEvidence {
        id,
        report,
        inventory,
        inventory_sha256: sha,
        ..
    } = captured(store, selector, session)?;
    let report_sha256 = digest(&report);
    #[derive(Deserialize)]
    struct Declarations {
        #[serde(default)]
        fixture_requirements: Option<Vec<crate::fixtures::Requirement>>,
    }
    let declarations: Declarations = serde_json::from_slice(&report).map_err(io::Error::other)?;
    let total = inventory.selected_tests_iter().count();
    let offset = position(cursor, &id, &sha, total)?;
    let mut records = Vec::new();
    let mut used = 0usize;
    let mut next_record_bytes = None;
    for identity in inventory
        .selected_tests_iter()
        .skip(offset)
        .take(limit.get())
    {
        let group = &inventory.groups[identity.group];
        let suite = group
            .listing
            .rust_suites
            .get(&nextest_metadata::RustBinaryId::new(&identity.binary_id))
            .ok_or_else(|| invalid("selected binary is absent"))?;
        let artifact = group
            .build
            .artifacts
            .iter()
            .find(|artifact| artifact.executable.captured == suite.binary.binary_path.as_std_path())
            .ok_or_else(|| invalid("selected binary has no same-build artifact"))?;
        let case = suite
            .test_cases
            .get(&nextest_metadata::TestCaseName::new(&identity.test_name))
            .ok_or_else(|| invalid("selected test is absent"))?;
        let fixtures = declarations
            .fixture_requirements
            .as_ref()
            .map(|requirements| {
                requirements
                    .iter()
                    .filter(|requirement| {
                        crate::test_workflow::matches_requirement(
                            requirement,
                            &identity,
                            &artifact.cargo,
                        )
                    })
                    .cloned()
                    .collect()
            });
        let row = TestRow {
            reference: crate::investigation::ResultReference::new(
                crate::investigation::Locator::FrozenTest {
                    invocation: id.to_string(),
                    report_sha256: report_sha256.clone(),
                    inventory_sha256: Some(sha.clone()),
                    selection: crate::investigation::FrozenTestSelection::Test {
                        identity: identity.clone(),
                    },
                },
            ),
            group_capabilities: group.build.resolution.native_capabilities.clone(),
            fixtures,
            identity,
            target: artifact.cargo.target.name.clone(),
            target_kinds: artifact
                .cargo
                .target
                .kind
                .iter()
                .map(ToString::to_string)
                .collect(),
            features: artifact.cargo.features.iter().cloned().collect(),
            ignored: case.ignored,
        };
        let extent = serde_json::to_vec(&row).map_err(io::Error::other)?.len();
        if used.saturating_add(extent) > byte_budget.get() {
            next_record_bytes = Some(extent);
            break;
        }
        used += extent;
        records.push(row);
    }
    let next_offset = offset + records.len();
    let next = (next_offset < total).then(|| Cursor {
        invocation: id.clone(),
        inventory_sha256: sha.clone(),
        offset: next_offset,
    });
    let mut response = Envelope::response(
        "inspect tests",
        format!("{} of {total} original selected tests", records.len()),
        TestPage {
            original_invocation: id,
            inventory_sha256: sha,
            selected_tests: total,
            records,
            next,
            next_record_bytes,
        },
    );
    response.observation.scope = "retained_native_test_inventory_page".into();
    response.observation.limitations=vec!["Listing evidence only: fixture readiness, executable reuse eligibility, physical settlement and test acceptance remain with their original owners.".into(),"The full retained inventory is decoded; page limits bound whole-record output, not underlying scan cost.".into()];
    Ok(response)
}
fn position(
    cursor: Option<&Cursor>,
    id: &CliInvocationId,
    digest: &str,
    total: usize,
) -> Result<usize, HostError> {
    match cursor {
        None => Ok(0),
        Some(cursor)
            if &cursor.invocation == id
                && cursor.inventory_sha256 == digest
                && cursor.offset <= total =>
        {
            Ok(cursor.offset)
        }
        Some(_) => Err(invalid(
            "continuation identity, inventory bytes or extent changed",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failure_pages_retain_exact_native_identities_and_wrapper_runner_distinction() {
        let mut source = serde_json::json!({"failures":[],"observations":[{"kind":"test","group":2,"observation":{"report":"/owned/group-2/junit.xml","summary":{"failed":[{"group":2,"package_id":"opaque:original","binary_id":"same-name","test_name":"module::case"}],"child_exit":1,"native_child_exit":0,"fixture_settled":false,"setup":"succeeded_by_runner"}}}]});
        source["observations"][0]["observation"]["failures"] = serde_json::json!([{"identity":source["observations"][0]["observation"]["summary"]["failed"][0].clone(),"message":"original assertion","description":"complete native assertion\nwith source location"}]);
        let rows = failure_rows(&serde_json::to_vec(&source).unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(
            matches!(&rows[0],FailureRow::Test {identity,description:Some(description),..} if identity.group==2 && identity.binary_id=="same-name" && description=="complete native assertion\nwith source location")
        );
        assert!(matches!(
            &rows[1],
            FailureRow::Runner {
                child_exit: 1,
                native_child_exit: Some(0),
                fixture_settled: Some(false),
                ..
            }
        ));
    }

    #[test]
    fn continuation_cannot_rebind_original_identity_or_changed_inventory() {
        let id = CliInvocationId::new();
        let cursor = Cursor {
            invocation: id.clone(),
            inventory_sha256: "original".into(),
            offset: 2,
        };
        assert_eq!(position(Some(&cursor), &id, "original", 3).unwrap(), 2);
        assert!(position(Some(&cursor), &CliInvocationId::new(), "original", 3).is_err());
        assert!(position(Some(&cursor), &id, "changed", 3).is_err());
        assert!(position(Some(&cursor), &id, "original", 1).is_err());
    }
    #[test]
    fn frozen_failure_addresses_bind_order_and_exact_report_separately_from_population_cursors() {
        let source = serde_json::json!({"failures":[],"observations":[{"kind":"test","group":2,"observation":{"report":"/owned/junit.xml","summary":{"failed":[{"group":2,"package_id":"original","binary_id":"binary","test_name":"same"}],"child_exit":1,"native_child_exit":0,"fixture_settled":false,"setup":"succeeded_by_runner"}}}]});
        let mut rows = failure_rows(&serde_json::to_vec(&source).unwrap()).unwrap();
        let invocation = CliInvocationId::new();
        let sha = "a".repeat(64);
        address_failures(&mut rows, &invocation, &sha);
        for (ordinal, row) in rows.iter().enumerate() {
            let reference = match row {
                FailureRow::Diagnostic { reference, .. }
                | FailureRow::Test { reference, .. }
                | FailureRow::Runner { reference, .. } => reference.as_ref().unwrap(),
            };
            reference.validate().unwrap();
            assert!(
                matches!(&reference.locator, crate::investigation::Locator::FrozenTest { invocation: original, report_sha256, inventory_sha256: None, selection: crate::investigation::FrozenTestSelection::Failure { ordinal: original_ordinal } } if original == invocation.as_str() && report_sha256 == &sha && *original_ordinal == ordinal)
            );
        }
        assert!(matches!(
            rows[1],
            FailureRow::Runner {
                native_child_exit: Some(0),
                fixture_settled: Some(false),
                ..
            }
        ));
    }
    #[test]
    fn evidence_reads_refuse_symlinked_files_and_nonprivate_directories() {
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
        let scratch = tempfile::tempdir().unwrap();
        let path = scratch.path().join("private");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        checked_parent(&path).unwrap();
        let original = path.join("original.json");
        fs::write(&original, b"{\"original\":true}").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o400)).unwrap();
        assert_eq!(read_private(&original).unwrap(), b"{\"original\":true}");
        std::os::unix::fs::symlink(&original, path.join("alias.json")).unwrap();
        assert!(read_private(&path.join("alias.json")).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(checked_parent(&path).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(read_private(&original).is_err());
    }

    #[test]
    fn developer_evidence_source_world_separates_sealed_owners_and_passive_facts() {
        use pse_contract::boundary_projection::{ConstructionRole, Shape};
        let graph = developer_evidence_projection().unwrap();
        for name in [
            "NativeWorkflowReport",
            "CapturedWorkflowReport",
            "CapturedWorkflowEvidence",
        ] {
            assert_eq!(
                graph.definitions()[name].construction,
                ConstructionRole::ObservationOnly
            );
        }
        for name in [
            "WorkflowReportObservation",
            "ErrorRecord",
            "TestIdentity",
            "ContentIdentity",
            "WorkflowTerminalRecord",
        ] {
            assert_eq!(
                graph.definitions()[name].construction,
                ConstructionRole::Data
            );
        }
        let Shape::Record(fields) = &graph.definitions()["CapturedWorkflowEvidence"].shape else {
            panic!("record expected")
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            [
                "invocation",
                "report_original",
                "inventory_original",
                "inventory_sha256",
                "report_observation",
                "terminal",
                "selected_tests",
                "selected_population_complete",
                "recorded_manager_settlement",
                "recorded_native_lifetime_pending"
            ]
        );
        assert!(
            !graph
                .definitions()
                .values()
                .any(|owner| owner.rust_type.contains("GroupObservation")
                    || owner.rust_type.contains("TerminalResult"))
        );
    }
}
