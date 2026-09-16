// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.10 case.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.10 case contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_authored_cases(builder);
    declare_authored_case_specs(builder);
    declare_authored_case_spec_targets(builder);
    declare_authored_case_activation_targets(builder);
    declare_authored_observation_targets(builder);
    declare_authored_case_activations(builder);
    declare_authored_case_objectives(builder);
    declare_authored_case_policies(builder);
    declare_authored_datasets(builder);
    declare_authored_observations(builder);
    declare_authored_scenarios(builder);
    declare_authored_case_sets(builder);
    declare_authored_case_set_samples(builder);
    declare_case_kind_vocabulary(builder);
    declare_treatment_vocabulary(builder);
    declare_target_kind_vocabulary(builder);
    declare_generator_kind_vocabulary(builder);
}

fn declare_authored_cases(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "cases",
        S::Case,
        &["case_id"],
        vec![
            column("case_id", T::id()),
            column("model_revision_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("parent_case_id", T::id())
                .optional()
                .with_fk("authored.cases", "case_id"),
            column("kind", T::enumeration("CaseKind")),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.10 case: cases.",
    );
}

fn declare_authored_case_specs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_specs",
        S::Case,
        &["spec_id"],
        vec![
            column("spec_id", T::id()),
            column("case_id", T::id()).with_fk("authored.cases", "case_id"),
            column(
                "target",
                T::extended(crate::model::ExtensionUse::TargetPath),
            ),
            column("treatment", T::enumeration("Treatment")).optional(),
            column("value", T::native(arrow_schema::DataType::Float64)).optional(),
            column("unit_id", T::id()).optional(),
            column("initial", T::native(arrow_schema::DataType::Float64)).optional(),
            column("lower", T::extended(crate::model::ExtensionUse::Bound)).optional(),
            column("upper", T::extended(crate::model::ExtensionUse::Bound)).optional(),
            column("scaling_factor", T::native(arrow_schema::DataType::Float64)).optional(),
            column("priority", T::native(arrow_schema::DataType::Int32)),
        ],
        "blueprint §6.10 case: case_specs.",
    );
}

