// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual CLI framing, schema selection and raw-protocol boundaries.
#![cfg(test)]
use assert_cmd::assert::Assert;
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Output},
};
fn pse() -> Command {
    configured_pse(
        std::env::var_os("CARGO_BIN_EXE_pse")
            .expect("Cargo/nextest supplies the selected runtime CLI executable"),
    )
}
fn checkout() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}
fn prepared_pse() -> Command {
    let root = checkout();
    let generation = pse_host::generation::prepared(root)
        .unwrap()
        .expect("explicit host preparation is required for effectful CLI qualification");
    configured_pse(
        generation
            .executable(root, pse_host::generation::HostTool::Pse)
            .unwrap(),
    )
}
fn configured_pse(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    // Cargo injects its selected toolchain into tests. Independent child
    // construction selects the checkout pin; all ownership/placement stays inherited.
    command
        .current_dir(checkout())
        .env("PSE_FORMAT", "json")
        .env_remove("RUSTUP_TOOLCHAIN");
    command
}
fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("{error}: {output:?}"))
}
#[test]
fn usage_errors_have_structured_context_in_the_requested_format() {
    let state = tempfile::tempdir().unwrap();
    let output = pse()
        .env("XDG_STATE_HOME", state.path())
        .args(["--format", "json", "--unknown-option"])
        .output()
        .unwrap();
    let assertion = Assert::new(output).failure().code(2).stdout(b"".as_slice());
    let output = assertion.get_output();
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(value["outcome"], "usage");
    assert_eq!(value["failures"][0]["code"], "cli.usage");
    assert_eq!(
        value["failures"][0]["parser_context"]["error_kind"],
        "UnknownArgument"
    );
    assert!(
        value["invocation_id"]
            .as_str()
            .is_some_and(|value| uuid::Uuid::parse_str(value).is_ok())
    );
    let early = pse()
        .env("PSE_FORMAT", "human")
        .args(["run", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(early.status.code(), Some(2));
    let _: serde_json::Value = serde_json::from_slice(&early.stderr).unwrap();
    assert_eq!(
        std::fs::read_dir(state.path()).unwrap().count(),
        0,
        "invalid arguments must acquire no runtime or retained-state effects"
    );
}

#[test]
fn settings_field_projection_is_passive_and_keeps_the_selected_schema() {
    let state = tempfile::tempdir().unwrap();
    let output = pse()
        .env("XDG_STATE_HOME", state.path())
        .env("XDG_CONFIG_HOME", state.path())
        .args(["--fields", "checkout", "inspect", "settings"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert_eq!(value["command"], "inspect settings");
    assert!(value["payload"]["checkout"].is_string());
    assert_eq!(value["payload"].as_object().unwrap().len(), 1);
    assert_eq!(value["artifacts"], serde_json::json!([]));
    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), 0);
}

#[test]
fn routine_inspection_projection_uses_native_views_without_retention() {
    let state = tempfile::tempdir().unwrap();
    let invoke = |args: &[&str]| {
        pse()
            .env("XDG_STATE_HOME", state.path())
            .args(args)
            .output()
            .unwrap()
    };
    let read = [
        "inspect",
        "dependency-units",
        "arrow-array",
        "pse-columnar",
        "--lib",
    ];
    let full = invoke(&read);
    assert!(full.status.success(), "{full:?}");
    let full = json(&full);
    let view = invoke(
        &[
            read.as_slice(),
            &[
                "--fields",
                "dependency,contexts,original_metadata",
                "--budget",
                "0",
            ],
        ]
        .concat(),
    );
    assert!(view.status.success(), "{view:?}");
    let view = json(&view);
    assert_eq!(view["payload"]["dependency"], full["payload"]["dependency"]);
    assert_eq!(view["payload"]["contexts"], serde_json::json!([]));
    assert!(view["payload"].get("original_metadata").is_none());
    assert_eq!(
        view["view"]["omitted_records"],
        full["payload"]["contexts"].as_array().unwrap().len()
    );
    for key in [
        "outcome",
        "subject_state",
        "observation",
        "failures",
        "next",
        "artifacts",
    ] {
        assert_eq!(view[key], full[key], "{key}");
    }
    let missing = invoke(&[
        "inspect",
        "dependency-units",
        "pse-missing-projection-fixture",
        "--fields",
        "dependency,contexts",
    ]);
    assert!(missing.status.success(), "{missing:?}");
    let missing = json(&missing);
    assert_eq!(missing["subject_state"], "no_match");
    assert!(missing["payload"]["contexts"].is_null());
    for (route, arguments, field) in [
        ("dependency", &["arrow-array"][..], "query"),
        ("affected", &[][..], "base_commit"),
        ("changes", &[][..], "groups"),
        ("owner", &["Cargo.toml"][..], "source"),
        (
            "check-selection",
            &["pse-columnar", "--lib", "--full"][..],
            "stage",
        ),
        ("test-selection", &["pse-columnar", "--lib"][..], "stage"),
    ] {
        let result = invoke(&[&["inspect", route][..], arguments, &["--fields", field]].concat());
        assert!(result.status.success(), "{route}: {result:?}");
        let result = json(&result);
        assert_eq!(result["payload"].as_object().unwrap().len(), 1, "{route}");
        assert_eq!(result["artifacts"], serde_json::json!([]), "{route}");
    }
    for (args, message) in [
        (
            &[
                "inspect",
                "dependency-units",
                "arrow-array",
                "--fields",
                "unknown-view-field",
            ][..],
            "unknown payload field",
        ),
        (
            &[
                "inspect",
                "dependency-usage",
                "fake-package",
                "--fields",
                "unknown-view-field",
            ][..],
            "unknown payload field",
        ),
        (
            &["inspect", "changes", "--budget", "1"][..],
            "original path indices",
        ),
        (
            &["inspect", "tests", "nonexistent", "--budget", "1"][..],
            "--page-bytes",
        ),
        (
            &[
                "inspect",
                "tests",
                "nonexistent",
                "--failures",
                "--budget",
                "1",
            ][..],
            "--page-bytes",
        ),
    ] {
        let result = invoke(args);
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(result.stdout.is_empty(), "{result:?}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(message),
            "{result:?}"
        );
    }
    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), 0);
}

