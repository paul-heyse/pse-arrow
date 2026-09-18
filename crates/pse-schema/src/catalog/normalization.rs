// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit P3 source, predicate and package-unit contracts (blueprint §7.7, §8.2).
mod syntax;

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
        4,
        S::Derived,
        &["source_id"],
        vec![
            column("source_id", T::id()),
            column("family", T::enumeration("ExpressionFamily")),
            column("source_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("source_key", T::row_key()),
            column("field_path", T::native(arrow_schema::DataType::Utf8)),
            column("owner", expression_owner()),
            column("syntax", T::enumeration("ExpressionSyntax")),
            column("root_id", T::nonnegative(i64::MAX)),
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

fn expression_owner() -> T {
    T::structure(vec![
        T::enumeration("ExpressionOwnerKind").with_name("kind"),
        T::structure(vec![
            T::id()
                .with_fk("authored.templates", "template_id")
                .with_name("template_id"),
        ])
        .with_name("template")
        .optional(),
        T::structure(vec![
            T::id()
                .with_fk("authored.instances", "instance_id")
                .with_name("instance_id"),
        ])
        .with_name("instance")
        .optional(),
    ])
    .with_alternative(&crate::model::TaggedAlternative::new(
        "kind",
        [
            ("template".into(), "template".into()),
            ("instance".into(), "instance".into()),
        ],
    ))
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
            column("predicate_id", T::nonnegative(i64::MAX)),
            column("value", syntax::predicate()),
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
            column("equation_id", T::nonnegative(i64::MAX)),
            column("value", syntax::equation()),
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
            column("domain", super::math_value::domain()),
            column("position", T::nonnegative(i64::from(u16::MAX))).optional(),
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
