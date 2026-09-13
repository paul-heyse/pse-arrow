// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::exit,
    clippy::disallowed_types,
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "driver binary: xtask is the command surface, so it prints, exits with codes, \
              and is the one place allowed to use anyhow (blueprint §23.2, clippy.toml)"
)]

//! `cargo xtask` — everything that needs Rust APIs, structured data, or cross-platform
//! behaviour. The justfile is the one-line surface; this is the logic (plan §3).
//!
//! Phase 0 implements `family-check`, `codegen --check`, `governance` and `probe-host`.
//! `codegen` (generate) and `doc-lint` are placeholders until `pse-schema` and the
//! extracted API facts exist; `release` lands with the release tooling ADR.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use serde::Deserialize;

/// Workspace driver for pse-arrow.
#[derive(Debug, Parser)]
#[command(
    name = "cargo xtask",
    about = "Workspace driver for pse-arrow (plan §3)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Assert one resolved version per dependency family, equal to the declared pin.
    FamilyCheck {
        /// Additional `Cargo.lock` files whose shared packages must agree with ours.
        #[arg(long, value_name = "LOCK")]
        evidence: Vec<PathBuf>,
        /// Compare only packages that belong to a declared family. The evidence lockfiles
        /// were resolved on their own day; unrelated transitive crates drift by patch
        /// version without telling us anything about the pins.
        #[arg(long)]
        evidence_families_only: bool,
    },
    /// Regenerate committed generated sources (phase 0: report only), or diff them.
    Codegen {
        /// Fail if any generated path differs from the index or is untracked.
        #[arg(long)]
        check: bool,
        /// Restrict to one generator.
        #[arg(long, value_enum, value_name = "TARGET")]
        only: Option<Target>,
    },
    /// The full governance gate: governance tests, codegen diff, family check.
    Governance,
    /// Resolve backticked API paths in `docs/**` against the extracted facts.
    DocLint,
    /// Report what this host can do: toolchain, Ipopt, external function libraries.
    ProbeHost {
        /// Emit JSON instead of a human-readable report.
        #[arg(long)]
        json: bool,
    },
    /// Bump the workspace version, write the changelog, commit and tag.
    Release {
        /// The new version, e.g. `0.1.0`.
        version: String,
    },
}

/// The four committed generated trees (blueprint §3.1 "generated sources committed").
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Target {
    /// `pse-schema` output: typed relation views.
    Relations,
    /// `pse-schema` output: the Python contract classes.
    Python,
    /// `pse-schema` output: the reference tables in the book.
    Docs,
    /// `bindgen` output for the Ipopt C API.
    Bindgen,
}

impl Target {
    /// Repository-relative path the generator owns.
    const fn path(self) -> &'static str {
        match self {
            Self::Relations => "crates/pse-relations/src/generated",
            Self::Python => "python/pse/contracts",
            Self::Docs => "docs/generated",
            Self::Bindgen => "crates/pse-ipopt-sys/src/bindings.rs",
        }
    }

    /// What phase-0 `codegen` would run, once the generator exists.
    const fn generator(self) -> &'static str {
        match self {
            Self::Relations => "pse_schema::codegen::generate(Language::Rust)",
            Self::Python => "pse_schema::codegen::generate(Language::Python)",
            Self::Docs => "pse_schema::codegen::generate(Language::Markdown)",
            Self::Bindgen => {
                "bindgen $IPOPT_DIR/include/coin-or/IpStdCInterface.h (solver container)"
            }
        }
    }

    /// All of them, in generation order.
    const ALL: [Self; 4] = [Self::Relations, Self::Python, Self::Docs, Self::Bindgen];
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = workspace_root()?;
    match cli.command {
        Cmd::FamilyCheck {
            evidence,
            evidence_families_only,
        } => family_check(&root, &evidence, evidence_families_only),
        Cmd::Codegen { check, only } => codegen(&root, check, only),
        Cmd::Governance => governance(&root),
        Cmd::DocLint => {
            println!("doc-lint: not implemented in phase 0 (see docs/adr/register.md)");
            std::process::exit(2);
        }
        Cmd::ProbeHost { json } => {
            probe_host(json);
            Ok(())
        }
        Cmd::Release { version } => {
            eprintln!(
                "release {version}: not implemented in phase 0. The publishing tool \
                 (release-plz vs cargo-workspaces) is decided by ADR at phase-0 exit; \
                 until then bump [workspace.package].version by hand and tag."
            );
            std::process::exit(2);
        }
    }
}

