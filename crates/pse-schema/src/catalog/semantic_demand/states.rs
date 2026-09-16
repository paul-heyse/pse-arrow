// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! State eligibility and prospective index admission use actual selected declarations.
use super::{
    Cell, E, RegistryBuilder, T, anti, eq, filter, index, join, literal, project, rule, scan,
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    let actual = join(
        scan("inferred.state_scope_keys", "keys"),
        scan("inferred.instances", "actual"),
        vec![("keys.state_instance_id", "actual.instance_id")],
    );
    let bound = join(
        actual,
        scan("normalized.instance_bindings", "bound"),
        vec![("actual.instance_id", "bound.instance_id")],
    );
    let package = join(
        bound,
        scan("normalized.property_packages", "packages"),
        vec![("bound.property_package_id", "packages.property_package_id")],
    );
    let method = join(
        package,
        scan("reference.method_specs", "methods"),
        vec![("packages.state_definition_method_id", "methods.method_id")],
    );
    let valid = filter(
        method,
        E::And(vec![
            eq(E::col("methods.family"), literal("state_definition")),
            eq(E::col("methods.realization"), literal("equation_template")),
            eq(E::col("methods.template_id"), E::col("actual.template_id")),
        ]),
    );
    rule(
        builder,
        "P6.state_scope",
        0,
        "inferred.state_scopes",
        "provenance.state_scope_assertions",
        project(
            valid,
            vec![
                ("state_scope_id", E::col("keys.state_scope_id")),
                ("state_instance_id", E::col("actual.instance_id")),
                (
                    "property_package_id",
                    E::col("packages.property_package_id"),
                ),
                ("derivation_id", E::col("keys.derivation_id")),
            ],
        ),
        false,
    );
    let invalid = anti(
        scan("inferred.requirement_key_axes", "axes"),
        scan("inferred.domain_eligible_members", "eligible"),
        vec![
            ("axes.domain_id", "eligible.domain_id"),
            ("axes.member_id", "eligible.member_id"),
        ],
    );
    let keys = anti(
        scan("inferred.requirement_keys", "keys"),
        invalid,
        vec![("keys.requirement_id", "axes.requirement_id")],
    );
    let keys = join(
        keys,
        scan("inferred.valid_index_tuples", "tuples"),
        vec![
            ("keys.product_id", "tuples.product_id"),
            ("keys.index", "tuples.tuple"),
        ],
    );
    let universe = join(
        keys,
        scan("inferred.state_scopes", "scopes"),
        vec![
            ("keys.state_scope_id", "scopes.state_scope_id"),
            ("keys.state_instance_id", "scopes.state_instance_id"),
        ],
    );
    rule(
        builder,
        "P6.requirement_universe",
        1,
        "inferred.requirement_universe",
        "provenance.requirement_universe_assertions",
        project(
            universe,
            vec![
                ("requirement_id", E::col("keys.requirement_id")),
                ("state_scope_id", E::col("keys.state_scope_id")),
                ("state_instance_id", E::col("keys.state_instance_id")),
                ("property_package_id", E::col("scopes.property_package_id")),
                ("property_kind_id", E::col("keys.property_kind_id")),
                ("index", E::col("keys.index")),
                ("derivation_id", E::col("keys.derivation_id")),
            ],
        ),
        true,
    );
    selection_inventory(builder);
}
fn selection_inventory(builder: &mut RegistryBuilder) {
    let selections = project(
        scan("normalized.method_selections", "selections"),
        vec![
            ("selection_id", E::col("selection_id")),
            ("is_default", E::col("is_default")),
            ("property_package_id", E::col("property_package_id")),
            ("scope_kind", E::col("scope_kind")),
            ("scope_ids", E::col("scope_ids")),
            ("property_kind_id", E::col("property_kind_id")),
            ("family", E::col("family")),
            ("method_id", E::col("method_id")),
            ("source_kind", E::Lit(Cell::text("authored_selection"))),
            ("derivation_id", E::col("selection_id")),
        ],
    );
    rule(
        builder,
        "P6.authored_selection_inventory",
        0,
        "inferred.selection_inventory",
        "provenance.selection_inventory_assertions",
        selections,
        false,
    );
    let state = join(
        scan("inferred.state_method_selection_keys", "keys"),
        scan("normalized.property_packages", "packages"),
        vec![("keys.property_package_id", "packages.property_package_id")],
    );
    let state = project(
        state,
        vec![
            ("selection_id", E::col("keys.selection_id")),
            ("is_default", E::Lit(Cell::Bool(false))),
            (
                "property_package_id",
                E::col("packages.property_package_id"),
            ),
            ("scope_kind", literal("package")),
            (
                "scope_ids",
                E::call(
                    "pse_index_tuple",
                    vec![E::call("pse_id_list", vec![], T::list(T::id()), false)],
                    index(),
                    false,
                ),
            ),
            ("property_kind_id", E::Lit(Cell::Null)),
            ("family", literal("state_definition")),
            ("method_id", E::col("packages.state_definition_method_id")),
            (
                "source_kind",
                E::Lit(Cell::text("selected_state_definition")),
            ),
            ("derivation_id", E::col("keys.derivation_id")),
        ],
    );
    rule(
        builder,
        "P6.state_selection_inventory",
        0,
        "inferred.selection_inventory",
        "provenance.selection_inventory_assertions",
        state,
        false,
    );
}
