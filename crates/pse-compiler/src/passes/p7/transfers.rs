// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Transfer claims bind an actual endpoint member and its exact expression providers.
use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::{NodeId, Payload, ValueRef};
use pse_quantity::{DomainId, QuantityTypeId};
use pse_relations::{
    columnar::RelationRow,
    generated::{compiled, inferred, normalized},
};
use pse_templates::InstantiationEnvironment;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, PartialEq, Eq)]
struct Layout {
    declaration: SemanticId,
    domains: Vec<SemanticId>,
    state_domains: Vec<SemanticId>,
    states: BTreeMap<Vec<SemanticId>, SemanticId>,
    providers: BTreeMap<Vec<SemanticId>, SemanticId>,
    quantity: QuantityTypeId,
}
impl Realizer<'_> {
    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "transfer keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn transfer(
        &mut self,
        instance: &inferred::instances::Row,
        declaration: &normalized::template_contributions::Row,
        contract: &normalized::template_contribution_contracts::Row,
        source: &normalized::expression_sources::Row,
        env: &InstantiationEnvironment,
        root: NodeId,
        domains: &[DomainId],
    ) -> Result<Option<SemanticId>, CompilerError> {
        let Some(transfer) = &contract.transfer else {
            return Ok(None);
        };
        let name = &transfer.port_name;
        let ordinal = transfer.member_ordinal;
        let mut workspace = self.reserver.open("P7:transfer-proof");
        let bytes = self.inputs.values().try_fold(0_usize, |sum, batch| {
            sum.checked_add(pse_ids::validation_extent(batch.batch())?)
                .ok_or_else(|| invalid("transfer workspace overflow"))
        })?;
        workspace
            .try_grow(
                bytes
                    .checked_mul(2)
                    .ok_or_else(|| invalid("transfer workspace overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        let port = one(
            self.inventory
                .actual_ports
                .iter()
                .filter(|row| row.instance_id == instance.instance_id && row.name == *name),
            "owned transfer port",
        )?
        .clone();
        let layout = self.port_layout(port.port_id, ordinal)?;
        if layout
            .domains
            .iter()
            .copied()
            .ne(domains.iter().map(|domain| domain.as_id()))
        {
            return Err(invalid(
                "contribution axes differ from its actual port member domains",
            ));
        }
        let expected_type = self.physical.quantity_type(layout.quantity)?;
        let mut full = expected_type.key.clone();
        full.shape = domains
            .iter()
            .map(|domain| {
                self.domain_facts
                    .get(domain)
                    .map(|facts| facts.kind)
                    .ok_or_else(|| invalid("transfer domain absent"))
            })
            .collect::<Result<_, _>>()?;
        let actual_type = self
            .physical
            .quantity_type(QuantityTypeId::from_id(contract.quantity_type_id))?;
        if full != actual_type.key || expected_type.canonical_unit != actual_type.canonical_unit {
            return Err(invalid(
                "transfer contribution physical claim differs from exact port member",
            ));
        }
        self.transfer_root(root, source, env, &layout)?;
        let outgoing = match declaration.orientation.as_str() {
            "out_of_scope" => true,
            "into_scope" => false,
            _ => return Err(invalid("transfer orientation must be incoming or outgoing")),
        };
        if !(if outgoing {
            matches!(port.direction.as_str(), "outlet" | "bidirectional")
        } else {
            matches!(port.direction.as_str(), "inlet" | "bidirectional")
        }) {
            return Err(invalid(
                "contribution orientation differs from its declared owned port",
            ));
        }
        let mut matches = Vec::new();
        for connection in &self.inventory.connections {
            self.cancel.checkpoint()?;
            let endpoint = if outgoing {
                connection.from_port_id
            } else {
                connection.to_port_id
            };
            let endpoint = one(
                self.inventory
                    .actual_ports
                    .iter()
                    .filter(|row| row.port_id == endpoint),
                "connection endpoint port",
            )?;
            if endpoint.kind != port.kind {
                continue;
            }
            if !self
                .inventory
                .port_members
                .iter()
                .any(|row| row.port_id == endpoint.port_id && row.ordinal == ordinal)
            {
                continue;
            }
            let actual = self.port_layout(endpoint.port_id, ordinal)?;
            if actual != layout {
                continue;
            }
            let binding = one(
                self.inventory
                    .connection_bindings
                    .iter()
                    .filter(|row| row.rule_template_id == connection.rule_template_id),
                "actual transfer connection expansion",
            )?;
            if binding.expansion.as_str() != "equality" {
                return Err(invalid(
                    "transfer connection has no supported equality correspondence",
                ));
            }
            if !(if outgoing {
                matches!(endpoint.direction.as_str(), "outlet" | "bidirectional")
            } else {
                matches!(endpoint.direction.as_str(), "inlet" | "bidirectional")
            }) {
                return Err(invalid(
                    "transfer orientation conflicts with actual endpoint direction",
                ));
            }
            let opposite = if outgoing {
                connection.to_port_id
            } else {
                connection.from_port_id
            };
            let other = self.port_layout(opposite, ordinal)?;
            let other_type = self.physical.quantity_type(other.quantity)?;
            if actual.domains != other.domains
                || expected_type.key != other_type.key
                || expected_type.canonical_unit != other_type.canonical_unit
            {
                return Err(invalid(
                    "transfer connection member endpoints have different full physical/domain meaning",
                ));
            }
            matches.push(connection.connection_id);
        }
        let connection = one(matches.into_iter(), "actual transfer connection")?;
        self.no_duplicate_transfer(instance.instance_id, declaration, ordinal, connection)?;
        // Selection and ambiguity depend on the complete actual endpoint inventories.
        for name in [
            "normalized.connections",
            "reference.connection_bindings",
            "inferred.ports",
            "inferred.port_members",
            "inferred.port_member_domains",
            "inferred.port_state_domains",
            "inferred.port_state_targets",
        ] {
            let spec = self
                .registry
                .relation(name)
                .ok_or_else(|| invalid("transfer support declaration absent"))?;
            let rows = self
                .inventory
                .source_keys
                .get(&spec.key)
                .ok_or_else(|| invalid("transfer support rows absent"))?;
            self.active_support
                .extend((0..rows.len()).map(|index| (spec.key, index)));
        }
        Ok(Some(connection))
    }
    fn port_layout(&self, port: SemanticId, ordinal: i64) -> Result<Layout, CompilerError> {
        let member = one(
            self.inventory
                .port_members
                .iter()
                .filter(|row| row.port_id == port && row.ordinal == ordinal),
            "transfer member ordinal",
        )?;
        let domains = one(
            self.inventory
                .port_member_domains
                .iter()
                .filter(|row| row.port_id == port && row.ordinal == ordinal),
            "transfer member domains",
        )?;
        let states = one(
            self.inventory
                .port_state_domains
                .iter()
                .filter(|row| row.port_id == port),
            "transfer state domains",
        )?;
        if !domains.domain_ids.starts_with(&states.domain_ids) {
            return Err(invalid(
                "transfer state domains are not the exact member prefix",
            ));
        }
        let state_rows = self
            .inventory
            .port_state_targets
            .iter()
            .filter(|row| row.port_id == port)
            .collect::<Vec<_>>();
        let state_map = state_rows
            .iter()
            .map(|row| (row.state_index.clone(), row.state_instance_id))
            .collect::<BTreeMap<_, _>>();
        let expected = self
            .inventory
            .tuples
            .iter()
            .filter(|row| row.product_id == states.product_id)
            .map(|row| row.tuple.clone())
            .collect::<BTreeSet<_>>();
        if state_map.len() != state_rows.len()
            || state_map.keys().cloned().collect::<BTreeSet<_>>() != expected
        {
            return Err(invalid(
                "transfer state targets are not the complete unique actual product",
            ));
        }
        let mut providers = BTreeMap::new();
        for tuple in self
            .inventory
            .tuples
            .iter()
            .filter(|row| row.product_id == domains.product_id)
        {
            if tuple.tuple.len() != domains.domain_ids.len() {
                return Err(invalid("transfer tuple arity differs"));
            }
            let (prefix, local) = tuple.tuple.split_at(states.domain_ids.len());
            let owner = state_map
                .get(prefix)
                .ok_or_else(|| invalid("transfer coordinate has no actual state owner"))?;
            let symbol = self
                .symbols
                .get(&(*owner, member.symbol_decl_id, local.to_vec()))
                .ok_or_else(|| invalid("transfer member promise has no actual symbol provider"))?;
            if providers
                .insert(tuple.tuple.clone(), symbol.symbol_id)
                .is_some()
            {
                return Err(invalid("transfer member coordinate repeats"));
            }
        }
        Ok(Layout {
            declaration: member.symbol_decl_id,
            domains: domains.domain_ids.clone(),
            state_domains: states.domain_ids.clone(),
            states: state_map,
            providers,
            quantity: QuantityTypeId::from_id(member.quantity_type_id),
        })
    }
    fn transfer_root(
        &mut self,
        root: NodeId,
        source: &normalized::expression_sources::Row,
        env: &InstantiationEnvironment,
        layout: &Layout,
    ) -> Result<(), CompilerError> {
        let node = self.graph.node(root)?;
        if !node.children.is_empty() {
            return Err(invalid("transfer root is not a direct member reference"));
        }
        let payload = node.payload.clone();
        match &payload {
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(symbol),
            } if layout.domains.is_empty() => {
                if layout.providers != BTreeMap::from([(Vec::new(), *symbol)]) {
                    return Err(invalid("transfer root reads a different scalar provider"));
                }
            }
            Payload::Gather {
                group,
                coordinate_map,
            } => {
                let groups =
                    self.output_rows::<compiled::symbol_groups::Row>("compiled.symbol_groups")?;
                let group_row = one(
                    groups.iter().filter(|row| row.group_id == *group),
                    "transfer root group",
                )?;
                let product = one(
                    self.inventory
                        .products
                        .iter()
                        .filter(|row| row.product_id == group_row.product_id),
                    "transfer group product",
                )?;
                if product.domain_ids != layout.domains {
                    return Err(invalid("transfer gather has different actual domains"));
                }
                let axes = self
                    .inventory
                    .indices
                    .iter()
                    .filter(|row| row.source_id == source.source_id && row.position.is_some())
                    .map(|row| {
                        let actual = env
                            .binders
                            .get(&pse_quantity::BoundIndexId::from_id(row.bound_index_id))
                            .ok_or_else(|| invalid("transfer source axis has no actual binder"))?;
                        Ok((
                            actual.bound_index,
                            usize::try_from(
                                row.position
                                    .ok_or_else(|| invalid("transfer source position absent"))?,
                            )
                            .map_err(|_| invalid("negative transfer source position"))?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, CompilerError>>()?;
                if coordinate_map.len() != layout.domains.len()
                    || axes.len() != layout.domains.len()
                {
                    return Err(invalid("transfer gather omits or adds source axes"));
                }
                let members = self.output_rows::<compiled::symbol_group_members::Row>(
                    "compiled.symbol_group_members",
                )?;
                let actual = members
                    .into_iter()
                    .filter(|row| row.group_id == *group)
                    .map(|row| (row.tuple, row.symbol_id))
                    .collect::<BTreeMap<_, _>>();
                for (tuple, expected) in &layout.providers {
                    let mut key = vec![SemanticId::NIL; tuple.len()];
                    let mut filled = BTreeSet::new();
                    for (binder, position) in coordinate_map {
                        let outer = *axes.get(binder).ok_or_else(|| {
                            invalid("transfer gather uses a different lexical index")
                        })?;
                        let position = usize::from(*position);
                        if !filled.insert(position) {
                            return Err(invalid("transfer gather repeats a group axis"));
                        }
                        *key.get_mut(position)
                            .ok_or_else(|| invalid("transfer group axis out of range"))? = *tuple
                            .get(outer)
                            .ok_or_else(|| invalid("transfer source axis out of range"))?;
                    }
                    if actual.get(&key) != Some(expected) {
                        return Err(invalid(
                            "transfer gather coordinate reads a different actual provider",
                        ));
                    }
                }
                if actual.len() != layout.providers.len() {
                    return Err(invalid("transfer gather has extra or missing providers"));
                }
            }
            _ => {
                return Err(invalid(
                    "an arbitrary expression cannot claim to be a connection transfer",
                ));
            }
        }
        Ok(())
    }
    fn no_duplicate_transfer(
        &mut self,
        owner: SemanticId,
        declaration: &normalized::template_contributions::Row,
        ordinal: i64,
        connection: SemanticId,
    ) -> Result<(), CompilerError> {
        for row in self.output_rows::<compiled::contributions::Row>("compiled.contributions")? {
            if row.owner_instance_id == owner
                && row.transfer_connection_id == Some(connection)
                && row.orientation == declaration.orientation
            {
                let contract = one(
                    self.inventory
                        .contribution_contracts
                        .iter()
                        .filter(|contract| {
                            contract.contribution_decl_id == row.contribution_decl_id
                        }),
                    "existing transfer contract",
                )?;
                if contract
                    .transfer
                    .as_ref()
                    .is_some_and(|transfer| transfer.member_ordinal == ordinal)
                {
                    return Err(invalid(
                        "duplicate same-orientation transfer claim for one actual owner/member",
                    ));
                }
            }
        }
        Ok(())
    }
    fn output_rows<T: RelationRow>(&mut self, name: &str) -> Result<Vec<T>, CompilerError> {
        if T::relation(self.registry)?.key.qualified_name() != name {
            return Err(invalid("typed transfer output declaration differs"));
        }
        self.output.rows::<T>()
    }
}
fn one<T>(mut values: impl Iterator<Item = T>, label: &str) -> Result<T, CompilerError> {
    let value = values
        .next()
        .ok_or_else(|| invalid(format!("{label} absent")))?;
    if values.next().is_some() {
        return Err(invalid(format!("{label} ambiguous")));
    }
    Ok(value)
}
