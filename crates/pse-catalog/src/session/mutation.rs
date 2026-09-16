// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native DML owns an isolated mutable target and admits its replacement before exposure.

mod deferred;

use super::{
    SnapshotSession,
    capture::CapturedProvider,
    operation::{NativeOperation, OperationNode},
};
use crate::{BoxFut, CatalogError, provider::binding::TableBinding};
use datafusion::{
    arrow::{
        array::{Array, RecordBatch, UInt64Array},
        datatypes::SchemaRef,
    },
    catalog::{Session, TableProvider},
    common::{Result as NativeResult, TableReference, tree_node::Transformed},
    datasource::{MemTable, provider_as_source, source_as_provider},
    logical_expr::{Expr, LogicalPlan, lit},
    physical_plan::ExecutionPlan,
};
use futures_util::TryStreamExt;
use pse_ids::CancellationToken;
use pse_schema::model::provider::OperationEffect;
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

/// Actual factory for an exclusively owned candidate. It must isolate mutable backing
/// from the source and all other attempts, preserve the exact schema/default semantics,
/// and perform no externally visible writes. Native hooks report their actual support.
/// This is a contract on the bound implementation, not a capability flag or provider roster.
pub trait PrivateTableFactory: std::fmt::Debug + Send + Sync {
    /// Construct the private target during native execution, using the shared runtime.
    /// # Errors
    /// Unsupported isolation, schema/default incompatibility, resource or native failure.
    fn begin<'a>(
        &'a self,
        source: Arc<dyn TableProvider>,
        state: &'a dyn Session,
    ) -> BoxFut<'a, NativeResult<Arc<dyn TableProvider>>>;
}

/// DataFusion's native in-memory DML hooks over fresh partition locks and retained buffers.
/// At the pinned release these support append INSERT, DELETE and UPDATE; other hooks
/// retain the engine's truthful unsupported result.
#[derive(Debug, Default)]
pub struct MemoryTableFactory;
impl PrivateTableFactory for MemoryTableFactory {
    fn begin<'a>(
        &'a self,
        source: Arc<dyn TableProvider>,
        state: &'a dyn Session,
    ) -> BoxFut<'a, NativeResult<Arc<dyn TableProvider>>> {
        Box::pin(async move {
            let defaults = source
                .schema()
                .fields()
                .iter()
                .filter_map(|field| {
                    source
                        .get_column_default(field.name())
                        .map(|default| (field.name().to_owned(), default.clone()))
                })
                .collect();
            let table: Arc<dyn TableProvider> = Arc::new(
                MemTable::load(source, None, state)
                    .await?
                    .with_column_defaults(defaults),
            );
            Ok(table)
        })
    }
}

impl SnapshotSession {
    /// Enable private DML on an actually captured source generation using a bound factory.
    /// Every successful mutation returns a new immutable generation; existing readers keep
    /// their original source. Failed or dropped attempts expose no replacement.
    /// # Errors
    /// Foreign capture, conflicting name, invalid fields or cancellation.
    pub fn with_mutable_capture(
        &self,
        reference: &TableReference,
        captured: &CapturedProvider,
        factory: Arc<dyn PrivateTableFactory>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut session = self.with_captured_provider(reference.clone(), captured, cancel)?;
        session
            .bindings
            .mutation_factory(reference, factory)
            .map_err(super::engine)?;
        Ok(session)
    }
}

#[derive(Debug)]
pub(super) struct PreparedMutation {
    operation: Arc<Mutation>,
    node: OperationNode,
    outer: LogicalPlan,
}
impl PreparedMutation {
    pub(super) fn bind(
        session: &SnapshotSession,
        plan: &LogicalPlan,
    ) -> Result<Option<Self>, CatalogError> {
        let body = match plan {
            LogicalPlan::Explain(explain) => explain.plan.as_ref(),
            plan => plan,
        };
        let LogicalPlan::Dml(dml) = body else {
            let mut nested = false;
            plan.apply_with_subqueries(|node| {
                nested |= matches!(node, LogicalPlan::Dml(_));
                Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
            })
            .map_err(super::engine)?;
            if nested {
                return Err(invalid(
                    "nested DML requires its own private operation root",
                ));
            }
            return Ok(None);
        };
        let source = source_as_provider(&dml.target).map_err(super::engine)?;
        let defaults = session.context.state();
        let config = &defaults.config_options().catalog;
        let name = dml
            .table_name
            .clone()
            .resolve(&config.default_catalog, &config.default_schema);
        let reference = TableReference::full(name.catalog, name.schema, name.table);
        let binding = session
            .bindings
            .iter()
            .find_map(|(_, binding)| {
                (binding.reference == reference && Arc::ptr_eq(&binding.provider, &source))
                    .then_some(binding.clone())
            })
            .ok_or_else(|| invalid("DML target is not its exact bound source"))?;
        if binding.mutation.is_none() {
            return Err(invalid(
                "DML target has no bound private-table factory; capture it and bind an isolation implementation",
            ));
        }
        let operation = Arc::new(Mutation {
            plan: body.clone(),
            binding,
            output: Mutex::new(None),
        });
        let native: Arc<dyn NativeOperation> = operation.clone();
        let node = OperationNode::new(native, vec![dml.input.as_ref().clone()])?;
        Ok(Some(Self {
            operation,
            node,
            outer: plan.clone(),
        }))
    }
    pub(super) async fn physical(
        &self,
        session: SnapshotSession,
        cancel: CancellationToken,
        state: &datafusion::execution::session_state::SessionState,
    ) -> NativeResult<Arc<dyn ExecutionPlan>> {
        let target: Arc<dyn TableProvider> = Arc::new(deferred::DeferredTarget {
            source: Arc::clone(&self.operation.binding.provider),
            physical: self.node.physical(session, cancel),
        });
        let plan = self
            .outer
            .clone()
            .transform_up_with_subqueries(|mut node| {
                if let LogicalPlan::Dml(dml) = &mut node {
                    dml.target = provider_as_source(Arc::clone(&target));
                    return Ok(Transformed::yes(node));
                }
                Ok(Transformed::no(node))
            })?
            .data;
        state
            .query_planner()
            .create_physical_plan(&plan, state)
            .await
    }
    pub(super) fn resulting_session(
        &self,
        session: &SnapshotSession,
    ) -> Result<SnapshotSession, CatalogError> {
        if matches!(self.outer, LogicalPlan::Explain(_)) {
            return Ok(session.clone());
        }
        let output = self
            .operation
            .output
            .lock()
            .map_err(|_| invalid("mutation completion unavailable"))?
            .clone()
            .ok_or_else(|| invalid("mutation did not complete admission"))?;
        let mut session = session.clone();
        session
            .bindings
            .replace_table(&self.operation.binding.reference, &output);
        Ok(session)
    }
}

