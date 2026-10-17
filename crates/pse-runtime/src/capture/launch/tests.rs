// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn request(directory: &std::path::Path, arguments: &[&str]) -> LaunchCaptureRequest {
    LaunchCaptureRequest {
        version: Version,
        argv: arguments.iter().map(|s| (*s).into()).collect(),
        cwd: None,
        output: directory.join("capture"),
        duration: Duration::from_millis(100),
        startup_timeout: Duration::from_millis(100),
        budget: CaptureBudget::default(),
    }
}
#[test]
fn launch_capture_gate_precedes_first_command_and_has_no_invented_operation() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("first-instruction");
    let observed = marker.clone();
    let mut request = request(
        directory.path(),
        &["/bin/sh", "-c", "printf started > \"$1\"", "science"],
    );
    request.argv.push(marker.display().to_string());
    let (mut command, capture) = launch_with(request, move |_, locator, _, _, ready| {
        assert!(locator.is_current());
        assert!(
            !observed.exists(),
            "scientific exec must remain gated before sampling acknowledgement"
        );
        ready.send(Ok(())).unwrap();
        Ok(())
    })
    .unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "started");
    let document = capture.wait().unwrap();
    assert!(document.operation.is_none());
    assert_eq!(document.launch_process.unwrap().pid, command.locator().pid);
}
#[test]
fn launch_capture_startup_failure_releases_science_with_failed_capture() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(
        request(directory.path(), &["/bin/sh", "-c", "exit 7"]),
        |_, _, _, _, _| Err(CaptureError::Invalid("selected sampler unavailable".into())),
    )
    .unwrap();
    assert_eq!(command.wait().unwrap(), 7);
    let document = capture.wait().unwrap();
    assert_eq!(document.status, CaptureStatus::Failed);
    assert!(
        document
            .errors
            .iter()
            .any(|error| error.contains("sampler unavailable"))
    );
}
#[test]
fn launch_capture_startup_timeout_releases_science_and_joins_only_capture() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(
        request(directory.path(), &["/bin/sh", "-c", "exit 0"]),
        |_, _, stop, _, _| {
            while !stop.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    assert_eq!(capture.wait().unwrap().status, CaptureStatus::Failed);
}
#[test]
fn launch_capture_stop_and_drop_cannot_cancel_independently_owned_command() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(
        request(directory.path(), &["/bin/sh", "-c", "sleep 60 & wait"]),
        |_, _, stop, _, ready| {
            ready.send(Ok(())).unwrap();
            while !stop.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(())
        },
    )
    .unwrap();
    let _ = capture.stop().unwrap();
    assert_eq!(
        command.try_wait().unwrap(),
        None,
        "capture stop must leave science running"
    );
    assert_eq!(command.cancel().unwrap(), 137);
}
#[test]
fn launch_capture_cancel_drains_owned_group_and_retains_capture() {
    let directory = tempfile::tempdir().unwrap();
    let (mut command, capture) = launch_with(
        request(directory.path(), &["/bin/sh", "-c", "sleep 60 & wait"]),
        |_, _, _, _, ready| {
            ready.send(Ok(())).unwrap();
            Ok(())
        },
    )
    .unwrap();
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
    let mut request = request(
        directory.path(),
        &[
            "/bin/sh",
            "-c",
            "printf '%s' \"$1\" > literal",
            "science",
            "$(touch WRONG); a b",
        ],
    );
    request.cwd = Some(directory.path().to_owned());
    let (mut command, capture) = launch_with(request, |_, _, _, _, ready| {
        ready.send(Ok(())).unwrap();
        Ok(())
    })
    .unwrap();
    assert_eq!(command.wait().unwrap(), 0);
    capture.wait().unwrap();
    assert_eq!(
        std::fs::read_to_string(directory.path().join("literal")).unwrap(),
        "$(touch WRONG); a b"
    );
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
    let script = format!(
        "#!/bin/sh\nIFS= read -r command <&3 || exit 4\n[ \"$command\" = enable ] || exit 5\n[ ! -e '{}' ] || exit 6\nprintf enabled > '{}'\nprintf 'ack\\n' >&4\nwhile [ ! -e '{}' ]; do sleep .01; done\n",
        marker.display(),
        enabled.display(),
        marker.display()
    );
    std::fs::write(&sampler, script).unwrap();
    std::fs::set_permissions(&sampler, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut request = request(
        directory.path(),
        &[
            "/bin/sh",
            "-c",
            "test -e \"$1\" || exit 8; printf science > \"$2\"",
            "science",
        ],
    );
    request.startup_timeout = Duration::from_secs(2);
    request
        .argv
        .extend([enabled.display().to_string(), marker.display().to_string()]);
    let sampler = sampler.display().to_string();
    let (mut command, capture) =
        launch_with(request, move |request, locator, stop, state, ready| {
            cpu::run_launched_program(request, locator, stop, state, ready, &sampler)
        })
        .unwrap();
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
    })
    .unwrap();
    assert_eq!(command.wait().unwrap(), 9);
    let document = capture.wait().unwrap();
    assert_eq!(document.status, CaptureStatus::Failed);
    assert!(
        document
            .outputs
            .iter()
            .any(|output| output.kind == super::super::CaptureOutputKind::ToolLog)
    );
    assert!(!document.errors.is_empty());
}

