// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use futures_util::FutureExt;
use std::{panic::AssertUnwindSafe, time::Duration};

#[tokio::test]
async fn original_origin_pidfd_refuses_reuse_and_observes_actual_exit() {
    let mut child = tokio::process::Command::new("/bin/sleep")
        .arg("60")
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let checked = AssertUnwindSafe(async {
        let identity = match ProcessReader::default().process(child.id().unwrap()) {
            ProcessRead::Present(facts) => facts.identity,
            facts => panic!("original origin process unavailable: {facts:?}"),
        };
        let lease = OriginLease::acquire(&identity).unwrap();
        let mut reused = identity.clone();
        reused.start_ticks += 1;
        assert!(OriginLease::acquire(&reused).is_err());
        assert!(
            tokio::time::timeout(Duration::from_millis(10), lease.ended())
                .await
                .is_err()
        );
        child.start_kill().unwrap();
        child.wait().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), lease.ended())
            .await
            .unwrap()
            .unwrap();
    })
    .catch_unwind()
    .await;
    let killed = child.start_kill();
    let reaped = child.wait().await;
    assert!(
        reaped.is_ok(),
        "original origin cleanup: kill={killed:?}, reap={reaped:?}"
    );
    if let Err(panic) = checked {
        std::panic::resume_unwind(panic);
    }
}

const FRONTEND: &str = "PSE_RECORDED_FRONTEND";
const FRONTEND_ROOT: &str = "PSE_RECORDED_FRONTEND_ROOT";
const FRONTEND_ID: &str = "PSE_RECORDED_FRONTEND_ID";
const FRONTEND_ENTRY: &str = "managed::supervisor::tests::detached_frontend_entry";

