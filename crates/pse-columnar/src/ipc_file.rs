// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One bounded native file encoder. Encoding establishes representation only;
//! callers own relation admission, allocation prepayment and statistics authority.

use arrow::{
    array::ArrayData, datatypes::DataType, error::ArrowError, ipc::writer::FileWriter,
    record_batch::RecordBatch,
};

/// Existing physical-file record batch ceiling, also used by protected readers.
pub const MAX_BATCHES: usize = 32 * 1024;

/// A visible native value-slot ceiling reached before encoding.
#[derive(Debug, thiserror::Error)]
#[error("native value-slot limit {limit} reached")]
pub struct ValueLimit {
    /// Caller-selected native value-slot bound.
    pub limit: usize,
}
fn invalid(message: &str) -> ArrowError {
    ArrowError::InvalidArgumentError(message.into())
}

/// Count only the visible native slice, including selected nested children.
/// This is a representation preflight; it grants no relation or statistics authority.
/// # Errors
/// Overflow, unsupported native extent or a caller-selected value-slot bound.
pub fn visible_size(
    data: &ArrayData,
    values: &mut usize,
    value_limit: usize,
) -> Result<usize, ArrowError> {
    *values = values
        .checked_add(data.len())
        .ok_or_else(|| invalid("field size overflow"))?;
    if *values > value_limit {
        return Err(ArrowError::ExternalError(Box::new(ValueLimit { limit: value_limit })));
    }
    let bitmap = if data.nulls().is_some() {
        data.len().div_ceil(8)
    } else {
        0
    };
    let add = |a: usize, b: usize| {
        a.checked_add(b)
            .ok_or_else(|| invalid("visible size overflow"))
    };
    let list =
        |start: usize, end: usize, width: usize, values: &mut usize| -> Result<usize, ArrowError> {
            let offsets = data
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(width))
                .ok_or_else(|| invalid("visible size overflow"))?;
            let length = end
                .checked_sub(start)
                .ok_or_else(|| invalid("negative list offset"))?;
            add(
                add(bitmap, offsets)?,
                visible_size(
                    &data.child_data()[0].slice(start, length),
                    values,
                    value_limit,
                )?,
            )
        };
    match data.data_type() {
        DataType::List(_) | DataType::Map(_, _) => {
            let offsets = data.buffer::<i32>(0);
            list(
                usize::try_from(offsets[0]).map_err(|_| invalid("negative list offset"))?,
                usize::try_from(offsets[data.len()])
                    .map_err(|_| invalid("negative list offset"))?,
                4,
                values,
            )
        }
        DataType::LargeList(_) => {
            let offsets = data.buffer::<i64>(0);
            list(
                usize::try_from(offsets[0]).map_err(|_| invalid("negative list offset"))?,
                usize::try_from(offsets[data.len()])
                    .map_err(|_| invalid("negative list offset"))?,
                8,
                values,
            )
        }
        DataType::FixedSizeList(_, width) => {
            let width = usize::try_from(*width).map_err(|_| invalid("negative list width"))?;
            let start = data
                .offset()
                .checked_mul(width)
                .ok_or_else(|| invalid("visible size overflow"))?;
            let length = data
                .len()
                .checked_mul(width)
                .ok_or_else(|| invalid("visible size overflow"))?;
            add(
                bitmap,
                visible_size(
                    &data.child_data()[0].slice(start, length),
                    values,
                    value_limit,
                )?,
            )
        }
        DataType::Struct(_) => data.child_data().iter().try_fold(bitmap, |size, child| {
            add(size, visible_size(child, values, value_limit)?)
        }),
        _ => data.get_slice_memory_size(),
    }
}

