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
use pse_ids::{EncodingHasher, Frame, FramedHasher};
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
    pub(crate) reviewed_closures: Vec<String>,
    /// Required for reviewed dynamic macro I/O: exact actual selected caller inputs.
    /// None grants no caller-specific exclusion; this is evidence, not a key input.
    #[serde(default)]
    pub(crate) reviewed_callers: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) files: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) environment: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) absent_environment: Vec<String>,
    #[serde(default)]
    pub(crate) absent_files: Vec<PathBuf>,
    /// Selected search namespaces retain additions, modes and symlink topology,
    /// rather than only the files present when a review was expanded.
    #[serde(default)]
    pub(crate) namespaces: Vec<PathBuf>,
    /// Only minted from the source-reviewed Pest owner/caller contract.
    #[serde(skip)]
    pest_grammars: Vec<PestGrammarInput>,
}

#[derive(Clone, Debug)]
struct PestGrammarInput {
    caller: String,
    preferred: PathBuf,
    fallback: PathBuf,
}

/// Reviewed executable I/O, bound to exact package source bytes. This is not a
/// package allowlist: a source change makes the review unusable until revisited.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerReview {
    reviewed_source: String,
    /// Only the reviewed workspace buildinfo owner can classify its exact
    /// deployment provenance outputs separately from scientific inputs.
    #[serde(default)]
    deployment_provenance: bool,
    /// Exact selected executable dependency contexts reviewed with this owner.
    /// Helpers cannot inherit completeness after their source/features change.
    #[serde(default)]
    reviewed_closures: Vec<String>,
    /// Dynamic macro arguments can introduce new ambient inputs without changing
    /// the macro implementation. Such reviews bind actual caller compilation inputs.
    #[serde(default)]
    reviewed_callers: Option<Vec<String>>,
    rationale: String,
    /// Reviews may deliberately exclude executable branches outside the supported
    /// deployment. A newly enabled feature must obtain its own complete I/O review.
    #[serde(default)]
    excluded_features: Vec<String>,
    /// An empty list reviews every target branch. Otherwise a Cargo host unit
    /// uses the selected compiler's actual host triple, not a guessed target.
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default)]
    environment: Vec<String>,
    /// Source-reviewed branch exclusions are valid only while these selectors
    /// are absent. A newly enabled helper branch obtains a separate review.
    #[serde(default)]
    required_absent_environment: Vec<String>,
    #[serde(default)]
    package_files: Vec<PathBuf>,
    #[serde(default)]
    workspace_files: Vec<PathBuf>,
    /// Exact source-reviewed workspace absence premises. Reappearance requires
    /// a fresh branch review rather than silently inheriting this grant.
    #[serde(default)]
    workspace_absent_files: Vec<PathBuf>,
    /// Fixed paths read relative to the invoking Cargo package. Every selected
    /// package is a possible caller; retain exact presence/absence and raw bytes.
    #[serde(default)]
    caller_package_files: Vec<PathBuf>,
    /// Fixed grammar arguments for exact selected caller packages. Pest checks
    /// each caller-root path before the caller-root/src fallback.
    #[serde(default)]
    pest_grammars: BTreeMap<String, Vec<PathBuf>>,
    #[serde(default)]
    environment_trees: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceReviews {
    #[serde(default)]
    build_scripts: BTreeMap<String, OwnerReview>,
    #[serde(default)]
    proc_macros: BTreeMap<String, OwnerReview>,
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
    /// Reviewed transitive file implementations and input contract, separately
    /// from the primary executable/configuration basis above.
    #[serde(default)]
    pub(crate) reviewed_closure: Option<String>,
    #[serde(default)]
    pub(crate) files: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) absent_files: Vec<PathBuf>,
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
    /// Cargo-owned invalidation of only selected effectful executor packages in
    /// this profile, before guarded capture. Existing artifacts cannot establish
    /// a newly reviewed arbitrary-I/O branch merely by being CargoFresh.
    #[serde(default)]
    pub(crate) refresh_executors: bool,
    /// Optional private audit output for actual selected Cargo messages/commands.
    /// This records effects; it never grants completeness or enters the key.
    #[serde(default)]
    pub(crate) actual_build_evidence: Option<PathBuf>,
    /// Review catalogs are resolved against the workspace, not the caller's home.
    #[serde(default)]
    pub(crate) review_catalogs: Vec<PathBuf>,
    /// Capture actual ambient Cargo/rustc values rather than copying stale values
    /// into a host-specific declaration. This grants no executable I/O authority.
    #[serde(default)]
    pub(crate) capture_caller_environment: bool,
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
    /// Prior reviewed raw native/tool closure basis. Capture reports the current
    /// basis but never promotes it into a completeness assertion.
    #[serde(default)]
    pub(crate) native_reviewed_source: Option<String>,
    #[serde(default)]
    pub(crate) native_environment: BTreeMap<String, Option<String>>,
    /// Source-reviewed executor startup families, e.g. exported Bash functions.
    /// Their absence is a premise, not permission for arbitrary ambient inputs.
    #[serde(default)]
    pub(crate) native_absent_environment_prefixes: Vec<String>,
    #[serde(default)]
    pub(crate) native_absent_files: Vec<PathBuf>,
    #[serde(default)]
    pub(crate) native_namespaces: Vec<PathBuf>,
    pub(crate) native_abi: Option<String>,
    /// Minted only by the source-review loader, never from receipt JSON.
    #[serde(skip)]
    deployment_provenance: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProducerOptions {
    pub(crate) workspace_root: PathBuf,
    pub(crate) package: String,
    pub(crate) profile: String,
    pub(crate) target: Option<String>,
    pub(crate) production_target: ProducerTarget,
    pub(crate) features: Vec<String>,
    pub(crate) no_default_features: bool,
    pub(crate) dep_info: Vec<PathBuf>,
    pub(crate) declared_inputs: Vec<PathBuf>,
    pub(crate) native_inputs: Vec<PathBuf>,
    pub(crate) declared_environment: BTreeMap<String, String>,
    pub(crate) declarations: InputDeclarations,
}

