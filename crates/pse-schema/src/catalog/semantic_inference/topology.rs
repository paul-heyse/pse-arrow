// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Connection admission and actual cut sets are declared relational operators.
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
        "connection_violations",
        S::Derived,
        &["connection_id", "reason"],
        vec![
            column("connection_id", T::id()),
            column("reason", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Exact missing endpoint or incompatible member contracts prevent topology publication.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_port_states",
        S::Derived,
        &["scope_id", "port_id", "state_index"],
        vec![
            column("scope_id", T::id()),
            column("port_id", T::id()),
            column(
                "state_index",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            provenance(),
        ],
        "Actual state tuples included by their state, owning instance, or explicit port membership.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_port_decisions",
        S::Derived,
        &["scope_id", "port_id", "state_index"],
        vec![
            column("scope_id", T::id()),
            column("port_id", T::id()),
            column(
                "state_index",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            column("included", T::native(arrow_schema::DataType::Boolean)),
            provenance(),
        ],
        "Complete endpoint state membership, retaining false as explicit positive data.",
    );
    for (head, name) in [
        (
            "inferred.connection_violations",
            "connection_violation_assertions",
        ),
        ("inferred.topology_edges", "topology_edge_assertions"),
        ("inferred.scope_port_states", "scope_port_state_assertions"),
        (
            "inferred.scope_port_decisions",
            "scope_port_decision_assertions",
        ),
        (
            "inferred.boundary_crossings",
            "boundary_crossing_assertions",
        ),
    ] {
        assertion(builder, head, name);
    }
    for (name, endpoint, reason) in [
        (
            "P5.connection_from_missing",
            "from_port_id",
            "from_port_missing",
        ),
        ("P5.connection_to_missing", "to_port_id", "to_port_missing"),
    ] {
        let missing = P::AntiJoin {
            left: Box::new(scan("normalized.connections", "connections")),
            right: Box::new(scan("inferred.ports", "ports")),
            keys: (vec![(endpoint, "port_id")])
                .into_iter()
                .map(|(left, right)| (left.into(), right.into()))
                .collect(),
        };
        violation(builder, name, missing, "connections.connection_id", reason);
    }
    let endpoints = endpoints();
    let directions = E::And(vec![
        E::Or(vec![
            equals(E::col("from.direction"), Cell::Enum("outlet")),
            equals(E::col("from.direction"), Cell::Enum("bidirectional")),
        ]),
        E::Or(vec![
            equals(E::col("to.direction"), Cell::Enum("inlet")),
            equals(E::col("to.direction"), Cell::Enum("bidirectional")),
        ]),
    ]);
    violation(
        builder,
        "P5.connection_direction",
        filter(endpoints.clone(), E::Not(Box::new(directions))),
        "connections.connection_id",
        "direction_mismatch",
    );
    violation(
        builder,
        "P5.connection_kind",
        filter(
            endpoints.clone(),
            E::IsDistinctFrom(Box::new(E::col("from.kind")), Box::new(E::col("to.kind"))),
        ),
        "connections.connection_id",
        "port_kind_mismatch",
    );
    let from_members = join(
        scan("normalized.connections", "connections"),
        scan("inferred.port_members", "members"),
        vec![("connections.from_port_id", "members.port_id")],
    );
    let from_missing = P::AntiJoin {
        left: Box::new(from_members.clone()),
        right: Box::new(scan("inferred.port_members", "other")),
        keys: (vec![
            ("connections.to_port_id", "other.port_id"),
            ("members.ordinal", "other.ordinal"),
            ("members.symbol_group", "other.symbol_group"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    violation(
        builder,
        "P5.connection_member_to_missing",
        from_missing,
        "connections.connection_id",
        "member_missing_to",
    );
    let to_members = join(
        scan("normalized.connections", "connections"),
        scan("inferred.port_members", "members"),
        vec![("connections.to_port_id", "members.port_id")],
    );
    let to_missing = P::AntiJoin {
        left: Box::new(to_members),
        right: Box::new(scan("inferred.port_members", "other")),
        keys: (vec![
            ("connections.from_port_id", "other.port_id"),
            ("members.ordinal", "other.ordinal"),
            ("members.symbol_group", "other.symbol_group"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    violation(
        builder,
        "P5.connection_member_from_missing",
        to_missing,
        "connections.connection_id",
        "member_missing_from",
    );
    let paired = join(
        from_members,
        scan("inferred.port_members", "other"),
        vec![
            ("connections.to_port_id", "other.port_id"),
            ("members.ordinal", "other.ordinal"),
            ("members.symbol_group", "other.symbol_group"),
        ],
    );
    let paired = join(
        join(
            paired,
            scan("inferred.port_member_domains", "left_domains"),
            vec![
                ("members.port_id", "left_domains.port_id"),
                ("members.ordinal", "left_domains.ordinal"),
            ],
        ),
        scan("inferred.port_member_domains", "right_domains"),
        vec![
            ("other.port_id", "right_domains.port_id"),
            ("other.ordinal", "right_domains.ordinal"),
        ],
    );
    violation(
        builder,
        "P5.connection_member_domains",
        filter(
            paired.clone(),
            E::IsDistinctFrom(
                Box::new(E::col("left_domains.domain_ids")),
                Box::new(E::col("right_domains.domain_ids")),
            ),
        ),
        "connections.connection_id",
        "member_domain_mismatch",
    );
    let quantities = join(
        join(
            paired,
            scan("reference.quantity_types", "left_quantity"),
            vec![("members.quantity_type_id", "left_quantity.quantity_type_id")],
        ),
        scan("reference.quantity_types", "right_quantity"),
        vec![("other.quantity_type_id", "right_quantity.quantity_type_id")],
    );
    let physical = [
        (
            "left_quantity.quantity_kind_id",
            "right_quantity.quantity_kind_id",
        ),
        ("left_quantity.basis_id", "right_quantity.basis_id"),
        (
            "left_quantity.reference_state_id",
            "right_quantity.reference_state_id",
        ),
        ("left_quantity.scale_kind", "right_quantity.scale_kind"),
        ("left_quantity.shape", "right_quantity.shape"),
        ("left_quantity.subject_kind", "right_quantity.subject_kind"),
    ];
    violation(
        builder,
        "P5.connection_member_physical",
        filter(
            quantities,
            E::Or(
                physical
                    .iter()
                    .map(|(a, b)| E::IsDistinctFrom(Box::new(E::col(*a)), Box::new(E::col(*b))))
                    .collect(),
            ),
        ),
        "connections.connection_id",
        "member_physical_mismatch",
    );
    let actual = P::AntiJoin {
        left: Box::new(endpoints),
        right: Box::new(scan("inferred.connection_violations", "invalid")),
        keys: (vec![("connections.connection_id", "invalid.connection_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    emit(
        builder,
        "P5.topology",
        18,
        "inferred.topology_edges",
        "provenance.topology_edge_assertions",
        actual,
        vec![
            ("from_instance_id", E::col("from.instance_id")),
            ("to_instance_id", E::col("to.instance_id")),
            ("connection_id", E::col("connections.connection_id")),
        ],
    );
    cut_sets(builder);
    let keys = ["connection_id", "reason"];
    super::super::inv::declare(
        builder,
        "inferred.connection_violations",
        "connections_admitted",
        crate::model::InvariantKind::Check,
        &keys,
        super::super::inv::project(scan("inferred.connection_violations", "violations"), &keys),
        "All connection endpoints, domains, ordered members and full physical contracts must be compatible.",
    );
}
fn endpoints() -> P {
    join(
        join(
            scan("normalized.connections", "connections"),
            scan("inferred.ports", "from"),
            vec![("connections.from_port_id", "from.port_id")],
        ),
        scan("inferred.ports", "to"),
        vec![("connections.to_port_id", "to.port_id")],
    )
}
#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
fn cut_sets(builder: &mut RegistryBuilder) {
    let targets = join(
        scan("inferred.port_state_targets", "targets"),
        scan("inferred.ports", "ports"),
        vec![("targets.port_id", "ports.port_id")],
    );
    for (name, subject) in [
        ("P5.scope_port_explicit", "targets.port_id"),
        ("P5.scope_port_owner", "ports.instance_id"),
        ("P5.scope_port_state", "targets.state_instance_id"),
    ] {
        let included = join(
            targets.clone(),
            scan("inferred.scope_members", "members"),
            vec![(subject, "members.entity_id")],
        );
        emit(
            builder,
            name,
            16,
            "inferred.scope_port_states",
            "provenance.scope_port_state_assertions",
            included,
            vec![
                ("scope_id", E::col("members.scope_id")),
                ("port_id", E::col("targets.port_id")),
                ("state_index", E::col("targets.state_index")),
                ("derivation_id", E::col("members.derivation_id")),
            ],
        );
    }
    let scopes = project(
        scan("inferred.resolved_scopes", "scopes"),
        vec![
            ("scope_id", E::col("scope_id")),
            ("scope_universe", E::Lit(Cell::Bool(true))),
        ],
    );
    let targets = project(
        targets,
        vec![
            ("port_id", E::col("targets.port_id")),
            ("state_index", E::col("targets.state_index")),
            ("target_universe", E::Lit(Cell::Bool(true))),
            ("derivation_id", E::col("targets.derivation_id")),
        ],
    );
    let universe = join(scopes, targets, vec![("scope_universe", "target_universe")]);
    let no = P::AntiJoin {
        left: Box::new(universe),
        right: Box::new(scan("inferred.scope_port_states", "included")),
        keys: (vec![
            ("scope_id", "included.scope_id"),
            ("port_id", "included.port_id"),
            ("state_index", "included.state_index"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    for (name, input, included) in [
        (
            "P5.scope_port_yes",
            scan("inferred.scope_port_states", "included"),
            true,
        ),
        ("P5.scope_port_no", no, false),
    ] {
        emit(
            builder,
            name,
            17,
            "inferred.scope_port_decisions",
            "provenance.scope_port_decision_assertions",
            input,
            vec![
                ("scope_id", E::col("scope_id")),
                ("port_id", E::col("port_id")),
                ("state_index", E::col("state_index")),
                ("included", E::Lit(Cell::Bool(included))),
                ("derivation_id", E::col("derivation_id")),
            ],
        );
    }
    let connections = join(
        scan("inferred.topology_edges", "edges"),
        scan("normalized.connections", "connections"),
        vec![("edges.connection_id", "connections.connection_id")],
    );
    let endpoints = join(
        join(
            connections,
            scan("inferred.scope_port_decisions", "from"),
            vec![("connections.from_port_id", "from.port_id")],
        ),
        scan("inferred.scope_port_decisions", "to"),
        vec![
            ("connections.to_port_id", "to.port_id"),
            ("from.scope_id", "to.scope_id"),
            ("from.state_index", "to.state_index"),
        ],
    );
    for (name, from, to, class) in [
        ("P5.boundary_internal", true, true, "internal"),
        ("P5.boundary_external", false, false, "external"),
        ("P5.boundary_inbound", false, true, "inbound"),
        ("P5.boundary_outbound", true, false, "outbound"),
    ] {
        let input = filter(
            endpoints.clone(),
            E::And(vec![
                equals(E::col("from.included"), Cell::Bool(from)),
                equals(E::col("to.included"), Cell::Bool(to)),
            ]),
        );
        emit(
            builder,
            name,
            19,
            "inferred.boundary_crossings",
            "provenance.boundary_crossing_assertions",
            input,
            vec![
                ("scope_id", E::col("from.scope_id")),
                ("connection_id", E::col("connections.connection_id")),
                ("classification", E::Lit(Cell::Enum(class))),
                ("derivation_id", E::col("from.derivation_id")),
            ],
        );
    }
}
fn filter(input: P, predicate: E) -> P {
    P::Filter {
        input: Box::new(input),
        predicate,
    }
}
fn violation(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    id: &'static str,
    reason: &'static str,
) {
    emit(
        builder,
        name,
        17,
        "inferred.connection_violations",
        "provenance.connection_violation_assertions",
        input,
        vec![
            ("connection_id", E::col(id)),
            ("reason", E::Lit(Cell::text(reason))),
            ("derivation_id", E::col(id)),
        ],
    );
}
fn emit(
    builder: &mut RegistryBuilder,
    name: &'static str,
    stratum: u16,
    head: &'static str,
    assertions: &'static str,
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
        .assertions(assertions)
        .stratified_negation(),
    );
}
