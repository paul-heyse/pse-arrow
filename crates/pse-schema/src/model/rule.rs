// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Rule specifications: inference written as a typed relational plan, never as a loop
//! over rows (blueprint §6.11 `reference.rule_*`, §14.2).
//!
//! A rule body is a bounded algebra so that it can be compiled to a DataFusion
//! `LogicalPlan`, optimized and executed, *and* stored as relations the platform can
//! query. A Rust loop over rows would be neither.

use core::fmt;

use pse_ids::SemanticId;

use crate::model::enums::Namespace;
use crate::model::rule_expr::RuleExpr;

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

/// Whether a join treats two nulls as equal (blueprint §14.2 rule 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NullEquality {
    /// SQL equality: a null matches nothing, including another null.
    NullEqualsNothing,
    /// `IS NOT DISTINCT FROM` equality: two nulls match.
    NullEqualsNull,
}

impl NullEquality {
    /// Both semantics.
    pub const ALL: [Self; 2] = [Self::NullEqualsNothing, Self::NullEqualsNull];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NullEqualsNothing => "null_equals_nothing",
            Self::NullEqualsNull => "null_equals_null",
        }
    }
}

impl fmt::Display for NullEquality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What an aggregate does with a missing input (blueprint §6.11 `rule_aggregates`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AggregateNullPolicy {
    /// A null input is an error: the caller declared the input total.
    Reject,
    /// A null input is skipped and counted.
    SkipMissing,
}

impl AggregateNullPolicy {
    /// Both policies.
    pub const ALL: [Self; 2] = [Self::Reject, Self::SkipMissing];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::SkipMissing => "skip_missing",
        }
    }
}

impl fmt::Display for AggregateNullPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What an aggregate produces for an empty group (blueprint §6.11 `rule_aggregates`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AggregateEmptyPolicy {
    /// Zero.
    Zero,
    /// An empty list.
    EmptyList,
    /// An error: an empty group means the input was incomplete.
    Error,
}

impl AggregateEmptyPolicy {
    /// Every policy, in blueprint §6.11 order.
    pub const ALL: [Self; 3] = [Self::Zero, Self::EmptyList, Self::Error];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::EmptyList => "empty_list",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for AggregateEmptyPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What unnest does with a null list (blueprint §6.11 `rule_unnest`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NullListPolicy {
    /// A null list is an error.
    Reject,
    /// A null list contributes no members.
    NoMembers,
}

impl NullListPolicy {
    /// Both policies.
    pub const ALL: [Self; 2] = [Self::Reject, Self::NoMembers];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::NoMembers => "no_members",
        }
    }
}

impl fmt::Display for NullListPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What unnest does with an empty list (blueprint §6.11 `rule_unnest`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EmptyListPolicy {
    /// An empty list contributes no members. There is no other option: an empty list is a
    /// stated fact, not a missing one.
    NoMembers,
}

impl EmptyListPolicy {
    /// The only policy.
    pub const ALL: [Self; 1] = [Self::NoMembers];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoMembers => "no_members",
        }
    }
}

impl fmt::Display for EmptyListPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The aggregate functions a rule plan may use (blueprint §6.11 `rule_aggregates`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleAggregateFn {
    /// Row count.
    Count,
    /// Exact checked integer sum. Floating reductions need a numerical policy (§14.2
    /// rule 9) and are not this.
    Sum,
    /// Minimum.
    Min,
    /// Maximum.
    Max,
    /// Collection in the declared order, never physical arrival order.
    CollectOrdered,
}

impl RuleAggregateFn {
    /// Every function, in blueprint §6.11 order.
    pub const ALL: [Self; 5] = [
        Self::Count,
        Self::Sum,
        Self::Min,
        Self::Max,
        Self::CollectOrdered,
    ];

    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Sum => "sum",
            Self::Min => "min",
            Self::Max => "max",
            Self::CollectOrdered => "collect_ordered",
        }
    }
}

impl fmt::Display for RuleAggregateFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How far a recursive plan may iterate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DepthBound {
    /// Iterate to the least fixed point. Termination comes from the deduplication, not
    /// from a counter.
    FixedPoint,
    /// Stop after this many iterations and report; a closure that has not settled is a
    /// finding, not a hang.
    Bounded(u32),
}

impl DepthBound {
    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixedPoint => "fixed_point",
            Self::Bounded(_) => "bounded",
        }
    }

    /// The bound, when there is one.
    pub const fn limit(self) -> Option<u32> {
        match self {
            Self::FixedPoint => None,
            Self::Bounded(limit) => Some(limit),
        }
    }
}

