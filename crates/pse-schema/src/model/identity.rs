// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entity identities: which thing an identity column names (ADR-0115 Outcome 1).
//!
//! A relation's identity column may declare the entity identity it carries, and a
//! foreign-key column referencing it inherits that identity at assembly. The declaration
//! is the one source of every typed id: the generated Rust newtype, the PostgreSQL
//! domain and the Python alias. An identity never changes bytes, framing or serde; it
//! only makes a swap between entities a type error.

use pse_ids::SemanticId;

/// An entity identity before assembly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityDecl {
    /// The identity's name, a snake-case entity noun such as `attempt`.
    pub name: &'static str,
    /// What the identity identifies.
    pub doc: &'static str,
}

impl IdentityDecl {
    /// Declare the identity `name`.
    pub const fn new(name: &'static str, doc: &'static str) -> Self {
        Self { name, doc }
    }
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

/// The relation whose single-column primary key declares an identity.
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
    /// The owning key, when one relation's single-column key declares the identity.
    /// Unowned identities (a run, a block) are named by keys that are not theirs alone.
    pub owner: Option<IdentityOwner>,
}
