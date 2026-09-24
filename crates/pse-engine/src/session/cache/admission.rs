// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reuse structural field admission only for an unchanged, owned native producer.
//! This is neither row validation nor an execution/authorization certificate.
use super::Cache;
use crate::session::EngineSession;
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::TableProvider,
    common::{
        Result,
        tree_node::{Transformed, TreeNodeRecursion},
    },
    datasource::source_as_provider,
    logical_expr::{Extension, LogicalPlan},
};
use pse_columnar::CancellationToken;
use pse_schema::Registry;
use std::sync::{Arc, Mutex};

pub(super) struct Admission {
    registry: Arc<Registry>,
    owner: super::super::reuse::Witness,
    sources: Vec<(Arc<dyn TableProvider>, SchemaRef)>,
    _reservation: Mutex<pse_columnar::MemoryReservation>,
}

impl Admission {
    pub(super) fn retention_eligible(&self) -> bool {
        self.owner.retention_eligible()
    }
    pub(super) fn matches(&self, session: &EngineSession) -> bool {
        self.owner.matches(session)
    }
    fn matches_structure(
        &self,
        session: &EngineSession,
        tables: &[Arc<dyn TableProvider>],
    ) -> bool {
        let context = self.owner.matches_structure(session);
        let sources = self.sources.iter().all(|(provider, schema)| {
            tables.iter().any(|bound| Arc::ptr_eq(provider, bound)) && provider.schema() == *schema
        });
        if !context || !sources {
            tracing::debug!(target: "pse::admission", context, sources,
                consumed = self.sources.len(), available = tables.len(), "field proof mismatch");
        }
        context && sources
    }
    pub(super) fn retained_extent(&self) -> usize {
        self.owner
            .retained_bytes()
            .saturating_add(self.sources.len().saturating_mul(128))
    }
}

pub(in crate::session) fn admitted(
    node: &LogicalPlan,
    registry: &Registry,
    tables: &[Arc<dyn TableProvider>],
) -> bool {
    let LogicalPlan::Extension(extension) = node else {
        return false;
    };
    let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
        return false;
    };
    let Some(proof) = &cache.admission else {
        return false;
    };
    std::ptr::eq(registry, proof.registry.as_ref())
        && proof.sources.iter().all(|(provider, schema)| {
            tables.iter().any(|bound| Arc::ptr_eq(bound, provider)) && provider.schema() == *schema
        })
}

/// Clear structural shortcuts when rebinding under a different semantic assembly.
pub(in crate::session) fn rebind(
    plan: LogicalPlan,
    session: &EngineSession,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    rebind_many(&[plan], session, cancel)?
        .pop()
        .ok_or_else(|| super::invalid("rebound native root absent"))
}

/// Rebind sibling roots in one owner/scope traversal, retaining per-root identity.
pub(in crate::session) fn rebind_many(
    plans: &[LogicalPlan],
    session: &EngineSession,
    cancel: &CancellationToken,
) -> Result<Vec<LogicalPlan>> {
    let tables = session.bindings.providers();
    crate::session::traversal::rewrite(
        plans,
        crate::session::traversal::Purpose::Evidence,
        &session.pool,
        cancel,
        |node, _| {
            if let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                && let Some(proof) = &cache.admission
                && proof.matches_structure(session, &tables)
                && cache
                    .retention
                    .as_ref()
                    .is_none_or(|selection| selection.matches(session))
            {
                return Ok(TreeNodeRecursion::Jump);
            }
            Ok(TreeNodeRecursion::Continue)
        },
        |node, _, _| {
            let LogicalPlan::Extension(extension) = &node else {
                return Ok(Transformed::no(node));
            };
            let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                return Ok(Transformed::no(node));
            };
            let structural_change = cache
                .admission
                .as_ref()
                .is_some_and(|proof| !proof.matches_structure(session, &tables));
            if structural_change
                || cache
                    .retention
                    .as_ref()
                    .is_some_and(|selection| !selection.matches(session))
            {
                tracing::debug!(target: "pse::admission",
                    structure = cache.admission.as_ref().is_some_and(|proof| proof.matches_structure(session, &tables)),
                    retained_values = cache.retention.as_ref().is_some_and(|selection| selection.matches(session)),
                    "invalidating producer retention on rebind");
                return Ok(Transformed::yes(LogicalPlan::Extension(Extension {
                    node: Arc::new(Cache {
                        // A changed value selection does not by itself change
                        // the actual producer's structural field meaning.
                        admission: (!structural_change)
                            .then(|| cache.admission.clone())
                            .flatten(),
                        repeatable_input: !structural_change && cache.repeatable_input,
                        retention: None,
                        optimizer_leaf: false,
                        prepared: None,
                        ..cache.clone()
                    }),
                })));
            }
            Ok(Transformed::no(node))
        },
    )
}

