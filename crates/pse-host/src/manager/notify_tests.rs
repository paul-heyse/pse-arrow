// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use futures_util::FutureExt;
use std::{os::unix::ffi::OsStringExt, panic::AssertUnwindSafe};

const FIXTURE_ROOT: &str = "PSE_NATIVE_NOTIFY_FIXTURE_ROOT";
const FIXTURE_INTENT: &str = "PSE_NATIVE_NOTIFY_FIXTURE_INTENT";
const MAIN_ENTRY: &str = "manager::notify_tests::notify_service_main_entry";
const FINALIZER_ENTRY: &str = "manager::notify_tests::notify_service_stop_post_entry";

#[derive(Debug, Serialize, Deserialize)]
struct MainReceipt {
    pid: u32,
    invocation: Vec<u8>,
}
#[derive(Debug, Serialize, Deserialize)]
struct FinalizerReceipt {
    pid: u32,
    invocation: Vec<u8>,
    arguments: Vec<String>,
    service_result: String,
    controller_refused: bool,
}

fn failure(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}
fn fixture_root() -> Option<PathBuf> {
    std::env::var_os(FIXTURE_ROOT).map(PathBuf::from)
}
fn fixture_intent() -> LaunchIntent {
    serde_json::from_str(&std::env::var(FIXTURE_INTENT).unwrap()).unwrap()
}
fn receipt<T: Serialize>(path: &Path, value: &T) {
    let pending = path.with_extension("pending");
    std::fs::write(&pending, serde_json::to_vec(value).unwrap()).unwrap();
    std::fs::rename(pending, path).unwrap();
}

