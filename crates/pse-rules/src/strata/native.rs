// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Stateful finite inference as a native multi-input, typed multi-output operator.
//! Only the iteration is specialized. Every round uses ordinary native relational
//! plans under the actual caller's configuration, policies and allocation owners.
mod execution;
mod output;

use super::{RuleBindings, RuleInputLocation, StratumLimits, admission, native_support};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::{
    common::{DFSchema, DFSchemaRef, Result},
    logical_expr::{Expr, Extension, LogicalPlan, LogicalPlanBuilder, UserDefinedLogicalNodeCore},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::{
    Registry,
    model::{RelationKey, RuleSpec},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

pub use execution::RuleExtensionPlanner;
pub(super) use output::Settled;

#[derive(Debug)]
struct Program {
    rules: Vec<RuleSpec>,
    bindings: BTreeMap<SemanticId, RuleBindings>,
    outputs: BTreeSet<RelationKey>,
    children: Vec<(String, RelationKey)>,
    limits: StratumLimits,
    layout: output::Layout,
}

/// Native fixed-point inference with real child plans and typed Arrow outputs.
/// Creating or explaining a plan performs no inference.
#[derive(Clone)]
pub struct FixedPoint {
    program: Arc<Program>,
    inputs: Vec<LogicalPlan>,
    schema: DFSchemaRef,
}
impl std::fmt::Debug for FixedPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Native JSON EXPLAIN uses Debug. Facts and their registry owners are
        // execution inputs, never an inline diagnostic dump of the whole model.
        self.fmt_for_explain(f)
    }
}

/// Bind a finite program's actual source and support scans into one native plan.
/// The returned session includes its private support roles and must be used for
/// preparation. Install [`RuleExtensionPlanner`] in the caller's composed planner.
/// # Errors
/// Invalid declarations, incomplete producers, different facts, cancellation or resources.
pub fn plan_strata(
    rules: &[RuleSpec],
    bindings: &BTreeMap<SemanticId, RuleBindings>,
    session: &SnapshotSession,
    registry: &Registry,
    outputs: &BTreeSet<RelationKey>,
    limits: StratumLimits,
    cancel: &CancellationToken,
) -> Result<(SnapshotSession, LogicalPlan), RuleError> {
    admission::validate(rules, bindings, session, registry, outputs, cancel)?;
    let mut keys = BTreeSet::new();
    let mut mappings = BTreeMap::new();
    for (rule, binding) in bindings {
        for (port, input) in &binding.ports {
            if !matches!(input.location, RuleInputLocation::Workspace) {
                keys.insert(input.relation);
            }
            if let RuleInputLocation::Native(input) = &input.location {
                mappings.insert(
                    native_support::mapping_role(*rule, port),
                    input.support_mapping().clone(),
                );
            }
        }
    }
    let session = session.with_checked_role_inputs(mappings.clone(), cancel)?;
    let mut children = Vec::new();
    let mut inputs = Vec::new();
    for key in keys {
        children.push((format!("__pse_fixed_input:{}", key.qualified_name()), key));
        inputs.push(
            LogicalPlanBuilder::scan(
                session.table_reference(&key)?,
                session.table_source(&key)?,
                None,
            )
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?,
        );
    }
    for (role, checked) in mappings {
        let key = registry
            .relation_by_id(checked.relation_id())
            .ok_or_else(|| internal("support mapping declaration absent"))?
            .key;
        inputs.push(session.scan_role(&role)?);
        children.push((role, key));
    }
    let layout = output::Layout::new(registry, outputs)?;
    let schema = Arc::new(DFSchema::try_from(layout.schema.as_ref().clone()).map_err(engine)?);
    let node = FixedPoint {
        program: Arc::new(Program {
            rules: rules.to_vec(),
            bindings: bindings.clone(),
            outputs: outputs.clone(),
            children,
            limits,
            layout,
        }),
        inputs,
        schema,
    };
    Ok((
        session,
        LogicalPlan::Extension(Extension {
            node: Arc::new(node),
        }),
    ))
}

impl PartialEq for FixedPoint {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.program, &other.program) && self.inputs == other.inputs
    }
}
impl Eq for FixedPoint {}
impl Hash for FixedPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.program).hash(state);
        self.inputs.hash(state);
    }
}
impl PartialOrd for FixedPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.program).cmp(&Arc::as_ptr(&other.program)) {
            std::cmp::Ordering::Equal => self.inputs.partial_cmp(&other.inputs),
            order => Some(order),
        }
    }
}
impl UserDefinedLogicalNodeCore for FixedPoint {
    fn name(&self) -> &'static str {
        "PseFixedPoint"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        self.inputs.iter().collect()
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
            "PseFixedPoint: rules=[{}], max_rounds={}",
            self.program
                .rules
                .iter()
                .map(RuleSpec::qualified_name)
                .collect::<Vec<_>>()
                .join(", "),
            self.program.limits.max_rounds
        )
    }
    fn with_exprs_and_inputs(&self, exprs: Vec<Expr>, inputs: Vec<LogicalPlan>) -> Result<Self> {
        if !exprs.is_empty() || inputs.len() != self.inputs.len() {
            return Err(datafusion::common::DataFusionError::Plan(
                "fixed-point input inventory changed".into(),
            ));
        }
        Ok(Self {
            inputs,
            ..self.clone()
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|field| field.name().to_owned())
            .collect()
    }
}

pub(super) async fn execute(
    node: &FixedPoint,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<super::StratumOutcome, RuleError> {
    let plan = LogicalPlan::Extension(Extension {
        node: Arc::new(node.clone()),
    });
    let completed = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?;
    let [batch] = completed.batches() else {
        return Err(internal("fixed-point execution did not return one outcome"));
    };
    let settled = node.program.layout.unpack(batch, session)?;
    let receipt = super::completed::CompletedProgram::seal(
        &settled.relations,
        &settled.derivations,
        &node.program.rules,
        session,
        cancel,
    )?;
    Ok(super::StratumOutcome {
        completed: receipt,
        relations: settled
            .relations
            .into_iter()
            .map(|(key, rows)| (key, rows.into_batch()))
            .collect(),
        derivations: settled.derivations,
        rounds: settled.rounds,
        plans: session.execution_observations()?,
    })
}
