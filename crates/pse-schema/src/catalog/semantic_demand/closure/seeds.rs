// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source read coordinates, exact predicate tuples and actual selected scopes bind demands.
use super::{Cell, E, P, RegistryBuilder, eq, filter, join, null, project, rule, scan};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    active(builder);
    let direct = join(
        scan("inferred.demand_active_reads", "reads"),
        scan("normalized.property_demand_seeds", "seeds"),
        vec![("reads.seed_id", "seeds.seed_id")],
    );
    request(
        builder,
        "P6.direct_read_scope",
        direct,
        "seeds.scope_id",
        "reads.requester_instance_id",
        "seeds.property_kind_id",
    );
    let path = join(
        scan("inferred.demand_active_reads", "reads"),
        scan("normalized.property_path_demands", "demands"),
        vec![("reads.seed_id", "demands.demand_id")],
    );
    let mapped = join(
        path.clone(),
        scan("normalized.template_symbol_properties", "properties"),
        vec![("reads.symbol_decl_id", "properties.symbol_decl_id")],
    );
    request(
        builder,
        "P6.path_read_scope",
        mapped,
        "properties.scope_selector_id",
        "reads.owner_instance_id",
        "properties.property_kind_id",
    );
    resolve_scopes(builder);
    scope_targets(builder);
    provisions(builder, path);
}
fn request(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    scope: &'static str,
    owner: &'static str,
    property: &'static str,
) {
    rule(
        builder,
        name,
        1,
        "inferred.demand_scope_requests",
        "provenance.demand_scope_request_assertions",
        project(
            input,
            vec![
                ("read_id", E::col("reads.read_id")),
                ("seed_id", E::col("reads.seed_id")),
                (
                    "requester_instance_id",
                    E::col("reads.requester_instance_id"),
                ),
                ("scope_decl_id", E::col(scope)),
                ("owner_instance_id", E::col(owner)),
                ("property_kind_id", E::col(property)),
                ("derivation_id", E::col("reads.derivation_id")),
            ],
        ),
        false,
    );
}
fn resolve_scopes(builder: &mut RegistryBuilder) {
    let requests = scan("inferred.demand_scope_requests", "requests");
    let relative = join(
        requests.clone(),
        scan("inferred.scope_bindings", "scopes"),
        vec![
            ("requests.scope_decl_id", "scopes.scope_decl_id"),
            ("requests.owner_instance_id", "scopes.owner_instance_id"),
        ],
    );
    let global = join(
        requests,
        filter(
            scan("inferred.resolved_scopes", "scopes"),
            null("owner_instance_id"),
        ),
        vec![("requests.scope_decl_id", "scopes.scope_decl_id")],
    );
    for (name, input) in [
        ("P6.relative_scope_obligation", relative),
        ("P6.global_scope_obligation", global),
    ] {
        obligations(
            builder,
            name,
            project(
                input,
                vec![
                    ("read_id", E::col("requests.read_id")),
                    ("seed_id", E::col("requests.seed_id")),
                    (
                        "requester_instance_id",
                        E::col("requests.requester_instance_id"),
                    ),
                    ("scope_id", E::col("scopes.scope_id")),
                    ("property_kind_id", E::col("requests.property_kind_id")),
                    ("derivation_id", E::col("requests.derivation_id")),
                ],
            ),
        );
    }
}
fn active(builder: &mut RegistryBuilder) {
    let names = [
        "read_id",
        "seed_id",
        "requester_instance_id",
        "owner_instance_id",
        "symbol_decl_id",
        "index",
        "guard_source_id",
        "guard_node_id",
        "guard_index",
        "outer_guard_source_id",
        "outer_guard_node_id",
        "derivation_id",
    ];
    let qualified = [
        "reads.read_id",
        "reads.seed_id",
        "reads.requester_instance_id",
        "reads.owner_instance_id",
        "reads.symbol_decl_id",
        "reads.index",
        "reads.guard_source_id",
        "reads.guard_node_id",
        "reads.guard_index",
        "reads.outer_guard_source_id",
        "reads.outer_guard_node_id",
        "reads.derivation_id",
    ];
    rule(
        builder,
        "P6.active_read",
        0,
        "inferred.demand_active_reads",
        "provenance.demand_active_read_assertions",
        active_input("inferred.demand_read_keys", &names, &qualified),
        false,
    );
}
pub(in super::super) fn active_input(
    relation: &'static str,
    names: &[&'static str],
    qualified: &[&'static str],
) -> P {
    let unguarded = filter(
        scan(relation, "reads"),
        E::And(vec![null("guard_source_id"), null("guard_node_id")]),
    );
    let guarded = join(
        scan(relation, "reads"),
        scan("inferred.predicate_outcomes", "guards"),
        vec![
            ("reads.requester_instance_id", "guards.instance_id"),
            ("reads.guard_source_id", "guards.source_id"),
            ("reads.guard_node_id", "guards.predicate_id"),
            ("reads.guard_index", "guards.index"),
        ],
    );
    let guarded = filter(
        guarded,
        E::InList {
            expr: Box::new(E::col("guards.outcome")),
            list: vec![Cell::Enum("true"), Cell::Enum("unknown")],
        },
    );
    let input = P::Union(vec![
        project(
            unguarded,
            names.iter().map(|name| (*name, E::col(*name))).collect(),
        ),
        project(
            guarded,
            names
                .iter()
                .zip(qualified.iter().copied())
                .map(|(name, value)| (*name, E::col(value)))
                .collect(),
        ),
    ]);
    outer(input, names)
}
fn obligations(builder: &mut RegistryBuilder, name: &'static str, input: P) {
    rule(
        builder,
        name,
        1,
        "inferred.demand_obligations",
        "provenance.demand_obligation_assertions",
        input,
        false,
    );
}
fn scope_targets(builder: &mut RegistryBuilder) {
    let input = join(
        scan("inferred.demand_obligations", "obligations"),
        scan("inferred.scope_members", "members"),
        vec![("obligations.scope_id", "members.scope_id")],
    );
    let input = join(
        input,
        scan("inferred.demand_index_maps", "maps"),
        vec![("obligations.read_id", "maps.read_id")],
    );
    let input = join(
        input,
        scan("inferred.requirement_universe", "required"),
        vec![
            ("maps.requirement_id", "required.requirement_id"),
            ("members.entity_id", "required.state_instance_id"),
            ("obligations.property_kind_id", "required.property_kind_id"),
        ],
    );
    let input = join(
        input,
        scan("inferred.demand_request_keys", "requesters"),
        vec![
            ("obligations.seed_id", "requesters.seed_id"),
            (
                "obligations.requester_instance_id",
                "requesters.requester_instance_id",
            ),
        ],
    );
    emit(
        builder,
        "P6.scoped_read_requirement",
        input,
        "obligations.read_id",
        "obligations.seed_id",
        "obligations.requester_instance_id",
    );
}
fn provisions(builder: &mut RegistryBuilder, input: P) {
    let input = join(
        input,
        scan("reference.method_provisions", "provisions"),
        vec![("reads.symbol_decl_id", "provisions.symbol_decl_id")],
    );
    let input = join(
        input,
        scan("inferred.demand_index_maps", "maps"),
        vec![("reads.read_id", "maps.read_id")],
    );
    let input = join(
        input,
        scan("inferred.requirement_universe", "required"),
        vec![
            ("maps.requirement_id", "required.requirement_id"),
            ("reads.owner_instance_id", "required.state_instance_id"),
            ("provisions.property_kind_id", "required.property_kind_id"),
        ],
    );
    let input = join(
        input,
        scan("normalized.property_packages", "packages"),
        vec![
            (
                "required.property_package_id",
                "packages.property_package_id",
            ),
            (
                "provisions.method_id",
                "packages.state_definition_method_id",
            ),
        ],
    );
    let input = join(
        input,
        scan("inferred.demand_request_keys", "requesters"),
        vec![
            ("reads.seed_id", "requesters.seed_id"),
            (
                "reads.requester_instance_id",
                "requesters.requester_instance_id",
            ),
        ],
    );
    emit(
        builder,
        "P6.state_provision_read_requirement",
        input,
        "reads.read_id",
        "reads.seed_id",
        "reads.requester_instance_id",
    );
}
fn emit(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    read: &'static str,
    seed: &'static str,
    requester: &'static str,
) {
    rule(
        builder,
        name,
        1,
        "inferred.demand_seed_bindings",
        "provenance.demand_seed_binding_assertions",
        project(
            input,
            vec![
                ("read_id", E::col(read)),
                ("seed_id", E::col(seed)),
                ("requester_id", E::col("requesters.requester_id")),
                ("requester_instance_id", E::col(requester)),
                ("requirement_id", E::col("required.requirement_id")),
                ("state_scope_id", E::col("required.state_scope_id")),
                ("property_kind_id", E::col("required.property_kind_id")),
                ("index", E::col("required.index")),
                ("derivation_id", E::col("requesters.requester_id")),
            ],
        ),
        false,
    );
}

fn outer(input: P, names: &[&'static str]) -> P {
    let unguarded = filter(
        input.clone(),
        E::And(vec![
            null("outer_guard_source_id"),
            null("outer_guard_node_id"),
        ]),
    );
    let outcomes = filter(
        scan("inferred.predicate_outcomes", "outer_guard"),
        E::And(vec![
            eq(E::ListLen(Box::new(E::col("index"))), E::Lit(Cell::U64(0))),
            E::InList {
                expr: Box::new(E::col("outcome")),
                list: vec![Cell::Enum("true"), Cell::Enum("unknown")],
            },
        ]),
    );
    let outcomes = project(
        outcomes,
        vec![
            ("outer_instance", E::col("instance_id")),
            ("outer_source", E::col("source_id")),
            ("outer_node", E::col("predicate_id")),
        ],
    );
    let guarded = join(
        input,
        outcomes,
        vec![
            ("requester_instance_id", "outer_instance"),
            ("outer_guard_source_id", "outer_source"),
            ("outer_guard_node_id", "outer_node"),
        ],
    );
    P::Union(vec![
        unguarded,
        project(
            guarded,
            names.iter().map(|name| (*name, E::col(*name))).collect(),
        ),
    ])
}
