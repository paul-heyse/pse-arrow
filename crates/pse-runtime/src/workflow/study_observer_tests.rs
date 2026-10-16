// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable observation controls without a service or a fabricated scientific result.

use super::*;
use pse_model::generated::identities::StudyId;
use std::{collections::VecDeque, future::ready};

async fn scripted_observation(
    study: StudyId,
    cancel: &crate::CancelSource,
    inventory: Vec<Option<usize>>,
    drain: Vec<Result<bool, WorkflowError>>,
) -> (Result<usize, WorkflowError>, usize, usize, usize) {
    let mut inventory = VecDeque::from(inventory);
    let mut drain = VecDeque::from(drain);
    let mut reads = 0;
    let mut cancellations = 0;
    let mut observations = 0;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        wait_study_inventory(
            study,
            cancel,
            std::time::Duration::from_millis(100),
            || {
                reads += 1;
                ready(Ok(inventory.pop_front().expect("unexpected inventory read")))
            },
            || {
                cancellations += 1;
                ready(Ok(()))
            },
            || {
                observations += 1;
                ready(drain.pop_front().expect("unexpected receiver observation"))
            },
        ),
    )
    .await
    .expect("scripted durable observation must terminate");
    (result, reads, cancellations, observations)
}

#[tokio::test]
async fn durable_observer_terminal_inventory_wins_before_receiver_observation() {
    let cancel = crate::CancelSource::new();
    let (result, reads, cancellations, observations) =
        scripted_observation(pse_operations::mint_id(), &cancel, vec![Some(7)], vec![]).await;
    assert_eq!(result.unwrap(), 7);
    assert_eq!((reads, cancellations, observations), (1, 0, 0));
}

#[tokio::test]
async fn durable_observer_finalization_racing_drain_keeps_original_inventory() {
    let cancel = crate::CancelSource::new();
    let (result, reads, cancellations, observations) = scripted_observation(
        pse_operations::mint_id(), &cancel, vec![None, Some(7)], vec![Ok(true)],
    ).await;
    assert_eq!(result.unwrap(), 7);
    assert_eq!((reads, cancellations, observations), (2, 0, 1));
}

#[tokio::test]
async fn durable_observer_cancelled_dead_primary_returns_recoverable_identity() {
    let study = pse_operations::mint_id();
    let cancel = crate::CancelSource::new();
    cancel.cancel();
    let (result, reads, cancellations, observations) =
        scripted_observation(study, &cancel, vec![None, None], vec![Ok(true)]).await;
    assert!(matches!(result, Err(WorkflowError::StudyExecutionUnavailable { study: actual }) if actual == study));
    assert_eq!((reads, cancellations, observations), (2, 1, 1));
}

#[tokio::test]
async fn durable_observer_dead_primary_needs_no_caller_cancellation() {
    let study = pse_operations::mint_id();
    let cancel = crate::CancelSource::new();
    let (result, reads, cancellations, observations) =
        scripted_observation(study, &cancel, vec![None, None], vec![Ok(true)]).await;
    assert!(matches!(result, Err(WorkflowError::StudyExecutionUnavailable { study: actual }) if actual == study));
    assert_eq!((reads, cancellations, observations), (2, 0, 1));
}

#[tokio::test]
async fn durable_observer_live_cancellation_records_once_and_joins_inventory() {
    let cancel = crate::CancelSource::new();
    cancel.cancel();
    let (result, reads, cancellations, observations) = scripted_observation(
        pse_operations::mint_id(), &cancel,
        vec![None, None, Some(7)], vec![Ok(false), Ok(false)],
    ).await;
    assert_eq!(result.unwrap(), 7);
    assert_eq!((reads, cancellations, observations), (3, 1, 2));
}

#[tokio::test]
async fn durable_observer_unknown_observation_does_not_authorize_unfinished_exit() {
    let cancel = crate::CancelSource::new();
    cancel.cancel();
    let (result, reads, cancellations, observations) = scripted_observation(
        pse_operations::mint_id(), &cancel,
        vec![None, None, Some(7)],
        vec![Err(super::super::contract("receiver observation unavailable")), Ok(false)],
    ).await;
    assert_eq!(result.unwrap(), 7);
    assert_eq!((reads, cancellations, observations), (3, 1, 2));
}
