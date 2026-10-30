// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Test selection stages, immutable listed commands and required observation, without launch.
mod check;
pub mod execute;
mod observation;
mod source;
use crate::{
    catalog::description::Information,
    correctness::{self, CorrectnessError},
    discovery::{self, DiscoveryError, Inventory, TestIdentity},
    environment::Environment,
    extraction::{CompletedReport, ExtractionError},
    fixtures::{self, FixtureError, Requirement},
    selection::{self, Plan, Request, Resolved},
};
pub use check::{
    AdmittedCheck, CheckIntent, CompilerFailureReference, CompilerObservation,
    CompilerObservationError, CompilerObserver, PreparedCheck,
};
pub use observation::{CompletedObserver, FailureReference, Observation, ObservationError};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

/// Checked stage transition or an explicitly unresolved preparation owner.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Original filesystem failure.
    #[error("test workflow I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Child status was not established; the enclosing managed scope still owns its descendants.
    #[error("workflow phase could not wait for child {pid:?}: {source}")]
    ChildWait {
        /// Original spawned leader, diagnostic only.
        pid: Option<u32>,
        /// Original native wait cause.
        source: std::io::Error,
    },
    /// A native phase returned its original nonzero waited status.
    #[error("workflow {phase} exited {exit}; original output remains in phase evidence")]
    PhaseExit {
        /// Fixed operation phase.
        phase: &'static str,
        /// Original waited child exit, preserving signal encoding.
        exit: i32,
    },
    /// Original selected-test record attribution and required protocol cause.
    #[error(transparent)]
    TestObservation(#[from] ObservationError),
    /// Original compiler record attribution and required protocol cause.
    #[error(transparent)]
    CompilerObservation(#[from] CompilerObservationError),
    /// Original native host/custody failure.
    #[error(transparent)]
    Host(#[from] crate::HostError),
    /// Original preparation error and independent custody-settlement error.
    #[error("check preparation failed: {operation}; custody settlement also failed: {settlement}")]
    Settlement {
        /// Original preparation failure.
        operation: Box<WorkflowError>,
        /// Original independent settlement failure.
        settlement: crate::HostError,
    },
    /// Original correctness admission refusal.
    #[error(transparent)]
    Correctness(#[from] CorrectnessError),
    /// Original explicit build/list failure.
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
    /// Original fixture declaration failure.
    #[error(transparent)]
    Fixture(#[from] FixtureError),
    /// Original required-protocol refusal.
    #[error(transparent)]
    Extraction(#[from] ExtractionError),
    /// A captured owning declaration changed; explicit re-resolution is required.
    #[error("test workflow input changed at {0}; explicitly re-resolve before effects")]
    Changed(PathBuf),
    /// A stage or captured association is inconsistent.
    #[error("test workflow: {0}")]
    Invalid(String),
    /// Native capability preparation/readiness remains with its original owner.
    #[error("test execution requires native-owner readiness for {0:?}")]
    NativePending(BTreeSet<String>),
    /// Original native provider selection refused; no preparation was attempted.
    #[error("native workflow readiness: {0}")]
    NativeReadiness(#[source] crate::HostError),
    /// Listing never creates fixtures or proves their execution-owner readiness.
    #[error("test execution requires fixture-owner readiness for selected group {0}")]
    FixturePending(usize),
}
pse_diagnostics::impl_diagnostic! {
    WorkflowError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { match this { Self::Host(error)|Self::NativeReadiness(error) => Some(error), Self::Correctness(error) => Some(error), Self::Discovery(error) => Some(error), Self::Fixture(error) => Some(error), Self::Extraction(error) => Some(error),Self::TestObservation(error)=>Some(error),Self::CompilerObservation(error)=>Some(error), _ => None } },
    help(_this) { None }, related(this) { match this { Self::Settlement { operation, settlement } => { let errors: [&dyn pse_diagnostics::TypedDiagnostic; 2] = [operation.as_ref(), settlement]; Some(Box::new(errors.into_iter())) }, _ => None } }, source(_this) { None }
}
#[derive(Debug)]
struct Input {
    path: PathBuf,
    digest: Option<[u8; 32]>,
}
fn digest(path: &Path) -> Result<Option<[u8; 32]>, std::io::Error> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(Sha256::digest(bytes).into())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
struct Context {
    checkout: PathBuf,
    metadata: cargo_metadata::Metadata,
    profile: String,
    environment: Environment,
    inputs: Vec<Input>,
    selected_native: BTreeSet<String>,
}
impl std::fmt::Debug for Context {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Context")
            .field("environment_count", &self.environment.len())
            .field("owning_input_count", &self.inputs.len())
            .finish_non_exhaustive()
    }
}
impl Context {
    fn capture(
        metadata: cargo_metadata::Metadata,
        plan: &Plan,
        profile: String,
        environment: Environment,
    ) -> Result<Self, WorkflowError> {
        let checkout = metadata.workspace_root.as_std_path().canonicalize()?;
        let mut paths: BTreeSet<_> = [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            ".cargo/config",
            ".cargo/config.toml",
        ]
        .iter()
        .map(|path| checkout.join(path))
        .collect();
        if plan.operation() == selection::Operation::Test {
            paths.extend([
                checkout.join(".config/nextest.toml"),
                checkout.join(".config/diagnostic-tools.toml"),
            ]);
        }
        for root in plan.groups().iter().flat_map(|group| group.roots()) {
            let package = metadata
                .packages
                .iter()
                .find(|package| package.id.repr == root.package_id())
                .ok_or_else(|| {
                    WorkflowError::Invalid("selected package metadata is absent".into())
                })?;
            paths.insert(package.manifest_path.as_std_path().to_owned());
        }
        let inputs = paths
            .into_iter()
            .map(|path| digest(&path).map(|digest| Input { path, digest }))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            checkout,
            metadata,
            profile,
            environment,
            inputs,
            selected_native: BTreeSet::new(),
        })
    }
    fn admit(&self, plan: Plan) -> Result<Resolved, WorkflowError> {
        self.verify()?;
        let cargo = correctness::real_cargo_in(&self.environment)?;
        let mut admission = correctness::AdmissionContext::default();
        let resolved = plan.resolve_with(|arguments| {
            admission.resolve_with(arguments, |arguments| {
                correctness::capture_at(&cargo, arguments, &self.checkout, &self.environment)
            })
        })?;
        self.verify()?;
        Ok(resolved)
    }
    fn verify(&self) -> Result<(), WorkflowError> {
        for input in &self.inputs {
            if digest(&input.path)? != input.digest {
                return Err(WorkflowError::Changed(input.path.clone()));
            }
        }
        Ok(())
    }
    fn select_native(&mut self, resolved: &Resolved) -> Result<(), WorkflowError> {
        let capabilities = resolved.native_capabilities();
        if capabilities.is_empty() {
            return Ok(());
        }
        self.verify()?;
        let environment = crate::native::select_capabilities(
            &self.checkout,
            &self.environment,
            &capabilities.iter().cloned().collect::<Vec<_>>().join(","),
        )
        .map_err(WorkflowError::NativeReadiness)?;
        let previous = std::mem::replace(&mut self.environment, environment);
        let checked = self.admit(resolved.plan().clone());
        let same = checked.as_ref().is_ok_and(|checked| {
            checked.plan() == resolved.plan()
                && checked.admitted().len() == resolved.admitted().len()
                && checked
                    .admitted()
                    .iter()
                    .zip(resolved.admitted())
                    .all(|(left, right)| {
                        left.arguments == right.arguments
                            && left.native_capabilities == right.native_capabilities
                            && left.units.len() == right.units.len()
                            && left.units.iter().all(|unit| right.units.contains(unit))
                    })
        });
        if !same {
            self.environment = previous;
            checked?;
            return Err(WorkflowError::Invalid(
                "compiler context changed after native selection; explicitly re-resolve".into(),
            ));
        }
        self.selected_native = capabilities;
        Ok(())
    }
}
/// Explicit available semantic stage. Later stages are never fabricated by explain.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Normalized declarations, before actual correctness admission.
    Intent,
    /// Actual compiler units, before explicit build/list.
    Admitted,
    /// Actual captured binaries and native test population.
    Listed,
}
/// Projection of the same operation-owned selection at its currently available stage.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Explanation<'a> {
    /// Current owned stage.
    pub stage: Stage,
    /// Original normalized selection, including roots/platform/features/filter/ignored policy.
    pub selection: &'a Plan,
    /// Actual correctness admission only when it has been performed.
    pub admission: Information<&'a [correctness::Resolution]>,
    /// Actual selected population only after explicit build/list.
    pub selected_tests: Information<usize>,
    /// Fixture-owner declarations only after admitted feature context is available.
    pub fixture_requirements: Information<&'a [Requirement]>,
    /// Explicit later obligations; none is silently prepared by this projection.
    pub pending: Vec<&'static str>,
}

/// Actual selected root contexts, omitting the reached dependency graph from the default view.
#[derive(Debug, Serialize, JsonSchema)]
pub struct AdmittedGroup<'a> {
    /// Literal compiler invocation owned by the correctness adapter.
    pub arguments: &'a [OsString],
    /// Actual roots; host/target context and feature policies retain their original meaning.
    pub roots: Vec<&'a correctness::CompilerUnit>,
    /// Reached compiler-unit count; omitted units are available through explicit full detail.
    pub reached_units: usize,
    /// Actual capability closure derived by the correctness owner.
    pub native_capabilities: &'a BTreeSet<String>,
}
/// Producer-owned useful default explanation. Full dependency units require explicit detail.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BriefExplanation<'a> {
    /// Available semantic stage.
    pub stage: Stage,
    /// The same normalized intent used by execution.
    pub selection: &'a Plan,
    /// Actual root contexts only, or the original unavailable reason.
    pub admission: Information<Vec<AdmittedGroup<'a>>>,
    /// Actual population only after native listing.
    pub selected_tests: Information<usize>,
    /// Authored requirements for the selected admitted roots.
    pub fixture_requirements: Information<&'a [Requirement]>,
    /// Explicit remaining obligations; no preparation was performed by presentation.
    pub pending: Vec<&'static str>,
}
impl<'a> Explanation<'a> {
    /// Select root-level context before serialization, preserving the full operation value
    /// for callers that explicitly request the reached dependency graph.
    pub fn brief(self) -> BriefExplanation<'a> {
        let admission = match self.admission {
            Information::Available(groups) => Information::Available(
                groups
                    .iter()
                    .map(|group| AdmittedGroup {
                        arguments: &group.arguments,
                        roots: group.units.iter().filter(|unit| unit.root).collect(),
                        reached_units: group.units.len(),
                        native_capabilities: &group.native_capabilities,
                    })
                    .collect(),
            ),
            Information::Unavailable { reason } => Information::Unavailable { reason },
        };
        BriefExplanation {
            stage: self.stage,
            selection: self.selection,
            admission,
            selected_tests: self.selected_tests,
            fixture_requirements: self.fixture_requirements,
            pending: self.pending,
        }
    }
}
fn pending<T>(reason: &str) -> Information<T> {
    Information::Unavailable {
        reason: reason.into(),
    }
}
pub(crate) fn matches_requirement(
    requirement: &Requirement,
    identity: &TestIdentity,
    artifact: &cargo_metadata::Artifact,
) -> bool {
    requirement.group() == identity.group
        && requirement.package_id() == identity.package_id
        && requirement.target_name() == artifact.target.name
        && requirement.unit().source_path == artifact.target.src_path.as_str()
        && requirement.unit().target_kinds.len() == artifact.target.kind.len()
        && requirement
            .unit()
            .target_kinds
            .iter()
            .zip(&artifact.target.kind)
            .all(|(expected, actual)| expected == &actual.to_string())
        && requirement.applies_to_test(&identity.test_name)
}

