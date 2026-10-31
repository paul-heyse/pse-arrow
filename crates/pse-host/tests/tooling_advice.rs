// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public hook protocol controls, without relying on a running agent host.
#![cfg(test)]
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn invoke(input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pse"))
        .args(["agent-hook", "tooling-advice"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn both_host_payloads_get_only_advice_and_unknown_input_is_quiet() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    for (tool, key) in [("Bash", "command"), ("exec_command", "cmd")] {
        let output = invoke(&serde_json::to_vec(&serde_json::json!({"cwd":root,"tool_name":tool,"tool_input":{key:"cargo test -p pse-host"}})).unwrap());
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let detail = &value["hookSpecificOutput"];
        assert_eq!(detail.as_object().unwrap().len(), 2);
        assert_eq!(detail["hookEventName"], "PreToolUse");
        assert!(
            detail["additionalContext"]
                .as_str()
                .unwrap()
                .contains("pse test")
        );
    }
    for input in [b"invalid".as_slice(), b"{}", &vec![b'x'; 256 * 1024 + 1]] {
        let output = invoke(input);
        assert!(output.status.success());
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
    let other = tempfile::tempdir().unwrap();
    let output = invoke(
        &serde_json::to_vec(
            &serde_json::json!({"cwd":other.path(),"tool_input":{"command":"cargo test"}}),
        )
        .unwrap(),
    );
    assert!(output.status.success() && output.stdout.is_empty());
}
