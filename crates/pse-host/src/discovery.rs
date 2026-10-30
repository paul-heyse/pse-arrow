// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit source-build/list discovery over admitted selections and captured binary bytes.
//! Listing establishes a population, never setup success or a reconciled test pass.
pub(crate) mod capture;
pub use capture::{BuildCapture, BuiltArtifact, Executable};

use crate::{
    HostError,
    correctness::CorrectnessError,
    generation,
    selection::{self, IgnoredPolicy, Operation},
};
use nextest_metadata::{BinaryListSummary, RustTestSuiteStatusSummary, TestListSummary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Failure of explicit discovery, distinct from selected-test outcomes.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// Native file or tool operation failed.
    #[error("test discovery I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Public upstream metadata could not be decoded.
    #[error("test discovery metadata: {0}")]
    Json(#[from] serde_json::Error),
    /// Correctness admission or artifact capture refused.
    #[error(transparent)]
    Correctness(#[from] CorrectnessError),
    /// Prepared host generation ownership could not be established or settled.
    #[error(transparent)]
    Host(#[from] HostError),
    /// A listed fact contradicts the admitted source build.
    #[error("test discovery: {0}")]
    Invalid(String),
    /// The actual listing child failed; its original streams remain under evidence.
    #[error("test discovery {stage} exited {exit}; original streams at {evidence}")]
    Command {
        /// Phase that launched the child.
        stage: &'static str,
        /// Child status, preserving signal encoding.
        exit: i32,
        /// Private retained phase directory.
        evidence: PathBuf,
    },
    /// Waiting failed without proving the original listing child exited; protection remains.
    #[error(
        "test discovery {stage} cannot establish child {pid} exit: {detail}; evidence at {evidence}"
    )]
    Unsettled {
        /// Original phase.
        stage: &'static str,
        /// Original child identity for operator diagnosis, not permission to kill a reused PID.
        pid: u32,
        /// Original wait failure.
        detail: String,
        /// Retained original streams.
        evidence: PathBuf,
    },
    /// Original discovery failure and independent generation settlement failure.
    #[error("test discovery failed: {operation}; generation settlement also failed: {settlement}")]
    Settlement {
        /// Original failure.
        operation: Box<DiscoveryError>,
        /// Independent failure to release the original host generation use.
        settlement: HostError,
    },
}
pse_diagnostics::impl_diagnostic! {
    DiscoveryError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> DiscoveryError {
    DiscoveryError::Invalid(message.into())
}

/// Exact identity of one observed test, including its independent build context.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct TestIdentity {
    /// Selection context; the same binary/test in incompatible feature contexts is distinct.
    pub group: usize,
    /// Opaque Cargo package ID.
    pub package_id: String,
    /// Nextest's stable binary ID.
    pub binary_id: String,
    /// Exact harness test ID.
    pub test_name: String,
}

/// Captured same-build artifact and actual nextest test population.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListedGroup {
    /// Original compiler selection and captured artifacts, never inferred from manifests.
    pub build: BuildCapture,
    /// Public nextest binary metadata, with test paths pointing to private captured bytes.
    pub binaries: BinaryListSummary,
    /// Actual public nextest enumeration and filter-match results.
    pub listing: TestListSummary,
}

/// Listing observes no setup-script execution or outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetupObservation {
    /// The original runner policy was retained, but setup remains unexecuted and unresolved.
    NotRunByListing,
}

/// Executable capture does not freeze linked libraries, build-script outputs or fixture inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeInputObservation {
    /// Actual execution must separately admit/capture its remaining runtime inputs.
    NotCapturedByDiscovery,
}

/// Waiting for listing processes does not establish original physical descendant drain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalScopeObservation {
    /// The enclosing physical owner must still establish settlement.
    NotEstablishedByDiscovery,
}

