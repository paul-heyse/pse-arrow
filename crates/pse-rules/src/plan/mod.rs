// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL binding and finite-domain admission. DataFusion owns the algebra.
pub(crate) mod delta;
mod head;
pub(crate) mod trace;
mod validation;

use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::arrow::datatypes::SchemaRef;
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::{
    Registry,
    model::{RelationKey, RuleQuery, RuleSpec},
};
use std::collections::BTreeMap;

/// Explicit relation selected for each domain input scope.
#[derive(Clone, Debug, Default)]
pub struct PortBinding {
    /// Exact schema versions; actual values are owned by the bound session.
    pub ports: BTreeMap<String, RelationKey>,
}

/// Bound native query with its admitted destination and domain identity.
#[derive(Clone, Debug)]
pub struct CompiledRule {
    /// Exact registry rule identity.
    pub rule_id: SemanticId,
    /// Versioned declaration name.
    pub name: String,
    /// Native assertion query.
    pub plan: LogicalPlan,
    /// Unknown candidates, when requested by the single-rule convenience executor.
    pub undecided: Option<LogicalPlan>,
    /// Exact output declaration.
    pub head_schema: SchemaRef,
    /// Deterministic key order.
    pub key_columns: Vec<String>,
    /// Qualified destination relation.
    pub head: String,
}

/// Bind the true and optional unknown queries without executing data operations.
/// # Errors
/// Refuses inconsistent input scopes, incompatible outputs or native planning errors.
pub async fn compile(
    rule: &RuleSpec,
    binding: &PortBinding,
    session: &SnapshotSession,
    registry: &Registry,
) -> Result<CompiledRule, RuleError> {
    let cancel = CancellationToken::new();
    let query = rule
        .queries
        .iter()
        .find(|query| query.truth == "true")
        .ok_or_else(|| internal("rule has no true assertion query"))?;
    let mut compiled = compile_query(rule, query, binding, session, registry, &cancel).await?;
    if let Some(query) = rule.queries.iter().find(|query| query.truth == "unknown") {
        compiled.undecided = Some(
            compile_query(rule, query, binding, session, registry, &cancel)
                .await?
                .plan,
        );
    }
    Ok(compiled)
}

pub(crate) async fn compile_query(
    rule: &RuleSpec,
    query: &RuleQuery,
    binding: &PortBinding,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<CompiledRule, RuleError> {
    let mut expected = BTreeMap::new();
    for input in &rule.inputs {
        let spec = registry
            .relation(&input.relation)
            .ok_or_else(|| internal("undeclared native input"))?;
        expected.insert(input.port.to_owned(), spec.key);
    }
    if binding.ports != expected {
        return Err(internal(
            "native query input bindings differ from its declaration",
        ));
    }
    let inputs = expected
        .values()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let plan = session
        .bind_declared_query(&query.sql, &inputs, cancel)
        .await?;
    validation::validate(&plan, rule, registry)?;
    let target = registry
        .relation(&rule.head)
        .ok_or_else(|| internal("native head is undeclared"))?;
    let head_schema = std::sync::Arc::new(
        pse_schema::arrow::relation_schema(registry, target)
            .map_err(|error| internal(error.to_string()))?,
    );
    let plan = head::prepare(plan, target, session, registry, false)?;
    let plan = LogicalPlanBuilder::from(plan)
        .sort(
            target
                .primary_key
                .iter()
                .map(|key| datafusion_expr::col(*key).sort(true, true)),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    Ok(CompiledRule {
        rule_id: rule.id,
        name: rule.qualified_name(),
        plan,
        undecided: None,
        head_schema,
        key_columns: target
            .primary_key
            .iter()
            .map(|key| (*key).to_owned())
            .collect(),
        head: rule.head.clone(),
    })
}
