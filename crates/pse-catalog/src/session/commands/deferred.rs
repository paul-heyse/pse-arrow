// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! DataFusion's eager namespace handlers enter through a deferred native operator.

use datafusion::{
    catalog::Session,
    common::{
        DFSchemaRef, DataFusionError, Result,
        tree_node::{Transformed, TreeNodeRecursion},
    },
    execution::{TaskContext, context::SessionContext, session_state::SessionState},
    logical_expr::{
        Expr, Extension, LogicalPlan, LogicalPlanBuilder, UserDefinedLogicalNode,
        UserDefinedLogicalNodeCore, physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::TryStreamExt;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, PartialOrd, Hash)]
struct Command {
    plan: LogicalPlan,
}
// Native plan renderers visit children separately. Debug describes this
// node without recursively duplicating complete subgraphs in JSON.
impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
    }
}
impl UserDefinedLogicalNodeCore for Command {
    fn name(&self) -> &'static str {
        "NativeCommand"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        self.plan.inputs()
    }
    fn schema(&self) -> &DFSchemaRef {
        self.plan.schema()
    }
    fn expressions(&self) -> Vec<Expr> {
        self.plan.expressions()
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeCommand: {}", self.plan.display())
    }
    fn with_exprs_and_inputs(&self, exprs: Vec<Expr>, inputs: Vec<LogicalPlan>) -> Result<Self> {
        let mut plan = self.plan.with_new_exprs(exprs, inputs)?;
        if let (
            LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(before)),
            LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(after)),
        ) = (&self.plan, &mut plan)
        {
            // DataFusion 55's generic reconstruction clears CREATE constraints.
            // This deferred command retains its declared output positions; never
            // transfer index-based keys to a changed column layout.
            if before.input.schema().fields() != after.input.schema().fields() {
                return Err(DataFusionError::Plan(
                    "deferred CREATE input changed its declared field layout".into(),
                ));
            }
            after.constraints = before.constraints.clone();
        }
        Ok(Self { plan })
    }
}
pub(in crate::session) fn defer_node(plan: LogicalPlan) -> Transformed<LogicalPlan> {
    if matches!(plan, LogicalPlan::Ddl(_) | LogicalPlan::Statement(_)) {
        Transformed::yes(LogicalPlan::Extension(Extension {
            node: Arc::new(Command { plan }),
        }))
    } else {
        Transformed::no(plan)
    }
}