/// Exact selected-test handoff for reconciliation/rerun, with original binary and policy.
#[derive(Debug, Serialize)]
pub struct CapturedTest<'a> {
    /// Full context/binary/test identity, never a name-only candidate.
    pub identity: TestIdentity,
    /// Same-build artifact owns enabled features, profile and captured executable bytes.
    pub artifact: &'a BuiltArtifact,
    /// Original ignored status and runner filter match.
    pub case: &'a nextest_metadata::RustTestCaseSummary,
    /// Original ignored-test policy; changing it requires explicit re-resolution.
    pub ignored_policy: IgnoredPolicy,
}

/// Completed discovery; its explicit owner decides retention/disposal independently of a run.
#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    /// Host-owned inventory envelope version.
    pub version: u32,
    /// Caller intent remains distinct from the actual listed population.
    pub selection: selection::Plan,
    /// Original prepared host identity, used and settled during this listing.
    pub host_generation: generation::PreparedGeneration,
    /// Captured nextest executable, independent of its original installation path.
    pub nextest: Executable,
    /// Original correctness adapter bytes retained independently of later host pruning.
    pub correctness: Executable,
    /// Exact tool version output observed from the captured executable.
    pub nextest_version: String,
    /// Exact repository policy input used for both build/list phases; not setup evidence.
    pub nextest_configuration: Vec<u8>,
    /// Explicit runner profile shared by build/list phases; ambient profile changes do not apply.
    pub runner_profile: String,
    /// Actual admitted groups, preserving incompatible feature contexts.
    pub groups: Vec<ListedGroup>,
    /// Setup scripts are not executed by listing. This never asserts setup success.
    pub setup: SetupObservation,
    /// Scope limit of captured bytes; inventory alone is not an execution-readiness grant.
    pub runtime_inputs: RuntimeInputObservation,
    /// Independent physical settlement obligation, never inferred from Cargo/nextest exit.
    pub physical_scope: PhysicalScopeObservation,
    /// Live custody for the enclosing physical owner; serialized inventory grants no release.
    #[serde(skip)]
    pub generation_use: Option<generation::GenerationUse>,
}
impl Inventory {
    /// Actual selected tests; duplicate names in different binaries remain distinct.
    pub fn selected_tests(&self) -> Vec<TestIdentity> {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(group, listed)| {
                listed.listing.rust_suites.values().flat_map(move |suite| {
                    suite
                        .test_cases
                        .iter()
                        .filter(|(_, case)| case.filter_match.is_match())
                        .map(move |(name, _)| TestIdentity {
                            group,
                            package_id: suite.binary.package_id.clone(),
                            binary_id: suite.binary.binary_id.to_string(),
                            test_name: name.to_string(),
                        })
                })
            })
            .collect()
    }
    /// Zero selected matches is a population fact, never a successful workload result.
    pub fn is_zero_match(&self) -> bool {
        self.selected_tests().is_empty()
    }
    /// Whether every reported suite was actually enumerated; skipped suites stay visible.
    pub fn all_suites_listed(&self) -> bool {
        self.groups.iter().all(|group| {
            group
                .listing
                .rust_suites
                .values()
                .all(|suite| suite.status == RustTestSuiteStatusSummary::LISTED)
        })
    }
    /// Verify captured binaries before another owner reuses them.
    pub fn verify_executables(&self) -> Result<(), DiscoveryError> {
        self.nextest.verify()?;
        self.correctness.verify()?;
        for group in &self.groups {
            for artifact in &group.build.artifacts {
                artifact.executable.verify()?;
            }
        }
        Ok(())
    }

    /// Resolve an original selected identity to its exact captured binary and feature context.
    pub fn test(&self, identity: &TestIdentity) -> Result<CapturedTest<'_>, DiscoveryError> {
        let group = self
            .groups
            .get(identity.group)
            .ok_or_else(|| invalid("test selection context is absent"))?;
        let suite = group
            .listing
            .rust_suites
            .get(&nextest_metadata::RustBinaryId::new(&identity.binary_id))
            .filter(|suite| suite.binary.package_id == identity.package_id)
            .ok_or_else(|| invalid("test binary/package identity is absent"))?;
        let case = suite
            .test_cases
            .get(&nextest_metadata::TestCaseName::new(&identity.test_name))
            .filter(|case| case.filter_match.is_match())
            .ok_or_else(|| invalid("test was not selected in the original listed binary"))?;
        let artifact = group
            .build
            .artifacts
            .iter()
            .find(|artifact| artifact.executable.captured == suite.binary.binary_path.as_std_path())
            .ok_or_else(|| invalid("test has no original captured executable"))?;
        artifact.executable.verify()?;
        Ok(CapturedTest {
            identity: identity.clone(),
            artifact,
            case,
            ignored_policy: self.selection.ignored(),
        })
    }
}

