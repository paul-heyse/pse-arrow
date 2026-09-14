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
    Authority, ColumnSpec, DerivationGranularity, ExtensionUse, LogicalType, Namespace,
    RelationDecl, SnapshotClass, Stability,
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
            ColumnSpec::key("package_id", LogicalType::id(), "The package identity."),
            ColumnSpec::label("name", LogicalType::Text, "The package name."),
            ColumnSpec::label(
                "version",
                LogicalType::Text,
                "The package version (semver).",
            ),
            ColumnSpec::label(
                "kind",
                LogicalType::enumeration("PackageKind"),
                "What kind of package it is.",
            ),
            ColumnSpec::label(
                "id_policy",
                LogicalType::enumeration("IdPolicy"),
                "`explicit` or `named` (blueprint §5.1). Recorded on the package, never in the ID.",
            ),
            ColumnSpec::payload(
                "dependencies",
                LogicalType::list(LogicalType::Struct(vec![
                    ("package_id", LogicalType::id(), false),
                    ("version_req", LogicalType::Text, false),
                ])),
                "The declared dependencies. Phase 0 admits exact version requirements only.",
            ),
            ColumnSpec::payload(
                "content_hash",
                LogicalType::hash(),
                "The digest of the package's documents.",
            ),
            ColumnSpec::label("doc", LogicalType::Text, "What the package is for."),
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
            ColumnSpec::key(
                "document_id",
                LogicalType::id(),
                "`named_id(package_id, path)` (blueprint §11).",
            ),
            ColumnSpec::reference("package_id", LogicalType::id(), "The owning package.")
                .with_fk("authored.packages", "package_id"),
            ColumnSpec::label(
                "path",
                LogicalType::Text,
                "The document path, relative to the package root.",
            ),
            ColumnSpec::payload(
                "content_hash",
                LogicalType::hash(),
                "The digest of the document bytes.",
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
            "Every authored entity. The invariant `closure:entity_registered` requires every other \
             authored relation's primary key to appear here.",
        )
        .stability(Stability::Stable)
        .pk(&["entity_id"])
        .columns(vec![
            ColumnSpec::key("entity_id", LogicalType::id(), "The entity identity."),
            ColumnSpec::reference("package_id", LogicalType::id(), "The declaring package.")
                .with_fk("authored.packages", "package_id"),
            ColumnSpec::label(
                "kind",
                LogicalType::enumeration("EntityKind"),
                "What kind of entity it is.",
            ),
            ColumnSpec::label(
                "name",
                LogicalType::Text,
                "An attribute, never identity: a rename changes this and nothing else.",
            ),
            ColumnSpec::label(
                "qualified_name",
                LogicalType::Text,
                "An attribute, never identity — except under the `named` policy, where it \
                 determines the identity at creation and a rename is a new entity.",
            ),
            ColumnSpec::reference(
                "parent_entity_id",
                LogicalType::id(),
                "Containment: flowsheet ⊃ unit ⊃ control volume ⊃ state block.",
            )
            .with_fk("authored.entities", "entity_id")
            .optional(),
            ColumnSpec::provenance(
                "source_span",
                LogicalType::Ext(ExtensionUse::SourceSpan),
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
            ColumnSpec::key("alias_id", LogicalType::id(), "The alias identity."),
            ColumnSpec::reference(
                "entity_id",
                LogicalType::id(),
                "The entity that kept the name.",
            )
            .with_fk("authored.entities", "entity_id"),
            ColumnSpec::label(
                "old_qualified_name",
                LogicalType::Text,
                "The deprecated qualified name.",
            ),
            ColumnSpec::label(
                "deprecated_in",
                LogicalType::Text,
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
            ColumnSpec::key("package_id", LogicalType::id(), "The resolved package.")
                .with_fk("authored.packages", "package_id"),
            ColumnSpec::label("version", LogicalType::Text, "The resolved version."),
            ColumnSpec::payload(
                "content_hash",
                LogicalType::hash(),
                "The digest the package is pinned by.",
            ),
            ColumnSpec::payload(
                "depth",
                LogicalType::U16,
                "The distance from the root package.",
            ),
            ColumnSpec::payload(
                "dependency_package_ids",
                LogicalType::list(LogicalType::id()),
                "The resolved direct dependencies.",
            ),
            ColumnSpec::provenance(
                "derivation_id",
                LogicalType::id(),
                "The `provenance.derivations` row this fact came from.",
            ),
        ]),
    );
}