/// Normalized declaration-backed intent. Construction reads only explicit owning inputs.
#[derive(Debug)]
pub struct Intent {
    context: Context,
    plan: Plan,
}
impl Intent {
    /// Consume caller-captured metadata and complete selected environment; never build/list.
    pub fn new(
        metadata: cargo_metadata::Metadata,
        request: Request,
        profile: String,
        environment: Environment,
    ) -> Result<Self, WorkflowError> {
        if profile.is_empty() {
            return Err(WorkflowError::Invalid("runner profile is empty".into()));
        }
        let (plan, profile) = source::resolve(&metadata, request, profile, &environment)?;
        if plan.operation() != selection::Operation::Test {
            return Err(WorkflowError::Invalid(
                "test workflow requires test intent".into(),
            ));
        }
        Ok(Self {
            context: Context::capture(metadata, &plan, profile, environment)?,
            plan,
        })
    }
    /// Pure projection: this does not query Cargo, prepare providers or create state.
    pub fn explain(&self) -> Explanation<'_> {
        Explanation {
            stage: Stage::Intent,
            selection: &self.plan,
            admission: pending("correctness admission not requested"),
            selected_tests: pending("explicit build/list not requested"),
            fixture_requirements: pending("admitted feature context is pending"),
            pending: vec![
                "correctness_admission",
                "explicit_build_and_list",
                "external_runtime_readiness",
                "physical_settlement",
            ],
        }
    }
    /// Admit through the existing correctness owner in this exact checkout/environment.
    pub fn admit(self) -> Result<Admitted, WorkflowError> {
        let resolved = self.context.admit(self.plan)?;
        let requirements = fixtures::requirements(&self.context.metadata, &resolved)?;
        Ok(Admitted {
            context: self.context,
            resolved,
            requirements,
        })
    }
}
/// Actual admitted compiler context, still without fabricated binaries or test names.
#[derive(Debug)]
pub struct Admitted {
    context: Context,
    resolved: Resolved,
    requirements: Vec<Requirement>,
}
impl Admitted {
    /// Read the actual owner result for capability/fixture consumers.
    pub fn resolved(&self) -> &Resolved {
        &self.resolved
    }
    /// Explain available admission without performing explicit build/list.
    pub fn explain(&self) -> Explanation<'_> {
        Explanation {
            stage: Stage::Admitted,
            selection: self.resolved.plan(),
            admission: Information::Available(self.resolved.admitted()),
            selected_tests: pending("explicit build/list not requested"),
            fixture_requirements: Information::Available(&self.requirements),
            pending: vec![
                "explicit_build_and_list",
                "external_runtime_readiness",
                "physical_settlement",
            ],
        }
    }
    /// Explicit source build/list; no service or native-provider preparation is performed.
    pub async fn list(
        self,
        nextest: &Path,
        destination: &Path,
        options: discovery::DiscoveryOptions,
    ) -> Result<Listed, WorkflowError> {
        self.context.verify()?;
        let inventory = discovery::list(
            &self.resolved,
            &self.context.checkout,
            nextest,
            destination,
            &self.context.profile,
            &self.context.environment,
            options,
        )
        .await?;
        self.context.verify()?;
        let mut paths = vec![destination.join("nextest.toml")];
        for group in 0..inventory.groups.len() {
            let directory = destination.join(format!("group-{group}"));
            paths.extend([
                directory.join("cargo-metadata.json"),
                directory.join("binaries.json"),
            ]);
        }
        let controls = paths
            .into_iter()
            .map(|path| digest(&path).map(|digest| Input { path, digest }))
            .collect::<Result<_, _>>()?;
        Ok(Listed {
            context: self.context,
            resolved: self.resolved,
            requirements: self.requirements,
            inventory,
            capture_root: destination.to_owned(),
            controls,
        })
    }
}
/// Explicit immutable-binary reuse. This never claims those bytes reflect later source edits.
#[derive(Debug)]
pub struct Listed {
    context: Context,
    resolved: Resolved,
    requirements: Vec<Requirement>,
    inventory: Inventory,
    capture_root: PathBuf,
    controls: Vec<Input>,
}
/// Byte-exact launch-neutral request, consumed by the shared invocation supervisor.
pub struct GroupCommand {
    /// Captured executable; mutable target/install paths are never reused.
    pub program: PathBuf,
    /// Literal operating-system arguments, without shell interpretation.
    pub arguments: Vec<OsString>,
    /// Explicit selected checkout.
    pub cwd: PathBuf,
    /// Complete child environment; consumers apply env_clear before envs.
    pub environment: Environment,
    /// Captured admitted build/feature group.
    pub group: usize,
    /// Discovery's explicit remaining runtime-input boundary, never a ready-launch assertion.
    pub runtime_inputs: discovery::RuntimeInputObservation,
}
impl std::fmt::Debug for GroupCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GroupCommand")
            .field("argument_count", &self.arguments.len())
            .field("environment_count", &self.environment.len())
            .field("group", &self.group)
            .field("runtime_inputs", &self.runtime_inputs)
            .finish_non_exhaustive()
    }
}
/// One whole semantic test record in a producer-owned page.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TestRecord<'a> {
    /// Exact actual binary/test identity under captured feature context.
    pub identity: TestIdentity,
    /// Cargo's actual target name from the captured compiler artifact.
    pub target: &'a str,
    /// Cargo's actual target kinds, never inferred from event name spelling.
    pub target_kinds: Vec<String>,
    /// Actual compiled features of this exact artifact.
    pub features: &'a [String],
    /// Capabilities of this captured compiler group; not an invented per-test dependency graph.
    pub group_capabilities: &'a BTreeSet<String>,
    /// Matching declarations, still pending their actual fixture owner.
    pub fixtures: Vec<&'a Requirement>,
}
/// A selected immutable inventory page; no arbitrary population cap is imposed.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TestPage<'a> {
    /// Original captured owner association for exact replay/rerun consumers.
    pub inventory: PathBuf,
    /// Whole records selected after actual native enumeration.
    pub records: Vec<TestRecord<'a>>,
    /// Matching selected population, independent of visible page length.
    pub total: usize,
    /// Next ordinal within this captured inventory, when known records remain.
    pub continuation: Option<usize>,
    /// Immutable captured bytes, not a claim about current source content.
    pub source_basis: &'static str,
}
impl Listed {
    /// Original immutable owner inventory; generation provenance is independent of run custody.
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    /// Project the already observed population without rebuilding/relisting it.
    pub fn page(&self, offset: usize, limit: usize) -> Result<TestPage<'_>, WorkflowError> {
        if limit == 0 {
            return Err(WorkflowError::Invalid(
                "test page limit must be positive".into(),
            ));
        }
        let total = self.inventory.selected_tests_iter().count();
        let mut records = Vec::new();
        for identity in self
            .inventory
            .selected_tests_iter()
            .skip(offset)
            .take(limit)
        {
            let captured = self.inventory.test(&identity)?;
            let fixtures = self
                .requirements
                .iter()
                .filter(|requirement| {
                    matches_requirement(requirement, &identity, &captured.artifact.cargo)
                })
                .collect();
            records.push(TestRecord {
                group_capabilities: &self.inventory.groups[identity.group]
                    .build
                    .resolution
                    .native_capabilities,
                target: &captured.artifact.cargo.target.name,
                target_kinds: captured
                    .artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                features: &captured.artifact.cargo.features,
                fixtures,
                identity,
            });
        }
        let next = offset.saturating_add(records.len());
        Ok(TestPage {
            inventory: self.capture_root.join("inventory.json"),
            records,
            total,
            continuation: (next < total).then_some(next),
            source_basis: "immutable_captured_binary_inventory",
        })
    }
    /// Explain this captured stage without new preparation or current-source claims.
    pub fn explain(&self) -> Explanation<'_> {
        Explanation {
            stage: Stage::Listed,
            selection: self.resolved.plan(),
            admission: Information::Available(self.resolved.admitted()),
            selected_tests: Information::Available(self.inventory.selected_tests_iter().count()),
            fixture_requirements: Information::Available(&self.requirements),
            pending: vec!["external_runtime_readiness", "physical_settlement"],
        }
    }
    /// Build an ordinary native reused-binary request. No Cargo build-scope flags are forwarded
    /// to reused execution: native metadata owns that scope, while profile/filter policy is exact.
    pub fn command(&self, group: usize) -> Result<GroupCommand, WorkflowError> {
        self.context.verify()?;
        for control in &self.controls {
            if digest(&control.path)? != control.digest {
                return Err(WorkflowError::Changed(control.path.clone()));
            }
        }
        self.inventory.verify_executables()?;
        let listed = self
            .inventory
            .groups
            .get(group)
            .ok_or_else(|| WorkflowError::Invalid("captured group is absent".into()))?;
        if !listed
            .build
            .resolution
            .native_capabilities
            .is_subset(&self.context.selected_native)
        {
            return Err(WorkflowError::NativePending(
                listed.build.resolution.native_capabilities.clone(),
            ));
        }
        let configuration = self.capture_root.join("nextest.toml");
        if fs::read(&configuration)? != self.inventory.nextest_configuration {
            return Err(WorkflowError::Changed(configuration));
        }
        let directory = self.capture_root.join(format!("group-{group}"));
        let binaries = directory.join("binaries.json");
        let captured: nextest_metadata::BinaryListSummary =
            serde_json::from_slice(&fs::read(&binaries)?).map_err(DiscoveryError::from)?;
        if serde_json::to_vec(&captured).map_err(DiscoveryError::from)?
            != serde_json::to_vec(&listed.binaries).map_err(DiscoveryError::from)?
        {
            return Err(WorkflowError::Changed(binaries));
        }
        let mut command = std::process::Command::new(&self.inventory.nextest.captured);
        command
            .args([
                "nextest",
                "run",
                "--profile",
                &self.inventory.runner_profile,
            ])
            .arg("--config-file")
            .arg(self.run_configuration(group)?)
            .arg("--binaries-metadata")
            .arg(binaries)
            .arg("--cargo-metadata")
            .arg(directory.join("cargo-metadata.json"));
        discovery::add_group_filters(&mut command, &self.inventory.selection, group);
        let mut environment = self.context.environment.clone();
        environment.remove(std::ffi::OsStr::new("NEXTEST_EXPERIMENTAL_LIBTEST_JSON"));
        environment.remove(std::ffi::OsStr::new("NEXTEST_TEST_PHASE"));
        Ok(GroupCommand {
            program: self.inventory.nextest.captured.clone(),
            arguments: command.get_args().map(std::ffi::OsStr::to_owned).collect(),
            cwd: self.context.checkout.clone(),
            environment,
            group,
            runtime_inputs: self.inventory.runtime_inputs,
        })
    }
    fn report_path(&self, group: usize) -> PathBuf {
        self.capture_root.join(format!("group-{group}/junit.xml"))
    }
    fn run_configuration(&self, group: usize) -> Result<PathBuf, WorkflowError> {
        let mut configuration: toml::Value = toml::from_str(
            std::str::from_utf8(&self.inventory.nextest_configuration).map_err(|error| {
                WorkflowError::Invalid(format!(
                    "captured native configuration is not UTF-8: {error}"
                ))
            })?,
        )
        .map_err(|error| {
            WorkflowError::Invalid(format!("captured nextest configuration: {error}"))
        })?;
        let profile = configuration
            .get_mut("profile")
            .and_then(|profiles| profiles.get_mut(&self.inventory.runner_profile))
            .and_then(toml::Value::as_table_mut)
            .ok_or_else(|| {
                WorkflowError::Invalid(
                    "selected native profile is absent from captured configuration".into(),
                )
            })?;
        let report = self.report_path(group);
        let report = report.to_str().ok_or_else(|| {
            WorkflowError::Invalid("native JUnit report path is not UTF-8".into())
        })?;
        let junit = profile
            .entry("junit".to_owned())
            .or_insert_with(|| toml::Value::Table(Default::default()))
            .as_table_mut()
            .ok_or_else(|| {
                WorkflowError::Invalid("captured JUnit configuration is not a table".into())
            })?;
        junit.insert("path".into(), toml::Value::String(report.to_owned()));
        let derived = toml::to_string(&configuration).map_err(|error| {
            WorkflowError::Invalid(format!("derived nextest configuration: {error}"))
        })?;
        let path = self
            .capture_root
            .join(format!("group-{group}/nextest-run.toml"));
        fs::write(&path, derived)?;
        Ok(path)
    }
    /// Bind the native completed report; raw output has no lifecycle interpreter.
    pub fn observer(&self, group: usize) -> Result<CompletedObserver, WorkflowError> {
        Ok(CompletedObserver::new(CompletedReport::new(
            &self.inventory,
            group,
            self.report_path(group),
        )?))
    }
}

#[cfg(test)]
mod tests;
