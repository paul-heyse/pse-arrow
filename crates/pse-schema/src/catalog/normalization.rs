// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit P3 source, predicate and package-unit contracts (blueprint §7.7, §8.2).

use super::declarations::{column, relation};
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
