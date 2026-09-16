// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual instance membership and containment use finite positive relation closure.
use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, equals, join,
    project, provenance, relation, scan,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "instance_reachability",
        S::Derived,
        &["ancestor_id", "descendant_id"],
        vec![
            column("ancestor_id", T::id()),
            column("descendant_id", T::id()),
            provenance(),
        ],
        "Finite containment closure; P5 validates acyclicity and computes exact tree depths.",
    );
    assertion(builder, "inferred.instances", "instance_assertions");
    assertion(
        builder,
        "inferred.instance_reachability",
        "instance_reachability_assertions",
    );
    let prospective = scan("normalized.instance_bindings", "prospective");
    let prospective = join(
        prospective,
        scan("normalized.instance_binding_products", "binding_products"),
        vec![("prospective.instance_id", "binding_products.instance_id")],
    );
    let prospective = join(
        prospective,
        scan("inferred.valid_index_tuples", "valid"),
        vec![
            ("binding_products.product_id", "valid.product_id"),
            ("binding_products.index", "valid.tuple"),
        ],
    );
    guard_checks(builder, prospective.clone());
    let roots = P::Filter {
        input: Box::new(prospective.clone()),
        predicate: E::IsNull(Box::new(E::col("parent_instance_id"))),
    };
    emit(builder, "P5.instance_roots", roots, "prospective");
    let children = join(
        prospective,
        scan("inferred.instances", "parents"),
        vec![("prospective.parent_instance_id", "parents.instance_id")],
    );
    let unguarded = P::Filter {
        input: Box::new(children.clone()),
        predicate: E::And(vec![
            E::IsNull(Box::new(E::col("prospective.guard_source_id"))),
            E::IsNull(Box::new(E::col("prospective.guard_node_id"))),
        ]),
    };
    emit(builder, "P5.instance_unguarded", unguarded, "prospective");
    let guards = P::Filter {
        input: Box::new(scan("inferred.predicate_outcomes", "guards")),
        predicate: E::And(vec![
            equals(E::col("outcome"), Cell::Enum("true")),
            equals(E::ListLen(Box::new(E::col("index"))), Cell::U64(0)),
        ]),
    };
    let guarded = join(
        children,
        guards,
        vec![
            ("prospective.guard_source_id", "guards.source_id"),
            ("prospective.guard_node_id", "guards.predicate_id"),
            ("prospective.parent_instance_id", "guards.instance_id"),
        ],
    );
    emit(builder, "P5.instance_guarded", guarded, "prospective");
    let direct = P::Filter {
        input: Box::new(scan("inferred.instances", "instances")),
        predicate: E::IsNotNull(Box::new(E::col("parent_instance_id"))),
    };
    builder.declare_rule(
        RuleDecl::new(
            "P5.containment_direct",
            "1",
            9,
            RuleHead::Relation("inferred.instance_reachability".to_owned()),
            P::Union(vec![project(
                direct,
                vec![
                    ("ancestor_id", E::col("parent_instance_id")),
                    ("descendant_id", E::col("instance_id")),
                    ("derivation_id", E::col("derivation_id")),
                ],
            )]),
        )
        .assertions("provenance.instance_reachability_assertions"),
    );
    let closure = join(
        scan("inferred.instance_reachability", "paths"),
        scan("inferred.instances", "children"),
        vec![("paths.descendant_id", "children.parent_instance_id")],
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.containment_transitive",
            "1",
            9,
            RuleHead::Relation("inferred.instance_reachability".to_owned()),
            project(
                closure,
                vec![
                    ("ancestor_id", E::col("paths.ancestor_id")),
                    ("descendant_id", E::col("children.instance_id")),
                    ("derivation_id", E::col("children.derivation_id")),
                ],
            ),
        )
        .assertions("provenance.instance_reachability_assertions"),
    );
    depth(builder);
}

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
fn guard_checks(builder: &mut RegistryBuilder, prospective: P) {
    relation(
        builder,
        N::Inferred,
        "instance_guard_violations",
        S::Derived,
        &["instance_id", "reason"],
        vec![
            column("instance_id", T::id()),
            column("reason", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Missing, unresolved or malformed scalar child guards refuse publication rather than silently disabling instances.",
    );
    assertion(
        builder,
        "inferred.instance_guard_violations",
        "instance_guard_violation_assertions",
    );
    let children = join(
        prospective,
        scan("inferred.instances", "parents"),
        vec![("prospective.parent_instance_id", "parents.instance_id")],
    );
    let declared = P::Filter {
        input: Box::new(children.clone()),
        predicate: E::And(vec![
            E::IsNotNull(Box::new(E::col("prospective.guard_source_id"))),
            E::IsNotNull(Box::new(E::col("prospective.guard_node_id"))),
        ]),
    };
    let outcomes = P::Filter {
        input: Box::new(scan("inferred.predicate_outcomes", "guards")),
        predicate: equals(E::ListLen(Box::new(E::col("index"))), Cell::U64(0)),
    };
    let keys = vec![
        ("prospective.parent_instance_id", "guards.instance_id"),
        ("prospective.guard_source_id", "guards.source_id"),
        ("prospective.guard_node_id", "guards.predicate_id"),
    ];
    let missing = P::AntiJoin {
        left: Box::new(declared.clone()),
        right: Box::new(outcomes.clone()),
        keys: (keys.clone())
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    let unresolved = P::Filter {
        input: Box::new(join(declared, outcomes, keys)),
        predicate: E::And(vec![
            E::Not(Box::new(equals(
                E::col("guards.outcome"),
                Cell::Enum("true"),
            ))),
            E::Not(Box::new(equals(
                E::col("guards.outcome"),
                Cell::Enum("false"),
            ))),
        ]),
    };
    let malformed = P::Filter {
        input: Box::new(children),
        predicate: E::IsDistinctFrom(
            Box::new(E::IsNull(Box::new(E::col("prospective.guard_source_id")))),
            Box::new(E::IsNull(Box::new(E::col("prospective.guard_node_id")))),
        ),
    };
    for (name, input, reason) in [
        ("P5.guard_missing", missing, "missing"),
        ("P5.guard_unresolved", unresolved, "unresolved"),
        ("P5.guard_malformed", malformed, "malformed"),
    ] {
        builder.declare_rule(
            RuleDecl::new(
                name,
                "1",
                9,
                RuleHead::Relation("inferred.instance_guard_violations".to_owned()),
                P::Union(vec![project(
                    input,
                    vec![
                        ("instance_id", E::col("prospective.instance_id")),
                        ("reason", E::Lit(Cell::text(reason))),
                        ("derivation_id", E::col("prospective.derivation_id")),
                    ],
                )]),
            )
            .assertions("provenance.instance_guard_violation_assertions")
            .stratified_negation(),
        );
    }
    let keys = ["instance_id", "reason"];
    super::super::inv::declare(
        builder,
        "inferred.instance_guard_violations",
        "child_guards_decided",
        crate::model::InvariantKind::Check,
        &keys,
        super::super::inv::project(
            scan("inferred.instance_guard_violations", "violations"),
            &keys,
        ),
        "Every eligible child guard must have one complete scalar true/false outcome.",
    );
}

fn depth(builder: &mut RegistryBuilder) {
    assertion(
        builder,
        "inferred.instance_tree",
        "instance_tree_assertions",
    );
    let base = project(
        scan("inferred.instance_reachability", "paths"),
        vec![
            ("ancestor_id", E::col("ancestor_id")),
            ("descendant_id", E::col("descendant_id")),
            ("step_id", E::col("descendant_id")),
        ],
    );
    let intermediate = join(
        scan("inferred.instance_reachability", "prefix"),
        scan("inferred.instance_reachability", "suffix"),
        vec![("prefix.descendant_id", "suffix.ancestor_id")],
    );
    let intermediate = project(
        intermediate,
        vec![
            ("ancestor_id", E::col("prefix.ancestor_id")),
            ("descendant_id", E::col("suffix.descendant_id")),
            ("step_id", E::col("prefix.descendant_id")),
        ],
    );
    let steps = P::Distinct(Box::new(P::Union(vec![base, intermediate])));
    let depths = P::Aggregate {
        input: Box::new(steps),
        group: (vec!["ancestor_id", "descendant_id"])
            .into_iter()
            .map(Into::into)
            .collect(),
        aggregates: vec![crate::model::RuleAggregate {
            function: crate::model::RuleAggregateFn::Count,
            input: Some(E::col("step_id")),
            output_name: ("depth").into(),
            order_by: vec![],
            null_policy: crate::model::AggregateNullPolicy::Reject,
            empty_policy: crate::model::AggregateEmptyPolicy::Error,
        }],
    };
    builder.declare_rule(
        RuleDecl::new(
            "P5.containment_depth",
            "1",
            10,
            RuleHead::Relation("inferred.instance_tree".to_owned()),
            project(
                depths,
                vec![
                    ("ancestor_id", E::col("ancestor_id")),
                    ("descendant_id", E::col("descendant_id")),
                    ("depth", E::col("depth")),
                    ("derivation_id", E::col("ancestor_id")),
                ],
            ),
        )
        .assertions("provenance.instance_tree_assertions"),
    );
    let keys = ["ancestor_id", "descendant_id"];
    let cycles = P::Filter {
        input: Box::new(scan("inferred.instance_reachability", "paths")),
        predicate: E::cmp(
            crate::model::CmpOp::Eq,
            E::col("ancestor_id"),
            E::col("descendant_id"),
        ),
    };
    super::super::inv::declare(
        builder,
        "inferred.instance_reachability",
        "containment_acyclic",
        crate::model::InvariantKind::Check,
        &keys,
        super::super::inv::project(cycles, &keys),
        "Containment must be acyclic; physical recycle edges are a separate relation.",
    );
}
fn emit(builder: &mut RegistryBuilder, name: &'static str, input: P, _port: &str) {
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            8,
            RuleHead::Relation("inferred.instances".to_owned()),
            P::Union(vec![project(
                input,
                vec![
                    ("instance_id", E::col("prospective.instance_id")),
                    (
                        "parent_instance_id",
                        E::col("prospective.parent_instance_id"),
                    ),
                    ("template_id", E::col("prospective.template_id")),
                    ("path", E::col("prospective.path")),
                    ("index", E::col("prospective.index")),
                    ("derivation_id", E::col("prospective.derivation_id")),
                ],
            )]),
        )
        .assertions("provenance.instance_assertions"),
    );
}
