// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every failure the catalog and the artifact store can return, carrying its blueprint
//! §23.2 class as a `#[diagnostic(code(...))]` in Rust path form.
//!
//! One enum rather than one per module, because the §23.2 class — not the module a
//! failure happened to originate in — is what a caller dispatches on. A reader who has
//! to ask "is a `StoreError::Corrupt` the same thing as a `ProviderError::Corrupt`?" has
//! already lost the taxonomy.
//!
//! The three `#[from]` variants are `#[diagnostic(transparent)]`: `pse-ids` already
//! assigned those failures a §23.2 class, and re-classifying them here would create a
//! second opinion about the same event.

use pse_ids::{ContentHash, LogicalHash};

/// A catalog, artifact-store or session failure (blueprint §23.2).
///
/// # Why some of these look alike
///
/// [`Self::CorruptObject`] and [`Self::LogicalHashMismatch`] are both "the bytes are not
/// what the manifest says", and ADR-0045 insists they stay apart: the first is an
/// encoding checksum over stored bytes and means the object is damaged, the second is a
/// `pse.canon.v2` logical hash and means the object is intact but does not hold the
/// relation it claims to. Collapsing them would make a corrupt disk and a mislabelled
/// publication indistinguishable.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum CatalogError {
    /// A declared Arrow field or local value contract failed admission.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Relation(std::sync::Arc<pse_relations::RelationError>),
    /// A typed semantic diagnostic from the rule layer, without a dependency cycle.
    /// Shared ownership preserves its code, labels and related findings through engine
    /// `Shared` wrappers instead of stringifying a non-cloneable source.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Semantic(std::sync::Arc<dyn miette::Diagnostic + Send + Sync>),

    /// Every classified engine leaf, each retaining its own diagnostic class.
    #[error("{} platform failures", .errors.len())]
    Multiple {
        /// Complete related findings, in the engine's original traversal order.
        #[related]
        errors: Vec<CatalogError>,
    },
    /// An accounted consumer could not obtain the memory it needs.
    ///
    /// Recoverable and configuration-driven: `config_keys` names the settings an
    /// operator can change, because a `runtime::resource_limit` a reader cannot act on
    /// is a stack trace with better manners (blueprint §14.3, ADR-0046).
    #[error("`{consumer}` exhausted its budget: {detail}")]
    #[diagnostic(
        code(runtime::resource_limit),
        help("adjust one of: {}", config_keys.join(", "))
    )]
    ResourceLimit {
        /// The consumer that asked, as the engine or the platform named it.
        consumer: String,
        /// Configuration keys named by the engine or established by the bound runtime,
        /// in order of first appearance. The original engine detail remains unchanged.
        config_keys: Vec<String>,
        /// The engine's own message, kept verbatim.
        detail: String,
    },

    /// A cancellation checkpoint observed a cancelled token.
    #[error("cancelled")]
    #[diagnostic(code(runtime::cancelled))]
    Cancelled,

    /// An object store, filesystem or Arrow operation failed on well-formed input.
    #[error("{op} failed")]
    #[diagnostic(code(runtime::infrastructure))]
    Infrastructure {
        /// What was being attempted, in the imperative: `read manifest`, `put relation`.
        op: String,
        /// The underlying failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A stored object's bytes do not match the checksum its name claims.
    ///
    /// Never a successful idempotent write: §20.1 requires an existing object to be
    /// verified before `AlreadyExists` is accepted, and a truncated object under a
    /// plausible name is exactly what that verification exists to catch.
    #[error("`{path}` hashes to {actual}, not the {expected} its name claims")]
    #[diagnostic(code(runtime::infrastructure))]
    CorruptObject {
        /// The object-store path of the offending object.
        path: String,
        /// The checksum the path asserts.
        expected: String,
        /// The checksum the bytes actually have.
        actual: String,
    },

    /// A ref's compare-and-swap lost to a concurrent publication.
    #[error("ref `{name}` moved under a conditional update")]
    #[diagnostic(code(runtime::infrastructure))]
    RefConflict {
        /// The ref that moved.
        name: String,
    },

    /// Execution or delivery failed after a conditional write may have taken effect.
    /// Exact outcomes must be reconciled before retrying the logical operation.
    #[error("publication outcome must be reconciled: {source}")]
    #[diagnostic(code(runtime::infrastructure))]
    Publication {
        /// Every actual write in this operation, independent of native result delivery.
        outcomes: Vec<crate::store::publication::PublicationOutcome>,
        /// Original failure, retaining its diagnostic class and detail.
        #[source]
        source: Box<CatalogError>,
    },

    /// A manifest is structurally or semantically inconsistent (blueprint §20.2).
    #[error("manifest is invalid: {reason}")]
    #[diagnostic(code(schema::manifest_invalid))]
    ManifestInvalid {
        /// Which of the §20.2 validations failed, and on what.
        reason: String,
    },

    /// A snapshot was declared under a schema registry this build does not have.
    ///
    /// Refused rather than read optimistically: §20.5 rejects an unknown registry
    /// fingerprint outright, because a relation read under the wrong contract decodes
    /// into plausible nonsense.
    #[error("schema registry fingerprint {fingerprint} is unknown")]
    #[diagnostic(code(schema::unknown_registry))]
    UnknownRegistry {
        /// The fingerprint the manifest carries.
        fingerprint: ContentHash,
    },

    /// A format version string this build does not implement (blueprint §20.5).
    #[error("`{field}` is `{value}`, which this build does not implement")]
    #[diagnostic(code(schema::unknown_version))]
    UnknownVersion {
        /// The manifest field carrying the version.
        field: String,
        /// The version that was offered.
        value: String,
    },

    /// A plan, batch or import was refused by semantic admission (blueprint §4.4, §5.4).
    #[error("`{path}` was refused at admission: {reason}")]
    #[diagnostic(code(schema::admission))]
    Admission {
        /// Where in the plan or schema the refusal happened.
        path: String,
        /// Why.
        reason: String,
    },

    /// A plan scanned a table that is not a snapshot relation.
    ///
    /// §5.4 pins a session to a snapshot; a scan of anything else would put a second,
    /// unversioned authority into the same query.
    #[error("`{table}` is not a relation of this snapshot")]
    #[diagnostic(code(schema::foreign_source))]
    ForeignSource {
        /// The offending table reference.
        table: String,
    },

    /// Registration was attempted after the session was sealed (blueprint §5.4).
    #[error("the session is sealed; its catalogs cannot change")]
    #[diagnostic(code(schema::sealed))]
    Sealed,

    /// A relation's decoded content does not hash to its declared logical hash.
    #[error("`{relation}` hashes to {actual}, but the manifest declares {expected}")]
    #[diagnostic(code(validation::invariant))]
    LogicalHashMismatch {
        /// The relation, as `namespace.name`.
        relation: String,
        /// The manifest's declared logical hash.
        expected: LogicalHash,
        /// What the decoded relation actually hashes to.
        actual: LogicalHash,
    },

    /// Snapshot membership is incomplete or inconsistent (blueprint §5.3 step 7).
    #[error("snapshot membership is invalid: {reason}")]
    #[diagnostic(code(validation::invariant))]
    Membership {
        /// Which membership rule failed.
        reason: String,
    },

    /// A configuration key was rejected by the typed configuration path.
    ///
    /// Typed, because `SessionConfig::set_str` panics on an invalid value instead of
    /// returning one of these; it is banned in `clippy.toml` for that reason.
    #[error("`{key}` is invalid: {reason}")]
    #[diagnostic(code(config::invalid))]
    ConfigInvalid {
        /// The configuration key.
        key: String,
        /// Why the value was refused.
        reason: String,
    },

    /// An authored query or model is wrong (blueprint §23.2, `user.model`).
    #[error("{message}")]
    #[diagnostic(code(user::model))]
    UserModel {
        /// What the user did that cannot work.
        message: String,
    },

    /// A generated kernel could not evaluate at the point it was given.
    #[error("{message}")]
    #[diagnostic(code(solve::evaluation_error))]
    EvaluationError {
        /// The engine's message.
        message: String,
    },

    /// A property was unsupported, ambiguous or unresolved in a kernel.
    #[error("{message}")]
    #[diagnostic(code(compile::property))]
    CompileProperty {
        /// The engine's message.
        message: String,
    },

    /// A `pse.canon.v2` failure, keeping the class `pse-ids` gave it.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Canon(#[from] pse_ids::CanonError),

    /// A snapshot frame could not be named, keeping the class `pse-ids` gave it.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Snapshot(#[from] pse_ids::SnapshotError),

    /// A reservation was refused, keeping the class `pse-ids` gave it.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Reserve(#[from] pse_ids::ReserveError),

    /// A postcondition of this crate failed: a platform bug, never a user error.
    #[error("internal invariant: {message}")]
    #[diagnostic(code(internal::invariant))]
    Internal {
        /// What was expected to hold.
        message: String,
    },
}

