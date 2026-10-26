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
    /// The Rust path of the hand-written type that owns this vocabulary, such as
    /// `pse_vocabulary::Namespace`. The generator re-exports that type instead of emitting
    /// a second enum (ADR-0115 Outcome 3). It is a generation fact, not part of the
    /// registry's self-description, so it does not enter the fingerprint.
    pub source: Option<&'static str>,
    /// The member used by Rust's derived `Default`, if explicitly declared. Like
    /// `source`, this is generation metadata and does not enter the fingerprint.
    pub default_member: Option<&'static str>,
    /// Declared lifecycle edges. Reflexive transitions are always admitted.
    pub transitions: Vec<EnumTransition>,
    /// Total projections into other declared closed vocabularies.
    pub projections: Vec<EnumProjection>,
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

/// One directed lifecycle edge, stated in the owning vocabulary's spellings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumTransition {
    /// Original member.
    pub before: &'static str,
    /// New member.
    pub after: &'static str,
}

/// One total closed-vocabulary projection. A boolean input represents an additional
/// fact admitted by the scientific owner; storage never derives that fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumProjection {
    /// Generated method name.
    pub method: &'static str,
    /// Target declared vocabulary.
    pub target: &'static str,
    /// Optional boolean input name.
    pub input: Option<&'static str>,
    /// Exactly one mapping for every source member.
    pub members: Vec<EnumProjectionMember>,
}

/// The result for one source member, with both values explicit for boolean projections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumProjectionMember {
    /// Member of the source vocabulary.
    pub source: &'static str,
    /// Target without an input, or when the input is false.
    pub value: &'static str,
    /// Target when the input is true; present exactly when the projection has an input.
    pub when_true: Option<&'static str>,
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
    /// See [`EnumSpec::source`].
    pub source: Option<&'static str>,
    /// See [`EnumSpec::default_member`].
    pub default_member: Option<&'static str>,
    /// See [`EnumSpec::transitions`].
    pub transitions: Vec<EnumTransition>,
    /// See [`EnumSpec::projections`].
    pub projections: Vec<EnumProjection>,
    /// See [`EnumSpec::members`].
    pub members: Vec<EnumMember>,
}

impl EnumDecl {
    /// A platform vocabulary with no IDAES counterpart.
    pub fn platform(name: &'static str, members: Vec<EnumMember>) -> Self {
        Self {
            name,
            idaes_source: None,
            source: None,
            default_member: None,
            transitions: Vec::new(),
            projections: Vec::new(),
            members,
        }
    }

    /// A platform vocabulary whose values are the hand-written Rust type at `path`, such
    /// as `pse_vocabulary::Namespace`; the registry name is the path's last segment. The
    /// members are the type's `ALL` spellings, stated here so the registry describes them
    /// (ADR-0115 Outcome 3, ADR-0117).
    pub fn sourced(path: &'static str, members: Vec<EnumMember>) -> Self {
        Self {
            name: path.rsplit_once("::").map_or(path, |(_, name)| name),
            idaes_source: None,
            source: Some(path),
            default_member: None,
            transitions: Vec::new(),
            projections: Vec::new(),
            members,
        }
    }

    /// An enumeration preserved from IDAES by name (blueprint §6.14).
    pub fn idaes(name: &'static str, source: &'static str, members: Vec<EnumMember>) -> Self {
        Self {
            name,
            idaes_source: Some(source),
            source: None,
            default_member: None,
            transitions: Vec::new(),
            projections: Vec::new(),
            members,
        }
    }

    /// Derive Rust's `Default` from one explicitly named member. Sourced types own
    /// their own defaults and cannot use this generation option.
    pub const fn with_default(mut self, member: &'static str) -> Self {
        self.default_member = Some(member);
        self
    }

    /// Declare the allowed nonreflexive lifecycle transitions.
    pub fn with_transitions(mut self, transitions: &[(&'static str, &'static str)]) -> Self {
        self.transitions = transitions.iter().map(|&(before, after)| EnumTransition { before, after }).collect();
        self
    }

    /// Add a total boolean observation over this vocabulary's declared members.
    pub fn with_predicate(self, method: &'static str, selected: &[&'static str]) -> Self {
        let mut members = self.members.iter().map(|member| EnumProjectionMember {
            source: member.name, value: if selected.contains(&member.name) { "true" } else { "false" }, when_true: None,
        }).collect::<Vec<_>>();
        members.extend(selected.iter().filter(|selected| !self.members.iter().any(|member| member.name == **selected)).map(|&source| EnumProjectionMember { source, value: "true", when_true: None }));
        self.with_projection(EnumProjection { method, target: "bool", input: None, members })
    }

    /// Add a total typed projection; assembly validates both vocabularies and coverage.
    pub fn with_projection(mut self, projection: EnumProjection) -> Self {
        self.projections.push(projection);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RegistryBuilder;

    fn projection(members: Vec<EnumProjectionMember>) -> EnumDecl {
        EnumDecl::platform("Outcome", vec![EnumMember::new("ok", "ok"), EnumMember::new("partial", "partial")])
            .with_projection(EnumProjection { method: "state", target: "State", input: Some("usable"), members })
    }
    fn assemble(declaration: EnumDecl) -> Result<crate::Registry, crate::SchemaError> {
        let mut builder = RegistryBuilder::new();
        builder.declare_enum(EnumDecl::platform("State", vec![EnumMember::new("done", "done"), EnumMember::new("failed", "failed")]))
            .declare_enum(declaration);
        builder.build()
    }
    #[test]
    fn canonical_projection_rejects_omitted_and_unknown_members() {
        let ok = EnumProjectionMember { source: "ok", value: "done", when_true: Some("done") };
        assert!(assemble(projection(vec![ok.clone()])).is_err());
        assert!(assemble(projection(vec![ok.clone(), EnumProjectionMember { source: "partial", value: "failed", when_true: Some("unknown") }])).is_err());
        assemble(projection(vec![ok, EnumProjectionMember { source: "partial", value: "failed", when_true: Some("done") }])).unwrap();
    }
    #[test]
    fn canonical_lifecycle_edges_reject_unknown_and_duplicate_transitions() {
        let vocabulary = || EnumDecl::platform("Phase", vec![EnumMember::new("open", "open"), EnumMember::new("closed", "closed")]);
        assert!(assemble(vocabulary().with_transitions(&[("open", "missing")])).is_err());
        assert!(assemble(vocabulary().with_transitions(&[("open", "closed"), ("open", "closed")])).is_err());
        assert!(assemble(vocabulary().with_predicate("visible", &["missing"])).is_err());
        assemble(vocabulary().with_transitions(&[("open", "closed")])).unwrap();
    }
}
