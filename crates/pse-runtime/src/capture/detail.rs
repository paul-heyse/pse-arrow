// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite source-issued detail using capture ownership and a nonblocking bounded writer.
use super::{CaptureBudget, CaptureError};
use pse_diagnostics::observation::{CoherentRecord, ObservationContext, ProducerDescriptor, ProducerId, RecordTap};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::OpenOptions, io::{BufRead, BufReader, Read, Write}, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}}, thread::JoinHandle, time::{Duration, Instant}};

/// Source coverage requested independently from scientific validity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum DetailRequirement {
    /// Deliberate finite sampling; never described as complete iterations.
    Sampled,
    /// Every source record in the selected family/interval must be retained.
    Complete,
}
/// Explicit selection acknowledged before future invocation children are constructed.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetailRequest {
    /// Standalone request version.
    pub version: pse_model::document::Version<1>,
    /// Exact versioned source families; absent/unsupported families remain incomplete.
    pub families: Vec<String>,
    /// Independent evidence sufficiency requirement.
    pub requirement: DetailRequirement,
    /// Retain one of every N observed selected-family records; complete requires one.
    pub sample_every: u64,
    /// Select CPU clocks only at actual synchronous callback boundaries.
    #[serde(default)]
    pub boundary_thread_cpu: bool,
    /// Explicit additional managed-group allowance, granted once; never copied from local budget.
    #[serde(default)]
    pub managed_budget: Option<CaptureBudget>,
    /// Original same-host CLOCK_MONOTONIC deadline; queued child starts cannot renew it.
    #[serde(default)]
    pub expiry_monotonic_ns: Option<u64>,
    /// Maximum window from acknowledgement, or earlier explicit owner completion.
    #[schemars(with="pse_model::document::ClosedDuration")]
    pub duration: Duration,
    /// Exclusive new output directory, owned by this selected capture only.
    pub output: PathBuf,
    /// Separate finite collector/queue/output allowances.
    pub budget: CaptureBudget,
}
/// Independent detail-collection outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum DetailOutcome {
    /// The selected producer interval remains open.
    Active,
    /// All selected source families were observed without loss and output flushed.
    Complete,
    /// Deliberately sampled, with any independent losses still reported.
    Sampled,
    /// Insufficient source coverage or failed/interrupted output.
    Incomplete,
}
/// Actual end of the acknowledged source interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum DetailEnd {
    /// Selected window reached its finite duration.
    WindowElapsed,
    /// Caller ended the invocation's selected interval.
    OwnerFinished,
    /// Capture was stopped without an observed owner/window completion.
    Stopped,
    /// Output failed or exhausted its finite bound.
    WriterFailed,
}
/// Exact source/writer populations for one selected family.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetailFamilyCoverage {
    /// Actual selected versioned family.
    pub family: String,
    /// Source events offered during the selected interval.
    pub observed: u64,
    /// Selected events admitted to the writer queue.
    pub queued: u64,
    /// Complete records written and flushed by this file owner.
    pub acknowledged: u64,
    /// Queued events without an observed successful flush.
    pub unacknowledged: u64,
    /// Intentionally omitted samples.
    pub sampled_out: u64,
    /// Queue/extent/collector refusals, distinct from sampling.
    pub dropped: u64,
}
/// Immutable/readable capture prefix, never scientific publication permission.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetailDocument {
    /// Standalone answer encoding.
    pub version: pse_model::document::Version<1>,
    /// Original explicit source selection.
    pub request: DetailRequest,
    /// Collection sufficiency independent of science.
    pub outcome: DetailOutcome,
    /// Actual acknowledged interval end, absent while active.
    pub end: Option<DetailEnd>,
    /// Producer timestamps use their individual operation process clocks.
    pub clock_scope: String,
    /// Same capture owner retains raw records independently from timeline rendering.
    pub records: PathBuf,
    /// Last successfully flushed complete JSONL prefix.
    pub readable_prefix_bytes: u64,
    /// Per-family populations.
    pub families: Vec<DetailFamilyCoverage>,
    /// Source sequence discontinuity, generation transition or tracker refusal.
    pub source_partial: bool,
    /// Producer descriptor output loss.
    pub metadata_dropped: u64,
    /// Explicit delegated-scope refusals; local completeness never proves these children.
    pub scope_unavailable: Vec<String>,
    /// Bounded writer error, absent when none was observed.
    pub error: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="kind", content="value", rename_all="snake_case", deny_unknown_fields)]
