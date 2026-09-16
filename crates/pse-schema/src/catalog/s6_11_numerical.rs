// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.11 numerical and rule.

use super::declarations::{column, relation, relation_version, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.11 numerical and rule contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_pass_specs(builder);
    declare_reference_pass_input_ports(builder);
    declare_reference_pass_output_ports(builder);
    declare_reference_rule_specs(builder);
    declare_reference_rule_plan_nodes(builder);
    declare_reference_rule_expr_nodes(builder);
    declare_reference_rule_expr_edges(builder);
    declare_reference_rule_expr_calls(builder);
    declare_reference_rule_aggregates(builder);
    declare_reference_rule_group_keys(builder);
    declare_reference_rule_unnest(builder);
    declare_reference_rule_plan_edges(builder);
    declare_reference_rule_dependencies(builder);
    declare_reference_engine_profiles(builder);
    declare_compiled_stage_bundles(builder);
    declare_reference_kernel_specs(builder);
    declare_reference_numerical_policies(builder);
    declare_compiled_kernel_bindings(builder);
    declare_authored_discretization_policies(builder);
    declare_authored_solver_profiles(builder);
    declare_compiled_variable_scales(builder);
    declare_compiled_equation_scales(builder);
    declare_compiled_initialization_plans(builder);
    declare_compiled_init_stages(builder);
    declare_compiled_solve_plans(builder);
    declare_solve_plan_class_vocabulary(builder);
    declare_kernel_behavior_vocabulary(builder);
    declare_kernel_null_policy_vocabulary(builder);
    declare_kernel_failure_policy_vocabulary(builder);
    declare_kernel_effects_vocabulary(builder);
    declare_monotonicity_vocabulary(builder);
    declare_convexity_vocabulary(builder);
    declare_derivative_kind_vocabulary(builder);
    declare_execution_form_vocabulary(builder);
    declare_reassociation_policy_vocabulary(builder);
    declare_fma_policy_vocabulary(builder);
    declare_nonfinite_policy_vocabulary(builder);
    declare_rewrite_mode_vocabulary(builder);
    declare_discretization_method_vocabulary(builder);
    declare_backend_vocabulary(builder);
    declare_scaling_mode_vocabulary(builder);
    declare_derivative_mode_vocabulary(builder);
    declare_scale_source_vocabulary(builder);
    declare_stage_kind_vocabulary(builder);
    declare_stage_target_kind_vocabulary(builder);
    declare_failure_policy_vocabulary(builder);
}

fn declare_reference_pass_specs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "pass_specs",
        S::Model,
        &["pass_id"],
        vec![
            column("pass_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("preconditions", T::list(T::id())),
            column("postconditions", T::list(T::id())),
            column("determinism", T::enumeration("Determinism")),
            column("diagnostics", T::list(T::enumeration("FailureClass"))),
            column("effects", T::list(T::enumeration("OperationEffect"))),
        ],
        "blueprint §6.11 numerical and rule: pass_specs.",
    );
}

