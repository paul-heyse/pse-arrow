// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic physical operation prerequisites and explicit mathematical context.
use super::declarations::{column,enumeration,relation};
use crate::{RegistryBuilder,model::{FieldContract as T,Namespace as N,SnapshotClass as S}};

/// Declare physical prerequisites consumed by quantity checking.
pub fn declare(builder:&mut RegistryBuilder){
    relation(
        builder,
        N::Reference,
        "quantity_operation_reductions",
        S::Model,
        &["operation_id"],
        vec![
            column("operation_id", T::id())
                .with_fk("reference.quantity_operations", "operation_id"),
            column("domain_kind", T::id()),
        ],
        "Exact structural dispatch domain for a registered reduction; physical precondition failure never permits a fallback.",
    );
    enumeration(
        builder,
        "QuantityPreconditionKind",
        [
            "equal_operand_bases",
            "operand_quantity_contract",
            "same_reference_differences",
        ],
    );
    relation(
        builder,
        N::Reference,
        "quantity_preconditions",
        S::Model,
        &["invariant_id"],
        vec![
            column("invariant_id", T::id()),
            column("kind", T::enumeration("QuantityPreconditionKind")),
            column(
                "operand_positions",
                T::list(T::nonnegative(i64::from(u16::MAX))),
            ),
            column("required_basis_id", T::id())
                .optional()
                .with_fk("reference.bases", "basis_id"),
            column("required_quantity_type_id", T::id())
                .optional()
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("match_shape", T::native(arrow_schema::DataType::Boolean)).optional(),
        ],
        "Operation preconditions proved from exact operand contracts at application time, never by invariant identity alone.",
    );

    relation(
        builder,
        N::Reference,
        "math_context",
        S::Model,
        &["package_id"],
        vec![
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("neutral_quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("boolean_kind_id", T::id())
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
        ],
        "Explicit physical context for canonicalization; no dimensional guessing.",
    );
}
