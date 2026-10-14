// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent selected scientific content, exact creation and attributed owners.

use crate::{
    canonical::{
        CanonicalError, CanonicalStore, ProtectedSelection, Revision, bounded_query,
        protected_query,
    },
    canonical_codec as codec,
    canonical_results::ResultRead,
    generated::surreal as wire,
};
use pse_ids::{Frame, FramedHasher};
use pse_model::SemanticFrame;
#[cfg(all(test, feature = "canonical-tests"))]
use std::time::Duration;
use surrealdb::types::Object;

pub use pse_model::generated::runtime::{
    canonical_artifact_obligations::Row as ArtifactObligation,
    canonical_artifacts::Row as Artifact,
};

/// Materialized artifact member. The catalog stores only its sealed file binding.
#[derive(Clone, Debug)]
pub struct ArtifactBlock {
    /// Original immutable artifact member identity.
    pub key:String,
    /// Independent artifact occurrence.
    pub artifact:String,
    /// Original member ordinal.
    pub ordinal:u64,
    /// Scientific relation or opaque capsule label.
    pub relation:String,
    /// Original exact copied bytes.
    pub payload:pse_model::Bytes,
    /// Exact copied encoding digest.
    pub digest:String,
    /// Producer encoding provenance, distinct from copied digest.
    pub original_digest:String,
    /// Original row coverage.
    pub rows:u64,
    allocation:Option<std::sync::Arc<dyn std::fmt::Debug+Send+Sync>>,
    encoding:String,
    schema:String,
    batch_count:u64,
}
impl ArtifactBlock {
    /// Bind a Runtime-validated standalone native IPC file to its exact schema.
    /// Capsules retain their original JSON/opaque encoding instead.
    pub fn native_file(mut self,schema:&str)->Result<Self,CanonicalError>{
        if schema.is_empty()||!self.payload.starts_with(b"ARROW1")||!self.payload.ends_with(b"ARROW1"){return Err(invalid("artifact native file/schema binding invalid"));}
        self.encoding=crate::canonical_files::ARROW_IPC_FILE.into();self.schema=schema.into();self.batch_count=1;Ok(self)
    }
}
impl PartialEq for ArtifactBlock { fn eq(&self,other:&Self)->bool{self.key==other.key&&self.artifact==other.artifact&&self.ordinal==other.ordinal&&self.relation==other.relation&&self.payload==other.payload&&self.digest==other.digest&&self.original_digest==other.original_digest&&self.rows==other.rows&&self.encoding==other.encoding&&self.schema==other.schema&&self.batch_count==other.batch_count} }
impl Eq for ArtifactBlock {}
impl SemanticFrame for ArtifactBlock {fn frame(&self,hash:&mut FramedHasher){hash.str(&self.key).str(&self.artifact).u64(self.ordinal).str(&self.relation).part(self.payload.as_slice()).str(&self.digest).str(&self.original_digest).u64(self.rows).str(&self.encoding).str(&self.schema).u64(self.batch_count);}}

/// A copied block locator retains scientific row and field coordinates, not a producer read.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
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
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.artifact.obligation.v1")
        .str(artifact)
        .str(owner_kind)
        .str(owner);
    ArtifactObligation {
        key: hash.finish_hash().to_hex(),
        artifact: artifact.into(),
        owner_kind: owner_kind.into(),
        owner: owner.into(),
    }
}

/// Deterministic coordinate within one original occurrence; content identities stay unchanged.
pub fn artifact_block(
    artifact: &str,
    ordinal: u64,
    relation: &str,
    payload: Vec<u8>,
    original_digest: &str,
    rows: u64,
) -> ArtifactBlock {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.artifact.block.v1").str(artifact).u64(ordinal);
    let key = hash.finish_hash().to_hex();
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.execution.result-payload.v1").part(&payload);
    let encoding=if serde_json::from_slice::<serde_json::Value>(&payload).is_ok(){crate::canonical_files::OPAQUE_JSON}else{crate::canonical_files::OPAQUE_BYTES};
    ArtifactBlock {
        key,
        artifact: artifact.into(),
        ordinal,
        relation: relation.into(),
        payload: payload.into(),
        digest: hash.finish_hash().to_hex(),
        original_digest: original_digest.into(),
        rows,
        allocation:None,
        encoding:encoding.into(),schema:String::new(),batch_count:0,
    }
}

