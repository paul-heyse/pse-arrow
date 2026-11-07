// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit reusable capture of the standalone native-lane executable.
use super::*;
use crate::{
    correctness, discovery::BuildCapture, environment::Environment, generation,
    paths::CheckoutState,
};
use std::{
    ffi::OsStr,
    fs,
    io::{Read as _, Write as _},
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
};

const MAX_RECEIPT: usize = 64 << 20;
const BINARY: &str = "pse-native-lane";
const SOURCE: &str = "xtask/src/bin/pse-native-lane.rs";
const FEATURES: &[&str] = &["solver-kinsol", "force-validate"];
pub(super) const VARIABLE: &str = "PSE_NATIVE_LANE_EXECUTABLE";

/// One retained original executable; byte identity does not establish scientific readiness.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NativeLaneArtifact {
    /// Retained original bytes, never a guessed Cargo output.
    pub path: PathBuf,
    /// SHA-256 of those exact bytes.
    pub sha256: String,
    /// Original byte extent.
    pub bytes: u64,
}

/// Explicitly selected private lane without a live fixture claim.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NativeLaneSelection {
    /// Opaque retained preparation identity.
    pub selection: String,
    /// Original protected selection record.
    pub receipt: PathBuf,
    /// Source-built private native-lane executable.
    pub native_lane: NativeLaneArtifact,
    /// Exact original build evidence.
    pub build: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    checkout: String,
    selection: String,
    source: generation::SourceFingerprint,
}

#[derive(Debug)]
pub(super) struct SelectedNativeLane {
    directory: PathBuf,
    record: Record,
    executable: Executable,
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
            "native lane fixture requires a private owned directory",
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
            "native lane fixture requires bounded protected original bytes",
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
            "native lane fixture artifact changed during acquisition",
        ));
    }
    Ok(bytes)
}
fn pointer(state: &CheckoutState) -> PathBuf {
    state.fixtures().join("native-lane.json")
}

/// Reuse the current standalone lane or build and capture its source owner.
/// # Errors
/// Source, compiler, native admission, capture or publication refuses.
pub fn prepare_native_lane(
    root: &Path,
    environment: &Environment,
) -> Result<NativeLaneSelection, FixtureError> {
    let root = root.canonicalize().map_err(HostError::from)?;
    let state = CheckoutState::resolve(&root, environment).map_err(HostError::from)?;
    let environment = construction_environment(&root, environment)?;
    private_directory(&state.fixtures(), true)?;
    let lock = generation::owned_file(&state.fixtures().join(".native-lane-prepare.lock"), true)?;
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive)
        .map_err(std::io::Error::from)
        .map_err(HostError::from)?;
    if pointer(&state).try_exists().map_err(HostError::from)?
        && let Ok(selected) = open(&state, &root)
        && selected.validate_current(&root, &environment).is_ok()
    {
        return Ok(selected.view());
    }
    prepare_with(
        &state,
        |directory, selection| {
            let source = capture_lane(&root, &environment, &directory.join("build"))?;
            Ok(Record {
                version: 1,
                checkout: state.checkout_id.as_str().into(),
                selection: selection.into(),
                source,
            })
        },
        |directory, selection| {
            let selected = open_directory(&state, &root, directory, selection)?;
            selected.validate_current(&root, &environment)?;
            Ok(selected.view())
        },
    )
}

