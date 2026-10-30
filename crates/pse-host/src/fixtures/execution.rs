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
    workers: BTreeMap<usize, Executable>,
    runner: Option<Executable>,
}
impl PreparedExecution {
    /// Source-build only the implementation tools demanded by actual selected tests.
    /// # Errors
    /// A selected input is unavailable or original source admission/capture fails.
    pub fn prepare(
        root: &Path,
        environment: &Environment,
        destination: &Path,
        requirements: &[Requirement],
    ) -> Result<Self, FixtureError> {
        let demand: BTreeSet<_> = requirements
            .iter()
            .flat_map(|requirement| requirement.inputs().iter().copied())
            .collect();
        for &input in &demand {
            if let Some(variable) = original_input_variable(input) {
                if environment
                    .get(OsStr::new(variable))
                    .is_none_or(|value| value.is_empty())
                {
                    return Err(FixtureError::Unprepared(input));
                }
            }
        }
        if demand.contains(&Input::ServiceHelper) || demand.contains(&Input::Worker) {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(destination)
                .map_err(HostError::from)?;
        }
        let module = if demand.contains(&Input::StudyModule) {
            Some(selected_module(root, environment)?)
        } else {
            None
        };
        if demand.contains(&Input::Server) {
            server::select(environment)?;
        }
        let mut selected = Self {
            module,
            tools: BTreeMap::new(),
            workers: BTreeMap::new(),
            runner: None,
        };
        let borrowed = requirements
            .iter()
            .any(|requirement| requirement.ownership() == Ownership::RunnerBorrowed);
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
        if demand.contains(&Input::Worker) {
            let mut groups = BTreeMap::<usize, BTreeSet<String>>::new();
            for requirement in requirements
                .iter()
                .filter(|requirement| requirement.inputs().contains(&Input::Worker))
            {
                let features = groups
                    .entry(requirement.group())
                    .or_insert_with(|| BTreeSet::from(["product-operators".to_owned()]));
                features.extend(requirement.unit().features.iter().cloned());
            }
            for (group, features) in groups {
                let artifacts = build(
                    root,
                    environment,
                    &destination.join(format!("worker-{group}")),
                    "pse-runtime",
                    &features.into_iter().collect::<Vec<_>>(),
                    &["pse-worker"],
                )?;
                if let Some(executable) = artifacts
                    .into_iter()
                    .find_map(|(name, executable)| (name == "pse-worker").then_some(executable))
                {
                    selected.workers.insert(group, executable);
                }
            }
        }
        Ok(selected)
    }

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
    ) -> Result<GroupCommand, FixtureError> {
        let relevant: Vec<_> = requirements
            .iter()
            .filter(|requirement| requirement.group() == command.group)
            .collect();
        let demand: BTreeSet<_> = relevant
            .iter()
            .flat_map(|requirement| requirement.inputs().iter().copied())
            .collect();
        if demand.contains(&Input::StudyModule) {
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
            (Input::Worker, "PSE_WORKER_BINARY"),
        ] {
            if demand.contains(&input) {
                let executable = if input == Input::Worker {
                    self.workers.get(&command.group)
                } else {
                    self.tools.get(&input)
                }
                .ok_or_else(|| invalid("required source-built fixture helper is absent"))?;
                executable.verify()?;
                command
                    .environment
                    .insert(variable.into(), executable.captured.as_os_str().to_owned());
            }
        }
        if relevant
            .iter()
            .any(|requirement| requirement.ownership() == Ownership::RunnerBorrowed)
        {
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

fn original_input_variable(input: Input) -> Option<&'static str> {
    match input {
        Input::Server | Input::StudyModule | Input::ServiceHelper | Input::Worker => None,
        Input::ScientificInputs => Some("PSE_SCIENTIFIC_INPUTS"),
        Input::CanonicalTools => Some("PSE_CANONICAL_TOOLS"),
        Input::DiagnosticEvidence => Some("PSE_DIAGNOSTIC_EVIDENCE"),
        Input::PredecessorModuleState => Some("PSE_NATIVE_FIXTURE_PREDECESSOR_MODULE_STATE"),
        Input::ProducerFixtureReceipt => Some("PSE_PRODUCER_FIXTURE_RECEIPT"),
        Input::WorkerProducerReceipt => Some("PSE_WORKER_PRODUCER_RECEIPT"),
        Input::DeploymentArtifactObservations => Some("PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS"),
        Input::CovarianceReferencePython => Some("PSE_COVARIANCE_REFERENCE_PYTHON"),
        Input::DegeneracyReferencePython => Some("PSE_DEGENERACY_REFERENCE_PYTHON"),
        Input::SensitivityReferencePython => Some("PSE_SENSITIVITY_REFERENCE_PYTHON"),
        Input::PidReferencePython => Some("PSE_PID_REFERENCE_PYTHON"),
        Input::NativeLane => Some("PSE_NATIVE_LANE_EXECUTABLE"),
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
    let mut command = Command::new(&cargo);
    command
        .args(&resolution.arguments)
        .arg("--message-format=json")
        .current_dir(root)
        .env_clear()
        .envs(environment);
    let status =
        discovery::capture::execute_build(command, &resolution, destination, &mut std::io::sink())?;
    if status != 0 {
        return Err(invalid(format!(
            "fixture helper source build exited {status}"
        )));
    }
    let captured: discovery::BuildCapture = serde_json::from_slice(
        &fs::read(destination.join("build.json")).map_err(HostError::from)?,
    )?;
    Ok(captured
        .artifacts
        .into_iter()
        .filter(|artifact| bins.contains(&artifact.cargo.target.name.as_str()))
        .map(|artifact| (artifact.cargo.target.name.to_string(), artifact.executable))
        .collect())
}
