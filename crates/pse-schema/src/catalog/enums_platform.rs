// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The platform's own vocabularies, declared as `pse.enum` dictionaries
//! (blueprint §4.1, §6.11, §23.2).
//!
//! The registry describes itself (§4.1), so the enumerations its own relations are written
//! in have to be declarations like any other — `schema_relations.namespace` is a
//! `pse.enum(Namespace)` column, and there is nowhere else for `Namespace` to come from.
//! Each member list is generated from the Rust enum's `ALL` constant and `as_str`, so the
//! declaration and the type the compiler enforces are one statement, not two, and each such
//! declaration names its source type, which the generator re-exports (ADR-0115 Outcome 3).
//!
//! The enumerations preserved from IDAES by name (§6.14) are packet A-2's
//! `s6_14_idaes_enums`, not this module.

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, ColumnRole, DerivationGranularity, Determinism, EnumDecl, EnumMember, InvariantKind,
    Namespace, Severity, SnapshotClass, Stability,
};

/// Declares every platform vocabulary.
pub fn declare(builder: &mut RegistryBuilder) {
    super::declarations::sourced_enumeration(
        builder,
        "pse_vocabulary::OperationEffect",
        crate::model::provider::OperationEffect::ALL
            .map(crate::model::provider::OperationEffect::as_str),
    );
    super::declarations::enumeration(builder, "BoundKind", ["finite", "unbounded"]);
    declare_schema_vocabularies(builder);
    declare_rule_vocabularies(builder);
    declare_identity_vocabularies(builder);
    declare_failure_classes(builder);
}

/// One member from a name and a doc line.
fn member(name: &'static str, doc: &'static str) -> EnumMember {
    EnumMember::new(name, doc)
}

/// The vocabularies `reference.schema_*` is written in (blueprint §4.1).
fn declare_schema_vocabularies(builder: &mut RegistryBuilder) {
    declare_namespace_and_authority(builder);
    declare_class_and_stability(builder);
    declare_column_and_invariant_vocabularies(builder);
}

/// `Namespace` and `Authority` (blueprint §4.1).
fn declare_namespace_and_authority(builder: &mut RegistryBuilder) {
    builder
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::Namespace",
            Namespace::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            Namespace::Reference => "Shipped contracts and reference data.",
                            Namespace::Authored => "Facts a human or an agent authored.",
                            Namespace::Normalized => "P3's canonical rewriting of authored facts.",
                            Namespace::Inferred => "Facts the rule engine inferred.",
                            Namespace::Compiled => "Compiler output.",
                            Namespace::Runtime => "Execution records and results.",
                            Namespace::Provenance => "Derivations, pass records and assertions.",
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::Authority",
            Authority::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            Authority::Authored => "Written only through a change set.",
                            Authority::Reference => "Shipped by a reference package.",
                            Authority::Derived => "Produced by a pass.",
                        },
                    )
                })
                .collect(),
        ));
}

/// `SnapshotClass`, `DerivationGranularity` and `Stability` (blueprint §4.1, §5.3).
fn declare_class_and_stability(builder: &mut RegistryBuilder) {
    builder
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::SnapshotClass",
            SnapshotClass::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            SnapshotClass::Model => "A structural authored or reference contract.",
                            SnapshotClass::Case => "A case, specification or observation relation.",
                            SnapshotClass::Derived => "Compiler or runtime output.",
                            SnapshotClass::Sidecar => {
                                "A catalog, log, ref or record excluded from its own membership."
                            }
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::DerivationGranularity",
            DerivationGranularity::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            DerivationGranularity::Row => "One derivation row per head row.",
                            DerivationGranularity::Rule => {
                                "The rule plus the identity formula, reconstructed on demand."
                            }
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::Stability",
            Stability::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            Stability::Stable => "Public and versioned.",
                            Stability::Evolving => "Public but still moving.",
                            Stability::Internal => "No external consumer may depend on it.",
                        },
                    )
                })
                .collect(),
        ));
}

