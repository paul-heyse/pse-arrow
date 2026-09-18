// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed selector syntax folds; selector set execution remains native rule evaluation.
use super::invalid;
use crate::{
    CompilerError,
    passes::{
        native_outputs::{OutputRows, SourceKey, Sources, materialize},
        native_rows::AlgorithmInputs,
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{authored, normalized};
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[expect(
    clippy::too_many_lines,
    reason = "emit keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn emit(
    sources: &Sources,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let registry = session.registry();
    let mut arguments = AlgorithmInputs::new(session.reserver(), "P3:selector-syntax-inputs");
    let terms = super::native::rows::<authored::selector_terms::Row>(
        &mut arguments,
        sources,
        session,
        cancel,
    )
    .await?;
    let scopes =
        super::native::rows::<authored::scopes::Row>(&mut arguments, sources, session, cancel)
            .await?;
    let mut work = session.reserver().open("P3:selector-syntax-folds");
    work.try_grow(
        terms
            .len()
            .checked_add(scopes.len())
            .and_then(|count| count.checked_mul(4096))
            .ok_or_else(|| invalid("selector syntax capacity overflow"))?,
    )
    .map_err(pse_ids::CanonError::from)?;
    let inventory = terms
        .iter()
        .map(|(row, key)| (row.term_id, (row, key)))
        .collect::<BTreeMap<_, _>>();
    let mut children =
        BTreeMap::<SemanticId, Vec<(&authored::selector_terms::Row, &SourceKey)>>::new();
    for (row, key) in &terms {
        if let Some(parent) = row.parent_term_id {
            children.entry(parent).or_default().push((row, key));
        }
    }
    let mut output = OutputRows::new(registry, session.reserver(), cancel)?;
    output.ensure::<normalized::selector_nodes::Row>()?;
    output.ensure::<normalized::selector_roots::Row>()?;
    output.ensure::<normalized::selector_children::Row>()?;
    for (parent, children) in &mut children {
        children.sort_by_key(|(row, _)| row.ordinal);
        let (_, parent_key) = inventory
            .get(parent)
            .ok_or_else(|| invalid("selector parent absent"))?;
        for (position, (child, child_key)) in children.iter().enumerate() {
            let position =
                i64::try_from(position).map_err(|_| invalid("selector position exceeds UInt16"))?;
            output.push(
                normalized::selector_children::Row {
                    source_term_id: *parent,
                    position,
                    source_ordinal: child.ordinal,
                    child_term_id: child.term_id,
                    derivation_id: derivation(*parent),
                },
                &BTreeSet::from([(*parent_key).clone(), (*child_key).clone()]),
            )?;
        }
    }
    let mut reached = BTreeSet::new();
    for (scope, scope_key) in &scopes {
        let (root, root_key) = inventory
            .get(&scope.root_term_id)
            .ok_or_else(|| invalid("selector root absent"))?;
        if root.scope_id != scope.scope_id || root.parent_term_id.is_some() {
            return Err(invalid("selector syntax root ownership differs"));
        }
        output.push(
            normalized::selector_roots::Row {
                scope_decl_id: scope.scope_id,
                node_id: root.term_id,
                derivation_id: derivation(root.term_id),
            },
            &BTreeSet::from([scope_key.clone(), (*root_key).clone()]),
        )?;
        let mut stack = vec![root.term_id];
        while let Some(id) = stack.pop() {
            cancel.checkpoint()?;
            if !reached.insert(id) {
                return Err(invalid("selector syntax repeats or cycles"));
            }
            stack.extend(
                children
                    .get(&id)
                    .into_iter()
                    .flatten()
                    .map(|(row, _)| row.term_id),
            );
        }
    }
    if reached.len() != terms.len() {
        return Err(invalid("selector syntax has unreachable terms"));
    }
    for (row, key) in &terms {
        cancel.checkpoint()?;
        let op = row.op.as_str();
        let kids = children.get(&row.term_id).map_or(&[][..], Vec::as_slice);
        let entity = row.entity_id.is_some();
        let kind = row.entity_kind.is_some();
        let parameter = row.parameter_name.is_some();
        let valid = match op {
            "include" | "exclude" | "descendant_of" => entity && !kind && !parameter,
            "kind_is" => !entity && kind && !parameter,
            "instance_parameter" => {
                !entity
                    && !kind
                    && row
                        .parameter_name
                        .as_ref()
                        .is_some_and(|name| !name.is_empty())
            }
            "self" | "union" | "intersection" | "difference" => !entity && !kind && !parameter,
            "tagged_with" => {
                return Err(invalid(
                    "tagged selector has no admitted entity-tag contract",
                ));
            }
            _ => false,
        };
        if !valid || row.tag.is_some() {
            return Err(invalid("selector syntax payload differs from operator"));
        }
        let compound = matches!(op, "union" | "intersection" | "difference");
        if (!compound && !kids.is_empty()) || (op == "difference" && kids.len() != 2) {
            return Err(invalid("selector syntax operator arity differs"));
        }
        let support = std::iter::once(key.clone())
            .chain(kids.iter().map(|(_, key)| (*key).clone()))
            .collect();
        let mut emit =
            |id, op: &str, left, right, constant, position| -> Result<(), CompilerError> {
                output.push(
                    normalized::selector_nodes::Row {
                        node_id: id,
                        source_term_id: row.term_id,
                        scope_decl_id: row.scope_id,
                        op: op.parse()?,
                        left_node_id: left,
                        right_node_id: right,
                        entity_id: row.entity_id,
                        entity_kind: row.entity_kind,
                        parameter_name: row.parameter_name.clone(),
                        constant,
                        fold_position: position,
                        derivation_id: derivation(row.term_id),
                    },
                    &support,
                )
            };
        if !compound {
            emit(row.term_id, op, None, None, None, None)?;
        } else if kids.is_empty() {
            emit(
                row.term_id,
                "constant",
                None,
                None,
                Some(op == "intersection"),
                None,
            )?;
        } else if kids.len() == 1 {
            emit(
                row.term_id,
                "identity",
                Some(kids[0].0.term_id),
                None,
                None,
                None,
            )?;
        } else {
            let mut left = kids[0].0.term_id;
            for (position, (right, _)) in kids.iter().enumerate().skip(1) {
                let id = if position + 1 == kids.len() {
                    row.term_id
                } else {
                    pse_ids::named_id(row.term_id, &format!("selector-fold:{position}"))
                };
                let position = i64::try_from(position)
                    .map_err(|_| invalid("selector fold position exceeds UInt16"))?;
                emit(
                    id,
                    op,
                    Some(left),
                    Some(right.term_id),
                    None,
                    Some(position),
                )?;
                left = id;
            }
        }
    }
    materialize(output.finish()?, sources, pass, session, cancel).await
}
fn derivation(id: SemanticId) -> SemanticId {
    pse_ids::named_id(id, "pass:P3@1")
}
