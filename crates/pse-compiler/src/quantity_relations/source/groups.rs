// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Indexed contracts resolve through actual instance/template declarations and tuples.
use super::{DomainFacts, GroupFacts, SymbolFacts, invalid, read};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_quantity::{DomainId, QuantityRegistry, QuantityTypeId};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::collections::{BTreeMap, BTreeSet};

struct Declaration {
    id: SemanticId,
    quantity: QuantityTypeId,
    indices: Vec<String>,
}
struct Group {
    owner: SemanticId,
    declaration: SemanticId,
    facts: GroupFacts,
}

pub(super) fn decode(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
    symbols: &BTreeMap<SemanticId, SymbolFacts>,
    domains: &BTreeMap<DomainId, DomainFacts>,
) -> Result<BTreeMap<SemanticId, GroupFacts>, CompilerError> {
    let products = products(rows, registry, domains)?;
    let types = declarations(rows, registry)?;
    let bindings = domain_bindings(rows, registry, domains)?;
    let mut valid = BTreeMap::<SemanticId, BTreeSet<Vec<SemanticId>>>::new();
    for row in read(rows, registry, "inferred.valid_index_tuples")? {
        let product = row.id("product_id")?;
        let tuple = tuple(row.get("tuple")?)?;
        validate_tuple(
            &tuple,
            products
                .get(&product)
                .ok_or_else(|| invalid("valid tuple product is absent"))?,
            domains,
        )?;
        if !valid.entry(product).or_default().insert(tuple) {
            return Err(invalid("duplicate valid product tuple"));
        }
    }
    let mut groups = BTreeMap::new();
    for row in read(rows, registry, "compiled.symbol_groups")? {
        let id = row.id("group_id")?;
        let owner = row.id("owner_instance_id")?;
        let name = row.text("name")?;
        let declaration = types
            .get(&(owner, name.to_owned()))
            .ok_or_else(|| invalid("group has no exact instance/template symbol declaration"))?;
        let quantity_type = declaration.quantity;
        let product = row.id("product_id")?;
        let factors = products
            .get(&product)
            .ok_or_else(|| invalid("group product absent"))?;
        let declared_domains = declaration
            .indices
            .iter()
            .map(|name| {
                bindings
                    .get(&(owner, name.clone()))
                    .copied()
                    .ok_or_else(|| invalid("group index has no explicit instance domain binding"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if factors != &declared_domains {
            return Err(invalid(
                "group product differs from explicit ordered domain bindings",
            ));
        }
        let shape = factors
            .iter()
            .map(|domain| domains[domain].kind)
            .collect::<Vec<_>>();
        if quantities.quantity_type(quantity_type)?.key.shape != shape {
            return Err(invalid(
                "group declared shape differs from ordered actual domains",
            ));
        }
        let facts = GroupFacts {
            quantity_type,
            domains: factors.clone(),
            valid_tuples: valid
                .get(&product)
                .map_or_else(Vec::new, |tuples| tuples.iter().cloned().collect()),
            members: BTreeMap::new(),
        };
        let group = Group {
            owner,
            declaration: declaration.id,
            facts,
        };
        if groups.insert(id, group).is_some() {
            return Err(invalid("duplicate symbol group"));
        }
    }
    admit_members(rows, registry, quantities, symbols, &mut groups)?;
    Ok(groups
        .into_iter()
        .map(|(id, group)| (id, group.facts))
        .collect())
}
fn admit_members(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
    symbols: &BTreeMap<SemanticId, SymbolFacts>,
    groups: &mut BTreeMap<SemanticId, Group>,
) -> Result<(), CompilerError> {
    let mut assigned = BTreeSet::new();
    for row in read(rows, registry, "compiled.symbol_group_members")? {
        let group = row.id("group_id")?;
        let tuple = tuple(row.get("tuple")?)?;
        let symbol = row.id("symbol_id")?;
        let group = groups
            .get_mut(&group)
            .ok_or_else(|| invalid("group member has no group"))?;
        let scalar = symbols
            .get(&symbol)
            .ok_or_else(|| invalid("group member symbol absent"))?;
        if scalar.owner != group.owner
            || scalar.declaration != group.declaration
            || scalar.index != tuple
        {
            return Err(invalid(
                "group member differs from the actual symbol owner, declaration or index",
            ));
        }
        let facts = &mut group.facts;
        let mut expected = quantities.quantity_type(facts.quantity_type)?.key.clone();
        expected.shape.clear();
        if quantities.quantity_type(scalar.quantity)?.key != expected {
            return Err(invalid(
                "group member differs from complete scalar quantity contract",
            ));
        }
        if !facts.valid_tuples.contains(&tuple) || facts.members.insert(tuple, symbol).is_some() {
            return Err(invalid("group tuple is invalid or repeated"));
        }
        if !assigned.insert(symbol) {
            return Err(invalid(
                "scalar symbol belongs to more than one group coordinate",
            ));
        }
    }
    for group in groups.values() {
        if group.facts.members.len() != group.facts.valid_tuples.len() {
            return Err(invalid(
                "group membership is incomplete for actual valid tuples",
            ));
        }
    }
    if symbols
        .iter()
        .any(|(id, facts)| !facts.index.is_empty() && !assigned.contains(id))
    {
        return Err(invalid(
            "indexed scalar symbol has no complete actual group membership",
        ));
    }
    Ok(())
}

fn declarations(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<BTreeMap<(SemanticId, String), Declaration>, CompilerError> {
    let mut templates = BTreeMap::new();
    for row in read(rows, registry, "authored.template_symbols")? {
        let key = (row.id("template_id")?, row.text("name")?.to_owned());
        if templates
            .insert(
                key,
                Declaration {
                    id: row.id("symbol_decl_id")?,
                    quantity: QuantityTypeId::from_id(row.id("quantity_type_id")?),
                    indices: row
                        .list("indexed_by")?
                        .iter()
                        .map(|cell| match cell {
                            Cell::Text(name) => Ok(name.clone()),
                            _ => Err(invalid("template index name is not text")),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                },
            )
            .is_some()
        {
            return Err(invalid("ambiguous template symbol name"));
        }
    }
    let mut instances = BTreeSet::new();
    let mut result = BTreeMap::new();
    for row in read(rows, registry, "authored.instances")? {
        let instance = row.id("instance_id")?;
        if !instances.insert(instance) {
            return Err(invalid("duplicate instance identity"));
        }
        let template = row.id("template_id")?;
        for ((owner, name), ty) in &templates {
            if *owner == template {
                result.insert(
                    (instance, name.clone()),
                    Declaration {
                        id: ty.id,
                        quantity: ty.quantity,
                        indices: ty.indices.clone(),
                    },
                );
            }
        }
    }
    Ok(result)
}
fn products(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    domains: &BTreeMap<DomainId, DomainFacts>,
) -> Result<BTreeMap<SemanticId, Vec<DomainId>>, CompilerError> {
    let mut products = BTreeMap::new();
    for row in read(rows, registry, "normalized.domain_products")? {
        let factors = row
            .ids("domain_ids")?
            .into_iter()
            .map(DomainId::from_id)
            .collect::<Vec<_>>();
        if factors.iter().any(|id| !domains.contains_key(id)) {
            return Err(invalid("product factor domain absent"));
        }
        if products.insert(row.id("product_id")?, factors).is_some() {
            return Err(invalid("duplicate domain product"));
        }
    }
    Ok(products)
}
fn tuple(cell: &Cell) -> Result<Vec<SemanticId>, CompilerError> {
    let Cell::List(values) = cell else {
        return Err(invalid("index tuple storage is not a list"));
    };
    values
        .iter()
        .map(|value| match value {
            Cell::Id(id) => Ok(*id),
            _ => Err(invalid("index tuple contains a non-ID")),
        })
        .collect()
}
fn validate_tuple(
    tuple: &[SemanticId],
    factors: &[DomainId],
    domains: &BTreeMap<DomainId, DomainFacts>,
) -> Result<(), CompilerError> {
    if tuple.len() != factors.len()
        || tuple
            .iter()
            .zip(factors)
            .any(|(member, domain)| !domains[domain].members.contains(member))
    {
        return Err(invalid(
            "tuple member/arity differs from actual product factors",
        ));
    }
    Ok(())
}

fn domain_bindings(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    domains: &BTreeMap<DomainId, DomainFacts>,
) -> Result<BTreeMap<(SemanticId, String), DomainId>, CompilerError> {
    let mut declarations = BTreeMap::new();
    for row in read(rows, registry, "authored.template_domains")? {
        let key = (row.id("template_id")?, row.text("name")?.to_owned());
        let contract = (
            row.enumeration("kind", pse_quantity::DomainKind::parse)?,
            row.boolean("continuous")?,
            row.optional_id("unit_id")?
                .map(pse_quantity::UnitId::from_id),
        );
        if declarations.insert(key, contract).is_some() {
            return Err(invalid("duplicate template domain declaration"));
        }
    }
    let owners = read(rows, registry, "authored.instances")?
        .iter()
        .map(|row| Ok((row.id("instance_id")?, row.id("template_id")?)))
        .collect::<Result<BTreeMap<_, _>, CompilerError>>()?;
    let mut result = BTreeMap::new();
    for row in read(rows, registry, "authored.instance_domain_bindings")? {
        let key = (row.id("instance_id")?, row.text("domain_name")?.to_owned());
        let template = owners
            .get(&key.0)
            .ok_or_else(|| invalid("domain binding instance absent"))?;
        let (kind, continuous, unit) = declarations
            .get(&(*template, key.1.clone()))
            .ok_or_else(|| invalid("domain binding has no actual template domain declaration"))?;
        let domain = DomainId::from_id(row.id("domain_id")?);
        let facts = domains
            .get(&domain)
            .ok_or_else(|| invalid("domain binding actual domain absent"))?;
        if facts.kind != *kind
            || facts.continuous != *continuous
            || unit.is_some_and(|unit| facts.unit != Some(unit))
        {
            return Err(invalid(
                "instance domain differs from template kind, continuity or declared unit",
            ));
        }
        if result.insert(key, domain).is_some() {
            return Err(invalid("duplicate instance domain binding"));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::{
        DomainKind,
        standard::{ids, standard_registry},
    };

    #[test]
    fn group_rows_bind_actual_scalar_owner_declaration_and_ordered_index() {
        let registry = pse_schema::catalog::assemble().unwrap();
        let mut quantities = standard_registry().unwrap().to_builder();
        let standard = standard_registry().unwrap();
        let mut indexed = standard
            .quantity_type(ids::quantity("neutral"))
            .unwrap()
            .clone();
        indexed.id = QuantityTypeId::from_id(SemanticId::from_bytes([70; 16]));
        indexed.key.shape = vec![DomainKind::Species];
        let group_type = indexed.id;
        quantities.quantity_type(indexed);
        let quantities = quantities.build().unwrap();
        let id = |value| SemanticId::from_bytes([value; 16]);
        let spec = registry.relation("compiled.symbol_group_members").unwrap();
        let row = vec![
            Cell::Id(id(1)),
            Cell::List(vec![Cell::Id(id(4))]),
            Cell::Id(id(2)),
        ];
        let rows = BTreeMap::from([(
            spec.key,
            pse_relations::cells::batch_from_cells(&registry, spec, &[row]).unwrap(),
        )]);
        for changed in 0..4 {
            let mut symbols = BTreeMap::from([(
                id(2),
                SymbolFacts {
                    quantity: ids::quantity("neutral"),
                    unit: ids::unit("1"),
                    owner: id(3),
                    declaration: id(5),
                    index: vec![id(4)],
                },
            )]);
            let symbol = symbols.get_mut(&id(2)).unwrap();
            match changed {
                1 => symbol.owner = id(6),
                2 => symbol.declaration = id(6),
                3 => symbol.index = vec![id(6)],
                _ => (),
            }
            let mut groups = BTreeMap::from([(
                id(1),
                Group {
                    owner: id(3),
                    declaration: id(5),
                    facts: GroupFacts {
                        quantity_type: group_type,
                        domains: vec![DomainId::from_id(id(7))],
                        valid_tuples: vec![vec![id(4)]],
                        members: BTreeMap::new(),
                    },
                },
            )]);
            assert_eq!(
                admit_members(&rows, &registry, &quantities, &symbols, &mut groups).is_ok(),
                changed == 0
            );
        }
    }
    #[test]
    fn domain_bindings_revalidate_actual_template_kind_continuity_and_unit() {
        let registry = pse_schema::catalog::assemble().unwrap();
        let id = |value| SemanticId::from_bytes([value; 16]);
        let mut rows = BTreeMap::new();
        for (name, values) in [
            (
                "authored.instances",
                vec![
                    Cell::Id(id(1)),
                    Cell::Null,
                    Cell::Id(id(2)),
                    Cell::Text("owner".into()),
                    Cell::List(vec![]),
                    Cell::List(vec![]),
                    Cell::Null,
                    Cell::Null,
                    Cell::Text(String::new()),
                ],
            ),
            (
                "authored.template_domains",
                vec![
                    Cell::Id(id(2)),
                    Cell::Text("coordinates".into()),
                    Cell::Enum("custom"),
                    Cell::Bool(true),
                    Cell::Null,
                    Cell::Null,
                    Cell::Id(id(5)),
                ],
            ),
            (
                "authored.instance_domain_bindings",
                vec![
                    Cell::Id(id(1)),
                    Cell::Text("coordinates".into()),
                    Cell::Id(id(3)),
                ],
            ),
        ] {
            let spec = registry.relation(name).unwrap();
            rows.insert(
                spec.key,
                pse_relations::cells::batch_from_cells(&registry, spec, &[values]).unwrap(),
            );
        }
        for changed in 0..4 {
            let mut domain = DomainFacts {
                kind: DomainKind::Custom,
                continuous: true,
                unit: Some(pse_quantity::UnitId::from_id(id(5))),
                members: vec![],
            };
            match changed {
                1 => domain.kind = DomainKind::Species,
                2 => domain.continuous = false,
                3 => domain.unit = None,
                _ => (),
            }
            let domains = BTreeMap::from([(DomainId::from_id(id(3)), domain)]);
            assert_eq!(
                domain_bindings(&rows, &registry, &domains).is_ok(),
                changed == 0
            );
        }
    }
}
