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
    collections::{BTreeMap, BTreeSet},
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
pub mod doctests;
pub mod export;
pub mod query;

/// A version, acquisition or original native input failure.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// A source-addressed structural query syntax failure.
    #[error(transparent)]
    Query(#[from] query::QueryError),
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
    forward(this) { match this { Self::Query(error) => Some(error), Self::Probe(error) => Some(error), Self::Correctness(error) => Some(error), _ => None } },
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
    /// Canonical original acquisition directory for all relative source fingerprints.
    pub source_root: PathBuf,
    /// Cargo's source-qualified exact package ID.
    pub package_id: String,
    /// Exact locked source address, including local path identity.
    pub manifest: PathBuf,
    /// Selected source bytes, including non-Rust authored files.
    pub package_source_sha256: String,
    /// Original package-byte evidence for the selected documentation unit's reachable dependencies.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependency_sources: BTreeMap<String, PackageSourceEvidence>,
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
/// Byte evidence captured from an original Cargo package, including registry/git sources.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub enum PackageSourceEvidence {
    /// The complete package source membership and bytes were observed.
    Recorded {
        /// Original package-relative byte digest using the acquisition source owner.
        sha256: String,
    },
    /// Source bytes could not be captured; later qualification remains unknown.
    Unavailable {
        /// Original capture failure, without inventing a source identity.
        reason: String,
    },
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
    #[serde(default)]
    dependencies: Option<Vec<DocumentationDependency>>,
}
#[derive(Deserialize)]
struct DocumentationDependency {
    index: usize,
    #[serde(default)]
    extern_crate_name: Option<String>,
}
#[derive(Deserialize)]
struct DocumentationTarget {
    name: String,
    src_path: PathBuf,
    #[serde(default)]
    kind: Vec<String>,
}

/// One original Cargo library unit; package spellings never replace its qualified identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DocumentationLibraryProfile {
    pub package_id: String,
    pub library_root: PathBuf,
    pub features: BTreeSet<String>,
    pub target: String,
    pub role: String,
}
/// Original compiler-plan reachability, not the broader optional metadata feature universe.
#[derive(Debug)]
pub(crate) struct DocumentationClosure {
    pub root: DocumentationLibraryProfile,
    pub dependencies: BTreeSet<DocumentationLibraryProfile>,
}

fn retained_documentation_graph(artifact: &Artifact) -> Result<DocumentationGraph, ApiError> {
    let directory = artifact
        .path
        .parent()
        .ok_or_else(|| invalid("API artifact directory absent"))?;
    let bytes = crate::test_inventory::read_private(&directory.join("unit-graph.json"))
        .map_err(|error| invalid(error.to_string()))?;
    if probe::digest(&bytes) != artifact.context.unit_graph_sha256 {
        return Err(invalid("original API documentation unit graph changed"));
    }
    let graph: DocumentationGraph = serde_json::from_slice(&bytes)?;
    if graph.version != 1 {
        return Err(invalid(
            "unsupported retained API documentation graph version",
        ));
    }
    Ok(graph)
}
fn documentation_root(graph: &DocumentationGraph, context: &Context) -> Result<usize, ApiError> {
    for index in &graph.roots {
        if *index >= graph.units.len() {
            return Err(invalid("retained API graph root is outside its units"));
        }
    }
    let selected = graph
        .roots
        .iter()
        .filter(|index| {
            graph.units.get(**index).is_some_and(|unit| {
                unit.pkg_id == context.package_id
                    && unit.mode == "doc"
                    && unit.platform.as_deref() == Some(context.target.as_str())
                    && unit.features == context.features
            })
        })
        .copied()
        .collect::<Vec<_>>();
    let [root_index] = selected.as_slice() else {
        return Err(invalid(
            "API context has no unique original documentation unit",
        ));
    };
    Ok(*root_index)
}
fn documentation_closure(
    graph: &DocumentationGraph,
    context: &Context,
) -> Result<DocumentationClosure, ApiError> {
    let root_index = documentation_root(graph, context)?;
    let root = documentation_profile(&graph.units[root_index], &context.source_root)?;
    let mut dependencies = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut pending = vec![root_index];
    while let Some(index) = pending.pop() {
        if !seen.insert(index) {
            continue;
        }
        let unit = graph
            .units
            .get(index)
            .ok_or_else(|| invalid("retained API dependency edge is outside its units"))?;
        if index != root_index {
            if !matches!(unit.mode.as_str(), "build" | "check") {
                return Err(invalid(format!(
                    "retained API dependency mode {} has no proved native library profile",
                    unit.mode
                )));
            }
            let profile = documentation_profile(unit, &context.source_root)?;
            if profile.role != "lib" {
                return Err(invalid(
                    "retained API host/proc-macro dependency context is not proved by the native library closure",
                ));
            }
            // Cargo may plan the selected library's own compilation before documenting it.
            // It is the already compared root profile, not an additional semantic dependency.
            if profile != root {
                dependencies.insert(profile);
            }
        }
        let edges = unit
            .dependencies
            .as_ref()
            .ok_or_else(|| invalid("retained API graph omitted authoritative dependency edges"))?;
        pending.extend(edges.iter().map(|edge| edge.index));
    }
    Ok(DocumentationClosure { root, dependencies })
}

