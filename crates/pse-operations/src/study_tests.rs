// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The study repository (Plan 22 O7) against isolated PostgreSQL 18 databases: points
//! follow their jobs, predecessors release or cancel their dependents, cancellation stops
//! a study, and the last terminal point concludes it for its one publication.

use std::time::Duration;

use crate::attempts::{
    AttemptKind, NewAttempt, RuntimeTermination, TerminationCode, TransitionNote,
};
use crate::cancellation::CancelOutcome;
use crate::catalog::{
    Committed, MemberDescriptor, NewIntent, PublicationCommit, PublicationId, PublicationKind,
};
use crate::jobs::{ClaimedJob, JobOutcome, JobState, NewJob, RetryPolicy};
use crate::lifecycle::AttemptState;
use crate::store_tests::{LEASE, Space, hash, member, new_attempt, new_job, space};
use crate::studies::{
    NewPoint, NewStudy, StudyFilter, StudyId, StudyPointState as P, StudyRecord, StudyState,
};
use crate::testing::TestDatabase;
use crate::{OperationsError, Store, mint_id};

/// A study of `predecessors.len()` points in `space`, point `i` waiting for
/// `predecessors[i]`, each point's job tried at most `tries` times.
fn new_study(space: &Space, predecessors: &[Option<u32>], tries: u32) -> NewStudy {
    let study_id: StudyId = mint_id();
    let attempt = NewAttempt {
        kind: AttemptKind::Study,
        preparation_identity: None,
        ..new_attempt()
    };
    let publication_id: PublicationId = mint_id();
    let retry = RetryPolicy {
        max_tries: tries,
        backoff: Duration::ZERO,
        backoff_cap: Duration::ZERO,
    };
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
        definition: serde_json::json!({ "version": 1, "points": predecessors.len() }),
        finalization: NewJob {
            attempt: NewAttempt {
                kind: AttemptKind::StudyFinalization,
                ..new_attempt()
            },
            ..new_job(&format!("study:{study_id}:finalization"), RetryPolicy::ONCE)
        },
        points: predecessors
            .iter()
            .enumerate()
            .map(|(index, predecessor)| NewPoint {
                binding_hash: hash(u8::try_from(index).unwrap()),
                predecessor: *predecessor,
                job: new_job(&format!("study:{study_id}:point:{index}"), retry),
            })
            .collect(),
    }
}

/// Claim the next job; the study point it runs (`u32::MAX` for none).
async fn claimed_point(store: &Store, worker: &str) -> Option<(u32, ClaimedJob)> {
    let claimed = store.jobs().claim(worker, LEASE).await.unwrap()?;
    let point = store
        .studies()
        .point_of_job(claimed.job_id)
        .await
        .unwrap()
        .map_or(u32::MAX, |p| u32::try_from(p.point_index).unwrap());
    Some((point, claimed))
}

fn ended(state: AttemptState, members: Vec<MemberDescriptor>) -> JobOutcome {
    JobOutcome {
        state,
        note: TransitionNote::by("worker-a"),
        retry_as: None,
        members,
    }
}

/// Result members of point `index` under the study's intent prefix.
fn point_members(study: &NewStudy, index: u32) -> Vec<MemberDescriptor> {
    ["solve_runs", "solve_metrics"]
        .iter()
        .map(|name| MemberDescriptor {
            catalog_name: format!("point_{index}"),
            ..member(
                &format!(
                    "{}points/{index}/runtime/{name}/",
                    study.intent.member_prefix
                ),
                name,
                0,
            )
        })
        .collect()
}

fn states(record: &StudyRecord) -> Vec<P> {
    record.points.iter().map(|p| p.state).collect()
}

