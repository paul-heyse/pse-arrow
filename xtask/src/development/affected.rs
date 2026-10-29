// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source/dependency reach preview; only explicit execution enters correctness admission.

use anyhow::{Context, Result, bail, ensure};
use cargo_metadata::{Metadata, MetadataCommand};
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::{BTreeMap, BTreeSet}, path::{Component, Path, PathBuf}, process::{Command, ExitStatus}};

const CATALOG: &str = "tests/fixtures/scientific-inputs.toml";
const WORKSPACE_FILES: &[&str] = &["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "clippy.toml", ".config/nextest.toml"];
const NATIVE_PACKAGES: &[&str] = &["pse-backend-native", "pse-runtime", "pse-ipopt-sys", "pse-uno-sys", "pse-petsc-sys"];

/// Input selection. Extra nextest arguments are passed unchanged to the original receiver.
#[derive(Clone, Debug)]
pub struct Options {
    /// Git commit/ref compared with the current working tree.
    pub base: String,
    /// Additional Cargo features, forwarded unchanged to correctness admission.
    pub features: String,
    /// Additional native nextest arguments; no second parser interprets them.
    pub extra: Vec<String>,
    /// An explicit change population replaces whole-worktree discovery.
    pub changes: Option<Vec<(String, String)>>,
}

impl Default for Options {
    fn default() -> Self { Self { base: "HEAD".into(), features: String::new(), extra: Vec::new(), changes: None } }
}

/// Disposable preview of source ownership and conservative dependency reach.
#[derive(Debug, Serialize)]
pub struct Preview {
    schema: &'static str,
    /// Selected Git comparison base.
    pub base: String,
    /// Observed or explicitly supplied status/path pairs.
    pub changes: Vec<(String, String)>,
    /// Source owners and conservatively selected scientific consumers.
    pub changed_packages: Vec<String>,
    /// Workspace packages reachable through declared reverse dependency edges.
    pub selected_packages: Vec<String>,
    /// Root configuration changes that broaden selection to the workspace.
    pub workspace_widening: Vec<String>,
    /// Authored scientific consumer leads and explicit unknown exact reach.
    pub scientific_inputs: Value,
    /// Proposed original command, absent when no executable selection is available.
    pub command: Option<Vec<String>>,
    /// Whether Cargo metadata was available, unavailable or unnecessary.
    pub graph_status: &'static str,
    /// Original metadata/ownership failure, when graph observation failed.
    pub graph_error: Option<String>,
    /// Correctness admission still required at actual execution.
    pub validation: &'static str,
    /// Feature, source-kind and unknown-behavior limits of the selection.
    pub exclusions: Vec<Value>,
    /// Bound on what this source preview establishes.
    pub qualification: &'static str,
}

#[derive(Default)]
struct Workspace {
    roots: BTreeMap<String, PathBuf>,
    dependents: BTreeMap<String, BTreeSet<String>>,
}

impl Workspace {
    fn from_metadata(root: &Path, metadata: &Metadata) -> Result<Self> {
        let mut workspace = Self::default();
        for package in &metadata.packages {
            if !metadata.workspace_members.contains(&package.id) { continue; }
            let directory = package.manifest_path.parent().context("package manifest has no parent")?.as_std_path();
            workspace.roots.insert(package.name.to_string(), directory.strip_prefix(root).with_context(|| format!("workspace member outside checkout: {}", directory.display()))?.to_owned());
        }
        ensure!(!workspace.roots.is_empty(), "Cargo metadata enumerated no workspace members");
        for package in &metadata.packages {
            if !metadata.workspace_members.contains(&package.id) { continue; }
            for dependency in &package.dependencies {
                // Include optional, build, dev and target-specific declarations: source reach
                // must not pretend to resolve the actual feature/target compiler closure.
                if workspace.roots.contains_key(&dependency.name) {
                    workspace.dependents.entry(dependency.name.clone()).or_default().insert(package.name.to_string());
                }
            }
        }
        Ok(workspace)
    }

    fn owner(&self, path: &str) -> Option<&str> {
        self.roots.iter().filter(|(_, directory)| !directory.as_os_str().is_empty() && Path::new(path).starts_with(directory)).max_by_key(|(_, directory)| directory.components().count()).map(|(name, _)| name.as_str())
    }

    fn closure(&self, seeds: &BTreeSet<String>) -> BTreeSet<String> {
        let mut found = seeds.clone();
        let mut frontier: Vec<_> = seeds.iter().cloned().collect();
        while let Some(name) = frontier.pop() {
            if let Some(dependents) = self.dependents.get(&name) {
                for dependent in dependents {
                    if found.insert(dependent.clone()) { frontier.push(dependent.clone()); }
                }
            }
        }
        found
    }
}

fn validate_changes(changes: &[(String, String)]) -> Result<()> {
    for (_, path) in changes {
        ensure!(!path.is_empty() && !path.contains('\0') && !Path::new(path).components().any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_))), "change paths must remain inside the selected checkout: {path:?}");
    }
    Ok(())
}

