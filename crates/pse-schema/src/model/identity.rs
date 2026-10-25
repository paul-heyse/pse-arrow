// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entity identities: which thing an identity column names (ADR-0115 Outcome 1).
//!
//! A relation's identity column may declare the entity identity it carries, and a
//! foreign-key column referencing it inherits that identity at assembly. The declaration
//! is the one source of every typed id: the generated Rust newtype and Python alias.
//! An identity never changes bytes, framing or serde; it
//! only makes a swap between entities a type error.

use pse_ids::SemanticId;

/// An entity identity before assembly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityDecl {
    /// The identity's name, a snake-case entity noun such as `attempt`.
    pub name: &'static str,
    /// What the identity identifies.
    pub doc: &'static str,
    /// Optional existing native owner; generation metadata, excluded from scientific fingerprints.
    pub source: Option<IdentitySource>,
}

impl IdentityDecl {
    /// Reuse an existing native identity with an explicitly declared storage base.
    pub const fn sourced(name: &'static str, doc: &'static str, path: &'static str, base: IdentityBase) -> Self {
        Self { name, doc, source: Some(IdentitySource { path, base }) }
    }

    /// Declare the identity `name`.
    pub const fn new(name: &'static str, doc: &'static str) -> Self {
        Self { name, doc, source: None }
    }
}

/// Existing native identity owner and its storage contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentitySource {
    /// Absolute Rust type path used by code generation.
    pub path: &'static str,
    /// Storage base validated against all declared carriers.
    pub base: IdentityBase,
}

/// The value an identity wraps; decided by the columns that carry it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentityBase {
    /// A 128-bit semantic identity (`pse.semantic_id`).
    SemanticId,
    /// A 256-bit content hash (`pse.content_hash`), for content-addressed entities.
    ContentHash,
}

impl IdentityBase {
    /// The registry logical type of the base value.
    pub const fn logical_type(self) -> &'static str {
        match self {
            Self::SemanticId => "semantic_id",
            Self::ContentHash => "content_hash",
        }
    }
}

/// The relation whose single-column primary key was declared the identity's owner
/// ([`crate::model::FieldContract::with_owned_identity`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityOwner {
    /// The owning relation's identity.
    pub relation_id: SemanticId,
    /// The owning relation's qualified name.
    pub relation: String,
    /// The key column.
    pub column: String,
}

/// An assembled entity identity (`reference.schema_identities`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentitySpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "identity:<name>")` (ADR-0050).
    pub id: SemanticId,
    /// See [`IdentityDecl::name`].
    pub name: &'static str,
    /// See [`IdentityDecl::doc`].
    pub doc: &'static str,
    /// The base value every carrying column shares.
    pub base: IdentityBase,
    /// The owning key, when one relation's single-column key was declared its owner.
    /// Unowned identities (a model instance, say) are carried by columns only.
    pub owner: Option<IdentityOwner>,
    /// Existing Rust owner, analogous to EnumSpec::source; not persisted scientific meaning.
    pub source: Option<&'static str>,
}

impl IdentitySpec {
    /// Restore generation ownership while reconstructing an admitted registry.
    pub(crate) fn with_source(mut self, source: Option<&'static str>) -> Self {
        self.source = source;
        self
    }

    /// Construct an assembled identity from its validated declaration and carriers.
    pub(crate) fn new(
        id: SemanticId,
        name: &'static str,
        doc: &'static str,
        base: IdentityBase,
        owner: Option<IdentityOwner>,
    ) -> Self {
        Self { id, name, doc, base, owner, source: None }
    }
}
