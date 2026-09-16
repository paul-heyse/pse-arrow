// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.9 math.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.9 math contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_equation_vocabularies(builder);
    declare_builtin_quantity_rule_vocabulary(builder);
    declare_compiled_math_expr_nodes(builder);
    declare_compiled_math_expr_args(builder);
    declare_compiled_math_symbol_refs(builder);
    declare_compiled_math_float_constants(builder);
    declare_compiled_math_int_constants(builder);
    declare_compiled_math_affine(builder);
    declare_compiled_math_weighted_means(builder);
    declare_compiled_math_reductions(builder);
    declare_compiled_math_gathers(builder);
    declare_compiled_math_broadcasts(builder);
    declare_compiled_math_derivatives(builder);
    declare_compiled_math_integrals(builder);
    declare_compiled_math_smooth_ops(builder);
    declare_compiled_math_conditionals(builder);
    declare_compiled_math_kernel_calls(builder);
    declare_compiled_math_implicit_refs(builder);
    declare_compiled_math_unit_converts(builder);
    declare_compiled_math_piecewise_linear(builder);
    declare_compiled_math_quantity_selections(builder);
    declare_compiled_math_indexed_equations(builder);
    declare_compiled_math_free_indices(builder);
    declare_compiled_math_equations(builder);
    declare_compiled_math_objectives(builder);
    declare_compiled_math_implicit_systems(builder);
    declare_compiled_math_complementarity(builder);
    declare_compiled_math_dae_links(builder);
    declare_sense_vocabulary(builder);
    declare_objective_sense_vocabulary(builder);
    declare_complementarity_form_vocabulary(builder);
}

/// Self-contained blueprint §7.5 vocabularies; admission is not P14 classification.
fn declare_equation_vocabularies(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "EquationFamily",
        [
            "BOUND",
            "AFFINE_EQUALITY",
            "AFFINE_INEQUALITY",
            "NETWORK_BALANCE",
            "SIMPLEX_ALLOCATION",
            "BILINEAR",
            "SMOOTH_TRANSCENDENTAL",
            "NONSMOOTH_CONVEX",
            "BLACK_BOX",
            "DEFINITION",
            "REPORTING_DEFINITION",
            "GENERAL_NONLINEAR",
            "UNCLASSIFIED",
        ],
    );
    super::declarations::enumeration(
        builder,
        "EquationRole",
        [
            "HARD_FEASIBILITY",
            "DEFINITION",
            "LINKING",
            "DOMAIN_GUARD",
            "REPORTING",
            "APPROXIMATION",
        ],
    );
}

fn declare_builtin_quantity_rule_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "BuiltinQuantityRule",
        pse_quantity::infer::BuiltInRule::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_compiled_math_expr_nodes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_expr_nodes",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("opcode", T::enumeration("Opcode")),
            column("quantity_type_id", T::id()).optional(),
            column("scope_instance_id", T::id()).optional(),
            column("subtree_hash", T::hash()),
        ],
        "blueprint §6.9 math: math_expr_nodes.",
    );
}

fn declare_compiled_math_expr_args(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_expr_args",
        S::Derived,
        &["parent_node_id", "argument_ordinal"],
        vec![
            column("parent_node_id", T::native(arrow_schema::DataType::UInt64)),
            column(
                "argument_ordinal",
                T::native(arrow_schema::DataType::UInt16),
            ),
            column("child_node_id", T::native(arrow_schema::DataType::UInt64)),
        ],
        "blueprint §6.9 math: math_expr_args.",
    );
}

fn declare_compiled_math_symbol_refs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_symbol_refs",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("symbol_id", T::id()),
        ],
        "blueprint §6.9 math: math_symbol_refs.",
    );
}

fn declare_compiled_math_float_constants(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_float_constants",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("value", T::native(arrow_schema::DataType::Float64)),
            column("unit_id", T::id()),
        ],
        "blueprint §6.9 math: math_float_constants.",
    );
}

fn declare_compiled_math_int_constants(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_int_constants",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("value", T::native(arrow_schema::DataType::Int64)),
        ],
        "blueprint §6.9 math: math_int_constants.",
    );
}

fn declare_compiled_math_affine(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_affine",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("constant", T::native(arrow_schema::DataType::Float64)),
            column("constant_quantity_type_id", T::id()).optional(),
            column("constant_unit_id", T::id()).optional(),
            column(
                "terms",
                T::list(structure(vec![
                    ("coefficient", T::native(arrow_schema::DataType::Float64)),
                    ("child_node_id", T::native(arrow_schema::DataType::UInt64)),
                ])),
            ),
        ],
        "blueprint §6.9 math: math_affine.",
    );
}

