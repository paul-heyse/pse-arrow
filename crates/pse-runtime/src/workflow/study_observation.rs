// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive bounded study telemetry: exact per-attempt streams, never a study execution owner.
use super::{Runtime,WorkflowError,contract,StudyCohortRequest,StudyWorkbenchFilter,StudyCohortDocument,ProgressEventDocument,ProgressCheckpointDocument};
use pse_columnar::CancellationToken;
use std::{collections::BTreeMap,sync::Arc,time::Duration};
/// Selected study and bounded observation allowance. These controls never govern science.
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationRequest {
    /// Exact existing study key.
    pub study:String,
    /// Maximum complete selected point population, from one through 64.
    pub maximum_members:usize,
    /// Maximum events returned per poll, from one through 512.
    pub maximum_events:usize,
}
/// Original immutable selection binds a cursor independently from mutable metadata revisions.
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationContext {
    /// Exact parent execution.
    pub run:String,
    /// Exact original source revision.
    pub source:String,
    /// Exact original physical source.
    pub physical_revision:Option<String>,
    /// Original physical interpretation.
    pub physical_identity:Option<String>,
    /// Original frozen engineering context.
    pub context:Option<String>,
}
/// Position in one exact attempt, with separate native source and durable batch clocks.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationPosition {
    /// Exact original execution.
    pub run:String,
    /// Exact original attempt; retries create another position.
    pub attempt:String,
    /// Actual canonical attempt generation.
    pub attempt_generation:u64,
    /// Actual point metadata revision, absent for parent summary.
    pub metadata_revision:Option<u64>,
    /// Actual numerical source generation, unavailable for remote live observations.
    pub source_generation:Option<u64>,
    /// Last actually observed source sequence; never a timestamp.
    pub source_sequence:u64,
    /// Next immutable terminal progress batch.
    pub batch:u64,
    /// Next event inside that batch.
    pub offset:usize,
    /// Highest consumed durable append sequence, distinct from source sequence.
    pub durable_sequence:Option<i64>,
    /// Terminal history was exhausted under its original reader.
    pub history_complete:bool,
}
/// Portable reconnect coordinates; decoding grants no result or cancellation authority.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationCursor {
    /// Exact canonical issuer.
    pub issuing_store:String,
    /// Exact original observation selection and limits.
    pub request:StudyObservationRequest,
    /// Exact scientific and engineering context.
    pub context:StudyObservationContext,
    /// Original parent and retry sources; older attempts remain present.
    pub sources:Vec<StudyObservationPosition>,
}
/// Explicit continuity statement for one selected source in this poll.
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum StudyObservationContinuity {
    /// Actual contiguous available source history after the captured position.
    CatchUp,
    /// Prefix or intervening samples are unavailable; current state is a resynchronization.
    GapCurrentResync,
    /// No local numeric telemetry owner exists for this ongoing remote attempt.
    LiveNumericsUnavailable,
    /// Selected attempt has no retained numerical history.
    HistoryUnavailable,
}
/// A source-local page; ordering across sources has no global causal interpretation.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservedSource {
    /// Exact original run/attempt position after this page.
    pub position:StudyObservationPosition,
    /// Actual source continuity, independent from scientific outcome.
    pub continuity:StudyObservationContinuity,
    /// Original whole numerical observations in their original source order.
    pub events:Vec<ProgressEventDocument>,
    /// Current live checkpoint when the local original owner is attached.
    pub checkpoint:Option<ProgressCheckpointDocument>,
    /// Original terminal writer coverage when available.
    pub coverage:Option<super::ProgressCoverage>,
}
/// One bounded read-only poll and its exact reconnect position.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyObservationPage {
    /// Canonical compact status with actual point metadata revisions.
    pub status:StudyCohortDocument,
    /// Bounded selected owner observations.
    pub sources:Vec<StudyObservedSource>,
    /// Exact next reconnect position.
    pub cursor:StudyObservationCursor,
}
/// Observer resources contain no study handle, worker, cancellation lease or dispatch route.
#[derive(Debug)]
pub struct StudyObserver {
    runtime:Runtime,
    cursor:StudyObservationCursor,
    live:BTreeMap<(String,String),Arc<pse_backend_native::solve::Progress>>,
    cancel:CancellationToken,
    closed:bool,
}
fn context(document:&StudyCohortDocument)->Result<StudyObservationContext,WorkflowError> {
    let h=document.headers.first().filter(|_|document.headers.len()==1).ok_or_else(||contract("selected observation study absent"))?;
    Ok(StudyObservationContext{run:h.run.clone(),source:h.source.clone(),physical_revision:h.physical_revision.clone(),physical_identity:h.physical_identity.clone(),context:h.context.clone()})
}
impl Runtime {
    /// Open or reconnect selected existing owners. This performs no coordination or science.
    pub async fn observe_study(&self,request:StudyObservationRequest,cursor:Option<StudyObservationCursor>)->Result<StudyObserver,WorkflowError> {
        if !(1..=64).contains(&request.maximum_members)||!(1..=512).contains(&request.maximum_events) {return Err(contract("observer capture allowance outside bounds"));}
        let status=self.freeze_study_cohort(StudyCohortRequest{filter:StudyWorkbenchFilter{study:Some(request.study.clone()),..Default::default()},maximum_studies:1,maximum_members:request.maximum_members}).await?;
        let selected=context(status.document())?;
        let issuer=self.canonical_store()?.issuing_store()?.to_owned();
        let cursor=match cursor {Some(cursor)=>{
            if cursor.issuing_store!=issuer||cursor.request!=request||cursor.context!=selected {return Err(contract("observer cursor issuer or exact selection differs"));}
            if cursor.sources.len()>64 {return Err(contract("observer cursor source bound"));}
            let mut keys=std::collections::BTreeSet::new();
            for source in &cursor.sources {
                if !keys.insert((&source.run,&source.attempt)) || source.offset>512 || source.run!=selected.run && !status.document().members.iter().any(|p|p.run==source.run) {return Err(contract("observer cursor includes foreign or malformed source"));}
                let attempt=self.canonical_store()?.canonical_attempt(&source.attempt).await?.ok_or_else(||contract("observer original attempt unavailable"))?;
                if attempt.run!=source.run||attempt.generation!=source.attempt_generation {return Err(contract("observer cursor attempt attribution differs"));}
            }
            cursor
        },None=>StudyObservationCursor{issuing_store:issuer,request,context:selected,sources:Vec::new()}};
        Ok(StudyObserver{runtime:self.clone(),cursor,live:BTreeMap::new(),cancel:CancellationToken::new(),closed:false})
    }
}
impl StudyObserver {
    /// Cancellation scope belongs only to observation, never to scientific execution.
    pub fn observation_token(&self)->CancellationToken{self.cancel.clone()}
    /// Capture portable coordinates without affecting execution or observation resources.
    pub fn cursor(&self)->StudyObservationCursor {self.cursor.clone()}
    /// Attach the original local telemetry owner without retaining its cancellation lease.
    pub fn attach_run(&mut self,handle:&super::RunHandle)->Result<(),WorkflowError> {
        if self.closed {return Err(contract("study observer closed"));}
        let (run,attempt,progress)=handle.study_progress_source().ok_or_else(||contract("study observer requires a canonical local source"))?;
        if !self.cursor.sources.iter().any(|s|s.run==run&&s.attempt==attempt) {return Err(contract("local telemetry source is outside current selected study; poll to discover its attempt"));}
        self.live.insert((run,attempt),progress); Ok(())
    }
    /// Release observation only. No study cancellation, draining, dispatch or coordination occurs.
    pub fn close(&mut self) {self.closed=true;self.cancel.cancel();self.live.clear();}
    /// Wait briefly for another poll; cancellation applies only to this observation future.
    pub async fn follow(&mut self,interval:Duration)->Result<StudyObservationPage,WorkflowError> {
        if interval>Duration::from_secs(60) {return Err(contract("observer follow interval exceeds bound"));}
        self.cancel.until_cancelled(tokio::time::sleep(interval)).await.map_err(pse_engine::EngineError::from)?;
        self.poll().await
    }
    /// Poll only this complete selected bounded study and its existing original attempt owners.
    pub async fn poll(&mut self)->Result<StudyObservationPage,WorkflowError> {
        let activity=self.runtime.admit_activity()?;
        activity.scope(self.poll_checked()).await
    }
    async fn poll_checked(&mut self)->Result<StudyObservationPage,WorkflowError> {
        if self.closed {return Err(contract("study observer closed"));}
        let status=self.runtime.freeze_study_cohort(StudyCohortRequest{filter:StudyWorkbenchFilter{study:Some(self.cursor.request.study.clone()),..Default::default()},maximum_studies:1,maximum_members:self.cursor.request.maximum_members}).await?;
        if context(status.document())?!=self.cursor.context {return Err(contract("observer selected context changed"));}
        let mut runs=vec![(self.cursor.context.run.clone(),None)];
        runs.extend(status.document().members.iter().map(|p|(p.run.clone(),Some(p.metadata_revision))));
        let mut attempts=Vec::new();
        for (run,revision) in runs {
            let Some(record)=self.runtime.canonical_store()?.canonical_run(&run).await? else {continue;};
            let page=self.runtime.canonical_store()?.canonical_attempt_page(&run,None,record.current_generation).await?;
            // Refuse an incomplete retry population rather than silently dropping earlier attempts.
            if page.len()==64 || attempts.len()+page.len()>64 {return Err(contract("observer attempt population exceeds bound; narrower observation required"));}
            for attempt in page {attempts.push((attempt,revision));}
        }
        for (attempt,revision) in &attempts {
            if let Some(position)=self.cursor.sources.iter_mut().find(|p|p.run==attempt.run&&p.attempt==attempt.key) {position.metadata_revision=*revision;}
            else {self.cursor.sources.push(StudyObservationPosition{run:attempt.run.clone(),attempt:attempt.key.clone(),attempt_generation:attempt.generation,metadata_revision:*revision,source_generation:None,source_sequence:0,batch:0,offset:0,durable_sequence:None,history_complete:false});}
        }
        let mut output=Vec::new(); let mut remaining=self.cursor.request.maximum_events;
        for (attempt,_) in attempts {
            let position=self.cursor.sources.iter_mut().find(|p|p.run==attempt.run&&p.attempt==attempt.key).ok_or_else(||contract("observed attempt position absent"))?;
            let mut events=Vec::new(); let mut checkpoint=None; let mut coverage=None;
            let mut continuity=StudyObservationContinuity::CatchUp;
            if attempt.terminal && !position.history_complete && remaining>0 {
                let mut history=self.runtime.progress(&position.run,&position.attempt,self.cancel.child_token()).await?;
                coverage=Some(history.coverage().clone()); history.resume_batch(position.batch)?;
                if let Some(page)=history.next_page().await? {
                    let length=page.len();
                    let begin=position.offset;
                    if begin>length {return Err(contract("observer event offset exceeds original batch"));}
                    let finish=length.min(begin+remaining);
                    for event in page.into_iter().skip(begin).take(finish-begin) {
                        position.durable_sequence=event.sequence;
                        if let Some(gap)=consume(position,&event) {if gap {continuity=StudyObservationContinuity::GapCurrentResync;} events.push(event);}
                    }
                    remaining-=finish-begin;
                    if finish==length {position.batch=history.batch_position();position.offset=0;}
                    else {position.offset=finish;}
                } else {position.history_complete=true;if matches!(history.coverage(),super::ProgressCoverage::Unknown{}) {continuity=StudyObservationContinuity::HistoryUnavailable;}}
                history.close();
            } else if !attempt.terminal {
                if let Some(progress)=self.live.get(&(position.run.clone(),position.attempt.clone())) {
                    let (page,dropped)=progress.snapshot();
                    let current=progress.current().map(ProgressEventDocument::from);
                    let (generation,head)=progress.source_identity();
                    let latest=current.as_ref().and_then(|e|e.source.as_ref()).is_some_and(|s|s.generation==generation&&s.sequence==head);
                    checkpoint=Some(ProgressCheckpointDocument{version:pse_model::document::Version,current,generation,highest_issued_sequence:head,dropped,current_is_latest:latest});
                    for event in page.into_iter().map(ProgressEventDocument::from) {
                        if events.len()==remaining {break;}
                        if let Some(gap)=consume(position,&event) {if gap {continuity=StudyObservationContinuity::GapCurrentResync;}events.push(event);}
                    }
                    remaining=remaining.saturating_sub(events.len());
                    if events.is_empty() && head>position.source_sequence {continuity=StudyObservationContinuity::GapCurrentResync;}
                }else{continuity=StudyObservationContinuity::LiveNumericsUnavailable;}
            }
            output.push(StudyObservedSource{position:position.clone(),continuity,events,checkpoint,coverage});
        }
        Ok(StudyObservationPage{status:status.document().clone(),sources:output,cursor:self.cursor.clone()})
    }
}
impl Drop for StudyObserver {fn drop(&mut self){self.close();}}
/// Source sequence alone owns continuity; append sequence and wall time never match events.
fn consume(position:&mut StudyObservationPosition,event:&ProgressEventDocument)->Option<bool> {
    let Some(source)=&event.source else {return Some(true);};
    if position.source_generation==Some(source.generation)&&source.sequence<=position.source_sequence {return None;}
    let gap=if position.source_generation==Some(source.generation){source.sequence!=position.source_sequence+1}else{position.source_generation.is_some()||source.sequence!=1};
    position.source_generation=Some(source.generation);position.source_sequence=source.sequence;Some(gap)
}
#[cfg(test)]
mod tests {
    use super::*;
    use pse_backend_native::solve::{Progress,Event,EventSource};
    fn position()->StudyObservationPosition {StudyObservationPosition{run:"run".into(),attempt:"attempt".into(),attempt_generation:1,metadata_revision:Some(9),source_generation:None,source_sequence:0,batch:0,offset:0,durable_sequence:None,history_complete:false}}
    fn event(progress:&Progress)->ProgressEventDocument {
        let mut e=Event{source:Some(EventSource::new("test",None,None)),phase:"step".into(),elapsed:Duration::ZERO,values:Default::default(),incumbent:None};
        progress.push(e.clone());
        e=progress.current().unwrap();
        ProgressEventDocument::from(e)
    }
    #[test]
    fn study_observer_reconnect_uses_exact_source_sequences_and_reports_missing_prefix() {
        let progress=Progress::new(8);
        let first=event(&progress);let second=event(&progress);let third=event(&progress);
        let mut original=position();assert_eq!(consume(&mut original,&first),Some(false));
        let mut reconnected=original.clone();
        assert_eq!(consume(&mut reconnected,&first),None);
        assert_eq!(consume(&mut reconnected,&second),Some(false));
        assert_eq!(consume(&mut reconnected,&third),Some(false));
        let mut gap=original;
        assert_eq!(consume(&mut gap,&third),Some(true));
        assert_eq!(gap.source_sequence,3);
        let mut absent_prefix=position();assert_eq!(consume(&mut absent_prefix,&second),Some(true));
        // Equal wall times cannot turn a different generation into continuity.
        let other=Progress::new(2);let replacement=event(&other);
        assert_eq!(consume(&mut gap,&replacement),Some(true));
    }
    #[test]
    fn study_observer_retains_telemetry_without_acquiring_scientific_cancellation() {
        let progress=Arc::new(Progress::new(1));
        let mut live=BTreeMap::<(String,String),Arc<Progress>>::new();live.insert(("run".into(),"attempt".into()),progress.clone());
        drop(live);
        assert_eq!(event(&progress).source.unwrap().sequence,1);
        assert_eq!(event(&progress).source.unwrap().sequence,2);
        let (retained,dropped)=progress.snapshot();assert_eq!(retained.len(),1);assert_eq!(dropped,1);
    }
}
#[cfg(all(test,feature="canonical-tests"))]
#[allow(unsafe_code,reason="controlled original sealed progress fixture admits partial outcome only")]
mod canonical_tests {
    use super::*;
    use pse_backend_native::solve::{Progress,Event,EventSource};
    use pse_operations::{canonical_execution::{ResultPayload,TerminalClass,SeedEligibility},canonical_studies::point_key};
    use pse_model::{study::OccurrenceKey,generated::enums::AttemptState};
    #[tokio::test]
    async fn study_observer_public_disconnect_reconnect_reads_full_history_or_reports_missing_prefix() {
        for missing_prefix in [false,true] {
            let runtime=crate::workflow::durable_tests::durable_runtime();
            let physical=runtime.record_physical_source(&crate::workflow::tests::physical(),&crate::CancelSource::new()).await.unwrap();
            let store=runtime.canonical_store().unwrap();let source=store.edit("observed-source",None,"source",&[]).await.unwrap();
            crate::workflow::study_workbench::canonical_tests::create(&runtime,"observed",&source,&physical,1).await;
            let scope=store.study_scope(&point_key("observed",OccurrenceKey(0))).await.unwrap();
            let claim=store.claim_study_point(&scope,None,"claim","worker",Duration::from_secs(60)).await.unwrap();
            let request=StudyObservationRequest{study:"observed".into(),maximum_members:2,maximum_events:1};
            let progress=Arc::new(Progress::new(8));let mut original_events=Vec::new();
            for _ in 0..3 {progress.push(Event{source:Some(EventSource::new("fixture",None,None)),phase:"step".into(),elapsed:Duration::ZERO,values:Default::default(),incumbent:None});original_events.push(ProgressEventDocument::from(progress.current().unwrap()));}
            let mut observer=runtime.observe_study(request.clone(),None).await.unwrap();observer.poll().await.unwrap();
            // Inject only the real passive Progress owner in this storage control;
            // production attach_run obtains this same Arc without the run's lease.
            observer.live.insert((claim.point.run.clone(),claim.fence.attempt().to_owned()),progress);
            let first=observer.poll().await.unwrap();assert_eq!(first.sources[0].events[0].source.as_ref().unwrap().sequence,1);
            let cursor=observer.cursor();observer.close();drop(observer);
            let selected=if missing_prefix {original_events[2..].to_vec()}else{original_events};
            let selected=selected.into_iter().enumerate().map(|(i,mut event)|{event.sequence=Some(i as i64+1);event}).collect::<Vec<_>>();
            store.append_result_batch(&claim.fence,"progress","__progress",0,ResultPayload::new(serde_json::to_vec(&selected).unwrap()).unwrap(),selected.len() as u64).await.unwrap();
            let stored=crate::workflow::StoredCompletion{version:2,attempt_id:pse_operations::mint_id(),completion:Some(Default::default()),termination:crate::workflow::TerminationDetail{version:pse_model::document::Version,cause:crate::workflow::TerminationCause::Assessment{usable:false,candidate_use:vec![],diagnostics:vec![]},point:None,retry_failure:None,effect:pse_model::study::EffectState::Absent},state:AttemptState::Partial,progress:crate::workflow::ProgressCoverage::Sampled{observed:3,queued:selected.len() as u64,dropped_before_queue:if missing_prefix{2}else{0},acknowledged:selected.len() as u64,unacknowledged:0,writer:crate::workflow::ProgressWriterOutcome::Complete}};
            let receipt=serde_json::to_vec(&serde_json::json!({"version":1,"batch_count":0,"payload_bytes":0,"digest":"","inline":stored})).unwrap();
            let closed=store.close_result_ingestion(&claim.fence,"close").await.unwrap();let manifest=store.reconcile_closed_attempt(&closed).await.unwrap();
            // SAFETY: original fixture progress is preserved exactly, independent of
            // the explicitly partial scientific completion and its unavailable result.
            unsafe {store.seal_attempt(&manifest,"seal",TerminalClass::Partial,&receipt,SeedEligibility::Unavailable).await}.unwrap();
            let mut reconnected=runtime.observe_study(request,Some(cursor)).await.unwrap();
            let mut sequences=Vec::new();let mut gap=false;
            for _ in 0..5 {let page=reconnected.poll().await.unwrap();for source in page.sources {gap|=source.continuity==StudyObservationContinuity::GapCurrentResync;sequences.extend(source.events.into_iter().map(|e|e.source.unwrap().sequence));}}
            assert_eq!(sequences,if missing_prefix{vec![3]}else{vec![2,3]});assert_eq!(gap,missing_prefix);
            assert!(!store.canonical_study("observed").await.unwrap().unwrap().cancelled);
            reconnected.close();drop(reconnected);store.remove_isolated_fixture().await.unwrap();
        }
    }
}
