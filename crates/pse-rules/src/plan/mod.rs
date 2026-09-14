// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Rule algebra lowered through DataFusion while retaining semantic typing derivations.
mod expr;
mod head;
mod lower;
mod recursive;

use crate::RuleError;
use crate::errmap::{engine, internal};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::SnapshotSession;
use pse_ids::SemanticId;
use pse_schema::Registry;
use pse_schema::model::{Cell, ColumnSpec, RelationKey, RuleHead, RulePlan, RuleSpec};
use std::collections::BTreeMap;

/// Explicit relation chosen for each declared input port. The sealed session owns rows.
#[derive(Clone, Debug, Default)]
pub struct PortBinding {
    /// A port cannot silently resolve to a different relation or mutable alias.
    pub ports: BTreeMap<String, RelationKey>,
}

/// One compiled rule plus actual head contracts and diagnostic-only plan text.
#[derive(Clone, Debug)]
pub struct CompiledRule {
    /// Registry rule identity.
    pub rule_id: SemanticId,
    /// Qualified declaration name.
    pub name: String,
    /// Decided-true result query.
    pub plan: LogicalPlan,
    /// Unknown candidates remain visible as their declared keys.
    pub undecided: Option<LogicalPlan>,
    /// Exact destination schema; attached only after output validation.
    pub head_schema: SchemaRef,
    /// Declared deterministic output ordering.
    pub key_columns: Vec<String>,
    /// Head declaration retained for invariant admission.
    pub head: RuleHead,
    pub(crate) checks: Vec<(LogicalPlan, String)>,
    pub(crate) contracts: Vec<Column>,
}

#[derive(Clone, Debug)]
pub(crate) struct Column {
    pub(crate) spec: ColumnSpec,
    qualifier: Option<String>,
    literal: Option<Cell>,
}
#[derive(Clone)]
struct Planned {
    plan: LogicalPlan,
    columns: Vec<Column>,
    hidden: Vec<String>,
}

struct Compiler<'a> {
    rule: &'a RuleSpec,
    binding: &'a PortBinding,
    session: &'a SnapshotSession,
    registry: &'a Registry,
    binders: Vec<(String, Planned)>,
    checks: Vec<(LogicalPlan, String)>,
    recursive_counter: usize,
}

/// Compile an admitted declaration against explicit candidate or pinned input bindings.
///
/// # Errors
/// Refuses unknown bindings, float keys, unsupported algebra and incompatible head meaning.
pub fn compile(
    rule: &RuleSpec,
    binding: &PortBinding,
    session: &SnapshotSession,
    registry: &Registry,
) -> Result<CompiledRule, RuleError> {
    let mut compiler = Compiler {
        rule,
        binding,
        session,
        registry,
        binders: vec![],
        checks: vec![],
        recursive_counter: 0,
    };
    let (decided, unknown) = split_root_predicate(&rule.plan);
    let output = compiler.lower(&decided)?;
    let target = registry
        .relation(rule.head.relation())
        .ok_or_else(|| internal("rule head is undeclared"))?;
    let keys: Vec<_> = match &rule.head {
        RuleHead::Relation(_) => target.primary_key.clone(),
        RuleHead::Violations { key_columns, .. } => key_columns.clone(),
    };
    let columns = match &rule.head {
        RuleHead::Relation(_) => target.columns.clone(),
        RuleHead::Violations { key_columns, .. } => key_columns
            .iter()
            .map(|name| {
                target
                    .column(name)
                    .cloned()
                    .ok_or_else(|| internal("violation key is undeclared"))
            })
            .collect::<Result<Vec<_>, _>>()?,
    };
    let (plan, contracts, head_schema) = head::prepare(rule, output, columns, registry)?;
    let plan = sort(plan, &keys)?;
    let undecided = unknown
        .map(|source| {
            let output = compiler.lower(&source)?;
            let expressions = keys
                .iter()
                .map(|name| expr::lookup(&output, name).map(expr::column_expression))
                .collect::<Result<Vec<_>, _>>()?;
            let plan = LogicalPlanBuilder::from(output.plan)
                .project(expressions)
                .map_err(engine)?
                .build()
                .map_err(engine)?;
            sort(plan, &keys)
        })
        .transpose()?;
    Ok(CompiledRule {
        rule_id: rule.id,
        name: rule.qualified_name(),
        plan,
        undecided,
        head_schema,
        key_columns: keys.iter().map(|key| (*key).to_owned()).collect(),
        head: rule.head.clone(),
        checks: compiler.checks,
        contracts,
    })
}

fn sort(plan: LogicalPlan, keys: &[&str]) -> Result<LogicalPlan, RuleError> {
    LogicalPlanBuilder::from(plan)
        .sort(
            keys.iter()
                .map(|key| datafusion_expr::col(*key).sort(true, true)),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)
}

// A root predicate can sit below output projections, never below membership-changing
// joins or aggregates. Internal filters retain their explicit SQL/Kleene semantics.
fn split_root_predicate(plan: &RulePlan) -> (RulePlan, Option<RulePlan>) {
    match plan {
        RulePlan::Filter { input, predicate } => (
            RulePlan::Filter {
                input: input.clone(),
                predicate: pse_schema::model::RuleExpr::IsTrue(Box::new(predicate.clone())),
            },
            Some(RulePlan::Filter {
                input: input.clone(),
                predicate: pse_schema::model::RuleExpr::IsUnknown(Box::new(predicate.clone())),
            }),
        ),
        RulePlan::Project { input, columns } => {
            let (yes, unknown) = split_root_predicate(input);
            (
                RulePlan::Project {
                    input: Box::new(yes),
                    columns: columns.clone(),
                },
                unknown.map(|plan| RulePlan::Project {
                    input: Box::new(plan),
                    columns: columns.clone(),
                }),
            )
        }
        _ => (plan.clone(), None),
    }
}
