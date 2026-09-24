// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Classify actual native owners without cloning expressions or expanding shared
//! extensions repeatedly. Contracts remain scoped to each root; a contract in a
//! sibling output cannot qualify an otherwise uncontracted operation.
use super::{EngineSession, contract::ExecutionContract};
use datafusion::{
    common::{Result, tree_node::TreeNodeRecursion},
    logical_expr::{Expr, LogicalPlan, Volatility},
};
use pse_columnar::CancellationToken;
use pse_schema::model::provider::OperationEffect;
use std::sync::Arc;

impl EngineSession {
    /// Inspect original expressions and sources before optimization can erase varying reads.
    /// # Errors
    /// Cancellation or unsupported native dependency inspection.
    pub fn plans_require_fresh<'a>(
        &self,
        plans: impl IntoIterator<Item = &'a LogicalPlan>,
        cancel: &CancellationToken,
    ) -> Result<bool> {
        self.plan_freshness(plans, cancel)
            .map(|facts| facts.required)
    }

    pub(super) fn plan_freshness<'a>(
        &self,
        plans: impl IntoIterator<Item = &'a LogicalPlan>,
        cancel: &CancellationToken,
    ) -> Result<Freshness> {
        self.plan_freshness_many(plans, cancel).map(combined)
    }

    pub(super) fn plan_freshness_many<'a>(
        &self,
        plans: impl IntoIterator<Item = &'a LogicalPlan>,
        cancel: &CancellationToken,
    ) -> Result<Vec<Freshness>> {
        let reservation =
            pse_columnar::MemoryConsumer::new("session:freshness-discovery").register(&self.pool);
        classify_many(
            plans,
            |bytes| {
                reservation
                    .try_grow(bytes)
                    .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))
            },
            cancel,
        )
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Freshness {
    pub required: bool,
    pub nondeterministic: bool,
}

#[derive(Clone, Copy)]
struct Facts {
    varying: bool,
    nondeterministic: bool,
    extension: Option<usize>,
}

pub(super) fn check<'a>(
    plans: impl IntoIterator<Item = &'a LogicalPlan>,
    charge: impl FnMut(usize) -> Result<()>,
    cancel: &CancellationToken,
) -> Result<bool> {
    classify(plans, charge, cancel).map(|facts| facts.required)
}

fn combined(values: Vec<Freshness>) -> Freshness {
    values
        .into_iter()
        .fold(Freshness::default(), |mut result, value| {
            result.required |= value.required;
            result.nondeterministic |= value.nondeterministic;
            result
        })
}
fn classify<'a>(
    plans: impl IntoIterator<Item = &'a LogicalPlan>,
    charge: impl FnMut(usize) -> Result<()>,
    cancel: &CancellationToken,
) -> Result<Freshness> {
    classify_many(plans, charge, cancel).map(combined)
}
fn classify_many<'a>(
    plans: impl IntoIterator<Item = &'a LogicalPlan>,
    mut charge: impl FnMut(usize) -> Result<()>,
    cancel: &CancellationToken,
) -> Result<Vec<Freshness>> {
    let graph = super::traversal::graph(
        plans,
        super::traversal::Purpose::Freshness,
        cancel,
        &mut charge,
        |node, _| {
            Ok(if super::cache::repeatable_input(node) {
                TreeNodeRecursion::Jump
            } else {
                TreeNodeRecursion::Continue
            })
        },
    )?;
    charge(graph.nodes.len().saturating_mul(size_of::<Freshness>()))?;
    let mut results = vec![Freshness::default(); graph.nodes.len()];
    let mut facts = std::collections::HashMap::new();
    for &id in &graph.postorder {
        cancel
            .checkpoint()
            .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))?;
        let node = &graph.nodes[id];
        let value = match facts.entry(node.owner) {
            std::collections::hash_map::Entry::Occupied(entry) => *entry.get(),
            std::collections::hash_map::Entry::Vacant(entry) => *entry.insert(inspect(&node.plan)?),
        };
        let mut result = Freshness {
            required: value.varying
                || value
                    .extension
                    .is_some_and(|owner| !node.scope.qualified.contains(&owner)),
            nondeterministic: value.nondeterministic,
        };
        if result.required {
            tracing::debug!(target: "pse::admission", node = id, varying = value.varying,
                nondeterministic = value.nondeterministic,
                extension = match &node.plan {
                    LogicalPlan::Extension(extension) => extension.node.name(),
                    _ => "native",
                },
                qualified = node.scope.qualified.len(),
                "freshness required by native owner");
        }
        for &child in &node.children {
            result.required |= results[child].required;
            result.nondeterministic |= results[child].nondeterministic;
        }
        results[id] = result;
    }
    charge(graph.roots.len().saturating_mul(size_of::<Freshness>()))?;
    Ok(graph.roots.iter().map(|&root| results[root]).collect())
}

