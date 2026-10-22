// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit retained selections and bounded reclamation. A revision receipt stays
//! immutable after its source intervals are reclaimed; range markers fence reopening.

use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, bounded_query},
    canonical_codec,
    generated::surreal as wire,
};
use pse_model::generated::runtime::canonical_revisions::Row as Revision;
use surrealdb::types::{Object, QueryError};

/// An explicit durable owner of an immutable revision selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetentionOwner {
    /// Deliberately retained revision history.
    History(String),
    /// Admitted compilation product.
    Product(String),
    /// Durable numerical run; only execution admission and retirement own this root.
    Run(String),
    /// Retained analysis result; only analysis admission and retirement own this root.
    Analysis(String),
    /// Active execution attempt; only its execution lifecycle owns this root.
    ActiveAttempt(String),
}
impl RetentionOwner {
    fn parts(&self) -> (&'static str, &str) {
        match self {
            Self::History(key) => ("history", key),
            Self::Product(key) => ("product", key),
            Self::Run(key) => ("run", key),
            Self::Analysis(key) => ("analysis", key),
            Self::ActiveAttempt(key) => ("active_attempt", key),
        }
    }
    fn key(&self) -> String {
        let (kind, owner) = self.parts();
        // Product admission already owns this exact record key.
        if kind == "product" {
            owner.to_owned()
        } else {
            format!("{kind}:{owner}")
        }
    }
    fn validate(&self) -> Result<(), CanonicalError> {
        if self.parts().1.is_empty() {
            return Err(CanonicalError::Configuration(
                "retention owner must be nonempty".into(),
            ));
        }
        if self.parts().1.len() > 4096 {
            return Err(CanonicalError::PayloadLimit);
        }
        Ok(())
    }
}

/// One bounded reclamation page. The canonical owner retains its continuation;
/// `after` observes that frontier and absence completes the current pass.
/// Edge-heavy versions keep their candidate at the cursor until its bounded edge
/// pages have been removed, so a caller does not lose the orphan frontier.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ReclamationPage {
    /// Observed canonical frontier; no caller supplies or resets this cursor.
    pub after: Option<String>,
    /// Candidate intervals whose fresh retention decisions were examined.
    pub examined: usize,
    /// Unreachable membership intervals removed.
    pub memberships: usize,
    /// Immutable versions removed after their last membership and edge disappeared.
    pub versions: usize,
    /// Structural edges removed in bounded pages.
    pub edges: usize,
    /// Exact payload transport blocks removed.
    pub blocks: usize,
}

const PAGE: usize = 64;
const RETRIES: usize = 8;

fn conflict(error: &CanonicalError) -> bool {
    matches!(error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(QueryError::TransactionConflict)))
}

/// Cleanup failure is observed independently of the already returned mutation
/// acknowledgment. No fresh budget or detached task is created after commitment.
pub(crate) async fn postcommit_cleanup<T: Default>(
    deadline: tokio::time::Instant,
    operation: &'static str,
    cleanup: impl Future<Output = Result<T, CanonicalError>>,
) -> T {
    let result = if deadline <= tokio::time::Instant::now() {
        Err(CanonicalError::Timeout)
    } else {
        within_clock(deadline, cleanup).await
    };
    match result {
        Ok(value) => value,
        Err(error) => {
            crate::canonical::operation_failed(operation, &error);
            T::default()
        }
    }
}

