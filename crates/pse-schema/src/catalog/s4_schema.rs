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
//! declared here as relations; they are the registry's [`crate::model::FieldContract`] and
//! [`crate::model::EXTENSION_TYPES`], and `reference.schema_logical_types` is generated
//! from them at assembly (see [`crate::builder`]).

use crate::builder::RegistryBuilder;
use crate::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass, Stability};

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
/// Assemblies that need only the relation catalog can reuse this exact declaration.
pub fn declare_relations(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        reference(
            "schema_relations",
            "One row per declared relation: the registry describing itself (blueprint §4.1).",
        )
        .pk(&["relation_id"])
        .columns(vec![
            FieldContract::key(
                "relation_id",
                FieldContract::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"relation:<ns>.<name>@<v>\")` (ADR-0050).",
            ),
            FieldContract::label(
                "namespace",
                FieldContract::enumeration("Namespace"),
                "The namespace, which is also the catalog schema name.",
            ),
            FieldContract::label(
                "name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The relation name inside its namespace.",
            ),
            FieldContract::payload(
                "version",
                FieldContract::nonnegative(i64::from(u32::MAX)),
                "The schema version.",
            ),
            FieldContract::label(
                "authority",
                FieldContract::enumeration("Authority"),
                "Who may write it.",
            ),
            FieldContract::label(
                "snapshot_class",
                FieldContract::enumeration("SnapshotClass"),
                "Explicit snapshot membership (blueprint §5.3 step 7).",
            ),
            FieldContract::payload(
                "primary_key",
                FieldContract::list(FieldContract::native(arrow_schema::DataType::Utf8)),
                "The primary key column names, in key order.",
            ),
            FieldContract::label(
                "derivation_granularity",
                FieldContract::enumeration("DerivationGranularity"),
                "Required when `authority = derived` (blueprint §14.2 rule 4).",
            )
            .optional(),
            FieldContract::label(
                "stability",
                FieldContract::enumeration("Stability"),
                "How much a consumer may rely on the shape.",
            ),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "What the relation means.",
            ),
            FieldContract::payload(
                "checks",
                FieldContract::list(FieldContract::structure(vec![
                    FieldContract::native(arrow_schema::DataType::Utf8).with_name("name"),
                    FieldContract::native(arrow_schema::DataType::Utf8).with_name("sql"),
                ])),
                "Native DataFusion SQL predicates; every predicate must evaluate to true.",
            ),
            FieldContract::payload(
                "delta_properties",
                FieldContract::list(FieldContract::structure(vec![
                    FieldContract::native(arrow_schema::DataType::Utf8).with_name("name"),
                    FieldContract::native(arrow_schema::DataType::Utf8).with_name("value"),
                ])),
                "Declared native Delta policies in name order; part of the exact table contract.",
            ),
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
            FieldContract::key("relation_id", FieldContract::id(), "The owning relation.")
                .with_fk("reference.schema_relations", "relation_id"),
            FieldContract::key(
                "ordinal",
                FieldContract::nonnegative(i64::from(u16::MAX)),
                "The column's position, which is its ordinal.",
            ),
            FieldContract::label("name", FieldContract::native(arrow_schema::DataType::Utf8), "The column name."),
            FieldContract::reference(
                "logical_type_id",
                FieldContract::id(),
                "The declared logical type.",
            )
            .with_fk("reference.schema_logical_types", "logical_type_id"),
            FieldContract::payload(
                "nullable",
                FieldContract::native(arrow_schema::DataType::Boolean),
                "Whether the column admits nulls.",
            ),
            FieldContract::reference(
                "quantity_type_id",
                FieldContract::id(),
                "One quantity contract for the whole column.",
            )
            .optional(),
            FieldContract::reference(
                "fk_relation_id",
                FieldContract::id(),
                "The referenced relation, when the column is a reference.",
            )
            .optional(),
            FieldContract::label(
                "fk_column",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The referenced column, when the column is a reference.",
            )
            .optional(),
            FieldContract::label(
                "role",
                FieldContract::enumeration("ColumnRole"),
                "What the column is for.",
            ),
            FieldContract::label("doc", FieldContract::native(arrow_schema::DataType::Utf8), "What the column means."),
            FieldContract::payload("native_field", FieldContract::native(arrow_schema::DataType::Utf8), "Complete canonical Arrow field declaration, including every nested domain facet and metadata entry."),
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
            FieldContract::key(
                "logical_type_id",
                FieldContract::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"logical_type:<name>\")` (ADR-0050).",
            ),
            FieldContract::label(
                "name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The registry name, for example `semantic_id` or `enum:PhaseType`.",
            ),
            FieldContract::label(
                "arrow_storage",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The canonical Arrow storage rendering. Never a `Debug` rendering (blueprint §5.3).",
            ),
            FieldContract::label(
                "extension_name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The `ARROW:extension:name`, when the type is an extension use.",
            )
            .optional(),
            FieldContract::payload(
                "metadata_schema",
                FieldContract::native(arrow_schema::DataType::Utf8),
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
            "schema_enum_types",
            "One row per declared enumeration identity.",
        )
        .pk(&["enum_id"])
        .columns(vec![
            FieldContract::key(
                "enum_id",
                FieldContract::id(),
                "The registry enumeration identity.",
            ),
            FieldContract::label(
                "name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The declared enumeration name.",
            ),
            FieldContract::label(
                "idaes_source",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The optional parity enumeration source.",
            )
            .optional(),
        ]),
    );
    builder.declare_relation(
        reference(
            "schema_enums",
            "One row per enumeration member; the member ordinal is presentation, never identity.",
        )
        .pk(&["enum_id", "member_ordinal"])
        .columns(vec![
            FieldContract::key(
                "enum_id",
                FieldContract::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"enum:<Name>\")` (ADR-0050).",
            )
            .with_fk("reference.schema_enum_types", "enum_id"),
            FieldContract::key(
                "member_ordinal",
                FieldContract::nonnegative(i64::from(u16::MAX)),
                "The member's declaration position.",
            ),
            FieldContract::label(
                "member",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The stored dictionary value.",
            ),
            FieldContract::label(
                "idaes_name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The IDAES spelling, for the §6.14 parity enumerations.",
            )
            .optional(),
            FieldContract::payload(
                "deprecated",
                FieldContract::native(arrow_schema::DataType::Boolean),
                "Declared but no longer selectable; kept so old artifacts still decode.",
            ),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "What the member means.",
            ),
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
            FieldContract::key(
                "invariant_id",
                FieldContract::id(),
                "`named_id(REGISTRY_PACKAGE_ID, \"invariant:<relation>:<name>\")` (ADR-0050).",
            ),
            FieldContract::reference(
                "relation_id",
                FieldContract::id(),
                "The relation the invariant constrains.",
            ),
            FieldContract::label(
                "kind",
                FieldContract::enumeration("InvariantKind"),
                "What kind of statement it makes.",
            ),
            FieldContract::payload(
                "query",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Native SQL returning offending keys.",
            ),
            FieldContract::payload(
                "inputs",
                FieldContract::list(FieldContract::native(arrow_schema::DataType::Utf8)),
                "Explicitly selected semantic input relations.",
            ),
            FieldContract::payload(
                "key_columns",
                FieldContract::list(FieldContract::native(arrow_schema::DataType::Utf8)),
                "Ordered offending key columns.",
            ),
            FieldContract::label(
                "severity",
                FieldContract::enumeration("Severity"),
                "Whether a violation stops a commit.",
            ),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "What the invariant means.",
            ),
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
            FieldContract::key("relation_id", FieldContract::id(), "The migrated relation.")
                .with_fk("reference.schema_relations", "relation_id"),
            FieldContract::key(
                "from_version",
                FieldContract::nonnegative(i64::from(u32::MAX)),
                "The version the migration reads.",
            ),
            FieldContract::key(
                "to_version",
                FieldContract::nonnegative(i64::from(u32::MAX)),
                "The version the migration writes.",
            ),
            FieldContract::payload(
                "plan_spec",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The migration steps, one per line, in application order.",
            ),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Why the schema changed.",
            ),
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
            FieldContract::key(
                "document_name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Document declaration name.",
            ),
            FieldContract::label(
                "kind",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "DocumentKind spelling.",
            ),
            FieldContract::label(
                "path_glob",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Package-relative file selection.",
            ),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Document meaning.",
            ),
        ]),
    );
    declare_document_sections(builder);
}