#[test]
fn jq_adapter_delivers_filtered_data_and_keeps_command_failures_separate() {
    let state = tempfile::tempdir().unwrap();
    let invoke = |filter, read: &[&str]| {
        pse()
            .env("XDG_STATE_HOME", state.path())
            .args(["jq", "--filter", filter, "--"])
            .args(read)
            .output()
            .unwrap()
    };
    let settings = ["inspect", "settings", "--fields", "checkout"];
    for (filter, value) in [
        (".payload.checkout | type", "\"string\"\n"),
        ("false", "false\n"),
        ("null", "null\n"),
        ("empty", ""),
    ] {
        let output = invoke(filter, &settings);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, value.as_bytes());
        assert!(output.stderr.is_empty(), "{output:?}");
    }
    let producer = invoke(
        "error(\"formatter started\")",
        &["inspect", "settings", "--fields", "unknown-view-field"],
    );
    assert_eq!(producer.status.code(), Some(2));
    assert!(producer.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&producer.stderr);
    assert!(stderr.contains("unknown payload field"), "{stderr}");
    assert!(stderr.contains("producer exit 2; jq skipped"), "{stderr}");
    assert!(!stderr.contains("formatter started"), "{stderr}");
    for (filter, status) in [(".", 0), ("{", 3), ("1, error(\"runtime fixture\")", 5)] {
        let output = invoke(filter, &settings);
        assert_eq!(output.status.code(), Some(status), "{output:?}");
        if status != 0 {
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains(&format!("producer exit 0; jq exit {status}")),
                "{output:?}"
            );
        }
    }
    for read in [
        &["test", "pse-host"][..],
        &["inspect", "settings", "--format", "human"][..],
    ] {
        let output = invoke(".", read);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
    }
    let missing = pse()
        .env("PATH", state.path())
        .env("XDG_STATE_HOME", state.path())
        .args(["jq", "--filter", ".", "--", "inspect", "settings"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(125), "{missing:?}");
    assert!(String::from_utf8_lossy(&missing.stderr).contains("producer exit 0; jq"));
    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), 0);
}

