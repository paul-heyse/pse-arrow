// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deferred native namespace commands; common operation shells own execution.
use crate::operation::{Body, Definition, Family, Operation};
use datafusion::{
    common::{DataFusionError, Result, tree_node::Transformed},
    execution::{TaskContext, context::SessionContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, stream::RecordBatchStreamAdapter},
};
use futures_util::TryStreamExt;
use std::sync::Arc;

#[derive(Debug)]
struct Command {
    plan: LogicalPlan,
}

pub(in crate::session) fn created_memory_table(
    plan: &LogicalPlan,
) -> Option<&datafusion::common::TableReference> {
    match plan {
        LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(command)) => {
            Some(&command.name)
        }
        LogicalPlan::Extension(extension) => {
            if let Some(contract) = extension
                .node
                .as_any()
                .downcast_ref::<super::super::contract::ExecutionContract>()
            {
                return created_memory_table(contract.operation());
            }
            extension
                .node
                .as_any()
                .downcast_ref::<Operation>()
                .and_then(Operation::definition::<Command>)
                .and_then(|command| created_memory_table(&command.plan))
        }
        _ => None,
    }
}
pub(in crate::session) fn defer_node(plan: LogicalPlan) -> Result<Transformed<LogicalPlan>> {
    if matches!(plan, LogicalPlan::Ddl(_) | LogicalPlan::Statement(_)) {
        let inputs = plan.inputs().into_iter().cloned().collect();
        Ok(Transformed::yes(Operation::plan(
            Arc::new(Command { plan }),
            inputs,
        )?))
    } else {
        Ok(Transformed::no(plan))
    }
}
fn definition_only(plan: &LogicalPlan) -> bool {
    matches!(
        plan,
        LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateView(_))
    )
}
#[async_trait::async_trait]
impl Definition for Command {
    fn definition_only(&self, _: usize) -> bool {
        definition_only(&self.plan)
    }
    fn name(&self) -> &'static str {
        "NativeCommand"
    }
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        Arc::new(self.plan.schema().as_arrow().clone())
    }
    fn family(&self) -> Family {
        Family::Command
    }
    fn effects(&self) -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
        std::collections::BTreeSet::from([pse_schema::model::provider::OperationEffect::Namespace])
    }
    fn expressions(&self) -> Vec<Expr> {
        self.plan.expressions()
    }
    async fn prepare(
        self: Arc<Self>,
        expressions: &[Expr],
        inputs: &[LogicalPlan],
        _: &[Arc<dyn ExecutionPlan>],
        state: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        let mut plan = self
            .plan
            .with_new_exprs(expressions.to_vec(), inputs.to_vec())?;
        if let (
            LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(before)),
            LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(after)),
        ) = (&self.plan, &mut plan)
        {
            if before.input.schema().fields() != after.input.schema().fields() {
                return Err(DataFusionError::Plan("CREATE input fields changed".into()));
            }
            after.constraints = before.constraints.clone();
        }
        Ok(Arc::new(CommandBody::new(plan, state.clone())))
    }
}
#[derive(Debug)]
struct CommandBody {
    plan: LogicalPlan,
    state: SessionState,
    completed: Arc<std::sync::Mutex<Option<Arc<SessionState>>>>,
}
impl CommandBody {
    fn new(plan: LogicalPlan, state: SessionState) -> Self {
        Self {
            plan,
            state,
            completed: Arc::default(),
        }
    }
}
impl Body for CommandBody {
    fn definition_only(&self, _: usize) -> bool {
        definition_only(&self.plan)
    }
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let services = super::super::execution::NativeExecutionContext::from_session(&self.state)?;
        let initial = if matches!(
            &self.plan,
            LogicalPlan::Statement(
                datafusion::logical_expr::Statement::Prepare(_)
                    | datafusion::logical_expr::Statement::Deallocate(_)
            )
        ) {
            services.definition_state(&self.state)
        } else {
            self.state.clone()
        };
        let context = SessionContext::new_with_state(services.command_state(&initial)?);
        let completed = Arc::clone(&self.completed);
        let mut plan = self.plan.clone();
        // CREATE VIEW retains its native logical definition. CTAS consumes the
        // already planned child; it never re-plans or executes a hidden source.
        if let LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(
            command,
        )) = &mut plan
        {
            let [input] = inputs.as_slice() else {
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
                .map_err(pse_columnar::external)?;
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
            Arc::new(self.plan.schema().as_arrow().clone()),
            stream,
        )))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{EngineFactory, execution::NativeExecutionContext};
    use datafusion::execution::{
        config::SessionConfig, runtime_env::RuntimeEnv, session_state::SessionStateBuilder,
    };
    use pse_columnar::CancellationToken;

    #[derive(Debug)]
    struct CallerOption(u64);

    #[tokio::test]
    async fn native_set_retains_actual_caller_state_without_a_state_ownership_cycle() {
        let cancel = CancellationToken::new();
        let factory = EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20)),
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
        let state = NativeExecutionContext::execution_state(&original, &cancel).unwrap();
        let services = NativeExecutionContext::from_session(&state).unwrap();
        let plan = state
            .create_logical_plan("SET datafusion.execution.batch_size = 7")
            .await
            .unwrap();
        let physical = crate::operation::Execution::plan(
            "NativeCommand",
            Arc::new(plan.schema().as_arrow().clone()),
            vec![],
            Arc::new(CommandBody::new(plan, state.clone())),
            Family::Command,
            Arc::default(),
            services.clone(),
        )
        .unwrap();
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
