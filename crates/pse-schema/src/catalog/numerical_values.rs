// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact prepared-program dimensions and scenario-vector results.
use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{
        Authority, CollectionContract, DerivationGranularity, FieldContract as T, Namespace as N,
        RelationDecl, SnapshotClass as S,
    },
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    solver_outcomes(builder);
    relation(
        builder,
        N::Runtime,
        "numerical_programs",
        S::Sidecar,
        &["program_id"],
        vec![
            column("program_id", T::id()),
            column("residual_dimension", T::nonnegative(i64::MAX)),
            column(
                "variable_columns",
                T::list(T::native(arrow_schema::DataType::Utf8)).with_collection(
                    CollectionContract {
                        unique: true,
                        ..CollectionContract::SEQUENCE
                    },
                ),
            ),
            column("jacobian_dimension", T::nonnegative(i64::MAX)),
        ],
        "Exact ordered variable columns and residual/Jacobian dimensions of one prepared program.",
    );
    relation(
        builder,
        N::Runtime,
        "jacobian_coordinates",
        S::Sidecar,
        &["program_id", "ordinal"],
        vec![
            column(
                "program_id",
                T::id().with_fk("runtime.numerical_programs", "program_id"),
            ),
            column("ordinal", T::nonnegative(i64::MAX)),
            column("residual", T::nonnegative(i64::MAX)),
            column("variable", T::nonnegative(i64::MAX)),
        ],
        "Ordered sparse coordinates bound to one exact prepared numerical program.",
    );
    builder.declare_relation(RelationDecl::new(N::Runtime, "numerical_evaluations", 1, Authority::Derived, S::Sidecar,
        "Scenario values in exact prepared-program order. Empty vectors represent zero dimensions.")
        .pk(&["program_id", "scenario_id"])
        .granularity(DerivationGranularity::Row)
        .columns(vec![
            T::key("program_id", T::id().with_fk("runtime.numerical_programs", "program_id"), "Exact prepared program."),
            T::key("scenario_id", T::id(), "Caller-supplied scenario identity."),
            column("residual_dimension", T::nonnegative(i64::MAX)),
            column("variable_dimension", T::nonnegative(i64::MAX)),
            column("jacobian_dimension", T::nonnegative(i64::MAX)),
            column("residuals", T::list(T::native(arrow_schema::DataType::Float64))),
            column("jacobian", T::list(T::native(arrow_schema::DataType::Float64))),
        ]).checks([
            ("residual_extent".into(), "array_length(residuals) = residual_dimension".into()),
            ("jacobian_extent".into(), "array_length(jacobian) = jacobian_dimension".into()),
        ].into()));
}

fn solver_outcomes(builder: &mut RegistryBuilder) {
    use super::declarations::enumeration;
    use arrow_schema::DataType;
    enumeration(
        builder,
        "SolverTermination",
        ["success", "stopped", "cancelled", "evaluation_failure"],
    );
    let vector = || T::list(T::native(DataType::Float64).optional());
    let iteration = T::structure(vec![
        T::nonnegative(i64::from(i32::MAX)).with_name("iteration"),
        T::native(DataType::Boolean).with_name("restoration"),
        T::native(DataType::Float64)
            .optional()
            .with_name("objective"),
        T::native(DataType::Float64)
            .optional()
            .with_name("primal_infeasibility"),
        T::native(DataType::Float64)
            .optional()
            .with_name("dual_infeasibility"),
        T::native(DataType::Float64).optional().with_name("barrier"),
        T::native(DataType::Float64).optional().with_name("step"),
    ]);
    builder.declare_relation(RelationDecl::new(N::Runtime, "solver_outcomes", 1, Authority::Derived, S::Sidecar,
        "Actual Ipopt outcome for one program/scenario. Null numeric values are unavailable or nonfinite; they never certify success. Objective is program residual zero; constraints follow in program order.")
        .pk(&["program_id", "scenario_id"])
        .granularity(DerivationGranularity::Row)
        .columns(vec![
            T::key("program_id", T::id().with_fk("runtime.numerical_programs", "program_id"), "Exact prepared program."),
            T::key("scenario_id", T::id(), "Exact input scenario."),
            column("variable_columns", T::list(T::native(DataType::Utf8)).with_collection(CollectionContract { unique: true, ..CollectionContract::SEQUENCE })),
            column("constraint_dimension", T::nonnegative(i64::MAX)),
            column("ipopt_status", T::native(DataType::Int64)),
            column("termination", T::enumeration("SolverTermination")),
            column("objective", T::native(DataType::Float64)).optional(),
            column("values", vector()),
            column("constraints", vector()),
            column("constraint_duals", vector()),
            column("lower_duals", vector()),
            column("upper_duals", vector()),
            column("diagnostic", T::structure(vec![
                T::native(DataType::Utf8).with_name("code").optional(),
                T::native(DataType::Utf8).with_name("message"),
            ])).optional(),
            column("iterations", T::list(iteration)),
        ]).checks([
            ("variable_extents".into(), "array_length(values) = array_length(variable_columns) AND array_length(lower_duals) = array_length(variable_columns) AND array_length(upper_duals) = array_length(variable_columns)".into()),
            ("constraint_extents".into(), "array_length(constraints) = constraint_dimension AND array_length(constraint_duals) = constraint_dimension".into()),
            ("successful_values".into(), "termination <> 'success' OR (ipopt_status IN (0, 1, 6) AND objective IS NOT NULL AND diagnostic IS NULL AND array_length(array_compact(values)) = array_length(values) AND array_length(array_compact(constraints)) = array_length(constraints) AND array_length(array_compact(constraint_duals)) = array_length(constraint_duals) AND array_length(array_compact(lower_duals)) = array_length(lower_duals) AND array_length(array_compact(upper_duals)) = array_length(upper_duals))".into()),
        ].into()));
}
