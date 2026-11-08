// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected test inputs delegate preparation and process lifetime to their original owners.
use super::*;
use crate::{
    correctness, discovery,
    environment::Environment,
    paths::{CheckoutState, HostPaths},
    test_workflow::GroupCommand,
};
use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    process::Command,
};

fn pointer(root: &Path, environment: &Environment) -> Result<PathBuf, FixtureError> {
    Ok(CheckoutState::resolve(root, environment)
        .map_err(HostError::from)?
        .fixtures()
        .join("study-module.json"))
}

/// Explicitly prepare the production module and atomically select its immutable receipt.
/// No canonical service or mutable scientific database is created.
/// # Errors
/// Original module preparation, private pointer publication or synchronization fails.
pub async fn prepare_module(
    root: &Path,
    environment: &Environment,
) -> Result<study_module::StudyModuleReceipt, FixtureError> {
    let tools = HostPaths::new(environment)
        .surreal_tools()
        .map_err(HostError::from)?;
    let module =
        study_module::prepare(root, environment, &tools, study_module::Variant::Production).await?;
    let path = pointer(root, environment)?;
    let parent = path
        .parent()
        .ok_or_else(|| invalid("fixture pointer has no parent"))?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)
        .map_err(HostError::from)?;
    let mut output = tempfile::NamedTempFile::new_in(parent).map_err(HostError::from)?;
    serde_json::to_writer(&mut output, &module.record())?;
    output.flush().map_err(HostError::from)?;
    output.as_file().sync_all().map_err(HostError::from)?;
    output
        .persist(&path)
        .map_err(|error| HostError::from(error.error))?;
    fs::File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(HostError::from)?;
    Ok(module)
}

/// Select an explicit receipt or the checkout's prepared pointer without building anything.
/// # Errors
/// The pointer is absent, unowned or invalid, or the original module owner refuses its receipt.
pub fn selected_module(
    root: &Path,
    environment: &Environment,
) -> Result<study_module::StudyModuleReceipt, FixtureError> {
    let path = if let Some(path) = environment.get(OsStr::new("PSE_NATIVE_FIXTURE_MODULE_RECEIPT"))
    {
        PathBuf::from(path)
    } else {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(pointer(root, environment)?)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    FixtureError::Unprepared(Input::StudyModule)
                } else {
                    FixtureError::Owner(HostError::from(error))
                }
            })?;
        let metadata = file.metadata().map_err(HostError::from)?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
            || metadata.len() > 16 * 1024
        {
            return Err(invalid(
                "fixture module pointer requires bounded private owned bytes",
            ));
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(HostError::from)?;
        serde_json::from_slice(&bytes)?
    };
    Ok(study_module::select(
        &path,
        study_module::Variant::Production,
    )?)
}

