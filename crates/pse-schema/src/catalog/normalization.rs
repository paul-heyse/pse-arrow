// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit P3 source, predicate and package-unit contracts (blueprint §7.7, §8.2).

use super::declarations::{column, enumeration, relation, relation_version};
use crate::RegistryBuilder;
use crate::model::{
    Authority, DerivationGranularity, ExtensionUse, FieldContract as T, Namespace as N,
    SnapshotClass as S,
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "package_unit_sets",
        S::Model,
        &["package_id"],
        vec![
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("unit_set_id", T::id()).with_fk("reference.unit_sets", "unit_set_id"),
        ],
        "Explicit package representation unit selection; absence is not a default.",
    );
    enumeration(
        builder,
        "ExpressionFamily",
        ["template", "instance", "display", "contribution", "guard"],
    );
    enumeration(
        builder,
        "ExpressionSyntax",
        crate::model::DslSyntax::ALL
            .into_iter()
            .map(crate::model::DslSyntax::as_str),
    );
    enumeration(
        builder,
        "PredicateKind",
        [
            "boolean", "null", "atom", "compare", "in", "and", "or", "not",
        ],
    );
    enumeration(
        builder,
        "PredicateComparison",
        ["eq", "not_eq", "lt", "le", "gt", "ge"],
    );
    enumeration(
        builder,
        "PredicateOperandKind",
        ["expression", "enum_literal"],
    );
    enumeration(builder, "EquationSyntax", ["relation", "conditional"]);
    enumeration(builder, "UnitConversionState", ["resolved", "pending"]);
    enumeration(builder, "GatherState", ["resolved", "pending"]);
    enumeration(
        builder,
        "SmoothingEpsilonState",
        ["coordinate", "pending_unit"],
    );
    enumeration(
        builder,
        "NormalizedReferenceKind",
        crate::math::NORMALIZED_REFERENCE_KINDS
            .iter()
            .copied()
            .chain([crate::math::PENDING_PATH_KIND]),
    );
    relation(
        builder,
        N::Authored,
        "template_symbol_properties",
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("property_kind_id", T::id())
                .with_fk("reference.property_kinds", "property_kind_id"),
            column("scope_selector_id", T::id()).with_fk("authored.scopes", "scope_id"),
        ],
        "Explicit property signature of a template symbol, shared by expression demand extraction.",
    );
    declare_sources(builder);
    declare_predicates(builder);
    declare_equations(builder);
    declare_bindings(builder);
    declare_units(builder);
}

fn declare_sources(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Normalized,
        "expression_sources",
        3,
        S::Derived,
        &["source_id"],
        vec![
            column("source_id", T::id()),
            column("family", T::enumeration("ExpressionFamily")),
            column("source_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("source_key", source_key_type()),
            column("field_path", T::native(arrow_schema::DataType::Utf8)),
            column("owner_template_id", T::id())
                .optional()
                .with_fk("authored.templates", "template_id"),
            column("owner_instance_id", T::id())
                .optional()
                .with_fk("authored.instances", "instance_id"),
            column("owner_kind", T::enumeration("ExpressionOwnerKind")),
            column("syntax", T::enumeration("ExpressionSyntax")),
            column("root_id", T::native(arrow_schema::DataType::UInt64)),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact normalized row derivation.",
            ),
            column("source_span", T::extended(ExtensionUse::SourceSpan)),
        ],
        "Complete source key and field to exact normalized syntax root.",
    );
}

fn source_key_type() -> T {
    T::list(T::structure(vec![
        T::native(arrow_schema::DataType::Utf8)
            .with_name("column_name")
            .with_nullable(false),
        T::id().with_name("logical_type_id").with_nullable(false),
        T::id().with_name("semantic_id").with_nullable(true),
        T::hash().with_name("content_hash").with_nullable(true),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("text")
            .with_nullable(true),
        T::native(arrow_schema::DataType::Int64)
            .with_name("signed_integer")
            .with_nullable(true),
        T::native(arrow_schema::DataType::UInt64)
            .with_name("unsigned_integer")
            .with_nullable(true),
        T::native(arrow_schema::DataType::Boolean)
            .with_name("boolean")
            .with_nullable(true),
        T::extended(ExtensionUse::IndexTuple)
            .with_name("index_tuple")
            .with_nullable(true),
    ]))
}

fn declare_predicates(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Normalized,
        "predicate_nodes",
        S::Derived,
        &["source_id", "predicate_id"],
        vec![
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("predicate_id", T::native(arrow_schema::DataType::UInt64)),
            column("kind", T::enumeration("PredicateKind")),
            column("boolean_value", T::native(arrow_schema::DataType::Boolean)).optional(),
            column("comparison", T::enumeration("PredicateComparison")).optional(),
            column("left_expr", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("right_expr", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("left_predicate", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("right_predicate", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("domain_id", T::id()).optional(),
            column("domain_template_id", T::id()).optional(),
            column("domain_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("left_kind", T::enumeration("PredicateOperandKind")).optional(),
            column("left_enum_id", T::id()).optional(),
            column("left_enum_member", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("right_kind", T::enumeration("PredicateOperandKind")).optional(),
            column("right_enum_id", T::id()).optional(),
            column("right_enum_member", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "Typed predicate structure with exact operand alternatives and order.",
    );
}

fn declare_equations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Normalized,
        "equation_nodes",
        S::Derived,
        &["source_id", "equation_id"],
        vec![
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("equation_id", T::native(arrow_schema::DataType::UInt64)),
            column("kind", T::enumeration("EquationSyntax")),
            column("sense", T::enumeration("Sense")).optional(),
            column("left_expr", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("right_expr", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("guard_predicate", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("then_equation", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("else_equation", T::native(arrow_schema::DataType::UInt64)).optional(),
        ],
        "Equation senses and conditional branch structure before instantiation.",
    );
}

fn declare_bindings(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Normalized,
        "expression_index_bindings",
        S::Derived,
        &["source_id", "bound_index_id"],
        vec![
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("bound_index_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("domain_id", T::id()).optional(),
            column("template_id", T::id()).optional(),
            column("domain_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("position", T::native(arrow_schema::DataType::UInt16)).optional(),
        ],
        "Explicit lexical index declaration and its actual or template domain.",
    );
}

fn declare_units(builder: &mut RegistryBuilder) {
    let Some(mut units) = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.namespace == N::Reference && spec.key.name == "units")
        .cloned()
    else {
        return;
    };
    units.key.namespace = N::Normalized;
    units.authority = Authority::Derived;
    units.snapshot_class = S::Derived;
    units.derivation_granularity = Some(DerivationGranularity::Row);
    units.columns.extend([
        column("package_id", T::id()).with_fk("authored.packages", "package_id"),
        column("unit_set_id", T::id()).with_fk("reference.unit_sets", "unit_set_id"),
    ]);
    builder.declare_relation(units);
}
