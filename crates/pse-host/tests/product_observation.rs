// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual captured reader and CLI against a source-built runtime-owned publication fixture.
//! Native fixture-owner inputs are required; passive reads never prepare them.
use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
use pse_host::{environment::Environment, product_reader::ProductReaderSelection};
use serde_json::{Value, json, value::RawValue};
use sha2::{Digest as _, Sha256};
use std::{
    io::Read as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[path = "product_observation/follow.rs"]
mod follow;

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
    let original_launch = std::env::var("PSE_MANAGED_INVOCATION")
        .expect("declared native workflow publishes its original managed launch routing hint");
    assert_eq!(
        delegated["payload"]["native"]["state"], "observed",
        "{delegated}"
    );
    assert_eq!(
        delegated["payload"]["native"]["candidate"]["invocation"],
        original_launch
    );
    let native = &delegated["payload"]["native"]["facts"];
    assert_eq!(native["observation"]["service"]["active_state"], "active");
    assert_eq!(
        native["observation"]["service"]["invocation"]
            .as_array()
            .unwrap()
            .len(),
        16
    );
    assert_eq!(native["native_drained"], false);
    assert_eq!(native["accounting"]["state"], "available", "{native}");
    assert_eq!(native["accounting"]["facts"]["atomic_snapshot"], false);
    assert!(native["accounting"]["facts"]["service_result"].is_string());
    let native_invocation: String = native["observation"]["service"]["invocation"]
        .as_array()
        .unwrap()
        .iter()
        .map(|byte| format!("{:02x}", byte.as_u64().unwrap()))
        .collect();
    assert!(
        delegated["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["argv"]
                == json!(["pse", "activity", native["observation"]["service"]["unit"]]))
    );
    assert!(delegated["next"].as_array().unwrap().iter().any(|action| {
        action["argv"].as_array().is_some_and(|argv| {
            argv.iter()
                .any(|argument| argument == &format!("_SYSTEMD_INVOCATION_ID={native_invocation}"))
        })
    }));
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
    let unprepared = tempfile::tempdir().unwrap();
    let help = pse()
        .env("XDG_STATE_HOME", unprepared.path())
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success(), "{help:?}");
    let unavailable = pse()
        .env("XDG_STATE_HOME", unprepared.path())
        .args(["observe", "discover", "--reader", &reader])
        .output()
        .unwrap();
    assert!(!unavailable.status.success());
    assert_eq!(
        std::fs::read_dir(unprepared.path()).unwrap().count(),
        0,
        "passive unprepared reads and help create no replacement reader/state"
    );
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
    let original_image: Value = serde_json::from_slice(&original).unwrap();
    let mut expected =
        json!({"process":original_image["process"],"locator":original_image["locator"]});
    expected["locator"]["start_ticks"] =
        json!(expected["locator"]["start_ticks"].as_u64().unwrap() + 1);
    let mut wrong_current = inspect.clone();
    wrong_current["source"] =
        json!({"selection":"current","checkpoint":checkpoint,"expected":expected});
    let refused_identity = query(directory.path(), &reader, &wrong_current, false);
    assert_eq!(refused_identity["payload"]["category"], "refused");
    let copied = directory.path().join("explicit-association-copy.json");
    for mode in ["absent", "wrong_invocation", "wrong_boot", "wrong_start"] {
        let mut image = original_image.clone();
        match mode {
            "absent" => image["managed_invocation"] = Value::Null,
            "wrong_invocation" => {
                image["managed_invocation"]["invocation"] =
                    json!("00000000-0000-4000-8000-000000000000")
            }
            "wrong_boot" => image["managed_invocation"]["process"]["boot_id"] = json!("wrong-boot"),
            "wrong_start" => image["managed_invocation"]["process"]["start_ticks"] = json!(0),
            _ => unreachable!(),
        }
        std::fs::write(&copied, image.to_string()).unwrap();
        let mut selected_copy = inspect.clone();
        selected_copy["source"]["checkpoint"] = json!(copied);
        let observed = query(directory.path(), &reader, &selected_copy, false);
        assert_eq!(
            observed["payload"]["category"], "success",
            "{mode}: {observed}"
        );
        assert_eq!(
            observed["payload"]["product"]["document"]["operation"]["id"],
            supplied["operation"]
        );
        assert_eq!(
            observed["payload"]["native"]["state"],
            if matches!(mode, "absent" | "wrong_start") {
                "absent"
            } else {
                "rejected"
            },
            "{mode}: {observed}"
        );
    }
    let catalogue = query(
        directory.path(),
        &reader,
        &json!({"operation":"processes","directory":Path::new(checkpoint).parent().unwrap(),"limit":1,"cursor":null}),
        false,
    );
    assert_eq!(
        catalogue["payload"]["product"]["page"]["processes"][0]["process"],
        original_image["process"]
    );
    let related_request = json!({"operation":"page","source":source,"operation_id":supplied["operation"],"section":"related","producer":null,"limit":1,"cursor":null});
    let first_related = query(directory.path(), &reader, &related_request, false);
    assert!(first_related["payload"]["product"]["page"]["continuation"].is_object());
    let mut next_related = related_request.clone();
    next_related["cursor"] = first_related["payload"]["product"]["page"]["continuation"].clone();
    let second_related = query(directory.path(), &reader, &next_related, false);
    assert_eq!(
        first_related["payload"]["product"]["page"]["records"]["records"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        second_related["payload"]["product"]["page"]["records"]["records"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_ne!(
        first_related["payload"]["product"]["page"]["records"]["records"][0]["id"],
        second_related["payload"]["product"]["page"]["records"]["records"][0]["id"]
    );
    let mut waiting = inspect.clone();
    waiting["request"]["operation"] = supplied["waiter"].clone();
    waiting["request"]["include_related"] = json!(true);
    let waiting = query(directory.path(), &reader, &waiting, false);
    assert_eq!(
        waiting["payload"]["product"]["document"]["operation"]["id"],
        supplied["waiter"]
    );
    assert_eq!(
        waiting["payload"]["product"]["document"]["related"][0]["id"],
        supplied["preparation"]
    );
    assert_eq!(
        waiting["payload"]["product"]["document"]["operation"]["children"][0]["relationship"],
        "SharedProduction"
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
    let capture_document = captured["payload"]["product"]["result"]["document"].clone();
    let timeline = capture_document["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|output| output["kind"] == "Perfetto" && output["readable"] == true)
        .unwrap();
    let prefix = timeline["readable_prefix_bytes"].as_u64().unwrap();
    assert!(prefix > 0);
    let mut original_raw = Vec::new();
    std::fs::File::open(timeline["path"].as_str().unwrap())
        .unwrap()
        .take(prefix)
        .read_to_end(&mut original_raw)
        .unwrap();
    assert_eq!(original_raw.len() as u64, prefix);
    let original_digest: String = Sha256::digest(&original_raw)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(capture_document["symbols"]["checked"], false);
    let window_length = prefix.min(16);
    for offset in [
        (prefix / 3).min(prefix - window_length),
        prefix - window_length,
    ] {
        let output = pse()
            .args(["artifact", "read"])
            .arg(timeline["path"].as_str().unwrap())
            .args([
                "--offset",
                &offset.to_string(),
                "--length",
                &window_length.to_string(),
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let read: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            read["payload"]["byte_range"],
            json!({"start":offset,"end":offset+window_length})
        );
        assert_eq!(read["payload"]["atomic_snapshot"], false);
        let digest: String =
            Sha256::digest(&original_raw[offset as usize..(offset + window_length) as usize])
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
        assert_eq!(read["payload"]["selected_sha256"], digest);
    }
    let mutable_copy = directory.path().join("mutable-capture-copy");
    std::fs::write(&mutable_copy, &original_raw).unwrap();
    let before = pse()
        .args(["artifact", "read"])
        .arg(&mutable_copy)
        .args(["--length", "1"])
        .output()
        .unwrap();
    assert!(before.status.success(), "{before:?}");
    let before: Value = serde_json::from_slice(&before.stdout).unwrap();
    let validator = before["payload"]["etag"].as_str().unwrap();
    let mut changed = original_raw.clone();
    changed.extend_from_slice(b"changed fixture tail");
    std::fs::write(&mutable_copy, changed).unwrap();
    let changed = pse()
        .args(["artifact", "read"])
        .arg(&mutable_copy)
        .args(["--length", "1", "--if-match", validator])
        .output()
        .unwrap();
    assert!(!changed.status.success());
    assert!(!changed.stderr.is_empty());
    assert_eq!(
        std::fs::read(timeline["path"].as_str().unwrap()).unwrap()[..prefix as usize],
        original_raw
    );

    let unaligned_path = directory.path().join("unaligned-checkpoint.json");
    let mut unaligned = original_image.clone();
    unaligned["clock"] = Value::Null;
    std::fs::write(&unaligned_path, unaligned.to_string()).unwrap();
    let mut unaligned_capture = capture.clone();
    unaligned_capture["request"]["request"]["checkpoint"] = json!(unaligned_path);
    unaligned_capture["request"]["request"]["output"] =
        json!(directory.path().join("unaligned-capture"));
    let unaligned = query(directory.path(), &reader, &unaligned_capture, true);
    assert_eq!(unaligned["payload"]["category"], "success", "{unaligned}");
    let unaligned = &unaligned["payload"]["product"]["result"]["document"];
    assert_eq!(unaligned["loss"]["unknown_clock"], true);
    assert!(
        unaligned["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|target| target["clock_aligned"] == false
                && target["clock_uncertainty_nanoseconds"].is_null())
    );

    // Use the already captured product through its native transport API to select an
    // actually absent optional perf executable; the source/scientific selection is unchanged.
    let no_tools = directory.path().join("empty-tool-path");
    std::fs::create_dir(&no_tools).unwrap();
    let mut without_tools = environment.clone();
    without_tools.insert("PATH".into(), no_tools.into_os_string());
    let mut missing_tool = capture.clone();
    missing_tool["request"]["request"]["kind"] = json!("Cpu");
    missing_tool["request"]["request"]["output"] =
        json!(directory.path().join("missing-tool-capture"));
    let missing_tool = selected
        .investigate(
            &RawValue::from_string(missing_tool.to_string()).unwrap(),
            "missing-optional-tool",
            &without_tools,
        )
        .unwrap();
    let missing_tool: Value = serde_json::from_str(missing_tool.payload().get()).unwrap();
    assert_eq!(
        missing_tool["result"]["document"]["status"], "Failed",
        "{missing_tool}"
    );
    assert!(
        !missing_tool["result"]["document"]["errors"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        missing_tool["result"]["document"]["symbols"]["checked"],
        false
    );

    let interrupted_path = directory.path().join("source-interrupted-capture");
    let mut interrupted = capture.clone();
    interrupted["request"]["request"]["output"] = json!(interrupted_path);
    interrupted["request"]["request"]["duration"] = json!({"secs":1,"nanos":0});
    let request_path = directory.path().join("interrupted-capture-request.json");
    std::fs::write(&request_path, interrupted.to_string()).unwrap();
    let mut command = pse();
    command
        .args(["observe", "investigate", "--reader", &reader, "--request"])
        .arg(request_path)
        .stdout(Stdio::piped());
    let mut collector = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Park,
        Some(pse_diagnostics::StderrCapture {
            bytes: 65536,
            stack_bytes: 256 << 10,
        }),
    )
    .unwrap();
    let stdout = collector.take_stdout().unwrap();
    let draining = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(1 << 20).read_to_end(&mut bytes).unwrap();
        bytes
    });
    wait(&interrupted_path.join("timeline.pftrace"));
    let temporarily_absent = directory.path().join("temporarily-absent-checkpoint");
    std::fs::rename(checkpoint, &temporarily_absent).unwrap();
    let ready = collector.wait_ready(None, Some(Instant::now() + Duration::from_secs(5)));
    let status = collector.cancel();
    let bytes = draining.join().unwrap();
    std::fs::rename(&temporarily_absent, checkpoint).unwrap();
    ready.unwrap();
    assert!(status.unwrap().success());
    let interrupted: Value = serde_json::from_slice(&bytes).unwrap();
    let interrupted = &interrupted["payload"]["product"]["result"]["document"];
    assert!(
        matches!(interrupted["status"].as_str(), Some("Partial" | "Failed")),
        "{interrupted}"
    );
    assert!(!interrupted["errors"].as_array().unwrap().is_empty());
    assert!(owner.try_wait().unwrap().is_none());
    let still_readable = query(directory.path(), &reader, &inspect, false);
    assert_eq!(still_readable["payload"]["category"], "success");
    assert_eq!(std::fs::read(checkpoint).unwrap(), original);
    let partial_path = directory.path().join("partial-capture");
    let mut curtailed = capture.clone();
    curtailed["request"]["request"]["output"] = json!(partial_path);
    curtailed["request"]["request"]["budget"] = json!({"memory_bytes":16777216,"output_bytes":32});
    let curtailed = query(directory.path(), &reader, &curtailed, true);
    assert_eq!(curtailed["payload"]["category"], "success", "{curtailed}");
    let partial_document = &curtailed["payload"]["product"]["result"]["document"];
    assert!(
        matches!(
            partial_document["status"].as_str(),
            Some("Partial" | "Failed")
        ),
        "{partial_document}"
    );
    assert_eq!(partial_document["loss"]["output_exhausted"], true);
    assert!(owner.try_wait().unwrap().is_none());
    assert_eq!(
        std::fs::read(checkpoint).unwrap(),
        original,
        "capture failure/partial output never publishes or completes original work"
    );
    let frozen_path = directory.path().join("frozen.json");
    let mut frozen: Value = serde_json::from_slice(&std::fs::read(&frozen_path).unwrap()).unwrap();
    frozen["captures"] = json!([capture_document, partial_document]);
    std::fs::write(&frozen_path, serde_json::to_vec(&frozen).unwrap()).unwrap();
    let pack = directory.path().join("selected-pack");
    let export = json!({"operation":"investigate","request":{"action":"export","evidence":frozen_path,"request":{"version":2,"operation":supplied["operation"],"baseline":null,"numerical_history_policy":null},"memory_bytes":134217728,"destination":pack,"selection":{"version":2,"members":[{"id":"timeline","role":"capture","required":true,"format":"perfetto","document_version":null,"source":"explicit selected finalized capture prefix","references":[{"member":"frozen","pointer":"/captures/0"}],"result":null,"capture_scope":format!("original status {}; source loss {}",capture_document["status"],capture_document["loss"])}],"maximum_output_bytes":268435456,"maximum_document_bytes":16777216},"files":[{"member":"timeline","path":timeline["path"],"readable_prefix_bytes":prefix}]}});
    let exported = query(directory.path(), &reader, &export, true);
    assert_eq!(exported["payload"]["category"], "success", "{exported}");
    assert!(pack.join("index.json").exists());
    std::fs::rename(&pack, directory.path().join("portable-pack")).unwrap();
    std::fs::rename(
        &capture_path,
        directory.path().join("capture-original-unavailable"),
    )
    .unwrap();
    std::fs::rename(
        &partial_path,
        directory.path().join("partial-original-unavailable"),
    )
    .unwrap();
    std::fs::remove_file(&frozen_path).unwrap();
    std::fs::rename(
        checkpoint,
        directory.path().join("checkpoint-original-unavailable"),
    )
    .unwrap();
    std::fs::write(directory.path().join("verify-portable"), []).unwrap();
    wait(&directory.path().join("portable.ready"));
    let portable: Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("portable.ready")).unwrap())
            .unwrap();
    assert_eq!(portable["operation"], supplied["operation"]);
    assert_eq!(portable["captures"], 2);
    assert_eq!(portable["raw_bytes"], prefix);
    assert_eq!(portable["raw_sha256"], original_digest);
    assert_eq!(portable["original_paths_available"], false);
    assert!(owner.try_wait().unwrap().is_none());
    assert!(!Path::new(checkpoint).exists());
    std::fs::write(directory.path().join("finish"), []).unwrap();
    wait(&directory.path().join("terminal.ready"));
    let terminal = query(directory.path(), &reader, &follow, false);
    assert_eq!(terminal["payload"]["product"]["sample"]["terminal"], true);
    assert_eq!(
        terminal["payload"]["product"]["sample"]["observation"]["document"]["operation"]["outcome"],
        "Failed"
    );
    assert!(terminal["payload"]["next_request"].is_null());
    let stale_page = query(directory.path(), &reader, &next_related, false);
    assert_eq!(stale_page["payload"]["category"], "refused");
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
