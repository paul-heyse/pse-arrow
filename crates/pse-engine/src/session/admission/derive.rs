// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Derive retained native plans using the shared lexical/effect scoped walker.
use super::{FieldAdmissions, derive_native_node};
use datafusion::{
    common::{
        Result,
        tree_node::{Transformed, TreeNodeRecursion},
    },
    logical_expr::LogicalPlan,
};
use pse_columnar::CancellationToken;
use pse_columnar::MemoryPool;
use pse_schema::Registry;
use std::sync::Arc;

pub(super) fn plans(
    originals: &[LogicalPlan],
    registry: &Registry,
    tables: &[Arc<dyn datafusion::catalog::TableProvider>],
    fields: &mut FieldAdmissions<'_>,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<Vec<LogicalPlan>> {
    let mut derived_nodes = 0usize;
    let result = crate::session::traversal::rewrite(
        originals,
        crate::session::traversal::Purpose::Rewrite,
        pool,
        cancel,
        |node, _| {
            Ok(if crate::session::cache::admitted(node, registry, tables) {
                TreeNodeRecursion::Jump
            } else {
                TreeNodeRecursion::Continue
            })
        },
        |node, _, _| {
            if crate::session::cache::admitted(&node, registry, tables) {
                Ok(Transformed::no(node))
            } else {
                derived_nodes += 1;
                derive_native_node(node, registry, fields)
            }
        },
    );
    tracing::debug!(target: "pse::admission", derived_nodes, success = result.is_ok(), "native fact derivation");
    result
}
/// Cloning the LogicalPlan enum retains an extension's exact immutable Arc. Its
/// enum wrapper address is not the producer identity: keying only that wrapper
/// rebuilt one shared extension per reference, multiplying every later traversal.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::session) enum NodeIdentity {
    Extension(usize),
    Plan(usize),
}
pub(in crate::session) fn identity(plan: &LogicalPlan) -> NodeIdentity {
    match plan {
        LogicalPlan::Extension(extension) => {
            NodeIdentity::Extension(Arc::as_ptr(&extension.node).cast::<()>() as usize)
        }
        _ => NodeIdentity::Plan(std::ptr::from_ref(plan) as usize),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::array::Int64Array,
        logical_expr::{LogicalPlanBuilder, Projection, Union, lit},
    };

    #[test]
    fn shared_native_children_are_derived_once_and_budget_is_released() {
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        let shared = Arc::new(
            LogicalPlanBuilder::empty(true)
                .project([lit(1i64).alias("n")])
                .unwrap()
                .build()
                .unwrap(),
        );
        let root = LogicalPlan::Union(Union {
            inputs: vec![Arc::clone(&shared); 32],
            schema: Arc::clone(shared.schema()),
        });
        {
            let mut fields = FieldAdmissions::new(&registry, &budget);
            let result = plans(&[root], &registry, &[], &mut fields, &budget, &cancel)
                .unwrap()
                .remove(0);
            assert_eq!(result.inputs().len(), 32);
        }
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn cloned_extension_roots_keep_one_actual_producer_after_repeated_admission() {
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        let producer = crate::session::contract::ExecutionContract::plan(
            LogicalPlanBuilder::empty(true).build().unwrap(),
            None,
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        );
        let mut siblings = vec![producer; 32];
        for _ in 0..4 {
            let LogicalPlan::Extension(previous) = &siblings[0] else {
                panic!("extension absent")
            };
            let previous = Arc::clone(&previous.node);
            let mut fields = FieldAdmissions::new(&registry, &budget);
            siblings = plans(&siblings, &registry, &[], &mut fields, &budget, &cancel).unwrap();
            let LogicalPlan::Extension(first) = &siblings[0] else {
                panic!("extension absent")
            };
            assert!(
                Arc::ptr_eq(&previous, &first.node),
                "unchanged admission replaced the owned producer"
            );
            for sibling in &siblings {
                let LogicalPlan::Extension(sibling) = sibling else {
                    panic!("extension absent")
                };
                assert!(
                    Arc::ptr_eq(&first.node, &sibling.node),
                    "native immutable producer sharing was lost"
                );
            }
        }
        assert_eq!(budget.reserved(), 0);
    }

    #[tokio::test]
    async fn equal_schema_owners_do_not_conflate_distinct_expressions() {
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        let input = Arc::new(LogicalPlanBuilder::empty(true).build().unwrap());
        let first = Projection::try_new(vec![lit(1i64).alias("n")], Arc::clone(&input)).unwrap();
        let second = Projection::try_new_with_schema(
            vec![lit(2i64).alias("n")],
            input,
            Arc::clone(&first.schema),
        )
        .unwrap();
        let originals = [
            LogicalPlan::Projection(first),
            LogicalPlan::Projection(second),
        ];
        let mut fields = FieldAdmissions::new(&registry, &budget);
        let derived = plans(&originals, &registry, &[], &mut fields, &budget, &cancel).unwrap();
        let context = datafusion::prelude::SessionContext::new();
        for (plan, expected) in derived.into_iter().zip([1, 2]) {
            let batches = context
                .execute_logical_plan(plan)
                .await
                .unwrap()
                .collect()
                .await
                .unwrap();
            assert_eq!(
                batches[0]
                    .column(0)
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .value(0),
                expected
            );
        }
        cancel.cancel();
        assert!(plans(&originals, &registry, &[], &mut fields, &budget, &cancel).is_err());
    }
}
