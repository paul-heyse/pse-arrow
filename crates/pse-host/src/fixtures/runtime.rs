// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit capsule/lane preparation through existing Cargo and scientific owners.
use super::*;
use crate::{
    correctness, discovery::BuildCapture, environment::Environment, generation,
    paths::CheckoutState,
};
use sha2::{Digest as _, Sha256};
use std::{
    ffi::OsStr,
    fs,
    io::{Read as _, Write as _},
    os::unix::fs::{
        DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _,
    },
    process::Command,
};

const MAX_CAPSULE: usize = 256 << 20;
const MAX_RECEIPT: usize = 64 << 20;
const CATALOG: &str = "tests/fixtures/scientific-inputs.toml";

/// One retained original artifact; byte identity does not establish scientific readiness.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeInputArtifact {
    /// Retained original bytes, never a guessed Cargo output.
    pub path: PathBuf,
    /// SHA-256 of those exact bytes.
    pub sha256: String,
    /// Original byte extent.
    pub bytes: u64,
}
/// Explicitly selected capsule and private lane inputs, without a live fixture claim.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RuntimeInputs {
    /// Opaque retained preparation identity.
    pub selection: String,
    /// Original protected selection record.
    pub receipt: PathBuf,
    /// Existing scientific-inputs owner output.
    pub scientific_inputs: RuntimeInputArtifact,
    /// Source-built captured private native-lane executable.
    pub native_lane: RuntimeInputArtifact,
    /// Exact original build evidence for both source-owned executables.
    pub builds: Vec<PathBuf>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    checkout: String,
    selection: String,
    capsule: RuntimeInputArtifact,
    sources: Vec<generation::SourceFingerprint>,
}
#[derive(Debug)]
pub(super) struct SelectedRuntime {
    directory: PathBuf,
    record: Record,
    lane: Option<Executable>,
    builder: Option<Executable>,
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn private_directory(path: &Path, create: bool) -> Result<(), FixtureError> {
    if create {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .map_err(HostError::from)?;
    }
    let metadata = fs::symlink_metadata(path).map_err(HostError::from)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(invalid(
            "runtime fixture requires a private owned directory",
        ));
    }
    Ok(())
}
fn read(path: &Path, maximum: usize, mode: u32) -> Result<Vec<u8>, FixtureError> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)
        .map_err(HostError::from)?;
    let metadata = file.metadata().map_err(HostError::from)?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != mode
        || metadata.nlink() != 1
        || metadata.len() > maximum as u64
    {
        return Err(invalid(
            "runtime fixture requires bounded protected original bytes",
        ));
    }
    let before = crate::artifacts::descriptor_identity(&file).map_err(HostError::from)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(HostError::from)?;
    if bytes.len() as u64 != metadata.len()
        || crate::artifacts::descriptor_identity(&file).map_err(HostError::from)? != before
    {
        return Err(invalid(
            "runtime fixture artifact changed during acquisition",
        ));
    }
    Ok(bytes)
}
fn pointer(state: &CheckoutState) -> PathBuf {
    state.fixtures().join("runtime.json")
}

/// Build and capture only the scientific capsule builder and private native lane.
/// # Errors
/// Source, compiler, native admission, capture, child execution or publication refuses.
pub fn prepare_runtime(
    root: &Path,
    environment: &Environment,
) -> Result<RuntimeInputs, FixtureError> {
    let root = root.canonicalize().map_err(HostError::from)?;
    let state = CheckoutState::resolve(&root, environment).map_err(HostError::from)?;
    let environment = construction_environment(&root, environment)?;
    prepare_with(
        &state,
        |directory, selection| {
            let (builder, builder_source) = capture_binary(
                &root,
                &environment,
                &directory.join("capsule-build"),
                "xtask",
                "xtask/src/main.rs",
                "physical-admission",
            )?;
            builder.verify()?;
            let output = directory.join("capsule-output.json");
            let status = Command::new(&builder.captured)
                .args([
                    OsStr::new("scientific-inputs"),
                    OsStr::new(CATALOG),
                    output.as_os_str(),
                ])
                .current_dir(&root)
                .env_clear()
                .envs(&environment)
                .status()
                .map_err(HostError::from)?;
            if !status.success() {
                return Err(invalid(format!("scientific input owner exited {status}")));
            }
            // Freeze the actual owner's output; a failed later lane build cannot select it.
            fs::set_permissions(&output, fs::Permissions::from_mode(0o400))
                .map_err(HostError::from)?;
            let capsule_bytes = read(&output, MAX_CAPSULE, 0o400)?;
            fs::File::open(&output)
                .and_then(|file| file.sync_all())
                .map_err(HostError::from)?;
            let capsule = RuntimeInputArtifact {
                path: output,
                sha256: hash(&capsule_bytes),
                bytes: capsule_bytes.len() as u64,
            };
            let (_, lane_source) = capture_binary(
                &root,
                &environment,
                &directory.join("lane-build"),
                "pse-native-lane",
                "xtask/src/bin/pse-native-lane.rs",
                "solver-kinsol",
            )?;
            Ok(Record {
                version: 1,
                checkout: state.checkout_id.as_str().into(),
                selection: selection.into(),
                capsule,
                sources: vec![builder_source, lane_source],
            })
        },
        |directory, selection| {
            let demand = BTreeSet::from([Input::ScientificInputs, Input::NativeLane]);
            let selected = open_directory(&state, &root, directory, selection, &demand)?;
            selected.validate_current(&root, &environment, &demand)?;
            selected.view()
        },
    )
}

