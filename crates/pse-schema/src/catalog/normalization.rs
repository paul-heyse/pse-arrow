// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit P3 source, predicate and package-unit contracts (blueprint §7.7, §8.2).

use super::declarations::{column, enumeration, relation};
use crate::RegistryBuilder;
use crate::model::{
    Authority, DerivationGranularity, ExtensionUse, LogicalType as T, Namespace as N,
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
        pse_mathir::ValueRef::KINDS.iter().copied(),
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
    relation(
        builder,
        N::Normalized,
        "expression_sources",
        S::Derived,
        &["source_id"],
        vec![
            column("source_id", T::id()),
            column("family", T::enumeration("ExpressionFamily")),
            column("source_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("source_key", source_key_type()),
            column("field_path", T::Text),
            column("owner_template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("syntax", T::enumeration("ExpressionSyntax")),
            column("root_id", T::U64),
            column("derivation_id", T::id()),
            column("source_span", T::Ext(ExtensionUse::SourceSpan)),
        ],
        "Complete source key and field to exact normalized syntax root.",
    );
}

fn source_key_type() -> T {
    T::list(T::Struct(vec![
        ("column_name", T::Text, false),
        ("logical_type_id", T::id(), false),
        ("semantic_id", T::id(), true),
        ("content_hash", T::hash(), true),
        ("text", T::Text, true),
        ("signed_integer", T::I64, true),
        ("unsigned_integer", T::U64, true),
        ("boolean", T::Bool, true),
        ("index_tuple", T::Ext(ExtensionUse::IndexTuple), true),
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
            column("predicate_id", T::U64),
            column("kind", T::enumeration("PredicateKind")),
            column("boolean_value", T::Bool).optional(),
            column("comparison", T::enumeration("PredicateComparison")).optional(),
            column("left_expr", T::U64).optional(),
            column("right_expr", T::U64).optional(),
            column("left_predicate", T::U64).optional(),
            column("right_predicate", T::U64).optional(),
            column("domain_id", T::id()).optional(),
            column("domain_template_id", T::id()).optional(),
            column("domain_name", T::Text).optional(),
            column("left_kind", T::enumeration("PredicateOperandKind")).optional(),
            column("left_enum_id", T::id()).optional(),
            column("left_enum_member", T::Text).optional(),
            column("right_kind", T::enumeration("PredicateOperandKind")).optional(),
            column("right_enum_id", T::id()).optional(),
            column("right_enum_member", T::Text).optional(),
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
            column("equation_id", T::U64),
            column("kind", T::enumeration("EquationSyntax")),
            column("sense", T::enumeration("Sense")).optional(),
            column("left_expr", T::U64).optional(),
            column("right_expr", T::U64).optional(),
            column("guard_predicate", T::U64).optional(),
            column("then_equation", T::U64).optional(),
            column("else_equation", T::U64).optional(),
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
            column("name", T::Text),
            column("domain_id", T::id()).optional(),
            column("template_id", T::id()).optional(),
            column("domain_name", T::Text).optional(),
            column("position", T::U16).optional(),
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