fn directory(path: &Path) -> Result<(), DiscoveryError> {
    fs::DirBuilder::new().mode(0o700).create(path)?;
    capture::private_directory(path)?;
    Ok(())
}
fn output(
    mut command: Command,
    directory: &Path,
    stage: &'static str,
    child_started: Option<&mut bool>,
) -> Result<Output, DiscoveryError> {
    let stdout = directory.join(format!("{stage}.stdout"));
    let stderr = directory.join(format!("{stage}.stderr"));
    let stdout_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&stdout)?;
    let stderr_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&stderr)?;
    let mut child = command.stdout(stdout_file).stderr(stderr_file).spawn()?;
    if let Some(started) = child_started {
        *started = true;
    }
    let status = child.wait().map_err(|error| DiscoveryError::Unsettled {
        stage,
        pid: child.id(),
        detail: error.to_string(),
        evidence: directory.into(),
    })?;
    let result = Output {
        status,
        stdout: fs::read(stdout)?,
        stderr: fs::read(stderr)?,
    };
    if !result.status.success() {
        use std::os::unix::process::ExitStatusExt as _;
        return Err(DiscoveryError::Command {
            stage,
            exit: result
                .status
                .code()
                .unwrap_or_else(|| 128 + result.status.signal().unwrap_or(1)),
            evidence: directory.into(),
        });
    }
    Ok(result)
}