/// The workspace root, derived from this crate's manifest directory rather than the
/// current directory: `cargo xtask` must behave the same from any subdirectory.
fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("xtask manifest directory has no parent"))
}

// ---------------------------------------------------------------------------
// family-check
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RootManifest {
    workspace: WorkspaceSection,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    metadata: MetadataSection,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    pse: PseMetadata,
}

#[derive(Debug, Deserialize)]
struct PseMetadata {
    families: BTreeMap<String, Family>,
}

#[derive(Debug, Deserialize)]
struct Family {
    version: String,
    #[serde(default, rename = "match")]
    match_kind: Option<String>,
    crates: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Lockfile {
    #[serde(default)]
    package: Vec<LockPackage>,
}

#[derive(Debug, Deserialize)]
struct LockPackage {
    name: String,
    version: String,
}

/// One resolved version per family, equal to the declared pin.
///
/// `=` pins bind direct dependencies only, and `cargo tree -d` does not report a mixed
/// family because a family is not a duplicate: two arrow majors in the graph make
/// `downcast_ref` return `None` with no compile error (blueprint §3.1).
fn family_check(root: &Path, evidence: &[PathBuf], families_only: bool) -> Result<()> {
    let manifest_path = root.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;
    let manifest: RootManifest =
        toml::from_str(&text).with_context(|| format!("parsing {}", manifest_path.display()))?;
    let families = manifest.workspace.metadata.pse.families;

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .other_options(vec!["--locked".to_owned()])
        .exec()
        .context("cargo metadata --locked --format-version 1")?;

    let resolved: BTreeMap<String, BTreeSet<String>> =
        metadata
            .packages
            .iter()
            .fold(BTreeMap::new(), |mut acc, pkg| {
                acc.entry(pkg.name.to_string())
                    .or_default()
                    .insert(pkg.version.to_string());
                acc
            });

    let mut failures: Vec<String> = Vec::new();
    let mut in_family: BTreeSet<String> = BTreeSet::new();

    println!("family            declared   match   resolved   packages");
    println!("----------------- ---------- ------- ---------- --------");
    for (name, family) in &families {
        if let Some(failure) = check_family(name, family, &resolved, &mut in_family)? {
            failures.push(failure);
        }
    }

    for lock in evidence {
        let path = if lock.is_absolute() {
            lock.clone()
        } else {
            root.join(lock)
        };
        if let Some(failure) = compare_evidence(&path, &resolved, &in_family, families_only)? {
            failures.push(failure);
        }
    }

    if failures.is_empty() {
        println!("\nfamily-check: OK");
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("family-check: {failure}");
        }
        bail!("family-check failed with {} problem(s)", failures.len())
    }
}

/// One family: collect its members from the resolved graph, print the row, and return the
/// failure message if there is one.
fn check_family(
    name: &str,
    family: &Family,
    resolved: &BTreeMap<String, BTreeSet<String>>,
    in_family: &mut BTreeSet<String>,
) -> Result<Option<String>> {
    let matchers = family
        .crates
        .iter()
        .map(|glob| glob_to_regex(glob))
        .collect::<Result<Vec<_>>>()?;
    let mut versions: BTreeSet<String> = BTreeSet::new();
    let mut members: Vec<String> = Vec::new();
    for (pkg, pkg_versions) in resolved {
        if matchers.iter().any(|re| re.is_match(pkg)) {
            in_family.insert(pkg.clone());
            members.push(pkg.clone());
            versions.extend(pkg_versions.iter().cloned());
        }
    }
    let kind = family.match_kind.as_deref().unwrap_or("exact");
    let shown: Vec<String> = versions.iter().cloned().collect();
    println!(
        "{:<17} {:<10} {:<7} {:<10} {}",
        name,
        family.version,
        kind,
        shown.join(", "),
        members.len()
    );
    if members.is_empty() {
        return Ok(Some(format!(
            "family `{name}`: no package in the resolved graph matches {:?}",
            family.crates
        )));
    }
    if versions.len() != 1 {
        return Ok(Some(format!(
            "family `{name}`: {} versions in the resolved graph ({}); members: {}",
            versions.len(),
            shown.join(", "),
            members.join(", ")
        )));
    }
    let resolved_version = shown.first().cloned().unwrap_or_default();
    if version_matches(&resolved_version, &family.version, kind)? {
        Ok(None)
    } else {
        Ok(Some(format!(
            "family `{name}`: resolved {resolved_version}, declared {} (match = {kind})",
            family.version
        )))
    }
}

