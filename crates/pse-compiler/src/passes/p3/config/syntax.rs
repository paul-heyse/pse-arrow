// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrow source syntax by its actual declaration key; never parse a second DSL.

use super::{invalid, native::Source};
use crate::{CompilerError, passes::native_rows::AlgorithmInputs};
use pse_authoring::{
    document::{OwnedDocumentSet, binding::ParsedExpression},
    dsl::{ExprKind, PathSegment},
};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{columnar::RelationRow, generated::authored::template_submodels};
use pse_schema::{Registry, model::RelationSpec};
use std::{borrow::Cow, collections::BTreeMap};

type Key<'a> = (SemanticId, &'a str, usize);
type Syntax<'a> = (&'a str, &'a ParsedExpression);

pub(super) struct ConfigurationSyntax<'a> {
    fields: BTreeMap<Key<'a>, Syntax<'a>>,
}

pub(super) enum Binding<'a> {
    Parent(&'a str),
    Literal(Cow<'a, str>),
}

impl<'a> ConfigurationSyntax<'a> {
    pub(super) fn new(
        documents: &'a OwnedDocumentSet,
        registry: &'a Registry,
        arguments: &mut AlgorithmInputs,
        cancel: &CancellationToken,
    ) -> Result<Self, CompilerError> {
        documents.validate_registry(registry)?;
        let relation = template_submodels::Row::relation(registry)?;
        let mut fields = BTreeMap::new();
        for document in documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
        {
            let Some(batch) = document.batches.get(&relation.id) else {
                continue;
            };
            let view = template_submodels::View::from_checked(batch)?;
            reserve_fields(&view, relation, arguments)?;
            for row in 0..view.len() {
                cancel.checkpoint()?;
                let id = SemanticId::from_bytes(
                    view.template_id_column()
                        .value(row)
                        .try_into()
                        .map_err(|_| invalid("configuration template identity width"))?,
                );
                let offsets = view.bindings_column().value_offsets();
                let count = usize::try_from(offsets[row + 1] - offsets[row])
                    .map_err(|_| invalid("configuration binding extent invalid"))?;
                for ordinal in 0..count {
                    let syntax = document
                        .parsed_expression(relation, row, &format!("bindings/{ordinal}/value"))
                        .ok_or_else(|| invalid("configuration binding has no cached expression"))?;
                    let key = (id, view.name_column().value(row), ordinal);
                    if fields.insert(key, syntax).is_some() {
                        return Err(invalid(
                            "configuration binding has multiple original sources",
                        ));
                    }
                }
            }
        }
        Ok(Self { fields })
    }

    pub(super) fn binding(
        &self,
        source: &Source<template_submodels::Row>,
        ordinal: usize,
    ) -> Result<Binding<'a>, CompilerError> {
        let assignment = source
            .row
            .bindings
            .get(ordinal)
            .ok_or_else(|| invalid("configuration binding ordinal absent"))?;
        let (text, parsed) = self
            .fields
            .get(&(source.row.template_id, source.row.name.as_str(), ordinal))
            .ok_or_else(|| invalid("configuration binding has no cached expression"))?;
        if *text != assignment.value {
            return Err(invalid(
                "configuration binding differs from its original parsed field",
            ));
        }
        binding(text, parsed)
    }
}

fn reserve_fields(
    view: &template_submodels::View<'_>,
    relation: &RelationSpec,
    arguments: &mut AlgorithmInputs,
) -> Result<(), CompilerError> {
    // A full B-tree node per borrowed field bounds underfilled nodes as well.
    let node = 11 * (size_of::<Key<'_>>() + size_of::<Syntax<'_>>()) + 16 * size_of::<usize>();
    let offsets = view.bindings_column().value_offsets();
    let count = usize::try_from(offsets[view.len()] - offsets[0])
        .map_err(|_| invalid("configuration syntax field extent invalid"))?;
    // Two transient paths use decimal row/binding ordinals and declared names.
    let paths =
        2 * (relation.key.name.len() + "//bindings//value".len() + 2 * usize::BITS as usize);
    arguments.reserve(
        count
            .checked_mul(node)
            .and_then(|bytes| bytes.checked_add(paths))
            .ok_or_else(|| invalid("configuration syntax index extent overflow"))?,
    )
}

fn binding<'a>(text: &'a str, parsed: &'a ParsedExpression) -> Result<Binding<'a>, CompilerError> {
    let ParsedExpression::Expr(expression) = parsed else {
        return Err(invalid("configuration binding requires expression syntax"));
    };
    match &expression.kind {
        ExprKind::Path(path) => match path.segments.as_slice() {
            [parent, member] if parent.name == "parent" && scalar(parent) && scalar(member) => {
                Ok(Binding::Parent(&member.name))
            }
            [literal] if scalar(literal) => Ok(Binding::Literal(Cow::Borrowed(&literal.name))),
            _ => Err(invalid(
                "configuration reference requires an unindexed parent member",
            )),
        },
        // Decode exact integer tokens at the declared scalar boundary. The AST's
        // approximate f64 value cannot preserve all i64/u64 configuration values.
        ExprKind::Number(number) if number.unit.is_none() => {
            Ok(Binding::Literal(Cow::Borrowed(text.trim())))
        }
        ExprKind::Neg(inner) if matches!(&inner.kind, ExprKind::Number(number) if number.unit.is_none()) =>
        {
            let token = text
                .get(inner.span.start as usize..inner.span.end as usize)
                .ok_or_else(|| invalid("configuration numeric source span is invalid"))?;
            Ok(Binding::Literal(Cow::Owned(format!("-{}", token.trim()))))
        }
        _ => Err(invalid(
            "configuration binding requires a scalar literal or parent reference",
        )),
    }
}

fn scalar(segment: &PathSegment) -> bool {
    segment.indices.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_uses_path_structure_and_exact_numeric_tokens() {
        let text = " (parent . count) ";
        let parsed = ParsedExpression::Expr(pse_authoring::dsl::parse_expr(text).unwrap());
        assert!(matches!(
            binding(text, &parsed).unwrap(),
            Binding::Parent("count")
        ));
        for text in ["18446744073709551615", "- 9223372036854775808"] {
            let parsed = ParsedExpression::Expr(pse_authoring::dsl::parse_expr(text).unwrap());
            let Binding::Literal(value) = binding(text, &parsed).unwrap() else {
                panic!("literal became a reference");
            };
            assert_eq!(value, text.replace(' ', ""));
        }
        for text in [
            "parent.count[0]",
            "parent[0].count",
            "parent.count + 1",
            "self.count",
        ] {
            let parsed = ParsedExpression::Expr(pse_authoring::dsl::parse_expr(text).unwrap());
            assert!(binding(text, &parsed).is_err(), "{text}");
        }
    }
}
