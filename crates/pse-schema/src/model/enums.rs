// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The closed vocabularies a `RelationSpec` is written in, and the declaration form of a
//! `pse.enum` (blueprint §4.1, §6.14).
//!
//! Two different things are called an "enum" here and they are deliberately separate. The
//! platform vocabularies ([`Namespace`], [`Authority`], …) are the *model's* vocabulary:
//! they decide what a declaration may say, and `pse-vocabulary` owns them (ADR-0117). An
//! [`EnumSpec`] is a *declared* closed dictionary that becomes `reference.schema_enums`
//! rows and a `pse.enum` column contract. The platform vocabularies are also declared as
//! `EnumSpec`s — in `catalog::enums_platform` — so that the registry can describe itself
//! (§4.1), and that is the only place the two meet.

use pse_ids::SemanticId;

pub use pse_vocabulary::{
    Authority, ColumnRole, DerivationGranularity, InvariantKind, Namespace, Severity,
    SnapshotClass, Stability,
};

/// A declared string enumeration: one `pse.enum` contract and its
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
    /// The platform spelling, which is the stored string value.
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
