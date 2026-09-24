// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native single-node display over the common scope-qualified graph.
use super::{BoundedText, EngineError, invalid};
use datafusion::{
    common::tree_node::{TreeNodeRecursion, TreeNodeVisitor},
    logical_expr::{LogicalPlan, logical_plan::display::PgJsonVisitor},
};
use pse_columnar::CancellationToken;
use std::fmt::{Display, Write};
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
) -> Result<(), EngineError> {
    let mut bytes = 0;
    let result = (|| {
        let graph = crate::session::traversal::graph(
            [plan],
            crate::session::traversal::Purpose::Observation,
            cancel,
            |extent| {
                writer
                    .reservation
                    .try_grow(extent)
                    .map_err(|e| pse_columnar::external(EngineError::from(e)))?;
                bytes += extent;
                Ok(())
            },
            |_, _| Ok(TreeNodeRecursion::Continue),
        )
        .map_err(crate::session::engine)?;
        writer
            .write_str("{\"Root\":0,\"Nodes\":[")
            .map_err(render_error)?;
        for (id, node) in graph.nodes.iter().enumerate() {
            cancel.checkpoint()?;
            if id != 0 {
                writer.write_char(',').map_err(render_error)?;
            }
            write!(
                writer,
                "{{\"Id\":{id},\"Native\":{}}}",
                NativeNode(&node.plan)
            )
            .map_err(render_error)?;
        }
        let mut wrappers = std::collections::BTreeMap::new();
        for (parent, node) in graph.nodes.iter().enumerate() {
            let mut ordinal = 0;
            node.plan
                .apply_subqueries(|wrapper| {
                    let id = graph.nodes.len() + wrappers.len();
                    writer
                        .reservation
                        .try_grow(128)
                        .map_err(|error| pse_columnar::external(EngineError::from(error)))?;
                    bytes += 128;
                    wrappers.insert((parent, ordinal), id);
                    ordinal += 1;
                    write!(
                        writer,
                        ",{{\"Id\":{id},\"Native\":{}}}",
                        NativeNode(wrapper)
                    )
                    .map_err(|error| pse_columnar::external(render_error(error)))?;
                    Ok(TreeNodeRecursion::Continue)
                })
                .map_err(crate::session::engine)?;
        }
        writer.write_str("],\"Edges\":[").map_err(render_error)?;
        let mut first = true;
        for (parent, node) in graph.nodes.iter().enumerate() {
            for (ordinal, child) in node.children.iter().enumerate() {
                if !first {
                    writer.write_char(',').map_err(render_error)?;
                }
                first = false;
                if ordinal < node.subqueries {
                    let wrapper = wrappers[&(parent, ordinal)];
                    write!(
                        writer,
                        "[{parent},{ordinal},{wrapper}],[{wrapper},0,{child}]"
                    )
                    .map_err(render_error)?;
                } else {
                    write!(writer, "[{parent},{ordinal},{child}]").map_err(render_error)?;
                }
            }
        }
        writer.write_str("]}").map_err(render_error)
    })();
    writer.reservation.shrink(bytes);
    result
}
fn render_error(_: std::fmt::Error) -> EngineError {
    invalid("native plan graph rendering failed")
}
