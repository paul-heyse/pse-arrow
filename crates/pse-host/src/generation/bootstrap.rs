// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit construction and passive inspection of the fixed native tool set.
use super::*;
use std::{os::unix::fs::PermissionsExt, process::Command};

const TOOLS: &[&str] = &["pse", "pse-host", "cargo-pse-correctness"];
const EXTERNAL_INPUTS: &[&str] = &[
    "crates/pse-study-module/surrealism.toml",
    "crates/pse-study-module/surrealism-qualification-long-hold.toml",
];

/// Freshness is an observation and never starts Cargo or changes the selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    /// Prepared bytes and their recorded inputs agree with the checkout.
    Current,
    /// Recorded source/build inputs differ; only eligible new work may rebuild.
    Stale,
    /// No versioned prepared generation is available.
    Unprepared,
}

/// Authenticated prepared identity suitable for an invocation's control reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparedGeneration {
    /// Relative component below the fixed checkout tool root, never an executable path.
    pub generation: String,
    /// Existing-work control wire contract.
    pub protocol: u32,
    /// Semantic manifest identity binding the prepared executable bytes.
    pub manifest_digest: String,
}

/// Fixed executable names supported by a prepared host tool set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostTool {
    /// Agent-facing entry point.
    Pse,
    /// Native host/internal entry point.
    Host,
    /// Cargo's force-validation adapter.
    Correctness,
}
impl HostTool {
    fn name(self) -> &'static str {
        match self {
            Self::Pse => "pse",
            Self::Host => "pse-host",
            Self::Correctness => "cargo-pse-correctness",
        }
    }
}
impl PreparedGeneration {
    /// Authenticate a recorded component/protocol/manifest under the fixed checkout tool
    /// root, then select a fixed executable. Recorded arbitrary paths are never accepted.
    pub fn executable(&self, root: &Path, tool: HostTool) -> Result<PathBuf, HostError> {
        if self.protocol != 1
            || !component(&self.generation)
            || !self.generation.starts_with("generation.")
        {
            return Err(invalid("unsupported or invalid prepared control reference"));
        }
        let tools = root.join("target/host-tools");
        private_directory(&tools, false)?;
        let directory = tools.join(&self.generation);
        let manifest = GenerationManifest::load(&directory)?;
        if manifest.control_protocol() != self.protocol
            || manifest.digest()? != self.manifest_digest
        {
            return Err(invalid(
                "prepared control reference differs from authenticated generation",
            ));
        }
        let name = tool.name();
        if !manifest.binaries.contains_key(name) {
            return Err(invalid("prepared control tool is absent from its manifest"));
        }
        Ok(directory.join(name))
    }
}

/// Authenticate selected control identity without source reads or builds.
pub fn prepared(root: &Path) -> Result<Option<PreparedGeneration>, HostError> {
    let Some(directory) = selected(root)? else {
        return Ok(None);
    };
    let manifest = GenerationManifest::load(&directory)?;
    Ok(Some(PreparedGeneration {
        generation: directory
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| invalid("prepared generation is not UTF-8"))?
            .into(),
        protocol: manifest.control_protocol(),
        manifest_digest: manifest.digest()?,
    }))
}

