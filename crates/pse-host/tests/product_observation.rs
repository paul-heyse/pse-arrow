// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual captured reader and CLI against a source-built runtime-owned publication fixture.
//! Native fixture-owner inputs are required; passive reads never prepare them.
use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
use pse_host::{environment::Environment, product_reader::ProductReaderSelection};
use serde_json::{Value, json, value::RawValue};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}
fn pse() -> Command {
    let mut command = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse").expect("actual selected native CLI executable"),
    );
    command.current_dir(root()).env("PSE_FORMAT", "json");
    command
}
fn wait(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "fixture handoff {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn query(directory: &Path, reader: &str, request: &Value, investigation: bool) -> Value {
    let path = directory.join("request.json");
    std::fs::write(&path, serde_json::to_vec(request).unwrap()).unwrap();
    let output = pse()
        .args([
            "observe",
            if investigation {
                "investigate"
            } else {
                "query"
            },
            "--reader",
            reader,
            "--request",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(matches!(output.status.code(), Some(0 | 2)), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn prepared_cli_follows_original_work_and_explicitly_captures_and_exports_selected_evidence() {
    let reader = std::env::var("PSE_NATIVE_FIXTURE_PRODUCT_READER")
        .expect("run through the declared native product-observation fixture owner");
    let fixture = std::env::var_os("PSE_NATIVE_FIXTURE_OBSERVATION_HELPER")
        .expect("native fixture owner supplies the source-built product publication helper");
    let directory = tempfile::tempdir().unwrap();
    let mut command = Command::new(fixture);
    command.arg(directory.path()).stdout(Stdio::null());
    let mut owner = SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, None).unwrap();
    wait(&directory.path().join("active.ready"));
    let supplied: Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("source.json")).unwrap())
            .unwrap();
    let checkpoint = supplied["checkpoint"].as_str().unwrap();
    let original = std::fs::read(checkpoint).unwrap();
    let source = json!({"selection":"retained","checkpoint":checkpoint,"expected":null});
    let inspect = json!({"operation":"inspect","source":source,"request":{"version":2,"operation":supplied["operation"],"window":{"secs":30,"nanos":0},"include_related":false,"related_limit":1},"invocation":null});
    let delegated = query(directory.path(), &reader, &inspect, false);
    let first_observer = delegated["invocation_id"].clone();
    let environment: Environment = std::env::vars_os().collect();
    let selected = ProductReaderSelection::open(&root(), &environment, &reader).unwrap();
    let discovery = pse()
        .args(["observe", "discover", "--reader", &reader])
        .output()
        .unwrap();
    assert!(discovery.status.success(), "{discovery:?}");
    let discovery: Value = serde_json::from_slice(&discovery.stdout).unwrap();
    assert_eq!(discovery["payload"]["product"]["kind"], "discovery");
    for (direction, contract) in [
        ("request", pse_host::product_reader::Contract::Request),
        ("response", pse_host::product_reader::Contract::Response),
    ] {
        let schema = pse()
            .args(["observe", "schema", "--reader", &reader, direction])
            .output()
            .unwrap();
        assert!(schema.status.success(), "{schema:?}");
        let schema: Value = serde_json::from_slice(&schema.stdout).unwrap();
        assert_eq!(
            schema,
            serde_json::from_str::<Value>(selected.schema(contract).unwrap().get()).unwrap()
        );
    }
    let direct = selected
        .read(
            &RawValue::from_string(inspect.to_string()).unwrap(),
            "direct-original",
            &environment,
        )
        .unwrap();
    let direct: Value = serde_json::from_str(direct.payload().get()).unwrap();
    let delegated = &delegated["payload"]["product"];
    assert_eq!(
        delegated["source"]["identity"],
        direct["source"]["identity"]
    );
    assert_eq!(delegated["process"], direct["process"]);
    assert_eq!(
        delegated["document"]["operation"]["outcome"],
        direct["document"]["operation"]["outcome"]
    );
    let another = query(directory.path(), &reader, &inspect, false);
    assert_ne!(another["invocation_id"], first_observer);
    assert_eq!(
        another["payload"]["product"]["document"]["operation"]["id"],
        direct["document"]["operation"]["id"]
    );
    let records = query(
        directory.path(),
        &reader,
        &json!({"operation":"page","source":source,"operation_id":supplied["operation"],"section":"records","producer":null,"limit":1,"cursor":null}),
        false,
    );
    let page = &records["payload"]["product"]["page"];
    assert_eq!(page["partial"], true);
    assert!(page["total"].is_null());
    let fields = page["records"]["records"][0]["values"].as_array().unwrap();
    assert!(fields.iter().any(|field| field["name"] == "obj_value"
        && field["unit"] == "native_objective"
        && field["value"]["value"] == 17.25));
    assert!(
        fields
            .iter()
            .any(|field| field["name"] == "inf_pr" && field["value"]["kind"] == "NonFinite")
    );
    let refused = query(
        directory.path(),
        &reader,
        &json!({"operation":"unknown_product_operation"}),
        false,
    );
    assert_eq!(refused["payload"]["category"], "refused");
    assert_eq!(refused["payload"]["product"]["kind"], "refusal");
    let follow = json!({"operation":"follow","source":source,"request":inspect["request"],"invocation":null,"resume":null});
    let request_path = directory.path().join("follow.json");
    std::fs::write(&request_path, follow.to_string()).unwrap();
    let followed = pse()
        .args([
            "--format",
            "ndjson",
            "observe",
            "follow",
            "--reader",
            &reader,
            "--request",
        ])
        .arg(&request_path)
        .args(["--samples", "2", "--interval-ms", "1", "--timeout", "5"])
        .output()
        .unwrap();
    assert!(followed.status.success(), "{followed:?}");
    let frames: Vec<Value> = followed
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0]["payload"]["observer_state"], "observing");
    assert_eq!(
        frames[1]["payload"]["product"]["sample"]["continuity"],
        "unchanged"
    );
    assert_eq!(frames[2]["payload"]["observer_state"], "stopped");
    assert_eq!(frames[2]["payload"]["summary"]["stop"], "sample_limit");
    assert!(owner.try_wait().unwrap().is_none());
    assert_eq!(std::fs::read(checkpoint).unwrap(), original);
    let capture_path = directory.path().join("capture");
    let capture = json!({"operation":"investigate","request":{"action":"capture","request":{"version":1,"checkpoint":checkpoint,"operation":supplied["operation"],"kind":"Timeline","output":capture_path,"duration":{"secs":0,"nanos":25000000}}}});
    let passive = query(directory.path(), &reader, &capture, false);
    assert_eq!(passive["payload"]["category"], "refused");
    assert!(!capture_path.exists());
    let captured = query(directory.path(), &reader, &capture, true);
    assert_eq!(captured["payload"]["category"], "success", "{captured}");
    assert!(
        !captured["payload"]["product"]["result"]["document"]["outputs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(owner.try_wait().unwrap().is_none());
    let pack = directory.path().join("selected-pack");
    let export = json!({"operation":"investigate","request":{"action":"export","evidence":directory.path().join("frozen.json"),"request":{"version":2,"operation":supplied["operation"],"baseline":null,"numerical_history_policy":null},"memory_bytes":134217728,"destination":pack,"selection":{"version":2,"members":[],"maximum_output_bytes":268435456,"maximum_document_bytes":16777216},"files":[]}});
    let exported = query(directory.path(), &reader, &export, true);
    assert_eq!(exported["payload"]["category"], "success", "{exported}");
    assert!(pack.join("index.json").exists());
    std::fs::write(directory.path().join("finish"), []).unwrap();
    wait(&directory.path().join("terminal.ready"));
    let terminal = query(directory.path(), &reader, &follow, false);
    assert_eq!(terminal["payload"]["product"]["sample"]["terminal"], true);
    assert_eq!(
        terminal["payload"]["product"]["sample"]["observation"]["document"]["operation"]["outcome"],
        "Failed"
    );
    assert!(terminal["payload"]["next_request"].is_null());
    std::fs::write(directory.path().join("expire"), []).unwrap();
    wait(&directory.path().join("expired.ready"));
    let expired = query(directory.path(), &reader, &follow, false);
    assert_eq!(expired["payload"]["product"]["kind"], "unavailable");
    std::fs::write(directory.path().join("close"), []).unwrap();
    owner
        .wait_ready(None, Some(Instant::now() + Duration::from_secs(5)))
        .unwrap();
    assert!(owner.wait().unwrap().success());
}
