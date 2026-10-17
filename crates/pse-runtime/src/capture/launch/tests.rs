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

#[test]
fn launch_capture_real_control_protocol_enables_sampling_before_exec() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("scientific-first-instruction");
    let enabled = directory.path().join("sampling-enabled");
    let sampler = directory.path().join("sampler");
    // This executable substitutes only the external sampler. Actual FIFO creation, fd wiring,
    // enable command, acknowledgement, gated exec and both independent owners remain production.
    let script = format!("#!/bin/sh\nIFS= read -r command <&3 || exit 4\n[ \"$command\" = enable ] || exit 5\n[ ! -e '{}' ] || exit 6\nprintf enabled > '{}'\nprintf 'ack\\n' >&4\nwhile [ ! -e '{}' ]; do sleep .01; done\n", marker.display(), enabled.display(), marker.display());
    std::fs::write(&sampler, script).unwrap();
    std::fs::set_permissions(&sampler, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut request = request(directory.path(), &["/bin/sh", "-c", "test -e \"$1\" || exit 8; printf science > \"$2\"", "science"]);
    request.startup_timeout = Duration::from_secs(2);
    request.argv.extend([enabled.display().to_string(), marker.display().to_string()]);
    let sampler = sampler.display().to_string();
    let (mut command, capture) = launch_with(request, move |request, locator, stop, state, ready| {
        cpu::run_launched_program(request, locator, stop, state, ready, &sampler)
    }).unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    let document = capture.wait().unwrap();
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "science");
    // No samples were manufactured by this protocol-only control.
    assert_eq!(document.status, CaptureStatus::Failed);
    assert!(document.loss.sampling_loss_unknown);
    assert!(!directory.path().join("capture/perf-control.fifo").exists());
    assert!(!directory.path().join("capture/perf-ack.fifo").exists());
}
#[test]
fn launch_capture_actual_sampler_start_failure_preserves_logs_and_science() {
    let directory = tempfile::tempdir().unwrap();
    let mut request = request(directory.path(), &["/bin/sh", "-c", "exit 9"]);
    request.startup_timeout = Duration::from_secs(1);
    let (mut command, capture) = launch_with(request, |request, locator, stop, state, ready| {
        cpu::run_launched_program(request, locator, stop, state, ready, "/no/such/pse-sampler")
    }).unwrap();
    assert_eq!(command.wait().unwrap(), 9);
    let document = capture.wait().unwrap();
    assert_eq!(document.status, CaptureStatus::Failed);
    assert!(document.outputs.iter().any(|output| output.kind == super::super::CaptureOutputKind::ToolLog));
    assert!(!document.errors.is_empty());
}
