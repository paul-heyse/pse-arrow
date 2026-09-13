// PROBE 1 — blake3: derive_key vs truncated finalize, and what §5.1's blake3_128 can mean.
// PROBE 2 — serde_arrow: does an extension-typed field survive a Rust-record round-trip,
//           and does STRATEGY_KEY land in field metadata (which §5.3 would then hash)?
use std::collections::HashMap;

fn hexn(b: &[u8], n: usize) -> String {
    b.iter().take(n).map(|x| format!("{x:02x}")).collect()
}

fn probe1() {
    println!("== PROBE 1: blake3 — the two routes to a 128-bit semantic ID");
    let material = b"pse:named:v1|pkg-01|flowsheet.heater.inlet";
    let context = "pse:named:v1";

    // Route A: plain hash, truncated to 16 bytes
    let plain = blake3::hash(material);
    let trunc = &plain.as_bytes()[..16];

    // Route B: derive_key with a domain-separation context, truncated to 16
    let derived = blake3::derive_key(context, material);
    let derived_trunc = &derived[..16];

    println!("   hash(material)[..16]        = {}", hexn(trunc, 16));
    println!("   derive_key(ctx, material)[..16] = {}", hexn(derived_trunc, 16));
    println!("   the two routes agree        : {}", trunc == derived_trunc);

    // Route C: the XOF — ask blake3 for exactly 16 bytes
    let mut xof = blake3::Hasher::new();
    xof.update(material);
    let mut out16 = [0u8; 16];
    xof.finalize_xof().fill(&mut out16);
    println!("   finalize_xof -> 16 bytes    = {}", hexn(&out16, 16));
    println!("   xof(16) == hash[..16]       : {}", out16 == trunc[..16]);

    // Is truncation a *defined* operation, or are we relying on an accident?
    let mut h2 = blake3::Hasher::new_derive_key(context);
    h2.update(material);
    let d2 = h2.finalize();
    println!("   Hasher::new_derive_key == derive_key : {}", d2.as_bytes() == &derived);

    // Does the context string actually separate domains?
    let other = blake3::derive_key("pse:symbol:v1", material);
    println!("   different context -> different bytes : {}", other != derived);
    println!("   NOTE: blake3 is an XOF, so any prefix of the output is itself a");
    println!("         valid-length digest; truncation to 16 bytes is defined, not ad hoc.");
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct Row {
    id: [u8; 16],
    v: f64,
    label: String,
}

fn probe2() {
    use serde_arrow::schema::{SchemaLike, TracingOptions};
    println!("== PROBE 2: serde_arrow — schema derivation and field metadata");
    let fields = match Vec::<arrow_schema::FieldRef>::from_type::<Row>(TracingOptions::default()) {
        Ok(f) => f,
        Err(e) => { println!("   from_type FAILED: {e}"); return; }
    };
    for f in &fields {
        let md: HashMap<_, _> = f.metadata().clone().into_iter().collect();
        println!("   field {:<6} type={:<24} metadata={:?}", f.name(), format!("{:?}", f.data_type()), md);
    }
    let rows = vec![
        Row { id: [0u8; 16], v: 1.0, label: "a".into() },
        Row { id: [1u8; 16], v: -0.0, label: "b".into() },
    ];
    match serde_arrow::to_record_batch(&fields, &rows) {
        Ok(batch) => {
            println!("   to_record_batch rows={} cols={}", batch.num_rows(), batch.num_columns());
            for f in batch.schema().fields() {
                if !f.metadata().is_empty() {
                    println!("   BATCH field {} carries metadata {:?}", f.name(), f.metadata());
                }
            }
            let has_strategy = batch.schema().fields().iter()
                .any(|f| f.metadata().keys().any(|k| k.contains("STRATEGY") || k.starts_with("SERDE_ARROW")));
            println!("   any STRATEGY-style key in field metadata : {has_strategy}");
            match serde_arrow::from_record_batch::<Vec<Row>>(&batch) {
                Ok(back) => println!("   round-trip equal : {}", back == rows),
                Err(e) => println!("   from_record_batch FAILED: {e}"),
            }
        }
        Err(e) => println!("   to_record_batch FAILED: {e}"),
    }
}

fn main() { probe1(); println!(); probe2(); }