/// A study's points follow their jobs in the transactions that move them: a point waits
/// for its predecessor, completed points record their members, and the transaction that
/// ends the last point ends the study's own attempt and releases its finalization, whose
/// publication commits every completed point's members.
#[tokio::test]
async fn study_points_follow_their_jobs_and_conclude() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let space = space(&store).await;
    let study = new_study(&space, &[None, Some(0), None], 1);
    let created = store.studies().create(&study).await.unwrap();
    assert_eq!(created.point_jobs.len(), 3);
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(record.study.state, StudyState::Open);
    assert_eq!(record.attempt.state, AttemptState::Queued);
    assert_eq!(record.attempt.kind, AttemptKind::Study);
    assert_eq!(record.finalization.state, JobState::Waiting);
    assert_eq!(
        record
            .points
            .iter()
            .map(|p| p.job_state)
            .collect::<Vec<_>>(),
        [JobState::Queued, JobState::Waiting, JobState::Queued]
    );
    assert_eq!(record.points[1].attempt_state, AttemptState::Planned);
    // The intent is registered for the study's own attempt before any member write.
    let intent = store
        .catalog()
        .intent(study.intent.publication_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(intent.attempt_id, study.attempt.attempt_id);
    // The study's attempt coordinates; it never runs under a lease.
    assert!(matches!(
        store
            .attempts()
            .start(study.attempt.attempt_id, "worker-a", LEASE)
            .await
            .unwrap_err(),
        OperationsError::IllegalTransition { .. }
    ));

    // Point 1 waits for point 0: only points 0 and 2 are claimable.
    let (first, a) = claimed_point(&store, "worker-a").await.unwrap();
    let (second, b) = claimed_point(&store, "worker-a").await.unwrap();
    let mut claimed = [first, second];
    claimed.sort_unstable();
    assert_eq!(claimed, [0, 2]);
    assert!(claimed_point(&store, "worker-a").await.is_none());
    let (zero, two) = if first == 0 { (a, b) } else { (b, a) };
    assert_eq!(
        states(&store.studies().get(study.study_id).await.unwrap()),
        [P::Assigned, P::Pending, P::Assigned]
    );

    // Members are recorded only with a completed point.
    let failed_with_members = ended(AttemptState::Failed, point_members(&study, 2));
    assert!(matches!(
        store
            .jobs()
            .finish(two.job_id, "worker-a", &failed_with_members)
            .await
            .unwrap_err(),
        OperationsError::InvalidRequest { .. }
    ));
    let completed = ended(AttemptState::Completed, point_members(&study, 0));
    store
        .jobs()
        .finish(zero.job_id, "worker-a", &completed)
        .await
        .unwrap();
    // Completing point 0 released point 1.
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(states(&record), [P::Completed, P::Pending, P::Assigned]);
    assert_eq!(record.points[1].job_state, JobState::Queued);
    assert_eq!(record.points[1].attempt_state, AttemptState::Queued);
    let (one, claim) = claimed_point(&store, "worker-a").await.unwrap();
    assert_eq!(one, 1);
    let completed = ended(AttemptState::Completed, point_members(&study, 1));
    store
        .jobs()
        .finish(claim.job_id, "worker-a", &completed)
        .await
        .unwrap();
    assert_eq!(
        store
            .studies()
            .get(study.study_id)
            .await
            .unwrap()
            .study
            .state,
        StudyState::Open
    );
    // The last point fails: the study concludes as partial and releases its finalization.
    store
        .jobs()
        .finish(
            two.job_id,
            "worker-a",
            &ended(AttemptState::Failed, Vec::new()),
        )
        .await
        .unwrap();
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(states(&record), [P::Completed, P::Completed, P::Failed]);
    assert_eq!(record.study.state, StudyState::Concluded);
    assert_eq!(record.attempt.state, AttemptState::Partial);
    assert_eq!(record.finalization.state, JobState::Queued);

    // The failed point contributed no members.
    let members = store
        .studies()
        .completed_members(study.study_id)
        .await
        .unwrap();
    assert_eq!(
        members.iter().map(|(point, _)| *point).collect::<Vec<_>>(),
        [0, 0, 1, 1]
    );
    let finalization = store
        .jobs()
        .claim("worker-b", LEASE)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(finalization.job_id, created.finalization_job);
    // The study's one publication names its own attempt and commits every completed
    // point's members, all under the intent's prefix.
    let commit = PublicationCommit {
        publication_id: study.intent.publication_id,
        workspace_id: space.id,
        attempt_id: study.attempt.attempt_id,
        expected_parent: None,
        kind: PublicationKind::Relations,
        members: members.into_iter().map(|(_, member)| member).collect(),
        inputs: Vec::new(),
        windows: Vec::new(),
    };
    assert!(matches!(
        store.catalog().commit(&commit).await.unwrap(),
        Committed::Advanced { .. }
    ));
    store
        .studies()
        .mark_published(study.study_id)
        .await
        .unwrap();
    store
        .studies()
        .mark_published(study.study_id)
        .await
        .unwrap();
    let done = ended(AttemptState::Completed, Vec::new());
    store
        .jobs()
        .finish(finalization.job_id, "worker-b", &done)
        .await
        .unwrap();
    let listed = store
        .studies()
        .list(&StudyFilter::newest(10))
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].state, StudyState::Published);
    database.remove().await.unwrap();
}

