// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned native input plans for finite algorithms. Exact source identity belongs
//! to the selected scan and is discarded if a caller replaces that computation.
use super::{RelationFacts, SnapshotSession, engine, execution::NativeExecutionContext};
use crate::CatalogError;
use datafusion::{
    common::ResolvedTableReference,
    execution::TaskContext,
    logical_expr::{LogicalPlan, LogicalPlanBuilder},
    physical_plan::{ExecutionPlan, execute_stream},
};
use futures_util::TryStreamExt;
use pse_ids::{SemanticId, owned_buffer::OwnedRecordBatch};
use pse_relations::{columnar::FieldCheckedBatch, generated::runtime::publications};
use pse_schema::Registry;
use std::sync::Arc;

/// A declared computation with an optional exact selected source witness.
#[derive(Clone, Debug)]
pub struct RelationPlan {
    relation: SemanticId,
    plan: LogicalPlan,
    selection: Option<publications::RuntimePublicationsFieldMembersItem>,
}
impl RelationPlan {
    /// Declare a derived native relation; derived values claim no durable identity.
    /// # Errors
    /// Undeclared relation or incompatible native output fields.
    pub fn derived(
        relation: SemanticId,
        plan: LogicalPlan,
        registry: &Registry,
    ) -> Result<Self, CatalogError> {
        let spec = registry
            .relation_by_id(relation)
            .ok_or_else(|| invalid("input declaration absent"))?;
        let plan = super::output::declare_relation_output(plan, registry, spec).map_err(engine)?;
        Ok(Self {
            relation,
            plan,
            selection: None,
        })
    }
    /// Actual logical child.
    pub const fn plan(&self) -> &LogicalPlan {
        &self.plan
    }
    /// Exact registered relation.
    pub const fn relation_id(&self) -> SemanticId {
        self.relation
    }
    /// Replace a native child. Only an unchanged selected scan keeps its witness.
    #[must_use]
    pub fn rewritten(&self, plan: LogicalPlan) -> Self {
        Self {
            selection: (plan == self.plan)
                .then(|| self.selection.clone())
                .flatten(),
            relation: self.relation,
            plan,
        }
    }
    /// Capture the actual planned physical child under the invocation's ownership.
    /// This is the finite algorithm boundary, not an additional scan of the source.
    /// # Errors
    /// Changed child fields, native execution failure, cancellation or budget refusal.
    pub async fn capture(
        &self,
        child: Arc<dyn ExecutionPlan>,
        context: Arc<TaskContext>,
        services: &NativeExecutionContext,
    ) -> Result<RelationFacts, CatalogError> {
        let registry = services.registry();
        let spec = registry
            .relation_by_id(self.relation)
            .ok_or_else(|| invalid("input declaration absent"))?;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(registry, spec)
                .map_err(pse_relations::RelationError::from)?,
        );
        if child.schema().fields() != schema.fields() {
            return Err(invalid(
                "executed child fields differ from its declared relation",
            ));
        }
        let cancel = services.cancellation();
        let mut stream = execute_stream(child, context).map_err(engine)?;
        let mut batches = Vec::new();
        let mut scratch = services.reserver().open("native:finite-child-admission");
        while let Some(batch) = stream.try_next().await.map_err(engine)? {
            cancel.checkpoint()?;
            scratch.try_grow(pse_ids::validation_extent(&batch)?)?;
            let batch = batch
                .with_schema(Arc::clone(&schema))
                .map_err(pse_relations::RelationError::from)?;
            let owned = OwnedRecordBatch::export(batch, services.reserver().as_ref(), cancel)?;
            batches.push(FieldCheckedBatch::admit_owned(registry, spec, owned)?);
        }
        let checked = FieldCheckedBatch::concat_reserved(
            registry,
            spec,
            &batches,
            services.reserver().as_ref(),
            cancel,
        )?;
        Ok(RelationFacts {
            checked,
            selection: self.selection.clone(),
        })
    }
}
impl SnapshotSession {
    /// Bind the actual native selected source as a finite algorithm child.
    /// # Errors
    /// Unknown or undeclared source, or invalid native scan.
    pub fn relation_plan(
        &self,
        reference: &ResolvedTableReference,
    ) -> Result<RelationPlan, CatalogError> {
        let reference = super::facts::table_reference(reference);
        let binding = self
            .bindings
            .iter()
            .find_map(|(_, binding)| (binding.reference == reference).then_some(binding))
            .ok_or_else(|| invalid("input source absent"))?;
        let spec = binding
            .relation
            .and_then(|key| self.registry.relation_by_key(key))
            .ok_or_else(|| invalid("input source undeclared"))?;
        let plan = LogicalPlanBuilder::scan(
            reference,
            datafusion::datasource::provider_as_source(Arc::clone(&binding.provider)),
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
        Ok(RelationPlan {
            relation: spec.id,
            plan,
            selection: binding.selection.clone(),
        })
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "native.input".into(),
        reason: reason.into(),
    }
}

impl SnapshotSession {
    /// Bind declared native plans as a private relational query scope. DataFusion's
    /// native view expansion keeps their real children visible; no rows are collected
    /// and no key or durable identity is inferred from their output schema.
    /// # Errors
    /// Foreign input providers, incompatible declarations, or native field admission.
    pub fn plan_workspace(
        &self,
        inputs: std::collections::BTreeMap<pse_schema::model::RelationKey, RelationPlan>,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<Self, CatalogError> {
        use crate::provider::binding::{BindingKey, TableBinding};
        use datafusion::{catalog::TableProvider, common::TableReference, datasource::ViewTable};
        let mut result = self.clone();
        result.bindings = self.bindings.resolutions_only();
        // All arguments share this exact immutable session. One native admission
        // traversal retains shared producer/field owners across sibling inputs.
        let plans = self.derive_plan_fields_many(
            &inputs
                .values()
                .map(|input| input.plan.clone())
                .collect::<Vec<_>>(),
            cancel,
        )?;
        for ((key, input), plan) in inputs.into_iter().zip(plans) {
            cancel.checkpoint()?;
            let spec = self
                .registry
                .relation_by_key(key)
                .filter(|spec| spec.id == input.relation)
                .ok_or_else(|| invalid("planned relation differs from its argument declaration"))?;
            let sources = self.source_bindings(&input.plan)?;
            let selected_dependencies = Self::selected_binding_dependencies(&sources)?;
            let plan = super::output::declare_relation_output(plan, &self.registry, spec)
                .map_err(engine)?;
            let provider: Arc<dyn TableProvider> = Arc::new(ViewTable::new(plan, None));
            let mut binding = TableBinding::new(
                TableReference::full("plans", key.namespace.as_str(), key.name),
                provider,
                Some(key),
                None,
            );
            binding.dependencies = sources
                .iter()
                .flat_map(|source| {
                    std::iter::once(Arc::clone(&source.provider)).chain(source.dependencies.clone())
                })
                .collect();
            binding.effects = sources
                .iter()
                .flat_map(|source| source.effects.clone())
                .collect();
            binding.selected_dependencies = selected_dependencies;
            binding.selection = input.selection;
            result
                .bindings
                .insert(BindingKey::Relation(key), binding)
                .map_err(engine)?;
        }
        Ok(result)
    }

    /// Exact selected sources actually consumed by a native plan, including inputs
    /// underneath derived views. Derived output values do not acquire those identities.
    /// Empty and unmatched inputs are included: they can affect anti joins and absence.
    /// # Errors
    /// A scan is outside the admitted native provider generation, or two sources use
    /// the same qualified role for different durable selections.
    pub fn selected_dependencies(
        &self,
        plan: &LogicalPlan,
    ) -> Result<Vec<publications::RuntimePublicationsFieldMembersItem>, CatalogError> {
        Self::selected_binding_dependencies(&self.source_bindings(plan)?)
    }

    fn selected_binding_dependencies(
        bindings: &[Arc<crate::provider::binding::TableBinding>],
    ) -> Result<Vec<publications::RuntimePublicationsFieldMembersItem>, CatalogError> {
        let mut selected = std::collections::BTreeMap::new();
        for binding in bindings {
            for member in binding
                .selection
                .iter()
                .chain(&binding.selected_dependencies)
            {
                let name = (
                    member.catalog_name.clone(),
                    member.schema_name.clone(),
                    member.table_name.clone(),
                );
                if selected
                    .insert(name, member.clone())
                    .is_some_and(|previous| previous != *member)
                {
                    return Err(invalid("ambiguous exact source dependency"));
                }
            }
        }
        Ok(selected.into_values().collect())
    }

    pub(crate) fn source_bindings(
        &self,
        plan: &LogicalPlan,
    ) -> Result<Vec<Arc<crate::provider::binding::TableBinding>>, CatalogError> {
        let mut sources = Vec::new();
        let mut visited = SourceDiscovery {
            nodes: std::collections::HashSet::new(),
            reservation: self.reserver.open("session:source-discovery"),
            scope_sequence: 0,
        };
        self.source_bindings_scoped(plan, &mut Vec::new(), &mut sources, 0, &mut visited)
            .map_err(engine)?;
        Ok(sources)
    }

    fn source_bindings_scoped(
        &self,
        plan: &LogicalPlan,
        scope: &mut Vec<(String, datafusion::arrow::datatypes::SchemaRef)>,
        sources: &mut Vec<Arc<crate::provider::binding::TableBinding>>,
        scope_id: usize,
        visited: &mut SourceDiscovery,
    ) -> datafusion::common::Result<()> {
        use datafusion::common::tree_node::TreeNodeRecursion;
        plan.apply_with_subqueries(|node| {
            // Shared immutable producer graphs are DAGs, not expanded trees.
            // Scope identity keeps internal recursive scans context-dependent;
            // ephemeral Subquery wrappers must never supply address keys.
            if !matches!(node, LogicalPlan::Subquery(_)) {
                let key = (super::admission::identity(node), scope_id);
                if visited.nodes.contains(&key) {
                    return Ok(TreeNodeRecursion::Jump);
                }
                visited.reservation.try_grow(64).map_err(|error| {
                    datafusion::common::DataFusionError::External(Box::new(CatalogError::from(
                        error,
                    )))
                })?;
                visited.nodes.insert(key);
            }
            if let LogicalPlan::RecursiveQuery(query) = node {
                self.source_bindings_scoped(&query.static_term, scope, sources, scope_id, visited)?;
                scope.push((
                    query.name.clone(),
                    Arc::clone(query.static_term.schema().inner()),
                ));
                visited.scope_sequence =
                    visited.scope_sequence.checked_add(1).ok_or_else(|| {
                        datafusion::common::DataFusionError::Internal(
                            "source scope counter overflow".into(),
                        )
                    })?;
                let result = self.source_bindings_scoped(
                    &query.recursive_term,
                    scope,
                    sources,
                    visited.scope_sequence,
                    visited,
                );
                scope.pop();
                result?;
                return Ok(TreeNodeRecursion::Jump);
            }
            if let LogicalPlan::TableScan(scan) = node {
                let provider = datafusion::datasource::source_as_provider(&scan.source)?;
                if provider
                    .downcast_ref::<datafusion_catalog::cte_worktable::CteWorkTable>()
                    .is_some()
                {
                    super::admission::admit_scan(scan, &self.registry, &[], scope)?;
                    return Ok(TreeNodeRecursion::Jump);
                }
                let exact = self.bindings.iter().find_map(|(_, binding)| {
                    Arc::ptr_eq(&binding.provider, &provider).then_some(binding)
                });
                let matching = match exact {
                    Some(binding) => vec![binding],
                    None => self
                        .bindings
                        .iter()
                        .filter_map(|(_, binding)| {
                            binding
                                .dependencies
                                .iter()
                                .any(|input| Arc::ptr_eq(input, &provider))
                                .then_some(binding)
                        })
                        .collect(),
                };
                if matching.is_empty() {
                    return Err(datafusion::common::DataFusionError::Plan(format!(
                        "dependency source {} is outside the native generation",
                        scan.table_name.to_quoted_string()
                    )));
                }
                for source in matching {
                    if !sources.iter().any(
                        |existing: &Arc<crate::provider::binding::TableBinding>| {
                            Arc::ptr_eq(existing, &source)
                        },
                    ) {
                        sources.push(source);
                    }
                }
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .map(|_| ())
    }
}

struct SourceDiscovery {
    nodes: std::collections::HashSet<(super::admission::NodeIdentity, usize)>,
    reservation: Box<dyn pse_ids::Reservation>,
    scope_sequence: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::MemoryReserver;

    #[test]
    fn source_discovery_visits_shared_nodes_once_per_scope() {
        let context = datafusion::prelude::SessionContext::new();
        let budget = pse_ids::FixedBudget::new(1 << 20);
        let factory = super::super::SessionFactory::from_builder(
            context.runtime_env(),
            budget.clone(),
            "source-discovery-unit",
            datafusion::execution::session_state::SessionStateBuilder::from(context.state()),
        );
        let session = factory
            .candidate_checked(
                std::collections::BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &pse_ids::CancellationToken::new(),
            )
            .unwrap();
        let mut plan = LogicalPlanBuilder::empty(false).build().unwrap();
        for _ in 0..20 {
            let shared = Arc::new(plan);
            plan = LogicalPlan::Union(datafusion::logical_expr::Union {
                inputs: vec![Arc::clone(&shared), Arc::clone(&shared)],
                schema: Arc::clone(shared.schema()),
            });
        }
        let before = budget.reserved();
        {
            let mut discovery = SourceDiscovery {
                nodes: std::collections::HashSet::new(),
                reservation: budget.open("test:source-discovery"),
                scope_sequence: 0,
            };
            let mut sources = Vec::new();
            session
                .source_bindings_scoped(&plan, &mut Vec::new(), &mut sources, 0, &mut discovery)
                .unwrap();
            assert_eq!(discovery.nodes.len(), 21);
            assert!(sources.is_empty());
            session
                .source_bindings_scoped(&plan, &mut Vec::new(), &mut sources, 1, &mut discovery)
                .unwrap();
            assert_eq!(
                discovery.nodes.len(),
                42,
                "distinct recursive scopes must be checked separately"
            );
        }
        assert_eq!(budget.reserved(), before);
    }
}
