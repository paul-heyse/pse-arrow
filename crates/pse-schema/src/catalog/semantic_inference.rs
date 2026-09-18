// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite feature and predicate closure declarations (blueprint §6.15.1–2).
mod domains;
mod feature_checks;
mod instances;
mod material;
mod material_checks;
mod methods;
mod ports;
mod scope_construction;
mod scopes;
mod selectors;
mod topology;
use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{
        Authority, DerivationGranularity, FieldContract, FieldContract as T, Namespace as N,
        RelationDecl, SnapshotClass as S,
    },
};
/// Declare the consumed predicate outcomes and finite feature candidate substrate.
pub fn declare(builder: &mut RegistryBuilder) {
    selectors::declare(builder);
    instances::declare(builder);
    domains::declare(builder);
    scopes::declare(builder);
    ports::declare(builder);
    topology::declare(builder);
    scope_construction::declare(builder);
    relation(
        builder,
        N::Inferred,
        "predicate_axes",
        S::Derived,
        &["source_id", "predicate_id", "position"],
        vec![
            column("source_id", T::id()),
            column("predicate_id", T::nonnegative(i64::MAX)),
            column("position", T::nonnegative(i64::from(u16::MAX))),
            column("bound_index_id", T::id()),
            provenance(),
        ],
        "Only free predicate indices, ordered by declared outer position then lexical binder ID; predicate tuple interpretation is explicit.",
    );
    relation(
        builder,
        N::Inferred,
        "predicate_outcomes",
        S::Derived,
        &["instance_id", "source_id", "predicate_id", "index"],
        vec![
            column("instance_id", T::id()),
            column("source_id", T::id()),
            column("predicate_id", T::nonnegative(i64::MAX)),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("outcome", T::enumeration("TruthValue")),
            provenance(),
        ],
        "Finite typed predicate outcome over one actual bound index tuple; scalar guards use the empty tuple.",
    );
    relation(
        builder,
        N::Inferred,
        "feature_candidates",
        S::Derived,
        &["instance_id", "name", "source_owner_id", "source_name"],
        vec![
            column("instance_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("source_owner_id", T::id()),
            column("source_name", T::native(arrow_schema::DataType::Utf8)),
            column("value", super::s6_15_semantic::config_value_type()),
            provenance(),
        ],
        "Finite feature implications/inheritance preserve the original assignment alternatives.",
    );
    relation(
        builder,
        N::Inferred,
        "feature_requirements",
        S::Derived,
        &["instance_id", "name"],
        vec![
            column("instance_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Every declared feature needs a settled value; missing candidates remain explicit unknown assertions.",
    );
    for (head, name) in [
        (
            "inferred.feature_candidates",
            "feature_candidate_assertions",
        ),
        ("inferred.instance_features", "instance_feature_assertions"),
        (
            "inferred.feature_requirements",
            "feature_requirement_assertions",
        ),
    ] {
        assertion(builder, head, name);
    }
    feature_checks::declare(builder);
    material::declare(builder);
    material_checks::declare(builder);
    methods::declare(builder);
    postconditions(builder);
}
fn postconditions(builder: &mut RegistryBuilder) {
    for (relation, name, valid) in [
        (
            "provenance.feature_requirement_assertions",
            "feature_resolved",
            "truth = 'true'",
        ),
        (
            "provenance.feature_check_assertions",
            "feature_rule_satisfied",
            "truth = 'true'",
        ),
        (
            "inferred.method_compatibility",
            "selection_compatible",
            "compatible",
        ),
    ] {
        if let Some(keys) = builder
            .declared_relations()
            .iter()
            .find(|spec| spec.key.qualified_name() == relation)
            .and_then(|spec| spec.primary_key.clone())
        {
            super::inv::declare(
                builder,
                relation,
                name,
                crate::model::InvariantKind::Check,
                &keys,
                format!(
                    "SELECT {} FROM {} s WHERE ({valid}) IS NOT TRUE",
                    super::inv::columns(&keys, "s"),
                    super::inv::table(relation),
                ),
                &[relation],
                "The complete feature or capability requirement must be settled and satisfied.",
            );
        }
    }
}
fn provenance() -> FieldContract {
    FieldContract::provenance("derivation_id", T::id(), "Exact rule derivation")
}
fn assertion(builder: &mut RegistryBuilder, head: &str, name: &'static str) {
    if let Some(spec) = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == head)
    {
        let columns = crate::model::rule::assertion_columns(&spec.columns);
        builder.declare_relation(
            RelationDecl::new(
                N::Provenance,
                name,
                1,
                Authority::Derived,
                S::Derived,
                "Mechanically projected typed rule assertions",
            )
            .pk(&["assertion_id"])
            .columns(columns)
            .granularity(DerivationGranularity::Rule),
        );
    }
}