enum Frame { Header(DetailRequest), Producer(ProducerDescriptor), Record(CoherentRecord), End(DetailDocument) }
/// Bounded page of original source records, independent from derived explanations.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetailRecordPage {
    /// Page encoding contract.
    pub version: pse_model::document::Version<1>,
    /// Whole original source records, never reconstructed metric tuples.
    pub records: Vec<CoherentRecord>,
    /// Producer declarations encountered on this page; IDs link earlier declarations.
    pub producers: Vec<ProducerDescriptor>,
    /// Exact next JSONL byte boundary, absent at terminal/end-of-file.
    pub next_offset: Option<u64>,
    /// A retained incomplete tail was encountered.
    pub partial: bool,
}
#[derive(Debug)]
struct Family {
    name: String, observed: AtomicU64, queued: AtomicU64, acknowledged: AtomicU64,
    sampled_out: AtomicU64, dropped: AtomicU64,
}
fn increment(value: &AtomicU64) -> u64 {
    value.try_update(Ordering::Relaxed,Ordering::Relaxed,|v|Some(v.saturating_add(1))).unwrap_or(u64::MAX).saturating_add(1)
}
impl Family {
    fn snapshot(&self) -> DetailFamilyCoverage {
        let queued=self.queued.load(Ordering::Acquire); let acknowledged=self.acknowledged.load(Ordering::Acquire);
        DetailFamilyCoverage { family:self.name.clone(),observed:self.observed.load(Ordering::Acquire),queued,acknowledged,unacknowledged:queued.saturating_sub(acknowledged),sampled_out:self.sampled_out.load(Ordering::Acquire),dropped:self.dropped.load(Ordering::Acquire) }
    }
}
#[derive(Debug)]
struct Tap {
    request: DetailRequest,
    sender: tokio::sync::mpsc::Sender<Frame>,
    families: Vec<Family>,
    every: u64,
    thread_cpu: bool,
    deadline: Instant,
    closed: AtomicBool,
    source_partial: AtomicBool,
    metadata_dropped: AtomicU64,
    sequences: Mutex<HashMap<(ProducerId,usize),(u64,u64)>>,
    issued: Mutex<HashMap<(ProducerId,usize),(u64,u64)>>,
    described: Mutex<std::collections::HashSet<ProducerId>>,
    in_flight: AtomicUsize,
    managed_claimed: AtomicBool,
    scope_unavailable: Mutex<Vec<String>>,
    record_bytes: usize,
    owner_budget: pse_diagnostics::observation::OwnerBudget,
}
struct Offer<'a>(&'a Tap);
impl Drop for Offer<'_> {fn drop(&mut self){self.0.in_flight.fetch_sub(1,Ordering::SeqCst);}}
impl Tap {
    fn enter(&self)->Option<Offer<'_>> {
        if self.closed.load(Ordering::SeqCst) || Instant::now()>=self.deadline{return None;}
        self.in_flight.fetch_add(1,Ordering::SeqCst);
        let guard=Offer(self);
        if self.closed.load(Ordering::SeqCst) || Instant::now()>=self.deadline{drop(guard);return None;}
        Some(guard)
    }
    fn describe_selected(&self,producer:&ProducerDescriptor) {
        if producer.name.len()>256 {increment(&self.metadata_dropped);return;}
        let Ok(mut described)=self.described.try_lock() else {increment(&self.metadata_dropped);return;};
        if described.contains(&producer.id){return;}
        if described.len()>=256 {increment(&self.metadata_dropped);return;}
        match self.sender.clone().try_reserve_owned(){
            Ok(permit)=>{described.insert(producer.id);permit.send(Frame::Producer(producer.clone()));},
            Err(_)=>{increment(&self.metadata_dropped);},
        }
    }
    fn observe_selected(&self,record:&CoherentRecord,index:usize) {
        let family=&self.families[index];
        if let Ok(mut sequences)=self.sequences.try_lock(){
            let key=(record.producer,index);
            if sequences.len()>=256 && !sequences.contains_key(&key){self.source_partial.store(true,Ordering::Release);}
            else if let Some(sequence)=record.family_sequence {
                let previous=sequences.insert(key,(record.generation,sequence));
                if previous.map_or(sequence!=1,|(generation,previous)|generation!=record.generation || previous.checked_add(1)!=Some(sequence)){self.source_partial.store(true,Ordering::Release);}
            }else{self.source_partial.store(true,Ordering::Release);}
        }else{self.source_partial.store(true,Ordering::Release);}
        if record.values.iter().any(|f|f.availability==Some(pse_diagnostics::observation::RecordAvailability::Refused)){self.source_partial.store(true,Ordering::Release);}
        let observed=increment(&family.observed);
        if observed==u64::MAX{self.source_partial.store(true,Ordering::Release);}
        if (observed-1)%self.every!=0{increment(&family.sampled_out);return;}
        if record.retained_bytes()>self.record_bytes{increment(&family.dropped);return;}
        match self.sender.clone().try_reserve_owned(){
            Ok(permit)=>{increment(&family.queued);permit.send(Frame::Record(record.clone()));},
            Err(_)=>{increment(&family.dropped);},
        }
    }
}
impl RecordTap for Tap {
    fn source_unavailable(&self,_family:&str) {
        if let Some(_guard)=self.enter(){self.source_partial.store(true,Ordering::Release);}
    }
    fn issued(&self,record:&CoherentRecord) {
        let Some(index)=self.families.iter().position(|f|f.name==record.family)else{return;};
        let Some(_guard)=self.enter()else{return;};
        let Some(sequence)=record.family_sequence else{self.source_partial.store(true,Ordering::Release);return;};
        let Ok(mut issued)=self.issued.try_lock()else{self.source_partial.store(true,Ordering::Release);return;};
        let key=(record.producer,index);
        if issued.len()>=256 && !issued.contains_key(&key){self.source_partial.store(true,Ordering::Release);return;}
        let head=issued.entry(key).or_insert((record.generation,sequence));
        if head.0!=record.generation{self.source_partial.store(true,Ordering::Release);}
        head.1=head.1.max(sequence);
    }
    fn is_active(&self)->bool{!self.closed.load(Ordering::SeqCst) && Instant::now()<self.deadline}
    fn reserve_owner_bytes(&self,bytes:usize)->Option<pse_diagnostics::observation::OwnerReservation>{if self.is_active(){self.owner_budget.reserve(bytes)}else{None}}
    fn thread_cpu(&self)->bool{self.thread_cpu && !self.closed.load(Ordering::SeqCst) && Instant::now()<self.deadline}
    fn selected(&self,family:&str)->bool{!self.closed.load(Ordering::SeqCst) && Instant::now()<self.deadline && self.families.iter().any(|f|f.name==family)}
    fn describe(&self,producer:&ProducerDescriptor){if let Some(_guard)=self.enter(){self.describe_selected(producer);}}
    fn observe(&self,record:&CoherentRecord){
        let Some(index)=self.families.iter().position(|f|f.name==record.family)else{return;};
        if let Some(_guard)=self.enter(){self.observe_selected(record,index);}
    }
    fn offer(&self,record:&CoherentRecord,producer:&ProducerDescriptor){
        let Some(index)=self.families.iter().position(|f|f.name==record.family)else{return;};
        if let Some(_guard)=self.enter(){self.describe_selected(producer);self.observe_selected(record,index);}
    }
}
fn monotonic_ns()->Option<u64>{
    #[cfg(target_os="linux")]{let clock=rustix::time::clock_gettime_dynamic(rustix::time::DynamicClockId::Known(rustix::time::ClockId::Monotonic)).ok()?;u64::try_from(clock.tv_sec).ok()?.checked_mul(1_000_000_000)?.checked_add(u64::try_from(clock.tv_nsec).ok()?)}
    #[cfg(not(target_os="linux"))]{None}
}
/// Grant the separately declared managed allowance to one actual managed job group.
/// A second group cannot multiply the original capture allowance.
pub fn claim_managed_detail_budget()->Option<CaptureBudget>{
    let context=pse_diagnostics::observation::current()?;let tap=(context.record_tap()? as &dyn std::any::Any).downcast_ref::<Tap>()?;
    if tap.closed.load(Ordering::SeqCst){return None;}
    let budget=tap.request.managed_budget.clone()?;
    tap.managed_claimed.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).ok()?;Some(budget)
}
/// Record an explicitly unavailable delegated scope without changing scientific work.
pub fn mark_detail_scope_unavailable(reason:&str){
    let Some(context)=pse_diagnostics::observation::current()else{return;};
    let Some(tap)=context.record_tap().and_then(|tap|(tap as &dyn std::any::Any).downcast_ref::<Tap>())else{return;};
    tap.source_partial.store(true,Ordering::Release);
    if let Ok(mut reasons)=tap.scope_unavailable.try_lock(){if reasons.len()<16{reasons.push(reason.chars().take(512).collect());}}
}
/// Copy the actual explicit selection for a managed execution envelope.
/// Remaining duration is capped by the original capture deadline, never renewed.
pub fn current_detail_request() -> Option<DetailRequest> {
    let context=pse_diagnostics::observation::current()?;
    let tap=context.record_tap()?;
    let tap=(tap as &dyn std::any::Any).downcast_ref::<Tap>()?;
    if tap.closed.load(Ordering::SeqCst) {return None;}
    let remaining=tap.deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {return None;}
    let mut request=tap.request.clone();request.duration=remaining;Some(request)
}
/// Owns an optional bounded writer and selected context, never a native handle.
#[derive(Debug)]
pub struct DetailSession {
    context: ObservationContext,
    tap: Arc<Tap>,
    state: Arc<Mutex<DetailDocument>>,
    finish: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl DetailSession {
    /// Enter this context before constructing the actual invocation/attempt.
    pub fn context(&self) -> ObservationContext { self.context.clone() }
    /// Copy current coverage without waiting for numerical work.
    pub fn snapshot(&self) -> DetailDocument {
        let mut d=self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        d.families=self.tap.families.iter().map(Family::snapshot).collect();
        d.source_partial=self.tap.source_partial.load(Ordering::Acquire);
        d.metadata_dropped=self.tap.metadata_dropped.load(Ordering::Acquire);
        d.scope_unavailable=self.tap.scope_unavailable.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(); d
    }
    /// Stop the collector only; this does not cancel science.
    pub fn request_stop(&self) { self.tap.closed.store(true,Ordering::SeqCst); }
    /// End an owner-completed interval and join only this writer.
    pub fn finish(mut self) -> Result<DetailDocument,CaptureError> {
        self.finish.store(true,Ordering::Release); self.request_stop(); self.join()?; Ok(self.snapshot())
    }
    /// Wait for the finite selected window, independently from native execution.
    pub fn wait(mut self) -> Result<DetailDocument,CaptureError> { self.join()?; Ok(self.snapshot()) }
    /// Stop without asserting a completed source interval.
    pub fn stop(mut self) -> Result<DetailDocument,CaptureError> { self.request_stop(); self.join()?; Ok(self.snapshot()) }
    fn join(&mut self) -> Result<(),CaptureError> {
        if let Some(worker)=self.worker.take() { worker.join().map_err(|_|CaptureError::Worker)?; } Ok(())
    }
}
impl Drop for DetailSession {
    fn drop(&mut self) { self.request_stop(); if let Some(worker)=self.worker.take() { let _=worker.join(); } }
}
/// Acknowledge finite detail before constructing future scientific work under its context.
/// This is distinct from sampled attachment to an already-running checkpoint.
pub fn start_detailed_collection(context: ObservationContext, mut request: DetailRequest) -> Result<DetailSession,CaptureError> {
    if request.families.is_empty() || request.families.len()>32 || request.families.iter().any(|f| f.is_empty() || f.len()>128) || request.families.iter().enumerate().any(|(i,f)|request.families[..i].contains(f)) || request.sample_every==0 || (request.requirement==DetailRequirement::Complete && request.sample_every!=1) || request.duration.is_zero() || request.duration>Duration::from_secs(3600) || request.budget.memory_bytes<4<<20 || request.budget.output_bytes==0 {
        return Err(CaptureError::Invalid("detail needs unique finite families, positive interval/stride/budgets; complete coverage requires stride one".into()));
    }
    if !context.is_active() { return Err(CaptureError::Invalid("selected detail requires an admitted active owner before invocation construction".into())); }
    let now=Instant::now();
    if let Some(expiry)=request.expiry_monotonic_ns {
        let current=monotonic_ns().ok_or_else(||CaptureError::Invalid("delegated detail clock alignment unavailable".into()))?;
        request.duration=request.duration.min(Duration::from_nanos(expiry.checked_sub(current).ok_or_else(||CaptureError::Invalid("selected detail interval expired before owner admission".into()))?));
        if request.duration.is_zero(){return Err(CaptureError::Invalid("selected detail interval expired".into()));}
    }else if let Some(current)=monotonic_ns(){request.expiry_monotonic_ns=u64::try_from(request.duration.as_nanos()).ok().and_then(|duration|current.checked_add(duration));}
    std::fs::create_dir(&request.output)?;
    let records=request.output.join("records.jsonl");
    let mut output=OpenOptions::new().write(true).create_new(true).open(&records)?;
    let record_bytes=(request.budget.memory_bytes/32).clamp(4096,65536);
    let capacity=request.budget.memory_bytes.saturating_sub(request.budget.memory_bytes/4+8*record_bytes+65536).checked_div(3*record_bytes).unwrap_or(0).clamp(1,512);
    let (sender,mut receiver)=tokio::sync::mpsc::channel(capacity);
    let tap=Arc::new(Tap { request:request.clone(), sender, families:request.families.iter().map(|name|Family{name:name.clone(),observed:AtomicU64::new(0),queued:AtomicU64::new(0),acknowledged:AtomicU64::new(0),sampled_out:AtomicU64::new(0),dropped:AtomicU64::new(0)}).collect(), every:request.sample_every,thread_cpu:request.boundary_thread_cpu,deadline:now+request.duration,closed:AtomicBool::new(false),source_partial:AtomicBool::new(false),metadata_dropped:AtomicU64::new(0),sequences:Mutex::new(HashMap::new()),issued:Mutex::new(HashMap::new()),described:Mutex::new(std::collections::HashSet::new()),in_flight:AtomicUsize::new(0),managed_claimed:AtomicBool::new(false),scope_unavailable:Mutex::new(vec![]),owner_budget:pse_diagnostics::observation::OwnerBudget::new(request.budget.memory_bytes/4),record_bytes });
    let state=Arc::new(Mutex::new(DetailDocument { version:pse_model::document::Version,request:request.clone(),outcome:DetailOutcome::Active,end:None,clock_scope:"source operation process clocks; selected offer interval begins at acknowledged construction".into(),records,readable_prefix_bytes:0,families:vec![],source_partial:false,metadata_dropped:0,scope_unavailable:vec![],error:None }));
    let finish=Arc::new(AtomicBool::new(false));
    let writer_tap=Arc::clone(&tap); let writer_state=Arc::clone(&state); let writer_finish=Arc::clone(&finish);
    let worker=std::thread::Builder::new().name("pse-detail-writer".into()).spawn(move|| {
        let mut prefix=0u64;
        let write_frame=|frame:&Frame, output:&mut std::fs::File,prefix:&mut u64| -> std::io::Result<()> {
            let mut bytes=serde_json::to_vec(frame).map_err(std::io::Error::other)?; bytes.push(b'\n');
            if bytes.len()>6*record_bytes+65536 || prefix.saturating_add(bytes.len() as u64)>request.budget.output_bytes { return Err(std::io::Error::new(std::io::ErrorKind::FileTooLarge,"selected detail output exhausted")); }
            output.write_all(&bytes)?; output.flush()?; *prefix+=bytes.len() as u64; Ok(())
        };
        let mut error=write_frame(&Frame::Header(request.clone()),&mut output,&mut prefix).err();
        loop {
            if error.is_some() {writer_tap.closed.store(true,Ordering::SeqCst);while writer_tap.in_flight.load(Ordering::SeqCst)!=0{std::thread::sleep(Duration::from_millis(1));}break;}
            if Instant::now()>=writer_tap.deadline { writer_tap.closed.store(true,Ordering::SeqCst); }
            match receiver.try_recv() {
                Ok(frame)=>{
                    error=write_frame(&frame,&mut output,&mut prefix).err();
                    if error.is_none() && let Frame::Record(record)=&frame && let Some(family)=writer_tap.families.iter().find(|f|f.name==record.family) { increment(&family.acknowledged); }
                    writer_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).readable_prefix_bytes=prefix;
                },
                Err(tokio::sync::mpsc::error::TryRecvError::Empty)=>{
                    if writer_tap.closed.load(Ordering::SeqCst) && writer_tap.in_flight.load(Ordering::SeqCst)==0 {receiver.close();if receiver.is_empty(){break;}continue;}
                    std::thread::sleep(Duration::from_millis(5));
                },
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)=>break,
            }
        }
        writer_tap.closed.store(true,Ordering::SeqCst); receiver.close();
        // Admission and active-offer counting use one sequentially consistent order:
        // no entrant can pass its second closed check after this drain has observed zero.
        let issued=writer_tap.issued.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let observed=writer_tap.sequences.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if issued.len()!=observed.len() || issued.iter().any(|(key,head)|observed.get(key)!=Some(head)) {
            writer_tap.source_partial.store(true,Ordering::Release);
        }
        drop(observed);drop(issued);
        let mut d=writer_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        d.readable_prefix_bytes=prefix; d.families=writer_tap.families.iter().map(Family::snapshot).collect(); d.source_partial=writer_tap.source_partial.load(Ordering::Acquire); d.metadata_dropped=writer_tap.metadata_dropped.load(Ordering::Acquire);d.scope_unavailable=writer_tap.scope_unavailable.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        d.end=Some(if error.is_some(){DetailEnd::WriterFailed}else if writer_finish.load(Ordering::Acquire){DetailEnd::OwnerFinished}else if Instant::now()>=writer_tap.deadline{DetailEnd::WindowElapsed}else{DetailEnd::Stopped});
        let sufficient=error.is_none() && d.end!=Some(DetailEnd::Stopped) && !d.source_partial && d.metadata_dropped==0 && d.families.iter().all(|f|f.observed>0 && f.dropped==0 && f.unacknowledged==0);
        d.outcome=if sufficient {if request.requirement==DetailRequirement::Complete {DetailOutcome::Complete}else{DetailOutcome::Sampled}}else{DetailOutcome::Incomplete};
        d.error=error.as_ref().map(|e|e.to_string().chars().take(512).collect());
        if error.is_none() && let Err(e)=write_frame(&Frame::End(d.clone()),&mut output,&mut prefix) { d.outcome=DetailOutcome::Incomplete;d.end=Some(DetailEnd::WriterFailed);d.error=Some(e.to_string().chars().take(512).collect()); }
        d.readable_prefix_bytes=prefix;
    })?;
    Ok(DetailSession { context:context.with_record_tap(tap.clone()),tap,state,finish,worker:Some(worker) })
}
/// Read a finite retained prefix with bounded framing; unknown versions refuse.
/// A missing end frame remains incomplete rather than synthesizing terminal coverage.
pub fn read_detailed_collection(path: &Path, maximum_bytes: u64) -> Result<DetailDocument,CaptureError> {
    let mut input=BufReader::new(std::fs::File::open(path)?); let mut consumed=0u64; let mut complete_prefix=0u64; let mut header=None; let mut end=None;
    loop {
        let mut line=Vec::new();
        let n=input.by_ref().take(512*1024+1).read_until(b'\n',&mut line)?;
        if n==0 { break; }
        consumed=consumed.saturating_add(n as u64);
        if n>512*1024 || consumed>maximum_bytes { return Err(CaptureError::Invalid("detail read exceeds finite framing/output allowance".into())); }
        if !line.ends_with(b"\n") { if end.is_some(){return Err(CaptureError::Invalid("unexpected tail after detail end".into()));}break; }
        complete_prefix=consumed;
        let frame:Frame=serde_json::from_slice(&line).map_err(|e|CaptureError::Invalid(e.to_string()))?;
        match frame { Frame::Header(request)=>{if header.is_some(){return Err(CaptureError::Invalid("duplicate detail header".into()));}header=Some(request);},Frame::End(document)=>{if end.is_some(){return Err(CaptureError::Invalid("duplicate detail end".into()));}end=Some(document);},_=>{if header.is_none() || end.is_some(){return Err(CaptureError::Invalid("detail frame outside declared interval".into()));}} }
    }
    if let Some(mut document)=end {
        let request=header.ok_or_else(||CaptureError::Invalid("detail end has no header".into()))?;
        if serde_json::to_value(&request).map_err(|e|CaptureError::Invalid(e.to_string()))?!=serde_json::to_value(&document.request).map_err(|e|CaptureError::Invalid(e.to_string()))? {return Err(CaptureError::Invalid("detail end selection differs from header".into()));}
        document.readable_prefix_bytes=complete_prefix;
        return Ok(document);
    }
    let request=header.ok_or_else(||CaptureError::Invalid("detail has no complete versioned header".into()))?;
    Ok(DetailDocument {version:pse_model::document::Version,request,outcome:DetailOutcome::Incomplete,end:None,clock_scope:"retained source clocks; terminal coverage unavailable".into(),records:path.into(),readable_prefix_bytes:complete_prefix,families:vec![],source_partial:true,metadata_dropped:0,scope_unavailable:vec![],error:Some("no complete terminal frame; retained prefix only".into())})
}

