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

/// A declared unique key besides the primary key. Rows whose key columns are all
/// present are unique; a row with an absent component is not constrained.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueKey {
    /// The key's name inside its relation.
    pub name: &'static str,
    /// The key columns, in key order.
    pub columns: Vec<&'static str>,
}

/// A declared (possibly composite) reference from ordered local columns to a target
/// key. A row whose local columns are all present references exactly one target row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ForeignKeyDecl {
    /// The reference's name inside its relation.
    pub name: &'static str,
    /// The local columns, in order.
    pub columns: Vec<&'static str>,
    /// The qualified target relation.
    pub target: &'static str,
    /// The target columns, pairwise with [`Self::columns`]; they form the target's
    /// primary key or one of its unique keys.
    pub target_columns: Vec<&'static str>,
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
    /// Named native DataFusion SQL predicates. Each must evaluate to true for a row;
    /// false and null are violations. Binding uses the actual execution session.
    pub checks: std::collections::BTreeMap<String, String>,
    /// Unique keys besides the primary key, in declaration order.
    pub unique_keys: Vec<UniqueKey>,
    /// Table-level (composite) references, in declaration order. A single column's
    /// reference is its field facet ([`FieldContract::fk`]).
    pub foreign_keys: Vec<ForeignKeyDecl>,
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

    /// Stable identity of a named native check in this exact relation version.
    /// The predicate remains authoritative in `checks`; this label grants no validity.
    pub fn row_check_id(&self, name: &str) -> Option<SemanticId> {
        self.checks
            .contains_key(name)
            .then(|| pse_ids::named_id(self.id, &format!("native-row-check:{name}")))
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
    /// Explicit ordered key declaration. `None` is undeclared; `Some(vec![])`
    /// declares a singleton relation with at most one row.
    pub primary_key: Option<Vec<&'static str>>,
    /// See [`RelationSpec::columns`].
    pub columns: Vec<FieldContract>,
    /// See [`RelationSpec::checks`].
    pub checks: std::collections::BTreeMap<String, String>,
    /// See [`RelationSpec::unique_keys`].
    pub unique_keys: Vec<UniqueKey>,
    /// See [`RelationSpec::foreign_keys`].
    pub foreign_keys: Vec<ForeignKeyDecl>,
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
            primary_key: None,
            columns: Vec::new(),
            checks: std::collections::BTreeMap::new(),
            unique_keys: Vec::new(),
            foreign_keys: Vec::new(),
            doc,
        }
    }

    /// The same declaration with `primary_key` set.
    #[must_use]
    pub fn pk(mut self, columns: &[&'static str]) -> Self {
        self.primary_key = Some(columns.to_vec());
        self
    }

    /// The same declaration with `columns` set.
    #[must_use]
    pub fn columns(mut self, columns: Vec<FieldContract>) -> Self {
        self.columns = columns;
        self
    }

    /// Declare native SQL row predicates. Keys are stable names, values are SQL
    /// expressions; there is no application expression language or evaluator.
    #[must_use]
    pub fn checks(mut self, checks: std::collections::BTreeMap<String, String>) -> Self {
        self.checks = checks;
        self
    }

    /// Add one named native row predicate (see [`RelationSpec::checks`]).
    #[must_use]
    pub fn check(mut self, name: &str, sql: impl Into<String>) -> Self {
        self.checks.insert(name.to_owned(), sql.into());
        self
    }

    /// Declare a unique key besides the primary key.
    #[must_use]
    pub fn unique(mut self, name: &'static str, columns: &[&'static str]) -> Self {
        self.unique_keys.push(UniqueKey {
            name,
            columns: columns.to_vec(),
        });
        self
    }

    /// Declare a (composite) reference from `columns` to `target_columns` of the
    /// qualified relation `target`.
    #[must_use]
    pub fn foreign_key(
        mut self,
        name: &'static str,
        columns: &[&'static str],
        target: &'static str,
        target_columns: &[&'static str],
    ) -> Self {
        self.foreign_keys.push(ForeignKeyDecl {
            name,
            columns: columns.to_vec(),
            target,
            target_columns: target_columns.to_vec(),
        });
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
