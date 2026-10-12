// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent selected scientific content, exact creation and attributed owners.

use crate::{
    canonical::{CanonicalError, CanonicalStore, ProtectedSelection, Revision, bounded_query, protected_query},
    canonical_codec as codec,
    canonical_results::ResultRead,
    generated::surreal as wire,
};
use pse_ids::{Frame, FramedHasher};
use pse_model::SemanticFrame;
use std::time::Duration;
use surrealdb::types::Object;

pub use pse_model::generated::runtime::{
    canonical_artifacts::Row as Artifact,
    canonical_artifact_blocks::Row as ArtifactBlock,
    canonical_artifact_obligations::Row as ArtifactObligation,
};

/// A copied block locator retains scientific row and field coordinates, not a producer read.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactLocator {
    /// Independent occurrence authority.
    pub artifact: String,
    /// Artifact-owned standalone IPC block.
    pub block: u64,
    /// Row within that block, when quantitative evidence selects one row.
    pub row: Option<u64>,
    /// Exact generated scientific field, when selecting one value.
    pub field: Option<String>,
}

/// Explicit owner identity. Other obligations are never released by this value.
pub fn artifact_obligation(artifact: &str, owner_kind: &str, owner: &str) -> ArtifactObligation {
    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.artifact.obligation.v1").str(artifact).str(owner_kind).str(owner);
    ArtifactObligation { key:hash.finish_hash().to_hex(),artifact:artifact.into(),owner_kind:owner_kind.into(),owner:owner.into() }
}

/// Deterministic coordinate within one original occurrence; content identities stay unchanged.
pub fn artifact_block(artifact: &str, ordinal: u64, relation: &str, payload: Vec<u8>, original_digest: &str, rows: u64) -> ArtifactBlock {
    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.artifact.block.v1").str(artifact).u64(ordinal);
    let key=hash.finish_hash().to_hex();
    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.execution.result-payload.v1").part(&payload);
    ArtifactBlock { key,artifact:artifact.into(),ordinal,relation:relation.into(),payload:payload.into(),digest:hash.finish_hash().to_hex(),original_digest:original_digest.into(),rows }
}

/// Bind the exact ordered scientific closure before the first durable effect.
pub fn seal_artifact_request(header: &mut Artifact, sources: &[Revision], inputs: &[ResultRead], blocks: &[ArtifactBlock]) {
    header.creation_request_digest.clear();
    header.creation_request_digest=request_digest(header,sources,inputs,blocks);
}
fn request_digest(header:&Artifact,sources:&[Revision],inputs:&[ResultRead],blocks:&[ArtifactBlock])->String {
    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.artifact.complete-request.v1");
    let mut row=header.clone(); row.creation_request_digest.clear(); row.active=false; row.retiring=false;
    row.frame(&mut hash);
    hash.u64(sources.len() as u64);
    for source in sources { source.frame(&mut hash); }
    hash.u64(inputs.len() as u64);
    for input in inputs { input.run().frame(&mut hash); input.attempt().frame(&mut hash); input.manifest().frame(&mut hash); hash.str(input.protected_selection().key()); }
    hash.u64(blocks.len() as u64);
    for block in blocks { block.frame(&mut hash); }
    hash.finish_hash().to_hex()
}
fn invalid(message:&str)->CanonicalError { CanonicalError::Configuration(message.into()) }
fn object(response:&mut surrealdb::IndexedResults)->Result<Object,CanonicalError> {
    let index=response.num_statements().saturating_sub(2);
    response.take::<Option<Object>>(index)?.ok_or(CanonicalError::IncompleteResponse)
}

