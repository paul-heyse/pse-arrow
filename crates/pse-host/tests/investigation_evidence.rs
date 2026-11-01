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
#[cfg(test)]
fn qualified_value(output: Output) -> Value {
    assert!(matches!(output.status.code(), Some(0 | 1)), "{output:?}");
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["outcome"], "success", "{response}");
    assert!(
        response["failures"].as_array().unwrap().is_empty(),
        "{response}"
    );
    if !output.status.success() {
        assert_eq!(
            response["observation"]["completeness"], "partial",
            "{response}"
        );
    }
    response
}

#[test]
fn engineering_quantity_inquiry_hands_off_opaque_source_without_native_preparation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let state = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        pse(root)
            .env("XDG_STATE_HOME", state.path())
            .args(args)
            .output()
            .unwrap()
    };
    let found = value(run(&[
        "search",
        "symbol",
        "QuantityRegistryBuilder",
        "--path",
        "crates/pse-quantity/src/registry.rs",
        "--limit",
        "2",
    ]));
    let reference = &found["payload"]["results"][0]["reference"];
    let address = reference["address"]
        .as_str()
        .expect("native publisher supplies an opaque source address");
    let inspected = value(run(&["inspect", "--result", address]));
    assert!(!inspected["payload"].is_null());
    let human = run(&["inspect", "--result", address, "--format", "human"]);
    assert!(human.status.success(), "{human:?}");
    assert!(
        String::from_utf8(human.stdout)
            .unwrap()
            .contains("QuantityRegistryBuilder")
    );
    let streamed = run(&[
        "search",
        "symbol",
        "QuantityRegistryBuilder",
        "--path",
        "crates/pse-quantity/src/registry.rs",
        "--format",
        "ndjson",
        "--limit",
        "1",
    ]);
    assert!(streamed.status.success(), "{streamed:?}");
    let events = String::from_utf8(streamed.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        events.iter().any(|event| event["event"] == "result"),
        "{events:?}"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "terminal")
            .count(),
        1,
        "{events:?}"
    );
    let relevance = run(&[
        "inspect",
        "--source",
        "crates/pse-quantity/src/registry.rs",
        "--view",
        "relevance",
        "--budget",
        "3",
    ]);
    assert!(
        matches!(relevance.status.code(), Some(0 | 1)),
        "{relevance:?}"
    );
    let relevance: Value = serde_json::from_slice(&relevance.stdout).unwrap();
    assert!(relevance["payload"]["structure"].is_object(), "{relevance}");
    assert!(relevance["payload"]["native_tests"].is_null());
    let described = value(run(&["commands", "search", "--limit", "1"]));
    assert!(!described["payload"].is_null());
    let invalid = run(&[
        "inspect",
        "--source",
        "crates/pse-quantity/src/registry.rs",
        "--fields",
        "records.path",
    ]);
    assert!(!invalid.status.success());
    let invalid: Value = serde_json::from_slice(&invalid.stderr).unwrap();
    assert!(
        invalid["failures"]
            .as_array()
            .is_some_and(|failures| !failures.is_empty())
    );
    let incompatible = run(&[
        "inspect",
        "--source",
        "missing.rs",
        "--view",
        "relevance",
        "--adjacency",
        "contains",
    ]);
    assert!(!incompatible.status.success());
    let incompatible: Value = serde_json::from_slice(&incompatible.stderr).unwrap();
    assert!(
        incompatible["failures"][0]["message"]
            .as_str()
            .unwrap()
            .contains("--adjacency requires"),
        "{incompatible}"
    );
    let passive = run(&[
        "inspect",
        "--source",
        "missing.rs",
        "--fields",
        "records.path",
    ]);
    assert_eq!(passive.status.code(), Some(2), "{passive:?}");
    let passive: Value = serde_json::from_slice(&passive.stderr).unwrap();
    assert!(
        passive["failures"][0]
            .to_string()
            .contains("Valid choices:"),
        "{passive}"
    );
    let missing_selector = run(&["inspect", "--source", "missing.rs", "--view", "hint"]);
    assert_eq!(
        missing_selector.status.code(),
        Some(2),
        "{missing_selector:?}"
    );
    let missing_selector: Value = serde_json::from_slice(&missing_selector.stderr).unwrap();
    assert!(
        missing_selector["failures"][0]["message"]
            .as_str()
            .unwrap()
            .contains("requires --select"),
        "{missing_selector}"
    );
    let adjacency_fields = run(&[
        "inspect",
        "--source",
        "missing.rs",
        "--view",
        "adjacency",
        "--fields",
        "graph.nodes",
    ]);
    assert_eq!(
        adjacency_fields.status.code(),
        Some(2),
        "{adjacency_fields:?}"
    );
    let adjacency_fields: Value = serde_json::from_slice(&adjacency_fields.stderr).unwrap();
    assert!(
        adjacency_fields["failures"][0]
            .to_string()
            .contains("Valid choices:"),
        "{adjacency_fields}"
    );
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