/// Preparation for Arrow 59.3 file writing, including native batch/column
/// clones, schema/FlatBuffer metadata and nested ArrayData clones. `write_array_data`
/// keeps borrowed ancestors alive while slicing descendants; allow the full
/// declared depth of simultaneous subtree copies before traversing actual arrays.
/// # Errors
/// Unsupported shape or checked allocation overflow.
pub fn writer_preparation_extent(schema: &arrow::datatypes::Schema, batches: usize) -> Result<usize, ArrowError> {
    fn depth(ty: &DataType) -> usize {
        match ty {
            DataType::List(field) | DataType::LargeList(field) | DataType::FixedSizeList(field, _) | DataType::Map(field, _) => 1 + depth(field.data_type()),
            DataType::Struct(fields) => fields.iter().map(|field| 1 + depth(field.data_type())).max().unwrap_or(0),
            _ => 0,
        }
    }
    let overflow = || invalid("IPC writer allocation extent overflow");
    let fields = crate::ipc_admission::expected_shape(schema)?;
    let depth = schema.fields().iter().map(|field| depth(field.data_type())).max().unwrap_or(0);
    let batch_headers = schema.fields().len().checked_mul(size_of::<arrow::array::ArrayRef>())
        .and_then(|bytes| bytes.checked_add(size_of::<RecordBatch>())).and_then(|bytes| bytes.checked_mul(batches))
        .and_then(|bytes| bytes.checked_mul(2)).ok_or_else(overflow)?;
    let cloned_node = size_of::<ArrayData>().checked_add(3 * size_of::<arrow::buffer::Buffer>()).ok_or_else(overflow)?;
    let nested = depth.checked_add(1).and_then(|depth| depth.checked_mul(fields)).and_then(|nodes| nodes.checked_mul(cloned_node))
        .and_then(|bytes| bytes.checked_mul(2)).ok_or_else(overflow)?;
    let schema_bytes = crate::schema_working_extent(schema).map_err(|error| ArrowError::ExternalError(Box::new(error)))?;
    let arrays = crate::allocation_extent::ipc_array_working_extent(fields).map_err(|error| ArrowError::ExternalError(Box::new(error)))?;
    schema_bytes.checked_mul(4).and_then(|bytes| bytes.checked_add(arrays)).and_then(|bytes| bytes.checked_add(nested))
        .and_then(|bytes| bytes.checked_add(batch_headers)).ok_or_else(overflow)
}

/// Complete conservative 59.3 streaming writer envelope. Bodies are borrowed;
/// there is no copied full-body buffer. Offset rebasing, generated/shifted
/// validity and Boolean bit slices are retained by the current encoded-buffer
/// inventory until its write finishes. Footer/record metadata and batch headers
/// coexist with it. Vec output realloc may hold old (<limit) and new (<2*limit)
/// backing simultaneously, so its peak allowance is three output limits.
/// The caller acquires `writer_preparation_extent` before this array traversal.
/// # Errors
/// Empty/incompatible/unsupported inventory or checked extent overflow.
pub fn writer_extent(batches: &[RecordBatch], byte_limit: usize) -> Result<usize, ArrowError> {
    fn add(a: usize, b: usize) -> Result<usize, ArrowError> { a.checked_add(b).ok_or_else(|| invalid("IPC writer allocation extent overflow")) }
    fn round(n: usize) -> Result<usize, ArrowError> { Ok(add(n, 63)? / 64 * 64) }
    fn scratch(data: &ArrayData) -> Result<usize, ArrowError> {
        let mut bytes = if matches!(data.data_type(), DataType::Null) { 0 } else { round(data.len().div_ceil(8))? };
        let list = |width: usize, start: usize, end: usize, bytes| -> Result<usize, ArrowError> {
            let offsets = data.len().checked_add(1).and_then(|length| length.checked_mul(width)).ok_or_else(|| invalid("IPC offset scratch extent"))?;
            let length = end.checked_sub(start).ok_or_else(|| invalid("negative list offset"))?;
            add(add(bytes, round(offsets)?)?, scratch(&data.child_data()[0].slice(start, length))?)
        };
        match data.data_type() {
            DataType::List(_) | DataType::Map(_, _) => {
                let offsets = data.buffer::<i32>(0);
                list(4, usize::try_from(offsets[0]).map_err(|_| invalid("negative list offset"))?, usize::try_from(offsets[data.len()]).map_err(|_| invalid("negative list offset"))?, bytes)
            }
            DataType::LargeList(_) => {
                let offsets = data.buffer::<i64>(0);
                list(8, usize::try_from(offsets[0]).map_err(|_| invalid("negative list offset"))?, usize::try_from(offsets[data.len()]).map_err(|_| invalid("negative list offset"))?, bytes)
            }
            DataType::FixedSizeList(_, width) => {
                let width = usize::try_from(*width).map_err(|_| invalid("negative list width"))?;
                let start = data.offset().checked_mul(width).ok_or_else(|| invalid("list scratch extent"))?;
                let length = data.len().checked_mul(width).ok_or_else(|| invalid("list scratch extent"))?;
                add(bytes, scratch(&data.child_data()[0].slice(start, length))?)
            }
            DataType::Struct(_) => { for child in data.child_data() { bytes = add(bytes, scratch(child)?)?; } Ok(bytes) }
            DataType::Boolean => add(bytes, round(data.len().div_ceil(8))?),
            DataType::Utf8 | DataType::Binary => add(bytes, round(data.len().checked_add(1).and_then(|n| n.checked_mul(4)).ok_or_else(|| invalid("offset scratch extent"))?)?),
            DataType::LargeUtf8 | DataType::LargeBinary => add(bytes, round(data.len().checked_add(1).and_then(|n| n.checked_mul(8)).ok_or_else(|| invalid("offset scratch extent"))?)?),
            _ => Ok(bytes),
        }
    }
    let first = batches.first().ok_or_else(|| invalid("empty file inventory"))?;
    let preparation = writer_preparation_extent(first.schema().as_ref(), batches.len())?;
    let mut maximum = 0;
    for batch in batches {
        if batch.schema() != first.schema() { return Err(invalid("incompatible packed schemas")); }
        let mut bytes = 0;
        for array in batch.columns() { bytes = add(bytes, scratch(&array.to_data())?)?; }
        maximum = maximum.max(bytes);
    }
    let output = byte_limit.checked_mul(3).ok_or_else(|| invalid("IPC output capacity extent overflow"))?;
    let blocks = batches.len().checked_mul(4 * size_of::<arrow::ipc::Block>()).ok_or_else(|| invalid("IPC footer inventory extent overflow"))?;
    add(add(add(preparation, output)?, maximum)?, blocks)
}