/// Read finite typed source records using a file cursor; repeated pages do not rescan history.
pub fn read_detail_records(path:&Path,offset:u64,limit:usize,maximum_bytes:u64)->Result<DetailRecordPage,CaptureError> {
    use std::io::{Seek,SeekFrom};
    if limit==0 || limit>512 || maximum_bytes==0 {return Err(CaptureError::Invalid("detail page needs limit 1..512 and positive read allowance".into()));}
    let mut input=BufReader::new(std::fs::File::open(path)?);
    let mut first=Vec::new(); input.by_ref().take(512*1024+1).read_until(b'\n',&mut first)?;
    if first.len() as u64>maximum_bytes {return Err(CaptureError::Invalid("detail header exceeds declared read allowance".into()));}
    if first.len()>512*1024 || !first.ends_with(b"\n") || !matches!(serde_json::from_slice::<Frame>(&first).map_err(|e|CaptureError::Invalid(e.to_string()))?,Frame::Header(_)) {return Err(CaptureError::Invalid("detail page has no complete versioned header".into()));}
    if offset!=0 {
        if offset<first.len() as u64 {return Err(CaptureError::Invalid("detail cursor precedes header boundary".into()));}
        input.seek(SeekFrom::Start(offset-1))?;let mut boundary=[0];input.read_exact(&mut boundary)?;
        if boundary[0]!=b'\n' {return Err(CaptureError::Invalid("detail cursor is not a complete frame boundary".into()));}
    }
    let mut page=DetailRecordPage {version:pse_model::document::Version,records:vec![],producers:vec![],next_offset:None,partial:false};let mut consumed=first.len() as u64;
    loop {
        let start=input.stream_position()?;let mut line=Vec::new();let n=input.by_ref().take(512*1024+1).read_until(b'\n',&mut line)?;
        if n==0 {break;} consumed=consumed.saturating_add(n as u64);
        if n>512*1024 || consumed>maximum_bytes {return Err(CaptureError::Invalid("detail page exceeds declared read allowance".into()));}
        if !line.ends_with(b"\n") {page.partial=true;break;}
        match serde_json::from_slice::<Frame>(&line).map_err(|e|CaptureError::Invalid(e.to_string()))? {
            Frame::Record(record)=>page.records.push(record),Frame::Producer(producer)=>page.producers.push(producer),Frame::End(_)=>break,Frame::Header(_)=>return Err(CaptureError::Invalid("duplicate detail header".into())),
        }
        if page.records.len()==limit || page.producers.len()==256 {page.next_offset=Some(start+n as u64);break;}
    }
    Ok(page)
}

