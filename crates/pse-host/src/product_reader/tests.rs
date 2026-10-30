// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::{collections::BTreeSet, os::unix::fs::PermissionsExt};

fn frame(payload: &str, correlation: &str, category: &str) -> Vec<u8> {
    format!(r#"{{"protocol":"pse.product-reader","version":1,"correlation":"{correlation}","schema":"urn:pse:product-reader:v1:response","category":"{category}","payload":{payload}}}"#).into_bytes()
}
fn output(bytes: Vec<u8>, status: i32) -> Output {
    Output {
        stdout_bytes: bytes.len() as u64,
        stdout: bytes,
        status,
        stderr: b"original product diagnostic".to_vec(),
    }
}
#[test]
fn opaque_forwarding_keeps_duplicate_keys_numeric_lexemes_and_failed_science() {
    let payload = r#"{"original_outcome":"failed","value":1.0000000000000000000001e+7,"value":340282366920938463463374607431768211455,"unknown":[true,null]}"#;
    let original = frame(payload, "reader-a", "success");
    let reply = decode(output(original.clone(), 0), "reader-a").unwrap();
    assert_eq!(reply.category(), Category::Success);
    assert_eq!(reply.payload().get(), payload);
    assert_eq!(reply.frame(), original);
    assert_eq!(reply.stderr(), b"original product diagnostic");
    let refusal = r#"{"kind":"refusal","failure":{"code":"owner.original","message":"original refusal","facts":{"x":1,"x":2}}}"#;
    let reply = decode(output(frame(refusal, "reader-b", "refused"), 2), "reader-b").unwrap();
    assert_eq!(reply.category(), Category::Refused);
    assert_eq!(reply.payload().get(), refusal);
}
#[test]
fn transport_mismatch_and_malformed_frames_preserve_original_bytes() {
    let good = frame(r#"{"scientific":"opaque"}"#, "expected", "success");
    for bad in [
        good.replace_bytes(b"\"version\":1", b"\"version\":9"),
        good.replace_bytes(b"pse.product-reader", b"other.product-reader"),
        good.replace_bytes(b"v1:response", b"v9:response"),
        good.replace_bytes(b"expected", b"wrong"),
        b"original malformed product frame".to_vec(),
    ] {
        let error = decode(output(bad.clone(), 0), "expected").unwrap_err();
        let ProductReaderError::Transport { stdout, stderr, .. } = error else {
            panic!("transport error")
        };
        assert_eq!(stdout, bad);
        assert_eq!(stderr, b"original product diagnostic");
    }
    assert!(decode(output(good, 2), "expected").is_err());
    let mut oversized = output(frame("null", "expected", "success"), 0);
    oversized.stdout_bytes = RESPONSE_BYTES as u64 + 1;
    assert!(decode(oversized, "expected").is_err());
}
trait ReplaceBytes {
    fn replace_bytes(&self, from: &[u8], to: &[u8]) -> Vec<u8>;
}
impl ReplaceBytes for Vec<u8> {
    fn replace_bytes(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        String::from_utf8(self.clone())
            .unwrap()
            .replacen(
                std::str::from_utf8(from).unwrap(),
                std::str::from_utf8(to).unwrap(),
                1,
            )
            .into_bytes()
    }
}
#[test]
fn native_helper_joins_both_streams_and_timeout_leaves_original_source_unchanged() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.checkpoint");
    fs::write(&original, b"original scientific owner").unwrap();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "cat; printf original-stderr >&2"])
        .current_dir(root.path())
        .env_clear();
    let completed = execute(
        command,
        b"exact-input".to_vec(),
        1024,
        Some(Duration::from_secs(5)),
    )
    .unwrap();
    assert_eq!(completed.status, 0);
    assert_eq!(completed.stdout, b"exact-input");
    assert_eq!(completed.stderr, b"original-stderr");
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "printf before-timeout; printf original-error >&2; sleep 60",
        ])
        .current_dir(root.path())
        .env_clear();
    let error = execute(command, Vec::new(), 1024, Some(Duration::from_millis(100))).unwrap_err();
    let ProductReaderError::Execution {
        failures,
        stdout,
        stderr,
    } = error
    else {
        panic!("observer failure")
    };
    assert!(failures.iter().any(|error| matches!(
        error,
        ProductReaderError::Supervision(SupervisionError::Deadline)
    )));
    assert_eq!(stdout, b"before-timeout");
    assert_eq!(stderr, b"original-error");
    assert_eq!(fs::read(&original).unwrap(), b"original scientific owner");
}
#[test]
fn independent_input_and_observer_failures_both_survive_owned_task_settlement() {
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "exec 0<&-; printf original-stderr >&2; sleep 60"])
        .env_clear();
    let error = execute(
        command,
        vec![0; 2 * 1024 * 1024],
        1024,
        Some(Duration::from_millis(100)),
    )
    .unwrap_err();
    let ProductReaderError::Execution {
        failures, stderr, ..
    } = &error
    else {
        panic!("aggregate execution failure")
    };
    assert!(failures.iter().any(|error|matches!(error,ProductReaderError::Io(error) if error.kind()==std::io::ErrorKind::BrokenPipe)));
    assert!(failures.iter().any(|error| matches!(
        error,
        ProductReaderError::Supervision(SupervisionError::Deadline)
    )));
    assert_eq!(stderr, b"original-stderr");
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_children(&error)
            .unwrap()
            .count(),
        failures.len()
    );
}
#[test]
fn reader_prefix_drains_every_byte_and_keeps_original_partial_read_error() {
    struct Broken {
        first: bool,
    }
    impl Read for Broken {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            if self.first {
                self.first = false;
                bytes[..4].copy_from_slice(b"kept");
                Ok(4)
            } else {
                Err(std::io::Error::from_raw_os_error(5))
            }
        }
    }
    let (bytes, total, error) = prefix(Broken { first: true }, 2);
    assert_eq!(bytes, b"ke");
    assert_eq!(total, 4);
    assert_eq!(error.unwrap().raw_os_error(), Some(5));
    let (bytes, total, error) = prefix(std::io::Cursor::new(vec![b'x'; 10000]), 32);
    assert_eq!(bytes.len(), 32);
    assert_eq!(total, 10000);
    assert!(error.is_none());
}
fn artifact(path: &Path) -> cargo_metadata::Artifact {
    serde_json::from_value(serde_json::json!({"package_id":"path+file:///workspace/runtime#pse-runtime@0.0.1","manifest_path":"/workspace/runtime/Cargo.toml","target":{"name":"pse-project","kind":["bin"],"crate_types":["bin"],"src_path":"/workspace/runtime/main.rs","edition":"2024"},"profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["product-reader"],"filenames":[path],"executable":path,"fresh":false})).unwrap()
}
fn build(directory: &Path, executable: Executable) -> BuildCapture {
    let cargo = artifact(&directory.join("mutable-target"));
    BuildCapture {
        version: 1,
        resolution: correctness::Resolution {
            arguments: vec!["build".into()],
            packages: vec!["pse-runtime".into()],
            units: vec![correctness::CompilerUnit {
                package_id: cargo.package_id.repr.clone(),
                package_name: "pse-runtime".into(),
                target_name: "pse-project".into(),
                target_kinds: vec!["bin".into()],
                source_path: cargo.target.src_path.to_string(),
                features: BTreeSet::from(["product-reader".into()]),
                platform: None,
                mode: "build".into(),
                root: true,
            }],
            native_capabilities: BTreeSet::new(),
        },
        artifacts: vec![crate::discovery::BuiltArtifact { cargo, executable }],
        cargo_exit: 0,
        incomplete: Default::default(),
    }
}
#[test]
fn recorded_executable_path_and_package_identity_cannot_redirect_selection() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path();
    fs::create_dir(directory.join("build")).unwrap();
    let source = directory.join("native");
    fs::write(&source, b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
    let executable =
        crate::discovery::capture::executable(&source, &directory.join("build/artifact-0"))
            .unwrap();
    let mut build = build(directory, executable);
    select_executable(&build, directory).unwrap();
    build.resolution.units[0].package_id = "different source identity".into();
    assert!(select_executable(&build, directory).is_err());
    build.resolution.units[0].package_id = build.artifacts[0].cargo.package_id.repr.clone();
    build.artifacts[0].executable.captured = source;
    assert!(select_executable(&build, directory).is_err());
    build.cargo_exit = 7;
    assert!(select_executable(&build, directory).is_err());
}
#[test]
fn unprepared_passive_selection_reads_no_host_generation_and_creates_no_state() {
    let root = tempfile::tempdir().unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let environment = Environment::from([
        ("HOME".into(), root.path().as_os_str().to_owned()),
        (
            "XDG_STATE_HOME".into(),
            root.path().join("state").into_os_string(),
        ),
    ]);
    assert!(
        ProductReaderSelection::open(checkout, &environment, &uuid::Uuid::new_v4().to_string())
            .is_err()
    );
    assert!(
        ProductReaderSelection::open(checkout, &environment, "../../arbitrary-reader").is_err()
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}
#[test]
fn captured_directional_schema_and_discovery_require_native_transport_associations() {
    let document = r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"urn:pse:product-reader:v1:request","type":"object","properties":{"value":{"type":"number"}}}"#;
    let native = frame(
        &format!(r#"{{"kind":"schema","identity":"{REQUEST_SCHEMA}","document":{document}}}"#),
        "prepare-request-schema",
        "success",
    );
    assert_eq!(
        schema_document(&native, REQUEST_SCHEMA).unwrap().get(),
        document
    );
    assert!(schema_document(&native, RESPONSE_SCHEMA).is_err());
    let schema: serde_json::Value = serde_json::from_str(document).unwrap();
    assert!(
        jsonschema::validator_for(&schema)
            .unwrap()
            .is_valid(&serde_json::json!({"value":7}))
    );
    let discovery = frame(
        &format!(
            r#"{{"kind":"discovery","request_schema":"{REQUEST_SCHEMA}","response_schema":"{RESPONSE_SCHEMA}","request_bytes":{REQUEST_BYTES},"response_bytes":{RESPONSE_BYTES}}}"#
        ),
        "prepare-discovery",
        "success",
    );
    discovery_document(&discovery).unwrap();
    assert!(discovery_document(&discovery.replace_bytes(b"v1:request", b"v2:request")).is_err());
}

#[test]
fn actual_captured_adapter_build_retains_original_product_artifact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir(scratch.path().join("src")).unwrap();
    fs::write(scratch.path().join("Cargo.toml"),"[package]\nname='pse-runtime'\nversion='0.0.1'\nedition='2024'\n[features]\nproduct-reader=[]\n[[bin]]\nname='pse-project'\npath='src/main.rs'\n").unwrap();
    fs::write(scratch.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::copy(
        root.join("rust-toolchain.toml"),
        scratch.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let mut environment =
        correctness::standalone_adapter_environment(&std::env::vars_os().collect()).unwrap();
    let cargo = correctness::real_cargo_in(&environment).unwrap();
    let generated = Command::new(&cargo)
        .args(["generate-lockfile", "--offline"])
        .current_dir(scratch.path())
        .env_clear()
        .envs(&environment)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let build_directory = scratch.path().join("build");
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&build_directory)
        .unwrap();
    let generation = crate::generation::prepared(root).unwrap().unwrap();
    let tools = root.join("target/host-tools");
    let usage = crate::generation::GenerationUse::acquire(&tools, &generation.generation).unwrap();
    let control = crate::discovery::capture::executable(
        &tools
            .join(&generation.generation)
            .join("cargo-pse-correctness"),
        &scratch.path().join("cargo-pse-correctness"),
    )
    .unwrap();
    usage.settle().unwrap();
    environment.insert(
        "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
        build_directory.clone().into_os_string(),
    );
    let output = execute(
        source_build_command(&control, scratch.path(), &environment),
        Vec::new(),
        STDERR_BYTES,
        None,
    )
    .unwrap();
    assert_eq!(
        output.status,
        0,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (_, build) = build_evidence(&build_directory).unwrap();
    let executable = select_executable(&build, scratch.path()).unwrap();
    assert_eq!(build.cargo_exit, 0);
    assert!(build.incomplete.is_empty());
    assert!(build.resolution.native_capabilities.is_empty());
    executable.verify().unwrap();
    fs::write(&executable.original, b"replaced mutable target").unwrap();
    executable.verify().unwrap();
    assert!(
        fs::metadata(build_directory.join("cargo-artifacts.jsonl"))
            .unwrap()
            .len()
            > 0
    );
}

#[test]
fn missing_build_capture_reports_its_original_path_and_io_cause() {
    let root = tempfile::tempdir().unwrap();
    let error = build_evidence(root.path()).unwrap_err();
    let ProductReaderError::BuildEvidence { path, source } = &error else {
        panic!("capture path context")
    };
    assert_eq!(path, &root.path().join("build.json"));
    assert!(
        matches!(source.as_ref(),ProductReaderError::Io(error) if error.kind()==std::io::ErrorKind::NotFound)
    );
    assert!(error.to_string().contains(path.to_str().unwrap()));
}