/// A point whose predecessor fails never starts: it and its own dependents are cancelled
/// as unattempted, each recording which predecessor did not complete.
#[tokio::test]
async fn failed_predecessor_cancels_its_dependents() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let space = space(&store).await;
    let study = new_study(&space, &[None, Some(0), Some(1)], 1);
    store.studies().create(&study).await.unwrap();
    let (zero, claim) = claimed_point(&store, "worker-a").await.unwrap();
    assert_eq!(zero, 0);
    store
        .jobs()
        .finish(
            claim.job_id,
            "worker-a",
            &ended(AttemptState::Failed, Vec::new()),
        )
        .await
        .unwrap();
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(states(&record), [P::Failed, P::Cancelled, P::Cancelled]);
    assert_eq!(
        record.points[1].last_error.as_deref(),
        Some("predecessor point 0 failed")
    );
    assert_eq!(
        record.points[2].last_error.as_deref(),
        Some("predecessor point 1 was cancelled")
    );
    let dependent = store
        .attempts()
        .get(record.points[1].attempt_id)
        .await
        .unwrap();
    assert_eq!(dependent.state, AttemptState::Cancelled);
    assert_eq!(
        TerminationCode::of(&dependent).unwrap(),
        Some(TerminationCode::Runtime(RuntimeTermination::Unattempted))
    );
    assert_eq!(record.study.state, StudyState::Concluded);
    assert_eq!(record.attempt.state, AttemptState::Failed);
    assert!(
        store
            .studies()
            .completed_members(study.study_id)
            .await
            .unwrap()
            .is_empty()
    );
    database.remove().await.unwrap();
}

/// Cancelling a study cancels the points that have not started, asks a running try to
/// stop, and concludes the study as cancelled once that try ends. Cancelling a study's
/// own attempt cancels the study.
#[tokio::test]
async fn study_cancel_cancels_pending_and_stops_running_points() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let space = space(&store).await;
    let study = new_study(&space, &[None, Some(0), None], 1);
    store.studies().create(&study).await.unwrap();
    let (running, claim) = claimed_point(&store, "worker-a").await.unwrap();
    let cancelled = store
        .studies()
        .cancel(study.study_id, "user")
        .await
        .unwrap();
    assert_eq!(cancelled.stopping, [running]);
    assert!(!cancelled.concluded);
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(record.study.state, StudyState::Open);
    assert!(record.attempt.cancel_requested);
    for point in &record.points {
        if point.point_index == running {
            assert_eq!(point.state, P::Assigned);
            let attempt = store.attempts().get(point.attempt_id).await.unwrap();
            assert!(attempt.cancel_requested);
        } else {
            assert_eq!(point.state, P::Cancelled);
            assert_eq!(point.job_state, JobState::Cancelled);
            assert_eq!(point.last_error.as_deref(), Some("study cancelled by user"));
        }
    }
    // Nothing else is claimable; the running try ends cancelled and the study concludes.
    assert!(claimed_point(&store, "worker-b").await.is_none());
    store
        .jobs()
        .finish(
            claim.job_id,
            "worker-a",
            &ended(AttemptState::Cancelled, Vec::new()),
        )
        .await
        .unwrap();
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(record.study.state, StudyState::Concluded);
    assert_eq!(record.attempt.state, AttemptState::Cancelled);
    assert_eq!(record.finalization.state, JobState::Queued);
    assert!(
        store
            .studies()
            .cancel(study.study_id, "user")
            .await
            .unwrap()
            .already_concluded
    );

    // A study whose points have not started concludes at once when its attempt is
    // cancelled.
    let other = new_study(&space, &[None, None], 1);
    store.studies().create(&other).await.unwrap();
    assert_eq!(
        store
            .request_cancel(other.attempt.attempt_id, "user")
            .await
            .unwrap(),
        CancelOutcome::CancelledBeforeStart
    );
    let record = store.studies().get(other.study_id).await.unwrap();
    assert_eq!(states(&record), [P::Cancelled, P::Cancelled]);
    assert_eq!(record.attempt.state, AttemptState::Cancelled);
    database.remove().await.unwrap();
}

