// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The sole declared native local-validation report contract.
use crate::{
    RegistryBuilder, SchemaError,
    model::{
        Authority, DerivationGranularity, FieldContract as F, Namespace, RelationDecl,
        SnapshotClass,
    },
};
use arrow_schema::{DataType, SchemaRef};
use std::sync::{Arc, LazyLock};

/// Register the same report relation used by local and engine-native observation.
pub fn declare(builder: &mut RegistryBuilder) {
    builder.declare_relation(declaration());
}
fn declaration() -> RelationDecl {
    RelationDecl::new(
        Namespace::Runtime,
        "validation_findings",
        1,
        Authority::Derived,
        SnapshotClass::Sidecar,
        "Bounded native predicate observations. Report completeness is separate from row validity.",
    )
    .pk(&["rule", "row", "path"])
    .granularity(DerivationGranularity::Row)
    .columns(vec![
        F::key("rule", F::native(DataType::Utf8), "Stable semantic rule."),
        F::label(
            "code",
            F::native(DataType::Utf8),
            "Detailed diagnostic vocabulary code.",
        ),
        F::payload("relation_id", F::id(), "Actual input relation, when known.").optional(),
        F::key(
            "row",
            F::native(DataType::UInt64),
            "Actual root row ordinal.",
        ),
        F::payload(
            "key_literals",
            F::native(DataType::Utf8),
            "Lossless native literals by declared key name.",
        )
        .optional(),
        F::key(
            "path",
            F::native(DataType::Utf8),
            "JSON Pointer with actual nested ordinals.",
        ),
        F::payload(
            "field_json",
            F::native(DataType::Utf8),
            "Exact observed Arrow field including metadata.",
        )
        .optional(),
        F::payload(
            "observed_literal",
            F::native(DataType::Utf8),
            "Field-directed bit-exact native literal.",
        )
        .optional(),
    ])
}
/// Exact immutable Arrow report schema projected from the declaration.
/// # Errors
/// The intrinsic declaration cannot be resolved.
pub fn findings_schema() -> Result<SchemaRef, SchemaError> {
    static SCHEMA: LazyLock<Result<SchemaRef, SchemaError>> = LazyLock::new(|| {
        let mut builder = RegistryBuilder::new();
        declare(&mut builder);
        let registry = builder.build()?;
        let spec = registry
            .relation("runtime.validation_findings")
            .ok_or_else(|| {
                crate::checks::invalid("validation report", "missing intrinsic relation")
            })?;
        Ok(Arc::new(crate::arrow::relation_schema(&registry, spec)?))
    });
    SCHEMA.clone()
}
