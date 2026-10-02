// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared study policy applied by the actual store adapter. Native dispatch is excluded.
use crate::attempts::{
    AttemptKind, RuntimeTermination, Termination, TerminationCode, TransitionNote,
};
use crate::catalog::NewIntent;
use crate::jobs::{JobOutcome, JobState, NewJob, RetryPolicy};
use crate::lifecycle::AttemptState;
use crate::store_tests::{LEASE, Space, hash, new_attempt, new_job, space};
use crate::studies::{NewPoint, NewStudy, StudyState};
use crate::testing::TestDatabase;
use crate::{OperationsError, Store, mint_id};
use pse_model::generated::enums::StudyPointState;
use pse_model::study::*;

fn policy(key: u32, dependencies: Vec<Dependency>) -> PointPolicy {
    PointPolicy {
        key: OccurrenceKey(key),
        dependencies,
        seed_need: SeedNeed::Required,
        start: StartPolicy::Fresh,
        attempt_limit: 1,
    }
}
fn new_study(space: &Space, policies: Vec<PointPolicy>) -> NewStudy {
    let study_id = mint_id();
    let attempt = crate::attempts::NewAttempt {
        kind: AttemptKind::Study,
        ..new_attempt()
    };
    let publication_id = mint_id();
    NewStudy {
        study_id,
        intent: NewIntent {
            publication_id,
            workspace_id: space.id,
            attempt_id: attempt.attempt_id,
            member_prefix: format!(
                "{}members/{}/{publication_id}/",
                space.root, attempt.attempt_id
            ),
        },
        attempt,
        definition: serde_json::json!({"version":3}),
        finalization: NewJob {
            attempt: crate::attempts::NewAttempt {
                kind: AttemptKind::StudyFinalization,
                ..new_attempt()
            },
            ..new_job(&format!("{study_id}:final"), RetryPolicy::ONCE)
        },
        points: policies
            .into_iter()
            .map(|policy| NewPoint {
                binding_hash: hash(8),
                job: new_job(&format!("{study_id}:{}", policy.key.0), RetryPolicy::ONCE),
                policy,
            })
            .collect(),
    }
}
async fn finish(
    store: &Store,
    claimed: &crate::jobs::ClaimedJob,
    state: AttemptState,
    usable: bool,
    effect: EffectState,
) {
    let point = PointAttemptOutcome {
        attempt_id: Some(claimed.attempt_id),
        lifecycle: Some(state),
        diagnostic: None,
        scientific: ScientificFacts {
            usable,
            candidate_use: None,
            seed_permission: usable,
        },
        start: Some(StartProvenance::Fresh),
        effect,
    };
    store.jobs().finish(claimed.job_id,"worker",&JobOutcome {state,note:TransitionNote::by("worker").terminated(Termination {code:TerminationCode::Runtime(RuntimeTermination::Unassessed),detail:Some(serde_json::json!({"version":2,"point":point,"retry_failure":"deterministic","effect":effect}))}),retry_as:None,members:vec![]}).await.unwrap();
}

