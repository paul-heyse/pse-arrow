// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry as relations: `reference.schema_*` exactly as blueprint §4.1 declares it.
//!
//! The registry is stored as relations so the platform can query its own schema, diff
//! versions and generate migrations. That is also why these six relations are declared
//! here like any other: a registry that described everything except itself would need a
//! second, undeclared description of what it is — and nothing would keep the two in step.
//!
//! The logical-type catalog of §4.5 and the eleven extension types of §4.4 are not
//! declared here as relations; they are the registry's [`crate::model::LogicalType`] and
//! [`crate::model::EXTENSION_TYPES`], and `reference.schema_logical_types` is generated
//! from them at assembly (see [`crate::builder`]).

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, ColumnSpec, LogicalType, Namespace, RelationDecl, SnapshotClass, Stability,
};

/// Declares the six `reference.schema_*` relations.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_relations(builder);
    declare_columns(builder);
    declare_logical_types(builder);
    declare_enums(builder);
    declare_invariants(builder);
    declare_migrations(builder);
    declare_document_contracts(builder);
}

/// A `reference` relation at stability `stable`, in the `model` snapshot class.
///
/// The registry's contracts are structural reference facts, so they are model members
/// (blueprint §5.3 step 7): a snapshot that did not name the schema it was written under
/// could be read back under a different one without anything noticing.
fn reference(name: &'static str, doc: &'static str) -> RelationDecl {
    RelationDecl::new(
        Namespace::Reference,
        name,
        1,
        Authority::Reference,
        SnapshotClass::Model,
        doc,
    )
    .stability(Stability::Stable)
}

/// `reference.schema_relations @1` (blueprint §4.1).
fn declare_relations(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_relations",
            "One row per declared relation: the registry describing itself (blueprint §4.1).",
        )
        .pk(&["relation_id"])
        .columns(vec![
            ColumnSpec::key(
                "relation_id",
                LogicalType::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"relation:<ns>.<name>@<v>\")` (ADR-0050).",
            ),
            ColumnSpec::label(
                "namespace",
                LogicalType::enumeration("Namespace"),
                "The namespace, which is also the catalog schema name.",
            ),
            ColumnSpec::label(
                "name",
                LogicalType::Text,
                "The relation name inside its namespace.",
            ),
            ColumnSpec::payload("version", LogicalType::U32, "The schema version."),
            ColumnSpec::label(
                "authority",
                LogicalType::enumeration("Authority"),
                "Who may write it.",
            ),
            ColumnSpec::label(
                "snapshot_class",
                LogicalType::enumeration("SnapshotClass"),
                "Explicit snapshot membership (blueprint §5.3 step 7).",
            ),
            ColumnSpec::payload(
                "primary_key",
                LogicalType::list(LogicalType::Text),
                "The primary key column names, in key order.",
            ),
            ColumnSpec::label(
                "derivation_granularity",
                LogicalType::enumeration("DerivationGranularity"),
                "Required when `authority = derived` (blueprint §14.2 rule 4).",
            )
            .optional(),
            ColumnSpec::label(
                "stability",
                LogicalType::enumeration("Stability"),
                "How much a consumer may rely on the shape.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "What the relation means."),
        ]),
    );
}

/// `reference.schema_columns @1` (blueprint §4.1).
fn declare_columns(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_columns",
            "One row per declared column, in declaration order (blueprint §4.1).",
        )
        .pk(&["relation_id", "ordinal"])
        .columns(vec![
            ColumnSpec::key("relation_id", LogicalType::id(), "The owning relation.")
                .with_fk("reference.schema_relations", "relation_id"),
            ColumnSpec::key(
                "ordinal",
                LogicalType::U16,
                "The column's position, which is its ordinal.",
            ),
            ColumnSpec::label("name", LogicalType::Text, "The column name."),
            ColumnSpec::reference(
                "logical_type_id",
                LogicalType::id(),
                "The declared logical type.",
            )
            .with_fk("reference.schema_logical_types", "logical_type_id"),
            ColumnSpec::payload(
                "nullable",
                LogicalType::Bool,
                "Whether the column admits nulls.",
            ),
            ColumnSpec::reference(
                "quantity_type_id",
                LogicalType::id(),
                "One quantity contract for the whole column.",
            )
            .optional(),
            ColumnSpec::payload(
                "per_row_quantity",
                LogicalType::Bool,
                "True when a sibling column named `<name>_quantity_type_id` carries the contract.",
            ),
            ColumnSpec::reference(
                "fk_relation_id",
                LogicalType::id(),
                "The referenced relation, when the column is a reference.",
            )
            .optional(),
            ColumnSpec::label(
                "fk_column",
                LogicalType::Text,
                "The referenced column, when the column is a reference.",
            )
            .optional(),
            ColumnSpec::label(
                "role",
                LogicalType::enumeration("ColumnRole"),
                "What the column is for.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "What the column means."),
        ]),
    );
}

/// `reference.schema_logical_types @1` (blueprint §4.1, §4.5).
fn declare_logical_types(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_logical_types",
            "The logical-type catalog: the §4.5 scalars, their composites and the §4.4 extension uses.",
        )
        .pk(&["logical_type_id"])
        .columns(vec![
            ColumnSpec::key(
                "logical_type_id",
                LogicalType::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"logical_type:<name>\")` (ADR-0050).",
            ),
            ColumnSpec::label(
                "name",
                LogicalType::Text,
                "The registry name, for example `semantic_id` or `enum:PhaseType`.",
            ),
            ColumnSpec::label(
                "arrow_storage",
                LogicalType::Text,
                "The canonical Arrow storage rendering. Never a `Debug` rendering (blueprint §5.3).",
            ),
            ColumnSpec::label(
                "extension_name",
                LogicalType::Text,
                "The `ARROW:extension:name`, when the type is an extension use.",
            )
            .optional(),
            ColumnSpec::payload(
                "metadata_schema",
                LogicalType::Text,
                "The JSON Schema of `ARROW:extension:metadata`, when the type is an extension use.",
            )
            .optional(),
        ]),
    );
}

