// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use futures_util::FutureExt;
use std::{panic::AssertUnwindSafe, time::Duration};

pub(in crate::managed) const FRONTEND: &str = "PSE_RECORDED_FRONTEND";
pub(in crate::managed) const FRONTEND_ROOT: &str = "PSE_RECORDED_FRONTEND_ROOT";
pub(in crate::managed) const FRONTEND_ID: &str = "PSE_RECORDED_FRONTEND_ID";
pub(in crate::managed) const FRONTEND_ENTRY: &str =
    "managed::supervisor::tests::detached_frontend_entry";
const PUBLICATION_COLLISION: &str = "PSE_DETACHED_TEST_PUBLICATION_COLLISION";

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
    let collision = std::env::var_os(PUBLICATION_COLLISION).is_some();
    if collision {
        directory
            .publish_evidence(InvocationEvidence::Launch, &"existing immutable record")
            .unwrap();
    }
    let launched = launch_generic(ManagedRequest {
        root,
        directory,
        command,
        selection,
        policy,
    })
    .await;
    if collision {
        assert!(
            matches!(launched, Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::AlreadyExists)
        );
        std::fs::write(
            scratch.join("ack"),
            b"original publication refusal and settlement completed",
        )
        .unwrap();
        return;
    }
    let launched = launched.unwrap();
    std::fs::write(
        scratch.join("ack"),
        serde_json::to_vec(&launched.observation).unwrap(),
    )
    .unwrap();
}

