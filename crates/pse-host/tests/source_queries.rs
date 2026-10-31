// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent native syntax query/preview consumer controls.
#![cfg(test)]
use std::{
    path::Path,
    process::{Command, Output},
};
fn pse(root: &Path) -> Command {
    let mut command = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse").expect("Cargo/nextest supplies native executable"),
    );
    command.current_dir(root).env("PSE_FORMAT", "json");
    command
}
fn value(output: Output) -> serde_json::Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
fn schema(root: &Path, command: &str, response: &serde_json::Value) {
    let schema = value(pse(root).args(["schema", command]).output().unwrap());
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(response)
        .unwrap();
}

#[test]
fn actual_syntax_search_retains_nested_matches_and_named_captures() {
    let root = tempfile::tempdir().unwrap();
    let source = "fn first() { call(inner(β)); }\nfn second() { call(γ); }\nfn broken(\n";
    std::fs::write(root.path().join("query.rs"), source).unwrap();
    let output = pse(root.path())
        .args([
            "search",
            "--path",
            "query.rs",
            "--all",
            "syntax",
            "$CALL($ARG)",
        ])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "malformed source cannot establish an exhaustive successful search"
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["observation"]["completeness"], "partial");
    assert_eq!(response["payload"]["summary"]["unknown"], 1);
    assert!(
        response["payload"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["code"] == "search.syntax_recovery")
    );
    let rows = response["payload"]["results"].as_array().unwrap();
    assert_eq!(rows.len(), 3, "{response}");
    assert!(
        rows.iter().any(|row| row["context"] == "inner(β)"),
        "{response}"
    );
    assert!(
        rows.iter().all(|row| row["captures"]["ARG"].is_array()),
        "{response}"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("query.rs")).unwrap(),
        source
    );
    assert!(!root.path().join("target").exists());
}

#[test]
fn actual_preview_uses_shared_proposal_and_exact_native_template_without_writes() {
    let root = tempfile::tempdir().unwrap();
    let source = "fn f() { old(α, /* keep */ β); }\n";
    std::fs::write(root.path().join("preview.rs"), source).unwrap();
    let response = value(
        pse(root.path())
            .args([
                "propose",
                "syntax",
                "--pattern",
                "old($FIRST, $$$REST)",
                "--replacement",
                "new($FIRST, $$$REST)",
                "--select",
                "0",
                "preview.rs",
            ])
            .output()
            .unwrap(),
    );
    schema(root.path(), "propose syntax", &response);
    assert_eq!(
        response["payload"]["proposal"]["producer"]["authority"],
        "syntax"
    );
    assert_eq!(
        response["payload"]["proposal"]["selected"],
        serde_json::json!([0])
    );
    let candidate = &response["payload"]["proposal"]["candidates"][0];
    let range = &candidate["range"];
    assert_eq!(
        &source[range["start"].as_u64().unwrap() as usize..range["end"].as_u64().unwrap() as usize],
        "old(α, /* keep */ β)"
    );
    assert!(
        candidate["replacement"]
            .as_str()
            .unwrap()
            .starts_with("new(α,")
    );
    assert!(candidate["replacement"].as_str().unwrap().contains("β"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("preview.rs")).unwrap(),
        source
    );
    assert!(!root.path().join(".git").exists());
}

#[test]
fn semantic_unavailability_undefined_templates_and_selected_overlaps_refuse() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("source.rs"), "fn f() { outer(inner(a)); }").unwrap();
    for (arguments, expected) in [
        (
            vec![
                "propose",
                "syntax",
                "--pattern",
                "outer($A)",
                "--replacement",
                "other($MISSING)",
                "source.rs",
            ],
            "undefined syntax capture",
        ),
        (
            vec![
                "propose",
                "syntax",
                "--pattern",
                "$CALL($ARG)",
                "--replacement",
                "new($ARG)",
                "--select",
                "0,1",
                "source.rs",
            ],
            "overlap",
        ),
    ] {
        let output = pse(root.path()).args(arguments).output().unwrap();
        assert!(!output.status.success(), "{output:?}");
        let failure: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert!(
            failure["failures"].to_string().contains(expected),
            "{failure}"
        );
    }
}

