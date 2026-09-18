// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.11 numerical and rule.

use super::declarations::{column, relation, relation_version, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.11 numerical and rule contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_algorithm_specs(builder);
    declare_reference_algorithm_arguments(builder);
    declare_reference_algorithm_results(builder);
    declare_reference_rule_specs(builder);
    declare_reference_rule_dependencies(builder);
    declare_reference_engine_profiles(builder);
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
            column("stratum", T::nonnegative(i64::from(u16::MAX))),
            column("head_relation_id", T::id()),
            column("assertion_relation_id", T::id()).optional(),
            column(
                "queries",
                T::list(T::structure(vec![
                    column("truth", T::native(arrow_schema::DataType::Utf8)),
                    column("sql", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column("negation", T::enumeration("NegationPolicy")),
            column("monotonic", T::native(arrow_schema::DataType::Boolean)),
            column("conflict_policy", T::enumeration("ConflictPolicy")),
        ],
        "blueprint §6.11 numerical and rule: rule_specs.",
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
            column("stratum", T::nonnegative(i64::from(u16::MAX))),
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
    super::declarations::enumeration(builder, "KernelParameterBindingKind", ["symbol", "literal"]);
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
                    T::structure(vec![
                        T::enumeration("KernelParameterBindingKind").with_name("kind"),
                        T::structure(vec![T::id().with_name("symbol_id")])
                            .with_name("symbol")
                            .optional(),
                        T::structure(vec![
                            T::native(arrow_schema::DataType::Float64).with_name("value"),
                            T::id().with_name("unit_id"),
                        ])
                        .with_name("literal")
                        .optional(),
                    ])
                    .with_alternative(&crate::model::TaggedAlternative::new(
                        "kind",
                        [
                            ("symbol".into(), "symbol".into()),
                            ("literal".into(), "literal".into()),
                        ],
                    ))
                    .with_name("binding"),
                ])),
            ),
            column(
                "input_bindings",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("node_id", T::nonnegative(i64::MAX)),
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
            column("finite_elements", T::nonnegative(i64::from(u32::MAX))),
            column("collocation_points", T::nonnegative(i64::from(u8::MAX))).optional(),
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
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
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
