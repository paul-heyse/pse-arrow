// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Method names become compatible only through the actual complete provision inventory.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "method_compatibility",
        S::Derived,
        &["selection_id", "method_id"],
        vec![
            column("selection_id", T::id()),
            column("method_id", T::id()),
            column("compatible", T::native(arrow_schema::DataType::Boolean)),
            column("reason", T::enumeration("MethodCandidateReason")),
            provenance(),
        ],
        "P4 selection family and supported provision validation; P6 checks each actual requirement signature and scope.",
    );
    assertion(
        builder,
        "inferred.method_compatibility",
        "method_compatibility_assertions",
    );
}