/// Source-captured implementation tools and immutable owner-selected module for one run.
/// This does not own a service or grant fixture disposal authority.
#[derive(Debug)]
pub struct PreparedExecution {
    module: Option<study_module::StudyModuleReceipt>,
    tools: BTreeMap<Input, Executable>,
    scientific_workers: BTreeMap<usize, Executable>,
    runner: Option<Executable>,
    reader: Option<crate::product_reader::ProductReaderSelection>,
    native_lane: Option<native_lane::SelectedNativeLane>,
}
/// Exact post-list prerequisite account. Assessment uses existing owner selection and
/// validation; the original selections follow it into explicit helper preparation.
pub struct ExecutionInputs {
    root: PathBuf,
    environment: Environment,
    requirements: Vec<Requirement>,
    demand: BTreeSet<Input>,
    module: Option<study_module::StudyModuleReceipt>,
    native_lane: Option<native_lane::SelectedNativeLane>,
    observations: Vec<Readiness>,
    failures: Vec<InputFailure>,
}
impl std::fmt::Debug for ExecutionInputs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutionInputs")
            .field("requirement_count", &self.requirements.len())
            .field("environment_count", &self.environment.len())
            .field("demand", &self.demand)
            .field("failure_count", &self.failures.len())
            .finish_non_exhaustive()
    }
}
impl ExecutionInputs {
    /// Inspect every exact selected input without source-building helpers or starting services.
    /// Opaque caller selectors remain subject to their target owner's admission.
    pub fn assess(root: &Path, environment: &Environment, requirements: &[Requirement]) -> Self {
        let demand: BTreeSet<_> = requirements
            .iter()
            .flat_map(|requirement| requirement.inputs().iter().copied())
            .collect();
        let mut failures = Vec::new();
        let mut states = BTreeMap::new();
        let mut native_lane = None;
        let borrowed = requirements
            .iter()
            .any(|requirement| requirement.ownership() == Ownership::RunnerBorrowed);
        let mut module = None;
        for &input in &demand {
            if let Some((variable, other, variant)) = qualification_module(input) {
                let result = if environment.contains_key(OsStr::new(other))
                    || demand.iter().any(|&selected| {
                        selected != input && qualification_module(selected).is_some()
                    }) {
                    Err(invalid(
                        "select one exact Study qualification variant per invocation",
                    ))
                } else {
                    environment
                        .get(OsStr::new(variable))
                        .ok_or(FixtureError::Unprepared(input))
                        .and_then(|path| {
                            study_module::select(Path::new(path), variant).map_err(Into::into)
                        })
                };
                match result {
                    Ok(receipt) => {
                        states.insert(
                            input,
                            InputState::Selected {
                                path: receipt.record(),
                                identity: receipt.sha256.clone(),
                            },
                        );
                    }
                    Err(cause) => record_refusal(&mut states, &mut failures, input, cause),
                }
                continue;
            }
            if input == Input::NativeLane {
                match native_lane::selected_for_run(root, environment) {
                    Ok(Some(selected)) => {
                        states.insert(
                            input,
                            InputState::OwnerAdmissionRequired {
                                path: Some(selected.path().into()),
                            },
                        );
                        native_lane = Some(selected);
                        continue;
                    }
                    Ok(None) => {}
                    Err(cause) => {
                        record_refusal(&mut states, &mut failures, input, cause);
                        continue;
                    }
                }
            }
            if let Some(variable) = original_input_variable(input, environment) {
                match environment
                    .get(OsStr::new(variable))
                    .filter(|value| !value.is_empty())
                {
                    Some(path) => {
                        states.insert(
                            input,
                            InputState::OwnerAdmissionRequired {
                                path: Some(path.into()),
                            },
                        );
                    }
                    None => record_refusal(
                        &mut states,
                        &mut failures,
                        input,
                        FixtureError::Unprepared(input),
                    ),
                }
                continue;
            }
            match input {
                Input::StudyModule if borrowed || !recorded_module_selected(environment) => {
                    match selected_module(root, environment) {
                        Ok(selected) => {
                            states.insert(
                                input,
                                InputState::Selected {
                                    path: selected.record(),
                                    identity: selected.sha256.clone(),
                                },
                            );
                            module = Some(selected);
                        }
                        Err(cause) => record_refusal(&mut states, &mut failures, input, cause),
                    }
                }
                Input::Server => match server::select(environment) {
                    Ok(selected) => {
                        states.insert(
                            input,
                            InputState::Selected {
                                path: selected.executable(),
                                identity: selected.recipe().into(),
                            },
                        );
                    }
                    Err(cause) => record_refusal(&mut states, &mut failures, input, cause.into()),
                },
                _ => {
                    states.insert(
                        input,
                        InputState::OwnerAdmissionRequired {
                            path: (input == Input::StudyModule)
                                .then(|| {
                                    environment
                                        .get(OsStr::new("PSE_NATIVE_FIXTURE_MODULE_STATE"))
                                        .map(PathBuf::from)
                                })
                                .flatten(),
                        },
                    );
                }
            }
        }
        let observations = requirements
            .iter()
            .map(|requirement| Readiness {
                ownership: requirement.ownership(),
                inputs: states
                    .iter()
                    .filter(|(input, _)| requirement.inputs().contains(input))
                    .map(|(input, state)| (*input, state.clone()))
                    .collect(),
            })
            .collect();
        Self {
            root: root.into(),
            environment: environment.clone(),
            requirements: requirements.into(),
            demand,
            module,
            native_lane,
            observations,
            failures,
        }
    }

    /// Complete input account in original requirement order, without any execution grant.
    pub fn observations(&self) -> &[Readiness] {
        &self.observations
    }

    /// Whether an original input owner refused readiness, before helper effects.
    pub fn blocked(&self) -> bool {
        !self.failures.is_empty()
    }

    /// Source-build demanded helpers only after every independent input check completed.
    /// # Errors
    /// All original prerequisite failures, or the original helper preparation failure.
    pub fn prepare(self, destination: &Path) -> Result<PreparedExecution, FixtureError> {
        self.prepare_with_permit(destination, None)
    }

