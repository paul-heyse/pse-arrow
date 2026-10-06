// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected production unit identities. Cargo's unit graph supplies compilation
//! context; dep-info supplies consumed files, not arbitrary build-script I/O.
//! Unknown evidence prevents persistent reuse without preventing execution.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};
use pse_ids::{Frame, FramedHasher};
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use syn::visit_mut::{self, VisitMut};

/// Explicit reviewed build-script inputs. Cargo rerun hints alone are not proof
/// that arbitrary file/environment/process/network inputs are absent.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct BuildScriptInputs {
    pub(crate) complete: bool,
    /// Exact raw package source identity reviewed for arbitrary I/O. A changed
    /// implementation cannot inherit a previous completeness assertion.
    #[serde(default)]
    pub(crate) reviewed_source: Option<String>,
    #[serde(default)]
    pub(crate) files: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) environment: BTreeMap<String, String>,
}

/// A reviewed closure for one executable selected by an exact Cargo config key.
/// Completeness covers transitive executables, shared libraries, files, ambient
/// environment and arbitrary I/O; merely naming a linker/wrapper is insufficient.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ConfigurationExecutorInputs {
    /// Relative paths are relative to the selected workspace root.
    pub(crate) configuration: PathBuf,
    /// Table keys separately preserve target triples/cfg expressions containing dots.
    pub(crate) selector: Vec<String>,
    pub(crate) complete: bool,
    /// Receipt's cargo-executor review basis: exact raw configuration, selector,
    /// configured command and resolved executable path/bytes. Any change requires
    /// another review. This is not a blanket configuration-executor allowlist.
    #[serde(default)]
    pub(crate) reviewed_source: Option<String>,
    #[serde(default)]
    pub(crate) files: Vec<PathBuf>,
    /// Actual ambient caller values consumed by the closure; null means absent.
    /// Cargo-injected unit/config values are already captured by those authorities.
    /// The completeness assertion excludes additional unlisted ambient inputs.
    #[serde(default)]
    pub(crate) environment: BTreeMap<String, Option<String>>,
}

/// Root wiring may deserialize this declaration from an operator-owned JSON file.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct InputDeclarations {
    /// Bootstrap actual production compilation in this selected Cargo context, then
    /// associate compiler-artifact outputs with their emitted dep-info. This does
    /// not assert arbitrary build-script or procedural-macro I/O completeness.
    #[serde(default)]
    pub(crate) capture_actual_build: bool,
    #[serde(default)]
    pub(crate) build_scripts: BTreeMap<String, BuildScriptInputs>,
    /// Same source-bound reviewed executable-input contract for proc-macro owners.
    #[serde(default)]
    pub(crate) proc_macros: BTreeMap<String, BuildScriptInputs>,
    #[serde(default)]
    pub(crate) configuration_executors: Vec<ConfigurationExecutorInputs>,
    /// Keys are unit keys emitted in the output. A package key may be used only
    /// when that package has one compilation unit; ambiguous associations fail closed.
    #[serde(default)]
    pub(crate) dep_info: BTreeMap<String, Vec<PathBuf>>,
    #[serde(default)]
    pub(crate) native_inputs_complete: bool,
    pub(crate) native_abi: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProducerOptions {
    pub(crate) workspace_root: PathBuf,
    pub(crate) package: String,
    pub(crate) profile: String,
    pub(crate) target: Option<String>,
    pub(crate) features: Vec<String>,
    pub(crate) no_default_features: bool,
    pub(crate) dep_info: Vec<PathBuf>,
    pub(crate) declared_inputs: Vec<PathBuf>,
    pub(crate) native_inputs: Vec<PathBuf>,
    pub(crate) declared_environment: BTreeMap<String, String>,
    pub(crate) declarations: InputDeclarations,
}

