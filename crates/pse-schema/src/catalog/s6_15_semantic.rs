// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Consumed semantic compilation contracts (blueprint §6.15, ADR-0062–0064).

mod algorithm_occurrences;
mod configuration;
mod elements;
mod expression_outputs;
mod kernel_methods;
mod law_contracts;
mod material_constraints;
mod methods;
mod paths;
mod ports;
mod projected_groups;
mod realization;
mod reindexings;
mod source_occurrences;
mod support;
mod symbol_contracts;

use super::declarations::{column, enumeration, relation};
use crate::RegistryBuilder;
use crate::model::{
    ExtensionUse, FieldContract, FieldContract as T, Namespace as N, SnapshotClass as S,
};

/// Declare contracts before normalized copies, documents, invariants and passes.
pub fn declare(builder: &mut RegistryBuilder) {
    algorithm_occurrences::declare(builder);
    configuration::declare(builder);
    elements::declare(builder);
    expression_outputs::declare(builder);
    kernel_methods::declare(builder);
    material_constraints::declare(builder);
    methods::declare(builder);
    support::declare(builder);
    realization::declare(builder);
    law_contracts::declare(builder);
    projected_groups::declare(builder);
    reindexings::declare(builder);
    paths::declare(builder);
    ports::declare(builder);
    symbol_contracts::declare(builder);
    source_occurrences::declare(builder);
}

/// The one physical shape of a configuration value, admitted by its explicit tag.
pub fn config_value_type() -> T {
    T::structure(vec![
        T::enumeration("ConfigValueKind")
            .with_name("kind")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Boolean)
            .with_name("boolean")
            .with_nullable(true),
        T::native(arrow_schema::DataType::Int64)
            .with_name("signed")
            .with_nullable(true),
        T::native(arrow_schema::DataType::UInt64)
            .with_name("unsigned")
            .with_nullable(true),
        T::native(arrow_schema::DataType::Float64)
            .with_name("real")
            .with_nullable(true),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("text")
            .with_nullable(true),
        T::id().with_name("semantic_id").with_nullable(true),
        T::id().with_name("enum_id").with_nullable(true),
        index().with_name("index").with_nullable(true),
        T::id().with_name("quantity_type_id").with_nullable(true),
        T::id().with_name("unit_id").with_nullable(true),
    ])
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
