// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete-message stream admission before Arrow 59.3 decoding. No chunk
//! accumulation, compression, dictionaries or schema-derived scientific authority.
use std::{io::{Read, Seek, SeekFrom}, sync::Arc};
use arrow::{buffer::{Buffer, MutableBuffer}, datatypes::{DataType, SchemaRef}, error::ArrowError,
    ipc::{self, reader::StreamDecoder}};
use crate::{AllocationLease, CancellationToken, CanonError, MemoryConsumer,
    MemoryPool, MemoryReservation};

use crate::owned_buffer::{AllocationScope, OwnedRecordBatch};

const MAX_FIELDS: usize = 16 * 1024;
const MAX_VALUES: usize = 32 * 1024 * 1024;
const MAX_TABLES: usize = 128 * 1024;
const MAX_BATCHES: usize = super::ipc_file::MAX_BATCHES;

/// Independent limits on each complete encoded message and its metadata.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Metadata (including its continuation prefix) and body together.
    pub message_bytes: usize,
    /// Schema and record metadata, excluding their eight-byte prefix.
    pub metadata_bytes: usize,
}
/// Complete conservative capacity for standalone offline validation/one-batch
/// preview. This is a scoped ownership adapter, never a process-global pool.
/// Native schema strings can repeat every verified table/field reference; each
/// occurrence is owned separately. Include the maximum table/field headers,
/// metadata/scratch, one encoded body, its per-buffer alignment copies and the
/// simultaneously live schema. Existing encoded/document limits remain separate.
/// # Errors
/// An unrepresentable complete envelope.
pub fn offline_pool(limits: Limits) -> Result<Arc<dyn MemoryPool>, CanonError> {
    let string_occurrences = mul(add(MAX_TABLES, MAX_FIELDS)?, 2)?;
    let strings = mul(mul(limits.metadata_bytes, 2)?, string_occurrences)?;
    let schema = add(strings, add(mul(MAX_FIELDS, 1024)?, mul(MAX_TABLES, 256)?)?)?;
    let buffers = add(mul(rounded(limits.message_bytes)?, 2)?, mul(MAX_FIELDS * 3, 64)?)?;
    let metadata = mul(rounded(add(limits.metadata_bytes, 8)?)?, 2)?;
    let headers = add(mul(MAX_FIELDS, 4096 + 64)?, 16384)?;
    let extent = add(add(schema, buffers)?, add(metadata, headers)?)?;
    Ok(Arc::new(crate::GreedyMemoryPool::new(extent)))
}
fn invalid(message: &'static str) -> CanonError {
    ArrowError::InvalidArgumentError(message.into()).into()
}
fn add(a: usize, b: usize) -> Result<usize, CanonError> {
    a.checked_add(b).ok_or_else(|| invalid("IPC allocation extent overflow"))
}
fn mul(a: usize, b: usize) -> Result<usize, CanonError> {
    a.checked_mul(b).ok_or_else(|| invalid("IPC allocation extent overflow"))
}
fn rounded(n: usize) -> Result<usize, CanonError> {
    Ok(add(n, 63)? / 64 * 64)
}
fn reserve(pool: &Arc<dyn MemoryPool>, bytes: usize) -> Result<MemoryReservation, CanonError> {
    let reservation = MemoryConsumer::new("ipc:complete-message").register(pool);
    reservation.try_grow(bytes)?;
    Ok(reservation)
}
fn io(error: std::io::Error) -> CanonError { ArrowError::from(error).into() }
fn options(bytes: usize) -> flatbuffers::VerifierOptions {
    flatbuffers::VerifierOptions { max_depth: 256, max_tables: MAX_TABLES,
        max_apparent_size: bytes, ..Default::default() }
}

