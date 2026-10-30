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