/// Bind the exact ordered scientific closure before the first durable effect.
pub fn seal_artifact_request(
    header: &mut Artifact,
    sources: &[Revision],
    inputs: &[ResultRead],
    blocks: &[ArtifactBlock],
    owner_kind: &str,
    owner: &str,
) {
    header.creation_request_digest.clear();
    header.creation_request_digest =
        request_digest(header, sources, inputs, blocks, owner_kind, owner);
}
fn request_digest(
    header: &Artifact,
    sources: &[Revision],
    inputs: &[ResultRead],
    blocks: &[ArtifactBlock],
    owner_kind: &str,
    owner: &str,
) -> String {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.artifact.complete-request.v1")
        .str(owner_kind)
        .str(owner);
    let mut row = header.clone();
    row.creation_request_digest.clear();
    row.active = false;
    row.retiring = false;
    row.frame(&mut hash);
    hash.u64(sources.len() as u64);
    for source in sources {
        source.frame(&mut hash);
    }
    hash.u64(inputs.len() as u64);
    for input in inputs {
        input.run().frame(&mut hash);
        input.attempt().frame(&mut hash);
        input.manifest().frame(&mut hash);
        hash.str(input.protected_selection().key());
    }
    hash.u64(blocks.len() as u64);
    for block in blocks {
        block.frame(&mut hash);
    }
    hash.finish_hash().to_hex()
}
fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn object(response: &mut surrealdb::IndexedResults) -> Result<Object, CanonicalError> {
    let index = response.num_statements().saturating_sub(2);
    response
        .take::<Option<Object>>(index)?
        .ok_or(CanonicalError::IncompleteResponse)
}

/// An exact nonexpiring artifact read; cloned decoded arrays retain their actual byte allowances.
#[derive(Clone, Debug)]
pub struct ArtifactRead {
    store: CanonicalStore,
    header: Artifact,
    selection: ProtectedSelection,
}
impl ArtifactRead {
    pub(crate) fn belongs_to(&self, store: &CanonicalStore) -> bool {
        self.store.db.same_driver(&store.db)
    }
    pub(crate) fn protected_selection(&self) -> &ProtectedSelection {
        &self.selection
    }
    /// Original protected acquisition identity for explicit uncertainty attribution.
    pub fn read_key(&self) -> &str {
        self.selection.key()
    }
    /// Exact immutable occurrence metadata.
    pub fn header(&self) -> &Artifact {
        &self.header
    }
    /// Explicit release is idempotent; the original grant remains fenced through its issue window.
    pub async fn release(self) -> Result<(), CanonicalError> {
        self.selection.finish().await
    }
    /// Read one bounded member under the exact artifact acquisition.
    pub async fn block(&self, ordinal: u64) -> Result<ArtifactBlock, CanonicalError> {
        let activity = self.store.admit_activity()?;
        activity.scope(async {
            let value=codec::encode_uint(ordinal)?;
            let mut response=protected_query("canonical_artifacts::read",||Ok(self.store.db.query("BEGIN; RETURN fn::pse_artifact_v1::read($pse_rpc_expires_at,$key,$protection,$ordinal); COMMIT;")
                .bind(("key",self.header.key.clone())).bind(("protection",self.selection.key().to_owned())).bind(("ordinal",value.clone())))).await?;
            let row=wire::decode_canonical_artifact_blocks(object(&mut response)?)?;
            if row.artifact!=self.header.key || row.ordinal!=ordinal || row.payload_bytes>wire::RESULT_BLOCK_BYTES as u64 { return Err(invalid("artifact block coordinate differs")); }
            let descriptor=self.store.descriptor_for_instance(&row.file_instance).await?;
            if descriptor.digest!=row.file_digest||descriptor.bytes!=row.payload_bytes||descriptor.encoding!=row.encoding||descriptor.schema!=row.schema||row.file_batch_index!=0{return Err(invalid("artifact sealed file binding differs"));}
            use std::io::Read;
            let mut file=self.store.open_physical_file(descriptor,std::sync::Arc::new(self.clone()))?;
            let mut bytes=Vec::with_capacity(row.payload_bytes as usize);file.read_to_end(&mut bytes).map_err(|error|invalid(&error.to_string()))?;
            let mut copied=artifact_block(&row.artifact,row.ordinal,&row.relation,bytes,&row.original_digest,row.rows);
            if row.encoding==crate::canonical_files::ARROW_IPC_FILE {copied=copied.native_file(&row.schema)?;}
            if copied.key!=row.key || copied.digest!=row.digest { return Err(invalid("artifact copied encoding digest differs")); }
            copied.allocation=crate::canonical::transport::current_response_owner();Ok(copied)
        }).await
    }
}