    pub(crate) fn prepare_with_permit(
        self,
        destination: &Path,
        permit: Option<&crate::build_admission::Permit>,
    ) -> Result<PreparedExecution, FixtureError> {
        let Self {
            root,
            environment,
            requirements,
            demand,
            module,
            native_lane,
            failures,
            ..
        } = self;
        if !failures.is_empty() {
            return Err(FixtureError::Inputs(failures));
        }
        let root = root.as_path();
        let environment = &environment;
        let requirements = requirements.as_slice();
        if demand.contains(&Input::ServiceHelper)
            || demand.contains(&Input::ScientificWorker)
            || demand.contains(&Input::ObservationHelper)
        {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(destination)
                .map_err(HostError::from)?;
        }
        let borrowed = requirements
            .iter()
            .any(|requirement| requirement.ownership() == Ownership::RunnerBorrowed);
        let mut selected = PreparedExecution {
            module,
            tools: BTreeMap::new(),
            scientific_workers: BTreeMap::new(),
            runner: None,
            reader: None,
            native_lane,
        };
        if demand.contains(&Input::ServiceHelper) || borrowed {
            let mut bins = vec!["pse-canonical-service"];
            if borrowed {
                bins.push("pse-canonical-test");
            }
            let artifacts = build(
                root,
                environment,
                &destination.join("canonical-tools"),
                "pse-operations",
                &["native-service".into()],
                &bins,
            )?;
            for (name, executable) in artifacts {
                match name.as_str() {
                    "pse-canonical-service" => {
                        selected.tools.insert(Input::ServiceHelper, executable);
                    }
                    "pse-canonical-test" => selected.runner = Some(executable),
                    _ => {}
                }
            }
        }
        if demand.contains(&Input::ScientificWorker) {
            let mut groups = BTreeMap::<usize, BTreeSet<String>>::new();
            for requirement in requirements
                .iter()
                .filter(|requirement| requirement.inputs().contains(&Input::ScientificWorker))
            {
                let features = groups
                    .entry(requirement.group())
                    .or_insert_with(|| BTreeSet::from(["property-coolprop".into()]));
                features.extend(requirement.unit().features.iter().cloned());
            }
            // All other build inputs are fixed by this invocation: checkout, environment
            // (including compiler/native selection), package, binary and host target.
            // Coalesce actual helper build requests, not the independent fixture groups.
            selected.scientific_workers =
                capture_scientific_worker_groups(groups, |group, features| {
                    let artifacts = build(
                        root,
                        environment,
                        &destination.join(format!("scientific-worker-{group}")),
                        "pse-runtime",
                        features,
                        &["pse-scientific-worker"],
                    )?;
                    artifacts
                        .into_iter()
                        .find_map(|(name, executable)| {
                            (name == "pse-scientific-worker").then_some(executable)
                        })
                        .ok_or_else(|| invalid("source-built scientific worker is absent"))
                })?;
        }
        if demand.contains(&Input::ProductReader) {
            selected.reader = Some(
                crate::product_reader::prepare_with_permit(root, environment, false, permit)
                    .map_err(|error| HostError::ProductReader(Box::new(error)))?,
            );
        }
        if demand.contains(&Input::ObservationHelper) {
            let artifacts = build(
                root,
                environment,
                &destination.join("observation-helper"),
                "pse-runtime",
                &["product-reader".into()],
                &["pse-observation-fixture"],
            )?;
            let executable = artifacts
                .into_iter()
                .find_map(|(name, executable)| {
                    (name == "pse-observation-fixture").then_some(executable)
                })
                .ok_or_else(|| invalid("source-built runtime observation helper is absent"))?;
            selected.tools.insert(Input::ObservationHelper, executable);
        }
        Ok(selected)
    }
}

fn record_refusal(
    states: &mut BTreeMap<Input, InputState>,
    failures: &mut Vec<InputFailure>,
    input: Input,
    cause: FixtureError,
) {
    let cause = std::sync::Arc::new(cause);
    let state = if matches!(cause.as_ref(), FixtureError::Unprepared(_)) {
        InputState::Unprepared {
            code: pse_diagnostics::DiagnosticCode::FixtureUnprepared
                .as_str()
                .into(),
            next: prepare(input),
        }
    } else {
        InputState::Refused {
            code: pse_diagnostics::TypedDiagnostic::diagnostic_code(cause.as_ref())
                .map(|code| code.as_str().into()),
            next: prepare(input),
        }
    };
    states.insert(input, state);
    failures.push(InputFailure { input, cause });
}