/// Explicitly build and list selected tests, retaining immutable executable copies.
/// The destination must be a new absolute directory under the caller's retained owner.
/// No native provider preparation or fixture/service creation occurs here.
pub fn list(
    resolved: &selection::Resolved,
    checkout: &Path,
    nextest: &Path,
    destination: &Path,
) -> Result<Inventory, DiscoveryError> {
    if resolved.plan().operation() != Operation::Test {
        return Err(invalid("only test selections have a nextest inventory"));
    }
    if !destination.is_absolute() {
        return Err(invalid("inventory destination must be absolute"));
    }
    let checkout = checkout.canonicalize()?;
    let prepared = generation::prepared(&checkout)?
        .ok_or_else(|| invalid("native host is unprepared; run scripts/bootstrap-host"))?;
    let tools = checkout.join("target/host-tools");
    let generation_use = generation::GenerationUse::acquire(&tools, &prepared.generation)?;
    let mut source_child_started = false;
    let result = (|| {
        directory(destination)?;
        let nextest = capture::executable(nextest, &destination.join("cargo-nextest"))?;
        let mut version = Command::new(&nextest.captured);
        version.arg("--version");
        let version = output(version, destination, "version", None)?;
        let version = String::from_utf8(version.stdout)
            .map_err(|_| invalid("nextest version is not UTF-8"))?;
        let declared: toml::Value = toml::from_str(&fs::read_to_string(
            checkout.join(".config/diagnostic-tools.toml"),
        )?)
        .map_err(|error| invalid(error.to_string()))?;
        let pin = declared["cargo-tools"]["cargo-nextest"]
            .as_str()
            .ok_or_else(|| invalid("nextest tool pin is missing"))?;
        if version.split_whitespace().next() != Some("cargo-nextest")
            || version.split_whitespace().nth(1) != Some(pin)
        {
            return Err(invalid(
                "nextest executable differs from the checkout's declared version",
            ));
        }
        let correctness = tools
            .join(&prepared.generation)
            .join("cargo-pse-correctness");
        let captured_correctness =
            capture::executable(&correctness, &destination.join("cargo-pse-correctness"))?;
        let cargo = crate::correctness::real_cargo()?;
        let nextest_configuration = fs::read(checkout.join(".config/nextest.toml"))?;
        let runner_profile = "default";
        let configuration = destination.join("nextest.toml");
        fs::write(&configuration, &nextest_configuration)?;
        let mut groups = Vec::new();
        for (index, group) in resolved.plan().groups().iter().enumerate() {
            let group_directory = destination.join(format!("group-{index}"));
            directory(&group_directory)?;
            let build_directory = group_directory.join("build");
            directory(&build_directory)?;
            let cargo_metadata = group_directory.join("cargo-metadata.json");
            let mut build = Command::new(&nextest.captured);
            build
                .current_dir(&checkout)
                .args([
                    "nextest",
                    "list",
                    "--list-type=binaries-only",
                    "--message-format=json",
                ])
                .args(["--profile", runner_profile])
                .arg("--config-file")
                .arg(&configuration)
                .args(&group.arguments()[1..])
                .env("CARGO", &correctness)
                .env("PSE_CORRECTNESS_ADAPTER", "1")
                .env("PSE_CORRECTNESS_REAL_CARGO", &cargo)
                .env("PSE_FIXTURE_CARGO_METADATA_CAPTURE", &cargo_metadata)
                .env("PSE_CORRECTNESS_BUILD_CAPTURE", &build_directory)
                .env("PSE_CORRECTNESS_EXECUTABLE", &correctness)
                .env_remove("NEXTEST_TEST_PHASE");
            let binary_output = output(
                build,
                &group_directory,
                "build",
                Some(&mut source_child_started),
            )?;
            let mut binaries: BinaryListSummary = serde_json::from_slice(&binary_output.stdout)?;
            let captured: BuildCapture =
                serde_json::from_slice(&fs::read(build_directory.join("build.json"))?)?;
            if captured.version != 1 || captured.cargo_exit != 0 || !captured.incomplete.is_empty()
            {
                return Err(invalid("source build capture is incomplete or failed"));
            }
            group.validate_resolution(&captured.resolution)?;
            let expected = &resolved.admitted()[index];
            if expected.units.len() != captured.resolution.units.len()
                || expected
                    .units
                    .iter()
                    .any(|unit| !captured.resolution.units.contains(unit))
            {
                return Err(invalid(
                    "compiler-unit selection changed after resolution; explicitly re-resolve before listing",
                ));
            }
            pin_binaries(&mut binaries, &captured)?;
            let binaries_path = group_directory.join("binaries.json");
            capture::write_json(&binaries_path, &binaries)?;
            let _: cargo_metadata::Metadata = serde_json::from_slice(&fs::read(&cargo_metadata)?)?;
            let mut enumerate = Command::new(&nextest.captured);
            enumerate
                .current_dir(&checkout)
                .args([
                    "nextest",
                    "list",
                    "--message-format=json",
                    "--binaries-metadata",
                ])
                .arg(&binaries_path)
                .arg("--cargo-metadata")
                .arg(&cargo_metadata);
            enumerate
                .arg("--config-file")
                .arg(&configuration)
                .env_remove("NEXTEST_TEST_PHASE");
            enumerate.args(["--profile", runner_profile]);
            add_filters(&mut enumerate, resolved.plan());
            let listing_output = output(enumerate, &group_directory, "list", None)?;
            let listing: TestListSummary = serde_json::from_slice(&listing_output.stdout)?;
            validate_listing(&binaries, &listing, resolved.plan().ignored())?;
            groups.push(ListedGroup {
                build: captured,
                binaries,
                listing,
            });
        }
        let inventory = Inventory {
            version: 1,
            selection: resolved.plan().clone(),
            host_generation: prepared,
            nextest,
            correctness: captured_correctness,
            nextest_version: version,
            nextest_configuration,
            runner_profile: runner_profile.into(),
            groups,
            setup: SetupObservation::NotRunByListing,
            runtime_inputs: RuntimeInputObservation::NotCapturedByDiscovery,
            physical_scope: PhysicalScopeObservation::NotEstablishedByDiscovery,
            generation_use: None,
        };
        capture::write_json(&destination.join("inventory.json"), &inventory)?;
        Ok(inventory)
    })();
    if source_child_started {
        return result.map(|mut inventory| {
            inventory.generation_use = Some(generation_use);
            inventory
        });
    }
    match (result, generation_use.settle()) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
        (Err(operation), Err(settlement)) => Err(DiscoveryError::Settlement {
            operation: Box::new(operation),
            settlement,
        }),
    }
}