fn declare_reference_pass_input_ports(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "pass_input_ports",
        S::Model,
        &["pass_id", "port"],
        vec![
            column("pass_id", T::id()),
            column("port", T::native(arrow_schema::DataType::Utf8)),
            column("relation_id", T::id()),
            column("source_pass_id", T::id()).optional(),
            column("source_port", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("required", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §6.11 numerical and rule: pass_input_ports.",
    );
}

fn declare_reference_pass_output_ports(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "pass_output_ports",
        S::Model,
        &["pass_id", "port"],
        vec![
            column("pass_id", T::id()),
            column("port", T::native(arrow_schema::DataType::Utf8)),
            column("relation_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: pass_output_ports.",
    );
}

fn declare_reference_rule_specs(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Reference,
        "rule_specs",
        2,
        S::Model,
        &["rule_id"],
        vec![
            column("rule_id", T::id()),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("stratum", T::native(arrow_schema::DataType::UInt16)),
            column("head_relation_id", T::id()),
            column("assertion_relation_id", T::id()).optional(),
            column("root_node_id", T::id()),
            column("negation", T::enumeration("NegationPolicy")),
            column("monotonic", T::native(arrow_schema::DataType::Boolean)),
            column("conflict_policy", T::enumeration("ConflictPolicy")),
            column("head_kind", T::enumeration("RuleHeadKind")),
            column(
                "head_key_columns",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
        ],
        "blueprint §6.11 numerical and rule: rule_specs.",
    );
}

fn declare_reference_rule_plan_nodes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_plan_nodes",
        S::Model,
        &["node_id"],
        vec![
            column("node_id", T::id()),
            column("rule_id", T::id()),
            column("op", T::enumeration("RulePlanOp")),
            column("relation_id", T::id()).optional(),
            column("input_port", T::native(arrow_schema::DataType::Utf8)).optional(),
            column(
                "join_keys",
                T::list(structure(vec![
                    ("left", T::native(arrow_schema::DataType::Utf8)),
                    ("right", T::native(arrow_schema::DataType::Utf8)),
                ])),
            )
            .optional(),
            column("predicate_expr_id", T::id()).optional(),
            column(
                "projection",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("expr_id", T::id()),
                ])),
            )
            .optional(),
            column("kernel_binding_id", T::id()).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
            column("null_equality", T::enumeration("NullEquality")).optional(),
            column("recursive_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("recursive_target_node_id", T::id()).optional(),
            column("is_distinct", T::native(arrow_schema::DataType::Boolean)).optional(),
            column("depth_bound", T::enumeration("DepthBound")).optional(),
            column("depth_limit", T::native(arrow_schema::DataType::UInt32)).optional(),
        ],
        "blueprint §6.11 numerical and rule: rule_plan_nodes.",
    );
}