fn git_output(root: &Path, arguments: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").args(arguments).current_dir(root).output().context("observing changed sources with git")?;
    ensure!(output.status.success(), "git observation failed ({}): {}", output.status, String::from_utf8_lossy(&output.stderr));
    Ok(output.stdout)
}

/// Observe staged, unstaged, deleted and untracked files, preserving tabs/newlines in names.
/// # Errors
/// Git observation failed, the base is invalid or a pathname is not UTF-8.
pub fn changed_files(root: &Path, base: &str) -> Result<Vec<(String, String)>> {
    // Resolve first so a ref beginning with '-' cannot become a git diff option.
    let resolved = git_output(root, &["rev-parse", "--verify", "--end-of-options", &format!("{base}^{{commit}}")])?;
    let resolved = std::str::from_utf8(&resolved)?.trim();
    let diff = git_output(root, &["diff", "--name-status", "-z", "--no-renames", resolved, "--"])?;
    let mut changes = parse_diff(&diff)?;
    let untracked = git_output(root, &["ls-files", "-z", "--others", "--exclude-standard"])?;
    for path in nul_fields(&untracked)? { changes.push(("?".into(), path.to_owned())); }
    validate_changes(&changes)?;
    Ok(changes)
}

fn nul_fields(bytes: &[u8]) -> Result<Vec<&str>> {
    if bytes.is_empty() { return Ok(Vec::new()); }
    ensure!(bytes.last() == Some(&0), "git path output is not NUL-terminated");
    bytes[..bytes.len() - 1].split(|byte| *byte == 0).map(|field| Ok(std::str::from_utf8(field)?)).collect()
}

fn parse_diff(bytes: &[u8]) -> Result<Vec<(String, String)>> {
    let fields = nul_fields(bytes)?;
    ensure!(fields.len() % 2 == 0, "git name-status output has an incomplete entry");
    Ok(fields.chunks_exact(2).map(|pair| (pair[0].to_owned(), pair[1].to_owned())).collect())
}