pub(super) fn seal(
    plans: &[LogicalPlan],
    session: &EngineSession,
    cancel: &CancellationToken,
) -> Result<Vec<LogicalPlan>> {
    // The complete factory result has already passed structural admission.
    // Open producers cannot receive a closed proof. They still occur in shared
    // DAGs, so walking each parent path would repeatedly prove the same refusal.
    let tables = session.bindings.providers();
    let retained = |node: &LogicalPlan| {
        matches!(node, LogicalPlan::Extension(extension)
            if extension.node.as_any().downcast_ref::<Cache>().is_some_and(|cache|
                cache.admission.as_ref().is_some_and(|proof| proof.matches_structure(session, &tables))
                    && cache.retention.as_ref().is_none_or(|owner| owner.matches(session))))
    };
    crate::session::traversal::rewrite(
        plans,
        crate::session::traversal::Purpose::Evidence,
        &session.pool,
        cancel,
        |node, _| {
            Ok(if retained(node) {
                TreeNodeRecursion::Jump
            } else {
                TreeNodeRecursion::Continue
            })
        },
        |node, _, _| {
            if retained(&node) {
                return Ok(Transformed::no(node));
            }
            let LogicalPlan::Extension(extension) = &node else {
                return Ok(Transformed::no(node));
            };
            let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                return Ok(Transformed::no(node));
            };
            let Some(proof) = proof(&cache.input, session, cancel)? else {
                return Ok(Transformed::no(node));
            };
            let retention = super::model::Selection::capture(session, &proof)?;
            Ok(Transformed::yes(LogicalPlan::Extension(Extension {
                node: Arc::new(Cache {
                    admission: Some(proof),
                    retention,
                    ..cache.clone()
                }),
            })))
        },
    )
}

