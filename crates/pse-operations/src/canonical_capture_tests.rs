// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Included in the canonical study server test module; uses its isolated fixture.

#[tokio::test]
async fn capture_locator_profile_admits_native_wrappers_and_refuses_deeper_values() {
    use crate::canonical::MetadataResponse;
    let (store, _, _) = fixture().await;
    // The exact locator nesting adds source/page wrappers around native rows.
    // A manifest can occupy nearly the whole native byte-bound page, while a
    // native record id adds RecordId/RecordIdKey protobuf message levels.
    let shape = "RETURN {items:[{position:0dec,manifest:{descriptors:$descriptors},page:{blocks:[{id:type::record('canonical_result_blocks','probe')}],batches:[],indexes:[],next:NONE,exhausted:true}}],member:1dec,after:NONE,exhausted:true};";
    let descriptors = Bytes::from(vec![
        0;
        pse_schema::catalog::study_workbench::STUDY_PAGE_BYTES
            - 4096
    ]);
    let mut response = bounded_query(
        store
            .db
            .metadata_query(MetadataResponse::CaptureLocators, shape)
            .bind(("descriptors", descriptors)),
    )
    .await
    .unwrap();
    assert!(matches!(
        response.take::<Value>(0).unwrap(),
        Value::Object(_)
    ));
    drop(response);
    let shallow = bounded_query(
        store
            .db
            .metadata_query(MetadataResponse::StudyPage, shape)
            .bind(("descriptors", Bytes::from(vec![]))),
    )
    .await
    .unwrap_err();
    assert!(
        shallow.to_string().contains("depth envelope"),
        "ordinary study-page depth must remain closed: {shallow:?}"
    );
    let deeper = shape.replace(
        "id:type::record('canonical_result_blocks','probe')",
        "extra:{extra:{extra:{extra:true}}}",
    );
    let error = bounded_query(
        store
            .db
            .metadata_query(MetadataResponse::CaptureLocators, deeper)
            .bind(("descriptors", Bytes::from(vec![]))),
    )
    .await
    .unwrap_err();
    assert!(
        error.to_string().contains("depth envelope"),
        "locator endpoint cannot admit arbitrary deeper values: {error:?}"
    );
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn capture_last_owner_drain_refuses_exact_replay_until_native_release() {
    use crate::canonical_study_workbench::{StudyWorkbenchFilter, pause_capture_release};
    let (store, _, revision) = fixture().await;
    let points = (0..3)
        .map(|n| occurrence(&revision, n, vec![]))
        .collect::<Vec<_>>();
    store
        .create_study(
            "capture-drain-study",
            &request(&revision, "capture-drain-parent"),
            &[],
            &points,
            &|| false,
        )
        .await
        .unwrap();
    let filter = StudyWorkbenchFilter {
        study: Some("capture-drain-study".into()),
        ..Default::default()
    };
    let key = format!("capture-drain-{}", uuid::Uuid::new_v4());
    let selected = store
        .capture_study_workbench(&key, &filter, 1, 3)
        .await
        .unwrap();
    let replay = store
        .capture_study_workbench(&key, &filter, 1, 3)
        .await
        .unwrap();
    assert_eq!(selected.capture.key(), replay.capture.key());
    let pause = pause_capture_release(&key);
    drop(selected);
    assert_eq!(
        replay
            .capture
            .member_page(None, 3)
            .await
            .unwrap()
            .rows
            .len(),
        3,
        "one replay handle cannot release another handle's facts"
    );
    drop(replay);
    tokio::time::timeout(Duration::from_secs(5), pause.entered.notified())
        .await
        .unwrap();
    assert!(matches!(
        store.capture_study_workbench(&key, &filter, 1, 3).await,
        Err(CanonicalError::CaptureDraining { .. })
    ));
    let mut facts = bounded_query(
        store
            .db
            .query("SELECT key FROM canonical_study_capture_members WHERE capture=$capture;")
            .bind(("capture", key.clone())),
    )
    .await
    .unwrap();
    assert_eq!(
        facts.take::<Vec<Object>>(0).unwrap().len(),
        3,
        "paused cleanup retains all frozen facts"
    );
    pause.resume.notify_one();
    store.result_read_drain.drain().await.unwrap();
    let mut closed=bounded_query(store.db.query("SELECT state FROM ONLY type::record('canonical_study_captures',$capture); SELECT key FROM canonical_study_capture_members WHERE capture=$capture;").bind(("capture",key))).await.unwrap();
    assert_eq!(
        closed
            .take::<Option<Object>>(0)
            .unwrap()
            .unwrap()
            .get("state"),
        Some(&Value::String("released".into()))
    );
    assert!(closed.take::<Vec<Object>>(1).unwrap().is_empty());
}

#[tokio::test]
async fn installation_expired_issued_page_replays_its_original_grant() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let (store, _, revision) = fixture().await;
    let points = (0..3)
        .map(|key| {
            occurrence(
                &revision,
                key,
                if key == 2 {
                    vec![]
                } else {
                    vec![Dependency::Ordering(OccurrenceKey(key + 1))]
                },
            )
        })
        .collect::<Vec<_>>();
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    let parent = request(&revision, "expired-page-parent");
    let created = CREATION_OBSERVER
        .scope(
            Box::new(move |boundary| {
                if boundary == CreationBoundary::NodesInstalled {
                    signal.store(true, Ordering::SeqCst);
                }
            }),
            store.create_study("expired-page", &parent, &[], &points, &|| {
                cancelled.load(Ordering::SeqCst)
            }),
        )
        .await
        .unwrap();
    assert!(created.is_none());
    let mut state = bounded_query(store.db.query(
        "SELECT digest FROM ONLY type::record('canonical_study_installations','expired-page');",
    ))
    .await
    .unwrap();
    let mut state = state.take::<Option<Object>>(0).unwrap().unwrap();
    let digest = codec::decode_string(codec::required(&mut state, "digest").unwrap()).unwrap();
    let original = Bytes::from(vec![11_u8]);
    let mut issue=protected_query("installation-test-issue",||Ok(store.db.query("RETURN fn::pse_study_v1::issue_page(time::micros()+100000,'expired-page',$digest,'edges',0dec,$request,1);").bind(("digest",digest.clone())).bind(("request",original.clone())))).await.unwrap();
    let issued = issue.take::<Option<Object>>(0).unwrap().unwrap();
    tokio::time::sleep(Duration::from_millis(110)).await;
    let mut replay=protected_query("installation-test-replay",||Ok(store.db.query("RETURN fn::pse_study_v1::issue_page($pse_rpc_expires_at,'expired-page',$digest,'edges',0dec,$request,1);").bind(("digest",digest.clone())).bind(("request",original.clone())))).await.unwrap();
    let replay = replay.take::<Option<Object>>(0).unwrap().unwrap();
    assert_eq!(
        issued.get("issued_expires_at"),
        replay.get("issued_expires_at")
    );
    let expired=bounded_query(store.db.query("RETURN fn::pse_study_v1::install_page($pse_rpc_expires_at,'expired-page',$digest,'edges',0dec,$request,[],[],[]);").bind(("digest",digest)).bind(("request",original))).await.unwrap_err();
    assert!(matches!(
        expired,
        CanonicalError::Refused {
            code: crate::canonical::RefusalCode::StudyInstallationOriginalPageExpired,
            ..
        }
    ));
    let header = store
        .canonical_study("expired-page")
        .await
        .unwrap()
        .unwrap();
    assert!(!header.active);
    assert_eq!(header.installed_dependencies, 0);
}

