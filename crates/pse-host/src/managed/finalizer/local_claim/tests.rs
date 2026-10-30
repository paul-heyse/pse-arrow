// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    HostManager, LaunchIntent,
    envelope::Settlement,
    environment::Environment,
    generation::{self, GenerationUse},
    invocation::{InvocationSpec, InvocationStore},
    managed::{
        payload::{PayloadCommand, PayloadPolicy},
        request::PrivateRequest,
        supervisor::tests::{FRONTEND, FRONTEND_ENTRY, FRONTEND_ID, FRONTEND_ROOT},
    },
    paths::CheckoutState,
};
use futures_util::FutureExt;
use std::{collections::BTreeMap, ffi::OsString, panic::AssertUnwindSafe};

#[derive(Clone, Copy, Debug)]
enum Case {
    Normal,
    Crash,
    FailedPublication,
}

fn records(path: &std::path::Path) -> Result<BTreeMap<PathBuf, Vec<u8>>, HostError> {
    let mut records = BTreeMap::new();
    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
            && path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| uuid::Uuid::parse_str(stem).is_ok())
        {
            let bytes = std::fs::read(&path)?;
            records.insert(path, bytes);
        }
    }
    Ok(records)
}
fn locator_path(locator: &serde_json::Value) -> Result<PathBuf, HostError> {
    let tools: OsString =
        serde_json::from_value(locator["tools"].clone()).map_err(std::io::Error::other)?;
    let generation = locator["generation"]
        .as_str()
        .ok_or_else(|| HostError::Ownership("claim generation locator absent".into()))?;
    let record = locator["record"]
        .as_str()
        .ok_or_else(|| HostError::Ownership("claim record locator absent".into()))?;
    Ok(PathBuf::from(tools)
        .join(".uses")
        .join(generation)
        .join(format!("{record}.json")))
}

