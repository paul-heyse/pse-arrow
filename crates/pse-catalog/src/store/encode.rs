// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finished, resource-accounted physical encodings (blueprint §5.3 step 6, §20.1).

use std::io::{self, Write};

use bytes::Bytes;
use datafusion::arrow::ipc::{
    self,
    writer::{FileWriter, IpcWriteOptions},
};
use datafusion::arrow::{
    array::{ArrayData, RecordBatch},
    error::ArrowError,
};
use datafusion::parquet;
use pse_ids::{CancellationToken, EncodingChecksum, MemoryReserver, ReservationLease};

use crate::{CatalogError, EncodingFormat};

/// A finished physical encoding whose clones retain its resource owner.
#[derive(Clone, Debug)]
pub struct EncodedRelation {
    /// The actual completed format.
    pub format: EncodingFormat,
    /// Exact finished object bytes, including its footer.
    pub bytes: Bytes,
    /// Checksum of these exact finished bytes.
    pub checksum: EncodingChecksum,
    /// Writer contract recorded in the manifest.
    pub writer_version: String,
}

/// Encode one sorted, semantically admitted relation as an uncompressed V5 IPC file.
///
/// # Errors
/// Reservation exhaustion, cancellation or unsuccessful Arrow serialization/finish.
pub fn ipc_file(
    batch: &RecordBatch,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<EncodedRelation, CatalogError> {
    cancel.checkpoint()?;
    let capacity = encoding_capacity(batch)?;
    let mut reservation = reserver.open("store:ipc-encode");
    // The output and IPC writer's body/metadata copy coexist. Input backing is already
    // owned by the admitted relation; this separate claim covers the encoder stages.
    reservation.try_grow(capacity.checked_mul(3).ok_or_else(overflow)?)?;
    let mut output = BoundedWriter {
        bytes: Vec::with_capacity(capacity),
        limit: capacity,
    };
    write_ipc(batch, &mut output, cancel)?;
    output.bytes.shrink_to_fit();
    reservation.shrink(reservation.size().saturating_sub(output.bytes.capacity()));
    let lease = ReservationLease::new(reservation);
    let checksum = pse_ids::encoding_checksum(&output.bytes);
    let bytes = pse_ids::owned_buffer::attach_bytes(output.bytes, lease)?;
    Ok(EncodedRelation {
        format: EncodingFormat::ArrowIpcFile,
        bytes,
        checksum,
        writer_version: "arrow-rs 59.3.0; ipc-v5; alignment=64; uncompressed".to_owned(),
    })
}

fn write_ipc(
    batch: &RecordBatch,
    output: &mut impl Write,
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    let options = IpcWriteOptions::try_new(64, false, ipc::MetadataVersion::V5).map_err(arrow)?;
    let mut writer = FileWriter::try_new_with_options(output, batch.schema().as_ref(), options)
        .map_err(arrow)?;
    writer.write(batch).map_err(arrow)?;
    cancel.checkpoint()?;
    writer.finish().map_err(arrow)
}

fn encoding_capacity(batch: &RecordBatch) -> Result<usize, CatalogError> {
    let retained = pse_ids::owned_buffer::retained_buffer_bytes(batch)?;
    // Count the admitted schema's complete nested metadata rendering without allocating
    // that rendering before the reservation is acquired.
    let mut schema = SchemaCount(0);
    std::fmt::write(&mut schema, format_args!("{:?}", batch.schema())).map_err(|_| overflow())?;
    retained
        .checked_mul(4)
        .and_then(|n| schema.0.checked_mul(16).and_then(|s| n.checked_add(s)))
        .and_then(|n| n.checked_add(16 << 10))
        .ok_or_else(overflow)
}

struct SchemaCount(usize);
impl std::fmt::Write for SchemaCount {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

pub(crate) fn arrow(source: ArrowError) -> CatalogError {
    CatalogError::Infrastructure {
        op: "encode Arrow relation".to_owned(),
        source: Box::new(source),
    }
}
pub(crate) fn overflow() -> CatalogError {
    CatalogError::ResourceLimit {
        consumer: "store encoding".to_owned(),
        config_keys: Vec::new(),
        detail: "checked encoding extent overflow".to_owned(),
    }
}

struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "encoded file exceeds its pre-reserved live extent",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Count an actual JSON serialization without allocating, reserve its bounded output,
/// then serialize exactly those bytes. This keeps manifest/ref control allocations on
/// the same before-allocation path as Arrow buffers.
pub(crate) fn control<T: serde::Serialize>(
    value: &T,
    reserver: &dyn MemoryReserver,
    owner: &str,
    limit: usize,
) -> Result<Bytes, CatalogError> {
    let mut count = CountWriter { len: 0, limit };
    serde_json::to_writer(&mut count, value)
        .map_err(|error| super::verify::admission(owner, &error.to_string()))?;
    let mut reservation = reserver.open(owner);
    reservation.try_grow(count.len)?;
    let lease = ReservationLease::new(reservation);
    let mut output = BoundedWriter {
        bytes: Vec::with_capacity(count.len),
        limit: count.len,
    };
    serde_json::to_writer(&mut output, value)
        .map_err(|error| super::verify::admission(owner, &error.to_string()))?;
    Ok(pse_ids::owned_buffer::attach_bytes(output.bytes, lease)?)
}
struct CountWriter {
    len: usize,
    limit: usize,
}
impl Write for CountWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .filter(|len| *len <= self.limit)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "control encoding exceeds supported finite extent",
                )
            })?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encode the Wave 1 durable baseline: finished, uncompressed, plain-value Parquet.
