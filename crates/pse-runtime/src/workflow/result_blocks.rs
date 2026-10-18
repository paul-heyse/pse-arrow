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
use std::io::Write;

const MAX_REPRESENTATION_BYTES: usize = pse_operations::canonical_files::FILE_BYTES;
const MAX_FIELDS: usize = 16 * 1024;
const MAX_DEPTH: usize = 64;
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
    /// Arrow validation retains its concrete error.
    #[error("result IPC: {0}")]
    Arrow(#[from] datafusion::arrow::error::ArrowError),
}

impl pse_diagnostics::TypedDiagnostic for ResultBlockError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(pse_diagnostics::DiagnosticCode::ValidationInvariant)
    }
}

struct LimitedWriter {
    bytes: Vec<u8>,
    purpose: Purpose,
}
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > self.purpose.bytes() {
            return Err(std::io::Error::other("result block limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn encode_block(batch: &RecordBatch, purpose: Purpose) -> Result<Vec<u8>, ResultBlockError> {
    if batch.num_rows() > MAX_ROWS || batch.num_columns() > MAX_FIELDS {
        return Err(ResultBlockError::Invalid("batch shape exceeds bound"));
    }
    // Validate schema without allowing the writer to enter unsupported variants.
    let mut fields = 0usize;
    for field in batch.schema().fields() {
        trusted_shape(field.data_type(), 0, &mut fields)?;
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
        return Err(ResultBlockError::Invalid("batch body exceeds bound"));
    }
    let mut writer = LimitedWriter {
        bytes: Vec::new(),
        purpose,
    };
    {
        let mut stream = StreamWriter::try_new(&mut writer, batch.schema().as_ref())?;
        stream.write(batch)?;
        stream.finish()?;
    }
    Ok(writer.bytes)
}

fn can_split(error: &ResultBlockError) -> bool {
    matches!(
        error,
        ResultBlockError::Invalid("batch body exceeds bound" | "field value bound")
    ) || matches!(error,ResultBlockError::Arrow(datafusion::arrow::error::ArrowError::IoError(message,_)) if message=="result block limit")
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
    *values = values
        .checked_add(data.len())
        .ok_or(ResultBlockError::Invalid("field size overflow"))?;
    if *values > MAX_VALUES {
        return Err(ResultBlockError::Invalid("field value bound"));
    }
    let bitmap = if data.nulls().is_some() {
        data.len().div_ceil(8)
    } else {
        0
    };
    let nested = match data.data_type() {
        DataType::List(_) | DataType::Map(_, _) => {
            let offsets = data.buffer::<i32>(0);
            let start = usize::try_from(offsets[0])
                .map_err(|_| ResultBlockError::Invalid("negative list offset"))?;
            let end = usize::try_from(offsets[data.len()])
                .map_err(|_| ResultBlockError::Invalid("negative list offset"))?;
            Some(
                bitmap
                    + (data.len() + 1) * 4
                    + visible_size(&data.child_data()[0].slice(start, end - start), values)?,
            )
        }
        DataType::LargeList(_) => {
            let offsets = data.buffer::<i64>(0);
            let start = usize::try_from(offsets[0])
                .map_err(|_| ResultBlockError::Invalid("negative list offset"))?;
            let end = usize::try_from(offsets[data.len()])
                .map_err(|_| ResultBlockError::Invalid("negative list offset"))?;
            Some(
                bitmap
                    + (data.len() + 1) * 8
                    + visible_size(&data.child_data()[0].slice(start, end - start), values)?,
            )
        }
        DataType::FixedSizeList(_, width) => Some(
            bitmap
                + visible_size(
                    &data.child_data()[0].slice(
                        data.offset() * (*width as usize),
                        data.len() * (*width as usize),
                    ),
                    values,
                )?,
        ),
        DataType::Struct(_) => {
            let mut size = bitmap;
            for child in data.child_data() {
                size = size
                    .checked_add(visible_size(child, values)?)
                    .ok_or(ResultBlockError::Invalid("visible size overflow"))?;
            }
            Some(size)
        }
        _ => None,
    };
    Ok(match nested {
        Some(size) => size,
        None => data.get_slice_memory_size()?,
    })
}

fn trusted_shape(ty: &DataType, depth: usize, count: &mut usize) -> Result<(), ResultBlockError> {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_FIELDS {
        return Err(ResultBlockError::Invalid("schema complexity"));
    }
    match ty {
        DataType::Null
        | DataType::Boolean
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float16
        | DataType::Float32
        | DataType::Float64
        | DataType::Utf8
        | DataType::LargeUtf8
        | DataType::Binary
        | DataType::LargeBinary
        | DataType::Date32
        | DataType::Date64
        | DataType::Time32(
            datafusion::arrow::datatypes::TimeUnit::Second
            | datafusion::arrow::datatypes::TimeUnit::Millisecond,
        )
        | DataType::Time64(
            datafusion::arrow::datatypes::TimeUnit::Microsecond
            | datafusion::arrow::datatypes::TimeUnit::Nanosecond,
        )
        | DataType::Timestamp(_, _)
        | DataType::Duration(_)
        | DataType::Decimal32(_, _)
        | DataType::Decimal64(_, _)
        | DataType::Decimal128(_, _)
        | DataType::Decimal256(_, _) => {}
        DataType::FixedSizeBinary(n) if *n > 0 && (*n as usize) <= MAX_REPRESENTATION_BYTES => {}
        DataType::List(child) | DataType::LargeList(child) | DataType::Map(child, _) => {
            trusted_shape(child.data_type(), depth + 1, count)?;
        }
        DataType::FixedSizeList(child, n) if *n > 0 && (*n as usize) <= MAX_VALUES => {
            trusted_shape(child.data_type(), depth + 1, count)?;
        }
        DataType::Struct(children) => {
            for child in children {
                trusted_shape(child.data_type(), depth + 1, count)?;
            }
        }
        _ => return Err(ResultBlockError::Invalid("unsupported schema type")),
    }
    Ok(())
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

pub(super) fn schema_field(
    field: ipc::Field<'_>,
    depth: usize,
    count: &mut usize,
    expected_fields: usize,
) -> Result<(), ResultBlockError> {
    *count += 1;
    if depth > MAX_DEPTH
        || *count > expected_fields
        || field.dictionary().is_some()
        || field.type_().is_none()
    {
        return Err(ResultBlockError::Invalid(
            "schema complexity/dictionary/type",
        ));
    }
    let children = field.children();
    let child_count = children.map_or(0, |c| c.len());
    let valid = match field.type_type() {
        ipc::Type::Null
        | ipc::Type::Bool
        | ipc::Type::Utf8
        | ipc::Type::LargeUtf8
        | ipc::Type::Binary
        | ipc::Type::LargeBinary => child_count == 0,
        ipc::Type::Int => {
            field
                .type_as_int()
                .is_some_and(|v| matches!(v.bitWidth(), 8 | 16 | 32 | 64))
                && child_count == 0
        }
        ipc::Type::FloatingPoint => {
            field.type_as_floating_point().is_some_and(|v| {
                matches!(
                    v.precision(),
                    ipc::Precision::HALF | ipc::Precision::SINGLE | ipc::Precision::DOUBLE
                )
            }) && child_count == 0
        }
        ipc::Type::FixedSizeBinary => {
            field.type_as_fixed_size_binary().is_some_and(|v| {
                v.byteWidth() > 0 && (v.byteWidth() as usize) <= MAX_REPRESENTATION_BYTES
            }) && child_count == 0
        }
        ipc::Type::Date => {
            field.type_as_date().is_some_and(|v| {
                matches!(v.unit(), ipc::DateUnit::DAY | ipc::DateUnit::MILLISECOND)
            }) && child_count == 0
        }
        ipc::Type::Time => {
            field.type_as_time().is_some_and(|v| {
                matches!(
                    (v.bitWidth(), v.unit()),
                    (32, ipc::TimeUnit::SECOND | ipc::TimeUnit::MILLISECOND)
                        | (64, ipc::TimeUnit::MICROSECOND | ipc::TimeUnit::NANOSECOND)
                )
            }) && child_count == 0
        }
        ipc::Type::Timestamp => {
            field
                .type_as_timestamp()
                .is_some_and(|v| time_unit(v.unit()))
                && child_count == 0
        }
        ipc::Type::Duration => {
            field
                .type_as_duration()
                .is_some_and(|v| time_unit(v.unit()))
                && child_count == 0
        }
        ipc::Type::Decimal => {
            field.type_as_decimal().is_some_and(|v| {
                matches!(v.bitWidth(), 32 | 64 | 128 | 256)
                    && u8::try_from(v.precision()).is_ok()
                    && i8::try_from(v.scale()).is_ok()
            }) && child_count == 0
        }
        ipc::Type::List | ipc::Type::LargeList | ipc::Type::Map => child_count == 1,
        ipc::Type::FixedSizeList => {
            field
                .type_as_fixed_size_list()
                .is_some_and(|v| v.listSize() > 0 && (v.listSize() as usize) <= MAX_VALUES)
                && child_count == 1
        }
        ipc::Type::Struct_ => true,
        _ => false,
    };
    if !valid {
        return Err(ResultBlockError::Invalid(
            "invalid or unsupported schema type",
        ));
    }
    if let Some(children) = children {
        for child in children {
            schema_field(child, depth + 1, count, expected_fields)?;
        }
    }
    Ok(())
}
fn time_unit(unit: ipc::TimeUnit) -> bool {
    matches!(
        unit,
        ipc::TimeUnit::SECOND
            | ipc::TimeUnit::MILLISECOND
            | ipc::TimeUnit::MICROSECOND
            | ipc::TimeUnit::NANOSECOND
    )
}

// Validate bitmap and data minima before decoder constructors (some Arrow
// constructors assert their buffer lengths). The nested node traversal is exactly
// the declared schema traversal; buffers cannot be borrowed from another field.
pub(super) fn layout(
    ty: &DataType,
    nodes: &[&ipc::FieldNode],
    buffers: &[&ipc::Buffer],
    node: &mut usize,
    buffer: &mut usize,
    expected: Option<usize>,
) -> Result<(), ResultBlockError> {
    let field = nodes
        .get(*node)
        .ok_or(ResultBlockError::Invalid("missing layout node"))?;
    *node += 1;
    let n = usize::try_from(field.length())
        .map_err(|_| ResultBlockError::Invalid("negative layout length"))?;
    if expected.is_some_and(|v| n != v) {
        return Err(ResultBlockError::Invalid("nested field length"));
    }
    if matches!(ty, DataType::Null) {
        if field.null_count() != field.length() {
            return Err(ResultBlockError::Invalid("null field shape"));
        }
        return Ok(());
    }
    let mut take = |minimum: usize| -> Result<(), ResultBlockError> {
        let value = buffers
            .get(*buffer)
            .ok_or(ResultBlockError::Invalid("missing layout buffer"))?;
        *buffer += 1;
        if usize::try_from(value.length())
            .ok()
            .is_none_or(|len| len < minimum)
        {
            return Err(ResultBlockError::Invalid("short layout buffer"));
        }
        Ok(())
    };
    take(if field.null_count() > 0 {
        n.div_ceil(8)
    } else {
        0
    })?;
    match ty {
        DataType::Struct(children) => {
            for child in children {
                layout(child.data_type(), nodes, buffers, node, buffer, Some(n))?;
            }
        }
        DataType::FixedSizeList(child, width) => {
            let size = n
                .checked_mul(*width as usize)
                .ok_or(ResultBlockError::Invalid("list size overflow"))?;
            if size > MAX_VALUES {
                return Err(ResultBlockError::Invalid("list size bound"));
            }
            layout(child.data_type(), nodes, buffers, node, buffer, Some(size))?;
        }
        DataType::List(child) | DataType::Map(child, _) => {
            take((n + 1) * 4)?;
            layout(child.data_type(), nodes, buffers, node, buffer, None)?;
        }
        DataType::LargeList(child) => {
            take((n + 1) * 8)?;
            layout(child.data_type(), nodes, buffers, node, buffer, None)?;
        }
        DataType::Utf8 | DataType::Binary => {
            take((n + 1) * 4)?;
            take(0)?;
        }
        DataType::LargeUtf8 | DataType::LargeBinary => {
            take((n + 1) * 8)?;
            take(0)?;
        }
        DataType::Boolean => take(n.div_ceil(8))?,
        DataType::FixedSizeBinary(width) => take(
            n.checked_mul(*width as usize)
                .ok_or(ResultBlockError::Invalid("binary size overflow"))?,
        )?,
        other => {
            let width = other
                .primitive_width()
                .ok_or(ResultBlockError::Invalid("unsupported primitive layout"))?;
            take(
                n.checked_mul(width)
                    .ok_or(ResultBlockError::Invalid("primitive size overflow"))?,
            )?;
        }
    }
    Ok(())
}

/// Validate the declared supported schema shape before native conversion.
pub(super) fn expected_shape(expected: &Schema) -> Result<usize, ResultBlockError> {
    let mut fields = 0;
    for field in expected.fields() {
        trusted_shape(field.data_type(), 0, &mut fields)?;
    }
    Ok(fields)
}

#[cfg(test)]
mod source_stream_unit {
    use super::*;
    use datafusion::arrow::{array::Float64Array, datatypes::Field, ipc::reader::StreamReader};
    use std::{io::Cursor, sync::Arc};

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
