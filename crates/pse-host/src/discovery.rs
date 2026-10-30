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
    os::unix::fs::DirBuilderExt,
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
    /// Original discovery failure and independent generation settlement failure.
    #[error("test discovery failed: {operation}; generation settlement also failed: {settlement}")]
    Settlement {
        /// Original failure.
        operation: Box<DiscoveryError>,
        /// Independent failure to release the original host generation use.
        settlement: HostError,
    },
    /// Original phase child/required raw-capture failures, without dropping independent causes.
    #[error("test discovery {stage} has incomplete raw capture at {evidence}")]
    Capture {
        /// Fixed native phase.
        stage: &'static str,
        /// Original retained evidence location.
        evidence: PathBuf,
        /// Original independent failures.
        failures: Vec<crate::test_workflow::WorkflowError>,
    },
}
pse_diagnostics::impl_diagnostic! {
    DiscoveryError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(this) { match this {Self::Capture{failures,..}=>Some(Box::new(failures.iter().map(|error|{let error:&dyn pse_diagnostics::TypedDiagnostic=error;error}))),_=>None} }, source(_this) { None }
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

/// Original required phase custody, retained before native I/O errors are projected.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseCapture {
    /// Fixed performing phase: version, build or list.
    pub stage: String,
    /// Original compiler context, absent for the version probe.
    pub group: Option<usize>,
    /// Original waited exit.
    pub child_exit: i32,
    /// Required original stdout retention.
    pub stdout: crate::managed::result::StreamSummary,
    /// Required original stderr retention.
    pub stderr: crate::managed::result::StreamSummary,
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
    /// Every required version/build/list phase retained complete original raw output.
    pub required_capture_complete: bool,
    /// Actual version/build/list required stream facts, rather than child-zero inference.
    pub capture: Vec<PhaseCapture>,
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
}
impl Inventory {
    /// Required version/build/list raw custody must all be known, independently of child exit.
    pub fn capture_complete(&self) -> bool {
        let stream = |value: &crate::managed::result::StreamSummary| {
            value.complete
                && value.eof
                && !value.io_unresolved
                && value.bytes_read == value.bytes_retained
        };
        let phase = |stage: &str, group: Option<usize>| {
            self.capture
                .iter()
                .filter(|capture| capture.stage == stage && capture.group == group)
                .count()
                == 1
        };
        self.version == 2
            && self.required_capture_complete
            && self.capture.len() == 1 + self.groups.len() * 2
            && phase("version", None)
            && (0..self.groups.len())
                .all(|group| phase("build", Some(group)) && phase("list", Some(group)))
            && self.capture.iter().all(|capture| {
                capture.child_exit == 0 && stream(&capture.stdout) && stream(&capture.stderr)
            })
    }
    /// Actual selected tests; duplicate names in different binaries remain distinct.
    pub fn selected_tests(&self) -> Vec<TestIdentity> {
        self.selected_tests_iter().collect()
    }
    /// Traverse selected identities without constructing an entire presentation population.
    pub fn selected_tests_iter(&self) -> impl Iterator<Item = TestIdentity> + '_ {
        self.groups.iter().enumerate().flat_map(|(group, listed)| {
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
    }
    /// Zero selected matches is a population fact, never a successful workload result.
    pub fn is_zero_match(&self) -> bool {
        self.selected_tests_iter().next().is_none()
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
        if !self.capture_complete() {
            return Err(invalid(
                "inventory lacks supported complete discovery capture",
            ));
        }
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
async fn output(
    command: Command,
    directory: &Path,
    stage: &'static str,
    policy: crate::managed::payload::PayloadPolicy,
    group: Option<usize>,
    phases: &mut Vec<PhaseCapture>,
) -> Result<Output, DiscoveryError> {
    let stdout = directory.join(format!("{stage}.stdout"));
    let stderr = directory.join(format!("{stage}.stderr"));
    let capture = crate::test_workflow::execute::phase::run(
        command,
        &stdout,
        &stderr,
        crate::managed::result::GenericObserver,
        policy,
    )
    .await
    .map_err(|error| DiscoveryError::Capture {
        stage,
        evidence: directory.into(),
        failures: vec![error],
    })?;
    if !capture.capture_complete || !capture.failures.is_empty() || capture.child_exit.is_none() {
        return Err(DiscoveryError::Capture {
            stage,
            evidence: directory.into(),
            failures: capture.failures,
        });
    }
    let status = capture
        .status
        .ok_or_else(|| invalid("captured phase child status missing"))?;
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
    phases.push(PhaseCapture {
        stage: stage.into(),
        group,
        child_exit: capture
            .child_exit
            .ok_or_else(|| invalid("captured phase exit missing"))?,
        stdout: capture
            .stdout
            .ok_or_else(|| invalid("captured phase stdout missing"))?,
        stderr: capture
            .stderr
            .ok_or_else(|| invalid("captured phase stderr missing"))?,
    });
    Ok(result)
}

/// Explicitly build and list selected tests, retaining immutable executable copies.
/// The destination must be a new absolute directory under the caller's retained owner.
/// No native provider preparation or fixture/service creation occurs here.
pub async fn list(
    resolved: &selection::Resolved,
    checkout: &Path,
    nextest: &Path,
    destination: &Path,
    runner_profile: &str,
    environment: &crate::environment::Environment,
    policy: crate::managed::payload::PayloadPolicy,
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
    let copies = (|| {
        directory(destination)?;
        let nextest = capture::executable(nextest, &destination.join("cargo-nextest"))?;
        let correctness = capture::executable(
            &tools
                .join(&prepared.generation)
                .join("cargo-pse-correctness"),
            &destination.join("cargo-pse-correctness"),
        )?;
        Ok((nextest, correctness))
    })();
    let (nextest, captured_correctness) = match (copies, generation_use.settle()) {
        (Ok(value), Ok(())) => value,
        (Err(error), Ok(())) => return Err(error),
        (Ok(_), Err(error)) => return Err(error.into()),
        (Err(operation), Err(settlement)) => {
            return Err(DiscoveryError::Settlement {
                operation: Box::new(operation),
                settlement,
            });
        }
    };
    // Every child uses captured controls. The original host generation is provenance only
    // and can be retired independently after its copy has been authenticated and settled.
    async {
        let mut phases=Vec::new();
        let mut version = Command::new(&nextest.captured);
        version
            .arg("--version")
            .env_clear()
            .envs(environment)
            .current_dir(&checkout);
        let version = output(version, destination, "version",policy,None,&mut phases).await?;
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
        let cargo = crate::correctness::real_cargo_in(environment)?;
        let nextest_configuration = fs::read(checkout.join(".config/nextest.toml"))?;
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
                .env_clear()
                .envs(environment)
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
                .env("CARGO", &captured_correctness.captured)
                .env("PSE_CORRECTNESS_ADAPTER", "1")
                .env("PSE_CORRECTNESS_REAL_CARGO", &cargo)
                .env("PSE_FIXTURE_CARGO_METADATA_CAPTURE", &cargo_metadata)
                .env("PSE_CORRECTNESS_BUILD_CAPTURE", &build_directory)
                .env("PSE_CORRECTNESS_EXECUTABLE", &captured_correctness.captured)
                .env_remove("NEXTEST_TEST_PHASE");
            if resolved.plan().filters().ignore_default_filter {build.arg("--ignore-default-filter");}
            let binary_output = output(
                build,
                &group_directory,
                "build",
                policy,
                Some(index),
                &mut phases,
            ).await?;
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
                .env_clear()
                .envs(environment)
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
            let listing_output = output(enumerate, &group_directory, "list",policy,Some(index),&mut phases).await?;
            let listing: TestListSummary = serde_json::from_slice(&listing_output.stdout)?;
            validate_listing(&binaries, &listing, resolved.plan().ignored())?;
            groups.push(ListedGroup {
                build: captured,
                binaries,
                listing,
            });
        }
        let inventory = Inventory {
            version: 2,
            required_capture_complete: true,
            capture:phases,
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
        };
        capture::write_json(&destination.join("inventory.json"), &inventory)?;
        Ok(inventory)
    }.await
}

pub(crate) fn add_filters(command: &mut Command, plan: &selection::Plan) {
    if plan.filters().ignore_default_filter {
        command.arg("--ignore-default-filter");
    }
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