impl From<pse_relations::RelationError> for CatalogError {
    fn from(error: pse_relations::RelationError) -> Self {
        Self::Relation(std::sync::Arc::new(error))
    }
}

#[cfg(test)]
mod tests {
    use miette::Diagnostic;

    use super::*;

    /// The §23.2 class of an error, as the diagnostic code renders it.
    fn code_of(error: &CatalogError) -> String {
        error
            .code()
            .map_or_else(|| "<none>".to_owned(), |code| code.to_string())
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "one row per §23.2 class; splitting the table hides that it is exhaustive"
    )]
    fn every_variant_carries_its_blueprint_class() {
        let cases: Vec<(CatalogError, &str)> = vec![
            (
                CatalogError::ResourceLimit {
                    consumer: "sort".to_owned(),
                    config_keys: vec!["datafusion.runtime.memory_limit".to_owned()],
                    detail: "out of memory".to_owned(),
                },
                "runtime::resource_limit",
            ),
            (CatalogError::Cancelled, "runtime::cancelled"),
            (
                CatalogError::Infrastructure {
                    op: "read manifest".to_owned(),
                    source: Box::new(std::io::Error::other("no such object")),
                },
                "runtime::infrastructure",
            ),
            (
                CatalogError::CorruptObject {
                    path: "manifests/aa.json".to_owned(),
                    expected: "aa".to_owned(),
                    actual: "bb".to_owned(),
                },
                "runtime::infrastructure",
            ),
            (
                CatalogError::RefConflict {
                    name: "head".to_owned(),
                },
                "runtime::infrastructure",
            ),
            (
                CatalogError::ManifestInvalid {
                    reason: "duplicate port".to_owned(),
                },
                "schema::manifest_invalid",
            ),
            (
                CatalogError::UnknownRegistry {
                    fingerprint: ContentHash::NIL,
                },
                "schema::unknown_registry",
            ),
            (
                CatalogError::UnknownVersion {
                    field: "manifest_version".to_owned(),
                    value: "pse.manifest.v1".to_owned(),
                },
                "schema::unknown_version",
            ),
            (
                CatalogError::Admission {
                    path: "0".to_owned(),
                    reason: "unknown extension".to_owned(),
                },
                "schema::admission",
            ),
            (
                CatalogError::ForeignSource {
                    table: "other.t".to_owned(),
                },
                "schema::foreign_source",
            ),
            (CatalogError::Sealed, "schema::sealed"),
            (
                CatalogError::LogicalHashMismatch {
                    relation: "authored.units".to_owned(),
                    expected: LogicalHash(ContentHash::NIL),
                    actual: LogicalHash(ContentHash::from_bytes([1; 32])),
                },
                "validation::invariant",
            ),
            (
                CatalogError::Membership {
                    reason: "missing required relation".to_owned(),
                },
                "validation::invariant",
            ),
            (
                CatalogError::ConfigInvalid {
                    key: "datafusion.execution.target_partitions".to_owned(),
                    reason: "oversubscribed".to_owned(),
                },
                "config::invalid",
            ),
            (
                CatalogError::UserModel {
                    message: "no such column".to_owned(),
                },
                "user::model",
            ),
            (
                CatalogError::EvaluationError {
                    message: "log of a negative number".to_owned(),
                },
                "solve::evaluation_error",
            ),
            (
                CatalogError::CompileProperty {
                    message: "unresolved method".to_owned(),
                },
                "compile::property",
            ),
            (
                CatalogError::Internal {
                    message: "postcondition".to_owned(),
                },
                "internal::invariant",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(code_of(&error), expected, "{error}");
        }
    }

    #[test]
    fn the_from_variants_keep_the_class_pse_ids_assigned() {
        let reserve = CatalogError::from(pse_ids::ReserveError::Exhausted {
            owner: "canonicalize".to_owned(),
            requested: 1,
            reserved: 0,
            limit_hint: "FixedBudget limit_bytes=0".to_owned(),
        });
        assert_eq!(code_of(&reserve), "runtime::resource_limit");

        let snapshot = CatalogError::from(pse_ids::SnapshotError::MissingModelParent);
        assert_eq!(code_of(&snapshot), "validation::invariant");

        let canon = CatalogError::from(pse_ids::CanonError::Cancelled);
        assert_eq!(code_of(&canon), "runtime::cancelled");
    }
}
