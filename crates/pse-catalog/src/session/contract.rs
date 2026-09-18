// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native execution contracts: requirements are real children, never eager callbacks.

use datafusion::{
    catalog::Session,
    common::{DFSchemaRef, DataFusionError, Result, tree_node::TreeNodeRecursion},
    execution::TaskContext,
    logical_expr::{
        Expr, Extension, LogicalPlan, UserDefinedLogicalNode, UserDefinedLogicalNodeCore,
        physical_planning_context::PhysicalPlanningContext,
    },
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, PlanProperties, SendableRecordBatchStream,
        execute_stream, stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::TryStreamExt;
use pse_schema::model::provider::OperationEffect;
use std::{collections::BTreeSet, sync::Arc};

/// Declared effects and a native zero-violation requirement plan enclosing a real
/// operation. Its value input cannot execute until the complete requirement input
/// has finished successfully. This contract is valid at any depth in a native plan.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct ExecutionContract {
    input: LogicalPlan,
    requirements: Option<LogicalPlan>,
    effects: BTreeSet<OperationEffect>,
}
impl ExecutionContract {
    /// Attach actual effect semantics and optionally a native violation relation.
    /// Native planning is side-effect free; validation starts when execution polls.
    pub fn plan(
        input: LogicalPlan,
        requirements: Option<LogicalPlan>,
        effects: BTreeSet<OperationEffect>,
    ) -> LogicalPlan {
        LogicalPlan::Extension(Extension {
            node: Arc::new(Self {
                input,
                requirements,
                effects,
            }),
        })
    }
    /// The effect declaration belongs to this actual node, not to its display name.
    pub fn effects(&self) -> &BTreeSet<OperationEffect> {
        &self.effects
    }
    pub(crate) fn operation(&self) -> &LogicalPlan {
        &self.input
    }
}
// Native plan renderers visit children separately. Debug describes this
// node without recursively duplicating complete subgraphs in JSON.
impl std::fmt::Debug for ExecutionContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
    }
}
impl UserDefinedLogicalNodeCore for ExecutionContract {
    fn name(&self) -> &'static str {
        "ExecutionContract"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        std::iter::once(&self.input)
            .chain(self.requirements.iter())
            .collect()
    }
    fn schema(&self) -> &DFSchemaRef {
        self.input.schema()
    }
    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExecutionContract: effects={:?}, requirements={}",
            self.effects,
            self.requirements.is_some()
        )
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if !expressions.is_empty() || inputs.len() != 1 + usize::from(self.requirements.is_some()) {
            return Err(invalid(
                "execution contract children differ from its declaration",
            ));
        }
        let mut inputs = inputs.into_iter();
        let input = inputs
            .next()
            .ok_or_else(|| invalid("execution contract input absent"))?;
        Ok(Self {
            input,
            requirements: inputs.next(),
            effects: self.effects.clone(),
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.input
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct ContractPlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for ContractPlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<ExecutionContract>() else {
            return Ok(None);
        };
        let services = super::execution::NativeExecutionContext::from_session(session)?;
        services
            .admit_effects(&node.effects)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        if inputs.len() != 1 + usize::from(node.requirements.is_some()) {
            return Err(invalid(
                "execution contract requires its declared physical children",
            ));
        }
        Ok(Some(ContractExec::new(
            Arc::clone(&inputs[0]),
            inputs.get(1).cloned(),
        )))
    }
}

/// A streaming barrier sharing one requirement result across every output partition.
/// Validation is deferred until first poll; no result batches are accumulated.
#[derive(Debug)]
pub(crate) struct ContractExec {
    input: Arc<dyn ExecutionPlan>,
    requirements: Option<Arc<dyn ExecutionPlan>>,
    checked: Arc<tokio::sync::OnceCell<std::result::Result<(), Arc<DataFusionError>>>>,
}
impl ContractExec {
    pub(crate) fn new(
        input: Arc<dyn ExecutionPlan>,
        requirements: Option<Arc<dyn ExecutionPlan>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            input,
            requirements,
            checked: Arc::new(tokio::sync::OnceCell::new()),
        })
    }
}
impl DisplayAs for ContractExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExecutionContractExec: requirements={}",
            self.requirements.is_some()
        )
    }
}
impl ExecutionPlan for ContractExec {
    fn name(&self) -> &'static str {
        "ExecutionContractExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        self.input.properties()
    }
    fn maintains_input_order(&self) -> Vec<bool> {
        // The requirement is a barrier; the value stream is passed through unchanged.
        std::iter::once(true)
            .chain(self.requirements.iter().map(|_| false))
            .collect()
    }
    fn benefits_from_input_partitioning(&self) -> Vec<bool> {
        vec![false; self.children().len()]
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        std::iter::once(&self.input)
            .chain(self.requirements.iter())
            .collect()
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != 1 + usize::from(self.requirements.is_some()) {
            return Err(invalid("execution contract physical children differ"));
        }
        Ok(Self::new(
            Arc::clone(&children[0]),
            children.get(1).cloned(),
        ))
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
        let input = Arc::clone(&self.input);
        let requirements = self.requirements.clone();
        let checked = Arc::clone(&self.checked);
        let stream = futures_util::stream::once(async move {
            let result = checked
                .get_or_init(|| async {
                    let result: Result<()> = async {
                        if let Some(requirements) = requirements {
                            let mut stream = execute_stream(requirements, Arc::clone(&context))?;
                            while let Some(batch) = stream.try_next().await? {
                                if batch.num_rows() != 0 {
                                    return Err(DataFusionError::Execution(
                                        "scoped invariant requirements produced violations".into(),
                                    ));
                                }
                            }
                        }
                        Ok(())
                    }
                    .await;
                    result.map_err(Arc::new)
                })
                .await;
            result.clone().map_err(DataFusionError::Shared)?;
            input.execute(partition, context)
        })
        .try_flatten();
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}

#[cfg(test)]
mod tests;