#[derive(Debug)]
struct Mutation {
    plan: LogicalPlan,
    binding: TableBinding,
    output: Mutex<Option<TableBinding>>,
}
impl NativeOperation for Mutation {
    fn name(&self) -> &'static str {
        "provider.private_dml"
    }
    fn schema(&self) -> SchemaRef {
        Arc::new(self.plan.schema().as_arrow().clone())
    }
    fn effects(&self) -> BTreeSet<OperationEffect> {
        BTreeSet::from([OperationEffect::Read, OperationEffect::Write])
    }
    fn arguments(&self) -> Vec<Expr> {
        vec![lit(self.binding.reference.to_string())]
    }
    fn execute<'a>(
        &'a self,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, Result<RecordBatch, CatalogError>> {
        Box::pin(async move {
            cancel.checkpoint()?;
            let state = session.bound_state()?;
            let factory = self
                .binding
                .mutation
                .as_ref()
                .ok_or_else(|| invalid("private factory absent"))?;
            let target = factory
                .begin(Arc::clone(&self.binding.provider), &state)
                .await
                .map_err(super::engine)?;
            if Arc::ptr_eq(&target, &self.binding.provider)
                || target.schema() != self.binding.provider.schema()
            {
                return Err(invalid(
                    "private factory must return a distinct target with the exact source schema",
                ));
            }
            let plan = retarget(&self.plan, &self.binding, &target, &state)?;
            let physical = state
                .query_planner()
                .create_physical_plan(&plan, &state)
                .await
                .map_err(super::engine)?;
            let batches: Vec<_> =
                datafusion::physical_plan::execute_stream(physical, state.task_ctx())
                    .map_err(super::engine)?
                    .try_collect()
                    .await
                    .map_err(super::engine)?;
            let result = count_result(&batches, self.schema())?;
            cancel.checkpoint()?;
            let mut validation = session.clone();
            let reference = self.binding.reference.clone();
            validation.bindings.retain(|key| !matches!(key, crate::provider::binding::BindingKey::Native(name) if *name == reference));
            let captured = validation
                .prepare_capture_constraints(
                    reference,
                    target,
                    self.binding
                        .provider
                        .constraints()
                        .cloned()
                        .unwrap_or_default(),
                    false,
                    cancel,
                )?
                .execute(cancel)
                .await?;
            let mut replacement = self.binding.clone();
            replacement.provider = captured.provider();
            replacement.checked = None;
            let mut admitted = session.clone();
            admitted
                .bindings
                .replace_table(&self.binding.reference, &replacement);
            admitted.check_requirements(cancel).await?;
            *self
                .output
                .lock()
                .map_err(|_| invalid("mutation completion unavailable"))? = Some(replacement);
            Ok(result)
        })
    }
}

fn retarget(
    plan: &LogicalPlan,
    binding: &TableBinding,
    target: &Arc<dyn TableProvider>,
    state: &datafusion::execution::session_state::SessionState,
) -> Result<LogicalPlan, CatalogError> {
    let defaults = &state.config_options().catalog;
    plan.clone()
        .transform_up_with_subqueries(|mut node| {
            if let LogicalPlan::TableScan(scan) = &mut node {
                let name = scan
                    .table_name
                    .clone()
                    .resolve(&defaults.default_catalog, &defaults.default_schema);
                if TableReference::full(name.catalog, name.schema, name.table) == binding.reference
                    && Arc::ptr_eq(&source_as_provider(&scan.source)?, &binding.provider)
                {
                    scan.source = provider_as_source(Arc::clone(target));
                    return Ok(Transformed::yes(node));
                }
            }
            if let LogicalPlan::Dml(dml) = &mut node {
                dml.target = provider_as_source(Arc::clone(target));
                return Ok(Transformed::yes(node));
            }
            Ok(Transformed::no(node))
        })
        .map(|result| result.data)
        .map_err(super::engine)
}

fn count_result(batches: &[RecordBatch], schema: SchemaRef) -> Result<RecordBatch, CatalogError> {
    if batches.len() != 1 || batches[0].num_rows() != 1 || batches[0].num_columns() != 1 {
        return Err(invalid(
            "native DML did not produce exactly one affected-row count",
        ));
    }
    let count = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .filter(|count| count.null_count() == 0)
        .ok_or_else(|| invalid("native DML count is not a nonnull UInt64"))?;
    RecordBatch::try_new(schema, vec![Arc::new(count.clone())])
        .map_err(|error| super::engine(error.into()))
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.private_dml".into(),
        reason: reason.into(),
    }
}
