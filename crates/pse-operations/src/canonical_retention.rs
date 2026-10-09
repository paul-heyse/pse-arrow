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

/// One bounded reclamation page. Continue with `after`; absence completes a pass.
/// Edge-heavy versions keep their candidate at the cursor until its bounded edge
/// pages have been removed, so a caller does not lose the orphan frontier.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReclamationPage {
    /// Cursor for the next bounded call; none completes this pass.
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

impl CanonicalStore {
    /// Retain or move an available revision under a deliberate history owner.
    /// Product and execution/analysis lifecycle roots are minted by their owning admission.
    /// The server receipt supplies its sequence; caller receipt fields are not authority.
    pub async fn retain_revision(
        &self,
        revision: &Revision,
        owner: &RetentionOwner,
    ) -> Result<(), CanonicalError> {
        within_clock(
            original_deadline(crate::canonical::REQUEST_TIMEOUT),
            async {
                owner.validate()?;
                if !matches!(owner, RetentionOwner::History(_)) {
                    return Err(CanonicalError::Configuration(
                        "only deliberate history roots can be minted or moved explicitly".into(),
                    ));
                }
                for attempt in 0..RETRIES {
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
                        }
                        other => return other,
                    }
                }
                Err(CanonicalError::Configuration(
                    "retention retries exhausted".into(),
                ))
            },
        )
        .await
    }

    /// Explicitly forget only this revision's default history root. Durable run,
    /// product, analysis, attempt and protected-reader roots remain independent.
    pub async fn forget_history(&self, revision: &Revision) -> Result<(), CanonicalError> {
        within_clock(
            original_deadline(crate::canonical::REQUEST_TIMEOUT),
            async {
                self.drop_retained_root(revision, &RetentionOwner::History(revision.key.clone()))
                    .await
            },
        )
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
        within_clock(
            original_deadline(crate::canonical::REQUEST_TIMEOUT),
            async {
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
                for attempt in 0..RETRIES {
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
                        }
                        other => return other,
                    }
                }
                Err(CanonicalError::Configuration(
                    "retention retries exhausted".into(),
                ))
            },
        )
        .await
    }

    /// Examine at most 64 closed intervals for one problem. Every deletion reruns
    /// its complete indexed reachability decision under the named retention guard.
    /// No elapsed-age default removes deliberately retained scientific history.
    pub async fn reclaim_page(
        &self,
        problem: &str,
        after: &str,
    ) -> Result<ReclamationPage, CanonicalError> {
        within_clock(original_deadline(crate::canonical::REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let mut response = bounded_query(self.db.query("SELECT key FROM canonical_memberships WHERE problem = $problem AND key > $after AND to_sequence != NONE ORDER BY key LIMIT 64;")
                .bind(("problem", problem.to_owned())).bind(("after", after.to_owned()))).await?;
            let rows: Vec<Object> = response.take(0)?;
            let full = rows.len() == PAGE;
            let mut page = ReclamationPage::default();
            let mut previous = after.to_owned();
            for mut row in rows {
                let key = canonical_codec::decode_string(canonical_codec::required(&mut row, "key")?)?;
                let mut decision = None;
                for attempt in 0..RETRIES {
                    self.ensure_writes()?;
                    let result = bounded_query(
                        self.db
                            .query(RECLAIM.replace(
                                "/* DELETE_VERSION */",
                                crate::canonical_staging::DELETE_VERSION,
                            ))
                            .bind(("problem", problem.to_owned()))
                            .bind(("candidate", key.clone())),
                    )
                    .await;
                    match result {
                        Err(error) if conflict(&error) && attempt + 1 < RETRIES => {
                            tokio::task::yield_now().await;
                        }
                        Err(error) => return Err(error),
                        Ok(mut response) => {
                            decision = response
                                .take::<Option<Object>>(response.num_statements().saturating_sub(2))?;
                            break;
                        }
                    }
                }
                let mut decision = decision.ok_or_else(|| {
                    CanonicalError::Configuration("reclamation retries exhausted".into())
                })?;
                page.examined += 1;
                page.memberships += usize::from(canonical_codec::decode_boolean(
                    canonical_codec::required(&mut decision, "membership")?,
                )?);
                page.versions += usize::from(canonical_codec::decode_boolean(
                    canonical_codec::required(&mut decision, "version")?,
                )?);
                let edges =
                    canonical_codec::decode_uint(canonical_codec::required(&mut decision, "edges")?)?;
                page.edges += usize::try_from(edges).map_err(|_| CanonicalError::PayloadLimit)?;
                let blocks =
                    canonical_codec::decode_uint(canonical_codec::required(&mut decision, "blocks")?)?;
                page.blocks += usize::try_from(blocks).map_err(|_| CanonicalError::PayloadLimit)?;
                if canonical_codec::decode_boolean(canonical_codec::required(
                    &mut decision,
                    "pending",
                )?)? {
                    page.after = Some(previous);
                    return Ok(page);
                }
                previous = key;
            }
            if full {
                page.after = Some(previous);
            }
            Ok(page)
        }).await
    }
}