fn prepare_with<T>(
    state: &CheckoutState,
    work: impl FnOnce(&Path, &str) -> Result<Record, FixtureError>,
    validate: impl FnOnce(&Path, &str) -> Result<T, FixtureError>,
) -> Result<T, FixtureError> {
    private_directory(&state.fixtures(), true)?;
    let parent = state.fixtures().join("runtime");
    private_directory(&parent, true)?;
    let selection = uuid::Uuid::new_v4().to_string();
    let directory = parent.join(&selection);
    private_directory(&directory, true)?;
    let record = work(&directory, &selection)?;
    crate::invocation::publish_bytes(
        &directory.join("receipt.json"),
        &serde_json::to_vec(&record)?,
        Some(MAX_RECEIPT),
        0o400,
    )?;
    // Reopen the complete candidate and establish current construction/capsule
    // correspondence before replacing any previous good selected pointer.
    let result = validate(&directory, &selection)?;
    let mut staged = tempfile::NamedTempFile::new_in(state.fixtures()).map_err(HostError::from)?;
    serde_json::to_writer(&mut staged, &selection)?;
    staged.flush().map_err(HostError::from)?;
    staged.as_file().sync_all().map_err(HostError::from)?;
    staged
        .persist(pointer(state))
        .map_err(|error| HostError::from(error.error))?;
    fs::File::open(state.fixtures())
        .and_then(|file| file.sync_all())
        .map_err(HostError::from)?;
    Ok(result)
}

