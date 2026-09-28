// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One Rust type per source-owned vocabulary (ADR-0115 Outcome 3, ADR-0117).

use std::any::TypeId;

use crate::generated::enums as generated;

/// The generated name and the source path are one type: the identity coercion compiles
/// only then, and the `TypeId`s agree when it runs.
macro_rules! one_type {
    ($($name:ident = $source:path),+ $(,)?) => {$(
        let same: fn($source) -> generated::$name = |value| value;
        let _ = same;
        assert_eq!(
            TypeId::of::<generated::$name>(),
            TypeId::of::<$source>(),
            stringify!($name)
        );
    )+};
}

#[test]
fn source_owned_vocabularies_have_one_rust_type() {
    one_type!(
        Namespace = pse_vocabulary::Namespace,
        Authority = pse_vocabulary::Authority,
        SnapshotClass = pse_vocabulary::SnapshotClass,
        DerivationGranularity = pse_vocabulary::DerivationGranularity,
        Stability = pse_vocabulary::Stability,
        ColumnRole = pse_vocabulary::ColumnRole,
        InvariantKind = pse_vocabulary::InvariantKind,
        Severity = pse_vocabulary::Severity,
        Determinism = pse_vocabulary::Determinism,
        OperationEffect = pse_vocabulary::OperationEffect,
        DiagnosticCode = pse_diagnostics::DiagnosticCode,
        FailureClass = pse_diagnostics::FailureClass,
        Opcode = pse_quantity::Opcode,
        ScaleKind = pse_quantity::ScaleKind,
        QuantityAdditionKind = pse_quantity::QuantityAdditionKind,
        QuantityKindCategory = pse_quantity::QuantityKindCategory,
        BasisKind = pse_quantity::BasisKind,
        CompositionBasis = pse_quantity::CompositionBasis,
        RateBasis = pse_quantity::RateBasis,
        ReferenceStateKind = pse_quantity::ReferenceStateKind,
        ConversionKind = pse_quantity::ConversionKind,
        BasisRule = pse_quantity::BasisRule,
        ReferenceRule = pse_quantity::ReferenceRule,
        QuantityScaleRule = pse_quantity::QuantityScaleRule,
        QuantityShapeRule = pse_quantity::QuantityShapeRule,
        SubjectRule = pse_quantity::SubjectRule,
        WeightNormalization = pse_quantity::WeightNormalization,
        ReductionKind = pse_quantity::ReductionKind,
    );
}
