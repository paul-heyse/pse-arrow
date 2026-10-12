// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deliberately retained detailed history and irreversible minimal replay fences.

use crate::{canonical::{CanonicalError, CanonicalStore, protected_query}, canonical_codec};
use surrealdb::types::Object;

/// Exact scientific history owner. Releasing payloads does not release its history.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag="kind",content="key",rename_all="snake_case",deny_unknown_fields)]
pub enum HistoryOwner {
    /// A semantic execution and its attempt/operation detail.
    Run(String),
    /// A study's occurrence, policy, observation and coordination detail.
    Study(String),
}
impl HistoryOwner {
    fn parts(&self) -> (&'static str, &str) {
        match self { Self::Run(key) => ("run", key), Self::Study(key) => ("study", key) }
    }
    fn validate(&self) -> Result<(), CanonicalError> { identity(self.parts().1) }
}

/// Why detailed history must survive an otherwise authorized release.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum HistoryPurpose {
    /// A deliberate scientific or operational audit obligation.
    Audit,
    /// Exact detail needed to settle a known uncertain external effect.
    Unsettled,
}
impl HistoryPurpose {
    fn name(self) -> &'static str { match self { Self::Audit => "audit", Self::Unsettled => "unsettled" } }
}

/// Independent exact ownership, not a timeout or a reference inferred from age.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoryObligation {
    /// Immutable obligation identity; equality includes all fields.
    pub key: String,
    /// Original owner whose detail must remain available.
    pub owner: HistoryOwner,
    /// Scientific audit or unresolved effect.
    pub purpose: HistoryPurpose,
    /// Explicit caller attribution, preserved and checked on release.
    pub attribution: String,
}

/// One restartable bounded page. Permanent identity/owner locators survive completion.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoryReclamationPage {
    /// No releasable detail remains for this exact owner.
    pub complete: bool,
    /// Detail records removed in this page.
    pub removed: u64,
    /// Operation receipts reduced to truthful settled attribution in this page.
    pub compacted: u64,
}

fn identity(key: &str) -> Result<(), CanonicalError> {
    if key.is_empty() || key.len() > 4096 { Err(CanonicalError::PayloadLimit) } else { Ok(()) }
}

// Each block is one server statement/transaction; guard and absence interpretation
// share its registered read set. Call sites supply fixed SQL and binding names.
pub(crate) fn study_query(sql: &'static str, binding: &'static str) -> String {
    format!("RETURN {{ fn::pse_history_v1::check_owner($pse_rpc_expires_at,'study',${binding}); {sql} }};")
}
pub(crate) fn point_query(sql: &'static str, binding: &'static str) -> String {
    format!("RETURN {{ fn::pse_history_v1::point($pse_rpc_expires_at,${binding}); {sql} }};")
}
pub(crate) fn run_query(sql: &'static str, binding: &'static str) -> String {
    format!("RETURN {{ fn::pse_history_v1::run($pse_rpc_expires_at,${binding}); {sql} }};")
}

impl CanonicalStore {
    /// Retain detailed history independently from scientific artifact ownership.
    /// A run obligation also protects its enclosing study's detailed history;
    /// a study obligation protects its member run histories.
    pub async fn retain_history_obligation(&self, obligation: &HistoryObligation) -> Result<(), CanonicalError> {
        self.change_history_obligation(obligation, true).await
    }

    /// Release exactly this obligation. Other audit and unknown-effect owners remain.
    pub async fn release_history_obligation(&self, obligation: &HistoryObligation) -> Result<(), CanonicalError> {
        self.change_history_obligation(obligation, false).await
    }

