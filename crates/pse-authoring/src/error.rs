// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What authoring can be wrong about (blueprint §23.2 `authoring.parse`,
//! `authoring.reference`).
//!
//! Every variant carries a [`crate::span::SourceSpan`] where one exists, because an
//! authoring failure that cannot point at the document that caused it is a failure the
//! author has to find by bisection.

use pse_ids::SemanticId;

use crate::span::SourceSpan;

/// A document, identity, reference or package resolution that does not hold.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum AuthoringError {
    /// Shared platform memory could not be reserved before construction.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Resource(#[from] pse_ids::ReserveError),
    /// Cancellation or checked allocation extent failure.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Allocation(#[from] pse_ids::CanonError),
    /// Actual generated row validation failed during owned construction.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Relation(#[from] pse_relations::RelationError),
    /// A package document could not be read within its declared filesystem boundary.
    #[error("document `{path}`: {reason}")]
    #[diagnostic(code(authoring::parse::document_io))]
    DocumentIo {
        /// Package-relative source path.
        path: String,
        /// Concrete read or boundary failure.
        reason: String,
    },
    /// Actual candidate values violate a declared schema, key or reference contract.
    #[error("authoring contract: {reason}")]
    #[diagnostic(code(authoring::reference::contract))]
    Contract {
        /// Source location, if the failure belongs to one document row.
        at: Option<SourceSpan>,
        /// The violated semantic requirement.
        reason: String,
    },
    /// The document does not parse.
    #[error("{expected} expected at byte {offset}, found {found}")]
    #[diagnostic(
        code(authoring::parse::syntax),
        help("the parser reports the first offset it could not continue from")
    )]
    Syntax {
        /// Where the parser stopped.
        at: SourceSpan,
        /// The byte offset within the document.
        offset: u32,
        /// What the grammar admits at that point.
        expected: String,
        /// What was there instead.
        found: String,
    },

    /// The document carries a key the schema does not declare.
    #[error("unknown key `{key}` in {context}")]
    #[diagnostic(
        code(authoring::parse::unknown_key),
        help("an unknown key is a typo or a version mismatch; it is never ignored")
    )]
    UnknownKey {
        /// Where the key appears.
        at: SourceSpan,
        /// The offending key.
        key: String,
        /// The enclosing document section.
        context: String,
    },

    /// An `explicit`-policy entity has no `id`.
    #[error("{kind} `{name}` has no `id`, and its package uses the explicit identity policy")]
    #[diagnostic(
        code(authoring::parse::missing_id),
        help(
            "`pse authoring assign-ids` inserts a UUIDv7; identity is assigned at creation, never computed from the name (blueprint §5.1)"
        )
    )]
    MissingId {
        /// Where the entity is declared.
        at: SourceSpan,
        /// The entity kind.
        kind: String,
        /// The entity's name.
        name: String,
    },

    /// A `pse.target_path` names nothing.
    #[error("target `{path}` resolves to no entity")]
    #[diagnostic(
        code(authoring::parse::unresolved_target),
        help(
            "P1 resolves every target to identities at commit; an unresolvable target is not deferred to run time"
        )
    )]
    UnresolvedTarget {
        /// Where the target appears.
        at: SourceSpan,
        /// The selector path.
        path: String,
    },

    /// The document exceeds a parse budget.
    #[error("parse budget exceeded: {limit} allows {allowed}, the document needs {needed}")]
    #[diagnostic(
        code(authoring::parse::budget),
        help(
            "the budget bounds nesting, aliases and allocation so that hostile input is refused rather than survived (blueprint §22.1)"
        )
    )]
    Budget {
        /// Which budget: `depth`, `aliases` or `bytes`.
        limit: &'static str,
        /// What the budget allows.
        allowed: u64,
        /// What the document needs.
        needed: u64,
    },

    /// A change set tries to write a derived relation.
    #[error("`{relation}` is derived and cannot be authored")]
    #[diagnostic(
        code(authoring::reference::derived_write),
        help(
            "author causes, derive consequences (decision D2); an \"expected\" derived fact goes to `provenance.assertions`"
        )
    )]
    DerivedWrite {
        /// The qualified relation.
        relation: String,
    },

    /// A change set renames a named-policy entity.
    #[error("`{qualified_name}` belongs to a named-policy package and cannot be renamed")]
    #[diagnostic(
        code(authoring::reference::rename_named),
        help(
            "under the named policy a rename is a new entity; deprecate the old name with a `reference.aliases` row (blueprint §5.1)"
        )
    )]
    RenameNamed {
        /// The entity.
        entity_id: SemanticId,
        /// Its qualified name.
        qualified_name: String,
    },

    /// A change operation names a row that is not there.
    #[error("`{relation}` has no row with the key this operation names")]
    #[diagnostic(
        code(authoring::reference::unknown_row_key),
        help("the base revision moved; rebase the change set rather than applying it partially")
    )]
    UnknownRowKey {
        /// The qualified relation.
        relation: String,
        /// The key, rendered.
        row_key: String,
    },

    /// A declared dependency does not resolve.
    #[error("package `{name}` requires `{dependency}`, which is not available")]
    #[diagnostic(
        code(authoring::pkg::unresolved),
        help(
            "every referenced package is pinned by content hash after P0; an unresolved one stops resolution"
        )
    )]
    PackageUnresolved {
        /// The depending package.
        name: String,
        /// The dependency it names.
        dependency: String,
    },

    /// Two packages require incompatible versions of a third.
    #[error("`{dependency}` is required at both `{first}` and `{second}`")]
    #[diagnostic(
        code(authoring::pkg::version_conflict),
        help(
            "phase 0 admits exact version requirements only, so a conflict is a conflict, never a range to intersect"
        )
    )]
    PackageVersionConflict {
        /// The dependency.
        dependency: String,
        /// One required version.
        first: String,
        /// The other.
        second: String,
    },

    /// The document was written against a different registry.
    #[error("`{relation}` is at version {found} here and version {expected} in the registry")]
    #[diagnostic(
        code(schema::version_mismatch),
        help("migrate the document, or read it through the registry it was written against")
    )]
    SchemaVersionMismatch {
        /// The qualified relation.
        relation: String,
        /// The version the registry declares.
        expected: u32,
        /// The version the document claims.
        found: u32,
    },
}
