// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual nested supervisor death: protected subgroup recovery never retires its live parent.
use super::*;
use crate::{envelope::Settlement, invocation::InvocationSpec, managed::result::ManagedSummary};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;

const FIXTURE: &str = "PSE_NESTED_CRASH_FIXTURE";
const PARENT_ENTRY: &str = "managed::supervisor::ancestry::crash_tests::nested_crash_parent_entry";
const CHILD_ENTRY: &str = "managed::supervisor::ancestry::crash_tests::nested_crash_child_entry";

async fn until_file(path: &std::path::Path) -> Result<(), HostError> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match std::fs::read(path) {
                Ok(bytes) if !bytes.is_empty() => return Ok(()),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(HostError::Io(error)),
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .map_err(|_| HostError::Deadline("original nested crash fixture receipt".into()))?
}

#[tokio::test]
async fn nested_crash_child_entry() {
    let Some(scratch) = std::env::var_os(FIXTURE).map(PathBuf::from) else {
        return;
    };
    let environment = std::env::vars_os().collect();
    let leader = current_payload(&environment).await.unwrap();
    let mut escaped = tokio::process::Command::new("/usr/bin/setsid")
        .args([
            "/bin/sh",
            "-c",
            "echo $$ > \"$1\"; exec /bin/sleep 90",
            "fixture",
        ])
        .arg(scratch.join("escaped-pid"))
        .spawn()
        .unwrap();
    until_file(&scratch.join("escaped-pid")).await.unwrap();
    std::fs::write(scratch.join("child-ready"), leader.invocation_id().as_str()).unwrap();
    tokio::time::sleep(Duration::from_secs(90)).await;
    // If the parent did not deliver its intended SIGKILL, still join this locally owned child.
    let killed = escaped.start_kill();
    let reaped = escaped.wait().await;
    panic!("original nested crash child unexpectedly survived; cleanup {killed:?}/{reaped:?}");
}

