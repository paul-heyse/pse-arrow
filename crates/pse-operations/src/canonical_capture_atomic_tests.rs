// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual selected-engine capture ordering, liveness and protection controls.

mod atomic_capture {
    use super::*;
    use crate::canonical_study_workbench::StudyWorkbenchFilter;

    async fn held(store: &CanonicalStore) -> crate::transport::client::HeldTransaction {
        use crate::transport::client::{Client, RequestContext, Root};
        let mut options = CanonicalOptions::from_state(store.deployment_state()).unwrap();
        options.database = store.database().to_owned();
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        let address = url::Url::parse(&options.endpoint)
            .unwrap()
            .socket_addrs(|| None)
            .unwrap()[0];
        let db = Client::connect(address, crate::canonical::MESSAGE_BYTES)
            .await
            .unwrap();
        db.signin(Root {
            username: options.username,
            password: options.password,
        })
        .request_context(RequestContext::control(deadline))
        .await
        .unwrap();
        db.use_ns(&options.namespace)
            .use_db(&options.database)
            .request_context(RequestContext::control(deadline))
            .await
            .unwrap();
        db.test_begin(deadline).await.unwrap()
    }

    async fn hold_capture(
        store: &CanonicalStore,
        key: &str,
        filter: &StudyWorkbenchFilter,
        maximum: usize,
        members: usize,
    ) -> crate::transport::client::HeldTransaction {
        let owner = store.registered_file_owner().await.unwrap();
        let transaction = held(store).await;
        let request = Bytes::from(serde_json::to_vec(&(1_u8, filter, maximum, members)).unwrap());
        transaction.query("LET $reply=fn::pse_workbench_v1::capture({expiry:time::micros()+30000000,key:$key,owner:$owner,request:$request,filter:$filter,maximum:$maximum,members:$members}); IF $reply.refusal!=NONE { THROW 'PSE_REFUSAL:'+$reply.refusal; }; RETURN $reply.receipt;")
            .bind(("key",key.to_owned())).bind(("owner",owner)).bind(("request",request))
            .bind(("filter",serde_json::to_value(filter).unwrap())).bind(("maximum",maximum)).bind(("members",members))
            .await.unwrap().check().unwrap();
        transaction
    }

    async fn create(store: &CanonicalStore, revision: &crate::canonical::Revision, study: &str) {
        let mut point = occurrence(revision, 0, vec![]);
        point.run.key = format!("{study}-point-run").as_str().into();
        store
            .create_study(
                study,
                &request(revision, &format!("{study}-parent")),
                &[],
                &[point],
                &|| false,
            )
            .await
            .unwrap();
    }

    fn peer(store: &CanonicalStore) -> CanonicalStore {
        let mut options = CanonicalOptions::from_state(store.deployment_state()).unwrap();
        options.database = store.database().to_owned();
        crate::testing::canonical_fixture_peer(store, &options).unwrap()
    }

    fn conflict(error: surrealdb_types::Error) {
        assert!(
            matches!(
                error.query_details(),
                Some(surrealdb_types::QueryError::TransactionConflict)
            ),
            "expected original selected-engine commit conflict: {error}"
        );
    }

