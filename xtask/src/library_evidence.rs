// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compose native evidence owners without interpreting Rust or resolving API identities.
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    /// Logical family from the selection configuration; repeat for several. Default: all.
    #[arg(long = "family")]
    families: Vec<String>,
    #[arg(long, default_value = ".config/library-leverage.toml")]
    config: PathBuf,
    /// New caller-owned output directory; existing output is never overwritten.
    #[arg(long)]
    output: PathBuf,
    /// Optional frozen project for reference evidence; API evidence keeps this checkout's context.
    #[arg(long)]
    reference_project: Option<PathBuf>,
    /// Reuse an explicitly selected original reference response, retaining its original scope.
    #[arg(long, conflicts_with_all = ["features", "target", "no_default_features", "test_context", "build_macros"])]
    reference_capture: Option<PathBuf>,
    /// Explicitly prepare the native analysis tool. Prepare the reusable session separately.
    #[arg(long)]
    prepare: bool,
    /// Explicitly acquire each selected package's API documentation before exporting.
    #[arg(long)]
    acquire: bool,
    #[arg(long)]
    features: Option<String>,
    #[arg(long)]
    target: Option<String>,
    #[arg(long)]
    no_default_features: bool,
    #[arg(long)]
    test_context: bool,
    #[arg(long)]
    build_macros: bool,
}
#[derive(Deserialize)]
struct Selection {
    families: BTreeMap<String, Family>,
}
#[derive(Deserialize, Serialize)]
struct Family {
    packages: Vec<String>,
    references: Vec<String>,
}
#[derive(Serialize)]
struct Step {
    arguments: Vec<String>,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    spawn_error: Option<String>,
    evidence_error: Option<String>,
}
impl Step {
    fn passed(&self) -> bool {
        self.exit_code == Some(0) && self.spawn_error.is_none() && self.evidence_error.is_none()
    }
}
fn capture(root: &Path, output: &Path, name: &str, arguments: Vec<String>) -> Result<Step> {
    let stdout = format!("{name}.json");
    let stderr = format!("{name}.stderr");
    // Cargo injects its selected toolchain into `cargo run` children. The host
    // deliberately resolves the checkout pin itself and refuses ambient overrides.
    let result = Command::new(root.join("scripts/pse"))
        .current_dir(root)
        .env_remove("RUSTUP_TOOLCHAIN")
        .args(["--format", "json"])
        .args(&arguments)
        .output();
    let (exit_code, spawn_error, out, err) = match result {
        Ok(value) => (value.status.code(), None, value.stdout, value.stderr),
        Err(error) => (None, Some(error.to_string()), Vec::new(), Vec::new()),
    };
    fs::write(output.join(&stdout), out)?;
    fs::write(output.join(&stderr), err)?;
    Ok(Step {
        arguments,
        exit_code,
        stdout,
        stderr,
        spawn_error,
        evidence_error: None,
    })
}

fn validate_export(directory: &Path, package: &str) -> Result<()> {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("manifest.json"))?)?;
    ensure!(
        manifest["version"] == 1 && manifest["package_id"].as_str() == Some(package),
        "export manifest package/version does not match the requested artifact"
    );
    let tables = manifest["tables"]
        .as_object()
        .context("export manifest has no tables")?;
    ensure!(!tables.is_empty(), "export manifest contains no tables");
    for table in tables.values() {
        let file = table["file"].as_str().context("export table has no file")?;
        ensure!(
            Path::new(file).components().count() == 1 && directory.join(file).is_file(),
            "export table file is missing or outside its bundle"
        );
    }
    Ok(())
}

