// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use futures_util::FutureExt;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;

const CEILING: u64 = 256 * 1024 * 1024;
struct Fixture {
    manager: HostManager,
    root: OwnedUnit,
    keeper: tokio::process::Child,
}
async fn fixture() -> Fixture {
    fixture_bound(true).await
}
async fn fixture_bound(native_parent: bool) -> Fixture {
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    let mut keeper = tokio::process::Command::new("/bin/sleep")
        .arg("60")
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let pid = rustix::process::Pid::from_raw(keeper.id().unwrap() as i32).unwrap();
    let fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
    let intent = LaunchIntent::fresh_scope().unwrap();
    let result = manager
        .start_scope_delegated(
            &intent,
            fd,
            if native_parent { &current.unit } else { &slice },
            &slice,
            Instant::now() + Duration::from_secs(15),
            Duration::from_secs(5),
            &ResourceControls {
                memory_max: Some(CEILING),
                memory_swap_max: Some(0),
                ..Default::default()
            },
        )
        .await;
    match result {
        Ok(root) => Fixture {
            manager,
            root,
            keeper,
        },
        Err(error) => {
            let original = manager
                .reconcile_if_present(&intent, Instant::now() + Duration::from_secs(10))
                .await;
            let cleanup = match original {
                Ok(Some(mut root)) => root
                    .stop_and_drain(Instant::now() + Duration::from_secs(10))
                    .await
                    .map(|_| ()),
                Ok(None) => Ok(()),
                Err(error) => Err(error),
            };
            let kill = keeper.start_kill();
            let reap = keeper.wait().await;
            panic!("fixture launch {error:?}; cleanup {cleanup:?}; kill {kill:?}; reap {reap:?}");
        }
    }
}
async fn create(parent: &PayloadParent, name: &str) -> OwnedSubgroup {
    let intent = parent.creation_intent(name).unwrap();
    match parent.create(&intent).await {
        CreationOutcome::Created(group) => group,
        CreationOutcome::NoEffect(cause) => panic!("creation refused before effect: {cause:?}"),
        CreationOutcome::CreatedButUnbound {
            intent,
            opened,
            cause,
        } => panic!(
            "creation committed but unbound: {intent:?}; opened={:?}; cause={cause:?}",
            opened.as_ref().map(OpenedSubgroupCustody::binding)
        ),
    }
}
fn gated(
    command: &[OsString],
    environment: &[(String, String)],
) -> (tokio::process::Child, tokio::process::ChildStdin) {
    let mut process = tokio::process::Command::new("/bin/sh");
    process.args([
        "-c",
        "read -r gate || exit 125; [ \"$gate\" = 1 ] || exit 125; exec \"$@\"",
        "gate",
    ]);
    process
        .args(command)
        .envs(environment.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::piped())
        .kill_on_drop(true);
    let mut child = process.spawn().unwrap();
    let gate = child.stdin.take().unwrap();
    (child, gate)
}
async fn cleanup(
    fixture: &mut Fixture,
    groups: &mut [OwnedSubgroup],
    children: &mut [tokio::process::Child],
) -> Vec<String> {
    let mut failures = Vec::new();
    let results=futures_util::future::join_all(groups.iter_mut().filter(|group|!group.removed).map(|group|async move {
        let kill=group.kill().await;let drain=group.wait_empty(Instant::now()+Duration::from_secs(10)).await;let remove=group.remove_tree(Instant::now()+Duration::from_secs(10)).await;
        if kill.is_err() || drain.is_err() || !matches!(&remove,Ok(receipt) if receipt.original_removed && receipt.failures.is_empty()) {Some(format!("{} kill={kill:?} drain={drain:?} remove={remove:?}",group.path().display()))}else{None}
    })).await;
    failures.extend(results.into_iter().flatten());
    let child_cleanup =
        futures_util::future::join_all(children.iter_mut().map(|child| async move {
            let kill = child.start_kill();
            let reap = child.wait().await;
            if reap.is_err() {
                Some(format!("child kill={kill:?} reap={reap:?}"))
            } else {
                None
            }
        }));
    let root_cleanup = fixture
        .root
        .stop_and_drain(Instant::now() + Duration::from_secs(10));
    let (children, root) = tokio::join!(child_cleanup, root_cleanup);
    failures.extend(children.into_iter().flatten());
    if let Err(error) = root {
        failures.push(format!("root {error:?}"));
    }
    let kill = fixture.keeper.start_kill();
    if let Err(error) = fixture.keeper.wait().await {
        failures.push(format!("keeper kill={kill:?} reap={error:?}"));
    }
    failures
}
async fn until_file(path: &Path, deadline: Instant) {
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "marker missing: {}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

/// This fixture is executed directly by the native parent after its shell gate is attached.
#[tokio::test]
async fn member_probe() {
    let Ok(encoded) = std::env::var("PSE_SUBGROUP_PROBE") else {
        return;
    };
    let manager = HostManager::connect_user(None).await.unwrap();
    let membership = manager.current_membership().await.unwrap();
    let expected = PathBuf::from(std::env::var_os("PSE_SUBGROUP_MEMBER").unwrap());
    assert_eq!(membership.member(), expected);
    assert_ne!(
        membership.member(),
        Group::path(&membership.root.control_group).unwrap()
    );
    assert_eq!(
        manager.current_unit().await.unwrap().control_group,
        membership.root.control_group
    );
    membership
        .require_memory_ceiling(&manager, CEILING)
        .unwrap();
    assert!(
        membership
            .require_memory_ceiling(&manager, CEILING + 1)
            .is_err()
    );
    assert!(
        membership
            .ancestry
            .last()
            .unwrap()
            .control("memory.max")
            .unwrap()
            .is_none()
    );
    let lent = membership.borrow_payload_parent(&manager).await.unwrap();
    assert_eq!(lent.ancestry.last().unwrap().path, expected);
    let separate_controller = HostManager::connect_user(None).await.unwrap();
    let lent = separate_controller
        .recover_payload_parent(lent.binding())
        .await
        .unwrap();
    assert_eq!(lent.ancestry.last().unwrap().path, expected);
    let binding: PayloadParentBinding = serde_json::from_str(&encoded).unwrap();
    let original = manager.recover_payload_parent(&binding).await.unwrap();
    let parent = original.at_current_member(&membership).await.unwrap();
    assert_eq!(parent.ancestry.last().unwrap().path, expected);
    // A disposable process-owned allocation proves both members charge the same finite root.
    let allocation = vec![0x5a_u8; 32 * 1024 * 1024].into_boxed_slice();
    let allocation = Box::leak(allocation);
    std::hint::black_box(allocation);
    if std::env::var("PSE_SUBGROUP_MODE").as_deref() == Ok("sibling") {
        std::fs::write(std::env::var_os("PSE_SUBGROUP_READY").unwrap(), b"ready").unwrap();
        std::future::pending::<()>().await;
    }
    let nested = create(&parent, "nested").await;
    let marker = PathBuf::from(std::env::var_os("PSE_SUBGROUP_ESCAPED").unwrap());
    let command = vec![
        OsString::from("/bin/sh"),
        OsString::from("-c"),
        OsString::from(
            "setsid /bin/sh -c 'echo $$ > \"$1\"; exec /bin/sleep 60' escaped \"$1\" & wait",
        ),
        OsString::from("nested"),
        marker.clone().into_os_string(),
    ];
    let (mut child, mut gate) = gated(&command, &[]);
    let attachment = nested.attach(&mut child).await.unwrap();
    assert_eq!(attachment.process.pid, child.id().unwrap());
    require_live(&attachment.pidfd).unwrap();
    nested
        .validate_attachment(&attachment, &child)
        .await
        .unwrap();
    gate.write_all(b"1\n").await.unwrap();
    drop(gate);
    until_file(&marker, Instant::now() + Duration::from_secs(10)).await;
    std::fs::write(std::env::var_os("PSE_SUBGROUP_READY").unwrap(), b"ready").unwrap();
    std::future::pending::<()>().await;
}

#[tokio::test]
async fn unrelated_delegated_scope_cannot_lend_current_member_authority() {
    let mut fixture = fixture_bound(false).await;
    let mut groups = Vec::new();
    let mut children = Vec::new();
    let body = std::panic::AssertUnwindSafe(async {
        let parent = fixture.root.payload_parent().await.unwrap();
        groups.push(create(&parent, "unrelated-member").await);
        let command = vec![
            std::env::current_exe().unwrap().into_os_string(),
            "--exact".into(),
            "manager::subgroup::payload::tests::unrelated_member_probe".into(),
            "--nocapture".into(),
        ];
        let (child, mut gate) = gated(&command, &[("PSE_UNRELATED_MEMBER".into(), "probe".into())]);
        children.push(child);
        let attached = groups[0].attach(&mut children[0]).await.unwrap();
        groups[0]
            .validate_attachment(&attached, &children[0])
            .await
            .unwrap();
        gate.write_all(b"1\n").await.unwrap();
        drop(gate);
        let status = tokio::time::timeout(Duration::from_secs(10), children[0].wait())
            .await
            .unwrap()
            .unwrap();
        assert!(status.success(), "unrelated member refusal probe: {status}");
    })
    .catch_unwind()
    .await;
    let failures = cleanup(&mut fixture, &mut groups, &mut children).await;
    if let Err(panic) = body {
        eprintln!("owned cleanup failures: {failures:?}");
        std::panic::resume_unwind(panic);
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[tokio::test]
async fn unrelated_member_probe() {
    if std::env::var_os("PSE_UNRELATED_MEMBER").is_none() {
        return;
    }
    let manager = HostManager::connect_user(None).await.unwrap();
    let membership = manager.current_membership().await.unwrap();
    assert!(!manager.native_workload(&membership.root).await.unwrap());
    assert!(membership.borrow_payload_parent(&manager).await.is_err());
}

#[tokio::test]
async fn native_cross_group_attach_nested_drain_sibling_isolation_and_replacement_refusal() {
    let mut fixture = fixture().await;
    let mut groups = Vec::new();
    let mut children = Vec::new();
    let scratch = tempfile::tempdir().unwrap();
    let ready = scratch.path().join("ready");
    let sibling_ready = scratch.path().join("sibling-ready");
    let escaped = scratch.path().join("escaped");
    let body = std::panic::AssertUnwindSafe(async {
        let parent = fixture.root.payload_parent().await.unwrap();
        groups.push(create(&parent, "invocation-a").await);
        groups.push(create(&parent, "sibling").await);
        let original_intent = parent.creation_intent("invocation-a").unwrap();
        let original_binding = groups[0].binding();
        let recovered = parent
            .recover_created(&original_intent, &original_binding)
            .await
            .unwrap();
        assert_eq!(recovered.binding(), original_binding);
        drop(recovered);
        let original_member = groups[0].as_parent().await.unwrap().binding().clone();
        assert!(groups[0].population_empty().unwrap());
        assert!(groups[1].population_empty().unwrap());
        assert!(groups[0].group.control("memory.max").unwrap().is_none());
        let environment = vec![
            (
                "PSE_SUBGROUP_PROBE".into(),
                serde_json::to_string(parent.binding()).unwrap(),
            ),
            (
                "PSE_SUBGROUP_MEMBER".into(),
                groups[0].path().to_str().unwrap().into(),
            ),
            ("PSE_SUBGROUP_READY".into(), ready.to_str().unwrap().into()),
            (
                "PSE_SUBGROUP_ESCAPED".into(),
                escaped.to_str().unwrap().into(),
            ),
        ];
        let command = vec![
            std::env::current_exe().unwrap().into_os_string(),
            OsString::from("--exact"),
            OsString::from("manager::subgroup::payload::tests::member_probe"),
            OsString::from("--nocapture"),
        ];
        let (child, mut gate) = gated(&command, &environment);
        children.push(child);
        let attached = groups[0].attach(&mut children[0]).await.unwrap();
        require_live(&attached.pidfd).unwrap();
        assert_eq!(attached.process.pid, children[0].id().unwrap());
        assert!(
            !ready.exists(),
            "attachment must never release the payload gate"
        );
        groups[0]
            .validate_attachment(&attached, &children[0])
            .await
            .unwrap();
        gate.write_all(b"1\n").await.unwrap();
        drop(gate);
        let mut sibling_environment = environment.clone();
        sibling_environment
            .retain(|(key, _)| key != "PSE_SUBGROUP_MEMBER" && key != "PSE_SUBGROUP_READY");
        sibling_environment.extend([
            (
                "PSE_SUBGROUP_MEMBER".into(),
                groups[1].path().to_str().unwrap().into(),
            ),
            (
                "PSE_SUBGROUP_READY".into(),
                sibling_ready.to_str().unwrap().into(),
            ),
            ("PSE_SUBGROUP_MODE".into(), "sibling".into()),
        ]);
        let (sibling, mut sibling_gate) = gated(&command, &sibling_environment);
        children.push(sibling);
        let sibling_identity = groups[1].attach(&mut children[1]).await.unwrap();
        sibling_gate.write_all(b"1\n").await.unwrap();
        drop(sibling_gate);
        until_file(&ready, Instant::now() + Duration::from_secs(15)).await;
        until_file(&sibling_ready, Instant::now() + Duration::from_secs(15)).await;
        let escaped_pid = std::fs::read_to_string(&escaped)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap();
        assert_eq!(
            Group::path(&process_group(escaped_pid).unwrap()).unwrap(),
            groups[0].path().join("nested")
        );
        assert!(!groups[0].population_empty().unwrap());
        assert_eq!(fixture.root.group.limit("memory.max").unwrap(), CEILING);
        assert_eq!(fixture.root.group.limit("memory.swap.max").unwrap(), 0);
        let aggregate = fixture
            .root
            .group
            .control("memory.current")
            .unwrap()
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        assert!(
            aggregate >= 64 * 1024 * 1024,
            "both32MiB child allocations must be charged to the original finite root: {aggregate}"
        );
        groups[0].kill().await.unwrap();
        groups[0]
            .wait_empty(Instant::now() + Duration::from_secs(10))
            .await
            .unwrap();
        let removal = groups[0]
            .remove_tree(Instant::now() + Duration::from_secs(10))
            .await
            .unwrap();
        assert!(removal.original_removed);
        assert!(removal.failures.is_empty(), "{removal:?}");
        assert_eq!(removal.removed_directories, 2);
        assert_ne!(removal.original_inode, 0);
        assert_ne!(removal.original_device, 0);
        assert!(!groups[0].path().exists());
        require_live(&sibling_identity.pidfd).unwrap();
        assert_eq!(
            child_identity(children[1].id().unwrap(), &fixture.manager).unwrap(),
            sibling_identity.process
        );
        assert_eq!(fixture.root.group.limit("memory.max").unwrap(), CEILING);
        groups.push(create(&parent, "invocation-a").await);
        assert!(
            parent
                .recover_created(&original_intent, &original_binding)
                .await
                .is_err()
        );
        assert!(
            fixture
                .manager
                .recover_payload_parent(&original_member)
                .await
                .is_err()
        );
        let (late, late_gate) = gated(&["/bin/sleep".into(), "60".into()], &[]);
        children.push(late);
        let source = process_group(children[2].id().unwrap()).unwrap();
        assert!(groups[0].attach(&mut children[2]).await.is_err());
        assert!(groups[0].kill().await.is_err());
        assert_eq!(process_group(children[2].id().unwrap()).unwrap(), source);
        assert!(groups[2].population_empty().unwrap());
        assert!(
            groups[0]
                .remove_tree(Instant::now() + Duration::from_secs(5))
                .await
                .is_err()
        );
        drop(late_gate);
    })
    .catch_unwind()
    .await;
    let failures = cleanup(&mut fixture, &mut groups, &mut children).await;
    if let Err(panic) = body {
        eprintln!("owned cleanup failures: {failures:?}");
        std::panic::resume_unwind(panic);
    }
    assert!(failures.is_empty(), "{failures:#?}");
    assert!(fixture.root.released());
}

#[tokio::test]
async fn duplicate_and_forged_creation_intents_refuse_without_changing_existing_groups() {
    let mut fixture = fixture().await;
    let mut groups = Vec::new();
    let body=std::panic::AssertUnwindSafe(async {
        let parent=fixture.root.payload_parent().await.unwrap();groups.push(create(&parent,"existing").await);
        let intent=parent.creation_intent("existing").unwrap();
        assert!(matches!(parent.create(&intent).await,CreationOutcome::NoEffect(HostError::Io(error)) if error.kind()==io::ErrorKind::AlreadyExists));
        let mut forged=parent.creation_intent("forged").unwrap();forged.parent.ancestry[0].inode^=1;
        assert!(matches!(parent.create(&forged).await,CreationOutcome::NoEffect(HostError::Ownership(_))));
        assert!(!fixture.root.group.path.join("forged").exists());assert!(groups[0].population_empty().unwrap());
        for name in ["",".","..","a/b","a//b","/root","a.b"] {assert!(parent.creation_intent(name).is_err());}
    }).catch_unwind().await;
    let failures = cleanup(&mut fixture, &mut groups, &mut []).await;
    if let Err(panic) = body {
        eprintln!("owned cleanup failures: {failures:?}");
        std::panic::resume_unwind(panic);
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
