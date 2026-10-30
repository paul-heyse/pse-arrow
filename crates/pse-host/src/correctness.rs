// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cargo-owned compiler-unit selection and Arrow correctness admission.
//!
//! The nextest adapter observes Cargo's translated build argv, never interprets nextest's
//! complete CLI grammar, and never sends unit-graph JSON into its compiler-message stream.
//! Cargo-hack retains feature enumeration; its native Cargo callback qualifies selected
//! package features and isolates actual diagnostic taskdump checks through Cargo profiles.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

const REAL_CARGO: &str = "PSE_CORRECTNESS_REAL_CARGO";
const ADAPTER: &str = "PSE_CORRECTNESS_ADAPTER";
const FIXTURE_METADATA_CAPTURE: &str = "PSE_FIXTURE_CARGO_METADATA_CAPTURE";
const BUILD_CAPTURE: &str = "PSE_CORRECTNESS_BUILD_CAPTURE";
const CARGO_HACK_ADAPTER: &str = "CARGO_HACK_CARGO_SRC";
const DIAGNOSTIC_TARGET: &str = "x86_64-unknown-linux-gnu";
const DIAGNOSTIC_CFG: &str = "target.'cfg(all())'.rustflags=[\"--cfg\",\"tokio_unstable\"]";

/// Failure to establish the selected correctness closure, distinct from a Cargo exit.
#[derive(Debug, thiserror::Error)]
pub enum CorrectnessError {
    /// External Cargo JSON was invalid.
    #[error("invalid Cargo graph: {0}")]
    Json(#[from] serde_json::Error),
    /// A selected external fact or invocation cannot establish coverage.
    #[error("correctness admission: {0}")]
    Invalid(String),
    /// Native tool I/O failed.
    #[error("correctness tool I/O: {0}")]
    Io(#[from] io::Error),
}
pse_diagnostics::impl_diagnostic! {
    CorrectnessError,
    code(this) { Some(match this {Self::Json(_) => pse_diagnostics::DiagnosticCode::CorrectnessGraph, Self::Invalid(_) => pse_diagnostics::DiagnosticCode::CorrectnessAdmission, Self::Io(_) => pse_diagnostics::DiagnosticCode::CorrectnessIo}) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> CorrectnessError {
    CorrectnessError::Invalid(message.into())
}

#[derive(Debug, Deserialize)]
struct Package {
    id: String,
    name: String,
    features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    manifest_path: Option<PathBuf>,
    #[serde(default)]
    metadata: serde_json::Value,
}
#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    #[serde(default)]
    workspace_root: Option<PathBuf>,
}
#[derive(Debug, Deserialize)]
struct Dependency {
    index: usize,
}
#[derive(Debug, Deserialize)]
struct Target {
    name: String,
    kind: Vec<String>,
    src_path: String,
}
#[derive(Debug, Deserialize)]
struct Unit {
    pkg_id: String,
    target: Target,
    features: Vec<String>,
    dependencies: Vec<Dependency>,
    platform: Option<String>,
    mode: String,
}
#[derive(Debug, Deserialize)]
struct Graph {
    version: u32,
    units: Vec<Unit>,
    roots: Vec<usize>,
}
impl Graph {
    fn reached(
        &self,
        packages: &BTreeMap<String, Package>,
        roots: &[usize],
    ) -> Result<BTreeSet<usize>, CorrectnessError> {
        if self.version != 1 {
            return Err(invalid("unsupported Cargo unit-graph version"));
        }
        let mut pending = roots.to_vec();
        let mut seen = BTreeSet::new();
        while let Some(index) = pending.pop() {
            if !seen.insert(index) {
                continue;
            }
            let unit = self
                .units
                .get(index)
                .ok_or_else(|| invalid("Cargo dependency/root index is out of bounds"))?;
            if !packages.contains_key(&unit.pkg_id) {
                return Err(invalid("Cargo unit references unresolved package"));
            }
            pending.extend(unit.dependencies.iter().map(|edge| edge.index));
        }
        Ok(seen)
    }
    fn root_identity(&self) -> Result<BTreeSet<RootIdentity>, CorrectnessError> {
        self.roots
            .iter()
            .map(|index| {
                let unit = self
                    .units
                    .get(*index)
                    .ok_or_else(|| invalid("Cargo root index is out of bounds"))?;
                Ok((
                    unit.pkg_id.clone(),
                    unit.target.name.clone(),
                    unit.target.kind.clone(),
                    unit.target.src_path.clone(),
                    unit.platform.clone(),
                    unit.mode.clone(),
                ))
            })
            .collect()
    }
}
type RootIdentity = (String, String, Vec<String>, String, Option<String>, String);
fn arrow(package: &Package) -> bool {
    package.name == "arrow" || package.name.starts_with("arrow-")
}

/// The validated original scope, with qualified features and selected diagnostic isolation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Resolution {
    /// Actual Cargo arguments, excluding its executable.
    pub arguments: Vec<OsString>,
    /// Selected root package names, not a test inventory.
    pub packages: Vec<String>,
    /// Reached compiler units after Arrow admission, including host/build contexts.
    /// These are planned compiler actions, never evidence of built test binaries.
    pub units: Vec<CompilerUnit>,
    /// Prepared provider capabilities required by the actual reached backend units.
    pub native_capabilities: BTreeSet<String>,
}

/// Cargo's selected compiler-unit context, retained without collapsing host/target instances.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub struct CompilerUnit {
    /// Cargo package identity; names alone cannot join versions or sources.
    pub package_id: String,
    /// Cargo package name for presentation and feature qualification.
    pub package_name: String,
    /// Actual target name.
    pub target_name: String,
    /// Cargo target kinds, including unknown future kinds verbatim.
    pub target_kinds: Vec<String>,
    /// Source identity distinguishes similarly named targets.
    pub source_path: String,
    /// Cargo's resolved feature set for this compiler unit.
    pub features: BTreeSet<String>,
    /// Explicit compilation platform; `None` retains Cargo's host context.
    pub platform: Option<String>,
    /// Cargo compiler mode, for example build, test or check.
    pub mode: String,
    /// Whether Cargo selected this unit as a root.
    pub root: bool,
}
fn mode_position(arguments: &[OsString]) -> Result<usize, CorrectnessError> {
    let mut index = 0;
    while let Some(argument) = arguments.get(index) {
        let text = argument
            .to_str()
            .ok_or_else(|| invalid("Cargo control arguments must be UTF-8"))?;
        if [
            "check",
            "build",
            "test",
            "bench",
            "run",
            "clippy",
            "metadata",
            "locate-project",
        ]
        .contains(&text)
        {
            return Ok(index);
        }
        if text.starts_with("--color=")
            || matches!(
                text,
                "--locked"
                    | "--offline"
                    | "--frozen"
                    | "--quiet"
                    | "-q"
                    | "--verbose"
                    | "-v"
                    | "-vv"
            )
        {
            index += 1;
        } else if text == "--color" {
            index += 2;
        } else {
            return Err(invalid(format!(
                "unsupported Cargo command envelope {text}"
            )));
        }
    }
    Err(invalid("Cargo operation is missing"))
}
fn graph_arguments(arguments: &[OsString]) -> Result<Vec<OsString>, CorrectnessError> {
    let position = mode_position(arguments)?;
    let mode = arguments[position]
        .to_str()
        .ok_or_else(|| invalid("Cargo operation must be UTF-8"))?;
    if !["check", "build", "test", "bench", "run", "clippy"].contains(&mode) {
        return Err(invalid("not a Cargo compiler operation"));
    }
    let end = arguments
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(arguments.len());
    let mut graph = arguments[..end].to_vec();
    if mode == "run" {
        if !arguments[position + 1..end].iter().any(|arg| {
            arg == "--bin"
                || arg == "--example"
                || arg
                    .to_str()
                    .is_some_and(|arg| arg.starts_with("--bin=") || arg.starts_with("--example="))
        }) {
            return Err(invalid(
                "Cargo run requires explicit --bin or --example correctness scope",
            ));
        }
        graph[position] = "build".into();
    } else if mode == "clippy" {
        graph[position] = "check".into();
        // Clippy owns this lint-scope switch; Cargo check still needs the whole
        // selected dependency graph to establish Arrow validation. The actual
        // invocation keeps its original arguments, including --no-deps.
        graph.retain(|argument| argument != "--no-deps");
    }
    if graph.iter().any(|arg| arg == "--unit-graph") {
        return Err(invalid("unit-graph is owned by correctness admission"));
    }
    graph.extend([
        "--unit-graph".into(),
        "-Z".into(),
        "unstable-options".into(),
    ]);
    Ok(graph)
}
fn metadata_arguments(arguments: &[OsString]) -> Result<Vec<OsString>, CorrectnessError> {
    let position = mode_position(arguments)?;
    let mut result: Vec<OsString> = [
        "metadata",
        "--format-version",
        "1",
        "--locked",
        "--all-features",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    let mut index = position + 1;
    while let Some(argument) = arguments.get(index) {
        if argument == "--" {
            break;
        }
        let text = argument
            .to_str()
            .ok_or_else(|| invalid("Cargo selector must be UTF-8"))?;
        let key = text.split('=').next().unwrap_or(text);
        if ["--manifest-path", "--config", "--lockfile-path"].contains(&key) {
            result.push(argument.clone());
            if !text.contains('=') {
                index += 1;
                result.push(
                    arguments
                        .get(index)
                        .ok_or_else(|| invalid("Cargo selector value is missing"))?
                        .clone(),
                );
            }
        } else if ["--offline", "--frozen"].contains(&text) {
            result.push(argument.clone());
        }
        index += 1;
    }
    Ok(result)
}
struct CheckControls {
    manifest: Option<PathBuf>,
    targets: Vec<String>,
    features: Vec<(usize, &'static str, Vec<String>)>,
    profile: Option<String>,
    release: bool,
    all_features: bool,
    end: usize,
}
fn check_controls(arguments: &[OsString]) -> Result<Option<CheckControls>, CorrectnessError> {
    let position = mode_position(arguments)?;
    if arguments[position] != "check" && arguments[position] != "clippy" {
        return Ok(None);
    }
    let end = arguments
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(arguments.len());
    let mut controls = CheckControls {
        manifest: None,
        targets: Vec::new(),
        features: vec![],
        profile: None,
        release: false,
        all_features: false,
        end,
    };
    let mut index = position + 1;
    while index < end {
        let argument = arguments[index]
            .to_str()
            .ok_or_else(|| invalid("Cargo check controls must be UTF-8"))?;
        let mut feature = None;
        match argument {
            "--manifest-path" | "--target" | "--features" | "-F" | "--profile" => {
                index += 1;
                if index >= end {
                    return Err(invalid(format!("{argument} requires a value")));
                }
                match argument {
                    "--manifest-path" => controls.manifest = Some(PathBuf::from(&arguments[index])),
                    "--target" => controls.targets.push(
                        arguments[index]
                            .to_str()
                            .ok_or_else(|| invalid("Cargo target must be UTF-8"))?
                            .into(),
                    ),
                    "--profile" => {
                        controls.profile = Some(
                            arguments[index]
                                .to_str()
                                .ok_or_else(|| invalid("Cargo profile must be UTF-8"))?
                                .into(),
                        )
                    }
                    _ => {
                        feature = Some((
                            "",
                            arguments[index]
                                .to_str()
                                .ok_or_else(|| invalid("Cargo features must be UTF-8"))?,
                        ))
                    }
                }
            }
            "--release" | "-r" => controls.release = true,
            "--all-features" => controls.all_features = true,
            _ => {
                if let Some(value) = argument.strip_prefix("--manifest-path=") {
                    controls.manifest = Some(value.into());
                } else if let Some(value) = argument.strip_prefix("--target=") {
                    controls.targets.push(value.into());
                } else if let Some(value) = argument.strip_prefix("--profile=") {
                    controls.profile = Some(value.into());
                } else if let Some(value) = argument.strip_prefix("--features=") {
                    feature = Some(("--features=", value));
                } else if let Some(value) = argument.strip_prefix("-F")
                    && !value.is_empty()
                {
                    feature = Some(("-F", value));
                }
            }
        }
        if let Some((prefix, features)) = feature {
            controls.features.push((
                index,
                prefix,
                features
                    .split(|character: char| character == ',' || character.is_whitespace())
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .collect(),
            ));
        }
        index += 1;
    }
    Ok(Some(controls))
}
// cargo-hack's per-package callback uses --manifest-path with bare features.
// Reuse this invocation's real Cargo metadata, rather than rereading manifests
// or obtaining a second resolution from a parallel selection owner.
fn qualify_selected_check_features(
    arguments: &[OsString],
    packages: &BTreeMap<String, Package>,
) -> Result<Vec<OsString>, CorrectnessError> {
    let Some(controls) = check_controls(arguments)? else {
        return Ok(arguments.to_vec());
    };
    if !controls
        .features
        .iter()
        .flat_map(|(_, _, features)| features)
        .any(|feature| !feature.contains('/'))
    {
        return Ok(arguments.to_vec());
    }
    let Some(manifest) = controls.manifest else {
        return Ok(arguments.to_vec());
    };
    let manifest = manifest.canonicalize()?;
    let mut selected = packages.values().filter(|package| {
        package
            .manifest_path
            .as_ref()
            .is_some_and(|path| path.canonicalize().is_ok_and(|path| path == manifest))
    });
    let package = selected
        .next()
        .ok_or_else(|| invalid("selected check manifest is absent from Cargo metadata"))?;
    if selected.next().is_some() {
        return Err(invalid(
            "selected check manifest resolves to multiple Cargo packages",
        ));
    }
    let mut qualified = arguments.to_vec();
    for (index, prefix, features) in controls.features {
        if features.iter().any(|feature| !feature.contains('/')) {
            qualified[index] = format!(
                "{prefix}{}",
                features
                    .into_iter()
                    .map(|feature| if feature.contains('/') {
                        feature
                    } else {
                        format!("{}/{feature}", package.name)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .into();
        }
    }
    Ok(qualified)
}
fn diagnostic_check_arguments(arguments: &[OsString]) -> Result<Vec<OsString>, CorrectnessError> {
    diagnostic_scope_arguments(arguments, false)
}
// This checkout already selects native Linux tuple rustflags (including mold),
// so Cargo joins the cfg flags without displacing build.rustflags fallback.
// This is not a universal target/config flag resolver. Profile rustflags alone
// reach compilation but not Cargo's rustc --print=cfg dependency queries.
fn diagnostic_scope_arguments(
    arguments: &[OsString],
    resolved_taskdump: bool,
) -> Result<Vec<OsString>, CorrectnessError> {
    let Some(controls) = check_controls(arguments)? else {
        return Ok(arguments.to_vec());
    };
    if !resolved_taskdump
        && !controls
            .features
            .iter()
            .flat_map(|(_, _, features)| features)
            .any(|feature| feature.rsplit('/').next() == Some("diagnostic-taskdump"))
    {
        return Ok(arguments.to_vec());
    }
    if std::env::consts::OS != "linux"
        || std::env::consts::ARCH != "x86_64"
        || controls
            .targets
            .iter()
            .any(|target| target != DIAGNOSTIC_TARGET)
    {
        return Err(invalid(
            "diagnostic-taskdump supports only native x86_64-unknown-linux-gnu selected Cargo targets",
        ));
    }
    if controls.release
        || controls
            .profile
            .as_deref()
            .is_some_and(|profile| profile != "diagnostic-taskdump")
    {
        return Err(invalid(
            "diagnostic-taskdump checks require the isolated diagnostic-taskdump profile",
        ));
    }
    let mut additions = vec![];
    if !arguments[..controls.end]
        .iter()
        .any(|argument| argument == DIAGNOSTIC_CFG)
    {
        additions.extend([OsString::from("--config"), OsString::from(DIAGNOSTIC_CFG)]);
    }
    if controls.profile.is_none() {
        additions.extend(["--profile".into(), "diagnostic-taskdump".into()]);
    }
    let mut selected = arguments.to_vec();
    selected.splice(controls.end..controls.end, additions);
    Ok(selected)
}
fn diagnostic_cfg_selected(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .take_while(|argument| *argument != "--")
        .any(|argument| argument == DIAGNOSTIC_CFG)
}
fn diagnostic_graph(arguments: &[OsString], graph: &Graph) -> Result<(), CorrectnessError> {
    if diagnostic_cfg_selected(arguments) {
        for root in &graph.roots {
            let unit = graph
                .units
                .get(*root)
                .ok_or_else(|| invalid("Cargo root index is out of bounds"))?;
            if unit
                .platform
                .as_deref()
                .is_some_and(|target| target != DIAGNOSTIC_TARGET)
            {
                return Err(invalid(
                    "diagnostic-taskdump supports only native x86_64-unknown-linux-gnu selected Cargo targets",
                ));
            }
        }
    }
    Ok(())
}
fn diagnostic_workspace(
    arguments: &[OsString],
    workspace: Option<&Path>,
) -> Result<(), CorrectnessError> {
    if diagnostic_cfg_selected(arguments) {
        let workspace = workspace.ok_or_else(|| invalid(
            "diagnostic-taskdump requires Cargo metadata.workspace_root for its executable checkout"
        ))?.canonicalize()?;
        if workspace != executable_checkout_root()? {
            return Err(invalid(
                "diagnostic-taskdump supports only its executable's checkout workspace; foreign workspace cannot preserve native target flag composition",
            ));
        }
    }
    Ok(())
}
/// Resolve with injected Cargo capture for focused controls; native execution uses real Cargo.
pub fn resolve_with<F>(
    arguments: &[OsString],
    mut capture: F,
) -> Result<Resolution, CorrectnessError>
where
    F: FnMut(&[OsString]) -> Result<Vec<u8>, CorrectnessError>,
{
    let arguments = diagnostic_check_arguments(arguments)?;
    let metadata: Metadata = serde_json::from_slice(&capture(&metadata_arguments(&arguments)?)?)?;
    let workspace = metadata.workspace_root;
    diagnostic_workspace(&arguments, workspace.as_deref())?;
    let mut packages = BTreeMap::new();
    for package in metadata.packages {
        if packages.insert(package.id.clone(), package).is_some() {
            return Err(invalid("Cargo metadata repeats a package identity"));
        }
    }
    let mut arguments = qualify_selected_check_features(&arguments, &packages)?;
    let mut original: Graph = serde_json::from_slice(&capture(&graph_arguments(&arguments)?)?)?;
    diagnostic_graph(&arguments, &original)?;
    // All-feature check/Clippy callbacks need the same isolation even when
    // cargo-hack (or the caller) does not spell the diagnostic feature. The
    // actual reached units decide, including target and host dependency units.
    if check_controls(&arguments)?.is_some_and(|controls| controls.all_features)
        && original
            .reached(&packages, &original.roots)?
            .iter()
            .any(|index| {
                original.units[*index]
                    .features
                    .iter()
                    .any(|feature| feature == "diagnostic-taskdump")
            })
    {
        let diagnostic = diagnostic_scope_arguments(&arguments, true)?;
        diagnostic_workspace(&diagnostic, workspace.as_deref())?;
        diagnostic_graph(&diagnostic, &original)?;
        if diagnostic != arguments {
            let graph: Graph = serde_json::from_slice(&capture(&graph_arguments(&diagnostic)?)?)?;
            diagnostic_graph(&diagnostic, &graph)?;
            if original.root_identity()? != graph.root_identity()? {
                return Err(invalid(
                    "diagnostic isolation changed selected Cargo package/target roots",
                ));
            }
            arguments = diagnostic;
            original = graph;
        }
    }
    let mut additions = BTreeSet::new();
    let mut names = BTreeSet::new();
    for root in &original.roots {
        let reached = original.reached(&packages, &[*root])?;
        let unit = &original.units[*root];
        let package = &packages[&unit.pkg_id];
        names.insert(package.name.clone());
        if reached
            .iter()
            .any(|index| arrow(&packages[&original.units[*index].pkg_id]))
        {
            let feature = if arrow(package) {
                "force_validate"
            } else {
                "force-validate"
            };
            if !package.features.contains_key(feature) {
                return Err(invalid(format!(
                    "{} has no explicit {feature} propagation",
                    package.name
                )));
            }
            if !unit.features.iter().any(|value| value == feature) {
                additions.insert(format!("{}/{feature}", package.name));
            }
        }
    }
    // Check malformed indices even when a graph has no selected roots.
    original.reached(&packages, &original.roots)?;
    let mut selected = arguments.clone();
    if !additions.is_empty() {
        let position = selected
            .iter()
            .position(|arg| arg == "--")
            .unwrap_or(selected.len());
        selected.splice(
            position..position,
            [
                "--features".into(),
                additions.into_iter().collect::<Vec<_>>().join(",").into(),
            ],
        );
    }
    let validated = if selected == arguments {
        original
    } else {
        let graph: Graph = serde_json::from_slice(&capture(&graph_arguments(&selected)?)?)?;
        if original.root_identity()? != graph.root_identity()? {
            return Err(invalid(
                "validation changed selected Cargo package/target roots",
            ));
        }
        graph
    };
    let reached = validated.reached(&packages, &validated.roots)?;
    for index in &reached {
        let unit = &validated.units[*index];
        let package = &packages[&unit.pkg_id];
        if arrow(package)
            && package.features.contains_key("force_validate")
            && !unit
                .features
                .iter()
                .any(|feature| feature == "force_validate")
        {
            return Err(invalid(format!(
                "{} lacks force_validate in {:?} {} unit",
                package.name, unit.platform, unit.mode
            )));
        }
    }
    let mut native_capabilities = BTreeSet::new();
    let mut units = Vec::with_capacity(reached.len());
    for index in reached {
        let unit = &validated.units[index];
        let package = &packages[&unit.pkg_id];
        if package.name == "pse-backend-native" {
            native_capabilities.extend(
                crate::native::requirements_from_backend_features(
                    &package.metadata,
                    &unit.features,
                )
                .map_err(|error| invalid(error.to_string()))?,
            );
        }
        units.push(CompilerUnit {
            package_id: unit.pkg_id.clone(),
            package_name: package.name.clone(),
            target_name: unit.target.name.clone(),
            target_kinds: unit.target.kind.clone(),
            source_path: unit.target.src_path.clone(),
            features: unit.features.iter().cloned().collect(),
            platform: unit.platform.clone(),
            mode: unit.mode.clone(),
            root: validated.roots.contains(&index),
        });
    }
    Ok(Resolution {
        arguments: selected,
        packages: names.into_iter().collect(),
        units,
        native_capabilities,
    })
}

/// Resolve real Cargo compiler actions without building, preparing providers or starting services.
pub fn resolve(arguments: &[OsString]) -> Result<Resolution, CorrectnessError> {
    let cargo = real_cargo()?;
    resolve_with(arguments, |arguments| capture(&cargo, arguments))
}
fn native_command(cargo: &OsStr, arguments: &[OsString]) -> Command {
    native_command_with_rustflags(
        cargo,
        arguments,
        std::env::var_os("CARGO_ENCODED_RUSTFLAGS"),
        std::env::var_os("RUSTFLAGS"),
    )
}
fn native_command_with_rustflags(
    cargo: &OsStr,
    arguments: &[OsString],
    encoded: Option<OsString>,
    rustflags: Option<OsString>,
) -> Command {
    let mut command = Command::new(cargo);
    command
        .args(arguments)
        .env("CARGO", cargo)
        .env_remove(ADAPTER)
        .env_remove(CARGO_HACK_ADAPTER);
    if diagnostic_cfg_selected(arguments) {
        // Cargo gives these channels precedence over target configuration, even
        // when explicitly empty. Extend only the active child's literal bytes;
        // do not split/re-encode flags, change the inactive channel, or mutate
        // the process environment. The same constructor serves every query and
        // the final compiler invocation. Explicit --target leaves host flags to Cargo.
        if let Some(mut flags) = encoded {
            if !flags.is_empty() {
                flags.push("\x1f");
            }
            flags.push("--cfg\x1ftokio_unstable");
            command.env("CARGO_ENCODED_RUSTFLAGS", flags);
        } else if let Some(mut flags) = rustflags {
            flags.push(" --cfg tokio_unstable");
            command.env("RUSTFLAGS", flags);
        }
    }
    command
}
fn capture(cargo: &OsStr, arguments: &[OsString]) -> Result<Vec<u8>, CorrectnessError> {
    let output = native_command(cargo, arguments).output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "Cargo graph request failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(output.stdout)
}
fn exit_status(status: ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt as _;
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
}
pub(crate) fn real_cargo() -> Result<OsString, CorrectnessError> {
    let cargo = std::env::var_os(REAL_CARGO)
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("CARGO").filter(|value| !value.is_empty()))
        .unwrap_or_else(|| "cargo".into());
    if std::fs::canonicalize(&cargo)
        .ok()
        .is_some_and(|path| std::env::current_exe().is_ok_and(|current| current == path))
    {
        return Err(invalid(
            "real Cargo resolves to correctness adapter; refusing recursion",
        ));
    }
    Ok(cargo)
}
fn execute(cargo: &OsStr, arguments: &[OsString]) -> Result<i32, CorrectnessError> {
    Ok(exit_status(native_command(cargo, arguments).status()?))
}
// Only nextest's actual metadata adapter request publishes this private copy.
// Compiler graph validation queries must not publish or reuse this capability.
fn execute_metadata(
    cargo: &OsStr,
    arguments: &[OsString],
    destination: &OsStr,
    forwarded: &mut impl Write,
) -> Result<i32, CorrectnessError> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut capture = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)?;
    let output = native_command(cargo, arguments)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output()?;
    // The same original bytes feed both consumers, including unsuccessful Cargo
    // output. Neither a capture failure nor a pipe failure skips the other write.
    let captured = capture
        .write_all(&output.stdout)
        .and_then(|()| capture.sync_all());
    let sent = forwarded
        .write_all(&output.stdout)
        .and_then(|()| forwarded.flush());
    match (captured, sent) {
        (Ok(()), Ok(())) => Ok(exit_status(output.status)),
        (Err(capture), Err(forward)) => Err(invalid(format!(
            "Cargo metadata exited {}; private capture failed: {capture}; stdout forwarding also failed: {forward}",
            output.status
        ))),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error.into()),
    }
}
fn reject_reuse(arguments: &[OsString]) -> Result<(), CorrectnessError> {
    for argument in arguments {
        let text = argument
            .to_str()
            .ok_or_else(|| invalid("nextest control arguments must be UTF-8"))?;
        let key = text.split('=').next().unwrap_or(text);
        if [
            "--manifest-path",
            "--config-file",
            "--tool-config-file",
            "--archive-file",
            "--binaries-metadata",
            "--cargo-metadata",
            "--workspace-remap",
            "--target-dir-remap",
            "--build-dir-remap",
            "--archive-format",
            "--extract-to",
            "--extract-overwrite",
            "--persist-extract-tempdir",
            "--reuse-build",
            "--rerun",
            "--unit-graph",
        ]
        .contains(&key)
            || text.starts_with("-R")
        {
            return Err(invalid(format!(
                "nextest {key} bypasses ordinary source-build validation and is not supported"
            )));
        }
    }
    Ok(())
}
fn executable_checkout_root() -> Result<PathBuf, CorrectnessError> {
    let executable = std::env::current_exe()?;
    Ok(executable
        .ancestors()
        .find(|path| path.join(".config/diagnostic-tools.toml").is_file())
        .ok_or_else(|| invalid("native executable is outside the supported checkout"))?
        .canonicalize()?)
}
fn require_nextest_workspace(cargo: &OsStr) -> Result<(), CorrectnessError> {
    let root = executable_checkout_root()?;
    let output = capture(
        cargo,
        &[
            "locate-project".into(),
            "--workspace".into(),
            "--message-format".into(),
            "plain".into(),
        ],
    )?;
    let manifest = std::str::from_utf8(&output)
        .map_err(|_| invalid("Cargo workspace path must be UTF-8"))?
        .trim();
    let selected = PathBuf::from(manifest).canonicalize()?;
    if selected.parent() != Some(root.as_path()) {
        return Err(invalid(
            "native Nextest supports only its executable's checkout workspace; foreign workspace cannot preserve required setup",
        ));
    }
    Ok(())
}
/// Restore native Cargo in actual test attempts through nextest's single-run setup surface.
pub fn restore_test_environment() -> Result<i32, CorrectnessError> {
    let Some(cargo) = std::env::var_os(REAL_CARGO) else {
        return Ok(0);
    };
    let cargo = cargo
        .to_str()
        .ok_or_else(|| invalid("nextest environment file requires a UTF-8 Cargo path"))?;
    if cargo.contains(['\n', '\r', '\0']) {
        return Err(invalid(
            "Cargo path cannot be represented in nextest environment file",
        ));
    }
    let path = std::env::var_os("NEXTEST_ENV")
        .ok_or_else(|| invalid("nextest setup environment destination is absent"))?;
    let mut output = std::fs::OpenOptions::new().append(true).open(path)?;
    writeln!(output, "CARGO={cargo}\n{ADAPTER}=0")?;
    Ok(0)
}
fn execute_adapter(
    cargo: &OsStr,
    arguments: &[OsString],
    metadata_capture: Option<&OsStr>,
) -> Result<i32, CorrectnessError> {
    // cargo-hack 0.6.45 requests -vV before metadata and per-manifest checks.
    // Discovery can be forwarded; the marker never authorizes compiler work.
    if arguments == [OsString::from("--version")]
        || arguments == [OsString::from("-V")]
        || arguments == [OsString::from("-vV")]
    {
        return execute(cargo, arguments);
    }
    let position = mode_position(arguments)?;
    if arguments[position] == "metadata" {
        if let Some(destination) = metadata_capture {
            return execute_metadata(cargo, arguments, destination, &mut io::stdout().lock());
        }
        return execute(cargo, arguments);
    }
    if arguments[position] == "locate-project" {
        return execute(cargo, arguments);
    }
    let resolution = resolve_with(arguments, |args| capture(cargo, args))?;
    if let Some(directory) = std::env::var_os(BUILD_CAPTURE) {
        return crate::discovery::capture::execute_build(
            native_command(cargo, &resolution.arguments),
            &resolution,
            Path::new(&directory),
            &mut io::stdout().lock(),
        );
    }
    execute(cargo, &resolution.arguments)
}
/// Native external Cargo command and invocation-local nextest build adapter.
/// Report no-effects only until this owner actually starts an external child. Direct
/// child status alone does not establish settlement of exported adapter descendants.
pub fn run(
    arguments: Vec<OsString>,
    settlement: &mut crate::generation::EntrySettlement,
) -> Result<i32, CorrectnessError> {
    *settlement = crate::generation::EntrySettlement::NoEffects;
    let mut arguments = arguments;
    if arguments
        .first()
        .is_some_and(|arg| arg == "pse-correctness")
    {
        arguments.remove(0);
    }
    if arguments
        .first()
        .is_some_and(|arg| arg == "--nextest-test-env")
    {
        return restore_test_environment();
    }
    let cargo_adapter = std::env::var_os(ADAPTER).is_some_and(|value| value == "1")
        || std::env::var_os(CARGO_HACK_ADAPTER).is_some_and(|value| !value.is_empty());
    if arguments.as_slice() == [OsString::from("--version")] && !cargo_adapter {
        writeln!(
            io::stdout(),
            "cargo-pse-correctness {} protocol 1",
            env!("CARGO_PKG_VERSION")
        )?;
        return Ok(0);
    }
    let cargo = real_cargo()?;
    if cargo_adapter {
        *settlement = crate::generation::EntrySettlement::Unresolved;
        return execute_adapter(
            &cargo,
            &arguments,
            std::env::var_os(FIXTURE_METADATA_CAPTURE).as_deref(),
        );
    }
    if arguments.is_empty() {
        crate::cli::validate_correctness(&[OsString::from("--help")])?;
        return Ok(0);
    }
    if !crate::cli::validate_correctness(&arguments)? {
        return Ok(0);
    }
    if arguments.first().is_some_and(|arg| arg == "nextest") {
        if !arguments
            .get(1)
            .is_some_and(|arg| arg == "run" || arg == "list")
        {
            return Err(invalid(
                "only nextest run/list source-build modes are supported",
            ));
        }
        reject_reuse(&arguments[2..])?;
        *settlement = crate::generation::EntrySettlement::Unresolved;
        require_nextest_workspace(&cargo)?;
        let executable = std::env::current_exe()?;
        return Ok(exit_status(
            Command::new("cargo-nextest")
                .args(&arguments)
                .env("CARGO", &executable)
                .env("PSE_CORRECTNESS_EXECUTABLE", &executable)
                .env(REAL_CARGO, &cargo)
                .env(ADAPTER, "1")
                .env_remove("NEXTEST_TEST_PHASE")
                .status()?,
        ));
    }
    let resolution = resolve_with(&arguments, |args| {
        *settlement = crate::generation::EntrySettlement::Unresolved;
        capture(&cargo, args)
    })?;
    *settlement = crate::generation::EntrySettlement::Unresolved;
    execute(&cargo, &resolution.arguments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn resolution_schema_preserves_non_utf8_operating_system_arguments() {
        use std::os::unix::ffi::OsStringExt;
        let resolution = Resolution {
            arguments: vec![OsString::from_vec(vec![b'-', 0xff])],
            packages: vec![],
            units: vec![],
            native_capabilities: BTreeSet::new(),
        };
        let value = serde_json::to_value(&resolution).unwrap();
        assert_eq!(value["arguments"][0], json!({"Unix":[45,255]}));
        let schema = schemars::generate::SchemaSettings::default()
            .for_serialize()
            .into_generator()
            .into_root_schema_for::<Resolution>();
        let validator = jsonschema::validator_for(schema.as_value()).unwrap();
        assert!(validator.is_valid(&value));
        let mut lossy = value.clone();
        lossy["arguments"][0] = json!("lossy string");
        assert!(!validator.is_valid(&lossy));
        let restored: Resolution = serde_json::from_value(value).unwrap();
        assert_eq!(restored.arguments, resolution.arguments);
    }
    #[test]
    fn usage_failure_reports_no_generation_resource_effects() {
        let mut settlement = crate::generation::EntrySettlement::Unresolved;
        assert!(run(args(&["--unknown-source-build-mode"]), &mut settlement).is_err());
        assert_eq!(settlement, crate::generation::EntrySettlement::NoEffects);
    }
    #[test]
    fn fixture_metadata_capture_forwards_exact_original_bytes_and_exit()
    -> Result<(), CorrectnessError> {
        use std::os::unix::fs::PermissionsExt as _;
        let private = tempfile::tempdir()?;
        let destination = private.path().join("cargo-metadata.json");
        let mut forwarded = Vec::new();
        let code = execute_metadata(
            OsStr::new("/bin/sh"),
            &args(&["-c", "printf 'original metadata\\n'; exit 7"]),
            destination.as_os_str(),
            &mut forwarded,
        )?;
        assert_eq!(code, 7, "a private capture does not promote Cargo success");
        assert_eq!(forwarded, b"original metadata\n");
        assert_eq!(std::fs::read(&destination)?, forwarded);
        assert_eq!(
            std::fs::metadata(&destination)?.permissions().mode() & 0o777,
            0o600
        );
        Ok(())
    }
    #[test]
    fn fixture_metadata_capture_refuses_existing_file_or_symlink_before_query()
    -> Result<(), CorrectnessError> {
        let private = tempfile::tempdir()?;
        let destination = private.path().join("cargo-metadata.json");
        std::fs::write(&destination, b"original private capture")?;
        let mut forwarded = Vec::new();
        assert!(
            execute_metadata(
                OsStr::new("/bin/sh"),
                &args(&["-c", "printf must-not-run"]),
                destination.as_os_str(),
                &mut forwarded
            )
            .is_err()
        );
        assert!(forwarded.is_empty());
        assert_eq!(std::fs::read(&destination)?, b"original private capture");
        let link = private.path().join("link.json");
        std::os::unix::fs::symlink(&destination, &link)?;
        assert!(
            execute_metadata(
                OsStr::new("/bin/sh"),
                &args(&["-c", "printf must-not-run"]),
                link.as_os_str(),
                &mut forwarded
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&destination)?, b"original private capture");
        assert!(std::fs::symlink_metadata(link)?.is_symlink());
        Ok(())
    }
    #[test]
    fn fixture_metadata_capture_survives_forwarding_failure_without_claiming_success()
    -> Result<(), CorrectnessError> {
        struct Refused;
        impl Write for Refused {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("fixture closed output"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let private = tempfile::tempdir()?;
        let destination = private.path().join("cargo-metadata.json");
        let result = execute_metadata(
            OsStr::new("/bin/sh"),
            &args(&["-c", "printf exact-original"]),
            destination.as_os_str(),
            &mut Refused,
        );
        assert!(result.is_err());
        assert_eq!(std::fs::read(destination)?, b"exact-original");
        Ok(())
    }
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }
    fn package(id: &str, name: &str, features: Value) -> Value {
        json!({"id":id,"name":name,"features":features})
    }
    fn unit(
        id: &str,
        features: &[&str],
        dependencies: &[usize],
        platform: Option<&str>,
        mode: &str,
    ) -> Value {
        json!({"pkg_id":id,"target":{"name":id,"kind":["lib"],"src_path":format!("/{id}/lib.rs")},"features":features,"dependencies":dependencies.iter().map(|i|json!({"index":i})).collect::<Vec<_>>(),"platform":platform,"mode":mode})
    }
    fn graph(units: Vec<Value>) -> Value {
        json!({"version":1,"roots":[0],"units":units})
    }
    fn resolve(
        arguments: &[OsString],
        responses: Vec<Value>,
    ) -> Result<(Resolution, Vec<Vec<OsString>>), CorrectnessError> {
        let mut responses = responses.into_iter();
        let mut calls = Vec::new();
        let result = resolve_with(arguments, |call| {
            calls.push(call.to_vec());
            Ok(
                serde_json::to_vec(&responses.next().expect("unexpected Cargo request"))
                    .expect("JSON fixture"),
            )
        })?;
        assert!(
            responses.next().is_none(),
            "all expected graph requests made"
        );
        Ok((result, calls))
    }
    fn arrow_metadata() -> Value {
        json!({"packages":[package("root","root",json!({"force-validate":[]})),package("arrow-array","arrow-array",json!({"force_validate":[]}))]})
    }
    #[test]
    fn reached_backend_contexts_join_metadata_by_id_and_preserve_host_target_units() {
        let mut backend = package("backend-id", "pse-backend-native", json!({}));
        backend["metadata"] =
            json!({"pse":{"native-capabilities":{"uno":["uno"],"idas":["klu"],"scip":["solver"]}}});
        let metadata = json!({"packages":[package("root", "root", json!({})), backend]});
        let graph = graph(vec![
            unit("root", &[], &[1, 2], Some("target-triple"), "check"),
            unit("backend-id", &["uno"], &[], None, "build"),
            unit("backend-id", &["idas"], &[], Some("target-triple"), "check"),
            unit("backend-id", &["scip"], &[], None, "build"),
        ]);
        let (actual, _) = resolve(&args(&["check", "-p", "root"]), vec![metadata, graph]).unwrap();
        assert_eq!(
            actual.native_capabilities,
            ["compiler", "solver", "klu", "uno"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
        assert_eq!(
            actual.units.len(),
            3,
            "unreached units do not enter selection"
        );
        assert_eq!(actual.units[1].package_id, "backend-id");
        assert_eq!(actual.units[1].platform, None);
        assert_eq!(actual.units[1].features, BTreeSet::from(["uno".into()]));
        assert_eq!(actual.units[2].platform.as_deref(), Some("target-triple"));
        assert_eq!(actual.units[2].features, BTreeSet::from(["idas".into()]));
        assert!(!actual.units[1].root);
    }
    #[test]
    fn cargo_hack_discovery_preserves_literal_requests_and_native_failure_without_compiler_bypass()
    {
        use std::os::unix::fs::PermissionsExt as _;
        let directory = tempfile::tempdir().unwrap();
        let cargo = directory.path().join("fake-cargo");
        std::fs::write(&cargo,b"#!/bin/sh\nprintf '%s\\0' \"$@\" >> \"${0}.calls\"\ncase \"$1\" in\n-vV) exit 13;;\nmetadata) exit 17;;\nlocate-project) exit 19;;\n*) exit 77;;\nesac\n").unwrap();
        std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            execute_adapter(cargo.as_os_str(), &args(&["-vV"]), None).unwrap(),
            13
        );
        assert_eq!(
            execute_adapter(
                cargo.as_os_str(),
                &args(&["metadata", "--format-version", "1"]),
                None
            )
            .unwrap(),
            17
        );
        assert_eq!(
            execute_adapter(cargo.as_os_str(), &args(&["locate-project"]), None).unwrap(),
            19
        );
        assert_eq!(
            std::fs::read(directory.path().join("fake-cargo.calls")).unwrap(),
            b"-vV\0metadata\0--format-version\x001\0locate-project\0"
        );
        assert!(
            execute_adapter(
                cargo.as_os_str(),
                &args(&["check", "--no-default-features"]),
                None
            )
            .is_err(),
            "compiler callbacks must still establish Cargo graph coverage"
        );
    }
    #[test]
    fn cargo_hack_selected_manifest_features_keep_target_host_graph_and_literal_controls() {
        let directory = tempfile::tempdir().unwrap();
        let manifest = directory.path().join("selected manifest; literal.toml");
        std::fs::write(&manifest, "Cargo metadata is the package-name owner").unwrap();
        let mut metadata = arrow_metadata();
        metadata["packages"][0]["name"] = json!("selected-package");
        metadata["packages"][0]["manifest_path"] = json!(manifest);
        let before = graph(vec![
            unit("root", &[], &[1, 2], Some("custom-target"), "check"),
            unit("arrow-array", &[], &[], Some("custom-target"), "build"),
            unit("arrow-array", &[], &[], None, "build"),
        ]);
        let after = graph(vec![
            unit(
                "root",
                &["force-validate"],
                &[1, 2],
                Some("custom-target"),
                "check",
            ),
            unit(
                "arrow-array",
                &["force_validate"],
                &[],
                Some("custom-target"),
                "build",
            ),
            unit("arrow-array", &["force_validate"], &[], None, "build"),
        ]);
        for (features, qualified) in [
            (
                args(&["--features", "native,default pse-relations/force-validate"]),
                args(&[
                    "--features",
                    "selected-package/native,selected-package/default,pse-relations/force-validate",
                ]),
            ),
            (
                args(&["--features=native"]),
                args(&["--features=selected-package/native"]),
            ),
            (
                args(&["-F", "native"]),
                args(&["-F", "selected-package/native"]),
            ),
            (args(&["-Fnative"]), args(&["-Fselected-package/native"])),
        ] {
            let mut original = args(&[
                "check",
                "--locked",
                "--no-default-features",
                "--tests",
                "--target",
                "custom-target",
                "--config",
                "target.custom-target.rustflags=['--cfg=original']",
                "--manifest-path",
            ]);
            original.push(manifest.clone().into_os_string());
            original.extend(features);
            let (resolution, calls) = resolve(
                &original,
                vec![metadata.clone(), before.clone(), after.clone()],
            )
            .unwrap();
            let mut expected = original[..10].to_vec();
            expected.extend(qualified);
            let qualified_before = expected.clone();
            expected.extend(args(&["--features", "selected-package/force-validate"]));
            assert_eq!(resolution.arguments, expected);
            assert_eq!(resolution.packages, ["selected-package"]);
            assert_eq!(
                calls.len(),
                3,
                "one invocation-local metadata capture and the original/validated unit graphs"
            );
            assert_eq!(calls[1], graph_arguments(&qualified_before).unwrap());
            assert_eq!(calls[2], graph_arguments(&resolution.arguments).unwrap());
        }
    }
    #[test]
    fn cargo_hack_no_default_scope_does_not_add_unrelated_features_or_targets() {
        let original = args(&[
            "check",
            "--manifest-path=literal-selection/Cargo.toml",
            "--locked",
            "--no-default-features",
            "--lib",
            "--target=target",
        ]);
        let (resolution, calls) = resolve(
            &original,
            vec![
                json!({"packages":[package("pure","pure",json!({}))]}),
                graph(vec![unit("pure", &[], &[], Some("target"), "check")]),
            ],
        )
        .unwrap();
        assert_eq!(resolution.arguments, original);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1], graph_arguments(&original).unwrap());
    }
    #[test]
    fn cargo_hack_bare_features_require_the_actual_selected_metadata_package() {
        let directory = tempfile::tempdir().unwrap();
        let manifest = directory.path().join("Cargo.toml");
        std::fs::write(&manifest, "selected manifest").unwrap();
        let original = vec![
            "check".into(),
            "--manifest-path".into(),
            manifest.into_os_string(),
            "-Fbare".into(),
        ];
        let error = resolve(&original, vec![arrow_metadata()]).unwrap_err();
        assert!(error.to_string().contains("absent from Cargo metadata"));
    }
    #[test]
    fn cargo_hack_taskdump_uses_same_cargo_cfg_for_dependency_queries_and_execution() {
        let original = args(&[
            "check",
            "--features=selected/diagnostic-taskdump",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--config",
            "target.x86_64-unknown-linux-gnu.rustflags=['--cfg=original']",
            "--profile=diagnostic-taskdump",
        ]);
        let (resolution, calls) = resolve(
            &original,
            vec![
                json!({"workspace_root":executable_checkout_root().unwrap(),"packages":[package("pure","pure",json!({}))]}),
                graph(vec![unit(
                    "pure",
                    &[],
                    &[],
                    Some("x86_64-unknown-linux-gnu"),
                    "check",
                )]),
            ],
        )
        .unwrap();
        assert_eq!(&resolution.arguments[..original.len()], original.as_slice());
        for call in calls.iter().chain(std::iter::once(&resolution.arguments)) {
            assert!(call.iter().any(
                |value| value == "target.x86_64-unknown-linux-gnu.rustflags=['--cfg=original']"
            ));
            assert!(call.iter().any(|value| value == DIAGNOSTIC_CFG));
            assert!(!call.iter().any(|value| {
                value.to_string_lossy().contains("profile-rustflags")
                    || value
                        .to_string_lossy()
                        .contains("profile.diagnostic-taskdump.rustflags")
            }));
        }
        let command =
            native_command_with_rustflags(OsStr::new("cargo"), &resolution.arguments, None, None);
        assert!(
            !command
                .get_envs()
                .any(|(key, _)| key == "RUSTFLAGS" || key == "CARGO_ENCODED_RUSTFLAGS"),
            "Cargo retains ownership of inherited flags and target configuration"
        );
    }
    #[test]
    fn diagnostic_environment_preserves_literal_flags_and_highest_active_channel() {
        let selected =
            diagnostic_check_arguments(&args(&["check", "-Fdiagnostic-taskdump"])).unwrap();
        for (encoded, raw, key, expected) in [
            (
                Some("-C\x1flink-arg=literal space\x1f--cfg=original"),
                Some("  --cfg other  "),
                "CARGO_ENCODED_RUSTFLAGS",
                "-C\x1flink-arg=literal space\x1f--cfg=original\x1f--cfg\x1ftokio_unstable",
            ),
            (
                Some(""),
                Some("--cfg ignored"),
                "CARGO_ENCODED_RUSTFLAGS",
                "--cfg\x1ftokio_unstable",
            ),
            (
                None,
                Some("  --cfg original -Clink-arg=literal  "),
                "RUSTFLAGS",
                "  --cfg original -Clink-arg=literal   --cfg tokio_unstable",
            ),
            (None, Some(""), "RUSTFLAGS", " --cfg tokio_unstable"),
        ] {
            // Cargo metadata, the validated unit graph and compilation share
            // exactly the same child-local extension, without parsing flags.
            for arguments in [
                metadata_arguments(&selected).unwrap(),
                graph_arguments(&selected).unwrap(),
                selected.clone(),
            ] {
                let command = native_command_with_rustflags(
                    OsStr::new("cargo"),
                    &arguments,
                    encoded.map(Into::into),
                    raw.map(Into::into),
                );
                let changed: Vec<_> = command
                    .get_envs()
                    .filter(|(name, _)| *name == "RUSTFLAGS" || *name == "CARGO_ENCODED_RUSTFLAGS")
                    .collect();
                assert_eq!(changed, [(OsStr::new(key), Some(OsStr::new(expected)))]);
                assert_eq!(
                    command.get_args().collect::<Vec<_>>(),
                    arguments
                        .iter()
                        .map(OsString::as_os_str)
                        .collect::<Vec<_>>()
                );
            }
        }
        let command = native_command_with_rustflags(OsStr::new("cargo"), &selected, None, None);
        assert!(
            !command
                .get_envs()
                .any(|(name, _)| name == "RUSTFLAGS" || name == "CARGO_ENCODED_RUSTFLAGS"),
            "without an environment channel Cargo joins the checkout's existing tuple/linker flags and cfg table"
        );
        let ordinary = args(&["check", "--", DIAGNOSTIC_CFG]);
        let command = native_command_with_rustflags(
            OsStr::new("cargo"),
            &ordinary,
            Some("original encoded".into()),
            Some("original raw".into()),
        );
        assert!(
            !command
                .get_envs()
                .any(|(name, _)| name == "RUSTFLAGS" || name == "CARGO_ENCODED_RUSTFLAGS")
        );
        assert_eq!(
            diagnostic_check_arguments(&selected).unwrap(),
            selected,
            "repeated scope selection must not duplicate the Cargo cfg table or profile"
        );
    }
    #[test]
    fn diagnostic_foreign_targets_refuse_before_compilation() {
        for target in [
            args(&["--target", "aarch64-unknown-linux-gnu"]),
            args(&["--target=x86_64-unknown-linux-musl"]),
        ] {
            let mut arguments = args(&["check", "-Fdiagnostic-taskdump"]);
            arguments.extend(target);
            assert!(
                diagnostic_check_arguments(&arguments)
                    .unwrap_err()
                    .to_string()
                    .contains("only native x86_64-unknown-linux-gnu")
            );
        }
        // Cargo's unit graph also exposes implicit build.target/environment
        // selection, without this owner reading a second configuration file.
        let metadata = json!({"workspace_root":executable_checkout_root().unwrap(),"packages":[package("root","root",json!({"diagnostic-taskdump":[]}))]});
        let foreign = graph(vec![unit(
            "root",
            &["diagnostic-taskdump"],
            &[],
            Some("foreign-target"),
            "check",
        )]);
        let error =
            resolve(&args(&["check", "--all-features"]), vec![metadata, foreign]).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("only native x86_64-unknown-linux-gnu")
        );
    }
    #[test]
    fn diagnostic_marker_in_literal_application_tail_does_not_restrict_ordinary_run() {
        let foreign = tempfile::tempdir().unwrap();
        let metadata =
            json!({"workspace_root":foreign.path(), "packages":[package("root","root",json!({}))]});
        let arguments = args(&["run", "--bin", "root", "--", DIAGNOSTIC_CFG]);
        let (resolution, calls) = resolve(
            &arguments,
            vec![
                metadata,
                graph(vec![unit(
                    "root",
                    &[],
                    &[],
                    Some("foreign-target"),
                    "build",
                )]),
            ],
        )
        .unwrap();
        assert_eq!(resolution.arguments, arguments);
        assert_eq!(calls.len(), 2);
    }
    #[test]
    fn diagnostic_checkout_identity_uses_original_metadata_and_refuses_before_compilation() {
        let foreign = tempfile::tempdir().unwrap();
        for workspace in [None, Some(foreign.path())] {
            let mut metadata =
                json!({"packages":[package("root","root",json!({"diagnostic-taskdump":[]}))]});
            if let Some(workspace) = workspace {
                metadata["workspace_root"] = json!(workspace);
            }
            for resolved in [false, true] {
                let arguments = if resolved {
                    args(&["check", "--all-features"])
                } else {
                    args(&["check", "-Fdiagnostic-taskdump"])
                };
                let mut calls = Vec::new();
                let error = resolve_with(&arguments, |call| {
                    calls.push(call.to_vec());
                    assert!(calls.len() <= if resolved { 2 } else { 1 },
                        "foreign/missing workspace must refuse before the diagnostic compiler graph or execution");
                    let response = if calls.len() == 1 {
                        metadata.clone()
                    } else {
                        graph(vec![unit("root", &["diagnostic-taskdump"], &[], None, "check")])
                    };
                    Ok(serde_json::to_vec(&response).unwrap())
                }).unwrap_err();
                assert_eq!(calls.len(), if resolved { 2 } else { 1 });
                assert_eq!(
                    calls.iter().filter(|call| call[0] == "metadata").count(),
                    1,
                    "checkout admission consumes the existing complete metadata capture"
                );
                assert!(error.to_string().contains(if workspace.is_some() {
                    "only its executable's checkout workspace"
                } else {
                    "metadata.workspace_root"
                }));
            }
            let ordinary = args(&["check"]);
            let (resolution, calls) = resolve(
                &ordinary,
                vec![metadata, graph(vec![unit("root", &[], &[], None, "check")])],
            )
            .unwrap();
            assert_eq!(
                resolution.arguments, ordinary,
                "ordinary correctness does not impose the diagnostic checkout restriction"
            );
            assert_eq!(calls.len(), 2);
        }
    }
    #[test]
    fn cargo_hack_taskdump_profile_conflicts_refuse_and_other_commands_stay_literal() {
        for suffix in [
            args(&["--release"]),
            args(&["-r"]),
            args(&["--profile", "dev"]),
            args(&["--profile=release"]),
        ] {
            let mut selected = args(&["check", "-Fdiagnostic-taskdump"]);
            selected.extend(suffix);
            assert!(
                diagnostic_check_arguments(&selected)
                    .unwrap_err()
                    .to_string()
                    .contains("isolated diagnostic-taskdump profile")
            );
        }
        for original in [
            args(&["check", "--features=other/diagnostic-taskdump-extra"]),
            args(&["check", "--", "-Fdiagnostic-taskdump"]),
            args(&["build", "-Fdiagnostic-taskdump"]),
        ] {
            assert_eq!(diagnostic_check_arguments(&original).unwrap(), original);
        }
        let selected =
            diagnostic_check_arguments(&args(&["check", "-Fdiagnostic-taskdump"])).unwrap();
        assert!(
            selected
                .windows(2)
                .any(|pair| pair == args(&["--profile", "diagnostic-taskdump"]))
        );
    }
    #[test]
    fn cargo_hack_all_feature_check_and_clippy_isolate_only_actual_reached_taskdump() {
        let metadata = json!({"workspace_root":executable_checkout_root().unwrap(),"packages":[package("root","root",json!({"diagnostic-taskdump":[]})),package("other","other",json!({"diagnostic-taskdump":[]}))]});
        let original_graph = graph(vec![unit(
            "root",
            &["diagnostic-taskdump"],
            &[],
            Some("x86_64-unknown-linux-gnu"),
            "check",
        )]);
        for mode in ["check", "clippy"] {
            let mut original = args(&[
                mode,
                "--workspace",
                "--all-features",
                "--all-targets",
                "--target",
                "x86_64-unknown-linux-gnu",
                "--config",
                "target.x86_64-unknown-linux-gnu.rustflags=['--cfg=original']",
            ]);
            if mode == "clippy" {
                original.push("--no-deps".into());
            }
            let (resolution, calls) = resolve(
                &original,
                vec![
                    metadata.clone(),
                    original_graph.clone(),
                    original_graph.clone(),
                ],
            )
            .unwrap();
            assert_eq!(&resolution.arguments[..original.len()], original.as_slice());
            assert!(
                resolution
                    .arguments
                    .windows(2)
                    .any(|pair| pair == args(&["--profile", "diagnostic-taskdump"]))
            );
            assert!(
                resolution
                    .arguments
                    .iter()
                    .any(|value| value == DIAGNOSTIC_CFG)
            );
            assert_eq!(calls.len(), 3);
            assert_eq!(
                calls
                    .iter()
                    .filter(|call| call.first().is_some_and(|value| value == "metadata"))
                    .count(),
                1
            );
            assert_eq!(calls[2], graph_arguments(&resolution.arguments).unwrap());
            if mode == "clippy" {
                assert!(
                    resolution
                        .arguments
                        .iter()
                        .any(|value| value == "--no-deps")
                );
                assert!(!calls[2].iter().any(|value| value == "--no-deps"));
            }
            let pure = graph(vec![unit(
                "root",
                &[],
                &[],
                Some("x86_64-unknown-linux-gnu"),
                "check",
            )]);
            let (unrelated, calls) = resolve(&original, vec![metadata.clone(), pure]).unwrap();
            assert_eq!(
                unrelated.arguments, original,
                "an unrelated metadata feature does not select diagnostic isolation"
            );
            assert_eq!(calls.len(), 2);
        }
    }
    #[test]
    fn cargo_hack_resolved_taskdump_refuses_conflicting_profile_or_changed_target_roots() {
        let metadata = json!({"workspace_root":executable_checkout_root().unwrap(),"packages":[package("root","root",json!({"diagnostic-taskdump":[]}))]});
        let before = graph(vec![unit(
            "root",
            &["diagnostic-taskdump"],
            &[],
            None,
            "check",
        )]);
        for mode in ["check", "clippy"] {
            let error = resolve(
                &args(&[mode, "--all-features", "--profile=dev"]),
                vec![metadata.clone(), before.clone()],
            )
            .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("isolated diagnostic-taskdump profile")
            );
        }
        let mut changed = before.clone();
        changed["units"][0]["target"]["name"] = json!("different-target");
        let error = resolve(
            &args(&["check", "--all-features"]),
            vec![metadata, before, changed],
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("isolation changed selected Cargo package/target roots")
        );
    }
    #[test]
    fn correctness_pure_scope_is_unchanged() {
        let original = args(&["test", "-p", "pure", "--", "--exact", "filter"]);
        let (resolution, calls) = resolve(
            &original,
            vec![
                json!({"packages":[package("pure","pure",json!({}))]}),
                graph(vec![unit("pure", &[], &[], None, "test")]),
            ],
        )
        .unwrap();
        assert_eq!(resolution.arguments, original);
        assert_eq!(resolution.packages, ["pure"]);
        assert_eq!(calls.len(), 2);
        assert!(!calls[1].iter().any(|v| v == "--exact"));
    }
    #[test]
    fn correctness_only_selected_root_gets_features_and_tail_survives() {
        let original = args(&["test", "-p", "root", "--", "--exact", "a"]);
        let before = graph(vec![
            unit("root", &[], &[1], None, "test"),
            unit("arrow-array", &[], &[], None, "build"),
        ]);
        let after = graph(vec![
            unit("root", &["force-validate"], &[1], None, "test"),
            unit("arrow-array", &["force_validate"], &[], None, "build"),
        ]);
        let (resolution, calls) =
            resolve(&original, vec![arrow_metadata(), before, after]).unwrap();
        assert_eq!(
            resolution.arguments,
            args(&[
                "test",
                "-p",
                "root",
                "--features",
                "root/force-validate",
                "--",
                "--exact",
                "a"
            ])
        );
        assert_eq!(calls.len(), 3);
        assert!(calls[2].iter().any(|v| v == "root/force-validate"));
    }
    #[test]
    fn correctness_host_build_leaf_is_independently_validated() {
        let graph = graph(vec![
            unit("root", &["force-validate"], &[1, 2], Some("target"), "test"),
            unit(
                "arrow-array",
                &["force_validate"],
                &[],
                Some("target"),
                "build",
            ),
            unit("arrow-array", &[], &[], None, "build"),
        ]);
        let error = resolve(
            &args(&["test", "-p", "root"]),
            vec![arrow_metadata(), graph],
        )
        .unwrap_err();
        assert!(error.to_string().contains("None build unit"));
    }
    #[test]
    fn correctness_missing_root_opt_in_refuses() {
        let metadata = json!({"packages":[package("root","root",json!({})),package("arrow-array","arrow-array",json!({"force_validate":[]}))]});
        let error = resolve(
            &args(&["check"]),
            vec![
                metadata,
                graph(vec![
                    unit("root", &[], &[1], None, "build"),
                    unit("arrow-array", &[], &[], None, "build"),
                ]),
            ],
        )
        .unwrap_err();
        assert!(error.to_string().contains("no explicit force-validate"));
    }
    #[test]
    fn correctness_changed_root_refuses() {
        let before = graph(vec![
            unit("root", &[], &[1], None, "build"),
            unit("arrow-array", &[], &[], None, "build"),
        ]);
        let mut after = before.clone();
        after["units"][0]["target"]["name"] = json!("changed");
        let error = resolve(&args(&["check"]), vec![arrow_metadata(), before, after]).unwrap_err();
        assert!(error.to_string().contains("changed selected"));
    }
    #[test]
    fn correctness_unknown_graph_or_edge_refuses() {
        for graph in [
            json!({"version":2,"roots":[],"units":[]}),
            graph(vec![unit("root", &[], &[9], None, "build")]),
            graph(vec![unit("unknown", &[], &[], None, "build")]),
        ] {
            assert!(resolve(&args(&["check"]), vec![arrow_metadata(), graph]).is_err());
        }
    }
    #[test]
    fn correctness_reused_nextest_bypasses_refuse() {
        for option in [
            "--archive-file=a",
            "--binaries-metadata",
            "--cargo-metadata=a",
            "--workspace-remap=a",
            "--reuse-build",
            "--rerun",
            "-Rlast",
            "--unit-graph",
        ] {
            assert!(reject_reuse(&args(&[option])).is_err(), "{option}");
        }
        assert!(reject_reuse(&args(&["-E", "test(a)", "--test-threads", "16"])).is_ok());
    }
    #[test]
    fn correctness_direct_graph_adapters_preserve_explicit_scope() {
        assert!(graph_arguments(&args(&["run"])).is_err());
        assert_eq!(
            graph_arguments(&args(&["run", "--bin", "tool", "--", "arg"])).unwrap(),
            args(&[
                "build",
                "--bin",
                "tool",
                "--unit-graph",
                "-Z",
                "unstable-options"
            ])
        );
        assert_eq!(
            graph_arguments(&args(&[
                "--color=always",
                "clippy",
                "-p",
                "root",
                "--",
                "-Dwarnings"
            ]))
            .unwrap()[1],
            "check"
        );
    }
}