/// Paths identify inputs; the per-input identity is versioned and framed too.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ConsumedInput {
    pub(crate) path: String,
    pub(crate) identity: String,
    pub(crate) representation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProducerIdentity {
    /// A fresh bootstrap binds this capture to its complete executable attestation.
    /// This changes capture eligibility, not the relevant producer identity.
    #[serde(default)]
    pub(crate) outer_attestation: Option<OuterAttestation>,
    pub(crate) frame: String,
    pub(crate) package: String,
    pub(crate) identity: String,
    pub(crate) persistent_reuse_eligible: bool,
    pub(crate) reasons: Vec<String>,
    pub(crate) units: Vec<ProductionUnit>,
    pub(crate) selected_lock_records: Vec<serde_json::Value>,
    pub(crate) consumed_inputs: BTreeMap<String, Vec<ConsumedInput>>,
    pub(crate) declared_environment: BTreeMap<String, String>,
    pub(crate) native_abi: Option<String>,
    /// Review-basis identities are evidence, separately from the scientific key.
    /// Reviewing a changed test file need not make that file a consumed dependency.
    #[serde(default)]
    pub(crate) reviewed_owner_sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct OuterAttestation {
    pub(crate) source: pse_ids::ContentHash,
    pub(crate) build: pse_ids::ContentHash,
}

#[derive(Clone, Debug, Deserialize)]
struct UnitGraph {
    version: u32,
    units: Vec<Unit>,
    roots: Vec<usize>,
}

#[derive(Clone, Debug, Deserialize)]
struct Unit {
    pkg_id: String,
    target: CargoTarget,
    profile: serde_json::Value,
    platform: Option<String>,
    mode: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    dependencies: Vec<Edge>,
    #[serde(default)]
    is_std: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoTarget {
    kind: Vec<String>,
    crate_types: Vec<String>,
    name: String,
    src_path: PathBuf,
    edition: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Edge {
    index: usize,
    extern_crate_name: String,
}

/// Canonical graph nodes carry dependency unit keys, never unstable array indices.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProductionUnit {
    pub(crate) key: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: Vec<String>,
    pub(crate) crate_types: Vec<String>,
    pub(crate) edition: String,
    pub(crate) mode: String,
    pub(crate) platform: Option<String>,
    pub(crate) profile: serde_json::Value,
    pub(crate) features: Vec<String>,
    pub(crate) dependencies: BTreeMap<String, String>,
}

fn cargo_output(root: &Path, args: &[String]) -> Result<Vec<u8>> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(args)
        .output()
        .context("running selected Cargo identity command")?;
    ensure!(
        output.status.success(),
        "selected Cargo command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

const FINITE_FIXTURE_MANIFEST: &str =
    "[package]\nname='producer-fixture'\nversion='0.1.0'\nedition='2024'\n";
const FINITE_FIXTURE_SOURCE: &str = "pub fn value()->u32 {2}\n";
/// Actual tool capture of a finite Rust-only fixture, for qualification/replay controls.
/// This is not a completeness claim about the production scientific kernel.
pub(crate) fn qualified_fixture(repository: &Path, output: &Path) -> Result<()> {
    const CHILD_ROOT: &str = "PSE_PRODUCER_FINITE_FIXTURE_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        ensure!(
            fs::read_to_string(root.join("Cargo.toml"))? == FINITE_FIXTURE_MANIFEST
                && fs::read_to_string(root.join("src/lib.rs"))? == FINITE_FIXTURE_SOURCE,
            "finite qualification fixture source changed"
        );
        // The selected owner has no build script, proc macro, dependency or external
        // native call. Only the actual selected pinned compiler emits this Rust rlib.
        cargo_output(&root, &["generate-lockfile".into(), "--offline".into()])?;
        let sysroot = Command::new("rustc")
            .current_dir(&root)
            .args(["--print", "sysroot"])
            .output()?;
        ensure!(
            sysroot.status.success(),
            "finite fixture rustc selection failed"
        );
        let compiler =
            PathBuf::from(std::str::from_utf8(&sysroot.stdout)?.trim()).join("bin/rustc");
        let options = ProducerOptions {
            workspace_root: root.clone(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: vec![compiler],
            declared_environment: std::env::vars()
                .filter(|(key, _)| caller_build_environment(key))
                .collect(),
            declarations: InputDeclarations {
                capture_actual_build: true,
                native_inputs_complete: true,
                native_abi: Some("finite-rust-rlib-pinned-compiler.v1".into()),
                ..Default::default()
            },
        };
        let mut identity = run(&options)?;
        ensure!(
            identity.persistent_reuse_eligible,
            "finite fixture remains unqualified: {:?}",
            identity.reasons
        );
        let artifacts = fs::read(root.join("target/debug/libproducer_fixture.rlib"))?;
        identity.outer_attestation = Some(OuterAttestation {
            source: pse_ids::ContentHash::parse_hex(&hash_parts(&[
                b"finite-fixture-source",
                FINITE_FIXTURE_MANIFEST.as_bytes(),
                FINITE_FIXTURE_SOURCE.as_bytes(),
            ]))?,
            build: pse_ids::ContentHash::parse_hex(&hash_parts(&[
                b"finite-fixture-build",
                &artifacts,
            ]))?,
        });
        write_if_changed(output, &identity)?;
        return Ok(());
    }
    let temporary = tempfile::tempdir()?;
    let root = temporary.path();
    fs::create_dir(root.join("src"))?;
    fs::create_dir(root.join("cargo-home"))?;
    fs::write(root.join("Cargo.toml"), FINITE_FIXTURE_MANIFEST)?;
    fs::write(root.join("src/lib.rs"), FINITE_FIXTURE_SOURCE)?;
    fs::copy(
        repository.join("rust-toolchain.toml"),
        root.join("rust-toolchain.toml"),
    )?;
    // A fresh empty Cargo home excludes unrelated user wrapper/config closures for
    // this dependency-free fixture only. The pinned rustup toolchain is unchanged.
    let result = Command::new(std::env::current_exe()?)
        .current_dir(repository)
        .args(["producer-identity-fixture", "--output"])
        .arg(output)
        .env(CHILD_ROOT, root)
        .env("CARGO_HOME", root.join("cargo-home"))
        .env_remove("RUSTC")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("RUSTFLAGS")
        .env_remove("RUSTDOCFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .output()?;
    ensure!(
        result.status.success(),
        "finite fixture capture failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

/// Actual selected profile/features/target; no tests, examples or benchmarks.
pub(crate) fn unit_graph_arguments(options: &ProducerOptions) -> Vec<String> {
    let mut args = vec![
        "build".into(),
        "-p".into(),
        options.package.clone(),
        "--lib".into(),
        "--profile".into(),
        options.profile.clone(),
        "--unit-graph".into(),
        "-Zunstable-options".into(),
        "--locked".into(),
        "--offline".into(),
    ];
    if let Some(target) = &options.target {
        args.extend(["--target".into(), target.clone()]);
    }
    if options.no_default_features {
        args.push("--no-default-features".into());
    }
    if !options.features.is_empty() {
        args.extend(["--features".into(), options.features.join(",")]);
    }
    args
}

pub(crate) fn run(options: &ProducerOptions) -> Result<ProducerIdentity> {
    let root = options
        .workspace_root
        .canonicalize()
        .context("producer workspace root")?;
    let lock_before = fs::read(root.join("Cargo.lock"))?;
    let manifest_before = fs::read(root.join("Cargo.toml"))?;
    let configuration_before = configuration_state(&root)?;
    let graph: UnitGraph =
        serde_json::from_slice(&cargo_output(&root, &unit_graph_arguments(options))?)
            .context("selected Cargo unit graph")?;
    ensure!(
        graph.version == 1,
        "unsupported Cargo unit graph version {}",
        graph.version
    );
    // Metadata names package roots, while the unit graph alone determines features
    // and closure. --all-features only makes optional package metadata available.
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&cargo_output(
        &root,
        &[
            "metadata".into(),
            "--format-version=1".into(),
            "--locked".into(),
            "--offline".into(),
            "--all-features".into(),
        ],
    )?)?;
    let packages = metadata
        .packages
        .into_iter()
        .map(|package| (package.id.to_string(), package))
        .collect();
    let messages = if options.declarations.capture_actual_build {
        let mut arguments = unit_graph_arguments(options);
        arguments
            .retain(|argument| !matches!(argument.as_str(), "--unit-graph" | "-Zunstable-options"));
        arguments.push("--message-format=json".into());
        parse_cargo_messages(&cargo_output(&root, &arguments)?)?
    } else {
        Vec::new()
    };
    ensure!(
        fs::read(root.join("Cargo.lock"))? == lock_before
            && fs::read(root.join("Cargo.toml"))? == manifest_before,
        "selected Cargo inputs changed during producer capture; capture again from one deployment baseline"
    );
    let identity = capture(options, &root, &graph, &packages, &messages)?;
    ensure!(
        configuration_state(&root)? == configuration_before,
        "Cargo configuration/executables changed during producer capture; capture again from one deployment baseline"
    );
    Ok(identity)
}

fn selected_indices(graph: &UnitGraph) -> Result<BTreeSet<usize>> {
    ensure!(
        !graph.roots.is_empty(),
        "selected producer has no unit roots"
    );
    let mut pending = graph.roots.clone();
    let mut selected = BTreeSet::new();
    while let Some(index) = pending.pop() {
        let unit = graph
            .units
            .get(index)
            .context("unit dependency index outside graph")?;
        ensure!(
            matches!(unit.mode.as_str(), "build" | "run-custom-build"),
            "production graph contains non-production mode {}",
            unit.mode
        );
        ensure!(
            !unit
                .target
                .kind
                .iter()
                .any(|kind| matches!(kind.as_str(), "test" | "bench" | "example")),
            "production graph contains a non-production target"
        );
        if selected.insert(index) {
            pending.extend(unit.dependencies.iter().map(|edge| edge.index));
        }
    }
    Ok(selected)
}

fn logical_package(id: &str, root: &Path) -> String {
    id.replace(
        &format!("path+file://{}/", root.display()),
        "path+workspace://",
    )
}

fn unit_key(unit: &Unit, root: &Path) -> Result<String> {
    let mut features = unit.features.clone();
    features.sort();
    features.dedup();
    let bytes = serde_json::to_vec(&(
        logical_package(&unit.pkg_id, root),
        &unit.target.name,
        &unit.target.kind,
        &unit.target.crate_types,
        &unit.target.edition,
        &unit.mode,
        &unit.platform,
        &unit.profile,
        unit.target
            .src_path
            .to_string_lossy()
            .replace(&root.to_string_lossy().to_string(), "workspace"),
        features,
    ))?;
    Ok(hash_parts(&[b"unit", &bytes]))
}

fn parse_cargo_messages(bytes: &[u8]) -> Result<Vec<serde_json::Value>> {
    let mut messages = Vec::new();
    for line in std::str::from_utf8(bytes)?
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let message: serde_json::Value =
            serde_json::from_str(line).context("actual Cargo JSON message")?;
        ensure!(
            message
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "Cargo evidence lacks message reason"
        );
        messages.push(message);
    }
    ensure!(
        messages.last().is_some_and(|message| message
            .get("reason")
            .and_then(serde_json::Value::as_str)
            == Some("build-finished")
            && message.get("success").and_then(serde_json::Value::as_bool) == Some(true)),
        "actual Cargo evidence lacks successful build completion"
    );
    Ok(messages)
}
fn artifact_matches(unit: &Unit, message: &serde_json::Value) -> bool {
    if unit.mode != "build"
        || message.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-artifact")
        || message
            .get("package_id")
            .and_then(serde_json::Value::as_str)
            != Some(unit.pkg_id.as_str())
    {
        return false;
    }
    let Some(target) = message.get("target") else {
        return false;
    };
    if target.get("name").and_then(serde_json::Value::as_str) != Some(unit.target.name.as_str())
        || target
            .get("src_path")
            .and_then(serde_json::Value::as_str)
            .map(Path::new)
            != Some(unit.target.src_path.as_path())
        || target.get("edition").and_then(serde_json::Value::as_str)
            != Some(unit.target.edition.as_str())
        || target.get("kind") != Some(&serde_json::json!(unit.target.kind))
        || target.get("crate_types") != Some(&serde_json::json!(unit.target.crate_types))
    {
        return false;
    }
    let Some(profile) = message.get("profile") else {
        return false;
    };
    if profile.get("test").and_then(serde_json::Value::as_bool) != Some(false) {
        return false;
    }
    for key in [
        "opt_level",
        "debuginfo",
        "debug_assertions",
        "overflow_checks",
    ] {
        if profile.get(key) != unit.profile.get(key) {
            return false;
        }
    }
    let Some(features) = message
        .get("features")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    let Some(mut features) = features
        .iter()
        .map(serde_json::Value::as_str)
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    features.sort_unstable();
    features.dedup();
    let mut expected = unit.features.iter().map(String::as_str).collect::<Vec<_>>();
    expected.sort_unstable();
    expected.dedup();
    features == expected
}
fn dep_info_outputs(text: &str) -> Result<BTreeSet<PathBuf>> {
    let joined = text.replace("\\\r\n", "").replace("\\\n", "");
    let mut outputs = BTreeSet::new();
    for line in joined
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
    {
        let mut escaped = false;
        let mut separator = None;
        for (index, character) in line.char_indices() {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == ':' {
                separator = Some(index);
                break;
            }
        }
        let prefix = &line[..separator.context("dep-info output rule lacks separator")?];
        outputs.extend(parse_dep_info(&format!("outputs: {prefix}"))?);
    }
    Ok(outputs)
}
fn artifact_dep_info(
    unit: &Unit,
    graph: &UnitGraph,
    messages: &[serde_json::Value],
    root: &Path,
) -> Result<Option<Vec<PathBuf>>> {
    let matching = messages
        .iter()
        .filter(|message| artifact_matches(unit, message))
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return Ok(None);
    }
    // Cargo messages do not carry the host/target unit platform. Identical contexts
    // are therefore ambiguous and still require an explicit unit-key association.
    if matching.iter().any(|message| {
        graph
            .units
            .iter()
            .filter(|candidate| artifact_matches(candidate, message))
            .count()
            != 1
    }) {
        return Ok(None);
    }
    let mut paths = BTreeSet::new();
    for message in matching {
        let Some(files) = message
            .get("filenames")
            .and_then(serde_json::Value::as_array)
        else {
            return Ok(None);
        };
        for file in files {
            let Some(file) = file.as_str() else {
                return Ok(None);
            };
            let artifact = absolute(root, Path::new(file));
            if !artifact.is_file() {
                continue;
            }
            let mut candidates = vec![artifact.with_extension("d")];
            if let Some(name) = artifact
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_prefix("lib"))
            {
                candidates.push(artifact.with_file_name(name).with_extension("d"));
            }
            for candidate in candidates.into_iter().filter(|path| path.is_file()) {
                let text = fs::read_to_string(&candidate)?;
                let outputs = dep_info_outputs(&text)?
                    .into_iter()
                    .map(|path| absolute(root, &path))
                    .collect::<BTreeSet<_>>();
                let inputs = parse_dep_info(&text)?
                    .into_iter()
                    .map(|path| absolute(root, &path))
                    .collect::<BTreeSet<_>>();
                if outputs.contains(&artifact) && inputs.contains(&unit.target.src_path) {
                    paths.insert(candidate);
                }
            }
        }
    }
    Ok((!paths.is_empty()).then(|| paths.into_iter().collect()))
}

fn capture(
    options: &ProducerOptions,
    root: &Path,
    graph: &UnitGraph,
    packages: &BTreeMap<String, cargo_metadata::Package>,
    messages: &[serde_json::Value],
) -> Result<ProducerIdentity> {
    let selected = selected_indices(graph)?;
    let keys = selected
        .iter()
        .map(|index| Ok((*index, unit_key(&graph.units[*index], root)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut reasons = BTreeSet::new();
    let mut inputs: BTreeMap<String, Vec<ConsumedInput>> = BTreeMap::new();
    let mut units = Vec::new();
    let mut selected_packages = BTreeSet::new();
    let mut reviewed_owner_sources = BTreeMap::new();
    let package_counts = selected.iter().fold(BTreeMap::new(), |mut counts, index| {
        *counts
            .entry(graph.units[*index].pkg_id.clone())
            .or_insert(0_usize) += 1;
        counts
    });
    let mut associated = BTreeMap::<usize, Vec<PathBuf>>::new();
    for index in &selected {
        let unit = &graph.units[*index];
        let key = &keys[index];
        let declared = options.declarations.dep_info.get(key).or_else(|| {
            if package_counts[&unit.pkg_id] == 1 {
                options.declarations.dep_info.get(&unit.pkg_id)
            } else {
                None
            }
        });
        if let Some(paths) = declared {
            associated.insert(*index, paths.clone());
        } else if let Some(paths) = artifact_dep_info(unit, graph, messages, root)? {
            associated.insert(*index, paths);
        }
    }
    let fallback_packages = selected
        .iter()
        .filter(|index| {
            graph.units[**index].mode == "build" && associated.get(index).is_none_or(Vec::is_empty)
        })
        .map(|index| graph.units[*index].pkg_id.clone())
        .collect::<BTreeSet<_>>();
    for index in &selected {
        let unit = &graph.units[*index];
        let key = &keys[index];
        let package = packages
            .get(&unit.pkg_id)
            .context("unit package absent from Cargo metadata")?;
        let package_root = package
            .manifest_path
            .as_std_path()
            .parent()
            .context("package manifest parent")?;
        let package_id = logical_package(&unit.pkg_id, root);
        if (unit.mode == "run-custom-build"
            || unit.target.kind.iter().any(|kind| kind == "proc-macro"))
            && !reviewed_owner_sources.contains_key(&package_id)
        {
            reviewed_owner_sources
                .insert(package_id.clone(), reviewed_source_identity(package_root)?);
        }
        let first_package_unit = selected_packages.insert(unit.pkg_id.clone());
        if unit.is_std {
            reasons.insert(format!(
                "{package_id}: build-std source evidence is not declared"
            ));
        }
        let mut features = unit.features.clone();
        features.sort();
        features.dedup();
        let dependencies = unit
            .dependencies
            .iter()
            .map(|edge| {
                Ok((
                    format!(
                        "{}:{}",
                        edge.extern_crate_name,
                        keys.get(&edge.index).context("closure edge missing")?
                    ),
                    keys.get(&edge.index)
                        .context("closure edge missing")?
                        .clone(),
                ))
            })
            .collect::<Result<_>>()?;
        units.push(ProductionUnit {
            key: key.clone(),
            package_id: package_id.clone(),
            target_name: unit.target.name.clone(),
            target_kind: unit.target.kind.clone(),
            crate_types: unit.target.crate_types.clone(),
            edition: unit.target.edition.clone(),
            mode: unit.mode.clone(),
            platform: unit.platform.clone(),
            profile: unit.profile.clone(),
            features,
            dependencies,
        });
        let group = format!("package:{package_id}");
        // Package-wide fallback covers consumed helpers/assets without a hand-maintained
        // filename allowlist. It may re-key on irrelevant files within this package.
        if first_package_unit && fallback_packages.contains(&unit.pkg_id) {
            let files = package_files(package_root)?;
            let mut raw_package = false;
            let mut uncertain_sources = 0_usize;
            let mut first_uncertainty = None;
            for path in &files {
                if path.extension().is_some_and(|extension| extension == "rs") {
                    match fs::read(path) {
                        Ok(bytes) => {
                            let (_, _, uncertainty) = production_source(&bytes);
                            raw_package |= !uncertainty.is_empty();
                            if let Some(reason) = uncertainty.first() {
                                uncertain_sources += 1;
                                first_uncertainty
                                    .get_or_insert_with(|| format!("{}: {reason}", path.display()));
                            }
                        }
                        Err(error) => {
                            raw_package = true;
                            reasons.insert(format!("{}: {error}", path.display()));
                        }
                    }
                }
            }
            // Raw package bytes cover source representation/location uncertainty.
            // Missing actual unit associations and arbitrary executable I/O remain
            // separate eligibility refusals below.
            let _ = (first_uncertainty, uncertain_sources);
            // A macro can read another .rs file as text or observe its locations.
            // Token normalization in that other file would silently omit consumed
            // bytes, so unknown macro/cfg evidence makes the whole package raw.
            for path in files {
                add_file(
                    &mut inputs,
                    &mut reasons,
                    &group,
                    &path,
                    package_root,
                    &package_id,
                    raw_package,
                )?;
            }
        } else if first_package_unit {
            // The actual selected units, not package-directory recursion, determine
            // consumed helpers/assets. Uncompiled tests and examples are excluded.
            add_file(
                &mut inputs,
                &mut reasons,
                &group,
                package.manifest_path.as_std_path(),
                package_root,
                &package_id,
                true,
            )?;
        }
        if unit.target.kind.iter().any(|kind| kind == "proc-macro") {
            let declaration = options
                .declarations
                .proc_macros
                .get(&unit.pkg_id)
                .or_else(|| options.declarations.proc_macros.get(&package_id));
            if let Some(declaration) = declaration.filter(|declaration| {
                review_matches(declaration, &reviewed_owner_sources[&package_id])
            }) {
                add_reviewed_inputs(
                    &mut inputs,
                    &mut reasons,
                    "proc-macro-inputs",
                    declaration,
                    root,
                    &package_id,
                )?;
            } else {
                reasons.insert(format!("{package_id}: procedural macro arbitrary I/O lacks a current source-bound review"));
            }
        }
        if let Some(paths) = associated.get(index) {
            if paths.is_empty() {
                reasons.insert(format!(
                    "unit {key}: associated rustc dep-info list is empty"
                ));
            }
            for dep_info in paths {
                add_dep_info(
                    &mut inputs,
                    &mut reasons,
                    &group,
                    dep_info,
                    root,
                    package_root,
                    &package_id,
                )?;
            }
        } else if unit.mode == "build" {
            reasons.insert(format!(
                "unit {key}: no associated actual rustc dep-info; package fallback only"
            ));
        }
        if unit.mode == "run-custom-build" {
            let declaration = options
                .declarations
                .build_scripts
                .get(&unit.pkg_id)
                .or_else(|| options.declarations.build_scripts.get(&package_id));
            if let Some(declaration) = declaration.filter(|declaration| {
                review_matches(declaration, &reviewed_owner_sources[&package_id])
            }) {
                add_reviewed_inputs(
                    &mut inputs,
                    &mut reasons,
                    "build-script-inputs",
                    declaration,
                    root,
                    &package_id,
                )?;
            } else {
                reasons.insert(format!(
                    "{package_id}: build-script arbitrary I/O lacks a current source-bound review"
                ));
            }
        }
    }
    for message in messages.iter().filter(|message| {
        message.get("reason").and_then(serde_json::Value::as_str) == Some("build-script-executed")
    }) {
        let Some(id) = message
            .get("package_id")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        if !selected_packages.contains(id) {
            continue;
        }
        let payload = serde_json::to_vec(
            &serde_json::json!({"linked_libs":message.get("linked_libs"),"linked_paths":message.get("linked_paths"),"cfgs":message.get("cfgs"),"env":message.get("env")}),
        )?;
        inputs
            .entry("actual-build-script-output".into())
            .or_default()
            .push(ConsumedInput {
                path: logical_package(id, root),
                identity: hash_parts(&[b"build-script-output", &payload]),
                representation: "actual-cargo-message".into(),
            });
    }
    for path in &options.dep_info {
        add_dep_info(
            &mut inputs,
            &mut reasons,
            "unassociated-dep-info",
            path,
            root,
            root,
            "workspace",
        )?;
    }
    // A loose dep-info list adds consumed include/OUT_DIR inputs but cannot prove
    // which selected units it covers. Only the explicit unit association above can.
    for (group, paths) in [
        ("declared-inputs", &options.declared_inputs),
        ("native-inputs", &options.native_inputs),
    ] {
        for path in paths {
            add_file(
                &mut inputs,
                &mut reasons,
                group,
                &absolute(root, path),
                root,
                "workspace",
                true,
            )?;
        }
    }
    if !options.declarations.native_inputs_complete
        || options
            .declarations
            .native_abi
            .as_ref()
            .is_none_or(|abi| abi.trim().is_empty())
    {
        reasons.insert("native ABI/binary/input completeness is unknown".into());
    }
    if options.declarations.native_inputs_complete && options.native_inputs.is_empty() {
        reasons.insert("native completeness declares no concrete binary/input evidence".into());
    }
    add_file(
        &mut inputs,
        &mut reasons,
        "toolchain",
        &root.join("rust-toolchain.toml"),
        root,
        "workspace",
        true,
    )?;
    for path in cargo_configuration_files(root) {
        capture_configuration(
            &mut inputs,
            &mut reasons,
            &mut reviewed_owner_sources,
            &path,
            root,
            &options.declarations.configuration_executors,
            current_environment,
        )?;
    }
    let rustc = Command::new("rustc")
        .current_dir(root)
        .arg("-vV")
        .output()
        .context("selected rustc identity")?;
    ensure!(rustc.status.success(), "selected rustc identity failed");
    inputs
        .entry("toolchain".into())
        .or_default()
        .push(ConsumedInput {
            path: "rustc -vV".into(),
            identity: hash_parts(&[b"rustc", &rustc.stdout]),
            representation: "actual-version".into(),
        });
    let sysroot = Command::new("rustc")
        .current_dir(root)
        .args(["--print", "sysroot"])
        .output()
        .context("selected rustc sysroot")?;
    ensure!(sysroot.status.success(), "selected rustc sysroot failed");
    let compiler = PathBuf::from(std::str::from_utf8(&sysroot.stdout)?.trim()).join("bin/rustc");
    add_file(
        &mut inputs,
        &mut reasons,
        "compiler-executables",
        &compiler,
        root,
        "workspace",
        true,
    )?;
    // Cargo/rustc environment not declared by the caller cannot be excluded from
    // compilation semantics just because a package source snapshot was complete.
    for (key, value) in std::env::vars().filter(|(key, _)| caller_build_environment(key)) {
        if options.declared_environment.get(&key) != Some(&value) {
            reasons.insert(format!(
                "active build environment {key} has no matching declared value"
            ));
            inputs
                .entry("undeclared-active-environment".into())
                .or_default()
                .push(ConsumedInput {
                    path: key.clone(),
                    identity: hash_parts(&[b"environment-value", value.as_bytes()]),
                    representation: "actual-value-digest".into(),
                });
        }
        if matches!(
            key.as_str(),
            "RUSTC" | "RUSTC_WRAPPER" | "RUSTC_WORKSPACE_WRAPPER"
        ) && !value.is_empty()
        {
            if let Some(path) = executable_path(&value, root) {
                add_file(
                    &mut inputs,
                    &mut reasons,
                    "compiler-executables",
                    &path,
                    root,
                    "workspace",
                    true,
                )?;
            } else {
                reasons.insert(format!(
                    "active compiler executable {key} could not be resolved"
                ));
            }
        }
    }
    units.sort_by(|a, b| a.key.cmp(&b.key));
    for values in inputs.values_mut() {
        values.sort();
        values.dedup();
    }
    let selected_lock_records = lock_records(root, &selected_packages, packages)?;
    let declared_environment = options
        .declared_environment
        .iter()
        .map(|(name, value)| {
            (
                name.clone(),
                hash_parts(&[b"environment-value", value.as_bytes()]),
            )
        })
        .collect();
    let mut output = ProducerIdentity {
        frame: Frame::ProducerV1.as_str().into(),
        outer_attestation: None,
        package: options.package.clone(),
        identity: String::new(),
        persistent_reuse_eligible: reasons.is_empty(),
        reasons: reasons.into_iter().collect(),
        units,
        selected_lock_records,
        consumed_inputs: inputs,
        declared_environment,
        native_abi: options.declarations.native_abi.clone(),
        reviewed_owner_sources,
    };
    // Diagnostic reasons and eligibility do not silently alter scientific identity.
    // Completeness is an independent condition for reuse of this identity.
    let bytes = serde_json::to_vec(&(
        &output.package,
        &output.units,
        &output.selected_lock_records,
        &output.consumed_inputs,
        &output.declared_environment,
        &output.native_abi,
    ))?;
    output.identity = hash_parts(&[b"producer", &bytes]);
    Ok(output)
}

fn hash_parts(parts: &[&[u8]]) -> String {
    let mut hash = FramedHasher::new(Frame::ProducerV1);
    for part in parts {
        hash.part(part);
    }
    hash.finish_hash().to_hex()
}
fn reviewed_source_identity(root: &Path) -> Result<String> {
    let mut hash = FramedHasher::new(Frame::ProducerV1);
    hash.str("reviewed-executable-source");
    for path in package_files(root)? {
        hash.str(&path.strip_prefix(root)?.to_string_lossy())
            .part(&fs::read(path)?);
    }
    Ok(hash.finish_hash().to_hex())
}
fn review_matches(declaration: &BuildScriptInputs, source: &str) -> bool {
    declaration.complete && declaration.reviewed_source.as_deref() == Some(source)
}
fn add_reviewed_inputs(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    reasons: &mut BTreeSet<String>,
    group: &str,
    declaration: &BuildScriptInputs,
    root: &Path,
    package: &str,
) -> Result<()> {
    for path in &declaration.files {
        add_file(
            inputs,
            reasons,
            group,
            &absolute(root, path),
            root,
            "workspace",
            true,
        )?;
    }
    let values = serde_json::to_vec(&declaration.environment)?;
    inputs
        .entry(format!("{group}-environment"))
        .or_default()
        .push(ConsumedInput {
            path: package.into(),
            identity: hash_parts(&[b"environment", &values]),
            representation: "reviewed-values".into(),
        });
    Ok(())
}

fn absolute(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.into()
    } else {
        root.join(path)
    }
}
fn caller_build_environment(key: &str) -> bool {
    // These variables describe the xtask executable's package, not the selected
    // production units. Cargo replaces them for each compiled package; their
    // actual env!/option_env! consumption remains recorded by rustc dep-info.
    const PACKAGE_VARIABLES: &[&str] = &[
        "CARGO_MANIFEST_DIR",
        "CARGO_MANIFEST_PATH",
        "CARGO_MANIFEST_LINKS",
        "CARGO_CRATE_NAME",
        "CARGO_BIN_NAME",
        "CARGO_PRIMARY_PACKAGE",
        "CARGO_PKG_VERSION",
        "CARGO_PKG_VERSION_MAJOR",
        "CARGO_PKG_VERSION_MINOR",
        "CARGO_PKG_VERSION_PATCH",
        "CARGO_PKG_VERSION_PRE",
        "CARGO_PKG_AUTHORS",
        "CARGO_PKG_NAME",
        "CARGO_PKG_DESCRIPTION",
        "CARGO_PKG_HOMEPAGE",
        "CARGO_PKG_REPOSITORY",
        "CARGO_PKG_LICENSE",
        "CARGO_PKG_LICENSE_FILE",
        "CARGO_PKG_RUST_VERSION",
        "CARGO_PKG_README",
    ];
    !PACKAGE_VARIABLES.contains(&key)
        && (key.starts_with("CARGO_")
            || matches!(
                key,
                "RUSTC"
                    | "RUSTFLAGS"
                    | "RUSTDOCFLAGS"
                    | "RUSTC_WRAPPER"
                    | "RUSTC_WORKSPACE_WRAPPER"
            ))
}

fn cargo_configuration_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    for ancestor in root.ancestors() {
        for name in ["config", "config.toml"] {
            let path = ancestor.join(".cargo").join(name);
            if path.is_file() {
                paths.insert(path);
            }
        }
    }
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")));
    if let Some(home) = cargo_home {
        for name in ["config", "config.toml"] {
            let path = absolute(root, &home).join(name);
            if path.is_file() {
                paths.insert(path);
            }
        }
    }
    paths
}

fn executable_path(value: &str, root: &Path) -> Option<PathBuf> {
    let path = Path::new(value);
    if path.components().count() > 1 || path.is_absolute() {
        let path = absolute(root, path);
        return path.is_file().then_some(path);
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|directory| absolute(root, &directory).join(value))
            .find(|path| path.is_file())
    })
}

