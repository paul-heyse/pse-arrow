// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Preserve native node rendering and child order without expanding a DAG into a
//! deeply indented JSON tree. DataFusion owns every operator-specific property.
use super::{BoundedText, CatalogError, invalid};
use crate::session::admission::{NodeIdentity, identity};
use datafusion::{
    common::{
        DataFusionError, Result,
        tree_node::{TreeNodeRecursion, TreeNodeVisitor},
    },
    logical_expr::{LogicalPlan, logical_plan::display::PgJsonVisitor},
};
use pse_ids::CancellationToken;
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    sync::Arc,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Node(NodeIdentity),
    // Native traversal creates temporary wrappers around expression subqueries.
    // The actual subquery Arc remains pinned by the original root throughout.
    Subquery(usize),
}
impl Key {
    fn of(node: &LogicalPlan) -> Self {
        match node {
            LogicalPlan::Subquery(query) => Self::Subquery(Arc::as_ptr(&query.subquery) as usize),
            _ => Self::Node(identity(node)),
        }
    }
}

struct NativeNode<'a>(&'a LogicalPlan);
impl Display for NativeNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Calling the public callbacks on this one node avoids serializing its
        // whole subtree. In particular, pretty-print indentation stays constant.
        let mut native = PgJsonVisitor::new(f);
        native.with_schema(true);
        native.f_down(self.0).map_err(|_| std::fmt::Error)?;
        native.f_up(self.0).map_err(|_| std::fmt::Error)?;
        Ok(())
    }
}

pub(super) fn write(
    plan: &LogicalPlan,
    writer: &mut BoundedText<'_>,
    cancel: &CancellationToken,
) -> std::result::Result<(), CatalogError> {
    let mut graph = Graph {
        writer,
        cancel,
        nodes: HashMap::new(),
        parents: Vec::new(),
        edges: Vec::new(),
        control_bytes: 0,
    };
    let result = graph.render(plan);
    // The retained evidence contains text only. All pointer keys and traversal
    // allocations disappear here, including on cancellation or budget refusal.
    graph.writer.reservation.shrink(graph.control_bytes);
    result
}

struct Graph<'a, 'b> {
    writer: &'a mut BoundedText<'b>,
    cancel: &'a CancellationToken,
    nodes: HashMap<Key, usize>,
    parents: Vec<(usize, usize)>,
    // Parent, child ordinal, child. Subqueries precede ordinary inputs exactly as
    // in DataFusion's visit_with_subqueries / display_pg_json traversal.
    edges: Vec<[usize; 3]>,
    control_bytes: usize,
}
impl Graph<'_, '_> {
    fn charge(&mut self, bytes: usize) -> std::result::Result<(), CatalogError> {
        self.writer.reservation.try_grow(bytes)?;
        self.control_bytes += bytes;
        Ok(())
    }
    fn render(&mut self, plan: &LogicalPlan) -> std::result::Result<(), CatalogError> {
        self.writer
            .write_str("{\"Root\":0,\"Nodes\":[")
            .map_err(render_error)?;
        plan.visit_with_subqueries(self)
            .map_err(crate::session::engine)?;
        self.writer
            .write_str("],\"Edges\":[")
            .map_err(render_error)?;
        for (index, [parent, ordinal, child]) in self.edges.iter().enumerate() {
            self.cancel.checkpoint()?;
            if index != 0 {
                self.writer.write_char(',').map_err(render_error)?;
            }
            write!(self.writer, "[{parent},{ordinal},{child}]").map_err(render_error)?;
        }
        self.writer.write_str("]}").map_err(render_error)
    }
    fn node(&mut self, node: &LogicalPlan) -> std::result::Result<TreeNodeRecursion, CatalogError> {
        self.cancel.checkpoint()?;
        let key = Key::of(node);
        let existing = self.nodes.get(&key).copied();
        let id = existing.unwrap_or(self.nodes.len());
        self.charge(64)?;
        if let Some((parent, ordinal)) = self.parents.last_mut() {
            self.edges.push([*parent, *ordinal, id]);
            *ordinal += 1;
        }
        self.parents.push((id, 0));
        if existing.is_some() {
            return Ok(TreeNodeRecursion::Jump);
        }
        self.charge(128)?;
        self.nodes.insert(key, id);
        if id != 0 {
            self.writer.write_char(',').map_err(render_error)?;
        }
        write!(
            self.writer,
            "{{\"Id\":{id},\"Native\":{}}}",
            NativeNode(node)
        )
        .map_err(render_error)?;
        Ok(TreeNodeRecursion::Continue)
    }
}
impl<'n> TreeNodeVisitor<'n> for Graph<'_, '_> {
    type Node = LogicalPlan;
    fn f_down(&mut self, node: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        self.node(node)
            .map_err(|error| DataFusionError::External(Box::new(error)))
    }
    fn f_up(&mut self, _: &'n LogicalPlan) -> Result<TreeNodeRecursion> {
        self.parents.pop();
        Ok(TreeNodeRecursion::Continue)
    }
}
fn render_error(_: std::fmt::Error) -> CatalogError {
    invalid("native plan graph rendering failed")
}