/// A point whose worker vanished goes back to pending as a new attempt of its job; a point
/// cancelled before it started takes its dependents with it; a job that runs no point
/// records no members.
#[tokio::test]
async fn study_point_requeue_and_direct_cancel() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let space = space(&store).await;
    let study = new_study(&space, &[None, None, Some(1)], 2);
    store.studies().create(&study).await.unwrap();
    let first = store
        .jobs()
        .claim("worker-a", Duration::from_millis(1))
        .await
        .unwrap()
        .unwrap();
    let point = store
        .studies()
        .point_of_job(first.job_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(point.state, P::Assigned);
    tokio::time::sleep(Duration::from_millis(20)).await;
    let swept = store.jobs().requeue_expired(10, mint_id).await.unwrap();
    assert_eq!(swept.len(), 1);
    let record = store.studies().get(study.study_id).await.unwrap();
    let requeued = &record.points[usize::try_from(point.point_index).unwrap()];
    assert_eq!(requeued.state, P::Pending);
    assert_eq!(requeued.job_state, JobState::Queued);
    assert_ne!(requeued.attempt_id, first.attempt_id);
    // Point 1 is cancelled before it starts: point 2, which waits on it, is cancelled too.
    assert_eq!(
        store
            .request_cancel(record.points[1].attempt_id, "user")
            .await
            .unwrap(),
        CancelOutcome::CancelledBeforeStart
    );
    let record = store.studies().get(study.study_id).await.unwrap();
    assert_eq!(states(&record)[1..], [P::Cancelled, P::Cancelled]);
    assert_eq!(record.study.state, StudyState::Open);

    // Members of a job that runs no point are refused.
    let plain = store
        .jobs()
        .enqueue(&new_job("plain", RetryPolicy::ONCE))
        .await
        .unwrap();
    loop {
        let claimed = store
            .jobs()
            .claim("worker-c", LEASE)
            .await
            .unwrap()
            .unwrap();
        if claimed.job_id == plain.job_id() {
            let refused = store
                .jobs()
                .finish(
                    claimed.job_id,
                    "worker-c",
                    &ended(AttemptState::Completed, point_members(&study, 0)),
                )
                .await
                .unwrap_err();
            assert!(matches!(refused, OperationsError::InvalidRequest { .. }));
            break;
        }
    }
    database.remove().await.unwrap();
}