fn configuration_executables<'a>(
    config: &'a toml::Value,
    selector: &mut Vec<String>,
    output: &mut Vec<(Vec<String>, &'a toml::Value)>,
) {
    match config {
        toml::Value::Table(values) => {
            for (key, value) in values {
                selector.push(key.clone());
                if matches!(
                    key.as_str(),
                    "rustc"
                        | "rustdoc"
                        | "rustc-wrapper"
                        | "rustc-workspace-wrapper"
                        | "linker"
                        | "runner"
                ) {
                    // Empty wrapper paths explicitly disable that wrapper in Cargo.
                    if !(matches!(key.as_str(), "rustc-wrapper" | "rustc-workspace-wrapper")
                        && value.as_str() == Some(""))
                    {
                        output.push((selector.clone(), value));
                    }
                } else {
                    configuration_executables(value, selector, output);
                }
                selector.pop();
            }
        }
        toml::Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                selector.push(index.to_string());
                configuration_executables(value, selector, output);
                selector.pop();
            }
        }
        _ => {}
    }
}

fn configured_executable(value: &toml::Value, selector: &[String], root: &Path) -> Option<PathBuf> {
    let command = match value {
        toml::Value::String(command)
            if !(selector.last().is_some_and(|key| key == "runner")
                && command.contains(char::is_whitespace)) =>
        {
            command.as_str()
        }
        toml::Value::Array(arguments)
            if selector.last().is_some_and(|key| key == "runner")
                && arguments.iter().all(|argument| argument.as_str().is_some()) =>
        {
            arguments.first()?.as_str()?
        }
        _ => return None,
    };
    let path = Path::new(command);
    if command.is_empty()
        || (!path.is_absolute()
            && (path.components().count() != 1 || command.contains(char::is_whitespace)))
    {
        // Multi-component relative paths and runner strings with arguments need
        // Cargo-relative/string-splitting semantics not established by this route.
        // Absolute paths (including spaces) and bare PATH names are unambiguous.
        return None;
    }
    let resolved = executable_path(command, root)?;
    let metadata = fs::metadata(&resolved).ok()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }
    metadata.is_file().then_some(resolved)
}

