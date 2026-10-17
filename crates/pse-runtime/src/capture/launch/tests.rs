// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn request(directory: &std::path::Path, arguments: &[&str]) -> LaunchCaptureRequest {
    LaunchCaptureRequest { version: Version, argv: arguments.iter().map(|s| (*s).into()).collect(),
        cwd: None, output: directory.join("capture"), duration: Duration::from_millis(100),
        startup_timeout: Duration::from_millis(100), budget: CaptureBudget::default() }
}
#[test]
fn launch_capture_gate_precedes_first_command_and_has_no_invented_operation() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("first-instruction");
    let observed = marker.clone();
    let mut request = request(directory.path(), &["/bin/sh", "-c", "printf started > \"$1\"", "science"]);
    request.argv.push(marker.display().to_string());
    let (mut command, capture) = launch_with(request, move |_, locator, _, _, ready| {
        assert!(locator.is_current());
        assert!(!observed.exists(), "scientific exec must remain gated before sampling acknowledgement");
        ready.send(Ok(())).unwrap();
        Ok(())
    }).unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "started");
    let document = capture.wait().unwrap();
    assert!(document.operation.is_none());
    assert_eq!(document.launch_process.unwrap().pid, command.locator().pid);
}
#[test]
fn launch_capture_startup_failure_releases_science_with_failed_capture() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(request(directory.path(), &["/bin/sh", "-c", "exit 7"]),
        |_, _, _, _, _| Err(CaptureError::Invalid("selected sampler unavailable".into()))).unwrap();
    assert_eq!(command.wait().unwrap(), 7);
    let document = capture.wait().unwrap();
    assert_eq!(document.status, CaptureStatus::Failed);
    assert!(document.errors.iter().any(|error| error.contains("sampler unavailable")));
}
#[test]
fn launch_capture_startup_timeout_releases_science_and_joins_only_capture() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(request(directory.path(), &["/bin/sh", "-c", "exit 0"]),
        |_, _, stop, _, _| {
            while !stop.load(Ordering::Acquire) { std::thread::sleep(Duration::from_millis(1)); }
            Ok(())
        }).unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    assert_eq!(capture.wait().unwrap().status, CaptureStatus::Failed);
}
#[test]
fn launch_capture_stop_and_drop_cannot_cancel_independently_owned_command() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(request(directory.path(), &["/bin/sh", "-c", "sleep 60 & wait"]),
        |_, _, stop, _, ready| {
            ready.send(Ok(())).unwrap();
            while !stop.load(Ordering::Acquire) { std::thread::sleep(Duration::from_millis(1)); }
            Ok(())
        }).unwrap();
    let _ = capture.stop().unwrap();
    assert_eq!(command.try_wait().unwrap(), None, "capture stop must leave science running");
    assert_eq!(command.cancel().unwrap(), 137);
}
#[test]
fn launch_capture_cancel_drains_owned_group_and_retains_capture() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(request(directory.path(), &["/bin/sh", "-c", "sleep 60 & wait"]),
        |_, _, _, _, ready| { ready.send(Ok(())).unwrap(); Ok(()) }).unwrap();
    assert_eq!(command.try_wait().unwrap(), None);
    assert_eq!(command.cancel().unwrap(), 137);
    assert!(command.try_wait().unwrap().is_some());
    assert!(capture.wait().is_ok());
}
#[test]
fn launch_capture_rejects_invalid_or_existing_output_before_command_spawn() {
    let directory = tempfile::tempdir().unwrap();
    let mut invalid = request(directory.path(), &[]);
    assert!(launch_capture(invalid.clone()).is_err());
    assert!(!invalid.output.exists());
    invalid.argv = vec!["/bin/true".into()];
    std::fs::create_dir(&invalid.output).unwrap();
    assert!(launch_capture(invalid).is_err());
}
#[test]
fn launch_capture_direct_arguments_preserve_spaces_metacharacters_and_cwd() {
    let directory = tempfile::tempdir().unwrap();
    let mut request = request(directory.path(), &["/bin/sh", "-c", "printf '%s' \"$1\" > literal", "science", "$(touch WRONG); a b"]);
    request.cwd = Some(directory.path().to_owned());
    let (mut command, capture) = launch_with(request,
        |_, _, _, _, ready| { ready.send(Ok(())).unwrap(); Ok(()) }).unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    capture.wait().unwrap();
    assert_eq!(std::fs::read_to_string(directory.path().join("literal")).unwrap(), "$(touch WRONG); a b");
    assert!(!directory.path().join("WRONG").exists());
}
