// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private replacement is a native TableProvider write contract at any plan depth.
use super::super::{EngineSession, physical_input::PhysicalInput};
use crate::provider::{binding::TableBinding, schema::SnapshotSchema};
use datafusion::{
    arrow::{
        array::{Array, RecordBatch, UInt64Array},
        datatypes::SchemaRef,
    },
    catalog::{Session, TableProvider},
    common::{Constraints, DFSchemaRef, DataFusionError, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, MergeIntoClause, TableType},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, execution_plan::Boundedness},
};
use datafusion_expr::dml::InsertOp;
use futures_util::TryStreamExt;
use std::sync::Arc;

mod isolation;
pub(crate) use isolation::isolate;

#[derive(Debug)]
struct IsolatedTarget {
    binding: Arc<TableBinding>,
    session: EngineSession,
}

// Only this module can construct the private target. Admission follows its actual
// retained binding; schema or name equality cannot certify a foreign provider.
pub(crate) fn bound_source(target: &Arc<dyn TableProvider>) -> &Arc<dyn TableProvider> {
    target
        .downcast_ref::<IsolatedTarget>()
        .map_or(target, |target| &target.binding.provider)
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
        let schema = Arc::new(datafusion::arrow::datatypes::Schema::new(vec![
            datafusion::arrow::datatypes::Field::new(
                "count",
                datafusion::arrow::datatypes::DataType::UInt64,
                false,
            ),
        ]));
        let children = source.iter().chain(input.iter()).cloned().collect();
        crate::operation::Execution::plan(
            "IsolatedWrite",
            schema.clone(),
            children,
            Arc::new(IsolatedWriteBody {
                action,
                has_input: input.is_some(),
                has_source: source.is_some(),
                binding: self.binding.clone(),
                session: self.session.clone(),
                state: actual.clone(),
                schema,
            }),
            crate::operation::Family::Command,
            Arc::default(),
            super::super::execution::NativeExecutionContext::from_session(actual)?,
        )
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
    fn statistics(&self) -> Option<datafusion::common::Statistics> {
        self.binding.provider.statistics()
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<datafusion::logical_expr::TableProviderFilterPushDown>> {
        self.binding.provider.supports_filters_pushdown(filters)
    }
    async fn scan_with_args<'a>(
        &self,
        state: &dyn Session,
        args: datafusion::catalog::ScanArgs<'a>,
    ) -> Result<datafusion::catalog::ScanResult> {
        self.binding.provider.scan_with_args(state, args).await
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
#[derive(Clone, Debug)]
struct IsolatedWriteBody {
    action: NativeWrite,
    has_input: bool,
    has_source: bool,
    binding: Arc<TableBinding>,
    session: EngineSession,
    state: SessionState,
    schema: SchemaRef,
}
impl crate::operation::Body for IsolatedWriteBody {
    fn execute(
        &self,
        children: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        if children.len() != usize::from(self.has_input) + usize::from(self.has_source) {
            return Err(invalid("private DML child count changed"));
        }
        let mut children = children.into_iter();
        let source = if self.has_source {
            children.next()
        } else {
            None
        };
        let input = if self.has_input {
            children.next()
        } else {
            None
        };
        let body = self.clone();
        Ok(crate::operation::batch(self.schema.clone(), async move {
            body.run(input, source).await
        }))
    }
}
impl IsolatedWriteBody {
    async fn run(
        self,
        input: Option<Arc<dyn ExecutionPlan>>,
        source: Option<Arc<dyn ExecutionPlan>>,
    ) -> Result<RecordBatch> {
        let binding = Arc::clone(&self.binding);
        let mut session = self.session.clone();
        let state = self.state.clone();
        let action = self.action.clone();
        let schema = self.schema.clone();
        let services = super::super::execution::NativeExecutionContext::from_session(&state)?;
        services.inherit_invocation(&mut session);
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
            let batches = datafusion::physical_plan::execute_stream(physical, state.task_ctx())?
                .try_collect::<Vec<_>>()
                .await?;
            return count_result(&batches, schema);
        };
        let source: Arc<dyn TableProvider> = Arc::new(
            PhysicalInput::native(source.ok_or_else(|| invalid("private source absent"))?)
                .with_defaults(binding.provider.as_ref()),
        );
        let target = factory.begin(source, &state).await?;
        if Arc::ptr_eq(&target, &binding.provider) || target.schema() != binding.provider.schema() {
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
        replacement.witness = None;
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
fn external(error: crate::EngineError) -> DataFusionError {
    pse_columnar::external(error)
}

#[cfg(test)]
mod tests;