impl CanonicalStore {
    /// Retain or move an available revision under a deliberate history owner.
    /// Product and execution/analysis lifecycle roots are minted by their owning admission.
    /// The server receipt supplies its sequence; caller receipt fields are not authority.
    pub async fn retain_revision(
        &self,
        revision: &Revision,
        owner: &RetentionOwner,
    ) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(
                    original_deadline(crate::canonical::REQUEST_TIMEOUT),
                    async {
                        owner.validate()?;
                        if !matches!(owner, RetentionOwner::History(_)) {
                            return Err(CanonicalError::Configuration(
                                "only deliberate history roots can be minted or moved explicitly"
                                    .into(),
                            ));
                        }
                        let mut attempt = 0;
                        loop {
                            self.ensure_writes()?;
                            let result = bounded_query(
                                self.db
                                    .query(RETAIN)
                                    .bind(("problem", revision.problem.clone()))
                                    .bind(("revision", revision.key.clone()))
                                    .bind(("root", owner.key()))
                                    .bind(("owner_kind", owner.parts().0))
                                    .bind(("owner", owner.parts().1.to_owned()))
                                    .bind(("interpretation", wire::INTERPRETATION)),
                            )
                            .await
                            .map(|_| ());
                            match result {
                                Err(error) if conflict(&error) && attempt + 1 < RETRIES => {
                                    tokio::task::yield_now().await;
                                    attempt += 1;
                                }
                                other => {
                                    return other.map_err(|error| {
                                        error
                                            .exhaust_conflict("canonical_retention", RETRIES as u32)
                                    });
                                }
                            }
                        }
                    },
                )
                .await
            })
            .await
    }

    /// Explicitly forget only this revision's default history root. Durable run,
    /// product, analysis, attempt and protected-reader roots remain independent.
    pub async fn forget_history(&self, revision: &Revision) -> Result<(), CanonicalError> {
        self.drop_retained_root(revision, &RetentionOwner::History(revision.key.clone()))
            .await
    }

    /// Drop exactly a history or product owner's expected selection. Lifecycle roots
    /// are released by their owning retirement. A changed selection cannot release
    /// another revision; missing roots are idempotent.
    pub async fn drop_retained_root(
        &self,
        revision: &Revision,
        owner: &RetentionOwner,
    ) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let deadline = original_deadline(crate::canonical::REQUEST_TIMEOUT);
                within_clock(deadline, async {
                    owner.validate()?;
                    if matches!(
                        owner,
                        RetentionOwner::Run(_)
                            | RetentionOwner::Analysis(_)
                            | RetentionOwner::ActiveAttempt(_)
                    ) {
                        return Err(CanonicalError::Configuration(
                            "lifecycle roots are released only by their owning retirement".into(),
                        ));
                    }
                    let mut attempt = 0;
                    loop {
                        self.ensure_writes()?;
                        let result = bounded_query(
                            self.db
                                .query(DROP_ROOT)
                                .bind(("problem", revision.problem.clone()))
                                .bind(("revision", revision.key.clone()))
                                .bind(("root", owner.key()))
                                .bind(("owner_kind", owner.parts().0))
                                .bind(("owner", owner.parts().1.to_owned())),
                        )
                        .await
                        .map(|_| ());
                        match result {
                            Err(error) if conflict(&error) && attempt + 1 < RETRIES => {
                                tokio::task::yield_now().await;
                                attempt += 1;
                            }
                            Ok(()) => return Ok(()),
                            Err(error) => {
                                return Err(
                                    error.exhaust_conflict("canonical_retention", RETRIES as u32)
                                );
                            }
                        }
                    }
                })
                .await?;
                self.reclaim_source_after_commit(&revision.problem, deadline)
                    .await;
                Ok(())
            })
            .await
    }

    /// Continue one canonical cursor through at most 64 closed intervals.
    /// Protected candidates advance the frontier; a partial physical cleanup keeps
    /// its predecessor. End-of-pass reset is atomic and later calls revisit holds.
    pub async fn reclaim_source_page(
        &self,
        problem: &str,
    ) -> Result<ReclamationPage, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
                if problem.is_empty() || problem.len() > 4096 { return Err(CanonicalError::PayloadLimit); }
                let mut page = ReclamationPage::default();
                for _ in 0..PAGE {
                    self.ensure_writes()?;
                    let mut response = crate::canonical::protected_query("canonical_retention::source_step", || {
                        Ok(self.db.query("RETURN fn::pse_retention_v1::source_step($pse_rpc_expires_at,$problem);").bind(("problem",problem.to_owned())))
                    }).await?;
                    let mut decision = response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse)?;
                    let done = canonical_codec::decode_boolean(canonical_codec::required(&mut decision,"done")?)?;
                    let after = canonical_codec::required(&mut decision,"after")?;
                    page.after = match after {
                        surrealdb::types::Value::None => None,
                        value => Some(canonical_codec::decode_string(value)?),
                    };
                    if done { return Ok(page); }
                    page.examined += 1;
                    page.memberships += usize::from(canonical_codec::decode_boolean(canonical_codec::required(&mut decision,"membership")?)?);
                    page.versions += usize::from(canonical_codec::decode_boolean(canonical_codec::required(&mut decision,"version")?)?);
                    page.edges += usize::try_from(canonical_codec::decode_uint(canonical_codec::required(&mut decision,"edges")?)?).map_err(|_| CanonicalError::PayloadLimit)?;
                    page.blocks += usize::try_from(canonical_codec::decode_uint(canonical_codec::required(&mut decision,"blocks")?)?).map_err(|_| CanonicalError::PayloadLimit)?;
                    if canonical_codec::decode_boolean(canonical_codec::required(&mut decision,"pending")?)? { return Ok(page); }
                }
                Ok(page)
            }).await
        }).await
    }

    /// The committed mutation has already left its timeout owner; cleanup keeps
    /// the same original deadline and cannot replace its immutable receipt.
    pub(crate) async fn reclaim_source_after_commit(
        &self,
        problem: &str,
        deadline: tokio::time::Instant,
    ) {
        postcommit_cleanup(
            deadline,
            "canonical_retention::postcommit_source_page_deferred",
            self.reclaim_source_page(problem),
        )
        .await;
    }
}