fn proof(
    input: &LogicalPlan,
    session: &EngineSession,
    cancel: &CancellationToken,
) -> Result<Option<Arc<Admission>>> {
    // Open correlated/worktable scopes cannot have a context-independent proof.
    if !super::logical::closed(input, &session.pool, cancel)? {
        tracing::debug!(target: "pse::admission", "field proof has open input");
        return Ok(None);
    }
    let reservation =
        pse_columnar::MemoryConsumer::new("native:cache-field-admission").register(&session.pool);
    if reservation.try_grow(256).is_err() {
        return Ok(None);
    }
    let mut sources: Vec<(Arc<dyn TableProvider>, SchemaRef)> = Vec::new();
    let tables = session.bindings.providers();
    let mut fits = true;
    crate::session::traversal::visit(
        input,
        crate::session::traversal::Purpose::Evidence,
        &session.pool,
        cancel,
        |node, _| {
            if admitted(node, &session.registry, &tables)
                && let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                && let Some(proof) = &cache.admission
            {
                for (provider, schema) in &proof.sources {
                    if !sources
                        .iter()
                        .any(|(other, _)| Arc::ptr_eq(other, provider))
                    {
                        if reservation.try_grow(128).is_err() {
                            fits = false;
                            return Ok(TreeNodeRecursion::Stop);
                        }
                        sources.push((Arc::clone(provider), Arc::clone(schema)));
                    }
                }
                return Ok(TreeNodeRecursion::Jump);
            }
            if let LogicalPlan::TableScan(scan) = node {
                let provider = source_as_provider(&scan.source)?;
                if provider
                    .downcast_ref::<datafusion_catalog::cte_worktable::CteWorkTable>()
                    .is_none()
                    && !sources
                        .iter()
                        .any(|(other, _)| Arc::ptr_eq(other, &provider))
                {
                    if reservation.try_grow(128).is_err() {
                        fits = false;
                        return Ok(TreeNodeRecursion::Stop);
                    }
                    sources.push((Arc::clone(&provider), provider.schema()));
                }
            }
            Ok(TreeNodeRecursion::Continue)
        },
    )?;
    if !fits {
        return Ok(None);
    }
    let owner = super::super::reuse::Witness::capture_plan(session, input, &sources, cancel)
        .map_err(pse_columnar::external)?;
    if reservation.try_grow(owner.retained_bytes()).is_err() {
        return Ok(None);
    }
    Ok(Some(Arc::new(Admission {
        registry: Arc::clone(&session.registry),
        owner,
        sources,
        _reservation: Mutex::new(reservation),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::datatypes::{DataType, Field, Schema},
        common::TableReference,
        datasource::{MemTable, provider_as_source},
        execution::{
            runtime_env::RuntimeEnv,
            session_state::{CacheFactory, SessionState},
        },
        logical_expr::{LogicalPlanBuilder, UserDefinedLogicalNodeCore},
    };

    #[test]
    fn shared_open_producers_remain_unsealed_without_expanding_parent_paths() {
        use datafusion::logical_expr::{Expr, Union};
        let budget: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20));
        let factory = crate::session::EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            budget.clone(),
            "open-shared-admission",
            datafusion::execution::session_state::SessionStateBuilder::new()
                .with_default_features(),
        );
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(
                std::collections::BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &cancel,
            )
            .unwrap();
        let mut plan = LogicalPlanBuilder::empty(true)
            .project([Expr::OuterReferenceColumn(
                Arc::new(Field::new("n", DataType::Int64, false)),
                datafusion::common::Column::new_unqualified("n"),
            )])
            .unwrap()
            .build()
            .unwrap();
        for _ in 0..24 {
            let shared = Arc::new(LogicalPlan::Extension(Extension {
                node: Arc::new(Cache {
                    input: plan,
                    required: false,
                    identity: Arc::new(()),
                    prepared: None,
                    optimizer_leaf: false,
                    admission: None,
                    repeatable_input: false,
                    binding: None,
                    retention: None,
                }),
            }));
            plan = LogicalPlan::Union(Union {
                schema: shared.schema().clone(),
                inputs: vec![Arc::clone(&shared), shared],
            });
        }
        let before = budget.reserved();
        let plan = seal(&[plan], &session, &cancel).unwrap().remove(0);
        let mut producers = 0;
        crate::session::traversal::visit(
            &plan,
            crate::session::traversal::Purpose::Evidence,
            &budget,
            &cancel,
            |node, _| {
                if let LogicalPlan::Extension(extension) = node
                    && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                {
                    producers += 1;
                    assert!(cache.admission.is_none());
                    assert!(cache.retention.is_none());
                }
                Ok(TreeNodeRecursion::Continue)
            },
        )
        .unwrap();
        assert_eq!(producers, 24);
        assert_eq!(budget.reserved(), before);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "structural ownership, mutable-result exclusion, rewrite and last-reader controls share the same native owners"
    )]
    fn structural_proof_requires_actual_owners_and_dies_on_rewrite_or_drop() {
        let budget: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20));
        let factory = crate::session::EngineFactory::new(
            Arc::new(RuntimeEnv::default()),
            budget.clone(),
            crate::session::ExecutionSettings::default(),
            crate::session::ThreadBudget {
                pool_threads: 1.try_into().unwrap(),
                target_partitions: 1.try_into().unwrap(),
            },
            crate::session::native_engine_profile(),
        )
        .unwrap();
        let cancel = CancellationToken::new();
        let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
        let empty = factory
            .candidate(
                std::collections::BTreeMap::new(),
                Arc::clone(&registry),
                &cancel,
            )
            .unwrap();
        let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
        let provider: Arc<dyn TableProvider> =
            Arc::new(MemTable::try_new(schema, vec![vec![]]).unwrap());
        let session = empty
            .with_provider(
                TableReference::bare("numbers"),
                Arc::clone(&provider),
                &cancel,
            )
            .unwrap();
        let scan =
            LogicalPlanBuilder::scan("numbers", provider_as_source(Arc::clone(&provider)), None)
                .unwrap()
                .build()
                .unwrap();
        let before = budget.reserved();
        let opaque = crate::operation::Operation::plan(
            Arc::new(FieldOnly(provider.schema())),
            vec![scan.clone()],
        )
        .unwrap();
        let opaque = crate::session::contract::ExecutionContract::plan(
            opaque,
            None,
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        );
        let plan = session.cache_plan(opaque, &cancel).unwrap();
        let mut found = false;
        let mut owner = None;
        plan.apply_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            {
                found = true;
                owner = Some(Arc::clone(&extension.node));
                assert!(admitted(node, &registry, &[Arc::clone(&provider)]));
                let proof = cache.admission.as_ref().unwrap();
                assert!(proof.matches_structure(&session, &[Arc::clone(&provider)]));
                assert!(!proof.owner.matches(&session));
                assert!(cache.retention.is_none());
                let unchanged = cache
                    .with_exprs_and_inputs(vec![], vec![cache.input.clone()])
                    .unwrap();
                assert!(Arc::ptr_eq(proof, unchanged.admission.as_ref().unwrap()));
                assert_eq!(unchanged.repeatable_input, cache.repeatable_input);
                assert!(!admitted(node, &registry, &[]));
                assert!(!admitted(
                    node,
                    &pse_schema::RegistryBuilder::new().build().unwrap(),
                    &[Arc::clone(&provider)]
                ));
                assert!(
                    cache
                        .with_exprs_and_inputs(vec![], vec![scan.clone()])
                        .unwrap()
                        .admission
                        .is_none()
                );
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .unwrap();
        assert!(found);
        assert_query_owner(
            &session,
            &plan,
            &registry,
            &provider,
            owner.as_ref().unwrap(),
            &cancel,
        );
        let unrelated = session
            .with_provider(
                TableReference::bare("unrelated"),
                Arc::new(MemTable::try_new(provider.schema(), vec![vec![]]).unwrap()),
                &cancel,
            )
            .unwrap();
        let restored = unrelated
            .derive_plan_fields_many(&vec![plan.clone(); 32], &cancel)
            .unwrap();
        assert_eq!(restored.len(), 32);
        for root in &restored {
            root.apply_with_subqueries(|node| {
                if let LogicalPlan::Extension(extension) = node
                    && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                {
                    assert!(Arc::ptr_eq(owner.as_ref().unwrap(), &extension.node));
                    assert!(cache.retention.is_none());
                    return Ok(TreeNodeRecursion::Jump);
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .unwrap();
        }
        drop(restored);
        drop(owner);
        assert!(budget.reserved() > before);
        assert!(
            empty.derive_plan_fields(plan.clone(), &cancel).is_err(),
            "structural proof cannot admit a foreign provider"
        );
        session.derive_plan_fields(plan.clone(), &cancel).unwrap();
        drop(plan);
        assert_eq!(budget.reserved(), before);

        let factory = crate::session::EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            budget.clone(),
            "foreign-cache-unit",
            datafusion::execution::session_state::SessionStateBuilder::new()
                .with_default_features()
                .with_cache_factory(Some(Arc::new(ForeignFactory(scan)))),
        );
        let foreign = factory
            .candidate(std::collections::BTreeMap::new(), registry, &cancel)
            .unwrap();
        assert!(
            foreign
                .cache_plan(LogicalPlanBuilder::empty(true).build().unwrap(), &cancel)
                .is_err(),
            "a caller cache factory cannot mint a proof for its unbound replacement input"
        );
    }

    #[test]
    fn changed_value_selection_keeps_unchanged_structural_proof() {
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20));
        let factory = crate::session::EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            pool.clone(),
            "field-selection-unit",
            datafusion::execution::session_state::SessionStateBuilder::new()
                .with_default_features(),
        );
        let cancel = CancellationToken::new();
        let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
        let session = factory
            .candidate(std::collections::BTreeMap::new(), registry.clone(), &cancel)
            .unwrap();
        // An opaque producer captures complete value selection, even when its
        // output field declaration consumes no source providers.
        let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]));
        let opaque =
            crate::operation::Operation::plan(Arc::new(FieldOnly(schema.clone())), vec![]).unwrap();
        let contracted = crate::session::contract::ExecutionContract::plan(
            opaque,
            None,
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        );
        let plan = session.cache_plan(contracted, &cancel).unwrap();
        let mut proof = None;
        plan.apply_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            {
                assert!(cache.retention.as_ref().unwrap().matches(&session));
                proof = cache.admission.clone();
                return Ok(TreeNodeRecursion::Jump);
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .unwrap();
        let proof = proof.unwrap();
        let changed = session
            .with_provider(
                TableReference::bare("unrelated"),
                Arc::new(MemTable::try_new(schema, vec![vec![]]).unwrap()),
                &cancel,
            )
            .unwrap();
        assert!(proof.matches_structure(&changed, &changed.bindings.providers()));
        assert!(!proof.matches(&changed));
        let mut roots = vec![plan; 16];
        for _ in 0..3 {
            roots = rebind_many(&roots, &changed, &cancel).unwrap();
            for root in &roots {
                root.apply_with_subqueries(|node| {
                    if let LogicalPlan::Extension(extension) = node
                        && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
                    {
                        assert!(Arc::ptr_eq(&proof, cache.admission.as_ref().unwrap()));
                        assert!(cache.retention.is_none());
                        assert!(cache.repeatable_input);
                        return Ok(TreeNodeRecursion::Jump);
                    }
                    Ok(TreeNodeRecursion::Continue)
                })
                .unwrap();
            }
        }
    }

    fn assert_query_owner(
        session: &EngineSession,
        plan: &LogicalPlan,
        registry: &Registry,
        provider: &Arc<dyn TableProvider>,
        owner: &Arc<dyn datafusion::logical_expr::UserDefinedLogicalNode>,
        cancel: &CancellationToken,
    ) {
        let mut memory =
            pse_columnar::MemoryConsumer::new("test:query-boundary").register(&session.pool);
        let hidden = super::super::logical::for_query_binding(
            plan.clone(),
            registry,
            &[Arc::clone(provider)],
            &mut memory,
            cancel,
        )
        .unwrap();
        // Native extension traversal can reconstruct even a no-op input.
        let rewritten = hidden
            .transform_up_with_subqueries(|node| Ok(Transformed::no(node)))
            .unwrap()
            .data;
        let restored = super::super::logical::restore_query_bindings(rewritten).unwrap();
        restored
            .apply_with_subqueries(|node| {
                if let LogicalPlan::Extension(extension) = node
                    && extension.node.as_any().is::<Cache>()
                {
                    assert!(Arc::ptr_eq(owner, &extension.node));
                    assert!(admitted(node, registry, &[Arc::clone(provider)]));
                    return Ok(TreeNodeRecursion::Jump);
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .unwrap();
    }
    #[derive(Debug)]
    struct FieldOnly(SchemaRef);
    #[async_trait::async_trait]
    impl crate::operation::Definition for FieldOnly {
        fn name(&self) -> &'static str {
            "structural-field-unit"
        }
        fn schema(&self) -> SchemaRef {
            Arc::clone(&self.0)
        }
        async fn prepare(
            self: Arc<Self>,
            _: &[datafusion::logical_expr::Expr],
            _: &[LogicalPlan],
            _: &[Arc<dyn datafusion::physical_plan::ExecutionPlan>],
            _: &SessionState,
        ) -> Result<Arc<dyn crate::operation::Body>> {
            Err(datafusion::common::DataFusionError::Internal(
                "structural unit cannot execute".into(),
            ))
        }
    }
    #[derive(Debug)]
    struct ForeignFactory(LogicalPlan);
    impl CacheFactory for ForeignFactory {
        fn create(&self, _: LogicalPlan, state: &SessionState) -> Result<LogicalPlan> {
            super::super::NativeCacheFactory.create(self.0.clone(), state)
        }
    }
}