struct Frame {
    metadata: Buffer,
    body: usize,
    verifier_bytes: usize,
    grant: MemoryReservation,
}
impl Frame {
    fn message(&self) -> Result<ipc::Message<'_>, CanonError> {
        ipc::root_as_message_with_opts(&options(self.verifier_bytes), &self.metadata[8..])
            .map_err(|_| invalid("invalid IPC message metadata"))
    }
}

/// One decoded batch and its pre-acquired non-buffer allocation owner.
/// Native buffer slices retain their own original reservations. Consumers that
/// retain typed wrappers also retain `working_owner` with their existing export owner.
#[derive(Debug)]
pub struct StreamBatch {
    /// Validated native buffers with their allocation scope.
    pub batch: OwnedRecordBatch,
    /// Schema, arrays and decoder/wrapper inventory admitted before construction.
    pub working_owner: Arc<StreamOwnership>,
}

/// Shared decode inventory; a batch keeps the original schema grant and its
/// own pre-acquired wrapper grant together without reserving shared schema twice.
#[derive(Debug)]
pub struct StreamOwnership {
    _schema: Arc<AllocationLease>,
    _working: Arc<AllocationLease>,
}

/// Bounded seekable source. The caller covers simultaneously live source storage;
/// this reader prepays metadata, schema, input backing, wrappers and realignment
/// against that same supplied pool before constructing any of them.
pub struct StreamReader<R> {
    input: R,
    remaining: u64,
    limits: Limits,
    pool: Arc<dyn MemoryPool>,
    cancel: CancellationToken,
    schema: SchemaRef,
    schema_owner: Arc<AllocationLease>,
    decoder: Option<StreamDecoder>,
    decoder_continuation: bool,
    scope: AllocationScope,
    batches: usize,
    ended: bool,
}
impl<R> std::fmt::Debug for StreamReader<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdmittedStreamReader").field("remaining", &self.remaining)
            .field("limits", &self.limits).field("batches", &self.batches).finish_non_exhaustive()
    }
}

// Arrow 59.3 uses one Field/Arc/DataType and short vectors per declared field;
// allow 1024 bytes per field, 256 per metadata pair (including the hash table's
// <=4x bucket capacity), and twice each visible string's length for owned copies.
// Count referenced strings per occurrence: shared FlatBuffer offsets do not make
// independently converted native strings share storage.
fn metadata_extent(entries: Option<flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ipc::KeyValue<'_>>>>) -> Result<usize, CanonError> {
    let mut bytes = 0;
    if let Some(entries) = entries {
        for entry in entries {
            let key = entry.key().ok_or_else(|| invalid("IPC metadata key absent"))?;
            let value = entry.value().ok_or_else(|| invalid("IPC metadata value absent"))?;
            bytes = add(bytes, add(256, mul(add(key.len(), value.len())?, 2)?)?)?;
        }
    }
    Ok(bytes)
}
fn schema_extent(schema: ipc::Schema<'_>) -> Result<(usize, usize), CanonError> {
    if schema.endianness() != ipc::Endianness::Little { return Err(invalid("IPC schema endianness")); }
    let fields = schema.fields().ok_or_else(|| invalid("IPC schema fields absent"))?;
    let mut count = 0;
    for field in fields { super::ipc_admission::schema_field(field, 0, &mut count, MAX_FIELDS)?; }
    let mut bytes = add(4096, metadata_extent(schema.custom_metadata())?)?;
    for field in fields { bytes = add(bytes, schema_extent_field(field)?)?; }
    Ok((count, bytes))
}
// Recursive field accounting (same 59.3 Field/hash/string derivation as above).
fn schema_extent_field(field: ipc::Field<'_>) -> Result<usize, CanonError> {
    let mut bytes = add(1024, mul(field.name().map_or(0, str::len), 2)?)?;
    bytes = add(bytes, metadata_extent(field.custom_metadata())?)?;
    if let Some(timestamp) = field.type_as_timestamp() {
        bytes = add(bytes, mul(timestamp.timezone().map_or(0, str::len), 2)?)?;
    }
    if let Some(children) = field.children() {
        for child in children { bytes = add(bytes, schema_extent_field(child)?)?; }
    }
    Ok(bytes)
}

