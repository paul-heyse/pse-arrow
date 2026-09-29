// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The factorable representation and its runner (ADR-0105). A global backend receives the
//! neutral [`FactorableProgram`] over original case columns; the runner re-qualifies every
//! candidate against the original compiled model and, where the relaxation-soundness rule
//! requires it, takes the candidate from a continuous re-solve with the backend's discrete
//! assignment fixed, run through the one NLP runner.
//!
//! Admission and the export plan are library-neutral and testable without a backend:
//! - every exported function is a row with a projection, an unconditional obligation
//!   (strict bounds closed by [`STRICT_MARGIN`]), or an implicit residual (`== 0`) or bound;
//! - rows without a projection are dropped, which keeps the export a sound relaxation;
//! - a semi column is lowered by the named, exact `semi(indicator)` transformation
//!   (ADR-0103 Outcome 7, ADR-0104): a binary indicator coordinate `z` and the links
//!   `x − u·z ≤ 0` and `x − l·z ≥ 0` over the box `[0, u]`;
//! - an unavailable objective, a variable or auxiliary of a nonlinear term without a finite
//!   box, a semi domain without a finite active interval or a nonfinite constant refuses
//!   with a typed [`Refusal`].
use super::{
    BackendExecution, BackendSettings, Input, LINKED, Nlp, OriginalModel, Problem, Retained, Step,
    nlp,
};
use crate::{
    NlpOracle, ProblemError, presolve,
    quality::{self, Observation, Quality, Violation},
    routing::{self, Requirements, Route},
    solve::{
        BoundSource, Compatibility, Controls, ExportTransformation, Metric, Options,
        PrimalSource, SolveIntent, SolveReport, SolverSelection, WarmPayload, WarmStart,
    },
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_math::{
    binding::ObjectiveSense,
    factorable::{
        BoundOwner, Constraint, FactorableProgram, Fidelity, MissingBound, NativeOperand, Node,
        NodeId, ProjectedNative,
    },
};
use pse_model::generated::enums::ModelingVariableDomain;
use std::collections::BTreeMap;

/// Relative margin that closes a strict obligation bound: `x > b` exports as
/// `x >= b + STRICT_MARGIN * max(1, |b|)`, and `x < b` symmetrically.
pub(crate) const STRICT_MARGIN: f64 = 1e-9;
/// Largest affine form tracked for one node; larger forms export as expressions.
const AFFINE_TERMS: usize = 4096;

/// Why a factorable program cannot be exported to a global backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The objective has no projection, or depends on an unbounded opaque auxiliary, so no
    /// bound exists (ADR-0105 §2).
    ObjectiveUnavailable,
    /// A variable or auxiliary inside a nonlinear term has no finite box; spatial
    /// branch-and-bound needs one.
    UnboundedNonlinear(MissingBound),
    /// A semi domain without a finite active interval `[l, u]` with `0 < l`, which the
    /// `semi(indicator)` lowering needs; domain admission establishes it for every case.
    SemiInterval(SemanticId),
    /// A constant or exponent outside finite representation.
    Constant(NodeId),
    /// A row is conditional on more than one indicator; a native handler takes one.
    Conjunctive(SemanticId),
    /// A native logic or indicator operand is not a binary column or a fixed 0/1 value.
    NativeOperand(usize),
}
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectiveUnavailable => {
                f.write_str("the objective has no factorable projection, so no bound exists")
            }
            Self::UnboundedNonlinear(m) => {
                match m.owner {
                    BoundOwner::Variable(id) => write!(f, "variable {id}")?,
                    BoundOwner::Auxiliary(k) => write!(f, "auxiliary {k}")?,
                }
                let side = match (m.lower, m.upper) {
                    (true, true) => "lower and upper bounds",
                    (true, false) => "a lower bound",
                    (false, _) => "an upper bound",
                };
                write!(
                    f,
                    " enters a nonlinear term without {side}; spatial branching needs a finite box"
                )
            }
            Self::SemiInterval(id) => write!(
                f,
                "variable {id} has a semi domain without a finite active interval [l, u] with 0 < l, which the semi(indicator) lowering needs"
            ),
            Self::Constant(node) => write!(f, "node {node} holds a nonfinite constant"),
            Self::Conjunctive(row) => write!(
                f,
                "row {row} is conditional on more than one indicator; a native indicator takes one"
            ),
            Self::NativeOperand(k) => write!(
                f,
                "native constraint {k} needs binary operands (a binary column or a fixed 0 or 1)"
            ),
        }
    }
}