fn configuration_executor_basis(
    configuration: &[u8],
    selector: &[String],
    value: &toml::Value,
    executable: &Path,
    binary: &[u8],
) -> Result<String> {
    Ok(hash_parts(&[
        b"reviewed-cargo-configuration-executor.v1",
        configuration,
        &serde_json::to_vec(&(selector, value, executable))?,
        binary,
    ]))
}

fn current_environment(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => anyhow::bail!("non-UTF8 environment value"),
    }
}

fn raw_input(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    group: &str,
    path: &Path,
    root: &Path,
    bytes: &[u8],
) {
    let name = path.strip_prefix(root).map_or_else(
        |_| path.to_string_lossy().into_owned(),
        |relative| format!("workspace/{}", relative.display()),
    );
    inputs.entry(group.into()).or_default().push(ConsumedInput {
        path: name,
        identity: hash_parts(&[b"input", b"raw", bytes]),
        representation: "raw".into(),
    });
}

fn capture_configuration(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    reasons: &mut BTreeSet<String>,
    review_bases: &mut BTreeMap<String, String>,
    path: &Path,
    root: &Path,
    declarations: &[ConfigurationExecutorInputs],
    environment: impl Fn(&str) -> Result<Option<String>>,
) -> Result<()> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            reasons.insert(format!(
                "unreadable Cargo configuration {}: {error}",
                path.display()
            ));
            return Ok(());
        }
    };
    raw_input(inputs, "cargo-configuration", path, root, &bytes);
    let config = match std::str::from_utf8(&bytes)
        .ok()
        .and_then(|text| toml::from_str::<toml::Value>(text).ok())
    {
        Some(config) => config,
        None => {
            reasons.insert(format!(
                "Cargo configuration {} could not be interpreted",
                path.display()
            ));
            return Ok(());
        }
    };
    if config.get("include").is_some() {
        reasons.insert(format!(
            "Cargo configuration {} has an unqualified include closure",
            path.display()
        ));
    }
    let mut selections = Vec::new();
    configuration_executables(&config, &mut Vec::new(), &mut selections);
    for (selector, value) in selections {
        let label = format!("{}:{}", path.display(), serde_json::to_string(&selector)?);
        let Some(executable) = configured_executable(value, &selector, root) else {
            reasons.insert(format!(
                "Cargo configuration executor {label} has unsupported syntax or cannot be resolved"
            ));
            continue;
        };
        let binary = match fs::read(&executable) {
            Ok(binary) => binary,
            Err(error) => {
                reasons.insert(format!(
                    "Cargo configuration executor {label} is unreadable: {error}"
                ));
                continue;
            }
        };
        raw_input(
            inputs,
            "configuration-executables",
            &executable,
            root,
            &binary,
        );
        let basis = configuration_executor_basis(&bytes, &selector, value, &executable, &binary)?;
        review_bases.insert(format!("cargo-executor:{label}"), basis.clone());
        let matching = declarations
            .iter()
            .filter(|declaration| {
                absolute(root, &declaration.configuration) == path
                    && declaration.selector == selector
            })
            .collect::<Vec<_>>();
        let [declaration] = matching.as_slice() else {
            reasons.insert(format!(
                "Cargo configuration executor {label} lacks one exact closure review"
            ));
            continue;
        };
        if !declaration.complete || declaration.reviewed_source.as_deref() != Some(basis.as_str()) {
            reasons.insert(format!("Cargo configuration executor {label} lacks a current source-bound complete closure review"));
            continue;
        }
        for file in &declaration.files {
            add_file(
                inputs,
                reasons,
                "configuration-executor-inputs",
                &absolute(root, file),
                root,
                "workspace",
                true,
            )?;
        }
        let mut actual = BTreeMap::new();
        for (name, expected) in &declaration.environment {
            if name.is_empty() || name.contains(['=', '\0']) {
                reasons.insert(format!(
                    "Cargo configuration executor {label} declares an invalid environment name"
                ));
                continue;
            }
            match environment(name) {
                Ok(value) => {
                    if &value != expected {
                        reasons.insert(format!("Cargo configuration executor {label} environment {name} differs from its reviewed value"));
                    }
                    actual.insert(name, value);
                }
                Err(_) => {
                    reasons.insert(format!("Cargo configuration executor {label} environment {name} could not be captured"));
                }
            }
        }
        inputs
            .entry("configuration-executor-environment".into())
            .or_default()
            .push(ConsumedInput {
                path: label,
                identity: hash_parts(&[
                    b"actual-configuration-executor-environment",
                    &serde_json::to_vec(&actual)?,
                ]),
                representation: "actual-value-digest".into(),
            });
    }
    Ok(())
}

