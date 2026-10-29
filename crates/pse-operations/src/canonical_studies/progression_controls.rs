// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Assertions for an explicitly owned private native service, never fixture discovery.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "isolated progression assertions fail immediately on unexpected canonical behavior"
)]
use super::*;
use crate::canonical::bounded_query;
use pse_model::study::{Dependency, SeedNeed};

// Erase the already-checked Send construction future at the fixture owner,
// rather than carrying its ingestion proof through the native service closure.
fn create<'a>(
    store: &'a CanonicalStore,
    study: &'a str,
    request: &'a RunRequest,
    points: &'a [NewOccurrence],
) -> std::pin::Pin<Box<dyn Future<Output = Result<Option<Study>, CanonicalError>> + Send + 'a>> {
    Box::pin(store.create_study(study, request, &[], points, &|| false))
}

fn request(revision: &crate::canonical::Revision, key: &str) -> RunRequest {
    RunRequest {
        purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
        admission_study: None,
        admission_point: None,
        key: key.into(),
        revision: revision.clone(),
        sources: vec![],
        request: vec![1],
        source_selection: vec![2],
        attestation: vec![3],
    }
}
fn occurrence(
    revision: &crate::canonical::Revision,
    key: u32,
    dependencies: Vec<Dependency>,
) -> NewOccurrence {
    NewOccurrence {
        projection: None,
        policy: PointPolicy {
            key: OccurrenceKey(key),
            dependencies,
            start: StartPolicy::Fresh,
            seed_need: SeedNeed::NotNeeded,
            attempt_limit: 1,
        },
        descriptor: vec![42],
        run: request(revision, &format!("point-run-{key}")),
    }
}
async fn progression_preparation(store: &CanonicalStore) -> StudyPreparation {
    let mut cursor = None;
    for _ in 0..64 {
        let progress = store.progress_studies(cursor.as_ref()).await.unwrap();
        cursor = progress.continuation;
        if let StudyProgressAction::Prepare(preparation) = progress.action {
            return *preparation;
        }
        assert!(
            cursor.is_some(),
            "requested preparation must remain discoverable"
        );
    }
    panic!("bounded fixture did not reach a preparation");
}

/// Actual prepared claims survive lost ACKs and failed readback without another attempt.
pub async fn prepared_claim_acknowledgement_is_original(store: &CanonicalStore) {
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    let points = vec![
        occurrence(&revision, 0, vec![]),
        occurrence(&revision, 1, vec![]),
    ];
    create(
        store,
        "claim-loss",
        &request(&revision, "claim-loss-parent"),
        &points,
    )
    .await
    .unwrap();
    let first = progression_preparation(store).await;
    let recovered = PREPARED_ACK_LOST
        .scope(
            true,
            store.claim_prepared_study_point(
                &first,
                None,
                "lost-claim",
                "original-worker",
                Duration::from_secs(60),
            ),
        )
        .await
        .unwrap();
    let PreparedStudyClaim::Claimed(recovered) = recovered else {
        panic!("original claim recovered");
    };
    let repeated = store
        .claim_prepared_study_point(
            &first,
            None,
            "lost-claim",
            "original-worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    let PreparedStudyClaim::Claimed(repeated) = repeated else {
        panic!("original claim replayed");
    };
    assert_eq!(recovered.fence.attempt(), repeated.fence.attempt());
    assert_eq!(recovered.fence.generation(), repeated.fence.generation());
    let second = progression_preparation(store).await;
    assert_ne!(first.scope().point().key, second.scope().point().key);
    let error = PREPARED_ACK_LOST
        .scope(
            true,
            PREPARED_READBACK_FAILED.scope(
                true,
                store.claim_prepared_study_point(
                    &second,
                    None,
                    "uncertain-claim",
                    "original-worker",
                    Duration::from_secs(60),
                ),
            ),
        )
        .await
        .unwrap_err();
    assert!(error.effect_is_unknown());
    match error {
        CanonicalError::SettlementUnresolved {
            owner,
            operation,
            request,
            readback: Some(_),
            ..
        } => {
            assert_eq!(owner, "study-preparation");
            assert_eq!(operation, "uncertain-claim");
            assert!(
                !request.is_empty(),
                "original immutable intent must remain available"
            );
        }
        error => panic!("uncertain prepared claim lost its original owner: {error:?}"),
    }
    let committed = store
        .canonical_study_point(&second.scope().point().key)
        .await
        .unwrap()
        .unwrap();
    assert!(committed.assigned && committed.attempt.is_some());
    let replay = store
        .claim_prepared_study_point(
            &second,
            None,
            "uncertain-claim",
            "original-worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    let PreparedStudyClaim::Claimed(replay) = replay else {
        panic!("original uncertain claim recovered");
    };
    assert_eq!(committed.attempt.as_deref(), Some(replay.fence.attempt()));
    assert!(
        store
            .claim_prepared_study_point(
                &second,
                None,
                "uncertain-claim",
                "different-worker",
                Duration::from_secs(60)
            )
            .await
            .is_err()
    );
    for preparation in [first, second] {
        let point = store
            .canonical_study_point(&preparation.scope().point().key)
            .await
            .unwrap()
            .unwrap();
        let facts = point_facts(&point).unwrap();
        assert_eq!(
            facts.attempt_count, 1,
            "ACK loss never claims a second attempt"
        );
        assert!(
            !facts.native_started,
            "recovering claim authority never starts native science"
        );
    }
}