/// An acquired current grant. Cloned decoded arrays own bytes independently of this DB grant.
#[derive(Clone, Debug)]
pub struct ArtifactRead { store:CanonicalStore, header:Artifact, selection:ProtectedSelection }
impl ArtifactRead {
    pub(crate) fn belongs_to(&self,store:&CanonicalStore)->bool { self.store.db.same_driver(&store.db) }
    pub(crate) fn protected_selection(&self)->&ProtectedSelection { &self.selection }
    /// Exact immutable occurrence metadata.
    pub fn header(&self)->&Artifact { &self.header }
    /// Renew this existing finite acquisition, never mint a new owner from copied metadata.
    pub async fn renew(&self,lifetime:Duration)->Result<(),CanonicalError> { self.selection.renew(lifetime).await }
    /// Explicit release is idempotent; the original grant remains fenced through its issue window.
    pub async fn release(&self)->Result<(),CanonicalError> { self.store.release(&self.selection).await }
    /// Read one bounded member under the exact artifact acquisition.
    pub async fn block(&self,ordinal:u64)->Result<ArtifactBlock,CanonicalError> {
        let activity=self.store.admit_activity()?;
        activity.scope(async {
            let value=codec::encode_uint(ordinal)?;
            let mut response=protected_query("canonical_artifacts::read",||Ok(self.store.db.query("BEGIN; RETURN fn::pse_artifact_v1::read($pse_rpc_expires_at,$key,$protection,$ordinal); COMMIT;")
                .bind(("key",self.header.key.clone())).bind(("protection",self.selection.key().to_owned())).bind(("ordinal",value.clone())))).await?;
            let row=wire::decode_canonical_artifact_blocks(object(&mut response)?)?;
            if row.artifact!=self.header.key || row.ordinal!=ordinal || row.payload.len()>wire::RESULT_BLOCK_BYTES { return Err(invalid("artifact block coordinate differs")); }
            let copied=artifact_block(&row.artifact,row.ordinal,&row.relation,row.payload.to_vec(),&row.original_digest,row.rows);
            if copied.key!=row.key || copied.digest!=row.digest { return Err(invalid("artifact copied encoding digest differs")); }
            Ok(row)
        }).await
    }
}

