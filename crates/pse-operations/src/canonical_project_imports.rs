// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fresh project import composes the original source activation and artifact custody owners.
use crate::{canonical::{CanonicalError,CanonicalStore,bounded_query,protected_query},canonical_projects::ProjectPublication,canonical_artifacts::Artifact,generated::surreal as wire};
use crate::canonical::transport::{original_deadline,within_clock};
use crate::canonical::REQUEST_TIMEOUT;
use surrealdb::types::{Object,Value};
pub use pse_model::generated::runtime::canonical_project_imports::Row as ProjectImport;

/// Original bundle and presentation commitment, carried privately by source activation.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportBinding {
    pub(crate) bundle_digest:String,
    pub(crate) origin:Vec<u8>,
    pub(crate) title:String,
    pub(crate) branch_name:String,
}
impl ImportBinding {
    pub(crate) fn validate(&self)->Result<(),CanonicalError> {
        pse_ids::ContentHash::parse_hex(&self.bundle_digest).map_err(|_|CanonicalError::OperationReused)?;
        if self.origin.len()>131072 || self.title.len()>4096 || self.branch_name.is_empty() || self.branch_name.len()>4096 { return Err(CanonicalError::PayloadLimit); }
        Ok(())
    }
    pub(crate) fn encode(&self)->Result<Object,CanonicalError> {
        self.validate()?; let mut row=Object::new();
        row.insert("bundle_digest",self.bundle_digest.clone()); row.insert("origin",surrealdb::types::Bytes::from(self.origin.clone()));
        row.insert("title",self.title.clone()); row.insert("branch_name",self.branch_name.clone()); Ok(row)
    }
}
/// Original immutable operation. Retry this exact intent after cancellation or a lost reply.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectImportIntent { publication:ProjectPublication }
impl ProjectImportIntent {
    /// Original receiving source/project activation identity.
    pub fn operation(&self)->&str { self.publication.operation() }
    /// Original receiving authority, bundle and destination commitment are validated before effects.
    pub fn validate(&self)->Result<(),CanonicalError> { self.publication.validate()?; if self.publication.import_binding().is_none() {return Err(CanonicalError::OperationReused);} Ok(()) }
    /// Exact manifest encoding checksum.
    pub fn bundle_digest(&self)->&str { self.publication.import_binding().map_or("",|b|b.bundle_digest.as_str()) }
    /// Original descriptive origin body bound to the manifest checksum.
    pub fn origin_bytes(&self)->&[u8] {self.publication.import_binding().map_or(&[],|b|b.origin.as_slice())}
    /// Fresh receiving project/branch/problem identities.
    pub fn destinations(&self)->(&str,&str,&str) { self.publication.import_coordinates() }
    /// Guarded original source publication; not an imported historical operation.
    pub fn publication(&self)->&ProjectPublication { &self.publication }
}
impl CanonicalStore {
    /// Read-only issue binds fresh receiving IDs chosen by the caller and original bundle bytes.
    pub fn project_import_intent(&self,operation:&str,project:&str,branch:&str,problem:&str,bundle_digest:&str,origin:Vec<u8>,title:&str,branch_name:&str)->Result<ProjectImportIntent,CanonicalError> {
        Ok(ProjectImportIntent{publication:ProjectPublication::import_intent(self.issuing_store()?,operation,project,branch,problem,ImportBinding{bundle_digest:bundle_digest.into(),origin,title:title.into(),branch_name:branch_name.into()})?})
    }
    /// Original immutable checkpoint. Pending source admission is not public project availability.
    pub async fn project_import_record(&self,intent:&ProjectImportIntent)->Result<Option<ProjectImport>,CanonicalError> {
        let activity=self.admit_activity()?;activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),self.project_import_record_inner(intent))).await
    }
    async fn project_import_record_inner(&self,intent:&ProjectImportIntent)->Result<Option<ProjectImport>,CanonicalError> {
        intent.validate()?;
        // Original publication settlement verifies receiving authority and full request before mutable checks.
        let Some(_revision)=self.settled_project_publication(intent.publication()).await? else { return Ok(None); };
        let mut response=bounded_query(self.project_query("SELECT * FROM ONLY type::record('canonical_project_imports',$key);")?.bind(("key",intent.operation().to_owned()))).await?;
        let row=response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?.map(wire::decode_canonical_project_imports).transpose()?;
        if row.as_ref().is_none_or(|r|r.request.as_slice()!=intent.publication.request() || r.bundle_digest!=intent.bundle_digest()) { return Err(CanonicalError::OperationReused); }
        Ok(row)
    }
    /// Retain the one original artifact creation intent before starting its potentially uncertain copy.
    pub async fn project_import_bind_artifact(&self,intent:&ProjectImportIntent,artifact:&Artifact)->Result<(),CanonicalError> {
        let activity=self.admit_activity()?;activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),self.project_import_bind_artifact_inner(intent,artifact))).await
    }
    async fn project_import_bind_artifact_inner(&self,intent:&ProjectImportIntent,artifact:&Artifact)->Result<(),CanonicalError> {
        intent.validate()?; self.ensure_writes()?;
        let bytes=serde_json::to_vec(artifact).map_err(|e|CanonicalError::Configuration(e.to_string()))?;
        let provenance=serde_json::to_vec(&(1u8,intent.bundle_digest())).map_err(|e|CanonicalError::Configuration(e.to_string()))?;
        let result=protected_query("canonical_project_imports::bind",||Ok(self.project_query("RETURN fn::pse_project_import_v1::bind($pse_rpc_expires_at,$key,$request,$artifact,$bytes,$provenance);")?.bind(("key",intent.operation().to_owned())).bind(("request",surrealdb::types::Bytes::from(intent.publication.request().to_vec()))).bind(("artifact",Value::Object(wire::encode_canonical_artifacts(artifact)?))).bind(("bytes",surrealdb::types::Bytes::from(bytes.clone()))).bind(("provenance",surrealdb::types::Bytes::from(provenance.clone()))))).await;
        match result {
            Ok(mut response)=>{let _=response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?;Ok(())},
            Err(error)=>if self.project_import_record(intent).await?.is_some_and(|r|r.artifact.as_ref().is_some_and(|b|b.as_slice()==bytes.as_slice())) {Ok(())} else {Err(error)},
        }
    }
    /// Make project availability true only after exact canonical copied membership is active and owned.
    pub async fn finish_project_import(&self,intent:&ProjectImportIntent)->Result<ProjectImport,CanonicalError> {
        let activity=self.admit_activity()?;activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),self.finish_project_import_inner(intent))).await
    }
    async fn finish_project_import_inner(&self,intent:&ProjectImportIntent)->Result<ProjectImport,CanonicalError> {
        intent.validate()?; self.ensure_writes()?;
        let row=self.project_import_record(intent).await?.ok_or(CanonicalError::IncompleteResponse)?;
        let artifact:Artifact=serde_json::from_slice(row.artifact.as_ref().ok_or(CanonicalError::IncompleteResponse)?.as_slice()).map_err(|e|CanonicalError::Configuration(e.to_string()))?;
        let obligation=crate::canonical_artifacts::artifact_obligation(&artifact.key,"project",intent.operation());
        let result=protected_query("canonical_project_imports::finish",||Ok(self.project_query("RETURN fn::pse_project_import_v1::finish($pse_rpc_expires_at,$key,$request,$artifact,$bytes,$obligation);")?.bind(("key",intent.operation().to_owned())).bind(("request",surrealdb::types::Bytes::from(intent.publication.request().to_vec()))).bind(("artifact",Value::Object(wire::encode_canonical_artifacts(&artifact)?))).bind(("bytes",surrealdb::types::Bytes::from(row.artifact.as_ref().ok_or(CanonicalError::IncompleteResponse)?.as_slice().to_vec()))).bind(("obligation",Value::Object(wire::encode_canonical_artifact_obligations(&obligation)?))))).await;
        if result.is_err() { return self.project_import_record(intent).await?.filter(|r|r.complete).ok_or(CanonicalError::IncompleteResponse); }
        self.project_import_record(intent).await?.filter(|r|r.complete).ok_or(CanonicalError::IncompleteResponse)
    }
}
