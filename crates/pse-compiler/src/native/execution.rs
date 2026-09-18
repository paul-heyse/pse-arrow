// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The finite domain boundary consumes only its actual executed Arrow children.
use super::{DomainAlgorithm, Program, value::Child};
use crate::{AlgorithmContext, AlgorithmInputs, BoundInput, CompilerError};
use datafusion::{
    catalog::Session,
    common::{DataFusionError, Result, tree_node::TreeNodeRecursion},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{
        LogicalPlan, UserDefinedLogicalNode, physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, ExecutionPlanProperties, Partitioning,
        PlanProperties, SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::TryStreamExt;
use pse_catalog::session::{RelationFacts, execution::NativeExecutionContext};
use std::{collections::BTreeMap, sync::Arc};

/// Compose finite process-model algorithms with native rules and Delta planners.
#[derive(Debug)]
pub struct CompilerExtensionPlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for CompilerExtensionPlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<DomainAlgorithm>() else {
            return Ok(None);
        };
        let services = NativeExecutionContext::from_session(session)?;
        services
            .admit_effects(&node.program.algorithm.spec().effects)
            .map_err(|error| native(error.into()))?;
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| invalid("domain algorithm requires actual caller state"))?;
        validate(&node.inputs, inputs)?;
        Ok(Some(Arc::new(DomainExec {
            program: Arc::clone(&node.program),
            sources: node.inputs.clone(),
            inputs: inputs.to_vec(),
            services,
            state: Arc::new(state.clone()),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(Arc::clone(&node.program.layout.schema)),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        })))
    }
}
fn validate(sources: &[Child], inputs: &[Arc<dyn ExecutionPlan>]) -> Result<()> {
    if inputs.len() != sources.len()
        || inputs
            .iter()
            .any(|input| input.boundedness().is_unbounded())
    {
        return Err(invalid(
            "domain algorithm requires all bounded native inputs",
        ));
    }
    for (source, child) in sources.iter().zip(inputs) {
        if source.plan().schema().fields() != child.schema().fields() {
            return Err(invalid("domain child fields changed"));
        }
    }
    Ok(())
}
struct DomainExec {
    program: Arc<Program>,
    sources: Vec<Child>,
    inputs: Vec<Arc<dyn ExecutionPlan>>,
    services: Arc<NativeExecutionContext>,
    state: Arc<SessionState>,
    properties: Arc<PlanProperties>,
}
impl std::fmt::Debug for DomainExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_as(DisplayFormatType::Default, f)
    }
}
impl DisplayAs for DomainExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProcessDomainExec: {}, children={}",
            self.program.algorithm.spec().qualified_name(),
            self.inputs.len()
        )
    }
}
impl ExecutionPlan for DomainExec {
    fn name(&self) -> &'static str {
        "ProcessDomainExec"
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
        validate(&self.sources, &inputs)?;
        Ok(Arc::new(Self {
            program: Arc::clone(&self.program),
            sources: self.sources.clone(),
            inputs,
            services: Arc::clone(&self.services),
            state: Arc::clone(&self.state),
            properties: Arc::clone(&self.properties),
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
        if partition != 0 {
            return Err(invalid("domain algorithm has one output partition"));
        }
        let program = Arc::clone(&self.program);
        let sources = self.sources.clone();
        let inputs = self.inputs.clone();
        let state = Arc::clone(&self.state);
        let services = Arc::clone(&self.services);
        let stream = futures_util::stream::once(async move {
            run(&program, &sources, inputs, state, services, context)
                .await
                .map_err(native)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "run keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn run(
    program: &Program,
    sources: &[Child],
    inputs: Vec<Arc<dyn ExecutionPlan>>,
    state: Arc<SessionState>,
    services: Arc<NativeExecutionContext>,
    context: Arc<TaskContext>,
) -> std::result::Result<datafusion::arrow::array::RecordBatch, CompilerError> {
    let cancel = services.cancellation();
    let mut bound = AlgorithmInputs::new();
    for role in &program.absent {
        bound.ports.insert(role.clone(), None);
    }
    let mut roles = BTreeMap::new();
    let mut documents = None;
    let mut projected_roles = BTreeMap::new();
    for (index, (source, input)) in sources.iter().zip(inputs).enumerate() {
        let captured = match source {
            Child::Relation(source) => Some(Arc::new(
                source
                    .capture(Arc::clone(&input), Arc::clone(&context), &services)
                    .await?,
            )),
            Child::Tuple(_) => None,
        };
        let tuple = if captured.is_none() {
            let mut stream = datafusion::physical_plan::execute_stream(input, Arc::clone(&context))
                .map_err(crate::passes::native_rows::engine)?;
            let batch = stream
                .try_next()
                .await
                .map_err(crate::passes::native_rows::engine)?
                .ok_or_else(|| crate::passes::invalid("algorithm tuple child is empty"))?;
            if stream
                .try_next()
                .await
                .map_err(crate::passes::native_rows::engine)?
                .is_some()
            {
                return Err(crate::passes::invalid("algorithm tuple child repeated"));
            }
            Some(pse_ids::owned_buffer::OwnedRecordBatch::export(
                batch,
                services.reserver().as_ref(),
                cancel,
            )?)
        } else {
            None
        };
        for argument in program
            .arguments
            .iter()
            .filter(|argument| argument.child == index)
        {
            let facts = match (source, &argument.member, &tuple, &captured) {
                (Child::Tuple(source), Some(member), Some(tuple), _) => {
                    Arc::new(RelationFacts::from_checked(source.layout.member(
                        member,
                        tuple,
                        services.registry(),
                    )?))
                }
                (Child::Relation(_), None, _, Some(facts)) => Arc::clone(facts),
                _ => return Err(crate::passes::invalid("algorithm child selector differs")),
            };
            if let Some(role) = &argument.role {
                let input = BoundInput::declared(
                    facts.clone(),
                    program.algorithm.spec(),
                    role,
                    services.registry(),
                )?;
                if input.relation().is_ok() {
                    roles.insert(role.to_owned(), facts.checked().clone());
                } else {
                    let spec = services
                        .registry()
                        .relation_by_id(input.relation_id())
                        .ok_or_else(|| crate::passes::invalid("argument relation absent"))?;
                    projected_roles.insert(role.to_owned(), (spec.key, input.fields().clone()));
                }
                bound.ports.insert(role.clone(), Some(input));
            } else {
                documents = Some(facts);
            }
        }
    }
    let session = services
        .candidate_roles(&state, roles)?
        .with_projected_argument_roles(projected_roles, cancel)?;
    let document_facts =
        documents.ok_or_else(|| crate::passes::invalid("parser argument absent"))?;
    let documents = crate::documents::parse(document_facts.as_ref(), &session, cancel)?;
    let physical = if program.algorithm.requires_physical() {
        Some(Arc::new(
            crate::quantity_relations::PhysicalInventory::load(
                &session,
                services.registry(),
                cancel,
            )
            .await?,
        ))
    } else {
        None
    };
    let ctx = AlgorithmContext {
        physical: physical.as_ref(),
        registry: services.registry(),
        documents: &documents,
        cancel,
        reserver: services.reserver().as_ref(),
        session: &session,
    };
    let output = program.algorithm.run(&ctx, &bound).await?;
    program.layout.pack(output, &session, cancel)
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn native(error: CompilerError) -> DataFusionError {
    DataFusionError::External(Box::new(pse_catalog::CatalogError::Semantic(Arc::new(
        error,
    ))))
}
