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
//! Implicit blocks export residual equations with compiler-owned selection evidence. A
//! selected function is exact only when the represented graph is library-established;
//! otherwise lifting all residual roots is a sound relaxation. Other providers and unrepresentable branches become auxiliary variables
//! within the envelope their evaluation enforces, which makes the dependent rows `Relaxed`.
//! The evaluator and original-coordinate qualification remain the authority for every
//! candidate (ADR-0105).
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, ObjectiveSense, Target},
    guarded::{Comparison, Condition, PreparedBody, Stage},
    library,
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, ProviderKey, ProviderSpec};
use pse_model::generated::enums::ModelingVariableDomain;
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    coefficient::{Coefficient, CoefficientView},
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

/// Library-owned arbitrary precision rational; no local arithmetic authority.
pub use symbolica::domains::rational::Rational;

/// A constant, retained exactly when the library atom is rational.
#[derive(Clone, Debug)]
pub enum Constant {
    /// Exact rational from the library atom.
    Rational(Rational),
    /// Finite binary64 value; every finite binary64 value is itself a dyadic rational.
    Float(f64),
}
impl Constant {
    /// Binary64 value used by numerical consumers.
    pub fn value(&self) -> f64 {
        match self {
            Self::Rational(r) => r.to_f64(),
            Self::Float(v) => *v,
        }
    }
    fn integer(value: i64) -> Self {
        Self::Rational(Rational::from(value))
    }
    fn fold(
        values: &[Self],
        rational: fn(&Rational, &Rational) -> Rational,
        float: fn(f64, f64) -> f64,
        unit: i64,
    ) -> Option<Self> {
        let exact = values
            .iter()
            .try_fold(Rational::from(unit), |acc, v| match v {
                Self::Rational(r) => Some(rational(&acc, r)),
                Self::Float(_) => None,
            });
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
    /// Canonical identity fields: normalized signed base-ten numerator, positive
    /// denominator, or tagged IEEE-754 bits. Library `Hash` is only used for interning.
    fn frame(&self, h: &mut FramedHasher) {
        match self {
            Self::Rational(r) => {
                h.str("rational:decimal:v1")
                    .str(&r.numerator_ref().to_string())
                    .str(&r.denominator_ref().to_string());
            }
            Self::Float(v) => {
                h.str("binary64").u64(v.to_bits());
            }
        }
    }
    fn allocated_bytes(&self) -> usize {
        match self {
            Self::Rational(r) => [r.numerator_ref(), r.denominator_ref()]
                .into_iter()
                .map(|integer| match integer {
                    symbolica::domains::integer::Integer::Large(value) => {
                        value.as_raw().capacity().div_ceil(8)
                    }
                    _ => 0,
                })
                .sum(),
            Self::Float(_) => 0,
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
    pub domain: ModelingVariableDomain,
    /// Closed lower bound (zero for semicontinuous domains), or negative infinity.
    pub lower: f64,
    /// Closed upper bound, or positive infinity.
    pub upper: f64,
    /// Declared lower bound of a semi domain's active branch `[active_lower, upper]`, which
    /// the box `[lower, upper]` joins with the zero branch; absent for every other domain,
    /// and for a semi domain whose case leaves it undeclared.
    pub active_lower: Option<f64>,
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
    /// Why residual lifting preserves or relaxes the selected meaning.
    pub selection: SelectedGraph,
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

/// Compiler-issued selected graph evidence retained by transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedGraph {
    /// The authored meaning is the entire relation, not a selected function.
    Relation,
    /// Library-checked nondegenerate affine graph with represented selector domain.
    NondegenerateAffine {
        /// Represented sign restriction, if the selector includes one.
        sign: Option<bool>,
        /// Whether the represented sign predicate excludes zero.
        strict: bool,
    },
    /// Library-checked square-root graph on the explicitly selected half-line.
    RestrictedSquareRoot {
        /// Select the nonnegative rather than nonpositive half-line.
        positive: bool,
        /// Whether the represented sign predicate excludes zero.
        strict: bool,
    },
    /// The residual graph includes roots the selector does not promise to return.
    Unestablished {
        /// Canonical selected-meaning tag, including an operational settings reference.
        meaning: String,
    },
}
impl SelectedGraph {
    fn fidelity(&self) -> Fidelity {
        match self {
            Self::Unestablished { .. } => Fidelity::Relaxed,
            _ => Fidelity::Exact,
        }
    }
    fn allocated_bytes(&self) -> usize {
        match self {
            Self::Unestablished { meaning } => meaning.capacity(),
            _ => 0,
        }
    }
    fn checked(&self, body: &PreparedBody, unknowns: usize) -> Result<Self, MathError> {
        use crate::implicit::{SelectionEquivalence, graph_equivalence};
        let (sign, expected) = match self {
            Self::NondegenerateAffine { sign, .. } => {
                (*sign, SelectionEquivalence::NondegenerateAffine)
            }
            Self::RestrictedSquareRoot { positive, .. } => {
                (Some(*positive), SelectionEquivalence::RestrictedSquareRoot)
            }
            Self::Relation | Self::Unestablished { .. } => return Ok(self.clone()),
        };
        if graph_equivalence(body, unknowns, sign)? == expected {
            Ok(self.clone())
        } else {
            Ok(Self::Unestablished {
                meaning: "unsupported-graph-equivalence-witness".into(),
            })
        }
    }
    fn frame(&self, h: &mut FramedHasher) {
        match self {
            Self::Relation => {
                h.str("relation");
            }
            Self::NondegenerateAffine { sign, strict } => {
                h.str("nondegenerate-affine")
                    .bool(sign.is_some())
                    .bool(*strict);
                if let Some(positive) = sign {
                    h.bool(*positive);
                }
            }
            Self::RestrictedSquareRoot { positive, strict } => {
                h.str("restricted-square-root")
                    .bool(*positive)
                    .bool(*strict);
            }
            Self::Unestablished { meaning } => {
                h.str("unestablished-selected-graph").str(meaning);
            }
        }
    }
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
    /// Relation/function selection and checked graph-equivalence witness.
    pub selection: SelectedGraph,
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
    /// Refuse a projection whose complete fidelity is not exact.
    pub require_exact: bool,
}

/// An operand of a native constraint: a free column, or a value the case fixes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NativeOperand {
    /// Free case column, in [`CasePlan::columns`] order.
    Column(usize),
    /// A fixed value; its bits are among the program's consumed values.
    Fixed(f64),
}
/// A constraint form a native realization leaves to the backend's handler (ADR-0104),
/// over program coordinates. The rows it names stay in [`FactorableProgram::rows`];
/// only their enforcement is conditional.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectedNative {
    /// Row `row` holds only while `indicator` equals `active`.
    Indicator {
        /// Row ordinal in [`FactorableProgram::rows`].
        row: usize,
        /// Binary indicator.
        indicator: NativeOperand,
        /// The indicator value that activates the row.
        active: bool,
    },
    /// A special ordered set of type 1 or 2, by strictly increasing weights.
    Sos {
        /// `Sos1` or `Sos2`.
        form: pse_model::generated::enums::NativeConstraintForm,
        /// Members and their weights.
        members: Vec<(NativeOperand, f64)>,
    },
    /// `resultant = op(operands)` for `And`, `Or` or `Xor`; an operand may be negated.
    Logic {
        /// `And`, `Or` or `Xor`.
        form: pse_model::generated::enums::NativeConstraintForm,
        /// Binary resultant.
        resultant: NativeOperand,
        /// Binary operands and whether each is complemented.
        operands: Vec<(NativeOperand, bool)>,
    },
    /// At most `bound` of `members` are nonzero.
    Cardinality {
        /// Members.
        members: Vec<NativeOperand>,
        /// Largest nonzero count.
        bound: u32,
    },
}
impl ProjectedNative {
    /// The handler the constraint needs.
    pub const fn form(&self) -> pse_model::generated::enums::NativeConstraintForm {
        use pse_model::generated::enums::NativeConstraintForm as F;
        match self {
            Self::Indicator { .. } => F::Indicator,
            Self::Sos { form, .. } | Self::Logic { form, .. } => *form,
            Self::Cardinality { .. } => F::Cardinality,
        }
    }
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

/// Exact original-coordinate program for a validated root-isolation adapter.
///
/// This is a projection, not a root proof. Strict bounds remain strict; nonzero
/// requirements retain their argument and kind even though their closure is empty.
/// Consumers must enforce obligations as well as eligibility constraints.
#[derive(Clone, Debug, PartialEq)]
pub struct RootIsolationProgram {
    /// All original formals, without coordinate selection or renumbering.
    pub inputs: usize,
    /// Shared exact real DAG; children precede parents.
    pub nodes: Vec<Node>,
    /// All residual outputs, in the original body output order.
    pub residuals: Vec<NodeId>,
    /// Exact conjunctive predicate, with each open bound marked by `strict`.
    pub eligibility: Vec<Constraint>,
    /// Exact score and absolute tolerance, in the authored criterion output order.
    pub criterion: [NodeId; 2],
    /// Unconditional value-domain obligations of all three demanded bodies.
    pub obligations: Vec<ProjectedObligation>,
    /// Additional guards, with their minimum required derivative order.
    /// First consumes entries through First; Second consumes entries through Second.
    pub derivative_obligations: Vec<(DerivativeOrder, ProjectedObligation)>,
}

/// Output positions for one original row in a point arithmetic projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointArithmeticRow {
    /// Original row value, before subtracting a bound.
    pub value: usize,
    /// Original row value minus its finite lower bound, evaluated inside the DAG.
    pub lower_residual: Option<usize>,
    /// Original row value minus its finite upper bound; equality shares the lower output.
    pub upper_residual: Option<usize>,
}

/// Provider-free original-case real arithmetic and derivative-domain projection.
///
/// The graph's residual inventory is an output vector, not a square system or a
/// root claim. Row values, bound residuals and the objective all retain their
/// original physical coordinates. Native adapters must admit every Value and
/// requested derivative obligation before producing an enclosure. The projection
/// initially requires complete First support; retained Second obligations let a
/// native Second request either establish that domain or explicitly refuse it.
#[derive(Clone, Debug)]
pub struct PointArithmeticProgram {
    /// Source projection identity, including the consumed fixed values.
    pub key: ContentHash,
    /// Exact shared DAG, complete Value/First guards and retained Second guards.
    pub graph: RootIsolationProgram,
    /// Output positions in original case row order.
    pub rows: Vec<PointArithmeticRow>,
    /// Objective value output, when the original case has one.
    pub objective: Option<usize>,
}
impl PointArithmeticProgram {
    /// Owned projection extent; transient native memory is admitted separately.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() - size_of::<RootIsolationProgram>()
            + self.graph.retained_bytes()
            + self.rows.capacity() * size_of::<PointArithmeticRow>()
    }
}
impl RootIsolationProgram {
    /// Exact source identity, including guards and derivative obligations.
    pub fn identity(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("exact-root-isolation-program")
            .u64(self.inputs as u64)
            .u64(self.nodes.len() as u64);
        for node in &self.nodes {
            match node {
                Node::Var(i) => {
                    h.str("var").u64(*i as u64);
                }
                Node::Aux(i) => {
                    h.str("aux").u64(*i as u64);
                }
                Node::Const(v) => {
                    h.str("constant");
                    v.frame(&mut h);
                }
                Node::Sum(v) | Node::Product(v) => {
                    h.str(if matches!(node, Node::Sum(_)) {
                        "sum"
                    } else {
                        "product"
                    })
                    .u64(v.len() as u64);
                    for i in v {
                        h.u64(*i as u64);
                    }
                }
                Node::Pow { base, exponent } => {
                    h.str("power").u64(*base as u64);
                    exponent.frame(&mut h);
                }
                Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
                    h.str(match node {
                        Node::Exp(_) => "exp",
                        Node::Log(_) => "log",
                        Node::Abs(_) => "abs",
                        Node::Sin(_) => "sin",
                        _ => "cos",
                    })
                    .u64(*i as u64);
                }
            }
        }
        h.u64(self.residuals.len() as u64);
        for i in &self.residuals {
            h.u64(*i as u64);
        }
        for i in self.criterion {
            h.u64(i as u64);
        }
        fn constraints(v: &[Constraint], h: &mut FramedHasher) {
            h.u64(v.len() as u64);
            for c in v {
                h.u64(c.expression as u64)
                    .f64(c.lower)
                    .f64(c.upper)
                    .bool(c.strict);
            }
        }
        constraints(&self.eligibility, &mut h);
        fn obligation(o: &ProjectedObligation, h: &mut FramedHasher) {
            h.id(&o.instance)
                .id(&o.source)
                .str(match o.kind {
                    ObligationKind::Domain => "domain",
                    ObligationKind::Require(Condition::Positive) => "positive",
                    ObligationKind::Require(Condition::Nonnegative) => "nonnegative",
                    ObligationKind::Require(Condition::Nonzero) => "nonzero",
                })
                .bool(o.scope == ObligationScope::Unconditional)
                .bool(o.argument.is_some());
            if let Some(a) = o.argument {
                h.u64(a as u64);
            }
            h.bool(o.represented).u64(o.fidelity as u64);
            constraints(&o.constraints, h);
        }
        h.u64(self.obligations.len() as u64);
        for o in &self.obligations {
            obligation(o, &mut h);
        }
        h.u64(self.derivative_obligations.len() as u64);
        for (order, o) in &self.derivative_obligations {
            h.u64(*order as u64);
            obligation(o, &mut h);
        }
        h.finish_hash()
    }
    /// Complete owned projection extent; native transient work is admitted separately.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.nodes.capacity() * size_of::<Node>()
            + self.residuals.capacity() * size_of::<NodeId>()
            + self.eligibility.capacity() * size_of::<Constraint>()
            + self.obligations.capacity() * size_of::<ProjectedObligation>()
            + self.derivative_obligations.capacity()
                * size_of::<(DerivativeOrder, ProjectedObligation)>()
            + self
                .nodes
                .iter()
                .map(|node| match node {
                    Node::Sum(children) | Node::Product(children) => {
                        children.capacity() * size_of::<NodeId>()
                    }
                    Node::Const(constant)
                    | Node::Pow {
                        exponent: constant, ..
                    } => constant.allocated_bytes(),
                    _ => 0,
                })
                .sum::<usize>()
            + self
                .obligations
                .iter()
                .chain(
                    self.derivative_obligations
                        .iter()
                        .map(|(_, obligation)| obligation),
                )
                .map(|obligation| obligation.constraints.capacity() * size_of::<Constraint>())
                .sum::<usize>()
    }
}