/// Refuse oversized visible buffers/schema before the native writer allocates.
/// # Errors
/// Empty/incompatible inventory, checked extent or selected representation bound.
pub fn preflight(batches: &[RecordBatch], byte_limit: usize) -> Result<(), ArrowError> {
    let first = batches
        .first()
        .ok_or_else(|| invalid("empty file inventory"))?;
    if batches.len() > MAX_BATCHES {
        return Err(invalid("file batch count bound"));
    }
    let mut visible = crate::schema_working_extent(first.schema().as_ref())
        .map_err(|error| invalid(&error.to_string()))?;
    for batch in batches {
        if batch.schema() != first.schema() {
            return Err(invalid("incompatible packed schemas"));
        }
        for array in batch.columns() {
            visible = visible
                .checked_add(visible_size(&array.to_data(), &mut 0, byte_limit)?)
                .ok_or_else(|| invalid("file body size overflow"))?;
            if visible > byte_limit {
                return Err(invalid("file body exceeds bound"));
            }
        }
    }
    if visible > byte_limit {
        return Err(invalid("file body exceeds bound"));
    }
    Ok(())
}

#[cfg(test)]
std::thread_local! { static WRITER_STARTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

/// Finish an uncompressed native IPC file within the caller's physical bound.
/// The caller prepays writer scratch/output and establishes any stronger admission.
/// # Errors
/// Empty/incompatible inventory, Arrow failure, or a complete file exceeding the bound.
pub fn encode(batches: &[RecordBatch], byte_limit: usize) -> Result<Vec<u8>, ArrowError> {
    preflight(batches, byte_limit)?;
    let first = &batches[0];
    #[cfg(test)]
    WRITER_STARTS.with(|count| count.set(count.get() + 1));
    let mut writer = FileWriter::try_new(
        pse_ids::BoundedSink::new(Vec::new(), byte_limit),
        first.schema().as_ref(),
    )?;
    for batch in batches {
        writer.write(batch)?;
    }
    writer.finish()?;
    Ok(writer.into_inner()?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::{
        array::{Array, Float64Array},
        datatypes::{DataType, Field, Schema},
        ipc::reader::FileReader,
    };
    use std::{io::Cursor, sync::Arc};

    #[test]
    fn writer_extent_covers_rebased_offsets_shifted_bits_and_generated_validity() {
        use arrow::array::{BooleanArray, StringArray, StructArray};
        let strings = StringArray::from(vec![Some("parent prefix"), Some("abc"), None, Some("z")]);
        let bits = BooleanArray::from(vec![Some(true), None, Some(false), Some(true)]);
        let strings: Arc<dyn Array> = Arc::new(strings.slice(1, 3));
        let bits: Arc<dyn Array> = Arc::new(bits.slice(1, 3));
        let batch = RecordBatch::try_from_iter(vec![
            ("text", strings),
            ("bits", bits),
        ]).unwrap();
        let limit = 16384;
        let preparation = writer_preparation_extent(batch.schema().as_ref(), 1).unwrap();
        // Both arrays need validity; text needs a rebased offset buffer and
        // Boolean values start at a non-byte boundary. MutableBuffer rounds
        // each of these four independent scratch allocations to 64 bytes.
        assert_eq!(writer_extent(std::slice::from_ref(&batch), limit).unwrap(),
            preparation + 3 * limit + 4 * 64 + 4 * size_of::<arrow::ipc::Block>());
        let encoded = encode(std::slice::from_ref(&batch), limit).unwrap();
        assert_eq!(FileReader::try_new(Cursor::new(encoded), None).unwrap().next().unwrap().unwrap(), batch);

        // This has no input buffers, but the writer creates an all-valid
        // bitmap. A bound based only on input buffer capacity would miss it.
        let empty_struct: Arc<dyn Array> = Arc::new(StructArray::new_empty_fields(8192, None));
        let batch = RecordBatch::try_from_iter(vec![("struct", empty_struct)]).unwrap();
        let preparation = writer_preparation_extent(batch.schema().as_ref(), 1).unwrap();
        assert_eq!(writer_extent(std::slice::from_ref(&batch), limit).unwrap(),
            preparation + 3 * limit + 1024 + 4 * size_of::<arrow::ipc::Block>());
        assert!(encode(&[batch], limit).is_ok());
    }

    #[test]
    fn writer_extent_borrows_values_and_checks_capacity_overflow() {
        let schema = Arc::new(Schema::new(vec![Field::new("value", DataType::Float64, false)]));
        let values: Arc<dyn Array> = Arc::new(Float64Array::from(vec![1.0; 65536]));
        let large = RecordBatch::try_new(schema, vec![values]).unwrap();
        let small = large.slice(3, 8192);
        let limit = 1024 * 1024;
        let difference = writer_extent(std::slice::from_ref(&large), limit).unwrap()
            - writer_extent(std::slice::from_ref(&small), limit).unwrap();
        assert_eq!(difference, (65536 - 8192) / 8,
            "value bodies are borrowed; only generated validity grows");
        assert!(writer_extent(std::slice::from_ref(&small), usize::MAX).is_err());
        assert!(writer_preparation_extent(small.schema().as_ref(), usize::MAX).is_err());
        assert!(writer_extent(&[], limit).is_err());
    }

    #[test]
    fn writer_preflight_refuses_before_native_allocation_and_counts_selected_nested_slice() {
        use arrow::array::{Int32Array, ListArray};
        use arrow::datatypes::Int32Type;
        let nested = ListArray::from_iter_primitive::<Int32Type, _, _>([
            Some(vec![Some(1)]),
            Some(vec![Some(2); 65536]),
        ]);
        let slice = nested.slice(0, 1);
        let nested: Arc<dyn Array> = Arc::new(slice);
        let batch = RecordBatch::try_from_iter(vec![("nested", nested)]).unwrap();
        assert!(
            encode(&[batch], 16384).is_ok(),
            "a small nested slice does not pay for its full parent"
        );
        let values: Arc<dyn Array> = Arc::new(Int32Array::from(vec![1; 65536]));
        let large = RecordBatch::try_from_iter(vec![("values", values)]).unwrap();
        let before = WRITER_STARTS.with(std::cell::Cell::get);
        assert!(encode(&[large], 16384).is_err());
        let schema = Arc::new(Schema::new_with_metadata(
            Vec::<Field>::new(),
            [("large".into(), "x".repeat(65536))].into(),
        ));
        assert!(encode(&[RecordBatch::new_empty(schema)], 16384).is_err());
        assert_eq!(WRITER_STARTS.with(std::cell::Cell::get), before);
    }

    #[test]
    fn bounded_file_writer_preserves_bits_and_batch_positions() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Float64,
            true,
        )]));
        let bits = [
            0x8000_0000_0000_0000,
            0,
            0x7ff8_0000_0000_0042,
            1,
            f64::INFINITY.to_bits(),
        ];
        let values = bits
            .iter()
            .map(|bits| Some(f64::from_bits(*bits)))
            .chain([None])
            .collect::<Vec<_>>();
        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Float64Array::from(values))])
                .unwrap();
        let bytes = encode(&[batch.clone(), batch], 65536).unwrap();
        let mut reader = FileReader::try_new(Cursor::new(bytes), None).unwrap();
        reader.set_index(1).unwrap();
        let decoded = reader.next().unwrap().unwrap();
        let values = decoded
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.len(), bits.len() + 1);
        assert!(values.is_null(bits.len()));
        assert_eq!(
            values.values()[..bits.len()]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            bits
        );
    }

    #[test]
    fn bounded_file_writer_refuses_complete_file_overflow_and_incompatible_inventory() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Float64,
            true,
        )]));
        let first = RecordBatch::new_empty(schema);
        assert!(encode(&[], 4096).is_err());
        assert!(encode(std::slice::from_ref(&first), 8).is_err());
        let other = RecordBatch::new_empty(Arc::new(Schema::new(vec![Field::new(
            "other",
            DataType::Float64,
            true,
        )])));
        assert!(encode(&[first, other], 4096).is_err());
    }
}

