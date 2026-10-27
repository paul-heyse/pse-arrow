//! Hostile Arrow IPC stream probe (arrow-ipc =59.3.0).
//!
//! `gen <dir>`            write crafted stream cases into <dir>
//! `reader <file>`        decode with `StreamReader::try_new(File, None)`
//! `decoder <file> [n]`   push the bytes into `StreamDecoder::decode` in chunks of n (default: all)
//!
//! A logging global allocator reports every request of at least 1 MiB on stderr (raw
//! `write(2)`, no allocation) before it reaches the system allocator, so the attempted size is
//! visible even when the process subsequently aborts.

use std::alloc::{GlobalAlloc, Layout, System};
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;

use arrow_array::{Int64Array, RecordBatch};
use arrow_ipc::reader::{StreamDecoder, StreamReader};
use arrow_ipc::writer::StreamWriter;
use arrow_ipc::{FieldNode, MessageBuilder, MessageHeader, MetadataVersion, RecordBatchBuilder};
use arrow_schema::{DataType, Field, Schema};

struct Logging;

const REPORT_AT: usize = 1 << 20;

fn report(kind: &str, size: usize) {
    let mut buf = [0u8; 96];
    let mut cur = std::io::Cursor::new(&mut buf[..]);
    let _ = write!(cur, "ALLOC {kind} {size} bytes\n");
    let n = cur.position() as usize;
    unsafe {
        libc::write(2, buf.as_ptr().cast(), n);
    }
}

unsafe impl GlobalAlloc for Logging {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if l.size() >= REPORT_AT {
            report("alloc", l.size());
        }
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        if l.size() >= REPORT_AT {
            report("alloc_zeroed", l.size());
        }
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if n >= REPORT_AT {
            report("realloc", n);
        }
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static GLOBAL: Logging = Logging;

/// A message as framed in the stream: metadata flatbuffer bytes and body bytes.
struct Framed {
    meta: Vec<u8>,
    body: Vec<u8>,
}

fn split_stream(bytes: &[u8]) -> Vec<Framed> {
    let mut out = Vec::new();
    let mut pos = 0;
    loop {
        assert_eq!(&bytes[pos..pos + 4], &[0xff; 4]);
        let len = i32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        pos += 8;
        if len == 0 {
            break;
        }
        let meta = bytes[pos..pos + len].to_vec();
        pos += len;
        let msg = arrow_ipc::root_as_message(&meta).unwrap();
        let body_len = msg.bodyLength() as usize;
        let body = bytes[pos..pos + body_len].to_vec();
        pos += body_len;
        out.push(Framed { meta, body });
    }
    out
}

struct BatchEdit {
    body_length: Option<i64>,
    length: Option<i64>,
    nodes: Option<Vec<FieldNode>>,
    buffers: Option<Vec<arrow_ipc::Buffer>>,
}

fn rebuild_batch(meta: &[u8], edit: &BatchEdit) -> Vec<u8> {
    let msg = arrow_ipc::root_as_message(meta).unwrap();
    let rb = msg.header_as_record_batch().unwrap();
    let nodes: Vec<FieldNode> = edit
        .nodes
        .clone()
        .unwrap_or_else(|| rb.nodes().unwrap().iter().copied().collect());
    let buffers: Vec<arrow_ipc::Buffer> = edit
        .buffers
        .clone()
        .unwrap_or_else(|| rb.buffers().unwrap().iter().copied().collect());
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let nodes = fbb.create_vector(&nodes);
    let buffers = fbb.create_vector(&buffers);
    let mut b = RecordBatchBuilder::new(&mut fbb);
    b.add_length(edit.length.unwrap_or(rb.length()));
    b.add_nodes(nodes);
    b.add_buffers(buffers);
    let root = b.finish().as_union_value();
    let mut m = MessageBuilder::new(&mut fbb);
    m.add_version(MetadataVersion::V5);
    m.add_header_type(MessageHeader::RecordBatch);
    m.add_bodyLength(edit.body_length.unwrap_or(msg.bodyLength()));
    m.add_header(root);
    let m = m.finish();
    fbb.finish(m, None);
    fbb.finished_data().to_vec()
}

fn frame(out: &mut Vec<u8>, meta: &[u8], body: &[u8]) {
    let mut meta = meta.to_vec();
    while meta.len() % 8 != 0 {
        meta.push(0);
    }
    out.extend_from_slice(&[0xff; 4]);
    out.extend_from_slice(&(meta.len() as i32).to_le_bytes());
    out.extend_from_slice(&meta);
    out.extend_from_slice(body);
}

fn eos(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0]);
}

