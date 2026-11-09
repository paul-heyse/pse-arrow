// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual native developer command, with no product Python or canonical deployment.
#![cfg(feature = "package-fixtures")]
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
    command.env("PSE_PYTHON", "/unavailable-conformance-python");
    command
}

fn fixture(expected: u32) -> Result<(tempfile::TempDir, PathBuf)> {
    let directory = tempfile::tempdir()?;
    let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/packages");
    let physical = packages.join("physical-primitives");
    let model = directory.path().join("model");
    std::fs::create_dir_all(model.join("models"))?;
    let header = std::fs::read_to_string(packages.join("minimal_explicit/package.toml"))?
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", &format!("dependencies = [{{ package_id = \"{}\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }}]", "5a".repeat(16)));
    std::fs::write(model.join("package.toml"), header)?;
    std::fs::write(
        model.join("models/pure.pse"),
        format!(
            r#"package command {{
        fn square(x:Scalar)->Scalar=x*x;
        test sample fixture {{ dof 0; route steady; procedure check; }} {{
            expect square(2)=={expected} tolerance 1e-12;
        }}
    }}"#
        ),
    )?;
    let path = directory.path().join("conformance.toml");
    std::fs::write(
        &path,
        format!(
            r#"version=1
[settings]
version=1
memory_limit_bytes=1073741824
[[runs]]
version=1
name="first"
package="model"
physical={physical:?}
execution="pure"
"#,
            physical = physical.to_string_lossy()
        ),
    )?;
    Ok((directory, path))
}

#[test]
fn conformance_command_help_needs_no_manifest_python_or_store() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let missing = directory.path().join("uncreated-store");
    let output = command()
        .env("PSE_SURREAL_STATE", &missing)
        .args(["conformance", "--help"])
        .output()?;
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stdout)?.contains("--manifest"));
    assert!(!missing.exists());
    Ok(())
}

#[test]
fn conformance_readiness_resolves_authored_roots_without_output_or_execution() -> Result<()> {
    let (directory, manifest) = fixture(5)?;
    let checkpoint = directory.path().join("progress.json");
    let output = command()
        .env("PSE_CONFORMANCE_PROGRESS", &checkpoint)
        .args(["conformance", "--readiness", "--manifest"])
        .arg(&manifest)
        .output()?;
    assert!(output.status.success(), "{output:?}");
    let readiness: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(readiness["runs"][0]["run"], "first");
    assert_eq!(readiness["runs"][0]["execution"], "pure");
    assert_eq!(readiness["fixture_admission"], "requires_runtime");
    assert!(!checkpoint.exists());
    std::fs::remove_file(directory.path().join("model/package.toml"))?;
    let missing = command()
        .args(["conformance", "--readiness", "--manifest"])
        .arg(&manifest)
        .output()?;
    assert!(!missing.status.success());
    assert!(String::from_utf8(missing.stderr)?.contains("has no package.toml"));
    let reports = directory.path().join("reports");
    let execution = command()
        .env("PSE_CONFORMANCE_PROGRESS", &checkpoint)
        .args(["conformance", "--manifest"])
        .arg(&manifest)
        .arg("--report-dir")
        .arg(&reports)
        .output()?;
    assert!(!execution.status.success());
    assert!(String::from_utf8(execution.stderr)?.contains("has no package.toml"));
    assert!(!checkpoint.exists());
    assert!(!reports.exists());
    Ok(())
}

#[test]
fn conformance_command_runs_pure_manifest_and_preserves_complete_reports() -> Result<()> {
    let (directory, manifest) = fixture(4)?;
    let reports = directory.path().join("reports");
    let missing = directory.path().join("uncreated-store");
    let output = command()
        .env("PSE_SURREAL_STATE", &missing)
        .args(["conformance", "--manifest"])
        .arg(&manifest)
        .arg("--report-dir")
        .arg(&reports)
        .output()?;
    assert!(output.status.success(), "{output:?}");
    let summary = String::from_utf8(output.stdout)?;
    assert!(
        summary.contains("first: passed=true complete=true coverage=package"),
        "{summary}"
    );
    assert_eq!(std::fs::read_dir(&reports)?.count(), 6);
    assert!(!missing.exists());
    let original = std::fs::read(reports.join("first.arrow"))?;
    let repeat = command()
        .args(["conformance", "--manifest"])
        .arg(&manifest)
        .arg("--report-dir")
        .arg(&reports)
        .output()?;
    assert!(
        !repeat.status.success(),
        "existing reports must refuse replacement"
    );
    assert_eq!(std::fs::read(reports.join("first.arrow"))?, original);
    let unknown_reports = directory.path().join("must-not-exist");
    let unknown = command()
        .args(["conformance", "--manifest"])
        .arg(&manifest)
        .args(["--run", "unknown", "--report-dir"])
        .arg(&unknown_reports)
        .output()?;
    assert!(!unknown.status.success());
    assert!(
        !unknown_reports.exists(),
        "run selection precedes publication effects"
    );
    Ok(())
}