fn declare_reference_rule_expr_nodes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_expr_nodes",
        S::Model,
        &["expr_id"],
        vec![
            column("expr_id", T::id()),
            column("rule_id", T::id()),
            column("op", T::enumeration("RuleExprOp")),
            column("column_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("comparison_op", T::enumeration("RuleCmpOp")).optional(),
            column("field_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("literal_kind", T::enumeration("RuleLiteralKind")).optional(),
            column("bool_value", T::native(arrow_schema::DataType::Boolean)).optional(),
            column("i64_value", T::native(arrow_schema::DataType::Int64)).optional(),
            column("u64_value", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("f64_bits", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("text_value", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("id_value", T::id()).optional(),
            column("hash_value", T::hash()).optional(),
        ],
        "blueprint §6.11 numerical and rule: rule_expr_nodes.",
    );
}

fn declare_reference_rule_expr_edges(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_expr_edges",
        S::Model,
        &["parent_expr_id", "ordinal"],
        vec![
            column("parent_expr_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt32)),
            column("child_expr_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: rule_expr_edges.",
    );
}

fn declare_reference_rule_expr_calls(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_expr_calls",
        S::Model,
        &["expr_id"],
        vec![
            column("expr_id", T::id()).with_fk("reference.rule_expr_nodes", "expr_id"),
            column("function_name", T::native(arrow_schema::DataType::Utf8)),
            column("result_type", T::native(arrow_schema::DataType::Utf8)),
            column(
                "result_nullable",
                T::native(arrow_schema::DataType::Boolean),
            ),
        ],
        "Native function-call output obligations; actual retained implementations and return fields are checked during planning.",
    );
}

fn declare_reference_rule_aggregates(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_aggregates",
        S::Model,
        &["node_id", "ordinal"],
        vec![
            column("node_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("function", T::enumeration("RuleAggregate")),
            column("input_expr_id", T::id()).optional(),
            column("output_name", T::native(arrow_schema::DataType::Utf8)),
            column(
                "order_by",
                T::list(structure(vec![
                    ("column", T::native(arrow_schema::DataType::Utf8)),
                    ("ascending", T::native(arrow_schema::DataType::Boolean)),
                ])),
            ),
            column("null_policy", T::enumeration("AggregateNullPolicy")),
            column("empty_policy", T::enumeration("AggregateEmptyPolicy")),
        ],
        "blueprint §6.11 numerical and rule: rule_aggregates.",
    );
}

fn declare_reference_rule_group_keys(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_group_keys",
        S::Model,
        &["node_id", "ordinal"],
        vec![
            column("node_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("column", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.11 numerical and rule: rule_group_keys.",
    );
}

fn declare_reference_rule_unnest(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_unnest",
        S::Model,
        &["node_id"],
        vec![
            column("node_id", T::id()),
            column("column", T::native(arrow_schema::DataType::Utf8)),
            column("value_name", T::native(arrow_schema::DataType::Utf8)),
            column("null_list_policy", T::enumeration("NullListPolicy")),
            column("empty_list_policy", T::enumeration("EmptyListPolicy")),
        ],
        "blueprint §6.11 numerical and rule: rule_unnest.",
    );
}

fn declare_reference_rule_plan_edges(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_plan_edges",
        S::Model,
        &["parent_node_id", "ordinal"],
        vec![
            column("parent_node_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("child_node_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: rule_plan_edges.",
    );
}

fn declare_reference_rule_dependencies(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "rule_dependencies",
        S::Model,
        &["rule_id", "relation_id", "mode", "derivation_id"],
        vec![
            column("rule_id", T::id()),
            column("relation_id", T::id()),
            column("input_port", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("mode", T::enumeration("DependencyMode")),
            column("stratum", T::native(arrow_schema::DataType::UInt16)),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: rule_dependencies.",
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

fn declare_compiled_stage_bundles(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "stage_bundles",
        S::Sidecar,
        &["bundle_id"],
        vec![
            column("bundle_id", T::hash()),
            column("pass_id", T::id()),
            column("pass_version", T::native(arrow_schema::DataType::Utf8)),
            column(
                "members",
                T::list(structure(vec![
                    ("port", T::native(arrow_schema::DataType::Utf8)),
                    ("relation_id", T::id()),
                    ("schema_version", T::native(arrow_schema::DataType::UInt32)),
                    ("logical_hash", T::hash()),
                ])),
            ),
        ],
        "blueprint §6.11 numerical and rule: stage_bundles.",
    );
}

fn declare_reference_kernel_specs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "kernel_specs",
        S::Model,
        &["kernel_id"],
        vec![
            column("kernel_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("provider", T::native(arrow_schema::DataType::Utf8)),
            column("artifact_digest", T::hash()),
            column("behavior", T::enumeration("KernelBehavior")),
            column(
                "inputs",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("quantity_type_id", T::id()),
                    ("natural_unit_id", T::id()),
                    ("nullable", T::native(arrow_schema::DataType::Boolean)),
                    ("shape", T::list(T::enumeration("DomainKind"))),
                ])),
            ),
            column(
                "outputs",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("quantity_type_id", T::id()),
                    ("natural_unit_id", T::id()),
                    ("shape", T::list(T::enumeration("DomainKind"))),
                ])),
            ),
            column(
                "parameters",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("quantity_type_id", T::id()),
                    ("natural_unit_id", T::id()),
                    ("indexed_by", T::list(T::enumeration("DomainKind"))),
                ])),
            ),
            column("null_input_policy", T::enumeration("KernelNullPolicy")),
            column("failure_policy", T::enumeration("KernelFailurePolicy")),
            column("tolerant_batch", T::native(arrow_schema::DataType::Boolean)),
            column("effects", T::enumeration("KernelEffects")),
            column("argument_evaluation", T::enumeration("ArgumentEvaluation")),
            column("smoothness", T::enumeration("Differentiability")),
            column(
                "validity",
                T::list(structure(vec![
                    ("input", T::native(arrow_schema::DataType::Utf8)),
                    ("lower", T::extended(crate::model::ExtensionUse::Bound)),
                    ("upper", T::extended(crate::model::ExtensionUse::Bound)),
                ])),
            ),
            column(
                "monotonicity",
                T::list(structure(vec![
                    ("input", T::native(arrow_schema::DataType::Utf8)),
                    ("direction", T::enumeration("Monotonicity")),
                ])),
            ),
            column("convexity", T::enumeration("Convexity")),
            column(
                "derivative_bindings",
                T::list(structure(vec![
                    ("kind", T::enumeration("DerivativeKind")),
                    ("implementation_id", T::id()),
                    ("validity_invariant_ids", T::list(T::id())),
                ])),
            ),
            column(
                "execution_forms",
                T::list(structure(vec![
                    ("form", T::enumeration("ExecutionForm")),
                    ("implementation_id", T::id()),
                ])),
            ),
            column("thread_safe", T::native(arrow_schema::DataType::Boolean)),
            column("failure_classes", T::list(T::enumeration("KernelFailure"))),
            column(
                "bindings",
                T::list(structure(vec![
                    ("backend", T::enumeration("BackendBinding")),
                    ("implementation_id", T::id()),
                    ("conformance_suite", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column("nl_function_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
            column("test_suite", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.11 numerical and rule: kernel_specs.",
    );
}

fn declare_reference_numerical_policies(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "numerical_policies",
        S::Model,
        &["policy_id"],
        vec![
            column("policy_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("reassociation", T::enumeration("ReassociationPolicy")),
            column("fma", T::enumeration("FmaPolicy")),
            column("nonfinite", T::enumeration("NonfinitePolicy")),
            column("rewrite_mode", T::enumeration("RewriteMode")),
            column(
                "absolute_tolerance",
                T::native(arrow_schema::DataType::Float64),
            ),
            column(
                "relative_tolerance",
                T::native(arrow_schema::DataType::Float64),
            ),
            column(
                "signed_zero_equal",
                T::native(arrow_schema::DataType::Boolean),
            ),
        ],
        "blueprint §6.11 numerical and rule: numerical_policies.",
    );
}

fn declare_compiled_kernel_bindings(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "kernel_bindings",
        S::Derived,
        &["binding_id"],
        vec![
            column("binding_id", T::id()),
            column("kernel_id", T::id()),
            column("scope_instance_id", T::id()),
            column(
                "parameter_bindings",
                T::list(T::structure(vec![
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("name")
                        .with_nullable(false),
                    T::id().with_name("symbol_id").with_nullable(true),
                    T::native(arrow_schema::DataType::Float64)
                        .with_name("value")
                        .with_nullable(true),
                    T::id().with_name("unit_id").with_nullable(true),
                ])),
            ),
            column(
                "input_bindings",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("node_id", T::native(arrow_schema::DataType::UInt64)),
                ])),
            ),
        ],
        "blueprint §6.11 numerical and rule: kernel_bindings.",
    );
}

fn declare_authored_discretization_policies(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "discretization_policies",
        S::Model,
        &["policy_id"],
        vec![
            column("policy_id", T::id()),
            column("package_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("method", T::enumeration("DiscretizationMethod")),
            column("scheme", T::enumeration("DiscretizationScheme")),
            column("finite_elements", T::native(arrow_schema::DataType::UInt32)),
            column(
                "collocation_points",
                T::native(arrow_schema::DataType::UInt8),
            )
            .optional(),
        ],
        "blueprint §6.11 numerical and rule: discretization_policies.",
    );
}

fn declare_authored_solver_profiles(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "solver_profiles",
        S::Model,
        &["profile_id"],
        vec![
            column("profile_id", T::id()),
            column("package_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("backend", T::enumeration("Backend")),
            column("solver", T::native(arrow_schema::DataType::Utf8)),
            column(
                "options",
                T::list(structure(vec![
                    ("key", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column("scaling_mode", T::enumeration("ScalingMode")),
            column("derivative_mode", T::enumeration("DerivativeMode")),
        ],
        "blueprint §6.11 numerical and rule: solver_profiles.",
    );
}

fn declare_compiled_variable_scales(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "variable_scales",
        S::Derived,
        &["symbol_id"],
        vec![
            column("symbol_id", T::id()),
            column("scale", T::native(arrow_schema::DataType::Float64)),
            column("offset", T::native(arrow_schema::DataType::Float64)),
            column("source", T::enumeration("ScaleSource")),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: variable_scales.",
    );
}

fn declare_compiled_equation_scales(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "equation_scales",
        S::Derived,
        &["equation_id"],
        vec![
            column("equation_id", T::id()),
            column("scale", T::native(arrow_schema::DataType::Float64)),
            column("scheme", T::enumeration("ConstraintScalingScheme")),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.11 numerical and rule: equation_scales.",
    );
}

fn declare_compiled_initialization_plans(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "initialization_plans",
        S::Derived,
        &["plan_id"],
        vec![
            column("plan_id", T::id()),
            column("case_id", T::id()),
            column("template_id", T::id()),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.11 numerical and rule: initialization_plans.",
    );
}

fn declare_compiled_init_stages(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "init_stages",
        S::Derived,
        &["stage_id"],
        vec![
            column("stage_id", T::id()),
            column("plan_id", T::id()),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("kind", T::enumeration("StageKind")),
            column("target_kind", T::enumeration("StageTargetKind")),
            column("target_ids", T::list(T::id())),
            column("overlay_case_id", T::id()).optional(),
            column("solver_profile_id", T::id()).optional(),
            column("tolerance", T::native(arrow_schema::DataType::Float64)).optional(),
            column("failure_policy", T::enumeration("FailurePolicy")),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.11 numerical and rule: init_stages.",
    );
}

fn declare_compiled_solve_plans(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "solve_plans",
        S::Derived,
        &["plan_id"],
        vec![
            column("plan_id", T::id()),
            column("problem_id", T::id()),
            column("class", T::enumeration("SolvePlanClass")),
            column("justification", T::native(arrow_schema::DataType::Utf8)),
            column(
                "modifiers",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
        ],
        "blueprint §6.11 and §18.7: solve class selected from actual static problem attributes, with justification and ordered modifiers.",
    );
}

fn declare_solve_plan_class_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SolvePlanClass",
        ["SQUARE_NLE", "NLP_LOCAL", "DAE_INTEGRATE", "MINLP", "GDP"],
    );
}

fn declare_kernel_behavior_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "KernelBehavior", ["explicit", "implicit"]);
}

fn declare_kernel_null_policy_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "KernelNullPolicy", ["reject", "propagate_missing"]);
}

