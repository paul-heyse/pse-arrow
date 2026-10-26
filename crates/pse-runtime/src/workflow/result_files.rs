// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completed native IPC files. Physical integrity and registry admission have
//! separate owners; this codec checks representation before Arrow allocates.

use super::result_blocks::{self, ResultBlockError};
use datafusion::arrow::{
    datatypes::SchemaRef,
    ipc::{
        self,
        reader::{FileReader, FileReaderBuilder},
    },
    record_batch::RecordBatch,
};
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Native IPC reads retain either the protected physical owner or the charged inline bytes.
#[derive(Debug)]
pub(super) enum ResultFileSource {
    Sealed(pse_operations::canonical_files::ProtectedFile),
    Inline(Cursor<Vec<u8>>),
}
impl Read for ResultFileSource {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Sealed(file) => file.read(bytes),
            Self::Inline(body) => body.read(bytes),
        }
    }
}
impl Seek for ResultFileSource {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Sealed(file) => file.seek(position),
            Self::Inline(body) => body.seek(position),
        }
    }
}

/// Default packing target; acknowledgement boundaries can seal smaller files.
pub const TARGET_FILE_BYTES: usize = pse_operations::canonical_files::FILE_TARGET_BYTES;
/// Every complete file, including its footer, fits this bound.
pub const MAX_FILE_BYTES: usize = pse_operations::canonical_files::FILE_BYTES;
const MAX_TABLES: usize = 128 * 1024;
const MAX_BATCHES: usize = pse_columnar::ipc_file::MAX_BATCHES;
#[cfg(test)]
std::thread_local! {
    static PREFLIGHT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Pack compatible logical batches into one uncompressed, finished native file.
/// Callers reserve writer scratch and output before calling, and seal before ACK.
pub fn encode_file_batches(batches: &[RecordBatch]) -> Result<Vec<u8>, ResultBlockError> {
    check_file_batches(batches)?;
    Ok(pse_columnar::ipc_file::encode(batches, MAX_FILE_BYTES)?)
}

/// Shared representation checks for raw and admitted scientific file producers.
pub(super) fn check_file_batches(batches: &[RecordBatch]) -> Result<(), ResultBlockError> {
    let first = batches
        .first()
        .ok_or(ResultBlockError::Invalid("empty file inventory"))?;
    if batches.len() > MAX_BATCHES {
        return Err(ResultBlockError::Invalid("file batch count bound"));
    }
    result_blocks::expected_shape(first.schema().as_ref())?;
    let mut visible = 0usize;
    for batch in batches {
        if batch.schema() != first.schema() {
            return Err(ResultBlockError::Invalid("incompatible packed schemas"));
        }
        for array in batch.columns() {
            // The counter is local to this array: the physical byte/file bound
            // controls packing, independently of old RPC row fragmentation.
            visible = visible
                .checked_add(result_blocks::visible_size(&array.to_data(), &mut 0)?)
                .ok_or(ResultBlockError::Invalid("file body size overflow"))?;
        }
    }
    if visible > MAX_FILE_BYTES {
        return Err(ResultBlockError::Invalid("file body exceeds bound"));
    }
    pse_columnar::ipc_file::preflight(batches, MAX_FILE_BYTES)?;
    Ok(())
}

/// One complete file, useful for independent artifact members and fixtures.
pub fn encode_result_file(batch: &RecordBatch) -> Result<Vec<u8>, ResultBlockError> {
    encode_file_batches(std::slice::from_ref(batch))
}

/// Select a contiguous useful prefix by actual complete-file size. A single
/// large row may exceed the target, but never the hard bound.
pub(super) fn encode_result_prefix(
    batch: &RecordBatch,
    start: usize,
    count: usize,
) -> Result<(Vec<u8>, usize), ResultBlockError> {
    if count == 0
        || start
            .checked_add(count)
            .is_none_or(|end| end > batch.num_rows())
    {
        return Err(ResultBlockError::Invalid("invalid file prefix"));
    }
    let mut rows = count;
    loop {
        match encode_result_file(&batch.slice(start, rows)) {
            Ok(bytes) if bytes.len() <= TARGET_FILE_BYTES || rows == 1 => return Ok((bytes, rows)),
            Ok(_) => rows = rows.div_ceil(2),
            Err(error) if rows > 1 && splittable(&error) => rows = rows.div_ceil(2),
            Err(error) => return Err(error),
        }
    }
}

/// Plan a useful contiguous slice without encoding an intermediate file. The
/// final packer enforces its complete physical bound with LimitedWriter.
pub(super) fn result_prefix_rows(
    batch: &RecordBatch,
    start: usize,
    count: usize,
) -> Result<usize, ResultBlockError> {
    if count == 0
        || start
            .checked_add(count)
            .is_none_or(|end| end > batch.num_rows())
    {
        return Err(invalid("invalid file prefix"));
    }
    let mut rows = count;
    loop {
        let slice = batch.slice(start, rows);
        match batch_extent(&slice) {
            Ok(bytes) if bytes <= TARGET_FILE_BYTES || rows == 1 && bytes <= MAX_FILE_BYTES => {
                return Ok(rows);
            }
            Ok(_) if rows > 1 => rows = rows.div_ceil(2),
            Ok(_) => return Err(invalid("file body exceeds bound")),
            Err(error) if rows > 1 && splittable(&error) => rows = rows.div_ceil(2),
            Err(error) => return Err(error),
        }
    }
}

pub(super) fn batch_extent(batch: &RecordBatch) -> Result<usize, ResultBlockError> {
    batch
        .columns()
        .iter()
        .try_fold(4096usize + batch.num_columns() * 256, |bytes, array| {
            bytes
                .checked_add(result_blocks::visible_size(&array.to_data(), &mut 0)?)
                .ok_or_else(|| invalid("file body size overflow"))
        })
}

pub(super) fn schema_extent(schema: &SchemaRef) -> Result<usize, ResultBlockError> {
    let fields = schema.fields().iter().try_fold(0usize, |bytes, field| {
        bytes
            .checked_add(field.size())
            .ok_or_else(|| invalid("file schema size overflow"))
    })?;
    schema
        .metadata()
        .iter()
        .try_fold(fields, |bytes, (key, value)| {
            bytes
                .checked_add(key.capacity())
                .and_then(|bytes| bytes.checked_add(value.capacity()))
                .and_then(|bytes| bytes.checked_add(128))
                .ok_or_else(|| invalid("file schema metadata overflow"))
        })
}
fn splittable(error: &ResultBlockError) -> bool {
    matches!(
        error,
        ResultBlockError::Invalid("file body exceeds bound" | "field value bound")
    ) || matches!(error, ResultBlockError::Arrow(datafusion::arrow::error::ArrowError::IoError(message, _)) if message == "sealed result file limit")
}

fn invalid(message: &'static str) -> ResultBlockError {
    ResultBlockError::Invalid(message)
}
fn read_range<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    bytes: usize,
) -> Result<Vec<u8>, ResultBlockError> {
    reader
        .seek(SeekFrom::Start(start))
        .map_err(datafusion::arrow::error::ArrowError::from)?;
    let mut buffer = vec![0; bytes];
    reader
        .read_exact(&mut buffer)
        .map_err(datafusion::arrow::error::ArrowError::from)?;
    Ok(buffer)
}
fn options() -> flatbuffers::VerifierOptions {
    flatbuffers::VerifierOptions {
        max_depth: 256,
        max_tables: MAX_TABLES,
        max_apparent_size: MAX_FILE_BYTES,
        ..Default::default()
    }
}