fn generate(dir: &Path) {
    let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(Int64Array::from(vec![1_i64, 2, 3, 4]))],
    )
    .unwrap();
    let mut valid = Vec::new();
    {
        let mut w = StreamWriter::try_new(&mut valid, &schema).unwrap();
        w.write(&batch).unwrap();
        w.finish().unwrap();
    }
    let parts = split_stream(&valid);
    assert_eq!(parts.len(), 2);
    let (schema_msg, batch_msg) = (&parts[0], &parts[1]);
    {
        let m = arrow_ipc::root_as_message(&batch_msg.meta).unwrap();
        let rb = m.header_as_record_batch().unwrap();
        eprintln!(
            "valid batch: bodyLength={} length={} nodes={:?} buffers={:?}",
            m.bodyLength(),
            rb.length(),
            rb.nodes().unwrap().iter().collect::<Vec<_>>(),
            rb.buffers().unwrap().iter().collect::<Vec<_>>()
        );
    }
    let none = || BatchEdit { body_length: None, length: None, nodes: None, buffers: None };
    let mut cases: Vec<(&str, BatchEdit)> = vec![
        ("valid_rebuilt", none()),
        ("body_len_2p50", BatchEdit { body_length: Some(1 << 50), ..none() }),
        ("body_len_neg1", BatchEdit { body_length: Some(-1), ..none() }),
        ("body_len_8gib", BatchEdit { body_length: Some(1 << 33), ..none() }),
        ("body_len_256gib", BatchEdit { body_length: Some(1 << 38), ..none() }),
        (
            "buffer_len_2p40",
            BatchEdit {
                buffers: Some(vec![
                    arrow_ipc::Buffer::new(0, 0),
                    arrow_ipc::Buffer::new(0, 1 << 40),
                ]),
                ..none()
            },
        ),
        (
            "buffer_offset_2p40",
            BatchEdit {
                buffers: Some(vec![
                    arrow_ipc::Buffer::new(0, 0),
                    arrow_ipc::Buffer::new(1 << 40, 32),
                ]),
                ..none()
            },
        ),
        (
            "node_len_2p40",
            BatchEdit {
                length: Some(1 << 40),
                nodes: Some(vec![FieldNode::new(1 << 40, 0)]),
                ..none()
            },
        ),
        (
            "node_len_neg1",
            BatchEdit {
                length: Some(-1),
                nodes: Some(vec![FieldNode::new(-1, 0)]),
                ..none()
            },
        ),
    ];
    // Node/buffer *count* far larger than the body: 200k nodes and 400k buffer entries
    // (first entries genuine, the rest pointing far outside the 32-byte body).
    {
        let m = arrow_ipc::root_as_message(&batch_msg.meta).unwrap();
        let rb = m.header_as_record_batch().unwrap();
        let mut nodes: Vec<FieldNode> = rb.nodes().unwrap().iter().copied().collect();
        let mut buffers: Vec<arrow_ipc::Buffer> = rb.buffers().unwrap().iter().copied().collect();
        nodes.extend(std::iter::repeat_n(FieldNode::new(1 << 40, 0), 200_000));
        buffers.extend(std::iter::repeat_n(arrow_ipc::Buffer::new(1 << 40, 1 << 40), 400_000));
        cases.push((
            "entry_count_huge",
            BatchEdit { nodes: Some(nodes), buffers: Some(buffers), ..none() },
        ));
    }
    for (name, edit) in &cases {
        let meta = rebuild_batch(&batch_msg.meta, edit);
        let mut out = Vec::new();
        frame(&mut out, &schema_msg.meta, &schema_msg.body);
        frame(&mut out, &meta, &batch_msg.body);
        eos(&mut out);
        fs::write(dir.join(format!("{name}.arrows")), &out).unwrap();
        eprintln!("wrote {name}: {} bytes", out.len());
    }
    // Schema message whose own bodyLength lies (StreamReader::try_new reads it).
    for (name, body_length) in [("schema_body_len_2p50", 1_i64 << 50), ("schema_body_len_neg1", -1)] {
        // Rebuild the schema message with the same schema and a lying bodyLength.
        let msg = arrow_ipc::root_as_message(&schema_msg.meta).unwrap();
        let mut fbb = flatbuffers::FlatBufferBuilder::new();
        let fb_schema = arrow_ipc::convert::schema_to_fb_offset(
            &mut fbb,
            &arrow_ipc::convert::fb_to_schema(msg.header_as_schema().unwrap()),
        );
        let mut m = MessageBuilder::new(&mut fbb);
        m.add_version(MetadataVersion::V5);
        m.add_header_type(MessageHeader::Schema);
        m.add_bodyLength(body_length);
        m.add_header(fb_schema.as_union_value());
        let m = m.finish();
        fbb.finish(m, None);
        let meta = fbb.finished_data().to_vec();
        let mut out = Vec::new();
        let check = arrow_ipc::root_as_message(&meta).unwrap();
        assert_eq!(check.bodyLength(), body_length);
        frame(&mut out, &meta, &[]);
        frame(&mut out, &batch_msg.meta, &batch_msg.body);
        eos(&mut out);
        fs::write(dir.join(format!("{name}.arrows")), &out).unwrap();
        eprintln!("wrote {name}: {} bytes", out.len());
    }
    // Metadata length prefix 0x7FFF_FFF0 (~2 GiB) after a valid schema, with 16 bytes following.
    let mut out = Vec::new();
    frame(&mut out, &schema_msg.meta, &schema_msg.body);
    out.extend_from_slice(&[0xff; 4]);
    out.extend_from_slice(&0x7fff_fff0_i32.to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);
    fs::write(dir.join("meta_len_2gib.arrows"), &out).unwrap();
    eprintln!("wrote meta_len_2gib: {} bytes", out.len());
}

