// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded native study candidate scheduling over exact canonical sources.
use super::{Durability,Operations,Runtime,WorkflowError,contract,RunDurability,durable::{DurableAttempt,DurableRecord}};
use crate::authoring_driver::document::{OwnedDocumentSet,load_package_documents_owned,package_checksum};
use pse_ids::{ContentHash,SemanticId};
use pse_model::generated::{enums::AttemptState,identities::{RunId,AttemptId}};
use std::{collections::BTreeMap,sync::Arc,time::Duration};

/// Exact canonical physical source receipt, separate from compiler physical context identity.
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalSource {
    /// Exact immutable revision in its declared canonical source kind.
    pub revision:String,
    /// Source interpretation checksum, distinct from the compiler context identity.
    pub identity:ContentHash,
}
/// One bounded worker scheduling action; no native reports accumulate in the supervisor.
#[derive(Clone,Debug)]
pub enum Processed {
    /// No currently dispatchable or recoverable occurrence was found.
    Idle,
    /// One claimed occurrence produced its retained execution receipt.
    Ran {
        /// Exact canonical occurrence key.
        point:String,
        /// Retained completion or observable persistence failure.
        record:Box<DurableRecord>,
    },
    /// Shared policy settled one occurrence without native work.
    Settled {
        /// Exact canonical occurrence key.
        point:String,
    },
    /// Every occurrence settled and the effect-free parent summary was admitted.
    Finalized {
        /// Exact canonical study key.
        study:String,
    },
}
impl Processed {
    /// Actual retained attempt state; effect-free actions have no attempt.
    pub fn state(&self)->Option<AttemptState>{match self {Self::Ran{record,..}=>record.completion.as_ref().map(|c|c.state),_=>None}}}