/// One aggregate of an [`RulePlan::Aggregate`] node (blueprint §6.11).
#[derive(Clone, Debug, PartialEq)]
pub struct RuleAggregate {
    /// The function.
    pub function: RuleAggregateFn,
    /// The input expression. `None` for [`RuleAggregateFn::Count`] over rows.
    pub input: Option<RuleExpr>,
    /// The output column name.
    pub output_name: &'static str,
    /// The order the aggregate consumes its input in, as `(column, ascending)`. Required
    /// for [`RuleAggregateFn::CollectOrdered`]: physical arrival order is not an order.
    pub order_by: Vec<(&'static str, bool)>,
    /// What to do with a null input.
    pub null_policy: AggregateNullPolicy,
    /// What to produce for an empty group.
    pub empty_policy: AggregateEmptyPolicy,
}

/// The bounded relational algebra a rule body is written in (blueprint §14.2).
#[derive(Clone, Debug, PartialEq)]
pub enum RulePlan {
    /// Read a relation through a named input port. The port, not an implicit latest
    /// producer, is the binding (§6.11).
    Scan {
        /// The qualified relation.
        relation: &'static str,
        /// The `pass_input_ports` port this scan binds to.
        port: &'static str,
    },
    /// Keep the rows where `predicate` is true.
    Filter {
        /// The input plan.
        input: Box<RulePlan>,
        /// The Kleene predicate.
        predicate: RuleExpr,
    },
    /// Compute named output columns.
    Project {
        /// The input plan.
        input: Box<RulePlan>,
        /// The output columns, as `(name, expression)`.
        columns: Vec<(&'static str, RuleExpr)>,
    },
    /// An inner join on equal keys.
    EquiJoin {
        /// The left input.
        left: Box<RulePlan>,
        /// The right input.
        right: Box<RulePlan>,
        /// The key pairs, as `(left column, right column)`.
        keys: Vec<(&'static str, &'static str)>,
        /// Whether two nulls match.
        null_equality: NullEquality,
    },
    /// The left rows with no matching right row: stratified negation.
    AntiJoin {
        /// The left input.
        left: Box<RulePlan>,
        /// The negated input.
        right: Box<RulePlan>,
        /// The key pairs, as `(left column, right column)`.
        keys: Vec<(&'static str, &'static str)>,
    },
    /// The union of several inputs, without deduplication.
    Union(Vec<RulePlan>),
    /// Deduplication by the full row.
    Distinct(Box<RulePlan>),
    /// Grouping.
    Aggregate {
        /// The input plan.
        input: Box<RulePlan>,
        /// The group columns.
        group: Vec<&'static str>,
        /// The aggregates.
        aggregates: Vec<RuleAggregate>,
    },
    /// Expand one declared list column, retaining the parent key.
    Unnest {
        /// The input plan.
        input: Box<RulePlan>,
        /// The list column to expand.
        column: &'static str,
        /// The name the element takes.
        value_name: &'static str,
        /// What to do with a null list.
        null_list: NullListPolicy,
        /// What to do with an empty list.
        empty_list: EmptyListPolicy,
    },
    /// A fixed-point closure: seed, then repeat `step` until nothing new appears.
    Recursive {
        /// The name `step` refers to the accumulated relation by.
        name: &'static str,
        /// The seed plan.
        seed: Box<RulePlan>,
        /// The step plan.
        step: Box<RulePlan>,
        /// Whether the accumulation deduplicates.
        is_distinct: bool,
        /// How far it may iterate.
        depth_bound: DepthBound,
    },
}

impl RulePlan {
    /// The `RulePlanOp` enumeration member this node is.
    pub const fn op(&self) -> &'static str {
        match self {
            Self::Scan { .. } => "scan",
            Self::Filter { .. } => "filter",
            Self::Project { .. } => "project",
            Self::EquiJoin { .. } => "equi_join",
            Self::AntiJoin { .. } => "anti_join",
            Self::Union(_) => "union",
            Self::Distinct(_) => "distinct",
            Self::Aggregate { .. } => "aggregate",
            Self::Unnest { .. } => "unnest",
            Self::Recursive { .. } => "recursive",
        }
    }