fn capture_binary(
    root: &Path,
    environment: &Environment,
    destination: &Path,
    binary: &str,
    source_path: &str,
    feature: &str,
) -> Result<(Executable, generation::SourceFingerprint), FixtureError> {
    let features = vec![feature.into(), "force-validate".into()];
    let (capture, before) = execution::build_capture_with(
        root,
        environment,
        destination,
        "xtask",
        &features,
        &[binary],
        |resolution| source(root, environment, &features, resolution),
    )?;
    let executable = capture.require_binary(
        root,
        "xtask",
        binary,
        source_path,
        &[feature, "force-validate"],
    )?;
    before.require_unchanged(&source(root, environment, &features, &capture.resolution)?)?;
    Ok((executable, before))
}
fn source(
    root: &Path,
    environment: &Environment,
    features: &[String],
    resolution: &correctness::Resolution,
) -> Result<generation::SourceFingerprint, FixtureError> {
    let cargo = correctness::real_cargo_in(environment)?;
    let context = current_context(root, environment, features)?;
    let features = features
        .iter()
        .map(|feature| format!("xtask/{feature}"))
        .collect::<BTreeSet<_>>();
    let args = [
        "metadata".into(),
        "--locked".into(),
        "--offline".into(),
        "--format-version=1".into(),
        "--features".into(),
        features.into_iter().collect::<Vec<_>>().join(",").into(),
    ];
    let metadata: Metadata =
        serde_json::from_slice(&correctness::capture_at(&cargo, &args, root, environment)?)?;
    Ok(generation::SourceFingerprint::from_resolution(
        root, &metadata, context, resolution,
    )?)
}
fn construction_environment(
    root: &Path,
    environment: &Environment,
) -> Result<Environment, FixtureError> {
    let environment = correctness::standalone_adapter_environment(environment)?;
    // Select and authenticate only a completed existing native generation. This
    // verifies its original flags/tool digests and never prepares a provider.
    let environment = crate::native::select(root, &environment)?;
    Ok(construction_inputs(&correctness::compiler_environment(
        root,
        &environment,
    )?))
}
fn construction_inputs(environment: &Environment) -> Environment {
    // Use the same fixed inputs for construction and its context comparison;
    // invocation/session/selection variables cannot invalidate an immutable build
    // or affect these source-owned build scripts through ambient inheritance.
    environment
        .iter()
        .filter(|(name, _)| {
            name.to_str().is_some_and(|name| {
                matches!(
                    name,
                    "PATH"
                        | "HOME"
                        | "CARGO_HOME"
                        | "RUSTUP_HOME"
                        | "RUSTC"
                        | "RUSTDOC"
                        | "RUSTFLAGS"
                        | "RUSTDOCFLAGS"
                        | "CARGO_ENCODED_RUSTFLAGS"
                        | "CARGO_ENCODED_RUSTDOCFLAGS"
                        | "CARGO_INCREMENTAL"
                        | "SOURCE_DATE_EPOCH"
                        | "RUST_MIN_STACK"
                        | "CC"
                        | "CXX"
                        | "AR"
                        | "FC"
                        | "CMAKE"
                        | "CFLAGS"
                        | "CXXFLAGS"
                        | "CPPFLAGS"
                        | "FFLAGS"
                        | "LDFLAGS"
                        | "CMAKE_TOOLCHAIN_FILE"
                        | "SDKROOT"
                        | "MACOSX_DEPLOYMENT_TARGET"
                        | "PKG_CONFIG"
                        | "PKG_CONFIG_PATH"
                        | "PKG_CONFIG_LIBDIR"
                        | "PKG_CONFIG_SYSROOT_DIR"
                        | "LLVM_CONFIG_PATH"
                        | "LIBCLANG_PATH"
                        | "CLANG_PATH"
                        | "BINDGEN_EXTRA_CLANG_ARGS"
                        | "LD_LIBRARY_PATH"
                        | "LIBRARY_PATH"
                        | "XDG_CACHE_HOME"
                        | "TMPDIR"
                        | "SYMBOLICA_LICENSE"
                        | "NUMERICA_LICENSE"
                        | "PSE_CORRECTNESS_REAL_CARGO"
                        | "PSE_NATIVE_CACHE"
                        | "PSE_NATIVE_COMPILER_CACHE"
                        | "PSE_KLU_PROVIDER_RECEIPT"
                        | "SUITESPARSE_INCLUDE_DIR"
                        | "SUITESPARSE_LIBRARY_DIR"
                        | "CARGO_NET_OFFLINE"
                        | "SSL_CERT_FILE"
                        | "SSL_CERT_DIR"
                        | "HTTP_PROXY"
                        | "HTTPS_PROXY"
                        | "ALL_PROXY"
                        | "NO_PROXY"
                        | "http_proxy"
                        | "https_proxy"
                        | "all_proxy"
                        | "no_proxy"
                ) || [
                    "CARGO_PROFILE_",
                    "CARGO_TARGET_",
                    "CARGO_BUILD_",
                    "CC_",
                    "CXX_",
                    "AR_",
                    "FC_",
                    "CFLAGS_",
                    "CXXFLAGS_",
                    "CPPFLAGS_",
                    "FFLAGS_",
                    "LDFLAGS_",
                    "BINDGEN_EXTRA_CLANG_ARGS_",
                    "SCCACHE_",
                ]
                .iter()
                .any(|prefix| name.starts_with(prefix))
            })
        })
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}
fn context(
    rustc: String,
    cargo: String,
    environment: &Environment,
    features: &[String],
) -> generation::BuildContext {
    let environment = construction_inputs(environment);
    let inputs = environment
        .iter()
        .map(|(name, value)| (name.to_string_lossy().into_owned(), Some(value.clone())))
        .collect();
    generation::BuildContext::new(
        rustc,
        cargo,
        "dev".into(),
        features
            .iter()
            .map(|feature| format!("xtask/{feature}"))
            .collect(),
        &environment,
    )
    .with_additional_inputs(&inputs)
}
fn current_context(
    root: &Path,
    environment: &Environment,
    features: &[String],
) -> Result<generation::BuildContext, FixtureError> {
    let cargo = correctness::real_cargo_in(environment)?;
    let version = |program: &OsStr| -> Result<String, FixtureError> {
        let bytes = correctness::capture_at(program, &["-vV".into()], root, environment)?;
        String::from_utf8(bytes).map_err(|_| invalid("compiler version output is not UTF-8"))
    };
    let rustc = environment
        .get(OsStr::new("RUSTC"))
        .map_or(OsStr::new("rustc"), |value| value.as_os_str());
    Ok(context(
        version(rustc)?,
        version(&cargo)?,
        environment,
        features,
    ))
}