#[tokio::test]
async fn capture_all_unavailable_members_retain_the_named_parent_summary() {
    use crate::canonical_study_workbench::StudyWorkbenchFilter;
    let (store, _, revision) = fixture().await;
    store
        .create_study(
            "capture-parent",
            &request(&revision, "capture-parent-run"),
            &[],
            &[occurrence(&revision, 0, vec![])],
            &|| false,
        )
        .await
        .unwrap();
    store.cancel_study("capture-parent").await.unwrap();
    let scope = store
        .study_scope(&point_key("capture-parent", OccurrenceKey(0)))
        .await
        .unwrap();
    store
        .refuse_study_point(&scope, &scope.action(None).unwrap(), None)
        .await
        .unwrap();
    let summary = store
        .canonical_study_summary("capture-parent")
        .await
        .unwrap()
        .unwrap();
    let fence = store
        .begin_study_finalization(&summary, "summary", Duration::from_secs(60))
        .await
        .unwrap()
        .unwrap();
    let closed = store
        .close_result_ingestion(&fence, "capture-parent-close")
        .await
        .unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: this fixture owns the exact effect-free cancelled point and parent summary.
    unsafe {
        store
            .seal_study_summary(
                "capture-parent",
                &manifest,
                "capture-parent-seal",
                TerminalClass::Cancelled,
                b"cancelled fixture",
            )
            .await
    }
    .unwrap();
    let selected = store
        .capture_study_workbench(
            "capture-parent-cut",
            &StudyWorkbenchFilter {
                study: Some("capture-parent".into()),
                ..Default::default()
            },
            1,
            1,
        )
        .await
        .unwrap();
    assert!(
        selected
            .capture
            .member_page(None, 1)
            .await
            .unwrap()
            .rows
            .iter()
            .all(|member| member.value.manifest.is_none())
    );
    assert_eq!(
        selected.capture.header_page(None, 1).await.unwrap().rows[0]
            .value
            .result_attempt
            .as_deref(),
        Some(fence.attempt())
    );
    let mut protected=bounded_query(store.db.query("SELECT run,attempt,manifest FROM canonical_study_capture_protections WHERE capture='capture-parent-cut';")).await.unwrap();
    let protected = protected.take::<Vec<Object>>(0).unwrap();
    assert_eq!(
        protected.len(),
        1,
        "the named parent is protected without counting it as a point result"
    );
    assert_eq!(
        protected[0].get("run"),
        Some(&Value::String(fence.run().to_owned()))
    );
    store.forget_study_results("capture-parent").await.unwrap();
    assert!(matches!(
        store.forget_run_results(fence.run()).await,
        Err(CanonicalError::Refused {
            code: crate::canonical::RefusalCode::RunResultsHaveProtectedReaders,
            ..
        })
    ));
    drop(selected);
    store.result_read_drain.drain().await.unwrap();
    store.forget_run_results(fence.run()).await.unwrap();
}