/// `ColumnRole`, `InvariantKind`, `Severity` and `MigrationOp` (blueprint §4.1).
fn declare_column_and_invariant_vocabularies(builder: &mut RegistryBuilder) {
    builder
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::ColumnRole",
            ColumnRole::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            ColumnRole::Key => "Part of the primary key.",
                            ColumnRole::Reference => "A reference to another relation's key.",
                            ColumnRole::Measure => "A numerical value under a quantity contract.",
                            ColumnRole::Label => "A human-facing name. Never identity.",
                            ColumnRole::Payload => "Structured content.",
                            ColumnRole::Provenance => "Evidence: a derivation, span or pass.",
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::InvariantKind",
            InvariantKind::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            InvariantKind::Unique => "The named columns are unique.",
                            InvariantKind::ForeignKey => "Every value resolves in its target.",
                            InvariantKind::Check => "A row-local predicate holds.",
                            InvariantKind::Cardinality => "A group has a declared size.",
                            InvariantKind::Domain => "A value lies in a declared domain.",
                            InvariantKind::Closure => "A set is closed under a relation.",
                            InvariantKind::Acyclic => "A declared edge relation has no cycle.",
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::sourced(
            "pse_vocabulary::Severity",
            Severity::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            Severity::Error => "The change set is rejected.",
                            Severity::Warning => "Reported; the commit proceeds.",
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::platform(
            "MigrationOp",
            vec![
                member("add_column", "Add a column with a declared default."),
                member("drop_column", "Drop a column."),
                member("rename_column", "Rename a column, keeping its values."),
                member("change_nullable", "Change a column's nullability."),
            ],
        ));
}

/// The vocabularies the rule algebra is written in (blueprint §6.11, §14.2).
fn declare_rule_vocabularies(builder: &mut RegistryBuilder) {
    declare_policy_vocabularies(builder);
    declare_null_vocabularies(builder);
}

/// Algorithm determinism (blueprint §14.1).
fn declare_policy_vocabularies(builder: &mut RegistryBuilder) {
    builder.declare_enum(EnumDecl::sourced(
        "pse_vocabulary::Determinism",
        Determinism::ALL
            .iter()
            .map(|value| {
                member(
                    value.as_str(),
                    match value {
                        Determinism::Deterministic => "The same inputs give the same outputs.",
                        Determinism::DeterministicFixedPoint => {
                            "Deterministic as a least fixed point."
                        }
                        Determinism::DeterministicPerBackend => {
                            "Deterministic once a backend is chosen."
                        }
                    },
                )
            })
            .collect(),
    ));
}

/// The null, empty and truth-value vocabularies (blueprint §6.11, §7.6, §14.2).
fn declare_null_vocabularies(builder: &mut RegistryBuilder) {
    builder.declare_enum(EnumDecl::platform(
        "TruthValue",
        vec![
            member("true", "Decided true; the row goes to the head relation."),
            member("false", "Decided false."),
            member("unknown", "The predicate could not decide."),
            member(
                "conflict",
                "Two rules asserted incompatible values for one key.",
            ),
        ],
    ));
}

/// The vocabularies §6.1 and §22.2 are written in.
fn declare_identity_vocabularies(builder: &mut RegistryBuilder) {
    builder
        .declare_enum(EnumDecl::platform(
            "PackageKind",
            vec![
                member(
                    "reference",
                    "Reference data under the named identity policy: units, elements, constants, property kinds.",
                ),
                member(
                    "library",
                    "A shipped library of templates, laws and methods.",
                ),
                member("model", "An authored model: materials, flowsheets, connections."),
                member("case", "Cases, observations and case sets."),
            ],
        ))
        .declare_enum(EnumDecl::platform(
            "IdPolicy",
            vec![
                member(
                    "explicit",
                    "The authoring tool assigns a UUIDv7 when the entity is first written; a rename changes an attribute.",
                ),
                member(
                    "named",
                    "`blake3_128(\"pse:named:v1\" ‖ package_id ‖ qualified_name)`; a rename is a new entity.",
                ),
            ],
        ))
        .declare_enum(EnumDecl::platform(
            "EntityKind",
            vec![
                member("package", "A package manifest."),
                member("unit", "A unit of measure."),
                member("unit_set", "A coherent selection of base units."),
                member("quantity_kind", "A physical quantity kind."),
                member("constant", "A named physical constant."),
                member("dataset", "An observation dataset."),
            ],
        ));
}

