// PROBE X1: the ExtensionTypeRegistry — can a pse.* type be registered and resolved?
// PROBE X2: machine-readable EXPLAIN — which formats does 55.1.0 actually accept?
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{ArrayRef, FixedSizeBinaryArray, Float64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use datafusion::execution::session_state::SessionStateBuilder;
use datafusion::prelude::SessionConfig;
use datafusion_common::Result;
use datafusion_common::types::DFExtensionType;
use datafusion_expr::registry::{ExtensionTypeRegistry, ExtensionTypeRegistration};

#[derive(Debug)]
struct PseSemanticId;
impl DFExtensionType for PseSemanticId {
    fn storage_type(&self) -> DataType { DataType::FixedSizeBinary(16) }
    fn serialize_metadata(&self) -> Option<String> { None }
}

fn sch() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::FixedSizeBinary(16), false).with_metadata(HashMap::from([
            ("ARROW:extension:name".to_string(), "pse.semantic_id".to_string())])),
        Field::new("v", DataType::Float64, true),
    ]))
}
fn bat() -> RecordBatch {
    let ids: Vec<[u8;16]> = (0..3).map(|i|{let mut b=[0u8;16]; b[15]=i as u8; b}).collect();
    RecordBatch::try_new(sch(), vec![
        Arc::new(FixedSizeBinaryArray::try_from_iter(ids.into_iter()).unwrap()) as ArrayRef,
        Arc::new(Float64Array::from(vec![Some(1.0),Some(2.0),None])) as ArrayRef]).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("== PROBE X1: ExtensionTypeRegistry — register and resolve a pse.* type");
    use datafusion_expr::registry::MemoryExtensionTypeRegistry;
    let reg = MemoryExtensionTypeRegistry::new_with_canonical_extension_types();
    let canonical: Vec<String> = reg.extension_type_registrations().iter()
        .map(|r| r.type_name().to_string()).collect();
    println!("   canonical types preloaded ({}): {:?}", canonical.len(), canonical);

    reg.add_extension_type_registration(ExtensionTypeRegistration::new_arc(
        "pse.semantic_id",
        |dt: &DataType, _md: Option<&str>| {
            if dt != &DataType::FixedSizeBinary(16) {
                return datafusion_common::plan_err!("pse.semantic_id requires FixedSizeBinary(16), got {dt}");
            }
            Ok(Arc::new(PseSemanticId) as Arc<dyn DFExtensionType>)
        }))?;
    println!("   after registering pse.semantic_id: {}", reg.extension_type_registrations().len());

    // Does it resolve from a Field carrying ARROW:extension:name?
    let f_ok = sch().field(0).clone();
    match reg.create_extension_type_for_field(&f_ok) {
        Ok(Some(t)) => println!("   resolve from good Field -> storage_type={:?}", t.storage_type()),
        Ok(None)    => println!("   resolve from good Field -> None (not resolved)"),
        Err(e)      => println!("   resolve from good Field -> Err {}", &format!("{e}")[..90]),
    }
    // And does it REJECT a field whose storage type is wrong?
    let f_bad = Field::new("id", DataType::Utf8, false).with_metadata(HashMap::from([
        ("ARROW:extension:name".to_string(), "pse.semantic_id".to_string())]));
    match reg.create_extension_type_for_field(&f_bad) {
        Ok(Some(_)) => println!("   wrong storage type      -> ACCEPTED (no validation!)"),
        Ok(None)    => println!("   wrong storage type      -> None"),
        Err(e)      => println!("   wrong storage type      -> REJECTED: {}", format!("{e}").lines().next().unwrap_or("")),
    }
    // An unregistered pse.* type
    let f_unk = Field::new("x", DataType::FixedSizeBinary(16), false).with_metadata(HashMap::from([
        ("ARROW:extension:name".to_string(), "pse.not_registered".to_string())]));
    match reg.create_extension_type_for_field(&f_unk) {
        Ok(Some(_)) => println!("   unregistered pse.* type -> resolved (unexpected)"),
        Ok(None)    => println!("   unregistered pse.* type -> None  (silent degradation to storage)"),
        Err(e)      => println!("   unregistered pse.* type -> Err {}", format!("{e}").lines().next().unwrap_or("")),
    }

    println!("\n== PROBE X2: machine-readable EXPLAIN formats accepted by 55.1.0");
    for fmt in ["indent","tree","pgjson","graphviz"] {
        let cfg = SessionConfig::new().set_str("datafusion.explain.format", fmt);
        let ctx = SessionContext::new_with_state(
            SessionStateBuilder::new().with_default_features().with_config(cfg).build());
        ctx.register_table("rel", Arc::new(MemTable::try_new(sch(), vec![vec![bat()]])?))?;
        match ctx.sql("EXPLAIN SELECT v FROM rel WHERE v > 1.0").await {
            Ok(df) => match df.collect().await {
                Ok(b) => {
                    let s = arrow::util::pretty::pretty_format_batches(&b).unwrap().to_string();
                    let first = s.lines().filter(|l| l.contains('|')).nth(2).unwrap_or("").trim().to_string();
                    println!("   {fmt:<9} -> OK   sample: {}", &first[..first.len().min(96)]);
                }
                Err(e) => println!("   {fmt:<9} -> exec error: {}", format!("{e}").lines().next().unwrap_or("")),
            },
            Err(e) => println!("   {fmt:<9} -> rejected:   {}", format!("{e}").lines().next().unwrap_or("")),
        }
    }
    println!("\n== PROBE X3: how does SessionConfig reject an invalid setting?");
    let r = std::panic::catch_unwind(|| {
        let _ = SessionConfig::new().set_str("datafusion.explain.format", "json");
    });
    println!("   set_str with an invalid value -> {}",
        if r.is_err() { "PANICS (not a typed error)" } else { "returned normally" });
    let mut opts = datafusion_common::config::ConfigOptions::new();
    match opts.set("datafusion.explain.format", "json") {
        Ok(_) => println!("   ConfigOptions::set              -> accepted (no validation)"),
        Err(e) => println!("   ConfigOptions::set              -> typed Err: {}",
            format!("{e}").lines().next().unwrap_or("")),
    }
    Ok(())
}