#[test]
fn jq_adapter_interrupts_the_original_formatter_and_discards_partial_output() {
    use rustix::process::{Pid, Signal, kill_process};
    use std::{
        fs,
        os::unix::fs::PermissionsExt as _,
        process::Stdio,
        time::{Duration, Instant},
    };

    let fixture = tempfile::tempdir().unwrap();
    let ready = fixture.path().join("ready");
    let formatter = fixture.path().join("jq");
    fs::write(&formatter, "#!/bin/sh\nprintf 'partial'\nprintf '%s\\n' \"$$\" > \"$PSE_JQ_TEST_READY\"\nexec /bin/sleep 20\n").unwrap();
    fs::set_permissions(&formatter, fs::Permissions::from_mode(0o700)).unwrap();
    let child = pse()
        .env("PATH", fixture.path())
        .env("PSE_JQ_TEST_READY", &ready)
        .args(["jq", "--filter", ".", "--", "inspect", "settings"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let formatter_pid = loop {
        if let Ok(pid) = fs::read_to_string(&ready)
            && let Ok(pid) = pid.trim().parse::<i32>()
        {
            break Some(pid);
        }
        if Instant::now() >= deadline {
            break None;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    let _ = kill_process(pid, Signal::INT);
    let output = child.wait_with_output().unwrap();
    let formatter_pid = formatter_pid.expect("formatter started before interruption");
    assert_eq!(output.status.code(), Some(130), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("interrupted by signal 2"));
    assert!(!Path::new("/proc").join(formatter_pid.to_string()).exists());
}

#[test]
fn source_change_status_reports_original_stage_evidence_passively() {
    use pse_host::paths::CheckoutState;
    use std::{
        fs::{self, OpenOptions},
        io::Write as _,
        os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _},
    };

    let state = tempfile::tempdir().unwrap();
    let root = CheckoutState::resolve(
        checkout(),
        &[("XDG_STATE_HOME".into(), state.path().into())]
            .into_iter()
            .collect(),
    )
    .unwrap()
    .root;
    let id = "1234567890abcdef1234567890abcdef";
    let directory = root.join("source-change").join(id);
    fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&directory)
        .unwrap();
    let record = directory.join("record.json");
    for (state_name, inflight, completed, recorded) in [
        (
            "generating",
            Some("prepare-host"),
            Some("snapshot"),
            Some("prepare-host"),
        ),
        (
            "generating",
            Some("receiving-tests"),
            Some("affected-selection"),
            Some("receiving-tests"),
        ),
        ("applying", None, Some("retain-generated-delta"), None),
        (
            "generation_failed",
            Some("full-codegen"),
            Some("surreal"),
            None,
        ),
        ("ready", None, Some("retain-generated-delta"), None),
        ("generating", None, None, None),
    ] {
        let failure = (state_name == "generation_failed").then(|| {
            serde_json::json!({
                "code": "source_change.stage_failed", "stage": "full-codegen",
            })
        });
        let mut value = serde_json::json!({
            "checkout": checkout(),
            "status": {"version": 1, "preview_id": id, "state": state_name, "failure": failure},
        });
        if let Some(inflight) = inflight {
            value["inflight_stage"] = inflight.into();
        }
        if let Some(completed) = completed {
            value["status"]["completed_stages"] = serde_json::json!(["earlier", completed]);
        }
        let bytes = serde_json::to_vec(&value).unwrap();
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&record)
            .unwrap()
            .write_all(&bytes)
            .unwrap();
        for format in ["json", "ndjson"] {
            let output = pse()
                .env("XDG_STATE_HOME", state.path())
                .args(["--format", format, "source-change", "status", id])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(i32::from(state_name != "ready")),
                "{output:?}"
            );
            assert!(output.stderr.is_empty(), "{output:?}");
            let response = json(&output);
            assert!(response["child_exit"].is_null());
            assert_eq!(response["payload"]["preview_id"], id);
            assert_eq!(response["payload"]["recorded_stage"].as_str(), recorded);
            assert_eq!(
                response["payload"]["last_completed_stage"].as_str(),
                completed
            );
            assert_eq!(
                response["payload"]["receipt"]["failure"],
                value["status"]["failure"]
            );
            assert_eq!(fs::read(&record).unwrap(), bytes);
        }
    }
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
    assert_eq!(fs::read_dir(directory).unwrap().count(), 1);
}

