// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The closed vocabularies a `RelationSpec` is written in, and the declaration form of a
//! `pse.enum` (blueprint §4.1, §6.14).
//!
//! Two different things are called an "enum" here and they are deliberately separate. The
//! Rust enums below ([`Namespace`], [`Authority`], …) are the *model's* vocabulary: they
//! decide what a declaration may say. An [`EnumSpec`] is a *declared* closed dictionary
//! that becomes `reference.schema_enums` rows and a `pse.enum` column contract. The
//! platform vocabularies are also declared as `EnumSpec`s — in `catalog::enums_platform` —
//! so that the registry can describe itself (§4.1), and that is the only place the two
//! meet.

use core::fmt;

use pse_ids::SemanticId;

/// The seven namespaces a relation can live in (blueprint §4.1, §6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Namespace {
    /// Shipped contracts and reference data; named-policy identity (blueprint §5.1).
    Reference,
    /// Facts a human or an agent authored. The only writable namespace (decision D2).
    Authored,
    /// P3's canonical rewriting of authored facts.
    Normalized,
    /// Facts the rule engine inferred (P4–P6).
    Inferred,
    /// Compiler output: symbols, mathematics, plans.
    Compiled,
    /// Execution records and their results.
    Runtime,
    /// Evidence: derivations, pass records, assertions.
    Provenance,
}

impl Namespace {
    /// Every namespace, in blueprint §4.1 order.
    pub const ALL: [Self; 7] = [
        Self::Reference,
        Self::Authored,
        Self::Normalized,
        Self::Inferred,
        Self::Compiled,
        Self::Runtime,
        Self::Provenance,
    ];

    /// The wire spelling, which is also the catalog schema name (blueprint §5.4).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Authored => "authored",
            Self::Normalized => "normalized",
            Self::Inferred => "inferred",
            Self::Compiled => "compiled",
            Self::Runtime => "runtime",
            Self::Provenance => "provenance",
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Who is allowed to write a relation (blueprint §4.1, decision D2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Authority {
    /// Written by the authoring path only, through a change set (blueprint §22.2).
    Authored,
    /// Shipped by a reference package.
    Reference,
    /// Produced by a pass. Requires a [`DerivationGranularity`].
    Derived,
}

impl Authority {
    /// Every authority, in blueprint §4.1 order.
    pub const ALL: [Self; 3] = [Self::Authored, Self::Reference, Self::Derived];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authored => "authored",
            Self::Reference => "reference",
            Self::Derived => "derived",
        }
    }
}

impl fmt::Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which snapshot a relation is a member of (blueprint §5.3 step 7).
///
/// Membership is generated from this field and from nothing else: there is no namespace
/// wildcard and no implicit omission, because an omission that reads as "not relevant" and
/// an omission that reads as "forgotten" are indistinguishable in a hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapshotClass {
    /// A structural authored or reference contract.
    Model,
    /// A §6.10 case, specification or observation relation.
    Case,
    /// Compiler or runtime output.
    Derived,
    /// A revision catalog, change log, mutable ref, stage catalog or pass record. A
    /// sidecar never enters its own membership.
    Sidecar,
}

impl SnapshotClass {
    /// Every class, in blueprint §5.3 order.
    pub const ALL: [Self; 4] = [Self::Model, Self::Case, Self::Derived, Self::Sidecar];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Case => "case",
            Self::Derived => "derived",
            Self::Sidecar => "sidecar",
        }
    }
}

impl fmt::Display for SnapshotClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How much provenance a derived relation stores per head row (blueprint §14.2 rule 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DerivationGranularity {
    /// One `provenance.derivations` row per head row: inference, law expansion and method
    /// resolution, where negative completeness is the deliverable.
    Row,
    /// The derivation is the rule plus the §5.1 identity formula, reconstructed on demand:
    /// index expansion, discretization, connection equations.
    Rule,
}

impl DerivationGranularity {
    /// Both granularities.
    pub const ALL: [Self; 2] = [Self::Row, Self::Rule];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Rule => "rule",
        }
    }
}

impl fmt::Display for DerivationGranularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How much a consumer may rely on a relation's shape (blueprint §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stability {
    /// Public and versioned; a change needs a migration.
    Stable,
    /// Public but still moving; a change needs a migration and a note.
    Evolving,
    /// Platform-internal; no external consumer may depend on it.
    Internal,
}

