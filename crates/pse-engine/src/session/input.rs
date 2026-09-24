// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned native input plans for finite algorithms. Exact source identity belongs
//! to the selected scan and is discarded if a caller replaces that computation.
use super::{EngineSession, RelationFacts, engine, execution::NativeExecutionContext};
use crate::EngineError;
use datafusion::{
    common::ResolvedTableReference,
    execution::TaskContext,
    logical_expr::{LogicalPlan, LogicalPlanBuilder},
    physical_plan::{ExecutionPlan, execute_stream},
};
use futures_util::TryStreamExt;
use pse_ids::SemanticId;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::Registry;
use std::sync::Arc;

/// A declared computation with an optional exact selected source witness.
#[derive(Clone, Debug)]
pub struct RelationPlan {
    relation: SemanticId,
    plan: LogicalPlan,
    witness: Option<crate::provider::witness::SourceWitness>,
}
impl RelationPlan {
    /// Declare a derived native relation; derived values claim no durable identity.
    /// # Errors
    /// Undeclared relation or incompatible native output fields.
    pub fn derived(
        relation: SemanticId,
        plan: LogicalPlan,
        registry: &Registry,
    ) -> Result<Self, EngineError> {
        let spec = registry
            .relation_by_id(relation)
            .ok_or_else(|| invalid("input declaration absent"))?;
        let plan = super::output::declare_relation_output(plan, registry, spec).map_err(engine)?;
        Ok(Self {
            relation,
            plan,
            witness: None,
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
    /// Execute this exact selected or derived input at an explicit driver boundary.
    /// The returned fields retain the original source witness only when the plan
    /// retained it; replacing a plan cannot manufacture a durable selection.
    /// # Errors
    /// Admission, execution, field validation, cancellation or memory refusal.
    pub async fn execute(
        &self,
        session: &EngineSession,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<RelationFacts, EngineError> {
        self.execute_bounded(session, usize::MAX, usize::MAX, cancel)
            .await
    }
    /// Capture a finite relation with limits checked before decoding/retaining each batch.
    /// Stream operators use the session pool; this bound covers the captured output,
    /// not process RSS or upstream operator workspace.
    /// # Errors
    /// Unbounded stream, exhausted output allowance or any ordinary execution failure.
    pub async fn execute_bounded(
        &self,
        session: &EngineSession,
        rows: usize,
        bytes: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<RelationFacts, EngineError> {
        let spec = session
            .registry()
            .relation_by_id(self.relation)
            .ok_or_else(|| invalid("input declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema(session.registry(), spec)
            .map_err(pse_relations::RelationError::from)?;
        let mut stream = session
            .prepare_rule_plan(self.plan.clone(), cancel)?
            .execute_stream(cancel)
            .await?;
        if !stream.is_bounded() {
            return Err(invalid("bounded capture requires a finite stream"));
        }
        let mut parts = Vec::new();
        let mut row_count = 0usize;
        let mut byte_count = 0usize;
        while let Some(batch) = stream.next_batch(cancel).await? {
            row_count = row_count.saturating_add(batch.batch().num_rows());
            byte_count =
                byte_count.saturating_add(pse_columnar::algorithm_decode_extent(batch.batch())?);
            if row_count > rows || byte_count > bytes {
                return Err(EngineError::ResourceLimit {
                    consumer: "bounded relation output".into(),
                    detail: format!(
                        "captured {row_count} rows / {byte_count} bytes exceeds {rows} rows / {bytes} bytes"
                    ),
                    config_keys: vec![
                        "compiler.output_rows".into(),
                        "compiler.output_bytes".into(),
                    ],
                });
            }
            parts.push(FieldCheckedBatch::admit_owned(
                session.registry(),
                spec,
                super::output::restore_relation_metadata(batch, &schema)?,
            )?);
        }
        Ok(RelationFacts {
            checked: FieldCheckedBatch::concat_reserved(
                session.registry(),
                spec,
                &parts,
                session.pool(),
                cancel,
            )?,
            witness: self.witness.clone(),
        })
    }
    /// Replace a native child. Only an unchanged selected scan keeps its witness.
    #[must_use]
    pub fn rewritten(&self, plan: LogicalPlan) -> Self {
        Self {
            witness: self.witness.clone().filter(|_| plan == self.plan),
            relation: self.relation,
            plan,
        }
    }
    /// Physical replacement cannot retain an exact selected-source receipt.
    #[must_use]
    pub fn without_source_witness(&self) -> Self {
        Self {
            witness: None,
            ..self.clone()
        }
    }
    /// Retain a required finite input with the caller's native cache factory.
    /// Only an unchanged underlying selected scan keeps its exact source witness.
    /// # Errors
    /// Factory refusal, changed fields, cancellation or resource admission failure.
    pub fn retained(
        &self,
        session: &EngineSession,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, EngineError> {
        Self::retained_many(std::slice::from_ref(self), session, cancel)?
            .pop()
            .ok_or_else(|| invalid("retained native input absent"))
    }
    /// Retain sibling finite inputs with one shared native admission scope.
    /// Each actual factory result is still checked before becoming an input.
    /// # Errors
    /// Factory refusal, changed fields, cancellation or resource admission failure.
    pub fn retained_many(
        inputs: &[Self],
        session: &EngineSession,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Vec<Self>, EngineError> {
        let plans = session.cache_plans(
            &inputs
                .iter()
                .map(|input| input.plan.clone())
                .collect::<Vec<_>>(),
            cancel,
        )?;
        Ok(inputs
            .iter()
            .zip(plans)
            .map(|(input, plan)| input.with_retained(plan))
            .collect())
    }
    fn with_retained(&self, plan: LogicalPlan) -> Self {
        let mut actual = &plan;
        loop {
            if let Some(input) = super::cache::evidence_input(actual) {
                actual = input;
                continue;
            }
            if let LogicalPlan::Extension(extension) = actual
                && let Some(contract) = extension
                    .node
                    .as_any()
                    .downcast_ref::<super::contract::ExecutionContract>()
            {
                actual = contract.operation();
                continue;
            }
            break;
        }
        let witness = self.witness.clone().filter(|_| actual == &self.plan);
        Self {
            relation: self.relation,
            plan,
            witness,
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
    ) -> Result<RelationFacts, EngineError> {
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
        if let Some(chunks) = crate::operation::ports::capture(&child, context.clone()) {
            let batches = chunks.await.map_err(engine)?;
            let checked = FieldCheckedBatch::concat_reserved(
                registry,
                spec,
                &batches,
                services.pool(),
                cancel,
            )?;
            return Ok(RelationFacts {
                checked,
                witness: self.witness.clone(),
            });
        }
        let mut stream = execute_stream(child, context).map_err(engine)?;
        let mut batches = Vec::new();
        let scratch = pse_columnar::MemoryConsumer::new("native:finite-child-admission")
            .register(services.pool());
        while let Some(batch) = stream.try_next().await.map_err(|error| {
            engine(crate::operation::completion::stream_error(
                error,
                !batches.is_empty(),
            ))
        })? {
            cancel.checkpoint()?;
            scratch.try_resize(pse_columnar::columnar_validation_extent(&batch)?)?;
            let owned = services
                .ownership()
                .export(batch, services.pool(), cancel)?
                .with_schema_metadata(schema.metadata().clone())?;
            batches.push(FieldCheckedBatch::admit_owned(registry, spec, owned)?);
        }
        let checked =
            FieldCheckedBatch::concat_reserved(registry, spec, &batches, services.pool(), cancel)?;
        Ok(RelationFacts {
            checked,
            witness: self.witness.clone(),
        })
    }
}
impl EngineSession {
    /// Bind the actual native selected source as a finite algorithm child.
    /// # Errors
    /// Unknown or undeclared source, or invalid native scan.
    pub fn relation_plan(
        &self,
        reference: &ResolvedTableReference,
    ) -> Result<RelationPlan, EngineError> {
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
            witness: binding.witness.clone(),
        })
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "native.input".into(),
        reason: reason.into(),
    }
}

impl EngineSession {
    /// Bind declared native plans as a private relational query scope. DataFusion's
    /// native view expansion keeps their real children visible; no rows are collected
    /// and no key or durable identity is inferred from their output schema.
    /// # Errors
    /// Foreign input providers, incompatible declarations, or native field admission.
    pub fn plan_workspace(
        &self,
        inputs: std::collections::BTreeMap<pse_schema::model::RelationKey, RelationPlan>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, EngineError> {
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
        let sources = self.source_bindings_many(&plans, cancel)?;
        for (((key, input), plan), sources) in inputs.into_iter().zip(plans).zip(sources) {
            cancel.checkpoint()?;
            let spec = self
                .registry
                .relation_by_key(key)
                .filter(|spec| spec.id == input.relation)
                .ok_or_else(|| invalid("planned relation differs from its argument declaration"))?;
            let source_dependencies = Self::source_binding_dependencies(&sources)?;
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
            binding.source_dependencies = source_dependencies;
            binding.witness = input.witness;
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
    pub fn source_dependencies(
        &self,
        plan: &LogicalPlan,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Vec<crate::provider::witness::SourceWitness>, EngineError> {
        Self::source_binding_dependencies(&self.source_bindings(plan, cancel)?)
    }

    fn source_binding_dependencies(
        bindings: &[Arc<crate::provider::binding::TableBinding>],
    ) -> Result<Vec<crate::provider::witness::SourceWitness>, EngineError> {
        let mut selected = std::collections::BTreeMap::new();
        for binding in bindings {
            for member in binding.witness.iter().chain(&binding.source_dependencies) {
                let name = member.reference();
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

    /// Resolve actual plan sources against this retained binding inventory.
    /// # Errors
    /// A plan names an unbound or different actual source.
    pub fn source_bindings(
        &self,
        plan: &LogicalPlan,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Vec<Arc<crate::provider::binding::TableBinding>>, EngineError> {
        self.source_bindings_many(std::slice::from_ref(plan), cancel)?
            .pop()
            .ok_or_else(|| invalid("source discovery root absent"))
    }

    /// One scoped native graph retains each root's exact source set. Shared
    /// descendants are inspected once, including hidden cache inputs and subqueries.
    fn source_bindings_many(
        &self,
        plans: &[LogicalPlan],
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Vec<Vec<Arc<crate::provider::binding::TableBinding>>>, EngineError> {
        use datafusion::common::tree_node::TreeNodeRecursion;
        let memory =
            pse_columnar::MemoryConsumer::new("session:source-discovery").register(&self.pool);
        let graph = super::traversal::graph(
            plans,
            super::traversal::Purpose::Evidence,
            cancel,
            |bytes| {
                memory
                    .try_grow(bytes)
                    .map_err(|error| pse_columnar::external(EngineError::from(error)))
            },
            |_, _| Ok(TreeNodeRecursion::Continue),
        )
        .map_err(engine)?;
        memory.try_grow(self.bindings.iter().count().saturating_mul(128))?;
        let bindings = self
            .bindings
            .iter()
            .map(|(_, binding)| binding)
            .collect::<Vec<_>>();
        memory.try_grow(graph.nodes.len().saturating_mul(size_of::<Vec<usize>>()))?;
        let mut selected = vec![Vec::<usize>::new(); graph.nodes.len()];
        for &id in &graph.postorder {
            cancel.checkpoint()?;
            let node = &graph.nodes[id];
            let mut sources = if let LogicalPlan::TableScan(scan) = &node.plan {
                self.scan_bindings(scan, &node.scope, &bindings)?
            } else {
                Vec::new()
            };
            memory.try_grow(sources.len().saturating_mul(2 * size_of::<usize>()))?;
            for &child in &node.children {
                for &source in &selected[child] {
                    if !sources.contains(&source) {
                        memory.try_grow(2 * size_of::<usize>())?;
                        sources.push(source);
                    }
                }
            }
            selected[id] = sources;
        }
        graph
            .roots
            .iter()
            .map(|&id| {
                memory.try_grow(selected[id].len().saturating_mul(16))?;
                Ok(selected[id]
                    .iter()
                    .map(|&index| bindings[index].clone())
                    .collect())
            })
            .collect()
    }

    fn scan_bindings(
        &self,
        scan: &datafusion::logical_expr::TableScan,
        scope: &super::traversal::Scope,
        bindings: &[Arc<crate::provider::binding::TableBinding>],
    ) -> Result<Vec<usize>, EngineError> {
        let provider = datafusion::datasource::source_as_provider(&scan.source).map_err(engine)?;
        if provider.is::<datafusion_catalog::cte_worktable::CteWorkTable>() {
            super::admission::admit_scan(scan, &self.registry, &[], &scope.worktables)
                .map_err(engine)?;
            return Ok(Vec::new());
        }
        let exact = bindings
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| {
                Arc::ptr_eq(&binding.provider, &provider).then_some(index)
            })
            .collect::<Vec<_>>();
        // Prefer the actual scan role. Expanded views retain all matching roles;
        // a shared provider address alone cannot select one durable identity.
        let matching = if let Some(&named) = exact
            .iter()
            .find(|&&index| bindings[index].reference == scan.table_name)
        {
            vec![named]
        } else if !exact.is_empty() {
            exact
        } else {
            bindings
                .iter()
                .enumerate()
                .filter_map(|(index, binding)| {
                    binding
                        .dependencies
                        .iter()
                        .any(|input| Arc::ptr_eq(input, &provider))
                        .then_some(index)
                })
                .collect()
        };
        if matching.is_empty() {
            return Err(engine(datafusion::common::DataFusionError::Plan(format!(
                "dependency source {} is outside the native generation",
                scan.table_name.to_quoted_string()
            ))));
        }
        Ok(matching)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_candidates_and_workspaces_use_their_actual_sql_function_owner() {
        use datafusion::{
            arrow::{
                array::{Int64Array, RecordBatch},
                datatypes::DataType,
            },
            common::ScalarValue,
            execution::session_state::SessionStateBuilder,
            logical_expr::{ColumnarValue, Volatility, create_udf},
        };
        use pse_schema::model::{
            Authority, DerivationGranularity, FieldContract, Namespace, RelationDecl, SnapshotClass,
        };
        use std::collections::BTreeMap;
        let mut declarations = pse_schema::RegistryBuilder::new();
        declarations.declare_relation(
            RelationDecl::new(
                Namespace::Normalized,
                "context_values",
                1,
                Authority::Derived,
                SnapshotClass::Model,
                "Actual admission function owner control",
            )
            .granularity(DerivationGranularity::Rule)
            .columns(vec![FieldContract::key(
                "value",
                FieldContract::native(DataType::Int64),
                "value",
            )])
            .pk(&["value"])
            .checks(BTreeMap::from([(
                "floor".into(),
                "value >= native_floor()".into(),
            )])),
        );
        let registry = Arc::new(declarations.build().unwrap());
        let spec = registry.relation("normalized.context_values").unwrap();
        let batch = RecordBatch::try_new(
            pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
            vec![Arc::new(Int64Array::from(vec![2]))],
        )
        .unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        for floor in [1, 3] {
            let function = create_udf(
                "native_floor",
                vec![],
                DataType::Int64,
                Volatility::Immutable,
                Arc::new(move |_| Ok(ColumnarValue::Scalar(ScalarValue::Int64(Some(floor))))),
            );
            let context = datafusion::prelude::SessionContext::new();
            let factory = super::super::EngineFactory::from_builder(
                context.runtime_env(),
                Arc::new(pse_columnar::GreedyMemoryPool::new(4 << 20)),
                "admission-owner-unit",
                SessionStateBuilder::from(context.state())
                    .with_scalar_functions(vec![Arc::new(function)]),
            );
            let rows = BTreeMap::from([(spec.key, batch.clone())]);
            assert_eq!(
                factory
                    .candidate(rows.clone(), registry.clone(), &cancel)
                    .is_ok(),
                floor == 1
            );
            let session = factory
                .candidate(BTreeMap::new(), registry.clone(), &cancel)
                .unwrap();
            assert_eq!(
                session.with_workspace(rows.clone(), &cancel).is_ok(),
                floor == 1
            );
            let stopped = pse_columnar::CancellationToken::new();
            stopped.cancel();
            assert!(
                factory
                    .candidate(rows.clone(), registry.clone(), &stopped)
                    .is_err()
            );
            assert!(session.with_workspace(rows, &stopped).is_err());
        }
    }

    #[test]
    fn sibling_source_sets_preserve_named_roles_and_empty_roots() {
        let context = datafusion::prelude::SessionContext::new();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(4 << 20));
        let cancel = pse_columnar::CancellationToken::new();
        let session = super::super::EngineFactory::from_builder(
            context.runtime_env(),
            pool.clone(),
            "source-set-unit",
            datafusion::execution::session_state::SessionStateBuilder::from(context.state()),
        )
        .candidate_checked(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
        let provider: Arc<dyn datafusion::catalog::TableProvider> = Arc::new(
            datafusion::datasource::MemTable::try_new(
                Arc::new(datafusion::arrow::datatypes::Schema::empty()),
                vec![vec![]],
            )
            .unwrap(),
        );
        let session = session
            .with_provider("first".into(), provider.clone(), &cancel)
            .unwrap()
            .with_provider("second".into(), provider.clone(), &cancel)
            .unwrap();
        let bindings = session
            .bindings
            .iter()
            .map(|(_, binding)| binding)
            .collect::<Vec<_>>();
        let scan = |name: &str| {
            LogicalPlanBuilder::scan(
                bindings
                    .iter()
                    .find(|binding| binding.reference.table() == name)
                    .unwrap()
                    .reference
                    .clone(),
                datafusion::datasource::provider_as_source(provider.clone()),
                None,
            )
            .unwrap()
            .build()
            .unwrap()
        };
        let mut shared = scan("first");
        for _ in 0..24 {
            let child = Arc::new(shared);
            shared = LogicalPlan::Union(datafusion::logical_expr::Union {
                schema: child.schema().clone(),
                inputs: vec![child.clone(), child],
            });
        }
        let mut plans = vec![shared; 32];
        plans.push(scan("second"));
        plans.push(LogicalPlanBuilder::empty(false).build().unwrap());
        let before = pool.reserved();
        let result = session.source_bindings_many(&plans, &cancel).unwrap();
        assert_eq!(result.len(), 34);
        for sources in &result[..32] {
            assert_eq!(sources.len(), 1);
            assert_eq!(sources[0].reference.table(), "first");
        }
        assert_eq!(result[32].len(), 1);
        assert_eq!(result[32][0].reference.table(), "second");
        assert!(result[33].is_empty());
        assert_eq!(pool.reserved(), before);
        cancel.cancel();
        assert!(session.source_bindings_many(&plans, &cancel).is_err());
    }

    #[test]
    fn source_discovery_visits_shared_nodes_once_per_scope() {
        let context = datafusion::prelude::SessionContext::new();
        let budget: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let factory = super::super::EngineFactory::from_builder(
            context.runtime_env(),
            budget.clone(),
            "source-discovery-unit",
            datafusion::execution::session_state::SessionStateBuilder::from(context.state()),
        );
        let session = factory
            .candidate_checked(
                std::collections::BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &pse_columnar::CancellationToken::new(),
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
            assert!(
                session
                    .source_bindings(&plan, &pse_columnar::CancellationToken::new())
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(
                super::super::traversal::visit(
                    &plan,
                    super::super::traversal::Purpose::Evidence,
                    &budget,
                    &pse_columnar::CancellationToken::new(),
                    |_, _| Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
                )
                .unwrap(),
                21
            );
        }
        assert_eq!(budget.reserved(), before);
        let cancel = pse_columnar::CancellationToken::new();
        let provider: Arc<dyn datafusion::catalog::TableProvider> = Arc::new(
            datafusion::datasource::MemTable::try_new(
                Arc::new(datafusion::arrow::datatypes::Schema::empty()),
                vec![vec![]],
            )
            .unwrap(),
        );
        let session = session
            .with_provider("first".into(), provider.clone(), &cancel)
            .unwrap()
            .with_provider("second".into(), provider.clone(), &cancel)
            .unwrap();
        let second = session
            .bindings
            .iter()
            .find_map(|(_, binding)| (binding.reference.table() == "second").then_some(binding))
            .unwrap();
        for (reference, expected) in [(second.reference.clone(), 1), ("expanded".into(), 2)] {
            let plan = LogicalPlanBuilder::scan(
                reference,
                datafusion::datasource::provider_as_source(provider.clone()),
                None,
            )
            .unwrap()
            .build()
            .unwrap();
            let sources = session.source_bindings(&plan, &cancel).unwrap();
            assert_eq!(sources.len(), expected);
            assert!(sources.iter().any(|source| Arc::ptr_eq(source, &second)));
        }
    }
}

/// A composed native query consumed at an explicit finite algorithm boundary.
/// It establishes field derivation; global relation validity is not implied.
#[derive(Clone, Debug)]
pub struct ComputationPlan {
    plan: LogicalPlan,
    registry: Arc<Registry>,
}
/// Captured native query chunks, with allocation ownership and exact field shape.
#[derive(Debug)]
pub struct CapturedComputation {
    pub(super) schema: datafusion::common::DFSchemaRef,
    pub(super) batches: Vec<pse_columnar::owned_buffer::OwnedRecordBatch>,
    pub(super) registry: Arc<Registry>,
}
impl EngineSession {
    /// Admit a composed query without executing it or inventing a registry relation.
    /// # Errors
    /// Invalid fields, providers, cancellation or memory admission.
    pub fn computation_plan(
        &self,
        plan: LogicalPlan,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ComputationPlan, EngineError> {
        Ok(ComputationPlan {
            plan: self.cache_plan(plan, cancel)?,
            registry: self.registry.clone(),
        })
    }
}
impl ComputationPlan {
    /// Actual native dependencies, available before execution.
    pub const fn plan(&self) -> &LogicalPlan {
        &self.plan
    }
    /// Retain the actual rewritten query; output field checks remain live.
    #[must_use]
    pub fn rewritten(&self, plan: LogicalPlan) -> Self {
        Self {
            plan,
            registry: self.registry.clone(),
        }
    }
    /// Execute the actual planned child at its finite consumer boundary.
    /// # Errors
    /// Changed fields/registry, stream failure, cancellation or resource refusal.
    pub async fn capture(
        &self,
        child: Arc<dyn ExecutionPlan>,
        context: Arc<TaskContext>,
        services: &NativeExecutionContext,
    ) -> Result<CapturedComputation, EngineError> {
        if !Arc::ptr_eq(&self.registry, services.registry())
            || !crate::operation::fields_match(self.plan.schema().fields(), child.schema().fields())
        {
            return Err(invalid("composed finite query fields or registry changed"));
        }
        let schema = self.plan.schema().clone();
        let mut stream = execute_stream(child, context).map_err(engine)?;
        let mut batches = Vec::new();
        while let Some(batch) = stream.try_next().await.map_err(|error| {
            engine(crate::operation::completion::stream_error(
                error,
                !batches.is_empty(),
            ))
        })? {
            services.cancellation().checkpoint()?;
            batches.push(services.ownership().export(
                batch,
                services.pool(),
                services.cancellation(),
            )?);
        }
        Ok(CapturedComputation {
            schema,
            batches,
            registry: self.registry.clone(),
        })
    }
}
