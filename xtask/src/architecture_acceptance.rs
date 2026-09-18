// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One final current-function campaign after the implementation/deletion cut.
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    fs::File,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

const RUST_TEST_GATE: [&str; 6] = [
    "test",
    "--profile",
    "ci",
    "--no-fail-fast",
    "--success-output",
    "final",
];

#[derive(Serialize)]
struct Check {
    command: Vec<String>,
    exit_code: Option<i32>,
    elapsed_seconds: f64,
    log: String,
}
#[derive(Serialize)]
struct Receipt {
    evidence: &'static str,
    baseline_failures: usize,
    source_revision: String,
    source_status: String,
    checks: Vec<Check>,
    start_at: Option<String>,
    selected_checks_complete: bool,
    checks_complete: bool,
    review: &'static str,
}

pub(crate) fn run(root: &Path, output: &Path, start_at: Option<&str>) -> Result<()> {
    std::fs::create_dir(output).context("architecture evidence directory must be new")?;
    let output = output.canonicalize()?;
    let mut receipt = Receipt {
        evidence: "Tested",
        baseline_failures: 0,
        source_revision: git(root, &["rev-parse", "HEAD"])?,
        source_status: git(root, &["status", "--porcelain=v1"])?,
        checks: Vec::new(),
        start_at: start_at.map(str::to_owned),
        selected_checks_complete: false,
        checks_complete: false,
        review: "Plan 09: Q01-Q14, G1-G7 and K01-K11 require independent assessment against these logs and measurements; this command does not invent review verdicts.",
    };
    std::fs::write(
        output.join("source.diff"),
        git(root, &["diff", "--binary", "HEAD"])?,
    )?;
    archive_source(root, &output)?;
    record_environment(root, &output)?;
    verify_deleted(root)?;
    let engineering = output.join("engineering");
    let commands = [
        vec!["fmt-check"],
        vec!["family-check"],
        vec!["codegen-check"],
        vec!["check"],
        vec!["clippy"],
        // Match the profile used by .github/workflows/rust.yml. Keep its checked-in
        // timeouts/retry policy, and preserve actual scientific/resource timings.
        RUST_TEST_GATE.to_vec(),
        vec!["doctest"],
        vec!["native-solver-test"],
        vec!["py-sync"],
        vec!["quality"],
        vec!["py-test"],
        vec!["bench-cache"],
        vec![
            "engineering-inspection",
            engineering.to_str().context("non-UTF-8 evidence path")?,
        ],
        vec!["features-powerset"],
        vec!["adr-lint"],
        vec!["docs-rust"],
        vec!["docs"],
    ];
    let start = start_at
        .map(|name| {
            commands
                .iter()
                .position(|command| command[0] == name)
                .with_context(|| format!("unknown architecture gate: {name}"))
        })
        .transpose()?
        .unwrap_or(0);
    for (index, arguments) in commands.iter().enumerate().skip(start) {
        println!("architecture acceptance: just {}", arguments.join(" "));
        let log = format!("{index:02}-{}.log", arguments[0]);
        let file = File::create(output.join(&log))?;
        let started = Instant::now();
        let status = recipe_command()
            .current_dir(root)
            .args(arguments)
            .stdout(Stdio::from(file.try_clone()?))
            .stderr(Stdio::from(file))
            .status()
            .with_context(|| format!("running just {}", arguments.join(" ")))?;
        receipt.checks.push(Check {
            command: std::iter::once("just".to_owned())
                .chain(arguments.iter().map(|arg| (*arg).to_owned()))
                .collect(),
            exit_code: status.code(),
            elapsed_seconds: started.elapsed().as_secs_f64(),
            log,
        });
        std::fs::write(
            output.join("checks.json"),
            serde_json::to_vec_pretty(&receipt)?,
        )?;
        ensure!(
            status.success(),
            "architecture acceptance stopped at {}: {status}; see {}",
            arguments[0],
            output.display()
        );
    }
    receipt.selected_checks_complete = true;
    // A continuation never certifies the commands it did not execute.
    receipt.checks_complete = start == 0;
    std::fs::write(
        output.join("checks.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    println!(
        "selected architecture checks complete: {}; assess the independent Plan 09 gates and any earlier receipts against this evidence",
        output.display()
    );
    Ok(())
}
fn recipe_command() -> Command {
    // Cargo's package-specific injected values are not the caller's build config.
    // Leaking them changes cc/ring fingerprints and rebuilds the native stack.
    let mut command = Command::new("just");
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("CARGO_PKG_") {
            command.env_remove(name);
        }
    }
    command
        .env_remove("CARGO_MANIFEST_DIR")
        .env_remove("CARGO_MANIFEST_PATH")
        .env_remove("CARGO_MANIFEST_LINKS");
    command
}
fn record_environment(root: &Path, output: &Path) -> Result<()> {
    for (program, arguments, name) in [
        ("rustc", vec!["-Vv"], "rust-toolchain.txt"),
        ("cargo", vec!["-Vv"], "cargo-version.txt"),
        ("uname", vec!["-a"], "machine.txt"),
    ] {
        let provenance = Command::new(program)
            .current_dir(root)
            .args(arguments)
            .output()?;
        ensure!(
            provenance.status.success(),
            "recording {program} provenance failed"
        );
        std::fs::write(output.join(name), provenance.stdout)?;
    }
    for (source, name) in [
        ("Cargo.lock", "Cargo.lock"),
        ("Cargo.toml", "Cargo.toml"),
        ("vendor/delta-rs/PROVENANCE.json", "delta-source.json"),
        (
            "docs/plans/09-native-caching-and-pivot-completion.md",
            "execution-plan.md",
        ),
    ] {
        std::fs::copy(root.join(source), output.join(name))?;
    }
    Ok(())
}