fn scientific_reach(root: &Path, changes: &[(String, String)], workspace: &Workspace) -> (BTreeSet<String>, Value) {
    // This is the retained passive contract: observe only the authored inventory/consumer
    // declaration. Exact package membership/import meaning belongs to scientific_inputs.
    let catalog = std::fs::read_to_string(root.join(CATALOG)).map_err(anyhow::Error::from).and_then(|text| toml::from_str::<toml::Value>(&text).map_err(Into::into));
    let inventory = catalog.as_ref().ok().and_then(|value| value.get("package_inventory")).and_then(toml::Value::as_str).unwrap_or("packages/reference");
    let relevant: Vec<_> = changes.iter().map(|(_, path)| path).filter(|path| Path::new(path).starts_with(inventory) || Path::new(path).starts_with("tests/fixtures/scientific") || [CATALOG, "tests/fixtures/scientific-inputs.rs", "tests/fixtures/reference-source-fixture.rs", "xtask/src/scientific_inputs.rs"].contains(&path.as_str())).cloned().collect();
    if relevant.is_empty() { return (BTreeSet::new(), json!({"consumers": [], "families": [], "unknown": []})); }
    let consumers = (|| -> Result<BTreeSet<String>> {
        let catalog = catalog?;
        ensure!(catalog.get("version").and_then(toml::Value::as_integer) == Some(1), "unsupported scientific input catalog version");
        let families = catalog.get("families").and_then(toml::Value::as_array).context("scientific catalog has no family declarations")?;
        let mut consumers = BTreeSet::new();
        for family in families {
            for consumer in family.get("consumers").and_then(toml::Value::as_array).context("scientific family has no consumers")? {
                consumers.insert(consumer.as_str().context("scientific consumer is not a string")?.to_owned());
            }
        }
        ensure!(!consumers.is_empty(), "scientific input catalog has no consumers");
        Ok(consumers)
    })();
    match consumers {
        Ok(consumers) => {
            let detail = json!({"consumers": consumers, "families": [], "unknown": relevant, "reason": "Source-only preview conservatively selects declared family consumers; exact scientific closure is not prepared."});
            (consumers, detail)
        },
        Err(error) => {
            let consumers: BTreeSet<_> = workspace.roots.keys().cloned().collect();
            let detail = json!({"consumers": consumers, "families": [], "unknown": relevant, "reason": error.to_string(), "selection": "scientific reach unavailable; conservatively selects the observed workspace"});
            (consumers, detail)
        },
    }
}

fn assemble(root: &Path, options: &Options, changes: Vec<(String, String)>, graph: Result<Workspace>) -> Result<Preview> {
    validate_changes(&changes)?;
    let empty = changes.is_empty();
    let (workspace, graph_error) = match graph { Ok(workspace) => (workspace, None), Err(error) => (Workspace::default(), Some(format!("{error:#}"))) };
    let mut seeds = BTreeSet::new();
    let mut wide = Vec::new();
    let mut python = Vec::new();
    let mut generators = Vec::new();
    let mut unowned = Vec::new();
    for (status, path) in &changes {
        if WORKSPACE_FILES.contains(&path.as_str()) || Path::new(path).starts_with(".cargo") { wide.push(path.clone()); }
        if Path::new(path).starts_with("xtask/src/codegen") || ["xtask/src/codegen.rs", "xtask/src/scientific_inputs.rs"].contains(&path.as_str()) || Path::new(path).starts_with("crates/pse-codegen") || Path::new(path).starts_with("crates/pse-schema") { generators.push(path.clone()); }
        if Path::new(path).starts_with("python") || ["conftest.py", "pyproject.toml", "uv.lock"].contains(&path.as_str()) { python.push(path.clone()); }
        if let Some(name) = workspace.owner(path) { seeds.insert(name.to_owned()); }
        else if !Path::new(path).starts_with("python") && !Path::new(path).starts_with("docs") && !wide.contains(path) { unowned.push(format!("{status} {path}")); }
    }
    let (scientific, scientific_inputs) = scientific_reach(root, &changes, &workspace);
    seeds.extend(scientific.into_iter().filter(|name| workspace.roots.contains_key(name)));
    let selected = if wide.is_empty() { workspace.closure(&seeds) } else { workspace.roots.keys().cloned().collect() };
    let command = if graph_error.is_none() && (!seeds.is_empty() || !wide.is_empty()) { Some(selection_command(options, &seeds, &selected, !wide.is_empty())) } else { None };
    let native: Vec<_> = NATIVE_PACKAGES.iter().copied().filter(|name| selected.contains(*name)).collect();
    let mut exclusions = vec![
        json!({"kind": "feature-gated-native", "packages": native, "route": "--features pse-runtime/native-solvers; explicitly prepare the required native capabilities"}),
        json!({"kind": "python", "files": python, "meaning": "Rust dependency selection does not execute isolated Python/reference consumers"}),
        json!({"kind": "generator", "files": generators, "route": "just codegen; just codegen-check at scope end"}),
        json!({"kind": "unowned", "files": unowned}),
        json!({"kind": "dynamic", "meaning": "Source reach conservatively includes declared scientific consumers; external-process reach and exhaustive behavior coverage are not inferred"}),
    ];
    if let Some(error) = &graph_error { exclusions.push(json!({"kind": "workspace-graph", "meaning": "Cargo graph unavailable; no executable selection can be established", "reason": error})); }
    Ok(Preview {
        schema: "affected-preview-v1", base: options.base.clone(), changes,
        changed_packages: seeds.into_iter().collect(), selected_packages: selected.into_iter().collect(), workspace_widening: wide, scientific_inputs, command,
        graph_status: if empty { "not-needed" } else if graph_error.is_some() { "unavailable" } else { "available" }, graph_error,
        validation: if empty { "no selected changes" } else { "proposed route; Arrow closure admitted by its original receiver at execution" }, exclusions,
        qualification: "dependency reach only; not exhaustive behavior coverage or test execution",
    })
}