impl<R: Read + Seek> StreamReader<R> {
    /// Admit and decode the schema without giving it registry/lineage authority.
    /// `bytes` is the actual selected source extent, never an advertised message length.
    /// # Errors
    /// Truncation, malformed/unsupported framing or schema, cancellation or pool refusal.
    pub fn try_new(input: R, bytes: u64, limits: Limits, pool: Arc<dyn MemoryPool>, cancel: CancellationToken) -> Result<Self, CanonError> {
        Self::open(input, bytes, limits, pool, cancel, true)
    }
    fn open(mut input: R, bytes: u64, limits: Limits, pool: Arc<dyn MemoryPool>, cancel: CancellationToken, decode: bool) -> Result<Self, CanonError> {
        cancel.checkpoint()?;
        let start = input.stream_position().map_err(io)?;
        let actual = input.seek(SeekFrom::End(0)).map_err(io)?;
        input.seek(SeekFrom::Start(start)).map_err(io)?;
        if start.checked_add(bytes).is_none_or(|end| end > actual) {
            return Err(invalid("IPC selected extent exceeds actual source"));
        }
        // Placeholder schema and reader-owned state are prepaid too.
        let placeholder = reserve(&pool, 4096)?;
        let mut reader = Self { input, remaining: bytes, limits, pool, cancel,
            schema: Arc::new(arrow::datatypes::Schema::empty()), schema_owner: AllocationLease::new(placeholder),
            decoder: decode.then(|| StreamDecoder::new().with_require_alignment(true)),
            decoder_continuation: false,
            scope: AllocationScope::default(), batches: 0, ended: false };
        let frame = reader.frame()?.ok_or_else(|| invalid("IPC schema message absent"))?;
        let message = frame.message()?;
        let schema = message.header_as_schema().ok_or_else(|| invalid("IPC schema must be first"))?;
        if frame.body != 0 { return Err(invalid("IPC schema has body")); }
        let (_, extent) = schema_extent(schema)?;
        // StreamDecoder copies metadata when a zero-body message exactly ends
        // its buffer. Cover that rounded accumulation before calling it.
        frame.grant.try_grow(add(extent, rounded(frame.metadata.len())?)?)?;
        reader.cancel.checkpoint()?;
        reader.schema = if let Some(decoder) = &mut reader.decoder {
            let mut buffer = frame.metadata.clone();
            if decoder.decode(&mut buffer)?.is_some() || !buffer.is_empty() {
                return Err(invalid("IPC schema decode state"));
            }
            reader.decoder_continuation = true;
            decoder.schema().ok_or_else(|| invalid("IPC schema decode absent"))?
        } else { Arc::new(ipc::convert::fb_to_schema(schema)) };
        reader.schema_owner = AllocationLease::new(frame.grant);
        Ok(reader)
    }
    /// Schema representation only. Callers retain registry and lineage admission.
    pub fn schema(&self) -> SchemaRef { self.schema.clone() }
    /// Keep schema allocation admitted through a caller's semantic wrapper lifetime.
    pub fn schema_owner(&self) -> Arc<AllocationLease> { self.schema_owner.clone() }
    fn frame(&mut self) -> Result<Option<Frame>, CanonError> {
        self.cancel.checkpoint()?;
        if self.ended { return Ok(None); }
        if self.remaining < 8 { return Err(invalid("IPC terminal marker absent/truncated")); }
        let mut header = [0u8; 8];
        self.input.read_exact(&mut header).map_err(io)?;
        self.remaining -= 8;
        if header[..4] != [255; 4] { return Err(invalid("IPC continuation marker")); }
        let length = i32::from_le_bytes(header[4..].try_into().map_err(|_| invalid("IPC metadata length"))?);
        let length = usize::try_from(length).map_err(|_| invalid("negative IPC metadata length"))?;
        if length == 0 {
            if self.remaining != 0 { return Err(invalid("IPC bytes after terminal marker")); }
            self.ended = true;
            if let Some(decoder) = &mut self.decoder {
                let grant = reserve(&self.pool, 64)?;
                let mut marker = Buffer::from_slice_ref(if self.decoder_continuation { &header[4..] } else { &header[..] });
                decoder.decode(&mut marker)?;
                decoder.finish()?;
                drop(grant);
            }
            return Ok(None);
        }
        if length > self.limits.metadata_bytes || length as u64 > self.remaining
            || add(length, 8)? > self.limits.message_bytes {
            return Err(invalid("IPC metadata outside actual extent/allowance"));
        }
        let capacity = rounded(add(length, 12)?)?;
        let grant = reserve(&self.pool, capacity)?;
        self.cancel.checkpoint()?;
        let mut metadata = MutableBuffer::from_len_zeroed(add(length, 12)?);
        metadata[..8].copy_from_slice(&header);
        self.input.read_exact(&mut metadata[8..length + 8]).map_err(io)?;
        metadata[length + 8..].copy_from_slice(&[255; 4]);
        self.remaining -= length as u64;
        let mut frame = Frame { metadata: metadata.into(), body: 0, verifier_bytes: self.limits.metadata_bytes, grant };
        let message = frame.message()?;
        if message.version() != ipc::MetadataVersion::V5 { return Err(invalid("unsupported IPC metadata version")); }
        let body = usize::try_from(message.bodyLength()).map_err(|_| invalid("negative IPC body length"))?;
        if body as u64 > self.remaining || add(add(length, 8)?, body)? > self.limits.message_bytes {
            return Err(invalid("IPC body outside actual extent/allowance"));
        }
        frame.body = body;
        Ok(Some(frame))
    }
    fn record(&mut self, decode: bool) -> Result<Option<StreamBatch>, CanonError> {
        let Some(frame) = self.frame()? else { return Ok(None); };
        self.batches += 1;
        if self.batches > MAX_BATCHES { return Err(invalid("IPC batch count bound")); }
        let message = frame.message()?;
        let batch = message.header_as_record_batch().ok_or_else(|| invalid("IPC record batch required; dictionary refused"))?;
        let fields = super::ipc_admission::expected_shape(self.schema.as_ref())?;
        let inventory = add(batch.nodes().map_or(0, |nodes| nodes.len()), batch.buffers().map_or(0, |buffers| buffers.len()))?;
        frame.grant.try_grow(add(mul(inventory, 16)?, 4096)?)?;
        validate_record(batch, self.schema.as_ref(), fields, frame.body)?;
        if !decode {
            self.input.seek(SeekFrom::Current(i64::try_from(frame.body).map_err(|_| invalid("IPC seek overflow"))?)).map_err(io)?;
            self.remaining -= frame.body as u64;
            return Ok(None);
        }
        // ArrayData, concrete array/Arc wrappers, decoder vectors and the ownership
        // attachment rebuild coexist. 4096 per declared node covers these bounded
        // headers/short vectors at 59.3 (no value-proportional decompression).
        let wrappers = add(mul(fields, 4096)?, 4096)?;
        frame.grant.try_grow(add(rounded(frame.body)?, add(wrappers, rounded(frame.metadata.len())?)?)?)?;
        self.cancel.checkpoint()?;
        let mut body = MutableBuffer::from_len_zeroed(frame.body);
        self.input.read_exact(body.as_slice_mut()).map_err(io)?;
        self.remaining -= frame.body as u64;
        let copies = alignment_copies(batch, self.schema.as_ref(), body.as_slice())?;
        frame.grant.try_grow(copies)?;
        self.cancel.checkpoint()?;
        let body = self.scope.attach_buffer(body.into(), frame.grant.split(rounded(frame.body)?))?;
        let decoder = self.decoder.take().ok_or_else(|| invalid("IPC decoder absent"))?;
        self.decoder = Some(decoder.with_require_alignment(copies == 0));
        let decoder = self.decoder.as_mut().ok_or_else(|| invalid("IPC decoder absent"))?;
        // A complete body follows complete metadata. Only the known metadata
        // accumulation above is permitted; no unbounded chunk accumulator exists.
        let metadata_length = frame.metadata.len() - if frame.body == 0 { 0 } else { 4 };
        let skip = if self.decoder_continuation { 4 } else { 0 };
        let mut metadata = frame.metadata.slice_with_length(skip, metadata_length - skip);
        let mut decoded = decoder.decode(&mut metadata)?;
        // A zero-body batch returns before consuming the admitted next-prefix
        // lookahead. Consume it without a second batch construction.
        if !metadata.is_empty() && decoder.decode(&mut metadata)?.is_some() {
            return Err(invalid("IPC unexpected second metadata batch"));
        }
        self.decoder_continuation = frame.body == 0;
        let mut body = body;
        if decoded.is_none() { decoded = decoder.decode(&mut body)?; }
        let decoded = decoded.ok_or_else(|| invalid("IPC complete batch absent"))?;
        if !body.is_empty() { return Err(invalid("IPC unconsumed complete body")); }
        let batch = self.scope.attach_borrowed_reserved(decoded, &frame.grant)?;
        Ok(Some(StreamBatch { batch, working_owner: Arc::new(StreamOwnership {
            _schema: self.schema_owner.clone(), _working: AllocationLease::new(frame.grant),
        }) }))
    }
    /// Decode one fully preflighted message. Call until `None` to establish EOS.
    /// # Errors
    /// Representation, cancellation and pool failures retain their native causes.
    pub fn next_batch(&mut self) -> Result<Option<StreamBatch>, CanonError> { self.record(true) }
    /// Validate every record's framing/layout without materializing its arrays.
    /// # Errors
    /// Invalid/unsupported complete stream or resource/cancellation refusal.
    pub fn validate_all(&mut self) -> Result<(), CanonError> {
        while !self.ended { self.record(false)?; }
        Ok(())
    }
}

