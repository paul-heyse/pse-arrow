// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit scientific history withdrawal and restartable bounded result cleanup.

use crate::canonical_script::{Envelope, GuardedScript, ResultSlot};
use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, bounded_query, protected_query},
    canonical_codec,
};
use surrealdb_types::Object;

/// One bounded cleanup operation after explicit run retirement.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ResultReclamationPage {
    /// All result payloads have been reclaimed; immutable lifecycle receipts remain.
    pub complete: bool,
    /// Original scientific IPC or seed blocks removed in this operation.
    pub batches: u64,
    /// Derived scalar/output indexes or seed descriptors removed.
    pub indexes: u64,
    /// Empty result-set memberships removed.
    pub sets: u64,
}

fn identity(key: &str) -> Result<(), CanonicalError> {
    if key.is_empty() || key.len() > 4096 {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}

impl CanonicalStore {
    // Only fixed local queries select original source owners; no caller-supplied
    // table/field or inferred result unit grants reclamation authority. Reading
    // this hint is postcommit: unavailable detail defers GC, never the mutation.
    async fn reclaim_result_source_after_commit(
        &self,
        key: &str,
        query: &'static str,
        deadline: tokio::time::Instant,
    ) {
        crate::canonical_retention::postcommit_cleanup(
            deadline,
            "canonical_result_retention::postcommit_source_owner_deferred",
            async {
                let problem = crate::canonical::transport::metadata_scope(
                    crate::canonical::MetadataResponse::Header,
                    async {
                        let mut response = bounded_query(
                            self.db
                                .metadata_query(crate::canonical::MetadataResponse::Header, query)
                                .bind(("key", key.to_owned())),
                        )
                        .await?;
                        let row = response.take::<Option<Object>>(0)?;
                        row.map(|mut row| {
                            canonical_codec::decode_string(canonical_codec::required(
                                &mut row, "problem",
                            )?)
                            .map_err(CanonicalError::from)
                        })
                        .transpose()
                    },
                )
                .await;
                match problem {
                    Ok(Some(problem)) => self.reclaim_source_after_commit(&problem, deadline).await,
                    Ok(None) => {}
                    Err(error) => return Err(error),
                }
                Ok(())
            },
        )
        .await;
    }

    /// Withdraw a terminal study's retention obligation without deleting its
    /// scientific occurrence and outcome receipts. Individual runs are retired
    /// explicitly, and protected readers and retained analyses still prevent it.
    pub async fn forget_study_results(&self, study: &str) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let deadline = original_deadline(crate::canonical::REQUEST_TIMEOUT);
        within_clock(deadline, async {
            identity(study)?;
            self.ensure_writes()?;
            loop {
                let mut response=protected_query("canonical_result_retention::forget_study_results", || {
                    Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<bool>>("","fn::pse_retention_v1::forget_study($pse_rpc_expires_at, $study)")?).bind(("study",study.to_owned())))
                }).await?;
                if ResultSlot::<Option<bool>>::returned().take(&mut response)?.ok_or(CanonicalError::IncompleteResponse)? { break; }
            }
            Ok(())
        }).await?;
        self.reclaim_result_source_after_commit(study, "SELECT problem FROM ONLY type::record('canonical_studies',$key);", deadline).await;
        Ok(())
            }).await
    }
    /// Retire an analysis and remove one bounded page of owned state.
    /// False means cleanup or immutable creation-window settlement remains pending.
    /// Unknown keys refuse; absence alone is not proof of settled creation authority.
    pub async fn forget_analysis_results(&self, analysis: &str) -> Result<bool, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let deadline = original_deadline(crate::canonical::REQUEST_TIMEOUT);
        let complete = within_clock(deadline, async {
        identity(analysis)?;
        self.ensure_writes()?;
        let mut response = protected_query("canonical_result_retention::forget_analysis_results", || {
            Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("","fn::pse_retention_v1::forget_analysis($pse_rpc_expires_at,$analysis)")?)
                .bind(("analysis", analysis.to_owned())))
        }).await?;
        let mut row = ResultSlot::<Option<Object>>::returned().take(&mut response)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        let complete = canonical_codec::decode_boolean(canonical_codec::required(&mut row,"complete")?)?;
        Ok(complete)
        }).await?;
        self.reclaim_result_source_after_commit(analysis, "SELECT primary_problem AS problem FROM ONLY type::record('canonical_analyses',$key);", deadline).await;
        Ok(complete)
            }).await
    }
    /// Resolve an uncertain final acknowledgement using the original immutable intent.
    /// This does not authorize creation or renew its expiry.
    pub async fn settle_analysis(
        &self,
        intent: &crate::canonical_analyses::Analysis,
    ) -> Result<bool, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        self.ensure_writes()?;
        let row = crate::generated::surreal::encode_canonical_analyses(intent)?;
        let mut response = protected_query("canonical_result_retention::settle_analysis", || {
            Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("","fn::pse_retention_v1::settle_analysis($pse_rpc_expires_at,$intent)")?)
                .bind(("intent", row.clone())))
        }).await?;
        let mut row = ResultSlot::<Option<Object>>::returned().take(&mut response)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        Ok(canonical_codec::decode_boolean(canonical_codec::required(
            &mut row, "complete",
        )?)?)
            }).await
    }
    /// Irreversibly withdraw scientific payloads only after recovery is complete
    /// and no live reader, retained study or analysis needs them. This atomically
    /// fences future claims/reads and releases every selected run source root.
    /// Repeating it resumes the same retirement after an uncertain acknowledgment.
    pub async fn forget_run_results(
        &self,
        run: &str,
    ) -> Result<ResultReclamationPage, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let deadline = original_deadline(crate::canonical::REQUEST_TIMEOUT);
        within_clock(
            deadline,
            async {
                identity(run)?;
                self.ensure_writes()?;
                loop {
                    let mut response = protected_query("canonical_result_retention::forget_run_results", || {
                        Ok(self.db.query("RETURN fn::pse_retention_v1::forget_run($pse_rpc_expires_at,$run);")
                            .bind(("run",run.to_owned())))
                    }).await?;
                    let row=response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse)?;
                    if row.get("pending")==Some(&surrealdb_types::Value::Bool(true)) { continue; }
                    if row.get("run")!=Some(&surrealdb_types::Value::String(run.into())) {
                        return Err(CanonicalError::IncompleteResponse);
                    }
                    return Ok(());
                }
            },
        )
        .await?;
        self.reclaim_result_source_after_commit(run, "SELECT problem FROM ONLY type::record('canonical_runs',$key);", deadline).await;
        Ok(crate::canonical_retention::postcommit_cleanup(deadline,
            "canonical_result_retention::postcommit_result_page_deferred",
            self.reclaim_result_page(run)).await)
            }).await
    }
    /// Reclaim at most one payload and 64 indexes of each supported class. A
    /// persistent generation cursor preserves progress across interruption.
    /// No elapsed-age policy deletes deliberately retained scientific history.
    pub async fn reclaim_result_page(
        &self,
        run: &str,
    ) -> Result<ResultReclamationPage, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            identity(run)?;
            self.ensure_writes()?;
            let mut response =
                protected_query("canonical_result_retention::reclaim_result_page", || {
                    Ok(self
                        .db
                        .query("RETURN fn::pse_retention_v1::collect_run($pse_rpc_expires_at, $run);")
                        .bind(("run", run.to_owned())))
                })
                .await?;
            let mut row = response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?;
            Ok(ResultReclamationPage {
                complete: canonical_codec::decode_boolean(canonical_codec::required(
                    &mut row, "complete",
                )?)?,
                batches: canonical_codec::decode_uint(canonical_codec::required(&mut row, "batches")?)?,
                indexes: canonical_codec::decode_uint(canonical_codec::required(&mut row, "indexes")?)?,
                sets: canonical_codec::decode_uint(canonical_codec::required(&mut row, "sets")?)?,
            })
        }).await
            }).await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    unsafe_code,
    reason = "controlled storage fixture supplies explicit scientific terminal classification"
)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "isolated retention fixtures and bounded cleanup assertions fail the test on unexpected results"
)]
mod canonical_result_retention_server_unit {
    use super::*;
    use crate::{
        canonical::{CanonicalOptions, bounded_query, checked},
        canonical_execution::{RunRequest, TerminalClass},
        generated::surreal as wire,
    };
    use std::{path::Path, time::Duration};

