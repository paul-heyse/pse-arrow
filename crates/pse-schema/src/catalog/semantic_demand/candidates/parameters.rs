// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Missing required values and incompatible natural units are actual anti-join witnesses.
use super::{E, P, anti, eq, filter, join, null, project, same, scan};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn complete(candidates: P) -> P {
    let parameters = filter(
        scan("reference.method_parameters", "parameters"),
        E::col("required"),
    );
    let required = join(
        candidates.clone(),
        parameters,
        vec![("selections.method_id", "parameters.method_id")],
    );
    let keys = join(
        required.clone(),
        scan("inferred.method_parameter_keys", "parameter_keys"),
        vec![
            (
                "requirements.requirement_id",
                "parameter_keys.requirement_id",
            ),
            ("selections.method_id", "parameter_keys.method_id"),
            ("parameters.name", "parameter_keys.name"),
        ],
    );
    let missing_frames = anti(
        required,
        project(
            keys.clone(),
            vec![
                ("framed_requirement", E::col("requirements.requirement_id")),
                ("framed_method", E::col("selections.method_id")),
                ("framed_name", E::col("parameters.name")),
            ],
        ),
        vec![
            ("requirements.requirement_id", "framed_requirement"),
            ("selections.method_id", "framed_method"),
            ("parameters.name", "framed_name"),
        ],
    );
    let values = join(
        keys.clone(),
        scan("normalized.parameter_values", "values"),
        vec![
            ("requirements.property_package_id", "values.owner_entity_id"),
            ("parameters.name", "values.parameter_kind"),
            ("parameter_keys.index", "values.index"),
        ],
    );
    let values = join(
        values,
        scan("reference.quantity_types", "parameter_types"),
        vec![(
            "parameters.quantity_type_id",
            "parameter_types.quantity_type_id",
        )],
    );
    let values = join(
        values,
        scan("reference.units", "parameter_units"),
        vec![("values.unit_id", "parameter_units.unit_id")],
    );
    let values = join(
        values,
        scan("reference.units", "declared_units"),
        vec![("parameters.natural_unit_id", "declared_units.unit_id")],
    );
    let values = filter(
        values,
        E::And(vec![
            eq(
                E::col("parameter_units.dimension"),
                E::col("declared_units.dimension"),
            ),
            E::Or(vec![
                null("parameter_units.reference_state_id"),
                same(
                    "parameter_units.reference_state_id",
                    "parameter_types.reference_state_id",
                ),
            ]),
        ]),
    );
    let missing_values = anti(
        keys,
        project(
            values,
            vec![
                ("bound_requirement", E::col("requirements.requirement_id")),
                ("bound_method", E::col("selections.method_id")),
                ("bound_name", E::col("parameters.name")),
                ("bound_index", E::col("parameter_keys.index")),
            ],
        ),
        vec![
            ("requirements.requirement_id", "bound_requirement"),
            ("selections.method_id", "bound_method"),
            ("parameters.name", "bound_name"),
            ("parameter_keys.index", "bound_index"),
        ],
    );
    let rejected = P::Union(
        [missing_frames, missing_values]
            .into_iter()
            .map(|input| {
                project(
                    input,
                    vec![
                        ("bad_requirement", E::col("requirements.requirement_id")),
                        ("bad_selection", E::col("selections.selection_id")),
                        ("bad_method", E::col("selections.method_id")),
                    ],
                )
            })
            .collect(),
    );
    anti(
        candidates,
        rejected,
        vec![
            ("requirements.requirement_id", "bad_requirement"),
            ("selections.selection_id", "bad_selection"),
            ("selections.method_id", "bad_method"),
        ],
    )
}
