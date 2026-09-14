// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.2 physical type.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{LogicalType as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.2 physical type contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_dimensions(builder);
    declare_reference_units(builder);
    declare_reference_unit_sets(builder);
    declare_reference_quantity_kinds(builder);
    declare_reference_bases(builder);
    declare_reference_reference_states(builder);
    declare_reference_quantity_types(builder);
    declare_reference_conversion_rules(builder);
    declare_reference_quantity_operations(builder);
    declare_reference_constants(builder);
    declare_opcode_vocabulary(builder);
    declare_domain_kind_vocabulary(builder);
    declare_subject_kind_vocabulary(builder);
    declare_scale_kind_vocabulary(builder);
    declare_quantity_addition_kind_vocabulary(builder);
    declare_basis_kind_vocabulary(builder);
    declare_composition_basis_vocabulary(builder);
    declare_rate_basis_vocabulary(builder);
    declare_reference_state_kind_vocabulary(builder);
    declare_conversion_kind_vocabulary(builder);
    declare_basis_rule_vocabulary(builder);
    declare_reference_rule_vocabulary(builder);
    declare_quantity_scale_rule_vocabulary(builder);
    declare_quantity_shape_rule_vocabulary(builder);
    declare_subject_rule_vocabulary(builder);
    declare_weight_normalization_vocabulary(builder);
    declare_reduction_kind_vocabulary(builder);
}

fn declare_reference_dimensions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "dimensions",
        S::Model,
        &["ordinal"],
        vec![column("ordinal", T::U16), column("name", T::Text)],
        "blueprint §6.2 physical type: dimensions.",
    );
}

fn declare_reference_units(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "units",
        S::Model,
        &["unit_id"],
        vec![
            column("unit_id", T::id()),
            column("symbol", T::Text),
            column("name", T::Text),
            column(
                "dimension",
                T::Ext(crate::model::ExtensionUse::DimensionVector),
            ),
            column("scale_to_canonical", T::F64),
            column("offset_to_canonical", T::F64),
            column("is_affine", T::Bool),
            column("reference_state_id", T::id())
                .optional()
                .with_fk("reference.reference_states", "reference_state_id"),
            column("system", T::Text),
            column("doc", T::Text),
        ],
        "blueprint §6.2 physical type: units.",
    );
}

fn declare_reference_unit_sets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "unit_sets",
        S::Model,
        &["unit_set_id"],
        vec![
            column("unit_set_id", T::id()),
            column("name", T::Text),
            column("time_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("length_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("mass_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("amount_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("temperature_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("current_unit_id", T::id())
                .optional()
                .with_fk("reference.units", "unit_id"),
            column("luminous_intensity_unit_id", T::id())
                .optional()
                .with_fk("reference.units", "unit_id"),
            column("currency_unit_id", T::id())
                .optional()
                .with_fk("reference.units", "unit_id"),
        ],
        "blueprint §6.2 physical type: unit_sets.",
    );
}

fn declare_reference_quantity_kinds(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "quantity_kinds",
        S::Model,
        &["quantity_kind_id"],
        vec![
            column("quantity_kind_id", T::id()),
            column("name", T::Text),
            column(
                "dimension",
                T::Ext(crate::model::ExtensionUse::DimensionVector),
            ),
            column("extensive", T::Bool),
            column("addition_kind", T::enumeration("QuantityAdditionKind")),
            column("doc", T::Text),
        ],
        "blueprint §6.2 physical type: quantity_kinds.",
    );
}

fn declare_reference_bases(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "bases",
        S::Model,
        &["basis_id"],
        vec![
            column("basis_id", T::id()),
            column("kind", T::enumeration("BasisKind")),
            column("composition_basis", T::enumeration("CompositionBasis")).optional(),
            column("rate_basis", T::enumeration("RateBasis")).optional(),
            column("reference_conditions_id", T::id())
                .optional()
                .with_fk("reference.reference_states", "reference_state_id"),
        ],
        "blueprint §6.2 physical type: bases.",
    );
}

fn declare_reference_reference_states(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "reference_states",
        S::Model,
        &["reference_state_id"],
        vec![
            column("reference_state_id", T::id()),
            column("kind", T::enumeration("ReferenceStateKind")),
            column("temperature", T::F64).optional(),
            column("pressure", T::F64).optional(),
            column("include_enthalpy_of_formation", T::Bool),
            column("phase_id", T::id())
                .optional()
                .with_fk("authored.phases", "phase_id"),
            column("doc", T::Text),
        ],
        "blueprint §6.2 physical type: reference_states.",
    );
}