/// Captured incumbent solutions expire with their streams (Plan 22 I13): once a finished
/// attempt's incumbents expire, its captures go too, while output seeds stay. A capture
/// survives while something may still start from it: the incumbents above an unfinished
/// retry (its resumed try), an unfinished job whose stored start names it, or a waiting
/// study point whose predecessor's attempt captured it.
#[tokio::test]
async fn captured_solutions_pruned_with_streams() {
    use crate::attempts::AttemptId;
    use crate::solutions::{NewSolution, SeedVectors, SolutionId};
    use crate::store_tests::{at, finished_attempt, finished_try};
    use crate::streams::{Retention, RuntimeOperationalIncumbentsRow};
    use pse_model::generated::enums::NativeBackend;
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let seed = |attempt: AttemptId| NewSolution {
        solution_id: mint_id(),
        compatibility_stamp: hash(5),
        preparation_identity: hash(6),
        backend: NativeBackend::Scip,
        profile_stamp: hash(3),
        data_stamp: hash(4),
        vectors: SeedVectors::Root { primal: vec![1.0] },
        created_by: Some(attempt),
    };
    // Each attempt streams one incumbent with its captured point.
    let capture = |attempt: AttemptId| {
        let store = store.clone();
        async move {
            let solution = seed(attempt);
            let incumbent = RuntimeOperationalIncumbentsRow {
                attempt_id: attempt,
                seq: 0,
                step: 0,
                at: at(0).timestamp_micros(),
                elapsed_seconds: 1.0,
                phase: "scip.incumbent".into(),
                objective: 1.0,
                dual_bound: None,
                gap: None,
                nodes: Some(1),
                seconds: Some(1.0),
                solution_id: Some(solution.solution_id),
            };
            store
                .streams()
                .record_incumbents(&[incumbent], std::slice::from_ref(&solution))
                .await
                .unwrap();
            solution.solution_id
        }
    };
    // A finished attempt with nothing waiting on it: its capture expires, its output
    // seed stays.
    let done = finished_attempt(&store).await;
    let expired = capture(done).await;
    let output = seed(done);
    store.solutions().put(&output).await.unwrap();
    // A finished try whose retry is still queued: a resumed try may start from it.
    let superseded = finished_attempt(&store).await;
    let resumable = capture(superseded).await;
    let retry = NewAttempt {
        parent_attempt: Some(superseded),
        ..new_attempt()
    };
    store.attempts().create(&retry, None).await.unwrap();
    store
        .attempts()
        .transition(
            retry.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("q"),
        )
        .await
        .unwrap();
    // A completed study point whose successor still waits for its seed.
    let space = space(&store).await;
    let study = new_study(&space, &[None, Some(0)], 1);
    store.studies().create(&study).await.unwrap();
    let (point, claimed) = claimed_point(&store, "worker-a").await.unwrap();
    assert_eq!(point, 0);
    let predecessor = capture(claimed.attempt_id).await;
    store
        .jobs()
        .finish(
            claimed.job_id,
            "worker-a",
            &ended(AttemptState::Completed, point_members(&study, 0)),
        )
        .await
        .unwrap();
    // A finished attempt whose capture a queued job's stored start names; the job runs
    // before the study's remaining point.
    let named_by = finished_try(&store, None).await;
    let started = capture(named_by).await;
    let job = NewJob {
        payload_version: 3,
        payload: serde_json::json!({
            "version": 3,
            "task": {
                "kind": "modeling",
                "start": { "kind": "stored_solution", "solution": started.to_string() }
            }
        }),
        priority: 1,
        ..new_job("resume-from-capture", RetryPolicy::ONCE)
    };
    let resume = store.jobs().enqueue(&job).await.unwrap().job_id();

    let solution = |id: SolutionId| {
        let store = store.clone();
        async move { store.solutions().get(id).await.unwrap().is_some() }
    };
    // Nothing is old enough under a long retention.
    store
        .streams()
        .apply_retention(Retention {
            finished_for: Duration::from_secs(3600),
        })
        .await
        .unwrap();
    for id in [expired, resumable, started, predecessor] {
        assert!(solution(id).await, "{id}");
    }
    store
        .streams()
        .apply_retention(Retention {
            finished_for: Duration::ZERO,
        })
        .await
        .unwrap();
    assert!(!solution(expired).await);
    assert!(solution(output.solution_id).await);
    for id in [resumable, started, predecessor] {
        assert!(solution(id).await, "{id}");
    }
    let incumbents = |attempt: AttemptId| {
        let store = store.clone();
        async move {
            store
                .streams()
                .incumbent_snapshot(attempt)
                .await
                .unwrap()
                .len()
        }
    };
    assert_eq!(incumbents(done).await, 0);
    assert_eq!(incumbents(superseded).await, 1);
    assert_eq!(incumbents(named_by).await, 0);
    assert_eq!(incumbents(claimed.attempt_id).await, 0);
    // Once the job and the successor have run, the captures follow their streams.
    let claim = store
        .jobs()
        .claim("worker-a", LEASE)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claim.job_id, resume);
    store
        .jobs()
        .finish(
            claim.job_id,
            "worker-a",
            &ended(AttemptState::Completed, Vec::new()),
        )
        .await
        .unwrap();
    let (point, successor) = claimed_point(&store, "worker-a").await.unwrap();
    assert_eq!(point, 1);
    store
        .jobs()
        .finish(
            successor.job_id,
            "worker-a",
            &ended(AttemptState::Completed, point_members(&study, 1)),
        )
        .await
        .unwrap();
    store
        .streams()
        .apply_retention(Retention {
            finished_for: Duration::ZERO,
        })
        .await
        .unwrap();
    assert!(!solution(started).await);
    assert!(!solution(predecessor).await);
    assert!(solution(resumable).await);
    database.remove().await.unwrap();
}