/// A native batch with the inventory and decoder working grants that produced it.
#[derive(Debug)]
pub struct DecodedFileBatch {
    /// Native buffers retain their separately admitted input/copy allocations.
    pub batch: crate::owned_buffer::OwnedRecordBatch,
    /// Keep shared schema and per-batch control allocations with semantic wrappers.
    pub working_owner: std::sync::Arc<FileDecodeOwner>,
}
impl std::ops::Deref for DecodedFileBatch {
    type Target = RecordBatch;
    fn deref(&self) -> &Self::Target { self.batch.batch() }
}
/// Shared file schema/inventory and one batch's pre-acquired wrapper allowance.
#[derive(Debug)]
pub struct FileDecodeOwner {
    _inventory: std::sync::Arc<crate::AllocationLease>,
    _working: std::sync::Arc<crate::AllocationLease>,
}

/// One verified footer/block inventory and one native decoder for a protected
/// seekable source. The caller retains source custody and pays source backing;
/// this owner pays all newly constructed metadata, input backing and arrays.
pub struct AdmittedFile<R> {
    source: R,
    schema: arrow::datatypes::SchemaRef,
    blocks: Vec<arrow::ipc::Block>,
    fields: usize,
    decoder: Option<arrow::ipc::reader::FileDecoder>,
    inventory: std::sync::Arc<crate::AllocationLease>,
    pool: std::sync::Arc<dyn crate::MemoryPool>,
    limit: usize,
}
impl<R> std::fmt::Debug for AdmittedFile<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdmittedFile").field("batches", &self.blocks.len()).finish_non_exhaustive()
    }
}
fn read_io(error: std::io::Error) -> crate::CanonError { ArrowError::from(error).into() }