const RETAIN: &str = r#"BEGIN;
IF $owner_kind != 'history' { THROW 'PSE_REFUSAL:only_deliberate_history_roots_can_be_minted_or_moved_explicitly'; };
LET $guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $selected = SELECT * FROM ONLY type::record('canonical_revisions', $revision);
IF $selected = NONE OR $selected.problem != $problem { THROW 'PSE_REFUSAL:retained_revision_unavailable'; };
IF $selected.interpretation != $interpretation { THROW 'PSE_REFUSAL:retained_revision_interpretation_mismatch'; };
LET $pruned = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $problem AND from_sequence <= $selected.sequence AND to_sequence > $selected.sequence LIMIT 1;
IF array::len($pruned) != 0 { THROW 'PSE_REFUSAL:revision_source_history_has_been_reclaimed'; };
LET $old = SELECT * FROM ONLY type::record('canonical_roots', $root) FOR UPDATE;
IF $old != NONE AND ($old.problem != $problem OR $old.owner_kind != $owner_kind OR $old.owner != $owner) { THROW 'PSE_REFUSAL:retention_owner_identity_collision'; };
UPSERT type::record('canonical_roots', $root) SET key = $root, problem = $problem, revision = $selected.key, sequence = $selected.sequence, owner_kind = $owner_kind, owner = $owner;
fn::pse_execution_v1::touch($pse_rpc_expires_at,'source-creators:'+$problem);
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
COMMIT;"#;

const DROP_ROOT: &str = r#"BEGIN;
IF $owner_kind IN ['run','analysis','active_attempt'] { THROW 'PSE_REFUSAL:lifecycle_roots_are_released_only_by_their_owning_retirement'; };
LET $guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $old = SELECT * FROM ONLY type::record('canonical_roots', $root) FOR UPDATE;
IF $old != NONE {
    IF $old.problem != $problem OR $old.revision != $revision OR $old.owner_kind != $owner_kind OR $old.owner != $owner { THROW 'PSE_REFUSAL:retained_owner_selection_changed'; };
    DELETE ONLY type::record('canonical_roots', $root);
    IF $owner_kind = 'product' {
        LET $association = SELECT * FROM ONLY type::record('canonical_product_associations',$root);
        IF $association != NONE {
            LET $descriptor = SELECT * FROM ONLY type::record('canonical_product_descriptors',$association.descriptor) FOR UPDATE;
            IF $descriptor != NONE {
                IF $descriptor.issuing_store != $association.issuing_store OR $descriptor.problem != $association.problem OR $descriptor.request != $association.request OR $descriptor.payload != $association.payload OR $descriptor.dependencies != $association.dependencies OR $descriptor.content != $association.content OR $descriptor.receipt != $association.receipt OR $descriptor.producer != $association.producer OR $descriptor.interpretation != $association.interpretation { THROW 'PSE_REFUSAL:released_association_descriptor_equality_differs'; };
                LET $remaining = SELECT key FROM canonical_product_associations WHERE descriptor = $association.descriptor AND issuing_store = $association.issuing_store AND problem = $association.problem AND request = $association.request AND payload = $association.payload AND dependencies = $association.dependencies AND content = $association.content AND receipt = $association.receipt AND producer = $association.producer AND interpretation = $association.interpretation AND type::record('canonical_roots',key).problem = problem AND type::record('canonical_roots',key).revision = revision AND type::record('canonical_roots',key).owner_kind = 'product' AND type::record('canonical_roots',key).owner = key ORDER BY key LIMIT 1;
                UPDATE type::record('canonical_product_descriptors',$association.descriptor) SET association = IF array::len($remaining) = 0 { NONE } ELSE { $remaining[0].key };
            };
        };
    };
};
fn::pse_execution_v1::touch($pse_rpc_expires_at,'source-creators:'+$problem);
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
COMMIT;"#;