/// Project a genuine single-root residual and its original guards for uniqueness
/// evidence. Neutral criterion transport expresses root uniqueness only; it creates
/// no numerical alternative, selection worker or approximation.
pub fn single_root_isolation_program(
    instance: SemanticId,
    residual: &PreparedBody,
    cancel: &Arc<AtomicBool>,
    max_nodes: usize,
) -> Result<Option<RootIsolationProgram>, FactorableError> {
    if cancel.load(Ordering::Relaxed) {
        return fail(MathError::Cancelled);
    }
    if max_nodes == 0 || residual.output_count() == 0 {
        return fail(MathError::Limit(EXTENT));
    }
    let request = FactorableRequest::default();
    let mut builder = Builder::new(&request, cancel, max_nodes, residual.slots)?;
    builder.exact_real = true;
    let inputs = (0..residual.input_count())
        .map(|column| Input::Column(column, 1.0, 0.0))
        .collect::<Vec<_>>();
    let outputs = (0..residual.output_count()).collect::<Vec<_>>();
    let projected = builder.instance(instance, residual, &inputs, &outputs)?;
    let residuals = outputs.iter().map(|o| projected[o]).collect();
    builder.establish_unconditional_obligations()?;
    if !builder.auxiliaries.is_empty()
        || !builder.implicit.is_empty()
        || builder
            .nodes
            .iter()
            .any(|n| matches!(n, Node::Aux(_) | Node::Abs(_)))
        || builder
            .obligations
            .iter()
            .chain(builder.derivative_obligations.iter().map(|(_, o)| o))
            .any(|o| o.scope != ObligationScope::Unconditional || !o.represented)
    {
        return Ok(None);
    }
    for obligation in &mut builder.obligations {
        obligation.fidelity = Fidelity::Exact;
    }
    for (_, obligation) in &mut builder.derivative_obligations {
        obligation.fidelity = Fidelity::Exact;
    }
    let zero = builder.constant(Constant::integer(0))?;
    builder.check()?;
    Ok(Some(RootIsolationProgram {
        inputs: residual.input_count(),
        nodes: builder.nodes,
        residuals,
        eligibility: Vec::new(),
        criterion: [zero, zero],
        obligations: builder.obligations,
        derivative_obligations: builder.derivative_obligations,
    }))
}

