// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn request(root: &Path, argv: &[&str]) -> SourceRecordingLaunchRequest {
    let selection: SourceRecordingRequest = serde_json::from_value(serde_json::json!({
        "version": 2,
        "detail": {"version":1,"families":["root_evaluation.v1"],"requirement":"complete",
            "sample_every":1,"boundary_thread_cpu":false,"managed_budget":null,
            "expiry_monotonic_ns":null,"duration":{"secs":1,"nanos":0},
            "output":root.join("source"),"budget":{"memory_bytes":4194304,"output_bytes":1048576}},
        "acknowledgement":root.join("ack.json")
    })).unwrap();
    let request_file = root.join("request.json");
    std::fs::write(&request_file, serde_json::to_vec(&selection).unwrap()).unwrap();
    SourceRecordingLaunchRequest {
        version: pse_model::document::Version, request_file,
        argv: argv.iter().map(|value| (*value).into()).collect(), cwd: None,
        acknowledgement_timeout: Duration::from_millis(30),
        command_timeout: Duration::from_secs(5), output: root.join("launch"), output_bytes: 1024,
    }
}

#[test]
fn literal_command_failure_and_prefix_exhaustion_do_not_invent_source_coverage() {
    let root = tempfile::tempdir().unwrap();
    let request = request(root.path(), &["/bin/sh","-c","printf '%s' \"$1\"; head -c 10000 /dev/zero; exit 7","science","$(touch never)"]);
    let result = record_launch(request, &AtomicBool::new(false)).unwrap();
    assert_eq!(result.command_status, Some(7));
    assert!(result.drained && result.logs_truncated);
    assert!(result.acknowledgement.is_none());
    assert_eq!(result.stdout_prefix_bytes, 512);
    assert!(std::fs::read(root.path().join("launch/stdout.txt")).unwrap().starts_with(b"$(touch never)"));
    assert!(!root.path().join("ack.json.launch").exists());
}

#[test]
fn command_timeout_drains_the_group_and_preserves_signal_status() {
    let root = tempfile::tempdir().unwrap();
    let mut request = request(root.path(), &["/bin/sh","-c","sleep 60 & wait"]);
    request.command_timeout = Duration::from_millis(50);
    let result = record_launch(request, &AtomicBool::new(false)).unwrap();
    assert!(result.timed_out && result.drained);
    assert_eq!(result.command_status, Some(-9));
}

#[test]
fn cancelled_before_spawn_has_no_command_side_effects_and_releases_claim() {
    let root = tempfile::tempdir().unwrap();
    let request = request(root.path(), &["/bin/sh","-c","touch never"]);
    assert!(record_launch(request, &AtomicBool::new(true)).is_err());
    assert!(!root.path().join("ack.json.launch").exists());
    assert!(!root.path().join("launch").exists());
}

#[test]
fn claim_drop_preserves_a_replacement_and_cleans_only_its_exact_file() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ack.json");
    let claim = LaunchClaim::acquire(&path).unwrap();
    std::fs::rename(root.path().join("ack.json.launch"), root.path().join("original")).unwrap();
    std::fs::write(root.path().join("ack.json.launch"), b"another owner").unwrap();
    drop(claim);
    assert_eq!(std::fs::read(root.path().join("ack.json.launch")).unwrap(), b"another owner");
}

#[test]
fn log_open_failure_reports_zero_retained_bytes_but_continues_pipe_drain() {
    let root = tempfile::tempdir().unwrap();
    let (reader, mut writer) = std::os::unix::net::UnixStream::pair().unwrap();
    let existing = root.path().join("stdout.txt");
    std::fs::write(&existing,b"existing").unwrap();
    let drain = LogDrain::start(reader, existing, 1024).unwrap();
    writer.write_all(&[b'x';65536]).unwrap();
    drop(writer);
    while !drain.finished() { std::thread::sleep(Duration::from_millis(1)); }
    let result = drain.finish();
    assert_eq!(result.bytes,0);
    assert!(result.truncated && !result.errors.is_empty());
}

#[test]
fn log_path_replacement_cannot_advertise_the_original_files_prefix() {
    let root = tempfile::tempdir().unwrap();
    let (reader, mut writer) = std::os::unix::net::UnixStream::pair().unwrap();
    let path = root.path().join("stdout.txt");
    let drain = LogDrain::start(reader, path.clone(), 1024).unwrap();
    writer.write_all(b"original prefix").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !std::fs::metadata(&path).is_ok_and(|metadata| metadata.len() > 0) {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    std::fs::rename(&path, root.path().join("original")).unwrap();
    std::fs::write(&path, b"replacement file owned elsewhere").unwrap();
    drop(writer);
    while !drain.finished() { std::thread::sleep(Duration::from_millis(1)); }
    let result = drain.finish();
    assert_eq!(result.bytes, 0);
    assert!(result.truncated && result.errors.iter().any(|error| error.contains("identity changed")));
    assert_eq!(std::fs::read(path).unwrap(), b"replacement file owned elsewhere");
}
