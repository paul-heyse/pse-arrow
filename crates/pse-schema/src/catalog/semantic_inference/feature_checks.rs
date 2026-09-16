// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Requires and excludes remain explicit four-valued predicates.
use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, equals, join,
    project, provenance, relation, scan,
};

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
    for (name, kind) in [
        ("P4.feature_requires", "requires"),
        ("P4.feature_excludes", "excludes"),
    ] {
        let declarations = P::Filter {
            input: Box::new(scan("normalized.template_feature_rules", "declarations")),
            predicate: equals(E::col("rule"), Cell::Enum(kind)),
        };
        let checks = join(
            declarations,
            scan("normalized.instance_bindings", "instances"),
            vec![("declarations.template_id", "instances.template_id")],
        );
        let checks = project(
            checks,
            vec![
                ("owner_id", E::col("instances.instance_id")),
                ("rule", E::col("declarations.rule")),
                ("antecedent", E::col("declarations.antecedent")),
                ("consequent", E::col("declarations.consequent")),
            ],
        );
        let checks = join(
            checks,
            scan("inferred.instance_features", "left_feature"),
            vec![
                ("owner_id", "left_feature.instance_id"),
                ("antecedent", "left_feature.name"),
            ],
        );
        let checks = join(
            checks,
            scan("inferred.instance_features", "right_feature"),
            vec![
                ("owner_id", "right_feature.instance_id"),
                ("consequent", "right_feature.name"),
            ],
        );
        let boolean = |name| E::Field {
            expr: Box::new(E::col(name)),
            name: "boolean".into(),
        };
        let left = boolean("left_feature.value");
        let right = boolean("right_feature.value");
        let predicate = if kind == "requires" {
            E::Or(vec![E::Not(Box::new(left)), right])
        } else {
            E::Not(Box::new(E::And(vec![left, right])))
        };
        let plan = project(
            P::Filter {
                input: Box::new(checks),
                predicate,
            },
            vec![
                ("instance_id", E::col("owner_id")),
                ("rule", E::col("rule")),
                ("antecedent", E::col("antecedent")),
                ("consequent", E::col("consequent")),
                ("derivation_id", E::col("left_feature.derivation_id")),
            ],
        );
        builder.declare_rule(
            RuleDecl::new(
                name,
                "1",
                2,
                RuleHead::Relation("inferred.feature_checks".to_owned()),
                plan,
            )
            .assertions("provenance.feature_check_assertions"),
        );
    }
}