/// Passively reopen both protected inputs; this does not build or start science.
/// # Errors
/// Missing, foreign, incomplete, changed or unprotected original preparation refuses.
pub fn selected_runtime(
    root: &Path,
    environment: &Environment,
) -> Result<RuntimeInputs, FixtureError> {
    let state = CheckoutState::resolve(root, environment).map_err(HostError::from)?;
    if !pointer(&state).try_exists().map_err(HostError::from)? {
        return Err(FixtureError::Unprepared(Input::ScientificInputs));
    }
    let selected = open(
        &state,
        root,
        &BTreeSet::from([Input::ScientificInputs, Input::NativeLane]),
    )?;
    selected.view()
}
pub(super) fn selected_for_demand(
    root: &Path,
    environment: &Environment,
    demand: &BTreeSet<Input>,
) -> Result<Option<SelectedRuntime>, FixtureError> {
    let needed = [Input::ScientificInputs, Input::NativeLane]
        .into_iter()
        .filter(|input| {
            demand.contains(input) && !environment.contains_key(OsStr::new(variable(*input)))
        })
        .collect::<BTreeSet<_>>();
    if needed.is_empty() || !root.join(".git").try_exists().map_err(HostError::from)? {
        return Ok(None);
    }
    let state = CheckoutState::resolve(root, environment).map_err(HostError::from)?;
    if !pointer(&state).try_exists().map_err(HostError::from)? {
        return Ok(None);
    }
    let selected = open(&state, root, &needed)?;
    selected.validate_current(root, environment, &needed)?;
    Ok(Some(selected))
}
fn variable(input: Input) -> &'static str {
    match input {
        Input::ScientificInputs => "PSE_SCIENTIFIC_INPUTS",
        Input::NativeLane => "PSE_NATIVE_LANE_EXECUTABLE",
        _ => "",
    }
}
fn open(
    state: &CheckoutState,
    root: &Path,
    demand: &BTreeSet<Input>,
) -> Result<SelectedRuntime, FixtureError> {
    private_directory(&state.fixtures(), false)?;
    private_directory(&state.fixtures().join("runtime"), false)?;
    let selection: String = serde_json::from_slice(&read(&pointer(state), 1024, 0o600)?)?;
    if uuid::Uuid::parse_str(&selection).is_err() || selection.len() != 36 {
        return Err(invalid("invalid runtime fixture selection"));
    }
    let directory = state.fixtures().join("runtime").join(&selection);
    open_directory(state, root, &directory, &selection, demand)
}
fn open_directory(
    state: &CheckoutState,
    root: &Path,
    directory: &Path,
    selection: &str,
    demand: &BTreeSet<Input>,
) -> Result<SelectedRuntime, FixtureError> {
    private_directory(&directory, false)?;
    let record: Record =
        serde_json::from_slice(&read(&directory.join("receipt.json"), MAX_RECEIPT, 0o400)?)?;
    if record.version != 1
        || record.checkout != state.checkout_id.as_str()
        || record.selection != selection
        || record.sources.len() != 2
        || record.capsule.path != directory.join("capsule-output.json")
    {
        return Err(invalid(
            "runtime fixture receipt differs from original checkout/selection",
        ));
    }
    let builder = if demand.contains(&Input::ScientificInputs) {
        let bytes = read(&record.capsule.path, MAX_CAPSULE, 0o400)?;
        if bytes.len() as u64 != record.capsule.bytes || hash(&bytes) != record.capsule.sha256 {
            return Err(invalid("prepared scientific capsule changed"));
        }
        Some(
            BuildCapture::read(&directory.join("capsule-build/build.json"))?.require_binary(
                root,
                "xtask",
                "xtask",
                "xtask/src/main.rs",
                &["physical-admission", "force-validate"],
            )?,
        )
    } else {
        None
    };
    let lane = if demand.contains(&Input::NativeLane) {
        Some(
            BuildCapture::read(&directory.join("lane-build/build.json"))?.require_binary(
                root,
                "xtask",
                "pse-native-lane",
                "xtask/src/bin/pse-native-lane.rs",
                &["solver-kinsol", "force-validate"],
            )?,
        )
    } else {
        None
    };
    Ok(SelectedRuntime {
        directory: directory.to_owned(),
        record,
        lane,
        builder,
    })
}
impl SelectedRuntime {
    fn view(&self) -> Result<RuntimeInputs, FixtureError> {
        let lane = self
            .lane
            .as_ref()
            .ok_or_else(|| invalid("selected lane capture is absent"))?;
        Ok(RuntimeInputs {
            selection: self.record.selection.clone(),
            receipt: self.directory.join("receipt.json"),
            scientific_inputs: self.record.capsule.clone(),
            native_lane: RuntimeInputArtifact {
                path: lane.captured.clone(),
                sha256: lane.sha256.clone(),
                bytes: lane.bytes,
            },
            builds: vec![
                self.directory.join("capsule-build/build.json"),
                self.directory.join("lane-build/build.json"),
            ],
        })
    }
    fn validate_current(
        &self,
        root: &Path,
        environment: &Environment,
        demand: &BTreeSet<Input>,
    ) -> Result<(), FixtureError> {
        let environment = construction_environment(root, environment)?;
        for (index, input, feature) in [
            (0, Input::ScientificInputs, "physical-admission"),
            (1, Input::NativeLane, "solver-kinsol"),
        ] {
            if demand.contains(&input)
                && !self.record.sources[index].matches_sources(
                    root,
                    &current_context(
                        root,
                        &environment,
                        &[feature.into(), "force-validate".into()],
                    )?,
                )?
            {
                return Err(invalid(
                    "prepared runtime construction context or sources changed; explicitly prepare again",
                ));
            }
        }
        if demand.contains(&Input::ScientificInputs) {
            let builder = self
                .builder
                .as_ref()
                .ok_or_else(|| invalid("original capsule builder capture is absent"))?;
            builder.verify()?;
            let status = Command::new(&builder.captured)
                .args([
                    OsStr::new("scientific-inputs-check"),
                    OsStr::new(CATALOG),
                    self.record.capsule.path.as_os_str(),
                ])
                .current_dir(root)
                .env_clear()
                .envs(&environment)
                .status()
                .map_err(HostError::from)?;
            if !status.success() {
                return Err(invalid(format!(
                    "scientific input owner refused current capsule correspondence: {status}"
                )));
            }
        }
        Ok(())
    }
    pub(super) fn bind(
        &self,
        root: &Path,
        environment: &mut Environment,
        demand: &BTreeSet<Input>,
    ) -> Result<(), FixtureError> {
        let needed = [Input::ScientificInputs, Input::NativeLane]
            .into_iter()
            .filter(|input| {
                demand.contains(input) && !environment.contains_key(OsStr::new(variable(*input)))
            })
            .collect::<BTreeSet<_>>();
        if !needed.is_empty() {
            self.validate_current(root, environment, &needed)?;
        }
        self.bind_original(environment, demand)
    }
    fn bind_original(
        &self,
        environment: &mut Environment,
        demand: &BTreeSet<Input>,
    ) -> Result<(), FixtureError> {
        for input in [Input::ScientificInputs, Input::NativeLane] {
            if !demand.contains(&input) || environment.contains_key(OsStr::new(variable(input))) {
                continue;
            }
            let path = match input {
                Input::ScientificInputs => {
                    let bytes = read(&self.record.capsule.path, MAX_CAPSULE, 0o400)?;
                    if bytes.len() as u64 != self.record.capsule.bytes
                        || hash(&bytes) != self.record.capsule.sha256
                    {
                        return Err(invalid(
                            "prepared scientific capsule changed before lending",
                        ));
                    }
                    &self.record.capsule.path
                }
                Input::NativeLane => {
                    let lane = self
                        .lane
                        .as_ref()
                        .ok_or_else(|| invalid("required prepared lane is absent"))?;
                    lane.verify()?;
                    &lane.captured
                }
                _ => continue,
            };
            environment.insert(variable(input).into(), path.as_os_str().into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn environment(root: &Path) -> Environment {
        BTreeMap::from([
            ("XDG_STATE_HOME".into(), root.join("state").into_os_string()),
            ("HOME".into(), root.as_os_str().to_owned()),
        ])
    }
    #[test]
    fn runtime_preparation_failure_retains_original_pointer_and_unpublished_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let state =
            CheckoutState::from_paths(root, &root.join(".git"), &environment(root)).unwrap();
        private_directory(&state.fixtures(), true).unwrap();
        fs::write(pointer(&state), b"previous complete original selection").unwrap();
        let mut partial = PathBuf::new();
        let error = prepare_with(
            &state,
            |directory, _| {
                partial = directory.to_owned();
                fs::write(
                    directory.join("capsule-output.json"),
                    b"original capsule prefix",
                )
                .map_err(HostError::from)?;
                Err(invalid("original native lane capture refused"))
            },
            |_, _| Ok(()),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("original native lane capture refused")
        );
        assert_eq!(
            fs::read(pointer(&state)).unwrap(),
            b"previous complete original selection"
        );
        assert_eq!(
            fs::read(partial.join("capsule-output.json")).unwrap(),
            b"original capsule prefix"
        );
        assert!(!partial.join("receipt.json").exists());
    }
    #[test]
    fn runtime_selection_lends_only_demanded_original_inputs_and_preserves_overrides() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("capsule-output.json");
        let bytes = b"original owner capsule";
        fs::write(&path, bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let selected = SelectedRuntime {
            directory: temp.path().into(),
            record: Record {
                version: 1,
                checkout: "checked".into(),
                selection: "original".into(),
                capsule: RuntimeInputArtifact {
                    path: path.clone(),
                    sha256: hash(bytes),
                    bytes: bytes.len() as u64,
                },
                sources: Vec::new(),
            },
            lane: None,
            builder: None,
        };
        let mut environment = Environment::new();
        selected
            .bind(temp.path(), &mut environment, &BTreeSet::new())
            .unwrap();
        assert!(environment.is_empty());
        // Exercise projection after admission separately from current compiler/native
        // admission, which cannot be established by a hand-constructed observation.
        selected
            .bind_original(&mut environment, &BTreeSet::from([Input::ScientificInputs]))
            .unwrap();
        assert_eq!(
            environment[OsStr::new("PSE_SCIENTIFIC_INPUTS")],
            path.as_os_str()
        );
        assert!(!environment.contains_key(OsStr::new("PSE_NATIVE_LANE_EXECUTABLE")));
        for original in ["", "/explicit/invalid-capsule"] {
            environment.insert("PSE_SCIENTIFIC_INPUTS".into(), original.into());
            selected
                .bind(
                    temp.path(),
                    &mut environment,
                    &BTreeSet::from([Input::ScientificInputs]),
                )
                .unwrap();
            assert_eq!(environment[OsStr::new("PSE_SCIENTIFIC_INPUTS")], original);
        }
        environment.clear();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"changed bytes").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            selected
                .bind_original(&mut environment, &BTreeSet::from([Input::ScientificInputs]))
                .is_err()
        );
    }
    #[test]
    fn unused_or_explicit_runtime_inputs_do_not_inspect_or_prepare_a_checkout() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("not-a-checkout");
        assert!(
            selected_for_demand(&root, &Environment::new(), &BTreeSet::new())
                .unwrap()
                .is_none()
        );
        let environment = Environment::from([
            ("PSE_SCIENTIFIC_INPUTS".into(), "".into()),
            (
                "PSE_NATIVE_LANE_EXECUTABLE".into(),
                "/invalid/original".into(),
            ),
        ]);
        assert!(
            selected_for_demand(
                &root,
                &environment,
                &BTreeSet::from([Input::ScientificInputs, Input::NativeLane])
            )
            .unwrap()
            .is_none()
        );
        assert!(!root.exists());
    }

    fn fingerprint(
        root: &Path,
        context: generation::BuildContext,
    ) -> generation::SourceFingerprint {
        for file in [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            "owner/Cargo.toml",
            "owner/main.rs",
        ] {
            let path = root.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"original source").unwrap();
        }
        let id = "path+file:///fixture#xtask@0.1.0";
        let metadata: Metadata = serde_json::from_value(serde_json::json!({
            "packages":[{"name":"xtask","version":"0.1.0","id":id,"source":null,"manifest_path":root.join("owner/Cargo.toml"),"dependencies":[],"targets":[],"features":{},"metadata":{}}],
            "workspace_members":[id], "workspace_default_members":[id], "workspace_root":root,"target_directory":root.join("target"),"version":1,
            "resolve":{"root":id,"nodes":[{"id":id,"deps":[],"dependencies":[],"features":[]}]}
        })).unwrap();
        generation::SourceFingerprint::from_metadata_for(root, &metadata, context, "xtask").unwrap()
    }
    #[test]
    fn runtime_current_context_refuses_changed_compiler_native_flags_and_features() {
        let temp = tempfile::tempdir().unwrap();
        let original = Environment::from([
            ("CFLAGS".into(), "-O2".into()),
            (
                "PSE_KLU_PROVIDER_RECEIPT".into(),
                "/native/original/.complete.json".into(),
            ),
            ("PSE_MANAGED_INVOCATION".into(), "first invocation".into()),
        ]);
        let features = ["solver-kinsol".into(), "force-validate".into()];
        let saved = context(
            "rustc original".into(),
            "cargo original".into(),
            &original,
            &features,
        );
        let source = fingerprint(temp.path(), saved.clone());
        let mut next_invocation = original.clone();
        next_invocation.insert("PSE_MANAGED_INVOCATION".into(), "second invocation".into());
        next_invocation.insert("PSE_SESSION".into(), "another session".into());
        assert!(
            source
                .matches_sources(
                    temp.path(),
                    &context(
                        "rustc original".into(),
                        "cargo original".into(),
                        &next_invocation,
                        &features
                    )
                )
                .unwrap()
        );
        for (name, value) in [
            ("CFLAGS", "-O3"),
            ("PSE_KLU_PROVIDER_RECEIPT", "/native/changed/.complete.json"),
            ("RUSTFLAGS", "--cfg changed"),
            (
                "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
                "/changed/linker",
            ),
        ] {
            let mut changed = original.clone();
            changed.insert(name.into(), value.into());
            assert!(
                !source
                    .matches_sources(
                        temp.path(),
                        &context(
                            "rustc original".into(),
                            "cargo original".into(),
                            &changed,
                            &features
                        )
                    )
                    .unwrap(),
                "{name}"
            );
        }
        assert!(
            !source
                .matches_sources(
                    temp.path(),
                    &context(
                        "rustc changed".into(),
                        "cargo original".into(),
                        &original,
                        &features
                    )
                )
                .unwrap()
        );
        assert!(
            !source
                .matches_sources(
                    temp.path(),
                    &context(
                        "rustc original".into(),
                        "cargo changed".into(),
                        &original,
                        &features
                    )
                )
                .unwrap()
        );
        assert!(
            !source
                .matches_sources(
                    temp.path(),
                    &context(
                        "rustc original".into(),
                        "cargo original".into(),
                        &original,
                        &["physical-admission".into(), "force-validate".into()]
                    )
                )
                .unwrap()
        );
    }
    #[test]
    fn runtime_late_source_drift_refuses_candidate_before_replacing_previous_pointer() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let state =
            CheckoutState::from_paths(root, &root.join(".git"), &environment(root)).unwrap();
        let context = context(
            "rustc original".into(),
            "cargo original".into(),
            &environment(root),
            &["solver-kinsol".into()],
        );
        let source = fingerprint(root, context.clone());
        private_directory(&state.fixtures(), true).unwrap();
        fs::write(pointer(&state), b"previous protected selection").unwrap();
        let mut candidate = PathBuf::new();
        let result = prepare_with(
            &state,
            |directory, selection| {
                candidate = directory.to_owned();
                // A later preparation stage changes a dependency after original capture.
                fs::write(root.join("owner/main.rs"), b"late source drift")
                    .map_err(HostError::from)?;
                Ok(Record {
                    version: 1,
                    checkout: state.checkout_id.as_str().into(),
                    selection: selection.into(),
                    capsule: RuntimeInputArtifact {
                        path: directory.join("capsule-output.json"),
                        sha256: "original".into(),
                        bytes: 1,
                    },
                    sources: vec![source.clone(), source.clone()],
                })
            },
            |_, _| {
                if !source.matches_sources(root, &context)? {
                    return Err(invalid("late source drift refused"));
                }
                Ok(())
            },
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("late source drift refused")
        );
        assert_eq!(
            fs::read(pointer(&state)).unwrap(),
            b"previous protected selection"
        );
        assert!(
            candidate.join("receipt.json").is_file(),
            "retain complete unselected candidate evidence"
        );
    }
}
