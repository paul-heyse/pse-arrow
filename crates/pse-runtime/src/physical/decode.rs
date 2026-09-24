// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated checked views cross once into the exact physical algorithm types.

use super::invalid;
use crate::physical::PhysicalError;
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_quantity::{
    Basis, BasisId, BasisKind, BasisRule, CompositionBasis, ConversionId, ConversionKind,
    ConversionRule, DimensionVector, DomainKind, InputConversion, InvariantId, Opcode, OperationId,
    QuantityAdditionKind, QuantityKind, QuantityKindId, QuantityOperation, QuantityRegistry,
    QuantityRegistryBuilder, QuantityScaleRule, QuantityShapeRule, QuantityType, QuantityTypeId,
    QuantityTypeKey, RateBasis, Ratio, ReferenceRule, ReferenceState, ReferenceStateId,
    ReferenceStateKind, ScaleKind, SubjectKind, SubjectRule, Unit, UnitId, UnitSet, UnitSetId,
};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{extension_values, reference as r},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

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
    rows!(units, row, {
        builder.unit(Unit {
            id: UnitId::from_id(row.unit_id),
            symbol: row.symbol,
            dimension: dimension(row.dimension)?,
            scale_to_canonical: row.scale_to_canonical,
            offset_to_canonical: row.offset_to_canonical,
            is_affine: row.is_affine,
            reference_state: row.reference_state_id.map(ReferenceStateId::from_id),
        });
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
        builder.reference_state(ReferenceState {
            id: ReferenceStateId::from_id(row.reference_state_id),
            kind: enumeration(row.kind.as_str(), ReferenceStateKind::parse)?,
            temperature: row.temperature,
            pressure: row.pressure,
            include_enthalpy_of_formation: row.include_enthalpy_of_formation,
            phase: row.phase_id,
        });
    });
    rows!(quantity_kinds, row, {
        builder.kind(QuantityKind {
            id: QuantityKindId::from_id(row.quantity_kind_id),
            dimension: dimension(row.dimension)?,
            extensive: row.extensive,
            addition_kind: enumeration(row.addition_kind.as_str(), QuantityAdditionKind::parse)?,
        });
    });
    rows!(bases, row, {
        builder.basis(Basis {
            id: BasisId::from_id(row.basis_id),
            kind: enumeration(row.kind.as_str(), BasisKind::parse)?,
            composition_basis: row
                .composition_basis
                .map(|x| enumeration(x.as_str(), CompositionBasis::parse))
                .transpose()?,
            rate_basis: row
                .rate_basis
                .map(|x| enumeration(x.as_str(), RateBasis::parse))
                .transpose()?,
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
                scale_kind: enumeration(row.scale_kind.as_str(), ScaleKind::parse)?,
                shape: row
                    .shape
                    .into_iter()
                    .map(|x| enumeration(x.as_str(), DomainKind::parse))
                    .collect::<Result<_, _>>()?,
                subject_kind: row
                    .subject_kind
                    .map(|x| enumeration(x.as_str(), SubjectKind::parse))
                    .transpose()?,
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
            kind: enumeration(row.kind.as_str(), ConversionKind::parse)?,
            kernel: row.kernel_id,
            required_parameters: row.required_parameters,
            scale: row.scale,
            offset: row.offset,
        });
    });
    rows!(quantity_operations, row, {
        builder.operation(QuantityOperation {
            id: OperationId::from_id(row.operation_id),
            opcode: enumeration(row.opcode.as_str(), Opcode::parse)?,
            input_kinds: row
                .input_kind_ids
                .into_iter()
                .map(QuantityKindId::from_id)
                .collect(),
            result_kind: QuantityKindId::from_id(row.result_kind_id),
            basis_rule: enumeration(row.basis_rule.as_str(), BasisRule::parse)?,
            reference_rule: enumeration(row.reference_rule.as_str(), ReferenceRule::parse)?,
            scale_rule: enumeration(row.scale_rule.as_str(), QuantityScaleRule::parse)?,
            shape_rule: enumeration(row.shape_rule.as_str(), QuantityShapeRule::parse)?,
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
            subject_rule: enumeration(row.subject_rule.as_str(), SubjectRule::parse)?,
            subject_source: row
                .subject_source
                .map(u16::try_from)
                .transpose()
                .map_err(|_| invalid("quantity operand ordinal exceeds declared width"))?,
            result_subject_kind: row
                .result_subject_kind
                .map(|x| enumeration(x.as_str(), SubjectKind::parse))
                .transpose()?,
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
            enumeration(row.domain_kind.as_str(), DomainKind::parse)?,
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

fn enumeration<T>(name: &str, parse: fn(&str) -> Option<T>) -> Result<T, PhysicalError> {
    parse(name).ok_or_else(|| invalid(format!("physical algorithm has no declared {name} member")))
}

fn dimension(value: extension_values::DimensionVector) -> Result<DimensionVector, PhysicalError> {
    let mut result = [Ratio::ZERO; 8];
    for (output, value) in result.iter_mut().zip(value) {
        *output =
            Ratio::from_parts(value.num, value.den).map_err(pse_quantity::QuantityError::from)?;
    }
    Ok(DimensionVector::new(result))
}
