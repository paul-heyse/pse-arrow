// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual predecessor facts for physical inference, joined by declared identities.
mod groups;
mod kernels;
use super::{invalid, read};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::{
    NodeId,
    index::DomainFacts,
    infer::{GroupFacts, KernelContract, SymbolTypeSource},
};
use pse_quantity::{
    ConversionId, DomainId, DomainKind, QuantityKindId, QuantityRegistry, QuantityTypeId, UnitId,
};
use pse_relations::RecordBatch;
use pse_schema::{Registry, model::RelationKey};
use std::collections::{BTreeMap, BTreeSet};

/// Admitted complete physical source facts. Private maps prevent unchecked replacement.
#[derive(Debug)]
pub struct RelationSymbolSource {
    symbols: BTreeMap<SemanticId, SymbolFacts>,
    domains: BTreeMap<DomainId, DomainFacts>,
    groups: BTreeMap<SemanticId, GroupFacts>,
    kernels: BTreeMap<SemanticId, KernelContract>,
    unknowns: BTreeMap<(SemanticId, u16), QuantityTypeId>,
    boolean: Option<QuantityKindId>,
    conversions: BTreeMap<(NodeId, u16, ConversionId), SemanticId>,
}
#[derive(Debug)]
struct SymbolFacts {
    quantity: QuantityTypeId,
    unit: UnitId,
    owner: SemanticId,
    declaration: SemanticId,
    index: Vec<SemanticId>,
}
impl SymbolTypeSource for RelationSymbolSource {
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId> {
        self.symbols.get(&symbol).map(|value| value.quantity)
    }
    fn symbol_unit(&self, symbol: SemanticId) -> Option<UnitId> {
        self.symbols.get(&symbol).map(|value| value.unit)
    }
    fn boolean_kind(&self) -> Option<QuantityKindId> {
        self.boolean
    }
    fn group(&self, group: SemanticId) -> Option<&GroupFacts> {
        self.groups.get(&group)
    }
    fn domain(&self, domain: DomainId) -> Option<&DomainFacts> {
        self.domains.get(&domain)
    }
    fn kernel_contract(&self, kernel: SemanticId) -> Option<&KernelContract> {
        self.kernels.get(&kernel)
    }
    fn implicit_unknown(&self, system: SemanticId, ordinal: u16) -> Option<QuantityTypeId> {
        self.unknowns.get(&(system, ordinal)).copied()
    }
    fn conversion_binding(
        &self,
        node: NodeId,
        operand: u16,
        conversion: ConversionId,
    ) -> Option<SemanticId> {
        self.conversions.get(&(node, operand, conversion)).copied()
    }
}
/// Decode exact predecessor bindings for P10. The Boolean kind is an explicit semantic
/// designation. Missing invariant evidence remains conservatively unavailable.
///
/// # Errors
/// Rejects absent relations, duplicate identities, invalid domain ordinals, broken joins,
/// incompatible symbol/unit/type contracts and inconsistent group/kernel declarations.
pub fn decode_symbol_source(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
    boolean: Option<QuantityKindId>,
) -> Result<RelationSymbolSource, CompilerError> {
    if let Some(id) = boolean {
        quantities.kind(id)?;
    }
    let symbols = symbols(rows, registry, quantities)?;
    let domains = domains(rows, registry, quantities)?;
    let groups = groups::decode(rows, registry, quantities, &symbols, &domains)?;
    let kernels = kernels::decode(rows, registry, quantities)?;
    let mut unknowns = BTreeMap::new();
    let mut systems = BTreeSet::new();
    for row in read(rows, registry, "compiled.math_implicit_systems")? {
        let system = row.id("implicit_system_id")?;
        if !systems.insert(system) {
            return Err(invalid("duplicate implicit system"));
        }
        let mut seen = BTreeSet::new();
        for (index, symbol) in row.ids("unknown_symbol_ids")?.into_iter().enumerate() {
            if !seen.insert(symbol) {
                return Err(invalid("implicit unknown symbol repeated"));
            }
            let ordinal =
                u16::try_from(index).map_err(|_| invalid("implicit unknown count exceeds u16"))?;
            let symbol = symbols
                .get(&symbol)
                .ok_or_else(|| invalid("implicit unknown symbol is missing"))?;
            unknowns.insert((system, ordinal), symbol.quantity);
        }
    }
    Ok(RelationSymbolSource {
        symbols,
        domains,
        groups,
        kernels,
        unknowns,
        boolean,
        conversions: BTreeMap::new(),
    })
}
fn symbols(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
) -> Result<BTreeMap<SemanticId, SymbolFacts>, CompilerError> {
    let mut declarations = BTreeMap::new();
    for row in read(rows, registry, "authored.template_symbols")? {
        let id = row.id("symbol_decl_id")?;
        let entry = (
            row.id("template_id")?,
            QuantityTypeId::from_id(row.id("quantity_type_id")?),
            row.list("indexed_by")?.len(),
        );
        if quantities.quantity_type(entry.1)?.key.shape.len() != entry.2 {
            return Err(invalid(
                "template symbol index names differ from its declared quantity shape",
            ));
        }
        if declarations.insert(id, entry).is_some() {
            return Err(invalid("duplicate template symbol declaration identity"));
        }
    }
    let mut instances = BTreeMap::new();
    for row in read(rows, registry, "authored.instances")? {
        if instances
            .insert(row.id("instance_id")?, row.id("template_id")?)
            .is_some()
        {
            return Err(invalid("duplicate instance identity"));
        }
    }
    let mut symbols = BTreeMap::new();
    let mut ordinals = BTreeSet::new();
    let mut addresses = BTreeSet::new();
    for row in read(rows, registry, "compiled.symbols")? {
        let id = row.id("symbol_id")?;
        let ty = QuantityTypeId::from_id(row.id("quantity_type_id")?);
        let unit = UnitId::from_id(row.id("unit_id")?);
        let contract = quantities.quantity_type(ty)?;
        let (template, declared_type, arity) = declarations
            .get(&row.id("symbol_decl_id")?)
            .ok_or_else(|| invalid("compiled symbol declaration is absent"))?;
        if instances.get(&row.id("owner_instance_id")?) != Some(template) {
            return Err(invalid(
                "compiled symbol belongs to a different template instance",
            ));
        }
        let mut expected = quantities.quantity_type(*declared_type)?.key.clone();
        expected.shape.clear();
        if expected != contract.key {
            return Err(invalid(
                "compiled scalar symbol differs from its complete declaration contract",
            ));
        }
        if !contract.key.shape.is_empty() {
            return Err(invalid("scalar symbol has an indexed quantity type"));
        }
        pse_quantity::convert_spec_for_type(
            quantities.unit(unit)?,
            quantities.unit(contract.canonical_unit)?,
            &contract.key,
        )?;
        let facts = SymbolFacts {
            quantity: ty,
            unit,
            owner: row.id("owner_instance_id")?,
            declaration: row.id("symbol_decl_id")?,
            index: row.ids("index")?,
        };
        if facts.index.len() != *arity
            || !addresses.insert((facts.owner, facts.declaration, facts.index.clone()))
        {
            return Err(invalid(
                "compiled symbol has wrong index arity or repeats an actual declaration coordinate",
            ));
        }
        if symbols.insert(id, facts).is_some() || !ordinals.insert(row.unsigned("ordinal")?) {
            return Err(invalid("duplicate symbol identity or ordinal"));
        }
    }
    Ok(symbols)
}
fn domains(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
) -> Result<BTreeMap<DomainId, DomainFacts>, CompilerError> {
    let mut domains = BTreeMap::new();
    for row in read(rows, registry, "authored.domains")? {
        let id = DomainId::from_id(row.id("domain_id")?);
        let unit = row.optional_id("unit_id")?.map(UnitId::from_id);
        if let Some(unit) = unit {
            quantities.unit(unit)?;
        }
        let facts = DomainFacts {
            kind: row.enumeration("kind", DomainKind::parse)?,
            continuous: row.boolean("continuous")?,
            unit,
            members: vec![],
        };
        if domains.insert(id, facts).is_some() {
            return Err(invalid("duplicate domain identity"));
        }
    }
    admit_continuous_domains(rows, registry, &domains)?;
    let mut members = BTreeMap::new();
    let mut identities = BTreeSet::new();
    for row in read(rows, registry, "authored.domain_members")? {
        let domain = DomainId::from_id(row.id("domain_id")?);
        let ordinal = row.unsigned("ordinal")?;
        let member = row.id("member_id")?;
        if !domains.contains_key(&domain)
            || !identities.insert(member)
            || members.insert((domain, ordinal), member).is_some()
        {
            return Err(invalid(
                "domain member has missing domain or repeated identity/ordinal",
            ));
        }
    }
    for ((domain, _ordinal), member) in members {
        let facts = domains
            .get_mut(&domain)
            .ok_or_else(|| invalid("domain disappeared"))?;
        facts.members.push(member);
    }
    Ok(domains)
}