fn declare_compiled_math_weighted_means(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_weighted_means",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column(
                "pairs",
                T::list(structure(vec![
                    ("weight_node_id", T::native(arrow_schema::DataType::UInt64)),
                    ("value_node_id", T::native(arrow_schema::DataType::UInt64)),
                ])),
            ),
            column("normalization", T::enumeration("WeightNormalization")),
            column("unit_sum_invariant_id", T::id()).optional(),
        ],
        "blueprint §6.9 math: math_weighted_means.",
    );
}

fn declare_compiled_math_reductions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_reductions",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("kind", T::enumeration("ReductionKind")),
            column("domain_id", T::id()),
            column("bound_index_id", T::id()),
            column("filter_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
        ],
        "blueprint §6.9 math: math_reductions.",
    );
}

fn declare_compiled_math_gathers(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_gathers",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("group_id", T::id()),
            column(
                "coordinate_map",
                T::list(structure(vec![
                    ("bound_index_id", T::id()),
                    ("position", T::native(arrow_schema::DataType::UInt16)),
                ])),
            ),
        ],
        "blueprint §6.9 math: math_gathers.",
    );
}

fn declare_compiled_math_broadcasts(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_broadcasts",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("domain_id", T::id()),
            column("bound_index_id", T::id()),
        ],
        "blueprint §6.9 math: math_broadcasts.",
    );
}

fn declare_compiled_math_derivatives(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_derivatives",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("wrt_domain_id", T::id()),
            column("order", T::native(arrow_schema::DataType::UInt8)),
        ],
        "blueprint §6.9 math: math_derivatives.",
    );
}

fn declare_compiled_math_integrals(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_integrals",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("domain_id", T::id()),
            column("quadrature_policy_id", T::id()).optional(),
            column("bound_index_id", T::id()),
            column("filter_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
        ],
        "blueprint §6.9 math: math_integrals.",
    );
}

fn declare_compiled_math_smooth_ops(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_smooth_ops",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("eps", T::native(arrow_schema::DataType::Float64)),
        ],
        "blueprint §6.9 math: math_smooth_ops.",
    );
}

fn declare_compiled_math_conditionals(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_conditionals",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("guard_node_id", T::native(arrow_schema::DataType::UInt64)),
        ],
        "blueprint §6.9 math: math_conditionals.",
    );
}

fn declare_compiled_math_kernel_calls(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_kernel_calls",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("kernel_binding_id", T::id()),
            column("output_ordinal", T::native(arrow_schema::DataType::UInt16)),
        ],
        "blueprint §6.9 math: math_kernel_calls.",
    );
}

fn declare_compiled_math_implicit_refs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_implicit_refs",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("implicit_system_id", T::id()),
            column("unknown_ordinal", T::native(arrow_schema::DataType::UInt16)),
        ],
        "blueprint §6.9 math: math_implicit_refs.",
    );
}

fn declare_compiled_math_unit_converts(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_unit_converts",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("scale", T::native(arrow_schema::DataType::Float64)),
            column("offset", T::native(arrow_schema::DataType::Float64)),
            column("from_unit_id", T::id()),
            column("to_unit_id", T::id()),
        ],
        "blueprint §6.9 math: math_unit_converts.",
    );
}

fn declare_compiled_math_piecewise_linear(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_piecewise_linear",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column(
                "breakpoints",
                T::list(structure(vec![
                    ("x", T::native(arrow_schema::DataType::Float64)),
                    ("y", T::native(arrow_schema::DataType::Float64)),
                ])),
            ),
            column("input_quantity_type_id", T::id()),
            column("output_quantity_type_id", T::id()),
        ],
        "blueprint §6.9 math: math_piecewise_linear.",
    );
}

fn declare_compiled_math_quantity_selections(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_quantity_selections",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("operation_id", T::id()).optional(),
            column("builtin_rule", T::enumeration("BuiltinQuantityRule")).optional(),
            column(
                "operand_permutation",
                T::list(T::native(arrow_schema::DataType::UInt16)),
            ),
            column(
                "conversions",
                T::list(structure(vec![
                    ("operand", T::native(arrow_schema::DataType::UInt16)),
                    ("conversion_id", T::id()),
                ])),
            ),
            column(
                "deferred_static_check",
                T::native(arrow_schema::DataType::Boolean),
            ),
        ],
        "blueprint §6.9 math: math_quantity_selections.",
    );
}