/// One evidence lockfile: every package it shares with our resolved graph must be at the
/// same version. The capability-map lockfiles are what the API surfaces were extracted
/// from, so a divergence means the maps describe a different build than ours.
fn compare_evidence(
    path: &Path,
    resolved: &BTreeMap<String, BTreeSet<String>>,
    in_family: &BTreeSet<String>,
    families_only: bool,
) -> Result<Option<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading evidence lockfile {}", path.display()))?;
    let parsed: Lockfile = toml::from_str(&text)
        .with_context(|| format!("parsing evidence lockfile {}", path.display()))?;

    let mut compared = 0_usize;
    let mut diffs: Vec<String> = Vec::new();
    for pkg in &parsed.package {
        if families_only && !in_family.contains(&pkg.name) {
            continue;
        }
        let Some(ours) = resolved.get(&pkg.name) else {
            continue;
        };
        compared += 1;
        if !ours.contains(&pkg.version) {
            diffs.push(format!(
                "  {} evidence {} vs resolved {}",
                pkg.name,
                pkg.version,
                ours.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
    }

    println!(
        "\nevidence {}: {compared} shared package(s){}",
        path.display(),
        if families_only {
            " (families only)"
        } else {
            ""
        }
    );
    if diffs.is_empty() {
        println!("  all shared packages agree");
        return Ok(None);
    }
    diffs.sort();
    for line in &diffs {
        println!("{line}");
    }
    Ok(Some(format!(
        "evidence {}: {} shared package(s) differ",
        path.display(),
        diffs.len()
    )))
}

/// `arrow-*` style globs. Only `*` is special; everything else is literal.
fn glob_to_regex(pattern: &str) -> Result<Regex> {
    let mut source = String::from("^");
    for ch in pattern.chars() {
        if ch == '*' {
            source.push_str(".*");
        } else {
            source.push_str(&regex::escape(&ch.to_string()));
        }
    }
    source.push('$');
    Regex::new(&source).with_context(|| format!("glob `{pattern}`"))
}

/// `match = "minor"` compares `major.minor`; anything else is an exact string compare.
fn version_matches(resolved: &str, declared: &str, kind: &str) -> Result<bool> {
    match kind {
        "exact" => Ok(resolved == declared),
        "minor" => {
            let r = semver::Version::parse(resolved)
                .with_context(|| format!("resolved version `{resolved}`"))?;
            let parts: Vec<&str> = declared.split('.').collect();
            let [major, minor, ..] = parts.as_slice() else {
                bail!("declared version `{declared}` has no minor component");
            };
            Ok(r.major.to_string() == *major && r.minor.to_string() == *minor)
        }
        other => bail!("unknown family match kind `{other}` (use \"exact\" or \"minor\")"),
    }
}

// ---------------------------------------------------------------------------
// codegen
// ---------------------------------------------------------------------------

/// Phase 0: the generators do not exist yet, so `codegen` reports and `codegen --check`
/// does the half that is real today — proving the committed generated trees are exactly
/// what is in the index, and that nothing new appeared beside them.
fn codegen(root: &Path, check: bool, only: Option<Target>) -> Result<()> {
    let targets: Vec<Target> = only.map_or_else(|| Target::ALL.to_vec(), |t| vec![t]);
    let paths: Vec<&str> = targets.iter().map(|t| t.path()).collect();

    if !check {
        println!("codegen: phase 0 has no generator yet (pse-schema is a declared boundary).");
        println!("Would regenerate:");
        for target in &targets {
            println!("  {:<40} <- {}", target.path(), target.generator());
        }
        println!("\nRun `cargo xtask codegen --check` to diff the committed trees.");
        return Ok(());
    }

    let mut failures: Vec<String> = Vec::new();

    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "HEAD", "--exit-code", "--stat", "--"])
        .args(&paths)
        .status()
        .context("running git diff")?;
    if !status.success() {
        failures.push(format!(
            "generated sources differ from HEAD; run `cargo xtask codegen` and commit \
             the result ({})",
            paths.join(" ")
        ));
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain", "--"])
        .args(&paths)
        .output()
        .context("running git status --porcelain")?;
    if !output.status.success() {
        bail!("git status --porcelain failed");
    }
    let untracked: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with("??"))
        .map(|line| line[2..].trim().to_owned())
        .collect();
    if !untracked.is_empty() {
        failures.push(format!(
            "untracked file(s) under a generated path — generated sources are committed \
             (blueprint §3.1): {}",
            untracked.join(", ")
        ));
    }

    if failures.is_empty() {
        println!("codegen --check: OK ({} path(s))", paths.len());
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("codegen --check: {failure}");
        }
        bail!("codegen --check failed with {} problem(s)", failures.len())
    }
}