impl Stability {
    /// Every level, in blueprint §4.1 order.
    pub const ALL: [Self; 3] = [Self::Stable, Self::Evolving, Self::Internal];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Evolving => "evolving",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for Stability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a column is for (blueprint §4.1).
///
/// The role is not decoration: [`ColumnRole::Key`] and [`ColumnRole::Reference`] are the
/// roles a rule plan may key on (§14.2 rule 7), and [`ColumnRole::Measure`] is the role
/// that carries a quantity contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColumnRole {
    /// Part of the primary key.
    Key,
    /// A reference to another relation's key.
    Reference,
    /// A numerical value, normally under a quantity contract.
    Measure,
    /// A human-facing name or label. Never identity.
    Label,
    /// Structured content that is neither key nor measure.
    Payload,
    /// Evidence: a derivation, a source span, a producing pass.
    Provenance,
}

impl ColumnRole {
    /// Every role, in blueprint §4.1 order.
    pub const ALL: [Self; 6] = [
        Self::Key,
        Self::Reference,
        Self::Measure,
        Self::Label,
        Self::Payload,
        Self::Provenance,
    ];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Reference => "reference",
            Self::Measure => "measure",
            Self::Label => "label",
            Self::Payload => "payload",
            Self::Provenance => "provenance",
        }
    }
}

impl fmt::Display for ColumnRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What kind of statement an invariant makes (blueprint §4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvariantKind {
    /// The named columns are unique.
    Unique,
    /// Every value resolves in the referenced relation.
    ForeignKey,
    /// A row-local predicate holds.
    Check,
    /// A group has a declared number of members.
    Cardinality,
    /// A value lies in a declared domain.
    Domain,
    /// A set is closed under a declared relation.
    Closure,
    /// A declared edge relation has no cycle.
    Acyclic,
}

impl InvariantKind {
    /// Every kind, in blueprint §4.1 order.
    pub const ALL: [Self; 7] = [
        Self::Unique,
        Self::ForeignKey,
        Self::Check,
        Self::Cardinality,
        Self::Domain,
        Self::Closure,
        Self::Acyclic,
    ];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unique => "unique",
            Self::ForeignKey => "foreign_key",
            Self::Check => "check",
            Self::Cardinality => "cardinality",
            Self::Domain => "domain",
            Self::Closure => "closure",
            Self::Acyclic => "acyclic",
        }
    }
}

impl fmt::Display for InvariantKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether a finding stops a commit (blueprint §4.1, §22.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// The commit contract is not met; the change set is rejected.
    Error,
    /// Reported, and the commit proceeds.
    Warning,
}

impl Severity {
    /// Both severities.
    pub const ALL: [Self; 2] = [Self::Error, Self::Warning];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A declared closed dictionary: one `pse.enum` contract and its
/// `reference.schema_enums` rows (blueprint §4.1, §4.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumSpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "enum:<Name>")` (ADR-0050).
    pub id: SemanticId,
    /// The enumeration's name, as it appears in `pse.enum(<Name>)`.
    pub name: &'static str,
    /// The IDAES module the member list was taken from, for the §6.14 parity enums.
    pub idaes_source: Option<&'static str>,
    /// The members, in declaration order. The ordinal is the position and is presentation
    /// only: a dictionary code never carries identity (blueprint §4.5).
    pub members: Vec<EnumMember>,
}

/// One member of an [`EnumSpec`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumMember {
    /// The platform spelling, which is the stored dictionary value.
    pub name: &'static str,
    /// The IDAES spelling, when the two differ (blueprint §6.14).
    pub idaes_name: Option<&'static str>,
    /// Declared but no longer selectable. Kept so old artifacts still decode.
    pub deprecated: bool,
    /// What the member means.
    pub doc: &'static str,
}

impl EnumMember {
    /// A live member whose platform and IDAES spellings agree, or that has no IDAES
    /// counterpart.
    pub const fn new(name: &'static str, doc: &'static str) -> Self {
        Self {
            name,
            idaes_name: None,
            deprecated: false,
            doc,
        }
    }

    /// A live member with a differing IDAES spelling (blueprint §6.14).
    pub const fn idaes(name: &'static str, idaes_name: &'static str, doc: &'static str) -> Self {
        Self {
            name,
            idaes_name: Some(idaes_name),
            deprecated: false,
            doc,
        }
    }
}

/// An [`EnumSpec`] before assembly, when it has no identity yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumDecl {
    /// See [`EnumSpec::name`].
    pub name: &'static str,
    /// See [`EnumSpec::idaes_source`].
    pub idaes_source: Option<&'static str>,
    /// See [`EnumSpec::members`].
    pub members: Vec<EnumMember>,
}

impl EnumDecl {
    /// A platform vocabulary with no IDAES counterpart.
    pub fn platform(name: &'static str, members: Vec<EnumMember>) -> Self {
        Self {
            name,
            idaes_source: None,
            members,
        }
    }

    /// An enumeration preserved from IDAES by name (blueprint §6.14).
    pub fn idaes(name: &'static str, source: &'static str, members: Vec<EnumMember>) -> Self {
        Self {
            name,
            idaes_source: Some(source),
            members,
        }
    }
}
