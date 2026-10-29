// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Targeted actual-user-manager foreground protocol journeys.
use std::{
    ffi::OsString,
    fs,
    io::{Read, Write},
    os::unix::ffi::OsStringExt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct Guard {
    child: std::process::Child,
    pid: u32,
    group: Option<String>,
    prior: String,
    _handoff: tempfile::TempDir,
}
impl Guard {
    fn top_level(command: &mut Command) -> Self {
        let handoff = tempfile::tempdir().unwrap();
        let pidfile = handoff.path().join("controller");
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let prior = format!(
            "pse-foreground-harness-{}-{nonce:x}.scope",
            std::process::id()
        );
        // The real manager creates a private application scope. Its shell stays
        // there while the native host relocates, so the prior lifetime survives.
        let mut wrapper = Command::new("/usr/bin/systemd-run");
        wrapper.args(["--user", "--scope", "--quiet", "--unit", &prior, "--", "/bin/sh", "-c", "pidfile=$1; shift; \"$@\" & controller=$!; printf '%s\\n' \"$controller\" > \"$pidfile\"; wait \"$controller\"; exit $?", "fixture"])
            .arg(&pidfile).arg(command.get_program()).args(command.get_args()).stdout(Stdio::null()).stderr(Stdio::piped());
        if let Some(cwd) = command.get_current_dir() {
            wrapper.current_dir(cwd);
        }
        for (key, value) in command.get_envs() {
            if let Some(value) = value {
                wrapper.env(key, value);
            } else {
                wrapper.env_remove(key);
            }
        }
        let child = wrapper.spawn().unwrap();
        wait_file(&pidfile);
        let pid = fs::read_to_string(pidfile).unwrap().trim().parse().unwrap();
        Self {
            child,
            pid,
            group: None,
            prior,
            _handoff: handoff,
        }
    }
    fn id(&self) -> u32 {
        self.pid
    }
    fn kill(&mut self) -> std::io::Result<()> {
        rustix::process::kill_process(
            rustix::process::Pid::from_raw(self.pid as i32).unwrap(),
            rustix::process::Signal::KILL,
        )
        .map_err(std::io::Error::from)
    }
}
impl std::ops::Deref for Guard {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl std::ops::DerefMut for Guard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.kill();
        if let Some(group) = &self.group {
            if !empty(group) {
                if let Some(name) = Path::new(group).file_name() {
                    let _ = Command::new("/usr/bin/systemctl")
                        .args(["--user", "kill", "--signal=KILL"])
                        .arg(name)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            }
        }
        let _ = Command::new("/usr/bin/systemctl")
            .args(["--user", "stop", "--no-block", &self.prior])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn host() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pse-host"))
}
fn record_fresh_group(child: &mut Guard, group: &str) {
    let current = fs::read_to_string("/proc/self/cgroup").unwrap();
    assert_ne!(
        current.trim().strip_prefix("0::").unwrap(),
        group,
        "test cannot acquire cleanup authority over enclosing native containment"
    );
    child.group = Some(group.to_owned());
}
fn wait_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !fs::read(path).is_ok_and(|bytes| bytes.ends_with(b"\n")) {
        assert!(
            Instant::now() < deadline,
            "workload readiness deadline: {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(5));
    }
}
fn empty(group: &str) -> bool {
    match fs::read_to_string(
        Path::new("/sys/fs/cgroup")
            .join(group.trim_start_matches('/'))
            .join("cgroup.events"),
    ) {
        Ok(events) => events.lines().any(|line| line == "populated 0"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => panic!("cgroup read failed: {error}"),
    }
}
fn wait_empty(group: &str) {
    let deadline = Instant::now() + Duration::from_secs(40);
    while !empty(group) {
        assert!(
            Instant::now() < deadline,
            "native scope still populated: {group}"
        );
        thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn foreground_preserves_raw_payload_pipes_cwd_and_status() {
    let temp = tempfile::tempdir().unwrap();
    let cwd = temp.path().join(OsString::from_vec(b"cwd-\xff".to_vec()));
    fs::create_dir(&cwd).unwrap();
    let mut child = host().args(["run", "/bin/sh", "-c", "printf '%s' \"$1\"; printf '%s' \"$RAW\" >&2; cat; test \"$PWD\" = \"$EXPECTED\" || exit 91; exit 7", "fixture"])
        .arg(OsString::from_vec(b"arg-\xfe".to_vec())).env("RAW", OsString::from_vec(b"env-\xfd".to_vec()))
        .env("EXPECTED", &cwd).current_dir(&cwd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"stdin\0payload")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(
        result.status.code(),
        Some(7),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"arg-\xfestdin\0payload");
    assert_eq!(result.stderr, b"env-\xfd");
}
#[test]
fn foreground_nested_execution_preserves_enclosing_membership() {
    let result = host()
        .args([
            "run",
            "/bin/sh",
            "-c",
            "cat /proc/self/cgroup; exec \"$1\" run /bin/cat /proc/self/cgroup",
            "fixture",
            env!("CARGO_BIN_EXE_pse-host"),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8(result.stdout).unwrap();
    let rows: Vec<_> = text.lines().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], rows[1]);
    assert!(rows[0].contains("pse-host-") && rows[0].ends_with(".scope"));
}
#[test]
fn foreground_controller_sigkill_drains_escaped_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let ready = temp.path().join("ready");
    let mut child = Guard::top_level(host().args(["run", "/bin/sh", "-c", "setsid /bin/sh -c 'trap \"\" TERM; cat /proc/self/cgroup > \"$1\"; while :; do sleep 1; done' fixture \"$1\" & wait", "fixture"])
        .arg(&ready));
    wait_file(&ready);
    let group = fs::read_to_string(&ready)
        .unwrap()
        .trim()
        .strip_prefix("0::")
        .unwrap()
        .to_owned();
    record_fresh_group(&mut child, &group);
    let owner = fs::read_to_string(format!("/proc/{}/cgroup", child.id())).unwrap();
    let owner = owner.trim().strip_prefix("0::").unwrap();
    assert_ne!(group, owner, "controller must not populate workload scope");
    child.kill().unwrap();
    child.wait().unwrap();
    wait_empty(owner);
    wait_empty(&group);
}
#[test]
fn foreground_startup_caller_death_cannot_release_command() {
    let temp = tempfile::tempdir().unwrap();
    let ran = temp.path().join("must-not-run");
    let mut child = host()
        .args(["run", "/usr/bin/touch"])
        .arg(&ran)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(!ran.exists());
}
#[test]
fn foreground_sigterm_settles_and_reports_signal_status() {
    let temp = tempfile::tempdir().unwrap();
    let ready = temp.path().join("ready");
    let mut child = Guard::top_level(
        host()
            .args([
                "run",
                "/bin/sh",
                "-c",
                "cat /proc/self/cgroup > \"$1\"; exec sleep 100",
                "fixture",
            ])
            .arg(&ready),
    );
    wait_file(&ready);
    let group = fs::read_to_string(&ready)
        .unwrap()
        .trim()
        .strip_prefix("0::")
        .unwrap()
        .to_owned();
    record_fresh_group(&mut child, &group);
    rustix::process::kill_process(
        rustix::process::Pid::from_raw(child.id() as i32).unwrap(),
        rustix::process::Signal::TERM,
    )
    .unwrap();
    let result = child.wait().unwrap();
    let mut errors = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut errors)
        .unwrap();
    assert_eq!(result.code(), Some(143), "{errors}");
    wait_empty(&group);
}

#[test]
fn foreground_exec_failures_retain_shell_statuses() {
    let missing = host()
        .args(["run", "/definitely/missing/pse-command"])
        .status()
        .unwrap();
    assert_eq!(missing.code(), Some(127));
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("not-executable");
    fs::write(&executable, b"test").unwrap();
    assert_eq!(
        host().arg("run").arg(executable).status().unwrap().code(),
        Some(126)
    );
    let nested = host()
        .args([
            "run",
            env!("CARGO_BIN_EXE_pse-host"),
            "run",
            "/definitely/missing/pse-command",
        ])
        .status()
        .unwrap();
    assert_eq!(nested.code(), Some(127));
}
#[test]
fn foreground_preserves_controlling_terminal_without_a_proxy() {
    let result = Command::new("/usr/bin/script").args(["-qec", "exec \"$PSE_HOST_TEST_BINARY\" run /bin/sh -c 'test -t 0 && test -t 1 && test -t 2 && stty -a >/dev/null'", "/dev/null"])
        .env("PSE_HOST_TEST_BINARY", env!("CARGO_BIN_EXE_pse-host")).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
}

#[test]
fn foreground_gate_eof_never_executes_and_release_is_explicit() {
    use command_fds::{CommandFdExt, FdMapping};
    let temp = tempfile::tempdir().unwrap();
    for release in [false, true] {
        let sentinel = temp
            .path()
            .join(if release { "released" } else { "closed" });
        let (reader, mut writer) = std::io::pipe().unwrap();
        let mut command = host();
        command
            .args(["__foreground-exec", "/usr/bin/touch"])
            .arg(&sentinel);
        command
            .fd_mappings(vec![FdMapping {
                parent_fd: reader.into(),
                child_fd: 3,
            }])
            .unwrap();
        let mut child = command.spawn().unwrap();
        drop(command);
        if release {
            writer.write_all(&[1]).unwrap();
        }
        drop(writer);
        assert_eq!(
            child.wait().unwrap().code(),
            Some(if release { 0 } else { 125 })
        );
        assert_eq!(sentinel.exists(), release);
    }
}

#[test]
fn foreground_witnessed_gate_survives_controller_sigkill_without_exec() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("executed");
    let mut controller = Guard::top_level(
        host()
            .args([
                "run",
                "/bin/sh",
                "-c",
                "printf executed > \"$1\"; exec sleep 100",
                "fixture",
            ])
            .arg(&marker),
    );
    let pid = rustix::process::Pid::from_raw(controller.id() as i32).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let gate_pid = loop {
        let children = fs::read_to_string(format!("/proc/{0}/task/{0}/children", controller.id()))
            .unwrap_or_default();
        if let Some(child) = children
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<u32>().ok())
        {
            break child;
        }
        assert!(
            Instant::now() < deadline,
            "controller never spawned its gated child"
        );
        thread::yield_now();
    };
    rustix::process::kill_process(pid, rustix::process::Signal::STOP).unwrap();
    loop {
        let command = fs::read(format!("/proc/{gate_pid}/cmdline")).unwrap();
        if command
            .split(|byte| *byte == 0)
            .any(|part| part == b"__foreground-exec")
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "must witness the actual blocked exec gate"
        );
        thread::yield_now();
    }
    assert!(!marker.exists());
    controller.kill().unwrap();
    controller.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = fs::read_to_string(format!("/proc/{gate_pid}/stat"));
        if state
            .as_ref()
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
            || state.as_ref().is_ok_and(|text| {
                text.rsplit_once(") ")
                    .is_some_and(|(_, tail)| tail.starts_with('Z'))
            })
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "original gated process survived controller death"
        );
        thread::sleep(Duration::from_millis(5));
    }
    assert!(!marker.exists());
}