#[tokio::test]
async fn detached_frontend_entry() {
    let Some(scratch) = std::env::var_os(FRONTEND).map(PathBuf::from) else {
        return;
    };
    tokio::time::timeout(Duration::from_secs(10), async {
        while !scratch.join("go").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let root = PathBuf::from(std::env::var_os(FRONTEND_ROOT).unwrap());
    let id = CliInvocationId::try_from(std::env::var(FRONTEND_ID).unwrap()).unwrap();
    let environment: Environment = std::env::vars_os().collect();
    let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
    let directory = Arc::new(store.directory(&id).unwrap());
    let private: PrivateRequest = directory.take_request().unwrap();
    let (command, selection, policy) = private.into_parts().unwrap();
    let launched = launch_generic(ManagedRequest {
        root,
        directory,
        command,
        selection,
        policy,
        mode: LaunchMode::Detached,
    })
    .await
    .unwrap();
    std::fs::write(
        scratch.join("ack"),
        serde_json::to_vec(&launched.observation).unwrap(),
    )
    .unwrap();
}

#[tokio::test]
async fn detached_launch_survives_original_process_and_recorded_control_waits_and_cancels() {
    use crate::invocation::InvocationSpec;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let control = crate::generation::prepared(&root).unwrap().unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    for cancel in [false, true] {
        let scratch = tempfile::tempdir().unwrap();
        let mut environment: Environment = std::env::vars_os().collect();
        environment.insert(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        );
        environment.insert("PSE_BUILD_MODE".into(), "unbounded".into());
        for key in [
            "PSE_HOST_ALLOCATION",
            "PSE_RESOURCE_CLASS",
            "PSE_MEMORY_MAX",
            "PSE_SLICE",
        ] {
            environment.remove(std::ffi::OsStr::new(key));
        }
        let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
        let id = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: "independent detached native control",
                control: control.clone(),
                environment_digest: "private-input-fixture".into(),
                scientific: Vec::new(),
            })
            .unwrap();
        let selection = Selection::load(&root, &environment, None, None).unwrap();
        let script = if cancel {
            "printf running; setsid /bin/sleep 60 & wait"
        } else {
            "/bin/sleep 0.2; printf completed; exit 4"
        };
        directory
            .publish_request(
                &PrivateRequest::new(
                    PayloadCommand {
                        program: "/bin/sh".into(),
                        arguments: vec!["-c".into(), script.into()],
                        cwd: root.clone(),
                        environment: environment.clone(),
                    },
                    selection,
                    PayloadPolicy {
                        startup: Duration::from_secs(15),
                        drain: Duration::from_secs(10),
                        write: Duration::from_secs(2),
                    },
                )
                .unwrap(),
            )
            .unwrap();
        let mut child = tokio::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", FRONTEND_ENTRY, "--nocapture"])
            .env_clear()
            .envs(&environment)
            .env(FRONTEND, scratch.path())
            .env(FRONTEND_ROOT, &root)
            .env(FRONTEND_ID, id.as_str())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let launcher = LaunchIntent::fresh_scope().unwrap();
        let mut launcher_unit = None;
        let mut recorded = None;
        let checked = AssertUnwindSafe(async {
            let pid = rustix::process::Pid::from_raw(child.id().unwrap() as i32).unwrap();
            let pidfd =
                rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
            launcher_unit = Some(
                manager
                    .start_scope(
                        &launcher,
                        pidfd,
                        &slice,
                        &slice,
                        Instant::now() + Duration::from_secs(10),
                        Duration::from_secs(5),
                    )
                    .await?,
            );
            std::fs::write(
                scratch.path().join("go"),
                b"native application placement established",
            )?;
            let status = tokio::time::timeout(Duration::from_secs(20), child.wait())
                .await
                .map_err(|_| HostError::Deadline("original detached launcher reap".into()))??;
            if !status.success() || !scratch.path().join("ack").exists() {
                return Err(invalid(
                    "independent detached frontend did not acknowledge launch",
                ));
            }
            recorded = Some(super::recorded(&root, &id, &environment).await?);
            let recovered = recorded
                .as_mut()
                .ok_or_else(|| invalid("recorded control missing"))?;
            if cancel {
                if recovered
                    .wait(Some(Instant::now() + Duration::from_millis(10)))
                    .await?
                    .is_some()
                {
                    return Err(invalid(
                        "bounded independent wait stopped ongoing detached work",
                    ));
                }
                recovered
                    .cancel(Instant::now() + Duration::from_secs(15))
                    .await?;
            } else {
                recovered
                    .wait(Some(Instant::now() + Duration::from_secs(15)))
                    .await?
                    .ok_or_else(|| {
                        invalid("independent native wait expired before original completion")
                    })?;
            }
            let result = store
                .result::<String, ManagedSummary>(&id)?
                .ok_or_else(|| {
                    invalid("original service ended without a readable terminal result")
                })?;
            if !cancel && result.child_exit != Some(4) {
                return Err(invalid(
                    "independent detached wait changed original payload status",
                ));
            }
            Ok::<_, HostError>(())
        })
        .catch_unwind()
        .await;
        let mut cleanup = Vec::new();
        if let Some(recorded) = &mut recorded {
            if let Err(error) = recorded
                .cancel(Instant::now() + Duration::from_secs(10))
                .await
            {
                cleanup.push(error);
            }
        } else if let Ok(mut recovered) = super::recorded(&root, &id, &environment).await {
            if let Err(error) = recovered
                .cancel(Instant::now() + Duration::from_secs(10))
                .await
            {
                cleanup.push(error);
            }
        }
        if let Some(launcher) = &mut launcher_unit {
            if let Err(error) = launcher
                .stop_and_drain(Instant::now() + Duration::from_secs(10))
                .await
            {
                cleanup.push(error);
            }
        }
        let _ = child.start_kill();
        let reap = child.wait().await;
        if !cleanup.is_empty() || reap.is_err() {
            let retained = scratch.keep();
            panic!(
                "independent invocation {id} cleanup {cleanup:?}; reap {reap:?}; retained {}",
                retained.display()
            );
        }
        match checked {
            Ok(result) => result.unwrap(),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}
