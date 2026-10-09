// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit scientific history withdrawal and restartable bounded result cleanup.

use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, protected_query},
    canonical_codec,
};
use surrealdb::types::Object;

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
    /// Withdraw a terminal study's retention obligation without deleting its
    /// scientific occurrence and outcome receipts. Individual runs are retired
    /// explicitly, and protected readers and retained analyses still prevent it.
    pub async fn forget_study_results(&self, study: &str) -> Result<(), CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            identity(study)?;
            self.ensure_writes()?;
            protected_query("canonical_result_retention::forget_study_results", || {
                Ok(self
                    .db
                    .query("RETURN fn::pse_retention_v1::forget_study($pse_rpc_expires_at, $study);")
                    .bind(("study", study.to_owned())))
            })
            .await?;
            Ok(())
        }).await
    }
    /// Withdraw a derived analysis's result retention. Source roots may then be
    /// released explicitly; the original method and input lineage remain receipts.
    pub async fn forget_analysis_results(&self, analysis: &str) -> Result<(), CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            identity(analysis)?;
            self.ensure_writes()?;
            protected_query(
                "canonical_result_retention::forget_analysis_results",
                || {
                    Ok(self
                        .db
                        .query("RETURN fn::pse_retention_v1::forget_analysis($pse_rpc_expires_at, $analysis);")
                        .bind(("analysis", analysis.to_owned())))
                },
            )
            .await?;
            Ok(())
        }).await
    }
    /// Irreversibly withdraw scientific payloads only after recovery is complete
    /// and no live reader, retained study or analysis needs them. This atomically
    /// fences future claims/reads and releases every selected run source root.
    /// Repeating it resumes the same retirement after an uncertain acknowledgment.
    pub async fn forget_run_results(&self, run: &str) -> Result<(), CanonicalError> {
        within_clock(
            original_deadline(crate::canonical::REQUEST_TIMEOUT),
            async {
                identity(run)?;
                self.ensure_writes()?;
                protected_query("canonical_result_retention::forget_run_results", || {
                    Ok(self
                        .db
                        .query(
                            "RETURN fn::pse_retention_v1::forget_run($pse_rpc_expires_at, $run);",
                        )
                        .bind(("run", run.to_owned())))
                })
                .await?;
                Ok(())
            },
        )
        .await
    }
    /// Reclaim at most one payload and 64 indexes of each supported class. A
    /// persistent generation cursor preserves progress across interruption.
    /// No elapsed-age policy deletes deliberately retained scientific history.
    pub async fn reclaim_result_page(
        &self,
        run: &str,
    ) -> Result<ResultReclamationPage, CanonicalError> {
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
        canonical::{CanonicalOptions, checked},
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
                    &[ordinal as u8, 17],
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
        let read = store
            .read_results(&request.key, &attempt, Duration::from_secs(60))
            .await
            .unwrap();
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
        let mut retired = false;
        for _ in 0..32 {
            if store.forget_run_results(&request.key).await.is_ok() {
                retired = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(retired);
        assert!(
            store
                .read_results(&request.key, &attempt, Duration::from_secs(60))
                .await
                .is_err()
        );
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
        assert_eq!(cleanup(&store.clone(), &request.key).await, (2, 1));
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
                .as_deref(),
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
        let mut roots = store
            .db
            .query("SELECT key FROM canonical_roots WHERE owner_kind='run' AND owner=$run;")
            .bind(("run", request.key.clone()))
            .await
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
            canonical_analyses::{Analysis, AnalysisNode},
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
                interpretation: wire::INTERPRETATION.into(),
            };
            store
                .append_result_block(
                    &fence,
                    &format!("append-multipage-{ordinal}"),
                    "trajectory",
                    ordinal,
                    &payload,
                    1,
                    &block,
                )
                .await
                .unwrap();
        }
        let closed = store
            .close_result_ingestion(&fence, "close-multipage")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this transport fixture asserts partial observations, not science.
        unsafe { store.seal_attempt(&manifest, "seal-multipage", TerminalClass::Partial, &[42]) }
            .await
            .unwrap();
        let read = store
            .read_results(&request.key, fence.attempt(), Duration::from_secs(60))
            .await
            .unwrap();
        let (first, retired) = tokio::join!(
            store.result_block_page(&read, &set, "temperature", "time", 0, 65, None),
            store.forget_run_results(&request.key)
        );
        let first = first.unwrap();
        assert_eq!(first.len(), 64);
        assert!(retired.is_err());
        let escaped = store.result_block(&read, first[0].clone()).await.unwrap();
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
        let header = Analysis {
            key: "multipage-analysis".into(),
            revision: request.revision.key.clone(),
            method: "bounded-reader-handoff-fixture:v1".into(),
            configuration: vec![1].into(),
            input_digest: manifest.row().key.clone(),
            interpretation: wire::INTERPRETATION.into(),
            node_count: 1,
            edge_count: 0,
            active: false,
        };
        let node = AnalysisNode {
            key: "multipage-node".into(),
            analysis: header.key.clone(),
            semantic: "temperature".into(),
            kind: "result".into(),
        };
        let nodes = [node];
        let (analysis, retired) = tokio::join!(
            store.persist_analysis(
                &header,
                std::slice::from_ref(&request.revision),
                std::slice::from_ref(&read),
                &nodes,
                &[]
            ),
            store.forget_run_results(&request.key)
        );
        assert!(analysis.unwrap().active);
        assert!(retired.is_err());
        let second = store
            .result_block_page(
                &read,
                &set,
                "temperature",
                "time",
                0,
                65,
                Some(first.last().unwrap().ordinal),
            )
            .await
            .unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].ordinal, 64);
        assert_eq!(
            store
                .result_block(&read, second[0].clone())
                .await
                .unwrap()
                .batch
                .payload
                .as_slice(),
            &[64, 17]
        );
        assert!(
            store
                .result_block_page(&read, &set, "temperature", "time", 0, 65, Some(64))
                .await
                .unwrap()
                .is_empty()
        );
        drop(read);
        assert!(store.forget_run_results(&request.key).await.is_err());
        assert_eq!(escaped.batch.payload.as_slice(), &[0, 17]);
        drop(escaped);
        store.result_read_drain.drain().await.unwrap();
        assert!(
            store.forget_run_results(&request.key).await.is_err(),
            "analysis input retention must take over before reader release"
        );
        let mut roots = store
            .db
            .query(
                "SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$analysis;",
            )
            .bind(("analysis", header.key.clone()))
            .await
            .and_then(checked)
            .unwrap();
        assert_eq!(roots.take::<Vec<Object>>(0).unwrap().len(), 1);
        store.forget_analysis_results(&header.key).await.unwrap();
        store.forget_run_results(&request.key).await.unwrap();
        let mut reclaimed = 0;
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
                .as_deref(),
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
                store.read_results(&request.key, &attempt, Duration::from_secs(60)),
                store.forget_run_results(&request.key)
            );
            assert!(!(read.is_ok() && retired.is_ok()));
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
                    if store.forget_run_results(&request.key).await.is_ok() {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            assert_eq!(cleanup(&store, &request.key).await, (3, 1));
        }
        assert_eq!(store.database(), database);
        store.remove_isolated_fixture().await.unwrap();
    }
}
