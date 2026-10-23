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
        if row.project != project.key || row.members.len() > 131072 {
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
            match protected_query("canonical_research::dataset",query).await {
                Ok(mut r)=>r.take::<Option<Object>>(r.num_statements().saturating_sub(2))?.map(wire::decode_canonical_research_datasets).transpose()?.ok_or(CanonicalError::IncompleteResponse),
                Err(e)=>match self.research_dataset(&row.key).await?{Some(old) if old.request==row.request=>Ok(old),_=>Err(e)},
            }
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
        if row.project != project.key || row.selection.len() > 131072 {
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
            match protected_query("canonical_research::selection",query).await {
                Ok(mut r)=>r.take::<Option<Object>>(r.num_statements().saturating_sub(2))?.map(wire::decode_canonical_research_selections).transpose()?.ok_or(CanonicalError::IncompleteResponse),
                Err(e)=>match self.research_selection(&row.key).await?{Some(old) if old.request==row.request=>Ok(old),_=>Err(e)},
            }
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