fn inspect(node: &LogicalPlan) -> Result<Facts> {
    let mut result = Facts {
        varying: false,
        nondeterministic: false,
        extension: None,
    };
    if let LogicalPlan::Extension(extension) = node {
        if let Some(contract) = extension.node.as_any().downcast_ref::<ExecutionContract>() {
            result.varying |= contract
                .effects()
                .iter()
                .any(|effect| *effect != OperationEffect::Read);
        } else if let Some(operation) = extension
            .node
            .as_any()
            .downcast_ref::<crate::operation::Operation>()
        {
            result.varying |= operation
                .effects()
                .iter()
                .any(|effect| *effect != OperationEffect::Read);
        } else if !super::cache::is_required(extension.node.as_ref()) {
            // Internal obligation sharing adds no effects; inspect its actual
            // children rather than treating the transport as an opaque owner.
            result.extension = Some(Arc::as_ptr(&extension.node).cast::<()>() as usize);
        }
    }
    node.apply_expressions(|expression| {
        super::traversal::expression(expression, |expression| {
            result.nondeterministic |= match expression {
                Expr::ScalarFunction(value) => {
                    value.func.signature().volatility != Volatility::Immutable
                }
                Expr::AggregateFunction(value) => {
                    value.func.signature().volatility != Volatility::Immutable
                }
                Expr::WindowFunction(value) => {
                    value.fun.signature().volatility != Volatility::Immutable
                }
                Expr::HigherOrderFunction(value) => {
                    value.func.signature().volatility != Volatility::Immutable
                }
                Expr::ScalarVariable(..) => true,
                _ => false,
            };
            result.varying |= result.nondeterministic;
            Ok(if result.nondeterministic {
                TreeNodeRecursion::Stop
            } else {
                TreeNodeRecursion::Continue
            })
        })
    })?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::logical_expr::{LogicalPlanBuilder, Union};

    #[test]
    fn sibling_freshness_keeps_each_root_qualification_and_effects() {
        use datafusion::execution::session_state::{CacheFactory, SessionStateBuilder};
        let state = SessionStateBuilder::new().with_default_features().build();
        let contracted = super::super::cache::NativeCacheFactory
            .create(LogicalPlanBuilder::empty(false).build().unwrap(), &state)
            .unwrap();
        let LogicalPlan::Extension(extension) = &contracted else {
            panic!("contract absent")
        };
        let unqualified = extension
            .node
            .as_any()
            .downcast_ref::<ExecutionContract>()
            .unwrap()
            .operation()
            .clone();
        let effect = ExecutionContract::plan(
            unqualified.clone(),
            None,
            [OperationEffect::Write].into_iter().collect(),
        );
        let results = classify_many(
            [&contracted, &unqualified, &effect, &contracted],
            |_| Ok(()),
            &CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(
            results
                .iter()
                .map(|facts| facts.required)
                .collect::<Vec<_>>(),
            [false, true, true, false]
        );
        assert!(results.iter().all(|facts| !facts.nondeterministic));
        assert!(super::super::cache::repeatable_input(&unqualified));
        let replacement = ExecutionContract::plan(
            LogicalPlanBuilder::empty(false).build().unwrap(),
            None,
            [OperationEffect::Observe].into_iter().collect(),
        );
        let rewritten = unqualified
            .with_new_exprs(vec![], vec![replacement])
            .unwrap();
        assert!(!super::super::cache::repeatable_input(&rewritten));
        let read = ExecutionContract::plan(
            rewritten,
            None,
            [OperationEffect::Read].into_iter().collect(),
        );
        assert!(check([&read], |_| Ok(()), &CancellationToken::new()).unwrap());
    }

    #[test]
    fn fresh_effects_do_not_fabricate_nondeterministic_expressions() {
        let plan = ExecutionContract::plan(
            LogicalPlanBuilder::empty(true).build().unwrap(),
            None,
            [OperationEffect::Read, OperationEffect::Observe]
                .into_iter()
                .collect(),
        );
        let facts = classify([&plan], |_| Ok(()), &CancellationToken::new()).unwrap();
        assert!(facts.required);
        assert!(!facts.nondeterministic);
    }

    #[tokio::test]
    async fn stable_time_fold_is_reprepared_for_a_second_attempt() {
        let context = datafusion::execution::context::SessionContext::new();
        let factory = super::super::EngineFactory::from_builder(
            context.runtime_env(),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            "stable-unit",
            datafusion::execution::session_state::SessionStateBuilder::from(context.state())
                .with_query_planner(Arc::new(super::super::planner::UnifiedPlanner::default())),
        );
        let cancel = CancellationToken::new();
        let session = factory
            .candidate_checked(
                std::collections::BTreeMap::new(),
                Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
                &cancel,
            )
            .unwrap();
        let prepared = session
            .prepare_sql("SELECT now() AS observed", &cancel)
            .await
            .unwrap();
        assert!(prepared.contains_volatile_expression());
        let first = prepared.clone().execute(&cancel).await.unwrap();
        let second = prepared.execute(&cancel).await.unwrap();
        assert_ne!(
            first.prepared().query_start_time(),
            second.prepared().query_start_time(),
            "a repeated stable query must receive a fresh preparation"
        );
        assert_ne!(
            first.batches()[0].column(0),
            second.batches()[0].column(0),
            "a folded stable value cannot survive a fresh attempt"
        );
    }

    #[test]
    fn sibling_outputs_share_only_native_facts_and_keep_bounded_scratch() {
        let mut plan = LogicalPlanBuilder::empty(true).build().unwrap();
        for _ in 0..18 {
            let input = Arc::new(ExecutionContract::plan(
                plan,
                None,
                [OperationEffect::Read].into_iter().collect(),
            ));
            plan = LogicalPlan::Union(Union {
                inputs: vec![Arc::clone(&input), Arc::clone(&input)],
                schema: Arc::clone(input.schema()),
            });
        }
        let roots = vec![plan; 128];
        let cancel = CancellationToken::new();
        let mut charged = 0;
        assert!(
            !check(
                roots.iter(),
                |bytes| {
                    charged += bytes;
                    Ok(())
                },
                &cancel
            )
            .unwrap()
        );
        // 128 distinct Union root values, with all their actual shared children.
        // Growth follows unique native owners, not the expanded binary tree or
        // a fresh scratch allocation for each of the 128 output roots.
        assert!(charged < 512 << 10, "actual charged scratch: {charged}");
        cancel.cancel();
        assert!(check(roots.iter(), |_| Ok(()), &cancel).is_err());
    }
}