fn retain_reference_capture(
    root: &Path,
    output: &Path,
    capture: &Path,
    project: &Path,
    packages: &BTreeSet<String>,
) -> Result<Step> {
    let bytes = fs::read(root.join(capture))?;
    let original: serde_json::Value = serde_json::from_slice(&bytes)?;
    ensure!(
        original["version"] == 1
            && original["record"] == "response"
            && original["command"] == "inspect dependency-usage"
            && original["outcome"] == "success",
        "reference capture is not a successful dependency-usage response"
    );
    ensure!(
        original["observation"]["completeness"] == "complete",
        "reference capture has incomplete requested observations"
    );
    ensure!(
        original["observation"]["scope"]
            == "selected_native_world_resolved_written_token_references"
            && original["view"]["truncated"] == false
            && original["view"]["omitted_records"] == 0
            && original["view"]["omitted_fields"]
                .as_array()
                .is_some_and(Vec::is_empty)
            && original["view"]["fields"].is_null(),
        "reference capture must retain the full unprojected native reference view"
    );
    let configuration = &original["payload"]["session"]["configuration"];
    let captured_project = configuration["project"]
        .as_str()
        .context("capture has no original project")?;
    ensure!(
        Path::new(captured_project).canonicalize()? == project.canonicalize()?,
        "reference capture belongs to a different project"
    );
    ensure!(
        configuration["backend"] == "native_snapshot",
        "reference capture is not native evidence"
    );
    ensure!(
        original["payload"]["references"].is_array()
            && original["payload"]["unmapped_definitions"] == 0
            && original["payload"]["unavailable_package_ids"]
                .as_array()
                .is_some_and(Vec::is_empty),
        "reference capture lacks complete delivered observations"
    );
    let captured: BTreeSet<String> =
        serde_json::from_value(original["payload"]["package_ids"].clone())?;
    ensure!(
        &captured == packages,
        "reference capture package selection differs; select the original family scope"
    );
    fs::write(output.join("references.json"), &bytes)?;
    fs::write(output.join("references.stderr"), b"")?;
    Ok(Step {
        arguments: vec![
            "retain-original-reference-capture".into(),
            root.join(capture).display().to_string(),
        ],
        exit_code: Some(0),
        stdout: "references.json".into(),
        stderr: "references.stderr".into(),
        spawn_error: None,
        evidence_error: None,
    })
}
fn context(args: &Args) -> Vec<String> {
    let mut result = vec!["--backend".into(), "native-snapshot".into()];
    if let Some(project) = &args.reference_project {
        result.extend(["--project".into(), project.to_string_lossy().into_owned()]);
    }
    if let Some(value) = &args.features {
        result.extend(["--features".into(), value.clone()]);
    }
    if let Some(value) = &args.target {
        result.extend(["--target".into(), value.clone()]);
    }
    for (enabled, flag) in [
        (args.no_default_features, "--no-default-features"),
        (args.test_context, "--test-context"),
        (args.build_macros, "--build-macros"),
    ] {
        if enabled {
            result.push(flag.into());
        }
    }
    result
}
fn package_matches(patterns: &[String], name: &str) -> Result<bool> {
    let mut matches = false;
    for pattern in patterns {
        matches |= globset::Glob::new(pattern)?
            .compile_matcher()
            .is_match(name);
    }
    Ok(matches)
}
fn reference_summary(capture: &serde_json::Value) -> String {
    let Some(rows) = capture
        .pointer("/payload/references")
        .and_then(serde_json::Value::as_array)
    else {
        return "Reference inventory unavailable; inspect references.json and references.stderr.\n"
            .into();
    };
    let mut names = BTreeMap::<String, usize>::new();
    for row in rows {
        if let Some(name) = row.get("name").and_then(serde_json::Value::as_str) {
            *names.entry(name.to_owned()).or_default() += 1;
        }
    }
    let mut result = format!(
        "{} resolved reference observations; {} declaration labels. Counts include non-call references and do not rank value.\n\n",
        rows.len(),
        names.len()
    );
    result.push_str("| Declaration label (alphabetical sample) | References |\n| --- | ---: |\n");
    for (name, count) in names.iter().take(20) {
        result.push_str(&format!(
            "| {} | {count} |\n",
            name.replace('|', "\\|").replace('\n', " ")
        ));
    }
    result.push_str("\nAt most 20 labels shown; references.json retains the complete returned observations and profile attribution.\n");
    result
}
pub(crate) fn run(root: &Path, args: Args) -> Result<()> {
    let config_path = root.join(&args.config);
    let config_bytes = fs::read_to_string(&config_path)
        .with_context(|| format!("reading {}", config_path.display()))?;
    let selection: Selection = toml::from_str(&config_bytes)?;
    let chosen = if args.families.is_empty() {
        selection.families.keys().cloned().collect::<BTreeSet<_>>()
    } else {
        args.families.iter().cloned().collect()
    };
    ensure!(!chosen.is_empty(), "no library families selected");
    for family in &chosen {
        ensure!(
            selection.families.contains_key(family),
            "unknown library family {family}"
        );
    }
    // Metadata is the population only; exact native owners establish all subsequent facts.
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(root)
        .other_options(vec!["--locked".into()])
        .exec()?;
    let reference_metadata = args
        .reference_project
        .as_ref()
        .map(|project| {
            cargo_metadata::MetadataCommand::new()
                .current_dir(root.join(project))
                .other_options(vec!["--locked".into()])
                .exec()
        })
        .transpose()?;
    let reference_metadata = reference_metadata.as_ref().unwrap_or(&metadata);
    let mut selected = BTreeMap::new();
    let mut family_packages = BTreeMap::new();
    for family in &chosen {
        let specification = &selection.families[family];
        let mut ids = Vec::new();
        for package in &metadata.packages {
            if package_matches(&specification.packages, package.name.as_str())? {
                let id = package.id.to_string();
                ids.push(id.clone());
                selected.insert(id, package.name.to_string());
            }
        }
        ids.sort();
        ensure!(
            !ids.is_empty(),
            "family {family} matches no locked packages"
        );
        family_packages.insert(family.clone(), ids);
    }
    let output = root.join(&args.output);
    fs::create_dir(&output)
        .with_context(|| format!("output must be a new directory: {}", output.display()))?;
    fs::write(output.join("selection.toml"), config_bytes)?;
    fs::write(
        output.join("metadata.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    fs::write(
        output.join("reference-metadata.json"),
        serde_json::to_vec_pretty(reference_metadata)?,
    )?;
    let mut reference_packages = BTreeSet::new();
    let mut reference_families = BTreeMap::new();
    for family in &chosen {
        let mut ids = Vec::new();
        for package in &reference_metadata.packages {
            if package_matches(&selection.families[family].packages, package.name.as_str())? {
                let id = package.id.to_string();
                reference_packages.insert(id.clone());
                ids.push(id);
            }
        }
        ensure!(
            !ids.is_empty(),
            "family {family} matches no packages in the reference project; no reference census was run"
        );
        ids.sort();
        reference_families.insert(family.clone(), ids);
    }
    let mut steps = Vec::new();
    if args.prepare {
        steps.push(capture(
            root,
            &output,
            "prepare-tool",
            vec!["tools".into(), "prepare".into(), "native-analysis".into()],
        )?);
    }
    if let Some(original) = &args.reference_capture {
        steps.push(retain_reference_capture(
            root,
            &output,
            original,
            &root.join(args.reference_project.as_deref().unwrap_or(root)),
            &reference_packages,
        )?);
    } else {
        let mut usage = vec!["inspect".into(), "dependency-usage".into()];
        usage.extend(reference_packages.iter().cloned());
        usage.extend(context(&args));
        steps.push(capture(root, &output, "references", usage)?);
    }
    for (index, id) in selected.keys().enumerate() {
        steps.push(capture(
            root,
            &output,
            &format!("package-{index}"),
            vec!["inspect".into(), "dependency".into(), id.clone()],
        )?);
        if args.acquire {
            let mut command = vec!["api".into(), id.clone(), "--prepare".into()];
            if let Some(target) = &args.target {
                command.extend(["--target".into(), target.clone()]);
            }
            steps.push(capture(
                root,
                &output,
                &format!("acquire-{index}"),
                command,
            )?);
        }
        let mut command = vec![
            "inspect".into(),
            "api".into(),
            id.clone(),
            "--export-jsonl".into(),
            output
                .join(format!("api-{index}"))
                .to_string_lossy()
                .into_owned(),
        ];
        if let Some(target) = &args.target {
            command.extend(["--target".into(), target.clone()]);
        }
        let mut exported = capture(root, &output, &format!("export-{index}"), command)?;
        if exported.passed() {
            exported.evidence_error = validate_export(&output.join(format!("api-{index}")), id)
                .err()
                .map(|error| error.to_string());
        }
        steps.push(exported);
    }
    let failed = steps.iter().filter(|step| !step.passed()).count();
    let references = serde_json::from_slice(&fs::read(output.join("references.json"))?)
        .unwrap_or(serde_json::Value::Null);
    let manifest = serde_json::json!({"version":1,"families":family_packages,"packages":selected,"reference_families":reference_families,"reference_packages":reference_packages,"reference_capture":args.reference_capture,"steps":steps,"failed_steps":failed,"capture_complete":failed == 0,"reference_context":references.pointer("/payload/session"),"requested_native_arguments":args.reference_capture.is_none().then(||context(&args)),"coverage":"Capture completion describes requested evidence delivery only. A selected original reference capture retains its own configuration and historical observations; it is not revalidated against current source. Consult each response for missing packages, incomplete references and profile ambiguity. API and reference populations retain separate metadata and contexts. References are not runtime calls. API exports do not establish ordinary-build availability."});
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    let mut report = format!(
        "# Library evidence\n\n{} packages; {failed} failed steps (target: zero).\n\nRead [manifest.json](manifest.json) for exact package identities, contexts and capture paths.\nReferences are observations in one selected world, not an exhaustive usage or call census.\n\n",
        selected.len()
    );
    if args.reference_capture.is_some() {
        report.push_str("Original reference capture reused with its historical context; current source was not revalidated.\n\n");
    }
    report.push_str(&reference_summary(&references));
    for family in chosen {
        report.push_str(&format!("## {family}\n\n"));
        for reference in &selection.families[&family].references {
            report.push_str(&format!("- Reference: `{reference}`\n"));
        }
        report.push('\n');
    }
    fs::write(output.join("README.md"), report)?;
    if failed > 0 {
        bail!(
            "library evidence retained at {}; {failed} failed steps (target: zero)",
            output.display()
        );
    }
    println!("library evidence: {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_command_without_export_tables_is_not_evidence() {
        let directory = tempfile::tempdir().unwrap();
        assert!(validate_export(directory.path(), "selected").is_err());
        let manifest = serde_json::json!({"version":1,"package_id":"selected","tables":{"declarations":{"file":"declarations.jsonl"}}});
        fs::write(
            directory.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(validate_export(directory.path(), "selected").is_err());
        fs::write(directory.path().join("declarations.jsonl"), b"{}\n").unwrap();
        validate_export(directory.path(), "selected").unwrap();
        assert!(validate_export(directory.path(), "other").is_err());
        let step = Step {
            arguments: vec![],
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            spawn_error: None,
            evidence_error: Some("missing export".into()),
        };
        assert!(!step.passed());
    }
    #[test]
    fn retained_response_preserves_bytes_and_rejects_scope_changes() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("project");
        let other_project = directory.path().join("other");
        let output = directory.path().join("output");
        for path in [&project, &other_project, &output] {
            fs::create_dir(path).unwrap();
        }
        let mut original = serde_json::json!({"version":1,"record":"response","command":"inspect dependency-usage","outcome":"success","observation":{"completeness":"complete","scope":"selected_native_world_resolved_written_token_references"},"view":{"fields":null,"omitted_fields":[],"omitted_records":0,"truncated":false},"payload":{"session":{"configuration":{"project":project,"backend":"native_snapshot"}},"package_ids":["selected"],"references":[],"unmapped_definitions":0,"unavailable_package_ids":[]}});
        let capture = directory.path().join("original.json");
        let bytes = serde_json::to_vec_pretty(&original).unwrap();
        fs::write(&capture, &bytes).unwrap();
        let packages = BTreeSet::from(["selected".into()]);
        let step =
            retain_reference_capture(directory.path(), &output, &capture, &project, &packages)
                .unwrap();
        assert!(step.passed());
        assert_eq!(fs::read(output.join("references.json")).unwrap(), bytes);
        assert!(
            retain_reference_capture(
                directory.path(),
                &output,
                &capture,
                &other_project,
                &packages
            )
            .is_err()
        );
        assert!(
            retain_reference_capture(
                directory.path(),
                &output,
                &capture,
                &project,
                &BTreeSet::new()
            )
            .is_err()
        );
        original["view"]["truncated"] = true.into();
        fs::write(&capture, serde_json::to_vec(&original).unwrap()).unwrap();
        assert!(
            retain_reference_capture(directory.path(), &output, &capture, &project, &packages)
                .is_err()
        );
        original["view"]["truncated"] = false.into();
        original["observation"]["completeness"] = "partial".into();
        fs::write(&capture, serde_json::to_vec(&original).unwrap()).unwrap();
        assert!(
            retain_reference_capture(directory.path(), &output, &capture, &project, &packages)
                .is_err()
        );
    }
    #[test]
    fn family_patterns_do_not_confuse_facade_and_companion() {
        assert!(package_matches(&["arrow-*".into()], "arrow-array").unwrap());
        assert!(!package_matches(&["arrow-*".into()], "arrow").unwrap());
        assert!(package_matches(&["symbolica".into()], "symbolica").unwrap());
        assert!(package_matches(&["[".into()], "arrow").is_err());
    }
    #[test]
    fn spawn_failure_is_retained_and_does_not_abort_independent_capture() {
        let directory = tempfile::tempdir().unwrap();
        let step = capture(
            directory.path(),
            directory.path(),
            "absent",
            vec!["inspect".into()],
        )
        .unwrap();
        assert!(!step.passed());
        assert!(step.spawn_error.is_some());
        assert!(directory.path().join("absent.json").exists());
        assert!(directory.path().join("absent.stderr").exists());
        let next = capture(directory.path(), directory.path(), "next", vec![]).unwrap();
        assert!(next.spawn_error.is_some());
    }
    #[test]
    fn missing_inventory_is_not_reported_as_zero_usage() {
        assert!(reference_summary(&serde_json::json!({"payload":null})).contains("unavailable"));
        let report = reference_summary(
            &serde_json::json!({"payload":{"references":[{"name":"call|label"},{"name":"call|label"}]}}),
        );
        assert!(report.contains("call\\|label | 2"));
        assert!(report.contains("do not rank value"));
    }
}
