// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Storage-independent obligations bound to explicit relation identities and native plans.
mod nested_values;
mod ordinals;
mod quantities;
mod references;
mod source_spans;
use crate::native::{
    common::{Column, DataFusionError, Result},
    functions_aggregate::expr_fn::count,
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, lit},
};
use pse_schema::{Registry, model::RelationSpec};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit selected relations. Missing and present-empty inputs are distinct.
pub type RelationInputs = BTreeMap<pse_ids::SemanticId, LogicalPlan>;

/// The independently established part of a bound relation obligation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObligationKind {
    /// Per-row declared value domains, including nested fields.
    LocalValues,
    /// Keys, references, source spans, quantities semantics.
    Relational,
}
/// Native query with its declaration and evidence category intact.
#[derive(Debug)]
pub struct BoundObligation {
    /// Actual declared relation identity.
    pub relation: pse_ids::SemanticId,
    /// Which evidence this query establishes.
    pub kind: ObligationKind,
    /// Rows violating the obligation.
    pub plan: LogicalPlan,
}

/// Pure native obligation templates. Catalog owns revision selection and publication.
#[derive(Debug)]
pub struct ObligationTemplates<'a> {
    registry: &'a Registry,
}
impl<'a> ObligationTemplates<'a> {
    /// Bind declarations from one immutable owner.
    pub const fn new(registry: &'a Registry) -> Self {
        Self { registry }
    }
    /// Check exact artifact membership independently of whether relations contain rows.
    /// # Errors
    /// Unknown profile or absent required relation identities.
    pub fn require_profile(
        &self,
        name: &str,
        present: &BTreeSet<pse_ids::SemanticId>,
    ) -> Result<()> {
        let required = self
            .registry
            .artifact_profile(name)
            .ok_or_else(|| invalid("unknown artifact completeness profile"))?;
        if !required.is_subset(present) {
            let missing = required
                .difference(present)
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            return Err(invalid(&format!(
                "incomplete {name} artifact: {}",
                missing.join(", ")
            )));
        }
        Ok(())
    }
    /// Instantiate local, key, reference, quantity, source queries.
    /// # Errors
    /// Unregistered inputs, incompatible fields or native expression construction failure.
    pub fn bind(
        &self,
        inputs: &RelationInputs,
        state: &dyn super::planner::ValidationPlanner,
    ) -> Result<Vec<LogicalPlan>> {
        Ok(self
            .bind_classified(inputs, state)?
            .into_iter()
            .map(|check| check.plan)
            .collect())
    }
    /// Bind all obligations with explicit categories. Consumers may omit a local
    /// query only when their own exact value owner already establishes it.
    /// # Errors
    /// Same declaration and construction errors as [`Self::bind`].
    pub fn bind_classified(
        &self,
        inputs: &RelationInputs,
        state: &dyn super::planner::ValidationPlanner,
    ) -> Result<Vec<BoundObligation>> {
        self.bind_required(inputs, state, |_, _| true)
    }

