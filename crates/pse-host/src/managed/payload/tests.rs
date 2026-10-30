// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    ServiceFinalizer, ServiceSpec,
    envelope::{CliInvocationId, RedactionPolicy, Settlement},
    generation::{self, HostTool},
    invocation::{InvocationSpec, InvocationStore},
    paths::CheckoutState,
};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;

const ROOT: &str = "PSE_SERVICE_PAYLOAD_TEST_ROOT";
const SCRATCH: &str = "PSE_SERVICE_PAYLOAD_TEST_SCRATCH";
const ID: &str = "PSE_SERVICE_PAYLOAD_TEST_ID";
const INTENT: &str = "PSE_SERVICE_PAYLOAD_TEST_INTENT";
const FINITE: &str = "PSE_SERVICE_PAYLOAD_TEST_FINITE";
const ENTRY: &str = "managed::payload::tests::service_payload_entry";
const FINALIZER: &str = "managed::payload::tests::service_finalizer_entry";

#[tokio::test]
async fn service_payload_entry() {
    let Some(root) = std::env::var_os(ROOT).map(PathBuf::from) else {
        return;
    };
    let environment: Environment = std::env::vars_os().collect();
    let store = InvocationStore::new(CheckoutState::resolve(&root, &environment).unwrap());
    let id = CliInvocationId::try_from(std::env::var(ID).unwrap()).unwrap();
    let directory = Arc::new(store.directory(&id).unwrap());
    let request: crate::managed::request::PrivateRequest = directory.take_request().unwrap();
    let (command, selection, policy) = request.into_parts().unwrap();
    let controller: LaunchIntent = serde_json::from_str(&std::env::var(INTENT).unwrap()).unwrap();
    let trampoline = generation::prepared(&root)
        .unwrap()
        .unwrap()
        .executable(&root, HostTool::Host)
        .unwrap();
    let report = run(
        directory.clone(),
        controller,
        root,
        command,
        selection,
        trampoline,
        policy,
        CancellationToken::new(),
        crate::managed::result::GenericObserver,
        None,
    )
    .await
    .unwrap();
    assert!(report.operation_error.is_none(), "{report:?}");
    assert!(report.resources_settled(), "{report:?}");
    assert!(report.capture.as_ref().unwrap().log_complete());
    assert_eq!(report.child_exit(), Some(3), "{report:?}");
    let terminal = crate::managed::result::TerminalResult::from_payload(
        &id,
        "run",
        report,
        &RedactionPolicy::default(),
    );
    assert_eq!(terminal.response().settlement, Settlement::Complete);
    // The ledger publication owner takes the opaque terminal, never a caller-created envelope.
    directory.publish_result(&terminal).unwrap();
}

#[test]
fn service_finalizer_entry() {
    let Some(scratch) = std::env::var_os(SCRATCH).map(PathBuf::from) else {
        return;
    };
    std::fs::write(
        scratch.join("finalizer"),
        std::env::var("INVOCATION_ID").unwrap(),
    )
    .unwrap();
}

#[tokio::test]
async fn actual_notify_service_places_captures_and_releases_ordinary_and_finite_payloads() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let current = generation::prepared(&root).unwrap().unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    for finite in [false, true] {
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
                operation: "native generic payload test",
                control: current.clone(),
                environment_digest: "test-empty-secret-selection".into(),
                scientific: Vec::new(),
            })
            .unwrap();
        let mut payload_environment: Environment = std::env::vars_os().collect();
        payload_environment.insert(
            "PSE_BUILD_MODE".into(),
            if finite { "bounded" } else { "unbounded" }.into(),
        );
        for name in [
            "PSE_RESOURCE_CLASS",
            "PSE_MEMORY_MAX",
            "PSE_HOST_ALLOCATION",
            "PSE_SLICE",
        ] {
            payload_environment.remove(std::ffi::OsStr::new(name));
        }
        let selection =
            Selection::load(&root, &payload_environment, finite.then_some("light"), None).unwrap();
        directory
            .publish_request(
                &crate::managed::request::PrivateRequest::new(
                    PayloadCommand {
                        program: "/bin/sh".into(),
                        arguments: vec![
                            "-c".into(),
                            "printf 'stdout'; printf 'stderr' >&2; exit 3".into(),
                        ],
                        cwd: root.clone(),
                        environment: payload_environment,
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
                format!("{FINITE}={finite}"),
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
            owned = Some(
                manager
                    .start_notify_service(
                        &spec,
                        &finalizer,
                        Instant::now() + Duration::from_secs(20),
                    )
                    .await?,
            );
            if owned.as_ref().and_then(OwnedUnit::launch_job_result) != Some("done") {
                return Err(HostError::Ownership(
                    "payload controller did not acknowledge readiness".into(),
                ));
            }
            let result = tokio::time::timeout(Duration::from_secs(15), async {
                loop {
                    if let Some(result) =
                        store.result::<String, crate::managed::result::ManagedSummary>(&id)?
                    {
                        return Ok::<_, HostError>(result);
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .map_err(|_| HostError::Deadline("native payload terminal publication".into()))??;
            if result.child_exit != Some(3) || result.settlement != Settlement::Complete {
                return Err(HostError::Ownership(format!(
                    "native payload owner result differs: {result:?}"
                )));
            }
            if std::fs::read(directory.path().join("stdout.raw"))? != b"stdout"
                || std::fs::read(directory.path().join("stderr.raw"))? != b"stderr"
            {
                return Err(HostError::Ownership(
                    "native original stream bytes differ".into(),
                ));
            }
            Ok::<_, HostError>(())
        })
        .catch_unwind()
        .await;
        let cleanup = async {
            let deadline = Instant::now() + Duration::from_secs(10);
            if owned.is_none() {
                owned = manager.reconcile_if_present(&intent, deadline).await?;
            }
            if let Some(owned) = &mut owned {
                owned.stop_and_drain(deadline).await?;
            }
            Ok::<_, HostError>(())
        }
        .await;
        if let Err(error) = cleanup {
            let retained = scratch.keep();
            panic!(
                "native payload {} cleanup failed {error:?}; retained {}",
                intent.unit(),
                retained.display()
            );
        }
        match checked {
            Ok(result) => result.unwrap(),
            Err(panic) => std::panic::resume_unwind(panic),
        }
        assert!(scratch.path().join("finalizer").exists());
    }
}
