// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::{IndexTuple, SemanticId, symbol_instance_id};
use pse_quantity::{DomainId, QuantityTypeId};
use pse_relations::generated::{
    compiled,
    enums::{BoundKind, VariableLifecycle},
    extension_values::Bound,
};
use pse_templates::{GroupBinding, identity};
use std::collections::BTreeMap;

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "declare_symbols keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn declare_symbols(&mut self) -> Result<(), CompilerError> {
        let mut instances = self.inventory.instances.clone();
        instances.sort_by_key(|row| row.instance_id);
        let mut declarations = self.inventory.symbols.clone();
        declarations.sort_by_key(|row| row.symbol_decl_id);
        for instance in &instances {
            if !self.selected.contains(&instance.instance_id) {
                continue;
            }
            self.cancel.checkpoint()?;
            self.context(instance.instance_id, None)?;
            let template = self
                .inventory
                .templates
                .iter()
                .filter(|row| row.template_id == instance.template_id)
                .count();
            if template != 1 {
                return Err(invalid("realized instance template absent or ambiguous"));
            }
            for declaration in declarations
                .iter()
                .filter(|row| row.template_id == instance.template_id)
            {
                if !self.guard_enabled(instance.instance_id, declaration.guard_id)? {
                    continue;
                }
                let contracts = self
                    .inventory
                    .contracts
                    .iter()
                    .filter(|row| row.symbol_decl_id == declaration.symbol_decl_id)
                    .cloned()
                    .collect::<Vec<_>>();
                let [contract] = contracts.as_slice() else {
                    return Err(invalid(
                        "realized symbol requires exactly one explicit numerical/semantic contract",
                    ));
                };
                let domains = self.domains(instance.instance_id, &declaration.indexed_by)?;
                let product = self.product(&domains)?;
                let declared_type = self
                    .physical
                    .quantity_type(QuantityTypeId::from_id(declaration.quantity_type_id))?;
                let shape = domains
                    .iter()
                    .map(|domain| {
                        self.domain_facts
                            .get(domain)
                            .map(|facts| facts.kind)
                            .ok_or_else(|| invalid("symbol domain facts absent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if declared_type.key.shape != shape {
                    return Err(invalid(
                        "symbol declared quantity shape differs from its actual ordered domain kinds",
                    ));
                }
                let mut scalar_key = declared_type.key.clone();
                scalar_key.shape.clear();
                let scalar_id = self.physical.resolve_key(&scalar_key)?;
                let scalar = self.physical.quantity_type(scalar_id)?.clone();
                let tuples = self
                    .inventory
                    .tuples
                    .iter()
                    .filter(|row| row.product_id == product)
                    .map(|row| row.tuple.clone())
                    .collect::<Vec<_>>();
                if domains.is_empty() && tuples != [Vec::<SemanticId>::new()] {
                    return Err(invalid(
                        "scalar product must contain exactly its empty tuple",
                    ));
                }
                let mut members = BTreeMap::new();
                for tuple in tuples {
                    self.check_tuple(&domains, &tuple)?;
                    let symbol_id = symbol_instance_id(
                        instance.instance_id,
                        declaration.symbol_decl_id,
                        IndexTuple(&tuple),
                    );
                    if members.insert(tuple.clone(), symbol_id).is_some() {
                        return Err(invalid("duplicate realized group tuple"));
                    }
                    let row = compiled::symbols::Row {
                        symbol_id,
                        ordinal: i64::try_from(self.symbols.len())
                            .map_err(|_| invalid("symbol ordinal overflow"))?,
                        owner_instance_id: instance.instance_id,
                        symbol_decl_id: declaration.symbol_decl_id,
                        qualified_name: format!(
                            "{}.{}{}",
                            instance.path,
                            declaration.name,
                            render_index(&tuple)
                        ),
                        index: tuple.clone(),
                        quantity_type_id: scalar.id.as_id(),
                        unit_id: scalar.canonical_unit.as_id(),
                        role: declaration.role,
                        solver_type: contract.solver_type,
                        semantic_role: contract.semantic_role,
                        lifecycle: VariableLifecycle::Authored,
                        default_lower: declaration.default_lower.clone().unwrap_or_else(unbounded),
                        default_upper: declaration.default_upper.clone().unwrap_or_else(unbounded),
                        default_initial: declaration.default_initial,
                        derivation_id: Self::derivation(
                            instance.instance_id,
                            declaration.symbol_decl_id,
                            "symbol",
                        ),
                    };
                    self.remember_symbol(row.symbol_id);
                    if self
                        .symbols
                        .insert(
                            (instance.instance_id, declaration.symbol_decl_id, tuple),
                            row,
                        )
                        .is_some()
                    {
                        return Err(invalid("duplicate actual symbol identity correspondence"));
                    }
                }
                if !domains.is_empty() {
                    let group = GroupBinding {
                        group: identity::symbol_group_id(
                            instance.instance_id,
                            declaration.symbol_decl_id,
                        ),
                        axes: domains,
                        members,
                    };
                    self.append(
                        "compiled.symbol_groups",
                        compiled::symbol_groups::Row {
                            group_id: group.group,
                            owner_instance_id: instance.instance_id,
                            name: declaration.name.clone(),
                            product_id: product,
                        },
                    )?;
                    for (tuple, symbol_id) in &group.members {
                        self.append(
                            "compiled.symbol_group_members",
                            compiled::symbol_group_members::Row {
                                group_id: group.group,
                                tuple: tuple.clone(),
                                symbol_id: *symbol_id,
                            },
                        )?;
                    }
                    self.groups
                        .insert((instance.instance_id, declaration.symbol_decl_id), group);
                }
            }
        }
        Ok(())
    }
    pub(super) fn domains(
        &self,
        instance: SemanticId,
        names: &[String],
    ) -> Result<Vec<DomainId>, CompilerError> {
        names
            .iter()
            .map(|name| {
                let choices = self
                    .inventory
                    .domain_bindings
                    .iter()
                    .filter(|row| row.instance_id == instance && row.domain_name == *name)
                    .collect::<Vec<_>>();
                let [binding] = choices.as_slice() else {
                    return Err(invalid(
                        "actual instance domain binding absent or ambiguous",
                    ));
                };
                Ok(DomainId::from_id(binding.domain_id))
            })
            .collect()
    }
    pub(super) fn product(&self, domains: &[DomainId]) -> Result<SemanticId, CompilerError> {
        let factors = domains
            .iter()
            .map(|domain| domain.as_id())
            .collect::<Vec<_>>();
        let expected = identity::domain_product_id(&factors);
        let products = self
            .inventory
            .products
            .iter()
            .filter(|row| row.product_id == expected || row.domain_ids == factors)
            .collect::<Vec<_>>();
        let [product] = products.as_slice() else {
            return Err(invalid(
                "actual ordered domain product missing or ambiguous",
            ));
        };
        if product.product_id != expected || product.domain_ids != factors {
            return Err(invalid("product identity differs from actual factors"));
        }
        Ok(expected)
    }
    pub(super) fn check_tuple(
        &self,
        domains: &[DomainId],
        tuple: &[SemanticId],
    ) -> Result<(), CompilerError> {
        if tuple.len() != domains.len() {
            return Err(invalid("tuple arity differs from actual product"));
        }
        for (domain, member) in domains.iter().zip(tuple) {
            if !self
                .domain_facts
                .get(domain)
                .is_some_and(|facts| facts.members.contains(member))
            {
                return Err(invalid(
                    "actual group tuple contains a foreign domain member",
                ));
            }
        }
        Ok(())
    }
}
fn unbounded() -> Bound {
    Bound {
        kind: BoundKind::Unbounded,
        value: None,
    }
}
fn render_index(tuple: &[SemanticId]) -> String {
    if tuple.is_empty() {
        String::new()
    } else {
        format!(
            "[{}]",
            tuple
                .iter()
                .map(|member| member.to_hex())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
