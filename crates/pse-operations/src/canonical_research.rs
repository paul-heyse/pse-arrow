// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact immutable research references; scientific values stay in admitted sources/results.
use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, REQUEST_TIMEOUT, bounded_query, protected_query},
    canonical_codec as codec,
    generated::surreal as wire,
};
pub use pse_model::generated::runtime::{
    canonical_research_datasets::Row as ResearchDataset,
    canonical_research_publications::Row as ResearchPublication,
    canonical_research_selections::Row as ResearchSelection,
};
use surrealdb::types::Object;
fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json::to_vec(value).map_err(|e| CanonicalError::Configuration(e.to_string()))
}
fn identity(key: &str) -> Result<(), CanonicalError> {
    if key.is_empty() || key.len() > 4096 {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}
impl CanonicalStore {
    fn research_query(
        &self,
        sql: &str,
    ) -> Result<surrealdb::method::Query<'_, surrealdb::engine::remote::grpc::Client>, CanonicalError>
    {
        Ok(self
            .db
            .query(format!(
                "BEGIN; fn::pse_project_v1::authority($pse_rpc_expires_at,$issuer); {sql} COMMIT;"
            ))
            .bind(("issuer", self.issuing_store()?.to_owned())))
    }
    /// Append one immutable admitted-object version. Replays compare complete original inputs.
    pub async fn create_research_dataset(
        &self,
        project: &crate::canonical_projects::Project,
        mut row: ResearchDataset,
    ) -> Result<ResearchDataset, CanonicalError> {
        for key in [
            &row.key,
            &row.project,
            &row.revision,
            &row.context,
            &row.physical_context,
        ] {
            identity(key)?;
        }
        if row.project_domain() != project.key_domain() || row.members.len() > 131072 {
            return Err(CanonicalError::PayloadLimit);
        }
        row.request = encode(&(
            "research-dataset-v1",
            self.issuing_store()?,
            project.generation,
            &row.key,
            &row.project,
            &row.revision,
            &row.physical_context,
            &row.context,
            &row.parent,
            &row.role,
            &row.members,
        ))?
        .into();
        let object = wire::encode_canonical_research_datasets(&row)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{self.ensure_writes()?;
            let query=||Ok(self.research_query("RETURN fn::pse_research_v1::dataset($pse_rpc_expires_at,$row,$generation);")?.bind(("row",object.clone())).bind(("generation",codec::encode_uint(project.generation)?)));
            let response = protected_query("canonical_research::dataset",query).await;
            #[cfg(any(test, feature = "test-support"))]
            let response = if response.is_ok() && crate::canonical_projects::simulate_reply_loss() { Err(CanonicalError::IncompleteResponse) } else { response };
            let write = response.and_then(|mut response| {
                response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_research_datasets).transpose()?
                    .ok_or(CanonicalError::IncompleteResponse)
            });
            crate::canonical::settlement::settle_exact(
                "research-dataset", &row.key, row.request.as_slice(), write,
                self.research_dataset(&row.key),
                |got| got.request == row.request,
                CanonicalError::source_transaction_refused,
            ).await
        })).await
    }
    /// Append one frozen fit/observation context through the original source roots.
    pub async fn create_research_selection(
        &self,
        project: &crate::canonical_projects::Project,
        mut row: ResearchSelection,
    ) -> Result<ResearchSelection, CanonicalError> {
        for key in [
            &row.key,
            &row.project,
            &row.dataset,
            &row.revision,
            &row.context,
            &row.fit,
        ] {
            identity(key)?;
        }
        if row.project_domain() != project.key_domain() || row.selection.len() > 131072 {
            return Err(CanonicalError::PayloadLimit);
        }
        row.request = encode(&(
            "research-selection-v1",
            self.issuing_store()?,
            project.generation,
            &row.key,
            &row.project,
            &row.dataset,
            &row.revision,
            &row.physical_context,
            &row.context,
            &row.fit,
            &row.selection,
        ))?
        .into();
        let object = wire::encode_canonical_research_selections(&row)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{self.ensure_writes()?;
            let query=||Ok(self.research_query("RETURN fn::pse_research_v1::selection($pse_rpc_expires_at,$row,$generation);")?.bind(("row",object.clone())).bind(("generation",codec::encode_uint(project.generation)?)));
            let response = protected_query("canonical_research::selection",query).await;
            #[cfg(any(test, feature = "test-support"))]
            let response = if response.is_ok() && crate::canonical_projects::simulate_reply_loss() { Err(CanonicalError::IncompleteResponse) } else { response };
            let write = response.and_then(|mut response| {
                response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_research_selections).transpose()?
                    .ok_or(CanonicalError::IncompleteResponse)
            });
            crate::canonical::settlement::settle_exact(
                "research-selection", &row.key, row.request.as_slice(), write,
                self.research_selection(&row.key),
                |got| got.request == row.request,
                CanonicalError::source_transaction_refused,
            ).await
        })).await
    }
    /// Select exact immutable metadata; this grants no payload read lifetime.
    pub async fn research_dataset(
        &self,
        key: &str,
    ) -> Result<Option<ResearchDataset>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            let mut r=bounded_query(self.research_query("RETURN SELECT * FROM ONLY type::record('canonical_research_datasets',$key) WHERE fn::pse_project_import_v1::available(project);")?.bind(("key",key.to_owned()))).await?;
            r.take::<Option<Object>>(r.num_statements().saturating_sub(2))?.map(wire::decode_canonical_research_datasets).transpose().map_err(Into::into)
        })).await
    }
    /// Select exact frozen observation intent without reading measurement or result payloads.
    pub async fn research_selection(
        &self,
        key: &str,
    ) -> Result<Option<ResearchSelection>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            let mut r=bounded_query(self.research_query("RETURN SELECT * FROM ONLY type::record('canonical_research_selections',$key) WHERE fn::pse_project_import_v1::available(project);")?.bind(("key",key.to_owned()))).await?;
            r.take::<Option<Object>>(r.num_statements().saturating_sub(2))?.map(wire::decode_canonical_research_selections).transpose().map_err(Into::into)
        })).await
    }
    /// Exact atomic source-publication receipt; values remain in original source/result owners.
    pub async fn research_publication(
        &self,
        key: &str,
    ) -> Result<Option<ResearchPublication>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            let mut r=bounded_query(self.research_query("RETURN SELECT * FROM ONLY type::record('canonical_research_publications',$key) WHERE fn::pse_project_import_v1::available(project);")?.bind(("key",key.to_owned()))).await?;
            r.take::<Option<Object>>(r.num_statements().saturating_sub(2))?.map(wire::decode_canonical_research_publications).transpose().map_err(Into::into)
        })).await
    }
    /// Bounded indexed metadata page, ordered by immutable key. Continue after the last key.
    pub async fn research_datasets(
        &self,
        project: &str,
        after: &str,
        limit: usize,
    ) -> Result<Vec<ResearchDataset>, CanonicalError> {
        identity(project)?;
        if limit == 0 || limit > 128 {
            return Err(CanonicalError::PayloadLimit);
        }
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            let mut r=bounded_query(self.research_query("IF !fn::pse_project_import_v1::available($project) {THROW 'PSE_REFUSAL:project_source_unavailable';}; SELECT * FROM canonical_research_datasets WITH INDEX research_dataset_project WHERE project=$project AND fn::pse_project_import_v1::available(project) AND key>$after ORDER BY key LIMIT $limit;")?.bind(("project",project.to_owned())).bind(("after",after.to_owned())).bind(("limit",limit))).await?;
            r.take::<Vec<Object>>(r.num_statements().saturating_sub(2))?.into_iter().map(|o|wire::decode_canonical_research_datasets(o).map_err(Into::into)).collect()
        })).await
    }
}


