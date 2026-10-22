// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One bounded native file encoder. Encoding establishes representation only;
//! callers own relation admission, allocation prepayment and statistics authority.

use arrow::{array::ArrayData, datatypes::DataType, error::ArrowError, ipc::writer::FileWriter, record_batch::RecordBatch};
use std::io::Write;

/// Existing physical-file record batch ceiling, also used by protected readers.
pub const MAX_BATCHES: usize = 32 * 1024;

struct LimitedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes.len().checked_add(bytes.len()).is_none_or(|end| end > self.limit) {
            return Err(std::io::Error::other("sealed result file limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn invalid(message: &str) -> ArrowError { ArrowError::InvalidArgumentError(message.into()) }

/// Count only the visible native slice, including selected nested children.
/// This is a representation preflight; it grants no relation or statistics authority.
/// # Errors
/// Overflow, unsupported native extent or a caller-selected value-slot bound.
pub fn visible_size(data: &ArrayData, values: &mut usize, value_limit: usize) -> Result<usize, ArrowError> {
    *values = values.checked_add(data.len()).ok_or_else(|| invalid("field size overflow"))?;
    if *values > value_limit { return Err(invalid("field value bound")); }
    let bitmap = if data.nulls().is_some() { data.len().div_ceil(8) } else { 0 };
    let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(|| invalid("visible size overflow"));
    let list = |start: usize, end: usize, width: usize, values: &mut usize| -> Result<usize, ArrowError> {
        let offsets = data.len().checked_add(1).and_then(|n| n.checked_mul(width)).ok_or_else(|| invalid("visible size overflow"))?;
        let length = end.checked_sub(start).ok_or_else(|| invalid("negative list offset"))?;
        add(add(bitmap, offsets)?, visible_size(&data.child_data()[0].slice(start, length), values, value_limit)?)
    };
    match data.data_type() {
        DataType::List(_) | DataType::Map(_, _) => {
            let offsets = data.buffer::<i32>(0);
            list(usize::try_from(offsets[0]).map_err(|_| invalid("negative list offset"))?, usize::try_from(offsets[data.len()]).map_err(|_| invalid("negative list offset"))?, 4, values)
        }
        DataType::LargeList(_) => {
            let offsets = data.buffer::<i64>(0);
            list(usize::try_from(offsets[0]).map_err(|_| invalid("negative list offset"))?, usize::try_from(offsets[data.len()]).map_err(|_| invalid("negative list offset"))?, 8, values)
        }
        DataType::FixedSizeList(_, width) => {
            let width = usize::try_from(*width).map_err(|_| invalid("negative list width"))?;
            let start = data.offset().checked_mul(width).ok_or_else(|| invalid("visible size overflow"))?;
            let length = data.len().checked_mul(width).ok_or_else(|| invalid("visible size overflow"))?;
            add(bitmap, visible_size(&data.child_data()[0].slice(start, length), values, value_limit)?)
        }
        DataType::Struct(_) => data.child_data().iter().try_fold(bitmap, |size, child| add(size, visible_size(child, values, value_limit)?)),
        _ => data.get_slice_memory_size(),
    }
}

/// Refuse oversized visible buffers/schema before the native writer allocates.
/// # Errors
/// Empty/incompatible inventory, checked extent or selected representation bound.
pub fn preflight(batches: &[RecordBatch], byte_limit: usize) -> Result<(), ArrowError> {
    let first = batches.first().ok_or_else(|| invalid("empty file inventory"))?;
    if batches.len() > MAX_BATCHES { return Err(invalid("file batch count bound")); }
    let mut visible = crate::schema_working_extent(first.schema().as_ref()).map_err(|error| invalid(&error.to_string()))?;
    for batch in batches {
        if batch.schema() != first.schema() { return Err(invalid("incompatible packed schemas")); }
        for array in batch.columns() {
            visible = visible.checked_add(visible_size(&array.to_data(), &mut 0, byte_limit)?).ok_or_else(|| invalid("file body size overflow"))?;
            if visible > byte_limit { return Err(invalid("file body exceeds bound")); }
        }
    }
    if visible > byte_limit { return Err(invalid("file body exceeds bound")); }
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
    let mut writer = FileWriter::try_new(LimitedWriter { bytes: Vec::new(), limit: byte_limit }, first.schema().as_ref())?;
    for batch in batches { writer.write(batch)?; }
    writer.finish()?;
    Ok(writer.into_inner()?.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::{array::{Array, Float64Array}, datatypes::{DataType, Field, Schema}, ipc::reader::FileReader};
    use std::{io::Cursor, sync::Arc};


    #[test]
    fn writer_preflight_refuses_before_native_allocation_and_counts_selected_nested_slice() {
        use arrow::array::{Int32Array, ListArray};
        use arrow::datatypes::Int32Type;
        let nested = ListArray::from_iter_primitive::<Int32Type, _, _>([Some(vec![Some(1)]), Some(vec![Some(2); 65536])]);
        let slice = nested.slice(0, 1);
        let nested: Arc<dyn Array> = Arc::new(slice);
        let batch = RecordBatch::try_from_iter(vec![("nested", nested)]).unwrap();
        assert!(encode(&[batch], 16384).is_ok(), "a small nested slice does not pay for its full parent");
        let values: Arc<dyn Array> = Arc::new(Int32Array::from(vec![1;65536]));
        let large = RecordBatch::try_from_iter(vec![("values", values)]).unwrap();
        let before = WRITER_STARTS.with(std::cell::Cell::get);
        assert!(encode(&[large], 16384).is_err());
        let schema = Arc::new(Schema::new_with_metadata(Vec::<Field>::new(), [("large".into(), "x".repeat(65536))].into()));
        assert!(encode(&[RecordBatch::new_empty(schema)], 16384).is_err());
        assert_eq!(WRITER_STARTS.with(std::cell::Cell::get), before);
    }

    #[test]
    fn bounded_file_writer_preserves_bits_and_batch_positions() {
        let schema = Arc::new(Schema::new(vec![Field::new("value", DataType::Float64, true)]));
        let bits = [0x8000_0000_0000_0000, 0, 0x7ff8_0000_0000_0042, 1, f64::INFINITY.to_bits()];
        let values = bits.iter().map(|bits| Some(f64::from_bits(*bits))).chain([None]).collect::<Vec<_>>();
        let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(Float64Array::from(values))]).unwrap();
        let bytes = encode(&[batch.clone(), batch], 65536).unwrap();
        let mut reader = FileReader::try_new(Cursor::new(bytes), None).unwrap();
        reader.set_index(1).unwrap();
        let decoded = reader.next().unwrap().unwrap();
        let values = decoded.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.len(), bits.len() + 1);
        assert!(values.is_null(bits.len()));
        assert_eq!(values.values()[..bits.len()].iter().map(|value| value.to_bits()).collect::<Vec<_>>(), bits);
    }

    #[test]
    fn bounded_file_writer_refuses_complete_file_overflow_and_incompatible_inventory() {
        let schema = Arc::new(Schema::new(vec![Field::new("value", DataType::Float64, true)]));
        let first = RecordBatch::new_empty(schema);
        assert!(encode(&[], 4096).is_err());
        assert!(encode(std::slice::from_ref(&first), 8).is_err());
        let other = RecordBatch::new_empty(Arc::new(Schema::new(vec![Field::new("other", DataType::Float64, true)])));
        assert!(encode(&[first, other], 4096).is_err());
    }
}

