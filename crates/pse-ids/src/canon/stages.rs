// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Internal stages run only after the complete preflight reservation is acquired.

use super::preflight::add;
use crate::{CanonError, CanonicalContract, ContentHash, EnvelopeBound, FieldPath, LogicalHash};
use arrow::compute::{SortOptions, concat_batches, take_record_batch};
use arrow_array::{ArrayRef, RecordBatch, StringArray, UInt32Array};
use arrow_ipc::writer::{IpcWriteOptions, StreamWriter};
use arrow_row::{RowConverter, SortField};
use arrow_schema::{DataType, Field, Schema};
use std::io::{self, Write};
use std::sync::Arc;

pub(super) fn admit_and_order(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
) -> Result<RecordBatch, CanonError> {
    let source = concat_batches(&contract.schema, batches)?;
    let fields = contract
        .primary_key
        .iter()
        .map(|index| {
            SortField::new_with_options(
                source.column(*index).data_type().clone(),
                SortOptions {
                    descending: false,
                    nulls_first: true,
                },
            )
        })
        .collect();
    let keys = contract
        .primary_key
        .iter()
        .map(|index| Arc::clone(source.column(*index)))
        .collect::<Vec<_>>();
    for (index, array) in contract.primary_key.iter().zip(&keys) {
        for row in 0..source.num_rows() {
            if array.is_null(row) {
                return Err(CanonError::NullKey {
                    column: contract.schema.field(*index).name().clone(),
                    row,
                });
            }
        }
    }
    let converter = RowConverter::new(fields)?;
    let rows = converter.convert_columns(&keys)?;
    let mut order = (0..source.num_rows()).collect::<Vec<_>>();
    order.sort_unstable_by(|one, two| rows.row(*one).cmp(&rows.row(*two)));
    for pair in order.windows(2) {
        if rows.row(pair[0]) == rows.row(pair[1]) {
            return Err(CanonError::DuplicateKey {
                first: pair[0],
                second: pair[1],
            });
        }
    }
    let indices = order
        .into_iter()
        .map(|index| {
            u32::try_from(index).map_err(|_| CanonError::Envelope {
                what: EnvelopeBound::Rows,
                limit: u64::from(u32::MAX),
                actual: u64::try_from(index).unwrap_or(u64::MAX),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(take_record_batch(&source, &UInt32Array::from(indices))?)
}

pub(super) fn metadata_relation(contract: &CanonicalContract) -> Result<RecordBatch, CanonError> {
    let mut rows = Vec::new();
    metadata(&FieldPath::root(), contract.schema.metadata(), &mut rows);
    for (index, field) in contract.schema.fields().iter().enumerate() {
        field_metadata(&FieldPath::root().child(index), field, &mut rows);
    }
    rows.sort_unstable();
    let fields = ["field_path", "key", "value"].map(|name| Field::new(name, DataType::Utf8, false));
    let columns: (Vec<_>, Vec<_>, Vec<_>) = rows.into_iter().fold(
        (Vec::new(), Vec::new(), Vec::new()),
        |(mut paths, mut keys, mut values), (path, key, value)| {
            paths.push(path);
            keys.push(key);
            values.push(value);
            (paths, keys, values)
        },
    );
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(columns.0)),
        Arc::new(StringArray::from(columns.1)),
        Arc::new(StringArray::from(columns.2)),
    ];
    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields.to_vec())),
        arrays,
    )?)
}
fn metadata(
    path: &FieldPath,
    metadata: &std::collections::HashMap<String, String>,
    out: &mut Vec<(String, String, String)>,
) {
    for (key, value) in metadata {
        if !matches!(key.as_str(), "pse.snapshot_id" | "pse.producer_pass_id") {
            out.push((path.as_str().to_owned(), key.clone(), value.clone()));
        }
    }
}
fn field_metadata(path: &FieldPath, field: &Field, out: &mut Vec<(String, String, String)>) {
    metadata(path, field.metadata(), out);
    match field.data_type() {
        DataType::List(child) | DataType::FixedSizeList(child, _) => {
            field_metadata(&path.child(0), child, out);
        }
        DataType::Struct(children) => {
            for (index, child) in children.iter().enumerate() {
                field_metadata(&path.child(index), child, out);
            }
        }
        _ => {}
    }
}

struct BoundedBytes {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}
impl Write for BoundedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|total| total > self.limit)
        {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "canonical stream exceeds reserved preflight capacity",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn stream(batch: &RecordBatch, limit: usize) -> Result<Vec<u8>, CanonError> {
    let mut output = BoundedBytes {
        bytes: Vec::with_capacity(limit),
        limit,
        exceeded: false,
    };
    let result = (|| -> Result<(), arrow_schema::ArrowError> {
        let options =
            IpcWriteOptions::try_new(super::IPC_ALIGNMENT, false, super::IPC_METADATA_VERSION)?;
        let mut writer = StreamWriter::try_new_with_options(&mut output, &batch.schema(), options)?;
        writer.write(batch)?;
        writer.finish()
    })();
    if output.exceeded {
        return Err(CanonError::Envelope {
            what: EnvelopeBound::Bytes,
            limit: u64::try_from(limit).unwrap_or(u64::MAX),
            actual: u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1),
        });
    }
    result?;
    Ok(output.bytes)
}

pub(super) fn frame_and_hash(
    contract: &CanonicalContract,
    metadata: &[u8],
    data: &[u8],
) -> Result<(LogicalHash, Vec<u8>), CanonError> {
    let capacity = add(
        add(add(8, super::CANON_VERSION.len())?, 16 + 4 + 32 + 8 + 8)?,
        add(metadata.len(), data.len())?,
    )?;
    let mut preimage = Vec::with_capacity(capacity);
    part(&mut preimage, super::CANON_VERSION.as_bytes())?;
    preimage.extend_from_slice(contract.relation_id.as_bytes());
    preimage.extend_from_slice(&contract.schema_version.0.to_le_bytes());
    preimage.extend_from_slice(contract.registry_fingerprint.as_bytes());
    part(&mut preimage, metadata)?;
    part(&mut preimage, data)?;
    let hash = LogicalHash(ContentHash::from_bytes(*blake3::hash(&preimage).as_bytes()));
    Ok((hash, preimage))
}
fn part(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CanonError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| CanonError::Internal("frame length does not fit u64".to_owned()))?;
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}
