// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Tool failures must not acquire dependency-finding meaning.
use super::*;
use serde_json::json;

#[test]
fn deny_bitmask_requires_one_complete_reconciled_summary() {
    let summary = json!({"type": "summary", "fields": {
        "advisories": {"errors": 3}, "bans": {"errors": 1},
        "licenses": {"errors": 0}, "sources": {"errors": 0}
    }})
    .to_string();
    assert_eq!(
        finding_status(Gate::Dependencies, Some(3), "", &summary),
        Status::Findings
    );
    for (code, stderr) in [
        (Some(1), summary.clone()),
        (Some(3), format!("{summary}\n{summary}")),
        (Some(3), format!("{summary}\nnetwork error")),
        (Some(3), "{\"type\":\"summary\",\"fields\":{}}".into()),
        (None, summary.clone()),
        (Some(125), summary),
    ] {
        assert_eq!(
            finding_status(Gate::Dependencies, code, "", &stderr),
            Status::Failed,
            "{code:?}: {stderr}"
        );
    }
}

#[test]
fn shear_requires_a_valid_report_even_for_exit_zero() {
    let clean = r#"{"findings":[],"summary":{"errors":0,"warnings":0}}"#;
    let warning = r#"{"findings":[{"code":"shear/shared_path","severity":"warning"}],"summary":{"errors":0,"warnings":1}}"#;
    let error = r#"{"findings":[{"code":"shear/unused_dependency","severity":"error"}],"summary":{"errors":1,"warnings":0}}"#;
    assert_eq!(
        finding_status(Gate::Shear, Some(0), clean, ""),
        Status::Passed
    );
    for report in [warning, error] {
        assert_eq!(
            finding_status(Gate::Shear, Some(1), report, ""),
            Status::Findings
        );
    }
    for (code, report) in [
        (Some(0), ""),
        (Some(0), "{}"),
        (Some(0), warning),
        (Some(1), clean),
        (Some(2), error),
        (None, error),
        (Some(1), r#"{"error":"failed to parse manifest"}"#),
        (
            Some(0),
            r#"{"findings":[],"summary":{"errors":false,"warnings":0}}"#,
        ),
        (
            Some(0),
            r#"{"findings":[null],"summary":{"errors":0,"warnings":0}}"#,
        ),
        (
            Some(1),
            r#"{"findings":[{"code":"shear/unused","severity":"warning"}],"summary":{"errors":1,"warnings":0}}"#,
        ),
        (
            Some(1),
            r#"{"findings":[{"code":"foreign/unused","severity":"error"}],"summary":{"errors":1,"warnings":0}}"#,
        ),
    ] {
        assert_eq!(
            finding_status(Gate::Shear, code, report, ""),
            Status::Failed,
            "{code:?}: {report}"
        );
    }
}

#[test]
fn advisories_require_reported_findings_and_the_finding_exit() {
    for report in [
        r#"{"vulnerabilities":{"found":true},"warnings":{}}"#,
        r#"{"vulnerabilities":{"found":false},"warnings":{"unmaintained":[{}]}}"#,
    ] {
        assert_eq!(
            finding_status(Gate::Advisories, Some(1), report, ""),
            Status::Findings
        );
        assert_eq!(
            finding_status(Gate::Advisories, Some(2), report, ""),
            Status::Failed
        );
        assert_eq!(
            finding_status(Gate::Advisories, None, report, ""),
            Status::Failed
        );
    }
    for report in [
        "",
        "{}",
        "not json",
        r#"{"vulnerabilities":{"found":false},"warnings":{}}"#,
        r#"{"vulnerabilities":{"found":"true"}}"#,
    ] {
        assert_eq!(
            finding_status(Gate::Advisories, Some(1), report, ""),
            Status::Failed
        );
    }
    assert_eq!(
        finding_status(Gate::Advisories, Some(0), "", ""),
        Status::Passed
    );
}

#[test]
fn retained_shear_command_is_read_only_and_findings_remain_strict() {
    assert_eq!(
        Gate::Shear.command(),
        [
            "cargo",
            "shear",
            "--locked",
            "--check-test-targets",
            "--deny-warnings",
            "--format",
            "json"
        ]
    );
    assert_eq!(Status::Passed.exit_code(), 0);
    assert_eq!(Status::Findings.exit_code(), 1);
    assert_eq!(Status::Failed.exit_code(), 2);
}

#[test]
fn launch_failure_retains_original_command_and_detail() -> Result<()> {
    let root = tempfile::tempdir()?;
    let command = vec![
        root.path()
            .join("absent-tool")
            .to_string_lossy()
            .into_owned(),
        "--json".into(),
    ];
    let invocation = invoke(root.path(), Gate::Shear, command.clone());
    assert_eq!(invocation.command, command);
    assert_eq!(invocation.exit_code, None);
    assert_eq!(invocation.signal, None);
    assert_eq!(invocation.status, Status::Failed);
    assert!(!invocation.stderr.is_empty());
    assert!(invocation.stdout.is_empty());
    let value = serde_json::to_value(invocation)?;
    assert_eq!(value["status"], "failed");
    assert!(value["exit_code"].is_null());
    Ok(())
}

#[cfg(unix)]
#[test]
fn original_finding_exit_and_signal_are_retained_separately() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir()?;
    let tool = root.path().join("tool");
    std::fs::write(
        &tool,
        "#!/bin/sh\nprintf '%s\\n' '{\"findings\":[{\"code\":\"shear/shared_path\",\"severity\":\"warning\"}],\"summary\":{\"errors\":0,\"warnings\":1}}'\nprintf 'original diagnostic\\n' >&2\nexit 1\n",
    )?;
    std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700))?;
    let finding = invoke(
        root.path(),
        Gate::Shear,
        vec![tool.to_string_lossy().into_owned()],
    );
    assert_eq!(finding.status, Status::Findings);
    assert_eq!(finding.exit_code, Some(1));
    assert_eq!(finding.stderr, "original diagnostic\n");
    std::fs::write(&tool, "#!/bin/sh\nkill -TERM $$\n")?;
    let killed = invoke(
        root.path(),
        Gate::Shear,
        vec![tool.to_string_lossy().into_owned()],
    );
    assert_eq!(killed.status, Status::Failed);
    assert_eq!(killed.exit_code, None);
    assert_eq!(killed.signal, Some(15));
    Ok(())
}