/// Check actual file/footer/record extents before FileReader's allocations.
/// In particular a corrupt final footer-length word cannot allocate gigabytes.
fn preflight<R: Read + Seek>(
    reader: &mut R,
    expected: &SchemaRef,
) -> Result<usize, ResultBlockError> {
    #[cfg(test)]
    PREFLIGHT_CALLS.with(|count| count.set(count.get() + 1));
    let bytes = reader
        .seek(SeekFrom::End(0))
        .map_err(datafusion::arrow::error::ArrowError::from)?;
    if !(18..=MAX_FILE_BYTES as u64).contains(&bytes) {
        return Err(invalid("file byte bound"));
    }
    if read_range(reader, 0, 8)?.as_slice() != b"ARROW1\0\0" {
        return Err(invalid("file header"));
    }
    let tail = read_range(reader, bytes - 10, 10)?;
    if &tail[4..] != b"ARROW1" {
        return Err(invalid("file footer magic"));
    }
    let length = i32::from_le_bytes(tail[..4].try_into().map_err(|_| invalid("footer length"))?);
    let length = usize::try_from(length).map_err(|_| invalid("negative footer length"))?;
    if length > bytes as usize - 18 {
        return Err(invalid("footer outside actual file"));
    }
    let footer_start = bytes - 10 - length as u64;
    let footer_bytes = read_range(reader, footer_start, length)?;
    let footer = ipc::root_as_footer_with_opts(&options(), &footer_bytes)
        .map_err(|_| invalid("invalid file footer"))?;
    if let Some(metadata) = footer.custom_metadata() {
        for item in metadata {
            if item.key().is_none() || item.value().is_none() {
                return Err(invalid("incomplete file custom metadata"));
            }
        }
    }
    if footer.version() != ipc::MetadataVersion::V5
        || footer
            .dictionaries()
            .is_some_and(|entries| !entries.is_empty())
    {
        return Err(invalid("file version/dictionaries"));
    }
    let schema = footer
        .schema()
        .ok_or_else(|| invalid("file schema absent"))?;
    let shape = result_blocks::expected_shape(expected.as_ref())?;
    let fields = schema
        .fields()
        .ok_or_else(|| invalid("file schema fields absent"))?;
    if schema.endianness() != ipc::Endianness::Little || fields.len() != expected.fields().len() {
        return Err(invalid("file schema shape/endianness"));
    }
    let mut count = 0;
    for field in fields {
        result_blocks::schema_field(field, 0, &mut count, shape)?;
    }
    if count != shape || ipc::convert::fb_to_schema(schema) != **expected {
        return Err(invalid("registry schema mismatch"));
    }
    let blocks = footer
        .recordBatches()
        .ok_or_else(|| invalid("file batch inventory absent"))?;
    if blocks.is_empty() || blocks.len() > MAX_BATCHES {
        return Err(invalid("file batch count bound"));
    }
    let mut last_end = 8u64;
    for block in blocks {
        let offset =
            u64::try_from(block.offset()).map_err(|_| invalid("negative file block offset"))?;
        let metadata = usize::try_from(block.metaDataLength())
            .map_err(|_| invalid("negative file metadata length"))?;
        let body =
            u64::try_from(block.bodyLength()).map_err(|_| invalid("negative file body length"))?;
        let end = offset
            .checked_add(metadata as u64)
            .and_then(|end| end.checked_add(body))
            .ok_or_else(|| invalid("file block overflow"))?;
        if offset < last_end
            || metadata < 8
            || end > footer_start
            || metadata % 8 != 0
            || body % 8 != 0
        {
            return Err(invalid("file block span/overlap"));
        }
        last_end = end;
        let metadata_bytes = read_range(reader, offset, metadata)?;
        if metadata_bytes[..4] != [255; 4] {
            return Err(invalid("file block continuation"));
        }
        let message_length = i32::from_le_bytes(
            metadata_bytes[4..8]
                .try_into()
                .map_err(|_| invalid("file metadata length"))?,
        );
        if usize::try_from(message_length).ok() != Some(metadata - 8) {
            return Err(invalid("file metadata extent"));
        }
        let message = ipc::root_as_message_with_opts(&options(), &metadata_bytes[8..])
            .map_err(|_| invalid("invalid file record metadata"))?;
        if message.version() != ipc::MetadataVersion::V5
            || u64::try_from(message.bodyLength()).ok() != Some(body)
        {
            return Err(invalid("file record version/body"));
        }
        let batch = message
            .header_as_record_batch()
            .ok_or_else(|| invalid("file record required"))?;
        if batch.compression().is_some() || batch.variadicBufferCounts().is_some() {
            return Err(invalid("compression/variadic buffers refused"));
        }
        let rows =
            usize::try_from(batch.length()).map_err(|_| invalid("negative file row count"))?;
        let nodes = batch
            .nodes()
            .ok_or_else(|| invalid("file field nodes absent"))?;
        let buffers = batch
            .buffers()
            .ok_or_else(|| invalid("file buffers absent"))?;
        if nodes.len() != shape || buffers.len() > shape * 3 {
            return Err(invalid("file node/buffer shape"));
        }
        let mut values = 0usize;
        for node in nodes {
            let n = usize::try_from(node.length())
                .map_err(|_| invalid("negative file field length"))?;
            let nulls = usize::try_from(node.null_count())
                .map_err(|_| invalid("negative file null count"))?;
            values = values
                .checked_add(n)
                .ok_or_else(|| invalid("file field size overflow"))?;
            if values > MAX_FILE_BYTES || nulls > n {
                return Err(invalid("file field value bound"));
            }
        }
        let mut previous = 0;
        for buffer in buffers {
            let start = u64::try_from(buffer.offset())
                .map_err(|_| invalid("negative file buffer offset"))?;
            let length = u64::try_from(buffer.length())
                .map_err(|_| invalid("negative file buffer length"))?;
            let end = start
                .checked_add(length)
                .ok_or_else(|| invalid("file buffer overflow"))?;
            if start < previous || end > body {
                return Err(invalid("file buffer span/overlap"));
            }
            previous = end;
        }
        let nodes = nodes.iter().collect::<Vec<_>>();
        let buffers = buffers.iter().collect::<Vec<_>>();
        let (mut node, mut buffer) = (0, 0);
        for field in expected.fields() {
            result_blocks::layout(
                field.data_type(),
                &nodes,
                &buffers,
                &mut node,
                &mut buffer,
                Some(rows),
            )?;
        }
        if node != nodes.len() || buffer != buffers.len() {
            return Err(invalid("extra file layout nodes/buffers"));
        }
    }
    Ok(blocks.len())
}