fn verify_deleted(root: &Path) -> Result<()> {
    for obsolete in [
        "crates/pse-catalog/src/store",
        "crates/pse-catalog/src/snapshot.rs",
        "crates/pse-compiler/src/driver.rs",
        "crates/pse-compiler/src/driver",
        "crates/pse-compiler/src/memo.rs",
        "crates/pse-compiler/src/passes/dag.rs",
        "crates/pse-authoring/src/change_set",
        "tests/support/native_catalog.rs",
    ] {
        ensure!(
            !root.join(obsolete).exists(),
            "legacy production path survives: {obsolete}"
        );
    }
    Ok(())
}

fn archive_source(root: &Path, output: &Path) -> Result<()> {
    // A dirty source tree includes new files that git diff cannot capture.
    // Hash all present source files and archive the untracked inputs separately.
    let mut source_files = std::collections::BTreeMap::new();
    let files = git(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
    )?;
    for name in files.split('\0').filter(|name| !name.is_empty()) {
        let path = root.join(name);
        if path.is_file() {
            use sha2::Digest;
            use std::fmt::Write;
            let mut hash = String::new();
            for byte in sha2::Sha256::digest(std::fs::read(path)?) {
                write!(hash, "{byte:02x}")?;
            }
            source_files.insert(name.to_owned(), hash);
        }
    }
    std::fs::write(
        output.join("source-files.json"),
        serde_json::to_vec_pretty(&source_files)?,
    )?;
    let untracked = git(root, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    for name in untracked.split('\0').filter(|name| !name.is_empty()) {
        let source = root.join(name);
        if source.is_file() {
            let destination = output.join("untracked").join(name);
            std::fs::create_dir_all(destination.parent().context("source archive parent")?)?;
            std::fs::copy(source, destination)?;
        }
    }
    Ok(())
}

fn git(root: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(arguments)
        .output()?;
    ensure!(output.status.success(), "source provenance command failed");
    Ok(String::from_utf8(output.stdout)?)
}