/// The maximum supported immediate fan-in is loaded completely, while a stale
/// eligibility hint cannot bypass its unfinished ordering predecessors.
pub async fn high_degree_dependency_scope_is_complete(store: &CanonicalStore) {
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    let mut points = (0..64)
        .map(|key| occurrence(&revision, key, vec![]))
        .collect::<Vec<_>>();
    points.push(occurrence(
        &revision,
        64,
        (0..64)
            .map(|key| Dependency::Ordering(OccurrenceKey(key)))
            .collect(),
    ));
    create(
        store,
        "fan-in",
        &request(&revision, "fan-in-parent"),
        &points,
    )
    .await
    .unwrap();
    let child = point_key("fan-in", OccurrenceKey(64));
    let scope = store.study_scope(&child).await.unwrap();
    assert_eq!(scope.predecessors().len(), 64);
    let keys = scope
        .predecessors()
        .iter()
        .map(|point| point.occurrence)
        .collect::<BTreeSet<_>>();
    assert_eq!(keys, (0..64).collect());
    bounded_query(store.db.query("UPDATE canonical_study_readiness SET eligible=true WHERE study='fan-in' AND ordinal=64dec;")).await.unwrap();
    let mut cursor = None;
    let mut selected = BTreeSet::new();
    let mut exhaustive = false;
    for _ in 0..80 {
        let page = store.progress_studies(cursor.as_ref()).await.unwrap();
        cursor = page.continuation;
        if let StudyProgressAction::Prepare(preparation) = page.action {
            assert_ne!(
                preparation.scope().point().key,
                child,
                "stale hint bypassed unfinished predecessors"
            );
            assert!(selected.insert(preparation.scope().point().occurrence));
        }
        if cursor.is_none() {
            exhaustive = true;
            break;
        }
    }
    assert!(
        exhaustive,
        "bounded continuation must finish the examined population"
    );
    assert_eq!(selected, (0..64).collect());
    let unchanged = store.canonical_study_point(&child).await.unwrap().unwrap();
    assert!(!unchanged.assigned && unchanged.attempt.is_none());
    assert_eq!(point_facts(&unchanged).unwrap().attempt_count, 0);
}