#[tokio::test]
async fn actual_finalizer_retires_only_own_context_and_retains_failed_publication() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    let receipts = root.join(".local/plan59-local-finalizer-claims");
    std::fs::create_dir_all(&receipts).unwrap();
    for case in [Case::Normal, Case::Crash, Case::FailedPublication] {
        let scratch = tempfile::Builder::new()
            .prefix(&format!("{case:?}-"))
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
        ] {
            environment.remove(std::ffi::OsStr::new(key));
        }
        let control = generation::prepared(&root).unwrap().unwrap();
        let uses = root
            .join("target/host-tools/.uses")
            .join(&control.generation);
        let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
        let id = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: "actual local finalizer context claim",
                control: control.clone(),
                environment_digest: "private-local-finalizer-fixture".into(),
                scientific: Vec::new(),
            })
            .unwrap();
        let selection = crate::admission::Selection::load(&root, &environment, None, None).unwrap();
        directory
            .publish_request(
                &PrivateRequest::new(
                    PayloadCommand {
                        program: "/bin/sh".into(),
                        arguments: vec![
                            "-c".into(),
                            "while [ ! -f \"$1\" ]; do /bin/sleep 0.01; done; exit 0".into(),
                            "fixture".into(),
                            scratch.path().join("release").into_os_string(),
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
            let fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
            launcher = Some(manager.start_scope(&intent, fd, &slice, &slice, Instant::now()+Duration::from_secs(10), Duration::from_secs(5)).await?);
            std::fs::write(scratch.path().join("go"), b"go")?;
            let status = tokio::time::timeout(Duration::from_secs(30), child.wait()).await.map_err(|_| HostError::Deadline("local finalizer frontend".into()))??;
            assert!(status.success());
            let original: supervisor::ControllerRecord = directory.evidence(InvocationEvidence::Controller)?.unwrap();
            let custody: crate::managed::payload::PayloadCustody = directory.evidence(InvocationEvidence::Custody)?.unwrap();
            let originals: BTreeMap<_, _> = records(&uses)?.into_iter().filter(|(_, bytes)| {
                let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                let owner = serde_json::from_value::<pse_diagnostics::process_facts::ProcessIdentity>(value["owner"].clone()).unwrap();
                owner == original.process || owner == custody.child
            }).collect();
            assert!(!originals.is_empty(), "original performing claim missing before finalization");
            let mut observed = supervisor::recorded(&root, &id, &environment).await?;
            match case {
                Case::Normal => { std::fs::write(scratch.path().join("release"), b"release")?; },
                Case::Crash | Case::FailedPublication => {
                    if matches!(case, Case::FailedPublication) { std::fs::write(directory.path().join("finalization.json"), b"occupied-by-fixture")?; }
                    let pid = rustix::process::Pid::from_raw(original.process.pid as i32).unwrap();
                    let fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
                    assert!(matches!(ProcessReader::default().process(original.process.pid), ProcessRead::Present(facts) if facts.identity == original.process));
                    rustix::process::pidfd_send_signal(&fd, rustix::process::Signal::KILL).map_err(std::io::Error::from)?;
                }
            }
            observed.wait(Some(Instant::now()+Duration::from_secs(30))).await?.ok_or_else(|| HostError::Deadline("local finalizer completion".into()))?;
            let result = store.result::<serde::de::IgnoredAny, ManagedSummary>(&id)?.unwrap();
            if matches!(case, Case::Normal) { assert_eq!(result.child_exit, Some(0)); assert_eq!(result.settlement, Settlement::Complete); }
            else { assert_eq!(result.child_exit, None); assert_eq!(result.settlement, Settlement::Unknown); }
            for (path, original_bytes) in originals { assert_eq!(std::fs::read(path)?, original_bytes, "local claim retirement changed an original performing claim"); }
            if matches!(case, Case::FailedPublication) {
                assert_eq!(std::fs::read(directory.path().join("finalization.json"))?, b"occupied-by-fixture");
                let bound: Vec<_> = records(&uses)?.into_iter().filter(|(_, bytes)| {
                    let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                    value.get("finalizer").filter(|scope| !scope.is_null()).is_some_and(|scope| {
                        serde_json::from_value::<OsString>(scope["directory"].clone()).unwrap() == directory.path().as_os_str()
                    })
                }).collect();
                assert_eq!(bound.len(), 1, "failed required publication lost its current finalizer claim");
                std::fs::write(scratch.path().join("retained-claim-path"), bound[0].0.as_os_str().as_encoded_bytes())?;
            } else {
                let finalization: serde_json::Value = directory.evidence(InvocationEvidence::Finalization)?.unwrap();
                assert_eq!(finalization["failures"], serde_json::json!([]));
                let locator = &finalization["current_finalizer_context_claim"];
                assert!(locator.is_object(), "prepared finalizer lacks actual local-claim consumer");
                assert!(!locator_path(locator)?.try_exists()?, "successful local finalization retained its own context claim");
            }
            assert_ne!(GenerationUse::inspect(&root.join("target/host-tools"), &control.generation)?, generation::GenerationUseState::Available);
            Ok::<_, HostError>(())
        }).catch_unwind().await;
        let mut failures = Vec::new();
        if directory
            .evidence::<supervisor::LaunchRecord>(InvocationEvidence::Launch)
            .ok()
            .flatten()
            .is_some()
        {
            match supervisor::recorded(&root, &id, &environment).await {
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
                .stop_and_drain(Instant::now() + Duration::from_secs(10))
                .await
        {
            failures.push(error);
        }
        let _ = child.start_kill();
        if let Err(error) = child.wait().await {
            failures.push(error.into());
        }
        let retained = scratch.keep();
        eprintln!(
            "local finalizer {case:?} {id} retained {}",
            retained.display()
        );
        assert!(
            failures.is_empty(),
            "original cleanup {failures:?}; retained {}",
            retained.display()
        );
        match checked {
            Ok(Ok(())) => {}
            Ok(Err(error)) => panic!(
                "{case:?} {id} failed {error:?}; retained {}",
                retained.display()
            ),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
}
