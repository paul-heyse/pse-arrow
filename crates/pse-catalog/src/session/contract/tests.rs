// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated physical barrier tests; no catalog, storage or compiler workflow.
use super::*;
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    datasource::memory::MemorySourceConfig,
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct Counted {
    input: Arc<dyn ExecutionPlan>,
    calls: Arc<AtomicUsize>,
}
impl DisplayAs for Counted {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Counted")
    }
}
impl ExecutionPlan for Counted {
    fn name(&self) -> &'static str {
        "Counted"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        self.input.properties()
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.input]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(Self {
            input: Arc::clone(&children[0]),
            calls: Arc::clone(&self.calls),
        }))
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.input.execute(partition, context)
    }
}
fn counted(rows: &[i64], partitions: usize) -> (Arc<dyn ExecutionPlan>, Arc<AtomicUsize>) {
    let schema = Arc::new(Schema::new(vec![Field::new("key", DataType::Int64, false)]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(Int64Array::from(rows.to_vec()))],
    )
    .unwrap();
    let input =
        MemorySourceConfig::try_new_exec(&vec![vec![batch]; partitions], schema, None).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    (
        Arc::new(Counted {
            input,
            calls: Arc::clone(&calls),
        }),
        calls,
    )
}
#[tokio::test]
async fn violations_prevent_all_value_execution_and_are_shared() {
    let (input, values) = counted(&[7], 2);
    let (requirement, checks) = counted(&[99], 1);
    let contract = ContractExec::new(input, Some(requirement));
    let context = Arc::new(TaskContext::default());
    let mut first = contract.execute(0, Arc::clone(&context)).unwrap();
    let mut second = contract.execute(1, context).unwrap();
    assert_eq!(
        checks.load(Ordering::SeqCst),
        0,
        "construction must be lazy"
    );
    let (left, right) = tokio::join!(first.try_next(), second.try_next());
    assert!(left.is_err());
    assert!(right.is_err());
    assert_eq!(checks.load(Ordering::SeqCst), 1);
    assert_eq!(values.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn empty_requirements_unlock_streaming_partitions_once() {
    let (input, values) = counted(&[7, 8], 2);
    let (requirement, checks) = counted(&[], 1);
    let contract = ContractExec::new(input, Some(requirement));
    assert_eq!(contract.children().len(), 2);
    let context = Arc::new(TaskContext::default());
    let mut first = contract.execute(0, Arc::clone(&context)).unwrap();
    let mut second = contract.execute(1, context).unwrap();
    let (left, right) = tokio::join!(first.try_next(), second.try_next());
    assert_eq!(left.unwrap().unwrap().num_rows(), 2);
    assert_eq!(right.unwrap().unwrap().num_rows(), 2);
    assert_eq!(checks.load(Ordering::SeqCst), 1);
    assert_eq!(values.load(Ordering::SeqCst), 2);
}
