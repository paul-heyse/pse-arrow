// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §7.3 operator.

use super::declarations::{column, relation, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §7.3 operator contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_operator_specs(builder);
    declare_operator_family_vocabulary(builder);
    declare_argument_evaluation_vocabulary(builder);
    declare_kernel_failure_vocabulary(builder);
    declare_differentiability_vocabulary(builder);
    declare_relation_op_vocabulary(builder);
    declare_backend_binding_vocabulary(builder);
    declare_arity_vocabulary(builder);
}

use crate::model::Cell;

/// Projects the sole operator table; no quantity package is bound in the schema registry.
pub(crate) fn rows() -> Vec<Vec<Cell>> {
    pse_mathir::OPERATOR_TABLE
        .iter()
        .map(|spec| {
            let (arity, count) = match spec.arity {
                pse_mathir::Arity::Fixed(count) => ("fixed", Cell::U64(u64::from(count))),
                pse_mathir::Arity::Variadic => ("variadic", Cell::Null),
                pse_mathir::Arity::Payload => ("payload", Cell::Null),
            };
            vec![
                Cell::Enum(spec.opcode.as_str()),
                Cell::Enum(arity),
                count,
                Cell::List(Vec::new()),
                Cell::text(spec.shape_rule),
                Cell::text(spec.derivative_rule),
                Cell::Enum(spec.argument_evaluation.as_str()),
                Cell::List(
                    spec.failure_classes
                        .iter()
                        .map(|v| Cell::Enum(v.as_str()))
                        .collect(),
                ),
                Cell::List(
                    spec.domain_restrictions
                        .iter()
                        .map(|r| {
                            Cell::Struct(vec![
                                Cell::U64(u64::from(r.argument)),
                                Cell::Enum(r.relation.as_str()),
                                Cell::F64(r.bound),
                            ])
                        })
                        .collect(),
                ),
                Cell::text(spec.domain_rule),
                Cell::Enum(spec.smoothness.as_str()),
                Cell::text(spec.convexity_rule),
                Cell::text(spec.monotonicity_rule),
                Cell::text(spec.sparsity_rule),
                Cell::List(
                    spec.rewrite_conditions
                        .iter()
                        .map(|v| Cell::text(*v))
                        .collect(),
                ),
                Cell::List(
                    spec.lowering
                        .iter()
                        .map(|v| Cell::Enum(v.as_str()))
                        .collect(),
                ),
                Cell::Enum(spec.family.as_str()),
                Cell::Bool(spec.foldable),
            ]
        })
        .collect()
}

fn declare_reference_operator_specs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "operator_specs",
        S::Model,
        &["opcode"],
        vec![
            column("opcode", T::enumeration("Opcode")),
            column("arity", T::enumeration("Arity")),
            column("fixed_arity", T::native(arrow_schema::DataType::UInt8)).optional(),
            column("quantity_operation_ids", T::list(T::id())),
            column("shape_rule", T::native(arrow_schema::DataType::Utf8)),
            column("derivative_rule", T::native(arrow_schema::DataType::Utf8)),
            column("argument_evaluation", T::enumeration("ArgumentEvaluation")),
            column("failure_classes", T::list(T::enumeration("KernelFailure"))),
            column(
                "domain_restrictions",
                T::list(structure(vec![
                    ("argument", T::native(arrow_schema::DataType::UInt16)),
                    ("relation", T::enumeration("RelationOp")),
                    ("bound", T::native(arrow_schema::DataType::Float64)),
                ])),
            ),
            column("domain_rule", T::native(arrow_schema::DataType::Utf8)),
            column("smoothness", T::enumeration("Differentiability")),
            column("convexity_rule", T::native(arrow_schema::DataType::Utf8)),
            column("monotonicity_rule", T::native(arrow_schema::DataType::Utf8)),
            column("sparsity_rule", T::native(arrow_schema::DataType::Utf8)),
            column(
                "rewrite_conditions",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("lowering", T::list(T::enumeration("BackendBinding"))),
            column("family", T::enumeration("OperatorFamily")),
            column("foldable", T::native(arrow_schema::DataType::Boolean)),
        ],
        "blueprint §7.3 operator: operator_specs.",
    );
}

fn declare_operator_family_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "OperatorFamily",
        pse_mathir::opspec::OperatorFamily::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_argument_evaluation_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ArgumentEvaluation",
        pse_mathir::opspec::ArgumentEvaluation::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_kernel_failure_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "KernelFailure",
        pse_mathir::opspec::KernelFailure::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_differentiability_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Differentiability",
        pse_mathir::opspec::Differentiability::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_relation_op_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "RelationOp",
        pse_mathir::opspec::RelationOp::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_backend_binding_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "BackendBinding",
        pse_mathir::opspec::BackendBinding::ALL
            .iter()
            .map(|v| v.as_str()),
    );
}

fn declare_arity_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "Arity", ["fixed", "variadic", "payload"]);
}
