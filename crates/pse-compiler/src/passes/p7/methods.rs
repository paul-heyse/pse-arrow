// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected method outputs are ordinary realized symbols with explicit root witnesses.
use super::{Realizer, invalid};
use crate::CompilerError;
use pse_mathir::NodeId;
use pse_relations::generated::{compiled, enums::ExpressionRootRole, inferred, reference};

impl Realizer<'_> {
    pub(super) fn method_outputs(&mut self) -> Result<(), CompilerError> {
        let bindings = self.inventory.method_realizations.clone();
        let resolutions = self.inventory.method_resolutions.clone();
        let methods = self.inventory.method_specs.clone();
        for (binding_position, binding) in bindings.into_iter().enumerate() {
            self.cancel.checkpoint()?;
            let target = binding
                .output_symbol_id
                .ok_or_else(|| invalid("template method output is not an actual symbol"))?;
            let symbol = self
                .symbols
                .values()
                .find(|row| row.symbol_id == target)
                .cloned()
                .ok_or_else(|| invalid("selected provision output did not realize"))?;
            let resolution = resolutions
                .iter()
                .filter(|row| {
                    row.requirement_id == binding.requirement_id
                        && row.method_id == Some(binding.method_id)
                })
                .collect::<Vec<_>>();
            let [resolution] = resolution.as_slice() else {
                return Err(invalid("selected provision lacks exact P6 resolution"));
            };
            let method = methods
                .iter()
                .filter(|row| row.method_id == binding.method_id)
                .collect::<Vec<_>>();
            let [method] = method.as_slice() else {
                return Err(invalid("selected provision method absent or repeated"));
            };
            if binding.template_instance_id != Some(symbol.owner_instance_id)
                || binding.kernel_binding_id.is_some()
                || binding.kernel_output_ordinal.is_some()
                || binding.output_kind.as_str() != "template_symbol"
                || method.template_id != resolution.template_id
            {
                return Err(invalid("selected provision alternative/owner differs"));
            }
            self.context(symbol.owner_instance_id, None)?;
            self.active_support.insert((
                compiled::method_realizations::RELATION_KEY,
                binding_position,
            ));
            for (position, row) in self.inventory.method_resolutions.iter().enumerate() {
                if row.requirement_id == binding.requirement_id {
                    self.active_support
                        .insert((inferred::method_resolutions::RELATION_KEY, position));
                }
            }
            for (position, row) in self.inventory.property_requirements.iter().enumerate() {
                if row.requirement_id == binding.requirement_id {
                    self.active_support
                        .insert((inferred::property_requirements::RELATION_KEY, position));
                }
            }
            for (position, row) in self.inventory.method_specs.iter().enumerate() {
                if row.method_id == binding.method_id {
                    self.active_support
                        .insert((reference::method_specs::RELATION_KEY, position));
                }
            }
            for (position, row) in self.inventory.method_provisions.iter().enumerate() {
                if row.method_id == binding.method_id {
                    self.active_support
                        .insert((reference::method_provisions::RELATION_KEY, position));
                }
            }
            let bodies = self.output.rows::<compiled::symbol_expressions::Row>()?;
            let bodies = bodies
                .iter()
                .filter(|row| row.symbol_id == target)
                .collect::<Vec<_>>();
            let root = match bodies.as_slice() {
                [] => self.graph.symbol(target)?,
                [body] => NodeId(body.node_id),
                _ => return Err(invalid("selected method symbol has repeated definitions")),
            };
            self.node_support
                .entry(root)
                .or_default()
                .extend(self.active_support.iter().copied());
            self.append(
                "compiled.expression_roots",
                compiled::expression_roots::Row {
                    owner_id: target,
                    role: ExpressionRootRole::MethodOutput,
                    ordinal: 0,
                    node_id: root.0,
                    derivation_id: Self::derivation(
                        symbol.owner_instance_id,
                        binding.requirement_id,
                        "method-output",
                    ),
                },
            )?;
            self.append("compiled.method_realizations", binding)?;
        }
        Ok(())
    }
}