fn selection_command(options: &Options, seeds: &BTreeSet<String>, selected: &BTreeSet<String>, wide: bool) -> Vec<String> {
    let mut command: Vec<String> = ["scripts/pse-env", "--", "scripts/cargo-bin/cargo-pse-correctness", "nextest", "run", "--locked"].into_iter().map(str::to_owned).collect();
    if !options.features.is_empty() { command.extend(["--features".to_owned(), options.features.clone()]); }
    if wide { command.push("--workspace".into()); }
    else {
        for name in selected { command.extend(["-p".to_owned(), name.clone()]); }
        let filter = seeds.iter().map(|name| format!("rdeps(={name})")).collect::<Vec<_>>().join(" | ");
        command.extend(["-E".to_owned(), filter]);
    }
    command.extend(options.extra.iter().cloned());
    command
}

/// Observe source-only reach. No compilation, scientific capture, nextest listing or DB startup.
/// # Errors
/// Change discovery or path validation failed. Metadata failures appear as unavailable graph
/// detail, with no executable selection.
pub fn preview(root: &Path, options: &Options) -> Result<Preview> {
    let changes = match &options.changes { Some(changes) => changes.clone(), None => changed_files(root, &options.base)? };
    validate_changes(&changes)?;
    let graph = if changes.is_empty() { Ok(Workspace::default()) } else {
        MetadataCommand::new().current_dir(root).manifest_path(root.join("Cargo.toml")).no_deps().other_options(vec!["--locked".into()]).exec().context("cargo metadata --no-deps --locked").and_then(|metadata| Workspace::from_metadata(root, &metadata))
    };
    assemble(root, options, changes, graph)
}

/// Execute exactly the displayed original command, only on an explicit run request.
/// # Errors
/// The graph is unavailable or command launch failed. The original exit/signal is returned.
pub fn run(root: &Path, selection: &Preview) -> Result<Option<ExitStatus>> {
    let Some(command) = &selection.command else {
        if selection.graph_error.is_some() { bail!("cannot run affected selection: Cargo graph is unavailable"); }
        return Ok(None);
    };
    let executable = root.join(&command[0]);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = Command::new(executable).args(&command[1..]).current_dir(root).exec();
        Err(error).context("executing original affected test command")
    }
    #[cfg(not(unix))]
    { Ok(Some(Command::new(executable).args(&command[1..]).current_dir(root).status().context("executing original affected test command")?)) }
}

#[cfg(test)]
#[path = "affected_tests.rs"]
mod tests;
