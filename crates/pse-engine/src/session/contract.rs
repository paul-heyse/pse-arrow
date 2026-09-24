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
        execute_stream,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::{FutureExt, TryStreamExt};
use pse_schema::model::provider::OperationEffect;
use std::{collections::BTreeSet, sync::Arc};

/// Required native work, distinct from whether any output values are requested.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
struct Requirement {
    plan: LogicalPlan,
    check: bool,
}
/// Native input contract; requirements are visible children and fully exhausted.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct ExecutionContract {
    input: LogicalPlan,
    requirements: Vec<Requirement>,
    effects: BTreeSet<OperationEffect>,
    protected: bool,
}
impl ExecutionContract {
    pub(crate) fn same_binding(&self, other: &Self) -> bool {
        self.effects == other.effects
            && self.requirements.len() == other.requirements.len()
            && self
                .requirements
                .iter()
                .zip(&other.requirements)
                .all(|(a, b)| {
                    a.check == b.check && crate::operation::ports::same_input(&a.plan, &b.plan)
                })
            && crate::operation::ports::same_input(&self.input, &other.input)
    }
    /// Attach an optional zero-violation relation to a value input.
    pub fn plan(
        input: LogicalPlan,
        requirements: Option<LogicalPlan>,
        effects: BTreeSet<OperationEffect>,
    ) -> LogicalPlan {
        Self {
            input,
            requirements: requirements
                .filter(|plan| !matches!(plan, LogicalPlan::EmptyRelation(empty) if !empty.produce_one_row))
                .into_iter()
                .map(|plan| Requirement { plan, check: true })
                .collect(),
            effects,
            protected: false,
        }
        .into_plan()
    }
    fn into_plan(self) -> LogicalPlan {
        LogicalPlan::Extension(Extension {
            node: Arc::new(self),
        })
    }
    /// Effects belong to the actual contract, not its display label.
    pub fn effects(&self) -> &BTreeSet<OperationEffect> {
        &self.effects
    }
    pub(crate) fn operation(&self) -> &LogicalPlan {
        &self.input
    }
}
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
            .chain(self.requirements.iter().map(|r| &r.plan))
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
            "ExecutionContract: effects={:?}, requirements={}, protected={}",
            self.effects,
            self.requirements.len(),
            self.protected
        )
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if !expressions.is_empty() || inputs.len() != self.requirements.len() + 1 {
            return Err(invalid("contract child arity changed"));
        }
        let mut inputs = inputs.into_iter();
        let input = inputs
            .next()
            .ok_or_else(|| invalid("contract value absent"))?;
        Ok(Self {
            input,
            requirements: self
                .requirements
                .iter()
                .zip(inputs)
                .map(|(r, plan)| Requirement {
                    plan,
                    check: r.check,
                })
                .collect(),
            ..self.clone()
        })
    }
    fn necessary_children_exprs(&self, output_columns: &[usize]) -> Option<Vec<Vec<usize>>> {
        Some(
            std::iter::once(output_columns.to_vec())
                .chain(
                    self.requirements
                        .iter()
                        .map(|requirement| (0..requirement.plan.schema().fields().len()).collect()),
                )
                .collect(),
        )
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        if self.requirements.is_empty()
            && self
                .effects
                .iter()
                .all(|effect| *effect == OperationEffect::Read)
        {
            return std::collections::HashSet::new();
        }
        self.input
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect()
    }
}