/// Scan all encapsulated-message metadata and actual spans without array decode.
/// The returned reader retains the admitted schema owner until its caller finishes.
/// # Errors
/// Malformed framing/schema/layout, unsupported policy, resource refusal or cancellation.
pub fn validate<R: Read + Seek>(input: R, bytes: u64, limits: Limits, pool: Arc<dyn MemoryPool>, cancel: CancellationToken) -> Result<StreamReader<R>, CanonError> {
    let mut stream = StreamReader::open(input, bytes, limits, pool, cancel, false)?;
    stream.validate_all()?;
    Ok(stream)
}

fn validate_record(batch: ipc::RecordBatch<'_>, schema: &arrow::datatypes::Schema, fields: usize, body: usize) -> Result<(), CanonError> {
    if batch.compression().is_some() || batch.variadicBufferCounts().is_some() {
        return Err(invalid("IPC compression/variadic buffers refused"));
    }
    let rows = usize::try_from(batch.length()).map_err(|_| invalid("negative IPC row count"))?;
    if rows > MAX_VALUES { return Err(invalid("IPC row/value bound")); }
    let nodes = batch.nodes().ok_or_else(|| invalid("IPC nodes absent"))?;
    let buffers = batch.buffers().ok_or_else(|| invalid("IPC buffers absent"))?;
    if nodes.len() != fields || buffers.len() > mul(fields, 3)? { return Err(invalid("IPC node/buffer count")); }
    let mut values = 0;
    for node in nodes {
        let n = usize::try_from(node.length()).map_err(|_| invalid("negative IPC node length"))?;
        let nulls = usize::try_from(node.null_count()).map_err(|_| invalid("negative IPC null count"))?;
        values = add(values, n)?;
        if values > MAX_VALUES || nulls > n { return Err(invalid("IPC node value/null bound")); }
    }
    let mut previous = 0;
    for buffer in buffers {
        let start = usize::try_from(buffer.offset()).map_err(|_| invalid("negative IPC buffer offset"))?;
        let length = usize::try_from(buffer.length()).map_err(|_| invalid("negative IPC buffer length"))?;
        let end = add(start, length)?;
        if start < previous || end > body { return Err(invalid("IPC buffer outside body/overlap")); }
        previous = end;
    }
    let nodes = nodes.iter().collect::<Vec<_>>();
    let buffers = buffers.iter().collect::<Vec<_>>();
    let (mut node, mut buffer) = (0, 0);
    for field in schema.fields() {
        super::ipc_admission::layout(field.data_type(), &nodes, &buffers, &mut node, &mut buffer, Some(rows))?;
    }
    if node != nodes.len() || buffer != buffers.len() { return Err(invalid("IPC extra layout nodes/buffers")); }
    Ok(())
}

