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

/// One top-level key of a document, and the relation its rows land in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentSection {
    /// The document key, for example `templates`.
    pub key: &'static str,
    /// The qualified relation the section populates.
    pub relation: &'static str,
    /// Whether the key holds a sequence of entities rather than one.
    pub repeated: bool,
    /// What the section declares.
    pub doc: &'static str,
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
