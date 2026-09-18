// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid, resource};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::{NodeId, Payload, ValueRef};
use pse_quantity::{BoundIndexId, QuantityTypeId};
use pse_relations::generated::{
    authored, compiled,
    enums::{AliasKind, ExpressionRootRole, SymbolRole},
    inferred, normalized,
};
use pse_templates::InstantiationEnvironment;

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "expressions keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn expressions(&mut self) -> Result<(), CompilerError> {
        let mut instances = self.inventory.instances.clone();
        instances.sort_by_key(|row| row.instance_id);
        for instance in &instances {
            if !self.selected.contains(&instance.instance_id) {
                continue;
            }
            self.cancel.checkpoint()?;
            let symbols = self
                .inventory
                .symbols
                .iter()
                .filter(|row| row.template_id == instance.template_id)
                .cloned()
                .collect::<Vec<_>>();
            for declaration in symbols {
                if !self.guard_enabled(instance.instance_id, declaration.guard_id)? {
                    continue;
                }
                match declaration.role {
                    SymbolRole::Expression => {
                        let bodies = self
                            .inventory
                            .bodies
                            .iter()
                            .filter(|row| row.symbol_decl_id == declaration.symbol_decl_id)
                            .collect::<Vec<_>>();
                        let [body] = bodies.as_slice() else {
                            return Err(invalid(
                                "expression-role symbol requires exactly one actual body",
                            ));
                        };
                        if body.template_id != instance.template_id {
                            return Err(invalid("expression body owner differs"));
                        }
                        let source = self.source(
                            authored::template_symbol_expressions::RELATION_ID,
                            "symbol_decl_id",
                            declaration.symbol_decl_id,
                            "expression",
                        )?;
                        self.symbol_body(instance, &declaration, &source)?;
                    }
                    SymbolRole::Reference | SymbolRole::Derivative => {
                        if declaration.reference_to.is_none() {
                            return Err(invalid(
                                "reference/derivative declaration lacks an actual base expression",
                            ));
                        }
                        let source = self.source(
                            authored::template_symbols::RELATION_ID,
                            "symbol_decl_id",
                            declaration.symbol_decl_id,
                            "reference_to",
                        )?;
                        self.symbol_body(instance, &declaration, &source)?;
                    }
                    SymbolRole::Variable | SymbolRole::Parameter => {
                        if declaration.reference_to.is_some()
                            || self
                                .inventory
                                .bodies
                                .iter()
                                .any(|row| row.symbol_decl_id == declaration.symbol_decl_id)
                        {
                            return Err(invalid(
                                "ordinary symbol declaration carries an incompatible expression body",
                            ));
                        }
                    }
                }
            }
            for equation in self
                .inventory
                .equations
                .iter()
                .filter(|row| row.template_id == instance.template_id)
                .cloned()
                .collect::<Vec<_>>()
            {
                if self.guard_enabled(instance.instance_id, equation.guard_id)? {
                    let source = self.source(
                        authored::template_equations::RELATION_ID,
                        "equation_decl_id",
                        equation.equation_decl_id,
                        "expression",
                    )?;
                    self.equation(
                        instance,
                        equation.equation_decl_id,
                        &equation.name,
                        &equation.indexed_by,
                        equation.filter.is_some(),
                        &source,
                    )?;
                }
            }
            for equation in self
                .inventory
                .instance_equations
                .iter()
                .filter(|row| row.instance_id == instance.instance_id)
                .cloned()
                .collect::<Vec<_>>()
            {
                if self.guard_enabled(instance.instance_id, equation.guard_id)? {
                    let source = self.source(
                        authored::instance_equations::RELATION_ID,
                        "equation_decl_id",
                        equation.equation_decl_id,
                        "expression",
                    )?;
                    self.equation(
                        instance,
                        equation.equation_decl_id,
                        &equation.name,
                        &equation.indexed_by,
                        equation.filter.is_some(),
                        &source,
                    )?;
                }
            }
            self.other_roots(instance)?;
            self.contributions(instance)?;
        }
        Ok(())
    }
    fn symbol_body(
        &mut self,
        instance: &inferred::instances::Row,
        declaration: &normalized::template_symbols::Row,
        source: &normalized::expression_sources::Row,
    ) -> Result<(), CompilerError> {
        let symbols = self
            .symbols
            .iter()
            .filter(|((owner, decl, _), _)| {
                *owner == instance.instance_id && *decl == declaration.symbol_decl_id
            })
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        for symbol in symbols {
            let mut env = self.prepare(instance, source)?;
            self.fix_outer(source.source_id, &symbol.index, &mut env)?;
            let root = self.instantiate_roots(source, &[source.root_id], &env)?[0];
            if declaration.role == SymbolRole::Expression {
                self.append(
                    "compiled.symbol_expressions",
                    compiled::symbol_expressions::Row {
                        symbol_id: symbol.symbol_id,
                        node_id: root.0,
                        derivation_id: Self::derivation(
                            instance.instance_id,
                            source.source_id,
                            "symbol_body",
                        ),
                    },
                )?;
                self.append(
                    "compiled.expression_roots",
                    compiled::expression_roots::Row {
                        owner_id: symbol.symbol_id,
                        role: ExpressionRootRole::SymbolExpression,
                        ordinal: 0,
                        node_id: root.0,
                        derivation_id: Self::derivation(
                            instance.instance_id,
                            source.source_id,
                            "symbol_body_root",
                        ),
                    },
                )?;
            } else {
                let Payload::SymbolRef {
                    symbol: ValueRef::ActualSymbol(target),
                } = self.graph.node(root)?.payload
                else {
                    return Err(invalid(
                        "reference/derivative base must resolve to one exact actual symbol member",
                    ));
                };
                if declaration.role == SymbolRole::Derivative {
                    self.derivative(instance, declaration, symbol.symbol_id, target)?;
                } else {
                    let actual = self
                        .symbols
                        .values()
                        .find(|row| row.symbol_id == target)
                        .ok_or_else(|| invalid("alias target symbol absent"))?;
                    let left = self
                        .physical
                        .quantity_type(QuantityTypeId::from_id(symbol.quantity_type_id))?;
                    let right = self
                        .physical
                        .quantity_type(QuantityTypeId::from_id(actual.quantity_type_id))?;
                    if left.key != right.key || left.canonical_unit != right.canonical_unit {
                        return Err(invalid(
                            "alias differs from actual target complete physical contract",
                        ));
                    }
                    self.append(
                        "compiled.symbol_references",
                        compiled::symbol_references::Row {
                            alias_symbol_id: symbol.symbol_id,
                            target_symbol_id: target,
                            kind: AliasKind::Reference,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn prepare(
        &mut self,
        instance: &inferred::instances::Row,
        source: &normalized::expression_sources::Row,
    ) -> Result<InstantiationEnvironment, CompilerError> {
        let extent = self
            .family_extents
            .get(source.family.as_str())
            .ok_or_else(|| invalid("source family extent absent"))?;
        self.work
            .try_grow(
                extent
                    .checked_mul(4)
                    .ok_or_else(|| invalid("source realization extent overflow"))?,
            )
            .map_err(|_| resource())?;
        self.context(instance.instance_id, Some(source.source_id))?;
        let mut env = self.environment(instance, source.source_id)?;
        self.bind_paths(instance, source.source_id, &mut env)?;
        self.bind_masks(instance, source.source_id, &mut env)?;
        let family = self
            .families
            .get(source.family.as_str())
            .ok_or_else(|| invalid("unknown normalized source family"))?;
        for binding in family.kernel_bindings.keys() {
            env.kernel_bindings.insert(
                *binding,
                pse_ids::named_id(
                    instance.instance_id,
                    &format!("pse:kernel-binding:v1:{}", binding.to_hex()),
                ),
            );
        }
        Ok(env)
    }
    pub(super) fn instantiate_roots(
        &mut self,
        source: &normalized::expression_sources::Row,
        roots: &[i64],
        env: &InstantiationEnvironment,
    ) -> Result<Vec<NodeId>, CompilerError> {
        let family = self
            .families
            .get(source.family.as_str())
            .ok_or_else(|| invalid("source family missing"))?;
        let roots = roots
            .iter()
            .map(|root| {
                family
                    .node_mapping
                    .get(&NodeId(*root))
                    .copied()
                    .ok_or_else(|| invalid("source root is absent from actual admitted graph"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut env = env.clone();
        self.bind_coordinates(source, &roots, &mut env)?;
        let family = self
            .families
            .get(source.family.as_str())
            .ok_or_else(|| invalid("source family missing"))?;
        let first_new = self.graph.len();
        let result =
            pse_templates::instantiate(family, &roots, &env, &mut self.graph, self.cancel)?;
        for ordinal in first_new..self.graph.len() {
            let node =
                NodeId(i64::try_from(ordinal).map_err(|_| invalid("graph ordinal overflow"))?);
            self.node_support
                .entry(node)
                .or_default()
                .extend(self.active_support.iter().copied());
        }
        self.source_node_evidence(source, &result.node_mapping)?;
        for mapped in result.node_mapping.values() {
            self.node_support
                .entry(*mapped)
                .or_default()
                .extend(self.active_support.iter().copied());
        }
        for (id, binding) in result.kernel_bindings {
            self.kernel_support
                .entry(id)
                .or_default()
                .extend(self.active_support.iter().copied());
            if self
                .kernels
                .insert(id, binding.clone())
                .is_some_and(|old| old != binding)
            {
                return Err(invalid("actual kernel instance bindings conflict"));
            }
        }
        for projection in result.projected_groups.values() {
            self.projection(env.instance, projection)?;
        }
        Ok(result.roots)
    }
    fn fix_outer(
        &mut self,
        source: SemanticId,
        tuple: &[SemanticId],
        env: &mut InstantiationEnvironment,
    ) -> Result<(), CompilerError> {
        let mut indices = self
            .inventory
            .indices
            .iter()
            .filter(|row| row.source_id == source && row.position.is_some())
            .collect::<Vec<_>>();
        indices.sort_by_key(|row| row.position);
        if indices.len() != tuple.len() {
            return Err(invalid(
                "symbol member tuple differs from expression's declared outer axes",
            ));
        }
        for (position, (index, member)) in indices.into_iter().zip(tuple).enumerate() {
            if index.position.and_then(|value| usize::try_from(value).ok()) != Some(position) {
                return Err(invalid(
                    "source outer axes are not a complete ordered inventory",
                ));
            }
            env.fixed_indices
                .insert(BoundIndexId::from_id(index.bound_index_id), *member);
        }
        env.free_indices = pse_quantity::IndexSet::new();
        // A fully fixed finite predicate has one actual decided outcome; never a guessed bool.
        let masks = env.predicate_masks.keys().copied().collect::<Vec<_>>();
        for key in masks {
            let mask = &env.predicate_masks[&key];
            if let Some(tuple) = mask
                .coordinates
                .iter()
                .map(|coordinate| env.fixed_indices.get(coordinate).copied())
                .collect::<Option<Vec<_>>>()
            {
                let outcomes = self
                    .inventory
                    .predicates
                    .iter()
                    .filter(|row| {
                        row.instance_id == env.instance
                            && row.source_id == key.0
                            && row.predicate_id == key.1
                            && row.index == tuple
                    })
                    .collect::<Vec<_>>();
                let [outcome] = outcomes.as_slice() else {
                    return Err(invalid("fixed predicate has no unique actual outcome"));
                };
                let value = match outcome.outcome.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(invalid("fixed predicate remains undecided")),
                };
                env.predicate_masks.remove(&key);
                env.predicates.insert(key, value);
            } else {
                let mask = env
                    .predicate_masks
                    .get(&key)
                    .ok_or_else(|| invalid("predicate mask disappeared"))?;
                let fixed = mask
                    .coordinates
                    .iter()
                    .enumerate()
                    .filter_map(|(axis, coordinate)| {
                        env.fixed_indices
                            .get(coordinate)
                            .map(|member| (axis, *member))
                    })
                    .map(|(axis, member)| {
                        Ok((
                            i64::try_from(axis).map_err(|_| invalid("predicate axis overflow"))?,
                            member,
                        ))
                    })
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                if !fixed.is_empty() {
                    let projection = pse_templates::project_group(&mask.group, &fixed, env)?;
                    let coordinates = mask
                        .coordinates
                        .iter()
                        .copied()
                        .filter(|coordinate| !env.fixed_indices.contains_key(coordinate))
                        .collect();
                    self.projection(env.instance, &projection)?;
                    env.predicate_masks.insert(
                        key,
                        pse_templates::PredicateMask {
                            group: projection.group,
                            coordinates,
                        },
                    );
                }
            }
        }
        Ok(())
    }
    fn derivative(
        &mut self,
        instance: &inferred::instances::Row,
        declaration: &normalized::template_symbols::Row,
        symbol: SemanticId,
        base: SemanticId,
    ) -> Result<(), CompilerError> {
        let orders = self
            .inventory
            .derivatives
            .iter()
            .filter(|row| row.symbol_decl_id == declaration.symbol_decl_id)
            .collect::<Vec<_>>();
        let [order] = orders.as_slice() else {
            return Err(invalid(
                "derivative declaration requires one explicit order",
            ));
        };
        if order.order == 0 {
            return Err(invalid("derivative order must be positive"));
        }
        let name = declaration
            .wrt_domain
            .as_ref()
            .ok_or_else(|| invalid("derivative coordinate declaration absent"))?;
        let domain = self.domains(instance.instance_id, std::slice::from_ref(name))?[0];
        if !self
            .domain_facts
            .get(&domain)
            .is_some_and(|facts| facts.continuous && facts.unit.is_some())
        {
            return Err(invalid(
                "derivative coordinate lacks actual continuous unit-bearing domain",
            ));
        }
        self.append(
            "inferred.math_dae_links",
            inferred::math_dae_links::Row {
                derivative_symbol_id: symbol,
                state_symbol_id: base,
                wrt_domain_id: domain.as_id(),
                derivative_order: order.order,
            },
        )
    }
    fn projection(
        &mut self,
        instance: SemanticId,
        projection: &pse_templates::GroupProjection,
    ) -> Result<(), CompilerError> {
        let product = self.product(&projection.group.axes)?;
        self.append(
            "compiled.symbol_groups",
            compiled::symbol_groups::Row {
                group_id: projection.group.group,
                owner_instance_id: instance,
                name: format!(
                    "projection:{}:{:?}",
                    projection.source_group.to_hex(),
                    projection.fixed_coordinates
                ),
                product_id: product,
            },
        )?;
        self.append("compiled.group_projections", compiled::group_projections::Row { group_id: projection.group.group, source_group_id: projection.source_group,
            fixed_coordinates: projection.fixed_coordinates.iter().map(|(axis, member_id)| compiled::group_projections::CompiledGroupProjectionsFieldFixedCoordinatesItem { axis: *axis, member_id: *member_id }).collect(),
            product_id: product, derivation_id: Self::derivation(instance, projection.source_group, "group_projection") })?;
        for (tuple, symbol) in &projection.group.members {
            self.append(
                "compiled.symbol_group_members",
                compiled::symbol_group_members::Row {
                    group_id: projection.group.group,
                    tuple: tuple.clone(),
                    symbol_id: *symbol,
                },
            )?;
        }
        Ok(())
    }
}

impl Realizer<'_> {
    fn other_roots(&mut self, instance: &inferred::instances::Row) -> Result<(), CompilerError> {
        let sources = self
            .inventory
            .sources
            .iter()
            .filter(|source| {
                source
                    .owner
                    .template
                    .as_ref()
                    .is_some_and(|owner| owner.template_id == instance.template_id)
                    && source.family.as_str() == "display"
                    && source.syntax.as_str() == "expression"
            })
            .cloned()
            .collect::<Vec<_>>();
        for source in sources {
            let env = self.prepare(instance, &source)?;
            let root = self.instantiate_roots(&source, &[source.root_id], &env)?[0];
            let owner = pse_ids::named_id(
                instance.instance_id,
                &format!("pse:display:v1:{}", source.source_id.to_hex()),
            );
            self.expression_root(owner, ExpressionRootRole::Display, root, &source, &env)?;
        }
        Ok(())
    }
}