/// Exercise sparse continuation and competing workers in this private store.
pub async fn sparse_pages_and_competing_workers(store: &CanonicalStore) {
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    let mut points = vec![occurrence(&revision, 0, vec![])];
    points.extend(
        (1..11).map(|key| occurrence(&revision, key, vec![Dependency::Ordering(OccurrenceKey(0))])),
    );
    points.push(occurrence(&revision, 11, vec![]));
    create(
        store,
        "progress",
        &request(&revision, "progress-parent"),
        &points,
    )
    .await
    .unwrap();
    let first = store.progress_studies(None).await.unwrap();
    let StudyProgressAction::Prepare(first_preparation) = first.action else {
        panic!("initial eligible root");
    };
    assert_eq!(first_preparation.scope().point().occurrence, 0);
    // Simulate stale discovery hints: authoritative policy must examine these
    // eight entries and preserve continuation to the independent twelfth point.
    bounded_query(store.db.query("UPDATE canonical_study_readiness SET eligible=true WHERE study='progress' AND ordinal>0dec AND ordinal<11dec;")).await.unwrap();
    let sparse = store
        .progress_studies(first.continuation.as_ref())
        .await
        .unwrap();
    assert!(matches!(sparse.action, StudyProgressAction::Waiting));
    assert_eq!(sparse.examined, 8);
    assert!(sparse.continuation.is_some());
    let final_page = store
        .progress_studies(sparse.continuation.as_ref())
        .await
        .unwrap();
    let StudyProgressAction::Prepare(selected) = final_page.action else {
        panic!("later independent point remains visible");
    };
    assert_eq!(selected.scope().point().occurrence, 11);
    let peer = store.independent_caller().unwrap();
    let peer_first = progression_preparation(&peer).await;
    assert_eq!(
        peer_first.scope().point().key,
        first_preparation.scope().point().key
    );
    let winner = store
        .claim_prepared_study_point(
            &first_preparation,
            None,
            "progress-winner",
            "worker-one",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    assert!(matches!(winner, PreparedStudyClaim::Claimed(_)));
    assert!(
        peer.claim_prepared_study_point(
            &peer_first,
            None,
            "progress-loser",
            "worker-two",
            Duration::from_secs(60)
        )
        .await
        .is_err()
    );
    let point = store
        .canonical_study_point(&first_preparation.scope().point().key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(point_facts(&point).unwrap().attempt_count, 1);
    assert!(!point_facts(&point).unwrap().native_started);
}

/// Exercise cancellation between canonical selection and native execution claim.
pub async fn cancellation_fences_preparation(store: &CanonicalStore) {
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    let points = vec![
        occurrence(&revision, 0, vec![]),
        occurrence(&revision, 1, vec![Dependency::Ordering(OccurrenceKey(0))]),
    ];
    create(
        store,
        "progress-cancel",
        &request(&revision, "progress-cancel-parent"),
        &points,
    )
    .await
    .unwrap();
    let selected = store.progress_studies(None).await.unwrap();
    let StudyProgressAction::Prepare(preparation) = selected.action else {
        panic!("initial root preparation");
    };
    store.cancel_study("progress-cancel").await.unwrap();
    assert!(
        store
            .claim_prepared_study_point(
                &preparation,
                None,
                "cancelled-claim",
                "worker",
                Duration::from_secs(60)
            )
            .await
            .is_err()
    );
    let unchanged = store
        .canonical_study_point(&preparation.scope().point().key)
        .await
        .unwrap()
        .unwrap();
    assert!(!unchanged.assigned && unchanged.attempt.is_none());
    assert_eq!(point_facts(&unchanged).unwrap().attempt_count, 0);
    // The pre-cancellation cursor already passed the root; the new authority
    // generation must revisit that prefix before claiming exhaustive no work.
    let mut cursor = selected.continuation;
    let mut settled = 0;
    for _ in 0..16 {
        let progress = store.progress_studies(cursor.as_ref()).await.unwrap();
        cursor = progress.continuation;
        if matches!(progress.action, StudyProgressAction::Settled { .. }) {
            settled += 1;
        }
        if matches!(progress.action, StudyProgressAction::Finalize(_)) {
            break;
        }
        assert!(cursor.is_some());
    }
    assert_eq!(settled, 2);
    for point in store
        .study_point_page("progress-cancel", None)
        .await
        .unwrap()
    {
        assert!(point.settled && !point.assigned && point.attempt.is_none());
        assert!(!point.facts().unwrap().native_started);
    }
}

/// Exercise issuing-store and immediate dependency revision fences before claim.
pub async fn dependency_revision_and_issuer(store: &CanonicalStore) {
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    let points = vec![
        occurrence(&revision, 0, vec![]),
        occurrence(&revision, 1, vec![Dependency::Ordering(OccurrenceKey(0))]),
    ];
    create(
        store,
        "progress-revision",
        &request(&revision, "progress-revision-parent"),
        &points,
    )
    .await
    .unwrap();
    let predecessor = store
        .canonical_study_point(&point_key("progress-revision", OccurrenceKey(0)))
        .await
        .unwrap()
        .unwrap();
    let mut facts = point_facts(&predecessor).unwrap();
    facts.revision = 0;
    facts.lifecycle = StudyPointState::Completed;
    // This private fixture supplies an already-settled ordering predecessor;
    // the control concerns consumed revision authority, not solver execution.
    bounded_query(store.db.query("UPDATE ONLY type::record('canonical_study_points',$point) SET facts=$facts,settled=true,revision+=1dec; RETURN fn::pse_study_v1::readiness_invalidate($pse_rpc_expires_at,(SELECT * FROM ONLY type::record('canonical_study_points',$point)),false);")
        .bind(("point",predecessor.key.clone())).bind(("facts",Bytes::from(encode(&facts).unwrap())))).await.unwrap();
    let predecessor = store
        .canonical_study_point(&predecessor.key)
        .await
        .unwrap()
        .unwrap();
    let child = progression_preparation(store).await;
    assert_eq!(child.scope().point().occurrence, 1);
    assert_eq!(
        child.scope().predecessors()[0].revision,
        predecessor.revision
    );
    let mut foreign = child.clone();
    foreign.issuer = "unrelated-issuing-store".into();
    assert!(
        store
            .claim_prepared_study_point(
                &foreign,
                None,
                "wrong-issuer",
                "worker",
                Duration::from_secs(60)
            )
            .await
            .is_err()
    );
    bounded_query(
        store
            .db
            .query("UPDATE ONLY type::record('canonical_study_points',$point) SET revision+=1dec;")
            .bind(("point", predecessor.key)),
    )
    .await
    .unwrap();
    assert!(
        store
            .claim_prepared_study_point(
                &child,
                None,
                "stale-predecessor",
                "worker",
                Duration::from_secs(60)
            )
            .await
            .is_err()
    );
    let unchanged = store
        .canonical_study_point(&child.scope().point().key)
        .await
        .unwrap()
        .unwrap();
    assert!(!unchanged.assigned && unchanged.attempt.is_none());
    assert_eq!(point_facts(&unchanged).unwrap().attempt_count, 0);
    assert!(!point_facts(&unchanged).unwrap().native_started);
}

/// Exercise native seed evidence refusal and exact original settlement identity.
pub async fn seed_refusal_retains_original_request(store: &CanonicalStore) {
    use crate::canonical_artifacts::{artifact_block, seal_artifact_request};
    use pse_model::study::{SeedAvailability, SeedRole};
    let revision = store.edit("problem", None, "source", &[]).await.unwrap();
    let mut point = occurrence(&revision, 0, vec![]);
    let seed = pse_model::generated::identities::SolutionId::from_bytes([11; 16]);
    let role = SeedRole::PrimalSolution;
    let mut artifact = store
        .new_artifact(&revision, "seed", vec![11], 1, 1)
        .await
        .unwrap();
    let blocks = [artifact_block(
        &artifact.key,
        0,
        "seed-fixture",
        vec![11],
        "original-seed",
        1,
    )];
    seal_artifact_request(
        &mut artifact,
        std::slice::from_ref(&revision),
        &[],
        &blocks,
        "user",
        "seed-owner",
    );
    store
        .persist_artifact(
            &artifact,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "user",
            "seed-owner",
        )
        .await
        .unwrap();
    store
        .retain_study_input("progress-seed", &seed.to_string(), &artifact.key)
        .await
        .unwrap();
    point.policy.seed_need = SeedNeed::Required;
    point.policy.start = StartPolicy::Explicit { role, seed };
    create(
        store,
        "progress-seed",
        &request(&revision, "progress-seed-parent"),
        &[point],
    )
    .await
    .unwrap();
    let preparation = progression_preparation(store).await;
    assert!(matches!(
        preparation.action().kind,
        ActionKind::Wait(pse_model::study::WaitReason::SeedResolution { .. })
    ));
    let absent = Some(SeedFact {
        role,
        availability: SeedAvailability::Absent,
    });
    for _ in 0..2 {
        assert!(matches!(
            store
                .claim_prepared_study_point(
                    &preparation,
                    absent.clone(),
                    "seed-settlement",
                    "worker",
                    Duration::from_secs(60)
                )
                .await
                .unwrap(),
            PreparedStudyClaim::Settled { .. }
        ));
    }
    let point = store
        .canonical_study_point(&preparation.scope().point().key)
        .await
        .unwrap()
        .unwrap();
    assert!(point.settled && !point.assigned && point.attempt.is_none());
    assert_eq!(point_facts(&point).unwrap().attempt_count, 0);
    assert!(!point_facts(&point).unwrap().native_started);
    assert!(
        store
            .claim_prepared_study_point(
                &preparation,
                Some(SeedFact {
                    role,
                    availability: SeedAvailability::Compatible { seed }
                }),
                "seed-settlement",
                "worker",
                Duration::from_secs(60)
            )
            .await
            .is_err()
    );
}
