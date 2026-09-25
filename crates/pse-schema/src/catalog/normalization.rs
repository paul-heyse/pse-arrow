// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit P3 source, predicate and package-unit contracts (blueprint §7.7, §8.2).

use super::declarations::{column, enumeration, relation};
use crate::RegistryBuilder;
use crate::model::{
    Authority, DerivationGranularity, FieldContract as T, Namespace as N, SnapshotClass as S,
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
    declare_units(builder);
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