/// Where an exported function comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Origin {
    /// A selected row.
    Row(usize),
    /// An unconditional obligation's closed constraint.
    Obligation(usize, usize),
    /// An implicit block's residual, constrained to zero.
    Residual(usize, usize),
    /// An implicit block's declared bound.
    ImplicitBound(usize, usize),
    /// A `semi(indicator)` link of the lowered semi column at this ordinal of
    /// [`Plan::semi`]: `x − u·z ≤ 0` when `upper`, otherwise `x − l·z ≥ 0`.
    SemiLink { semi: usize, upper: bool },
}
/// What an exported function evaluates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expression {
    /// A node of the program.
    Node(NodeId),
    /// A `semi(indicator)` link, affine in the combined coordinates: its ordinal in
    /// [`Plan::links`].
    Link(usize),
}
/// A semi column lowered by `semi(indicator)` (ADR-0103 Outcome 7, ADR-0104): its box is
/// `[0, upper]` with its integrality kept, its binary indicator coordinate follows the
/// auxiliaries, and the links `x − upper·z ≤ 0` and `x − lower·z ≥ 0` make `z = 0` fix the
/// column at zero and `z = 1` keep it in its active interval. The lowering is exact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Semi {
    /// The column, in program order.
    pub column: usize,
    /// Active interval `[lower, upper]`, `0 < lower <= upper`.
    pub lower: f64,
    pub upper: f64,
}
impl Semi {
    /// Whether `x` lies nearer the active interval than the zero branch.
    pub(crate) fn active(&self, x: f64) -> bool {
        quality::interval(x, self.lower, self.upper) < x.abs()
    }
    /// Physical distance from `x` to the domain `{0} ∪ [lower, upper]`.
    pub(crate) fn violation(&self, x: f64) -> f64 {
        x.abs().min(quality::interval(x, self.lower, self.upper))
    }
}
/// The binary column whose value activates a conditional row (ADR-0104).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Condition {
    pub column: usize,
    pub active: bool,
}
/// Whether a selected row is enforced at a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Enforcement {
    /// At every point.
    Always,
    /// Never: its indicator is fixed at the inactive value.
    Never,
    /// While the condition holds.
    When(Condition),
}
impl Enforcement {
    /// Whether the row is enforced at `x`; an indicator counts as set when it rounds to 1.
    pub(crate) fn enforced(self, x: &[f64]) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::When(c) => (x[c.column].round() == 1.0) == c.active,
        }
    }
}
/// One exported constraint `lower <= expression <= upper`, enforced while `condition` holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Function {
    pub expression: Expression,
    pub lower: f64,
    pub upper: f64,
    pub origin: Origin,
    pub condition: Option<Condition>,
}
/// An affine form over the combined coordinates: columns first, then auxiliaries, then
/// the indicators of lowered semi columns.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Affine {
    pub terms: Vec<(usize, f64)>,
    pub constant: f64,
}
impl Affine {
    #[cfg_attr(
        not(feature = "scip"),
        expect(dead_code, reason = "the SCIP adapter reads links back")
    )]
    fn at(&self, coordinates: &[f64]) -> f64 {
        self.terms
            .iter()
            .map(|(j, c)| c * coordinates[*j])
            .sum::<f64>()
            + self.constant
    }
}
/// The library-neutral export of one admitted program.
#[derive(Debug)]
pub(crate) struct Plan<'p> {
    pub program: &'p FactorableProgram,
    /// Exported constraints, in a deterministic order.
    pub constraints: Vec<Function>,
    /// Exported objective, when the intent optimizes and one is declared.
    pub objective: Option<(NodeId, ObjectiveSense)>,
    /// Rows without a projection, dropped from the export.
    pub dropped: usize,
    /// Worst fidelity of the export, counting dropped rows as a relaxation.
    pub fidelity: Fidelity,
    /// Closed boxes of the columns, then the auxiliaries, then the indicators of lowered
    /// semi columns; a binary column is `[0, 1]` intersected with its declaration, and a
    /// semi column `[0, u]`.
    pub boxes: Vec<(f64, f64)>,
    /// Affine form of every node, when it is affine and small enough.
    pub affine: Vec<Option<Affine>>,
    /// Semi columns lowered by `semi(indicator)`, in column order; the indicator of the
    /// one at ordinal `j` is coordinate [`Plan::indicator`]`(j)`.
    pub semi: Vec<Semi>,
    /// The `semi(indicator)` links, two per lowered semi column (upper side first).
    pub links: Vec<Affine>,
    /// Identity of the declared box and domains the backend branches over.
    pub domain: ContentHash,
    /// Enforcement of every selected row, in row order.
    pub rows: Vec<Enforcement>,
    /// Native constraints other than indicators, by ordinal in the program.
    pub native: Vec<usize>,
}
impl Plan<'_> {
    /// Some exported function or the objective is not affine.
    pub(crate) fn nonlinear(&self) -> bool {
        self.constraints
            .iter()
            .any(|c| self.form(c.expression).is_none())
            || self.objective.is_some_and(|o| self.affine[o.0].is_none())
    }
    /// A column has a discrete domain (integer, binary or semi), or a native form makes
    /// the program combinatorial.
    pub(crate) fn discrete(&self) -> bool {
        self.program.variables.iter().any(|v| v.domain.is_discrete())
            || !self.program.native.is_empty()
    }
    /// The affine form of an exported function, when it has one.
    pub(crate) fn form(&self, expression: Expression) -> Option<&Affine> {
        match expression {
            Expression::Node(node) => self.affine[node].as_ref(),
            Expression::Link(k) => Some(&self.links[k]),
        }
    }
    /// The coordinate of the indicator of the lowered semi column at ordinal `j`.
    #[cfg_attr(
        not(feature = "scip"),
        expect(dead_code, reason = "the SCIP adapter exports the indicators")
    )]
    pub(crate) fn indicator(&self, j: usize) -> usize {
        self.program.variables.len() + self.program.auxiliaries.len() + j
    }
    /// The lowered semi column of a program column, if any.
    pub(crate) fn semi_of(&self, column: usize) -> Option<&Semi> {
        self.semi
            .binary_search_by_key(&column, |s| s.column)
            .ok()
            .map(|j| &self.semi[j])
    }
    /// Every combined coordinate at a program point: the columns, the auxiliaries, and
    /// each lowered semi column's indicator at the branch nearest its value.
    #[cfg_attr(
        not(feature = "scip"),
        expect(dead_code, reason = "the SCIP adapter reads its export back")
    )]
    pub(crate) fn coordinates(&self, point: &[f64], auxiliary: &[f64]) -> Vec<f64> {
        point
            .iter()
            .chain(auxiliary)
            .copied()
            .chain(
                self.semi
                    .iter()
                    .map(|s| f64::from(u8::from(s.active(point[s.column])))),
            )
            .collect()
    }
    /// An exported function's value from the program's node values and the combined
    /// coordinates at one point.
    #[cfg_attr(
        not(feature = "scip"),
        expect(dead_code, reason = "the SCIP adapter reads its export back")
    )]
    pub(crate) fn value(&self, expression: Expression, nodes: &[f64], coordinates: &[f64]) -> f64 {
        match expression {
            Expression::Node(node) => nodes[node],
            Expression::Link(k) => self.links[k].at(coordinates),
        }
    }
    /// The named transformations of this export, in column order (ADR-0104 item 2).
    pub(crate) fn transformations(&self) -> Vec<ExportTransformation> {
        self.semi
            .iter()
            .map(|s| ExportTransformation::SemiIndicator {
                variable: self.program.variables[s.column].id,
                lower: s.lower,
                upper: s.upper,
                integer: self.program.variables[s.column].domain.is_integer(),
            })
            .collect()
    }
}
/// A native operand that must be binary: a binary column, or a fixed 0 or 1.
fn binary(program: &FactorableProgram, operand: NativeOperand) -> bool {
    match operand {
        NativeOperand::Column(c) => program.variables[c].domain == ModelingVariableDomain::Binary,
        NativeOperand::Fixed(v) => v == 0.0 || v == 1.0,
    }
}

