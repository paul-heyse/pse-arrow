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

fn application_capture(directory: &Path) -> BuildCapture {
    let original = source(directory);
    let executable = executable(&original, &directory.join("captured-tool")).unwrap();
    let mut cargo = artifact(&original, "tool", "bin");
    cargo.profile.test = false;
    let mut resolution = resolution();
    resolution.units.push(crate::correctness::CompilerUnit {
        package_id: cargo.package_id.repr.clone(),
        package_name: "alpha".into(),
        target_name: "tool".into(),
        target_kinds: vec!["bin".into()],
        source_path: "/workspace/alpha/tool.rs".into(),
        features: ["feature-a".into()].into(),
        platform: None,
        mode: "build".into(),
        root: true,
    });
    BuildCapture {
        version: 1,
        resolution,
        artifacts: vec![BuiltArtifact { cargo, executable }],
        cargo_exit: 0,
        incomplete: BTreeMap::new(),
    }
}

#[test]
fn settled_failed_compiler_group_releases_images_without_inventory() {
    let scratch = tempfile::tempdir().unwrap();
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let build = scratch.path().join("workflow/inventory/group-0/build");
    let original = source(scratch.path());
    let image = artifact_path(&build, 0, &original).unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(image.parent().unwrap())
        .unwrap();
    let build_capture = BuildCapture {
        version: 1,
        resolution: resolution(),
        artifacts: vec![BuiltArtifact {
            cargo: artifact(&original, "captured-before-failure", "test"),
            executable: executable(&original, &image).unwrap(),
        }],
        cargo_exit: 101,
        incomplete: BTreeMap::new(),
    };
    let receipt = build.join("build.json");
    write_json(&receipt, &build_capture).unwrap();
    let groups = [(0, build_capture)];
    for _ in 0..2 {
        crate::managed::workflow::release_images(
            scratch.path(),
            crate::test_workflow::execute::Action::Test,
            None,
            None,
            &groups,
        )
        .unwrap();
        assert!(!image.exists());
        assert!(!executable_record(&image).unwrap().exists());
        assert!(receipt.is_file());
        assert!(original.is_file());
    }
}

#[test]
fn captured_application_requires_complete_original_root_source_and_features() {
    let directory = tempfile::tempdir().unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let mut capture = application_capture(directory.path());
    let require = |capture: &BuildCapture| {
        capture.require_binary(
            Path::new("/workspace/alpha"),
            "alpha",
            "tool",
            "tool.rs",
            &["feature-a"],
        )
    };
    require(&capture).unwrap();
    assert!(
        capture
            .require_binary(
                Path::new("/other/alpha"),
                "alpha",
                "tool",
                "tool.rs",
                &["feature-a"]
            )
            .is_err()
    );
    assert!(
        capture
            .require_binary(
                Path::new("/workspace/alpha"),
                "foreign",
                "tool",
                "tool.rs",
                &["feature-a"]
            )
            .is_err()
    );
    assert!(
        capture
            .require_binary(
                Path::new("/workspace/alpha"),
                "alpha",
                "tool",
                "tool.rs",
                &["feature-b"]
            )
            .is_err()
    );
    capture.resolution.units[0].root = false;
    assert!(require(&capture).is_err());
    capture.resolution.units[0].root = true;
    capture.artifacts[0].cargo.profile.test = true;
    assert!(require(&capture).is_err());
    capture.artifacts[0].cargo.profile.test = false;
    capture.artifacts.push(capture.artifacts[0].clone());
    assert!(require(&capture).is_err());
    capture.artifacts.pop();
    capture.cargo_exit = 1;
    assert!(require(&capture).is_err());
    capture.cargo_exit = 0;
    fs::remove_file(&capture.artifacts[0].executable.original).unwrap();
    require(&capture).unwrap(); // Capture provenance survives replacement of mutable Cargo output.
}

#[test]
fn captured_application_manifest_refuses_unprotected_linked_or_oversized_inputs() {
    let directory = tempfile::tempdir().unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let capture = application_capture(directory.path());
    let path = directory.path().join("build.json");
    write_json(&path, &capture).unwrap();
    BuildCapture::read(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(BuildCapture::read(&path).is_err());
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    let link = directory.path().join("linked.json");
    fs::hard_link(&path, &link).unwrap();
    assert!(BuildCapture::read(&path).is_err());
    fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(BuildCapture::read(&link).is_err());
    let large = directory.path().join("large.json");
    let file = File::create(&large).unwrap();
    file.set_len((64 << 20) + 1).unwrap();
    fs::set_permissions(&large, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(BuildCapture::read(&large).is_err());
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
fn retained_origin_is_published_before_use_and_survives_original_removal() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let source = source(root.path());
    let captured = executable(&source, &root.path().join("captured")).unwrap();
    assert_eq!(
        original_executable(&captured.captured).unwrap(),
        Some(source.clone())
    );
    assert!(
        super::original_executable(&captured.captured, &mut &b"different executing image"[..])
            .is_err()
    );
    fs::remove_file(&source).unwrap();
    assert_eq!(
        original_executable(&captured.captured).unwrap(),
        Some(source)
    );
    fs::set_permissions(&captured.captured, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(&captured.captured, b"replacement image").unwrap();
    assert!(original_executable(&captured.captured).is_err());
}
#[test]
fn captured_origin_refuses_corrupt_redirected_unprotected_and_symlink_records() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let source = source(root.path());
    let mut captured = executable(&source, &root.path().join("captured")).unwrap();
    let record = executable_record(&captured.captured).unwrap();
    fs::remove_file(&record).unwrap();
    assert_eq!(original_executable(&captured.captured).unwrap(), None);
    fs::write(&record, b"invalid capture").unwrap();
    fs::set_permissions(&record, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(original_executable(&captured.captured).is_err());
    fs::remove_file(&record).unwrap();
    captured.captured = root.path().join("different-image");
    write_json(&record, &captured).unwrap();
    assert!(original_executable(&root.path().join("captured")).is_err());
    fs::set_permissions(&record, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(original_executable(&root.path().join("captured")).is_err());
    fs::remove_file(&record).unwrap();
    std::os::unix::fs::symlink(&source, &record).unwrap();
    assert!(original_executable(&root.path().join("captured")).is_err());
}
fn original_executable(path: &Path) -> Result<Option<PathBuf>, CorrectnessError> {
    super::original_executable(path, &mut read_file(path)?)
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
