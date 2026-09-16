// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected method coefficients retain their actual source key/value and complete type.
use super::{Realizer, invalid};
use crate::CompilerError;
use pse_mathir::Payload;
use pse_quantity::{QuantityTypeId, UnitId};
use pse_relations::generated::{compiled, enums::SymbolRole, normalized};
use pse_templates::BindingValue;
use std::collections::BTreeSet;

impl Realizer<'_> {
    pub(super) fn bind_method_parameters(&mut self) -> Result<(), CompilerError> {
        let bindings = self.inventory.parameter_bindings.clone();
        let sources = self.inventory.parameter_values.clone();
        let mut seen = BTreeSet::new();
        for (binding_position, binding) in bindings.into_iter().enumerate() {
            if !self.selected.contains(&binding.method_instance_id) {
                continue;
            }
            self.cancel.checkpoint()?;
            let key = (
                binding.method_instance_id,
                binding.symbol_decl_id,
                binding.index.clone(),
            );
            let symbol = self
                .symbols
                .get(&key)
                .ok_or_else(|| invalid("method parameter binding has no actual realized member"))?;
            if symbol.role != SymbolRole::Parameter || !seen.insert(symbol.symbol_id) {
                return Err(invalid("method parameter binding role differs or repeats"));
            }
            let quantity = self
                .physical
                .quantity_type(QuantityTypeId::from_id(binding.quantity_type_id))?;
            let actual = self
                .physical
                .quantity_type(QuantityTypeId::from_id(symbol.quantity_type_id))?;
            if quantity.key != actual.key
                || !quantity.key.shape.is_empty()
                || quantity.canonical_unit != actual.canonical_unit
            {
                return Err(invalid(
                    "method coefficient binding differs from actual scalar physical contract",
                ));
            }
            let selected = sources
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.owner_entity_id == binding.source_owner
                        && row.parameter_kind == binding.parameter_kind
                        && row.index == binding.source_index
                })
                .collect::<Vec<_>>();
            let [(source_position, source)] = selected.as_slice() else {
                return Err(invalid("method coefficient source key missing or repeated"));
            };
            if source.value.to_bits() != binding.value.to_bits()
                || source.unit_id != binding.unit_id
                || !binding.value.is_finite()
            {
                return Err(invalid("method coefficient source value/unit differs"));
            }
            pse_quantity::convert_spec_for_type(
                self.physical.unit(UnitId::from_id(binding.unit_id))?,
                self.physical.unit(quantity.canonical_unit)?,
                &quantity.key,
            )?;
            let symbol_id = symbol.symbol_id;
            self.parameter_literals.insert(
                symbol_id,
                BindingValue::Literal {
                    payload: Payload::FloatConst {
                        value: binding.value,
                        unit: UnitId::from_id(binding.unit_id),
                    },
                    quantity: Some(quantity.id),
                },
            );
            self.context(binding.method_instance_id, None)?;
            self.active_support
                .insert((normalized::parameter_values::RELATION_KEY, *source_position));
            self.active_support.insert((
                compiled::method_parameter_bindings::RELATION_KEY,
                binding_position,
            ));
            self.append("compiled.method_parameter_bindings", binding)?;
        }
        Ok(())
    }
}
