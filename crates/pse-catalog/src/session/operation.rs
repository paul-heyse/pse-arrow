// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native operation roots for contracted algorithms and multi-output commands.

mod input;

use super::{PreparedComputation, SnapshotSession};
use crate::{BoxFut, CatalogError};
use datafusion::{
    arrow::{array::RecordBatch, datatypes::SchemaRef},
    common::{DFSchema, DFSchemaRef, DataFusionError, Result},
    execution::TaskContext,
    logical_expr::{Expr, LogicalPlan, UserDefinedLogicalNodeCore},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
};
use pse_ids::CancellationToken;
use pse_schema::model::provider::OperationEffect;
use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// The actual implementation of a bound command or specialized algorithm.
/// Its single native outcome stream is separate from its privately owned output ports.
pub trait NativeOperation: std::fmt::Debug + Send + Sync {
    /// Retain irreversible external outcomes when execution or delivery fails.
    fn execution_failure(&self, error: CatalogError) -> CatalogError {
        error
    }
    /// True only after this actual operation completed an irreversible effect.
    /// Later cancellation cannot turn that completed effect into a rollback report.
    fn irreversible_completion(&self) -> bool {
        false
    }
    /// The body owns cancellation checkpoints, including indivisible commit sections.
    /// Such a body must wake pending reads on cancellation and finish an initiated
    /// conditional write before reporting its actual outcome.
    fn handles_cancellation(&self) -> bool {
        false
    }
    /// Registered operation name, used for plan display rather than admission identity.
    fn name(&self) -> &str;
    /// Exact declared native outcome schema.
    fn schema(&self) -> SchemaRef;
    /// Effects of this actual implementation's invocation.
    fn effects(&self) -> BTreeSet<OperationEffect>;
    /// Captured immutable control literals for native plan inspection.
    fn arguments(&self) -> Vec<Expr> {
        Vec::new()
    }
    /// Execute once under the retained resource/source scope, before exposing completion.
    fn execute<'a>(
        &'a self,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, std::result::Result<RecordBatch, CatalogError>>;
}

#[derive(Clone)]
pub(crate) struct OperationNode {
    pub(crate) operation: Arc<dyn NativeOperation>,
    pub(crate) effects: BTreeSet<OperationEffect>,
    schema: DFSchemaRef,
    inputs: Vec<LogicalPlan>,
    arguments: Vec<Expr>,
    started: Arc<AtomicBool>,
}
impl std::fmt::Debug for OperationNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Operation")
            .field("name", &self.operation.name())
            .field("effects", &self.effects)
            .field("arguments", &self.arguments)
            .field("input_count", &self.inputs.len())
            .finish_non_exhaustive()
    }
}
impl PartialEq for OperationNode {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.operation, &other.operation)
            && Arc::ptr_eq(&self.started, &other.started)
            && self.inputs == other.inputs
    }
}
impl Eq for OperationNode {}
impl Hash for OperationNode {
    fn hash<H: Hasher>(&self, h: &mut H) {
        Arc::as_ptr(&self.started).hash(h);
        self.inputs.hash(h);
    }
}
impl PartialOrd for OperationNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.started).cmp(&Arc::as_ptr(&other.started)) {
            std::cmp::Ordering::Equal => self.inputs.partial_cmp(&other.inputs),
            order => Some(order),
        }
    }
}
impl UserDefinedLogicalNodeCore for OperationNode {
    fn name(&self) -> &str {
        self.operation.name()
    }
    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        self.inputs.iter().collect()
    }
    fn expressions(&self) -> Vec<Expr> {
        self.arguments.clone()
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation: {} effects={:?} arguments={:?}",
            self.operation.name(),
            self.effects,
            self.arguments
        )
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if expressions != self.arguments || inputs.len() != self.inputs.len() {
            return Err(invalid(
                "operation inputs differ from the captured contract",
            ));
        }
        Ok(Self {
            inputs,
            ..self.clone()
        })
    }
}
impl SnapshotSession {
    /// Prepare one actual native operation over the captured provider scopes.
    /// # Errors
    /// Effects, fields, inputs, policy requirements or analysis do not satisfy the contract.
    pub fn prepare_operation(
        &self,
        operation: Arc<dyn NativeOperation>,
        cancel: &CancellationToken,
    ) -> std::result::Result<PreparedComputation, CatalogError> {
        let mut inputs = Vec::new();
        for (key, binding) in self.bindings.iter() {
            if matches!(key, crate::provider::binding::BindingKey::Output(_)) {
                continue;
            }
            inputs.push(input::scoped(
                datafusion::logical_expr::LogicalPlanBuilder::scan(
                    binding.reference.clone(),
                    datafusion::datasource::provider_as_source(Arc::clone(&binding.provider)),
                    None,
                )
                .map_err(super::snapshot_session::engine)?
                .build()
                .map_err(super::snapshot_session::engine)?,
            ));
        }
        let node = OperationNode::new(operation, inputs)?;
        self.prepare(
            LogicalPlan::Extension(datafusion::logical_expr::Extension {
                node: Arc::new(node),
            }),
            cancel,
        )
    }
}

impl OperationNode {
    pub(crate) fn new(
        operation: Arc<dyn NativeOperation>,
        inputs: Vec<LogicalPlan>,
    ) -> std::result::Result<Self, CatalogError> {
        Ok(Self {
            effects: operation.effects(),
            arguments: operation.arguments(),
            schema: Arc::new(
                DFSchema::try_from(operation.schema()).map_err(super::snapshot_session::engine)?,
            ),
            operation,
            inputs,
            started: Arc::new(AtomicBool::new(false)),
        })
    }
    pub(crate) fn physical(
        &self,
        session: SnapshotSession,
        cancel: CancellationToken,
    ) -> Arc<dyn ExecutionPlan> {
        Arc::new(OperationExec {
            node: self.clone(),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(Arc::new(self.schema.as_arrow().clone())),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
            session,
            cancel,
        })
    }
}
#[derive(Debug)]
struct OperationExec {
    node: OperationNode,
    properties: Arc<PlanProperties>,
    session: SnapshotSession,
    cancel: CancellationToken,
}
impl DisplayAs for OperationExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OperationExec: {}", self.node.operation.name())
    }
}
impl ExecutionPlan for OperationExec {
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn name(&self) -> &'static str {
        "OperationExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.is_empty() {
            Ok(self)
        } else {
            Err(invalid("operation owns its captured inputs"))
        }
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.node.started.swap(true, Ordering::AcqRel) {
            return Err(invalid(
                "operation executes only once in its single partition",
            ));
        }
        let schema = Arc::new(self.node.schema.as_arrow().clone());
        let output_schema = Arc::clone(&schema);
        let operation = Arc::clone(&self.node.operation);
        let session = self.session.clone();
        let cancel = self.cancel.clone();
        let stream = futures_util::stream::once(async move {
            cancel
                .checkpoint()
                .map_err(|e| DataFusionError::External(Box::new(CatalogError::from(e))))?;
            let batch = operation
                .execute(&session, &cancel)
                .await
                .map_err(|e| DataFusionError::External(Box::new(e)))?;
            if batch.schema() != schema {
                return Err(invalid("operation outcome schema differs from declaration"));
            }
            if !operation.irreversible_completion() {
                cancel
                    .checkpoint()
                    .map_err(|e| DataFusionError::External(Box::new(CatalogError::from(e))))?;
            }
            Ok(batch)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            output_schema,
            stream,
        )))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}
