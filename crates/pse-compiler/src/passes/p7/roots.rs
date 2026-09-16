// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::NodeId;
use pse_quantity::BoundIndexId;
use pse_relations::generated::{compiled, enums::ExpressionRootRole, normalized};
use pse_templates::InstantiationEnvironment;

impl Realizer<'_> {
    pub(super) fn expression_root(
        &mut self,
        owner: SemanticId,
        role: ExpressionRootRole,
        node: NodeId,
        source: &normalized::expression_sources::Row,
        env: &InstantiationEnvironment,
    ) -> Result<(), CompilerError> {
        let derivation_id = Self::derivation(env.instance, source.source_id, "expression_root");
        self.append(
            "compiled.expression_roots",
            compiled::expression_roots::Row {
                owner_id: owner,
                role,
                ordinal: 0,
                node_id: node.0,
                derivation_id,
            },
        )?;
        let mut rows = self
            .inventory
            .indices
            .iter()
            .filter(|row| row.source_id == source.source_id && row.position.is_some())
            .cloned()
            .collect::<Vec<_>>();
        rows.sort_by_key(|row| row.position);
        for (position, index) in rows.into_iter().enumerate() {
            let position =
                u16::try_from(position).map_err(|_| invalid("expression root axis overflow"))?;
            if index.position != Some(position) {
                return Err(invalid("expression root axes are not contiguous"));
            }
            let source_binder = BoundIndexId::from_id(index.bound_index_id);
            if env.fixed_indices.contains_key(&source_binder) {
                return Err(invalid(
                    "indexed root unexpectedly has a fixed outer coordinate",
                ));
            }
            let binding = env
                .binders
                .get(&source_binder)
                .ok_or_else(|| invalid("expression root source binder absent"))?;
            if env.free_indices.get(binding.bound_index) != Some(binding) {
                return Err(invalid(
                    "expression root binder differs from actual free environment",
                ));
            }
            self.append(
                "compiled.expression_root_indices",
                compiled::expression_root_indices::Row {
                    owner_id: owner,
                    role,
                    ordinal: 0,
                    position,
                    bound_index_id: binding.bound_index.as_id(),
                    domain_id: binding.domain.as_id(),
                    derivation_id,
                },
            )?;
        }
        Ok(())
    }
}
