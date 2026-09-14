// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Relational invariants: the commit contract, declared
//! (blueprint §4.1 `reference.schema_invariants`, §22.2).
//!
//! An invariant is a rule, not a message. `rule` names a declared [`crate::model::RuleSpec`]
//! whose head is the violating keys, so P2 runs the same relational plan the registry
//! declared rather than a Rust loop that agrees with it by inspection.

use pse_ids::SemanticId;

use crate::model::enums::{InvariantKind, Severity};

/// A declared invariant (blueprint §4.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantSpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "invariant:<relation>:<name>")` (ADR-0050).
    pub id: SemanticId,
    /// The invariant's name inside its relation, for example `entity_registered`.
    pub name: String,
    /// The qualified relation the invariant constrains, for example `authored.entities`.
    pub relation: String,
    /// What kind of statement it makes.
    pub kind: InvariantKind,
    /// The declared rule whose head is the violating keys, as `<name>@<version>`.
    pub rule: String,
    /// Whether a violation stops a commit.
    pub severity: Severity,
    /// What the invariant means and why it holds.
    pub doc: &'static str,
}

impl InvariantSpec {
    /// The qualified invariant name, `<relation>:<name>`.
    pub fn qualified_name(&self) -> String {
        format!("{}:{}", self.relation, self.name)
    }
}

/// An [`InvariantSpec`] before assembly, when it has no identity yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantDecl {
    /// See [`InvariantSpec::name`].
    pub name: String,
    /// See [`InvariantSpec::relation`].
    pub relation: String,
    /// See [`InvariantSpec::kind`].
    pub kind: InvariantKind,
    /// See [`InvariantSpec::rule`].
    pub rule: String,
    /// See [`InvariantSpec::severity`].
    pub severity: Severity,
    /// See [`InvariantSpec::doc`].
    pub doc: &'static str,
}

impl InvariantDecl {
    /// An invariant that stops a commit when it is violated.
    pub fn error(
        relation: impl Into<String>,
        name: impl Into<String>,
        kind: InvariantKind,
        rule: impl Into<String>,
        doc: &'static str,
    ) -> Self {
        Self {
            name: name.into(),
            relation: relation.into(),
            kind,
            rule: rule.into(),
            severity: Severity::Error,
            doc,
        }
    }

    /// The same invariant, reported and not enforced.
    #[must_use]
    pub const fn warning(mut self) -> Self {
        self.severity = Severity::Warning;
        self
    }
}
