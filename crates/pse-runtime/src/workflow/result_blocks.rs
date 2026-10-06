// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent bounded IPC blocks. Registry schemas retain scientific meaning;
//! this layer supplies transport and refuses advertised allocations before decode.

use datafusion::arrow::{
    array::ArrayData,
    datatypes::{DataType, Schema, SchemaRef},
    ipc::{self, reader::StreamReader, writer::StreamWriter},
    record_batch::RecordBatch,
};
use std::io::{Cursor, Write};

/// One self-contained schema, batch and explicit end marker per database blob.
pub const RESULT_BLOCK_BYTES: usize = 512 * 1024;
const MAX_FIELDS: usize = 16 * 1024;
const MAX_DEPTH:usize=64;
const MAX_ROWS: usize = 32 * 1024;
const MAX_VALUES: usize = RESULT_BLOCK_BYTES;

// Source objects belong to canonical revision ingestion, whose declared object
// bound differs from admitted scientific result blobs. Callers cannot pick a
// larger limit for an arbitrary result read or write.
#[derive(Clone,Copy)]
enum Purpose { Result, CanonicalSource }
impl Purpose {
    const fn bytes(self)->usize {match self {Self::Result=>RESULT_BLOCK_BYTES,Self::CanonicalSource=>pse_operations::canonical::PAYLOAD_BYTES}}
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
    fn diagnostic_code(&self)->Option<pse_diagnostics::DiagnosticCode> {Some(pse_diagnostics::DiagnosticCode::ValidationInvariant)}
}

