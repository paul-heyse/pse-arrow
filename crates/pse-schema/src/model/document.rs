// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authoring document shapes, declared (blueprint §22.1, ADR-0051).
//!
//! A `DocumentSpec` says which authored relation each section of a package document
//! populates. `crates/pse-authoring/src/generated/documents.rs` — the serde structs the
//! package loader parses into — and `docs/generated/schema/authoring.schema.json` are
//! generated from these, so the YAML surface and the relation it lands in cannot drift
//! apart without a red `codegen --check`.
//!
//! Packet A-6 fills `catalog::documents`; this module is the type it fills it with.

use core::fmt;

/// What kind of file a document is (blueprint §22.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentKind {
    /// `package.toml`: the package header, parsed by `toml` with `Spanned<T>`.
    PackageHeader,
    /// A YAML entity document under one of the package's directories.
    Entities,
}

impl DocumentKind {
    /// Both kinds.
    pub const ALL: [Self; 2] = [Self::PackageHeader, Self::Entities];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PackageHeader => "package_header",
            Self::Entities => "entities",
        }
    }
}

impl fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Declared grammar of one authoring DSL field, including nested values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DslSyntax {
    /// Arithmetic or value expression.
    Expression,
    /// Boolean predicate, including an atomic Boolean feature.
    Predicate,
    /// A relation or conditional equation.
    Equation,
}
impl DslSyntax {
    /// Complete grammar inventory.
    pub const ALL: [Self; 3] = [Self::Expression, Self::Predicate, Self::Equation];
    /// Stable declared spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expression => "expression",
            Self::Predicate => "predicate",
            Self::Equation => "equation",
        }
    }
}

/// One top-level key of a document, and the relation its rows land in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentSection {
    /// The document key, for example `templates`.
    pub key: &'static str,
    /// The qualified relation the section populates.
    pub relation: &'static str,
    /// Whether the key holds a sequence of entities rather than one.
    pub repeated: bool,
    /// Sole semantic-ID key that accepts the authoring `id` alias.
    pub identity_column: Option<&'static str>,
    /// Registered `EntityKind` member; absent means no entity registration.
    pub entity_kind: Option<&'static str>,
    /// Text column carrying the entity's local name.
    pub name_column: Option<&'static str>,
    /// Explicit owning entity foreign key; absent means package scope.
    pub naming_scope_column: Option<&'static str>,
    /// Owning template foreign key used by the shared source expression binder.
    pub expression_owner_column: Option<&'static str>,
    /// Exact DSL leaf paths (`column[].field`) and their grammar, in declaration order.
    pub expression_fields: &'static [(&'static str, DslSyntax)],
    /// What the section declares.
    pub doc: &'static str,
}

/// Source-column treatment implied by the declared document and relation context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceColumn {
    /// The document supplies this value, subject to declared nullability and identity policy.
    Authored,
    /// A package foreign key may be omitted and is verified against the enclosing package.
    PackageContext,
    /// The parser supplies this value; authored overrides are forbidden.
    ParserSpan,
    /// The loader computes this package integrity field from the complete source inventory.
    PackageIntegrity,
}
impl DocumentSection {
    /// One shared projection used by source editors and loader hydration.
    pub fn source_column(&self, column: &super::ColumnSpec) -> SourceColumn {
        if column.logical_type == super::LogicalType::Ext(super::ExtensionUse::SourceSpan) {
            SourceColumn::ParserSpan
        } else if self.relation == "authored.packages" && column.name == "content_hash" {
            SourceColumn::PackageIntegrity
        } else if column
            .fk
            .is_some_and(|fk| fk.relation == "authored.packages" && fk.column == "package_id")
        {
            SourceColumn::PackageContext
        } else {
            SourceColumn::Authored
        }
    }
}

/// A declared authoring document shape (blueprint §22.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSpec {
    /// The document's name, which is also its generated struct's name stem, for example
    /// `templates`.
    pub name: &'static str,
    /// What kind of file it is.
    pub kind: DocumentKind,
    /// The glob the loader matches, relative to the package root, for example
    /// `templates/*.yaml`.
    pub path_glob: &'static str,
    /// The sections, in declaration order.
    pub sections: Vec<DocumentSection>,
    /// What the document is for.
    pub doc: &'static str,
}