pub(super) fn open_file_reader<R: Read + Seek>(
    mut reader: R,
    expected: &SchemaRef,
    count: Option<usize>,
) -> Result<FileReader<R>, ResultBlockError> {
    let actual = preflight(&mut reader, expected)?;
    if count.is_some_and(|count| count != actual) {
        return Err(invalid("sealed file batch count differs"));
    }
    Ok(FileReaderBuilder::new()
        .with_max_footer_fb_tables(MAX_TABLES)
        .with_max_footer_fb_depth(256)
        .build(reader)?)
}

/// Decode a completed standalone file member with the exact declared schema.
pub fn decode_result_file(
    bytes: &[u8],
    expected: SchemaRef,
    rows: usize,
) -> Result<RecordBatch, ResultBlockError> {
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("canonical.result.standalone_decodes", "count", 1);
    }
    let mut reader = open_file_reader(Cursor::new(bytes), &expected, Some(1))?;
    let batch = reader
        .next()
        .transpose()?
        .ok_or_else(|| invalid("file batch absent"))?;
    if batch.num_rows() != rows || batch.schema() != expected {
        return Err(invalid("file batch rows/schema differ"));
    }
    Ok(batch)
}
/// Backpressured file ingestion. Every visitor receives a completed native file;
/// no unsealed prefix or unrelated future producer is needed for its ACK.
pub async fn visit_result_files_async<F, Fut, E>(
    batch: &RecordBatch,
    mut visitor: F,
) -> Result<(), E>
where
    F: FnMut(usize, usize, Vec<u8>) -> Fut,
    Fut: Future<Output = Result<(), E>>,
    E: From<ResultBlockError>,
{
    if batch.num_rows() == 0 {
        return visitor(0, 0, encode_result_file(batch)?).await;
    }
    let mut start = 0;
    while start < batch.num_rows() {
        let (bytes, rows) = encode_result_prefix(batch, start, batch.num_rows() - start)?;
        visitor(start, rows, bytes).await?;
        start += rows;
    }
    Ok(())
}

