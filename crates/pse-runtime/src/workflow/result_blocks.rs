// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical source-object stream encoding and shared bounded IPC layout checks.
//! Scientific result files are owned by result_files.

use datafusion::arrow::{
    array::ArrayData,
    datatypes::{DataType, Schema},
    ipc::{self, writer::StreamWriter},
    record_batch::RecordBatch,
};

const MAX_REPRESENTATION_BYTES: usize = pse_operations::canonical_files::FILE_BYTES;
const MAX_FIELDS: usize = 16 * 1024;
const MAX_ROWS: usize = 32 * 1024;
const MAX_VALUES: usize = MAX_REPRESENTATION_BYTES;
const MAX_IPC_TABLES: usize = 128 * 1024;

#[derive(Clone, Copy, Debug)]
enum Purpose {
    CanonicalSource,
}
impl Purpose {
    const fn bytes(self) -> usize {
        pse_operations::canonical::PAYLOAD_BYTES
    }
}

/// Malformed, incompatible or oversized scientific result transport.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(validation::invariant))]
pub enum ResultBlockError {
    /// Bounded transport refusal occurs before the Arrow decoder allocates.
    #[error("invalid result IPC block: {0}")]
    Invalid(&'static str),
    /// A representation extent can be retried with a deterministic smaller prefix.
    #[error("result IPC extent limit: {0}")]
    LimitReached(&'static str),
    /// Arrow validation retains its concrete error.
    #[error("result IPC: {0}")]
    Arrow(#[from] datafusion::arrow::error::ArrowError),
}

impl pse_diagnostics::TypedDiagnostic for ResultBlockError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(pse_diagnostics::DiagnosticCode::ValidationInvariant)
    }
}

fn encode_block(batch: &RecordBatch, purpose: Purpose) -> Result<Vec<u8>, ResultBlockError> {
    if batch.num_rows() > MAX_ROWS || batch.num_columns() > MAX_FIELDS {
        return Err(ResultBlockError::Invalid("batch shape exceeds bound"));
    }
    // Validate schema without allowing the writer to enter unsupported variants.
    let mut fields = 0usize;
    for field in batch.schema().fields() {
        pse_columnar::ipc_admission::trusted_shape(field.data_type(), 0, &mut fields)?;
    }
    // Schema/field/type tables and every custom-metadata entry must fit the
    // independent reader's FlatBuffer verifier before serialization allocates.
    let mut tables = 2 + 2 * fields + batch.schema().metadata().len();
    if tables > MAX_IPC_TABLES {
        return Err(ResultBlockError::Invalid("schema metadata complexity"));
    }
    let mut schema_bytes = 128usize;
    for (key, value) in batch.schema().metadata() {
        writer_schema_extent(&mut schema_bytes, 64, purpose)?;
        writer_schema_extent(&mut schema_bytes, key.len(), purpose)?;
        writer_schema_extent(&mut schema_bytes, value.len(), purpose)?;
    }
    for field in batch.schema().fields() {
        writer_metadata_tables(field, &mut tables, &mut schema_bytes, purpose)?;
    }
    // Bound writer scratch before IpcDataGenerator packs/copies the body. Sliced
    // arrays count their visible buffers rather than their retained parent buffers.
    let mut bytes = 0usize;
    let mut values = 0usize;
    for column in batch.columns() {
        bytes = bytes
            .checked_add(visible_size(&column.to_data(), &mut values)?)
            .ok_or(ResultBlockError::Invalid("array size overflow"))?;
    }
    if bytes > purpose.bytes() {
        return Err(ResultBlockError::LimitReached("batch body exceeds bound"));
    }
    let mut writer = pse_ids::BoundedSink::new(Vec::new(), purpose.bytes());
    {
        let mut stream = StreamWriter::try_new(&mut writer, batch.schema().as_ref())?;
        stream.write(batch)?;
        stream.finish()?;
    }
    Ok(writer.into_inner())
}

fn can_split(error: &ResultBlockError) -> bool {
    matches!(error, ResultBlockError::LimitReached(_))
        || matches!(pse_ids::sink_failure(error), Some(pse_ids::SinkFailure::LimitReached { .. }))
        || pse_ids::error_payload::<pse_columnar::ipc_file::ValueLimit>(error).is_some()
}

/// Backpressured canonical source-object ingestion with its fixed input bound.
pub(super) async fn visit_source_blocks_async<F, Fut, E>(
    batch: &RecordBatch,
    mut visitor: F,
) -> Result<(), E>
where
    F: FnMut(usize, usize, Vec<u8>) -> Fut,
    Fut: Future<Output = Result<(), E>>,
    E: From<ResultBlockError>,
{
    visit_blocks_async(batch, &mut visitor, Purpose::CanonicalSource).await
}

async fn visit_blocks_async<F, Fut, E>(
    batch: &RecordBatch,
    visitor: &mut F,
    purpose: Purpose,
) -> Result<(), E>
where
    F: FnMut(usize, usize, Vec<u8>) -> Fut,
    Fut: Future<Output = Result<(), E>>,
    E: From<ResultBlockError>,
{
    if batch.num_rows() == 0 {
        return visitor(0, 0, encode_block(batch, purpose)?).await;
    }
    let mut start = 0;
    while start < batch.num_rows() {
        let mut rows = (batch.num_rows() - start).min(MAX_ROWS);
        let payload = loop {
            match encode_block(&batch.slice(start, rows), purpose) {
                Ok(payload) => break payload,
                Err(error) if rows > 1 && can_split(&error) => rows = rows.div_ceil(2),
                Err(error) => return Err(error.into()),
            }
        };
        visitor(start, rows, payload).await?;
        start += rows;
    }
    Ok(())
}

pub(super) fn visible_size(
    data: &ArrayData,
    values: &mut usize,
) -> Result<usize, ResultBlockError> {
    Ok(pse_columnar::ipc_file::visible_size(data, values, MAX_VALUES)?)
}

// Include conservative FlatBuffer table/offset/alignment storage before Arrow
// copies schema strings. The output writer alone cannot bound that earlier copy.
fn writer_schema_extent(
    bytes: &mut usize,
    additional: usize,
    purpose: Purpose,
) -> Result<(), ResultBlockError> {
    *bytes = bytes
        .checked_add(additional)
        .ok_or(ResultBlockError::Invalid("schema metadata bytes"))?;
    if *bytes > purpose.bytes() {
        return Err(ResultBlockError::Invalid("schema metadata bytes"));
    }
    Ok(())
}

fn writer_metadata_tables(
    field: &datafusion::arrow::datatypes::Field,
    tables: &mut usize,
    bytes: &mut usize,
    purpose: Purpose,
) -> Result<(), ResultBlockError> {
    *tables = tables
        .checked_add(field.metadata().len())
        .ok_or(ResultBlockError::Invalid("schema metadata complexity"))?;
    if *tables > MAX_IPC_TABLES {
        return Err(ResultBlockError::Invalid("schema metadata complexity"));
    }
    writer_schema_extent(bytes, 128, purpose)?;
    writer_schema_extent(bytes, field.name().len(), purpose)?;
    for (key, value) in field.metadata() {
        writer_schema_extent(bytes, 64, purpose)?;
        writer_schema_extent(bytes, key.len(), purpose)?;
        writer_schema_extent(bytes, value.len(), purpose)?;
    }
    if let DataType::Timestamp(_, Some(timezone)) = field.data_type() {
        writer_schema_extent(bytes, timezone.len(), purpose)?;
    }
    match field.data_type() {
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::Map(child, _)
        | DataType::FixedSizeList(child, _) => {
            writer_metadata_tables(child, tables, bytes, purpose)?
        }
        DataType::Struct(children) => {
            for child in children {
                writer_metadata_tables(child, tables, bytes, purpose)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn schema_field(field: ipc::Field<'_>, depth: usize, count: &mut usize, expected_fields: usize) -> Result<(), ResultBlockError> {
    Ok(pse_columnar::ipc_admission::schema_field(field, depth, count, expected_fields)?)
}
pub(super) fn layout(ty: &DataType, nodes: &[&ipc::FieldNode], buffers: &[&ipc::Buffer], node: &mut usize, buffer: &mut usize, expected: Option<usize>) -> Result<(), ResultBlockError> {
    Ok(pse_columnar::ipc_admission::layout(ty, nodes, buffers, node, buffer, expected)?)
}
pub(super) fn expected_shape(expected: &Schema) -> Result<usize, ResultBlockError> {
    Ok(pse_columnar::ipc_admission::expected_shape(expected)?)
}

#[cfg(test)]
mod source_stream_unit {
    use super::*;
    use datafusion::arrow::{array::Float64Array, datatypes::Field, ipc::reader::StreamReader};
    use std::{io::Cursor, sync::Arc};

    #[test]
    fn arrow_wrapped_limits_are_splittable_but_cancellation_and_io_are_not() {
        use std::io::Write;
        let mut bounded = pse_ids::BoundedSink::new(Vec::new(), 0);
        let error = StreamWriter::try_new(&mut bounded, &Schema::empty()).err().expect("zero bound refuses schema");
        assert!(can_split(&ResultBlockError::Arrow(error)));
        let mut cancelled = pse_ids::BoundedSink::new(Vec::new(), 0).with_checkpoint(|| {
            Err(std::io::Error::other(pse_diagnostics::WorkError::Cancelled))
        });
        let error = StreamWriter::try_new(&mut cancelled, &Schema::empty()).err().expect("checkpoint refuses schema");
        assert!(matches!(pse_ids::error_payload::<pse_diagnostics::WorkError>(&error), Some(pse_diagnostics::WorkError::Cancelled)));
        assert!(!can_split(&ResultBlockError::Arrow(error)));
        #[derive(Debug)]
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> { Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)) }
            fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
        }
        let mut broken = pse_ids::BoundedSink::new(Broken, 4096);
        let error = StreamWriter::try_new(&mut broken, &Schema::empty()).err().expect("inner I/O fails");
        assert!(!can_split(&ResultBlockError::Arrow(error)));
    }
    #[tokio::test]
    async fn canonical_source_encoder_retains_its_bounded_stream_contract() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Float64,
            false,
        )]));
        let source = RecordBatch::try_new(
            schema,
            vec![Arc::new(Float64Array::from_iter_values(
                (0..150_000).map(|i| i as f64),
            ))],
        )
        .unwrap();
        let mut covered = 0;
        visit_source_blocks_async(&source, |start, rows, bytes| {
            std::future::ready((|| {
                assert_eq!(start, covered);
                assert!(bytes.len() <= pse_operations::canonical::PAYLOAD_BYTES);
                let mut reader = StreamReader::try_new(Cursor::new(bytes), None)?;
                let decoded = reader.next().unwrap()?;
                assert_eq!(decoded.schema(), source.schema());
                assert_eq!(decoded.num_rows(), rows);
                assert!(reader.next().is_none());
                covered += rows;
                Ok::<(), ResultBlockError>(())
            })())
        })
        .await
        .unwrap();
        assert_eq!(covered, source.num_rows());
    }
}
