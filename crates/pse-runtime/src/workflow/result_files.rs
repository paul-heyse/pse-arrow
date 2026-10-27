// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completed native IPC files. Physical integrity and registry admission have
//! separate owners; this codec checks representation before Arrow allocates.

use super::result_blocks::{self, ResultBlockError};
use datafusion::arrow::{datatypes::SchemaRef, record_batch::RecordBatch};
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Native IPC reads retain either the protected physical owner or the charged inline bytes.
#[derive(Debug)]
pub(super) enum ResultFileSource {
    Sealed(Box<pse_operations::canonical_files::ProtectedFile>),
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
        return Err(ResultBlockError::LimitReached("file body exceeds bound"));
    }
    pse_columnar::ipc_file::preflight(batches, MAX_FILE_BYTES)?;
    Ok(())
}

/// One complete file, useful for independent artifact members and fixtures.
pub fn encode_result_file(batch: &RecordBatch) -> Result<Vec<u8>, ResultBlockError> {
    encode_file_batches(std::slice::from_ref(batch))
}

/// Plan a useful contiguous slice without encoding an intermediate file. The
/// final packer enforces its complete physical bound with the shared typed bounded sink.
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
            Ok(_) => return Err(ResultBlockError::LimitReached("file body exceeds bound")),
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

fn splittable(error: &ResultBlockError) -> bool {
    matches!(error, ResultBlockError::LimitReached(_))
        || matches!(
            pse_ids::sink_failure(error),
            Some(pse_ids::SinkFailure::LimitReached { .. })
        )
        || pse_ids::error_payload::<pse_columnar::ipc_file::ValueLimit>(error).is_some()
}

fn invalid(message: &'static str) -> ResultBlockError {
    ResultBlockError::Invalid(message)
}
/// One columnar-owned admitted footer/schema/block inventory, with no stock
/// FileReader or second footer read. Registry equality is selected by this caller.
pub(super) fn open_file_reader<R: Read + Seek>(
    reader: R,
    expected: &SchemaRef,
    count: Option<usize>,
    pool: std::sync::Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<pse_columnar::ipc_file::AdmittedFile<R>, ResultBlockError> {
    #[cfg(test)]
    PREFLIGHT_CALLS.with(|count| count.set(count.get() + 1));
    let file = pse_columnar::ipc_file::AdmittedFile::open(
        reader,
        expected.clone(),
        MAX_FILE_BYTES,
        pool,
        cancel,
    )?;
    if count.is_some_and(|count| count != file.len()) {
        return Err(invalid("sealed file batch count differs"));
    }
    Ok(file)
}

/// Decode a completed standalone file member with the exact declared schema.
pub fn decode_result_file(
    bytes: &[u8],
    expected: SchemaRef,
    rows: usize,
    pool: std::sync::Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<pse_columnar::ipc_file::DecodedFileBatch, ResultBlockError> {
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("canonical.result.standalone_decodes", "count", 1);
    }
    let mut reader = open_file_reader(Cursor::new(bytes), &expected, Some(1), pool, cancel)?;
    let batch = reader.read_batch(0, cancel)?;
    if batch.num_rows() != rows || batch.schema() != expected {
        return Err(invalid("file batch rows/schema differ"));
    }
    Ok(batch)
}
#[cfg(test)]
mod result_files_unit {
    use super::*;
    use datafusion::arrow::{
        array::{Array, Float64Array, ListArray, types::Float64Type},
        datatypes::{DataType, Field, Schema},
    };
    use std::{collections::HashMap, sync::Arc};

    fn fixture_pool() -> Arc<dyn pse_columnar::MemoryPool> {
        Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20))
    }
    fn open_file_reader<R: Read + Seek>(
        reader: R,
        schema: &SchemaRef,
        count: Option<usize>,
    ) -> Result<pse_columnar::ipc_file::AdmittedFile<R>, ResultBlockError> {
        super::open_file_reader(
            reader,
            schema,
            count,
            fixture_pool(),
            &pse_columnar::CancellationToken::new(),
        )
    }
    fn decode_result_file(
        bytes: &[u8],
        schema: SchemaRef,
        rows: usize,
    ) -> Result<pse_columnar::ipc_file::DecodedFileBatch, ResultBlockError> {
        super::decode_result_file(
            bytes,
            schema,
            rows,
            fixture_pool(),
            &pse_columnar::CancellationToken::new(),
        )
    }
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
            let actual = reader
                .read_batch(index, &pse_columnar::CancellationToken::new())
                .unwrap();
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
        let reader = open_file_reader(Cursor::new(&bytes), &batch(&[]).schema(), Some(2)).unwrap();
        assert!(reader.check_index(2).is_err());
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
        let error = open_file_reader(&mut reader, &source.schema(), None)
            .err()
            .expect("corrupt footer refused");
        assert!(error.to_string().contains("footer outside actual file"));
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
        use pse_columnar::MemoryPool;
        let source = batch(&[0, 1 << 63, 0x7ff8_0000_0000_0042]);
        let bytes = encode_result_file(&source).unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let batch = super::decode_result_file(
            &bytes,
            source.schema(),
            3,
            pool.clone(),
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
        let escaped = batch.column(0).slice(1, 2);
        drop(batch);
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
}
