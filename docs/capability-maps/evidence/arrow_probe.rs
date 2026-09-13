use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{Array, ArrayRef, Float64Array, FixedSizeBinaryArray, RecordBatch};
use arrow::compute::{sort_to_indices, SortOptions};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::{IpcWriteOptions, StreamWriter};
use arrow::ipc::reader::StreamReader;

fn meta(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn schema() -> Schema {
    let f_id = Field::new("id", DataType::FixedSizeBinary(16), false).with_metadata(meta(&[
        ("ARROW:extension:name", "pse.semantic_id"),
        ("pse.semantic.role", "key"),
    ]));
    let f_v = Field::new("v", DataType::Float64, true).with_metadata(meta(&[
        ("pse.semantic.logical_type", "f64"),
        ("pse.semantic.quantity_type", "qt-demo"),
    ]));
    Schema::new_with_metadata(
        vec![f_id, f_v],
        meta(&[("pse.contract.id", "rel-1"), ("pse.contract.version", "1")]),
    )
}

fn batch(vals: Vec<Option<f64>>, n: usize) -> RecordBatch {
    let ids: Vec<[u8; 16]> = (0..n).map(|i| { let mut b=[0u8;16]; b[15]=i as u8; b }).collect();
    let idarr = FixedSizeBinaryArray::try_from_iter(ids.into_iter()).unwrap();
    RecordBatch::try_new(
        Arc::new(schema()),
        vec![Arc::new(idarr) as ArrayRef, Arc::new(Float64Array::from(vals)) as ArrayRef],
    ).unwrap()
}

fn ipc_bytes(batches: &[RecordBatch], align: usize) -> Vec<u8> {
    let opts = IpcWriteOptions::try_new(align, false, arrow::ipc::MetadataVersion::V5)
        .unwrap()
        .try_with_compression(None)
        .unwrap();
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut w = StreamWriter::try_new_with_options(&mut buf, &schema(), opts).unwrap();
        for b in batches { w.write(b).unwrap(); }
        w.finish().unwrap();
    }
    buf
}

fn main() {
    println!("== PROBE 1: schema + field metadata through IPC round-trip");
    let b = batch(vec![Some(1.0), None, Some(-0.0)], 3);
    let bytes = ipc_bytes(&[b.clone()], 8);
    let mut rdr = StreamReader::try_new(std::io::Cursor::new(bytes.clone()), None).unwrap();
    let rt = rdr.next().unwrap().unwrap();
    let s = rt.schema();
    println!("   schema md preserved : {:?}", s.metadata().get("pse.contract.id"));
    println!("   field md preserved  : {:?}", s.field(1).metadata().get("pse.semantic.quantity_type"));
    println!("   ext name preserved  : {:?}", s.field(0).extension_type_name());

    println!("== PROBE 2: null vs NaN vs -0.0 distinctness after round-trip");
    let col = rt.column(1).as_any().downcast_ref::<Float64Array>().unwrap();
    println!("   is_null(1)={} value(2)={} signbit(2)={}",
        col.is_null(1), col.value(2), col.value(2).is_sign_negative());

    println!("== PROBE 3: totalOrder — does sort separate -0.0 from +0.0, and place NaN?");
    let a = Float64Array::from(vec![Some(0.0), Some(-0.0), Some(f64::NAN), Some(-1.0), None]);
    let idx = sort_to_indices(&a, Some(SortOptions{descending:false, nulls_first:true}), None).unwrap();
    let order: Vec<String> = idx.values().iter().map(|i| {
        let i = *i as usize;
        if a.is_null(i) { "null".into() }
        else { let v=a.value(i); format!("{}{}", if v.is_sign_negative() {"-"} else {""}, if v.is_nan(){"NaN".into()} else {v.abs().to_string()}) }
    }).collect();
    println!("   sorted: {:?}", order);

    println!("== PROBE 4: batch splitting — do different splits give identical IPC bytes?");
    let whole = batch(vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)], 4);
    let p1 = whole.slice(0, 2);
    let p2 = whole.slice(2, 2);
    let one = ipc_bytes(&[whole.clone()], 8);
    let two = ipc_bytes(&[p1, p2], 8);
    let cat = arrow::compute::concat_batches(&Arc::new(schema()), &[whole.slice(0,2), whole.slice(2,2)]).unwrap();
    let catb = ipc_bytes(&[cat], 8);
    println!("   1-batch len={} 2-batch len={} concat-then-write len={}", one.len(), two.len(), catb.len());
    println!("   1-batch == 2-batch      : {}", one == two);
    println!("   1-batch == concat-first : {}", one == catb);

    println!("== PROBE 5: alignment affects bytes?");
    let a8 = ipc_bytes(&[whole.clone()], 8);
    let a64 = ipc_bytes(&[whole.clone()], 64);
    println!("   align8 len={} align64 len={} equal={}", a8.len(), a64.len(), a8 == a64);
}