/// Registry projections of the leaf's diagnostic vocabulary.
pub(super) fn declare_failure_classes(builder: &mut RegistryBuilder) {
    builder.declare_enum(EnumDecl::sourced(
        "pse_diagnostics::FailureClass",
        pse_diagnostics::FailureClass::ALL
            .iter()
            .map(|value| member(value.as_str(), value.description()))
            .collect(),
    ));
    builder.declare_enum(EnumDecl::sourced(
        "pse_diagnostics::DiagnosticCode",
        pse_diagnostics::DiagnosticCode::ALL
            .iter()
            .map(|value| member(value.as_str(), value.description()))
            .collect(),
    ));
    builder.declare_enum(EnumDecl::sourced(
        "pse_diagnostics::DiagnosticRule",
        pse_diagnostics::DiagnosticRule::ALL
            .iter()
            .map(|v| member(v.as_str(), v.description()))
            .collect(),
    ));
    builder.declare_enum(EnumDecl::sourced(
        "pse_diagnostics::DiagnosticStage",
        pse_diagnostics::DiagnosticStage::ALL
            .iter()
            .map(|v| member(v.as_str(), v.description()))
            .collect(),
    ));

    builder.declare_enum(EnumDecl::sourced(
        "pse_diagnostics::DiagnosticObservationKind",
        pse_diagnostics::DiagnosticObservationKind::ALL
            .iter()
            .map(|v| member(v.as_str(), v.description()))
            .collect(),
    ));
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "tests report failures by panicking"
    )]

    /// The member spellings of the source type at `path`, for every source-owned
    /// vocabulary the registry may declare.
    macro_rules! source_members {
        ($path:expr; $($krate:ident :: $ty:ident),+ $(,)?) => {
            match $path {
                $(p if p == concat!(stringify!($krate), "::", stringify!($ty)) => Some(
                    $krate::$ty::ALL.iter().map(|value| value.as_str()).collect::<Vec<_>>(),
                ),)+
                _ => None,
            }
        };
    }

    /// Each sourced declaration names a real vocabulary type, and its members are exactly
    /// that type's `ALL` spellings in order: the re-exported type and the registry agree
    /// (ADR-0115 Outcome 3, ADR-0117).
    #[test]
    fn sourced_vocabularies_match_their_source() {
        let registry = crate::registry().unwrap();
        let mut sourced = 0;
        for spec in registry.enums() {
            let Some(path) = spec.source else { continue };
            sourced += 1;
            let expected = source_members!(path;
                pse_vocabulary::Namespace, pse_vocabulary::Authority,
                pse_vocabulary::SnapshotClass, pse_vocabulary::DerivationGranularity,
                pse_vocabulary::Stability, pse_vocabulary::ColumnRole,
                pse_vocabulary::InvariantKind, pse_vocabulary::Severity,
                pse_vocabulary::Determinism, pse_vocabulary::OperationEffect,
                pse_diagnostics::DiagnosticCode, pse_diagnostics::FailureClass,
                pse_diagnostics::DiagnosticRule, pse_diagnostics::DiagnosticStage, pse_diagnostics::DiagnosticObservationKind,
                pse_quantity::Opcode, pse_quantity::ScaleKind, pse_quantity::QuantityAdditionKind,
                pse_quantity::QuantityKindCategory, pse_quantity::BasisKind,
                pse_quantity::CompositionBasis, pse_quantity::RateBasis,
                pse_quantity::ReferenceStateKind, pse_quantity::ConversionKind,
                pse_quantity::BasisRule, pse_quantity::ReferenceRule,
                pse_quantity::QuantityScaleRule, pse_quantity::QuantityShapeRule,
                pse_quantity::SubjectRule, pse_quantity::WeightNormalization,
                pse_quantity::ReductionKind,
            )
            .unwrap_or_else(|| panic!("{} names an unknown source {path}", spec.name));
            assert!(path.ends_with(&format!("::{}", spec.name)), "{path}");
            let declared = spec.members.iter().map(|m| m.name).collect::<Vec<_>>();
            assert_eq!(declared, expected, "{path}");
        }
        assert_eq!(
            sourced, 31,
            "every source-owned vocabulary is declared as sourced"
        );
    }
}