#[test]
fn launch_capture_late_operation_links_require_exact_published_process_identity() {
    let directory = tempfile::tempdir().unwrap();
    let owner = crate::observation::Diagnostics::new(
        pse_diagnostics::observation::Policy::default(),
        directory.path().to_owned(),
    );
    let operation = owner.hub().start("late published scientific operation");
    owner.publish().unwrap();
    let image = crate::observation::read_checkpoint(&owner.checkpoint_path()).unwrap();
    let state = Arc::new(Mutex::new(CaptureDocument {
        version: Version,
        kind: CaptureKind::Cpu,
        operation: None,
        launch_process: Some(image.locator.clone()),
        related_operations: Vec::new(),
        status: CaptureStatus::Running,
        requested_duration: Duration::from_secs(1),
        observed_duration: Duration::ZERO,
        targets: Vec::new(),
        outputs: Vec::new(),
        loss: CaptureLoss::default(),
        symbols: CaptureSymbols::default(),
        open_intervals: 0,
        errors: Vec::new(),
    }));
    let mut wrong = image.locator.clone();
    wrong.start_ticks = wrong.start_ticks.map(|ticks| ticks + 1);
    let mut checkpoint = None;
    let mut cursor = None;
    cpu::discover_launch(
        directory.path(),
        &wrong,
        &mut checkpoint,
        &mut cursor,
        &state,
    );
    assert!(checkpoint.is_none());
    assert!(document(&state).related_operations.is_empty());
    cpu::discover_launch(
        directory.path(),
        &image.locator,
        &mut checkpoint,
        &mut cursor,
        &state,
    );
    assert_eq!(checkpoint, Some(owner.checkpoint_path()));
    assert_eq!(
        document(&state).related_operations,
        vec![operation.context().id()]
    );
    assert!(
        document(&state).operation.is_none(),
        "observed links never invent an operation selection"
    );
    // A retained locator mismatch is a diagnostic error, not a switch to the reused PID.
    cpu::discover_launch(
        directory.path(),
        &wrong,
        &mut checkpoint,
        &mut cursor,
        &state,
    );
    assert!(
        document(&state)
            .errors
            .iter()
            .any(|error| error.contains("identity"))
    );
}

#[test]
fn launch_capture_member_identity_handles_comm_delimiters_and_refuses_missing_start() {
    let stat = format!(
        "123 (worker ) name) S 1 456 {} 987 0",
        vec!["0"; 16].join(" ")
    );
    assert_eq!(member_identity(&stat).unwrap(), (true, 456, 987));
    assert_eq!(
        member_identity(&stat.replace(") S ", ") Z ")).unwrap(),
        (false, 456, 987)
    );
    assert!(member_identity("123 (worker) S 1 456").is_err());
}

#[test]
fn launch_capture_owned_group_waits_exact_member_notifications_without_stopping_foreign_child() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("descendant-started");
    let mut request = request(
        directory.path(),
        &[
            "/bin/sh",
            "-c",
            "sleep 60 & printf '%s' \"$!\" > \"$1\"; wait",
            "science",
        ],
    );
    request.argv.push(marker.display().to_string());
    let (mut command, capture) = launch_with(request, |_, _, _, _, ready| {
        ready.send(Ok(())).unwrap();
        Ok(())
    })
    .unwrap();
    let started = Instant::now();
    while !marker.exists() && started.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(marker.exists());
    let members = group_members(command.locator().pid).unwrap();
    assert!(!members.is_empty());
    let mut foreign = Command::new("/bin/sleep").arg("60").spawn().unwrap();
    let result = command.cancel();
    let foreign_survives = foreign.try_wait().unwrap().is_none();
    let _ = foreign.kill();
    foreign.wait().unwrap();
    assert_eq!(result.unwrap(), 137);
    assert!(
        foreign_survives,
        "command drain cannot signal a child outside the owned group"
    );
    for member in &members {
        let mut descriptors = [rustix::event::PollFd::new(
            member,
            rustix::event::PollFlags::IN,
        )];
        let zero = rustix::event::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        assert!(rustix::event::poll(&mut descriptors, Some(&zero)).unwrap() > 0);
    }
    capture.wait().unwrap();
}