#[test]
fn relevance_exposes_shared_expectations_positional_assertions_and_opaque_macros_without_preparation()
 {
    let root = tempfile::tempdir().unwrap();
    let source = "fn calculation(v:u32)->u32 {v+1}\nfn comparator(_left:u32,_right:u32)->bool {true}\nmacro_rules! opaque { ($a:expr,$b:expr)=>{} }\nfn first(){let expected=calculation(2);let actual=calculation(2);assert_eq!(actual,expected);assert!(comparator(actual,expected));}\nfn second(){assert!(comparator(calculation(4),99));opaque!(4,99);}\n";
    std::fs::write(root.path().join("source.rs"), source).unwrap();
    let response = value(
        pse(root.path())
            .args(["inspect", "--source", "source.rs", "--view", "relevance"])
            .output()
            .unwrap(),
    );
    let report = &response["payload"];
    assert!(report["original"].is_null());
    assert!(report["native_tests"].is_null());
    assert!(report["native_calls"].is_null());
    let structure = &report["structure"];
    assert_eq!(structure["bindings"].as_array().unwrap().len(), 2);
    assert_eq!(structure["assertions"][0]["operands"][0]["text"], "actual");
    assert_eq!(
        structure["assertions"][0]["operands"][1]["text"],
        "expected"
    );
    assert!(
        structure["assertions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["interpretation"]
                .as_str()
                .unwrap()
                .starts_with("opaque_custom_macro"))
    );
    assert!(
        report["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("oracle-independence"))
    );
    let reference = report["reference"]["address"].as_str().unwrap();
    std::fs::write(root.path().join("source.rs"), source.replace("v+1", "v+2")).unwrap();
    let stale = pse(root.path())
        .args(["inspect", "--result", reference, "--view", "relevance"])
        .output()
        .unwrap();
    assert!(!stale.status.success(), "{stale:?}");
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join("Cargo.lock").exists());
}

#[test]
fn associations_keep_rejected_alternative_citations_separate_from_declared_authority() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("source.rs"),
        "/// Rejected alternative [Other](missing.md); see blueprint §24.2.\nfn selected() {}\n",
    )
    .unwrap();
    let response = qualified_value(
        pse(root.path())
            .args(["inspect", "--source", "source.rs", "--view", "associations"])
            .output()
            .unwrap(),
    );
    let report = &response["payload"];
    assert_eq!(report["owning_plan"]["status"], "not_declared");
    assert!(
        report["document"].is_null(),
        "unprepared interpreter must remain unavailable: {response}"
    );
    assert!(
        report["citations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["written"] == "missing.md" && row["resolved"].is_null())
    );
    assert!(
        report["citations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["written"] == "blueprint §24.2")
    );
    assert!(report["reference"]["locator"]["guard"]["sha256"].is_string());
    let address = report["reference"]["address"].as_str().unwrap();
    std::fs::write(root.path().join("source.rs"), "fn changed() {}\n").unwrap();
    let stale = pse(root.path())
        .args(["inspect", "--result", address, "--view", "associations"])
        .output()
        .unwrap();
    assert!(!stale.status.success(), "{stale:?}");
    let stale: Value = serde_json::from_slice(&stale.stderr).unwrap();
    assert!(!stale["failures"].as_array().unwrap().is_empty(), "{stale}");
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join("Cargo.lock").exists());
}

#[test]
fn associations_read_comment_and_literal_doc_attribute_citations_from_original_facts() {
    let root = tempfile::tempdir().unwrap();
    let prose = "Rejected alternative [Other](missing.md); see blueprint §24.2.";
    for (path, source) in [
        ("comment.rs", format!("/// {prose}\nfn selected() {{}}\n")),
        (
            "attribute.rs",
            format!("#[doc = {prose:?}]\nfn selected() {{}}\n"),
        ),
    ] {
        std::fs::write(root.path().join(path), source).unwrap();
        let response = qualified_value(
            pse(root.path())
                .args(["inspect", "--source", path, "--view", "associations"])
                .output()
                .unwrap(),
        );
        let report = &response["payload"];
        let written = report["citations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["written"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            written,
            ["missing.md", "blueprint §24.2"].into_iter().collect(),
            "{response}"
        );
        assert!(
            report["documentation"]
                .as_array()
                .unwrap()
                .iter()
                .any(|fact| !fact["value"].is_null()),
            "{response}"
        );
        assert_eq!(report["owning_plan"]["status"], "not_declared");
    }
    assert!(!root.path().join("target").exists());
}

fn original_relevance_setup() -> (u32, u32) {
    (2, 9)
}
fn original_relevance_comparator(actual: u32, expected: u32) -> bool {
    actual == expected
}

/// Explicit failing child control for the public original-outcome/relevance journey.
/// Run only by selecting this exact ignored fixture through the real native test owner;
/// its expected failure creates original custody, never a synthetic performing report.
#[test]
#[ignore = "explicit original-failure evidence control; intentionally fails when selected"]
fn original_relevance_failure_fixture() {
    let (actual, expected) = original_relevance_setup();
    assert!(
        original_relevance_comparator(actual, expected),
        "intentional original relevance control"
    );
}
