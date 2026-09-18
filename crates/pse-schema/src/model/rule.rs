// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Rule specifications: inference written as a typed relational plan, never as a loop
//! over rows (blueprint §6.11 `reference.rule_*`, §14.2).
//!
//! Native SQL is the executable declaration. DataFusion owns its expression and
//! relational algebra; this module declares only domain outcomes and input scopes.

use core::fmt;

use pse_ids::SemanticId;

use crate::model::enums::Namespace;

/// How a rule may use negation (blueprint §14.2 rule 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NegationPolicy {
    /// The rule contains no negation.
    None,
    /// The rule negates only relations fully computed in a lower stratum.
    Stratified,
}

impl NegationPolicy {
    /// Both policies.
    pub const ALL: [Self; 2] = [Self::None, Self::Stratified];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Stratified => "stratified",
        }
    }
}

impl fmt::Display for NegationPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What happens when two rules assert incompatible values for one key
/// (blueprint §14.2 rule 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConflictPolicy {
    /// The conflict is an error; the executor detects it after the stratum's fixed point.
    Reject,
    /// The key goes to `inferred.undecided` with `truth = conflict` and a diagnostic. The
    /// head relation holds only decided-true rows either way (§7.6).
    Undecided,
}

impl ConflictPolicy {
    /// Both policies.
    pub const ALL: [Self; 2] = [Self::Reject, Self::Undecided];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Undecided => "undecided",
        }
    }
}

impl fmt::Display for ConflictPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a rule touches a relation (blueprint §6.11 `reference.rule_dependencies`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyMode {
    /// The rule scans it.
    Read,
    /// The rule anti-joins against it, which is what makes stratification a requirement.
    Negate,
    /// The rule's head writes it.
    Write,
}

impl DependencyMode {
    /// Every mode, in blueprint §6.11 order.
    pub const ALL: [Self; 3] = [Self::Read, Self::Negate, Self::Write];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Negate => "negate",
            Self::Write => "write",
        }
    }
}

impl fmt::Display for DependencyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One exact semantic input scope used by a native query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleInput {
    /// Qualified relation selected from the provider hierarchy.
    pub relation: String,
    /// Stable domain input name, also used by source support.
    pub port: &'static str,
    /// Read or stratified absence dependency.
    pub mode: DependencyMode,
}

/// Native query producing one four-valued assertion outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleQuery {
    /// One of true, false, or unknown; conflict is derived from assertions.
    pub truth: &'static str,
    /// DataFusion SQL. All expressions and operators are native.
    pub sql: String,
}

/// A native rule plus its finite-domain policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSpec {
    /// Exact versioned registry identity.
    pub id: SemanticId,
    /// Semantic rule name.
    pub name: String,
    /// Declaration version.
    pub version: &'static str,
    /// Ordered fixed-point stratum.
    pub stratum: u16,
    /// Qualified output relation.
    pub head: String,
    /// Relation retaining competing typed assertions.
    pub assertion_relation: Option<String>,
    /// Native assertion queries.
    pub queries: Vec<RuleQuery>,
    /// Explicit input scopes, checked against the native plan during binding.
    pub inputs: Vec<RuleInput>,
    /// Negation policy.
    pub negation: NegationPolicy,
    /// Whether fixed-point evaluation may only add facts.
    pub monotonic: bool,
    /// Conflicting assertion handling.
    pub conflict_policy: ConflictPolicy,
}
impl RuleSpec {
    /// Exact declaration name.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// A native rule before registry identity assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDecl {
    /// Semantic name.
    pub name: String,
    /// Declaration version.
    pub version: &'static str,
    /// Fixed-point stratum.
    pub stratum: u16,
    /// Qualified head relation.
    pub head: String,
    /// Typed assertion relation.
    pub assertion_relation: Option<String>,
    /// Native query programs.
    pub queries: Vec<RuleQuery>,
    /// Input scopes.
    pub inputs: Vec<RuleInput>,
    /// Negation policy.
    pub negation: NegationPolicy,
    /// Positive fixed-point policy.
    pub monotonic: bool,
    /// Conflict handling.
    pub conflict_policy: ConflictPolicy,
}
impl RuleDecl {
    /// Declare one native query producing true assertions.
    pub fn new(
        name: impl Into<String>,
        version: &'static str,
        stratum: u16,
        head: impl Into<String>,
        sql: impl Into<String>,
        inputs: Vec<RuleInput>,
    ) -> Self {
        Self {
            name: name.into(),
            version,
            stratum,
            head: head.into(),
            assertion_relation: None,
            queries: vec![RuleQuery {
                truth: "true",
                sql: sql.into(),
            }],
            inputs,
            negation: NegationPolicy::None,
            monotonic: true,
            conflict_policy: ConflictPolicy::Reject,
        }
    }
    /// Retain competing typed payloads in the declared assertion relation.
    #[must_use]
    pub fn assertions(mut self, relation: impl Into<String>) -> Self {
        self.assertion_relation = Some(relation.into());
        self
    }
    /// Declare lower-stratum negation.
    #[must_use]
    pub const fn stratified_negation(mut self) -> Self {
        self.negation = NegationPolicy::Stratified;
        self
    }
    /// Declare conflict handling.
    #[must_use]
    pub const fn conflicts(mut self, policy: ConflictPolicy) -> Self {
        self.conflict_policy = policy;
        self
    }
}

/// One derived `reference.rule_dependencies` fact (blueprint §6.11).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleDependency {
    /// The exact versioned rule identity, never an ambiguous bare name.
    pub rule_id: SemanticId,
    /// The qualified relation.
    pub relation: String,
    /// The resolved relation version bound by this registry.
    pub relation_id: SemanticId,
    /// The input port the read or negation is bound to; absent for a write.
    pub input_port: Option<&'static str>,
    /// How the rule touches it.
    pub mode: DependencyMode,
    /// The rule's stratum, carried so a stratification check is one scan of this relation.
    pub stratum: u16,
}

/// The namespaces a compiler output port may never target (blueprint §14.1).
///
/// P3 onward cannot write `authored` or `reference`: the only write path into `authored`
/// is a change set (§22.2, decision D2), and a pass that could write one would make
/// "author causes, derive consequences" unenforceable.
pub const NON_DERIVABLE_NAMESPACES: [Namespace; 2] = [Namespace::Authored, Namespace::Reference];

/// Original key columns become payload; their complete types and physical contracts stay.
pub fn assertion_columns(head: &[crate::model::FieldContract]) -> Vec<crate::model::FieldContract> {
    use crate::model::{ColumnRole, FieldContract};
    let mut columns = vec![
        FieldContract::key(
            "assertion_id",
            FieldContract::id(),
            "Finite assertion identity",
        ),
        FieldContract::reference("rule_id", FieldContract::id(), "Versioned producer rule"),
        FieldContract::payload(
            "truth",
            FieldContract::enumeration("TruthValue"),
            "Candidate truth",
        ),
    ];
    columns.extend(
        head.iter()
            .filter(|column| column.role() != ColumnRole::Provenance)
            .cloned()
            .map(|mut column| {
                if column.role() == ColumnRole::Key {
                    column = column.with_role(ColumnRole::Payload);
                }
                column
            }),
    );
    columns
}