#[test]
fn launch_capture_drains_live_thread_group_after_its_main_thread_exits() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("threaded-descendant.c");
    let executable = directory.path().join("threaded-descendant");
    let marker = directory.path().join("main-thread-exited");
    // The worker writes readiness only after /proc reports its TGID leader as Z.
    // Its still-running thread keeps the process pidfd unready: state Z alone is
    // therefore not sufficient authority to exclude this owned descendant.
    std::fs::write(&source, r#"
#include <pthread.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
static void *live_worker(void *marker) {
    struct timespec delay = {0, 1000000};
    for (int attempt = 0; attempt < 2000; ++attempt) {
        char text[2048];
        FILE *stat = fopen("/proc/self/stat", "r");
        if (!stat) _exit(2);
        char *line = fgets(text, sizeof text, stat);
        fclose(stat);
        char *end = line ? strrchr(text, ')') : NULL;
        if (end && end[1] == ' ' && end[2] == 'Z') {
            FILE *ready = fopen((const char *)marker, "w");
            if (!ready) _exit(3);
            fprintf(ready, "%ld", (long)getpid());
            fclose(ready);
            for (;;) pause();
        }
        nanosleep(&delay, NULL);
    }
    _exit(4);
}
int main(int argc, char **argv) {
    if (argc != 2) return 5;
    pthread_t worker;
    if (pthread_create(&worker, NULL, live_worker, argv[1])) return 6;
    pthread_exit(NULL);
}
"#).unwrap();
    // A tiny Linux pthread fixture uses the workstation's existing C compiler;
    // missing compiler support is a test prerequisite failure, never a skip.
    let compiled = Command::new("/usr/bin/cc").arg("-O0").arg("-pthread")
        .arg(&source).arg("-o").arg(&executable).output().unwrap();
    assert!(compiled.status.success(), "C fixture compilation failed: {}",
        String::from_utf8_lossy(&compiled.stderr));
    let mut request = request(directory.path(), &["/bin/sh", "-c", "\"$1\" \"$2\" & wait", "science"]);
    request.argv.extend([executable.display().to_string(), marker.display().to_string()]);
    let (mut command, capture) = launch_with(request,
        |_, _, _, _, ready| { ready.send(Ok(())).unwrap(); Ok(()) }).unwrap();
    let started = Instant::now();
    let descendant = loop {
        if let Some(pid) = std::fs::read_to_string(&marker).ok().and_then(|text| text.parse::<u32>().ok()) { break pid; }
        assert!(started.elapsed() < Duration::from_secs(3), "threaded descendant did not establish readiness");
        std::thread::sleep(Duration::from_millis(1));
    };
    let stat = std::fs::read_to_string(format!("/proc/{descendant}/stat")).unwrap();
    assert_eq!(stat.rsplit_once(')').unwrap().1.split_whitespace().next(), Some("Z"));
    let members = group_members(command.locator().pid).unwrap();
    assert_eq!(members.len(), 1, "a Z TGID leader with a live thread must remain in owned inventory");
    let zero = rustix::event::Timespec { tv_sec: 0, tv_nsec: 0 };
    let mut live = [rustix::event::PollFd::new(&members[0], rustix::event::PollFlags::IN)];
    assert_eq!(rustix::event::poll(&mut live, Some(&zero)).unwrap(), 0);
    let mut foreign = Command::new("/bin/sleep").arg("60").spawn().unwrap();
    let cancelled = command.cancel();
    let foreign_status = foreign.try_wait();
    let _ = foreign.kill();
    foreign.wait().unwrap();
    assert_eq!(cancelled.unwrap(), 137);
    assert!(foreign_status.unwrap().is_none(), "thread-group drain must preserve the unrelated child");
    let mut drained = [rustix::event::PollFd::new(&members[0], rustix::event::PollFlags::IN)];
    assert!(rustix::event::poll(&mut drained, Some(&zero)).unwrap() > 0,
        "the complete descendant thread group must exit before command ownership settles");
    capture.wait().unwrap();
}
