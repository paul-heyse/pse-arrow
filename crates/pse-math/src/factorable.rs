// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Library-neutral factorable projection for global solvers; never an evaluator.
//!
//! Each instance's demanded stage program ([`PreparedBody`]) is projected into one shared
//! DAG. A stage result stays one node however often it is read, so the export never
//! depends on flattening a body into a single expression; large factorable bodies such as
//! Helmholtz derivatives stay exact. Nodes map one to one onto the factorable vocabulary of
//! global solvers: variable, value, sum, product, constant power, exp, log, abs, sin and
//! cos. Minimum, maximum and absolute-value branches are exported through the exact
//! absolute-value identity.
//!
//! Every row and the objective carry a [`Fidelity`]. `Require` and `Domain` stages are
//! obligations, never dependencies of a value: they become closed constraints, and a
//! validity predicate is decomposed into its closed conjunction where its structure allows.
//! Implicit blocks export their residual equations and declared bounds whatever their
//! realization. Other providers and unrepresentable branches become auxiliary variables
//! within the envelope their evaluation enforces, which makes the dependent rows `Relaxed`.
//! The evaluator and original-coordinate qualification remain the authority for every
//! candidate (ADR-0105).
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, ObjectiveSense, Target, VariableDomain},
    guarded::{Comparison, Condition, PreparedBody, Stage},
    library,
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, ProviderKey, ProviderSpec};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    coefficient::CoefficientView,
};

/// Index of a node in [`FactorableProgram::nodes`]; children always precede their parent.
pub type NodeId = usize;

/// Maximum symbolic nesting followed inside one stage expression.
const MAX_DEPTH: usize = 128;
/// Maximum nesting of implicit blocks exported through their residuals.
const MAX_NESTING: usize = 8;
/// Largest stage expression retained for exact branch identities.
const DEFINITION_BYTES: usize = 16 * 1024;
/// Candidate factors α of the absolute-value identity, as `(numerator, denominator)`.
const IDENTITY_FACTORS: [(i64, i64); 6] = [(1, 1), (-1, 1), (-2, 1), (2, 1), (1, 2), (-1, 2)];

/// Exact rational constant `numerator / denominator` with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rational {
    /// Signed numerator.
    pub numerator: i64,
    /// Positive denominator.
    pub denominator: i64,
}
impl Rational {
    /// Normalized rational; `None` for a zero denominator or an unrepresentable sign.
    pub fn new(numerator: i64, denominator: i64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let (mut n, mut d) = (numerator, denominator);
        if d < 0 {
            n = n.checked_neg()?;
            d = d.checked_neg()?;
        }
        let g = gcd(n.unsigned_abs(), d.unsigned_abs());
        let g = i64::try_from(g).ok()?;
        Some(Self {
            numerator: n / g,
            denominator: d / g,
        })
    }
    /// Nearest binary64 value.
    pub fn value(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
    fn add(self, other: Self) -> Option<Self> {
        let n = self
            .numerator
            .checked_mul(other.denominator)?
            .checked_add(other.numerator.checked_mul(self.denominator)?)?;
        Self::new(n, self.denominator.checked_mul(other.denominator)?)
    }
    fn mul(self, other: Self) -> Option<Self> {
        Self::new(
            self.numerator.checked_mul(other.numerator)?,
            self.denominator.checked_mul(other.denominator)?,
        )
    }
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// A constant, retained exactly when the library atom is rational.
#[derive(Clone, Copy, Debug)]
pub enum Constant {
    /// Exact rational from the library atom.
    Rational(Rational),
    /// Finite binary64 value; every finite binary64 value is itself a dyadic rational.
    Float(f64),
}
impl Constant {
    /// Binary64 value used by numerical consumers.
    pub fn value(self) -> f64 {
        match self {
            Self::Rational(r) => r.value(),
            Self::Float(v) => v,
        }
    }
    const fn integer(value: i64) -> Self {
        Self::Rational(Rational {
            numerator: value,
            denominator: 1,
        })
    }
    fn fold(
        values: &[Self],
        rational: fn(Rational, Rational) -> Option<Rational>,
        float: fn(f64, f64) -> f64,
        unit: i64,
    ) -> Option<Self> {
        let exact = values.iter().try_fold(
            Rational {
                numerator: unit,
                denominator: 1,
            },
            |acc, v| match v {
                Self::Rational(r) => rational(acc, *r),
                Self::Float(_) => None,
            },
        );
        match exact {
            Some(r) => Some(Self::Rational(r)),
            None => {
                let value = values
                    .iter()
                    .fold(unit as f64, |acc, v| float(acc, v.value()));
                value.is_finite().then_some(Self::Float(value))
            }
        }
    }
}
impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rational(a), Self::Rational(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}
impl Eq for Constant {}
impl Hash for Constant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Rational(r) => {
                0_u8.hash(state);
                r.hash(state);
            }
            Self::Float(v) => {
                1_u8.hash(state);
                v.to_bits().hash(state);
            }
        }
    }
}

/// One factorable operation. Children always precede their parent in the node list.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    /// Free case column, in [`CasePlan::columns`] order.
    Var(usize),
    /// Auxiliary variable, indexing [`FactorableProgram::auxiliaries`].
    Aux(usize),
    /// Constant value.
    Const(Constant),
    /// N-ary sum.
    Sum(Vec<NodeId>),
    /// N-ary product.
    Product(Vec<NodeId>),
    /// Power with a constant exponent (rational, or an exact binary64 value).
    Pow {
        /// Base operand.
        base: NodeId,
        /// Constant exponent.
        exponent: Constant,
    },
    /// Natural exponential.
    Exp(NodeId),
    /// Natural logarithm.
    Log(NodeId),
    /// Absolute value.
    Abs(NodeId),
    /// Sine.
    Sin(NodeId),
    /// Cosine.
    Cos(NodeId),
}

/// How faithfully an exported function represents the evaluator; ordered best to worst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fidelity {
    /// The evaluator's function on its admitted domain.
    Exact,
    /// Admits the evaluator's value for some auxiliary value: a sound relaxation that
    /// supports bounds and infeasibility conclusions only.
    Relaxed,
    /// Not projected; it yields no bound.
    Unavailable,
}

/// Why a value could not be exported exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opacity {
    /// A provider output without an exported definition, or a provider partial.
    Provider {
        /// Provider implementation identity.
        provider: SemanticId,
        /// Output ordinal.
        output: usize,
    },
    /// A branch result outside the exact identity, under the auxiliary branch policy.
    Branch,
    /// A library function outside the factorable vocabulary.
    Function(String),
    /// A constant outside finite real representation.
    Constant,
    /// The symbolic nesting bound of one stage expression.
    Depth,
}

/// Why an auxiliary variable exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuxiliaryRole {
    /// An implicit unknown determined by its exported residual equations.
    Implicit {
        /// Provider identity of the implicit block.
        provider: SemanticId,
        /// Unknown ordinal, equal to the provider output ordinal.
        unknown: usize,
    },
    /// A stand-in for an opaque value, bounded only by what its evaluation enforces.
    Opaque(Opacity),
}

/// One auxiliary variable with its closed box.
#[derive(Clone, Debug, PartialEq)]
pub struct Auxiliary {
    /// Instance whose evaluation owns the value.
    pub instance: SemanticId,
    /// Origin of the auxiliary.
    pub role: AuxiliaryRole,
    /// Closed lower bound; negative infinity when none is enforced.
    pub lower: f64,
    /// Closed upper bound; positive infinity when none is enforced.
    pub upper: f64,
    /// Exact for an implicit unknown with an exact residual export, otherwise relaxed.
    pub fidelity: Fidelity,
}

/// Closed constraint `lower <= expression <= upper`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constraint {
    /// Constrained node.
    pub expression: NodeId,
    /// Closed lower bound (possibly negative infinity).
    pub lower: f64,
    /// Closed upper bound (possibly positive infinity).
    pub upper: f64,
    /// The original condition excludes the finite bound; this constraint is its closure.
    pub strict: bool,
}

/// A free case column and its declared box.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedVariable {
    /// Case variable identity.
    pub id: SemanticId,
    /// Declared decision domain.
    pub domain: VariableDomain,
    /// Closed lower bound (zero for semicontinuous domains), or negative infinity.
    pub lower: f64,
    /// Closed upper bound, or positive infinity.
    pub upper: f64,
}

/// One selected constraint row.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedRow {
    /// Row identity.
    pub id: SemanticId,
    /// Row function; absent when `Unavailable`.
    pub expression: Option<NodeId>,
    /// Closed lower bound.
    pub lower: f64,
    /// Closed upper bound.
    pub upper: f64,
    /// Export fidelity.
    pub fidelity: Fidelity,
}

