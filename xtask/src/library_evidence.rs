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
}
impl Step {
    fn passed(&self) -> bool {
        self.exit_code == Some(0) && self.spawn_error.is_none()
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
    })
}
fn context(args: &Args) -> Vec<String> {
    let mut result = vec!["--backend".into(), "native-snapshot".into()];
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
    let mut steps = Vec::new();
    if args.prepare {
        steps.push(capture(
            root,
            &output,
            "prepare-tool",
            vec!["tools".into(), "prepare".into(), "native-analysis".into()],
        )?);
    }
    let mut usage = vec!["inspect".into(), "dependency-usage".into()];
    usage.extend(selected.keys().cloned());
    usage.extend(context(&args));
    steps.push(capture(root, &output, "references", usage)?);
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
        steps.push(capture(root, &output, &format!("export-{index}"), command)?);
    }
    let failed = steps.iter().filter(|step| !step.passed()).count();
    let manifest = serde_json::json!({"version":1,"families":family_packages,"packages":selected,"steps":steps,"failed_steps":failed,"capture_complete":failed == 0,"semantic_context":context(&args),"coverage":"Capture completion describes command success only. Consult each original response for missing packages, incomplete references and profile ambiguity. Selected locked packages, one explicitly selected native world, and separately acquired documentation contexts. References are not calls or proof of runtime use. API exports do not establish ordinary-build availability."});
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    let mut report = format!(
        "# Library evidence\n\n{} packages; {failed} failed steps (target: zero).\n\nRead [manifest.json](manifest.json) for exact package identities, contexts and capture paths.\nReferences are observations in one selected world, not an exhaustive usage or call census.\n\n",
        selected.len()
    );
    let references = serde_json::from_slice(&fs::read(output.join("references.json"))?)
        .unwrap_or(serde_json::Value::Null);
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