fn close(c: &Constraint) -> (f64, f64) {
    let margin = |b: f64| STRICT_MARGIN * b.abs().max(1.0);
    let lower = if c.strict && c.lower.is_finite() {
        c.lower + margin(c.lower)
    } else {
        c.lower
    };
    let upper = if c.strict && c.upper.is_finite() {
        c.upper - margin(c.upper)
    } else {
        c.upper
    };
    (lower, upper)
}
fn column_box(domain: ModelingVariableDomain, lower: f64, upper: f64) -> (f64, f64) {
    if domain == ModelingVariableDomain::Binary {
        (lower.max(0.0), upper.min(1.0))
    } else if domain.is_semi() {
        (0.0, upper)
    } else {
        (lower, upper)
    }
}
/// Operands of a node, children first in the DAG.
pub(crate) fn children(node: &Node) -> &[NodeId] {
    match node {
        Node::Var(_) | Node::Aux(_) | Node::Const(_) => &[],
        Node::Sum(c) | Node::Product(c) => c,
        Node::Pow { base, .. } => std::slice::from_ref(base),
        Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
            std::slice::from_ref(i)
        }
    }
}
fn constant(form: &Affine) -> Option<f64> {
    form.terms.is_empty().then_some(form.constant)
}
fn scaled(form: &Affine, scale: f64) -> Affine {
    Affine {
        terms: form
            .terms
            .iter()
            .map(|(i, c)| (*i, c * scale))
            .filter(|(_, c)| *c != 0.0)
            .collect(),
        constant: form.constant * scale,
    }
}
/// Affine forms of every node, children first; `None` for a nonlinear node.
fn affine_forms(program: &FactorableProgram) -> Vec<Option<Affine>> {
    let columns = program.variables.len();
    let mut forms: Vec<Option<Affine>> = Vec::with_capacity(program.nodes.len());
    for node in &program.nodes {
        let unary = |i: &NodeId, f: fn(f64) -> f64, forms: &[Option<Affine>]| {
            forms[*i].as_ref().and_then(constant).map(|v| Affine {
                terms: vec![],
                constant: f(v),
            })
        };
        let form = match node {
            Node::Var(c) => Some(Affine {
                terms: vec![(*c, 1.0)],
                constant: 0.0,
            }),
            Node::Aux(k) => Some(Affine {
                terms: vec![(columns + k, 1.0)],
                constant: 0.0,
            }),
            Node::Const(c) => Some(Affine {
                terms: vec![],
                constant: c.value(),
            }),
            Node::Sum(c) => c
                .iter()
                .try_fold(BTreeMap::<usize, f64>::new(), |mut acc, i| {
                    let form = forms[*i].as_ref()?;
                    for (j, v) in &form.terms {
                        *acc.entry(*j).or_insert(0.0) += v;
                    }
                    (acc.len() <= AFFINE_TERMS).then_some(acc)
                })
                .map(|terms| Affine {
                    terms: terms.into_iter().filter(|(_, v)| *v != 0.0).collect(),
                    constant: c
                        .iter()
                        .filter_map(|i| forms[*i].as_ref().map(|f| f.constant))
                        .sum(),
                }),
            Node::Product(c) => {
                let mut factor = 1.0;
                let mut varying: Option<&Affine> = None;
                let mut linear = true;
                for i in c {
                    match forms[*i].as_ref() {
                        Some(f) if f.terms.is_empty() => factor *= f.constant,
                        Some(f) if varying.is_none() => varying = Some(f),
                        _ => linear = false,
                    }
                }
                if linear {
                    Some(varying.map_or(
                        Affine {
                            terms: vec![],
                            constant: factor,
                        },
                        |f| scaled(f, factor),
                    ))
                } else {
                    None
                }
            }
            Node::Pow { base, exponent } => match forms[*base].as_ref() {
                Some(f) if f.terms.is_empty() => Some(Affine {
                    terms: vec![],
                    constant: f.constant.powf(exponent.value()),
                }),
                Some(f) if exponent.value() == 1.0 => Some(f.clone()),
                _ => None,
            },
            Node::Exp(i) => unary(i, f64::exp, &forms),
            Node::Log(i) => unary(i, f64::ln, &forms),
            Node::Abs(i) => unary(i, f64::abs, &forms),
            Node::Sin(i) => unary(i, f64::sin, &forms),
            Node::Cos(i) => unary(i, f64::cos, &forms),
        };
        forms.push(form.filter(|f| f.constant.is_finite()));
    }
    forms
}