/// The selected objective.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedObjective {
    /// Objective function; absent when `Unavailable`.
    pub expression: Option<NodeId>,
    /// Authored orientation.
    pub sense: ObjectiveSense,
    /// `Unavailable` when it was not projected or depends on an unbounded opaque auxiliary.
    pub fidelity: Fidelity,
}

/// Kind of a retained obligation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObligationKind {
    /// A pre-normalization condition on one value.
    Require(Condition),
    /// A value-only domain predicate, such as a function's `valid(...)` guard.
    Domain,
}

/// Whether an obligation applies at every evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObligationScope {
    /// Enforced whenever the owning outputs are evaluated; its constraints belong to the
    /// exported program.
    Unconditional,
    /// Enforced only inside a branch region; recorded for domain analysis, never a
    /// constraint of the exported program.
    Conditional,
}

/// One retained obligation and its closed constraints.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedObligation {
    /// Bound instance.
    pub instance: SemanticId,
    /// Authored occurrence.
    pub source: SemanticId,
    /// Obligation kind.
    pub kind: ObligationKind,
    /// Scope of enforcement.
    pub scope: ObligationScope,
    /// Checked value of a `Require`.
    pub argument: Option<NodeId>,
    /// Closed constraints; their conjunction is the obligation's closure when `represented`.
    pub constraints: Vec<Constraint>,
    /// Every part of the obligation is represented up to closure. A nonzero requirement's
    /// closure has no constraint; `argument` and `kind` keep its exact condition.
    pub represented: bool,
    /// `Exact` when represented through exact nodes, otherwise `Relaxed`.
    pub fidelity: Fidelity,
}

/// An implicit block exported through its residual equations.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedImplicit {
    /// Instance that evaluates the block.
    pub instance: SemanticId,
    /// Provider identity of the block.
    pub provider: SemanticId,
    /// Auxiliary indices of the unknowns, in provider output order.
    pub unknowns: Vec<usize>,
    /// Residual functions; each equals zero.
    pub residuals: Vec<NodeId>,
    /// Declared bounds that depend on the block inputs.
    pub bounds: Vec<Constraint>,
    /// Worst fidelity of its residuals and bounds.
    pub fidelity: Fidelity,
}

/// Declared export policy for a branch that the exact identity does not cover.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BranchPolicy {
    /// A free auxiliary, bounded by constant branch values when all of them are constant.
    #[default]
    Auxiliary,
    /// An exact disjunctive (mixed-integer) form for branches with proven continuity.
    Disjunctive,
}

/// An implicit block's original residual definition, independent of its realization.
#[derive(Clone, Debug)]
pub struct ImplicitDefinition {
    /// Residual body: the unknowns are its leading formals, the provider inputs follow in
    /// provider order, and its outputs are the residuals.
    pub residual: Arc<PreparedBody>,
    /// Declared closed interval of each unknown; infinite where none is declared.
    pub unknowns: Vec<(f64, f64)>,
    /// Bounds evaluated from the block inputs, as the evaluator enforces them.
    pub bounds: Option<ImplicitBounds>,
}

/// A bound program over the residual formals; unknown coordinates are not read.
#[derive(Clone, Debug)]
pub struct ImplicitBounds {
    /// Body over the same formal layout as the residual.
    pub body: Arc<PreparedBody>,
    /// Output ordinal of each unknown's lower bound.
    pub lower: Vec<Option<usize>>,
    /// Output ordinal of each unknown's upper bound.
    pub upper: Vec<Option<usize>>,
}

/// Owner-supplied definitions and policies for one projection.
#[derive(Clone, Debug, Default)]
pub struct FactorableRequest {
    /// Implicit blocks exported through their residuals, keyed by provider.
    pub implicit: BTreeMap<ProviderKey, ImplicitDefinition>,
    /// Closed output intervals that a provider's evaluation enforces.
    pub envelopes: BTreeMap<ProviderKey, Vec<(f64, f64)>>,
    /// Branch export policy.
    pub branches: BranchPolicy,
}

/// Which bound of a variable or auxiliary is missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundOwner {
    /// A free case variable.
    Variable(SemanticId),
    /// An auxiliary index.
    Auxiliary(usize),
}

/// A variable or auxiliary without a finite bound; spatial branch-and-bound needs both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingBound {
    /// Owner of the box.
    pub owner: BoundOwner,
    /// The lower bound is not finite.
    pub lower: bool,
    /// The upper bound is not finite.
    pub upper: bool,
}

/// Row counts per fidelity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FidelityCounts {
    /// Rows exported exactly.
    pub exact: usize,
    /// Rows exported as sound relaxations.
    pub relaxed: usize,
    /// Rows not projected.
    pub unavailable: usize,
}

/// Immutable factorable projection of one case under fixed consumed values.
#[derive(Clone, Debug)]
pub struct FactorableProgram {
    /// Structure, consumed values and request identity.
    pub key: ContentHash,
    /// Original case layout.
    pub structure: ContentHash,
    /// Values whose change invalidates this projection.
    pub values: BTreeMap<SemanticId, u64>,
    /// Shared DAG; children precede their parents.
    pub nodes: Vec<Node>,
    /// Free columns in [`CasePlan::columns`] order.
    pub variables: Vec<ProjectedVariable>,
    /// Auxiliary variables.
    pub auxiliaries: Vec<Auxiliary>,
    /// Every selected row, in case order.
    pub rows: Vec<ProjectedRow>,
    /// The selected objective, when one is declared.
    pub objective: Option<ProjectedObjective>,
    /// Retained obligations of every projected instance.
    pub obligations: Vec<ProjectedObligation>,
    /// Implicit blocks exported through their residuals.
    pub implicit: Vec<ProjectedImplicit>,
    /// Instances whose projection exhausted the node budget.
    pub incomplete: Vec<SemanticId>,
    fidelity: Vec<Fidelity>,
}
impl FactorableProgram {
    /// Refuse stale numeric assumptions without including free trial values.
    pub fn matches(&self, plan: &CasePlan, values: &CaseValues) -> bool {
        self.structure == plan.structure().key()
            && self
                .values
                .iter()
                .all(|(id, b)| values.scalars.get(id).is_some_and(|v| v.to_bits() == *b))
    }
    /// Fidelity of one node: the worst auxiliary it depends on.
    pub fn node_fidelity(&self, node: NodeId) -> Option<Fidelity> {
        self.fidelity.get(node).copied()
    }
    /// Worst fidelity over the rows, objective, unconditional obligations and implicit blocks.
    pub fn fidelity(&self) -> Fidelity {
        self.rows
            .iter()
            .map(|r| r.fidelity)
            .chain(self.objective.iter().map(|o| o.fidelity))
            .chain(
                self.obligations
                    .iter()
                    .filter(|o| o.scope == ObligationScope::Unconditional)
                    .map(|o| o.fidelity),
            )
            .chain(self.implicit.iter().map(|i| i.fidelity))
            .chain((!self.incomplete.is_empty()).then_some(Fidelity::Unavailable))
            .max()
            .unwrap_or(Fidelity::Exact)
    }
    /// Row counts per fidelity.
    pub fn row_counts(&self) -> FidelityCounts {
        let mut counts = FidelityCounts::default();
        for row in &self.rows {
            match row.fidelity {
                Fidelity::Exact => counts.exact += 1,
                Fidelity::Relaxed => counts.relaxed += 1,
                Fidelity::Unavailable => counts.unavailable += 1,
            }
        }
        counts
    }
    /// Variables and auxiliaries without a finite closed box.
    pub fn missing_bounds(&self) -> Vec<MissingBound> {
        let missing = |owner, lower: f64, upper: f64| {
            (!lower.is_finite() || !upper.is_finite()).then_some(MissingBound {
                owner,
                lower: !lower.is_finite(),
                upper: !upper.is_finite(),
            })
        };
        self.variables
            .iter()
            .filter_map(|v| missing(BoundOwner::Variable(v.id), v.lower, v.upper))
            .chain(
                self.auxiliaries
                    .iter()
                    .enumerate()
                    .filter_map(|(i, a)| missing(BoundOwner::Auxiliary(i), a.lower, a.upper)),
            )
            .collect()
    }
    /// Evaluate every node at free-column values and auxiliary values, in node order.
    /// This is export readback, never the model evaluator.
    /// # Errors
    /// Coordinate vectors of the wrong length.
    pub fn evaluate(&self, point: &[f64], auxiliary: &[f64]) -> Result<Vec<f64>, MathError> {
        if point.len() != self.variables.len() || auxiliary.len() != self.auxiliaries.len() {
            return Err(MathError::Contract(
                "factorable readback coordinates".into(),
            ));
        }
        let mut values: Vec<f64> = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let value = match node {
                Node::Var(c) => point[*c],
                Node::Aux(k) => auxiliary[*k],
                Node::Const(c) => c.value(),
                Node::Sum(children) => children.iter().map(|&i| values[i]).sum(),
                Node::Product(children) => children.iter().map(|&i| values[i]).product(),
                Node::Pow { base, exponent } => power(values[*base], *exponent),
                Node::Exp(i) => values[*i].exp(),
                Node::Log(i) => values[*i].ln(),
                Node::Abs(i) => values[*i].abs(),
                Node::Sin(i) => values[*i].sin(),
                Node::Cos(i) => values[*i].cos(),
            };
            values.push(value);
        }
        Ok(values)
    }
    /// Bounded accounting for the principal owned allocations.
    pub fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.nodes.capacity() * size_of::<Node>()
            + self
                .nodes
                .iter()
                .map(|n| match n {
                    Node::Sum(c) | Node::Product(c) => c.capacity() * size_of::<NodeId>(),
                    _ => 0,
                })
                .sum::<usize>()
            + self.fidelity.capacity()
            + self.variables.capacity() * size_of::<ProjectedVariable>()
            + self.auxiliaries.capacity() * (size_of::<Auxiliary>() + 32)
            + self.rows.capacity() * size_of::<ProjectedRow>()
            + self
                .obligations
                .iter()
                .map(|o| size_of_val(o) + o.constraints.capacity() * size_of::<Constraint>())
                .sum::<usize>()
            + self
                .implicit
                .iter()
                .map(|i| {
                    size_of_val(i)
                        + i.unknowns.capacity() * size_of::<usize>()
                        + i.residuals.capacity() * size_of::<NodeId>()
                        + i.bounds.capacity() * size_of::<Constraint>()
                })
                .sum::<usize>()
            + self.values.len() * 64
            + self.incomplete.capacity() * size_of::<SemanticId>()
    }
}
fn power(base: f64, exponent: Constant) -> f64 {
    match exponent {
        Constant::Rational(Rational {
            numerator,
            denominator: 1,
        }) => {
            i32::try_from(numerator).map_or_else(|_| base.powf(numerator as f64), |n| base.powi(n))
        }
        Constant::Rational(Rational {
            numerator: 1,
            denominator: 2,
        }) => base.sqrt(),
        other => base.powf(other.value()),
    }
}

