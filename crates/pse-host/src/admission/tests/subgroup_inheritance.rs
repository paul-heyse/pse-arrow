// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::manager::subgroup::payload::{CreationOutcome, OwnedSubgroup};
use futures_util::FutureExt;
use std::{ffi::OsString, process::Stdio};
use tokio::io::AsyncWriteExt;

const MEMORY: u64 = 256 * 1024 * 1024;

/// Executed directly after the native parent attaches and releases its exec gate.
#[tokio::test]
async fn borrowed_member_probe() {
    let Ok(encoded) = std::env::var("PSE_BORROW_FIXTURE_ENV") else {
        return;
    };
    let entries: Vec<(OsString, OsString)> = serde_json::from_str(&encoded).unwrap();
    let environment: Environment = entries.into_iter().collect();
    let coordinator = Coordinator::open(Path::new("/first-checkout"), &environment).unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let membership = manager.current_membership().await.unwrap();
    assert_ne!(
        membership.member(),
        Path::new("/sys/fs/cgroup").join(current.control_group.trim_start_matches('/'))
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let before = coordinator.snapshot(deadline).await.unwrap();
    let before_wire = serde_json::to_value(&before).unwrap();
    let mut permit = coordinator
        .inherit(&environment, &manager, None, &current, deadline)
        .await
        .unwrap()
        .expect("actual child member must inherit its registered finite root");
    assert!(permit.borrowed());
    permit.prepare(&manager, deadline).await.unwrap();
    assert!(permit.release(&manager, deadline).await.unwrap());
    let after = coordinator.snapshot(deadline).await.unwrap();
    assert_eq!(
        serde_json::to_value(&after).unwrap(),
        before_wire,
        "borrowed settlement must not mutate/release its original reservation"
    );
    assert_eq!(after.finite.len(), 1);
    assert!(after.finite.values().all(|row| !row.released));
    let original = manager.current_unit().await.unwrap();
    assert_eq!(original.unit, current.unit);
    assert_eq!(original.invocation, current.invocation);
    assert_eq!(original.control_group, current.control_group);
    assert_eq!(original.active_state, "active");
}

#[tokio::test]
async fn actual_subgroup_inherits_root_reservation_without_releasing_its_owner() {
    let (_runtime, mut environment, coordinator) = fixture();
    environment.insert("PSE_MEMORY_MAX".into(), (MEMORY / 2).to_string().into());
    let manager = HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    let allocation_intent = LaunchIntent::fresh_allocation(&slice).unwrap();
    let root_intent = LaunchIntent::fresh_scope().unwrap();
    let nonce = root_intent.unit().to_owned();
    environment.insert(MARKER.into(), nonce.clone().into());
    let mut allocation = None;
    let mut root = None;
    let mut owner_permit = None;
    let mut groups: Vec<OwnedSubgroup> = vec![];
    let mut children = vec![];
    let body = std::panic::AssertUnwindSafe(async {
        allocation = Some(
            manager
                .start_slice(
                    &allocation_intent,
                    &ResourceControls {
                        memory_max: Some(MEMORY),
                        memory_swap_max: Some(0),
                        ..Default::default()
                    },
                    Instant::now() + Duration::from_secs(15),
                )
                .await
                .unwrap(),
        );
        children.push(
            tokio::process::Command::new("/bin/sleep")
                .arg("60")
                .kill_on_drop(true)
                .spawn()
                .unwrap(),
        );
        let pid = rustix::process::Pid::from_raw(children[0].id().unwrap() as i32).unwrap();
        let fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
        root = Some(
            manager
                .start_scope_delegated(
                    &root_intent,
                    fd,
                    &current.unit,
                    allocation_intent.unit(),
                    Instant::now() + Duration::from_secs(15),
                    Duration::from_secs(5),
                    &ResourceControls {
                        memory_max: Some(MEMORY),
                        memory_swap_max: Some(0),
                        ..Default::default()
                    },
                )
                .await
                .unwrap(),
        );
        let profile = selected(Some("light"), Some("268435456"), "bounded").profile;
        let row = Reservation {
            boot: manager.boot_id().into(),
            manager: manager.identity_key(),
            caller: caller_identity().unwrap(),
            gated_child: process_identity(children[0].id().unwrap()).unwrap(),
            profile: profile.clone(),
            units: vec![
                RegisteredUnit {
                    intent: allocation_intent.clone(),
                    binding: Some(allocation.as_ref().unwrap().binding().await.unwrap()),
                    controller: false,
                },
                RegisteredUnit {
                    intent: root_intent.clone(),
                    binding: Some(root.as_ref().unwrap().binding().await.unwrap()),
                    controller: false,
                },
            ],
            released: false,
        };
        let mut state = Journal::default();
        state.finite.insert(nonce.clone(), row);
        coordinator.write(&state).unwrap();
        owner_permit = Some(Permit {
            coordinator: coordinator.clone(),
            selection: None,
            profile,
            nonce: Some(nonce.clone()),
            slice: Some(allocation_intent.clone()),
            slice_owner: allocation.take(),
            borrowed: false,
        });
        let parent = root.as_ref().unwrap().payload_parent().await.unwrap();
        let intent = parent.creation_intent("borrowed-member").unwrap();
        match parent.create(&intent).await {
            CreationOutcome::Created(group) => groups.push(group),
            outcome => panic!("{outcome:?}"),
        }
        let mut child = tokio::process::Command::new("/bin/sh")
            .args([
                "-c",
                "read -r gate || exit 125; [ \"$gate\" = 1 ] || exit 125; exec \"$@\"",
                "gate",
            ])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "admission::tests::subgroup_inheritance::borrowed_member_probe",
                "--nocapture",
            ])
            .env(
                "PSE_BORROW_FIXTURE_ENV",
                serde_json::to_string(&environment.iter().collect::<Vec<_>>()).unwrap(),
            )
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut gate = child.stdin.take().unwrap();
        children.push(child);
        let attached = groups[0].attach(&mut children[1]).await.unwrap();
        groups[0]
            .validate_attachment(&attached, &children[1])
            .await
            .unwrap();
        gate.write_all(b"1\n").await.unwrap();
        drop(gate);
        let status = tokio::time::timeout(Duration::from_secs(15), children[1].wait())
            .await
            .unwrap()
            .unwrap();
        assert!(
            status.success(),
            "actual borrowed-member probe failed: {status}"
        );
        let state = coordinator
            .snapshot(Instant::now() + Duration::from_secs(5))
            .await
            .unwrap();
        assert!(!state.finite[&nonce].released);
        assert_eq!(
            root.as_ref().unwrap().observe().await.unwrap().active_state,
            "active"
        );
    })
    .catch_unwind()
    .await;
    let mut failures = Vec::new();
    if root.is_none() {
        match manager
            .reconcile_if_present(&root_intent, Instant::now() + Duration::from_secs(10))
            .await
        {
            Ok(original) => root = original,
            Err(error) => failures.push(format!("original root reconciliation {error:?}")),
        }
    }
    for group in &mut groups {
        let kill = group.kill().await;
        let drain = group
            .wait_empty(Instant::now() + Duration::from_secs(10))
            .await;
        let remove = group
            .remove_tree(Instant::now() + Duration::from_secs(10))
            .await;
        if kill.is_err()
            || drain.is_err()
            || !matches!(&remove,Ok(receipt) if receipt.original_removed&&receipt.failures.is_empty())
        {
            failures.push(format!(
                "child group kill={kill:?} drain={drain:?} removal={remove:?}"
            ));
        }
    }
    if let Some(root) = root.as_mut() {
        if let Err(error) = root
            .stop_and_drain(Instant::now() + Duration::from_secs(10))
            .await
        {
            failures.push(format!("root {error:?}"));
        }
    }
    let reaps = futures_util::future::join_all(children.iter_mut().map(|child| async move {
        let kill = child.start_kill();
        let reap = child.wait().await;
        if reap.is_err() {
            Some(format!("child kill={kill:?} reap={reap:?}"))
        } else {
            None
        }
    }))
    .await;
    failures.extend(reaps.into_iter().flatten());
    if let Some(permit) = owner_permit.as_mut() {
        let released = permit
            .release(&manager, Instant::now() + Duration::from_secs(10))
            .await;
        if !matches!(released, Ok(true)) {
            failures.push(format!("original owner reservation release {released:?}"));
        }
    }
    if allocation.is_none() && owner_permit.is_none() {
        match manager
            .reconcile_if_present(&allocation_intent, Instant::now() + Duration::from_secs(10))
            .await
        {
            Ok(original) => allocation = original,
            Err(error) => failures.push(format!("original allocation reconciliation {error:?}")),
        }
    }
    if let Some(allocation) = allocation.as_mut() {
        if let Err(error) = allocation
            .stop_and_drain(Instant::now() + Duration::from_secs(10))
            .await
        {
            failures.push(format!("allocation {error:?}"));
        }
    }
    if let Err(panic) = body {
        eprintln!("owned cleanup failures: {failures:?}");
        std::panic::resume_unwind(panic);
    }
    assert!(failures.is_empty(), "{failures:#?}");
    assert!(
        coordinator
            .snapshot(Instant::now() + Duration::from_secs(5))
            .await
            .unwrap()
            .finite
            .is_empty()
    );
}