/// The export plan of an admitted program, or every typed refusal.
pub(crate) fn plan(
    program: &FactorableProgram,
    intent: SolveIntent,
) -> Result<Plan<'_>, Vec<Refusal>> {
    let mut refusals = Vec::new();
    let optimizes = matches!(intent, SolveIntent::Optimize | SolveIntent::Certify);
    let objective = match (&program.objective, optimizes) {
        (Some(o), true) => match o.expression {
            Some(node) if o.fidelity != Fidelity::Unavailable => Some((node, o.sense)),
            _ => {
                refusals.push(Refusal::ObjectiveUnavailable);
                None
            }
        },
        _ => None,
    };
    // Indicator rows are enforced conditionally; a fixed inactive indicator removes its
    // row exactly, and every other native form needs binary operands where it is logic.
    let mut enforcement = vec![Enforcement::Always; program.rows.len()];
    let mut native = Vec::new();
    for (k, n) in program.native.iter().enumerate() {
        match n {
            ProjectedNative::Indicator {
                row,
                indicator,
                active,
            } => {
                if !binary(program, *indicator) {
                    refusals.push(Refusal::NativeOperand(k));
                    continue;
                }
                let e = &mut enforcement[*row];
                *e = match (*e, *indicator) {
                    (Enforcement::Never, _) => Enforcement::Never,
                    (_, NativeOperand::Fixed(v)) if (v == 1.0) != *active => Enforcement::Never,
                    (e, NativeOperand::Fixed(_)) => e,
                    (Enforcement::Always, NativeOperand::Column(column)) => {
                        Enforcement::When(Condition {
                            column,
                            active: *active,
                        })
                    }
                    (Enforcement::When(_), NativeOperand::Column(_)) => {
                        refusals.push(Refusal::Conjunctive(program.rows[*row].id));
                        *e
                    }
                };
            }
            ProjectedNative::Logic {
                resultant,
                operands,
                ..
            } => {
                if !binary(program, *resultant)
                    || operands.iter().any(|(o, _)| !binary(program, *o))
                {
                    refusals.push(Refusal::NativeOperand(k));
                }
                native.push(k);
            }
            ProjectedNative::Sos { .. } | ProjectedNative::Cardinality { .. } => native.push(k),
        }
    }
    let mut constraints = Vec::new();
    let mut dropped = 0;
    let mut fidelity = Fidelity::Exact;
    for (r, row) in program.rows.iter().enumerate() {
        let condition = match enforcement[r] {
            Enforcement::Never => continue,
            Enforcement::Always => None,
            Enforcement::When(c) => Some(c),
        };
        match row.expression {
            Some(node) => {
                fidelity = fidelity.max(row.fidelity);
                constraints.push(Function {
                    expression: Expression::Node(node),
                    lower: row.lower,
                    upper: row.upper,
                    origin: Origin::Row(r),
                    condition,
                });
            }
            None => dropped += 1,
        }
    }
    if dropped > 0 {
        fidelity = fidelity.max(Fidelity::Relaxed);
    }
    for (o, obligation) in program.obligations.iter().enumerate() {
        if obligation.scope != pse_math::factorable::ObligationScope::Unconditional {
            continue;
        }
        fidelity = fidelity.max(obligation.fidelity);
        for (k, c) in obligation.constraints.iter().enumerate() {
            let (lower, upper) = close(c);
            constraints.push(Function {
                expression: Expression::Node(c.expression),
                lower,
                upper,
                origin: Origin::Obligation(o, k),
                condition: None,
            });
        }
    }
    for (b, block) in program.implicit.iter().enumerate() {
        fidelity = fidelity.max(block.fidelity);
        for (k, residual) in block.residuals.iter().enumerate() {
            constraints.push(Function {
                expression: Expression::Node(*residual),
                lower: 0.0,
                upper: 0.0,
                origin: Origin::Residual(b, k),
                condition: None,
            });
        }
        for (k, c) in block.bounds.iter().enumerate() {
            let (lower, upper) = close(c);
            constraints.push(Function {
                expression: Expression::Node(c.expression),
                lower,
                upper,
                origin: Origin::ImplicitBound(b, k),
                condition: None,
            });
        }
    }
    if let (Some(o), Some(_)) = (&program.objective, objective) {
        fidelity = fidelity.max(o.fidelity);
    }
    let columns = program.variables.len();
    // `semi(indicator)`: each semi column gets a binary indicator after the auxiliaries
    // and two exact links over its box [0, u].
    let mut semi = Vec::new();
    for (column, v) in program.variables.iter().enumerate() {
        if !v.domain.is_semi() {
            continue;
        }
        match v.active_lower {
            Some(lower) if lower > 0.0 && lower <= v.upper && v.upper.is_finite() => {
                semi.push(Semi {
                    column,
                    lower,
                    upper: v.upper,
                });
            }
            _ => refusals.push(Refusal::SemiInterval(v.id)),
        }
    }
    let first = columns + program.auxiliaries.len();
    let mut links = Vec::with_capacity(2 * semi.len());
    for (j, s) in semi.iter().enumerate() {
        for (upper, bound) in [(true, s.upper), (false, s.lower)] {
            constraints.push(Function {
                expression: Expression::Link(links.len()),
                lower: if upper { f64::NEG_INFINITY } else { 0.0 },
                upper: if upper { 0.0 } else { f64::INFINITY },
                origin: Origin::SemiLink { semi: j, upper },
                condition: None,
            });
            links.push(Affine {
                terms: vec![(s.column, 1.0), (first + j, -bound)],
                constant: 0.0,
            });
        }
    }
    let boxes: Vec<(f64, f64)> = program
        .variables
        .iter()
        .map(|v| column_box(v.domain, v.lower, v.upper))
        .chain(program.auxiliaries.iter().map(|a| (a.lower, a.upper)))
        .chain(semi.iter().map(|_| (0.0, 1.0)))
        .collect();
    // Reachable nodes, and those under a nonlinear operator, in one reverse pass each.
    let nodes = &program.nodes;
    let affine = affine_forms(program);
    let mut reachable = vec![false; nodes.len()];
    for root in constraints
        .iter()
        .filter_map(|c| match c.expression {
            Expression::Node(node) => Some(node),
            Expression::Link(_) => None,
        })
        .chain(objective.map(|o| o.0))
    {
        reachable[root] = true;
    }
    let mut nonlinear = vec![false; nodes.len()];
    for i in (0..nodes.len()).rev() {
        if !reachable[i] {
            continue;
        }
        let node = &nodes[i];
        if let Node::Const(c) = node
            && !c.value().is_finite()
        {
            refusals.push(Refusal::Constant(i));
        }
        if let Node::Pow { exponent, .. } = node
            && !exponent.value().is_finite()
        {
            refusals.push(Refusal::Constant(i));
        }
        // Sums and products with one varying factor pass linearity through; every other
        // operator over a varying operand makes its operands nonlinear.
        let varying = |c: &NodeId| affine[*c].as_ref().is_none_or(|f| !f.terms.is_empty());
        let operator = match node {
            Node::Var(_) | Node::Aux(_) | Node::Const(_) | Node::Sum(_) => false,
            Node::Product(c) => c.iter().filter(|c| varying(c)).count() > 1,
            Node::Pow { base, exponent } => exponent.value() != 1.0 && varying(base),
            Node::Exp(c) | Node::Log(c) | Node::Abs(c) | Node::Sin(c) | Node::Cos(c) => varying(c),
        };
        let inner = nonlinear[i] || operator;
        for &child in children(node) {
            reachable[child] = true;
            nonlinear[child] |= inner;
        }
    }
    let mut unbounded = BTreeMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if !(reachable[i] && nonlinear[i]) {
            continue;
        }
        let (owner, index) = match node {
            Node::Var(c) => (BoundOwner::Variable(program.variables[*c].id), *c),
            Node::Aux(k) => (BoundOwner::Auxiliary(*k), columns + k),
            _ => continue,
        };
        let (lower, upper) = boxes[index];
        if !lower.is_finite() || !upper.is_finite() {
            unbounded.insert(
                index,
                MissingBound {
                    owner,
                    lower: !lower.is_finite(),
                    upper: !upper.is_finite(),
                },
            );
        }
    }
    refusals.extend(unbounded.into_values().map(Refusal::UnboundedNonlinear));
    if !refusals.is_empty() {
        return Err(refusals);
    }
    let mut h = FramedHasher::new(pse_ids::Frame::FactorableDomainV1);
    h.hash(&program.key);
    for (v, (lower, upper)) in program.variables.iter().zip(&boxes) {
        h.id(&v.id)
            .u64(v.domain as u64)
            .u64(lower.to_bits())
            .u64(upper.to_bits());
        // A semi column's active interval is part of the domain it branches over.
        if let Some(active) = v.active_lower.filter(|_| v.domain.is_semi()) {
            h.u64(active.to_bits());
        }
    }
    for (lower, upper) in &boxes[columns..first] {
        h.u64(lower.to_bits()).u64(upper.to_bits());
    }
    Ok(Plan {
        program,
        constraints,
        objective,
        dropped,
        fidelity,
        boxes,
        affine,
        domain: h.finish_hash(),
        rows: enforcement,
        native,
        semi,
        links,
    })
}
/// Every typed refusal of exporting `program` for `intent`; empty means admitted.
pub fn admit_program(program: &FactorableProgram, intent: SolveIntent) -> Vec<Refusal> {
    plan(program, intent).err().unwrap_or_default()
}
fn refused(refusals: &[Refusal]) -> ProblemError {
    let reasons: Vec<String> = refusals.iter().map(ToString::to_string).collect();
    ProblemError::Unsupported(format!("factorable export refused: {}", reasons.join("; ")))
}