fn alignment_copies(batch: ipc::RecordBatch<'_>, schema: &arrow::datatypes::Schema, body: &[u8]) -> Result<usize, CanonError> {
    fn field(ty: &DataType, buffers: &[&ipc::Buffer], index: &mut usize, body: &[u8]) -> Result<usize, CanonError> {
        if matches!(ty, DataType::Null) { return Ok(0); }
        let layout = arrow_data::layout(ty);
        *index += 1; // admitted validity buffer
        let mut copies = 0;
        for spec in layout.buffers {
            let buffer = buffers.get(*index).ok_or_else(|| invalid("IPC alignment buffer absent"))?;
            *index += 1;
            if let arrow_data::BufferSpec::FixedWidth { alignment, .. } = spec {
                let start = usize::try_from(buffer.offset()).map_err(|_| invalid("IPC alignment offset"))?;
                let len = usize::try_from(buffer.length()).map_err(|_| invalid("IPC alignment length"))?;
                if (body.as_ptr() as usize + start) % alignment != 0 {
                    // Arrow validates even empty typed-buffer addresses and
                    // realigns them too; retain a nonzero admitted copy marker.
                    copies = add(copies, rounded(len.max(1))?)?;
                }
            }
        }
        match ty {
            DataType::Struct(children) => for child in children { copies = add(copies, field(child.data_type(), buffers, index, body)?)?; },
            DataType::List(child) | DataType::LargeList(child) | DataType::FixedSizeList(child, _) | DataType::Map(child, _) => {
                copies = add(copies, field(child.data_type(), buffers, index, body)?)?;
            }
            _ => {}
        }
        Ok(copies)
    }
    let buffers = batch.buffers().ok_or_else(|| invalid("IPC alignment buffers absent"))?.iter().collect::<Vec<_>>();
    let mut index = 0;
    let mut copies = 0;
    for value in schema.fields() { copies = add(copies, field(value.data_type(), &buffers, &mut index, body)?)?; }
    Ok(copies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::{array::{Array, Float64Array, NullArray}, datatypes::{DataType, Field, Schema}, ipc::writer::StreamWriter, record_batch::RecordBatch};
    use std::io::Cursor;
    fn encode(batch: &RecordBatch) -> Vec<u8> {
        let mut writer = StreamWriter::try_new(Vec::new(), batch.schema().as_ref()).unwrap();
        writer.write(batch).unwrap(); writer.finish().unwrap(); writer.into_inner().unwrap()
    }
    fn fixture() -> Vec<u8> {
        let batch = RecordBatch::try_new(Arc::new(Schema::new(vec![Field::new("value", DataType::Float64, true)])),
            vec![Arc::new(Float64Array::from(vec![Some(-0.0), Some(f64::from_bits(0x7ff8_0000_0000_0042)), None]))]).unwrap();
        encode(&batch)
    }
    fn limits() -> Limits { Limits { message_bytes: 1 << 20, metadata_bytes: 65536 } }
    fn pool() -> Arc<dyn MemoryPool> { Arc::new(crate::GreedyMemoryPool::new(16 << 20)) }
    fn record_stream(ty: DataType, length: i64, nodes: &[ipc::FieldNode], buffers: &[ipc::Buffer], body_length: i64, body: &[u8], compressed: bool) -> Vec<u8> {
        let schema = Schema::new(vec![Field::new("value", ty, true)]);
        let mut out = StreamWriter::try_new(Vec::new(), &schema).unwrap().into_inner().unwrap();
        out.truncate(out.len() - 8); // replace the writer-produced EOS with the selected record
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let nodes = builder.create_vector(nodes);
        let buffers = builder.create_vector(buffers);
        let compression = compressed.then(|| ipc::BodyCompression::create(&mut builder, &ipc::BodyCompressionArgs::default()));
        let batch = ipc::RecordBatch::create(&mut builder, &ipc::RecordBatchArgs { length, nodes: Some(nodes), buffers: Some(buffers), compression, ..Default::default() });
        let message = ipc::Message::create(&mut builder, &ipc::MessageArgs { version: ipc::MetadataVersion::V5, header_type: ipc::MessageHeader::RecordBatch, header: Some(batch.as_union_value()), bodyLength: body_length, ..Default::default() });
        builder.finish(message, None);
        let mut metadata = builder.finished_data().to_vec();
        metadata.resize(metadata.len().div_ceil(8) * 8, 0);
        out.extend_from_slice(&[255; 4]);
        out.extend_from_slice(&(metadata.len() as i32).to_le_bytes());
        out.extend_from_slice(&metadata);
        out.extend_from_slice(body);
        out.extend_from_slice(&[255,255,255,255,0,0,0,0]);
        out
    }
    #[test]
    fn actual_misaligned_value_addresses_take_a_prepaid_copy() {
        let bits = [(-0.0f64).to_bits(), 0x7ff8_0000_0000_0042u64, 1];
        let mut body = vec![0];
        for value in bits { body.extend_from_slice(&value.to_le_bytes()); }
        body.resize(32, 0);
        let bytes = record_stream(DataType::Float64, 3, &[ipc::FieldNode::new(3, 0)],
            &[ipc::Buffer::new(0, 0), ipc::Buffer::new(1, 24)], 32, &body, false);
        let pool = pool();
        let mut stream = StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).unwrap();
        let decoded = stream.next_batch().unwrap().unwrap();
        let values = decoded.batch.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.values().iter().map(|value| value.to_bits()).collect::<Vec<_>>(), bits);
        assert_eq!(values.to_data().buffers()[0].as_ptr().align_offset(8), 0);
        drop(stream);
        assert!(pool.reserved() > 0, "batch retains schema and working grants after reader drops");
        drop(decoded); assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn empty_misaligned_typed_buffers_keep_the_existing_realignment_path() {
        let bytes = record_stream(DataType::Float64, 0, &[ipc::FieldNode::new(0, 0)],
            &[ipc::Buffer::new(0, 0), ipc::Buffer::new(1, 0)], 8, &[0; 8], false);
        let pool = pool();
        let mut stream = StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).unwrap();
        let decoded = stream.next_batch().unwrap().unwrap();
        assert_eq!(decoded.batch.num_rows(), 0);
        drop(stream); drop(decoded); assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn hostile_streams_are_refused_in_disposable_processes() {
        for case in ["huge-body", "negative-body", "span", "bitmap", "null-values", "compression"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "ipc_stream::tests::hostile_stream_child", "--nocapture"])
                .env("PSE_IPC_HOSTILE_CASE", case).output().unwrap();
            assert!(output.status.success(), "{case}: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    #[test]
    fn hostile_stream_child() {
        let Ok(case) = std::env::var("PSE_IPC_HOSTILE_CASE") else { return; };
        let node = [ipc::FieldNode::new(1, 0)];
        let valid = [ipc::Buffer::new(0, 0), ipc::Buffer::new(0, 8)];
        let bytes = match case.as_str() {
            "huge-body" => record_stream(DataType::Float64, 1, &node, &valid, 1 << 50, &[], false),
            "negative-body" => record_stream(DataType::Float64, 1, &node, &valid, -1, &[], false),
            "span" => record_stream(DataType::Float64, 1, &node, &[ipc::Buffer::new(0, 0), ipc::Buffer::new(1 << 40, 8)], 8, &[0; 8], false),
            "bitmap" => record_stream(DataType::Float64, 1, &[ipc::FieldNode::new(1, 1)], &valid, 8, &[0; 8], false),
            "null-values" => record_stream(DataType::Null, 1 << 40, &[ipc::FieldNode::new(1 << 40, 1 << 40)], &[], 0, &[], false),
            "compression" => record_stream(DataType::Float64, 1, &node, &valid, 8, &[0; 8], true),
            _ => panic!("unknown hostile fixture"),
        };
        let pool = pool();
        let result = StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new())
            .and_then(|mut stream| stream.next_batch());
        assert!(result.is_err(), "hostile message reached decode success");
        drop(result); assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn complete_messages_preserve_ieee_bits_and_escaped_buffers_keep_their_charge() {
        let bytes = fixture(); let pool = pool();
        let mut stream = StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).unwrap();
        let batch = stream.next_batch().unwrap().unwrap();
        assert!(stream.next_batch().unwrap().is_none());
        let values = batch.batch.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(values.value(1).to_bits(), 0x7ff8_0000_0000_0042);
        assert!(values.is_null(2));
        let escaped = values.to_data().buffers()[0].clone();
        drop(batch); drop(stream);
        assert!(pool.reserved() > 0);
        drop(escaped); assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn schema_only_scan_validates_every_frame_without_decoding_arrays() {
        let mut bytes = fixture(); let pool = pool();
        let stream = validate(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).unwrap();
        assert_eq!(stream.schema().fields().len(), 1); drop(stream); assert_eq!(pool.reserved(), 0);
        bytes.extend_from_slice(&[0]);
        assert!(validate(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).is_err());
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn advertised_lengths_and_truncation_refuse_before_large_reservations() {
        let pool = pool();
        for bytes in [vec![255,255,255,255,255,255,255,127], vec![255,255,255,255,255,255,255,255], fixture()[..16].to_vec()] {
            assert!(StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).is_err());
            assert_eq!(pool.reserved(), 0);
        }
        let bytes = fixture();
        assert!(StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64 + 1, limits(), pool.clone(), CancellationToken::new()).is_err());
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn zero_body_values_are_bounded_and_cancelled_work_does_not_allocate() {
        let batch = RecordBatch::try_new(Arc::new(Schema::new(vec![Field::new("null", DataType::Null, true)])), vec![Arc::new(NullArray::new(3))]).unwrap();
        let bytes = encode(&batch); let pool = pool();
        let mut stream = StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), CancellationToken::new()).unwrap();
        assert_eq!(stream.next_batch().unwrap().unwrap().batch.num_rows(), 3);
        assert!(stream.next_batch().unwrap().is_none()); drop(stream); assert_eq!(pool.reserved(), 0);
        let cancel = CancellationToken::new(); cancel.cancel();
        assert!(matches!(StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), pool.clone(), cancel), Err(CanonError::Cancelled)));
        assert_eq!(pool.reserved(), 0);
        let tiny: Arc<dyn MemoryPool> = Arc::new(crate::GreedyMemoryPool::new(1));
        assert!(StreamReader::try_new(Cursor::new(&bytes), bytes.len() as u64, limits(), tiny.clone(), CancellationToken::new()).is_err());
        assert_eq!(tiny.reserved(), 0);
    }
}