impl<R: std::io::Read + std::io::Seek> AdmittedFile<R> {
    /// Admit actual size/magic/footer/schema and nonoverlapping block spans once.
    /// Each selected record's layout is verified before its native decoder runs.
    /// # Errors
    /// Invalid representation/schema, cancellation, checked overflow or pool refusal.
    pub fn open(mut source: R, expected: arrow::datatypes::SchemaRef, limit: usize,
        pool: std::sync::Arc<dyn crate::MemoryPool>, cancel: &crate::CancellationToken,
    ) -> Result<Self, crate::CanonError> {
        use std::{io::SeekFrom, sync::Arc};
        use crate::ipc_stream::{add, mul, options, reserve, rounded};
        cancel.checkpoint()?;
        let control = reserve(&pool, 4096)?;
        let bytes = source.seek(SeekFrom::End(0)).map_err(read_io)?;
        if !(18..=limit as u64).contains(&bytes) { return Err(invalid("file byte bound").into()); }
        source.seek(SeekFrom::Start(0)).map_err(read_io)?;
        let mut header = [0; 8]; source.read_exact(&mut header).map_err(read_io)?;
        if &header != b"ARROW1\0\0" { return Err(invalid("file header").into()); }
        source.seek(SeekFrom::Start(bytes - 10)).map_err(read_io)?;
        let mut tail = [0; 10]; source.read_exact(&mut tail).map_err(read_io)?;
        if &tail[4..] != b"ARROW1" { return Err(invalid("file footer magic").into()); }
        let length = i32::from_le_bytes(tail[..4].try_into().map_err(|_| invalid("footer length"))?);
        let length = usize::try_from(length).map_err(|_| invalid("negative footer length"))?;
        if length > bytes as usize - 18 { return Err(invalid("footer outside actual file").into()); }
        let footer_start = bytes - 10 - length as u64;
        control.try_grow(rounded(length)?)?;
        cancel.checkpoint()?;
        let mut storage = arrow::buffer::MutableBuffer::from_len_zeroed(length);
        source.seek(SeekFrom::Start(footer_start)).map_err(read_io)?;
        source.read_exact(storage.as_slice_mut()).map_err(read_io)?;
        let footer = arrow::ipc::root_as_footer_with_opts(&options(limit), storage.as_slice())
            .map_err(|_| invalid("invalid file footer"))?;
        if footer.version() != arrow::ipc::MetadataVersion::V5
            || footer.dictionaries().is_some_and(|items| !items.is_empty()) {
            return Err(invalid("file version/dictionaries").into());
        }
        if let Some(entries) = footer.custom_metadata() {
            for entry in entries {
                if entry.key().is_none() || entry.value().is_none() { return Err(invalid("incomplete file custom metadata").into()); }
            }
        }
        let declaration = footer.schema().ok_or_else(|| invalid("file schema absent"))?;
        let (fields, schema_extent) = crate::ipc_stream::schema_extent(declaration)?;
        let entries = footer.recordBatches().ok_or_else(|| invalid("file batch inventory absent"))?;
        if entries.is_empty() || entries.len() > MAX_BATCHES { return Err(invalid("file batch count bound").into()); }
        let inventory_bytes = add(schema_extent, mul(entries.len(), size_of::<arrow::ipc::Block>())?)?;
        control.try_grow(inventory_bytes)?;
        cancel.checkpoint()?;
        let schema = Arc::new(arrow::ipc::convert::fb_to_schema(declaration));
        if schema != expected { return Err(invalid("registry schema mismatch").into()); }
        let mut blocks = Vec::with_capacity(entries.len());
        let mut previous = 8;
        for block in entries {
            cancel.checkpoint()?;
            let offset = u64::try_from(block.offset()).map_err(|_| invalid("negative file block offset"))?;
            let metadata = usize::try_from(block.metaDataLength()).map_err(|_| invalid("negative file metadata length"))?;
            let body = usize::try_from(block.bodyLength()).map_err(|_| invalid("negative file body length"))?;
            let end = offset.checked_add(add(metadata, body)? as u64).ok_or_else(|| invalid("file block overflow"))?;
            if offset < previous || metadata < 8 || metadata % 8 != 0 || body % 8 != 0 || end > footer_start {
                return Err(invalid("file block span/overlap").into());
            }
            previous = end;
            blocks.push(*block);
        }
        drop(storage);
        control.shrink(rounded(length)?);
        let decoder = arrow::ipc::reader::FileDecoder::new(schema.clone(), arrow::ipc::MetadataVersion::V5)
            .with_require_alignment(true);
        Ok(Self { source, schema, blocks, fields, decoder: Some(decoder), inventory: crate::AllocationLease::new(control),
            pool, limit })
    }
    /// Number of admitted batch locations; no record body has been materialized.
    pub fn len(&self) -> usize { self.blocks.len() }
    /// Whether this inventory has no batches (empty inventories are refused).
    pub fn is_empty(&self) -> bool { self.blocks.is_empty() }
    /// Representation schema; retain the reader or a decoded owner while using it.
    pub fn schema(&self) -> crate::Leased<arrow::datatypes::Schema> { crate::Leased::new(self.schema.clone(), self.inventory.clone()) }
    /// Check an exact footer-selected batch location without reading its body.
    /// # Errors
    /// Index outside the admitted inventory.
    pub fn check_index(&self, index: usize) -> Result<(), crate::CanonError> {
        self.blocks.get(index).map(|_| ()).ok_or_else(|| invalid("file batch index outside inventory").into())
    }
    /// Read one actual block once, then admit its complete message/layout before
    /// native decoding. Old cached files/slices retain their original pool grants.
    /// # Errors
    /// Malformed record, cancellation, I/O, checked overflow or resource refusal.
    pub fn read_batch(&mut self, index: usize, cancel: &crate::CancellationToken) -> Result<DecodedFileBatch, crate::CanonError> {
        use crate::ipc_stream::{add, mul, options, reserve, rounded};
        cancel.checkpoint()?;
        let block = *self.blocks.get(index).ok_or_else(|| invalid("file batch index outside inventory"))?;
        let metadata = usize::try_from(block.metaDataLength()).map_err(|_| invalid("file metadata length"))?;
        let body = usize::try_from(block.bodyLength()).map_err(|_| invalid("file body length"))?;
        let input = add(metadata, body)?;
        // 59.3 FileDecoder borrows the whole block. Native ArrayData/Arc/vector
        // headers and attachment rebuilds coexist: 4096 per node plus control.
        // Misalignment copies total <= body span + one 64-byte rounding per
        // possible buffer (at most three per node); metadata/node references fit
        // this same per-node header envelope. Compression is rejected below.
        let working = add(add(crate::allocation_extent::ipc_array_working_extent(self.fields)?, mul(self.fields, 3 * 64)?)?, rounded(body)?)?;
        let grant = reserve(&self.pool, add(rounded(input)?, working)?)?;
        cancel.checkpoint()?;
        let mut bytes = arrow::buffer::MutableBuffer::from_len_zeroed(input);
        self.source.seek(std::io::SeekFrom::Start(block.offset() as u64)).map_err(read_io)?;
        self.source.read_exact(bytes.as_slice_mut()).map_err(read_io)?;
        if bytes[..4] != [255; 4] { return Err(invalid("file block continuation").into()); }
        let length = i32::from_le_bytes(bytes[4..8].try_into().map_err(|_| invalid("file metadata length"))?);
        if usize::try_from(length).ok() != Some(metadata - 8) { return Err(invalid("file metadata extent").into()); }
        let message = arrow::ipc::root_as_message_with_opts(&options(self.limit), &bytes[8..metadata])
            .map_err(|_| invalid("invalid file record metadata"))?;
        if message.version() != arrow::ipc::MetadataVersion::V5 || usize::try_from(message.bodyLength()).ok() != Some(body) {
            return Err(invalid("file record version/body").into());
        }
        let record = message.header_as_record_batch().ok_or_else(|| invalid("file record required"))?;
        crate::ipc_stream::validate_record(record, self.schema.as_ref(), self.fields, body)?;
        let copies = crate::ipc_stream::alignment_copies(record, self.schema.as_ref(), &bytes[metadata..])?;
        if copies > working { return Err(invalid("file alignment copy envelope").into()); }
        cancel.checkpoint()?;
        // Input backing is unique to this decode. A per-batch scope avoids a
        // cached weak-owner inventory retaining dead entries after grants retire.
        let scope = crate::owned_buffer::AllocationScope::default();
        let buffer = scope.attach_buffer(bytes.into(), grant.split(rounded(input)?))?;
        let decoder = self.decoder.take().ok_or_else(|| invalid("file decoder absent"))?;
        self.decoder = Some(decoder.with_require_alignment(copies == 0));
        let decoded = self.decoder.as_ref().ok_or_else(|| invalid("file decoder absent"))?
            .read_record_batch(&block, &buffer)?.ok_or_else(|| invalid("file batch absent"))?;
        let batch = scope.attach_borrowed_reserved(decoded, &grant)?;
        let working_owner = std::sync::Arc::new(FileDecodeOwner {
            _inventory: self.inventory.clone(), _working: crate::AllocationLease::new(grant),
        });
        let batch = batch.with_payload_owner(working_owner.clone(), cancel)?;
        Ok(DecodedFileBatch { batch, working_owner })
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use crate::{CancellationToken, GreedyMemoryPool, MemoryPool};
    use arrow::{array::{Array, Float64Array}, datatypes::{DataType, Field, Schema}};
    use std::{io::{Cursor, Read, Seek, SeekFrom}, sync::{Arc, Mutex}};
    #[derive(Debug)]
    struct Observed {
        source: Cursor<Vec<u8>>,
        pool: Arc<dyn MemoryPool>,
        reads: Arc<Mutex<Vec<(u64, usize, usize)>>>,
    }
    impl Read for Observed {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            let reserved = self.pool.reserved();
            assert!(reserved >= output.len(), "source materialization must be prepaid");
            self.reads.lock().unwrap().push((self.source.position(), output.len(), reserved));
            self.source.read(output)
        }
    }
    impl Seek for Observed {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> { self.source.seek(position) }
    }
    fn schema() -> Arc<Schema> { Arc::new(Schema::new(vec![Field::new("value", DataType::Float64, false)])) }
    fn batch(schema: Arc<Schema>, rows: usize) -> RecordBatch {
        RecordBatch::try_new(schema, vec![Arc::new(Float64Array::from(vec![-0.; rows]))]).unwrap()
    }
    #[test]
    fn selected_small_block_in_maximum_size_file_uses_one_footer_and_prepays_before_reads() {
        const LIMIT: usize = 32 << 20;
        let schema = schema();
        let mut bytes = encode(&[batch(schema.clone(), (LIMIT - (1 << 20)) / 9), batch(schema.clone(), 3)], LIMIT).unwrap();
        let length = bytes.len();
        let footer_length = i32::from_le_bytes(bytes[length - 10..length - 6].try_into().unwrap()) as usize;
        let insertion = length - 10 - footer_length;
        bytes.splice(insertion..insertion, std::iter::repeat_n(0, LIMIT - length));
        assert_eq!(bytes.len(), LIMIT);
        let footer_start = LIMIT - 10 - footer_length;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 << 10));
        let reads = Arc::new(Mutex::new(Vec::new()));
        let observed = Observed { source: Cursor::new(bytes), pool: pool.clone(), reads: reads.clone() };
        let cancel = CancellationToken::new();
        let mut reader = AdmittedFile::open(observed, schema, LIMIT, pool.clone(), &cancel).unwrap();
        let after_footer = reads.lock().unwrap().len();
        let small = reader.read_batch(1, &cancel).unwrap();
        assert_eq!(small.num_rows(), 3);
        assert_eq!(reads.lock().unwrap().iter().filter(|(position, _, _)| *position == footer_start as u64).count(), 1);
        assert_eq!(reads.lock().unwrap().len(), after_footer + 1, "one complete input read per selected block");
        let before_refusal = reads.lock().unwrap().len();
        assert!(matches!(reader.read_batch(0, &cancel), Err(crate::CanonError::NativeResource(_))));
        assert_eq!(reads.lock().unwrap().len(), before_refusal, "refusal precedes reading or allocating the large input");
        drop(small); drop(reader); assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn escaped_slices_share_input_and_schema_working_leases_until_the_last_slice() {
        let schema = schema();
        let bytes = encode(&[batch(schema.clone(), 4)], 65536).unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 << 10));
        let cancel = CancellationToken::new();
        let mut reader = AdmittedFile::open(Cursor::new(&bytes), schema, 65536, pool.clone(), &cancel).unwrap();
        let decoded = reader.read_batch(0, &cancel).unwrap();
        let first = decoded.column(0).slice(0, 1);
        let second = decoded.column(0).slice(1, 1);
        let charged = pool.reserved();
        let first_buffer = first.to_data().buffers()[0].clone();
        let second_buffer = second.to_data().buffers()[0].clone();
        assert_eq!(first_buffer.as_ptr().wrapping_add(8), second_buffer.as_ptr());
        drop(first_buffer); drop(second_buffer);
        drop(decoded); drop(reader); drop(bytes);
        assert_eq!(pool.reserved(), charged, "escaped arrays keep schema and wrapper leases too");
        drop(first); assert_eq!(pool.reserved(), charged);
        drop(second); assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn leased_schema_and_metadata_heavy_footer_keep_original_allocation_admission() {
        let schema = Arc::new(Schema::new_with_metadata(schema().fields().clone(), [("large".into(), "x".repeat(1 << 20))].into()));
        let bytes = encode(&[batch(schema.clone(), 1)], 4 << 20).unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8 << 20));
        let reader = AdmittedFile::open(Cursor::new(bytes), schema, 4 << 20, pool.clone(), &CancellationToken::new()).unwrap();
        let leased = reader.schema();
        let charged = pool.reserved(); drop(reader);
        assert_eq!(leased.metadata()["large"].len(), 1 << 20);
        assert_eq!(pool.reserved(), charged);
        drop(leased); assert_eq!(pool.reserved(), 0);
    }
}