#[tokio::test]
async fn capture_opaque_public_ids_and_protection_tuple_keys_do_not_alias() {
    use crate::canonical_study_workbench::StudyWorkbenchFilter;
    let (store, _, revision) = fixture().await;
    store
        .create_study(
            "opaque-study",
            &request(&revision, "opaque-parent"),
            &[],
            &[occurrence(&revision, 0, vec![])],
            &|| false,
        )
        .await
        .unwrap();
    let scope = store
        .study_scope(&point_key("opaque-study", OccurrenceKey(0)))
        .await
        .unwrap();
    let claim = store
        .claim_study_point(
            &scope,
            None,
            "opaque-claim",
            "worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    fail(&store, &claim).await;
    let filter = StudyWorkbenchFilter {
        study: Some("opaque-study".into()),
        ..Default::default()
    };
    let first = store
        .capture_study_workbench("opaque:prefix", &filter, 1, 1)
        .await
        .unwrap();
    let second = store
        .capture_study_workbench("opaque", &filter, 1, 1)
        .await
        .unwrap();
    // These physical tuples collided under raw delimiter concatenation. All
    // inputs are admitted opaque strings; a UUID-only restriction is unnecessary.
    let mut keys=bounded_query(store.db.query("RETURN [fn::pse_workbench_v1::protection_key('opaque:prefix','study',$attempt),fn::pse_workbench_v1::protection_key('opaque','prefix:study',$attempt)];").bind(("attempt",claim.fence.attempt().to_owned()))).await.unwrap();
    let keys = keys.take::<Vec<String>>(0).unwrap();
    assert_ne!(keys[0], keys[1]);
    drop(first);
    store
        .result_read_drain
        .join_pending_releases()
        .await
        .unwrap();
    let mut references=bounded_query(store.db.query("SELECT capture FROM canonical_study_capture_protections WHERE capture IN ['opaque:prefix','opaque'];")).await.unwrap();
    let references = references.take::<Vec<Object>>(0).unwrap();
    assert_eq!(references.len(), 1);
    assert_eq!(
        references[0].get("capture"),
        Some(&Value::String("opaque".into()))
    );
    assert_eq!(
        second
            .capture
            .member_page(None, 1)
            .await
            .unwrap()
            .rows
            .len(),
        1
    );
    drop(second);
    store.result_read_drain.drain().await.unwrap();
}

#[tokio::test]
async fn capture_controlled_lost_cleanup_ack_recovers_the_original_release() {
    use crate::canonical_study_workbench::{StudyWorkbenchFilter, lose_capture_release_reply};
    let (store, _, revision) = fixture().await;
    store
        .create_study(
            "release-loss-study",
            &request(&revision, "release-loss-parent"),
            &[],
            &[occurrence(&revision, 0, vec![])],
            &|| false,
        )
        .await
        .unwrap();
    let before = store
        .canonical_study("release-loss-study")
        .await
        .unwrap()
        .unwrap();
    let filter = StudyWorkbenchFilter {
        study: Some("release-loss-study".into()),
        ..Default::default()
    };
    let selected = store
        .capture_study_workbench("release-loss-cut", &filter, 1, 1)
        .await
        .unwrap();
    lose_capture_release_reply("release-loss-cut");
    drop(selected);
    let failed = store.result_read_drain.drain().await.unwrap_err();
    assert!(failed.effect_is_unknown());
    assert!(
        matches!(failed,CanonicalError::ProtectionRelease{ref source} if matches!(source.as_ref(),CanonicalError::IncompleteResponse))
    );
    assert!(matches!(
        store
            .capture_study_workbench("release-loss-cut", &filter, 1, 1)
            .await,
        Err(CanonicalError::CaptureDraining { .. })
    ));
    store.recover_protection_releases().await.unwrap();
    store.result_read_drain.drain().await.unwrap();
    let mut recovered=bounded_query(store.db.query("SELECT state FROM ONLY type::record('canonical_study_captures','release-loss-cut'); SELECT key FROM canonical_study_capture_members WHERE capture='release-loss-cut';")).await.unwrap();
    assert_eq!(
        recovered
            .take::<Option<Object>>(0)
            .unwrap()
            .unwrap()
            .get("state"),
        Some(&Value::String("released".into()))
    );
    assert!(recovered.take::<Vec<Object>>(1).unwrap().is_empty());
    let after = store
        .canonical_study("release-loss-study")
        .await
        .unwrap()
        .unwrap();
    assert!(pse_model::SemanticEq::semantic_eq(&before, &after));
}

#[tokio::test]
async fn current_summary_and_fixed_viewer_follow_preserve_scientific_classification() {
    use crate::canonical_study_workbench::{CurrentSnapshot, FollowSelection, FollowUpdate};
    let (store, _, revision) = fixture().await;
    let study = "native-summary-follow";
    let points = (0..65)
        .map(|n| occurrence(&revision, n, vec![]))
        .collect::<Vec<_>>();
    store
        .create_study(
            study,
            &request(&revision, "native-summary-parent"),
            &[],
            &points,
            &|| false,
        )
        .await
        .unwrap();
    let summary = store.study_summary(study).await.unwrap().unwrap();
    assert_eq!(summary.header.point_count, 65);
    assert_eq!(
        summary.groups.iter().map(|group| group.count).sum::<u64>(),
        65
    );
    assert!(
        summary
            .groups
            .iter()
            .all(|group| !group.usable && !group.settled)
    );
    assert!(
        store
            .study_summary("absent-summary")
            .await
            .unwrap()
            .is_none()
    );
    #[derive(Default)]
    struct Members(
        Vec<
            crate::canonical_study_workbench::CapturedStudyRow<
                crate::canonical_study_workbench::StudyCohortMember,
            >,
        >,
    );
    impl crate::canonical_study_workbench::CaptureMemberConsumer for Members {
        fn consume(
            &mut self,
            rows: Vec<
                crate::canonical_study_workbench::CapturedStudyRow<
                    crate::canonical_study_workbench::StudyCohortMember,
                >,
            >,
        ) -> Result<(), CanonicalError> {
            self.0.extend(rows);
            Ok(())
        }
    }
    let captured = store
        .capture_study_workbench(
            "native-streamed-capture",
            &crate::canonical_study_workbench::StudyWorkbenchFilter {
                study: Some(study.into()),
                ..Default::default()
            },
            1,
            65,
        )
        .await
        .unwrap();
    let streamed = captured
        .capture
        .stream_members(65, 1 << 20, Members::default())
        .await
        .unwrap();
    assert_eq!(streamed.0.len(), 65);
    assert_eq!(streamed.0[64].position, 64);
    assert!(streamed.0.iter().all(|row| !row.value.settled));
    drop(captured);
    let mut follow = store
        .follow_current(
            FollowSelection::Study {
                study: study.into(),
            },
            std::time::Instant::now() + Duration::from_secs(20),
        )
        .await
        .unwrap();
    let FollowUpdate::Snapshot {
        snapshot: CurrentSnapshot::Study(initial),
        next_revalidation_at,
        ..
    } = follow.next().await.unwrap()
    else {
        panic!("initial identified snapshot missing")
    };
    assert!(!initial.header.cancelled);
    assert!(next_revalidation_at > initial.observed_as_of);
    assert!(bounded_query(store.db.query("UPDATE ONLY type::record('canonical_studies',$study) SET cancelled=true; THROW 'rollback';").bind(("study",study.to_owned()))).await.is_err());
    assert!(
        tokio::time::timeout(Duration::from_millis(150), follow.next())
            .await
            .is_err(),
        "rolled-back writes must not invalidate the selected root"
    );
    let boundary = std::time::Instant::now();
    follow.cross_principal_boundary_after_next_read(
        boundary + Duration::from_secs(1),
        boundary + Duration::from_millis(1100),
    );
    // This fixture changes only the joined run header. No study/point row is
    // touched, so its exact indirect dependency must supply the invalidation.
    bounded_query(
        store
            .db
            .query(
                "UPDATE ONLY type::record('canonical_runs',$run) SET terminal_class='cancelled';",
            )
            .bind(("run", initial.header.run.clone())),
    )
    .await
    .unwrap();
    assert!(
        matches!(follow.next().await.unwrap(), FollowUpdate::Unavailable),
        "a fully received snapshot crossing its principal boundary must be discarded before publication"
    );
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let FollowUpdate::Snapshot {
                snapshot: CurrentSnapshot::Study(current),
                ..
            } = follow.next().await.unwrap()
            {
                if current.header.outcome
                    == Some(pse_model::generated::enums::CanonicalTerminalOutcome::Cancelled)
                {
                    assert!(!current.header.cancelled);
                    break;
                }
            }
        }
    })
    .await
    .expect("exact linked run transition must invalidate study header");
    follow.lose_linked_run_after_next_read();
    bounded_query(
        store
            .db
            .query("UPDATE ONLY type::record('canonical_runs',$run) SET terminal_class='failed';")
            .bind(("run", initial.header.run.clone())),
    )
    .await
    .unwrap();
    assert!(
        matches!(follow.next().await.unwrap(), FollowUpdate::Unavailable),
        "losing the linked-run channel during a read must withdraw currentness"
    );
    let FollowUpdate::Snapshot {
        snapshot: CurrentSnapshot::Study(recovered),
        ..
    } = follow.next().await.unwrap()
    else {
        panic!("lost dependency must be registered again before resnapshot")
    };
    assert_eq!(
        recovered.header.outcome,
        Some(pse_model::generated::enums::CanonicalTerminalOutcome::Failed)
    );
    store.cancel_study(study).await.unwrap();
    let observed = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let FollowUpdate::Snapshot {
                snapshot: CurrentSnapshot::Study(current),
                ..
            } = follow.next().await.unwrap()
            {
                if current.header.cancelled {
                    break;
                }
            }
        }
    })
    .await;
    assert!(
        observed.is_ok(),
        "stable root invalidation should wake the follow"
    );
    follow.close().await.unwrap();
    drop(follow);
    store.remove_isolated_fixture().await.unwrap();
}
