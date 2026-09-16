// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::CompilerError;
use pse_quantity::QuantityTypeId;
use std::collections::BTreeSet;

impl Realizer<'_> {
    pub(super) fn validate_promises(&mut self) -> Result<(), CompilerError> {
        for member in &self.inventory.port_members {
            self.cancel.checkpoint()?;
            let ports = self
                .inventory
                .actual_ports
                .iter()
                .filter(|row| row.port_id == member.port_id)
                .collect::<Vec<_>>();
            let [port] = ports.as_slice() else {
                return Err(invalid("promised member port is absent or ambiguous"));
            };
            let targets = self
                .inventory
                .port_state_targets
                .iter()
                .filter(|row| row.port_id == port.port_id)
                .collect::<Vec<_>>();
            let domain_rows = self
                .inventory
                .port_state_domains
                .iter()
                .filter(|row| row.port_id == port.port_id)
                .collect::<Vec<_>>();
            let [state_domains] = domain_rows.as_slice() else {
                return Err(invalid(
                    "port actual state domain contract missing or ambiguous",
                ));
            };
            if port.state_instance_id.is_some() != state_domains.domain_ids.is_empty() {
                return Err(invalid("scalar/collection port owner alternatives differ"));
            }
            let factors = state_domains
                .domain_ids
                .iter()
                .copied()
                .map(pse_quantity::DomainId::from_id)
                .collect::<Vec<_>>();
            if self.product(&factors)? != state_domains.product_id {
                return Err(invalid(
                    "port state product differs from its actual ordered factors",
                ));
            }
            let expected = self
                .inventory
                .tuples
                .iter()
                .filter(|row| row.product_id == state_domains.product_id)
                .map(|row| row.tuple.clone())
                .collect::<BTreeSet<_>>();
            let actual = targets
                .iter()
                .map(|row| row.state_index.clone())
                .collect::<BTreeSet<_>>();
            if actual != expected || targets.len() != actual.len() {
                return Err(invalid(
                    "port state collection targets differ from complete actual domain product",
                ));
            }
            for target in targets {
                if port
                    .state_instance_id
                    .is_some_and(|state| state != target.state_instance_id)
                {
                    return Err(invalid("scalar port target owner differs"));
                }
                self.validate_member(member, target.state_instance_id, &state_domains.domain_ids)?;
            }
        }
        self.validate_aliases()
    }

    fn validate_member(
        &self,
        member: &pse_relations::generated::inferred::port_members::Row,
        owner: pse_ids::SemanticId,
        prefix: &[pse_ids::SemanticId],
    ) -> Result<(), CompilerError> {
        let instances = self
            .inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == owner)
            .collect::<Vec<_>>();
        let [instance] = instances.as_slice() else {
            return Err(invalid("promised state instance is absent or ambiguous"));
        };
        let declarations = self
            .inventory
            .symbols
            .iter()
            .filter(|row| {
                row.template_id == instance.template_id
                    && row.symbol_decl_id == member.symbol_decl_id
            })
            .collect::<Vec<_>>();
        let [declaration] = declarations.as_slice() else {
            return Err(invalid(
                "promised member declaration differs from actual state template",
            ));
        };
        if member.symbol_group != declaration.name {
            return Err(invalid(
                "promised member label differs from its exact declaration",
            ));
        }
        let promised = self
            .physical
            .quantity_type(QuantityTypeId::from_id(member.quantity_type_id))?;
        let declared = self
            .physical
            .quantity_type(QuantityTypeId::from_id(declaration.quantity_type_id))?;
        if promised.key != declared.key || promised.canonical_unit != declared.canonical_unit {
            return Err(invalid(
                "port promise differs from the complete local physical contract",
            ));
        }
        let domains = self.domains(instance.instance_id, &declaration.indexed_by)?;
        let combined = prefix
            .iter()
            .copied()
            .chain(domains.iter().map(|domain| domain.as_id()))
            .collect::<Vec<_>>();
        let domain_rows = self
            .inventory
            .port_member_domains
            .iter()
            .filter(|row| row.port_id == member.port_id && row.ordinal == member.ordinal)
            .collect::<Vec<_>>();
        let [bound_domains] = domain_rows.as_slice() else {
            return Err(invalid("port member domain contract missing or ambiguous"));
        };
        if combined != bound_domains.domain_ids
            || pse_templates::identity::domain_product_id(&combined) != bound_domains.product_id
        {
            return Err(invalid("port member combined ordered domains differ"));
        }
        let product = self.product(&domains)?;
        let expected = self
            .inventory
            .tuples
            .iter()
            .filter(|row| row.product_id == product)
            .map(|row| row.tuple.clone())
            .collect::<BTreeSet<_>>();
        let actual = self
            .symbols
            .keys()
            .filter(|(actual_owner, decl, _)| {
                *actual_owner == instance.instance_id && *decl == declaration.symbol_decl_id
            })
            .map(|(_, _, index)| index.clone())
            .collect::<BTreeSet<_>>();
        if expected != actual {
            return Err(invalid(
                "promised member domain/provider inventory differs from actual realization",
            ));
        }
        Ok(())
    }

    fn validate_aliases(&mut self) -> Result<(), CompilerError> {
        use pse_relations::generated::compiled;
        let rows = self.output.rows::<compiled::symbol_references::Row>()?;
        let mut aliases = std::collections::BTreeMap::new();
        for row in rows {
            if !self
                .symbols
                .values()
                .any(|symbol| symbol.symbol_id == row.alias_symbol_id)
                || !self
                    .symbols
                    .values()
                    .any(|symbol| symbol.symbol_id == row.target_symbol_id)
            {
                return Err(invalid("alias refers to an absent actual symbol"));
            }
            if aliases
                .insert(row.alias_symbol_id, row.target_symbol_id)
                .is_some()
            {
                return Err(invalid("one alias has multiple actual targets"));
            }
        }
        for alias in aliases.keys() {
            let mut current = *alias;
            let mut active = BTreeSet::new();
            while let Some(next) = aliases.get(&current) {
                self.cancel.checkpoint()?;
                if !active.insert(current) {
                    return Err(invalid("actual symbol reference graph contains a cycle"));
                }
                current = *next;
            }
        }
        Ok(())
    }
}
