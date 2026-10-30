// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[tokio::test]
async fn current_membership_preserves_the_manager_root_and_actual_member() {
    let manager = HostManager::connect_user(None).await.unwrap();
    let membership = manager.current_membership().await.unwrap();
    assert_eq!(
        membership.member(),
        Group::path(&process_group(std::process::id()).unwrap()).unwrap()
    );
    assert_eq!(membership.root, manager.current_unit().await.unwrap());
    membership.validate(&manager).await.unwrap();
}

#[tokio::test]
async fn service_controller_authentication_refuses_foreign_and_nonservice_intents() {
    let manager = HostManager::connect_user(None).await.unwrap();
    assert!(
        manager
            .authenticate_service_controller(&LaunchIntent::fresh().unwrap())
            .await
            .is_err()
    );
    assert!(
        manager
            .authenticate_service_controller(&LaunchIntent::fresh_scope().unwrap())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn explicit_core_delegated_scope_keeps_its_original_finite_controls() {
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    let mut child = tokio::process::Command::new("/bin/sleep")
        .arg("60")
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let pid = rustix::process::Pid::from_raw(child.id().unwrap() as i32).unwrap();
    let pidfd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
    let intent = LaunchIntent::fresh_scope().unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let launch = manager
        .start_scope_delegated(
            &intent,
            pidfd,
            &current.unit,
            &slice,
            deadline,
            Duration::from_secs(5),
            &ResourceControls {
                memory_max: Some(256 * 1024 * 1024),
                memory_swap_max: Some(0),
                ..Default::default()
            },
        )
        .await;
    let checked = async {
        let owned = launch
            .as_ref()
            .map_err(|error| HostError::Ownership(format!("test launch: {error}")))?;
        let observed = owned.observe().await?;
        assert_eq!(delegation(&manager, &observed).await?, (true, vec![]));
        assert_eq!(
            owned
                .group
                .control("cgroup.subtree_control")?
                .unwrap()
                .trim(),
            ""
        );
        assert_eq!(owned.group.limit("memory.max")?, 256 * 1024 * 1024);
        assert_eq!(owned.group.limit("memory.swap.max")?, 0);
        assert_eq!(process_group(child.id().unwrap())?, observed.control_group);
        Ok::<_, HostError>(())
    }
    .await;
    let cleanup = match launch {
        Ok(mut owned) => owned
            .stop_and_drain(Instant::now() + Duration::from_secs(10))
            .await
            .map(|_| ()),
        Err(_) => match manager
            .reconcile_if_present(&intent, Instant::now() + Duration::from_secs(10))
            .await
        {
            Ok(Some(mut owned)) => owned
                .stop_and_drain(Instant::now() + Duration::from_secs(10))
                .await
                .map(|_| ()),
            Ok(None) => Ok(()),
            Err(error) => Err(error),
        },
    };
    let kill = child.start_kill();
    let reap = child.wait().await;
    assert!(
        checked.is_ok(),
        "delegation check: {checked:?}; cleanup: {cleanup:?}; kill: {kill:?}; reap: {reap:?}"
    );
    assert!(cleanup.is_ok(), "{cleanup:?}");
    assert!(reap.is_ok(), "{reap:?}");
}
