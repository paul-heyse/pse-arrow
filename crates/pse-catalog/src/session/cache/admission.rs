// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reuse structural field admission only for an unchanged, owned native producer.
//! This is neither row validation nor an execution/authorization certificate.
use super::Cache;
use crate::session::SnapshotSession;
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
use pse_ids::{CancellationToken, Reservation};
use pse_schema::Registry;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub(super) struct Admission {
    registry: Arc<Registry>,
    sources: Vec<(Arc<dyn TableProvider>, SchemaRef)>,
    _reservation: Mutex<Box<dyn Reservation>>,
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

pub(super) fn seal(
    plan: LogicalPlan,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<LogicalPlan> {
    // The complete factory result has already passed structural admission.
    Ok(plan
        .transform_down_with_subqueries(|node| {
            let LogicalPlan::Extension(extension) = &node else {
                return Ok(Transformed::no(node));
            };
            let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                return Ok(Transformed::no(node));
            };
            let Some(proof) = proof(&cache.input, session, cancel)? else {
                return Ok(Transformed::no(node));
            };
            Ok(Transformed::new(
                LogicalPlan::Extension(Extension {
                    node: Arc::new(Cache {
                        admission: Some(proof),
                        ..cache.clone()
                    }),
                }),
                true,
                TreeNodeRecursion::Jump,
            ))
        })?
        .data)
}

fn proof(
    input: &LogicalPlan,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Option<Arc<Admission>>> {
    // Open correlated/worktable scopes cannot have a context-independent proof.
    if !super::logical::closed(input)? {
        return Ok(None);
    }
    let mut reservation = session.reserver.open("native:cache-field-admission");
    if reservation.try_grow(256).is_err() {
        return Ok(None);
    }
    let mut scratch = session.reserver.open("native:cache-field-discovery");
    let mut seen = HashSet::new();
    let mut sources: Vec<(Arc<dyn TableProvider>, SchemaRef)> = Vec::new();
    let mut fits = true;
    input.apply_with_subqueries(|node| {
        cancel.checkpoint().map_err(|error| {
            datafusion::common::DataFusionError::External(Box::new(crate::CatalogError::from(
                error,
            )))
        })?;
        let key = super::super::admission::identity(node);
        if !matches!(node, LogicalPlan::Subquery(_)) {
            if seen.contains(&key) {
                return Ok(TreeNodeRecursion::Jump);
            }
            if scratch.try_grow(128).is_err() {
                fits = false;
                return Ok(TreeNodeRecursion::Stop);
            }
            seen.insert(key);
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
    })?;
    if !fits {
        return Ok(None);
    }
    Ok(Some(Arc::new(Admission {
        registry: Arc::clone(&session.registry),
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
    fn structural_proof_requires_actual_owners_and_dies_on_rewrite_or_drop() {
        let budget = pse_ids::FixedBudget::new(32 << 20);
        let factory = crate::session::SessionFactory::new(
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
        let plan = session.cache_plan(scan.clone(), &cancel).unwrap();
        let mut found = false;
        let mut owner = None;
        plan.apply_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            {
                found = true;
                owner = Some(Arc::clone(&extension.node));
                assert!(admitted(node, &registry, &[Arc::clone(&provider)]));
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
        drop(owner);
        assert!(budget.reserved() > before);
        assert!(
            empty.derive_plan_fields(plan.clone(), &cancel).is_err(),
            "structural proof cannot admit a foreign provider"
        );
        session.derive_plan_fields(plan.clone(), &cancel).unwrap();
        drop(plan);
        assert_eq!(budget.reserved(), before);

        let factory = crate::session::SessionFactory::from_builder(
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

    fn assert_query_owner(
        session: &SnapshotSession,
        plan: &LogicalPlan,
        registry: &Registry,
        provider: &Arc<dyn TableProvider>,
        owner: &Arc<dyn datafusion::logical_expr::UserDefinedLogicalNode>,
        cancel: &CancellationToken,
    ) {
        let mut memory = session.reserver.open("test:query-boundary");
        let hidden = super::super::logical::for_query_binding(
            plan.clone(),
            registry,
            &[Arc::clone(provider)],
            memory.as_mut(),
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
    struct ForeignFactory(LogicalPlan);
    impl CacheFactory for ForeignFactory {
        fn create(&self, _: LogicalPlan, state: &SessionState) -> Result<LogicalPlan> {
            super::super::NativeCacheFactory.create(self.0.clone(), state)
        }
    }
}