impl PreparedExecution {
    /// Project already prepared inputs onto one selected native command. Borrowed cases use
    /// the existing product fixture runner; target-owned cases only receive their inputs.
    /// # Errors
    /// A required captured input changed or the selected native command has invalid metadata.
    pub fn command(
        &self,
        mut command: GroupCommand,
        requirements: &[Requirement],
        inventory: &Path,
        state_parent: &Path,
        invocation: &crate::envelope::CliInvocationId,
    ) -> Result<GroupCommand, FixtureError> {
        let relevant: Vec<_> = requirements
            .iter()
            .filter(|requirement| requirement.group() == command.group)
            .collect();
        let demand: BTreeSet<_> = relevant
            .iter()
            .flat_map(|requirement| requirement.inputs().iter().copied())
            .collect();
        for (input, variable) in [
            (
                Input::QualificationStudyModule,
                QUALIFICATION_MODULE_RECEIPT_ENV,
            ),
            (Input::LongHoldStudyModule, LONG_HOLD_MODULE_RECEIPT_ENV),
        ] {
            if !demand.contains(&input) {
                command.environment.remove(OsStr::new(variable));
            }
        }
        if demand.contains(&Input::NativeLane)
            && let Some(selected) = &self.native_lane
        {
            selected.lend(&command.cwd, &mut command.environment)?;
        }
        let borrowed = relevant
            .iter()
            .any(|requirement| requirement.ownership() == Ownership::RunnerBorrowed);
        if demand.contains(&Input::StudyModule)
            && (borrowed || !recorded_module_selected(&command.environment))
        {
            let module = self
                .module
                .as_ref()
                .ok_or_else(|| invalid("selected module input is unavailable"))?;
            command.environment.insert(
                "PSE_NATIVE_FIXTURE_MODULE_RECEIPT".into(),
                module.record().into_os_string(),
            );
        }
        for (input, variable) in [
            (Input::ServiceHelper, "PSE_NATIVE_FIXTURE_SERVICE_HELPER"),
            (
                Input::ObservationHelper,
                "PSE_NATIVE_FIXTURE_OBSERVATION_HELPER",
            ),
        ] {
            if demand.contains(&input) {
                let executable = self
                    .tools
                    .get(&input)
                    .ok_or_else(|| invalid("required source-built fixture helper is absent"))?;
                executable.verify()?;
                command
                    .environment
                    .insert(variable.into(), executable.captured.as_os_str().to_owned());
            }
        }
        if demand.contains(&Input::ScientificWorker) {
            let executable = self
                .scientific_workers
                .get(&command.group)
                .ok_or_else(|| invalid("required source-built scientific worker is absent"))?;
            executable.verify()?;
            command.environment.insert(
                "PSE_SCIENTIFIC_WORKER_BINARY".into(),
                executable.captured.as_os_str().to_owned(),
            );
        }
        if demand.contains(&Input::ProductReader) {
            // Native correctness commands deliberately drop managed execution markers.
            // Publish the original workflow ID as fixture data, not process authority.
            command.environment.insert(
                "PSE_NATIVE_FIXTURE_ORIGINAL_INVOCATION".into(),
                invocation.as_str().into(),
            );
            let selected = self
                .reader
                .as_ref()
                .ok_or_else(|| invalid("prepared product reader input is absent"))?;
            let reader = crate::product_reader::ProductReaderSelection::open(
                &command.cwd,
                &command.environment,
                selected.id(),
            )
            .map_err(|error| HostError::ProductReader(Box::new(error)))?;
            command.environment.insert(
                "PSE_NATIVE_FIXTURE_PRODUCT_READER".into(),
                reader.id().into(),
            );
            let generation = crate::generation::prepared(&command.cwd)?
                .ok_or_else(|| invalid("declared product consumer CLI is unprepared"))?;
            let cli = generation.executable(&command.cwd, crate::generation::HostTool::Pse)?;
            command
                .environment
                .insert("PSE_NATIVE_FIXTURE_CLI".into(), cli.into_os_string());
        }
        if borrowed {
            let runner = self
                .runner
                .as_ref()
                .ok_or_else(|| invalid("source-built canonical test owner is absent"))?;
            runner.verify()?;
            let helper = self
                .tools
                .get(&Input::ServiceHelper)
                .ok_or_else(|| invalid("canonical service helper is absent"))?;
            let module = self
                .module
                .as_ref()
                .ok_or_else(|| invalid("production module is absent"))?;
            let mut policy = Vec::new();
            let mut arguments = command.arguments.into_iter();
            if arguments.next().as_deref() != Some(OsStr::new("nextest"))
                || arguments.next().as_deref() != Some(OsStr::new("run"))
            {
                return Err(invalid(
                    "fixture owner requires a native nextest run command",
                ));
            }
            while let Some(argument) = arguments.next() {
                if argument == "--binaries-metadata" || argument == "--cargo-metadata" {
                    arguments
                        .next()
                        .ok_or_else(|| invalid("native metadata selector lacks a value"))?;
                } else {
                    policy.push(argument);
                }
            }
            command.program = runner.captured.clone();
            command.arguments = vec![
                "--root".into(),
                command.cwd.as_os_str().to_owned(),
                "--captured-inventory".into(),
                inventory.as_os_str().to_owned(),
                "--group".into(),
                command.group.to_string().into(),
                "--helper".into(),
                helper.captured.as_os_str().to_owned(),
                "--module-receipt".into(),
                module.record().into_os_string(),
                "--state-parent".into(),
                state_parent.as_os_str().to_owned(),
                "--outcome".into(),
                state_parent
                    .join(format!("fixture-outcome-{}.json", command.group))
                    .into_os_string(),
                "--".into(),
            ];
            command.arguments.extend(policy);
        }
        Ok(command)
    }
}