#[tokio::test]
async fn notify_service_main_entry() {
    let Some(root) = fixture_root() else { return };
    let manager = HostManager::connect_user(None).await.unwrap();
    let intent = fixture_intent();
    let original = manager
        .authenticate_service_controller(&intent)
        .await
        .unwrap();
    receipt(
        &root.join("main.json"),
        &MainReceipt {
            pid: std::process::id(),
            invocation: original.invocation,
        },
    );
    tokio::time::timeout(Duration::from_secs(15), async {
        while !root.join("ready").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let notifier = crate::managed::readiness::SupervisorNotifier::authenticate(&manager, &intent)
        .await
        .unwrap();
    notifier
        .send_ready("original service main ready")
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while !root.join("exit").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn notify_service_stop_post_entry() {
    let Some(root) = fixture_root() else { return };
    let manager = HostManager::connect_user(None).await.unwrap();
    let invocation = uuid::Uuid::parse_str(&std::env::var("INVOCATION_ID").unwrap()).unwrap();
    receipt(
        &root.join("finalizer.json"),
        &FinalizerReceipt {
            pid: std::process::id(),
            invocation: invocation.as_bytes().to_vec(),
            arguments: std::env::args().collect(),
            service_result: std::env::var("SERVICE_RESULT").unwrap(),
            controller_refused: manager
                .authenticate_service_controller(&fixture_intent())
                .await
                .is_err(),
        },
    );
}

#[test]
fn service_control_commands_are_literal_and_require_absolute_utf8_executables() {
    let arguments = vec![
        "$LITERAL".into(),
        "%i".into(),
        "argument with spaces".into(),
    ];
    let (executable, argv, flags) = service_command(Path::new("/bin/example"), &arguments).unwrap();
    assert_eq!(executable, "/bin/example");
    assert_eq!(argv[1..], arguments);
    assert_eq!(flags, ["no-env-expand"]);
    assert!(service_command(Path::new("relative"), &[]).is_err());
    let non_utf8 = PathBuf::from(std::ffi::OsString::from_vec(vec![b'/', 0xff]));
    assert!(service_command(&non_utf8, &[]).is_err());
}

#[test]
fn stopping_only_accepts_known_original_absence_and_never_reuse_or_unknown_identity() {
    use pse_diagnostics::process_facts::ProcessStat;
    let original = ProcessIdentity {
        boot_id: "original-boot".into(),
        pid_namespace: "original-namespace".into(),
        pid: 42,
        start_ticks: 100,
    };
    let facts = |identity: ProcessIdentity| ProcessFacts {
        stat: ProcessStat {
            pid: identity.pid,
            state: 'S',
            parent_pid: 1,
            process_group: 42,
            start_ticks: identity.start_ticks,
        },
        identity,
    };
    for stopping in [false, true] {
        assert!(original_main_matches(
            Some(&original),
            42,
            ProcessRead::Present(facts(original.clone())),
            stopping
        ));
        let mut reused = original.clone();
        reused.start_ticks += 1;
        assert!(!original_main_matches(
            Some(&original),
            42,
            ProcessRead::Present(facts(reused)),
            stopping
        ));
        assert!(!original_main_matches(
            Some(&original),
            42,
            ProcessRead::Unavailable("unavailable".into()),
            stopping
        ));
        assert!(!original_main_matches(
            Some(&original),
            42,
            ProcessRead::Changed,
            stopping
        ));
        assert!(!original_main_matches(
            None,
            42,
            ProcessRead::Absent,
            stopping
        ));
        assert!(!original_main_matches(
            None,
            42,
            ProcessRead::Present(facts(original.clone())),
            stopping
        ));
        assert!(!original_main_matches(
            Some(&original),
            43,
            ProcessRead::Absent,
            stopping
        ));
        assert_eq!(
            original_main_matches(Some(&original), 42, ProcessRead::Absent, stopping),
            stopping
        );
    }
}

async fn cleanup(
    manager: &HostManager,
    intent: &LaunchIntent,
    owned: &mut Option<OwnedUnit>,
) -> Result<(), HostError> {
    let deadline = Instant::now() + Duration::from_secs(10);
    if owned.is_none() {
        *owned = manager.reconcile_if_present(intent, deadline).await?;
    }
    if let Some(owned) = owned {
        owned.stop_and_drain(deadline).await?;
        if !owned.released() {
            return Err(failure(
                "notify fixture retained original ownership after drain",
            ));
        }
    }
    Ok(())
}

async fn native_notify_case(ready: bool) {
    let manager = HostManager::connect_user(None).await.unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let intent = LaunchIntent::fresh().unwrap();
    let spec = ServiceSpec {
        intent: intent.clone(),
        executable: executable.clone(),
        arguments: vec!["--exact".into(), MAIN_ENTRY.into(), "--nocapture".into()],
        environment: vec![
            format!("{FIXTURE_ROOT}={}", scratch.path().to_str().unwrap()),
            format!(
                "{FIXTURE_INTENT}={}",
                serde_json::to_string(&intent).unwrap()
            ),
            "PSE_STOP_POST_LITERAL=must-not-expand".into(),
        ],
    };
    let finalizer = ServiceFinalizer {
        executable: executable.clone(),
        arguments: vec![
            "--exact".into(),
            FINALIZER_ENTRY.into(),
            "--nocapture".into(),
            "--skip".into(),
            "$PSE_STOP_POST_LITERAL-%i".into(),
        ],
    };
    let mut owned = None;
    // Catch panics as well as errors so every submitted original intent reaches owned cleanup.
    let checked = AssertUnwindSafe(async {
        let deadline = Instant::now() + Duration::from_secs(if ready { 10 } else { 2 });
        let launching = manager.start_notify_service(&spec, &finalizer, deadline);
        tokio::pin!(launching);
        let original: MainReceipt = loop {
            tokio::select! {
                result = &mut launching => {
                    owned = Some(result?);
                    return Err(failure("notify start completed before the original main sent READY"));
                }
                () = tokio::time::sleep(Duration::from_millis(10)) => {
                    match std::fs::read(scratch.path().join("main.json")) {
                        Ok(bytes) => break serde_json::from_slice(&bytes)
                            .map_err(|error| failure(format!("invalid main receipt: {error}")))?,
                        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        };
        let before = manager.observe(intent.unit()).await?;
        if before.main_pid != original.pid || before.invocation != original.invocation
            || before.active_state != "activating" {
            return Err(failure("notify job is not waiting on the authenticated original main"));
        }
        let unit = manager.unit(intent.unit()).await?;
        let service = ServiceProxy::builder(&manager.connection)
            .destination(manager.owner.as_str()).path(unit.inner().path().to_owned())
            .cache_properties(CacheProperties::No).build().await?;
        let commands = service.exec_stop_post_ex().await?;
        let mut expected_argv = vec![executable.to_str().unwrap().to_owned()];
        expected_argv.extend(finalizer.arguments.clone());
        if service.type_property().await? != "notify" || service.notify_access().await? != "main"
            || service.restart().await? != "no" || service.remain_after_exit().await?
            || !matches!(commands.as_slice(), [(path, argv, flags, ..)]
                if Path::new(path) == finalizer.executable && argv == &expected_argv
                    && flags == &vec!["no-env-expand".to_owned()]) {
            return Err(failure("native notify/finalizer configuration differs from submitted command"));
        }
        if ready {
            std::fs::write(scratch.path().join("ready"), b"original main may notify")?;
            owned = Some(launching.await?);
            let owned = owned.as_ref().ok_or_else(|| failure("original notify ownership missing"))?;
            let observed = owned.observe().await?;
            if owned.launch_job_result() != Some("done") || observed.active_state != "active"
                || observed.main_pid != original.pid || observed.invocation != original.invocation {
                return Err(failure("READY did not complete the matching original job/main identity"));
            }
            std::fs::write(scratch.path().join("exit"), b"main may finish")?;
            // Natural main exit must run stop-post without a later StopUnit trigger.
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    let observed = manager.observe(intent.unit()).await?;
                    if observed.invocation != original.invocation {
                        return Err(failure("notify incarnation changed during original completion"));
                    }
                    if matches!(observed.active_state.as_str(), "inactive" | "failed") {
                        return Ok::<_, HostError>(());
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }).await.map_err(|_| HostError::Deadline("notify fixture completion".into()))??;
        } else {
            match launching.await {
                Err(HostError::UnknownLaunch { unit, .. }) if unit == intent.unit() => {}
                Err(error) => return Err(error),
                Ok(unit) => {
                    owned = Some(unit);
                    return Err(failure("withheld READY unexpectedly completed the start job"));
                }
            }
        }
        Ok(original)
    }).catch_unwind().await;
    let cleaned = cleanup(&manager, &intent, &mut owned).await;
    if let Err(error) = cleaned {
        // The finalizer may still need this context. Preserve it for intent-bound recovery,
        // rather than turn a cleanup observation failure into a second fixture I/O failure.
        let retained = scratch.keep();
        panic!(
            "original notify {} ({}) cleanup failed: {error:?}; retained fixture {}",
            intent.unit(),
            intent.description,
            retained.display()
        );
    }
    let original = match checked {
        Ok(result) => result.unwrap(),
        Err(panic) => std::panic::resume_unwind(panic),
    };
    // Inspect stop-post evidence only after original service completion and physical drain.
    let finalizer_receipt: FinalizerReceipt =
        serde_json::from_slice(&std::fs::read(scratch.path().join("finalizer.json")).unwrap())
            .unwrap();
    assert_eq!(finalizer_receipt.invocation, original.invocation);
    assert_ne!(finalizer_receipt.pid, original.pid);
    assert!(finalizer_receipt.controller_refused);
    assert!(!finalizer_receipt.service_result.is_empty());
    let mut expected_argv = vec![executable.to_str().unwrap().to_owned()];
    expected_argv.extend(finalizer.arguments);
    assert_eq!(finalizer_receipt.arguments, expected_argv);
}

#[tokio::test]
async fn real_notify_service_matches_original_ready_job_and_runs_finalizer_after_main_exit() {
    native_notify_case(true).await;
}

#[tokio::test]
async fn real_notify_service_unknown_ack_is_reconciled_and_finalized_without_resubmission() {
    native_notify_case(false).await;
}
