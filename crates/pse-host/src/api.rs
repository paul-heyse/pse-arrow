// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit, source/context-qualified rustdoc acquisition. Documentation is not build applicability.
use crate::{
    catalog::MetadataSnapshot,
    environment::Environment,
    generation::{BuildContext, SourceFingerprint},
    probe,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::Write as _,
    os::unix::{
        ffi::OsStrExt as _,
        fs::{DirBuilderExt as _, OpenOptionsExt as _},
    },
    path::{Path, PathBuf},
    process::Command,
};
pub mod bridge;
pub mod comparison;
pub mod query;

/// A version, acquisition or original native input failure.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Original file/process failure.
    #[error("API acquisition I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original JSON error after format preflight.
    #[error("API JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original source-qualified selection failure.
    #[error(transparent)]
    Probe(#[from] probe::ProbeError),
    /// Original compiler context observation failure.
    #[error(transparent)]
    Correctness(#[from] crate::correctness::CorrectnessError),
    /// Unavailable or inconsistent extraction premises.
    #[error("API acquisition: {0}")]
    Invalid(String),
    /// Refuse before typed decode; changing the pin alone is not an implicit migration.
    #[error(
        "rustdoc JSON format {actual} is unsupported (expected {expected}); select a matching pinned rustdoc-types/toolchain pair and reacquire"
    )]
    Format {
        /// Producer's declared rustdoc schema version.
        actual: u32,
        /// Decoder's supported rustdoc schema version.
        expected: u32,
    },
}
pse_diagnostics::impl_diagnostic! {
    ApiError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(this) { match this { Self::Probe(error) => Some(error), Self::Correctness(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::Invalid(message.into())
}

/// Check the format field before decoding version-specific rustdoc declarations.
pub fn decode(bytes: &[u8]) -> Result<rustdoc_types::Crate, ApiError> {
    #[derive(Deserialize)]
    struct Header {
        format_version: u32,
    }
    let header: Header = serde_json::from_slice(bytes)?;
    if header.format_version != rustdoc_types::FORMAT_VERSION {
        return Err(ApiError::Format {
            actual: header.format_version,
            expected: rustdoc_types::FORMAT_VERSION,
        });
    }
    let document: rustdoc_types::Crate = serde_json::from_slice(bytes)?;
    if !document.index.contains_key(&document.root) {
        return Err(invalid("rustdoc root is absent from its item index"));
    }
    Ok(document)
}

/// Exact acquisition context; numeric item IDs have meaning only within its artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    /// Checked local context protocol.
    pub version: u32,
    /// Cargo's source-qualified exact package ID.
    pub package_id: String,
    /// Exact locked source address, including local path identity.
    pub manifest: PathBuf,
    /// Selected source bytes, including non-Rust authored files.
    pub package_source_sha256: String,
    /// Originating metadata bytes, retaining transitive feature resolution.
    pub metadata_sha256: String,
    /// Metadata's enabled features; host/target compiler-unit applicability stays distinct.
    pub features: BTreeSet<String>,
    /// Broader package-metadata feature union, separate from selected documentation units.
    pub metadata_features: BTreeSet<String>,
    /// Exact native Cargo documentation unit-graph bytes; a plan, never built-artifact evidence.
    pub unit_graph_sha256: String,
    /// Source closure plus toolchain/build environment/configuration fingerprints.
    pub source: SourceFingerprint,
    /// Explicit target triple, including an explicitly observed host default.
    pub target: String,
    /// Rustdoc flags passed after Cargo's separator, including documentation-only conditions.
    pub extraction_flags: Vec<String>,
    /// Version checked before typed decode.
    pub rustdoc_format: u32,
    /// Actual selected rustdoc identity, independent of the rustc executable.
    pub rustdoc_identity: String,
    /// Complete captured environment identity; values remain private and never appear in output.
    pub environment_sha256: String,
}
impl Context {
    /// Source/feature/target/toolchain/cfg/schema changes cannot alias this key.
    pub fn key(&self) -> Result<String, ApiError> {
        Ok(probe::digest(&serde_json::to_vec(self)?))
    }
}
/// Prepared explicit effect, executed only through the caller's managed native owner.
#[derive(Debug)]
pub struct Acquisition {
    /// Complete observed acquisition context.
    pub context: Context,
    /// Context-keyed API artifact directory.
    pub directory: PathBuf,
    /// Independent Cargo output, never the checkout target.
    pub target_directory: PathBuf,
    /// Original native command, without a shell.
    pub command: Vec<OsString>,
    /// Environment with ambient shared-output overrides removed.
    pub environment: Environment,
    /// Original checkout selected for pinned-toolchain/configuration resolution.
    pub cwd: PathBuf,
    json_output: PathBuf,
    query: String,
}
fn observe(program: &OsStr, root: &Path, environment: &Environment) -> Result<String, ApiError> {
    let output = Command::new(program)
        .arg("-vV")
        .current_dir(root)
        .env_clear()
        .envs(environment)
        .output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "compiler identity observation exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(|_| invalid("compiler identity is not UTF-8"))
}
fn source_files(
    root: &Path,
    relative: &Path,
    files: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), ApiError> {
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() {
        return Err(invalid(
            "symlink API source requires explicit acquisition support",
        ));
    }
    if metadata.is_file() {
        files.push((relative.to_owned(), fs::read(&path)?));
    } else if metadata.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            if matches!(entry.file_name().to_str(), Some("target" | ".git")) {
                continue;
            }
            source_files(root, &relative.join(entry.file_name()), files)?;
        }
    }
    Ok(())
}
#[derive(Deserialize)]
struct DocumentationGraph {
    version: u32,
    units: Vec<DocumentationUnit>,
    roots: Vec<usize>,
}
#[derive(Deserialize)]
struct DocumentationUnit {
    pkg_id: String,
    features: BTreeSet<String>,
    mode: String,
    platform: Option<String>,
    target: DocumentationTarget,
}
#[derive(Deserialize)]
struct DocumentationTarget {
    name: String,
    src_path: PathBuf,
}
fn documentation_features(
    bytes: &[u8],
    package: &str,
    library: &str,
    target: &str,
    source: &Path,
) -> Result<BTreeSet<String>, ApiError> {
    let graph: DocumentationGraph = serde_json::from_slice(bytes)?;
    if graph.version != 1 {
        return Err(invalid(
            "unsupported Cargo documentation unit-graph version; select the pinned Cargo graph protocol",
        ));
    }
    let roots = graph
        .roots
        .iter()
        .map(|index| {
            graph
                .units
                .get(*index)
                .ok_or_else(|| invalid("documentation unit-graph root is out of bounds"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let selected: Vec<_> = roots
        .into_iter()
        .filter(|unit| {
            unit.pkg_id == package
                && unit.target.name == library
                && unit.target.src_path == source
                && unit.mode == "doc"
                && unit.platform.as_deref() == Some(target)
        })
        .collect();
    match selected.as_slice() {
        [unit] => Ok(unit.features.clone()),
        _ => Err(invalid(
            "Cargo did not plan exactly one documentation root for the addressed package/library/target",
        )),
    }
}
type SourceFiles = Vec<(PathBuf, Vec<u8>)>;
fn source_files_digest(files: &SourceFiles) -> String {
    use sha2::Digest as _;
    let mut hash = sha2::Sha256::new();
    for (path, bytes) in files {
        hash.update(path.as_os_str().as_bytes());
        hash.update([0]);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
struct Observation {
    context: Context,
    files: SourceFiles,
    library: String,
    unit_graph: Vec<u8>,
}
fn extraction_environment(environment: &Environment) -> Result<Environment, ApiError> {
    let mut selected = crate::correctness::standalone_adapter_environment(
        &crate::environment::preparation_inputs(environment),
    )?;
    selected.insert(
        "CARGO".into(),
        crate::correctness::real_cargo_in(&selected)?,
    );
    // These are transport/shell state, not extraction inputs. Remove them from the
    // actual compiler environment as well as its identity, so reuse cannot hide an input.
    for key in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_BUILD_DIR",
    ] {
        selected.remove(OsStr::new(key));
    }
    Ok(selected)
}
fn observed_context(
    root: &Path,
    store: &Path,
    snapshot: &MetadataSnapshot,
    query: &str,
    target: Option<&str>,
    environment: &Environment,
) -> Result<Observation, ApiError> {
    let package = probe::select(&snapshot.metadata, query)?;
    let library = package
        .targets
        .iter()
        .find(|target| {
            target.kind.iter().any(|kind| {
                matches!(
                    kind,
                    cargo_metadata::TargetKind::Lib | cargo_metadata::TargetKind::ProcMacro
                )
            })
        })
        .ok_or_else(|| invalid("selected package has no documented library target"))?;
    let metadata_features: BTreeSet<String> = snapshot
        .metadata
        .resolve
        .as_ref()
        .and_then(|resolve| resolve.nodes.iter().find(|node| node.id == package.id))
        .ok_or_else(|| invalid("resolved package features unavailable"))?
        .features
        .iter()
        .map(ToString::to_string)
        .collect();
    let cargo = crate::correctness::real_cargo_in(environment)?;
    let rustc = environment
        .get(OsStr::new("RUSTC"))
        .map_or(OsStr::new("rustc"), |value| value.as_os_str());
    let rustc_identity = observe(rustc, root, environment)?;
    let host = rustc_identity
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| invalid("rustc identity has no host triple"))?;
    let target = target.unwrap_or(host).to_owned();
    if target.is_empty() || target.contains('/') || target.contains('\\') {
        return Err(invalid("API target must be an explicit target triple"));
    }
    let arguments: Vec<OsString> = vec![
        "rustdoc".into(),
        "--locked".into(),
        "--offline".into(),
        "--lib".into(),
        "-p".into(),
        package.id.to_string().into(),
        "--target".into(),
        target.clone().into(),
        "--unit-graph".into(),
        "-Z".into(),
        "unstable-options".into(),
        "--target-dir".into(),
        store.join("api/.plan-target").into_os_string(),
    ];
    let unit_graph = crate::correctness::capture_at(&cargo, &arguments, root, environment)?;
    let features = documentation_features(
        &unit_graph,
        &package.id.to_string(),
        &library.name,
        &target,
        library.src_path.as_std_path(),
    )?;
    let build_context = BuildContext::new(
        rustc_identity,
        observe(&cargo, root, environment)?,
        "dev-rustdoc".into(),
        features.clone(),
        environment,
    )
    .with_additional_inputs(
        &environment
            .iter()
            .filter_map(|(key, value)| {
                key.to_str()
                    .filter(|key| {
                        key.starts_with("CARGO_TARGET_")
                            || key.starts_with("CARGO_BUILD_")
                            || *key == "RUSTDOC"
                            || *key == "RUSTUP_TOOLCHAIN"
                    })
                    .map(|key| (key.to_owned(), Some(value.clone())))
            })
            .collect(),
    );
    let mut metadata = snapshot.metadata.clone();
    metadata
        .packages
        .sort_by_key(|candidate| candidate.id != package.id);
    let source =
        SourceFingerprint::from_metadata_for(root, &metadata, build_context, package.name.as_str())
            .map_err(|error| invalid(error.to_string()))?;
    let package_root = package
        .manifest_path
        .parent()
        .ok_or_else(|| invalid("package manifest has no parent"))?
        .as_std_path();
    let mut files = Vec::new();
    source_files(package_root, Path::new(""), &mut files)?;
    let package_source_sha256 = source_files_digest(&files);
    let rustdoc = environment
        .get(OsStr::new("RUSTDOC"))
        .map_or(OsStr::new("rustdoc"), |value| value.as_os_str());
    let rustdoc_identity = observe(rustdoc, root, environment)?;
    let mut environment_bytes = Vec::new();
    for (key, value) in environment {
        environment_bytes.extend_from_slice(key.as_bytes());
        environment_bytes.push(0);
        environment_bytes.extend_from_slice(value.as_bytes());
        environment_bytes.push(0);
    }
    Ok(Observation {
        context: Context {
            version: 1,
            package_id: package.id.to_string(),
            manifest: package.manifest_path.as_std_path().to_owned(),
            package_source_sha256,
            metadata_sha256: snapshot.digest.clone(),
            metadata_features,
            features,
            unit_graph_sha256: probe::digest(&unit_graph),
            source,
            target,
            extraction_flags: vec![
                "-Z".into(),
                "unstable-options".into(),
                "--output-format".into(),
                "json".into(),
            ],
            rustdoc_format: rustdoc_types::FORMAT_VERSION,
            rustdoc_identity,
            environment_sha256: probe::digest(&environment_bytes),
        },
        files,
        library: library.name.clone(),
        unit_graph,
    })
}
/// Prepare one exact addressed crate's JSON generation. This does not execute Cargo/rustdoc.
/// Version observations are read-only; source bytes are retained before later compiler effects.
pub fn prepare(
    root: &Path,
    store: &Path,
    snapshot: &MetadataSnapshot,
    query: &str,
    target: Option<&str>,
    environment: &Environment,
) -> Result<Acquisition, ApiError> {
    let environment = extraction_environment(environment)?;
    let Observation {
        context,
        files,
        library,
        unit_graph,
    } = observed_context(root, store, snapshot, query, target, &environment)?;
    let directory = store
        .join("api")
        .join(context.key()?)
        .join(uuid::Uuid::now_v7().to_string());
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&directory)?;
    for (path, bytes) in files {
        let destination = directory.join("source").join(path);
        if let Some(parent) = destination.parent() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)?;
        }
        write(&destination, &bytes)?;
    }
    for (path, expected) in context.source.inputs() {
        let bytes = fs::read(root.join(path))?;
        if probe::digest(&bytes) != expected {
            return Err(invalid(
                "API source closure changed before its original bytes could be retained",
            ));
        }
        let destination = directory.join("source-checkout").join(path);
        if let Some(parent) = destination.parent() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)?;
        }
        write(&destination, &bytes)?;
    }
    write(
        &directory.join("context.json"),
        &serde_json::to_vec_pretty(&context)?,
    )?;
    write(&directory.join("unit-graph.json"), &unit_graph)?;
    let target_directory = directory.join("target");
    let cargo = crate::correctness::real_cargo_in(&environment)?;
    let mut command: Vec<OsString> = vec![
        cargo,
        "--config".into(),
        format!("build.build-dir={target_directory:?}").into(),
        "rustdoc".into(),
        "--locked".into(),
        "--offline".into(),
        "--lib".into(),
        "-p".into(),
        context.package_id.clone().into(),
        "--target".into(),
        context.target.clone().into(),
        "--target-dir".into(),
        target_directory.clone().into_os_string(),
        "--".into(),
    ];
    command.extend(context.extraction_flags.iter().map(OsString::from));
    let json_output = target_directory
        .join(&context.target)
        .join("doc")
        .join(format!("{}.json", library.replace('-', "_")));
    Ok(Acquisition {
        context,
        directory,
        target_directory,
        command,
        environment,
        cwd: root.to_owned(),
        json_output,
        query: query.into(),
    })
}
fn write(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
/// Publish only after the caller observed successful original native completion.
/// Re-observation rejects source/context changes that occurred during the extraction.
pub fn finish(
    acquisition: &Acquisition,
    snapshot: &MetadataSnapshot,
) -> Result<Artifact, ApiError> {
    let store = acquisition
        .directory
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| invalid("API acquisition has no addressed store"))?;
    let after = observed_context(
        &acquisition.cwd,
        store,
        snapshot,
        &acquisition.query,
        Some(&acquisition.context.target),
        &acquisition.environment,
    )?;
    if after.context != acquisition.context {
        return Err(invalid(
            "API source/build context changed during extraction; retain original diagnostics and reacquire",
        ));
    }
    let bytes = fs::read(&acquisition.json_output)?;
    let document = decode(&bytes)?;
    if document.target.triple != acquisition.context.target {
        return Err(invalid(
            "rustdoc output target differs from acquisition context",
        ));
    }
    let path = acquisition.directory.join("document.json");
    write(&path, &bytes)?;
    write(
        &acquisition.directory.join("artifact.json"),
        &serde_json::to_vec(&ArtifactRecord {
            context_key: acquisition.context.key()?,
            sha256: probe::digest(&bytes),
        })?,
    )?;
    Ok(Artifact {
        context: acquisition.context.clone(),
        path,
        sha256: probe::digest(&bytes),
        document,
    })
}
/// A successfully acquired, decoded declaration artifact. It grants no compilation verdict.
#[derive(Debug)]
pub struct Artifact {
    /// Acquisition source/context identity.
    pub context: Context,
    /// Immutable retained original rustdoc JSON.
    pub path: PathBuf,
    /// Original rustdoc bytes.
    pub sha256: String,
    /// Typed exact-format declarations for H consumers.
    pub document: rustdoc_types::Crate,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRecord {
    context_key: String,
    sha256: String,
}
/// Reopen an explicitly selected historical artifact without requiring present-day
/// source to match it. The acquisition record, context key and original digest must agree.
pub fn retained(path: &Path) -> Result<Artifact, ApiError> {
    if path.file_name().is_none_or(|name| name != "document.json") {
        return Err(invalid(
            "select an acquired API document.json with its original sidecars",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("API artifact parent missing"))?;
    let private = |path: &Path| {
        crate::test_inventory::read_private(path).map_err(|error| invalid(error.to_string()))
    };
    let context: Context = serde_json::from_slice(&private(&parent.join("context.json"))?)?;
    let record: ArtifactRecord = serde_json::from_slice(&private(&parent.join("artifact.json"))?)?;
    let bytes = private(path)?;
    if context.key()? != record.context_key || probe::digest(&bytes) != record.sha256 {
        return Err(invalid(
            "retained API identity, original bytes or context changed",
        ));
    }
    let document = decode(&bytes)?;
    if document.target.triple != context.target {
        return Err(invalid(
            "retained rustdoc target disagrees with acquisition context",
        ));
    }
    Ok(Artifact {
        context,
        path: path.to_owned(),
        sha256: record.sha256,
        document,
    })
}
pub(crate) fn artifact_bytes(artifact: &Artifact) -> Result<Vec<u8>, ApiError> {
    let bytes = crate::test_inventory::read_private(&artifact.path)
        .map_err(|error| invalid(error.to_string()))?;
    if probe::digest(&bytes) != artifact.sha256 {
        return Err(invalid("original API artifact bytes changed"));
    }
    Ok(bytes)
}
/// Passively select only a completed artifact in the currently observed exact context.
/// Missing preparation returns `None`; a malformed completed artifact remains an error.
pub fn locate(
    root: &Path,
    store: &Path,
    snapshot: &MetadataSnapshot,
    query: &str,
    target: Option<&str>,
    environment: &Environment,
) -> Result<Selection, ApiError> {
    let environment = extraction_environment(environment)?;
    let Observation { context, .. } =
        observed_context(root, store, snapshot, query, target, &environment)?;
    let directory = store.join("api").join(context.key()?);
    let mut entries = match fs::read_dir(&directory) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>()?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Selection {
                context,
                artifact: None,
            });
        }
        Err(error) => return Err(error.into()),
    };
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries.into_iter().rev() {
        let record = entry.path().join("artifact.json");
        if !record.is_file() {
            continue;
        }
        let stored: Context =
            serde_json::from_slice(&fs::read(entry.path().join("context.json"))?)?;
        let record: ArtifactRecord = serde_json::from_slice(&fs::read(record)?)?;
        if stored != context || record.context_key != context.key()? {
            return Err(invalid(
                "retained API context does not agree with its addressed key",
            ));
        }
        let path = entry.path().join("document.json");
        let bytes = fs::read(&path)?;
        if probe::digest(&bytes) != record.sha256 {
            return Err(invalid("retained API artifact digest changed"));
        }
        let document = decode(&bytes)?;
        return Ok(Selection {
            context: context.clone(),
            artifact: Some(Artifact {
                context,
                path,
                sha256: record.sha256,
                document,
            }),
        });
    }
    Ok(Selection {
        context,
        artifact: None,
    })
}
/// One current context observation, including explicit missing preparation.
#[derive(Debug)]
pub struct Selection {
    /// Exact observed package/target/source/toolchain context.
    pub context: Context,
    /// Successfully decoded finalized artifact, or absent preparation.
    pub artifact: Option<Artifact>,
}
/// Source-qualified route from one documented public reexport to an exact Cargo dependency.
#[derive(Debug)]
pub struct Reexport {
    /// Exact dependency package selected by Cargo's resolved edge.
    pub package_id: String,
    /// Public facade prefix, independent of the defining item's path.
    pub public_prefix: String,
    /// Canonical rustdoc declaration path used only within the addressed dependency artifact.
    pub definition_path: Vec<String>,
    /// Associated-item suffix selected below that exported prefix.
    pub suffix: String,
}
/// Follow native public reexport facts and the original exact dependency edge; never pick by name.
pub fn reexport(
    snapshot: &MetadataSnapshot,
    artifact: &Artifact,
    path: Option<&str>,
) -> Result<Option<Reexport>, ApiError> {
    let routes = query::external_uses(&artifact.document, path);
    if routes.is_empty() {
        return Ok(None);
    }
    if routes.len() != 1 {
        return Err(invalid(
            "multiple external reexports remain relevant; select an explicit public path or the addressed dependency",
        ));
    }
    let route = &routes[0];
    let node = snapshot
        .metadata
        .resolve
        .as_ref()
        .and_then(|resolve| {
            resolve
                .nodes
                .iter()
                .find(|node| node.id.repr == artifact.context.package_id)
        })
        .ok_or_else(|| invalid("exact facade package is missing from the resolved graph"))?;
    let edges: Vec<_> = node
        .deps
        .iter()
        .filter(|edge| edge.name == route.crate_name)
        .collect();
    let [edge] = edges.as_slice() else {
        return Err(invalid(
            "external rustdoc crate has no unique direct resolved Cargo edge; select an explicit source-qualified dependency",
        ));
    };
    Ok(Some(Reexport {
        package_id: edge.pkg.to_string(),
        public_prefix: route.public_prefix.clone(),
        definition_path: route.definition_path.clone(),
        suffix: route.suffix.clone(),
    }))
}

#[cfg(test)]
mod tests;
