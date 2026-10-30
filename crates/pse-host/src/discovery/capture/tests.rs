// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;

pub(crate) fn artifact(path: &Path, name: &str, kind: &str) -> Artifact {
    serde_json::from_value(serde_json::json!({
        "package_id":"path+file:///workspace/alpha#0.1.0", "manifest_path":"/workspace/alpha/Cargo.toml",
        "target":{"name":name,"kind":[kind],"crate_types":["bin"],"src_path":format!("/workspace/alpha/{name}.rs"),"edition":"2024"},
        "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":true},
        "features":["feature-a"],"filenames":[path],"executable":path,"fresh":false
    })).unwrap()
}
pub(crate) fn resolution() -> Resolution {
    Resolution {
        arguments: vec!["test".into()],
        packages: vec!["alpha".into()],
        units: vec![],
        native_capabilities: BTreeSet::new(),
    }
}
fn source(root: &Path) -> PathBuf {
    let source = root.join("original");
    fs::write(&source, b"#!/bin/sh\nprintf original").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
    source
}
#[test]
fn retained_executable_survives_original_replacement_and_detects_own_tampering() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let captured = executable(&source, &root.path().join("captured")).unwrap();
    fs::write(&source, b"replacement").unwrap();
    captured.verify().unwrap();
    assert_eq!(
        fs::read(&captured.captured).unwrap(),
        b"#!/bin/sh\nprintf original"
    );
    fs::set_permissions(&captured.captured, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(&captured.captured, b"tampered").unwrap();
    assert!(captured.verify().is_err());
}
#[test]
fn source_symlinks_and_existing_capture_destinations_refuse() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let link = root.path().join("link");
    std::os::unix::fs::symlink(&source, &link).unwrap();
    assert!(executable(&link, &root.path().join("captured")).is_err());
    assert!(executable(&source, &source).is_err());
}
#[test]
fn executable_is_captured_before_its_cargo_record_is_forwarded_and_child_exit_is_retained() {
    struct ReplaceOnForward {
        source: PathBuf,
        bytes: Vec<u8>,
    }
    impl Write for ReplaceOnForward {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            fs::write(&self.source, b"overwritten by following build")?;
            self.bytes.extend(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let captured = root.path().join("build");
    fs::create_dir(&captured).unwrap();
    fs::set_permissions(&captured, fs::Permissions::from_mode(0o700)).unwrap();
    let message = Message::CompilerArtifact(artifact(&source, "test_target", "test"));
    let bytes = serde_json::to_string(&message).unwrap();
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "printf '%s\\n' \"$1\"; exit 7", "fixture", &bytes]);
    let mut forwarded = ReplaceOnForward {
        source,
        bytes: vec![],
    };
    assert_eq!(
        execute_build(command, &resolution(), &captured, &mut forwarded).unwrap(),
        7
    );
    assert_eq!(forwarded.bytes, format!("{bytes}\n").as_bytes());
    let build: BuildCapture =
        serde_json::from_slice(&fs::read(captured.join("build.json")).unwrap()).unwrap();
    assert_eq!(build.cargo_exit, 7);
    assert_eq!(build.artifacts.len(), 1);
    assert_eq!(
        build.artifacts[0].executable.captured.file_name(),
        build.artifacts[0].executable.original.file_name(),
        "native owners retain the original Cargo executable basename"
    );
    assert_eq!(build.artifacts[0].cargo.features, ["feature-a"]);
    assert_eq!(
        fs::read(&build.artifacts[0].executable.captured).unwrap(),
        b"#!/bin/sh\nprintf original"
    );
}
#[test]
fn forwarding_failure_still_waits_and_retains_original_child_status() {
    struct Closed;
    impl Write for Closed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("receiver closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let root = tempfile::tempdir().unwrap();
    let mut command = Command::new("/bin/sh");
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    command.args(["-c", "printf 'original raw line\\n'; exit 9"]);
    let error = execute_build(command, &resolution(), root.path(), &mut Closed).unwrap_err();
    assert!(error.to_string().contains("Cargo exited 9"), "{error}");
    let build: BuildCapture =
        serde_json::from_slice(&fs::read(root.path().join("build.json")).unwrap()).unwrap();
    assert_eq!(build.cargo_exit, 9);
    assert_eq!(build.incomplete.len(), 1);
    assert!(
        build
            .incomplete
            .contains_key(&CaptureFailureDomain::Forwarding)
    );
    assert_eq!(
        fs::read(root.path().join("cargo-artifacts.jsonl")).unwrap(),
        b"original raw line\n"
    );
}

#[test]
fn failed_sinks_are_not_retried_and_failure_retention_does_not_grow_per_line() {
    #[derive(Default)]
    struct Closed {
        attempts: usize,
    }
    impl Write for Closed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            self.attempts += 1;
            Err(io::Error::other("closed sink"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let bytes = b"ordinary output\n".repeat(50_000);
    let mut input = io::Cursor::new(&bytes);
    let mut raw = Closed::default();
    let mut forwarded = Closed::default();
    let mut failures = Failures::new();
    drain(
        &mut input,
        &mut raw,
        &mut forwarded,
        Path::new("/unused"),
        &mut vec![],
        &mut failures,
    )
    .unwrap();
    assert_eq!(input.position(), bytes.len() as u64);
    assert_eq!((raw.attempts, forwarded.attempts), (1, 1));
    assert_eq!(failures.len(), 2);
    assert!(failures.contains_key(&CaptureFailureDomain::Storage));
    assert!(failures.contains_key(&CaptureFailureDomain::Forwarding));
}

#[test]
fn oversized_record_is_streamed_in_full_and_capture_remains_incomplete() {
    #[derive(Default)]
    struct Count(usize);
    impl Write for Count {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            assert!(bytes.iter().all(|byte| *byte == b'x' || *byte == b'\n'));
            self.0 += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let size = MAX_RECORD_BYTES * 3 + 19;
    let mut input = io::repeat(b'x')
        .take(size as u64)
        .chain(io::Cursor::new(b"\n"));
    let mut raw = Count::default();
    let mut forwarded = Count::default();
    let mut failures = Failures::new();
    let mut artifacts = vec![];
    drain(
        &mut input,
        &mut raw,
        &mut forwarded,
        Path::new("/unused"),
        &mut artifacts,
        &mut failures,
    )
    .unwrap();
    assert_eq!((raw.0, forwarded.0), (size + 1, size + 1));
    assert!(artifacts.is_empty());
    assert_eq!(failures.len(), 1);
    assert!(failures.contains_key(&CaptureFailureDomain::Protocol));
}

#[test]
fn malformed_required_record_and_zero_child_exit_do_not_create_a_complete_capture() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let mut command = Command::new("/bin/sh");
    command.args([
        "-c",
        "printf '%s\\n' '{\"reason\":\"compiler-artifact\",\"package_id\":3}'; exit 0",
    ]);
    let mut forwarded = vec![];
    assert!(execute_build(command, &resolution(), root.path(), &mut forwarded).is_err());
    let captured: BuildCapture =
        serde_json::from_slice(&fs::read(root.path().join("build.json")).unwrap()).unwrap();
    assert_eq!(captured.cargo_exit, 0);
    assert!(
        captured
            .incomplete
            .contains_key(&CaptureFailureDomain::Protocol)
    );
    assert_eq!(
        forwarded,
        fs::read(root.path().join("cargo-artifacts.jsonl")).unwrap()
    );
}