const RETAIN: &str = r#"BEGIN;
IF $owner_kind != 'history' { THROW 'only deliberate history roots can be minted or moved explicitly'; };
LET $guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $selected = SELECT * FROM ONLY type::record('canonical_revisions', $revision);
IF $selected = NONE OR $selected.problem != $problem { THROW 'retained revision unavailable'; };
IF $selected.interpretation != $interpretation { THROW 'retained revision interpretation mismatch'; };
LET $pruned = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $problem AND from_sequence <= $selected.sequence AND to_sequence > $selected.sequence LIMIT 1;
IF array::len($pruned) != 0 { THROW 'revision source history has been reclaimed'; };
LET $old = SELECT * FROM ONLY type::record('canonical_roots', $root);
IF $old != NONE AND ($old.problem != $problem OR $old.owner_kind != $owner_kind OR $old.owner != $owner) { THROW 'retention owner identity collision'; };
UPSERT type::record('canonical_roots', $root) SET key = $root, problem = $problem, revision = $selected.key, sequence = $selected.sequence, owner_kind = $owner_kind, owner = $owner;
UPSERT $guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
COMMIT;"#;

const DROP_ROOT: &str = r#"BEGIN;
IF $owner_kind IN ['run','analysis','active_attempt'] { THROW 'lifecycle roots are released only by their owning retirement'; };
LET $guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $old = SELECT * FROM ONLY type::record('canonical_roots', $root);
IF $old != NONE {
    IF $old.problem != $problem OR $old.revision != $revision OR $old.owner_kind != $owner_kind OR $old.owner != $owner { THROW 'retained owner selection changed'; };
    DELETE ONLY type::record('canonical_roots', $root);
};
UPSERT $guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
COMMIT;"#;