fn documentation_profile(
    unit: &DocumentationUnit,
    root: &Path,
) -> Result<DocumentationLibraryProfile, ApiError> {
    let kinds = &unit.target.kind;
    let role = if kinds.iter().any(|kind| kind == "proc-macro") {
        "proc-macro"
    } else if !kinds.is_empty()
        && kinds.iter().all(|kind| {
            matches!(
                kind.as_str(),
                "lib" | "rlib" | "dylib" | "cdylib" | "staticlib"
            )
        })
    {
        "lib"
    } else {
        return Err(invalid(
            "retained API build-only/nonlibrary unit has no native library equivalence",
        ));
    };
    let target=unit.platform.clone().ok_or_else(||invalid("retained API host unit has no explicit target profile; compiler equivalence remains unknown"))?;
    Ok(DocumentationLibraryProfile {
        package_id: unit.pkg_id.clone(),
        library_root: root.join(&unit.target.src_path).canonicalize()?,
        features: unit.features.clone(),
        target,
        role: role.into(),
    })
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
fn dependency_source_evidence(
    snapshot: &MetadataSnapshot,
    bytes: &[u8],
    package: &str,
) -> Result<BTreeMap<String, PackageSourceEvidence>, ApiError> {
    let graph: DocumentationGraph = serde_json::from_slice(bytes)?;
    if graph.version != 1 {
        return Err(invalid("unsupported documentation dependency graph"));
    }
    let mut pending = graph.roots.to_vec();
    let mut seen = BTreeSet::new();
    let mut packages = BTreeSet::new();
    while let Some(index) = pending.pop() {
        if !seen.insert(index) {
            continue;
        }
        let unit = graph
            .units
            .get(index)
            .ok_or_else(|| invalid("documentation dependency edge invalid"))?;
        if unit.pkg_id != package {
            packages.insert(unit.pkg_id.clone());
        }
        let edges = unit
            .dependencies
            .as_ref()
            .ok_or_else(|| invalid("documentation dependency edges absent"))?;
        pending.extend(edges.iter().map(|edge| edge.index));
    }
    let mut sources = BTreeMap::new();
    for package_id in packages {
        let observed = (|| {
            let package = snapshot
                .metadata
                .packages
                .iter()
                .find(|package| package.id.repr == package_id)
                .ok_or_else(|| {
                    invalid("documentation package source absent from original metadata")
                })?;
            let root = package
                .manifest_path
                .parent()
                .ok_or_else(|| invalid("dependency manifest parent absent"))?
                .as_std_path();
            let mut files = Vec::new();
            source_files(root, Path::new(""), &mut files)?;
            Ok::<_, ApiError>(source_files_digest(&files))
        })();
        sources.insert(
            package_id,
            match observed {
                Ok(sha256) => PackageSourceEvidence::Recorded { sha256 },
                Err(error) => PackageSourceEvidence::Unavailable {
                    reason: error.to_string(),
                },
            },
        );
    }
    Ok(sources)
}
fn extraction_environment(root: &Path, environment: &Environment) -> Result<Environment, ApiError> {
    let mut selected = crate::correctness::standalone_adapter_environment(
        &crate::environment::preparation_inputs(environment),
    )?;
    selected = crate::correctness::compiler_environment(root, &selected)?;
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
    let dependency_sources =
        dependency_source_evidence(snapshot, &unit_graph, &package.id.to_string())?;
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
            source_root: root.canonicalize()?,
            package_id: package.id.to_string(),
            manifest: package.manifest_path.as_std_path().to_owned(),
            package_source_sha256,
            dependency_sources,
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
    prepare_mode(root, store, snapshot, query, target, environment, false)
}
pub(crate) fn prepare_mode(
    root: &Path,
    store: &Path,
    snapshot: &MetadataSnapshot,
    query: &str,
    target: Option<&str>,
    environment: &Environment,
    doctests: bool,
) -> Result<Acquisition, ApiError> {
    let environment = extraction_environment(root, environment)?;
    let Observation {
        mut context,
        files,
        library,
        unit_graph,
    } = observed_context(root, store, snapshot, query, target, &environment)?;
    if doctests {
        context.extraction_flags = vec![
            "-Z".into(),
            "unstable-options".into(),
            "--output-format".into(),
            "doctest".into(),
        ];
        context.rustdoc_format = doctests::FORMAT_VERSION;
    }
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
    validate_acquisition(acquisition, snapshot)?;
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
fn validate_acquisition(
    acquisition: &Acquisition,
    snapshot: &MetadataSnapshot,
) -> Result<(), ApiError> {
    let store = acquisition
        .directory
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| invalid("API acquisition has no addressed store"))?;
    let mut after = observed_context(
        &acquisition.cwd,
        store,
        snapshot,
        &acquisition.query,
        Some(&acquisition.context.target),
        &acquisition.environment,
    )?;
    if acquisition
        .context
        .extraction_flags
        .last()
        .is_some_and(|flag| flag == "doctest")
    {
        after.context.extraction_flags = vec![
            "-Z".into(),
            "unstable-options".into(),
            "--output-format".into(),
            "doctest".into(),
        ];
        after.context.rustdoc_format = doctests::FORMAT_VERSION;
    }
    if after.context != acquisition.context {
        return Err(invalid(
            "API source/build context changed during extraction; retain original diagnostics and reacquire",
        ));
    }
    Ok(())
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
    let environment = extraction_environment(root, environment)?;
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