struct LimitedWriter {bytes:Vec<u8>,purpose:Purpose}
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > self.purpose.bytes() {
            return Err(std::io::Error::other("result block limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

/// Serialize a single registry-produced batch without compression or dictionaries.
/// The caller stages each returned blob individually rather than queueing them all.
pub fn encode_result_block(batch: &RecordBatch) -> Result<Vec<u8>, ResultBlockError> {
    encode_block(batch,Purpose::Result)
}

fn encode_block(batch:&RecordBatch,purpose:Purpose)->Result<Vec<u8>,ResultBlockError> {
    if batch.num_rows() > MAX_ROWS || batch.num_columns() > MAX_FIELDS {
        return Err(ResultBlockError::Invalid("batch shape exceeds bound"));
    }
    // Validate schema without allowing the writer to enter unsupported variants.
    let mut fields = 0usize;
    for field in batch.schema().fields() { trusted_shape(field.data_type(), 0, &mut fields)?; }
    // Bound writer scratch before IpcDataGenerator packs/copies the body. Sliced
    // arrays count their visible buffers rather than their retained parent buffers.
    let mut bytes = 0usize;
    for column in batch.columns() {
        bytes = bytes.checked_add(visible_size(&column.to_data())?)
            .ok_or(ResultBlockError::Invalid("array size overflow"))?;
    }
    if bytes > purpose.bytes() {
        return Err(ResultBlockError::Invalid("batch body exceeds bound"));
    }
    let mut writer = LimitedWriter {bytes:Vec::new(),purpose};
    {
        let mut stream = StreamWriter::try_new(&mut writer, batch.schema().as_ref())?;
        stream.write(batch)?;
        stream.finish()?;
    }
    preflight_block(&writer.bytes, batch.schema().as_ref(), batch.num_rows(),purpose)?;
    Ok(writer.bytes)
}

fn can_split(error:&ResultBlockError)->bool {
    matches!(error,ResultBlockError::Invalid("batch body exceeds bound"|"field value bound"))
        || matches!(error,ResultBlockError::Arrow(datafusion::arrow::error::ArrowError::IoError(message,_)) if message=="result block limit")
}

/// Split by rows, writing and consuming one bounded independent block at a time.
/// No prefix is a successful retained result until the attempt closes and seals it.
pub fn visit_result_blocks(
    batch: &RecordBatch,
    mut visitor: impl FnMut(usize, usize, Vec<u8>) -> Result<(), ResultBlockError>,
) -> Result<(), ResultBlockError> {
    visit_blocks(batch,&mut visitor,Purpose::Result)
}

/// Canonical revision inputs retain their exact source schema and rows under
/// the existing 3 MiB source-object bound, independently of result retention.
pub(super) fn visit_source_blocks(batch:&RecordBatch,mut visitor:impl FnMut(usize,usize,Vec<u8>)->Result<(),ResultBlockError>)->Result<(),ResultBlockError>{
    visit_blocks(batch,&mut visitor,Purpose::CanonicalSource)
}

fn visit_blocks(batch:&RecordBatch,visitor:&mut impl FnMut(usize,usize,Vec<u8>)->Result<(),ResultBlockError>,purpose:Purpose)->Result<(),ResultBlockError>{
    if batch.num_rows() == 0 { return visitor(0, 0, encode_block(batch,purpose)?); }
    let mut start = 0;
    while start < batch.num_rows() {
        let mut rows = (batch.num_rows() - start).min(MAX_ROWS);
        let payload = loop {
            match encode_block(&batch.slice(start, rows),purpose) {
                Ok(payload) => break payload,
                Err(error) if rows>1 && can_split(&error) => {
                    rows = rows.div_ceil(2);
                }
                Err(error) => return Err(error),
            }
        };
        visitor(start, rows, payload)?;
        start += rows;
    }
    Ok(())
}

/// Async ingestion visits one independently bounded blob at a time. Completion
/// of each visitor is backpressure; this helper never collects a trajectory's
/// encoded blobs or carries provisional database rows across an await.
pub async fn visit_result_blocks_async<F,Fut,E>(batch:&RecordBatch,mut visitor:F)->Result<(),E>
where F:FnMut(usize,usize,Vec<u8>)->Fut,
      Fut:Future<Output=Result<(),E>>,
      E:From<ResultBlockError>,
{
    visit_blocks_async(batch,&mut visitor,Purpose::Result).await
}

/// Backpressured canonical source-object ingestion with its fixed input bound.
pub(super) async fn visit_source_blocks_async<F,Fut,E>(batch:&RecordBatch,mut visitor:F)->Result<(),E>
where F:FnMut(usize,usize,Vec<u8>)->Fut,Fut:Future<Output=Result<(),E>>,E:From<ResultBlockError>{
    visit_blocks_async(batch,&mut visitor,Purpose::CanonicalSource).await
}

async fn visit_blocks_async<F,Fut,E>(batch:&RecordBatch,visitor:&mut F,purpose:Purpose)->Result<(),E>
where F:FnMut(usize,usize,Vec<u8>)->Fut,Fut:Future<Output=Result<(),E>>,E:From<ResultBlockError>{
    if batch.num_rows()==0 { return visitor(0,0,encode_block(batch,purpose)?).await; }
    let mut start=0;
    while start<batch.num_rows() {
        let mut rows=(batch.num_rows()-start).min(MAX_ROWS);
        let payload=loop {
            match encode_block(&batch.slice(start,rows),purpose) {
                Ok(payload)=>break payload,
                Err(error) if rows>1 && can_split(&error)=>rows=rows.div_ceil(2),
                Err(error)=>return Err(error.into()),
            }
        };
        visitor(start,rows,payload).await?;
        start+=rows;
    }
    Ok(())
}

fn visible_size(data: &ArrayData) -> Result<usize,ResultBlockError> {
    let bitmap=if data.nulls().is_some() { data.len().div_ceil(8) } else { 0 };
    let nested=match data.data_type() {
        DataType::List(_) | DataType::Map(_,_) => {
            let offsets=data.buffer::<i32>(0);
            let start=usize::try_from(offsets[0]).map_err(|_|ResultBlockError::Invalid("negative list offset"))?;
            let end=usize::try_from(offsets[data.len()]).map_err(|_|ResultBlockError::Invalid("negative list offset"))?;
            Some(bitmap+(data.len()+1)*4+visible_size(&data.child_data()[0].slice(start,end-start))?)
        }
        DataType::LargeList(_) => {
            let offsets=data.buffer::<i64>(0);
            let start=usize::try_from(offsets[0]).map_err(|_|ResultBlockError::Invalid("negative list offset"))?;
            let end=usize::try_from(offsets[data.len()]).map_err(|_|ResultBlockError::Invalid("negative list offset"))?;
            Some(bitmap+(data.len()+1)*8+visible_size(&data.child_data()[0].slice(start,end-start))?)
        }
        DataType::FixedSizeList(_,width) => Some(bitmap+visible_size(&data.child_data()[0].slice(data.offset()*(*width as usize),data.len()*(*width as usize)))?),
        DataType::Struct(_) => {
            let mut size=bitmap;
            for child in data.child_data() { size=size.checked_add(visible_size(child)?).ok_or(ResultBlockError::Invalid("visible size overflow"))?; }
            Some(size)
        }
        _ => None,
    };
    Ok(match nested { Some(size)=>size,None=>data.get_slice_memory_size()? })
}

fn trusted_shape(ty: &DataType, depth: usize, count: &mut usize) -> Result<(), ResultBlockError> {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_FIELDS { return Err(ResultBlockError::Invalid("schema complexity")); }
    match ty {
        DataType::Null | DataType::Boolean | DataType::Int8 | DataType::Int16 | DataType::Int32 |
        DataType::Int64 | DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
        DataType::Float16 | DataType::Float32 | DataType::Float64 | DataType::Utf8 | DataType::LargeUtf8 |
        DataType::Binary | DataType::LargeBinary | DataType::Date32 | DataType::Date64 |
        DataType::Time32(_) | DataType::Time64(_) | DataType::Timestamp(_, _) | DataType::Duration(_) |
        DataType::Decimal32(_, _) | DataType::Decimal64(_, _) | DataType::Decimal128(_, _) |
        DataType::Decimal256(_, _) => {},
        DataType::FixedSizeBinary(n) if *n > 0 && (*n as usize) <= RESULT_BLOCK_BYTES => {},
        DataType::List(child) | DataType::LargeList(child) | DataType::Map(child, _) => {
            trusted_shape(child.data_type(), depth + 1, count)?;
        }
        DataType::FixedSizeList(child, n) if *n > 0 && (*n as usize) <= MAX_VALUES => {
            trusted_shape(child.data_type(), depth + 1, count)?;
        }
        DataType::Struct(children) => {
            for child in children { trusted_shape(child.data_type(), depth + 1, count)?; }
        }
        _ => return Err(ResultBlockError::Invalid("unsupported schema type")),
    }
    Ok(())
}

// FlatBuffer access is borrowed and verified. In particular do not hand the
// length-prefixed stream to StreamReader before this complete pass.
fn frame<'a>(bytes: &'a [u8], offset: &mut usize,purpose:Purpose) -> Result<Option<(ipc::Message<'a>, &'a [u8])>, ResultBlockError> {
    let prefix = bytes.get(*offset..offset.saturating_add(8))
        .ok_or(ResultBlockError::Invalid("truncated message prefix"))?;
    if prefix[..4] != [255; 4] { return Err(ResultBlockError::Invalid("continuation marker required")); }
    let length = i32::from_le_bytes(prefix[4..8].try_into().map_err(|_| ResultBlockError::Invalid("metadata length"))?);
    *offset += 8;
    if length == 0 { return Ok(None); }
    let length = usize::try_from(length).map_err(|_| ResultBlockError::Invalid("negative metadata length"))?;
    if length > purpose.bytes() || length % 8 != 0 { return Err(ResultBlockError::Invalid("metadata bound/alignment")); }
    let metadata = bytes.get(*offset..offset.saturating_add(length)).ok_or(ResultBlockError::Invalid("truncated metadata"))?;
    let options=flatbuffers::VerifierOptions {max_depth:256,max_tables:128*1024,max_apparent_size:32*1024*1024,..Default::default()};
    let message = ipc::root_as_message_with_opts(&options,metadata).map_err(|_| ResultBlockError::Invalid("invalid FlatBuffer"))?;
    if message.version() != ipc::MetadataVersion::V5 { return Err(ResultBlockError::Invalid("unsupported IPC version")); }
    let body = usize::try_from(message.bodyLength()).map_err(|_| ResultBlockError::Invalid("negative body length"))?;
    if body > purpose.bytes() || body % 8 != 0 { return Err(ResultBlockError::Invalid("body bound/alignment")); }
    *offset += length;
    let body = bytes.get(*offset..offset.saturating_add(body)).ok_or(ResultBlockError::Invalid("truncated body"))?;
    *offset += body.len();
    Ok(Some((message, body)))
}

fn schema_field(field: ipc::Field<'_>, depth: usize, count: &mut usize,expected_fields:usize) -> Result<(), ResultBlockError> {
    *count += 1;
    if depth > MAX_DEPTH || *count > expected_fields || field.dictionary().is_some() || field.type_().is_none() {
        return Err(ResultBlockError::Invalid("schema complexity/dictionary/type"));
    }
    let children = field.children();
    let child_count = children.map_or(0, |c| c.len());
    let valid = match field.type_type() {
        ipc::Type::Null | ipc::Type::Bool | ipc::Type::Utf8 | ipc::Type::LargeUtf8 |
        ipc::Type::Binary | ipc::Type::LargeBinary => child_count == 0,
        ipc::Type::Int => field.type_as_int().is_some_and(|v| matches!(v.bitWidth(), 8 | 16 | 32 | 64)) && child_count == 0,
        ipc::Type::FloatingPoint => field.type_as_floating_point().is_some_and(|v| matches!(v.precision(), ipc::Precision::HALF | ipc::Precision::SINGLE | ipc::Precision::DOUBLE)) && child_count == 0,
        ipc::Type::FixedSizeBinary => field.type_as_fixed_size_binary().is_some_and(|v| v.byteWidth() > 0 && (v.byteWidth() as usize) <= RESULT_BLOCK_BYTES) && child_count == 0,
        ipc::Type::Date => field.type_as_date().is_some_and(|v| matches!(v.unit(), ipc::DateUnit::DAY | ipc::DateUnit::MILLISECOND)) && child_count == 0,
        ipc::Type::Time => field.type_as_time().is_some_and(|v| matches!((v.bitWidth(),v.unit()), (32,ipc::TimeUnit::SECOND | ipc::TimeUnit::MILLISECOND) | (64,ipc::TimeUnit::MICROSECOND | ipc::TimeUnit::NANOSECOND))) && child_count == 0,
        ipc::Type::Timestamp => field.type_as_timestamp().is_some_and(|v| time_unit(v.unit())) && child_count == 0,
        ipc::Type::Duration => field.type_as_duration().is_some_and(|v| time_unit(v.unit())) && child_count == 0,
        ipc::Type::Decimal => field.type_as_decimal().is_some_and(|v| matches!(v.bitWidth(), 32 | 64 | 128 | 256) && u8::try_from(v.precision()).is_ok() && i8::try_from(v.scale()).is_ok()) && child_count == 0,
        ipc::Type::List | ipc::Type::LargeList | ipc::Type::Map => child_count == 1,
        ipc::Type::FixedSizeList => field.type_as_fixed_size_list().is_some_and(|v| v.listSize() > 0 && (v.listSize() as usize) <= MAX_VALUES) && child_count == 1,
        ipc::Type::Struct_ => true,
        _ => false,
    };
    if !valid { return Err(ResultBlockError::Invalid("invalid or unsupported schema type")); }
    if let Some(children) = children { for child in children { schema_field(child, depth+1, count,expected_fields)?; } }
    Ok(())
}
fn time_unit(unit: ipc::TimeUnit) -> bool {
    matches!(unit, ipc::TimeUnit::SECOND | ipc::TimeUnit::MILLISECOND | ipc::TimeUnit::MICROSECOND | ipc::TimeUnit::NANOSECOND)
}

// Validate bitmap and data minima before decoder constructors (some Arrow
// constructors assert their buffer lengths). The nested node traversal is exactly
// the declared schema traversal; buffers cannot be borrowed from another field.
fn layout(ty: &DataType, nodes: &[&ipc::FieldNode], buffers: &[&ipc::Buffer], node: &mut usize, buffer: &mut usize, expected: Option<usize>) -> Result<(), ResultBlockError> {
    let field = nodes.get(*node).ok_or(ResultBlockError::Invalid("missing layout node"))?;
    *node += 1;
    let n = usize::try_from(field.length()).map_err(|_| ResultBlockError::Invalid("negative layout length"))?;
    if expected.is_some_and(|v| n != v) { return Err(ResultBlockError::Invalid("nested field length")); }
    if matches!(ty,DataType::Null) {
        if field.null_count() != field.length() { return Err(ResultBlockError::Invalid("null field shape")); }
        return Ok(());
    }
    let mut take = |minimum: usize| -> Result<(),ResultBlockError> {
        let value=buffers.get(*buffer).ok_or(ResultBlockError::Invalid("missing layout buffer"))?;
        *buffer += 1;
        if usize::try_from(value.length()).ok().is_none_or(|len| len < minimum) { return Err(ResultBlockError::Invalid("short layout buffer")); }
        Ok(())
    };
    take(if field.null_count()>0 { n.div_ceil(8) } else { 0 })?;
    match ty {
        DataType::Struct(children) => {
            for child in children { layout(child.data_type(),nodes,buffers,node,buffer,Some(n))?; }
        }
        DataType::FixedSizeList(child,width) => {
            let size=n.checked_mul(*width as usize).ok_or(ResultBlockError::Invalid("list size overflow"))?;
            if size>MAX_VALUES { return Err(ResultBlockError::Invalid("list size bound")); }
            layout(child.data_type(),nodes,buffers,node,buffer,Some(size))?;
        }
        DataType::List(child) | DataType::Map(child,_) => {
            take((n+1)*4)?;
            layout(child.data_type(),nodes,buffers,node,buffer,None)?;
        }
        DataType::LargeList(child) => {
            take((n+1)*8)?;
            layout(child.data_type(),nodes,buffers,node,buffer,None)?;
        }
        DataType::Utf8 | DataType::Binary => { take((n+1)*4)?; take(0)?; }
        DataType::LargeUtf8 | DataType::LargeBinary => { take((n+1)*8)?; take(0)?; }
        DataType::Boolean => take(n.div_ceil(8))?,
        DataType::FixedSizeBinary(width) => take(n.checked_mul(*width as usize).ok_or(ResultBlockError::Invalid("binary size overflow"))?)?,
        other => {
            let width=other.primitive_width().ok_or(ResultBlockError::Invalid("unsupported primitive layout"))?;
            take(n.checked_mul(width).ok_or(ResultBlockError::Invalid("primitive size overflow"))?)?;
        }
    }
    Ok(())
}

/// Verify the complete framing, exact schema, shape and allocation-advertising
/// metadata before StreamReader can resize a metadata/body buffer or build arrays.
pub fn preflight_result_block(bytes: &[u8], expected: &Schema, rows: usize) -> Result<(), ResultBlockError> {
    preflight_block(bytes,expected,rows,Purpose::Result)
}

fn preflight_block(bytes:&[u8],expected:&Schema,rows:usize,purpose:Purpose)->Result<(),ResultBlockError>{
    if bytes.len() > purpose.bytes() || rows > MAX_ROWS { return Err(ResultBlockError::Invalid("block/row bound")); }
    let mut expected_fields=0;for field in expected.fields(){trusted_shape(field.data_type(),0,&mut expected_fields)?;}
    let mut offset = 0;
    let (message, body) = frame(bytes, &mut offset,purpose)?.ok_or(ResultBlockError::Invalid("missing schema"))?;
    let schema = message.header_as_schema().ok_or(ResultBlockError::Invalid("schema must be first"))?;
    if !body.is_empty() || schema.endianness() != ipc::Endianness::Little { return Err(ResultBlockError::Invalid("schema body/endianness")); }
    let fields = schema.fields().ok_or(ResultBlockError::Invalid("missing schema fields"))?;
    if fields.len() != expected.fields().len() { return Err(ResultBlockError::Invalid("schema field bound")); }
    let mut field_count = 0;
    for field in fields { schema_field(field, 0, &mut field_count,expected_fields)?; }
    if field_count!=expected_fields{return Err(ResultBlockError::Invalid("declared schema node shape"));}
    // All converter panic domains, recursion and string sizes were bounded above.
    let decoded = ipc::convert::fb_to_schema(schema);
    if &decoded != expected { return Err(ResultBlockError::Invalid("registry schema mismatch")); }
    let (message, body) = frame(bytes, &mut offset,purpose)?.ok_or(ResultBlockError::Invalid("missing record batch"))?;
    let batch = message.header_as_record_batch().ok_or(ResultBlockError::Invalid("one record batch required"))?;
    if batch.compression().is_some() || batch.variadicBufferCounts().is_some() { return Err(ResultBlockError::Invalid("compression/variadic buffers refused")); }
    if usize::try_from(batch.length()).ok() != Some(rows) { return Err(ResultBlockError::Invalid("record row count")); }
    let nodes = batch.nodes().ok_or(ResultBlockError::Invalid("missing field nodes"))?;
    if nodes.len() != field_count { return Err(ResultBlockError::Invalid("field node count")); }
    let mut values = 0usize;
    for node in nodes {
        let length = usize::try_from(node.length()).map_err(|_| ResultBlockError::Invalid("negative field length"))?;
        let nulls = usize::try_from(node.null_count()).map_err(|_| ResultBlockError::Invalid("negative null count"))?;
        values = values.checked_add(length).ok_or(ResultBlockError::Invalid("field size overflow"))?;
        if length > MAX_VALUES || nulls > length || values > MAX_VALUES { return Err(ResultBlockError::Invalid("field value bound")); }
    }
    let buffers = batch.buffers().ok_or(ResultBlockError::Invalid("missing buffers"))?;
    if buffers.len() > field_count * 3 { return Err(ResultBlockError::Invalid("buffer count bound")); }
    let mut last = 0usize;
    for buffer in buffers {
        let start = usize::try_from(buffer.offset()).map_err(|_| ResultBlockError::Invalid("negative buffer offset"))?;
        let length = usize::try_from(buffer.length()).map_err(|_| ResultBlockError::Invalid("negative buffer length"))?;
        let end = start.checked_add(length).ok_or(ResultBlockError::Invalid("buffer overflow"))?;
        if start < last || end > body.len() { return Err(ResultBlockError::Invalid("buffer span/overlap")); }
        last = end;
    }
    let nodes=nodes.iter().collect::<Vec<_>>();
    let buffers=buffers.iter().collect::<Vec<_>>();
    let mut node=0; let mut buffer=0;
    for field in expected.fields() { layout(field.data_type(),&nodes,&buffers,&mut node,&mut buffer,Some(rows))?; }
    if node!=nodes.len() || buffer!=buffers.len() { return Err(ResultBlockError::Invalid("extra layout nodes/buffers")); }
    if frame(bytes, &mut offset,purpose)?.is_some() || offset != bytes.len() { return Err(ResultBlockError::Invalid("explicit EOS/trailing message required")); }
    Ok(())
}

/// Decode exactly one admitted blob, preserving IEEE bits and Arrow validity.
/// Scientific local validation remains the registry admission owner's obligation.
pub fn decode_result_block(bytes: &[u8], expected: SchemaRef, rows: usize) -> Result<RecordBatch, ResultBlockError> {
    decode_block(bytes,expected,rows,Purpose::Result)
}

fn decode_block(bytes:&[u8],expected:SchemaRef,rows:usize,purpose:Purpose)->Result<RecordBatch,ResultBlockError>{
    preflight_block(bytes, expected.as_ref(), rows,purpose)?;
    let mut stream = StreamReader::try_new(Cursor::new(bytes), None)?;
    let batch = stream.next().ok_or(ResultBlockError::Invalid("missing decoded batch"))??;
    if batch.schema() != expected || batch.num_rows() != rows || stream.next().is_some() {
        return Err(ResultBlockError::Invalid("decoded schema/shape mismatch"));
    }
    Ok(batch)
}

#[cfg(test)]
mod canonical_result_blocks_unit {
    use super::*;
    use datafusion::arrow::{array::{Array, Float64Array, UInt64Array}, datatypes::Field};
    use std::sync::Arc;
    #[test]
    fn admitted_physical_source_schemas_fit_canonical_input_transport() {
        let registry=pse_schema::registry().unwrap();
        let roots=crate::physical::input_keys(registry).iter().map(|key|registry.relation(&key.qualified_name()).unwrap().id).collect();
        let support=pse_schema::product::support_closure(registry,&roots).unwrap();
        for id in support {
            let spec=registry.relation_by_id(id).unwrap();
            let schema=pse_schema::arrow::relation_schema_ref(registry,spec).unwrap();
            let empty=RecordBatch::new_empty(schema);
            let bytes=encode_block(&empty,Purpose::CanonicalSource).unwrap_or_else(|error|panic!("physical source {}: {error}",spec.key.qualified_name()));
            assert!(bytes.len()<=pse_operations::canonical::PAYLOAD_BYTES);
            assert_eq!(decode_block(&bytes,empty.schema(),0,Purpose::CanonicalSource).unwrap().num_rows(),0);
            if bytes.len()>RESULT_BLOCK_BYTES {
                assert!(preflight_result_block(&bytes,empty.schema().as_ref(),0).is_err());
                assert!(encode_result_block(&empty).is_err());
            }
        }
    }
    #[tokio::test]
    async fn canonical_input_large_authored_schema_preserves_nonempty_rows_and_result_bound() {
        use pse_relations::generated::authored::modeling_declarations;
        let registry=pse_schema::registry().unwrap();
        let validation=pse_relations::validate::ValidationContext::local(registry).unwrap();
        let rows=pse_authoring::language::parse("package p { def Root { param a:Scalar=4; var x:Scalar; eq square:x*x==a; annotation start x(-0.0); } }",pse_ids::SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let mut builder=modeling_declarations::Builder::with_registry(registry,rows.len(),&validation).unwrap();
        for row in &rows {builder.push(row.clone()).unwrap();}
        let table=builder.finish().unwrap();let mut visited=0;
        // The source decoder uses the same strict borrowed preflight. Reserve
        // its bounded conversion work before any Arrow allocation is admitted.
        let pool:Arc<dyn pse_columnar::MemoryPool>=Arc::new(datafusion::execution::memory_pool::GreedyMemoryPool::new(64*1024*1024));
        let owner=pse_columnar::MemoryConsumer::new("test:canonical-source-decode").register(&pool);owner.try_grow(16*pse_operations::canonical::PAYLOAD_BYTES).unwrap();
        visit_source_blocks_async(table.batch(),|start,count,bytes| {
            assert_eq!(start,visited);visited+=count;
            let schema=table.batch().schema();let validation=&validation;let expected=&rows[start..start+count];
            async move {
                assert!(bytes.len()>RESULT_BLOCK_BYTES&&bytes.len()<=pse_operations::canonical::PAYLOAD_BYTES);
                assert!(decode_result_block(&bytes,schema.clone(),count).is_err());
                let decoded=decode_block(&bytes,schema,count,Purpose::CanonicalSource)?;
                let view=modeling_declarations::View::try_from_batch_with_registry(registry,&decoded,validation).unwrap();
                assert_eq!((0..view.len()).map(|index|view.row(index).unwrap()).collect::<Vec<_>>(),expected);
                assert!(preflight_block(&bytes[..bytes.len()-1],decoded.schema().as_ref(),count,Purpose::CanonicalSource).is_err());
                Ok::<(),ResultBlockError>(())
            }
        }).await.unwrap();
        assert_eq!(visited,rows.len());
    }
    #[test]
    fn generated_study_diagnostic_schema_and_nested_observations_round_trip() {
        use pse_relations::generated::{runtime::study_outcomes,enums::{StudyPointState,StudyEffectState}};
        use pse_model::diagnostic::{BoundaryDiagnostic,BoundaryClass,DiagnosticStage,DiagnosticRule,Observation,NonfiniteObservation};
        let registry=pse_schema::registry().unwrap();let validation=pse_relations::validate::ValidationContext::local(registry).unwrap();
        let mut diagnostic=BoundaryDiagnostic::new(BoundaryClass::Nonfinite,DiagnosticStage::Workflow,[],DiagnosticRule::WorkflowUnclassified);
        diagnostic.observations.insert("negative_zero".into(),Observation::Real(-0.0));diagnostic.observations.insert("nonfinite".into(),Observation::Nonfinite(NonfiniteObservation::PositiveInfinity));
        let mut child=BoundaryDiagnostic::new(BoundaryClass::InvalidModel,DiagnosticStage::Workflow,[],DiagnosticRule::WorkflowUnclassified);child.observations.insert("counter".into(),Observation::Integer((1i64<<53)+1));diagnostic.causes.push(child);
        let row=study_outcomes::Row{study_id:pse_operations::mint_id(),point_index:0,case_id:None,binding_hash:pse_ids::ContentHash::from_bytes([1;32]),state:StudyPointState::Failed,attempt_id:None,attempt_state:None,result_id:None,usable:false,seed_permission:false,candidate_use:None,effect:StudyEffectState::Absent,start:None,diagnostic:Some(super::super::diagnostic_rows::project_study_diagnostic(&diagnostic)),attempts:Vec::new()};
        let mut builder=study_outcomes::Builder::with_registry(registry,1,&validation).unwrap();builder.push(row.clone()).unwrap();let checked=builder.finish().unwrap();let mut fields=0;for field in checked.batch().schema().fields(){trusted_shape(field.data_type(),0,&mut fields).unwrap();}assert!(fields>256);
        let bytes=encode_result_block(checked.batch()).unwrap();assert!(bytes.len()<=RESULT_BLOCK_BYTES);
        let read=decode_result_block(&bytes,checked.batch().schema(),1).unwrap();let decoded=study_outcomes::View::try_from_batch_with_registry(registry,&read,&validation).unwrap().row(0).unwrap();assert_eq!(decoded,row);
        assert_eq!(decoded.diagnostic.as_ref().unwrap().observations.iter().find(|value|value.name=="negative_zero").unwrap().real.unwrap().to_bits(),(-0.0f64).to_bits());
        for name in ["runtime.solve_runs","runtime.study_outcomes"] {let spec=registry.relation(name).unwrap();let schema=pse_schema::arrow::relation_schema_ref(registry,spec).unwrap();let empty=RecordBatch::new_empty(schema);let bytes=encode_result_block(&empty).unwrap();assert_eq!(decode_result_block(&bytes,empty.schema(),0).unwrap().num_rows(),0);}
    }
    fn fixture() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new("value",DataType::Float64,true),Field::new("coordinate",DataType::UInt64,false)]));
        RecordBatch::try_new(schema,vec![Arc::new(Float64Array::from(vec![Some(-0.0),None,Some(f64::from_bits(0x7ff8_0000_0000_0042))])),Arc::new(UInt64Array::from(vec![0,1,u64::MAX]))]).unwrap()
    }
    #[test]
    fn ieee_bits_nulls_and_unsigned_coordinates_survive_independent_blocks() {
        let batch=fixture();
        let bytes=encode_result_block(&batch).unwrap();
        let read=decode_result_block(&bytes,batch.schema(),3).unwrap();
        let values=read.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.value(0).to_bits(),(-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert_eq!(values.value(2).to_bits(),0x7ff8_0000_0000_0042);
        assert_eq!(read.column(1).as_any().downcast_ref::<UInt64Array>().unwrap().value(2),u64::MAX);
        assert!(decode_result_block(&bytes,batch.schema(),2).is_err());
    }
    #[test]
    fn advertised_lengths_truncation_and_trailing_batches_are_refused_before_decode() {
        let batch=fixture();
        let bytes=encode_result_block(&batch).unwrap();
        let mut huge=bytes.clone(); huge[4..8].copy_from_slice(&i32::MAX.to_le_bytes());
        assert!(preflight_result_block(&huge,batch.schema().as_ref(),3).is_err());
        for cut in [0,4,8,bytes.len()-1,bytes.len()-8] { assert!(decode_result_block(&bytes[..cut],batch.schema(),3).is_err()); }
        let mut trailing=bytes.clone(); trailing.extend_from_slice(&[0;8]);
        assert!(decode_result_block(&trailing,batch.schema(),3).is_err());
        let wrong=Arc::new(Schema::new(vec![Field::new("other",DataType::Float64,true)]));
        assert!(decode_result_block(&bytes,wrong,3).is_err());
    }
    #[test]
    fn oversized_node_and_short_validity_buffers_are_refused_before_arrow_constructor() {
        let source=fixture();
        let bytes=encode_result_block(&source).unwrap();
        let (node_pos,buffer_pos)={
            let mut offset=0; frame(&bytes,&mut offset,Purpose::Result).unwrap();
            let (message,_)=frame(&bytes,&mut offset,Purpose::Result).unwrap().unwrap();
            let batch=message.header_as_record_batch().unwrap();
            (batch.nodes().unwrap().bytes().as_ptr() as usize-bytes.as_ptr() as usize,
             batch.buffers().unwrap().bytes().as_ptr() as usize-bytes.as_ptr() as usize)
        };
        let mut oversized=bytes.clone();
        oversized[node_pos..node_pos+8].copy_from_slice(&i64::MAX.to_le_bytes());
        assert!(preflight_result_block(&oversized,source.schema().as_ref(),3).is_err());
        let mut short=bytes;
        short[buffer_pos+8..buffer_pos+16].copy_from_slice(&0i64.to_le_bytes());
        assert!(preflight_result_block(&short,source.schema().as_ref(),3).is_err());
    }
    #[test]
    fn nested_lists_are_split_by_visible_children_and_preserve_exact_offsets() {
        use datafusion::arrow::{array::{ListArray,types::Float64Type},datatypes::Field};
        let array=ListArray::from_iter_primitive::<Float64Type,_,_>((0..30_000).map(|i|Some(vec![Some(i as f64),Some(-(i as f64))])));
        let schema=Arc::new(Schema::new(vec![Field::new("values",array.data_type().clone(),true)]));
        let source=RecordBatch::try_new(schema,vec![Arc::new(array)]).unwrap();
        let mut end=0;
        visit_result_blocks(&source,|start,rows,bytes| {
            assert_eq!(start,end);
            let result=decode_result_block(&bytes,source.schema(),rows)?;
            let values=result.column(0).as_any().downcast_ref::<ListArray>().unwrap().value(0);
            assert_eq!(values.as_any().downcast_ref::<Float64Array>().unwrap().value(0),start as f64);
            end+=rows; Ok(())
        }).unwrap();
        assert_eq!(end,30_000);
    }
    #[test]
    fn nested_struct_fixed_lists_preserve_sliced_children_and_validity() {
        use datafusion::arrow::array::{BooleanArray,FixedSizeListArray,StructArray};
        let child=Arc::new(Field::new("item",DataType::Float64,true));
        let lists=FixedSizeListArray::try_new(child,2,Arc::new(Float64Array::from_iter((0..60_000).map(|i|if i%5==0 {None} else {Some(i as f64)}))),None).unwrap();
        let fields=vec![Arc::new(Field::new("values",lists.data_type().clone(),false)),Arc::new(Field::new("flag",DataType::Boolean,false))];
        let structure=StructArray::try_new(fields.into(),vec![Arc::new(lists),Arc::new(BooleanArray::from((0..30_000).map(|i|i%2==0).collect::<Vec<_>>()))],None).unwrap();
        let schema=Arc::new(Schema::new(vec![Field::new("observation",structure.data_type().clone(),false)]));
        let source=RecordBatch::try_new(schema,vec![Arc::new(structure)]).unwrap().slice(7,29_980);
        let mut count=0;
        visit_result_blocks(&source,|start,rows,bytes| {
            let decoded=decode_result_block(&bytes,source.schema(),rows)?;
            let structure=decoded.column(0).as_any().downcast_ref::<StructArray>().unwrap();
            let lists=structure.column(0).as_any().downcast_ref::<FixedSizeListArray>().unwrap();
            let values=lists.value(0);let values=values.as_any().downcast_ref::<Float64Array>().unwrap();
            let first=(start+7)*2;
            assert_eq!(values.is_null(0),first%5==0);
            if first%5!=0 {assert_eq!(values.value(0),first as f64);}
            assert_eq!(structure.column(1).as_any().downcast_ref::<BooleanArray>().unwrap().value(0),(start+7)%2==0);
            count+=rows;Ok(())
        }).unwrap();
        assert_eq!(count,29_980);
    }
    #[test]
    fn compression_and_cross_block_dictionary_state_are_refused() {
        use datafusion::arrow::{array::{DictionaryArray,Int8Array,StringArray,types::Int8Type},ipc::writer::IpcWriteOptions};
        let source=fixture();
        let options=IpcWriteOptions::default().try_with_compression(Some(ipc::CompressionType::ZSTD)).unwrap();
        let mut compressed=Vec::new();
        { let mut writer=StreamWriter::try_new_with_options(&mut compressed,source.schema().as_ref(),options).unwrap();writer.write(&source).unwrap();writer.finish().unwrap(); }
        assert!(matches!(preflight_result_block(&compressed,source.schema().as_ref(),3),Err(ResultBlockError::Invalid("compression/variadic buffers refused"))));
        let dictionary=DictionaryArray::<Int8Type>::try_new(Int8Array::from(vec![0,1]),Arc::new(StringArray::from(vec!["x","y"]))).unwrap();
        let schema=Arc::new(Schema::new(vec![Field::new("label",dictionary.data_type().clone(),false)]));
        let batch=RecordBatch::try_new(schema,vec![Arc::new(dictionary)]).unwrap();
        let mut bytes=Vec::new();
        {let mut writer=StreamWriter::try_new(&mut bytes,batch.schema().as_ref()).unwrap();writer.write(&batch).unwrap();writer.finish().unwrap();}
        assert!(preflight_result_block(&bytes,batch.schema().as_ref(),2).is_err());
        assert!(encode_result_block(&batch).is_err());
    }
    #[tokio::test]
    async fn async_ingestion_stops_after_first_failed_bounded_visitor() {
        use std::sync::atomic::{AtomicUsize,Ordering};
        let schema=Arc::new(Schema::new(vec![Field::new("value",DataType::Float64,false)]));
        let batch=RecordBatch::try_new(schema,vec![Arc::new(Float64Array::from_iter_values((0..150_000).map(|i|i as f64)))]).unwrap();
        let visited=AtomicUsize::new(0);
        let result=visit_result_blocks_async(&batch,|_,_,bytes| {
            assert!(bytes.len()<=RESULT_BLOCK_BYTES);
            let ordinal=visited.fetch_add(1,Ordering::SeqCst);
            async move {if ordinal==1 { Err(ResultBlockError::Invalid("injected ingestion failure")) } else {Ok(())}}
        }).await;
        assert!(result.is_err());assert_eq!(visited.load(Ordering::SeqCst),2);
    }
    #[test]
    fn trajectory_blocks_are_bounded_contiguous_and_independently_readable() {
        let schema=Arc::new(Schema::new(vec![Field::new("value",DataType::Float64,false)]));
        let batch=RecordBatch::try_new(schema,vec![Arc::new(Float64Array::from_iter_values((0..150_000).map(|i| i as f64)))]).unwrap();
        let mut end=0; let mut count=0;
        visit_result_blocks(&batch,|start,rows,bytes| {
            assert_eq!(start,end); assert!(bytes.len()<=RESULT_BLOCK_BYTES);
            let read=decode_result_block(&bytes,batch.schema(),rows)?;
            assert_eq!(read.column(0).as_any().downcast_ref::<Float64Array>().unwrap().value(0),start as f64);
            end+=rows; count+=1; Ok(())
        }).unwrap();
        assert_eq!(end,150_000); assert!(count>1);
    }
}