    async fn change_history_obligation(&self, obligation: &HistoryObligation, retaining: bool) -> Result<(), CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            obligation.owner.validate()?; identity(&obligation.key)?; identity(&obligation.attribution)?;
            self.ensure_writes()?;
            let query=if retaining {"BEGIN; RETURN fn::pse_history_v1::retain($pse_rpc_expires_at,$key,$kind,$owner,$purpose,$attribution); COMMIT;"} else {"BEGIN; RETURN fn::pse_history_v1::drop_obligation($pse_rpc_expires_at,$key,$kind,$owner,$purpose,$attribution); COMMIT;"};
            protected_query("canonical_history::obligation", || Ok(self.db.query(query)
                .bind(("key",obligation.key.clone())).bind(("kind",obligation.owner.parts().0))
                .bind(("owner",obligation.owner.parts().1.to_owned())).bind(("purpose",obligation.purpose.name()))
                .bind(("attribution",obligation.attribution.clone())))).await?;
            Ok(())
        }).await
    }

    /// Explicitly and irreversibly release detailed history after payload retirement,
    /// recovery, borrowers and attributed obligations have all settled. Terminality
    /// alone never authorizes release. Repeating resumes the same permanent fence.
    pub async fn release_history(&self, owner: &HistoryOwner) -> Result<(), CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            owner.validate()?; self.ensure_writes()?;
            protected_query("canonical_history::release", || Ok(self.db.query("BEGIN; RETURN fn::pse_history_v1::release($pse_rpc_expires_at,$kind,$owner); COMMIT;")
                .bind(("kind",owner.parts().0)).bind(("owner",owner.parts().1.to_owned())))).await?;
            Ok(())
        }).await
    }

    /// Remove at most 64 records, or compact at most 64 operation receipts.
    /// Source authority floors, operation identities, point/attempt owner locators
    /// and retirement fences are permanent and never garbage-collected here.
    pub async fn reclaim_history_page(&self, owner: &HistoryOwner) -> Result<HistoryReclamationPage, CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            owner.validate()?; self.ensure_writes()?;
            let mut response=protected_query("canonical_history::collect", || Ok(self.db.query("BEGIN; RETURN fn::pse_history_v1::collect($pse_rpc_expires_at,$kind,$owner); COMMIT;")
                .bind(("kind",owner.parts().0)).bind(("owner",owner.parts().1.to_owned())))).await?;
            let index=response.num_statements().saturating_sub(2);
            let mut row=response.take::<Option<Object>>(index)?.ok_or(CanonicalError::IncompleteResponse)?;
            Ok(HistoryReclamationPage {
                complete:canonical_codec::decode_boolean(canonical_codec::required(&mut row,"complete")?)?,
                removed:canonical_codec::decode_uint(canonical_codec::required(&mut row,"removed")?)?,
                compacted:canonical_codec::decode_uint(canonical_codec::required(&mut row,"compacted")?)?,
            })
        }).await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(unsafe_code, reason = "controlled synthetic terminal observations exercise storage ownership only")]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, reason = "owned disposable history fixtures fail on unexpected results")]
mod canonical_history_tests {
    use super::*;
    use crate::{canonical::{CanonicalOptions, bounded_query}, canonical_execution::{RunRequest, TerminalClass, SeedEligibility}};
    use std::{path::Path, time::Duration};