    /// Construct only requested categories; omission does not mint validation evidence.
    /// Callers must establish coverage from their actual checked or completed owners.
    /// # Errors
    /// Unknown declarations, incompatible schemas or expression construction failure.
    pub fn bind_required(
        &self,
        inputs: &RelationInputs,
        state: &dyn super::planner::ValidationPlanner,
        mut required: impl FnMut(pse_ids::SemanticId, ObligationKind) -> bool,
    ) -> Result<Vec<BoundObligation>> {
        let mut all = Vec::new();
        for (id, input) in inputs {
            let mut checks = Vec::new();
            let spec = self
                .registry
                .relation_by_id(*id)
                .ok_or_else(|| invalid("unknown obligation input identity"))?;
            super::validate_schema(self.registry, spec, input.schema().as_arrow()).map_err(
                |errors| pse_columnar::external(crate::RelationError::Validation { errors }),
            )?;
            if required(*id, ObligationKind::LocalValues) {
                all.push(BoundObligation {
                    relation: *id,
                    kind: ObligationKind::LocalValues,
                    plan: local_values(self.registry, spec, input.clone(), state)?,
                });
            }
            if !required(*id, ObligationKind::Relational) {
                continue;
            }
            let documents = self
                .registry
                .relation("authored.documents")
                .and_then(|spec| inputs.get(&spec.id));
            checks.extend(source_spans::plans(input, documents)?);
            checks.extend(quantities::plans(input, inputs, self.registry)?);
            let keys = self
                .registry
                .obligations(spec.key)
                .map_err(pse_columnar::external)?
                .primary_key
                .iter()
                .map(|name| column(name))
                .collect::<Vec<_>>();
            let duplicates = LogicalPlanBuilder::from(input.clone())
                .aggregate(keys, vec![count(lit(1_i64)).alias("pse_key_count")])?
                .filter(column("pse_key_count").gt(lit(1_i64)))?
                .build()?;
            checks.push(violation(duplicates, spec, "duplicate primary key")?);
            checks.push(key_consistency(input.clone(), spec)?);
            checks.extend(references::plans(input, spec, inputs, self.registry)?);
            checks.extend(ordinals::plans(input, spec, inputs, self.registry)?);
            all.extend(checks.into_iter().map(|plan| BoundObligation {
                relation: *id,
                kind: ObligationKind::Relational,
                plan,
            }));
        }
        Ok(all)
    }
}
fn key_consistency(input: LogicalPlan, spec: &RelationSpec) -> Result<LogicalPlan> {
    let token = crate::identity::key(
        spec.id,
        spec.primary_key
            .iter()
            .map(|name| (*name, column(name)))
            .collect(),
    );
    key_collisions(input, spec, token)
}

fn key_collisions(input: LogicalPlan, spec: &RelationSpec, token: Expr) -> Result<LogicalPlan> {
    // Compare actual distinct tuples. A token collision cannot merge different keys
    // into a published identity, even though ordinary key uniqueness holds.
    let mut projection = spec
        .primary_key
        .iter()
        .map(|name| column(name))
        .collect::<Vec<_>>();
    projection.push(token.alias("__pse_row_token"));
    let collisions = LogicalPlanBuilder::from(input)
        .project(projection)?
        .distinct()?
        .aggregate(
            vec![column("__pse_row_token")],
            vec![count(lit(1_i64)).alias("__pse_distinct_keys")],
        )?
        .filter(column("__pse_distinct_keys").gt(lit(1_i64)))?
        .build()?;
    violation(collisions, spec, "distinct primary keys share a row token")
}
fn local_values(
    registry: &Registry,
    spec: &RelationSpec,
    input: LogicalPlan,
    state: &dyn super::planner::ValidationPlanner,
) -> Result<LogicalPlan> {
    let context = super::ValidationContext::new(registry, state.snapshot());
    let prepared = context
        .relation(registry, spec)
        .map_err(pse_columnar::external)?;
    let predicate = prepared.predicate(
        input
            .schema()
            .columns()
            .into_iter()
            .map(Expr::Column)
            .collect(),
    );
    violation(
        LogicalPlanBuilder::from(input)
            .filter(predicate.is_not_true())?
            .build()?,
        spec,
        "local values",
    )
}

pub(crate) fn violation(
    input: LogicalPlan,
    spec: &RelationSpec,
    reason: &str,
) -> Result<LogicalPlan> {
    let mut columns = input
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    columns.push(lit(format!("{}: {reason}", spec.qualified_name())).alias("__pse_violation"));
    columns.push(
        lit(pse_diagnostics::DiagnosticCode::ValidationInvariant.as_str()).alias("__pse_code"),
    );
    LogicalPlanBuilder::from(input).project(columns)?.build()
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}

#[cfg(test)]
mod consolidation_unit;

fn selected(name: &str, inputs: &RelationInputs, registry: &Registry) -> Option<LogicalPlan> {
    registry
        .relation(name)
        .and_then(|spec| inputs.get(&spec.id))
        .cloned()
}
