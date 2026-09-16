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
        Authority, Cell, DerivationGranularity, FieldContract, FieldContract as T, Namespace as N,
        RelationDecl, RuleDecl, RuleExpr as E, RuleHead, RulePlan as P, SnapshotClass as S,
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
            column("predicate_id", T::native(arrow_schema::DataType::UInt64)),
            column("position", T::native(arrow_schema::DataType::UInt16)),
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
            column("predicate_id", T::native(arrow_schema::DataType::UInt64)),
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
    feature_rules(builder);
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
            E::cmp(
                crate::model::CmpOp::Eq,
                E::col("truth"),
                E::Lit(Cell::Enum("true")),
            ),
        ),
        (
            "provenance.feature_check_assertions",
            "feature_rule_satisfied",
            E::cmp(
                crate::model::CmpOp::Eq,
                E::col("truth"),
                E::Lit(Cell::Enum("true")),
            ),
        ),
        (
            "inferred.method_compatibility",
            "selection_compatible",
            E::col("compatible"),
        ),
    ] {
        if let Some(keys) = builder
            .declared_relations()
            .iter()
            .find(|spec| spec.key.qualified_name() == relation)
            .and_then(|spec| spec.primary_key.clone())
        {
            let plan = super::inv::project(
                super::inv::filter(
                    super::inv::scan(relation, "subject"),
                    E::Not(Box::new(E::IsTrue(Box::new(valid)))),
                ),
                &keys,
            );
            super::inv::declare(
                builder,
                relation,
                name,
                crate::model::InvariantKind::Check,
                &keys,
                plan,
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
fn scan(relation: &str, port: &'static str) -> P {
    P::Scan {
        relation: relation.to_owned(),
        port,
    }
}
fn project(input: P, columns: Vec<(&str, E)>) -> P {
    P::Project {
        input: Box::new(input),
        columns: (columns)
            .into_iter()
            .map(|(name, expression)| (name.to_owned().into(), expression))
            .collect(),
    }
}
fn join(left: P, right: P, keys: Vec<(&'static str, &'static str)>) -> P {
    P::EquiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: crate::model::NullEquality::NullEqualsNothing,
    }
}
fn equals(left: E, right: Cell) -> E {
    E::cmp(crate::model::CmpOp::Eq, left, E::Lit(right))
}
#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
fn feature_rules(builder: &mut RegistryBuilder) {
    let source = P::Filter {
        input: Box::new(scan("normalized.config_values", "config")),
        predicate: equals(E::col("category"), Cell::Enum("feature")),
    };
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_seed",
            "1",
            0,
            RuleHead::Relation("inferred.feature_candidates".to_owned()),
            P::Union(vec![project(
                source,
                vec![
                    ("instance_id", E::col("owner_id")),
                    ("name", E::col("name")),
                    ("source_owner_id", E::col("owner_id")),
                    ("source_name", E::col("name")),
                    ("value", E::col("value")),
                    ("derivation_id", E::col("derivation_id")),
                ],
            )]),
        )
        .assertions("provenance.feature_candidate_assertions"),
    );
    let implications = P::Filter {
        input: Box::new(scan("normalized.template_feature_rules", "implications")),
        predicate: equals(E::col("rule"), Cell::Enum("implies")),
    };
    let owners = join(
        implications,
        scan("normalized.instance_bindings", "instances"),
        vec![("implications.template_id", "instances.template_id")],
    );
    let owners = project(
        owners,
        vec![
            ("target_instance_id", E::col("instances.instance_id")),
            ("antecedent", E::col("implications.antecedent")),
            ("consequent", E::col("implications.consequent")),
        ],
    );
    let enabled = P::Filter {
        input: Box::new(scan("inferred.feature_candidates", "features")),
        predicate: E::IsTrue(Box::new(E::Field {
            expr: Box::new(E::Field {
                expr: Box::new(E::col("value")),
                name: "boolean".into(),
            }),
            name: "value".into(),
        })),
    };
    let implied = join(
        owners,
        enabled,
        vec![
            ("target_instance_id", "features.instance_id"),
            ("antecedent", "features.name"),
        ],
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_implies",
            "1",
            0,
            RuleHead::Relation("inferred.feature_candidates".to_owned()),
            project(
                implied,
                vec![
                    ("instance_id", E::col("target_instance_id")),
                    ("name", E::col("consequent")),
                    ("source_owner_id", E::col("features.source_owner_id")),
                    ("source_name", E::col("features.source_name")),
                    ("value", E::col("features.value")),
                    ("derivation_id", E::col("features.derivation_id")),
                ],
            ),
        )
        .assertions("provenance.feature_candidate_assertions"),
    );
    let inherited = join(
        scan("normalized.feature_inheritance", "inheritance"),
        scan("inferred.feature_candidates", "features"),
        vec![
            ("inheritance.source_instance_id", "features.instance_id"),
            ("inheritance.source_name", "features.name"),
        ],
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_inherit",
            "1",
            0,
            RuleHead::Relation("inferred.feature_candidates".to_owned()),
            project(
                inherited,
                vec![
                    ("instance_id", E::col("inheritance.instance_id")),
                    ("name", E::col("inheritance.name")),
                    ("source_owner_id", E::col("features.source_owner_id")),
                    ("source_name", E::col("features.source_name")),
                    ("value", E::col("features.value")),
                    ("derivation_id", E::col("inheritance.derivation_id")),
                ],
            ),
        )
        .assertions("provenance.feature_candidate_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_resolve",
            "1",
            1,
            RuleHead::Relation("inferred.instance_features".to_owned()),
            project(
                scan("inferred.feature_candidates", "features"),
                vec![
                    ("instance_id", E::col("instance_id")),
                    ("name", E::col("name")),
                    ("value", E::col("value")),
                    ("derivation_id", E::col("derivation_id")),
                ],
            ),
        )
        .assertions("provenance.instance_feature_assertions"),
    );
    let requests = join(
        scan("normalized.instance_bindings", "instances"),
        scan("normalized.template_features", "declarations"),
        vec![("instances.template_id", "declarations.template_id")],
    );
    let requests = project(
        requests,
        vec![
            ("instance_id", E::col("instances.instance_id")),
            ("name", E::col("declarations.name")),
            ("derivation_id", E::col("instances.derivation_id")),
        ],
    );
    let missing = P::AntiJoin {
        left: Box::new(requests),
        right: Box::new(scan("inferred.instance_features", "features")),
        keys: (vec![
            ("instance_id", "features.instance_id"),
            ("name", "features.name"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    let unknown = P::Filter {
        input: Box::new(missing),
        predicate: E::Lit(Cell::Null),
    };
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_missing",
            "1",
            2,
            RuleHead::Relation("inferred.feature_requirements".to_owned()),
            unknown,
        )
        .assertions("provenance.feature_requirement_assertions")
        .stratified_negation(),
    );
}
