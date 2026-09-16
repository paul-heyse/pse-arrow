// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An applicable method must have a complete actual dependency correspondence.
use super::{E, P, anti, eq, filter, join, literal, project, scan};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn complete(input: P) -> P {
    let required = join(
        input.clone(),
        scan("reference.method_dependencies", "dependencies"),
        vec![("selections.method_id", "dependencies.method_id")],
    );
    let property = filter(
        required.clone(),
        eq(E::col("dependencies.target_kind"), literal("property")),
    );
    let property = join(
        property,
        scan("inferred.dependency_key_maps", "dependency_maps"),
        vec![
            (
                "requirements.requirement_id",
                "dependency_maps.requirement_id",
            ),
            ("selections.method_id", "dependency_maps.method_id"),
            ("dependencies.ordinal", "dependency_maps.dependency_ordinal"),
        ],
    );
    let property = join(
        property,
        scan("inferred.requirement_universe", "dependency_targets"),
        vec![
            (
                "dependency_maps.target_requirement_id",
                "dependency_targets.requirement_id",
            ),
            (
                "dependencies.target_id",
                "dependency_targets.property_kind_id",
            ),
            (
                "requirements.state_instance_id",
                "dependency_targets.state_instance_id",
            ),
        ],
    );
    let state = filter(
        required.clone(),
        eq(E::col("dependencies.target_kind"), literal("state_symbol")),
    );
    let state = join(
        state,
        scan("inferred.state_dependency_keys", "state_maps"),
        vec![
            ("requirements.requirement_id", "state_maps.requirement_id"),
            ("selections.method_id", "state_maps.method_id"),
            ("dependencies.ordinal", "state_maps.dependency_ordinal"),
            ("dependencies.target_id", "state_maps.symbol_decl_id"),
        ],
    );
    let state = join(
        state,
        scan("inferred.valid_index_tuples", "state_tuples"),
        vec![
            ("state_maps.product_id", "state_tuples.product_id"),
            ("state_maps.index", "state_tuples.tuple"),
        ],
    );
    let state = join(
        state,
        scan("normalized.instance_bindings", "state_instances"),
        vec![(
            "requirements.state_instance_id",
            "state_instances.instance_id",
        )],
    );
    let state = join(
        state,
        scan("normalized.template_symbols", "state_symbols"),
        vec![
            ("state_maps.symbol_decl_id", "state_symbols.symbol_decl_id"),
            ("state_instances.template_id", "state_symbols.template_id"),
        ],
    );
    let valid = P::Union(
        vec![property, state]
            .into_iter()
            .map(|plan| {
                project(
                    plan,
                    vec![
                        (
                            "satisfied_requirement",
                            E::col("requirements.requirement_id"),
                        ),
                        ("satisfied_selection", E::col("selections.selection_id")),
                        ("satisfied_method", E::col("selections.method_id")),
                        ("satisfied_ordinal", E::col("dependencies.ordinal")),
                    ],
                )
            })
            .collect(),
    );
    let missing = anti(
        required,
        valid,
        vec![
            ("requirements.requirement_id", "satisfied_requirement"),
            ("selections.selection_id", "satisfied_selection"),
            ("selections.method_id", "satisfied_method"),
            ("dependencies.ordinal", "satisfied_ordinal"),
        ],
    );
    let missing = project(
        missing,
        vec![
            ("bad_requirement", E::col("requirements.requirement_id")),
            ("bad_selection", E::col("selections.selection_id")),
            ("bad_method", E::col("selections.method_id")),
        ],
    );
    anti(
        input,
        missing,
        vec![
            ("requirements.requirement_id", "bad_requirement"),
            ("selections.selection_id", "bad_selection"),
            ("selections.method_id", "bad_method"),
        ],
    )
}