#[tokio::test]
async fn nested_crash_parent_entry() {
    let Some(scratch) = std::env::var_os(FIXTURE).map(PathBuf::from) else {
        return;
    };
    let environment: Environment = std::env::vars_os().collect();
    let leader = current_payload(&environment).await.unwrap();
    let root = leader.root().to_owned();
    let metadata = leader
        .metadata::<Box<serde_json::value::RawValue>>()
        .unwrap();
    let selection = Selection::load(&root, &environment, None, None).unwrap();
    let parent = parent(
        &environment,
        &selection,
        Instant::now() + Duration::from_secs(10),
    )
    .await
    .unwrap()
    .unwrap();
    let parent_custody: payload::PayloadCustody = leader
        .directory()
        .evidence(InvocationEvidence::Custody)
        .unwrap()
        .unwrap();
    let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
    let id = CliInvocationId::new();
    let directory = Arc::new(
        store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: Some(parent.invocation_id().clone()),
                operation: "actual nested supervisor crash",
                control: metadata.selection.control,
                environment_digest: "nested-crash-private-fixture".into(),
                scientific: Vec::new(),
            })
            .unwrap(),
    );
    let mut launched = launch_generic(ManagedRequest {
        root: root.clone(),
        directory: directory.clone(),
        command: PayloadCommand {
            program: std::env::current_exe().unwrap(),
            arguments: vec!["--exact".into(), CHILD_ENTRY.into(), "--nocapture".into()],
            cwd: root.clone(),
            environment: environment.clone(),
        },
        selection,
        policy: PayloadPolicy {
            startup: Duration::from_secs(20),
            drain: Duration::from_secs(10),
            write: Duration::from_secs(2),
        },
        parent: Some(parent),
    })
    .await
    .unwrap();
    let checked = AssertUnwindSafe(async {
        until_file(&scratch.join("child-ready")).await?;
        let custody: payload::PayloadCustody =
            directory.evidence(InvocationEvidence::Custody)?.unwrap();
        assert_eq!(custody.version, 3);
        let original_group = custody.subgroup.as_ref().unwrap();
        let escaped_pid = std::fs::read_to_string(scratch.join("escaped-pid"))?
            .trim()
            .parse::<u32>()
            .unwrap();
        let escaped = match ProcessReader::default().process(escaped_pid) {
            ProcessRead::Present(facts) => facts.identity,
            facts => panic!("original escaped nested descendant unavailable: {facts:?}"),
        };
        let pid = rustix::process::Pid::from_raw(escaped.pid as i32).unwrap();
        let escaped_fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).map_err(std::io::Error::from)?;
        assert!(matches!(ProcessReader::default().process(escaped.pid), ProcessRead::Present(facts) if facts.identity == escaped));
        let controller: ControllerRecord =
            directory.evidence(InvocationEvidence::Controller)?.unwrap();
        assert_eq!(controller.process.pid, launched.observation.main_pid);
        // This is the actual original launch-client capability, not arbitrary-unit adoption.
        launched
            .unit
            .kill_and_drain(Instant::now() + Duration::from_secs(20))
            .await?;
        let exited = tokio::task::spawn_blocking(move || {
            let mut fds = [rustix::event::PollFd::new(&escaped_fd, rustix::event::PollFlags::IN)];
            let timeout = rustix::event::Timespec::try_from(Duration::from_secs(2)).unwrap();
            rustix::event::poll(&mut fds, Some(&timeout))
        }).await.map_err(HostError::Task)?.map_err(std::io::Error::from)?;
        assert_eq!(exited, 1, "original escaped descendant did not exit");
        let result = store
            .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
            .unwrap();
        assert_eq!(result.settlement, Settlement::Unknown);
        assert_eq!(result.child_exit, None);
        assert!(result.payload.stdout.is_none() && result.payload.stderr.is_none());
        assert_eq!(result.payload.service_result.as_deref(), Some("signal"));
        let finalization: serde_json::Value = directory
            .evidence(InvocationEvidence::Finalization)?
            .unwrap();
        assert_eq!(finalization["failures"], serde_json::json!([]));
        assert_eq!(finalization["launch_closed"], serde_json::json!(true));
        let subgroup = &finalization["recovery"]["subgroup"];
        assert_eq!(subgroup["kill"], serde_json::json!({"Ok":null}));
        assert_eq!(subgroup["drain"], serde_json::json!({"Ok":null}));
        let removal = &subgroup["removal"]["Ok"];
        assert_eq!(removal["original_removed"], serde_json::json!(true));
        assert_eq!(
            removal["original_device"],
            serde_json::json!(original_group.identity().0)
        );
        assert_eq!(
            removal["original_inode"],
            serde_json::json!(original_group.identity().1)
        );
        assert_eq!(removal["failures"], serde_json::json!([]));
        assert!(!original_group.path().try_exists()?);
        let manager = HostManager::connect_user(None).await?;
        let membership = manager.current_membership().await?;
        assert_eq!(membership.root().active_state, "active");
        if let Some(reservation) = parent_custody.reservation {
            let record = serde_json::to_value(reservation).unwrap();
            let bytes: Vec<u8> = serde_json::from_value(record["directory"].clone()).unwrap();
            let journal: serde_json::Value = serde_json::from_slice(&std::fs::read(
                PathBuf::from(OsString::from_vec(bytes)).join("finite.json"),
            )?)
            .unwrap();
            assert_eq!(
                journal["finite"][record["nonce"].as_str().unwrap()]["released"],
                serde_json::json!(false)
            );
        }
        std::fs::write(
            scratch.join("recovery-receipt.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind":"observed_native_nested_crash", "invocation":id, "subgroup":original_group,
                "escaped":escaped, "escaped_pidfd_ended":true, "parent_still_active":true,
                "terminal_settlement":"unknown", "finalization":finalization,
            }))
            .unwrap(),
        )?;
        Ok::<_, HostError>(())
    })
    .catch_unwind()
    .await;
    let cleanup = if launched.unit.released() {
        Ok(())
    } else {
        launched
            .cancel(Instant::now() + Duration::from_secs(15))
            .await
            .map(|_| ())
    };
    match checked {
        Ok(Ok(())) => assert!(
            cleanup.is_ok(),
            "original nested {id} cleanup={cleanup:?}; retained {}",
            scratch.display()
        ),
        Ok(Err(error)) => panic!(
            "original nested {id} failed {error:?}; cleanup={cleanup:?}; retained {}",
            scratch.display()
        ),
        Err(panic) => {
            eprintln!(
                "original nested {id} cleanup={cleanup:?}; retained {}",
                scratch.display()
            );
            std::panic::resume_unwind(panic);
        }
    }
}