fn declare_reference_quantity_types(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "quantity_types",
        S::Model,
        &["quantity_type_id"],
        vec![
            column("quantity_type_id", T::id()),
            column("quantity_kind_id", T::id())
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
            column("basis_id", T::id())
                .optional()
                .with_fk("reference.bases", "basis_id"),
            column("reference_state_id", T::id())
                .optional()
                .with_fk("reference.reference_states", "reference_state_id"),
            column("scale_kind", T::enumeration("ScaleKind")),
            column("shape", T::list(T::enumeration("DomainKind"))),
            column("subject_kind", T::enumeration("SubjectKind")).optional(),
            column("canonical_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("nominal_magnitude", T::F64).optional(),
            column("doc", T::Text),
        ],
        "blueprint §6.2 physical type: quantity_types.",
    );
}

fn declare_reference_conversion_rules(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "conversion_rules",
        S::Model,
        &["conversion_id"],
        vec![
            column("conversion_id", T::id()),
            column("from_quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("to_quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("kind", T::enumeration("ConversionKind")),
            column("kernel_id", T::id()).optional(),
            column("required_parameters", T::list(T::Text)),
            column("scale", T::F64).optional(),
            column("offset", T::F64).optional(),
        ],
        "blueprint §6.2 physical type: conversion_rules.",
    );
}

fn declare_reference_quantity_operations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "quantity_operations",
        S::Model,
        &["operation_id"],
        vec![
            column("operation_id", T::id()),
            column("opcode", T::enumeration("Opcode")),
            column("input_kind_ids", T::list(T::id())),
            column("result_kind_id", T::id())
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
            column("basis_rule", T::enumeration("BasisRule")),
            column("reference_rule", T::enumeration("ReferenceRule")),
            column("scale_rule", T::enumeration("QuantityScaleRule")),
            column("shape_rule", T::enumeration("QuantityShapeRule")),
            column("basis_source", T::U16).optional(),
            column("reference_source", T::U16).optional(),
            column("scale_source", T::U16).optional(),
            column("shape_source", T::U16).optional(),
            column("subject_rule", T::enumeration("SubjectRule")),
            column("subject_source", T::U16).optional(),
            column("result_subject_kind", T::enumeration("SubjectKind")).optional(),
            column("result_basis_id", T::id()).optional(),
            column("result_reference_state_id", T::id()).optional(),
            column(
                "input_conversions",
                T::list(structure(vec![
                    ("operand", T::U16),
                    ("conversion_id", T::id()),
                ])),
            ),
            column("precondition_invariant_ids", T::list(T::id())),
        ],
        "blueprint §6.2 physical type: quantity_operations.",
    );
}

fn declare_reference_constants(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "constants",
        S::Model,
        &["constant_id"],
        vec![
            column("constant_id", T::id()),
            column("name", T::Text),
            column("idaes_name", T::Text).optional(),
            column("value", T::F64),
            column("unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("quantity_kind_id", T::id())
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
            column("doc", T::Text),
        ],
        "blueprint §6.2 physical type: constants.",
    );
}

fn declare_opcode_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Opcode",
        pse_quantity::enums::Opcode::ALL.iter().map(|v| v.as_str()),
    );
}

fn declare_domain_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "DomainKind",
        pse_quantity::enums::DomainKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_subject_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SubjectKind",
        pse_quantity::enums::SubjectKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_scale_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ScaleKind",
        pse_quantity::enums::ScaleKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_quantity_addition_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "QuantityAdditionKind",
        pse_quantity::enums::QuantityAdditionKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_basis_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "BasisKind",
        pse_quantity::enums::BasisKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_composition_basis_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "CompositionBasis",
        pse_quantity::enums::CompositionBasis::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_rate_basis_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "RateBasis",
        pse_quantity::enums::RateBasis::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_reference_state_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ReferenceStateKind",
        pse_quantity::enums::ReferenceStateKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_conversion_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ConversionKind",
        pse_quantity::enums::ConversionKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_basis_rule_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "BasisRule",
        pse_quantity::enums::BasisRule::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_reference_rule_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ReferenceRule",
        pse_quantity::enums::ReferenceRule::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_quantity_scale_rule_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "QuantityScaleRule",
        pse_quantity::enums::QuantityScaleRule::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_quantity_shape_rule_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "QuantityShapeRule",
        pse_quantity::enums::QuantityShapeRule::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_subject_rule_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SubjectRule",
        pse_quantity::enums::SubjectRule::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_weight_normalization_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "WeightNormalization",
        pse_quantity::enums::WeightNormalization::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_reduction_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ReductionKind",
        pse_quantity::enums::ReductionKind::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}