/// Builds original-coordinate NLP callbacks over every column, with the given column
/// indices committed to the given closed boxes: a degenerate box `[v, v]` fixes a discrete
/// column at `v`, and a semicontinuous column on its active branch keeps its active
/// interval `[l, u]`.
pub type FixedOracle<'a> =
    dyn FnMut(&BTreeMap<usize, (f64, f64)>) -> Result<Box<dyn NlpOracle>, ProblemError> + 'a;
/// A continuous re-solve with the discrete columns fixed (ADR-0105 §2, T07).
pub struct Resolve<'a> {
    /// The fixed-assignment callbacks.
    pub oracle: &'a mut FixedOracle<'a>,
    /// Qualified library preprocessing policy of the re-solve.
    pub presolve: &'a presolve::Policy,
    /// Presolve dimension ceiling.
    pub limit: usize,
    /// The sensitivity request of an optimizing re-solve, conditional on its assignment
    /// like its multipliers (ADR-0118 item 4).
    pub sensitivity: Option<ResolveSensitivity<'a>>,
}
impl std::fmt::Debug for Resolve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolve")
            .field("limit", &self.limit)
            .field("sensitivity", &self.sensitivity)
            .finish_non_exhaustive()
    }
}
/// A parametric sensitivity request of the fixed-assignment re-solve (Plan 22 S1).
pub struct ResolveSensitivity<'a> {
    /// Builds the parametric callbacks ([`crate::kkt::Sensitivity::oracle`]) with the given
    /// columns fixed at the given values, as the re-solve's callbacks are.
    pub oracle: &'a mut FixedOracle<'a>,
    /// Each parameter's identity and value, in request order.
    pub parameters: Vec<(SemanticId, f64)>,
    /// Also compute the reduced Hessian over the parameters.
    pub reduced_hessian: bool,
}
impl std::fmt::Debug for ResolveSensitivity<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolveSensitivity")
            .field("parameters", &self.parameters)
            .field("reduced_hessian", &self.reduced_hessian)
            .finish_non_exhaustive()
    }
}
/// One factorable run.
pub struct Factorable<'a> {
    /// Program projected from the prepared case under the step's values.
    pub program: &'a FactorableProgram,
    /// Original start in program column order.
    pub initial: &'a [f64],
    /// Mathematical purpose; only optimization intents export the objective.
    pub intent: SolveIntent,
    /// The original compiled model every candidate is re-checked against.
    pub original: &'a mut dyn OriginalModel,
    /// The fixed-assignment re-solve; without it, a relaxed or unqualified incumbent
    /// remains an observation.
    pub resolve: Option<Resolve<'a>>,
}
impl std::fmt::Debug for Factorable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Factorable")
            .field("intent", &self.intent)
            .field("resolve", &self.resolve)
            .finish_non_exhaustive()
    }
}

