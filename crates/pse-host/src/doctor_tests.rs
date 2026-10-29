// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native diagnostic observations with explicit availability and repair boundaries.
use super::*;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn probe(expected: Option<&str>) -> Probe { Probe::version("fixture", "/bin/sh".into(), expected.map(str::to_owned), false, "explicit fixture repair") }

#[test]
fn exact_version_observation_rejects_prefix_or_suffix_drift() {
    let probe = probe(Some("0.19.0"));
    assert!(probe.assess("sccache 0.19.0\n").is_ok());
    assert!(probe.assess("sccache v0.19.0 (build)\n").is_ok());
    for actual in ["sccache 0.19.01", "sccache 0.19.0-dev", "sccache 0.18.0", ""] { assert!(probe.assess(actual).is_err(), "{actual}"); }
}

#[test]
fn component_observation_requires_each_declared_component() {
    let mut probe = probe(None);
    probe.components = Some(vec!["rustfmt".into(), "clippy".into(), "rust-src".into()]);
    assert!(probe.assess("rustfmt-x86_64-unknown-linux-gnu\nclippy-x86_64-unknown-linux-gnu\nrust-src").is_ok());
    assert!(probe.assess("rustfmt-x86_64-unknown-linux-gnu\nclippy-x86_64-unknown-linux-gnu").is_err());
    assert!(probe.assess("rustfmt-extra\nnot-clippy\nrust-src").is_err());
}

#[test]
fn optional_unavailable_is_retained_without_granting_required_readiness() -> Result {
    let mut report = Report { version: 1, checks: vec![Check::new("optional", false, Err("not observed".into()), "explicit optional preparation")] };
    assert!(report.ready());
    assert!(!report.checks[0].ok);
    report.checks.push(Check::new("required", true, Err("missing native tool".into()), "scripts/bootstrap-host"));
    assert!(!report.ready());
    let encoded = serde_json::to_value(&report)?;
    assert_eq!(encoded["checks"][0]["ok"], false);
    assert_eq!(encoded["checks"][1]["blocking"], true);
    Ok(())
}

#[test]
fn missing_checkout_declarations_report_failures_without_creating_runtime_or_tools() -> Result {
    let root = tempfile::tempdir()?;
    let (checks, probes) = declarations(root.path(), &Environment::new());
    assert!(checks.iter().any(|check| check.name == "environment" && !check.ok && check.blocking));
    assert!(checks.iter().any(|check| check.name == "rust-toolchain" && !check.ok && check.blocking));
    assert!(probes.iter().any(|probe| probe.name == "prepared-pse-host" && probe.blocking));
    assert!(std::fs::read_dir(root.path())?.next().is_none(), "diagnosis is not preparation");
    assert!(!checks.iter().any(|check| check.name == "python" || check.name == "extension"));
    Ok(())
}

#[test]
fn declaration_planning_reuses_environment_and_declared_tool_pins_without_python_runtime() -> Result {
    let root = tempfile::tempdir()?;
    for (name, source) in [
        (".config/build.toml", include_str!("../../../.config/build.toml")),
        (".config/agent-capacity.toml", include_str!("../../../.config/agent-capacity.toml")),
        (".config/diagnostic-tools.toml", include_str!("../../../.config/diagnostic-tools.toml")),
        (".config/sccache.toml", include_str!("../../../.config/sccache.toml")),
        (".cargo/config.toml", include_str!("../../../.cargo/config.toml")),
        ("rust-toolchain.toml", include_str!("../../../rust-toolchain.toml")),
        (".python-version", include_str!("../../../.python-version")),
        ("docs/site.toml", include_str!("../../../docs/site.toml")),
    ] {
        let path = root.path().join(name);
        std::fs::create_dir_all(path.parent().ok_or("fixture parent")?)?;
        std::fs::write(path, source)?;
    }
    let values = Environment::from([("HOME".into(), root.path().as_os_str().to_owned()), ("RUSTC_WRAPPER".into(), OsString::new()), ("PSE_BUILD_MODE".into(), "unbounded".into())]);
    let (checks, probes) = declarations(root.path(), &values);
    assert!(checks.iter().any(|check| check.name == "environment" && check.ok), "{checks:?}");
    assert!(!root.path().join(".venv").exists());
    let tools: toml::Value = toml::from_str(include_str!("../../../.config/diagnostic-tools.toml"))?;
    for (name, version) in tools["cargo-tools"].as_table().ok_or("Cargo tool declaration")? {
        let probe = probes.iter().find(|probe| &probe.name == name).ok_or("declared Cargo probe")?;
        assert_eq!(probe.expected.as_deref(), version.as_str());
        assert!(probe.blocking);
        assert!(probe.arguments.iter().any(|argument| argument == "--version"));
    }
    assert!(checks.iter().any(|check| check.name == "canonical-readiness" && !check.ok && !check.blocking));
    Ok(())
}

#[tokio::test]
async fn version_probe_observes_real_output_and_reports_missing_or_nonzero_tool() -> Result {
    let root = tempfile::tempdir()?;
    let mut actual = probe(Some("1.2.3"));
    actual.arguments = ["-c", "printf 'fixture 1.2.3\\n'"].map(OsString::from).into();
    assert!(actual.run(root.path()).await.ok);
    let mut failed = probe(None);
    failed.arguments = ["-c", "printf 'version failed' >&2; exit 9"].map(OsString::from).into();
    let check = failed.run(root.path()).await;
    assert!(!check.ok && !check.blocking);
    assert!(check.detail.contains("version failed"));
    let mut missing = probe(None);
    missing.program = root.path().join("missing");
    assert!(!missing.run(root.path()).await.ok);
    Ok(())
}

#[tokio::test]
async fn timed_out_version_probe_stops_and_reaps_its_original_child() -> Result {
    let root = tempfile::tempdir()?;
    let pid = root.path().join("pid");
    let mut actual = probe(None);
    actual.timeout = Duration::from_millis(150);
    actual.arguments = [OsString::from("-c"), OsString::from("printf '%s' $$ > \"$1\"; exec sleep 30"), OsString::from("fixture"), pid.clone().into_os_string()].into();
    let result = actual.run(root.path()).await;
    assert!(!result.ok && result.detail.contains("timed out"), "{}", result.detail);
    let pid = std::fs::read_to_string(pid)?;
    assert!(!Path::new("/proc").join(pid).exists(), "the version child was reaped before diagnosis returned");
    Ok(())
}

#[test]
fn optional_unreceipted_executable_is_never_planned_for_execution() -> Result {
    let root = tempfile::tempdir()?;
    std::fs::create_dir(root.path().join(".config"))?;
    std::fs::write(root.path().join("rust-toolchain.toml"), include_str!("../../../rust-toolchain.toml"))?;
    std::fs::write(root.path().join(".config/diagnostic-tools.toml"), include_str!("../../../.config/diagnostic-tools.toml"))?;
    let values = Environment::from([("HOME".into(), root.path().as_os_str().to_owned())]);
    let binary = crate::diagnostic_tools::binary_path(root.path(), crate::diagnostic_tools::Tool::Mcpls, &values)?;
    std::fs::create_dir_all(binary.parent().ok_or("tool parent")?)?;
    std::fs::write(&binary, b"unreceipted executable")?;
    let (checks, probes) = declarations(root.path(), &values);
    assert!(!probes.iter().any(|probe| probe.program == binary));
    assert!(checks.iter().any(|check| check.name == "optional-mcpls" && !check.ok && !check.blocking));
    assert_eq!(std::fs::read(binary)?, b"unreceipted executable");
    Ok(())
}
