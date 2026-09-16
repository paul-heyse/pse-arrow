// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated source columns cross once into material and physical predicate algorithms.

use super::{CompilerError, invalid};
use pse_ids::CancellationToken;
use pse_material::{Element, ElementId, ElementTable};
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

pub(super) fn elements(
    batches: &BTreeMap<RelationKey, FieldCheckedBatch>,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<ElementTable, CompilerError> {
    let mut elements = Vec::new();
    if let Some(spec) = registry.relation_by_id(reference::elements::RELATION_ID)
        && let Some(batch) = batches.get(&spec.key)
    {
        let view = reference::elements::View::from_checked(batch)?;
        for index in 0..view.len() {
            cancel.checkpoint()?;
            let row = view.row(index)?;
            elements.push(Element {
                id: ElementId::from_id(row.element_id),
                symbol: row.symbol,
                name: row.name,
                atomic_mass: row.atomic_mass,
            });
        }
    }
    Ok(ElementTable::new(elements)?)
}

pub(super) fn preconditions(
    batches: &BTreeMap<RelationKey, FieldCheckedBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
    cancel: &CancellationToken,
) -> Result<Vec<PhysicalPrecondition>, CompilerError> {
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
                    if row.required_basis_id.is_none() =>
                {
                    PhysicalRequirement::OperandQuantityContract {
                        required: QuantityTypeId::from_id(
                            row.required_quantity_type_id.ok_or_else(|| {
                                invalid("physical quantity prerequisite has no required type")
                            })?,
                        ),
                        match_shape: row.match_shape.ok_or_else(|| {
                            invalid("physical quantity prerequisite has no shape policy")
                        })?,
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
                operand_positions: row.operand_positions,
                requirement,
            };
            declaration.validate(quantities)?;
            declarations.push(declaration);
        }
    }
    Ok(declarations)
}