/// Export, native global solve, readback, original re-qualification and the candidate
/// rule of ADR-0105 §2:
/// - an exact export's incumbent is the candidate when it meets the original tolerances
///   and the program is not mixed-integer nonlinear;
/// - otherwise the candidate comes from the fixed-assignment continuous re-solve through
///   the one NLP runner, started at the incumbent, and the backend's incumbent stays an
///   assignment proposal;
/// - without a re-solve, a relaxed export's incumbent is an observation only.
///
/// The dual bound and the candidate are recorded with their sources; `quality` grants a
/// gap only from both.
///
/// # Errors
/// A typed export refusal, or native execution failed before a report existed.
pub fn factorable(
    step: Step<'_>,
    retained: &mut Retained,
    run: Factorable<'_>,
) -> Result<SolveReport, ProblemError> {
    let plan = plan(run.program, run.intent).map_err(|r| refused(&r))?;
    if run.initial.len() != run.program.variables.len() {
        return Err(ProblemError::Contract("factorable start dimensions".into()));
    }
    let mut report = step.adapter.execute(
        retained,
        Input {
            problem: Problem::Factorable {
                program: run.program,
                initial: run.initial,
                intent: run.intent,
                normalization: step.normalization,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            execution: step.execution.clone(),
            tolerances: step.tolerances,
            warm: step.warm,
            compatibility: step.compatibility.clone(),
        },
    )?;
    observe(&mut report, &plan, run.original, step.tolerances);
    // Every pooled solution is re-qualified in original coordinates, like the candidate,
    // and the record states the named transformations the plan applied.
    if let Some(record) = report.global.as_mut() {
        let record = std::sync::Arc::make_mut(record);
        record.transformations = plan.transformations();
        for solution in &mut record.pool {
            solution.feasible = assess(&plan, run.original, step.tolerances, &solution.primal)
                .ok()
                .map(|(q, _)| q.feasible());
        }
    }
    report.metrics.insert(
        "export.rows.dropped".into(),
        Metric::Integer(i64::try_from(plan.dropped).unwrap_or(i64::MAX)),
    );
    report.metrics.insert(
        "export.lowered.semi_indicator".into(),
        Metric::Integer(i64::try_from(plan.semi.len()).unwrap_or(i64::MAX)),
    );
    if let Some(mut global) = report.evidence.global {
        // The runner's plan is the authority for what was exported.
        global.fidelity = plan.fidelity;
        global.domain = plan.domain;
        let feasible = report.validation_failure().is_none()
            && report.quality.as_ref().is_some_and(Quality::feasible);
        let proposal =
            global.fidelity != Fidelity::Exact || plan.discrete() && plan.nonlinear() || !feasible;
        global.primal = if global.fidelity == Fidelity::Exact {
            PrimalSource::Backend
        } else {
            PrimalSource::RelaxedIncumbent
        };
        if report.candidate.is_some()
            && proposal
            && let Some(resolve) = run.resolve
        {
            let objective = plan.objective.is_some();
            match fixed_assignment(&step, &report, &plan, resolve, objective) {
                Ok(resolved) => {
                    if adopt(&mut report, resolved, &plan, run.original, &step) {
                        global.primal = PrimalSource::FixedAssignment;
                    }
                }
                Err(e) => {
                    report
                        .metrics
                        .insert("resolve.refused".into(), Metric::Text(e.to_string()));
                }
            }
        }
        global.dual = if global.fidelity == Fidelity::Exact {
            BoundSource::ExactExport
        } else {
            BoundSource::RelaxedExport
        };
        report.evidence.global = Some(global);
    }
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}

/// Fresh original values, the plan's declared boxes and integrality at the reported
/// candidate.
fn observe(
    report: &mut SolveReport,
    plan: &Plan<'_>,
    original: &mut dyn OriginalModel,
    tolerances: &quality::Tolerances,
) {
    let Some(candidate) = report.candidate.as_ref() else {
        return;
    };
    match assess(plan, original, tolerances, &candidate.primal) {
        Ok((q, o)) => {
            report.quality = Some(q);
            report.observation = Some(o);
            report.clear_validation_failure();
        }
        Err(e) => {
            report.quality = None;
            report.observation = None;
            report.record_validation_failure(e);
        }
    }
}
/// Original-coordinate quality at `x`: rows enforced there against their bounds (an
/// inactive indicator row is unconstrained), the declared box (for a semi column, its
/// domain `{0} ∪ [l, u]`), integrality and every native form's discrete structure.
pub(crate) fn assess(
    plan: &Plan<'_>,
    original: &mut dyn OriginalModel,
    tolerances: &quality::Tolerances,
    x: &[f64],
) -> Result<(Quality, Observation), ProblemError> {
    let program = plan.program;
    quality::contained(|| {
        tolerances.validate(program.variables.len(), program.rows.len())?;
        if x.len() != program.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("invalid factorable candidate"));
        }
        let fresh = original.evaluate(x)?;
        let limits: Vec<(f64, f64)> = program
            .rows
            .iter()
            .zip(&plan.rows)
            .map(|(r, e)| {
                if e.enforced(x) {
                    (r.lower, r.upper)
                } else {
                    (f64::NEG_INFINITY, f64::INFINITY)
                }
            })
            .collect();
        let rows = program
            .rows
            .iter()
            .zip(&fresh.constraints)
            .zip(&limits)
            .zip(&tolerances.rows)
            .map(|(((r, v), (lower, upper)), t)| Violation {
                id: r.id,
                physical: quality::interval(*v, *lower, *upper),
                tolerance: *t,
            })
            .collect();
        let mut observation = Observation::from_values(fresh.objective, fresh.constraints, limits)?;
        observation.sources = fresh.sources;
        let mut bounds = Vec::with_capacity(x.len());
        let mut integrality = Vec::new();
        for (column, (((v, x), t), (lower, upper))) in program
            .variables
            .iter()
            .zip(x)
            .zip(&tolerances.variables)
            .zip(&plan.boxes)
            .enumerate()
        {
            bounds.push(Violation {
                id: v.id,
                physical: plan.semi_of(column).map_or_else(
                    || quality::interval(*x, *lower, *upper),
                    |s| s.violation(*x),
                ),
                tolerance: *t,
            });
            if v.domain.is_integer() {
                integrality.push(Violation {
                    id: v.id,
                    physical: (x - x.round()).abs(),
                    tolerance: tolerances.integrality,
                });
            }
        }
        integrality.extend(native_violations(plan, tolerances, x));
        Ok((Quality::new(rows, bounds, integrality)?, observation))
    })
}
/// Dimensionless discrete-structure violations of the native forms at `x`: the excess
/// nonzero count of an SOS or cardinality set (SOS2 members must also be adjacent), and
/// a logic resultant that differs from its operands' value.
fn native_violations(
    plan: &Plan<'_>,
    tolerances: &quality::Tolerances,
    x: &[f64],
) -> Vec<Violation> {
    use pse_model::generated::enums::NativeConstraintForm as F;
    let program = plan.program;
    let value = |o: &NativeOperand| match o {
        NativeOperand::Column(c) => x[*c],
        NativeOperand::Fixed(v) => *v,
    };
    let tolerance = |o: &NativeOperand| match o {
        NativeOperand::Column(c) => tolerances.variables[*c],
        NativeOperand::Fixed(_) => tolerances.integrality,
    };
    // The first column member names the violation; an all-fixed form has no identity.
    fn owner<'o>(
        program: &FactorableProgram,
        mut ops: impl Iterator<Item = &'o NativeOperand>,
    ) -> SemanticId {
        ops.find_map(|o| match o {
            NativeOperand::Column(c) => Some(program.variables[*c].id),
            NativeOperand::Fixed(_) => None,
        })
        .unwrap_or(SemanticId::NIL)
    }
    let nonzero = |o: &NativeOperand| value(o).abs() > tolerance(o);
    plan.native
        .iter()
        .filter_map(|k| {
            let (excess, owner) = match &program.native[*k] {
                ProjectedNative::Indicator { .. } => return None,
                ProjectedNative::Sos { form, members } => {
                    let set: Vec<usize> = (0..members.len())
                        .filter(|i| nonzero(&members[*i].0))
                        .collect();
                    let allowed = if *form == F::Sos2 { 2 } else { 1 };
                    let mut excess = set.len().saturating_sub(allowed);
                    if *form == F::Sos2 && set.len() == 2 && set[1] != set[0] + 1 {
                        excess = 1;
                    }
                    (excess, owner(program, members.iter().map(|(o, _)| o)))
                }
                ProjectedNative::Cardinality { members, bound } => (
                    members
                        .iter()
                        .filter(|o| nonzero(o))
                        .count()
                        .saturating_sub(*bound as usize),
                    owner(program, members.iter()),
                ),
                ProjectedNative::Logic {
                    form,
                    resultant,
                    operands,
                } => {
                    let literal = |(o, negated): &(NativeOperand, bool)| {
                        (value(o).round() == 1.0) != *negated
                    };
                    let truth = match form {
                        F::And => operands.iter().all(literal),
                        F::Or => operands.iter().any(literal),
                        _ => operands.iter().filter(|o| literal(o)).count() % 2 == 1,
                    };
                    (
                        usize::from((value(resultant).round() == 1.0) != truth),
                        owner(program, std::iter::once(resultant)),
                    )
                }
            };
            Some(Violation {
                id: owner,
                physical: excess as f64,
                tolerance: tolerances.integrality,
            })
        })
        .collect()
}