/// A review of post-build bytes cannot qualify a different tool/configuration
/// used during capture. Unknown selections remain explicit refusals in capture.
fn configuration_state(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut state = BTreeMap::new();
    for path in cargo_configuration_files(root) {
        let bytes = fs::read(&path)
            .with_context(|| format!("Cargo configuration snapshot {}", path.display()))?;
        state.insert(
            format!("config:{}", path.display()),
            hash_parts(&[b"raw-configuration", &bytes]),
        );
        if let Some(config) = std::str::from_utf8(&bytes)
            .ok()
            .and_then(|text| toml::from_str::<toml::Value>(text).ok())
        {
            let mut selections = Vec::new();
            configuration_executables(&config, &mut Vec::new(), &mut selections);
            for (selector, value) in selections {
                if let Some(executable) = configured_executable(value, &selector, root) {
                    let binary = fs::read(&executable).with_context(|| {
                        format!("Cargo executable snapshot {}", executable.display())
                    })?;
                    state.insert(
                        format!(
                            "executor:{}:{}",
                            path.display(),
                            serde_json::to_string(&selector)?
                        ),
                        configuration_executor_basis(
                            &bytes,
                            &selector,
                            value,
                            &executable,
                            &binary,
                        )?,
                    );
                }
            }
        }
    }
    Ok(state)
}

fn package_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .with_context(|| format!("package source directory {}", directory.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                if entry.file_name() != ".git" && entry.file_name() != "target" {
                    pending.push(path);
                }
            } else if entry.file_type()?.is_file() || entry.file_type()?.is_symlink() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn add_file(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    reasons: &mut BTreeSet<String>,
    group: &str,
    path: &Path,
    base: &Path,
    label: &str,
    force_raw: bool,
) -> Result<()> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            reasons.insert(format!(
                "unreadable consumed input {}: {error}",
                path.display()
            ));
            return Ok(());
        }
    };
    let name = path.strip_prefix(base).map_or_else(
        |_| path.to_string_lossy().into_owned(),
        |relative| format!("{label}/{}", relative.display()),
    );
    let (content, representation, uncertainty) =
        if !force_raw && path.extension().is_some_and(|x| x == "rs") {
            production_source(&bytes)
        } else {
            (bytes, "raw".into(), Vec::new())
        };
    // Once raw bytes are retained, macro source locations/cfg/unparsed syntax
    // cannot be lost by token normalization. External arbitrary I/O remains an
    // owning proc-macro/build-script/config completeness obligation, independently.
    debug_assert!(uncertainty.is_empty() || representation == "raw");
    inputs.entry(group.into()).or_default().push(ConsumedInput {
        path: name,
        identity: hash_parts(&[b"input", representation.as_bytes(), &content]),
        representation,
    });
    Ok(())
}

fn add_dep_info(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    reasons: &mut BTreeSet<String>,
    group: &str,
    dep_info: &Path,
    root: &Path,
    base: &Path,
    label: &str,
) -> Result<()> {
    let path = absolute(root, dep_info);
    let bytes = match fs::read_to_string(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            reasons.insert(format!(
                "unreadable rustc dep-info {}: {error}",
                path.display()
            ));
            return Ok(());
        }
    };
    for input in parse_dep_info(&bytes)? {
        add_file(
            inputs,
            reasons,
            group,
            &absolute(root, &input),
            base,
            label,
            false,
        )?;
    }
    // Rustc emits env-dep comments for env!/option_env!; these values are consumed too.
    for line in bytes.lines().filter(|line| line.starts_with("# env-dep:")) {
        inputs
            .entry("rustc-environment".into())
            .or_default()
            .push(ConsumedInput {
                path: line
                    .strip_prefix("# env-dep:")
                    .unwrap_or(line)
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .into(),
                identity: hash_parts(&[b"env-dep", line.as_bytes()]),
                representation: "actual-rustc-env-dep".into(),
            });
    }
    Ok(())
}

/// Make-style rustc dependency files: escaped whitespace, #, :, backslashes,
/// continuation lines and multiple/phony rules. Only prerequisites are inputs.
pub(crate) fn parse_dep_info(text: &str) -> Result<BTreeSet<PathBuf>> {
    let joined = text.replace("\\\r\n", "").replace("\\\n", "");
    let mut result = BTreeSet::new();
    for line in joined.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let mut escaped = false;
        let mut separator = None;
        for (index, character) in line.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
            } else if character == ':' {
                separator = Some(index);
                break;
            }
        }
        let tail = &line[separator.context("dep-info rule has no unescaped colon")? + 1..];
        let mut token = String::new();
        let mut escaped = false;
        let mut characters = tail.chars().peekable();
        while let Some(character) = characters.next() {
            if escaped {
                token.push(character);
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '#' {
                break;
            } else if character == '$' && characters.peek() == Some(&'$') {
                token.push('$');
                let _ = characters.next();
            } else if character.is_whitespace() {
                if !token.is_empty() {
                    result.insert(PathBuf::from(std::mem::take(&mut token)));
                }
            } else {
                token.push(character);
            }
        }
        ensure!(!escaped, "dep-info ends with an incomplete escape");
        if !token.is_empty() {
            result.insert(token.into());
        }
    }
    ensure!(
        !result.is_empty(),
        "dep-info contains no consumed prerequisites"
    );
    Ok(result)
}

fn lock_records(
    root: &Path,
    ids: &BTreeSet<String>,
    packages: &BTreeMap<String, cargo_metadata::Package>,
) -> Result<Vec<serde_json::Value>> {
    let lock: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.lock"))?)?;
    let records = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .context("Cargo.lock package records")?;
    let mut selected = Vec::new();
    for id in ids {
        let package = packages.get(id).context("lock package metadata")?;
        let source = package.source.as_ref().map(ToString::to_string);
        let record = records
            .iter()
            .find(|record| {
                record.get("name").and_then(toml::Value::as_str) == Some(package.name.as_str())
                    && record.get("version").and_then(toml::Value::as_str)
                        == Some(package.version.to_string().as_str())
                    && cargo_sources_match(
                        record.get("source").and_then(toml::Value::as_str),
                        source.as_deref(),
                    )
            })
            .with_context(|| {
                format!("selected unit {id} has no matching lock record (source {source:?})")
            })?;
        // Dependency edges are already selected by Cargo's unit graph. Whole lock
        // dependency lists include unselected optional/dev edges and are excluded.
        selected.push(serde_json::json!({"package_id": logical_package(id, root), "name": package.name,
            "version": package.version.to_string(), "source": source, "checksum": record.get("checksum").and_then(toml::Value::as_str)}));
    }
    selected.sort_by_key(serde_json::Value::to_string);
    Ok(selected)
}

// Cargo metadata decodes git query values while Cargo.lock retains URL encoding.
// Compare decoded query pairs without weakening exact repository/commit identity.
fn cargo_sources_match(left: Option<&str>, right: Option<&str>) -> bool {
    if left == right {
        return true;
    }
    let normalize = |source: Option<&str>| -> Option<(url::Url, Vec<(String, String)>)> {
        let mut url = url::Url::parse(source?.strip_prefix("git+")?).ok()?;
        let mut query = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<Vec<_>>();
        query.sort();
        url.set_query(None);
        Some((url, query))
    };
    match (normalize(left), normalize(right)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

fn cfg_test(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Meta>()
                .is_ok_and(|meta| matches!(meta, syn::Meta::Path(path) if path.is_ident("test")))
    })
}