    /// This node's children, in the order the `rule_plan_edges` ordinals follow.
    pub fn children(&self) -> Vec<&RulePlan> {
        match self {
            Self::Scan { .. } => Vec::new(),
            Self::Filter { input, .. }
            | Self::Project { input, .. }
            | Self::Aggregate { input, .. }
            | Self::Unnest { input, .. }
            | Self::Distinct(input) => vec![input.as_ref()],
            Self::EquiJoin { left, right, .. } | Self::AntiJoin { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            Self::Union(inputs) => inputs.iter().collect(),
            Self::Recursive { seed, step, .. } => vec![seed.as_ref(), step.as_ref()],
        }
    }

    /// Every `(relation, port, mode)` this plan touches, in preorder.
    ///
    /// A scan under an [`RulePlan::AntiJoin`]'s right input is a
    /// [`DependencyMode::Negate`]; everywhere else it is a [`DependencyMode::Read`]. That
    /// distinction is what stratification is checked against.
    pub fn dependencies(&self) -> Vec<(&'static str, &'static str, DependencyMode)> {
        let mut out = Vec::new();
        self.collect_dependencies(DependencyMode::Read, &mut out);
        out
    }

    /// Appends this plan's dependencies to `out` under `mode`.
    fn collect_dependencies(
        &self,
        mode: DependencyMode,
        out: &mut Vec<(&'static str, &'static str, DependencyMode)>,
    ) {
        match self {
            Self::Scan { relation, port } => out.push((relation, port, mode)),
            Self::AntiJoin { left, right, .. } => {
                left.collect_dependencies(mode, out);
                right.collect_dependencies(DependencyMode::Negate, out);
            }
            other => {
                for child in other.children() {
                    child.collect_dependencies(mode, out);
                }
            }
        }
    }
}

/// What a rule writes (blueprint §14.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleHead {
    /// Rows of a declared relation.
    Relation(&'static str),
    /// The violating keys of an invariant, which is what a `reference.schema_invariants`
    /// row points at.
    Violations {
        /// The qualified relation the invariant constrains.
        of: &'static str,
        /// The key columns the violations are reported by.
        key_columns: Vec<&'static str>,
    },
}

impl RuleHead {
    /// The qualified relation this head reads or writes.
    pub const fn relation(&self) -> &'static str {
        match self {
            Self::Relation(relation) | Self::Violations { of: relation, .. } => relation,
        }
    }
}

/// A declared rule (blueprint §6.11 `reference.rule_specs`, §14.2).
#[derive(Clone, Debug, PartialEq)]
pub struct RuleSpec {
    /// `named_id(REGISTRY_PACKAGE_ID, "rule:<name>@<v>")` (ADR-0050).
    pub id: SemanticId,
    /// The rule name, for example `phase_species_valid`.
    pub name: &'static str,
    /// The rule version.
    pub version: &'static str,
    /// The stratum. Strata run in order; within one, rules reach a least fixed point.
    pub stratum: u16,
    /// What the rule writes.
    pub head: RuleHead,
    /// The body.
    pub plan: RulePlan,
    /// How the rule may negate.
    pub negation: NegationPolicy,
    /// Whether the rule only ever adds rows.
    pub monotonic: bool,
    /// What happens on a conflict.
    pub conflict_policy: ConflictPolicy,
}

impl RuleSpec {
    /// The rule's registry name, `<name>@<version>`.
    pub fn qualified_name(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// A [`RuleSpec`] before assembly, when it has no identity yet.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleDecl {
    /// See [`RuleSpec::name`].
    pub name: &'static str,
    /// See [`RuleSpec::version`].
    pub version: &'static str,
    /// See [`RuleSpec::stratum`].
    pub stratum: u16,
    /// See [`RuleSpec::head`].
    pub head: RuleHead,
    /// See [`RuleSpec::plan`].
    pub plan: RulePlan,
    /// See [`RuleSpec::negation`].
    pub negation: NegationPolicy,
    /// See [`RuleSpec::monotonic`].
    pub monotonic: bool,
    /// See [`RuleSpec::conflict_policy`].
    pub conflict_policy: ConflictPolicy,
}

impl RuleDecl {
    /// A monotonic, non-negating rule that rejects conflicts.
    pub const fn new(
        name: &'static str,
        version: &'static str,
        stratum: u16,
        head: RuleHead,
        plan: RulePlan,
    ) -> Self {
        Self {
            name,
            version,
            stratum,
            head,
            plan,
            negation: NegationPolicy::None,
            monotonic: true,
            conflict_policy: ConflictPolicy::Reject,
        }
    }

    /// The same rule, declared to use stratified negation.
    #[must_use]
    pub const fn stratified_negation(mut self) -> Self {
        self.negation = NegationPolicy::Stratified;
        self
    }

    /// The same rule, under a different conflict policy.
    #[must_use]
    pub const fn conflicts(mut self, policy: ConflictPolicy) -> Self {
        self.conflict_policy = policy;
        self
    }
}

/// One derived `reference.rule_dependencies` fact (blueprint §6.11).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleDependency {
    /// The rule, as `<name>@<version>`.
    pub rule: &'static str,
    /// The qualified relation.
    pub relation: &'static str,
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
