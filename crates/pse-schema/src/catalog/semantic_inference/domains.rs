// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual member admission precedes tuple difference; absence reads settled strata.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "domain_eligible_members",
        S::Derived,
        &["domain_id", "member_id"],
        vec![
            column("domain_id", T::id()),
            column("member_id", T::id()),
            provenance(),
        ],
        "Actual finite members admitted by explicit material subject correspondence.",
    );
    relation(
        builder,
        N::Inferred,
        "invalid_index_tuples",
        S::Derived,
        &["product_id", "tuple"],
        vec![
            column("product_id", T::id()),
            column("tuple", T::extended(crate::model::ExtensionUse::IndexTuple)),
            provenance(),
        ],
        "Finite candidates with at least one member excluded by actual material membership.",
    );
    for (head, name) in [
        (
            "inferred.domain_eligible_members",
            "domain_member_assertions",
        ),
        ("inferred.invalid_index_tuples", "invalid_index_assertions"),
        ("inferred.valid_index_tuples", "valid_index_assertions"),
    ] {
        assertion(builder, head, name);
    }
}