fn item_attributes(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(x) => &x.attrs,
        syn::Item::Enum(x) => &x.attrs,
        syn::Item::ExternCrate(x) => &x.attrs,
        syn::Item::Fn(x) => &x.attrs,
        syn::Item::ForeignMod(x) => &x.attrs,
        syn::Item::Impl(x) => &x.attrs,
        syn::Item::Macro(x) => &x.attrs,
        syn::Item::Mod(x) => &x.attrs,
        syn::Item::Static(x) => &x.attrs,
        syn::Item::Struct(x) => &x.attrs,
        syn::Item::Trait(x) => &x.attrs,
        syn::Item::TraitAlias(x) => &x.attrs,
        syn::Item::Type(x) => &x.attrs,
        syn::Item::Union(x) => &x.attrs,
        syn::Item::Use(x) => &x.attrs,
        _ => &[],
    }
}

struct ProductionSyntax {
    uncertainty: BTreeSet<String>,
}
impl VisitMut for ProductionSyntax {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        file.items.retain(|item| !cfg_test(item_attributes(item)));
        visit_mut::visit_file_mut(self, file);
    }
    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        if let Some((_, items)) = &mut item.content {
            items.retain(|item| !cfg_test(item_attributes(item)));
        }
        visit_mut::visit_item_mod_mut(self, item);
    }
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        block.stmts.retain(|statement| !matches!(statement, syn::Stmt::Item(item) if cfg_test(item_attributes(item))));
        visit_mut::visit_block_mut(self, block);
    }
    fn visit_item_impl_mut(&mut self, item: &mut syn::ItemImpl) {
        item.items.retain(|item| {
            !cfg_test(match item {
                syn::ImplItem::Const(x) => &x.attrs,
                syn::ImplItem::Fn(x) => &x.attrs,
                syn::ImplItem::Type(x) => &x.attrs,
                syn::ImplItem::Macro(x) => &x.attrs,
                _ => &[],
            })
        });
        visit_mut::visit_item_impl_mut(self, item);
    }
    fn visit_item_trait_mut(&mut self, item: &mut syn::ItemTrait) {
        item.items.retain(|item| {
            !cfg_test(match item {
                syn::TraitItem::Const(x) => &x.attrs,
                syn::TraitItem::Fn(x) => &x.attrs,
                syn::TraitItem::Type(x) => &x.attrs,
                syn::TraitItem::Macro(x) => &x.attrs,
                _ => &[],
            })
        });
        visit_mut::visit_item_trait_mut(self, item);
    }
    fn visit_item_foreign_mod_mut(&mut self, item: &mut syn::ItemForeignMod) {
        item.items.retain(|item| {
            !cfg_test(match item {
                syn::ForeignItem::Fn(x) => &x.attrs,
                syn::ForeignItem::Static(x) => &x.attrs,
                syn::ForeignItem::Type(x) => &x.attrs,
                syn::ForeignItem::Macro(x) => &x.attrs,
                _ => &[],
            })
        });
        visit_mut::visit_item_foreign_mod_mut(self, item);
    }
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        self.uncertainty.insert(format!(
            "macro {} may observe source locations or undeclared inputs",
            mac.path.to_token_stream()
        ));
        visit_mut::visit_macro_mut(self, mac);
    }
    fn visit_attribute_mut(&mut self, attribute: &mut syn::Attribute) {
        let path = attribute.path();
        if path.is_ident("cfg") || path.is_ident("cfg_attr") {
            self.uncertainty
                .insert("unresolved active cfg/cfg_attr requires raw source".into());
        } else if ![
            "doc",
            "allow",
            "warn",
            "deny",
            "forbid",
            "repr",
            "inline",
            "cold",
            "must_use",
            "deprecated",
            "no_mangle",
            "export_name",
            "link",
            "link_name",
        ]
        .iter()
        .any(|name| path.is_ident(name))
        {
            self.uncertainty.insert(format!(
                "attribute {} may be a source-sensitive procedural macro",
                path.to_token_stream()
            ));
        }
        visit_mut::visit_attribute_mut(self, attribute);
    }
    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        if matches!(item, syn::Item::Verbatim(_)) {
            self.uncertainty
                .insert("unparsed syntax requires raw source".into());
        }
        visit_mut::visit_item_mut(self, item);
    }
}

fn production_source(bytes: &[u8]) -> (Vec<u8>, String, Vec<String>) {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return (
            bytes.into(),
            "raw".into(),
            vec!["non-UTF8 Rust source".into()],
        );
    };
    let Ok(mut syntax) = syn::parse_file(text) else {
        return (
            bytes.into(),
            "raw".into(),
            vec!["unparsed Rust source".into()],
        );
    };
    let mut visitor = ProductionSyntax {
        uncertainty: BTreeSet::new(),
    };
    visitor.visit_file_mut(&mut syntax);
    if visitor.uncertainty.is_empty() {
        (
            syntax.into_token_stream().to_string().into_bytes(),
            "production-tokens".into(),
            Vec::new(),
        )
    } else {
        (
            bytes.into(),
            "raw".into(),
            visitor.uncertainty.into_iter().collect(),
        )
    }
}

