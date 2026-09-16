// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete positive Boolean selector decisions admit arbitrary nested difference.
use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, equals, join,
    project, provenance, relation, scan,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    let context_columns = || {
        vec![
            column("scope_id", T::id()),
            column("scope_decl_id", T::id()),
            column("owner_instance_id", T::id()).optional(),
            provenance(),
        ]
    };
    relation(
        builder,
        N::Inferred,
        "scope_candidates",
        S::Derived,
        &["scope_id"],
        context_columns(),
        "Replayed finite scope identity construction from actual declarations and prospective instances.",
    );
    relation(
        builder,
        N::Inferred,
        "resolved_scopes",
        S::Derived,
        &["scope_id"],
        context_columns(),
        "Actual global or instance-bound scope contexts.",
    );
    relation(
        builder,
        N::Inferred,
        "selector_contexts",
        S::Derived,
        &["scope_id", "node_id"],
        vec![
            column("scope_id", T::id()),
            column("node_id", T::id()),
            column("op", T::enumeration("SelectorNodeOp")),
            column("left_node_id", T::id()).optional(),
            column("right_node_id", T::id()).optional(),
            column("target_entity_id", T::id()).optional(),
            column("target_kind", T::enumeration("EntityKind")).optional(),
            column("constant", T::native(arrow_schema::DataType::Boolean)).optional(),
            provenance(),
        ],
        "Replayed selector structural contexts; relative parameter leaves bind exact typed configured identities.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_entities",
        S::Derived,
        &["entity_id"],
        vec![
            column("entity_id", T::id()),
            column("kind", T::enumeration("EntityKind")),
            provenance(),
        ],
        "Complete actual instance/port universe at the topology boundary.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_reachability",
        S::Derived,
        &["ancestor_id", "entity_id"],
        vec![
            column("ancestor_id", T::id()),
            column("entity_id", T::id()),
            provenance(),
        ],
        "Actual containment of instances and their ports, separate from physical flow.",
    );
    relation(
        builder,
        N::Inferred,
        "selector_decisions",
        S::Derived,
        &["scope_id", "node_id", "entity_id"],
        vec![
            column("scope_id", T::id()),
            column("node_id", T::id()),
            column("entity_id", T::id()),
            column("included", T::native(arrow_schema::DataType::Boolean)),
            provenance(),
        ],
        "Complete Boolean membership facts; false is positive data, so nested set difference remains monotone.",
    );
    for (head, assertion_name) in [
        ("inferred.resolved_scopes", "scope_assertions"),
        ("inferred.scope_bindings", "scope_binding_assertions"),
        ("inferred.scope_entities", "scope_entity_assertions"),
        (
            "inferred.scope_reachability",
            "scope_reachability_assertions",
        ),
        (
            "inferred.selector_decisions",
            "selector_decision_assertions",
        ),
        ("inferred.scope_members", "scope_member_assertions"),
    ] {
        assertion(builder, head, assertion_name);
    }
    let globals = filter(
        scan("inferred.scope_candidates", "scopes"),
        E::IsNull(Box::new(E::col("owner_instance_id"))),
    );
    emit(
        builder,
        "P5.global_scopes",
        13,
        "inferred.resolved_scopes",
        "provenance.scope_assertions",
        globals,
        vec![
            ("scope_id", E::col("scopes.scope_id")),
            ("scope_decl_id", E::col("scopes.scope_decl_id")),
            ("owner_instance_id", E::col("scopes.owner_instance_id")),
            ("derivation_id", E::col("scopes.derivation_id")),
        ],
    );
    let actual = join(
        scan("inferred.scope_candidates", "scopes"),
        scan("inferred.instances", "instances"),
        vec![("scopes.owner_instance_id", "instances.instance_id")],
    );
    emit(
        builder,
        "P5.relative_scopes",
        13,
        "inferred.resolved_scopes",
        "provenance.scope_assertions",
        actual,
        vec![
            ("scope_id", E::col("scopes.scope_id")),
            ("scope_decl_id", E::col("scopes.scope_decl_id")),
            ("owner_instance_id", E::col("scopes.owner_instance_id")),
            ("derivation_id", E::col("scopes.derivation_id")),
        ],
    );
    let bound = filter(
        scan("inferred.resolved_scopes", "scopes"),
        E::IsNotNull(Box::new(E::col("owner_instance_id"))),
    );
    emit(
        builder,
        "P5.scope_bindings",
        14,
        "inferred.scope_bindings",
        "provenance.scope_binding_assertions",
        bound,
        vec![
            ("scope_decl_id", E::col("scope_decl_id")),
            ("owner_instance_id", E::col("owner_instance_id")),
            ("scope_id", E::col("scope_id")),
            ("derivation_id", E::col("derivation_id")),
        ],
    );
    emit(
        builder,
        "P5.scope_instance_universe",
        13,
        "inferred.scope_entities",
        "provenance.scope_entity_assertions",
        scan("inferred.instances", "instances"),
        vec![
            ("entity_id", E::col("instance_id")),
            ("kind", E::Lit(Cell::Enum("instance"))),
            ("derivation_id", E::col("derivation_id")),
        ],
    );
    emit(
        builder,
        "P5.scope_port_universe",
        13,
        "inferred.scope_entities",
        "provenance.scope_entity_assertions",
        scan("inferred.ports", "ports"),
        vec![
            ("entity_id", E::col("port_id")),
            ("kind", E::Lit(Cell::Enum("port"))),
            ("derivation_id", E::col("port_id")),
        ],
    );
    emit(
        builder,
        "P5.scope_instance_descendants",
        13,
        "inferred.scope_reachability",
        "provenance.scope_reachability_assertions",
        scan("inferred.instance_reachability", "paths"),
        vec![
            ("ancestor_id", E::col("ancestor_id")),
            ("entity_id", E::col("descendant_id")),
            ("derivation_id", E::col("derivation_id")),
        ],
    );
    emit(
        builder,
        "P5.scope_direct_ports",
        13,
        "inferred.scope_reachability",
        "provenance.scope_reachability_assertions",
        scan("inferred.ports", "ports"),
        vec![
            ("ancestor_id", E::col("instance_id")),
            ("entity_id", E::col("port_id")),
            ("derivation_id", E::col("port_id")),
        ],
    );
    let port_descendants = join(
        scan("inferred.instance_reachability", "paths"),
        scan("inferred.ports", "ports"),
        vec![("paths.descendant_id", "ports.instance_id")],
    );
    emit(
        builder,
        "P5.scope_descendant_ports",
        13,
        "inferred.scope_reachability",
        "provenance.scope_reachability_assertions",
        port_descendants,
        vec![
            ("ancestor_id", E::col("paths.ancestor_id")),
            ("entity_id", E::col("ports.port_id")),
            ("derivation_id", E::col("paths.derivation_id")),
        ],
    );
    let contexts = join(
        scan("inferred.selector_contexts", "nodes"),
        scan("inferred.resolved_scopes", "scopes"),
        vec![("nodes.scope_id", "scopes.scope_id")],
    );
    for (op, name, decision) in [
        (
            "constant",
            "P5.selector_constant",
            E::IsTrue(Box::new(E::col("constant"))),
        ),
        (
            "include",
            "P5.selector_include",
            E::IsNotDistinctFrom(
                Box::new(E::col("entity_id")),
                Box::new(E::col("target_entity_id")),
            ),
        ),
        (
            "exclude",
            "P5.selector_exclude",
            E::IsDistinctFrom(
                Box::new(E::col("entity_id")),
                Box::new(E::col("target_entity_id")),
            ),
        ),
        (
            "self",
            "P5.selector_self",
            E::IsNotDistinctFrom(
                Box::new(E::col("entity_id")),
                Box::new(E::col("target_entity_id")),
            ),
        ),
        (
            "instance_parameter",
            "P5.selector_parameter",
            E::IsNotDistinctFrom(
                Box::new(E::col("entity_id")),
                Box::new(E::col("target_entity_id")),
            ),
        ),
        (
            "kind_is",
            "P5.selector_kind",
            E::IsNotDistinctFrom(Box::new(E::col("kind")), Box::new(E::col("target_kind"))),
        ),
    ] {
        let leaves = filter(contexts.clone(), equals(E::col("nodes.op"), Cell::Enum(op)));
        decision_rule(builder, name, universe(leaves), decision);
    }
    let descendants = universe(filter(
        contexts.clone(),
        equals(E::col("nodes.op"), Cell::Enum("descendant_of")),
    ));
    let yes = join(
        descendants.clone(),
        scan("inferred.scope_reachability", "reach"),
        vec![
            ("target_entity_id", "reach.ancestor_id"),
            ("entity_id", "reach.entity_id"),
        ],
    );
    decision_rule(
        builder,
        "P5.selector_descendant_yes",
        project(
            yes,
            vec![
                ("scope_id", E::col("scope_id")),
                ("node_id", E::col("node_id")),
                ("entity_id", E::col("reach.entity_id")),
                ("derivation_id", E::col("reach.derivation_id")),
            ],
        ),
        E::Lit(Cell::Bool(true)),
    );
    let no = P::AntiJoin {
        left: Box::new(descendants),
        right: Box::new(scan("inferred.scope_reachability", "reach")),
        keys: (vec![
            ("target_entity_id", "reach.ancestor_id"),
            ("entity_id", "reach.entity_id"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    decision_rule(
        builder,
        "P5.selector_descendant_no",
        no,
        E::Lit(Cell::Bool(false)),
    );
    let child = |port| scan("inferred.selector_decisions", port);
    for (op, name, included) in [
        (
            "union",
            "P5.selector_union",
            E::Or(vec![E::col("left.included"), E::col("right.included")]),
        ),
        (
            "intersection",
            "P5.selector_intersection",
            E::And(vec![E::col("left.included"), E::col("right.included")]),
        ),
        (
            "difference",
            "P5.selector_difference",
            E::And(vec![
                E::col("left.included"),
                E::Not(Box::new(E::col("right.included"))),
            ]),
        ),
    ] {
        let nodes = filter(contexts.clone(), equals(E::col("nodes.op"), Cell::Enum(op)));
        let input = join(
            join(
                nodes,
                child("left"),
                vec![
                    ("nodes.scope_id", "left.scope_id"),
                    ("nodes.left_node_id", "left.node_id"),
                ],
            ),
            child("right"),
            vec![
                ("nodes.scope_id", "right.scope_id"),
                ("nodes.right_node_id", "right.node_id"),
                ("left.entity_id", "right.entity_id"),
            ],
        );
        emit(
            builder,
            name,
            14,
            "inferred.selector_decisions",
            "provenance.selector_decision_assertions",
            input,
            vec![
                ("scope_id", E::col("nodes.scope_id")),
                ("node_id", E::col("nodes.node_id")),
                ("entity_id", E::col("left.entity_id")),
                ("included", included),
                ("derivation_id", E::col("nodes.derivation_id")),
            ],
        );
    }
    let identity = join(
        filter(contexts, equals(E::col("nodes.op"), Cell::Enum("identity"))),
        child("child"),
        vec![
            ("nodes.scope_id", "child.scope_id"),
            ("nodes.left_node_id", "child.node_id"),
        ],
    );
    emit(
        builder,
        "P5.selector_identity",
        14,
        "inferred.selector_decisions",
        "provenance.selector_decision_assertions",
        identity,
        vec![
            ("scope_id", E::col("nodes.scope_id")),
            ("node_id", E::col("nodes.node_id")),
            ("entity_id", E::col("child.entity_id")),
            ("included", E::col("child.included")),
            ("derivation_id", E::col("nodes.derivation_id")),
        ],
    );
    let roots = join(
        scan("normalized.selector_roots", "roots"),
        scan("inferred.resolved_scopes", "scopes"),
        vec![("roots.scope_decl_id", "scopes.scope_decl_id")],
    );
    let members = join(
        roots,
        scan("inferred.selector_decisions", "decisions"),
        vec![
            ("scopes.scope_id", "decisions.scope_id"),
            ("roots.node_id", "decisions.node_id"),
        ],
    );
    emit(
        builder,
        "P5.scope_members",
        15,
        "inferred.scope_members",
        "provenance.scope_member_assertions",
        filter(members, E::col("decisions.included")),
        vec![
            ("scope_id", E::col("scopes.scope_id")),
            ("entity_id", E::col("decisions.entity_id")),
            ("derivation_id", E::col("decisions.derivation_id")),
        ],
    );
}
fn filter(input: P, predicate: E) -> P {
    P::Filter {
        input: Box::new(input),
        predicate,
    }
}
fn emit(
    builder: &mut RegistryBuilder,
    name: &'static str,
    stratum: u16,
    head: &'static str,
    assertion: &'static str,
    input: P,
    columns: Vec<(&str, E)>,
) {
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            stratum,
            RuleHead::Relation(head.to_owned()),
            P::Union(vec![project(input, columns)]),
        )
        .assertions(assertion)
        .stratified_negation(),
    );
}
fn decision_rule(builder: &mut RegistryBuilder, name: &'static str, input: P, included: E) {
    emit(
        builder,
        name,
        14,
        "inferred.selector_decisions",
        "provenance.selector_decision_assertions",
        input,
        vec![
            ("scope_id", E::col("scope_id")),
            ("node_id", E::col("node_id")),
            ("entity_id", E::col("entity_id")),
            ("included", included),
            ("derivation_id", E::col("derivation_id")),
        ],
    );
}
fn universe(input: P) -> P {
    let input = project(
        input,
        vec![
            ("scope_id", E::col("nodes.scope_id")),
            ("node_id", E::col("nodes.node_id")),
            ("target_entity_id", E::col("nodes.target_entity_id")),
            ("target_kind", E::col("nodes.target_kind")),
            ("constant", E::col("nodes.constant")),
            ("derivation_id", E::col("nodes.derivation_id")),
            ("left_universe", E::Lit(Cell::Bool(true))),
        ],
    );
    let entities = project(
        scan("inferred.scope_entities", "entities"),
        vec![
            ("entity_id", E::col("entity_id")),
            ("kind", E::col("kind")),
            ("right_universe", E::Lit(Cell::Bool(true))),
        ],
    );
    join(input, entities, vec![("left_universe", "right_universe")])
}
