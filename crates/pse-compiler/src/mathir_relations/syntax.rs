// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed generated source values and their ordered algorithm dependencies.
use crate::CompilerError;

pub(crate) use pse_relations::generated::normalized::{
    equation_nodes::{
        NormalizedEquationNodesFieldValue as Equation,
        NormalizedEquationNodesFieldValueConditional as Conditional,
        NormalizedEquationNodesFieldValueRelation as Relation,
        NormalizedEquationNodesFieldValueSelected as EquationSelected,
    },
    predicate_nodes::{
        NormalizedPredicateNodesFieldValue as Predicate,
        NormalizedPredicateNodesFieldValueAnd as And,
        NormalizedPredicateNodesFieldValueAtom as Atom,
        NormalizedPredicateNodesFieldValueBoolean as Boolean,
        NormalizedPredicateNodesFieldValueCompare as Compare,
        NormalizedPredicateNodesFieldValueCompareOperandsItem as Operand,
        NormalizedPredicateNodesFieldValueCompareOperandsItemEnumLiteral as EnumLiteral,
        NormalizedPredicateNodesFieldValueCompareOperandsItemExpression as Expression,
        NormalizedPredicateNodesFieldValueCompareOperandsItemSelected as OperandSelected,
        NormalizedPredicateNodesFieldValueIn as Membership,
        NormalizedPredicateNodesFieldValueNot as Not, NormalizedPredicateNodesFieldValueOr as Or,
        NormalizedPredicateNodesFieldValueSelected as PredicateSelected,
    },
};

pub(crate) fn math_dependencies(value: &Predicate) -> Result<Vec<i64>, CompilerError> {
    Ok(match value.selected()? {
        PredicateSelected::Atom(value) => vec![value.expression],
        PredicateSelected::In(value) => vec![value.expression],
        PredicateSelected::Compare(value) => {
            let mut nodes = Vec::with_capacity(2);
            for operand in &value.operands {
                if let OperandSelected::Expression(value) = operand.selected()? {
                    nodes.push(value.node_id);
                }
            }
            nodes
        }
        PredicateSelected::Boolean(_)
        | PredicateSelected::Null
        | PredicateSelected::And(_)
        | PredicateSelected::Or(_)
        | PredicateSelected::Not(_) => vec![],
    })
}

pub(crate) fn predicate_dependencies(value: &Predicate) -> Result<Vec<i64>, CompilerError> {
    Ok(match value.selected()? {
        PredicateSelected::And(value) => vec![value.left, value.right],
        PredicateSelected::Or(value) => vec![value.left, value.right],
        PredicateSelected::Not(value) => vec![value.predicate],
        PredicateSelected::Boolean(_)
        | PredicateSelected::Null
        | PredicateSelected::Atom(_)
        | PredicateSelected::Compare(_)
        | PredicateSelected::In(_) => vec![],
    })
}
