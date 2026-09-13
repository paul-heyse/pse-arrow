// Hypothesis: proto plan bytes are unstable when a Field carries >1 metadata key,
// because Field metadata is a HashMap and proto serialises it in iteration order.
use std::collections::HashMap;
use std::sync::Arc;
use arrow::array::{ArrayRef, Float64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use datafusion_catalog::TableProvider;
use datafusion_common::Result;
use datafusion_proto::bytes::logical_plan_to_bytes_with_extension_codec;

fn schema_with(n: usize) -> SchemaRef {
    let md: HashMap<String,String> = (0..n)
        .map(|i| (format!("pse.k{i}"), format!("v{i}"))).collect();
    Arc::new(Schema::new(vec![Field::new("v", DataType::Float64, true).with_metadata(md)]))
}
fn batch(s: SchemaRef) -> RecordBatch {
    RecordBatch::try_new(s, vec![Arc::new(Float64Array::from(vec![Some(1.0),Some(2.0)])) as ArrayRef]).unwrap()
}
#[derive(Debug)] struct C(SchemaRef);
impl datafusion_proto::logical_plan::LogicalExtensionCodec for C {
    fn try_decode(&self,_b:&[u8],_i:&[datafusion_expr::LogicalPlan],_c:&datafusion_execution::TaskContext)->Result<datafusion_expr::logical_plan::Extension>{datafusion_common::not_impl_err!("n/a")}
    fn try_encode(&self,_n:&datafusion_expr::logical_plan::Extension,_b:&mut Vec<u8>)->Result<()>{datafusion_common::not_impl_err!("n/a")}
    fn try_decode_table_provider(&self,_b:&[u8],_t:&datafusion_common::TableReference,_s:SchemaRef,_c:&datafusion_execution::TaskContext)->Result<Arc<dyn TableProvider>>{
        Ok(Arc::new(MemTable::try_new(self.0.clone(), vec![vec![batch(self.0.clone())]])?))}
    fn try_encode_table_provider(&self,t:&datafusion_common::TableReference,_n:Arc<dyn TableProvider>,b:&mut Vec<u8>)->Result<()>{
        b.extend_from_slice(t.table().as_bytes()); Ok(())}
}
async fn enc(n: usize) -> Result<Vec<u8>> {
    let s = schema_with(n);
    let ctx = SessionContext::new();
    ctx.register_table("rel", Arc::new(MemTable::try_new(s.clone(), vec![vec![batch(s.clone())]])?))?;
    let p = ctx.sql("SELECT v FROM rel").await?.into_optimized_plan()?;
    Ok(logical_plan_to_bytes_with_extension_codec(&p, &C(s))?.to_vec())
}
#[tokio::main]
async fn main() -> Result<()> {
    println!("== proto plan-byte stability vs number of field-metadata keys");
    println!("   (each row: 6 independent encodings of the same query/schema)");
    for n in [0usize,1,2,3,5] {
        let mut set = std::collections::HashSet::new();
        let mut len = 0;
        for _ in 0..6 { let b = enc(n).await?; len = b.len(); set.insert(b); }
        println!("   {n} metadata key(s): len={len:<5} distinct encodings among 6 runs = {}  {}",
            set.len(), if set.len()==1 {"STABLE"} else {"UNSTABLE"});
    }
    println!("\n   Same process, same Schema object reused (not rebuilt):");
    let s = schema_with(3);
    let mut set = std::collections::HashSet::new();
    for _ in 0..6 {
        let ctx = SessionContext::new();
        ctx.register_table("rel", Arc::new(MemTable::try_new(s.clone(), vec![vec![batch(s.clone())]])?))?;
        let p = ctx.sql("SELECT v FROM rel").await?.into_optimized_plan()?;
        set.insert(logical_plan_to_bytes_with_extension_codec(&p,&C(s.clone()))?.to_vec());
    }
    println!("   3 keys, one shared Schema: distinct = {}  {}", set.len(),
        if set.len()==1 {"STABLE (instability is per-HashMap-instance)"} else {"UNSTABLE"});
    Ok(())
}
