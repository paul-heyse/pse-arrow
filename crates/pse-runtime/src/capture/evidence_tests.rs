// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Consumer controls over immutable evidence, not reconstructed solver facts.
use super::*;
use datafusion::{arrow::array::Array, prelude::SessionContext};
use pse_columnar::GreedyMemoryPool;
use pse_diagnostics::observation::{OperationId, ProducerId, RecordAvailability, RecordField};
use std::time::Duration;

fn pool(bytes: usize) -> Arc<dyn MemoryPool> { Arc::new(GreedyMemoryPool::new(bytes)) }
fn field(name: &str, value: Value) -> RecordField {
    RecordField { name: name.into(), unit: "source-unit".into(), value, availability: None }
}
fn record(sequence: u64, family_sequence: Option<u64>, values: Vec<RecordField>) -> CoherentRecord {
    CoherentRecord {
        producer: ProducerId { operation: OperationId { process_instance: [17, 19], sequence: 23 }, sequence: 29 },
        generation: 31, sequence, family_sequence,
        observed_at: Duration::new(u64::MAX, 987_654_321),
        family: "original_callback.v1".into(), phase: "original assessment".into(),
        coordinates: Some("physical-original".into()), norm: None, values,
    }
}
fn page(records: Vec<CoherentRecord>) -> DetailRecordPage {
    DetailRecordPage { version: Default::default(), records, producers: Vec::new(), next_offset: None, partial: false }
}
fn array<'a, T: datafusion::arrow::array::Array + 'static>(batch: &'a RecordBatch, name: &str) -> &'a T {
    batch.column_by_name(name).unwrap().as_any().downcast_ref::<T>().unwrap()
}
#[test]
fn evidence_empty_page_and_empty_source_event_remain_different_populations() {
    let pool = pool(1 << 20);
    let cancel = CancellationToken::new();
    let empty = detail_arrow(&page(Vec::new()), &pool, &cancel).unwrap();
    assert_eq!(empty.num_rows(), 0);
    assert_eq!(empty.num_columns(), 22);
    let source = page(vec![record(1, Some(1), Vec::new())]);
    let event = detail_arrow(&source, &pool, &cancel).unwrap();
    assert_eq!(event.num_rows(), 1);
    assert_eq!(array::<UInt64Array>(&event, "field_ordinal").value(0), 0);
    assert!(array::<StringArray>(&event, "field_name").is_null(0));
    assert!(array::<StringArray>(&event, "value_kind").is_null(0));
    assert_eq!(array::<UInt64Array>(&event, "event_sequence").value(0), 1);
    assert!(!array::<BooleanArray>(&event, "partial").value(0));
}
#[test]
fn evidence_exact_unsigned_time_and_typed_absence_survive_arrow_projection() {
    let mut absent = field("absent", Value::Unavailable("native source did not supply".into()));
    absent.availability = Some(RecordAvailability::NotSupplied);
    let source = page(vec![record(7, Some(1), vec![
        field("maximum", Value::Count(u64::MAX)), field("zero", Value::Count(0)),
        field("signed", Value::Integer(i64::MIN)), field("numeric_zero", Value::Number(0.)),
        field("false", Value::Boolean(false)), absent,
        field("nonfinite", Value::number(f64::INFINITY)), field("overflow", Value::Overflow),
    ])]);
    let raw = serde_json::to_vec(&source).unwrap();
    let batch = detail_arrow(&source, &pool(2 << 20), &CancellationToken::new()).unwrap();
    assert_eq!(batch.num_rows(), 8);
    assert_eq!(array::<UInt64Array>(&batch, "count_value").value(0), u64::MAX);
    assert_eq!(array::<UInt64Array>(&batch, "count_value").value(1), 0);
    assert!(array::<UInt64Array>(&batch, "count_value").is_null(5));
    assert_eq!(array::<Int64Array>(&batch, "integer_value").value(2), i64::MIN);
    assert_eq!(array::<Float64Array>(&batch, "number_value").value(3), 0.);
    assert!(!array::<BooleanArray>(&batch, "boolean_value").value(4));
    assert!(array::<Float64Array>(&batch, "number_value").is_null(6));
    assert_eq!(array::<StringArray>(&batch, "value_kind").value(6), "nonfinite");
    assert_eq!(array::<StringArray>(&batch, "availability").value(5), "not_supplied");
    assert_eq!(array::<UInt64Array>(&batch, "observed_seconds").value(0), u64::MAX);
    assert_eq!(array::<UInt32Array>(&batch, "observed_nanoseconds").value(0), 987_654_321);
    assert_eq!(array::<UInt64Array>(&batch, "producer").value(0), 29);
    assert_eq!(array::<UInt64Array>(&batch, "generation").value(0), 31);
    assert_eq!(serde_json::to_vec(&source).unwrap(), raw, "projection preserves raw authority");
}
#[test]
fn evidence_duplicate_identity_refuses_and_source_gaps_stay_partial() {
    let pool = pool(2 << 20);
    let cancel = CancellationToken::new();
    let original = record(1, Some(1), Vec::new());
    let duplicate = page(vec![original.clone(), original]);
    let raw = serde_json::to_vec(&duplicate).unwrap();
    assert!(detail_arrow(&duplicate, &pool, &cancel).is_err());
    assert_eq!(pool.reserved(), 0);
    assert_eq!(serde_json::to_vec(&duplicate).unwrap(), raw);
    // Family order, not the interleaved all-family source order, determines loss.
    let mut other_family = record(2, Some(1), Vec::new());
    other_family.family = "native_return.v1".into();
    let complete = detail_arrow(&page(vec![record(1, Some(1), Vec::new()), other_family,
        record(3, Some(2), Vec::new())]), &pool, &cancel).unwrap();
    assert!((0..3).all(|row| !array::<BooleanArray>(&complete, "partial").value(row)));
    drop(complete);
    let missing = detail_arrow(&page(vec![record(1, Some(1), Vec::new()), record(4, Some(3), Vec::new()),
        record(5, Some(4), Vec::new())]), &pool, &cancel).unwrap();
    assert_eq!((0..3).map(|row| array::<BooleanArray>(&missing, "partial").value(row)).collect::<Vec<_>>(), [false, true, true]);
    for sequence in [Some(4), None] {
        let prefix = detail_arrow(&page(vec![record(4, sequence, Vec::new())]), &pool, &cancel).unwrap();
        assert!(array::<BooleanArray>(&prefix, "partial").value(0));
    }
    let mut tail = page(vec![record(1, Some(1), Vec::new())]);
    tail.partial = true;
    let tail = detail_arrow(&tail, &pool, &cancel).unwrap();
    assert!(array::<BooleanArray>(&tail, "partial").value(0));
}
#[test]
fn evidence_refusal_cancellation_and_invalid_number_preserve_source_without_leaking_charge() {
    let source = page(vec![record(1, Some(1), vec![field("text", Value::Text("raw retained".into()))])]);
    let raw = serde_json::to_vec(&source).unwrap();
    let tiny = pool(1);
    assert!(detail_arrow(&source, &tiny, &CancellationToken::new()).is_err());
    assert_eq!(tiny.reserved(), 0);
    let roomy = pool(1 << 20);
    let cancel = CancellationToken::new(); cancel.cancel();
    assert!(detail_arrow(&source, &roomy, &cancel).is_err());
    assert_eq!(roomy.reserved(), 0);
    assert_eq!(serde_json::to_vec(&source).unwrap(), raw);
    let invalid = page(vec![record(1, Some(1), vec![field("bad", Value::Number(f64::NAN))])]);
    assert!(detail_arrow(&invalid, &roomy, &CancellationToken::new()).is_err());
    assert_eq!(roomy.reserved(), 0);
    assert!(matches!(invalid.records[0].values[0].value, Value::Number(value) if value.is_nan()));
}
#[test]
fn evidence_last_escaped_buffer_retains_its_allocation_after_batch_and_source_depart() {
    let pool = pool(2 << 20);
    let source = page(vec![record(1, Some(1), vec![field("text", Value::Text("retained payload".into()))])]);
    let batch = detail_arrow(&source, &pool, &CancellationToken::new()).unwrap();
    let charged = pool.reserved(); assert!(charged > 0);
    let buffer = batch.column_by_name("text_value").unwrap().to_data().buffers()[1].clone();
    drop(batch); drop(source);
    assert_eq!(pool.reserved(), charged, "the last exported buffer retains the full finite view lease");
    assert!(buffer.as_slice().windows("retained payload".len()).any(|text| text == b"retained payload"));
    drop(buffer);
    assert_eq!(pool.reserved(), 0);
}
#[tokio::test]
async fn evidence_registered_memtable_queries_typed_availability_without_reinterpreting_zero() {
    let source = page(vec![record(1, Some(1), vec![field("zero", Value::Count(0)),
        field("maximum", Value::Count(u64::MAX)), field("absent", Value::Unavailable("unavailable".into()))])]);
    let pool = pool(2 << 20);
    let ctx = SessionContext::new();
    ctx.register_table("selected_evidence", detail_table(&source, &pool, &CancellationToken::new()).unwrap()).unwrap();
    drop(source);
    let batches = ctx.sql("SELECT field_name, count_value, count_value IS NULL AS missing FROM selected_evidence ORDER BY field_ordinal")
        .await.unwrap().collect().await.unwrap();
    let batch = &batches[0];
    assert_eq!(batch.num_rows(), 3);
    assert_eq!(array::<UInt64Array>(batch, "count_value").value(0), 0);
    assert_eq!(array::<UInt64Array>(batch, "count_value").value(1), u64::MAX);
    assert_eq!((0..3).map(|row| array::<BooleanArray>(batch, "missing").value(row)).collect::<Vec<_>>(), [false, false, true]);
    assert!(pool.reserved() > 0, "table retains exported source buffers");
    drop(batches); drop(ctx);
    assert_eq!(pool.reserved(), 0);
}
#[test]
fn evidence_invocation_and_failed_capture_keep_independent_readiness_and_raw_links() {
    use crate::invocation::{InvocationEndpoints, InvocationOutcome};
    use crate::capture::{CaptureKind, CaptureLoss, CaptureOutput, CaptureOutputKind, CaptureStatus, CaptureSymbols};
    use pse_model::generated::identities::RunId;
    let run_id = RunId::from_bytes([73; 16]);
    let invocation = InvocationEvidenceDocument {
        version: 1, run_id, attempt_id: None, operation: None, family: "fit".into(),
        clock_scope: "actual supervisor".into(), timing_unavailable: None,
        canonical_run: None, canonical_attempt: None, outcome: InvocationOutcome::Failed,
        endpoints: InvocationEndpoints { qualification_seconds: None, terminal_seconds: Some(0.), response_ready_seconds: Some(1.) },
        intervals: Vec::new(), omitted_intervals: 0, products: Vec::new(), omitted_product_steps: 0,
        preparations: None, attempts: Arc::new(crate::workflow::AttemptSummary::empty_for_test(run_id)),
    };
    let pool = pool(2 << 20);
    let batch = invocation_arrow(&invocation, &pool, &CancellationToken::new()).unwrap();
    assert!(array::<Float64Array>(&batch, "qualification_seconds").is_null(0));
    assert_eq!(array::<Float64Array>(&batch, "terminal_seconds").value(0), 0.);
    assert_eq!(array::<Float64Array>(&batch, "response_ready_seconds").value(0), 1.);
    let directory = tempfile::tempdir().unwrap();
    let raw = directory.path().join("retained.partial.pftrace");
    std::fs::write(&raw, [0x0a, 0x00, 0x0a]).unwrap();
    let capture = CaptureDocument {
        version: Default::default(), kind: CaptureKind::Timeline, operation: record(1, Some(1), Vec::new()).producer.operation,
        status: CaptureStatus::Failed, requested_duration: Duration::from_secs(1), observed_duration: Duration::ZERO,
        targets: Vec::new(), outputs: vec![CaptureOutput { path: raw.clone(), kind: CaptureOutputKind::Perfetto,
            bytes: 3, readable_prefix_bytes: 2, readable: true }], loss: CaptureLoss::default(),
        symbols: CaptureSymbols::default(), open_intervals: 1, errors: vec!["controlled writer failure".into()],
    };
    let capture_batch = capture_arrow(&capture, &pool, &CancellationToken::new()).unwrap();
    let readback: CaptureDocument = serde_json::from_str(array::<StringArray>(&capture_batch, "document_json").value(0)).unwrap();
    assert_eq!(readback.status, CaptureStatus::Failed);
    assert_eq!(readback.outputs[0].path, raw);
    assert_eq!(readback.outputs[0].readable_prefix_bytes, 2);
    assert_eq!(std::fs::read(raw).unwrap(), [0x0a, 0x00, 0x0a], "failed capture projection retains the original source bytes");
}