#[derive(Debug)]
pub(crate) struct CommandPlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for CommandPlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<Command>() else {
            return Ok(None);
        };
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("native command requires caller SessionState".into())
            })?;
        Ok(Some(Arc::new(CommandExec::new(
            node.plan.clone(),
            inputs.to_vec(),
            state.clone(),
        ))))
    }
}
#[derive(Debug)]
struct CommandExec {
    plan: LogicalPlan,
    inputs: Vec<Arc<dyn ExecutionPlan>>,
    state: SessionState,
    properties: Arc<PlanProperties>,
    completed: Arc<std::sync::Mutex<Option<Arc<SessionState>>>>,
}
impl CommandExec {
    fn new(plan: LogicalPlan, inputs: Vec<Arc<dyn ExecutionPlan>>, state: SessionState) -> Self {
        let properties = Arc::new(PlanProperties::new(
            EquivalenceProperties::new(Arc::new(plan.schema().as_arrow().clone())),
            Partitioning::UnknownPartitioning(1),
            EmissionType::Final,
            Boundedness::Bounded,
        ));
        Self {
            plan,
            inputs,
            state,
            properties,
            completed: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}
impl DisplayAs for CommandExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeCommandExec: {}", self.plan.display())
    }
}
impl ExecutionPlan for CommandExec {
    fn name(&self) -> &'static str {
        "NativeCommandExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        self.inputs.iter().collect()
    }
    fn with_new_children(
        self: Arc<Self>,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if inputs.len() != self.inputs.len() {
            return Err(DataFusionError::Plan(
                "native command input count changed".into(),
            ));
        }
        Ok(Arc::new(Self::new(
            self.plan.clone(),
            inputs,
            self.state.clone(),
        )))
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Execution(
                "native command has one partition".into(),
            ));
        }
        let services = super::super::execution::NativeExecutionContext::from_session(&self.state)?;
        let context = SessionContext::new_with_state(services.command_state(&self.state)?);
        let completed = Arc::clone(&self.completed);
        let mut plan = self.plan.clone();
        // CREATE VIEW retains its native logical definition. CTAS consumes the
        // already planned child; it never re-plans or executes a hidden source.
        if let LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(
            command,
        )) = &mut plan
        {
            let [input] = self.inputs.as_slice() else {
                return Err(DataFusionError::Plan("CTAS requires one child".into()));
            };
            command.input = Arc::new(
                LogicalPlanBuilder::scan(
                    "native_command_input",
                    datafusion::datasource::provider_as_source(Arc::new(
                        crate::session::physical_input::PhysicalInput::native(Arc::clone(input)),
                    )),
                    None,
                )?
                .build()?,
            );
        }
        let stream = futures_util::stream::once(async move {
            use pse_schema::model::provider::OperationEffect;
            let effect = OperationEffect::Namespace;
            services
                .admit_effects(&[effect].into_iter().collect())
                .map_err(|error| DataFusionError::External(Box::new(error)))?;
            let frame = if let LogicalPlan::Ddl(command) = &plan {
                super::create_namespace(&context, command)?
            } else {
                None
            };
            let frame = match frame {
                Some(frame) => frame,
                None => context.execute_logical_plan(plan).await?,
            };
            let stream = frame.execute_stream().await?;
            let state = Arc::new(context.state());
            services.record_command_state(&state)?;
            *completed.lock().map_err(|_| {
                DataFusionError::Internal("native command result lock poisoned".into())
            })? = Some(state);
            Ok::<_, DataFusionError>(stream)
        })
        .try_flatten();
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionFactory, execution::NativeExecutionContext};
    use datafusion::execution::{
        config::SessionConfig, runtime_env::RuntimeEnv, session_state::SessionStateBuilder,
    };
    use pse_ids::{CancellationToken, FixedBudget};

    #[derive(Debug)]
    struct CallerOption(u64);

    #[tokio::test]
    async fn native_set_retains_actual_caller_state_without_a_state_ownership_cycle() {
        let cancel = CancellationToken::new();
        let factory = SessionFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            FixedBudget::new(8 << 20),
            "command-unit",
            SessionStateBuilder::new_with_default_features()
                .with_config(SessionConfig::new().with_extension(Arc::new(CallerOption(219)))),
        );
        let session = factory
            .candidate_checked(
                std::collections::BTreeMap::new(),
                Arc::new(pse_schema::builder::RegistryBuilder::new().build().unwrap()),
                &cancel,
            )
            .unwrap()
            .with_purpose(pse_schema::model::provider::OperationPurpose::Mutate);
        let original =
            NativeExecutionContext::bind(&session, session.bound_state().unwrap(), &cancel)
                .unwrap();
        let state = NativeExecutionContext::execution_state(&original, &cancel);
        let services = NativeExecutionContext::from_session(&state).unwrap();
        let plan = state
            .create_logical_plan("SET datafusion.execution.batch_size = 7")
            .await
            .unwrap();
        let physical: Arc<dyn ExecutionPlan> =
            Arc::new(CommandExec::new(plan, vec![], state.clone()));
        let before = state.config_options().execution.batch_size.get();
        assert_ne!(before, 7);
        datafusion::physical_plan::execute_stream(Arc::clone(&physical), state.task_ctx())
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        let completed = services.command_state(&state).unwrap();
        assert_eq!(completed.config_options().execution.batch_size.get(), 7);
        assert_eq!(
            completed
                .config()
                .get_extension::<CallerOption>()
                .unwrap()
                .0,
            219
        );
        assert_eq!(state.config_options().execution.batch_size.get(), before);
        drop(completed);
        drop(physical);
        assert_eq!(
            services
                .command_state(&state)
                .unwrap()
                .config_options()
                .execution
                .batch_size
                .get(),
            before
        );
        let weak = Arc::downgrade(&services);
        drop(state);
        drop(services);
        assert!(weak.upgrade().is_none());
    }
}
