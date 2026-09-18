// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Demand traversal preserves conditional evaluation regions from the actual syntax graph.

use super::{invalid, lower::Lowered};
use crate::{CompilerError, mathir_relations::syntax};
use pse_ids::{CancellationToken, Reservation, SemanticId};
use pse_mathir::{GuardRef, NodeId, Payload, ValueRef};
use pse_relations::generated::normalized::predicate_nodes;
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Task {
    Math(NodeId),
    Predicate(i64),
    Equation(i64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Read {
    Symbol { symbol: SemanticId, node: i64 },
    Path { path: i64, node: i64 },
}
pub(super) fn collect(
    lowered: &mut Lowered,
    source: SemanticId,
    offset: i64,
    cancel: &CancellationToken,
    work: &mut dyn Reservation,
) -> Result<BTreeSet<(Read, Option<i64>)>, CompilerError> {
    work.try_grow(4096).map_err(pse_ids::CanonError::from)?;
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
        seen: BTreeSet::new(),
        synthesized: std::collections::BTreeMap::new(),
        work,
    };
    while let Some((task, guard)) = walker.pending.pop() {
        walker.cancel.checkpoint()?;
        if walker.seen.contains(&(task, guard)) {
            continue;
        }
        walker
            .work
            .try_grow(4096)
            .map_err(pse_ids::CanonError::from)?;
        walker.seen.insert((task, guard));
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
    offset: i64,
    cancel: &'a CancellationToken,
    pending: Vec<(Task, Option<i64>)>,
    demands: BTreeSet<(Read, Option<i64>)>,
    seen: BTreeSet<(Task, Option<i64>)>,
    synthesized: std::collections::BTreeMap<(&'static str, i64, Option<i64>), i64>,
    work: &'a mut dyn Reservation,
}
fn ordinal(value: Option<i64>) -> Result<i64, CompilerError> {
    value.ok_or_else(|| invalid("missing demand graph ordinal"))
}
fn local(id: i64, offset: i64) -> Result<NodeId, CompilerError> {
    id.checked_sub(offset)
        .map(NodeId)
        .ok_or_else(|| invalid("demand node is outside source family"))
}
impl Walker<'_> {
    fn push_math(&mut self, value: i64, guard: Option<i64>) -> Result<(), CompilerError> {
        self.pending
            .push((Task::Math(local(value, self.offset)?), guard));
        Ok(())
    }
    fn predicate(&mut self, id: i64, guard: Option<i64>) -> Result<(), CompilerError> {
        let row = self
            .lowered
            .predicates
            .get(usize::try_from(id).map_err(|_| invalid("predicate ordinal overflow"))?)
            .cloned()
            .ok_or_else(|| invalid("demand predicate absent"))?;
        for expression in syntax::math_dependencies(&row.value)? {
            self.push_math(expression, guard)?;
        }
        for child in syntax::predicate_dependencies(&row.value)? {
            self.pending.push((Task::Predicate(child), guard));
        }
        Ok(())
    }
    fn equation(&mut self, id: i64, guard: Option<i64>) -> Result<(), CompilerError> {
        let row = self
            .lowered
            .equations
            .get(usize::try_from(id).map_err(|_| invalid("equation ordinal overflow"))?)
            .cloned()
            .ok_or_else(|| invalid("demand equation absent"))?;
        match row.value.selected()? {
            syntax::EquationSelected::Relation(value) => {
                self.push_math(value.left, guard)?;
                self.push_math(value.right, guard)?;
            }
            syntax::EquationSelected::Conditional(value) => {
                let condition = value.guard;
                self.pending.push((Task::Predicate(condition), guard));
                let yes = self.combine(guard, condition, false)?;
                let no = self.combine(guard, condition, true)?;
                self.pending.push((Task::Equation(value.then), Some(yes)));
                self.pending
                    .push((Task::Equation(value.otherwise), Some(no)));
            }
        }
        Ok(())
    }
    fn math(&mut self, id: NodeId, guard: Option<i64>) -> Result<(), CompilerError> {
        let children = self.lowered.graph.node(id)?.children.len();
        self.work
            .try_grow(
                children
                    .checked_mul(512)
                    .and_then(|bytes| bytes.checked_add(8192))
                    .ok_or_else(|| invalid("demand graph traversal extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        let node = self.lowered.graph.node(id)?.clone();
        match &node.payload {
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(symbol),
            }
            | Payload::Gather { group: symbol, .. }
            | Payload::PendingGather { group: symbol, .. } => {
                self.demands.insert((
                    Read::Symbol {
                        symbol: *symbol,
                        node: self.global(id)?,
                    },
                    guard,
                ));
            }
            Payload::PendingPath {
                source_id, path_id, ..
            } => {
                if *source_id != self.source {
                    return Err(invalid(
                        "path demand belongs to a different expression source",
                    ));
                }
                self.demands.insert((
                    Read::Path {
                        path: *path_id,
                        node: self.global(id)?,
                    },
                    guard,
                ));
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
    fn global(&self, id: NodeId) -> Result<i64, CompilerError> {
        id.0.checked_add(self.offset)
            .ok_or_else(|| invalid("read occurrence ordinal overflow"))
    }
    fn guard(&self, guard: &GuardRef) -> Result<i64, CompilerError> {
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
        outer: Option<i64>,
        inner: i64,
        negated: bool,
    ) -> Result<i64, CompilerError> {
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
        left: i64,
        right: Option<i64>,
    ) -> Result<i64, CompilerError> {
        if let Some(id) = self.synthesized.get(&(op, left, right)) {
            return Ok(*id);
        }
        self.cancel.checkpoint()?;
        self.work
            .try_grow(4096)
            .map_err(pse_ids::CanonError::from)?;
        let id = i64::try_from(self.lowered.predicates.len())
            .map_err(|_| invalid("predicate ordinal overflow"))?;
        let value = match op {
            "not" => syntax::Predicate::from_not(syntax::Not { predicate: left }),
            "and" => syntax::Predicate::from_and(syntax::And {
                left,
                right: ordinal(right)?,
            }),
            _ => return Err(invalid("unsupported synthesized guard operator")),
        };
        let row = predicate_nodes::Row {
            source_id: self.source,
            predicate_id: id,
            value,
        };
        self.lowered.predicates.push(row);
        self.synthesized.insert((op, left, right), id);
        Ok(id)
    }
}
