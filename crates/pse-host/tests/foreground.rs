// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Targeted actual-user-manager foreground protocol journeys.
#![cfg(test)]
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
        if let Some(group) = &self.group
            && !empty(group)
            && let Some(name) = Path::new(group).file_name()
        {
            let _ = Command::new("/usr/bin/systemctl")
                .args(["--user", "kill", "--signal=KILL"])
                .arg(name)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
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

// This helper is entered in a separate test process under the real native
// boundary. Its dispatch flag supplies data only: start_service_bound verifies
// actual manager/kernel containment before granting service launch authority.
#[test]
#[allow(
    clippy::exit,
    reason = "The separate owned-process fixture must preserve a specific subprocess exit status"
)]
fn ordinary_bound_service_fixture() {
    let Some(directory) = std::env::var_os("PSE_FOREST_FIXTURE_DIRECTORY") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let manager = pse_host::HostManager::connect_user(None).await.unwrap();
        let current = manager.current_unit().await.unwrap();
        if std::env::var_os("PSE_FOREST_FIXTURE_LITERAL").is_some() {
            literal_bound_service_fixture(&manager, &current, &directory).await;
            std::process::exit(7);
        }
        let intent = pse_host::LaunchIntent::fresh().unwrap();
        let spec = pse_host::ServiceSpec {
            intent,
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), r#"
/usr/bin/setsid /bin/sh -c '
trap '\''printf "term\n" > "$PSE_FOREST_TERM"; /bin/sleep 2; printf "drained\n" > "$PSE_FOREST_DRAINED"; exit 0'\'' TERM
group=$(/bin/sed -n "s/^0:://p" /proc/self/cgroup)
printf "%s\n%s\nready\n" "$group" "$$" > "$PSE_FOREST_DESCENDANT"
while :; do /bin/sleep 100; done
' &
wait
"#.into()],
            environment: vec![
                format!("PSE_FOREST_DESCENDANT={}", directory.join("descendant").display()),
                format!("PSE_FOREST_TERM={}", directory.join("term").display()),
                format!("PSE_FOREST_DRAINED={}", directory.join("drained").display()),
            ],
        };
        let service = manager.start_service_bound(&spec, &manager.borrow(&current.unit), &pse_host::ResourceControls::default(), Instant::now() + Duration::from_secs(15)).await.unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if fs::read_to_string(directory.join("descendant")).is_ok_and(|text| text.ends_with("ready\n")) { break; }
            assert!(Instant::now() < deadline, "bound service descendant did not become ready");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let borrowed = service.borrow_service(deadline).await.unwrap();
        let observed = borrowed.observe(deadline).await.unwrap();
        assert_eq!(observed.invocation, service.observe().await.unwrap().invocation);
        fs::write(directory.join("ready-pending"), format!("{}\n{}\n{}\n", current.control_group, observed.control_group, service.intent().unit())).unwrap();
        fs::rename(directory.join("ready-pending"), directory.join("ready")).unwrap();
        while !directory.join("release").exists() { tokio::time::sleep(Duration::from_millis(5)).await; }
        // The original performing service capability remains live until process
        // exit. The enclosing foreground owner must settle the entire hierarchy.
        std::process::exit(7);
    });
}

