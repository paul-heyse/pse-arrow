// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Identity and packages (blueprint §6.1, revision 6).
//!
//! Identity is assigned when an entity is created and stored in the document that declares
//! it; `name` and `qualified_name` are attributes. That is the whole point of §6.1: a
//! rename changes an attribute and nothing else, so no authored fact can depend on a name
//! (§5.1). `reference.aliases` is what a named-policy package leaves behind instead of a
//! rename, because under that policy a rename *is* a new entity.

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, DerivationGranularity, ExtensionUse, FieldContract, Namespace, RelationDecl,
    SnapshotClass, Stability,
};

/// Declares the §6.1 identity and package relations.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_packages(builder);
    declare_documents(builder);
    declare_entities(builder);
    declare_aliases(builder);
    declare_package_graph(builder);
}

/// `authored.packages @1` (blueprint §6.1).
fn declare_packages(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "packages",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "One row per package: its version, its identity policy and its exact dependencies.",
        )
        .stability(Stability::Stable)
        .pk(&["package_id"])
        .columns(vec![
            FieldContract::key("package_id", FieldContract::id(), "The package identity."),
            FieldContract::label(
                "name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The package name.",
            ),
            FieldContract::label(
                "version",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The package version (semver).",
            ),
            FieldContract::label(
                "kind",
                FieldContract::enumeration("PackageKind"),
                "What kind of package it is.",
            ),
            FieldContract::label(
                "id_policy",
                FieldContract::enumeration("IdPolicy"),
                "`explicit` or `named` (blueprint §5.1). Recorded on the package, never in the ID.",
            ),
            FieldContract::payload(
                "dependencies",
                FieldContract::list(FieldContract::structure(vec![
                    FieldContract::id()
                        .with_name("package_id")
                        .with_nullable(false),
                    FieldContract::native(arrow_schema::DataType::Utf8)
                        .with_name("version_req")
                        .with_nullable(false),
                ])),
                "The declared dependencies. Phase 0 admits exact version requirements only.",
            ),
            FieldContract::payload(
                "content_hash",
                FieldContract::hash(),
                "The digest of the package's documents.",
            ),
            FieldContract::label(
                "doc",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "What the package is for.",
            ),
        ]),
    );
}

/// `authored.documents @1` (blueprint §6.1).
fn declare_documents(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "documents",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "One row per authoring document; every row's source span points back into one of these.",
        )
        .stability(Stability::Stable)
        .pk(&["document_id"])
        .columns(vec![
            FieldContract::key(
                "document_id",
                FieldContract::id(),
                "`named_id(package_id, path)` (blueprint §11).",
            ),
            FieldContract::reference("package_id", FieldContract::id(), "The owning package.")
                .with_fk("authored.packages", "package_id"),
            FieldContract::label(
                "path",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The document path, relative to the package root.",
            ),
            FieldContract::payload(
                "source_text",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Exact original UTF-8 source, including whitespace and comments (ADR-0068).",
            ),
        ]),
    );
}

/// `authored.entities @1` (blueprint §6.1).
fn declare_entities(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "entities",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Explicitly declared authored entities; registration validates exact declaration correspondence.",
        )
        .stability(Stability::Stable)
        .pk(&["entity_id"])
        .columns(vec![
            FieldContract::key("entity_id", FieldContract::id(), "The entity identity."),
            FieldContract::reference("package_id", FieldContract::id(), "The declaring package.")
                .with_fk("authored.packages", "package_id"),
            FieldContract::label(
                "kind",
                FieldContract::enumeration("EntityKind"),
                "What kind of entity it is.",
            ),
            FieldContract::label(
                "name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "An attribute, never identity: a rename changes this and nothing else.",
            ),
            FieldContract::label(
                "qualified_name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "An attribute, never identity — except under the `named` policy, where it \
                 determines the identity at creation and a rename is a new entity.",
            ),
            FieldContract::reference(
                "parent_entity_id",
                FieldContract::id(),
                "Containment: flowsheet ⊃ unit ⊃ control volume ⊃ state block.",
            )
            .with_fk("authored.entities", "entity_id")
            .optional(),
            FieldContract::provenance(
                "source_span",
                FieldContract::extended(ExtensionUse::SourceSpan),
                "Where the entity was declared.",
            )
            .optional(),
        ]),
    );
}

/// `reference.aliases @1` (blueprint §6.1).
fn declare_aliases(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Reference,
            "aliases",
            1,
            Authority::Reference,
            SnapshotClass::Model,
            "Deprecated qualified names of named-policy entities: what a reference package leaves \
             behind instead of a rename (blueprint §5.1).",
        )
        .stability(Stability::Stable)
        .pk(&["alias_id"])
        .columns(vec![
            FieldContract::key("alias_id", FieldContract::id(), "The alias identity."),
            FieldContract::reference(
                "entity_id",
                FieldContract::id(),
                "The entity that kept the name.",
            )
            .with_fk("authored.entities", "entity_id"),
            FieldContract::label(
                "old_qualified_name",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The deprecated qualified name.",
            ),
            FieldContract::label(
                "deprecated_in",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The package version that deprecated it.",
            ),
        ]),
    );
}

/// `normalized.package_graph @1` (blueprint §6.1, revision 6).
fn declare_package_graph(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Normalized,
            "package_graph",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "P0's output: every referenced package pinned by content hash, with its depth and its \
             resolved dependencies. Phase 0 admits exact version requirements only.",
        )
        .stability(Stability::Stable)
        .granularity(DerivationGranularity::Row)
        .pk(&["package_id"])
        .columns(vec![
            FieldContract::key("package_id", FieldContract::id(), "The resolved package.")
                .with_fk("authored.packages", "package_id"),
            FieldContract::label(
                "version",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "The resolved version.",
            ),
            FieldContract::payload(
                "content_hash",
                FieldContract::hash(),
                "The digest the package is pinned by.",
            ),
            FieldContract::payload(
                "depth",
                FieldContract::native(arrow_schema::DataType::UInt16),
                "The distance from the root package.",
            ),
            FieldContract::payload(
                "dependency_package_ids",
                FieldContract::list(FieldContract::id()),
                "The resolved direct dependencies.",
            ),
            FieldContract::provenance(
                "derivation_id",
                FieldContract::id(),
                "The `provenance.derivations` row this fact came from.",
            ),
        ]),
    );
}
