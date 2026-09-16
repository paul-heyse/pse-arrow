// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite algorithm inputs own Arrow facts and exact source selections, never a store graph.
use super::{SnapshotSession, engine};
use crate::{CatalogError, delta::publication::Publication};
use datafusion::common::{ResolvedTableReference, TableReference};
use pse_ids::CancellationToken;
use pse_relations::{columnar::FieldCheckedBatch, generated::runtime::publications};
use std::{collections::BTreeMap, sync::Arc};

/// Complete immutable fields for a finite algorithm. This establishes local field
/// validity, not foreign keys, global invariants or arbitrary optimizer constraints.
/// A durable selection is minted only by executing the actual published provider.
#[derive(Clone, Debug)]
pub struct RelationFacts {
    checked: FieldCheckedBatch,
    selection: Option<publications::RuntimePublicationsFieldMembersItem>,
}
impl RelationFacts {
    /// Retain generated or already checked temporary facts without allocating or
    /// inventing a durable identity. Consumers check the actual declaration and
    /// retain buffers under their shared budget at the native binding boundary.
    pub const fn from_checked(checked: FieldCheckedBatch) -> Self {
        Self {
            checked,
            selection: None,
        }
    }
    /// Actual owned fields; clones retain buffers but no producing computation.
    pub const fn checked(&self) -> &FieldCheckedBatch {
        &self.checked
    }
    /// Exact durable selection, if these fields were captured from a publication.
    pub const fn selection(&self) -> Option<&publications::RuntimePublicationsFieldMembersItem> {
        self.selection.as_ref()
    }
}

impl SnapshotSession {
    /// Exact durable member selected in this invocation's native namespace.
    /// # Errors
    /// The fully qualified name has no selected Delta relation binding.
    pub fn selected_member(
        &self,
        reference: &ResolvedTableReference,
    ) -> Result<&publications::RuntimePublicationsFieldMembersItem, CatalogError> {
        let reference = table_reference(reference);
        self.bindings
            .iter()
            .find_map(|(_, binding)| {
                (binding.reference == reference)
                    .then_some(binding.selection.as_ref())
                    .flatten()
            })
            .ok_or_else(|| invalid("no selected Delta member has this native name"))
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
    ) -> Result<RelationFacts, CatalogError> {
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
            selection: binding.selection.clone(),
        })
    }
}

pub(super) async fn bind_publication(
    publication: &Publication,
    registry: Arc<pse_schema::Registry>,
    factory: &super::SessionFactory,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, CatalogError> {
    use crate::provider::binding::{BindingKey, TableBinding};
    let mut session = factory.candidate(BTreeMap::new(), registry, cancel)?;
    let mut unique = BTreeMap::new();
    for member in &publication.record().members {
        cancel.checkpoint()?;
        let reference = ResolvedTableReference {
            catalog: member.catalog_name.clone().into(),
            schema: member.schema_name.clone().into(),
            table: member.table_name.clone().into(),
        };
        let provider = publication
            .member_provider(&reference)
            .await
            .map_err(engine)?;
        let spec = session
            .registry
            .relation_by_id(member.relation_id)
            .filter(|spec| {
                i64::from(spec.key.version) == member.relation_version
                    && spec.fingerprint == member.contract_fingerprint
            })
            .ok_or_else(|| invalid("selected member declaration differs"))?;
        let schema = pse_schema::arrow::relation_schema(&session.registry, spec)
            .map_err(pse_relations::RelationError::from)?;
        if provider.schema().fields() != schema.fields() {
            return Err(invalid(
                "selected Delta fields differ from their declaration",
            ));
        }
        let reference = table_reference(&reference);
        let mut binding = TableBinding::new(reference.clone(), provider, Some(spec.key), None);
        binding.dependencies = view_dependencies(&binding.provider).map_err(engine)?;
        binding.selection = Some(member.clone());
        // Repeated declarations in separate roles remain fully qualified. An
        // unambiguous declaration also supports generated relation-key consumers.
        unique
            .entry(spec.key)
            .and_modify(|value| *value = None)
            .or_insert_with(|| Some(binding.clone()));
        session
            .bindings
            .insert(BindingKey::Native(reference), binding)
            .map_err(engine)?;
    }
    for (key, binding) in unique
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key, value)))
    {
        session
            .bindings
            .insert(BindingKey::Relation(key), binding)
            .map_err(engine)?;
    }
    Ok(session)
}

fn view_dependencies(
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
fn table_reference(reference: &ResolvedTableReference) -> TableReference {
    TableReference::full(
        reference.catalog.clone(),
        reference.schema.clone(),
        reference.table.clone(),
    )
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "relation.facts".into(),
        reason: reason.into(),
    }
}
