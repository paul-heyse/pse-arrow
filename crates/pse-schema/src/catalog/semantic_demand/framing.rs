// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native identity projections over the complete prospective demand inventory.

use super::{Cell, E, P, RegistryBuilder, RuleDecl, RuleHead, T, assertion, join, project, scan};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (head, name) in [
        ("inferred.state_scope_keys", "state_scope_key_assertions"),
        (
            "inferred.state_method_selection_keys",
            "state_method_selection_key_assertions",
        ),
        (
            "inferred.demand_request_keys",
            "demand_request_key_assertions",
        ),
    ] {
        assertion(builder, head, name);
    }
    identity(
        builder,
        "P6.state_scope_key",
        "inferred.state_scope_keys",
        "provenance.state_scope_key_assertions",
        IdentityProjection {
            source: "normalized.instance_bindings",
            source_key: "instance_id",
            projected_key: "state_instance_id",
            result: "state_scope_id",
            label: "pse:state-scope:v1",
        },
    );
    identity(
        builder,
        "P6.state_method_selection_key",
        "inferred.state_method_selection_keys",
        "provenance.state_method_selection_key_assertions",
        IdentityProjection {
            source: "normalized.property_packages",
            source_key: "property_package_id",
            projected_key: "property_package_id",
            result: "selection_id",
            label: "pse:state-method-selection:v1",
        },
    );
    requesters(builder);
}

#[derive(Clone, Copy)]
struct IdentityProjection {
    source: &'static str,
    source_key: &'static str,
    projected_key: &'static str,
    result: &'static str,
    label: &'static str,
}

fn identity(
    builder: &mut RegistryBuilder,
    name: &'static str,
    head: &'static str,
    assertion: &'static str,
    projection: IdentityProjection,
) {
    let IdentityProjection {
        source,
        source_key,
        projected_key,
        result,
        label,
    } = projection;
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            48,
            RuleHead::Relation(head.to_owned()),
            P::Union(vec![project(
                scan(source, "source"),
                vec![
                    (projected_key, E::col(source_key)),
                    (
                        result,
                        E::call(
                            "pse_named_id",
                            vec![E::col(source_key), E::Lit(Cell::text(label))],
                            T::id(),
                            false,
                        ),
                    ),
                    ("derivation_id", E::col(source_key)),
                ],
            )]),
        )
        .assertions(assertion),
    );
}

fn requesters(builder: &mut RegistryBuilder) {
    for (source, key, name) in [
        (
            "normalized.property_demand_seeds",
            "seed_id",
            "P6.seed_request_key",
        ),
        (
            "normalized.property_path_demands",
            "demand_id",
            "P6.path_request_key",
        ),
    ] {
        let seeds = project(
            scan(source, "seed"),
            vec![
                ("seed_id", E::col(key)),
                ("seed_cross", E::Lit(Cell::Bool(true))),
            ],
        );
        let instances = project(
            scan("normalized.instance_bindings", "instances"),
            vec![
                ("owner_instance_id", E::col("instance_id")),
                ("instance_cross", E::Lit(Cell::Bool(true))),
            ],
        );
        let input = join(seeds, instances, vec![("seed_cross", "instance_cross")]);
        let hexadecimal = E::call(
            "encode",
            vec![E::col("owner_instance_id"), E::Lit(Cell::text("hex"))],
            T::native(arrow_schema::DataType::Utf8),
            true,
        );
        let label = E::call(
            "concat",
            vec![E::Lit(Cell::text("pse:demand-occurrence:v1:")), hexadecimal],
            T::native(arrow_schema::DataType::Utf8),
            true,
        );
        let label = E::call(
            "coalesce",
            vec![label, E::Lit(Cell::text(""))],
            T::native(arrow_schema::DataType::Utf8),
            false,
        );
        builder.declare_rule(
            RuleDecl::new(
                name,
                "1",
                48,
                RuleHead::Relation("inferred.demand_request_keys".to_owned()),
                P::Union(vec![project(
                    input,
                    vec![
                        ("seed_id", E::col("seed_id")),
                        ("requester_instance_id", E::col("owner_instance_id")),
                        (
                            "requester_id",
                            E::call(
                                "pse_named_id",
                                vec![E::col("seed_id"), label],
                                T::id(),
                                false,
                            ),
                        ),
                        ("derivation_id", E::col("seed_id")),
                    ],
                )]),
            )
            .assertions("provenance.demand_request_key_assertions"),
        );
    }
}