const RECLAIM: &str = r#"BEGIN;
LET $guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $member = SELECT * FROM ONLY type::record('canonical_memberships', $candidate);
LET $result = IF $member = NONE OR $member.problem != $problem OR $member.to_sequence = NONE {
    {membership:false, version:false, edges:0dec, blocks:0dec, pending:false}
} ELSE {
    LET $head = SELECT * FROM ONLY type::record('canonical_problems', $problem);
    IF $head = NONE { THROW 'reclamation problem head unavailable'; };
    LET $roots = SELECT key FROM canonical_roots WHERE problem = $problem AND sequence >= $member.from_sequence AND sequence < $member.to_sequence LIMIT 1;
    LET $pins = SELECT key FROM canonical_protections WHERE problem = $problem AND sequence >= $member.from_sequence AND sequence < $member.to_sequence AND released = false AND expires_at > time::micros() LIMIT 1;
    IF ($head.sequence >= $member.from_sequence AND $head.sequence < $member.to_sequence) OR array::len($roots) != 0 OR array::len($pins) != 0 {
        {membership:false, version:false, edges:0dec, blocks:0dec, pending:false}
    } ELSE {
        UPSERT type::record('canonical_reclaimed_ranges', $candidate) SET key = $candidate, problem = $problem, from_sequence = $member.from_sequence, to_sequence = $member.to_sequence;
        LET $version = $member.version;
        LET $excluded_membership = $candidate;
        LET $excluded_stage = NONE;
        /* DELETE_VERSION */
        IF $deleted.pending = false { DELETE ONLY type::record('canonical_memberships', $candidate); };
        {membership:!$deleted.pending, version:$deleted.version, edges:$deleted.edges, blocks:$deleted.blocks, pending:$deleted.pending}
    };
};
UPSERT $guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
LET $pse_rpc_result = $result;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
RETURN $pse_rpc_result;
COMMIT;"#;

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
        store.forget_history(&first).await.unwrap();

        let pin = store.protect(first, Duration::from_secs(60)).await.unwrap();
        let before_protected_reclaim = retention_generation(&store, "guarded").await;
        let protected = store.reclaim_page("guarded", "").await.unwrap();
        let after_protected_reclaim = retention_generation(&store, "guarded").await;
        assert_eq!(protected.memberships, 0);
        assert!(after_protected_reclaim > before_protected_reclaim);
        assert!(store.object("guarded-v1").await.unwrap().is_some());
        store.release(&pin).await.unwrap();

        let before_unprotected_reclaim = retention_generation(&store, "guarded").await;
        let unprotected = store.reclaim_page("guarded", "").await.unwrap();
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
            store.reclaim_page("problem", "")
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
            .append_result_batch(&fence, "still-live", "values", 0, &[1], 1)
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
            store.reclaim_page("problem", "").await.unwrap().memberships,
            0
        );
        let pin = store
            .protect(first.clone(), Duration::from_secs(30))
            .await
            .unwrap();
        store.forget_history(&first).await.unwrap();
        assert_eq!(
            store.reclaim_page("problem", "").await.unwrap().memberships,
            0
        );
        let owner = RetentionOwner::History("deliberate-retention".into());
        store.retain_revision(&first, &owner).await.unwrap();
        store.release(&pin).await.unwrap();
        assert_eq!(
            store.reclaim_page("problem", "").await.unwrap().memberships,
            0
        );
        assert!(store.drop_retained_root(&second, &owner).await.is_err());
        store.drop_retained_root(&first, &owner).await.unwrap();
        let one = store.reclaim_page("problem", "").await.unwrap();
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
        let two = store
            .reclaim_page("problem", one.after.as_deref().unwrap())
            .await
            .unwrap();
        assert_eq!(two.edges, 64);
        assert_eq!(two.memberships, 0);
        let three = store
            .reclaim_page("problem", two.after.as_deref().unwrap())
            .await
            .unwrap();
        assert_eq!(three.edges, 2);
        assert_eq!(three.memberships, 1);
        assert_eq!(three.versions, 1);
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
            store.forget_history(&first).await.unwrap();
            if index % 2 == 0 {
                let (pin, reclaimed) = tokio::join!(
                    store.protect(first.clone(), Duration::from_secs(30)),
                    store.reclaim_page(&problem, "")
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
                    store.reclaim_page(&problem, "")
                );
                reclaimed.unwrap();
                if root.is_ok() {
                    assert!(store.object(&version).await.unwrap().is_some());
                    store.drop_retained_root(&first, &owner).await.unwrap();
                } else {
                    assert!(store.object(&version).await.unwrap().is_none());
                }
            }
            store.reclaim_page(&problem, "").await.unwrap();
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
            store.reclaim_page("problem", "").await.unwrap().memberships,
            0
        );
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let (selected, page) = tokio::join!(
            store.selected_object(&pin, "x-v1"),
            store.reclaim_page("problem", "")
        );
        assert!(selected.is_err());
        assert_eq!(page.unwrap().memberships, 1);
        assert!(store.object("x-v1").await.unwrap().is_none());
        assert!(store.protect(first, Duration::from_secs(1)).await.is_err());
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
        store.forget_history(&first).await.unwrap();
        let page = store.reclaim_page("problem", "").await.unwrap();
        assert_eq!(page.examined, 64);
        assert_eq!(page.memberships, 64);
        assert_eq!(page.versions, 64);
        let last = store
            .reclaim_page("problem", page.after.as_deref().unwrap())
            .await
            .unwrap();
        assert_eq!(last.examined, 6);
        assert_eq!(last.memberships, 6);
        assert_eq!(last.versions, 6);
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
