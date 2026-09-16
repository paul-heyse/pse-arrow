// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded admission for the uncompressed, plain-value Parquet baseline.

use std::sync::Arc;

use bytes::Bytes;
use datafusion::parquet;
use datafusion::parquet::arrow::arrow_reader::{
    ArrowReaderMetadata, ArrowReaderOptions, ParquetRecordBatchReaderBuilder,
};
use datafusion::parquet::basic::{Compression, Encoding};
use datafusion::parquet::column::page::Page;
use datafusion::parquet::file::metadata::ParquetStatisticsPolicy;
use datafusion::parquet::file::properties::ReaderProperties;
use datafusion::parquet::file::reader::FileReader;
use datafusion::parquet::file::serialized_reader::{ReadOptionsBuilder, SerializedFileReader};
use pse_ids::{CancellationToken, Envelope, MemoryReserver, ReservationLease};
use pse_schema::Registry;
use pse_schema::model::RelationSpec;

use super::admission;
use crate::CatalogError;

/// Actual file/footer/page extents are checked before Arrow allocation. The supported
/// physical baseline is explicit: uncompressed PLAIN values and the exact declared
/// Arrow schema envelope; other encodings return typed admission errors.
pub(super) fn decode(
    bytes: &Bytes,
    reg: &Registry,
    spec: &RelationSpec,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<pse_relations::columnar::FieldCheckedBatch, CatalogError> {
    cancel.checkpoint()?;
    Envelope::new(envelope.max_rows, envelope.max_normalized_bytes)?;
    let footer = footer(bytes)?;
    let mut control = reserver.open("store:parquet-metadata");
    control.try_grow(
        footer
            .len()
            .checked_mul(40)
            .ok_or_else(super::super::encode::overflow)?,
    )?;
    let measured = super::compact::preflight(footer)?;
    control.try_grow(measured.saturating_sub(control.size()))?;
    let options = ReadOptionsBuilder::new()
        .with_reader_properties(
            ReaderProperties::builder()
                .set_read_bloom_filter(false)
                .set_read_page_statistics(false)
                .build(),
        )
        .with_encoding_stats_policy(ParquetStatisticsPolicy::SkipAll)
        .with_column_stats_policy(ParquetStatisticsPolicy::SkipAll)
        .with_size_stats_policy(ParquetStatisticsPolicy::SkipAll)
        .build();
    let reader = SerializedFileReader::new_with_options(bytes.clone(), options)
        .map_err(|source| error(&source))?;
    let metadata = reader.metadata();
    let schema = Arc::new(
        pse_schema::arrow::relation_schema(reg, spec)
            .map_err(|error| admission("Parquet schema", &error.to_string()))?,
    );
    let expected = parquet::arrow::ArrowSchemaConverter::new()
        .convert(&schema)
        .map_err(|source| error(&source))?;
    if metadata.file_metadata().schema_descr() != &expected {
        return Err(admission(
            "Parquet schema",
            "physical declaration differs from the registry projection",
        ));
    }
    let expected_arrow = parquet::arrow::encode_arrow_schema(&schema);
    let entries = metadata
        .file_metadata()
        .key_value_metadata()
        .into_iter()
        .flatten()
        .filter(|entry| entry.key == parquet::arrow::ARROW_SCHEMA_META_KEY)
        .collect::<Vec<_>>();
    if entries.len() != 1 || entries[0].value.as_deref() != Some(expected_arrow.as_str()) {
        return Err(admission(
            "Parquet Arrow metadata",
            "one exact declared Arrow schema envelope is required; semantic metadata is never inferred",
        ));
    }
    let rows = u64::try_from(metadata.file_metadata().num_rows())
        .map_err(|_| admission("Parquet", "negative row count"))?;
    if rows > envelope.max_rows {
        return Err(admission("Parquet", "row count exceeds supported envelope"));
    }
    let extent = page_extent(&reader, bytes.len() - footer.len() - 8, cancel)?;
    let mut reservation = reserver.open("store:parquet-decode");
    reservation.try_grow(extent)?;
    let arrow_metadata =
        ArrowReaderMetadata::try_new(Arc::new(metadata.clone()), ArrowReaderOptions::new())
            .map_err(|source| error(&source))?;
    pse_relations::validate::validate_schema(reg, spec, arrow_metadata.schema())
        .map_err(|errors| admission("Parquet Arrow schema", &super::messages(&errors)))?;
    let builder = ParquetRecordBatchReaderBuilder::new_with_metadata(bytes.clone(), arrow_metadata)
        .with_batch_size(1024);
    let mut batches = Vec::new();
    let mut total = 0usize;
    for batch in builder.build().map_err(|source| error(&source))? {
        cancel.checkpoint()?;
        let batch = batch.map_err(|error| admission("Parquet values", &error.to_string()))?;
        total = total
            .checked_add(batch.num_rows())
            .ok_or_else(super::super::encode::overflow)?;
        if u64::try_from(total).ok().is_none_or(|actual| actual > rows) {
            return Err(admission("Parquet", "decoded rows exceed metadata"));
        }
        batches.push(batch);
    }
    if u64::try_from(total).ok() != Some(rows) {
        return Err(admission(
            "Parquet",
            "decoded row count differs from actual metadata",
        ));
    }
    let batch = datafusion::arrow::compute::concat_batches(&schema, &batches)
        .map_err(|error| admission("Parquet batches", &error.to_string()))?;
    let mut validation = reserver.open("store:parquet-value-admission");
    validation.try_grow(super::super::membership::validation_extent(&batch)?)?;
    drop(batches);
    let retained = pse_ids::owned_buffer::retained_buffer_bytes(&batch)?;
    reservation.shrink(reservation.size().saturating_sub(retained));
    let batch =
        pse_ids::owned_buffer::attach_reservation(batch, ReservationLease::new(reservation))?;
    Ok(pse_relations::columnar::FieldCheckedBatch::admit(
        reg, spec, batch,
    )?)
}

fn footer(bytes: &[u8]) -> Result<&[u8], CatalogError> {
    let end = bytes
        .len()
        .checked_sub(8)
        .ok_or_else(|| admission("Parquet", "missing footer"))?;
    if !bytes.starts_with(b"PAR1") || bytes.get(end + 4..) != Some(b"PAR1".as_slice()) {
        return Err(admission(
            "Parquet",
            "finished plaintext file magic is absent",
        ));
    }
    let encoded: [u8; 4] = bytes[end..end + 4]
        .try_into()
        .map_err(|_| admission("Parquet", "invalid footer length"))?;
    let len = usize::try_from(u32::from_le_bytes(encoded))
        .map_err(|_| admission("Parquet", "footer extent overflow"))?;
    let start = end
        .checked_sub(len)
        .filter(|start| *start >= 4)
        .ok_or_else(|| admission("Parquet", "footer exceeds actual file extent"))?;
    Ok(&bytes[start..end])
}

fn page_extent(
    reader: &SerializedFileReader<Bytes>,
    data_end: usize,
    cancel: &CancellationToken,
) -> Result<usize, CatalogError> {
    let mut extent = page_construction_extent(data_end)?;
    let mut ranges = Vec::new();
    let mut total_rows = 0i64;
    for (index, group) in reader.metadata().row_groups().iter().enumerate() {
        total_rows = total_rows
            .checked_add(group.num_rows())
            .filter(|rows| *rows >= 0)
            .ok_or_else(|| admission("Parquet", "invalid row-group row count"))?;
        let row_group = reader
            .get_row_group(index)
            .map_err(|source| error(&source))?;
        for (column_index, column) in group.columns().iter().enumerate() {
            cancel.checkpoint()?;
            if column.compression() != Compression::UNCOMPRESSED {
                return Err(admission(
                    "Parquet compression",
                    "only the uncompressed Wave 1 baseline is supported",
                ));
            }
            let start = usize::try_from(
                column
                    .dictionary_page_offset()
                    .unwrap_or(column.data_page_offset()),
            )
            .map_err(|_| admission("Parquet", "negative column offset"))?;
            let len = usize::try_from(column.compressed_size())
                .map_err(|_| admission("Parquet", "negative column extent"))?;
            let end = start
                .checked_add(len)
                .filter(|end| *end <= data_end)
                .ok_or_else(|| admission("Parquet", "column exceeds actual file data extent"))?;
            if start < 4
                || ranges
                    .iter()
                    .any(|(left, right)| start < *right && *left < end)
            {
                return Err(admission("Parquet", "overlapping column byte ranges"));
            }
            ranges.push((start, end));
            let count = usize::try_from(column.num_values())
                .map_err(|_| admission("Parquet", "negative value count"))?;
            let width = usize::try_from(column.column_descr().type_length().max(0))
                .map_err(|_| admission("Parquet", "invalid physical width"))?;
            extent = extent
                .checked_add(
                    count
                        .checked_mul(width.saturating_add(256))
                        .ok_or_else(super::super::encode::overflow)?,
                )
                .ok_or_else(super::super::encode::overflow)?;
            let mut pages = row_group
                .get_column_page_reader(column_index)
                .map_err(|source| error(&source))?;
            let mut values = 0usize;
            while let Some(page) = pages.get_next_page().map_err(|source| error(&source))? {
                cancel.checkpoint()?;
                if matches!(page, Page::DictionaryPage { .. })
                    || page.encoding() != Encoding::PLAIN
                    || matches!(
                        page,
                        Page::DataPageV2 {
                            is_compressed: true,
                            ..
                        }
                    )
                {
                    return Err(admission(
                        "Parquet encoding",
                        "only uncompressed PLAIN data pages are supported",
                    ));
                }
                values = values
                    .checked_add(
                        usize::try_from(page.num_values())
                            .map_err(|_| admission("Parquet", "page value count overflow"))?,
                    )
                    .ok_or_else(super::super::encode::overflow)?;
                if values > count {
                    return Err(admission(
                        "Parquet",
                        "actual page value counts exceed column claim",
                    ));
                }
            }
            if values != count {
                return Err(admission(
                    "Parquet",
                    "actual page value counts differ from column claim",
                ));
            }
        }
    }
    if total_rows != reader.metadata().file_metadata().num_rows() {
        return Err(admission(
            "Parquet",
            "row-group counts differ from file claim",
        ));
    }
    Ok(extent)
}
fn error(error: &parquet::errors::ParquetError) -> CatalogError {
    admission("Parquet", &error.to_string())
}

fn page_construction_extent(data_end: usize) -> Result<usize, CatalogError> {
    data_end
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(16 << 10))
        .ok_or_else(super::super::encode::overflow)
}