fn run_reader(path: &Path) {
    let file = fs::File::open(path).unwrap();
    let reader = match StreamReader::try_new(file, None) {
        Ok(r) => r,
        Err(e) => {
            println!("OUTCOME typed-error at try_new: {e}");
            return;
        }
    };
    let mut rows = 0;
    for item in reader {
        match item {
            Ok(b) => rows += b.num_rows(),
            Err(e) => {
                println!("OUTCOME typed-error after {rows} rows: {e}");
                return;
            }
        }
    }
    println!("OUTCOME ok rows={rows}");
}

/// Mirror of a schema-only header read: `StreamReader::try_new(file.take(meta_len + 8))`.
fn run_schema_take(path: &Path) {
    use std::io::Read as _;
    let bytes = fs::read(path).unwrap();
    let len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as u64;
    let file = fs::File::open(path).unwrap().take(len + 8);
    match StreamReader::try_new(file, None) {
        Ok(r) => println!("OUTCOME ok schema fields={}", r.schema().fields().len()),
        Err(e) => println!("OUTCOME typed-error at try_new: {e}"),
    }
}

fn run_decoder(path: &Path, chunk: Option<usize>) {
    let bytes = fs::read(path).unwrap();
    let chunk = chunk.unwrap_or(bytes.len()).max(1);
    let mut dec = StreamDecoder::new();
    let mut rows = 0;
    for piece in bytes.chunks(chunk) {
        let mut buf = arrow_buffer::Buffer::from(piece.to_vec());
        while !buf.is_empty() {
            match dec.decode(&mut buf) {
                Ok(Some(b)) => rows += b.num_rows(),
                Ok(None) => {}
                Err(e) => {
                    println!("OUTCOME typed-error after {rows} rows: {e}");
                    return;
                }
            }
        }
    }
    match dec.finish() {
        Ok(()) => println!("OUTCOME ok rows={rows} finish=Ok"),
        Err(e) => println!("OUTCOME typed-error at finish after {rows} rows: {e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("gen") => generate(Path::new(&args[2])),
        Some("reader") => run_reader(Path::new(&args[2])),
        Some("schema-take") => run_schema_take(Path::new(&args[2])),
        Some("decoder") => run_decoder(Path::new(&args[2]), args.get(3).map(|s| s.parse().unwrap())),
        _ => panic!("usage: gen <dir> | reader <file> | decoder <file> [chunk]"),
    }
}
