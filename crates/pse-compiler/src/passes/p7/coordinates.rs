// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite subscripts use the sole P4 scalar evaluator and P5 valid products.
use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::{NodeId, Payload, ValueRef, relations::LoadedMath};
use pse_quantity::BoundIndexId;
use pse_relations::generated::{compiled, normalized};
use pse_templates::{
    EvaluatedGather, GroupBinding, InstantiationEnvironment, PathBinding, paths::Coordinate,
};
use std::collections::{BTreeMap, BTreeSet};

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "bind_coordinates keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn bind_coordinates(
        &mut self,
        source: &normalized::expression_sources::Row,
        roots: &[NodeId],
        env: &mut InstantiationEnvironment,
    ) -> Result<(), CompilerError> {
        let family = self
            .families
            .get(source.family.as_str())
            .ok_or_else(|| invalid("coordinate source family missing"))?;
        let nodes = pse_templates::coordinate_nodes(family, roots, env, self.cancel)?;
        for node in nodes {
            let family = self
                .families
                .get(source.family.as_str())
                .ok_or_else(|| invalid("coordinate family absent"))?;
            let (group, indices) = match &family.graph.node(node)?.payload {
                Payload::PendingGather { group, indices } => (env.groups.get(group), indices),
                Payload::PendingPath {
                    source_id,
                    path_id,
                    indices,
                } => (
                    match env.paths.get(&(*source_id, *path_id)) {
                        Some(PathBinding::Group(group)) => Some(group),
                        _ => None,
                    },
                    indices,
                ),
                _ => return Err(invalid("coordinate traversal returned another payload")),
            };
            let Some(group) = group else {
                continue;
            };
            if indices.iter().all(|id| {
                family.graph.node(*id).is_ok_and(|node| {
                    matches!(
                        node.payload,
                        Payload::IntConst { .. }
                            | Payload::SymbolRef {
                                symbol: ValueRef::Index(_)
                            }
                    )
                })
            }) {
                continue;
            }
            let group = group.clone();
            if group.axes.len() != indices.len() {
                return Err(invalid("arithmetic subscript arity differs"));
            }
            let used = coordinate_binders(family, indices, self.cancel)?;
            let coordinates = indices
                .iter()
                .map(|id| original(family, *id))
                .collect::<Result<Vec<_>, _>>()?;
            let original_node = original(family, node)?;
            let mut free = self
                .inventory
                .indices
                .iter()
                .filter(|row| {
                    row.source_id == source.source_id
                        && used.contains(&row.bound_index_id)
                        && !env
                            .fixed_indices
                            .contains_key(&BoundIndexId::from_id(row.bound_index_id))
                })
                .cloned()
                .collect::<Vec<_>>();
            free.sort_by_key(|row| (row.position.is_none(), row.position, row.bound_index_id));
            let binders = free
                .iter()
                .map(|row| BoundIndexId::from_id(row.bound_index_id))
                .collect::<Vec<_>>();
            if used
                .iter()
                .any(|id| !env.binders.contains_key(&BoundIndexId::from_id(*id)))
            {
                return Err(invalid("coordinate uses an undeclared source binder"));
            }
            let axes = binders
                .iter()
                .map(|id| {
                    env.binders
                        .get(id)
                        .map(|bound| bound.domain)
                        .ok_or_else(|| invalid("coordinate binder domain absent"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let product = self.product(&axes)?;
            let fixed = env
                .fixed_indices
                .iter()
                .map(|(id, member)| (id.as_id(), *member))
                .collect::<BTreeMap<_, _>>();
            let id = pse_templates::identity::reindexed_group_id(
                env.instance,
                source.source_id,
                original_node.0,
                &fixed,
            );
            let tuples = self
                .inventory
                .tuples
                .iter()
                .filter(|row| row.product_id == product)
                .map(|row| row.tuple.clone())
                .collect::<Vec<_>>();
            let bytes = tuples
                .len()
                .checked_mul(
                    (binders.len() + group.axes.len() + fixed.len() + 1)
                        .checked_mul(256)
                        .ok_or_else(|| invalid("coordinate workspace overflow"))?,
                )
                .ok_or_else(|| invalid("coordinate workspace overflow"))?;
            self.work
                .try_grow(bytes)
                .map_err(pse_ids::CanonError::from)?;
            let evaluator = self
                .coordinate_evaluator
                .as_ref()
                .ok_or_else(|| invalid("coordinate evaluator absent"))?;
            evaluator.validate_bindings(env.instance, source.source_id, &fixed)?;
            for (binder, domain) in binders.iter().zip(&axes) {
                if evaluator.binding_domain(env.instance, source.source_id, binder.as_id())?
                    != domain.as_id()
                {
                    return Err(invalid(
                        "arithmetic free axis differs from actual source binder domain",
                    ));
                }
            }
            let mut members = BTreeMap::new();
            let mut correspondence = Vec::new();
            for tuple in tuples {
                self.cancel.checkpoint()?;
                if tuple.len() != binders.len() {
                    return Err(invalid("arithmetic free tuple arity differs"));
                }
                let mut bindings = fixed.clone();
                bindings.extend(
                    binders
                        .iter()
                        .zip(&tuple)
                        .map(|(binder, member)| (binder.as_id(), *member)),
                );
                let evaluator = self
                    .coordinate_evaluator
                    .as_ref()
                    .ok_or_else(|| invalid("coordinate evaluator absent"))?;
                let source_index = coordinates
                    .iter()
                    .zip(&group.axes)
                    .map(|(coordinate, axis)| {
                        match evaluator.evaluate(
                            env.instance,
                            source.source_id,
                            *coordinate,
                            &bindings,
                        )? {
                            Coordinate::Member(member) => Ok(member),
                            Coordinate::Integer(value) => env
                                .integer_members
                                .get(&(*axis, value))
                                .copied()
                                .ok_or_else(|| {
                                    invalid("arithmetic coordinate has no exact actual member")
                                }),
                        }
                    })
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                let symbol = *group.members.get(&source_index).ok_or_else(|| {
                    invalid("arithmetic coordinate resolves outside the exact source group")
                })?;
                if members.insert(tuple.clone(), symbol).is_some() {
                    return Err(invalid("repeated P5-valid coordinate"));
                }
                correspondence.push((tuple, source_index, symbol));
            }
            self.emit_reindexing(
                env.instance,
                source.source_id,
                original_node,
                group.group,
                id,
                product,
                &binders,
                &fixed,
                &correspondence,
            )?;
            env.evaluated_gathers.insert(
                node,
                EvaluatedGather {
                    group: GroupBinding {
                        group: id,
                        axes,
                        members,
                    },
                    coordinates: binders,
                },
            );
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "emit_reindexing keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    fn emit_reindexing(
        &mut self,
        instance: SemanticId,
        source: SemanticId,
        node: NodeId,
        original_group: SemanticId,
        group: SemanticId,
        product: SemanticId,
        binders: &[BoundIndexId],
        fixed: &BTreeMap<SemanticId, SemanticId>,
        members: &[(Vec<SemanticId>, Vec<SemanticId>, SemanticId)],
    ) -> Result<(), CompilerError> {
        self.append(
            "compiled.symbol_groups",
            compiled::symbol_groups::Row {
                group_id: group,
                owner_instance_id: instance,
                name: format!("reindex:{}:{}", source.to_hex(), node.0),
                product_id: product,
            },
        )?;
        self.append(
            "compiled.group_reindexings",
            compiled::group_reindexings::Row {
                group_id: group,
                source_group_id: original_group,
                instance_id: instance,
                source_id: source,
                source_node_id: node.0,
                product_id: product,
                bound_indices: binders.iter().map(|id| id.as_id()).collect(),
                fixed_indices: fixed
                    .iter()
                    .map(|(id, member)| {
                        compiled::group_reindexings::CompiledGroupReindexingsFieldFixedIndicesItem {
                            bound_index_id: *id,
                            member_id: *member,
                        }
                    })
                    .collect(),
                derivation_id: Self::derivation(instance, source, "reindexing"),
            },
        )?;
        for (index, source_index, symbol) in members {
            self.append(
                "compiled.symbol_group_members",
                compiled::symbol_group_members::Row {
                    group_id: group,
                    tuple: index.clone(),
                    symbol_id: *symbol,
                },
            )?;
            self.append(
                "compiled.group_reindexing_members",
                compiled::group_reindexing_members::Row {
                    group_id: group,
                    index: index.clone(),
                    source_index: source_index.clone(),
                    symbol_id: *symbol,
                    derivation_id: Self::derivation(instance, source, "reindexing_member"),
                },
            )?;
        }
        Ok(())
    }
}

fn original(family: &LoadedMath, node: NodeId) -> Result<NodeId, CompilerError> {
    family
        .node_mapping
        .iter()
        .find_map(|(old, actual)| (*actual == node).then_some(*old))
        .ok_or_else(|| invalid("coordinate node has no original source row"))
}
fn coordinate_binders(
    family: &LoadedMath,
    roots: &[NodeId],
    cancel: &pse_ids::CancellationToken,
) -> Result<BTreeSet<SemanticId>, CompilerError> {
    let mut seen = BTreeSet::new();
    let mut pending = roots.to_vec();
    let mut result = BTreeSet::new();
    while let Some(id) = pending.pop() {
        cancel.checkpoint()?;
        if !seen.insert(id) {
            continue;
        }
        let node = family.graph.node(id)?;
        if let Payload::SymbolRef {
            symbol: ValueRef::Index(index),
        } = node.payload
        {
            result.insert(index.as_id());
        }
        pending.extend(node.children.iter().copied());
        pending.extend(node.payload.referenced_nodes());
    }
    Ok(result)
}
