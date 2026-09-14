// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cross-graph semantic admission before normalized rows can become outputs.

use super::{invalid, lower::Lowered};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::{GuardRef, Payload, ValueRef};
use pse_schema::{Registry, model::Cell};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn lowered(
    value: &Lowered,
    source: SemanticId,
    offset: u64,
    registry: &Registry,
) -> Result<(), CompilerError> {
    let nodes = value
        .graph
        .iter()
        .map(|(id, _)| {
            id.0.checked_add(offset)
                .ok_or_else(|| invalid("node identity overflow"))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let predicates = index(&value.predicates, source)?;
    let equations = index(&value.equations, source)?;
    let mut bindings = BTreeSet::new();
    for row in &value.bindings {
        if row.len() != 7
            || row[0] != Cell::Id(source)
            || !matches!(row[1], Cell::Id(_))
            || !bindings.insert(row[1].literal_spec())
        {
            return Err(invalid("invalid or duplicate expression index binding"));
        }
        domain(&row[3..6])?;
    }
    for (id, row) in &predicates {
        predicate(row, *id, &nodes, &predicates, registry)?;
    }
    for (id, row) in &equations {
        equation(row, *id, &nodes, &predicates, &equations)?;
    }
    let root_exists = match value.syntax {
        "expression" => nodes.contains(&value.root),
        "predicate" => predicates.contains_key(&value.root),
        "equation" => equations.contains_key(&value.root),
        _ => false,
    };
    if !root_exists {
        return Err(invalid(
            "normalized syntax root is missing or has the wrong family",
        ));
    }
    let guard = |guard: &GuardRef| -> Result<(), CompilerError> {
        match guard {
            GuardRef::Math(node) if value.graph.node(*node).is_ok() => Ok(()),
            GuardRef::Predicate {
                source_id,
                predicate_id,
            } if *source_id == source && predicates.contains_key(predicate_id) => Ok(()),
            _ => Err(invalid(
                "normalized guard references a different source or missing predicate",
            )),
        }
    };
    for (_, node) in value.graph.iter() {
        match &node.payload {
            Payload::Gather { coordinate_map, .. } => {
                let mut positions = BTreeSet::new();
                for (index, position) in coordinate_map {
                    if !bindings.contains(&Cell::Id(index.as_id()).literal_spec())
                        || !positions.insert(position)
                    {
                        return Err(invalid(
                            "Gather coordinate lacks a unique declared lexical binding",
                        ));
                    }
                }
            }
            Payload::PendingGather { indices, .. } => {
                for index in indices {
                    value.graph.node(*index)?;
                }
            }
            Payload::SymbolRef {
                symbol: ValueRef::Index(index),
            } if !bindings.contains(&Cell::Id(index.as_id()).literal_spec()) => {
                return Err(invalid("index value has no exact lexical declaration"));
            }
            Payload::Reduction {
                bound_index,
                filter,
                ..
            }
            | Payload::Integral {
                bound_index,
                filter,
                ..
            } => {
                if !bindings.contains(&Cell::Id(bound_index.as_id()).literal_spec()) {
                    return Err(invalid("reduction binder is undeclared"));
                }
                if let Some(filter) = filter {
                    guard(filter)?;
                }
            }
            Payload::Conditional {
                guard: predicate, ..
            } => guard(predicate)?,
            _ => {}
        }
    }
    Ok(())
}
fn index(rows: &[Vec<Cell>], source: SemanticId) -> Result<BTreeMap<u64, &[Cell]>, CompilerError> {
    let mut result = BTreeMap::new();
    for row in rows {
        let Some(Cell::U64(id)) = row.get(1) else {
            return Err(invalid("syntax node ordinal is absent"));
        };
        if row.first() != Some(&Cell::Id(source)) || result.insert(*id, row.as_slice()).is_some() {
            return Err(invalid("syntax node has another source or duplicate key"));
        }
    }
    Ok(result)
}
fn reference(value: &Cell, targets: &BTreeSet<u64>) -> Result<(), CompilerError> {
    if let Cell::U64(id) = value
        && targets.contains(id)
    {
        return Ok(());
    }
    Err(invalid("dangling normalized math reference"))
}
fn child(value: &Cell, parent: u64, targets: &BTreeMap<u64, &[Cell]>) -> Result<(), CompilerError> {
    // Syntax ordinals are assigned after every child. This direct ordering check proves
    // acyclicity while independently checking each referenced complete key.
    if let Cell::U64(id) = value
        && *id < parent
        && targets.contains_key(id)
    {
        return Ok(());
    }
    Err(invalid("dangling or cyclic normalized syntax branch"))
}
fn slots(row: &[Cell], present: &[usize]) -> Result<(), CompilerError> {
    for (index, cell) in row.iter().enumerate().skip(3) {
        if present.contains(&index) == matches!(cell, Cell::Null) {
            return Err(invalid(
                "normalized syntax payload has a missing or extraneous field",
            ));
        }
    }
    Ok(())
}
fn predicate(
    row: &[Cell],
    id: u64,
    nodes: &BTreeSet<u64>,
    predicates: &BTreeMap<u64, &[Cell]>,
    registry: &Registry,
) -> Result<(), CompilerError> {
    if row.len() != 18 {
        return Err(invalid("predicate row width differs"));
    }
    match row[2] {
        Cell::Enum("boolean") => {
            slots(row, &[3])?;
            if !matches!(row[3], Cell::Bool(_)) {
                return Err(invalid("predicate Boolean value required"));
            }
        }
        Cell::Enum("null") => slots(row, &[])?,
        Cell::Enum("atom") => {
            slots(row, &[5, 12])?;
            if row[12] != Cell::Enum("expression") {
                return Err(invalid("predicate atom requires expression operand"));
            }
            reference(&row[5], nodes)?;
        }
        Cell::Enum("compare") => {
            let mut present = vec![4];
            present.extend(operand(row, true, nodes, registry)?);
            present.extend(operand(row, false, nodes, registry)?);
            slots(row, &present)?;
            if !matches!(
                row[4],
                Cell::Enum("eq" | "not_eq" | "lt" | "le" | "gt" | "ge")
            ) {
                return Err(invalid("comparison meaning absent"));
            }
            if (row[12] == Cell::Enum("enum_literal") || row[15] == Cell::Enum("enum_literal"))
                && !matches!(row[4], Cell::Enum("eq" | "not_eq"))
            {
                return Err(invalid("enum ordering is not declared"));
            }
        }
        Cell::Enum("in") => {
            let fields = if row[9] == Cell::Null {
                vec![5, 10, 11, 12]
            } else {
                vec![5, 9, 12]
            };
            slots(row, &fields)?;
            if row[12] != Cell::Enum("expression") {
                return Err(invalid("domain membership requires expression operand"));
            }
            domain(&row[9..12])?;
            reference(&row[5], nodes)?;
        }
        Cell::Enum("and" | "or") => {
            slots(row, &[7, 8])?;
            child(&row[7], id, predicates)?;
            child(&row[8], id, predicates)?;
        }
        Cell::Enum("not") => {
            slots(row, &[7])?;
            child(&row[7], id, predicates)?;
        }
        _ => return Err(invalid("unknown predicate operator")),
    }
    Ok(())
}
fn operand(
    row: &[Cell],
    left: bool,
    nodes: &BTreeSet<u64>,
    registry: &Registry,
) -> Result<Vec<usize>, CompilerError> {
    let (value, kind, identity, member) = if left {
        (5, 12, 13, 14)
    } else {
        (6, 15, 16, 17)
    };
    match row[kind] {
        Cell::Enum("expression") => {
            reference(&row[value], nodes)?;
            Ok(vec![value, kind])
        }
        Cell::Enum("enum_literal") => {
            let (Cell::Id(id), Cell::Text(spelling)) = (&row[identity], &row[member]) else {
                return Err(invalid("enum literal needs exact enum identity and member"));
            };
            if !registry.enums().iter().any(|definition| {
                definition.id == *id
                    && definition
                        .members
                        .iter()
                        .any(|member| member.name == spelling.as_str())
            }) {
                return Err(invalid(
                    "enum literal is not in its exact registered member set",
                ));
            }
            Ok(vec![kind, identity, member])
        }
        _ => Err(invalid("predicate operand has no declared alternative")),
    }
}
fn equation(
    row: &[Cell],
    id: u64,
    nodes: &BTreeSet<u64>,
    predicates: &BTreeMap<u64, &[Cell]>,
    equations: &BTreeMap<u64, &[Cell]>,
) -> Result<(), CompilerError> {
    if row.len() != 9 {
        return Err(invalid("equation row width differs"));
    }
    match row[2] {
        Cell::Enum("relation") => {
            slots(row, &[3, 4, 5])?;
            if !matches!(row[3], Cell::Enum("eq" | "le" | "ge")) {
                return Err(invalid("equation sense absent"));
            }
            reference(&row[4], nodes)?;
            reference(&row[5], nodes)?;
        }
        Cell::Enum("conditional") => {
            slots(row, &[6, 7, 8])?;
            if !matches!(&row[6],Cell::U64(id) if predicates.contains_key(id)) {
                return Err(invalid("equation guard missing"));
            }
            child(&row[7], id, equations)?;
            child(&row[8], id, equations)?;
        }
        _ => return Err(invalid("unknown equation syntax")),
    }
    Ok(())
}
fn domain(values: &[Cell]) -> Result<(), CompilerError> {
    if matches!(
        values,
        [Cell::Id(_), Cell::Null, Cell::Null] | [Cell::Null, Cell::Id(_), Cell::Text(_)]
    ) {
        Ok(())
    } else {
        Err(invalid(
            "domain reference alternatives are incomplete or overlap",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, Lowered, SemanticId, lowered};
    use pse_mathir::ExprGraph;
    #[test]
    #[allow(clippy::unwrap_used, reason = "fixture assertions")]
    fn cross_graph_admission_rejects_cycles_other_sources_and_dangling_math() {
        let registry = pse_schema::registry().unwrap();
        let source = SemanticId::from_bytes([1; 16]);
        let mut graph = ExprGraph::new();
        graph.int_const(1).unwrap();
        let row = vec![
            Cell::Id(source),
            Cell::U64(0),
            Cell::Enum("atom"),
            Cell::Null,
            Cell::Null,
            Cell::U64(0),
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Enum("expression"),
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
            Cell::Null,
        ];
        let mut value = Lowered {
            graph,
            root: 0,
            syntax: "predicate",
            predicates: vec![row.clone()],
            equations: vec![],
            bindings: vec![],
        };
        assert!(lowered(&value, source, 0, registry).is_ok());
        value.predicates[0][5] = Cell::U64(99);
        assert!(lowered(&value, source, 0, registry).is_err());
        value.predicates[0] = row.clone();
        value.predicates[0][0] = Cell::Id(SemanticId::NIL);
        assert!(lowered(&value, source, 0, registry).is_err());
        value.predicates[0] = row;
        value.predicates[0][2] = Cell::Enum("not");
        value.predicates[0][5] = Cell::Null;
        value.predicates[0][12] = Cell::Null;
        value.predicates[0][7] = Cell::U64(0);
        assert!(lowered(&value, source, 0, registry).is_err());
    }
}
