// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected template cardinalities count actual distinct material members.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "material_member_counts",
        S::Derived,
        &["material_system_id", "kind"],
        vec![
            column("material_system_id", T::id()),
            column("kind", T::enumeration("DomainKind")),
            column("count", T::nonnegative(i64::MAX)),
            provenance(),
        ],
        "Exact counts over distinct selected material IDs, including explicit empty lists.",
    );
    relation(
        builder,
        N::Inferred,
        "material_template_checks",
        S::Derived,
        &["instance_id"],
        vec![
            column("instance_id", T::id()),
            column("material_system_id", T::id()),
            column("phase_count", T::nonnegative(i64::MAX)),
            column("species_count", T::nonnegative(i64::MAX)),
            provenance(),
        ],
        "Actual selected material membership satisfies the template's declared bounds.",
    );
    assertion(
        builder,
        "inferred.material_member_counts",
        "material_count_assertions",
    );
    assertion(
        builder,
        "inferred.material_template_checks",
        "material_check_assertions",
    );
    let keys = ["assertion_id"];
    super::super::inv::declare(
        builder,
        "provenance.material_check_assertions",
        "material_bounds_satisfied",
        crate::model::InvariantKind::Check,
        &keys,
        "SELECT assertion_id FROM provenance.material_check_assertions WHERE (truth = 'true') IS NOT TRUE",
        &["provenance.material_check_assertions"],
        "Actual phase and species counts satisfy selected template constraints.",
    );
}