#[cfg(test)]
#[path = "detail_acceptance_tests.rs"]
mod acceptance_tests;
#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{ObservationHub,Policy,ProducerRole,RecordField,Value};
    fn request(path:PathBuf,requirement:DetailRequirement)->DetailRequest {
        DetailRequest {version:pse_model::document::Version,families:vec!["iteration.v1".into()],requirement,sample_every:1,boundary_thread_cpu:false,managed_budget:None,expiry_monotonic_ns:None,duration:Duration::from_secs(10),output:path,budget:CaptureBudget::default()}
    }
    fn record(p:&pse_diagnostics::observation::Producer,n:i64)->CoherentRecord {
        p.record("iteration.v1","ordinary",Some("submitted"),None,vec![RecordField{name:"iteration".into(),unit:"iteration".into(),value:Value::Integer(n),availability:None}])
    }
    #[test]
    fn selected_complete_and_sampled_are_independent_from_history_zero() {
        let directory=tempfile::tempdir().unwrap(); let hub=ObservationHub::new(Policy {record_capacity:0,..Policy::default()}); let operation=hub.start("solve");
        let session=start_detailed_collection(operation.context(),request(directory.path().join("complete"),DetailRequirement::Complete)).unwrap();
        let child=session.context().child("native",pse_diagnostics::Relationship::Nested);
        let p=child.context().producer(ProducerRole::NativeIteration,"native");
        for n in 0..3 {p.publish(&record(&p,n));}
        let result=session.finish().unwrap();
        assert_eq!(result.outcome,DetailOutcome::Complete);assert_eq!(result.families[0].acknowledged,3);
        assert_eq!(read_detailed_collection(&result.records,1<<20).unwrap().outcome,DetailOutcome::Complete);
        assert!(child.reader().snapshot(Duration::from_secs(60)).producers[0].recent.is_empty());
        let mut selection=request(directory.path().join("sampled"),DetailRequirement::Sampled);selection.sample_every=2;
        let session=start_detailed_collection(operation.context(),selection).unwrap();let p=session.context().producer(ProducerRole::NativeIteration,"native");
        for n in 0..3 {p.publish(&record(&p,n));}
        let result=session.finish().unwrap();assert_eq!(result.outcome,DetailOutcome::Sampled);assert_eq!(result.families[0].sampled_out,1);
    }
    #[test]
    fn filtered_other_family_is_not_a_source_gap_but_missing_publication_is() {
        let directory=tempfile::tempdir().unwrap();let hub=ObservationHub::new(Policy::default());let operation=hub.start("solve");
        let session=start_detailed_collection(operation.context(),request(directory.path().join("filtered"),DetailRequirement::Complete)).unwrap();let p=session.context().producer(ProducerRole::NativeIteration,"native");
        p.publish(&record(&p,1));p.publish(&p.record("other.v1","trial",None,None,vec![]));p.publish(&record(&p,2));
        assert_eq!(session.finish().unwrap().outcome,DetailOutcome::Complete);
        let session=start_detailed_collection(operation.context(),request(directory.path().join("gap"),DetailRequirement::Complete)).unwrap();let p=session.context().producer(ProducerRole::NativeIteration,"native");
        p.publish(&record(&p,1));let _lost=record(&p,2);p.publish(&record(&p,3));let result=session.finish().unwrap();assert_eq!(result.outcome,DetailOutcome::Incomplete);assert!(result.source_partial);
    }
    #[test]
    fn optional_output_failure_and_unsupported_family_never_change_science() {
        let directory=tempfile::tempdir().unwrap();let hub=ObservationHub::new(Policy::default());let operation=hub.start("science");
        let mut selection=request(directory.path().join("exhausted"),DetailRequirement::Complete);selection.budget.output_bytes=1;
        let session=start_detailed_collection(operation.context(),selection).unwrap();let p=session.context().producer(ProducerRole::NativeIteration,"native");p.publish(&record(&p,1));
        let result=session.finish().unwrap();assert_eq!(result.outcome,DetailOutcome::Incomplete);assert!(result.error.is_some());assert!(operation.context().is_active());
        let session=start_detailed_collection(operation.context(),request(directory.path().join("unsupported"),DetailRequirement::Complete)).unwrap();
        assert_eq!(session.finish().unwrap().outcome,DetailOutcome::Incomplete);
        operation.finish(pse_diagnostics::OperationOutcome::Completed);
    }
    #[test]
    fn queue_pressure_and_oversized_events_report_loss_without_blocking() {
        let directory=tempfile::tempdir().unwrap();let hub=ObservationHub::new(Policy::default());let operation=hub.start("science");
        let session=start_detailed_collection(operation.context(),request(directory.path().join("pressure"),DetailRequirement::Complete)).unwrap();let p=session.context().producer(ProducerRole::NativeIteration,"native");
        let mut large=record(&p,1);large.values.push(RecordField{name:"large".into(),unit:"bytes".into(),value:Value::Text("x".repeat(1<<20)),availability:None});p.publish(&large);
        let result=session.finish().unwrap();assert_eq!(result.outcome,DetailOutcome::Incomplete);assert_eq!(result.families[0].dropped,1);assert!(operation.context().is_active());
    }
    #[test]
    fn stop_and_crash_prefix_do_not_manufacture_a_complete_interval() {
        let directory=tempfile::tempdir().unwrap();let hub=ObservationHub::new(Policy::default());let operation=hub.start("science");
        let session=start_detailed_collection(operation.context(),request(directory.path().join("stopped"),DetailRequirement::Complete)).unwrap();let p=session.context().producer(ProducerRole::NativeIteration,"native");p.publish(&record(&p,1));
        let result=session.stop().unwrap();assert_eq!(result.outcome,DetailOutcome::Incomplete);assert_eq!(result.end,Some(DetailEnd::Stopped));
        let path=directory.path().join("crash.jsonl");let mut bytes=serde_json::to_vec(&Frame::Header(request(directory.path().join("prefix"),DetailRequirement::Complete))).unwrap();bytes.push(b'\n');let prefix=bytes.len();bytes.extend_from_slice(b"{\"kind\":\"record\"");std::fs::write(&path,bytes).unwrap();
        let result=read_detailed_collection(&path,1<<20).unwrap();assert_eq!(result.outcome,DetailOutcome::Incomplete);assert_eq!(result.readable_prefix_bytes,prefix as u64);
    }
}