impl CasePlan {
    /// Project the case into a library-neutral factorable program under fixed consumed
    /// values. `limit` bounds the number of DAG nodes.
    ///
    /// # Errors
    /// Missing or nonfinite consumed values, cancellation, an inconsistent implicit
    /// definition or envelope ([`FactorableError::Math`]), or the typed refusal of the
    /// declared disjunctive branch policy ([`FactorableError::DisjunctiveBranch`]). An
    /// exhausted node budget leaves the affected rows `Unavailable` and records the instance
    /// in [`FactorableProgram::incomplete`].
    pub fn factorable_program(
        &self,
        values: &CaseValues,
        request: &FactorableRequest,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<FactorableProgram, FactorableError> {
        if limit == 0 {
            return fail(MathError::Limit(EXTENT));
        }
        let columns: BTreeMap<_, _> = self
            .columns()
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, i))
            .collect();
        let declared: BTreeMap<_, _> = self
            .structure()
            .variables()
            .iter()
            .map(|v| (v.port.id, v))
            .collect();
        let variables = self
            .columns()
            .iter()
            .map(|id| {
                let v = declared
                    .get(id)
                    .ok_or_else(|| MathError::Contract("column without a declaration".into()))?;
                Ok(ProjectedVariable {
                    id: *id,
                    domain: v.domain,
                    lower: if v.domain.is_semi() {
                        0.0_f64.min(v.lower.unwrap_or(0.0))
                    } else {
                        v.lower.unwrap_or(f64::NEG_INFINITY)
                    },
                    upper: v.upper.unwrap_or(f64::INFINITY),
                })
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let rows: BTreeMap<_, _> = self
            .structure()
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();
        let slots = self
            .bodies()
            .values()
            .map(|b| b.slots)
            .chain(request.implicit.values().flat_map(|d| {
                std::iter::once(d.residual.slots).chain(d.bounds.iter().map(|b| b.body.slots))
            }))
            .max()
            .unwrap_or(0);
        let mut builder = Builder::new(request, cancel, limit, slots)?;
        let mut terms: Vec<Vec<NodeId>> = vec![vec![]; rows.len()];
        let mut unavailable = vec![false; rows.len()];
        let mut objective_terms = vec![];
        let mut objective_unavailable = false;
        let mut consumed = BTreeMap::new();
        for b in self.structure().instances() {
            builder.check()?;
            let body = self
                .bodies()
                .get(&b.body)
                .ok_or_else(|| MathError::Contract("missing prepared body".into()))?;
            let mut inputs = Vec::with_capacity(b.slots.len());
            for s in &b.slots {
                inputs.push(if let Some(&column) = columns.get(&s.source()) {
                    Input::Column(column, s.scale(), s.offset())
                } else {
                    let v = *values
                        .scalars
                        .get(&s.source())
                        .ok_or_else(|| MathError::Contract("missing projected parameter".into()))?;
                    if !v.is_finite() {
                        return fail(MathError::Contract("nonfinite projected parameter".into()));
                    }
                    consumed.insert(s.source(), v.to_bits());
                    Input::Value(s.scale() * v + s.offset())
                });
            }
            let outputs: Vec<usize> = b
                .contributions
                .iter()
                .map(|c| c.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let projected = optional(builder.instance(b.instance, body, &inputs, &outputs))?;
            if projected.is_none() {
                builder.incomplete.push(b.instance);
            }
            for c in &b.contributions {
                let term = match projected.as_ref().and_then(|p| p.get(&c.output)) {
                    Some(&node) => optional(builder.scaled(node, c.scale))?,
                    None => None,
                };
                match (c.target, term) {
                    (Target::Row(id), Some(t)) => terms[rows[&id]].push(t),
                    (Target::Row(id), None) => unavailable[rows[&id]] = true,
                    (Target::Objective, Some(t)) => objective_terms.push(t),
                    (Target::Objective, None) => objective_unavailable = true,
                }
            }
        }
        let mut projected_rows = Vec::with_capacity(rows.len());
        for (r, row) in self.structure().rows().iter().enumerate() {
            let expression = if unavailable[r] {
                None
            } else {
                optional(builder.sum(std::mem::take(&mut terms[r])))?
            };
            projected_rows.push(ProjectedRow {
                id: row.id,
                expression,
                lower: row.lower,
                upper: row.upper,
                fidelity: Fidelity::Unavailable,
            });
        }
        let objective = match self.structure().objective() {
            Some(o) => Some(ProjectedObjective {
                expression: if objective_unavailable {
                    None
                } else {
                    optional(builder.sum(objective_terms))?
                },
                sense: o.sense,
                fidelity: Fidelity::Unavailable,
            }),
            None => None,
        };
        let mut h = FramedHasher::new("pse.math.factorable.v1");
        h.hash(&self.structure().key())
            .str(match request.branches {
                BranchPolicy::Auxiliary => "branches:auxiliary",
                BranchPolicy::Disjunctive => "branches:disjunctive",
            })
            .u64(limit as u64);
        for (id, bits) in &consumed {
            h.id(id).u64(*bits);
        }
        for (key, definition) in &request.implicit {
            h.hash(&key.0).u64(definition.unknowns.len() as u64);
            for (lower, upper) in &definition.unknowns {
                h.u64(lower.to_bits()).u64(upper.to_bits());
            }
        }
        for (key, envelope) in &request.envelopes {
            h.hash(&key.0);
            for (lower, upper) in envelope {
                h.u64(lower.to_bits()).u64(upper.to_bits());
            }
        }
        let mut program = FactorableProgram {
            key: h.finish_hash(),
            structure: self.structure().key(),
            values: consumed,
            nodes: builder.nodes,
            variables,
            auxiliaries: builder.auxiliaries,
            rows: projected_rows,
            objective,
            obligations: builder.obligations,
            implicit: builder.implicit,
            incomplete: builder.incomplete,
            fidelity: vec![],
        };
        classify(&mut program);
        Ok(program)
    }
}

/// Propagate auxiliary fidelity through the DAG. Implicit unknowns take the fidelity of
/// their residual export, iterated to a fixed point for nested blocks.
fn classify(p: &mut FactorableProgram) {
    let mut aux: Vec<Fidelity> = p
        .auxiliaries
        .iter()
        .map(|a| match a.role {
            AuxiliaryRole::Implicit { .. } => Fidelity::Exact,
            AuxiliaryRole::Opaque(_) => Fidelity::Relaxed,
        })
        .collect();
    let nodes = |aux: &[Fidelity]| {
        let mut fidelity: Vec<Fidelity> = Vec::with_capacity(p.nodes.len());
        for node in &p.nodes {
            let f = match node {
                Node::Var(_) | Node::Const(_) => Fidelity::Exact,
                Node::Aux(k) => aux[*k],
                Node::Sum(c) | Node::Product(c) => c
                    .iter()
                    .map(|&i| fidelity[i])
                    .max()
                    .unwrap_or(Fidelity::Exact),
                Node::Pow { base: i, .. }
                | Node::Exp(i)
                | Node::Log(i)
                | Node::Abs(i)
                | Node::Sin(i)
                | Node::Cos(i) => fidelity[*i],
            };
            fidelity.push(f);
        }
        fidelity
    };
    let mut fidelity = nodes(&aux);
    for _ in 0..=MAX_NESTING {
        let mut changed = false;
        for block in &mut p.implicit {
            block.fidelity = block
                .residuals
                .iter()
                .chain(block.bounds.iter().map(|c| &c.expression))
                .map(|&i| fidelity[i])
                .max()
                .unwrap_or(Fidelity::Exact);
            for &u in &block.unknowns {
                if aux[u] < block.fidelity {
                    aux[u] = block.fidelity;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
        fidelity = nodes(&aux);
    }
    for (a, f) in p.auxiliaries.iter_mut().zip(&aux) {
        a.fidelity = *f;
    }
    // An opaque auxiliary without a finite enforced box admits every value.
    let mut free: Vec<bool> = Vec::with_capacity(p.nodes.len());
    for node in &p.nodes {
        let f = match node {
            Node::Var(_) | Node::Const(_) => false,
            Node::Aux(k) => {
                let a = &p.auxiliaries[*k];
                matches!(a.role, AuxiliaryRole::Opaque(_))
                    && (!a.lower.is_finite() || !a.upper.is_finite())
            }
            Node::Sum(c) | Node::Product(c) => c.iter().any(|&i| free[i]),
            Node::Pow { base: i, .. }
            | Node::Exp(i)
            | Node::Log(i)
            | Node::Abs(i)
            | Node::Sin(i)
            | Node::Cos(i) => free[*i],
        };
        free.push(f);
    }
    for row in &mut p.rows {
        row.fidelity = row
            .expression
            .map_or(Fidelity::Unavailable, |e| fidelity[e]);
    }
    if let Some(o) = &mut p.objective {
        o.fidelity = match o.expression {
            Some(e) if !free[e] => fidelity[e],
            _ => Fidelity::Unavailable,
        };
    }
    for o in &mut p.obligations {
        let worst = o
            .constraints
            .iter()
            .map(|c| fidelity[c.expression])
            .max()
            .unwrap_or(Fidelity::Exact);
        o.fidelity = if o.represented {
            worst
        } else {
            Fidelity::Relaxed.max(worst)
        };
    }
    p.fidelity = fidelity;
}

/// A factorable projection failure.
#[derive(Debug, thiserror::Error)]
pub enum FactorableError {
    /// Admission, consumed values, cancellation or a resource bound.
    #[error(transparent)]
    Math(#[from] MathError),
    /// The declared policy asks for the exact disjunctive (mixed-integer) export of a
    /// branch with proven continuity; it belongs to the discrete-decision packets
    /// (Plan 22 M4, G7). The auxiliary policy exports the same branch as a relaxation.
    #[error("instance {instance}: disjunctive export of a continuous branch is not provided")]
    DisjunctiveBranch {
        /// Instance whose branch required the disjunctive form.
        instance: SemanticId,
    },
}
pse_diagnostics::impl_diagnostic! {
    FactorableError,
    code(this) { match this {
        Self::Math(_) => None,
        Self::DisjunctiveBranch { .. } => Some(pse_diagnostics::DiagnosticCode::CompileMath),
    } },
    forward(this) { match this { Self::Math(e) => Some(e), Self::DisjunctiveBranch { .. } => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn fail<T>(error: MathError) -> Result<T, FactorableError> {
    Err(FactorableError::Math(error))
}
const EXTENT: &str = "factorable projection extent";
fn optional<T>(result: Result<T, FactorableError>) -> Result<Option<T>, FactorableError> {
    match result {
        Err(FactorableError::Math(MathError::Limit(EXTENT))) => Ok(None),
        other => other.map(Some),
    }
}

#[derive(Clone, Copy, Debug)]
enum Input {
    Column(usize, f64, f64),
    Value(f64),
}

/// Symbolic value of one stage slot during projection.
#[derive(Clone, Debug)]
enum Value {
    Unset,
    Node(NodeId),
    /// A branch result, kept symbolic so predicates can be decomposed.
    Select(Arc<Select>),
    /// A sum whose terms include branch results.
    Terms(Arc<[Value]>),
}
impl Value {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unset, Self::Unset) => true,
            (Self::Node(a), Self::Node(b)) => a == b,
            (Self::Select(a), Self::Select(b)) => Arc::ptr_eq(a, b),
            (Self::Terms(a), Self::Terms(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
#[derive(Debug)]
struct Select {
    id: usize,
    instance: SemanticId,
    comparison: Comparison,
    left: Value,
    right: Value,
    then: Value,
    otherwise: Value,
    /// Exact absolute-value identity, when it holds.
    exact: Option<NodeId>,
}
#[derive(Clone, Debug)]
struct Env {
    values: Vec<Value>,
    /// Stage expression that assigned each slot, for exact branch identities.
    definitions: Vec<Option<Arc<Atom>>>,
}
impl Env {
    fn new(slots: usize) -> Self {
        Self {
            values: vec![Value::Unset; slots],
            definitions: vec![None; slots],
        }
    }
    fn read(&self, slot: usize) -> Result<Value, MathError> {
        match self.values.get(slot) {
            Some(Value::Unset) | None => Err(MathError::Contract(
                "projection reads a value before its producer".into(),
            )),
            Some(v) => Ok(v.clone()),
        }
    }
    fn assign(
        &mut self,
        slot: usize,
        value: Value,
        definition: Option<Arc<Atom>>,
    ) -> Result<(), MathError> {
        if slot >= self.values.len() {
            return Err(MathError::Contract("projected stage destination".into()));
        }
        self.values[slot] = value;
        self.definitions[slot] = definition;
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
struct Context {
    instance: SemanticId,
    /// Inside a branch region.
    local: bool,
    /// Record obligations (false for bound programs).
    record: bool,
    depth: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Test {
    Positive,
    Zero,
}
impl Test {
    fn accepts(self, value: f64) -> bool {
        match self {
            Self::Positive => value > 0.0,
            Self::Zero => value == 0.0,
        }
    }
}
/// Closed representation of a predicate over the evaluator's admitted inputs.
#[derive(Debug)]
enum Truth {
    Never,
    /// Conjunction of closed constraints; `exact` when nothing was dropped.
    When(Vec<Constraint>, bool),
}
impl Truth {
    const ALWAYS: Self = Self::When(Vec::new(), true);
    const UNKNOWN: Self = Self::When(Vec::new(), false);
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Never, _) | (_, Self::Never) => Self::Never,
            (Self::When(mut a, x), Self::When(b, y)) => {
                a.extend(b);
                Self::When(a, x && y)
            }
        }
    }
    fn always(&self) -> bool {
        matches!(self, Self::When(c, true) if c.is_empty())
    }
}
type Call = (ProviderKey, bool, Vec<usize>, Vec<NodeId>);
struct Builder<'a> {
    request: &'a FactorableRequest,
    cancel: &'a Arc<AtomicBool>,
    limit: usize,
    symbols: HashMap<Symbol, usize>,
    nodes: Vec<Node>,
    interned: HashMap<Node, NodeId>,
    auxiliaries: Vec<Auxiliary>,
    obligations: Vec<ProjectedObligation>,
    implicit: Vec<ProjectedImplicit>,
    incomplete: Vec<SemanticId>,
    calls: HashMap<Call, Vec<NodeId>>,
    selects: HashMap<usize, NodeId>,
    next_select: usize,
}
impl<'a> Builder<'a> {
    fn new(
        request: &'a FactorableRequest,
        cancel: &'a Arc<AtomicBool>,
        limit: usize,
        slots: usize,
    ) -> Result<Self, FactorableError> {
        let mut symbols = HashMap::with_capacity(slots);
        for slot in 0..slots {
            if let AtomView::Var(v) = library::formal(slot)?.as_view() {
                symbols.insert(v.get_symbol(), slot);
            }
        }
        let zero = Node::Const(Constant::integer(0));
        Ok(Self {
            request,
            cancel,
            limit,
            symbols,
            // The neutral constant exists before any budgeted node so that empty rows stay exact.
            nodes: vec![zero.clone()],
            interned: HashMap::from([(zero, 0)]),
            auxiliaries: vec![],
            obligations: vec![],
            implicit: vec![],
            incomplete: vec![],
            calls: HashMap::new(),
            selects: HashMap::new(),
            next_select: 0,
        })
    }
    fn check(&self) -> Result<(), FactorableError> {
        if self.cancel.load(Ordering::Relaxed) {
            fail(MathError::Cancelled)
        } else {
            Ok(())
        }
    }
    fn constant_of(&self, node: NodeId) -> Option<Constant> {
        match self.nodes.get(node) {
            Some(Node::Const(c)) => Some(*c),
            _ => None,
        }
    }
    fn value_constant(&self, value: &Value) -> Option<f64> {
        match value {
            Value::Node(n) => self.constant_of(*n).map(Constant::value),
            _ => None,
        }
    }
    /// Intern a node, folding constant operands and trivial arities.
    fn push(&mut self, node: Node) -> Result<NodeId, FactorableError> {
        let node = match node {
            Node::Sum(mut c) | Node::Product(mut c) if c.len() == 1 => return Ok(c.remove(0)),
            Node::Sum(c) if c.is_empty() => Node::Const(Constant::integer(0)),
            Node::Product(c) if c.is_empty() => Node::Const(Constant::integer(1)),
            node => node,
        };
        let folded = match &node {
            Node::Sum(c) => self.fold(c, Rational::add, |a, b| a + b, 0),
            Node::Product(c) => self.fold(c, Rational::mul, |a, b| a * b, 1),
            Node::Pow { base, exponent } => self
                .constant_of(*base)
                .map(|b| Constant::Float(power(b.value(), *exponent))),
            Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
                self.constant_of(*i).map(|c| {
                    let v = c.value();
                    Constant::Float(match &node {
                        Node::Exp(_) => v.exp(),
                        Node::Log(_) => v.ln(),
                        Node::Abs(_) => v.abs(),
                        Node::Sin(_) => v.sin(),
                        _ => v.cos(),
                    })
                })
            }
            _ => None,
        };
        let node = match folded {
            Some(Constant::Float(v)) if !v.is_finite() => node,
            Some(c) => Node::Const(c),
            None => node,
        };
        if let Some(&id) = self.interned.get(&node) {
            return Ok(id);
        }
        if self.nodes.len() >= self.limit {
            return fail(MathError::Limit(EXTENT));
        }
        let id = self.nodes.len();
        self.nodes.push(node.clone());
        self.interned.insert(node, id);
        Ok(id)
    }
    fn fold(
        &self,
        children: &[NodeId],
        rational: fn(Rational, Rational) -> Option<Rational>,
        float: fn(f64, f64) -> f64,
        unit: i64,
    ) -> Option<Constant> {
        let constants = children
            .iter()
            .map(|&c| self.constant_of(c))
            .collect::<Option<Vec<_>>>()?;
        Constant::fold(&constants, rational, float, unit)
    }
    fn constant(&mut self, c: Constant) -> Result<NodeId, FactorableError> {
        self.push(Node::Const(c))
    }
    fn float(&mut self, v: f64) -> Result<NodeId, FactorableError> {
        if !v.is_finite() {
            return fail(MathError::Contract("nonfinite projected constant".into()));
        }
        self.push(Node::Const(Constant::Float(v)))
    }
    fn scaled(&mut self, node: NodeId, scale: f64) -> Result<NodeId, FactorableError> {
        if scale == 1.0 {
            return Ok(node);
        }
        let s = self.float(scale)?;
        self.push(Node::Product(vec![s, node]))
    }
    fn sum(&mut self, terms: Vec<NodeId>) -> Result<NodeId, FactorableError> {
        self.push(Node::Sum(terms))
    }
    /// `a - b`.
    fn difference(&mut self, a: NodeId, b: NodeId) -> Result<NodeId, FactorableError> {
        let minus = self.constant(Constant::integer(-1))?;
        let negated = self.push(Node::Product(vec![minus, b]))?;
        self.push(Node::Sum(vec![a, negated]))
    }
    fn auxiliary(
        &mut self,
        instance: SemanticId,
        role: AuxiliaryRole,
        lower: f64,
        upper: f64,
    ) -> Result<NodeId, FactorableError> {
        if lower.is_nan() || upper.is_nan() || lower > upper {
            return fail(MathError::Contract("auxiliary interval".into()));
        }
        let index = self.auxiliaries.len();
        let node = self.push(Node::Aux(index))?;
        self.auxiliaries.push(Auxiliary {
            instance,
            role,
            lower,
            upper,
            fidelity: Fidelity::Relaxed,
        });
        Ok(node)
    }
    fn opaque(&mut self, cx: Context, opacity: Opacity) -> Result<NodeId, FactorableError> {
        self.auxiliary(
            cx.instance,
            AuxiliaryRole::Opaque(opacity),
            f64::NEG_INFINITY,
            f64::INFINITY,
        )
    }

    /// Project the demanded outputs of one bound body; returns output ordinal → node.
    fn instance(
        &mut self,
        instance: SemanticId,
        body: &PreparedBody,
        inputs: &[Input],
        outputs: &[usize],
    ) -> Result<BTreeMap<usize, NodeId>, FactorableError> {
        let mut formals = Vec::with_capacity(inputs.len());
        for input in inputs {
            formals.push(match *input {
                Input::Column(column, scale, offset) => {
                    let v = self.push(Node::Var(column))?;
                    let v = self.scaled(v, scale)?;
                    if offset == 0.0 {
                        v
                    } else {
                        let o = self.float(offset)?;
                        self.push(Node::Sum(vec![v, o]))?
                    }
                }
                Input::Value(v) => self.float(v)?,
            });
        }
        let cx = Context {
            instance,
            local: false,
            record: true,
            depth: 0,
        };
        self.body(body, &formals, outputs, cx)
    }
    fn body(
        &mut self,
        body: &PreparedBody,
        formals: &[NodeId],
        outputs: &[usize],
        cx: Context,
    ) -> Result<BTreeMap<usize, NodeId>, FactorableError> {
        if formals.len() != body.input_count() {
            return fail(MathError::Contract("projected body arity".into()));
        }
        let stages = body.demanded_stages(outputs)?;
        let mut env = Env::new(body.slots);
        for (slot, &node) in formals.iter().enumerate() {
            env.assign(
                slot,
                Value::Node(node),
                Some(Arc::new(library::formal(slot)?)),
            )?;
        }
        self.stages(&stages, &mut env, cx)?;
        outputs
            .iter()
            .map(|&o| {
                let value = env.read(body.outputs[o])?;
                Ok((o, self.materialize(&value)?))
            })
            .collect()
    }
    fn stages(
        &mut self,
        stages: &[Stage],
        env: &mut Env,
        cx: Context,
    ) -> Result<(), FactorableError> {
        for stage in stages {
            self.check()?;
            match stage {
                Stage::Block {
                    expressions,
                    outputs,
                    ..
                } => {
                    for (expression, &slot) in expressions.iter().zip(outputs) {
                        let value = self.block(expression, env, cx)?;
                        let definition = (expression.as_view().get_byte_size() <= DEFINITION_BYTES)
                            .then(|| Arc::new(expression.clone()));
                        env.assign(slot, value, definition)?;
                    }
                }
                Stage::Require {
                    argument,
                    condition,
                    order,
                    source,
                } => {
                    // Higher-order requirements admit derivatives; the value domain is the
                    // Value-order obligation.
                    if cx.record && *order == DerivativeOrder::Value {
                        let value = env.read(*argument)?;
                        let node = self.materialize(&value)?;
                        let truth = match (self.constant_of(node), condition) {
                            (Some(c), _) if !c.value().is_finite() => Truth::Never,
                            (Some(c), condition) => {
                                if condition.permits(c.value()) {
                                    Truth::ALWAYS
                                } else {
                                    Truth::Never
                                }
                            }
                            (None, Condition::Positive) => {
                                Truth::When(vec![at_least(node, 0.0, true)], true)
                            }
                            (None, Condition::Nonnegative) => {
                                Truth::When(vec![at_least(node, 0.0, false)], true)
                            }
                            // The closure of a nonzero requirement admits every point.
                            (None, Condition::Nonzero) => Truth::ALWAYS,
                        };
                        self.obligation(
                            cx,
                            *source,
                            ObligationKind::Require(*condition),
                            Some(node),
                            truth,
                        )?;
                    }
                }
                Stage::Domain {
                    stages,
                    argument,
                    token,
                    source,
                } => {
                    let mut local = env.clone();
                    self.stages(stages, &mut local, cx)?;
                    if cx.record {
                        let predicate = local.read(*argument)?;
                        let truth = self.holds(&predicate, Test::Positive)?;
                        self.obligation(cx, *source, ObligationKind::Domain, None, truth)?;
                    }
                    // The token only orders the obligation; it never carries a value.
                    let zero = self.constant(Constant::integer(0))?;
                    env.assign(*token, Value::Node(zero), Some(Arc::new(Atom::num(0))))?;
                }
                Stage::Branch {
                    continuity,
                    comparison,
                    left,
                    right,
                    then,
                    otherwise,
                } => self.branch(
                    *continuity,
                    *comparison,
                    (*left, *right),
                    (then, otherwise),
                    env,
                    cx,
                )?,
                Stage::Provider {
                    spec,
                    partial,
                    inputs,
                    outputs,
                    ..
                } => self.provider(spec, partial, inputs, outputs, env, cx)?,
            }
        }
        Ok(())
    }
    fn obligation(
        &mut self,
        cx: Context,
        source: SemanticId,
        kind: ObligationKind,
        argument: Option<NodeId>,
        truth: Truth,
    ) -> Result<(), FactorableError> {
        let (constraints, represented) = match truth {
            Truth::Never => {
                let one = self.constant(Constant::integer(1))?;
                (
                    vec![Constraint {
                        expression: one,
                        lower: f64::NEG_INFINITY,
                        upper: 0.0,
                        strict: false,
                    }],
                    true,
                )
            }
            Truth::When(constraints, exact) => (constraints, exact),
        };
        self.obligations.push(ProjectedObligation {
            instance: cx.instance,
            source,
            kind,
            scope: if cx.local {
                ObligationScope::Conditional
            } else {
                ObligationScope::Unconditional
            },
            argument,
            constraints,
            represented,
            fidelity: Fidelity::Relaxed,
        });
        Ok(())
    }
    /// A stage expression's value. Aliases and sums over branch results stay symbolic.
    fn block(&mut self, atom: &Atom, env: &Env, cx: Context) -> Result<Value, FactorableError> {
        match atom.as_view() {
            AtomView::Var(v) => {
                if let Some(&k) = self.symbols.get(&v.get_symbol()) {
                    return env.read(k).map_err(FactorableError::from);
                }
            }
            AtomView::Add(add) => {
                let mut terms = Vec::new();
                let mut symbolic = false;
                let mut plain = true;
                for term in add.iter() {
                    match term {
                        AtomView::Var(v) if self.symbols.contains_key(&v.get_symbol()) => {
                            let value = env.read(self.symbols[&v.get_symbol()])?;
                            symbolic |= matches!(value, Value::Select(_) | Value::Terms(_));
                            terms.push(value);
                        }
                        AtomView::Num(_) => terms.push(Value::Node(self.atom(term, env, cx, 1)?)),
                        _ => {
                            plain = false;
                            break;
                        }
                    }
                }
                if plain && symbolic {
                    return Ok(Value::Terms(terms.into()));
                }
            }
            _ => {}
        }
        Ok(Value::Node(self.atom(atom.as_view(), env, cx, 0)?))
    }
    fn atom(
        &mut self,
        view: AtomView<'_>,
        env: &Env,
        cx: Context,
        depth: usize,
    ) -> Result<NodeId, FactorableError> {
        if depth > MAX_DEPTH {
            return self.opaque(cx, Opacity::Depth);
        }
        self.check()?;
        match view {
            AtomView::Num(n) => match constant(n.get_coeff_view()) {
                Some(c) => self.constant(c),
                None => self.opaque(cx, Opacity::Constant),
            },
            AtomView::Var(v) => {
                let s = v.get_symbol();
                if let Some(&k) = self.symbols.get(&s) {
                    let value = env.read(k)?;
                    self.materialize(&value)
                } else if s == Symbol::E {
                    self.float(std::f64::consts::E)
                } else if s == Symbol::PI {
                    self.float(std::f64::consts::PI)
                } else {
                    fail(MathError::Contract(
                        "unregistered symbol in a projected stage".into(),
                    ))
                }
            }
            AtomView::Add(add) => {
                let children = add
                    .iter()
                    .map(|x| self.atom(x, env, cx, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                self.push(Node::Sum(children))
            }
            AtomView::Mul(mul) => {
                let children = mul
                    .iter()
                    .map(|x| self.atom(x, env, cx, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                self.push(Node::Product(children))
            }
            AtomView::Pow(pow) => {
                let (base, exponent) = pow.get_base_exp();
                if matches!(base, AtomView::Var(b) if b.get_symbol() == Symbol::E) {
                    let e = self.atom(exponent, env, cx, depth + 1)?;
                    return self.push(Node::Exp(e));
                }
                let b = self.atom(base, env, cx, depth + 1)?;
                if let AtomView::Num(n) = exponent {
                    return match constant(n.get_coeff_view()) {
                        Some(c) => self.push(Node::Pow {
                            base: b,
                            exponent: c,
                        }),
                        None => self.opaque(cx, Opacity::Constant),
                    };
                }
                // A variable real power is defined for a positive base: b^e = exp(e log b).
                let e = self.atom(exponent, env, cx, depth + 1)?;
                let log = self.push(Node::Log(b))?;
                let product = self.push(Node::Product(vec![e, log]))?;
                self.push(Node::Exp(product))
            }
            AtomView::Fun(f) => {
                let s = f.get_symbol();
                let unary: Option<fn(NodeId) -> Node> = match s {
                    _ if f.get_nargs() != 1 => None,
                    s if s == Symbol::EXP => Some(Node::Exp),
                    s if s == Symbol::LOG => Some(Node::Log),
                    s if s == Symbol::SIN => Some(Node::Sin),
                    s if s == Symbol::COS => Some(Node::Cos),
                    s if s == Symbol::ABS => Some(Node::Abs),
                    s if s == Symbol::SQRT => Some(|base| Node::Pow {
                        base,
                        exponent: Constant::Rational(Rational {
                            numerator: 1,
                            denominator: 2,
                        }),
                    }),
                    _ => None,
                };
                match (unary, f.iter().next()) {
                    (Some(op), Some(argument)) => {
                        let a = self.atom(argument, env, cx, depth + 1)?;
                        self.push(op(a))
                    }
                    _ => self.opaque(cx, Opacity::Function(s.get_name().to_owned())),
                }
            }
        }
    }
    /// A node for a slot value. Unresolved branch results become auxiliaries under the
    /// auxiliary policy, bounded by their constant branch values when all are constant.
    fn materialize(&mut self, value: &Value) -> Result<NodeId, FactorableError> {
        match value {
            Value::Unset => fail(MathError::Contract(
                "projection reads a value before its producer".into(),
            )),
            Value::Node(n) => Ok(*n),
            Value::Terms(terms) => {
                let children = terms
                    .iter()
                    .map(|t| self.materialize(t))
                    .collect::<Result<Vec<_>, _>>()?;
                self.push(Node::Sum(children))
            }
            Value::Select(s) => {
                if let Some(e) = s.exact {
                    return Ok(e);
                }
                if let Some(&n) = self.selects.get(&s.id) {
                    return Ok(n);
                }
                let (lower, upper) = self
                    .range(value)
                    .unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
                let n = self.auxiliary(
                    s.instance,
                    AuxiliaryRole::Opaque(Opacity::Branch),
                    lower,
                    upper,
                )?;
                self.selects.insert(s.id, n);
                Ok(n)
            }
        }
    }
    /// Range of a value whose every branch outcome is constant.
    fn range(&self, value: &Value) -> Option<(f64, f64)> {
        match value {
            Value::Unset => None,
            Value::Node(n) => self.constant_of(*n).map(|c| (c.value(), c.value())),
            Value::Select(s) => {
                let (a, b) = (self.range(&s.then)?, self.range(&s.otherwise)?);
                Some((a.0.min(b.0), a.1.max(b.1)))
            }
            Value::Terms(terms) => terms.iter().try_fold((0.0, 0.0), |acc, t| {
                let r = self.range(t)?;
                Some((acc.0 + r.0, acc.1 + r.1))
            }),
        }
    }
    fn branch(
        &mut self,
        continuity: DerivativeOrder,
        comparison: Comparison,
        (left, right): (usize, usize),
        (then, otherwise): (&[Stage], &[Stage]),
        env: &mut Env,
        cx: Context,
    ) -> Result<(), FactorableError> {
        let l = env.read(left)?;
        let r = env.read(right)?;
        // Both regions were physically admitted; a constant guard selects one statically.
        if let (Some(a), Some(b)) = (self.value_constant(&l), self.value_constant(&r)) {
            let taken = if comparison.select(a, b) {
                then
            } else {
                otherwise
            };
            return self.stages(taken, env, cx);
        }
        let region = Context { local: true, ..cx };
        let mut a = env.clone();
        let mut b = env.clone();
        self.stages(then, &mut a, region)?;
        self.stages(otherwise, &mut b, region)?;
        let pure = pure(then) && pure(otherwise);
        // Identities compare against the parent as it was before the branch.
        let mut merged = Vec::new();
        for slot in 0..env.values.len() {
            if a.values[slot].same(&b.values[slot]) {
                if !a.values[slot].same(&env.values[slot]) {
                    merged.push((slot, a.values[slot].clone(), a.definitions[slot].clone()));
                }
                continue;
            }
            if matches!(a.values[slot], Value::Unset) || matches!(b.values[slot], Value::Unset) {
                // Region-local values do not survive the branch.
                merged.push((slot, Value::Unset, None));
                continue;
            }
            let exact = if pure {
                self.identity(env, (&a, &b), slot, (left, right))?
            } else {
                None
            };
            if exact.is_none()
                && continuity != DerivativeOrder::Value
                && self.request.branches == BranchPolicy::Disjunctive
            {
                return Err(FactorableError::DisjunctiveBranch {
                    instance: cx.instance,
                });
            }
            let id = self.next_select;
            self.next_select += 1;
            let select = Value::Select(Arc::new(Select {
                id,
                instance: cx.instance,
                comparison,
                left: l.clone(),
                right: r.clone(),
                then: a.values[slot].clone(),
                otherwise: b.values[slot].clone(),
                exact,
            }));
            merged.push((slot, select, None));
        }
        for (slot, value, definition) in merged {
            env.values[slot] = value;
            env.definitions[slot] = definition;
        }
        Ok(())
    }
    /// The exact absolute-value identity: when `X - Y = α (L - R)` holds symbolically for a
    /// constant α, `if L cmp R then X else Y` equals `(X + Y)/2 - (α/2)|L - R|` everywhere,
    /// both at and away from the boundary. The admitted minimum (α = 1), maximum (α = -1)
    /// and absolute value (α = -2) take this form, in either guard orientation.
    fn identity(
        &mut self,
        parent: &Env,
        (a, b): (&Env, &Env),
        slot: usize,
        (left, right): (usize, usize),
    ) -> Result<Option<NodeId>, FactorableError> {
        let (Some(x), Some(y), Some(l), Some(r)) = (
            &a.definitions[slot],
            &b.definitions[slot],
            &parent.definitions[left],
            &parent.definitions[right],
        ) else {
            return Ok(None);
        };
        let (Some(x), Some(y)) = (
            self.region_definition(x, a, parent)?,
            self.region_definition(y, b, parent)?,
        ) else {
            return Ok(None);
        };
        let separation = (l.as_ref() - r.as_ref()).expand();
        if separation.is_constant() {
            return Ok(None);
        }
        let difference = (&x - &y).expand();
        // Each candidate is proved by expansion, never sampled. Rational cancellation is not
        // used: it requires exact coefficients, and stage expressions carry binary64 ones.
        let Some(alpha) = IDENTITY_FACTORS
            .iter()
            .find_map(|&(numerator, denominator)| {
                let factor = Atom::num(numerator) / Atom::num(denominator);
                (&difference - &(&factor * &separation))
                    .expand()
                    .is_zero()
                    .then_some(Constant::Rational(Rational {
                        numerator,
                        denominator,
                    }))
            })
        else {
            return Ok(None);
        };
        let xn = self.materialize(&a.values[slot])?;
        let yn = self.materialize(&b.values[slot])?;
        let ln = self.materialize(&parent.values[left])?;
        let rn = self.materialize(&parent.values[right])?;
        let half = self.constant(Constant::Rational(Rational {
            numerator: 1,
            denominator: 2,
        }))?;
        let total = self.push(Node::Sum(vec![xn, yn]))?;
        let mean = self.push(Node::Product(vec![half, total]))?;
        if alpha.value() == 0.0 {
            return Ok(Some(mean));
        }
        let gap = self.difference(ln, rn)?;
        let magnitude = self.push(Node::Abs(gap))?;
        let coefficient = match alpha {
            Constant::Rational(r) => r
                .mul(Rational {
                    numerator: -1,
                    denominator: 2,
                })
                .map(Constant::Rational),
            Constant::Float(v) => Some(Constant::Float(-v / 2.0)),
        };
        let Some(coefficient) = coefficient else {
            return Ok(None);
        };
        let coefficient = self.constant(coefficient)?;
        let term = self.push(Node::Product(vec![coefficient, magnitude]))?;
        self.push(Node::Sum(vec![mean, term])).map(Some)
    }
    /// Substitute definitions of region-local slots so that the identity compares
    /// expressions over values shared with the parent.
    fn region_definition(
        &self,
        atom: &Atom,
        region: &Env,
        parent: &Env,
    ) -> Result<Option<Atom>, FactorableError> {
        let mut atom = atom.clone();
        for _ in 0..MAX_NESTING {
            let mut changed = false;
            for symbol in atom.get_all_symbols(false) {
                let Some(&k) = self.symbols.get(&symbol) else {
                    continue;
                };
                if !matches!(parent.values.get(k), Some(Value::Unset)) {
                    continue;
                }
                let Some(definition) = region.definitions.get(k).cloned().flatten() else {
                    return Ok(None);
                };
                atom = atom
                    .replace(library::formal(k)?)
                    .with(definition.as_ref().clone());
                changed = true;
            }
            if !changed {
                return Ok(Some(atom));
            }
        }
        Ok(None)
    }
    fn provider(
        &mut self,
        spec: &ProviderSpec,
        partial: &[usize],
        inputs: &[usize],
        outputs: &[usize],
        env: &mut Env,
        cx: Context,
    ) -> Result<(), FactorableError> {
        let mut nodes = Vec::with_capacity(inputs.len());
        for &i in inputs {
            let value = env.read(i)?;
            nodes.push(self.materialize(&value)?);
        }
        // Residual equations are exported only where the call is unconditional; inside a
        // branch region they would constrain points where the call is never made.
        let definition = if partial.is_empty() && !cx.local && cx.depth < MAX_NESTING {
            self.request.implicit.get(&spec.key())
        } else {
            None
        };
        let key = (
            spec.key(),
            definition.is_some(),
            partial.to_vec(),
            nodes.clone(),
        );
        let results = if let Some(results) = self.calls.get(&key) {
            results.clone()
        } else {
            let results = match definition {
                Some(definition) => self.implicit_block(spec, definition, &nodes, cx)?,
                None => {
                    let envelope = if partial.is_empty() {
                        self.request.envelopes.get(&spec.key())
                    } else {
                        None
                    };
                    if envelope.is_some_and(|e| e.len() != spec.outputs.len()) {
                        return fail(MathError::Contract("provider envelope arity".into()));
                    }
                    (0..spec.outputs.len())
                        .map(|k| {
                            let (lower, upper) = envelope
                                .and_then(|e| e.get(k))
                                .copied()
                                .unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
                            self.auxiliary(
                                cx.instance,
                                AuxiliaryRole::Opaque(Opacity::Provider {
                                    provider: spec.id,
                                    output: k,
                                }),
                                lower,
                                upper,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?
                }
            };
            self.calls.insert(key, results.clone());
            results
        };
        for (k, &slot) in outputs.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let node = *results
                .get(k)
                .ok_or_else(|| MathError::Contract("provider output arity".into()))?;
            env.assign(
                slot,
                Value::Node(node),
                Some(Arc::new(library::formal(slot)?)),
            )?;
        }
        Ok(())
    }
    fn implicit_block(
        &mut self,
        spec: &ProviderSpec,
        definition: &ImplicitDefinition,
        inputs: &[NodeId],
        cx: Context,
    ) -> Result<Vec<NodeId>, FactorableError> {
        let n = definition.unknowns.len();
        let residual = &definition.residual;
        if n == 0
            || n != spec.outputs.len()
            || residual.input_count() != n + inputs.len()
            || residual.output_count() != n
        {
            return fail(MathError::Contract("implicit definition layout".into()));
        }
        let first = self.auxiliaries.len();
        let mut unknowns = Vec::with_capacity(n);
        for (k, &(lower, upper)) in definition.unknowns.iter().enumerate() {
            unknowns.push(self.auxiliary(
                cx.instance,
                AuxiliaryRole::Implicit {
                    provider: spec.id,
                    unknown: k,
                },
                lower,
                upper,
            )?);
        }
        let inner = Context {
            depth: cx.depth + 1,
            ..cx
        };
        let mut bounds = Vec::new();
        if let Some(program) = &definition.bounds {
            if program.body.input_count() != n + inputs.len()
                || program.lower.len() != n
                || program.upper.len() != n
            {
                return fail(MathError::Contract("implicit bound program layout".into()));
            }
            // The evaluator never reads unknown coordinates in its bound program.
            let zero = self.constant(Constant::integer(0))?;
            let formals: Vec<_> = std::iter::repeat_n(zero, n)
                .chain(inputs.iter().copied())
                .collect();
            let selected: Vec<usize> = program
                .lower
                .iter()
                .chain(&program.upper)
                .flatten()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let values = self.body(
                &program.body,
                &formals,
                &selected,
                Context {
                    record: false,
                    ..inner
                },
            )?;
            for (k, &unknown) in unknowns.iter().enumerate() {
                let index = first + k;
                for (ordinal, lower) in [(program.lower[k], true), (program.upper[k], false)] {
                    let Some(ordinal) = ordinal else { continue };
                    let bound = values[&ordinal];
                    match self.constant_of(bound).map(Constant::value) {
                        Some(v) if lower => {
                            let a = &mut self.auxiliaries[index];
                            a.lower = a.lower.max(v);
                        }
                        Some(v) => {
                            let a = &mut self.auxiliaries[index];
                            a.upper = a.upper.min(v);
                        }
                        None => {
                            let gap = self.difference(unknown, bound)?;
                            bounds.push(if lower {
                                at_least(gap, 0.0, false)
                            } else {
                                at_most(gap, 0.0, false)
                            });
                        }
                    }
                }
                if self.auxiliaries[index].lower > self.auxiliaries[index].upper {
                    return fail(MathError::Contract("empty implicit interval".into()));
                }
            }
        }
        let formals: Vec<_> = unknowns.iter().chain(inputs).copied().collect();
        let ordinals: Vec<usize> = (0..n).collect();
        let values = self.body(residual, &formals, &ordinals, inner)?;
        self.implicit.push(ProjectedImplicit {
            instance: cx.instance,
            provider: spec.id,
            unknowns: (first..first + n).collect(),
            residuals: ordinals.iter().map(|o| values[o]).collect(),
            bounds,
            fidelity: Fidelity::Exact,
        });
        Ok(unknowns)
    }
    /// Closed representation of `value > 0` or `value == 0`.
    fn holds(&mut self, value: &Value, test: Test) -> Result<Truth, FactorableError> {
        match value {
            Value::Unset => fail(MathError::Contract(
                "predicate reads a value before its producer".into(),
            )),
            Value::Node(n) => Ok(match self.constant_of(*n) {
                Some(c) if test.accepts(c.value()) => Truth::ALWAYS,
                Some(_) => Truth::Never,
                None => Truth::When(
                    vec![match test {
                        Test::Positive => at_least(*n, 0.0, true),
                        Test::Zero => Constraint {
                            expression: *n,
                            lower: 0.0,
                            upper: 0.0,
                            strict: false,
                        },
                    }],
                    true,
                ),
            }),
            Value::Select(s) => {
                if let Some(e) = s.exact {
                    return self.holds(&Value::Node(e), test);
                }
                let t = self.holds(&s.then, test)?;
                let e = self.holds(&s.otherwise, test)?;
                Ok(match (t, e) {
                    (Truth::Never, Truth::Never) => Truth::Never,
                    (t, e) if t.always() && e.always() => Truth::ALWAYS,
                    (t, Truth::Never) => self.comparison(s, false)?.and(t),
                    (Truth::Never, e) => self.comparison(s, true)?.and(e),
                    // A disjunction has no closed conjunctive form; dropping it is sound.
                    _ => Truth::UNKNOWN,
                })
            }
            Value::Terms(terms) => {
                // A sum of terms whose outcomes are all nonnegative is zero only when every
                // term is zero.
                if test == Test::Zero
                    && terms
                        .iter()
                        .all(|t| self.range(t).is_some_and(|(lower, _)| lower >= 0.0))
                {
                    let mut truth = Truth::ALWAYS;
                    for term in terms.iter() {
                        truth = truth.and(self.holds(term, Test::Zero)?);
                    }
                    return Ok(truth);
                }
                let node = self.materialize(value)?;
                self.holds(&Value::Node(node), test)
            }
        }
    }
    /// Closed form of a branch guard, or of its negation.
    fn comparison(&mut self, s: &Select, negate: bool) -> Result<Truth, FactorableError> {
        let equality = matches!(
            (s.comparison, negate),
            (Comparison::Eq, false) | (Comparison::Ne, true)
        );
        if equality {
            if self.value_constant(&s.right) == Some(0.0) {
                return self.holds(&s.left, Test::Zero);
            }
            if self.value_constant(&s.left) == Some(0.0) {
                return self.holds(&s.right, Test::Zero);
            }
        }
        let l = self.materialize(&s.left)?;
        let r = self.materialize(&s.right)?;
        // The closed constraint `small <= large`, from a strict comparison when `strict`.
        let (small, large, strict) = match (s.comparison, negate) {
            (Comparison::Lt, false) => (l, r, true),
            (Comparison::Le, false) => (l, r, false),
            (Comparison::Lt, true) => (r, l, false),
            (Comparison::Le, true) => (r, l, true),
            (Comparison::Eq, false) | (Comparison::Ne, true) => {
                let gap = self.difference(l, r)?;
                return Ok(Truth::When(
                    vec![Constraint {
                        expression: gap,
                        lower: 0.0,
                        upper: 0.0,
                        strict: false,
                    }],
                    true,
                ));
            }
            // An inequation is open; its closure admits every point.
            (Comparison::Eq, true) | (Comparison::Ne, false) => return Ok(Truth::UNKNOWN),
        };
        let constraint = match (self.constant_of(small), self.constant_of(large)) {
            (Some(a), Some(b)) => {
                let (a, b) = (a.value(), b.value());
                return Ok(if a < b || (!strict && a == b) {
                    Truth::ALWAYS
                } else {
                    Truth::Never
                });
            }
            (_, Some(b)) => at_most(small, b.value(), strict),
            (Some(a), _) => at_least(large, a.value(), strict),
            (None, None) => {
                let gap = self.difference(small, large)?;
                at_most(gap, 0.0, strict)
            }
        };
        Ok(Truth::When(vec![constraint], true))
    }
}
fn at_least(expression: NodeId, lower: f64, strict: bool) -> Constraint {
    Constraint {
        expression,
        lower,
        upper: f64::INFINITY,
        strict,
    }
}
fn at_most(expression: NodeId, upper: f64, strict: bool) -> Constraint {
    Constraint {
        expression,
        lower: f64::NEG_INFINITY,
        upper,
        strict,
    }
}
/// Only arithmetic: no obligation, provider or domain predicate inside the regions.
fn pure(stages: &[Stage]) -> bool {
    stages.iter().all(|s| match s {
        Stage::Block { .. } => true,
        Stage::Branch {
            then, otherwise, ..
        } => pure(then) && pure(otherwise),
        _ => false,
    })
}
/// A real finite constant from a library coefficient, exact when rational.
fn constant(view: CoefficientView<'_>) -> Option<Constant> {
    use symbolica::domains::float::RealLike;
    match view {
        CoefficientView::Natural(n, d, 0, _) => Rational::new(n, d).map(Constant::Rational),
        CoefficientView::Float(re, im) if im.is_zero() => {
            let v = re.to_float().to_f64();
            v.is_finite().then_some(Constant::Float(v))
        }
        CoefficientView::Large(re, im) if im.is_zero() => {
            let v = re.to_rat().to_f64();
            v.is_finite().then_some(Constant::Float(v))
        }
        _ => None,
    }
}