#[tokio::test]
async fn actual_nested_supervisor_crash_recovers_ordinary_and_borrowed_finite_subgroups() {
    use super::super::tests::{FRONTEND, FRONTEND_ENTRY, FRONTEND_ID, FRONTEND_ROOT};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let receipts = root.join(".local/plan59-nested-crash");
    std::fs::create_dir_all(&receipts).unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    for finite in [false, true] {
        let scratch = tempfile::Builder::new()
            .prefix(if finite { "finite-" } else { "ordinary-" })
            .tempdir_in(&receipts)
            .unwrap();
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
            "PSE_MANAGED_INVOCATION",
            CHECKOUT,
            STATE_BASE,
        ] {
            environment.remove(std::ffi::OsStr::new(key));
        }
        environment.insert(FIXTURE.into(), scratch.path().as_os_str().to_owned());
        let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
        let id = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: "native nested crash parent",
                control: crate::generation::prepared(&root).unwrap().unwrap(),
                environment_digest: "nested-crash-private-fixture".into(),
                scientific: Vec::new(),
            })
            .unwrap();
        let selection =
            Selection::load(&root, &environment, finite.then_some("light"), None).unwrap();
        directory
            .publish_request(
                &PrivateRequest::new(
                    PayloadCommand {
                        program: std::env::current_exe().unwrap(),
                        arguments: vec![
                            "--exact".into(),
                            PARENT_ENTRY.into(),
                            "--nocapture".into(),
                        ],
                        cwd: root.clone(),
                        environment: environment.clone(),
                    },
                    selection,
                    PayloadPolicy {
                        startup: Duration::from_secs(20),
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
        let intent = LaunchIntent::fresh_scope().unwrap();
        let mut launcher = None;
        let checked = AssertUnwindSafe(async {
            let pid = rustix::process::Pid::from_raw(child.id().unwrap() as i32).unwrap();
            let fd =
                rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
            launcher = Some(
                manager
                    .start_scope(
                        &intent,
                        fd,
                        &slice,
                        &slice,
                        Instant::now() + Duration::from_secs(10),
                        Duration::from_secs(5),
                    )
                    .await?,
            );
            std::fs::write(scratch.path().join("go"), b"go")?;
            let status = tokio::time::timeout(Duration::from_secs(35), child.wait())
                .await
                .map_err(|_| HostError::Deadline("original nested crash frontend".into()))??;
            assert!(status.success(), "original frontend failed {status}");
            let mut original = recorded(&root, &id, &environment).await?;
            original
                .wait(Some(Instant::now() + Duration::from_secs(45)))
                .await?
                .ok_or_else(|| {
                    HostError::Deadline("original nested crash parent completion".into())
                })?;
            let result = store
                .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
                .unwrap();
            assert_eq!(result.child_exit, Some(0), "{result:?}");
            assert_eq!(result.settlement, Settlement::Complete, "{result:?}");
            assert!(scratch.path().join("recovery-receipt.json").is_file());
            Ok::<_, HostError>(())
        })
        .catch_unwind()
        .await;
        let mut failures = Vec::new();
        if directory
            .evidence::<LaunchRecord>(InvocationEvidence::Launch)
            .ok()
            .flatten()
            .is_some()
        {
            match recorded(&root, &id, &environment).await {
                Ok(mut original) => {
                    if let Err(error) = original
                        .cancel(Instant::now() + Duration::from_secs(15))
                        .await
                    {
                        failures.push(error);
                    }
                }
                Err(error) => failures.push(error),
            }
        }
        if let Err(error) = directory.discard_request() {
            failures.push(error);
        }
        if launcher.is_none() {
            match manager
                .reconcile_if_present(&intent, Instant::now() + Duration::from_secs(10))
                .await
            {
                Ok(unit) => launcher = unit,
                Err(error) => failures.push(error),
            }
        }
        if let Some(launcher) = &mut launcher
            && let Err(error) = launcher
                .stop_and_drain(Instant::now() + Duration::from_secs(15))
                .await
        {
            failures.push(error);
        }
        let _ = child.start_kill();
        if let Err(error) = child.wait().await {
            failures.push(error.into());
        }
        let retained = scratch.keep();
        eprintln!("original nested crash {id} retained {}", retained.display());
        assert!(
            failures.is_empty(),
            "original cleanup failures {failures:?}; retained {}",
            retained.display()
        );
        match checked {
            Ok(Ok(())) => {}
            Ok(Err(error)) => panic!(
                "original {id} failed {error:?}; retained {}",
                retained.display()
            ),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}
