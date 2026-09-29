// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.2 physical type.

use super::declarations::{column, relation, relation_version, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

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
    for (path, members) in pse_quantity::enums::dictionaries() {
        super::declarations::sourced_enumeration(builder, path, members);
    }
}

fn declare_reference_dimensions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "dimensions",
        S::Model,
        &["ordinal"],
        vec![
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("name", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.2 physical type: dimensions.",
    );
}

fn declare_reference_units(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Reference,
        "units",
        2,
        S::Model,
        &["unit_id"],
        vec![
            column("unit_id", T::id()),
            column("symbol", T::native(arrow_schema::DataType::Utf8)),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column(
                "dimension",
                T::extended(crate::model::ExtensionUse::DimensionVector),
            )
            .optional(),
            column(
                "scale_to_canonical",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column(
                "offset_to_canonical",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column("is_affine", T::native(arrow_schema::DataType::Boolean)).optional(),
            column("reference_state_id", T::id())
                .optional()
                .with_fk("reference.reference_states", "reference_state_id"),
            column(
                "definition",
                T::list(structure(vec![
                    ("unit_id", T::id()),
                    ("num", T::native(arrow_schema::DataType::Int16)),
                    ("den", T::native(arrow_schema::DataType::Int16)),
                ])),
            )
            .optional(),
            column("system", T::native(arrow_schema::DataType::Utf8)),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.2 physical type: units. Version two separates atomic from defined units (ADR-0124): an atomic unit authors its dimension, scale, offset, affinity and optional datum restriction; a defined unit authors only its composition, factors over declared units with reduced rational exponents, and admission derives its dimension and scale. A defined unit's identity is its unit-product identity, and literals spell it by its composition, never by its symbol.",
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
            column("name", T::native(arrow_schema::DataType::Utf8)),
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
    relation_version(
        builder,
        N::Reference,
        "quantity_kinds",
        3,
        S::Model,
        &["quantity_kind_id"],
        vec![
            column("quantity_kind_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column(
                "dimension",
                T::extended(crate::model::ExtensionUse::DimensionVector),
            )
            .optional(),
            column("extensive", T::native(arrow_schema::DataType::Boolean)),
            column("addition_kind", T::enumeration("QuantityAdditionKind")),
            column("category", T::enumeration("QuantityKindCategory")).optional(),
            column(
                "definition",
                structure(vec![
                    (
                        "monomial",
                        T::list(structure(vec![
                            ("quantity_kind_id", T::id()),
                            ("num", T::native(arrow_schema::DataType::Int16)),
                            ("den", T::native(arrow_schema::DataType::Int16)),
                        ])),
                    ),
                    ("canonical_unit_id", T::id()),
                    ("basis_id", T::id().optional()),
                    ("reference_state_id", T::id().optional()),
                    ("scale_kind", T::enumeration("ScaleKind")),
                    ("subject_kind", T::id().optional()),
                ]),
            )
            .optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.2 physical type: quantity_kinds. Version two adds the count or indicator category of a dimensionless pure-number kind (ADR-0103); measured kinds carry none. Version three adds derived kinds (ADR-0124): a derived kind authors a monomial over declared kinds with reduced rational exponents, its canonical unit and the complete result a multiplicative chain resolving to it takes (scale, optional datum and subject, and a basis when the factors' bases may differ); admission derives its dimension, so it authors none.",
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
            column("temperature", T::native(arrow_schema::DataType::Float64)).optional(),
            column("pressure", T::native(arrow_schema::DataType::Float64)).optional(),
            column(
                "include_enthalpy_of_formation",
                T::native(arrow_schema::DataType::Boolean),
            ),
            column("subject_id", T::id())
                .optional()
                .with_fk("authored.modeling_declarations", "declaration_id"),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
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
            column("shape", T::list(T::id())),
            column("subject_kind", T::id()).optional(),
            column("canonical_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column(
                "nominal_magnitude",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
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
            column(
                "required_parameters",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("scale", T::native(arrow_schema::DataType::Float64)).optional(),
            column("offset", T::native(arrow_schema::DataType::Float64)).optional(),
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
            column("basis_source", T::nonnegative(i64::from(u16::MAX))).optional(),
            column("reference_source", T::nonnegative(i64::from(u16::MAX))).optional(),
            column("scale_source", T::nonnegative(i64::from(u16::MAX))).optional(),
            column("shape_source", T::nonnegative(i64::from(u16::MAX))).optional(),
            column("subject_rule", T::enumeration("SubjectRule")),
            column("subject_source", T::nonnegative(i64::from(u16::MAX))).optional(),
            column("result_subject_kind", T::id()).optional(),
            column("result_basis_id", T::id()).optional(),
            column("result_reference_state_id", T::id()).optional(),
            column(
                "input_conversions",
                T::list(structure(vec![
                    ("operand", T::nonnegative(i64::from(u16::MAX))),
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
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("idaes_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("value", T::native(arrow_schema::DataType::Float64)),
            column("unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("quantity_kind_id", T::id())
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.2 physical type: constants.",
    );
}