#[test]
fn missing_fixture_readiness_keeps_owner_diagnostic_and_exact_preparation() {
    let state = tempfile::tempdir().unwrap();
    let help = pse()
        .env("XDG_STATE_HOME", state.path())
        .args(["help", "prepare", "host"])
        .output()
        .unwrap();
    let help = json(&help);
    assert_eq!(help["payload"]["records"][0]["effect"], "prepare");
    {
        let fixture = "study-module";
        let output = pse()
            .env("XDG_STATE_HOME", state.path())
            .env_remove("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
            .args(["fixture", "show", fixture])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(125));
        assert!(output.stdout.is_empty());
        let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["subject_state"], "unprepared");
        assert_eq!(diagnostic["failures"][0]["code"], "fixture.unprepared");
        assert_eq!(
            diagnostic["next"][0]["argv"],
            serde_json::json!([
                checkout().join("scripts/pse"),
                "fixture",
                "prepare",
                fixture,
            ])
        );
    }
    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), 0);
}

#[test]
fn surrealkit_readiness_only_offers_install_for_the_absent_default() {
    let data = tempfile::tempdir().unwrap();
    for (selected, expected, state) in [
        (None, "missing_default", "unprepared"),
        (
            Some(data.path().join("unavailable")),
            "rejected_override",
            "unavailable",
        ),
    ] {
        let mut command = pse();
        command.env("XDG_DATA_HOME", data.path());
        match selected {
            Some(path) => command.env("PSE_SURREALKIT_BIN", path),
            None => command.env_remove("PSE_SURREALKIT_BIN"),
        };
        let output = command
            .args(["diagnostic-tools", "verify", "surrealkit"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(125), "{output:?}");
        assert!(output.stdout.is_empty());
        let response: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(response["subject_state"], state);
        assert_eq!(
            response["failures"][0]["facts"]["observations"]["preparation_state"]["value"],
            expected
        );
        if expected == "missing_default" {
            assert_eq!(
                response["next"][0]["argv"],
                serde_json::json!([
                    checkout().join("scripts/pse"),
                    "diagnostic-tools",
                    "install",
                    "surrealkit",
                ])
            );
        } else {
            assert_eq!(response["next"], serde_json::json!([]));
        }
        assert_eq!(std::fs::read_dir(data.path()).unwrap().count(), 0);
    }
}

#[test]
fn retained_result_cli_reads_and_exports_one_selected_original() {
    use pse_host::{
        envelope::{CliInvocationId, Envelope, Settlement, SubjectState},
        generation::PreparedGeneration,
        invocation::{InvocationSpec, InvocationStore},
        paths::CheckoutState,
    };
    use std::{fs, os::unix::fs::PermissionsExt as _};

    let state = tempfile::tempdir().unwrap();
    let selection = CheckoutState::resolve(
        checkout(),
        &[("XDG_STATE_HOME".into(), state.path().into())]
            .into_iter()
            .collect(),
    )
    .unwrap();
    let root = selection.root.clone();
    let store = InvocationStore::new(selection);
    let id = CliInvocationId::new();
    let original = store
        .begin(InvocationSpec {
            invocation_id: id.clone(),
            command: "run".into(),
            session: Some("export-fixture".into()),
            parent: None,
            operation: (),
            control: PreparedGeneration {
                generation: "generation.fixture".into(),
                protocol: 1,
                manifest_digest: "a".repeat(64),
            },
            environment_digest: "b".repeat(64),
            scientific: Vec::new(),
        })
        .unwrap();
    let result = original.path().join("result.json");
    let mut terminal = Envelope::response("run", "original bounded result", "private payload");
    terminal.invocation_id = id.clone();
    terminal.subject_state = SubjectState::Succeeded;
    terminal.settlement = Settlement::Complete;
    let original_bytes = serde_json::to_vec(&terminal).unwrap();
    fs::write(&result, &original_bytes).unwrap();
    fs::set_permissions(&result, fs::Permissions::from_mode(0o600)).unwrap();
    let stream = original.path().join("stdout.raw");
    fs::write(&stream, b"0123456789").unwrap();
    fs::set_permissions(&stream, fs::Permissions::from_mode(0o600)).unwrap();
    let invoke = || {
        let mut command = pse();
        command.env("XDG_STATE_HOME", state.path());
        command
    };
    let history = json(
        &invoke()
            .args([
                "history",
                "--session",
                "export-fixture",
                "--fields",
                "records",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        history["payload"]["records"][0]["invocation_id"],
        id.as_str()
    );
    let observed = json(
        &invoke()
            .args([
                "inspect",
                "invocation",
                "latest",
                "--session",
                "export-fixture",
                "--detail",
                "provenance",
                "--fields",
                "detail",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        observed["payload"]["detail"]["environment_digest"],
        "b".repeat(64)
    );
    assert_eq!(observed["artifacts"], serde_json::json!([]));
    let log = json(
        &invoke()
            .args([
                "inspect",
                "log",
                id.as_str(),
                "--offset",
                "1",
                "--length",
                "3",
                "--fields",
                "content",
            ])
            .output()
            .unwrap(),
    );
    assert!(log["payload"]["content"].to_string().contains("123"));
    assert!(!root.join("artifacts").exists());

    let destination = state.path().join("result-export.json");
    let export = || {
        invoke()
            .args([
                "tools",
                "export-result",
                id.as_str(),
                "--detail",
                "provenance",
                "--output",
            ])
            .arg(&destination)
            .output()
            .unwrap()
    };
    let receipt = json(&export());
    assert_eq!(receipt["payload"]["original_invocation"], id.as_str());
    let exported = fs::read(&destination).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&exported).unwrap();
    assert_eq!(document["invocation_id"], receipt["invocation_id"]);
    assert_eq!(document["payload"]["detail"], observed["payload"]["detail"]);
    assert!(!export().status.success());
    assert_eq!(fs::read(&destination).unwrap(), exported);
    assert_eq!(fs::read(&result).unwrap(), original_bytes);
    assert!(!root.join("artifacts").exists());
}
#[test]
fn env_projection_retains_full_evidence_and_matches_its_projected_schema() {
    let state = tempfile::tempdir().unwrap();
    let output = pse()
        .env("XDG_STATE_HOME", state.path())
        .args([
            "env",
            "--explain",
            "--fields",
            "selections",
            "--budget",
            "0",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert_eq!(value["command"], "env explain");
    assert_eq!(value["outcome"], "success");
    assert!(value["payload"].get("local_unresolved").is_none());
    assert!(value["payload"]["selections"].is_object());
    assert_eq!(
        value["view"]["omitted_fields"],
        serde_json::json!(["local_unresolved"])
    );
    let artifact = value["artifacts"][0]["location"].as_str().unwrap();
    assert!(Path::new(artifact).starts_with(state.path()));
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(artifact).unwrap()).unwrap();
    assert!(original["payload"]["local_unresolved"].is_boolean());
    assert_eq!(original["invocation_id"], value["invocation_id"]);
    let schema_output = pse()
        .args([
            "schema",
            "env",
            "explain",
            "--fields",
            "selections",
            "--budget",
            "0",
        ])
        .output()
        .unwrap();
    assert!(schema_output.status.success(), "{schema_output:?}");
    let schema = json(&schema_output);
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["$id"], value["schema"]);
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validator.is_valid(&value),
        "{:?}",
        validator
            .iter_errors(&value)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );
}
#[test]
fn incomplete_activity_ndjson_is_one_valid_final_record_with_managed_failure() {
    let directory = tempfile::tempdir().unwrap();
    let output = pse()
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}/absent", directory.path().display()),
        )
        .args(["--format", "ndjson", "activity", "selected.service"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    let value = json(&output);
    assert_eq!(value["record"], "observation_summary");
    assert_eq!(value["subject_state"], "not_applicable");
    assert_eq!(value["observation"]["completeness"], "partial");
    assert!(
        !value["observation"]["limitations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let schema = json(&pse().args(["schema", "activity"]).output().unwrap());
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&value));
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}
#[test]
fn protocol_outputs_ignore_format_and_never_gain_an_envelope() {
    let version = pse()
        .env("PSE_FORMAT", "ndjson")
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(version.stdout.starts_with(b"pse "));
    assert!(version.stdout.ends_with(b"protocol 1\n"));
    let tools = pse()
        .args(["--format", "json", "cargo-tools", "cargo-nextest"])
        .output()
        .unwrap();
    assert!(tools.status.success(), "{tools:?}");
    assert!(!tools.stdout.starts_with(b"{"));
    assert!(!tools.stdout.is_empty());
    let print = pse()
        .args(["--format", "json", "env", "--print"])
        .output()
        .unwrap();
    assert!(print.status.success(), "{print:?}");
    assert!(!print.stdout.starts_with(b"{"));
    assert!(String::from_utf8_lossy(&print.stdout).contains("activation-data"));
}
#[test]
fn foreground_managed_failure_retains_literal_child_arguments_and_original_exit() {
    let state = tempfile::tempdir().unwrap();
    let output = prepared_pse()
        .env("XDG_STATE_HOME", state.path())
        .args([
            "run",
            "--",
            "/bin/sh",
            "-c",
            "printf '%s\\n' \"$@\"; exit 7",
            "child",
            "--format",
            "json",
            "-h",
        ])
        .output()
        .unwrap();
    let assertion = Assert::new(output).failure().code(1);
    let value = json(assertion.get_output());
    assert_eq!(value["outcome"], "failure");
    assert_eq!(value["child_exit"], 7);
    assert_eq!(value["settlement"], "complete");
    assert_eq!(value["observation"]["completeness"], "complete");
    assert_eq!(
        value["payload"]["stderr"]["bytes_retained"], 0,
        "bootstrap diagnostics do not enter the original child stream"
    );
    let id = value["invocation_id"].as_str().unwrap();
    let log = pse()
        .env("XDG_STATE_HOME", state.path())
        .args(["inspect", "log", id])
        .output()
        .unwrap();
    let assertion = Assert::new(log).success().stderr(b"".as_slice());
    let log = json(assertion.get_output());
    assert_eq!(
        log["payload"]["content"],
        serde_json::json!({"encoding":"utf8","data":"--format\njson\n-h\n"})
    );
    let original = pse()
        .env("XDG_STATE_HOME", state.path())
        .args(["inspect", "invocation", id])
        .output()
        .unwrap();
    // A successful passive read preserves the performing operation's failed state.
    let assertion = Assert::new(original).success();
    let original = json(assertion.get_output());
    assert_eq!(original["payload"]["response"]["child_exit"], 7);
    assert_eq!(original["payload"]["response"]["outcome"], "failure");
}

#[test]
fn native_bundle_keeps_successful_siblings_and_original_failure_in_one_owner() {
    let fixture = tempfile::tempdir().unwrap();
    let justfile = fixture.path().join("bundle.just");
    std::fs::write(
        &justfile,
        "failed:\n    @exit 3\n\npassed:\n    @printf sibling-completed\n",
    )
    .unwrap();
    let output = prepared_pse()
        .env("XDG_STATE_HOME", fixture.path().join("state"))
        .env("PSE_SESSION", "bundle-test")
        .arg("run-all")
        .arg("--justfile")
        .arg(justfile)
        .args(["failed", "passed"])
        .output()
        .unwrap();
    let assertion = Assert::new(output).failure().code(1);
    let value = json(assertion.get_output());
    assert_eq!(value["settlement"], "complete");
    assert_eq!(value["observation"]["completeness"], "complete");
    let children = value["payload"]["bundle"]["children"].as_array().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0]["result"]["outcome"], "failure");
    assert_eq!(children[0]["result"]["child_exit"], 3);
    assert_eq!(children[1]["result"]["outcome"], "success");
    let mut units = Vec::new();
    for child in children {
        let directory = Path::new(child["result"]["path"].as_str().unwrap())
            .parent()
            .unwrap();
        let metadata: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("meta.json")).unwrap()).unwrap();
        assert_eq!(metadata["selection"]["parent"], value["invocation_id"]);
        assert_eq!(metadata["selection"]["session"], "bundle-test");
        let custody: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("custody.json")).unwrap())
                .unwrap();
        units.push(custody["unit"].clone());
        if child["recipe"] == "passed" {
            assert_eq!(
                std::fs::read(directory.join("stdout.raw")).unwrap(),
                b"sibling-completed"
            );
        }
    }
    assert_eq!(
        units[0], units[1],
        "children borrow the same original native unit"
    );
}
#[test]
fn native_explanation_json_selection_uses_the_same_envelope_renderer() {
    let output = pse()
        .args([
            "--format",
            "human",
            "boundary",
            "--explain",
            "--json",
            "--native=compiler",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert_eq!(value["payload"]["native_profile"], "compiler");
    assert_eq!(value["payload"]["readiness"], "not_observed");
    assert_eq!(value["subject_state"], "not_applicable");
}