fn declare_compiled_math_indexed_equations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_indexed_equations",
        S::Derived,
        &["indexed_equation_id"],
        vec![
            column("indexed_equation_id", T::id()),
            column("owner_instance_id", T::id()),
            column("equation_decl_id", T::id()).optional(),
            column("qualified_name", T::native(arrow_schema::DataType::Utf8)),
            column("product_id", T::id()).optional(),
            column("filter_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("body_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("sense", T::enumeration("Sense")),
            column("lower_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("upper_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("residual_quantity_type_id", T::id()).optional(),
            column("law_instance_id", T::id()).optional(),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.9 math: math_indexed_equations.",
    );
}

fn declare_compiled_math_free_indices(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_free_indices",
        S::Derived,
        &["indexed_equation_id", "bound_index_id"],
        vec![
            column("indexed_equation_id", T::id()),
            column("bound_index_id", T::id()),
            column("domain_id", T::id()),
            column("position", T::native(arrow_schema::DataType::UInt16)),
        ],
        "blueprint §6.9 math: math_free_indices.",
    );
}

fn declare_compiled_math_equations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_equations",
        S::Derived,
        &["equation_id"],
        vec![
            column("equation_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt64)),
            column("owner_instance_id", T::id()),
            column("equation_decl_id", T::id()).optional(),
            column("parent_indexed_equation_id", T::id()).optional(),
            column("qualified_name", T::native(arrow_schema::DataType::Utf8)),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("body_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("sense", T::enumeration("Sense")),
            column("lower_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("upper_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("residual_quantity_type_id", T::id()),
            column("family", T::enumeration("EquationFamily")),
            column("role", T::enumeration("EquationRole")),
            column("differentiability", T::enumeration("Differentiability")),
            column("convexity", T::enumeration("Convexity")),
            column("monotonicity", T::enumeration("Monotonicity")),
            column("default_active", T::native(arrow_schema::DataType::Boolean)),
            column("group_id", T::id()).optional(),
            column("law_instance_id", T::id()).optional(),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.9 math: math_equations.",
    );
}

fn declare_compiled_math_objectives(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_objectives",
        S::Derived,
        &["objective_id"],
        vec![
            column("objective_id", T::id()),
            column("owner_instance_id", T::id()),
            column("body_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("sense", T::enumeration("ObjectiveSense")),
            column("quantity_type_id", T::id()),
            column("default_active", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.9 math: math_objectives.",
    );
}

fn declare_compiled_math_implicit_systems(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        crate::model::RelationDecl::new(
            N::Compiled,
            "math_implicit_systems",
            1,
            crate::model::Authority::Derived,
            S::Derived,
            "blueprint §6.9 math: math_implicit_systems.",
        )
        .pk(&["implicit_system_id"])
        .granularity(crate::model::DerivationGranularity::Row)
        .checks(std::collections::BTreeMap::from([(
            "implicit_cardinality".into(),
            "array_length(unknown_symbol_ids) = array_length(equation_ids)".into(),
        )]))
        .columns(vec![
            T::key("implicit_system_id", T::id(), "Implicit system identity."),
            column(
                "unknown_symbol_ids",
                T::list(T::id()).with_collection(crate::model::CollectionContract {
                    unique: true,
                    ..crate::model::CollectionContract::SEQUENCE
                }),
            ),
            column(
                "equation_ids",
                T::list(T::id()).with_collection(crate::model::CollectionContract {
                    unique: true,
                    ..crate::model::CollectionContract::SEQUENCE
                }),
            ),
            column("branch_policy", T::native(arrow_schema::DataType::Utf8)),
            column("kernel_binding_id", T::id()).optional(),
        ]),
    );
}

fn declare_compiled_math_complementarity(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "math_complementarity",
        S::Derived,
        &["pair_id"],
        vec![
            column("pair_id", T::id()),
            column("expr_a_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("expr_b_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("formulation", T::enumeration("ComplementarityForm")),
        ],
        "blueprint §6.9 math: math_complementarity.",
    );
}

fn declare_compiled_math_dae_links(builder: &mut RegistryBuilder) {
    super::declarations::relation_version(
        builder,
        N::Compiled,
        "math_dae_links",
        2,
        S::Derived,
        &["derivative_symbol_id"],
        vec![
            column("derivative_symbol_id", T::id()),
            column("state_symbol_id", T::id()),
            column("wrt_domain_id", T::id()),
            column("derivative_order", T::native(arrow_schema::DataType::UInt8)),
        ],
        "blueprint §6.9 math: explicit actual state/derivative/domain and positive derivative order.",
    );
}

fn declare_sense_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Sense",
        crate::math::Sense::ALL.iter().map(|sense| sense.as_str()),
    );
}

fn declare_objective_sense_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "ObjectiveSense", ["minimize", "maximize"]);
}

fn declare_complementarity_form_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ComplementarityForm",
        ["smooth_eps", "binary", "sos1"],
    );
}
