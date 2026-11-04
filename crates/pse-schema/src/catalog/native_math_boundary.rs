// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Stable wire presentation at the actual catalog owner; native fields own all types.

use super::boundary_source::RecordLayout;
use crate::RegistryBuilder;
use crate::model::{FieldContract, Namespace, SnapshotClass};

#[expect(
    clippy::too_many_arguments,
    reason = "the actual relation declaration includes all source facets"
)]
pub(super) fn relation(
    source: &'static str,
    builder: &mut RegistryBuilder,
    namespace: Namespace,
    name: &'static str,
    class: SnapshotClass,
    keys: &[&'static str],
    columns: Vec<FieldContract>,
    doc: &'static str,
) {
    relation_version(
        source, builder, namespace, name, 1, class, keys, columns, doc,
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "the actual relation declaration includes its explicit interpretation version"
)]
pub(super) fn relation_version(
    source: &'static str,
    builder: &mut RegistryBuilder,
    namespace: Namespace,
    name: &'static str,
    version: u32,
    class: SnapshotClass,
    keys: &[&'static str],
    columns: Vec<FieldContract>,
    doc: &'static str,
) {
    let selection = match name {
        "fit_cases" => Some(("AuthoredFitCasesRow", FIT_CASES)),
        "numerical_requirements" => {
            Some(("AuthoredNumericalRequirementsRow", NUMERICAL_REQUIREMENTS))
        }
        "accuracy_goals" => Some(("AuthoredAccuracyGoalsRow", ACCURACY_GOALS)),
        "caloric_requirements" => Some(("AuthoredCaloricRequirementsRow", CALORIC_REQUIREMENTS)),
        "engineering_scales" => Some(("AuthoredEngineeringScalesRow", ENGINEERING_SCALES)),
        "engineering_default_rules" => Some((
            "AuthoredEngineeringDefaultRulesRow",
            ENGINEERING_DEFAULT_RULES,
        )),
        _ => None,
    };
    let result = selection
        .ok_or_else(|| crate::checks::invalid(name, "unselected native row presentation"))
        .and_then(|(native_name, layouts)| {
            super::boundary_source::present(
                super::declarations::declaration(
                    namespace, name, version, class, keys, columns, doc,
                ),
                source,
                native_name,
                layouts,
            )
        });
    builder.declare_relation_result(result);
}

const FIT_CASES: &[RecordLayout] = &[
    RecordLayout {
        path: "",
        fields: &[
            ("fit_id", 1),
            ("parameters", 2),
            ("experiments", 3),
            ("observations", 4),
        ],
        alternatives: &[],
    },
    RecordLayout {
        path: "experiments",
        fields: &[
            ("experiment_id", 1),
            ("case_id", 2),
            ("route", 3),
            ("bindings", 4),
        ],
        alternatives: &[],
    },
    RecordLayout {
        path: "experiments.bindings",
        fields: &[("parameter_id", 1), ("path", 2)],
        alternatives: &[],
    },
    RecordLayout {
        path: "observations",
        fields: &[
            ("observation_id", 1),
            ("value_attribute", 2),
            ("standard_deviation_attribute", 3),
            ("experiment_id", 4),
            ("output_path", 5),
            ("time", 6),
            ("time_basis", 7),
            ("time_unit_id", 8),
            ("included", 9),
            ("importance", 10),
        ],
        alternatives: &[],
    },
    RecordLayout {
        path: "parameters",
        fields: &[
            ("symbol_id", 1),
            ("fixed", 2),
            ("value", 3),
            ("lower", 4),
            ("upper", 5),
            ("scale", 6),
        ],
        alternatives: &[],
    },
];

const NUMERICAL_REQUIREMENTS: &[RecordLayout] = &[RecordLayout {
    path: "",
    fields: &[
        ("requirement_id", 1),
        ("model_id", 2),
        ("case_id", 3),
        ("instance_id", 4),
        ("fit_id", 5),
        ("target_id", 6),
        ("target_kind", 7),
        ("nominal", 8),
        ("scaling_factor", 9),
        ("absolute_tolerance", 10),
        ("relative_tolerance", 11),
        ("shared_engineering_allowance", 12),
        ("engineering_rule_id", 13),
        ("unit_id", 14),
        ("coordinates", 15),
        ("priority", 16),
        ("required", 17),
        ("provenance", 18),
    ],
    alternatives: &[],
}];

const ACCURACY_GOALS: &[RecordLayout] = &[RecordLayout {
    path: "",
    fields: &[
        ("goal_id", 1),
        ("model_id", 2),
        ("case_id", 3),
        ("instance_id", 4),
        ("fit_id", 5),
        ("target_id", 6),
        ("target_kind", 7),
        ("quantity_id", 8),
        ("unit_id", 9),
        ("subject", 10),
        ("observation", 11),
        ("time", 12),
        ("resolution", 13),
        ("criterion_lower", 14),
        ("criterion_upper", 15),
        ("required_class", 16),
        ("use_policy", 17),
        ("refine", 18),
        ("source", 19),
        ("priority", 20),
        ("provenance", 21),
    ],
    alternatives: &[],
}];

const CALORIC_REQUIREMENTS: &[RecordLayout] = &[
    RecordLayout {
        path: "",
        fields: &[
            ("requirement_id", 1),
            ("instance_id", 2),
            ("target_id", 3),
            ("target_kind", 4),
            ("quantity_id", 5),
            ("unit_id", 6),
            ("source", 7),
            ("science_revision", 8),
            ("science_records", 9),
            ("response_quantity_id", 10),
            ("response_unit_id", 11),
            ("response", 12),
            ("thermal_quantity_id", 13),
            ("thermal_unit_id", 14),
            ("thermal_resolution", 15),
            ("consumers", 16),
        ],
        alternatives: &[],
    },
    RecordLayout {
        path: "consumers",
        fields: &[
            ("target_id", 1),
            ("declaration_id", 2),
            ("power_rule_id", 3),
            ("actual_flow_id", 4),
            ("exposure_expression", 5),
            ("power_quantity_id", 6),
            ("power_unit_id", 7),
            ("power_allowance", 8),
            ("exposure_quantity_id", 9),
            ("exposure_unit_id", 10),
            ("aggregate_exposure", 11),
        ],
        alternatives: &[],
    },
];

const ENGINEERING_SCALES: &[RecordLayout] = &[RecordLayout {
    path: "",
    fields: &[
        ("scale_id", 1),
        ("model_id", 2),
        ("case_id", 3),
        ("instance_id", 4),
        ("fit_id", 5),
        ("target_id", 6),
        ("target_kind", 7),
        ("quantity_id", 8),
        ("unit_id", 9),
        ("kind", 10),
        ("value", 11),
        ("source", 12),
        ("priority", 13),
        ("provenance", 14),
    ],
    alternatives: &[],
}];

const ENGINEERING_DEFAULT_RULES: &[RecordLayout] = &[RecordLayout {
    path: "",
    fields: &[
        ("rule_id", 1),
        ("quantity_id", 2),
        ("unit_id", 3),
        ("physical_allowance", 4),
        ("relative_fraction", 5),
        ("provenance", 6),
    ],
    alternatives: &[],
}];