fn capture_scientific_worker_groups(
    groups: BTreeMap<usize, BTreeSet<String>>,
    mut capture: impl FnMut(usize, &[String]) -> Result<Executable, FixtureError>,
) -> Result<BTreeMap<usize, Executable>, FixtureError> {
    let mut builds = BTreeMap::<BTreeSet<String>, Vec<usize>>::new();
    for (group, features) in groups {
        builds.entry(features).or_default().push(group);
    }
    let mut workers = BTreeMap::new();
    for (features, groups) in builds {
        let executable = capture(groups[0], &features.into_iter().collect::<Vec<_>>())?;
        for group in groups {
            workers.insert(group, executable.clone());
        }
    }
    Ok(workers)
}

fn recorded_module_selected(environment: &Environment) -> bool {
    // The target's existing NativeModuleSource owner interprets this opaque state selector.
    // Preserve its original explicit-receipt precedence, including an explicitly empty value.
    !environment.contains_key(OsStr::new("PSE_NATIVE_FIXTURE_MODULE_RECEIPT"))
        && environment.contains_key(OsStr::new("PSE_NATIVE_FIXTURE_MODULE_STATE"))
}

fn qualification_module(
    input: Input,
) -> Option<(&'static str, &'static str, study_module::Variant)> {
    match input {
        Input::QualificationStudyModule => Some((
            QUALIFICATION_MODULE_RECEIPT_ENV,
            LONG_HOLD_MODULE_RECEIPT_ENV,
            study_module::Variant::Qualification,
        )),
        Input::LongHoldStudyModule => Some((
            LONG_HOLD_MODULE_RECEIPT_ENV,
            QUALIFICATION_MODULE_RECEIPT_ENV,
            study_module::Variant::LongHold,
        )),
        _ => None,
    }
}

fn original_input_variable(input: Input, environment: &Environment) -> Option<&'static str> {
    match input {
        Input::Server
        | Input::StudyModule
        | Input::QualificationStudyModule
        | Input::LongHoldStudyModule
        | Input::ServiceHelper
        | Input::Worker
        | Input::ScientificInputs
        | Input::ScientificWorker
        | Input::ProductReader
        | Input::ObservationHelper => None,
        Input::NativeLane => Some(native_lane::VARIABLE),
        Input::EngineeringEndpoint => Some("PSE_ENGINEERING_ENDPOINT"),
        Input::CanonicalTools => Some("PSE_CANONICAL_TOOLS"),
        Input::DiagnosticEvidence => Some("PSE_DIAGNOSTIC_EVIDENCE"),
        Input::PredecessorModuleState => Some(
            if environment.contains_key(OsStr::new(PREDECESSOR_MODULE_RECEIPT_ENV)) {
                PREDECESSOR_MODULE_RECEIPT_ENV
            } else {
                PREDECESSOR_MODULE_STATE_ENV
            },
        ),
        Input::ProducerFixtureReceipt => Some("PSE_PRODUCER_FIXTURE_RECEIPT"),
        Input::WorkerProducerReceipt => Some("PSE_WORKER_PRODUCER_RECEIPT"),
        Input::DeploymentArtifactObservations => Some("PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS"),
        Input::CovarianceReferencePython => Some("PSE_COVARIANCE_REFERENCE_PYTHON"),
        Input::OrdinaryReferencePython => Some("PSE_ORDINARY_REFERENCE_PYTHON"),
        Input::DegeneracyReferencePython => Some("PSE_DEGENERACY_REFERENCE_PYTHON"),
        Input::SensitivityReferencePython => Some("PSE_SENSITIVITY_REFERENCE_PYTHON"),
        Input::PidReferencePython => Some("PSE_PID_REFERENCE_PYTHON"),
        Input::TraceProcessor => Some("PSE_TRACE_PROCESSOR"),
        Input::CanonicalRestartControl => Some("PSE_CANONICAL_RESTART_CONTROL"),
    }
}