/// The continuous problem of the backend's discrete assignment, solved through the one NLP
/// runner by the automatic NLP route and seeded there: every integer column is fixed at its
/// rounded incumbent value; a semi column on its zero branch is fixed at zero, and on its
/// active branch kept in `[l, u]` (a semi-integer one fixed at its rounded value); every
/// SOS or cardinality member at zero stays zero, and a row whose indicator is inactive
/// under the assignment is unconstrained. The re-solve's native state is its own and never
/// replaces a retained global session.
fn fixed_assignment(
    step: &Step<'_>,
    report: &SolveReport,
    plan: &Plan<'_>,
    resolve: Resolve<'_>,
    objective: bool,
) -> Result<SolveReport, ProblemError> {
    let program = plan.program;
    let incumbent = report
        .candidate
        .as_ref()
        .ok_or_else(|| ProblemError::Internal("fixed assignment without an incumbent".into()))?;
    let mut assignment: BTreeMap<usize, (f64, f64)> = BTreeMap::new();
    for (i, v) in program.variables.iter().enumerate() {
        let x = incumbent.primal[i];
        let committed = match plan.semi_of(i) {
            Some(s) if !s.active(x) => Some((0.0, 0.0)),
            Some(s) if v.domain.is_integer() => {
                let value = x.round().clamp(s.lower, s.upper);
                Some((value, value))
            }
            Some(s) => Some((s.lower, s.upper)),
            None if v.domain.is_integer() => Some((x.round(), x.round())),
            None => None,
        };
        if let Some(committed) = committed {
            assignment.insert(i, committed);
        }
    }
    for k in &plan.native {
        let members: Vec<NativeOperand> = match &program.native[*k] {
            ProjectedNative::Sos { members, .. } => members.iter().map(|(o, _)| *o).collect(),
            ProjectedNative::Cardinality { members, .. } => members.clone(),
            ProjectedNative::Indicator { .. } | ProjectedNative::Logic { .. } => continue,
        };
        for member in members {
            if let NativeOperand::Column(c) = member
                && incumbent.primal[c].abs() <= step.tolerances.variables[c]
            {
                assignment.insert(c, (0.0, 0.0));
            }
        }
    }
    let mut start = incumbent.primal.clone();
    for (i, (lower, upper)) in &assignment {
        start[*i] = start[*i].clamp(*lower, *upper);
    }
    let mut oracle = (resolve.oracle)(&assignment)?;
    let relaxed: Vec<usize> = plan
        .rows
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.enforced(&start))
        .map(|(r, _)| r)
        .collect();
    let unconstrained = |oracle: Box<dyn NlpOracle>| -> Result<Box<dyn NlpOracle>, ProblemError> {
        Ok(if relaxed.is_empty() {
            oracle
        } else {
            Box::new(crate::transform::Unconstrained::new(oracle, &relaxed)?)
        })
    };
    oracle = unconstrained(oracle)?;
    let equalities = oracle
        .constraint_bounds()
        .iter()
        .all(|(l, u)| l.is_finite() && l == u);
    let facts = routing::oracle_facts(oracle.contract(), objective, equalities);
    let intent = if objective {
        SolveIntent::Optimize
    } else {
        SolveIntent::FeasiblePoint
    };
    // Native options belong to the global backend; the re-solve uses NLP defaults.
    let controls = Controls {
        options: Options::new(),
        threads: 1,
        ..step.controls.clone()
    };
    let Route::Native(backend) = (Requirements {
        table: &LINKED,
        facts: &facts,
        intent,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &BackendSettings::Default,
    })
    .select(SolverSelection::Auto)?
    else {
        return Err(ProblemError::Internal(
            "fixed-assignment re-solve has no free column".into(),
        ));
    };
    let adapter: &dyn BackendExecution = super::adapter(backend);
    if adapter.representation() != super::Representation::Nlp {
        return Err(ProblemError::Unsupported(format!(
            "fixed-assignment re-solve routed to {}, which is not an NLP adapter",
            backend.as_str()
        )));
    }
    let mut layout = FramedHasher::new(pse_ids::Frame::FactorableFixedAssignmentV1);
    layout.hash(&step.compatibility.layout);
    for (i, (lower, upper)) in &assignment {
        layout
            .u64(*i as u64)
            .u64(lower.to_bits())
            .u64(upper.to_bits());
    }
    // The re-solve's native profile is its own: default settings of its NLP adapter under
    // the step's profile.
    let mut profile = FramedHasher::new(pse_ids::Frame::FactorableFixedAssignmentProfileV1);
    profile
        .hash(&step.compatibility.profile)
        .str(backend.as_str());
    let compatibility = Compatibility {
        layout: layout.finish_hash(),
        profile: profile.finish_hash(),
        data: step.compatibility.data,
        backend,
    };
    let seed = WarmStart {
        origin: None,
        compatibility: compatibility.clone(),
        payload: adapter.primal_start(start.clone())?,
    };
    // A feasibility re-solve has no objective to differentiate. Parametric callbacks that
    // cannot be built withhold the sensitivities; they never refuse the re-solve.
    let (sensitivity, unbuilt) = match resolve.sensitivity {
        Some(request) if intent == SolveIntent::Optimize => {
            match (request.oracle)(&assignment).and_then(unconstrained) {
                Ok(oracle) => (
                    Some(crate::kkt::Sensitivity {
                        oracle,
                        parameters: request.parameters,
                        reduced_hessian: request.reduced_hessian,
                    }),
                    None,
                ),
                Err(cause) => (
                    None,
                    Some(crate::kkt::Parametric::withheld(
                        request.parameters.iter().map(|(id, _)| *id).collect(),
                        request.reduced_hessian,
                        crate::kkt::Withheld::Analysis(cause.into()),
                    )),
                ),
            }
        }
        _ => (None, None),
    };
    nlp(
        Step {
            adapter,
            settings: &BackendSettings::Default,
            controls: &controls,
            accuracy: step.accuracy,
            execution: step.execution.clone(),
            tolerances: step.tolerances,
            normalization: step.normalization,
            compatibility,
            warm: Some(&seed),
        },
        &mut Retained::default(),
        Nlp {
            oracle,
            initial: &start,
            presolve: resolve.presolve,
            intent,
            sense: program
                .objective
                .as_ref()
                .map_or(ObjectiveSense::Minimize, |o| o.sense),
            limit: resolve.limit,
            // The standing analysis of the re-solve's purpose and the requested
            // sensitivities; its point, like its multipliers, is conditional on the
            // assignment.
            analysis: super::Analysis {
                sensitivity,
                ..super::Analysis::for_intent(intent)
            },
        },
    )
    .map(|mut report| {
        if unbuilt.is_some() {
            report.evidence.sensitivity = unbuilt;
        }
        report
    })
}
/// Adopt the re-solve's candidate when it met its tolerances, re-observed against the
/// original model with the declared box and integrality; returns whether it was adopted.
/// The backend's incumbent objective is kept as a metric.
fn adopt(
    report: &mut SolveReport,
    resolved: SolveReport,
    plan: &Plan<'_>,
    original: &mut dyn OriginalModel,
    step: &Step<'_>,
) -> bool {
    report.metrics.insert(
        "resolve.backend".into(),
        Metric::Text(resolved.backend.as_str().into()),
    );
    report.metrics.insert(
        "resolve.termination".into(),
        Metric::Text(resolved.termination.category.as_str().into()),
    );
    report.metrics.insert(
        "resolve.qualification".into(),
        Metric::Text(resolved.qualification.as_str().into()),
    );
    let usable = resolved.validation_failure().is_none()
        && resolved.quality.as_ref().is_some_and(Quality::feasible)
        && resolved.candidate.is_some();
    if !usable {
        return false;
    }
    if let Some(incumbent) = &report.candidate
        && let Some(value) = incumbent.objective
    {
        report
            .metrics
            .insert("scip.incumbent.objective".into(), Metric::Real(value));
    }
    let local = resolved.observation.clone();
    report.candidate = resolved.candidate;
    // A later global step is seeded with the adopted candidate, not the proposal.
    if let (Some(seed), Some(candidate)) = (report.warm_start.as_mut(), &report.candidate) {
        seed.payload = WarmPayload::primal(candidate.primal.clone());
    }
    report.evidence.kkt = resolved.evidence.kkt;
    // The re-solve analysed the original rows and columns of this report; its local
    // analysis is conditional on the assignment, like its multipliers.
    let aligned = resolved.rows == report.rows && resolved.variables == report.variables;
    let misaligned = || {
        crate::kkt::Unavailable::Failed(std::sync::Arc::new(ProblemError::internal(
            "the re-solve's rows or columns differ from the program's",
        )))
    };
    report.evidence.local = if aligned {
        resolved.evidence.local
    } else {
        Some(Err(misaligned()))
    };
    // Its sensitivities are conditional on the assignment in the same way.
    report.evidence.sensitivity = if aligned {
        resolved.evidence.sensitivity
    } else {
        resolved.evidence.sensitivity.map(|p| {
            let withheld = crate::kkt::Withheld::Analysis(misaligned());
            crate::kkt::Parametric {
                point: None,
                sensitivities: Err(withheld.clone()),
                reduced_hessian: p.reduced_hessian.map(|_| Err(withheld)),
                ..p
            }
        })
    };
    report.preprocessing = resolved.preprocessing;
    observe(report, plan, original, step.tolerances);
    // Multipliers of the fixed-assignment problem are conditional on the assignment.
    if let (Some(observation), Some(local)) = (report.observation.as_mut(), local) {
        observation.stationarity = local.stationarity;
        observation.complementarity = local.complementarity;
        observation.dual_error = local.dual_error;
    }
    true
}