fn declare_authored_case_spec_targets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_spec_targets",
        S::Case,
        &["spec_id", "ordinal"],
        vec![
            column("spec_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("instance_id", T::id()),
            column("member_kind", T::enumeration("TargetKind")),
            column("symbol_decl_id", T::id()).optional(),
            column("equation_decl_id", T::id())
                .optional()
                .with_fk("authored.template_equations", "equation_decl_id"),
            column("port_template_id", T::id())
                .optional()
                .with_fk("authored.templates", "template_id"),
            column("port_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)).optional(),
            column("wildcard", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.10 case: case_spec_targets.",
    );
}

fn declare_authored_case_activation_targets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_activation_targets",
        S::Case,
        &["activation_id", "ordinal"],
        vec![
            column("activation_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("instance_id", T::id()),
            column("member_kind", T::enumeration("TargetKind")),
            column("symbol_decl_id", T::id()).optional(),
            column("equation_decl_id", T::id())
                .optional()
                .with_fk("authored.template_equations", "equation_decl_id"),
            column("port_template_id", T::id())
                .optional()
                .with_fk("authored.templates", "template_id"),
            column("port_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)).optional(),
            column("wildcard", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.10 case: case_activation_targets.",
    );
}

fn declare_authored_observation_targets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "observation_targets",
        S::Case,
        &["observation_id", "ordinal"],
        vec![
            column("observation_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("instance_id", T::id()),
            column("member_kind", T::enumeration("TargetKind")),
            column("symbol_decl_id", T::id()).optional(),
            column("equation_decl_id", T::id())
                .optional()
                .with_fk("authored.template_equations", "equation_decl_id"),
            column("port_template_id", T::id())
                .optional()
                .with_fk("authored.templates", "template_id"),
            column("port_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)).optional(),
            column("wildcard", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.10 case: observation_targets.",
    );
}

fn declare_authored_case_activations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_activations",
        S::Case,
        &["activation_id"],
        vec![
            column("activation_id", T::id()),
            column("case_id", T::id()),
            column(
                "target",
                T::extended(crate::model::ExtensionUse::TargetPath),
            ),
            column("active", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.10 case: case_activations.",
    );
}

fn declare_authored_case_objectives(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_objectives",
        S::Case,
        &["case_id", "objective_id"],
        vec![
            column("case_id", T::id()),
            column("objective_id", T::id()),
            column("active", T::native(arrow_schema::DataType::Boolean)),
            column("weight", T::native(arrow_schema::DataType::Float64)),
        ],
        "blueprint §6.10 case: case_objectives.",
    );
}

fn declare_authored_case_policies(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_policies",
        S::Case,
        &["case_id"],
        vec![
            column("case_id", T::id()),
            column("discretization_policy_id", T::id()).optional(),
            column("scaler_template_id", T::id()).optional(),
            column("numerical_policy_id", T::id()).optional(),
            column("initializer_template_id", T::id()).optional(),
            column("solver_profile_id", T::id()).optional(),
        ],
        "blueprint §6.10 case: case_policies.",
    );
}

fn declare_authored_datasets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "datasets",
        S::Case,
        &["dataset_id"],
        vec![
            column("dataset_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("source", T::native(arrow_schema::DataType::Utf8)),
            column("content_hash", T::hash()),
        ],
        "blueprint §6.10 case: datasets.",
    );
}

fn declare_authored_observations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "observations",
        S::Case,
        &["observation_id"],
        vec![
            column("observation_id", T::id()),
            column("dataset_id", T::id()),
            column(
                "target",
                T::extended(crate::model::ExtensionUse::TargetPath),
            ),
            column("value", T::native(arrow_schema::DataType::Float64)).optional(),
            column("unit_id", T::id()),
            column("std_dev", T::native(arrow_schema::DataType::Float64)).optional(),
            column(
                "timestamp",
                T::native(crate::model::extension::timestamp_storage()),
            )
            .optional(),
            column("tag", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "blueprint §6.10 case: observations.",
    );
}

fn declare_authored_scenarios(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "scenarios",
        S::Case,
        &["scenario_id"],
        vec![
            column("scenario_id", T::id()),
            column("case_id", T::id()),
            column("weight", T::native(arrow_schema::DataType::Float64)),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.10 case: scenarios.",
    );
}

fn declare_authored_case_sets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_sets",
        S::Case,
        &["case_set_id"],
        vec![
            column("case_set_id", T::id()),
            column("base_case_id", T::id()),
            column("generator_kind", T::enumeration("GeneratorKind")),
            column(
                "generator_params",
                T::list(structure(vec![
                    ("key", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column("seed", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("sample_count", T::native(arrow_schema::DataType::UInt64)),
        ],
        "blueprint §6.10 case: case_sets.",
    );
}

fn declare_authored_case_set_samples(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_set_samples",
        S::Case,
        &["case_set_id", "sample_ordinal"],
        vec![
            column("case_set_id", T::id()),
            column("sample_ordinal", T::native(arrow_schema::DataType::UInt64)),
            column("case_id", T::id()),
        ],
        "blueprint §6.10 case: case_set_samples.",
    );
}

fn declare_case_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "CaseKind",
        [
            "base",
            "overlay",
            "initialization_stage",
            "scenario",
            "sweep_sample",
        ],
    );
}

fn declare_treatment_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "Treatment", ["fixed", "free", "parameter"]);
}

fn declare_target_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "TargetKind",
        ["symbol", "port", "group", "equation", "instance_wildcard"],
    );
}

fn declare_generator_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "GeneratorKind",
        ["grid", "latin_hypercube", "uniform_random", "list"],
    );
}
