// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite immutable projections over existing raw evidence; no scientific work or store.
use super::{CaptureDocument, CaptureError, DetailRecordPage};
use crate::invocation::InvocationEvidenceDocument;
use datafusion::{arrow::{array::{ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray, UInt32Array, UInt64Array}, record_batch::RecordBatch}, datasource::MemTable};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryPool, AllocationLease};
use pse_diagnostics::observation::{CoherentRecord, Value};
use std::{collections::{HashMap,HashSet},sync::Arc};
fn invalid(error:impl std::fmt::Display)->CaptureError { CaptureError::Invalid(error.to_string()) }
fn schema(name:&str)->Result<datafusion::arrow::datatypes::SchemaRef,CaptureError> {
    let registry=pse_schema::registry().map_err(invalid)?;
    let relation=registry.relation(name).ok_or_else(||invalid("missing declared diagnostic view"))?;
    pse_schema::arrow::relation_schema_ref(registry,relation).map_err(invalid)
}
fn reserve(pool:&Arc<dyn MemoryPool>,bytes:usize)->Result<Arc<AllocationLease>,CaptureError> {
    let reservation=MemoryConsumer::new("diagnostic:evidence-view").register(pool);
    reservation.try_grow(bytes).map_err(invalid)?;
    Ok(AllocationLease::new(reservation))
}
fn publish(name:&str,arrays:Vec<ArrayRef>,owner:Arc<AllocationLease>,cancel:&CancellationToken)->Result<RecordBatch,CaptureError> {
    let batch=RecordBatch::try_new(schema(name)?,arrays).map_err(invalid)?;
    // Preflight includes wrapper/schema/storage headroom; inspect actual escaped capacity too.
    let wrappers=pse_columnar::owned_buffer::payload_wrapper_units(&batch).map_err(invalid)?;
    let actual=batch.get_array_memory_size().checked_add(wrappers.checked_mul(512).ok_or_else(||invalid("view wrapper overflow"))?)
        .ok_or_else(||invalid("view allocation overflow"))?;
    if actual>owner.size() { return Err(invalid("diagnostic view exceeded its pre-admitted allocation")); }
    pse_columnar::owned_buffer::retain_payload(&batch,owner,cancel).map_err(invalid)
}
fn strings<'a>(values:impl IntoIterator<Item=Option<&'a str>>)->ArrayRef { Arc::new(StringArray::from_iter(values)) }
fn json(value:&impl serde::Serialize)->Result<String,CaptureError> { serde_json::to_string(value).map_err(invalid) }
fn reserve_document(value:&impl serde::Serialize,pool:&Arc<dyn MemoryPool>)->Result<Arc<AllocationLease>,CaptureError> {
    struct Extent(usize);
    impl std::io::Write for Extent {
        fn write(&mut self,bytes:&[u8])->std::io::Result<usize> {
            self.0=self.0.checked_add(bytes.len()).ok_or_else(||std::io::Error::other("diagnostic document extent overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self)->std::io::Result<()> {Ok(())}
    }
    let mut extent=Extent(0);serde_json::to_writer(&mut extent,value).map_err(invalid)?;
    reserve(pool,extent.0.checked_mul(8).and_then(|n|n.checked_add(65536)).ok_or_else(||invalid("document view extent overflow"))?)
}
/// Project a selected source page. Repeated event identities refuse instead of hiding duplicates.
/// Missing prefixes and source gaps remain partial; an empty event survives with ordinal zero.
/// # Errors
/// Cancellation, duplicate source identity, invalid declaration/value or finite allocation refusal.
pub fn detail_arrow(page:&DetailRecordPage,pool:&Arc<dyn MemoryPool>,cancel:&CancellationToken)->Result<RecordBatch,CaptureError> {
    cancel.checkpoint().map_err(invalid)?;
    let rows=page.records.iter().try_fold(0usize,|n,r|n.checked_add(r.values.len().max(1))).ok_or_else(||invalid("view row overflow"))?;
    let source=page.records.iter().try_fold(0usize,|n,r|n.checked_add(r.retained_bytes())).ok_or_else(||invalid("view source overflow"))?;
    let repeated=page.records.iter().try_fold(0usize,|n,r| {
        let text=r.family.len().checked_add(r.phase.len()).and_then(|n|n.checked_add(r.coordinates.as_ref().map_or(0,String::len)))
            .and_then(|n|n.checked_add(r.norm.as_ref().map_or(0,String::len)))?;
        n.checked_add(text.checked_mul(r.values.len().max(1))?)
    }).ok_or_else(||invalid("view repeated metadata overflow"))?;
    let bytes=rows.checked_mul(8192).and_then(|n|source.checked_mul(16).and_then(|s|n.checked_add(s)))
        .and_then(|n|repeated.checked_mul(8).and_then(|r|n.checked_add(r))).and_then(|n|n.checked_add(65536)).ok_or_else(||invalid("view extent overflow"))?;
    let owner=reserve(pool,bytes)?;
    let mut seen=HashSet::new(); let mut previous=HashMap::new(); let mut lost=HashSet::new();
    let mut members=Vec::with_capacity(rows); let mut operations=Vec::with_capacity(rows);
    for record in &page.records {
        cancel.checkpoint().map_err(invalid)?;
        if !seen.insert((record.producer,record.generation,record.sequence)) { return Err(invalid("repeated diagnostic source event; raw page retained")); }
        let family=(record.producer,record.generation,record.family.as_str());
        let gap=match previous.insert(family,record.family_sequence) {
            Some(Some(before)) => record.family_sequence.is_none_or(|after| before.checked_add(1)!=Some(after)),
            Some(None) => true,
            None => record.family_sequence!=Some(1),
        };
        if gap { lost.insert(family); }
        if record.values.iter().any(|f|matches!(f.value,Value::Number(v) if !v.is_finite())) { return Err(invalid("nonfinite source Number lacks explicit category")); }
        for ordinal in 0..record.values.len().max(1) {
            operations.push(record.producer.operation.to_string());
            members.push((record,record.values.get(ordinal),if record.values.is_empty(){0}else{ordinal as u64+1},page.partial||lost.contains(&family)));
        }
    }
    let text=|get:fn(&CoherentRecord)->Option<&str>|strings(members.iter().map(|(r,_,_,_)|get(r)));
    let count=|get:fn(&CoherentRecord)->u64|Arc::new(UInt64Array::from_iter_values(members.iter().map(|(r,_,_,_)|get(r)))) as ArrayRef;
    let kinds=members.iter().map(|(_,f,_,_)|f.map(|f|match &f.value{Value::Count(_)=>"count",Value::Integer(_)=>"integer",Value::Number(_)=>"number",Value::Boolean(_)=>"boolean",Value::Text(_)=>"text",Value::Unavailable(_)=>"unavailable",Value::NonFinite(_)=>"nonfinite",Value::Overflow=>"overflow"})).collect::<Vec<_>>();
    let texts=members.iter().map(|(_,f,_,_)|f.and_then(|f|match &f.value{Value::Text(v)|Value::Unavailable(v)=>Some(v.clone()),Value::NonFinite(v)=>Some(format!("{v:?}")),_=>None})).collect::<Vec<_>>();
    let availability=members.iter().map(|(_,f,_,_)|f.and_then(|f|f.availability).map(|v|format!("{v:?}"))).collect::<Vec<_>>();
    let arrays:Vec<ArrayRef>=vec![
        strings(operations.iter().map(|v|Some(v.as_str()))), count(|r|r.producer.sequence),count(|r|r.generation),count(|r|r.sequence),
        Arc::new(UInt64Array::from_iter(members.iter().map(|(r,_,_,_)|r.family_sequence))),count(|r|r.observed_at.as_secs()),
        Arc::new(UInt32Array::from_iter_values(members.iter().map(|(r,_,_,_)|r.observed_at.subsec_nanos()))),
        text(|r|Some(&r.family)),text(|r|Some(&r.phase)),text(|r|r.coordinates.as_deref()),text(|r|r.norm.as_deref()),
        Arc::new(UInt64Array::from_iter_values(members.iter().map(|(_,_,o,_)|*o))),
        strings(members.iter().map(|(_,f,_,_)|f.map(|f|f.name.as_str()))),strings(members.iter().map(|(_,f,_,_)|f.map(|f|f.unit.as_str()))), strings(kinds),
        Arc::new(UInt64Array::from_iter(members.iter().map(|(_,f,_,_)|f.and_then(|f|if let Value::Count(v)=f.value{Some(v)}else{None})))),
        Arc::new(Int64Array::from_iter(members.iter().map(|(_,f,_,_)|f.and_then(|f|if let Value::Integer(v)=f.value{Some(v)}else{None})))),
        Arc::new(Float64Array::from_iter(members.iter().map(|(_,f,_,_)|f.and_then(|f|if let Value::Number(v)=f.value{v.is_finite().then_some(v)}else{None})))),
        Arc::new(BooleanArray::from_iter(members.iter().map(|(_,f,_,_)|f.and_then(|f|if let Value::Boolean(v)=f.value{Some(v)}else{None})))),
        strings(texts.iter().map(|v|v.as_deref())),strings(availability.iter().map(|v|v.as_deref())),
        Arc::new(BooleanArray::from_iter(members.iter().map(|(_,_,_,p)|Some(*p)))),
    ];
    publish("runtime.diagnostic_evidence_fields",arrays,owner,cancel)
}
/// The same admitted buffers can be registered in an existing DataFusion context.
/// # Errors
/// The source page's projection or DataFusion table contract refuses.
pub fn detail_table(page:&DetailRecordPage,pool:&Arc<dyn MemoryPool>,cancel:&CancellationToken)->Result<Arc<MemTable>,CaptureError> {
    let batch=detail_arrow(page,pool,cancel)?;
    MemTable::try_new(batch.schema(),vec![vec![batch]]).map(Arc::new).map_err(invalid)
}
/// Project the actual joined invocation; no solver/attempt counters are summed.
/// # Errors
/// Serialization, declaration, cancellation or finite memory admission refuses.
pub fn invocation_arrow(document:&InvocationEvidenceDocument,pool:&Arc<dyn MemoryPool>,cancel:&CancellationToken)->Result<RecordBatch,CaptureError> {
    cancel.checkpoint().map_err(invalid)?;
    let owner=reserve_document(document,pool)?;
    let raw=json(document)?; let run=document.run_id.to_string(); let attempt=document.attempt_id.map(|v|v.to_string());let operation=document.operation.map(|v|v.to_string());let outcome=json(&document.outcome)?;
    let arrays:Vec<ArrayRef>=vec![strings([Some(run.as_str())]),strings([attempt.as_deref()]),strings([operation.as_deref()]),strings([Some(document.family.as_str())]),strings([Some(outcome.trim_matches('"'))]),strings([Some(document.clock_scope.as_str())]),
        Arc::new(Float64Array::from(vec![document.endpoints.qualification_seconds])),Arc::new(Float64Array::from(vec![document.endpoints.terminal_seconds])),Arc::new(Float64Array::from(vec![document.endpoints.response_ready_seconds])),
        Arc::new(BooleanArray::from(vec![document.omitted_intervals!=0||document.omitted_product_steps!=0||document.timing_unavailable.is_some()])),strings([Some(raw.as_str())])];
    publish("runtime.diagnostic_invocations",arrays,owner,cancel)
}
/// Link raw optional captures and their loss/clock/symbol scope to the exact operation.
/// # Errors
/// Serialization, declaration, cancellation or finite memory admission refuses.
pub fn capture_arrow(document:&CaptureDocument,pool:&Arc<dyn MemoryPool>,cancel:&CancellationToken)->Result<RecordBatch,CaptureError> {
    cancel.checkpoint().map_err(invalid)?;
    let owner=reserve_document(document,pool)?;
    let raw=json(document)?;let operation=document.operation.to_string();let kind=json(&document.kind)?;let status=json(&document.status)?;
    publish("runtime.diagnostic_captures",vec![strings([Some(operation.as_str())]),strings([Some(kind.trim_matches('"'))]),strings([Some(status.trim_matches('"'))]),strings([Some(raw.as_str())])],owner,cancel)
}