#[cfg(test)]
mod result_files_unit {
    use super::*;
    use datafusion::arrow::{
        array::{Array, Float64Array, ListArray, types::Float64Type},
        datatypes::{DataType, Field, Schema},
    };
    use std::{collections::HashMap, sync::Arc};

    fn batch(bits: &[u64]) -> RecordBatch {
        let schema = Arc::new(Schema::new_with_metadata(
            vec![
                Field::new("value", DataType::Float64, false)
                    .with_metadata(HashMap::from([("units".into(), "m".into())])),
            ],
            HashMap::from([("interpretation".into(), "exact".into())]),
        ));
        RecordBatch::try_new(
            schema,
            vec![Arc::new(Float64Array::from_iter_values(
                bits.iter().copied().map(f64::from_bits),
            ))],
        )
        .unwrap()
    }

    #[test]
    fn completed_native_file_packs_batches_and_random_access_preserves_ieee_metadata() {
        let first = batch(&[0, 1 << 63, 0x7ff8_0000_0000_0042]);
        let second = batch(&[0x7ff0_0000_0000_0000, 0xfff0_0000_0000_0000]);
        let bytes = encode_file_batches(&[first.clone(), second.clone()]).unwrap();
        assert_eq!(&bytes[..6], b"ARROW1");
        assert_eq!(&bytes[bytes.len() - 6..], b"ARROW1");
        for (index, expected) in [first, second].iter().enumerate() {
            let mut reader =
                open_file_reader(Cursor::new(&bytes), &expected.schema(), Some(2)).unwrap();
            reader.set_index(index).unwrap();
            let actual = reader.next().unwrap().unwrap();
            assert_eq!(actual.num_rows(), expected.num_rows());
            assert_eq!(actual.schema(), expected.schema());
            let actual = actual
                .column(0)
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            let expected = expected
                .column(0)
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            assert_eq!(
                actual
                    .values()
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                expected
                    .values()
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
        }
        let mut reader =
            open_file_reader(Cursor::new(&bytes), &batch(&[]).schema(), Some(2)).unwrap();
        assert!(reader.set_index(2).is_err());
    }

    #[test]
    fn standalone_decode_preflights_once_and_preserves_ieee_payloads() {
        let bits = [0, 1 << 63, 0x7ff8_0000_0000_0042, 0x7ff0_0000_0000_0000];
        let source = batch(&bits);
        let bytes = encode_result_file(&source).unwrap();
        PREFLIGHT_CALLS.with(|count| count.set(0));
        let decoded = decode_result_file(&bytes, source.schema(), bits.len()).unwrap();
        PREFLIGHT_CALLS.with(|count| assert_eq!(count.get(), 1));
        let values = decoded
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(
            values
                .values()
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            bits
        );
        let packed = encode_file_batches(&[source.clone(), source.clone()]).unwrap();
        PREFLIGHT_CALLS.with(|count| count.set(0));
        assert!(decode_result_file(&packed, source.schema(), bits.len()).is_err());
        PREFLIGHT_CALLS.with(|count| assert_eq!(count.get(), 1));
    }

    #[test]
    fn file_has_no_obsolete_rpc_row_limit_and_preserves_nested_nulls() {
        let source = batch(&vec![42; 65_537]);
        let bytes = encode_result_file(&source).unwrap();
        assert_eq!(
            decode_result_file(&bytes, source.schema(), 65_537)
                .unwrap()
                .num_rows(),
            65_537
        );
        let list = ListArray::from_iter_primitive::<Float64Type, _, _>([
            Some(vec![Some(-0.0), None]),
            None,
            Some(vec![]),
            Some(vec![Some(f64::from_bits(0x7ff8_0000_0000_0042))]),
        ]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            "nested",
            list.data_type().clone(),
            true,
        )]));
        let source = RecordBatch::try_new(schema, vec![Arc::new(list)]).unwrap();
        let bytes = encode_result_file(&source).unwrap();
        let actual = decode_result_file(&bytes, source.schema(), 4).unwrap();
        assert_eq!(actual.column(0).to_data(), source.column(0).to_data());
    }

    #[test]
    fn corrupt_footer_length_is_refused_before_an_advertised_allocation_or_read() {
        struct Observed(Cursor<Vec<u8>>, usize);
        impl Read for Observed {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                self.1 = self.1.max(out.len());
                assert!(out.len() <= MAX_FILE_BYTES);
                self.0.read(out)
            }
        }
        impl Seek for Observed {
            fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
                self.0.seek(position)
            }
        }
        let source = batch(&[1, 2]);
        let mut bytes = encode_result_file(&source).unwrap();
        let length = bytes.len();
        bytes[length - 10..length - 6].copy_from_slice(&i32::MAX.to_le_bytes());
        let mut reader = Observed(Cursor::new(bytes), 0);
        assert!(matches!(
            preflight(&mut reader, &source.schema()),
            Err(ResultBlockError::Invalid("footer outside actual file"))
        ));
        assert_eq!(reader.1, 10);
    }

    #[test]
    fn missing_footer_wrong_schema_rows_and_incompatible_packing_fail() {
        let source = batch(&[1, 2]);
        let bytes = encode_result_file(&source).unwrap();
        assert!(decode_result_file(&bytes[..bytes.len() - 1], source.schema(), 2).is_err());
        assert!(decode_result_file(&bytes, source.schema(), 3).is_err());
        let wrong = Arc::new(Schema::new(vec![Field::new(
            "other",
            DataType::Float64,
            false,
        )]));
        assert!(decode_result_file(&bytes, wrong.clone(), 2).is_err());
        let packed = encode_file_batches(&[source.clone(), source.clone()]).unwrap();
        assert!(decode_result_file(&packed, source.schema(), 2).is_err());
        let second = RecordBatch::try_new(wrong, source.columns().to_vec()).unwrap();
        assert!(encode_file_batches(&[source, second]).is_err());
    }

    #[test]
    fn copied_native_file_arrays_retain_actual_allocation_after_reader_and_scope_drop() {
        use pse_columnar::{MemoryConsumer, MemoryPool, owned_buffer::AllocationScope};
        let source = batch(&[0, 1 << 63, 0x7ff8_0000_0000_0042]);
        let bytes = encode_result_file(&source).unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let grant = MemoryConsumer::new("file:test-owned-decode").register(&pool);
        grant.try_grow(bytes.len() * 4 + 64 * 1024).unwrap();
        let decoded = decode_result_file(&bytes, source.schema(), 3).unwrap();
        let scope = AllocationScope::default();
        let owned = scope.attach_borrowed_reserved(decoded, &grant).unwrap();
        let batch = owned.into_batch();
        let escaped = batch.column(0).slice(1, 2);
        drop(batch);
        drop(scope);
        drop(grant);
        drop(bytes);
        assert!(
            pool.reserved() > 0,
            "escaped copied native buffers keep their actual grant"
        );
        let values = escaped.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.value(0).to_bits(), 1 << 63);
        assert_eq!(values.value(1).to_bits(), 0x7ff8_0000_0000_0042);
        drop(escaped);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn packing_targets_complete_file_bytes_and_refuses_an_oversized_single_row() {
        use datafusion::arrow::array::BinaryArray;
        let source = batch(&vec![0; 3_000_000]);
        let (bytes, rows) = encode_result_prefix(&source, 0, source.num_rows()).unwrap();
        assert!(rows > 32_768 && rows < source.num_rows());
        assert!(bytes.len() <= TARGET_FILE_BYTES);
        assert_eq!(
            decode_result_file(&bytes, source.schema(), rows)
                .unwrap()
                .num_rows(),
            rows
        );
        let value = vec![0u8; MAX_FILE_BYTES];
        let array = BinaryArray::from_iter_values([value.as_slice()]);
        let schema = Arc::new(Schema::new(vec![Field::new(
            "large",
            DataType::Binary,
            false,
        )]));
        let source = RecordBatch::try_new(schema, vec![Arc::new(array)]).unwrap();
        assert!(
            encode_result_prefix(&source, 0, 1).is_err(),
            "footer and offsets also fit the hard file bound"
        );
    }
}