#[test]
fn generated_query_preview_origin_and_emitter_form_an_actual_source_only_consumer() {
    use pse_host::ownership::protocol::{
        GeneratorOwner, MAP_PATH, MAP_PRODUCER, OutputMap, RelationshipDocument, Scope, ScopeKind,
        SourceRelationship, VERSION,
    };
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("generated")).unwrap();
    std::fs::create_dir_all(root.path().join("emitter")).unwrap();
    let original = "fn generated() { old(β); }\n";
    std::fs::write(root.path().join("generated/hit.rs"), original).unwrap();
    std::fs::write(
        root.path().join("emitter/build.rs"),
        "pub fn emit() { /* authored emitter */ }\n",
    )
    .unwrap();
    let map = OutputMap {
        schema_version: VERSION,
        comment: "fixture declared aggregate".into(),
        owners: vec![
            GeneratorOwner {
                producer: MAP_PRODUCER.into(),
                outputs: vec![Scope {
                    path: MAP_PATH.into(),
                    kind: ScopeKind::File,
                }],
                regenerate: vec!["just".into(), "codegen".into()],
                inputs: vec![],
                contributors: None,
            },
            GeneratorOwner {
                producer: "independent-fixture".into(),
                outputs: vec![Scope {
                    path: "generated".into(),
                    kind: ScopeKind::Tree,
                }],
                regenerate: vec!["just".into(), "codegen".into()],
                inputs: vec![Scope {
                    path: "emitter".into(),
                    kind: ScopeKind::Tree,
                }],
                contributors: Some("lineage.json".into()),
            },
        ],
        coverage_limits: vec!["existing declared snapshot, not current-source inference".into()],
    };
    std::fs::create_dir_all(root.path().join(MAP_PATH).parent().unwrap()).unwrap();
    std::fs::write(
        root.path().join(MAP_PATH),
        serde_json::to_vec(&map).unwrap(),
    )
    .unwrap();
    let lineage = RelationshipDocument {
        comment: "fixture producer declarations".into(),
        schema_version: 1,
        producer: "independent-fixture".into(),
        coverage_limits: vec!["authored declaration location is unknown".into()],
        relationships: vec![SourceRelationship {
            declaration_key: "fixture/relation/v1".into(),
            declaration_kind: "relation".into(),
            source_path: None,
            source_symbol: None,
            source_role: "semantic_declaration_without_source_location".into(),
            source_identity: Some("independent-declaration-identity".into()),
            generator_path: "emitter/build.rs".into(),
            generator_symbol: "emit".into(),
            output_path: "generated/hit.rs".into(),
            direction: None,
            contract_sections: vec![],
            basis: "declared".into(),
        }],
    };
    std::fs::write(
        root.path().join("lineage.json"),
        serde_json::to_vec(&lineage).unwrap(),
    )
    .unwrap();
    let outline = value(
        pse(root.path())
            .args(["inspect", "outline", "generated/hit.rs", "--budget", "1"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        outline["payload"]["outline"]["entries"][0]["path"][0],
        "generated"
    );
    let preview = value(
        pse(root.path())
            .args([
                "propose",
                "syntax",
                "generated/hit.rs",
                "--pattern",
                "old($ARG)",
                "--replacement",
                "new($ARG)",
                "--select",
                "0",
            ])
            .output()
            .unwrap(),
    );
    let origin_argv = preview["next"][0]["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        preview["payload"]["proposal"]["candidates"][0]["replacement"],
        "new(β)"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("generated/hit.rs")).unwrap(),
        original
    );
    assert_eq!(origin_argv[0], "pse");
    let origin = value(
        pse(root.path())
            .args(origin_argv.iter().skip(1))
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect origin", &origin);
    assert_eq!(origin["payload"]["text"], "old(β)");
    assert_eq!(
        origin["payload"]["owner"]["ownership"]["owner"]["regenerate"],
        serde_json::json!(["just", "codegen"])
    );
    let relationship = &origin["payload"]["owner"]["contributors"]["records"][0];
    assert_eq!(relationship["generator_path"], "emitter/build.rs");
    assert_eq!(relationship["generator_symbol"], "emit");
    assert!(relationship["source_path"].is_null());
    assert!(relationship["source_symbol"].is_null());
    let emitter_argv = origin["next"][0]["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(emitter_argv[0], "pse");
    let emitter = value(
        pse(root.path())
            .args(emitter_argv.iter().skip(1))
            .output()
            .unwrap(),
    );
    assert_eq!(
        emitter["payload"]["outline"]["entries"][0]["path"][0],
        "emit"
    );
    std::fs::write(
        root.path().join("generated/hit.rs"),
        format!("//changed\n{original}"),
    )
    .unwrap();
    let stale = pse(root.path())
        .args(origin_argv.iter().skip(1))
        .output()
        .unwrap();
    assert!(!stale.status.success());
    let failure: serde_json::Value = serde_json::from_slice(&stale.stderr).unwrap();
    assert!(failure["failures"].to_string().contains("source changed"));
    assert!(!root.path().join("target").exists());
}
