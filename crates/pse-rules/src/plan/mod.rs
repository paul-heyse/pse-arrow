// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Rule algebra lowered through DataFusion while retaining semantic typing derivations.
mod aggregate;
pub(crate) mod closure;
pub(crate) mod delta;
mod expr;
mod head;
mod identity;
mod lower;
pub(crate) mod outcomes;
mod recursive;
pub(crate) mod trace;

use crate::RuleError;
use crate::errmap::{engine, internal};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::SnapshotSession;
use pse_ids::SemanticId;
use pse_schema::Registry;
use pse_schema::model::{Cell, FieldContract, RelationKey, RuleHead, RulePlan, RuleSpec};
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
    /// Exact destination schema declared in the native projection before preparation.
    pub head_schema: SchemaRef,
    /// Declared deterministic output ordering.
    pub key_columns: Vec<String>,
    /// Head declaration retained for invariant admission.
    pub head: RuleHead,
    pub(crate) checks: Vec<(LogicalPlan, String)>,
}

#[derive(Clone, Debug)]
pub(crate) struct Column {
    name: std::borrow::Cow<'static, str>,
    pub(crate) spec: FieldContract,
    qualifier: Option<String>,
    /// Native address after join scoping; logical rule names remain unchanged.
    physical: Option<datafusion_common::Column>,
    literal: Option<Cell>,
}
#[derive(Clone)]
struct Planned {
    plan: LogicalPlan,
    columns: Vec<Column>,
    hidden: Vec<String>,
}

#[derive(Clone, Default)]
pub(crate) struct NativeBindings {
    results: Vec<(RulePlan, Planned)>,
    traces: Vec<(RulePlan, Vec<trace::Trace>)>,
}

impl NativeBindings {
    pub(crate) fn has_results(&self) -> bool {
        !self.results.is_empty()
    }
}

struct Compiler<'a> {
    rule: &'a RuleSpec,
    binding: &'a PortBinding,
    session: &'a SnapshotSession,
    registry: &'a Registry,
    binders: Vec<(String, Planned)>,
    trace_binders: Vec<(String, Vec<trace::Trace>)>,
    native: &'a NativeBindings,
    checks: Vec<(LogicalPlan, String)>,
    recursive_counter: usize,
    binding_counter: usize,
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
    compile_bound(rule, binding, session, registry, &NativeBindings::default())
}

pub(crate) fn compile_bound(
    rule: &RuleSpec,
    binding: &PortBinding,
    session: &SnapshotSession,
    registry: &Registry,
    native: &NativeBindings,
) -> Result<CompiledRule, RuleError> {
    let mut compiler = Compiler {
        rule,
        binding,
        session,
        registry,
        binders: vec![],
        trace_binders: vec![],
        native,
        checks: vec![],
        recursive_counter: 0,
        binding_counter: 0,
    };
    let mut candidates = outcomes::candidates(&rule.plan).into_iter();
    let (_, decided) = candidates
        .next()
        .ok_or_else(|| internal("rule lacks a decided truth query"))?;
    let unknown = candidates.find_map(|(truth, plan)| (truth == "unknown").then_some(plan));
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
    let (plan, _, head_schema) = head::prepare(rule, output, columns, registry, false, session)?;
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
            let schema = datafusion::arrow::datatypes::Schema::new(
                keys.iter()
                    .map(|key| {
                        head_schema
                            .field_with_name(key)
                            .cloned()
                            .map_err(|error| internal(error.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            sort(
                head::declare_schema(plan, &std::sync::Arc::new(schema), session)?,
                &keys,
            )
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