    async fn fixture() -> (CanonicalStore, String) {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("supervised canonical fixture required");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_result_retention_{}",
            uuid::Uuid::new_v4().simple()
        );
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        (store, options.database)
    }
    async fn request(store: &CanonicalStore, key: &str) -> RunRequest {
        RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: key.into(),
            revision: store
                .edit(key, None, &format!("source-{key}"), &[])
                .await
                .unwrap(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        }
    }
    async fn finish(store: &CanonicalStore, request: &RunRequest) -> String {
        store.begin_run(request).await.unwrap();
        let fence = store
            .claim_run(
                &request.key,
                &format!("claim-{}", request.key),
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        for ordinal in 0..3 {
            store
                .append_result_batch(
                    &fence,
                    &format!("append-{}-{ordinal}", request.key),
                    "values",
                    ordinal,
                    crate::canonical_execution::ResultPayload::new([ordinal as u8, 17].to_vec())
                        .unwrap(),
                    1,
                )
                .await
                .unwrap();
        }
        let closed = store
            .close_result_ingestion(&fence, &format!("close-{}", request.key))
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: the fixture owns these three synthetic batches and admits their partial classification and completion.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    &format!("seal-{}", request.key),
                    TerminalClass::Partial,
                    &[42],
                    crate::canonical_execution::SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        fence.attempt().into()
    }
    async fn cleanup(store: &CanonicalStore, run: &str) -> (u64, u64) {
        let mut total = (0, 0);
        for _ in 0..32 {
            let page = store.reclaim_result_page(run).await.unwrap();
            total.0 += page.batches;
            total.1 += page.sets;
            if page.complete {
                return total;
            }
        }
        panic!("bounded fixture cleanup did not finish")
    }
    #[tokio::test]
    async fn project_research_reference_retains_exact_closed_results() {
        let (store, _database) = fixture().await;
        let request = request(&store, "project-reference").await;
        let attempt = finish(&store, &request).await;
        store
            .create_project("create-project", "project", "Research")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        store
            .retain_research_reference(
                "reference",
                &project,
                "research",
                &request.revision.key,
                Some((&request.key, &attempt)),
                None,
            )
            .await
            .unwrap();
        assert!(matches!(
            store.forget_run_results(&request.key).await,
            Err(CanonicalError::Refused {
                code: crate::canonical::RefusalCode::ProjectResearchRetainsResults,
                ..
            })
        ));
        assert!(
            store
                .canonical_result_manifest(&attempt)
                .await
                .unwrap()
                .is_some()
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn retired_results_respect_readers_resume_cleanup_and_preserve_receipts() {
        let (store, database) = fixture().await;
        let mut request = request(&store, "retained").await;
        request.sources.push(
            store
                .edit("physical", None, "physical-1", &[])
                .await
                .unwrap(),
        );
        let attempt = finish(&store, &request).await;
        let read = store.read_results(&request.key, &attempt).await.unwrap();
        let payload = store
            .result_payload(&read, &read.sets()[0].key, 0)
            .await
            .unwrap();
        assert!(store.forget_run_results(&request.key).await.is_err());
        drop(read);
        assert!(store.forget_run_results(&request.key).await.is_err());
        assert_eq!(payload.batch.payload.as_slice(), [0, 17]);
        drop(payload);
        // The last returned buffer releases its admitted protection asynchronously.
        let mut retired = None;
        for _ in 0..32 {
            if let Ok(page) = store.forget_run_results(&request.key).await {
                retired = Some(page);
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let retired = retired.expect("the drained reader permits retirement");
        assert_eq!(retired.batches, 1, "retirement drives one foreground page");
        assert!(store.read_results(&request.key, &attempt).await.is_err());
        assert!(store.begin_run(&request).await.is_err());
        assert!(
            store
                .claim_run(
                    &request.key,
                    "claim-retained",
                    "worker",
                    Duration::from_secs(60)
                )
                .await
                .is_err()
        );
        assert_eq!(
            store
                .reclaim_result_page(&request.key)
                .await
                .unwrap()
                .batches,
            1
        );
        // A reopened handle continues the recorded cleanup cursor without re-solving.
        assert_eq!(cleanup(&store.clone(), &request.key).await, (1, 1));
        assert!(
            store
                .reclaim_result_page(&request.key)
                .await
                .unwrap()
                .complete
        );
        assert_eq!(
            store
                .canonical_attempt(&attempt)
                .await
                .unwrap()
                .unwrap()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("partial")
        );
        assert!(
            store
                .canonical_attempt(&attempt)
                .await
                .unwrap()
                .unwrap()
                .completion
                .is_none()
        );
        let compacted = store.canonical_attempt(&attempt).await.unwrap().unwrap();
        assert_eq!(
            compacted.phase,
            pse_model::generated::enums::CanonicalAttemptPhase::TerminalCompacted
        );
        assert!(compacted.terminal());
        let resurrected=bounded_query(store.db.query("UPDATE ONLY type::record('canonical_attempts',$attempt) SET completion=$completion;").bind(("attempt",attempt.clone())).bind(("completion",surrealdb_types::Bytes::from(vec![9_u8])))).await.unwrap_err();
        assert!(matches!(resurrected, CanonicalError::Driver(_)));
        assert!(
            store
                .canonical_attempt(&attempt)
                .await
                .unwrap()
                .unwrap()
                .completion
                .is_none()
        );
        let mut roots = store
            .db
            .query("SELECT key FROM canonical_roots WHERE owner_kind='run' AND owner=$run;")
            .bind(("run", request.key.as_str().to_owned()))
            .await
            .map_err(CanonicalError::from)
            .and_then(checked)
            .unwrap();
        assert!(roots.take::<Vec<Object>>(0).unwrap().is_empty());
        assert_eq!(
            store
                .canonical_run(&request.key)
                .await
                .unwrap()
                .unwrap()
                .interpretation,
            wire::INTERPRETATION
        );
        assert_eq!(store.database(), database);
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn multipage_result_read_hands_protection_to_analysis_before_retirement() {
        use crate::{
            canonical_analyses::AnalysisNode,
            canonical_execution::{result_batch_key, result_payload_digest, result_set_key},
        };
        use pse_model::generated::runtime::canonical_result_blocks::Row as BlockMetadata;
        let (store, database) = fixture().await;
        let request = request(&store, "multipage").await;
        store.begin_run(&request).await.unwrap();
        let fence = store
            .claim_run(
                &request.key,
                "claim-multipage",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let set = result_set_key(fence.attempt(), "trajectory");
        // Cross the real 64-member metadata page boundary, not merely a payload
        // chunk boundary in one database response.
        for ordinal in 0..65_u64 {
            let payload = vec![ordinal as u8, 17];
            let key = result_batch_key(fence.attempt(), &set, ordinal);
            let block = BlockMetadata {
                key: key.clone(),
                result_set: set.clone(),
                batch: key,
                output: "temperature".into(),
                partition: "time".into(),
                ordinal,
                start: ordinal,
                end: ordinal + 1,
                rows: 1,
                columns: 1,
                coordinate_min: Some(ordinal as f64),
                coordinate_max: Some(ordinal as f64),
                payload_bytes: payload.len() as u64,
                payload_digest: result_payload_digest(&payload),
                statistics: None,
                interpretation: wire::INTERPRETATION.into(),
            };
            store
                .append_result_block(
                    &fence,
                    &format!("append-multipage-{ordinal}"),
                    "trajectory",
                    ordinal,
                    crate::canonical_execution::ResultPayload::new(payload).unwrap(),
                    1,
                    &block,
                )
                .await
                .unwrap();
        }
        // Deliberate privileged metadata corruption, after ordinary append has
        // enforced its admitted-file proof: exercise the transport's legal blob extent
        // and conservative unknown-statistics behavior. These bytes are not an
        // admitted-file proof and must never be submitted as exact statistics.
        store.db.query("UPDATE canonical_result_blocks SET statistics=$statistics WHERE result_set=$set RETURN NONE;")
            .bind(("statistics", surrealdb_types::Bytes::from(vec![0xff; pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES])))
            .bind(("set", set.clone()))
            .await.unwrap().check().unwrap();
        let closed = store
            .close_result_ingestion(&fence, "close-multipage")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this transport fixture asserts partial observations, not science.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal-multipage",
                TerminalClass::Partial,
                &[42],
                crate::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let read = store
            .read_results(&request.key, fence.attempt())
            .await
            .unwrap();
        let joined = store
            .result_joined_block_page(&read, &set, "temperature", "time", 0, 65, None)
            .await
            .unwrap();
        assert!(!joined.exhausted);
        assert!(
            !joined.blocks.is_empty() && joined.blocks.len() < 64,
            "complete member bytes stop this legal page before the row ceiling"
        );
        assert_eq!(joined.blocks.len(), joined.batches.len());
        let mut ordinals = joined
            .blocks
            .iter()
            .map(|block| block.ordinal)
            .collect::<Vec<_>>();
        let mut next = joined.next;
        loop {
            let joined = store
                .result_joined_block_page(&read, &set, "temperature", "time", 0, 65, next)
                .await
                .unwrap();
            ordinals.extend(joined.blocks.iter().map(|block| block.ordinal));
            next = joined.next.or(next);
            if joined.exhausted {
                break;
            }
        }
        assert_eq!(
            ordinals,
            (0..65).collect::<Vec<_>>(),
            "resuming a short byte-limited page loses no member"
        );
        let (first, retired) = tokio::join!(
            store.result_joined_block_page(&read, &set, "temperature", "time", 0, 65, None),
            store.forget_run_results(&request.key)
        );
        let first = first.unwrap();
        assert!(!first.blocks.is_empty() && first.blocks.len() < 64);
        assert!(!first.exhausted);
        assert_eq!(first.next, first.blocks.last().map(|block| block.ordinal));
        assert!(retired.is_err());
        let escaped = store
            .result_block(&read, first.blocks[0].clone())
            .await
            .unwrap();
        // Advance the source head and relinquish default history: the protected
        // read and resulting analysis must preserve this exact older source.
        store
            .edit(
                &request.revision.problem,
                Some(&request.revision.key),
                "multipage-source-next",
                &[],
            )
            .await
            .unwrap();
        store.forget_history(&request.revision).await.unwrap();
        let mut artifact = store
            .new_artifact(&request.revision, "analysis-fixture", vec![1], 1, 2)
            .await
            .unwrap();
        let block = crate::canonical_artifacts::artifact_block(
            &artifact.key,
            0,
            &set,
            vec![0, 17],
            &escaped.batch.digest,
            1,
        );
        crate::canonical_artifacts::seal_artifact_request(
            &mut artifact,
            std::slice::from_ref(&request.revision),
            std::slice::from_ref(&read),
            std::slice::from_ref(&block),
            "user",
            "multipage-artifact",
        );
        store
            .persist_artifact(
                &artifact,
                std::slice::from_ref(&request.revision),
                std::slice::from_ref(&read),
                &[block],
                "user",
                "multipage-artifact",
            )
            .await
            .unwrap();
        let artifact_read = store.read_artifact(&artifact.key).await.unwrap();
        let mut header = store
            .new_analysis(
                &request.revision,
                "bounded-reader-handoff-fixture:v2",
                vec![1],
                manifest.row().key.clone(),
                1,
                0,
            )
            .await
            .unwrap();
        let node = AnalysisNode {
            key: "multipage-node".into(),
            analysis: header.key.clone(),
            semantic: "temperature".into(),
            kind: "result".into(),
        };
        let nodes = [node];
        crate::canonical_analyses::seal_analysis_request(
            &mut header,
            std::slice::from_ref(&request.revision),
            std::slice::from_ref(&artifact_read),
            &nodes,
            &[],
        );
        let (analysis, retired) = tokio::join!(
            store.persist_analysis(
                &header,
                std::slice::from_ref(&request.revision),
                std::slice::from_ref(&artifact_read),
                &nodes,
                &[]
            ),
            store.forget_run_results(&request.key)
        );
        assert!(analysis.unwrap().active);
        assert!(retired.is_err());
        let second = store
            .result_joined_block_page(&read, &set, "temperature", "time", 0, 65, first.next)
            .await
            .unwrap();
        assert_eq!(
            second
                .blocks
                .iter()
                .map(|block| block.ordinal)
                .collect::<Vec<_>>(),
            (first.next.unwrap() + 1..65).collect::<Vec<_>>()
        );
        assert_eq!(second.next, Some(64));
        assert!(second.exhausted);
        assert_eq!(
            store
                .result_block(&read, second.blocks.last().unwrap().clone())
                .await
                .unwrap()
                .batch
                .payload
                .as_slice(),
            &[64, 17]
        );
        assert!(
            store
                .result_joined_block_page(&read, &set, "temperature", "time", 0, 65, Some(64))
                .await
                .unwrap()
                .blocks
                .is_empty()
        );
        drop(read);
        assert!(store.forget_run_results(&request.key).await.is_err());
        assert_eq!(escaped.batch.payload.as_slice(), &[0, 17]);
        drop(escaped);
        // Join the result reader's exact cleanup without declaring the still
        // borrowed independent artifact acquisition globally drained.
        store
            .result_read_drain
            .join_pending_releases()
            .await
            .unwrap();
        let retired = store.forget_run_results(&request.key).await.unwrap();
        assert_eq!(
            artifact_read.block(0).await.unwrap().payload.as_slice(),
            &[0, 17]
        );
        artifact_read.release().await.unwrap();
        let mut roots = store
            .db
            .query(
                "SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$analysis;",
            )
            .bind(("analysis", header.key.clone()))
            .await
            .map_err(CanonicalError::from)
            .and_then(checked)
            .unwrap();
        assert_eq!(roots.take::<Vec<Object>>(0).unwrap().len(), 1);
        for _ in 0..4 {
            assert!(!store.forget_analysis_results(&header.key).await.unwrap());
        }

        let continued = store.forget_run_results(&request.key).await.unwrap();
        let mut reclaimed = retired.batches + continued.batches;
        for _ in 0..70 {
            let page = store.reclaim_result_page(&request.key).await.unwrap();
            reclaimed += page.batches;
            if page.complete {
                break;
            }
        }
        assert_eq!(reclaimed, 65);
        assert!(
            store
                .reclaim_result_page(&request.key)
                .await
                .unwrap()
                .complete
        );
        assert_eq!(
            store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("partial")
        );
        assert_eq!(store.database(), database);
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn result_read_and_retirement_conflict_on_exact_selection() {
        let (store, database) = fixture().await;
        for index in 0..8 {
            let request = request(&store, &format!("race-{index}")).await;
            let attempt = finish(&store, &request).await;
            let (read, retired) = tokio::join!(
                store.read_results(&request.key, &attempt),
                store.forget_run_results(&request.key)
            );
            assert!(!(read.is_ok() && retired.is_ok()));
            let mut foreground = retired.ok();
            if let Ok(read) = read {
                assert_eq!(
                    store
                        .result_payload(&read, &read.sets()[0].key, 0)
                        .await
                        .unwrap()
                        .batch
                        .payload
                        .as_slice(),
                    [0, 17]
                );
                drop(read);
                for _ in 0..32 {
                    if let Ok(page) = store.forget_run_results(&request.key).await {
                        foreground = Some(page);
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            let foreground = foreground.expect("retirement follows the drained reader");
            assert_eq!(foreground.batches, 1);
            let rest = cleanup(&store, &request.key).await;
            assert_eq!(
                (foreground.batches + rest.0, foreground.sets + rest.1),
                (3, 1)
            );
        }
        assert_eq!(store.database(), database);
        store.remove_isolated_fixture().await.unwrap();
    }
}