/// Finite polling and optional total-work bound; live service retains only a count.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct WorkerSettings {
    /// Finite delay between discovery iterations.
    pub poll:Duration,
    /// Finite interval between bounded assigned-attempt recovery passes.
    pub recovery:Duration,
    /// Optional maximum number of processed actions.
    pub maximum_actions:Option<usize>,
    /// Finish when a complete scoped discovery pass has no work.
    pub until_idle:bool,
}
impl Default for WorkerSettings {fn default()->Self{Self{poll:Duration::from_millis(500),recovery:Duration::from_secs(30),maximum_actions:None,until_idle:false}}}
fn operations(runtime:&Runtime)->Result<&Operations,WorkflowError>{match &runtime.durability{Durability::Durable(operations)=>Ok(operations),Durability::Ephemeral=>Err(contract("canonical study worker requires durable runtime"))}}
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEntry {path:String,chunks:u64,bytes:u64,digest:String}
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PhysicalManifest {version:u8,identity:ContentHash,documents:Vec<SourceEntry>}
impl Operations {
    /// Store exact authored document bytes in bounded canonical objects.
    pub async fn put_sources(&self,sources:&BTreeMap<String,Vec<u8>>)->Result<PhysicalSource,WorkflowError>{
        let identity=package_checksum(sources);let problem=format!("physical:documents:{identity}");let mut parent=None;let mut documents=Vec::new();
        for (path,bytes) in sources {
            let chunks=bytes.len().div_ceil(pse_operations::canonical_execution::RESULT_BATCH_BYTES)as u64;
            let path_key=pse_ids::document::of(pse_ids::Frame::CanonicalPayloadV1,&("physical.document.path.v1",path)).map_err(|e|contract(e.to_string()))?;
            for (ordinal,payload) in bytes.chunks(pse_operations::canonical_execution::RESULT_BATCH_BYTES).enumerate(){
                let logical=format!("document:{path_key}:{ordinal:020}");let edit=super::durable::source_edit(logical.clone(),"physical:documents".into(),logical,"physical:documents:bytes:v1",payload.to_vec());
                let revision=self.store().edit(&problem,parent.as_deref(),&format!("physical:{identity}:{path_key}:{ordinal}"),&[edit]).await?;parent=Some(revision.key);
            }
            documents.push(SourceEntry{path:path.clone(),chunks,bytes:bytes.len()as u64,digest:pse_operations::canonical_execution::result_payload_digest(bytes)});
        }
        let payload=serde_json::to_vec(&PhysicalManifest{version:1,identity,documents}).map_err(|e|contract(e.to_string()))?;
        if payload.len()>128*1024{return Err(contract("physical document manifest exceeds metadata bound"));}
        let edit=super::durable::source_edit("manifest".into(),"physical:documents".into(),"manifest".into(),"physical:documents:manifest:v1",payload);
        let revision=self.store().edit(&problem,parent.as_deref(),&format!("physical:{identity}:manifest"),&[edit]).await?;
        Ok(PhysicalSource{revision:revision.key,identity})
    }
    /// Load only the selected protected document-kind revision, preserving original bytes.
    pub async fn sources(&self,source:&PhysicalSource)->Result<Arc<pse_columnar::Leased<BTreeMap<String,Vec<u8>>>>,WorkflowError>{
        let revision=self.store().revision(&source.revision).await?.ok_or_else(||contract("physical source revision absent"))?;
        if revision.problem!=format!("physical:documents:{}",source.identity){return Err(contract("physical document receipt interpretation differs"));}
        let protection=self.store().protect(revision,Duration::from_secs(3600)).await?;
        let mut read=pse_operations::canonical_selection::SelectedRead::new(protection);
        let result=async {
            let reservation=datafusion::execution::memory_pool::MemoryConsumer::new("canonical:physical-documents").register(&self.pool);
            reservation.try_grow(2*pse_operations::canonical_execution::RESULT_BATCH_BYTES).map_err(pse_engine::EngineError::from)?;
            let memberships=self.store().resolve_logicals(&mut read,&["manifest".into()]).await?;
            let member=memberships.first().ok_or_else(||contract("physical document manifest absent"))?;
            let object=self.store().selected_object(read.selection(),&member.version).await?.ok_or_else(||contract("physical document manifest object absent"))?;
            if object.kind!="physical:documents:manifest:v1"{return Err(contract("physical source kind differs"));}
            let manifest:PhysicalManifest=serde_json::from_slice(object.payload.as_slice()).map_err(|e|contract(e.to_string()))?;
            if manifest.version!=1||manifest.identity!=source.identity{return Err(contract("physical document manifest interpretation differs"));}
            let bytes=manifest.documents.iter().try_fold(0_usize,|sum,document|usize::try_from(document.bytes).ok().and_then(|bytes|sum.checked_add(bytes))).ok_or_else(||contract("physical source declared extent overflow"))?;
            reservation.try_grow(bytes.checked_mul(4).ok_or_else(||contract("physical source allocation extent overflow"))?).map_err(pse_engine::EngineError::from)?;
            let mut sources=BTreeMap::new();
            for document in manifest.documents {
                if document.chunks!=document.bytes.div_ceil(pse_operations::canonical_execution::RESULT_BATCH_BYTES as u64){return Err(contract("physical document coverage differs"));}
                let path_key=pse_ids::document::of(pse_ids::Frame::CanonicalPayloadV1,&("physical.document.path.v1",&document.path)).map_err(|e|contract(e.to_string()))?;
                let mut bytes=Vec::new();bytes.try_reserve_exact(usize::try_from(document.bytes).map_err(|_|contract("physical document extent"))?).map_err(|_|contract("physical document allocation refused"))?;
                for ordinal in 0..document.chunks {
                    let logical=format!("document:{path_key}:{ordinal:020}");let members=self.store().resolve_logicals(&mut read,&[logical]).await?;let member=members.first().ok_or_else(||contract("physical document chunk absent"))?;
                    let object=self.store().selected_object(read.selection(),&member.version).await?.ok_or_else(||contract("physical document chunk object absent"))?;
                    let length=(document.bytes-ordinal*pse_operations::canonical_execution::RESULT_BATCH_BYTES as u64).min(pse_operations::canonical_execution::RESULT_BATCH_BYTES as u64);
                    if object.kind!="physical:documents:bytes:v1"||object.payload.len()as u64!=length{return Err(contract("physical document chunk kind/extent differs"));}bytes.extend_from_slice(object.payload.as_slice());
                }
                if pse_operations::canonical_execution::result_payload_digest(&bytes)!=document.digest||sources.insert(document.path,bytes).is_some(){return Err(contract("physical document digest/path differs"));}
            }
            if package_checksum(&sources)!=source.identity{return Err(contract("physical document package checksum differs"));}Ok::<_,WorkflowError>(Arc::new(pse_columnar::Leased::new(Arc::new(sources),pse_columnar::AllocationLease::new(reservation))))
        }.await;
        self.store().release(read.selection()).await?;result
    }
}

