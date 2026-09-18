// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Derive native shared subplans once within one registry-bound admission.
use super::{FieldAdmissions, derive_native_node};
use datafusion::{
    common::{
        DataFusionError, Result,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion, TreeNodeVisitor},
    },
    logical_expr::LogicalPlan,
};
use pse_ids::{CancellationToken, MemoryReserver, Reservation};
use pse_schema::Registry;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(super) fn plans(
    originals: &[LogicalPlan],
    registry: &Registry,
    tables: &[Arc<dyn datafusion::catalog::TableProvider>],
    fields: &mut FieldAdmissions<'_>,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<Vec<LogicalPlan>> {
    let mut visitor = Deriver {
        registry,
        tables,
        fields,
        cancel,
        restored: HashMap::new(),
        real_nodes: originals.iter().map(identity).collect(),
        reservation: reserver.open("session:plan-admission"),
    };
    // Keep every original root alive throughout this traversal. Their child Arcs
    // pin all memo keys; no address can be recycled or serve as a cross-call key.
    for plan in originals {
        plan.visit_with_subqueries(&mut visitor)?;
    }
    originals
        .iter()
        .map(|plan| visitor.restored_node(identity(plan)))
        .collect()
}

struct Deriver<'a, 'r, 'f, 'c> {
    cancel: &'c CancellationToken,
    registry: &'r Registry,
    tables: &'r [Arc<dyn datafusion::catalog::TableProvider>],
    fields: &'a mut FieldAdmissions<'f>,
    restored: HashMap<NodeIdentity, (LogicalPlan, bool)>,
    real_nodes: HashSet<NodeIdentity>,
    reservation: Box<dyn Reservation>,
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
fn missing() -> DataFusionError {
    DataFusionError::Internal("native admission child was not visited".into())
}
impl Deriver<'_, '_, '_, '_> {
    fn restored_node(&self, key: NodeIdentity) -> Result<LogicalPlan> {
        self.restored
            .get(&key)
            .map(|(plan, _)| plan.clone())
            .ok_or_else(missing)
    }
    fn restored_child(&self, key: NodeIdentity) -> Result<Transformed<LogicalPlan>> {
        self.restored
            .get(&key)
            .map(|(plan, changed)| Transformed::new_transformed(plan.clone(), *changed))
            .ok_or_else(missing)
    }
}
impl<'n> TreeNodeVisitor<'n> for Deriver<'_, '_, '_, '_> {
    type Node = LogicalPlan;
    fn f_down(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        self.cancel.checkpoint().map_err(|error| {
            DataFusionError::External(Box::new(crate::CatalogError::from(error)))
        })?;
        if self.restored.contains_key(&identity(node)) {
            return Ok(TreeNodeRecursion::Jump);
        }
        if crate::session::cache::admitted(node, self.registry, self.tables) {
            self.reservation
                .try_grow(2 * size_of::<LogicalPlan>() + 256)
                .map_err(|error| {
                    DataFusionError::External(Box::new(crate::CatalogError::from(error)))
                })?;
            self.restored.insert(identity(node), (node.clone(), false));
            return Ok(TreeNodeRecursion::Jump);
        }
        self.real_nodes
            .extend(node.inputs().into_iter().map(identity));
        Ok(TreeNodeRecursion::Continue)
    }
    fn f_up(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        let key = identity(node);
        // DataFusion synthesizes temporary Subquery wrappers during visiting.
        // Their children are real retained nodes; their stack addresses are not.
        if !self.real_nodes.contains(&key) || self.restored.contains_key(&key) {
            return Ok(TreeNodeRecursion::Continue);
        }
        // Bound memo/control-table overhead. Native derived plans and Arrow
        // declarations retain their own owners; this is not an allocator limit
        // for DataFusion's schema reconstruction or expression temporaries.
        self.reservation
            .try_grow(2 * size_of::<LogicalPlan>() + 256)
            .map_err(|error| {
                DataFusionError::External(Box::new(crate::CatalogError::from(error)))
            })?;
        let mut inputs = node.inputs().into_iter().map(identity);
        let rebuilt = node
            .clone()
            .map_subqueries(|plan| {
                let LogicalPlan::Subquery(mut query) = plan else {
                    return Err(missing());
                };
                let child = self.restored_child(identity(query.subquery.as_ref()))?;
                if !child.transformed {
                    return Ok(Transformed::no(LogicalPlan::Subquery(query)));
                }
                query.subquery = Arc::new(child.data);
                let mut restored =
                    derive_native_node(LogicalPlan::Subquery(query), self.registry, self.fields)?;
                restored.transformed = true;
                Ok(restored)
            })?
            .transform_data(|plan| {
                plan.map_children(|_| self.restored_child(inputs.next().ok_or_else(missing)?))
            })?;
        let children_changed = rebuilt.transformed;
        let restored = derive_native_node(rebuilt.data, self.registry, self.fields)?;
        let changed = children_changed || restored.transformed;
        // Native reconstruction may allocate an equivalent wrapper. Preserve the
        // original immutable graph when neither children, expressions nor fields
        // changed; otherwise each admission duplicates its shared producers.
        self.restored.insert(
            key,
            (if changed { restored.data } else { node.clone() }, changed),
        );
        Ok(TreeNodeRecursion::Continue)
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
        let budget = pse_ids::FixedBudget::new(1 << 20);
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
            let mut fields = FieldAdmissions::new(&registry, budget.as_ref());
            let mut visitor = Deriver {
                registry: &registry,
                tables: &[],
                fields: &mut fields,
                cancel: &cancel,
                restored: HashMap::new(),
                real_nodes: [identity(&root)].into_iter().collect(),
                reservation: budget.open("test:plan-admission"),
            };
            root.visit_with_subqueries(&mut visitor).unwrap();
            assert_eq!(
                visitor.restored.len(),
                3,
                "union, shared projection and empty input"
            );
            let result = visitor.restored_node(identity(&root)).unwrap();
            assert_eq!(result.inputs().len(), 32);
        }
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn cloned_extension_roots_keep_one_actual_producer_after_repeated_admission() {
        let registry = pse_schema::RegistryBuilder::new().build().unwrap();
        let budget = pse_ids::FixedBudget::new(1 << 20);
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
            let mut fields = FieldAdmissions::new(&registry, budget.as_ref());
            siblings = plans(
                &siblings,
                &registry,
                &[],
                &mut fields,
                budget.as_ref(),
                &cancel,
            )
            .unwrap();
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
        let budget = pse_ids::FixedBudget::new(1 << 20);
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
        let mut fields = FieldAdmissions::new(&registry, budget.as_ref());
        let derived = plans(
            &originals,
            &registry,
            &[],
            &mut fields,
            budget.as_ref(),
            &cancel,
        )
        .unwrap();
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
        assert!(
            plans(
                &originals,
                &registry,
                &[],
                &mut fields,
                budget.as_ref(),
                &cancel
            )
            .is_err()
        );
    }
}
