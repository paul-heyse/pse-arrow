// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual passive consumers retain original bytes and existing source contracts.
use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Output},
};
#[cfg(test)]
fn pse(root: &Path) -> Command {
    let mut command = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse")
            .expect("native test runner supplies actual pse executable"),
    );
    command.current_dir(root).env("PSE_FORMAT", "json");
    command
}
#[cfg(test)]
fn value(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn direct_outline_and_declaration_work_without_checkout_or_analyzer_preparation() {
    let root = tempfile::tempdir().unwrap();
    let source =
        "// authored comment\nmod engine { pub fn café() { /* exact λ */ } }\nfn unfinished(\n";
    std::fs::write(root.path().join("source.rs"), source).unwrap();
    let outline = value(
        pse(root.path())
            .args(["inspect", "outline", "source.rs", "--budget", "3"])
            .output()
            .unwrap(),
    );
    assert_eq!(outline["command"], "inspect outline");
    assert!(
        outline["payload"]["outline"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path"] == serde_json::json!(["engine"]))
    );
    let declaration = value(
        pse(root.path())
            .args(["inspect", "declaration", "source.rs::engine::café"])
            .output()
            .unwrap(),
    );
    assert!(
        declaration["payload"]["selection"]["text"]
            .as_str()
            .unwrap()
            .contains("/* exact λ */")
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("source.rs")).unwrap(),
        source
    );
    assert!(!root.path().join("target").exists());
}
#[test]
fn artifact_inspection_preserves_binary_ranges_and_has_native_guarded_continuation() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("capture.bin"), [b'a', 0, 255, b'z']).unwrap();
    let response = value(
        pse(root.path())
            .args([
                "inspect",
                "artifact",
                "capture.bin",
                "--offset",
                "1",
                "--length",
                "2",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(response["payload"]["byte_range"]["start"], 1);
    assert_eq!(response["payload"]["byte_range"]["end"], 3);
    assert_eq!(response["payload"]["content"]["encoding"], "base64");
    assert_eq!(response["payload"]["content"]["data"], "AP8=");
    assert!(
        response["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|next| next["argv"][1] == "inspect" && next["argv"][2] == "artifact")
    );
    assert!(!root.path().join("target").exists());
}
#[test]
fn missing_retained_population_never_turns_into_build_or_fixture_preparation() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".config")).unwrap();
    std::fs::write(root.path().join(".config/diagnostic-tools.toml"), "").unwrap();
    let output = pse(root.path())
        .args(["inspect", "tests", "missing-original"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join("Cargo.lock").exists());
}
#[test]
fn portable_source_references_bind_the_original_root_even_when_cloned_bytes_and_times_match() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    for root in [first.path(), second.path()] {
        std::fs::create_dir(root.join(".config")).unwrap();
        std::fs::write(root.join(".config/diagnostic-tools.toml"), "").unwrap();
        std::fs::write(root.join("source.rs"), "fn needle() {}\n").unwrap();
    }
    let time = std::fs::metadata(first.path().join("source.rs"))
        .unwrap()
        .modified()
        .unwrap();
    std::fs::File::options()
        .write(true)
        .open(second.path().join("source.rs"))
        .unwrap()
        .set_modified(time)
        .unwrap();
    let response = value(
        pse(first.path())
            .args([
                "search",
                "text",
                "needle",
                "--literal",
                "--path",
                "source.rs",
                "--limit",
                "1",
            ])
            .output()
            .unwrap(),
    );
    let reference: pse_host::investigation::ResultReference =
        serde_json::from_value(response["payload"]["results"][0]["reference"].clone()).unwrap();
    let token = reference.encode().unwrap();
    let same_origin = value(
        pse(second.path())
            .args(["inspect", "--result", &token])
            .output()
            .unwrap(),
    );
    assert_eq!(same_origin["outcome"], "success");
    let wrong_root = pse(second.path())
        .arg("-C")
        .arg(second.path())
        .args(["inspect", "--result", &token])
        .output()
        .unwrap();
    assert!(
        !wrong_root.status.success(),
        "a cloned file must not make another source universe the original: {wrong_root:?}"
    );
}
#[test]
fn native_capture_template_preview_preserves_trivia_and_has_no_filesystem_effect() {
    let root = tempfile::tempdir().unwrap();
    let source = "// retained author comment\nfn f() { target(β); }\n";
    std::fs::write(root.path().join("source.rs"), source).unwrap();
    let response = value(
        pse(root.path())
            .args([
                "propose",
                "syntax",
                "source.rs",
                "--pattern",
                "target($ARG)",
                "--replacement",
                "wrapped($ARG)",
                "--select",
                "0",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(response["command"], "propose syntax");
    assert_eq!(
        response["payload"]["proposal"]["candidates"][0]["replacement"],
        "wrapped(β)"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("source.rs")).unwrap(),
        source
    );
    assert!(!root.path().join("target").exists());
}
#[test]
fn source_trait_witness_and_proposal_reach_current_native_selection_without_listing_or_execution() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".config")).unwrap();
    std::fs::create_dir(root.path().join("src")).unwrap();
    std::fs::write(root.path().join(".config/diagnostic-tools.toml"), "").unwrap();
    std::fs::write(root.path().join("Cargo.toml"),"[package]\nname=\"source-candidate\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[features]\nextra=[]\nabsent=[]\n").unwrap();
    std::fs::write(
        root.path().join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"source-candidate\"\nversion = \"0.0.0\"\n",
    )
    .unwrap();
    let source = "trait Value { fn value(&self) -> u32; }\nstruct Input;\nimpl Value for Input { fn value(&self) -> u32 { 7 } }\n";
    std::fs::write(root.path().join("src/lib.rs"), source).unwrap();
    let search = value(
        pse(root.path())
            .args([
                "search",
                "--path",
                "src/lib.rs",
                "syntax",
                "fn value(&self) -> u32 { $$$BODY }",
            ])
            .output()
            .unwrap(),
    );
    let reference: pse_host::investigation::ResultReference =
        serde_json::from_value(search["payload"]["results"][0]["reference"].clone()).unwrap();
    let reference = reference.encode().unwrap();
    let proposal = value(
        pse(root.path())
            .args([
                "propose",
                "--result",
                &reference,
                "--replacement",
                "fn value(&self) -> u32 { 8 }",
            ])
            .output()
            .unwrap(),
    );
    assert!(!proposal["payload"].is_null());
    let selection = value(
        pse(root.path())
            .args([
                "inspect",
                "test-selection",
                "--from-result",
                &reference,
                "--lib",
                "--features",
                "extra",
                "--without-feature",
                "absent",
                "-E",
                "test(value)",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(selection["payload"]["stage"], "intent");
    let selected = &selection["payload"]["selection"]["groups"][0]["roots"][0];
    assert_eq!(selected["package_name"], "source-candidate");
    assert_eq!(
        selected["features"]["enabled"],
        serde_json::json!(["extra"])
    );
    assert_eq!(
        selected["features"]["absent"],
        serde_json::json!(["absent"])
    );
    assert_eq!(
        selection["payload"]["selection"]["filters"]["expressions"],
        serde_json::json!(["test(value)"])
    );
    assert_eq!(
        selection["payload"]["selected_tests"]["availability"],
        "unavailable"
    );
    assert!(
        selection["observation"]["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row
                .as_str()
                .unwrap()
                .contains("containing-package test candidate"))
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("src/lib.rs")).unwrap(),
        source
    );
    assert!(!root.path().join("target").exists());
    for arguments in [
        vec!["--workspace"],
        vec!["source-candidate"],
        vec!["--rerun-failed", "original"],
    ] {
        let output = pse(root.path())
            .args(["inspect", "test-selection", "--from-result", &reference])
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "source candidate cannot override explicit native selection: {output:?}"
        );
    }
    let clone = tempfile::tempdir().unwrap();
    std::fs::create_dir(clone.path().join(".config")).unwrap();
    std::fs::write(clone.path().join(".config/diagnostic-tools.toml"), "").unwrap();
    let foreign = pse(root.path())
        .arg("-C")
        .arg(clone.path())
        .args(["inspect", "test-selection", "--from-result", &reference])
        .output()
        .unwrap();
    assert!(!foreign.status.success());
    assert!(
        String::from_utf8_lossy(&foreign.stderr).contains("root"),
        "{foreign:?}"
    );
    std::fs::write(root.path().join("src/lib.rs"), source.replace("7", "9")).unwrap();
    let stale = pse(root.path())
        .args(["inspect", "test-selection", "--from-result", &reference])
        .output()
        .unwrap();
    assert!(
        !stale.status.success(),
        "changed original source must refuse selection: {stale:?}"
    );
}