#[tokio::test]
async fn ordering_failed_terminal_releases_usable_dependency_refuses_equal_bindings_remain_distinct()
 {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store();
    let space = space(store).await;
    let study = new_study(
        &space,
        vec![
            policy(3, vec![]),
            policy(7, vec![Dependency::Ordering(OccurrenceKey(3))]),
            policy(11, vec![Dependency::UsableResult(OccurrenceKey(3))]),
        ],
    );
    store.studies().create(&study).await.unwrap();
    let first = store.jobs().claim("worker", LEASE).await.unwrap().unwrap();
    finish(
        store,
        &first,
        AttemptState::Failed,
        false,
        EffectState::Absent,
    )
    .await;
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(
        record
            .points
            .iter()
            .map(|point| point.binding_hash)
            .collect::<Vec<_>>(),
        vec![hash(8); 3]
    );
    assert_eq!(record.points[1].job_state, JobState::Queued);
    assert_eq!(record.points[2].state, StudyPointState::Failed);
    assert!(
        record.points[2]
            .outcome
            .as_ref()
            .unwrap()
            .diagnostic
            .is_some()
    );
    assert_ne!(record.points[2].state, StudyPointState::Cancelled);
    let next = store.jobs().claim("worker", LEASE).await.unwrap().unwrap();
    finish(
        store,
        &next,
        AttemptState::Completed,
        true,
        EffectState::Idempotent,
    )
    .await;
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(record.study.state, StudyState::Concluded);
    assert_eq!(record.attempt.state, AttemptState::Partial);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn unresolved_seed_is_acquisition_only_dispatch_rechecks_revision_and_live_lease() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store();
    let space = space(store).await;
    let mut child = policy(9, vec![]);
    child.start = StartPolicy::Continuation(SeedEdge {
        predecessor: OccurrenceKey(2),
        role: SeedRole::PrimalSolution,
        permission: ContinuationPermission::RequireUsable,
        unavailable: UnavailableSeedPolicy::Refuse,
    });
    let study = new_study(&space, vec![policy(2, vec![]), child]);
    store.studies().create(&study).await.unwrap();
    let parent = store.jobs().claim("worker", LEASE).await.unwrap().unwrap();
    finish(
        store,
        &parent,
        AttemptState::Completed,
        true,
        EffectState::Idempotent,
    )
    .await;
    let child = store.jobs().claim("worker", LEASE).await.unwrap().unwrap();
    let point = store.studies().point(study.study_id, 9).await.unwrap();
    assert!(point.outcome.as_ref().unwrap().start.is_none());
    let args = Some(SeedFact {
        role: SeedRole::PrimalSolution,
        availability: SeedAvailability::Compatible { seed: mint_id() },
    });
    assert!(matches!(
        store
            .studies()
            .admit_dispatch(
                study.study_id,
                OccurrenceKey(9),
                child.job_id,
                child.attempt_id,
                "other",
                point.revision,
                args.clone()
            )
            .await,
        Err(OperationsError::LeaseLost { .. })
    ));
    assert!(matches!(
        store
            .studies()
            .admit_dispatch(
                study.study_id,
                OccurrenceKey(9),
                child.job_id,
                child.attempt_id,
                "worker",
                point.revision,
                None
            )
            .await
            .unwrap(),
        ActionKind::Wait(WaitReason::SeedResolution { .. })
    ));
    assert!(matches!(
        store
            .studies()
            .admit_dispatch(
                study.study_id,
                OccurrenceKey(9),
                child.job_id,
                child.attempt_id,
                "worker",
                point.revision,
                args.clone()
            )
            .await
            .unwrap(),
        ActionKind::Start(StartProvenance::Continuation { .. })
    ));
    assert!(
        store
            .studies()
            .admit_dispatch(
                study.study_id,
                OccurrenceKey(9),
                child.job_id,
                child.attempt_id,
                "worker",
                point.revision,
                args
            )
            .await
            .is_err()
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn unknown_publication_effect_waits_for_reconciliation_and_cancellation_is_explicit() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store();
    let space = space(store).await;
    let study = new_study(
        &space,
        vec![
            policy(0, vec![]),
            policy(1, vec![Dependency::Ordering(OccurrenceKey(0))]),
        ],
    );
    store.studies().create(&study).await.unwrap();
    let first = store.jobs().claim("worker", LEASE).await.unwrap().unwrap();
    finish(
        store,
        &first,
        AttemptState::Failed,
        false,
        EffectState::Unknown,
    )
    .await;
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(record.study.state, StudyState::Open);
    assert_eq!(record.points[1].job_state, JobState::Waiting);
    let cancelled = store
        .studies()
        .cancel(study.study_id, "user")
        .await
        .unwrap();
    assert!(!cancelled.concluded);
    assert_eq!(
        store
            .studies()
            .point(study.study_id, 1)
            .await
            .unwrap()
            .state,
        StudyPointState::Cancelled
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn pre_effect_receipt_requires_dispatch_revision_and_live_owner_and_absence_is_fenced() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store();
    let space = space(store).await;
    let study = new_study(&space, vec![policy(4, vec![])]);
    store.studies().create(&study).await.unwrap();
    let claim = store.jobs().claim("worker", LEASE).await.unwrap().unwrap();
    let point = store.studies().point(study.study_id, 4).await.unwrap();
    assert!(matches!(
        store
            .studies()
            .admit_dispatch(
                study.study_id,
                OccurrenceKey(4),
                claim.job_id,
                claim.attempt_id,
                "worker",
                point.revision,
                None
            )
            .await
            .unwrap(),
        ActionKind::Start(_)
    ));
    let point = store.studies().point(study.study_id, 4).await.unwrap();
    let receipt = serde_json::json!({"exact":"native ticket"});
    assert!(matches!(
        store
            .studies()
            .record_receipt(
                study.study_id,
                OccurrenceKey(4),
                claim.job_id,
                claim.attempt_id,
                "other",
                point.revision,
                ScientificFacts::default(),
                None,
                receipt.clone()
            )
            .await,
        Err(OperationsError::LeaseLost { .. })
    ));
    assert!(
        store
            .studies()
            .record_receipt(
                study.study_id,
                OccurrenceKey(4),
                claim.job_id,
                claim.attempt_id,
                "worker",
                point.revision - 1,
                ScientificFacts::default(),
                None,
                receipt.clone()
            )
            .await
            .is_err()
    );
    store
        .studies()
        .record_receipt(
            study.study_id,
            OccurrenceKey(4),
            claim.job_id,
            claim.attempt_id,
            "worker",
            point.revision,
            ScientificFacts::default(),
            None,
            receipt.clone(),
        )
        .await
        .unwrap();
    let point = store.studies().point(study.study_id, 4).await.unwrap();
    assert_eq!(point.receipt, Some(receipt.clone()));
    assert_eq!(point.outcome.unwrap().effect, EffectState::Unknown);
    finish(
        store,
        &claim,
        AttemptState::Failed,
        false,
        EffectState::Unknown,
    )
    .await;
    let point = store.studies().point(study.study_id, 4).await.unwrap();
    // Merely observing absent native receipts never proves a persisted ticket safe to replay.
    assert!(
        store
            .studies()
            .reconcile_effect(
                study.study_id,
                OccurrenceKey(4),
                point.revision,
                EffectState::Absent
            )
            .await
            .is_err()
    );
    assert!(matches!(
        store
            .studies()
            .record_receipt(
                study.study_id,
                OccurrenceKey(4),
                claim.job_id,
                claim.attempt_id,
                "worker",
                point.revision,
                ScientificFacts::default(),
                None,
                receipt
            )
            .await,
        Err(OperationsError::LeaseLost { .. })
    ));
    database.remove().await.unwrap();
}