#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_research_server_unit {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "isolated research receipt fixtures assert exact committed metadata")]
    use super::*;

    #[tokio::test]
    async fn research_lost_ack_settles_original_requests_and_reuse_is_explicit() {
        let state = std::env::var("PSE_SURREAL_STATE").expect("canonical fixture service");
        let mut options = crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_research_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let source = store.edit("science", None, "source", &[]).await.unwrap();
        store.create_project("create", "project", "Research").await.unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        store.append_project_context("context", &project, "scope", &source.key, "source-hash", "physical", &source.key, "physical-hash", 0, None, None, false).await.unwrap();
        // These are exact metadata receipt controls; scientific admission stays
        // with the runtime research owner and is not inferred from these bytes.
        let dataset = ResearchDataset {
            key: "dataset".into(), project: project.key.clone(), revision: source.key.clone(),
            physical_context: "physical".into(), context: "context".into(), parent: None,
            role: "raw".into(), members: b"[]".to_vec().into(), request: Vec::new().into(),
        };
        let committed = crate::canonical_projects::PROJECT_REPLY_LOSS.scope(true, store.create_research_dataset(&project, dataset.clone())).await.unwrap();
        assert_eq!(store.research_dataset("dataset").await.unwrap(), Some(committed.clone()));
        assert_eq!(store.create_research_dataset(&project, dataset.clone()).await.unwrap(), committed);
        let mut changed = dataset;
        changed.members = b"[1]".to_vec().into();
        assert!(matches!(store.create_research_dataset(&project, changed).await, Err(CanonicalError::OperationReused)));
        let selection = ResearchSelection {
            key: "selection".into(), project: project.key.clone(), dataset: committed.key,
            revision: source.key, physical_context: "physical".into(), context: "context".into(),
            fit: "fit".into(), selection: b"[]".to_vec().into(), request: Vec::new().into(),
        };
        let committed = crate::canonical_projects::PROJECT_REPLY_LOSS.scope(true, store.create_research_selection(&project, selection.clone())).await.unwrap();
        assert_eq!(store.research_selection("selection").await.unwrap(), Some(committed.clone()));
        assert_eq!(store.create_research_selection(&project, selection.clone()).await.unwrap(), committed);
        let mut changed = selection;
        changed.fit = "different-fit".into();
        assert!(matches!(store.create_research_selection(&project, changed).await, Err(CanonicalError::OperationReused)));
        store.remove_isolated_fixture().await.unwrap();
    }
}