fn declare_document_sections(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        {
            let mut declaration = reference(
                "schema_document_sections",
                "Exact identity and row projections (ADR-0059, ADR-0063).",
            );
            declaration.key.version = 2;
            declaration
        }
        .pk(&["document_name", "ordinal"])
        .columns(vec![
            FieldContract::key(
                "document_name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Owning document declaration.",
            )
            .with_fk("reference.schema_documents", "document_name"),
            FieldContract::key(
                "ordinal",
                FieldContract::nonnegative(i64::from(u32::MAX)),
                "Section declaration ordinal.",
            ),
            FieldContract::label(
                "key",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Authoring section key.",
            ),
            FieldContract::reference("relation_id", FieldContract::id(), "Exact target relation.")
                .with_fk("reference.schema_relations", "relation_id"),
            FieldContract::payload(
                "repeated",
                FieldContract::native(arrow_schema::DataType::Boolean),
                "Sequence rather than singleton.",
            ),
            FieldContract::label(
                "identity_column",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Explicit identity alias.",
            )
            .optional(),
            FieldContract::label(
                "entity_kind",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Registered EntityKind member.",
            )
            .optional(),
            FieldContract::label(
                "name_column",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Entity local name.",
            )
            .optional(),
            FieldContract::label(
                "naming_scope_column",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Owning entity foreign key.",
            )
            .optional(),
            FieldContract::label(
                "expression_owner_column",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Explicit source expression owner.",
            )
            .optional(),
            FieldContract::payload(
                "expression_owner_kind",
                FieldContract::enumeration("ExpressionOwnerKind"),
                "Explicit expression ownership alternative.",
            )
            .optional(),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Section meaning.",
            ),
            FieldContract::payload(
                "expression_fields",
                FieldContract::list(FieldContract::structure(vec![
                    FieldContract::native(arrow_schema::DataType::Utf8)
                        .with_name("path")
                        .with_nullable(false),
                    FieldContract::enumeration("ExpressionSyntax")
                        .with_name("syntax")
                        .with_nullable(false),
                ])),
                "Complete exact DSL field grammar mapping.",
            ),
        ]),
    );
}