fn build_environment(inputs: &Environment) -> Result<Environment, HostError> {
    for name in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_TARGET",
        "RUSTUP_TOOLCHAIN",
    ] {
        if inputs
            .get(OsStr::new(name))
            .is_some_and(|value| !value.is_empty())
        {
            return Err(invalid(format!(
                "remove {name}; host construction uses the pinned checkout-local target"
            )));
        }
    }
    // A fixed construction environment makes additional ambient variables incapable of
    // changing compiler/build-script output without entering the fingerprint.
    Ok(inputs
        .iter()
        .filter(|(name, _)| {
            let Some(name) = name.to_str() else {
                return false;
            };
            matches!(
                name,
                "PATH"
                    | "HOME"
                    | "CARGO_HOME"
                    | "RUSTUP_HOME"
                    | "CARGO_BUILD_JOBS"
                    | "RUST_MIN_STACK"
                    | "RUSTC"
                    | "RUSTFLAGS"
                    | "RUSTDOCFLAGS"
                    | "CARGO_ENCODED_RUSTFLAGS"
                    | "CARGO_ENCODED_RUSTDOCFLAGS"
                    | "CARGO_INCREMENTAL"
                    | "SOURCE_DATE_EPOCH"
                    | "CC"
                    | "CXX"
                    | "AR"
                    | "CFLAGS"
                    | "CXXFLAGS"
                    | "LDFLAGS"
                    | "PKG_CONFIG"
                    | "PKG_CONFIG_PATH"
                    | "PKG_CONFIG_LIBDIR"
                    | "PKG_CONFIG_SYSROOT_DIR"
                    | "LLVM_CONFIG_PATH"
                    | "LIBCLANG_PATH"
                    | "SYMBOLICA_LICENSE"
                    | "HTTP_PROXY"
                    | "HTTPS_PROXY"
                    | "ALL_PROXY"
                    | "NO_PROXY"
                    | "http_proxy"
                    | "https_proxy"
                    | "all_proxy"
                    | "no_proxy"
                    | "SSL_CERT_FILE"
                    | "SSL_CERT_DIR"
                    | "CARGO_NET_OFFLINE"
            ) || name.starts_with("CARGO_PROFILE_")
                || name.starts_with("CARGO_TARGET_")
                || name.starts_with("CC_")
                || name.starts_with("CXX_")
                || name.starts_with("AR_")
                || name.starts_with("CFLAGS_")
                || name.starts_with("CXXFLAGS_")
                || name.starts_with("SCCACHE_")
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

fn context(rustc: String, cargo: String, environment: &Environment) -> BuildContext {
    let inputs = environment
        .iter()
        .map(|(name, value)| (name.to_string_lossy().into_owned(), Some(value.clone())))
        .collect();
    BuildContext::new(rustc, cargo, "dev".into(), BTreeSet::new(), environment)
        .with_additional_inputs(&inputs)
}

fn observed_context(
    root: &Path,
    rustc: String,
    cargo: String,
    environment: &Environment,
) -> Result<BuildContext, HostError> {
    fn file(
        root: &Path,
        path: &Path,
        seen: &mut BTreeSet<PathBuf>,
        context: &mut BuildContext,
    ) -> Result<(), HostError> {
        if !seen.insert(path.to_owned()) {
            return Ok(());
        }
        if seen.len() > 256 {
            return Err(invalid(
                "Cargo configuration include closure exceeds its bound",
            ));
        }
        let key = match path.strip_prefix(root) {
            Ok(relative) => format!(
                "cargo_checkout_configuration:{}",
                digest(relative.as_os_str().as_bytes())
            ),
            Err(_) => format!(
                "cargo_external_configuration:{}",
                digest(path.as_os_str().as_bytes())
            ),
        };
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                context.environment.insert(key, None);
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        context.environment.insert(key, Some(digest(&bytes)));
        // Invalid configuration still gets a content identity. Cargo owns its diagnostic;
        // a passive observation can report stale before explicit construction rejects it.
        let Some(value) = std::str::from_utf8(&bytes)
            .ok()
            .and_then(|value| toml::from_str::<toml::Value>(value).ok())
        else {
            return Ok(());
        };
        if let Some(includes) = value.get("include").and_then(toml::Value::as_array) {
            for include in includes {
                let include_path = include
                    .as_str()
                    .or_else(|| include.get("path").and_then(toml::Value::as_str));
                if let Some(include_path) = include_path {
                    let parent = path
                        .parent()
                        .ok_or_else(|| invalid("Cargo configuration has no parent"))?;
                    file(root, &parent.join(include_path), seen, context)?;
                }
            }
        }
        Ok(())
    }
    fn configs(
        root: &Path,
        directory: &Path,
        seen: &mut BTreeSet<PathBuf>,
        context: &mut BuildContext,
    ) -> Result<(), HostError> {
        let legacy = directory.join("config");
        let modern = directory.join("config.toml");
        let selected = if legacy.exists() { legacy } else { modern };
        // Missing ancestor configs do not make otherwise equivalent worktrees differ.
        if selected.exists() {
            file(root, &selected, seen, context)?;
        }
        Ok(())
    }
    let mut result = context(rustc, cargo, environment);
    let mut seen = BTreeSet::new();
    for ancestor in root.ancestors() {
        configs(root, &ancestor.join(".cargo"), &mut seen, &mut result)?;
    }
    let home = environment
        .get(OsStr::new("CARGO_HOME"))
        .map(PathBuf::from)
        .or_else(|| {
            environment
                .get(OsStr::new("HOME"))
                .map(|home| PathBuf::from(home).join(".cargo"))
        });
    if let Some(home) = home {
        configs(
            root,
            &if home.is_absolute() {
                home
            } else {
                root.join(home)
            },
            &mut seen,
            &mut result,
        )?;
    }
    Ok(result)
}

fn version(root: &Path, environment: &Environment, name: &str) -> Result<String, HostError> {
    let output = Command::new(name)
        .arg("-vV")
        .current_dir(root)
        .env_clear()
        .envs(environment)
        .output()?;
    if !output.status.success() {
        return Err(invalid(format!("{name} version query failed")));
    }
    String::from_utf8(output.stdout).map_err(|_| invalid("tool version output is not UTF-8"))
}

fn capture(
    root: &Path,
    environment: &Environment,
    context: BuildContext,
) -> Result<SourceFingerprint, HostError> {
    SourceFingerprint::discover(root, environment, context)?
        .with_additional_files(root, EXTERNAL_INPUTS.iter().map(PathBuf::from))
}

fn prepare_root(path: &Path) -> Result<(), HostError> {
    if !path.exists() {
        fs::DirBuilder::new().mode(0o700).create(path)?;
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
    {
        return Err(invalid("prepared tool root is not an owned directory"));
    }
    // Explicit bootstrap adopts the previous seed's owned, umask-derived directory.
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    let lock = path.join(".update.lock");
    if let Ok(metadata) = fs::symlink_metadata(&lock) {
        if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() {
            return Err(invalid("existing update lock is not an owned regular file"));
        }
        fs::set_permissions(lock, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn selected(root: &Path) -> Result<Option<PathBuf>, HostError> {
    let tools = root.join("target/host-tools");
    match fs::symlink_metadata(&tools) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(_) => private_directory(&tools, false)?,
    }
    let link = match fs::read_link(tools.join("current")) {
        Ok(link) => link,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let Some(name) = link
        .to_str()
        .filter(|name| component(name) && name.starts_with("generation."))
    else {
        return Err(invalid(
            "current selector is not a fixed relative generation",
        ));
    };
    Ok(Some(tools.join(name)))
}

/// Inspect the saved closure and permitted build inputs without invoking a compiler,
/// Cargo, a hook or construction. Invalid prepared evidence remains an explicit error.
pub fn freshness(root: &Path, inputs: &Environment) -> Result<Freshness, HostError> {
    let Some(directory) = selected(root)? else {
        return Ok(Freshness::Unprepared);
    };
    if !directory.join(MANIFEST).exists() {
        return Ok(Freshness::Unprepared);
    }
    let manifest = GenerationManifest::load(&directory)?;
    let environment = build_environment(inputs)?;
    let saved = &manifest.source.context;
    let current = observed_context(root, saved.rustc.clone(), saved.cargo.clone(), &environment)?;
    Ok(if manifest.source.matches_sources(root, &current)? {
        Freshness::Current
    } else {
        Freshness::Stale
    })
}

fn publish(
    root: &Path,
    artifacts: &Path,
    expected: Option<&BTreeMap<String, String>>,
    before: SourceFingerprint,
    after: &SourceFingerprint,
    final_capture: impl FnOnce() -> Result<SourceFingerprint, HostError>,
) -> Result<(), HostError> {
    before.require_unchanged(after)?;
    let tools = root.join("target/host-tools");
    let stage = tempfile::Builder::new()
        .prefix("generation.")
        .tempdir_in(&tools)?;
    fs::set_permissions(stage.path(), fs::Permissions::from_mode(0o700))?;
    for tool in TOOLS {
        let source = artifacts.join(tool);
        if !fs::symlink_metadata(&source)?.is_file() {
            return Err(invalid("built tool is not a regular file"));
        }
        fs::copy(source, stage.path().join(tool))?;
        fs::set_permissions(stage.path().join(tool), fs::Permissions::from_mode(0o500))?;
        File::open(stage.path().join(tool))?.sync_all()?;
        if let Some(expected) = expected
            && expected.get(*tool) != Some(&hash_file(&stage.path().join(tool))?)
        {
            return Err(invalid(
                "copied binary differs from the pinned source manifest",
            ));
        }
        let result = Command::new(stage.path().join(tool))
            .arg("--version")
            .status()?;
        if !result.success() {
            return Err(invalid(format!("completed {tool} probe failed")));
        }
    }
    let manifest = GenerationManifest::capture(stage.path(), before, after, TOOLS)?;
    if expected.is_some_and(|expected| expected != &manifest.binaries) {
        return Err(invalid("copied tool set changed before publication"));
    }
    manifest.write(stage.path())?;
    after.require_unchanged(&final_capture()?)?;
    File::open(stage.path())?.sync_all()?;
    // Once selector publication can succeed, no cleanup path may delete the generation.
    let directory = stage.keep();
    let name = directory
        .file_name()
        .ok_or_else(|| invalid("generation has no name"))?;
    let selector = tools.join(format!(".selection-{}", uuid::Uuid::new_v4()));
    std::os::unix::fs::symlink(name, &selector)?;
    fs::rename(&selector, tools.join("current"))?;
    File::open(&tools)?.sync_all()?;
    Ok(())
}

/// Private seed result: this destination requires construction by its fresh local helper.
pub const LOCAL_SEED_REQUIRED: i32 = 79;
/// Whether this process owns the destination's current construction policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapMode {
    /// A freshly compiled local seed can build on a copy mismatch.
    CopyOrBuild,
    /// A source helper can copy a match; it must not build different destination code.
    CopyOnly,
}

/// Interpret a cold seed's expected-source constraint against actual entry ownership.
/// The environment value never grants prepared identity or construction authority.
pub fn bootstrap_mode(entry: &EntryGuard, from: Option<&Path>) -> Result<BootstrapMode, HostError> {
    source_mode(
        entry,
        from,
        std::env::var_os("PSE_BOOTSTRAP_SOURCE_HELPER").as_deref(),
    )
}
fn source_mode(
    entry: &EntryGuard,
    from: Option<&Path>,
    expected: Option<&OsStr>,
) -> Result<BootstrapMode, HostError> {
    let Some(expected) = expected else {
        return Ok(if entry.prepared().is_some() {
            BootstrapMode::CopyOnly
        } else {
            BootstrapMode::CopyOrBuild
        });
    };
    let expected = PathBuf::from(expected).canonicalize()?;
    let selected = from
        .ok_or_else(|| invalid("source helper requires its explicitly selected source checkout"))?
        .canonicalize()?;
    if expected != selected || entry.checkout().as_ref() != Some(&expected) {
        return Err(invalid(
            "source helper does not own the expected authenticated prepared checkout",
        ));
    }
    Ok(BootstrapMode::CopyOnly)
}

/// Performing-owner preparation result, separate from whether the command succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapOutcome {
    /// Original construction status; nonzero never publishes a failed build's selector.
    pub exit_code: i32,
    /// A copy's read/probe/file obligations were closed; a build's descendant scope remains
    /// unresolved until its original physical owner supplies actual settlement evidence.
    pub settlement: EntrySettlement,
}

/// Explicit preparation with an optional matching immutable source checkout. Only a
/// source/build identity match copies; a mismatch takes the ordinary construction path.
pub fn bootstrap_from(
    root: &Path,
    from: Option<&Path>,
    mode: BootstrapMode,
) -> Result<BootstrapOutcome, HostError> {
    construct(
        root,
        &build_environment(&std::env::vars_os().collect())?,
        from,
        mode,
    )
}

struct CopySource {
    directory: PathBuf,
    usage: GenerationUse,
}
impl CopySource {
    fn acquire(root: &Path) -> Result<Self, HostError> {
        let root = root.canonicalize()?;
        let reference = prepared(&root)?
            .ok_or_else(|| invalid("source checkout has no prepared generation"))?;
        let usage = GenerationUse::acquire(&root.join("target/host-tools"), &reference.generation)?;
        let verified = usage.manifest().digest().and_then(|digest| {
            if digest == reference.manifest_digest
                && usage.manifest().control_protocol() == reference.protocol
            {
                Ok(())
            } else {
                Err(invalid(
                    "source generation identity changed during acquisition",
                ))
            }
        });
        if let Err(operation) = verified {
            return Err(match usage.settle() {
                Ok(()) => operation,
                Err(cleanup) => HostError::Settlement {
                    operation: Box::new(operation),
                    cleanup: Box::new(cleanup),
                },
            });
        }
        Ok(Self {
            directory: root.join("target/host-tools").join(reference.generation),
            usage,
        })
    }
}

fn construct(
    root: &Path,
    environment: &Environment,
    from: Option<&Path>,
    mode: BootstrapMode,
) -> Result<BootstrapOutcome, HostError> {
    // Acquire source use before destination exclusion. A<--B and B<--A concurrent
    // preparations therefore never hold opposite exclusive update locks.
    let source = match from.map(CopySource::acquire).transpose() {
        Ok(source) => source,
        Err(error) => {
            if mode == BootstrapMode::CopyOnly {
                return Err(error);
            }
            writeln!(
                std::io::stderr(),
                "bootstrap-host: source generation unavailable or incompatible ({error}); building locally"
            )?;
            None
        }
    };
    let operation = construct_selected(root, environment, source.as_ref(), mode);
    // This owner only retained source file paths; every copy descriptor is now closed.
    // A probe, if any, executed the destination's private copy, never the source path.
    let released = source.map_or(Ok(()), |source| source.usage.settle());
    match (operation, released) {
        (result, Ok(())) => result,
        (Err(operation), Err(cleanup)) => Err(HostError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        }),
        (Ok(outcome), Err(cleanup)) => Err(HostError::Settlement {
            operation: Box::new(invalid(format!(
                "construction returned {}",
                outcome.exit_code
            ))),
            cleanup: Box::new(cleanup),
        }),
    }
}

fn construct_selected(
    root: &Path,
    environment: &Environment,
    source: Option<&CopySource>,
    mode: BootstrapMode,
) -> Result<BootstrapOutcome, HostError> {
    let root = root.canonicalize()?;
    let target = root.join("target");
    match fs::DirBuilder::new().mode(0o700).create(&target) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    let metadata = fs::symlink_metadata(&target)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
    {
        return Err(invalid("checkout target is not an owned direct directory"));
    }
    let tools = root.join("target/host-tools");
    prepare_root(&tools)?;
    // Serialize mutable build outputs separately. Passive entry protection takes the
    // publication lock and must remain usable throughout a cold host compilation.
    let lock = owned_file(&tools.join(".construction.lock"), true)?;
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive)
        .map_err(std::io::Error::from)?;
    let observe = || {
        capture(
            &root,
            environment,
            observed_context(
                &root,
                version(&root, environment, "rustc")?,
                version(&root, environment, "cargo")?,
                environment,
            )?,
        )
    };
    let before = observe()?;
    if let Some(source) = source.filter(|source| source.usage.manifest().source() == &before) {
        let after = observe()?;
        let _publication = publication_lock(&tools)?;
        publish(
            &root,
            &source.directory,
            Some(&source.usage.manifest().binaries),
            before,
            &after,
            observe,
        )?;
        return Ok(BootstrapOutcome {
            exit_code: 0,
            settlement: EntrySettlement::ResourcesSettled,
        });
    }
    if mode == BootstrapMode::CopyOnly {
        writeln!(
            std::io::stderr(),
            "bootstrap-host: source generation does not match destination inputs; a fresh local seed is required"
        )?;
        return Ok(BootstrapOutcome {
            exit_code: LOCAL_SEED_REQUIRED,
            settlement: EntrySettlement::ResourcesSettled,
        });
    }
    if source.is_some() {
        writeln!(
            std::io::stderr(),
            "bootstrap-host: source generation does not match destination source/build inputs; building locally"
        )?;
    }
    let result = Command::new("cargo")
        .args(["build", "--locked", "--package", "pse-host", "--bins"])
        .current_dir(&root)
        .env_clear()
        .envs(environment)
        .status()?;
    if !result.success() {
        return Ok(BootstrapOutcome {
            exit_code: result.code().unwrap_or(125),
            settlement: EntrySettlement::Unresolved,
        });
    }
    let after = observe()?;
    let _publication = publication_lock(&tools)?;
    publish(
        &root,
        &root.join("target/debug"),
        None,
        before,
        &after,
        observe,
    )?;
    Ok(BootstrapOutcome {
        exit_code: 0,
        settlement: EntrySettlement::Unresolved,
    })
}

fn publication_lock(tools: &Path) -> Result<File, HostError> {
    let lock = owned_file(&tools.join(".update.lock"), true)?;
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive)
        .map_err(std::io::Error::from)?;
    Ok(lock)
}

#[cfg(test)]
mod tests;
