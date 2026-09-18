// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.9 math.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.9 math contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    super::math_value::declare(builder);
    declare_equation_vocabularies(builder);
    declare_builtin_quantity_rule_vocabulary(builder);
    declare_compiled_math_expr_nodes(builder);
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
    let target = "compiled.math_expr_nodes";
    builder.declare_relation(
        crate::model::RelationDecl::new(
            N::Compiled,
            "math_expr_nodes",
            1,
            crate::model::Authority::Derived,
            S::Derived,
            "Coherent mathematical nodes with ordered children and tagged payloads.",
        )
        .pk(&["node_id"])
        .granularity(crate::model::DerivationGranularity::Row)
        .checks(super::math_value::checks(N::Compiled))
        .columns(vec![
            T::key(
                "node_id",
                T::nonnegative(i64::MAX),
                "Node identity in the selected expression family.",
            ),
            column("opcode", T::enumeration("Opcode")),
            column(
                "children",
                T::list(T::nonnegative(i64::MAX).with_fk(target, "node_id")),
            ),
            column("payload", super::math_value::payload(target)),
            column("quantity_type_id", T::id()).optional(),
            column("scope_instance_id", T::id()).optional(),
            column("subtree_hash", T::hash()),
        ]),
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
            column("node_id", T::nonnegative(i64::MAX)),
            column("operation_id", T::id()).optional(),
            column("builtin_rule", T::enumeration("BuiltinQuantityRule")).optional(),
            column(
                "operand_permutation",
                T::list(T::nonnegative(i64::from(u16::MAX))),
            ),
            column(
                "conversions",
                T::list(structure(vec![
                    ("operand", T::nonnegative(i64::from(u16::MAX))),
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
            column("filter_node_id", T::nonnegative(i64::MAX)).optional(),
            column("body_node_id", T::nonnegative(i64::MAX)),
            column(
                "constraint",
                equation_constraint("compiled.math_expr_nodes"),
            ),
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
            column("position", T::nonnegative(i64::from(u16::MAX))),
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
            column("ordinal", T::nonnegative(i64::MAX)),
            column("owner_instance_id", T::id()),
            column("equation_decl_id", T::id()).optional(),
            column("parent_indexed_equation_id", T::id()).optional(),
            column("qualified_name", T::native(arrow_schema::DataType::Utf8)),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("body_node_id", T::nonnegative(i64::MAX)),
            column(
                "constraint",
                equation_constraint("compiled.math_expr_nodes"),
            ),
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
            column("body_node_id", T::nonnegative(i64::MAX)),
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
            column("expr_a_node_id", T::nonnegative(i64::MAX)),
            column("expr_b_node_id", T::nonnegative(i64::MAX)),
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
            column("derivative_order", T::nonnegative(i64::from(u8::MAX))),
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

/// Bound shape and node closure travel with the equation's comparison sense.
pub(super) fn equation_constraint(nodes: &str) -> T {
    let node = || T::nonnegative(i64::MAX).with_fk(nodes, "node_id");
    T::structure(vec![
        T::enumeration("Sense").with_name("kind"),
        T::structure(vec![node().with_name("node_id")])
            .with_name("single")
            .optional(),
        T::structure(vec![
            node().with_name("lower_node_id"),
            node().with_name("upper_node_id"),
        ])
        .with_name("range")
        .optional(),
    ])
    .with_alternative(&crate::model::TaggedAlternative::new(
        "kind",
        [
            ("eq".into(), "single".into()),
            ("le".into(), "single".into()),
            ("ge".into(), "single".into()),
            ("definition".into(), "single".into()),
            ("range".into(), "range".into()),
        ],
    ))
}