async fn literal_bound_service_fixture(
    manager: &pse_host::HostManager,
    current: &pse_host::UnitObservation,
    directory: &Path,
) {
    let executable = directory.join("sh-$HOME-%n");
    std::os::unix::fs::symlink("/bin/sh", &executable).unwrap();
    let arguments = [
        "$PSE_LITERAL_VALUE",
        "${PSE_LITERAL_VALUE}",
        "$$",
        "$",
        "%n",
        "%%",
        "%",
        "",
        "space and\t tab",
        "line\nbreak",
        "quote'\"\\",
        "unicode-λ",
    ];
    let environment = "value $HOME ${OTHER} $$ %n %% % literal";
    let output = directory.join("literal-argv");
    let mut argv = vec![
        "-c".to_owned(),
        r#"printf '%s\000' "$@" "$PSE_LITERAL_VALUE" > "$PSE_LITERAL_OUTPUT"; exec /bin/sleep 100"#
            .to_owned(),
        "fixture".to_owned(),
    ];
    argv.extend(arguments.iter().map(|value| (*value).to_owned()));
    let spec = pse_host::ServiceSpec {
        intent: pse_host::LaunchIntent::fresh().unwrap(),
        executable,
        arguments: argv,
        environment: vec![
            format!("PSE_LITERAL_VALUE={environment}"),
            format!("PSE_LITERAL_OUTPUT={}", output.display()),
        ],
    };
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut service = manager
        .start_service_bound(
            &spec,
            &manager.borrow(&current.unit),
            &pse_host::ResourceControls::default(),
            deadline,
        )
        .await
        .unwrap();
    let expected = arguments
        .into_iter()
        .chain(std::iter::once(environment))
        .flat_map(|value| value.as_bytes().iter().copied().chain(std::iter::once(0)))
        .collect::<Vec<_>>();
    let result = async {
        while fs::read(&output).is_ok_and(|bytes| bytes.len() < expected.len()) || !output.exists()
        {
            assert!(
                Instant::now() < deadline,
                "literal service argument observation incomplete"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        fs::read(&output)
    }
    .await;
    let cleanup = service.stop_and_drain(deadline).await;
    assert!(
        cleanup.is_ok(),
        "literal service original cleanup failed: {cleanup:?}"
    );
    assert_eq!(
        result.unwrap(),
        expected,
        "native service must preserve literal argument and environment bytes"
    );
}

#[test]
fn foreground_bound_service_preserves_literal_arguments_environment_and_executable_path() {
    let temp = tempfile::tempdir().unwrap();
    let mut command = host();
    command
        .env("PSE_FOREST_FIXTURE_DIRECTORY", temp.path())
        .env("PSE_FOREST_FIXTURE_LITERAL", "1")
        .args(["run"])
        .arg(std::env::current_exe().unwrap())
        .args(["--exact", "ordinary_bound_service_fixture", "--nocapture"]);
    let mut child = Guard::top_level(&mut command);
    let result = child.wait().unwrap();
    let mut errors = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut errors)
        .unwrap();
    assert_eq!(result.code(), Some(7), "{errors}");
}

fn bound_service_forest(signal: Option<rustix::process::Signal>) {
    let temp = tempfile::tempdir().unwrap();
    let mut command = host();
    command
        .env("PSE_FOREST_FIXTURE_DIRECTORY", temp.path())
        .args(["run"])
        .arg(std::env::current_exe().unwrap())
        .args(["--exact", "ordinary_bound_service_fixture", "--nocapture"]);
    let mut child = Guard::top_level(&mut command);
    wait_file(&temp.path().join("ready"));
    let ready = fs::read_to_string(temp.path().join("ready")).unwrap();
    let rows = ready.lines().collect::<Vec<_>>();
    assert_eq!(rows.len(), 3, "{ready}");
    let operation = Path::new(rows[0]).parent().unwrap();
    assert!(
        operation
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("-operation_")
    );
    assert_eq!(
        Path::new(rows[1]).parent(),
        Some(operation),
        "bound service must share the private operation hierarchy"
    );
    let descendant = fs::read_to_string(temp.path().join("descendant")).unwrap();
    let descendant = descendant.lines().collect::<Vec<_>>();
    assert_eq!(descendant[0], rows[1]);
    assert!(
        Path::new(&format!("/proc/{}", descendant[1])).exists(),
        "original descendant PID absent: {descendant:?}; {ready}"
    );
    record_fresh_group(&mut child, rows[0]);
    if let Some(signal) = signal {
        rustix::process::kill_process(
            rustix::process::Pid::from_raw(child.id() as i32).unwrap(),
            signal,
        )
        .unwrap();
    } else {
        fs::write(temp.path().join("release"), b"exit\n").unwrap();
    }
    let status = child.wait().unwrap();
    let mut errors = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut errors)
        .unwrap();
    assert_eq!(
        status.code(),
        Some(match signal {
            None => 7,
            Some(rustix::process::Signal::TERM) => 143,
            Some(_) => 137,
        }),
        "{errors}"
    );
    if signal != Some(rustix::process::Signal::KILL) {
        assert!(
            empty(operation.to_str().unwrap()),
            "boundary returned before its original operation forest drained"
        );
        assert!(
            temp.path().join("drained").exists(),
            "delayed descendant settlement must precede the original boundary result"
        );
    }
    // Independent kernel observation includes every descendant, beyond the
    // workload scope and beyond dependency StopUnit acknowledgement.
    wait_empty(operation.to_str().unwrap());
    wait_empty(rows[0]);
    wait_empty(rows[1]);
    assert!(temp.path().join("term").exists());
    assert!(temp.path().join("drained").exists());
}

