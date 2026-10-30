// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    ServiceFinalizer, ServiceSpec,
    admission::Selection,
    envelope::{CliInvocationId, Settlement},
    environment::Environment,
    generation::{self, HostTool},
    invocation::{InvocationSpec, InvocationStore},
    managed::{
        payload::{self, PayloadCommand, PayloadPolicy},
        request::PrivateRequest,
        result::{GenericObserver, ManagedSummary, TerminalResult},
    },
    paths::CheckoutState,
};
use futures_util::FutureExt;
use std::{panic::AssertUnwindSafe, path::PathBuf, sync::Arc, time::Duration};

const ROOT: &str = "PSE_RECOVER_TEST_ROOT";
const SCRATCH: &str = "PSE_RECOVER_TEST_SCRATCH";
const ID: &str = "PSE_RECOVER_TEST_ID";
const INTENT: &str = "PSE_RECOVER_TEST_INTENT";
const ENTRY: &str = "managed::recovery::tests::crash_supervisor_entry";
const FINALIZER: &str = "managed::recovery::tests::recovering_finalizer_entry";

fn inputs() -> Option<(
    PathBuf,
    PathBuf,
    CliInvocationId,
    LaunchIntent,
    InvocationStore,
)> {
    let root = std::env::var_os(ROOT).map(PathBuf::from)?;
    let scratch = PathBuf::from(std::env::var_os(SCRATCH).unwrap());
    let id = CliInvocationId::try_from(std::env::var(ID).unwrap()).unwrap();
    let intent = serde_json::from_str(&std::env::var(INTENT).unwrap()).unwrap();
    let store = InvocationStore::new(
        CheckoutState::resolve(&root, &std::env::vars_os().collect()).unwrap(),
    );
    Some((root, scratch, id, intent, store))
}

#[tokio::test]
async fn crash_supervisor_entry() {
    let Some((root, _, id, intent, store)) = inputs() else {
        return;
    };
    let manager = HostManager::connect_user(None).await.unwrap();
    manager
        .authenticate_service_controller(&intent)
        .await
        .unwrap();
    let process = match pse_diagnostics::process_facts::ProcessReader::default()
        .process(std::process::id())
    {
        pse_diagnostics::process_facts::ProcessRead::Present(facts) => facts.identity,
        _ => panic!("original controller process unavailable"),
    };
    let directory = Arc::new(store.directory(&id).unwrap());
    directory
        .publish_evidence(
            InvocationEvidence::Controller,
            &ControllerRecord {
                version: 1,
                manager: manager.identity_key(),
                binding: manager.service_controller_binding(&intent).await.unwrap(),
                process,
            },
        )
        .unwrap();
    let request: PrivateRequest = directory.take_request().unwrap();
    let (command, selection, policy) = request.into_parts().unwrap();
    let trampoline = generation::prepared(&root)
        .unwrap()
        .unwrap()
        .executable(&root, HostTool::Host)
        .unwrap();
    let _report = payload::run(
        directory,
        intent,
        root,
        command,
        selection,
        trampoline,
        policy,
        tokio_util::sync::CancellationToken::new(),
        GenericObserver,
        None,
    )
    .await
    .unwrap();
    panic!("crash fixture must be killed by its original native owner");
}

#[tokio::test]
async fn recovering_finalizer_entry() {
    let Some((_, scratch, id, intent, store)) = inputs() else {
        return;
    };
    let manager = HostManager::connect_user(None).await.unwrap();
    manager
        .authenticate_service_finalizer(&intent)
        .await
        .unwrap();
    let directory = store.directory(&id).unwrap();
    let original: ControllerRecord = directory
        .evidence(InvocationEvidence::Controller)
        .unwrap()
        .unwrap();
    directory.try_launch_lock().unwrap().close().unwrap();
    let report = recover(
        &directory,
        &manager,
        &intent,
        &original,
        Instant::now() + Duration::from_secs(10),
    )
    .await
    .unwrap();
    let settled = report.resources_settled();
    let policy = RedactionPolicy::default();
    let (view, failures, _custody) = report.finish(&policy);
    std::fs::write(
        scratch.join("recovery.json"),
        serde_json::to_vec(&view).unwrap(),
    )
    .unwrap();
    std::fs::write(
        scratch.join("settled"),
        if settled {
            b"true".as_slice()
        } else {
            b"false".as_slice()
        },
    )
    .unwrap();
    let terminal = TerminalResult::finalizer_unknown(
        &id,
        "run",
        std::env::var("SERVICE_RESULT").unwrap(),
        failures,
        &policy,
    );
    directory.publish_result(&terminal).unwrap();
}