fn admit_continuous_domains(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    domains: &BTreeMap<DomainId, DomainFacts>,
) -> Result<(), CompilerError> {
    let mut seen = BTreeSet::new();
    for row in read(rows, registry, "authored.continuous_domains")? {
        let id = DomainId::from_id(row.id("domain_id")?);
        let domain = domains
            .get(&id)
            .ok_or_else(|| invalid("continuous detail domain absent"))?;
        if !seen.insert(id)
            || !domain.continuous
            || domain.unit != Some(UnitId::from_id(row.id("unit_id")?))
        {
            return Err(invalid(
                "continuous detail must occur once with the exact continuous-domain unit",
            ));
        }
        let lower = row.number("lower")?;
        let upper = row.number("upper")?;
        if !lower.is_finite() || !upper.is_finite() || lower >= upper {
            return Err(invalid(
                "continuous domain lower bound must precede upper bound",
            ));
        }
    }
    if domains
        .iter()
        .any(|(id, facts)| facts.continuous && !seen.contains(id))
    {
        return Err(invalid("continuous domain has no actual coordinate detail"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_schema::model::Cell;

    #[test]
    fn domain_members_follow_declared_unique_order_without_inventing_contiguity() {
        let registry = pse_schema::catalog::assemble().unwrap();
        let quantities = pse_quantity::standard::standard_registry().unwrap();
        let id = |value| SemanticId::from_bytes([value; 16]);
        let mut rows = BTreeMap::new();
        for (name, values) in [
            (
                "authored.domains",
                vec![vec![
                    Cell::Id(id(1)),
                    Cell::Id(id(9)),
                    Cell::Enum("custom"),
                    Cell::Bool(false),
                    Cell::Null,
                    Cell::Null,
                    Cell::Text(String::new()),
                ]],
            ),
            (
                "authored.domain_members",
                vec![
                    vec![
                        Cell::Id(id(1)),
                        Cell::Id(id(2)),
                        Cell::U64(20),
                        Cell::Text("last".into()),
                        Cell::Null,
                        Cell::Null,
                    ],
                    vec![
                        Cell::Id(id(1)),
                        Cell::Id(id(3)),
                        Cell::U64(7),
                        Cell::Text("first".into()),
                        Cell::Null,
                        Cell::Null,
                    ],
                ],
            ),
            ("authored.continuous_domains", vec![]),
        ] {
            let spec = registry.relation(name).unwrap();
            rows.insert(
                spec.key,
                pse_relations::cells::batch_from_cells(&registry, spec, &values).unwrap(),
            );
        }
        let result = domains(&rows, &registry, &quantities).unwrap();
        assert_eq!(
            result[&DomainId::from_id(id(1))].members,
            vec![id(3), id(2)]
        );
    }
}