fn add_filters(command: &mut Command, plan: &selection::Plan) {
    command.args([
        "--run-ignored",
        match plan.ignored() {
            IgnoredPolicy::Default => "default",
            IgnoredPolicy::Only => "only",
            IgnoredPolicy::All => "all",
        },
    ]);
    for expression in &plan.filters().expressions {
        command.arg("-E").arg(expression);
    }
    command.arg("--").args(&plan.filters().words);
}

fn pin_binaries(
    binaries: &mut BinaryListSummary,
    captured: &BuildCapture,
) -> Result<(), DiscoveryError> {
    for (id, binary) in &mut binaries.rust_binaries {
        if *id != binary.binary_id {
            return Err(invalid(
                "nextest binary map key differs from its embedded identity",
            ));
        }
        let mut artifacts = captured.artifacts.iter().filter(|artifact| {
            artifact.cargo.profile.test
                && artifact.cargo.package_id.repr == binary.package_id
                && artifact.cargo.target.name == binary.binary_name
                && artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == binary.kind.as_str())
                && artifact.executable.original == binary.binary_path.as_std_path()
        });
        let selected = artifacts
            .next()
            .ok_or_else(|| invalid("listed binary has no same-build captured Cargo artifact"))?;
        if artifacts.next().is_some() {
            return Err(invalid(
                "listed binary has ambiguous captured Cargo artifacts",
            ));
        }
        selected.executable.verify()?;
        binary.binary_path = selected
            .executable
            .captured
            .to_str()
            .ok_or_else(|| invalid("captured binary path is not UTF-8"))?
            .into();
    }
    Ok(())
}

fn validate_listing(
    binaries: &BinaryListSummary,
    listing: &TestListSummary,
    ignored: IgnoredPolicy,
) -> Result<(), DiscoveryError> {
    if listing.test_count
        != listing
            .rust_suites
            .values()
            .map(|suite| suite.test_cases.len())
            .sum::<usize>()
    {
        return Err(invalid(
            "nextest test count differs from its actual listed records",
        ));
    }
    let expected: BTreeSet<_> = binaries.rust_binaries.keys().collect();
    let actual: BTreeSet<_> = listing.rust_suites.keys().collect();
    if expected != actual {
        return Err(invalid(
            "nextest enumeration omitted or added binary identities",
        ));
    }
    for (id, suite) in &listing.rust_suites {
        if suite.binary != binaries.rust_binaries[id] {
            return Err(invalid(
                "enumerated binary differs from the captured source-built binary",
            ));
        }
        if suite.status != RustTestSuiteStatusSummary::LISTED && !suite.test_cases.is_empty() {
            return Err(invalid("unlisted suite contains fabricated test records"));
        }
        if suite.test_cases.values().any(|case| {
            case.filter_match.is_match()
                && match ignored {
                    IgnoredPolicy::Default => case.ignored,
                    IgnoredPolicy::Only => !case.ignored,
                    IgnoredPolicy::All => false,
                }
        }) {
            return Err(invalid(
                "nextest selected a test incompatible with the original ignored policy",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