enum LifetimeCase {
    Exit,
    Cancel,
    Crash,
    PublicationCollision,
}
async fn detached_case(finite: bool, case: LifetimeCase) {
    let cancel = matches!(case, LifetimeCase::Cancel);
    let crash = matches!(case, LifetimeCase::Crash);
    let collision = matches!(case, LifetimeCase::PublicationCollision);
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
    {
        let scratch = tempfile::tempdir().unwrap();
        let mut environment: Environment = std::env::vars_os().collect();
        environment.insert(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        );
        environment.insert("PSE_BUILD_MODE".into(), "unbounded".into());
        if collision {
            environment.insert(PUBLICATION_COLLISION.into(), "yes".into());
        }
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
        let selection =
            Selection::load(&root, &environment, finite.then_some("light"), None).unwrap();
        let script = if cancel || crash {
            "cat /proc/self/cgroup; printf running; setsid /bin/sleep 60 & wait"
        } else {
            "cat /proc/self/cgroup; /bin/sleep 0.2; printf completed; exit 4"
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
            let child_pid = child.id().unwrap();
            let pid = rustix::process::Pid::from_raw(i32::try_from(child_pid).unwrap()).unwrap();
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
            if collision {
                // The real finite slice was prepared before the deliberately conflicting
                // immutable publication. Returning the original I/O refusal requires all
                // owned cleanup attempts to succeed and retire this exact caller's charge.
                let journal = HostPaths::new(&environment).runtime()?.join("pse-host-admission/finite.json");
                let state: serde_json::Value = serde_json::from_slice(&std::fs::read(journal)?).unwrap();
                assert!(state["finite"].as_object().unwrap().values().all(|row| row["caller"]["pid"].as_u64() != Some(u64::from(child_pid))));
                return Ok(());
            }
            let custody: super::super::service_payload::ServiceCustody = directory.evidence(InvocationEvidence::Custody)?.unwrap();
            let controller: ControllerRecord = directory.evidence(InvocationEvidence::Controller)?.unwrap();
            assert_eq!(custody.version, 5);
            assert_eq!(custody.service, controller.binding, "wrapper and original child require one service root");
            if (cancel || crash) && finite {
                let limit = std::fs::read_to_string(PathBuf::from("/sys/fs/cgroup").join(custody.service.group.trim_start_matches('/')).join("memory.max"))?;
                assert_ne!(limit.trim(), "max", "finite service wrapper must be constrained from birth");
            }
            recorded = Some(super::recorded(&root, &id, &environment).await?);
            let recovered = recorded
                .as_mut()
                .ok_or_else(|| invalid("recorded control missing"))?;
            if cancel || crash {
                if recovered
                    .wait(Some(Instant::now() + Duration::from_millis(10)))
                    .await?
                    .is_some()
                {
                    return Err(invalid(
                        "bounded independent wait stopped ongoing detached work",
                    ));
                }
                if crash {
                    let pid = rustix::process::Pid::from_raw(i32::try_from(controller.process.pid).unwrap()).unwrap();
                    let fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).map_err(std::io::Error::from)?;
                    assert!(matches!(ProcessReader::default().process(controller.process.pid), ProcessRead::Present(facts) if facts.identity == controller.process));
                    rustix::process::pidfd_send_signal(&fd, rustix::process::Signal::KILL).map_err(std::io::Error::from)?;
                    recovered.wait(Some(Instant::now() + Duration::from_secs(15))).await?
                        .ok_or_else(|| invalid("native wrapper crash did not drain original descendants"))?;
                } else {
                    recovered.cancel(Instant::now() + Duration::from_secs(15)).await?;
                }
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
            if !cancel && !crash && result.child_exit != Some(4) {
                return Err(invalid(
                    "independent detached wait changed original payload status",
                ));
            }
            assert!(result.payload.native_lifetime_pending, "immutable operation report cannot prove wrapper exit");
            assert_eq!(result.settlement, crate::envelope::Settlement::Unknown);
            if crash {
                assert_eq!(result.child_exit, None, "native wrapper death cannot reconstruct original child reap");
                assert!(result.payload.stdout.is_none() && result.payload.stderr.is_none());
                assert_eq!(result.observation.completeness, crate::envelope::Completeness::Partial);
                assert!(!result.failures.is_empty());
            } else {
                assert!(std::fs::read_to_string(directory.path().join("stdout.raw"))?.contains(&format!("0::{}", custody.service.group)));
                assert!(result.payload.stdout.as_ref().unwrap().complete && result.payload.stderr.as_ref().unwrap().complete, "native whole-population TERM must settle ordinary escaped pipe holders");
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
        } else if let Ok(mut recovered) = super::recorded(&root, &id, &environment).await
            && let Err(error) = recovered
                .cancel(Instant::now() + Duration::from_secs(10))
                .await
        {
            cleanup.push(error);
        }
        if let Some(launcher) = &mut launcher_unit
            && let Err(error) = launcher
                .stop_and_drain(Instant::now() + Duration::from_secs(10))
                .await
        {
            cleanup.push(error);
        }
        let _ = child.start_kill();
        let reap = child.wait().await;
        if !cleanup.is_empty() || reap.is_err() || !matches!(&checked, Ok(Ok(()))) {
            let retained = scratch.keep();
            panic!(
                "independent invocation {id} checked {checked:?}; cleanup {cleanup:?}; reap {reap:?}; retained {}",
                retained.display()
            );
        }
        match checked {
            Ok(result) => result.unwrap(),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}

#[tokio::test]
async fn detached_lifetime_ordinary_exit() {
    detached_case(false, LifetimeCase::Exit).await;
}

#[tokio::test]
async fn detached_lifetime_finite_exit() {
    detached_case(true, LifetimeCase::Exit).await;
}

#[tokio::test]
async fn detached_lifetime_ordinary_cancel() {
    detached_case(false, LifetimeCase::Cancel).await;
}

#[tokio::test]
async fn detached_lifetime_finite_cancel() {
    detached_case(true, LifetimeCase::Cancel).await;
}

#[tokio::test]
async fn detached_lifetime_ordinary_main_crash() {
    detached_case(false, LifetimeCase::Crash).await;
}

#[tokio::test]
async fn detached_lifetime_finite_main_crash() {
    detached_case(true, LifetimeCase::Crash).await;
}

#[tokio::test]
async fn detached_finite_publication_failure_settles_predispatch_allocation() {
    detached_case(true, LifetimeCase::PublicationCollision).await;
}
