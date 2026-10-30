// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{invocation::InvocationSpec, managed::result::ManagedSummary};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;
const PROBE: &str = "managed::supervisor::ancestry::tests::nested_member_probe";
const FIXTURE: &str = "PSE_NESTED_FIXTURE";
const DESCENDANT: &str =
    "managed::supervisor::ancestry::tests::descendant_cannot_adopt_payload_leader";

#[tokio::test]
async fn descendant_cannot_adopt_payload_leader() {
    if std::env::var_os(FIXTURE).is_none() {
        return;
    }
    let environment = std::env::vars_os().collect();
    assert!(
        current_payload(&environment).await.is_err(),
        "a descendant acquired original leader authority from inherited hints"
    );
}

#[tokio::test]
async fn managed_hint_never_supplies_actual_native_membership() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let mut environment: Environment = std::env::vars_os().collect();
    environment.insert("PSE_BUILD_MODE".into(), "unbounded".into());
    environment.insert(
        "PSE_MANAGED_INVOCATION".into(),
        CliInvocationId::new().to_string().into(),
    );
    environment.insert(CHECKOUT.into(), request::encode_path(&root).into());
    environment.insert(
        STATE_BASE.into(),
        request::encode_path(PathBuf::from("/missing-original-ledger").as_path()).into(),
    );
    let selection = Selection::load(&root, &environment, None, None).unwrap();
    assert!(
        parent(
            &environment,
            &selection,
            Instant::now() + Duration::from_secs(5)
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn nested_member_probe() {
    let Some(scratch) = std::env::var_os(FIXTURE).map(PathBuf::from) else {
        return;
    };
    let root = request::decode_path(&std::env::var(CHECKOUT).unwrap()).unwrap();
    let mut environment: Environment = std::env::vars_os().collect();
    let leader = current_payload(&environment).await.unwrap();
    assert_eq!(leader.root(), root);
    assert_eq!(
        leader.invocation_id().as_str(),
        std::env::var("PSE_MANAGED_INVOCATION").unwrap()
    );
    let metadata = leader
        .metadata::<Box<serde_json::value::RawValue>>()
        .unwrap();
    assert_eq!(&metadata.selection.invocation_id, leader.invocation_id());
    assert_eq!(leader.directory().id(), leader.invocation_id());
    let mut descendant = tokio::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", DESCENDANT, "--nocapture"])
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let observed = tokio::time::timeout(Duration::from_secs(10), descendant.wait()).await;
    let killed = descendant.start_kill();
    let reaped = descendant.wait().await;
    assert!(
        matches!(observed, Ok(Ok(status)) if status.success()),
        "descendant refusal: {observed:?}; cleanup {killed:?}/{reaped:?}"
    );
    assert!(reaped.is_ok(), "{reaped:?}");
    let selection = Selection::load(&root, &environment, None, None).unwrap();
    let parent = parent(
        &environment,
        &selection,
        Instant::now() + Duration::from_secs(10),
    )
    .await
    .unwrap()
    .unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    let level = std::env::var("PSE_NESTED_LEVEL").unwrap();
    assert_eq!(parent.ancestors.len(), if level == "B" { 1 } else { 2 });
    assert_eq!(
        parent.permit.is_some(),
        std::env::var("PSE_NESTED_FINITE").unwrap() == "true"
    );
    let original_id = parent.invocation_id().clone();
    let root_custody: payload::PayloadCustody = parent.ancestors[0]
        .directory
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
                parent: Some(original_id),
                operation: "actual nested member fixture",
                control: crate::generation::prepared(&root).unwrap().unwrap(),
                environment_digest: "nested-private-fixture".into(),
                scientific: Vec::new(),
            })
            .unwrap(),
    );
    if level == "C" {
        assert!(!parent.ancestors[1].directory.launch_closed().unwrap());
        parent.ancestors[0]
            .directory
            .try_launch_lock()
            .unwrap()
            .close()
            .unwrap();
        assert!(
            parent
                .locks(&manager, Instant::now() + Duration::from_secs(5))
                .await
                .is_err()
        );
    }
    environment.insert("PSE_NESTED_LEVEL".into(), "C".into());
    let launched = launch_generic(ManagedRequest {
        root: root.clone(),
        directory: directory.clone(),
        command: PayloadCommand {
            program: std::env::current_exe().unwrap(),
            arguments: vec!["--exact".into(), PROBE.into(), "--nocapture".into()],
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
    .await;
    if level == "C" {
        assert!(
            launched.is_err(),
            "closed grandparent admitted another external service"
        );
        let intent: LaunchRecord = directory
            .evidence(InvocationEvidence::Launch)
            .unwrap()
            .unwrap();
        assert!(
            matches!(manager.borrow(intent.controller.unit()).observe().await,
            Err(HostError::Manager(zbus::Error::MethodError(name, _, _)))
            if name.as_str() == "org.freedesktop.systemd1.NoSuchUnit")
        );
        assert!(
            directory
                .evidence::<serde_json::Value>(InvocationEvidence::Custody)
                .unwrap()
                .is_none()
        );
        assert!(!directory.path().join("request.json").exists());
        std::fs::write(scratch.join("grandparent-refused"), id.as_str()).unwrap();
        return;
    }
    let mut launched = launched.unwrap();
    let checked = AssertUnwindSafe(async {
        launched
            .wait(Some(Instant::now() + Duration::from_secs(30)))
            .await
            .unwrap()
            .unwrap();
        let result = store
            .result::<serde::de::IgnoredAny, ManagedSummary>(&id)
            .unwrap()
            .unwrap();
        assert_eq!(result.child_exit, Some(0), "{result:?}");
        assert_eq!(
            result.settlement,
            crate::envelope::Settlement::Complete,
            "{result:?}"
        );
        assert_eq!(
            result.payload.borrowed_reservation,
            std::env::var("PSE_NESTED_FINITE").unwrap() == "true"
        );
        let finalization: serde_json::Value = directory
            .evidence(InvocationEvidence::Finalization)
            .unwrap()
            .unwrap();
        assert_eq!(finalization["failures"], serde_json::json!([]));
        let custody: payload::PayloadCustody = directory
            .evidence(InvocationEvidence::Custody)
            .unwrap()
            .unwrap();
        assert_eq!(custody.version, 3);
        assert!(!custody.subgroup.unwrap().path().exists());
        let current = manager.current_membership().await.unwrap();
        assert_eq!(current.root().active_state, "active");
        if let Some(reservation) = root_custody.reservation {
            let record = serde_json::to_value(reservation).unwrap();
            let bytes: Vec<u8> = serde_json::from_value(record["directory"].clone()).unwrap();
            let journal: serde_json::Value = serde_json::from_slice(
                &std::fs::read(PathBuf::from(OsString::from_vec(bytes)).join("finite.json"))
                    .unwrap(),
            )
            .unwrap();
            let nonce = record["nonce"].as_str().unwrap();
            assert_eq!(
                journal["finite"][nonce]["released"],
                serde_json::json!(false),
                "nested owner retired its live parent's reservation"
            );
        }
        assert!(scratch.join("grandparent-refused").is_file());
        std::fs::write(scratch.join("nested-complete"), id.as_str()).unwrap();
    })
    .catch_unwind()
    .await;
    let cleanup = launched
        .cancel(Instant::now() + Duration::from_secs(10))
        .await;
    if let Err(panic) = checked {
        eprintln!(
            "nested original {id} cleanup={cleanup:?}; retained {}",
            scratch.display()
        );
        std::panic::resume_unwind(panic);
    }
    assert!(cleanup.is_ok(), "{cleanup:?}");
}

#[tokio::test]
async fn actual_nested_ordinary_and_finite_members_refuse_a_closed_grandparent() {
    use super::super::tests::{FRONTEND, FRONTEND_ENTRY, FRONTEND_ID, FRONTEND_ROOT};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    for finite in [false, true] {
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
            "PSE_MANAGED_INVOCATION",
            CHECKOUT,
            STATE_BASE,
        ] {
            environment.remove(std::ffi::OsStr::new(key));
        }
        environment.insert(FIXTURE.into(), scratch.path().as_os_str().to_owned());
        environment.insert("PSE_NESTED_LEVEL".into(), "B".into());
        environment.insert("PSE_NESTED_FINITE".into(), finite.to_string().into());
        let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
        let id = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: "nested native original-member journey",
                control: crate::generation::prepared(&root).unwrap().unwrap(),
                environment_digest: "nested-private-fixture".into(),
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
                        arguments: vec!["--exact".into(), PROBE.into(), "--nocapture".into()],
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
                .map_err(|_| HostError::Deadline("nested original frontend".into()))??;
            assert!(
                status.success(),
                "original nested frontend failed: {status}"
            );
            let mut original = recorded(&root, &id, &environment).await?;
            original
                .wait(Some(Instant::now() + Duration::from_secs(40)))
                .await?
                .ok_or_else(|| HostError::Deadline("nested original service completion".into()))?;
            let result = store
                .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
                .unwrap();
            assert_eq!(result.child_exit, Some(0), "{result:?}");
            assert_eq!(
                result.settlement,
                crate::envelope::Settlement::Complete,
                "{result:?}"
            );
            assert!(scratch.path().join("nested-complete").is_file());
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
        if !failures.is_empty() {
            let retained = scratch.keep();
            panic!(
                "nested original {id} cleanup {failures:?}; retained {}",
                retained.display()
            );
        }
        match checked {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let retained = scratch.keep();
                panic!(
                    "nested original {id} failed {error:?}; retained {}",
                    retained.display()
                );
            }
            Err(panic) => {
                let retained = scratch.keep();
                eprintln!("nested original {id} retained {}", retained.display());
                std::panic::resume_unwind(panic);
            }
        }
    }
}
