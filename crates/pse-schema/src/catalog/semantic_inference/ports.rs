// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ports retain full state collections; existence and membership are relational facts.
use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, equals, join,
    project, provenance, relation, scan,
};
#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (head, candidate, assertions) in [
        ("inferred.ports", "port_candidates", "port_assertions"),
        (
            "inferred.port_state_targets",
            "port_state_candidates",
            "port_state_assertions",
        ),
        (
            "inferred.port_state_domains",
            "port_state_domain_candidates",
            "port_state_domain_assertions",
        ),
        (
            "inferred.port_members",
            "port_member_candidates",
            "port_member_assertions",
        ),
        (
            "inferred.port_member_domains",
            "port_member_domain_candidates",
            "port_member_domain_assertions",
        ),
    ] {
        let Some(source) = builder
            .declared_relations()
            .iter()
            .find(|spec| spec.key.qualified_name() == head)
            .cloned()
        else {
            continue;
        };
        let Some(keys) = &source.primary_key else {
            continue;
        };
        let mut columns = source.columns.clone();
        if candidate == "port_candidates" {
            columns.push(column("guard_outcome", T::enumeration("TruthValue")));
            columns.push(provenance());
        }
        relation(
            builder,
            N::Inferred,
            candidate,
            S::Derived,
            keys,
            columns,
            "Replayed prospective port declaration and exact finite state/member context; RulePlans decide actual membership.",
        );
        assertion(builder, head, assertions);
    }
    relation(
        builder,
        N::Inferred,
        "unbound_port_targets",
        S::Derived,
        &["port_id", "state_index"],
        vec![
            column("port_id", T::id()),
            column(
                "state_index",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            column("state_instance_id", T::id()),
            provenance(),
        ],
        "Enabled port target has no actual admitted instance; publication is refused.",
    );
    assertion(
        builder,
        "inferred.unbound_port_targets",
        "unbound_port_assertions",
    );
    let enabled = || P::Filter {
        input: Box::new(scan("inferred.port_candidates", "ports")),
        predicate: equals(E::col("guard_outcome"), Cell::Enum("true")),
    };
    let eligible_targets = || {
        let targets = join(
            scan("inferred.port_state_candidates", "targets"),
            scan("inferred.port_state_domain_candidates", "domains"),
            vec![("targets.port_id", "domains.port_id")],
        );
        join(
            targets,
            scan("inferred.valid_index_tuples", "valid"),
            vec![
                ("domains.product_id", "valid.product_id"),
                ("targets.state_index", "valid.tuple"),
            ],
        )
    };
    let expected = join(
        eligible_targets(),
        enabled(),
        vec![("targets.port_id", "ports.port_id")],
    );
    let missing = P::AntiJoin {
        left: Box::new(expected),
        right: Box::new(scan("inferred.instances", "instances")),
        keys: (vec![("targets.state_instance_id", "instances.instance_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    rule(
        builder,
        "P5.unbound_port_target",
        10,
        "inferred.unbound_port_targets",
        "provenance.unbound_port_assertions",
        missing,
        vec![
            ("port_id", E::col("targets.port_id")),
            ("state_index", E::col("targets.state_index")),
            ("state_instance_id", E::col("targets.state_instance_id")),
            ("derivation_id", E::col("targets.derivation_id")),
        ],
    );
    let nonempty = P::Distinct(Box::new(project(
        eligible_targets(),
        vec![("nonempty_port_id", E::col("targets.port_id"))],
    )));
    let owners = join(
        enabled(),
        scan("inferred.instances", "instances"),
        vec![("ports.instance_id", "instances.instance_id")],
    );
    let actual = P::AntiJoin {
        left: Box::new(join(
            owners,
            nonempty,
            vec![("ports.port_id", "nonempty_port_id")],
        )),
        right: Box::new(scan("inferred.unbound_port_targets", "missing")),
        keys: (vec![("ports.port_id", "missing.port_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    rule(
        builder,
        "P5.ports",
        11,
        "inferred.ports",
        "provenance.port_assertions",
        actual,
        vec![
            ("port_id", E::col("ports.port_id")),
            ("instance_id", E::col("ports.instance_id")),
            ("name", E::col("ports.name")),
            ("kind", E::col("ports.kind")),
            ("direction", E::col("ports.direction")),
            ("state_instance_id", E::col("ports.state_instance_id")),
        ],
    );
    for (head, candidate, name, assertions) in [
        (
            "inferred.port_state_targets",
            "inferred.port_state_candidates",
            "P5.port_state_targets",
            "provenance.port_state_assertions",
        ),
        (
            "inferred.port_state_domains",
            "inferred.port_state_domain_candidates",
            "P5.port_state_domains",
            "provenance.port_state_domain_assertions",
        ),
        (
            "inferred.port_members",
            "inferred.port_member_candidates",
            "P5.port_members",
            "provenance.port_member_assertions",
        ),
        (
            "inferred.port_member_domains",
            "inferred.port_member_domain_candidates",
            "P5.port_member_domains",
            "provenance.port_member_domain_assertions",
        ),
    ] {
        let Some(spec) = builder
            .declared_relations()
            .iter()
            .find(|spec| spec.key.qualified_name() == head)
            .cloned()
        else {
            continue;
        };
        let source = if head == "inferred.port_state_targets" {
            let actual = join(
                eligible_targets(),
                scan("inferred.instances", "states"),
                vec![("targets.state_instance_id", "states.instance_id")],
            );
            project(
                actual,
                spec.columns
                    .iter()
                    .map(|column| {
                        (
                            column.name(),
                            match column.name() {
                                "port_id" => E::col("targets.port_id"),
                                "state_index" => E::col("targets.state_index"),
                                "state_instance_id" => E::col("targets.state_instance_id"),
                                "derivation_id" => E::col("targets.derivation_id"),
                                _ => E::col(column.name().to_owned()),
                            },
                        )
                    })
                    .collect(),
            )
        } else {
            scan(candidate, "candidates")
        };
        let ports = project(
            scan("inferred.ports", "ports"),
            vec![("actual_port_id", E::col("port_id"))],
        );
        let joined = join(source, ports, vec![("port_id", "actual_port_id")]);
        rule(
            builder,
            name,
            12,
            head,
            assertions,
            joined,
            spec.columns
                .iter()
                .map(|column| (column.name(), E::col(column.name().to_owned())))
                .collect(),
        );
    }
    let keys = ["port_id", "state_index"];
    super::super::inv::declare(
        builder,
        "inferred.unbound_port_targets",
        "all_state_targets_bound",
        crate::model::InvariantKind::Check,
        &keys,
        super::super::inv::project(
            super::super::inv::scan("inferred.unbound_port_targets", "subject"),
            &keys,
        ),
        "Every enabled port target must be an actual admitted instance.",
    );
}
fn rule(
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
