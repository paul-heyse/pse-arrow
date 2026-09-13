// PROBE B: does a UserDefinedLogicalNode carrying a rule_id survive the optimizer,
// reach EXPLAIN, and reach execution?  Register #26 / blueprint F8 depend on this.
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use arrow::array::{ArrayRef, Float64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use datafusion_common::{DFSchemaRef, Result};
use datafusion_expr::{Expr, LogicalPlan, Extension};
use datafusion_expr::logical_plan::UserDefinedLogicalNodeCore;

#[derive(PartialEq, Eq, Hash, Debug)]
struct RuleNode { rule_id: String, input: LogicalPlan, schema: DFSchemaRef }

impl PartialOrd for RuleNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.rule_id.partial_cmp(&other.rule_id)
    }
}

impl UserDefinedLogicalNodeCore for RuleNode {
    fn name(&self) -> &str { "PseRule" }
    fn inputs(&self) -> Vec<&LogicalPlan> { vec![&self.input] }
    fn schema(&self) -> &DFSchemaRef { &self.schema }
    fn expressions(&self) -> Vec<Expr> { vec![] }
    fn fmt_for_explain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PseRule: rule_id={}", self.rule_id)
    }
    fn with_exprs_and_inputs(&self, _e: Vec<Expr>, mut i: Vec<LogicalPlan>) -> Result<Self> {
        Ok(RuleNode { rule_id: self.rule_id.clone(), input: i.remove(0), schema: self.schema.clone() })
    }
}

fn sch() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("v", DataType::Float64, true)
        .with_metadata(HashMap::from([("pse.semantic.role".to_string(),"value".to_string())]))]))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("== PROBE B: UserDefinedLogicalNode survival through the optimizer");
    let ctx = SessionContext::new();
    let b = RecordBatch::try_new(sch(), vec![
        Arc::new(Float64Array::from(vec![Some(1.0),Some(2.0),Some(3.0)])) as ArrayRef]).unwrap();
    ctx.register_table("rel", Arc::new(MemTable::try_new(sch(), vec![vec![b]])?))?;

    let base = ctx.sql("SELECT v FROM rel WHERE v > 1.0").await?.into_unoptimized_plan();
    let schema = base.schema().clone();
    let node = RuleNode { rule_id: "rule-42".into(), input: base, schema };
    let plan = LogicalPlan::Extension(Extension { node: Arc::new(node) });

    println!("   before optimize:\n{}", indent(&format!("{}", plan.display_indent())));

    let state = ctx.state();
    match state.optimize(&plan) {
        Ok(opt) => {
            let txt = format!("{}", opt.display_indent());
            println!("   after optimize:\n{}", indent(&txt));
            println!("   node survived        : {}", txt.contains("PseRule"));
            println!("   rule_id still visible: {}", txt.contains("rule-42"));
            // Does it reach a physical plan?
            match state.create_physical_plan(&opt).await {
                Ok(pp) => println!("   physical planning    : {}",
                    format!("{}", datafusion::physical_plan::displayable(pp.as_ref()).indent(false))
                        .lines().next().unwrap_or("").to_string()),
                Err(e) => { let s=format!("{e}");
                    println!("   physical planning    : FAILS without an ExtensionPlanner -> {}",
                        &s[..s.len().min(110)]); }
            }
        }
        Err(e) => println!("   optimize FAILED: {}", &format!("{e}")[..120]),
    }
    Ok(())
}
fn indent(s: &str) -> String { s.lines().map(|l| format!("     {l}")).collect::<Vec<_>>().join("\n") }