fn declare_kernel_failure_policy_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "KernelFailurePolicy", ["typed_error"]);
}

fn declare_kernel_effects_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "KernelEffects", ["pure"]);
}

fn declare_monotonicity_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Monotonicity",
        [
            "increasing",
            "decreasing",
            "constant",
            "nonmonotone",
            "unknown",
        ],
    );
}

fn declare_convexity_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Convexity",
        ["affine", "convex", "concave", "nonconvex", "unknown"],
    );
}

fn declare_derivative_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "DerivativeKind",
        ["jacobian", "hessian", "jvp", "vjp"],
    );
}

fn declare_execution_form_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ExecutionForm",
        ["scalar", "batch_arrow", "dual", "hyperdual"],
    );
}

fn declare_reassociation_policy_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ReassociationPolicy",
        ["preserve_order", "canonical_order"],
    );
}

fn declare_fma_policy_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "FmaPolicy", ["disabled", "explicit"]);
}

fn declare_nonfinite_policy_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "NonfinitePolicy", ["evaluation_error"]);
}

fn declare_rewrite_mode_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "RewriteMode", ["guarded_floating"]);
}

fn declare_discretization_method_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "DiscretizationMethod",
        ["finite_difference", "collocation"],
    );
}

fn declare_backend_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Backend",
        [
            "native_ipopt",
            "nl_ipopt",
            "nl_bonmin",
            "nl_couenne",
            "nl_cbc",
            "nl_scip",
            "nl_petsc_snes",
            "nl_petsc_ts",
            "pyomo",
        ],
    );
}

fn declare_scaling_mode_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "ScalingMode", ["user", "gradient_based", "none"]);
}

fn declare_derivative_mode_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "DerivativeMode",
        ["exact_hessian", "limited_memory"],
    );
}

fn declare_scale_source_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ScaleSource",
        ["case", "package", "nominal", "rule"],
    );
}

fn declare_stage_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "StageKind",
        [
            "solve_subset",
            "solve_blocks",
            "propagate",
            "apply_overlay",
            "continuation",
            "call_plan",
            "check",
            "restore",
        ],
    );
}

fn declare_stage_target_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "StageTargetKind",
        ["instance", "equation_set", "blocks", "connection", "all"],
    );
}

fn declare_failure_policy_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "FailurePolicy",
        ["abort", "continue", "retry_with_profile"],
    );
}