/// Arrow semantic metadata is preserved explicitly; dictionary storage is restored
/// from the declared Arrow schema when reading.
///
/// # Errors
/// Resource exhaustion, cancellation or unsuccessful Parquet write/finish.
pub fn parquet_file(
    batch: &RecordBatch,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<EncodedRelation, CatalogError> {
    use datafusion::parquet::basic::{Compression, Encoding};
    use datafusion::parquet::file::properties::{
        EnabledStatistics, WriterProperties, WriterVersion,
    };
    cancel.checkpoint()?;
    let mut expansion = 0usize;
    for array in batch.columns() {
        let data = array.to_data();
        data.validate_full().map_err(arrow)?;
        expansion = expansion
            .checked_add(dictionary_expansion(&data)?)
            .ok_or_else(overflow)?;
    }
    let capacity = super::membership::validation_extent(batch)?
        .checked_add(encoding_capacity(batch)?)
        .and_then(|bytes| {
            expansion
                .checked_mul(4)
                .and_then(|expanded| bytes.checked_add(expanded))
        })
        .ok_or_else(overflow)?;
    let mut reservation = reserver.open("store:parquet-encode");
    reservation.try_grow(capacity.checked_mul(4).ok_or_else(overflow)?)?;
    let mut output = BoundedWriter {
        bytes: Vec::with_capacity(capacity),
        limit: capacity,
    };
    let properties = WriterProperties::builder()
        .set_writer_version(WriterVersion::PARQUET_1_0)
        .set_compression(Compression::UNCOMPRESSED)
        .set_dictionary_enabled(false)
        .set_encoding(Encoding::PLAIN)
        .set_statistics_enabled(EnabledStatistics::None)
        .set_offset_index_disabled(true)
        .set_max_row_group_row_count(Some(65_536))
        .build();
    {
        let mut writer =
            parquet::arrow::ArrowWriter::try_new(&mut output, batch.schema(), Some(properties))
                .map_err(parquet)?;
        writer.write(batch).map_err(parquet)?;
        cancel.checkpoint()?;
        writer.finish().map_err(parquet)?;
    }
    output.bytes.shrink_to_fit();
    reservation.shrink(reservation.size().saturating_sub(output.bytes.capacity()));
    let lease = ReservationLease::new(reservation);
    let checksum = pse_ids::encoding_checksum(&output.bytes);
    Ok(EncodedRelation {
        format: EncodingFormat::Parquet,
        bytes: pse_ids::owned_buffer::attach_bytes(output.bytes, lease)?,
        checksum,
        writer_version: "parquet-rs 59.3.0; parquet-1.0; uncompressed; plain".to_owned(),
    })
}
fn parquet(source: parquet::errors::ParquetError) -> CatalogError {
    CatalogError::Infrastructure {
        op: "encode Parquet relation".to_owned(),
        source: Box::new(source),
    }
}

/// Bounds dictionary text when tagged keys or physical encoders expand each key.
pub(super) fn dictionary_expansion(data: &ArrayData) -> Result<usize, CatalogError> {
    pse_ids::validation_extent::dictionary_expansion(data).map_err(Into::into)
}

#[cfg(test)]
mod finish_tests {
    use datafusion::arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema},
    };
    use std::io::{self, Write};
    use std::sync::Arc;

    #[derive(Default)]
    struct RefuseFinalMagic {
        initial_magic: bool,
        failed_at_finish: bool,
    }
    impl Write for RefuseFinalMagic {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.starts_with(b"ARROW1") {
                if self.initial_magic {
                    self.failed_at_finish = true;
                    return Err(io::Error::other("injected final footer write failure"));
                }
                self.initial_magic = true;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn the_actual_encoder_propagates_failed_finish_before_an_artifact_exists() {
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("id", DataType::UInt64, false)])),
            vec![Arc::new(UInt64Array::from(vec![1]))],
        )
        .expect("batch");
        let mut writer = RefuseFinalMagic::default();
        let outcome = super::write_ipc(&batch, &mut writer, &pse_ids::CancellationToken::default());
        assert!(
            writer.failed_at_finish,
            "fault fires on the final file magic after the body was written"
        );
        assert!(
            outcome.is_err(),
            "the production encoder cannot return an EncodedRelation after this error"
        );
    }
}