#[tokio::test]
async fn supervisor_crash_recovers_ordinary_and_finite_resources_without_inventing_reap_or_capture()
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let control = generation::prepared(&root).unwrap().unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    for (finite, wrong_binding) in [(false, false), (true, false), (false, true)] {
        let scratch = tempfile::tempdir().unwrap();
        let mut environment: Environment = std::env::vars_os().collect();
        environment.insert(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        );
        let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
        let id = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: "original supervisor crash recovery",
                control: control.clone(),
                environment_digest: "private-recovery-fixture".into(),
                scientific: Vec::new(),
            })
            .unwrap();
        let mut payload_environment: Environment = std::env::vars_os().collect();
        payload_environment.insert(
            "PSE_BUILD_MODE".into(),
            if finite { "bounded" } else { "unbounded" }.into(),
        );
        for key in [
            "PSE_HOST_ALLOCATION",
            "PSE_RESOURCE_CLASS",
            "PSE_MEMORY_MAX",
            "PSE_SLICE",
        ] {
            payload_environment.remove(std::ffi::OsStr::new(key));
        }
        let selection =
            Selection::load(&root, &payload_environment, finite.then_some("light"), None).unwrap();
        directory.publish_request(&PrivateRequest::new(PayloadCommand {
            program: "/bin/sh".into(),
            arguments: vec!["-c".into(), "setsid /bin/sh -c 'echo $$ > \"$1\"; exec /bin/sleep 60' sh \"$1\" & wait".into(),
                "fixture".into(), scratch.path().join("descendant").into_os_string()],
            cwd: root.clone(), environment: payload_environment,
        }, selection, PayloadPolicy { startup: Duration::from_secs(15), drain: Duration::from_secs(10), write: Duration::from_secs(2) }).unwrap()).unwrap();
        let intent = LaunchIntent::fresh().unwrap();
        let executable = std::env::current_exe().unwrap();
        let spec = ServiceSpec {
            intent: intent.clone(),
            executable: executable.clone(),
            arguments: vec!["--exact".into(), ENTRY.into(), "--nocapture".into()],
            environment: vec![
                format!("{ROOT}={}", root.to_str().unwrap()),
                format!("{SCRATCH}={}", scratch.path().to_str().unwrap()),
                format!("{ID}={id}"),
                format!("{INTENT}={}", serde_json::to_string(&intent).unwrap()),
                format!(
                    "XDG_STATE_HOME={}",
                    scratch.path().join("state").to_str().unwrap()
                ),
                format!(
                    "XDG_RUNTIME_DIR={}",
                    std::env::var("XDG_RUNTIME_DIR").unwrap()
                ),
            ],
        };
        let finalizer = ServiceFinalizer {
            executable,
            arguments: vec!["--exact".into(), FINALIZER.into(), "--nocapture".into()],
        };
        let mut owned = None;
        let checked = AssertUnwindSafe(async {
            owned = Some(manager.start_notify_service(&spec, &finalizer, Instant::now() + Duration::from_secs(20)).await?);
            let descendant = tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    match std::fs::read_to_string(scratch.path().join("descendant")) {
                        Ok(pid) => break pid.trim().parse::<u32>().unwrap(),
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => tokio::time::sleep(Duration::from_millis(5)).await,
                        Err(error) => panic!("original descendant receipt: {error}"),
                    }
                }
            }).await.map_err(|_| HostError::Deadline("original escaped descendant receipt".into()))?;
            let _: PayloadCustody = directory.evidence(InvocationEvidence::Custody)?.unwrap();
            if wrong_binding {
                let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(directory.path().join("custody.json"))?).unwrap();
                value["workload"]["group"] = serde_json::json!("/invalid/../replacement");
                std::fs::write(directory.path().join("custody.json"), serde_json::to_vec(&value).unwrap())?;
            }
            owned.as_mut().unwrap().kill_and_drain(Instant::now() + Duration::from_secs(15)).await?;
            let result = store.result::<String, ManagedSummary>(&id)?.unwrap();
            if result.child_exit.is_some() || result.settlement != Settlement::Unknown
                || result.payload.stdout.is_some() || result.payload.stderr.is_some() {
                return Err(HostError::Ownership("finalizer invented child reap or complete capture".into()));
            }
            let view: serde_json::Value = serde_json::from_slice(&std::fs::read(scratch.path().join("recovery.json"))?).unwrap();
            if wrong_binding {
                if view["workload"].get("Err").is_none() || view["operation"].get("Ok").is_none()
                    || result.failures.is_empty() {
                    return Err(HostError::Ownership(format!("failed workload identity hid independent operation drain: {view:?}")));
                }
            } else if std::fs::read(scratch.path().join("settled"))? != b"true" {
                return Err(HostError::Ownership(format!("original recovery incomplete: {view:?}")));
            }
            if matches!(pse_diagnostics::process_facts::ProcessReader::default().process(descendant),
                pse_diagnostics::process_facts::ProcessRead::Present(facts) if facts.stat.state != 'Z') {
                return Err(HostError::Ownership("escaped original descendant remained live after native recovery".into()));
            }
            Ok::<_, HostError>(())
        }).catch_unwind().await;
        let cleanup = async {
            let deadline = Instant::now() + Duration::from_secs(15);
            if owned.is_none() {
                owned = manager.reconcile_if_present(&intent, deadline).await?;
            }
            if let Some(owned) = &mut owned {
                if !owned.released() {
                    owned.stop_and_drain(deadline).await?;
                }
            }
            Ok::<_, HostError>(())
        }
        .await;
        if let Err(error) = cleanup {
            let retained = scratch.keep();
            panic!(
                "original recovery {id} {} cleanup {error:?}; retained {}",
                intent.unit(),
                retained.display()
            );
        }
        match checked {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let retained = scratch.keep();
                panic!(
                    "original recovery {id} {} failed {error:?}; retained {}",
                    intent.unit(),
                    retained.display()
                );
            }
            Err(panic) => {
                let retained = scratch.keep();
                eprintln!(
                    "original recovery {id} {} retained after panic: {}",
                    intent.unit(),
                    retained.display()
                );
                std::panic::resume_unwind(panic);
            }
        }
    }
}