fn prepare_with<T>(
    state: &CheckoutState,
    work: impl FnOnce(&Path, &str) -> Result<Record, FixtureError>,
    validate: impl FnOnce(&Path, &str) -> Result<T, FixtureError>,
) -> Result<T, FixtureError> {
    private_directory(&state.fixtures(), true)?;
    let parent = state.fixtures().join("native-lane");
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
    // Reopen the complete candidate and establish current construction
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

fn capture_lane(
    root: &Path,
    environment: &Environment,
    destination: &Path,
) -> Result<generation::SourceFingerprint, FixtureError> {
    let features = FEATURES.iter().map(|feature| (*feature).into()).collect::<Vec<_>>();
    let (capture, before) = execution::build_capture_with(
        root,
        environment,
        destination,
        "xtask",
        &features,
        &[BINARY],
        |resolution| source(root, environment, &features, resolution),
    )?;
    capture.require_binary(root, "xtask", BINARY, SOURCE, FEATURES)?;
    before.require_unchanged(&source(root, environment, &features, &capture.resolution)?)?;
    Ok(before)
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
                        | "RUSTC_WRAPPER"
                        | "RUSTC_WORKSPACE_WRAPPER"
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

/// Passively reopen the protected standalone lane; this does not build or start science.
/// # Errors
/// Missing, foreign, incomplete, changed or unprotected original preparation refuses.
pub fn selected_native_lane(
    root: &Path,
    environment: &Environment,
) -> Result<NativeLaneSelection, FixtureError> {
    let state = CheckoutState::resolve(root, environment).map_err(HostError::from)?;
    if !pointer(&state).try_exists().map_err(HostError::from)? {
        return Err(FixtureError::Unprepared(Input::NativeLane));
    }
    Ok(open(&state, root)?.view())
}

pub(super) fn selected_for_run(
    root: &Path,
    environment: &Environment,
) -> Result<Option<SelectedNativeLane>, FixtureError> {
    if environment.contains_key(OsStr::new(VARIABLE))
        || !root.join(".git").try_exists().map_err(HostError::from)?
    {
        return Ok(None);
    }
    let state = CheckoutState::resolve(root, environment).map_err(HostError::from)?;
    if !pointer(&state).try_exists().map_err(HostError::from)? {
        return Ok(None);
    }
    let selected = open(&state, root)?;
    selected.validate_current(root, environment)?;
    Ok(Some(selected))
}

fn open(state: &CheckoutState, root: &Path) -> Result<SelectedNativeLane, FixtureError> {
    private_directory(&state.fixtures(), false)?;
    private_directory(&state.fixtures().join("native-lane"), false)?;
    let selection: String = serde_json::from_slice(&read(&pointer(state), 1024, 0o600)?)?;
    if uuid::Uuid::parse_str(&selection).is_err() || selection.len() != 36 {
        return Err(invalid("invalid native lane fixture selection"));
    }
    let directory = state.fixtures().join("native-lane").join(&selection);
    open_directory(state, root, &directory, &selection)
}

fn open_directory(
    state: &CheckoutState,
    root: &Path,
    directory: &Path,
    selection: &str,
) -> Result<SelectedNativeLane, FixtureError> {
    private_directory(directory, false)?;
    let record: Record =
        serde_json::from_slice(&read(&directory.join("receipt.json"), MAX_RECEIPT, 0o400)?)?;
    if record.version != 1
        || record.checkout != state.checkout_id.as_str()
        || record.selection != selection
    {
        return Err(invalid(
            "native lane fixture receipt differs from original checkout/selection",
        ));
    }
    let executable = BuildCapture::read(&directory.join("build/build.json"))?
        .require_binary(root, "xtask", BINARY, SOURCE, FEATURES)?;
    Ok(SelectedNativeLane {
        directory: directory.to_owned(),
        record,
        executable,
    })
}

impl SelectedNativeLane {
    fn view(&self) -> NativeLaneSelection {
        NativeLaneSelection {
            selection: self.record.selection.clone(),
            receipt: self.directory.join("receipt.json"),
            native_lane: NativeLaneArtifact {
                path: self.executable.captured.clone(),
                sha256: self.executable.sha256.clone(),
                bytes: self.executable.bytes,
            },
            build: self.directory.join("build/build.json"),
        }
    }

    fn validate_current(
        &self,
        root: &Path,
        environment: &Environment,
    ) -> Result<(), FixtureError> {
        let environment = construction_environment(root, environment)?;
        let features = FEATURES.iter().map(|feature| (*feature).into()).collect::<Vec<_>>();
        self.record.source.require_current_sources(
            root,
            &current_context(root, &environment, &features)?,
        )?;
        Ok(())
    }

    pub(super) fn lend(
        &self,
        root: &Path,
        environment: &mut Environment,
    ) -> Result<(), FixtureError> {
        if !environment.contains_key(OsStr::new(VARIABLE)) {
            self.validate_current(root, environment)?;
            self.executable.verify()?;
            environment.insert(VARIABLE.into(), self.executable.captured.as_os_str().into());
        }
        Ok(())
    }

    pub(super) fn path(&self) -> &Path {
        &self.executable.captured
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
    fn native_lane_preparation_failure_retains_original_pointer_and_unpublished_prefix() {
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
                    directory.join("partial-build-output"),
                    b"original build prefix",
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
            fs::read(partial.join("partial-build-output")).unwrap(),
            b"original build prefix"
        );
        assert!(!partial.join("receipt.json").exists());
    }
    #[test]
    fn explicit_native_lane_path_does_not_inspect_or_prepare_a_checkout() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("not-a-checkout");
        assert!(selected_for_run(&root, &Environment::new()).unwrap().is_none());
        for original in ["", "/invalid/original"] {
            let environment = Environment::from([(VARIABLE.into(), original.into())]);
            assert!(selected_for_run(&root, &environment).unwrap().is_none());
        }
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
    fn native_lane_construction_retains_original_cache_selection_across_reentry() {
        let original = Environment::from([
            ("RUSTC_WRAPPER".into(), "/tools/sccache".into()),
            ("PSE_NATIVE_COMPILER_CACHE".into(), "/tools/sccache".into()),
            (
                "CMAKE_TOOLCHAIN_FILE".into(),
                "/checkout/native-cache.cmake".into(),
            ),
            (
                "RUSTC_WORKSPACE_WRAPPER".into(),
                "/tools/workspace-wrapper".into(),
            ),
            ("PSE_MANAGED_INVOCATION".into(), "one invocation".into()),
        ]);
        let selected = construction_inputs(&original);
        for name in [
            "RUSTC_WRAPPER",
            "RUSTC_WORKSPACE_WRAPPER",
            "PSE_NATIVE_COMPILER_CACHE",
            "CMAKE_TOOLCHAIN_FILE",
        ] {
            assert_eq!(
                selected.get(OsStr::new(name)),
                original.get(OsStr::new(name)),
                "{name}"
            );
        }
        assert!(!selected.contains_key(OsStr::new("PSE_MANAGED_INVOCATION")));
        assert_eq!(construction_inputs(&selected), selected);

        let mut off = original;
        off.insert("RUSTC_WRAPPER".into(), "".into());
        assert_eq!(
            construction_inputs(&off)[OsStr::new("RUSTC_WRAPPER")],
            OsStr::new("")
        );
    }
    #[test]
    fn native_lane_current_context_refuses_changed_compiler_native_flags_and_features() {
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
            ("RUSTC_WRAPPER", "/changed/cache-wrapper"),
            ("RUSTC_WORKSPACE_WRAPPER", "/changed/workspace-wrapper"),
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
    fn native_lane_late_source_drift_refuses_candidate_before_replacing_previous_pointer() {
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
                    source: source.clone(),
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
