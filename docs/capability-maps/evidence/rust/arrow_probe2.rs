// Extends arrow_probe.rs. PROBE 6: does metadata key ORDER change the canonical
// IPC bytes that §5.3 hashes?  PROBE 7: Parquet metadata + extension survival and
// write determinism.  PROBE 8: LogicalPlan::Extension survival through the optimizer.
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{ArrayRef, Float64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::ipc::writer::{IpcWriteOptions, StreamWriter};

fn schema_md(n: usize) -> SchemaRef {
    let md: HashMap<String,String> = (0..n).map(|i| (format!("pse.k{i}"), format!("v{i}"))).collect();
    Arc::new(Schema::new(vec![
        Field::new("v", DataType::Float64, true).with_metadata(md.clone())
    ]).with_metadata(md))
}
fn batch(s: SchemaRef) -> RecordBatch {
    RecordBatch::try_new(s, vec![Arc::new(Float64Array::from(vec![Some(1.0),Some(2.0),None])) as ArrayRef]).unwrap()
}
fn ipc(s: &SchemaRef, b: &RecordBatch) -> Vec<u8> {
    let opts = IpcWriteOptions::try_new(8, false, arrow::ipc::MetadataVersion::V5).unwrap()
        .try_with_compression(None).unwrap();
    let mut buf = Vec::new();
    { let mut w = StreamWriter::try_new_with_options(&mut buf, s, opts).unwrap();
      w.write(b).unwrap(); w.finish().unwrap(); }
    buf
}

fn probe6() {
    println!("== PROBE 6: does field/schema metadata key order change the canonical IPC bytes?");
    println!("   (§5.3 hashes this encoding; each row is 6 independently built schemas)");
    for n in [0usize,1,2,3,5] {
        let mut set = std::collections::HashSet::new();
        let mut len = 0;
        for _ in 0..6 {
            let s = schema_md(n);
            let b = ipc(&s, &batch(s.clone()));
            len = b.len(); set.insert(b);
        }
        println!("   {n} metadata key(s): len={len:<5} distinct IPC encodings among 6 = {}  {}",
            set.len(), if set.len()==1 {"STABLE"} else {"UNSTABLE"});
    }
    // And the fix: build the metadata from a BTreeMap-sorted source
    let sorted_schema = || -> SchemaRef {
        let mut keys: Vec<(String,String)> = (0..5).map(|i|(format!("pse.k{i}"),format!("v{i}"))).collect();
        keys.sort();
        let md: std::collections::BTreeMap<String,String> = keys.into_iter().collect();
        let hm: HashMap<String,String> = md.into_iter().collect();
        Arc::new(Schema::new(vec![Field::new("v", DataType::Float64, true).with_metadata(hm.clone())]).with_metadata(hm))
    };
    let mut set = std::collections::HashSet::new();
    for _ in 0..6 { let s = sorted_schema(); set.insert(ipc(&s, &batch(s.clone()))); }
    println!("   5 keys inserted in sorted order: distinct = {}  {}", set.len(),
        if set.len()==1 {"still STABLE?" } else {"STILL UNSTABLE — sorting the input does not help" });
}

fn probe7() {
    use parquet::arrow::ArrowWriter;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::file::properties::WriterProperties;
    println!("== PROBE 7: Parquet — does pse.* / ARROW:extension:name metadata survive? deterministic bytes?");
    let s = Arc::new(Schema::new(vec![
        Field::new("id", DataType::FixedSizeBinary(16), false).with_metadata(
            [("ARROW:extension:name".to_string(),"pse.semantic_id".to_string())].into_iter().collect()),
        Field::new("v", DataType::Float64, true).with_metadata(
            [("pse.semantic.quantity_type".to_string(),"qt-demo".to_string())].into_iter().collect()),
    ]).with_metadata([("pse.contract.id".to_string(),"rel-1".to_string())].into_iter().collect()));
    let ids: Vec<[u8;16]> = (0..3).map(|i|{let mut b=[0u8;16]; b[15]=i as u8; b}).collect();
    let b = RecordBatch::try_new(s.clone(), vec![
        Arc::new(arrow::array::FixedSizeBinaryArray::try_from_iter(ids.into_iter()).unwrap()) as ArrayRef,
        Arc::new(Float64Array::from(vec![Some(1.0),None,Some(-0.0)])) as ArrayRef,
    ]).unwrap();
    let write = || -> Vec<u8> {
        let mut buf = Vec::new();
        { let mut w = ArrowWriter::try_new(&mut buf, s.clone(), Some(WriterProperties::builder().build())).unwrap();
          w.write(&b).unwrap(); w.close().unwrap(); }
        buf
    };
    let p1 = write(); let p2 = write();
    println!("   two writes of identical data: len={} identical={}", p1.len(), p1 == p2);
    let rdr = ParquetRecordBatchReaderBuilder::try_new(bytes::Bytes::from(p1.clone())).unwrap();
    let rs = rdr.schema().clone();
    println!("   schema md survives : {:?}", rs.metadata().get("pse.contract.id"));
    println!("   field md survives  : {:?}", rs.field(1).metadata().get("pse.semantic.quantity_type"));
    println!("   ext name survives  : {:?}", rs.field(0).extension_type_name());
    let mut r = rdr.build().unwrap();
    let rb = r.next().unwrap().unwrap();
    let col = rb.column(1).as_any().downcast_ref::<Float64Array>().unwrap();
    use arrow::array::Array;
    println!("   null preserved={} -0.0 sign preserved={}", col.is_null(1), col.value(2).is_sign_negative());
    // does created_by / writer version appear in the bytes?
    let hay = String::from_utf8_lossy(&p1);
    println!("   'created_by' writer string embedded in file: {}", hay.contains("parquet-rs"));
}

fn main() { probe6(); println!(); probe7(); }