fn build(
    root: &Path,
    environment: &Environment,
    destination: &Path,
    package: &str,
    features: &[String],
    bins: &[&str],
) -> Result<Vec<(String, Executable)>, FixtureError> {
    let captured = build_capture(root, environment, destination, package, features, bins)?;
    Ok(captured
        .artifacts
        .into_iter()
        .filter(|artifact| bins.contains(&artifact.cargo.target.name.as_str()))
        .map(|artifact| (artifact.cargo.target.name.to_string(), artifact.executable))
        .collect())
}

pub(super) fn build_capture(
    root: &Path,
    environment: &Environment,
    destination: &Path,
    package: &str,
    features: &[String],
    bins: &[&str],
) -> Result<discovery::BuildCapture, FixtureError> {
    build_capture_with(
        root,
        environment,
        destination,
        package,
        features,
        bins,
        |_| Ok(()),
    )
    .map(|(capture, ())| capture)
}

pub(super) fn build_capture_with<T>(
    root: &Path,
    environment: &Environment,
    destination: &Path,
    package: &str,
    features: &[String],
    bins: &[&str],
    before: impl FnOnce(&correctness::Resolution) -> Result<T, FixtureError>,
) -> Result<(discovery::BuildCapture, T), FixtureError> {
    let cargo = correctness::real_cargo_in(environment)?;
    let mut arguments: Vec<OsString> = vec![
        "build".into(),
        "--locked".into(),
        "-p".into(),
        package.into(),
        "--features".into(),
        features.join(",").into(),
    ];
    for bin in bins {
        arguments.extend(["--bin".into(), (*bin).into()]);
    }
    let resolution = correctness::resolve_with(&arguments, |arguments| {
        let output = Command::new(&cargo)
            .args(arguments)
            .current_dir(root)
            .env_clear()
            .envs(environment)
            .stderr(std::process::Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(correctness::CorrectnessError::Invalid(
                "fixture helper source admission failed".into(),
            ));
        }
        Ok(output.stdout)
    })?;
    let observation = before(&resolution)?;
    let mut command = Command::new(&cargo);
    command
        .args(&resolution.arguments)
        .arg("--message-format=json")
        .current_dir(root)
        .env_clear()
        .envs(environment);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(destination)
        .map_err(HostError::from)?;
    let status =
        discovery::capture::execute_build(command, &resolution, destination, &mut std::io::sink())?;
    if status != 0 {
        return Err(invalid(format!(
            "fixture helper source build exited {status}"
        )));
    }
    Ok((
        discovery::BuildCapture::read(&destination.join("build.json"))?,
        observation,
    ))
}

#[cfg(test)]
mod capture_tests {
    use super::*;

    #[test]
    fn equivalent_helper_requests_share_capture_with_distinct_group_associations() {
        let groups = BTreeMap::from([
            (1, BTreeSet::from(["base".into()])),
            (2, BTreeSet::from(["base".into()])),
            (3, BTreeSet::from(["base".into(), "extra".into()])),
        ]);
        let mut calls = Vec::new();
        let workers = capture_scientific_worker_groups(groups, |group, features| {
            calls.push(features.to_vec());
            Ok(Executable {
                original: format!("original-{group}").into(),
                captured: format!("captured-{group}").into(),
                sha256: group.to_string(),
                bytes: 1,
            })
        })
        .unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(workers.len(), 3);
        assert_eq!(workers[&1], workers[&2]);
        assert_ne!(workers[&1], workers[&3]);
        let mut empty_calls = 0;
        assert!(
            capture_scientific_worker_groups(BTreeMap::new(), |_, _| {
                empty_calls += 1;
                Err(invalid("empty selection must not capture"))
            })
            .unwrap()
            .is_empty()
        );
        assert_eq!(empty_calls, 0);
    }
}
