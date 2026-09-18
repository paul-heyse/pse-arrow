// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native composition of the finite process-model algorithms. There is no stage
//! executor: DataFusion plans, schedules and executes every dependency.
pub mod edit;
mod evidence;
mod execution;
mod layout;
mod source;
mod value;
pub use value::Value;
use value::{Argument, Child};
pub mod model;

use crate::{
    Algorithm, CompilerError,
    passes::{invalid, native_rows::engine},
};
use datafusion::{
    common::{DFSchema, DFSchemaRef, Result},
    logical_expr::{Expr, Extension, LogicalPlan, UserDefinedLogicalNodeCore},
};
pub use execution::CompilerExtensionPlanner;
use pse_catalog::session::{SnapshotSession, contract::ExecutionContract};
use pse_ids::CancellationToken;
use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
struct Program {
    algorithm: Arc<dyn Algorithm>,
    arguments: Vec<Argument>,
    absent: BTreeSet<String>,
    layout: Arc<layout::Layout>,
}
/// A specialized finite algorithm with real native relational dependencies.
/// It cannot publish a stage, access a store, look up a memo or restore a snapshot.
#[derive(Clone)]
pub struct DomainAlgorithm {
    program: Arc<Program>,
    inputs: Vec<Child>,
    schema: DFSchemaRef,
}
impl std::fmt::Debug for DomainAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_for_explain(f)
    }
}
/// Bind exact native arguments, and expose every result through ordinary UNNEST.
/// The actual caller's cache factory shares the finite result across its consumers.
/// Planning, inspecting and optimizing this graph executes no domain work.
/// # Errors
/// Missing/extra arguments, mismatched declarations, or native planning refusal.
#[expect(
    clippy::too_many_lines,
    reason = "plan keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub async fn plan(
    algorithm: Arc<dyn Algorithm>,
    mut arguments: BTreeMap<String, Option<Value>>,
    documents: Value,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> std::result::Result<BTreeMap<String, Value>, CompilerError> {
    let registry = session.registry();
    if arguments.keys().cloned().collect::<BTreeSet<_>>()
        != algorithm
            .spec()
            .inputs
            .iter()
            .map(|port| port.port.clone())
            .collect()
    {
        return Err(invalid("native algorithm argument inventory differs"));
    }
    let supplied = arguments
        .values()
        .flatten()
        .map(|value| {
            let spec = registry
                .relation_by_id(value.relation.relation_id())
                .ok_or_else(|| invalid("argument declaration absent"))?;
            Ok((spec.key, value.relation.clone()))
        })
        .collect::<std::result::Result<BTreeMap<_, _>, CompilerError>>()?;
    let preconditions = requirements(
        session,
        supplied.clone(),
        &algorithm.spec().preconditions,
        cancel,
    )
    .await?;
    let mut bindings = Vec::new();
    let mut inputs = Vec::new();
    let mut absent = BTreeSet::new();
    for port in &algorithm.spec().inputs {
        let input = arguments.remove(&port.port).flatten();
        match input {
            Some(input) => {
                let spec = registry
                    .relation(&port.relation)
                    .ok_or_else(|| invalid("algorithm argument undeclared"))?;
                if input.relation.relation_id() != spec.id {
                    return Err(invalid(format!(
                        "argument {} has another relation",
                        port.port
                    )));
                }
                value::bind(input, Some(port.port.clone()), &mut inputs, &mut bindings);
            }
            None if port.required => {
                return Err(invalid(format!("required argument {} absent", port.port)));
            }
            None => {
                absent.insert(port.port.clone());
            }
        }
    }
    if documents.relation.relation_id()
        != pse_relations::generated::authored::documents::spec(registry)?.id
    {
        return Err(invalid(
            "parser input is not the declared document relation",
        ));
    }
    value::bind(documents, None, &mut inputs, &mut bindings);
    let layout = Arc::new(layout::Layout::new(registry, algorithm.spec())?);
    let schema = Arc::new(DFSchema::try_from(layout.schema.as_ref().clone()).map_err(engine)?);
    let effects = algorithm.spec().effects.clone();
    let program = Arc::new(Program {
        algorithm,
        arguments: bindings,
        absent,
        layout,
    });
    let node = DomainAlgorithm {
        program: Arc::clone(&program),
        inputs,
        schema,
    };
    let tuple = ExecutionContract::plan(
        LogicalPlan::Extension(Extension {
            node: Arc::new(node),
        }),
        preconditions,
        effects.clone(),
    );
    let tuple = session.cache_plan(tuple, cancel)?;
    let unpacked = program.layout.unpack_plans(&tuple, registry)?;
    let mut result_scope = supplied;
    for output in &program.algorithm.spec().outputs {
        let value = unpacked
            .get(&output.port)
            .ok_or_else(|| invalid("result value absent"))?;
        let key = registry
            .relation_by_id(value.relation_id())
            .ok_or_else(|| invalid("result declaration absent"))?
            .key;
        result_scope.insert(key, value.clone());
    }
    let postconditions = requirements(
        session,
        result_scope,
        &program.algorithm.spec().postconditions,
        cancel,
    )
    .await?;
    let findings = LogicalPlan::Filter(
        datafusion::logical_expr::Filter::try_new(
            datafusion::logical_expr::col("severity").eq(datafusion::logical_expr::lit("error")),
            Arc::new(unpacked["__findings"].plan().clone()),
        )
        .map_err(engine)?,
    );
    let violations = match postconditions {
        Some(plan) => datafusion::logical_expr::LogicalPlanBuilder::from(plan)
            .union(findings)
            .map_err(engine)?
            .build()
            .map_err(engine)?,
        None => findings,
    };
    let tuple = session.cache_plan(
        ExecutionContract::plan(tuple, Some(violations), effects),
        cancel,
    )?;
    let tuple = Arc::new(value::Tuple {
        plan: tuple,
        layout: Arc::clone(&program.layout),
        identity: Arc::new(()),
    });
    Ok(program
        .layout
        .unpack_plans(&tuple.plan, registry)?
        .into_iter()
        .map(|(name, relation)| {
            (
                name.clone(),
                Value {
                    relation,
                    tuple: Some((Arc::clone(&tuple), name)),
                },
            )
        })
        .collect())
}
impl PartialEq for DomainAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.program, &other.program)
            && self
                .inputs
                .iter()
                .map(Child::plan)
                .eq(other.inputs.iter().map(Child::plan))
    }
}
impl Eq for DomainAlgorithm {}
impl Hash for DomainAlgorithm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.program).hash(state);
        for input in &self.inputs {
            input.plan().hash(state);
        }
    }
}
impl PartialOrd for DomainAlgorithm {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.program).cmp(&Arc::as_ptr(&other.program)) {
            std::cmp::Ordering::Equal => self
                .inputs
                .iter()
                .map(Child::plan)
                .partial_cmp(other.inputs.iter().map(Child::plan)),
            ordering => Some(ordering),
        }
    }
}
impl UserDefinedLogicalNodeCore for DomainAlgorithm {
    fn name(&self) -> &'static str {
        "ProcessDomainAlgorithm"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        self.inputs.iter().map(Child::plan).collect()
    }
    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }
    fn expressions(&self) -> Vec<Expr> {
        Vec::new()
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProcessDomainAlgorithm: {}, arguments={}, results={}",
            self.program.algorithm.spec().qualified_name(),
            self.inputs.len(),
            self.schema.fields().len()
        )
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if !expressions.is_empty() || inputs.len() != self.inputs.len() {
            return Err(datafusion::common::DataFusionError::Plan(
                "domain algorithm children differ".into(),
            ));
        }
        Ok(Self {
            inputs: self
                .inputs
                .iter()
                .zip(inputs)
                .map(|(before, plan)| before.rewritten(plan))
                .collect(),
            ..self.clone()
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests;

async fn requirements(
    session: &SnapshotSession,
    inputs: BTreeMap<pse_schema::model::RelationKey, pse_catalog::session::RelationPlan>,
    declarations: &[String],
    cancel: &CancellationToken,
) -> std::result::Result<Option<LogicalPlan>, CompilerError> {
    use pse_catalog::session::policy::RequirementPlanner;
    if declarations.is_empty() {
        return Ok(None);
    }
    let selected = declarations
        .iter()
        .map(|name| {
            session
                .registry()
                .invariant_id(name)
                .ok_or_else(|| invalid(format!("algorithm obligation {name} is undeclared")))
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let workspace = session.plan_workspace(inputs, cancel)?;
    Ok(Some(
        pse_rules::invariants::RegistryRequirementPlanner
            .plan(&workspace, &selected, cancel)
            .await?,
    ))
}
