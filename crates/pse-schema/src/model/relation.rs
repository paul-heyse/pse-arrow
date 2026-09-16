// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A relation and its columns: the declaration a `RelationSpec` is (blueprint §4.1).
//!
//! A relation is declared exactly once. Everything else — the Arrow schema, the typed
//! view, the builder, the validator, the provider, the documentation, the migration — is
//! generated from this declaration (§4.2), which is why nothing here is allowed to be
//! restated by hand anywhere else.

use core::cmp::Ordering;
use core::fmt;

use pse_ids::{ContentHash, SemanticId};

use crate::model::enums::{Authority, DerivationGranularity, Namespace, SnapshotClass, Stability};
use crate::model::field::FieldContract;

/// A relation's name, in the one form that is stable across renames of anything else.
///
/// `Ord` is by `(namespace spelling, name, version)` — the namespace's *text*, not the
/// variant order — so that a reordering of [`Namespace`] cannot silently reorder
/// `relations()` and change the registry fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RelationKey {
    /// The namespace, which is also the catalog schema (blueprint §5.4).
    pub namespace: Namespace,
    /// The relation name inside its namespace, for example `stoichiometry`.
    pub name: &'static str,
    /// The schema version, starting at 1 (blueprint §20.5).
    pub version: u32,
}

impl RelationKey {
    /// A key for `<namespace>.<name>@<version>`.
    pub const fn new(namespace: Namespace, name: &'static str, version: u32) -> Self {
        Self {
            namespace,
            name,
            version,
        }
    }

    /// The qualified name without the version, for example `authored.stoichiometry`.
    ///
    /// This is the form a foreign key, an invariant and a pass port name a relation by:
    /// they bind to the relation, and the version is the registry's business.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.namespace.as_str(), self.name)
    }

    /// The sort key: the namespace spelling, the name, then the version.
    fn sort_key(&self) -> (&'static str, &'static str, u32) {
        (self.namespace.as_str(), self.name, self.version)
    }
}

impl fmt::Display for RelationKey {
    /// `authored.stoichiometry@1`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}@{}",
            self.namespace.as_str(),
            self.name,
            self.version
        )
    }
}

impl Ord for RelationKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl PartialOrd for RelationKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The quantity contract a column carries (blueprint §4.1, §8.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuantityContract<'a> {
    /// The column carries no physical quantity.
    None,
    /// One quantity type for the whole column, named by its qualified name, for example
    /// `pse.quantity.temperature`.
    Column(&'a str),
    /// The quantity type varies per row. A sibling column named
    /// `<name>_quantity_type_id` declares it, and assembly rejects the declaration if that
    /// sibling is missing — a per-row contract with nowhere to put the contract is a
    /// column of unlabelled numbers.
    PerRow,
}

/// A reference from one relation's column to another's (blueprint §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ForeignKey<'a> {
    /// The qualified target relation, for example `authored.species`.
    pub relation: &'a str,
    /// The target column.
    pub column: &'a str,
}

impl<'a> ForeignKey<'a> {
    /// A foreign key into `relation.column`.
    pub const fn new(relation: &'a str, column: &'a str) -> Self {
        Self { relation, column }
    }
}

impl fmt::Display for ForeignKey<'_> {
    /// `authored.species.species_id`, which is also the `pse.semantic.fk` metadata value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.relation, self.column)
    }
}

/// A declared relation (blueprint §4.1 `reference.schema_relations`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationSpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "relation:<ns>.<name>@<v>")` (ADR-0050).
    pub id: SemanticId,
    /// The relation's name and version.
    pub key: RelationKey,
    /// Who may write it.
    pub authority: Authority,
    /// Which snapshot it belongs to (blueprint §5.3 step 7).
    pub snapshot_class: SnapshotClass,
    /// How much provenance a derived relation stores per head row. Required when
    /// `authority` is [`Authority::Derived`].
    pub derivation_granularity: Option<DerivationGranularity>,
    /// How much a consumer may rely on the shape.
    pub stability: Stability,
    /// The primary key column names, in key order.
    pub primary_key: Vec<&'static str>,
    /// The columns, in declaration order. The ordinal is the position.
    pub columns: Vec<FieldContract>,
    /// What the relation means.
    pub doc: &'static str,
    /// The BLAKE3 digest of this relation's registry rows, filled at assembly.
    ///
    /// It is [`ContentHash::NIL`] until [`crate::builder::RegistryBuilder::build`] fills
    /// it, which is also why a `RelationSpec` is never constructed by hand outside the
    /// builder: a hand-built one would carry a fingerprint that means nothing.
    pub fingerprint: ContentHash,
}

impl RelationSpec {
    /// The qualified name without the version.
    pub fn qualified_name(&self) -> String {
        self.key.qualified_name()
    }

    /// The column of that name, if the relation has one.
    pub fn column(&self, name: &str) -> Option<&FieldContract> {
        self.columns.iter().find(|column| column.name() == name)
    }
}

/// A [`RelationSpec`] before assembly, when it has neither identity nor fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationDecl {
    /// See [`RelationSpec::key`].
    pub key: RelationKey,
    /// See [`RelationSpec::authority`].
    pub authority: Authority,
    /// See [`RelationSpec::snapshot_class`].
    pub snapshot_class: SnapshotClass,
    /// See [`RelationSpec::derivation_granularity`].
    pub derivation_granularity: Option<DerivationGranularity>,
    /// See [`RelationSpec::stability`].
    pub stability: Stability,
    /// See [`RelationSpec::primary_key`].
    pub primary_key: Vec<&'static str>,
    /// See [`RelationSpec::columns`].
    pub columns: Vec<FieldContract>,
    /// See [`RelationSpec::doc`].
    pub doc: &'static str,
}

impl RelationDecl {
    /// A relation declaration with the defaults a catalog module almost always wants:
    /// [`Stability::Evolving`] and no derivation granularity.
    pub fn new(
        namespace: Namespace,
        name: &'static str,
        version: u32,
        authority: Authority,
        snapshot_class: SnapshotClass,
        doc: &'static str,
    ) -> Self {
        Self {
            key: RelationKey::new(namespace, name, version),
            authority,
            snapshot_class,
            derivation_granularity: None,
            stability: Stability::Evolving,
            primary_key: Vec::new(),
            columns: Vec::new(),
            doc,
        }
    }

    /// The same declaration with `primary_key` set.
    #[must_use]
    pub fn pk(mut self, columns: &[&'static str]) -> Self {
        self.primary_key = columns.to_vec();
        self
    }

    /// The same declaration with `columns` set.
    #[must_use]
    pub fn columns(mut self, columns: Vec<FieldContract>) -> Self {
        self.columns = columns;
        self
    }

    /// The same declaration at the given stability.
    #[must_use]
    pub const fn stability(mut self, stability: Stability) -> Self {
        self.stability = stability;
        self
    }

    /// The same declaration at the given derivation granularity.
    #[must_use]
    pub const fn granularity(mut self, granularity: DerivationGranularity) -> Self {
        self.derivation_granularity = Some(granularity);
        self
    }
}