#[test]
fn conformance_command_reports_failed_science_and_returns_nonzero() -> Result<()> {
    let (directory, manifest) = fixture(5)?;
    let reports = directory.path().join("failed-report");
    let progress = directory.path().join("conformance-progress.json");
    let output = command()
        .env("PSE_CONFORMANCE_PROGRESS", &progress)
        .args(["conformance", "--manifest"])
        .arg(&manifest)
        .arg("--report-dir")
        .arg(&reports)
        .output()?;
    assert!(
        !output.status.success(),
        "false scientific expectation must not become success"
    );
    let summary = String::from_utf8(output.stdout)?;
    assert!(summary.contains("passed=false"), "{summary}");
    let file = std::fs::File::open(reports.join("first.arrow"))?;
    let batches = datafusion::arrow::ipc::reader::FileReader::try_new(file, None)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let checks = batches.first().context("failed checks table")?;
    let status = checks.column_by_name("status").context("check status")?;
    let statuses = (0..checks.num_rows())
        .map(|row| datafusion::arrow::util::display::array_value_to_string(status.as_ref(), row))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert!(statuses.iter().any(|status| status == "failed"));
    let progress: serde_json::Value = serde_json::from_slice(&std::fs::read(progress)?)?;
    let run = &progress["runs"]["first"];
    assert_eq!(run["passed"], false);
    assert_eq!(run["settlement"], "no_native_lanes");
    assert_eq!(run["export"], "published");
    assert_eq!(run["published_outputs"].as_array().map(Vec::len), Some(6));
    let case = run["cases"].as_object().and_then(|cases| cases.values().next()).context("original case")?;
    assert_eq!(case["phase"], "returned");
    assert_eq!(case["status"], "Failed");
    Ok(())
}

#[test]
fn conformance_reports_science_and_partial_export_after_collision() -> Result<()> {
    let (directory, manifest) = fixture(4)?;
    let source = std::fs::read_to_string(&manifest)?;
    let (_, run) = source.split_once("[[runs]]").context("original campaign run")?;
    std::fs::write(
        &manifest,
        format!("{source}\n[[runs]]{}", run.replace("name=\"first\"", "name=\"second\"")),
    )?;
    let reports = directory.path().join("partial-reports");
    let progress = directory.path().join("conformance-progress.json");
    std::fs::create_dir(&reports)?;
    let existing = reports.join("first.fixtures.arrow");
    std::fs::write(&existing, "earlier result")?;
    let output = command()
        .env("PSE_CONFORMANCE_PROGRESS", &progress)
        .args(["conformance", "--manifest"])
        .arg(&manifest)
        .arg("--report-dir")
        .arg(&reports)
        .output()?;
    assert!(!output.status.success(), "a partial export is not complete");
    assert_eq!(std::fs::read(&existing)?, b"earlier result");
    let progress: serde_json::Value = serde_json::from_slice(&std::fs::read(progress)?)?;
    let run = &progress["runs"]["first"];
    assert_eq!(run["passed"], true, "original science completed before export failed");
    assert_eq!(run["export"], "failed");
    assert!(run["export_error"].as_str().is_some_and(|error| !error.is_empty()));
    assert_eq!(run["published_outputs"], serde_json::json!(["first.arrow"]));
    let skipped = &progress["runs"]["second"];
    assert_eq!(skipped["phase"], "unattempted");
    assert!(skipped["runtime_run"].is_null());
    assert!(skipped["passed"].is_null());
    Ok(())
}

#[test]
fn conformance_same_executable_admits_changed_model_source() -> Result<()> {
    let (directory, manifest) = fixture(4)?;
    let run = || command().args(["conformance", "--manifest"]).arg(&manifest).output();
    let first = run()?;
    assert!(first.status.success(), "{first:?}");
    let model = directory.path().join("model/models/pure.pse");
    let source = std::fs::read_to_string(&model)?;
    assert!(source.contains("square(2)==4"));
    std::fs::write(&model, source.replace("square(2)==4", "square(2)==5"))?;
    let changed = run()?;
    assert!(!changed.status.success());
    assert!(String::from_utf8(changed.stdout)?.contains("passed=false complete=true"));
    Ok(())
}