#[test]
fn foreground_ordinary_bound_service_forest_drains_before_normal_result() {
    bound_service_forest(None);
}

#[test]
fn foreground_ordinary_bound_service_forest_drains_on_controller_death() {
    bound_service_forest(Some(rustix::process::Signal::TERM));
    bound_service_forest(Some(rustix::process::Signal::KILL));
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
fn foreground_explicit_finite_scope_and_nested_borrow_keep_actual_envelope_and_release() {
    let temp = tempfile::tempdir().unwrap();
    let observation = temp.path().join("finite-observation");
    let mut command = host();
    command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .env("PSE_BUILD_MODE", "unbounded").env_remove("PSE_HOST_ALLOCATION")
        .env_remove("PSE_MEMORY_MAX").env("PSE_SLICE", "pse.slice")
        .args(["boundary", "--resource-class", "light", "--", "/bin/sh", "-ec", r#"
cat /proc/self/cgroup > "$1"
group=$(sed -n 's/^0:://p' /proc/self/cgroup)
cat "/sys/fs/cgroup$group/memory.max" >> "$1"
printf '%s\n' "$PSE_HOST_ALLOCATION" >> "$1"
"$2" boundary --resource-class light -- /bin/sh -ec 'cat /proc/self/cgroup; printf "%s\n" "$PSE_HOST_ALLOCATION"' >> "$1"
exit 7
"#, "fixture"]).arg(&observation).arg(env!("CARGO_BIN_EXE_pse-host"));
    let mut child = Guard::top_level(&mut command);
    let status = child.wait().unwrap();
    let mut errors = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut errors)
        .unwrap();
    assert_eq!(status.code(), Some(7), "{errors}");
    let document = fs::read_to_string(observation).unwrap();
    let lines = document.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 5, "{document}");
    assert_eq!(lines[1], (2_u64 << 30).to_string());
    assert_eq!(
        lines[0], lines[3],
        "nested command must reuse original containment"
    );
    assert_eq!(
        lines[2], lines[4],
        "nested command must reuse original charge"
    );
    let group = lines[0].strip_prefix("0::").unwrap();
    assert!(group.contains("pse.slice/pse-"), "{group}");
    record_fresh_group(&mut child, group);
    wait_empty(group);
    let journal = Path::new(&std::env::var_os("XDG_RUNTIME_DIR").unwrap())
        .join("pse-host-admission/finite.json");
    let state: serde_json::Value = serde_json::from_slice(&fs::read(journal).unwrap()).unwrap();
    assert!(
        state["finite"].get(lines[2]).is_none(),
        "original finite charge remains after successful boundary release"
    );
}

#[test]
fn foreground_ordinary_explicit_target_preserves_owner_and_refuses_nested_escape() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let manager = runtime
        .block_on(pse_host::HostManager::connect_user(None))
        .unwrap();
    let intent = pse_host::LaunchIntent::fresh_allocation("app.slice").unwrap();
    let mut target = runtime
        .block_on(manager.start_slice(
            &intent,
            &pse_host::ResourceControls::default(),
            Instant::now() + Duration::from_secs(15),
        ))
        .unwrap();
    let original = runtime.block_on(target.observe()).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let temp = tempfile::tempdir().unwrap();
        let observation = temp.path().join("placement");
        let escaped = temp.path().join("must-not-execute");
        let mut command = host();
        command
            .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .env("PSE_BUILD_MODE", "unbounded")
            .env_remove("PSE_HOST_ALLOCATION")
            .env_remove("PSE_MEMORY_MAX")
            .env("PSE_SLICE", intent.unit())
            .args([
                "boundary",
                "--resource-class",
                "build",
                "--",
                "/bin/sh",
                "-ec",
                r#"
cat /proc/self/cgroup > "$1"
cat "/proc/$PPID/cgroup" >> "$1"
"$2" boundary --resource-class build -- /bin/cat /proc/self/cgroup >> "$1"
set +e
/usr/bin/env PSE_SLICE=app.slice "$2" boundary --resource-class build -- /usr/bin/touch "$3"
status=$?
set -e
test "$status" -eq 125
test -z "${PSE_HOST_ALLOCATION-}"
group=$(sed -n 's/^0:://p' /proc/self/cgroup)
cat "/sys/fs/cgroup$group/memory.max" >> "$1"
"#,
                "fixture",
            ])
            .arg(&observation)
            .arg(env!("CARGO_BIN_EXE_pse-host"))
            .arg(&escaped);
        let mut child = Guard::top_level(&mut command);
        let status = child.wait().unwrap();
        let mut errors = String::new();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut errors)
            .unwrap();
        assert!(status.success(), "{errors}");
        assert!(!escaped.exists());
        let document = fs::read_to_string(observation).unwrap();
        let rows = document.lines().collect::<Vec<_>>();
        assert_eq!(rows.len(), 4, "{document}");
        assert_eq!(
            rows[0], rows[2],
            "nested work must retain actual target containment"
        );
        let group = rows[0].strip_prefix("0::").unwrap();
        let operation = Path::new(group).parent().unwrap();
        assert_eq!(operation.parent(), Some(Path::new(&original.control_group)));
        assert!(
            operation
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("-operation_")
        );
        assert!(
            !rows[1].contains(intent.unit()),
            "controller must retain its prior parent"
        );
        assert_eq!(
            rows[3], "max",
            "ordinary work must not acquire a new leaf cap"
        );
        record_fresh_group(&mut child, group);
        wait_empty(group);
        wait_empty(operation.to_str().unwrap());
    }));
    let cleanup = runtime.block_on(target.stop_and_drain(Instant::now() + Duration::from_secs(15)));
    assert!(
        cleanup.is_ok(),
        "original target cleanup failed after body success={}: {cleanup:?}",
        result.is_ok()
    );
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn foreground_finite_controller_death_drains_original_work_and_reclaims_only_afterward() {
    for signal in [rustix::process::Signal::TERM, rustix::process::Signal::KILL] {
        let temp = tempfile::tempdir().unwrap();
        let ready = temp.path().join("ready");
        let mut command = host();
        command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .env("PSE_BUILD_MODE", "unbounded").env_remove("PSE_HOST_ALLOCATION")
            .env_remove("PSE_MEMORY_MAX").env("PSE_SLICE", "pse.slice")
            .args(["boundary", "--resource-class", "light", "--", "/bin/sh", "-ec",
                "printf '%s\\n' \"$PSE_HOST_ALLOCATION\" > \"$1\"; cat /proc/self/cgroup >> \"$1\"; exec sleep 100", "fixture"])
            .arg(&ready);
        let mut child = Guard::top_level(&mut command);
        wait_file(&ready);
        // The complete observation binds the actual original workload independently
        // of the controller's own cleanup report.
        let deadline = Instant::now() + Duration::from_secs(15);
        let document = loop {
            let document = fs::read_to_string(&ready).unwrap();
            if document.lines().count() == 2 {
                break document;
            }
            assert!(Instant::now() < deadline, "finite observation incomplete");
            thread::sleep(Duration::from_millis(5));
        };
        let rows = document.lines().collect::<Vec<_>>();
        let group = rows[1].strip_prefix("0::").unwrap();
        record_fresh_group(&mut child, group);
        rustix::process::kill_process(
            rustix::process::Pid::from_raw(child.id() as i32).unwrap(),
            signal,
        )
        .unwrap();
        let status = child.wait().unwrap();
        let mut errors = String::new();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut errors)
            .unwrap();
        assert_eq!(
            status.code(),
            Some(if signal == rustix::process::Signal::TERM {
                143
            } else {
                137
            }),
            "{errors}"
        );
        wait_empty(group);

        // SIGKILL cannot execute local release. A new real admission must reconcile
        // the original bound unit/cgroup before reclaiming that dead owner's row.
        let mut next = host();
        next.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .env("PSE_BUILD_MODE", "unbounded")
            .env_remove("PSE_HOST_ALLOCATION")
            .env_remove("PSE_MEMORY_MAX")
            .env("PSE_SLICE", "pse.slice")
            .args(["boundary", "--resource-class", "light", "--", "/bin/true"]);
        let mut next = Guard::top_level(&mut next);
        let status = next.wait().unwrap();
        let mut errors = String::new();
        next.stderr
            .take()
            .unwrap()
            .read_to_string(&mut errors)
            .unwrap();
        assert!(status.success(), "{errors}");
        let journal = Path::new(&std::env::var_os("XDG_RUNTIME_DIR").unwrap())
            .join("pse-host-admission/finite.json");
        let state: serde_json::Value = serde_json::from_slice(&fs::read(journal).unwrap()).unwrap();
        assert!(
            state["finite"].get(rows[0]).is_none(),
            "drained original finite owner was not reclaimed"
        );
    }
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
