// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private replacement is a native TableProvider write contract at any plan depth.
use super::super::{SnapshotSession, physical_input::PhysicalInput};
use crate::provider::{binding::TableBinding, schema::SnapshotSchema};
use datafusion::{
    arrow::{
        array::{Array, RecordBatch, UInt64Array},
        datatypes::SchemaRef,
    },
    catalog::{Session, TableProvider},
    common::{Constraints, DFSchemaRef, DataFusionError, Result, tree_node::TreeNodeRecursion},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, MergeIntoClause, TableType},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
};
use datafusion_expr::dml::InsertOp;
use futures_util::TryStreamExt;
use std::sync::Arc;

mod isolation;
pub(crate) use isolation::isolate;

#[derive(Debug)]
struct IsolatedTarget {
    binding: Arc<TableBinding>,
    session: SnapshotSession,
}
impl IsolatedTarget {
    async fn write(
        &self,
        action: NativeWrite,
        input: Option<Arc<dyn ExecutionPlan>>,
        state: &dyn Session,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let actual = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| invalid("private DML requires actual caller state"))?;
        let source = if self.binding.mutation.is_some() {
            Some(self.binding.provider.scan(state, None, &[], None).await?)
        } else {
            None
        };
        if self.binding.mutation.is_some()
            && (source.as_ref().is_some_and(|source| {
                !matches!(source.properties().boundedness, Boundedness::Bounded)
            }) || input.as_ref().is_some_and(|input| {
                !matches!(input.properties().boundedness, Boundedness::Bounded)
            }))
        {
            return Err(invalid("private table replacement requires bounded inputs"));
        }
        let properties = Arc::new(PlanProperties::new(
            EquivalenceProperties::new(Arc::new(datafusion::arrow::datatypes::Schema::new(vec![
                datafusion::arrow::datatypes::Field::new(
                    "count",
                    datafusion::arrow::datatypes::DataType::UInt64,
                    false,
                ),
            ]))),
            Partitioning::UnknownPartitioning(1),
            EmissionType::Final,
            Boundedness::Bounded,
        ));
        Ok(Arc::new(IsolatedWriteExec {
            action,
            input,
            source,
            binding: Arc::clone(&self.binding),
            session: self.session.clone(),
            state: actual.clone(),
            properties,
            started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }))
    }
}
#[async_trait::async_trait]
impl TableProvider for IsolatedTarget {
    fn schema(&self) -> SchemaRef {
        self.binding.provider.schema()
    }
    fn table_type(&self) -> TableType {
        self.binding.provider.table_type()
    }
    fn constraints(&self) -> Option<&Constraints> {
        self.binding.provider.constraints()
    }
    fn get_column_default(&self, name: &str) -> Option<&Expr> {
        self.binding.provider.get_column_default(name)
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.binding
            .provider
            .scan(state, projection, filters, limit)
            .await
    }
    async fn insert_into(
        &self,
        state: &dyn Session,
        input: Arc<dyn ExecutionPlan>,
        op: InsertOp,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.write(NativeWrite::Insert(op), Some(input), state)
            .await
    }
    async fn delete_from(
        &self,
        state: &dyn Session,
        filters: Vec<Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.write(NativeWrite::Delete(filters), None, state).await
    }
    async fn update(
        &self,
        state: &dyn Session,
        assignments: Vec<(String, Expr)>,
        filters: Vec<Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.write(
            NativeWrite::Update {
                assignments,
                filters,
            },
            None,
            state,
        )
        .await
    }
    async fn truncate(&self, state: &dyn Session) -> Result<Arc<dyn ExecutionPlan>> {
        self.write(NativeWrite::Truncate, None, state).await
    }
    async fn merge_into(
        &self,
        state: &dyn Session,
        source: Arc<dyn ExecutionPlan>,
        schema: DFSchemaRef,
        on: Expr,
        clauses: Vec<MergeIntoClause>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.write(
            NativeWrite::Merge {
                schema,
                on,
                clauses,
            },
            Some(source),
            state,
        )
        .await
    }
}

