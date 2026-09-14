// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete physical registry decoding, using the leaf declarations as sole authority.
use super::{
    Row, decode_reference_states, decode_unit_sets, decode_units, dimension, invalid, read,
};
use crate::CompilerError;
use pse_quantity::{
    Basis, BasisId, BasisKind, BasisRule, CompositionBasis, ConversionId, ConversionKind,
    ConversionRule, DomainKind, InputConversion, InvariantId, Opcode, OperationId,
    QuantityAdditionKind, QuantityKind, QuantityKindId, QuantityOperation, QuantityRegistry,
    QuantityRegistryBuilder, QuantityScaleRule, QuantityShapeRule, QuantityType, QuantityTypeId,
    QuantityTypeKey, RateBasis, ReferenceRule, ReferenceStateId, ScaleKind, SubjectKind,
    SubjectRule, UnitId,
};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::collections::BTreeMap;

/// Decode and admit every physical declaration from exact primitive relation bindings.
/// The neutral type is an explicit caller binding (blueprint §6.2), never inferred from
/// dimensions or a name. Omission leaves neutral-dependent inference unavailable.
///
/// # Errors
/// Missing bindings, mismatched schemas, duplicate declarations, unresolved references or
/// incompatible complete physical contracts, including the explicit neutral designation.
pub fn decode_quantity_registry(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    neutral: Option<QuantityTypeId>,
) -> Result<QuantityRegistry, CompilerError> {
    let units = decode_units(rows, registry)?;
    let sets = decode_unit_sets(rows, registry, &units)?;
    let mut builder = QuantityRegistryBuilder::new();
    for unit in units.into_values() {
        builder.unit(unit);
    }
    for set in sets.into_values() {
        builder.unit_set(set);
    }
    for state in decode_reference_states(rows, registry)? {
        builder.reference_state(state);
    }
    for row in read(rows, registry, "reference.quantity_kinds")? {
        builder.kind(QuantityKind {
            id: QuantityKindId::from_id(row.id("quantity_kind_id")?),
            dimension: dimension(row.get("dimension")?)?,
            extensive: row.boolean("extensive")?,
            addition_kind: row.enumeration("addition_kind", QuantityAdditionKind::parse)?,
        });
    }
    for row in read(rows, registry, "reference.bases")? {
        builder.basis(Basis {
            id: BasisId::from_id(row.id("basis_id")?),
            kind: row.enumeration("kind", BasisKind::parse)?,
            composition_basis: row.optional_enum("composition_basis", CompositionBasis::parse)?,
            rate_basis: row.optional_enum("rate_basis", RateBasis::parse)?,
            reference_conditions: row
                .optional_id("reference_conditions_id")?
                .map(ReferenceStateId::from_id),
        });
    }
    for row in read(rows, registry, "reference.quantity_types")? {
        builder.quantity_type(quantity_type(&row)?);
    }
    for row in read(rows, registry, "reference.conversion_rules")? {
        builder.conversion(conversion(&row)?);
    }
    for row in read(rows, registry, "reference.quantity_operations")? {
        builder.operation(operation(&row)?);
    }
    if let Some(id) = neutral {
        builder.neutral_dimensionless(id);
    }
    Ok(builder.build()?)
}
fn quantity_type(row: &Row<'_>) -> Result<QuantityType, CompilerError> {
    Ok(QuantityType {
        id: QuantityTypeId::from_id(row.id("quantity_type_id")?),
        key: QuantityTypeKey {
            kind: QuantityKindId::from_id(row.id("quantity_kind_id")?),
            basis: row.optional_id("basis_id")?.map(BasisId::from_id),
            reference_state: row
                .optional_id("reference_state_id")?
                .map(ReferenceStateId::from_id),
            scale_kind: row.enumeration("scale_kind", ScaleKind::parse)?,
            shape: row.enum_list("shape", DomainKind::parse)?,
            subject_kind: row.optional_enum("subject_kind", SubjectKind::parse)?,
        },
        canonical_unit: UnitId::from_id(row.id("canonical_unit_id")?),
        nominal_magnitude: row.optional_number("nominal_magnitude")?,
    })
}
fn conversion(row: &Row<'_>) -> Result<ConversionRule, CompilerError> {
    Ok(ConversionRule {
        id: ConversionId::from_id(row.id("conversion_id")?),
        from: QuantityTypeId::from_id(row.id("from_quantity_type_id")?),
        to: QuantityTypeId::from_id(row.id("to_quantity_type_id")?),
        kind: row.enumeration("kind", ConversionKind::parse)?,
        kernel: row.optional_id("kernel_id")?,
        required_parameters: row
            .list("required_parameters")?
            .iter()
            .map(|cell| {
                if let Cell::Text(value) = cell {
                    Ok(value.clone())
                } else {
                    Err(invalid("conversion parameter is not text"))
                }
            })
            .collect::<Result<_, _>>()?,
        scale: row.optional_number("scale")?,
        offset: row.optional_number("offset")?,
    })
}
fn operation(row: &Row<'_>) -> Result<QuantityOperation, CompilerError> {
    Ok(QuantityOperation {
        id: OperationId::from_id(row.id("operation_id")?),
        opcode: row.enumeration("opcode", Opcode::parse)?,
        input_kinds: row
            .ids("input_kind_ids")?
            .into_iter()
            .map(QuantityKindId::from_id)
            .collect(),
        result_kind: QuantityKindId::from_id(row.id("result_kind_id")?),
        basis_rule: row.enumeration("basis_rule", BasisRule::parse)?,
        reference_rule: row.enumeration("reference_rule", ReferenceRule::parse)?,
        scale_rule: row.enumeration("scale_rule", QuantityScaleRule::parse)?,
        shape_rule: row.enumeration("shape_rule", QuantityShapeRule::parse)?,
        basis_source: row.optional_u16("basis_source")?,
        reference_source: row.optional_u16("reference_source")?,
        scale_source: row.optional_u16("scale_source")?,
        shape_source: row.optional_u16("shape_source")?,
        subject_rule: row.enumeration("subject_rule", SubjectRule::parse)?,
        subject_source: row.optional_u16("subject_source")?,
        result_subject_kind: row.optional_enum("result_subject_kind", SubjectKind::parse)?,
        result_basis: row.optional_id("result_basis_id")?.map(BasisId::from_id),
        result_reference_state: row
            .optional_id("result_reference_state_id")?
            .map(ReferenceStateId::from_id),
        input_conversions: row
            .list("input_conversions")?
            .iter()
            .map(|cell| {
                let Cell::Struct(fields) = cell else {
                    return Err(invalid("input conversion is not a struct"));
                };
                let [Cell::U64(operand), Cell::Id(conversion)] = fields.as_slice() else {
                    return Err(invalid("input conversion fields differ"));
                };
                Ok(InputConversion {
                    operand: u16::try_from(*operand)
                        .map_err(|_| invalid("conversion operand out of range"))?,
                    conversion: ConversionId::from_id(*conversion),
                })
            })
            .collect::<Result<_, _>>()?,
        precondition_invariants: row
            .ids("precondition_invariant_ids")?
            .into_iter()
            .map(InvariantId::from_id)
            .collect(),
    })
}