/// Project residuals, eligibility and score/tolerance without introducing auxiliaries.
///
/// Unsupported arithmetic, nonsmooth residuals, providers, conditional obligations
/// and predicates without an exact conjunctive representation return `Ok(None)`.
/// Resource limits, cancellation and invalid body layout remain explicit errors.
/// This makes no claim about the existence or uniqueness of roots, or about guard
/// discharge: a validated consumer must establish every retained exact condition.
/// # Errors
/// Unequal input arities, invalid eligibility or criterion output arity, cancellation,
/// or a node limit.
pub fn root_isolation_program(
    instance: SemanticId,
    residual: &PreparedBody,
    eligibility: &PreparedBody,
    criterion: &PreparedBody,
    cancel: &Arc<AtomicBool>,
    max_nodes: usize,
) -> Result<Option<RootIsolationProgram>, FactorableError> {
    root_isolation_program_for_outputs(
        instance,
        residual,
        &(0..residual.output_count()).collect::<Vec<_>>(),
        eligibility,
        criterion,
        cancel,
        max_nodes,
    )
}
/// Project explicitly selected compiler-owned residual outputs. All original value
/// and derivative guard obligations of that demanded source remain represented.
/// # Errors
/// Invalid output selection, interruption, bounds or unsupported exact projection.
pub fn root_isolation_program_for_outputs(
    instance: SemanticId,
    residual: &PreparedBody,
    residual_outputs: &[usize],
    eligibility: &PreparedBody,
    criterion: &PreparedBody,
    cancel: &Arc<AtomicBool>,
    max_nodes: usize,
) -> Result<Option<RootIsolationProgram>, FactorableError> {
    if residual_outputs.is_empty()
        || residual_outputs
            .iter()
            .any(|i| *i >= residual.output_count())
        || residual_outputs.iter().collect::<BTreeSet<_>>().len() != residual_outputs.len()
    {
        return fail(MathError::Contract(
            "selected residual output layout".into(),
        ));
    }
    if cancel.load(Ordering::Relaxed) {
        return fail(MathError::Cancelled);
    }
    if residual.input_count() != eligibility.input_count()
        || residual.input_count() != criterion.input_count()
        || eligibility.output_count() != 1
        || criterion.output_count() != 2
    {
        return fail(MathError::Contract("root isolation body layout".into()));
    }
    if max_nodes == 0 {
        return fail(MathError::Limit(EXTENT));
    }
    let request = FactorableRequest::default();
    let mut builder = Builder::new(
        &request,
        cancel,
        max_nodes,
        residual.slots.max(eligibility.slots).max(criterion.slots),
    )?;
    builder.exact_real = true;
    let inputs = (0..residual.input_count())
        .map(|column| Input::Column(column, 1.0, 0.0))
        .collect::<Vec<_>>();
    let outputs = residual_outputs.to_vec();
    let projected = builder.instance(instance, residual, &inputs, &outputs)?;
    let residuals = outputs.iter().map(|o| projected[o]).collect();
    // Preserve branch predicates symbolically: materializing an indicator would
    // replace its exact truth set with an unconstrained auxiliary.
    let formals = (0..residual.input_count())
        .map(|column| builder.push(Node::Var(column)))
        .collect::<Result<Vec<_>, _>>()?;
    let cx = Context {
        instance,
        local: false,
        record: true,
        depth: 0,
    };
    let mut env = Env::new(eligibility.slots);
    for (slot, &node) in formals.iter().enumerate() {
        env.assign(
            slot,
            Value::Node(node),
            Some(Arc::new(library::formal(slot)?)),
        )?;
    }
    builder.stages(&eligibility.demanded_stages(&[0])?, &mut env, cx)?;
    let predicate = env.read(eligibility.outputs[0])?;
    let constraints = match builder.holds(&predicate, Test::Positive)? {
        Truth::When(constraints, true) => constraints,
        Truth::Never => {
            let one = builder.constant(Constant::integer(1))?;
            vec![at_most(one, 0.0, false)]
        }
        Truth::When(_, false) => return Ok(None),
    };
    let projected_criterion = builder.instance(instance, criterion, &inputs, &[0, 1])?;
    let criterion = [projected_criterion[&0], projected_criterion[&1]];
    builder.establish_unconditional_obligations()?;
    if !builder.auxiliaries.is_empty()
        || !builder.implicit.is_empty()
        || builder
            .nodes
            .iter()
            .any(|n| matches!(n, Node::Aux(_) | Node::Abs(_)))
        || builder
            .obligations
            .iter()
            .chain(builder.derivative_obligations.iter().map(|(_, o)| o))
            .any(|o| o.scope != ObligationScope::Unconditional || !o.represented)
    {
        return Ok(None);
    }
    // Without auxiliaries every retained node is exact. Fidelity alone cannot
    // establish guard completeness; represented and unconditional were checked above.
    for obligation in &mut builder.obligations {
        obligation.fidelity = Fidelity::Exact;
    }
    for (_, obligation) in &mut builder.derivative_obligations {
        obligation.fidelity = Fidelity::Exact;
    }
    builder.check()?;
    Ok(Some(RootIsolationProgram {
        inputs: residual.input_count(),
        nodes: builder.nodes,
        residuals,
        eligibility: constraints,
        criterion,
        obligations: builder.obligations,
        derivative_obligations: builder.derivative_obligations,
    }))
}

/// Immutable factorable projection of one case under fixed consumed values.
#[derive(Clone, Debug)]
pub struct FactorableProgram {
    /// Whether native transport must refuse any further relaxation.
    pub require_exact: bool,
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
    /// Constraint forms left to native handlers, in structure order (ADR-0104).
    pub native: Vec<ProjectedNative>,
    fidelity: Vec<Fidelity>,
}
#[cfg(test)]
pub(crate) fn tape_construction_fixture(nodes: Vec<Node>) -> FactorableProgram {
    FactorableProgram {
        require_exact: false,
        key: ContentHash::from_bytes([0; 32]),
        structure: ContentHash::from_bytes([0; 32]),
        values: BTreeMap::new(),
        fidelity: vec![Fidelity::Exact; nodes.len()],
        nodes,
        variables: vec![],
        auxiliaries: vec![],
        rows: vec![],
        objective: None,
        obligations: vec![],
        implicit: vec![],
        native: vec![],
    }
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
                Node::Pow { base, exponent } => power(values[*base], exponent),
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
                    Node::Const(c) | Node::Pow { exponent: c, .. } => c.allocated_bytes(),
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
                        + i.selection.allocated_bytes()
                })
                .sum::<usize>()
            + self.values.len() * 64
            + self
                .native
                .iter()
                .map(|n| {
                    size_of_val(n)
                        + match n {
                            ProjectedNative::Indicator { .. } => 0,
                            ProjectedNative::Sos { members, .. } => {
                                members.capacity() * size_of::<(NativeOperand, f64)>()
                            }
                            ProjectedNative::Logic { operands, .. } => {
                                operands.capacity() * size_of::<(NativeOperand, bool)>()
                            }
                            ProjectedNative::Cardinality { members, .. } => {
                                members.capacity() * size_of::<NativeOperand>()
                            }
                        }
                })
                .sum::<usize>()
    }
}
pub(crate) fn power(base: f64, exponent: &Constant) -> f64 {
    match exponent {
        Constant::Rational(r) if r.is_integer() => {
            i32::try_from(r.numerator()).map_or_else(|_| base.powf(r.to_f64()), |n| base.powi(n))
        }
        Constant::Rational(r) if r == &Rational::new(1, 2) => base.sqrt(),
        other => base.powf(other.value()),
    }
}

impl PreparedBody {
    /// Known descriptor/container population of an exact root-isolation projection.
    /// Rational coefficient limbs and symbolic identities remain foreign allocations.
    /// The exact builder can lower a u64 dyadic denominator to at most 63 square
    /// roots and one integer power. Charge that source vertex before construction,
    /// rather than the policy's maximum node population.
    pub fn root_isolation_allocation_bound(
        &self,
        max_nodes: usize,
    ) -> Result<Option<usize>, MathError> {
        if !self.is_flat_arithmetic() {
            return Ok(None);
        }
        let overflow = || MathError::Limit("root isolation construction extent");
        let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
        let mut source_nodes = Some(0usize);
        for stage in &self.stages {
            let Stage::Block { expressions, .. } = stage else {
                // A Require may materialize its existing scalar and retain one
                // obligation with one closed constraint, not a new region.
                source_nodes = source_nodes.and_then(|count| count.checked_add(2));
                continue;
            };
            for atom in expressions {
                atom.as_view().visitor(&mut |view| {
                    let emitted = match view {
                        AtomView::Pow(power) if matches!(power.get_exp(), AtomView::Num(_)) => 64,
                        AtomView::Pow(_) => 3,
                        AtomView::Var(_) => 2,
                        _ => 1,
                    };
                    source_nodes = source_nodes.and_then(|count| count.checked_add(emitted));
                    source_nodes.is_some()
                });
            }
        }
        let all_nodes = add(
            add(source_nodes.ok_or_else(overflow)?, self.input_count())?,
            2,
        )?;
        let nodes = all_nodes.min(max_nodes);
        // Trees plus generated unary/root/Product edges; n-ary source operands
        // are bounded by visited vertices. Keep edges uncapped: a candidate's
        // operand Vec can be allocated before Builder::push refuses its node.
        let edges = mul(all_nodes, 2)?;
        let graph = add(
            mul(nodes, 4 * size_of::<Node>() + 256)?,
            mul(edges, 4 * size_of::<NodeId>() + 2 * size_of::<Constant>())?,
        )?;
        let requirements = self
            .stages
            .iter()
            .filter(|stage| matches!(stage, Stage::Require { .. }))
            .count();
        let metadata = add(
            add(
                mul(self.slot_count(), 2 * size_of::<Value>() + 128)?,
                mul(add(self.input_count(), self.output_count())?, 256)?,
            )?,
            mul(
                requirements,
                4 * size_of::<ProjectedObligation>() + 4 * size_of::<Constraint>(),
            )?,
        )?;
        Ok(Some(add(
            add(add(graph, metadata)?, mul(self.retained_bytes(), 4)?)?,
            4096,
        )?))
    }
}