/// One production root, independently of the target architecture triple.
#[derive(Clone, Debug, Serialize)]
pub(crate) enum ProducerTarget {
    Library,
    Binary(String),
    /// Match Maturin's Cargo rustc selection for a declared cdylib manifest.
    Cdylib(PathBuf),
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
    /// Actual selected Cargo production root, separate from dependency membership.
    #[serde(default)]
    pub(crate) selected_root: Option<String>,
    pub(crate) identity: String,
    pub(crate) persistent_reuse_eligible: bool,
    pub(crate) reasons: Vec<String>,
    pub(crate) units: Vec<ProductionUnit>,
    pub(crate) selected_lock_records: Vec<serde_json::Value>,
    pub(crate) consumed_inputs: BTreeMap<String, Vec<ConsumedInput>>,
    /// Exact actual deployment data excluded from the scientific producer key.
    #[serde(default)]
    pub(crate) deployment_provenance: BTreeMap<String, Vec<ConsumedInput>>,
    pub(crate) declared_environment: BTreeMap<String, String>,
    pub(crate) native_abi: Option<String>,
    /// Review-basis identities are evidence, separately from the scientific key.
    /// Reviewing a changed test file need not make that file a consumed dependency.
    #[serde(default)]
    pub(crate) reviewed_owner_sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct OuterAttestation {
    pub(crate) source: pse_ids::ContentHash,
    pub(crate) build: pse_ids::ContentHash,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UnitGraph {
    version: u32,
    units: Vec<Unit>,
    roots: Vec<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CargoTarget {
    kind: Vec<String>,
    crate_types: Vec<String>,
    name: String,
    src_path: PathBuf,
    edition: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        let mut options = ProducerOptions {
            workspace_root: root.clone(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
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
        // This fixture's complete native closure is the selected compiler only:
        // its fixed source has no native calls or arbitrary executable owner.
        // This scoped review must never be used for a production package.
        options.declarations.native_reviewed_source = Some(native_review_basis(&options, &root)?);
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
    let root = temporary.path().join("package");
    fs::create_dir(&root)?;
    fs::create_dir(root.join("src"))?;
    let cargo_home = temporary.path().join("cargo-home");
    fs::create_dir(&cargo_home)?;
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
        .env(CHILD_ROOT, &root)
        .env("CARGO_HOME", &cargo_home)
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
        if matches!(options.production_target, ProducerTarget::Cdylib(_)) {
            "rustc"
        } else {
            "build"
        }
        .into(),
        "--profile".into(),
        options.profile.clone(),
        "--unit-graph".into(),
        "-Zunstable-options".into(),
        "--locked".into(),
        "--offline".into(),
    ];
    match &options.production_target {
        ProducerTarget::Library => {
            args.extend(["-p".into(), options.package.clone(), "--lib".into()])
        }
        ProducerTarget::Binary(name) => args.extend([
            "-p".into(),
            options.package.clone(),
            "--bin".into(),
            name.clone(),
        ]),
        ProducerTarget::Cdylib(manifest) => args.extend([
            "--manifest-path".into(),
            absolute(&options.workspace_root, manifest)
                .to_string_lossy()
                .into_owned(),
            "--lib".into(),
        ]),
    }
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
    ensure!(
        options.declarations.actual_build_evidence.is_none()
            || options.declarations.capture_actual_build,
        "actual build evidence requires an actual selected production build"
    );
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
    validate_production_root(&graph, &options.production_target)?;
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
    let packages: BTreeMap<String, cargo_metadata::Package> = metadata
        .packages
        .into_iter()
        .map(|package| (package.id.to_string(), package))
        .collect();
    if let ProducerTarget::Cdylib(manifest) = &options.production_target {
        let package: &cargo_metadata::Package = packages
            .get(&graph.units[graph.roots[0]].pkg_id)
            .context("selected cdylib package metadata")?;
        ensure!(
            package.name.as_str() == options.package
                && package.manifest_path.as_std_path() == absolute(&root, manifest),
            "selected cdylib manifest does not match requested package"
        );
    }
    let options = expand_reviews(options, &root, &graph, &packages)?;
    let options = &options;
    let mut refresh_diagnostics = None;
    let refreshed_executors = if options.declarations.refresh_executors {
        ensure!(
            options.declarations.capture_actual_build,
            "executor refresh requires an actual selected production build"
        );
        let selected_packages = executor_refresh_packages(&graph)?;
        if !selected_packages.is_empty() {
            let mut arguments = vec!["clean".into(), "--profile".into(), options.profile.clone()];
            if let Some(target) = &options.target {
                arguments.extend(["--target".into(), target.clone()]);
            }
            let families = executor_refresh_families(&selected_packages, &packages)?;
            for family in &families {
                arguments.extend(["--package".into(), family.clone()]);
            }
            let output = Command::new("cargo")
                .current_dir(&root)
                .args(&arguments)
                .output()?;
            use std::io::Write;
            std::io::stderr().write_all(&output.stderr)?;
            ensure!(
                output.status.success(),
                "selected executor family refresh failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            refresh_diagnostics = Some(
                serde_json::json!({"arguments": arguments, "package_name_families": families,
                "stdout": String::from_utf8_lossy(&output.stdout), "stderr": String::from_utf8_lossy(&output.stderr)}),
            );
        }
        selected_packages
    } else {
        BTreeSet::new()
    };
    let reviewed_before = reviewed_input_state(options, &root)?;
    let native_namespaces_before = native_namespace_state(options, &root)?;
    // Source review and dep-info must describe the source that Cargo actually
    // compiled. Refuse a concurrently edited closure rather than associating a
    // previous artifact with post-build source bytes. This snapshot is evidence
    // consistency, not an additional whole-package scientific-key dependency.
    let sources_before = selected_source_state(&graph, &packages)?;
    let mut observed_build_outputs = BTreeMap::new();
    let messages = if options.declarations.capture_actual_build {
        let mut arguments = unit_graph_arguments(options);
        arguments
            .retain(|argument| !matches!(argument.as_str(), "--unit-graph" | "-Zunstable-options"));
        arguments.push("--message-format=json".into());
        let output = Command::new("cargo")
            .current_dir(&root)
            .args(&arguments)
            .output()
            .context("running selected Cargo production build")?;
        let messages = if output.status.success() {
            parse_cargo_messages(&output.stdout)?
        } else {
            Vec::new()
        };
        if options.declarations.actual_build_evidence.is_some() && output.status.success() {
            observed_build_outputs = selected_build_output_observations(&messages, &root)?;
        }
        if let Some(path) = &options.declarations.actual_build_evidence {
            let path = absolute(&root, path);
            let mut file = fs::OpenOptions::new();
            file.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                file.mode(0o600);
            }
            serde_json::to_writer(
                file.open(&path)?,
                &serde_json::json!({
                    "arguments": arguments, "unit_graph": graph,
                    "success": output.status.success(),
                    "stdout": String::from_utf8_lossy(&output.stdout),
                    "stderr": String::from_utf8_lossy(&output.stderr),
                    "selected_build_outputs": observed_build_outputs,
                    "native_caller_environment": native_caller_environment(options)?,
                    "guarded_input_state": reviewed_before,
                    "native_namespaces_before": native_namespaces_before,
                    "native_environment_prefixes": native_environment_prefix_state(options)?,
                    "refreshed_executor_packages": refreshed_executors,
                    "executor_refresh_diagnostics": refresh_diagnostics,
                }),
            )?;
        }
        ensure!(
            output.status.success(),
            "selected Cargo command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        for package in &refreshed_executors {
            let artifacts: Vec<_> = messages
                .iter()
                .filter(|message| {
                    message["reason"] == "compiler-artifact"
                        && message["package_id"] == package.as_str()
                        && message["target"]["kind"].as_array().is_some_and(|kinds| {
                            kinds
                                .iter()
                                .any(|kind| kind == "custom-build" || kind == "proc-macro")
                        })
                })
                .collect();
            ensure!(
                !artifacts.is_empty()
                    && artifacts.iter().all(|artifact| artifact["fresh"] == false),
                "selected executor {package} was not freshly compiled after Cargo-owned refresh"
            );
        }
        messages
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
    ensure!(
        reviewed_input_state(options, &root)? == reviewed_before,
        "reviewed executable inputs changed during producer capture"
    );
    ensure!(
        native_namespace_state(options, &root)? == native_namespaces_before,
        "selected native search namespaces changed during producer capture"
    );
    ensure!(
        selected_source_state(&graph, &packages)? == sources_before,
        "selected source closure changed during producer capture"
    );
    if options.declarations.actual_build_evidence.is_some() {
        ensure!(
            selected_build_output_observations(&messages, &root)? == observed_build_outputs,
            "selected build output evidence changed during producer capture"
        );
    }
    Ok(identity)
}

/// Private observation of the actual Cargo caller, before its child-specific
/// Cargo-injected variables. These values are evidence, never completeness grants.
fn native_caller_environment(
    options: &ProducerOptions,
) -> Result<BTreeMap<String, Option<String>>> {
    let mut names: BTreeSet<_> = [
        "PATH",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "LD_AUDIT",
        "RUSTC",
        "RUSTFMT",
        "RUSTUP_TOOLCHAIN",
        "RUSTUP_HOME",
        "RUSTUP_OVERRIDE_TOOLCHAIN",
        "CARGO_HOME",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "PSE_NATIVE_COMPILER_CACHE",
        "CLANG_PATH",
        "LIBCLANG_PATH",
        "LLVM_CONFIG_PATH",
        "CLANG_NO_DEFAULT_CONFIG",
        "BINDGEN_EXTRA_CLANG_ARGS",
        "CC",
        "CXX",
        "CFLAGS",
        "CXXFLAGS",
        "CFLAGS_x86_64_unknown_linux_gnu",
        "CXXFLAGS_x86_64_unknown_linux_gnu",
        "CC_ENABLE_DEBUG_OUTPUT",
        "CMAKE_TOOLCHAIN_FILE",
        "COMPILER_PATH",
        "GCC_EXEC_PREFIX",
        "IPOPT_DIR",
        "SCIPOPTDIR",
        "UNO_DIR",
        "PETSC_DIR",
        "SUITESPARSE_INCLUDE_DIR",
        "SUITESPARSE_LIBRARY_DIR",
        "GMP_MPFR_SYS_CACHE",
        "PYO3_PYTHON",
        "PYO3_CONFIG_FILE",
        "PYO3_ENVIRONMENT_SIGNATURE",
        "PYO3_NO_PYTHON",
        "PYO3_PRINT_CONFIG",
        "PYO3_CROSS",
        "PYO3_CROSS_LIB_DIR",
        "PYO3_CROSS_PYTHON_VERSION",
        "PYO3_CROSS_PYTHON_IMPLEMENTATION",
        "UNSAFE_PYO3_SKIP_VERSION_CHECK",
        "PYTHONPATH",
        "PYTHONHOME",
        "PYTHONNOUSERSITE",
        "COVERAGE_PROCESS_START",
        "COVERAGE_PROCESS_CONFIG",
        "_PYTHON_PROJECT_BASE",
        "_PYTHON_SYSCONFIGDATA_NAME",
        "_PYTHON_SYSCONFIGDATA_PATH",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    names.extend(options.declarations.native_environment.keys().cloned());
    names
        .into_iter()
        .map(|name| Ok((name.clone(), current_environment(&name)?)))
        .collect()
}

/// Snapshots only actual Cargo-selected build-script log/output namespaces.
/// The finite text inventory is for source review association, not input proof:
/// dep-info alone does not discharge native include-search or executor obligations.
fn selected_build_output_observations(
    messages: &[serde_json::Value],
    root: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut paths = BTreeSet::new();
    let mut examined = 0usize;
    for message in messages
        .iter()
        .filter(|message| message["reason"] == "build-script-executed")
    {
        let out = absolute(
            root,
            Path::new(
                message["out_dir"]
                    .as_str()
                    .context("actual build script output directory missing")?,
            ),
        );
        let parent = out
            .parent()
            .context("actual build script output parent missing")?;
        for log in [
            parent.join("run/stdout"),
            parent.join("run/stderr"),
            parent.join("output"),
            parent.join("stderr"),
        ] {
            if log.is_file() {
                paths.insert(log);
            }
        }
        let mut pending = vec![out];
        while let Some(directory) = pending.pop() {
            if !directory.is_dir() {
                continue;
            }
            for entry in fs::read_dir(directory)? {
                examined += 1;
                ensure!(
                    examined <= 262_144,
                    "selected build evidence inventory exceeds its finite bound"
                );
                let entry = entry?;
                let kind = entry.file_type()?;
                if kind.is_dir() {
                    pending.push(entry.path());
                } else if kind.is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "d")
                {
                    paths.insert(entry.path());
                }
            }
        }
    }
    let mut total = 0u64;
    let mut observed = BTreeMap::new();
    for path in paths {
        let extent = fs::metadata(&path)?.len();
        total = total
            .checked_add(extent)
            .context("selected build evidence extent overflow")?;
        ensure!(
            total <= 64 * 1024 * 1024,
            "selected build evidence text exceeds its finite 64 MiB bound"
        );
        observed.insert(
            path.to_string_lossy().into_owned(),
            fs::read_to_string(path)?,
        );
    }
    Ok(observed)
}

fn selected_source_state(
    graph: &UnitGraph,
    packages: &BTreeMap<String, cargo_metadata::Package>,
) -> Result<BTreeMap<String, String>> {
    let mut sources = BTreeMap::new();
    for index in selected_indices(graph)? {
        let id = &graph.units[index].pkg_id;
        if sources.contains_key(id) {
            continue;
        }
        let package = packages
            .get(id)
            .context("selected source owner absent from metadata")?;
        let root = package
            .manifest_path
            .as_std_path()
            .parent()
            .context("selected source root")?;
        sources.insert(id.clone(), reviewed_source_identity(root)?);
    }
    Ok(sources)
}

fn validate_production_root(graph: &UnitGraph, requested: &ProducerTarget) -> Result<()> {
    ensure!(
        graph.roots.len() == 1,
        "producer capture requires exactly one production root"
    );
    let root = graph
        .units
        .get(graph.roots[0])
        .context("production root outside graph")?;
    ensure!(
        root.mode == "build",
        "producer root is not a production build"
    );
    let matches = match requested {
        ProducerTarget::Library => root.target.kind.iter().any(|kind| {
            matches!(
                kind.as_str(),
                "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro"
            )
        }),
        ProducerTarget::Binary(name) => {
            !name.is_empty() && root.target.name == *name && root.target.kind == ["bin"]
        }
        ProducerTarget::Cdylib(_) => {
            root.target.kind == ["cdylib"] && root.target.crate_types == ["cdylib"]
        }
    };
    ensure!(
        matches,
        "Cargo production root does not match requested target {requested:?}"
    );
    Ok(())
}

/// Exact selected IDs remain evidence and the post-refresh freshness obligation.
/// Cargo clean invalidates package-name families, including every version in the
/// selected profile; version and source qualifiers are ignored by that command.
fn executor_refresh_packages(graph: &UnitGraph) -> Result<BTreeSet<String>> {
    let packages: BTreeSet<_> = selected_indices(graph)?
        .into_iter()
        .filter_map(|index| {
            let unit = &graph.units[index];
            (unit.mode == "run-custom-build"
                || unit.target.kind.iter().any(|kind| kind == "proc-macro"))
            .then(|| unit.pkg_id.clone())
        })
        .collect();
    ensure!(
        packages.len() <= 256,
        "selected executor refresh exceeds its finite 256 package bound"
    );
    ensure!(
        packages
            .iter()
            .all(|package| package.contains('#') && !package.contains('\0')),
        "selected executor refresh requires exact source/version package IDs"
    );
    Ok(packages)
}

fn executor_refresh_families(
    packages: &BTreeSet<String>,
    metadata: &BTreeMap<String, cargo_metadata::Package>,
) -> Result<BTreeSet<String>> {
    let mut families = BTreeSet::new();
    for package in packages {
        // Resolve actual package identities through Cargo metadata; neither a
        // target name nor its checkout directory is necessarily a package name.
        let name = metadata
            .get(package)
            .context("selected executor package metadata")?
            .name
            .to_string();
        ensure!(
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "invalid selected executor package-name family"
        );
        families.insert(name);
    }
    ensure!(
        families.len() <= 256,
        "executor refresh family bound exceeded"
    );
    Ok(families)
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

fn unit_keys(graph: &UnitGraph, root: &Path) -> Result<BTreeMap<usize, String>> {
    fn visit(
        index: usize,
        graph: &UnitGraph,
        root: &Path,
        active: &mut BTreeSet<usize>,
        keys: &mut BTreeMap<usize, String>,
    ) -> Result<String> {
        if let Some(key) = keys.get(&index) {
            return Ok(key.clone());
        }
        ensure!(active.insert(index), "production unit dependency cycle");
        let unit = graph.units.get(index).context("unit index outside graph")?;
        let mut dependencies = Vec::new();
        for dependency in &unit.dependencies {
            dependencies.push((
                dependency.extern_crate_name.clone(),
                visit(dependency.index, graph, root, active, keys)?,
            ));
        }
        dependencies.sort();
        let key = hash_parts(&[
            b"dependency-complete-unit",
            unit_key(unit, root)?.as_bytes(),
            &serde_json::to_vec(&dependencies)?,
        ]);
        active.remove(&index);
        keys.insert(index, key.clone());
        Ok(key)
    }
    let mut keys = BTreeMap::new();
    for index in selected_indices(graph)? {
        visit(index, graph, root, &mut BTreeSet::new(), &mut keys)?;
    }
    Ok(keys)
}

fn expand_reviews(
    options: &ProducerOptions,
    root: &Path,
    graph: &UnitGraph,
    packages: &BTreeMap<String, cargo_metadata::Package>,
) -> Result<ProducerOptions> {
    let mut expanded = options.clone();
    let sources = selected_source_state(graph, packages)?;
    let keys = unit_keys(graph, root)?;
    if options.declarations.capture_caller_environment {
        expanded
            .declared_environment
            .extend(std::env::vars().filter(|(name, _)| caller_build_environment(name)));
    }
    let mut reviews = SourceReviews::default();
    for path in &options.declarations.review_catalogs {
        let catalog: SourceReviews = serde_json::from_slice(&fs::read(absolute(root, path))?)
            .with_context(|| format!("source-bound producer reviews {}", path.display()))?;
        for (target, additions) in [
            (&mut reviews.build_scripts, catalog.build_scripts),
            (&mut reviews.proc_macros, catalog.proc_macros),
        ] {
            for (owner, review) in additions {
                ensure!(
                    target.insert(owner.clone(), review).is_none(),
                    "duplicate executable closure review {owner}"
                );
            }
        }
    }
    let host = if reviews
        .build_scripts
        .values()
        .chain(reviews.proc_macros.values())
        .any(|review| !review.platforms.is_empty())
    {
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let version = Command::new(compiler)
            .current_dir(root)
            .arg("-vV")
            .output()?;
        ensure!(
            version.status.success(),
            "reviewed platform compiler query failed"
        );
        let version = std::str::from_utf8(&version.stdout)?;
        Some(
            version
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .context("reviewed platform compiler host missing")?
                .to_owned(),
        )
    } else {
        None
    };
    for index in selected_indices(graph)? {
        let unit = &graph.units[index];
        let (reviews, declarations) = if unit.mode == "run-custom-build" {
            (
                &reviews.build_scripts,
                &mut expanded.declarations.build_scripts,
            )
        } else if unit.target.kind.iter().any(|kind| kind == "proc-macro") {
            (&reviews.proc_macros, &mut expanded.declarations.proc_macros)
        } else {
            continue;
        };
        let owner = logical_package(&unit.pkg_id, root);
        let Some(review) = reviews.get(&owner) else {
            continue;
        };
        if review
            .excluded_features
            .iter()
            .any(|feature| unit.features.contains(feature))
        {
            continue;
        }
        if !required_absence_matches(review, current_environment)? {
            continue;
        }
        if !review.platforms.is_empty()
            && !review.platforms.iter().any(|platform| {
                Some(platform.as_str()) == unit.platform.as_deref().or(host.as_deref())
            })
        {
            continue;
        }
        ensure!(
            !review.rationale.trim().is_empty(),
            "closure review {owner} lacks its I/O rationale"
        );
        let package = packages
            .get(&unit.pkg_id)
            .context("review owner absent from metadata")?;
        let package_root = package
            .manifest_path
            .as_std_path()
            .parent()
            .context("review package parent")?;
        if review.reviewed_source != reviewed_source_identity(package_root)? {
            // A stale review grants nothing; capture reports the ordinary owner
            // refusal, rather than silently approving changed executable bytes.
            continue;
        }
        if review.deployment_provenance {
            ensure!(
                unit.mode == "run-custom-build"
                    && package.name.as_str() == "pse-buildinfo"
                    && package_root.canonicalize()? == root.join("crates/pse-buildinfo"),
                "deployment provenance classification is restricted to the workspace buildinfo owner"
            );
            expanded
                .declarations
                .deployment_provenance
                .insert(owner.clone());
        }
        let closure = executable_closure_basis(graph, index, &sources, &keys)?;
        if !review.reviewed_closures.contains(&closure) {
            continue;
        }
        let mut declaration = BuildScriptInputs {
            complete: true,
            reviewed_source: Some(review.reviewed_source.clone()),
            reviewed_closures: review.reviewed_closures.clone(),
            reviewed_callers: review.reviewed_callers.clone(),
            ..Default::default()
        };
        for (base, paths) in [
            (package_root, &review.package_files),
            (root, &review.workspace_files),
        ] {
            for path in paths {
                let path = absolute(base, path);
                if path.is_dir() {
                    declaration.files.extend(package_files(&path)?);
                } else {
                    ensure!(
                        path.is_file(),
                        "reviewed input {} is absent",
                        path.display()
                    );
                    declaration.files.push(path);
                }
            }
        }
        for path in &review.workspace_absent_files {
            ensure!(
                !path.as_os_str().is_empty()
                    && path
                        .components()
                        .all(|part| matches!(part, std::path::Component::Normal(_))),
                "reviewed workspace absence must be a workspace-relative path"
            );
            let path = root.join(path);
            ensure!(
                !path_entry_exists(&path)?,
                "reviewed absent workspace input {} is present or inaccessible",
                path.display()
            );
            declaration.absent_files.push(path);
        }
        for name in &review.environment_trees {
            let prefix = std::env::var_os(name)
                .with_context(|| format!("reviewed input prefix {name} is absent"))?;
            let prefix = absolute(root, Path::new(&prefix));
            ensure!(
                prefix.is_dir(),
                "reviewed input prefix {name} is not a directory"
            );
            declaration.namespaces.push(prefix);
        }
        if !review.caller_package_files.is_empty() {
            let callers: BTreeSet<_> = selected_indices(graph)?
                .into_iter()
                .map(|index| graph.units[index].pkg_id.clone())
                .collect();
            for caller in callers {
                let package = &packages[&caller];
                let caller_root = package
                    .manifest_path
                    .as_std_path()
                    .parent()
                    .context("reviewed macro caller package root")?;
                for path in &review.caller_package_files {
                    ensure!(path.is_relative(), "reviewed caller path must be relative");
                    let path = caller_root.join(path);
                    if path.is_file() {
                        declaration.files.push(path);
                    } else {
                        ensure!(!path.exists(), "reviewed caller path is not a file");
                        declaration.absent_files.push(path);
                    }
                }
            }
        }
        if !review.pest_grammars.is_empty() {
            ensure!(
                package.name.as_str() == "pest_derive" && review.reviewed_callers.is_some(),
                "Pest grammar input review requires its source-reviewed owner and actual caller basis"
            );
            let selected_packages: BTreeSet<_> = selected_indices(graph)?
                .into_iter()
                .map(|index| graph.units[index].pkg_id.clone())
                .collect();
            for caller in selected_packages {
                let caller_id = logical_package(&caller, root);
                let Some(grammars) = review.pest_grammars.get(&caller_id) else {
                    continue;
                };
                let caller_root = packages[&caller]
                    .manifest_path
                    .as_std_path()
                    .parent()
                    .context("Pest caller package root")?;
                for grammar in grammars {
                    ensure!(
                        !grammar.as_os_str().is_empty()
                            && grammar
                                .components()
                                .all(|part| matches!(part, std::path::Component::Normal(_))),
                        "reviewed Pest grammar must be a package-relative path"
                    );
                    let input = PestGrammarInput {
                        caller: caller_id.clone(),
                        preferred: caller_root.join(grammar),
                        fallback: caller_root.join("src").join(grammar),
                    };
                    for path in [&input.preferred, &input.fallback] {
                        if path.is_file() {
                            declaration.files.push(path.clone());
                        } else {
                            ensure!(
                                !path.try_exists()?,
                                "reviewed Pest grammar candidate is not a file"
                            );
                            declaration.absent_files.push(path.clone());
                        }
                    }
                    declaration.pest_grammars.push(input);
                }
            }
        }
        for name in review.environment.iter().chain(&review.environment_trees) {
            match current_environment(name)? {
                Some(value) => {
                    declaration.environment.insert(name.clone(), value);
                }
                None => declaration.absent_environment.push(name.clone()),
            }
        }
        declaration
            .absent_environment
            .extend(review.required_absent_environment.iter().cloned());
        declaration.files.sort();
        declaration.files.dedup();
        declaration.absent_environment.sort();
        declaration.absent_environment.dedup();
        declaration.absent_files.sort();
        declaration.absent_files.dedup();
        if let Some(previous) = declarations.get(&owner) {
            ensure!(
                serde_json::to_vec(previous)? == serde_json::to_vec(&declaration)?,
                "conflicting explicit executable review {owner}"
            );
        } else {
            declarations.insert(owner, declaration);
        }
    }
    Ok(expanded)
}

/// A reviewed disabled I/O branch is usable only while all its exact ambient
/// selectors remain absent. Even an empty present value may enable the branch.
fn required_absence_matches(
    review: &OwnerReview,
    environment: impl Fn(&str) -> Result<Option<String>>,
) -> Result<bool> {
    for name in &review.required_absent_environment {
        if environment(name)?.is_some() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn executable_closure_basis(
    graph: &UnitGraph,
    index: usize,
    sources: &BTreeMap<String, String>,
    keys: &BTreeMap<usize, String>,
) -> Result<String> {
    let mut pending = vec![index];
    let mut visited = BTreeSet::new();
    let mut closure = Vec::new();
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        let unit = graph
            .units
            .get(index)
            .context("review closure unit missing")?;
        closure.push((
            keys.get(&index).context("review closure key missing")?,
            sources
                .get(&unit.pkg_id)
                .context("review closure source missing")?,
        ));
        pending.extend(unit.dependencies.iter().map(|edge| edge.index));
    }
    closure.sort();
    closure.dedup();
    Ok(hash_parts(&[
        b"reviewed-executable-closure.v1",
        &serde_json::to_vec(&closure)?,
    ]))
}

/// Review basis for a macro's selected callers, derived from actual compiler
/// consumption rather than a workspace inventory or declared dep-info. Reverse
/// reachability covers macros that emit other macro invocations and their callers.
/// Raw bytes preserve invocation arguments and generated includes; unit keys bind
/// features/configuration. The basis is reported separately from scientific inputs.
fn macro_caller_input_basis(
    graph: &UnitGraph,
    macro_index: usize,
    keys: &BTreeMap<usize, String>,
    associated: &BTreeMap<usize, Vec<PathBuf>>,
    actual_associated: &BTreeSet<usize>,
    root: &Path,
    declaration: Option<&BuildScriptInputs>,
) -> Result<Option<String>> {
    let mut callers = BTreeSet::from([macro_index]);
    loop {
        let previous = callers.len();
        for index in selected_indices(graph)? {
            if graph.units[index]
                .dependencies
                .iter()
                .any(|edge| callers.contains(&edge.index))
            {
                callers.insert(index);
            }
        }
        if previous == callers.len() {
            break;
        }
    }
    callers.remove(&macro_index);
    // Another macro/helper can generate arguments in a caller. Conservatively
    // include its consumed compilation closure too; otherwise changing that
    // helper could introduce a new environment name under unchanged caller text.
    let mut pending: Vec<_> = callers.iter().copied().collect();
    while let Some(index) = pending.pop() {
        for edge in &graph.units[index].dependencies {
            if callers.insert(edge.index) {
                pending.push(edge.index);
            }
        }
    }
    let mut closure = BTreeMap::new();
    for index in callers {
        let unit = &graph.units[index];
        if unit.mode != "build" {
            continue;
        }
        if !actual_associated.contains(&index) {
            return Ok(None);
        }
        let Some(paths) = associated.get(&index).filter(|paths| !paths.is_empty()) else {
            return Ok(None);
        };
        let mut consumed = BTreeMap::new();
        for path in paths {
            let text = fs::read_to_string(absolute(root, path))?;
            for input in parse_dep_info(&text)? {
                let path = absolute(root, &input);
                let name = path.strip_prefix(root).map_or_else(
                    |_| path.to_string_lossy().into_owned(),
                    |relative| format!("workspace/{}", relative.display()),
                );
                consumed.insert(
                    name,
                    hash_parts(&[b"raw-macro-caller-input", &fs::read(path)?]),
                );
            }
            for line in text.lines().filter(|line| line.starts_with("# env-dep:")) {
                let name = line
                    .strip_prefix("# env-dep:")
                    .unwrap()
                    .split('=')
                    .next()
                    .unwrap();
                consumed.insert(
                    format!("rustc-environment:{name}"),
                    hash_parts(&[b"macro-caller-env-dep", line.as_bytes()]),
                );
            }
        }
        closure.insert(
            keys.get(&index).context("macro caller unit key missing")?,
            consumed,
        );
    }
    // No callers is a real selected-graph exclusion, not a blanket source review.
    let mut grammar_candidates = BTreeMap::new();
    if let Some(declaration) = declaration {
        for grammar in &declaration.pest_grammars {
            for path in [&grammar.preferred, &grammar.fallback] {
                let name = path.strip_prefix(root).map_or_else(
                    |_| path.to_string_lossy().into_owned(),
                    |relative| format!("workspace/{}", relative.display()),
                );
                let bytes = if path.try_exists()? {
                    Some(fs::read(path)?)
                } else {
                    None
                };
                grammar_candidates.insert(
                    name,
                    hash_parts(&[b"reviewed-pest-candidate", &serde_json::to_vec(&bytes)?]),
                );
            }
        }
    }
    Ok(Some(hash_parts(&[
        b"reviewed-macro-caller-inputs.v1",
        &serde_json::to_vec(&closure)?,
        &serde_json::to_vec(&grammar_candidates)?,
    ])))
}

/// Pest emits include_str! for the grammar it actually read. A reviewed current
/// candidate cannot certify a fresh-looking artifact compiled from the fallback
/// before a newly present preferred grammar started shadowing it.
fn pest_grammars_match_compilation(
    declaration: &BuildScriptInputs,
    graph: &UnitGraph,
    associated: &BTreeMap<usize, Vec<PathBuf>>,
    actual_associated: &BTreeSet<usize>,
    root: &Path,
) -> Result<bool> {
    for grammar in &declaration.pest_grammars {
        let chosen = if grammar.preferred.try_exists()? {
            &grammar.preferred
        } else {
            &grammar.fallback
        };
        if !chosen.is_file() {
            return Ok(false);
        }
        let chosen = chosen.canonicalize()?;
        let mut matched = false;
        for index in actual_associated {
            let unit = &graph.units[*index];
            if unit.mode != "build" || logical_package(&unit.pkg_id, root) != grammar.caller {
                continue;
            }
            for dep_info in associated.get(index).into_iter().flatten() {
                let text = fs::read_to_string(absolute(root, dep_info))?;
                for input in parse_dep_info(&text)? {
                    if absolute(root, &input).canonicalize()? == chosen {
                        matched = true;
                    }
                }
            }
        }
        if !matched {
            return Ok(false);
        }
    }
    Ok(true)
}

fn reviewed_input_state(
    options: &ProducerOptions,
    root: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut state = BTreeMap::new();
    state.insert(
        "native:review-basis".into(),
        native_review_basis(options, root)?,
    );
    for path in options
        .native_inputs
        .iter()
        .chain(&options.declared_inputs)
        .chain(
            options
                .declarations
                .configuration_executors
                .iter()
                .flat_map(|review| &review.files),
        )
    {
        let path = absolute(root, path);
        state.insert(
            path.to_string_lossy().into_owned(),
            hash_parts(&[b"explicit-closure-input", &fs::read(path)?]),
        );
    }
    for review in &options.declarations.configuration_executors {
        for path in &review.absent_files {
            let path = absolute(root, path);
            state.insert(
                path.to_string_lossy().into_owned(),
                hash_parts(&[
                    b"reviewed-file-absence",
                    &[u8::from(path_entry_exists(&path)?)],
                ]),
            );
        }
        for name in review.environment.keys() {
            state.insert(
                format!("configuration:environment:{name}"),
                hash_parts(&[
                    b"reviewed-environment",
                    &serde_json::to_vec(&current_environment(name)?)?,
                ]),
            );
        }
    }
    for path in &options.declarations.review_catalogs {
        let path = absolute(root, path);
        state.insert(
            path.to_string_lossy().into_owned(),
            hash_parts(&[b"review-catalog", &fs::read(path)?]),
        );
    }
    for (owner, declaration) in options
        .declarations
        .build_scripts
        .iter()
        .chain(&options.declarations.proc_macros)
    {
        for path in &declaration.files {
            let path = absolute(root, path);
            state.insert(
                path.to_string_lossy().into_owned(),
                hash_parts(&[b"reviewed-input", &fs::read(path)?]),
            );
        }
        for path in &declaration.absent_files {
            let path = absolute(root, path);
            state.insert(
                path.to_string_lossy().into_owned(),
                hash_parts(&[
                    b"reviewed-file-absence",
                    &[u8::from(path_entry_exists(&path)?)],
                ]),
            );
        }
        for path in &declaration.namespaces {
            let path = absolute(root, path);
            state.insert(
                format!("{owner}:namespace:{}", path.display()),
                namespace_identity(&path)?,
            );
        }
        for name in declaration
            .environment
            .keys()
            .chain(&declaration.absent_environment)
        {
            state.insert(
                format!("{owner}:environment:{name}"),
                hash_parts(&[
                    b"reviewed-environment",
                    &serde_json::to_vec(&current_environment(name)?)?,
                ]),
            );
        }
    }
    Ok(state)
}

fn native_review_basis(options: &ProducerOptions, root: &Path) -> Result<String> {
    let mut files = Vec::new();
    for path in &options.native_inputs {
        let path = absolute(root, path);
        let name = path.strip_prefix(root).map_or_else(
            |_| path.to_string_lossy().into_owned(),
            |relative| format!("workspace/{}", relative.display()),
        );
        files.push((name, hash_parts(&[b"raw-native-input", &fs::read(path)?])));
    }
    files.sort();
    files.dedup();
    let mut absent = Vec::new();
    for path in &options.declarations.native_absent_files {
        let path = absolute(root, path);
        absent.push((path.to_string_lossy().into_owned(), path.try_exists()?));
    }
    absent.sort();
    absent.dedup();
    let environment = options
        .declarations
        .native_environment
        .keys()
        .map(|name| Ok((name.clone(), current_environment(name)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut features = options.features.clone();
    features.sort();
    features.dedup();
    Ok(hash_parts(&[
        b"reviewed-native-closure.v1",
        &serde_json::to_vec(&(
            &options.package,
            &options.profile,
            &options.target,
            &options.production_target,
            features,
            options.no_default_features,
            &options.declarations.native_abi,
            files,
            absent,
            environment,
            native_environment_prefix_state(options)?,
            native_namespace_state(options, root)?,
        ))?,
    ]))
}

fn native_namespace_state(
    options: &ProducerOptions,
    root: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut state = BTreeMap::new();
    let mut aliases = BTreeMap::<(u64, u64), BTreeSet<PathBuf>>::new();
    for path in &options.declarations.native_namespaces {
        let path = absolute(root, path);
        let observation = namespace_observation(&path, 262_144)?;
        state.insert(path.to_string_lossy().into_owned(), observation.identity);
        for (identity, paths) in observation.aliases {
            aliases.entry(identity).or_default().extend(paths);
        }
    }
    // Header identity matters across distinct include roots too. Normalize the
    // path equivalence classes; raw device/inode numbers never enter a key.
    state.insert(
        "file-identity-aliases".into(),
        namespace_alias_identity(&aliases)?,
    );
    Ok(state)
}

fn native_environment_prefix_state(options: &ProducerOptions) -> Result<BTreeMap<String, bool>> {
    let names: Vec<_> = std::env::vars_os().map(|(name, _)| name).collect();
    options
        .declarations
        .native_absent_environment_prefixes
        .iter()
        .map(|prefix| {
            ensure!(
                !prefix.is_empty() && !prefix.contains(['=', '\0']),
                "invalid absent environment prefix"
            );
            Ok((prefix.clone(), environment_prefix_present(prefix, &names)))
        })
        .collect()
}

fn environment_prefix_present(prefix: &str, names: &[std::ffi::OsString]) -> bool {
    names
        .iter()
        .any(|name| name.to_string_lossy().starts_with(prefix))
}

fn namespace_alias_identity(aliases: &BTreeMap<(u64, u64), BTreeSet<PathBuf>>) -> Result<String> {
    let mut groups: Vec<_> = aliases.values().filter(|paths| paths.len() > 1).collect();
    groups.sort();
    Ok(hash_parts(&[
        b"native-file-alias-equivalence.v1",
        &serde_json::to_vec(&groups)?,
    ]))
}

struct NamespaceObservation {
    identity: String,
    aliases: BTreeMap<(u64, u64), BTreeSet<PathBuf>>,
}

/// Exact bounded search namespace evidence. Unlike Rust package source traversal,
/// this includes every name, modes, dangling links and directory link targets.
/// Following each canonical directory once closes cycles without omitting topology.
fn namespace_identity(root: &Path) -> Result<String> {
    namespace_identity_with_limit(root, 262_144)
}

fn namespace_identity_with_limit(root: &Path, entry_limit: usize) -> Result<String> {
    Ok(namespace_observation(root, entry_limit)?.identity)
}

fn namespace_observation(root: &Path, entry_limit: usize) -> Result<NamespaceObservation> {
    use std::io::Read;
    let mut pending = vec![root.to_path_buf()];
    let mut visited = BTreeSet::new();
    let mut entries = BTreeMap::new();
    let mut content_extent = 0u64;
    let mut aliases = BTreeMap::<(u64, u64), BTreeSet<PathBuf>>::new();
    while let Some(path) = pending.pop() {
        ensure!(
            entries.len() < entry_limit,
            "native namespace exceeds its finite entry bound"
        );
        let label = path.strip_prefix(root)?.to_string_lossy().into_owned();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                entries.insert(label, hash_parts(&[b"namespace-absence"]));
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::MetadataExt;
            serde_json::to_vec(&(metadata.mode(), metadata.uid(), metadata.gid()))?
        };
        #[cfg(not(unix))]
        let mode = serde_json::to_vec(&metadata.permissions().readonly())?;
        let link = if metadata.is_symlink() {
            Some(fs::read_link(&path)?)
        } else {
            None
        };
        let target = match fs::metadata(&path) {
            Ok(target) => Some(target),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        #[cfg(unix)]
        let target_mode = {
            use std::os::unix::fs::MetadataExt;
            serde_json::to_vec(
                &target
                    .as_ref()
                    .map(|metadata| (metadata.mode(), metadata.uid(), metadata.gid())),
            )?
        };
        #[cfg(not(unix))]
        let target_mode = serde_json::to_vec(
            &target
                .as_ref()
                .map(|metadata| metadata.permissions().readonly()),
        )?;
        let mut content = Vec::new();
        let kind = if target.as_ref().is_some_and(fs::Metadata::is_file) {
            let extent = target.as_ref().context("namespace file metadata")?.len();
            content_extent = content_extent
                .checked_add(extent)
                .context("native namespace extent overflow")?;
            ensure!(
                content_extent <= 16 * 1024 * 1024 * 1024,
                "native namespace exceeds its finite 16 GiB content bound"
            );
            let mut file = fs::File::open(&path)?;
            let mut hash = EncodingHasher::new();
            let mut buffer = [0u8; 65_536];
            let mut read_extent = 0u64;
            loop {
                let read = file.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                read_extent = read_extent
                    .checked_add(read as u64)
                    .context("native namespace read overflow")?;
                ensure!(
                    read_extent <= extent,
                    "native namespace file grew during capture"
                );
                hash.update(&buffer[..read]);
            }
            ensure!(
                read_extent == extent,
                "native namespace file changed during capture"
            );
            content.extend_from_slice(&extent.to_le_bytes());
            content.extend_from_slice(hash.finish().0.as_bytes());
            let canonical = path.canonicalize()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let metadata = target.as_ref().context("namespace file metadata")?;
                if metadata.nlink() > 1 {
                    aliases
                        .entry((metadata.dev(), metadata.ino()))
                        .or_default()
                        .insert(canonical.clone());
                }
            }
            content.extend_from_slice(canonical.to_string_lossy().as_bytes());
            "file"
        } else if target.as_ref().is_some_and(fs::Metadata::is_dir) {
            let canonical = path.canonicalize()?;
            content.extend_from_slice(canonical.to_string_lossy().as_bytes());
            if visited.insert(canonical) {
                let mut children = Vec::new();
                for child in fs::read_dir(&path)? {
                    // Charge listed-but-not-yet-visited entries too. Refuse
                    // before retaining an unbounded directory listing.
                    ensure!(
                        entries.len() + 1 + pending.len() + children.len() < entry_limit,
                        "native namespace exceeds its finite entry bound"
                    );
                    children.push(child?.path());
                }
                children.sort();
                pending.extend(children.into_iter().rev());
            }
            "directory"
        } else if target.is_none() && link.is_some() {
            "dangling-link"
        } else {
            anyhow::bail!(
                "native namespace contains an unsupported special file: {}",
                path.display()
            );
        };
        entries.insert(
            label,
            hash_parts(&[
                b"native-namespace-entry.v1",
                kind.as_bytes(),
                &mode,
                &target_mode,
                &serde_json::to_vec(&link)?,
                &content,
            ]),
        );
    }
    Ok(NamespaceObservation {
        identity: hash_parts(&[
            b"native-namespace.v1",
            &serde_json::to_vec(&entries)?,
            namespace_alias_identity(&aliases)?.as_bytes(),
        ]),
        aliases,
    })
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

/// Cargo exposes an installed executable or shared library, which may be a hard link to rustc's
/// unit-local output. File identity supplies the association; filenames and
/// matching source text alone cannot select an output from another build.
fn executable_artifact_aliases(artifact: &Path) -> Result<Vec<PathBuf>> {
    let mut aliases = Vec::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let expected = fs::metadata(artifact)?;
        let Some(parent) = artifact.parent() else {
            return Ok(aliases);
        };
        let mut directories = vec![parent.join("deps")];
        // Both Cargo's conventional deps layout and its selected build-dir
        // layout are supported. Walk only the fixed package/unit/out levels.
        let build = parent.join("build");
        if build.is_dir() {
            for package in fs::read_dir(build)? {
                let package = package?.path();
                if !package.is_dir() {
                    continue;
                }
                for unit in fs::read_dir(package)? {
                    let out = unit?.path().join("out");
                    if out.is_dir() {
                        directories.push(out);
                    }
                    ensure!(
                        directories.len() <= 65_536,
                        "Cargo executable alias inventory exceeds its finite bound"
                    );
                }
            }
        }
        let mut examined = 0usize;
        for directory in directories
            .into_iter()
            .filter(|directory| directory.is_dir())
        {
            for entry in fs::read_dir(directory)? {
                examined += 1;
                ensure!(
                    examined <= 262_144,
                    "Cargo executable alias inventory exceeds its finite bound"
                );
                let entry = entry?;
                if !entry.file_type()?.is_file() {
                    continue;
                }
                let metadata = entry.metadata()?;
                if metadata.dev() == expected.dev() && metadata.ino() == expected.ino() {
                    aliases.push(entry.path());
                }
            }
        }
    }
    aliases.sort();
    aliases.dedup();
    Ok(aliases)
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
    // Cargo messages do not carry unit dependency edges. Indistinguishable units
    // with the same compilation context conservatively consume every matching
    // artifact's dep-info. Their dependency-complete keys keep their edge meanings
    // separate; no claim is made about which output belongs to which edge set.
    // Different platforms remain ambiguous because their build environments differ.
    if matching.iter().any(|message| {
        graph
            .units
            .iter()
            .filter(|candidate| artifact_matches(candidate, message))
            .any(|candidate| {
                candidate.platform != unit.platform || candidate.profile != unit.profile
            })
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
            let mut artifacts = vec![artifact.clone()];
            if unit.target.kind == ["bin"]
                || unit.target.crate_types.iter().any(|kind| kind == "cdylib")
            {
                artifacts.extend(executable_artifact_aliases(&artifact)?);
            }
            for artifact in artifacts {
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
                    // Cargo also writes aggregate final-artifact .d files containing
                    // transitive rerun hints. Only rustc's own dependency file lists
                    // itself as an output; accepting the aggregate imports unrelated
                    // provenance scans into this compilation unit's scientific key.
                    if outputs.contains(&candidate)
                        && outputs.contains(&artifact)
                        && inputs.contains(&unit.target.src_path)
                    {
                        paths.insert(candidate);
                    }
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
    let keys = unit_keys(graph, root)?;
    let sources = selected_source_state(graph, packages)?;
    let mut reasons = BTreeSet::new();
    let mut inputs: BTreeMap<String, Vec<ConsumedInput>> = BTreeMap::new();
    let mut units = Vec::new();
    let mut selected_packages = BTreeSet::new();
    let mut reviewed_owner_sources = BTreeMap::new();
    let mut deployment_provenance = BTreeMap::new();
    let mut provenance_outputs = BTreeMap::new();
    for owner in &options.declarations.deployment_provenance {
        let outputs = messages
            .iter()
            .filter(|message| {
                message.get("reason").and_then(serde_json::Value::as_str)
                    == Some("build-script-executed")
                    && message
                        .get("package_id")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|id| logical_package(id, root) == *owner)
            })
            .filter_map(|message| message.get("out_dir").and_then(serde_json::Value::as_str))
            .map(|path| absolute(root, Path::new(path)))
            .collect::<BTreeSet<_>>();
        if outputs.len() == 1 {
            provenance_outputs.insert(
                owner.clone(),
                outputs.into_iter().next().context("provenance output")?,
            );
        } else {
            reasons.insert(format!(
                "{owner}: deployment provenance lacks one actual Cargo output association"
            ));
        }
    }
    let package_counts = selected.iter().fold(BTreeMap::new(), |mut counts, index| {
        *counts
            .entry(graph.units[*index].pkg_id.clone())
            .or_insert(0_usize) += 1;
        counts
    });
    let mut associated = BTreeMap::<usize, Vec<PathBuf>>::new();
    let mut actual_associated = BTreeSet::new();
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
        if let Some(paths) = artifact_dep_info(unit, graph, messages, root)? {
            actual_associated.insert(*index);
            associated.insert(*index, paths);
        } else if let Some(paths) = declared {
            // Manual inventories are useful for explicit instrumentation, but
            // cannot impersonate compiler evidence for a supported deployment.
            associated.insert(*index, paths.clone());
            if unit.mode == "build" {
                reasons.insert(format!("unit {key}: manual dep-info cannot qualify production without actual Cargo artifact association"));
            }
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
        let closure = if unit.mode == "run-custom-build"
            || unit.target.kind.iter().any(|kind| kind == "proc-macro")
        {
            let closure = executable_closure_basis(graph, *index, &sources, &keys)?;
            reviewed_owner_sources.insert(
                format!("executable-closure:{package_id}:{key}"),
                closure.clone(),
            );
            Some(closure)
        } else {
            None
        };
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
            let requires_callers = matches!(package.name.as_str(), "paste" | "pest_derive")
                || declaration.is_some_and(|declaration| declaration.reviewed_callers.is_some());
            let caller_basis = if requires_callers {
                macro_caller_input_basis(
                    graph,
                    *index,
                    &keys,
                    &associated,
                    &actual_associated,
                    root,
                    declaration,
                )?
            } else {
                None
            };
            if let Some(basis) = &caller_basis {
                reviewed_owner_sources.insert(
                    format!("macro-caller-inputs:{package_id}:{key}"),
                    basis.clone(),
                );
            }
            if requires_callers
                && !declaration.is_some_and(|declaration| {
                    caller_basis.as_ref().is_some_and(|basis| {
                        declaration
                            .reviewed_callers
                            .as_ref()
                            .is_some_and(|reviews| reviews.contains(basis))
                    })
                })
            {
                reasons.insert(format!(
                    "{package_id}: dynamic macro I/O lacks a current actual caller-input review"
                ));
            }
            let grammar_matches = if package.name.as_str() == "pest_derive" {
                if let Some(declaration) =
                    declaration.filter(|declaration| !declaration.pest_grammars.is_empty())
                {
                    pest_grammars_match_compilation(
                        declaration,
                        graph,
                        &associated,
                        &actual_associated,
                        root,
                    )?
                } else {
                    false
                }
            } else {
                true
            };
            if !grammar_matches {
                reasons.insert(format!(
                    "{package_id}: chosen Pest grammar lacks actual compiler consumption"
                ));
            }
            if let Some(declaration) = declaration.filter(|declaration| {
                review_matches(declaration, &reviewed_owner_sources[&package_id])
                    && grammar_matches
                    && closure
                        .as_ref()
                        .is_some_and(|closure| declaration.reviewed_closures.contains(closure))
                    && (!requires_callers
                        || caller_basis.as_ref().is_some_and(|basis| {
                            declaration
                                .reviewed_callers
                                .as_ref()
                                .is_some_and(|reviews| reviews.contains(basis))
                        }))
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
                let mut captured = BTreeMap::new();
                add_dep_info(
                    &mut captured,
                    &mut reasons,
                    &group,
                    dep_info,
                    root,
                    package_root,
                    &package_id,
                )?;
                if let Some(out) = provenance_outputs.get(&package_id) {
                    classify_deployment_inputs(&mut captured, &mut deployment_provenance, out);
                }
                merge_inputs(&mut inputs, captured);
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
                    && closure
                        .as_ref()
                        .is_some_and(|closure| declaration.reviewed_closures.contains(closure))
            }) {
                let mut captured = BTreeMap::new();
                add_reviewed_inputs(
                    &mut captured,
                    &mut reasons,
                    "build-script-inputs",
                    declaration,
                    root,
                    &package_id,
                )?;
                // The reviewed buildinfo script reads external data solely to
                // report outer provenance. Its own compiled sources/helpers and
                // selected compiler/config/native inputs remain independently
                // bound in the ordinary capture groups above and below.
                if provenance_outputs.contains_key(&package_id) {
                    merge_inputs(&mut deployment_provenance, captured);
                } else {
                    merge_inputs(&mut inputs, captured);
                }
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
        let owner = logical_package(id, root);
        let mut env = message
            .get("env")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        if provenance_outputs.contains_key(&owner)
            && let Some(values) = env.as_array_mut()
        {
            let mut outer = Vec::new();
            values.retain(|value| {
                if value
                    .get(0)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(provenance_environment)
                {
                    outer.push(value.clone());
                    false
                } else {
                    true
                }
            });
            deployment_provenance
                .entry("actual-build-script-provenance".into())
                .or_insert_with(Vec::new)
                .push(ConsumedInput {
                    path: owner.clone(),
                    identity: hash_parts(&[
                        b"build-script-provenance",
                        &serde_json::to_vec(&outer)?,
                    ]),
                    representation: "actual-cargo-message".into(),
                });
        }
        let payload = serde_json::to_vec(
            &serde_json::json!({"linked_libs":message.get("linked_libs"),"linked_paths":message.get("linked_paths"),"cfgs":message.get("cfgs"),"env":env}),
        )?;
        inputs
            .entry("actual-build-script-output".into())
            .or_default()
            .push(ConsumedInput {
                path: owner,
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
    for (path, identity) in native_namespace_state(options, root)? {
        inputs
            .entry("native-namespaces".into())
            .or_default()
            .push(ConsumedInput {
                path,
                identity,
                representation: "bounded-raw-search-namespace.v1".into(),
            });
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
    let native_basis = native_review_basis(options, root)?;
    reviewed_owner_sources.insert("native-closure".into(), native_basis.clone());
    if options.declarations.native_reviewed_source.as_ref() != Some(&native_basis) {
        reasons.insert("native/tool closure has no matching source-bound review".into());
    }
    for (name, expected) in &options.declarations.native_environment {
        let actual = current_environment(name)?;
        if &actual != expected {
            reasons.insert(format!("reviewed native environment {name} changed"));
        }
        inputs
            .entry("native-environment".into())
            .or_default()
            .push(ConsumedInput {
                path: name.clone(),
                identity: hash_parts(&[b"native-environment", &serde_json::to_vec(&actual)?]),
                representation: "actual-value-digest".into(),
            });
    }
    for (prefix, present) in native_environment_prefix_state(options)? {
        if present {
            reasons.insert(format!(
                "reviewed absent native environment prefix {prefix} is present"
            ));
        }
        inputs
            .entry("native-environment-prefix-absence".into())
            .or_default()
            .push(ConsumedInput {
                path: prefix,
                identity: hash_parts(&[b"native-environment-prefix-absence", &[u8::from(present)]]),
                representation: "actual-family-presence".into(),
            });
    }
    for path in &options.declarations.native_absent_files {
        let path = absolute(root, path);
        let present = path.try_exists()?;
        if present {
            reasons.insert(format!(
                "reviewed absent native file {} is present",
                path.display()
            ));
        }
        inputs
            .entry("native-file-absence".into())
            .or_default()
            .push(ConsumedInput {
                path: path.to_string_lossy().into_owned(),
                identity: hash_parts(&[b"native-file-absence", &[u8::from(present)]]),
                representation: "actual-presence".into(),
            });
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
    for values in deployment_provenance.values_mut() {
        values.sort();
        values.dedup();
    }
    let outer_attestation = provenance_outputs
        .values()
        .map(|out| {
            let source = fs::read(out.join("source.identity"))?;
            let build = fs::read(out.join("build.identity"))?;
            for (name, bytes) in [("source.identity", &source), ("build.identity", &build)] {
                let path = out.join(name).to_string_lossy().into_owned();
                let expected = hash_parts(&[b"input", b"raw", bytes]);
                ensure!(
                    deployment_provenance
                        .values()
                        .flatten()
                        .any(|input| input.path == path && input.identity == expected),
                    "actual {name} provenance changed or lacks compiler consumption evidence"
                );
            }
            Ok(OuterAttestation {
                source: pse_ids::ContentHash::from_bytes(
                    source
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("source provenance must be 32 bytes"))?,
                ),
                build: pse_ids::ContentHash::from_bytes(
                    build
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("build provenance must be 32 bytes"))?,
                ),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        outer_attestation.len() <= 1,
        "ambiguous deployment provenance owner"
    );
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
        outer_attestation: outer_attestation.into_iter().next(),
        package: options.package.clone(),
        selected_root: Some(keys[&graph.roots[0]].clone()),
        identity: String::new(),
        persistent_reuse_eligible: reasons.is_empty(),
        reasons: reasons.into_iter().collect(),
        units,
        selected_lock_records,
        consumed_inputs: inputs,
        deployment_provenance,
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

fn path_entry_exists(path: &Path) -> std::io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
fn add_reviewed_inputs(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    reasons: &mut BTreeSet<String>,
    group: &str,
    declaration: &BuildScriptInputs,
    root: &Path,
    package: &str,
) -> Result<()> {
    add_reviewed_inputs_with_environment(
        inputs,
        reasons,
        group,
        declaration,
        root,
        package,
        current_environment,
    )
}

fn add_reviewed_inputs_with_environment(
    inputs: &mut BTreeMap<String, Vec<ConsumedInput>>,
    reasons: &mut BTreeSet<String>,
    group: &str,
    declaration: &BuildScriptInputs,
    root: &Path,
    package: &str,
    environment: impl Fn(&str) -> Result<Option<String>>,
) -> Result<()> {
    for (name, expected) in &declaration.environment {
        if environment(name)?.as_ref() != Some(expected) {
            reasons.insert(format!("{package}: reviewed environment {name} changed"));
        }
    }
    for name in &declaration.absent_environment {
        if environment(name)?.is_some() {
            reasons.insert(format!(
                "{package}: reviewed absent environment {name} is present"
            ));
        }
    }
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
    for path in &declaration.absent_files {
        let path = absolute(root, path);
        if path_entry_exists(&path)? {
            reasons.insert(format!(
                "{package}: reviewed absent file {} is present",
                path.display()
            ));
        }
        inputs
            .entry(format!("{group}-absence"))
            .or_default()
            .push(ConsumedInput {
                path: path.to_string_lossy().into_owned(),
                identity: hash_parts(&[b"reviewed-file-absence"]),
                representation: "reviewed-absence".into(),
            });
    }
    for path in &declaration.namespaces {
        let path = absolute(root, path);
        inputs
            .entry(format!("{group}-namespaces"))
            .or_default()
            .push(ConsumedInput {
                path: path.to_string_lossy().into_owned(),
                identity: namespace_identity(&path)?,
                representation: "bounded-raw-search-namespace.v1".into(),
            });
    }
    let values = serde_json::to_vec(&(&declaration.environment, &declaration.absent_environment))?;
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

fn configuration_closure_basis(
    declaration: &ConfigurationExecutorInputs,
    root: &Path,
) -> Result<String> {
    let mut files = Vec::new();
    for file in &declaration.files {
        let file = absolute(root, file);
        files.push((
            file.to_string_lossy().into_owned(),
            hash_parts(&[b"configuration-closure-raw-input", &fs::read(file)?]),
        ));
    }
    files.sort();
    files.dedup();
    let mut absent = declaration
        .absent_files
        .iter()
        .map(|file| absolute(root, file))
        .collect::<Vec<_>>();
    absent.sort();
    absent.dedup();
    Ok(hash_parts(&[
        b"reviewed-configuration-closure.v1",
        &serde_json::to_vec(&(files, absent, &declaration.environment))?,
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
        let wrapper_override = match selector.as_slice() {
            [table, key] if table == "build" && key == "rustc-wrapper" => Some("RUSTC_WRAPPER"),
            [table, key] if table == "build" && key == "rustc-workspace-wrapper" => {
                Some("RUSTC_WORKSPACE_WRAPPER")
            }
            _ => None,
        };
        if let Some(name) = wrapper_override {
            // Cargo explicitly interprets an empty wrapper override as disabling
            // the configured executable. Its configuration bytes remain retained.
            if environment(name)?.is_some_and(|value| value.is_empty()) {
                inputs
                    .entry("configuration-executor-exclusions".into())
                    .or_default()
                    .push(ConsumedInput {
                        path: format!("{label}:{name}"),
                        identity: hash_parts(&[
                            b"effective-empty-wrapper-override",
                            name.as_bytes(),
                        ]),
                        representation: "actual-empty-wrapper-override".into(),
                    });
                continue;
            }
        }
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
        match configuration_closure_basis(declaration, root) {
            Ok(closure) => {
                review_bases.insert(format!("cargo-executor-closure:{label}"), closure.clone());
                if declaration.reviewed_closure.as_ref() != Some(&closure) {
                    reasons.insert(format!("Cargo configuration executor {label} lacks a current transitive source-bound closure review"));
                }
            }
            Err(error) => {
                reasons.insert(format!("Cargo configuration executor {label} transitive source basis could not be captured: {error}"));
            }
        }
        if !declaration.complete || declaration.reviewed_source.as_deref() != Some(basis.as_str()) {
            reasons.insert(format!("Cargo configuration executor {label} lacks a current source-bound complete closure review"));
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
        for file in &declaration.absent_files {
            let file = absolute(root, file);
            if file.try_exists()? {
                reasons.insert(format!(
                    "Cargo configuration executor {label} reviewed absent file {} is present",
                    file.display()
                ));
            }
            inputs
                .entry("configuration-executor-input-absence".into())
                .or_default()
                .push(ConsumedInput {
                    path: file.to_string_lossy().into_owned(),
                    identity: hash_parts(&[b"reviewed-file-absence"]),
                    representation: "reviewed-absence".into(),
                });
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

fn provenance_environment(name: &str) -> bool {
    matches!(
        name,
        "PSE_RUSTC_VERSION"
            | "PSE_PROFILE"
            | "PSE_GIT_SHA"
            | "PSE_CARGO_LOCK_SHA256"
            | "PSE_UV_LOCK_SHA256"
    )
}

fn merge_inputs(
    into: &mut BTreeMap<String, Vec<ConsumedInput>>,
    from: BTreeMap<String, Vec<ConsumedInput>>,
) {
    for (group, inputs) in from {
        into.entry(group).or_default().extend(inputs);
    }
}

/// This boundary is used only for the source-reviewed buildinfo owner and its
/// one Cargo-issued OUT_DIR. Same-named inputs of another owner remain semantic.
fn classify_deployment_inputs(
    scientific: &mut BTreeMap<String, Vec<ConsumedInput>>,
    deployment: &mut BTreeMap<String, Vec<ConsumedInput>>,
    out: &Path,
) {
    let files = ["source.identity", "build.identity", "cargo.lock", "uv.lock"]
        .map(|name| out.join(name).to_string_lossy().into_owned());
    for (group, inputs) in scientific.iter_mut() {
        let mut outer = Vec::new();
        inputs.retain(|input| {
            let provenance = files.contains(&input.path)
                || group == "rustc-environment" && provenance_environment(&input.path);
            if provenance {
                outer.push(input.clone());
            }
            !provenance
        });
        if !outer.is_empty() {
            deployment.entry(group.clone()).or_default().extend(outer);
        }
    }
    scientific.retain(|_, inputs| !inputs.is_empty());
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

    #[test]
    fn producer_reviewed_disabled_executor_refuses_present_or_unknown_selector() {
        let review: OwnerReview = serde_json::from_value(serde_json::json!({
            "reviewed_source": "actual-reviewed-source", "rationale": "The reviewed debug branch launches rustfmt only when its selector is present.",
            "required_absent_environment": ["SALSA_DEBUG_MACRO"]
        })).unwrap();
        assert!(required_absence_matches(&review, |_| Ok(None)).unwrap());
        for value in ["", "tracked", "all"] {
            assert!(!required_absence_matches(&review, |_| Ok(Some(value.into()))).unwrap());
        }
        assert!(
            required_absence_matches(&review, |_| anyhow::bail!("non-Unicode selector")).is_err()
        );
    }

    #[test]
    fn producer_native_namespace_refuses_growth_before_retaining_large_listing() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["one", "two", "three"] {
            fs::write(directory.path().join(name), b"header").unwrap();
        }
        let accepted = namespace_identity_with_limit(directory.path(), 4).unwrap();
        assert_eq!(
            accepted,
            namespace_identity_with_limit(directory.path(), 5).unwrap()
        );
        fs::write(
            directory.path().join("four"),
            b"new earlier header candidate",
        )
        .unwrap();
        assert!(
            namespace_identity_with_limit(directory.path(), 4)
                .unwrap_err()
                .to_string()
                .contains("finite entry bound")
        );
        assert_ne!(
            accepted,
            namespace_identity_with_limit(directory.path(), 5).unwrap()
        );
    }

    #[test]
    fn producer_exported_shell_function_guard_checks_name_family_presence() {
        let prefix = "BASH_FUNC_";
        assert!(!environment_prefix_present(
            prefix,
            &["BASH_ENV".into(), "PATH".into()]
        ));
        assert!(environment_prefix_present(
            prefix,
            &["BASH_FUNC_ldd%%".into()]
        ));
        assert!(environment_prefix_present(
            prefix,
            &["BASH_FUNC_new_name%%".into()]
        ));
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            assert!(environment_prefix_present(
                prefix,
                &[std::ffi::OsString::from_vec(b"BASH_FUNC_\xff%%".to_vec())]
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn producer_native_namespace_tracks_header_aliases_across_include_roots() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let source = first.join("header.h");
        let destination = second.join("header.h");
        fs::write(&source, b"#pragma once\nstruct example {};\n").unwrap();
        fs::copy(&source, &destination).unwrap();
        let options = ProducerOptions {
            workspace_root: directory.path().into(),
            package: "namespace-fixture".into(),
            profile: "producer".into(),
            target: None,
            production_target: ProducerTarget::Library,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: Vec::new(),
            declared_environment: BTreeMap::new(),
            declarations: InputDeclarations {
                native_namespaces: vec![first.clone(), second.clone()],
                ..Default::default()
            },
        };
        let independent = native_namespace_state(&options, directory.path()).unwrap();
        fs::remove_file(&destination).unwrap();
        fs::hard_link(&source, &destination).unwrap();
        let shared = native_namespace_state(&options, directory.path()).unwrap();
        // Both one-file include roots still have identical names, content,
        // modes and canonical paths; only their cross-root alias relation differs.
        assert_eq!(
            independent[&first.to_string_lossy().into_owned()],
            shared[&first.to_string_lossy().into_owned()]
        );
        assert_eq!(
            independent[&second.to_string_lossy().into_owned()],
            shared[&second.to_string_lossy().into_owned()]
        );
        assert_ne!(
            independent["file-identity-aliases"],
            shared["file-identity-aliases"]
        );
        fs::remove_file(&destination).unwrap();
        fs::copy(&source, &destination).unwrap();
        assert_eq!(
            independent,
            native_namespace_state(&options, directory.path()).unwrap()
        );
    }

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
        let mut declaration = ConfigurationExecutorInputs {
            configuration: PathBuf::from(".cargo/config.toml"),
            selector,
            complete: true,
            reviewed_source: Some(bases.values().next().unwrap().clone()),
            reviewed_closure: None,
            files: vec!["linker-input".into()],
            absent_files: Vec::new(),
            environment: BTreeMap::from([
                ("REVIEWED_VALUE".into(), Some("exact".into())),
                ("REVIEWED_ABSENCE".into(), None),
            ]),
        };
        declaration.reviewed_closure =
            Some(configuration_closure_basis(&declaration, root).unwrap());
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
        let (directory, path, mut declaration) = configuration_fixture();
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
            b"changed helper implementation reads an additional undeclared input",
        )
        .unwrap();
        let (after, reasons, _) = reviewed_configuration_capture(
            root,
            &path,
            std::slice::from_ref(&declaration),
            Some("exact"),
        );
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("current transitive source-bound"))
        );
        assert_ne!(
            serde_json::to_vec(&before).unwrap(),
            serde_json::to_vec(&after).unwrap()
        );
        // An explicit renewed review accepts this fixture's known changed helper
        // contract. Capture itself never silently renews it.
        declaration.reviewed_closure =
            Some(configuration_closure_basis(&declaration, root).unwrap());
        assert!(
            reviewed_configuration_capture(
                root,
                &path,
                std::slice::from_ref(&declaration),
                Some("exact")
            )
            .1
            .is_empty()
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
    fn producer_empty_wrapper_override_excludes_only_the_disabled_executor() {
        let (directory, path, _) = configuration_fixture();
        let root = directory.path();
        let tool = root.join("reviewed-linker");
        fs::write(
            &path,
            format!(
                "[build]\nrustc-wrapper='{}'\nrustc-workspace-wrapper='{}'\n",
                tool.display(),
                tool.display()
            ),
        )
        .unwrap();
        let observe = |value: Option<&str>| {
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
                |_| Ok(value.map(str::to_owned)),
            )
            .unwrap();
            (inputs, reasons)
        };
        let (disabled, reasons) = observe(Some(""));
        assert!(reasons.is_empty());
        assert!(disabled.contains_key("cargo-configuration"));
        assert_eq!(disabled["configuration-executor-exclusions"].len(), 2);
        assert!(!disabled.contains_key("configuration-executables"));
        for value in [None, Some("unreviewed-wrapper")] {
            let (inputs, reasons) = observe(value);
            assert_eq!(reasons.len(), 2);
            assert!(
                reasons
                    .iter()
                    .all(|reason| reason.contains("lacks one exact closure review"))
            );
            assert!(!inputs.contains_key("configuration-executor-exclusions"));
        }
        // No empty wrapper environment can suppress a configured linker.
        let (link_directory, link_path, _) = configuration_fixture();
        let (_, reasons, _) =
            reviewed_configuration_capture(link_directory.path(), &link_path, &[], Some(""));
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("lacks one exact closure review"))
        );
    }

    #[test]
    fn producer_native_namespace_tracks_shadow_candidates_content_and_modes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let preferred = root.join("preferred");
        let fallback = root.join("fallback");
        fs::create_dir(&preferred).unwrap();
        fs::create_dir(&fallback).unwrap();
        fs::write(fallback.join("science.h"), "selected fallback").unwrap();
        let before = namespace_identity(root).unwrap();
        assert_eq!(before, namespace_identity(root).unwrap());
        // The previously absent earlier search candidate changes meaning even
        // though every originally consumed file still has the same bytes.
        fs::write(preferred.join("science.h"), "new shadow candidate").unwrap();
        let added = namespace_identity(root).unwrap();
        assert_ne!(before, added);
        fs::write(preferred.join("science.h"), "changed candidate").unwrap();
        let content_changed = namespace_identity(root).unwrap();
        assert_ne!(added, content_changed);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(preferred.join("science.h"))
                .unwrap()
                .permissions()
                .mode();
            fs::set_permissions(
                preferred.join("science.h"),
                fs::Permissions::from_mode(mode ^ 0o040),
            )
            .unwrap();
            assert_ne!(content_changed, namespace_identity(root).unwrap());
        }
        fs::remove_file(preferred.join("science.h")).unwrap();
        assert_eq!(before, namespace_identity(root).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn producer_native_namespace_tracks_directory_links_and_missing_search_roots() {
        use std::os::unix::fs::symlink;
        let directory = tempfile::tempdir().unwrap();
        let roots = directory.path();
        let namespace = roots.join("search");
        fs::create_dir(&namespace).unwrap();
        for target in ["first", "second"] {
            fs::create_dir(roots.join(target)).unwrap();
            fs::write(roots.join(target).join("science.h"), "same header bytes").unwrap();
        }
        let alias = namespace.join("selected");
        symlink("../first", &alias).unwrap();
        let first = namespace_identity(&namespace).unwrap();
        fs::remove_file(&alias).unwrap();
        symlink("../second", &alias).unwrap();
        assert_ne!(first, namespace_identity(&namespace).unwrap());
        // Cyclic directory links are retained as topology and traversed once.
        symlink(".", namespace.join("cycle")).unwrap();
        assert_eq!(
            namespace_identity(&namespace).unwrap(),
            namespace_identity(&namespace).unwrap()
        );
        let absent = roots.join("missing");
        let missing = namespace_identity(&absent).unwrap();
        fs::create_dir(&absent).unwrap();
        assert_ne!(missing, namespace_identity(&absent).unwrap());
    }

    #[test]
    fn producer_actual_cargo_consumed_edit_rekeys_uncompiled_test_edit_does_not() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("tests")).unwrap();
        fs::create_dir(root.join("src/bin")).unwrap();
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
            root.join("src/bin/worker.rs"),
            "fn main() {let _ = producer_fixture::value();}",
        )
        .unwrap();
        fs::write(root.join("src/bin/other.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("build.rs"), r#"fn main() {
            let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
            let ambient = std::fs::read_to_string("ambient-header-name").unwrap();
            std::fs::write(out.join("observed-object.d"), format!("observed-object: {ambient}\n")).unwrap();
            println!("cargo:rerun-if-changed=build.rs");
        }"#).unwrap();
        fs::write(root.join("ambient-header-name"), "selected-header").unwrap();
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
            production_target: ProducerTarget::Library,
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
        let evidence_directory = tempfile::tempdir().unwrap();
        let evidence_path = evidence_directory.path().join("build.json");
        let mut instrumented = options.clone();
        instrumented.declarations.actual_build_evidence = Some(evidence_path.clone());
        let observed = run(&instrumented).unwrap();
        assert_eq!(observed.identity, first.identity);
        let evidence: serde_json::Value =
            serde_json::from_slice(&fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(evidence["success"], true);
        let observed_outputs = evidence["selected_build_outputs"].as_object().unwrap();
        assert!(
            observed_outputs
                .iter()
                .any(|(path, bytes)| path.ends_with("observed-object.d")
                    && bytes.as_str() == Some("observed-object: selected-header\n"))
        );
        assert!(
            observed_outputs
                .keys()
                .any(|path| path.ends_with("stdout") || path.ends_with("output"))
        );
        assert!(
            evidence["native_caller_environment"]
                .as_object()
                .unwrap()
                .contains_key("PYO3_CONFIG_FILE")
        );
        assert!(
            evidence["arguments"]
                .as_array()
                .unwrap()
                .iter()
                .any(|argument| argument == "--lib")
        );
        let messages =
            parse_cargo_messages(evidence["stdout"].as_str().unwrap().as_bytes()).unwrap();
        assert!(
            messages
                .iter()
                .any(|message| message["reason"] == "compiler-artifact")
        );
        let recorded: UnitGraph = serde_json::from_value(evidence["unit_graph"].clone()).unwrap();
        assert_eq!(
            recorded.units[recorded.roots[0]].target.name,
            "producer_fixture"
        );
        assert_eq!(
            first.selected_root.as_ref(),
            unit_keys(&recorded, root).unwrap().get(&recorded.roots[0])
        );
        // A new arbitrary-input review cannot rely on CargoFresh output from a
        // preceding unguarded invocation whose script omitted a rerun hint.
        fs::write(root.join("ambient-header-name"), "changed-header").unwrap();
        run(&instrumented).unwrap();
        let stale: serde_json::Value =
            serde_json::from_slice(&fs::read(&evidence_path).unwrap()).unwrap();
        assert!(
            stale["selected_build_outputs"]
                .as_object()
                .unwrap()
                .values()
                .any(|bytes| bytes.as_str() == Some("observed-object: selected-header\n"))
        );
        instrumented.declarations.refresh_executors = true;
        run(&instrumented).unwrap();
        let refreshed: serde_json::Value =
            serde_json::from_slice(&fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            refreshed["refreshed_executor_packages"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            refreshed["executor_refresh_diagnostics"]["package_name_families"],
            serde_json::json!(["producer-fixture"])
        );
        assert!(
            !refreshed["executor_refresh_diagnostics"]["stderr"]
                .as_str()
                .unwrap()
                .contains("warning:")
        );
        assert!(
            refreshed["selected_build_outputs"]
                .as_object()
                .unwrap()
                .values()
                .any(|bytes| bytes.as_str() == Some("observed-object: changed-header\n"))
        );
        instrumented.declarations.refresh_executors = false;
        instrumented.declarations.capture_actual_build = false;
        assert!(
            run(&instrumented)
                .unwrap_err()
                .to_string()
                .contains("requires an actual selected production build")
        );
        assert!(
            first
                .units
                .iter()
                .all(|unit| !unit.target_kind.iter().any(|kind| kind == "bin"))
        );
        assert!(
            !first
                .reasons
                .iter()
                .any(|reason| reason.contains("no associated actual rustc dep-info")),
            "{:?}",
            first.reasons
        );
        let manual = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            manual.path(),
            format!("manual-output: {}\n", root.join("src/lib.rs").display()),
        )
        .unwrap();
        let mut manual_options = options.clone();
        manual_options.declarations.capture_actual_build = false;
        manual_options.declarations.dep_info = first
            .units
            .iter()
            .filter(|unit| unit.mode == "build")
            .map(|unit| (unit.key.clone(), vec![manual.path().to_path_buf()]))
            .collect();
        let manual_receipt = run(&manual_options).unwrap();
        assert!(!manual_receipt.persistent_reuse_eligible);
        assert!(
            manual_receipt
                .reasons
                .iter()
                .any(|reason| reason.contains("manual dep-info cannot qualify production"))
        );
        manual_options.declarations.capture_actual_build = true;
        assert_eq!(
            first.identity,
            run(&manual_options).unwrap().identity,
            "manual inventory cannot replace the actual selected compiler inputs"
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
        let mut worker_options = options.clone();
        worker_options.production_target = ProducerTarget::Binary("worker".into());
        let worker = run(&worker_options).unwrap();
        assert!(
            worker
                .reasons
                .iter()
                .all(|reason| !reason.contains("no associated actual rustc dep-info")),
            "{:?}",
            worker.reasons
        );
        assert!(
            worker
                .units
                .iter()
                .any(|unit| unit.target_name == "worker" && unit.target_kind == ["bin"])
        );
        assert!(worker.units.iter().all(|unit| unit.target_name != "other"));
        assert_ne!(worker.identity, production_changed.identity);
        fs::write(root.join("src/bin/other.rs"), "fn main() {let _ = 17;}\n").unwrap();
        assert_eq!(worker.identity, run(&worker_options).unwrap().identity);
        fs::write(
            root.join("src/bin/worker.rs"),
            "fn main() {let _ = producer_fixture::value() + 1;}",
        )
        .unwrap();
        assert_ne!(worker.identity, run(&worker_options).unwrap().identity);
        // The installed cdylib is also a Cargo alias of rustc's unit output.
        // Its own dep-info, including the helper, must remain attributable;
        // Cargo's aggregate installed-artifact .d is not sufficient evidence.
        let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            format!("{manifest}\n[lib]\ncrate-type=['cdylib']\n"),
        )
        .unwrap();
        let shared = run(&options).unwrap();
        assert!(
            shared
                .units
                .iter()
                .any(|unit| unit.crate_types == ["cdylib"])
        );
        assert!(
            shared
                .reasons
                .iter()
                .all(|reason| !reason.contains("no associated actual rustc dep-info")),
            "{:?}",
            shared.reasons
        );
        assert!(
            shared
                .consumed_inputs
                .values()
                .flatten()
                .any(|input| input.path.ends_with("src/helper.rs"))
        );
        let mut rustc_shared_options = options.clone();
        rustc_shared_options.production_target = ProducerTarget::Cdylib(root.join("Cargo.toml"));
        rustc_shared_options.declarations.actual_build_evidence = Some(evidence_path.clone());
        let rustc_shared = run(&rustc_shared_options).unwrap();
        assert!(
            rustc_shared
                .reasons
                .iter()
                .all(|reason| !reason.contains("no associated actual rustc dep-info")),
            "{:?}",
            rustc_shared.reasons
        );
        let rustc_evidence: serde_json::Value =
            serde_json::from_slice(&fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(rustc_evidence["arguments"][0], "rustc");
        assert!(
            rustc_evidence["arguments"]
                .as_array()
                .unwrap()
                .iter()
                .any(|argument| argument == "--manifest-path")
        );
        assert!(
            rustc_shared
                .consumed_inputs
                .values()
                .flatten()
                .any(|input| input.path.ends_with("src/helper.rs"))
        );
        rustc_shared_options.package = "another-package".into();
        assert!(
            run(&rustc_shared_options)
                .unwrap_err()
                .to_string()
                .contains("does not match requested package")
        );
        rustc_shared_options.package = "producer-fixture".into();
        fs::write(root.join("src/helper.rs"), "pub fn value()->u32 {3}").unwrap();
        assert_ne!(shared.identity, run(&options).unwrap().identity);
        assert_ne!(
            rustc_shared.identity,
            run(&rustc_shared_options).unwrap().identity
        );
        // This tests actual consumed-unit locality, not an assertion that every
        // deployment-native/environment input has been fully qualified.
    }

    #[test]
    fn producer_actual_deployment_provenance_is_separate_and_source_bound() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let owner = root.join("crates/pse-buildinfo");
        fs::create_dir_all(owner.join("src")).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"),
            "[workspace]\nmembers=['crates/pse-buildinfo']\n[package]\nname='producer-fixture'\nversion='0.1.0'\nedition='2024'\n[dependencies]\npse-buildinfo={path='crates/pse-buildinfo'}\n").unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn value()->u32 {pse_buildinfo::value()}\n",
        )
        .unwrap();
        fs::write(
            owner.join("Cargo.toml"),
            "[package]\nname='pse-buildinfo'\nversion='0.0.1'\nedition='2024'\n",
        )
        .unwrap();
        let behavior = "pub fn value()->u32 {7} pub const SOURCE:&[u8]=include_bytes!(concat!(env!(\"OUT_DIR\"),\"/source.identity\")); pub const BUILD:&[u8]=include_bytes!(concat!(env!(\"OUT_DIR\"),\"/build.identity\")); pub const CARGO:&[u8]=include_bytes!(concat!(env!(\"OUT_DIR\"),\"/cargo.lock\")); pub const UV:&[u8]=include_bytes!(concat!(env!(\"OUT_DIR\"),\"/uv.lock\")); pub const GIT:&str=env!(\"PSE_GIT_SHA\");";
        fs::write(owner.join("src/lib.rs"), behavior).unwrap();
        fs::write(owner.join("build.rs"), r#"fn main() {
            let input=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../provenance.input");
            println!("cargo:rerun-if-changed={}",input.display());
            let bytes=std::fs::read(input).unwrap();
            let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
            std::fs::write(out.join("source.identity"),&bytes).unwrap();
            std::fs::write(out.join("build.identity"),[3u8;32]).unwrap();
            std::fs::write(out.join("cargo.lock"),&bytes).unwrap();
            std::fs::write(out.join("uv.lock"),&bytes).unwrap();
            println!("cargo:rustc-env=PSE_GIT_SHA={}",bytes[0]);
        }"#).unwrap();
        fs::write(root.join("provenance.input"), [1u8; 32]).unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repository.join("rust-toolchain.toml"),
            root.join("rust-toolchain.toml"),
        )
        .unwrap();
        cargo_output(root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
        let options = ProducerOptions {
            workspace_root: root.into(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: Vec::new(),
            declared_environment: BTreeMap::new(),
            declarations: InputDeclarations {
                capture_actual_build: true,
                review_catalogs: vec!["review.json".into()],
                ..Default::default()
            },
        };
        let graph: UnitGraph =
            serde_json::from_slice(&cargo_output(root, &unit_graph_arguments(&options)).unwrap())
                .unwrap();
        let metadata: cargo_metadata::Metadata = serde_json::from_slice(
            &cargo_output(
                root,
                &[
                    "metadata".into(),
                    "--format-version=1".into(),
                    "--offline".into(),
                ],
            )
            .unwrap(),
        )
        .unwrap();
        let packages = metadata
            .packages
            .into_iter()
            .map(|package| (package.id.to_string(), package))
            .collect();
        let sources = selected_source_state(&graph, &packages).unwrap();
        let keys = unit_keys(&graph, root).unwrap();
        let (index, unit) = graph
            .units
            .iter()
            .enumerate()
            .find(|(_, unit)| unit.mode == "run-custom-build")
            .unwrap();
        let owner_key = logical_package(&unit.pkg_id, root);
        let review = serde_json::json!({"reviewed_source": reviewed_source_identity(&owner).unwrap(),
            "reviewed_closures":[executable_closure_basis(&graph,index,&sources,&keys).unwrap()],
            "rationale":"This isolated fixture reads only provenance.input to produce outer data; value remains seven.",
            "deployment_provenance":true,"workspace_files":["provenance.input"],
            "workspace_absent_files":["vendor"]});
        fs::write(
            root.join("review.json"),
            serde_json::to_vec(
                &serde_json::json!({"build_scripts":{owner_key.clone():review.clone()}}),
            )
            .unwrap(),
        )
        .unwrap();
        let first = run(&options).unwrap();
        assert!(
            !first.persistent_reuse_eligible,
            "fixture has no production native qualification"
        );
        assert_eq!(
            first.outer_attestation.as_ref().unwrap().source,
            pse_ids::ContentHash::from_bytes([1; 32])
        );
        assert!(
            first
                .deployment_provenance
                .values()
                .flatten()
                .any(|input| input.path.ends_with("provenance.input"))
        );
        fs::write(root.join("provenance.input"), [2u8; 32]).unwrap();
        let changed_provenance = run(&options).unwrap();
        for group in first
            .consumed_inputs
            .keys()
            .chain(changed_provenance.consumed_inputs.keys())
        {
            assert_eq!(
                first.consumed_inputs.get(group),
                changed_provenance.consumed_inputs.get(group),
                "changed scientific group {group}"
            );
        }
        assert_eq!(first.identity, changed_provenance.identity);
        assert_ne!(
            first.outer_attestation,
            changed_provenance.outer_attestation
        );
        assert_ne!(
            serde_json::to_vec(&first.deployment_provenance).unwrap(),
            serde_json::to_vec(&changed_provenance.deployment_provenance).unwrap()
        );
        assert!(
            first
                .deployment_provenance
                .values()
                .flatten()
                .any(|input| input.path.ends_with("vendor"))
        );
        fs::create_dir(root.join("vendor")).unwrap();
        let reappeared = run(&options).unwrap_err();
        assert!(reappeared.to_string().contains("absent workspace input"));
        fs::remove_dir(root.join("vendor")).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("missing-target", root.join("vendor")).unwrap();
            // The raw source inventory can reject a symlink before review
            // expansion; either boundary must refuse the capture.
            assert!(run(&options).is_err());
            fs::remove_file(root.join("vendor")).unwrap();
        }
        fs::write(owner.join("src/lib.rs"), behavior.replace("{7}", "{8}")).unwrap();
        let changed_behavior = run(&options).unwrap();
        assert_ne!(first.identity, changed_behavior.identity);
        assert!(changed_behavior.deployment_provenance.is_empty());
        assert!(changed_behavior.outer_attestation.is_none());
        assert!(
            changed_behavior
                .reasons
                .iter()
                .any(|reason| reason.contains("build-script arbitrary I/O"))
        );
        fs::write(owner.join("src/lib.rs"), behavior).unwrap();
        fs::rename(&owner, root.join("crates/other-buildinfo")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            fs::read_to_string(root.join("Cargo.toml"))
                .unwrap()
                .replace("crates/pse-buildinfo", "crates/other-buildinfo"),
        )
        .unwrap();
        let external_options = ProducerOptions {
            declarations: InputDeclarations {
                review_catalogs: vec!["external.json".into()],
                ..options.declarations.clone()
            },
            ..options
        };
        let moved_graph: UnitGraph = serde_json::from_slice(
            &cargo_output(root, &unit_graph_arguments(&external_options)).unwrap(),
        )
        .unwrap();
        let moved_unit = moved_graph
            .units
            .iter()
            .find(|unit| unit.mode == "run-custom-build")
            .unwrap();
        let moved_key = logical_package(&moved_unit.pkg_id, root);
        let mut moved_review = review;
        moved_review["reviewed_source"] = serde_json::json!(
            reviewed_source_identity(&root.join("crates/other-buildinfo")).unwrap()
        );
        fs::write(
            root.join("external.json"),
            serde_json::to_vec(&serde_json::json!({"build_scripts":{moved_key:moved_review}}))
                .unwrap(),
        )
        .unwrap();
        assert!(
            run(&external_options)
                .unwrap_err()
                .to_string()
                .contains("restricted to the workspace buildinfo owner")
        );
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
                "{} {}: {} {}\n",
                root.join("producer-123.d").display(),
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
        // A Cargo aggregate has an actual artifact output but no compiler .d
        // output. It cannot stand in for this unit's compiler consumption.
        assert!(
            artifact_dep_info(&unit, &graph, &[message.clone()], root)
                .unwrap()
                .is_none()
        );
        fs::write(
            root.join("producer-123.d"),
            format!(
                "{} {}: {}\n",
                root.join("producer-123.d").display(),
                root.join("libproducer-123.rlib").display(),
                root.join("lib.rs").display()
            ),
        )
        .unwrap();
        assert!(
            artifact_dep_info(&unit, &graph, &[message.clone()], root)
                .unwrap()
                .is_some()
        );
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
    fn producer_same_context_artifacts_cover_each_dependency_closure() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let (mut left, first) = artifact_fixture(root);
        let (mut dependency, _) = artifact_fixture(root);
        dependency.pkg_id = "dependency#1".into();
        dependency.target.name = "dependency".into();
        left.dependencies.push(Edge {
            index: 2,
            extern_crate_name: "helper".into(),
        });
        let mut right = left.clone();
        right.dependencies[0].index = 3;
        let mut changed_dependency = dependency.clone();
        changed_dependency.features.push("changed".into());
        let graph = UnitGraph {
            version: 1,
            roots: vec![0, 1],
            units: vec![left.clone(), right.clone(), dependency, changed_dependency],
        };
        let keys = unit_keys(&graph, root).unwrap();
        assert_ne!(keys[&0], keys[&1]);
        let mut second = first.clone();
        second["filenames"] = serde_json::json!([root.join("libproducer-456.rlib")]);
        fs::write(root.join("libproducer-456.rlib"), b"other actual output").unwrap();
        fs::write(root.join("lib.rs"), "fn production() {}").unwrap();
        for suffix in ["123", "456"] {
            fs::write(
                root.join(format!("producer-{suffix}.d")),
                format!(
                    "{} {}: {} {}\n",
                    root.join(format!("producer-{suffix}.d")).display(),
                    root.join(format!("libproducer-{suffix}.rlib")).display(),
                    root.join("lib.rs").display(),
                    root.join(format!("consumed-{suffix}.rs")).display()
                ),
            )
            .unwrap();
        }
        let messages = [first, second];
        let expected = vec![root.join("producer-123.d"), root.join("producer-456.d")];
        assert_eq!(
            artifact_dep_info(&left, &graph, &messages, root).unwrap(),
            Some(expected.clone())
        );
        assert_eq!(
            artifact_dep_info(&right, &graph, &messages, root).unwrap(),
            Some(expected)
        );
        let mut cycle = graph;
        cycle.units[2].dependencies.push(Edge {
            index: 0,
            extern_crate_name: "cycle".into(),
        });
        assert!(unit_keys(&cycle, root).is_err());
    }

    #[test]
    fn producer_reviewed_ambient_values_and_absence_cannot_be_fabricated() {
        let directory = tempfile::tempdir().unwrap();
        let mut declaration = BuildScriptInputs::default();
        declaration
            .environment
            .insert("PATH".into(), "not-the-active-path".into());
        declaration.absent_environment.push("PATH".into());
        let mut inputs = BTreeMap::new();
        let mut reasons = BTreeSet::new();
        add_reviewed_inputs(
            &mut inputs,
            &mut reasons,
            "review",
            &declaration,
            directory.path(),
            "owner",
        )
        .unwrap();
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("reviewed environment PATH changed"))
        );
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("reviewed absent environment PATH is present"))
        );
        declaration.environment.clear();
        declaration.absent_environment.clear();
        if let Some(value) = current_environment("PATH").unwrap() {
            declaration.environment.insert("PATH".into(), value);
        } else {
            declaration.absent_environment.push("PATH".into());
        }
        reasons.clear();
        add_reviewed_inputs(
            &mut inputs,
            &mut reasons,
            "review",
            &declaration,
            directory.path(),
            "owner",
        )
        .unwrap();
        assert!(reasons.is_empty());
    }

    #[test]
    fn producer_explicit_native_and_configuration_closures_cannot_drift() {
        let (directory, configuration, mut review) = configuration_fixture();
        let root = directory.path();
        review.absent_files.push("implicit-tool-config".into());
        review.reviewed_closure = Some(configuration_closure_basis(&review, root).unwrap());
        fs::write(root.join("native-library"), b"reviewed native binary").unwrap();
        fs::write(root.join("explicit-data"), b"reviewed external data").unwrap();
        let mut options = ProducerOptions {
            workspace_root: root.into(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: vec!["explicit-data".into()],
            native_inputs: vec!["native-library".into()],
            declared_environment: BTreeMap::new(),
            declarations: InputDeclarations {
                configuration_executors: vec![review.clone()],
                native_abi: Some("reviewed-test-native.v1".into()),
                native_absent_files: vec!["implicit-native-config".into()],
                native_environment: BTreeMap::from([(
                    "PATH".into(),
                    current_environment("PATH").unwrap(),
                )]),
                ..Default::default()
            },
        };
        let native_basis = native_review_basis(&options, root).unwrap();
        options.declarations.native_reviewed_source = Some(native_basis.clone());
        let before = reviewed_input_state(&options, root).unwrap();
        for file in ["native-library", "explicit-data", "linker-input"] {
            let original = fs::read(root.join(file)).unwrap();
            fs::write(root.join(file), b"changed during capture").unwrap();
            assert_ne!(before, reviewed_input_state(&options, root).unwrap());
            fs::write(root.join(file), original).unwrap();
            assert_eq!(before, reviewed_input_state(&options, root).unwrap());
        }
        options.declarations.native_abi = Some("changed-test-native.v2".into());
        assert_ne!(native_basis, native_review_basis(&options, root).unwrap());
        options.declarations.native_abi = Some("reviewed-test-native.v1".into());
        options.features.push("new-native-branch".into());
        assert_ne!(native_basis, native_review_basis(&options, root).unwrap());
        options.features.clear();
        options.production_target = ProducerTarget::Binary("worker".into());
        assert_ne!(native_basis, native_review_basis(&options, root).unwrap());
        options.production_target = ProducerTarget::Library;
        fs::write(root.join("implicit-native-config"), b"new configuration").unwrap();
        assert_ne!(native_basis, native_review_basis(&options, root).unwrap());
        fs::remove_file(root.join("implicit-native-config")).unwrap();
        assert_eq!(native_basis, native_review_basis(&options, root).unwrap());
        fs::write(
            root.join("implicit-tool-config"),
            b"new implicit tool configuration",
        )
        .unwrap();
        assert_ne!(before, reviewed_input_state(&options, root).unwrap());
        let (_, reasons, _) =
            reviewed_configuration_capture(root, &configuration, &[review], Some("exact"));
        assert_eq!(reasons.len(), 1);
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("reviewed absent file"))
        );
    }

    #[test]
    fn producer_dynamic_macro_review_binds_actual_caller_inputs_and_environment() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("macro/src")).unwrap();
        fs::create_dir_all(root.join("caller-helper/src")).unwrap();
        fs::write(root.join("Cargo.toml"), format!("{FINITE_FIXTURE_MANIFEST}\n[dependencies]\npaste={{path='macro'}}\ncaller-helper={{path='caller-helper'}}\n")).unwrap();
        fs::write(
            root.join("caller-helper/Cargo.toml"),
            "[package]\nname='caller-helper'\nversion='0.0.1'\nedition='2024'\n",
        )
        .unwrap();
        fs::write(
            root.join("macro/Cargo.toml"),
            "[package]\nname='paste'\nversion='0.0.1'\nedition='2024'\n[lib]\nproc-macro=true\n",
        )
        .unwrap();
        fs::write(root.join("macro/src/lib.rs"), r#"
            extern crate proc_macro;
            #[proc_macro] pub fn from_env(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let name = input.to_string().trim_matches('"').to_owned();
                std::env::var(name).unwrap_or_else(|_| "7".into()).parse().unwrap()
            }
        "#).unwrap();
        let first_name = "PSE_PRODUCER_DYNAMIC_FIXTURE_FIRST";
        let second_name = "PSE_PRODUCER_DYNAMIC_FIXTURE_SECOND";
        assert!(current_environment(first_name).unwrap().is_none());
        assert!(current_environment(second_name).unwrap().is_none());
        let caller_input = |name| {
            format!(
                "#[macro_export] macro_rules! from_env {{ () => {{ paste::from_env!(\"{name}\") }} }}"
            )
        };
        fs::write(
            root.join("caller-helper/src/lib.rs"),
            caller_input(first_name),
        )
        .unwrap();
        let direct_caller = "pub fn value()->u32 { caller_helper::from_env!() }";
        fs::write(root.join("src/lib.rs"), direct_caller).unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repository.join("rust-toolchain.toml"),
            root.join("rust-toolchain.toml"),
        )
        .unwrap();
        cargo_output(root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
        let mut options = ProducerOptions {
            workspace_root: root.into(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
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
        let macro_unit = first
            .units
            .iter()
            .find(|unit| unit.target_kind == ["proc-macro"])
            .unwrap();
        let owner = &macro_unit.package_id;
        let basis_key = format!("macro-caller-inputs:{owner}:{}", macro_unit.key);
        let executable_key = format!("executable-closure:{owner}:{}", macro_unit.key);
        let catalog = directory.path().with_extension("dynamic-review.json");
        let mut review = serde_json::json!({
            "reviewed_source": first.reviewed_owner_sources[owner],
            "reviewed_closures": [first.reviewed_owner_sources[&executable_key]],
            "reviewed_callers": [first.reviewed_owner_sources[&basis_key]],
            "environment": [first_name],
            "rationale": "Finite fixture: one literal macro argument reads this exact environment name, with a constant absent-value fallback; no other I/O."
        });
        fs::write(
            &catalog,
            serde_json::to_vec(&serde_json::json!({"proc_macros": {owner: &review}})).unwrap(),
        )
        .unwrap();
        options.declarations.review_catalogs = vec![catalog.clone()];
        let admitted = run(&options).unwrap();
        assert!(
            !admitted
                .reasons
                .iter()
                .any(|reason| reason.contains("macro I/O")
                    || reason.contains("procedural macro arbitrary I/O"))
        );
        assert!(!admitted.persistent_reuse_eligible); // Native/tool closure deliberately unqualified.
        // A new actual invocation cannot inherit the old environment exclusion,
        // despite unchanged macro source and helper implementation.
        fs::write(
            root.join("caller-helper/src/lib.rs"),
            caller_input(second_name),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            direct_caller
        );
        let changed = run(&options).unwrap();
        assert_ne!(
            first.reviewed_owner_sources[&basis_key],
            changed.reviewed_owner_sources[&basis_key]
        );
        assert!(
            changed
                .reasons
                .iter()
                .any(|reason| reason.contains("actual caller-input review"))
        );
        assert_ne!(admitted.identity, changed.identity);
        review["reviewed_callers"] =
            serde_json::json!([changed.reviewed_owner_sources[&basis_key]]);
        review["environment"] = serde_json::json!([second_name]);
        fs::write(
            &catalog,
            serde_json::to_vec(&serde_json::json!({"proc_macros": {owner: &review}})).unwrap(),
        )
        .unwrap();
        let readmitted = run(&options).unwrap();
        assert!(
            !readmitted
                .reasons
                .iter()
                .any(|reason| reason.contains("macro I/O")
                    || reason.contains("procedural macro arbitrary I/O"))
        );
        // Exercise presence/value capture without mutating this process's ambient
        // environment (parallel tests). Production uses the same checked lowering.
        let digest = |actual: Option<&str>| {
            let declaration = match actual {
                Some(value) => BuildScriptInputs {
                    environment: BTreeMap::from([(second_name.into(), value.into())]),
                    ..Default::default()
                },
                None => BuildScriptInputs {
                    absent_environment: vec![second_name.into()],
                    ..Default::default()
                },
            };
            let mut inputs = BTreeMap::new();
            let mut reasons = BTreeSet::new();
            add_reviewed_inputs_with_environment(
                &mut inputs,
                &mut reasons,
                "macro-inputs",
                &declaration,
                root,
                owner,
                |_| Ok(actual.map(str::to_owned)),
            )
            .unwrap();
            assert!(reasons.is_empty());
            serde_json::to_vec(&inputs).unwrap()
        };
        assert_ne!(digest(None), digest(Some("7")));
        assert_ne!(digest(Some("7")), digest(Some("8")));
        let declaration = BuildScriptInputs {
            absent_environment: vec![second_name.into()],
            ..Default::default()
        };
        let mut reasons = BTreeSet::new();
        add_reviewed_inputs_with_environment(
            &mut BTreeMap::new(),
            &mut reasons,
            "macro-inputs",
            &declaration,
            root,
            owner,
            |_| Ok(Some("8".into())),
        )
        .unwrap();
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("reviewed absent environment"))
        );
        fs::remove_file(catalog).unwrap();
    }

    #[test]
    fn producer_pest_preferred_grammar_revokes_review_and_requires_actual_consumption() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("macro/src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            format!("{FINITE_FIXTURE_MANIFEST}\n[dependencies]\npest_derive={{path='macro'}}\n"),
        )
        .unwrap();
        fs::write(root.join("macro/Cargo.toml"), "[package]\nname='pest_derive'\nversion='0.0.1'\nedition='2024'\n[lib]\nproc-macro=true\n").unwrap();
        fs::write(root.join("macro/src/lib.rs"), r#"
            extern crate proc_macro;
            #[proc_macro] pub fn value(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
                let preferred = root.join("numbers.pest");
                let path = if preferred.exists() { preferred } else { root.join("src/numbers.pest") };
                let value = std::fs::read_to_string(&path).unwrap();
                format!("const VALUE:u32={value};const _: &str=include_str!({:?});", path).parse().unwrap()
            }
        "#).unwrap();
        fs::write(root.join("src/numbers.pest"), "7").unwrap();
        let caller = "pest_derive::value!();pub fn value()->u32 { VALUE }";
        fs::write(root.join("src/lib.rs"), caller).unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repository.join("rust-toolchain.toml"),
            root.join("rust-toolchain.toml"),
        )
        .unwrap();
        cargo_output(root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
        let mut options = ProducerOptions {
            workspace_root: root.into(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
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
        let macro_unit = first
            .units
            .iter()
            .find(|unit| unit.target_kind == ["proc-macro"])
            .unwrap();
        let owner = &macro_unit.package_id;
        let caller_owner = &first
            .units
            .iter()
            .find(|unit| unit.target_kind == ["lib"])
            .unwrap()
            .package_id;
        let basis_key = format!("macro-caller-inputs:{owner}:{}", macro_unit.key);
        let executable_key = format!("executable-closure:{owner}:{}", macro_unit.key);
        let catalog = root.with_extension("pest-review.json");
        let mut review = serde_json::json!({
            "reviewed_source": first.reviewed_owner_sources[owner],
            "reviewed_closures": [first.reviewed_owner_sources[&executable_key]],
            "reviewed_callers": [], "pest_grammars": {caller_owner: ["numbers.pest"]},
            "rationale": "Finite fixed-path fixture for Pest preferred/fallback reads and emitted include_str dependency; no other macro I/O."
        });
        let write_review = |review: &serde_json::Value| {
            fs::write(
                &catalog,
                serde_json::to_vec(&serde_json::json!({"proc_macros": {owner: review}})).unwrap(),
            )
            .unwrap()
        };
        write_review(&review);
        options.declarations.review_catalogs = vec![catalog.clone()];
        let with_candidates = run(&options).unwrap();
        review["reviewed_callers"] =
            serde_json::json!([with_candidates.reviewed_owner_sources[&basis_key]]);
        write_review(&review);
        let admitted = run(&options).unwrap();
        assert!(
            !admitted
                .reasons
                .iter()
                .any(|reason| reason.contains("Pest grammar")
                    || reason.contains("macro I/O")
                    || reason.contains("procedural macro arbitrary I/O"))
        );
        assert!(!admitted.persistent_reuse_eligible); // Tool/native closure is deliberately unqualified.
        fs::write(root.join("numbers.pest"), "9").unwrap();
        let shadowed = run(&options).unwrap();
        assert_ne!(
            admitted.reviewed_owner_sources[&basis_key],
            shadowed.reviewed_owner_sources[&basis_key]
        );
        assert!(
            shadowed
                .reasons
                .iter()
                .any(|reason| reason.contains("actual caller-input review"))
        );
        assert!(shadowed.reasons.iter().any(|reason| {
            reason.contains("chosen Pest grammar lacks actual compiler consumption")
        }));
        // Even approving the new candidate-state basis cannot certify Cargo's
        // still-fresh artifact from the old fallback. It must compile the choice.
        review["reviewed_callers"] =
            serde_json::json!([shadowed.reviewed_owner_sources[&basis_key]]);
        write_review(&review);
        let stale_artifact = run(&options).unwrap();
        assert!(stale_artifact.reasons.iter().any(|reason| {
            reason.contains("chosen Pest grammar lacks actual compiler consumption")
        }));
        fs::write(root.join("src/lib.rs"), format!("{caller}\n")).unwrap();
        let recompiled = run(&options).unwrap();
        assert!(!recompiled.reasons.iter().any(|reason| {
            reason.contains("chosen Pest grammar lacks actual compiler consumption")
        }));
        review["reviewed_callers"] =
            serde_json::json!([recompiled.reviewed_owner_sources[&basis_key]]);
        write_review(&review);
        let readmitted = run(&options).unwrap();
        assert!(
            !readmitted
                .reasons
                .iter()
                .any(|reason| reason.contains("Pest grammar")
                    || reason.contains("macro I/O")
                    || reason.contains("procedural macro arbitrary I/O"))
        );
        assert_ne!(admitted.identity, readmitted.identity);
        fs::remove_file(catalog).unwrap();
    }

    #[test]
    fn producer_review_catalog_only_grants_current_selected_owner() {
        let directory = tempfile::tempdir().unwrap();
        let helper_directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(helper_directory.path().join("src")).unwrap();
        fs::write(
            helper_directory.path().join("Cargo.toml"),
            "[package]\nname='reviewed-helper'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        fs::write(
            helper_directory.path().join("src/lib.rs"),
            "pub fn helper() {}\n",
        )
        .unwrap();
        let helper_path =
            toml::Value::String(helper_directory.path().to_string_lossy().into_owned()).to_string();
        fs::write(root.join("Cargo.toml"), format!("{FINITE_FIXTURE_MANIFEST}\n[features]\nexternal-io=[]\n[build-dependencies]\nreviewed-helper={{path={helper_path}}}\n")).unwrap();
        fs::write(root.join("src/lib.rs"), FINITE_FIXTURE_SOURCE).unwrap();
        fs::write(root.join("build.rs"), "fn main() {}\n").unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repository.join("rust-toolchain.toml"),
            root.join("rust-toolchain.toml"),
        )
        .unwrap();
        cargo_output(root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
        let options = ProducerOptions {
            workspace_root: root.into(),
            package: "producer-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: Vec::new(),
            declared_environment: BTreeMap::new(),
            declarations: InputDeclarations {
                review_catalogs: vec!["reviews.json".into()],
                ..Default::default()
            },
        };
        let graph: UnitGraph =
            serde_json::from_slice(&cargo_output(root, &unit_graph_arguments(&options)).unwrap())
                .unwrap();
        let metadata: cargo_metadata::Metadata = serde_json::from_slice(
            &cargo_output(
                root,
                &[
                    "metadata".into(),
                    "--format-version=1".into(),
                    "--offline".into(),
                ],
            )
            .unwrap(),
        )
        .unwrap();
        let packages: BTreeMap<_, _> = metadata
            .packages
            .into_iter()
            .map(|package| (package.id.to_string(), package))
            .collect();
        let owner = logical_package(&graph.units[graph.roots[0]].pkg_id, root);
        let basis = reviewed_source_identity(root).unwrap();
        let sources = selected_source_state(&graph, &packages).unwrap();
        let keys = unit_keys(&graph, root).unwrap();
        let closures: Vec<_> = selected_indices(&graph)
            .unwrap()
            .into_iter()
            .filter(|index| graph.units[*index].mode == "run-custom-build")
            .map(|index| executable_closure_basis(&graph, index, &sources, &keys).unwrap())
            .collect();
        let compiler = Command::new("rustc")
            .current_dir(root)
            .arg("-vV")
            .output()
            .unwrap();
        assert!(compiler.status.success());
        let compiler = std::str::from_utf8(&compiler.stdout).unwrap();
        let host = compiler
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .unwrap();
        // Keep the catalog outside the reviewed package's source tree: including
        // a package's own review would create a self-referential review identity.
        let catalog = directory.path().with_extension("reviews.json");
        fs::write(
            &catalog,
            serde_json::to_vec(&serde_json::json!({
                "build_scripts": {owner.clone(): {
                    "reviewed_source": basis, "rationale": "The complete build script is empty.",
                    "reviewed_closures": closures,
                    "excluded_features": ["external-io"],
                    "platforms": [host],
                    "caller_package_files": ["src/lib.rs", "optional-caller-input.txt"],
                    "environment": ["PATH"], "package_files": ["src/lib.rs"]
                }}
            }))
            .unwrap(),
        )
        .unwrap();
        let mut options = options;
        options.declarations.review_catalogs = vec![catalog.clone()];
        let admitted = expand_reviews(&options, root, &graph, &packages).unwrap();
        let declaration = &admitted.declarations.build_scripts[&owner];
        assert!(declaration.complete);
        assert!(declaration.files.contains(&root.join("src/lib.rs")));
        assert!(
            declaration
                .files
                .contains(&helper_directory.path().join("src/lib.rs"))
        );
        assert!(
            declaration
                .absent_files
                .contains(&root.join("optional-caller-input.txt"))
        );
        let owner_before = reviewed_source_identity(root).unwrap();
        fs::write(
            helper_directory.path().join("src/lib.rs"),
            "pub fn helper() { let _ = std::fs::read(\"new-input\"); }\n",
        )
        .unwrap();
        assert_eq!(owner_before, reviewed_source_identity(root).unwrap());
        assert!(
            expand_reviews(&options, root, &graph, &packages)
                .unwrap()
                .declarations
                .build_scripts
                .is_empty()
        );
        fs::write(
            helper_directory.path().join("src/lib.rs"),
            "pub fn helper() {}\n",
        )
        .unwrap();
        assert!(
            !expand_reviews(&options, root, &graph, &packages)
                .unwrap()
                .declarations
                .build_scripts
                .is_empty()
        );
        let before = reviewed_input_state(&admitted, root).unwrap();
        fs::write(
            root.join("optional-caller-input.txt"),
            b"new ambient caller data",
        )
        .unwrap();
        assert_ne!(before, reviewed_input_state(&admitted, root).unwrap());
        let mut inputs = BTreeMap::new();
        let mut reasons = BTreeSet::new();
        add_reviewed_inputs(
            &mut inputs,
            &mut reasons,
            "macro-caller",
            declaration,
            root,
            &owner,
        )
        .unwrap();
        assert!(
            reasons
                .iter()
                .any(|reason| reason.contains("reviewed absent file"))
        );
        fs::remove_file(root.join("optional-caller-input.txt")).unwrap();
        let mut unreviewed_feature = options.clone();
        unreviewed_feature.features.push("external-io".into());
        let graph_with_feature: UnitGraph = serde_json::from_slice(
            &cargo_output(root, &unit_graph_arguments(&unreviewed_feature)).unwrap(),
        )
        .unwrap();
        assert!(
            expand_reviews(&unreviewed_feature, root, &graph_with_feature, &packages)
                .unwrap()
                .declarations
                .build_scripts
                .is_empty()
        );
        let mut unreviewed_target = graph.clone();
        for unit in &mut unreviewed_target.units {
            unit.platform = Some("unreviewed-scientific-target".into());
        }
        assert!(
            expand_reviews(&options, root, &unreviewed_target, &packages)
                .unwrap()
                .declarations
                .build_scripts
                .is_empty()
        );
        let before = reviewed_input_state(&admitted, root).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn value()->u32 {3}\n").unwrap();
        assert_ne!(before, reviewed_input_state(&admitted, root).unwrap());
        assert!(
            expand_reviews(&options, root, &graph, &packages)
                .unwrap()
                .declarations
                .build_scripts
                .is_empty()
        );
        fs::remove_file(catalog).unwrap();
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
        assert!(validate_production_root(&graph, &ProducerTarget::Library).is_ok());
        assert!(
            validate_production_root(&graph, &ProducerTarget::Binary("producer".into())).is_err()
        );
        let mut ambiguous = graph.clone();
        ambiguous.roots.push(0);
        assert!(validate_production_root(&ambiguous, &ProducerTarget::Library).is_err());
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
            selected_root: None,
            identity: "first".into(),
            persistent_reuse_eligible: false,
            reasons: vec!["unknown inputs".into()],
            units: Vec::new(),
            selected_lock_records: Vec::new(),
            consumed_inputs: BTreeMap::new(),
            deployment_provenance: BTreeMap::new(),
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