// ---------------------------------------------------------------------------
// governance
// ---------------------------------------------------------------------------

/// The gate `governance / *` runs: the governance test crate, then the two checks that
/// need a resolved graph or git.
fn governance(root: &Path) -> Result<()> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    // `cargo -C <path>` is nightly-only; the working directory is the portable form.
    let status = Command::new(cargo)
        .args([
            "nextest",
            "run",
            "-p",
            "pse-tests-governance",
            "--no-fail-fast",
            "--locked",
        ])
        .current_dir(root)
        .status()
        .context("running cargo nextest for pse-tests-governance")?;
    if !status.success() {
        bail!("pse-tests-governance failed");
    }
    codegen(root, true, None)?;
    family_check(root, &[], false)
}

// ---------------------------------------------------------------------------
// probe-host
// ---------------------------------------------------------------------------

/// What this host can do. The runtime's own `probe_host()` (blueprint §20) supersedes
/// this once it exists; today it answers "why did the ipopt feature not link?".
fn probe_host(json: bool) {
    let rustc = rustc_version();
    let dep_ipopt = std::env::var("DEP_IPOPT_VERSION").ok();
    let ipopt_on_path = which("ipopt");

    if json {
        println!("{{");
        println!("  \"rustc_version\": {},", json_string(&rustc));
        println!(
            "  \"dep_ipopt_version\": {},",
            dep_ipopt
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string)
        );
        println!(
            "  \"ipopt_on_path\": {}",
            ipopt_on_path.as_ref().map_or_else(
                || "null".to_owned(),
                |p| json_string(&p.display().to_string())
            )
        );
        println!("}}");
    } else {
        println!("rustc               {rustc}");
        println!(
            "DEP_IPOPT_VERSION   {}",
            dep_ipopt
                .as_deref()
                .unwrap_or("(unset — the `ipopt` feature is off, or nothing built the -sys crate)")
        );
        match &ipopt_on_path {
            Some(path) => println!("ipopt on PATH       {}", path.display()),
            None => println!(
                "ipopt on PATH       no (the NL backend needs the binary; see docker/solvers)"
            ),
        }
    }
}

/// `rustc -vV`'s `release:` line, or `unknown`.
fn rustc_version() -> String {
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    Command::new(rustc).arg("-vV").output().map_or_else(
        |_| "unknown".to_owned(),
        |out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .find_map(|line| line.strip_prefix("release: "))
                .map_or("unknown", str::trim)
                .to_owned()
        },
    )
}

/// First `name` on `PATH` (no `which` binary dependency: this has to work on Windows).
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let direct = dir.join(name);
        if direct.is_file() {
            return Some(direct);
        }
        let exe = dir.join(format!("{name}.exe"));
        exe.is_file().then_some(exe)
    })
}

/// Minimal JSON string escaping; the probe emits three scalars and never needs `serde_json`.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