impl Runtime {
    /// Claim a structurally ready candidate under its exact consumed revisions.
    pub async fn work_once(&self)->Result<Processed,WorkflowError>{Ok(self.work_once_with_result().await?.0)}
    /// Return one drained native result, whose owner remains retained by this view.
    pub async fn work_once_with_result(&self)->Result<(Processed,Option<Arc<super::RunResult>>),WorkflowError>{self.work_once_scoped(true).await}
    async fn work_once_scoped(&self,recovery:bool)->Result<(Processed,Option<Arc<super::RunResult>>),WorkflowError>{
        use pse_model::study::{ActionKind,SeedFact,SeedAvailability,StartPolicy,WaitReason};
        let operations=operations(self)?;
        match operations.store().check_write_admission(){Ok(())=>{},Err(pse_operations::canonical::CanonicalError::Quiesced)=>return Ok((Processed::Idle,None)),Err(error)=>return Err(error.into())}
        let mut after=None;
        loop {
            let studies=operations.store().study_page(after.as_deref()).await?;
            if studies.is_empty(){return Ok((Processed::Idle,None));}
            for study in studies {
                after=Some(study.key.clone());
                let mut assigned_after=None;
                while recovery {
                    let assigned=operations.store().study_assigned_page(&study.key,assigned_after).await?;
                    if assigned.is_empty(){break;}
                    for point in assigned {
                        assigned_after=Some(point.ordinal);
                        if self.recover_study_point(&point.key).await?{return Ok((Processed::Settled{point:point.key},None));}
                    }
                }
                let mut point_after=None;
                loop {
                    let page=operations.store().study_candidates(&study.key,point_after).await?;
                    if page.is_empty(){break;}
                    for candidate in page {
                        point_after=Some(candidate.ordinal);
                        let scope=operations.store().study_scope(&candidate.key).await?;
                        let policy=scope.point().policy()?;
                        let unresolved=match &policy.start{StartPolicy::Fresh=>None,StartPolicy::Explicit{role,..}=>Some(SeedFact{role:*role,availability:SeedAvailability::Unresolved}),StartPolicy::Continuation(edge)=>Some(SeedFact{role:edge.role,availability:SeedAvailability::Unresolved})};
                        let preliminary=scope.action(unresolved)?;
                        match &preliminary.kind {
                            ActionKind::Wait(WaitReason::SeedResolution{..})|ActionKind::Start(_)=>{},
                            ActionKind::Refuse(_)|ActionKind::Cancel=>{self.settle_study_candidate(&scope,&preliminary,None).await?;return Ok((Processed::Settled{point:candidate.key},None));},
                            ActionKind::Wait(_)|ActionKind::Reconcile=>continue,
                        }
                        let cancel=crate::CancelSource::new();
                        let mut prepared=self.prepare_study_candidate(&scope,&cancel).await?;
                        let action=scope.action(prepared.seed.clone())?;
                        if !matches!(action.kind,ActionKind::Start(_)){match action.kind{ActionKind::Refuse(_)|ActionKind::Cancel=>{self.settle_study_candidate(&scope,&action,None).await?;return Ok((Processed::Settled{point:candidate.key},None));},_=>continue}}
                        let claim_operation=format!("study-claim:{}:{}:{}",candidate.key,scope.point().revision,operations.worker());
                        let claim=match operations.store().claim_study_point(&scope,prepared.seed.clone(),&claim_operation,operations.worker(),operations.policy().lease).await {
                            Ok(claim)=>claim,
                            Err(error)=>{let current=operations.store().study_scope(&candidate.key).await?;if current.point().revision!=scope.point().revision||current.cancelled()!=scope.cancelled()||current.predecessors().iter().map(|p|(&p.key,p.revision)).ne(scope.predecessors().iter().map(|p|(&p.key,p.revision))){continue;}return Err(error.into());}
                        };
                        let run_key=claim.fence.run().strip_prefix("run:").ok_or_else(||contract("scientific run lineage key absent"))?;
                        let run=RunId::from_id(SemanticId::parse_hex(run_key).map_err(|e|contract(e.to_string()))?);
                        let mut hash=pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);hash.str("scientific.attempt.lineage.v1").str(claim.fence.attempt());let attempt_id=AttemptId::from_id(hash.finish_id());
                        let mut attempt=DurableAttempt::claimed(operations,claim.fence.clone(),run,attempt_id);
                        let stop=cancel.clone();attempt.start(Arc::new(move ||stop.cancel())).await?;
                        let point=match operations.store().mark_study_started(&claim).await{Ok(point)=>point,Err(error)=>return Ok((Processed::Ran{point:claim.point.key,record:Box::new(attempt.abandon(&Arc::new(error.into())).await)},None))};
                        if let Err(error)=self.apply_study_prediction(&mut prepared,&cancel).await {
                            let record=attempt.abandon(&Arc::new(error)).await;self.recover_study_point(&point.key).await?;
                            return Ok((Processed::Ran{point:point.key,record:Box::new(record)},None));
                        }
                        operations.store().renew_attempt(&claim.fence,operations.policy().lease).await?;
                        let handle=prepared.operation.start_attempt(self,&cancel,attempt).await?;
                        let result=tokio::select!{result=handle.wait()=>result?,()=cancel.cancelled()=>{handle.cancel();handle.wait().await?}};
                        let record=match result.durability(){RunDurability::Durable(record)=>record.clone(),RunDurability::Ephemeral=>return Err(contract("claimed native study attempt was not retained"))};
                        self.record_study_attempt(&point,&claim.start,&result,&record).await?;
                        return Ok((Processed::Ran{point:point.key,record},Some(result)));
                    }
                }
                if self.finalize_canonical_study(&study.key).await? {return Ok((Processed::Finalized{study:study.key},None));}
            }
        }
    }
    /// Serve bounded candidate pages until stopped; completed native owners are released
    /// between iterations and only the processed count remains live.
    pub async fn serve(&self,settings:WorkerSettings,stop:&crate::CancelSource)->Result<usize,WorkflowError>{
        if settings.poll.is_zero()||settings.recovery.is_zero(){return Err(contract("worker polling and recovery intervals must be positive"));}
        let mut count=0_usize;let mut last_recovery=None;
        while !stop.token().is_cancelled()&&settings.maximum_actions.is_none_or(|limit|count<limit){let recovery=last_recovery.is_none_or(|last:tokio::time::Instant|last.elapsed()>=settings.recovery);if recovery{last_recovery=Some(tokio::time::Instant::now());}match self.work_once_scoped(recovery).await?.0{Processed::Idle if settings.until_idle=>break,Processed::Idle=>tokio::select!{_=tokio::time::sleep(settings.poll)=>{},()=stop.cancelled()=>{}},_=>{count=count.checked_add(1).ok_or_else(||contract("worker processed count overflow"))?;}}}
        Ok(count)
    }
    /// Shared document physical admission stays with its scientific owner.
    pub(crate) async fn physical_from_sources(&self,sources:&BTreeMap<String,Vec<u8>>,cancel:&crate::CancelSource)->Result<super::PhysicalContext,WorkflowError>{
        let validation=self.sessions.validation_context(&self.registry)?;
        let pool=self.shared.pool();let token=cancel.token();
        let bundle=load_package_documents_owned(sources,&self.registry,pse_authoring::ParseBudget::default(),&pool,&token,&validation)?;
        let documents=OwnedDocumentSet::try_from_bundles(vec![bundle],&pool,&token)?;
        self.physical_from_documents(&documents,&token).await
    }
    pub(super) async fn package_from_sources(
        &self,
        bundles: &[BTreeMap<String, Vec<u8>>],
        physical: super::PhysicalContext,
    ) -> Result<super::ModelingPackage, WorkflowError> {
        // Source bytes and load order define the complete immutable admission closure.
        // The cache owns only admitted values; every attempt has its own mutable workspace.
        let service = self.shared.math();
        let generation = service.modeling_cache.generation();
        let mut framed = pse_ids::FramedHasher::new(pse_ids::Frame::ModelingPackageAdmissionV1);
        framed.u64(bundles.len() as u64).hash(&physical.key)
            // Local cache authority is the exact immutable validation assembly and registry.
            // Its owner is retained with the admission, preventing pointer reuse. These
            // process-local slots never enter source/scientific/operational identities.
            .u64(Arc::as_ptr(&self.sessions) as usize as u64)
            .u64(Arc::as_ptr(&self.registry) as usize as u64);
        for sources in bundles {
            framed.hash(&package_checksum(sources));
        }
        match &physical.package {
            Some(package) => {
                use pse_model::SemanticFrame;
                framed.u64(1);
                package.header.frame(&mut framed);
            }
            None => {
                framed.u64(0);
            }
        }
        let identity = pse_ids::roles::AdmittedClosureHash::from_id(framed.finish_hash());
        if let Some(admitted) = service.modeling_cache.package(identity) {
            return self.package_from_admission(admitted, physical);
        }
        let validation = self.sessions.validation_context(&self.registry)?;
        let pool = self.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let bundles = bundles
            .iter()
            .map(|sources| {
                load_package_documents_owned(
                    sources,
                    &self.registry,
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &token,
                    &validation,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
        let package = self.modeling_from_documents(&documents, physical).await?;
        service
            .modeling_cache
            .retain_package(generation, identity, package.admission()?);
        Ok(package)
    }
}