#[test]
fn foreground_prior_application_lifetime_stops_both_native_scopes() {
    struct PriorScope(String);
    impl Drop for PriorScope {
        fn drop(&mut self) {
            let _ = Command::new("/usr/bin/systemctl")
                .args(["--user", "stop", "--no-block", &self.0])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let ready = temp.path().join("ready");
    let controller_pid = temp.path().join("controller");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let prior = PriorScope(format!(
        "pse-foreground-parent-{}-{nonce:x}.scope",
        std::process::id()
    ));
    let mut wrapper = Command::new("/usr/bin/systemd-run").args(["--user", "--scope", "--quiet", "--unit", &prior.0, "--", "/bin/sh", "-c", "sleep 100 & printf '%s\\n' $$ > \"$3\"; exec \"$1\" run /bin/sh -c 'cat /proc/self/cgroup > \"$1\"; exec sleep 100' fixture \"$2\"", "fixture", env!("CARGO_BIN_EXE_pse-host")])
        .arg(&ready).arg(&controller_pid).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().unwrap();
    wait_file(&ready);
    wait_file(&controller_pid);
    let group = fs::read_to_string(&ready)
        .unwrap()
        .trim()
        .strip_prefix("0::")
        .unwrap()
        .to_owned();
    let pid = fs::read_to_string(&controller_pid).unwrap();
    let owner = fs::read_to_string(format!("/proc/{}/cgroup", pid.trim())).unwrap();
    let owner = owner.trim().strip_prefix("0::").unwrap();
    assert!(group.contains("pse-host-") && owner.contains("pse-host-"));
    assert!(
        Command::new("/usr/bin/systemctl")
            .args(["--user", "stop", "--no-block", &prior.0])
            .status()
            .unwrap()
            .success()
    );
    wait_empty(owner);
    wait_empty(&group);
    assert!(
        !wrapper.wait().unwrap().success(),
        "prior lifetime stop must terminate its dependent controller"
    );
}
