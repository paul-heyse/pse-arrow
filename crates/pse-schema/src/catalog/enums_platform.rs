// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The platform's own vocabularies, declared as `pse.enum` dictionaries
//! (blueprint §4.1, §6.11, §23.2).
//!
//! The registry describes itself (§4.1), so the enumerations its own relations are written
//! in have to be declarations like any other — `schema_relations.namespace` is a
//! `pse.enum(Namespace)` column, and there is nowhere else for `Namespace` to come from.
//! Each member list is generated from the Rust enum's `ALL` constant and `as_str`, so the
//! declaration and the type the compiler enforces are one statement, not two.
//!
//! The enumerations preserved from IDAES by name (§6.14) are packet A-2's
//! `s6_14_idaes_enums`, not this module.

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, ColumnRole, ConflictPolicy, DependencyMode, DerivationGranularity, Determinism,
    EnumDecl, EnumMember, InvariantKind, Namespace, NegationPolicy, Severity, SnapshotClass,
    Stability,
};

/// Declares every platform vocabulary.
pub fn declare(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "OperationEffect",
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
        .declare_enum(EnumDecl::platform(
            "Namespace",
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
        .declare_enum(EnumDecl::platform(
            "Authority",
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
        .declare_enum(EnumDecl::platform(
            "SnapshotClass",
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
        .declare_enum(EnumDecl::platform(
            "DerivationGranularity",
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
        .declare_enum(EnumDecl::platform(
            "Stability",
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
        .declare_enum(EnumDecl::platform(
            "ColumnRole",
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
        .declare_enum(EnumDecl::platform(
            "InvariantKind",
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
        .declare_enum(EnumDecl::platform(
            "Severity",
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
    declare_plan_vocabularies(builder);
    declare_policy_vocabularies(builder);
    declare_null_vocabularies(builder);
}

/// Native rule dependency roles.
fn declare_plan_vocabularies(builder: &mut RegistryBuilder) {
    builder.declare_enum(EnumDecl::platform(
        "DependencyMode",
        DependencyMode::ALL
            .iter()
            .map(|value| member(value.as_str(), "Native query input or output scope."))
            .collect(),
    ));
}

/// `NegationPolicy`, `ConflictPolicy` and `Determinism` (blueprint §14.1, §14.2).
fn declare_policy_vocabularies(builder: &mut RegistryBuilder) {
    builder
        .declare_enum(EnumDecl::platform(
            "NegationPolicy",
            NegationPolicy::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            NegationPolicy::None => "The rule contains no negation.",
                            NegationPolicy::Stratified => "Only lower strata are negated.",
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::platform(
            "ConflictPolicy",
            ConflictPolicy::ALL
                .iter()
                .map(|value| {
                    member(
                        value.as_str(),
                        match value {
                            ConflictPolicy::Reject => "A conflict is an error.",
                            ConflictPolicy::Undecided => "The key goes to `inferred.undecided`.",
                        },
                    )
                })
                .collect(),
        ))
        .declare_enum(EnumDecl::platform(
            "Determinism",
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
                member("package", "A package."),
                member("dimension", "A base or derived dimension."),
                member("quantity_type", "A quantity type."),
                member("unit", "A unit of measure."),
                member("unit_set", "A coherent selection of base units."),
                member("quantity_kind", "A physical quantity kind."),
                member("constant", "A named physical constant."),
                member("symbol_declaration", "A template symbol declaration."),
                member("equation_declaration", "A template equation declaration."),
                member("contribution_declaration", "A template contribution declaration."),
                member("element", "A chemical element."),
                member("domain", "An index set or continuous domain."),
                member("species", "A chemical species."),
                member("phase", "A phase."),
                member("material_system", "A material system."),
                member("reaction", "A reaction."),
                member("property_package", "A property package."),
                member("reaction_package", "A reaction package."),
                member("method", "A property or reaction method."),
                member("kernel", "A kernel."),
                member("template", "A template."),
                member("instance", "A template instance."),
                member("port", "An actual typed interface bound to a state instance or collection."),
                member("flowsheet", "A flowsheet."),
                member("scope", "A scope."),
                member("connection", "A connection between ports."),
                member("case", "A case."),
                member("dataset", "A dataset."),
                member("observation", "An observation."),
                member("solver_profile", "A solver profile."),
                member("discretization_policy", "A discretization policy."),
            ],
        ));
}

/// The §23.2 failure classes, as a closed dictionary.
///
/// The member spelling is the diagnostic code, so a `algorithm_specs.diagnostics` row and a
/// `#[diagnostic(code(...))]` are the same string rather than two spellings of one idea.
pub(super) fn declare_failure_classes(builder: &mut RegistryBuilder) {
    builder.declare_enum(EnumDecl::platform(
        "FailureClass",
        vec![
            member("authoring.parse", "A syntax error or an unknown key."),
            member("authoring.reference", "An unknown path or a derived write."),
            member(
                "validation.invariant",
                "A declared invariant does not hold.",
            ),
            member("compile.feature", "An incompatible feature combination."),
            member("compile.property", "An unsupported or ambiguous property."),
            member("compile.law", "An unsupported balance binding."),
            member(
                "compile.math",
                "Incompatible physical contracts or a cyclic expression.",
            ),
            member(
                "kernel.unbound_parameter",
                "A selected kernel lacks its actual executable or parameter binding.",
            ),
            member(
                "compile.discretization",
                "A mixed derivative or a missing policy.",
            ),
            member(
                "capability.backend",
                "An unsupported opcode or missing derivative.",
            ),
            member(
                "plan.initialization",
                "A structural singularity or a failed postcheck.",
            ),
            member("solve.infeasible", "The problem is infeasible."),
            member(
                "solve.locally_infeasible",
                "The solver converged to local infeasibility.",
            ),
            member("solve.unbounded", "The objective is unbounded."),
            member("solve.limit", "An iteration, time or evaluation limit."),
            member("solve.evaluation_error", "A function evaluation failed."),
            member(
                "solve.solver_error",
                "The solver reported an internal failure.",
            ),
            member("runtime.cancelled", "A cancellation token fired."),
            member("runtime.timeout", "A wall-clock limit fired."),
            member(
                "runtime.resource_limit",
                "A reservation or size limit was exceeded.",
            ),
            member(
                "runtime.infrastructure",
                "Store input/output or an integrity failure.",
            ),
            member(
                "config.invalid",
                "An invalid engine or platform configuration key.",
            ),
            member("internal.invariant", "A pass postcondition failed."),
            member(
                "user.model",
                "An authored assertion or user equation failed.",
            ),
        ],
    ));
}
