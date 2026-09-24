// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.11 numerical and rule.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.11 numerical and rule contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_algorithm_specs(builder);
    declare_reference_algorithm_arguments(builder);
    declare_reference_algorithm_results(builder);
    declare_reference_engine_profiles(builder);
}

fn declare_reference_algorithm_specs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "algorithm_specs",
        S::Model,
        &["algorithm_id"],
        vec![
            column("algorithm_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("preconditions", T::list(T::id())),
            column("postconditions", T::list(T::id())),
            column("determinism", T::enumeration("Determinism")),
            column("diagnostics", T::list(T::enumeration("FailureClass"))),
            column("effects", T::list(T::enumeration("OperationEffect"))),
        ],
        "blueprint §6.11 numerical and rule: algorithm_specs.",
    );
}

fn declare_reference_algorithm_arguments(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "InputConsumptionKind", ["whole", "columns"]);
    relation(
        builder,
        N::Reference,
        "algorithm_arguments",
        S::Model,
        &["algorithm_id", "port"],
        vec![
            column("algorithm_id", T::id()),
            column("port", T::native(arrow_schema::DataType::Utf8)),
            column("relation_id", T::id()),
            column("required", T::native(arrow_schema::DataType::Boolean)),
            column(
                "consumption",
                T::structure(vec![
                    T::enumeration("InputConsumptionKind").with_name("kind"),
                    T::structure(vec![
                        T::list(T::native(arrow_schema::DataType::Utf8)).with_name("names"),
                    ])
                    .with_name("columns")
                    .optional(),
                ])
                .with_alternative(
                    &crate::model::TaggedAlternative::new(
                        "kind",
                        [("columns".into(), "columns".into())],
                    )
                    .with_unit("whole"),
                ),
            ),
        ],
        "blueprint §6.11 numerical and rule: algorithm_arguments.",
    );
}

fn declare_reference_algorithm_results(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "algorithm_results",
        S::Model,
        &["algorithm_id", "port"],
        vec![
            column("algorithm_id", T::id()),
            column("port", T::native(arrow_schema::DataType::Utf8)),
            column("relation_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: algorithm_results.",
    );
}

fn declare_reference_engine_profiles(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "engine_profiles",
        S::Model,
        &["engine_profile_id"],
        vec![
            column("engine_profile_id", T::id()),
            column(
                "datafusion_version",
                T::native(arrow_schema::DataType::Utf8),
            ),
            column("arrow_version", T::native(arrow_schema::DataType::Utf8)),
            column(
                "analyzer_rules",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column(
                "optimizer_rules",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column(
                "physical_rules",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column(
                "semantic_settings",
                T::list(structure(vec![
                    ("key", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column(
                "setting_allow_list_version",
                T::native(arrow_schema::DataType::Utf8),
            ),
        ],
        "blueprint §6.11 numerical and rule: engine_profiles.",
    );
}