/// `reference.schema_enums @1` (blueprint §4.1, §6.14).
fn declare_enums(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_enums",
            "One row per enumeration member; the member ordinal is presentation, never identity.",
        )
        .pk(&["enum_id", "member_ordinal"])
        .columns(vec![
            ColumnSpec::key(
                "enum_id",
                LogicalType::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"enum:<Name>\")` (ADR-0050).",
            ),
            ColumnSpec::key(
                "member_ordinal",
                LogicalType::U16,
                "The member's declaration position.",
            ),
            ColumnSpec::label("member", LogicalType::Text, "The stored dictionary value."),
            ColumnSpec::label(
                "idaes_name",
                LogicalType::Text,
                "The IDAES spelling, for the §6.14 parity enumerations.",
            )
            .optional(),
            ColumnSpec::payload(
                "deprecated",
                LogicalType::Bool,
                "Declared but no longer selectable; kept so old artifacts still decode.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "What the member means."),
        ]),
    );
}

/// `reference.schema_invariants @1` (blueprint §4.1, §22.2).
fn declare_invariants(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_invariants",
            "The commit contract: every invariant P2 must find satisfied (blueprint §22.2).",
        )
        .pk(&["invariant_id"])
        .columns(vec![
            ColumnSpec::key(
                "invariant_id",
                LogicalType::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"invariant:<relation>:<name>\")` (ADR-0050).",
            ),
            ColumnSpec::reference(
                "relation_id",
                LogicalType::id(),
                "The relation the invariant constrains.",
            ),
            ColumnSpec::label(
                "kind",
                LogicalType::enumeration("InvariantKind"),
                "What kind of statement it makes.",
            ),
            ColumnSpec::reference(
                "rule_id",
                LogicalType::id(),
                "The typed rule plan that returns the violating keys (blueprint §6.11).",
            ),
            ColumnSpec::label(
                "severity",
                LogicalType::enumeration("Severity"),
                "Whether a violation stops a commit.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "What the invariant means."),
        ]),
    );
}

/// `reference.schema_migrations @1` (blueprint §4.1, §20.5).
fn declare_migrations(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_migrations",
            "How to read an artifact written under an earlier version of a relation.",
        )
        .pk(&["relation_id", "from_version", "to_version"])
        .columns(vec![
            ColumnSpec::key("relation_id", LogicalType::id(), "The migrated relation.")
                .with_fk("reference.schema_relations", "relation_id"),
            ColumnSpec::key(
                "from_version",
                LogicalType::U32,
                "The version the migration reads.",
            ),
            ColumnSpec::key(
                "to_version",
                LogicalType::U32,
                "The version the migration writes.",
            ),
            ColumnSpec::payload(
                "plan_spec",
                LogicalType::Text,
                "The migration steps, one per line, in application order.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "Why the schema changed."),
        ]),
    );
}

/// Document selection and its exact authoring semantics (ADR-0059).
fn declare_document_contracts(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_documents",
            "Declared authoring document surfaces (ADR-0059).",
        )
        .pk(&["document_name"])
        .columns(vec![
            ColumnSpec::key(
                "document_name",
                LogicalType::Text,
                "Document declaration name.",
            ),
            ColumnSpec::label("kind", LogicalType::Text, "DocumentKind spelling."),
            ColumnSpec::label(
                "path_glob",
                LogicalType::Text,
                "Package-relative file selection.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "Document meaning."),
        ]),
    );
    builder.declare_relation(
        reference(
            "schema_document_sections",
            "Exact identity and row projections (ADR-0059).",
        )
        .pk(&["document_name", "ordinal"])
        .columns(vec![
            ColumnSpec::key(
                "document_name",
                LogicalType::Text,
                "Owning document declaration.",
            )
            .with_fk("reference.schema_documents", "document_name"),
            ColumnSpec::key("ordinal", LogicalType::U32, "Section declaration ordinal."),
            ColumnSpec::label("key", LogicalType::Text, "Authoring section key."),
            ColumnSpec::reference("relation_id", LogicalType::id(), "Exact target relation.")
                .with_fk("reference.schema_relations", "relation_id"),
            ColumnSpec::payload(
                "repeated",
                LogicalType::Bool,
                "Sequence rather than singleton.",
            ),
            ColumnSpec::label(
                "identity_column",
                LogicalType::Text,
                "Explicit identity alias.",
            )
            .optional(),
            ColumnSpec::label(
                "entity_kind",
                LogicalType::Text,
                "Registered EntityKind member.",
            )
            .optional(),
            ColumnSpec::label("name_column", LogicalType::Text, "Entity local name.").optional(),
            ColumnSpec::label(
                "naming_scope_column",
                LogicalType::Text,
                "Owning entity foreign key.",
            )
            .optional(),
            ColumnSpec::label(
                "expression_owner_column",
                LogicalType::Text,
                "Explicit source expression owner.",
            )
            .optional(),
            ColumnSpec::label("doc", LogicalType::Text, "Section meaning."),
            ColumnSpec::payload(
                "expression_fields",
                LogicalType::list(LogicalType::Struct(vec![
                    ("path", LogicalType::Text, false),
                    (
                        "syntax",
                        LogicalType::enumeration("ExpressionSyntax"),
                        false,
                    ),
                ])),
                "Complete exact DSL field grammar mapping.",
            ),
        ]),
    );
}