impl CanonicalStore {
    /// Abandon one exact pre-header study staging identity. The permanent fence
    /// is published before bounded cleanup and rejects every late staging call.
    /// Existing study headers require the ordinary study retirement owner.
    pub async fn abandon_study_inputs(&self, study: &str) -> Result<bool, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            if study.is_empty() || study.len()>4096 { return Err(CanonicalError::PayloadLimit); }
            let mut response=protected_query("canonical_artifacts::abandon_study_inputs",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::abandon_study_inputs($pse_rpc_expires_at,$study); COMMIT;").bind(("study",study.to_owned())))).await?;
            let index=response.num_statements().saturating_sub(2);
            response.take::<Option<bool>>(index)?.ok_or(CanonicalError::IncompleteResponse)
        }).await
    }
    /// Read-only issue of a finite original intent. Seal it once and retain it through uncertainty.
    pub async fn new_artifact(
        &self,
        revision: &Revision,
        kind: &str,
        provenance: Vec<u8>,
        block_count: u64,
        payload_bytes: u64,
    ) -> Result<Artifact, CanonicalError> {
        let activity = self.admit_activity()?;
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
    pub async fn persist_artifact(
        &self,
        header: &Artifact,
        sources: &[Revision],
        inputs: &[ResultRead],
        blocks: &[ArtifactBlock],
        owner_kind: &str,
        owner: &str,
    ) -> Result<Artifact, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            if header.creation_request_digest!=request_digest(header,sources,inputs,blocks,owner_kind,owner) { return Err(CanonicalError::OperationReused); }
            let result=async {
                self.ensure_writes()?;
                if header.active || header.retiring || header.interpretation!=wire::INTERPRETATION || header.block_count!=blocks.len() as u64 || sources.is_empty() || sources.len()>65 || inputs.len()>64 || inputs.iter().any(|input|!input.belongs_to(self)) { return Err(invalid("artifact exact closure differs")); }
                let total=blocks.iter().enumerate().try_fold(0u64,|total,(ordinal,block)|{
                    let mut copied=artifact_block(&header.key,ordinal as u64,&block.relation,block.payload.to_vec(),&block.original_digest,block.rows);
                    if block.encoding==crate::canonical_files::ARROW_IPC_FILE {copied=copied.native_file(&block.schema)?;} else if block.payload.starts_with(b"ARROW1") {return Err(invalid("native artifact file requires explicit schema binding"));}
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
                    let intent=self.reserve_artifact_file(&header.key,&format!("artifact-file:{}",block.key),&header.creation_request_digest).await?;
                    let sealed=self.seal_file(&intent,block.payload.as_slice(),&block.encoding,block.batch_count,&block.schema).await?;
                    let descriptor=sealed.descriptor();
                    let stored=pse_model::generated::runtime::canonical_artifact_blocks::Row{key:block.key.clone(),artifact:block.artifact.clone(),ordinal:block.ordinal,relation:block.relation.clone(),file_instance:descriptor.instance.clone(),file_digest:descriptor.digest.clone(),file_batch_index:0,payload_bytes:descriptor.bytes,schema:descriptor.schema.clone(),encoding:descriptor.encoding.clone(),digest:block.digest.clone(),original_digest:block.original_digest.clone(),rows:block.rows};
                    let row=wire::encode_canonical_artifact_blocks(&stored)?;
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
    pub async fn artifact(&self, key: &str) -> Result<Artifact, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            let mut response=protected_query("canonical_artifacts::artifact",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::available($pse_rpc_expires_at,$key); COMMIT;").bind(("key",key.to_owned())))).await?;
            Ok(wire::decode_canonical_artifacts(object(&mut response)?)?)
        }).await
    }
    /// Acquire under independent artifact authority; producer payload is never consulted.
    pub async fn read_artifact(
        &self,
        key: &str,
    ) -> Result<ArtifactRead, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            let header=self.artifact(key).await?;
            let revision=self.revision(&header.revision).await?.ok_or_else(||invalid("artifact source absent"))?;
            let selection=self.protect(revision,crate::canonical::REQUEST_TIMEOUT).await?;
            let mut response=protected_query("canonical_artifacts::acquire",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::acquire($pse_rpc_expires_at,$key,$protection); COMMIT;").bind(("key",key.to_owned())).bind(("protection",selection.key().to_owned())))).await?;
            let actual=wire::decode_canonical_artifacts(object(&mut response)?)?;
            if actual!=header { return Err(invalid("artifact changed during acquisition")); }
            self.admit_manifest_ticket("artifact",key,None,selection.key()).await?;
            Ok(ArtifactRead { store:self.clone(),header,selection })
        }).await
    }
    /// Attach a named independent obligation to an active occurrence.
    pub async fn retain_artifact(
        &self,
        key: &str,
        owner_kind: &str,
        owner: &str,
    ) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let row=wire::encode_canonical_artifact_obligations(&artifact_obligation(key,owner_kind,owner))?;
            protected_query("canonical_artifacts::retain",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::attach($pse_rpc_expires_at,$key,$obligation,true); COMMIT;").bind(("key",key.to_owned())).bind(("obligation",row.clone())))).await?; Ok(())
        }).await
    }
    /// Release exactly one owner, preserving all independent owners and read grants.
    pub async fn release_artifact(
        &self,
        key: &str,
        owner_kind: &str,
        owner: &str,
    ) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let row=wire::encode_canonical_artifact_obligations(&artifact_obligation(key,owner_kind,owner))?;
            protected_query("canonical_artifacts::release",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::release($pse_rpc_expires_at,$obligation); COMMIT;").bind(("obligation",row.clone())))).await?; Ok(())
        }).await
    }
    /// Release only the named owner after proving the unchanged original intent.
    pub async fn release_artifact_intent(
        &self,
        intent: &Artifact,
        owner_kind: &str,
        owner: &str,
    ) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let obligation=wire::encode_canonical_artifact_obligations(&artifact_obligation(&intent.key,owner_kind,owner))?;
            let intent=wire::encode_canonical_artifacts(intent)?;
            protected_query("canonical_artifacts::release_intent",||Ok(self.db.query("BEGIN; fn::pse_artifact_v1::original($pse_rpc_expires_at,$intent); RETURN fn::pse_artifact_v1::release($pse_rpc_expires_at,$obligation); COMMIT;").bind(("intent",intent.clone())).bind(("obligation",obligation.clone())))).await?;
            Ok(())
        }).await
    }
    /// Retire or settle the exact original intent, reclaiming one bounded page per call.
    pub async fn collect_artifact(&self, intent: &Artifact) -> Result<bool, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            self.ensure_writes()?;
            let row=wire::encode_canonical_artifacts(intent)?;
            let mut response=protected_query("canonical_artifacts::collect",||Ok(self.db.query("BEGIN; RETURN fn::pse_artifact_v1::collect($pse_rpc_expires_at,$intent); COMMIT;").bind(("intent",row.clone())))).await?;
            Ok(codec::decode_boolean(codec::required(&mut object(&mut response)?,"complete")?)?)
        }).await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_artifact_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "isolated owned artifact fixtures require valid native setup and assertions"
    )]
    use super::*;
    async fn fixture() -> (CanonicalStore, Revision) {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit native fixture required");
        let mut options =
            crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_artifact_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let revision = store
            .edit("artifact-source", None, "original-source", &[])
            .await
            .unwrap();
        (store, revision)
    }
    async fn content(
        store: &CanonicalStore,
        revision: &Revision,
        owner_kind: &str,
        owner: &str,
    ) -> (Artifact, Vec<ArtifactBlock>) {
        let bytes = vec![0, 255, 1, 128];
        let mut intent = store
            .new_artifact(
                revision,
                "test-exact-capsule",
                vec![7],
                1,
                bytes.len() as u64,
            )
            .await
            .unwrap();
        // Test controls issue a shorter original window before sealing/effects.
        intent.creation_expires_at -= 59_500_000;
        intent.key = format!(
            "pse.artifact.v1:{}:{}:{}",
            intent.primary_authority, intent.creation_nonce, intent.creation_expires_at
        );
        let blocks = vec![artifact_block(
            &intent.key,
            0,
            "capsule",
            bytes,
            "original-encoding",
            1,
        )];
        seal_artifact_request(
            &mut intent,
            std::slice::from_ref(revision),
            &[],
            &blocks,
            owner_kind,
            owner,
        );
        (intent, blocks)
    }
    #[tokio::test]
    async fn canonical_artifact_independent_owners_and_exact_copied_read() {
        let (store, revision) = fixture().await;
        let (intent, blocks) = content(&store, &revision, "user", "alice").await;
        let saved = store
            .persist_artifact(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "user",
                "alice",
            )
            .await
            .unwrap();
        store
            .retain_artifact(&saved.key, "audit", "audit-1")
            .await
            .unwrap();
        store
            .release_artifact(&saved.key, "user", "alice")
            .await
            .unwrap();
        assert!(
            store.collect_artifact(&intent).await.is_err(),
            "another attributed obligation prevents collection"
        );
        let read = store
            .read_artifact(&saved.key)
            .await
            .unwrap();
        let copied = read.block(0).await.unwrap();
        assert_eq!(copied.payload.as_slice(), [0, 255, 1, 128]);
        assert_eq!(copied.original_digest, "original-encoding");
        assert_eq!(
            copied.digest,
            crate::canonical_execution::result_payload_digest(&copied.payload)
        );
        read.release().await.unwrap();
        drop(read);
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_owner_release_fences_both_acquisition_orders() {
        let (store, revision) = fixture().await;
        let (intent, blocks) = content(&store, &revision, "audit", "independent").await;
        store
            .persist_artifact(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "audit",
                "independent",
            )
            .await
            .unwrap();
        store
            .retain_artifact(&intent.key, "user", "retain-first")
            .await
            .unwrap();
        store
            .release_artifact(&intent.key, "user", "retain-first")
            .await
            .unwrap();
        assert!(
            store
                .retain_artifact(&intent.key, "user", "retain-first")
                .await
                .is_err()
        );
        store
            .release_artifact(&intent.key, "project", "release-first")
            .await
            .unwrap();
        store
            .release_artifact(&intent.key, "project", "release-first")
            .await
            .unwrap();
        assert!(
            store
                .retain_artifact(&intent.key, "project", "release-first")
                .await
                .is_err()
        );
        store
            .retain_artifact(&intent.key, "user", "new-independent-lifetime")
            .await
            .unwrap();
        assert!(store.collect_artifact(&intent).await.is_err());
        assert!(
            store
                .read_artifact(&intent.key)
                .await
                .is_ok(),
            "other owners remain available"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_live_read_prevents_collection_after_owner_release() {
        let (store, revision) = fixture().await;
        let (intent, blocks) = content(&store, &revision, "user", "reader").await;
        store
            .persist_artifact(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "user",
                "reader",
            )
            .await
            .unwrap();
        let read = store
            .read_artifact(&intent.key)
            .await
            .unwrap();
        let copied = read.block(0).await.unwrap();
        store
            .release_artifact(&intent.key, "user", "reader")
            .await
            .unwrap();
        assert!(
            !store.collect_artifact(&intent).await.unwrap(),
            "a live exact read keeps its immutable content available"
        );
        assert_eq!(read.block(0).await.unwrap().payload, copied.payload);
        read.release().await.unwrap();
        // Original acquisition authority still has its finite issue window;
        // release does not let collection invent proof that a delayed read cannot commit.
        assert!(!store.collect_artifact(&intent).await.unwrap());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_bounded_collection_fences_late_original_creation() {
        let (store, revision) = fixture().await;
        let (intent, blocks) = content(&store, &revision, "project", "p").await;
        store
            .persist_artifact(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "project",
                "p",
            )
            .await
            .unwrap();
        store
            .release_artifact(&intent.key, "project", "p")
            .await
            .unwrap();
        assert!(!store.collect_artifact(&intent).await.unwrap());
        assert!(
            store.artifact(&intent.key).await.is_err(),
            "retirement closes fresh acquisition"
        );
        tokio::time::sleep(Duration::from_millis(550)).await;
        let mut complete = false;
        for _ in 0..8 {
            if store.collect_artifact(&intent).await.unwrap() {
                complete = true;
                break;
            }
        }
        assert!(complete);
        assert!(
            store
                .persist_artifact(
                    &intent,
                    std::slice::from_ref(&revision),
                    &[],
                    &blocks,
                    "project",
                    "p"
                )
                .await
                .is_err(),
            "the original finite intent cannot recreate a collected occurrence"
        );
        assert!(
            store.collect_artifact(&intent).await.unwrap(),
            "same original settlement is idempotent after collection"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_exact_seal_refuses_changed_content_before_effects() {
        let (store, revision) = fixture().await;
        let (intent, mut blocks) = content(&store, &revision, "user", "alice").await;
        assert!(matches!(
            store
                .persist_artifact(
                    &intent,
                    std::slice::from_ref(&revision),
                    &[],
                    &blocks,
                    "user",
                    "changed-owner"
                )
                .await,
            Err(CanonicalError::OperationReused)
        ));
        blocks[0].payload = vec![9].into();
        assert!(matches!(
            store
                .persist_artifact(
                    &intent,
                    std::slice::from_ref(&revision),
                    &[],
                    &blocks,
                    "user",
                    "alice"
                )
                .await,
            Err(CanonicalError::OperationReused)
        ));
        assert!(store.artifact(&intent.key).await.is_err());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_original_intent_settles_inactive_staging_and_refuses_alteration() {
        let (store, revision) = fixture().await;
        let (intent, _blocks) = content(&store, &revision, "audit", "staging-owner").await;
        let mut root = Object::new();
        root.insert("key", format!("test-staged-root:{}", intent.key));
        root.insert("problem", revision.problem.clone());
        root.insert("revision", revision.key.clone());
        root.insert("sequence", codec::encode_uint(revision.sequence).unwrap());
        let row = wire::encode_canonical_artifacts(&intent).unwrap();
        protected_query("test:artifact-staging-only",||Ok(store.db.query("BEGIN; RETURN fn::pse_artifact_v1::begin($pse_rpc_expires_at,$row,$sources,[]); COMMIT;").bind(("row",row.clone())).bind(("sources",vec![root.clone()])))).await.unwrap();
        assert!(
            store.artifact(&intent.key).await.is_err(),
            "staged original intent grants no active discovery"
        );
        assert!(
            store
                .read_artifact(&intent.key)
                .await
                .is_err(),
            "staged original intent grants no reader"
        );
        let mut changed = intent.clone();
        changed.creation_request_digest = "f".repeat(64);
        assert!(
            store
                .release_artifact_intent(&changed, "audit", "staging-owner")
                .await
                .is_err(),
            "release proves complete immutable original equality"
        );
        assert!(
            store.collect_artifact(&changed).await.is_err(),
            "collection cannot substitute changed original metadata"
        );
        store
            .release_artifact_intent(&intent, "audit", "staging-owner")
            .await
            .unwrap();
        store
            .release_artifact_intent(&intent, "audit", "staging-owner")
            .await
            .unwrap();
        assert!(
            !store.collect_artifact(&intent).await.unwrap(),
            "original creation window remains pending"
        );
        tokio::time::sleep(Duration::from_millis(550)).await;
        let mut complete = false;
        for _ in 0..8 {
            if store.collect_artifact(&intent).await.unwrap() {
                complete = true;
                break;
            }
        }
        assert!(complete);
        assert!(
            store.collect_artifact(&intent).await.unwrap(),
            "same original intent settles lost ACK after deletion"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_artifact_preheader_study_abandonment_fences_both_orders() {
        let (store, revision) = fixture().await;
        let (mut intent, blocks) = content(&store, &revision, "user", "independent-owner").await;
        intent.kind = "seed".into();
        seal_artifact_request(
            &mut intent,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "user",
            "independent-owner",
        );
        store
            .persist_artifact(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "user",
                "independent-owner",
            )
            .await
            .unwrap();
        store
            .retain_study_input("staging-first", "solution", &intent.key)
            .await
            .unwrap();
        assert_eq!(
            store
                .study_input("staging-first", "solution")
                .await
                .unwrap(),
            intent.key
        );
        assert!(store.abandon_study_inputs("staging-first").await.unwrap());
        assert!(
            store.abandon_study_inputs("staging-first").await.unwrap(),
            "same abandonment safely settles a lost ACK"
        );
        assert!(
            store
                .study_input("staging-first", "solution")
                .await
                .is_err()
        );
        assert!(
            store
                .retain_study_input("staging-first", "solution", &intent.key)
                .await
                .is_err(),
            "late original staging cannot recreate an abandoned identity"
        );
        assert!(
            store
                .abandon_study_inputs("abandonment-first")
                .await
                .unwrap()
        );
        assert!(
            store
                .retain_study_input("abandonment-first", "solution", &intent.key)
                .await
                .is_err()
        );
        assert!(
            store
                .retain_artifact(&intent.key, "study", "abandonment-first")
                .await
                .is_err()
        );
        assert!(
            store.artifact(&intent.key).await.unwrap().active,
            "abandoning either study does not release the independent user"
        );
        let mut response=bounded_query(store.db.query("SELECT key FROM canonical_artifact_obligations WHERE owner_kind='study'; SELECT key FROM canonical_study_inputs;")).await.unwrap();
        assert!(response.take::<Vec<Object>>(0).unwrap().is_empty());
        assert!(response.take::<Vec<Object>>(1).unwrap().is_empty());
        store.remove_isolated_fixture().await.unwrap();
    }
}