impl CanonicalStore {
    /// Abandon one exact pre-header study staging identity. The permanent fence
    /// is published before bounded cleanup and rejects every late staging call.
    /// Existing study headers require the ordinary study retirement owner.
    pub async fn abandon_study_inputs(&self,study:&str)->Result<bool,CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            if study.is_empty() || study.len()>4096 { return Err(CanonicalError::PayloadLimit); }
            let mut response=protected_query("canonical_artifacts::abandon_study_inputs",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::abandon_study_inputs($pse_rpc_expires_at,$study); COMMIT;").bind(("study",study.to_owned())))).await?;
            let index=response.num_statements().saturating_sub(2);
            response.take::<Option<bool>>(index)?.ok_or(CanonicalError::IncompleteResponse)
        }).await
    }
    /// Read-only issue of a finite original intent. Seal it once and retain it through uncertainty.
    pub async fn new_artifact(&self, revision:&Revision, kind:&str, provenance:Vec<u8>, block_count:u64, payload_bytes:u64)->Result<Artifact,CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            if kind.is_empty() || kind.len()>128 || provenance.len()>128*1024 || block_count==0 { return Err(CanonicalError::PayloadLimit); }
            let mut response=bounded_query(self.db.query("LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$problem); IF $guard=NONE OR $guard.incarnation=NONE { THROW 'artifact source authority unavailable'; }; RETURN {authority:$guard.incarnation,now:time::micros()};").bind(("problem",revision.problem.clone()))).await?;
            let index=response.num_statements().saturating_sub(1);
            let mut row=response.take::<Option<Object>>(index)?.ok_or(CanonicalError::IncompleteResponse)?;
            let primary_authority=codec::decode_string(codec::required(&mut row,"authority")?)?;
            let now=u64::try_from(codec::decode_int(codec::required(&mut row,"now")?)?).map_err(|_|invalid("negative artifact clock"))?;
            let creation_expires_at=now.checked_add(60_000_000).ok_or(CanonicalError::PayloadLimit)?;
            let creation_nonce=uuid::Uuid::new_v4().to_string();
            Ok(Artifact { key:format!("pse.artifact.v1:{primary_authority}:{creation_nonce}:{creation_expires_at}"),kind:kind.into(),revision:revision.key.clone(),primary_problem:revision.problem.clone(),primary_authority,creation_nonce,creation_expires_at,creation_request_digest:String::new(),interpretation:wire::INTERPRETATION.into(),provenance:provenance.into(),block_count,payload_bytes,active:false,retiring:false })
        }).await
    }
    /// Stage bounded content and transfer exact producer closure to independent ownership atomically.
    pub async fn persist_artifact(&self, header:&Artifact, sources:&[Revision], inputs:&[ResultRead], blocks:&[ArtifactBlock], owner_kind:&str, owner:&str)->Result<Artifact,CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            if header.creation_request_digest!=request_digest(header,sources,inputs,blocks) { return Err(CanonicalError::OperationReused); }
            let result=async {
                self.ensure_writes()?;
                if header.active || header.retiring || header.interpretation!=wire::INTERPRETATION || header.block_count!=blocks.len() as u64 || sources.is_empty() || sources.len()>65 || inputs.len()>64 || inputs.iter().any(|input|!input.belongs_to(self)) { return Err(invalid("artifact exact closure differs")); }
                let total=blocks.iter().enumerate().try_fold(0u64,|total,(ordinal,block)|{
                    let copied=artifact_block(&header.key,ordinal as u64,&block.relation,block.payload.to_vec(),&block.original_digest,block.rows);
                    if block!=&copied || block.payload.len()>wire::RESULT_BLOCK_BYTES { return Err(invalid("artifact block admission differs")); }
                    total.checked_add(block.payload.len() as u64).ok_or(CanonicalError::PayloadLimit)
                })?;
                if total!=header.payload_bytes { return Err(invalid("artifact payload extent differs")); }
                let mut roots=Vec::with_capacity(sources.len());
                for source in sources {
                    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1); hash.str("pse.artifact.root.v1").str(&header.key).str(&source.key);
                    let mut row=Object::new(); row.insert("key",hash.finish_hash().to_hex()); row.insert("problem",source.problem.clone()); row.insert("revision",source.key.clone()); row.insert("sequence",codec::encode_uint(source.sequence)?); roots.push(row);
                }
                let mut selected=Vec::with_capacity(inputs.len());
                for input in inputs {
                    let mut hash=FramedHasher::new(Frame::CanonicalPayloadV1); hash.str("pse.artifact.input.v1").str(&header.key).str(&input.attempt().key).str(&input.manifest().key);
                    let mut row=Object::new(); row.insert("key",hash.finish_hash().to_hex()); row.insert("artifact",header.key.clone()); row.insert("run",input.run().key.clone()); row.insert("attempt",input.attempt().key.clone()); row.insert("manifest",input.manifest().key.clone()); row.insert("protection",input.protected_selection().key().to_owned()); selected.push(row);
                }
                let row=wire::encode_canonical_artifacts(header)?;
                protected_query("canonical_artifacts::begin",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::begin($pse_rpc_expires_at,$row,$sources,$inputs); COMMIT;").bind(("row",row.clone())).bind(("sources",roots.clone())).bind(("inputs",selected.clone())))).await?;
                for block in blocks {
                    let row=wire::encode_canonical_artifact_blocks(block)?;
                    protected_query("canonical_artifacts::append",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::append($pse_rpc_expires_at,$key,$block); COMMIT;").bind(("key",header.key.clone())).bind(("block",row.clone())))).await?;
                }
                let obligation=wire::encode_canonical_artifact_obligations(&artifact_obligation(&header.key,owner_kind,owner))?;
                let mut response=protected_query("canonical_artifacts::activate",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::activate($pse_rpc_expires_at,$key,$obligation); COMMIT;").bind(("key",header.key.clone())).bind(("obligation",obligation.clone())))).await?;
                Ok(wire::decode_canonical_artifacts(object(&mut response)?)?)
            }.await;
            result.map_err(|source|CanonicalError::ArtifactUnsettled { intent:Box::new(header.clone()),source:Box::new(source) })
        }).await
    }
    /// Metadata is discovery only; content requires a current protected acquisition.
    pub async fn artifact(&self,key:&str)->Result<Artifact,CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            let mut response=protected_query("canonical_artifacts::artifact",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::available($pse_rpc_expires_at,$key); COMMIT;").bind(("key",key.to_owned())))).await?;
            Ok(wire::decode_canonical_artifacts(object(&mut response)?)?)
        }).await
    }
    /// Acquire under independent artifact authority; producer payload is never consulted.
    pub async fn read_artifact(&self,key:&str,lifetime:Duration)->Result<ArtifactRead,CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            let header=self.artifact(key).await?;
            let revision=self.revision(&header.revision).await?.ok_or_else(||invalid("artifact source absent"))?;
            let selection=self.protect(revision,lifetime).await?;
            let mut response=protected_query("canonical_artifacts::acquire",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::acquire($pse_rpc_expires_at,$key,$protection); COMMIT;").bind(("key",key.to_owned())).bind(("protection",selection.key().to_owned())))).await?;
            let actual=wire::decode_canonical_artifacts(object(&mut response)?)?;
            if actual!=header { return Err(invalid("artifact changed during acquisition")); }
            Ok(ArtifactRead { store:self.clone(),header,selection })
        }).await
    }
    /// Attach a named independent obligation to an active occurrence.
    pub async fn retain_artifact(&self,key:&str,owner_kind:&str,owner:&str)->Result<(),CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let row=wire::encode_canonical_artifact_obligations(&artifact_obligation(key,owner_kind,owner))?;
            protected_query("canonical_artifacts::retain",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::attach($pse_rpc_expires_at,$key,$obligation,true); COMMIT;").bind(("key",key.to_owned())).bind(("obligation",row.clone())))).await?; Ok(())
        }).await
    }
    /// Release exactly one owner, preserving all independent owners and read grants.
    pub async fn release_artifact(&self,key:&str,owner_kind:&str,owner:&str)->Result<(),CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let row=wire::encode_canonical_artifact_obligations(&artifact_obligation(key,owner_kind,owner))?;
            protected_query("canonical_artifacts::release",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::release($pse_rpc_expires_at,$obligation); COMMIT;").bind(("obligation",row.clone())))).await?; Ok(())
        }).await
    }
    /// Retire or settle the exact original intent, reclaiming one bounded page per call.
    pub async fn collect_artifact(&self,intent:&Artifact)->Result<bool,CanonicalError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let row=wire::encode_canonical_artifacts(intent)?;
            let mut response=protected_query("canonical_artifacts::collect",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::collect($pse_rpc_expires_at,$intent); COMMIT;").bind(("intent",row.clone())))).await?;
            Ok(codec::decode_boolean(codec::required(&mut object(&mut response)?,"complete")?)?)
        }).await
    }
}

