// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated source columns cross once into physical predicate algorithms.

use super::{PhysicalError, invalid};
use pse_columnar::CancellationToken;
use pse_quantity::{
    BasisId, InvariantId, PhysicalPrecondition, PhysicalRequirement, QuantityRegistry,
    QuantityTypeId,
};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{enums::QuantityPreconditionKind, reference},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

pub(super) fn preconditions(
    batches: &BTreeMap<RelationKey, FieldCheckedBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
    cancel: &CancellationToken,
) -> Result<Vec<PhysicalPrecondition>, PhysicalError> {
    let mut declarations = Vec::new();
    if let Some(spec) = registry.relation_by_id(reference::quantity_preconditions::RELATION_ID)
        && let Some(batch) = batches.get(&spec.key)
    {
        let view = reference::quantity_preconditions::View::from_checked(batch)?;
        for index in 0..view.len() {
            cancel.checkpoint()?;
            let row = view.row(index)?;
            let requirement = match row.kind {
                QuantityPreconditionKind::EqualOperandBases
                    if row.required_quantity_type_id.is_none() && row.match_shape.is_none() =>
                {
                    PhysicalRequirement::EqualOperandBases {
                        required: row.required_basis_id.map(BasisId::from_id),
                    }
                }
                QuantityPreconditionKind::OperandQuantityContract
                | QuantityPreconditionKind::SameReferenceDifferences
                    if row.required_basis_id.is_none() =>
                {
                    let required =
                        QuantityTypeId::from_id(row.required_quantity_type_id.ok_or_else(
                            || invalid("physical quantity prerequisite has no required type"),
                        )?);
                    let match_shape = row.match_shape.ok_or_else(|| {
                        invalid("physical quantity prerequisite has no shape policy")
                    })?;
                    if row.kind == QuantityPreconditionKind::SameReferenceDifferences {
                        PhysicalRequirement::SameReferenceDifferences {
                            required,
                            match_shape,
                        }
                    } else {
                        PhysicalRequirement::OperandQuantityContract {
                            required,
                            match_shape,
                        }
                    }
                }
                _ => {
                    return Err(invalid(format!(
                        "physical prerequisite {} has fields inconsistent with its declared predicate",
                        row.invariant_id
                    )));
                }
            };
            let declaration = PhysicalPrecondition {
                id: InvariantId::from_id(row.invariant_id),
                operand_positions: row
                    .operand_positions
                    .into_iter()
                    .map(u16::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|_| invalid("physical operand ordinal exceeds declared width"))?,
                requirement,
            };
            declaration.validate(quantities)?;
            declarations.push(declaration);
        }
    }
    Ok(declarations)
}
