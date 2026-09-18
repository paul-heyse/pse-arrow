// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Requires and excludes remain explicit four-valued predicates.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "feature_checks",
        S::Derived,
        &["instance_id", "rule", "antecedent", "consequent"],
        vec![
            column("instance_id", T::id()),
            column("rule", T::enumeration("FeatureRuleKind")),
            column("antecedent", T::native(arrow_schema::DataType::Utf8)),
            column("consequent", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Declared requires/excludes predicates retain false and unknown assertion outcomes.",
    );
    assertion(
        builder,
        "inferred.feature_checks",
        "feature_check_assertions",
    );
}