    #[tokio::test]
    async fn capture_lost_ack_replays_exact_receipt_and_rejects_changed_body() {
        let (store, _, revision) = fixture().await;
        create(&store, &revision, "lost-ack-study").await;
        let filter = StudyWorkbenchFilter {
            study: Some("lost-ack-study".into()),
            ..Default::default()
        };
        crate::canonical_study_workbench::lose_capture_reply("lost-ack-cut");
        let first = store
            .capture_study_workbench("lost-ack-cut", &filter, 1, 1)
            .await
            .unwrap();
        store.cancel_study("lost-ack-study").await.unwrap();
        let replay = store
            .capture_study_workbench("lost-ack-cut", &filter, 1, 1)
            .await
            .unwrap();
        assert_eq!(first.header_count, replay.header_count);
        assert_eq!(first.member_count, replay.member_count);
        assert!(
            !replay.capture.header_page(None, 1).await.unwrap().rows[0]
                .value
                .cancelled,
            "receipt replay cannot reclassify later facts"
        );
        assert!(matches!(
            store
                .capture_study_workbench("lost-ack-cut", &filter, 2, 1)
                .await,
            Err(CanonicalError::OperationReused)
        ));
        assert_eq!(
            first.capture.member_page(None, 1).await.unwrap().rows.len(),
            1,
            "changed-body refusal must preserve original custody"
        );
        drop(first);
        drop(replay);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_exact_request_preserves_full_admitted_selector_encoding_extent() {
        let (store, _, _) = fixture().await;
        let filter = StudyWorkbenchFilter {
            study: Some("absent".into()),
            source: Some("\u{1}".repeat(4096)),
            physical_revision: Some("\u{1}".repeat(4096)),
            context: Some("\u{1}".repeat(4096)),
            ..Default::default()
        };
        assert!(
            serde_json::to_vec(&(1_u8, &filter, 1_usize, 1_usize))
                .unwrap()
                .len()
                > 64 * 1024
        );
        let captured = store
            .capture_study_workbench("escaped-request-cut", &filter, 1, 1)
            .await
            .unwrap();
        assert_eq!((captured.header_count, captured.member_count), (0, 0));
        drop(captured);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_future_member_and_selector_metadata_commit_before_capture_force_complete_retry()
     {
        for metadata in [false, true] {
            let (store, _, revision) = fixture().await;
            if metadata {
                create(&store, &revision, "future-member").await;
            }
            let filter = if metadata {
                StudyWorkbenchFilter {
                    research_reference: Some("future-dataset".into()),
                    ..Default::default()
                }
            } else {
                StudyWorkbenchFilter::default()
            };
            let pending = hold_capture(&store, "future-cut", &filter, 2, 2).await;
            if metadata {
                bounded_query(store.db.query("UPDATE ONLY type::record('canonical_runs','future-member-parent') SET research_dataset='future-dataset';")).await.unwrap();
            } else {
                create(&store, &revision, "future-member").await;
            }
            conflict(pending.commit().await.unwrap_err());
            let captured = store
                .capture_study_workbench("future-cut", &filter, 2, 2)
                .await
                .unwrap();
            assert_eq!(captured.header_count, 1);
            assert_eq!(captured.member_count, 1);
            assert_eq!(
                captured.capture.header_page(None, 2).await.unwrap().rows[0]
                    .value
                    .study,
                "future-member"
            );
            drop(captured);
            store.result_read_drain.drain().await.unwrap();
            store.remove_isolated_fixture().await.unwrap();
        }
    }

    #[tokio::test]
    async fn capture_stalled_transaction_preserves_unrelated_renewal_and_cancellation() {
        let (store, _, revision) = fixture().await;
        create(&store, &revision, "captured-study").await;
        create(&store, &revision, "unrelated-study").await;
        let run = request(&revision, "unrelated-worker");
        store.begin_run(&run).await.unwrap();
        let fence = store
            .claim_run(
                run.key.as_str(),
                "unrelated-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let filter = StudyWorkbenchFilter {
            study: Some("captured-study".into()),
            ..Default::default()
        };
        let pending = hold_capture(&store, "lease-cut", &filter, 1, 1).await;
        let before = {
            let mut response = bounded_query(store.db.query(
                "SELECT generation FROM ONLY type::record('canonical_guards','study-publication');",
            ))
            .await
            .unwrap();
            response.take::<Option<Object>>(0).unwrap().unwrap()
        };
        tokio::time::timeout(
            Duration::from_secs(3),
            store.renew_attempt(&fence, Duration::from_secs(60)),
        )
        .await
        .unwrap()
        .unwrap();
        let mut after = bounded_query(store.db.query(
            "SELECT generation FROM ONLY type::record('canonical_guards','study-publication');",
        ))
        .await
        .unwrap();
        assert_eq!(
            before,
            after.take::<Option<Object>>(0).unwrap().unwrap(),
            "lease-only renewal must not change the publication witness"
        );
        drop(after);
        pending.commit().await.unwrap();
        store.recover_study_capture("lease-cut").await.unwrap();
        let pending = hold_capture(&store, "cancel-cut", &filter, 1, 1).await;
        tokio::time::timeout(
            Duration::from_secs(3),
            store.cancel_study("unrelated-study"),
        )
        .await
        .unwrap()
        .unwrap();
        conflict(pending.commit().await.unwrap_err());
        let captured = store
            .capture_study_workbench("cancel-cut", &filter, 1, 1)
            .await
            .unwrap();
        assert!(
            !captured.capture.header_page(None, 1).await.unwrap().rows[0]
                .value
                .cancelled
        );
        drop(captured);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_committing_before_unrelated_lease_does_not_invalidate_that_lease() {
        let (store, _, revision) = fixture().await;
        create(&store, &revision, "lease-independent-study").await;
        let run = request(&revision, "lease-independent-worker");
        store.begin_run(&run).await.unwrap();
        let fence = store
            .claim_run(
                run.key.as_str(),
                "lease-independent-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let renewal = held(&store).await;
        renewal.query("RETURN fn::pse_execution_v1::renew(time::micros()+30000000,$run,$attempt,$generation,60000000);")
            .bind(("run",fence.run().to_owned())).bind(("attempt",fence.attempt().to_owned())).bind(("generation",codec::encode_uint(fence.generation()).unwrap()))
            .await.unwrap().check().unwrap();
        let filter = StudyWorkbenchFilter {
            study: Some("lease-independent-study".into()),
            ..Default::default()
        };
        let captured = store
            .capture_study_workbench("lease-independent-cut", &filter, 1, 1)
            .await
            .unwrap();
        renewal.commit().await.unwrap();
        drop(captured);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_rejects_caller_sql_or_function_fields_and_keeps_selector_strings_as_data() {
        let (store, _, revision) = fixture().await;
        create(&store, &revision, "input-control-study").await;
        let refused=bounded_query(store.db.query("RETURN fn::pse_workbench_v1::capture({expiry:time::micros()+30000000,key:'unissued-input-control',owner:'unissued',request:$request,maximum:1,members:1,filter:{study:NONE,project:NONE,branch:NONE,source:NONE,physical_revision:NONE,context:NONE,outcome:NONE,research_reference:NONE,caller_sql:'DELETE canonical_studies',caller_function:'fn::pse_execution_v1::cancel'}});").bind(("request",Bytes::from(vec![1])))).await.unwrap_err();
        assert!(
            matches!(
                refused,
                CanonicalError::Refused {
                    code: crate::canonical::RefusalCode::ControlRequestInvalid,
                    ..
                }
            ),
            "unexpected refusal: {refused}"
        );
        let filter = StudyWorkbenchFilter {
            study: Some("'; DELETE canonical_studies; fn::pse_execution_v1::cancel()".into()),
            ..Default::default()
        };
        let captured = store
            .capture_study_workbench("opaque-input-control", &filter, 1, 1)
            .await
            .unwrap();
        assert_eq!((captured.header_count, captured.member_count), (0, 0));
        assert!(
            store
                .canonical_study("input-control-study")
                .await
                .unwrap()
                .is_some()
        );
        drop(captured);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_and_absent_protection_retirement_are_fenced_in_both_commit_orders() {
        for capture_first in [false, true] {
            let (store, _, revision) = fixture().await;
            create(&store, &revision, "protected-study").await;
            let scope = store
                .study_scope(&point_key("protected-study", OccurrenceKey(0)))
                .await
                .unwrap();
            let claim = store
                .claim_study_point(
                    &scope,
                    None,
                    "protected-claim",
                    "worker",
                    Duration::from_secs(60),
                )
                .await
                .unwrap();
            fail(&store, &claim).await;
            // A Study retains its member runs until its actual terminal summary
            // has been sealed and that Study obligation is explicitly withdrawn.
            let summary = store
                .canonical_study_summary("protected-study")
                .await
                .unwrap()
                .unwrap();
            let parent = store
                .begin_study_finalization(&summary, "summary", Duration::from_secs(60))
                .await
                .unwrap()
                .unwrap();
            let closed = store
                .close_result_ingestion(&parent, "protected-summary-close")
                .await
                .unwrap();
            let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
            // SAFETY: this fixture's sole member has the admitted failed outcome
            // from fail(), and the summary records no successful scientific use.
            unsafe {
                store
                    .seal_study_summary(
                        "protected-study",
                        &manifest,
                        "protected-summary-seal",
                        TerminalClass::Failed,
                        b"failed fixture",
                    )
                    .await
            }
            .unwrap();
            assert!(matches!(
                store.forget_run_results(claim.fence.run()).await,
                Err(CanonicalError::Refused {
                    code: crate::canonical::RefusalCode::RunResultsAreRetainedByAStudy,
                    ..
                })
            ));
            store.forget_study_results("protected-study").await.unwrap();
            // Both held transactions start after owner admission; only the
            // capture/retirement commit order supplies the discriminating race.
            store.registered_file_owner().await.unwrap();
            let retirement = held(&store).await;
            retirement
                .query("RETURN fn::pse_retention_v1::forget_run(time::micros()+30000000,$run);")
                .bind(("run", claim.fence.run().to_owned()))
                .await
                .unwrap()
                .check()
                .unwrap();
            let filter = StudyWorkbenchFilter {
                study: Some("protected-study".into()),
                ..Default::default()
            };
            let captured = hold_capture(&store, "retirement-cut", &filter, 1, 1).await;
            if capture_first {
                captured.commit().await.unwrap();
                conflict(retirement.commit().await.unwrap_err());
                let selected = store
                    .capture_study_workbench("retirement-cut", &filter, 1, 1)
                    .await
                    .unwrap();
                assert!(
                    selected.capture.member_page(None, 1).await.unwrap().rows[0]
                        .value
                        .manifest
                        .is_some()
                );
                assert!(matches!(
                    store.forget_run_results(claim.fence.run()).await,
                    Err(CanonicalError::Refused {
                        code: crate::canonical::RefusalCode::RunResultsHaveProtectedReaders,
                        ..
                    })
                ));
                drop(selected);
                store.result_read_drain.drain().await.unwrap();
                store.forget_run_results(claim.fence.run()).await.unwrap();
            } else {
                retirement.commit().await.unwrap();
                conflict(captured.commit().await.unwrap_err());
                let selected = store
                    .capture_study_workbench("retirement-cut", &filter, 1, 1)
                    .await
                    .unwrap();
                assert!(
                    selected.capture.member_page(None, 1).await.unwrap().rows[0]
                        .value
                        .manifest
                        .is_none()
                );
                drop(selected);
                store.result_read_drain.drain().await.unwrap();
            }
            store.remove_isolated_fixture().await.unwrap();
        }
    }

    #[tokio::test]
    async fn capture_native_owner_drain_fences_stale_work_and_authenticates_abandoned_cleanup() {
        let (store, _, revision) = fixture().await;
        create(&store, &revision, "owner-study").await;
        let filter = StudyWorkbenchFilter {
            study: Some("owner-study".into()),
            ..Default::default()
        };
        let original = peer(&store);
        let frozen = hold_capture(&original, "abandoned-cut", &filter, 1, 1).await;
        frozen.commit().await.unwrap();
        assert!(matches!(
            store.recover_study_capture("abandoned-cut").await,
            Err(CanonicalError::Refused {
                code: crate::canonical::RefusalCode::StudyCaptureOriginalOwnerNotDrained,
                ..
            })
        ));
        let stale = hold_capture(&original, "stale-owner-cut", &filter, 1, 1).await;
        original
            .drain_owned_connection_until(tokio::time::Instant::now() + Duration::from_secs(10))
            .await
            .unwrap();
        conflict(stale.commit().await.unwrap_err());
        store.recover_study_capture("abandoned-cut").await.unwrap();
        let mut released=bounded_query(store.db.query("SELECT state FROM ONLY type::record('canonical_study_captures','abandoned-cut'); SELECT key FROM canonical_study_capture_members WHERE capture='abandoned-cut'; SELECT key FROM canonical_study_capture_protections WHERE capture='abandoned-cut';")).await.unwrap();
        assert_eq!(
            released
                .take::<Option<Object>>(0)
                .unwrap()
                .unwrap()
                .get("state"),
            Some(&Value::String("released".into()))
        );
        assert!(released.take::<Vec<Object>>(1).unwrap().is_empty());
        assert!(released.take::<Vec<Object>>(2).unwrap().is_empty());
        drop(released);
        assert!(matches!(
            store
                .capture_study_workbench("abandoned-cut", &filter, 1, 1)
                .await,
            Err(CanonicalError::OperationReused)
        ));
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_native_10000_phase_profile_rolls_back_complete_host_sql() {
        let (store, _, revision) = fixture().await;
        let points = (0..10_000)
            .map(|key| occurrence(&revision, key, vec![]))
            .collect::<Vec<_>>();
        store
            .create_study(
                "profile-study",
                &request(&revision, "profile-parent"),
                &[],
                &points,
                &|| false,
            )
            .await
            .unwrap();
        let filter = StudyWorkbenchFilter {
            study: Some("profile-study".into()),
            ..Default::default()
        };
        let owner = store.registered_file_owner().await.unwrap();
        let mut before = bounded_query(store.db.query(
            "RETURN (SELECT generation FROM ONLY type::record('canonical_guards','study-publication')).generation;"
        )).await.unwrap();
        let before = before.take::<Value>(0).unwrap();

        // Instrument the chosen transaction's exact SQL body, execute its real
        // schema/index writes, then cancel. This diagnostic is never a second
        // production path, and the phase measurements do not qualify throughput.
        let source = include_str!("../../pse-codegen/src/codegen/study-capture.surql");
        let mut sql = source
            .strip_prefix(
                "DEFINE FUNCTION fn::pse_workbench_v1::capture($capture:object) -> object ",
            )
            .unwrap()
            .trim_end()
            .strip_suffix(" PERMISSIONS NONE;")
            .unwrap()
            .to_owned();
        fn mark_before(sql: &mut String, anchor: &str, marker: &str) {
            assert_eq!(
                sql.matches(anchor).count(),
                1,
                "capture phase anchor changed"
            );
            *sql = sql.replacen(
                anchor,
                &format!("LET ${marker}=time::micros();\n{anchor}"),
                1,
            );
        }
        mark_before(&mut sql, "    LET $old=", "profile_started");
        mark_before(&mut sql, "    LET $facts=", "profile_before_facts");
        mark_before(
            &mut sql,
            "    IF $facts.refusal!=NONE",
            "profile_after_facts",
        );
        mark_before(
            &mut sql,
            "    IF array::len($facts.members)!=0",
            "profile_after_headers",
        );
        mark_before(
            &mut sql,
            "    IF array::len($facts.protections)!=0",
            "profile_after_members",
        );
        mark_before(&mut sql, "    LET $receipt=", "profile_after_protections");
        let result = "RETURN {receipt:$saved,refusal:NONE};";
        assert_eq!(sql.matches(result).count(), 1);
        sql = sql.replacen(result, "RETURN {receipt:$saved,refusal:NONE,profile:{preflight_us:<decimal>($profile_before_facts-$profile_started),facts_us:<decimal>($profile_after_facts-$profile_before_facts),headers_us:<decimal>($profile_after_headers-$profile_after_facts),members_us:<decimal>($profile_after_members-$profile_after_headers),protections_us:<decimal>($profile_after_protections-$profile_after_members),receipt_us:<decimal>(time::micros()-$profile_after_protections),total_us:<decimal>(time::micros()-$profile_started)}};", 1);
        let sql = format!(
            "LET $capture={{expiry:time::micros()+30000000,key:$key,owner:$owner,request:$request,filter:$filter,maximum:1,members:10000}}; RETURN {sql};"
        );
        let transaction = held(&store).await;
        let started = std::time::Instant::now();
        let response = transaction
            .query(sql)
            .bind(("key", "profile-cut"))
            .bind(("owner", owner))
            .bind((
                "request",
                Bytes::from(serde_json::to_vec(&(1_u8, &filter, 1_usize, 10_000_usize)).unwrap()),
            ))
            .bind(("filter", serde_json::to_value(&filter).unwrap()))
            .await;
        eprintln!(
            "G08 10000-member direct-host-SQL rollback surrogate: caller_elapsed={:?}, error={:?}",
            started.elapsed(),
            response.as_ref().err()
        );
        let mut response = response.unwrap().check().unwrap();
        let slot = response.num_statements() - 1;
        let mut reply = response.take::<Option<Object>>(slot).unwrap().unwrap();
        assert!(matches!(
            codec::required(&mut reply, "refusal").unwrap(),
            Value::None | Value::Null
        ));
        let Value::Object(mut receipt) = codec::required(&mut reply, "receipt").unwrap() else {
            panic!("complete diagnostic capture receipt required");
        };
        assert_eq!(
            codec::decode_uint(codec::required(&mut receipt, "header_count").unwrap()).unwrap(),
            1
        );
        assert_eq!(
            codec::decode_uint(codec::required(&mut receipt, "member_count").unwrap()).unwrap(),
            10_000
        );
        let Value::Object(mut profile) = codec::required(&mut reply, "profile").unwrap() else {
            panic!("bounded phase timings required");
        };
        eprintln!("G08 10000-member direct-host-SQL phases (microseconds): {profile:?}");
        let total = codec::decode_uint(codec::required(&mut profile, "total_us").unwrap()).unwrap();
        assert!(
            total > 0,
            "the complete 10000-member operation must have an observed duration"
        );
        for phase in [
            "preflight_us",
            "facts_us",
            "headers_us",
            "members_us",
            "protections_us",
            "receipt_us",
        ] {
            let elapsed =
                codec::decode_uint(codec::required(&mut profile, phase).unwrap()).unwrap();
            assert!(
                elapsed <= total,
                "phase exceeds complete measured host operation"
            );
        }
        assert!(reply.is_empty() && profile.is_empty());
        let mut installed = transaction.query("RETURN {headers:<decimal>array::len((SELECT key FROM canonical_study_capture_headers WHERE capture='profile-cut')),members:<decimal>array::len((SELECT key FROM canonical_study_capture_members WHERE capture='profile-cut')),protections:<decimal>array::len((SELECT key FROM canonical_study_capture_protections WHERE capture='profile-cut'))};")
            .await.unwrap().check().unwrap();
        let mut installed = installed.take::<Option<Object>>(0).unwrap().unwrap();
        for (kind, expected) in [("headers", 1), ("members", 10_000), ("protections", 0)] {
            assert_eq!(
                codec::decode_uint(codec::required(&mut installed, kind).unwrap()).unwrap(),
                expected
            );
        }
        transaction.cancel().await.unwrap();
        let mut after = bounded_query(store.db.query("RETURN (SELECT generation FROM ONLY type::record('canonical_guards','study-publication')).generation; SELECT * FROM ONLY type::record('canonical_study_captures','profile-cut'); SELECT key FROM canonical_study_capture_headers WHERE capture='profile-cut'; SELECT key FROM canonical_study_capture_members WHERE capture='profile-cut'; SELECT key FROM canonical_study_capture_protections WHERE capture='profile-cut';"))
            .await.unwrap();
        assert_eq!(
            after.take::<Value>(0).unwrap(),
            before,
            "cancel restores the publication witness"
        );
        assert!(after.take::<Option<Object>>(1).unwrap().is_none());
        for slot in 2..5 {
            assert!(after.take::<Vec<Object>>(slot).unwrap().is_empty());
        }
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_native_10000_members_preserves_complete_bounded_pages_and_receipt() {
        let (store, _, revision) = fixture().await;
        let points = (0..10_000)
            .map(|key| occurrence(&revision, key, vec![]))
            .collect::<Vec<_>>();
        store
            .create_study(
                "full-capture-study",
                &request(&revision, "full-capture-parent"),
                &[],
                &points,
                &|| false,
            )
            .await
            .unwrap();
        let filter = StudyWorkbenchFilter {
            study: Some("full-capture-study".into()),
            ..Default::default()
        };
        let captured = store
            .capture_study_workbench("full-capture-cut", &filter, 1, 10_000)
            .await
            .unwrap();
        assert_eq!((captured.header_count, captured.member_count), (1, 10_000));
        let mut after = None;
        let mut count = 0;
        loop {
            let page = captured.capture.member_page(after, 256).await.unwrap();
            if page.rows.is_empty() {
                break;
            }
            after = page.rows.last().map(|row| row.position);
            assert!(page.rows.iter().all(|row| row.value.manifest.is_none()));
            count += page.rows.len();
        }
        assert_eq!(count, 10_000);
        drop(captured);
        store.result_read_drain.drain().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn capture_cutover_refuses_an_unsettled_legacy_gate() {
        let (store, _, _) = fixture().await;
        bounded_query(store.db.query("CREATE ONLY type::record('canonical_study_capture_gate','current') CONTENT {capture:'legacy-original-owner'};")).await.unwrap();
        let refused=bounded_query(store.db.query("RETURN fn::pse_maintenance_v1::activate(time::micros()+30000000,NONE,'unissued','unissued','unissued','unissued','unissued','unissued','unissued',$request);").bind(("request",Bytes::from(vec![0])))).await.unwrap_err();
        assert!(matches!(
            refused,
            CanonicalError::Refused {
                code: crate::canonical::RefusalCode::StudyCaptureLegacyPendingNotDrained,
                ..
            }
        ));
        store.remove_isolated_fixture().await.unwrap();
    }
}
