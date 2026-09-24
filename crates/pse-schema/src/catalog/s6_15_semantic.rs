// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Consumed semantic compilation contracts (blueprint §6.15, ADR-0062–0064).

mod configuration;
pub(super) mod conservation_values;
mod elements;
mod kernel_methods;
mod law_contracts;
mod material_constraints;
mod methods;
mod ports;
mod source_occurrences;
mod source_definitions;
mod symbol_contracts;

use super::declarations::{column, enumeration, relation};
use crate::RegistryBuilder;
use crate::model::{
    ExtensionUse, FieldContract, FieldContract as T, Namespace as N, SnapshotClass as S,
};

/// Declare contracts before normalized copies, documents, invariants and passes.
pub fn declare(builder: &mut RegistryBuilder) {
    configuration::declare(builder);
    elements::declare(builder);
    kernel_methods::declare(builder);
    material_constraints::declare(builder);
    methods::declare(builder);
    conservation_values::declare(builder);
    law_contracts::declare(builder);
    ports::declare(builder);
    symbol_contracts::declare(builder);
    source_occurrences::declare(builder);
    source_definitions::declare(builder);
}

/// The one physical shape of a configuration value, admitted by its explicit tag.
pub fn config_value_type() -> T {
    let scalar = |name: &str, value: T| {
        T::structure(vec![value.with_name("value")])
            .with_name(name)
            .optional()
    };
    let arms = vec![
        scalar("boolean", T::native(arrow_schema::DataType::Boolean)),
        scalar("signed", T::native(arrow_schema::DataType::Int64)),
        scalar("unsigned", T::native(arrow_schema::DataType::UInt64)),
        scalar("real", T::native(arrow_schema::DataType::Float64)),
        scalar("text", T::native(arrow_schema::DataType::Utf8)),
        scalar("semantic_id", T::id()),
        T::structure(vec![
            T::id().with_name("enum_id"),
            T::native(arrow_schema::DataType::Utf8).with_name("member"),
        ])
        .with_name("enumeration")
        .optional(),
        scalar("index", index()),
        T::extended(ExtensionUse::QuantityValue)
            .with_name("quantity")
            .optional(),
    ];
    let alternative = crate::model::TaggedAlternative::new(
        "kind",
        arms.iter().map(|arm| {
            let tag = if arm.name() == "enumeration" {
                "enum"
            } else {
                arm.name()
            };
            (tag.to_owned(), arm.name().to_owned())
        }),
    );
    let mut fields = vec![T::enumeration("ConfigValueKind").with_name("kind")];
    fields.extend(arms);
    T::structure(fields).with_alternative(&alternative)
}

fn index() -> T {
    T::extended(ExtensionUse::IndexTuple)
}

fn derivation() -> FieldContract {
    FieldContract::provenance("derivation_id", T::id(), "Exact source derivation.")
}

fn derived(
    builder: &mut RegistryBuilder,
    namespace: N,
    name: &'static str,
    keys: &[&'static str],
    mut columns: Vec<FieldContract>,
    doc: &'static str,
) {
    columns.push(derivation());
    relation(builder, namespace, name, S::Derived, keys, columns, doc);
}
