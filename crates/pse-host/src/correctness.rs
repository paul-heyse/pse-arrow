// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cargo-owned compiler-unit selection and Arrow correctness admission.
//!
//! The nextest adapter observes Cargo's translated build argv, never interprets nextest's
//! complete CLI grammar, and never sends unit-graph JSON into its compiler-message stream.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitStatus},
};

const REAL_CARGO: &str = "PSE_CORRECTNESS_REAL_CARGO";
const ADAPTER: &str = "PSE_CORRECTNESS_ADAPTER";

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
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
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
}
#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
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

/// The validated original operation, with only necessary root feature additions.
#[derive(Debug)]
pub struct Resolution {
    /// Actual Cargo arguments, excluding its executable.
    pub arguments: Vec<OsString>,
    /// Selected root package names, not a test inventory.
    pub packages: Vec<String>,
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
/// Resolve with injected Cargo capture for focused controls; native execution uses real Cargo.
pub fn resolve_with<F>(
    arguments: &[OsString],
    mut capture: F,
) -> Result<Resolution, CorrectnessError>
where
    F: FnMut(&[OsString]) -> Result<Vec<u8>, CorrectnessError>,
{
    let metadata: Metadata = serde_json::from_slice(&capture(&metadata_arguments(arguments)?)?)?;
    let mut packages = BTreeMap::new();
    for package in metadata.packages {
        if packages.insert(package.id.clone(), package).is_some() {
            return Err(invalid("Cargo metadata repeats a package identity"));
        }
    }
    let original: Graph = serde_json::from_slice(&capture(&graph_arguments(arguments)?)?)?;
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
    let mut selected = arguments.to_vec();
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
    for index in validated.reached(&packages, &validated.roots)? {
        let unit = &validated.units[index];
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
    Ok(Resolution {
        arguments: selected,
        packages: names.into_iter().collect(),
    })
}
fn native_command(cargo: &OsStr, arguments: &[OsString]) -> Command {
    let mut command = Command::new(cargo);
    command
        .args(arguments)
        .env("CARGO", cargo)
        .env_remove(ADAPTER);
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
fn real_cargo() -> Result<OsString, CorrectnessError> {
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
fn require_nextest_workspace(cargo: &OsStr) -> Result<(), CorrectnessError> {
    let executable = std::env::current_exe()?;
    let root = executable
        .ancestors()
        .find(|path| path.join(".config/diagnostic-tools.toml").is_file())
        .ok_or_else(|| invalid("native executable is outside the supported checkout"))?
        .canonicalize()?;
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
/// Native external Cargo command and invocation-local nextest build adapter.
pub fn run(arguments: Vec<OsString>) -> Result<i32, CorrectnessError> {
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
    if arguments.as_slice() == [OsString::from("--version")]
        && std::env::var_os(ADAPTER).is_none_or(|value| value != "1")
    {
        writeln!(
            io::stdout(),
            "cargo-pse-correctness {} protocol 1",
            env!("CARGO_PKG_VERSION")
        )?;
        return Ok(0);
    }
    let cargo = real_cargo()?;
    if std::env::var_os(ADAPTER).is_some_and(|value| value == "1") {
        // A discovery marker is not compilation authority: nested Cargo compiler
        // calls must establish the same graph even when a caller supplies the marker.
        if arguments.as_slice() == [OsString::from("--version")]
            || arguments.as_slice() == [OsString::from("-V")]
        {
            return execute(&cargo, &arguments);
        }
        let position = mode_position(&arguments)?;
        if arguments[position] == "metadata" || arguments[position] == "locate-project" {
            return execute(&cargo, &arguments);
        }
        let resolution = resolve_with(&arguments, |args| capture(&cargo, args))?;
        return execute(&cargo, &resolution.arguments);
    }
    if arguments
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
        || arguments.is_empty()
    {
        writeln!(
            io::stdout(),
            "cargo pse-correctness check|build|test|bench|run|clippy [Cargo arguments]\ncargo pse-correctness nextest run|list [native nextest arguments]\nSource-build correctness only. Nextest supports this executable checkout; foreign workspace, manifest/config overrides, archive/reused binaries are refused before nextest starts."
        )?;
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
    let resolution = resolve_with(&arguments, |args| capture(&cargo, args))?;
    execute(&cargo, &resolution.arguments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
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
