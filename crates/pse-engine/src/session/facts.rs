// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite algorithm inputs own Arrow facts and exact source selections, never a store graph.
use super::{EngineSession, engine};
use crate::EngineError;
use datafusion::common::{ResolvedTableReference, TableReference};
use pse_columnar::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use std::sync::Arc;

/// Complete immutable fields for a finite algorithm. This establishes local field
/// validity, not foreign keys, global invariants or arbitrary optimizer constraints.
/// A durable selection is minted only by executing the actual published provider.
#[derive(Clone, Debug)]
pub struct RelationFacts {
    pub(super) checked: FieldCheckedBatch,
    pub(super) witness: Option<crate::provider::witness::SourceWitness>,
}
impl RelationFacts {
    /// Retain generated or already checked temporary facts without allocating or
    /// inventing a durable identity. Consumers check the actual declaration and
    /// retain buffers under their shared budget at the native binding boundary.
    pub const fn from_checked(checked: FieldCheckedBatch) -> Self {
        Self {
            checked,
            witness: None,
        }
    }
    /// Actual owned fields; clones retain buffers but no producing computation.
    pub const fn checked(&self) -> &FieldCheckedBatch {
        &self.checked
    }
    /// Exact durable selection, if these fields were captured from a publication.
    pub const fn witness(&self) -> Option<&crate::provider::witness::SourceWitness> {
        self.witness.as_ref()
    }
}

impl EngineSession {
    /// Exact durable member selected in this invocation's native namespace.
    /// # Errors
    /// The fully qualified name has no selected Delta relation binding.
    pub fn source_witness(
        &self,
        reference: &ResolvedTableReference,
    ) -> Result<crate::provider::witness::SourceWitness, EngineError> {
        let reference = table_reference(reference);
        self.bindings
            .iter()
            .find_map(|(_, binding)| {
                (binding.reference == reference)
                    .then(|| binding.witness.clone())
                    .flatten()
            })
            .ok_or_else(|| invalid("no selected Delta member has this native name"))
    }

    /// Begin a declared relation read through common native policy and execution.
    /// # Errors
    /// The qualified name is unbound or native admission/planning fails.
    pub async fn relation_stream(
        &self,
        reference: &ResolvedTableReference,
        cancel: &CancellationToken,
    ) -> Result<super::OwnedComputationStream, EngineError> {
        let reference = table_reference(reference);
        let binding = self
            .bindings
            .iter()
            .find_map(|(_, binding)| (binding.reference == reference).then_some(binding))
            .ok_or_else(|| invalid("declared native relation is absent"))?;
        if binding.relation.is_none() {
            return Err(invalid("native binding has no relation declaration"));
        }
        let plan = datafusion::logical_expr::LogicalPlanBuilder::scan(
            reference,
            datafusion::datasource::provider_as_source(Arc::clone(&binding.provider)),
            None,
        )
        .and_then(datafusion::logical_expr::LogicalPlanBuilder::build)
        .map_err(engine)?;
        self.prepare_rule_plan(plan, cancel)?
            .execute_stream(cancel)
            .await
    }

    /// Materialize one declared relation through the same native admission/policy
    /// as SQL. The result owns Arrow buffers and the optional exact Delta selection;
    /// no publication, execution session or producing plan survives in this owner.
    /// # Errors
    /// Unknown declaration/name, policy refusal, source error, cancellation or resources.
    pub async fn capture_relation(
        &self,
        reference: &ResolvedTableReference,
        cancel: &CancellationToken,
    ) -> Result<RelationFacts, EngineError> {
        cancel.checkpoint()?;
        let reference = table_reference(reference);
        let binding = self
            .bindings
            .iter()
            .find_map(|(_, binding)| (binding.reference == reference).then_some(binding))
            .ok_or_else(|| invalid("declared native relation is absent"))?;
        let spec = binding
            .relation
            .and_then(|key| self.registry.relation_by_key(key))
            .ok_or_else(|| invalid("native binding has no relation declaration"))?;
        let plan = datafusion::logical_expr::LogicalPlanBuilder::scan(
            reference,
            datafusion::datasource::provider_as_source(Arc::clone(&binding.provider)),
            None,
        )
        .and_then(datafusion::logical_expr::LogicalPlanBuilder::build)
        .map_err(engine)?;
        let completed = self
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        let checked = completed.into_checked_relation(&self.registry, spec, cancel)?;
        Ok(RelationFacts {
            checked,
            witness: binding.witness.clone(),
        })
    }
}

/// Inspect actual native view providers, retaining their source dependencies.
/// # Errors
/// A native source cannot expose its actual underlying provider.
pub fn view_dependencies(
    provider: &Arc<dyn datafusion::catalog::TableProvider>,
) -> datafusion::common::Result<Vec<Arc<dyn datafusion::catalog::TableProvider>>> {
    use datafusion::common::tree_node::TreeNodeRecursion;
    let mut inputs = Vec::new();
    if let Some(plan) = provider.get_logical_plan() {
        plan.apply_with_subqueries(|plan| {
            if let datafusion::logical_expr::LogicalPlan::TableScan(scan) = plan {
                let source = datafusion::datasource::source_as_provider(&scan.source)?;
                if !inputs.iter().any(|input| Arc::ptr_eq(input, &source)) {
                    inputs.extend(view_dependencies(&source)?);
                    inputs.push(source);
                }
            }
            Ok(TreeNodeRecursion::Continue)
        })?;
    }
    Ok(inputs)
}
pub(super) fn table_reference(reference: &ResolvedTableReference) -> TableReference {
    TableReference::full(
        reference.catalog.clone(),
        reference.schema.clone(),
        reference.table.clone(),
    )
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "relation.facts".into(),
        reason: reason.into(),
    }
}