    async fn fixture() -> CanonicalStore {
        let state=std::env::var("PSE_SURREAL_STATE").expect("owned current-version canonical fixture required");
        let mut options=CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database=format!("canonical_test_history_{}",uuid::Uuid::new_v4().simple());
        crate::testing::canonical_fixture_with_options(&options,true).unwrap()
    }
    async fn request(store: &CanonicalStore, key: &str) -> RunRequest {
        RunRequest { key:key.into(), revision:store.edit(key,None,&format!("source-{key}"),&[]).await.unwrap(),sources:vec![],request:vec![1],source_selection:vec![2],attestation:vec![3] }
    }
    async fn finish(store: &CanonicalStore, request: &RunRequest) -> String {
        store.begin_run(request).await.unwrap();
        let fence=store.claim_run(&request.key,&format!("claim-{}",request.key),"worker",Duration::from_secs(60)).await.unwrap();
        store.append_result_batch(&fence,&format!("append-{}",request.key),"values",0,crate::canonical_execution::ResultPayload::new(vec![17]).unwrap(),1).await.unwrap();
        let closed=store.close_result_ingestion(&fence,&format!("close-{}",request.key)).await.unwrap();
        let manifest=store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: the fixture owns this empty attempt; partial classification promises no scientific success.
        unsafe { store.seal_attempt(&manifest,&format!("seal-{}",request.key),TerminalClass::Partial,&[42],SeedEligibility::Unavailable).await }.unwrap();
        fence.attempt().into()
    }
    async fn retire_payload(store: &CanonicalStore, run: &str) {
        store.forget_run_results(run).await.unwrap();
        for _ in 0..64 { if store.reclaim_result_page(run).await.unwrap().complete {return;} }
        panic!("payload cleanup did not converge");
    }
    async fn collect(store: &CanonicalStore, owner: &HistoryOwner) -> u64 {
        let mut compacted=0;
        for _ in 0..64 {
            let page=store.reclaim_history_page(owner).await.unwrap();
            assert!(page.removed<=64 && page.compacted<=64);
            compacted+=page.compacted;
            if page.complete {return compacted;}
        }
        panic!("history cleanup did not converge");
    }
    #[tokio::test]
    async fn explicit_history_release_preserves_fences_and_compact_truthful_receipts() {
        let store=fixture().await;
        let request=request(&store,"exploratory").await;
        let owner=HistoryOwner::Run(request.key.clone());
        assert!(store.release_history(&owner).await.is_err());
        let attempt=finish(&store,&request).await;
        assert!(store.release_history(&owner).await.is_err());
        let audit=HistoryObligation{key:"audit".into(),owner:owner.clone(),purpose:HistoryPurpose::Audit,attribution:"selected scientific investigation".into()};
        let unknown=HistoryObligation{key:"unknown".into(),owner:owner.clone(),purpose:HistoryPurpose::Unsettled,attribution:"unacknowledged external observation".into()};
        store.retain_history_obligation(&audit).await.unwrap();
        store.retain_history_obligation(&unknown).await.unwrap();
        retire_payload(&store,&request.key).await;
        assert!(store.release_history(&owner).await.is_err());
        assert!(store.canonical_run(&request.key).await.unwrap().is_some());
        store.release_history_obligation(&audit).await.unwrap();
        assert!(store.retain_history_obligation(&audit).await.is_err());
        let early=HistoryObligation{key:"late-acquire".into(),..audit.clone()};
        store.release_history_obligation(&early).await.unwrap();
        assert!(store.retain_history_obligation(&early).await.is_err());
        assert!(store.release_history(&owner).await.is_err());
        let mut wrong=unknown.clone();wrong.attribution="another caller".into();
        assert!(store.release_history_obligation(&wrong).await.is_err());
        store.release_history_obligation(&unknown).await.unwrap();
        store.release_history(&owner).await.unwrap();
        assert!(store.begin_run(&request).await.is_err());
        assert!(store.claim_run(&request.key,"late-claim","late",Duration::from_secs(1)).await.is_err());
        assert!(store.canonical_attempt(&attempt).await.is_err());
        assert!(store.canonical_result_manifest(&attempt).await.is_err());
        assert!(store.settle_operation_for_run("absent","claim",&[],Some(&request.key)).await.is_err());
        assert_eq!(collect(&store,&owner).await,4);
        // Exact old identity dominates changed caller bytes and owner substitution.
        assert!(store.settle_operation_for_run("claim-exploratory","claim",&[9],Some("unrelated-new-run")).await.is_err());
        let mut response=bounded_query(store.db.query("SELECT * FROM ONLY type::record('canonical_execution_operations','seal-exploratory');")).await.unwrap();
        let row=crate::generated::surreal::decode_canonical_execution_operations(response.take::<Option<Object>>(0).unwrap().unwrap()).unwrap();
        assert!(row.minimal);assert!(row.request.is_empty() && row.result.is_empty());
        assert_eq!(row.run,request.key);assert_eq!(row.attempt.as_deref(),Some(attempt.as_str()));assert_eq!(row.disposition,"settled");
        assert_eq!(row.issuing_store,store.issuing_store().unwrap());
        let mut response=bounded_query(store.db.query("SELECT * FROM ONLY type::record('canonical_history_retirements','run:exploratory');")).await.unwrap();
        let row=crate::generated::surreal::decode_canonical_history_retirements(response.take::<Option<Object>>(0).unwrap().unwrap()).unwrap();
        assert!(row.complete);assert_eq!(row.outcome,"partial");
        store.release_history(&owner).await.unwrap();assert!(store.reclaim_history_page(&owner).await.unwrap().complete);
        assert!(store.canonical_run(&request.key).await.is_err());
        assert!(store.retain_history_obligation(&audit).await.is_err());
        store.remove_isolated_fixture().await.unwrap();
    }
}