/// Native hook arguments, retained unchanged until the private table exists.
/// DataFusion and the actual target implement every expression and write operation.
#[derive(Clone, Debug)]
enum NativeWrite {
    Insert(InsertOp),
    Delete(Vec<Expr>),
    Update {
        assignments: Vec<(String, Expr)>,
        filters: Vec<Expr>,
    },
    Truncate,
    Merge {
        schema: DFSchemaRef,
        on: Expr,
        clauses: Vec<MergeIntoClause>,
    },
}
impl NativeWrite {
    async fn plan(
        &self,
        target: &dyn TableProvider,
        input: Option<Arc<dyn ExecutionPlan>>,
        state: &SessionState,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        match self {
            Self::Insert(op) => {
                target
                    .insert_into(
                        state,
                        input.ok_or_else(|| invalid("native INSERT input absent"))?,
                        *op,
                    )
                    .await
            }
            Self::Delete(filters) => target.delete_from(state, filters.clone()).await,
            Self::Update {
                assignments,
                filters,
            } => {
                target
                    .update(state, assignments.clone(), filters.clone())
                    .await
            }
            Self::Truncate => target.truncate(state).await,
            Self::Merge {
                schema,
                on,
                clauses,
            } => {
                target
                    .merge_into(
                        state,
                        input.ok_or_else(|| invalid("native MERGE input absent"))?,
                        Arc::clone(schema),
                        on.clone(),
                        clauses.clone(),
                    )
                    .await
            }
        }
    }
}
#[derive(Debug)]
struct IsolatedWriteExec {
    action: NativeWrite,
    input: Option<Arc<dyn ExecutionPlan>>,
    source: Option<Arc<dyn ExecutionPlan>>,
    binding: Arc<TableBinding>,
    session: SnapshotSession,
    state: SessionState,
    properties: Arc<PlanProperties>,
    started: Arc<std::sync::atomic::AtomicBool>,
}
impl DisplayAs for IsolatedWriteExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IsolatedWriteExec: {} {:?}",
            self.binding.reference.to_quoted_string(),
            self.action
        )
    }
}
impl ExecutionPlan for IsolatedWriteExec {
    fn name(&self) -> &'static str {
        "IsolatedWriteExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        self.source.iter().chain(self.input.iter()).collect()
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != self.children().len() {
            return Err(invalid("native DML child count changed"));
        }
        Ok(Arc::new(Self {
            action: self.action.clone(),
            source: self.source.as_ref().map(|_| Arc::clone(&children[0])),
            input: self
                .input
                .as_ref()
                .map(|_| Arc::clone(&children[usize::from(self.source.is_some())])),
            binding: Arc::clone(&self.binding),
            session: self.session.clone(),
            state: self.state.clone(),
            properties: Arc::clone(&self.properties),
            started: Arc::clone(&self.started),
        }))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one effect barrier, native hook and optional private validation/commit sequence"
    )]
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.started.swap(true, std::sync::atomic::Ordering::AcqRel) {
            return Err(invalid("private DML executes once in partition zero"));
        }
        let source = self.source.clone();
        let input = self.input.clone();
        let binding = Arc::clone(&self.binding);
        let session = self.session.clone();
        let state = self.state.clone();
        let action = self.action.clone();
        let schema = self.schema();
        let output_schema = Arc::clone(&schema);
        let stream = futures_util::stream::once(async move {
            let services = super::super::execution::NativeExecutionContext::from_session(&state)?;
            services
                .admit_effects(
                    &[
                        pse_schema::model::provider::OperationEffect::Read,
                        pse_schema::model::provider::OperationEffect::Write,
                    ]
                    .into_iter()
                    .collect(),
                )
                .map_err(external)?;

            let cancel = services.cancellation();
            cancel.checkpoint().map_err(|e| external(e.into()))?;
            let Some(factory) = binding.mutation.as_ref() else {
                let physical = action
                    .plan(binding.provider.as_ref(), input, &state)
                    .await?;
                let batches =
                    datafusion::physical_plan::execute_stream(physical, state.task_ctx())?
                        .try_collect::<Vec<_>>()
                        .await?;
                return count_result(&batches, schema);
            };
            let source: Arc<dyn TableProvider> = Arc::new(
                PhysicalInput::native(source.ok_or_else(|| invalid("private source absent"))?)
                    .with_defaults(binding.provider.as_ref()),
            );
            let target = factory.begin(source, &state).await?;
            if Arc::ptr_eq(&target, &binding.provider)
                || target.schema() != binding.provider.schema()
            {
                return Err(invalid(
                    "private factory must isolate backing and retain the exact source schema",
                ));
            }
            let physical = action.plan(target.as_ref(), input, &state).await?;
            let batches = datafusion::physical_plan::execute_stream(physical, state.task_ctx())?
                .try_collect::<Vec<_>>()
                .await?;
            let count = count_result(&batches, schema)?;
            cancel.checkpoint().map_err(|e| external(e.into()))?;
            let mut validation = session.clone();
            let reference = binding.reference.clone();
            validation.bindings.retain(|key| {
                session
                    .bindings
                    .get(key)
                    .is_none_or(|entry| entry.reference != reference)
            });
            let captured = validation
                .prepare_capture_constraints(
                    reference.clone(),
                    target,
                    binding.provider.constraints().cloned().unwrap_or_default(),
                    false,
                    cancel,
                )
                .map_err(external)?
                .execute(cancel)
                .await
                .map_err(external)?;
            let mut replacement = binding.as_ref().clone();
            replacement.provider = captured.provider();
            replacement.checked = None;
            replacement.selection = None;
            replacement.dependencies.clear();
            let mut admitted = session;
            admitted.bindings.replace_table(&reference, &replacement)?;
            admitted
                .check_requirements(cancel)
                .await
                .map_err(external)?;
            let catalog = state
                .catalog_list()
                .catalog(
                    reference
                        .catalog()
                        .ok_or_else(|| invalid("DML catalog absent"))?,
                )
                .ok_or_else(|| invalid("DML catalog disappeared"))?;
            let schema = catalog
                .schema(
                    reference
                        .schema()
                        .ok_or_else(|| invalid("DML schema absent"))?,
                )
                .ok_or_else(|| invalid("DML schema disappeared"))?;
            schema
                .downcast_ref::<SnapshotSchema>()
                .ok_or_else(|| invalid("DML requires an owned native schema"))?
                .bind(replacement);
            Ok(count)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            output_schema,
            stream,
        )))
    }
}
fn count_result(batches: &[RecordBatch], schema: SchemaRef) -> Result<RecordBatch> {
    if batches.len() != 1 || batches[0].num_rows() != 1 || batches[0].num_columns() != 1 {
        return Err(invalid(
            "native DML must return exactly one affected-row count",
        ));
    }
    let count = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .filter(|count| count.null_count() == 0)
        .ok_or_else(|| invalid("native DML count must be nonnull UInt64"))?;
    Ok(RecordBatch::try_new(schema, vec![Arc::new(count.clone())])?)
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}
fn external(error: crate::CatalogError) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

#[cfg(test)]
mod tests;