/// Do not touch an unchanged per-producer output; no global generated identity.
pub(crate) fn write_if_changed(path: &Path, identity: &ProducerIdentity) -> Result<bool> {
    let mut bytes = serde_json::to_vec_pretty(identity)?;
    bytes.push(b'\n');
    if fs::read(path).is_ok_and(|existing| existing == bytes) {
        return Ok(false);
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut file, &bytes)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|error| error.error)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration_fixture() -> (tempfile::TempDir, PathBuf, ConfigurationExecutorInputs) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir(root.join(".cargo")).unwrap();
        let tool = root.join("reviewed-linker");
        fs::write(&tool, b"reviewed executable bytes").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let path = root.join(".cargo/config.toml");
        let selector = vec![
            "target".into(),
            "x86_64-unknown-linux-gnu".into(),
            "linker".into(),
        ];
        let config = BTreeMap::from([(
            "target",
            BTreeMap::from([(
                "x86_64-unknown-linux-gnu",
                BTreeMap::from([("linker", tool.to_str().unwrap())]),
            )]),
        )]);
        fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        fs::write(
            root.join("linker-input"),
            b"first reviewed transitive input",
        )
        .unwrap();
        let mut inputs = BTreeMap::new();
        let mut reasons = BTreeSet::new();
        let mut bases = BTreeMap::new();
        capture_configuration(
            &mut inputs,
            &mut reasons,
            &mut bases,
            &path,
            root,
            &[],
            |_| Ok(None),
        )
        .unwrap();
        assert_eq!(reasons.len(), 1);
        let declaration = ConfigurationExecutorInputs {
            configuration: PathBuf::from(".cargo/config.toml"),
            selector,
            complete: true,
            reviewed_source: Some(bases.values().next().unwrap().clone()),
            files: vec!["linker-input".into()],
            environment: BTreeMap::from([
                ("REVIEWED_VALUE".into(), Some("exact".into())),
                ("REVIEWED_ABSENCE".into(), None),
            ]),
        };
        (directory, path, declaration)
    }

    type ConfigurationCapture = (
        BTreeMap<String, Vec<ConsumedInput>>,
        BTreeSet<String>,
        BTreeMap<String, String>,
    );

    fn reviewed_configuration_capture(
        root: &Path,
        path: &Path,
        declarations: &[ConfigurationExecutorInputs],
        actual: Option<&str>,
    ) -> ConfigurationCapture {
        let mut inputs = BTreeMap::new();
        let mut reasons = BTreeSet::new();
        let mut bases = BTreeMap::new();
        capture_configuration(
            &mut inputs,
            &mut reasons,
            &mut bases,
            path,
            root,
            declarations,
            |name| {
                Ok(if name == "REVIEWED_VALUE" {
                    actual.map(str::to_owned)
                } else {
                    None
                })
            },
        )
        .unwrap();
        (inputs, reasons, bases)
    }

    #[test]
    fn producer_configuration_executor_current_review_captures_exact_closure() {
        let (directory, path, declaration) = configuration_fixture();
        let root = directory.path();
        let (before, reasons, _) = reviewed_configuration_capture(
            root,
            &path,
            std::slice::from_ref(&declaration),
            Some("exact"),
        );
        assert!(reasons.is_empty(), "{reasons:?}");
        assert_eq!(before["configuration-executables"].len(), 1);
        assert_eq!(before["configuration-executor-inputs"].len(), 1);
        assert_eq!(before["configuration-executor-environment"].len(), 1);
        fs::write(
            root.join("linker-input"),
            b"changed actual transitive input",
        )
        .unwrap();
        let (after, reasons, _) = reviewed_configuration_capture(
            root,
            &path,
            std::slice::from_ref(&declaration),
            Some("exact"),
        );
        assert!(reasons.is_empty());
        assert_ne!(
            serde_json::to_vec(&before).unwrap(),
            serde_json::to_vec(&after).unwrap()
        );
        fs::remove_file(root.join("linker-input")).unwrap();
        let (_, reasons, _) =
            reviewed_configuration_capture(root, &path, &[declaration], Some("exact"));
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("unreadable consumed input"))
        );
    }

    #[test]
    fn producer_configuration_executor_refuses_stale_binary_config_and_reviews() {
        let (directory, path, mut declaration) = configuration_fixture();
        let root = directory.path();
        declaration.complete = false;
        assert!(
            !reviewed_configuration_capture(
                root,
                &path,
                std::slice::from_ref(&declaration),
                Some("exact")
            )
            .1
            .is_empty()
        );
        declaration.complete = true;
        assert!(
            !reviewed_configuration_capture(
                root,
                &path,
                &[declaration.clone(), declaration.clone()],
                Some("exact")
            )
            .1
            .is_empty()
        );
        declaration.selector = vec!["build".into(), "rustc-wrapper".into()];
        assert!(
            !reviewed_configuration_capture(
                root,
                &path,
                std::slice::from_ref(&declaration),
                Some("exact")
            )
            .1
            .is_empty()
        );
        declaration.selector = vec![
            "target".into(),
            "x86_64-unknown-linux-gnu".into(),
            "linker".into(),
        ];
        fs::write(
            root.join("reviewed-linker"),
            b"different executable implementation",
        )
        .unwrap();
        let (changed, reasons, _) = reviewed_configuration_capture(
            root,
            &path,
            std::slice::from_ref(&declaration),
            Some("exact"),
        );
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("current source-bound"))
        );
        assert_eq!(changed["configuration-executables"].len(), 1);
        fs::write(root.join("reviewed-linker"), b"reviewed executable bytes").unwrap();
        let old_state = configuration_state(root).unwrap();
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\n# changed reviewed configuration bytes\n");
        fs::write(&path, text).unwrap();
        assert_ne!(old_state, configuration_state(root).unwrap());
        let (_, reasons, _) =
            reviewed_configuration_capture(root, &path, &[declaration], Some("exact"));
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("current source-bound"))
        );
    }

    #[test]
    fn producer_configuration_executor_refuses_environment_mismatch_and_unknown_io() {
        let (directory, path, declaration) = configuration_fixture();
        let root = directory.path();
        for actual in [Some("different"), None] {
            let (_, reasons, _) = reviewed_configuration_capture(
                root,
                &path,
                std::slice::from_ref(&declaration),
                actual,
            );
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason.contains("environment REVIEWED_VALUE differs"))
            );
        }
        let mut inputs = BTreeMap::new();
        let mut reasons = BTreeSet::new();
        let mut bases = BTreeMap::new();
        capture_configuration(
            &mut inputs,
            &mut reasons,
            &mut bases,
            &path,
            root,
            std::slice::from_ref(&declaration),
            |_| anyhow::bail!("unrepresentable actual value"),
        )
        .unwrap();
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("could not be captured"))
        );
        let config = fs::read_to_string(&path).unwrap();
        fs::write(&path, format!("include = 'unknown.toml'\n{config}")).unwrap();
        let (_, reasons, bases) = reviewed_configuration_capture(root, &path, &[], Some("exact"));
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("unqualified include closure"))
        );
        // Even a matching executor review cannot discharge an unknown include.
        let declaration = ConfigurationExecutorInputs {
            reviewed_source: Some(bases.values().next().unwrap().clone()),
            ..declaration
        };
        let (_, reasons, _) =
            reviewed_configuration_capture(root, &path, &[declaration], Some("exact"));
        assert_eq!(reasons.len(), 1);
        assert!(
            reasons
                .iter()
                .next()
                .unwrap()
                .contains("unqualified include closure")
        );
    }

    #[test]
    fn producer_configuration_executor_preserves_selectors_and_refuses_ambiguous_commands() {
        let (directory, path, _) = configuration_fixture();
        let root = directory.path();
        let tool = root.join("reviewed-linker").to_str().unwrap().to_owned();
        let selector = vec![
            "target".into(),
            "cfg(target_os = \"linux\")".into(),
            "runner".into(),
        ];
        let value = toml::Value::Array(vec![
            toml::Value::String(tool.clone()),
            toml::Value::String("--exact-arg".into()),
        ]);
        assert_eq!(
            configured_executable(&value, &selector, root),
            Some(root.join("reviewed-linker"))
        );
        assert!(
            configured_executable(
                &toml::Value::String(format!("{tool} --arg")),
                &selector,
                root
            )
            .is_none()
        );
        assert!(
            configured_executable(
                &toml::Value::String("./reviewed-linker".into()),
                &selector,
                root
            )
            .is_none()
        );
        assert!(configured_executable(&toml::Value::Integer(2), &selector, root).is_none());
        let config = BTreeMap::from([(
            "target",
            BTreeMap::from([(
                "cfg(target_os = \"linux\")",
                BTreeMap::from([("runner", vec![tool, "--exact-arg".into()])]),
            )]),
        )]);
        fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        let config: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let mut selections = Vec::new();
        configuration_executables(&config, &mut Vec::new(), &mut selections);
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].0, selector);
        let (_, reasons, _) = reviewed_configuration_capture(root, &path, &[], None);
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("lacks one exact closure review"))
        );
    }

    #[test]
    fn producer_actual_cargo_consumed_edit_rekeys_uncompiled_test_edit_does_not() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("tests")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='producer-fixture'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repository.join("rust-toolchain.toml"),
            root.join("rust-toolchain.toml"),
        )
        .unwrap();
        fs::write(root.join("src/lib.rs"),"mod helper; pub fn value()->u32 {helper::value()} #[cfg(test)] fn inline_test() { assert_eq!(1,1); }").unwrap();
        fs::write(root.join("src/helper.rs"), "pub fn value()->u32 {1}").unwrap();
        fs::write(
            root.join("tests/uncompiled.rs"),
            "#[test] fn unused() {assert!(true);}",
        )
        .unwrap();
        cargo_output(root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
        let options = ProducerOptions {
            workspace_root: root.into(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: Vec::new(),
            declared_environment: BTreeMap::new(),
            declarations: InputDeclarations {
                capture_actual_build: true,
                ..Default::default()
            },
        };
        let first = run(&options).unwrap();
        assert!(
            !first
                .reasons
                .iter()
                .any(|reason| reason.contains("no associated actual rustc dep-info")),
            "{:?}",
            first.reasons
        );
        assert!(
            first
                .consumed_inputs
                .values()
                .flatten()
                .any(|input| input.path.ends_with("src/helper.rs"))
        );
        assert!(
            !first
                .consumed_inputs
                .values()
                .flatten()
                .any(|input| input.path.ends_with("tests/uncompiled.rs"))
        );
        fs::write(root.join("src/lib.rs"),"mod helper; pub fn value()->u32 {helper::value()} #[cfg(test)] fn inline_test() { assert_eq!(2,2); }").unwrap();
        fs::write(
            root.join("tests/uncompiled.rs"),
            "#[test] fn unused() {assert!(false);}",
        )
        .unwrap();
        let tests_changed = run(&options).unwrap();
        assert_eq!(first.identity, tests_changed.identity);
        fs::write(root.join("src/helper.rs"), "pub fn value()->u32 {2}").unwrap();
        let production_changed = run(&options).unwrap();
        assert_ne!(first.identity, production_changed.identity);
        // This tests actual consumed-unit locality, not an assertion that every
        // deployment-native/environment input has been fully qualified.
    }

    fn artifact_fixture(root: &Path) -> (Unit, serde_json::Value) {
        let profile = serde_json::json!({"opt_level":"2","debuginfo":"line-tables-only","debug_assertions":true,"overflow_checks":true});
        let unit = Unit {
            pkg_id: "producer#0.1.0".into(),
            target: CargoTarget {
                kind: vec!["lib".into()],
                crate_types: vec!["lib".into()],
                name: "producer".into(),
                src_path: root.join("lib.rs"),
                edition: "2024".into(),
            },
            profile: profile.clone(),
            platform: None,
            mode: "build".into(),
            features: vec!["selected".into()],
            dependencies: Vec::new(),
            is_std: false,
        };
        let mut message_profile = profile;
        message_profile["test"] = serde_json::json!(false);
        let artifact = root.join("libproducer-123.rlib");
        fs::write(&artifact, b"fixture compiled output").unwrap();
        let message = serde_json::json!({"reason":"compiler-artifact","package_id":unit.pkg_id,"target":{"name":"producer","kind":["lib"],"crate_types":["lib"],"edition":"2024","src_path":unit.target.src_path},"profile":message_profile,"features":["selected"],"filenames":[artifact],"fresh":true});
        (unit, message)
    }
    #[test]
    fn producer_git_source_encoding_preserves_exact_commit_and_branch() {
        let encoded =
            "git+https://github.com/buoyant-data/delta-kernel-rs?branch=buoyant%2Fmain#8ba063f8";
        let decoded =
            "git+https://github.com/buoyant-data/delta-kernel-rs?branch=buoyant/main#8ba063f8";
        assert!(cargo_sources_match(Some(encoded), Some(decoded)));
        assert!(!cargo_sources_match(
            Some(encoded),
            Some("git+https://github.com/buoyant-data/delta-kernel-rs?branch=buoyant/main#changed")
        ));
        assert!(!cargo_sources_match(
            Some(encoded),
            Some("git+https://github.com/buoyant-data/delta-kernel-rs?branch=other/main#8ba063f8")
        ));
        assert!(!cargo_sources_match(Some(encoded), None));
    }
    #[test]
    fn producer_actual_artifact_inventory_excludes_uncompiled_and_inline_tests() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let (unit, message) = artifact_fixture(root);
        fs::write(
            root.join("lib.rs"),
            "mod helper; fn production() {} #[cfg(test)] fn test_only() { assert!(true); }",
        )
        .unwrap();
        fs::write(root.join("helper.rs"), "pub fn consumed() {}").unwrap();
        fs::write(root.join("unused-test.rs"), "not compiled").unwrap();
        fs::write(
            root.join("producer-123.d"),
            format!(
                "{}: {} {}\n",
                root.join("libproducer-123.rlib").display(),
                root.join("lib.rs").display(),
                root.join("helper.rs").display()
            ),
        )
        .unwrap();
        let graph = UnitGraph {
            version: 1,
            roots: vec![0],
            units: vec![unit.clone()],
        };
        let paths = artifact_dep_info(&unit, &graph, &[message], root)
            .unwrap()
            .unwrap();
        let capture = || {
            let mut inputs = BTreeMap::new();
            let mut reasons = BTreeSet::new();
            for path in &paths {
                add_dep_info(
                    &mut inputs,
                    &mut reasons,
                    "unit",
                    path,
                    root,
                    root,
                    "producer",
                )
                .unwrap();
            }
            assert!(reasons.is_empty());
            serde_json::to_vec(&inputs).unwrap()
        };
        let before = capture();
        fs::write(root.join("unused-test.rs"), "unrelated changed").unwrap();
        assert_eq!(before, capture());
        fs::write(
            root.join("lib.rs"),
            "mod helper; fn production() {} #[cfg(test)] fn test_only() { assert!(false); }",
        )
        .unwrap();
        assert_eq!(before, capture());
        fs::write(
            root.join("helper.rs"),
            "pub fn consumed() { let meaningful = 2; }",
        )
        .unwrap();
        assert_ne!(before, capture());
    }
    #[test]
    fn producer_artifact_context_and_output_linkage_refuse_ambiguity() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let (unit, mut message) = artifact_fixture(root);
        fs::write(root.join("lib.rs"), "fn production() {}").unwrap();
        fs::write(
            root.join("producer-123.d"),
            format!("wrong-output.rlib: {}\n", root.join("lib.rs").display()),
        )
        .unwrap();
        let graph = UnitGraph {
            version: 1,
            roots: vec![0],
            units: vec![unit.clone()],
        };
        assert!(
            artifact_dep_info(&unit, &graph, &[message.clone()], root)
                .unwrap()
                .is_none()
        );
        fs::write(
            root.join("producer-123.d"),
            format!(
                "{}: {}\n",
                root.join("libproducer-123.rlib").display(),
                root.join("lib.rs").display()
            ),
        )
        .unwrap();
        message["profile"]["test"] = serde_json::json!(true);
        assert!(!artifact_matches(&unit, &message));
        message["profile"]["test"] = serde_json::json!(false);
        message["features"] = serde_json::json!(["different"]);
        assert!(!artifact_matches(&unit, &message));
        message["features"] = serde_json::json!(["selected"]);
        let mut ambiguous = graph;
        let mut target_unit = unit.clone();
        target_unit.platform = Some("other-target".into());
        ambiguous.units.push(target_unit);
        assert!(
            artifact_dep_info(&unit, &ambiguous, &[message], root)
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn producer_actual_build_messages_and_reviews_fail_closed() {
        assert!(
            parse_cargo_messages(b"{\"reason\":\"build-finished\",\"success\":true}\n").is_ok()
        );
        assert!(parse_cargo_messages(b"{\"reason\":\"compiler-artifact\"}\n").is_err());
        assert!(
            parse_cargo_messages(b"{\"reason\":\"build-finished\",\"success\":false}\n").is_err()
        );
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("build.rs"), "fn main() {}").unwrap();
        let source = reviewed_source_identity(directory.path()).unwrap();
        let mut reviewed = BuildScriptInputs {
            complete: true,
            reviewed_source: Some(source.clone()),
            ..Default::default()
        };
        assert!(review_matches(&reviewed, &source));
        fs::write(
            directory.path().join("build.rs"),
            "fn main() { let changed = 1; }",
        )
        .unwrap();
        assert!(!review_matches(
            &reviewed,
            &reviewed_source_identity(directory.path()).unwrap()
        ));
        reviewed.reviewed_source = None;
        assert!(!review_matches(&reviewed, &source));
    }
    #[test]
    fn producer_caller_environment_excludes_cargo_injected_tool_package_fields() {
        assert!(!caller_build_environment("CARGO_PKG_NAME"));
        assert!(!caller_build_environment("CARGO_MANIFEST_DIR"));
        assert!(caller_build_environment("CARGO_ENCODED_RUSTFLAGS"));
        assert!(caller_build_environment("RUSTC_WORKSPACE_WRAPPER"));
        assert!(caller_build_environment("CARGO_PKG_ARBITRARY_INPUT"));
    }

    #[test]
    fn producer_inline_cfg_test_is_inactive_in_nested_items() {
        let before = b"mod a { fn p() { #[cfg(test)] fn helper() { panic!(\"a\"); } } #[cfg(test)] fn t() { assert!(true); } }";
        let after = b"mod a { fn p() { #[cfg(test)] fn helper() { panic!(\"changed\"); } } #[cfg(test)] fn t() { assert!(false); } }";
        assert_eq!(production_source(before).0, production_source(after).0);
        assert_eq!(production_source(before).1, "production-tokens");
    }

    #[test]
    fn producer_unknown_macro_location_requires_raw_rekey() {
        let a = production_source(b"fn x() -> u32 { line!() }\n#[cfg(test)] fn t() {}\n");
        let b = production_source(b"\nfn x() -> u32 { line!() }\n#[cfg(test)] fn t() {}\n");
        assert_eq!(a.1, "raw");
        assert!(!a.2.is_empty());
        assert_ne!(hash_parts(&[&a.0]), hash_parts(&[&b.0]));
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("lib.rs");
        fs::write(&source, b"fn x() -> u32 { line!() }\n").unwrap();
        let mut inputs = BTreeMap::new();
        let mut reasons = BTreeSet::new();
        add_file(
            &mut inputs,
            &mut reasons,
            "consumed",
            &source,
            directory.path(),
            "fixture",
            false,
        )
        .unwrap();
        assert!(reasons.is_empty());
        assert_eq!(inputs["consumed"][0].representation, "raw");
    }

    #[test]
    fn producer_consumed_helper_rekeys_unrelated_package_does_not() {
        let directory = tempfile::tempdir().unwrap();
        let package = directory.path().join("producer");
        fs::create_dir(&package).unwrap();
        fs::write(
            package.join("lib.rs"),
            "mod helper; fn f() { helper::f(); }",
        )
        .unwrap();
        fs::write(package.join("helper.rs"), "pub fn f() {}").unwrap();
        let inventory = |root: &Path| {
            let mut inputs = BTreeMap::new();
            let mut reasons = BTreeSet::new();
            for path in package_files(root).unwrap() {
                add_file(
                    &mut inputs,
                    &mut reasons,
                    "package",
                    &path,
                    root,
                    "producer",
                    false,
                )
                .unwrap();
            }
            serde_json::to_vec(&inputs).unwrap()
        };
        let before = inventory(&package);
        fs::write(directory.path().join("unrelated.rs"), "fn different() {}").unwrap();
        assert_eq!(before, inventory(&package));
        fs::write(package.join("helper.rs"), "pub fn f() { let _x = 1; }").unwrap();
        assert_ne!(before, inventory(&package));
    }

    #[test]
    fn producer_dep_info_parses_escaped_make_prerequisites() {
        let inputs = parse_dep_info("target/a\\ b.rlib target/a.d: src/lib.rs src/a\\ b.rs data/has\\#hash.bin \\\n generated/escaped\\:name.rs C\\:\\path\\\\name data/has$$dollar.bin\n\nsrc/lib.rs:\n# env-dep:VALUE=x\n").unwrap();
        assert!(inputs.contains(Path::new("src/a b.rs")));
        assert!(inputs.contains(Path::new("data/has#hash.bin")));
        assert!(inputs.contains(Path::new("generated/escaped:name.rs")));
        assert!(inputs.contains(Path::new("data/has$dollar.bin")));
        assert!(!inputs.contains(Path::new("target/a.d")));
    }

    #[test]
    fn producer_dep_info_keeps_generated_inputs_outside_package() {
        let directory = tempfile::tempdir().unwrap();
        let generated = directory.path().join("generated");
        fs::create_dir(&generated).unwrap();
        let asset = generated.join("asset.bin");
        fs::write(&asset, b"first").unwrap();
        let info = directory.path().join("unit.d");
        fs::write(&info, format!("target/lib.rlib: {}\n", asset.display())).unwrap();
        let capture = || {
            let mut inputs = BTreeMap::new();
            let mut reasons = BTreeSet::new();
            add_dep_info(
                &mut inputs,
                &mut reasons,
                "package",
                &info,
                directory.path(),
                Path::new("unrelated-package"),
                "producer",
            )
            .unwrap();
            assert!(reasons.is_empty());
            serde_json::to_vec(&inputs).unwrap()
        };
        let before = capture();
        fs::write(&asset, b"second").unwrap();
        assert_ne!(before, capture());
    }

    #[test]
    fn producer_selected_unit_closure_keeps_build_scripts_and_proc_macros() {
        let unit = |name: &str, mode: &str, kind: &str, dependencies: Vec<Edge>| Unit {
            pkg_id: name.into(),
            target: CargoTarget {
                kind: vec![kind.into()],
                crate_types: vec![kind.into()],
                name: name.into(),
                src_path: PathBuf::from("src/lib.rs"),
                edition: "2024".into(),
            },
            profile: serde_json::json!({"name":"dev"}),
            platform: None,
            mode: mode.into(),
            features: vec![],
            dependencies,
            is_std: false,
        };
        let graph = UnitGraph {
            version: 1,
            roots: vec![0],
            units: vec![
                unit(
                    "producer",
                    "build",
                    "lib",
                    vec![
                        Edge {
                            index: 1,
                            extern_crate_name: "build".into(),
                        },
                        Edge {
                            index: 2,
                            extern_crate_name: "macro".into(),
                        },
                    ],
                ),
                unit("build", "run-custom-build", "custom-build", vec![]),
                unit("macro", "build", "proc-macro", vec![]),
                unit("unrelated", "test", "lib", vec![]),
            ],
        };
        assert_eq!(selected_indices(&graph).unwrap(), BTreeSet::from([0, 1, 2]));
        let mut invalid = graph;
        invalid.roots = vec![3];
        assert!(selected_indices(&invalid).is_err());
    }

    #[test]
    fn producer_output_preserves_unchanged_per_producer_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("producer.json");
        let mut output = ProducerIdentity {
            frame: Frame::ProducerV1.as_str().into(),
            outer_attestation: None,
            package: "producer".into(),
            identity: "first".into(),
            persistent_reuse_eligible: false,
            reasons: vec!["unknown inputs".into()],
            units: Vec::new(),
            selected_lock_records: Vec::new(),
            consumed_inputs: BTreeMap::new(),
            declared_environment: BTreeMap::new(),
            native_abi: None,
            reviewed_owner_sources: BTreeMap::new(),
        };
        assert!(write_if_changed(&path, &output).unwrap());
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        assert!(!write_if_changed(&path, &output).unwrap());
        assert_eq!(modified, fs::metadata(&path).unwrap().modified().unwrap());
        output.identity = "changed".into();
        assert!(write_if_changed(&path, &output).unwrap());
        let written: ProducerIdentity = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(written.identity, "changed");
    }
}
