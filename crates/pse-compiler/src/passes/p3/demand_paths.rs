// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Demand traversal preserves conditional evaluation regions from the actual syntax graph.

use super::{invalid, lower::Lowered};
use crate::CompilerError;
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{GuardRef, NodeId, Payload, ValueRef};
use pse_schema::model::Cell;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum Task {
    Math(NodeId),
    Predicate(u64),
    Equation(u64),
}
pub(super) fn collect(
    lowered: &mut Lowered,
    source: SemanticId,
    offset: u64,
    cancel: &CancellationToken,
) -> Result<BTreeSet<(SemanticId, Option<u64>)>, CompilerError> {
    let root = match lowered.syntax {
        "expression" => Task::Math(local(lowered.root, offset)?),
        "predicate" => Task::Predicate(lowered.root),
        "equation" => Task::Equation(lowered.root),
        _ => return Err(invalid("unknown demand source syntax")),
    };
    let mut walker = Walker {
        lowered,
        source,
        offset,
        cancel,
        pending: vec![(root, None)],
        demands: BTreeSet::new(),
        steps: 0,
    };
    while let Some((task, guard)) = walker.pending.pop() {
        walker.cancel.checkpoint()?;
        walker.steps += 1;
        if walker.steps > 65_536 {
            return Err(invalid(
                "conditional demand traversal exceeds finite budget",
            ));
        }
        match task {
            Task::Math(node) => walker.math(node, guard)?,
            Task::Predicate(node) => walker.predicate(node, guard)?,
            Task::Equation(node) => walker.equation(node, guard)?,
        }
    }
    Ok(walker.demands)
}
struct Walker<'a> {
    lowered: &'a mut Lowered,
    source: SemanticId,
    offset: u64,
    cancel: &'a CancellationToken,
    pending: Vec<(Task, Option<u64>)>,
    demands: BTreeSet<(SemanticId, Option<u64>)>,
    steps: usize,
}
fn ordinal(cell: &Cell) -> Result<u64, CompilerError> {
    if let Cell::U64(id) = cell {
        Ok(*id)
    } else {
        Err(invalid("missing demand graph ordinal"))
    }
}
fn local(id: u64, offset: u64) -> Result<NodeId, CompilerError> {
    id.checked_sub(offset)
        .map(NodeId)
        .ok_or_else(|| invalid("demand node is outside source family"))
}
impl Walker<'_> {
    fn push_math(&mut self, cell: &Cell, guard: Option<u64>) -> Result<(), CompilerError> {
        self.pending
            .push((Task::Math(local(ordinal(cell)?, self.offset)?), guard));
        Ok(())
    }
    fn predicate(&mut self, id: u64, guard: Option<u64>) -> Result<(), CompilerError> {
        let row = self
            .lowered
            .predicates
            .get(usize::try_from(id).map_err(|_| invalid("predicate ordinal overflow"))?)
            .cloned()
            .ok_or_else(|| invalid("demand predicate absent"))?;
        match row[2] {
            Cell::Enum("atom" | "in") => self.push_math(&row[5], guard)?,
            Cell::Enum("compare") => {
                if row[12] == Cell::Enum("expression") {
                    self.push_math(&row[5], guard)?;
                }
                if row[15] == Cell::Enum("expression") {
                    self.push_math(&row[6], guard)?;
                }
            }
            Cell::Enum("not") => self
                .pending
                .push((Task::Predicate(ordinal(&row[7])?), guard)),
            Cell::Enum("and" | "or") => {
                self.pending
                    .push((Task::Predicate(ordinal(&row[7])?), guard));
                self.pending
                    .push((Task::Predicate(ordinal(&row[8])?), guard));
            }
            Cell::Enum("boolean" | "null") => {}
            _ => return Err(invalid("unknown demand predicate")),
        }
        Ok(())
    }
    fn equation(&mut self, id: u64, guard: Option<u64>) -> Result<(), CompilerError> {
        let row = self
            .lowered
            .equations
            .get(usize::try_from(id).map_err(|_| invalid("equation ordinal overflow"))?)
            .cloned()
            .ok_or_else(|| invalid("demand equation absent"))?;
        match row[2] {
            Cell::Enum("relation") => {
                self.push_math(&row[4], guard)?;
                self.push_math(&row[5], guard)?;
            }
            Cell::Enum("conditional") => {
                let condition = ordinal(&row[6])?;
                self.pending.push((Task::Predicate(condition), guard));
                let yes = self.combine(guard, condition, false)?;
                let no = self.combine(guard, condition, true)?;
                self.pending
                    .push((Task::Equation(ordinal(&row[7])?), Some(yes)));
                self.pending
                    .push((Task::Equation(ordinal(&row[8])?), Some(no)));
            }
            _ => return Err(invalid("unknown demand equation")),
        }
        Ok(())
    }
    fn math(&mut self, id: NodeId, guard: Option<u64>) -> Result<(), CompilerError> {
        let node = self.lowered.graph.node(id)?.clone();
        match &node.payload {
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(symbol),
            }
            | Payload::Gather { group: symbol, .. }
            | Payload::PendingGather { group: symbol, .. } => {
                self.demands.insert((*symbol, guard));
            }
            Payload::Conditional { guard: condition } => {
                let condition = self.guard(condition)?;
                self.pending.push((Task::Predicate(condition), guard));
                let [yes, no] = node.children.as_slice() else {
                    return Err(invalid("conditional demand branch count differs"));
                };
                let yes_guard = self.combine(guard, condition, false)?;
                let no_guard = self.combine(guard, condition, true)?;
                self.pending.push((Task::Math(*yes), Some(yes_guard)));
                self.pending.push((Task::Math(*no), Some(no_guard)));
                return Ok(());
            }
            Payload::Reduction {
                filter: Some(condition),
                ..
            }
            | Payload::Integral {
                filter: Some(condition),
                ..
            } => {
                let condition = self.guard(condition)?;
                self.pending.push((Task::Predicate(condition), guard));
                let body_guard = self.combine(guard, condition, false)?;
                for child in &node.children {
                    self.pending.push((Task::Math(*child), Some(body_guard)));
                }
                return Ok(());
            }
            _ => {}
        }
        for child in node
            .children
            .into_iter()
            .chain(node.payload.referenced_nodes())
        {
            self.pending.push((Task::Math(child), guard));
        }
        Ok(())
    }
    fn guard(&self, guard: &GuardRef) -> Result<u64, CompilerError> {
        match guard {
            GuardRef::Predicate {
                source_id,
                predicate_id,
            } if *source_id == self.source => Ok(*predicate_id),
            _ => Err(invalid(
                "demand guard has no complete normalized predicate key",
            )),
        }
    }
    fn combine(
        &mut self,
        outer: Option<u64>,
        inner: u64,
        negated: bool,
    ) -> Result<u64, CompilerError> {
        let inner = if negated {
            self.insert("not", inner, None)?
        } else {
            inner
        };
        outer.map_or(Ok(inner), |outer| self.insert("and", outer, Some(inner)))
    }
    fn insert(
        &mut self,
        op: &'static str,
        left: u64,
        right: Option<u64>,
    ) -> Result<u64, CompilerError> {
        if self.lowered.predicates.len() >= 65_536 {
            return Err(invalid("demand predicate budget exceeded"));
        }
        let id = u64::try_from(self.lowered.predicates.len())
            .map_err(|_| invalid("predicate ordinal overflow"))?;
        let mut row = vec![Cell::Null; 18];
        row[0] = Cell::Id(self.source);
        row[1] = Cell::U64(id);
        row[2] = Cell::Enum(op);
        row[7] = Cell::U64(left);
        row[8] = right.map_or(Cell::Null, Cell::U64);
        self.lowered.predicates.push(row);
        Ok(id)
    }
}