mod protection;
pub(super) use protection::protect;
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
        super::execution::NativeExecutionContext::from_session(session)?
            .admit_effects(&node.effects)
            .map_err(pse_columnar::external)?;
        if inputs.len() != node.requirements.len() + 1 {
            return Err(invalid("contract physical child arity changed"));
        }
        if node.requirements.is_empty() {
            return Ok(Some(inputs[0].clone()));
        }
        Ok(Some(ContractExec::from_requirements(
            inputs[0].clone(),
            inputs[1..]
                .iter()
                .cloned()
                .zip(node.requirements.iter().map(|r| r.check))
                .collect(),
        )))
    }
}
#[derive(Debug)]
pub(crate) struct ContractExec {
    input: Arc<dyn ExecutionPlan>,
    requirements: Vec<(Arc<dyn ExecutionPlan>, bool)>,
    checked: Arc<crate::operation::completion::Completion<()>>,
    metrics: datafusion::physical_plan::metrics::ExecutionPlanMetricsSet,
}
impl ContractExec {
    pub(crate) fn checked_output(
        &self,
        context: Arc<TaskContext>,
    ) -> Option<
        futures_util::future::BoxFuture<
            'static,
            Result<Vec<pse_relations::columnar::FieldCheckedBatch>>,
        >,
    > {
        let output = crate::operation::ports::capture(&self.input, context.clone())?;
        let check = self.check(context);
        Some(
            async move {
                check.await?;
                output.await
            }
            .boxed(),
        )
    }
    fn check(
        &self,
        context: Arc<TaskContext>,
    ) -> futures_util::future::BoxFuture<'static, Result<()>> {
        let requirements = self.requirements.clone();
        let checked = self.checked.clone();
        let requests = datafusion::physical_plan::metrics::MetricBuilder::new(&self.metrics)
            .counter("requirement_requests", 0);
        let executions = datafusion::physical_plan::metrics::MetricBuilder::new(&self.metrics)
            .counter("requirement_executions", 0);
        async move {
            requests.add(1);
            checked
                .get(|| {
                    async move {
                        executions.add(1);
                        let mut violated = false;
                        for (requirement, check) in requirements {
                            let mut stream = execute_stream(requirement, context.clone())?;
                            while let Some(batch) = stream.try_next().await? {
                                violated |= check && batch.num_rows() != 0;
                            }
                        }
                        if violated {
                            return Err(crate::operation::FailureKind::Invalid.error(
                                DataFusionError::Execution(
                                    "scoped invariant requirements produced violations".into(),
                                ),
                            ));
                        }
                        Ok(())
                    }
                    .boxed()
                })
                .await?;
            Ok(())
        }
        .boxed()
    }
    #[cfg(test)]
    pub(crate) fn new(
        input: Arc<dyn ExecutionPlan>,
        requirement: Option<Arc<dyn ExecutionPlan>>,
    ) -> Arc<Self> {
        Self::from_requirements(
            input,
            requirement.into_iter().map(|plan| (plan, true)).collect(),
        )
    }
    fn from_requirements(
        input: Arc<dyn ExecutionPlan>,
        requirements: Vec<(Arc<dyn ExecutionPlan>, bool)>,
    ) -> Arc<Self> {
        Arc::new(Self {
            input,
            requirements,
            checked: Arc::default(),
            metrics: datafusion::physical_plan::metrics::ExecutionPlanMetricsSet::new(),
        })
    }
}
impl DisplayAs for ContractExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExecutionContractExec: requirements={}",
            self.requirements.len()
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
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        std::iter::once(&self.input)
            .chain(self.requirements.iter().map(|(plan, _)| plan))
            .collect()
    }
    fn maintains_input_order(&self) -> Vec<bool> {
        std::iter::once(true)
            .chain(self.requirements.iter().map(|_| false))
            .collect()
    }
    fn benefits_from_input_partitioning(&self) -> Vec<bool> {
        vec![false; self.children().len()]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.replace_children(
            children,
            datafusion::physical_plan::execution_plan::ReplaceChildrenOptions::new(
                datafusion::physical_plan::execution_plan::ChildrenPropertiesMode::Recompute,
            ),
        )
    }
    fn replace_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
        _: datafusion::physical_plan::execution_plan::ReplaceChildrenOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != self.requirements.len() + 1 {
            return Err(invalid("contract physical child arity changed"));
        }
        if self
            .children()
            .iter()
            .zip(&children)
            .all(|(a, b)| Arc::ptr_eq(a, b))
        {
            return Ok(self);
        }
        Ok(Self::from_requirements(
            children[0].clone(),
            children[1..]
                .iter()
                .cloned()
                .zip(self.requirements.iter().map(|(_, check)| *check))
                .collect(),
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
    fn child_stats_requests(
        &self,
        partition: Option<usize>,
    ) -> Vec<datafusion::physical_plan::statistics::ChildStats> {
        use datafusion::physical_plan::statistics::ChildStats;
        if self.requirements.is_empty() {
            vec![ChildStats::At(partition)]
        } else {
            vec![ChildStats::Skip; self.children().len()]
        }
    }
    fn statistics_from_inputs(
        &self,
        inputs: &[Arc<datafusion::common::Statistics>],
        _: &datafusion::physical_plan::StatisticsArgs,
    ) -> Result<Arc<datafusion::common::Statistics>> {
        if self.requirements.is_empty() {
            inputs
                .first()
                .cloned()
                .ok_or_else(|| invalid("contract child statistics absent"))
        } else {
            // Exact counts could let native statistics substitution erase required work.
            Ok(Arc::new(datafusion::common::Statistics::new_unknown(
                &self.schema(),
            )))
        }
    }
    fn metrics(&self) -> Option<datafusion::physical_plan::metrics::MetricsSet> {
        Some(self.metrics.clone_inner())
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let input = self.input.clone();
        let check = self.check(context.clone());
        let stream = futures_util::stream::once(async move {
            check.await?;
            input.execute(partition, context)
        })
        .try_flatten();
        Ok(crate::operation::observe(
            self.schema(),
            Box::pin(stream),
            &self.metrics,
            partition,
        ))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
#[cfg(test)]
mod tests;