#[cfg(all(test,feature="canonical-tests"))]
mod canonical_artifact_tests {
    #![allow(clippy::unwrap_used,clippy::expect_used,reason="isolated owned artifact fixtures require valid native setup and assertions")]
    use super::*;
    async fn fixture()->(CanonicalStore,Revision) {
        let state=std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options=crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database=format!("canonical_test_artifact_{}",uuid::Uuid::new_v4().simple());
        let store=crate::testing::canonical_fixture_with_options(&options,true).unwrap();
        let revision=store.edit("artifact-source",None,"original-source",&[]).await.unwrap();
        (store,revision)
    }
    async fn content(store:&CanonicalStore,revision:&Revision)->(Artifact,Vec<ArtifactBlock>) {
        let bytes=vec![0,255,1,128];
        let mut intent=store.new_artifact(revision,"test-exact-capsule",vec![7],1,bytes.len() as u64).await.unwrap();
        // Test controls issue a shorter original window before sealing/effects.
        intent.creation_expires_at-=59_500_000;
        intent.key=format!("pse.artifact.v1:{}:{}:{}",intent.primary_authority,intent.creation_nonce,intent.creation_expires_at);
        let blocks=vec![artifact_block(&intent.key,0,"capsule",bytes,"original-encoding",1)];
        seal_artifact_request(&mut intent,std::slice::from_ref(revision),&[],&blocks);
        (intent,blocks)
    }
    #[tokio::test]
    async fn canonical_artifact_independent_owners_and_exact_copied_read() {
        let (store,revision)=fixture().await;
        let (intent,blocks)=content(&store,&revision).await;
        let saved=store.persist_artifact(&intent,std::slice::from_ref(&revision),&[],&blocks,"user","alice").await.unwrap();
        store.retain_artifact(&saved.key,"audit","audit-1").await.unwrap();
        store.release_artifact(&saved.key,"user","alice").await.unwrap();
        assert!(store.collect_artifact(&intent).await.is_err(),"another attributed obligation prevents collection");
        let read=store.read_artifact(&saved.key,Duration::from_secs(10)).await.unwrap();
        let copied=read.block(0).await.unwrap();
        assert_eq!(copied.payload.as_slice(),[0,255,1,128]);
        assert_eq!(copied.original_digest,"original-encoding");
        assert_eq!(copied.digest,crate::canonical_execution::result_payload_digest(&copied.payload));
        read.release().await.unwrap();
        drop(read);
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_owner_release_fences_both_acquisition_orders() {
        let (store,revision)=fixture().await;
        let (intent,blocks)=content(&store,&revision).await;
        store.persist_artifact(&intent,std::slice::from_ref(&revision),&[],&blocks,"audit","independent").await.unwrap();
        store.retain_artifact(&intent.key,"user","retain-first").await.unwrap();
        store.release_artifact(&intent.key,"user","retain-first").await.unwrap();
        assert!(store.retain_artifact(&intent.key,"user","retain-first").await.is_err());
        store.release_artifact(&intent.key,"project","release-first").await.unwrap();
        store.release_artifact(&intent.key,"project","release-first").await.unwrap();
        assert!(store.retain_artifact(&intent.key,"project","release-first").await.is_err());
        store.retain_artifact(&intent.key,"user","new-independent-lifetime").await.unwrap();
        assert!(store.collect_artifact(&intent).await.is_err());
        assert!(store.read_artifact(&intent.key,Duration::from_secs(10)).await.is_ok(),"other owners remain available");
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_bounded_collection_fences_late_original_creation() {
        let (store,revision)=fixture().await;
        let (intent,blocks)=content(&store,&revision).await;
        store.persist_artifact(&intent,std::slice::from_ref(&revision),&[],&blocks,"project","p").await.unwrap();
        store.release_artifact(&intent.key,"project","p").await.unwrap();
        assert!(!store.collect_artifact(&intent).await.unwrap());
        assert!(store.artifact(&intent.key).await.is_err(),"retirement closes fresh acquisition");
        tokio::time::sleep(Duration::from_millis(550)).await;
        let mut complete=false;
        for _ in 0..8 { if store.collect_artifact(&intent).await.unwrap() { complete=true; break; } }
        assert!(complete);
        assert!(store.persist_artifact(&intent,std::slice::from_ref(&revision),&[],&blocks,"project","p").await.is_err(),"the original finite intent cannot recreate a collected occurrence");
        assert!(store.collect_artifact(&intent).await.unwrap(),"same original settlement is idempotent after collection");
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_exact_seal_refuses_changed_content_before_effects() {
        let (store,revision)=fixture().await;
        let (intent,mut blocks)=content(&store,&revision).await;
        blocks[0].payload=vec![9].into();
        assert!(matches!(store.persist_artifact(&intent,std::slice::from_ref(&revision),&[],&blocks,"user","alice").await,Err(CanonicalError::OperationReused)));
        assert!(store.artifact(&intent.key).await.is_err());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_preheader_study_abandonment_fences_both_orders() {
        let (store,revision)=fixture().await;
        let (mut intent,blocks)=content(&store,&revision).await;
        intent.kind="seed".into();
        seal_artifact_request(&mut intent,std::slice::from_ref(&revision),&[],&blocks);
        store.persist_artifact(&intent,std::slice::from_ref(&revision),&[],&blocks,"user","independent-owner").await.unwrap();
        store.retain_study_input("staging-first","solution",&intent.key).await.unwrap();
        assert_eq!(store.study_input("staging-first","solution").await.unwrap(),intent.key);
        assert!(store.abandon_study_inputs("staging-first").await.unwrap());
        assert!(store.abandon_study_inputs("staging-first").await.unwrap(),"same abandonment safely settles a lost ACK");
        assert!(store.study_input("staging-first","solution").await.is_err());
        assert!(store.retain_study_input("staging-first","solution",&intent.key).await.is_err(),"late original staging cannot recreate an abandoned identity");
        assert!(store.abandon_study_inputs("abandonment-first").await.unwrap());
        assert!(store.retain_study_input("abandonment-first","solution",&intent.key).await.is_err());
        assert!(store.retain_artifact(&intent.key,"study","abandonment-first").await.is_err());
        assert!(store.artifact(&intent.key).await.unwrap().active,"abandoning either study does not release the independent user");
        let mut response=bounded_query(store.db.query("SELECT key FROM canonical_artifact_obligations WHERE owner_kind='study'; SELECT key FROM canonical_study_inputs;")).await.unwrap();
        assert!(response.take::<Vec<Object>>(0).unwrap().is_empty());
        assert!(response.take::<Vec<Object>>(1).unwrap().is_empty());
        store.remove_isolated_fixture().await.unwrap();
    }
}