#[cfg(test)]
mod postcommit_clock_tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[tokio::test]
    async fn committed_receipt_survives_cleanup_deadline_and_original_clock_is_not_renewed()
    -> Result<(), CanonicalError> {
        struct Dropped(Arc<AtomicBool>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(100);
        let receipt = within_clock(deadline, async { Ok(37_u64) }).await?;
        let entered = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicBool::new(false));
        let gate = tokio::sync::Notify::new();
        let deferred = postcommit_cleanup(deadline, "test:postcommit_gate_deferred", async {
            let _guard = Dropped(dropped.clone());
            entered.store(true, Ordering::SeqCst);
            gate.notified().await;
            Ok(true)
        })
        .await;
        assert_eq!(
            receipt, 37,
            "cleanup expiry does not replace the committed acknowledgment"
        );
        assert!(!deferred);
        assert!(entered.load(Ordering::SeqCst));
        assert!(
            dropped.load(Ordering::SeqCst),
            "the bounded cleanup drains synchronously"
        );
        let started_after_expiry = AtomicBool::new(false);
        let repeated = postcommit_cleanup(deadline, "test:expired_postcommit_deferred", async {
            started_after_expiry.store(true, Ordering::SeqCst);
            Ok(true)
        })
        .await;
        assert!(!repeated);
        assert!(
            !started_after_expiry.load(Ordering::SeqCst),
            "no fresh cleanup budget or request after original expiry"
        );
        let explicit = within_clock(
            deadline,
            std::future::pending::<Result<(), CanonicalError>>(),
        )
        .await;
        assert!(
            matches!(explicit, Err(CanonicalError::Timeout)),
            "an explicit continuation remains a typed failure"
        );
        Ok(())
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        reason = "isolated retention fixtures and assertions fail the test on unexpected results"
    )]
    use super::*;
    use crate::canonical::{CanonicalOptions, ObjectEdit};
    use pse_model::generated::runtime::canonical_versions::Row as ObjectVersion;
    use std::{path::Path, time::Duration};

    fn edit(key: &str, references: usize) -> ObjectEdit {
        ObjectEdit {
            logical: "x".into(),
            scope: "root".into(),
            name: "x".into(),
            version: Some(ObjectVersion {
                key: key.into(),
                logical: "x".into(),
                kind: "test".into(),
                payload: vec![1, 2, 3].into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
            references: (0..references)
                .map(|index| ("root".into(), format!("target-{index}")))
                .collect(),
        }
    }

    async fn fixture() -> (CanonicalStore, String) {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_retention_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        (store, options.database)
    }

    async fn remove(store: &CanonicalStore, database: &str) {
        assert_eq!(store.database(), database);
        store.remove_isolated_fixture().await.unwrap();
    }

    async fn retention_generation(store: &CanonicalStore, problem: &str) -> u64 {
        let key = format!("retention:{problem}");
        let row: Option<Object> = store
            .db
            .select(("canonical_guards", key.as_str()))
            .await
            .unwrap();
        wire::decode_canonical_guards(row.unwrap())
            .unwrap()
            .generation
    }

    #[tokio::test]
    async fn canonical_source_cursor_reopens_past_a_protected_first_page() {
        let (store, database) = fixture().await;
        let initial = (0..64)
            .map(|index| {
                let mut value = edit(&format!("protected-{index}"), 0);
                value.logical = format!("protected-{index}");
                value.name = value.logical.clone();
                value.version.as_mut().unwrap().logical = value.logical.clone();
                value
            })
            .collect::<Vec<_>>();
        let first = store
            .edit("cursor-problem", None, "a-first", &initial)
            .await
            .unwrap();
        let pin = store
            .protect(first.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        let last = (0..6)
            .map(|index| {
                let mut value = edit(&format!("unprotected-{index}"), 0);
                value.logical = format!("unprotected-{index}");
                value.name = value.logical.clone();
                value.version.as_mut().unwrap().logical = value.logical.clone();
                value
            })
            .collect::<Vec<_>>();
        let mut replacement = initial
            .iter()
            .cloned()
            .map(|mut value| {
                value.version = None;
                value
            })
            .collect::<Vec<_>>();
        replacement.extend(last.clone());
        let second = store
            .edit("cursor-problem", Some("a-first"), "b-second", &replacement)
            .await
            .unwrap();
        let removals = last
            .into_iter()
            .map(|mut value| {
                value.version = None;
                value
            })
            .collect::<Vec<_>>();
        store
            .edit("cursor-problem", Some("b-second"), "c-empty", &removals)
            .await
            .unwrap();
        store.forget_history(&first).await.unwrap();
        let mut cursor = bounded_query(store.db.query("SELECT source_reclaim_after FROM ONLY type::record('canonical_guards','retention:cursor-problem');")).await.unwrap();
        let mut cursor = cursor.take::<Option<Object>>(0).unwrap().unwrap();
        let after = canonical_codec::decode_string(
            canonical_codec::required(&mut cursor, "source_reclaim_after").unwrap(),
        )
        .unwrap();
        assert!(
            !after.is_empty(),
            "the first protected page advances its canonical frontier"
        );
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = database.clone();
        let peer = crate::testing::canonical_fixture_peer(&store, &options).unwrap();
        peer.forget_history(&second).await.unwrap();
        for index in 0..6 {
            assert!(
                store
                    .object(&format!("unprotected-{index}"))
                    .await
                    .unwrap()
                    .is_none(),
                "later candidates are not starved by the first64 protected intervals"
            );
        }
        for index in 0..64 {
            assert!(
                store
                    .object(&format!("protected-{index}"))
                    .await
                    .unwrap()
                    .is_some()
            );
        }
        store.release(&pin).await.unwrap();
        let page = peer.reclaim_source_page("cursor-problem").await.unwrap();
        assert_eq!(
            (page.examined, page.memberships, page.versions),
            (64, 64, 64)
        );
        let end = peer.reclaim_source_page("cursor-problem").await.unwrap();
        assert_eq!(end.examined, 0);
        assert!(end.after.is_none());
        drop(peer);
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn source_activation_replay_and_postcommit_cleanup_failure_preserve_receipts()
    -> Result<(), CanonicalError> {
        let (store, database) = fixture().await;
        let start = wire::SCHEMA
            .find("DEFINE FUNCTION fn::pse_retention_v1::source_step(")
            .unwrap();
        let suffix = &wire::SCHEMA[start..];
        let next = suffix["DEFINE FUNCTION ".len()..]
            .find("DEFINE FUNCTION ")
            .unwrap()
            + "DEFINE FUNCTION ".len();
        let original = &suffix[..next];
        let restored = original.replacen("DEFINE FUNCTION ", "DEFINE FUNCTION OVERWRITE ", 1);
        let refused = restored.replacen(
            "-> object {",
            "-> object { THROW 'PSE_REFUSAL:reclamation_problem_head_unavailable';",
            1,
        );
        assert_ne!(refused, restored);
        for direct in [true, false] {
            bounded_query(store.db.query(refused.clone())).await?;
            let problem = if direct {
                "receipt-direct"
            } else {
                "receipt-stage-ack"
            };
            let operation = format!("{problem}-first");
            let version = format!("{problem}-v1");
            let second_version = format!("{problem}-v2");
            let edits = [edit(&version, 0)];
            let crate::canonical_staging::StageOutcome::Closed(stage) =
                store.stage_edits(problem, None, &operation, &edits).await?
            else {
                return Err(CanonicalError::Configuration(
                    "fresh cleanup fixture was unexpectedly acknowledged".into(),
                ));
            };
            let first = store.activate_source_stage(&stage).await?;
            let second = store
                .edit(
                    problem,
                    Some(&operation),
                    &format!("{problem}-second"),
                    &[edit(&second_version, 0)],
                )
                .await?;
            store.forget_history(&first).await?;
            assert!(
                store.object(&version).await?.is_some(),
                "failed postcommit cleanup stays explicitly pending"
            );
            assert!(
                store.reclaim_source_page(problem).await.is_err(),
                "explicit continuation retains its typed cleanup failure"
            );
            assert_eq!(store.revision(&first.key).await?, Some(first.clone()));
            assert_eq!(store.revision(&second.key).await?, Some(second.clone()));
            bounded_query(store.db.query(restored.clone())).await?;
            let replay = if direct {
                store.activate_source_stage(&stage).await?
            } else {
                store.edit(problem, None, &operation, &edits).await?
            };
            assert_eq!(
                replay, first,
                "an immutable acknowledgment remains success after GC failure"
            );
            assert!(
                store.object(&version).await?.is_none(),
                "both immutable-ACK paths actually drive the deferred page"
            );
            assert!(store.object(&second_version).await?.is_some());
        }
        remove(&store, &database).await;
        Ok(())
    }

    #[tokio::test]
    async fn reclamation_guard_generation_advances_for_protected_and_deleted_intervals() {
        let (store, database) = fixture().await;
        let first = store
            .edit("guarded", None, "guarded-first", &[edit("guarded-v1", 0)])
            .await
            .unwrap();
        store
            .edit(
                "guarded",
                Some("guarded-first"),
                "guarded-second",
                &[edit("guarded-v2", 0)],
            )
            .await
            .unwrap();
        let pin = store
            .protect(first.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        store.forget_history(&first).await.unwrap();
        let before_protected_reclaim = retention_generation(&store, "guarded").await;
        let protected = store.reclaim_source_page("guarded").await.unwrap();
        let after_protected_reclaim = retention_generation(&store, "guarded").await;
        assert_eq!(protected.memberships, 0);
        assert!(after_protected_reclaim > before_protected_reclaim);
        assert!(store.object("guarded-v1").await.unwrap().is_some());
        store.release(&pin).await.unwrap();

        let before_unprotected_reclaim = retention_generation(&store, "guarded").await;
        let unprotected = store.reclaim_source_page("guarded").await.unwrap();
        let after_unprotected_reclaim = retention_generation(&store, "guarded").await;
        assert_eq!(unprotected.memberships, 1);
        assert_eq!(unprotected.versions, 1);
        assert!(after_unprotected_reclaim > before_unprotected_reclaim);
        assert!(store.object("guarded-v1").await.unwrap().is_none());
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn lifecycle_roots_cannot_be_redirected_or_released_by_generic_retention() {
        use crate::canonical_execution::RunRequest;
        let (store, database) = fixture().await;
        let first = store
            .edit("problem", None, "first", &[edit("x-first", 0)])
            .await
            .unwrap();
        let second = store
            .edit("problem", Some("first"), "second", &[edit("x-second", 0)])
            .await
            .unwrap();
        store
            .begin_run(&RunRequest {
                key: "real-run".into(),
                revision: first.clone(),
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("real-run", "real-claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let mut response = bounded_query(store.db.query("SELECT * FROM canonical_roots WHERE owner_kind IN ['run','active_attempt'] ORDER BY key;")).await.unwrap();
        let before: Vec<Object> = response.take(0).unwrap();
        assert_eq!(before.len(), 2);
        for owner in [
            RetentionOwner::Run("real-run".into()),
            RetentionOwner::ActiveAttempt(fence.attempt().into()),
            RetentionOwner::Analysis("unissued-analysis".into()),
        ] {
            assert!(store.retain_revision(&second, &owner).await.is_err());
            assert!(store.drop_retained_root(&first, &owner).await.is_err());
            // Native boundary also refuses lifecycle roots, independently of the Rust facade.
            assert!(
                bounded_query(
                    store
                        .db
                        .query(RETAIN)
                        .bind(("problem", second.problem.clone()))
                        .bind(("revision", second.key.clone()))
                        .bind(("root", owner.key()))
                        .bind(("owner_kind", owner.parts().0))
                        .bind(("owner", owner.parts().1.to_owned()))
                        .bind(("interpretation", wire::INTERPRETATION))
                )
                .await
                .is_err()
            );
            assert!(
                bounded_query(
                    store
                        .db
                        .query(DROP_ROOT)
                        .bind(("problem", first.problem.clone()))
                        .bind(("revision", first.key.clone()))
                        .bind(("root", owner.key()))
                        .bind(("owner_kind", owner.parts().0))
                        .bind(("owner", owner.parts().1.to_owned()))
                )
                .await
                .is_err()
            );
        }
        let mut response = bounded_query(store.db.query("SELECT * FROM canonical_roots WHERE owner_kind IN ['run','active_attempt'] ORDER BY key;")).await.unwrap();
        assert_eq!(response.take::<Vec<Object>>(0).unwrap(), before);
        store.forget_history(&first).await.unwrap();
        let (retired, reclaimed) = tokio::join!(
            store.forget_run_results("real-run"),
            store.reclaim_source_page("problem")
        );
        assert!(
            retired.is_err(),
            "a live attempt still owns its selected source"
        );
        assert_eq!(reclaimed.unwrap().memberships, 0);
        let mut response = bounded_query(store.db.query("SELECT * FROM canonical_roots WHERE owner_kind IN ['run','active_attempt'] ORDER BY key;")).await.unwrap();
        assert_eq!(response.take::<Vec<Object>>(0).unwrap(), before);
        let pin = store
            .protect(first.clone(), Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(
            store
                .selected_object(&pin, "x-first")
                .await
                .unwrap()
                .unwrap()
                .payload
                .as_slice(),
            [1, 2, 3]
        );
        store.release(&pin).await.unwrap();
        store
            .append_result_batch(
                &fence,
                "still-live",
                "values",
                0,
                crate::canonical_execution::ResultPayload::new([1].to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        let history = RetentionOwner::History("movable-history".into());
        store.retain_revision(&first, &history).await.unwrap();
        store.retain_revision(&second, &history).await.unwrap();
        assert!(store.drop_retained_root(&first, &history).await.is_err());
        store.drop_retained_root(&second, &history).await.unwrap();
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn explicit_history_roots_pins_and_bounded_edges_preserve_receipts() {
        let (store, database) = fixture().await;
        let initial = [edit("x-v1", 130)];
        let first = store.edit("problem", None, "op-1", &initial).await.unwrap();
        let second = store
            .edit("problem", Some("op-1"), "op-2", &[edit("x-v2", 0)])
            .await
            .unwrap();
        assert_eq!(
            store
                .reclaim_source_page("problem")
                .await
                .unwrap()
                .memberships,
            0
        );
        let pin = store
            .protect(first.clone(), Duration::from_secs(30))
            .await
            .unwrap();
        store.forget_history(&first).await.unwrap();
        assert_eq!(
            store
                .reclaim_source_page("problem")
                .await
                .unwrap()
                .memberships,
            0
        );
        let owner = RetentionOwner::History("deliberate-retention".into());
        store.retain_revision(&first, &owner).await.unwrap();
        store.release(&pin).await.unwrap();
        assert_eq!(
            store
                .reclaim_source_page("problem")
                .await
                .unwrap()
                .memberships,
            0
        );
        assert!(store.drop_retained_root(&second, &owner).await.is_err());
        store.drop_retained_root(&first, &owner).await.unwrap();
        let one = store.reclaim_source_page("problem").await.unwrap();
        assert_eq!(one.edges, 64);
        assert_eq!(one.memberships, 0);
        assert!(one.after.is_some());
        assert!(
            store
                .protect(first.clone(), Duration::from_secs(30))
                .await
                .is_err()
        );
        assert!(
            store
                .retain_revision(&first, &RetentionOwner::History(first.key.clone()))
                .await
                .is_err()
        );
        let two = store.reclaim_source_page("problem").await.unwrap();
        // Root withdrawal already reclaimed the first 64 edges; the first
        // explicit continuation reclaimed the next 64.
        assert_eq!(two.edges, 2);
        assert_eq!(two.memberships, 1);
        assert_eq!(two.versions, 1);
        assert!(store.object("x-v1").await.unwrap().is_none());
        assert!(store.object("x-v2").await.unwrap().is_some());
        assert_eq!(store.revision("op-1").await.unwrap(), Some(first.clone()));
        assert_eq!(
            store.edit("problem", None, "op-1", &initial).await.unwrap(),
            first
        );
        // A valid empty head retains its own exact identity without replaying ancestors.
        let empty = store.edit("empty", None, "empty-1", &[]).await.unwrap();
        let pin = store.protect(empty, Duration::from_secs(30)).await.unwrap();
        assert!(
            store
                .membership_page(&pin, None, "")
                .await
                .unwrap()
                .is_empty()
        );
        store.release(&pin).await.unwrap();
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn root_and_pin_admission_race_reclamation_on_fresh_decisions() {
        let (store, database) = fixture().await;
        for index in 0..12 {
            let problem = format!("race-{index}");
            let first_key = format!("first-{index}");
            let version = format!("old-{index}");
            let next_version = format!("new-{index}");
            let first = store
                .edit(&problem, None, &first_key, &[edit(&version, 0)])
                .await
                .unwrap();
            store
                .edit(
                    &problem,
                    Some(&first_key),
                    &format!("next-{index}"),
                    &[edit(&next_version, 0)],
                )
                .await
                .unwrap();
            if index % 2 == 0 {
                let (pin, reclaimed) = tokio::join!(
                    store.protect(first.clone(), Duration::from_secs(30)),
                    store.forget_history(&first)
                );
                reclaimed.unwrap();
                if let Ok(pin) = pin {
                    let selected = store
                        .selected_object(&pin, &version)
                        .await
                        .unwrap()
                        .unwrap();
                    assert_eq!(selected.payload.as_slice(), [1, 2, 3]);
                    store.release(&pin).await.unwrap();
                } else {
                    assert!(store.object(&version).await.unwrap().is_none());
                }
            } else {
                let owner = RetentionOwner::History(format!("deliberate-{index}"));
                let (root, reclaimed) = tokio::join!(
                    store.retain_revision(&first, &owner),
                    store.forget_history(&first)
                );
                reclaimed.unwrap();
                if root.is_ok() {
                    assert!(store.object(&version).await.unwrap().is_some());
                    store.drop_retained_root(&first, &owner).await.unwrap();
                } else {
                    assert!(store.object(&version).await.unwrap().is_none());
                }
            }
            store.reclaim_source_page(&problem).await.unwrap();
            assert!(store.object(&version).await.unwrap().is_none());
            assert!(store.object(&next_version).await.unwrap().is_some());
        }
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn concurrent_protected_reads_and_lease_changes_retry_complete_transactions() {
        let (store, database) = fixture().await;
        let revision = store
            .edit("read-race", None, "read-race-1", &[edit("read-race-x", 0)])
            .await
            .unwrap();
        let mut tasks = tokio::task::JoinSet::new();
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(16));
        for _ in 0..16 {
            let store = store.clone();
            let revision = revision.clone();
            let barrier = barrier.clone();
            tasks.spawn(async move {
                barrier.wait().await;
                for _ in 0..8 {
                    let pin = store
                        .protect(revision.clone(), Duration::from_secs(60))
                        .await
                        .unwrap();
                    let rows = store
                        .select_names(&pin, "root", &["x".into(), "absent".into()])
                        .await
                        .unwrap();
                    assert_eq!(rows.len(), 1);
                    assert_eq!(
                        store
                            .selected_object(&pin, "read-race-x")
                            .await
                            .unwrap()
                            .unwrap()
                            .payload
                            .as_slice(),
                        [1, 2, 3]
                    );
                    store.release(&pin).await.unwrap();
                    assert!(
                        store
                            .select_names(&pin, "root", &["x".into()])
                            .await
                            .is_err()
                    );
                }
            });
        }
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            results.push(result);
        }
        remove(&store, &database).await;
        for result in results {
            result.unwrap();
        }
    }

    #[tokio::test]
    async fn expired_selection_cannot_read_or_resurrect_reclaimed_history() {
        let (store, database) = fixture().await;
        let first = store
            .edit("problem", None, "old", &[edit("x-v1", 0)])
            .await
            .unwrap();
        store
            .edit("problem", Some("old"), "new", &[edit("x-v2", 0)])
            .await
            .unwrap();
        let pin = store
            .protect(first.clone(), Duration::from_secs(1))
            .await
            .unwrap();
        store.forget_history(&first).await.unwrap();
        assert_eq!(
            store
                .reclaim_source_page("problem")
                .await
                .unwrap()
                .memberships,
            0
        );
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let (selected, page) = tokio::join!(
            store.selected_object(&pin, "x-v1"),
            store.reclaim_source_page("problem")
        );
        assert!(selected.is_err());
        assert_eq!(page.unwrap().memberships, 1);
        assert!(store.object("x-v1").await.unwrap().is_none());
        assert!(store.protect(first, Duration::from_secs(1)).await.is_err());
        pin.finish().await.unwrap();
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn interval_reclamation_pages_are_bounded_and_keep_empty_current_head() {
        let (store, database) = fixture().await;
        let initial = (0..70)
            .map(|index| {
                let mut value = edit(&format!("version-{index}"), 0);
                value.logical = format!("logical-{index}");
                value.name = format!("name-{index}");
                value.version.as_mut().unwrap().logical = value.logical.clone();
                value
            })
            .collect::<Vec<_>>();
        let first = store
            .edit("problem", None, "initial", &initial)
            .await
            .unwrap();
        let removals = initial
            .into_iter()
            .map(|mut value| {
                value.version = None;
                value
            })
            .collect::<Vec<_>>();
        let empty = store
            .edit("problem", Some("initial"), "empty", &removals)
            .await
            .unwrap();
        // Activation examined the first 64 closed intervals while the default
        // history root still protected them. Withdrawal resumes its persisted
        // frontier; it does not restart the scan at the beginning.
        let position: Option<Object> = store
            .db
            .select(("canonical_guards", "retention:problem"))
            .await
            .unwrap();
        assert!(
            wire::decode_canonical_guards(position.unwrap())
                .unwrap()
                .source_reclaim_after
                .is_some()
        );
        let protected: Vec<Object> = store.db.select("canonical_memberships").await.unwrap();
        assert_eq!(protected.len(), 70);
        store.forget_history(&first).await.unwrap();
        let remaining: Vec<Object> = store.db.select("canonical_memberships").await.unwrap();
        assert_eq!(
            remaining.len(),
            64,
            "withdrawal completes only the six-candidate tail of the canonical pass"
        );
        let position: Option<Object> = store
            .db
            .select(("canonical_guards", "retention:problem"))
            .await
            .unwrap();
        assert!(
            wire::decode_canonical_guards(position.unwrap())
                .unwrap()
                .source_reclaim_after
                .is_none()
        );
        let page = store.reclaim_source_page("problem").await.unwrap();
        assert_eq!(page.examined, 64);
        assert_eq!(page.memberships, 64);
        assert_eq!(page.versions, 64);
        let last = store.reclaim_source_page("problem").await.unwrap();
        assert_eq!(last.examined, 0);
        assert_eq!(last.memberships, 0);
        assert_eq!(last.versions, 0);
        assert!(last.after.is_none());
        let pin = store.protect(empty, Duration::from_secs(30)).await.unwrap();
        assert!(
            store
                .membership_page(&pin, None, "")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(store.protect(first, Duration::from_secs(30)).await.is_err());
        store.release(&pin).await.unwrap();
        remove(&store, &database).await;
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "canonical_conflict_tests.rs"]
mod canonical_server_unit_exact_conflicts;
