// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated checked views cross once into the exact physical algorithm types.

use super::invalid;
use crate::physical::PhysicalError;
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_quantity::{
    Basis, BasisId, ConversionId, ConversionRule, DimensionVector, EntityKind, EntityKindId,
    InputConversion, InvariantId, OperationId, QuantityKind, QuantityKindId, QuantityOperation,
    QuantityRegistry, QuantityRegistryBuilder, QuantityType, QuantityTypeId, QuantityTypeKey,
    Ratio, ReferenceState, ReferenceStateId, Unit, UnitFactor, UnitId, UnitSet, UnitSetId,
    DefinedUnit,
};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{extension_values, reference as r},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::{BTreeMap, BTreeSet};

#[expect(
    clippy::too_many_lines,
    reason = "inventory keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) fn inventory(
    batches: &BTreeMap<RelationKey, FieldCheckedBatch>,
    registry: &Registry,
    cancel: &CancellationToken,
    context: Option<(SemanticId, SemanticId)>,
) -> Result<(QuantityRegistry, Option<QuantityKindId>), PhysicalError> {
    let mut builder = QuantityRegistryBuilder::new();
    let mut entities = BTreeSet::new();
    macro_rules! rows {
        ($relation:ident, $row:ident, $body:block) => {
            if let Some(spec) = registry.relation_by_id(r::$relation::RELATION_ID)
                && let Some(batch) = batches.get(&spec.key)
            {
                let view = r::$relation::View::from_checked(batch)?;
                for index in 0..view.len() {
                    cancel.checkpoint()?;
                    let $row = view.row(index)?;
                    $body
                }
            }
        };
    }
    {
        use pse_model::generated::enums::ModelingDeclarationKind;
        use pse_relations::generated::authored::modeling_declarations as declarations;
        if let Some(spec) = registry.relation_by_id(declarations::RELATION_ID)
            && let Some(batch) = batches.get(&spec.key)
        {
            let view = declarations::View::from_checked(batch)?;
            for index in 0..view.len() {
                cancel.checkpoint()?;
                let row = view.row(index)?;
                if row.value.kind == ModelingDeclarationKind::Entity {
                    entities.insert(row.declaration_id);
                }
                if row.value.kind == ModelingDeclarationKind::EntityKind {
                    builder.entity_kind(EntityKind {
                        id: EntityKindId::from_id(row.declaration_id.as_id()),
                        name: row.name,
                    });
                }
            }
        }
    }
    rows!(units, row, {
        // Version two: an atomic unit authors its measure; a defined unit authors only its
        // composition, and admission derives its dimension and scale (ADR-0124).
        let id = UnitId::from_id(row.unit_id);
        match (
            row.definition,
            row.dimension,
            row.scale_to_canonical,
            row.offset_to_canonical,
            row.is_affine,
        ) {
            (None, Some(value), Some(scale_to_canonical), Some(offset_to_canonical), Some(is_affine)) => {
                builder.unit(Unit {
                    id,
                    symbol: row.symbol,
                    dimension: dimension(value)?,
                    scale_to_canonical,
                    offset_to_canonical,
                    is_affine,
                    reference_state: row.reference_state_id.map(ReferenceStateId::from_id),
                    definition: None,
                });
            }
            (Some(composition), None, None, None, None) if row.reference_state_id.is_none() => {
                builder.defined_unit(DefinedUnit {
                    id,
                    symbol: row.symbol,
                    composition: composition
                        .into_iter()
                        .map(|factor| {
                            Ok(UnitFactor {
                                unit: UnitId::from_id(factor.unit_id),
                                exponent: Ratio::from_parts(factor.num, factor.den)
                                    .map_err(pse_quantity::QuantityError::from)?,
                            })
                        })
                        .collect::<Result<_, PhysicalError>>()?,
                });
            }
            _ => {
                return Err(invalid(format!(
                    "unit {id} must author either its measure (atomic) or only its composition (defined)"
                )));
            }
        }
    });
    rows!(unit_sets, row, {
        builder.unit_set(UnitSet {
            id: UnitSetId::from_id(row.unit_set_id),
            base: [
                Some(row.length_unit_id),
                Some(row.mass_unit_id),
                Some(row.time_unit_id),
                Some(row.temperature_unit_id),
                Some(row.amount_unit_id),
                row.current_unit_id,
                row.luminous_intensity_unit_id,
                row.currency_unit_id,
            ]
            .map(|id| id.map(UnitId::from_id)),
        });
    });
    rows!(reference_states, row, {
        if let Some(subject) = row.subject_id
            && !entities.contains(&subject)
        {
            return Err(invalid(format!(
                "reference state {} subject {subject} is not an authored entity",
                row.reference_state_id
            )));
        }
        builder.reference_state(ReferenceState {
            id: ReferenceStateId::from_id(row.reference_state_id),
            kind: row.kind,
            temperature: row.temperature,
            pressure: row.pressure,
            include_enthalpy_of_formation: row.include_enthalpy_of_formation,
            subject: row.subject_id.map(Into::into),
        });
    });
    rows!(quantity_kinds, row, {
        builder.kind(QuantityKind {
            id: QuantityKindId::from_id(row.quantity_kind_id),
            dimension: dimension(row.dimension)?,
            extensive: row.extensive,
            addition_kind: row.addition_kind,
            category: row.category,
        });
    });
    rows!(bases, row, {
        builder.basis(Basis {
            id: BasisId::from_id(row.basis_id),
            kind: row.kind,
            composition_basis: row.composition_basis,
            rate_basis: row.rate_basis,
            reference_conditions: row.reference_conditions_id.map(ReferenceStateId::from_id),
        });
    });
    rows!(quantity_types, row, {
        builder.quantity_type(QuantityType {
            id: QuantityTypeId::from_id(row.quantity_type_id),
            key: QuantityTypeKey {
                kind: QuantityKindId::from_id(row.quantity_kind_id),
                basis: row.basis_id.map(BasisId::from_id),
                reference_state: row.reference_state_id.map(ReferenceStateId::from_id),
                scale_kind: row.scale_kind,
                shape: row.shape.into_iter().map(EntityKindId::from_id).collect(),
                subject_kind: row.subject_kind.map(EntityKindId::from_id),
            },
            canonical_unit: UnitId::from_id(row.canonical_unit_id),
            nominal_magnitude: row.nominal_magnitude,
        });
    });
    rows!(conversion_rules, row, {
        builder.conversion(ConversionRule {
            id: ConversionId::from_id(row.conversion_id),
            from: QuantityTypeId::from_id(row.from_quantity_type_id),
            to: QuantityTypeId::from_id(row.to_quantity_type_id),
            kind: row.kind,
            kernel: row.kernel_id,
            required_parameters: row.required_parameters,
            scale: row.scale,
            offset: row.offset,
        });
    });
    rows!(quantity_operations, row, {
        builder.operation(QuantityOperation {
            id: OperationId::from_id(row.operation_id),
            opcode: row.opcode,
            input_kinds: row
                .input_kind_ids
                .into_iter()
                .map(QuantityKindId::from_id)
                .collect(),
            result_kind: QuantityKindId::from_id(row.result_kind_id),
            basis_rule: row.basis_rule,
            reference_rule: row.reference_rule,
            scale_rule: row.scale_rule,
            shape_rule: row.shape_rule,
            basis_source: row
                .basis_source
                .map(u16::try_from)
                .transpose()
                .map_err(|_| invalid("quantity operand ordinal exceeds declared width"))?,
            reference_source: row
                .reference_source
                .map(u16::try_from)
                .transpose()
                .map_err(|_| invalid("quantity operand ordinal exceeds declared width"))?,
            scale_source: row
                .scale_source
                .map(u16::try_from)
                .transpose()
                .map_err(|_| invalid("quantity operand ordinal exceeds declared width"))?,
            shape_source: row
                .shape_source
                .map(u16::try_from)
                .transpose()
                .map_err(|_| invalid("quantity operand ordinal exceeds declared width"))?,
            subject_rule: row.subject_rule,
            subject_source: row
                .subject_source
                .map(u16::try_from)
                .transpose()
                .map_err(|_| invalid("quantity operand ordinal exceeds declared width"))?,
            result_subject_kind: row.result_subject_kind.map(EntityKindId::from_id),
            result_basis: row.result_basis_id.map(BasisId::from_id),
            result_reference_state: row.result_reference_state_id.map(ReferenceStateId::from_id),
            input_conversions: row
                .input_conversions
                .into_iter()
                .map(|x| {
                    Ok(InputConversion {
                        operand: u16::try_from(x.operand)
                            .map_err(|_| invalid("conversion operand exceeds declared width"))?,
                        conversion: ConversionId::from_id(x.conversion_id),
                    })
                })
                .collect::<Result<_, PhysicalError>>()?,
            precondition_invariants: row
                .precondition_invariant_ids
                .into_iter()
                .map(InvariantId::from_id)
                .collect(),
        });
    });
    rows!(quantity_operation_reductions, row, {
        builder.reduction_domain(
            OperationId::from_id(row.operation_id),
            EntityKindId::from_id(row.domain_kind),
        );
    });
    let boolean = if let Some((neutral, boolean)) = context {
        builder.neutral_dimensionless(QuantityTypeId::from_id(neutral));
        Some(QuantityKindId::from_id(boolean))
    } else {
        None
    };
    let quantities = builder.build()?;
    if let Some(boolean) = boolean {
        quantities.kind(boolean)?;
    }
    Ok((quantities, boolean))
}

fn dimension(value: extension_values::DimensionVector) -> Result<DimensionVector, PhysicalError> {
    let mut result = [Ratio::ZERO; 8];
    for (output, value) in result.iter_mut().zip(value) {
        *output =
            Ratio::from_parts(value.num, value.den).map_err(pse_quantity::QuantityError::from)?;
    }
    Ok(DimensionVector::new(result))
}