impl CasePlan {
    /// Initial construction extent for a flat factorable export. Implicit residual
    /// substitution and provider envelopes have separate expansion populations;
    /// they remain conservative until their producer supplies that contract.
    /// # Errors
    /// Overflow in the source projection population.
    pub fn factorable_allocation_bound(
        &self,
        request: &FactorableRequest,
        limit: usize,
    ) -> Result<Option<usize>, MathError> {
        if !request.implicit.is_empty() || !request.envelopes.is_empty() {
            return Ok(None);
        }
        self.rebind_projection_allocation_bound(limit)
    }
    /// Known construction populations for the ordinary flat arithmetic projection.
    /// Count actual visited source vertices and instance bindings. A variable real
    /// power emits Log/Product/Exp; a scaled, offset column emits up to five nodes.
    /// Rational payloads remain the separate foreign-library allocation allowance.
    /// Returns `None` for control/provider lowering without this population contract.
    pub fn rebind_projection_allocation_bound(
        &self,
        limit: usize,
    ) -> Result<Option<usize>, MathError> {
        let overflow = || MathError::Limit("rebind projection construction extent");
        let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
        let mut source = 0usize;
        let mut vertices = 0usize;
        let mut bindings = 0usize;
        let mut environments = 0usize;
        let mut contributions = 0usize;
        let mut requirements = 0usize;
        for instance in self.structure().instances() {
            let body = &self.bodies()[&instance.body];
            if !body.is_flat_arithmetic() {
                return Ok(None);
            }
            source = add(source, body.retained_bytes())?;
            bindings = add(bindings, instance.slots.len())?;
            environments = add(environments, body.slot_count())?;
            contributions = add(contributions, instance.contributions.len())?;
            // Counting repeats is intentional: each semantic instance and expression
            // is lowered independently before node interning can discard duplicates.
            for stage in &body.stages {
                let Stage::Block { expressions, .. } = stage else {
                    requirements = add(requirements, 1)?;
                    vertices = add(vertices, 2)?;
                    continue;
                };
                for atom in expressions {
                    let mut count = Some(0usize);
                    atom.as_view().visitor(&mut |_| {
                        count = count.and_then(|count| count.checked_add(1));
                        count.is_some()
                    });
                    vertices = add(vertices, count.ok_or_else(overflow)?)?;
                }
            }
        }
        let rows = self.structure().rows().len();
        let columns = self.columns().len();
        let nodes = add(
            add(
                add(mul(vertices, 3)?, mul(bindings, 5)?)?,
                mul(contributions, 2)?,
            )?,
            add(rows, 2)?,
        )?
        .min(limit);
        // A source expression is a tree: operand edges <= vertices. Variable Pow
        // adds two edges and E can add one; four per vertex covers these exact
        // lowerings. Binding scale/offset uses four edges; contributions use two
        // scaling edges and one row/objective summation edge.
        let edges = add(
            add(mul(vertices, 4)?, mul(bindings, 4)?)?,
            mul(contributions, 3)?,
        )?;
        // Node Vec growth, interned Node clones and hash metadata; operand vectors
        // include their interned clones and the pre-push candidate/fold temporary.
        let graph = add(
            add(
                mul(nodes, 4 * size_of::<Node>() + 256)?,
                mul(
                    edges,
                    4 * size_of::<NodeId>() + 2 * size_of::<Constant>() + 2 * size_of::<Value>(),
                )?,
            )?,
            mul(source, 4)?,
        )?;
        // Negative integer Pow emits three native operations; n-ary operations
        // emit one per operand edge. A refused late row still constructs reachable,
        // pending edges, initial Vec capacity and the slot map before operation
        // budget refusal. Charge that candidate alongside all retained prior tapes.
        let candidate_operations = add(mul(nodes, 3)?, edges)?;
        let retained_operations = mul(candidate_operations, rows)?.min(limit);
        // Domain tape Vec capacity may double during push growth.
        let tapes = add(
            add(
                add(
                    mul(
                        retained_operations,
                        2 * size_of::<pounce_nlp::expression_provider::FbbtOp>(),
                    )?,
                    mul(
                        candidate_operations,
                        2 * size_of::<pounce_nlp::expression_provider::FbbtOp>(),
                    )?,
                )?,
                mul(nodes, 256 + 128)?,
            )?,
            mul(edges, 2 * size_of::<NodeId>())?,
        )?;
        // Curvature memoizes a rational quadratic form per node, including every
        // column pair. Arbitrary precision limbs remain foreign admission.
        let cells = mul(mul(add(columns, 1)?, add(columns, 1)?)?, add(nodes, 1)?)?;
        let curvature = add(
            mul(cells, 4 * size_of::<Rational>() + 256)?,
            mul(nodes, 512)?,
        )?;
        let population = add(
            add(add(rows, columns)?, self.structure().variables().len())?,
            add(
                self.structure().parameters().len(),
                self.structure().instances().len(),
            )?,
        )?;
        let metadata = add(
            mul(population, 1024)?,
            add(
                mul(environments, 2 * size_of::<Value>() + 128)?,
                mul(bindings, 256)?,
            )?,
        )?;
        let guards = mul(
            requirements,
            4 * size_of::<ProjectedObligation>() + 4 * size_of::<Constraint>(),
        )?;
        Ok(Some(add(
            add(add(add(add(graph, tapes)?, curvature)?, metadata)?, guards)?,
            4096,
        )?))
    }
    /// Project the case into a library-neutral factorable program under fixed consumed
    /// values. `limit` bounds the number of DAG nodes.
    ///
    /// # Errors
    /// Missing or nonfinite consumed values, cancellation, an inconsistent implicit
    /// definition or envelope ([`FactorableError::Math`]), or the typed refusal of the
    /// declared disjunctive branch policy ([`FactorableError::DisjunctiveBranch`]). An
    /// exhausted node budget returns [`MathError::Limit`] through [`FactorableError::Math`].
    pub fn factorable_program(
        &self,
        values: &CaseValues,
        request: &FactorableRequest,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<FactorableProgram, FactorableError> {
        self.factorable_program_core(values, request, limit, cancel, false)
            .map(|(program, _)| program)
    }
    fn factorable_program_core(
        &self,
        values: &CaseValues,
        request: &FactorableRequest,
        limit: usize,
        cancel: &Arc<AtomicBool>,
        exact_real: bool,
    ) -> Result<
        (
            FactorableProgram,
            Vec<(DerivativeOrder, ProjectedObligation)>,
        ),
        FactorableError,
    > {
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
                let Some(v) = declared.get(id) else {
                    if self
                        .structure()
                        .parameters()
                        .iter()
                        .any(|parameter| parameter.id == *id)
                    {
                        return Ok(ProjectedVariable {
                            id: *id,
                            domain: ModelingVariableDomain::Continuous,
                            lower: f64::NEG_INFINITY,
                            upper: f64::INFINITY,
                            active_lower: None,
                        });
                    }
                    return Err(MathError::Contract("column without a declaration".into()));
                };
                Ok(ProjectedVariable {
                    id: *id,
                    domain: v.domain,
                    lower: if v.domain.is_semi() {
                        0.0_f64.min(v.lower.unwrap_or(0.0))
                    } else {
                        v.lower.unwrap_or(f64::NEG_INFINITY)
                    },
                    upper: v.upper.unwrap_or(f64::INFINITY),
                    active_lower: v.lower.filter(|_| v.domain.is_semi()),
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
        builder.exact_real = exact_real;
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
                    if exact_real {
                        Input::ScaledValue(v, s.scale(), s.offset())
                    } else {
                        Input::Value(s.scale() * v + s.offset())
                    }
                });
            }
            let outputs: Vec<usize> = b
                .contributions
                .iter()
                .map(|c| c.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let projected = builder.instance(b.instance, body, &inputs, &outputs)?;
            for c in &b.contributions {
                let term = match projected.get(&c.output) {
                    Some(&node) => Some(builder.scaled(node, c.scale)?),
                    None => None,
                };
                match (c.target, term) {
                    (Target::Row(id), Some(t)) => terms[rows[&id]].push(t),
                    (Target::Row(id), None) => unavailable[rows[&id]] = true,
                    (Target::Objective(0), Some(t)) => objective_terms.push(t),
                    // A later lexicographic objective has no factorable projection; only
                    // a native lexicographic route optimizes it with the first.
                    (Target::Objective(_), _) => objective_unavailable = true,
                }
            }
        }
        let mut projected_rows = Vec::with_capacity(rows.len());
        for (r, row) in self.structure().rows().iter().enumerate() {
            let expression = if unavailable[r] {
                None
            } else {
                Some(builder.sum(std::mem::take(&mut terms[r]))?)
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
                    Some(builder.sum(objective_terms)?)
                },
                sense: o.sense,
                fidelity: Fidelity::Unavailable,
            }),
            None => None,
        };
        let native = project_native(self, &columns, &rows, values, &mut consumed)?;
        let mut h = FramedHasher::new(pse_ids::Frame::MathFactorableV2);
        h.hash(&self.structure().key())
            .str(match request.branches {
                BranchPolicy::Auxiliary => "branches:auxiliary",
                BranchPolicy::Disjunctive => "branches:disjunctive",
            })
            .u64(limit as u64)
            .bool(request.require_exact);
        for (id, bits) in &consumed {
            h.id(id).u64(*bits);
        }
        for (key, definition) in &request.implicit {
            h.hash(&key.0).u64(definition.unknowns.len() as u64);
            definition.selection.frame(&mut h);
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
        // Transport identity includes explicit node tags and arbitrary precision contents.
        // Neither library Hash nor Debug/Display of an expression is a durable identity.
        h.u64(builder.nodes.len() as u64);
        for node in &builder.nodes {
            match node {
                Node::Var(i) => {
                    h.str("var").u64(*i as u64);
                }
                Node::Aux(i) => {
                    h.str("aux").u64(*i as u64);
                }
                Node::Const(c) => {
                    h.str("constant");
                    c.frame(&mut h);
                }
                Node::Sum(children) | Node::Product(children) => {
                    h.str(if matches!(node, Node::Sum(_)) {
                        "sum"
                    } else {
                        "product"
                    })
                    .u64(children.len() as u64);
                    for child in children {
                        h.u64(*child as u64);
                    }
                }
                Node::Pow { base, exponent } => {
                    h.str("power").u64(*base as u64);
                    exponent.frame(&mut h);
                }
                Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
                    h.str(match node {
                        Node::Exp(_) => "exp",
                        Node::Log(_) => "log",
                        Node::Abs(_) => "abs",
                        Node::Sin(_) => "sin",
                        _ => "cos",
                    })
                    .u64(*i as u64);
                }
            }
        }
        for block in &builder.implicit {
            h.str("checked-selected-graph");
            block.selection.frame(&mut h);
        }
        if exact_real {
            h.str("actual-root-real-constants-and-derivative-guards");
            builder.establish_unconditional_obligations()?;
        }
        let mut derivative_obligations = builder.derivative_obligations;
        let mut program = FactorableProgram {
            key: h.finish_hash(),
            require_exact: request.require_exact,
            structure: self.structure().key(),
            values: consumed,
            nodes: builder.nodes,
            variables,
            auxiliaries: builder.auxiliaries,
            rows: projected_rows,
            objective,
            obligations: builder.obligations,
            implicit: builder.implicit,
            native,
            fidelity: vec![],
        };
        classify(&mut program);
        for (_, guard) in &mut derivative_obligations {
            classify_obligation(guard, &program.fidelity);
        }
        if request.require_exact && program.fidelity() != Fidelity::Exact {
            return Err(FactorableError::ExactRequired {
                fidelity: program.fidelity(),
            });
        }
        Ok((program, derivative_obligations))
    }
    /// Project original row/objective arithmetic, including finite bound subtraction,
    /// for outward point values and derivatives. First must be fully represented;
    /// a native Second request additionally consumes every retained Second guard.
    /// No provider envelope, lifted
    /// implicit graph, discrete column or native constraint supplies arithmetic authority.
    /// # Errors
    /// Source, cancellation and extent errors. Unsupported original projections return
    /// `None`, without manufacturing uncertainty from a tolerance or binary spacing.
    pub fn point_arithmetic_program(
        &self,
        values: &CaseValues,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<PointArithmeticProgram>, FactorableError> {
        self.point_arithmetic_program_for_order(values, DerivativeOrder::First, limit, cancel)
    }
    /// Project the same original DAG with admission restricted to the consumed order.
    /// Value-only box evaluation retains Value domains and does not demand First
    /// regularity. Higher guard inventories remain present for stronger native requests.
    /// # Errors
    /// Source, cancellation and bounded projection extent failures.
    pub fn point_arithmetic_program_for_order(
        &self,
        values: &CaseValues,
        requested_order: DerivativeOrder,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<PointArithmeticProgram>, FactorableError> {
        let (mut program, derivative_obligations) = self.factorable_program_core(
            values,
            &FactorableRequest::default(),
            limit,
            cancel,
            true,
        )?;
        let valid = |guard: &ProjectedObligation| {
            guard.represented
                && guard.fidelity == Fidelity::Exact
                && guard.scope == ObligationScope::Unconditional
        };
        if program.variables.is_empty()
            || (requested_order != DerivativeOrder::Value
                && program
                    .variables
                    .iter()
                    .any(|v| v.domain != ModelingVariableDomain::Continuous))
            || !program.auxiliaries.is_empty()
            || !program.implicit.is_empty()
            || !program.native.is_empty()
            || program.fidelity() != Fidelity::Exact
            || program.obligations.iter().any(|g| !valid(g))
            || derivative_obligations
                .iter()
                .any(|(order, g)| *order <= requested_order && !valid(g))
            || program
                .rows
                .iter()
                .any(|r| r.expression.is_none() || r.lower.is_nan() || r.upper.is_nan())
            || program
                .objective
                .as_ref()
                .is_some_and(|o| o.expression.is_none())
        {
            return Ok(None);
        }
        let mut outputs = Vec::new();
        let mut rows = Vec::with_capacity(program.rows.len());
        for row in &program.rows {
            if cancel.load(Ordering::Acquire) {
                return fail(MathError::Cancelled);
            }
            let expression = row
                .expression
                .ok_or_else(|| MathError::Contract("point arithmetic row missing".into()))?;
            let value = outputs.len();
            outputs.push(expression);
            let mut residual = |bound: f64| -> Option<usize> {
                if !bound.is_finite() {
                    return None;
                }
                let output = outputs.len();
                if bound == 0.0 {
                    outputs.push(expression);
                } else {
                    let constant = program.nodes.len();
                    program.nodes.push(Node::Const(Constant::Float(-bound)));
                    let node = program.nodes.len();
                    program.nodes.push(Node::Sum(vec![expression, constant]));
                    outputs.push(node);
                }
                Some(output)
            };
            let lower_residual = residual(row.lower);
            let upper_residual = if row.lower == row.upper {
                lower_residual
            } else {
                residual(row.upper)
            };
            rows.push(PointArithmeticRow {
                value,
                lower_residual,
                upper_residual,
            });
            if program.nodes.len() >= limit {
                return fail(MathError::Limit(EXTENT));
            }
        }
        let objective = if let Some(o) = &program.objective {
            let expression = o
                .expression
                .ok_or_else(|| MathError::Contract("point arithmetic objective missing".into()))?;
            let output = outputs.len();
            outputs.push(expression);
            Some(output)
        } else {
            None
        };
        if outputs.is_empty() {
            return Ok(None);
        }
        let zero = program.nodes.len();
        program.nodes.push(Node::Const(Constant::integer(0)));
        if program.nodes.len() > limit {
            return fail(MathError::Limit(EXTENT));
        }
        Ok(Some(PointArithmeticProgram {
            key: program.key,
            graph: RootIsolationProgram {
                inputs: program.variables.len(),
                nodes: program.nodes,
                residuals: outputs,
                eligibility: vec![],
                criterion: [zero, zero],
                obligations: program.obligations,
                derivative_obligations,
            },
            rows,
            objective,
        }))
    }

    /// Original exact-real Value projection for checking a native export's arithmetic
    /// correspondence. This retains the existing factorable vocabulary and original
    /// domains; it confers neither a solver certificate nor derivative regularity.
    /// # Errors
    /// The existing bounded projection's source, cancellation and extent failures.
    pub fn exact_value_factorable_program(
        &self,
        values: &CaseValues,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<FactorableProgram, FactorableError> {
        self.factorable_program_core(values, &FactorableRequest::default(), limit, cancel, true)
            .map(|(program, _)| program)
    }

    /// Project a genuine scalar root family using the same exact-real Builder as
    /// selected-root isolation. Original states retain their contiguous prefix;
    /// the declared external parameter follows them for the library adapter. A singleton zero-score
    /// union can establish regular RootSheet transport, never selector-minimum meaning.
    /// # Errors
    /// Invalid scalar inventory or finite interval, source/resource errors. Unsupported
    /// original domains/guards/relaxations return None rather than inventing proof support.
    pub fn root_path_isolation_program(
        &self,
        values: &CaseValues,
        parameter: SemanticId,
        interval: (f64, f64),
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<(ContentHash, RootIsolationProgram)>, FactorableError> {
        let n =
            self.columns().len().checked_sub(1).ok_or_else(|| {
                MathError::Contract("empty root path coordinate inventory".into())
            })?;
        if self.columns()[n] != parameter
            || !self
                .structure()
                .parameters()
                .iter()
                .any(|p| p.id == parameter)
            || !interval.0.is_finite()
            || !interval.1.is_finite()
            || interval.0 >= interval.1
        {
            return fail(MathError::Contract(
                "root path requires its declared last scalar and finite physical interval".into(),
            ));
        }
        let request = FactorableRequest::default();
        let (mut program, derivative_obligations) =
            self.factorable_program_core(values, &request, limit, cancel, true)?;
        let valid_guard = |g: &ProjectedObligation| {
            g.represented
                && g.fidelity == Fidelity::Exact
                && g.scope == ObligationScope::Unconditional
        };
        if program.rows.len() != n
            || program.objective.is_some()
            || !program.auxiliaries.is_empty()
            || !program.implicit.is_empty()
            || !program.native.is_empty()
            || program.fidelity() != Fidelity::Exact
            || program.variables[..n].iter().any(|v| {
                v.domain != ModelingVariableDomain::Continuous
                    || !v.lower.is_finite()
                    || !v.upper.is_finite()
                    || v.lower >= v.upper
            })
            || program
                .rows
                .iter()
                .any(|r| r.expression.is_none() || !r.lower.is_finite() || r.lower != r.upper)
            || program.obligations.iter().any(|g| !valid_guard(g))
            || derivative_obligations
                .iter()
                .any(|(order, g)| *order <= DerivativeOrder::First && !valid_guard(g))
        {
            return Ok(None);
        }
        if program
            .nodes
            .iter()
            .any(|node| matches!(node,Node::Var(column) if *column>n))
        {
            return fail(MathError::Contract(
                "root path factorable coordinate outside actual source".into(),
            ));
        }
        let mut residuals = Vec::with_capacity(n);
        for row in &program.rows {
            let expression = row
                .expression
                .ok_or_else(|| MathError::Contract("root path row expression missing".into()))?;
            if row.lower == 0. {
                residuals.push(expression);
            } else {
                let constant = program.nodes.len();
                program.nodes.push(Node::Const(Constant::Float(-row.lower)));
                let residual = program.nodes.len();
                program.nodes.push(Node::Sum(vec![expression, constant]));
                residuals.push(residual);
            }
        }
        let zero = program.nodes.len();
        program.nodes.push(Node::Const(Constant::integer(0)));
        if program.nodes.len() > limit {
            return fail(MathError::Limit(EXTENT));
        }
        Ok(Some((
            program.key,
            RootIsolationProgram {
                inputs: n + 1,
                nodes: program.nodes,
                residuals,
                eligibility: vec![],
                criterion: [zero, zero],
                obligations: program.obligations,
                derivative_obligations,
            },
        )))
    }
}

/// Native constraints over program coordinates; a fixed operand's value is consumed.
fn project_native(
    plan: &CasePlan,
    columns: &BTreeMap<SemanticId, usize>,
    rows: &BTreeMap<SemanticId, usize>,
    values: &CaseValues,
    consumed: &mut BTreeMap<SemanticId, u64>,
) -> Result<Vec<ProjectedNative>, MathError> {
    use pse_model::forms::NativeConstraint as C;
    let mut operand = |id: &SemanticId| -> Result<NativeOperand, MathError> {
        if let Some(&c) = columns.get(id) {
            return Ok(NativeOperand::Column(c));
        }
        let v = *values
            .scalars
            .get(id)
            .ok_or_else(|| MathError::Contract("missing fixed native operand".into()))?;
        if !v.is_finite() {
            return Err(MathError::Contract("nonfinite fixed native operand".into()));
        }
        consumed.insert(*id, v.to_bits());
        Ok(NativeOperand::Fixed(v))
    };
    plan.structure()
        .native()
        .iter()
        .map(|c| {
            Ok(match c {
                C::Indicator {
                    row,
                    variable,
                    active,
                } => ProjectedNative::Indicator {
                    row: *rows
                        .get(row)
                        .ok_or_else(|| MathError::Contract("native indicator row".into()))?,
                    indicator: operand(variable)?,
                    active: *active,
                },
                C::Sos { form, members } => ProjectedNative::Sos {
                    form: *form,
                    members: members
                        .iter()
                        .map(|(id, w)| Ok((operand(id)?, *w)))
                        .collect::<Result<_, MathError>>()?,
                },
                C::Logic {
                    form,
                    resultant,
                    operands,
                } => ProjectedNative::Logic {
                    form: *form,
                    resultant: operand(resultant)?,
                    operands: operands
                        .iter()
                        .map(|o| Ok((operand(&o.variable)?, o.negated)))
                        .collect::<Result<_, MathError>>()?,
                },
                C::Cardinality { members, bound } => ProjectedNative::Cardinality {
                    members: members.iter().map(&mut operand).collect::<Result<_, _>>()?,
                    bound: *bound,
                },
            })
        })
        .collect()
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
                .unwrap_or(Fidelity::Exact)
                .max(block.selection.fidelity());
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
        classify_obligation(o, &fidelity);
    }
    p.fidelity = fidelity;
}
fn classify_obligation(obligation: &mut ProjectedObligation, fidelity: &[Fidelity]) {
    let worst = obligation
        .constraints
        .iter()
        .map(|c| c.expression)
        .chain(obligation.argument)
        .map(|node| fidelity.get(node).copied().unwrap_or(Fidelity::Unavailable))
        .max()
        .unwrap_or(Fidelity::Exact);
    obligation.fidelity = if obligation.represented {
        worst
    } else {
        Fidelity::Relaxed.max(worst)
    };
}

/// A factorable projection failure.
#[derive(Debug, thiserror::Error)]
pub enum FactorableError {
    /// Admission, consumed values, cancellation or a resource bound.
    #[error(transparent)]
    Math(#[from] MathError),
    /// An exact-only consumer cannot use a relaxed or unavailable projection.
    #[error("exact factorable export required; projection is {fidelity:?}")]
    ExactRequired {
        /// Actual complete projection fidelity.
        fidelity: Fidelity,
    },
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
        Self::DisjunctiveBranch { .. } | Self::ExactRequired { .. } => Some(pse_diagnostics::DiagnosticCode::CompileMath),
    } },
    forward(this) { match this { Self::Math(e) => Some(e), Self::DisjunctiveBranch { .. } | Self::ExactRequired { .. } => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl pse_model::diagnostic::DiagnosticProjection for FactorableError {}
fn fail<T>(error: MathError) -> Result<T, FactorableError> {
    Err(FactorableError::Math(error))
}
const EXTENT: &str = "factorable projection extent";

#[derive(Clone, Copy, Debug)]
enum Input {
    Column(usize, f64, f64),
    Value(f64),
    ScaledValue(f64, f64, f64),
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
    calls: HashMap<Call, Vec<NodeId>>,
    selects: HashMap<usize, NodeId>,
    next_select: usize,
    /// Validated projection preserves exact real arithmetic and guard bounds.
    exact_real: bool,
    derivative_obligations: Vec<(DerivativeOrder, ProjectedObligation)>,
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
            calls: HashMap::new(),
            selects: HashMap::new(),
            next_select: 0,
            exact_real: false,
            derivative_obligations: vec![],
        })
    }
    fn check(&self) -> Result<(), FactorableError> {
        if self.cancel.load(Ordering::Relaxed) {
            fail(MathError::Cancelled)
        } else {
            Ok(())
        }
    }
    /// A branch-local guard is unconditional only when the same complete guard is
    /// already enforced at every evaluation, at no later derivative order. Preserve
    /// each authored occurrence; equal closed bounds alone do not cover a different
    /// Require kind or argument (notably the constraint-free nonzero condition).
    fn establish_unconditional_obligations(&mut self) -> Result<(), FactorableError> {
        let covers = |guard: &ProjectedObligation, other: &ProjectedObligation| {
            guard.represented
                && other.represented
                && other.scope == ObligationScope::Unconditional
                && guard.instance == other.instance
                && guard.kind == other.kind
                && guard.argument == other.argument
                && guard.constraints == other.constraints
        };
        for index in 0..self.obligations.len() {
            self.check()?;
            let guard = &self.obligations[index];
            if guard.scope == ObligationScope::Conditional
                && self.obligations.iter().any(|other| covers(guard, other))
            {
                self.obligations[index].scope = ObligationScope::Unconditional;
            }
        }
        for index in 0..self.derivative_obligations.len() {
            self.check()?;
            let (minimum, guard) = &self.derivative_obligations[index];
            if guard.scope == ObligationScope::Conditional
                && (self.obligations.iter().any(|other| covers(guard, other))
                    || self
                        .derivative_obligations
                        .iter()
                        .any(|(order, other)| order <= minimum && covers(guard, other)))
            {
                self.derivative_obligations[index].1.scope = ObligationScope::Unconditional;
            }
        }
        Ok(())
    }
    fn constant_of(&self, node: NodeId) -> Option<Constant> {
        match self.nodes.get(node) {
            Some(Node::Const(c)) => Some(c.clone()),
            _ => None,
        }
    }
    fn value_constant(&self, value: &Value) -> Option<f64> {
        match value {
            Value::Node(n) => self.constant_of(*n).map(|c| c.value()),
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
            Node::Sum(c) => self.fold(c, |a, b| a + b, |a, b| a + b, 0),
            Node::Product(c) => self.fold(c, |a, b| a * b, |a, b| a * b, 1),
            Node::Pow {
                base,
                exponent: Constant::Rational(exponent),
            } if exponent.is_integer() => {
                match self.constant_of(*base) {
                    Some(Constant::Rational(base)) => {
                        let integer = i64::try_from(exponent.numerator()).ok();
                        integer.and_then(|n| {
                            // Folding is optional: retain the exact power node when a
                            // constant expansion would exceed the bounded expression budget.
                            let bits = base
                                .numerator_ref()
                                .significant_bits()
                                .saturating_add(base.denominator_ref().significant_bits());
                            if bits.saturating_mul(n.unsigned_abs()) > (DEFINITION_BYTES as u64) * 8
                            {
                                return None;
                            }
                            let powered = base.pow(n.unsigned_abs());
                            if n < 0 && powered.is_zero() {
                                None
                            } else if n < 0 {
                                Some(Constant::Rational(Rational::one() / powered))
                            } else {
                                Some(Constant::Rational(powered))
                            }
                        })
                    }
                    _ => None,
                }
            }
            Node::Abs(i) => self.constant_of(*i).map(|c| match c {
                Constant::Rational(r) => Constant::Rational(r.abs()),
                Constant::Float(v) => Constant::Float(v.abs()),
            }),
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
        rational: fn(&Rational, &Rational) -> Rational,
        float: fn(f64, f64) -> f64,
        unit: i64,
    ) -> Option<Constant> {
        let constants = children
            .iter()
            .map(|&c| self.constant_of(c))
            .collect::<Option<Vec<_>>>()?;
        if self.exact_real && constants.iter().any(|c| matches!(c, Constant::Float(_))) {
            return None;
        }
        Constant::fold(&constants, rational, float, unit)
    }
    /// Exact stored binary coefficients use the library's rational conversion.
    /// The ordinary numerical export keeps its existing binary64 conversion.
    fn coefficient(&self, view: CoefficientView<'_>) -> Option<Constant> {
        if self.exact_real
            && let CoefficientView::Float(re, im) = view
        {
            return im
                .is_zero()
                .then(|| re.to_float().try_to_rational())
                .flatten()
                .map(Constant::Rational);
        }
        constant(view)
    }
    fn constant_power(
        &mut self,
        base: NodeId,
        exponent: Constant,
        cx: Context,
    ) -> Result<NodeId, FactorableError> {
        if self.exact_real
            && let Constant::Rational(rational) = &exponent
            && !rational.is_integer()
            && rational != &Rational::new(1, 2)
        {
            // General real powers already require a strictly positive base in typed
            // source programs. Use that actual retained value guard, never a new
            // assumption, to project arbitrary rational powers through exp/log.
            let positive = self.obligations.iter().any(|guard| {
                guard.instance == cx.instance
                    && guard.represented
                    && guard.scope == ObligationScope::Unconditional
                    && guard.kind == ObligationKind::Require(Condition::Positive)
                    && guard.argument == Some(base)
            });
            if positive {
                let log = self.push(Node::Log(base))?;
                let exponent = self.constant(exponent)?;
                let product = self.push(Node::Product(vec![exponent, log]))?;
                return self.push(Node::Exp(product));
            }
            // A raw dyadic power may also admit zero. Repeated square roots followed
            // by the exact integer numerator retain that boundary; negative powers
            // retain their pole. Other unguarded rational domains remain unchanged.
            if let (Ok(denominator), Ok(numerator)) = (
                u64::try_from(rational.denominator()),
                i32::try_from(rational.numerator()),
            ) && denominator.is_power_of_two()
            {
                let mut root = base;
                for _ in 0..denominator.trailing_zeros() {
                    root = self.push(Node::Pow {
                        base: root,
                        exponent: Constant::Rational(Rational::new(1, 2)),
                    })?;
                }
                return self.push(Node::Pow {
                    base: root,
                    exponent: Constant::integer(i64::from(numerator)),
                });
            }
        }
        self.push(Node::Pow { base, exponent })
    }
    fn value_is_zero(&self, value: &Value) -> bool {
        if !self.exact_real {
            return self.value_constant(value) == Some(0.0);
        }
        match value {
            Value::Node(node) => match self.constant_of(*node) {
                Some(Constant::Rational(r)) => r.is_zero(),
                Some(Constant::Float(v)) => v == 0.0,
                _ => false,
            },
            _ => false,
        }
    }
    fn nonnegative_outcomes(&self, value: &Value) -> bool {
        match value {
            Value::Node(node) => match self.constant_of(*node) {
                Some(Constant::Rational(r)) => r >= 0,
                Some(Constant::Float(v)) => v.is_finite() && v >= 0.0,
                _ => false,
            },
            Value::Select(s) => {
                self.nonnegative_outcomes(&s.then) && self.nonnegative_outcomes(&s.otherwise)
            }
            Value::Terms(terms) => terms.iter().all(|v| self.nonnegative_outcomes(v)),
            Value::Unset => false,
        }
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
                Input::ScaledValue(value, scale, offset) => {
                    let v = self.float(value)?;
                    let v = self.scaled(v, scale)?;
                    if offset == 0.0 {
                        v
                    } else {
                        let o = self.float(offset)?;
                        self.push(Node::Sum(vec![v, o]))?
                    }
                }
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
                    ..
                } => {
                    // Ordinary export retains only the value domain. Root projection
                    // separately retains each derivative guard's minimum admission order.
                    if cx.record && (*order == DerivativeOrder::Value || self.exact_real) {
                        let value = env.read(*argument)?;
                        let node = self.materialize(&value)?;
                        let truth = match (self.constant_of(node), condition) {
                            (Some(Constant::Rational(r)), condition) if self.exact_real => {
                                let permitted = match condition {
                                    Condition::Positive => r > 0,
                                    Condition::Nonnegative => r >= 0,
                                    Condition::Nonzero => !r.is_zero(),
                                };
                                if permitted {
                                    Truth::ALWAYS
                                } else {
                                    Truth::Never
                                }
                            }
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
                        if self.exact_real
                            && *order > DerivativeOrder::Value
                            && let Some(obligation) = self.obligations.pop()
                        {
                            self.derivative_obligations.push((*order, obligation));
                        }
                    }
                }
                Stage::Applicability {
                    stages,
                    predicates,
                    token,
                    plan,
                    ..
                } => {
                    let mut local = env.clone();
                    self.stages(stages, &mut local, cx)?;
                    if cx.record {
                        let truth = self.applicability_truth(plan, predicates, &local)?;
                        self.obligation(
                            cx,
                            plan.claim.id.unwrap_or(plan.claim.form),
                            ObligationKind::Domain,
                            None,
                            truth,
                        )?;
                    }
                    let zero = self.constant(Constant::integer(0))?;
                    env.assign(*token, Value::Node(zero), Some(Arc::new(Atom::num(0))))?;
                }
                Stage::Domain {
                    stages,
                    argument,
                    token,
                    lineage,
                } => {
                    let mut local = env.clone();
                    self.stages(stages, &mut local, cx)?;
                    if cx.record {
                        let predicate = local.read(*argument)?;
                        let truth = self.holds(&predicate, Test::Positive)?;
                        self.obligation(cx, lineage.source, ObligationKind::Domain, None, truth)?;
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
    fn applicability_truth(
        &mut self,
        plan: &pse_model::applicability::Node,
        predicates: &[usize],
        env: &Env,
    ) -> Result<Truth, FactorableError> {
        use pse_model::applicability::Region;
        let permissions = plan
            .permissions
            .iter()
            .filter(|p| p.covers(&plan.claim))
            .collect::<Vec<_>>();
        let unknown = permissions.iter().any(|p| p.allow_unknown);
        let extrapolation = permissions.iter().any(|p| p.allow_extrapolation);
        let mut truth = match &plan.region {
            Region::Unrestricted => Truth::ALWAYS,
            Region::Unknown => {
                if unknown {
                    Truth::ALWAYS
                } else {
                    Truth::Never
                }
            }
            Region::Predicate(index) => {
                if extrapolation {
                    Truth::ALWAYS
                } else {
                    let slot = predicates
                        .get(*index)
                        .ok_or_else(|| MathError::Contract("claim predicate index".into()))?;
                    self.holds(&env.read(*slot)?, Test::Positive)?
                }
            }
            // Existing factorable exports represent conjunctions exactly and report
            // disjunctive obligations as incomplete. Never invent a global union proof.
            Region::Union(_) => Truth::UNKNOWN,
        };
        for dependency in &plan.dependencies {
            truth = truth.and(self.applicability_truth(dependency, predicates, env)?);
        }
        Ok(truth)
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
            AtomView::Num(n) => match self.coefficient(n.get_coeff_view()) {
                Some(c) => self.constant(c),
                None => self.opaque(cx, Opacity::Constant),
            },
            AtomView::Var(v) => {
                let s = v.get_symbol();
                if let Some(&k) = self.symbols.get(&s) {
                    let value = env.read(k)?;
                    self.materialize(&value)
                } else if s == Symbol::E && self.exact_real {
                    let one = self.constant(Constant::integer(1))?;
                    self.push(Node::Exp(one))
                } else if s == Symbol::PI && self.exact_real {
                    self.opaque(cx, Opacity::Constant)
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
                    return match self.coefficient(n.get_coeff_view()) {
                        Some(c) => self.constant_power(b, c, cx),
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
                        exponent: Constant::Rational(Rational::new(1, 2)),
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
        if self.exact_real
            && let (Value::Node(a), Value::Node(b)) = (&l, &r)
            && let (Some(Constant::Rational(a)), Some(Constant::Rational(b))) =
                (self.constant_of(*a), self.constant_of(*b))
        {
            let selected = match comparison {
                Comparison::Eq => a == b,
                Comparison::Ne => a != b,
                Comparison::Lt => a < b,
                Comparison::Le => a <= b,
            };
            return self.stages(if selected { then } else { otherwise }, env, cx);
        }
        if !self.exact_real
            && let (Some(a), Some(b)) = (self.value_constant(&l), self.value_constant(&r))
        {
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
            let exact = if pure
                && (!self.exact_real || matches!(comparison, Comparison::Lt | Comparison::Le))
            {
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
        let (Some(l), Some(r)) = (self.identity_definition(l)?, self.identity_definition(r)?)
        else {
            return Ok(None);
        };
        let separation = (&l - &r).expand();
        if separation.is_constant() {
            return Ok(None);
        }
        let difference = (&x - &y).expand();
        self.check()?;
        // Each candidate is proved by expansion, never sampled. In exact-real mode all
        // coefficients were converted before substitution or proof arithmetic, so floating
        // cancellation cannot establish an identity for the outward arithmetic graph.
        let Some(alpha) = IDENTITY_FACTORS
            .iter()
            .find_map(|&(numerator, denominator)| {
                let factor = Atom::num(numerator) / Atom::num(denominator);
                (&difference - &(&factor * &separation))
                    .expand()
                    .is_zero()
                    .then_some(Constant::Rational(Rational::new(numerator, denominator)))
            })
        else {
            return Ok(None);
        };
        let xn = self.materialize(&a.values[slot])?;
        let yn = self.materialize(&b.values[slot])?;
        let ln = self.materialize(&parent.values[left])?;
        let rn = self.materialize(&parent.values[right])?;
        let half = self.constant(Constant::Rational(Rational::new(1, 2)))?;
        let total = self.push(Node::Sum(vec![xn, yn]))?;
        let mean = self.push(Node::Product(vec![half, total]))?;
        if alpha.value() == 0.0 {
            return Ok(Some(mean));
        }
        let gap = self.difference(ln, rn)?;
        let magnitude = self.push(Node::Abs(gap))?;
        let coefficient = match alpha {
            Constant::Rational(r) => Constant::Rational(&r * &Rational::new(-1, 2)),
            Constant::Float(v) => Constant::Float(-v / 2.0),
        };
        let coefficient = self.constant(coefficient)?;
        let term = self.push(Node::Product(vec![coefficient, magnitude]))?;
        self.push(Node::Sum(vec![mean, term])).map(Some)
    }
    /// Preserve the same exact coefficients as the arithmetic projection before the
    /// library normalizes a branch proof. Conversion after subtraction could already
    /// have lost a small term next to a large binary floating coefficient.
    fn identity_definition(&self, atom: &Atom) -> Result<Option<Atom>, FactorableError> {
        self.check()?;
        if atom.as_view().get_byte_size() > DEFINITION_BYTES {
            return Ok(None);
        }
        if !self.exact_real {
            return Ok(Some(atom.clone()));
        }
        let supported = Cell::new(true);
        let exact = atom.map_coefficient(|coefficient| {
            if matches!(coefficient, CoefficientView::Float(..)) {
                if let Some(Constant::Rational(rational)) = self.coefficient(coefficient) {
                    return Coefficient::from(rational);
                }
                supported.set(false);
            }
            coefficient.to_owned()
        });
        self.check()?;
        Ok(
            (supported.get() && exact.as_view().get_byte_size() <= DEFINITION_BYTES)
                .then_some(exact),
        )
    }
    /// Substitute definitions of region-local slots so that the identity compares
    /// expressions over values shared with the parent.
    fn region_definition(
        &self,
        atom: &Atom,
        region: &Env,
        parent: &Env,
    ) -> Result<Option<Atom>, FactorableError> {
        let Some(mut atom) = self.identity_definition(atom)? else {
            return Ok(None);
        };
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
                let Some(definition) = self.identity_definition(&definition)? else {
                    return Ok(None);
                };
                atom = atom.replace(library::formal(k)?).with(definition);
                self.check()?;
                if atom.as_view().get_byte_size() > DEFINITION_BYTES {
                    return Ok(None);
                }
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
        let selection = definition.selection.checked(residual, n)?;
        let first = self.auxiliaries.len();
        let mut unknowns = Vec::with_capacity(n);
        for (k, &(mut lower, mut upper)) in definition.unknowns.iter().enumerate() {
            let sign = match selection {
                SelectedGraph::RestrictedSquareRoot { positive, .. } => Some(positive),
                SelectedGraph::NondegenerateAffine { sign, .. } => sign,
                _ => None,
            };
            if let Some(positive) = sign {
                if positive {
                    lower = lower.max(0.0);
                } else {
                    upper = upper.min(0.0);
                }
            }
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
                    match self.constant_of(bound).map(|c| c.value()) {
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
        // Keep the selector's open/closed domain in transport, independently of the
        // closed auxiliary box. A native backend must label its closure as relaxation.
        let selected_sign = match selection {
            SelectedGraph::RestrictedSquareRoot { positive, strict } => Some((positive, strict)),
            SelectedGraph::NondegenerateAffine {
                sign: Some(positive),
                strict,
            } => Some((positive, strict)),
            _ => None,
        };
        if let Some((positive, strict)) = selected_sign {
            for &unknown in &unknowns {
                bounds.push(if positive {
                    at_least(unknown, 0.0, strict)
                } else {
                    at_most(unknown, 0.0, strict)
                });
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
            fidelity: selection.fidelity(),
            selection,
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
                Some(Constant::Rational(r)) if self.exact_real => {
                    let accepted = match test {
                        Test::Positive => r > 0,
                        Test::Zero => r.is_zero(),
                    };
                    if accepted {
                        Truth::ALWAYS
                    } else {
                        Truth::Never
                    }
                }
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
                    && terms.iter().all(|t| {
                        if self.exact_real {
                            self.nonnegative_outcomes(t)
                        } else {
                            self.range(t).is_some_and(|(lower, _)| lower >= 0.0)
                        }
                    })
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
            if self.value_is_zero(&s.right) {
                return self.holds(&s.left, Test::Zero);
            }
            if self.value_is_zero(&s.left) {
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
        if self.exact_real {
            // A rational or stored binary bound must not be rounded into the
            // binary64 Constraint envelope. Keep both operands in the DAG.
            let gap = self.difference(small, large)?;
            return Ok(Truth::When(vec![at_most(gap, 0.0, strict)], true));
        }
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
        CoefficientView::Natural(n, d, 0, _) => {
            (d != 0).then(|| Constant::Rational(Rational::new(n, d)))
        }
        CoefficientView::Float(re, im) if im.is_zero() => {
            let v = re.to_float().to_f64();
            v.is_finite().then_some(Constant::Float(v))
        }
        CoefficientView::Large(re, im) if im.is_zero() => Some(Constant::Rational(re.to_rat())),
        _ => None,
    }
}

#[cfg(test)]
mod exact_constant_tests {
    use super::*;
    #[test]
    fn flat_projection_accounts_for_variable_power_and_scaled_binding_populations() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let body = PreparedBody::new(
            2,
            3,
            vec![2],
            vec![Stage::Block {
                expressions: vec![x.pow(&y)],
                outputs: vec![2],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let request = FactorableRequest::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = Builder::new(&request, &cancel, 64, 3).unwrap();
        let output = builder
            .instance(
                SemanticId::NIL,
                &body,
                &[Input::Column(0, 2.0, 3.0), Input::Column(0, 4.0, 5.0)],
                &[0],
            )
            .unwrap();
        let root = output[&0];
        assert!(matches!(builder.nodes[root], Node::Exp(_)));
        assert!(
            builder
                .nodes
                .iter()
                .any(|node| matches!(node, Node::Log(_)))
        );
        // One initial zero, <=five nodes per binding, <=three per source vertex.
        assert!(builder.nodes.len() <= 1 + 2 * 5 + 3 * 3);
        assert_eq!(
            builder
                .nodes
                .iter()
                .filter(|node| matches!(node, Node::Var(_)))
                .count(),
            1
        );
    }

    #[test]
    fn exact_branch_identity_converts_floating_coefficients_before_expansion() {
        crate::initialize().unwrap();
        let request = FactorableRequest::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = Builder::new(&request, &cancel, 32, 0).unwrap();
        builder.exact_real = true;
        let x = library::formal(0).unwrap();
        // The binary floating product rounds to one, but its exact dyadic product
        // differs from one. That residual must prevent a purported branch identity.
        let expression = Atom::num(1e16) * (&x + Atom::num(1e-16)) - Atom::num(1);
        let exact = builder.identity_definition(&expression).unwrap().unwrap();
        assert!(
            !(exact - Atom::num(10_000_000_000_000_000_i64) * x)
                .expand()
                .is_zero()
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            builder.identity_definition(&expression),
            Err(FactorableError::Math(MathError::Cancelled))
        ));
    }
    #[test]
    fn validated_projection_does_not_round_binary64_constant_arithmetic() {
        crate::initialize().unwrap();
        let request = FactorableRequest::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = Builder::new(&request, &cancel, 32, 0).unwrap();
        builder.exact_real = true;
        let large = builder.float(1e16).unwrap();
        let one = builder.float(1.0).unwrap();
        let sum = builder.sum(vec![large, one]).unwrap();
        assert!(matches!(builder.nodes[sum], Node::Sum(_)));
        let product = builder.push(Node::Product(vec![large, one])).unwrap();
        assert!(matches!(builder.nodes[product], Node::Product(_)));
    }
    #[test]
    fn rational_folding_exceeds_machine_integer_range_without_rounding() {
        let max = Constant::Rational(Rational::from(i64::MAX));
        let sum =
            Constant::fold(&[max.clone(), max.clone()], |a, b| a + b, |a, b| a + b, 0).unwrap();
        let product = Constant::fold(&[sum, max], |a, b| a * b, |a, b| a * b, 1).unwrap();
        assert_eq!(
            product,
            Constant::Rational(Rational::from(i64::MAX).pow(2) * Rational::from(2))
        );
    }
    #[test]
    fn rational_identity_normalizes_ratios_and_retains_large_adjacent_integers() {
        let key = |constant: Constant| {
            let mut frame = FramedHasher::new(pse_ids::Frame::MathFactorableV2);
            constant.frame(&mut frame);
            frame.finish_hash()
        };
        assert_eq!(
            key(Constant::Rational(Rational::new(2, 6))),
            key(Constant::Rational(Rational::new(1, 3)))
        );
        // Fixed canonical preimage vector, independent of library printing/Hash.
        let golden = pse_ids::derive_hash(
            pse_ids::Frame::MathFactorableV2,
            &[b"rational:decimal:v1", b"1", b"3"],
        );
        assert_eq!(key(Constant::Rational(Rational::new(1, 3))), golden);
        assert_ne!(
            golden,
            pse_ids::derive_hash(
                pse_ids::Frame::MathFactorableV1,
                &[b"rational:decimal:v1", b"1", b"3"]
            )
        );
        let huge = Rational::from(i64::MAX).pow(4);
        let adjacent = &huge + &Rational::one();
        assert_eq!(huge.to_f64(), adjacent.to_f64());
        assert_ne!(
            key(Constant::Rational(huge)),
            key(Constant::Rational(adjacent))
        );
    }
}
